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
# Revision 10 (Phase 4A.5A): adds the `usage-signature` case for the
# Static Usage Signature schema module (the U dimension: additive meter
# identity, forward/recovery phases and finite/unbounded upper bounds).
# Revision 9 (Phase 4A.4): adds the `information-flow` case for the
# Static Information Flow Contract schema module (the F dimension:
# confidentiality labels, taints, instruction authority, static
# influence summary, flow sinks and the closed flow contract).
# Revision 8 (Phase 4A.3): adds the `resource-requirement` case for the
# Static Resource Requirement Row schema module (the R dimension:
# ResourceTypeRef, ResourceAttributeConstraint, ResourceRequirement,
# ResourceRequirementRow).
# Revision 7 (Phase 4A.2): adds the `failure-row` case for the static
# Typed Failure Row schema module (nominal FailureTypeRef over the core's
# TypeSchemaRef, plus the closed FailureRow set).
# Revision 6 (Phase 4A.1): added the `effect-signature` case for the Effect
# Signature schema module (the second production cross-module importer; it
# reuses the semantic core's PrimitiveConstant for guard payloads).
# Revision 5 (Phase 3B) added the `semantic-ir` case; revision 4 (Phase 2C)
# added cross-module import cases; revision 3 (Phase 3A) added the
# `semantic-core` case; revision 2 (Phase 2B) added the self-hosting
# registry schema module. Every bump is purely additive: existing cases,
# bytes and digests are unchanged.
CORPUS_REVISION = 10

# The frozen 13-opcode case table (spec/semantic/semantic-ir.md section 4).
OPCODE_TABLE = [
    (1, "invoke"), (2, "await"), (3, "sequence"), (4, "select"),
    (5, "parallel"), (6, "race"), (7, "iterate"), (8, "retry"),
    (9, "timeout"), (10, "handle"), (11, "scope"),
    (12, "compensation_scope"), (13, "call_subflow"),
]

PRIMITIVES = [
    "unit", "null", "bool", "int32", "uint32", "int64", "uint64",
    "decimal", "float64", "string", "bytes", "timestamp", "duration",
]

# Reserved import aliases: exactly the words with structural meaning in a type
# position (spec/idl/afidl1-grammar.md section 5.1).
RESERVED_IMPORT_ALIASES = {
    "afidl", "schema", "import", "as", "alias", "enum", "record", "variant",
    "list", "map", "option", *PRIMITIVES,
}

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
        self.expect(';')
        self.expect("LowerIdent", "schema")
        spec = self.specification_id()
        self.expect(';')
        imports = []
        declarations = []
        seen_declaration = False
        while self.peek() is not None:
            token = self.peek()
            if token[0] == "LowerIdent" and token[1] == "import":
                if seen_declaration:
                    raise AfidfSyntaxError(
                        f"import after a declaration at line {token[2]}: imports "
                        "belong to the module header")
                imports.append(self.import_decl())
            elif token[0] == "LowerIdent" and token[1] in (
                    "alias", "enum", "record", "variant"):
                seen_declaration = True
                if token[1] == "alias":
                    declarations.append(self.alias_decl())
                elif token[1] == "enum":
                    declarations.append(self.enum_decl())
                elif token[1] == "record":
                    declarations.append(self.record_decl())
                else:
                    declarations.append(self.variant_decl())
            else:
                raise AfidfSyntaxError(
                    f"expected a declaration, found {token[1]!r} at line {token[2]}")
        return {"spec": spec, "imports": imports, "declarations": declarations}

    def import_decl(self):
        keyword = self.expect("LowerIdent", "import")
        import_spec = self.specification_id()
        self.expect("LowerIdent", "as")
        alias = self.import_alias()
        self.expect(';')
        return {"spec": import_spec, "alias": alias, "line": keyword[2]}

    def import_alias(self):
        token = self.peek()
        if token is None or token[0] != "LowerIdent":
            raise AfidfSyntaxError("expected an import alias")
        text = self.bump()[1]
        if '-' in text or text in RESERVED_IMPORT_ALIASES:
            raise AfidfSyntaxError(f"invalid or reserved import alias {text!r}")
        return text

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
            # `<alias>.<TypeName>` is a qualified reference; a local type name
            # is UpperCamelCase so the two never collide.
            if self.tokens[self.pos + 1:self.pos + 2] and \
                    self.tokens[self.pos + 1][0] == '.':
                self.bump()  # alias
                self.bump()  # '.'
                if text in PRIMITIVES:
                    raise AfidfSyntaxError(
                        f"qualified primitive reference {text!r}: primitives are "
                        "spelled bare")
                name_token = self.peek()
                if name_token is None or name_token[0] != "UpperIdent":
                    found = name_token[1] if name_token else "end of file"
                    raise AfidfSyntaxError(
                        f"qualified references name an UpperCamelCase type, found {found!r}")
                self.bump()
                return ("qualified_ref", {"alias": text, "name": name_token[1]})
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

