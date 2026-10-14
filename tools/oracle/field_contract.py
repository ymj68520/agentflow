"""Closed conformance field grammar. Schema comes from reviewed Rust types.

Value expectations come from independent declarations/algebra elsewhere. This
grammar validates both sides before comparison, including unvisited variants.
No unspecified object key or untyped array element is accepted.
"""

from __future__ import annotations

import copy
import json
import re

DEFS = {}


def ref(name):
    return {"kind": "ref", "name": name}


def arr(item, order="exact"):
    return {"kind": "array", "element": item, "ordering": order}


def opt(item):
    return {"kind": "nullable", "value": item, "presence": "required_key_explicit_null"}


def lit(value):
    return {"kind": "literal", "value": value}


def enum(*choices):
    return {"kind": "enum", "values": list(choices)}


def rec(**fields):
    return {"kind": "record", "fields": fields, "additional_fields": False}


def define(definition_name, **fields):
    DEFS[definition_name] = rec(**fields)
    return ref(definition_name)


def union(*choices):
    return {"kind": "union", "alternatives": list(choices)}


S = {"kind": "string"}
U = {"kind": "u64"}
I = {"kind": "i64"}
B = {"kind": "bool"}
N = {"kind": "null"}
V = enum("proven", "unproven", "violation", "requires_evidence")
D = {"kind": "string", "classification": "DiagnosticOnly", "reason": "Human-readable proof explanation has no nominal identity; verdict/rule/reference/prerequisites remain required."}

define("Digest", algorithm=lit("sha256"), domain=S, hex={"kind":"string","pattern":"^[0-9a-f]{64}$"})
define("SchemaRef", specification_id=S, type_name=S, schema_digest=ref("Digest"))
define("Named", catalog=S, name=S)
define("Attribute", key=S, value=S)
define("Checksum", algorithm=lit("sha256"), hex=S)
define("RelationPin", schema=S, revision=S, canonical_content_checksum=ref("Checksum"))
define("ProofContext", same_as_root=lit(True), relation_catalog_pins=arr(ref("RelationPin")))
define("Proof", dimension=S, verdict=V, rule_id=S, source_node=U, reference=opt(S), reason=D, prerequisites=arr(S))
define("LeafProof", dimension=S, verdict=V, rule_id=S, reference=opt(S), reason=D, prerequisites=arr(S))
define("Obligation", source_node=U, dimension=S, verdict=V, rule_id=S, reference=opt(S), reason=D)
P = arr(ref("Proof"), "semantic_multiset")
define("Upper", **{})
DEFS["Upper"] = union(rec(finite=U), rec(unbounded=N))
define("Bounds", lower=U, upper=ref("Upper"))
DEFS["Primitive"] = union(*[rec(**{key: schema}) for key, schema in [
    ("unit",N),("null",N),("bool",B),("int64",I),("decimal",S),("string",S),("bytes",S),("timestamp",I),("duration",I)]])
