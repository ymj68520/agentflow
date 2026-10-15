# Registered non-wire typed schema — materialization proposal

**PROPOSED — NOT REGISTERED. All adapters/codecs/getters below: NOT IMPLEMENTED.**

## 1. Registration and interpretation

RM=`agentflow.computation.proof-report.model@1` is a closed normative descriptor table. OS defines its semantic relation. No AFIDL module/ABI number is required because no public transport field/type is added: report bytes are a versioned report-local diagnostic/comparison interface, not a Portable Bundle field. All fields below except reason are exact cross-process RequiredSemantic. Reasons remain presence/type-checked DiagnosticOnly. A future public AFIDL/wire exposure needs separate approval and ABI registration; RM tags must not be called AFIDL assignments.

RM placement is `spec/tables/proof-report-model-v1.md`, a registered closed_table source with valid Markdown rule anchors and one uniquely designated normative JSON descriptor block. This respects the existing non-AFIDL anchor validator; it is not a bare JSON rule source. Extraction/closed-model validation is proposed implementation work, not a change to registry validation.

Proposed tokens: ModelEdition=`agentflow.g01.report-shape@1`; ObligationEdition=`agentflow.g01.obligation@1`; owner=`agentflow.root-export-checker@2`. Descriptor/source/rule bytes participate in full owner identity. Equal version strings alone never establish support.

Notation: product fields below are **all required**, in declared semantic comparison order. `Opt<T>` is explicit None/Some; never missing/null/unknown. `Seq<T>` preserves order; `Set<T>` is unique, typed-sorted; `Map<K,V>` is a Set of {key,value}, unique K and compared by K then full V (conflicting same key rejects). Nonempty adds cardinality>0. IDs Computation/Expression/RunParameter/ValueSlot/ResourceSlot/Port/ContractRef are mutually nominal uint32; positive constraints reuse their original validators. Ordinal/indices/DagRank/work/bytes are checked uint64 (no truncating usize casts). Stage is closed D0..D10. Checked nominal text is NFC UTF8; RawClaim is bytes of exact original UTF8, including invalid semantic spelling, not normalized text. Byte sequence is unsigned lexicographic.

A named existing type means **all** fields/cases of that exact frozen source-defined type, recursively, not its Rust Debug/Display/serde output. Reference domain names below denote actual registered DigestRef and TypeSchemaRef shapes, never free type-name strings. The descriptor closure is finite and source-bound: the17 existing registered AFIDL modules (their complete canonical TypeSchema definitions, not names/digests alone) plus the explicitly listed logical snapshots and new types here. Closure derivation: start at Scope/Fact/Obligation/Prerequisite; follow every product, case, option, collection and named-type edge to primitive terminals; reject any undeclared reachable type, field or case. Record `fields` lists every equality-bearing field in declaration order; it is not an open map. The future RM generator must freeze that closure as actual data and both Python/Rust validate it independently before activation. No reflection fallback or "opaque" case is allowed.

Untrusted assertion descriptors use the deterministic **Raw(T)** companion family: transform each text/string/identifier/digest-spelling scalar position into exact UTF8 bytes, preserving its original nominal field/model tag; keep actual byte fields as bytes, fixed-width native numerics as their exact nominal widths, option tags and actual native enum cases; recursively transform every field/case payload; preserve every Vec as sequence, including bad assertion order. Native set values retain their defined set semantics. Freeze the complete resulting companion descriptors in RM, not a runtime reflection transform. Thus Raw(CompilerClaim),Raw(Context),Raw(binding) retain invalid/NFD compiler labels,paths,pins and names without normalizing them. Checked owner/source subjects use ordinary validated descriptors. Comparing a claimed text scalar to the owner compares exact UTF8 bytes, not raw/checked wrapper tags or normalized text. A truly unknown native model case still fails UnsupportedModel; unknown transport cases fail earlier. This family gives byte-exact claim preservation without an opaque serialized object or a natural-language parser.

## 2. Core types

