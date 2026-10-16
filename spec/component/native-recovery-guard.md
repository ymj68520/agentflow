# agentflow.component.native-recovery-guard.semantics@1

Status: normative, registered non-wire semantics; G2 registration only. Producers, collector, normalizer and complete Scope remain NotImplemented; positive RootExport is NotAuthorized. Symbolic owner references below mean the exact new model/report/obligation/analysis tuple in portable-proof-inputs-v4.md. The concrete table in proof-report-model-v3.md resolves every logical shorthand and generic. No AFIDL, ABI, ACE1 grammar or HashV1 change.

## NG-SUBJECT — source and authority

MUST distinguish raw declaration,canonical validated declaration,native diagnostic fact,complete Contract Checked,root fact and RootExport capability. The native profile MUST accept only an actually canonicalized complete declaration under the actual native context;canonicalization failure MUST return the genuine earlier rejection,not an empty successful fragment. Canonical-valid/full-check-negative inputs MUST remain eligible for non-authoritative native diagnostic evaluation without inventing CheckedContractContents or a Checked-only pin.

The source unit MUST be the complete canonical declaration,not an invocation. Canonicalization sorts full action identities and rejects duplicates. Source admission MUST retain genuine validation provenance;when original raw validation history is unavailable,a future adapter may explicitly perform fresh instrumented validation of the supplied canonical declaration under its own context. It MUST NOT pretend to reconstruct the old raw occurrence/history. The guard subject consumes that explicitly admitted canonical input. Captured validator/support facts remain outside input Scope;no result/fact/report is a Scope descendant.

Append ScopeSubject.native_recovery_guard:NativeRecoveryGuardSubject only in the newly allocated native edition. Do not mutate the frozen contract_validation/root_export Scope variants or fabricate resolver fields for the nine-member native contract context. The subject declaration is an admitted INPUT to guard evaluation,not a stored guard result. Canonicalization provenance may be freshly established as stated above;no unverifiable historical raw-origin claim is implied by canonical bytes.

Registered logical input definitions (fully monomorphized in the closed table):

```text
CanonicalNativeContractDeclaration = {contract:ComponentContract,canonical_bytes:Bytes}
NativeGuardContextRole = analysis | canonical_validation
NativeGuardContextPartition = Seq<NonemptySeq<NativeGuardContextRole>>
NativeRecoveryGuardSubject = {
 owner_compiler:CompleteCompiler,
 declaration:CanonicalNativeContractDeclaration,
 context:ContractProofContextContents,
 context_roles:NativeGuardContextPartition
}
NativeEffectRowKind = closed | open_known | unknown
NativeEffectOccurrence = {ordinal:U64,entry:EffectEntry}
NativeUsageOccurrence = {ordinal:U64,binding:RecoveryUsageBinding}
NativeResolution<T> = not_queried:unit | absent:unit | present:T
NativeGuardOccurrence = {
 action_binding_ordinal:U64,action_identity:RecoveryActionIdentity,
 effect_row_kind:NativeEffectRowKind,
 forward:NativeResolution<NativeEffectOccurrence>,
 action:NativeResolution<NativeEffectOccurrence>,
 usage:NativeResolution<NativeUsageOccurrence>
}
NativeGuardRole = forward | action
NativeGuardMemberPath = subject:unit
 | binding_identity {ordinal:U64}
 | effect_guard {row_kind:closed|open_known,entry_ordinal:U64}
 | input_type {input_ordinal:U64,port_no:U32}
 | usage_profile {ordinal:U64}
```

Every T,closed scalar union and imported logical type MUST be concretely materialized with exact fields/widths/ranks before registration support is claimed. These are registered logical authoring definitions concretized in the model table,not a new ACE1 grammar,opaque serialized object or runtime Rust reflection plan. Native-only binding lifetimes remain local;future full Scope requires G02 and no native address is serialized.

## NG-ROSTER — exact closed profile

