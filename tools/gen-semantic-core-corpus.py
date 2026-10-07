#!/usr/bin/env python3
"""Freezes the Phase 3A semantic-core conformance corpus.

This is an INDEPENDENT implementation of:
  * the semantic type DAG resolution and the ACE1 `Shape` derivation
    (spec/semantic/core-semantic-types.md section 3-4);
  * the closed expression type checker (section 8);
  * ACE1 value encoding for the workflow subset (section 7 of
    spec/encoding/ace1-grammar.md), used to freeze golden bytes.

It reuses the independent AFIDL1 parser/compiler from tools/gen-afidl-corpus.py
(that module *is* the second implementation of the schema layer); everything
above the schema layer is written here from scratch. The Rust implementation in
crates/af-semantic must agree byte-for-byte and error-kind-for-error-kind;
tests never generate expected values at runtime.

Run from the repository root:  python tools/gen-semantic-core-corpus.py
"""

import base64
import importlib.util
import json
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CONF = ROOT / "spec" / "conformance" / "semantic-core"
CORPUS_REVISION = 1

# --------------------------------------------------------------------------
# Import the independent AFIDL1 implementation (tools/gen-afidl-corpus.py).
# --------------------------------------------------------------------------

_spec = importlib.util.spec_from_file_location(
    "afidl_corpus", Path(__file__).resolve().parent / "gen-afidl-corpus.py")
afidl = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(afidl)


class SemanticSyntaxError(Exception):
    """Raised whenever a case must be rejected; carries a stable error kind."""

    def __init__(self, kind):
        super().__init__(kind)
        self.kind = kind


WORKFLOW_PRIMITIVES = [
    "unit", "null", "bool", "int64", "decimal", "string", "bytes",
    "timestamp", "duration",
]
MACHINE_ONLY = ["int32", "uint32", "uint64", "float64"]


# --------------------------------------------------------------------------
# Independent schema universe (identity -> canonical body + digest)
# --------------------------------------------------------------------------

class Universe:
    def __init__(self, afidl_path: Path):
        module = afidl.parse_afidl(afidl_path.read_text(encoding="utf-8"))
        self.specification_id = (
            f"{module['spec']['namespace']}@{module['spec']['major']}")
        self.entries = {}
        for declaration in module["declarations"]:
            schema = afidl.canonical_type(self.specification_id, declaration)
            _payload, digest = afidl.type_schema_digest(schema)
            self.entries[schema["name"]] = {
                "schema": schema,
                "digest_hex": digest,
                "identity": f"{self.specification_id}/{schema['name']}",
            }

    def by_identity(self, identity):
        return self.entries.get(identity.rsplit("/", 1)[-1]) if identity.startswith(
            self.specification_id + "/") else None

    def expand_named(self, identity):
        entry = self.by_identity(identity)
        if entry is None:
            raise SemanticSyntaxError("unknown_type_schema")
        kind, payload = entry["schema"]["body"]
        if kind == "alias":
            return self.expand(payload)
        return ("schema", self.specification_id, entry["schema"]["name"],
                entry["digest_hex"])

    def expand(self, expr):
        kind, value = expr
        if kind == "primitive":
            if value not in WORKFLOW_PRIMITIVES:
                raise SemanticSyntaxError("unsupported_workflow_primitive")
            return ("primitive", value)
        if kind == "ref":
            return self.expand_named(value)
        if kind in ("list", "map_string", "option"):
            return (kind, self.expand(value))
        raise AssertionError(f"unknown type expr {kind!r}")

    def resolve_ref(self, reference):
        """Validates a TypeSchemaRef exactly (digest pinned) and returns entry."""
        digest = reference["schema_digest"]
        if digest["algorithm"] != "sha256":
            raise SemanticSyntaxError("invalid_digest_ref")
        if digest["domain"] != "type-schema":
            raise SemanticSyntaxError("invalid_digest_ref")
        hex_value = digest["hex"]
        if len(hex_value) != 64 or any(
                c not in "0123456789abcdef" for c in hex_value):
            raise SemanticSyntaxError("invalid_digest_ref")
        identity = f"{reference['specification_id']}/{reference['type_name']}"
        entry = self.by_identity(identity)
        if entry is None:
            raise SemanticSyntaxError("unknown_type_schema")
        if entry["digest_hex"] != hex_value:
            raise SemanticSyntaxError("type_schema_digest_mismatch")
        return entry

    def field_type(self, entry, field_no):
        kind, payload = entry["schema"]["body"]
        if kind != "record":
            raise SemanticSyntaxError("expected_record_schema")
        for field in payload["fields"]:
            if field["no"] == field_no:
                return self.expand(field["type"])
        raise SemanticSyntaxError("unknown_field_number")

    def case_payload_type(self, entry, case_no):
        kind, payload = entry["schema"]["body"]
        if kind != "variant":
            raise SemanticSyntaxError("expected_variant_schema")
        for case in payload["cases"]:
            if case["no"] == case_no:
                return self.expand(case["type"])
        raise SemanticSyntaxError("unknown_variant_case")


def structural_string(structural):
    kind = structural[0]
    if kind == "primitive":
        return f"primitive:{structural[1]}"
    if kind == "schema":
        return f"schema:{structural[1]}/{structural[2]}@{structural[3]}"
    return f"{kind}:{structural_string(structural[1])}"


# --------------------------------------------------------------------------
# Independent semantic checker (mirrors the frozen specification)
# --------------------------------------------------------------------------