```text
Scope = {
  edition: ObligationEdition,
  analysis_semantics: OwnerVersion,
  content: CanonicalValueDag,
  subject: ScopeSubject
}
ScopeSubject =
  root_export { caller_ir:DigestRef, top_export:DigestRef,
    compiler:Node<Compiler>, context:Node<Context>,
    resolver_associations:Node<ResolverAssociations>,
    root_provenance:Node<RootProvenance>, associations:ContextPartition,
    assertion_inputs:Node<AssertionGraph> }
| contract_validation { compiler:Node<Compiler>, context:Node<Context>,
    associations:ContextPartition, candidate:Node<RawCandidateContract>,
    old_checked:Opt<Node<CheckedContractContents>> }
| computation { compiler:Node<Compiler>, caller_ir:DigestRef,
    context:Node<Context>, resolver_associations:Node<ResolverAssociations>,
    root_provenance:Node<RootProvenance>, associations:ContextPartition,
    owned_root:ComputationId }

RuleRef = { specification_id:SpecificationId, rule_id:RuleId, anchor:Text }
AssertionPath = Seq<{caller_ir:DigestRef, call_site:ComputationId,
                    target_ir:DigestRef, target_export:Opt<DigestRef>}>
PredicateId = {family:PredicateFamily, case:ClosedPredicateCase, rule:RuleRef}
Origin = {predicate:PredicateId, declared_path:Seq<DeclaredStep>,
          source_ordinal:Opt<Ordinal64>, mapping_ordinal:Opt<Ordinal64>}
DeclaredStep = field {owner:ModelType, field:ClosedField}
             | entry {index:Ordinal64}
             | member {key:TypedValue}
FactKey = {stage:Stage, assertion_path:AssertionPath, kind:Kind,
  dimension:Dimension, source:Source, target:Target, coordinate:Coordinate,
  rule:RuleRef, requirement:Requirement, origin:Origin}
ObligationIdentity = {edition:ObligationEdition, scope:Scope,
  assertion_path:AssertionPath, stage:Stage, kind:Kind, dimension:Dimension,
  source:Source, target:Target, coordinate:Coordinate,
  rule:RuleRef, requirement:Requirement, origin:Origin}

Fact = {key:FactKey, role:Role, evaluation:Evaluation, status:Status,
  prerequisites:Set<PrerequisiteKey>, references:Set<EvidenceRef>,
  primary:Opt<EvidenceRef>, contradiction:Opt<TypedContradiction>,
  cause_fact_keys:Set<FactKey>, companion_evidence_status:Opt<ProofVerdict>,
  observations:NonemptySet<ObservationSite>}
Evaluation = evaluated:unit | blocked_by:NonemptySet<FactKey>
Role = mandatory_preflight | static_dimension | execution_child
     | independent_boundary | supporting_fact
Status = proof:ProofVerdict | authority:AuthorityStatus
ProofVerdict = Proven | Violation | Unproven | RequiresEvidence
AuthorityStatus = NotEstablished
Disposition = missing_static_proof | needs_independent_evidence
            | contradiction_remains | authority_not_established
Obligation = {identity:ObligationIdentity, status:Status,
  disposition:Disposition, prerequisite_links:Set<PrerequisiteKey>,
  references:Set<EvidenceRef>, primary:Opt<EvidenceRef>,
  contradiction:Opt<TypedContradiction>,
  companion_evidence_status:Opt<ProofVerdict>,
  observed_at:NonemptySet<ObservationSite>}
PrerequisiteKey = {edition:ObligationEdition, scope:Scope,
  stage:Stage, predicate:PrerequisitePredicate, source:Source, target:Target,
  coordinate:Coordinate, rule:RuleRef, requirement:Requirement, origin:Origin}
Prerequisite = {key:PrerequisiteKey, applicability:Applicability,
  status:PrerequisiteStatus, supporting_fact:Opt<FactKey>,
  contradiction:Opt<TypedContradiction>, consumers:NonemptySet<ConsumerEdge>}
Applicability = applies | unknown
PrerequisiteStatus = established | missing | requires_evidence | contradicted
ConsumerEdge = {consumer:FactKey, mandatory:Bool}
```

False conditional applicability has no instance; its evaluated-false applicability fact is retained in the fact index. Unknown creates an unresolved explicit premise, never established. Established prerequisite requires exact supporting predicate fact Proven; missing/contradicted/evidence-required support cannot serve a Proven mandatory consumer. A separately originated prerequisite with no existing supporting fact derives one kind25 fact; supporting an existing fact reuses that identity, not a duplicate.

Fact scope is supplied by its enclosing CompleteView. All repeated Identity/Scope editions must agree. Fact.role is derived from PredicateId plus applicability/owner roster, not editable from an arbitrary string or its carrier field. Kinds, dimension, clause/class, coordinates and status domain are checked against that roster. Copy merge requires all semantic fields equal except observations; status outside identity is still mandatory equality.