DEFS["Guard"] = union(rec(unconditional=lit(True)), rec(input_equals=rec(port_no=U, expected=ref("Primitive"))))
define("Selector", requirement_port_no=U, scope_bindings=arr(ref("Attribute")))
define("Qualifier", family=lit("idempotency"), member=enum("natural","none"))
define("Atom", operation=ref("Named"), resource_selector=ref("Selector"), qualifiers=arr(ref("Qualifier")))
define("EffectEntry", atom=ref("Atom"), bounds=ref("Bounds"), guard=ref("Guard"))
define("EffectResidual", catalog=S, selector=ref("Selector"), bounds=ref("Bounds"))
DEFS["Effect"] = union(rec(row=lit("closed"),entries=arr(ref("EffectEntry"))),rec(row=lit("open"),known=arr(ref("EffectEntry")),residual=arr(ref("EffectResidual"))),rec(row=lit("unknown")))
define("Family", control_node=U, kind=enum("iterate","retry"), logical_repetitions=ref("Bounds"), physical_attempts=opt(ref("Bounds")))
define("Occurrence", caller_semantic_ir_digest=ref("Digest"),source_contract_digest=ref("Digest"),source_component=ref("Named"),computation=U,source_entry_ordinal=U,declared=ref("EffectEntry"),mapped_resource_slot=opt(U),control_families=arr(ref("Family")))
define("ResidualOccurrence",caller_semantic_ir_digest=ref("Digest"),source_contract_digest=ref("Digest"),source_component=ref("Named"),computation=U,source_entry_ordinal=U,residual=ref("EffectResidual"),mapped_resource_slot=opt(U))
DEFS["Target"] = union(rec(kind=lit("component"),identity=ref("Named"),contract_digest=ref("Digest")),rec(kind=lit("subflow"),semantic_ir_digest=ref("Digest"),exported_contract_digest=opt(ref("Digest"))),rec(kind=lit("await_signal"),signal_type=ref("SchemaRef")))
define("ResourceType",schema=ref("SchemaRef"))
DEFS["ResourceType"] = ref("SchemaRef")
define("Requirement",port_no=U,resource_type=ref("ResourceType"),required_attributes=arr(ref("Attribute")))
define("ResourceMapping",component_port_no=U,resource_slot=U,required_type=ref("SchemaRef"),required_attributes_proven=B)
define("ResourceOccurrence",source_node=U,target=ref("Target"),requirement=opt(ref("Requirement")),mapping=ref("ResourceMapping"))
define("ResourceAlias",resource_slot=U,occurrences=arr(ref("ResourceOccurrence")))
DEFS["FlowSource"] = union(rec(workflow_input=U),rec(resource_requirement=U),rec(unknown_external=N))
DEFS["FlowSink"] = union(rec(workflow_output=U),rec(external_effect=ref("Named")))
define("FlowConstraint",kind=enum("data_egress","decision_influence"),sink=ref("FlowSink"))
define("Information",sources=arr(ref("FlowSource")),taints=arr(ref("Named")),instruction_authority=enum("none","external_data","delegated_agent","user","workflow","platform"),influence_unknown=B)
define("FlowContract",confidentiality_restrictions=arr(ref("Named")),summary=ref("Information"),flow_constraints=arr(ref("FlowConstraint")))
DEFS["SourceBinding"] = union(rec(input_expression=U),rec(resource_slot=U),rec(unknown_external=N))
DEFS["SinkBinding"] = union(rec(output_value=U),rec(effect_occurrences=arr(U)))
define("FlowMapping",source=opt(ref("FlowSource")),sink=opt(ref("FlowConstraint")),source_binding=opt(ref("SourceBinding")),sink_binding=opt(ref("SinkBinding")),matched_effect_occurrences=arr(U))
define("FlowPartition",operation=ref("Named"),kind=S,allowed_effects=arr(ref("Atom")))
define("FlowSourceContract",source_node=U,target=ref("Target"),contract=ref("FlowContract"),source_mappings=arr(ref("FlowMapping")),effect_flow_partitions=arr(ref("FlowPartition")))
define("Dataflow",producer_child=U,producer_node=U,value_slot=U,consumer_child=U,consumer_node=U,expression=U)
define("Flow",known_may_influence=opt(ref("Information")),unresolved_children=arr(U),sources=arr(ref("FlowSourceContract")),sequence_dependencies=arr(ref("Dataflow")),select_decision_dependencies=arr(U),control_decision_dependencies=arr(U),control_decision_influence_unknown=B,unknown_external_children=arr(U))
DEFS["UsageBound"] = union(rec(state=lit("known_finite"),upper=U),rec(state=lit("known_unbounded")),rec(state=lit("unknown")))
define("UsageCoordinate",phase=enum("forward","recovery"),meter=ref("Named"),bound=ref("UsageBound"))
define("Usage",globally_unknown=B,coordinates=arr(ref("UsageCoordinate")))
define("UsageLimit",meter=ref("Named"),phase=enum("forward","recovery"),upper=ref("Upper"))
define("ActionIdentity",forward_effect=ref("Atom"),action_kind=enum("compensation","reconciliation"),action_effect=ref("Atom"))
define("UsageBinding",action=ref("ActionIdentity"),activations_per_forward=ref("Bounds"),attempts_per_activation=ref("Bounds"),per_attempt_usage=arr(ref("UsageLimit")))
define("KeyRequirement",stable_across_attempts=B,distinct_across_invocations=B,bound_to_operation=B,provider_honors=B)
DEFS["Compensation"] = union(rec(none=N),rec(required=ref("Named")))
define("RecoveryClause",effect=ref("Atom"),idempotency=S,key_requirement=opt(ref("KeyRequirement")),reconciliation=S,compensation=ref("Compensation"),unknown_effect_handling=S)
define("RecoverySource",source_node=U,target=ref("Target"),replay_semantics=S,effect_recovery_clauses=arr(ref("RecoveryClause")),action_bindings=arr(ref("ActionIdentity")),usage_bindings=arr(ref("UsageBinding")),effect_occurrences=arr(ref("Occurrence")),action_guard_compatibility=arr(ref("ActionGuard")))
define("ExecutionCapability",capability=ref("Named"),exact_version=S,required_attributes=arr(ref("Attribute")))
define("Execution",capabilities=arr(ref("ExecutionCapability")),quantities=arr(rec(kind_no=U,minimum=U)),sandbox=opt(rec(minimum=enum("basic","strong"))),trust_domain=opt(ref("Named")),region_residency=opt(rec(execution_regions=arr(ref("Named")),residency_regions=arr(ref("Named")))),runtime_profile=opt(rec(family=S,profile=ref("Named"),required_features=arr(S))),implementation_abis=arr(ref("SchemaRef")),soft_preferences=arr(U))
define("ExecutionSource",source_node=U,target=ref("Target"),requirements=ref("Execution"))


def dimension(name, payload, key="value", proofs=P):
    contribution = rec(source_node=U, **{key: payload})
    choices = [rec(state=lit("complete"), **{key:payload},proofs=proofs),
               rec(state=lit("partial"),**{key:opt(payload)},unresolved_sources=arr(U),proofs=proofs),
               rec(state=lit("unknown"),known_contributions=arr(contribution),unresolved_sources=arr(U),proofs=proofs),
               rec(state=lit("per_child"),contributions=arr(contribution),unresolved_sources=arr(U),proofs=proofs),
               rec(state=lit("not_applicable"),proofs=proofs)]
    DEFS[name] = union(*choices)
    return ref(name)


dimension("FailureDimension",arr(ref("SchemaRef")))
dimension("EffectDimension",ref("Effect"))
dimension("ResourceDimension",arr(ref("ResourceOccurrence")),"occurrences")
dimension("FlowDimension",ref("Flow"))
dimension("UsageDimension",ref("Usage"))
dimension("RecoveryDimension",arr(ref("RecoverySource")),"sources")
dimension("ExecutionDimension",arr(ref("ExecutionSource")),"sources")
define("OutputSource",child_node=U,child_port=U,child_value_slot=U)
define("OutputPort",parent_port=U,parent_value_slot=U,local_type_id=U,sources=arr(ref("OutputSource")))
dimension("OutputDimension",arr(ref("OutputPort")),"ports",None)
for alt in DEFS["OutputDimension"]["alternatives"]:
    del alt["fields"]["proofs"]
