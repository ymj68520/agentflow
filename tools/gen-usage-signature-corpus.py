#!/usr/bin/env python3
"""Freeze the Phase 4A.5A static Usage conformance corpus.

The Usage oracle is independent of crates/af-usage. It implements only the
declaration canonicalization, static envelope algebra, conservative control
propagation and non-increase proof required by the normative Usage document.
It never measures runtime consumption, consults pricing, or creates ledger
state.
"""

import importlib.util
import json
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CONF = ROOT / "spec" / "conformance" / "usage-signature"
CORPUS_REVISION = 1

_sir_spec = importlib.util.spec_from_file_location(
    "sir", Path(__file__).resolve().parent / "gen-semantic-ir-corpus.py")
sir = importlib.util.module_from_spec(_sir_spec)
_sir_spec.loader.exec_module(sir)
afidl = sir.afidl

SEMANTIC_CORE = ROOT / "spec" / "idl" / "semantic-core.afidl"
USAGE_AFDL = ROOT / "spec" / "idl" / "usage-signature.afidl"

INVALID_USAGE_METER = "invalid_usage_meter"
UNKNOWN_USAGE_METER = "unknown_usage_meter"
DUPLICATE_USAGE_LIMIT = "duplicate_usage_limit"
UNSUPPORTED_USAGE_AGGREGATION = "unsupported_usage_aggregation"
USAGE_ARITHMETIC_OVERFLOW = "usage_arithmetic_overflow"
USAGE_BOUND_UNPROVEN = "usage_bound_unproven"
USAGE_CEILING_EXCEEDED = "usage_ceiling_exceeded"

METER_CATALOG = "agentflow.test.usage.meters@1"
UNIT_CATALOG = "agentflow.test.usage.units@1"
METER_DATA = [
    ("model.input_tokens", "tokens"),
    ("model.output_tokens", "tokens"),
    ("tool.invocations", "calls"),
    ("data.egress_bytes", "bytes"),
]


def fixture_catalogs(meters=None):
    meters = meters or METER_DATA
    return {
        "meters": [
            {
                "catalog": METER_CATALOG,
                "name": name,
                "unit": {"catalog": UNIT_CATALOG, "name": unit},
                "aggregation": "additive",
            }
            for name, unit in meters
        ]
    }


def load_shape():
    sources = [
        ("semantic-core.afidl", SEMANTIC_CORE.read_text(encoding="utf-8")),
        ("usage-signature.afidl", USAGE_AFDL.read_text(encoding="utf-8")),
    ]
    _modules, types_by_module = afidl.compile_module_set(sources)
    types = {}
    for spec, schemas in types_by_module.items():
        for schema in schemas:
            types[f"{spec}/{schema['name']}"] = schema
    return types


FLOW_TYPES = load_shape()


def shape_of_identity(identity):
    kind, payload = FLOW_TYPES[identity]["body"]
    if kind == "alias":
        return expr_shape(payload)
    if kind == "enum":
        return ("enum", identity, {v["name"] for v in payload["values"]})
    if kind == "record":
        return ("record", identity,
                [(f["name"], expr_shape(f["type"])) for f in payload["fields"]])
    if kind == "variant":
        return ("variant", identity,
                [(c["name"], expr_shape(c["type"])) for c in payload["cases"]])
    raise AssertionError(f"unknown type body {kind!r}")


def expr_shape(texpr):
    kind, value = texpr
    if kind == "primitive":
        return sir.PRIM_SHAPES[value]
    if kind == "ref":
        return shape_of_identity(value)
    if kind == "list":
        return ("list", expr_shape(value))
    if kind == "map_string":
        return ("map", expr_shape(value))
    if kind == "option":
        return ("option", expr_shape(value))
    raise AssertionError(f"unknown type expression {kind!r}")


SIGNATURE_SHAPE = shape_of_identity(
    "agentflow.usage.signature@1/UsageSignature")


