#!/usr/bin/env python3
"""Independent Phase 4B.2E.1 guard and root-recovery oracle.

Expected values are derived only from the case's declarative inputs. This
generator never invokes Rust, reads production projections, or rewrites an
Expected file from observed analyzer output.
"""

from __future__ import annotations

import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "spec/conformance/computation-root-recovery"
U64_MAX = (1 << 64) - 1


def atom(member: str = "natural", port: int = 1) -> dict:
    qualifiers = [] if member == "plain" else [{"family": "idempotency", "member": member}]
    return {
        "operation": {"catalog": "agentflow.effects@1", "name": "write"},
        "qualifiers": qualifiers,
        "resource_selector": {"requirement_port_no": port, "scope_bindings": []},
    }


def entry(effect_atom: dict, upper: int | str, guard: object = "unconditional", lower: int = 0) -> dict:
    return {"atom": effect_atom, "guard": guard, "lower": lower, "upper": upper}


def action_item(
    forward: dict,
    action: dict,
    forward_upper: int,
    activation: int,
    attempts: int,
    per_attempt: int,
) -> dict:
    return {
        "_forward_upper": forward_upper,
        "action": "compensate",
        "action_atom": action,
        "activations_per_forward": {"lower": 0, "upper": activation},
        "attempts_per_activation": {"lower": 1, "upper": attempts},
        "clause": {
            "compensation": {"required": {"catalog": "agentflow.effects@1", "name": "write"}},
            "idempotency": "natural",
            "key_requirement": None,
            "reconciliation": "none",
            "unknown_effect_handling": "block",
        },
        "coverage_meters": ["api_calls"],
        "forward_atom": forward,
        "per_attempt_usage": {"coordinates": {"recovery/api_calls": per_attempt}},
    }


def leaf(
    node: int,
    alias: str,
    *,
    forward: dict | None = None,
    action: dict | None = None,
    forward_upper: int = 1,
    forward_lower: int = 0,
    action_epsilon_upper: int | str | None = None,
    activation: int = 1,
    attempts: int = 1,
    per_attempt: int = 1,
    guard_forward: object = "unconditional",
    guard_action: object = "unconditional",
    input_types: list[dict] | None = None,
    recovery_usage: int | str | None = None,
    recovery_actions: list[dict] | None = None,
    omit_usage_binding: bool = False,
    effects_row: str = "closed",
    failures: list[str] | None = None,
    resource_alias: str | None = None,
) -> dict:
    forward = forward or atom("natural")
    action = action or atom("none")
    if recovery_actions is None:
        recovery_actions = [action_item(forward, action, forward_upper, activation, attempts, per_attempt)]
    if action_epsilon_upper is None:
        local_action = forward_upper * activation * attempts
        action_epsilon_upper = local_action + forward_upper if forward == action else local_action
    effects = [entry(forward, forward_upper, guard_forward, forward_lower)]
    if action != forward:
        effects.append(entry(action, action_epsilon_upper, guard_action))
    else:
        effects[0] = entry(forward, action_epsilon_upper, guard_action, forward_lower)
    if effects_row != "closed":
        effects = []
    usage_bound = (
        forward_upper * activation * attempts * per_attempt
        if recovery_usage is None
        else recovery_usage
    )
    result = {
        "caller_digest": "semantic-ir:caller",
        "contract_digest": alias,
        "effects": {"entries": effects, "residuals": [], "row": effects_row},
        "execution": None,
        "failures": failures or [],
        "flow": {"authority": "none", "constraints": [], "influence_unknown": False, "sources": [], "taints": []},
        "node": node,
        "outputs": [],
        "proof_source": "checked_component",
        "recovery_actions": recovery_actions,
        "resolved": True,
        "resources": [] if not effects else [
            {"component_port": 1, "resource_slot": resource_alias or f"shared-resource:{node}"}
        ],
        "usage": {"coordinates": {"forward/api_calls": 0, "recovery/api_calls": usage_bound}},
    }
    if input_types:
        result["input_types"] = input_types
    if omit_usage_binding:
        result["omit_usage_binding"] = True
    return result


def scope_child(node: int, op: str, child: dict, **options) -> dict:
    return {"node": node, "operator": op, "child": child, **options}


def retry_node(node: int, child: dict, attempts: int) -> dict:
    return scope_child(node, "retry", child, delays_micros=[0] * (attempts - 1))


def sequence_node(node: int, children: list[dict]) -> dict:
    return {"node": node, "operator": "sequence", "children": children}


def parallel_node(node: int, op: str, children: list[dict]) -> dict:
    return {"node": node, "operator": op, "children": children}


def leaf_node(source: dict) -> dict:
    return {"node": source["node"], "operator": "leaf"}


def collect_source_paths(tree: dict) -> dict[int, dict]:
    paths: dict[int, dict] = {}

    def walk(node: dict, factors: list[dict], local_factors: list[dict], owner: int | None, path: list[dict], reachable: bool, local_reachable: bool):
        op = node["operator"]
        current_owner = owner
        current_local = list(local_factors)
        current_local_reachable = local_reachable
        if op == "compensation_scope":
            current_owner = node["node"]
            current_local = []
            current_local_reachable = True
        current_factors = list(factors)
        if op in ("retry", "iterate_foreach", "iterate_while"):
            if op == "retry":
                count = len(node.get("delays_micros", [])) + 1
                kind = "retry"
            else:
                count = node.get("item_count", node.get("max_iterations"))
                kind = "iterate"
                if count is None:
                    count = "unbounded"
            factor = {"control_node": node["node"], "kind": kind, "upper": count}
            current_factors.append(factor)
            if current_owner is not None:
                current_local.append(factor)
        if op == "leaf":
            paths[node["node"]] = {
                "owner": current_owner,
                "root_factors": current_factors,
                "local_factors": current_local,
                "path": path,
                "root_reachable": reachable,
                "local_reachable": current_local_reachable,
            }
            return
        children = node.get("children", [node.get("child")])
        for index, child in enumerate(children):
            kind = {
                "sequence": "sequence_child",
                "select": "select_case",
                "parallel_all": "parallel_branch",
                "parallel_fail_fast": "parallel_branch",
                "race": "race_branch",
                "handle": "handle_protected" if index == 0 else "handle_clause",
                "compensation_scope": "compensation_scope_child",
                "scope": "scope_child",
                "retry": "retry_child",
                "iterate_foreach": "iterate_body",
                "iterate_while": "iterate_body",
            }.get(op, f"{op}_child")
            reachability = node.get("reachability", "possible")
            if op == "select" and node.get("exact_child") is not None:
                reachability = "exact_selected" if index == node["exact_child"] else "proven_unreachable"
            child_reachable = reachable and reachability != "proven_unreachable"
            child_local_reachable = current_local_reachable
            if current_owner is not None and op == "select" and reachability == "proven_unreachable":
                child_local_reachable = False
            next_path = path + [{"parent_node": node["node"], "child_node": child["node"], "kind": kind, "reachability": reachability}]
            walk(child, current_factors, current_local, current_owner, next_path, child_reachable, child_local_reachable)

    walk(tree, [], [], None, [], True, True)
    return paths


