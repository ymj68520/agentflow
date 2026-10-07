#!/usr/bin/env python3
"""Freezes the Phase 3D subflow-interface conformance corpus.

INDEPENDENT implementation of the subflow interface rules
(spec/semantic/semantic-ir.md, subflow-interface-derivation section):
interface derivation from the target's run_parameters / workflow_outputs /
resources tables, structural cross-document type expansion, exact-digest
target lookup and CALL_SUBFLOW port/type checking. The Rust implementation
in crates/af-ir (`SubflowIrUniverse` / `check_subflow_interfaces`) must
agree with the frozen expectations; expected values are never derived from
the Rust code.

Plumbing reuse (allowed by the phase): the Phase 3B document builder,
its validator and the SemanticIrDigest computation; the Phase 3C
availability oracle for the target/caller validity prerequisite. The
subflow logic itself — port derivation, type expansion, lookup, checking —
is implemented here, independently of the Rust checker.

Run from the repository root:  python tools/gen-subflow-interface-corpus.py
"""

import copy
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CORPUS_REVISION = 1


def _load(name, file_name):
    spec = importlib.util.spec_from_file_location(
        name, Path(__file__).resolve().parent / file_name)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


sir = _load("semantic_ir_corpus", "gen-semantic-ir-corpus.py")
dfc = _load("semantic_dataflow_corpus", "gen-semantic-dataflow-corpus.py")

USER_REF = sir.USER_REF
STATUS_REF = sir.STATUS_REF

# A second fixture schema: same record name, different specification and
# therefore a different exact TypeSchemaDigest. Both refs are universe-valid,
# so a pin mismatch is a Phase 3D rejection, never a Phase 3A one.
V2_SCHEMA_SOURCE = (
    "afidl 1;\nschema agentflow.test.semantic.v2@1;\n"
    "record User { 1 name: string; }\n"
)


def v2_user_ref():
    _modules, types_by_module = sir.afidl.compile_module_set(
        [("v2.afidl", V2_SCHEMA_SOURCE)])
    schema = types_by_module["agentflow.test.semantic.v2@1"][0]
    _payload, digest = sir.afidl.type_schema_digest(schema)
    return {
        "specification_id": "agentflow.test.semantic.v2@1",
        "type_name": "User",
        "schema_digest": {"algorithm": "sha256", "domain": "type-schema",
                          "hex": digest},
    }


V2_USER_REF = v2_user_ref()

FIXTURE_SCHEMAS = [{
    "specification_id": "agentflow.test.semantic.v2@1",
    "source": V2_SCHEMA_SOURCE,
    "types": [{"name": "User", "digest_hex": V2_USER_REF["schema_digest"]["hex"]}],
}]


# ---------------------------------------------------------------------------
# Independent subflow oracle
# ---------------------------------------------------------------------------

class SubflowError(Exception):
    def __init__(self, kind, detail=""):
        super().__init__(f"{kind}: {detail}")
        self.kind = kind
        self.detail = detail


def expand_type(types_by_id, type_id):
    """Expands a document-local type id into a structural tuple."""
    body = types_by_id[type_id]["body"]
    kind_, payload = next(iter(body.items()))
    if kind_ == "primitive":
        return ("primitive", payload)
    if kind_ == "schema":
        return ("schema", payload["specification_id"], payload["type_name"],
                payload["schema_digest"]["hex"])
    if kind_ == "list":
        return ("list", expand_type(types_by_id, payload))
    if kind_ == "map_string":
        return ("map_string", expand_type(types_by_id, payload))
    if kind_ == "option":
        return ("option", expand_type(types_by_id, payload))
    raise AssertionError(f"unknown type body {kind_!r}")


def render_type(structural):
    """The frozen diagnostic rendering (mirrors the Rust canonical_string)."""
    kind_ = structural[0]
    if kind_ == "primitive":
        return f"primitive:{structural[1]}"
    if kind_ == "schema":
        return f"schema:{structural[1]}/{structural[2]}@{structural[3]}"
    inner = render_type(structural[1])
    if kind_ == "list":
        return f"list:{inner}"
    if kind_ == "map_string":
        return f"map_string:{inner}"
    if kind_ == "option":
        return f"option:{inner}"
    raise AssertionError(structural)


def render_ref(reference):
    return (f"{reference['specification_id']}/{reference['type_name']}"
            f"@{reference['schema_digest']['hex']}")


def nfc(text):
    return sir.nfc(text)


def derive_interface(target):
    """Derives the subflow interface (frozen canonical port mappings)."""
    types_by_id = {t["id"]: t for t in target["types"]}
    expressions = {e["id"]: e for e in target["expressions"]}

    parameters = sorted(target["run_parameters"], key=lambda p: p["id"])
    inputs = [{
        "port_no": index + 1,
        "name": parameter["name"],
        "type": render_type(expand_type(types_by_id, parameter["type"])),
    } for index, parameter in enumerate(parameters)]

    outputs_by_name = sorted(
        target["workflow_outputs"],
        key=lambda output: nfc(output["name"]).encode("utf-8"))
    outputs = [{
        "port_no": index + 1,
        "name": output["name"],
        "type": render_type(expand_type(
            types_by_id, expressions[output["expression"]]["result_type"])),
    } for index, output in enumerate(outputs_by_name)]

    slots = sorted(target["resources"], key=lambda slot: slot["id"])
    resources = [{
        "port_no": index + 1,
        "source_resource_slot": slot["id"],
        "requirement_type": render_ref(slot["requirement_type"]),
    } for index, slot in enumerate(slots)]

    return {"inputs": inputs, "outputs": outputs, "resources": resources}