def to_value_tree(shape, value):
    kind = shape[0]
    if kind == "variant":
        if isinstance(value, str):
            return {value: None}
        case, payload = next(iter(value.items()))
        case_shape = dict(shape[2])[case]
        return {case: to_value_tree(case_shape, payload)}
    if kind == "enum":
        return value
    if kind == "record":
        return {name: to_value_tree(field_shape, value[name])
                for name, field_shape in shape[2]}
    if kind == "list":
        return [to_value_tree(shape[1], item) for item in value]
    return value


def canonical_ace1(signature):
    tree = to_value_tree(SIGNATURE_SHAPE, signature)
    return sir.encode_value(SIGNATURE_SHAPE,
                            sir.json_to_value(SIGNATURE_SHAPE, tree))


class UsageOracleError(Exception):
    def __init__(self, kind, detail):
        super().__init__(f"{kind}: {detail}")
        self.kind = kind


def nfc(text):
    return unicodedata.normalize("NFC", text)


def specification_id(text):
    if "@" not in text:
        return False
    namespace, _, major = text.rpartition("@")
    if not namespace or not major:
        return False
    for segment in namespace.split("."):
        if not segment or segment[0] not in "abcdefghijklmnopqrstuvwxyz":
            return False
        if not all(c.isascii() and (c.isalnum() or c == "-") for c in segment):
            return False
    return major.isascii() and major.isdigit() and major[0] != "0"


def check_meter(meter):
    if not specification_id(meter["catalog"]):
        raise UsageOracleError(INVALID_USAGE_METER, "invalid meter catalog")
    name = nfc(meter["name"])
    if not name or any(c.isspace() for c in name):
        raise UsageOracleError(INVALID_USAGE_METER, "invalid meter name")
    return {"catalog": meter["catalog"], "name": name}


def catalog_map(catalogs):
    return {
        (m["catalog"], m["name"]): m for m in catalogs["meters"]
    }


def resolve_meter(meter, catalogs):
    normalized = check_meter(meter)
    descriptor = catalog_map(catalogs).get((normalized["catalog"], normalized["name"]))
    if descriptor is None:
        raise UsageOracleError(UNKNOWN_USAGE_METER, "meter is not recognized")
    if descriptor["aggregation"] != "additive":
        raise UsageOracleError(UNSUPPORTED_USAGE_AGGREGATION,
                               "meter aggregation is not additive")
    return normalized


def upper_key(upper):
    if "finite" in upper:
        return ("finite", upper["finite"])
    return ("unbounded", None)


def canonicalize(signature, catalogs):
    seen = set()
    limits = []
    for limit in signature["limits"]:
        meter = resolve_meter(limit["meter"], catalogs)
        key = (limit["phase"], meter["catalog"], meter["name"])
        if key in seen:
            raise UsageOracleError(DUPLICATE_USAGE_LIMIT, "duplicate phase/meter")
        seen.add(key)
        limits.append({"meter": meter, "phase": limit["phase"],
                       "upper": limit["upper"]})
    phase_no = {"forward": 1, "recovery": 2}
    limits.sort(key=lambda x: (phase_no[x["phase"]],
                               x["meter"]["catalog"].encode(),
                               x["meter"]["name"].encode()))
    return {"limits": limits}


def env_key(phase, meter):
    return (phase, meter["catalog"], meter["name"])


def envelope(bounds=(), global_unknown=False):
    return {"global_unknown": global_unknown,
            "bounds": sorted(
                [{"phase": phase, "meter": meter, "bound": bound}
                 for phase, meter, bound in bounds],
                key=lambda x: ({"forward": 1, "recovery": 2}[x["phase"]],
                               x["meter"]["catalog"], x["meter"]["name"]))}


def declared_envelope(signature, catalogs):
    canonical = canonicalize(signature, catalogs)
    return envelope((limit["phase"], limit["meter"],
                     {"kind": "known_finite", "value": limit["upper"]["finite"]}
                     if "finite" in limit["upper"] else
                     {"kind": "known_unbounded"})
                    for limit in canonical["limits"])


