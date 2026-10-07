#!/usr/bin/env python3
"""Freezes the Phase 3C semantic-dataflow conformance corpus.

INDEPENDENT implementation of the scoped typed dataflow rules
(spec/semantic/semantic-ir.md, scoped-dataflow section): expression value-slot
dependency closure, per-use-site value availability, producer dominance,
branch isolation, loop-local lifetimes, the workflow output boundary and
resource lexical visibility. The Rust analyzer in crates/af-ir
(`analyze_dataflow`) must agree with the frozen expectations; expected values
are never derived from the Rust code.

The corpus never carries a digest: the analysis is derived-only and never
enters the SemanticIrDigest. All documents are Phase 3B-valid by construction
(each negative is re-checked with the Phase 3B validator to prove the failure
belongs to the dataflow layer alone).

Run from the repository root:  python tools/gen-semantic-dataflow-corpus.py
"""

import copy
import importlib.util
import json
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CORPUS_REVISION = 1

# The Phase 3B module (document construction helpers and the frozen golden
# corpus) is reused for document plumbing only; every dataflow rule below is
# implemented independently in this file.
_spec = importlib.util.spec_from_file_location(
    "semantic_ir_corpus", Path(__file__).resolve().parent / "gen-semantic-ir-corpus.py")
sir = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(sir)

LIST_STRING = sir.LIST_STRING
USER_REF = sir.USER_REF

# One extra type node: id 9 = list<list<string>>.
LIST_LIST_STRING = 9
MATRIX_TYPES = sir.PRIMITIVE_TYPES + [{"id": LIST_LIST_STRING,
                                       "body": {"list": LIST_STRING}}]


# ---------------------------------------------------------------------------
# Independent Phase 3C analysis oracle
# ---------------------------------------------------------------------------

class AvailabilityError(Exception):
    """Raised when a use site reads a value that is not available."""

    def __init__(self, kind, reason, detail):
        super().__init__(f"{kind}: {detail}")
        self.kind = kind
        self.reason = reason
        self.detail = detail


def expression_deps(expressions_by_id, expr_id, memo):
    """The transitive value-slot closure of one expression.

    Edges mirror the closed Phase 3A expression set (value_slot reads; every
    other body contributes its child expressions). The expression graph is a
    DAG per Phase 3A, so the recursion is total.
    """
    if expr_id in memo:
        return memo[expr_id]
    body = expressions_by_id[expr_id]["body"]
    opcode, payload = next(iter(body.items()))
    slots = set()

    def go(inner):
        slots.update(expression_deps(expressions_by_id, inner, memo))

    if opcode == "value_slot":
        slots.add(payload)
    elif opcode == "record_construct":
        for field in payload["fields"]:
            go(field["expr_id"])
    elif opcode == "variant_construct":
        go(payload["payload_expr"])
    elif opcode == "list_construct":
        for element in payload["elements"]:
            go(element)
    elif opcode == "map_construct":
        for entry in payload["entries"]:
            go(entry["value_expr"])
    elif opcode == "option_some":
        go(payload["payload_expr"])
    elif opcode == "field_get":
        go(payload["record"])
    elif opcode in ("equals", "and", "or"):
        go(payload["left"])
        go(payload["right"])
    elif opcode == "not":
        go(payload["operand"])
    elif opcode == "if_then_else":
        go(payload["condition"])
        go(payload["then_expr"])
        go(payload["else_expr"])
    # constant / enum_value / option_none reference nothing.
    memo[expr_id] = slots
    return slots


