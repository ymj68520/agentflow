#!/usr/bin/env python3
"""Independent Portable D9-D10 fixtures and expected static facts.

The oracle consumes only declared Bundle assets and the frozen IR/control
algebra. It does not execute Rust, inspect Rust projections, or call a checker.
"""

from __future__ import annotations

import copy
from legacy_edition1_inputs import assert_historical_output
import importlib.util
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "spec" / "conformance" / "portable-root-export-revalidation"
BASE_PATH = "spec/conformance/portable-proof-codec/full-ten-field-bundle.ace1"
POSITIVE_PATH = "spec/conformance/portable-root-export-revalidation/full-static-positive.ace1"
D9_DIR = "spec/conformance/portable-root-export-revalidation/d9"
COMPARATOR_MUTATIONS = [
    {"mutation_id":"root-opcode","path":["root_operator"],"value":"sequence"},
    {"mutation_id":"root-effect-count","path":["root_effect_entries"],"value":1},
    {"mutation_id":"root-recovery-count","path":["root_recovery_actions"],"value":1},
    {"mutation_id":"usage-forward","path":["usage_coordinates","forward/api_calls"],"value":1},
    {"mutation_id":"execution-coordinate-count","path":["execution_source_mapping_count"],"value":0},
    *[{"mutation_id":f"dimension-{dimension}","path":["dimension_verdicts",dimension],"value":"unproven"} for dimension in ["I","O","Phi","epsilon","R","F","U","K","X"]],
    {"mutation_id":"static-verdict","path":["static_compatibility"],"value":"unproven"},
    {"mutation_id":"capability-issuance","path":["capability_issued"],"value":False},
    {"mutation_id":"body-boundary","path":["body_conformance"],"value":"proven"},
    {"mutation_id":"implementation-boundary","path":["implementation_conformance"],"value":"proven"},
    {"mutation_id":"publisher-boundary","path":["publisher_authentication"],"value":"authenticated"},
    {"mutation_id":"runtime-boundary","path":["runtime_authority"],"value":"proven"},
]

sys.path.insert(0, str(ROOT / "tools"))
from portable_proof_python_oracle import b64url, canonical_bytes, hash_v1  # noqa: E402

_spec = importlib.util.spec_from_file_location(
    "portable_static_reconstruction_generator",
    ROOT / "tools" / "gen-portable-static-reconstruction-corpus.py",
)
assert _spec and _spec.loader
_static = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_static)


def full_static_positive() -> dict:
    bundle = json.loads((ROOT / BASE_PATH).read_text(encoding="utf-8"))
    context = bundle["proof_contexts"][0]["content"]
    meter = {"catalog": "agentflow.meters.test@1", "name": "api_calls"}
    context["usage_meters"]["complete"] = [{
        "meter": copy.deepcopy(meter),
        "unit": {"catalog": "agentflow.units.test@1", "name": "call"},
        "aggregation": "additive",
    }]

    old_contract = json.loads(
        _static.b64decode(bundle["contract_assets"][0]["canonical_content"]).decode("utf-8")
    )
    old_contract["usage"]["limits"] = [
        {"meter": copy.deepcopy(meter), "phase": "forward", "upper": {"finite": "0"}},
        {"meter": copy.deepcopy(meter), "phase": "recovery", "upper": {"finite": "0"}},
    ]
    bundle, contract_ref = _static.install_contract(bundle, old_contract)

    assertion = bundle["root_export_assertions"][0]["assertion"]
    resolver = bundle["resolver_dependency_manifests"][0]["content"]
    caller_ref = resolver["caller_ir"]
    caller = json.loads(
        _static.b64decode(_static.find_ir(bundle, caller_ref["digest"]["hex"])["canonical_content"])
        .decode("utf-8")
    )
    invoke = next(item for item in caller["computations"] if "invoke" in item["body"])
    assertion["execution_mappings"] = [{
        "source_node": invoke["id"],
        "source_contract_digest": copy.deepcopy(contract_ref["digest"]),
        "exact_execution_requirements": copy.deepcopy(old_contract["execution"]),
    }]
    assertion["execution_mappings"].sort(
        key=lambda row: (int(row["source_node"]), row["source_contract_digest"]["hex"])
    )
    return bundle


def independent_positive_expected(bundle: dict, *, missing_execution: bool = False) -> dict:
    """Small semantic oracle for the intentionally empty-root positive case."""
    root = bundle["root_export_assertions"][0]["assertion"]
    assert root["input_mappings"] == root["output_mappings"] == []
    assert root["resource_mappings"] == root["effect_mappings"] == []
    assert root["flow_mappings"] == root["recovery_mappings"] == []
    assert root["subflow_dependencies"] == []
    assert len(root["execution_mappings"]) == (0 if missing_execution else 1)
    assert [item["phase"] for item in json.loads(
        _static.b64decode(bundle["contract_assets"][0]["canonical_content"]).decode("utf-8")
    )["usage"]["limits"]] == ["forward", "recovery"]
    # The only leaf and the top export use one exact checked contract. Its U
    # coordinates are explicit finite zeroes; X maps the exact source row.
    verdicts = {name: "proven" for name in ["I", "O", "Phi", "epsilon", "R", "F", "U", "K", "X"]}
    if missing_execution:
        verdicts["X"] = "unproven"
    return {
        "root_operator": "invoke",
        "root_effect_entries": 0,
        "root_recovery_actions": 0,
        "usage_coordinates": {"forward/api_calls": 0, "recovery/api_calls": 0},
        "execution_source_mapping_count": 1,
        "dimension_verdicts": verdicts,
        "static_compatibility": "unproven" if missing_execution else "proven",
        "capability_issued": not missing_execution,
        "body_conformance": "requires_evidence",
        "implementation_conformance": "requires_evidence",
        "publisher_authentication": "not_established",
        "runtime_authority": "not_established",
    }


def _invoke(node: int) -> dict:
    return {
        "id": str(node),
        "outputs": [],
        "body": {"invoke": {"contract": "1", "inputs": [], "resources": []}},
    }


