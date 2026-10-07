#!/usr/bin/env python3
"""Freezes the Phase 3B semantic-ir conformance corpus.

INDEPENDENT implementation of the Canonical Semantic IR rules
(spec/semantic/semantic-ir.md) and of the SemanticIrDigest formula:
SemanticIrDigest = SHA256("AF\\0semantic-ir\\0" || ACE1(canonical document)).
The Rust implementation in crates/af-ir must agree with the frozen
expectations; expected values are never derived from the Rust code.

Run from the repository root:  python tools/gen-semantic-ir-corpus.py
"""

import copy
import hashlib
import importlib.util
import json
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CONF = ROOT / "spec" / "conformance" / "semantic-ir"
CORPUS_REVISION = 1

# The independent AFIDL1 implementation (Phase 2A/2C) supplies the lexer,
# parser and canonical-form builder used to derive the ACE1 shapes.
_spec = importlib.util.spec_from_file_location(
    "afidl_corpus", Path(__file__).resolve().parent / "gen-afidl-corpus.py")
afidl = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(afidl)

SEMANTIC_CORE = ROOT / "spec" / "idl" / "semantic-core.afidl"
SEMANTIC_IR = ROOT / "spec" / "idl" / "semantic-ir.afidl"
FIXTURE_SCHEMA = ROOT / "spec" / "conformance" / "semantic-core" / "schema-universe.afidl"

# The frozen Phase 1 domains this corpus pins (never any new domain).
COMPONENT_DIGEST = "a1" * 32
SUBFLOW_DIGEST = "b2" * 32


# ---------------------------------------------------------------------------
# Module set: canonical forms and cross-module shape derivation
# ---------------------------------------------------------------------------

def load_module_set():
    sources = [
        ("semantic-core.afidl", SEMANTIC_CORE.read_text(encoding="utf-8")),
        ("semantic-ir.afidl", SEMANTIC_IR.read_text(encoding="utf-8")),
    ]
    modules, types_by_module = afidl.compile_module_set(sources)
    types_by_identity = {}
    for spec, types in types_by_module.items():
        for schema in types:
            types_by_identity[f"{spec}/{schema['name']}"] = schema
    return types_by_identity


TYPES_BY_IDENTITY = load_module_set()

PRIM_SHAPES = {
    "unit": ("unit",), "null": ("null",), "bool": ("bool",),
    "int32": ("int32",), "uint32": ("uint32",), "int64": ("int64",),
    "uint64": ("uint64",), "decimal": ("decimal",), "float64": ("float64",),
    "string": ("string",), "bytes": ("bytes",), "timestamp": ("timestamp",),
    "duration": ("duration",),
}


def shape_of_identity(identity, _stack=None):
    schema = TYPES_BY_IDENTITY[identity]
    return body_shape(identity, schema["body"])


def body_shape(identity, body):
    kind, payload = body
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
        return PRIM_SHAPES[value]
    if kind == "ref":
        return shape_of_identity(value)
    if kind == "list":
        return ("list", expr_shape(value))
    if kind == "map_string":
        return ("map", expr_shape(value))
    if kind == "option":
        return ("option", expr_shape(value))
    raise AssertionError(f"unknown type expr {kind!r}")


DOCUMENT_SHAPE = shape_of_identity("agentflow.semantic.ir@1/SemanticIrDocument")


# ---------------------------------------------------------------------------
# ACE1 encoding of the canonical document value tree
# ---------------------------------------------------------------------------

class OptionSome:
    """A `Some(payload)` option value; OPTION_NONE is the distinct None."""

    def __init__(self, payload):
        self.payload = payload


OPTION_NONE = object()


def nfc(text):
    return unicodedata.normalize("NFC", text)


def encode_value(shape, value):
    kind = shape[0]
    if kind in ("unit", "null"):
        return "null"
    if kind == "bool":
        return "true" if value else "false"
    if kind in ("int32", "uint32", "int64", "uint64", "timestamp", "duration"):
        return afidl.escape_string(str(value))
    if kind == "decimal":
        return afidl.escape_string(value)
    if kind == "string":
        return afidl.escape_string(nfc(value))
    if kind == "bytes":
        raise AssertionError("bytes not used by the IR corpus")
    if kind == "enum":
        assert value in shape[2], f"enum value {value!r} not in declared set"
        return afidl.escape_string(nfc(value))
    if kind == "list":
        return "[" + ",".join(encode_value(shape[1], item) for item in value) + "]"
    if kind == "option":
        if value is None:
            return '{"none":null}'
        return '{"some":' + encode_value(shape[1], value) + "}"
    if kind == "map":
        raise AssertionError("maps not used by the IR corpus")
    if kind == "record":
        declared = dict(shape[2])
        assert set(declared) == set(value), "record fields must be exactly declared"
        members = sorted(value, key=lambda key: key.encode("utf-8"))
        return "{" + ",".join(
            afidl.escape_string(name) + ":" + encode_value(declared[name], value[name])
            for name in members) + "}"
    if kind == "variant":
        declared = dict(shape[2])
        assert len(value) == 1, "a variant is a single-case JSON object"
        case, payload = next(iter(value.items()))
        assert case in declared, f"variant case {case!r} not declared"
        return "{" + afidl.escape_string(nfc(case)) + ":" \
            + encode_value(declared[case], payload) + "}"
    raise AssertionError(f"unknown shape {kind!r}")


def json_to_value(shape, json_value):
    """Converts the serde-form document (corpus JSON) into the ACE1 value
    tree, driven entirely by the schema shape."""
    kind = shape[0]
    if kind in ("unit", "null"):
        return None
    if kind == "bool":
        return json_value
    if kind in ("int32", "uint32", "int64", "uint64", "timestamp", "duration"):
        return int(json_value)
    if kind in ("decimal", "string"):
        return json_value
    if kind == "enum":
        return json_value
    if kind == "list":
        return [json_to_value(shape[1], item) for item in json_value]
    if kind == "option":
        if json_value is None:
            return None
        return json_to_value(shape[1], json_value)
    if kind == "record":
        return {name: json_to_value(field_shape, json_value[name])
                for name, field_shape in shape[2]}
    if kind == "variant":
        case, payload = next(iter(json_value.items()))
        case_shape = dict(shape[2])[case]
        return {case: json_to_value(case_shape, payload)}
    raise AssertionError(f"unknown shape {kind!r}")


def canonicalize(document):
    """The frozen canonical ordering (spec/semantic/semantic-ir.md section 11).

    Sequence children and retry delays are semantic order and are preserved.
    """
    document = copy.deepcopy(document)
    document["semantic_refs"].sort(
        key=lambda ref: ref["specification_id"].encode("utf-8"))
    for table, key in (
        ("types", "id"), ("values", "id"), ("expressions", "id"),
        ("resources", "id"), ("contracts", "id"), ("run_parameters", "id"),
        ("computations", "id"),
    ):
        document[table].sort(key=lambda entry: entry[key])
    document["workflow_outputs"].sort(
        key=lambda output: nfc(output["name"]).encode("utf-8"))
    for computation in document["computations"]:
        computation["outputs"].sort(key=lambda binding: binding["port_no"])
        body = computation["body"]
        opcode, payload = next(iter(body.items()))
        if opcode == "invoke" or opcode == "call_subflow":
            payload["inputs"].sort(key=lambda binding: binding["port_no"])
            payload["resources"].sort(key=lambda binding: binding["port_no"])
        elif opcode == "select":
            payload["cases"].sort(key=lambda case: case["ordinal"])
        elif opcode == "parallel":
            payload["branches"].sort(key=lambda branch: branch["ordinal"])
            payload["output_bindings"].sort(
                key=lambda binding: binding["parent_port_no"])
        elif opcode == "race":
            payload["branches"].sort(key=lambda branch: branch["ordinal"])
        elif opcode == "iterate":
            mode, loop = next(iter(payload.items()))
            loop["carry"].sort(key=lambda binding: binding["slot"])
        elif opcode == "handle":
            payload["clauses"].sort(key=lambda clause: clause["ordinal"])
        elif opcode == "scope":
            payload["resources"].sort()
    return document


def document_digest(document):
    canonical = canonicalize(document)
    value = json_to_value(DOCUMENT_SHAPE, canonical)
    text = encode_value(DOCUMENT_SHAPE, value)
    payload = text.encode("utf-8")
    digest = hashlib.sha256(b"AF\x00semantic-ir\x00" + payload).hexdigest()
    return payload, digest


# ---------------------------------------------------------------------------
# Fixture schema digests (independent, from the shared Phase 3A fixture)
# ---------------------------------------------------------------------------

def fixture_digests():
    sources = [("schema-universe.afidl", FIXTURE_SCHEMA.read_text(encoding="utf-8"))]
    _modules, types_by_module = afidl.compile_module_set(sources)
    out = {}
    for spec, types in types_by_module.items():
        for schema in types:
            _payload, digest = afidl.type_schema_digest(schema)
            out[schema["name"]] = digest
    return out


FIXTURE_DIGESTS = fixture_digests()


def schema_ref(type_name):
    return {
        "specification_id": "agentflow.test.semantic@1",
        "type_name": type_name,
        "schema_digest": {
            "algorithm": "sha256",
            "domain": "type-schema",
            "hex": FIXTURE_DIGESTS[type_name],
        },
    }


USER_REF = schema_ref("User")
STATUS_REF = schema_ref("Status")


# ---------------------------------------------------------------------------
# Document construction helpers (serde/corpus JSON form)
# ---------------------------------------------------------------------------