def analyze(document):
    """Runs the scoped dataflow analysis; returns the frozen observations.

    Raises AvailabilityError with the stable kind and reason on the first
    unavailable use site (deterministic: slots ascending, children in
    sequence order / ordinal order, workflow outputs by name).
    """
    computations = {c["id"]: c for c in document["computations"]}
    expressions = {e["id"]: e for e in document["expressions"]}
    parents = {}
    for computation in computations.values():
        for child in sir.owned_children(computation):
            parents[child] = computation["id"]

    producers = {}
    for parameter in document["run_parameters"]:
        producers[parameter["value_slot"]] = ("run_parameter", parameter["id"])
    for computation in computations.values():
        for output in computation["outputs"]:
            producers[output["value_slot"]] = (
                "computation_output", computation["id"], output["port_no"])
        opcode, payload = sir.body_opcode(computation)
        if opcode == "iterate" and "foreach" in payload:
            producers[payload["foreach"]["item_slot"]] = (
                "iterate_item", computation["id"])
            for carry in payload["foreach"]["carry"]:
                producers[carry["slot"]] = ("iterate_carry", computation["id"])
        elif opcode == "iterate":
            for carry in payload["while"]["carry"]:
                producers[carry["slot"]] = ("iterate_carry", computation["id"])
    memo = {}

    def deps(expr_id):
        return expression_deps(expressions, expr_id, memo)

    def is_ancestor(ancestor, node):
        current = node
        while current in parents:
            current = parents[current]
            if current == ancestor:
                return True
        return False

    def child_under(root, node):
        current = node
        while current in parents:
            if parents[current] == root:
                return current
            current = parents[current]
        return None

    def lowest_common_ancestor(a, b):
        ancestors = set()
        current = a
        while True:
            ancestors.add(current)
            if current not in parents:
                break
            current = parents[current]
        current = b
        while current not in ancestors:
            current = parents[current]
        return current

    def classify(producer, use_comp):
        if use_comp is None:
            # A workflow output env can only miss loop locals (they escape
            # their iterate) or values outside the root boundary.
            if producer[0] in ("iterate_item", "iterate_carry"):
                return "iterate_local_outside"
            return "outside_owner_boundary"
        kind_ = producer[0]
        if kind_ == "run_parameter":
            return "before_producer"  # unreachable: run params are global
        if kind_ in ("iterate_item", "iterate_carry"):
            return "iterate_local_outside"
        producer_comp = producer[1]
        if producer_comp == use_comp:
            return "before_producer"
        if is_ancestor(producer_comp, use_comp):
            return "parent_output_not_yet_available"
        if is_ancestor(use_comp, producer_comp):
            return "before_producer"
        lowest = lowest_common_ancestor(use_comp, producer_comp)
        use_side = child_under(lowest, use_comp)
        producer_side = child_under(lowest, producer_comp)
        if use_side is None or producer_side is None or use_side == producer_side:
            return "outside_owner_boundary"
        opcode, _payload = sir.body_opcode(computations[lowest])
        if opcode in ("select", "parallel", "race", "handle"):
            return "cross_branch"
        if opcode == "sequence":
            children = computations[lowest]["body"]["sequence"]["children"]
            if producer_side in children \
                    and children.index(producer_side) > children.index(use_side):
                return "before_producer"
        return "outside_owner_boundary"

    def exports(comp_id):
        return {o["value_slot"] for o in computations[comp_id]["outputs"]}

    def check_use(expr_id, env, use_comp, site):
        missing = sorted(deps(expr_id) - env)
        if missing:
            slot = missing[0]
            reason = classify(producers[slot], use_comp)
            raise AvailabilityError(
                "value_not_available", reason,
                f"value slot {slot} is not available at {site}")

    def check_resources(uses, env, opcode, comp_id):
        for binding in sorted(uses, key=lambda b: b["port_no"]):
            if binding["resource_slot"] not in env:
                raise AvailabilityError(
                    "resource_not_visible", None,
                    f"computation {comp_id}: {opcode} resource port "
                    f"{binding['port_no']} names resource slot "
                    f"{binding['resource_slot']}, which is not lexically "
                    "visible here")

    def visit(comp_id, env, renv):
        entry_values[comp_id] = sorted(env)
        entry_resources[comp_id] = sorted(renv)
        computation = computations[comp_id]
        opcode, payload = sir.body_opcode(computation)
        if opcode == "invoke":
            for binding in sorted(payload["inputs"], key=lambda b: b["port_no"]):
                check_use(binding["expression"], env, comp_id,
                          f"invoke input (computation {comp_id})")
            check_resources(payload["resources"], renv, "invoke", comp_id)
        elif opcode == "call_subflow":
            for binding in sorted(payload["inputs"], key=lambda b: b["port_no"]):
                check_use(binding["expression"], env, comp_id,
                          f"call_subflow input (computation {comp_id})")
            check_resources(payload["resources"], renv, "call_subflow", comp_id)
        elif opcode == "await":
            check_use(payload["correlation"], env, comp_id,
                      f"await correlation (computation {comp_id})")
        elif opcode == "sequence":
            inner = set(env)
            for child in payload["children"]:
                visit(child, inner, renv)
                inner |= exports(child)
        elif opcode == "select":
            check_use(payload["selector"], env, comp_id,
                      f"select selector (computation {comp_id})")
            for case in sorted(payload["cases"], key=lambda c: c["ordinal"]):
                visit(case["child"], env, renv)
        elif opcode == "parallel":
            for branch in sorted(payload["branches"], key=lambda b: b["ordinal"]):
                visit(branch["child"], env, renv)
        elif opcode == "race":
            for branch in sorted(payload["branches"], key=lambda b: b["ordinal"]):
                visit(branch["child"], env, renv)
        elif opcode == "iterate" and "foreach" in payload:
            loop = payload["foreach"]
            check_use(loop["source"], env, comp_id,
                      f"foreach source (computation {comp_id})")
            for carry in sorted(loop["carry"], key=lambda c: c["slot"]):
                check_use(carry["initial"], env, comp_id,
                          f"iterate carry initial (computation {comp_id})")
            body_env = set(env) | {loop["item_slot"]} | {
                c["slot"] for c in loop["carry"]}
            visit(loop["body"], body_env, renv)
        elif opcode == "iterate":
            loop = payload["while"]
            for carry in sorted(loop["carry"], key=lambda c: c["slot"]):
                check_use(carry["initial"], env, comp_id,
                          f"iterate carry initial (computation {comp_id})")
            inner = set(env) | {c["slot"] for c in loop["carry"]}
            check_use(loop["guard"], inner, comp_id,
                      f"while guard (computation {comp_id})")
            visit(loop["body"], inner, renv)
        elif opcode == "retry":
            visit(payload["child"], env, renv)
        elif opcode == "timeout":
            check_use(payload["timeout"], env, comp_id,
                      f"timeout duration (computation {comp_id})")
            visit(payload["child"], env, renv)
        elif opcode == "handle":
            visit(payload["child"], env, renv)
            for clause in sorted(payload["clauses"], key=lambda c: c["ordinal"]):
                visit(clause["handler"], env, renv)
        elif opcode == "scope":
            listed = set(payload["resources"])
            for resource in sorted(listed):
                if resource not in renv:
                    raise AvailabilityError(
                        "resource_not_visible", None,
                        f"computation {comp_id}: scope declares resource slot "
                        f"{resource}, which is not visible in its inherited "
                        "resource environment")
            visit(payload["child"], env, listed)
        elif opcode == "compensation_scope":
            visit(payload["child"], env, renv)
        else:
            raise AssertionError(f"unknown opcode {opcode!r}")

    entry_values = {}
    entry_resources = {}
    root_values = {p["value_slot"] for p in document["run_parameters"]}
    root_resources = {r["id"] for r in document["resources"]}
    visit(document["root"], root_values, root_resources)

    exit_values = set(root_values) | exports(document["root"])
    for output in sorted(document["workflow_outputs"],
                         key=lambda o: unicodedata.normalize("NFC", o["name"]).encode()):
        check_use(output["expression"], exit_values, None,
                  f"workflow output {output['name']!r}")

    return {
        "entry_values": entry_values,
        "entry_resources": entry_resources,
        "workflow_exit_values": sorted(exit_values),
    }


# ---------------------------------------------------------------------------
# Document construction (serde/corpus JSON form; Phase 3B-valid by design)
# ---------------------------------------------------------------------------

def rd(expr_id, slot, result_type):
    """A pure reader of one value slot."""
    return sir.expr(expr_id, result_type, {"value_slot": slot})


def inv(comp_id, outputs, inputs=(), resources=()):
    return sir.comp(comp_id, list(outputs),
                    {"invoke": {"contract": 1, "inputs": list(inputs),
                                "resources": list(resources)}})


def csub(comp_id, outputs, inputs=(), resources=()):
    return sir.comp(comp_id, list(outputs),
                    {"call_subflow": {"contract": 2, "inputs": list(inputs),
                                      "resources": list(resources)}})


def comp_of(document, comp_id):
    return next(c for c in document["computations"] if c["id"] == comp_id)


def base_document(computations, root, contracts=None, values=None,
                  run_parameters=None, workflow_outputs=None, expressions=None,
                  types=None):
    """The shared table set (Phase 3B base) with dataflow-specific extras.

    Every defaulted table is copied: mutations in `negative()` append to the
    per-document lists and must never touch the shared Phase 3B bases.
    """
    return sir.doc(
        semantic_refs=[],
        values=list(sir.BASE_VALUES if values is None else values),
        expressions=list(sir.BASE_EXPRESSIONS if expressions is None
                         else expressions),
        resources=list(sir.BASE_RESOURCES),
        contracts=list([] if contracts is None else contracts),
        run_parameters=list(sir.BASE_RUN_PARAMETERS if run_parameters is None
                            else run_parameters),
        workflow_outputs=([{"name": "done", "expression": 2}]
                          if workflow_outputs is None else workflow_outputs),
        computations=computations,
        root=root,
        types=list(sir.PRIMITIVE_TYPES if types is None else types),
    )


def wrapped(builder, *children_after):
    """Re-roots `builder()`'s document under sequence 10 = [old_root, *more].

    The extra child ids are appended to the sequence's children but their
    computations are added by the caller's mutation.
    """
    document = builder()
    old_root = document["root"]
    document["computations"].append(
        sir.comp(10, [], {"sequence": {"children": [old_root]}}))
    document["root"] = 10
    if children_after:
        comp_of(document, 10)["body"]["sequence"]["children"].extend(children_after)
    return document