def d9_opcode_cases() -> list[dict]:
    """Materialize one exact D0-D8 bundle for each registered IR opcode."""
    base = full_static_positive()
    first = _static.component_ref(base)
    signal_type = _static.schema_reference(base, "agentflow.semantic.ir@1", "SemanticIrDocument")
    cases: list[tuple[str, dict, list[dict], dict]] = []

    cases.append(("invoke", {"invoke": {"contract": "1", "inputs": [], "resources": []}}, [], {}))
    cases.append(("await", {"await": {"signal_type": signal_type, "correlation": "1"}}, [], {
        "types": [{"id": "1", "body": {"primitive": "int64"}}],
        "expressions": [{"id": "1", "result_type": "1", "body": {"constant": {"int64": "1"}}}],
    }))

    two_invokes = [_invoke(2), _invoke(3)]
    cases.extend([
        ("sequence", {"sequence": {"children": ["2", "3"]}}, two_invokes, {}),
        ("select", {"select": {"selector": "1", "cases": [
            {"ordinal": "1", "child": "2"}, {"ordinal": "2", "child": "3"},
        ]}}, two_invokes, {
            "types": [{"id": "1", "body": {"primitive": "int64"}}],
            "expressions": [{"id": "1", "result_type": "1", "body": {"constant": {"int64": "1"}}}],
        }),
        ("parallel", {"parallel": {"mode": "all", "branches": [
            {"ordinal": "1", "child": "2"}, {"ordinal": "2", "child": "3"},
        ], "max_inflight": None, "output_bindings": []}}, two_invokes, {}),
        ("race", {"race": {"branches": [
            {"ordinal": "1", "child": "2"}, {"ordinal": "2", "child": "3"},
        ]}}, two_invokes, {}),
        ("handle", {"handle": {"child": "2", "clauses": [
            {"ordinal": "1", "match": {"timed_out": None}, "handler": "3"},
        ]}}, two_invokes, {}),
    ])
    cases.extend([
        ("iterate", {"iterate": {"while": {"guard": "1", "body": "2", "carry": [], "max_iterations": "3"}}}, [_invoke(2)], {
            "types": [{"id": "1", "body": {"primitive": "bool"}}],
            "expressions": [{"id": "1", "result_type": "1", "body": {"constant": {"bool": True}}}],
        }),
        ("retry", {"retry": {"child": "2", "delays": ["0"]}}, [_invoke(2)], {}),
        ("timeout", {"timeout": {"child": "2", "timeout": "1"}}, [_invoke(2)], {
            "types": [{"id": "1", "body": {"primitive": "duration"}}],
            "expressions": [{"id": "1", "result_type": "1", "body": {"constant": {"duration": "100"}}}],
        }),
        ("scope", {"scope": {"child": "2", "resources": []}}, [_invoke(2)], {}),
        ("compensation_scope", {"compensation_scope": {"child": "2"}}, [_invoke(2)], {}),
    ])

    output: list[dict] = []
    for opcode, root_body, children, extra in cases:
        if opcode == "invoke":
            computations = [{"id": "1", "outputs": [], "body": root_body}]
            associations = [{"source_node": "1", "target": {"invoke": copy.deepcopy(first)}}]
        elif opcode == "await":
            computations = [{"id": "1", "outputs": [], "body": root_body}]
            associations = [{"source_node": "1", "target": {"await_signal": copy.deepcopy(signal_type)}}]
        else:
            computations = [{"id": "1", "outputs": [], "body": root_body}, *children]
            associations = [
                {"source_node": row["id"], "target": {"invoke": copy.deepcopy(first)}}
                for row in children
            ]
        ir = _static.minimal_ir(
            root_body if opcode in {"invoke", "await"} else {},
            [] if opcode == "await" else [{"id": "1", "kind": "component", "digest": copy.deepcopy(first["digest"])}],
            computations=computations,
        )
        ir.update(copy.deepcopy(extra))
        ir = _static.prepare_ir_machine_value(ir)
        bundle = copy.deepcopy(base)
        bundle, _, _ = _static.set_full_resolver(
            bundle, ir, [], [first], associations, export_ref=first,
        )
        output.append({
            "opcode": opcode,
            "bundle": bundle,
            "expected_root_operator": opcode,
            "expected_computation_order": [int(row["id"]) for row in computations],
            "expected_leaf_nodes": [int(row["id"]) for row in computations if row["body"].get("invoke") or row["body"].get("await")],
        })

    # CALL_SUBFLOW is independently pinned through the D8 exact target graph.
    target_ir = _static.component_ir(base, first)
    target_ir["semantic_refs"] = [{"specification_id": "agentflow.target.portable-d9@1"}]
    target_asset = _static.canonical_asset(
        base, "ir_assets", target_ir, "semantic-ir", "agentflow.semantic.ir@1", "SemanticIrDocument"
    )
    target_ref = target_asset["reference"]
    caller_ir = _static.subflow_ir(base, target_ref["digest"], "agentflow.target.caller-d9@1")
    subflow_bundle, _, _ = _static.set_full_resolver(
        copy.deepcopy(base), caller_ir, [(target_ir, first)], [first],
        [_static.subflow_association(1, target_ref["digest"], first)], first,
    )
    output.append({
        "opcode": "call_subflow",
        "bundle": subflow_bundle,
        "expected_root_operator": "call_subflow",
        "expected_computation_order": [1],
        "expected_leaf_nodes": [1],
    })

    variants = [
        ("sequence-reversed", {"sequence": {"children": ["3", "2"]}}, [_invoke(2), _invoke(3)], {}, [1, 3, 2], [3, 2]),
        ("parallel-fail-fast", {"parallel": {"mode": "fail_fast", "branches": [
            {"ordinal": "1", "child": "2"}, {"ordinal": "2", "child": "3"},
        ], "max_inflight": None, "output_bindings": []}}, [_invoke(2), _invoke(3)], {}, [1, 2, 3], [2, 3]),
        ("race-three-contestants", {"race": {"branches": [
            {"ordinal": "1", "child": "2"}, {"ordinal": "2", "child": "3"}, {"ordinal": "3", "child": "4"},
        ]}}, [_invoke(2), _invoke(3), _invoke(4)], {}, [1, 2, 3, 4], [2, 3, 4]),
        ("retry-four-attempts", {"retry": {"child": "2", "delays": ["0", "1", "2"]}}, [_invoke(2)], {}, [1, 2], [2]),
        ("timeout-retry-nested", {"timeout": {"child": "2", "timeout": "1"}}, [
            {"id": "2", "outputs": [], "body": {"retry": {"child": "3", "delays": ["0"]}}}, _invoke(3),
        ], {"types": [{"id": "1", "body": {"primitive": "duration"}}], "expressions": [{"id": "1", "result_type": "1", "body": {"constant": {"duration": "50"}}}]}, [1, 2, 3], [3]),
        ("scope-compensation-nested", {"scope": {"child": "2", "resources": []}}, [
            {"id": "2", "outputs": [], "body": {"compensation_scope": {"child": "3"}}}, _invoke(3),
        ], {}, [1, 2, 3], [3]),
        ("compensation-sequence", {"compensation_scope": {"child": "2"}}, [
            {"id": "2", "outputs": [], "body": {"sequence": {"children": ["3", "4"]}}}, _invoke(3), _invoke(4),
        ], {}, [1, 2, 3, 4], [3, 4]),
        ("iterate-while-eight", {"iterate": {"while": {"guard": "1", "body": "2", "carry": [], "max_iterations": "8"}}}, [_invoke(2)], {
            "types": [{"id": "1", "body": {"primitive": "bool"}}], "expressions": [{"id": "1", "result_type": "1", "body": {"constant": {"bool": True}}}],
        }, [1, 2], [2]),
        ("select-two-exclusive-branches", {"select": {"selector": "1", "cases": [
            {"ordinal": "1", "child": "2"}, {"ordinal": "2", "child": "3"},
        ]}}, [_invoke(2), _invoke(3)], {
            "types": [{"id": "1", "body": {"primitive": "int64"}}], "expressions": [{"id": "1", "result_type": "1", "body": {"constant": {"int64": "1"}}}],
        }, [1, 2, 3], [2, 3]),
    ]
    for name, root_body, children, extra, expected_order, expected_leaves in variants:
        computations = [{"id": "1", "outputs": [], "body": root_body}, *children]
        ir = _static.minimal_ir(
            {}, [{"id": "1", "kind": "component", "digest": copy.deepcopy(first["digest"])}],
            computations=computations,
        )
        ir.update(copy.deepcopy(extra))
        ir = _static.prepare_ir_machine_value(ir)
        associations = [
            {"source_node": row["id"], "target": {"invoke": copy.deepcopy(first)}}
            for row in children if row["body"].get("invoke")
        ]
        bundle, _, _ = _static.set_full_resolver(
            copy.deepcopy(base), ir, [], [first], associations, export_ref=first,
        )
        output.append({
            "opcode": name,
            "bundle": bundle,
            "expected_root_operator": next(iter(root_body)),
            "expected_computation_order": expected_order,
            "expected_leaf_nodes": expected_leaves,
        })
    return output


