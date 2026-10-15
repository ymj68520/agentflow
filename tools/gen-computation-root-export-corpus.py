#!/usr/bin/env python3
"""Independent Phase 4B.2E.2 RootExport oracle and fixture generator.

The oracle below derives expected dimension states from declared root/export
facts and intentional mutations only. It never invokes Rust, reads a production
projection, or updates Expected values from observed output.
"""

from __future__ import annotations

import hashlib
from legacy_edition1_inputs import assert_historical_output
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "spec/conformance/computation-root-export"
DIMENSIONS = ["I", "O", "Phi", "epsilon", "R", "F", "U", "K", "X"]
RULES = {
    "I": "AF-SEMANTIC-0162", "O": "AF-SEMANTIC-0163", "Phi": "AF-SEMANTIC-0165",
    "epsilon": "AF-SEMANTIC-0166", "R": "AF-SEMANTIC-0164", "F": "AF-SEMANTIC-0167",
    "U": "AF-SEMANTIC-0168", "K": "AF-SEMANTIC-0169", "X": "AF-SEMANTIC-0170",
}
SPECS = [
    ("agentflow.component.contract.semantics@1", "spec/component/component-contract.md"),
    ("agentflow.component.relation-catalogs.semantics@1", "spec/component/relation-catalogs.md"),
    ("agentflow.computation.root-export-contract-proof.semantics@1", "spec/computation/root-export-contract-proof.md"),
    ("agentflow.effect.signature.semantics@1", "spec/effect/effect-signature.md"),
    ("agentflow.execution.requirements.semantics@1", "spec/execution/execution-requirements.md"),
    ("agentflow.failure.row.semantics@1", "spec/failure/failure-row.md"),
    ("agentflow.information.flow.semantics@1", "spec/information-flow/information-flow.md"),
    ("agentflow.recovery.contract.semantics@1", "spec/recovery/recovery-contract.md"),
    ("agentflow.resource.requirement.semantics@1", "spec/resource/resource-requirement.md"),
    ("agentflow.semantic.advanced-control-contract-algebra.semantics@1", "spec/computation/advanced-control-contract-algebra.md"),
    ("agentflow.semantic.control-contract-algebra.semantics@1", "spec/computation/control-contract-algebra.md"),
    ("agentflow.semantic.ir.semantics@1", "spec/semantic/semantic-ir.md"),
    ("agentflow.semantic.leaf-contract-resolution.semantics@1", "spec/computation/leaf-contract-resolution.md"),
    ("agentflow.semantic.scope-compensation-contract-algebra.semantics@1", "spec/computation/scope-compensation-contract-algebra.md"),
    ("agentflow.semantic.types@1", "spec/semantic/core-semantic-types.md"),
    ("agentflow.usage.signature.semantics@1", "spec/usage/usage-signature.md"),
]


def effect_atom(member: str = "natural", port: int = 1) -> dict:
    qualifiers = [] if member == "plain" else [{"family": "idempotency", "member": member}]
    return {
        "operation": {"catalog": "agentflow.effects@1", "name": "write"},
        "qualifiers": qualifiers,
        "resource_selector": {"requirement_port_no": port, "scope_bindings": []},
    }


def effect_entry(atom: dict, upper: int, guard: object = "unconditional") -> dict:
    return {"atom": atom, "lower": 0, "upper": upper, "guard": guard}


def recovery_action(*, per_attempt: int = 1, forward_guard: object = "unconditional") -> dict:
    forward = effect_atom("natural")
    action = effect_atom("none")
    return {
        "action": "compensate",
        "forward_atom": forward,
        "action_atom": action,
        "activations_per_forward": {"lower": 0, "upper": 1},
        "attempts_per_activation": {"lower": 1, "upper": 1},
        "clause": {
            "compensation": {"required": {"catalog": "agentflow.effects@1", "name": "write"}},
            "idempotency": "natural", "key_requirement": None,
            "reconciliation": "none", "unknown_effect_handling": "block",
        },
        "coverage_meters": ["api_calls"],
        "per_attempt_usage": {"coordinates": {"recovery/api_calls": per_attempt}},
    }


