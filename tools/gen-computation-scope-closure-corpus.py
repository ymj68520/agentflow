#!/usr/bin/env python3
"""Independent Phase 4B.2B.3 Scope/CompensationScope static oracle.

The oracle uses only frozen Semantic IR/contract facts and writes conformance
vectors. It never calls the Rust analyzer or invents runtime authority,
receipts, reservations, compensation dispatch, or commit order.
"""

from __future__ import annotations

import json
import copy
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "spec/conformance/computation-scope-closure"
U64_MAX = (1 << 64) - 1
VERDICT_ORDER = {"proven": 0, "unproven": 1, "requires_evidence": 2, "violation": 3}


def atom(member: str = "natural") -> dict:
    qualifiers = [] if member == "plain" else [{"family": "idempotency", "member": member}]
    return {
        "operation": {"catalog": "agentflow.effects@1", "name": "write"},
        "qualifiers": qualifiers,
        "resource_selector": {"requirement_port_no": 1, "scope_bindings": []},
    }


def entry(which: str, lower: int, upper: int) -> dict:
    return {"atom": atom(which), "guard": "unconditional", "lower": lower, "upper": upper}


def compensation_action(forward_upper: int, activation_upper: int, attempt_upper: int, per_attempt: int) -> dict:
    return {
        "action": "compensate",
        "action_atom": atom("none"),
        "activations_per_forward": {"lower": 0, "upper": activation_upper},
        "attempts_per_activation": {"lower": 1, "upper": attempt_upper},
        "clause": {
            "compensation": {"required": {"catalog": "agentflow.effects@1", "name": "write"}},
            "idempotency": "natural",
            "key_requirement": None,
            "reconciliation": "none",
            "unknown_effect_handling": "block",
        },
        "coverage_meters": ["api_calls"],
        "forward_atom": atom("natural"),
        "per_attempt_usage": {"coordinates": {"recovery/api_calls": per_attempt}},
        "_forward_upper": forward_upper,
    }


def leaf(node: int, *, effects: list[dict] | None = None, failures: list[str] | None = None,
         forward_usage: int = 0, recovery_usage: int = 0,
         recovery_actions: list[dict] | None = None) -> dict:
    entries = effects or []
    return {
        "caller_digest": "semantic-ir:caller",
        "contract_digest": f"component-contract:{node + 100:064x}",
        "effect_entries": entries,
        "effects": {"entries": entries, "residuals": [], "row": "closed"},
        "execution": None,
        "failures": failures or [],
        "flow": {"authority": "none", "constraints": [], "influence_unknown": False, "sources": [], "taints": []},
        "node": node,
        "outputs": [],
        "proof_source": "checked_component",
        "recovery_actions": recovery_actions,
        "resolved": True,
        "resources": None,
        "usage": {"coordinates": {"forward/api_calls": forward_usage, "recovery/api_calls": recovery_usage}},
    }


def compose_input(operator: str, children: list[dict], **options) -> dict:
    return {"operation": "compose", "operator": operator, "children": children, **options}


def direct_scope_cases() -> list[dict]:
    result = []
    for i in range(18):
        node = 2
        has_effect = i % 3 != 0
        effects = [] if not has_effect else [entry("natural", i % 2, 1 + i % 5)]
        recovery_actions = None
        recovery_usage = (i + 1) % 4
        if i == 17:
            forward_upper = effects[0]["upper"]
            action = compensation_action(forward_upper, 1, 1, 1)
            action.pop("_forward_upper")
            recovery_actions = [action]
            effects.append(entry("none", 0, forward_upper))
            recovery_usage = forward_upper
        auto = f"fixture:auto:{node}:1" if has_effect else None
        extra = f"unused-resource-{i + 1:02}"
        include_extra = i % 2 == 0
        scope_resources = ([auto] if auto else []) + ([extra] if include_extra else [])
        additional = [extra]
        child = leaf(
            node,
            effects=effects,
            failures=[[], ["FailureA"], ["FailureA", "FailureB"]][i % 3],
            forward_usage=i % 7,
            recovery_usage=recovery_usage,
            recovery_actions=recovery_actions,
        )
        if i == 17:
            child["execution"] = {"sandbox": "strict"}
        children = [child]
        result.append(make_case(
            f"computation-scope-closure/scope/resource-boundary-{i + 1:02}",
            "scope",
            compose_input("scope", children, scope_resources=scope_resources, additional_resources=additional),
            "requires_evidence",
        ))
    return result


