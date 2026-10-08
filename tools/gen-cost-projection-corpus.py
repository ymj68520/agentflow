#!/usr/bin/env python3
"""Freeze the Phase 4A.5B fixed Cost Projection corpus.

This oracle is independent of Rust. It reuses only the independent AFIDL1 /
ACE1 plumbing and the Usage fixture vocabulary; price validation, rational
ceil arithmetic, coverage, budget and cost-compatibility logic are implemented
here directly.
"""

import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CONF = ROOT / "spec" / "conformance" / "cost-projection"
CORPUS_REVISION = 1

usage_spec = importlib.util.spec_from_file_location(
    "usage_oracle", Path(__file__).resolve().parent / "gen-usage-signature-corpus.py")
usage = importlib.util.module_from_spec(usage_spec)
usage_spec.loader.exec_module(usage)

_sir_spec = importlib.util.spec_from_file_location(
    "sir", Path(__file__).resolve().parent / "gen-semantic-ir-corpus.py")
sir = importlib.util.module_from_spec(_sir_spec)
_sir_spec.loader.exec_module(sir)
afidl = sir.afidl

SEMANTIC_CORE = ROOT / "spec" / "idl" / "semantic-core.afidl"
USAGE_AFDL = ROOT / "spec" / "idl" / "usage-signature.afidl"
COST_AFDL = ROOT / "spec" / "idl" / "cost-projection.afidl"

INVALID_CURRENCY = "invalid_currency"
UNKNOWN_CURRENCY = "unknown_currency"
CURRENCY_UNIT_MISMATCH = "currency_unit_mismatch"
INVALID_PRICE_RATE = "invalid_price_rate"
DUPLICATE_PRICE_ENTRY = "duplicate_price_entry"
UNKNOWN_PRICE_METER = "unknown_price_meter"
PRICE_METER_UNIT_MISMATCH = "price_meter_unit_mismatch"
UNSUPPORTED_PRICING_MODEL = "unsupported_pricing_model"
UNSUPPORTED_ROUNDING_RULE = "unsupported_rounding_rule"
COST_ARITHMETIC_OVERFLOW = "cost_arithmetic_overflow"
MISSING_PRICE_COVERAGE = "missing_price_coverage"
COST_PROJECTION_UNPROVEN = "cost_projection_unproven"
STATIC_BUDGET_EXCEEDED = "static_budget_exceeded"
STATIC_BUDGET_UNPROVEN = "static_budget_unproven"

CURRENCY_CATALOG = "agentflow.test.cost.currencies@1"
METER_UNIT_CATALOG = usage.UNIT_CATALOG
CURRENCY_UNIT_CATALOG = "agentflow.test.cost.units@1"
USD = {"catalog": CURRENCY_CATALOG, "code": "USD"}
USD_ATOMIC = {"catalog": CURRENCY_UNIT_CATALOG, "name": "usd_atomic"}
JPY = {"catalog": CURRENCY_CATALOG, "code": "JPY"}
JPY_ATOMIC = {"catalog": CURRENCY_UNIT_CATALOG, "name": "jpy_atomic"}


def cost_catalogs(meters=None, currencies=None):
    return {
        "meters": usage.fixture_catalogs(meters)["meters"],
        "currencies": currencies or [
            {"currency": USD, "atomic_unit": USD_ATOMIC, "display_scale": 2},
            {"currency": JPY, "atomic_unit": JPY_ATOMIC, "display_scale": 0},
        ],
    }


def load_types():
    sources = [
        ("semantic-core.afidl", SEMANTIC_CORE.read_text(encoding="utf-8")),
        ("usage-signature.afidl", USAGE_AFDL.read_text(encoding="utf-8")),
        ("cost-projection.afidl", COST_AFDL.read_text(encoding="utf-8")),
    ]
    _modules, by_module = afidl.compile_module_set(sources)
    types = {}
    for spec, schemas in by_module.items():
        for schema in schemas:
            types[f"{spec}/{schema['name']}"] = schema
    return types


TYPES = load_types()