def base_case(case_id: str, *, inputs=(), outputs=(), resources=(), effects=None,
              usage=None, failures=(), flow=None, execution=None, recoveries=(),
              effect_flow_partitions=(), mutation=None, export_overrides=None, subflow=False, expected_stage=None) -> dict:
    effects = effects if effects is not None else {"row": "closed", "entries": []}
    flow = flow if flow is not None else {
        "authority": "none", "constraints": [], "influence_unknown": False,
        "sources": [], "taints": [],
    }
    usage = usage if usage is not None else {"coordinates": {"forward/api_calls": 0, "recovery/api_calls": 0}}
    outputs = list(outputs)
    parent_outputs = [{"port": item["root_port"], "slot": item.get("parent_slot", item["slot"]), "type_id": item["type_id"]} for item in outputs]
    workflow_outputs = [
        {"name": item["name"], "expression": item["expression"], "slot": item.get("parent_slot", item["slot"]),
         "type_id": item["type_id"], "root_port": item["root_port"], "export_port": item["export_port"]}
        for item in outputs
    ]
    child = {
        "node": 2,
        "contract_digest": "component-contract:fixture",
        "effects": effects,
        "failures": list(failures),
        "flow": flow,
        "usage": usage,
        "execution": execution,
        "outputs": [{"port": item["root_port"], "type_id": item["type_id"], "slot": item["slot"]} for item in outputs],
        "resources": list(resources),
        "input_types": [{"port": port, "primitive": primitive} for port, primitive in inputs],
        "recovery_actions": list(recoveries),
        "effect_flow_partitions": list(effect_flow_partitions),
        "proof_source": "subflow_body_evidence" if subflow else "checked_component",
    }
    return {
        "case_id": case_id,
        "group": case_id.split("/")[1],
        "kind": "agentflow.conformance.computation-root-export.test-vector",
        "note": "Independent static root/export assertion vector; no runtime behavior is simulated.",
        "input": {
            "operation": "compose", "operator": "sequence", "include_workflow_closure": True,
            "children": [child], "root_child_nodes": [2],
            "parent_outputs": parent_outputs, "workflow_outputs": workflow_outputs,
            "nested_nodes": [], "scope_resources": [], "additional_resources": [],
            "export_overrides": export_overrides or {},
            **({"mutation": mutation} if mutation else {}),
            **({"expected_stage": expected_stage} if expected_stage else {}),
        },
    }