def env_map(value):
    return {env_key(x["phase"], x["meter"]): x["bound"]
            for x in value["bounds"]}


def env_get(value, key):
    return env_map(value).get(key, {"kind": "unknown"})


def upper_from_bound(bound):
    if bound["kind"] == "known_finite":
        return ("finite", bound["value"])
    if bound["kind"] == "known_unbounded":
        return ("unbounded", None)
    return None


def bound_from_upper(upper):
    if upper[0] == "finite":
        return {"kind": "known_finite", "value": upper[1]}
    return {"kind": "known_unbounded"}


def all_keys(a, b):
    return set(env_map(a)) | set(env_map(b))


def accumulation(a, b):
    if a["global_unknown"] or b["global_unknown"]:
        return envelope(global_unknown=True)
    bounds = []
    maps = (env_map(a), env_map(b))
    for key in sorted(all_keys(a, b)):
        left, right = (m.get(key, {"kind": "unknown"}) for m in maps)
        lupper, rupper = upper_from_bound(left), upper_from_bound(right)
        if lupper is None or rupper is None:
            bound = {"kind": "unknown"}
        elif "unbounded" in (lupper[0], rupper[0]):
            bound = {"kind": "known_unbounded"}
        else:
            value = lupper[1] + rupper[1]
            if value > 2**64 - 1:
                raise UsageOracleError(USAGE_ARITHMETIC_OVERFLOW, "addition overflow")
            bound = {"kind": "known_finite", "value": value}
        bounds.append((key[0], {"catalog": key[1], "name": key[2]}, bound))
    return envelope(bounds)


def join(a, b):
    if a["global_unknown"] or b["global_unknown"]:
        return envelope(global_unknown=True)
    bounds = []
    for key in sorted(all_keys(a, b)):
        left, right = env_get(a, key), env_get(b, key)
        lu, ru = upper_from_bound(left), upper_from_bound(right)
        if lu is None or ru is None:
            bound = {"kind": "unknown"}
        elif lu[0] == "unbounded" or ru[0] == "unbounded":
            bound = {"kind": "known_unbounded"}
        else:
            bound = {"kind": "known_finite", "value": max(lu[1], ru[1])}
        bounds.append((key[0], {"catalog": key[1], "name": key[2]}, bound))
    return envelope(bounds)


def repeat(count, value):
    if count == 0:
        return envelope((x["phase"], x["meter"],
                         {"kind": "known_finite", "value": 0})
                        for x in value["bounds"])
    if value["global_unknown"]:
        return envelope(global_unknown=True)
    bounds = []
    for item in value["bounds"]:
        bound = item["bound"]
        if bound["kind"] == "unknown":
            output = bound
        elif bound["kind"] == "known_unbounded":
            output = bound
        else:
            value_ = bound["value"] * count
            if value_ > 2**64 - 1:
                raise UsageOracleError(USAGE_ARITHMETIC_OVERFLOW, "multiplication overflow")
            output = {"kind": "known_finite", "value": value_}
        bounds.append((item["phase"], item["meter"], output))
    return envelope(bounds)


def within_ceiling(observed, declared):
    if observed["global_unknown"] or declared["global_unknown"]:
        return {"outcome": "unproven", "reason": "unknown_bound"}
    keys = all_keys(observed, declared)
    if not keys:
        return {"outcome": "unproven", "reason": "missing_meter"}
    for key in sorted(keys):
        actual = upper_from_bound(env_get(observed, key))
        limit = upper_from_bound(env_get(declared, key))
        if actual is None or limit is None:
            return {"outcome": "unproven", "reason": "missing_meter"}
        if limit[0] == "finite" and (actual[0] == "unbounded" or actual[1] > limit[1]):
            return {"outcome": "violation"}
    return {"outcome": "proven"}


