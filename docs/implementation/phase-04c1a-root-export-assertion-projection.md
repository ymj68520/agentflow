# Phase 4C.1A — Complete Typed Assertion Projection

All new shapes are **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED ABI /
NOT IMPLEMENTED**. Existing imported shapes retain their frozen definitions.

## Assertion and current-field correspondence

~~~text
PortableProofContextManifest = record {
  context_id: ContextId; resolver_id: ResolverId;
}
PortableRootExportAssertion = record {
  exact_semantic_ir_digest: IrPin;
  exact_exported_component_contract_digest: ContractPin;
  compiler_semantics_id: CompilerManifestId;
  proof_context_manifest: PortableProofContextManifest;
  input_mappings: set<RootInputPortBinding, InputMappingKey>;
  output_mappings: set<RootOutputPortBinding, OutputMappingKey>;
  resource_mappings: set<RootResourcePortBinding, ResourceMappingKey>;
  effect_mappings: set<RootEffectCoordinateBinding, EffectMappingKey>;
  flow_mappings: set<RootFlowCoordinateBinding, FlowMappingKey>;
  recovery_mappings: set<RootRecoveryCoordinateBinding, RecoveryMappingKey>;
  execution_mappings: set<RootExecutionCoordinateBinding, ExecutionMappingKey>;
  subflow_dependencies: set<SubflowProofDependency, SubflowMappingKey>;
}
~~~

The compiler ID expands to the exact current CompilerSemanticsIdentity
projection. The context/resolver pair expands to current ProofContextManifest:
every base universe plus the freshly reconstructed resolver dependency
inventory. The two root pins are unchanged. Each existing semantic field is
therefore represented; no report or supplied verdict is added.

All caller-local IDs below are scoped by assertion.exact_semantic_ir_digest.
A source contract port is scoped by its exact source ContractPin; an exported
port is scoped by the exact exported ContractPin. Integers alone never create
a mapping.

## Input and output witnesses

~~~text
ComponentInputPortIdentity = record { exported_contract_digest: ContractPin; port_no: U32; }
ComponentOutputPortIdentity = record { exported_contract_digest: ContractPin; port_no: U32; }
ResourceRequirementPortIdentity = record { exported_contract_digest: ContractPin; port_no: U32; }
RootInputUseSite = record {
  computation: ComputationId; source_contract_digest: ContractPin;
  source_component_port: U32; input_expression: ExpressionId;
  exact_type: SchemaRef; dependency_slots: set<ValueSlotId, numeric>;
}
RootInputPortBinding = record {
  export_input_port: ComponentInputPortIdentity; run_parameter_id: RunParameterId;
  root_value_slot: ValueSlotId; root_type: StructuralTypeId;
  export_type: SchemaRef; use_sites: set<RootInputUseSite, InputUseSiteKey>;
}
RootOutputProducerEvidence = record {
  exact_result_type: SchemaRef; dependency_slots: set<ValueSlotId, numeric>;
}
RootOutputPortBinding = record {
  export_output_port: ComponentOutputPortIdentity; workflow_output_name: Text;
  workflow_output_expression: ExpressionId; root_result_type: StructuralTypeId;
  producer_evidence: RootOutputProducerEvidence;
}
~~~

Ports and all imported local IDs are positive uint32 at validation. Source
type, expression, producer and dependency witnesses are rederived by IR/Phase
3C. Complete inputs form the current exact bijection; multiple exported
independent inputs may not share one root parameter. Outputs require every
advertised producer/type and checked availability. Unknown/transformed input
identity or incomplete output witnesses stay Unproven, not guessed equal.

## Resource and effect coordinates

~~~text
RootResourceSourceCoordinate = record {
  source_node: ComputationId; source_contract_digest: ContractPin;
  component_resource_port: U32; resource_slot: ResourceSlotId;
  alias_component_ports: set<U32, numeric>;
}
RootResourcePortBinding = record {
  export_resource_port: ResourceRequirementPortIdentity;
  root_resource_slot: ResourceSlotId; root_resource_type: SchemaRef;
  export_resource_type: ResourceTypeRef;
  source_coordinates: set<RootResourceSourceCoordinate, ResourceSourceKey>;
}
RootEffectOccurrenceIdentity = record {
  caller_semantic_ir_digest: IrPin; source_node: ComputationId;
  source_contract_digest: ContractPin; source_entry_ordinal: Ordinal64;
  atom: EffectAtom; guard: EffectGuard;
}
RootEffectCoordinateBinding = record {
  source: RootEffectOccurrenceIdentity; root_resource_slot: option<ResourceSlotId>;
  export_atom: EffectAtom; export_guard: EffectGuard;
}
~~~

Resource aliases are only the exact source ports bound to the same checked IR
ResourceSlot. Decode rejects duplicate aliases; reconstruction compares the
complete fresh group. Scope visibility and attributes remain validator facts.
An equal ResourceType label with a different SchemaPin is not compatible.