def independent_oracle(case: dict) -> dict:
    """Expected verdicts from declared fixture facts and mutation labels."""
    if case["input"].get("expected_stage") == "component_validation_rejected":
        return {"stage":"component_validation_rejected","checked_component_produced":False,"verdict":"requires_evidence"}
    mutation = case["input"].get("mutation")
    overrides = case["input"].get("export_overrides", {})
    root = case["input"]["children"][0]
    verdicts = {dimension: "proven" for dimension in DIMENSIONS}
    preflight = {name: "proven" for name in ["assertion-inputs", "exact-ir-export-pins", "proof-context", "compiler-semantics", "subflow-dependency-pins"]}
    static = "proven"
    if mutation in {"change_ir_digest", "change_export_digest"}:
        preflight["exact-ir-export-pins"] = "violation"
        verdicts = {dimension: "unproven" for dimension in DIMENSIONS}
        static = "violation"
    elif mutation == "change_relation_content":
        preflight["proof-context"] = "unproven"
        verdicts = {dimension: "unproven" for dimension in DIMENSIONS}
        static = "unproven"
    elif mutation == "change_compiler_semantics":
        preflight["compiler-semantics"] = "unproven"
        verdicts = {dimension: "unproven" for dimension in DIMENSIONS}
        static = "unproven"
    elif mutation == "unsort_mappings":
        preflight["assertion-inputs"] = "violation"
        verdicts = {dimension: "unproven" for dimension in DIMENSIONS}
        static = "violation"
    elif mutation == "delete_input_mapping":
        verdicts["I"] = "unproven"
        static = "unproven"
    elif mutation == "duplicate_input_source":
        verdicts["I"] = "violation"
        static = "violation"
    elif mutation in {"delete_output_mapping", "change_output_type"}:
        verdicts["O"] = "unproven"
        static = verdicts["O"]
    elif mutation in {"change_resource_slot", "change_resource_type"}:
        verdicts["R"] = "unproven"
        static = "unproven"
    elif mutation == "change_effect_qualifier":
        verdicts["epsilon"] = "unproven"
        static = "unproven"
    elif mutation == "delete_flow_mapping":
        verdicts["F"] = "unproven"
        static = "unproven"
    elif mutation == "delete_recovery_mapping":
        verdicts["K"] = "unproven"
        static = "unproven"
    elif mutation == "delete_recovery_profile":
        verdicts["K"] = "unproven"
        static = "unproven"
    elif mutation == "delete_execution_mapping":
        verdicts["X"] = "unproven"
        static = "unproven"
    elif mutation == "change_execution_requirement":
        verdicts["X"] = "unproven"
        static = "unproven"
    elif mutation == "narrow_action_guard":
        verdicts["epsilon"] = "unproven"
        static = "unproven"

    flow = case["input"]["children"][0].get("flow", {})
    if flow.get("influence_unknown") or "unknown_external" in flow.get("sources", []):
        verdicts["F"] = "unproven"
        if static == "proven": static = "unproven"
    if case["input"]["children"][0].get("recovery_actions"):
        verdicts["F"] = "unproven"
        verdicts["X"] = "unproven"
        if static == "proven": static = "unproven"

    if overrides.get("remove_input_ports"):
        verdicts["I"] = "violation"
        static = "violation"
    if overrides.get("add_input_ports"):
        verdicts["I"] = "violation"
        static = "violation"
    if overrides.get("input_types"):
        verdicts["I"] = "violation"
        static = "violation"
    if overrides.get("remove_output_ports"):
        verdicts["O"] = "unproven"
        static = "unproven"
    if overrides.get("output_types"):
        verdicts["O"] = "violation"
        static = "violation"
    if overrides.get("effects"):
        root_upper = max((entry.get("upper", 0) for entry in case["input"]["children"][0]["effects"].get("entries", [])), default=0)
        export_upper = max((entry.get("upper", 0) for entry in overrides["effects"].get("entries", [])), default=0)
        if root_upper > export_upper or root_upper == "unbounded" and export_upper != "unbounded":
            verdicts["epsilon"] = "violation"
            static = "violation"
        elif case["input"]["children"][0]["effects"] != overrides["effects"]:
            # Guard mismatch and unsupported cross-row changes are exact E proof obligations.
            verdicts["epsilon"] = "violation"
            static = "violation"
            if case["input"]["children"][0].get("recovery_actions"):
                verdicts["K"] = "violation"
    if overrides.get("failures") is not None and set(case["input"]["children"][0].get("failures", [])) - set(overrides["failures"]):
        verdicts["Phi"] = "violation"
        static = "violation"
    if overrides.get("usage"):
        source = case["input"]["children"][0].get("usage", {}).get("coordinates", {})
        export = overrides["usage"].get("coordinates", {})
        if any(isinstance(value, int) and isinstance(export.get(key), int) and value > export[key] for key, value in source.items()):
            verdicts["U"] = "violation"
            static = "violation"
    if overrides.get("recovery_usage_upper") is not None:
        verdicts["K"] = "unproven"
    if overrides.get("execution", {}).get("sandbox") == "strict":
        verdicts["X"] = "violation"
        static = "violation"
    if (root.get("execution") or {}).get("sandbox") == "strict" and overrides.get("execution") == {}:
        verdicts["X"] = "violation"
        static = "violation"
    resources = root.get("resources", [])
    aliases = [item.get("resource_slot") for item in resources]
    if len(aliases) != len(set(aliases)) and len({item.get("component_port") for item in resources}) > 1:
        verdicts["R"] = "unproven"
        if static == "proven": static = "unproven"
    for source_resource in resources:
        source_attributes = source_resource.get("attributes", [])
        export_resource = next((item for item in overrides.get("resources", []) if item.get("port") == source_resource.get("component_port")), None)
        if source_attributes and export_resource is not None and not export_resource.get("attributes", []):
            verdicts["R"] = "violation"
            static = "violation"
    if overrides.get("effect_flow_partitions") == [] and root.get("effect_flow_partitions"):
        verdicts["F"] = "violation"
        static = "violation"
    if overrides.get("effect_flow_partitions") and overrides.get("effect_flow_partitions") != root.get("effect_flow_partitions"):
        verdicts["F"] = "violation"
        static = "violation"

    if static == "proven" and any(value != "proven" for value in verdicts.values()):
        static = "violation" if "violation" in verdicts.values() else "unproven"
    if "violation" in verdicts.values() and static != "violation":
        static = "violation"

    port_numbers = {item["port"] for item in root.get("input_types", [])}
    for source in root.get("flow", {}).get("sources", []):
        if isinstance(source, str) and source.startswith("input:"):
            port_numbers.add(int(source.split(":", 1)[1]))
    for effect in root.get("effects", {}).get("entries", []):
        if isinstance(effect.get("guard"), dict):
            port_numbers.add(effect["guard"]["input_equals"]["port"])
    effect_ports = {entry["atom"]["resource_selector"]["requirement_port_no"] for entry in root.get("effects", {}).get("entries", [])}
    effect_ports |= {resource["component_port"] for resource in root.get("resources", [])}
    outputs = root.get("outputs", [])
    child_flow_count = len(root.get("flow", {}).get("sources", [])) + len(root.get("flow", {}).get("constraints", []))
    mapping_counts = {
        "inputs": len(port_numbers), "outputs": len(case["input"].get("workflow_outputs", [])),
        "resources": len(effect_ports), "effects": len(root.get("effects", {}).get("entries", [])),
        "flow": child_flow_count, "recovery": len(root.get("recovery_actions", [])),
        "execution": 1, "subflows": int(root.get("proof_source") == "subflow_body_evidence"),
    }
    if mutation == "delete_input_mapping": mapping_counts["inputs"] = max(0, mapping_counts["inputs"] - 1)
    if mutation == "delete_output_mapping": mapping_counts["outputs"] = max(0, mapping_counts["outputs"] - 1)
    if mutation == "delete_flow_mapping": mapping_counts["flow"] = 0
    if mutation == "delete_recovery_mapping": mapping_counts["recovery"] = max(0, mapping_counts["recovery"] - 1)
    if mutation == "delete_execution_mapping": mapping_counts["execution"] = 0
    if mutation == "delete_recovery_mapping": mapping_counts["recovery"] = 0
    dimensions = []
    for dimension in DIMENSIONS:
        count_key = {"I":"inputs","O":"outputs","Phi":None,"epsilon":"effects","R":"resources","F":"flow","U":None,"K":"recovery","X":"execution"}[dimension]
        preflight_failed = any(value != "proven" for value in preflight.values())
        mapped = mapping_counts.get(count_key, 0) if count_key else (4 if dimension == "U" and not preflight_failed else 0)
        dimensions.append({"dimension":dimension,"mapped_coordinate_count":mapped,"proof":{"verdict":verdicts[dimension],"rule_id":RULES[dimension]}})
    preflight_rows = [{"dimension":name,"verdict":verdict,"rule_id":{
        "assertion-inputs":"AF-SEMANTIC-0159","exact-ir-export-pins":"AF-SEMANTIC-0159",
        "proof-context":"AF-SEMANTIC-0160","compiler-semantics":"AF-SEMANTIC-0161",
        "subflow-dependency-pins":"AF-SEMANTIC-0171"}[name]} for name, verdict in preflight.items()]
    if root.get("proof_source") == "subflow_body_evidence":
        preflight_rows.append({"dimension":"subflow-body-evidence","verdict":"requires_evidence","rule_id":"AF-SEMANTIC-0171"})
    specs = [{"id":spec_id,"path":path,"content_sha256":hashlib.sha256((ROOT/path).read_bytes()).hexdigest(),"content_bytes":len((ROOT/path).read_bytes())} for spec_id,path in SPECS]
    preflight_failed = any(value != "proven" for value in preflight.values())
    execution_children = []
    if not preflight_failed and mutation != "delete_execution_mapping":
        execution = root.get("execution")
        if overrides.get("execution", {}).get("sandbox") == "strict" and execution is None:
            execution_children = [{"source_node":2,"feasibility":"proven","security":"violation"}]
        elif execution and execution.get("sandbox") == "strict" and overrides.get("execution") == {}:
            execution_children = [{"source_node":2,"feasibility":"violation","security":"proven"}]
        else:
            execution_children = [{"source_node":2,"feasibility":"proven","security":"proven"}]
    expected = {
            "assertion_identity": {
            "semantic_ir_digest":{"algorithm":"sha256","domain":"semantic-ir"},
            "export_digest":{"algorithm":"sha256","domain":"component-contract"},
            "compiler_semantics_version":"agentflow.root-export-checker@1-stale" if mutation == "change_compiler_semantics" else "agentflow.root-export-checker@1",
            "specifications":specs,
            "rules":sorted([f"AF-CONTRACT-{number:04d}" for number in range(1,23)] + [f"AF-SEMANTIC-{number:04d}" for number in range(111,173)]),
            "mapping_counts":mapping_counts,
            "manifest":{"module_count":16,"schema_count":206 + len({item["primitive"] for item in root.get("input_types",[])} | {item["primitive"] for item in overrides.get("input_types",[])}),"effect_catalogs":[{"id":"agentflow.effects@1","operations":["write"]}],"resource_type_count":1,"flow_label_count":len(set(root.get("flow",{}).get("taints",[]))),"flow_sink_count":1,"usage_meter_count":1,"subflow_target_count":mapping_counts["subflows"]},
        },
        "preflight":preflight_rows,
        "dimensions":dimensions,
        "execution_children":execution_children,
        "static_compatibility":static,
        "checked_static_compatibility_capability_issued":static == "proven",
        "body_conformance":"requires_evidence",
        "implementation_conformance":"requires_evidence",
        "scope_authority_attenuation":"unproven",
        "scope_budget_attenuation":"unproven",
        "runtime_authority":"not_established",
        "runtime_authority_evidence":"requires_evidence",
        "publisher_authentication":"not_established",
    }
    if case["case_id"] == "computation-root-export/positive/full-nine-dimension-witnesses":
        core_type = {"specification_id":"agentflow.semantic.core@1","type_name":"TypeSchemaRef","schema_digest":{"algorithm":"sha256","domain":"type-schema","hex":"e1739609320518db9f8e00714ca67e9a79fcd5735a5eb8e8dae15911f15558e4"}}
        int64_alias = {"specification_id":"agentflow.test.guard-types@1","type_name":"GuardInt64","schema_digest":{"algorithm":"sha256","domain":"type-schema","hex":"395382f76ef8d9ac6c59fd873ee35ace1e321ad999259b228ac1521eb362b9ac"}}
        atom = effect_atom("natural")
        expected["assertion_identity"]["typed_mappings"] = {
            "inputs":[{"export_port":{"port":1},"run_parameter":1,"root_value_slot":11,"root_type":f"schema:agentflow.test.guard-types@1/GuardInt64@{int64_alias['schema_digest']['hex']}","export_type":int64_alias,"use_sites":[{"computation":2,"source_port":1,"expression":11,"type":int64_alias,"dependencies":[11]}]}],
            "outputs":[{"export_port":{"port":1},"name":"result","expression":500,"root_type":f"schema:agentflow.semantic.core@1/TypeSchemaRef@{core_type['schema_digest']['hex']}","producer_type":core_type,"dependencies":[31]}],
            "resources":[{"export_port":{"port":1},"root_slot":1,"root_type":core_type,"export_type":core_type,"source_coordinates":[{"node":2,"component_port":1,"resource_slot":1,"alias_component_ports":[1]}]}],
            "effects":[{"source":{"node":2,"ordinal":0,"atom":atom,"guard":"unconditional"},"root_resource_slot":1,"export_atom":atom,"mapped_guard":"unconditional"}],
            "flow":[],"recovery":[],
            "execution":[{"node":2,"requirements":{"capabilities":[],"quantities":[],"sandbox":"strong","trust_domain":None,"region_residency":None,"runtime_profile":None,"implementation_abis":[],"soft_preferences":[]}}],
            "subflows":[],
        }
        expected["assertion_identity"]["manifest"]["relation_snapshots"] = [
            {"schema":"agentflow.effect.resource.compatibility@1","revision":"fixture-effect-resource-r1","checksum":"1c9bda755f8814f9618828aa9159144ad24a665753904cdf750e072843e1579e","content_bytes":547},
            {"schema":"agentflow.flow.execution.relation@1","revision":"fixture-flow-execution-r1","checksum":"ca3eb2f6e33882ae14e3ac3899286c61a55290152a155a55e3e6a2a29a68e849","content_bytes":869},
        ]
        expected["dimensions"][0]["mapped_coordinates"] = [{"kind":"input","export_port":1,"run_parameter":1,"root_value_slot":11,"root_type":expected["assertion_identity"]["typed_mappings"]["inputs"][0]["root_type"],"export_type":int64_alias}]
        expected["dimensions"][1]["mapped_coordinates"] = [{"kind":"output","export_port":1,"name":"result","expression":500,"root_type":f"schema:agentflow.semantic.core@1/TypeSchemaRef@{core_type['schema_digest']['hex']}"}]
        expected["dimensions"][3]["mapped_coordinates"] = [{"kind":"effect","node":2,"ordinal":0,"atom":atom,"export_atom":atom,"guard":"unconditional","mapped_guard":"unconditional"}]
        expected["dimensions"][4]["mapped_coordinates"] = [{"kind":"resource","export_port":1,"root_slot":1,"root_type":core_type,"export_type":core_type,"source_coordinates":[{"node":2,"component_port":1,"resource_slot":1,"alias_component_ports":[1]}]}]
        expected["dimensions"][8]["mapped_coordinates"] = [{"kind":"execution","node":2,"requirements":{"capabilities":[],"quantities":[],"sandbox":"strong","trust_domain":None,"region_residency":None,"runtime_profile":None,"implementation_abis":[],"soft_preferences":[]}}]
    if root.get("recovery_actions"):
        action = root["recovery_actions"][0]
        identity = {"forward_effect":action["forward_atom"],"action_kind":"compensation" if action["action"] == "compensate" else "reconciliation","action_effect":action["action_atom"]}
        binding = {
            "action":identity,
            "activations_per_forward":action["activations_per_forward"],
            "attempts_per_activation":action["attempts_per_activation"],
            "per_attempt_usage":{"limits":[{"meter":{"catalog":"agentflow.meters@1","name":"api_calls"},"phase":"recovery","upper":action["per_attempt_usage"]["coordinates"]["recovery/api_calls"]}]},
        }
        if mutation == "delete_recovery_profile": binding["per_attempt_usage"]["limits"] = []
        expected["assertion_identity"]["typed_mappings"] = {
            "recovery":[{"source":{"node":2,"forward_ordinal":0,"action":identity},"export_action":identity,"usage_binding":binding}]
        }
        expected["dimensions"][7]["mapped_coordinates"] = [{"kind":"recovery","node":2,"forward_ordinal":0,"export_action":identity,"usage_binding":binding}]
        if mutation == "delete_recovery_mapping":
            expected["assertion_identity"]["typed_mappings"]["recovery"] = []
            expected["dimensions"][7]["mapped_coordinates"] = []
    return expected


