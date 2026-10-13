# Phase 4C.1B.1 — Schema Materialization Audit

Start HEAD: efc2a8d90a65432916a8dbf559510e2936f0d146; initial worktree clean.

The complete Bundle type closure was materialized before ABI registration:
108 named types, including 17 path-specific auxiliary types, and 298 numbered
members. Independent Python module-set compilation passed. Existing production
Rust spec-check compiled all 17 modules; its only 298 errors were the expected
unregistered new assignments. No parse, import or recursive-type conflict.

The 477-row pre-allocation snapshot is [baseline.json](../../spec/conformance/portable-proof-codec/baseline.json).
The complete machine inventory is [materialization.json](../../spec/conformance/portable-proof-codec/materialization.json).

## Compatibility and naming proof

AFIDL1 sections 2, 4 and 5 require named declarations, explicit positive unique
numbers, explicit imports and an acyclic named-type graph. Anonymous shapes
and CompleteCollection occurrences are lifted into named declarations.
Traversal follows the declared owner/field/case path, never HashMap iteration
or Rust reflection. Sibling numbers are unique, wrapper steps explicit, named
references stop traversal, and no anonymous-shape deduplication occurs.
Owner/path pairs and emitted names are checked for injectivity over the whole
inventory. PpiAux is reserved; no ordinary declaration uses that prefix.
Existing imported full nominal identities are neither shadowed nor renumbered.
Same local spelling in different specification families is legal under AFIDL1.

No formal schema conflict was found. Restricted DigestRef synonyms render as
the existing imported record plus D2 domain rules, preserving nominal identity.
set renders as list with explicit normative order/uniqueness. Generic macros
add no runtime or wire extension mechanism.

Origin/build submission sidecars, local VerifierSupportSnapshot and future
UpcasterDefinition lie outside the Bundle type closure and remain unregistered
interfaces. No Bundle field or reachable declaration is omitted by this scope.

## Imports

| Alias | Exact family |
| --- | --- |
| contract | agentflow.component.contract@1 |
| core | agentflow.semantic.core@1 |
| effect | agentflow.effect.signature@1 |
| execution | agentflow.execution.requirements@1 |
| flow | agentflow.information.flow@1 |
| ir | agentflow.semantic.ir@1 |
| resource | agentflow.resource.requirement@1 |
| usage | agentflow.usage.signature@1 |

The whole module-set compilers check transitive dependencies. The machine
inventory also lists every imported symbol binding.

## Aliases

| Name | Target |
| --- | --- |
| AssertionId | U32 |
| Bytes | bytes |
| CompilerManifestId | U32 |
| ContextId | U32 |
| LogicalPath | Text |
| LowerHex64 | Text |
| Ordinal64 | U64 |
| ResolverId | U32 |
| SpecId | Text |
| StructuralTypeId | U32 |
| Text | string |
| TypeName | Text |
| U32 | uint32 |
| U64 | uint64 |

## Auxiliary expansion

| Emitted name | Owner | Traversal path |
| --- | --- | --- |
| PpiAuxExecutionUniverseContentPF1G1 | ExecutionUniverseContent | F1G1 |
| PpiAuxExecutionUniverseContentPF2G1 | ExecutionUniverseContent | F2G1 |
| PpiAuxExecutionUniverseContentPF3G1 | ExecutionUniverseContent | F3G1 |
| PpiAuxExecutionUniverseContentPF4G1 | ExecutionUniverseContent | F4G1 |
| PpiAuxExecutionUniverseContentPF5G1 | ExecutionUniverseContent | F5G1 |
| PpiAuxFlowCatalogContentPF1G1 | FlowCatalogContent | F1G1 |
| PpiAuxFlowCatalogContentPF2G1 | FlowCatalogContent | F2G1 |
| PpiAuxFlowCatalogContentPF3G1 | FlowCatalogContent | F3G1 |
| PpiAuxPlainChecksumPF1 | PlainChecksum | F1 |
| PpiAuxPortableContextContentPF1G1 | PortableContextContent | F1G1 |
| PpiAuxPortableContextContentPF2G1 | PortableContextContent | F2G1 |
| PpiAuxPortableContextContentPF3G1 | PortableContextContent | F3G1 |
| PpiAuxPortableContextContentPF6G1 | PortableContextContent | F6G1 |
| PpiAuxRecoveryUniverseContentPF1G1 | RecoveryUniverseContent | F1G1 |
| PpiAuxRecoveryUniverseContentPF2G1 | RecoveryUniverseContent | F2G1 |
| PpiAuxResourceUniverseContentPF1G1 | ResourceUniverseContent | F1G1 |
| PpiAuxResourceUniverseContentPF2G1 | ResourceUniverseContent | F2G1 |