def shape_of(identity):
    kind, payload = TYPES[identity]["body"]
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
    raise AssertionError(kind)


def expr_shape(expr):
    kind, value = expr
    if kind == "primitive":
        return sir.PRIM_SHAPES[value]
    if kind == "ref":
        return shape_of(value)
    if kind == "list":
        return ("list", expr_shape(value))
    if kind == "option":
        return ("option", expr_shape(value))
    if kind == "map_string":
        return ("map", expr_shape(value))
    raise AssertionError(kind)


SNAPSHOT_SHAPE = shape_of("agentflow.usage.cost@1/PriceTableSnapshot")
CEILING_SHAPE = shape_of("agentflow.usage.cost@1/StaticCostCeiling")


def to_value_tree(shape, value):
    kind = shape[0]
    if kind == "variant":
        if isinstance(value, str):
            return {value: None}
        case, payload = next(iter(value.items()))
        return {case: to_value_tree(dict(shape[2])[case], payload)}
    if kind == "enum":
        return value
    if kind == "record":
        return {name: to_value_tree(field_shape, value[name])
                for name, field_shape in shape[2]}
    if kind == "list":
        return [to_value_tree(shape[1], item) for item in value]
    if kind == "option":
        return None if value is None else to_value_tree(shape[1], value)
    return value


def canonical_ace1(snapshot):
    tree = to_value_tree(SNAPSHOT_SHAPE, snapshot)
    return sir.encode_value(SNAPSHOT_SHAPE,
                            sir.json_to_value(SNAPSHOT_SHAPE, tree))


def ceiling_ace1(ceiling):
    tree = to_value_tree(CEILING_SHAPE, ceiling)
    return sir.encode_value(CEILING_SHAPE,
                            sir.json_to_value(CEILING_SHAPE, tree))


class CostOracleError(Exception):
    def __init__(self, kind, detail):
        super().__init__(f"{kind}: {detail}")
        self.kind = kind


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
    return major.isdigit() and major[0] != "0"


def check_currency(currency, catalogs):
    if not specification_id(currency["catalog"]) or not currency["code"]:
        raise CostOracleError(INVALID_CURRENCY, "invalid currency identity")
    currency = {"catalog": currency["catalog"], "code": currency["code"]}
    if not any(c["currency"] == currency for c in catalogs["currencies"]):
        raise CostOracleError(UNKNOWN_CURRENCY, "currency is not recognized")
    return currency


def unit_equal(left, right):
    return left == right


def meter_descriptor(meter, catalogs):
    for descriptor in catalogs["meters"]:
        if descriptor["catalog"] == meter["catalog"] and descriptor["name"] == meter["name"]:
            if descriptor["aggregation"] != "additive":
                raise CostOracleError(UNKNOWN_PRICE_METER, "meter is not additive")
            return descriptor
    raise CostOracleError(UNKNOWN_PRICE_METER, "meter is not recognized")


def canonicalize(snapshot, catalogs):
    currency = check_currency(snapshot["currency"], catalogs)
    descriptor = next(c for c in catalogs["currencies"] if c["currency"] == currency)
    if snapshot["atomic_unit"] != descriptor["atomic_unit"]:
        raise CostOracleError(CURRENCY_UNIT_MISMATCH, "atomic currency unit mismatch")
    if snapshot["model"] != "fixed_linear":
        raise CostOracleError(UNSUPPORTED_PRICING_MODEL, "unsupported pricing model")
    seen = set()
    entries = []
    for entry in snapshot["entries"]:
        meter_descriptor(entry["meter"], catalogs)
        meter = {"catalog": entry["meter"]["catalog"],
                 "name": entry["meter"]["name"]}
        expected_unit = next(m["unit"] for m in catalogs["meters"]
                             if m["catalog"] == meter["catalog"] and m["name"] == meter["name"])
        if not unit_equal(entry["unit"], expected_unit):
            raise CostOracleError(PRICE_METER_UNIT_MISMATCH, "meter unit mismatch")
        if entry["rate"]["denominator"] == 0:
            raise CostOracleError(INVALID_PRICE_RATE, "zero denominator")
        if entry["rounding"] != "per_meter_unit_ceil":
            raise CostOracleError(UNSUPPORTED_ROUNDING_RULE, "unsupported rounding")
        key = (entry["phase"], meter["catalog"], meter["name"])
        if key in seen:
            raise CostOracleError(DUPLICATE_PRICE_ENTRY, "duplicate price entry")
        seen.add(key)
        entries.append({**entry, "meter": meter})
    phase_no = {None: 0, "forward": 1, "recovery": 2}
    entries.sort(key=lambda e: (phase_no[e["phase"]], e["meter"]["catalog"], e["meter"]["name"]))
    return {"currency": currency, "atomic_unit": snapshot["atomic_unit"],
            "model": "fixed_linear", "entries": entries}