define("Interface",outputs=ref("OutputDimension"),select_decision=opt(rec(expression=U,dependencies=arr(U),case_ordinals=arr(U),exact_case=opt(U),totality=V)))
define("Scope",control_node=U,child=U,parent_visible_resources=arr(U),declared_resources=arr(U),child_visible_resources=arr(U),resource_attenuation=V,authority_attenuation=V,budget_attenuation=V,owned_children_settle_before_terminal=V)
define("MultiplicityIdentity",caller_semantic_ir_digest=ref("Digest"),control_node=U,kind=enum("iterate","retry"))
define("Multiplicity",identity=ref("MultiplicityIdentity"),logical_repetitions=ref("Bounds"),physical_attempts=opt(ref("Bounds")))
DEFS["ActionUpper"] = union(rec(state=lit("known"),upper=ref("Upper")),*[rec(state=lit(tag)) for tag in ["overflow","invalid_multiplicity","unknown"]])
define("LocalBound",compensation_scope_node=U,upper=ref("ActionUpper"),applied_families=arr(ref("Multiplicity")))
define("RootBound",root_semantic_ir_digest=ref("Digest"),upper=ref("ActionUpper"),applied_families=arr(ref("Multiplicity")))
define("ActionSource",caller_semantic_ir_digest=ref("Digest"),source_node=U,source_contract_digest=ref("Digest"),forward_entry_ordinal=U,action=ref("ActionIdentity"))
define("GuardCoordinate",source_contract_digest=ref("Digest"),component_input_port=U,input_expression=U,exact_type=opt(ref("SchemaRef")),dependency_slots=arr(U))
define("GuardAssignment",port_no=U,value=ref("Primitive"),**{"type":ref("SchemaRef")})
define("Countermodel",source_component=ref("Named"),recovery_action=ref("ActionIdentity"),forward_entry_ordinal=U,action_entry_ordinal=U,forward_guard=ref("Guard"),declared_action_guard=ref("Guard"),forward_bounds=ref("Bounds"),activation_upper=ref("Upper"),attempt_upper=ref("Upper"),action_upper=ref("Upper"),assignments=arr(ref("GuardAssignment")),reachability=S,related_rule_id=S)
define("ActionGuard",source_component=ref("Named"),recovery_action=ref("ActionIdentity"),forward_entry_ordinal=U,action_entry_ordinal=U,forward_guard=opt(ref("Guard")),declared_action_guard=opt(ref("Guard")),action_upper=opt(ref("Upper")),verdict=V,basis=S,reason=D,countermodel=opt(ref("Countermodel")))
DEFS["PathDetail"] = union(rec(),rec(case_ordinal=U,selector=opt(U),selector_dependencies=arr(U),exact_case=opt(U)),rec(branch_ordinal=U),rec(max_attempts=U),rec(ordinal=U))
define("ControlPath",parent_node=U,child_node=U,child_index=U,kind=S,detail=ref("PathDetail"),reachability=enum("possible","exact_selected","proven_unreachable"))
DEFS["RecoveryOwner"] = union(rec(kind=lit("compensation_scope"),node=U),rec(kind=lit("root_recovery")))
define("RootAction",source=ref("ActionSource"),forward_seed_upper=ref("Upper"),target=ref("Target"),owner=ref("RecoveryOwner"),local_bound=opt(ref("LocalBound")),root_bound=ref("RootBound"),forward_occurrence=ref("Occurrence"),action_effect_occurrence=ref("Occurrence"),recovery_usage_binding=ref("UsageBinding"),local_recovery_usage=opt(ref("Usage")),root_recovery_usage=ref("Usage"),guard_compatibility=opt(ref("ActionGuard")),guard_coordinates=arr(ref("GuardCoordinate")),eligibility_evidence=V,control_path=arr(ref("ControlPath")))
dimension("RootActions",arr(ref("RootAction")),"actions")
define("SourceClosure",source_nodes_accounted=V,action_bindings_accounted=V,usage_profiles_accounted=V,multiplicity_factors_accounted=V,root_action_totals_sound=V)
define("RootRecovery",actions=ref("RootActions"),action_effects=ref("EffectDimension"),epsilon_coverage_requirement=ref("EffectDimension"),recovery_usage=ref("UsageDimension"),source_closure=ref("SourceClosure"),proof_dimensions=P,overall_verdict=V)
define("PotentialAction",source_node=U,identity=ref("ActionIdentity"),forward_occurrence=ref("Occurrence"),action_effect_occurrence=ref("Occurrence"),local_bound=ref("LocalBound"),potential_effect=ref("EffectEntry"),potential_recovery_usage=ref("Usage"),receipt_evidence=V)
define("LocalAction",source=ref("ActionSource"),forward_seed_upper=ref("Upper"),owner_scope=U,action_kind=S,local_bound=ref("LocalBound"),forward_occurrence=ref("Occurrence"),action_effect_occurrence=ref("Occurrence"),recovery_usage_binding=ref("UsageBinding"),recovery_usage=ref("Usage"),guard_compatibility=opt(ref("ActionGuard")),eligibility_evidence=V)
dimension("PotentialActions",arr(ref("PotentialAction")),"actions")
dimension("LocalActions",arr(ref("LocalAction")),"actions")
define("CompensationScope",potential_actions=ref("PotentialActions"),local_recovery_actions=ref("LocalActions"),compensation_order=V,recovery_blocked_outcome=V,forward_effects_retained=B,forward_outcome_retained=B)
DEFS["PortBinding"] = union(rec(input_expression=U),rec(output_value=U))
define("PortMapping",contract_port_no=U,binding=ref("PortBinding"),exact_type=opt(ref("SchemaRef")))
define("LeafDimensions",failures=arr(ref("SchemaRef")),effects=ref("Effect"),resources=arr(ref("Requirement")),information_flow=ref("FlowContract"),usage=arr(ref("UsageLimit")),recovery=rec(replay_semantics=S,effect_recovery_clauses=arr(ref("RecoveryClause"))),execution=ref("Execution"),effect_flow_partitions=arr(ref("FlowPartition")),recovery_action_bindings=arr(ref("ActionIdentity")),recovery_usage_bindings=arr(ref("UsageBinding")))
define("Await",signal_type=ref("SchemaRef"),correlation=U,output_bindings=arr({"kind":"tuple","elements":[U,U]}),timeout=lit("no_implicit_timeout"),interaction=lit("contract_unavailable"),provenance=lit("unknown_external_influence"),authority=lit("not_inferred"))
define("Leaf",computation=U,caller_semantic_ir_digest=ref("Digest"),kind=enum("invoke","await","call_subflow"),target=opt(ref("Target")),input_mappings=arr(ref("PortMapping")),output_mappings=arr(ref("PortMapping")),resource_mappings=arr(ref("ResourceMapping")),resource_aliases=arr(rec(resource_slot=U,component_ports=arr(U))),effect_occurrences=arr(ref("Occurrence")),effect_residuals=arr(ref("ResidualOccurrence")),flow_mappings=arr(ref("FlowMapping")),dimensions=opt(ref("LeafDimensions")),action_guard_compatibility=arr(ref("ActionGuard")),await_summary=opt(ref("Await")),proof=arr(ref("LeafProof"),"semantic_multiset"),proof_context=ref("ProofContext"))
DEFS["OperatorSemantics"] = union(rec(kind=S),rec(mode=enum("all","fail_fast"),max_inflight=opt(U)),rec(mode=union(rec(kind=lit("while"),max_iterations=U),rec(kind=lit("foreach"),statically_known_item_count=opt(U),max_inflight=opt(U))),effect_repetitions=ref("Bounds"),usage_repetitions=opt(U)),rec(max_attempts=U,delay_micros_in_order=arr(I)),rec(expression=U,static_duration_micros=opt(I),possible_outcomes=arr(enum("child_completed","timed_out"))),rec(handled_failures=arr(ref("SchemaRef")),clauses=arr(rec(ordinal=U,matcher=union(rec(failure=ref("SchemaRef")),rec(timed_out=lit(True))),handler=U))),rec(opcode=S))
define("Node",source_semantic_ir_digest=ref("Digest"),computation=U,operator=enum("invoke","await","call_subflow","sequence","select","parallel","race","iterate","retry","timeout","handle","scope","compensation_scope","unsupported"),operator_semantics=ref("OperatorSemantics"),direct_children=arr(U),children=arr(ref("Node")),leaf_summary=opt(ref("Leaf")),interface=ref("Interface"),effects=ref("EffectDimension"),failures=ref("FailureDimension"),resources=ref("ResourceDimension"),information_flow=ref("FlowDimension"),usage=ref("UsageDimension"),recovery_sources=ref("RecoveryDimension"),execution_sources=ref("ExecutionDimension"),scope_boundary=opt(ref("Scope")),compensation_scope=opt(ref("CompensationScope")),effect_occurrences=arr(ref("Occurrence")),effect_residuals=arr(ref("ResidualOccurrence")),resource_aliases=arr(ref("ResourceAlias")),effect_lower_bound_adjustments=arr(rec(source_node=U,reason=enum("sequence_continuation_can_be_skipped","fail_fast_sibling_can_be_cancelled","race_branch_may_lose_or_be_cancelled","select_may_have_no_covered_case"))),sequence_dataflow_dependencies=arr(ref("Dataflow")),inherited_obligations=arr(ref("Obligation"),"semantic_multiset"),proof=P,proof_context=ref("ProofContext"))
define("ResolverTarget",semantic_ir_digest=ref("Digest"),direct_dependencies=opt(arr(ref("Digest"))),exported_contract_digest=opt(ref("Digest")))
define("ResolverIdentity",component_contract_digests=arr(ref("Digest")),subflow_targets=arr(ref("ResolverTarget")))
define("Closure",computation_order=arr(U),leaf_order=arr(U),known_effect_occurrences_accounted=B,known_effect_residuals_accounted=B,known_recovery_sources_accounted=B,root_export_compatibility=lit("not_expressed_in_semantic_ir"),proof=P)
define("RootDimensions",failure=ref("FailureDimension"),effect=ref("EffectDimension"),resource=ref("ResourceDimension"),information_flow=ref("FlowDimension"),usage=ref("UsageDimension"),recovery_sources=ref("RecoveryDimension"),execution_sources=ref("ExecutionDimension"))
define("Root",source_semantic_ir_digest=ref("Digest"),overall_verdict=V,root_control=ref("Node"),closure=ref("Closure"),root_dimensions=ref("RootDimensions"),effect_occurrences=arr(ref("Occurrence")),effect_residuals=arr(ref("ResidualOccurrence")),resource_aliases=arr(ref("ResourceAlias")),effect_lower_bound_adjustments=arr(rec(source_node=U,reason=S)),sequence_dataflow_dependencies=arr(ref("Dataflow")),inherited_obligations=arr(ref("Obligation"),"semantic_multiset"),proof=P,resolver_identity=ref("ResolverIdentity"),proof_context=ref("ProofContext"))
define("ContextRelation",schema=ref("SchemaRef"),revision=S,canonical_content_checksum=ref("Checksum"),canonical_content=arr(U))
define("Context",context_id=U,modules=arr(rec(specification_id=S,named_types=arr(ref("SchemaRef")))),type_schemas=arr(ref("SchemaRef")),effect_catalogs=arr(rec(specification_id=S,operations=arr(S))),resource_schemas=arr(ref("SchemaRef")),resource_descriptors=arr(rec(schema=ref("SchemaRef"),recognized_attributes=arr(S))),confidentiality_labels=arr(ref("Named")),taints=arr(ref("Named")),flow_effect_sinks=arr(rec(operation=ref("Named"),decision_capable=B)),usage_meters=arr(rec(meter=ref("Named"),unit=ref("Named"),aggregation=enum("additive","unsupported"))),recovery_effect_catalogs=arr(rec(specification_id=S,operations=arr(S))),reconciliation_operations=arr(ref("Named")),execution_capabilities=arr(rec(capability=ref("Named"),exact_version=S,supported_versions=arr(S),attributes=arr(ref("Attribute")))),trust_domains=arr(ref("Named")),regions=arr(ref("Named")),region_compatibility=arr(rec(**{"from":ref("Named"),"to":ref("Named")})),runtime_profiles=arr(rec(profile=ref("Named"),family=S,features=arr(S))),effect_resource_relation=opt(ref("ContextRelation")),recovery_usage_relation=opt(ref("ContextRelation")),flow_execution_relation=opt(ref("ContextRelation")))
define("D9Facts",case_id=S,input_identity=rec(canonical_bytes=U,canonical_sha256=S),production_stage=lit("D10"),context=ref("Context"),d9_root=ref("Root"),d9_root_recovery=ref("RootRecovery"))
define("EarlyRejection",case_id=S,input_identity=rec(canonical_bytes=U,canonical_sha256=S),production_stage=lit("D6"),error_family=lit("ContractValidationFailed"),error_path=lit("$.contract_assets"),d9_root=N,capability_issued=lit(False))