def scale(value: int | str, factor: int | str) -> int | str:
    if value == "unknown" or factor == "unknown":
        return "unknown"
    if value == "unbounded" or factor == "unbounded":
        return "unbounded" if value != 0 and factor != 0 else 0
    product = value * factor
    if product > U64_MAX:
        return "overflow"
    return product


def add(left: int | str, right: int | str) -> int | str:
    if left == "unknown" or right == "unknown":
        return "unknown"
    if left == "unbounded" or right == "unbounded":
        return "unbounded"
    if left == "overflow" or right == "overflow" or left + right > U64_MAX:
        return "overflow"
    return left + right


def atom_key(value: dict) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"))


def fold_tree(tree: dict, sources: dict[int, dict], projection: str) -> dict[str, int | str]:
    op = tree["operator"]
    if op == "leaf":
        source = sources[tree["node"]]
        if projection == "action":
            return {atom_key(source["action"]): source["base"]}
        if projection == "declared_epsilon":
            return {atom_key(source["action"]): source["epsilon_upper"]}
        if projection in ("usage", "declared_usage"):
            declared = source.get("usage_declaration")
            value = (declared if declared is not None else source["base"] * source["per_attempt"]) if projection == "declared_usage" else source["base"] * source["per_attempt"]
            return {"recovery/api_calls": value}
        result = {atom_key(source["action"]): source["base"]}
        if source["action"] == source["forward"]:
            key = atom_key(source["forward"])
            result[key] = add(result.get(key, 0), source["forward_upper"])
        return result
    children = tree.get("children", [tree.get("child")])
    branch_results = [fold_tree(child, sources, projection) for child in children]
    if op in ("retry", "iterate_foreach", "iterate_while"):
        if op == "retry":
            factor = len(tree.get("delays_micros", [])) + 1
        else:
            factor = tree.get("item_count", tree.get("max_iterations"))
        if op == "iterate_foreach" and factor is None and projection in ("usage", "declared_usage"):
            return {key:"unknown" for key in branch_results[0]}
        return {key: scale(value, factor if factor is not None else "unbounded") for key, value in branch_results[0].items()}
    if op in ("sequence", "parallel_all", "parallel_fail_fast", "race", "handle"):
        if op == "handle" and len(branch_results) > 1:
            branch_results = [branch_results[0], join_maps(branch_results[1:])]
        return sum_maps(branch_results)
    if op == "select":
        if tree.get("exact_child") is not None:
            return branch_results[tree["exact_child"]]
        return join_maps(branch_results)
    if op in ("compensation_scope", "scope", "timeout"):
        return branch_results[0]
    raise ValueError(f"oracle has no root algebra for {op}")


def sum_maps(maps: list[dict[str, int | str]]) -> dict[str, int | str]:
    result: dict[str, int | str] = {}
    for item in maps:
        for key, value in item.items():
            result[key] = add(result.get(key, 0), value)
    return result


def join_maps(maps: list[dict[str, int | str]]) -> dict[str, int | str]:
    result: dict[str, int | str] = {}
    for item in maps:
        for key, value in item.items():
            old = result.get(key, 0)
            if value == "unbounded" or old == "unbounded":
                result[key] = "unbounded"
            elif value == "unknown" or old == "unknown":
                result[key] = "unknown"
            elif value == "overflow" or old == "overflow":
                result[key] = "overflow"
            else:
                result[key] = max(old, value)
    return result


def guard_identity(forward: dict, action: dict) -> dict:
    return {"forward_effect": forward, "action_kind": "compensation", "action_effect": action}


def primitive_type_name(value: object) -> str:
    if isinstance(value, bool):
        return "bool"
    if isinstance(value, int):
        return "int64"
    if isinstance(value, str):
        return "string"
    if isinstance(value, dict) and "unit" in value:
        return "unit"
    if isinstance(value, dict) and "null" in value:
        return "null"
    return "int64"


def type_schema_name(primitive: str) -> str:
    return "Guard" + primitive[:1].upper() + primitive[1:]


def guard_case(case_id: str, forward_guard: object, action_guard: object, input_types: list[dict] | None = None,
               *, effect_row: str = "closed", action_present: bool = True, root: dict | None = None,
               sources: list[dict] | None = None) -> dict:
    forward = atom("natural")
    action = atom("none")
    source = leaf(
        2,
        "component-contract:guard-source-01",
        forward=forward,
        action=action,
        forward_upper=1,
        action_epsilon_upper=1,
        guard_forward=forward_guard,
        guard_action=action_guard,
        input_types=input_types,
        effects_row=effect_row,
        recovery_actions=None if action_present else [
            action_item(forward, action, 1, 1, 1, 1)
        ],
    )
    if not action_present:
        source["effects"] = {"entries": [entry(forward, 1, forward_guard)], "residuals": [], "row": "closed"}
    if root is None:
        fixture_input = {"operation": "validate_contract", "contract": source}
        stage = "component_validation"
    else:
        all_sources = sources or [source]
        fixture_input = make_control_input(root, all_sources)
        stage = "root_recovery"
    oracle = {
        "forward_guard": forward_guard,
        "action_guard": action_guard,
        "input_types": input_types or [],
        "effect_row": effect_row,
        "action_present": action_present,
    }
    expected = expected_guard_component(oracle) if stage == "component_validation" else expected_root(root, all_sources)
    return {"case_id": case_id, "stage": stage, "input": fixture_input, "oracle": oracle, "expected": expected}