def check_subflows(caller, interfaces_by_hex):
    """Checks every CALL_SUBFLOW against the explicit universe."""
    computations = {c["id"]: c for c in caller["computations"]}
    contracts = {c["id"]: c for c in caller["contracts"]}
    types_by_id = {t["id"]: t for t in caller["types"]}
    expressions = {e["id"]: e for e in caller["expressions"]}
    values = {v["id"]: v for v in caller["values"]}
    resources = {r["id"]: r for r in caller["resources"]}

    def ports_of(n):
        return set(range(1, n + 1))

    def port_difference(expected, actual):
        return sorted(expected - actual), sorted(actual - expected)

    for computation_id in sorted(computations):
        computation = computations[computation_id]
        opcode, payload = sir.body_opcode(computation)
        if opcode != "call_subflow":
            continue
        contract = contracts[payload["contract"]]
        hex_ = contract["digest"]["hex"]
        if hex_ not in interfaces_by_hex:
            raise SubflowError(
                "unknown_subflow_target",
                f"computation {computation_id}: digest {hex_} not in universe")
        interface = interfaces_by_hex[hex_]

        bound_inputs = {b["port_no"] for b in payload["inputs"]}
        expected_inputs = ports_of(len(interface["inputs"]))
        if bound_inputs != expected_inputs:
            raise SubflowError("subflow_input_port_mismatch",
                               f"computation {computation_id}")
        for binding in sorted(payload["inputs"], key=lambda b: b["port_no"]):
            actual = expand_type(
                types_by_id, expressions[binding["expression"]]["result_type"])
            expected = interface["inputs"][binding["port_no"] - 1]["type"]
            if render_type(actual) != expected:
                raise SubflowError(
                    "subflow_input_type_mismatch",
                    f"computation {computation_id} input "
                    f"{binding['port_no']}: expected {expected}, "
                    f"got {render_type(actual)}")

        bound_outputs = {o["port_no"] for o in computation["outputs"]}
        expected_outputs = ports_of(len(interface["outputs"]))
        if bound_outputs != expected_outputs:
            raise SubflowError("subflow_output_port_mismatch",
                               f"computation {computation_id}")
        for binding in sorted(computation["outputs"], key=lambda b: b["port_no"]):
            actual = expand_type(types_by_id, values[binding["value_slot"]]["type"])
            expected = interface["outputs"][binding["port_no"] - 1]["type"]
            if render_type(actual) != expected:
                raise SubflowError(
                    "subflow_output_type_mismatch",
                    f"computation {computation_id} output "
                    f"{binding['port_no']}: expected {expected}, "
                    f"got {render_type(actual)}")

        bound_resources = {b["port_no"] for b in payload["resources"]}
        expected_resources = ports_of(len(interface["resources"]))
        if bound_resources != expected_resources:
            raise SubflowError("subflow_resource_port_mismatch",
                               f"computation {computation_id}")
        for binding in sorted(payload["resources"], key=lambda b: b["port_no"]):
            actual = render_ref(resources[binding["resource_slot"]]
                                ["requirement_type"])
            expected = (interface["resources"][binding["port_no"] - 1]
                        ["requirement_type"])
            if actual != expected:
                raise SubflowError(
                    "subflow_resource_type_mismatch",
                    f"computation {computation_id} resource "
                    f"{binding['port_no']}: expected {expected}, got {actual}")


def digest_of(document):
    return sir.document_digest(document)[1]


def valid(document):
    """The target/caller validity prerequisite: Phase 3B + Phase 3C."""
    sir.validate(document)
    dfc.analyze(document)


# ---------------------------------------------------------------------------
# Document construction helpers
# ---------------------------------------------------------------------------

def prim(node_id, keyword):
    return {"id": node_id, "body": {"primitive": keyword}}


def lst(node_id, inner):
    return {"id": node_id, "body": {"list": inner}}


def mapstr(node_id, inner):
    return {"id": node_id, "body": {"map_string": inner}}


def opt(node_id, inner):
    return {"id": node_id, "body": {"option": inner}}


def schema_node(node_id, reference):
    return {"id": node_id, "body": {"schema": reference}}


def subflow_contract(digest_hex, contract_id=1):
    return {"id": contract_id, "kind": "subflow",
            "digest": {"algorithm": "sha256", "domain": "semantic-ir",
                       "hex": digest_hex}}


def doc(**fields):
    base = {
        "semantic_refs": [], "contracts": [], "run_parameters": [],
        "workflow_outputs": [], "resources": [], "values": [],
        "expressions": [],
    }
    base.update(fields)
    return sir.doc(
        semantic_refs=base["semantic_refs"], types=base["types"],
        values=base["values"], expressions=base["expressions"],
        resources=base["resources"], contracts=base["contracts"],
        run_parameters=base["run_parameters"],
        workflow_outputs=base["workflow_outputs"],
        computations=base["computations"], root=base["root"])


def await_root(correlation_expr=1):
    return {"await": {"signal_type": USER_REF, "correlation": correlation_expr}}


def call_body(contract_id=1, inputs=(), resources=()):
    return {"call_subflow": {"contract": contract_id,
                             "inputs": list(inputs),
                             "resources": list(resources)}}


POSITIVE_CASES = []
NEGATIVE_CASES = []


