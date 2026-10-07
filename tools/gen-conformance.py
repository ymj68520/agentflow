#!/usr/bin/env python3
"""Freezes the AgentFlow Phase-1 conformance corpus (golden + negative fixtures).

This script is an INDEPENDENT implementation of ACE1 canonical encoding and
HashV1 (Python: unicodedata / base64 / struct / hashlib). It exists so the
expected values stored under spec/conformance/ are generated once, cross-validated
against a second implementation, and then physically frozen in the repository.
The Rust conformance tests only CONSUME these files; they never generate
expected values at test time.

Run from the repository root:  python tools/gen-conformance.py
"""

import base64
import hashlib
import json
import struct
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CONF = ROOT / "spec" / "conformance"

# Corpus revision: 1 = Phase 1 initial freeze (null-sugar Option encoding);
# 2 = Phase 1A: Option re-encoded as explicit tagged objects. This records
# that the Option wire encoding changed BEFORE the v1 artifact freeze; it is
# a non-semantic provenance note, not a spec release version.
CORPUS_REVISION = 2

DOMAINS = [
    "type-schema", "value", "provenance", "artifact", "semantic-ir",
    "plan-core", "run-manifest", "lock-graph", "journal-event-payload",
    "runtime-command-payload", "semantic-execution-state",
    "component-contract", "resource-contract", "conformance",
]

# ---------------------------------------------------------------------------
# Independent ACE1 implementation
# ---------------------------------------------------------------------------

def nfc(s: str) -> str:
    return unicodedata.normalize("NFC", s)


def escape_string(s: str) -> str:
    out = []
    for ch in s:
        o = ord(ch)
        if ch == '"':
            out.append('\\"')
        elif ch == "\\":
            out.append("\\\\")
        elif ch == "\b":
            out.append("\\b")
        elif ch == "\t":
            out.append("\\t")
        elif ch == "\n":
            out.append("\\n")
        elif ch == "\f":
            out.append("\\f")
        elif ch == "\r":
            out.append("\\r")
        elif o < 0x20:
            out.append("\\u%04x" % o)
        else:
            out.append(ch)
    return '"' + "".join(out) + '"'


def canonical_decimal(text: str):
    """Canonicalizes a decimal literal per the frozen rules (independent impl)."""
    s = text
    neg = s.startswith("-")
    if neg:
        s = s[1:]
    if not s or s.count(".") > 1:
        return None
    int_part, _, frac_part = s.partition(".")
    if not int_part or not all(c in "0123456789" for c in int_part):
        return None
    if len(int_part) > 1 and int_part.startswith("0"):
        return None  # leading zeros rejected, not normalized
    if s.endswith(".") or ("." in s and not all(c in "0123456789" for c in frac_part)):
        return None
    is_zero = all(c == "0" for c in int_part) and all(c == "0" for c in frac_part)
    if is_zero:
        return "0"  # negative zero and zero-normalized fractions collapse to 0
    frac_part = frac_part.rstrip("0")
    sign = "-" if neg else ""
    if frac_part:
        return f"{sign}{int_part}.{frac_part}"
    return f"{sign}{int_part}"


def b64url(data: bytes) -> str:
    return base64.urlsafe_b64encode(data).decode("ascii").rstrip("=")


def float_hex_from_bits(bits_hex: str) -> str:
    bits = int(bits_hex, 16)
    if bits == 0x8000000000000000:  # -0 canonicalizes to +0
        bits = 0
    return "%016x" % bits


def key_member(key: str, value_bytes: str) -> str:
    return escape_string(nfc(key)) + ":" + value_bytes


def encode(shape, value) -> str:
    """Encodes a fixture-grammar (shape, value) pair into canonical ACE1 text."""
    if isinstance(shape, dict):
        kind = next(iter(shape))
        spec = shape[kind]
    else:
        kind, spec = shape, None

    if kind == "unit":
        assert value == {"unit": None}
        return "null"
    if kind == "null":
        assert value == {"null": None}
        return "null"
    if kind == "bool":
        return "true" if value["bool"] else "false"
    if kind in ("int32", "uint32", "int64", "uint64", "timestamp", "duration"):
        return escape_string(str(int(value[kind])))
    if kind == "decimal":
        canonical = canonical_decimal(value["decimal"])
        assert canonical is not None, f"invalid fixture decimal {value['decimal']!r}"
        return escape_string(canonical)
    if kind == "float64":
        return escape_string(float_hex_from_bits(value["float64"]))
    if kind == "string":
        return escape_string(nfc(value["string"]))
    if kind == "bytes":
        return escape_string(b64url(bytes(value["bytes"])))
    if kind == "enum":
        assert value["enum"] in spec["values"], "fixture enum value not closed"
        return escape_string(value["enum"])
    if kind == "list":
        return "[" + ",".join(encode(spec, item) for item in value["list"]) + "]"
    if kind == "option":
        # Phase 1A: explicit single-case tagged objects.
        if value == {"none": None}:
            return '{"none":null}'
        assert set(value) == {"some"}, f"option value must be none/some, got {value}"
        return '{"some":' + encode(spec, value["some"]) + '}'
    if kind == "map":
        entries = {}
        for entry in value["map"]:
            key = nfc(entry["key"])
            if key in entries:
                raise ValueError(f"duplicate normalized map key {key!r}")
            entries[key] = encode(spec, entry["value"])
        ordered = sorted(entries, key=lambda k: k.encode("utf-8"))
        return "{" + ",".join(key_member(k, entries[k]) for k in ordered) + "}"
    if kind == "record":
        supplied = {}
        for field in value["record"]:
            name = field["field"]
            if name in supplied:
                raise ValueError(f"duplicate record field {name!r}")
            supplied[name] = field["value"]
        names = [f["field"] for f in spec["fields"]]
        missing = [n for n in names if n not in supplied]
        unknown = [n for n in supplied if n not in names]
        assert not missing, f"missing record fields {missing}"
        assert not unknown, f"unknown record fields {unknown}"
        shapes = {f["field"]: f["type"] for f in spec["fields"]}
        ordered = sorted(names, key=lambda n: nfc(n).encode("utf-8"))
        return "{" + ",".join(
            key_member(n, encode(shapes[n], supplied[n])) for n in ordered
        ) + "}"
    if kind == "variant":
        case = value["variant"]["case"]
        case_shapes = {c["case"]: c["type"] for c in spec["cases"]}
        assert case in case_shapes, f"unknown variant case {case!r}"
        payload = encode(case_shapes[case], value["variant"]["value"])
        return "{" + key_member(case, payload) + "}"
    raise AssertionError(f"unknown shape kind {kind!r}")


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def hash_v1(domain: str, payload: bytes) -> str:
    return sha256_hex(b"AF\x00" + domain.encode("ascii") + b"\x00" + payload)


