#!/usr/bin/env python3
"""Freezes the Phase 4A.1 effect-signature conformance corpus.

INDEPENDENT implementation of the Effect Signature rules
(spec/effect/effect-signature.md): effect atom canonicalization, occurrence
bounds arithmetic, the closed/open/top row distinction, the effect algebra
(containment, accumulation, branch join, repetition), static control
propagation and the fail-closed validator. The Rust implementation in
crates/af-effect must agree with the frozen expectations byte-for-byte;
expected values are NEVER derived from the Rust code.

Plumbing reuse (allowed): the independent AFIDL1 implementation
(tools/gen-afidl-corpus.py, via the semantic-ir corpus module) supplies the
module compiler and the ACE1 value encoder. The effect ORACLE logic below —
validation, canonicalization, algebra, control propagation — is implemented
here from scratch and shares nothing with crates/af-effect.

Run from the repository root:  python tools/gen-effect-signature-corpus.py
"""

import copy
import importlib.util
import json
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CONF = ROOT / "spec" / "conformance" / "effect-signature"
# Revision 2 (Phase 4A.1B): guarded branch-join lower-bound correction.
# The Rust join_one kept min(lower) across DIFFERING guards and stamped
# the widened entry unconditional - promoting a condition-guaranteed
# lower bound to an unconditional one. The oracle already weakened to
# zero; the original different-guards fixture masked the divergence (one
# side had lower 0). Revision 2 freezes the guarded-join fixtures that
# expose the rule: identical guards keep the conditional minimum,
# differing guards weaken to zero, one-sided keys keep their guard. No
# pre-existing expected value changed; every earlier golden byte is
# identical.
# Revision 1 (Phase 4A.1): initial freeze.
CORPUS_REVISION = 2

# Plumbing: the semantic-ir corpus module carries the ACE1 encoder and the
# independent AFIDL1 compiler (which it loads as `afidl`).
_sir_spec = importlib.util.spec_from_file_location(
    "sir", Path(__file__).resolve().parent / "gen-semantic-ir-corpus.py")
sir = importlib.util.module_from_spec(_sir_spec)
_sir_spec.loader.exec_module(sir)
afidl = sir.afidl

SEMANTIC_CORE = ROOT / "spec" / "idl" / "semantic-core.afidl"
EFFECT_AFDL = ROOT / "spec" / "idl" / "effect-signature.afidl"

UINT64_MAX = 2 ** 64 - 1

# Stable error kinds (must mirror spec/effect/effect-signature.md section 13).
INVALID_EFFECT_OPERATION = "invalid_effect_operation"
UNKNOWN_EFFECT_OPERATION = "unknown_effect_operation"
INVALID_RESOURCE_SELECTOR = "invalid_resource_selector"
INVALID_EFFECT_QUALIFIER = "invalid_effect_qualifier"
INVALID_OCCURRENCE_BOUNDS = "invalid_occurrence_bounds"
DUPLICATE_EFFECT_ATOM = "duplicate_effect_atom"
INVALID_EFFECT_GUARD = "invalid_effect_guard"
UNBOUNDED_OPEN_EFFECT_ROW = "unbounded_open_effect_row"
EFFECT_ENVELOPE_UNPROVABLE = "effect_envelope_unprovable"
EFFECT_ARITHMETIC_OVERFLOW = "effect_arithmetic_overflow"

# The explicit, immutable fixture catalogs. The production capability
# catalog does not exist yet; these are TEST catalogs and are never
# presented as a production universe.
CATALOG = "agentflow.test.capability-catalog@1"
CATALOG_OPS = [
    "database.row.read",
    "database.row.write",
    "github.issue.create",
    "github.issue.read",
    "memory.append",
    "memory.search",
    "network.egress",
    "payment.charge",
]
ALT_CATALOG = "agentflow.test.alt-catalog@1"
ALT_CATALOG_OPS = ["storage.object.put"]


class EffectOracleError(Exception):
    """A structured static rejection: stable kind + detail."""

    def __init__(self, kind, detail):
        super().__init__(f"{kind}: {detail}")
        self.kind = kind
        self.detail = detail


# ---------------------------------------------------------------------------
# Shapes (local walker over the effect module set; ACE1 plumbing from sir)
# ---------------------------------------------------------------------------

def load_effect_types():
    sources = [
        ("semantic-core.afidl", SEMANTIC_CORE.read_text(encoding="utf-8")),
        ("effect-signature.afidl", EFFECT_AFDL.read_text(encoding="utf-8")),
    ]
    _modules, types_by_module = afidl.compile_module_set(sources)
    out = {}
    for spec, types in types_by_module.items():
        for schema in types:
            out[f"{spec}/{schema['name']}"] = schema
    return out


EFFECT_TYPES = load_effect_types()


def shape_of_identity(identity):
    kind, payload = EFFECT_TYPES[identity]["body"]
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
    raise AssertionError(f"unknown body kind {kind!r}")


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
    raise AssertionError(f"unknown type expr {kind!r}")


EFFECT_SIGNATURE_SHAPE = shape_of_identity("agentflow.effect.signature@1/EffectSignature")


def canonical_ace1(canonical_row):
    """The canonical ACE1 text of a signature whose row is already in
    value-tree form (unit variants as single-key dicts)."""
    value = sir.json_to_value(EFFECT_SIGNATURE_SHAPE, {"row": canonical_row})
    return sir.encode_value(EFFECT_SIGNATURE_SHAPE, value)


# ---------------------------------------------------------------------------
# Grammar checks
# ---------------------------------------------------------------------------

NAMESPACE_CHARS = set("abcdefghijklmnopqrstuvwxyz0123456789-")
OPERATION_CHARS = set("abcdefghijklmnopqrstuvwxyz0123456789-")


def is_specification_id(text):
    if "@" not in text:
        return False
    namespace, _, major = text.rpartition("@")
    if not namespace or not major:
        return False
    for segment in namespace.split("."):
        if not segment or segment[0] not in "abcdefghijklmnopqrstuvwxyz":
            return False
        if not all(c.isascii() and c in NAMESPACE_CHARS for c in segment):
            return False
    return major.isascii() and major.isdigit() and major[0] != "0"


def is_operation_name(text):
    if not text or len(text) > 256:
        return False
    if unicodedata.normalize("NFC", text) != text:
        return False
    for segment in text.split("."):
        if not segment:
            return False
        if not all(c.isascii() and c in OPERATION_CHARS for c in segment):
            return False
    return True


def check_operation_ref(ref):
    catalog = ref["catalog"]
    name = ref["name"]
    if not is_specification_id(catalog):
        raise EffectOracleError(
            INVALID_EFFECT_OPERATION,
            f"catalog identity {catalog!r} is not a SpecificationId")
    if not is_operation_name(name):
        raise EffectOracleError(
            INVALID_EFFECT_OPERATION,
            f"operation name {name!r} is invalid")


def check_selector(selector):
    port = selector["requirement_port_no"]
    if not isinstance(port, int) or isinstance(port, bool) or port < 1:
        raise EffectOracleError(
            INVALID_RESOURCE_SELECTOR,
            f"requirement port {port!r} is invalid: ports are 1-based")
    seen = set()
    for binding in selector["scope_bindings"]:
        key = binding["key"]
        value = binding["value"]
        if not key or not value:
            raise EffectOracleError(
                INVALID_RESOURCE_SELECTOR,
                "scope binding key and value must both be non-empty")
        if (unicodedata.normalize("NFC", key) != key
                or unicodedata.normalize("NFC", value) != value):
            raise EffectOracleError(
                INVALID_RESOURCE_SELECTOR,
                f"scope binding {key!r}={value!r} is not NFC-normalized")
        if key in seen:
            raise EffectOracleError(
                INVALID_RESOURCE_SELECTOR,
                f"scope key {key!r} is bound twice")
        seen.add(key)


QUALIFIER_FAMILIES = {"idempotency"}
IDEMPOTENCY_MEMBERS = {"natural", "none"}