def matrix_run_parameter(values, run_parameters, base_slot=6):
    """Adds the list<list<string>> run parameter producing `base_slot`."""
    values.append(sir.slot(base_slot, LIST_LIST_STRING))
    run_parameters.append(sir.rp(6, "matrix", LIST_LIST_STRING, base_slot))


POSITIVE_CASES = []
NEGATIVE_CASES = []


def positive(case_id, document, note=""):
    POSITIVE_CASES.append({"case_id": case_id, "note": note, "document": document})


def negative(case_id, reason, mutate, base, note=""):
    document = base()
    mutate(document)
    NEGATIVE_CASES.append({
        "case_id": case_id,
        "note": note,
        "document": document,
        "error_kind": "value_not_available" if reason else "resource_not_visible",
        "rejected_at": "dataflow",
        **({"reason": reason} if reason else {}),
    })


# --- Positives: value availability -----------------------------------------

positive("value/run-parameter-deep-in-nested", base_document(
    computations=[
        sir.comp(1, [], {"sequence": {"children": [2, 5]}}),
        sir.comp(2, [], {"retry": {"child": 3, "delays": [0]}}),
        sir.comp(3, [], {"timeout": {"child": 4, "timeout": 5}}),
        inv(4, [], inputs=[sir.ibind(1, 1)]),
        sir.comp(5, [], sir.AWAIT_USER),
    ],
    root=1,
    contracts=[sir.component_contract()],
), "a run parameter dominates every computation")

positive("sequence/previous-sibling-output", base_document(
    computations=[
        sir.comp(1, [], {"sequence": {"children": [2, 3, 4]}}),
        inv(2, [sir.obind(1, 6)]),
        inv(3, [sir.obind(1, 7)], inputs=[sir.ibind(1, 10)]),
        inv(4, [], inputs=[sir.ibind(1, 10), sir.ibind(2, 11)]),
    ],
    root=1,
    contracts=[sir.component_contract()],
    values=sir.BASE_VALUES + [sir.slot(6, 1), sir.slot(7, 1)],
    expressions=sir.BASE_EXPRESSIONS + [rd(10, 6, 1), rd(11, 7, 1)],
), "EntryValues(Ci) = E + exports of all earlier direct siblings")

positive("sequence/nested-export-via-own-output", base_document(
    computations=[
        sir.comp(1, [], {"sequence": {"children": [2, 4]}}),
        sir.comp(2, [sir.obind(1, 7)], {"sequence": {"children": [3]}}),
        inv(3, [sir.obind(1, 6)]),
        inv(4, [], inputs=[sir.ibind(1, 11)]),
    ],
    root=1,
    contracts=[sir.component_contract()],
    values=sir.BASE_VALUES + [sir.slot(6, 1), sir.slot(7, 1)],
    expressions=sir.BASE_EXPRESSIONS + [rd(11, 7, 1)],
), "only the wrapper's own export crosses its boundary")

positive("select/branch-uses-outer-value", base_document(
    computations=[
        sir.comp(1, [], {"select": {"selector": 6, "cases": [
            {"ordinal": 1, "child": 2},
            {"ordinal": 2, "child": 3},
        ]}}),
        inv(2, [], inputs=[sir.ibind(1, 1)]),
        sir.comp(3, [], sir.AWAIT_USER),
    ],
    root=1,
    contracts=[sir.component_contract()],
), "every case starts from the select's entry environment")

positive("parallel/branches-use-outer-immutable", base_document(
    computations=[
        sir.comp(1, [], {"parallel": {
            "mode": "all",
            "branches": [{"ordinal": 1, "child": 2},
                         {"ordinal": 2, "child": 3}],
            "max_inflight": 2,
            "output_bindings": [],
        }}),
        inv(2, [], inputs=[sir.ibind(1, 1)]),
        inv(3, [], inputs=[sir.ibind(1, 1)]),
    ],
    root=1,
    contracts=[sir.component_contract()],
), "both branches read the same run parameter independently")

positive("race/branches-use-outer-value", base_document(
    computations=[
        sir.comp(1, [], {"race": {"branches": [
            {"ordinal": 1, "child": 2},
            {"ordinal": 2, "child": 3},
        ]}}),
        sir.comp(2, [], sir.AWAIT_USER),
        sir.comp(3, [], sir.AWAIT_USER),
    ],
    root=1,
), "race branches share the outer environment without ordering")

positive("foreach/source-and-body-item", base_document(
    computations=[
        sir.comp(1, [], {"iterate": {"foreach": {
            "source": 9,
            "item_slot": 6,
            "body": 2,
            "carry": [],
            "max_inflight": None,
        }}}),
        inv(2, [], inputs=[sir.ibind(1, 12)]),
    ],
    root=1,
    contracts=[sir.component_contract()],
    values=sir.BASE_VALUES + [sir.slot(6, 3)],
    expressions=sir.BASE_EXPRESSIONS + [rd(12, 6, 3)],
), "source is a pre-loop snapshot; the body reads the item slot")

positive("foreach/body-carry", base_document(
    computations=[
        sir.comp(1, [], {"iterate": {"foreach": {
            "source": 9,
            "item_slot": 6,
            "body": 2,
            "carry": [{"slot": 7, "initial": 1, "body_output_port": 1}],
            "max_inflight": None,
        }}}),
        inv(2, [sir.obind(1, 8)], inputs=[sir.ibind(1, 13)]),
    ],
    root=1,
    contracts=[sir.component_contract()],
    values=sir.BASE_VALUES + [sir.slot(6, 3), sir.slot(7, 1), sir.slot(8, 1)],
    expressions=sir.BASE_EXPRESSIONS + [rd(13, 7, 1)],
), "the body sees the item slot and every carry slot")

positive("while/guard-carry", base_document(
    computations=[
        sir.comp(1, [], {"iterate": {"while": {
            "guard": 14,
            "body": 2,
            "carry": [{"slot": 7, "initial": 2, "body_output_port": 1}],
            "max_iterations": 5,
        }}}),
        sir.comp(2, [sir.obind(1, 8)], sir.AWAIT_USER),
    ],
    root=1,
    values=sir.BASE_VALUES + [sir.slot(7, 2), sir.slot(8, 2)],
    expressions=sir.BASE_EXPRESSIONS + [rd(14, 7, 2)],
), "the guard evaluates at the loop header over the carry slots")

positive("while/body-carry", base_document(
    computations=[
        sir.comp(1, [], {"iterate": {"while": {
            "guard": 2,
            "body": 2,
            "carry": [{"slot": 7, "initial": 2, "body_output_port": 1}],
            "max_iterations": 5,
        }}}),
        inv(2, [sir.obind(1, 8)], inputs=[sir.ibind(1, 14)]),
    ],
    root=1,
    contracts=[sir.component_contract()],
    values=sir.BASE_VALUES + [sir.slot(7, 2), sir.slot(8, 2)],
    expressions=sir.BASE_EXPRESSIONS + [rd(14, 7, 2)],
), "the body reads the carry slot")