def recovery_fixture_base() -> tuple[dict, dict, dict, dict]:
    """Build one D6-checked, exact K source without action-guard ambiguity."""
    bundle = full_static_positive()
    schema_ref = _static.schema_reference(
        bundle, "agentflow.portable.proof.inputs@1", "PortableProofInputBundle"
    )
    context = bundle["proof_contexts"][0]["content"]
    context["resource_types"]["schemas"]["complete"] = [copy.deepcopy(schema_ref)]
    context["resource_types"]["descriptors"]["complete"] = [{
        "schema": copy.deepcopy(schema_ref),
        "recognized_attributes": [],
    }]
    _static.add_context_catalog(bundle, operations=["compensate", "forward"])
    _static.add_context_catalog(bundle, recovery=True, operations=["compensate", "forward"])

    selector = {"requirement_port_no": "1", "scope_bindings": []}
    forward = {
        "operation": {"catalog": "agentflow.effects.test@1", "name": "forward"},
        "resource_selector": copy.deepcopy(selector),
        "qualifiers": [{"idempotency": "natural"}],
    }
    action_effect = {
        "operation": {"catalog": "agentflow.effects.test@1", "name": "compensate"},
        "resource_selector": copy.deepcopy(selector),
        "qualifiers": [{"idempotency": "none"}],
    }
    action_identity = {
        "forward_effect": copy.deepcopy(forward),
        "action_kind": "compensation",
        "action_effect": copy.deepcopy(action_effect),
    }
    usage_binding = {
        "action": copy.deepcopy(action_identity),
        "activations_per_forward": {"lower": "1", "upper": {"finite": "1"}},
        "attempts_per_activation": {"lower": "1", "upper": {"finite": "1"}},
        "per_attempt_usage": {"limits": []},
    }
    contract = _static.empty_contract("portable.d9.recovery")
    contract["effects"] = {"row": {"closed": [
        {"atom": action_effect, "bounds": {"lower": "0", "upper": {"finite": "1"}}, "guard": {"unconditional": None}},
        {"atom": forward, "bounds": {"lower": "1", "upper": {"finite": "1"}}, "guard": {"unconditional": None}},
    ]}}
    contract["resources"]["requirements"] = [{
        "port_no": "1", "resource_type": {"schema": copy.deepcopy(schema_ref)},
        "required_attributes": [],
    }]
    contract["recovery"] = {
        "replay_semantics": "recompute",
        "effect_recovery_clauses": [{
            "effect": copy.deepcopy(forward),
            "idempotency": "natural",
            "key_requirement": {"none": None},
            "reconciliation": "none",
            "compensation": {"required": {"catalog": "agentflow.effects.test@1", "name": "compensate"}},
            "unknown_effect_handling": "block",
        }],
    }
    contract["recovery_action_bindings"] = [{"identity": copy.deepcopy(action_identity)}]
    contract["recovery_usage_bindings"] = [copy.deepcopy(usage_binding)]

    operations = [
        {"catalog": "agentflow.effects.test@1", "name": "compensate"},
        {"catalog": "agentflow.effects.test@1", "name": "forward"},
    ]
    qualifier_profiles = [
        {"qualifiers": [{"idempotency": "natural"}]},
        {"qualifiers": [{"idempotency": "none"}]},
    ]
    pairs = [{
        "operation": copy.deepcopy(operation),
        "resource_type": {"schema": copy.deepcopy(schema_ref)},
        "allowed_qualifier_profiles": copy.deepcopy(qualifier_profiles),
        "selector_keys": [],
    } for operation in operations]
    _static.install_relation(
        bundle, "effect_resource", "agentflow.effect.resource.compatibility@1",
        "EffectResourceCompatibilityCatalog",
        {"complete_operations": operations, "supported_pairs": pairs},
    )
    _static.install_relation(
        bundle, "recovery_usage", "agentflow.recovery.usage.coverage@1",
        "RecoveryUsageCoverageCatalog",
        {"rules": [{"action": copy.deepcopy(action_identity), "complete_meters": []}]},
    )
    bundle, source_ref = _static.install_contract(bundle, contract)
    return bundle, source_ref, schema_ref, action_identity