The roster's status-domain matrix is closed: ordinary static/leaf/control/premise facts use the four existing proof verdicts; execution-child feasibility/security use only the existing CompatibilityVerdict cases Proven/Violation/Unproven (no new RequiresEvidence child aspect); authority predicates use only authority(NotEstablished). For this static-only owner the six root boundaries have exactly body/implementation RequiresEvidence,Scope authority/budget Unproven,runtime/publisher NotEstablished,with runtime companion Some(RequiresEvidence); all other companion fields None. No caller evidence input in this edition can change these boundary statuses. Evidence-consuming boundary validators,if later authorized,need their own registered supported model rather than interpreting an arbitrary Proven flag as authority.

Blocked root static facts retain proof(Unproven), blocked_by=exact failed five-core keys, no invented evidence/countermodel. No unmet obligation derives from a blocked fact. The five keys distinguish assertion-input validity, exact pins, proof context (including resolver premise), compiler identity, subflow dependency. Core failure does not suppress the six independent boundaries.

```text
CompleteView = {shape:ModelEdition, scope:Scope,
 facts:Map<FactKey,Fact>, prerequisites:Map<PrerequisiteKey,Prerequisite>,
 obligations:Seq<Obligation>, identity_strings:Seq<Text>,
 blockers:Map<FactKey,NonemptySet<FactKey>>,
 contradictions:Set<TypedContradiction>}
Fragment = {shape:ModelEdition, profile:contract_validation|computation,
            scope:Scope, facts:Map<FactKey,Fact>, ...same derived collections...}
```

The ellipsis in Fragment is **exactly** prerequisites,obligations,identity_strings,blockers,contradictions with identical types above; not an extension field. Only root_export can inhabit CompleteView. Fragment cannot masquerade as a RootExport report or mint a capability. No `complete=true` caller bit substitutes for validation.

The contract_validation fragment profile covers the18 declared prerequisite predicates and their original supporting checks; it does not relabel every existing ComponentContract/substitution dimension as a RootExport predicate. Their original full typed proof reports remain separately required semantic values in the outer comparison. D6 sources use context/member paths into candidate/old/context content, not fabricated IR/computation IDs or Root binding records. An absent-context/schema query uses the exact context_member coordinate/Request reference; root binding coordinate variants apply only when those actual bindings exist. This profile restriction is closed, not an unknown-dimension fallback. Computation fragments cover the declared leaf/control/recovery predicate roster in the producer map.

## 3. Subjects, coordinates and references

Variant order below supplies ranks starting1. Payload fields compare in written order. Every option's legality is a predicate-specific nonapplicability/declaration rule, not missing-content permission.