positive("timeout/duration-outer-value", base_document(
    computations=[
        sir.comp(1, [], {"timeout": {"child": 2, "timeout": 3}}),
        sir.comp(2, [], sir.AWAIT_USER),
    ],
    root=1,
), "the timeout duration is evaluated at the timeout entry")

positive("workflow-output/root-output", base_document(
    computations=[inv(1, [sir.obind(1, 6)])],
    root=1,
    contracts=[sir.component_contract()],
    values=sir.BASE_VALUES + [sir.slot(6, 1)],
    workflow_outputs=[{"name": "result", "expression": 10}],
    expressions=sir.BASE_EXPRESSIONS + [rd(10, 6, 1)],
), "workflow outputs read the root's own export")

positive("workflow-output/run-parameter", base_document(
    computations=[sir.comp(1, [], sir.AWAIT_USER)],
    root=1,
    workflow_outputs=[{"name": "limit_out", "expression": 1}],
), "run parameters live for the whole run, workflow outputs included")


# --- Positives: resource lexical visibility ---------------------------------

positive("resource/root-invoke-uses-resource", base_document(
    computations=[inv(1, [], resources=[sir.rbind(1, 1)])],
    root=1,
    contracts=[sir.component_contract()],
), "EntryResources(root) is every declared resource slot")

positive("resource/scope-child-uses-listed", base_document(
    computations=[
        sir.comp(1, [], {"scope": {"child": 2, "resources": [1]}}),
        inv(2, [], resources=[sir.rbind(1, 1)]),
    ],
    root=1,
    contracts=[sir.component_contract()],
), "the scope child sees exactly the listed set")

positive("resource/nested-scope-narrows", base_document(
    computations=[
        sir.comp(1, [], {"scope": {"child": 2, "resources": [1, 2]}}),
        sir.comp(2, [], {"scope": {"child": 3, "resources": [1]}}),
        inv(3, [], resources=[sir.rbind(1, 1)]),
    ],
    root=1,
    contracts=[sir.component_contract()],
), "a nested scope attenuates its inherited environment")

positive("resource/parallel-branches-share-visible", base_document(
    computations=[
        sir.comp(1, [], {"parallel": {
            "mode": "all",
            "branches": [{"ordinal": 1, "child": 2},
                         {"ordinal": 2, "child": 3}],
            "max_inflight": 2,
            "output_bindings": [],
        }}),
        inv(2, [], resources=[sir.rbind(1, 1)]),
        inv(3, [], resources=[sir.rbind(1, 1)]),
    ],
    root=1,
    contracts=[sir.component_contract()],
), "lexical visibility is shared; concurrency safety is a later layer")

positive("resource/scope-exit-restores-sibling", base_document(
    computations=[
        sir.comp(1, [], {"sequence": {"children": [2, 3]}}),
        sir.comp(2, [], {"scope": {"child": 4, "resources": [1]}}),
        inv(4, [], resources=[sir.rbind(1, 1)]),
        inv(3, [], resources=[sir.rbind(1, 2)]),
    ],
    root=1,
    contracts=[sir.component_contract()],
), "narrowing ends at the scope boundary")


# --- Positives: expression reuse and unused expressions ----------------------

def inside_use_document():
    """Expression 10 reads comp 4's export and is used only inside branch 3."""
    return base_document(
        computations=[
            sir.comp(1, [], {"sequence": {"children": [2]}}),
            sir.comp(2, [], {"select": {"selector": 6, "cases": [
                {"ordinal": 1, "child": 3},
            ]}}),
            sir.comp(3, [], {"sequence": {"children": [4, 5]}}),
            inv(4, [sir.obind(1, 8)]),
            inv(5, [], inputs=[sir.ibind(1, 10)]),
        ],
        root=1,
        contracts=[sir.component_contract()],
        values=sir.BASE_VALUES + [sir.slot(8, 1)],
        expressions=sir.BASE_EXPRESSIONS + [rd(10, 8, 1)],
    )


positive("expression/inside-use-valid", inside_use_document(),
         "one expression, one use site inside the producing branch")

negative(
    "expression/reuse-per-use-site", "outside_owner_boundary",
    lambda d: (d["computations"].append(inv(7, [], inputs=[sir.ibind(1, 10)])),
               comp_of(d, 1)["body"]["sequence"]["children"].append(7)),
    base=inside_use_document,
    note="the second use of expr 10 is outside the owning branch",
)

positive("expression/unused-branch-local-reference", base_document(
    computations=[
        sir.comp(1, [sir.obind(1, 7)], {"select": {"selector": 6, "cases": [
            {"ordinal": 1, "child": 2},
            {"ordinal": 2, "child": 3},
        ]}}),
        inv(2, [sir.obind(1, 6)]),
        inv(3, [sir.obind(1, 8)]),
    ],
    root=1,
    contracts=[sir.component_contract()],
    values=sir.BASE_VALUES + [sir.slot(6, 1), sir.slot(7, 1), sir.slot(8, 1)],
    expressions=sir.BASE_EXPRESSIONS + [rd(20, 6, 1)],
), "expr 20 reads case 1's export but is referenced by no use site: "
   "an expression definition is not an execution")


# --- Negatives: sequence dominance ------------------------------------------

def sequence_chain_document():
    return base_document(
        computations=[
            sir.comp(1, [], {"sequence": {"children": [2, 3, 4]}}),
            inv(2, [sir.obind(1, 6)]),
            inv(3, [sir.obind(1, 7)], inputs=[sir.ibind(1, 10)]),
            inv(4, [], inputs=[sir.ibind(1, 10), sir.ibind(2, 11)]),
        ],
        root=1,
        contracts=[sir.component_contract()],
        values=sir.BASE_VALUES + [sir.slot(6, 1), sir.slot(7, 1)],
        expressions=sir.BASE_EXPRESSIONS + [rd(10, 6, 1), rd(11, 7, 1)],
    )


negative(
    "sequence/future-sibling-output", "before_producer",
    lambda d: comp_of(d, 2)["body"]["invoke"]["inputs"].append(sir.ibind(1, 11)),
    base=sequence_chain_document,
    note="child 1 reads child 2's export before it exists",
)

negative(
    "sequence/self-output-used-as-input", "before_producer",
    lambda d: comp_of(d, 2)["body"]["invoke"]["inputs"].append(sir.ibind(1, 10)),
    base=sequence_chain_document,
    note="an output exists only after its own completion boundary",
)

