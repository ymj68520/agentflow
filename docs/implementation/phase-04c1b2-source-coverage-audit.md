# Phase 4C.1B.2 — Portable-to-Rust Source Coverage Audit

This audit describes the implemented D5-D8 path. It consumes only
`SupportedPortableStaticInputs`, which can be constructed by the registered
D0-D4 codec. It changes no registered schema, source, ABI assignment, or
HashV1 domain.

| Portable source | Fresh Rust projection / consumer | Equality evidence |
| --- | --- | --- |
| `compiled_modules` | `ModuleProjection` → deterministic AFIDL source → `compile_module_set` | Full module IDs, named member refs, compiled bodies and ACE1 schema bytes compare exactly |
| `type_schemas` | independent `TypeSchemaUniverse` | Full sorted `TypeSchemaRef` set recaptured from read-only entries |
| `effect_catalogs` | `EffectCatalogUniverse` | Every catalog ID and complete operation set recaptured |
| `resource_types.schemas` | ResourceTypeUniverse's private `TypeSchemaUniverse` | Kept separate from global schema membership; exact refs recaptured |
| `resource_types.descriptors` | `ResourceTypeDescriptor` recognition | Exact type pin and complete attribute vocabulary recaptured |
| Flow labels, taints, effect sinks | `FlowCatalogs` | Each separate set and decision-capable bit recaptured |
| Usage meters | `UsageMeterUniverse` | Meter, unit and Additive/Unsupported state recaptured |
| Recovery effect catalogs and reconciliation operations | `RecoveryOperationUniverse` | Independently constructed; never aliased to context EffectCatalogUniverse |
| Execution capabilities, versions, attributes | `ExecutionCapabilityUniverse` | Exact version inserted first; full supported set and attributes recaptured |
| Trust domains, regions, directed compatibility, profiles | `ExecutionUniverses` | Exact member sets and directional pairs recaptured |
| Three optional relation slots | typed `RelationCatalogSnapshot<T>` | None/Some state, schema pin, revision, SHA-256 checksum, typed canonical bytes all compare |
| Resolver component references | D6 checked assets → formal snapshot Builder | Fresh snapshot pins compare as exact digest tuples |
| Target IR/export/direct/transitive fields | D7 assets + `SubflowIrUniverse` + formal Builder | Fresh target, export and IR-derived edge association compare to declaration |
| Source associations | checked caller IR + `resolve_computation_leaf` | Caller digest, computation ID, target kind and exact target pin compare per occurrence |

`Complete([])` is represented as a complete empty list. An `Unavailable`
required collection fails D5 as `IncompleteContext`. No constructor's default,
deduplication, normalization, or replacement behavior is accepted as decode
semantics: members and canonical spellings are checked before insertion, then
the entire typed projection is recaptured.

The complete registered Bundle carries 314 SchemaAssets in this fixture; its
16 static ModuleContent rows select the compiler module set independently.
The 108 newly registered Portable schema types (including 17 auxiliary PPI
types) are not a substitute for either complete context membership or the
static module member set.

Implementation entry points are `SupportedPortableStaticInputs::reconstruct_context`
and `SupportedPortableStaticInputs::reconstruct_static_inputs` in
`crates/af-portable/src/support.rs`; field decoding and D5 construction are in
`crates/af-portable/src/reconstruction.rs`, and fresh D6-D8 validation is in
`crates/af-portable/src/pipeline.rs`.

The audit also found two registered D1 set-key projections that were not
implemented for populated data: `ResourceTypeContent.schema` under
`SchemaRefKey`, and directional `RegionCompatibility(from,to)`. The existing
order-key interpreter now follows those already-registered keys. No set rule,
schema field, ABI row, or current-profile identity changed.

## Field-level reconstruction matrix

| Portable path and registered shape | Rust owner / formal API | Identity retained and recapture | Reject stage |
| --- | --- | --- | --- |
| `schema_assets[]: SchemaAsset` | Registered TypeSchema shape decoder; `TypeSchemaRef`; `parse_schema_assets` | Exact `(SpecificationId, TypeName)`, schema digest and canonical ACE1 bytes | D3 for asset table; D5 for reconstruction/resource bounds |
| `proof_contexts[].id/content: ProofContextEntry/PortableContextContent` | `parse_context_projection` | Exact ContextId; all nine subfields decoded without substituting defaults | D5 |
| `content.compiled_modules: PpiAuxPortableContextContentPF1G1<ModuleContent>` | AFIDL projection + `compile_module_set` | Complete module IDs, selected named members and compiled member bytes | D5 |
| `content.type_schemas: PpiAuxPortableContextContentPF2G1<TypeSchemaRef>` | `TypeSchemaUniverse::insert_type` | Global exact SchemaRef set, separate from Resource-private membership | D5 |
| `content.effect_catalogs: PpiAuxPortableContextContentPF3G1<EffectCatalogContent>` | `EffectCatalogUniverse` builder | Catalog identity and complete operation sets, including unused declarations | D5 |
| `content.resource_types.schemas/descriptors: ResourceUniverseContent` | Private schema universe + `ResourceTypeUniverse::insert_schema_type` and `ResourceTypeDescriptor` | Private schema membership, exact descriptor pin and recognized attributes | D5 |
| `content.flow_catalogs: FlowCatalogContent` | `FlowCatalogs` constructors | Labels, taints, effect sinks and decision-capable flag; unknown inputs stay unavailable | D5 |
| `content.usage_meters: PpiAuxPortableContextContentPF6G1<UsageMeterContent>` | `UsageMeterUniverse` builder | Meter/unit identity and Additive/Unsupported aggregation | D5 |
| `content.recovery_operations: RecoveryUniverseContent` | `RecoveryOperationUniverse` builder | Recovery effect catalogs and reconciliation operations independently owned | D5 |
| `content.execution_universes: ExecutionUniverseContent` | `ExecutionUniverses` builders | Exact capability version, supported versions, attributes, trust domains, regions, directed pairs and profile features | D5 |
| `content.relation_snapshots: RelationSnapshotSlots` | Exact SchemaRef decoder + typed `RelationCatalogSnapshot<T>::new` | Each None/Some state, schema pin, revision, checksum and typed canonical bytes | D5 |
| `contract_assets[]: ContractAsset` reached from resolver component/export refs | D5 module-set shape + `check_contract` | TypedContentRef, canonical contract bytes and ComponentContractDigest; fresh semantics under the D5 context | D3 then D6 |
| `ir_assets[]: IrAsset` reached from caller/target refs | D5 module-set shape + `SemanticIrDocument::check` + Phase 3C | TypedContentRef, exact canonical bytes/digest, ownership and dataflow facts | D3 then D7 |
| `resolver_dependency_manifests[].content: PortableResolverManifest` | `StaticComputationResolutionSnapshotBuilder`, `resolve_computation_leaf` | Caller/context, component inventory, per-source occurrence, target/export pairs, direct and transitive sets, Option state | D2 then D8 |
| `compiler_semantics_manifests[]: CompilerSemanticsManifest` | Existing D4 current-profile validator | Exact current source/rule/compiler tuple remains the D0-D4 support premise; D5-D8 does not reinterpret it | D4 |
| `structural_types[]` and `root_export_assertions[]` | Existing D0-D4 typed/reference/content gates only | Their registered bytes remain pinned; no new D5-D8 Root assertion or structural witness capability is issued | D0-D4; D9-D10 intentionally absent |

The cache identity for SchemaAssets and compiled module content is exact,
length-delimited input material. It is an optimization after D0-D4, not an
alternate identity or a way to skip per-bundle D0-D4 validation.