def d9_recovery_cases() -> list[dict]:
    base, component, resource_schema, _action = recovery_fixture_base()

    def invoke(node: int) -> dict:
        return {
            "id": str(node), "outputs": [],
            "body": {"invoke": {"contract": "1", "inputs": [], "resources": [{"port_no": "1", "resource_slot": "1"}]}},
        }

    def build(opcode: str, computations: list[dict], *, iteration_bound: int | None = None) -> dict:
        ir = _static.minimal_ir({}, [{"id": "1", "kind": "component", "digest": copy.deepcopy(component["digest"])}], computations=computations)
        ir["resources"] = [{"id": "1", "requirement_type": copy.deepcopy(resource_schema)}]
        if iteration_bound is not None:
            ir["types"] = [{"id": "1", "body": {"primitive": "bool"}}]
            ir["expressions"] = [{"id": "1", "result_type": "1", "body": {"constant": {"bool": True}}}]
        ir = _static.prepare_ir_machine_value(ir)
        associations = [
            {"source_node": row["id"], "target": {"invoke": copy.deepcopy(component)}}
            for row in computations if row["body"].get("invoke")
        ]
        bundle, _, _ = _static.set_full_resolver(
            copy.deepcopy(base), ir, [], [component], associations, export_ref=component,
        )
        return bundle

    output: list[dict] = []
    for attempts in range(1, 5):
        delays = ["0"] * (attempts - 1)
        computations = [
            {"id": "1", "outputs": [], "body": {"retry": {"child": "2", "delays": delays}}},
            {"id": "2", "outputs": [], "body": {"compensation_scope": {"child": "3"}}},
            invoke(3),
        ]
        output.append({
            "scenario": f"outer-retry-{attempts}", "bundle": build("retry", computations),
            "expected_action_count": 1, "expected_root_action_uppers": [attempts],
            "expected_local_action_uppers": [1],
        })

    for bound in range(1, 5):
        computations = [
            {"id": "1", "outputs": [], "body": {"iterate": {"while": {
                "guard": "1", "body": "2", "carry": [], "max_iterations": str(bound),
            }}}},
            {"id": "2", "outputs": [], "body": {"compensation_scope": {"child": "3"}}},
            invoke(3),
        ]
        output.append({
            "scenario": f"outer-iterate-{bound}", "bundle": build("iterate", computations, iteration_bound=bound),
            "expected_action_count": 1, "expected_root_action_uppers": [bound],
            "expected_local_action_uppers": [1],
        })

    for attempts, iterations in [(2, 2), (2, 4), (3, 2), (3, 4)]:
        computations = [
            {"id": "1", "outputs": [], "body": {"retry": {"child": "2", "delays": ["0"] * (attempts - 1)}}},
            {"id": "2", "outputs": [], "body": {"iterate": {"while": {
                "guard": "1", "body": "3", "carry": [], "max_iterations": str(iterations),
            }}}},
            {"id": "3", "outputs": [], "body": {"compensation_scope": {"child": "4"}}},
            invoke(4),
        ]
        output.append({
            "scenario": f"retry-{attempts}-times-iterate-{iterations}",
            "bundle": build("retry_iterate", computations, iteration_bound=iterations),
            "expected_action_count": 1,
            "expected_root_action_uppers": [attempts * iterations],
            "expected_local_action_uppers": [1],
        })

    special = [
        ("local-retry-inside-compensation-scope", [
            {"id": "1", "outputs": [], "body": {"compensation_scope": {"child": "2"}}},
            {"id": "2", "outputs": [], "body": {"retry": {"child": "3", "delays": ["0", "0"]}}}, invoke(3),
        ], 1, 3, 3),
        ("two-sequence-actions", [
            {"id": "1", "outputs": [], "body": {"sequence": {"children": ["2", "4"]}}},
            {"id": "2", "outputs": [], "body": {"compensation_scope": {"child": "3"}}}, invoke(3),
            {"id": "4", "outputs": [], "body": {"compensation_scope": {"child": "5"}}}, invoke(5),
        ], 2, 1, 1),
        ("race-retains-loser-action", [
            {"id": "1", "outputs": [], "body": {"race": {"branches": [{"ordinal": "1", "child": "2"}, {"ordinal": "2", "child": "4"}]}}},
            {"id": "2", "outputs": [], "body": {"compensation_scope": {"child": "3"}}}, invoke(3),
            {"id": "4", "outputs": [], "body": {"compensation_scope": {"child": "5"}}}, invoke(5),
        ], 2, 1, 1),
        ("parallel-keeps-both-actions", [
            {"id": "1", "outputs": [], "body": {"parallel": {"mode": "all", "branches": [{"ordinal": "1", "child": "2"}, {"ordinal": "2", "child": "4"}], "max_inflight": None, "output_bindings": []}}},
            {"id": "2", "outputs": [], "body": {"compensation_scope": {"child": "3"}}}, invoke(3),
            {"id": "4", "outputs": [], "body": {"compensation_scope": {"child": "5"}}}, invoke(5),
        ], 2, 1, 1),
    ]
    for scenario, computations, count, upper, local_upper in special:
        output.append({
            "scenario": scenario,
            "bundle": build(scenario, computations, iteration_bound=1 if "iterate" in scenario else None),
            "expected_action_count": count,
            "expected_root_action_uppers": [upper] * count,
            "expected_local_action_uppers": [local_upper] * count,
        })
    return output