def case(case_id, description, shape, value, equivalent_to=None):
    text = encode(shape, value)
    raw = text.encode("utf-8")
    out = {
        "case_id": case_id,
        "description": description,
        "shape": shape,
        "value": value,
        "expected_ace1_utf8": text,
        "expected_bytes_hex": raw.hex(),
    }
    if equivalent_to:
        out["equivalent_to"] = equivalent_to
    return out


def neg(case_id, description, shape, value, error_class, target="encoder"):
    out = {
        "case_id": case_id,
        "description": description,
        "shape": shape,
        "value": value,
        "error_class": error_class,
        "target": target,
        "requires_decoder": target == "decoder",
    }
    return out


def write_json(path: Path, payload):
    payload = {"corpus_revision": CORPUS_REVISION, **payload}
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    count = len(payload.get("cases", payload.get("domains", [])))
    print(f"wrote {path.relative_to(ROOT)} ({count} entries)")


# ---------------------------------------------------------------------------
# Encoding golden: primitives
# ---------------------------------------------------------------------------

PRIMITIVES = []

def add(case_id, description, shape, value, equivalent_to=None):
    PRIMITIVES.append(case(case_id, description, shape, value, equivalent_to))

# int32
for cid, v, d in [
    ("int32/zero", 0, "int32 zero"),
    ("int32/one", 1, "int32 one"),
    ("int32/negative-one", -1, "int32 negative one"),
    ("int32/min", -2**31, "int32 minimum boundary"),
    ("int32/max", 2**31 - 1, "int32 maximum boundary"),
    ("int32/sample", 123456789, "int32 multi-digit sample"),
]:
    add(cid, d, "int32", {"int32": str(v)})

# uint32
for cid, v in [("uint32/zero", 0), ("uint32/one", 1), ("uint32/max", 2**32 - 1),
               ("uint32/large", 4294967294)]:
    add(cid, f"{cid.split('/')[-1]} uint32 value", "uint32", {"uint32": str(v)})

# int64 (2**53+1 must never pass through a float)
for cid, v in [("int64/zero", 0), ("int64/one", 1), ("int64/negative-one", -1),
               ("int64/min", -2**63), ("int64/max", 2**63 - 1),
               ("int64/beyond-float53", 2**53 + 1),
               ("int64/negative-beyond-float53", -(2**53 + 1))]:
    add(cid, f"{cid.split('/')[-1]} int64 value (exact integer, no float path)", "int64",
        {"int64": str(v)})

# uint64
for cid, v in [("uint64/zero", 0), ("uint64/one", 1), ("uint64/max", 2**64 - 1),
               ("uint64/high-bit", 2**63), ("uint64/near-max", 2**64 - 2)]:
    add(cid, f"{cid.split('/')[-1]} uint64 value", "uint64", {"uint64": str(v)})

# decimal (canonical forms)
for cid, v in [
    ("decimal/zero", "0"),
    ("decimal/one", "1"),
    ("decimal/negative-one", "-1"),
    ("decimal/fraction", "0.5"),
    ("decimal/negative-fraction", "-0.5"),
    ("decimal/two-point-twenty-five", "1.25"),
    ("decimal/small", "0.001"),
    ("decimal/tiny", "0.000000000000000000000000000001"),
    ("decimal/large-integer", "123456789012345678901234567890"),
    ("decimal/large-fraction", "123456789012345678901234567890.123456789"),
    ("decimal/repeating-look", "3.141592653589793238462643383279"),
]:
    add(cid, f"decimal canonical form {v}", "decimal", {"decimal": v})

# Equivalent (non-canonical) textual inputs must freeze to identical bytes.
add("decimal/equivalent-input-1.250", "decimal input 1.250 canonicalizes to the same bytes as its canonical form 1.25", "decimal", {"decimal": "1.250"}, equivalent_to="decimal/two-point-twenty-five")
add("decimal/equivalent-input-1.000", "decimal input 1.000 canonicalizes to an integral form", "decimal", {"decimal": "1.000"}, equivalent_to="decimal/one")
add("decimal/equivalent-input-negative-zero", "decimal input -0 canonicalizes to zero", "decimal", {"decimal": "-0"}, equivalent_to="decimal/zero")
add("decimal/equivalent-input-0.000", "decimal input 0.000 canonicalizes to zero", "decimal", {"decimal": "0.000"}, equivalent_to="decimal/zero")
add("decimal/equivalent-input-0.0010", "decimal input 0.0010 canonicalizes to 0.001", "decimal", {"decimal": "0.0010"}, equivalent_to="decimal/small")

# float64: input is the exact IEEE-754 bit pattern
for cid, bits, d in [
    ("float64/positive-zero", "0000000000000000", "float64 +0.0"),
    ("float64/negative-zero-canonicalized", "8000000000000000",
     "float64 -0.0 bit pattern must canonicalize to +0"),
    ("float64/one", "3ff0000000000000", "float64 1.0"),
    ("float64/negative-one", "bff0000000000000", "float64 -1.0"),
    ("float64/half", "3fe0000000000000", "float64 0.5"),
    ("float64/two", "4000000000000000", "float64 2.0"),
    ("float64/one-tenth", "3fb999999999999a", "float64 0.1 (exact stored bits)"),
    ("float64/min-subnormal", "0000000000000001", "smallest subnormal 2**-1074"),
    ("float64/max-subnormal", "000fffffffffffff", "largest subnormal"),
    ("float64/min-normal", "0010000000000000", "smallest normal 2**-1022"),
    ("float64/max-finite", "7fefffffffffffff", "largest finite float64"),
    ("float64/large-finite", "7fe8000000000000", "large finite float64"),
]:
    add(cid, d, "float64", {"float64": bits})