def doc(semantic_refs, values, expressions, resources, contracts,
        run_parameters, workflow_outputs, computations, root,
        types=None):
    return {
        "semantic_refs": semantic_refs,
        "types": PRIMITIVE_TYPES if types is None else types,
        "values": values,
        "expressions": expressions,
        "resources": resources,
        "contracts": contracts,
        "run_parameters": run_parameters,
        "workflow_outputs": workflow_outputs,
        "computations": computations,
        "root": root,
    }


PRIMITIVE_TYPES = [
    {"id": 1, "body": {"primitive": "int64"}},
    {"id": 2, "body": {"primitive": "bool"}},
    {"id": 3, "body": {"primitive": "string"}},
    {"id": 4, "body": {"primitive": "duration"}},
    {"id": 5, "body": {"primitive": "unit"}},
    {"id": 6, "body": {"primitive": "null"}},
    {"id": 7, "body": {"list": 3}},
    {"id": 8, "body": {"option": 1}},
]

LIST_STRING = 7


def slot(id_, type_):
    return {"id": id_, "type": type_}


def expr(id_, result_type, body):
    return {"id": id_, "result_type": result_type, "body": body}


def rp(id_, name, type_, value_slot):
    return {"id": id_, "name": name, "type": type_, "value_slot": value_slot}


def comp(id_, outputs, body):
    return {"id": id_, "outputs": outputs, "body": body}


def obind(port, value_slot):
    return {"port_no": port, "value_slot": value_slot}


def ibind(port, expression):
    return {"port_no": port, "expression": expression}


def rbind(port, resource_slot):
    return {"port_no": port, "resource_slot": resource_slot}


def outb(port, branch, child_port):
    return {"parent_port_no": port, "branch_ordinal": branch,
            "child_port_no": child_port}


def resource(id_, requirement):
    return {"id": id_, "requirement_type": requirement}


CONTRACT_DOMAINS = {"component": "component-contract", "subflow": "semantic-ir"}


def contract(id_, kind, hex_):
    return {"id": id_, "kind": kind,
            "digest": {"algorithm": "sha256", "domain": CONTRACT_DOMAINS[kind],
                       "hex": hex_}}


def component_contract(id_=1):
    return contract(id_, "component", COMPONENT_DIGEST)


def subflow_contract(id_=2):
    return contract(id_, "subflow", SUBFLOW_DIGEST)


def await_body(signal, correlation):
    return {"await": {"signal_type": signal, "correlation": correlation}}


AWAIT_USER = await_body(USER_REF, 1)

# The shared run-parameter table: produces every base slot exactly once.
BASE_RUN_PARAMETERS = [
    rp(1, "limit", 1, 1),
    rp(2, "flag", 2, 2),
    rp(3, "delay", 4, 3),
    rp(4, "label", 3, 4),
    rp(5, "items", LIST_STRING, 5),
]

BASE_VALUES = [
    slot(1, 1), slot(2, 2), slot(3, 4), slot(4, 3), slot(5, LIST_STRING),
]

BASE_EXPRESSIONS = [
    expr(1, 1, {"value_slot": 1}),                       # int64 reader
    expr(2, 2, {"value_slot": 2}),                       # bool reader
    expr(3, 4, {"value_slot": 3}),                       # duration reader
    expr(4, 3, {"value_slot": 4}),                       # string reader
    expr(5, 4, {"constant": {"duration": 5000000}}),     # 5s constant
    expr(6, 1, {"constant": {"int64": 2}}),              # selector constant
    expr(7, 2, {"constant": {"bool": True}}),
    expr(8, 3, {"constant": {"string": "payload"}}),
    expr(9, LIST_STRING, {"value_slot": 5}),             # list reader
]

BASE_RESOURCES = [resource(1, USER_REF), resource(2, STATUS_REF)]


def base_document(computations, root, contracts=None, values=None,
                  run_parameters=None, workflow_outputs=None):
    return doc(
        semantic_refs=[],
        values=BASE_VALUES if values is None else values,
        expressions=BASE_EXPRESSIONS,
        resources=BASE_RESOURCES,
        contracts=[] if contracts is None else contracts,
        run_parameters=BASE_RUN_PARAMETERS if run_parameters is None else run_parameters,
        workflow_outputs=[{"name": "done", "expression": 2}]
        if workflow_outputs is None else workflow_outputs,
        computations=computations,
        root=root,
    )


# ---------------------------------------------------------------------------
# Golden documents — at least one per opcode plus the nested compositions
# ---------------------------------------------------------------------------

def goldens():
    out = []

    # invoke/minimal ---------------------------------------------------------
    out.append(("invoke/minimal", base_document(
        computations=[comp(
            1, [obind(1, 6)],
            {"invoke": {"contract": 1,
                        "inputs": [ibind(1, 1)],
                        "resources": [rbind(1, 1)]}})],
        root=1,
        contracts=[component_contract()],
        values=BASE_VALUES + [slot(6, 1)],
    )))

    # await/minimal ----------------------------------------------------------
    out.append(("await/minimal", base_document(
        computations=[comp(1, [], AWAIT_USER)],
        root=1,
    )))

    # sequence/minimal -------------------------------------------------------
    out.append(("sequence/minimal", base_document(
        computations=[
            comp(1, [], {"sequence": {"children": [2]}}),
            comp(2, [], AWAIT_USER),
        ],
        root=1,
    )))

    # select/minimal ---------------------------------------------------------
    out.append(("select/minimal", base_document(
        computations=[
            comp(1, [], {"select": {"selector": 6, "cases": [
                {"ordinal": 1, "child": 2},
                {"ordinal": 2, "child": 3},
            ]}}),
            comp(2, [], AWAIT_USER),
            comp(3, [], AWAIT_USER),
        ],
        root=1,
    )))

    # parallel/all -----------------------------------------------------------
    out.append(("parallel/all", base_document(
        computations=[
            comp(1, [], {"parallel": {
                "mode": "all",
                "branches": [{"ordinal": 1, "child": 2},
                             {"ordinal": 2, "child": 3}],
                "max_inflight": None,
                "output_bindings": [],
            }}),
            comp(2, [], AWAIT_USER),
            comp(3, [], AWAIT_USER),
        ],
        root=1,
    )))

    # parallel/fail-fast -----------------------------------------------------
    # The parent port maps the branch child's output: two distinct SSA slots
    # of the same semantic type (slot 6 parent, slot 7 branch child).
    out.append(("parallel/fail-fast", base_document(
        computations=[
            comp(1, [obind(1, 6)], {"parallel": {
                "mode": "fail_fast",
                "branches": [{"ordinal": 1, "child": 2},
                             {"ordinal": 2, "child": 3}],
                "max_inflight": 4,
                "output_bindings": [outb(1, 1, 1)],
            }}),
            comp(2, [obind(1, 7)], AWAIT_USER),
            comp(3, [], AWAIT_USER),
        ],
        root=1,
        values=BASE_VALUES + [slot(6, 1), slot(7, 1)],
    )))

    # race/minimal -----------------------------------------------------------
    out.append(("race/minimal", base_document(
        computations=[
            comp(1, [], {"race": {"branches": [
                {"ordinal": 1, "child": 2},
                {"ordinal": 2, "child": 3},
            ]}}),
            comp(2, [], AWAIT_USER),
            comp(3, [], AWAIT_USER),
        ],
        root=1,
    )))

    # iterate/foreach --------------------------------------------------------
    out.append(("iterate/foreach", base_document(
        computations=[
            comp(1, [], {"iterate": {"foreach": {
                "source": 9,
                "item_slot": 6,
                "body": 2,
                "carry": [],
                "max_inflight": None,
            }}}),
            comp(2, [], AWAIT_USER),
        ],
        root=1,
        values=BASE_VALUES + [slot(6, 3)],
    )))

    # iterate/while ----------------------------------------------------------
    out.append(("iterate/while", base_document(
        computations=[
            comp(1, [], {"iterate": {"while": {
                "guard": 2,
                "body": 2,
                "carry": [{"slot": 6, "initial": 1, "body_output_port": 1}],
                "max_iterations": 10,
            }}}),
            comp(2, [obind(1, 7)], AWAIT_USER),
        ],
        root=1,
        values=BASE_VALUES + [slot(6, 1), slot(7, 1)],
    )))

    # retry/minimal ----------------------------------------------------------
    out.append(("retry/minimal", base_document(
        computations=[
            comp(1, [], {"retry": {"child": 2, "delays": [0, 5000000]}}),
            comp(2, [], AWAIT_USER),
        ],
        root=1,
    )))

    # timeout/minimal --------------------------------------------------------
    out.append(("timeout/minimal", base_document(
        computations=[
            comp(1, [], {"timeout": {"child": 2, "timeout": 5}}),
            comp(2, [], AWAIT_USER),
        ],
        root=1,
    )))

    # handle/failure ---------------------------------------------------------
    out.append(("handle/failure", base_document(
        computations=[
            comp(1, [], {"handle": {"child": 2, "clauses": [
                {"ordinal": 1,
                 "match": {"failure": {"failure_type": USER_REF}},
                 "handler": 3},
            ]}}),
            comp(2, [], AWAIT_USER),
            comp(3, [], AWAIT_USER),
        ],
        root=1,
    )))

    # handle/timed-out -------------------------------------------------------
    out.append(("handle/timed-out", base_document(
        computations=[
            comp(1, [], {"handle": {"child": 2, "clauses": [
                {"ordinal": 1, "match": {"timed_out": None}, "handler": 3},
                {"ordinal": 2,
                 "match": {"failure": {"failure_type": USER_REF}},
                 "handler": 4},
            ]}}),
            comp(2, [], AWAIT_USER),
            comp(3, [], AWAIT_USER),
            comp(4, [], AWAIT_USER),
        ],
        root=1,
    )))

    # scope/minimal ----------------------------------------------------------
    out.append(("scope/minimal", base_document(
        computations=[
            comp(1, [], {"scope": {"child": 2, "resources": [1, 2]}}),
            comp(2, [], AWAIT_USER),
        ],
        root=1,
    )))

    # compensation-scope/minimal --------------------------------------------
    out.append(("compensation-scope/minimal", base_document(
        computations=[
            comp(1, [], {"compensation_scope": {"child": 2}}),
            comp(2, [], AWAIT_USER),
        ],
        root=1,
    )))

    # call-subflow/minimal ---------------------------------------------------
    out.append(("call-subflow/minimal", doc(
        semantic_refs=[{"specification_id": "agentflow.semantic.core@1"}],
        values=BASE_VALUES + [slot(6, 1)],
        expressions=BASE_EXPRESSIONS,
        resources=BASE_RESOURCES,
        contracts=[component_contract(), subflow_contract()],
        run_parameters=BASE_RUN_PARAMETERS,
        workflow_outputs=[{"name": "done", "expression": 2}],
        computations=[comp(
            1, [obind(1, 6)],
            {"call_subflow": {"contract": 2,
                              "inputs": [ibind(1, 1)],
                              "resources": [rbind(1, 1)]}})],
        root=1,
    )))

    # nested compositions ----------------------------------------------------
    out.append(("nested/sequence-invoke-await", base_document(
        computations=[
            comp(1, [], {"sequence": {"children": [2, 3]}}),
            comp(2, [obind(1, 6)],
                 {"invoke": {"contract": 1, "inputs": [], "resources": []}}),
            comp(3, [], AWAIT_USER),
        ],
        root=1,
        contracts=[component_contract()],
        values=BASE_VALUES + [slot(6, 1)],
    )))

    out.append(("nested/retry-timeout-invoke", base_document(
        computations=[
            comp(1, [obind(1, 8)], {"retry": {"child": 2, "delays": [0]}}),
            comp(2, [obind(1, 7)], {"timeout": {"child": 3, "timeout": 5}}),
            comp(3, [obind(1, 6)],
                 {"invoke": {"contract": 1, "inputs": [], "resources": []}}),
        ],
        root=1,
        contracts=[component_contract()],
        values=BASE_VALUES + [slot(6, 1), slot(7, 1), slot(8, 1)],
    )))

    out.append(("nested/scope-parallel", base_document(
        computations=[
            comp(1, [], {"scope": {"child": 2, "resources": [1]}}),
            comp(2, [], {"parallel": {
                "mode": "all",
                "branches": [{"ordinal": 1, "child": 3},
                             {"ordinal": 2, "child": 4}],
                "max_inflight": 2,
                "output_bindings": [],
            }}),
            comp(3, [], AWAIT_USER),
            comp(4, [], AWAIT_USER),
        ],
        root=1,
    )))

    out.append(("nested/compensation-scope-sequence", base_document(
        computations=[
            comp(1, [], {"compensation_scope": {"child": 2}}),
            comp(2, [], {"sequence": {"children": [3, 4]}}),
            comp(3, [obind(1, 6)],
                 {"invoke": {"contract": 1, "inputs": [], "resources": []}}),
            comp(4, [], AWAIT_USER),
        ],
        root=1,
        contracts=[component_contract()],
        values=BASE_VALUES + [slot(6, 1)],
    )))

    out.append(("nested/select-retry-call-subflow", base_document(
        computations=[
            comp(1, [], {"select": {"selector": 6, "cases": [
                {"ordinal": 1, "child": 2},
                {"ordinal": 2, "child": 3},
            ]}}),
            comp(2, [], {"retry": {"child": 4, "delays": [250000]}}),
            comp(3, [], {"call_subflow": {"contract": 2,
                                          "inputs": [], "resources": []}}),
            comp(4, [], AWAIT_USER),
        ],
        root=1,
        contracts=[subflow_contract()],
    )))

    out.append(("nested/while-sequence", base_document(
        computations=[
            comp(1, [], {"iterate": {"while": {
                "guard": 2, "body": 2, "carry": [], "max_iterations": 3,
            }}}),
            comp(2, [], {"sequence": {"children": [3]}}),
            comp(3, [], AWAIT_USER),
        ],
        root=1,
    )))

    return out