Registered PredicateFamily/Profile native_recovery_guard;append in the separately allocated native edition. It MUST NOT change any existing family's meaning. Exactly seven cases in this registered order:

|Rank|Case|Kind|Requirement clause|Role|Applicability|Registration clause label|
|---|---|---|---|---|---|---|
|0|canonical_source|native_recovery_guard_evidence|native_guard:canonical_source|mandatory_preflight|admitted canonical declaration|NG-SUBJECT|
|1|context_association|proof_context|direct:proof_context|mandatory_preflight|admitted validation/analysis binding|NG-CONTEXT|
|2|action_sources|native_recovery_guard_evidence|native_guard:action_sources|supporting_fact|each actual canonical action binding|NG-SOURCES|
|3|action_bounds|native_recovery_guard_evidence|native_guard:action_bounds|supporting_fact|current E entries and usage resolved|NG-BOUNDS|
|4|guard_inputs|native_recovery_guard_evidence|native_guard:guard_inputs|supporting_fact|bounds available;exact forward/action parameter|NG-INPUTS|
|5|guard_containment|native_recovery_guard_evidence|native_guard:guard_containment|supporting_fact|inputs valid and action upper nonzero;actual frozen call|NG-CONTAINMENT|
|6|guard_evaluation|native_recovery_guard_evidence|native_guard:guard_evaluation|supporting_fact|each actual native ActionGuardCompatibility result|NG-EVALUATION|

All use existing stage contract_validation (original native/D6 meaning),EvidenceClass.static_declaration,and Dimension.supporting(the exact PredicateId),not Root's nine or substitution's twelve dimensions. Append registered Kind native_recovery_guard_evidence and RequirementClause.native_guard(NativeGuardClause);NativeGuardClause contains exactly the six non-context cases above in written order. Context uses the existing proof_context kind/direct clause with an explicit new native-family binding. New governing RuleRefs are allocated in the clause mapping below;AF-CONTRACT-0022 and actual related schema/bounds rules remain frozen substantive dependencies,not replaced by invented current RuleIds.

canonical_source/context are mandatory only inside this native subject,NOT extra root core checks. guard_inputs is exactly parameterized by forward/action,not two new arbitrary cases. Cases not reached by the native algorithm MUST NOT be fabricated as successful comparisons. Empty action-binding input yields no per-binding guard results;it does not create authority or imply a guessed complete root roster. The native fragment MUST carry explicit admitted subject/applicability information rather than present an unqualified empty collection as complete capture.

## NG-CONTEXT — actual native association

MUST retain all nine actual ContractProofContextContents members,without root resolver/caller placeholders. Both roles appear once;analysis and canonical_validation MUST associate with the same actually used context object. Pointer identity is checked in-process where the binding is reused;equivalent context bytes do not prove aliasing. Revalidation under a different context creates a fresh verified binding,not a claim that the original wrapper belonged to it. Missing source/context role fails explicitly. Full content/partition portability remains pending G02.

The registered role ranks are analysis0,canonical_validation1. Every applicable role occurs exactly once;partition classes are nonempty,members use numeric role order,and classes use minimum-member order. For an admitted native binding the two roles are in the same class. Addresses are not class identifiers. All new case/variant/field ranks append in the new tuple's explicitly declared order;record semantic order and existing ACE1 unsigned-UTF8 output-field order remain distinct. U64 ordinals compare mathematically (2 before10),not as decimal strings.

## NG-ORIGIN — IR-free typed members

Append Source.native_guard_subject{subject:Node<NativeRecoveryGuardSubject>} and Source.native_guard_binding{subject,occurrence:NativeGuardOccurrence}. Append corresponding Target.native_guard_subject{subject} and Target.native_guard_binding{subject,action_binding_ordinal,action_identity}. Append Coordinate.native_guard{subject,clause:NativeGuardCoordinateClause,occurrence:Opt<NativeGuardOccurrence>,guard_role:Opt<NativeGuardRole>,member:NativeGuardMemberPath};NativeGuardCoordinateClause is the exact closed sum context_association:unit|native:NativeGuardClause.