# Option v1 (Phase 1A): explicit tagged encoding; nested options and
# option<unit>/option<null> are legal. Cases with identical tokens across
# different shapes are intentional: the schema nominally disambiguates.
add("option/string/none", "option<string> None → tagged {\"none\":null}",
    {"option": "string"}, {"none": None})
add("option/string/some-x", "option<string> Some(\"x\")",
    {"option": "string"}, {"some": {"string": "x"}})
add("option/null/none", "option<null> None",
    {"option": "null"}, {"none": None})
add("option/null/some-null", "option<null> Some(null) — bytes differ from None",
    {"option": "null"}, {"some": {"null": None}})
add("option/unit/none", "option<unit> None",
    {"option": "unit"}, {"none": None})
add("option/unit/some-unit", "option<unit> Some(unit) — same token as Some(null), nominally distinct inner type",
    {"option": "unit"}, {"some": {"unit": None}})
add("option/nested-string/none", "option<option<string>> None",
    {"option": {"option": "string"}}, {"none": None})
add("option/nested-string/some-none", "option<option<string>> Some(None)",
    {"option": {"option": "string"}}, {"some": {"none": None}})
add("option/nested-string/some-some-x", "option<option<string>> Some(Some(\"x\"))",
    {"option": {"option": "string"}}, {"some": {"some": {"string": "x"}}})
add("option/nested-null/some-some-null", "option<option<null>> Some(Some(null))",
    {"option": {"option": "null"}}, {"some": {"some": {"null": None}}})

# Cross-check (§六): None must be byte-distinguishable from every Some form.
_none_b = encode({"option": "string"}, {"none": None})
_some_null_b = encode({"option": "null"}, {"some": {"null": None}})
_some_unit_b = encode({"option": "unit"}, {"some": {"unit": None}})
_some_none_b = encode({"option": {"option": "string"}}, {"some": {"none": None}})
assert _none_b != _some_null_b, "None must not share bytes with Some(null)"
assert _none_b != _some_unit_b, "None must not share bytes with Some(unit)"
assert _none_b != _some_none_b, "None must not share bytes with Some(None)"
assert _some_none_b == '{"some":{"none":null}}'

# bool / null / unit
add("bool/true", "bool true literal", "bool", {"bool": True})
add("bool/false", "bool false literal", "bool", {"bool": False})
add("null/explicit", "explicit null primitive", "null", {"null": None})
add("unit/value", "unit primitive (distinct nominal type from null)", "unit", {"unit": None})

# timestamp / duration (int64 microseconds, nominal types)
for cid, v in [("timestamp/zero", 0), ("timestamp/one-micro", 1),
               ("timestamp/negative-one", -1),
               ("timestamp/2023-11-14", 1700000000000000),
               ("timestamp/min", -2**63), ("timestamp/max", 2**63 - 1)]:
    add(cid, f"timestamp microseconds {v}", "timestamp", {"timestamp": str(v)})
for cid, v in [("duration/zero", 0), ("duration/one-micro", 1),
               ("duration/negative-one", -1),
               ("duration/one-day", 86_400_000_000),
               ("duration/negative-day", -86_400_000_000),
               ("duration/min", -2**63), ("duration/max", 2**63 - 1)]:
    add(cid, f"duration microseconds {v}", "duration", {"duration": str(v)})

# bytes
for cid, data in [
    ("bytes/empty", []),
    ("bytes/f", [0x66]),
    ("bytes/fo", [0x66, 0x6F]),
    ("bytes/foo", [0x66, 0x6F, 0x6F]),
    ("bytes/url-safe-alphabet", [0xFB, 0xFF, 0xBF]),
    ("bytes/all-byte-values-low", list(range(0, 64))),
    ("bytes/all-byte-values-high", list(range(192, 256))),
]:
    add(cid, f"bytes {len(data)} byte(s), unpadded base64url", "bytes", {"bytes": data})

write_json(CONF / "encoding" / "primitive-golden.json", {
    "kind": "agentflow.conformance.ace1-golden",
    "ace1_version": "1",
    "note": (
        "Golden fixtures for ACE1 machine primitives. Value grammar: scalars are "
        "tagged objects ({\"int32\": \"-2147483648\"}); float64 input is the exact "
        "IEEE-754 bit pattern as 16 hex chars (negative-zero pattern included to "
        "prove canonicalization); bytes input is an array of byte values. "
        "expected_ace1_utf8 / expected_bytes_hex are frozen; tests must not "
        "regenerate them."
    ),
    "cases": PRIMITIVES,
})

# ---------------------------------------------------------------------------
# Encoding golden: strings
# ---------------------------------------------------------------------------

STRINGS = []

def add_str(cid, description, raw, shape="string"):
    STRINGS.append(case(cid, description, shape, {"string": raw}))

add_str("string/empty", "empty string", "")
add_str("string/ascii", "plain ASCII text", "hello world")
add_str("string/nfc-precomposed", "precomposed e-acute U+00E9", "é")
add_str("string/nfc-decomposed-input", "decomposed e + U+0301 must canonicalize to U+00E9",
        "e\u0301")
add_str("string/nfc-hangul-precomposed", "precomposed Hangul syllable U+AC01", "각")
add_str("string/nfc-hangul-decomposed-input",
        "decomposed Hangul jamo must canonicalize to U+AC01", "\u1100\u1161\u11A8")
