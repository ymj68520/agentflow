"""G2 reviewed allocation after collision census; registration, NOT capture."""
BASELINE='4d3bb59031795d31ecfccf29e3d9f2d89a7d9ce8'
OWNER='agentflow.root-export-checker@4'
SHAPE='agentflow.g01.report-shape@3'
OBLIGATION='agentflow.g01.obligation@3'
MODEL='agentflow.computation.proof-report.model@3'
HISTORICAL='agentflow.computation.proof-report.model@1'
SOURCES={
 'NG':('agentflow.component.native-recovery-guard.semantics@1','schema_language','spec/component/native-recovery-guard.md','IR-free local native guard subject, seven predicates, exact algorithm traces, witnesses and restricted imports.'),
 'OS':('agentflow.computation.proof-obligation.semantics@3','schema_language','spec/computation/proof-obligation-semantics-v3.md','Non-root native fragment, input/output separation and unchanged root/substitution authority.'),
 'RM':(MODEL,'closed_table','spec/tables/proof-report-model-v3.md','Complete native successor graph and exact historical selected-import descriptors.'),
 'PP':('agentflow.portable.proof.inputs.semantics@4','schema_language','spec/portable/portable-proof-inputs-v4.md','Exact current-only native staging tuple and denial at every capability boundary.'),
}
CASES='canonical_source context_association action_sources action_bounds guard_inputs guard_containment guard_evaluation'.split()
NATIVE_NAMES=['af-contract::'+n for n in ['ActionGuardCompatibility','GuardCompatibilityBasis','TypedGuardCountermodel','GuardInputAssignment','GuardCountermodelReachability']]
CLAUSES=[c for c in CASES if c!='context_association']
BRANCHES='missing_entries missing_usage unisolated_forward same_atom_bounds_failure action_upper_overflow missing_port schema_unresolved type_mismatch zero_potential frozen_containment typed_countermodel no_countermodel'.split()
ALLOCATIONS=[
 ('NG','NG-SUBJECT','native-subject','Native guard subjects retain complete compiler, canonical declaration and bytes, actual nine-member context and validated provenance without inventing Checked contracts.', ['AF-CONTRACT-0022','AF-SEMANTIC-0198']),
 ('NG','NG-ROSTER','native-roster','The native_recovery_guard family has exactly seven ordered cases, six non-context clauses and contract_validation stage; its preflights do not extend the root roster.', ['AF-SEMANTIC-0199','AF-SEMANTIC-0229']),
 ('NG','NG-CONTEXT','native-context','Analysis and canonical_validation retain the actual same context association and exact ordered role partition; equal contents do not prove object identity.', ['AF-SEMANTIC-0202','AF-SEMANTIC-0233']),
 ('NG','NG-ORIGIN','native-origin','IR-free native origins and typed coordinates identify the complete declaration, real action binding, effect and member occurrences; observations do not create origins.', ['AF-SEMANTIC-0200','AF-SEMANTIC-0210']),
 ('NG','NG-SOURCES','native-sources','Full AtomKey forward and action queries preserve both executed resolutions and exact usage lookup without converting unknown or absent into pure, unconditional or zero.', ['AF-CONTRACT-0022','AF-SEMANTIC-0206']),
 ('NG','NG-BOUNDS','native-bounds','Native bounds preserve declared and effective ceilings, consulted peers, checked arithmetic order and all five typed failure payloads without root multipliers.', ['AF-CONTRACT-0021','AF-CONTRACT-0022']),
 ('NG','NG-INPUTS','native-inputs','Both eager port results retain closed full payloads while the first forward-before-action failure controls the native verdict before the zero shortcut.', ['AF-CONTRACT-0022']),
 ('NG','NG-CONTAINMENT','native-containment','Only an actual frozen containment call yields its Boolean fact: true is Proven and false is Unproven; skipped calls are not invented.', ['AF-CONTRACT-0022']),
 ('NG','NG-EVALUATION','native-evaluation','Native evaluation preserves the eleven-field record, four native bases, twelve selected branches and exact legacy verdict without reason parsing.', ['AF-CONTRACT-0022','AF-SEMANTIC-0211']),
 ('NG','NG-WITNESS','native-witness','Typed countermodels retain all fourteen fields and three-field assignments; structural violations use exact non-countermodel witnesses and local reachability never proves root reachability.', ['AF-CONTRACT-0022','AF-SEMANTIC-0222']),
 ('NG','NG-LINKS','native-links','Native prerequisites, typed references, supporting facts and all consumers retain applicability, original stage and origin; observed false containment is not upgraded to Proven.', ['AF-SEMANTIC-0206','AF-SEMANTIC-0207','AF-SEMANTIC-0208','AF-SEMANTIC-0209']),
 ('NG','NG-OWNERSHIP','native-imports','Selected historical imports bind each exact owner, name, full source and recursively closed descriptor; unknown owners and same-name latest fallbacks reject.', ['AF-SEMANTIC-0214','AF-SEMANTIC-0242']),
 ('OS','NG-FRAGMENT','native-fragment','Native input Scope excludes generated results and witnesses; the independent non-root fragment and separate finite witness DAG cannot grant root CompleteView.', ['AF-SEMANTIC-0197','AF-SEMANTIC-0203','AF-SEMANTIC-0229']),
 ('RM','NG-DESCRIPTORS','native-descriptors','Every native descriptor is concretely closed, source-pinned, ranked, width-exact and bounded, including all parent variants and imported recursive dependencies.', ['AF-SEMANTIC-0214','AF-SEMANTIC-0215','AF-SEMANTIC-0216','AF-SEMANTIC-0217','AF-SEMANTIC-0218','AF-SEMANTIC-0221']),
 ('PP','NG-MIGRATION','native-migration','The distinct current native tuple binds all complete registered content, applied attribution, selected sources and closed tables; frozen old editions reject without relabelling.', ['AF-SEMANTIC-0223','AF-SEMANTIC-0225','AF-SEMANTIC-0226','AF-SEMANTIC-0228']),
 ('PP','NG-CONFORMANCE','native-staging-denial','Registered shape admission never implies capture, support or activation; direct, portable and private RootExport issuance deny the incomplete native edition.', ['AF-SEMANTIC-0212','AF-SEMANTIC-0213','AF-SEMANTIC-0244']),
]
RULES=[dict(source=s,draft=d,anchor=a,statement=t,dependencies=deps,rule_id=f'AF-SEMANTIC-{n:04}',case_id=f'g01-native-guard-staging/rule-{n:04}') for n,(s,d,a,t,deps) in enumerate(ALLOCATIONS,245)]