def positive(case_id, targets, caller, note=""):
    """Freezes one positive; the oracle validates and derives everything."""
    valid(caller)
    for target in targets:
        valid(target)
    interfaces_by_hex = {digest_of(target): derive_interface(target)
                         for target in targets}
    check_subflows(caller, interfaces_by_hex)
    interfaces = [derive_interface(target) for target in targets]
    POSITIVE_CASES.append({
        "case_id": case_id,
        "note": note,
        "targets": targets,
        "caller": caller,
        "frozen_target_digests": sorted(interfaces_by_hex),
        "interfaces": interfaces,
    })


def negative(case_id, error_kind, targets, caller, note=""):
    valid(caller)
    for target in targets:
        valid(target)
    interfaces_by_hex = {digest_of(target): derive_interface(target)
                         for target in targets}
    try:
        check_subflows(caller, interfaces_by_hex)
    except SubflowError as error:
        assert error.kind == error_kind, (
            f"{case_id}: expected {error_kind}, got {error.kind}")
    else:
        raise AssertionError(f"{case_id} unexpectedly checked clean")
    NEGATIVE_CASES.append({
        "case_id": case_id,
        "note": note,
        "targets": targets,
        "caller": caller,
        "error_kind": error_kind,
    })


# --- Basic targets -----------------------------------------------------------

BASIC_TARGET = doc(
    types=[prim(1, "int64")],
    run_parameters=[sir.rp(1, "limit", 1, 1)],
    values=[sir.slot(1, 1)],
    expressions=[sir.expr(1, 1, {"value_slot": 1})],
    workflow_outputs=[{"name": "total", "expression": 1}],
    computations=[sir.comp(1, [], await_root())],
    root=1,
)

BASIC_CALLER_TEMPLATE = doc(
    types=[prim(1, "int64")],
    values=[sir.slot(9, 1)],
    expressions=[sir.expr(1, 1, {"constant": {"int64": 7}})],
    computations=[sir.comp(1, [sir.obind(1, 9)],
                           call_body(inputs=[sir.ibind(1, 1)]))],
    root=1,
)


def caller_pinning(caller, digest_hex):
    caller = copy.deepcopy(caller)
    caller["contracts"] = [subflow_contract(digest_hex)]
    return caller


# --- Positives ---------------------------------------------------------------

_EMPTY_TARGET = doc(types=[prim(1, "int64")],
                    expressions=[sir.expr(1, 1, {"constant": {"int64": 0}})],
                    computations=[sir.comp(1, [], await_root())], root=1)
positive("subflow/minimal-empty-interface",
         [_EMPTY_TARGET],
         caller_pinning(doc(
             types=[prim(1, "int64")],
             computations=[sir.comp(1, [], call_body())], root=1),
             digest_of(_EMPTY_TARGET)),
         note="empty target interface; the caller binds nothing")

positive("subflow/basic-input-output",
         [BASIC_TARGET],
         caller_pinning(BASIC_CALLER_TEMPLATE, digest_of(BASIC_TARGET)),
         note="one input port, one output port")

# Multiple inputs: ports follow RunParameterId order, not table order.
ORDER_INPUTS_TARGET = doc(
    types=[prim(1, "int64"), prim(2, "string"), prim(3, "bool")],
    run_parameters=[sir.rp(5, "limit", 1, 6), sir.rp(2, "label", 2, 7),
                    sir.rp(9, "flag", 3, 8)],
    values=[sir.slot(6, 1), sir.slot(7, 2), sir.slot(8, 3)],
    expressions=[sir.expr(1, 1, {"constant": {"int64": 0}})],
    computations=[sir.comp(1, [], await_root())],
    root=1,
)
positive("subflow/multiple-inputs-canonical-port-order",
         [ORDER_INPUTS_TARGET],
         caller_pinning(doc(
             types=[prim(1, "int64"), prim(2, "string"), prim(3, "bool")],
             expressions=[
                 sir.expr(1, 1, {"constant": {"int64": 5}}),
                 sir.expr(2, 2, {"constant": {"string": "x"}}),
                 sir.expr(3, 3, {"constant": {"bool": True}}),
             ],
             computations=[sir.comp(1, [], call_body(inputs=[
                 sir.ibind(2, 1), sir.ibind(3, 3), sir.ibind(1, 2)]))],
             root=1),
             digest_of(ORDER_INPUTS_TARGET)),
         note="input ports 1..3 map parameter ids 2, 5, 9")

# Multiple outputs: ports follow canonical NFC-name order, not table order.
ORDER_OUTPUTS_TARGET = doc(
    types=[prim(1, "int64"), prim(2, "string"), prim(3, "bool")],
    run_parameters=[sir.rp(1, "limit", 1, 6)],
    values=[sir.slot(6, 1)],
    expressions=[
        sir.expr(1, 1, {"value_slot": 6}),
        sir.expr(2, 2, {"constant": {"string": "a"}}),
        sir.expr(3, 3, {"constant": {"bool": True}}),
    ],
    workflow_outputs=[{"name": "zeta", "expression": 1},
                      {"name": "alpha", "expression": 2},
                      {"name": "mica", "expression": 3}],
    computations=[sir.comp(1, [], await_root())],
    root=1,
)
positive("subflow/multiple-outputs-canonical-port-order",
         [ORDER_OUTPUTS_TARGET],
         caller_pinning(doc(
             types=[prim(1, "int64"), prim(2, "string"), prim(3, "bool")],
             run_parameters=[sir.rp(1, "limit", 1, 6)],
             values=[sir.slot(6, 1), sir.slot(7, 2), sir.slot(8, 3),
                     sir.slot(12, 1)],
             expressions=[sir.expr(1, 1, {"value_slot": 6})],
             computations=[sir.comp(
                 1,
                 [sir.obind(3, 12), sir.obind(1, 7), sir.obind(2, 8)],
                 call_body(inputs=[sir.ibind(1, 1)]))],
             root=1),
             digest_of(ORDER_OUTPUTS_TARGET)),
         note="output ports 1..3 map names alpha, mica, zeta")