add_str("string/quotes", "embedded double quotes need escaping", 'He said "hi"')
add_str("string/backslash", "embedded backslash needs escaping", "C:\\path\\to")
add_str("string/short-escapes", "tab and newline use short escapes", "a\tb\nc")
add_str("string/short-escapes-2", "form feed and carriage return short escapes", "a\fb\rc")
add_str("string/control-nul", "NUL control character escaped as \\u0000", "a\u0000b")
add_str("string/control-low", "control U+0001 escaped as \\u0001 (lowercase hex)", "a\u0001b")
add_str("string/control-high", "control U+001F escaped as \\u001f", "a\u001fb")
add_str("string/del-raw", "U+007F DEL is not escaped", "a\u007fb")
add_str("string/cjk", "CJK text stays raw UTF-8", "中文测试")
add_str("string/emoji", "emoji (4-byte UTF-8) stays raw", "🚀🎉")
add_str("string/greek", "Greek text", "Ωμέγα")
add_str("string/mixed", "mixed scripts, emoji, spaces", "café ☕ 中文 🚀 end")
add_str("string/nfc-multiple-combining", "multiple combining marks canonicalize",
        "a\u0301\u0323")

# Cross-check the equivalence pairs really are equal after NFC.
pre = encode("string", {"string": "é"})
dec = encode("string", {"string": "e\u0301"})
assert pre == dec, "NFC equivalence broken for e-acute"
pre = encode("string", {"string": "각"})
dec = encode("string", {"string": "\u1100\u1161\u11A8"})
assert pre == dec, "NFC equivalence broken for Hangul"

write_json(CONF / "encoding" / "string-golden.json", {
    "kind": "agentflow.conformance.ace1-golden",
    "ace1_version": "1",
    "note": (
        "Golden fixtures for ACE1 strings: NFC normalization, deterministic "
        "escaping, non-ASCII raw UTF-8. Decomposed-input cases must produce "
        "byte-identical output to their precomposed counterparts."
    ),
    "cases": STRINGS,
})

# ---------------------------------------------------------------------------
# Encoding golden: collections (record / variant / enum / list / map / option)
# ---------------------------------------------------------------------------

COLLECTIONS = []

def add_col(cid, description, shape, value):
    COLLECTIONS.append(case(cid, description, shape, value))

POINT = {"record": {"name": "Point", "fields": [
    {"field": "x", "type": "int64"},
    {"field": "y", "type": "int64"},
]}}

TASK_OUTCOME = {"enum": {"name": "TaskOutcome", "values": ["SUCCEEDED", "FAILED", "CANCELLED", "TIMED_OUT"]}}

RESULT = {"variant": {"name": "Result", "cases": [
    {"case": "ok", "type": "string"},
    {"case": "error", "type": "string"},
    {"case": "empty", "type": "unit"},
]}}

# lists
add_col("list/empty", "empty list", {"list": "int32"}, {"list": []})
add_col("list/int32-order-preserved", "list keeps order (not sorted)", {"list": "int32"},
        {"list": [{"int32": "3"}, {"int32": "1"}, {"int32": "2"}]})
add_col("list/int32-boundaries", "int32 boundaries in a list", {"list": "int32"},
        {"list": [{"int32": "-2147483648"}, {"int32": "2147483647"}]})
add_col("list/string", "list of strings", {"list": "string"},
        {"list": [{"string": "b"}, {"string": "a"}, {"string": "c"}]})
add_col("list/nested", "nested lists", {"list": {"list": "uint32"}},
        {"list": [{"list": [{"uint32": "1"}, {"uint32": "2"}]}, {"list": []}]})
add_col("list/enum", "list of closed enum values", {"list": TASK_OUTCOME},
        {"list": [{"enum": "SUCCEEDED"}, {"enum": "TIMED_OUT"}]})

# maps: unsigned UTF-8 byte ordering
MAP_I32 = {"map": "int32"}
add_col("map/empty", "empty map", MAP_I32, {"map": []})
add_col("map/single", "single-entry map", MAP_I32,
        {"map": [{"key": "a", "value": {"int32": "1"}}]})
add_col("map/unsorted-input", "unsorted input keys are canonicalized into byte order", MAP_I32,
        {"map": [{"key": "zeta", "value": {"int32": "3"}},
                 {"key": "alpha", "value": {"int32": "1"}},
                 {"key": "mid", "value": {"int32": "2"}}]})
add_col("map/sorted-input-equivalent", "the same map in sorted input order — bytes identical "
        "to map/unsorted-input", MAP_I32,
        {"map": [{"key": "alpha", "value": {"int32": "1"}},
                 {"key": "mid", "value": {"int32": "2"}},
                 {"key": "zeta", "value": {"int32": "3"}}]})
add_col("map/byte-order-case", "uppercase B (0x42) sorts before lowercase a (0x61)", MAP_I32,
        {"map": [{"key": "a", "value": {"int32": "2"}},
                 {"key": "B", "value": {"int32": "1"}}]})
add_col("map/byte-order-multibyte", "multibyte keys sort by unsigned UTF-8 bytes: "
        "z(0x7A) < é(U+00E9) < 中(U+4E2D)", MAP_I32,
        {"map": [{"key": "中", "value": {"int32": "3"}},
                 {"key": "z", "value": {"int32": "1"}},
                 {"key": "é", "value": {"int32": "2"}}]})
add_col("map/empty-key", "empty string key sorts first", MAP_I32,
        {"map": [{"key": "a", "value": {"int32": "2"}},
                 {"key": "", "value": {"int32": "1"}}]})
add_col("map/key-needs-escaping", "map key containing a quote", MAP_I32,
        {"map": [{"key": "a\"b", "value": {"int32": "1"}},
                 {"key": "ab", "value": {"int32": "2"}}]})
add_col("map/nfc-key-input", "decomposed key input is NFC-normalized", MAP_I32,
        {"map": [{"key": "e\u0301cole", "value": {"int32": "1"}}]})
add_col("map/value-shapes", "map<string,string> values", {"map": "string"},
        {"map": [{"key": "b", "value": {"string": "second"}},
                 {"key": "a", "value": {"string": "first"}}]})