# D10 schemas are inventoried and shape-checkable; every D10 VALUE obligation
# is Pending for this phase. They do not acquire an independent verdict here.
define("ExportPort",contract_digest=ref("Digest"),port_no=U)
define("InputUse",computation=U,source_contract_digest=ref("Digest"),source_component_port=U,input_expression=U,exact_type=ref("SchemaRef"),dependency_slots=arr(U))
define("InputMapping",export_input_port=ref("ExportPort"),run_parameter_id=U,root_value_slot=U,root_structural_type=S,export_type=ref("SchemaRef"),use_sites=arr(ref("InputUse")))
define("OutputMapping",export_output_port=ref("ExportPort"),workflow_output_name=S,workflow_output_expression=U,root_structural_type=S,producer_evidence=rec(exact_result_type=ref("SchemaRef"),dependency_slots=arr(U)))
define("ResourceSourceCoordinate",source_node=U,source_contract_digest=ref("Digest"),component_resource_port=U,resource_slot=U,alias_component_ports=arr(U))
define("ResourceCoordinate",export_resource_port=ref("ExportPort"),root_resource_slot=U,root_resource_type=ref("SchemaRef"),export_resource_type=ref("ResourceType"),source_coordinates=arr(ref("ResourceSourceCoordinate")))
define("EffectIdentity",caller_semantic_ir_digest=ref("Digest"),source_node=U,source_contract_digest=ref("Digest"),source_entry_ordinal=U,atom=ref("Atom"),guard=ref("Guard"))
define("EffectCoordinate",source=ref("EffectIdentity"),root_resource_slot=opt(U),export_atom=ref("Atom"),export_guard=ref("Guard"))
define("FlowCoordinate",source_node=U,source_contract_digest=ref("Digest"),flow_mapping_ordinal=U,source=opt(ref("FlowSource")),sink=opt(ref("FlowConstraint")),source_binding=opt(ref("SourceBinding")),sink_binding=opt(ref("SinkBinding")),export_source=opt(ref("FlowSource")),export_sink=opt(ref("FlowConstraint")),source_effect_partition=opt(ref("FlowPartition")),export_effect_partition=opt(ref("FlowPartition")))
define("RecoveryCoordinate",source=ref("ActionSource"),export_action_identity=ref("ActionIdentity"),exact_usage_binding=ref("UsageBinding"))
define("ExecutionCoordinate",source_node=U,source_contract_digest=ref("Digest"),exact_execution_requirements=ref("Execution"))
define("CompilerIdentity",analysis_semantics_version=S,registry_content_bytes=U,registry_content_sha256=S,rule_registry_content_bytes=U,rule_registry_content_sha256=S,specifications=arr(rec(specification_id=S,logical_path=S,content_sha256=S,content_bytes=U)),rules=arr(rec(rule_id=S,normative_source=S,source_anchor=S,short_statement=S)))
define("ContextManifest",compiled_module_count=U,type_schema_count=U,effect_catalogs=arr(rec(specification_id=S,operations=arr(S))),resource_type_count=U,flow_label_count=U,flow_sink_count=U,usage_meter_count=U,recovery_operation_count=U,relation_snapshots=arr(rec(schema=S,revision=S,canonical_content_checksum=ref("Checksum"),content_bytes=U)),resolver_dependencies=rec(component_contract_digests=arr(ref("Digest")),subflow_targets=arr(rec(semantic_ir_digest=ref("Digest"),direct_dependencies=arr(ref("Digest")),exported_contract_digest=opt(ref("Digest"))))))
define("SubflowProof",call_site=U,target_semantic_ir_digest=ref("Digest"),target_exported_contract_digest=opt(ref("Digest")),target_root_assertion_identity=opt(union(ref("Assertion"),rec(projection_depth_limit=lit(True)))),transitive_targets=arr(rec(semantic_ir_digest=ref("Digest"),exported_contract_digest=opt(ref("Digest")))))
define("Assertion",semantic_ir_digest=ref("Digest"),exported_component_contract_digest=ref("Digest"),compiler_semantics=ref("CompilerIdentity"),proof_context_manifest=ref("ContextManifest"),input_mappings=arr(ref("InputMapping")),output_mappings=arr(ref("OutputMapping")),resource_mappings=arr(ref("ResourceCoordinate")),effect_mappings=arr(ref("EffectCoordinate")),flow_mappings=arr(ref("FlowCoordinate")),recovery_mappings=arr(ref("RecoveryCoordinate")),execution_mappings=arr(ref("ExecutionCoordinate")),subflow_dependencies=arr(ref("SubflowProof")))
DEFS["TypedWitness"] = union(*[rec(kind=lit(kind),mapping=ref(name)) for kind,name in [("input","InputMapping"),("output","OutputMapping"),("resource","ResourceCoordinate"),("effect","EffectCoordinate"),("flow","FlowCoordinate"),("recovery","RecoveryCoordinate"),("execution","ExecutionCoordinate")]])
define("SourceReference",semantic_ir_digest=ref("Digest"),computation=opt(U),contract_digest=opt(ref("Digest")),source_ordinal=opt(U))
define("DimensionProof",dimension=enum("I","O","Phi","epsilon","R","F","U","K","X"),proof=ref("LeafProof"),source_facts=arr(ref("SourceReference")),mapped_coordinates=arr(ref("TypedWitness")),mapped_coordinate_count=U)
define("RootExportReport",stage=lit("root_export_checker"),assertion_identity=ref("Assertion"),preflight=arr(ref("LeafProof")),dimensions=arr(ref("DimensionProof")),execution_children=arr(rec(source_node=U,source_contract_digest=ref("Digest"),feasibility=enum("proven","unproven","violation"),security=enum("proven","unproven","violation"))),static_compatibility=V,checked_static_compatibility_capability_issued=B,body_conformance=V,implementation_conformance=V,scope_authority_attenuation=V,scope_budget_attenuation=V,runtime_authority=lit("not_established"),runtime_authority_evidence=V,publisher_authentication=lit("not_established"),unresolved_obligations=arr(S))