# Resources: every declared slot is the interface; ports follow slot id order.
RESOURCES_TARGET = doc(
    types=[prim(1, "int64")],
    expressions=[sir.expr(1, 1, {"constant": {"int64": 0}})],
    resources=[sir.resource(7, USER_REF), sir.resource(3, STATUS_REF)],
    computations=[sir.comp(1, [], await_root())],
    root=1,
)
positive("subflow/resources",
         [RESOURCES_TARGET],
         caller_pinning(doc(
             types=[prim(1, "int64")],
             resources=[sir.resource(1, USER_REF), sir.resource(2, STATUS_REF)],
             expressions=[sir.expr(1, 1, {"constant": {"int64": 0}})],
             computations=[sir.comp(1, [], call_body(resources=[
                 sir.rbind(1, 2), sir.rbind(2, 1)]))],
             root=1),
             digest_of(RESOURCES_TARGET)),
         note="resource ports 1..2 map slot ids 3, 7")

# All three dimensions together.
ALL3_TARGET = doc(
    types=[prim(1, "int64"), prim(2, "string")],
    run_parameters=[sir.rp(4, "name", 2, 6), sir.rp(2, "limit", 1, 7)],
    values=[sir.slot(6, 2), sir.slot(7, 1)],
    expressions=[sir.expr(1, 2, {"value_slot": 6}),
                 sir.expr(2, 1, {"value_slot": 7})],
    workflow_outputs=[{"name": "greeting", "expression": 1},
                      {"name": "count", "expression": 2}],
    resources=[sir.resource(5, USER_REF)],
    computations=[sir.comp(1, [], await_root(correlation_expr=1))],
    root=1,
)
positive("subflow/all-three-dimensions",
         [ALL3_TARGET],
         caller_pinning(doc(
             types=[prim(1, "int64"), prim(2, "string")],
             run_parameters=[sir.rp(1, "who", 2, 1)],
             values=[sir.slot(1, 2), sir.slot(9, 1), sir.slot(8, 2)],
             expressions=[sir.expr(1, 2, {"value_slot": 1}),
                          sir.expr(2, 1, {"constant": {"int64": 5}})],
             resources=[sir.resource(1, USER_REF)],
             computations=[sir.comp(1, [sir.obind(1, 9), sir.obind(2, 8)],
                                    call_body(inputs=[sir.ibind(1, 2),
                                                      sir.ibind(2, 1)],
                                              resources=[sir.rbind(1, 1)]))],
             root=1),
             digest_of(ALL3_TARGET)),
         note="inputs, outputs and resources checked in one call")

# Deeply nested container types across different local type tables.
NESTED_TARGET = doc(
    types=[prim(1, "int64"), prim(2, "string"), opt(12, 1), lst(11, 12),
           mapstr(10, 11)],
    run_parameters=[sir.rp(1, "matrix", 10, 6)],
    values=[sir.slot(6, 10)],
    expressions=[sir.expr(1, 10, {"value_slot": 6})],
    workflow_outputs=[{"name": "out", "expression": 1}],
    computations=[sir.comp(1, [], await_root())],
    root=1,
)
positive("subflow/nested-list-map-option-types",
         [NESTED_TARGET],
         caller_pinning(doc(
             types=[prim(21, "int64"), prim(22, "string"), opt(23, 21),
                    lst(24, 23), mapstr(25, 24)],
             run_parameters=[sir.rp(1, "matrix", 25, 1)],
             values=[sir.slot(1, 25), sir.slot(9, 25)],
             expressions=[sir.expr(1, 25, {"value_slot": 1})],
             computations=[sir.comp(1, [sir.obind(1, 9)],
                                    call_body(inputs=[sir.ibind(1, 1)]))],
             root=1),
             digest_of(NESTED_TARGET)),
         note="map<string, list<option<int64>>> across different type ids")

# A schema-pinned input port.
SCHEMA_TARGET = doc(
    types=[prim(1, "int64"), schema_node(2, USER_REF)],
    run_parameters=[sir.rp(1, "user", 2, 6)],
    values=[sir.slot(6, 2)],
    expressions=[sir.expr(1, 1, {"constant": {"int64": 0}})],
    workflow_outputs=[{"name": "ok", "expression": 1}],
    computations=[sir.comp(1, [], await_root())],
    root=1,
)
positive("subflow/schema-type-input",
         [SCHEMA_TARGET],
         caller_pinning(doc(
             types=[prim(1, "int64"), schema_node(9, USER_REF)],
             run_parameters=[sir.rp(1, "user", 9, 1)],
             values=[sir.slot(1, 9), sir.slot(10, 1)],
             expressions=[sir.expr(1, 9, {"value_slot": 1})],
             computations=[sir.comp(1, [sir.obind(1, 10)],
                                    call_body(inputs=[sir.ibind(1, 1)]))],
             root=1),
             digest_of(SCHEMA_TARGET)),
         note="the input port type is an exact pinned schema")