def check_qualifiers(qualifiers):
    seen = set()
    for qualifier in qualifiers:
        if len(qualifier) != 1:
            raise EffectOracleError(
                INVALID_EFFECT_QUALIFIER,
                "a qualifier is exactly one family member")
        family = next(iter(qualifier))
        if family not in QUALIFIER_FAMILIES:
            raise EffectOracleError(
                INVALID_EFFECT_QUALIFIER,
                f"unknown qualifier family {family!r}")
        if family in seen:
            raise EffectOracleError(
                INVALID_EFFECT_QUALIFIER,
                f"qualifier family {family!r} appears twice")
        seen.add(family)
        member = qualifier[family]
        if member not in IDEMPOTENCY_MEMBERS:
            raise EffectOracleError(
                INVALID_EFFECT_QUALIFIER,
                f"unknown idempotency member {member!r}")


def check_ceiling(ceiling):
    if ceiling == "unbounded" or ceiling == {"unbounded": None}:
        return
    if isinstance(ceiling, dict) and len(ceiling) == 1 \
            and "finite" in ceiling and isinstance(ceiling["finite"], int) \
            and not isinstance(ceiling["finite"], bool) \
            and 0 <= ceiling["finite"] <= UINT64_MAX:
        return
    raise EffectOracleError(
        INVALID_OCCURRENCE_BOUNDS,
        f"occurrence ceiling {ceiling!r} is neither finite nor unbounded")


def check_bounds(bounds):
    lower = bounds["lower"]
    if not isinstance(lower, int) or isinstance(lower, bool) or lower < 0:
        raise EffectOracleError(
            INVALID_OCCURRENCE_BOUNDS,
            f"occurrence lower {lower!r} must be a uint64")
    check_ceiling(bounds["upper"])
    if bounds["upper"] != "unbounded" and bounds["upper"]["finite"] < lower:
        raise EffectOracleError(
            INVALID_OCCURRENCE_BOUNDS,
            f"occurrence bounds [{lower}, {bounds['upper']['finite']}] "
            "have upper < lower")


CONSTANT_KEYS = {"unit", "null", "bool", "int64", "decimal", "string",
                 "bytes", "timestamp", "duration"}


def check_guard(guard):
    if guard == "unconditional" or guard == {"unconditional": None}:
        return
    if isinstance(guard, dict) and len(guard) == 1 \
            and "input_equals" in guard:
        body = guard["input_equals"]
        port = body["port_no"]
        if not isinstance(port, int) or isinstance(port, bool) or port < 1:
            raise EffectOracleError(
                INVALID_EFFECT_GUARD,
                f"guard input port {port!r} is invalid: ports are 1-based")
        expected = body["expected"]
        if not isinstance(expected, dict) or len(expected) != 1 \
                or next(iter(expected)) not in CONSTANT_KEYS:
            raise EffectOracleError(
                INVALID_EFFECT_GUARD,
                "guard expected value is not a PrimitiveConstant")
        return
    raise EffectOracleError(
        INVALID_EFFECT_GUARD,
        f"guard {guard!r} is not part of the closed guard language")


# ---------------------------------------------------------------------------
# Canonical identity and canonicalization
# ---------------------------------------------------------------------------

def scope_pairs(selector):
    """The canonical (sorted) scope-binding list of a selector."""
    pairs = [(binding["key"], binding["value"])
             for binding in selector["scope_bindings"]]
    return sorted(pairs, key=lambda pair: (pair[0].encode("utf-8"),
                                           pair[1].encode("utf-8")))


def qualifier_family_list(qualifiers):
    """The canonical (sorted by frozen family case number) family list."""
    case_numbers = {"idempotency": 1}
    return sorted(case_numbers[qualifier_key] for qualifier_key, _ in
                  (next(iter(q.items())) for q in qualifiers))


def atom_key(atom):
    return (
        atom["operation"]["catalog"],
        atom["operation"]["name"],
        atom["resource_selector"]["requirement_port_no"],
        tuple(scope_pairs(atom["resource_selector"])),
        tuple(qualifier_family_list(atom["qualifiers"])),
    )


def check_entry(entry):
    check_operation_ref(entry["atom"]["operation"])
    check_selector(entry["atom"]["resource_selector"])
    check_qualifiers(entry["atom"]["qualifiers"])
    check_bounds(entry["bounds"])
    check_guard(entry["guard"])


def ceiling_upper(ceiling):
    """The comparable upper of a ceiling in either form: `None` represents
    infinity."""
    if ceiling == "unbounded" or ceiling == {"unbounded": None}:
        return None
    return ceiling["finite"]


def entry_cannot_occur(bounds):
    return ceiling_upper(bounds["upper"]) == 0


def canonical_entries(entries):
    keyed = []
    seen = set()
    for entry in entries:
        check_entry(entry)
        key = atom_key(entry["atom"])
        if key in seen:
            raise EffectOracleError(
                DUPLICATE_EFFECT_ATOM,
                f"effect atom {key[0]}/{key[1]} on port {key[2]} is "
                "declared twice")
        seen.add(key)
        keyed.append((key, normalized_entry(entry)))
    keyed.sort(key=lambda pair: sort_key_of(pair[0]))
    return [normalized
            for _key, normalized in keyed
            if not entry_cannot_occur(normalized["bounds"])]


def sort_key_of(key):
    """A byte-comparable form of an atom key (catalog, name, port, scope,
    qualifier families)."""
    catalog, name, port, scope, families = key
    return (catalog.encode("utf-8"), name.encode("utf-8"), port,
            tuple((k.encode("utf-8"), v.encode("utf-8")) for k, v in scope),
            tuple(families))


def residual_sort_key(residual):
    return (residual["catalog"].encode("utf-8"),
            residual["selector"]["requirement_port_no"],
            tuple((k.encode("utf-8"), v.encode("utf-8"))
                  for k, v in scope_pairs(residual["selector"])))


FAMILY_CASE_NUMBERS = {"idempotency": 1}


def normalized_entry(entry):
    """Canonical atom internals: scope bindings by (key, value), qualifier
    families by case number. Input order is not identity."""
    normalized = copy.deepcopy(entry)
    selector = normalized["atom"]["resource_selector"]
    selector["scope_bindings"] = [
        {"key": k, "value": v} for k, v in scope_pairs(selector)]
    normalized["atom"]["qualifiers"] = sorted(
        normalized["atom"]["qualifiers"],
        key=lambda q: FAMILY_CASE_NUMBERS[next(iter(q))])
    return normalized


def normalized_residual(residual):
    normalized = copy.deepcopy(residual)
    normalized["selector"]["scope_bindings"] = [
        {"key": k, "value": v}
        for k, v in scope_pairs(normalized["selector"])]
    return normalized


def canonical_residuals(residuals):
    keyed = []
    for residual in residuals:
        if not is_specification_id(residual["catalog"]):
            raise EffectOracleError(
                INVALID_EFFECT_OPERATION,
                f"residual catalog {residual['catalog']!r} is not a "
                "SpecificationId")
        check_selector(residual["selector"])
        check_bounds(residual["bounds"])
        if (not residual["selector"]["scope_bindings"]
                and residual["bounds"]["upper"] == "unbounded"):
            raise EffectOracleError(
                UNBOUNDED_OPEN_EFFECT_ROW,
                f"residual catalog {residual['catalog']} on port "
                f"{residual['selector']['requirement_port_no']} has no "
                "scope binding and an unbounded occurrence upper: its "
                "ceiling proves nothing")
        keyed.append((residual_sort_key(residual), normalized_residual(residual)))
    keyed.sort(key=lambda pair: pair[0])
    return [normalized
            for _key, normalized in keyed
            if not entry_cannot_occur(normalized["bounds"])]


def canonical_row(row):
    """row: serde-form dict. Returns the canonical row, still in serde
    form (validate() converts unit variants afterwards)."""
    if "closed" in row:
        return {"closed": canonical_entries(row["closed"])}
    if "open" in row:
        known = canonical_entries(row["open"]["known"])
        residual = canonical_residuals(row["open"]["residual"])
        if not known and not residual:
            return {"closed": []}
        return {"open": {"known": known, "residual": residual}}
    if "unknown" in row or row == "unknown":
        return "unknown"
    raise EffectOracleError(INVALID_EFFECT_QUALIFIER,
                            f"unknown row state {row!r}")