def direct_compensation_cases() -> list[dict]:
    result = []
    for i in range(18):
        node = 2
        forward_upper = 1 + i % 5
        activation_upper = 1 + i % 3
        attempt_upper = 1 + (i // 2) % 3
        per_attempt = 1 + (i // 3) % 2
        recovery_upper = forward_upper * activation_upper * attempt_upper * per_attempt
        actions = [compensation_action(forward_upper, activation_upper, attempt_upper, per_attempt)]
        effects = [entry("natural", i % 2, forward_upper), entry("none", 0, recovery_upper)]
        child = leaf(
            node,
            effects=effects,
            failures=[[], ["FailureA"], ["FailureA", "FailureB"]][i % 3],
            forward_usage=(i * 2) % 9,
            recovery_usage=recovery_upper,
            recovery_actions=actions,
        )
        result.append(make_case(
            f"computation-scope-closure/compensation/action-bound-{i + 1:02}",
            "compensation",
            compose_input("compensation_scope", [child]),
            "requires_evidence",
        ))
    return result


def nested_cases() -> list[dict]:
    specifications = [
        ("select", [2, 3], [], 2, "unproven"),
        ("scope", [2], [{"node": 2, "operator": "parallel_all", "children": [3, 4]}], 2, "requires_evidence"),
        ("scope", [2], [{"node": 2, "operator": "retry", "child": 3, "delays_micros": [0, 2]}], 1, "requires_evidence"),
        ("compensation_scope", [2], [{"node": 2, "operator": "sequence", "children": [3, 4]}], 2, "unproven"),
        ("iterate_foreach", [2], [{"node": 2, "operator": "compensation_scope", "child": 3}], 1, "unproven"),
        ("scope", [2], [{"node": 2, "operator": "compensation_scope", "child": 3}], 1, "requires_evidence"),
        ("compensation_scope", [2], [{"node": 2, "operator": "scope", "child": 3, "scope_resources": []}], 1, "requires_evidence"),
        ("iterate_while", [2], [{"node": 2, "operator": "compensation_scope", "child": 3}], 1, "unproven"),
        ("handle", [2], [{"node": 2, "operator": "compensation_scope", "child": 3}], 1, "unproven"),
        ("scope", [2], [
            {"node": 2, "operator": "scope", "child": 3, "scope_resources": ["outer-resource"]},
            {"node": 3, "operator": "scope", "child": 4, "scope_resources": []},
        ], 1, "requires_evidence"),
        ("compensation_scope", [2], [{"node": 2, "operator": "retry", "child": 3, "delays_micros": [1]}], 1, "unproven"),
        ("scope", [2], [{"node": 2, "operator": "sequence", "children": [3, 4]}], 2, "requires_evidence"),
        ("scope", [2], [{"node": 2, "operator": "parallel_fail_fast", "children": [3, 4]}], 2, "requires_evidence"),
        ("compensation_scope", [2], [{"node": 2, "operator": "parallel_all", "children": [3, 4]}], 2, "unproven"),
        ("sequence", [2, 3], [
            {"node": 2, "operator": "scope", "child": 4, "scope_resources": []},
            {"node": 3, "operator": "compensation_scope", "child": 5},
        ], 2, "requires_evidence"),
        ("compensation_scope", [2], [{"node": 2, "operator": "race", "children": [3, 4]}], 2, "unproven"),
        ("scope", [2], [{"node": 2, "operator": "handle", "child": 3,
                           "clauses": [{"ordinal": 1, "match": {"failure": "FailureA"}, "handler": 4}]}], 2, "requires_evidence"),
        ("timeout", [2], [{"node": 2, "operator": "compensation_scope", "child": 3}], 1, "unproven"),
    ]
    result = []
    for index, (root_op, root_children, nested, leaf_count, verdict_value) in enumerate(specifications, 1):
        nested_ids = {node["node"] for node in nested}
        leaf_start = max(nested_ids | {1}) + 1
        # Use a compact disjoint leaf namespace and rewrite each nested child
        # endpoint; IDs are static source coordinates, not runtime identities.
        leaf_ids = list(range(leaf_start, leaf_start + leaf_count))
        cursor = 0
        rewritten = []
        for node in nested:
            item = dict(node)
            if "children" in item:
                targets = []
                for target in item["children"]:
                    if target in nested_ids:
                        targets.append(target)
                    else:
                        targets.append(leaf_ids[cursor])
                        cursor += 1
                item["children"] = targets
            elif item.get("operator") in ("scope", "compensation_scope", "retry", "timeout"):
                if item.get("child") not in nested_ids:
                    item["child"] = leaf_ids[cursor]
                    cursor += 1
            elif item.get("operator") == "handle":
                if item.get("child") not in nested_ids:
                    item["child"] = leaf_ids[cursor]
                    cursor += 1
                for clause in item["clauses"]:
                    if clause["handler"] not in nested_ids:
                        clause["handler"] = leaf_ids[cursor]
                        cursor += 1
            rewritten.append(item)
        # Root direct children already identify nested nodes for these shapes.
        leaves = []
        for pos, node in enumerate(leaf_ids):
            child = leaf(node, effects=[], failures=(["FailureA"] if index == 17 and pos == 0 else []),
                         forward_usage=0, recovery_usage=0)
            if index == 18 and pos == 0:
                child["leaf_kind"] = "await"
            if index == 17 and pos == 1:
                child["proof_source"] = "subflow_body_evidence"
            leaves.append(child)
        scope_resources = []
        additional_resources = []
        if root_op == "scope" and index == 10:
            scope_resources = ["outer-resource"]
            additional_resources = ["outer-resource"]
        input_value = compose_input(
            root_op,
            leaves,
            root_child_nodes=root_children,
            nested_nodes=rewritten,
            scope_resources=scope_resources,
            additional_resources=additional_resources,
            clauses=[] if root_op == "handle" else None,
            timeout_micros=5 if root_op == "timeout" else None,
            delays_micros=[1] if root_op == "retry" else None,
            item_count=2 if root_op == "iterate_foreach" else None,
            max_inflight=1 if root_op == "iterate_foreach" else None,
            max_iterations=2 if root_op == "iterate_while" else None,
            guard_value=True if root_op == "iterate_while" else None,
        )
        # Drop irrelevant null selectors; fixture code reads only the active
        # body's fields, while canonical IR has no placeholder properties.
        for key in ["clauses", "timeout_micros", "delays_micros", "item_count", "max_inflight", "max_iterations", "guard_value"]:
            if input_value.get(key) is None:
                input_value.pop(key, None)
        result.append(make_case(
            f"computation-scope-closure/nested/tree-{index:02}",
            "nested",
            input_value,
            verdict_value,
        ))
    return result


def generated_cases() -> list[dict]:
    compensation = direct_compensation_cases()
    underdeclared = copy.deepcopy(compensation[5])
    underdeclared["case_id"] = "computation-scope-closure/negative/action-row-underdeclared"
    underdeclared["group"] = "negative"
    action_row = underdeclared["input"]["children"][0]
    action_ordinal = next(
        index for index, item in enumerate(action_row["effect_entries"])
        if item["atom"] == action_row["recovery_actions"][0]["action_atom"]
    )
    action_row["effect_entries"][action_ordinal]["upper"] = 1
    action_row["effects"]["entries"][action_ordinal]["upper"] = 1
    underdeclared["expected"] = expected_projection(underdeclared["input"], "violation")
    underdeclared["note"] = "The checked K-derived compensation upper exceeds the exact declared epsilon action-effect row; production must fail closed."
    return direct_scope_cases() + compensation + nested_cases() + [underdeclared]


def finite_product(value: int | str, count: int | str | None) -> int | str:
    if count is None or count == "unbounded":
        return 0 if value == 0 else "unbounded"
    if value == "unbounded":
        return "unbounded" if count else 0
    return value * count


def input_children(input_value: dict, node: int) -> list[int]:
    leaves = {child["node"] for child in input_value["children"]}
    nested = {item["node"]: item for item in input_value.get("nested_nodes", [])}
    if node == 1:
        operator = input_value["operator"]
        roots = input_value.get("root_child_nodes", [item["node"] for item in input_value["children"]])
        if operator == "handle":
            return roots[:1] + [clause["handler"] for clause in input_value.get("clauses", [])]
        return roots
    if node in leaves:
        return []
    shape = nested.get(node)
    if shape is None:
        raise ValueError(f"missing owner node {node}")
    operator = shape["operator"]
    if operator == "handle":
        return [shape["child"]] + [clause["handler"] for clause in shape.get("clauses", [])]
    if operator in {"sequence", "parallel_all", "parallel_fail_fast", "race"}:
        return shape["children"]
    if operator in {"iterate_foreach", "iterate_while", "retry", "timeout", "scope", "compensation_scope"}:
        return [shape.get("body", shape.get("child"))]
    raise ValueError(f"unknown structured operator {operator}")


def computation_tree(input_value: dict) -> tuple[list[int], dict[int, list[int]], dict[int, dict]]:
    nested = {item["node"]: item for item in input_value.get("nested_nodes", [])}
    nodes = {1: {"operator": input_value["operator"], **input_value}}
    nodes.update(nested)
    nodes.update({item["node"]: {"operator": "leaf"} for item in input_value["children"]})
    children_by_node: dict[int, list[int]] = {}
    order: list[int] = []

    def visit(node: int) -> None:
        if node in order:
            raise ValueError(f"non-tree input repeats computation {node}")
        order.append(node)
        children = input_children(input_value, node)
        children_by_node[node] = children
        for child_node in children:
            visit(child_node)

    visit(1)
    return order, children_by_node, nodes


def resource_slot_ids(input_value: dict) -> dict[str, int]:
    aliases: dict[str, int] = {}

    def add(alias: str) -> None:
        if alias not in aliases:
            aliases[alias] = len(aliases) + 1

    for child in input_value["children"]:
        mapped = {
            (int(item["component_port"]), item["resource_slot"])
            for item in (child.get("resources") or [])
        }
        mapped_ports = {port for port, _ in mapped}
        effects = child.get("effect_entries", [])
        for declared in effects:
            port = declared["atom"]["resource_selector"]["requirement_port_no"]
            if port not in mapped_ports:
                mapped.add((port, f"fixture:auto:{child['node']}:{port}"))
        for _, alias in sorted(mapped):
            add(alias)

    resource_names = set(input_value.get("scope_resources", []))
    resource_names.update(input_value.get("additional_resources", []))
    for nested in input_value.get("nested_nodes", []):
        resource_names.update(nested.get("scope_resources", []))
        resource_names.update(item["resource_slot"] for item in nested.get("resources", []))
    for alias in sorted(resource_names):
        add(alias)
    return aliases


def scope_boundaries(input_value: dict, order: list[int], children_by_node: dict[int, list[int]], nodes: dict[int, dict]) -> list[dict]:
    slots = resource_slot_ids(input_value)
    root_resources = sorted(slots.values())
    found: list[dict] = []

    def visit(node: int, visible: list[int]) -> None:
        shape = nodes[node]
        operator = shape["operator"]
        declared = visible
        if operator == "scope":
            aliases = shape.get("scope_resources", [])
            declared = sorted(slots[alias] for alias in aliases)
            found.append({
                "control_node": node,
                "child": children_by_node[node][0],
                "parent_visible_resources": visible,
                "declared_resources": declared,
                "child_visible_resources": declared,
                "resource_attenuation": "proven",
                "authority_attenuation": "unproven",
                "budget_attenuation": "unproven",
                "owned_children_settle_before_terminal": "requires_evidence",
            })
        for child_node in children_by_node[node]:
            visit(child_node, declared if operator == "scope" else visible)

    visit(1, root_resources)
    return found


def occurrence_families(input_value: dict, source_node: int, nodes: dict[int, dict]) -> list[tuple[int, int | str]]:
    parents: dict[int, int] = {}
    _, children_by_node, _ = computation_tree(input_value)
    for owner, children in children_by_node.items():
        for child in children:
            parents[child] = owner
    chain = []
    current = source_node
    while current in parents:
        current = parents[current]
        chain.append(current)
    factors: list[tuple[int, int | str]] = []
    for owner in reversed(chain):
        shape = nodes[owner]
        operator = shape["operator"]
        if operator == "iterate_foreach":
            count = shape.get("item_count")
            factors.append((owner, "unbounded" if count is None else count))
        elif operator == "iterate_while":
            factors.append((owner, shape["max_iterations"]))
        elif operator == "retry":
            factors.append((owner, len(shape.get("delays_micros", [])) + 1))
    return factors


def compensation_scopes(input_value: dict, order: list[int], children_by_node: dict[int, list[int]], nodes: dict[int, dict]) -> list[dict]:
    leaves = {child["node"]: child for child in input_value["children"]}
    scopes: list[dict] = []

    def descendants(node: int) -> list[int]:
        result = []
        for child in children_by_node[node]:
            result.append(child)
            result.extend(descendants(child))
        return result

    for node in order:
        if nodes[node]["operator"] != "compensation_scope":
            continue
        actions = []
        descendant_nodes = descendants(node)
        unresolved_sources = [source_node for source_node in descendant_nodes
                              if source_node in leaves and leaves[source_node].get("leaf_kind") == "await"]
        for source_node in descendant_nodes:
            child = leaves.get(source_node)
            if child is None:
                continue
            for action in child.get("recovery_actions") or []:
                if action.get("action") != "compensate":
                    continue
                for ordinal, forward in enumerate(child.get("effect_entries", [])):
                    if forward["atom"] != action["forward_atom"]:
                        continue
                    upper: int | str = forward["upper"]
                    for _, count in occurrence_families(input_value, source_node, nodes):
                        upper = finite_product(upper, count)
                    upper = finite_product(upper, action["activations_per_forward"]["upper"])
                    upper = finite_product(upper, action["attempts_per_activation"]["upper"])
                    if upper == "unbounded":
                        receipt = "requires_evidence"
                        usage_upper: int | str = 0 if action["per_attempt_usage"]["coordinates"].get("recovery/api_calls", 0) == 0 else "unbounded"
                    else:
                        receipt = "requires_evidence" if upper > 0 else "proven"
                        usage_upper = upper * action["per_attempt_usage"]["coordinates"].get("recovery/api_calls", 0)
                    action_ordinal = next((index for index, effect in enumerate(child.get("effect_entries", []))
                                           if effect["atom"] == action["action_atom"]), None)
                    if action_ordinal is None:
                        unresolved_sources.append(source_node)
                        continue
                    port = action["action_atom"]["resource_selector"]["requirement_port_no"]
                    declared_upper = child["effect_entries"][action_ordinal]["upper"]
                    mapped_resource_slot = next((item["resource_slot"] for item in (child.get("resources") or [])
                                                  if item["component_port"] == port),
                                                 f"fixture:auto:{source_node}:{port}")
                    actions.append({
                        "source_node": source_node,
                        "forward_occurrence": {"node": source_node, "entry_ordinal": ordinal, "families": []},
                        "action_effect_occurrence": {
                            "node": source_node,
                            "entry_ordinal": action_ordinal,
                            "atom": action["action_atom"],
                            "declared_upper": declared_upper,
                            "mapped_resource_slot": mapped_resource_slot,
                        },
                        "identity": {"action_kind": "compensation"},
                        "potential_effect": {"lower": 0, "upper": upper},
                        "potential_recovery_usage": {"state": "known", "coordinates": {"recovery/api_calls": usage_upper}},
                        "receipt_evidence": receipt,
                    })
                    if upper == "unbounded" or declared_upper == "unbounded" or declared_upper < upper:
                        unresolved_sources.append(source_node)
        unresolved_sources = sorted(set(unresolved_sources))
        total_upper: int | str = 0
        for action in actions:
            total_upper = "unbounded" if "unbounded" in (total_upper, action["potential_effect"]["upper"]) else total_upper + action["potential_effect"]["upper"]
        compensation_order = "requires_evidence" if total_upper == "unbounded" or total_upper > 1 else "proven"
        receipt_required = any(action["receipt_evidence"] == "requires_evidence" for action in actions)
        if unresolved_sources and actions:
            potential_actions = {"state": "partial", "actions": actions, "unresolved_sources": unresolved_sources}
        elif unresolved_sources:
            potential_actions = {"state": "unknown", "known_contributions": [], "unresolved_sources": unresolved_sources}
        else:
            potential_actions = {"state": "complete", "actions": actions}
        summary = {
            "potential_actions": potential_actions,
            "compensation_order": compensation_order,
            "recovery_blocked_outcome": "unproven" if unresolved_sources else ("requires_evidence" if receipt_required else "unproven"),
            "forward_effects_retained": True,
            "forward_outcome_retained": True,
        }
        scopes.append({"control_node": node, "summary": summary})
    return scopes


def expected_projection(input_value: dict, expected_root_verdict: str) -> dict:
    order, children_by_node, nodes = computation_tree(input_value)
    leaves = {child["node"] for child in input_value["children"]}
    boundaries = scope_boundaries(input_value, order, children_by_node, nodes)
    compensations = compensation_scopes(input_value, order, children_by_node, nodes)
    by_id = {item["control_node"]: item for item in boundaries}
    root_boundary = by_id.get(1)
    root_compensation = next((item["summary"] for item in compensations if item["control_node"] == 1), None)
    projection = {
        "verdict": expected_root_verdict,
        "scope_boundary": root_boundary,
        "scope_boundaries_by_node": boundaries,
        "compensation_scope": root_compensation,
        "compensation_scopes_by_node": compensations,
        "workflow_closure": {
            "computation_order": order,
            "leaf_order": [node for node in order if node in leaves],
            "known_effect_occurrences_accounted": True,
            "known_effect_residuals_accounted": True,
            "known_recovery_sources_accounted": True,
            "root_export_compatibility": "not_expressed_in_semantic_ir",
            "closure_verdict": "unproven",
            "overall_verdict": max((expected_root_verdict, "unproven"), key=VERDICT_ORDER.__getitem__),
        },
    }
    if input_value["operator"] in {"scope", "compensation_scope"} and len(input_value.get("root_child_nodes", [item["node"] for item in input_value["children"]])) == 1:
        child_node = input_value.get("root_child_nodes", [item["node"] for item in input_value["children"]])[0]
        child = next((item for item in input_value["children"] if item["node"] == child_node), None)
        if child is not None:
            projection.update({
                "effect_row_complete": True,
                "effects": {"row": "closed"},
                "effect_occurrences": [
                    {"node": child_node, "entry_ordinal": ordinal, "atom": declared["atom"]}
                    for ordinal, declared in enumerate(child.get("effect_entries", []))
                ],
                "failures": sorted(child.get("failures", [])),
                "usage": {"state": "known", "coordinates": child["usage"]["coordinates"]},
                "execution_by_child": ([{"node": child_node, "x": {"sandbox": "strict"}}]
                                       if (child.get("execution") or {}).get("sandbox") == "strict" else []),
                "recovery_sources": ([{
                    "node": child_node,
                    "actions": [{
                        "action": action["action"],
                        "forward_atom": action["forward_atom"],
                        "action_atom": action["action_atom"],
                    } for action in child.get("recovery_actions") or []],
                }] if child.get("recovery_actions") else []),
            })
    return projection


def make_case(case_id: str, group: str, input_value: dict, expected_root_verdict: str) -> dict:
    input_value["include_scope_details"] = True
    input_value["include_workflow_closure"] = True
    return {
        "case_id": case_id,
        "expected": expected_projection(input_value, expected_root_verdict),
        "group": group,
        "input": input_value,
        "kind": "agentflow.conformance.computation-scope-closure.test-vector",
        "note": "Python-only oracle over frozen Semantic IR fields; no runtime grant, budget reservation, receipt state, compensation dispatch or commit order is inferred.",
    }


def write_json(path: Path, value: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")


def main() -> None:
    cases = generated_cases()
    case_ids = sorted(case["case_id"] for case in cases)
    for case in cases:
        write_json(OUT / case["group"] / f"{case['case_id'].split('/')[-1]}.json", case)
    write_json(OUT / "manifest.json", {
        "case_count": len(cases),
        "case_ids": case_ids,
        "corpus_revision": 1,
        "format_note": "static differential vectors only; not ABI/ACE/hash/runtime fixtures",
        "kind": "agentflow.conformance.computation-scope-closure.test-corpus",
    })
    print(f"wrote {len(cases)} independent Scope/CompensationScope vectors")


if __name__ == "__main__":
    main()