# Concrete logical definitions. Generic authoring syntax is fully resolved by
# the static materializer, never interpreted from caller input at runtime.
DEFINITIONS='''
CanonicalNativeContractDeclaration = {contract:ComponentContract,canonical_bytes:Bytes}
NativeGuardContextRole = analysis:unit | canonical_validation:unit
NativeGuardContextPartition = NonemptySeq<NonemptySeq<NativeGuardContextRole>>
NativeRecoveryGuardSubject = {owner_compiler:CompleteCompiler,declaration:CanonicalNativeContractDeclaration,context:ContractProofContextContents,context_roles:NativeGuardContextPartition}
NativeEffectRowKind = closed:unit | open_known:unit | unknown:unit
NativeKnownEffectRowKind = closed:unit | open_known:unit
NativeEffectOccurrence = {ordinal:U64,entry:EffectEntry}
NativeUsageOccurrence = {ordinal:U64,binding:RecoveryUsageBinding}
NativeEffectResolution = not_queried:unit | absent:unit | present:NativeEffectOccurrence
NativeUsageResolution = not_queried:unit | absent:unit | present:NativeUsageOccurrence
NativeCeilingResolution = not_queried:unit | absent:unit | present:OccurrenceCeiling
NativeGuardOccurrence = {action_binding_ordinal:U64,action_identity:RecoveryActionIdentity,effect_row_kind:NativeEffectRowKind,forward:NativeEffectResolution,action:NativeEffectResolution,usage:NativeUsageResolution}
NativeGuardRole = forward:unit | action:unit
NativeGuardMemberPath = subject:unit | binding_identity {ordinal:U64} | effect_guard {row_kind:NativeKnownEffectRowKind,entry_ordinal:U64} | input_type {input_ordinal:U64,port_no:U32} | usage_profile {ordinal:U64}
NativeGuardCoordinateClause = context_association:unit | native:NativeGuardClause
NativeGuardProfile = native_recovery_guard:unit
NativeGuardMultiplyStep = forward_times_activation:unit | partial_times_attempt:unit
NativeBoundFailure = same_atom_product_overflow {binding_ordinal:U64,activation_upper:OccurrenceCeiling,attempt_upper:OccurrenceCeiling} | same_atom_sum_overflow {processed_binding_ordinals:Seq<U64>,before:OccurrenceCeiling,next:OccurrenceCeiling} | same_atom_divisor_overflow {sum:OccurrenceCeiling} | same_atom_lower_conflict {declared_bounds:OccurrenceBounds,sum:OccurrenceCeiling,divisor:OccurrenceCeiling,effective_upper:OccurrenceCeiling} | action_upper_overflow {step:NativeGuardMultiplyStep,left:OccurrenceCeiling,right:OccurrenceCeiling}
NativePeerBinding = {binding_ordinal:U64,binding:RecoveryActionBinding,usage:NativeUsageResolution}
NativeUnisolatedCause = foreign_forward {peer:NativePeerBinding} | missing_peer_usage {peer:NativePeerBinding}
NativeBoundsOutcome = ready:unit | unisolated:NativeUnisolatedCause | typed_failure:NativeBoundFailure
NativeBoundTrace = {declared_forward_bounds:OccurrenceBounds,consulted_peer_bindings:Seq<NativePeerBinding>,effective_upper:NativeCeilingResolution,forward_times_activation:NativeCeilingResolution,partial_times_attempt:NativeCeilingResolution,action_upper:NativeCeilingResolution,outcome:NativeBoundsOutcome}
NativeSchemaFailureStep = resolve:unit | expand:unit
NativeGuardPortResult = unconditional_not_required:unit | valid {port:ComponentInputPort,guard:EffectGuardInputEquals,resolved:SchemaEntry,expanded:StructuralType} | missing_port {guard:EffectGuardInputEquals,input_declarations:Seq<ComponentInputPort>} | unresolved_schema {port:ComponentInputPort,guard:EffectGuardInputEquals,step:NativeSchemaFailureStep} | type_mismatch {port:ComponentInputPort,guard:EffectGuardInputEquals,resolved:SchemaEntry,expanded:StructuralType}
NativeGuardPortResolution = not_queried:unit | absent:unit | present:NativeGuardPortResult
NativeGuardInputTrace = {forward:NativeGuardPortResolution,action:NativeGuardPortResolution,primary_error:Opt<NativeGuardRole>}
NativeGuardPrerequisiteClause = canonical_source:unit | context_association:unit | action_sources_ready:unit | action_bounds_ready:unit | guard_input_valid {role:NativeGuardRole} | containment_outcome {expected:Bool}
NativeGuardNonpropagatedWitness = input_countermodel:ImportedTypedGuardCountermodel | missing_port {guard_role:NativeGuardRole,requested_port:U32,actual_input_declarations:Seq<ComponentInputPort>} | constant_type_mismatch {guard_role:NativeGuardRole,port:ComponentInputPort,type_ref:TypeSchemaRef,expanded_type:StructuralType,constant:PrimitiveConstant} | bound_failure:NativeBoundFailure
NativeGuardWitness = input_countermodel:ImportedTypedGuardCountermodel | missing_port {guard_role:NativeGuardRole,requested_port:U32,actual_input_declarations:Seq<ComponentInputPort>} | constant_type_mismatch {guard_role:NativeGuardRole,port:ComponentInputPort,type_ref:TypeSchemaRef,expanded_type:StructuralType,constant:PrimitiveConstant} | bound_failure:NativeBoundFailure | propagated_failure {cause_fact:FactKey,exact_nonpropagated_witness:NativeGuardNonpropagatedWitness}
NativeGuardBasis = canonical_source {subject:Node<NativeRecoveryGuardSubject>,actual_validation_fact_refs:Set<FactKey>} | context_association {partition:NativeGuardContextPartition} | source_resolution {occurrence:NativeGuardOccurrence} | bounds:NativeBoundTrace | guard_input {role:NativeGuardRole,member_path:NativeGuardMemberPath,actual_closed_port_result:NativeGuardPortResult} | containment {forward_guard:EffectGuard,action_guard:EffectGuard,frozen_result:Bool} | evaluation {native_record:ImportedActionGuardCompatibility,branch:NativeGuardBranch,causes:Set<FactKey>}
NativeGuardFragment = {shape:ModelEdition,profile:NativeGuardProfile,scope:Scope,native_records:Seq<ImportedActionGuardCompatibility>,facts:Map<FactKey,Fact>,prerequisites:Map<PrerequisiteKey,Prerequisite>,obligations:Seq<Obligation>,identity_strings:Seq<Text>,blockers:Map<FactKey,NonemptySet<FactKey>>,contradictions:Set<TypedContradiction>}
'''