def validate_structure(signature):
    """Full structural validation; returns the canonical signature dict
    (value-tree row form)."""
    return {"row": canonical_row(signature["row"])}


def resolve_operations(canonical, catalogs):
    """Fail-closed resolution against the explicit catalog universe."""
    universe = {c["specification_id"]: set(c["operations"]) for c in catalogs}
    row = canonical["row"]
    entries = []
    residuals = []
    if "closed" in row:
        entries = row["closed"]
    elif "open" in row:
        entries = row["open"]["known"]
        residuals = row["open"]["residual"]
    for entry in entries:
        ref = entry["atom"]["operation"]
        if ref["catalog"] not in universe \
                or ref["name"] not in universe[ref["catalog"]]:
            raise EffectOracleError(
                UNKNOWN_EFFECT_OPERATION,
                f"operation {ref['catalog']}/{ref['name']} does not resolve "
                "in the supplied catalog universe")
    for residual in residuals:
        if residual["catalog"] not in universe:
            raise EffectOracleError(
                UNKNOWN_EFFECT_OPERATION,
                f"residual catalog {residual['catalog']} is not registered "
                "in the supplied universe")


def to_value_tree(row):
    """Unit variants carry no payload: their serde form is a bare string,
    their ACE1 value-tree form a single-key dict."""
    if row == "unknown":
        return {"unknown": None}
    entries = row.get("closed")
    if entries is not None:
        return {"closed": [vt_entry(entry) for entry in entries]}
    open_row = row["open"]
    return {"open": {
        "known": [vt_entry(entry) for entry in open_row["known"]],
        "residual": [vt_residual(item) for item in open_row["residual"]],
    }}


def vt_entry(entry):
    return {
        "atom": entry["atom"],
        "bounds": vt_bounds(entry["bounds"]),
        "guard": ({"unconditional": None}
                  if entry["guard"] == "unconditional" else entry["guard"]),
    }


def vt_residual(item):
    return {
        "catalog": item["catalog"],
        "selector": item["selector"],
        "bounds": vt_bounds(item["bounds"]),
    }


def vt_bounds(bounds):
    upper = bounds["upper"]
    if upper == "unbounded":
        upper = {"unbounded": None}
    return {"lower": bounds["lower"], "upper": upper}


def validate(signature, catalogs=None):
    """Validates the serde-form signature and returns the canonical
    signature in value-tree form."""
    canonical = validate_structure(signature)
    if catalogs is not None:
        resolve_operations(canonical, catalogs)
    return {"row": to_value_tree(canonical["row"])}


def classification(canonical):
    row = canonical["row"]
    if "closed" in row:
        return "pure" if not row["closed"] else "closed"
    if "open" in row:
        return "open"
    return "unknown"


# ---------------------------------------------------------------------------
# Occurrence arithmetic (checked)
# ---------------------------------------------------------------------------

class Overflow(EffectOracleError):
    def __init__(self, detail):
        super().__init__(EFFECT_ARITHMETIC_OVERFLOW, detail)


def checked_add(a, b):
    if a + b > UINT64_MAX:
        raise Overflow(f"occurrence lower bounds {a} + {b} overflow uint64")
    return a + b


def ceiling_add(a, b):
    ua, ub = ceiling_upper(a), ceiling_upper(b)
    if ua is None or ub is None:
        return "unbounded"
    return {"finite": checked_add(ua, ub)}


def ceiling_mul(a, b):
    ua, ub = ceiling_upper(a), ceiling_upper(b)
    if ua == 0 or ub == 0:
        return {"finite": 0}
    if ua is None or ub is None:
        return "unbounded"
    if ua * ub > UINT64_MAX:
        raise Overflow(f"occurrence ceiling {ua} * {ub} overflow uint64")
    return {"finite": ua * ub}


def bounds_add(x, y):
    return {
        "lower": checked_add(x["lower"], y["lower"]),
        "upper": ceiling_add(x["upper"], y["upper"]),
    }


def bounds_mul(x, reps):
    lower = x["lower"] * reps["lower"]
    if lower > UINT64_MAX:
        raise Overflow(
            f"occurrence lower bounds {x['lower']} * {reps['lower']} "
            "overflow uint64")
    return {"lower": lower, "upper": ceiling_mul(x["upper"], reps["upper"])}


def ceiling_max(a, b):
    ua, ub = ceiling_upper(a), ceiling_upper(b)
    if ua is None or ub is None:
        return "unbounded"
    return {"finite": max(ua, ub)}


def bounds_cover(outer, inner):
    uo, ui = ceiling_upper(outer["upper"]), ceiling_upper(inner["upper"])
    if outer["lower"] > inner["lower"]:
        return False
    if ui is None:
        return uo is None
    return uo is None or uo >= ui


# ---------------------------------------------------------------------------
# The algebra
# ---------------------------------------------------------------------------

def merge_entries(entries, combine):
    """combine(existing_entry, incoming_entry) -> entry; keyed by atom."""
    merged = {}
    order = []
    for entry in entries:
        key = sort_key_of(atom_key(entry["atom"]))
        if key in merged:
            merged[key] = combine(merged[key], entry)
        else:
            merged[key] = copy.deepcopy(entry)
            order.append(key)
    return [merged[key] for key in sorted(merged)]


def accumulate(a, b):
    a, b = validate(a), validate(b)
    if "unknown" in a["row"] or "unknown" in b["row"]:
        return {"row": {"unknown": None}}
    entries = []

    def combine(existing, incoming):
        upper = ceiling_add(existing["bounds"]["upper"],
                            incoming["bounds"]["upper"])
        same_guard = existing["guard"] == incoming["guard"]
        if same_guard and existing["guard"] == {"unconditional": None}:
            lower = checked_add(existing["bounds"]["lower"],
                                incoming["bounds"]["lower"])
        else:
            lower = 0
        guard = (existing["guard"] if same_guard
                 else {"unconditional": None})
        return {
            "atom": existing["atom"],
            "bounds": {"lower": lower, "upper": upper},
            "guard": guard,
        }

    row_a, row_b = a["row"], b["row"]
    known_a = row_a.get("closed", row_a.get("open", {}).get("known", []))
    known_b = row_b.get("closed", row_b.get("open", {}).get("known", []))
    entries = merge_entries(known_a + known_b, combine)
    residuals = merge_residuals(
        row_a.get("open", {}).get("residual", []),
        row_b.get("open", {}).get("residual", []),
        bounds_add)
    return assemble(entries, residuals)


def merge_residuals(residuals_a, residuals_b, combine):
    merged = {}
    for residual in residuals_a + residuals_b:
        key = residual_sort_key(residual)
        if key in merged:
            merged[key] = {
                "catalog": merged[key]["catalog"],
                "selector": merged[key]["selector"],
                "bounds": combine(merged[key]["bounds"], residual["bounds"]),
            }
        else:
            merged[key] = copy.deepcopy(residual)
    return [merged[key] for key in sorted(merged)]


def assemble(entries, residuals):
    if not residuals:
        return {"row": {"closed": entries}}
    return {"row": {"open": {"known": entries, "residual": residuals}}}


def join_one(existing, incoming):
    same_guard = existing["guard"] == incoming["guard"]
    if same_guard:
        return {
            "atom": existing["atom"],
            "bounds": {
                "lower": min(existing["bounds"]["lower"],
                             incoming["bounds"]["lower"]),
                "upper": ceiling_max(existing["bounds"]["upper"],
                                     incoming["bounds"]["upper"]),
            },
            "guard": existing["guard"],
        }
    return {
        "atom": existing["atom"],
        "bounds": {
            "lower": 0,
            "upper": ceiling_max(existing["bounds"]["upper"],
                                 incoming["bounds"]["upper"]),
        },
        "guard": {"unconditional": None},
    }


