# Evaluation, basis, status and typed contradiction

**PROPOSED — NOT REGISTERED.** Existing four ProofVerdicts and authority(NotEstablished) remain distinct. Standalone facts use proof status only; no Established authority is invented.

Local lineage_equality after valid checked/context premises: exact complete identities equal => evaluated/P;unequal => evaluated/U,contradiction=None. Final result_identity is a different predicate: B shortcut=>P;changed pair=>U,even with equal lineage. This reconciles the required local relation with actual reports without promoting a result or pretending two conflicting observations have one key.

Context mismatch: evaluated/U association fact,all12 result facts blocked_by(nonempty set containing that key),placeholder status U;no candidate relation fact is marked evaluated. No twelve evaluated missing-evidence obligations arise. Missing Checked source/application origin or incomplete context projection is an explicit model/capture failure,not an empty successful fragment. Legacy no-C path's report remains12 U and is not converted to an error-valued API in R1.

Exact-body shortcut: evaluated/P results justified by checked inputs,C and B,with reflexive basis. Directional comparator instances are not fabricated as if run. On changed path retain local candidate status plus actual conservative result basis. RE stays in the reused verdict type,but no reachable changed-body branch creates it under the audited implementation. Unsupported future relation/variant fails model validation,not a new synthetic RE or fifth verdict.

When the execution comparison returns Err,the existing Err branch actually assigns three U candidates. Their evaluated local predicate means **that provability/fallback judgment was made**,with the shared exact typed dependency error;it does not assert successful execution of the unavailable aspect relations or fabricate separate comparison calls. On Ok,preserve the two actual aspect values and their combination. Context-failed cases do not even reach this branch and remain blocked,not evaluated fallback judgments.

## Required typed basis (new nonidentity semantic payload)

Add an explicit closed `substitution_basis:Opt<SubstitutionBasis>` field to successor Fact;Some exactly for this family,None for unaffected profiles. Basis is required full-field comparison data,not Fact identity or a Scope descendant. It is not a caller-supplied proof.

```text
SubstitutionBasis =
 checked_origin {role:OperandRole,validation_stage:Stage,validator_facts:Set<FactKey>}
 | association {partition:ContextRolePartition}
 | body_equality {equal:Bool,old:ContractContents,replacement:ContractContents}
 | changed_pair_floor {whole_body_fact:FactKey,verdict:Unproven}
 | local_relation {case:LocalRelationCase,decision:ClosedRelationDecision}
 | reflexive_result {whole_body_fact:FactKey,checked_inputs:Set<FactKey>}
 | conservative_result {floor:FactKey,candidate:FactKey}
 | aspect_combination {feasibility:FactKey,security:FactKey,decision:ProofVerdict}
 | cross_fold {results:Map<SubstitutionDimension,FactKey>}
 | blocked_result {blockers:NonemptySet<FactKey>}
ClosedRelationDecision = proven | unproven {basis:UnprovenBasis}
 | violation {witness:SubstitutionWitness}
UnprovenBasis = lineage_differs | io_not_in_exact_subset
 | effect_reason:existing EffectUnprovenReason
 | resource_reason:existing ResourceCompatibilityUnprovenReason
 | flow_reason:existing FlowCompatibilityUnprovenReason
 | partitions_differ | usage_reason:existing UsageUnprovenReason
 | recovery_nonidentical | recovery_bindings_differ
 | execution_aspects:existing ExecutionCompatibility
 | dependency_error:{domain:resource|flow|usage|recovery|execution,kind:DependencyErrorKind}
```

DependencyErrorKind is exactly the finite stable literal error-kind constants in the respective frozen `crates/af-*/src/error.rs` files at2497b30 (12 resource,11 flow,11 usage,13 recovery,22 execution). Domain-tagged imports include their full literal sets,not a free string;the [draft](phase-04c1b4c4r1-normative-amendment-draft.md) requires exact materialization/content binding at R2. Unknown kinds fail UnsupportedModel. In the current dispatcher only resource_requirement_strengthened/flow_contract_relaxed are mapped from Err to V;other Err=>U. A known V without structured source witness is IncompleteFacts until captured at the earlier actual decision branch;never parse error.detail/reason to reconstruct it.

The Bool/body relation above is a validator-observed comparison result,not an editable approval flag. Repeated basis/status must agree. The full actual typed operands remain in Scope/coordinate;basis does not replace them or introduce report recursion into Scope.

## Closed contradiction/witness contract

SubstitutionWitness variants: io_added_input{full new port,old inputs};io_missing_output{full old port,new outputs};failure_extra{full new nominal failure,old row};resource_added_port{full new requirement,old row};resource_attribute_strengthened{port,full new attribute pair,old attributes,type};flow_dropped_restriction{full label,old/new sets};flow_added_sink{kind,full sink,old/new sets};flow_authority_raised{old/new instruction authority};usage_ceiling_increase{phase,meter,full unit/aggregation,old/new upper};recovery_clause_removed{old/new canonical clause sequences,removed exact clause};execution_constraint{aspect,typed closed differing field,full old/new requirements,violating coordinate};aggregate{violating_fact:FactKey,witness:nonaggregate SubstitutionWitness}.

Execution differing field is closed capability(version/attributes),quantity(kind),sandbox,trust_domain,execution_regions,residency_regions,runtime_profile,implementation_abis;V is allowed only for actual V conditions in the matrix/source. Runtime-profile/ABI difference alone stays U,no V witness. Combined witness allowed only if its own combined candidate is V;U-over-V inner rule is retained. All V facts require the exact witness;aggregate witness is bounded/nonrecursive and must match a supporting violating fact. Effects/lineage unsupported subsets remain U and do not acquire countermodels. This is an explicit proposed TypedContradiction extension,not a current Report field or implemented witness generator.
