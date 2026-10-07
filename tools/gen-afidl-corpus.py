#!/usr/bin/env python3
"""Freezes the AFIDL1 conformance corpus (schema golden + digest golden).

This is an INDEPENDENT implementation of the AFIDL1 grammar (Section 2 of
spec/idl/afidl1-grammar.md), the canonical TypeSchema machine model (Section 6)
and TypeSchemaDigest (Section 7). The Rust implementation in crates/af-idl must
agree with the frozen expectations byte-for-byte; tests never generate expected
values at runtime.

Run from the repository root:  python tools/gen-afidl-corpus.py
"""

import hashlib
import json
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CONF = ROOT / "spec" / "conformance" / "afidl"
CORPUS_REVISION = 1

PRIMITIVES = [
    "unit", "null", "bool", "int32", "uint32", "int64", "uint64",
    "decimal", "float64", "string", "bytes", "timestamp", "duration",
]

# ---------------------------------------------------------------------------
# Independent AFIDL1 implementation (lexer + parser, valid inputs only)
# ---------------------------------------------------------------------------

class AfidfSyntaxError(Exception):
    pass


PUNCT = {'{', '}', '<', '>', ';', ':', ',', '=', '@', '.'}


def tokenize(src):
    tokens = []  # (kind, text, line, col)
    i, line, col = 0, 1, 1
    n = len(src)
    while i < n:
        c = src[i]
        if c in ' \t\r':
            i += 1; col += 1
        elif c == '\n':
            i += 1; line += 1; col = 1
        elif c == '/' and i + 1 < n and src[i + 1] == '/':
            while i < n and src[i] != '\n':
                i += 1
        elif c == '/' and i + 1 < n and src[i + 1] == '*':
            i += 2; col += 2
            while True:
                if i + 1 >= n:
                    raise AfidfSyntaxError(f"unterminated block comment at line {line}")
                if src[i] == '*' and src[i + 1] == '/':
                    i += 2; col += 2
                    break
                if src[i] == '\n':
                    i += 1; line += 1; col = 1
                else:
                    i += 1; col += 1
        elif c in PUNCT:
            tokens.append((c, c, line, col))
            i += 1; col += 1
        elif c.isascii() and c.isupper():
            j = i
            while j < n and src[j].isascii() and src[j].isalnum():
                j += 1
            tokens.append(("UpperIdent", src[i:j], line, col))
            col += j - i; i = j
        elif c.isascii() and c.islower():
            j = i
            while j < n and (src[j].isascii() and (src[j].islower() or src[j].isdigit() or src[j] in ('_', '-'))):
                j += 1
            tokens.append(("LowerIdent", src[i:j], line, col))
            col += j - i; i = j
        elif c.isascii() and c.isdigit():
            j = i
            while j < n and src[j].isascii() and src[j].isdigit():
                j += 1
            text = src[i:j]
            if len(text) > 1 and text.startswith('0'):
                raise AfidfSyntaxError(f"leading zeros in integer {text!r} at line {line}")
            tokens.append(("Int", text, line, col))
            col += j - i; i = j
        else:
            raise AfidfSyntaxError(f"unexpected character {c!r} at line {line}")
    return tokens


