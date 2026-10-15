"""Explicit non-wire R1 logical declarations. Authoring data, not Rust layout reflection."""
from closed_g01_model import parse_defs,enum,UNIT,ref
from substitution_registration_data import CASES,DIMENSIONS,LOCAL

SUPPLEMENT = r"""
DigestRef = {algorithm:Text,domain:Text,hex:Text}
RuleRef = {specification_id:SpecificationId,rule_id:RuleId,anchor:Text}
ModelRef = {owner:SpecificationId,name:Text}
ModelType = {owner:SpecificationId,name:Text}
RawClaim = {type:ClaimType,bytes:Bytes}
Compiler = {analysis_semantics_version:OwnerVersion,registry_content:Bytes,rule_registry_content:Bytes,specifications:Seq<SpecificationContentIdentity>,rules:Seq<RuleContentIdentity>,support_manifest:FullSupportManifest}
CompleteCompiler = Compiler
FullSupportManifest = {normative_sources:Set<SpecificationContentIdentity>,closed_tables:{hash_domains:Bytes,abi_numbers:Bytes,rule_areas:Bytes,afidl_primitives:Bytes},selected_specifications:Set<SpecificationId>,applied_rules:Set<RuleContentIdentity>,semantic_roles:Seq<agentflow.portable.proof.inputs@1/SemanticRoleEntry>,attribution:agentflow.portable.proof.inputs@1/AttributionProfile,report_shape:ModelEdition,obligation_edition:ObligationEdition}
CompilerClaim = legacy_v1 {analysis_semantics_version:OwnerVersion,registry_content:Bytes,rule_registry_content:Bytes,specifications:Seq<SpecificationContentIdentity>,rules:Seq<RuleContentIdentity>} | successor_v2:Compiler
CompiledModuleContents = {specification_id:SpecificationId,types:Seq<CanonicalTypeSchema>}
LogicalTypeIdentity = {specification_id:SpecificationId,type_name:Text,schema_digest:DigestRef}
ResourceTypeDescriptor = {specification_id:SpecificationId,type_name:Text,schema_hex:Text,recognized_attributes:Set<Text>}
ExecutionUniverseContents = {capabilities:Map<ExecutionCapabilityRef,{capability:ExecutionCapabilityRef,exact_version:Text,supported_versions:Set<Text>,attributes:Set<{key:Text,value:Text}>}>,trust_domains:Set<TrustDomainRef>,regions:Set<RegionRef>,region_compatibility:Set<{from:RegionRef,to:RegionRef}>,runtime_profiles:Map<RuntimeProfileRef,{profile:RuntimeProfileRef,family:Text,features:Set<Text>}>}
RelationSnapshotContent = {pin:{schema:TypeSchemaRef,revision:Text,canonical_content_checksum:{algorithm:Text,hex:Text}},canonical_content:Bytes}
ContractProofContextContents = {compiled_modules:Map<SpecificationId,CompiledModuleContents>,type_schemas:Map<LogicalTypeIdentity,CanonicalTypeSchema>,effect_catalogs:Map<SpecificationId,Set<Text>>,resource_types:{schemas:Map<LogicalTypeIdentity,CanonicalTypeSchema>,descriptors:Map<LogicalTypeIdentity,ResourceTypeDescriptor>},flow_catalogs:{confidentiality:Set<ConfidentialityLabelRef>,taints:Set<TaintRef>,sinks:Map<EffectOperationRef,Bool>},usage_meters:Map<UsageMeterRef,{meter:UsageMeterRef,unit:UsageUnitRef,aggregation:UsageAggregation}>,recovery_operations:{effects:Map<SpecificationId,Set<Text>>,reconciliation_operations:Set<{catalog:Text,name:Text}>},execution_universes:ExecutionUniverseContents,relation_snapshots:{effect_resource:Opt<RelationSnapshotContent>,recovery_usage:Opt<RelationSnapshotContent>,flow_execution:Opt<RelationSnapshotContent>}}
ResolverDependencyManifest = {component_contract_digests:Seq<DigestRef>,subflow_targets:Seq<{semantic_ir_digest:DigestRef,direct_dependencies:Seq<DigestRef>,exported_contract_digest:Opt<DigestRef>}>}
ResolverAssociations = Seq<{caller_ir:DigestRef,source_node:ComputationId,request:Request,component_contract:Opt<Node<CheckedContractContents>>,target_ir:Opt<Node<CheckedIrContents>>,target_export:ExportState,direct_dependencies:Set<DigestRef>,transitive_dependencies:Set<DigestRef>}>
Context = {compiled_modules:Map<SpecificationId,CompiledModuleContents>,type_schemas:Map<LogicalTypeIdentity,CanonicalTypeSchema>,effect_catalogs:Map<SpecificationId,Set<Text>>,resource_types:{schemas:Map<LogicalTypeIdentity,CanonicalTypeSchema>,descriptors:Map<LogicalTypeIdentity,ResourceTypeDescriptor>},flow_catalogs:{confidentiality:Set<ConfidentialityLabelRef>,taints:Set<TaintRef>,sinks:Map<EffectOperationRef,Bool>},usage_meters:Map<UsageMeterRef,{meter:UsageMeterRef,unit:UsageUnitRef,aggregation:UsageAggregation}>,recovery_operations:{effects:Map<SpecificationId,Set<Text>>,reconciliation_operations:Set<{catalog:Text,name:Text}>},execution_universes:ExecutionUniverseContents,relation_snapshots:Seq<RelationSnapshotContent>,resolver_dependencies:ResolverDependencyManifest}
RootProvenance = {source_ir:DigestRef,root_node:ComputationId,root_resolver_identity:Node<ResolverAssociations>,root_context:Node<Context>,root_k_context:Node<Context>,source_ir_contents:Node<CheckedIrContents>}
CheckedContractContents = {contract:ComponentContract,canonical_bytes:Bytes,pin:DigestRef}
ContractContents = CheckedContractContents
MultiplicityFactor = af-computation::AppliedMultiplicity
CheckedIrContents = {document:af-ir::SemanticIrDocument,canonical_bytes:Bytes,pin:DigestRef}
RawCandidateContract = Raw<ComponentContract>
AssertionGraph = Raw<af-computation::RootExportAssertion>
ContextPartition = Seq<NonemptySeq<ContextRole>>
ContextRolePartition = Seq<NonemptySeq<SubstitutionContextRole>>
PortIdentity = {owner:DigestRef,direction:PortDirection,number:PortNo}
TypedValue = {model_type:ModelType,value:CanonicalValueDag}
CanonicalValueDag = {nodes:Seq<DagNode>,roots:NonemptySeq<U64>}
DagNode = {type:ModelType,body:NodeBody}
NodeBody = scalar:ScalarValue | record:Seq<{field:ClosedField,value:U64}> | variant {case:ClosedCase,payload:U64} | sequence:Seq<U64> | set:Seq<U64> | option_none:unit | option_some:U64
ScalarValue = unit:unit | null:null | bool:Bool | int32:I32 | uint32:U32 | int64:I64 | uint64:U64 | decimal:af-core::Decimal | float64:af-core::Float64 | string:NfcText | bytes:Bytes | timestamp:TimestampMicros | duration:DurationMicros
ContractContextSet = {analysis:ContractProofContextContents,old_checked:ContractProofContextContents,replacement_checked:ContractProofContextContents}
OperandSubject = {role:OperandRole,contents:ContractContents,identity:ComponentIdentity,context_role:SubstitutionContextRole}
ApplicationDocument = {descriptor:ModelRef,owner:SpecificationId,applications:Seq<ApplicationDeclaration>}
ApplicationDeclaration = {name:NfcText,old:ContractContents,replacement:ContractContents,context_requirement:SameCheckedContext}
ApplicationOrigin = {document:ApplicationDocument,declaration_path:Seq<DeclaredStep>}
SubstitutionProofSubject = {owner_compiler:CompleteCompiler,old:OperandSubject,replacement:OperandSubject,contexts:ContractContextSet,associations:ContextRolePartition,application:ApplicationOrigin}
SubstitutionCoordinate = lineage {old:ComponentIdentity,replacement:ComponentIdentity} | io {old_inputs:Seq<af-contract::ComponentInputPort>,new_inputs:Seq<af-contract::ComponentInputPort>,old_outputs:Seq<af-contract::ComponentOutputPort>,new_outputs:Seq<af-contract::ComponentOutputPort>} | failure {old:FailureRow,replacement:FailureRow} | effects {old:EffectSignature,replacement:EffectSignature} | resources {old:ResourceRequirementRow,replacement:ResourceRequirementRow,old_namespace:ComponentIdentity,new_namespace:ComponentIdentity} | flow {old:InformationFlowContract,replacement:InformationFlowContract,old_partitions:Seq<EffectFlowPartition>,new_partitions:Seq<EffectFlowPartition>} | usage {old:UsageSignature,replacement:UsageSignature,old_profiles:Seq<RecoveryUsageBinding>,new_profiles:Seq<RecoveryUsageBinding>} | recovery {old:RecoveryContract,replacement:RecoveryContract,old_bindings:Seq<RecoveryActionBinding>,new_bindings:Seq<RecoveryActionBinding>} | execution {old:ExecutionRequirements,replacement:ExecutionRequirements,aspect:SubstitutionExecutionAspect} | whole_pair {old:ContractContents,replacement:ContractContents} | context_roles {analysis:ContractProofContextContents,old_checked:ContractProofContextContents,replacement_checked:ContractProofContextContents,association:ContextRolePartition} | operand_validation {operand:OperandSubject}
SubstitutionBasis = checked_origin {role:OperandRole,validation_stage:Stage,validator_facts:Set<FactKey>} | association {partition:ContextRolePartition} | body_equality {equal:Bool,old:ContractContents,replacement:ContractContents} | changed_pair_floor {whole_body_fact:FactKey,verdict:UnprovenOnly} | local_relation {case:LocalRelationCase,decision:ClosedRelationDecision} | reflexive_result {whole_body_fact:FactKey,checked_inputs:Set<FactKey>} | conservative_result {floor:FactKey,candidate:FactKey} | aspect_combination {feasibility:FactKey,security:FactKey,decision:ProofVerdict} | cross_fold {results:Map<SubstitutionDimension,FactKey>} | blocked_result {blockers:NonemptySet<FactKey>}
ClosedRelationDecision = proven:unit | unproven {basis:UnprovenBasis} | violation {witness:SubstitutionWitness}
UnprovenBasis = lineage_differs:unit | io_not_in_exact_subset:unit | effect_reason:EffectUnprovenReason | resource_reason:ResourceCompatibilityUnprovenReason | flow_reason:FlowCompatibilityUnprovenReason | partitions_differ:unit | usage_reason:UsageUnprovenReason | recovery_nonidentical:unit | recovery_bindings_differ:unit | execution_aspects:ExecutionCompatibility | dependency_error:DependencyErrorPayload
DependencyErrorPayload = {domain:DependencyErrorDomain,kind:DependencyErrorKind,detail:Text}
SubstitutionWitness = io_added_input {new_port:af-contract::ComponentInputPort,old_inputs:Seq<af-contract::ComponentInputPort>} | io_missing_output {old_port:af-contract::ComponentOutputPort,new_outputs:Seq<af-contract::ComponentOutputPort>} | failure_extra {new_member:FailureTypeRef,old_row:FailureRow} | resource_added_port {new_requirement:ResourceRequirement,old_row:ResourceRequirementRow} | resource_attribute_strengthened {port:PortNo,new_attribute:ResourceAttributeConstraint,old_attributes:Seq<ResourceAttributeConstraint>,type:TypeSchemaRef} | flow_dropped_restriction {label:ConfidentialityLabelRef,old:Set<ConfidentialityLabelRef>,replacement:Set<ConfidentialityLabelRef>} | flow_added_sink {kind:FlowKind,sink:FlowSink,old:Set<FlowSink>,replacement:Set<FlowSink>} | flow_authority_raised {old:InstructionAuthority,replacement:InstructionAuthority} | usage_ceiling_increase {phase:UsagePhase,meter:UsageMeterRef,unit:UsageUnitRef,aggregation:UsageAggregation,old:UsageUpperBound,replacement:UsageUpperBound} | recovery_clause_removed {old:Seq<EffectRecoveryClause>,replacement:Seq<EffectRecoveryClause>,removed:EffectRecoveryClause} | execution_constraint {aspect:SubstitutionExecutionAspect,field:ExecutionWitnessField,old:ExecutionRequirements,replacement:ExecutionRequirements,violating_coordinate:ExecutionWitnessCoordinate} | aggregate {violating_fact:FactKey,witness:NonaggregateSubstitutionWitness}
ExecutionWitnessCoordinate = capability {capability:ExecutionCapabilityRef} | quantity {kind:ComputeQuantityKind} | sandbox:unit | trust_domain:unit | execution_regions:unit | residency_regions:unit | runtime_profile:unit | implementation_abis:unit
StandaloneFragment = {shape:ModelEdition,profile:StandaloneProfile,scope:Scope,facts:Map<FactKey,Fact>,prerequisites:Map<PrerequisiteKey,Prerequisite>,obligations:Seq<Obligation>,identity_strings:Seq<Text>,blockers:Map<FactKey,NonemptySet<FactKey>>,contradictions:Set<TypedContradiction>}
"""