GOLDENS = goldens()


# ---------------------------------------------------------------------------
# Independent validation (mirrors spec/semantic/semantic-ir.md, section order)
# ---------------------------------------------------------------------------

OPCODE_TABLE = [
    (1, "invoke"), (2, "await"), (3, "sequence"), (4, "select"),
    (5, "parallel"), (6, "race"), (7, "iterate"), (8, "retry"),
    (9, "timeout"), (10, "handle"), (11, "scope"),
    (12, "compensation_scope"), (13, "call_subflow"),
]


class ValidationError(Exception):
    def __init__(self, kind, detail=""):
        super().__init__(f"{kind}: {detail}")
        self.kind = kind


def body_opcode(computation):
    return next(iter(computation["body"].items()))


def owned_children(computation):
    opcode, payload = body_opcode(computation)
    if opcode in ("invoke", "await", "call_subflow"):
        return []
    if opcode == "sequence":
        return list(payload["children"])
    if opcode == "select":
        return [case["child"] for case in payload["cases"]]
    if opcode == "parallel" or opcode == "race":
        return [branch["child"] for branch in payload["branches"]]
    if opcode == "iterate":
        return [next(iter(payload.values()))["body"]]
    if opcode == "retry" or opcode == "timeout":
        return [payload["child"]]
    if opcode == "handle":
        return [payload["child"]] + [c["handler"] for c in payload["clauses"]]
    if opcode == "scope" or opcode == "compensation_scope":
        return [payload["child"]]
    raise AssertionError(f"unknown opcode {opcode!r}")