class Checker:
    def __init__(self, universe, document):
        self.universe = universe
        self.document = document
        self.resolved = {}
        self.slot_types = {}
        self.expr_types = {}
        self.expressions = {e["id"]: e for e in document["expressions"]}

    # -- ids and the type DAG -------------------------------------------

    def check_types(self):
        by_id = {}
        for node in self.document["types"]:
            if node["id"] <= 0:
                raise SemanticSyntaxError("invalid_local_id")
            if node["id"] in by_id:
                raise SemanticSyntaxError("duplicate_semantic_type_id")
            by_id[node["id"]] = node
        for node_id in sorted(by_id):
            self.resolve_type(by_id, node_id, [])

    def resolve_type(self, by_id, node_id, stack):
        if node_id in self.resolved:
            return self.resolved[node_id]
        if node_id in stack:
            raise SemanticSyntaxError("semantic_type_cycle")
        node = by_id.get(node_id)
        if node is None:
            raise SemanticSyntaxError("unknown_semantic_type")
        body = node["body"]
        case, payload = next(iter(body.items()))
        if case == "primitive":
            if payload not in WORKFLOW_PRIMITIVES:
                raise SemanticSyntaxError("unsupported_workflow_primitive")
            structural = ("primitive", payload)
        elif case == "schema":
            self.universe.resolve_ref(payload)
            structural = self.universe.expand_named(
                f"{payload['specification_id']}/{payload['type_name']}")
        elif case in ("list", "map_string", "option"):
            structural = (case, self.resolve_type(by_id, payload, stack + [node_id]))
        else:
            raise AssertionError(f"unknown type node body {case!r}")
        self.resolved[node_id] = structural
        return structural

    def structural_of_type_id(self, type_id):
        if type_id not in self.resolved:
            raise SemanticSyntaxError("unknown_semantic_type")
        return self.resolved[type_id]

    # -- value slots -----------------------------------------------------

    def check_value_slots(self):
        for slot in self.document["values"]:
            if slot["id"] <= 0:
                raise SemanticSyntaxError("invalid_local_id")
            if slot["id"] in self.slot_types:
                raise SemanticSyntaxError("duplicate_value_slot_id")
            self.slot_types[slot["id"]] = self.structural_of_type_id(slot["type"])

    # -- expressions -----------------------------------------------------

    def check_expressions(self):
        # Duplicate detection (first repeated id in source order).
        seen = set()
        for expression in self.document["expressions"]:
            if expression["id"] in seen:
                raise SemanticSyntaxError("duplicate_expression_id")
            seen.add(expression["id"])
        # Id validity and reference existence, in canonical id order.
        for expression_id in sorted(self.expressions):
            if expression_id <= 0:
                raise SemanticSyntaxError("invalid_local_id")
            for reference in self.references(self.expressions[expression_id]["body"]):
                if reference not in self.expressions:
                    raise SemanticSyntaxError("unknown_expression")
        # Cycles.
        self.find_cycle()
        # Derived types, then declared-vs-derived agreement.
        for expression_id in sorted(self.expressions):
            self.expression_type(expression_id)
        for expression_id in sorted(self.expressions):
            expression = self.expressions[expression_id]
            declared = self.structural_of_type_id(expression["result_type"])
            derived = self.expr_types[expression_id]
            if declared != derived:
                raise SemanticSyntaxError("expression_result_type_mismatch")

    @staticmethod
    def references(body):
        case, payload = next(iter(body.items()))
        if case in ("constant", "value_slot", "enum_value", "option_none"):
            return []
        if case == "record_construct":
            return [field["expr_id"] for field in payload["fields"]]
        if case == "list_construct":
            return list(payload["elements"])
        if case == "map_construct":
            return [entry["value_expr"] for entry in payload["entries"]]
        if case in ("variant_construct", "option_some"):
            return [payload["payload_expr"]]
        if case == "field_get":
            return [payload["record"]]
        if case in ("equals", "and", "or"):
            return [payload["left"], payload["right"]]
        if case == "not":
            return [payload["operand"]]
        if case == "if_then_else":
            return [payload["condition"], payload["then_expr"], payload["else_expr"]]
        raise AssertionError(f"unknown expression body {case!r}")

    def find_cycle(self):
        marks = {}

        def visit(node_id):
            if marks.get(node_id) == "done":
                return False
            if marks.get(node_id) == "progress":
                return True
            marks[node_id] = "progress"
            for reference in self.references(self.expressions[node_id]["body"]):
                if reference in self.expressions and visit(reference):
                    return True
            marks[node_id] = "done"
            return False

        for expression_id in sorted(self.expressions):
            if marks.get(expression_id) is None and visit(expression_id):
                raise SemanticSyntaxError("expression_cycle")

    def expression_type(self, expression_id):
        if expression_id in self.expr_types:
            return self.expr_types[expression_id]
        expression = self.expressions.get(expression_id)
        if expression is None:
            raise SemanticSyntaxError("unknown_expression")
        derived = self.body_type(expression_id, expression["body"])
        self.expr_types[expression_id] = derived
        return derived

    def operand(self, expression_id):
        return self.expression_type(expression_id)

    def require_bool(self, expression_id):
        actual = self.operand(expression_id)
        if actual != ("primitive", "bool"):
            raise SemanticSyntaxError("type_mismatch")

    def require_comparable(self, structural):
        if structural[0] == "primitive":
            return
        if structural[0] == "schema":
            entry = self.universe.by_identity(
                f"{structural[1]}/{structural[2]}")
            if entry is not None and entry["schema"]["body"][0] == "enum":
                return
        raise SemanticSyntaxError("type_not_comparable")

    def body_type(self, self_id, body):
        case, payload = next(iter(body.items()))
        if case == "constant":
            constant_case = next(iter(payload))
            if constant_case not in WORKFLOW_PRIMITIVES:
                raise SemanticSyntaxError("unsupported_workflow_primitive")
            if constant_case == "bytes":
                _decode_base64url(payload["bytes"])
            return ("primitive", constant_case)
        if case == "value_slot":
            if payload not in self.slot_types:
                raise SemanticSyntaxError("unknown_value_slot")
            return self.slot_types[payload]
        if case == "record_construct":
            entry = self.universe.resolve_ref(payload["schema"])
            if entry["schema"]["body"][0] != "record":
                raise SemanticSyntaxError("expected_record_schema")
            declared = [f["no"] for f in entry["schema"]["body"][1]["fields"]]
            seen = set()
            for field in payload["fields"]:
                if field["field_no"] in seen:
                    raise SemanticSyntaxError("duplicate_field_binding")
                seen.add(field["field_no"])
                expected = self.universe.field_type(entry, field["field_no"])
                actual = self.operand(field["expr_id"])
                if expected != actual:
                    raise SemanticSyntaxError("type_mismatch")
            for field_no in declared:
                if field_no not in seen:
                    raise SemanticSyntaxError("missing_field_binding")
            return ("schema", entry["schema"]["schema"],
                    entry["schema"]["name"], entry["digest_hex"])
        if case == "variant_construct":
            entry = self.universe.resolve_ref(payload["schema"])
            expected = self.universe.case_payload_type(entry, payload["case_no"])
            actual = self.operand(payload["payload_expr"])
            if expected != actual:
                raise SemanticSyntaxError("type_mismatch")
            return ("schema", entry["schema"]["schema"],
                    entry["schema"]["name"], entry["digest_hex"])
        if case == "enum_value":
            entry = self.universe.resolve_ref(payload["schema"])
            if entry["schema"]["body"][0] != "enum":
                raise SemanticSyntaxError("expected_enum_schema")
            for value in entry["schema"]["body"][1]["values"]:
                if value["no"] == payload["value_no"]:
                    return ("schema", entry["schema"]["schema"],
                            entry["schema"]["name"], entry["digest_hex"])
            raise SemanticSyntaxError("unknown_enum_value")
        if case == "list_construct":
            element = self.structural_of_type_id(payload["element_type"])
            for element_id in payload["elements"]:
                if self.operand(element_id) != element:
                    raise SemanticSyntaxError("type_mismatch")
            return ("list", element)
        if case == "map_construct":
            value_type = self.structural_of_type_id(payload["value_type"])
            canonical_map_entries(payload["entries"])
            for entry in payload["entries"]:
                if self.operand(entry["value_expr"]) != value_type:
                    raise SemanticSyntaxError("type_mismatch")
            return ("map_string", value_type)
        if case == "option_none":
            return ("option", self.structural_of_type_id(payload["inner_type"]))
        if case == "option_some":
            return ("option", self.operand(payload["payload_expr"]))
        if case == "field_get":
            input_type = self.operand(payload["record"])
            if input_type[0] != "schema":
                raise SemanticSyntaxError("expected_record_schema")
            entry = self.universe.by_identity(f"{input_type[1]}/{input_type[2]}")
            if entry is None or entry["schema"]["body"][0] != "record":
                raise SemanticSyntaxError("expected_record_schema")
            return self.universe.field_type(entry, payload["field_no"])
        if case == "equals":
            left = self.operand(payload["left"])
            right = self.operand(payload["right"])
            if left != right:
                raise SemanticSyntaxError("type_mismatch")
            self.require_comparable(left)
            return ("primitive", "bool")
        if case == "not":
            self.require_bool(payload["operand"])
            return ("primitive", "bool")
        if case == "and" or case == "or":
            self.require_bool(payload["left"])
            self.require_bool(payload["right"])
            return ("primitive", "bool")
        if case == "if_then_else":
            self.require_bool(payload["condition"])
            then_type = self.operand(payload["then_expr"])
            else_type = self.operand(payload["else_expr"])
            if then_type != else_type:
                raise SemanticSyntaxError("type_mismatch")
            return then_type
        raise AssertionError(f"unknown expression body {case!r}")

    # -- resources -------------------------------------------------------

    def check_resources(self):
        seen = set()
        for resource in self.document["resources"]:
            if resource["id"] <= 0:
                raise SemanticSyntaxError("invalid_local_id")
            if resource["id"] in seen:
                raise SemanticSyntaxError("duplicate_resource_slot_id")
            seen.add(resource["id"])
            self.universe.resolve_ref(resource["requirement_type"])

    def run(self):
        self.check_types()
        self.check_value_slots()
        self.check_expressions()
        self.check_resources()
        return self