def canonical_type_expr(spec_display, expr, imports=None):
    """Lowers a source type expression to the canonical form.

    `imports` maps an import alias to the imported specification's canonical
    display form. It is used *only* to resolve a qualified reference's target
    specification identity; the alias itself never reaches the canonical
    schema (spec/idl/afidl1-grammar.md section 5.3).
    """
    kind, value = expr
    if kind == "primitive":
        return ("primitive", value)
    if kind == "ref":
        return ("ref", f"{spec_display}/{value}")
    if kind == "qualified_ref":
        alias = value["alias"]
        if imports is None or alias not in imports:
            raise AfidfSyntaxError(f"unknown import alias {alias!r}")
        return ("ref", f"{imports[alias]}/{value['name']}")
    if kind in ("list", "map_string", "option"):
        return (kind, canonical_type_expr(spec_display, value, imports))
    raise AfidfSyntaxError(f"unknown type expr kind {kind!r}")


def canonical_type(spec_display, declaration, imports=None):
    kind = declaration["kind"]
    if kind == "alias":
        body = ("alias", canonical_type_expr(spec_display, declaration["target"], imports))
    elif kind == "enum":
        values = sorted(
            ({"no": v["no"], "name": v["name"]} for v in declaration["values"]),
            key=lambda v: v["no"])
        body = ("enum", {"values": values})
    elif kind == "record":
        fields = sorted(
            ({"no": f["no"], "name": f["name"],
              "type": canonical_type_expr(spec_display, f["type"], imports)}
             for f in declaration["fields"]),
            key=lambda f: f["no"])
        body = ("record", {"fields": fields})
    elif kind == "variant":
        cases = sorted(
            ({"no": c["no"], "name": c["name"],
              "type": canonical_type_expr(spec_display, c["type"], imports)}
             for c in declaration["cases"]),
            key=lambda c: c["no"])
        body = ("variant", {"cases": cases})
    else:
        raise AfidfSyntaxError(f"unknown declaration kind {kind!r}")
    return {"schema": spec_display, "name": declaration["name"], "body": body}


def import_table(module):
    """alias -> canonical imported specification display form.

    Duplicate aliases and self/duplicate-specification imports are rejected
    here as well: the generator is an independent implementation of the same
    frozen rules (spec/idl/afidl1-grammar.md section 5.1).
    """
    this = f"{module['spec']['namespace']}@{module['spec']['major']}"
    table = {}
    imported = set()
    for entry in module["imports"]:
        alias = entry["alias"]
        spec = entry["spec"]
        display = f"{spec['namespace']}@{spec['major']}"
        if alias in table:
            raise AfidfSyntaxError(f"duplicate import alias {alias!r}")
        if display == this:
            raise AfidfSyntaxError(f"self import of {display!r}")
        if display in imported:
            raise AfidfSyntaxError(f"duplicate import of specification {display!r}")
        table[alias] = display
        imported.add(display)
    return table


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
# Module-set compilation: cross-module resolution and the global type graph
# ---------------------------------------------------------------------------

class ModuleSetError(Exception):
    """A module-set level failure (unresolved import, cross-module cycle)."""


def compile_module_set(sources):
    """Compiles a set of (label, source) pairs and validates the global
    named-type dependency graph.

    Independent Python implementation of `af_idl::compile_module_set`. The
    module *import* graph may contain cycles; only the named-type graph must be
    acyclic (spec/idl/afidl1-grammar.md section 5.4).
    """
    modules = {}
    types_by_module = {}
    declared = {}
    for label, source in sources:
        module, imports, types = compile_module_from_source(source)
        spec = f"{module['spec']['namespace']}@{module['spec']['major']}"
        if spec in modules:
            raise ModuleSetError(f"duplicate module specification {spec!r}")
        modules[spec] = (label, imports, module)
        types_by_module[spec] = types
        declared[spec] = {t["name"] for t in types}

    for spec, (label, imports, module) in modules.items():
        for alias, target in imports.items():
            if target not in modules:
                raise ModuleSetError(
                    f"{label}: unknown imported specification {target!r}")
        for declaration in module["declarations"]:
            for expr in _type_exprs(declaration):
                _check_qualified(imports, declared, expr)

    edges = {}
    for spec, types in types_by_module.items():
        for schema in types:
            edges[f"{spec}/{schema['name']}"] = _targets(schema["body"])

    _check_acyclic(edges)
    return modules, types_by_module