# Same structure, different local type ids on both sides.
STRUCT_TARGET = doc(
    types=[prim(1, "int64"), prim(7, "string"), lst(8, 7)],
    run_parameters=[sir.rp(1, "tags", 8, 6)],
    values=[sir.slot(6, 8)],
    expressions=[sir.expr(1, 1, {"constant": {"int64": 0}})],
    workflow_outputs=[{"name": "ok", "expression": 1}],
    computations=[sir.comp(1, [], await_root())],
    root=1,
)
positive("subflow/same-structure-different-local-type-ids",
         [STRUCT_TARGET],
         caller_pinning(doc(
             types=[prim(50, "int64"), prim(41, "string"), lst(42, 41)],
             run_parameters=[sir.rp(1, "tags", 42, 1)],
             values=[sir.slot(1, 42), sir.slot(9, 50)],
             expressions=[sir.expr(1, 42, {"value_slot": 1})],
             computations=[sir.comp(1, [sir.obind(1, 9)],
                                    call_body(inputs=[sir.ibind(1, 1)]))],
             root=1),
             digest_of(STRUCT_TARGET)),
         note="list<string> as ids (8,7) vs (42,41)")

# Completely different type-table numbering; target type 3 = caller type 19.
NUMBERING_TARGET = doc(
    types=[prim(1, "int64"), prim(2, "string"), opt(3, 2)],
    run_parameters=[sir.rp(2, "count", 1, 6), sir.rp(5, "tags", 3, 7)],
    values=[sir.slot(6, 1), sir.slot(7, 3)],
    expressions=[sir.expr(1, 1, {"constant": {"int64": 0}})],
    computations=[sir.comp(1, [], await_root())],
    root=1,
)
positive("subflow/caller-and-target-use-different-type-table-numbering",
         [NUMBERING_TARGET],
         caller_pinning(doc(
             types=[prim(17, "string"), prim(18, "int64"), opt(19, 17)],
             run_parameters=[sir.rp(1, "n", 18, 1), sir.rp(2, "t", 19, 2)],
             values=[sir.slot(1, 18), sir.slot(2, 19)],
             expressions=[sir.expr(1, 18, {"value_slot": 1}),
                          sir.expr(2, 19, {"value_slot": 2})],
             computations=[sir.comp(1, [], call_body(inputs=[
                 sir.ibind(1, 1), sir.ibind(2, 2)]))],
             root=1),
             digest_of(NUMBERING_TARGET)),
         note="target type 3 (option<string>) == caller type 19")

# The same exact target called twice.
positive("subflow/same-target-called-twice",
         [BASIC_TARGET],
         caller_pinning(doc(
             types=[prim(1, "int64")],
             run_parameters=[sir.rp(1, "limit", 1, 1)],
             values=[sir.slot(1, 1), sir.slot(9, 1), sir.slot(10, 1),
                     sir.slot(11, 1)],
             expressions=[sir.expr(1, 1, {"value_slot": 1})],
             computations=[
                 sir.comp(1, [sir.obind(1, 11)],
                          {"sequence": {"children": [2, 3]}}),
                 sir.comp(2, [sir.obind(1, 9)],
                          call_body(inputs=[sir.ibind(1, 1)])),
                 sir.comp(3, [sir.obind(1, 10)],
                          call_body(inputs=[sir.ibind(1, 1)])),
             ],
             root=1),
             digest_of(BASIC_TARGET)),
         note="two calls pinning one digest")

# No CALL_SUBFLOW at all: the empty universe is legal.
positive("subflow/document-with-no-call-subflow-and-empty-universe",
         [],
         doc(types=[prim(1, "int64"), prim(2, "bool")],
             values=[sir.slot(1, 1), sir.slot(2, 2)],
             expressions=[sir.expr(1, 1, {"value_slot": 1})],
             run_parameters=[sir.rp(1, "limit", 1, 1), sir.rp(2, "flag", 2, 2)],
             computations=[
                 sir.comp(1, [], {"sequence": {"children": [2]}}),
                 sir.comp(2, [], await_root()),
             ],
             root=1),
         note="an empty universe against a document without CALL_SUBFLOW")


# --- Negatives ---------------------------------------------------------------

# Target resolution.
negative("subflow/unknown-target-digest", "unknown_subflow_target",
         [BASIC_TARGET],
         caller_pinning(BASIC_CALLER_TEMPLATE, "cd" * 32),
         note="the pinned digest is not in the universe")
_stale_v1 = BASIC_TARGET
_stale_v2 = doc(
    types=[prim(1, "int64"), prim(2, "string")],
    run_parameters=[sir.rp(1, "limit", 2, 7)],
    values=[sir.slot(7, 2)],
    expressions=[sir.expr(1, 2, {"value_slot": 7})],
    workflow_outputs=[{"name": "total", "expression": 1}],
    computations=[sir.comp(1, [], await_root())],
    root=1,
)
negative("subflow/stale-target-digest", "unknown_subflow_target",
         [_stale_v2],
         caller_pinning(BASIC_CALLER_TEMPLATE, digest_of(_stale_v1)),
         note="the caller pins the superseded digest; the universe holds v2")
negative("subflow/universe-does-not-fallback-latest", "unknown_subflow_target",
         [BASIC_TARGET],
         caller_pinning(BASIC_CALLER_TEMPLATE, "ab" * 32),
         note="no name-based latest fallback exists")