# --------------------------------------------------------------------------
# Independent ACE1 value encoder for the workflow subset
# --------------------------------------------------------------------------

class OptionSome:
    """A `Some(payload)` option value. `OPTION_NONE` is the distinct sentinel
    for `None`: `Some(null)` and `None` must never collapse."""

    __slots__ = ("payload",)

    def __init__(self, payload):
        self.payload = payload


OPTION_NONE = object()


def nfc(text):
    return unicodedata.normalize("NFC", text)


def decode_base64url(text):
    return _decode_base64url(text)


def _decode_base64url(text):
    if "+" in text or "/" in text or "=" in text:
        raise SemanticSyntaxError("invalid_constant")
    padding = "=" * (-len(text) % 4)
    try:
        raw = base64.urlsafe_b64decode(text + padding)
    except Exception:
        raise SemanticSyntaxError("invalid_constant")
    if base64.urlsafe_b64encode(raw).rstrip(b"=").decode() != text:
        raise SemanticSyntaxError("invalid_constant")
    return raw


def encode_base64url(raw):
    return base64.urlsafe_b64encode(raw).rstrip(b"=").decode()


def canonical_map_entries(entries):
    normalized = []
    for entry in entries:
        key = nfc(entry["key"])
        if any(existing["key"] == key for existing in normalized):
            raise SemanticSyntaxError("duplicate_normalized_map_key")
        normalized.append({"key": key, "value_expr": entry["value_expr"]})
    normalized.sort(key=lambda e: e["key"].encode("utf-8"))
    return normalized


def encode_value(shape, value):
    """ACE1 encoding of a (shape, value) pair, workflow subset."""
    kind = shape[0]
    if kind in ("unit", "null"):
        return "null"
    if kind == "bool":
        return "true" if value else "false"
    if kind in ("int64", "timestamp", "duration"):
        return afidl.escape_string(str(value))
    if kind == "decimal":
        return afidl.escape_string(value)
    if kind == "string":
        return afidl.escape_string(nfc(value))
    if kind == "bytes":
        return afidl.escape_string(encode_base64url(value))
    if kind == "enum":
        assert value in shape[2], f"enum value {value!r} not in declared set"
        return afidl.escape_string(nfc(value))
    if kind == "list":
        return "[" + ",".join(encode_value(shape[1], item) for item in value) + "]"
    if kind == "option":
        if value is OPTION_NONE:
            return '{"none":null}'
        assert isinstance(value, OptionSome), "option values are None or Some"
        return '{"some":' + encode_value(shape[1], value.payload) + "}"
    if kind == "map":
        members = sorted(value, key=lambda kv: nfc(kv[0]).encode("utf-8"))
        for name, _ in members:
            assert nfc(name) == name, "map keys must already be normalized"
        return "{" + ",".join(
            afidl.escape_string(name) + ":" + encode_value(shape[1], item)
            for name, item in members) + "}"
    if kind == "record":
        declared = {name: field_shape for name, field_shape in shape[2]}
        assert len(declared) == len(value), "record fields must be exactly declared"
        members = sorted(value, key=lambda kv: kv[0].encode("utf-8"))
        return "{" + ",".join(
            afidl.escape_string(name) + ":" + encode_value(declared[name], item)
            for name, item in members) + "}"
    if kind == "variant":
        declared = {name: case_shape for name, case_shape in shape[2]}
        case, payload = value
        assert case in declared, f"variant case {case!r} not declared"
        return "{" + afidl.escape_string(nfc(case)) + ":" \
            + encode_value(declared[case], payload) + "}"
    raise AssertionError(f"unknown shape {kind!r}")


def shape_of(universe, structural):
    """Derives the ACE1 shape of a structural type."""
    kind = structural[0]
    if kind == "primitive":
        return (structural[1],)
    if kind == "list":
        return ("list", shape_of(universe, structural[1]))
    if kind == "map_string":
        return ("map", shape_of(universe, structural[1]))
    if kind == "option":
        return ("option", shape_of(universe, structural[1]))
    if kind == "schema":
        entry = universe.by_identity(f"{structural[1]}/{structural[2]}")
        assert entry is not None, "schema entry must exist"
        body_kind, payload = entry["schema"]["body"]
        if body_kind == "alias":
            return shape_of(universe, universe.expand(payload))
        if body_kind == "enum":
            return ("enum", entry["schema"]["name"],
                    [value["name"] for value in payload["values"]])
        if body_kind == "record":
            return ("record", entry["schema"]["name"], [
                (field["name"], shape_of(universe, universe.expand(field["type"])))
                for field in payload["fields"]])
        return ("variant", entry["schema"]["name"], [
            (case["name"], shape_of(universe, universe.expand(case["type"])))
            for case in payload["cases"]])
    raise AssertionError(f"unknown structural type {kind!r}")


def lower(checker, universe, expression_id, bindings):
    """Lowers a well-typed expression to (shape, value) — the Python mirror of
    `CheckedDocument::lower_with`."""
    expression = checker.expressions[expression_id]
    shape, value = lower_body(checker, universe, expression["body"], bindings)
    declared = checker.structural_of_type_id(expression["result_type"])
    assert shape_of(universe, declared) == shape, "lowered shape must match declared type"
    return shape, value