Subject-level None is legal only for canonical/context cases;all per-binding cases require the real occurrence. Unknown/not_queried/absent/present MUST be distinct. A present ordinal0 MUST never stand for absence.

Origins MUST use the actual registered subject.declaration.contract.recovery_action_bindings[i] input application;input-check parameter applications further address the real identity.forward_effect/action_effect reference field. Coordinates preserve the resolved E guard's actual Closed/Open.known selection and entry ordinal. EffectRow variants MUST NOT be encoded as fictitious ClosedFields. Subject source/context cases use their actual declaration/context application. Source ordinal is the actual binding ordinal;no arbitrary mapping ordinal is added. Case/role and declared path distinguish applications;assertion_path is known empty. Same complete declaration/context/application denotes one native originating fact;different real declaration/binding/role remains distinct. Runtime call counts,time,addresses,Probe IDs and Root observation paths MUST NOT be origins.

Append EvidenceRef.native_guard_member{subject,member:NativeGuardMemberPath}. Derive all actual references and typed minimum;RuleRef is not EvidenceRef. Root/consumer edges MUST retain original native stage/source/origin and all observers.

## NG-SOURCES — resolution before arbitration

MUST resolve Forward and action E entries using full AtomKey,not operation-only. Use exactly the selected canonical entries vector. Both E lookups occur before the current missing-entry return. Matching usage uses full action identity. Preserve each actual resolution result and every input/occurrence;do not replace absent values by pure/unconditional or infer lookup status from reason text. Missing source/usage yields the existing native U/InsufficientEvidence result,not V or P.

## NG-BOUNDS — local checked arithmetic

MUST retain declared Forward bounds,all consulted same-atom peer bindings/profiles and actual checked-operation order. For different atoms,effective upper is declared upper. For the same atom,derive sum of same-atom activation×attempt multipliers,divisor1+sum,then effective upper:Unbounded ceiling→Unbounded;finite ceiling/Unbounded divisor→0;otherwise floor(C/divisor). Foreign Forward contributions or missing peer profiles preserve the native unknown/unisolated result. Effective upper below declared lower or checked finite overflow preserves the actual premise-failure V,not a fabricated assignment witness.

NativeBoundFailure MUST be closed:

```text
same_atom_product_overflow {binding_ordinal,activation_upper,attempt_upper}
same_atom_sum_overflow {processed_binding_ordinals,before,next}
same_atom_divisor_overflow {sum}
same_atom_lower_conflict {declared_bounds,sum,divisor,effective_upper}
action_upper_overflow {step:forward_times_activation|partial_times_attempt,left,right}
```

All ordinals are U64;all ceilings are the exact imported Finite(U64)|Unbounded values;all lists retain actual evaluation order. Unknown/unisolated outcomes separately retain the actual foreign/missing peer binding,not an open String error kind. Zero×anything=0 precedes Unbounded handling. The second multiplication MUST be not_queried if the first failed. Root Retry/Iterate factors MUST NOT be inserted here. Actual typed causes MUST be captured inside recovery_forward_upper before its Err(()) reduction,not reconstructed from a later sentence.

## NG-INPUTS — eager checks and first-error selection

MUST retain both actually evaluated forward/action validate_guard_port results,including when the loop returns its first failure. Each guarded port retains exact input/member path,TypeSchemaRef,resolution/expansion step,expanded type and constant. Use closed outcomes unconditional_not_required|valid|MissingPort|UnresolvedSchema|TypeMismatch with exact typed source payload. MissingPort and TypeMismatch are structural V,UnresolvedSchema is U. A canonicalization failure before this stage remains earlier rejection,not an executed guard-input fact.

MUST preserve forward-before-action error selection,not max all helper verdicts. Thus a later independently observed failure is retained without changing the reported native result chosen by the frozen algorithm. Unqueried and evaluated outcomes MUST not be conflated. Port validation precedes zero-action shortcut;zero does not excuse a missing/mistyped port.