negative(
    "sequence/grandchild-output-leaks-to-later-sibling",
    "outside_owner_boundary",
    lambda d: None,
    base=lambda: base_document(
        computations=[
            sir.comp(1, [], {"sequence": {"children": [2, 5]}}),
            sir.comp(2, [sir.obind(1, 7)], {"sequence": {"children": [3]}}),
            inv(3, [sir.obind(1, 6)]),
            inv(5, [], inputs=[sir.ibind(1, 10)]),
        ],
        root=1,
        contracts=[sir.component_contract()],
        values=sir.BASE_VALUES + [sir.slot(6, 1), sir.slot(7, 1)],
        expressions=sir.BASE_EXPRESSIONS + [rd(10, 6, 1)],
    ),
    note="the later sibling must read the wrapper's export (slot 7), not slot 6",
)

negative(
    "sequence/parent-output-used-by-child", "parent_output_not_yet_available",
    lambda d: None,
    base=lambda: base_document(
        computations=[
            sir.comp(1, [sir.obind(1, 6)], {"sequence": {"children": [2]}}),
            inv(2, [sir.obind(1, 7)], inputs=[sir.ibind(1, 10)]),
        ],
        root=1,
        contracts=[sir.component_contract()],
        values=sir.BASE_VALUES + [sir.slot(6, 1), sir.slot(7, 1)],
        expressions=sir.BASE_EXPRESSIONS + [rd(10, 6, 1)],
    ),
    note="a parent's output is available only after the parent completes",
)


# --- Negatives: select isolation ---------------------------------------------

negative(
    "select/selector-uses-case-output", "before_producer",
    lambda d: None,
    base=lambda: base_document(
        computations=[
            sir.comp(1, [sir.obind(1, 7)], {"select": {"selector": 10, "cases": [
                {"ordinal": 1, "child": 2},
                {"ordinal": 2, "child": 3},
            ]}}),
            inv(2, [sir.obind(1, 8)]),
            inv(3, [sir.obind(1, 9)]),
        ],
        root=1,
        contracts=[sir.component_contract()],
        values=sir.BASE_VALUES + [sir.slot(7, 1), sir.slot(8, 1), sir.slot(9, 1)],
        expressions=sir.BASE_EXPRESSIONS + [rd(10, 8, 1)],
    ),
    note="the selector is evaluated before any case runs",
)

negative(
    "select/case-cross-reference", "cross_branch",
    lambda d: comp_of(d, 3)["body"]["invoke"]["inputs"].append(sir.ibind(1, 10)),
    base=lambda: base_document(
        computations=[
            sir.comp(1, [sir.obind(1, 7)], {"select": {"selector": 6, "cases": [
                {"ordinal": 1, "child": 2},
                {"ordinal": 2, "child": 3},
            ]}}),
            inv(2, [sir.obind(1, 8)]),
            inv(3, [sir.obind(1, 9)]),
        ],
        root=1,
        contracts=[sir.component_contract()],
        values=sir.BASE_VALUES + [sir.slot(7, 1), sir.slot(8, 1), sir.slot(9, 1)],
        expressions=sir.BASE_EXPRESSIONS + [rd(10, 8, 1)],
    ),
    note="case ordinals are identity, never execution order",
)

negative(
    "select/raw-case-output-used-after-select", "outside_owner_boundary",
    lambda d: None,
    base=lambda: base_document(
        computations=[
            sir.comp(1, [], {"sequence": {"children": [2, 4]}}),
            sir.comp(2, [sir.obind(1, 7)], {"select": {"selector": 6, "cases": [
                {"ordinal": 1, "child": 3},
            ]}}),
            inv(3, [sir.obind(1, 8)]),
            inv(4, [], inputs=[sir.ibind(1, 10)]),
        ],
        root=1,
        contracts=[sir.component_contract()],
        values=sir.BASE_VALUES + [sir.slot(7, 1), sir.slot(8, 1)],
        expressions=sir.BASE_EXPRESSIONS + [rd(10, 8, 1)],
    ),
    note="outside the select only its own export (slot 7) is visible",
)


# --- Negatives: parallel / race isolation ------------------------------------

negative(
    "parallel/cross-branch-value", "cross_branch",
    lambda d: comp_of(d, 3)["body"]["invoke"]["inputs"].append(sir.ibind(1, 10)),
    base=lambda: base_document(
        computations=[
            sir.comp(1, [], {"parallel": {
                "mode": "all",
                "branches": [{"ordinal": 1, "child": 2},
                             {"ordinal": 2, "child": 3}],
                "max_inflight": 2,
                "output_bindings": [],
            }}),
            inv(2, [sir.obind(1, 6)]),
            inv(3, []),
        ],
        root=1,
        contracts=[sir.component_contract()],
        values=sir.BASE_VALUES + [sir.slot(6, 1)],
        expressions=sir.BASE_EXPRESSIONS + [rd(10, 6, 1)],
    ),
    note="branches run concurrently; neither dominates the other",
)

negative(
    "parallel/raw-branch-output-used-after-parallel", "outside_owner_boundary",
    lambda d: None,
    base=lambda: base_document(
        computations=[
            sir.comp(1, [], {"sequence": {"children": [2, 4]}}),
            sir.comp(2, [sir.obind(1, 7)], {"parallel": {
                "mode": "fail_fast",
                "branches": [{"ordinal": 1, "child": 3}],
                "max_inflight": 2,
                "output_bindings": [sir.outb(1, 1, 1)],
            }}),
            inv(3, [sir.obind(1, 6)]),
            inv(4, [], inputs=[sir.ibind(1, 10)]),
        ],
        root=1,
        contracts=[sir.component_contract()],
        values=sir.BASE_VALUES + [sir.slot(6, 1), sir.slot(7, 1)],
        expressions=sir.BASE_EXPRESSIONS + [rd(10, 6, 1)],
    ),
    note="only the parallel's mapped parent export (slot 7) is visible outside",
)

negative(
    "race/cross-branch-value", "cross_branch",
    lambda d: comp_of(d, 3)["body"]["invoke"]["inputs"].append(sir.ibind(1, 10)),
    base=lambda: base_document(
        computations=[
            sir.comp(1, [sir.obind(1, 7)], {"race": {"branches": [
                {"ordinal": 1, "child": 2},
                {"ordinal": 2, "child": 3},
            ]}}),
            inv(2, [sir.obind(1, 6)]),
            inv(3, [sir.obind(1, 9)]),
        ],
        root=1,
        contracts=[sir.component_contract()],
        values=sir.BASE_VALUES + [sir.slot(6, 1), sir.slot(7, 1), sir.slot(9, 1)],
        expressions=sir.BASE_EXPRESSIONS + [rd(10, 6, 1)],
    ),
    note="FIRST_TERMINAL gives no stable branch order",
)

negative(
    "race/raw-loser-or-winner-output-used-after-race", "outside_owner_boundary",
    lambda d: None,
    base=lambda: base_document(
        computations=[
            sir.comp(1, [], {"sequence": {"children": [2, 4]}}),
            sir.comp(2, [sir.obind(1, 7)], {"race": {"branches": [
                {"ordinal": 1, "child": 3},
                {"ordinal": 2, "child": 5},
            ]}}),
            inv(3, [sir.obind(1, 6)]),
            inv(4, [], inputs=[sir.ibind(1, 10)]),
            inv(5, [sir.obind(1, 9)]),
        ],
        root=1,
        contracts=[sir.component_contract()],
        values=sir.BASE_VALUES + [sir.slot(6, 1), sir.slot(7, 1), sir.slot(9, 1)],
        expressions=sir.BASE_EXPRESSIONS + [rd(10, 6, 1)],
    ),
    note="no branch output — winner or loser — escapes the race raw",
)