def provenance_cases() -> list[dict]:
    base = full_static_positive()
    signal_type = _static.schema_reference(base, "agentflow.semantic.ir@1", "SemanticIrDocument")
    return [
        {"case_id":"portable-root-export-revalidation/provenance-context-id-missing","group":"Context-Resolver-Compiler","expected_stage":"D2","mutation":{"path":["root_export_assertions","0","assertion","proof_context_manifest","context_id"],"value":"99"}},
        {"case_id":"portable-root-export-revalidation/provenance-resolver-id-missing","group":"Context-Resolver-Compiler","expected_stage":"D2","mutation":{"path":["root_export_assertions","0","assertion","proof_context_manifest","resolver_id"],"value":"99"}},
        {"case_id":"portable-root-export-revalidation/provenance-compiler-id-missing","group":"Context-Resolver-Compiler","expected_stage":"D2","mutation":{"path":["root_export_assertions","0","assertion","compiler_semantics_id"],"value":"99"}},
        {"case_id":"portable-root-export-revalidation/provenance-root-assertion-id-missing","group":"Context-Resolver-Compiler","expected_stage":"D2","mutation":{"path":["root_assertion_id"],"value":"99"}},
        {"case_id":"portable-root-export-revalidation/provenance-compiler-version-drift","group":"Context-Resolver-Compiler","expected_stage":"D4","mutation":{"path":["compiler_semantics_manifests","0","content","static_identity_projection","analysis_semantics_version"],"value":"agentflow.root-export-checker@0"}},
        {"case_id":"portable-root-export-revalidation/provenance-applied-rule-content-drift","group":"Context-Resolver-Compiler","expected_stage":"D3","mutation":{"path":["compiler_semantics_manifests","0","content","static_identity_projection","applied_rules","0","short_statement"],"value":"mutated rule attribution is not current support"}},
        {"case_id":"portable-root-export-revalidation/provenance-resolver-source-drift","group":"Context-Resolver-Compiler","expected_stage":"D8","mutation":{"path":["resolver_dependency_manifests","0","content","source_associations","0","target"],"value":{"await_signal":signal_type}}},
        {"case_id":"portable-root-export-revalidation/provenance-fresh-context-unit-change","group":"Context-Resolver-Compiler","expected_stage":"D10","expected_static_compatibility":"proven","expected_capability_issued":True,"mutation":{"path":["proof_contexts","0","content","usage_meters","complete","0","unit","name"],"value":"call-v2"}},
    ]


def p30_effect_case(*, split_rows: bool) -> tuple[dict, dict]:
    """Build a fully referenced Root/Export effect selector containment case."""
    bundle = full_static_positive()
    guard_schema = {
        "schema": "agentflow.guard.fixture@1",
        "name": "GuardInt64",
        "body": {"alias": {"primitive": "int64"}},
    }
    schema_bytes = canonical_bytes(guard_schema)
    schema_ref = {
        "specification_id": guard_schema["schema"],
        "type_name": guard_schema["name"],
        "schema_digest": {"algorithm": "sha256", "domain": "type-schema", "hex": hash_v1("type-schema", schema_bytes)},
    }
    bundle["schema_assets"].append({"reference": copy.deepcopy(schema_ref), "canonical_content": b64url(schema_bytes)})
    bundle["schema_assets"].sort(key=lambda row:(row["reference"]["specification_id"],row["reference"]["type_name"]))
    context = bundle["proof_contexts"][0]["content"]
    context["type_schemas"]["complete"].append(copy.deepcopy(schema_ref))
    context["type_schemas"]["complete"].sort(key=lambda row:(row["specification_id"],row["type_name"],row["schema_digest"]["hex"]))
    context["resource_types"]["schemas"]["complete"] = [copy.deepcopy(schema_ref)]
    context["resource_types"]["descriptors"]["complete"] = [{"schema":copy.deepcopy(schema_ref),"recognized_attributes":["a","b"]}]
    _static.add_context_catalog(bundle, operations=["write"])

    selector = lambda bindings: {"requirement_port_no":"1","scope_bindings":[{"key":key,"value":"1"} for key in bindings]}
    atom = lambda bindings: {"operation":{"catalog":"agentflow.effects.test@1","name":"write"},"resource_selector":selector(bindings),"qualifiers":[{"idempotency":"natural"}]}
    root_atom = atom(["a","b"])
    split_a = atom(["a"])
    split_b = atom(["b"])
    unrestricted = {"unconditional":None}
    p_zero = {"input_equals":{"port_no":"1","expected":{"int64":"0"}}}
    root_effect_entry = {"atom":copy.deepcopy(root_atom),"bounds":{"lower":"0","upper":{"finite":"10"}},"guard":unrestricted}
    export_effects = [
        {"atom":copy.deepcopy(split_a),"bounds":{"lower":"0","upper":{"finite":"10"}},"guard":p_zero},
        {"atom":copy.deepcopy(split_b),"bounds":{"lower":"0","upper":{"finite":"5"}},"guard":unrestricted},
    ] if split_rows else [copy.deepcopy(root_effect_entry)]

    template = json.loads(_static.b64decode(_static.find_contract(bundle)["canonical_content"]).decode("utf-8"))
    def make_contract(name: str, effect_entries: list[dict]) -> dict:
        contract = copy.deepcopy(template)
        contract["component_identity"]["name"] = name
        contract["inputs"] = [{"port_no":"1","type":copy.deepcopy(schema_ref)}]
        contract["resources"]["requirements"] = [{"port_no":"1","resource_type":{"schema":copy.deepcopy(schema_ref)},"required_attributes":[]}]
        contract["effects"] = {"row":{"closed":copy.deepcopy(effect_entries)}}
        return contract

    source_contract = make_contract("portable.p30.source", [root_effect_entry])
    export_contract = make_contract("portable.p30.export", export_effects)
    source_asset = _static.canonical_asset(bundle,"contract_assets",source_contract,"component-contract","agentflow.component.contract@1","ComponentContract")
    export_asset = _static.canonical_asset(bundle,"contract_assets",export_contract,"component-contract","agentflow.component.contract@1","ComponentContract")
    source_ref, export_ref = source_asset["reference"], export_asset["reference"]
    bundle["contract_assets"] = [source_asset,export_asset]
    bundle["contract_assets"].sort(key=lambda row:(row["reference"]["digest"]["algorithm"],row["reference"]["digest"]["domain"],row["reference"]["digest"]["hex"]))

    operation = {"catalog":"agentflow.effects.test@1","name":"write"}
    pair = {
        "operation":copy.deepcopy(operation),
        "resource_type":{"schema":copy.deepcopy(schema_ref)},
        "allowed_qualifier_profiles":[{"qualifiers":[{"idempotency":"natural"}]}],
        "selector_keys":[
            {"selector_key":"a","resource_attribute_key":"a","required":False,"values":{"opaque_nfc_exact_string":None}},
            {"selector_key":"b","resource_attribute_key":"b","required":False,"values":{"opaque_nfc_exact_string":None}},
        ],
    }
    _static.install_relation(
        bundle,"effect_resource","agentflow.effect.resource.compatibility@1","EffectResourceCompatibilityCatalog",
        {"complete_operations":[operation],"supported_pairs":[pair]},
    )

    caller_ir = _static.minimal_ir({},[{"id":"1","kind":"component","digest":copy.deepcopy(source_ref["digest"])}],computations=[{
        "id":"1","outputs":[],"body":{"invoke":{"contract":"1","inputs":[{"port_no":"1","expression":"1"}],"resources":[{"port_no":"1","resource_slot":"1"}]}}
    }])
    caller_ir["types"] = [{"id":"1","body":{"schema":copy.deepcopy(schema_ref)}}]
    caller_ir["values"] = [{"id":"11","type":"1"}]
    caller_ir["expressions"] = [{"id":"1","result_type":"1","body":{"value_slot":"11"}}]
    caller_ir["resources"] = [{"id":"1","requirement_type":copy.deepcopy(schema_ref)}]
    caller_ir["run_parameters"] = [{"id":"1","name":"p1","type":"1","value_slot":"11"}]
    bundle, caller_ref, _ = _static.set_full_resolver(
        bundle,caller_ir,[],[source_ref,export_ref],[_static.invoke_association(1,source_ref)],export_ref,
    )
    assertion = bundle["root_export_assertions"][0]["assertion"]
    assertion["input_mappings"] = [{
        "export_input_port":{"exported_contract_digest":copy.deepcopy(export_ref["digest"]),"port_no":"1"},
        "run_parameter_id":"1","root_value_slot":"11","root_type":"1","export_type":copy.deepcopy(schema_ref),
        "use_sites":[{"computation":"1","source_contract_digest":copy.deepcopy(source_ref["digest"]),"source_component_port":"1","input_expression":"1","exact_type":copy.deepcopy(schema_ref),"dependency_slots":["11"]}],
    }]
    assertion["resource_mappings"] = [{
        "export_resource_port":{"exported_contract_digest":copy.deepcopy(export_ref["digest"]),"port_no":"1"},
        "root_resource_slot":"1","root_resource_type":copy.deepcopy(schema_ref),"export_resource_type":{"schema":copy.deepcopy(schema_ref)},
        "source_coordinates":[{"source_node":"1","source_contract_digest":copy.deepcopy(source_ref["digest"]),"component_resource_port":"1","resource_slot":"1","alias_component_ports":["1"]}],
    }]
    selected_export_atom = split_a if split_rows else root_atom
    selected_guard = p_zero if split_rows else unrestricted
    assertion["effect_mappings"] = [{
        "source":{"caller_semantic_ir_digest":copy.deepcopy(caller_ref["digest"]),"source_node":"1","source_contract_digest":copy.deepcopy(source_ref["digest"]),"source_entry_ordinal":"0","atom":copy.deepcopy(root_atom),"guard":copy.deepcopy(unrestricted)},
        "root_resource_slot":{"some":"1"},"export_atom":copy.deepcopy(selected_export_atom),"export_guard":copy.deepcopy(selected_guard),
    }]
    assertion["execution_mappings"] = [{"source_node":"1","source_contract_digest":copy.deepcopy(source_ref["digest"]),"exact_execution_requirements":copy.deepcopy(source_contract["execution"])}]
    bundle["structural_types"] = [{"id":"1","body":{"schema":copy.deepcopy(schema_ref)}}]

    expected = {
        "static_compatibility":"unproven" if split_rows else "proven",
        "capability_issued":not split_rows,
        "epsilon":"unproven" if split_rows else "proven",
        "export_effect_entry_count":len(export_effects),
        "root_effect_entry_count":1,
        "split_rows":split_rows,
    }
    return bundle, expected