The registered closed port-result payloads are:unconditional_not_required:unit;valid{port:ComponentInputPort,guard:EffectGuardInputEquals,resolved:SchemaEntry,expanded:StructuralType};missing_port{guard:EffectGuardInputEquals,input_declarations:Seq<ComponentInputPort>};unresolved_schema{port:ComponentInputPort,guard:EffectGuardInputEquals,step:resolve|expand};type_mismatch{port:ComponentInputPort,guard:EffectGuardInputEquals,resolved:SchemaEntry,expanded:StructuralType}. SchemaEntry is the actual imported four-field specification_id/type_name/digest/body record,not a guessed schema summary. Source context retains the entire actual universe;unresolved cases never invent a resolved entry. These result tags correspond to the three frozen private port-error alternatives,with explicit successful/unconditional cases. Capture before the map_err reduction;do not parse SemanticError text.

## NG-CONTAINMENT — frozen local relation

Only an actual guard_contained(F,A) call emits this case. True→P;false→U,not V. Zero action potential skips that call and does not fabricate a successful containment result. Preserve F/A full payload and the Boolean outcome. This supporting case is not the entire native evaluation,which can later prove a typed countermodel. Failed countermodel construction remains U,even when a stronger future algorithm might decide the relation.

## NG-EVALUATION — retain the real reported native judgment

MUST keep all11 actual ActionGuardCompatibility fields,all four exact native basis variants,and original native verdict with a separately typed selected-branch trace. Evaluation here is the native algorithm's judgment,including an evaluated unavailable/premise-failure fallback;it does not falsely claim the guard-containment kernel ran. No native branch produces RequiresEvidence. Case guard_containment remains separate.

NativeGuardBranch is closed:missing_entries|missing_usage|unisolated_forward|same_atom_bounds_failure|action_upper_overflow|missing_port|schema_unresolved|type_mismatch|zero_potential|frozen_containment|typed_countermodel|no_countermodel. Branches with several detailed typed causes retain the specific payload from earlier hooks. No reason parser selects them.

Append Fact.native_guard_basis:Opt<NativeGuardBasis>,Some exactly for this family,None for unaffected profiles. Existing substitution_basis stays None for native facts. Registered closed basis:

```text
canonical_source {subject,actual_validation_fact_refs}
context_association {partition:NativeGuardContextPartition}
source_resolution {occurrence}
bounds {declared_forward_bounds,consulted_peer_bindings,effective_upper,
        action_upper,outcome:ready|unisolated|typed_failure}
guard_input {role,member_path,actual_closed_port_result}
containment {forward_guard,action_guard,frozen_result:Bool}
evaluation {native_record:ImportedActionGuardCompatibility,
            branch:NativeGuardBranch,causes:Set<FactRef>}
```

The association is verified by an internal lifetime-bound binding,not a caller Boolean or serialized address;the canonical basis contains the actual partition,not a runtime proof object. Fields have the concrete subject/reference/ceiling/member types defined above;registration MUST resolve every generic/shorthand and may not introduce opaque placeholders. Whole-contract status is separately obtained by the actual check_contract,not by max-folding all captured helper facts. Capturing an unselected helper V MUST NOT silently strengthen the legacy primary outcome.

## NG-WITNESS — exact native and structural causes

A typed-countermodel branch MUST retain all14 historical TypedGuardCountermodel fields and all3 fields of every GuardInputAssignment using the exact selected owner/import. Forward evaluates true,action false,input types match exact pins,component-local potential is positive,and reachability is exactly WellTypedComponentInputDomain. No Root-global reachability is implied. Native14 has no action-binding ordinal or context field;those belong to the subject/coordinate. Every original field differs semantically when its complete typed value differs,except diagnostic reason on the outer record.

Append TypedContradiction.native_guard with a closed NativeGuardWitness sum:

```text
input_countermodel:ImportedTypedGuardCountermodel
missing_port {guard_role,requested_port,actual_input_declarations}
constant_type_mismatch {guard_role,port,type_ref,expanded_type,constant}
bound_failure:NativeBoundFailure
propagated_failure {cause_fact,exact_nonpropagated_witness}
```

No recursive/self propagated witness is allowed. Main V from structural premise failure MUST use the actual structural cause,not input_countermodel. Main U or failed containment MUST NOT be upgraded to V. Unresolved schemas have no fabricated concrete type or assignment. Missing required V witness returns IncompleteFacts for capture without rewriting the checker verdict. Diagnostic reason is retained only after all decision-bearing structured content exists.

## NG-LINKS — premises and observations

The future closed prerequisite model MUST append PrerequisitePredicate.native_guard:NativeGuardPrerequisiteClause. That clause is exactly canonical_source|context_association|action_sources_ready|action_bounds_ready|guard_input_valid{role:NativeGuardRole}|containment_outcome{expected:Bool}. Reuse the existing exact Prerequisite/ConsumerEdge fields,with original contract_validation stage and native subject/member origin. Do not force a native bound/input premise into exact_k_identity or one of the eighteen historical premise meanings.

Ready/source/bounds/input premises are established only by the actual applicable supporting result and its full structured basis. containment_outcome is specifically an observed Boolean query:it is established only when a genuine evaluated guard_containment fact's frozen_result equals expected. A false outcome may support the countermodel branch without changing the supporting compatibility fact's Unproven into Proven. Zero/unavailable paths have no invented containment instance or requirement. A result's witness-validation condition is checked directly on its exact payload;no self-referential prerequisite or cause link is allowed.

Native P MUST have genuine canonical/context/source/bound/input premises applicable to its selected branch;zero potential makes containment not-applicable,not established by an invented call. A countermodel branch retains the actual failed-containment support and typed assignment validation. Every source fact has its original stage,path,rule,requirement,source,target and all consumers. Prerequisite links may name genuine supporting local facts in T3;complete FactKey/Scope association remains T4/T5. Do not manufacture original validator facts from the absence of missing strings.

A root consumer MUST verify exact Checked-source body/context/pin association before using an eligible positive native source. Negative native diagnostics carry no Checked token;normal root resolution rejects them earlier. One native origin may have multiple root observations;each actual root call-site/source and root aggregate keeps a different declared identity. Native local bounds and root multipliers remain separate. Root F01/F02 rules are unchanged;F03/F04 and G02/G03 are not closed by this profile.

Append ObservationOwner.native_recovery_guard and an exact native non-root fragment/profile,not CompleteView. A registered future complete NativeGuardFragment has shape,profile,scope,native_records:Seq<ImportedActionGuardCompatibility>,facts,prerequisites,obligations,identity_strings,blockers and contradictions with the same fully typed containers used by existing non-root fragments. native_records and basis/witness payloads are OUTPUTS outside Scope. Observation paths may address native_records[i].verdict/basis/countermodel or the corresponding fact fields under their exact registered owners. They are output observations,not native originating paths. Copies must agree on semantic record fields;diagnostic reason alone never supplies a predicate or witness. Complete fragment normalization/Scope is T4/T5,not G1/T3;T3 would expose only the lifetime-bound local seed view with explicit PendingScope and local references,never a falsely complete canonical fragment.

## NG-OWNERSHIP — closed selected imports

MUST bind historical model@1 owner/name/full source contents and exact native declarations to a reviewed future selected import map for the five guard roots and their full recursive dependencies (including the exact three private port-error alternatives where used). No broad historical-owner whitelist or same-name alias is permitted. Native enum ranks follow native declarations,not alphabetic JSON object keys. Complete14-field projection is defined in countermodel-projection.md;unknown owners,names,fields,variants and inconsistent tags reject.

The new native ScopeSubject consumes only inputs. Generated assignment values and diagnostic records are payloads outside Scope. A witness's self-contained value DAG has its OWN root/rank space and exact type;only input-source references refer to the native subject. Require reachability/minimality in each respective DAG,not inclusion of a generated witness value as a Scope input. This native-profile clarification is explicitly registered here;it does not implement G02 or rewrite old Root canonical semantics.