def finite(value):
    return {"kind": "known_finite", "amount": value}


def unbounded():
    return {"kind": "known_unbounded"}


def unproven(reason):
    return {"kind": "unproven", "reason": reason}


def projection(currency, forward, recovery):
    total = add_bound(forward, recovery)
    return {"currency": currency, "forward": forward, "recovery": recovery, "total": total}


def add_bound(left, right):
    if left["kind"] == "unproven":
        return left
    if right["kind"] == "unproven":
        return right
    if left["kind"] == "known_unbounded" or right["kind"] == "known_unbounded":
        return {"kind": "known_unbounded"}
    value = left["amount"] + right["amount"]
    if value > 2**64 - 1:
        raise CostOracleError(COST_ARITHMETIC_OVERFLOW, "cost addition overflow")
    return finite(value)


def price_entry(entries, meter, phase):
    exact = next((e for e in entries if e["meter"] == meter and e["phase"] == phase), None)
    return exact or next((e for e in entries if e["meter"] == meter and e["phase"] is None), None)


def project(usage_envelope, snapshot, catalogs):
    canonical = canonicalize(snapshot, catalogs)
    currency = canonical["currency"]
    if usage_envelope["global_unknown"]:
        return projection(currency, unproven("unknown_usage"), unproven("unknown_usage"))
    if not usage_envelope["bounds"]:
        return projection(currency, unproven("missing_price_coverage"),
                          unproven("missing_price_coverage"))
    forward, recovery = finite(0), finite(0)
    for item in usage_envelope["bounds"]:
        entry = price_entry(canonical["entries"], item["meter"], item["phase"])
        if entry is None:
            return projection(currency, unproven("missing_price_coverage"),
                              unproven("missing_price_coverage"))
        bound = item["bound"]
        if bound["kind"] == "unknown":
            one = unproven("unknown_usage")
        elif bound["kind"] == "known_unbounded":
            one = finite(0) if entry["rate"]["numerator"] == 0 else unbounded()
        else:
            unit_upper = (entry["rate"]["numerator"] + entry["rate"]["denominator"] - 1) // entry["rate"]["denominator"]
            amount = bound["value"] * unit_upper
            if amount > 2**64 - 1:
                raise CostOracleError(COST_ARITHMETIC_OVERFLOW, "cost multiplication overflow")
            one = finite(amount)
        if item["phase"] == "forward":
            forward = add_bound(forward, one)
        else:
            recovery = add_bound(recovery, one)
    return projection(currency, forward, recovery)


def compare_bound(bound, ceiling):
    if bound["kind"] == "unproven":
        return {"outcome": "unproven", "reason": bound["reason"]}
    if bound["kind"] == "known_unbounded" or bound["amount"] > ceiling:
        return {"outcome": "violation"}
    return {"outcome": "proven"}


def budget(proj, ceiling):
    forward = compare_bound(proj["forward"], ceiling["forward"]["atomic_units"])
    recovery = compare_bound(proj["recovery"], ceiling["recovery"]["atomic_units"])
    if forward["outcome"] == "violation" or recovery["outcome"] == "violation":
        return {"outcome": "violation"}
    if forward["outcome"] == "unproven":
        return forward
    if recovery["outcome"] == "unproven":
        return recovery
    return {"outcome": "proven"}