def declarations(legacy,error_kinds):
 d=parse_defs(legacy["core_type_definitions"]);d.update(parse_defs(SUPPLEMENT))
 for n,k in {"af-core::Decimal":"decimal","af-core::Float64":"finite_f64","af-core::Timestamp":"timestamp_i64","af-core::Duration":"duration_i64","TimestampMicros":"i64","DurationMicros":"i64"}.items():d[n]={"kind":k,"nominal":n}
 def en(n,values):d[n]=enum([(x,UNIT) for x in values])
 en("AuthorityStatus",["NotEstablished"])
 en("OperandRole",["old","replacement"]);en("SubstitutionContextRole",["analysis","old_checked","replacement_checked"])
 en("SameCheckedContext",["same_checked_context"]);en("StandaloneProfile",["standalone_substitution"]);en("UnprovenOnly",["Unproven"])
 en("SubstitutionDimension",DIMENSIONS);en("SubstitutionCase",CASES);en("LocalRelationCase",LOCAL)
 en("SubstitutionClause",[x for x in CASES if x!="context_association"])
 en("SubstitutionExecutionAspect",["combined","feasibility","security"])
 en("Stage",[f"D{i}" for i in range(11)]+["contract_validation","standalone_substitution"])
 en("PredicateFamily",list(legacy["closed_predicates"])+["standalone_substitution"])
 d["ClosedPredicateCase"]=enum([(family,enum([(case,UNIT) for case in cases])) for family,cases in list(legacy["closed_predicates"].items())+[("standalone_substitution",CASES)]])
 kinds="assertion_input exact_pin proof_context resolver_provenance compiler_identity subflow_dependency input_mapping output_producer failure_containment effect_containment resource_mapping information_flow usage_coverage recovery_coverage execution_source execution_feasibility execution_security body_conformance implementation_conformance scope_authority scope_budget runtime_authority publisher_authentication control_evidence prerequisite".split()
 en("Kind",kinds+["substitution_evidence"])
 d["Dimension"]=enum([("none",UNIT)]+[(x,UNIT) for x in legacy["closed_predicates"]["root_dimension"]]+[("supporting",ref("PredicateId")),("substitution",ref("SubstitutionDimension"))])
 en("DirectKind",kinds[:13]+kinds[14:23])
 en("RecoveryClause","source_closure action_binding usage_profile multiplicity guard_compatibility epsilon_coverage recovery_usage_coverage eligibility commit_order".split())
 en("ControlClause","resolved_leaf topology output_interface selector_totality effect_envelope failure_row resource_provenance information_flow usage retry_safety child_settlement forward_history compensation_coverage root_export_boundary unsupported_control recovery_blocked_outcome".split())
 en("PrerequisitePredicate",legacy["closed_predicates"]["prerequisite"][:18])
 d.update(parse_defs("Requirement = {clause:RequirementClause,evidence_class:EvidenceClass}\nRequirementClause = direct:DirectKind | recovery:RecoveryClause | control:ControlClause | prerequisite:PrerequisitePredicate | substitution:SubstitutionClause"))
 en("EvidenceClass","static_declaration typed_counterexample_resolution trusted_body implementation_attestation provider_receipt authority_attestation publisher_authentication control_terminal_state committed_effect_order recovery_outcome provider_honoring application_idempotency".split())
 d["ContextRole"]={"kind":"u32","nominal":"profile-specific ContextRole rank","allowed_by_scope":{"root_export":[0,1,2,3,4],"contract_validation":[0,1,2],"computation":[0,1,2,3,4]}}
 for name,vals in {"PinRole":"caller_ir root_ir export_contract component_contract subflow_ir subflow_export type_schema relation compiler","DigestRole":"semantic_ir component_contract type_schema","PortDirection":"input output resource","SourceRole":"component subflow await owned_child recovery_action execution_child","ClaimType":"pin identifier compiler context contract assertion_mapping","UsageCoverageRole":"forward_envelope recovery_envelope action_recovery_coverage","ExecutionAspect":"source_coverage feasibility security","ResolverDifference":"component_pins target_absent direct_dependencies target_export additional_targets association_identity","ExecutionWitnessField":"capability_version capability_attributes quantity sandbox trust_domain execution_regions residency_regions runtime_profile implementation_abis","ContextCatalogRole":"modules type_schemas effect_catalogs resource_schemas resource_descriptors flow_labels flow_taints flow_sinks usage_meters recovery_effects reconciliation_operations execution_capabilities trust_domains regions region_compatibility runtime_profiles effect_resource_relation recovery_usage_relation flow_execution_relation resolver"}.items():en(name,vals.split())
 for domain,rows in error_kinds.items():en(domain+"ErrorKind",[x["kind"] for x in rows])
 en("DependencyErrorDomain",list(error_kinds))
 d["DependencyErrorKind"]=enum([(domain,ref(domain+"ErrorKind")) for domain in error_kinds])
 # Parent amendments preserve old variant order and payload, append new cases.
 extra=parse_defs("ScopeSubject = standalone_substitution:SubstitutionProofSubject\nSource = contract_operand {role:OperandRole,subject:OperandSubject} | substitution_application {application:ApplicationOrigin,old:OperandSubject,replacement:OperandSubject}\nTarget = contract_operand {role:OperandRole,subject:OperandSubject} | substitution_pair {old:OperandSubject,replacement:OperandSubject}\nCoordinate = substitution {application:ApplicationOrigin,dimension:Opt<SubstitutionDimension>,clause:RequirementClause,comparison:SubstitutionCoordinate}\nEvidenceRef = contract_operand {role:OperandRole,subject:OperandSubject,member_path:Seq<MemberStep>} | application_declaration {application:ApplicationOrigin}\nTypedContradiction = substitution {witness:SubstitutionWitness}")
 for k,v in extra.items():d[k]["variants"].extend(v["variants"])
 d["Fact"]["fields"].append(["substitution_basis",{"_generic":"Opt","args":[ref("SubstitutionBasis")]}])
 d["Fragment"]["fields"][1][1]["variants"].append(["standalone_substitution",UNIT])
 d["ObservationOwner"]["variants"].append(["standalone_substitution",UNIT])
 d["NonaggregateSubstitutionWitness"]={"kind":"enum","variants":d["SubstitutionWitness"]["variants"][:-1]}
 return d