add_col("map/variant-values", "map of variants", {"map": RESULT},
        {"map": [{"key": "a", "value": {"variant": {"case": "ok", "value": {"string": "fine"}}}},
                 {"key": "b", "value": {"variant": {"case": "empty", "value": {"unit": None}}}}]})

# records
add_col("record/declared-order-not-canonical",
        "fields declared y,x are emitted in canonical byte order x,y", POINT,
        {"record": [{"field": "y", "value": {"int64": "2"}},
                    {"field": "x", "value": {"int64": "1"}}]})
add_col("record/empty", "record with no declared fields",
        {"record": {"name": "Empty", "fields": []}}, {"record": []})
add_col("record/with-option-none", "record with Option field None (explicit null)",
        {"record": {"name": "Maybe", "fields": [
            {"field": "value", "type": {"option": "string"}}]}},
        {"record": [{"field": "value", "value": {"none": None}}]})
add_col("record/with-option-some", "record with Option field Some",
        {"record": {"name": "Maybe", "fields": [
            {"field": "value", "type": {"option": "string"}}]}},
        {"record": [{"field": "value", "value": {"some": {"string": "present"}}}]})
add_col("record/with-null-field", "record with a null-typed field",
        {"record": {"name": "Nullable", "fields": [
            {"field": "value", "type": "null"}]}},
        {"record": [{"field": "value", "value": {"null": None}}]})
add_col("record/with-unit-field", "record with a unit-typed field",
        {"record": {"name": "Unitary", "fields": [
            {"field": "value", "type": "unit"}]}},
        {"record": [{"field": "value", "value": {"unit": None}}]})
add_col("record/nested", "nested records", {"record": {"name": "Outer", "fields": [
    {"field": "inner", "type": POINT}, {"field": "label", "type": "string"}]}},
        {"record": [{"field": "inner", "value": {"record": [
            {"field": "x", "value": {"int64": "10"}}, {"field": "y", "value": {"int64": "20"}}]}},
            {"field": "label", "value": {"string": "origin"}}]})
add_col("record/all-primitives", "record touching every machine primitive",
        {"record": {"name": "Everything", "fields": [
            {"field": "a_bool", "type": "bool"},
            {"field": "a_bytes", "type": "bytes"},
            {"field": "a_decimal", "type": "decimal"},
            {"field": "a_duration", "type": "duration"},
            {"field": "a_f64", "type": "float64"},
            {"field": "a_i32", "type": "int32"},
            {"field": "a_i64", "type": "int64"},
            {"field": "a_string", "type": "string"},
            {"field": "a_ts", "type": "timestamp"},
            {"field": "a_u32", "type": "uint32"},
            {"field": "a_u64", "type": "uint64"},
        ]}},
        {"record": [
            {"field": "a_bool", "value": {"bool": True}},
            {"field": "a_bytes", "value": {"bytes": [1, 2, 3]}},
            {"field": "a_decimal", "value": {"decimal": "1.5"}},
            {"field": "a_duration", "value": {"duration": "-42"}},
            {"field": "a_f64", "value": {"float64": "3ff0000000000000"}},
            {"field": "a_i32", "value": {"int32": "-5"}},
            {"field": "a_i64", "value": {"int64": "9007199254740993"}},
            {"field": "a_string", "value": {"string": "héllo"}},
            {"field": "a_ts", "value": {"timestamp": "1700000000000000"}},
            {"field": "a_u32", "value": {"uint32": "4294967295"}},
            {"field": "a_u64", "value": {"uint64": "18446744073709551615"}},
        ]})

# variants
add_col("variant/string-payload", "variant with string payload", RESULT,
        {"variant": {"case": "ok", "value": {"string": "fine"}}})
add_col("variant/unit-payload", "variant with unit payload", RESULT,
        {"variant": {"case": "empty", "value": {"unit": None}}})
add_col("variant/nested-variant", "variant whose payload is another variant",
        {"variant": {"name": "Outer", "cases": [
            {"case": "wrap", "type": RESULT}, {"case": "nothing", "type": "unit"}]}},
        {"variant": {"case": "wrap", "value": {"variant": {"case": "error", "value": {"string": "boom"}}}}})
add_col("variant/record-payload", "variant with record payload",
        {"variant": {"name": "Shape", "cases": [{"case": "point", "type": POINT}]}},
        {"variant": {"case": "point", "value": {"record": [
            {"field": "x", "value": {"int64": "1"}}, {"field": "y", "value": {"int64": "2"}}]}}})

# options
add_col("option/some-string", "standalone Some", {"option": "string"},
        {"some": {"string": "x"}})
add_col("option/none-string", "standalone None (explicit null)", {"option": "string"},
        {"none": None})
add_col("option/int32-in-list", "options inside a list", {"list": {"option": "int32"}},
        {"list": [{"some": {"int32": "1"}}, {"none": None}, {"some": {"int32": "3"}}]})
add_col("option/record-some", "Some(record)", {"option": POINT},
        {"some": {"record": [{"field": "x", "value": {"int64": "1"}},
                             {"field": "y", "value": {"int64": "2"}}]}})

# cross-check the two map orderings really produce identical bytes
a = encode(MAP_I32, {"map": [{"key": "zeta", "value": {"int32": "3"}},
                             {"key": "alpha", "value": {"int32": "1"}},
                             {"key": "mid", "value": {"int32": "2"}}]})
b = encode(MAP_I32, {"map": [{"key": "alpha", "value": {"int32": "1"}},
                             {"key": "mid", "value": {"int32": "2"}},
                             {"key": "zeta", "value": {"int32": "3"}}]})
assert a == b, "map canonical order broken"

