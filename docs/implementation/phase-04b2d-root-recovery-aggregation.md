# Phase 4B.2D — Root Recovery Action and Usage Aggregation

All candidate coordinates and rules in this document are **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED ABI / NOT IMPLEMENTED**.

## D2 decision

The current ConditionalCompensationAction is a node-local projection. Its count is not a root total unless every enclosing multiplicity and branch operator has been applied. D2 freezes two non-interchangeable coordinates:

| Coordinate | Definition |
| --- | --- |
| LocalActionBound | Potential action upper for one activation of one exact CompensationScope node; includes unique repetition factors inside that scope's child, not factors outside it |
| RootActionBound | Potential K-action upper after the complete checked root tree has applied all path factors and branch operators exactly once |

Root aggregation starts from the checked leaf RecoverySourceSummary bindings, not by adding each recursively nested CompScope vector. A local summary remains useful evidence, but is never used as a root total without an explicit coordinate conversion.

## Candidate typed records

The following are illustrative design shapes only. They are not Rust types.

~~~text
ControlMultiplicityIdentity {
  semantic_ir_digest: DigestRefValue,
  control_node: ComputationId,
  kind: Iterate | Retry
}

AppliedMultiplicity {
  identity: ControlMultiplicityIdentity,
  logical_repetitions: OccurrenceBounds,
  physical_attempts: Option<OccurrenceBounds>
}

RecoveryActionSourceIdentity {
  caller_semantic_ir_digest: DigestRefValue,
  source_node: ComputationId,
  source_contract_digest: DigestRefValue,
  forward_entry_ordinal: u64,
  action_identity: RecoveryActionIdentity
}

LocalConditionalAction {
  compensation_scope_node: ComputationId,
  source: RecoveryActionSourceIdentity,
  action_kind: RecoveryActionKind,
  local_upper: OccurrenceCeiling,
  applied_families: Vec<AppliedMultiplicity>,
  guard_facts: TypedGuardPath,
  usage_profile: UsageSignature,
  verdict: ProofVerdict
}

RootConditionalAction {
  root_semantic_ir_digest: DigestRefValue,
  source_owner: CompensationScope(ComputationId) | RootRecovery,
  source: RecoveryActionSourceIdentity,
  action_kind: RecoveryActionKind,
  root_upper: OccurrenceCeiling,
  applied_families: Vec<AppliedMultiplicity>,
  guard_facts: TypedGuardPath,
  recovery_usage: StaticUsageEnvelope,
  verdict: ProofVerdict
}
~~~

Identity is the complete tuple shown; no free-text metadata, boolean proof claim, or source-node-only key is allowed. Vectors are sorted by exact digest, node ID, full EffectAtom/RecoveryActionIdentity, and factor identity. Duplicate exact factor IDs with unequal bounds are a closure Violation, not two factors to multiply.

Both action kinds are retained. Compensation contributes a conditional compensation Effect. Reconciliation remains a distinct K action kind and may have its own action Effect and recovery usage; it is never relabeled as compensation. Receipt classification and actual reconciliation execution remain RequiresEvidence.

## Count construction and factor uniqueness

For one action binding and matched forward occurrence:

~~~text
base = ForwardOccurrenceUpper
local upper = base × each unique Iterate/Retry factor inside the owner scope
             × ActivationUpper × AttemptUpper
root upper = base × each unique Iterate/Retry factor on the root path
             × ActivationUpper × AttemptUpper
~~~

The factor identity is (caller SemanticIrDigest, control node, operator kind). Its logical_repetitions.upper is the logical effect/usage multiplier: RETRY's maximum child attempts or ITERATE's proven maximum total count. Physical retry attempts remain separately described and are not an extra multiplier. ITERATE max_inflight is concurrency only.

An action occurrence list carries all applied factor identities. Conversion from local to root may apply only missing ancestor IDs. A factor already present in the scoped forward occurrence must not be applied again. The checker validates both ID uniqueness and exact agreement between the stored factor and the checked control node.

Bounds use checked arithmetic. Missing/unknown factors cannot be treated as 1. A finite overflow is Violation; an unbounded factor produces an unbounded upper; absent source or binding evidence is Unproven, not zero.

There are two equivalent safe derivation strategies, and an implementation must choose one per value:

1. **Leaf-seed fold:** seed each K action from the forward upper × activation upper × attempt upper, with no control factor. Walk the root once; each Retry/Iterate applies its factor and each other operator applies its branch algebra once.
2. **Tagged-scope conversion:** start from a LocalActionBound or local usage projection with its exact applied_families set. Add only missing ancestor factor IDs and compose each enclosing non-repeat operator once.

Never compute a full RootActionBound and then fold that result through the same root Repeat nodes. Never combine an already family-scaled occurrence with an untagged scalar multiplier. The closed-form root equation above describes the result of the unique fold; it is not an extra post-fold multiplication.

## Operator-specific root algebra

The root fold uses the frozen algebras, not one generic sum or multiplication. It computes upper bounds on possible actions, not exact runtime counts.

| Operator | Root action and K-usage composition |
| --- | --- |
| SEQUENCE | Checked conservative accumulation of possible children; a later child may be skipped, so its lower remains zero. |
| SELECT | Mutually exclusive branch join; for the same action/meter coordinate use the frozen max/join, preserve one-sided entries, and use an exact constant case only with existing proof. |
| PARALLEL all / fail_fast | Checked accumulation of every possible branch. Cancellation does not erase effects or recovery; max_inflight is not a total multiplier. |
| RACE | Checked accumulation of all contestants, including losers. Winner is runtime evidence. |
| ITERATE | Scale by its frozen total-count upper once; unknown foreach count makes finite root proof unavailable. |
| RETRY | Scale by max attempts once; idempotency or K does not narrow logical effects. |
| TIMEOUT | Identity for possible child actions and usage; timeout never discounts effects that may already have occurred. |
| HANDLE | Protected path followed by a mutually exclusive join of possible handlers and the no-handler path, using exact existing handler coordinates. |
| SCOPE | Preserve exact child contributions and resource visibility; no compensation, authority, or budget is invented. |
| COMPENSATION_SCOPE | Establish local accounting ownership; does not multiply by itself or add its child actions a second time. |