# Id/port coordinates are nominal U32 values; occurrence ceilings, Usage and
# work/count indices remain U64/usize. Preserve the namespace in schema metadata.
_ID_FIELDS={"computation","source_node","parent_node","child_node","control_node","handler","parent_port","child_port","port_no","requirement_port_no","component_port_no","resource_slot","root_resource_slot","root_value_slot","parent_value_slot","child_value_slot","value_slot","local_type_id","expression","input_expression","workflow_output_expression","run_parameter_id","source_component_port","component_resource_port","component_input_port","case_ordinal","branch_ordinal","ordinal","child","context_id","max_iterations","max_inflight","exact_case","selector","correlation","kind_no","producer_child","producer_node","consumer_child","consumer_node","owner_scope","compensation_scope_node","call_site","workflow_input","workflow_output","resource_requirement","output_value"}
_ID_ARRAYS={"direct_children","computation_order","leaf_order","unresolved_sources","unresolved_children","unknown_external_children","select_decision_dependencies","control_decision_dependencies","dependency_slots","dependencies","case_ordinals","parent_visible_resources","declared_resources","child_visible_resources","component_ports","alias_component_ports","selector_dependencies"}


def _numeric_widths(schema,field_name=""):
    value=copy.deepcopy(schema)
    kind=value["kind"]
    if kind=="u64" and field_name in _ID_FIELDS | _ID_ARRAYS:
        return {"kind":"u32","nominal_coordinate":field_name}
    if kind=="record":
        value["fields"]={key:_numeric_widths(child,key) for key,child in value["fields"].items()}
    elif kind=="nullable":value["value"]=_numeric_widths(value["value"],field_name)
    elif kind=="array":value["element"]=_numeric_widths(value["element"],field_name if field_name in _ID_ARRAYS else "")
    elif kind=="union":value["alternatives"]=[_numeric_widths(child,field_name) for child in value["alternatives"]]
    elif kind=="tuple":value["elements"]=[_numeric_widths(child,field_name) for child in value["elements"]]
    return value


