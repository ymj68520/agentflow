# Phase 4B.2D — Root / Export Coordinate Mapping

All candidate records and rules here are **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED ABI / NOT IMPLEMENTED**.

## Mapping invariant

A proof compares values only after both values are in one explicit typed namespace. Equal integers, names, vector lengths, or operation short names are never a mapping. A map entry must retain the exact source identity, exact target identity, type evidence, and mapping provenance. Missing or ambiguous maps prevent Proven.

The canonical order for any candidate vector is by target port, then source identity/digest, then exact typed payload. Duplicate target entries are invalid. Reusing one root input or resource source for multiple exported ports is not silently accepted.

## Candidate record family

Each record below is a non-wire proposal. Its owner is the build-time RootExportAssertion. Its identity includes the complete typed fields shown, the exact root/export digests, and the proof context. No record has a standalone digest.

~~~text
RootInputPortBinding {
  export_input_port: ComponentInputPortIdentity,
  run_parameter_id: RunParameterId,
  root_value_slot: ValueSlotId,
  root_type: StructuralType,
  export_type: TypeSchemaRef,
  use_sites: Vec<InvocationInputUse>
}

RootOutputPortBinding {
  export_output_port: ComponentOutputPortIdentity,
  workflow_output_name: NFC string,
  workflow_output_expression: ExpressionId,
  root_result_type: StructuralType,
  producer_evidence: RootOutputProducerEvidence
}

RootResourcePortBinding {
  export_resource_port: ResourceRequirementPortIdentity,
  root_resource_slot: ResourceSlotId,
  root_resource_type: TypeSchemaRef,
  export_resource_type: ResourceTypeRef,
  requirement_relation: ResourceRequirementProofEvidence
}

RootEffectCoordinateBinding {
  source: ScopedEffectOccurrenceIdentity,
  root_resource_slot: ResourceSlotId,
  export_effect_atom: EffectAtom
}

RootFlowCoordinateBinding {
  source_node: ComputationId,
  source_contract_digest: DigestRefValue,
  flow_mapping_ordinal: u64,
  source_coordinate: ComponentInputPort | ComponentResourcePort | UnknownExternal,
  sink_coordinate: ComponentOutputPort | ExactEffectAtom | ExactEffectOperation
}

RootRecoveryCoordinateBinding {
  source: RecoveryActionSourceIdentity,
  export_action_identity: RecoveryActionIdentity,
  exact_usage_binding: RecoveryUsageBinding
}

RootExecutionCoordinateBinding {
  source_node: ComputationId,
  source_contract_digest: DigestRefValue,
  exact_execution_requirements: ExecutionRequirements
}
~~~

The names and fields above do not prescribe an AFIDL shape. If a future persistent publisher schema is needed, it must be additive and separately versioned; this phase creates none.

## Inputs: Export ComponentInputPort to root input

Map every exported input port to exactly one RunParameterId and its producing ValueSlotId. For v1 positive proof, the mapping must be bijective between the export input-port set and the root run-parameter set:

* Every export port is mapped once; every RunParameter is mapped once.
* Port numbers are scoped to the exact exported ComponentContractDigest.
* A RunParameter's ValueSlot producer and its exact semantic type are checked through the Semantic IR type DAG.
* Proven type compatibility requires the same exact nominal TypeSchemaRef pin after resolving the root type node. Structural resemblance, matching names, alias expansion, or subtype intuition is not enough.
* Two exported ports cannot reuse one root source as a positive proof. That would require the root to supply equal values where the export permits independent values, strengthening the input precondition. An explicit duplicate-source assertion is a Violation of the selected v1 map rule.
* Any root RunParameter not mapped to an export port is an extra required input and is a Violation. An export input with no root source is a Violation.
* Each leaf input use retains the exact computation node, source contract digest, target port, input ExpressionId, exact type, and Phase 3C availability/dependency evidence. Direct identity-preserving uses can be proven. An expression transformation that needs semantic equivalence reasoning remains Unproven.

The public input map does not prove that all runtime values are valid, that component code behaves correctly, or that an external caller actually supplied them.

## Outputs: Export ComponentOutputPort from root WorkflowOutput

Every exported output port maps to exactly one named WorkflowOutputDecl and exact expression. The map explicitly carries both the name and ComponentOutputPort number; neither is inferred from the other.