# Input ports.
_TWO_INPUT_TARGET = NUMBERING_TARGET
negative("input/missing-port", "subflow_input_port_mismatch",
         [_TWO_INPUT_TARGET],
         caller_pinning(doc(
             types=[prim(17, "string"), prim(18, "int64"), opt(19, 17)],
             run_parameters=[sir.rp(1, "n", 18, 1)],
             values=[sir.slot(1, 18)],
             expressions=[sir.expr(1, 18, {"value_slot": 1})],
             computations=[sir.comp(1, [], call_body(inputs=[
                 sir.ibind(1, 1)]))],
             root=1),
             digest_of(_TWO_INPUT_TARGET)),
         note="input port 2 is never bound")
negative("input/extra-port", "subflow_input_port_mismatch",
         [BASIC_TARGET],
         caller_pinning(doc(
             types=[prim(1, "int64")],
             expressions=[sir.expr(1, 1, {"constant": {"int64": 7}}),
                          sir.expr(2, 1, {"constant": {"int64": 8}})],
             computations=[sir.comp(1, [], call_body(inputs=[
                 sir.ibind(1, 1), sir.ibind(2, 2)]))],
             root=1),
             digest_of(BASIC_TARGET)),
         note="the target has no input port 2")

# Input types.
_SAME_ID_TARGET = doc(
    types=[prim(1, "int64"), prim(5, "string")],
    run_parameters=[sir.rp(1, "label", 5, 6)],
    values=[sir.slot(6, 5)],
    expressions=[sir.expr(1, 1, {"constant": {"int64": 0}})],
    computations=[sir.comp(1, [], await_root())],
    root=1,
)
negative("input/type-mismatch", "subflow_input_type_mismatch",
         [_SAME_ID_TARGET],
         caller_pinning(doc(
             # Caller type id 5 is int64: numeric id equality must not leak
             # across documents.
             types=[prim(1, "int64"), prim(5, "int64")],
             expressions=[sir.expr(1, 5, {"constant": {"int64": 7}})],
             computations=[sir.comp(1, [], call_body(inputs=[
                 sir.ibind(1, 1)]))],
             root=1),
             digest_of(_SAME_ID_TARGET)),
         note="local id 5 means string in the target, int64 in the caller")
_NESTED_IN_TARGET = doc(
    types=[prim(1, "int64"), prim(2, "string"), lst(8, 2)],
    run_parameters=[sir.rp(1, "tags", 8, 6)],
    values=[sir.slot(6, 8)],
    expressions=[sir.expr(1, 1, {"constant": {"int64": 0}})],
    computations=[sir.comp(1, [], await_root())],
    root=1,
)
negative("input/nested-type-mismatch", "subflow_input_type_mismatch",
         [_NESTED_IN_TARGET],
         caller_pinning(doc(
             types=[prim(1, "int64"), prim(2, "string"), lst(11, 1)],
             run_parameters=[sir.rp(1, "xs", 11, 1)],
             values=[sir.slot(1, 11)],
             expressions=[sir.expr(1, 11, {"value_slot": 1})],
             computations=[sir.comp(1, [], call_body(inputs=[
                 sir.ibind(1, 1)]))],
             root=1),
             digest_of(_NESTED_IN_TARGET)),
         note="list<string> vs list<int64>")
_SCHEMA_IN_TARGET = SCHEMA_TARGET
negative("input/schema-digest-mismatch", "subflow_input_type_mismatch",
         [_SCHEMA_IN_TARGET],
         caller_pinning(doc(
             types=[prim(1, "int64"), schema_node(9, V2_USER_REF)],
             run_parameters=[sir.rp(1, "user", 9, 1)],
             values=[sir.slot(1, 9)],
             expressions=[sir.expr(1, 9, {"value_slot": 1})],
             computations=[sir.comp(1, [], call_body(inputs=[
                 sir.ibind(1, 1)]))],
             root=1),
             digest_of(_SCHEMA_IN_TARGET)),
         note="same record name, different exact schema pin")

# Output ports.
_TWO_OUT_TARGET = ALL3_TARGET
negative("output/missing-port", "subflow_output_port_mismatch",
         [_TWO_OUT_TARGET],
         caller_pinning(doc(
             types=[prim(1, "int64"), prim(2, "string")],
             run_parameters=[sir.rp(1, "who", 2, 1)],
             values=[sir.slot(1, 2), sir.slot(9, 1)],
             expressions=[sir.expr(1, 2, {"value_slot": 1}),
                          sir.expr(2, 1, {"constant": {"int64": 5}})],
             resources=[sir.resource(1, USER_REF)],
             computations=[sir.comp(1, [sir.obind(2, 9)],
                                    call_body(inputs=[sir.ibind(1, 2),
                                                      sir.ibind(2, 1)],
                                              resources=[sir.rbind(1, 1)]))],
             root=1),
             digest_of(_TWO_OUT_TARGET)),
         note="the caller declares only output port 2")
negative("output/extra-port", "subflow_output_port_mismatch",
         [BASIC_TARGET],
         caller_pinning(doc(
             types=[prim(1, "int64")],
             values=[sir.slot(9, 1), sir.slot(10, 1)],
             expressions=[sir.expr(1, 1, {"constant": {"int64": 7}})],
             computations=[sir.comp(1, [sir.obind(1, 9), sir.obind(2, 10)],
                                    call_body(inputs=[sir.ibind(1, 1)]))],
             root=1),
             digest_of(BASIC_TARGET)),
         note="the target has no output port 2")