def p31_resolver_cases() -> list[tuple[str, dict, dict]]:
    """Create same-target A/B exports with exact portable resolver pins."""
    base, _ = p30_effect_case(split_rows=False)
    source_asset = next(
        row for row in base["contract_assets"]
        if json.loads(_static.b64decode(row["canonical_content"]).decode("utf-8"))["component_identity"]["name"] == "portable.p30.source"
    )
    source_ref = copy.deepcopy(source_asset["reference"])
    source_contract = json.loads(_static.b64decode(source_asset["canonical_content"]).decode("utf-8"))
    base["contract_assets"] = [
        row for row in base["contract_assets"]
        if json.loads(_static.b64decode(row["canonical_content"]).decode("utf-8"))["component_identity"]["name"] != "portable.p30.export"
    ]
    schema_ref = copy.deepcopy(base["structural_types"][0]["body"]["schema"])
    root_atom = copy.deepcopy(source_contract["effects"]["row"]["closed"][0]["atom"])

    def component_contract(name: str, upper: int) -> tuple[dict, dict]:
        contract = copy.deepcopy(source_contract)
        contract["component_identity"]["name"] = name
        contract["effects"]["row"]["closed"][0]["bounds"]["upper"] = {"finite": str(upper)}
        asset = _static.canonical_asset(base,"contract_assets",contract,"component-contract","agentflow.component.contract@1","ComponentContract")
        return contract, asset

    target_a_contract, target_a_asset = component_contract("portable.p31.target.a",1)
    target_b_contract, target_b_asset = component_contract("portable.p31.target.b",10)
    top_contract, top_asset = component_contract("portable.p31.top",1)
    for asset in (target_a_asset,target_b_asset,top_asset):
        if not any(row["reference"] == asset["reference"] for row in base["contract_assets"]):
            base["contract_assets"].append(asset)
    base["contract_assets"].sort(key=lambda row:(row["reference"]["digest"]["algorithm"],row["reference"]["digest"]["domain"],row["reference"]["digest"]["hex"]))

    def ir(body: dict, contract_ref: dict, kind: str) -> dict:
        document = _static.minimal_ir(body,[{"id":"1","kind":kind,"digest":copy.deepcopy(contract_ref["digest"])}],computations=[{
            "id":"1","outputs":[],"body":body,
        }])
        document["types"] = [{"id":"1","body":{"schema":copy.deepcopy(schema_ref)}}]
        document["values"] = [{"id":"11","type":"1"}]
        document["expressions"] = [{"id":"1","result_type":"1","body":{"value_slot":"11"}}]
        document["resources"] = [{"id":"1","requirement_type":copy.deepcopy(schema_ref)}]
        document["run_parameters"] = [{"id":"1","name":"p1","type":"1","value_slot":"11"}]
        return document

    target_ir = ir({"invoke":{"contract":"1","inputs":[{"port_no":"1","expression":"1"}],"resources":[{"port_no":"1","resource_slot":"1"}]}},source_ref,"component")
    target_ref = _static.canonical_asset(base,"ir_assets",target_ir,"semantic-ir","agentflow.semantic.ir@1","SemanticIrDocument")["reference"]
    caller_ir = ir({"call_subflow":{"contract":"1","inputs":[{"port_no":"1","expression":"1"}],"resources":[{"port_no":"1","resource_slot":"1"}]}},target_ref,"subflow")

    def build(current_target_export: dict, declared_target_export: dict, *, fresh_upper_one: bool) -> dict:
        bundle, caller_ref, _ = _static.set_full_resolver(
            copy.deepcopy(base),
            caller_ir,
            [(target_ir,current_target_export)],
            [source_ref,top_asset["reference"],target_a_asset["reference"]],
            [_static.subflow_association(1,target_ref["digest"],current_target_export)],
            top_asset["reference"],
        )
        assertion = bundle["root_export_assertions"][0]["assertion"]
        assertion["input_mappings"] = [{
            "export_input_port":{"exported_contract_digest":copy.deepcopy(top_asset["reference"]["digest"]),"port_no":"1"},
            "run_parameter_id":"1","root_value_slot":"11","root_type":"1","export_type":copy.deepcopy(schema_ref),
            "use_sites":[{"computation":"1","source_contract_digest":copy.deepcopy(current_target_export["digest"]),"source_component_port":"1","input_expression":"1","exact_type":copy.deepcopy(schema_ref),"dependency_slots":["11"]}],
        }]
        assertion["resource_mappings"] = [{
            "export_resource_port":{"exported_contract_digest":copy.deepcopy(top_asset["reference"]["digest"]),"port_no":"1"},
            "root_resource_slot":"1","root_resource_type":copy.deepcopy(schema_ref),"export_resource_type":{"schema":copy.deepcopy(schema_ref)},
            "source_coordinates":[{"source_node":"1","source_contract_digest":copy.deepcopy(current_target_export["digest"]),"component_resource_port":"1","resource_slot":"1","alias_component_ports":["1"]}],
        }]
        assertion["effect_mappings"] = [{
            "source":{"caller_semantic_ir_digest":copy.deepcopy(caller_ref["digest"]),"source_node":"1","source_contract_digest":copy.deepcopy(current_target_export["digest"]),"source_entry_ordinal":"0","atom":copy.deepcopy(root_atom),"guard":{"unconditional":None}},
            "root_resource_slot":{"some":"1"},"export_atom":copy.deepcopy(root_atom),"export_guard":{"unconditional":None},
        }]
        assertion["execution_mappings"] = [{"source_node":"1","source_contract_digest":copy.deepcopy(current_target_export["digest"]),"exact_execution_requirements":copy.deepcopy(target_b_contract["execution"])}]
        assertion["subflow_dependencies"] = [{
            "call_site":"1","target_semantic_ir_digest":copy.deepcopy(target_ref["digest"]),
            "target_exported_contract_digest":{"some":copy.deepcopy(declared_target_export["digest"])},
            "target_root_assertion_id":{"none":None},"transitive_targets":[],
        }]
        return bundle

    mismatch = build(target_b_asset["reference"],target_a_asset["reference"],fresh_upper_one=False)
    fresh_b = build(target_b_asset["reference"],target_b_asset["reference"],fresh_upper_one=True)
    return [
        ("p31-snapshot-a-declaration-b-current", mismatch, {"expected_static_compatibility":"unproven","expected_capability_issued":False,"expected_target_ir_digest":target_ref["digest"],"expected_current_target_export_digest":target_b_asset["reference"]["digest"],"expected_declared_target_export_digest":target_a_asset["reference"]["digest"],"expected_reason":"portable assertion names A while D8 snapshot binds same target IR to B"}),
        ("p31-fresh-root-b-top-ceiling-one", fresh_b, {"expected_static_compatibility":"violation","expected_capability_issued":False,"expected_epsilon":"violation","expected_target_ir_digest":target_ref["digest"],"expected_current_target_export_digest":target_b_asset["reference"]["digest"],"expected_declared_target_export_digest":target_b_asset["reference"]["digest"],"expected_reason":"fresh D9 root derives B upper ten against top export upper one"}),
    ]