DEFS={name:_numeric_widths(schema) for name,schema in DEFS.items()}
DEFS["Await"]["fields"]["output_bindings"]["element"]["elements"]=[{"kind":"u32","nominal_coordinate":"port_no"},{"kind":"u32","nominal_coordinate":"ValueSlotId"}]
DEFS["OperatorSemantics"]["alternatives"][4]["fields"]["possible_outcomes"]["exact_length"]=2
DEFS["RootExportReport"]["fields"]["dimensions"]["exact_length"]=9
DEFS["RootExportReport"]["fields"]["dimensions"]["unique_by"]="dimension"
DEFS["Node"]["constraints"]=["owned_children","leaf_identity"]
DEFS["ContextRelation"]["fields"]["canonical_content"]["element"]={"kind":"u8"}


def schema_fields():
    """Finite fields of every declared projection definition/variant, not samples."""
    def walk(schema,path,stage):
        kind=schema["kind"]
        if kind=="ref":
            return
        if kind=="record":
            for key,field in schema["fields"].items():
                yield {"field_path":path+"."+key,"schema":copy.deepcopy(field),"classification":field.get("classification","RequiredSemantic"),"stage":stage}
                yield from walk(field,path+"."+key,stage)
        elif kind in {"array","nullable"}:
            yield from walk(schema["element" if kind=="array" else "value"],path+("[]" if kind=="array" else "?"),stage)
        elif kind=="union":
            for index,alternative in enumerate(schema["alternatives"]):
                yield from walk(alternative,path+f"::variant[{index}]",stage)
        elif kind=="tuple":
            for index,field in enumerate(schema["elements"]):
                yield from walk(field,path+f"[{index}]",stage)
    d10=set()
    def reachable(name,seen):
        if name in seen:return
        seen.add(name)
        def visit(schema):
            if schema["kind"]=="ref":reachable(schema["name"],seen)
            elif schema["kind"]=="record":
                for child in schema["fields"].values():visit(child)
            elif schema["kind"]=="union":
                for child in schema["alternatives"]:visit(child)
            elif schema["kind"]=="array":visit(schema["element"])
            elif schema["kind"]=="nullable":visit(schema["value"])
            elif schema["kind"]=="tuple":
                for child in schema["elements"]:visit(child)
        visit(DEFS[name])
    d9=set();reachable("D9Facts",d9);reachable("RootExportReport",d10)
    for name,schema in sorted(DEFS.items()):
        stage="D9" if name in d9 else "D10"
        yield from walk(schema,name,stage)