# --- Negatives: retry / timeout / scope / compensation scope -----------------

negative(
    "retry/raw-child-output-used-after-retry", "outside_owner_boundary",
    lambda d: None,
    base=lambda: base_document(
        computations=[
            sir.comp(1, [], {"sequence": {"children": [2, 4]}}),
            sir.comp(2, [sir.obind(1, 7)],
                     {"retry": {"child": 3, "delays": [0]}}),
            inv(3, [sir.obind(1, 6)]),
            inv(4, [], inputs=[sir.ibind(1, 10)]),
        ],
        root=1,
        contracts=[sir.component_contract()],
        values=sir.BASE_VALUES + [sir.slot(6, 1), sir.slot(7, 1)],
        expressions=sir.BASE_EXPRESSIONS + [rd(10, 6, 1)],
    ),
    note="outside the retry only its own export (slot 7) is visible",
)

negative(
    "timeout/duration-uses-child-output", "before_producer",
    lambda d: None,
    base=lambda: base_document(
        computations=[
            sir.comp(1, [sir.obind(1, 7)],
                     {"timeout": {"child": 2, "timeout": 15}}),
            inv(2, [sir.obind(1, 6)]),
        ],
        root=1,
        contracts=[sir.component_contract()],
        values=sir.BASE_VALUES + [sir.slot(6, 4), sir.slot(7, 4)],
        expressions=sir.BASE_EXPRESSIONS + [rd(15, 6, 4)],
    ),
    note="the timeout duration is evaluated at the timeout entry",
)

negative(
    "timeout/raw-child-output-used-after-timeout", "outside_owner_boundary",
    lambda d: None,
    base=lambda: base_document(
        computations=[
            sir.comp(1, [], {"sequence": {"children": [2, 4]}}),
            sir.comp(2, [sir.obind(1, 7)],
                     {"timeout": {"child": 3, "timeout": 5}}),
            inv(3, [sir.obind(1, 6)]),
            inv(4, [], inputs=[sir.ibind(1, 10)]),
        ],
        root=1,
        contracts=[sir.component_contract()],
        values=sir.BASE_VALUES + [sir.slot(6, 1), sir.slot(7, 1)],
        expressions=sir.BASE_EXPRESSIONS + [rd(10, 6, 1)],
    ),
    note="outside the timeout only its own export (slot 7) is visible",
)

negative(
    "scope/raw-child-output-used-after-scope", "outside_owner_boundary",
    lambda d: None,
    base=lambda: base_document(
        computations=[
            sir.comp(1, [], {"sequence": {"children": [2, 4]}}),
            sir.comp(2, [sir.obind(1, 7)],
                     {"scope": {"child": 3, "resources": []}}),
            inv(3, [sir.obind(1, 6)]),
            inv(4, [], inputs=[sir.ibind(1, 10)]),
        ],
        root=1,
        contracts=[sir.component_contract()],
        values=sir.BASE_VALUES + [sir.slot(6, 1), sir.slot(7, 1)],
        expressions=sir.BASE_EXPRESSIONS + [rd(10, 6, 1)],
    ),
    note="a scope does not widen value visibility either",
)

negative(
    "comp-scope/raw-child-output-used-after-comp-scope", "outside_owner_boundary",
    lambda d: None,
    base=lambda: base_document(
        computations=[
            sir.comp(1, [], {"sequence": {"children": [2, 4]}}),
            sir.comp(2, [sir.obind(1, 7)],
                     {"compensation_scope": {"child": 3}}),
            inv(3, [sir.obind(1, 6)]),
            inv(4, [], inputs=[sir.ibind(1, 10)]),
        ],
        root=1,
        contracts=[sir.component_contract()],
        values=sir.BASE_VALUES + [sir.slot(6, 1), sir.slot(7, 1)],
        expressions=sir.BASE_EXPRESSIONS + [rd(10, 6, 1)],
    ),
    note="compensation scope never erases history nor widens visibility",
)


# --- Negatives: handle isolation ---------------------------------------------

negative(
    "handle/handler-uses-protected-child-output", "cross_branch",
    lambda d: comp_of(d, 3)["body"]["invoke"]["inputs"].append(sir.ibind(1, 10)),
    base=lambda: base_document(
        computations=[
            sir.comp(1, [sir.obind(1, 7)], {"handle": {"child": 2, "clauses": [
                {"ordinal": 1,
                 "match": {"failure": {"failure_type": USER_REF}},
                 "handler": 3},
            ]}}),
            inv(2, [sir.obind(1, 6)]),
            inv(3, [sir.obind(1, 8)]),
        ],
        root=1,
        contracts=[sir.component_contract()],
        values=sir.BASE_VALUES + [sir.slot(6, 1), sir.slot(7, 1), sir.slot(8, 1)],
        expressions=sir.BASE_EXPRESSIONS + [rd(10, 6, 1)],
    ),
    note="the protected child may have failed; its outputs do not exist",
)

negative(
    "handle/handler-cross-reference", "cross_branch",
    lambda d: comp_of(d, 4)["body"]["invoke"]["inputs"].append(sir.ibind(1, 11)),
    base=lambda: base_document(
        computations=[
            sir.comp(1, [sir.obind(1, 7)], {"handle": {"child": 2, "clauses": [
                {"ordinal": 1,
                 "match": {"failure": {"failure_type": USER_REF}},
                 "handler": 3},
                {"ordinal": 2, "match": {"timed_out": None}, "handler": 4},
            ]}}),
            inv(2, [sir.obind(1, 6)]),
            inv(3, [sir.obind(1, 8)]),
            inv(4, [sir.obind(1, 9)]),
        ],
        root=1,
        contracts=[sir.component_contract()],
        values=sir.BASE_VALUES + [sir.slot(6, 1), sir.slot(7, 1),
                                  sir.slot(8, 1), sir.slot(9, 1)],
        expressions=sir.BASE_EXPRESSIONS + [rd(11, 8, 1)],
    ),
    note="handler ordinals are identity, never execution order",
)

negative(
    "handle/raw-handler-output-used-after-handle", "outside_owner_boundary",
    lambda d: None,
    base=lambda: base_document(
        computations=[
            sir.comp(1, [], {"sequence": {"children": [2, 5]}}),
            sir.comp(2, [sir.obind(1, 7)], {"handle": {"child": 3, "clauses": [
                {"ordinal": 1,
                 "match": {"failure": {"failure_type": USER_REF}},
                 "handler": 4},
            ]}}),
            inv(3, [sir.obind(1, 6)]),
            inv(4, [sir.obind(1, 8)]),
            inv(5, [], inputs=[sir.ibind(1, 11)]),
        ],
        root=1,
        contracts=[sir.component_contract()],
        values=sir.BASE_VALUES + [sir.slot(6, 1), sir.slot(7, 1), sir.slot(8, 1)],
        expressions=sir.BASE_EXPRESSIONS + [rd(11, 8, 1)],
    ),
    note="outside the handle only its own export (slot 7) is visible",
)