## Complete numbered declaration inventory

These audit proposals are allocated by the existing registry generator and
checked against production AFIDL extraction before becoming registered ABI.
No old family participates in this new allocation.

| Type | Kind | Number | Member | Exact payload type |
| --- | --- | --- | --- | --- |
| AssertionEntry | record | 1 | id | AssertionId |
| AssertionEntry | record | 2 | ir_asset | TypedContentRef |
| AssertionEntry | record | 3 | export_contract_asset | TypedContentRef |
| AssertionEntry | record | 4 | assertion | PortableRootExportAssertion |
| AttributionProfile | variant | 1 | full_content_revalidation | IncompleteAttribution |
| AttributionProfile | variant | 2 | rule_complete_attribution | CompleteAttributionClaim |
| AttributionProfile | variant | 3 | portable_certificate | unit |
| ClosedTableContentSet | record | 1 | hash_domains | RegistryContent |
| ClosedTableContentSet | record | 2 | abi_numbers | RegistryContent |
| ClosedTableContentSet | record | 3 | rule_areas | RegistryContent |
| ClosedTableContentSet | record | 4 | afidl_primitives | RegistryContent |
| CompilerManifestEntry | record | 1 | id | CompilerManifestId |
| CompilerManifestEntry | record | 2 | content | CompilerSemanticsManifest |
| CompilerSemanticsManifest | record | 1 | registries | RegistryContentSet |
| CompilerSemanticsManifest | record | 2 | closed_tables | ClosedTableContentSet |
| CompilerSemanticsManifest | record | 3 | normative_sources | list<SourceContentIdentity> |
| CompilerSemanticsManifest | record | 4 | semantic_roles | list<SemanticRoleEntry> |
| CompilerSemanticsManifest | record | 5 | attribution | AttributionProfile |
| CompilerSemanticsManifest | record | 6 | static_identity_projection | StaticIdentityProjection |
| CompleteAttributionClaim | record | 1 | direct_rules | list<RuleContentIdentity> |
| CompleteAttributionClaim | record | 2 | transitive_rules | list<RuleContentIdentity> |
| CompleteAttributionClaim | record | 3 | dependency_edges | list<RuleDependencyEdge> |
| ComponentInputPortIdentity | record | 1 | exported_contract_digest | core.DigestRefValue |
| ComponentInputPortIdentity | record | 2 | port_no | U32 |
| ComponentOutputPortIdentity | record | 1 | exported_contract_digest | core.DigestRefValue |
| ComponentOutputPortIdentity | record | 2 | port_no | U32 |
| ContractAsset | record | 1 | reference | TypedContentRef |
| ContractAsset | record | 2 | canonical_content | Bytes |
| EffectCatalogContent | record | 1 | specification_id | SpecId |
| EffectCatalogContent | record | 2 | operations | list<Text> |
| EffectResourceRelationContent | record | 1 | header | RelationContentHeader |
| EffectResourceRelationContent | record | 2 | canonical_content | Bytes |
| ExecutionCapabilityContent | record | 1 | capability | execution.ExecutionCapabilityRef |
| ExecutionCapabilityContent | record | 2 | exact_version | Text |
| ExecutionCapabilityContent | record | 3 | supported_versions | list<Text> |
| ExecutionCapabilityContent | record | 4 | attributes | list<execution.CapabilityAttribute> |
| ExecutionRuntimeProfileContent | record | 1 | profile | execution.RuntimeProfileRef |
| ExecutionRuntimeProfileContent | record | 2 | family | Text |
| ExecutionRuntimeProfileContent | record | 3 | features | list<Text> |
| ExecutionUniverseContent | record | 1 | capabilities | PpiAuxExecutionUniverseContentPF1G1 |
| ExecutionUniverseContent | record | 2 | trust_domains | PpiAuxExecutionUniverseContentPF2G1 |
| ExecutionUniverseContent | record | 3 | regions | PpiAuxExecutionUniverseContentPF3G1 |
| ExecutionUniverseContent | record | 4 | region_compatibility | PpiAuxExecutionUniverseContentPF4G1 |
| ExecutionUniverseContent | record | 5 | runtime_profiles | PpiAuxExecutionUniverseContentPF5G1 |
| FlowCatalogContent | record | 1 | confidentiality_labels | PpiAuxFlowCatalogContentPF1G1 |
| FlowCatalogContent | record | 2 | taints | PpiAuxFlowCatalogContentPF2G1 |
| FlowCatalogContent | record | 3 | effect_sinks | PpiAuxFlowCatalogContentPF3G1 |
| FlowEffectSinkContent | record | 1 | operation | effect.EffectOperationRef |
| FlowEffectSinkContent | record | 2 | decision_capable | bool |
| FlowExecutionRelationContent | record | 1 | header | RelationContentHeader |
| FlowExecutionRelationContent | record | 2 | canonical_content | Bytes |
| FlowSinkBinding | variant | 1 | output_value | core.ValueSlotId |
| FlowSinkBinding | variant | 2 | effect_occurrences | list<Ordinal64> |
| FlowSourceBinding | variant | 1 | input_expression | core.ExpressionId |
| FlowSourceBinding | variant | 2 | resource_slot | core.ResourceSlotId |
| FlowSourceBinding | variant | 3 | unknown_external | unit |
| IncompleteAttribution | record | 1 | listed_rules | list<RuleContentIdentity> |
| IrAsset | record | 1 | reference | TypedContentRef |
| IrAsset | record | 2 | canonical_content | Bytes |
| ModelMissingReason | enum | 1 | not_formalized | (enum value) |
| ModelMissingReason | enum | 2 | future_dependency | (enum value) |
| ModuleContent | record | 1 | specification_id | SpecId |
| ModuleContent | record | 2 | named_types | list<core.TypeSchemaRef> |
| PlainChecksum | record | 1 | algorithm | PpiAuxPlainChecksumPF1 |
| PlainChecksum | record | 2 | hex | LowerHex64 |
| PortableContextContent | record | 1 | compiled_modules | PpiAuxPortableContextContentPF1G1 |
| PortableContextContent | record | 2 | type_schemas | PpiAuxPortableContextContentPF2G1 |
| PortableContextContent | record | 3 | effect_catalogs | PpiAuxPortableContextContentPF3G1 |
| PortableContextContent | record | 4 | resource_types | ResourceUniverseContent |
| PortableContextContent | record | 5 | flow_catalogs | FlowCatalogContent |
| PortableContextContent | record | 6 | usage_meters | PpiAuxPortableContextContentPF6G1 |
| PortableContextContent | record | 7 | recovery_operations | RecoveryUniverseContent |
| PortableContextContent | record | 8 | execution_universes | ExecutionUniverseContent |
| PortableContextContent | record | 9 | relation_snapshots | RelationSnapshotSlots |
| PortableProofContextManifest | record | 1 | context_id | ContextId |
| PortableProofContextManifest | record | 2 | resolver_id | ResolverId |
| PortableProofInputBundle | record | 1 | schema_version | SchemaVersion |
| PortableProofInputBundle | record | 2 | root_assertion_id | AssertionId |
| PortableProofInputBundle | record | 3 | schema_assets | list<SchemaAsset> |
| PortableProofInputBundle | record | 4 | ir_assets | list<IrAsset> |
| PortableProofInputBundle | record | 5 | contract_assets | list<ContractAsset> |
| PortableProofInputBundle | record | 6 | structural_types | list<StructuralTypeNode> |
| PortableProofInputBundle | record | 7 | proof_contexts | list<ProofContextEntry> |
| PortableProofInputBundle | record | 8 | resolver_dependency_manifests | list<ResolverEntry> |
| PortableProofInputBundle | record | 9 | compiler_semantics_manifests | list<CompilerManifestEntry> |
| PortableProofInputBundle | record | 10 | root_export_assertions | list<AssertionEntry> |
| PortableResolverManifest | record | 1 | caller_ir | TypedContentRef |
| PortableResolverManifest | record | 2 | context_id | ContextId |
| PortableResolverManifest | record | 3 | components | list<TypedContentRef> |
| PortableResolverManifest | record | 4 | subflow_targets | list<ResolverTarget> |
| PortableResolverManifest | record | 5 | source_associations | list<SourceAssociation> |
| PortableRootExportAssertion | record | 1 | exact_semantic_ir_digest | core.DigestRefValue |
| PortableRootExportAssertion | record | 2 | exact_exported_component_contract_digest | core.DigestRefValue |
| PortableRootExportAssertion | record | 3 | compiler_semantics_id | CompilerManifestId |
| PortableRootExportAssertion | record | 4 | proof_context_manifest | PortableProofContextManifest |
| PortableRootExportAssertion | record | 5 | input_mappings | list<RootInputPortBinding> |
| PortableRootExportAssertion | record | 6 | output_mappings | list<RootOutputPortBinding> |
| PortableRootExportAssertion | record | 7 | resource_mappings | list<RootResourcePortBinding> |
| PortableRootExportAssertion | record | 8 | effect_mappings | list<RootEffectCoordinateBinding> |
| PortableRootExportAssertion | record | 9 | flow_mappings | list<RootFlowCoordinateBinding> |
| PortableRootExportAssertion | record | 10 | recovery_mappings | list<RootRecoveryCoordinateBinding> |
| PortableRootExportAssertion | record | 11 | execution_mappings | list<RootExecutionCoordinateBinding> |
| PortableRootExportAssertion | record | 12 | subflow_dependencies | list<SubflowProofDependency> |
| PpiAuxExecutionUniverseContentPF1G1 | variant | 1 | complete | list<ExecutionCapabilityContent> |
| PpiAuxExecutionUniverseContentPF1G1 | variant | 2 | unavailable | UnavailableReason |
| PpiAuxExecutionUniverseContentPF2G1 | variant | 1 | complete | list<execution.TrustDomainRef> |
| PpiAuxExecutionUniverseContentPF2G1 | variant | 2 | unavailable | UnavailableReason |
| PpiAuxExecutionUniverseContentPF3G1 | variant | 1 | complete | list<execution.RegionRef> |
| PpiAuxExecutionUniverseContentPF3G1 | variant | 2 | unavailable | UnavailableReason |
| PpiAuxExecutionUniverseContentPF4G1 | variant | 1 | complete | list<RegionCompatibility> |
| PpiAuxExecutionUniverseContentPF4G1 | variant | 2 | unavailable | UnavailableReason |
| PpiAuxExecutionUniverseContentPF5G1 | variant | 1 | complete | list<ExecutionRuntimeProfileContent> |
| PpiAuxExecutionUniverseContentPF5G1 | variant | 2 | unavailable | UnavailableReason |
| PpiAuxFlowCatalogContentPF1G1 | variant | 1 | complete | list<flow.ConfidentialityLabelRef> |
| PpiAuxFlowCatalogContentPF1G1 | variant | 2 | unavailable | UnavailableReason |
| PpiAuxFlowCatalogContentPF2G1 | variant | 1 | complete | list<flow.TaintRef> |
| PpiAuxFlowCatalogContentPF2G1 | variant | 2 | unavailable | UnavailableReason |
| PpiAuxFlowCatalogContentPF3G1 | variant | 1 | complete | list<FlowEffectSinkContent> |
| PpiAuxFlowCatalogContentPF3G1 | variant | 2 | unavailable | UnavailableReason |
| PpiAuxPlainChecksumPF1 | enum | 1 | sha256 | (enum value) |
| PpiAuxPortableContextContentPF1G1 | variant | 1 | complete | list<ModuleContent> |
| PpiAuxPortableContextContentPF1G1 | variant | 2 | unavailable | UnavailableReason |
| PpiAuxPortableContextContentPF2G1 | variant | 1 | complete | list<core.TypeSchemaRef> |
| PpiAuxPortableContextContentPF2G1 | variant | 2 | unavailable | UnavailableReason |
| PpiAuxPortableContextContentPF3G1 | variant | 1 | complete | list<EffectCatalogContent> |
| PpiAuxPortableContextContentPF3G1 | variant | 2 | unavailable | UnavailableReason |
| PpiAuxPortableContextContentPF6G1 | variant | 1 | complete | list<UsageMeterContent> |
| PpiAuxPortableContextContentPF6G1 | variant | 2 | unavailable | UnavailableReason |
| PpiAuxRecoveryUniverseContentPF1G1 | variant | 1 | complete | list<EffectCatalogContent> |
| PpiAuxRecoveryUniverseContentPF1G1 | variant | 2 | unavailable | UnavailableReason |
| PpiAuxRecoveryUniverseContentPF2G1 | variant | 1 | complete | list<ReconciliationOperationRef> |
| PpiAuxRecoveryUniverseContentPF2G1 | variant | 2 | unavailable | UnavailableReason |
| PpiAuxResourceUniverseContentPF1G1 | variant | 1 | complete | list<core.TypeSchemaRef> |
| PpiAuxResourceUniverseContentPF1G1 | variant | 2 | unavailable | UnavailableReason |
| PpiAuxResourceUniverseContentPF2G1 | variant | 1 | complete | list<ResourceTypeContent> |
| PpiAuxResourceUniverseContentPF2G1 | variant | 2 | unavailable | UnavailableReason |
| ProofContextEntry | record | 1 | id | ContextId |
| ProofContextEntry | record | 2 | content | PortableContextContent |
| ReconciliationOperationRef | record | 1 | catalog | SpecId |
| ReconciliationOperationRef | record | 2 | name | Text |
| RecoveryActionSourceIdentity | record | 1 | caller_semantic_ir_digest | core.DigestRefValue |
| RecoveryActionSourceIdentity | record | 2 | source_node | ir.ComputationId |
| RecoveryActionSourceIdentity | record | 3 | source_contract_digest | core.DigestRefValue |
| RecoveryActionSourceIdentity | record | 4 | forward_entry_ordinal | Ordinal64 |
| RecoveryActionSourceIdentity | record | 5 | action | contract.RecoveryActionIdentity |
| RecoveryUniverseContent | record | 1 | effect_catalogs | PpiAuxRecoveryUniverseContentPF1G1 |
| RecoveryUniverseContent | record | 2 | reconciliation_operations | PpiAuxRecoveryUniverseContentPF2G1 |
| RecoveryUsageRelationContent | record | 1 | header | RelationContentHeader |
| RecoveryUsageRelationContent | record | 2 | canonical_content | Bytes |
| RegionCompatibility | record | 1 | from | execution.RegionRef |
| RegionCompatibility | record | 2 | to | execution.RegionRef |
| RegistryContent | record | 1 | content_sha256 | PlainChecksum |
| RegistryContent | record | 2 | content | Bytes |
| RegistryContentSet | record | 1 | specifications | RegistryContent |
| RegistryContentSet | record | 2 | rules | RegistryContent |
| RelationContentHeader | record | 1 | schema | core.TypeSchemaRef |
| RelationContentHeader | record | 2 | revision | Text |
| RelationContentHeader | record | 3 | canonical_content_checksum | PlainChecksum |
| RelationSnapshotSlots | record | 1 | effect_resource | option<EffectResourceRelationContent> |
| RelationSnapshotSlots | record | 2 | recovery_usage | option<RecoveryUsageRelationContent> |
| RelationSnapshotSlots | record | 3 | flow_execution | option<FlowExecutionRelationContent> |
| ResolvedSubflowAssociation | record | 1 | target_semantic_ir_digest | core.DigestRefValue |
| ResolvedSubflowAssociation | record | 2 | exported_contract_digest | option<core.DigestRefValue> |
| ResolverEntry | record | 1 | id | ResolverId |
| ResolverEntry | record | 2 | content | PortableResolverManifest |
| ResolverTarget | record | 1 | target_semantic_ir_digest | core.DigestRefValue |
| ResolverTarget | record | 2 | target_ir_content | option<TypedContentRef> |
| ResolverTarget | record | 3 | exported_contract | option<TypedContentRef> |
| ResolverTarget | record | 4 | direct_dependencies | option<list<core.DigestRefValue>> |
| ResolverTarget | record | 5 | transitive_dependencies | option<list<SubflowTargetIdentity>> |
| ResourceRequirementPortIdentity | record | 1 | exported_contract_digest | core.DigestRefValue |
| ResourceRequirementPortIdentity | record | 2 | port_no | U32 |
| ResourceTypeContent | record | 1 | schema | core.TypeSchemaRef |
| ResourceTypeContent | record | 2 | recognized_attributes | list<Text> |
| ResourceUniverseContent | record | 1 | schemas | PpiAuxResourceUniverseContentPF1G1 |
| ResourceUniverseContent | record | 2 | descriptors | PpiAuxResourceUniverseContentPF2G1 |
| RoleDeclaration | variant | 1 | defined | SemanticRoleDefinition |
| RoleDeclaration | variant | 2 | model_missing | ModelMissingReason |
| RootEffectCoordinateBinding | record | 1 | source | RootEffectOccurrenceIdentity |
| RootEffectCoordinateBinding | record | 2 | root_resource_slot | option<core.ResourceSlotId> |
| RootEffectCoordinateBinding | record | 3 | export_atom | effect.EffectAtom |
| RootEffectCoordinateBinding | record | 4 | export_guard | effect.EffectGuard |
| RootEffectOccurrenceIdentity | record | 1 | caller_semantic_ir_digest | core.DigestRefValue |
| RootEffectOccurrenceIdentity | record | 2 | source_node | ir.ComputationId |
| RootEffectOccurrenceIdentity | record | 3 | source_contract_digest | core.DigestRefValue |
| RootEffectOccurrenceIdentity | record | 4 | source_entry_ordinal | Ordinal64 |
| RootEffectOccurrenceIdentity | record | 5 | atom | effect.EffectAtom |
| RootEffectOccurrenceIdentity | record | 6 | guard | effect.EffectGuard |
| RootExecutionCoordinateBinding | record | 1 | source_node | ir.ComputationId |
| RootExecutionCoordinateBinding | record | 2 | source_contract_digest | core.DigestRefValue |
| RootExecutionCoordinateBinding | record | 3 | exact_execution_requirements | execution.ExecutionRequirements |
| RootFlowCoordinateBinding | record | 1 | source_node | ir.ComputationId |
| RootFlowCoordinateBinding | record | 2 | source_contract_digest | core.DigestRefValue |
| RootFlowCoordinateBinding | record | 3 | flow_mapping_ordinal | U64 |
| RootFlowCoordinateBinding | record | 4 | source | option<flow.FlowSourceRef> |
| RootFlowCoordinateBinding | record | 5 | sink | option<flow.FlowConstraint> |
| RootFlowCoordinateBinding | record | 6 | source_binding | option<FlowSourceBinding> |
| RootFlowCoordinateBinding | record | 7 | sink_binding | option<FlowSinkBinding> |
| RootFlowCoordinateBinding | record | 8 | export_source | option<flow.FlowSourceRef> |
| RootFlowCoordinateBinding | record | 9 | export_sink | option<flow.FlowConstraint> |
| RootFlowCoordinateBinding | record | 10 | source_effect_partition | option<contract.EffectFlowPartition> |
| RootFlowCoordinateBinding | record | 11 | export_effect_partition | option<contract.EffectFlowPartition> |
| RootInputPortBinding | record | 1 | export_input_port | ComponentInputPortIdentity |
| RootInputPortBinding | record | 2 | run_parameter_id | ir.RunParameterId |
| RootInputPortBinding | record | 3 | root_value_slot | core.ValueSlotId |
| RootInputPortBinding | record | 4 | root_type | StructuralTypeId |
| RootInputPortBinding | record | 5 | export_type | core.TypeSchemaRef |
| RootInputPortBinding | record | 6 | use_sites | list<RootInputUseSite> |
| RootInputUseSite | record | 1 | computation | ir.ComputationId |
| RootInputUseSite | record | 2 | source_contract_digest | core.DigestRefValue |
| RootInputUseSite | record | 3 | source_component_port | U32 |
| RootInputUseSite | record | 4 | input_expression | core.ExpressionId |
| RootInputUseSite | record | 5 | exact_type | core.TypeSchemaRef |
| RootInputUseSite | record | 6 | dependency_slots | list<core.ValueSlotId> |
| RootOutputPortBinding | record | 1 | export_output_port | ComponentOutputPortIdentity |
| RootOutputPortBinding | record | 2 | workflow_output_name | Text |
| RootOutputPortBinding | record | 3 | workflow_output_expression | core.ExpressionId |
| RootOutputPortBinding | record | 4 | root_result_type | StructuralTypeId |
| RootOutputPortBinding | record | 5 | producer_evidence | RootOutputProducerEvidence |
| RootOutputProducerEvidence | record | 1 | exact_result_type | core.TypeSchemaRef |
| RootOutputProducerEvidence | record | 2 | dependency_slots | list<core.ValueSlotId> |
| RootRecoveryCoordinateBinding | record | 1 | source | RecoveryActionSourceIdentity |
| RootRecoveryCoordinateBinding | record | 2 | export_action_identity | contract.RecoveryActionIdentity |
| RootRecoveryCoordinateBinding | record | 3 | exact_usage_binding | contract.RecoveryUsageBinding |
| RootResourcePortBinding | record | 1 | export_resource_port | ResourceRequirementPortIdentity |
| RootResourcePortBinding | record | 2 | root_resource_slot | core.ResourceSlotId |
| RootResourcePortBinding | record | 3 | root_resource_type | core.TypeSchemaRef |
| RootResourcePortBinding | record | 4 | export_resource_type | resource.ResourceTypeRef |
| RootResourcePortBinding | record | 5 | source_coordinates | list<RootResourceSourceCoordinate> |
| RootResourceSourceCoordinate | record | 1 | source_node | ir.ComputationId |
| RootResourceSourceCoordinate | record | 2 | source_contract_digest | core.DigestRefValue |
| RootResourceSourceCoordinate | record | 3 | component_resource_port | U32 |
| RootResourceSourceCoordinate | record | 4 | resource_slot | core.ResourceSlotId |
| RootResourceSourceCoordinate | record | 5 | alias_component_ports | list<U32> |
| RuleContentIdentity | record | 1 | rule_id | Text |
| RuleContentIdentity | record | 2 | normative_source | LogicalPath |
| RuleContentIdentity | record | 3 | source_anchor | Text |
| RuleContentIdentity | record | 4 | short_statement | Text |
| RuleDependencyEdge | record | 1 | dependent | Text |
| RuleDependencyEdge | record | 2 | prerequisite | Text |
| SchemaAsset | record | 1 | reference | core.TypeSchemaRef |
| SchemaAsset | record | 2 | canonical_content | Bytes |
| SchemaVersion | enum | 1 | portable_proof_inputs_v1 | (enum value) |
| SemanticRole | enum | 1 | static_analysis | (enum value) |
| SemanticRole | enum | 2 | contract_validation | (enum value) |
| SemanticRole | enum | 3 | semantic_ir_validation | (enum value) |
| SemanticRole | enum | 4 | dataflow | (enum value) |
| SemanticRole | enum | 5 | leaf_resolution | (enum value) |
| SemanticRole | enum | 6 | control_algebra | (enum value) |
| SemanticRole | enum | 7 | root_recovery | (enum value) |
| SemanticRole | enum | 8 | root_export | (enum value) |
| SemanticRole | enum | 9 | plan_lowering | (enum value) |
| SemanticRole | enum | 10 | runtime_semantics_profile | (enum value) |
| SemanticRoleDefinition | record | 1 | analysis_semantics_version | Text |
| SemanticRoleDefinition | record | 2 | normative_specifications | list<SpecId> |
| SemanticRoleDefinition | record | 3 | required_schema_pins | list<core.TypeSchemaRef> |
| SemanticRoleDefinition | record | 4 | depends_on | list<SemanticRole> |
| SemanticRoleEntry | record | 1 | role | SemanticRole |
| SemanticRoleEntry | record | 2 | declaration | RoleDeclaration |
| SourceAssociation | record | 1 | caller_ir | TypedContentRef |
| SourceAssociation | record | 2 | source_node | ir.ComputationId |
| SourceAssociation | record | 3 | target | SourceAssociationTarget |
| SourceAssociationTarget | variant | 1 | invoke | TypedContentRef |
| SourceAssociationTarget | variant | 2 | call_subflow | ResolvedSubflowAssociation |
| SourceAssociationTarget | variant | 3 | await_signal | core.TypeSchemaRef |
| SourceContentIdentity | record | 1 | specification_id | SpecId |
| SourceContentIdentity | record | 2 | logical_path | LogicalPath |
| SourceContentIdentity | record | 3 | content_sha256 | PlainChecksum |
| SourceContentIdentity | record | 4 | content | Bytes |
| StaticIdentityProjection | record | 1 | analysis_semantics_version | Text |
| StaticIdentityProjection | record | 2 | normative_specification_ids | list<SpecId> |
| StaticIdentityProjection | record | 3 | applied_rules | list<RuleContentIdentity> |
| StructuralTypeBody | variant | 1 | primitive | core.SemanticPrimitiveType |
| StructuralTypeBody | variant | 2 | schema | core.TypeSchemaRef |
| StructuralTypeBody | variant | 3 | list | StructuralTypeId |
| StructuralTypeBody | variant | 4 | map_string | StructuralTypeId |
| StructuralTypeBody | variant | 5 | option | StructuralTypeId |
| StructuralTypeNode | record | 1 | id | StructuralTypeId |
| StructuralTypeNode | record | 2 | body | StructuralTypeBody |
| SubflowProofDependency | record | 1 | call_site | ir.ComputationId |
| SubflowProofDependency | record | 2 | target_semantic_ir_digest | core.DigestRefValue |
| SubflowProofDependency | record | 3 | target_exported_contract_digest | option<core.DigestRefValue> |
| SubflowProofDependency | record | 4 | target_root_assertion_id | option<AssertionId> |
| SubflowProofDependency | record | 5 | transitive_targets | list<SubflowTargetIdentity> |
| SubflowTargetIdentity | record | 1 | semantic_ir_digest | core.DigestRefValue |
| SubflowTargetIdentity | record | 2 | exported_contract_digest | option<core.DigestRefValue> |
| TypedContentRef | record | 1 | digest | core.DigestRefValue |
| TypedContentRef | record | 2 | type_schema | core.TypeSchemaRef |
| UnavailableReason | enum | 1 | not_supplied | (enum value) |
| UnavailableReason | enum | 2 | content_not_retained | (enum value) |
| UnavailableReason | enum | 3 | unsupported_projection | (enum value) |
| UsageAggregation | enum | 1 | additive | (enum value) |
| UsageAggregation | enum | 2 | unsupported | (enum value) |
| UsageMeterContent | record | 1 | meter | usage.UsageMeterRef |
| UsageMeterContent | record | 2 | unit | UsageUnitRef |
| UsageMeterContent | record | 3 | aggregation | UsageAggregation |
| UsageUnitRef | record | 1 | catalog | SpecId |
| UsageUnitRef | record | 2 | name | Text |