def expected_guard_component(oracle: dict) -> dict:
    forward = oracle["forward_guard"]
    action = oracle["action_guard"]
    def serialized_guard(value: object) -> object:
        if isinstance(value, dict) and isinstance(value.get("input_equals"), dict):
            item = value["input_equals"]
            literal = item["value"]
            if isinstance(literal, dict) and "unit" in literal:
                literal = "unit"
            elif isinstance(literal, dict) and "null" in literal:
                literal = "null"
            return {"input_equals":{"port":item["port"],"value":literal}}
        return value
    inputs = {item["port"]: item["primitive"] for item in oracle["input_types"]}
    verdict = "proven"
    basis = "frozen_guard_containment"
    countermodel = None
    if oracle["effect_row"] != "closed":
        verdict, basis = "unproven", "insufficient_evidence"
    elif not oracle["action_present"]:
        verdict, basis = "unproven", "insufficient_evidence"
    elif oracle.get("action_upper") == 0:
        verdict, basis = "proven", "zero_possible_actions"
    elif forward == "unconditional" and isinstance(action, dict):
        expected_value = action["input_equals"]["value"]
        port = action["input_equals"]["port"]
        if inputs.get(port) in ("unit", "null"):
            verdict, basis = "unproven", "insufficient_evidence"
        else:
            verdict, basis = "violation", "typed_countermodel"
            counter_value = 1 if expected_value == 0 else 0
            countermodel = {
                "assignments": [{"port": port, "value": counter_value,"exact_type":{"type_name":type_schema_name(inputs.get(port,"int64"))}}],
                "reachability": "well_typed_component_input_domain",
                "related_rule_id": "AF-CONTRACT-0022",
            }
    elif isinstance(forward, dict) and isinstance(action, dict):
        f = forward["input_equals"]
        a = action["input_equals"]
        if f["port"] == a["port"] and f["value"] == a["value"]:
            verdict, basis = "proven", "frozen_guard_containment"
        elif inputs.get(f["port"]) in ("unit", "null") or inputs.get(a["port"]) in ("unit", "null"):
            verdict, basis = "unproven", "insufficient_evidence"
        else:
            verdict, basis = "violation", "typed_countermodel"
            assignments = [{"port": f["port"], "value": f["value"]}]
            if f["port"] == a["port"]:
                assignments[0]["value"] = f["value"]
            else:
                assignments.append({"port": a["port"], "value": 1 if a["value"] == 0 else 0})
            countermodel = {
                "assignments": [
                    {**item,"exact_type":{"type_name":type_schema_name(inputs.get(item["port"],"int64"))}}
                    for item in assignments
                ],
                "reachability": "well_typed_component_input_domain",
                "related_rule_id": "AF-CONTRACT-0022",
            }
    elif forward == "unconditional" and action == "unconditional":
        verdict, basis = "proven", "frozen_guard_containment"
    elif isinstance(forward, dict) and action == "unconditional":
        verdict, basis = "proven", "frozen_guard_containment"
    overall = oracle.get("component_verdict") or ("proven" if verdict == "proven" else "violation" if verdict == "violation" else "unproven")
    if not oracle["action_present"]:
        overall = "violation"
    return {
        "stage": "contract_validation",
        "verdict": overall,
        "checked_component_produced": overall == "proven",
        "action_guard_compatibility": [{
            "verdict": verdict,
            "basis": basis,
            "forward_guard": serialized_guard(forward) if oracle["effect_row"] == "closed" else None,
            "declared_action_guard": serialized_guard(action) if oracle["effect_row"] == "closed" and oracle["action_present"] else None,
            "countermodel": countermodel,
        }],
    }


def leaf_production(
    source: dict,
    *,
    guards: tuple[object, object] = ("unconditional", "unconditional"),
    input_types: list[dict] | None = None,
) -> dict:
    forward = source["forward"]
    action = source["action"]
    forward_upper = source["forward_upper"]
    action_upper = source["base"]
    epsilon_upper = source["epsilon_upper"]
    row_usage = source.get("usage_declaration")
    return leaf(
        source["node"],
        source["alias"],
        forward=forward,
        action=action,
        forward_upper=forward_upper,
        forward_lower=source.get("forward_lower", 0),
        action_epsilon_upper=epsilon_upper,
        activation=source["activation"],
        attempts=source["attempts"],
        per_attempt=source["per_attempt"],
        guard_forward=guards[0],
        guard_action=guards[1],
        input_types=input_types,
        recovery_usage=row_usage,
        resource_alias=source.get("resource_alias"),
        failures=source.get("failures"),
    )


def make_control_input(tree: dict, sources: list[dict], *, extra: dict | None = None) -> dict:
    nested: list[dict] = []
    source_by_node = {source["node"]: source for source in sources}
    next_node = 10

    def materialize(node: dict, root: bool = False) -> int:
        nonlocal next_node
        if node["operator"] == "leaf":
            return node["node"]
        node_id = 1 if root else next_node
        if not root:
            next_node += 1
        children = node.get("children", [node.get("child")])
        child_ids = [materialize(child) for child in children]
        op = node["operator"]
        if root:
            result["operator"] = op
            result["root_child_nodes"] = child_ids
            result.update(node.get("root_options", {}))
        else:
            item = {"node": node_id, "operator": op}
            if op in ("sequence", "parallel_all", "parallel_fail_fast", "race"):
                item["children"] = child_ids
            elif op == "retry":
                item.update({"child": child_ids[0], "delays_micros": node.get("delays_micros", [])})
            elif op in ("compensation_scope", "scope", "timeout"):
                item["child"] = child_ids[0]
            elif op == "iterate_foreach":
                item.update({
                    "body": child_ids[0],
                    "source_expression": 1,
                    "item_slot": 40_000,
                    "item_count": node.get("item_count"),
                    "max_inflight": node.get("max_inflight"),
                })
            elif op == "handle":
                item.update({"child": child_ids[0], "clauses": node.get("clauses", [])})
            else:
                raise ValueError(f"unsupported nested fixture operator {op}")
            nested.append(item)
        return node_id

    result = {"operation": "compose", "children": [leaf_production(source) for source in sources]}
    materialize(tree, root=True)
    result["include_workflow_closure"] = True
    result["include_scope_details"] = True
    result["nested_nodes"] = nested
    if tree["operator"] == "iterate_while":
        result["max_iterations"] = tree.get("max_iterations")
        result["guard_value"] = True
    if tree["operator"] == "iterate_foreach":
        result["item_count"] = tree.get("item_count")
        result["max_inflight"] = tree.get("max_inflight")
    if tree["operator"] == "retry":
        result["delays_micros"] = tree.get("delays_micros", [])
    if tree["operator"] == "select":
        result["exact_child"] = tree.get("exact_child")
    if tree["operator"] == "handle":
        result["clauses"] = tree.get("clauses", [])
    if extra:
        result.update(extra)
    result["children"] = [
        leaf_production(source, guards=source.get("guards", ("unconditional", "unconditional")), input_types=source.get("input_types"))
        for source in sources
    ]
    return result