* Validate the expression under Phase 3C, including its result type, availability, producer/dominance evidence, and scope.
* Resolve the root SemanticTypeId/StructuralType to the exact nominal TypeSchemaRef and require exact type equality for a positive v1 proof.
* One output mapping cannot target two WorkflowOutput names. A single producer expression may feed multiple named outputs only when each explicit map and exact type is valid; no alias is inferred from similar names.
* Missing export output, invalid producer, or a proven type mismatch is Violation. Missing producer/type evidence is Unproven.
* An additional root WorkflowOutput not described by the export has no frozen additive-interface relation; return Unproven rather than silently ignoring it or asserting that it is safe.
* SELECT/RACE/HANDLE alternatives are covered only by the root summary's typed interface proof for every possible branch. A possible branch with no compatible producer is Violation; unresolved branch/interface evidence is Unproven.

## Resources: root ResourceSlotId to exported ResourceRequirement port

The map explicitly connects each public exported resource port to one exact root ResourceSlotId. It carries both exact resource type identities and requirement proof evidence.

* No Component resource port number equals a root ResourceSlotId by implication.
* Require exact TypeSchemaRef/ResourceTypeRef pins and validate required attributes, selector-key constraints, operation/resource compatibility, scope visibility, and the complete explicit resource relation.
* Each exported requirement port maps once. A missing root slot or a root slot with a proven incompatible type/requirement is Violation; unavailable catalog or attribute evidence is Unproven.
* A single root slot reused at multiple call sites for the same mapped export port is allowed and remains one public resource identity. Reusing a root slot for two distinct exported resource ports is Unproven in v1 because no public alias-equivalence rule exists.
* Explicit Effect resource selectors are translated through the full chain: source Component resource port -> scoped ResourceSlotId -> export ResourceRequirement port. Scope bindings and qualifiers remain exact.
* Resource alias groups are retained as explicit equivalence evidence; they never reduce occurrence bounds or create a ResourceHandle, binding, grant, or authority.

## Effect coordinates

For each source effect occurrence, retain caller SemanticIrDigest, source ComputationId, source ComponentContractDigest, entry ordinal, full EffectAtom, EffectGuard, bounds, and mapped resource slot. Map its operation by exact (catalog, name), its exported resource port through the resource map, scope bindings by exact key/value, and qualifiers by complete family/member/payload identity.

The RootEffectCoordinateBinding does not create a new atom and does not remove the source. Every possible source occurrence and residual must be accounted exactly once. A full atom match with a different qualifier, scope, operation, or mapped resource port is not a match.

## Flow coordinates

Map each source/sink fact from its FlowPortMapping, never from a name:

* Input source: component input port -> exact invocation input expression -> root RunParameter/ValueSlot -> exported input port.
* Resource source: component resource port -> scoped ResourceSlotId -> exported resource port.
* Output sink: component output port -> output ValueSlot/ExpressionId -> named WorkflowOutput -> exported output port.
* Effect sink: exact effect occurrence or operation plus exact FlowKind and EffectFlowPartition member.
* Confidentiality labels, taints, decision influence, and sink classifications retain exact catalog identities. Unknown external provenance, selector influence, and open partitions remain unresolved.

No root-level branch condition or Flow string is allowed to erase an input/resource/output identity. If a control influence cannot be translated by exact typed evidence, the affected Flow proof is Unproven.

## Recovery and Usage coordinates

Map every source RecoveryActionBinding and matching RecoveryUsageBinding by:

* caller SemanticIrDigest, source computation node, source ComponentContractDigest;
* exact forward occurrence ordinal and full forward EffectAtom;
* exact RecoveryActionKind and full action EffectAtom;
* exact activation/attempt bounds and per-attempt UsageSignature.

Translate resource selectors in both forward and action atoms through the same explicit resource map. Compare RecoveryUsage by exact phase, meter, unit, and bound. Preserve action kind and source/action multiset until root aggregation completes. RecoverySource node equality alone does not prove action/profile closure.

The exported K comparison is not a name-level operation mapping. If the existing Recovery compatibility relation cannot prove a changed declaration's direction, return Unproven. Root action totals are separately checked against root/export epsilon and recovery Usage ceilings; they are not added again.

## Execution coordinates

ExecutionSourceSummary remains per child. Compare every source's exact ExecutionRequirements to the exported X constraint through the explicit execution universes. Record feasibility and security-preservation proofs separately. Never sum profiles into a Worker, infer placement, or treat a profile catalog as admission evidence.

## Failure semantics and identity

* Proven requires complete, unique typed mappings and all dimension-specific relations.
* Violation requires an exact contradiction, missing required public input/output, incompatible required type, or proven stronger requirement.
* Unproven covers missing/ambiguous identity, unsupported variance, absent catalog/attribute/producer relation, or a relation the frozen model does not express.
* RequiresEvidence covers facts that require runtime receipts or trusted body/implementation evidence.

Changing any mapping changes the assertion input identity and requires root derivation and comparison again. No mapping is persisted by port number alone; no map contains metadata:string, proof:bool, or compatible:bool shortcuts.