def cases() -> list[dict]:
    result = []
    result.append(base_case("computation-root-export/positive/exact-empty-static-envelope"))
    result.append(base_case(
        "computation-root-export/positive/exact-input-output",
        inputs=[(1,"int64")],
        outputs=[{"root_port":1,"export_port":1,"name":"result","expression":500,"slot":"output:30","parent_slot":"output:31","type_id":"semantic-type:1"}],
    ))
    result.append(base_case(
        "computation-root-export/positive/full-nine-dimension-witnesses",
        inputs=[(1,"int64")],
        outputs=[{"root_port":1,"export_port":1,"name":"result","expression":500,"slot":"output:30","parent_slot":"output:31","type_id":"semantic-type:1"}],
        resources=[{"component_port":1,"resource_slot":"db"}],
        effects={"row":"closed","entries":[effect_entry(effect_atom("natural"),1)]},
        execution={"sandbox":"strict"},
    ))
    result.append(base_case("computation-root-export/negative/input-map-missing", inputs=[(1,"int64")], mutation="delete_input_mapping"))
    result.append(base_case("computation-root-export/negative/export-input-without-root-source", export_overrides={"add_input_ports":[{"port":1,"type_id":"semantic-type:1"}]}))
    result.append(base_case("computation-root-export/negative/input-source-alias", inputs=[(1,"int64"),(2,"string")], mutation="duplicate_input_source"))
    result.append(base_case("computation-root-export/negative/input-extra-root-parameter", inputs=[(1,"int64")], export_overrides={"remove_input_ports":[1]}))
    result.append(base_case("computation-root-export/negative/input-nominal-type", inputs=[(1,"int64")], export_overrides={"input_types":[{"port":1,"primitive":"string"}]}))
    result.append(base_case("computation-root-export/negative/output-map-missing", outputs=[{"root_port":1,"export_port":1,"name":"result","expression":500,"slot":"output:30","parent_slot":"output:31","type_id":"semantic-type:1"}], mutation="delete_output_mapping"))
    result.append(base_case("computation-root-export/negative/output-type-incompatible", outputs=[{"root_port":1,"export_port":1,"name":"result","expression":500,"slot":"output:30","parent_slot":"output:31","type_id":"semantic-type:1"}], export_overrides={"output_types":[{"port":1,"type_id":"semantic-type:2"}]}))
    result.append(base_case("computation-root-export/positive/exact-resource-map", resources=[{"component_port":1,"resource_slot":"db"}]))
    result.append(base_case("computation-root-export/negative/resource-slot-map-change", resources=[{"component_port":1,"resource_slot":"db"}], mutation="change_resource_slot"))
    result.append(base_case("computation-root-export/negative/resource-type-map-change", resources=[{"component_port":1,"resource_slot":"db"}], mutation="change_resource_type"))
    result.append(base_case("computation-root-export/negative/resource-slot-alias-without-public-rule", resources=[{"component_port":1,"resource_slot":"db"},{"component_port":2,"resource_slot":"db"}]))
    result.append(base_case("computation-root-export/negative/resource-attribute-strengthening", resources=[{"component_port":1,"resource_slot":"db","attributes":[{"key":"region","value":"us"}]}], export_overrides={"resources":[{"port":1,"attributes":[]}]}))
    result.append(base_case("computation-root-export/positive/exact-effect-map", effects={"row":"closed","entries":[effect_entry(effect_atom("natural"),2)]}))
    result.append(base_case("computation-root-export/negative/effect-upper-underdeclared", effects={"row":"closed","entries":[effect_entry(effect_atom("natural"),2)]}, export_overrides={"effects":{"row":"closed","entries":[effect_entry(effect_atom("natural"),1)]}}))
    result.append(base_case("computation-root-export/negative/effect-qualifier-map-change", effects={"row":"closed","entries":[effect_entry(effect_atom("natural"),2)]}, mutation="change_effect_qualifier"))
    result.append(base_case("computation-root-export/positive/exact-root-recovery", effects={"row":"closed","entries":[effect_entry(effect_atom("natural"),1),effect_entry(effect_atom("none"),1)]}, usage={"coordinates":{"forward/api_calls":1,"recovery/api_calls":1}}, recoveries=[recovery_action()]))
    result.append(base_case("computation-root-export/negative/recovery-map-missing", effects={"row":"closed","entries":[effect_entry(effect_atom("natural"),1),effect_entry(effect_atom("none"),1)]}, usage={"coordinates":{"forward/api_calls":1,"recovery/api_calls":1}}, recoveries=[recovery_action()], mutation="delete_recovery_mapping"))
    result.append(base_case("computation-root-export/negative/recovery-profile-missing", effects={"row":"closed","entries":[effect_entry(effect_atom("natural"),1),effect_entry(effect_atom("none"),1)]}, usage={"coordinates":{"forward/api_calls":1,"recovery/api_calls":1}}, recoveries=[recovery_action()], mutation="delete_recovery_profile"))
    result.append(base_case("computation-root-export/negative/recovery-usage-underdeclared", effects={"row":"closed","entries":[effect_entry(effect_atom("natural"),1),effect_entry(effect_atom("none"),1)]}, usage={"coordinates":{"forward/api_calls":1,"recovery/api_calls":2}}, recoveries=[recovery_action(per_attempt=2)], export_overrides={"usage":{"coordinates":{"forward/api_calls":1,"recovery/api_calls":1}},"recovery_usage_upper":1}))
    result.append(base_case("computation-root-export/negative/failure-outside-phi", failures=["FailureA"], export_overrides={"failures":[]}))
    result.append(base_case("computation-root-export/negative/flow-map-missing", inputs=[(1,"int64")], flow={"authority":"none","constraints":[],"influence_unknown":False,"sources":["input:1"],"taints":[]}, mutation="delete_flow_mapping"))
    flow_partition = {"operation":{"catalog":"agentflow.effects@1","name":"write"},"kind":"data_egress","allowed_effects":[effect_atom("natural")]}
    explicit_flow = {"authority":"none","constraints":[{"kind":"data_egress","sink":{"external_effect":{"catalog":"agentflow.effects@1","name":"write"}}}],"influence_unknown":False,"sources":["input:1"],"taints":[]}
    result.append(base_case("computation-root-export/positive/exact-flow-source-sink-partition", inputs=[(1,"int64")], effects={"row":"closed","entries":[effect_entry(effect_atom("natural"),1)]}, flow=explicit_flow, effect_flow_partitions=[flow_partition]))
    result.append(base_case("computation-root-export/negative/flow-partition-membership-mismatch", inputs=[(1,"int64")], effects={"row":"closed","entries":[effect_entry(effect_atom("natural"),1),effect_entry(effect_atom("none"),1)]}, flow=explicit_flow, effect_flow_partitions=[flow_partition], export_overrides={"effect_flow_partitions":[{"operation":{"catalog":"agentflow.effects@1","name":"write"},"kind":"data_egress","allowed_effects":[effect_atom("none")]}]}))
    result.append(base_case("computation-root-export/negative/unknown-flow-preserved", flow={"authority":"none","constraints":[],"influence_unknown":True,"sources":["unknown_external"],"taints":[]}, expected_stage="component_validation_rejected"))
    result.append(base_case("computation-root-export/negative/execution-security", export_overrides={"execution":{"sandbox":"strict"}}))
    result.append(base_case("computation-root-export/negative/execution-feasibility", execution={"sandbox":"strict"}, export_overrides={"execution":{}}))
    result.append(base_case("computation-root-export/context/relation-content-change", mutation="change_relation_content"))
    result.append(base_case("computation-root-export/context/compiler-semantics-change", mutation="change_compiler_semantics"))
    result.append(base_case("computation-root-export/context/ir-digest-change", mutation="change_ir_digest"))
    result.append(base_case("computation-root-export/context/export-digest-change", mutation="change_export_digest"))
    result.append(base_case("computation-root-export/subflow/body-evidence-absent", subflow=True))
    result.append(base_case("computation-root-export/recovery/action-guard-outside-export", inputs=[(1,"int64")], effects={"row":"closed","entries":[effect_entry(effect_atom("natural"),1,{"input_equals":{"port":1,"value":0}}),effect_entry(effect_atom("none"),1,{"input_equals":{"port":1,"value":0}})]}, usage={"coordinates":{"forward/api_calls":1,"recovery/api_calls":1}}, recoveries=[recovery_action()], export_overrides={"effects":{"row":"closed","entries":[effect_entry(effect_atom("natural"),1,{"input_equals":{"port":1,"value":1}}),effect_entry(effect_atom("none"),1,{"input_equals":{"port":1,"value":1}})]}}))
    design_vectors = {
        "X01":"computation-root-export/positive/full-nine-dimension-witnesses",
        "X02":"computation-root-export/negative/export-input-without-root-source",
        "X03":"computation-root-export/negative/input-extra-root-parameter",
        "X04":"computation-root-export/negative/input-source-alias",
        "X05":"computation-root-export/negative/input-nominal-type",
        "X06":"computation-root-export/positive/exact-input-output",
        "X07":"computation-root-export/negative/output-map-missing",
        "X08":"computation-root-export/negative/output-type-incompatible",
        "X09":"computation-root-export/positive/exact-resource-map",
        "X10":"computation-root-export/negative/resource-slot-map-change",
        "X11":"computation-root-export/negative/resource-slot-alias-without-public-rule",
        "X12":"computation-root-export/negative/effect-upper-underdeclared",
        "X13":"computation-root-export/negative/failure-outside-phi",
        "X14":"computation-root-export/negative/recovery-usage-underdeclared",
        "X15":"computation-root-export/recovery/action-guard-outside-export",
        "X16":"computation-root-export/negative/flow-partition-membership-mismatch",
        "X17":"computation-root-export/negative/unknown-flow-preserved",
        "X18":"computation-root-export/negative/execution-security",
        "X19":"computation-root-export/context/relation-content-change",
        "X20":"computation-root-export/context/compiler-semantics-change",
        "X21":"computation-root-export/subflow/body-evidence-absent",
        "X22":"computation-root-export/negative/recovery-profile-missing",
    }
    for case in result:
        case["design_vectors"] = [vector for vector, case_id in design_vectors.items() if case["case_id"] == case_id]
    return result