def expected_root(tree: dict, sources_list: list[dict], *, guard_verdict: str = "proven") -> dict:
    paths = collect_source_paths(tree)
    sources = {}
    actions_expected = []
    for source in sources_list:
        path = paths[source["node"]]
        root_bound = source["base"]
        for factor in path["root_factors"]:
            root_bound = scale(root_bound, factor["upper"])
        if not path["root_reachable"]:
            root_bound = 0
        local_bound = source["base"]
        for factor in path["local_factors"]:
            local_bound = scale(local_bound, factor["upper"])
        if not path["local_reachable"]:
            local_bound = 0
        source = {**source, "root_bound": root_bound, "local_bound": local_bound, "path_info": path}
        sources[source["node"]] = source
        factors = [
            {"control_node": factor["control_node"], "kind": factor["kind"], "logical_repetitions": {"lower": 0 if factor["kind"] == "iterate" else 1, "upper": factor["upper"]}, "physical_attempts": {"lower": 1, "upper": factor["upper"]} if factor["kind"] == "retry" else None}
            for factor in path["root_factors"]
        ]
        local_factors = [
            {"control_node": factor["control_node"], "kind": factor["kind"], "logical_repetitions": {"lower": 0 if factor["kind"] == "iterate" else 1, "upper": factor["upper"]}, "physical_attempts": {"lower": 1, "upper": factor["upper"]} if factor["kind"] == "retry" else None}
            for factor in path["local_factors"]
        ]
        action_identity = guard_identity(source["forward"], source["action"])
        action_guard = source.get("guards", ("unconditional", "unconditional"))[1]
        forward_guard = source.get("guards", ("unconditional", "unconditional"))[0]
        source_digest = source["alias"]
        resource_alias = source.get("resource_alias") or f"shared-resource:{source['node']}"
        owner = {"kind": "compensation_scope", "node": path["owner"]} if path["owner"] is not None else {"kind": "root_recovery"}
        local_json = None if path["owner"] is None else {
            "coordinate": "one_compensation_scope_activation",
            "compensation_scope_node": path["owner"],
            "upper": upper_json(local_bound),
            "applied_families": [{"identity":{"control_node":item["control_node"],"kind":item["kind"]},"logical_repetitions":item["logical_repetitions"],"physical_attempts":item["physical_attempts"]} for item in local_factors],
        }
        action_usage = scale(source["base"], source["per_attempt"])
        for factor in path["root_factors"]:
            action_usage = scale(action_usage, factor["upper"])
        if not path["root_reachable"]:
            action_usage = 0
        source_profile = {
            "action": action_identity,
            "activations_per_forward": {"lower": 0, "upper": source["activation"]},
            "attempts_per_activation": {"lower": 1, "upper": source["attempts"]},
            "per_attempt_usage": {"limits": [{"meter":{"catalog":"agentflow.meters@1","name":"api_calls"},"phase":"recovery","upper":source["per_attempt"]}]},
        }
        input_type = source.get("input_types", [])
        guard_coordinates = []
        if action_guard != "unconditional" or forward_guard != "unconditional":
            for guard in (forward_guard, action_guard):
                if guard == "unconditional":
                    continue
                port = guard["input_equals"]["port"]
                if not any(item.get("component_input_port") == port for item in guard_coordinates):
                    schema = next((item["primitive"] for item in input_type if item["port"] == port), "int64")
                    guard_coordinates.append({"source_contract_digest":source_digest,"component_input_port":port,"exact_type":{"type_name":type_schema_name(schema)}})
        actions_expected.append({
            "source_node": source["node"],
            "source_contract_digest": source_digest,
            "forward_entry_ordinal": 0,
            "forward_seed_upper": source["forward_upper"],
            "action": action_identity,
            "owner": owner,
            "local_action_bound": local_json,
            "root_upper": root_bound,
            "root_factors": [{"control_node":item["control_node"],"kind":item["kind"],"upper":item["upper"]} for item in path["root_factors"]],
            "recovery_usage_binding": source_profile,
            "root_usage": action_usage,
            "guard_verdict": "proven" if source.get("guard_verdict", guard_verdict) == "proven" else source.get("guard_verdict"),
            "guard_basis": "zero_possible_actions" if source["base"] == 0 else "frozen_guard_containment",
            "guard_coordinates": guard_coordinates,
            "path": path["path"],
            "resource_alias": resource_alias,
        })

    action_map = fold_tree(tree, sources, "action")
    epsilon_map = fold_tree(tree, sources, "epsilon")
    declared_epsilon_map = fold_tree(tree, sources, "declared_epsilon")
    usage_map = fold_tree(tree, sources, "usage")
    declared_usage_map = fold_tree(tree, sources, "declared_usage")
    action_entries = map_entries(action_map, sources_list, "action")
    epsilon_entries = map_entries(epsilon_map, sources_list, "epsilon")
    action_verdict = "proven"
    for key, required in epsilon_map.items():
        declared = declared_epsilon_map.get(key)
        if declared is not None and isinstance(required, int) and isinstance(declared, int) and required > declared:
            action_verdict = "violation"
    usage_total = next(iter(usage_map.values()), 0)
    declared_usage = next(iter(declared_usage_map.values()), 0)
    usage_verdict = "proven" if isinstance(usage_total, int) and isinstance(declared_usage, int) and usage_total <= declared_usage else "unproven" if usage_total == "unknown" or declared_usage == "unknown" else "violation"
    if action_verdict == "violation":
        overall = "violation"
    elif usage_verdict == "violation":
        overall = "violation"
    elif usage_verdict == "unproven":
        overall = "requires_evidence" if any(item["root_upper"] not in (0, "overflow") for item in actions_expected) else "unproven"
    elif any(item["root_upper"] not in (0, "overflow") for item in actions_expected):
        overall = "requires_evidence"
    else:
        overall = "proven"
    source_closure = {
        "source_nodes_accounted": "proven",
        "action_bindings_accounted": "proven",
        "usage_profiles_accounted": "proven",
        "multiplicity_factors_accounted": "proven",
        "root_action_totals_sound": "proven",
    }
    root_actions = []
    for item in actions_expected:
        local = item["local_action_bound"]
        root_actions.append({
            "source": {
                "source_node": item["source_node"],
                "source_contract_digest": item["source_contract_digest"],
                "forward_entry_ordinal": item["forward_entry_ordinal"],
                "action": item["action"],
            },
            "forward_seed_upper":item["forward_seed_upper"],
            "owner": item["owner"],
            "local_action_bound": local,
            "root_action_bound": {
                "upper": upper_json(item["root_upper"]),
                "applied_families": [{"identity":{"control_node":factor["control_node"],"kind":factor["kind"]},"logical_repetitions":{"upper":factor["upper"]},"physical_attempts":{"upper":factor["upper"]} if factor["kind"] == "retry" else None} for factor in item["root_factors"]],
            },
            "recovery_usage_binding": item["recovery_usage_binding"],
            "root_recovery_usage": usage_json_value(item["root_usage"]),
            "guard_compatibility": {"verdict":item["guard_verdict"],"basis":item["guard_basis"]},
            "guard_coordinates": item["guard_coordinates"],
            "eligibility_evidence": "requires_evidence" if item["root_upper"] not in (0, "overflow") else "proven",
            "control_path": item["path"],
            "forward_occurrence":{"mapped_resource_slot":item["resource_alias"]},
            "action_effect_occurrence":{"mapped_resource_slot":item["resource_alias"]},
        })
    proof_dimensions = [
        {"dimension":"root-recovery-source-closure","verdict":"proven","rule_id":"AF-SEMANTIC-0154"},
        {"dimension":"root-recovery-action-totals","verdict":"proven","rule_id":"AF-SEMANTIC-0155"},
        {"dimension":"root-recovery-multiplicity","verdict":"proven","rule_id":"AF-SEMANTIC-0155"},
        {"dimension":"root-k-action-epsilon-coverage","verdict":action_verdict,"rule_id":"AF-SEMANTIC-0157"},
        {"dimension":"root-k-recovery-usage-coverage","verdict":usage_verdict,"rule_id":"AF-SEMANTIC-0158"},
        {"dimension":"root-recovery-guard-compatibility","verdict":guard_verdict,"rule_id":"AF-SEMANTIC-0156"},
        {"dimension":"root-recovery-eligibility","verdict":"requires_evidence" if any(item["root_upper"] not in (0,"overflow") for item in actions_expected) else "proven","rule_id":"AF-SEMANTIC-0150"},
    ]
    return {
        "stage":"control_derivation",
        "workflow_closure":{"root_recovery":{
            "actions":{"state":"complete","actions":root_actions},
            "action_effects":{"state":"complete","value":{"row":"closed","entries":action_entries}},
            "epsilon_coverage_requirement":{"state":"complete","value":{"row":"closed","entries":epsilon_entries}},
            "recovery_usage":{"state":"complete","value":usage_json_value(usage_total)},
            "source_closure":source_closure,
            "overall_verdict":overall,
            "proof_dimensions":proof_dimensions,
        }},
    }