def validate(document):
    """Validates an IR document; raises ValidationError with the stable kind.

    The four core tables are assumed to satisfy the Phase 3A rules (the
    golden documents construct them directly; the IR negatives never mutate
    them), so this validator covers the Phase 3B rule set only.
    """
    # 1. semantic refs
    seen_refs = set()
    for reference in document["semantic_refs"]:
        spec = reference["specification_id"]
        if "@" not in spec or spec != spec.strip() or " " in spec \
                or not spec.split("@")[1].isdigit() or int(spec.split("@")[1]) <= 0:
            raise ValidationError("invalid_semantic_ref", spec)
        if spec in seen_refs:
            raise ValidationError("duplicate_semantic_ref", spec)
        seen_refs.add(spec)

    types = {t["id"]: t for t in document["types"]}
    values = {v["id"]: v for v in document["values"]}
    expressions = {e["id"]: e for e in document["expressions"]}
    resources = {r["id"] for r in document["resources"]}
    computations = {c["id"]: c for c in document["computations"]}
    contracts = {c["id"]: c for c in document["contracts"]}

    def require_slot(id_):
        if id_ not in values:
            raise ValidationError("unknown_value_slot", str(id_))
        return values[id_]

    def require_expression(id_):
        if id_ not in expressions:
            raise ValidationError("unknown_expression", str(id_))
        return expressions[id_]

    def require_resource(id_):
        if id_ not in resources:
            raise ValidationError("unknown_resource_slot", str(id_))

    def result_type(id_):
        return require_expression(id_)["result_type"]

    def is_primitive(type_id, keyword):
        body = types[type_id]["body"]
        return "primitive" in body and body["primitive"] == keyword

    def check_name(name, invalid, duplicate, seen):
        if not name:
            raise ValidationError(invalid, "empty")
        if nfc(name) != name:
            raise ValidationError(invalid, name)
        if nfc(name) in seen:
            raise ValidationError(duplicate, name)
        seen.add(nfc(name))

    producers = {}

    def produce(id_, producer):
        if id_ in producers:
            raise ValidationError("duplicate_value_producer",
                                  f"slot {id_} produced twice")
        producers[id_] = producer

    # 2. run parameters
    parameter_ids = set()
    parameter_names = set()
    for parameter in document["run_parameters"]:
        if parameter["id"] <= 0:
            raise ValidationError("invalid_local_id", "run-parameter")
        if parameter["id"] in parameter_ids:
            raise ValidationError("duplicate_run_parameter", str(parameter["id"]))
        parameter_ids.add(parameter["id"])
        check_name(parameter["name"], "invalid_run_parameter_name",
                   "duplicate_run_parameter_name", parameter_names)
        if parameter["type"] not in types:
            raise ValidationError("unknown_semantic_type", str(parameter["type"]))
        slot_ = require_slot(parameter["value_slot"])
        if slot_["type"] != parameter["type"]:
            raise ValidationError("run_parameter_type_mismatch",
                                  str(parameter["id"]))
        produce(parameter["value_slot"], "run_parameter")

    # 3. workflow outputs
    output_names = set()
    for output in document["workflow_outputs"]:
        check_name(output["name"], "invalid_workflow_output_name",
                   "duplicate_workflow_output_name", output_names)
        require_expression(output["expression"])

    # 4. contracts
    contract_ids = set()
    contract_identities = set()
    for entry in document["contracts"]:
        if entry["id"] <= 0:
            raise ValidationError("invalid_local_id", "contract-ref")
        if entry["id"] in contract_ids:
            raise ValidationError("duplicate_contract_ref", str(entry["id"]))
        contract_ids.add(entry["id"])
        digest = entry["digest"]
        if digest["algorithm"] != "sha256" or len(digest["hex"]) != 64 \
                or any(ch not in "0123456789abcdef" for ch in digest["hex"]):
            raise ValidationError("invalid_contract_digest", entry["kind"])
        expected_domain = CONTRACT_DOMAINS[entry["kind"]]
        if digest["domain"] != expected_domain:
            raise ValidationError("contract_digest_domain",
                                  f"{entry['kind']} vs {digest['domain']}")
        identity = (digest["algorithm"], digest["domain"], digest["hex"])
        if identity in contract_identities:
            raise ValidationError("duplicate_contract_identity", digest["hex"])
        contract_identities.add(identity)

    # 5. computations, pass A: ids and outputs
    computation_ids = set()
    for computation in document["computations"]:
        if computation["id"] <= 0:
            raise ValidationError("invalid_local_id", "computation")
        if computation["id"] in computation_ids:
            raise ValidationError("duplicate_computation", str(computation["id"]))
        computation_ids.add(computation["id"])
        ports = set()
        for binding in computation["outputs"]:
            if binding["port_no"] <= 0:
                raise ValidationError("invalid_local_id", "port")
            if binding["port_no"] in ports:
                raise ValidationError("duplicate_output_port",
                                      str(binding["port_no"]))
            ports.add(binding["port_no"])
            require_slot(binding["value_slot"])
            produce(binding["value_slot"], "computation_output")

    def require_computation(id_):
        if id_ not in computations:
            raise ValidationError("unknown_computation", str(id_))

    def check_ordinals(ordinals):
        seen = set()
        for ordinal in ordinals:
            if ordinal <= 0:
                raise ValidationError("invalid_local_id", "branch")
            if ordinal in seen:
                raise ValidationError("duplicate_branch_ordinal", str(ordinal))
            seen.add(ordinal)

    def check_call(opcode, payload):
        contract_id = payload["contract"]
        if contract_id not in contracts:
            kind = (opcode + "_unknown_contract")
            raise ValidationError(kind, str(contract_id))
        expected = "component" if opcode == "invoke" else "subflow"
        if contracts[contract_id]["kind"] != expected:
            raise ValidationError(opcode + "_contract_kind",
                                  contracts[contract_id]["kind"])
        input_ports = set()
        for binding in payload["inputs"]:
            if binding["port_no"] <= 0:
                raise ValidationError("invalid_local_id", "port")
            if binding["port_no"] in input_ports:
                raise ValidationError("duplicate_input_port",
                                      str(binding["port_no"]))
            input_ports.add(binding["port_no"])
            require_expression(binding["expression"])
        resource_ports = set()
        for binding in payload["resources"]:
            if binding["port_no"] <= 0:
                raise ValidationError("invalid_local_id", "port")
            if binding["port_no"] in resource_ports:
                raise ValidationError("duplicate_resource_port",
                                      str(binding["port_no"]))
            resource_ports.add(binding["port_no"])
            require_resource(binding["resource_slot"])

    def check_carry(computation_id, carry, body_id):
        seen_slots = set()
        for binding in carry:
            if binding["slot"] in seen_slots:
                raise ValidationError("duplicate_carry_slot", str(binding["slot"]))
            seen_slots.add(binding["slot"])
            carry_slot = values.get(binding["slot"])
            if carry_slot is None:
                raise ValidationError("carry_unknown_slot", str(binding["slot"]))
            initial_type = result_type(binding["initial"])
            body_computation = computations[body_id]
            body_port = next((o for o in body_computation["outputs"]
                              if o["port_no"] == binding["body_output_port"]), None)
            if body_port is None:
                raise ValidationError("carry_unknown_port",
                                      str(binding["body_output_port"]))
            next_slot = values[body_port["value_slot"]]
            if not (carry_slot["type"] == initial_type == next_slot["type"]):
                raise ValidationError("carry_type_mismatch", str(binding["slot"]))
            produce(binding["slot"], "iterate_carry")

    # 6. computations, pass B: per-body rules in canonical id order
    for computation_id in sorted(computations):
        computation = computations[computation_id]
        for child in owned_children(computation):
            require_computation(child)
        if len(set(owned_children(computation))) != len(owned_children(computation)):
            raise ValidationError("duplicate_child_reference",
                                  str(computation_id))
        opcode, payload = body_opcode(computation)
        if opcode == "invoke" or opcode == "call_subflow":
            check_call(opcode, payload)
        elif opcode == "sequence":
            if not payload["children"]:
                raise ValidationError("sequence_empty", str(computation_id))
        elif opcode == "select":
            if not payload["cases"]:
                raise ValidationError("select_empty", str(computation_id))
            selector_type = result_type(payload["selector"])
            if not is_primitive(selector_type, "int64"):
                raise ValidationError("selector_not_int64", str(selector_type))
            check_ordinals(case["ordinal"] for case in payload["cases"])
        elif opcode == "parallel":
            if not payload["branches"]:
                raise ValidationError("parallel_empty", str(computation_id))
            if payload["max_inflight"] == 0:
                raise ValidationError("invalid_max_inflight", "")
            check_ordinals(branch["ordinal"] for branch in payload["branches"])
            bound_parent_ports = set()
            for binding in payload["output_bindings"]:
                if binding["parent_port_no"] in bound_parent_ports:
                    raise ValidationError("parallel_output_duplicate_port",
                                          str(binding["parent_port_no"]))
                bound_parent_ports.add(binding["parent_port_no"])
                branch = next((b for b in payload["branches"]
                               if b["ordinal"] == binding["branch_ordinal"]), None)
                if branch is None:
                    raise ValidationError("parallel_output_unknown_branch",
                                          str(binding["branch_ordinal"]))
                parent_output = next(
                    (o for o in computation["outputs"]
                     if o["port_no"] == binding["parent_port_no"]), None)
                child_output = next(
                    (o for o in computations[branch["child"]]["outputs"]
                     if o["port_no"] == binding["child_port_no"]), None)
                if child_output is None:
                    raise ValidationError("parallel_output_unknown_port",
                                          str(binding["child_port_no"]))
                parent_slot = values[parent_output["value_slot"]]
                child_slot = values[child_output["value_slot"]]
                if parent_slot["type"] != child_slot["type"]:
                    raise ValidationError("parallel_output_type_mismatch",
                                          str(binding["parent_port_no"]))
        elif opcode == "race":
            if len(payload["branches"]) < 2:
                raise ValidationError("race_insufficient_branches", "")
            check_ordinals(branch["ordinal"] for branch in payload["branches"])
        elif opcode == "iterate":
            mode, loop = next(iter(payload.items()))
            if mode == "foreach":
                if loop["max_inflight"] == 0:
                    raise ValidationError("invalid_max_inflight", "")
                source_type = result_type(loop["source"])
                source_body = types[source_type]["body"]
                if "list" not in source_body:
                    raise ValidationError("foreach_source_not_list",
                                          str(source_type))
                item_slot = require_slot(loop["item_slot"])
                if item_slot["type"] != source_body["list"]:
                    raise ValidationError("foreach_item_type_mismatch",
                                          str(loop["item_slot"]))
                produce(loop["item_slot"], "iterate_item")
            else:
                if loop["max_iterations"] <= 0:
                    raise ValidationError("invalid_max_iterations", "")
                guard_type = result_type(loop["guard"])
                if not is_primitive(guard_type, "bool"):
                    raise ValidationError("while_guard_not_bool",
                                          str(guard_type))
            check_carry(computation_id, loop["carry"], loop["body"])
        elif opcode == "retry":
            for delay in payload["delays"]:
                if delay < 0:
                    raise ValidationError("retry_invalid_delay", str(delay))
        elif opcode == "timeout":
            timeout_type = result_type(payload["timeout"])
            if not is_primitive(timeout_type, "duration"):
                raise ValidationError("timeout_not_duration", str(timeout_type))
        elif opcode == "handle":
            handler_ordinals = set()
            failure_matches = []
            timed_out = 0
            for clause in payload["clauses"]:
                if clause["ordinal"] <= 0:
                    raise ValidationError("invalid_local_id", "handler")
                if clause["ordinal"] in handler_ordinals:
                    raise ValidationError("duplicate_handler_ordinal",
                                          str(clause["ordinal"]))
                handler_ordinals.add(clause["ordinal"])
                match_kind, match_payload = next(iter(clause["match"].items()))
                if match_kind == "failure":
                    if match_payload["failure_type"] in failure_matches:
                        raise ValidationError("duplicate_failure_match",
                                              str(match_payload))
                    failure_matches.append(match_payload["failure_type"])
                else:
                    timed_out += 1
                    if timed_out > 1:
                        raise ValidationError("duplicate_timed_out", "")
        elif opcode == "scope":
            seen_resources = set()
            for resource_id in payload["resources"]:
                if resource_id not in resources:
                    raise ValidationError("scope_unknown_resource",
                                          str(resource_id))
                if resource_id in seen_resources:
                    raise ValidationError("scope_duplicate_resource",
                                          str(resource_id))
                seen_resources.add(resource_id)

    # 7. producer uniqueness
    for slot_id in values:
        if slot_id not in producers:
            raise ValidationError("unbound_value_slot", str(slot_id))

    # 8. ownership
    if document["root"] == 0:
        raise ValidationError("invalid_local_id", "computation")
    if document["root"] not in computations:
        raise ValidationError("unknown_root", str(document["root"]))
    parents = {}
    children_of = {}
    for computation in document["computations"]:
        children_of[computation["id"]] = owned_children(computation)
        for child in owned_children(computation):
            if child in parents and parents[child] != computation["id"]:
                raise ValidationError("multiple_structural_parents",
                                      str(child))
            parents[child] = computation["id"]

    state = {}

    def dfs(node):
        if state.get(node) == 1:
            return True
        if state.get(node) == 2:
            return False
        state[node] = 1
        cyclic = any(dfs(child) for child in children_of.get(node, []))
        state[node] = 2
        return cyclic

    if dfs(document["root"]):
        raise ValidationError("computation_cycle", "")
    if document["root"] in parents:
        raise ValidationError("root_has_parent", str(document["root"]))
    reachable = set()
    stack = [document["root"]]
    while stack:
        current = stack.pop()
        if current in reachable:
            continue
        reachable.add(current)
        stack.extend(children_of.get(current, []))
    for computation_id in computations:
        if computation_id not in reachable:
            raise ValidationError("unreachable_computation",
                                  str(computation_id))

    # 9. wrapper interface equality
    def interface(computation_id):
        return {binding["port_no"]: values[binding["value_slot"]]["type"]
                for binding in computations[computation_id]["outputs"]}

    for computation in document["computations"]:
        mine = interface(computation["id"])
        opcode, payload = body_opcode(computation)
        mismatch = None
        if opcode == "sequence":
            if interface(payload["children"][-1]) != mine:
                mismatch = "sequence_interface_mismatch"
        elif opcode == "select":
            if any(interface(case["child"]) != mine for case in payload["cases"]):
                mismatch = "select_interface_mismatch"
        elif opcode == "race":
            if any(interface(branch["child"]) != mine
                   for branch in payload["branches"]):
                mismatch = "race_interface_mismatch"
        elif opcode == "retry":
            if interface(payload["child"]) != mine:
                mismatch = "retry_interface_mismatch"
        elif opcode == "timeout":
            if interface(payload["child"]) != mine:
                mismatch = "timeout_interface_mismatch"
        elif opcode == "handle":
            owned = [payload["child"]] + [c["handler"] for c in payload["clauses"]]
            if any(interface(owned_id) != mine for owned_id in owned):
                mismatch = "handle_interface_mismatch"
        elif opcode == "scope":
            if interface(payload["child"]) != mine:
                mismatch = "scope_interface_mismatch"
        elif opcode == "compensation_scope":
            if interface(payload["child"]) != mine:
                mismatch = "compensation_scope_interface_mismatch"
        if mismatch:
            raise ValidationError(mismatch, str(computation["id"]))
    return None