# --- Negatives: iterate loop locals ------------------------------------------

def matrix_foreach_document():
    """foreach over list<list<string>>; the item slot is a list<string>."""
    values = list(sir.BASE_VALUES)
    run_parameters = list(sir.BASE_RUN_PARAMETERS)
    matrix_run_parameter(values, run_parameters, base_slot=6)
    values.append(sir.slot(7, LIST_STRING))
    return base_document(
        computations=[
            sir.comp(1, [], {"iterate": {"foreach": {
                "source": 10,
                "item_slot": 7,
                "body": 2,
                "carry": [],
                "max_inflight": None,
            }}}),
            sir.comp(2, [], sir.AWAIT_USER),
        ],
        root=1,
        types=MATRIX_TYPES,
        values=values,
        run_parameters=run_parameters,
        expressions=sir.BASE_EXPRESSIONS + [rd(10, 6, LIST_LIST_STRING)],
        contracts=[sir.component_contract()],
    )


def matrix_carry_document():
    """The matrix foreach with a list<string> carry skeleton.

    Carry slot 8, initial expr 9 (the pre-loop list reader), body output
    slot 9 -- all list<string>, so every Phase 3B carry rule passes.
    """
    document = matrix_foreach_document()
    document["values"].extend([sir.slot(8, LIST_STRING), sir.slot(9, LIST_STRING)])
    foreach = comp_of(document, 1)["body"]["iterate"]["foreach"]
    foreach["carry"] = [{"slot": 8, "initial": 9, "body_output_port": 1}]
    comp_of(document, 2)["outputs"] = [sir.obind(1, 9)]
    return document


negative(
    "foreach/source-uses-item-slot", "iterate_local_outside",
    lambda d: (
        comp_of(d, 1)["body"]["iterate"]["foreach"].__setitem__("source", 11),
        d["expressions"].append(sir.expr(11, LIST_LIST_STRING,
                                         {"list_construct": {
                                             "element_type": LIST_STRING,
                                             "elements": [12]}})),
        d["expressions"].append(sir.expr(12, LIST_STRING, {"value_slot": 7}))),
    base=matrix_foreach_document,
    note="the rebuilt source list still reads the item slot: pre-loop only",
)

negative(
    "foreach/source-uses-carry-slot", "iterate_local_outside",
    lambda d: (comp_of(d, 1)["body"]["iterate"]["foreach"].__setitem__(
        "source", 21),
        d["expressions"].append(sir.expr(21, LIST_LIST_STRING,
                                         {"list_construct": {
                                             "element_type": LIST_STRING,
                                             "elements": [22]}})),
        d["expressions"].append(sir.expr(22, LIST_STRING, {"value_slot": 8}))),
    base=matrix_carry_document,
    note="the rebuilt source list still reads the carry slot: pre-loop only",
)

negative(
    "foreach/carry-initial-uses-item-slot", "iterate_local_outside",
    lambda d: (comp_of(d, 1)["body"]["iterate"]["foreach"]["carry"][0].__setitem__(
        "initial", 22),
        d["expressions"].append(sir.expr(22, LIST_STRING, {"value_slot": 7}))),
    base=matrix_carry_document,
    note="carry initials evaluate from the pre-loop environment",
)

negative(
    "foreach/carry-initial-uses-carry-slot", "iterate_local_outside",
    lambda d: (comp_of(d, 1)["body"]["iterate"]["foreach"]["carry"][0].__setitem__(
        "initial", 23),
        d["expressions"].append(sir.expr(23, LIST_STRING, {"value_slot": 8}))),
    base=matrix_carry_document,
    note="initials cannot order among themselves",
)

negative(
    "foreach/item-escapes-loop", "iterate_local_outside",
    lambda d: (d["computations"].append(inv(3, [], inputs=[sir.ibind(1, 11)])),
               d["expressions"].append(sir.expr(11, LIST_STRING, {"value_slot": 7}))),
    base=lambda: wrapped(matrix_foreach_document, 3),
    note="the item slot exists only inside the foreach body region",
)

negative(
    "foreach/carry-escapes-loop", "iterate_local_outside",
    lambda d: (d["computations"].append(inv(3, [], inputs=[sir.ibind(1, 24)])),
               d["expressions"].append(sir.expr(24, LIST_STRING, {"value_slot": 8}))),
    base=lambda: wrapped(matrix_carry_document, 3),
    note="carry slots live only in the loop header/body region",
)

negative(
    "foreach/body-output-escapes-loop", "outside_owner_boundary",
    lambda d: (d["computations"].append(inv(3, [], inputs=[sir.ibind(1, 25)])),
               d["expressions"].append(sir.expr(25, LIST_STRING, {"value_slot": 9}))),
    base=lambda: wrapped(matrix_carry_document, 3),
    note="body outputs are iteration-internal; export via the iterate instead",
)


# --- Negatives: while loop locals --------------------------------------------

def while_document():
    return base_document(
        computations=[
            sir.comp(1, [], {"iterate": {"while": {
                "guard": 2,
                "body": 2,
                "carry": [{"slot": 7, "initial": 2, "body_output_port": 1}],
                "max_iterations": 5,
            }}}),
            sir.comp(2, [sir.obind(1, 8)], sir.AWAIT_USER),
        ],
        root=1,
        values=sir.BASE_VALUES + [sir.slot(7, 2), sir.slot(8, 2)],
        contracts=[sir.component_contract()],
    )


negative(
    "while/carry-initial-uses-carry-slot", "iterate_local_outside",
    lambda d: (comp_of(d, 1)["body"]["iterate"]["while"]["carry"][0].__setitem__(
        "initial", 16),
        d["expressions"].append(sir.expr(16, 2, {"value_slot": 7}))),
    base=while_document,
    note="initials evaluate from the pre-loop environment",
)

negative(
    "while/guard-uses-body-output", "before_producer",
    lambda d: (comp_of(d, 1)["body"]["iterate"]["while"].__setitem__("guard", 17),
               d["expressions"].append(sir.expr(17, 2, {"value_slot": 8}))),
    base=while_document,
    note="body outputs reach the next round only via the carry binding",
)

negative(
    "while/carry-escapes-loop", "iterate_local_outside",
    lambda d: (d["computations"].append(inv(3, [], inputs=[sir.ibind(1, 18)])),
               d["expressions"].append(sir.expr(18, 2, {"value_slot": 7}))),
    base=lambda: wrapped(while_document, 3),
    note="carry slots live only in the loop header/body region",
)