def upper_json(value: int | str) -> dict:
    return {"state":"known","upper":value} if value in ("unbounded", "overflow") else {"state":"known","upper":value}


def usage_json_value(value: int | str) -> dict:
    coordinate = {"recovery/api_calls":value}
    state = "known" if isinstance(value, int) or value == "unbounded" else "partial_unknown"
    return {"state":state,"coordinates":coordinate}


def map_entries(values: dict[str, int | str], sources: list[dict], projection: str) -> list[dict]:
    atom_by_key = {}
    guard_by_key = {}
    for source in sources:
        key = atom_key(source["action"])
        atom_by_key[key] = source["action"]
        guard_by_key[key] = source.get("guards", ("unconditional", "unconditional"))[0]
        if projection == "epsilon" and source["action"] == source["forward"]:
            atom_by_key[atom_key(source["forward"])] = source["forward"]
    return [
        {"atom":atom_by_key[key],"guard":guard_by_key.get(key,"unconditional"),"lower":0,"upper":value}
        for key, value in sorted(values.items())
    ]


def action_map_declared_ceiling(tree: dict, sources: list[dict], key: str) -> int | None:
    atom_value = next((source["action"] for source in sources if atom_key(source["action"]) == key), None)
    if atom_value is None:
        return None
    values = []
    for source in sources:
        if source["action"] == atom_value:
            values.append(source.get("epsilon_upper", source["base"]))
    if tree["operator"] == "select":
        return max(values) if values else None
    return sum(values) if values else None


def expected_root_case(case_id: str, tree: dict, sources: list[dict], *, guards: tuple[object, object] = ("unconditional", "unconditional"), input_types: list[dict] | None = None,
                       root_extra: dict | None = None) -> dict:
    for source in sources:
        source["guards"] = guards
        source["input_types"] = input_types or []
    fixture_input = make_control_input(tree, sources, extra=root_extra)
    expected = expected_root(tree, sources)
    return {"case_id":case_id,"stage":"root_recovery","input":fixture_input,"oracle":{"tree":tree,"sources":sources},"expected":expected}


def source_model(node: int, alias: str, *, forward_member: str = "natural", action_member: str = "none", forward_upper: int = 1,
                 forward_lower: int = 0, activation: int = 1, attempts: int = 1, per_attempt: int = 1, epsilon_upper: int | None = None,
                 resource_alias: str | None = None, usage_declaration: int | None = None, failures: list[str] | None = None) -> dict:
    forward, action = atom(forward_member), atom(action_member)
    base = forward_upper * activation * attempts
    if epsilon_upper is None:
        epsilon_upper = base + forward_upper if forward == action else base
    return {
        "node":node,"alias":alias.lower(),"forward":forward,"action":action,"forward_upper":forward_upper,
        "forward_lower":forward_lower,
        "activation":activation,"attempts":attempts,"per_attempt":per_attempt,"base":base,
        "epsilon_upper":epsilon_upper,"resource_alias":resource_alias,
        "usage_declaration":usage_declaration,"failures":failures,
    }