def cost_not_increased(new, new_snapshot, old, old_snapshot):
    if new["currency"] != old["currency"] or new_snapshot != old_snapshot:
        return {"outcome": "unproven", "reason": "different_price_snapshot"}
    results = []
    for n, o in ((new["forward"], old["forward"]), (new["recovery"], old["recovery"])):
        if n["kind"] == "unproven" or o["kind"] == "unproven":
            results.append({"outcome": "unproven", "reason": "unknown_usage"})
        elif n["kind"] == "known_unbounded" and o["kind"] != "known_unbounded":
            results.append({"outcome": "violation"})
        elif n["kind"] == "known_finite" and o["kind"] == "known_unbounded":
            results.append({"outcome": "proven"})
        elif n["kind"] == "known_finite" and o["kind"] == "known_finite":
            results.append({"outcome": "proven" if n["amount"] <= o["amount"] else "violation"})
        else:
            results.append({"outcome": "proven"})
    if any(r["outcome"] == "violation" for r in results):
        return {"outcome": "violation"}
    if any(r["outcome"] == "unproven" for r in results):
        return next(r for r in results if r["outcome"] == "unproven")
    return {"outcome": "proven"}


def currency(code="USD"):
    return {"catalog": CURRENCY_CATALOG, "code": code}


def unit(name):
    return {"catalog": METER_UNIT_CATALOG, "name": name}


def money(amount, code="USD"):
    return {"currency": currency(code), "atomic_units": amount}


def rate(numerator, denominator):
    return {"numerator": numerator, "denominator": denominator}


def entry(name, phase=None, numerator=1, denominator=1, unit_name="tokens", rounding="per_meter_unit_ceil"):
    return {"meter": usage.meter(name), "phase": phase, "unit": unit(unit_name),
            "rate": rate(numerator, denominator), "rounding": rounding}


def snapshot(entries=(), model="fixed_linear", code="USD", atomic=None, catalogs=None):
    atomic = atomic or (USD_ATOMIC if code == "USD" else JPY_ATOMIC)
    return {"currency": currency(code), "atomic_unit": atomic, "model": model,
            "entries": list(entries)}


def ceiling(forward, recovery, code="USD"):
    return {"currency": currency(code), "forward": money(forward, code),
            "recovery": money(recovery, code)}


def base(kind, case_id, note, catalogs):
    return {"kind": "agentflow.conformance.cost-projection.case",
            "case_kind": kind, "corpus_revision": CORPUS_REVISION,
            "case_id": case_id, "note": note, "catalogs": catalogs}


CASES = []


def write_case(subdir, payload):
    path = CONF / subdir / f"{payload['case_id'].replace('/', '-')}.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, ensure_ascii=False,
                               sort_keys=True) + "\n", encoding="utf-8",
                    newline="\n")
    CASES.append(payload["case_id"])


def golden(case_id, note, usage_envelope, snap, catalogs, expected=None):
    canonical = canonicalize(snap, catalogs)
    payload = {**base("golden", case_id, note, catalogs), "usage": usage_envelope,
               "snapshot": snap, "canonical_ace1_utf8": canonical_ace1(canonical)}
    if expected is None:
        payload["projection"] = project(usage_envelope, snap, catalogs)
    else:
        payload["projection"] = expected
    write_case("golden", payload)


def pricing(case_id, note, usage_envelope, snap, catalogs):
    payload = {**base("pricing", case_id, note, catalogs), "usage": usage_envelope,
               "snapshot": snap, "projection": project(usage_envelope, snap, catalogs)}
    write_case("pricing", payload)