def instantiated_slots(value, schema, name="D9Facts", output=None):
    """Applicable type slots determined from independent Expected, never Actual."""
    if output is None:output=set()
    kind=schema["kind"]
    if kind=="ref":return instantiated_slots(value,DEFS[schema["name"]],schema["name"],output)
    if kind=="union":
        for index,alternative in enumerate(schema["alternatives"]):
            try:validate(value,alternative)
            except CoverageError:continue
            return instantiated_slots(value,alternative,name+f"::variant[{index}]",output)
        raise CoverageError("missing variant")
    if kind=="record":
        for key,field in schema["fields"].items():
            output.add(name+"."+key)
            instantiated_slots(value[key],field,name+"."+key,output)
    elif kind=="array":
        for item in value:instantiated_slots(item,schema["element"],name+"[]",output)
    elif kind=="nullable" and value is not None:
        instantiated_slots(value,schema["value"],name+"?",output)
    elif kind=="tuple":
        for index,(item,field) in enumerate(zip(value,schema["elements"])):instantiated_slots(item,field,name+f"[{index}]",output)
    return output


def instance_provenance(value,schema,path="$",counts=None):
    """Same typed visit accounting as validate, with explicit origin families."""
    if counts is None:counts={"IndependentAlgebra":0,"ExactDeclarations":0,"ContentIdentity":0,"DiagnosticOnly":0}
    kind=schema["kind"]
    if kind=="ref":return instance_provenance(value,DEFS[schema["name"]],path,counts)
    if kind=="union":
        for alternative in schema["alternatives"]:
            try:validate(value,alternative)
            except CoverageError:continue
            return instance_provenance(value,alternative,path,counts)
        raise CoverageError("unknown provenance variant")
    if schema.get("classification")=="DiagnosticOnly":origin="DiagnosticOnly"
    elif path.startswith("$.context") or any(token in path for token in ["schema_digest","contract_digest","semantic_ir_digest",".input_identity",".relation_catalog_pins"]):origin="ContentIdentity"
    elif ".leaf_summary" in path and ".proof" not in path or path in {"$.case_id","$.production_stage"} or path.endswith((".computation",".source_node",".operator",".direct_children")):origin="ExactDeclarations"
    else:origin="IndependentAlgebra"
    counts[origin]+=1
    if kind=="record":
        for field,child in schema["fields"].items():instance_provenance(value[field],child,path+"."+field,counts)
    elif kind=="array":
        for index,item in enumerate(value):instance_provenance(item,schema["element"],f"{path}[{index}]",counts)
    elif kind=="nullable" and value is not None:instance_provenance(value,schema["value"],path,counts)
    elif kind=="tuple":
        for index,(item,child) in enumerate(zip(value,schema["elements"])):instance_provenance(item,child,f"{path}[{index}]",counts)
    return counts


class CoverageError(ValueError):
    pass