def lower_body(checker, universe, body, bindings):
    case, payload = next(iter(body.items()))
    if case == "constant":
        constant_case, literal = next(iter(payload.items()))
        return (constant_case,), constant_value(constant_case, literal)
    if case == "value_slot":
        if payload not in bindings:
            raise SemanticSyntaxError("value_slot_not_bound")
        structural = checker.slot_types[payload]
        return shape_of(universe, structural), bindings[payload]
    if case == "record_construct":
        entry = universe.resolve_ref(payload["schema"])
        fields = entry["schema"]["body"][1]["fields"]
        shape = shape_of(universe, ("schema", entry["schema"]["schema"],
                                    entry["schema"]["name"], entry["digest_hex"]))
        members = []
        for field in sorted(payload["fields"], key=lambda f: f["field_no"]):
            index = next(i for i, f in enumerate(fields)
                         if f["no"] == field["field_no"])
            _, value = lower(checker, universe, field["expr_id"], bindings)
            members.append((shape[2][index][0], value))
        return shape, members
    if case == "variant_construct":
        entry = universe.resolve_ref(payload["schema"])
        shape = shape_of(universe, ("schema", entry["schema"]["schema"],
                                    entry["schema"]["name"], entry["digest_hex"]))
        cases = entry["schema"]["body"][1]["cases"]
        case_name = next(c["name"] for c in cases if c["no"] == payload["case_no"])
        _, value = lower(checker, universe, payload["payload_expr"], bindings)
        return shape, (case_name, value)
    if case == "enum_value":
        entry = universe.resolve_ref(payload["schema"])
        shape = shape_of(universe, ("schema", entry["schema"]["schema"],
                                    entry["schema"]["name"], entry["digest_hex"]))
        values = entry["schema"]["body"][1]["values"]
        name = next(v["name"] for v in values if v["no"] == payload["value_no"])
        return shape, name
    if case == "list_construct":
        element_shape = shape_of(
            universe, checker.structural_of_type_id(payload["element_type"]))
        items = [lower(checker, universe, e, bindings)[1] for e in payload["elements"]]
        return ("list", element_shape), items
    if case == "map_construct":
        value_shape = shape_of(
            universe, checker.structural_of_type_id(payload["value_type"]))
        entries = [(e["key"], lower(checker, universe, e["value_expr"], bindings)[1])
                   for e in canonical_map_entries(payload["entries"])]
        return ("map", value_shape), entries
    if case == "option_none":
        inner_shape = shape_of(
            universe, checker.structural_of_type_id(payload["inner_type"]))
        return ("option", inner_shape), OPTION_NONE
    if case == "option_some":
        inner_shape, value = lower(checker, universe, payload["payload_expr"], bindings)
        return ("option", inner_shape), OptionSome(value)
    if case == "field_get":
        input_type = checker.expression_type(payload["record"])
        assert input_type[0] == "schema", "field_get input must be a record schema"
        entry = universe.by_identity(f"{input_type[1]}/{input_type[2]}")
        declared = entry["schema"]["body"][1]["fields"]
        index = next(i for i, f in enumerate(declared)
                     if f["no"] == payload["field_no"])
        input_shape, input_value = lower(checker, universe, payload["record"], bindings)
        # shape field order mirrors the canonical declaration order.
        return input_shape[2][index][1], dict(input_value)[input_shape[2][index][0]]
    if case == "equals":
        _, left_value = lower(checker, universe, payload["left"], bindings)
        _, right_value = lower(checker, universe, payload["right"], bindings)
        return ("bool",), left_value == right_value
    if case == "not":
        _, value = lower(checker, universe, payload["operand"], bindings)
        return ("bool",), not value
    if case == "and":
        _, left = lower(checker, universe, payload["left"], bindings)
        _, right = lower(checker, universe, payload["right"], bindings)
        return ("bool",), left and right
    if case == "or":
        _, left = lower(checker, universe, payload["left"], bindings)
        _, right = lower(checker, universe, payload["right"], bindings)
        return ("bool",), left or right
    if case == "if_then_else":
        _, condition = lower(checker, universe, payload["condition"], bindings)
        branch = payload["then_expr"] if condition else payload["else_expr"]
        return lower(checker, universe, branch, bindings)
    raise AssertionError(f"unknown expression body {case!r}")


def constant_value(constant_case, literal):
    if constant_case in ("unit", "null"):
        return None
    if constant_case == "bytes":
        return _decode_base64url(literal)
    if constant_case in ("decimal", "string"):
        return literal
    return literal


def encode_case(document, expression_id, bindings):
    """Checks a document and returns (derived_type, ace1_bytes)."""
    checker = Checker(UNIVERSE, document).run()
    derived = checker.expression_type(expression_id)
    shape, value = lower(checker, UNIVERSE, expression_id, bindings)
    payload = encode_value(shape, value).encode("utf-8")
    return structural_string(derived), payload


# --------------------------------------------------------------------------
# Fixture schema universe
# --------------------------------------------------------------------------

UNIVERSE_FILE = CONF / "schema-universe.afidl"
UNIVERSE = None  # set in main()


def ref(type_name):
    entry = UNIVERSE.entries[type_name]
    return {
        "specification_id": UNIVERSE.specification_id,
        "type_name": type_name,
        "schema_digest": {
            "algorithm": "sha256",
            "domain": "type-schema",
            "hex": entry["digest_hex"],
        },
    }


def t_prim(node_id, keyword):
    return {"id": node_id, "body": {"primitive": keyword}}


def t_schema(node_id, type_name):
    return {"id": node_id, "body": {"schema": ref(type_name)}}


def t_list(node_id, inner):
    return {"id": node_id, "body": {"list": inner}}


def t_map(node_id, inner):
    return {"id": node_id, "body": {"map_string": inner}}


def t_option(node_id, inner):
    return {"id": node_id, "body": {"option": inner}}


# The standard type table shared by most cases (ids 1..25).
def standard_types():
    return [
        t_prim(1, "string"),
        t_prim(2, "int64"),
        t_prim(3, "bool"),
        t_prim(4, "decimal"),
        t_prim(5, "bytes"),
        t_prim(6, "timestamp"),
        t_prim(7, "duration"),
        t_prim(8, "unit"),
        t_prim(9, "null"),
        t_schema(10, "User"),
        t_schema(11, "Status"),
        t_schema(12, "LookupResult"),
        t_schema(13, "Container"),
        t_list(14, 1),
        t_map(15, 1),
        t_option(16, 1),
        t_option(17, 2),
        t_list(18, 19),
        t_list(19, 2),
        t_option(20, 21),
        t_list(21, 22),
        t_map(22, 1),
        t_schema(23, "UserId"),
        t_schema(24, "Metrics"),
        t_option(25, 10),
        t_map(26, 2),
    ]


def expr(expression_id, result_type, body):
    return {"id": expression_id, "result_type": result_type, "body": body}


def constant(kind, literal):
    return {"constant": {kind: literal}}


def document(types=None, values=None, expressions=None, resources=None):
    return {
        "types": standard_types() if types is None else types,
        "values": [] if values is None else values,
        "expressions": [] if expressions is None else expressions,
        "resources": [] if resources is None else resources,
    }


def user_record_expressions():
    """expr 1: name string; expr 2: status enum; expr 3: User record."""
    return [
        expr(1, 1, constant("string", "Ada")),
        expr(2, 11, {"enum_value": {"schema": ref("Status"), "value_no": 2}}),
        expr(3, 10, {"record_construct": {
            "schema": ref("User"),
            "fields": [
                {"field_no": 1, "expr_id": 1},
                {"field_no": 2, "expr_id": 1},
                {"field_no": 3, "expr_id": 2},
            ],
        }}),
    ]