def root_for_leaf(op: str, node: int = 2, **options) -> dict:
    return {"node":1,"operator":op,"children":[leaf_node({"node":node})],**options}


def build_cases() -> list[dict]:
    cases: list[dict] = []
    g = lambda n: f"computation-root-recovery/guard/G{n:02}"
    cases.extend([
        guard_case(g(1), "unconditional", "unconditional"),
        guard_case(g(2), {"input_equals":{"port":1,"value":0}}, "unconditional", [{"port":1,"primitive":"int64"}]),
        guard_case(g(3), "unconditional", {"input_equals":{"port":1,"value":0}}, [{"port":1,"primitive":"int64"}]),
        guard_case(g(4), {"input_equals":{"port":1,"value":0}}, {"input_equals":{"port":1,"value":0}}, [{"port":1,"primitive":"int64"}]),
        guard_case(g(5), {"input_equals":{"port":1,"value":0}}, {"input_equals":{"port":1,"value":1}}, [{"port":1,"primitive":"int64"}]),
        guard_case(g(6), {"input_equals":{"port":1,"value":0}}, {"input_equals":{"port":2,"value":0}}, [{"port":1,"primitive":"int64"},{"port":2,"primitive":"int64"}]),
        guard_case(g(7), "unconditional", "unconditional", effect_row="unknown"),
        guard_case(g(8), "unconditional", {"input_equals":{"port":1,"value":0}}, [{"port":1,"primitive":"int64"}], effect_row="unknown"),
        guard_case(g(12), "unconditional", "unconditional", action_present=False),
        guard_case(g(15), "unconditional", {"input_equals":{"port":1,"value":0}}, [{"port":1,"primitive":"int64"}]),
        guard_case(g(16), "unconditional", {"input_equals":{"port":1,"value":{"unit":None}}}, [{"port":1,"primitive":"unit"}]),
    ])
    audit_forward=atom("natural")
    audit_action=atom("none")
    audit_source=source_model(2,"component-contract:audit-10",forward_upper=1,activation=3,attempts=3,per_attempt=2,epsilon_upper=18)
    audit_guards=("unconditional",{"input_equals":{"port":1,"value":0}})
    audit_types=[{"port":1,"primitive":"int64"}]
    audit_leaf=leaf_production(audit_source,guards=audit_guards,input_types=audit_types)
    cases.append({
        "case_id":"computation-root-recovery/audit/AUDIT-10",
        "stage":"component_validation",
        "input":{"operation":"validate_contract","contract":audit_leaf},
        "oracle":{"shape":"Forward F=1 unconditional; K action uses A=3,T=3,q=2; declared action epsilon guard p1=0","origin":"Phase 4B.2C AUDIT-10"},
        "expected":expected_guard_component({"forward_guard":audit_guards[0],"action_guard":audit_guards[1],"input_types":audit_types,"effect_row":"closed","action_present":True}),
    })
    src = source_model(2,"component-contract:G09",activation=1,attempts=1)
    src_b = source_model(3,"component-contract:G09-B",activation=1,attempts=1)
    cases.append(expected_root_case(g(9),{"node":1,"operator":"select","children":[leaf_node(src),leaf_node(src_b)]},[src,src_b],guards=({"input_equals":{"port":1,"value":0}},"unconditional"),input_types=[{"port":1,"primitive":"int64"}]))
    src10=source_model(2,"component-contract:G10",activation=1,attempts=1)
    src10b=source_model(3,"component-contract:G10-B",activation=1,attempts=1)
    cases.append(expected_root_case(g(10),{"node":1,"operator":"select","children":[leaf_node(src10),leaf_node(src10b)],"exact_child":0},[src10,src10b],guards=({"input_equals":{"port":1,"value":0}},{"input_equals":{"port":1,"value":0}}),input_types=[{"port":1,"primitive":"int64"}]))
    src11=source_model(2,"component-contract:G11",activation=1,attempts=1)
    cases.append(expected_root_case(g(11),root_for_leaf("compensation_scope"),[src11]))
    src13a=source_model(2,"component-contract:G13-A",activation=1,attempts=1)
    src13b=source_model(3,"component-contract:G13-B",activation=1,attempts=1)
    cases.append(expected_root_case(g(13),{"node":1,"operator":"sequence","children":[leaf_node(src13a),leaf_node(src13b)]},[src13a,src13b],guards=({"input_equals":{"port":1,"value":0}},"unconditional"),input_types=[{"port":1,"primitive":"int64"}]))
    zero_guard={"input_equals":{"port":1,"value":0}}
    zero_leaf=leaf(2,"component-contract:g14-zero",forward=atom("natural"),action=atom("none"),forward_upper=1,action_epsilon_upper=1,activation=0,attempts=1,per_attempt=1,guard_forward="unconditional",guard_action=zero_guard,input_types=[{"port":1,"primitive":"int64"}],recovery_usage=0)
    cases.append({
        "case_id":g(14),"stage":"component_validation",
        "input":{"operation":"validate_contract","contract":zero_leaf},
        "oracle":{"forward_guard":"unconditional","action_guard":zero_guard,"input_types":[{"port":1,"primitive":"int64"}],"effect_row":"closed","action_present":True,"action_upper":0,"component_verdict":"violation"},
        "expected":expected_guard_component({"forward_guard":"unconditional","action_guard":zero_guard,"input_types":[{"port":1,"primitive":"int64"}],"effect_row":"closed","action_present":True,"action_upper":0,"component_verdict":"violation"}),
    })
    selected14=source_model(2,"component-contract:G14-A",forward_upper=1,activation=1,attempts=1)
    unreachable14=source_model(3,"component-contract:G14-B",forward_upper=1,activation=1,attempts=1)
    cases.append(expected_root_case("computation-root-recovery/guard/G14-root-path",{"node":1,"operator":"select","children":[leaf_node(selected14),leaf_node(unreachable14)],"exact_child":0},[selected14,unreachable14]))

    k = lambda n: f"computation-root-recovery/root/K{n:02}"
    # K01-K05 are the four required production arithmetic shapes plus the
    # orthogonal inner/outer retry and iterate checks.
    k01=source_model(2,"component-contract:K01",forward_upper=1,activation=2,attempts=3)
    cases.append(expected_root_case(k(1),root_for_leaf("compensation_scope"),[k01]))
    k02=source_model(2,"component-contract:K02",forward_upper=1,activation=2,attempts=2)
    cases.append(expected_root_case(k(2),{"node":1,"operator":"retry","children":[{"node":10,"operator":"compensation_scope","child":leaf_node(k02)}],"delays_micros":[0,0]},[k02]))
    k03=source_model(2,"component-contract:K03",forward_upper=1,activation=2,attempts=2)
    cases.append(expected_root_case(k(3),{"node":1,"operator":"compensation_scope","children":[{"node":10,"operator":"retry","child":leaf_node(k03),"delays_micros":[0,0]}]},[k03]))
    k04=source_model(2,"component-contract:K04",forward_upper=2,activation=1,attempts=2)
    cases.append(expected_root_case(k(4),{"node":1,"operator":"iterate_foreach","children":[{"node":10,"operator":"compensation_scope","child":leaf_node(k04)}],"item_count":4,"max_inflight":2},[k04]))
    k05=source_model(2,"component-contract:K05",forward_upper=2,activation=2,attempts=2)
    cases.append(expected_root_case(k(5),{"node":1,"operator":"iterate_while","children":[{"node":10,"operator":"retry","child":{"node":11,"operator":"compensation_scope","child":leaf_node(k05)},"delays_micros":[0,0]}],"delays_micros":[0,0],"max_iterations":4},[k05]))
    same=source_model(2,"component-contract:K06",forward_member="natural",action_member="natural",forward_upper=1,forward_lower=1,activation=2,attempts=3)
    cases.append(expected_root_case(k(6),root_for_leaf("compensation_scope"),[same]))
    same_bad=source_model(2,"component-contract:k06-underdeclared",forward_member="natural",action_member="natural",forward_upper=6,forward_lower=1,activation=2,attempts=3,epsilon_upper=6)
    same_bad_leaf=leaf_production(same_bad)
    cases.append({
        "case_id":"computation-root-recovery/root/K06-underdeclared",
        "stage":"component_validation",
        "input":{"operation":"validate_contract","contract":same_bad_leaf},
        "oracle":{"reason":"shared epsilon ceiling 6 cannot cover a mandatory same-atom Forward lower 1 plus six K actions"},
        "expected":{"stage":"contract_validation","verdict":"violation","checked_component_produced":False,"action_guard_compatibility":[{"verdict":"violation","basis":"insufficient_evidence"}]},
    })
    a=source_model(2,"component-contract:K07-A",forward_member="natural",action_member="none",forward_upper=4,activation=1,attempts=1,resource_alias="shared-r")
    b=source_model(3,"component-contract:K07-B",forward_member="natural",action_member="none",forward_upper=6,activation=1,attempts=1,resource_alias="shared-r")
    cases.append(expected_root_case(k(7),{"node":1,"operator":"sequence","children":[leaf_node(a),leaf_node(b)]},[a,b]))
    a=source_model(2,"component-contract:K08-A",forward_upper=2,activation=1,attempts=1,resource_alias="shared-r")
    b=source_model(3,"component-contract:K08-B",forward_upper=3,activation=1,attempts=1,resource_alias="shared-r")
    cases.append(expected_root_case(k(8),{"node":1,"operator":"sequence","children":[leaf_node(a),leaf_node(b)]},[a,b]))
    for number, op, amounts in [(9,"select",[4,8]),(10,"parallel_all",[2,5]),(11,"race",[2,5])]:
        branch_sources=[source_model(2,"component-contract:K%02d-A"%number,forward_upper=amounts[0],resource_alias="shared-r"),source_model(3,"component-contract:K%02d-B"%number,forward_upper=amounts[1],resource_alias="shared-r")]
        tree={"node":1,"operator":op,"children":[leaf_node(item) for item in branch_sources]}
        cases.append(expected_root_case(k(number),tree,branch_sources))
    # K12 HANDLE keeps the protected source and possible handler contribution.
    h1=source_model(2,"component-contract:K12-P",forward_upper=3,resource_alias="shared-r",failures=["FailureA"])
    h2=source_model(3,"component-contract:K12-H",forward_upper=4,resource_alias="shared-r")
    handle_tree={"node":1,"operator":"handle","children":[leaf_node(h1),leaf_node(h2)],"clauses":[{"ordinal":1,"match":{"failure":"FailureA"},"handler":3}]}
    cases.append(expected_root_case(k(12),handle_tree,[h1,h2]))
    for number in (13,14):
        nested_source=source_model(2,f"component-contract:K{number:02}",forward_upper=1,activation=2,attempts=2)
        tree={"node":1,"operator":"compensation_scope","children":[{"node":10,"operator":"compensation_scope","child":leaf_node(nested_source)}]}
        cases.append(expected_root_case(k(number),tree,[nested_source]))
    unscoped=source_model(2,"component-contract:K15",activation=2,attempts=2)
    cases.append(expected_root_case(k(15),{"node":1,"operator":"sequence","children":[leaf_node(unscoped)]},[unscoped]))
    k16=source_model(2,"component-contract:K16",forward_upper=3,activation=3,attempts=1,per_attempt=2)
    cases.append(expected_root_case(k(16),root_for_leaf("compensation_scope"),[k16]))
    # K17 and K18 stop at their true component-validation stage.
    invalid_u=source_model(2,"component-contract:K17",forward_upper=3,activation=3,attempts=1,per_attempt=2,usage_declaration=17)
    cases.append({"case_id":k(17),"stage":"component_validation","input":{"operation":"validate_contract","contract":leaf_production(invalid_u)},"oracle":{"reason":"root K usage 18 exceeds component U.recovery 17"},"expected":{"stage":"contract_validation","verdict":"violation","checked_component_produced":False,"action_guard_compatibility":[{"verdict":"proven","basis":"frozen_guard_containment"}]}})
    missing_profile=source_model(2,"component-contract:K18")
    missing_profile_leaf=leaf_production(missing_profile)
    missing_profile_leaf["omit_usage_binding"]=True
    cases.append({"case_id":k(18),"stage":"component_validation","input":{"operation":"validate_contract","contract":missing_profile_leaf},"oracle":{"reason":"matching K RecoveryUsageBinding is absent"},"expected":{"stage":"contract_validation","verdict":"violation","checked_component_produced":False,"action_guard_compatibility":[{"verdict":"unproven","basis":"insufficient_evidence"}]}})
    # K19 is intentionally a pre-root rejection: duplicate computation IDs
    # cannot produce two authenticated factor records in a checked IR tree.
    invalid_factor=source_model(2,"component-contract:K19")
    duplicate_tree={"node":1,"operator":"sequence","children":[{"node":10,"operator":"retry","child":leaf_node(invalid_factor),"delays_micros":[0]},{"node":10,"operator":"retry","child":leaf_node(invalid_factor),"delays_micros":[0,0]}]}
    k19_input=make_control_input(duplicate_tree,[invalid_factor])
    if len(k19_input["nested_nodes"]) > 1:
        k19_input["nested_nodes"][1]["node"] = k19_input["nested_nodes"][0]["node"]
    cases.append({"case_id":k(19),"stage":"ir_rejection","input":k19_input,"oracle":{"reason":"duplicate computation identity rejected before control derivation"},"expected":{"stage":"ir_validation_rejected"}})
    unbounded=source_model(2,"component-contract:K20",forward_upper=1,activation=1,attempts=1)
    cases.append(expected_root_case(k(20),{"node":1,"operator":"iterate_foreach","children":[leaf_node(unbounded)],"item_count":None,"max_inflight":2},[unbounded]))
    u1=source_model(2,"component-contract:K21-A",forward_upper=1,activation=3,attempts=1,per_attempt=1,resource_alias="shared-r")
    u2=source_model(3,"component-contract:K21-B",forward_upper=1,activation=5,attempts=1,per_attempt=1,resource_alias="shared-r")
    cases.append(expected_root_case(k(21),{"node":1,"operator":"select","children":[leaf_node(u1),leaf_node(u2)]},[u1,u2]))
    p1=source_model(2,"component-contract:K22-A",forward_upper=3,resource_alias="shared-r")
    p2=source_model(3,"component-contract:K22-B",forward_upper=4,resource_alias="shared-r")
    cases.append(expected_root_case(k(22),{"node":1,"operator":"parallel_all","children":[leaf_node(p1),leaf_node(p2)]},[p1,p2]))
    q1=source_model(2,"component-contract:K23-A",forward_member="natural",action_member="natural",forward_upper=2,epsilon_upper=4,resource_alias="shared-r")
    q2=source_model(3,"component-contract:K23-B",forward_member="none",action_member="none",forward_upper=3,epsilon_upper=6,resource_alias="shared-r")
    cases.append(expected_root_case(k(23),{"node":1,"operator":"parallel_all","children":[leaf_node(q1),leaf_node(q2)]},[q1,q2]))
    bad_profile=source_model(2,"component-contract:K24")
    bad_profile_leaf=leaf_production(bad_profile)
    bad_profile_leaf["omit_usage_binding"]=True
    cases.append({"case_id":k(24),"stage":"component_validation","input":{"operation":"validate_contract","contract":bad_profile_leaf},"oracle":{"reason":"complete source lacks the exact matching usage profile"},"expected":{"stage":"contract_validation","verdict":"violation","checked_component_produced":False,"action_guard_compatibility":[{"verdict":"unproven","basis":"insufficient_evidence"}]}})
    overflow=source_model(2,"component-contract:K25",forward_upper=U64_MAX,activation=1,attempts=1,per_attempt=2,usage_declaration=U64_MAX)
    cases.append({"case_id":"computation-root-recovery/root/K25-overflow","stage":"component_validation","input":{"operation":"validate_contract","contract":leaf_production(overflow)},"oracle":{"reason":"checked recovery meter multiplication U64_MAX times two must fail closed"},"expected":{"stage":"contract_validation","verdict":"violation","checked_component_produced":False,"action_guard_compatibility":[{"verdict":"proven","basis":"frozen_guard_containment"}]}})
    return cases