def negative(case_id, note, payload):
    project_unproven = False
    try:
        catalogs = payload.get("catalogs", cost_catalogs())
        operation = payload["operation"]
        if operation == "canonicalize":
            canonicalize(payload["snapshot"], catalogs)
        elif operation == "project":
            result = project(payload["usage"], payload["snapshot"], catalogs)
            if payload["expected_kind"] != COST_PROJECTION_UNPROVEN \
                    or result["forward"]["kind"] != "unproven":
                raise AssertionError(f"{case_id}: project did not produce expected unproven state")
            project_unproven = True
        elif operation == "budget":
            result = budget(project(payload["usage"], payload["snapshot"], catalogs), payload["ceiling"])
            if result["outcome"] != payload["expected_outcome"]:
                raise CostOracleError(payload["expected_kind"], "expected proof result")
        else:
            raise AssertionError(operation)
    except CostOracleError as error:
        if error.kind != payload["expected_kind"]:
            raise AssertionError(f"{case_id}: {error.kind} != {payload['expected_kind']}")
    else:
        if operation != "budget" and not project_unproven:
            raise AssertionError(f"{case_id}: expected {payload['expected_kind']}")
    write_case("negative", {**base("negative", case_id, note, catalogs), **payload})


def budget_case(case_id, note, usage_envelope, snap, ceiling_value, catalogs, expected):
    proj = project(usage_envelope, snap, catalogs)
    payload = {**base("budget", case_id, note, catalogs), "usage": usage_envelope,
               "snapshot": snap, "ceiling": ceiling_value,
               "projection": proj, "expected": expected}
    if budget(proj, ceiling_value) != expected:
        raise AssertionError(f"{case_id}: budget oracle drift")
    write_case("budget", payload)


def equivalence(case_id, note, members, catalogs):
    frozen = canonical_ace1(canonicalize(members[0], catalogs))
    for member in members[1:]:
        if canonical_ace1(canonicalize(member, catalogs)) != frozen:
            raise AssertionError(f"{case_id}: canonical bytes differ")
    write_case("equivalence", {**base("equivalence", case_id, note, catalogs),
                               "members": members, "canonical_ace1_utf8": frozen})