def usage_not_increased(new, old, new_catalogs, old_catalogs):
    new_c, old_c = canonicalize(new, new_catalogs), canonicalize(old, old_catalogs)
    if not new_c["limits"] or not old_c["limits"]:
        return {"outcome": "unproven", "reason": "missing_meter"}
    def keyed(c):
        return {env_key(x["phase"], x["meter"]): x for x in c["limits"]}
    new_map, old_map = keyed(new_c), keyed(old_c)
    if set(new_map) != set(old_map):
        return {"outcome": "unproven", "reason": "missing_meter"}
    for key, new_limit in new_map.items():
        old_limit = old_map[key]
        new_meter = catalog_map(new_catalogs)[(key[1], key[2])]
        old_meter = catalog_map(old_catalogs)[(key[1], key[2])]
        if new_meter["unit"] != old_meter["unit"]:
            return {"outcome": "unproven", "reason": "incompatible_meter_unit"}
        n, o = upper_key(new_limit["upper"]), upper_key(old_limit["upper"])
        if n[0] == "unbounded" and o[0] == "finite":
            return {"outcome": "violation"}
        if n[0] == "finite" and o[0] == "finite" and n[1] > o[1]:
            return {"outcome": "violation"}
    return {"outcome": "proven"}


def require_proven(proof):
    if proof["outcome"] == "proven":
        return
    if proof["outcome"] == "violation":
        raise UsageOracleError(USAGE_CEILING_EXCEEDED, "usage proof is a violation")
    raise UsageOracleError(USAGE_BOUND_UNPROVEN, "usage proof is unproven")


def meter(name, catalog=METER_CATALOG):
    return {"catalog": catalog, "name": name}


def unit(name, catalog=UNIT_CATALOG):
    return {"catalog": catalog, "name": name}


def finite(value):
    return {"finite": value}


def unbounded():
    return {"unbounded": None}


def limit(name, phase="forward", upper=None, catalog=METER_CATALOG):
    return {"meter": meter(name, catalog), "phase": phase,
            "upper": finite(upper) if isinstance(upper, int) else upper}


def signature(limits=()):
    return {"limits": list(limits)}


def base(case_kind, case_id, note, catalogs=None):
    return {"kind": "agentflow.conformance.usage-signature.case",
            "case_kind": case_kind, "corpus_revision": CORPUS_REVISION,
            "case_id": case_id, "note": note,
            "catalogs": catalogs or fixture_catalogs()}


CASES = []


def write_case(subdir, payload):
    path = CONF / subdir / f"{payload['case_id'].replace('/', '-')}.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, ensure_ascii=False,
                               sort_keys=True) + "\n", encoding="utf-8",
                    newline="\n")
    CASES.append(payload["case_id"])


def golden(case_id, note, sig, catalogs=None, proof=None):
    catalogs = catalogs or fixture_catalogs()
    canonical = canonicalize(sig, catalogs)
    payload = base("golden", case_id, note, catalogs)
    payload.update({"signature": sig, "canonical_ace1_utf8": canonical_ace1(canonical),
                    "limit_count": len(canonical["limits"]),
                    "proof": proof} if proof is not None else
                   {"signature": sig, "canonical_ace1_utf8": canonical_ace1(canonical),
                    "limit_count": len(canonical["limits"])})
    write_case("golden", payload)


def negative(case_id, note, payload):
    expected = payload["expected_kind"]
    try:
        operation = payload.get("operation")
        catalogs = payload.get("catalogs", fixture_catalogs())
        if operation == "canonicalize":
            canonicalize(payload["signature"], catalogs)
        elif operation == "accumulate":
            accumulation(payload["left"], payload["right"])
        elif operation == "require_within_ceiling":
            require_proven(within_ceiling(
                payload["observed"], declared_envelope(payload["declared"], catalogs)))
        elif operation == "require_compatibility":
            require_proven(usage_not_increased(
                payload["new_signature"], payload["old_signature"],
                payload.get("new_catalogs", catalogs), payload.get("old_catalogs", catalogs)))
        else:
            raise AssertionError(f"unknown negative operation {operation!r}")
    except UsageOracleError as error:
        if error.kind != expected:
            raise AssertionError(f"{case_id}: oracle raised {error.kind}, expected {expected}")
    else:
        raise AssertionError(f"{case_id}: expected {expected}")
    write_case("negative", {**base("negative", case_id, note, payload.get("catalogs")), **payload})