def _type_exprs(declaration):
    kind = declaration["kind"]
    if kind == "alias":
        return [declaration["target"]]
    if kind in ("record", "variant"):
        members = declaration.get("fields") or declaration.get("cases")
        return [m["type"] for m in members]
    return []


def _check_qualified(imports, declared, expr):
    kind, value = expr
    if kind == "qualified_ref":
        alias = value["alias"]
        if alias not in imports:
            raise ModuleSetError(f"unknown import alias {alias!r}")
        target = imports[alias]
        if value["name"] not in declared.get(target, set()):
            raise ModuleSetError(
                f"{target} declares no type {value['name']!r}")
        return
    if kind in ("list", "map_string", "option"):
        _check_qualified(imports, declared, value)


def _targets(body):
    kind, payload = body
    if kind == "alias":
        return _expr_targets(payload)
    if kind == "enum":
        return []
    members = payload.get("fields") or payload.get("cases")
    out = []
    for member in members:
        out.extend(_expr_targets(member["type"]))
    return out


def _expr_targets(expr):
    kind, value = expr
    if kind == "ref":
        return [value]
    if kind in ("list", "map_string", "option"):
        return _expr_targets(value)
    return []


def _body_identities(body):
    """Every full logical type identity referenced by one canonical body."""
    kind, payload = body
    if kind == "alias":
        return set(_expr_targets(payload))
    if kind == "enum":
        return set()
    members = payload.get("fields") or payload.get("cases")
    out = set()
    for member in members:
        out.update(_expr_targets(member["type"]))
    return out


def _check_acyclic(edges):
    done = set()

    def walk(node, path):
        for target in edges.get(node, []):
            if target in path:
                cycle = path[path.index(target):] + [target]
                raise ModuleSetError(
                    f"cross-module recursive type reference {' -> '.join(cycle)}")
            if target not in edges or target in done:
                continue
            path.append(target)
            walk(target, path)
            path.pop()
        done.add(node)

    for start in edges:
        if start not in done:
            walk(start, [start])


# ---------------------------------------------------------------------------
# Corpus generation
# ---------------------------------------------------------------------------

def compile_module_from_source(source):
    """Parses and canonicalizes one module source, returning
    (module, imports, canonical types as (name, schema) pairs)."""
    module = parse_afidl(source)
    spec_display = f"{module['spec']['namespace']}@{module['spec']['major']}"
    imports = import_table(module)
    types = [canonical_type(spec_display, declaration, imports)
             for declaration in module["declarations"]]
    types.sort(key=lambda t: t["name"])
    return module, imports, types


def compile_file(path: Path):
    return compile_module_from_source(path.read_text(encoding="utf-8"))[2]


def type_entries(types):
    entries = []
    for schema in types:
        payload, digest = type_schema_digest(schema)
        entries.append({
            "name": schema["name"],
            "ace1_utf8": payload.decode("utf-8"),
            "ace1_hex": payload.hex(),
            "digest_hex": digest,
        })
    return entries


def case_entry(path: Path, case_id=None, requires=None):
    rel = path.relative_to(ROOT).as_posix()
    types = compile_file(path)
    entry = {
        "case_id": case_id or path.stem,
        "file": rel,
        "types": type_entries(types),
    }
    if requires:
        entry["requires"] = [
            {"case_id": target, "file": (CONF / "parser-valid" / f"{target}.afidl")
             .relative_to(ROOT).as_posix()}
            for target in requires
        ]
    return entry


def write_json(path, payload):
    payload = {"corpus_revision": CORPUS_REVISION, **payload}
    path.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    print(f"wrote {path.relative_to(ROOT)}")


