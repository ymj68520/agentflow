#!/usr/bin/env python3
"""Independent Phase 4B.2B.1 control-algebra oracle and frozen vectors.

The oracle is Python-only and emits test vectors, not a wire shape, ACE1
object, Control digest, or HashV1 domain.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "spec" / "conformance" / "computation-control"
MAX_U64 = (1 << 64) - 1
AUTHORITY_ORDER = {"none": 0, "external_data": 1, "delegated_agent": 2, "user": 3, "workflow": 4, "platform": 5}


def canonical(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def verdict(values: list[str]) -> str:
    for preferred in ("violation", "requires_evidence", "unproven"):
        if preferred in values:
            return preferred
    return "proven"


def atom_key(atom: dict) -> tuple:
    selector = atom.get("resource_selector", {})
    scope = tuple(sorted((item["key"], item["value"]) for item in selector.get("scope_bindings", [])))
    qualifiers = tuple(sorted(canonical(item) for item in atom.get("qualifiers", [])))
    operation = atom.get("operation", {})
    return (operation.get("catalog"), operation.get("name"), selector.get("requirement_port_no"), scope, qualifiers)


def upper_add(left: int | str, right: int | str) -> int | str | None:
    if left == "unbounded" or right == "unbounded":
        return "unbounded"
    value = int(left) + int(right)
    return value if value <= MAX_U64 else None


def upper_max(left: int | str, right: int | str) -> int | str:
    if left == "unbounded" or right == "unbounded":
        return "unbounded"
    return max(int(left), int(right))


def zero_effect_lower(signature: dict) -> dict:
    if signature["row"] == "unknown":
        return {"row": "unknown"}
    result = json.loads(json.dumps(signature))
    for entry in result.get("entries", []):
        entry["lower"] = 0
    for residual in result.get("residuals", []):
        residual["lower"] = 0
    return result


def merge_entry(left: dict, right: dict, operator: str) -> dict:
    if operator == "accumulate":
        upper = upper_add(left["upper"], right["upper"])
        if upper is None:
            raise OverflowError("effect occurrence upper overflow")
        same_guard = left.get("guard", "unconditional") == right.get("guard", "unconditional")
        left_guard = left.get("guard", "unconditional")
        if same_guard and left_guard == "unconditional":
            lower = int(left["lower"]) + int(right["lower"])
        else:
            lower = 0
        guard = left_guard if same_guard else "unconditional"
        return {"atom": left["atom"], "lower": lower, "upper": upper, "guard": guard}
    same_guard = left.get("guard", "unconditional") == right.get("guard", "unconditional")
    guard = left.get("guard", "unconditional") if same_guard else "unconditional"
    lower = min(int(left["lower"]), int(right["lower"])) if same_guard else 0
    return {"atom": left["atom"], "lower": lower, "upper": upper_max(left["upper"], right["upper"]), "guard": guard}


def combine_effects(signatures: list[dict], mode: str) -> dict:
    if any(signature is None or signature.get("row") == "unknown" for signature in signatures):
        return {"row": "unknown"}
    rows = {signature.get("row", "closed") for signature in signatures}
    is_open = "open" in rows
    entries: dict[tuple, dict] = {}
    residuals: dict[tuple, dict] = {}
    for signature in signatures:
        for entry in signature.get("entries", []):
            key = atom_key(entry["atom"])
            if key not in entries:
                entries[key] = dict(entry)
            else:
                entries[key] = merge_entry(entries[key], entry, mode)
        for residual in signature.get("residuals", []):
            selector = residual.get("selector", {})
            scope = tuple(sorted((item["key"], item["value"]) for item in selector.get("scope_bindings", [])))
            key = (residual.get("catalog"), selector.get("requirement_port_no"), scope)
            if key not in residuals:
                residuals[key] = dict(residual)
            elif mode == "accumulate":
                upper = upper_add(residuals[key]["upper"], residual["upper"])
                if upper is None:
                    raise OverflowError("effect residual upper overflow")
                residuals[key] = {**residuals[key], "lower": 0, "upper": upper}
            else:
                residuals[key] = {**residuals[key], "lower": 0, "upper": upper_max(residuals[key]["upper"], residual["upper"])}
    return {
        "row": "open" if is_open or residuals else "closed",
        "entries": [entries[key] for key in sorted(entries)],
        "residuals": [residuals[key] for key in sorted(residuals)],
    }


def control_effect(operator: str, children: list[dict], exact_child: int | None = None) -> tuple[dict, str, list[dict]]:
    known = [(index, child["node"], child.get("effects")) for index, child in enumerate(children) if child.get("effects") is not None]
    missing = [child["node"] for child in children if child.get("effects") is None]
    occurrences = []
    for child in children:
        for ordinal, entry in enumerate(child.get("effect_entries", (child.get("effects") or {}).get("entries", []))):
            occurrences.append({"caller_digest": child.get("caller_digest", "caller"), "node": child["node"], "contract_digest": child.get("contract_digest", "contract"), "entry_ordinal": ordinal, "atom": entry["atom"]})
    if missing:
        return {"row": "unknown", "known_contributions": [value for _, _, value in known]}, "unproven", occurrences
    ordered = [value for _, _, value in known]
    if operator == "select" and exact_child is not None:
        result = ordered[exact_child]
    else:
        if operator in ("race", "parallel_fail_fast"):
            ordered = [zero_effect_lower(value) for value in ordered]
        if operator == "sequence":
            ordered = [value if index == 0 else zero_effect_lower(value) for index, value in enumerate(ordered)]
        try:
            if operator == "select":
                for index, signature in enumerate(ordered):
                    if index == 0:
                        result = signature
                    else:
                        result = combine_effects([result, signature], "join")
                result = zero_effect_lower(result)
            else:
                result = {"row": "closed", "entries": [], "residuals": []}
                for signature in ordered:
                    result = combine_effects([result, signature], "accumulate")
        except OverflowError:
            return {"row": "unknown", "arithmetic": "overflow"}, "violation", occurrences
    return result, "unproven" if result.get("row") == "unknown" else "proven", occurrences


def usage_combine(envelopes: list[dict | None], operator: str) -> tuple[dict, str]:
    if any(envelope is None for envelope in envelopes):
        return {"state": "unknown", "known_contributions": [item for item in envelopes if item is not None]}, "unproven"
    if not envelopes:
        return {"state": "unknown", "coordinates": {}}, "unproven"
    coordinates = set().union(*(set(envelope.get("coordinates", {})) for envelope in envelopes))
    if not coordinates:
        return {"state": "empty_without_zero_claim", "coordinates": {}}, "unproven"
    output: dict[str, int | str] = {}
    for coordinate in sorted(coordinates):
        values = [envelope.get("coordinates", {}).get(coordinate) for envelope in envelopes]
        if any(value is None or value == "unknown" for value in values):
            output[coordinate] = "unknown"
            continue
        if operator == "join":
            output[coordinate] = max(values, key=lambda value: float("inf") if value == "unbounded" else int(value)) if "unbounded" in values else max(int(value) for value in values)
        else:
            upper: int | str = 0
            for value in values:
                upper = upper_add(upper, value)
                if upper is None:
                    return {"state": "overflow", "coordinates": output}, "violation"
            output[coordinate] = upper
    state = "known" if all(value != "unknown" for value in output.values()) else "partial_unknown"
    return {"state": state, "coordinates": output}, "proven" if state == "known" else "unproven"


def compose_failure(children: list[dict]) -> tuple[list[str] | None, str]:
    if any(child.get("failures") is None for child in children):
        return None, "unproven"
    return sorted(set().union(*(set(child["failures"]) for child in children))), "proven"


def compose_flow(children: list[dict], operator: str, decision_dependencies: list[str] | None = None) -> dict:
    known = [child["flow"] for child in children if child.get("flow") is not None]
    sources = sorted(set().union(*(set(item.get("sources", [])) for item in known)))
    taints = sorted(set().union(*(set(item.get("taints", [])) for item in known)))
    authority = max((item.get("authority", "none") for item in known), key=lambda value: AUTHORITY_ORDER[value], default="none")
    unknown = any(item.get("influence_unknown", False) for item in known) or len(known) != len(children)
    decision_unknown = bool(decision_dependencies) or operator in ("race", "parallel_fail_fast")
    unknown |= decision_unknown
    return {
        "sources": sources,
        "taints": taints,
        "may_carry_authority": authority,
        "effective_authority_inferred": False,
        "influence_unknown": unknown,
        "decision_dependencies": decision_dependencies or [],
        "constraints_by_child": [{"node": child["node"], "constraints": child.get("flow", {}).get("constraints", [])} for child in children if child.get("flow") is not None],
        "verdict": "unproven" if unknown else "proven",
    }


def control_compose(data: dict) -> dict:
    operator = data["operator"]
    children = data["children"]
    exact_child = data.get("exact_child")
    effects, effect_verdict, occurrences = control_effect(operator, children, exact_child)
    if operator == "select" and data.get("selector_coverage") != "proven":
        if effects.get("row") in ("closed", "open"):
            effects = zero_effect_lower(effects)
            effect_verdict = "unproven"
    usage_operator = "join" if operator == "select" else "accumulate"
    usage, usage_verdict = usage_combine([child.get("usage") for child in children], usage_operator)
    failures, failure_verdict = compose_failure(children)
    flow = compose_flow(children, operator, data.get("decision_dependencies"))
    all_effects = all(child.get("effects") is not None for child in children)
    missing = [child["node"] for child in children if child.get("resolved", True) is False]
    adjustment_names = []
    if operator == "sequence":
        adjustment_names = [child["node"] for child in children[1:] if any(int(entry["lower"]) > 0 for entry in (child.get("effects") or {}).get("entries", []))]
    elif operator in ("race", "parallel_fail_fast", "select"):
        adjustment_names = [child["node"] for child in children if any(int(entry["lower"]) > 0 for entry in (child.get("effects") or {}).get("entries", []))]
    proof_verdicts = [child.get("verdict", "proven") for child in children]
    proof_verdicts.extend([effect_verdict, usage_verdict, failure_verdict, flow["verdict"]])
    if missing:
        proof_verdicts.append("unproven")
    return {
        "verdict": verdict(proof_verdicts),
        "effects": effects,
        "effect_row_complete": all_effects and effects.get("row") != "unknown",
        "effect_lower_bound_adjusted_children": adjustment_names,
        "effect_occurrences": occurrences,
        "usage": usage,
        "failures": failures,
        "flow": flow,
        "recovery_sources": [
            {"node": child["node"], "contract_digest": child.get("contract_digest"), "actions": child.get("recovery_actions", [])}
            for child in children if child.get("recovery_actions") is not None
        ],
        "execution_by_child": [{"node": child["node"], "x": child.get("execution", {})} for child in children if child.get("execution") is not None],
        "resources_by_child": [{"node": child["node"], "requirements": child.get("resources", [])} for child in children if child.get("resources") is not None],
        "unresolved_children": missing,
    }


def interface_mapping(data: dict) -> dict:
    expected = data["parent_outputs"]
    mappings = data["bindings"]
    verdicts = []
    outputs = []
    for output in expected:
        bindings = [binding for binding in mappings if binding["parent_port"] == output["port"]]
        if len(bindings) != 1:
            verdicts.append("violation")
            continue
        binding = bindings[0]
        branches = data.get("branches", [])
        if branches and not any(branch["ordinal"] == binding["branch_ordinal"] and branch["child"] == binding["child"] for branch in branches):
            verdicts.append("violation")
            continue
        child = next((child for child in data["children"] if child["node"] == binding["child"]), None)
        child_output = next((item for item in (child or {}).get("outputs", []) if item["port"] == binding["child_port"]), None)
        if child_output is None:
            verdicts.append("violation")
            continue
        if child_output["type_id"] != output["type_id"]:
            verdicts.append("violation")
        outputs.append({"parent_port": output["port"], "parent_slot": output["slot"], "source_child": binding["child"], "source_port": binding["child_port"], "type_id": output["type_id"]})
    return {"verdict": verdict(verdicts), "outputs": outputs}


def evaluate(case: dict) -> dict:
    operation = case["input"]["operation"]
    data = case["input"]
    if operation == "compose":
        return control_compose(data)
    if operation == "parallel_output":
        return interface_mapping(data)
    if operation == "sequence_edge":
        return {"verdict": "unproven", "edge": data["edge"], "may_influence_sources": sorted(set(data["prior_sources"] + data["consumer_sources"])), "influence_unknown": True}
    if operation == "namespace_alias":
        return {"verdict": "proven", "component_ports": data["component_ports"], "resource_slots": sorted(set(data["resource_slots"])), "merged_by_port_number": False}
    if operation == "authority":
        upper = max(data["authorities"], key=lambda value: AUTHORITY_ORDER[value])
        return {"verdict": "proven", "may_carry_upper": upper, "effective_authority": None, "authorization": False}
    if operation == "unknown_control":
        return {"verdict": "unproven", "operator": data["operator"], "dimension_state": "unknown", "treated_as_empty": False, "known_children": data.get("known_children", [])}
    if operation == "verdicts":
        return {"verdict": verdict(data["inputs"]), "retained": data["inputs"]}
    if operation == "equivalence":
        left = data["left"]
        right = data["right"]
        if data.get("semantic_order"):
            return {"verdict": "violation" if left != right else "proven", "equivalent": left == right}
        normalize = lambda values: sorted(values, key=canonical)
        return {"verdict": "proven" if normalize(left) == normalize(right) else "violation", "normalized": normalize(left)}
    raise ValueError(f"unknown oracle operation {operation!r}")


def case(case_id: str, group: str, operation: str, payload: dict, note: str) -> dict:
    item = {"case_id": case_id, "group": group, "kind": "agentflow.conformance.computation-control.test-vector", "input": {"operation": operation, **payload}, "note": note}
    item["expected"] = evaluate(item)
    return item


ATOM_NATURAL = {"operation": {"catalog": "agentflow.effects@1", "name": "write"}, "resource_selector": {"requirement_port_no": 1, "scope_bindings": []}, "qualifiers": [{"family": "idempotency", "member": "natural"}]}
ATOM_NONE = {**ATOM_NATURAL, "qualifiers": [{"family": "idempotency", "member": "none"}]}


def effect_entry(atom: dict = ATOM_NATURAL, lower: int = 0, upper: int | str = 2, guard: Any = "unconditional") -> dict:
    return {"atom": atom, "lower": lower, "upper": upper, "guard": guard}


def effect_row(*entries: dict, row: str = "closed", residuals: list[dict] | None = None) -> dict:
    return {"row": row, "entries": list(entries), "residuals": residuals or []}


def usage(**coordinates: int | str) -> dict:
    return {"coordinates": coordinates}


def child(node: int, contract: str, *, effects: dict | None = None, usage_row: dict | None = None, failures: list[str] | None = None, sources: list[str] | None = None, taints: list[str] | None = None, authority: str = "none", unknown: bool = False, recovery_actions: list[dict] | None = None, execution: dict | None = None, resources: list[dict] | None = None, verdict_value: str = "proven") -> dict:
    return {"node": node, "contract_digest": contract, "caller_digest": "semantic-ir:caller", "effects": effects, "effect_entries": (effects or {}).get("entries", []), "usage": usage_row, "failures": failures, "flow": None if sources is None and taints is None else {"sources": sources or [], "taints": taints or [], "authority": authority, "influence_unknown": unknown, "constraints": []}, "recovery_actions": recovery_actions, "execution": execution, "resources": resources, "verdict": verdict_value, "resolved": effects is not None}


def compose_case(case_id: str, group: str, operator: str, children: list[dict], note: str, **options: Any) -> dict:
    return case(case_id, group, "compose", {"operator": operator, "children": children, **options}, note)


COMPONENT_A = "component-contract:" + "a" * 64
COMPONENT_B = "component-contract:" + "b" * 64
BASE_CHILDREN = [child(2, COMPONENT_A, effects=effect_row(effect_entry(lower=1, upper=2)), usage_row=usage(**{"forward/api_calls": 100}), failures=["FailureA"], sources=["input:1"], taints=["web_untrusted"], execution={"sandbox": "strict"}, resources=[{"component_port": 1, "resource_slot": "slot:1"}]), child(3, COMPONENT_B, effects=effect_row(effect_entry(lower=1, upper=3)), usage_row=usage(**{"forward/api_calls": 200}), failures=["FailureB"], sources=["input:2"], taints=["user_text"], execution={"runtime": "python"}, resources=[{"component_port": 1, "resource_slot": "slot:1"}])]


CASES = [
    compose_case("computation-control/golden/minimal-known-sequence-envelope", "golden", "sequence", [child(2, COMPONENT_A, effects=effect_row(), usage_row=usage(**{"forward/api_calls": 0}), failures=[], sources=[], taints=[]), child(3, COMPONENT_B, effects=effect_row(), usage_row=usage(**{"forward/api_calls": 0}), failures=[], sources=[], taints=[])], "A fully specified minimal control envelope is deterministic and keeps explicit zero distinct from missing."),
    compose_case("computation-control/sequence/two-effects-accumulate", "sequence", "sequence", BASE_CHILDREN, "Sequence accumulates possible effect upper bounds."),
    compose_case("computation-control/sequence/same-atom-occurrences-distinct", "sequence", "sequence", BASE_CHILDREN, "Envelope may merge the atom, but the two call-site occurrence identities remain distinct."),
    compose_case("computation-control/sequence/second-lower-conditional", "sequence", "sequence", BASE_CHILDREN, "A later child may be skipped after earlier non-success; its positive lower is weakened."),
    compose_case("computation-control/sequence/first-failure-can-skip-next", "sequence", "sequence", BASE_CHILDREN, "Failure rows include every child that may run even when a prior failure can skip later children."),
    compose_case("computation-control/sequence/unknown-first-child", "sequence", "sequence", [child(2, COMPONENT_A, effects=None, usage_row=None, failures=None, verdict_value="unproven"), BASE_CHILDREN[1]], "Unknown first child is not replaced with a pure/zero row."),
    compose_case("computation-control/sequence/usage-checked-sum", "sequence", "sequence", BASE_CHILDREN, "Forward meter bounds use checked additive accumulation."),
    compose_case("computation-control/sequence/typed-cross-sibling-edge", "sequence", "sequence", BASE_CHILDREN, "Sequence retains the Phase 3C producer-to-use edge and marks per-value Flow transfer unproven.", edge={"producer_child": 2, "slot": "value-slot:1", "consumer_child": 3}, prior_sources=["input:1"], consumer_sources=["input:2"]),
    case("computation-control/sequence/typed-output-last-child", "sequence", "parallel_output", {"parent_outputs": [{"port": 1, "slot": "slot:parent", "type_id": "semantic-type:1"}], "children": [{"node": 3, "outputs": [{"port": 1, "type_id": "semantic-type:1", "slot": "slot:last"}]}], "bindings": [{"parent_port": 1, "child": 3, "child_port": 1}]}, "Sequence output interface follows the final child and retains typed evidence."),
    case("computation-control/parallel/output-explicit-branch-map", "parallel", "parallel_output", {"parent_outputs": [{"port": 1, "slot": "slot:parent", "type_id": "semantic-type:1"}], "branches": [{"ordinal": 1, "child": 2}, {"ordinal": 2, "child": 3}], "children": [{"node": 2, "outputs": [{"port": 1, "type_id": "semantic-type:1", "slot": "slot:left"}]}, {"node": 3, "outputs": [{"port": 1, "type_id": "semantic-type:1", "slot": "slot:right"}]}], "bindings": [{"parent_port": 1, "branch_ordinal": 2, "child": 3, "child_port": 1}]}, "Parallel output is sourced only by its explicit checked branch/port mapping."),
    compose_case("computation-control/sequence/forward-recovery-phases-separate", "sequence", "sequence", [child(2, COMPONENT_A, usage_row=usage(**{"forward/api_calls": 10, "recovery/api_calls": 2})), child(3, COMPONENT_B, usage_row=usage(**{"forward/api_calls": 20, "recovery/api_calls": 3}))], "Usage coordinates preserve Forward and Recovery as separate meters/phases."),
    compose_case("computation-control/sequence/empty-failure-not-success", "sequence", "sequence", [child(2, COMPONENT_A, failures=[]), child(3, COMPONENT_B, failures=[])], "An empty typed Failure row is a known row, not a guaranteed-success claim."),
    compose_case("computation-control/select/exclusive-effect-join", "select", "select", BASE_CHILDREN, "Select joins mutually exclusive effects, not additive totals."),
    compose_case("computation-control/select/different-guards-lower-zero", "select", "select", [child(2, COMPONENT_A, effects=effect_row(effect_entry(lower=1, upper=2, guard={"input_equals": {"port": 1, "value": "a"}}))), child(3, COMPONENT_B, effects=effect_row(effect_entry(lower=1, upper=3, guard={"input_equals": {"port": 1, "value": "b"}})))], "Different guards widen without a false positive lower bound."),
    compose_case("computation-control/select/qualifier-members-distinct", "select", "select", [child(2, COMPONENT_A, effects=effect_row(effect_entry(ATOM_NATURAL, 1, 2))), child(3, COMPONENT_B, effects=effect_row(effect_entry(ATOM_NONE, 1, 3)))], "Full qualifier identity survives an exclusive join."),
    compose_case("computation-control/select/usage-maximum", "select", "select", BASE_CHILDREN, "Known mutually exclusive Usage ceilings join by maximum."),
    compose_case("computation-control/select/decision-input-influence", "select", "select", BASE_CHILDREN, "Selector value dependencies are retained as decision influence.", decision_dependencies=["value-slot:1"]),
    compose_case("computation-control/select/unmatched-ordinal-unproven", "select", "select", BASE_CHILDREN, "The frozen IR has no default/range field; selector totality remains Unproven.", selector_coverage="unproven"),
    case("computation-control/select/typed-merge-mismatch", "negative", "parallel_output", {"parent_outputs": [{"port": 1, "slot": "slot:p", "type_id": "semantic-type:1"}], "children": [{"node": 2, "outputs": [{"port": 1, "type_id": "semantic-type:2", "slot": "slot:a"}]}], "bindings": [{"parent_port": 1, "child": 2, "child_port": 1}]}, "A typed output mismatch is not accepted by a display-name or local-id heuristic."),
    compose_case("computation-control/select/unknown-branch-not-pure", "select", "select", [child(2, COMPONENT_A, effects=effect_row(effect_entry())), child(3, COMPONENT_B, effects=None, verdict_value="unproven")], "An unknown selectable branch contaminates the complete Effect envelope."),
    compose_case("computation-control/select/taints-union", "select", "select", BASE_CHILDREN, "May-influence taints from every selectable branch remain."),
    compose_case("computation-control/select/exact-constant-case", "select", "select", BASE_CHILDREN, "A constant exact ordinal may resolve the selected branch without dynamic selection.", exact_child=0, selector_coverage="proven"),
    compose_case("computation-control/parallel/all-effects-accumulate", "parallel", "parallel_all", BASE_CHILDREN, "Every potentially executed branch contributes its Effect upper bound."),
    compose_case("computation-control/parallel/all-usage-accumulates", "parallel", "parallel_all", BASE_CHILDREN, "Parallel branch usage sums; max_inflight does not cap total usage."),
    compose_case("computation-control/parallel/fail-fast-sibling-effect-retained", "parallel", "parallel_fail_fast", BASE_CHILDREN, "Fail-fast does not erase effects that may happen before sibling cancellation."),
    compose_case("computation-control/parallel/fail-fast-lower-zero", "parallel", "parallel_fail_fast", BASE_CHILDREN, "A positive branch lower bound is not guaranteed after possible early cancellation."),
    compose_case("computation-control/parallel/same-contract-occurrences", "parallel", "parallel_all", [child(2, COMPONENT_A, effects=effect_row(effect_entry())), child(3, COMPONENT_A, effects=effect_row(effect_entry()))], "Identical target digests on different branch nodes remain distinct occurrences."),
    compose_case("computation-control/parallel/max-inflight-not-total-count", "parallel", "parallel_all", BASE_CHILDREN, "Concurrency limit does not scale or cap total usage.", max_inflight=1),
    compose_case("computation-control/parallel/shared-slot-alias-explicit", "parallel", "parallel_all", BASE_CHILDREN, "A shared caller resource slot is an explicit alias with two source ports."),
    case("computation-control/parallel/different-port-namespaces", "parallel", "namespace_alias", {"component_ports": [{"component": "A", "port": 1}, {"component": "B", "port": 1}], "resource_slots": ["resource-slot:4", "resource-slot:9"]}, "Equal component port numbers do not merge resource identities."),
    compose_case("computation-control/parallel/execution-per-child", "parallel", "parallel_all", BASE_CHILDREN, "X stays child-scoped; no single Worker demand is formed."),
    compose_case("computation-control/parallel/unknown-sibling-obligations", "parallel", "parallel_all", [BASE_CHILDREN[0], child(3, COMPONENT_B, effects=None, usage_row=None, failures=None, recovery_actions=None, execution=None, verdict_value="unproven")], "An unknown sibling preserves all known child contributions and remains unresolved."),
    compose_case("computation-control/race/all-contestants-not-winner-only", "race", "race", BASE_CHILDREN, "RACE accumulation contains every contestant, not only a guessed winner."),
    compose_case("computation-control/race/loser-effect-retained", "race", "race", BASE_CHILDREN, "Loser cancellation cannot erase an already possible Effect."),
    compose_case("computation-control/race/loser-usage-retained", "race", "race", BASE_CHILDREN, "Usage from all contestants is accumulated before/after cancellation."),
    compose_case("computation-control/race/loser-recovery-retained", "race", "race", [child(2, COMPONENT_A, effects=effect_row(effect_entry()), recovery_actions=[{"forward_atom": ATOM_NATURAL, "action": "compensate"}]), child(3, COMPONENT_B, effects=effect_row(effect_entry()), recovery_actions=[{"forward_atom": ATOM_NATURAL, "action": "reconcile"}])], "K/action facts from a losing branch remain source-scoped."),
    compose_case("computation-control/race/cancel-not-no-effect", "race", "race", BASE_CHILDREN, "Cancellation is not evidence that no branch Effect occurred."),
    compose_case("computation-control/race/winner-may-fail", "race", "race", BASE_CHILDREN, "Every contestant Failure row is retained; winner is not assumed successful."),
    compose_case("computation-control/race/unknown-loser-contaminates", "race", "race", [BASE_CHILDREN[0], child(3, COMPONENT_B, effects=None, usage_row=None, failures=None, verdict_value="unproven")], "An unknown loser cannot be dropped from the envelope."),
    compose_case("computation-control/race/winner-output-taints-union", "race", "race", [child(2, COMPONENT_A, sources=["input:a"], taints=["web"]), child(3, COMPONENT_B, sources=["input:b"], taints=["user"])], "Every possible winner contributes output taint and source facts."),
    compose_case("computation-control/race/same-atom-distinct-occurrences", "race", "race", [child(2, COMPONENT_A, effects=effect_row(effect_entry())), child(3, COMPONENT_A, effects=effect_row(effect_entry()))], "Same AtomKey across branches may merge in the upper row, never in occurrence provenance."),
    compose_case("computation-control/race/physical-timing-not-decision", "race", "race", BASE_CHILDREN, "Static analysis does not select a winner from physical completion timing."),
    compose_case("computation-control/proof/requires-evidence-propagates", "proof", "compose", [child(2, COMPONENT_A, effects=effect_row(effect_entry()), verdict_value="requires_evidence"), child(3, COMPONENT_B, effects=effect_row())], "Known Effect envelope and inherited RequiresEvidence coexist."),
    compose_case("computation-control/proof/unproven-propagates", "proof", "compose", [child(2, COMPONENT_A, effects=None, verdict_value="unproven"), child(3, COMPONENT_B, effects=effect_row())], "Unproven child never becomes a Proven parent."),
    compose_case("computation-control/proof/known-bound-with-evidence", "proof", "compose", [child(2, COMPONENT_A, effects=effect_row(effect_entry()), verdict_value="requires_evidence"), child(3, COMPONENT_B, effects=effect_row())], "The conservative declared bound remains materialized while the overall proof is RequiresEvidence."),
    compose_case("computation-control/proof/unknown-effect-row-not-empty", "proof", "compose", [child(2, COMPONENT_A, effects={"row": "unknown"}), child(3, COMPONENT_B, effects=effect_row())], "EffectRow::Unknown contaminates the result; no pure Closed row is synthesized."),
    compose_case("computation-control/proof/open-effect-row-remains-open", "proof", "compose", [child(2, COMPONENT_A, effects=effect_row(effect_entry(), row="open", residuals=[{"catalog": "agentflow.effects@1", "selector": {"requirement_port_no": 1, "scope_bindings": []}, "lower": 0, "upper": 4}])), child(3, COMPONENT_B, effects=effect_row())], "Bounded open residuals remain open after composition."),
    compose_case("computation-control/proof/missing-usage-not-zero", "proof", "compose", [child(2, COMPONENT_A, usage_row=None, verdict_value="unproven"), child(3, COMPONENT_B, usage_row=usage(**{"forward/api_calls": 5}))], "Missing Usage remains unknown, never finite(0)."),
    compose_case("computation-control/proof/cancellation-not-failure", "proof", "race", [child(2, COMPONENT_A, failures=["FailureA"]), child(3, COMPONENT_B, failures=["FailureB"])], "Cancellation is not inserted into the typed Failure row."),
    compose_case("computation-control/negative/effect-overflow", "negative", "sequence", [child(2, COMPONENT_A, effects=effect_row(effect_entry(lower=0, upper=MAX_U64))), child(3, COMPONENT_B, effects=effect_row(effect_entry(lower=0, upper=1)))], "Finite upper overflow is a violation and never wraps."),
    compose_case("computation-control/proof/authority-upper-not-permission", "proof", "parallel_all", [child(2, COMPONENT_A, authority="user", sources=["input:1"]), child(3, COMPONENT_B, authority="external_data", sources=["unknown_external"], unknown=True)], "The maximum is only a may-carry upper bound; no effective authority or grant is inferred."),
    compose_case("computation-control/proof/compensation-not-subtraction", "proof", "sequence", [child(2, COMPONENT_A, effects=effect_row(effect_entry())), child(3, COMPONENT_B, effects=effect_row(effect_entry()))], "Forward and counter/recovery declarations add; no effect subtraction exists."),
    compose_case("computation-control/proof/execution-not-worker-sum", "proof", "parallel_all", BASE_CHILDREN, "Per-child X is preserved instead of summing CPU/quantity into a single Worker."),
    case("computation-control/proof/unsupported-timeout-not-empty", "proof", "unknown_control", {"operator": "timeout", "known_children": [2], "known_effect": effect_row()}, "An unsupported wrapper carries an explicit unknown state, not empty."),
    case("computation-control/equivalence/unordered-members-normalized", "equivalence", "equivalence", {"left": ["input:b", "input:a"], "right": ["input:a", "input:b"]}, "Unordered summary members normalize deterministically."),
    case("computation-control/equivalence/sequence-order-is-semantic", "equivalence", "equivalence", {"left": ["node:2", "node:3"], "right": ["node:3", "node:2"], "semantic_order": True}, "SEQUENCE list order is not sorted to create equivalence."),
    case("computation-control/equivalence/same-snapshot-deterministic", "equivalence", "equivalence", {"left": [{"op": "race", "children": [2, 3]}], "right": [{"op": "race", "children": [2, 3]}]}, "One exact checked IR/context yields deterministic summaries."),
    case("computation-control/equivalence/presentation-metadata-only", "equivalence", "equivalence", {"left": [{"contract_body": "A", "label": "before"}], "right": [{"contract_body": "A", "label": "after"}]}, "Presentation metadata does not change semantic control facts."),
]


def write_case(item: dict) -> None:
    path = OUT / item["group"] / (item["case_id"].split("/")[-1] + ".json")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(item, indent=2, ensure_ascii=False, sort_keys=True) + "\n", encoding="utf-8", newline="\n")


def main() -> None:
    ids = [item["case_id"] for item in CASES]
    if len(ids) != len(set(ids)):
        raise ValueError("duplicate computation-control case id")
    verdicts = {item["expected"]["verdict"] for item in CASES}
    required = {"proven", "violation", "unproven", "requires_evidence"}
    if not required.issubset(verdicts):
        raise ValueError(f"missing verdict classes: {sorted(required - verdicts)}")
    for item in CASES:
        write_case(item)
    manifest = {
        "kind": "agentflow.conformance.computation-control.test-corpus",
        "corpus_revision": 1,
        "format_note": "static test vectors only; no AFIDL, ACE1 or HashV1 control identity",
        "case_count": len(CASES),
        "case_ids": sorted(ids),
    }
    path = OUT / "manifest.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False, sort_keys=True) + "\n", encoding="utf-8", newline="\n")
    print(f"generated {len(CASES)} computation-control vectors")


if __name__ == "__main__":
    main()