def algebra(case_id, note, payload):
    operation = payload["operation"]
    if operation == "accumulate":
        result = accumulation(payload["left"], payload["right"])
    elif operation == "branch_join":
        result = join(payload["left"], payload["right"])
    elif operation == "repeat":
        result = repeat(payload["count"], payload["input"])
    elif operation == "iterate_unknown":
        result = envelope((x["phase"], x["meter"], {"kind": "unknown"})
                          for x in payload["body"]["bounds"])
    elif operation == "within_ceiling":
        result = within_ceiling(payload["observed"],
                                declared_envelope(payload["declared"],
                                                  payload.get("catalogs", fixture_catalogs())))
    elif operation == "usage_not_increased":
        result = usage_not_increased(
            payload["new_signature"], payload["old_signature"],
            payload.get("new_catalogs", fixture_catalogs()),
            payload.get("old_catalogs", fixture_catalogs()))
    elif operation == "race":
        result = payload["branches"][0]
        for branch in payload["branches"][1:]:
            result = accumulation(result, branch)
    elif operation == "retry":
        result = repeat(payload["max_attempts"], payload["body"])
    elif operation == "compensation":
        result = accumulation(payload["forward"], payload["recovery"])
    else:
        raise AssertionError(f"unknown algebra operation {operation!r}")
    payload = {**base("algebra", case_id, note, payload.get("catalogs")), **payload,
               "result": result}
    write_case("algebra", payload)


def compatibility(case_id, note, payload):
    expected = payload["expected"]
    actual = usage_not_increased(
        payload["new_signature"], payload["old_signature"],
        payload.get("new_catalogs", fixture_catalogs()),
        payload.get("old_catalogs", fixture_catalogs()))
    if actual != expected:
        raise AssertionError(f"{case_id}: oracle returned {actual}, expected {expected}")
    write_case("compatibility", {**base("compatibility", case_id, note, payload.get("catalogs")), **payload})


def equivalence(case_id, note, members, catalogs):
    frozen = canonical_ace1(canonicalize(members[0], catalogs))
    for member in members[1:]:
        if canonical_ace1(canonicalize(member, catalogs)) != frozen:
            raise AssertionError(f"{case_id}: equivalence member changed canonical bytes")
    write_case("equivalence", {**base("equivalence", case_id, note, catalogs),
                               "members": members, "canonical_ace1_utf8": frozen})