GOLDEN = []


def golden(case_id, document_json, expression_id, bindings=None, note=None):
    GOLDEN.append({
        "case_id": case_id,
        "document": document_json,
        "expression_id": expression_id,
        "value_bindings": bindings or [],
        **({"note": note} if note else {}),
    })


NEGATIVE = []


def negative(case_id, document_json, error_kind, note=None, rejected_at="check"):
    NEGATIVE.append({
        "case_id": case_id,
        "document": document_json,
        "error_kind": error_kind,
        "rejected_at": rejected_at,
        **({"note": note} if note else {}),
    })


# --------------------------------------------------------------------------
# Case construction
# --------------------------------------------------------------------------

def build_golden():
    # 1..9: every primitive semantic type as a constant.
    primitive_literals = [
        ("unit", 8, None), ("null", 9, None), ("bool", 3, True),
        ("int64", 2, 5), ("decimal", 4, "1.25"), ("string", 1, "Ada"),
        ("bytes", 5, "aGk"), ("timestamp", 6, 1700000000000000),
        ("duration", 7, -1),
    ]
    for kind, type_id, literal in primitive_literals:
        golden(
            f"constant/{kind}",
            document(expressions=[expr(1, type_id, constant(kind, literal))]),
            1,
        )

    # 10: nested list/map/option with canonical key normalization.
    golden(
        "type/nested-list-map-option",
        document(expressions=[
            expr(1, 20, {"option_some": {"payload_expr": 2}}),
            expr(2, 21, {"list_construct": {
                "element_type": 22,
                "elements": [3, 4],
            }}),
            expr(3, 22, {"map_construct": {
                "value_type": 1,
                "entries": [
                    {"key": "z", "value_expr": 5},
                    {"key": "a", "value_expr": 5},
                ],
            }}),
            expr(4, 22, {"map_construct": {"value_type": 1, "entries": []}}),
            expr(5, 1, constant("string", "v")),
        ]),
        1,
        note="outer option, inner list of maps; keys are emitted in "
             "unsigned UTF-8 byte order, not source order",
    )

    # 11: schema type reference (record construct).
    golden("type/schema-ref", document(expressions=user_record_expressions()), 3)

    # 12: value slot read.
    golden(
        "expr/value-slot",
        document(
            values=[{"id": 1, "type": 1}],
            expressions=[expr(1, 1, {"value_slot": 1})],
        ),
        1,
        bindings=[{"slot": 1, "constant": {"string": "bound"}}],
        note="Phase 3A has no producers, so the harness binds the slot",
    )

    # 13: record construct (explicit case id for the corpus checklist).
    golden("expr/record-construct", document(expressions=user_record_expressions()), 3)

    # 14: variant construct, record payload.
    golden(
        "expr/variant-construct",
        document(expressions=user_record_expressions() + [
            expr(4, 12, {"variant_construct": {
                "schema": ref("LookupResult"),
                "case_no": 1,
                "payload_expr": 3,
            }}),
        ]),
        4,
    )

    # 15: variant construct, unit payload.
    golden(
        "expr/variant-construct-unit",
        document(expressions=[
            expr(1, 8, constant("unit", None)),
            expr(2, 12, {"variant_construct": {
                "schema": ref("LookupResult"),
                "case_no": 2,
                "payload_expr": 1,
            }}),
        ]),
        2,
    )

    # 16: enum value.
    golden(
        "expr/enum-value",
        document(expressions=[expr(
            1, 11, {"enum_value": {"schema": ref("Status"), "value_no": 2}})]),
        1,
    )

    # 17: list construct + shared subexpression (DAG reuse).
    golden(
        "expr/list-construct",
        document(expressions=[
            expr(1, 2, constant("int64", 5)),
            expr(2, 19, {"list_construct": {"element_type": 2, "elements": [1, 1, 3]}}),
            expr(3, 2, constant("int64", 7)),
        ]),
        2,
        note="expression 1 is referenced twice: shared subexpressions are not copied",
    )

    # 18: map construct with a decomposed (non-NFC) key.
    golden(
        "expr/map-construct-nfc-key",
        document(expressions=[
            expr(1, 1, constant("string", "B")),
            expr(2, 15, {"map_construct": {
                "value_type": 1,
                "entries": [{"key": "e\u0301", "value_expr": 1}],
            }}),
        ]),
        2,
        note="the decomposed key is NFC-normalized in the canonical bytes",
    )

    # 19: map construct, empty.
    golden(
        "expr/map-construct-empty",
        document(expressions=[
            expr(1, 15, {"map_construct": {"value_type": 1, "entries": []}}),
        ]),
        1,
    )

    # 20/21: option none / some.
    golden(
        "expr/option-none",
        document(expressions=[expr(1, 16, {"option_none": {"inner_type": 1}})]),
        1,
    )
    golden(
        "expr/option-some",
        document(expressions=[
            expr(1, 2, constant("int64", 42)),
            expr(2, 17, {"option_some": {"payload_expr": 1}}),
        ]),
        2,
    )

    # 22/23: field_get, by field_no only.
    golden(
        "expr/field-get",
        document(expressions=user_record_expressions() + [
            expr(4, 1, {"field_get": {"record": 3, "field_no": 2}}),
        ]),
        4,
    )
    golden(
        "expr/field-get-container-map",
        document(expressions=[
            expr(1, 1, constant("string", "x")),
            expr(2, 2, constant("int64", 3)),
            # Container field 1: list<string>
            expr(3, 14, {"list_construct": {"element_type": 1, "elements": [1]}}),
            # Container field 2: map<string,int64>
            expr(4, 26, {"map_construct": {
                "value_type": 2, "entries": [{"key": "k", "value_expr": 2}]}}),
            # Container field 3: option<string>
            expr(5, 16, {"option_none": {"inner_type": 1}}),
            # Container field 4: list<list<int64>>
            expr(6, 19, {"list_construct": {"element_type": 2, "elements": []}}),
            expr(7, 18, {"list_construct": {"element_type": 19, "elements": [6]}}),
            expr(8, 13, {"record_construct": {
                "schema": ref("Container"),
                "fields": [
                    {"field_no": 1, "expr_id": 3},
                    {"field_no": 2, "expr_id": 4},
                    {"field_no": 3, "expr_id": 5},
                    {"field_no": 4, "expr_id": 7},
                ],
            }}),
            expr(9, 26, {"field_get": {"record": 8, "field_no": 2}}),
        ]),
        9,
        note="Container is a nested list/map/option record; the selected field 2 "
             "is map<string,int64>, re-encoded as a canonical object",
    )

    # 24/25: equality of primitives and of enum values.
    golden(
        "expr/equals-primitive",
        document(expressions=[
            expr(1, 1, constant("string", "a")),
            expr(2, 3, {"equals": {"left": 1, "right": 1}}),
        ]),
        2,
    )
    golden(
        "expr/equals-enum",
        document(expressions=[
            expr(1, 11, {"enum_value": {"schema": ref("Status"), "value_no": 2}}),
            expr(2, 11, {"enum_value": {"schema": ref("Status"), "value_no": 2}}),
            expr(3, 3, {"equals": {"left": 1, "right": 2}}),
        ]),
        3,
    )

    # 26..28: bool connectives.
    golden(
        "expr/not",
        document(expressions=[
            expr(1, 3, constant("bool", False)),
            expr(2, 3, {"not": {"operand": 1}}),
        ]),
        2,
    )
    golden(
        "expr/and",
        document(expressions=[
            expr(1, 3, constant("bool", True)),
            expr(2, 3, constant("bool", False)),
            expr(3, 3, {"and": {"left": 1, "right": 2}}),
        ]),
        3,
    )
    golden(
        "expr/or",
        document(expressions=[
            expr(1, 3, constant("bool", True)),
            expr(2, 3, constant("bool", False)),
            expr(3, 3, {"or": {"left": 1, "right": 2}}),
        ]),
        3,
    )

    # 29/30: if_then_else, both branches (a pure value expression).
    for case_id, flag, expected in [
        ("expr/if-then-else-then", True, "Ada"),
        ("expr/if-then-else-else", False, "Bob"),
    ]:
        golden(
            case_id,
            document(expressions=[
                expr(1, 3, constant("bool", flag)),
                expr(2, 1, constant("string", "Ada")),
                expr(3, 1, constant("string", "Bob")),
                expr(4, 1, {"if_then_else": {
                    "condition": 1, "then_expr": 2, "else_expr": 3}}),
            ]),
            4,
            note=f"expected branch: {expected}",
        )

    # 31: canonical ordering — expressions declared in reverse order.
    sorted_case = document(expressions=[
        expr(1, 2, constant("int64", 5)),
        expr(2, 19, {"list_construct": {"element_type": 2, "elements": [1]}}),
    ])
    shuffled_case = document(expressions=[
        expr(2, 19, {"list_construct": {"element_type": 2, "elements": [1]}}),
        expr(1, 2, constant("int64", 5)),
    ])
    golden("canonical/expressions-sorted", sorted_case, 2)
    golden("canonical/expressions-shuffled", shuffled_case, 2)

    # 32: canonical ordering — type nodes declared in a different order.
    order_types = [t_prim(1, "string"), t_prim(2, "int64"), t_list(3, 2)]
    shuffled_types = [t_list(3, 2), t_prim(2, "int64"), t_prim(1, "string")]
    order_expressions = [
        expr(1, 2, constant("int64", 9)),
        expr(2, 3, {"list_construct": {"element_type": 2, "elements": [1]}}),
    ]
    golden(
        "canonical/types-sorted",
        {"types": order_types, "values": [], "expressions": order_exprs(order_expressions),
         "resources": []},
        2,
    )
    golden(
        "canonical/types-shuffled",
        {"types": shuffled_types, "values": [], "expressions": order_exprs(order_expressions),
         "resources": []},
        2,
    )