```text
Source =
 root {ir:DigestRef, root_node:ComputationId}
| component {caller_ir:DigestRef,node:ComputationId,contract:DigestRef,
             entry_ordinal:Opt<Ordinal64>}
| subflow {caller_ir:DigestRef,call_site:ComputationId,target_ir:DigestRef,
           export_state:ExportState}
| scope {caller_ir:DigestRef,node:ComputationId}
| context {member:Seq<MemberStep>,catalog_role:Opt<ContextCatalogRole>}
| compiler {member:Seq<MemberStep>}
| control {caller_ir:DigestRef,node:ComputationId}
ExportState = provided:DigestRef | absent:unit | unresolved:Request
Target =
 analysis_boundary:unit
| export {digest:DigestRef,port:Opt<PortIdentity>}
| component {digest:DigestRef}
| subflow {ir:DigestRef,export_state:ExportState}
| relation {schema:TypeSchemaRef,revision:Text,content:Bytes}
| proof {key:FactKey}
| unresolved_request {caller_ir:DigestRef,node:ComputationId,request:Request}
Request = contract_ref {id:ContractRefId,declared_pin:RawClaim}
        | digest {role:PinRole,claim:RawClaim}
        | schema {claim:RawClaim}
        | source_lookup {caller_ir:DigestRef,node:ComputationId,declared_role:SourceRole}
        | universe_member {catalog:ContextCatalogRole,member:TypedValue}

Coordinate =
 whole:unit
| input {binding:RootInputPortBinding,export_port:ComponentInputPortIdentity,run_parameter:RunParameterId,
         slot:ValueSlotId,use_site:Opt<RootInputUseSite>}
| output {binding:RootOutputPortBinding,export_port:ComponentOutputPortIdentity,name:RawClaim,
          expression:ExpressionId,path:Seq<MemberStep>}
| resource {binding:RootResourcePortBinding,source_port:PortIdentity,slot:ResourceSlotId,
            export_port:ResourceRequirementPortIdentity,type:TypeSchemaRef,
            attributes:Set<ResourceAttributeConstraint>,aliases:Set<PortIdentity>}
| effect {binding:RootEffectCoordinateBinding,source:RootEffectOccurrenceIdentity,export_atom:EffectAtom,
          export_guard:EffectGuard,bounds:OccurrenceBounds}
| flow {binding:RootFlowCoordinateBinding,source:Opt<FlowSourceRef>,sink:Opt<FlowConstraint>,kind:FlowKind,
        labels:Set<ConfidentialityLabelRef>,taints:Set<TaintRef>,
        partition:Opt<EffectFlowPartition>,ordinal:Ordinal64}
| usage {phase:UsagePhase,meter:UsageMeterRef,unit:UsageUnitRef,
         aggregation:UsageAggregation,coverage_role:UsageCoverageRole,
         source_bound:UsageUpperBound,export_bound:UsageUpperBound}
| recovery {binding:Opt<RootRecoveryCoordinateBinding>,source:RecoveryActionSourceIdentity,action:RecoveryActionIdentity,
            profile:Opt<RecoveryUsageBinding>,factors:Seq<MultiplicityFactor>,
            path:Seq<DeclaredStep>}
| execution {source_node:ComputationId,contract:DigestRef,
             requirements:ExecutionRequirements,aspect:ExecutionAspect}
| dependency {call_site:ComputationId,target:DigestRef,edge_ordinal:Ordinal64}
| control {node:ComputationId,operator:ControlOperator,predicate:PredicateId,
           edge_path:Seq<DeclaredStep>}
| context_member {catalog:ContextCatalogRole,member:TypedValue}
| pin {role:PinRole,expected:ClaimValue,observed:ClaimValue}
ClaimValue = validated {type:ModelType,value:TypedValue}
           | raw {type:ClaimType,bytes:Bytes}

EvidenceRef =
 computation {caller_ir:DigestRef,node:ComputationId}
| digest {role:DigestRole,pin:DigestRef}
| schema {pin:TypeSchemaRef}
| port {owner:DigestRef,direction:PortDirection,number:PortNo}
| operation {operation:EffectOperationRef}
| effect_atom {source:RootEffectOccurrenceIdentity,atom:EffectAtom}
| contract_ref {caller_ir:DigestRef,id:ContractRefId,
                resolved:ExportState}
| control_edge {caller_ir:DigestRef,owner:ComputationId,child:ComputationId,
                path:Seq<DeclaredStep>}
| opcode {caller_ir:DigestRef,node:ComputationId,operator:ControlOperator}
| resolver_difference {case:ResolverDifference,expected:Node<ResolverAssociations>,
                       observed:Node<ResolverAssociations>,target:Opt<DigestRef>}
| fact {key:FactKey}
| scope_member {path:Seq<MemberStep>}
| unresolved_request {request:Request}
MemberStep = field {owner:ModelType,field:ClosedField}
           | member {key:TypedValue} | index {index:Ordinal64}
```

D09's prose says "typed input/output/resource port" as one ref family: these are PortDirection input/output/resource, not three independently ranked outer variants. Its enumeration materializes as13 variants (computation1 through unresolved_request13), with no generic fallback. This preserves the accepted semantic cases.

PortIdentity={owner:DigestRef,direction:PortDirection,number:PortNo}. Repeated coordinate/binding/source fields must agree exactly; explicit expected-versus-observed operands alone may differ by tagged side. Recovery binding=None is only for a source predicate before any export mapping exists, never a missing required mapping. Other root bindings are complete existing records including types,attributes,flow bindings/partitions and all source coordinates; a projection cannot discard them as redundant.

Context/compiler Source is a **scope-relative member path**, never recursively contains Scope. Resolver-difference expected/observed nodes resolve inside the same complete scope DAG. Every evidence Node<T> must be reachable from the complete standalone scope, or the needed typed value must be inline; no detached out-of-band table index. References name evidence objects, not a rule dependency certificate or proof identity.

Derive the full reference set from every source/target/coordinate subject (node, contract, schema, port, operation, atom, edge, declared request, resolver operands). For six root whole analysis boundaries primary=None. Otherwise require a nonempty derived set and primary=typed minimum. Extra refs not derivable from subject facts reject; intentionally retained supporting facts use explicit fact refs. Legacy None on a non-boundary is not silently equivalent.