# ---------------------------------------------------------------------------
# Negative cases
# ---------------------------------------------------------------------------

def find_computation(document, computation_id):
    return next(c for c in document["computations"] if c["id"] == computation_id)


def await_document():
    """A small single-await document used as the base of most negatives."""
    return copy.deepcopy(next(doc for case_id, doc in GOLDENS
                              if case_id == "sequence/minimal"))


def negative(coro):
    """Runs a mutation callable on a fresh sequence/minimal copy."""
    document = await_document()
    coro(document)
    return document


NEGATIVE_CASES = []


def neg(case_id, expected_kind, mutate, rejected_at="check"):
    """Freezes one negative: `mutate` is applied to a fresh base document."""
    document = await_document()
    mutate(document)
    NEGATIVE_CASES.append({
        "case_id": case_id,
        "document": document,
        "error_kind": expected_kind,
        "rejected_at": rejected_at,
    })


def build_negatives():
    # document structure ----------------------------------------------------
    neg("document/root-zero", "invalid_local_id",
        lambda d: d.__setitem__("root", 0))
    neg("document/unknown-root", "unknown_root",
        lambda d: d.__setitem__("root", 99))

    def duplicate_computation(d):
        d["computations"].append(copy.deepcopy(find_computation(d, 2)))
    neg("document/duplicate-computation-id", "duplicate_computation",
        duplicate_computation)

    def unreachable(d):
        d["computations"].append(comp(9, [], AWAIT_USER))
    neg("document/unreachable-computation", "unreachable_computation",
        unreachable)

    def cycle(d):
        find_computation(d, 2)["body"] = {"sequence": {"children": [1]}}
    neg("document/structural-cycle", "computation_cycle",
        cycle)

    def multiple_parents(d):
        d["computations"].append(comp(
            3, [], {"parallel": {"mode": "all",
                                 "branches": [{"ordinal": 1, "child": 2}],
                                 "max_inflight": None,
                                 "output_bindings": []}}))
        d["computations"].append(comp(4, [], AWAIT_USER))
    neg("document/multiple-structural-parents", "multiple_structural_parents",
        multiple_parents)

    def child_missing(d):
        find_computation(d, 1)["body"]["sequence"]["children"].append(99)
    neg("document/child-missing", "unknown_computation",
        child_missing)

    # semantic refs ---------------------------------------------------------
    def bad_ref(d):
        d["semantic_refs"] = [{"specification_id": "Not A Spec"}]
    neg("semantic-ref/invalid-syntax", "invalid_semantic_ref", bad_ref)

    def dup_ref(d):
        d["semantic_refs"] = [
            {"specification_id": "agentflow.semantic.core@1"},
            {"specification_id": "agentflow.semantic.core@1"},
        ]
    neg("semantic-ref/duplicate", "duplicate_semantic_ref", dup_ref)

    # producers -------------------------------------------------------------
    def unbound(d):
        d["values"].append(slot(9, 1))
    neg("producer/unbound-slot", "unbound_value_slot", unbound)

    def param_and_computation(d):
        find_computation(d, 2)["outputs"] = [obind(1, 1)]
    neg("producer/param-and-computation-same-slot",
        "duplicate_value_producer", param_and_computation)

    def two_outputs(d):
        find_computation(d, 2)["outputs"] = [obind(1, 9)]
        d["computations"].append(comp(3, [obind(1, 9)], AWAIT_USER))
        d["values"].append(slot(9, 2))
    neg("producer/two-computation-outputs", "duplicate_value_producer",
        two_outputs)

    def duplicate_port(d):
        find_computation(d, 2)["outputs"] = [obind(1, 9), obind(1, 8)]
        d["values"].append(slot(8, 2))
        d["values"].append(slot(9, 2))
    neg("producer/duplicate-output-port", "duplicate_output_port",
        duplicate_port)

    # run parameters --------------------------------------------------------
    def param_type_mismatch(d):
        d["run_parameters"][0]["type"] = 2
    neg("run-parameter/type-mismatch", "run_parameter_type_mismatch",
        param_type_mismatch)

    def param_unknown_type(d):
        d["run_parameters"][0]["type"] = 99
    neg("run-parameter/unknown-type", "unknown_semantic_type",
        param_unknown_type)

    def param_unknown_slot(d):
        d["run_parameters"][0]["value_slot"] = 99
    neg("run-parameter/unknown-slot", "unknown_value_slot", param_unknown_slot)

    def param_name_empty(d):
        d["run_parameters"][0]["name"] = ""
    neg("run-parameter/name-empty", "invalid_run_parameter_name",
        param_name_empty)

    def param_name_nfc(d):
        d["run_parameters"][0]["name"] = "cafe\u0301"
    neg("run-parameter/name-not-nfc", "invalid_run_parameter_name",
        param_name_nfc)

    def param_name_duplicate(d):
        d["run_parameters"][1]["name"] = d["run_parameters"][0]["name"]
    neg("run-parameter/name-duplicate", "duplicate_run_parameter_name",
        param_name_duplicate)

    # workflow outputs ------------------------------------------------------
    def output_name_duplicate(d):
        d["workflow_outputs"].append({"name": "done", "expression": 1})
    neg("workflow-output/name-duplicate", "duplicate_workflow_output_name",
        output_name_duplicate)

    def output_name_empty(d):
        d["workflow_outputs"][0]["name"] = ""
    neg("workflow-output/name-empty", "invalid_workflow_output_name",
        output_name_empty)

    def output_unknown_expression(d):
        d["workflow_outputs"][0]["expression"] = 99
    neg("workflow-output/unknown-expression", "unknown_expression",
        output_unknown_expression)

    # contracts -------------------------------------------------------------
    def contract_unknown(d):
        d["contracts"] = [component_contract()]
        find_computation(d, 2)["body"] = {
            "invoke": {"contract": 99, "inputs": [], "resources": []}}
    neg("contract/unknown-id", "invoke_unknown_contract", contract_unknown)

    def invoke_uses_subflow(d):
        d["contracts"] = [subflow_contract(id_=1)]
        find_computation(d, 2)["body"] = {
            "invoke": {"contract": 1, "inputs": [], "resources": []}}
    neg("contract/invoke-uses-subflow", "invoke_contract_kind",
        invoke_uses_subflow)

    def subflow_uses_component(d):
        d["contracts"] = [component_contract(id_=1)]
        find_computation(d, 2)["body"] = {
            "call_subflow": {"contract": 1, "inputs": [], "resources": []}}
    neg("contract/call-subflow-uses-component", "call_subflow_contract_kind",
        subflow_uses_component)

    def component_wrong_domain(d):
        d["contracts"] = [{"id": 1, "kind": "component",
                           "digest": {"algorithm": "sha256",
                                      "domain": "semantic-ir",
                                      "hex": COMPONENT_DIGEST}}]
        find_computation(d, 2)["body"] = {
            "invoke": {"contract": 1, "inputs": [], "resources": []}}
    neg("contract/component-wrong-domain", "contract_digest_domain",
        component_wrong_domain)

    def subflow_wrong_domain(d):
        d["contracts"] = [{"id": 2, "kind": "subflow",
                           "digest": {"algorithm": "sha256",
                                      "domain": "component-contract",
                                      "hex": SUBFLOW_DIGEST}}]
        find_computation(d, 2)["body"] = {
            "call_subflow": {"contract": 2, "inputs": [], "resources": []}}
    neg("contract/subflow-wrong-domain", "contract_digest_domain",
        subflow_wrong_domain)

    def contract_duplicate_id(d):
        d["contracts"] = [component_contract(), component_contract(id_=1)]
    neg("contract/duplicate-id", "duplicate_contract_ref", contract_duplicate_id)

    def contract_duplicate_identity(d):
        d["contracts"] = [component_contract(), component_contract(id_=2)]
    neg("contract/duplicate-identity", "duplicate_contract_identity",
        contract_duplicate_identity)

    # sequence / select -----------------------------------------------------
    def sequence_empty(d):
        find_computation(d, 1)["body"]["sequence"]["children"] = []
    neg("sequence/empty", "sequence_empty", sequence_empty)

    def parent_interface_mismatch(d):
        find_computation(d, 1)["outputs"] = [obind(1, 9)]
        d["values"].append(slot(9, 1))
    neg("sequence/parent-interface-mismatch", "sequence_interface_mismatch",
        parent_interface_mismatch)

    def select_base(d):
        d["computations"] = [
            comp(1, [], {"select": {"selector": 6, "cases": [
                {"ordinal": 1, "child": 2}, {"ordinal": 2, "child": 3}]}}),
            comp(2, [], AWAIT_USER),
            comp(3, [], AWAIT_USER),
        ]
        d["root"] = 1
        return d

    def select_empty(d):
        select_base(d)
        find_computation(d, 1)["body"]["select"]["cases"] = []
    neg("select/empty", "select_empty", select_empty)

    def selector_not_int64(d):
        select_base(d)
        find_computation(d, 1)["body"]["select"]["selector"] = 2
    neg("select/selector-not-int64", "selector_not_int64", selector_not_int64)

    def case_ordinal_zero(d):
        select_base(d)
        find_computation(d, 1)["body"]["select"]["cases"][0]["ordinal"] = 0
    neg("select/case-ordinal-zero", "invalid_local_id", case_ordinal_zero)

    def duplicate_case_ordinal(d):
        select_base(d)
        find_computation(d, 1)["body"]["select"]["cases"][1]["ordinal"] = 1
    neg("select/duplicate-case-ordinal", "duplicate_branch_ordinal",
        duplicate_case_ordinal)

    def branch_interface_mismatch(d):
        select_base(d)
        find_computation(d, 2)["outputs"] = [obind(1, 9)]
        d["values"].append(slot(9, 1))
    neg("select/branch-interface-mismatch", "select_interface_mismatch",
        branch_interface_mismatch)

    # parallel / race -------------------------------------------------------
    def parallel_base(d):
        d["computations"] = [
            comp(1, [], {"parallel": {
                "mode": "all",
                "branches": [{"ordinal": 1, "child": 2},
                             {"ordinal": 2, "child": 3}],
                "max_inflight": None,
                "output_bindings": [],
            }}),
            comp(2, [], AWAIT_USER),
            comp(3, [], AWAIT_USER),
        ]
        d["root"] = 1
        return d

    def parallel_empty(d):
        parallel_base(d)
        find_computation(d, 1)["body"]["parallel"]["branches"] = []
    neg("parallel/empty-branches", "parallel_empty", parallel_empty)

    def parallel_duplicate_ordinal(d):
        parallel_base(d)
        find_computation(d, 1)["body"]["parallel"]["branches"][1]["ordinal"] = 1
    neg("parallel/duplicate-ordinal", "duplicate_branch_ordinal",
        parallel_duplicate_ordinal)

    def parallel_max_inflight_zero(d):
        parallel_base(d)
        find_computation(d, 1)["body"]["parallel"]["max_inflight"] = 0
    neg("parallel/max-inflight-zero", "invalid_max_inflight",
        parallel_max_inflight_zero)

    def parallel_output_unknown_branch(d):
        parallel_base(d)
        find_computation(d, 1)["outputs"] = [obind(1, 9)]
        find_computation(d, 1)["body"]["parallel"]["output_bindings"] = [
            outb(1, 9, 1)]
        d["values"].append(slot(9, 1))
    neg("parallel/output-unknown-branch", "parallel_output_unknown_branch",
        parallel_output_unknown_branch)

    def parallel_output_unknown_child_port(d):
        parallel_base(d)
        find_computation(d, 1)["outputs"] = [obind(1, 9)]
        find_computation(d, 1)["body"]["parallel"]["output_bindings"] = [
            outb(1, 1, 9)]
        d["values"].append(slot(9, 1))
    neg("parallel/output-unknown-child-port", "parallel_output_unknown_port",
        parallel_output_unknown_child_port)

    def parallel_output_type_mismatch(d):
        parallel_base(d)
        find_computation(d, 1)["outputs"] = [obind(1, 9)]
        find_computation(d, 2)["outputs"] = [obind(1, 10)]
        find_computation(d, 1)["body"]["parallel"]["output_bindings"] = [
            outb(1, 1, 1)]
        d["values"].append(slot(9, 1))
        d["values"].append(slot(10, 2))
    neg("parallel/output-type-mismatch", "parallel_output_type_mismatch",
        parallel_output_type_mismatch)

    def race_base(d):
        d["computations"] = [
            comp(1, [], {"race": {"branches": [
                {"ordinal": 1, "child": 2}, {"ordinal": 2, "child": 3}]}}),
            comp(2, [], AWAIT_USER),
            comp(3, [], AWAIT_USER),
        ]
        d["root"] = 1
        return d

    def race_one_branch(d):
        race_base(d)
        find_computation(d, 1)["body"]["race"]["branches"] = [
            {"ordinal": 1, "child": 2}]
    neg("race/one-branch", "race_insufficient_branches", race_one_branch)

    def race_duplicate_ordinal(d):
        race_base(d)
        find_computation(d, 1)["body"]["race"]["branches"][1]["ordinal"] = 1
    neg("race/duplicate-ordinal", "duplicate_branch_ordinal",
        race_duplicate_ordinal)

    def race_interface_mismatch(d):
        race_base(d)
        find_computation(d, 2)["outputs"] = [obind(1, 9)]
        d["values"].append(slot(9, 1))
    neg("race/branch-interface-mismatch", "race_interface_mismatch",
        race_interface_mismatch)

    # iterate ---------------------------------------------------------------
    def foreach_base(d):
        d["computations"] = [
            comp(1, [], {"iterate": {"foreach": {
                "source": 9, "item_slot": 6, "body": 2,
                "carry": [], "max_inflight": None}}}),
            comp(2, [], AWAIT_USER),
        ]
        d["root"] = 1
        d["values"] = BASE_VALUES + [slot(6, 3)]
        return d

    def foreach_source_not_list(d):
        foreach_base(d)
        find_computation(d, 1)["body"]["iterate"]["foreach"]["source"] = 1
    neg("foreach/source-not-list", "foreach_source_not_list",
        foreach_source_not_list)

    def foreach_item_wrong_type(d):
        foreach_base(d)
        d["values"] = BASE_VALUES + [slot(6, 2)]
    neg("foreach/item-slot-wrong-type", "foreach_item_type_mismatch",
        foreach_item_wrong_type)

    def foreach_item_missing(d):
        foreach_base(d)
        d["values"] = BASE_VALUES
    neg("foreach/item-slot-missing", "unknown_value_slot",
        foreach_item_missing)

    def foreach_max_inflight_zero(d):
        foreach_base(d)
        find_computation(d, 1)["body"]["iterate"]["foreach"]["max_inflight"] = 0
    neg("foreach/max-inflight-zero", "invalid_max_inflight",
        foreach_max_inflight_zero)

    def while_base(d):
        d["computations"] = [
            comp(1, [], {"iterate": {"while": {
                "guard": 2, "body": 2,
                "carry": [{"slot": 6, "initial": 1, "body_output_port": 1}],
                "max_iterations": 10}}}),
            comp(2, [obind(1, 7)], AWAIT_USER),
        ]
        d["root"] = 1
        d["values"] = BASE_VALUES + [slot(6, 1), slot(7, 1)]
        return d

    def while_guard_not_bool(d):
        while_base(d)
        find_computation(d, 1)["body"]["iterate"]["while"]["guard"] = 1
    neg("while/guard-not-bool", "while_guard_not_bool", while_guard_not_bool)

    def while_max_iterations_zero(d):
        while_base(d)
        find_computation(d, 1)["body"]["iterate"]["while"]["max_iterations"] = 0
    neg("while/max-iterations-zero", "invalid_max_iterations",
        while_max_iterations_zero)

    def carry_slot_missing(d):
        while_base(d)
        find_computation(d, 1)["body"]["iterate"]["while"]["carry"][0]["slot"] = 99
    neg("carry/slot-missing", "carry_unknown_slot", carry_slot_missing)

    def carry_initial_type_mismatch(d):
        while_base(d)
        find_computation(d, 1)["body"]["iterate"]["while"]["carry"][0]["initial"] = 2
    neg("carry/initial-type-mismatch", "carry_type_mismatch",
        carry_initial_type_mismatch)

    def carry_body_output_missing(d):
        while_base(d)
        find_computation(d, 1)["body"]["iterate"]["while"]["carry"][0][
            "body_output_port"] = 9
    neg("carry/body-output-missing", "carry_unknown_port",
        carry_body_output_missing)

    def carry_next_type_mismatch(d):
        while_base(d)
        d["values"][-1] = slot(7, 2)
    neg("carry/next-type-mismatch", "carry_type_mismatch",
        carry_next_type_mismatch)

    def carry_duplicate_slot(d):
        while_base(d)
        find_computation(d, 1)["body"]["iterate"]["while"]["carry"].append(
            {"slot": 6, "initial": 1, "body_output_port": 1})
    neg("carry/duplicate-slot", "duplicate_carry_slot", carry_duplicate_slot)

    # retry / timeout -------------------------------------------------------
    def retry_child_missing(d):
        d["computations"] = [
            comp(1, [], {"retry": {"child": 99, "delays": []}}),
        ]
        d["root"] = 1
        return d
    neg("retry/child-missing", "unknown_computation", retry_child_missing)

    def retry_interface_mismatch(d):
        find_computation(d, 1)["body"] = {"retry": {"child": 2, "delays": []}}
        find_computation(d, 1)["outputs"] = [obind(1, 9)]
        d["values"].append(slot(9, 1))
    neg("retry/interface-mismatch", "retry_interface_mismatch",
        retry_interface_mismatch)

    def timeout_base(d):
        d["computations"] = [
            comp(1, [], {"timeout": {"child": 2, "timeout": 5}}),
            comp(2, [], AWAIT_USER),
        ]
        d["root"] = 1
        return d

    def timeout_not_duration(d):
        timeout_base(d)
        find_computation(d, 1)["body"]["timeout"]["timeout"] = 1
    neg("timeout/expression-not-duration", "timeout_not_duration",
        timeout_not_duration)

    def timeout_interface_mismatch(d):
        timeout_base(d)
        find_computation(d, 1)["outputs"] = [obind(1, 9)]
        d["values"].append(slot(9, 1))
    neg("timeout/interface-mismatch", "timeout_interface_mismatch",
        timeout_interface_mismatch)

    # handle ----------------------------------------------------------------
    def handle_base(d):
        d["computations"] = [
            comp(1, [], {"handle": {"child": 2, "clauses": [
                {"ordinal": 1,
                 "match": {"failure": {"failure_type": USER_REF}},
                 "handler": 3},
                {"ordinal": 2, "match": {"timed_out": None}, "handler": 4},
            ]}}),
            comp(2, [], AWAIT_USER),
            comp(3, [], AWAIT_USER),
            comp(4, [], AWAIT_USER),
        ]
        d["root"] = 1
        return d

    def duplicate_handler_ordinal(d):
        handle_base(d)
        find_computation(d, 1)["body"]["handle"]["clauses"][1]["ordinal"] = 1
    neg("handle/duplicate-handler-ordinal", "duplicate_handler_ordinal",
        duplicate_handler_ordinal)

    def duplicate_failure_matcher(d):
        handle_base(d)
        find_computation(d, 1)["body"]["handle"]["clauses"][1]["match"] = {
            "failure": {"failure_type": USER_REF}}
    neg("handle/duplicate-failure-matcher", "duplicate_failure_match",
        duplicate_failure_matcher)

    def two_timed_out(d):
        handle_base(d)
        find_computation(d, 1)["body"]["handle"]["clauses"][1]["match"] = {
            "timed_out": None}
        d["computations"].append(comp(5, [], AWAIT_USER))
        find_computation(d, 1)["body"]["handle"]["clauses"].append(
            {"ordinal": 3, "match": {"timed_out": None}, "handler": 5})
    neg("handle/two-timed-out", "duplicate_timed_out", two_timed_out)

    def unknown_handler(d):
        handle_base(d)
        find_computation(d, 1)["body"]["handle"]["clauses"][0]["handler"] = 99
    neg("handle/unknown-handler", "unknown_computation", unknown_handler)

    def handler_interface_mismatch(d):
        handle_base(d)
        find_computation(d, 3)["outputs"] = [obind(1, 9)]
        d["values"].append(slot(9, 1))
    neg("handle/handler-interface-mismatch", "handle_interface_mismatch",
        handler_interface_mismatch)

    def protected_child_interface_mismatch(d):
        handle_base(d)
        find_computation(d, 2)["outputs"] = [obind(1, 9)]
        d["values"].append(slot(9, 1))
    neg("handle/protected-child-interface-mismatch", "handle_interface_mismatch",
        protected_child_interface_mismatch)

    def cancelled_matcher(d):
        handle_base(d)
        find_computation(d, 1)["body"]["handle"]["clauses"][1]["match"] = {
            "cancelled": None}
        return d
    neg("handle/cancelled-matcher", "deserialize_rejected", cancelled_matcher,
        rejected_at="deserialize")

    # scope / compensation scope --------------------------------------------
    def scope_base(d):
        d["computations"] = [
            comp(1, [], {"scope": {"child": 2, "resources": [1, 2]}}),
            comp(2, [], AWAIT_USER),
        ]
        d["root"] = 1
        return d

    def scope_unknown_resource(d):
        scope_base(d)
        find_computation(d, 1)["body"]["scope"]["resources"] = [1, 99]
    neg("scope/unknown-resource", "scope_unknown_resource",
        scope_unknown_resource)

    def scope_duplicate_resource(d):
        scope_base(d)
        find_computation(d, 1)["body"]["scope"]["resources"] = [1, 1]
    neg("scope/duplicate-resource", "scope_duplicate_resource",
        scope_duplicate_resource)

    def scope_interface_mismatch(d):
        scope_base(d)
        find_computation(d, 1)["outputs"] = [obind(1, 9)]
        d["values"].append(slot(9, 1))
    neg("scope/interface-mismatch", "scope_interface_mismatch",
        scope_interface_mismatch)

    def compscope_interface_mismatch(d):
        find_computation(d, 1)["body"] = {"compensation_scope": {"child": 2}}
        find_computation(d, 1)["outputs"] = [obind(1, 9)]
        d["values"].append(slot(9, 1))
    neg("compensation-scope/interface-mismatch",
        "compensation_scope_interface_mismatch", compscope_interface_mismatch)

    # call subflow ----------------------------------------------------------
    def call_subflow_base(d):
        d["computations"] = [
            comp(1, [], {"call_subflow": {"contract": 2,
                                          "inputs": [ibind(1, 1)],
                                          "resources": [rbind(1, 1)]}}),
        ]
        d["root"] = 1
        d["contracts"] = [component_contract(), subflow_contract()]
        return d

    def call_subflow_wrong_kind(d):
        call_subflow_base(d)
        find_computation(d, 1)["body"]["call_subflow"]["contract"] = 1
    neg("call-subflow/wrong-contract-kind", "call_subflow_contract_kind",
        call_subflow_wrong_kind)

    def call_subflow_duplicate_input_port(d):
        call_subflow_base(d)
        find_computation(d, 1)["body"]["call_subflow"]["inputs"] = [
            ibind(1, 1), ibind(1, 2)]
    neg("call-subflow/duplicate-input-port", "duplicate_input_port",
        call_subflow_duplicate_input_port)

    def call_subflow_duplicate_resource_port(d):
        call_subflow_base(d)
        find_computation(d, 1)["body"]["call_subflow"]["resources"] = [
            rbind(1, 1), rbind(1, 2)]
    neg("call-subflow/duplicate-resource-port", "duplicate_resource_port",
        call_subflow_duplicate_resource_port)

    def call_subflow_unknown_expression(d):
        call_subflow_base(d)
        find_computation(d, 1)["body"]["call_subflow"]["inputs"] = [
            ibind(1, 99)]
    neg("call-subflow/unknown-expression", "unknown_expression",
        call_subflow_unknown_expression)

    def call_subflow_unknown_resource(d):
        call_subflow_base(d)
        find_computation(d, 1)["body"]["call_subflow"]["resources"] = [
            rbind(1, 99)]
    neg("call-subflow/unknown-resource", "unknown_resource_slot",
        call_subflow_unknown_resource)

    def iterate_item_and_output(d):
        foreach_base(d)
        find_computation(d, 2)["outputs"] = [obind(1, 6)]
    neg("producer/iterate-item-and-output", "duplicate_value_producer",
        iterate_item_and_output)

    def output_unknown_slot(d):
        find_computation(d, 2)["outputs"] = [obind(1, 99)]
    neg("output/unknown-slot", "unknown_value_slot", output_unknown_slot)

    def duplicate_child_reference(d):
        find_computation(d, 1)["body"]["sequence"]["children"] = [2, 2]
    neg("document/duplicate-child-reference", "duplicate_child_reference",
        duplicate_child_reference)