def order_exprs(expressions):
    return list(reversed(expressions))


def build_negative():
    base_user = user_record_expressions()

    # -- machine-only primitives ----------------------------------------
    negative(
        "primitive/machine-only-alias-float64",
        document(types=[t_schema(1, "MachineFloat")], expressions=[]),
        "unsupported_workflow_primitive",
        note="MachineFloat is an alias of float64; the workflow universe is closed",
    )
    negative(
        "primitive/machine-only-alias-uint64",
        document(types=[t_schema(1, "MachineWide")], expressions=[]),
        "unsupported_workflow_primitive",
    )
    negative(
        "primitive/machine-only-alias-int32",
        document(types=[t_schema(1, "MachineSmall")], expressions=[]),
        "unsupported_workflow_primitive",
    )
    negative(
        "primitive/machine-only-record-field",
        document(expressions=[
            expr(1, 24, {"record_construct": {
                "schema": ref("Metrics"),
                "fields": [
                    {"field_no": 1, "expr_id": 2},
                    {"field_no": 2, "expr_id": 2},
                ],
            }}),
            expr(2, 2, constant("int64", 1)),
        ]),
        "unsupported_workflow_primitive",
        note="Metrics.ratio is float64: binding it forces the machine-only "
             "primitive into the workflow layer",
    )

    # -- schema references ----------------------------------------------
    unknown = ref("User")
    unknown["type_name"] = "Ghost"
    negative(
        "schema/unknown",
        document(types=[{"id": 1, "body": {"schema": unknown}}], expressions=[]),
        "unknown_type_schema",
    )
    wrong_digest = ref("User")
    wrong_digest["schema_digest"]["hex"] = UNIVERSE.entries["Status"]["digest_hex"]
    negative(
        "schema/wrong-digest",
        document(types=[{"id": 1, "body": {"schema": wrong_digest}}], expressions=[]),
        "type_schema_digest_mismatch",
    )
    bad_hex = ref("User")
    bad_hex["schema_digest"]["hex"] = "a" * 63
    negative(
        "schema/digest-bad-hex",
        document(types=[{"id": 1, "body": {"schema": bad_hex}}], expressions=[]),
        "invalid_digest_ref",
    )
    bad_algo = ref("User")
    bad_algo["schema_digest"]["algorithm"] = "sha512"
    negative(
        "schema/digest-bad-algorithm",
        document(types=[{"id": 1, "body": {"schema": bad_algo}}], expressions=[]),
        "invalid_digest_ref",
    )
    bad_domain = ref("User")
    bad_domain["schema_digest"]["domain"] = "value"
    negative(
        "schema/digest-wrong-domain",
        document(types=[{"id": 1, "body": {"schema": bad_domain}}], expressions=[]),
        "invalid_digest_ref",
    )

    # -- type DAG --------------------------------------------------------
    negative(
        "type/id-zero",
        document(types=[t_prim(0, "string")], expressions=[]),
        "invalid_local_id",
    )
    negative(
        "type/duplicate-id",
        document(types=[t_prim(1, "string"), t_prim(1, "int64")], expressions=[]),
        "duplicate_semantic_type_id",
    )
    negative(
        "type/unknown-node-reference",
        document(types=[t_list(1, 9)], expressions=[]),
        "unknown_semantic_type",
    )
    negative(
        "type/self-cycle",
        document(types=[t_list(1, 1)], expressions=[]),
        "semantic_type_cycle",
    )
    negative(
        "type/two-node-cycle",
        document(types=[t_list(1, 2), t_option(2, 1)], expressions=[]),
        "semantic_type_cycle",
    )
    negative(
        "type/longer-cycle",
        document(types=[t_list(1, 2), t_option(2, 3), t_map(3, 1)], expressions=[]),
        "semantic_type_cycle",
    )

    # -- value slots -----------------------------------------------------
    negative(
        "value-slot/id-zero",
        document(values=[{"id": 0, "type": 1}], expressions=[]),
        "invalid_local_id",
    )
    negative(
        "value-slot/duplicate-id",
        document(values=[{"id": 1, "type": 1}, {"id": 1, "type": 2}], expressions=[]),
        "duplicate_value_slot_id",
    )
    negative(
        "value-slot/unknown-type",
        document(values=[{"id": 1, "type": 99}], expressions=[]),
        "unknown_semantic_type",
    )
    negative(
        "value-slot/read-unknown-slot",
        document(expressions=[expr(1, 1, {"value_slot": 4})]),
        "unknown_value_slot",
    )

    # -- expressions -----------------------------------------------------
    negative(
        "expression/id-zero",
        document(expressions=[expr(0, 1, constant("string", "x"))]),
        "invalid_local_id",
    )
    negative(
        "expression/duplicate-id",
        document(expressions=[
            expr(1, 1, constant("string", "x")),
            expr(1, 2, constant("int64", 1)),
        ]),
        "duplicate_expression_id",
    )
    negative(
        "expression/unknown-reference",
        document(expressions=[expr(1, 1, {"option_some": {"payload_expr": 8}})]),
        "unknown_expression",
    )
    negative(
        "expression/self-cycle",
        document(expressions=[expr(1, 3, {"not": {"operand": 1}})]),
        "expression_cycle",
    )
    negative(
        "expression/two-node-cycle",
        document(expressions=[
            expr(1, 3, {"not": {"operand": 2}}),
            expr(2, 3, {"not": {"operand": 1}}),
        ]),
        "expression_cycle",
    )
    negative(
        "expression/longer-cycle",
        document(expressions=[
            expr(1, 3, {"and": {"left": 2, "right": 3}}),
            expr(2, 3, {"or": {"left": 3, "right": 3}}),
            expr(3, 3, {"not": {"operand": 1}}),
        ]),
        "expression_cycle",
    )
    negative(
        "expression/result-type-mismatch",
        document(expressions=[expr(1, 17, constant("string", "x"))]),
        "expression_result_type_mismatch",
        note="declared option<int64>, derived string",
    )
    negative(
        "expression/invalid-bytes-constant",
        document(expressions=[expr(1, 5, constant("bytes", "aGk="))]),
        "invalid_constant",
    )

    # -- bool / equality -------------------------------------------------
    negative(
        "bool/not-non-bool",
        document(expressions=[
            expr(1, 3, {"not": {"operand": 2}}),
            expr(2, 1, constant("string", "x")),
        ]),
        "type_mismatch",
    )
    negative(
        "bool/and-non-bool",
        document(expressions=[
            expr(1, 3, {"and": {"left": 2, "right": 2}}),
            expr(2, 2, constant("int64", 1)),
        ]),
        "type_mismatch",
    )
    negative(
        "bool/or-non-bool",
        document(expressions=[
            expr(1, 3, {"or": {"left": 2, "right": 2}}),
            expr(2, 2, constant("int64", 1)),
        ]),
        "type_mismatch",
    )
    negative(
        "equals/type-mismatch",
        document(expressions=[
            expr(1, 3, {"equals": {"left": 2, "right": 3}}),
            expr(2, 1, constant("string", "a")),
            expr(3, 2, constant("int64", 1)),
        ]),
        "type_mismatch",
    )
    negative(
        "equals/not-comparable-record",
        document(expressions=base_user + [
            expr(4, 10, {"record_construct": {
                "schema": ref("User"),
                "fields": [
                    {"field_no": 1, "expr_id": 1},
                    {"field_no": 2, "expr_id": 1},
                    {"field_no": 3, "expr_id": 2},
                ],
            }}),
            expr(5, 3, {"equals": {"left": 4, "right": 4}}),
        ]),
        "type_not_comparable",
    )
    negative(
        "equals/not-comparable-list",
        document(expressions=[
            expr(1, 18, {"list_construct": {"element_type": 2, "elements": []}}),
            expr(2, 3, {"equals": {"left": 1, "right": 1}}),
        ]),
        "type_not_comparable",
    )

    # -- if_then_else ----------------------------------------------------
    negative(
        "if/non-bool-condition",
        document(expressions=[
            expr(1, 1, {"if_then_else": {"condition": 2, "then_expr": 3, "else_expr": 3}}),
            expr(2, 2, constant("int64", 1)),
            expr(3, 1, constant("string", "x")),
        ]),
        "type_mismatch",
    )
    negative(
        "if/branch-type-mismatch",
        document(expressions=[
            expr(1, 1, {"if_then_else": {"condition": 2, "then_expr": 3, "else_expr": 4}}),
            expr(2, 3, constant("bool", True)),
            expr(3, 1, constant("string", "x")),
            expr(4, 2, constant("int64", 1)),
        ]),
        "type_mismatch",
    )

    # -- record construct ------------------------------------------------
    negative(
        "record/target-not-record",
        document(expressions=[expr(1, 11, {"record_construct": {
            "schema": ref("Status"), "fields": [],
        }})]),
        "expected_record_schema",
    )
    negative(
        "record/unknown-field-no",
        document(expressions=base_user + [expr(4, 10, {"record_construct": {
            "schema": ref("User"),
            "fields": [
                {"field_no": 1, "expr_id": 1},
                {"field_no": 2, "expr_id": 1},
                {"field_no": 9, "expr_id": 2},
            ],
        }})]),
        "unknown_field_number",
    )
    negative(
        "record/duplicate-field",
        document(expressions=base_user + [expr(4, 10, {"record_construct": {
            "schema": ref("User"),
            "fields": [
                {"field_no": 1, "expr_id": 1},
                {"field_no": 1, "expr_id": 1},
                {"field_no": 2, "expr_id": 1},
                {"field_no": 3, "expr_id": 2},
            ],
        }})]),
        "duplicate_field_binding",
    )
    negative(
        "record/missing-field",
        document(expressions=base_user + [expr(4, 10, {"record_construct": {
            "schema": ref("User"),
            "fields": [
                {"field_no": 1, "expr_id": 1},
                {"field_no": 2, "expr_id": 1},
            ],
        }})]),
        "missing_field_binding",
    )
    negative(
        "record/field-type-mismatch",
        document(expressions=base_user + [expr(4, 10, {"record_construct": {
            "schema": ref("User"),
            "fields": [
                {"field_no": 1, "expr_id": 1},
                {"field_no": 2, "expr_id": 1},
                {"field_no": 3, "expr_id": 1},
            ],
        }})]),
        "type_mismatch",
        note="field 3 is Status, bound with a string",
    )

    # -- variant construct -----------------------------------------------
    negative(
        "variant/target-not-variant",
        document(expressions=[
            expr(1, 8, constant("unit", None)),
            expr(2, 10, {"variant_construct": {
                "schema": ref("User"), "case_no": 1, "payload_expr": 1}}),
        ]),
        "expected_variant_schema",
    )
    negative(
        "variant/unknown-case",
        document(expressions=[
            expr(1, 8, constant("unit", None)),
            expr(2, 12, {"variant_construct": {
                "schema": ref("LookupResult"), "case_no": 7, "payload_expr": 1}}),
        ]),
        "unknown_variant_case",
    )
    negative(
        "variant/payload-mismatch",
        document(expressions=[
            expr(1, 1, constant("string", "x")),
            expr(2, 12, {"variant_construct": {
                "schema": ref("LookupResult"), "case_no": 2, "payload_expr": 1}}),
        ]),
        "type_mismatch",
        note="case 2 is unit, bound with a string",
    )

    # -- enum ------------------------------------------------------------
    negative(
        "enum/target-not-enum",
        document(expressions=[expr(1, 1, {"enum_value": {
            "schema": ref("User"), "value_no": 1}})]),
        "expected_enum_schema",
    )
    negative(
        "enum/unknown-value",
        document(expressions=[expr(1, 11, {"enum_value": {
            "schema": ref("Status"), "value_no": 5}})]),
        "unknown_enum_value",
    )

    # -- field_get -------------------------------------------------------
    negative(
        "field-get/target-not-record",
        document(expressions=[
            expr(1, 2, constant("int64", 1)),
            expr(2, 2, {"field_get": {"record": 1, "field_no": 1}}),
        ]),
        "expected_record_schema",
    )
    negative(
        "field-get/unknown-field-no",
        document(expressions=base_user + [
            expr(4, 10, {"record_construct": {
                "schema": ref("User"),
                "fields": [
                    {"field_no": 1, "expr_id": 1},
                    {"field_no": 2, "expr_id": 1},
                    {"field_no": 3, "expr_id": 2},
                ],
            }}),
            expr(5, 1, {"field_get": {"record": 4, "field_no": 8}}),
        ]),
        "unknown_field_number",
    )

    # -- list / map ------------------------------------------------------
    negative(
        "list/element-type-mismatch",
        document(expressions=[
            expr(1, 18, {"list_construct": {"element_type": 2, "elements": [2]}}),
            expr(2, 1, constant("string", "x")),
        ]),
        "type_mismatch",
    )
    negative(
        "map/duplicate-nfc-key",
        document(expressions=[
            expr(1, 1, constant("string", "x")),
            expr(2, 15, {"map_construct": {
                "value_type": 1,
                "entries": [
                    {"key": "e\u0301", "value_expr": 1},
                    {"key": "\u00e9", "value_expr": 1},
                ],
            }}),
        ]),
        "duplicate_normalized_map_key",
    )
    negative(
        "map/value-type-mismatch",
        document(expressions=[
            expr(1, 2, constant("int64", 1)),
            expr(2, 15, {"map_construct": {
                "value_type": 1,
                "entries": [{"key": "k", "value_expr": 1}],
            }}),
        ]),
        "type_mismatch",
    )

    # -- option_some derivation ------------------------------------------
    negative(
        "option-some/derivation-mismatch",
        document(expressions=[
            expr(1, 1, constant("string", "x")),
            expr(2, 17, {"option_some": {"payload_expr": 1}}),
        ]),
        "expression_result_type_mismatch",
        note="option_some of a string derives option<string>, not option<int64>",
    )

    # -- resources -------------------------------------------------------
    negative(
        "resource/id-zero",
        document(resources=[{"id": 0, "requirement_type": ref("User")}]),
        "invalid_local_id",
    )
    negative(
        "resource/duplicate-id",
        document(resources=[
            {"id": 1, "requirement_type": ref("User")},
            {"id": 1, "requirement_type": ref("Status")},
        ]),
        "duplicate_resource_slot_id",
    )
    negative(
        "resource/unknown-requirement-schema",
        document(resources=[{"id": 1, "requirement_type": unknown}]),
        "unknown_type_schema",
    )

    # -- constant payload ------------------------------------------------
    negative(
        "constant/machine-only-case",
        document(expressions=[expr(1, 2, constant("float64", 1.0))]),
        "unsupported_workflow_primitive",
        rejected_at="deserialize",
        note="PrimitiveConstant is closed to the 9 workflow primitives, so a "
             "machine-only constant case is unrepresentable in the typed model: "
             "rejection happens when the document is parsed",
    )