```text
TypedContradiction =
 pin_mismatch {role:PinRole,expected:ClaimValue,observed:ClaimValue}
| missing_members {universe:TypedValue,required:Set<TypedValue>,missing:NonemptySet<TypedValue>}
| noncontainment {relation:PredicateId,actual:TypedValue,ceiling:TypedValue,
                  witness:TypedValue}
| invalid_mapping {input:TypedValue,offending:NonemptySeq<DeclaredStep>,
                   expected:TypedValue}
| invalid_shape {candidate:ClaimValue,expected_descriptor:ModelType,
                 offending:NonemptySeq<DeclaredStep>}
| guard_countermodel {source:RecoveryActionSourceIdentity,source_guard:EffectGuard,
                      target_guard:EffectGuard,bounds:OccurrenceBounds,
                      countermodel:TypedValue}
| source_mismatch {expected:TypedValue,observed:TypedValue}
| prerequisite_contradiction {key:PrerequisiteKey,witness:TypedValue}
ObservationSite = {report_owner:ObservationOwner,
 field_path:Seq<MemberStep>,exposing_path:AssertionPath,
 exposing_node:Opt<ComputationId>}
ObservationOwner = root_export | contract_validation | leaf | control
                 | root_recovery | portable_projection
```

Contradiction payload ModelTypes are fixed by the predicate descriptor, not an arbitrary caller T. Violation requires the applicable typed counterexample/validation witness; Unproven cannot fabricate one. An old reason-only essential error must be captured at its typed validation branch. All graph links (blockers,causes,prerequisite support/proof targets) form a validated finite dependency DAG; Target.proof may reference only an earlier dependency, never itself or a consumer. Missing, cyclic or ambiguous support rejects. Observation paths may point to report carrier locations but are descriptive edges, not report object references.

## 4. Closed domains and predicate roster

Kind ranks1–25, exactly: assertion_input,exact_pin,proof_context,resolver_provenance,compiler_identity,subflow_dependency,input_mapping,output_producer,failure_containment,effect_containment,resource_mapping,information_flow,usage_coverage,recovery_coverage,execution_source,execution_feasibility,execution_security,body_conformance,implementation_conformance,scope_authority,scope_budget,runtime_authority,publisher_authentication,control_evidence,prerequisite.

Dimension = none(rank0), I1,O2,Phi3,epsilon4,R5,F6,U7,K8,X9, or supporting{predicate:PredicateId}. Supporting cases compare after root dimensions by the closed predicate case text. Role ranks follow section2; they are derived and need not appear in Identity again.

Requirement={clause:RequirementClause,evidence_class:EvidenceClass}.
RequirementClause is a closed tagged sum direct:DirectKind | recovery:RecoveryClause | control:ControlClause | prerequisite:PrerequisitePredicate. DirectKind is precisely kinds1–13 and15–23, in kind-rank order. The other three payload enums are the following ordered lists. The enclosing kind must agree with the clause family and direct payload. This explicit tag avoids conflating identically named control/direct clauses in a context-free codec.
For kinds1–13 and15–23, clause is that kind's tag. Recovery14 clauses in order: source_closure,action_binding,usage_profile,multiplicity,guard_compatibility,epsilon_coverage,recovery_usage_coverage,eligibility,commit_order.
Control24: resolved_leaf,topology,output_interface,selector_totality,effect_envelope,failure_row,resource_provenance,information_flow,usage,retry_safety,child_settlement,forward_history,compensation_coverage,root_export_boundary,unsupported_control,recovery_blocked_outcome.
Prerequisite25: closed_contract_schema,imported_module_closure,schema_universe,positive_unique_ports,failure_pins,effect_operation_membership,resource_universe,effect_resource_relation,flow_catalogs,io_resource_namespaces,additive_meter_units,recovery_usage_exhaustiveness,closed_effect_action_coverage,exact_k_identity,execution_universes,flow_execution_relation,required_dimensions,exact_relation_pins.

EvidenceClass ranks: static_declaration,typed_counterexample_resolution,trusted_body,implementation_attestation,provider_receipt,authority_attestation,publisher_authentication,control_terminal_state,committed_effect_order,recovery_outcome,provider_honoring,application_idempotency. Ordinary violations KEEP the predicate's evidence class; they do not automatically become typed_counterexample_resolution.