def branch_join(a, b):
    a, b = validate(a), validate(b)
    if "unknown" in a["row"] or "unknown" in b["row"]:
        return {"row": {"unknown": None}}
    row_a, row_b = a["row"], b["row"]
    known_a = row_a.get("closed", row_a.get("open", {}).get("known", []))
    known_b = row_b.get("closed", row_b.get("open", {}).get("known", []))

    # Both-declared keys keep the guaranteed minimum lower; one-sided keys
    # (from either side) drop to zero.
    keyed = {}
    order = []
    for entry in known_a + known_b:
        key = sort_key_of(atom_key(entry["atom"]))
        if key not in keyed:
            keyed[key] = {"entry": copy.deepcopy(entry), "both": False}
            order.append(key)
        elif not keyed[key]["both"]:
            keyed[key]["entry"] = join_one(keyed[key]["entry"], entry)
            keyed[key]["both"] = True
    entries = []
    for key in sorted(keyed):
        slot = keyed[key]
        entry = slot["entry"]
        if not slot["both"]:
            entry = {
                "atom": entry["atom"],
                "bounds": {"lower": 0, "upper": entry["bounds"]["upper"]},
                "guard": entry["guard"],
            }
        entries.append(entry)

    residuals = {}
    for residual in (row_a.get("open", {}).get("residual", [])
                     + row_b.get("open", {}).get("residual", [])):
        key = residual_sort_key(residual)
        zeroed = {
            "catalog": residual["catalog"],
            "selector": residual["selector"],
            "bounds": {"lower": 0, "upper": residual["bounds"]["upper"]},
        }
        if key in residuals:
            residuals[key] = {
                "catalog": residuals[key]["catalog"],
                "selector": residuals[key]["selector"],
                "bounds": {
                    "lower": 0,
                    "upper": ceiling_max(residuals[key]["bounds"]["upper"],
                                         zeroed["bounds"]["upper"]),
                },
            }
        else:
            residuals[key] = zeroed
    return assemble(entries, [residuals[key] for key in sorted(residuals)])


def repeat_op(reps, effect):
    check_bounds(reps)
    effect = validate(effect)
    if "unknown" in effect["row"]:
        return {"row": {"unknown": None}}
    row = effect["row"]

    def scale_entry(entry):
        bounds = bounds_mul(entry["bounds"], reps)
        if ceiling_upper(bounds["upper"]) == 0:
            return None
        return {
            "atom": entry["atom"],
            "bounds": bounds,
            "guard": entry["guard"],
        }

    if "closed" in row:
        return {"row": {"closed": [scaled for scaled in
                                   map(scale_entry, row["closed"])
                                   if scaled is not None]}}
    known = [s for s in map(scale_entry, row["open"]["known"])
             if s is not None]
    residual = []
    for item in row["open"]["residual"]:
        bounds = bounds_mul(item["bounds"], reps)
        if ceiling_upper(bounds["upper"]) != 0:
            residual.append({
                "catalog": item["catalog"],
                "selector": item["selector"],
                "bounds": bounds,
            })
    if not known and not residual:
        return {"row": {"closed": []}}
    return {"row": {"open": {"known": known, "residual": residual}}}


# --- containment -------------------------------------------------------------

def guard_contained(inner, outer):
    if outer == "unconditional" or outer == {"unconditional": None}:
        return True
    if inner == "unconditional" or inner == {"unconditional": None}:
        return False
    return inner == outer


def pair_scopes(atom):
    return scope_pairs(atom["resource_selector"])


def entry_covered_by_known(entry, candidate):
    a = atom_key(entry["atom"])
    b = atom_key(candidate["atom"])
    # Exact operation, equal qualifiers; scope of a narrower or equal
    # (every constraint of b present in a).
    if (a[0], a[1], a[2], a[4]) != (b[0], b[1], b[2], b[4]):
        return False
    if not all(binding in a[3] for binding in b[3]):
        return False
    if not bounds_cover(candidate["bounds"], entry["bounds"]):
        return False
    return guard_contained(entry["guard"], candidate["guard"])


def entry_covered_by_residual(entry, residual):
    a = atom_key(entry["atom"])
    if residual["catalog"] != a[0]:
        return False
    if residual["selector"]["requirement_port_no"] != a[2]:
        return False
    residual_pairs = scope_pairs(residual["selector"])
    if not all(binding in a[3] for binding in residual_pairs):
        return False
    return bounds_cover(residual["bounds"], entry["bounds"])


def residual_covered_by(inner, outer):
    inner_pairs = scope_pairs(inner["selector"])
    outer_pairs = scope_pairs(outer["selector"])
    return (inner["catalog"] == outer["catalog"]
            and inner["selector"]["requirement_port_no"]
            == outer["selector"]["requirement_port_no"]
            and all(binding in inner_pairs for binding in outer_pairs)
            and bounds_cover(outer["bounds"], inner["bounds"]))


def contains(a, b):
    """Returns (proven, reason_or_None)."""
    try:
        a, b = validate(a), validate(b)
    except EffectOracleError:
        return (False, "unknown_row")
    if "unknown" in b["row"]:
        return (True, None)
    if "unknown" in a["row"]:
        return (False, "unknown_row")
    row_a, row_b = a["row"], b["row"]
    known_a = row_a.get("closed", row_a.get("open", {}).get("known", []))
    known_b = row_b.get("closed", row_b.get("open", {}).get("known", []))
    resid_a = row_a.get("open", {}).get("residual", [])
    resid_b = row_b.get("open", {}).get("residual", [])
    if not resid_b and resid_a:
        return (False, "residual_inside_closed")
    known_slots = [(entry, True) for entry in known_b]
    residual_slots = [(residual, True) for residual in resid_b]
    for entry in known_a:
        claimed = False
        for index, (candidate, free) in enumerate(known_slots):
            if free and entry_covered_by_known(entry, candidate):
                known_slots[index] = (candidate, False)
                claimed = True
                break
        if not claimed:
            for index, (candidate, free) in enumerate(residual_slots):
                if free and entry_covered_by_residual(entry, candidate):
                    residual_slots[index] = (candidate, False)
                    claimed = True
                    break
        if not claimed:
            return (False, "entry_uncovered")
    for residual in resid_a:
        claimed = False
        for index, (candidate, free) in enumerate(residual_slots):
            if free and residual_covered_by(residual, candidate):
                residual_slots[index] = (candidate, False)
                claimed = True
                break
        if not claimed:
            return (False, "residual_uncovered")
    return (True, None)


# ---------------------------------------------------------------------------
# Control propagation
# ---------------------------------------------------------------------------

def pure():
    return {"row": {"closed": []}}


def sequence_envelope(a, b):
    return accumulate(a, b)


def parallel_envelope(a, b):
    return accumulate(a, b)


def select_envelope(a, b):
    return branch_join(a, b)


def race_envelope(branches):
    envelope = pure()
    for branch in branches:
        envelope = accumulate(envelope, branch)
    return envelope


def iterate_envelope(reps, body):
    return repeat_op(reps, body)


RETRY_NARROWING_OBLIGATION = \
    "retry_logical_narrowing_requires_recovery_contract"


def retry_envelope(child, max_attempts):
    if max_attempts < 1:
        raise EffectOracleError(
            INVALID_OCCURRENCE_BOUNDS,
            "a retry makes at least one attempt")
    attempts = {"lower": 1, "upper": {"finite": max_attempts}}
    logical = repeat_op(attempts, child)
    return {
        "logical": logical,
        "attempts": attempts,
        "logical_narrowing_proven": False,
        "obligations": [RETRY_NARROWING_OBLIGATION],
    }


def compensation_envelope(forward, counter):
    return accumulate(forward, counter)


# ---------------------------------------------------------------------------
# Fixture builders (serde-form dicts)
# ---------------------------------------------------------------------------

def atom(op, port, scope=None, qualifier=None):
    return {
        "operation": {"catalog": CATALOG, "name": op},
        "resource_selector": {
            "requirement_port_no": port,
            "scope_bindings": [{"key": k, "value": v}
                               for k, v in (scope or [])],
        },
        "qualifiers": ([{"idempotency": qualifier}] if qualifier else []),
    }


def entry(op, port, scope=None, lower=1, upper=1, guard="unconditional",
          qualifier=None, catalog=None):
    built = atom(op, port, scope, qualifier)
    if catalog is not None:
        built["operation"]["catalog"] = catalog
    ceiling = "unbounded" if upper == "unbounded" else {"finite": upper}
    return {"atom": built, "bounds": {"lower": lower, "upper": ceiling},
            "guard": guard}