## NG-MIGRATION / NG-CONFORMANCE

New family/case/kind/clause/Source/Target/Coordinate/EvidenceRef/Contradiction/basis/ScopeSubject/profile/ranks use the distinct model/report/obligation/analysis complete identities allocated in portable-proof-inputs-v4.md. Keep current sources and all old tuple meanings byte-exact. The exact new identities and RuleIds are allocated below; old complete tuples remain frozen. Registered,Supported,Activated/release remain distinct controlled states;selected type admission or successful native diagnostics MUST NOT issue RootExport capability.

Before later producer closure require independent declaration-derived field Expected,all seven cases and applicable native branches,14-field and parent-field mutations,real source/admission/context inputs,full per-site/per-field coverage and capability denial. The48 G1 vectors are design-only Not Executed,not a substitute. G2 registration and later R3 implementation are separate authorizations;T4–T8 remain separate. Registration changes metadata/current-only admission and exact compiler content only; it does not implement producers, normalization or positive capability.


## Registration interpretation

NG-* headings above are clause names, not RuleIds. The mapping below supplies the governing registered RuleId. Logical NativeResolution<T> is concretely NativeEffectResolution, NativeUsageResolution and NativeCeilingResolution, plus NativeGuardPortResolution; no generic remains in the runtime catalogue. NativeGuardBranch ranks follow the twelve-case order. Complete parent shapes, NativeGuardBasis, five NativeBoundFailure payloads, port outcomes, nonpropagated witness and fragment are in the exact registered model table. NativeBoundTrace also retains both multiplication resolutions, and NativeGuardInputTrace preserves both actual input outcomes and the selected primary error. No metadata acceptance proves the semantic constraints or object association. G02 complete Scope remains pending. Future-facing MUST obligations bind future implementations, not claims that hooks or normalization exist today.

<a id="native-subject"></a>

## AF-SEMANTIC-0245

MUST: Native guard subjects retain complete compiler, canonical declaration and bytes, actual nine-member context and validated provenance without inventing Checked contracts.

Direct normative dependencies: AF-CONTRACT-0022, AF-SEMANTIC-0198.

Draft mapping: NG-SUBJECT. Registration conformance: g01-native-guard-staging/rule-0245. Full producer/normalizer/oracle semantics remain unimplemented and are not proved by reference coverage.


<a id="native-roster"></a>

## AF-SEMANTIC-0246

MUST: The native_recovery_guard family has exactly seven ordered cases, six non-context clauses and contract_validation stage; its preflights do not extend the root roster.

Direct normative dependencies: AF-SEMANTIC-0199, AF-SEMANTIC-0229.

Draft mapping: NG-ROSTER. Registration conformance: g01-native-guard-staging/rule-0246. Full producer/normalizer/oracle semantics remain unimplemented and are not proved by reference coverage.


<a id="native-context"></a>

## AF-SEMANTIC-0247

MUST: Analysis and canonical_validation retain the actual same context association and exact ordered role partition; equal contents do not prove object identity.

Direct normative dependencies: AF-SEMANTIC-0202, AF-SEMANTIC-0233.

Draft mapping: NG-CONTEXT. Registration conformance: g01-native-guard-staging/rule-0247. Full producer/normalizer/oracle semantics remain unimplemented and are not proved by reference coverage.


<a id="native-origin"></a>

## AF-SEMANTIC-0248

MUST: IR-free native origins and typed coordinates identify the complete declaration, real action binding, effect and member occurrences; observations do not create origins.

Direct normative dependencies: AF-SEMANTIC-0200, AF-SEMANTIC-0210.

Draft mapping: NG-ORIGIN. Registration conformance: g01-native-guard-staging/rule-0248. Full producer/normalizer/oracle semantics remain unimplemented and are not proved by reference coverage.