build_negatives()


# ---------------------------------------------------------------------------
# Equivalence (ordering) groups and semantic-order mutations
# ---------------------------------------------------------------------------

def ordering_base():
    return copy.deepcopy(next(doc for case_id, doc in GOLDENS
                              if case_id == "select/minimal"))


def rich_base():
    """A document exercising every canonically-ordered nested list."""
    return copy.deepcopy(next(doc for case_id, doc in GOLDENS
                              if case_id == "parallel/fail-fast"))


def equivalence_cases():
    """Ordering equivalence groups and semantic-order mutations.

    Every entry embeds its documents and freezes their expected digests; the
    Rust harness recomputes and compares against the frozen values.
    """
    groups = []
    mutations = []

    def golden(case_name):
        return copy.deepcopy(
            next(doc for name, doc in GOLDENS if name == case_name))

    def group(case_id, claim, base_case, mutate):
        base = golden(base_case)
        member = copy.deepcopy(base)
        mutate(member)
        validate(base)
        validate(member)
        _p, base_digest = document_digest(base)
        _p, member_digest = document_digest(member)
        assert base_digest == member_digest, (
            f"{case_id}: ordering changed the digest")
        groups.append({
            "case_id": case_id,
            "claim": claim,
            "base_case": base_case,
            "base_document": base,
            "member_document": member,
            "expected_digest": base_digest,
        })

    group("ordering/computations",
          "computation table source order does not change the digest",
          "select/minimal", lambda d: d["computations"].reverse())
    group("ordering/core-tables",
          "type/value/expression/resource source order does not change the digest",
          "select/minimal", lambda d: (
              d["types"].reverse(),
              d["values"].reverse(),
              d["expressions"].reverse(),
              d["resources"].reverse()))
    group("ordering/select-cases",
          "select case source order with the same ordinals does not change the digest",
          "select/minimal",
          lambda d: find_computation(d, 1)["body"]["select"]["cases"].reverse())
    group("ordering/parallel-branches",
          "parallel branch source order with the same ordinals does not change the digest",
          "parallel/fail-fast",
          lambda d: find_computation(d, 1)["body"]["parallel"]["branches"].reverse())
    group("ordering/parallel-output-bindings",
          "parallel output binding order does not change the digest",
          "parallel/fail-fast",
          lambda d: find_computation(d, 1)["body"]["parallel"]["output_bindings"].reverse())
    group("ordering/scope-resources",
          "scope resource source order does not change the digest",
          "scope/minimal",
          lambda d: find_computation(d, 1)["body"]["scope"]["resources"].reverse())
    group("ordering/run-parameters",
          "run parameter table order does not change the digest",
          "select/minimal",
          lambda d: d["run_parameters"].reverse())
    group("ordering/workflow-outputs",
          "workflow output table order does not change the digest",
          "select/minimal",
          lambda d: d["workflow_outputs"].reverse())
    group("ordering/semantic-refs",
          "semantic ref order does not change the digest",
          "call-subflow/minimal",
          lambda d: d["semantic_refs"].reverse())
    group("ordering/handle-clauses",
          "handle clause order does not change the digest",
          "handle/timed-out",
          lambda d: find_computation(d, 1)["body"]["handle"]["clauses"].reverse())

    def carry_base():
        base = golden("iterate/while")
        loop = find_computation(base, 1)["body"]["iterate"]["while"]
        loop["carry"].append({"slot": 8, "initial": 1, "body_output_port": 1})
        base["values"].append(slot(8, 1))
        return base

    def group_from_documents(case_id, claim, base, mutate):
        member = copy.deepcopy(base)
        mutate(member)
        validate(base)
        validate(member)
        _p, base_digest = document_digest(base)
        _p, member_digest = document_digest(member)
        assert base_digest == member_digest, (
            f"{case_id}: ordering changed the digest")
        groups.append({
            "case_id": case_id,
            "claim": claim,
            "base_case": None,
            "base_document": base,
            "member_document": member,
            "expected_digest": base_digest,
        })

    group_from_documents(
        "ordering/carry",
        "loop carry binding order does not change the digest",
        carry_base(),
        lambda d: find_computation(d, 1)["body"]["iterate"]["while"]["carry"].reverse())

    def mutation(case_id, claim, base, mutated):
        validate(base)
        validate(mutated)
        _p, base_digest = document_digest(base)
        _p, mutated_digest = document_digest(mutated)
        assert base_digest != mutated_digest, (
            f"{case_id}: semantic order change did not move the digest")
        mutations.append({
            "case_id": case_id,
            "claim": claim,
            "base_document": base,
            "mutated_document": mutated,
            "base_digest": base_digest,
            "mutated_digest": mutated_digest,
        })

    def sequence_pair(children):
        document = doc(
            semantic_refs=[],
            values=BASE_VALUES,
            expressions=BASE_EXPRESSIONS,
            resources=BASE_RESOURCES,
            contracts=[],
            run_parameters=BASE_RUN_PARAMETERS,
            workflow_outputs=[{"name": "done", "expression": 2}],
            computations=[
                comp(1, [], {"sequence": {"children": children}}),
                comp(2, [], AWAIT_USER),
                comp(3, [], AWAIT_USER),
            ],
            root=1,
        )
        return document

    mutation("mutation/sequence-child-order",
             "sequence child order is semantic: swapping children changes the digest",
             sequence_pair([2, 3]), sequence_pair([3, 2]))

    def swapped_delays():
        document = golden("retry/minimal")
        find_computation(document, 1)["body"]["retry"]["delays"] = [5000000, 0]
        return document
    mutation("mutation/retry-delay-order",
             "retry delay order is the schedule: reordering changes the digest",
             golden("retry/minimal"), swapped_delays())

    def fail_fast_mode():
        document = golden("parallel/all")
        find_computation(document, 1)["body"]["parallel"]["mode"] = "fail_fast"
        return document
    mutation("mutation/parallel-mode",
             "a semantic body configuration change changes the digest",
             golden("parallel/all"), fail_fast_mode())

    return groups, mutations