def main():
    if CONF.exists():
        for path in CONF.rglob("*.json"):
            path.unlink()
    catalogs = cost_catalogs()
    usage_one = usage.envelope([("forward", usage.meter("model.input_tokens"),
                                {"kind": "known_finite", "value": 3})])
    usage_zero = usage.envelope([("forward", usage.meter("model.input_tokens"),
                                 {"kind": "known_finite", "value": 0})])
    usage_two_phases = usage.envelope([
        ("forward", usage.meter("model.input_tokens"), {"kind": "known_finite", "value": 3}),
        ("recovery", usage.meter("model.input_tokens"), {"kind": "known_finite", "value": 2}),
    ])
    usage_many = usage.envelope([
        ("forward", usage.meter("model.input_tokens"), {"kind": "known_finite", "value": 3}),
        ("forward", usage.meter("tool.invocations"), {"kind": "known_finite", "value": 2}),
    ])
    # Golden --------------------------------------------------------------
    golden("cost/golden/single-currency-meter", "one USD meter projection",
           usage_one, snapshot([entry("model.input_tokens", numerator=2, denominator=1)]), catalogs)
    golden("cost/golden/multiple-meters", "multiple additive meters share one currency",
           usage_many, snapshot([entry("tool.invocations", numerator=3, denominator=1, unit_name="calls"),
                                 entry("model.input_tokens", numerator=2, denominator=1)]), catalogs)
    golden("cost/golden/forward-recovery", "forward and recovery costs remain separate",
           usage_two_phases, snapshot([entry("model.input_tokens", "forward", 2, 1),
                                       entry("model.input_tokens", "recovery", 3, 1)]), catalogs)
    golden("cost/golden/explicit-zero-rate", "zero price is explicit and finite",
           usage_one, snapshot([entry("model.input_tokens", numerator=0, denominator=1)]), catalogs)
    golden("cost/golden/explicit-zero-usage", "explicit zero usage projects to zero",
           usage_zero, snapshot([entry("model.input_tokens", numerator=7, denominator=3)]), catalogs)
    golden("cost/golden/canonical-price-order", "price entry order is canonicalized",
           usage_many, snapshot([entry("tool.invocations", numerator=3, unit_name="calls"), entry("model.input_tokens", numerator=2)]), catalogs)
    golden("cost/golden/single-jpy-currency", "currency identity is explicit",
           usage_one, snapshot([entry("model.input_tokens", numerator=1)], code="JPY"), catalogs)
    # Pricing -------------------------------------------------------------
    pricing("cost/pricing/exact-integer-rate", "exact integer rate", usage_one,
            snapshot([entry("model.input_tokens", numerator=2, denominator=1)]), catalogs)
    pricing("cost/pricing/fractional-rate", "fractional rational rate uses unit ceiling", usage_one,
            snapshot([entry("model.input_tokens", numerator=1, denominator=3)]), catalogs)
    pricing("cost/pricing/per-unit-conservative-rounding", "per-unit ceil is conservative", usage_two_phases,
            snapshot([entry("model.input_tokens", numerator=1, denominator=3)]), catalogs)
    pricing("cost/pricing/high-usage-fractional", "high usage remains checked integer arithmetic", usage_many,
            snapshot([entry("model.input_tokens", numerator=5, denominator=2), entry("tool.invocations", numerator=1, denominator=3, unit_name="calls")]), catalogs)
    pricing("cost/pricing/explicit-free-meter", "zero rate is free only by explicit entry", usage_many,
            snapshot([entry("model.input_tokens", numerator=0), entry("tool.invocations", numerator=0, unit_name="calls")]), catalogs)
    pricing("cost/pricing/unbounded-positive", "unbounded usage with positive rate stays unbounded",
            usage.envelope([("forward", usage.meter("model.input_tokens"), {"kind": "known_unbounded"})]),
            snapshot([entry("model.input_tokens", numerator=1)]), catalogs)
    pricing("cost/pricing/unbounded-free", "unbounded explicit free meter is finite zero",
            usage.envelope([("forward", usage.meter("model.input_tokens"), {"kind": "known_unbounded"})]),
            snapshot([entry("model.input_tokens", numerator=0)]), catalogs)
    # Negative ------------------------------------------------------------
    negative("cost/negative/missing-price", "unpriced usage is not free",
             {"operation": "project", "usage": usage_one, "snapshot": snapshot([]),
              "expected_kind": COST_PROJECTION_UNPROVEN})
    negative("cost/negative/missing-currency", "unknown currency fails closed",
             {"operation": "canonicalize", "snapshot": snapshot([entry("model.input_tokens")], code="GBP"),
              "expected_kind": UNKNOWN_CURRENCY})
    negative("cost/negative/different-currency", "currency atomic unit mismatch fails closed",
             {"operation": "canonicalize", "snapshot": snapshot([entry("model.input_tokens")], code="JPY", atomic=USD_ATOMIC),
              "expected_kind": CURRENCY_UNIT_MISMATCH})
    negative("cost/negative/wrong-meter-unit", "price unit must match Usage meter unit",
             {"operation": "canonicalize", "snapshot": snapshot([entry("model.input_tokens", unit_name="bytes")]),
              "expected_kind": PRICE_METER_UNIT_MISMATCH})
    negative("cost/negative/zero-denominator", "price denominator is positive",
             {"operation": "canonicalize", "snapshot": snapshot([entry("model.input_tokens", denominator=0)]),
              "expected_kind": INVALID_PRICE_RATE})
    negative("cost/negative/duplicate-price-entry", "price applicability key is unique",
             {"operation": "canonicalize", "snapshot": snapshot([entry("model.input_tokens"), entry("model.input_tokens")]),
              "expected_kind": DUPLICATE_PRICE_ENTRY})
    negative("cost/negative/unsupported-tiered", "tiered pricing cannot be approximated",
             {"operation": "canonicalize", "snapshot": snapshot([entry("model.input_tokens")], model="tiered"),
              "expected_kind": UNSUPPORTED_PRICING_MODEL})
    negative("cost/negative/unsupported-batch-rounding", "aggregate-once rounding is not safe",
             {"operation": "canonicalize", "snapshot": snapshot([entry("model.input_tokens", rounding="aggregate_once_ceil")]),
              "expected_kind": UNSUPPORTED_ROUNDING_RULE})
    negative("cost/negative/missing-usage-not-zero", "empty usage cannot prove zero cost",
             {"operation": "project", "usage": usage.envelope(),
              "snapshot": snapshot([entry("model.input_tokens")]),
              "expected_kind": COST_PROJECTION_UNPROVEN})
    negative("cost/negative/overflow", "large usage times rate fails checked arithmetic",
             {"operation": "project", "usage": usage.envelope([("forward", usage.meter("model.input_tokens"), {"kind": "known_finite", "value": 2**64 - 1})]),
              "snapshot": snapshot([entry("model.input_tokens", numerator=2)]),
              "expected_kind": COST_ARITHMETIC_OVERFLOW})
    # Budget --------------------------------------------------------------
    snap = snapshot([entry("model.input_tokens", numerator=2)])
    budget_case("cost/budget/forward-within", "forward cost fits ceiling", usage_one, snap, ceiling(6, 100), catalogs, {"outcome": "proven"})
    budget_case("cost/budget/forward-exceeds", "forward cost exceeds ceiling", usage_one, snap, ceiling(5, 100), catalogs, {"outcome": "violation"})
    budget_case("cost/budget/recovery-within", "recovery cost fits independently", usage_two_phases, snapshot([entry("model.input_tokens", "forward", 2), entry("model.input_tokens", "recovery", 3)]), ceiling(6, 6), catalogs, {"outcome": "proven"})
    budget_case("cost/budget/recovery-exceeds", "recovery overspend is independent", usage_two_phases, snapshot([entry("model.input_tokens", "forward", 2), entry("model.input_tokens", "recovery", 3)]), ceiling(100, 5), catalogs, {"outcome": "violation"})
    budget_case("cost/budget/forward-surplus-no-subsidy", "forward allowance cannot subsidize recovery", usage_two_phases, snapshot([entry("model.input_tokens", "forward", 2), entry("model.input_tokens", "recovery", 3)]), ceiling(100, 5), catalogs, {"outcome": "violation"})
    budget_case("cost/budget/unproven-not-proven", "unknown usage remains unproven", usage.envelope([("forward", usage.meter("model.input_tokens"), {"kind": "unknown"})]), snap, ceiling(100, 100), catalogs, {"outcome": "unproven", "reason": "unknown_usage"})
    # Cost compatibility -------------------------------------------------
    old_projection = project(usage_one, snap, catalogs)
    new_projection = project(usage.envelope([("forward", usage.meter("model.input_tokens"), {"kind": "known_finite", "value": 2})]), snap, catalogs)
    budget_case("cost/budget/same-price-cost-non-increase", "same price scenario proves lower cost", usage_one, snap, ceiling(100, 100), catalogs, {"outcome": "proven"})
    write_case("budget", {**base("budget", "cost/budget/cost-non-increase", "same-price cost comparison", catalogs),
                           "old_projection": old_projection, "new_projection": new_projection,
                           "old_snapshot": snap, "new_snapshot": snap,
                           "expected": cost_not_increased(new_projection, snap, old_projection, snap)})
    changed_snap = snapshot([entry("model.input_tokens", numerator=3)])
    write_case("budget", {**base("budget", "cost/budget/different-price-separated", "different price snapshots are not comparable", catalogs),
                           "old_projection": old_projection, "new_projection": project(usage_one, changed_snap, catalogs),
                           "old_snapshot": snap, "new_snapshot": changed_snap,
                           "expected": {"outcome": "unproven", "reason": "different_price_snapshot"}})
    # Equivalence ---------------------------------------------------------
    equivalence("cost/equivalence/price-entry-order", "price declaration order is not identity",
                [snapshot([entry("tool.invocations", numerator=3, unit_name="calls"), entry("model.input_tokens", numerator=2)]),
                 snapshot([entry("model.input_tokens", numerator=2), entry("tool.invocations", numerator=3, unit_name="calls")])], catalogs)
    print(f"frozen {len(CASES)} cases")


if __name__ == "__main__":
    main()