class Parser:
    def __init__(self, tokens):
        self.tokens = tokens
        self.pos = 0

    def peek(self):
        return self.tokens[self.pos] if self.pos < len(self.tokens) else None

    def bump(self):
        token = self.tokens[self.pos]
        self.pos += 1
        return token

    def expect(self, kind, text=None):
        token = self.peek()
        if token is None or token[0] != kind or (text is not None and token[1] != text):
            found = token[1] if token else "end of file"
            raise AfidfSyntaxError(f"expected {text or kind}, found {found!r}")
        return self.bump()

    def module(self):
        self.expect("LowerIdent", "afidl")
        version = int(self.expect("Int")[1])
        if version != 1:
            raise AfidfSyntaxError(f"unsupported AFIDL language version {version}")
        self.expect("Semicolon" if False else ';')
        self.expect("LowerIdent", "schema")
        spec = self.specification_id()
        self.expect(';')
        declarations = []
        while self.peek() is not None:
            token = self.peek()
            if token[0] == "LowerIdent" and token[1] == "alias":
                declarations.append(self.alias_decl())
            elif token[0] == "LowerIdent" and token[1] == "enum":
                declarations.append(self.enum_decl())
            elif token[0] == "LowerIdent" and token[1] == "record":
                declarations.append(self.record_decl())
            elif token[0] == "LowerIdent" and token[1] == "variant":
                declarations.append(self.variant_decl())
            else:
                raise AfidfSyntaxError(
                    f"expected a declaration, found {token[1]!r} at line {token[2]}")
        return {"spec": spec, "declarations": declarations}

    def specification_id(self):
        segments = []
        token = self.peek()
        if token is None or token[0] != "LowerIdent":
            raise AfidfSyntaxError("schema ids start with a lowercase segment")
        segments.append(self.bump()[1])
        while True:
            token = self.peek()
            if token is None:
                raise AfidfSyntaxError("schema id ended at end of file")
            if token[0] == '.':
                self.bump()
                segments.append(self.expect("LowerIdent")[1])
            elif token[0] == '@':
                self.bump()
                major = int(self.expect("Int")[1])
                if major <= 0:
                    raise AfidfSyntaxError("schema major must be positive")
                return {"namespace": ".".join(segments), "major": major}
            else:
                raise AfidfSyntaxError(f"expected '.' or '@' in schema id, found {token[1]!r}")

    def type_name(self):
        token = self.peek()
        if token is None or token[0] != "UpperIdent":
            raise AfidfSyntaxError(
                f"type names are UpperCamelCase, found {token[1] if token else 'end of file'!r}")
        return self.bump()[1]

    def lower_name(self):
        token = self.peek()
        if token is None or token[0] != "LowerIdent":
            raise AfidfSyntaxError(
                f"names are lowercase identifiers, found {token[1] if token else 'end of file'!r}")
        text = self.bump()[1]
        if any(ch == '-' for ch in text[1:]):
            raise AfidfSyntaxError(f"field/case/value names may not contain '-': {text!r}")
        return text

    def alias_decl(self):
        self.expect("LowerIdent", "alias")
        name = self.type_name()
        self.expect('=')
        target = self.type_expr()
        self.expect(';')
        return {"kind": "alias", "name": name, "target": target}

    def enum_decl(self):
        self.expect("LowerIdent", "enum")
        name = self.type_name()
        self.expect('{')
        values = []
        while self.peek()[0] != '}':
            no = int(self.expect("Int")[1])
            value_name = self.lower_name()
            self.expect(';')
            values.append({"no": no, "name": value_name})
        self.bump()  # '}'
        return {"kind": "enum", "name": name, "values": values}

    def record_decl(self):
        self.expect("LowerIdent", "record")
        name = self.type_name()
        self.expect('{')
        fields = []
        while self.peek()[0] != '}':
            no = int(self.expect("Int")[1])
            field_name = self.lower_name()
            self.expect(':')
            type_expr = self.type_expr()
            self.expect(';')
            fields.append({"no": no, "name": field_name, "type": type_expr})
        self.bump()  # '}'
        return {"kind": "record", "name": name, "fields": fields}

    def variant_decl(self):
        self.expect("LowerIdent", "variant")
        name = self.type_name()
        self.expect('{')
        cases = []
        while self.peek()[0] != '}':
            no = int(self.expect("Int")[1])
            case_name = self.lower_name()
            self.expect(':')
            payload = self.type_expr()
            self.expect(';')
            cases.append({"no": no, "name": case_name, "type": payload})
        self.bump()  # '}'
        return {"kind": "variant", "name": name, "cases": cases}

    def type_expr(self):
        token = self.peek()
        if token is None:
            raise AfidfSyntaxError("expected a type, found end of file")
        kind, text = token[0], token[1]
        if kind == "UpperIdent":
            self.bump()
            return ("ref", text)
        if kind == "LowerIdent":
            if text == "list":
                self.bump(); self.expect('<')
                inner = self.type_expr()
                self.expect('>')
                return ("list", inner)
            if text == "map":
                self.bump(); self.expect('<')
                key = self.lower_name()
                if key != "string":
                    raise AfidfSyntaxError(f"map keys are exactly 'string', found {key!r}")
                self.expect(',')
                inner = self.type_expr()
                self.expect('>')
                return ("map_string", inner)
            if text == "option":
                self.bump(); self.expect('<')
                inner = self.type_expr()
                self.expect('>')
                return ("option", inner)
            if text in PRIMITIVES:
                self.bump()
                return ("primitive", text)
            self.bump()
            return ("ref", text)
        raise AfidfSyntaxError(f"expected a type, found {text!r}")


def parse_afidl(source):
    return Parser(tokenize(source)).module()


# ---------------------------------------------------------------------------
# Canonical TypeSchema model + ACE1 encoding (Section 6)
# ---------------------------------------------------------------------------

