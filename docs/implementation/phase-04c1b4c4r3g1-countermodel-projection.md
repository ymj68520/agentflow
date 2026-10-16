# Exact fourteen-field native witness projection

All proposed bindings here are DRAFT — NOT REGISTERED. Historical field/type declarations below are actual registered model@1/native source facts. Proposed selected import preserves every field and its exact historical nominal identity;it does not manufacture an equivalent-looking new same-name type.

|Native field|Exact/logical type and option|Source getter|Canonical representation (future)|Normative role|Independent Expected rule|
|---|---|---|---|---|---|
|source_contract|ComponentContract;Complete canonical declaration,NOT CheckedContractContents;all13 fields|actual witness.source_contract()|existing exact ComponentContract machine value;no invented Checked pin|native source subject|equal admitted canonical source;not count/hash-only|
|source_component|ComponentIdentity;required catalogue AND name|source_component()|complete NFC catalog/name record|lineage association only|equals source_contract.component_identity|
|recovery_action|RecoveryActionIdentity;full forward atom,action kind,full action atom|recovery_action()|full exact AtomKey-bearing values|action identity|matches exact unique binding,not operation-only|
|forward_entry_ordinal|usize → nominal logical u64;required;0 is a valid ordinal|forward_entry_ordinal()|mathematical unsigned64;native list indexing verified|Forward occurrence|selected Closed/Open.known entry equals action.forward_effect|
|action_entry_ordinal|usize → nominal logical u64;required;not binding ordinal|action_entry_ordinal()|mathematical unsigned64;not decimal text order|Action E occurrence|selected entry equals action.action_effect|
|forward_guard|EffectGuard;closed Unconditional/InputEquals payload|forward_guard()|exact enum/payload,including primitive constant|Forward possibility|equals actual selected Forward guard|
|declared_action_guard|EffectGuard;closed Unconditional/InputEquals payload|declared_action_guard()|exact enum/payload|public action ceiling guard|equals selected action E guard|
|forward_bounds|OccurrenceBounds;lower u64,upper Finite(u64)/Unbounded|forward_bounds()|full exact lower/upper,not upper alone|effective Forward bounds passed to countermodel builder|retain recovered seed upper plus declared lower;separately retain original entry bounds in subject/trace|
|activation_upper|OccurrenceCeiling;Finite(u64) or Unbounded|activation_upper()|closed ceiling,zero distinct from absent|component-local K multiplier|equals matching RecoveryUsageBinding activation upper|
|attempt_upper|OccurrenceCeiling;Finite(u64) or Unbounded|attempt_upper()|closed ceiling|component-local attempts multiplier|equals matching usage attempts upper|
|action_upper|OccurrenceCeiling;Finite(u64) or Unbounded|action_upper()|checked mathematical ceiling product|derived native possibility bound|effective Forward×activation×attempt;no Root factors;positive for a countermodel|
|assignments|Vec<GuardInputAssignment>;sequence of full port_no/type_ref/value triples|assignments()|port-sorted native sequence;port u32,full schema pin,exact PrimitiveConstant|typed counterexample values|domain-valid assignments;Forward true and action false;no count/Debug-only projection|
|reachability|GuardCountermodelReachability;sole variant WellTypedComponentInputDomain|reachability()|closed native variant with exact imported owner|LOCAL input-domain reachability|only local primitive domain;never Root-global reachability|
|related_rule_id|&'static str → RuleId;required exact AF-CONTRACT-0022|related_rule_id()|exact nominal RuleId token|governing substantive rule|no fabricated rule or parsed reason|

Owner for each original field is af-contract::TypedGuardCountermodel as declared by exact model@1 source;the future selected view retains that owner provenance and maps recursively referenced types explicitly. The complete original14-field value is separate from proposed NativeGuardSubject/Occurrence metadata;in particular it has NO recovery_action_binding_ordinal or context field. The latter must be retained by the subject/coordinate,not falsely added to the native witness's field count.

The full positive/negative/unknown diagnostic and negative14-field Actual are in audit.json. GuardInputAssignment retains all3 fields and exact type pin. ActionGuardCompatibility retains all11 fields,including explicit None/Some and all four basis variants;reason remains diagnostic after the other10 fields/structural causes are preserved. No exists/count/hash/Debug representation substitutes for a witness.

forward_bounds is the effective builder argument:recovery_forward_upper may replace the declared upper before the builder is called. With the frozen unique same-atom entry,the guards are identical and frozen containment normally returns before constructing a countermodel;do not invent a reachable same-atom countermodel merely to exhibit a changed effective bound. Both declared and effective bounds remain explicitly available in the proposed source/derivation trace.

A guard V does not always carry this14-field assignment witness. MissingPort/TypeMismatch/arithmetic failures currently return V/InsufficientEvidence with countermodel=None. Preserve those exact legacy outcomes via DIFFERENT proposed closed structural witnesses;never fake a typed input assignment or downgrade/upgrade the checker. Structural input failure is not a proof that a well-formed guard implication has a false valuation. Failed frozen containment alone remains U until an actual valid countermodel is available.

Derived assignment values belong to the witness payload,not the input Scope. The draft distinguishes Scope input DAG ranks from a self-contained witness payload's ranks;each DAG is closed/reachable within its own roots. No generated witness/report is inserted into Scope to make identity self-referential. This is a proposed native-profile ownership/representation clarification,NOT a G02 implementation or a retroactive rewrite of legacy Root rules.
