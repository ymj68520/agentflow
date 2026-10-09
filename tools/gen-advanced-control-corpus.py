#!/usr/bin/env python3
"""Independent Phase 4B.2B.2 oracle and corpus materializer.

This oracle models only static contract algebra. It imports no Rust output,
does not execute a workflow, and writes ordinary conformance vectors rather
than an ABI, digest or persistence format.
"""

from __future__ import annotations

import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CORPUS = ROOT / "spec/conformance/computation-advanced-control"
U64_MAX = (1 << 64) - 1


def effect_entry(lower: int, upper: int | str) -> dict:
    return {
        "atom": {
            "operation": {"catalog": "agentflow.effects@1", "name": "write"},
            "qualifiers": [{"family": "idempotency", "member": "natural"}],
            "resource_selector": {"requirement_port_no": 1, "scope_bindings": []},
        },
        "guard": "unconditional",
        "lower": lower,
        "upper": upper,
    }


def child(node: int, *, effect: tuple[int, int | str] | None, failures: list[str], usage: int, execution: str | None = None, replay_semantics: str = "recompute", outputs: list[dict] | None = None) -> dict:
    entries = [] if effect is None else [effect_entry(*effect)]
    return {
        "caller_digest": "semantic-ir:caller",
        "contract_digest": f"component-contract:{node + 100:064x}",
        "effect_entries": entries,
        "effects": {"entries": entries, "residuals": [], "row": "closed"},
        "execution": None if execution is None else {"sandbox": execution},
        "failures": failures,
        "flow": {"authority": "none", "constraints": [], "influence_unknown": False, "sources": [], "taints": []},
        "node": node,
        "outputs": outputs or [],
        "proof_source": "checked_component",
        "recovery_actions": None,
        "replay_semantics": replay_semantics,
        "resolved": True,
        "resources": None,
        "usage": {"coordinates": {"forward/api_calls": usage, "recovery/api_calls": usage // 2}},
    }


def failure_type(alias: str) -> str:
    return "TypeSchemaRef" if alias == "FailureA" else "DigestRefValue"


def make_input(operator: str, children: list[dict], **options) -> dict:
    result = {"children": children, "operation": "compose", "operator": operator}
    result.update(options)
    return result


def generated_cases() -> list[dict]:
    cases: list[dict] = []
    foreach_counts: list[int | None] = [0, 1, 2, 3, 4, None, 1, 2, 5, None, 0, 3]
    while_budgets = [1, 2, 3, 4, 5, 1, 2, 3, 4, 5, 2, 4]
    retry_delays = [[], [0], [5], [3, 1], [0, 0, 0], [7, 2], [1], [], [9, 0, 4], [2, 2], [0, 6, 1, 0], [11]]
    timeout_values = [0, 1, 2, 100, 7, 25, 3, 9, 50, 4, 12, 1]

    for index in range(12):
        effect = None if index % 4 == 0 else (index % 2, 1 + index % 5)
        failures = [[], ["FailureA"], ["FailureA", "FailureB"]][index % 3]
        usage = (index * 3) % 11
        count = foreach_counts[index]
        child_node = child(2, effect=effect, failures=failures, usage=usage)
        case_id = f"computation-advanced-control/iterate/foreach-{index + 1:02}"
        cases.append(vector(case_id, "iterate", make_input(
            "iterate_foreach", [child_node], item_count=count,
            max_inflight=[None, 1, 2, 3][index % 4],
        )))

    for index, budget in enumerate(while_budgets):
        effect = None if index % 3 == 0 else (index % 2, 2 + index % 4)
        failures = [[], ["FailureB"], ["FailureA"]][index % 3]
        child_node = child(2, effect=effect, failures=failures, usage=(index * 5) % 13)
        cases.append(vector(
            f"computation-advanced-control/iterate/while-{index + 1:02}",
            "iterate",
            make_input("iterate_while", [child_node], guard_value=index % 2 == 0, max_iterations=budget),
        ))

    for index, delays in enumerate(retry_delays):
        effect = None if index % 4 == 0 else (1 if index % 2 else 0, 1 + index % 6)
        failures = [[], ["FailureA"], ["FailureB"]][index % 3]
        child_node = child(2, effect=effect, failures=failures, usage=(index * 7) % 17)
        cases.append(vector(
            f"computation-advanced-control/retry/schedule-{index + 1:02}",
            "retry",
            make_input("retry", [child_node], delays_micros=delays),
        ))

    for index, timeout in enumerate(timeout_values):
        effect = None if index % 4 == 0 else (index % 2, 1 + index % 5)
        failures = [[], ["FailureA"], ["FailureA", "FailureB"]][index % 3]
        child_node = child(2, effect=effect, failures=failures, usage=(index * 2) % 13)
        cases.append(vector(
            f"computation-advanced-control/timeout/boundary-{index + 1:02}",
            "timeout",
            make_input("timeout", [child_node], timeout_micros=timeout),
        ))

    for index in range(12):
        protected_failures = [["FailureA"], ["FailureB"], ["FailureA", "FailureB"]][index % 3]
        matcher_sets = [["FailureA"], ["FailureB"], ["FailureA", "timed_out"]][index % 3]
        protected = child(2, effect=None if index % 3 == 0 else (index % 2, 2 + index % 4), failures=protected_failures, usage=(index * 3) % 13)
        handler_nodes = [3 + sub for sub in range(len(matcher_sets))]
        handlers = [
            child(node, effect=None if (index + sub) % 3 == 0 else (0, 1 + (index + sub) % 5),
                  failures=["FailureB"] if sub % 2 == 0 else ["FailureA"],
                  usage=(index + 2 * sub) % 9)
            for sub, node in enumerate(handler_nodes)
        ]
        clauses = []
        for sub, (matcher, handler_node) in enumerate(zip(matcher_sets, handler_nodes, strict=True), start=1):
            clause_match = "timed_out" if matcher == "timed_out" else {"failure": matcher}
            clauses.append({"handler": handler_node, "match": clause_match, "ordinal": sub})
        cases.append(vector(
            f"computation-advanced-control/handle/typed-clause-{index + 1:02}",
            "handle",
            make_input("handle", [protected, *handlers], clauses=clauses),
        ))

    output = lambda slot: [{"port": 1, "slot": f"value-slot:{slot}", "type_id": "semantic-type:1"}]
    parent_output = [{"port": 1, "slot": "value-slot:1000", "type_id": "semantic-type:1"}]
    carry = [{"body_output_port": 1, "initial_expression": 30001, "initial_slot": 40011, "slot": 40010, "type_id": "semantic-type:1"}]
    cases.append(vector(
        "computation-advanced-control/iterate/foreach-carry-interface",
        "iterate",
        make_input("iterate_foreach", [child(2, effect=(0, 2), failures=[], usage=3, outputs=output(30))],
                   item_count=2, max_inflight=1, carry=carry, parent_outputs=parent_output),
    ))
    cases.append(vector(
        "computation-advanced-control/iterate/while-carry-interface",
        "iterate",
        make_input("iterate_while", [child(2, effect=None, failures=[], usage=2, outputs=output(30))],
                   guard_value=True, max_iterations=4, carry=carry, parent_outputs=parent_output),
    ))
    cases.append(vector(
        "computation-advanced-control/retry/child-output-interface",
        "retry",
        make_input("retry", [child(2, effect=(0, 2), failures=[], usage=2, outputs=output(30))],
                   delays_micros=[3], parent_outputs=parent_output),
    ))
    cases.append(vector(
        "computation-advanced-control/timeout/child-output-interface",
        "timeout",
        make_input("timeout", [child(2, effect=None, failures=[], usage=2, outputs=output(30))],
                   timeout_micros=10, parent_outputs=parent_output),
    ))
    cases.append(vector(
        "computation-advanced-control/handle/all-handler-output-interfaces",
        "handle",
        make_input("handle", [
            child(2, effect=None, failures=["FailureA"], usage=2, outputs=output(30)),
            child(3, effect=None, failures=[], usage=1, outputs=output(40)),
        ], clauses=[{"handler": 3, "match": {"failure": "FailureA"}, "ordinal": 1}], parent_outputs=parent_output),
    ))
    nested_leaf = child(3, effect=(1, 2), failures=["FailureA"], usage=3)
    cases.append(vector(
        "computation-advanced-control/retry/nested-foreach-amplification",
        "retry",
        make_input(
            "retry", [nested_leaf], delays_micros=[0], root_child_nodes=[2],
            nested_nodes=[{"body": 3, "item_count": 2, "item_slot": 40000,
                           "max_inflight": 1, "node": 2,
                           "operator": "iterate_foreach", "source_expression": 20000}],
        ),
    ))

    # Adversarial production paths: exact-match violation and checked overflow.
    protected = child(2, effect=(0, 2), failures=["FailureA"], usage=2)
    handler = child(3, effect=(0, 1), failures=["FailureB"], usage=1)
    cases.append(vector(
        "computation-advanced-control/negative/handle-unmatched-type",
        "negative",
        make_input("handle", [protected, handler], clauses=[{"handler": 3, "match": {"failure": "FailureB"}, "ordinal": 1}]),
    ))

    max_effect = child(2, effect=(1, U64_MAX), failures=["FailureA"], usage=1)
    cases.append(vector(
        "computation-advanced-control/negative/retry-effect-overflow",
        "negative",
        make_input("retry", [max_effect], delays_micros=[0]),
    ))
    max_usage = child(2, effect=None, failures=[], usage=U64_MAX)
    cases.append(vector(
        "computation-advanced-control/negative/retry-usage-overflow",
        "negative",
        make_input("retry", [max_usage], delays_micros=[1]),
    ))
    forbidden_replay = child(2, effect=(1, 2), failures=["FailureA"], usage=2, replay_semantics="forbidden")
    cases.append(vector(
        "computation-advanced-control/negative/retry-forbidden-replay",
        "negative",
        make_input("retry", [forbidden_replay], delays_micros=[0]),
    ))
    dynamic_timeout = child(2, effect=None, failures=["FailureB"], usage=3)
    cases.append(vector(
        "computation-advanced-control/timeout/dynamic-duration",
        "timeout",
        make_input("timeout", [dynamic_timeout], timeout_dynamic=True),
    ))
    unsupported_scope = child(2, effect=None, failures=[], usage=1)
    cases.append(vector(
        "computation-advanced-control/negative/scope-unsupported",
        "negative",
        make_input("scope", [unsupported_scope]),
    ))
    return cases


def vector(case_id: str, group: str, input_value: dict) -> dict:
    return {
        "case_id": case_id,
        "expected": oracle_projection(input_value),
        "group": group,
        "input": input_value,
        "kind": "agentflow.conformance.computation-advanced-control.test-vector",
        "note": "Independent static algebra oracle; this vector is executed through checked Semantic IR, Phase 3C, leaf resolution, and the production control analyzer.",
    }


def finite_product(value: int | str, count: int | None) -> int | str:
    if count is None:
        return 0 if value == 0 else "unbounded"
    if value == "unbounded":
        return "unbounded" if count else 0
    return value * count


def oracle_projection(input_value: dict) -> dict:
    operator = input_value["operator"]
    children = input_value["children"]
    nodes = [item["node"] for item in children]
    root_children = input_value.get("root_child_nodes", nodes)
    if operator == "scope":
        return {
            "identity": {"operator": "unsupported", "node": 1, "children": nodes},
            "control_semantics": {"kind": "unsupported"},
            "effects": {"row": "unknown"},
            "effect_row_complete": False,
            "control_decision_dependencies": [],
            "control_decision_influence_unknown": False,
            "dimension_availability": {
                "failure": "unknown", "effect": "unknown", "resource": "per_child",
                "information_flow": "partial", "usage": "unknown", "recovery": "per_child",
                "execution": "per_child", "typed_outputs": "unknown",
            },
            "typed_outputs": {"state": "unknown", "unresolved_sources": [1]},
            "output_interface": {"state": "unknown", "unresolved_sources": [1]},
            "verdict": "unproven",
        }
    effect_occurrences = []
    family_projection = []
    for item in children:
        for entry_ordinal, declared in enumerate(item["effect_entries"]):
            effect_occurrences.append({
                "atom": declared["atom"],
                "caller_digest": "semantic-ir:caller",
                "contract_digest": item["contract_digest"],
                "entry_ordinal": entry_ordinal,
                "node": item["node"],
            })
            if operator in ("iterate_foreach", "iterate_while", "retry"):
                if operator == "iterate_foreach":
                    upper = input_value["item_count"]
                    repetitions = {"lower": 0, "upper": "unbounded" if upper is None else upper}
                    physical = None
                    kind = "iterate"
                elif operator == "iterate_while":
                    upper = input_value["max_iterations"]
                    repetitions = {"lower": 0, "upper": upper}
                    physical = None
                    kind = "iterate"
                else:
                    upper = len(input_value["delays_micros"]) + 1
                    repetitions = {"lower": 1, "upper": upper}
                    physical = {"lower": 1, "upper": upper}
                    kind = "retry"
                families = []
                for nested in input_value.get("nested_nodes", []):
                    if nested["operator"] == "iterate_foreach":
                        families.append({
                            "control_node": nested["node"],
                            "kind": "iterate",
                            "logical_repetitions": {"lower": 0, "upper": nested["item_count"]},
                            "physical_attempts": None,
                        })
                families.append({
                    "control_node": 1,
                    "kind": kind,
                    "logical_repetitions": repetitions,
                    "physical_attempts": physical,
                })
                family_projection.append({
                    "occurrence_index": len(effect_occurrences) - 1,
                    "source_node": item["node"],
                    "families": families,
                })

    effect_bounds = [entry for item in children for entry in item["effect_entries"]]
    effect = {"entries": [], "residuals": [], "row": "closed"}
    effect_overflow = False
    if effect_bounds:
        if operator == "iterate_foreach":
            count = input_value["item_count"]
            lower, upper = 0, finite_product(effect_bounds[0]["upper"], count)
        elif operator == "iterate_while":
            lower, upper = 0, finite_product(effect_bounds[0]["upper"], input_value["max_iterations"])
        elif operator == "retry":
            attempts = len(input_value["delays_micros"]) + 1
            lower = effect_bounds[0]["lower"]
            upper = effect_bounds[0]["upper"]
            for nested in input_value.get("nested_nodes", []):
                if nested["operator"] == "iterate_foreach":
                    lower = 0
                    upper = finite_product(upper, nested["item_count"])
            upper = finite_product(upper, attempts)
        elif operator == "timeout":
            lower, upper = effect_bounds[0]["lower"], effect_bounds[0]["upper"]
        else:
            protected_count = len(children[0]["effect_entries"])
            protected_entry = children[0]["effect_entries"][0] if protected_count else None
            handler_entries = [entry for item in children[1:] for entry in item["effect_entries"]]
            if protected_entry is None and not handler_entries:
                lower, upper = 0, 0
                atom = None
            else:
                atom = (protected_entry or handler_entries[0])["atom"]
                lower = 0 if protected_entry is None else protected_entry["lower"]
                protected_upper = 0 if protected_entry is None else protected_entry["upper"]
                handler_upper = max((entry["upper"] for entry in handler_entries), default=0)
                if "unbounded" in (protected_upper, handler_upper):
                    upper = "unbounded"
                else:
                    upper = protected_upper + handler_upper
            if upper != "unbounded" and upper > U64_MAX:
                effect_overflow = True
        if operator != "handle":
            atom = effect_bounds[0]["atom"]
        if upper != "unbounded" and upper > U64_MAX:
            effect_overflow = True
        if not effect_overflow and atom is not None and upper != 0:
            effect["entries"] = [{"atom": atom, "guard": "unconditional", "lower": lower, "upper": upper}]

    def usage_coordinates(item: dict) -> dict[str, int]:
        return item["usage"]["coordinates"]

    usage_overflow = False
    usage_result: dict[str, int | str] = {}
    all_coordinates = sorted({key for item in children for key in usage_coordinates(item)})
    for coordinate in all_coordinates:
        values = [usage_coordinates(item)[coordinate] for item in children]
        if operator == "iterate_foreach":
            count = input_value["item_count"]
            value = "unknown" if count is None else values[0] * count
        elif operator == "iterate_while":
            value = values[0] * input_value["max_iterations"]
        elif operator == "retry":
            value = values[0] * (len(input_value["delays_micros"]) + 1)
            for nested in input_value.get("nested_nodes", []):
                if nested["operator"] == "iterate_foreach":
                    value *= nested["item_count"]
        elif operator == "timeout":
            value = values[0]
        else:
            value = values[0] + max(values[1:], default=0)
        if isinstance(value, int) and value > U64_MAX:
            usage_overflow = True
        usage_result[coordinate] = value
    if usage_overflow:
        usage = None
    elif "unknown" in usage_result.values():
        usage = {"state": "partial_unknown", "coordinates": usage_result}
    else:
        usage = {"state": "known", "coordinates": usage_result}

    handle_mismatch = False
    if operator == "handle":
        handled = {
            clause["match"]["failure"]
            for clause in input_value["clauses"]
            if isinstance(clause["match"], dict)
        }
        handle_mismatch = not handled.issubset(set(children[0]["failures"]))
        failures = None if handle_mismatch else sorted(
            (set(children[0]["failures"]) - handled)
            | {failure for item in children[1:] for failure in item["failures"]}
        )
        clauses = [
            {
                "ordinal": clause["ordinal"],
                "matcher": {"timed_out": None} if clause["match"] == "timed_out" else {
                    "failure_type": failure_type(clause["match"]["failure"])
                },
                "handler": clause["handler"],
            }
            for clause in input_value["clauses"]
        ]
        control = {
            "kind": "handle",
            "handled_failure_types": [
                clause["match"]["failure"]
                for clause in input_value["clauses"]
                if isinstance(clause["match"], dict)
            ],
            "clauses": clauses,
            "handler_paths_are_conditional": True,
            "unmatched_failures_remain_in_residual": True,
        }
    else:
        failures = children[0]["failures"]
        handled = set()
        if operator == "iterate_foreach":
            count = input_value["item_count"]
            max_count = "unbounded" if count is None else count
            repetitions = {"lower": 0, "upper": max_count}
            mode = {
                "kind": "foreach",
                "statically_known_item_count": count,
                "max_inflight": input_value["max_inflight"],
            }
            usage_repetitions = count
            control = {
                "kind": "iterate", "mode": mode,
                "effect_repetitions": repetitions,
                "usage_repetitions": usage_repetitions,
                "max_inflight_is_not_total_count": True,
            }
        elif operator == "iterate_while":
            count = input_value["max_iterations"]
            repetitions = {"lower": 0, "upper": count}
            mode = {"kind": "while", "max_iterations": count}
            control = {
                "kind": "iterate", "mode": mode,
                "effect_repetitions": repetitions,
                "usage_repetitions": count,
                "max_inflight_is_not_total_count": True,
            }
        elif operator == "retry":
            delays = input_value["delays_micros"]
            attempts = len(delays) + 1
            control = {
                "kind": "retry",
                "delays_micros_in_semantic_order": delays,
                "maximum_attempts": attempts,
                "physical_attempt_bounds": {"lower": 1, "upper": attempts},
                "logical_narrowing_proven": False,
                "retryable_failure_policy": "not_present_in_frozen_ir",
            }
        else:
            dynamic = input_value.get("timeout_dynamic", False)
            control = {
                "kind": "timeout",
                "timeout_expression": 1,
                "static_duration_micros": None if dynamic else input_value["timeout_micros"],
                "possible_outcomes": ["child_completed", "timed_out"],
                "timed_out_is_typed_failure": False,
                "successful_cancellation_inferred": False,
            }

    # Known usage dimensions use the independent additive / max-path formulas.
    if usage is not None:
        for coordinate in all_coordinates:
            if usage["coordinates"][coordinate] == "unknown":
                continue
            values = [usage_coordinates(item)[coordinate] for item in children]
            if operator == "iterate_foreach":
                count = input_value["item_count"]
                usage["coordinates"][coordinate] = "unknown" if count is None else values[0] * count
            elif operator == "iterate_while":
                usage["coordinates"][coordinate] = values[0] * input_value["max_iterations"]
            elif operator == "retry":
                usage["coordinates"][coordinate] = values[0] * (len(input_value["delays_micros"]) + 1)
                for nested in input_value.get("nested_nodes", []):
                    if nested["operator"] == "iterate_foreach":
                        usage["coordinates"][coordinate] *= nested["item_count"]
            elif operator == "timeout":
                usage["coordinates"][coordinate] = values[0]
            else:
                usage["coordinates"][coordinate] = values[0] + max(values[1:], default=0)

    control_dependencies = []
    if operator == "iterate_foreach" and input_value["item_count"] is None:
        control_dependencies = [200]
    if operator == "timeout" and input_value.get("timeout_dynamic", False):
        control_dependencies = [201]
    decision_unknown = True
    effect_status = "unknown" if effect_overflow else "complete"
    usage_status = "unknown" if usage_overflow else "complete"
    failure_status = "unknown" if handle_mismatch else "complete"
    forbidden_replay = operator == "retry" and any(
        item.get("replay_semantics") == "forbidden" for item in children
    ) and bool(effect_bounds)
    if handle_mismatch or effect_overflow or usage_overflow or forbidden_replay:
        verdict = "violation"
    elif operator == "retry" and bool(effect_bounds):
        verdict = "requires_evidence"
    else:
        verdict = "unproven"

    expected = {
        "identity": {"operator": operator_name(operator), "node": 1, "children": root_children},
        "control_semantics": control,
        "effects": effect,
        "effect_occurrences": effect_occurrences,
        "effect_row_complete": not effect_overflow,
        "effect_lower_bound_adjusted_children": [],
        "failures": failures,
        "usage": usage,
        "control_effect_families": family_projection,
        "control_decision_dependencies": control_dependencies,
        "control_decision_influence_unknown": decision_unknown,
        "dimension_availability": {
            "failure": failure_status,
            "effect": effect_status,
            "resource": "per_child",
            "information_flow": "partial",
            "usage": usage_status,
            "recovery": "per_child",
            "execution": "per_child",
            "typed_outputs": "complete",
        },
        "typed_outputs": {"state": "complete", "port_count": len(input_value.get("parent_outputs", []))},
        "output_interface": {
            "state": "complete",
            "ports": [
                {
                    "parent_port": output["port"],
                    "parent_slot": int(output["slot"].rsplit(":", 1)[1]),
                    "local_type_id": int(output["type_id"].split(":", 1)[1]),
                    "sources": [
                        {"child": child["node"], "port": output["port"], "value_slot": int(child["outputs"][0]["slot"].rsplit(":", 1)[1])}
                        for child in children
                        if any(child_output["port"] == output["port"] for child_output in child["outputs"])
                    ],
                }
                for output in input_value.get("parent_outputs", [])
            ],
        },
        "execution_by_child": [],
        "recovery_sources": [],
        "unresolved_children": [],
        "verdict": verdict,
    }
    if not effect_overflow:
        expected["resources_by_child"] = [
            {"node": item["node"], "requirements": [{
                "component_port": 1,
                "resource_slot": f"fixture:auto:{item['node']}:1",
            }]}
            for item in children if item["effect_entries"]
        ]
    else:
        expected.pop("effects", None)
    if usage_overflow:
        expected.pop("usage", None)
    if operator in ("iterate_foreach", "iterate_while", "retry", "timeout") and not effect_overflow:
        expected["flow"] = {
            "sources": [], "taints": [], "may_carry_authority": "none",
            "effective_authority_inferred": False, "influence_unknown": True,
            "decision_dependencies": [],
            "constraints_by_child": [{"node": node, "constraints": []} for node in nodes],
            "verdict": "unproven",
        }
    if operator in ("iterate_foreach", "iterate_while", "retry", "timeout") and effect is not None:
        expected["flow"] = {
            "sources": [], "taints": [], "may_carry_authority": "none",
            "effective_authority_inferred": False, "influence_unknown": True,
            "decision_dependencies": [],
            "constraints_by_child": [{"node": node, "constraints": []} for node in nodes],
            "verdict": "unproven",
        }
    return expected


def operator_name(operator: str) -> str:
    return "iterate" if operator.startswith("iterate_") else operator


def write_json(path: Path, value: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")


def main() -> None:
    cases = generated_cases()
    case_ids = sorted(case["case_id"] for case in cases)
    for case in cases:
        group = case["group"]
        case_id = case["case_id"].split("/")[-1]
        write_json(CORPUS / group / f"{case_id}.json", case)
    write_json(CORPUS / "manifest.json", {
        "case_count": len(cases),
        "case_ids": case_ids,
        "corpus_revision": 1,
        "format_note": "independent static vectors only; no ABI, control digest, runtime identity, timer, cancellation, retry policy, grant or execution evidence",
        "kind": "agentflow.conformance.computation-advanced-control.test-corpus",
    })
    print(f"wrote {len(cases)} independent advanced-control vectors")


if __name__ == "__main__":
    main()