# --------------------------------------------------------------------------
# Emission
# --------------------------------------------------------------------------

def write_json(path, payload):
    text = json.dumps(payload, indent=2, ensure_ascii=False, sort_keys=True) + "\n"
    path.write_text(text, encoding="utf-8", newline="\n")
    print(f"wrote {path.relative_to(ROOT)}")


def main():
    global UNIVERSE
    UNIVERSE = Universe(UNIVERSE_FILE)
    build_golden()
    build_negative()

    cases = []
    for case in GOLDEN:
        bindings = {}
        for binding in case["value_bindings"]:
            constant_case, literal = next(iter(binding["constant"].items()))
            bindings[binding["slot"]] = constant_value(constant_case, literal)
        derived, payload = encode_case(
            case["document"], case["expression_id"], bindings)
        cases.append({
            "case_id": case["case_id"],
            "expression_id": case["expression_id"],
            "derived_type": derived,
            "ace1_utf8": payload.decode("utf-8"),
            "ace1_hex": payload.hex(),
            "value_bindings": case["value_bindings"],
            "document": case["document"],
            **({"note": case["note"]} if "note" in case else {}),
        })

    negatives = []
    for case in NEGATIVE:
        checker = Checker(UNIVERSE, case["document"])
        try:
            checker.run()
            # Constant payload validation happens inside the type checker for
            # well-formed bodies; the machine-only constant case is caught there
            # too, so reaching this line means the case no longer fails.
            raise AssertionError(f"{case['case_id']} unexpectedly validated")
        except SemanticSyntaxError as error:
            assert error.kind == case["error_kind"], (
                f"{case['case_id']}: expected {case['error_kind']}, got {error.kind}")
        negatives.append({
            "case_id": case["case_id"],
            "error_kind": case["error_kind"],
            "rejected_at": case["rejected_at"],
            "document": case["document"],
            **({"note": case["note"]} if "note" in case else {}),
        })

    # Equivalence groups: cases that must produce identical canonical output.
    groups = []
    by_id = {case["case_id"]: case for case in cases}
    for left, right in [
        ("canonical/expressions-sorted", "canonical/expressions-shuffled"),
        ("canonical/types-sorted", "canonical/types-shuffled"),
    ]:
        assert by_id[left]["ace1_hex"] == by_id[right]["ace1_hex"], (
            f"{left} and {right} must have identical canonical bytes")
        assert by_id[left]["derived_type"] == by_id[right]["derived_type"]
        groups.append([left, right])

    write_json(CONF / "semantic-core-golden.json", {
        "kind": "agentflow.conformance.semantic-core",
        "corpus_revision": CORPUS_REVISION,
        "note": (
            "Frozen semantic-core golden and negative cases. Golden cases carry "
            "a full SemanticCoreDocument (types/values/expressions/resources), "
            "one expression id and the canonical ACE1 bytes of its lowered "
            "value; expected bytes, derived types and negative error kinds were "
            "produced by the independent Python implementation "
            "(tools/gen-semantic-core-corpus.py) and must never be regenerated "
            "by the Rust implementation under test."
        ),
        "fixture_schema": {
            "file": "spec/conformance/semantic-core/schema-universe.afidl",
            "specification_id": UNIVERSE.specification_id,
            "types": [
                {"name": name, "digest_hex": entry["digest_hex"]}
                for name, entry in sorted(UNIVERSE.entries.items())
            ],
        },
        "golden": cases,
        "negative": negatives,
        "equivalence_groups": groups,
    })
    print(f"golden cases: {len(cases)}, negative cases: {len(negatives)}, "
          f"equivalence groups: {len(groups)}")


if __name__ == "__main__":
    main()