def main() -> None:
    bundle = full_static_positive()
    payload = canonical_bytes(bundle)
    OUT.mkdir(parents=True, exist_ok=True)
    assert_historical_output(ROOT / POSITIVE_PATH, payload)
    expected = independent_positive_expected(bundle)
    missing_execution = copy.deepcopy(bundle)
    missing_execution["root_export_assertions"][0]["assertion"]["execution_mappings"] = []
    missing_payload = canonical_bytes(missing_execution)
    missing_path = "spec/conformance/portable-root-export-revalidation/negative-execution-map-missing.ace1"
    assert_historical_output(ROOT / missing_path, missing_payload)
    missing_expected = independent_positive_expected(missing_execution, missing_execution=True)
    d9_cases = d9_opcode_cases()
    d9_manifest_cases = []
    for case in d9_cases:
        relative = f"{D9_DIR}/{case['opcode']}.ace1"
        raw = canonical_bytes(case["bundle"])
        (ROOT / relative).parent.mkdir(parents=True, exist_ok=True)
        assert_historical_output(ROOT / relative, raw)
        d9_manifest_cases.append({
            "case_id": f"portable-root-export-revalidation/d9-opcode-{case['opcode']}",
            "group": "D9-Control",
            "bundle_file": relative,
            "bundle_sha256": __import__("hashlib").sha256(raw).hexdigest(),
            "expected_root_operator": case["expected_root_operator"],
            "expected_computation_order": case["expected_computation_order"],
            "expected_leaf_nodes": case["expected_leaf_nodes"],
        })
    recovery_cases = d9_recovery_cases()
    recovery_manifest_cases = []
    for case in recovery_cases:
        slug = case["scenario"].replace("_", "-")
        relative = f"{D9_DIR}/recovery/{slug}.ace1"
        raw = canonical_bytes(case["bundle"])
        (ROOT / relative).parent.mkdir(parents=True, exist_ok=True)
        assert_historical_output(ROOT / relative, raw)
        caller_ref = case["bundle"]["resolver_dependency_manifests"][0]["content"]["caller_ir"]
        caller_ir = json.loads(_static.b64decode(
            _static.find_ir(case["bundle"], caller_ref["digest"]["hex"])["canonical_content"]
        ).decode("utf-8"))
        scenario = case["scenario"]
        if scenario.startswith("outer-"):
            root_factor_count, local_factor_count = 1, 0
        elif scenario.startswith("retry-"):
            root_factor_count, local_factor_count = 2, 0
        elif scenario == "local-retry-inside-compensation-scope":
            root_factor_count, local_factor_count = 1, 1
        else:
            root_factor_count, local_factor_count = 0, 0
        recovery_manifest_cases.append({
            "case_id": f"portable-root-export-revalidation/d9-recovery-{slug}",
            "group": "D9-Recovery",
            "bundle_file": relative,
            "bundle_sha256": __import__("hashlib").sha256(raw).hexdigest(),
            "expected_action_count": case["expected_action_count"],
            "expected_source_nodes": [int(row["id"]) for row in caller_ir["computations"] if "invoke" in row["body"]],
            "expected_forward_entry_ordinal": 1,
            "expected_root_action_uppers": case["expected_root_action_uppers"],
            "expected_local_action_uppers": case["expected_local_action_uppers"],
            "expected_root_factor_counts": [root_factor_count] * case["expected_action_count"],
            "expected_local_factor_counts": [local_factor_count] * case["expected_action_count"],
        })
    cases = [
        {
            "case_id": "portable-root-export-revalidation/d10-full-static-positive",
            "group": "D10",
            "bundle_file": POSITIVE_PATH,
            "bundle_sha256": __import__("hashlib").sha256(payload).hexdigest(),
            "expected_static_compatibility": expected["static_compatibility"],
            "expected_capability_issued": expected["capability_issued"],
            "expected_dimension_verdicts": expected["dimension_verdicts"],
        },
        {
            "case_id": "portable-root-export-revalidation/d10-execution-source-unmapped",
            "group": "D10",
            "bundle_file": missing_path,
            "bundle_sha256": __import__("hashlib").sha256(missing_payload).hexdigest(),
            "expected_static_compatibility": missing_expected["static_compatibility"],
            "expected_capability_issued": missing_expected["capability_issued"],
            "expected_dimension_verdicts": missing_expected["dimension_verdicts"],
        },
    ]
    for split_rows in (True, False):
        p30_bundle, p30_expected = p30_effect_case(split_rows=split_rows)
        slug = "p30-split-row-negative" if split_rows else "p30-same-row-positive"
        relative = f"spec/conformance/portable-root-export-revalidation/{slug}.ace1"
        p30_payload = canonical_bytes(p30_bundle)
        assert_historical_output(ROOT / relative, p30_payload)
        cases.append({
            "case_id": f"portable-root-export-revalidation/{slug}",
            "group": "D10",
            "bundle_file": relative,
            "bundle_sha256": __import__("hashlib").sha256(p30_payload).hexdigest(),
            "expected_static_compatibility": p30_expected["static_compatibility"],
            "expected_capability_issued": p30_expected["capability_issued"],
            "expected_epsilon": p30_expected["epsilon"],
            "expected_root_effect_entry_count": p30_expected["root_effect_entry_count"],
            "expected_export_effect_entry_count": p30_expected["export_effect_entry_count"],
        })
    p31_manifest_cases = []
    for slug, p31_bundle, p31_expected in p31_resolver_cases():
        relative = f"spec/conformance/portable-root-export-revalidation/{slug}.ace1"
        p31_payload = canonical_bytes(p31_bundle)
        assert_historical_output(ROOT / relative, p31_payload)
        p31_manifest_cases.append({
            "case_id": f"portable-root-export-revalidation/{slug}",
            "group": "D10",
            "bundle_file": relative,
            "bundle_sha256": __import__("hashlib").sha256(p31_payload).hexdigest(),
            **p31_expected,
        })
    cases.extend(p31_manifest_cases)
    for node in range(2, 22):
        cases.append({
            "case_id": f"portable-root-export-revalidation/d10-execution-source-node-{node}-not-root",
            "group": "D10",
            "bundle_file": POSITIVE_PATH,
            "bundle_sha256": __import__("hashlib").sha256(payload).hexdigest(),
            "mutations": [{
                "op": "set",
                "path": ["root_export_assertions", "0", "assertion", "execution_mappings", "0", "source_node"],
                "value": str(node),
            }],
            "expected_execution_verdict": "not_proven",
            "expected_capability_issued": False,
        })
    provenance = provenance_cases()
    for case in provenance:
        case["bundle_file"] = POSITIVE_PATH
        case["bundle_sha256"] = __import__("hashlib").sha256(payload).hexdigest()
        mutation = case.pop("mutation")
        mutation["op"] = "set"
        case["mutations"] = [mutation]
    by_case_id = {case["case_id"]: case for case in [*cases, *provenance]}
    cross_process_sources = [
        ("positive", "portable-root-export-revalidation/d10-full-static-positive", "D10"),
        ("p30-split-row", "portable-root-export-revalidation/p30-split-row-negative", "D10"),
        ("p31-stale-target-export", "portable-root-export-revalidation/p31-snapshot-a-declaration-b-current", "D10"),
        ("context-content-change", "portable-root-export-revalidation/provenance-fresh-context-unit-change", "D10"),
        ("compiler-semantics-drift", "portable-root-export-revalidation/provenance-compiler-version-drift", "D4"),
        ("resolver-association-drift", "portable-root-export-revalidation/provenance-resolver-source-drift", "D8"),
    ]
    cross_process_cases = []
    for suffix, source_id, expected_stage in cross_process_sources:
        row = copy.deepcopy(by_case_id[source_id])
        row["case_id"] = f"portable-root-export-revalidation/cross-process-{suffix}"
        row["group"] = "Cross-Process"
        row["expected_consumer_stage"] = expected_stage
        cross_process_cases.append(row)
    manifest = {
        "corpus": "agentflow.portable.root-export-revalidation",
        "revision": 1,
        "positive_file": POSITIVE_PATH,
        "positive_sha256": __import__("hashlib").sha256(payload).hexdigest(),
        "positive_bytes": len(payload),
        "positive_expected": expected,
        "negative_expected": missing_expected,
        "comparator_mutation_count": len(COMPARATOR_MUTATIONS),
        "comparator_mutations": COMPARATOR_MUTATIONS,
        "case_count": len(cases) + len(d9_manifest_cases) + len(recovery_manifest_cases) + len(provenance) + len(cross_process_cases),
        "group_counts": {"D9-Control": len(d9_manifest_cases), "D9-Recovery": len(recovery_manifest_cases), "D10": len(cases), "Context-Resolver-Compiler": len(provenance), "Cross-Process": len(cross_process_cases)},
        "cases": [*d9_manifest_cases, *recovery_manifest_cases, *provenance, *cases, *cross_process_cases],
    }
    assert_historical_output(OUT / "manifest.json", json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"portable root/export fixtures: {manifest['case_count']}; independent declaration oracle")


if __name__ == "__main__":
    main()