write_json(CONF / "encoding" / "collection-golden.json", {
    "kind": "agentflow.conformance.ace1-golden",
    "ace1_version": "1",
    "note": (
        "Golden fixtures for ACE1 containers. Shape grammar: primitive names as "
        "strings; {\"enum\": {name, values}}, {\"list\": T}, {\"option\": T}, "
        "{\"map\": T}, {\"record\": {name, fields:[{field,type}]}}, "
        "{\"variant\": {name, cases:[{case,type}]}}. Record/map member order in "
        "canonical bytes is unsigned UTF-8 byte order of NFC names, independent "
        "of declaration or input order."
    ),
    "cases": COLLECTIONS,
})

# ---------------------------------------------------------------------------
# Encoding negative corpus
# ---------------------------------------------------------------------------

NEGATIVES = []

def add_neg(*args, **kwargs):
    NEGATIVES.append(neg(*args, **kwargs))

# integers
add_neg("int32/overflow", "int32 above maximum", "int32", {"int32": "2147483648"},
        "integer_out_of_range")
add_neg("int32/underflow", "int32 below minimum", "int32", {"int32": "-2147483649"},
        "integer_out_of_range")
add_neg("int64/overflow", "int64 above maximum", "int64", {"int64": "9223372036854775808"},
        "integer_out_of_range")
add_neg("int64/underflow", "int64 below minimum", "int64", {"int64": "-9223372036854775809"},
        "integer_out_of_range")
add_neg("uint32/negative", "uint32 rejects negative values", "uint32", {"uint32": "-1"},
        "integer_out_of_range")
add_neg("uint64/negative", "uint64 rejects negative values", "uint64", {"uint64": "-1"},
        "integer_out_of_range")
add_neg("uint64/overflow", "uint64 above maximum", "uint64", {"uint64": "18446744073709551616"},
        "integer_out_of_range")
add_neg("int/plus-sign", "leading + is not canonical integer text", "int32",
        {"int32": "+1"}, "invalid_integer")
add_neg("int/leading-zeros", "leading zeros are not canonical integer text", "int32",
        {"int32": "007"}, "invalid_integer")
add_neg("int/negative-zero", "-0 is not canonical integer text", "int64",
        {"int64": "-0"}, "invalid_integer")
add_neg("int/empty", "empty integer text", "int32", {"int32": ""}, "invalid_integer")
add_neg("int/junk", "non-digit characters", "int32", {"int32": "12a4"}, "invalid_integer")
add_neg("int/float-text", "float text is not integer text", "int32", {"int32": "1.5"},
        "invalid_integer")

# nominal type distinctions
add_neg("nominal/int64-into-int32", "int64 value into an int32 shape", "int32",
        {"int64": "5"}, "type_mismatch")
add_neg("nominal/timestamp-into-int64", "timestamp into an int64 shape", "int64",
        {"timestamp": "0"}, "type_mismatch")
add_neg("nominal/duration-into-timestamp", "duration into a timestamp shape", "timestamp",
        {"duration": "0"}, "type_mismatch")
add_neg("nominal/int32-into-timestamp", "int32 into a timestamp shape", "timestamp",
        {"int32": "0"}, "type_mismatch")

# decimal
add_neg("decimal/plus-sign", "decimal with + sign", "decimal", {"decimal": "+1.5"},
        "invalid_decimal")
add_neg("decimal/leading-zero", "decimal with leading zero", "decimal", {"decimal": "01.5"},
        "invalid_decimal")
add_neg("decimal/trailing-dot", "decimal without fractional digits", "decimal",
        {"decimal": "1."}, "invalid_decimal")
add_neg("decimal/leading-dot", "decimal without integer digits", "decimal", {"decimal": ".5"},
        "invalid_decimal")
add_neg("decimal/exponent", "exponent notation is not part of ACE1 decimals", "decimal",
        {"decimal": "1.5e2"}, "invalid_decimal")
add_neg("decimal/double-dot", "malformed decimal", "decimal", {"decimal": "1.2.3"},
        "invalid_decimal")
add_neg("decimal/empty", "empty decimal", "decimal", {"decimal": ""}, "invalid_decimal")
add_neg("decimal/text", "non-numeric decimal", "decimal", {"decimal": "abc"},
        "invalid_decimal")

# float64
add_neg("float64/nan", "NaN is forbidden", "float64",
        {"float64": "7ff8000000000000"}, "non_finite_float")
add_neg("float64/nan-signaling", "signaling NaN is forbidden", "float64",
        {"float64": "7ff0000000000001"}, "non_finite_float")
add_neg("float64/positive-infinity", "+Inf is forbidden", "float64",
        {"float64": "7ff0000000000000"}, "non_finite_float")
add_neg("float64/negative-infinity", "-Inf is forbidden", "float64",
        {"float64": "fff0000000000000"}, "non_finite_float")

# strings / bytes
add_neg("bytes/invalid-base64url", "invalid base64url character", "bytes",
        {"bytes_decoded_from_text": None}, "invalid_bytes_encoding", target="placeholder")

# structural
add_neg("record/missing-field", "declared field not supplied", POINT,
        {"record": [{"field": "x", "value": {"int64": "1"}}]}, "missing_field")
add_neg("record/unknown-field", "undeclared field supplied", POINT,
        {"record": [{"field": "x", "value": {"int64": "1"}},
                    {"field": "z", "value": {"int64": "2"}}]}, "unknown_field")
add_neg("record/duplicate-field", "field supplied twice", POINT,
        {"record": [{"field": "x", "value": {"int64": "1"}},
                    {"field": "x", "value": {"int64": "2"}}]}, "duplicate_field")
add_neg("map/duplicate-key", "identical map keys", MAP_I32,
        {"map": [{"key": "a", "value": {"int32": "1"}},
                 {"key": "a", "value": {"int32": "2"}}]}, "duplicate_normalized_key")
add_neg("map/nfc-duplicate-key", "NFC-equivalent map keys (é vs e+U+0301)", MAP_I32,
        {"map": [{"key": "é", "value": {"int32": "1"}},
                 {"key": "e\u0301", "value": {"int32": "2"}}]}, "duplicate_normalized_key")
add_neg("enum/unknown-value", "closed enum rejects unknown value", TASK_OUTCOME,
        {"enum": "ABORTED"}, "unknown_enum_value")