negative(
    "while/body-output-escapes-loop", "outside_owner_boundary",
    lambda d: (d["computations"].append(inv(3, [], inputs=[sir.ibind(1, 19)])),
               d["expressions"].append(sir.expr(19, 2, {"value_slot": 8}))),
    base=lambda: wrapped(while_document, 3),
    note="body outputs are iteration-internal",
)


# --- Negatives: workflow output boundary --------------------------------------

negative(
    "workflow-output/raw-root-descendant-output", "outside_owner_boundary",
    lambda d: d.__setitem__("workflow_outputs",
                            [{"name": "leak", "expression": 10}]),
    base=lambda: base_document(
        computations=[
            sir.comp(1, [], {"sequence": {"children": [2, 3]}}),
            inv(2, [sir.obind(1, 6)]),
            sir.comp(3, [], sir.AWAIT_USER),
        ],
        root=1,
        contracts=[sir.component_contract()],
        values=sir.BASE_VALUES + [sir.slot(6, 1)],
        expressions=sir.BASE_EXPRESSIONS + [rd(10, 6, 1)],
    ),
    note="only root exports and run parameters survive to the exit",
)

negative(
    "workflow-output/iterate-local", "iterate_local_outside",
    lambda d: d.__setitem__("workflow_outputs",
                            [{"name": "leak", "expression": 12}]),
    base=lambda: base_document(
        computations=[
            sir.comp(1, [], {"iterate": {"foreach": {
                "source": 9,
                "item_slot": 6,
                "body": 2,
                "carry": [],
                "max_inflight": None,
            }}}),
            sir.comp(2, [], sir.AWAIT_USER),
        ],
        root=1,
        values=sir.BASE_VALUES + [sir.slot(6, 3)],
        expressions=sir.BASE_EXPRESSIONS + [rd(12, 6, 3)],
    ),
    note="an item slot never escapes its iterate",
)

negative(
    "workflow-output/select-case-output", "outside_owner_boundary",
    lambda d: d.__setitem__("workflow_outputs",
                            [{"name": "leak", "expression": 10}]),
    base=lambda: base_document(
        computations=[
            sir.comp(1, [sir.obind(1, 7)], {"select": {"selector": 6, "cases": [
                {"ordinal": 1, "child": 2},
            ]}}),
            inv(2, [sir.obind(1, 8)]),
        ],
        root=1,
        contracts=[sir.component_contract()],
        values=sir.BASE_VALUES + [sir.slot(7, 1), sir.slot(8, 1)],
        expressions=sir.BASE_EXPRESSIONS + [rd(10, 8, 1)],
    ),
    note="case exports merge only through the select's own outputs",
)


# --- Negatives: resource lexical visibility -----------------------------------

negative(
    "resource/scope-child-uses-omitted", None,
    lambda d: None,
    base=lambda: base_document(
        computations=[
            sir.comp(1, [], {"scope": {"child": 2, "resources": [1]}}),
            inv(2, [], resources=[sir.rbind(1, 2)]),
        ],
        root=1,
        contracts=[sir.component_contract()],
    ),
    note="resource 2 is not in the scope's listed set",
)

negative(
    "resource/nested-scope-reintroduces", None,
    lambda d: None,
    base=lambda: base_document(
        computations=[
            sir.comp(1, [], {"scope": {"child": 2, "resources": [1]}}),
            sir.comp(2, [], {"scope": {"child": 3, "resources": [1, 2]}}),
            sir.comp(3, [], sir.AWAIT_USER),
        ],
        root=1,
    ),
    note="a nested scope cannot widen the inherited environment",
)

negative(
    "resource/call-subflow-uses-invisible", None,
    lambda d: None,
    base=lambda: base_document(
        computations=[
            sir.comp(1, [], {"scope": {"child": 2, "resources": [1]}}),
            csub(2, [], resources=[sir.rbind(1, 2)]),
        ],
        root=1,
        contracts=[sir.component_contract(), sir.subflow_contract()],
    ),
    note="call_subflow resource uses need lexical visibility too",
)


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
    positives = []
    for case in POSITIVE_CASES:
        document = case["document"]
        sir.validate(document)  # Phase 3B-clean: the dataflow layer accepts it
        observations = analyze(document)
        positives.append({
            "case_id": case["case_id"],
            "note": case["note"],
            "document": document,
            "entry_values": {
                str(comp_id): sorted(slots)
                for comp_id, slots in sorted(observations["entry_values"].items())
            },
            "entry_resources": {
                str(comp_id): sorted(slots)
                for comp_id, slots in sorted(observations["entry_resources"].items())
            },
            "workflow_exit_values": observations["workflow_exit_values"],
        })
    print(f"validated {len(positives)} positive documents")

    for case in NEGATIVE_CASES:
        document = case["document"]
        try:
            sir.validate(document)
        except sir.ValidationError as error:
            raise AssertionError(
                f"{case['case_id']} must be Phase 3B-valid, failed at "
                f"{error.kind}") from error
        try:
            analyze(document)
        except AvailabilityError as error:
            assert error.kind == case["error_kind"], (
                f"{case['case_id']}: expected {case['error_kind']}, "
                f"got {error.kind}")
            if case.get("reason") is not None:
                assert error.reason == case["reason"], (
                    f"{case['case_id']}: expected reason {case['reason']}, "
                    f"got {error.reason}")
        else:
            raise AssertionError(f"{case['case_id']} unexpectedly analyzed clean")
    print(f"validated {len(NEGATIVE_CASES)} negative documents")

    # Cross-proof: every frozen Phase 3B golden document also passes the
    # Phase 3C analysis — the analysis layer adds rejections without
    # invalidating the shipped IR corpus.
    for case_id, document in sir.GOLDENS:
        sir.validate(document)
        analyze(document)
    print(f"cross-checked {len(sir.GOLDENS)} Phase 3B goldens against the "
          "availability oracle")

    write_json("spec/conformance/semantic-dataflow/positive/"
               "semantic-dataflow-positive.json", {
        "kind": "agentflow.conformance.semantic-dataflow.positive",
        "corpus_revision": CORPUS_REVISION,
        "note": (
            "Frozen Phase 3C scoped-dataflow observations: every positive is "
            "Phase 3B-valid, passes the availability analysis, and pins the "
            "derived EntryValues/EntryResources per computation plus "
            "WorkflowExitValues. Generated by the independent Python "
            "implementation (this script); never from the Rust analyzer."
        ),
        "cases": positives,
    })

    write_json("spec/conformance/semantic-dataflow/negative/"
               "semantic-dataflow-negative.json", {
        "kind": "agentflow.conformance.semantic-dataflow.negative",
        "corpus_revision": CORPUS_REVISION,
        "note": (
            "Frozen Phase 3C negatives: every document passes the Phase 3B "
            "validator and is rejected only by the dataflow analysis with "
            "the stable kind value_not_available or resource_not_visible; "
            "reason is the frozen diagnostic token."
        ),
        "cases": NEGATIVE_CASES,
    })


if __name__ == "__main__":
    main()