def write_case(case: dict) -> None:
    case["expected"] = independent_oracle(case)
    case["oracle"] = {"model":"independent-declarative-root-export-v1","inputs":"case input + frozen typed relation directions","not_from_production":True}
    group = case["group"]
    slug = case["case_id"].split("/")[-1]
    path = OUT / group / f"{slug}.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    assert_historical_output(path, json.dumps(case, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")


def main() -> None:
    generated = cases()
    ids = [case["case_id"] for case in generated]
    assert len(ids) == len(set(ids))
    for case in generated:
        write_case(case)
    manifest = {
        "kind":"agentflow.conformance.computation-root-export.test-corpus",
        "corpus_revision":1,
        "case_count":len(ids),
        "case_ids":sorted(ids),
        "stage_distribution":{
            "component_validation_rejected":sum(case["input"].get("expected_stage") == "component_validation_rejected" for case in generated),
            "root_export_checker":sum(case["input"].get("expected_stage") != "component_validation_rejected" for case in generated),
        },
        "design_vectors":{vector:case_id for vector,case_id in sorted({
            vector:case_id for case in generated for vector in case.get("design_vectors", [])
            for case_id in [case["case_id"]]
        }.items())},
        "format_note":"non-wire static RootExportAssertion vectors; no RootContractDigest or runtime claim",
    }
    assert_historical_output(OUT / "manifest.json", json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    print(f"generated {len(ids)} root-export cases")


if __name__ == "__main__":
    main()