def canonical_type_expr(spec_display, expr):
    kind, value = expr
    if kind == "primitive":
        return ("primitive", value)
    if kind == "ref":
        return ("ref", f"{spec_display}/{value}")
    if kind in ("list", "map_string", "option"):
        return (kind, canonical_type_expr(spec_display, value))
    raise AfidfSyntaxError(f"unknown type expr kind {kind!r}")


def canonical_type(spec_display, declaration):
    kind = declaration["kind"]
    if kind == "alias":
        body = ("alias", canonical_type_expr(spec_display, declaration["target"]))
    elif kind == "enum":
        values = sorted(
            ({"no": v["no"], "name": v["name"]} for v in declaration["values"]),
            key=lambda v: v["no"])
        body = ("enum", {"values": values})
    elif kind == "record":
        fields = sorted(
            ({"no": f["no"], "name": f["name"],
              "type": canonical_type_expr(spec_display, f["type"])}
             for f in declaration["fields"]),
            key=lambda f: f["no"])
        body = ("record", {"fields": fields})
    elif kind == "variant":
        cases = sorted(
            ({"no": c["no"], "name": c["name"],
              "type": canonical_type_expr(spec_display, c["type"])}
             for c in declaration["cases"]),
            key=lambda c: c["no"])
        body = ("variant", {"cases": cases})
    else:
        raise AfidfSyntaxError(f"unknown declaration kind {kind!r}")
    return {"schema": spec_display, "name": declaration["name"], "body": body}


def nfc(s):
    return unicodedata.normalize("NFC", s)


def escape_string(s):
    out = []
    for ch in s:
        o = ord(ch)
        if ch == '"':
            out.append('\\"')
        elif ch == '\\':
            out.append('\\\\')
        elif ch == '\b':
            out.append('\\b')
        elif ch == '\t':
            out.append('\\t')
        elif ch == '\n':
            out.append('\\n')
        elif ch == '\f':
            out.append('\\f')
        elif ch == '\r':
            out.append('\\r')
        elif o < 0x20:
            out.append('\\u%04x' % o)
        else:
            out.append(ch)
    return '"' + ''.join(out) + '"'


def encode_ace1(value):
    """Encodes the canonical-schema value tree with ACE1 rules.

    value := dict (record) | ("variant-case", payload) | list | str | int
    """
    if isinstance(value, dict):
        members = sorted(value, key=lambda k: k.encode("utf-8"))
        return "{" + ",".join(
            escape_string(nfc(k)) + ":" + encode_ace1(value[k]) for k in members) + "}"
    if isinstance(value, tuple):
        assert len(value) == 2, "variant is a (case, payload) pair"
        case, payload = value
        return "{" + escape_string(nfc(case)) + ":" + encode_ace1(payload) + "}"
    if isinstance(value, list):
        return "[" + ",".join(encode_ace1(item) for item in value) + "]"
    if isinstance(value, str):
        return escape_string(nfc(value))
    if isinstance(value, int):
        return escape_string(str(value))
    raise AfidfSyntaxError(f"cannot encode {type(value)!r}")


def canonical_ace1_bytes(schema):
    """Builds the machine value of a canonical TypeSchema and encodes it."""
    kind, payload = schema["body"]
    if kind == "alias":
        body = {"alias": payload}
    elif kind == "enum":
        body = {"enum": {"values": [
            {"no": v["no"], "name": v["name"]} for v in payload["values"]]}}
    elif kind == "record":
        body = {"record": {"fields": [
            {"no": f["no"], "name": f["name"], "type": f["type"]} for f in payload["fields"]]}}
    else:
        body = {"variant": {"cases": [
            {"no": c["no"], "name": c["name"], "type": c["type"]} for c in payload["cases"]]}}

    machine_value = {
        "schema": schema["schema"],
        "name": schema["name"],
        "body": body,
    }
    return encode_ace1(machine_value).encode("utf-8")


def type_schema_digest(schema):
    payload = canonical_ace1_bytes(schema)
    digest = hashlib.sha256(b"AF\x00type-schema\x00" + payload).hexdigest()
    return payload, digest


# ---------------------------------------------------------------------------
# Corpus generation
# ---------------------------------------------------------------------------

def compile_file(path: Path):
    module = parse_afidl(path.read_text(encoding="utf-8"))
    spec_display = f"{module['spec']['namespace']}@{module['spec']['major']}"
    types = []
    for declaration in module["declarations"]:
        schema = canonical_type(spec_display, declaration)
        payload, digest = type_schema_digest(schema)
        types.append({
            "name": schema["name"],
            "ace1_utf8": payload.decode("utf-8"),
            "ace1_hex": payload.hex(),
            "digest_hex": digest,
        })
    types.sort(key=lambda t: t["name"])
    return types