def guarded_entry(op, port, expected, guard_port=1, scope=None, lower=1,
                  upper=1):
    return entry(op, port, scope, lower, upper,
                 guard={"input_equals": {"port_no": guard_port,
                                         "expected": expected}})


def sig_closed(entries):
    return {"row": {"closed": entries}}


def sig_open(known, residual):
    return {"row": {"open": {"known": known, "residual": residual}}}


def sig_unknown():
    return {"row": "unknown"}


def residual(catalog, port, scope, lower, upper):
    ceiling = "unbounded" if upper == "unbounded" else {"finite": upper}
    return {
        "catalog": catalog,
        "selector": {
            "requirement_port_no": port,
            "scope_bindings": [{"key": k, "value": v} for k, v in scope],
        },
        "bounds": {"lower": lower, "upper": ceiling},
    }


def std_catalogs(with_alt=False):
    catalogs = [{"specification_id": CATALOG, "operations": CATALOG_OPS}]
    if with_alt:
        catalogs.append({"specification_id": ALT_CATALOG,
                         "operations": ALT_CATALOG_OPS})
    return catalogs


# ---------------------------------------------------------------------------
# Corpus assembly
# ---------------------------------------------------------------------------

CASES = []


def write_case(subdir, payload):
    path = CONF / subdir / f"{payload['case_id'].replace('/', '-')}.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    text = json.dumps(payload, indent=2, ensure_ascii=False, sort_keys=True)
    path.write_text(text + "\n", encoding="utf-8", newline="\n")
    print(f"wrote {path.relative_to(ROOT)}")
    CASES.append(payload["case_id"])


def golden(case_id, note, signature, catalogs=None):
    catalogs = catalogs if catalogs is not None else std_catalogs()
    canonical = validate(signature, catalogs)
    text = canonical_ace1(canonical["row"])
    write_case("golden", {
        "kind": "agentflow.conformance.effect-signature.case",
        "case_kind": "golden",
        "corpus_revision": CORPUS_REVISION,
        "case_id": case_id,
        "note": note,
        "fixture_catalogs": catalogs,
        "signature": signature,
        "canonical_ace1_utf8": text,
        "is_pure": classification(canonical) == "pure",
        "classification": classification(canonical),
    })


def negative(case_id, note, signature, expected_kind, catalogs=None):
    """A validator rejection: the oracle must raise exactly this kind."""
    catalogs = catalogs if catalogs is not None else std_catalogs()
    try:
        validate(signature, catalogs)
    except EffectOracleError as error:
        if error.kind != expected_kind:
            raise AssertionError(
                f"{case_id}: oracle raised {error.kind}, expected "
                f"{expected_kind}") from error
    else:
        raise AssertionError(f"{case_id}: expected rejection {expected_kind}")
    write_case("negative", {
        "kind": "agentflow.conformance.effect-signature.case",
        "case_kind": "negative",
        "corpus_revision": CORPUS_REVISION,
        "case_id": case_id,
        "note": note,
        "fixture_catalogs": catalogs,
        "signature": signature,
        "expected_kind": expected_kind,
    })


def algebra(case_id, note, operation, operands, expected, catalogs=None,
            reps=None, max_attempts=None, property_kind=None):
    catalogs = catalogs if catalogs is not None else std_catalogs()
    for operand in operands:
        validate(operand, catalogs)
    write_case("algebra", {
        "kind": "agentflow.conformance.effect-signature.case",
        "case_kind": "algebra",
        "corpus_revision": CORPUS_REVISION,
        "case_id": case_id,
        "note": note,
        "fixture_catalogs": catalogs,
        "operation": operation,
        "operands": operands,
        "reps": reps,
        "max_attempts": max_attempts,
        "expected": expected,
        "property": property_kind,
    })


def equivalence(case_id, note, members, canonical_row_text, catalogs=None):
    catalogs = catalogs if catalogs is not None else std_catalogs()
    for member in members:
        canonical = validate(member, catalogs)
        if canonical_ace1(canonical["row"]) != canonical_row_text:
            raise AssertionError(
                f"{case_id}: equivalence member drifted off the frozen "
                "canonical bytes")
    write_case("equivalence", {
        "kind": "agentflow.conformance.effect-signature.case",
        "case_kind": "equivalence",
        "corpus_revision": CORPUS_REVISION,
        "case_id": case_id,
        "note": note,
        "fixture_catalogs": catalogs,
        "members": members,
        "canonical_ace1_utf8": canonical_row_text,
    })


def result_ace1(result_row):
    return canonical_ace1(result_row)