def validate(value, schema, path="$", depth=0, count=None):
    if count is None:
        count = {"required":0,"diagnostic":0}
    if depth > 256:
        raise CoverageError(path + ": OracleResourceLimit")
    kind = schema["kind"]
    if kind == "ref":
        return validate(value, DEFS[schema["name"]], path, depth+1, count)
    if kind == "union":
        errors = []
        for alternative in schema["alternatives"]:
            trial = copy.deepcopy(count)
            try:
                validate(value, alternative, path, depth+1, trial)
                count.update(trial)
                return count
            except CoverageError as error:
                errors.append(str(error))
        raise CoverageError(path + ": wrong variant; " + " | ".join(errors))
    count["diagnostic" if schema.get("classification") == "DiagnosticOnly" else "required"] += 1
    if kind == "record":
        if not isinstance(value, dict):
            raise CoverageError(path + ": record required")
        for key in schema["fields"]:
            if key not in value:
                raise CoverageError(path + "." + key + ": missing required field")
        for key in value:
            if key not in schema["fields"]:
                raise CoverageError(path + "." + key + ": unexpected field")
        for key, field_schema in schema["fields"].items():
            validate(value[key], field_schema, path + "." + key, depth+1, count)
        if "owned_children" in schema.get("constraints",[]):
            if value["direct_children"] != [item["computation"] for item in value["children"]] or len(value["direct_children"])!=len(set(value["direct_children"])):
                raise CoverageError(path+".children: checked ownership identity/order mismatch")
        if "leaf_identity" in schema.get("constraints",[]):
            leaf=value["leaf_summary"]
            is_leaf=value["operator"] in {"invoke","await","call_subflow"}
            if is_leaf!=(leaf is not None) or is_leaf and (value["children"] or leaf["computation"]!=value["computation"] or leaf["kind"]!=value["operator"] or leaf["caller_semantic_ir_digest"]!=value["source_semantic_ir_digest"]):
                raise CoverageError(path+".leaf_summary: exact leaf identity/availability mismatch")
    elif kind == "array":
        if not isinstance(value, list):
            raise CoverageError(path + ": array required")
        if "exact_length" in schema and len(value)!=schema["exact_length"]:
            raise CoverageError(path+": exact typed array length required")
        if "unique_by" in schema and len({json.dumps(item[schema["unique_by"]],sort_keys=True) for item in value})!=len(value):
            raise CoverageError(path+": duplicated nominal member")
        for index, item in enumerate(value):
            validate(item, schema["element"], f"{path}[{index}]", depth+1, count)
    elif kind == "tuple":
        if not isinstance(value, list) or len(value) != len(schema["elements"]):
            raise CoverageError(path + ": exact tuple arity required")
        for index, item_schema in enumerate(schema["elements"]):
            validate(value[index], item_schema, f"{path}[{index}]", depth+1, count)
    elif kind == "nullable":
        if value is not None:
            validate(value, schema["value"], path, depth+1, count)
    elif kind == "literal":
        if type(value) is not type(schema["value"]) or value != schema["value"]:
            raise CoverageError(path + ": exact literal required")
    elif kind == "enum":
        if not isinstance(value, str) or value not in schema["values"]:
            raise CoverageError(path + ": exact enum member required")
    elif kind in {"u8","u32","u64","i64"}:
        lower, upper = {"u8":(0,255),"u32":(0,2**32-1),"u64":(0,2**64-1),"i64":(-2**63,2**63-1)}[kind]
        if type(value) is not int or not lower <= value <= upper:
            raise CoverageError(path + ": integer width/type violation")
    elif {"string":str,"bool":bool,"null":type(None)}.get(kind) is not type(value):
        raise CoverageError(path + ": scalar type mismatch")
    if kind=="string" and "pattern" in schema and re.fullmatch(schema["pattern"],value) is None:
        raise CoverageError(path+": canonical nominal string pattern mismatch")
    return count


def canonical(value, schema):
    kind = schema["kind"]
    if kind == "ref":
        return canonical(value, DEFS[schema["name"]])
    if kind == "union":
        for alternative in schema["alternatives"]:
            try:
                validate(value, alternative)
                return canonical(value, alternative)
            except CoverageError:
                pass
        raise CoverageError("no canonical variant")
    if schema.get("classification") == "DiagnosticOnly":
        return "DiagnosticOnly"
    if kind == "record":
        return {key:canonical(value[key],field_schema) for key,field_schema in schema["fields"].items()}
    if kind == "array":
        result = [canonical(item,schema["element"]) for item in value]
        return sorted(result,key=lambda item:json.dumps(item,sort_keys=True,separators=(",",":"))) if schema["ordering"] == "semantic_multiset" else result
    if kind == "nullable":
        return None if value is None else canonical(value,schema["value"])
    if kind == "tuple":
        return [canonical(item,item_schema) for item,item_schema in zip(value,schema["elements"])]
    return value


def compare(expected, actual, schema=ref("D9Facts")):
    left_count, right_count = validate(expected,schema), validate(actual,schema)
    left, right = canonical(expected,schema), canonical(actual,schema)
    def first(a,b,path):
        if type(a) is not type(b):
            return path
        if isinstance(a,dict):
            if a.keys()!=b.keys():
                return path+"."+sorted(a.keys() ^ b.keys())[0]
            for key in sorted(a):
                diff = first(a[key],b[key],path+"."+key)
                if diff:
                    return diff
        elif isinstance(a,list):
            if len(a) != len(b):
                return path+".length"
            for index,(x,y) in enumerate(zip(a,b)):
                diff = first(x,y,f"{path}[{index}]")
                if diff:
                    return diff
        elif a != b:
            return path
        return None
    difference = first(left,right,"$")
    if difference:
        raise CoverageError(difference + ": independent Expected/Actual mismatch")
    return {"expected":left_count,"actual":right_count,"compared":left_count["required"],"missing_expected":0,"missing_actual":0,"unexpected":0,"uncompared":0}