EQUIVALENCE_GROUPS, MUTATIONS = equivalence_cases()


# ---------------------------------------------------------------------------
# Self-checks + emit
# ---------------------------------------------------------------------------

def write_json(relative, payload):
    path = ROOT / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    text = json.dumps(payload, indent=2, ensure_ascii=False) + "\n"
    path.write_text(text, encoding="utf-8", newline="\n")
    print(f"wrote {relative}")


def main():
    # Every golden validates under the independent Python validator.
    golden_entries = []
    for case_id, document in GOLDENS:
        validate(document)
        payload, digest = document_digest(document)
        golden_entries.append({
            "case_id": case_id,
            "document": document,
            "ace1_utf8": payload.decode("utf-8"),
            "ace1_hex": payload.hex(),
            "semantic_ir_digest": digest,
        })
    print(f"validated {len(golden_entries)} golden documents")

    # Every negative fails with exactly its frozen kind (or cannot even be
    # represented by the closed model: rejected_at == "deserialize").
    for case in NEGATIVE_CASES:
        if case["rejected_at"] == "deserialize":
            continue
        try:
            validate(case["document"])
        except ValidationError as error:
            assert error.kind == case["error_kind"], (
                f"{case['case_id']}: expected {case['error_kind']}, "
                f"got {error.kind}")
        else:
            raise AssertionError(f"{case['case_id']} unexpectedly validated")
    print(f"validated {len(NEGATIVE_CASES)} negative cases")

    # Equivalence groups and mutations were verified at construction time
    # (each entry froze its own expected digests); here we cross-check the
    # linked bases against the frozen golden digests.
    golden_digests = {entry["case_id"]: entry["semantic_ir_digest"]
                      for entry in golden_entries}
    for group in EQUIVALENCE_GROUPS:
        if group["base_case"] is not None:
            assert group["expected_digest"] == golden_digests[group["base_case"]], (
                f"{group['case_id']}: base digest does not match the golden")
    print(f"verified {len(EQUIVALENCE_GROUPS)} ordering groups and "
          f"{len(MUTATIONS)} mutations")

    # The 13-opcode table in the schema module is frozen.
    module = afidl.parse_afidl(SEMANTIC_IR.read_text(encoding="utf-8"))
    body_variant = next(d for d in module["declarations"]
                        if d["kind"] == "variant" and d["name"] == "ComputationBody")
    actual = [(case["no"], case["name"]) for case in body_variant["cases"]]
    assert actual == OPCODE_TABLE, f"opcode table drifted: {actual}"
    forbidden = ("agent", "model", "tool", "http", "sql", "shell", "human",
                 "mcp", "a2a", "trigger", "call_composite", "cancelled")
    names = {name for _, name in actual}
    assert not (names & set(forbidden)), "forbidden opcode present"
    print("ComputationBody opcode table is exactly the frozen 13")

    write_json("spec/conformance/semantic-ir/golden/semantic-ir-golden.json", {
        "kind": "agentflow.conformance.semantic-ir.golden",
        "corpus_revision": CORPUS_REVISION,
        "note": (
            "Frozen canonical ACE1 bytes and SemanticIrDigest values for the "
            "Phase 3B semantic IR corpus, generated by the independent Python "
            "implementation (this script). The Rust implementation must "
            "reproduce them; tests never generate expected values at runtime."
        ),
        "fixture_schema": {
            "file": "spec/conformance/semantic-core/schema-universe.afidl",
            "specification_id": "agentflow.test.semantic@1",
            "types": [{"name": name, "digest_hex": digest}
                      for name, digest in sorted(FIXTURE_DIGESTS.items())],
        },
        "golden": golden_entries,
    })

    write_json("spec/conformance/semantic-ir/negative/semantic-ir-negative.json", {
        "kind": "agentflow.conformance.semantic-ir.negative",
        "corpus_revision": CORPUS_REVISION,
        "note": (
            "Frozen negative documents with stable error kinds. "
            "rejected_at selects the layer: \"check\" (validator) or "
            "\"deserialize\" (the closed serde/schema model cannot even "
            "represent the document)."
        ),
        "cases": NEGATIVE_CASES,
    })

    write_json("spec/conformance/semantic-ir/equivalence/semantic-ir-equivalence.json", {
        "kind": "agentflow.conformance.semantic-ir.equivalence",
        "corpus_revision": CORPUS_REVISION,
        "note": (
            "Ordering equivalence: each member must produce exactly the base "
            "case's frozen digest. Mutations: the mutated document must "
            "produce a different digest than its base."
        ),
        "ordering_groups": EQUIVALENCE_GROUPS,
        "mutations": MUTATIONS,
    })


def documents_equal(a, b):
    return a == b


def find_opcode(document):
    """Raises StopIteration when a body opcode is not decodable."""
    for computation in document["computations"]:
        opcode, _payload = body_opcode(computation)
        if opcode not in {name for _, name in OPCODE_TABLE}:
            raise StopIteration(opcode)


if __name__ == "__main__":
    main()