add_neg("variant/unknown-case", "closed variant rejects unknown case", RESULT,
        {"variant": {"case": "panic", "value": {"string": "x"}}}, "unknown_variant_case")

# remove the placeholder bytes case: it needs raw decoder input, handled below
NEGATIVES = [n for n in NEGATIVES if n["target"] != "placeholder"]

DECODER_NEGATIVES = []

def add_dec(cid, description, shape, raw_text, error_class):
    DECODER_NEGATIVES.append({
        "case_id": cid,
        "description": description,
        "shape": shape,
        "bytes": raw_text,
        "error_class": error_class,
        "target": "decoder",
        "requires_decoder": False,
    })

add_dec("dec/whitespace", "insignificant whitespace is not canonical", "int32", " \"42\" ",
        "non_canonical_encoding")
add_dec("dec/trailing-newline", "trailing newline is not canonical", "int32", "\"42\"\n",
        "non_canonical_encoding")
add_dec("dec/trailing-bytes", "trailing data after the value", "int32", "\"42\"x",
        "non_canonical_encoding")
add_dec("dec/bare-number", "bare JSON numbers are not ACE1", "int32", "42",
        "non_canonical_encoding")
add_dec("dec/unsorted-keys", "map keys not in byte order", MAP_I32,
        "{\"b\":\"1\",\"a\":\"2\"}", "non_canonical_encoding")
add_dec("dec/duplicate-keys", "duplicate map keys in bytes", MAP_I32,
        "{\"a\":\"1\",\"a\":\"2\"}", "non_canonical_encoding")
add_dec("dec/non-canonical-decimal", "decimal text not in canonical form", "decimal",
        "\"1.50\"", "non_canonical_encoding")
add_dec("dec/float-uppercase-hex", "uppercase float hex is not canonical", "float64",
        "\"3FF0000000000000\"", "non_canonical_encoding")
add_dec("dec/float-negative-zero", "-0 float bits are non-canonical", "float64",
        "\"8000000000000000\"", "non_canonical_encoding")
add_dec("dec/float-short-hex", "float hex must be 16 characters", "float64",
        "\"3ff00000000000\"", "non_canonical_encoding")
add_dec("dec/float-nan", "NaN bits rejected by decoder", "float64", "\"7ff8000000000000\"",
        "non_finite_float")
add_dec("dec/float-inf", "+Inf bits rejected by decoder", "float64", "\"7ff0000000000000\"",
        "non_finite_float")
add_dec("dec/padded-base64url", "padded base64url is forbidden", "bytes", "\"Zg==\"",
        "invalid_bytes_encoding")
add_dec("dec/standard-base64-alphabet", "'+' and '/' are forbidden", "bytes", "\"Zm8+\"",
        "invalid_bytes_encoding")
add_dec("dec/int-leading-zeros", "leading zeros in encoded integer", "int32", "\"007\"",
        "invalid_integer")
add_dec("dec/int-plus", "+ sign in encoded integer", "int32", "\"+1\"", "invalid_integer")
add_dec("dec/int-out-of-range", "encoded integer out of int32 range", "int32",
        "\"2147483648\"", "integer_out_of_range")
add_dec("dec/record-missing-field", "record bytes missing a declared field", POINT,
        "{\"x\":\"1\"}", "missing_field")
add_dec("dec/record-unknown-field", "record bytes with an undeclared field", POINT,
        "{\"x\":\"1\",\"z\":\"2\"}", "unknown_field")
add_dec("dec/record-unsorted-fields", "record members not in canonical order", POINT,
        "{\"y\":\"2\",\"x\":\"1\"}", "non_canonical_encoding")
add_dec("dec/variant-two-cases", "variant must encode exactly one case", RESULT,
        "{\"ok\":\"a\",\"error\":\"b\"}", "non_canonical_encoding")
add_dec("dec/variant-unknown-case", "unknown variant case in bytes", RESULT,
        "{\"panic\":\"x\"}", "unknown_variant_case")
add_dec("dec/enum-unknown-value", "unknown enum value in bytes", TASK_OUTCOME,
        "\"ABORTED\"", "unknown_enum_value")
add_dec("dec/escape-slash", "\\ escape is non-canonical", "string", "\"a\\/b\"",
        "non_canonical_encoding")
add_dec("dec/escape-uppercase-hex", "uppercase \\u hex digit is non-canonical", "string",
        "\"a\\u000B\"", "non_canonical_encoding")
add_dec("dec/escape-non-control", "\\u escape for a non-control is non-canonical", "string",
        "\"a\\u0041b\"", "non_canonical_encoding")
add_dec("dec/escape-short-form-required", "\\u000a must be written as \\n", "string",
        "\"a\\u000ab\"", "non_canonical_encoding")
add_dec("dec/raw-control-in-string", "raw control byte inside a string", "string",
        "\"a\u0001b\"", "non_canonical_encoding")
add_dec("dec/non-nfc-string", "raw non-NFC (decomposed) string content", "string",
        "\"e\u0301\"", "invalid_unicode")
add_dec("dec/non-nfc-map-key", "raw non-NFC (decomposed) map key", MAP_I32,
        "{\"e\u0301cole\":\"1\"}", "invalid_unicode")

# Option v1 (Phase 1A): explicit tagged objects only; null sugar is gone.
add_dec("dec/option-null-sugar", "bare null is no longer an Option::None encoding",
        {"option": "string"}, "null", "non_canonical_encoding")
add_dec("dec/option-none-int-payload",
        "'none' payload 1 is a bare JSON number — rejected by the canonical layer",
        {"option": "string"}, "{\"none\":1}", "non_canonical_encoding")
add_dec("dec/option-none-string-payload", "'none' payload must be literal null",
        {"option": "string"}, "{\"none\":\"x\"}", "invalid_option_encoding")
add_dec("dec/option-empty-object", "option must encode exactly one case member",
        {"option": "string"}, "{}", "invalid_option_encoding")