negative("output/type-mismatch", "subflow_output_type_mismatch",
         [BASIC_TARGET],
         caller_pinning(doc(
             types=[prim(1, "int64"), prim(2, "string")],
             values=[sir.slot(9, 2)],
             expressions=[sir.expr(1, 1, {"constant": {"int64": 7}})],
             computations=[sir.comp(1, [sir.obind(1, 9)],
                                    call_body(inputs=[sir.ibind(1, 1)]))],
             root=1),
             digest_of(BASIC_TARGET)),
         note="int64 output bound over a string slot")
_OPT_OUT_TARGET = doc(
    types=[prim(1, "int64"), prim(2, "string"), opt(8, 1)],
    run_parameters=[sir.rp(1, "limit", 1, 6)],
    values=[sir.slot(6, 1)],
    expressions=[sir.expr(1, 1, {"value_slot": 6})],
    workflow_outputs=[{"name": "maybe", "expression": 1}],
    computations=[sir.comp(1, [], await_root())],
    root=1,
)
negative("output/nested-type-mismatch", "subflow_output_type_mismatch",
         [_OPT_OUT_TARGET],
         caller_pinning(doc(
             types=[prim(1, "int64"), prim(2, "string"), opt(12, 2)],
             values=[sir.slot(9, 12)],
             expressions=[sir.expr(1, 1, {"constant": {"int64": 7}})],
             computations=[sir.comp(1, [sir.obind(1, 9)],
                                    call_body(inputs=[sir.ibind(1, 1)]))],
             root=1),
             digest_of(_OPT_OUT_TARGET)),
         note="option<int64> vs option<string>")

# Resources.
negative("resource/missing-port", "subflow_resource_port_mismatch",
         [RESOURCES_TARGET],
         caller_pinning(doc(
             types=[prim(1, "int64")],
             resources=[sir.resource(1, USER_REF), sir.resource(2, STATUS_REF)],
             expressions=[sir.expr(1, 1, {"constant": {"int64": 0}})],
             computations=[sir.comp(1, [], call_body(resources=[
                 sir.rbind(1, 1)]))],
             root=1),
             digest_of(RESOURCES_TARGET)),
         note="resource port 1 (status) is never bound")
negative("resource/extra-port", "subflow_resource_port_mismatch",
         [RESOURCES_TARGET],
         caller_pinning(doc(
             types=[prim(1, "int64")],
             resources=[sir.resource(1, USER_REF), sir.resource(2, STATUS_REF)],
             expressions=[sir.expr(1, 1, {"constant": {"int64": 0}})],
             computations=[sir.comp(1, [], call_body(resources=[
                 sir.rbind(1, 1), sir.rbind(2, 2), sir.rbind(3, 1)]))],
             root=1),
             digest_of(RESOURCES_TARGET)),
         note="the target has no resource port 3")
negative("resource/type-mismatch", "subflow_resource_type_mismatch",
         [RESOURCES_TARGET],
         caller_pinning(doc(
             types=[prim(1, "int64")],
             resources=[sir.resource(1, STATUS_REF)],
             expressions=[sir.expr(1, 1, {"constant": {"int64": 0}})],
             computations=[sir.comp(1, [], call_body(resources=[
                 sir.rbind(1, 1), sir.rbind(2, 1)]))],
             root=1),
             digest_of(RESOURCES_TARGET)),
         note="port 2 requires User, the bound slot requires Status")
negative("resource/schema-digest-mismatch", "subflow_resource_type_mismatch",
         [RESOURCES_TARGET],
         caller_pinning(doc(
             types=[prim(1, "int64")],
             resources=[sir.resource(1, V2_USER_REF)],
             expressions=[sir.expr(1, 1, {"constant": {"int64": 0}})],
             computations=[sir.comp(1, [], call_body(resources=[
                 sir.rbind(1, 1), sir.rbind(2, 1)]))],
             root=1),
             digest_of(RESOURCES_TARGET)),
         note="port 2 requires User@v1, the bound slot requires User@v2")


# --- Equivalence -------------------------------------------------------------

def shuffled(document):
    """Reorders every non-semantic table (canonical order must not move)."""
    member = copy.deepcopy(document)
    member["run_parameters"] = list(reversed(member["run_parameters"]))
    member["values"] = list(reversed(member["values"]))
    member["expressions"] = list(reversed(member["expressions"]))
    member["resources"] = list(reversed(member["resources"]))
    member["workflow_outputs"] = list(reversed(member["workflow_outputs"]))
    member["computations"] = sorted(
        member["computations"], key=lambda c: -c["id"])
    return member


_TABLE_ORDER_BASE = ALL3_TARGET
_TABLE_ORDER_MEMBER = shuffled(_TABLE_ORDER_BASE)
_table_order_interface = derive_interface(_TABLE_ORDER_BASE)
assert digest_of(_TABLE_ORDER_BASE) == digest_of(_TABLE_ORDER_MEMBER)

# Different local type DAG numbering, identical public interface shape.
_TYPE_ID_A = NESTED_TARGET
_TYPE_ID_B = doc(
    types=[prim(31, "int64"), prim(32, "string"), opt(33, 31), lst(34, 33),
           mapstr(35, 34)],
    run_parameters=[sir.rp(1, "matrix", 35, 6)],
    values=[sir.slot(6, 35)],
    expressions=[sir.expr(1, 35, {"value_slot": 6})],
    workflow_outputs=[{"name": "out", "expression": 1}],
    computations=[sir.comp(1, [], await_root())],
    root=1,
)
assert digest_of(_TYPE_ID_A) != digest_of(_TYPE_ID_B)
_interface_a = derive_interface(_TYPE_ID_A)
_interface_b = derive_interface(_TYPE_ID_B)
assert _interface_a == _interface_b