def main():
    valid_dir = CONF / "parser-valid"

    # Cross-module fixtures: `requires` names the parser-valid cases whose
    # modules form the import universe for a universe-compiled fixture.
    import_requires = {
        "import-basic": ["import-core"],
        "import-multiple": ["import-core", "import-second"],
        "import-nested-container-ref": ["import-core"],
        "import-alias-rename-a": ["import-core"],
        "import-alias-rename-b": ["import-core"],
        "import-order-a": ["import-core", "import-second"],
        "import-order-b": ["import-core", "import-second"],
        "import-unused-b": ["import-core"],
    }

    cases = []
    for source in sorted(valid_dir.glob("*.afidl")):
        cases.append(case_entry(source, requires=import_requires.get(source.stem)))
    # The normative builtin registry file is itself part of the corpus.
    primitives_file = ROOT / "spec" / "idl" / "core" / "machine-primitives.afidl"
    cases.append(case_entry(primitives_file, case_id="machine-primitives"))
    # The Phase 2B self-hosting registry schema module is part of the corpus.
    registry_file = ROOT / "spec" / "idl" / "specification-registry.afidl"
    cases.append(case_entry(registry_file, case_id="specification-registry"))
    # The Phase 3A workflow semantic core schema module is part of the corpus.
    semantic_file = ROOT / "spec" / "idl" / "semantic-core.afidl"
    cases.append(case_entry(semantic_file, case_id="semantic-core"))
    # The Phase 3B canonical Semantic IR schema module is part of the corpus.
    ir_file = ROOT / "spec" / "idl" / "semantic-ir.afidl"
    cases.append(case_entry(ir_file, case_id="semantic-ir"))
    # The Phase 4A.1 Effect Signature schema module is part of the corpus.
    effect_file = ROOT / "spec" / "idl" / "effect-signature.afidl"
    cases.append(case_entry(effect_file, case_id="effect-signature"))
    # The Phase 4A.2 Failure Row schema module is part of the corpus.
    failure_file = ROOT / "spec" / "idl" / "failure-row.afidl"
    cases.append(case_entry(failure_file, case_id="failure-row"))
    # The Phase 4A.3 Resource Requirement Row schema module is part of the
    # corpus.
    resource_file = ROOT / "spec" / "idl" / "resource-requirement.afidl"
    cases.append(case_entry(resource_file, case_id="resource-requirement"))
    # The Phase 4A.4 Information Flow schema module is part of the corpus.
    flow_file = ROOT / "spec" / "idl" / "information-flow.afidl"
    cases.append(case_entry(flow_file, case_id="information-flow"))
    # The Phase 4A.5A Usage Signature schema module is part of the corpus.
    usage_file = ROOT / "spec" / "idl" / "usage-signature.afidl"
    cases.append(case_entry(usage_file, case_id="usage-signature"))

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
            ["import-alias-rename-a", "import-alias-rename-b"],
            ["import-order-a", "import-order-b"],
            ["import-unused-a", "import-unused-b"],
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

    # --- cross-module import conformance -----------------------------------
    # Every fixture with a `requires` list is compiled through the same
    # module-set path the Rust harness uses, and the resulting canonical bytes
    # must equal the bytes already frozen above (imports are not identity
    # material: the universe only adds validity).
    frozen = {(case["case_id"], t["name"]): t for case in cases for t in case["types"]}
    import_cases = []
    for case in cases:
        requires = import_requires.get(case["case_id"])
        if not requires:
            continue
        universe_sources = [
            (f"{target}.afidl", (valid_dir / f"{target}.afidl").read_text(encoding="utf-8"))
            for target in requires
        ]
        consumer_source = (ROOT / case["file"]).read_text(encoding="utf-8")
        modules, types_by_module = compile_module_set(
            [*universe_sources, (f"{case['case_id']}.afidl", consumer_source)])
        consumer_spec = next(
            spec for spec, (label, _imports, _module) in modules.items()
            if label == f"{case['case_id']}.afidl")
        recompiled = type_entries(types_by_module[consumer_spec])
        for entry in recompiled:
            original = frozen[(case["case_id"], entry["name"])]
            assert entry["ace1_hex"] == original["ace1_hex"], (
                f"{case['case_id']}/{entry['name']}: module-set compilation "
                "diverged from single-module compilation")
            assert entry["digest_hex"] == original["digest_hex"], (
                f"{case['case_id']}/{entry['name']}: digest diverged")
        # For each import, record which specification the alias actually
        # resolved to — the frozen proof that an alias lowers to the target's
        # full logical identity and not to its own spelling.
        _module, imports, _types = compile_module_from_source(consumer_source)
        import_cases.append({
            "case_id": case["case_id"],
            "file": case["file"],
            "requires": requires,
            "resolved_imports": [
                {"alias": alias, "specification_id": spec}
                for alias, spec in sorted(imports.items())
            ],
            "type_identities": {
                schema["name"]: sorted(_body_identities(schema["body"]))
                for schema in types_by_module[consumer_spec]
            },
        })
        print(f"cross-module fixture {case['case_id']} agrees with single-module bytes")

    import_golden = {
        "kind": "agentflow.conformance.afidl.import-golden",
        "note": (
            "Cross-module import conformance: the resolvable import table of "
            "each fixture and the full logical identities its declarations "
            "reference. Import aliases are source spelling only and never "
            "appear here; the module-set compilation of every fixture must "
            "reproduce the canonical bytes frozen in schema-golden.json."
        ),
        "cases": import_cases,
    }
    write_json(CONF / "import-golden.json", import_golden)

    # Cross-module recursion: the cycle fixtures must be rejected, and mutual
    # imports without a type dependency must be accepted.
    cycle_dir = CONF / "import-cycles"
    for case_id, files in (
        ("cross-module-direct-cycle", ["cycle-a", "cycle-b"]),
        ("cross-module-indirect-cycle", ["ring-a", "ring-b", "ring-c"]),
    ):
        sources = [(f"{name}.afidl", (cycle_dir / f"{name}.afidl").read_text(encoding="utf-8"))
                   for name in files]
        try:
            compile_module_set(sources)
        except ModuleSetError as error:
            assert "cross-module recursive type" in str(error), (
                f"{case_id}: expected a recursion rejection, got {error}")
            print(f"{case_id}: rejected as expected")
        else:
            raise AssertionError(f"{case_id}: cycle was accepted")
    free_sources = [
        (f"{name}.afidl", (cycle_dir / f"{name}.afidl").read_text(encoding="utf-8"))
        for name in ("free-a", "free-b")
    ]
    modules, _ = compile_module_set(free_sources)
    assert len(modules) == 2, "mutual imports without a type cycle must compile"
    print("mutual imports without a type cycle compile")

    # Unqualified references to imported types are rejected, and the local-only
    # (universe-free) path reports the plain unresolved-reference error.
    for fixture in ("qualified-ref-without-import", "unknown-import-alias"):
        source = (CONF / "parser-invalid" / f"{fixture}.afidl").read_text(encoding="utf-8")
        try:
            parse_afidl(source)
        except AfidfSyntaxError as error:
            raise AssertionError(f"{fixture} must parse, got {error}") from error
    print("qualified-reference fixtures parse and fail at validation, not at parse")

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

    # Self-check: the registry schema module must expose the closed registry
    # object set (spec/registry/registry-spec.md).
    reg = next(c for c in cases if c["case_id"] == "specification-registry")
    reg_names = {t["name"] for t in reg["types"]}
    for expected in (
        "AbiNumberAssignment", "AbiNumberKind", "AbiNumberStatus",
        "ArtifactFileDigest", "ConformanceCaseId", "DigestRef", "LogicalPath",
        "NormativeRuleDescriptor", "RequirementLevel", "RuleArea", "RuleId",
        "Sha256Hex", "SpecificationArtifactRef", "SpecificationIdText",
        "SpecificationKind", "SpecificationRegistryEntry", "SpecificationStatus",
        "SourceAnchor", "SymbolicName", "TypeName",
    ):
        assert expected in reg_names, f"registry schema is missing {expected}"
    assert len(reg["types"]) == 20, "registry schema type count drifted"
    print("specification-registry.afidl exposes the closed registry object set")

    # Self-check: the semantic core module must expose the closed Phase 3A
    # object set and must not name any machine-only primitive.
    sem = next(c for c in cases if c["case_id"] == "semantic-core")
    sem_names = {t["name"] for t in sem["types"]}
    for expected in (
        "AndBody", "DigestRefValue", "EnumValueBody", "EqualsBody", "ExpressionBody",
        "ExpressionDef", "ExpressionId", "FieldGetBody", "IfThenElseBody",
        "ListConstructBody", "MapConstructBody", "MapEntryExpr", "NotBody",
        "OptionNoneBody", "OptionSomeBody", "OrBody", "PrimitiveConstant",
        "RecordConstructBody", "RecordFieldExpr", "ResourceSlot", "ResourceSlotId",
        "SemanticPrimitiveType", "SemanticTypeId", "SemanticTypeNode",
        "SemanticTypeNodeBody", "TypeSchemaRef", "ValueSlot", "ValueSlotId",
        "VariantConstructBody",
    ):
        assert expected in sem_names, f"semantic core schema is missing {expected}"
    assert len(sem["types"]) == 29, "semantic core type count drifted"
    for machine_only in ("int32", "uint32", "uint64", "float64"):
        assert machine_only not in sem_names, (
            f"semantic core must not declare {machine_only}")
    # The closed primitive enum must be exactly the 9 workflow primitives.
    primitive_enum = next(
        d for d in parse_afidl(
            semantic_file.read_text(encoding="utf-8"))["declarations"]
        if d["kind"] == "enum" and d["name"] == "SemanticPrimitiveType")
    assert [v["name"] for v in primitive_enum["values"]] == [
        "unit", "null", "bool", "int64", "decimal", "string", "bytes",
        "timestamp", "duration",
    ], "workflow primitive universe drifted"
    # Structural primitive audit: the only primitive keywords this module may
    # mention are the 9 workflow primitives plus uint32 for local id and
    # numbering fields. int32/uint64/float64 must appear nowhere.
    used = set()

    def walk(expr):
        if expr[0] == "primitive":
            used.add(expr[1])
        elif expr[0] in ("list", "map_string", "option"):
            walk(expr[1])

    for declaration in parse_afidl(
            semantic_file.read_text(encoding="utf-8"))["declarations"]:
        if declaration["kind"] == "alias":
            walk(declaration["target"])
        elif declaration["kind"] == "enum":
            continue
        else:
            members = declaration.get("fields") or declaration.get("cases") or []
            for member in members:
                walk(member["type"])
    expected_primitives = {
        "unit", "null", "bool", "int64", "decimal", "string", "bytes",
        "timestamp", "duration", "uint32",
    }
    assert used == expected_primitives, (
        f"semantic core primitive audit failed: {sorted(used)}")
    print("semantic-core.afidl primitive audit clean")

    # Self-check: the semantic IR module must expose the frozen 13-opcode
    # table, no forbidden opcode, the core import, and only machine primitives
    # that are ids/numbering (uint32), strings or durations.
    ir = next(c for c in cases if c["case_id"] == "semantic-ir")
    ir_names = {t["name"] for t in ir["types"]}
    for expected in (
        "AwaitBody", "CallSubflowBody", "ComputationBody", "ComputationDef",
        "CompensationScopeBody", "ContractRefEntry", "ContractRefKind",
        "FailureMatch", "ForeachBody", "HandleBody", "HandleClause",
        "HandleMatch", "InputBinding", "InvokeBody", "IterateBody",
        "LoopCarryBinding", "ParallelBody", "ParallelBranch", "ParallelMode",
        "ParallelOutputBinding", "RaceBody", "RaceBranch", "ResourceUseBinding",
        "RetryBody", "ScopeBody", "SelectBody", "SelectCase",
        "SemanticIrDocument", "SemanticVersionRef", "SequenceBody",
        "TimeoutBody", "WhileBody", "WorkflowOutputDecl", "RunParameterDecl",
        "ComputationOutputBinding",
    ):
        assert expected in ir_names, f"semantic IR schema is missing {expected}"
    ir_module = parse_afidl(ir_file.read_text(encoding="utf-8"))
    body_variant = next(d for d in ir_module["declarations"]
                        if d["kind"] == "variant" and d["name"] == "ComputationBody")
    assert [(c["no"], c["name"]) for c in body_variant["cases"]] == OPCODE_TABLE, (
        "the frozen 13-opcode table drifted")
    forbidden_opcodes = {"agent", "model", "tool", "http", "sql", "shell",
                         "human", "mcp", "a2a", "trigger", "call_composite",
                         "cancelled"}
    assert not ({name for _, name in OPCODE_TABLE} & forbidden_opcodes)
    assert ir_module["imports"] == [{
        "spec": {"namespace": "agentflow.semantic.core", "major": 1},
        "alias": "core", "line": ir_module["imports"][0]["line"],
    }], "the semantic IR module must import exactly agentflow.semantic.core@1 as core"
    document_record = next(d for d in ir_module["declarations"]
                           if d["kind"] == "record" and d["name"] == "SemanticIrDocument")
    assert [f["no"] for f in document_record["fields"]] == list(range(1, 11)), (
        "the document field numbering drifted")
    print("semantic-ir.afidl opcode table and structure frozen")


if __name__ == "__main__":
    main()