Allowed class relation is predicate-specific, not merely clause-wide: static predicates use static_declaration; body/implementation use trusted_body/implementation_attestation; runtime/publisher use authority_attestation/publisher_authentication; Scope authority/budget static_declaration with Unproven; actual owned-child settlement uses control_terminal_state, while static timeout-outcome separation and passive-AWAIT declarations use static_declaration; commit_order uses committed_effect_order; recovery_blocked_outcome uses recovery_outcome; compensation eligibility uses provider_receipt; retry facts distinguish static declarations, provider_honoring and application_idempotency predicates by their actual evidence query. Counterexample_resolution is only an explicitly declared prerequisite asking to resolve a typed contradiction. No unsupported cross-product is accepted.

PredicateFamily = root_preflight | root_dimension | execution_child | root_boundary | leaf | control | root_recovery | prerequisite. ClosedPredicateCase:
- root_preflight: assertion_input,exact_pin,proof_context,compiler_identity,subflow_dependency.
- root_dimension: I,O,Phi,epsilon,R,F,U,K,X.
- execution_child: source,feasibility,security.
- root_boundary: body,implementation,scope_authority,scope_budget,runtime,publisher; subflow-body is leaf/body_conformance under its exact call path.
- prerequisite: the18 clauses above plus conditional-applicability queries epsilon_nonempty and flow_constraints_require_relation (support predicates, not new arbitrary missing categories).
- leaf/control/root_recovery: the exact named predicates in each row of the [92-site map](phase-04c1b4c2-producer-adaptation-map.md); dynamic wrappers keep the original PredicateId, not the wrapper function name. Each map row binds one of the closed clauses above and its frozen governing RuleRef. No site line number is a runtime case.
The RM roster must include both successful and non-successful branch variants and their mandatory source prerequisites; proof aggregation priority is never a substitute for this binding.

Other closed domains:
- ContextCatalogRole = modules,type_schemas,effect_catalogs,resource_schemas,resource_descriptors,flow_labels,flow_taints,flow_sinks,usage_meters,recovery_effects,reconciliation_operations,execution_capabilities,trust_domains,regions,region_compatibility,runtime_profiles,effect_resource_relation,recovery_usage_relation,flow_execution_relation,resolver.
- PinRole = caller_ir,root_ir,export_contract,component_contract,subflow_ir,subflow_export,type_schema,relation,compiler.
- DigestRole = semantic_ir,component_contract,type_schema; only existing registered domain combinations.
- PortDirection = input,output,resource.
- SourceRole = component,subflow,await,owned_child,recovery_action,execution_child.
- ClaimType = pin,identifier,compiler,context,contract,assertion_mapping.
- UsageCoverageRole = forward_envelope,recovery_envelope,action_recovery_coverage.
- ExecutionAspect = source_coverage,feasibility,security.
- ResolverDifference = component_pins,target_absent,direct_dependencies,target_export,additional_targets,association_identity (six current branches, all operands preserved).
- ControlOperator is the existing13-opcode closed Semantic IR domain; payloads retain actual mode/bound/selector fields, not a newly invented opcode.

## 5. Complete content closure (not opaque snapshots)

Compiler={analysis_semantics_version:Text,registry_content:Bytes,rule_registry_content:Bytes,specifications:Seq<SpecificationContentIdentity>,rules:Seq<RuleContentIdentity>,support_manifest:FullSupportManifest}; each SpecificationContentIdentity={specification_id,logical_path,content_sha256,content:Bytes}; each RuleContentIdentity={rule_id,normative_source,source_anchor,short_statement}. These vectors retain original assertion sequence order when untrusted; owner-generated projections use their declared canonical order. All bytes, including complete registry rows not repeated in the four-field rule projection, are retained.

FullSupportManifest={normative_sources:Set<SpecificationContentIdentity>,closed_tables:{hash_domains:Bytes,abi_numbers:Bytes,rule_areas:Bytes,afidl_primitives:Bytes},selected_specifications:Set<SpecificationId>,applied_rules:Set<RuleContentIdentity>,semantic_roles:Seq<SemanticRoleEntry>,attribution:AttributionProfile,report_shape:ModelEdition,obligation_edition:ObligationEdition}. SemanticRoleEntry and AttributionProfile reuse the exact full existing Portable AFIDL types, including all role/dependency/status fields; only the full_content_revalidation case is supported. The owner has all46 source bindings here,20 selected specifications and138 applied rules as specified in the edition plan. This complete support view is recaptured from existing Portable manifest fields/implementation-owned source bytes; it is a new non-wire report model field, NOT a new AFIDL field or a hidden 46->20 content reduction. Owner support is data built from frozen sources, not a report-dependent self-checksum.