EQUIVALENCE = {
    "table_order_groups": [{
        "case_id": "equivalence/target-table-order",
        "note": ("reordering every non-semantic table changes neither the "
                 "SemanticIrDigest nor the derived interface"),
        "base": _TABLE_ORDER_BASE,
        "member": _TABLE_ORDER_MEMBER,
        "expected_digest": digest_of(_TABLE_ORDER_BASE),
        "expected_interface": _table_order_interface,
    }],
    "type_id_groups": [{
        "case_id": "equivalence/local-type-ids-differ",
        "note": ("two targets with different type DAG numbering: digests "
                 "differ, the derived interface shape is identical"),
        "target_a": _TYPE_ID_A,
        "target_b": _TYPE_ID_B,
        "expected_digest_a": digest_of(_TYPE_ID_A),
        "expected_digest_b": digest_of(_TYPE_ID_B),
        "expected_interface": _interface_a,
    }],
    "port_order_proofs": [{
        "case_id": "equivalence/port-order-canonical-keys",
        "note": ("input ports follow RunParameterId order, output ports the "
                 "canonical NFC-name order and resource ports the "
                 "ResourceSlotId order, regardless of table order"),
        "target": ALL3_TARGET,
        "expected_inputs": [{"port_no": 1, "parameter_id": 2},
                            {"port_no": 2, "parameter_id": 4}],
        "expected_outputs": [{"port_no": 1, "name": "count"},
                             {"port_no": 2, "name": "greeting"}],
        "expected_resources": [{"port_no": 1, "source_resource_slot": 5}],
    }],
}


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
    for case in POSITIVE_CASES:
        # Re-run the oracle over the frozen cases: the caller must check
        # clean against its own targets.
        interfaces_by_hex = {digest_of(target): derive_interface(target)
                             for target in case["targets"]}
        check_subflows(case["caller"], interfaces_by_hex)
        assert sorted(interfaces_by_hex) == case["frozen_target_digests"]
    print(f"validated {len(POSITIVE_CASES)} positive cases")

    for case in NEGATIVE_CASES:
        interfaces_by_hex = {digest_of(target): derive_interface(target)
                             for target in case["targets"]}
        try:
            check_subflows(case["caller"], interfaces_by_hex)
        except SubflowError as error:
            assert error.kind == case["error_kind"], (
                f"{case['case_id']}: expected {case['error_kind']}, "
                f"got {error.kind}")
        else:
            raise AssertionError(f"{case['case_id']} unexpectedly clean")
    print(f"validated {len(NEGATIVE_CASES)} negative cases")

    # Port-order proof re-derivation: the port ordinals map onto the
    # target-local ids exactly in the frozen canonical key orders.
    for proof in EQUIVALENCE["port_order_proofs"]:
        target = proof["target"]
        interface = derive_interface(target)
        parameters = sorted(target["run_parameters"], key=lambda p: p["id"])
        assert [(port["port_no"], parameter["id"])
                for port, parameter in zip(interface["inputs"], parameters)] \
            == [(p["port_no"], p["parameter_id"])
                for p in proof["expected_inputs"]]
        assert [(p["port_no"], p["name"]) for p in interface["outputs"]] \
            == [(p["port_no"], p["name"]) for p in proof["expected_outputs"]]
        assert [(p["port_no"], p["source_resource_slot"])
                for p in interface["resources"]] \
            == [(p["port_no"], p["source_resource_slot"])
                for p in proof["expected_resources"]]
    print("port order proofs hold")

    write_json("spec/conformance/subflow-interface/positive/"
               "subflow-interface-positive.json", {
        "kind": "agentflow.conformance.subflow-interface.positive",
        "corpus_revision": CORPUS_REVISION,
        "note": (
            "NON_NORMATIVE CONFORMANCE PROJECTION. Frozen Phase 3D positive "
            "cases: targets are Phase 3B+3C-valid documents, the caller "
            "checks clean against the explicit universe, target digests are "
            "computed from the targets themselves and the derived "
            "SubflowInterface projections are frozen. Generated by the "
            "independent Python implementation (this script)."
        ),
        "fixture_schemas": FIXTURE_SCHEMAS,
        "cases": POSITIVE_CASES,
    })

    write_json("spec/conformance/subflow-interface/negative/"
               "subflow-interface-negative.json", {
        "kind": "agentflow.conformance.subflow-interface.negative",
        "corpus_revision": CORPUS_REVISION,
        "note": (
            "NON_NORMATIVE CONFORMANCE PROJECTION. Frozen Phase 3D "
            "negatives: every caller and target passes Phase 3B and the "
            "Phase 3C analysis; the rejection comes from the subflow "
            "interface check alone."
        ),
        "fixture_schemas": FIXTURE_SCHEMAS,
        "cases": NEGATIVE_CASES,
    })

    write_json("spec/conformance/subflow-interface/equivalence/"
               "subflow-interface-equivalence.json", {
        "kind": "agentflow.conformance.subflow-interface.equivalence",
        "corpus_revision": CORPUS_REVISION,
        "note": (
            "NON_NORMATIVE CONFORMANCE PROJECTION. Equivalence properties: "
            "table order never moves the digest or the interface; local "
            "type numbering never changes the derived interface shape; "
            "port ordinals follow the frozen canonical key orders."
        ),
        "fixture_schemas": FIXTURE_SCHEMAS,
        **EQUIVALENCE,
    })


if __name__ == "__main__":
    main()