add_dec("dec/option-unknown-key", "option case keys are exactly none/some",
        {"option": "string"}, "{\"foo\":\"x\"}", "invalid_option_encoding")
add_dec("dec/option-two-members-sorted", "two case members (even canonically sorted) are invalid",
        {"option": "string"}, "{\"none\":null,\"some\":\"x\"}", "invalid_option_encoding")
add_dec("dec/option-two-members-unsorted", "non-canonical member order",
        {"option": "string"}, "{\"some\":\"x\",\"none\":null}", "non_canonical_encoding")
add_dec("dec/option-duplicate-key", "duplicate case key",
        {"option": "string"}, "{\"none\":null,\"none\":null}", "non_canonical_encoding")
add_neg("nominal/null-into-option", "bare null is a distinct nominal value, not Option::None",
        {"option": "string"}, {"null": None}, "type_mismatch")
add_neg("shape/duplicate-field-names", "record shape with duplicate field names is invalid",
        {"record": {"name": "Dup", "fields": [
            {"field": "x", "type": "int32"}, {"field": "x", "type": "int32"}]}},
        {"record": []}, "invalid_shape")

write_json(CONF / "encoding" / "negative.json", {
    "kind": "agentflow.conformance.ace1-negative",
    "ace1_version": "1",
    "note": (
        "Negative corpus. encoder cases fail while building/encoding the typed "
        "value; decoder cases fail while strict-decoding the raw bytes. "
        "error_class values are stable identifiers (AceError::kind / "
        "DigestError::kind). requires_decoder is false everywhere because this "
        "phase ships a strict decoder; the field remains for corpus evolution."
    ),
    "cases": NEGATIVES + DECODER_NEGATIVES,
})

# ---------------------------------------------------------------------------
# Digest golden corpus
# ---------------------------------------------------------------------------

DIGEST_CASES = []
for case_id, domain, payload in [
    ("empty-payload/value", "value", b""),
    ("ascii/plan-core", "plan-core", b"hello"),
    ("ascii/semantic-ir", "semantic-ir", b"hello"),
    ("prefix-in-payload/plan-core", "plan-core", b"AF\x00hello"),
    ("separator-in-payload/value", "value", b"a\x00b"),
    ("utf8/value", "value", "é".encode("utf-8")),
    ("utf8/journal-event-payload", "journal-event-payload", "中文 payload".encode("utf-8")),
    ("binary/lock-graph", "lock-graph", bytes(range(256))),
    ("long/conformance", "conformance", b"AF" * 512),
    ("run-manifest", "run-manifest", b"run-manifest-payload"),
    ("runtime-command-payload", "runtime-command-payload", b"command-payload"),
    ("semantic-execution-state", "semantic-execution-state", b"state-payload"),
    ("type-schema", "type-schema", b"schema-payload"),
    ("component-contract", "component-contract", b"component-payload"),
    ("resource-contract", "resource-contract", b"resource-payload"),
    ("artifact", "artifact", b"artifact-payload"),
    ("provenance", "provenance", b"provenance-payload"),
]:
    DIGEST_CASES.append({
        "case_id": case_id,
        "domain": domain,
        "payload_hex": payload.hex(),
        "expected_sha256_hex": hash_v1(domain, payload),
    })

write_json(CONF / "digest" / "hash-v1-golden.json", {
    "kind": "agentflow.conformance.hash-v1-golden",
    "hash_algorithm": "sha256",
    "formula": "SHA256(\"AF\\0\" || domain || \"\\0\" || payload)",
    "note": (
        "Golden HashV1 vectors. payload_hex is the exact payload byte sequence; "
        "expected_sha256_hex was computed with Python hashlib against the frozen "
        "formula and must never be regenerated by the implementation under test."
    ),
    "cases": DIGEST_CASES,
})

SEP_PAYLOAD = b"domain-separation-probe-payload"
SEP_CASES = [{"domain": d, "expected_sha256_hex": hash_v1(d, SEP_PAYLOAD)} for d in DOMAINS]
assert len({c["expected_sha256_hex"] for c in SEP_CASES}) == len(DOMAINS), \
    "domain separation produced a collision"

write_json(CONF / "digest" / "domain-separation.json", {
    "kind": "agentflow.conformance.hash-v1-domain-separation",
    "hash_algorithm": "sha256",
    "payload_hex": SEP_PAYLOAD.hex(),
    "note": (
        "One fixed payload hashed under every frozen domain. All digests must be "
        "pairwise distinct and match the frozen values."
    ),
    "domains": SEP_CASES,
})

# Negative digest cases
DIGEST_NEGATIVES = [
    {"case_id": "digest/unknown-algorithm", "description": "md5 is not a v1 digest algorithm",
     "input": "md5/value/0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
     "error_class": "invalid_digest"},
    {"case_id": "digest/unknown-domain",
     "description": "domain outside the frozen registry",
     "input": "sha256/not-a-domain/0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
     "error_class": "unknown_digest_domain"},
    {"case_id": "digest/short-hex", "description": "63 hex characters",
     "input": "sha256/value/0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcde",
     "error_class": "invalid_digest"},
    {"case_id": "digest/long-hex", "description": "65 hex characters",
     "input": "sha256/value/0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdeff",
     "error_class": "invalid_digest"},
    {"case_id": "digest/uppercase-hex", "description": "uppercase digest hex is forbidden",
     "input": "sha256/value/0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF",
     "error_class": "invalid_digest"},
    {"case_id": "digest/non-hex", "description": "non-hex characters in digest",
     "input": "sha256/value/zz23456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
     "error_class": "invalid_digest"},
    {"case_id": "digest/malformed", "description": "wrong number of '/'-separated parts",
     "input": "sha256/value", "error_class": "invalid_digest"},
]

write_json(CONF / "digest" / "negative.json", {
    "kind": "agentflow.conformance.hash-v1-negative",
    "note": "Negative cases for DigestRef parsing/validation.",
    "cases": DIGEST_NEGATIVES,
})

print("conformance corpus frozen.")