CompilerClaim is the closed sum legacy_v1{analysis_semantics_version,registry_content,rule_registry_content,specifications,rules with their original exact types} | successor_v2:Compiler. Assertion input descriptors use Raw(CompilerClaim), rather than demanding a stale v1 claim grow an invented support_manifest or canonical NFC labels. The original full old claim can therefore yield a new-owner compiler-preflight mismatch with its bytes intact. Scope's owner compiler must be successor_v2 and exact supported; these are different tagged comparison operands.

Context={compiled_modules:Map<SpecificationId,CompiledModuleContents>,type_schemas:Map<LogicalTypeIdentity,CanonicalTypeSchema>,effect_catalogs:Map<SpecificationId,Set<Text>>,resource_types:{schemas:Map<LogicalTypeIdentity,CanonicalTypeSchema>,descriptors:Map<LogicalTypeIdentity,ResourceTypeDescriptor>},flow_catalogs:{confidentiality:Set<ConfidentialityLabelRef>,taints:Set<TaintRef>,sinks:Map<EffectOperationRef,Bool>},usage_meters:Map<UsageMeterRef,{meter:UsageMeterRef,unit:UsageUnitRef,aggregation:UsageAggregation}>,recovery_operations:{effects:Map<SpecificationId,Set<Text>>,reconciliation_operations:Set<{catalog:Text,name:Text}>},execution_universes:ExecutionUniverseContents,relation_snapshots:Seq<RelationSnapshotContent>,resolver_dependencies:ResolverDependencyManifest}.

CompiledModuleContents={specification_id,types:Seq<CanonicalTypeSchema>}; CanonicalTypeSchema uses the exact complete TypeSchema machine grammar in AFIDL1 §§6–7, including nominal identities, field/case/value numbers and full type expressions. It is not just TypeSchemaRef. ResourceTypeDescriptor={specification_id,type_name,schema_hex,recognized_attributes:Set<Text>}. ExecutionUniverseContents={capabilities:Map<ExecutionCapabilityRef,{capability,exact_version:Text,supported_versions:Set<Text>,attributes:Set<{key:Text,value:Text}>}>,trust_domains:Set<TrustDomainRef>,regions:Set<RegionRef>,region_compatibility:Set<{from:RegionRef,to:RegionRef}>,runtime_profiles:Map<RuntimeProfileRef,{profile,family:Text,features:Set<Text>}>}. Preserve empty recognized universes and direction of compatibility edges.

RelationSnapshotContent={pin:{schema:TypeSchemaRef,revision:Text,canonical_content_checksum:{algorithm:Text,hex:Text}},canonical_content:Bytes}. ResolverDependencyManifest={component_contract_digests:Seq<DigestRef>,subflow_targets:Seq<{semantic_ir_digest:DigestRef,direct_dependencies:Seq<DigestRef>,exported_contract_digest:Opt<DigestRef>}>}. ResolverAssociations adds exact caller_ir,source_node/request and checked target/export association, complete component/target content, direct/transitive dependencies; its per-association record is {caller_ir,source_node,request:Request,component_contract:Opt<Node<CheckedContractContents>>,target_ir:Opt<Node<CheckedIrContents>>,target_export:ExportState,direct_dependencies:Set<DigestRef>,transitive_dependencies:Set<DigestRef>}. Assert the resolved variant's required fields; no arbitrary all-None resolved row.

RootProvenance={source_ir:DigestRef,root_node:ComputationId,root_resolver_identity:Node<ResolverAssociations>,root_context:Node<Context>,root_k_context:Node<Context>,source_ir_contents:Node<CheckedIrContents>}. CheckedContractContents and CheckedIrContents contain the complete existing ComponentContract/SemanticIrDocument values AND exact canonical input bytes/pins, not a Checked wrapper serialized as authority. RawCandidateContract uses the same named-field descriptor but retains raw claims/order before validation.

AssertionGraph nodes have all fields of Raw(RootExportAssertion) (root_export.rs317–329), all Raw(binding) records (183–313) with their full declared widths/options, and all recursively nested target assertion identities. Seven mapping vectors and dependency vectors are sequences; invalid order is not repaired. Full RootInputUseSite, structural type AST, flow mapping partition/bindings, action/source/usage profiles, execution requirements and transitive targets use their exact source-defined logical models with Raw companions at untrusted text positions. All imported AFIDL payloads are full source-pinned values. A serializer containing only a count, checksum, StructuralType display string without an invertible declared grammar, or omitted universe is not this model.