Within one branch, two distinct binding occurrences with the same full action atom add if both may execute. Across exclusive SELECT/HANDLE branches, join/max rather than sum. Distinct action atoms retain separate rows. Same atom identity is not occurrence identity.

## Nested CompensationScope ownership

The selected accounting rule is **nearest enclosing CompScope owns the local projection**:

1. Walk the checked ownership tree and assign each RecoveryActionSourceIdentity to the innermost/nearest CompensationScope ancestor.
2. A nested CompScope emits its local action entries once. An outer CompScope may expose them as transitive child contributions, but must not generate a second owned entry for the same action binding.
3. Root aggregation traverses each leaf source/action binding once and applies the full path algebra. It does not sum the per-scope projections.
4. A source with no CompScope ancestor is retained under the typed RootRecovery owner, not dropped.
5. Two source nodes using the same full action atom remain separate contributions. The shared atom affects epsilon-row aggregation, not ownership identity.
6. UNKNOWN receipt, partial application, reconciliation obligation, and recovery ordering stay on the original action source. Tree nesting does not prove commit order or effect application.
7. If tree ownership or exact source/action multiset cannot be reconstructed, return Unproven/closure failure. Never deduplicate by source node alone.

This is a static accounting ownership convention. It does not claim that a runtime recovery dispatcher executes only one level, that the inner scope commits first, or that outer compensation is semantically unnecessary. If later Runtime semantics require independent outer and inner dispatch, that is a new versioned recovery model and must replace this single-owner design before implementation; it cannot be guessed from nesting.

## Same-atom and multi-binding accounting

For each source node, contract digest, and full action atom, sum all K action upper contributions that can co-occur. If action atom equals a Forward atom, include both the Forward occurrence upper and K action upper in the same declared epsilon ceiling check. Never merge the two opportunities merely because atom identity is equal.

At root, the complete declared root epsilon envelope already propagates each source E ceiling through the control algebra. The root K action projection is checked against that root epsilon envelope and the exported epsilon ceiling; it is **not added** to either envelope. This avoids double counting while proving that K actions are actually covered.

## Root Recovery Usage: compare, do not add

For every exact recovery-phase/meter coordinate, use the same one-pass strategy as action counts:

1. At a leaf, scale each matching RecoveryUsageBinding per-attempt meter upper by the control-family-free action count (forward occurrence × activation × attempts), with checked arithmetic.
2. Preserve ActionKind and RecoveryActionSourceIdentity until after the source/action/profile multiset has been checked.
3. Fold those leaf usage contributions through the control tree once, applying each Repeat factor and branch operator once; alternatively start from tagged local usage and add only missing factors.
4. Compare it in the same root-relative coordinate with RootDeclaredUsageEnvelope.recovery.
5. Never compute Root U.recovery + RootKRequiredRecoveryUsage. The latter is a coverage projection of K already checked into the component U envelope, not a second charge.

Missing meter coverage is not finite zero. Unknown meter semantics or mismatched units are Unproven. Finite overflow is Violation. Forward Usage and recovery Usage remain separate; recovery never refunds forward usage. A static usage ceiling is not a budget reservation, ledger balance, price, or runtime reading.

## Independent arithmetic

### Single forward

~~~text
F=1, activation=2, attempts=3, no inner factor
local upper = 1×2×3 = 6
~~~

### Retry outside CompensationScope

~~~text
RETRY(3, COMPENSATION_SCOPE(INVOKE(F=1)))
activation=2, attempts=2
local = 1×2×2 = 4
root   = 4×RetryFactor(3) = 12
~~~

### Retry inside CompensationScope

~~~text
COMPENSATION_SCOPE(RETRY(3, INVOKE(F=1)))
activation=2, attempts=2
local/root = 1×RetryFactor(3)×2×2 = 12
root conversion adds no second Retry factor
~~~

### Iterate outside CompensationScope

~~~text
ITERATE(4, COMPENSATION_SCOPE(INVOKE(F=2)))
activation=1, attempts=2
local = 2×1×2 = 4
root  = 4×IterateFactor(4) = 16
~~~

### Retry × Iterate

~~~text
RETRY(3, ITERATE(4, COMPENSATION_SCOPE(INVOKE(F=2))))
activation=2, attempts=2
local = 2×2×2 = 8
root  = 8×IterateFactor(4)×RetryFactor(3) = 96
~~~

Each factor is identified by its actual control node and kind. These examples do not use max_inflight. If per-attempt Recovery Usage is q=2, the meter results are: single-forward local 12; outer-Retry local 8/root 24; inner-Retry local/root 24; outer-Iterate local 8/root 32; and Retry×Iterate local 16/root 192. These values are still compared against U, never added to it.

## Verdicts and boundaries

* **Proven:** complete binding/profile multiset, valid unique factor identities, checked finite or explicit unbounded arithmetic, operator fold complete, and K coverage fits E/U.
* **Violation:** exact underdeclared E/U, malformed ownership/factor identity, or checked arithmetic overflow.
* **Unproven:** missing source/action/profile, open/unknown row, absent factor/domain mapping, or unsupported control-path aggregation.
* **RequiresEvidence:** concrete receipt, action eligibility, reconciliation result, or actual ordering.

This decision closes the local/root semantic ambiguity. It does not add a public ABI, Root K field, execution command, or runtime recovery contract.