def main() -> None:
    cases = build_cases()
    OUT.mkdir(parents=True, exist_ok=True)
    for group in ("guard", "root", "audit"):
        (OUT / group).mkdir(exist_ok=True)
    for case in cases:
        group = case["case_id"].split("/")[-2]
        target = OUT / group / f"{case['case_id'].split('/')[-1]}.json"
        target.write_text(json.dumps(case, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    manifest = {
        "case_count": len(cases),
        "case_ids": sorted(case["case_id"] for case in cases),
        "cases": [
            {
                "case_id":case["case_id"],
                "expected_stage":case["stage"],
                "production_api":{
                    "component_validation":"validate_contract_case",
                    "root_recovery":"analyze_compose_case",
                    "ir_rejection":"analyze_compose_case",
                }[case["stage"]],
                "oracle":"independent-python",
                "expected_fields":sorted(case["expected"].keys()),
                "stage_reachability":{
                    "ComponentValidation":"executed",
                    "IRValidation":"executed" if case["stage"] in ("root_recovery","ir_rejection") else "not_reached_by_case",
                    "LeafResolution":"executed" if case["stage"] == "root_recovery" else "not_reached_by_case",
                    "ControlDerivation":"executed" if case["stage"] == "root_recovery" else "not_reached_by_case",
                    "RootRecoveryValidation":"executed" if case["stage"] == "root_recovery" else "not_reached_by_case",
                    "LowerLevelEffectAlgebra":"reused_by_root_fold" if case["stage"] == "root_recovery" else "not_reached_by_case",
                },
                "d3_export_assertion":"deferred" if case["case_id"].endswith("/K20") else "not_in_case",
            }
            for case in sorted(cases,key=lambda item:item["case_id"])
        ],
        "d3_export_assertions_deferred": ["K20 export epsilon ceiling; only internal root K/E/U facts execute in this phase"],
        "revision": "phase-04b2e1-v1",
    }
    (OUT / "manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