<a id="native-sources"></a>

## AF-SEMANTIC-0249

MUST: Full AtomKey forward and action queries preserve both executed resolutions and exact usage lookup without converting unknown or absent into pure, unconditional or zero.

Direct normative dependencies: AF-CONTRACT-0022, AF-SEMANTIC-0206.

Draft mapping: NG-SOURCES. Registration conformance: g01-native-guard-staging/rule-0249. Full producer/normalizer/oracle semantics remain unimplemented and are not proved by reference coverage.


<a id="native-bounds"></a>

## AF-SEMANTIC-0250

MUST: Native bounds preserve declared and effective ceilings, consulted peers, checked arithmetic order and all five typed failure payloads without root multipliers.

Direct normative dependencies: AF-CONTRACT-0021, AF-CONTRACT-0022.

Draft mapping: NG-BOUNDS. Registration conformance: g01-native-guard-staging/rule-0250. Full producer/normalizer/oracle semantics remain unimplemented and are not proved by reference coverage.


<a id="native-inputs"></a>

## AF-SEMANTIC-0251

MUST: Both eager port results retain closed full payloads while the first forward-before-action failure controls the native verdict before the zero shortcut.

Direct normative dependencies: AF-CONTRACT-0022.

Draft mapping: NG-INPUTS. Registration conformance: g01-native-guard-staging/rule-0251. Full producer/normalizer/oracle semantics remain unimplemented and are not proved by reference coverage.


<a id="native-containment"></a>

## AF-SEMANTIC-0252

MUST: Only an actual frozen containment call yields its Boolean fact: true is Proven and false is Unproven; skipped calls are not invented.

Direct normative dependencies: AF-CONTRACT-0022.

Draft mapping: NG-CONTAINMENT. Registration conformance: g01-native-guard-staging/rule-0252. Full producer/normalizer/oracle semantics remain unimplemented and are not proved by reference coverage.


<a id="native-evaluation"></a>

## AF-SEMANTIC-0253

MUST: Native evaluation preserves the eleven-field record, four native bases, twelve selected branches and exact legacy verdict without reason parsing.

Direct normative dependencies: AF-CONTRACT-0022, AF-SEMANTIC-0211.

Draft mapping: NG-EVALUATION. Registration conformance: g01-native-guard-staging/rule-0253. Full producer/normalizer/oracle semantics remain unimplemented and are not proved by reference coverage.


<a id="native-witness"></a>

## AF-SEMANTIC-0254

MUST: Typed countermodels retain all fourteen fields and three-field assignments; structural violations use exact non-countermodel witnesses and local reachability never proves root reachability.

Direct normative dependencies: AF-CONTRACT-0022, AF-SEMANTIC-0222.

Draft mapping: NG-WITNESS. Registration conformance: g01-native-guard-staging/rule-0254. Full producer/normalizer/oracle semantics remain unimplemented and are not proved by reference coverage.


<a id="native-links"></a>

## AF-SEMANTIC-0255

MUST: Native prerequisites, typed references, supporting facts and all consumers retain applicability, original stage and origin; observed false containment is not upgraded to Proven.

Direct normative dependencies: AF-SEMANTIC-0206, AF-SEMANTIC-0207, AF-SEMANTIC-0208, AF-SEMANTIC-0209.

Draft mapping: NG-LINKS. Registration conformance: g01-native-guard-staging/rule-0255. Full producer/normalizer/oracle semantics remain unimplemented and are not proved by reference coverage.


<a id="native-imports"></a>

## AF-SEMANTIC-0256

MUST: Selected historical imports bind each exact owner, name, full source and recursively closed descriptor; unknown owners and same-name latest fallbacks reject.

Direct normative dependencies: AF-SEMANTIC-0214, AF-SEMANTIC-0242.

Draft mapping: NG-OWNERSHIP. Registration conformance: g01-native-guard-staging/rule-0256. Full producer/normalizer/oracle semantics remain unimplemented and are not proved by reference coverage.