Effect ordinal is zero-based into the exact checked source's canonical effect
entry vector. Its full atom and guard must agree with that entry. Native usize
is replaced by uint64 with checked bounds/conversion. Decoder cannot narrow
or synthesize a missing mapped slot. None is an explicit absent coordinate and
can lead to Unproven. Positive containment continues to use the same-row,
one-use-slot relation; no new guard language is introduced.

## Flow coordinates

~~~text
FlowSourceBinding = variant {
  input_expression: ExpressionId; resource_slot: ResourceSlotId; unknown_external: unit;
}
FlowSinkBinding = variant {
  output_value: ValueSlotId; effect_occurrences: set<Ordinal64, numeric>;
}
RootFlowCoordinateBinding = record {
  source_node: ComputationId; source_contract_digest: ContractPin;
  flow_mapping_ordinal: U64;
  source: option<FlowSourceRef>; sink: option<FlowConstraint>;
  source_binding: option<FlowSourceBinding>; sink_binding: option<FlowSinkBinding>;
  export_source: option<FlowSourceRef>; export_sink: option<FlowConstraint>;
  source_effect_partition: option<EffectFlowPartition>;
  export_effect_partition: option<EffectFlowPartition>;
}
~~~

The flow ordinal is into the freshly resolved leaf flow-mapping vector;
effect_occurrences indexes that leaf's occurrence vector, not a global root
vector. Required source/sink/partition combinations are those of the current
mapping and are checked afresh. None is not an empty source/partition; unknown
external influence stays explicit. All eight Option fields are preserved.
No source naming convention grants decision authority.

## Recovery and Execution witnesses

~~~text
RecoveryActionSourceIdentity = record {
  caller_semantic_ir_digest: IrPin; source_node: ComputationId;
  source_contract_digest: ContractPin; forward_entry_ordinal: Ordinal64;
  action: RecoveryActionIdentity;
}
RootRecoveryCoordinateBinding = record {
  source: RecoveryActionSourceIdentity; export_action_identity: RecoveryActionIdentity;
  exact_usage_binding: RecoveryUsageBinding;
}
RootExecutionCoordinateBinding = record {
  source_node: ComputationId; source_contract_digest: ContractPin;
  exact_execution_requirements: ExecutionRequirements;
}
~~~

Recovery action identity retains forward atom, kind and action atom; the usage
binding retains activation/attempt bounds and per-attempt meters. All source/
action/profile rows are compared with fresh RootRecovery. K coverage is an
obligation against already-derived root/export epsilon and U, not another
addend. Unsupported Recovery monotonicity remains Unproven.

Execution requires every freshly resolved child, exact profile data and
separate feasibility/security checks. No Worker or runtime qualification is
part of this declaration.

## Subflow identity projection

~~~text
SubflowTargetIdentity = record {
  semantic_ir_digest: IrPin; exported_contract_digest: option<ContractPin>;
}
SubflowProofDependency = record {
  call_site: ComputationId; target_semantic_ir_digest: IrPin;
  target_exported_contract_digest: option<ContractPin>;
  target_root_assertion_id: option<AssertionId>;
  transitive_targets: set<SubflowTargetIdentity, target_digest_key>;
}
~~~

The optional target assertion ID expands to the complete current boxed
RootExportAssertionIdentity tuple, not a digest. Target identities must match
the call site's exact target/export pins. Absence is legal raw metadata and
does not prove body conformance. Its graph is acyclic and scoped to this
Bundle. Child metadata does not issue a child checked capability; a separate
child revalidation is required to obtain one.

## Uniqueness, sorting and rejection

| Mapping | Unique identity | Canonical primary key / completeness owner |
| --- | --- | --- |
| Inputs | exported input identity; independent root source bijection checked later | (export port, run parameter, value slot); fresh I proof |
| Input use sites | full computation/contract/port/expression coordinate | same tuple; fresh Phase 3C dependency set |
| Outputs | exported output identity | (export port, output name, expression); fresh O proof |
| Resources | exported requirement identity | (export port, root slot); complete source/alias groups |
| Resource sources | (source node, contract, component resource port, slot) | same tuple, exact type/scope relation |
| Effects | complete RootEffectOccurrenceIdentity | (export AtomKey, caller digest, node, source digest, ordinal); fresh occurrences |
| Flow | (source node, source contract, flow ordinal) | current FlowMappingKey defined in encoding doc |
| Recovery | complete RecoveryActionSourceIdentity | (export action forward AtomKey, kind rank, action AtomKey, caller, node, source digest, ordinal) |
| Execution | (source node, source contract) | same tuple; complete freshly resolved child set |
| Subflow | caller-local call_site | (target digest, call_site); fresh dependency closure |

Digest keys retain algorithm/domain/hex. Full record ACE1 bytes break primary-key
ties; exact duplicate rows and declared identity duplicates are rejected.
Positive inputs remain in the current canonicalizer's order.

Structural duplicates/range/broken references fail D0–D2. Existing but
incorrect semantic witnesses reach the relevant formal comparison and return
Violation or Unproven. Missing required mapping *rows* are not invented by
Decoder; completeness is checked by the fresh nine-dimensional checker.
F04's guard precision boundary remains unchanged.