def main():
    if CONF.exists():
        for path in CONF.rglob("*.json"):
            path.unlink()

    # --- Golden ----------------------------------------------------------
    golden("usage/golden/single-finite-meter", "one finite additive forward limit",
           signature([limit("model.input_tokens", upper=100)]))
    golden("usage/golden/explicit-zero", "finite(0) is retained as an explicit zero ceiling",
           signature([limit("tool.invocations", upper=0)]))
    golden("usage/golden/unbounded", "unbounded is distinct from UINT64_MAX",
           signature([limit("model.output_tokens", upper=unbounded())]))
    golden("usage/golden/forward-recovery-separate",
           "the same meter in forward and recovery is two coordinates",
           signature([limit("model.output_tokens", "recovery", 3),
                      limit("model.output_tokens", "forward", 20)]))
    golden("usage/golden/multiple-meters", "distinct additive meters coexist",
           signature([limit("tool.invocations", upper=4),
                      limit("model.input_tokens", upper=1000),
                      limit("data.egress_bytes", upper=2048)]))
    golden("usage/golden/empty-not-zero", "an empty declaration is not a zero proof",
           signature(), proof={"outcome": "unproven", "reason": "missing_meter"})
    golden("usage/golden/canonical-ordering", "phase then complete meter identity controls canonical order",
           signature([limit("model.output_tokens", "recovery", 2),
                      limit("data.egress_bytes", "forward", 5),
                      limit("model.input_tokens", "forward", 1)]))

    # --- Negative --------------------------------------------------------
    negative("usage/negative/unknown-meter", "unknown meter identities fail closed",
             {"operation": "canonicalize", "signature": signature([limit("model.unknown", upper=1)]),
              "expected_kind": UNKNOWN_USAGE_METER})
    negative("usage/negative/invalid-catalog", "meter catalog syntax is validated",
             {"operation": "canonicalize", "signature": signature([limit("model.input_tokens", catalog="not-a-spec", upper=1)]),
              "expected_kind": INVALID_USAGE_METER})
    negative("usage/negative/duplicate-phase-meter", "phase/meter is a unique declaration key",
             {"operation": "canonicalize", "signature": signature([limit("model.input_tokens", upper=1),
                                                                         limit("model.input_tokens", upper=2)]),
              "expected_kind": DUPLICATE_USAGE_LIMIT})
    negative("usage/negative/arithmetic-overflow", "finite addition is checked",
             {"operation": "accumulate",
              "left": envelope([("forward", meter("model.input_tokens"), {"kind": "known_finite", "value": 2**64 - 1})], False),
              "right": envelope([("forward", meter("model.input_tokens"), {"kind": "known_finite", "value": 1})], False),
              "expected_kind": USAGE_ARITHMETIC_OVERFLOW})
    negative("usage/negative/meter-omitted-not-zero", "missing observed usage is not finite zero",
             {"operation": "require_within_ceiling",
              "observed": envelope(),
              "declared": signature([limit("model.output_tokens", upper=0)]),
              "expected_kind": USAGE_BOUND_UNPROVEN})
    negative("usage/negative/unsupported-aggregation", "non-additive meters are rejected by the universe",
             {"operation": "canonicalize", "signature": signature([limit("model.input_tokens", upper=1)]),
              "catalogs": {"meters": [{"catalog": METER_CATALOG, "name": "model.input_tokens",
                                         "unit": unit("tokens"), "aggregation": "latency"}]},
              "expected_kind": UNSUPPORTED_USAGE_AGGREGATION})
    negative("usage/negative/quota-not-semantic-budget",
             "a quota-shaped catalog identity cannot become a runtime quota through Usage",
             {"operation": "canonicalize", "signature": signature([limit("tenant.quota", upper=1)]),
              "expected_kind": UNKNOWN_USAGE_METER})

    # --- Algebra ---------------------------------------------------------
    one = envelope([("forward", meter("model.input_tokens"), {"kind": "known_finite", "value": 4})])
    two = envelope([("forward", meter("model.input_tokens"), {"kind": "known_finite", "value": 6})])
    zero = envelope([("forward", meter("model.input_tokens"), {"kind": "known_finite", "value": 0})])
    unknown = envelope([("forward", meter("model.input_tokens"), {"kind": "unknown"})])
    algebra("usage/algebra/accumulation-associativity", "additive accumulation is associative",
            {"operation": "accumulate", "left": one, "right": two})
    algebra("usage/algebra/accumulation-zero-identity", "explicit zero is an additive identity",
            {"operation": "accumulate", "left": one, "right": zero})
    algebra("usage/algebra/branch-join-commutative", "branch join takes the finite maximum",
            {"operation": "branch_join", "left": one, "right": two})
    algebra("usage/algebra/branch-join-associative", "branch join is associative over static maxima",
            {"operation": "branch_join", "left": join(one, two), "right": one})
    algebra("usage/algebra/branch-join-idempotent", "joining a branch with itself is unchanged",
            {"operation": "branch_join", "left": one, "right": one})
    algebra("usage/algebra/repeat-zero", "proven zero repetitions create explicit zero coordinates",
            {"operation": "repeat", "count": 0, "input": one})
    algebra("usage/algebra/repeat-one", "one repetition preserves the bound",
            {"operation": "repeat", "count": 1, "input": one})
    algebra("usage/algebra/repeat-n", "finite repetition multiplies with checked arithmetic",
            {"operation": "repeat", "count": 3, "input": one})
    algebra("usage/algebra/unknown-propagation", "unknown never becomes zero",
            {"operation": "accumulate", "left": unknown, "right": one})
    algebra("usage/algebra/race-loser-retained", "race conservatively accumulates losers",
            {"operation": "race", "branches": [one, two]})
    algebra("usage/algebra/retry-amplification", "retry scales metered usage by max attempts",
            {"operation": "retry", "max_attempts": 3, "body": one})
    algebra("usage/algebra/foreach-max-inflight-not-count", "unknown repetition does not use max_inflight as total count",
            {"operation": "iterate_unknown", "body": one})
    algebra("usage/algebra/compensation-no-refund", "recovery usage is retained beside forward usage",
            {"operation": "compensation", "forward": one,
             "recovery": envelope([("recovery", meter("model.input_tokens"), {"kind": "known_finite", "value": 2})])})

    # --- Compatibility ---------------------------------------------------
    compatibility("usage/compatibility/same-limits", "identical usage signatures are compatible",
                  {"new_signature": signature([limit("model.input_tokens", upper=5)]),
                   "old_signature": signature([limit("model.input_tokens", upper=5)]),
                   "expected": {"outcome": "proven"}})
    compatibility("usage/compatibility/new-lower-limit", "a lower finite ceiling is non-increasing",
                  {"new_signature": signature([limit("model.input_tokens", upper=3)]),
                   "old_signature": signature([limit("model.input_tokens", upper=5)]),
                   "expected": {"outcome": "proven"}})
    compatibility("usage/compatibility/new-higher-limit", "a higher finite ceiling is a violation",
                  {"new_signature": signature([limit("model.input_tokens", upper=8)]),
                   "old_signature": signature([limit("model.input_tokens", upper=5)]),
                   "expected": {"outcome": "violation"}})
    compatibility("usage/compatibility/missing-new-meter", "missing new declarations are not zero",
                  {"new_signature": signature(),
                   "old_signature": signature([limit("model.input_tokens", upper=5)]),
                   "expected": {"outcome": "unproven", "reason": "missing_meter"}})
    compatibility("usage/compatibility/different-meter-unit", "unit mismatch is not comparable",
                  {"new_signature": signature([limit("model.input_tokens", upper=3)]),
                   "old_signature": signature([limit("model.input_tokens", upper=5)]),
                   "new_catalogs": fixture_catalogs(),
                   "old_catalogs": fixture_catalogs([("model.input_tokens", "bytes"),
                                                     ("model.output_tokens", "tokens"),
                                                     ("tool.invocations", "calls"),
                                                     ("data.egress_bytes", "bytes")]),
                   "expected": {"outcome": "unproven", "reason": "incompatible_meter_unit"}})
    compatibility("usage/compatibility/forward-recovery-not-interchangeable",
                  "forward and recovery are independent coordinates",
                  {"new_signature": signature([limit("model.input_tokens", "recovery", 1)]),
                   "old_signature": signature([limit("model.input_tokens", "forward", 5)]),
                   "expected": {"outcome": "unproven", "reason": "missing_meter"}})

    # --- Equivalence -----------------------------------------------------
    equivalence("usage/equivalence/limit-ordering", "declaration order is not identity",
                [signature([limit("model.output_tokens", "recovery", 2),
                            limit("model.input_tokens", "forward", 1)]),
                 signature([limit("model.input_tokens", "forward", 1),
                            limit("model.output_tokens", "recovery", 2)])],
                fixture_catalogs())

    counts = {}
    for case in CASES:
        category = case.split("/")[1]
        counts[category] = counts.get(category, 0) + 1
    print(f"frozen {len(CASES)} cases: {counts}")


if __name__ == "__main__":
    main()