def main():
    if CONF.exists():
        for path in CONF.rglob("*.json"):
            path.unlink()

    # --- Golden ----------------------------------------------------------

    golden("effect/golden/pure-empty-row",
           "the closed empty row is the one representation of pure",
           sig_closed([]))

    golden("effect/golden/single-read",
           "reads are effects: one database read with an exact table scope",
           sig_closed([entry("database.row.read", 1,
                             [("table", "customer")], 1, 1)]))

    golden("effect/golden/single-write",
           "one issue creation with an exact repo scope and [1,3] bounds",
           sig_closed([entry("github.issue.create", 1,
                             [("repo", "foo/bar")], 1, 3)]))

    golden("effect/golden/multiple-distinct",
           "distinct atoms stay side by side; input order is not canonical "
           "order (charge, read, write in canonical order)",
           sig_closed([
               entry("payment.charge", 2, [("merchant", "acme")], 1, 1),
               entry("github.issue.create", 1, [("repo", "foo/bar")], 1, 2),
               entry("database.row.write", 3, [("table", "customer")], 0, 4),
           ]))

    golden("effect/golden/occurrence-bounds",
           "explicit bounds with the unbounded ceiling and an explicit "
           "idempotency-none qualifier",
           sig_closed([entry("payment.charge", 1, [("merchant", "acme")],
                             2, "unbounded", qualifier="none")]))

    golden("effect/golden/bounded-row",
           "a closed row mixing finite and unbounded entry bounds",
           sig_closed([
               entry("github.issue.read", 1, [("repo", "foo/bar")], 0, 10),
               entry("network.egress", 2, [("host", "docs.example.com")],
                     1, "unbounded"),
           ]))

    golden("effect/golden/bounded-open-row",
           "a bounded-open row: one known effect plus a residual ceiling "
           "from the same catalog with a scope binding and a finite upper",
           sig_open(
               [entry("github.issue.read", 1, [("repo", "foo/bar")], 1, 2)],
               [residual(CATALOG, 1, [("repo", "foo/bar")], 0, 5)]),
           catalogs=std_catalogs(with_alt=True))

    golden("effect/golden/guarded",
           "a typed input-equality guard narrows when the effect can occur",
           sig_closed([guarded_entry("payment.charge", 1,
                                     {"int64": 1}, guard_port=1,
                                     scope=[("merchant", "acme")], lower=0,
                                     upper=2)]))

    golden("effect/golden/resource-scope",
           "exact-binding scope narrowing with an idempotency-natural "
           "qualifier",
           sig_closed([entry("memory.append", 1,
                             [("scope", "tenant-42"), ("store", "notes")],
                             1, 8, qualifier="natural")]))

    golden("effect/golden/zero-occurrence-entry",
           "an entry that cannot occur is semantically pure: the canonical "
           "form drops it and the row becomes the empty closed row",
           sig_closed([entry("github.issue.create", 1,
                             [("repo", "foo/bar")], 0, 0)]))

    golden("effect/golden/canonical-reordering",
           "the same declarations in non-canonical input order: canonical "
           "bytes follow the atom key order, never the input order",
           sig_closed([
               entry("network.egress", 1, [("host", "a.example.com")], 1, 1),
               entry("database.row.read", 2, [("table", "customer")], 1, 1),
               entry("database.row.read", 1, [("table", "orders")], 1, 1),
           ]))

    # --- Negative --------------------------------------------------------

    negative("effect/negative/unknown-operation",
             "a well-formed operation that the catalog does not declare is "
             "rejected fail-closed",
             sig_closed([entry("github.issue.merge", 1,
                               [("repo", "foo/bar")], 1, 1)]),
             UNKNOWN_EFFECT_OPERATION)

    negative("effect/negative/invalid-operation-name",
             "operation names are lowercase dotted segments; uppercase is "
             "a grammar violation",
             sig_closed([entry("GitHub.Issue.Create", 1, [], 1, 1)]),
             INVALID_EFFECT_OPERATION)

    negative("effect/negative/invalid-catalog-identity",
             "a catalog identity must be a full SpecificationId (missing "
             "@major here)",
             sig_closed([entry("github.issue.create", 1, [], 1, 1,
                               catalog="agentflow.test.capability-catalog")]),
             INVALID_EFFECT_OPERATION)

    negative("effect/negative/zero-requirement-port",
             "requirement ports are 1-based; port 0 is invalid",
             sig_closed([entry("github.issue.create", 0, [], 1, 1)]),
             INVALID_RESOURCE_SELECTOR)

    negative("effect/negative/duplicate-scope-key",
             "at most one exact binding per scope key",
             sig_closed([entry("github.issue.create", 1,
                               [("repo", "foo/bar"), ("repo", "baz/qux")],
                               1, 1)]),
             INVALID_RESOURCE_SELECTOR)

    negative("effect/negative/occurrence-lower-above-upper",
             "an occurrence interval with lower above the finite upper is "
             "impossible",
             sig_closed([entry("github.issue.create", 1, [], 5, 2)]),
             INVALID_OCCURRENCE_BOUNDS)

    conflicting = atom("payment.charge", 1, [("merchant", "acme")])
    conflicting["qualifiers"] = [{"idempotency": "natural"},
                                 {"idempotency": "none"}]
    negative("effect/negative/conflicting-qualifiers",
             "one qualifier family may appear at most once per atom",
             sig_closed([{"atom": conflicting,
                          "bounds": {"lower": 1, "upper": {"finite": 1}},
                          "guard": "unconditional"}]),
             INVALID_EFFECT_QUALIFIER)

    negative("effect/negative/invalid-guard-port",
             "guard input ports are 1-based; port 0 is invalid",
             sig_closed([guarded_entry("payment.charge", 1, {"int64": 1},
                                       guard_port=0, lower=0, upper=1)]),
             INVALID_EFFECT_GUARD)

    negative("effect/negative/duplicate-atom",
             "the same canonical atom declared twice in one row is a "
             "duplicate, never a silent merge",
             sig_closed([
                 entry("github.issue.create", 1, [("repo", "foo/bar")], 1, 1),
                 entry("github.issue.create", 1, [("repo", "foo/bar")], 0, 2),
             ]),
             DUPLICATE_EFFECT_ATOM)

    negative("effect/negative/unbounded-open-row",
             "a residual with no scope binding and an unbounded upper "
             "proves nothing and is rejected",
             sig_open([], [residual(CATALOG, 1, [], 0, "unbounded")]),
             UNBOUNDED_OPEN_EFFECT_ROW)

    # The top-row observations: accepted as declared states, never pure,
    # never authorization.
    top = sig_unknown()
    top_canonical = validate(top)
    top_bytes = canonical_ace1(top_canonical["row"])
    assert top_bytes == '{"row":{"unknown":null}}', top_bytes

    write_case("negative", {
        "kind": "agentflow.conformance.effect-signature.case",
        "case_kind": "negative",
        "corpus_revision": CORPUS_REVISION,
        "case_id": "effect/negative/top-is-not-pure",
        "note": "the unknown row validates as a declared state but is "
                "never pure: classification is unknown and is_pure false",
        "fixture_catalogs": std_catalogs(),
        "signature": top,
        "expect_validation": "accept",
        "classification": "unknown",
        "is_pure": False,
    })

    write_case("negative", {
        "kind": "agentflow.conformance.effect-signature.case",
        "case_kind": "negative",
        "corpus_revision": CORPUS_REVISION,
        "case_id": "effect/negative/top-is-not-authorization",
        "note": "the unknown row carries no capability material: its "
                "canonical form is the literal unknown encoding, and "
                "demanding containment inside a closed limit fails closed "
                "with effect_envelope_unprovable",
        "fixture_catalogs": std_catalogs(),
        "signature": top,
        "expect_validation": "accept",
        "canonical_ace1_utf8": top_bytes,
        "containment_limit": sig_closed([entry("github.issue.read", 1,
                                               [], 0, 100)]),
        "containment_proven": False,
        "containment_error_kind": EFFECT_ENVELOPE_UNPROVABLE,
    })

    # --- Algebra ---------------------------------------------------------

    write_entry = entry
    x_sig = sig_closed([write_entry("github.issue.create", 1,
                                    [("repo", "foo/bar")], 1, 2)])
    x_bytes = result_ace1(validate(x_sig)["row"])

    # empty identity for accumulation
    assert result_ace1(accumulate(pure(), x_sig)["row"]) == x_bytes
    assert result_ace1(accumulate(x_sig, pure())["row"]) == x_bytes
    algebra("effect/algebra/empty-identity",
            "the pure row is the two-sided identity of accumulation",
            "accumulate", [pure(), x_sig],
            {"canonical_ace1_utf8": x_bytes},
            property_kind="identity")

    # associativity
    a_sig = sig_closed([write_entry("github.issue.create", 1,
                                    [("repo", "foo/bar")], 1, 2)])
    b_sig = sig_closed([write_entry("database.row.write", 2,
                                    [("table", "customer")], 0, 3)])
    c_sig = sig_closed([write_entry("payment.charge", 3,
                                    [("merchant", "acme")], 1, 1)])
    left = accumulate(accumulate(a_sig, b_sig), c_sig)
    right = accumulate(a_sig, accumulate(b_sig, c_sig))
    assoc_bytes = result_ace1(left["row"])
    assert assoc_bytes == result_ace1(right["row"]), "associativity broke"
    algebra("effect/algebra/accumulation-associativity",
            "accumulation is associative over canonical signatures",
            "accumulate", [a_sig, b_sig, c_sig],
            {"canonical_ace1_utf8": assoc_bytes},
            property_kind="associativity")

    # commutativity of branch join (with guard widening frozen)
    ga = guarded_entry("payment.charge", 1, {"int64": 1}, guard_port=1,
                       scope=[("merchant", "acme")], lower=1, upper=2)
    gb = write_entry("payment.charge", 1, [("merchant", "acme")], 0, 3)
    joined = select_envelope(sig_closed([ga]), sig_closed([gb]))
    joined_backwards = select_envelope(sig_closed([gb]), sig_closed([ga]))
    joined_bytes = result_ace1(joined["row"])
    assert joined_bytes == result_ace1(joined_backwards["row"])
    algebra("effect/algebra/join-commutativity",
            "branch join is commutative; different guards widen to "
            "unconditional",
            "select_envelope", [sig_closed([ga]), sig_closed([gb])],
            {"canonical_ace1_utf8": joined_bytes},
            property_kind="commutativity")

    # idempotence of branch join
    idem = branch_join(a_sig, a_sig)
    idem_bytes = result_ace1(idem["row"])
    assert idem_bytes == result_ace1(validate(a_sig)["row"]), \
        "join idempotence broke"
    algebra("effect/algebra/join-idempotence",
            "branch join with itself reproduces the branch envelope",
            "branch_join", [a_sig, a_sig],
            {"canonical_ace1_utf8": idem_bytes},
            property_kind="idempotence")

    # repetition
    zero = repeat_op({"lower": 0, "upper": {"finite": 0}}, a_sig)
    assert classification(zero) == "pure"
    algebra("effect/algebra/repeat-zero",
            "zero repetitions of any envelope are pure",
            "repeat", [a_sig],
            {"canonical_ace1_utf8": result_ace1(zero["row"])},
            reps={"lower": 0, "upper": {"finite": 0}})

    one = repeat_op({"lower": 1, "upper": {"finite": 1}}, a_sig)
    assert result_ace1(one["row"]) == result_ace1(validate(a_sig)["row"])
    algebra("effect/algebra/repeat-one",
            "one repetition is the identity of the body envelope",
            "repeat", [a_sig],
            {"canonical_ace1_utf8": result_ace1(one["row"])},
            reps={"lower": 1, "upper": {"finite": 1}})

    n_sig = repeat_op({"lower": 3, "upper": {"finite": 5}}, a_sig)
    n_bytes = result_ace1(n_sig["row"])
    n_entry = n_sig["row"]["closed"][0]
    assert n_entry["bounds"] == {"lower": 3, "upper": {"finite": 10}}
    algebra("effect/algebra/repeat-n",
            "n ⊗ ε scales the interval to [3·1, 5·2] = [3, 10]",
            "iterate_envelope", [a_sig],
            {"canonical_ace1_utf8": n_bytes},
            reps={"lower": 3, "upper": {"finite": 5}})

    # containment reflexivity (closed and open rows, and top)
    open_self = sig_open(
        [write_entry("github.issue.read", 1, [("repo", "foo/bar")], 1, 2)],
        [residual(CATALOG, 1, [("repo", "foo/bar")], 0, 5)])
    for probe, name in ((a_sig, "closed"), (open_self, "open"),
                        (sig_unknown(), "unknown")):
        proven, reason = contains(probe, probe)
        assert proven, f"reflexivity failed for {name}: {reason}"
    algebra("effect/algebra/containment-reflexivity",
            "every row is contained in itself: closed, open and unknown",
            "contains", [open_self],
            {"proven": True, "reason": None})

    # containment transitivity where provable (exact-binding scope chains)
    narrow = sig_closed([write_entry("github.issue.create", 1,
                                     [("repo", "foo/bar"),
                                      ("branch", "main")], 1, 2)])
    medium = sig_closed([write_entry("github.issue.create", 1,
                                     [("repo", "foo/bar")], 0, 5)])
    wide = sig_closed([write_entry("github.issue.create", 1, [], 0, 9)])
    proven_pairs = []
    for i, inner in enumerate((narrow, medium, wide)):
        for j, outer in enumerate((narrow, medium, wide)):
            if i < j:
                proven, reason = contains(inner, outer)
                assert proven, f"transitivity chain broke at {i}->{j}: {reason}"
                proven_pairs.append([i, j])
    algebra("effect/algebra/containment-transitivity",
            "exact-binding scope chains prove transitively: "
            "narrow ⊑ medium ⊑ wide implies narrow ⊑ wide",
            "contains", [narrow, medium, wide],
            {"proven_pairs": proven_pairs},
            property_kind="transitivity")

    # sequence / parallel accumulation
    seq = sequence_envelope(a_sig, b_sig)
    seq_bytes = result_ace1(seq["row"])
    seq_entries = seq["row"]["closed"]
    assert len(seq_entries) == 2, "distinct atoms stay side by side"
    by_name = {e["atom"]["operation"]["name"]: e for e in seq_entries}
    assert by_name["github.issue.create"]["bounds"] == \
        {"lower": 1, "upper": {"finite": 2}}
    assert by_name["database.row.write"]["bounds"] == \
        {"lower": 0, "upper": {"finite": 3}}
    algebra("effect/algebra/sequence-accumulation",
            "SEQUENCE(A, B) = εA ⊕ εB: intervals add per atom key",
            "sequence_envelope", [a_sig, b_sig],
            {"canonical_ace1_utf8": seq_bytes})

    par = parallel_envelope(a_sig, b_sig)
    assert result_ace1(par["row"]) == seq_bytes
    algebra("effect/algebra/parallel-accumulation",
            "PARALLEL(A, B) = εA ⊕ εB: concurrency limits never change "
            "total occurrence bounds",
            "parallel_envelope", [a_sig, b_sig],
            {"canonical_ace1_utf8": seq_bytes})

    # race: the loser's effects are retained
    loser = sig_closed([write_entry("payment.charge", 3,
                                    [("merchant", "acme")], 1, 1)])
    race = race_envelope([a_sig, b_sig, loser])
    race_bytes = result_ace1(race["row"])
    race_entries = race["row"]["closed"]
    assert len(race_entries) == 3, "the loser effect must survive"
    algebra("effect/algebra/race-loser-retained",
            "RACE conservatively accumulates ALL contestants: a loser may "
            "have acted before cancellation, so its effects are retained",
            "race_envelope", [a_sig, b_sig, loser],
            {"canonical_ace1_utf8": race_bytes})

    # retry: conservative amplification with the explicit obligation
    retry = retry_envelope(a_sig, 3)
    retry_bytes = result_ace1(retry["logical"]["row"])
    retry_entry = retry["logical"]["row"]["closed"][0]
    assert retry_entry["bounds"] == {"lower": 1, "upper": {"finite": 6}}
    assert retry["attempts"] == {"lower": 1, "upper": {"finite": 3}}
    assert retry["logical_narrowing_proven"] is False
    algebra("effect/algebra/retry-amplification",
            "RETRY conservatively scales the logical bounds by the attempt "
            "ceiling ([1,3] ⊗ [1,2] = [1,6]) and records that narrowing "
            "requires a recovery contract",
            "retry_envelope", [a_sig],
            {
                "attempts": retry["attempts"],
                "logical_ace1_utf8": retry_bytes,
                "logical_narrowing_proven": False,
                "obligations": retry["obligations"],
            },
            max_attempts=3)

    # compensation never subtracts
    fwd = sig_closed([write_entry("github.issue.create", 1,
                                  [("repo", "foo/bar")], 2, 3)])
    counter = sig_closed([write_entry("github.issue.create", 1,
                                      [("repo", "foo/bar")], 1, 1)])
    comp = compensation_envelope(fwd, counter)
    comp_entry = comp["row"]["closed"][0]
    assert comp_entry["bounds"] == {"lower": 3, "upper": {"finite": 4}}, \
        comp_entry["bounds"]
    algebra("effect/algebra/compensation-no-subtraction",
            "compensation ADDS the counter-effect to the envelope: the "
            "forward effect is never subtracted from history",
            "compensation_envelope", [fwd, counter],
            {"canonical_ace1_utf8": result_ace1(comp["row"])})

    # overflow is an explicit error
    big = sig_closed([write_entry("github.issue.create", 1,
                                  [("repo", "foo/bar")], 1, UINT64_MAX - 1)])
    try:
        repeat_op({"lower": 4, "upper": {"finite": 4}}, big)
        raise AssertionError("overflow must be detected")
    except Overflow:
        pass
    algebra("effect/algebra/overflow",
            "checked occurrence arithmetic: 4 ⊗ (uint64-max - 1) overflows "
            "and is an explicit error, never a wrap-around",
            "repeat", [big],
            {"error_kind": EFFECT_ARITHMETIC_OVERFLOW},
            reps={"lower": 4, "upper": {"finite": 4}})

    # --- Guarded branch join (Phase 4A.1B) --------------------------------
    #
    # A lower occurrence bound guarded by G is guaranteed only under G.
    # Case A (identical guards): keep the conditional minimum.
    # Case B (differing guards): widen to unconditional, lower -> 0.
    # Case C (one-sided key): [0, u] with the guard kept.

    def gstring(value):
        return {"input_equals": {"port_no": 1, "expected": {"string": value}}}

    # Case A, unconditional: guaranteed lower bounds add no condition.
    ja = sig_closed([entry("github.issue.create", 1, [], 1, 2)])
    jb = sig_closed([entry("github.issue.create", 1, [], 3, 5)])
    j_same_uncond = branch_join(ja, jb)
    assert j_same_uncond["row"]["closed"][0]["bounds"] == {
        "lower": 1, "upper": {"finite": 5}}
    assert j_same_uncond["row"]["closed"][0]["guard"] == {"unconditional": None}
    algebra("effect/algebra/join-same-unconditional-guard",
            "identical unconditional guards keep the tightest unconditional "
            "lower bound [min(1,3), max(2,5)] = [1,5]",
            "branch_join", [ja, jb],
            {"canonical_ace1_utf8": result_ace1(j_same_uncond["row"])})

    # Case A, conditional: the minimum survives AS a conditional bound.
    jc = sig_closed([entry("github.issue.create", 1, [], 2, 4,
                           guard=gstring("a"))])
    jd = sig_closed([entry("github.issue.create", 1, [], 3, 5,
                           guard=gstring("a"))])
    j_same_cond = branch_join(jc, jd)
    assert j_same_cond["row"]["closed"][0]["bounds"] == {
        "lower": 2, "upper": {"finite": 5}}
    assert j_same_cond["row"]["closed"][0]["guard"] == gstring("a")
    algebra("effect/algebra/join-same-conditional-guard",
            'guard(input=="a") [2,4] ⊔ guard(input=="a") [3,5] keeps guard '
            'input_equals("a") with the conditional minimum [2,5]',
            "branch_join", [jc, jd],
            {"canonical_ace1_utf8": result_ace1(j_same_cond["row"])})

    # Case B: the task's key case - lower bounds weaken to zero.
    je = sig_closed([entry("github.issue.create", 1, [], 1, 2,
                           guard=gstring("a"))])
    jf = sig_closed([entry("github.issue.create", 1, [], 1, 3,
                           guard=gstring("b"))])
    j_diff = branch_join(je, jf)
    assert j_diff["row"]["closed"][0]["bounds"] == {
        "lower": 0, "upper": {"finite": 3}}
    assert j_diff["row"]["closed"][0]["guard"] == {"unconditional": None}
    algebra("effect/algebra/join-different-input-equals-guards",
            'guard(input=="a") [1,2] ⊔ guard(input=="b") [1,3] = '
            "unconditional [0,3]: the conditions' union is not expressible "
            "and a positive unconditional lower would claim occurrences "
            "when NEITHER condition holds",
            "branch_join", [je, jf],
            {"canonical_ace1_utf8": result_ace1(j_diff["row"])})

    # Case B mix: an unconditional branch guarantees its lower only when IT
    # runs; the guarded branch may run with its condition false.
    jg = sig_closed([entry("github.issue.create", 1, [], 1, 2)])
    jh = sig_closed([entry("github.issue.create", 1, [], 2, 4,
                           guard=gstring("a"))])
    j_mix = branch_join(jg, jh)
    assert j_mix["row"]["closed"][0]["bounds"] == {
        "lower": 0, "upper": {"finite": 4}}
    assert j_mix["row"]["closed"][0]["guard"] == {"unconditional": None}
    algebra("effect/algebra/join-unconditional-vs-conditional",
            'unconditional [1,2] ⊔ guard(input=="a") [2,4] = unconditional '
            "[0,4]: the guarded branch may contribute zero",
            "branch_join", [jg, jh],
            {"canonical_ace1_utf8": result_ace1(j_mix["row"])})

    # Case C: a one-sided guarded key keeps its guard with lower 0.
    ji = sig_closed([entry("github.issue.create", 1, [], 2, 4,
                           guard=gstring("a"))])
    j_one = branch_join(ji, pure())
    assert j_one["row"]["closed"][0]["bounds"] == {
        "lower": 0, "upper": {"finite": 4}}
    assert j_one["row"]["closed"][0]["guard"] == gstring("a")
    algebra("effect/algebra/join-one-sided-guarded-effect",
            "a key present in only one branch drops to [0,u] and KEEPS its "
            "provable guard: the other branch's absence invents no "
            "occurrences",
            "branch_join", [ji, pure()],
            {"canonical_ace1_utf8": result_ace1(j_one["row"])})

    # Properties over the guarded closed set.
    jk = sig_closed([entry("github.issue.create", 1, [], 1, 2,
                           guard=gstring("a"))])
    jl = sig_closed([entry("github.issue.create", 1, [], 1, 3,
                           guard=gstring("b"))])
    j_comm = select_envelope(jk, jl)
    j_comm_back = select_envelope(jl, jk)
    j_comm_bytes = result_ace1(j_comm["row"])
    assert j_comm_bytes == result_ace1(j_comm_back["row"])
    algebra("effect/algebra/join-commutativity-with-guards",
            "guarded branch join is commutative; differing input_equals "
            "conditions widen to unconditional [0,3]",
            "select_envelope", [jk, jl],
            {"canonical_ace1_utf8": j_comm_bytes},
            property_kind="commutativity")

    jm = sig_closed([entry("github.issue.create", 1, [], 2, 4,
                           guard=gstring("a"))])
    j_idem = branch_join(jm, jm)
    j_idem_bytes = result_ace1(j_idem["row"])
    assert j_idem_bytes == result_ace1(validate(jm)["row"]), (
        "guarded join idempotence broke")
    algebra("effect/algebra/join-idempotence-with-guards",
            "guarded branch join with itself reproduces the branch envelope "
            "exactly (conditional lower bound kept)",
            "branch_join", [jm, jm],
            {"canonical_ace1_utf8": j_idem_bytes},
            property_kind="idempotence")

    jn = sig_closed([entry("github.issue.create", 1, [], 1, 2,
                           guard=gstring("a"))])
    jo = sig_closed([entry("github.issue.create", 1, [], 1, 3,
                           guard=gstring("b"))])
    jp = sig_closed([entry("github.issue.create", 1, [], 2, 5,
                           guard=gstring("a"))])
    j_assoc_left = branch_join(branch_join(jn, jo), jp)
    j_assoc_right = branch_join(jn, branch_join(jo, jp))
    j_assoc_bytes = result_ace1(j_assoc_left["row"])
    assert j_assoc_bytes == result_ace1(j_assoc_right["row"]), (
        "guarded join associativity broke")
    assert j_assoc_left["row"]["closed"][0]["bounds"] == {
        "lower": 0, "upper": {"finite": 5}}
    assert j_assoc_left["row"]["closed"][0]["guard"] == {"unconditional": None}
    algebra("effect/algebra/join-associativity-with-guards",
            "guarded branch join is associative over the closed guard set: "
            "both groupings of guard(a) ⊔ guard(b) ⊔ guard(a) give "
            "unconditional [0,5]",
            "branch_join", [jn, jo, jp],
            {"canonical_ace1_utf8": j_assoc_bytes},
            property_kind="associativity")

    # --- Equivalence -------------------------------------------------------

    reordered_members = [
        sig_closed([
            write_entry("github.issue.create", 1, [("repo", "foo/bar")], 1, 2),
            write_entry("database.row.read", 2, [("table", "customer")], 0, 3),
        ]),
        sig_closed([
            write_entry("database.row.read", 2, [("table", "customer")], 0, 3),
            write_entry("github.issue.create", 1, [("repo", "foo/bar")], 1, 2),
        ]),
    ]
    equivalence("effect/equivalence/entry-reordering",
                "entry order never changes the canonical bytes",
                reordered_members,
                result_ace1(validate(reordered_members[0])["row"]))

    scope_members = [
        sig_closed([write_entry("memory.append", 1,
                                [("scope", "tenant-42"), ("store", "notes")],
                                1, 8)]),
        sig_closed([write_entry("memory.append", 1,
                                [("store", "notes"), ("scope", "tenant-42")],
                                1, 8)]),
    ]
    equivalence("effect/equivalence/scope-binding-reordering",
                "scope binding order never changes the canonical bytes",
                scope_members,
                result_ace1(validate(scope_members[0])["row"]))

    zero_members = [
        sig_closed([write_entry("github.issue.create", 1,
                                [("repo", "foo/bar")], 0, 0)]),
        sig_closed([]),
    ]
    equivalence("effect/equivalence/zero-occurrence-equals-pure",
                "an entry that cannot occur is semantically pure: the "
                "signature with only zero-occurrence entries has the "
                "canonical bytes of the empty closed row",
                zero_members,
                result_ace1(validate(pure())["row"]))

    open_members = [
        sig_open([], [residual(CATALOG, 1, [("repo", "foo/bar")], 0, 5),
                      residual(CATALOG, 2, [("table", "customer")], 0, 2)]),
        sig_open([], [residual(CATALOG, 2, [("table", "customer")], 0, 2),
                      residual(CATALOG, 1, [("repo", "foo/bar")], 0, 5)]),
    ]
    equivalence("effect/equivalence/residual-reordering",
                "residual order never changes the canonical bytes",
                open_members,
                result_ace1(validate(open_members[0])["row"]))

    print(f"\neffect-signature corpus: {len(CASES)} cases frozen "
          f"(revision {CORPUS_REVISION})")


if __name__ == "__main__":
    main()