ContextPartition=Seq<NonemptySeq<ContextRole>>: each applicable role exactly once; members sorted numeric; classes sorted minimum. Root roles analysis0,checked_export1,snapshot2,root3,root_K4; contract roles analysis0,candidate1,old_checked2 iff present; computation roles analysis0,snapshot1,leaves2,root3,root_K4. Equal contents can occupy different classes. Never encode native addresses. This metadata is a comparison of already checked object associations, not evidence that supplied contents share an arena.

## 6. Canonical Value DAG and order

TypedValue={model_type:ModelType,value:CanonicalValueDag}, with exactly one root of that model_type, is a value under a **closed ModelType descriptor**, materialized as the following finite DAG. Its self-contained DAG is finite; it never contains Scope/Fact/Obligation/report as a descendant. ModelType is an exact member of RM's closure, bound to source edition/type name and full descriptor; no arbitrary string creates a new model.

ModelType={owner:SpecificationId,name:Text}: owner is an exact source-pinned imported AFIDL specification or RM@1; name must occur in its registered closed descriptor closure. Full descriptors are obtained from the complete owner/source bundle in Scope, not serialized recursively inside each ModelType. ClosedField/ClosedCase are exact text tokens present in that ModelType's approved ordered descriptor. Unknown owner/name/field/case is UnsupportedModel. This is type reference by full supported content, not an unbound local table index or arbitrary model-name string.

DagNode={type:ModelType,body:NodeBody}.
NodeBody = scalar:ScalarValue
 | record:Seq<{field:ClosedField,value:DagRank}>
 | variant:{case:ClosedCase,payload:DagRank}
 | sequence:Seq<DagRank> | set:Seq<DagRank>
 | option_none:unit | option_some:DagRank.
ScalarValue = unit:unit | null:unit | bool:Bool | int32:I32 | uint32:U32
 | int64:I64 | uint64:U64 | decimal:ExactDecimal | float64:FiniteFloat64
 | string:NfcText | bytes:Bytes | timestamp:MicrosI64 | duration:MicrosI64.
CanonicalValueDag={nodes:Seq<DagNode>,roots:NonemptySeq<DagRank>}.
Node<T> is a typed DagRank whose descriptor is exactly T. Roots are in ScopeSubject field order, duplicates allowed for equal-valued roots. Value nodes cannot contain Fact/Obligation/report/Scope as descendants. Identity's Scope is a complete graph; every used member/evidence node must be reachable from its subject roots. Object graphs that need extra evidence values add them as declared input roots in their registered subject, never as final-report roots.

Validate acyclicity, widths/descriptors and semantic sets first. Intern equal typed values bottom-up (no hash-only equality); node height0 for scalars/empty containers, otherwise1+maximum child height. Sort each height by descriptor type identity then nonrecursive payload and ordered child ranks, assign contiguous ranks starting0; child ranks strictly smaller. Sets sort by their domain's typed key/full payload; sequence order is preserved. Require no duplicates,unreachable nodes,alternate node numbering or redundant equal nodes at decode. A duplicated equal subtree and shared subtree yield the same minimal DAG. No recursion through final report or digest fixed point exists.

Identity order is scope,assertion_path,stage,derived role,dimension,kind,source,source_ordinal,mapping_ordinal,target,coordinate,rule,requirement,origin. Ordinals are repeated from validated Origin/Source solely for this key and MUST agree. Edition precedes Scope for differing editions (normalizers admit one edition only). Source/Target/Coordinate/Evidence/contradiction/observation variants use declaration ranks; other records field order above; Requirement clauses use kind-family rank then listed clause order. Rule order=(specification_id,rule_id,anchor). Scalar types are nominal; mathematical numeric comparison, exact rational Decimal, finite float after frozen zero canonicalization, bool false<true, unsigned UTF8/bytes, None<Some, lexicographic tuple/sequence prefix first. Map/set compare sorted complete logical values; DAG ranks are canonical representation, **not a shortcut for semantic numeric order**. Comparisons memoize typed node pairs and charge work.

Set uniqueness: FactKey/PrerequisiteKey by complete key; obligations by complete identity; evidence/observation/contradictions by complete typed value. Conflicting key payload rejects. Semantic sequences preserve duplicates. Unknown fields/cases/type descriptors/role/domain combinations reject; new additions require new model/obligation/analysis editions under0214/0221, not an ignore-unknown option.