def case_entry(path: Path, case_id=None):
    rel = path.relative_to(ROOT).as_posix()
    entry = {"case_id": case_id or path.stem, "file": rel, "types": compile_file(path)}
    return entry


def write_json(path, payload):
    payload = {"corpus_revision": CORPUS_REVISION, **payload}
    path.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"wrote {path.relative_to(ROOT)}")


def main():
    valid_dir = CONF / "parser-valid"

    cases = []
    for source in sorted(valid_dir.glob("*.afidl")):
        cases.append(case_entry(source))
    # The normative builtin registry file is itself part of the corpus.
    primitives_file = ROOT / "spec" / "idl" / "core" / "machine-primitives.afidl"
    cases.append(case_entry(primitives_file, case_id="machine-primitives"))

    schema_golden = {
        "kind": "agentflow.conformance.afidl.schema-golden",
        "note": (
            "Frozen canonical ACE1 bytes of every canonical TypeSchema in the "
            "AFIDL1 corpus, generated by the independent Python implementation "
            "(this script). The Rust implementation must reproduce them "
            "byte-for-byte; tests never generate expected values at runtime."
        ),
        "cases": cases,
        "equivalence_groups": [
            ["record-multi-field", "record-reordered", "record-multi-field-comments"],
            ["variant", "variant-reordered"],
            ["enum", "enum-reordered"],
            ["module-order", "module-order-reordered"],
        ],
    }
    write_json(CONF / "schema-golden.json", schema_golden)

    digest_cases = []
    by_key = {}
    for case in cases:
        for type_entry in case["types"]:
            key = (case["case_id"], type_entry["name"])
            by_key[key] = type_entry["digest_hex"]
            digest_cases.append({
                "case_id": case["case_id"],
                "file": case["file"],
                "type_name": type_entry["name"],
                "digest_hex": type_entry["digest_hex"],
            })

    mutation_sources = {
        "mutation/field-no": ("record-multi-field", "mut-field-no", "User"),
        "mutation/field-name": ("record-multi-field", "mut-field-name", "User"),
        "mutation/field-type": ("record-multi-field", "mut-field-type", "User"),
        "mutation/case-no": ("variant", "mut-case-no", "LookupResult"),
        "mutation/case-name": ("variant", "mut-case-name", "LookupResult"),
        "mutation/case-payload": ("variant", "mut-case-payload", "LookupResult"),
        "mutation/enum-no": ("enum", "mut-enum-no", "Status"),
        "mutation/enum-name": ("enum", "mut-enum-name", "Status"),
        "mutation/alias-target": ("alias-string", "mut-alias-target", "UserId"),
        "mutation/schema-major": ("record-one-field", "mut-major", "Single"),
    }
    mutation_cases = []
    for case_id, (base, mutated, type_name) in mutation_sources.items():
        base_digest = by_key[(base, type_name)]
        mutated_digest = by_key[(mutated, type_name)]
        assert base_digest != mutated_digest, f"mutation {case_id} did not change the digest"
        mutation_cases.append({
            "case_id": case_id,
            "base": base,
            "mutated": mutated,
            "type_name": type_name,
            "expect": "different-digest",
        })

    digest_golden = {
        "kind": "agentflow.conformance.afidl.type-schema-digest-golden",
        "note": (
            "Frozen TypeSchemaDigest values "
            "(SHA256(\"AF\\0type-schema\\0\" || canonical ACE1 bytes)). Mutation "
            "cases assert that semantic changes alter the digest while the "
            "equivalence groups in schema-golden.json assert that formatting "
            "does not."
        ),
        "cases": digest_cases,
        "mutation_cases": mutation_cases,
    }
    write_json(CONF / "type-schema-digest-golden.json", digest_golden)

    # Self-check: machine-primitives.afidl must expose exactly the closed
    # primitive universe through its aliases.
    mp = next(c for c in cases if c["case_id"] == "machine-primitives")
    alias_targets = set()
    source_text = primitives_file.read_text(encoding="utf-8")
    module = parse_afidl(source_text)
    for declaration in module["declarations"]:
        assert declaration["kind"] == "alias"
        target = declaration["target"]
        assert target[0] == "primitive", "registry aliases must target primitives"
        alias_targets.add(target[1])
    assert alias_targets == set(PRIMITIVES), "builtin universe drifted"
    assert len(mp["types"]) == len(PRIMITIVES)
    print("machine-primitives.afidl alias targets match the closed primitive table")


if __name__ == "__main__":
    main()
