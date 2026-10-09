# Phase 4B.2C — Full Computation Contract Closure Audit

Status: audit complete; a P1 cross-dimension guard-design gap was reproduced.
No production, ABI, corpus Expected, or registry change was made. The audit was
performed at baseline `d96dac047dfca8ae8cc129e0861ff6b66e284219`.

## Executive determination

No P0 undercount was confirmed. A P1 ambiguity exists in the K↔epsilon
action-guard relation: the checked E row can be narrower than the conditional
K action projection. The latter is conservatively wider and remains
RequiresEvidence, so the probe does not demonstrate a false-negative total
action bound; it demonstrates that the current proof does not establish
action-E guard compatibility. Level 1 structural closure is accepted for a
`CheckedIrDocument`. Level 2 is accepted for root E/U envelopes and source
facts when K actions remain a separate conditional sidecar. Level 3 is
**partial** until guard semantics and root-total K aggregation are frozen.
Level 4 is not established. Level 5 remains outside this static layer.

### P1 — AUDIT-10: K action Effect guard relation is not proven

**Affected:** `COMPENSATION_SCOPE`, RecoveryActionBinding ↔ ComponentContract
`epsilon`; `crates/af-computation/src/control.rs` in
`derive_compensation_scope` and `conditional_compensation_bounds`. The
ComponentContract Recovery validator checks action-atom presence but does not
relate the action row's EffectGuard to the Forward/K trigger.

**Frozen rule:** `spec/effect/effect-signature.md#effect-guards` says an
`input_equals` guard narrows when an Effect may occur. The action identity in
`spec/component/component-contract.md#component-contract-recovery-bindings`
contains the complete atom but no action guard field. The CompScope analysis
derives a new conditional action from the Forward occurrence.

**Reproduction:** start from the committed
`computation-scope-closure/compensation/action-bound-06` input. Keep the
Forward natural-idempotency Effect unconditional. On the second (`none`)
entry, change both fixture mirrors `children[0].effects.entries[1].guard` and
`children[0].effect_entries[1].guard` to:

```json
{"input_equals":{"port":1,"value":0}}
```

Keep the exact K binding, activation/attempt bounds, per-attempt Usage, and
resource mapping. The checked fixture reaches
the production API and returns overall `requires_evidence`, not Violation.
The declared action E entry remains guarded by `input_equals`; the derived
conditional action's `potential_effect.guard` is `unconditional`, copied from
the unconditional Forward occurrence. K has no condition that prevents the
action when the input is not zero. The E/K coverage check compares occurrence
upper bounds only and does not compare guards.

**Consequence:** the declared E action row is conditional while the separate
K-derived potential action is unconditional. The wider action sidecar is
conservative, and the overall result remains RequiresEvidence due to receipt
eligibility, so this probe alone does not establish a false-negative total
effect bound. But the analyzer neither proves that E's guard covers the K
action nor records a dedicated K↔E guard obligation. An export checker that
reads only `epsilon` would miss the separate conditional action; a checker
that unions both must define how guards combine.

**Narrow resolution (not implemented here):** formally decide whether the
declared action-E guard is an execution ceiling or whether K's Forward guard
alone defines possible action eligibility. Then either prove guard containment
with the frozen guard algebra and retain Unproven/Violation on mismatch, or
define a conservative composition for the separate action sidecar. Add
same-guard, wider-E, narrower-E and different-guard cases. Do not silently
widen E or invent a K guard.

**Blocking stage:** root cross-dimension/export compatibility design. This
audit does not patch production.

Two P1 design issues prevent an unqualified “full exported computation
contract” claim:

1. There is no formal Root Derived Summary ↔ exported ComponentContract
   mapping/proof model. The current `NotExpressedInSemanticIr` / `Unproven`
   result is correct, but the release boundary has not been designed.
2. `ConditionalCompensationAction.potential_effect` is attached to a
   `COMPENSATION_SCOPE` node and uses occurrence families visible in that
   node's child subtree. An enclosing `RETRY`/`ITERATE` family is added to the
   parent occurrence summary later; no root-wide conditional K-action
   aggregation or explicit “per scope activation” coordinate is exposed.
   Root `epsilon` and `U` still receive their outer multipliers, so this is
   not an observed undercount in those envelopes. But reading the nested
   action upper as a whole-workflow total undercounts; independently
   multiplying without a frozen rule risks double counting. The model needs
   to state the coordinate and aggregation rule before Level 3/root export is
   claimed.

The second issue was reproduced with the committed
`action-bound-06` fixture in an isolated, temporary probe, which was removed
after use:

| Shape | K-derived action at CompScope node | Root epsilon action atom | Root U.recovery |
| --- | ---: | ---: | ---: |
| `RETRY(3, COMPENSATION_SCOPE(INVOKE))` | 9 local | 54 | 54 |
| `COMPENSATION_SCOPE(RETRY(3, INVOKE))` | 27 | 54 | 54 |

The fixture has Forward upper 1, activation upper 3, action attempts upper 3,
per-attempt Recovery Usage 2, and declared action-Effect upper 18. The local
formula is `1 × 3 × 3 = 9`; with the enclosing Retry, total possible actions
are `9 × 3 = 27`, and corresponding Recovery Usage is `27 × 2 = 54`. The
second shape sees Retry in the inner forward occurrence families and reports
27. In the first shape, the CompScope summary reports its local 9 while the
outer Retry carries the factor in the root occurrence/U envelope. This is a
coordinate/API ambiguity, not evidence that root epsilon or U is below the
possible total.

## Five completion levels

| Level | Verdict | Evidence / limit |
| --- | --- | --- |
| 1. Structural Closure | Accepted for checked IR | Phase 3B rejects cycles, multiple parents and unreachable nodes; the root walk checks every declared node exactly once and resolves every reachable leaf or returns an explicit error. |
| 2. Conservative Envelope Soundness | Accepted for `epsilon`, `Phi`, and `U`; K source obligations retained | Existing operator algebras are conservative and checked; nested conditional K-action summary remains node-local, with no root aggregate. |
| 3. Cross-Dimension Consistency | Proven locally; root-wide K-action relation not materialized | Phase 4B.1 checks leaf `epsilon`/`R`/`F`/`K`/`U`/`X`; controls preserve per-child evidence and amplify `epsilon`/`U`. Root action totals need the coordinate decision above. |
| 4. Root Export Compatibility | Not established | No root ComponentContract pin or cross-coordinate mapping exists in Semantic IR. |
| 5. Implementation / Runtime Assurance | Not established; intentionally out of scope | No body-conformance attestation, grant, Worker admission, budget reservation, receipt, or recovery execution is produced. |

## Structural/source audit

`SemanticIrDocument::check` runs `check_ownership` after body validation. The
ownership check constructs the exact per-opcode owned-child graph, rejects
cycles and multiple parents, requires a parentless root, and rejects every
unreachable declaration. `derive_workflow_contract_summary` then independently
walks the checked ownership edges, compares the walk against all declared
computation IDs, resolves leaf nodes in semantic order, derives bottom-up,
compares the derived preorder, and checks equality of known leaf Effect
occurrences/residuals and Recovery source nodes. A failed leaf resolver returns
`LeafResolution`; a closure/accounting mismatch returns `ClosureMismatch`—no
missing source becomes an empty row.

The occurrence comparison clears only the derived control-family annotation;
it retains caller SemanticIrDigest, source contract digest, computation node,
source entry ordinal, full EffectAtom, guard/bounds and resource mapping. Two
uses of one ComponentContract remain different by computation node. The
Recovery closure check compares source computation identities, not a second
reconstruction of every action binding; the checked leaf summary remains the
source of exact K identities and usage bindings.

SELECT's exact constant case changes the Effect envelope chosen by
`combine_effect_signatures`, but does not skip recursive derivation of other
case children. The owned-tree walk resolves them all; inherited proofs and
per-child Recovery/Flow/Resource/X sources remain. Other dimensions (notably
Failure and Recovery) conservatively retain all possible cases. This is safe
but may be less precise than exact branch pruning.

Source accounted for is not behavior independently verified. A CALL_SUBFLOW
whose export resolves but lacks trusted body evidence remains
`RequiresEvidence`; AWAIT's exact signal pin can be Proven while its external
provenance and interaction contract remain Unproven.

## P0–P3 findings

| ID | Severity | Finding | Consequence / next action |
| --- | --- | --- | --- |
| AUDIT-10 | P1 | K-derived conditional action guard may be wider than declared action-E guard; guard compatibility is not proved | Wider sidecar is conservative, but root cross-dimension/export comparison is undefined. Freeze guard semantics and add regressions before Level 3 export claims. |
| AUDIT-01 | P1 | Root export proof model and coordinate map are absent | Level 4 cannot pass. Decide a typed build-time assertion in a separate phase; do not reuse ComponentContract substitution without mapping. |
| AUDIT-02 | P1 | Per-node versus root-total conditional K-action multiplicity is not formally stated or aggregated | Root epsilon/U remain amplified, but a nested action upper can be mistaken for a root total. Freeze local-coordinate semantics and a single root aggregation procedure before export comparison. |
| AUDIT-03 | P2 | No frozen differential case has a nonempty K action under an enclosing Retry/Iterate outside its CompScope | Existing nested Retry/Iterate vectors carry no actions. Add targeted cases only in a later authorized conformance phase. |
| AUDIT-04 | P2 | Same full EffectAtom used as both Forward and compensation action is not covered by the 55-case oracle | The production checker includes Forward upper plus K-derived action upper for row coverage; hand derivation is in the counterexample report. Add a focused regression later. |
| AUDIT-05 | P2 | The 55-case comparator is intentionally projection-scoped, not a full root-summary oracle | It checks closure, Scope/CompScope summaries, selected occurrences, failures, Usage, X and K; it does not compare every Flow/interface/resource/proof field on every nested case. Existing 58/72 and leaf/component suites cover adjacent algebras, not every new cross-product. |
| AUDIT-06 | P2 | No action-specific Flow/X model exists in frozen K | Source-contract F/X and their leaf proofs remain available per child; action-specific constraints cannot be asserted. Keep this Unproven/source-scoped unless the formal model adds typed action bindings. |
| AUDIT-07 | P2 | Root `workflow_outputs` and exported ComponentContract ports have no explicit mapping in the root derived type | Root computation output-interface facts are not a public export proof. Include exact name/port/type mappings in the future assertion model. |
| AUDIT-08 | P3 | Scope authority/budget ceilings, receipts, reconciliation results, dispatch, Worker and actual settlement are runtime/context evidence | These are intentionally absent. Never default absent ceilings to unlimited or present runtime evidence as static Proven. |
| AUDIT-09 | P2 | Closure's `known_recovery_sources_accounted` equality compares source computation IDs, not a second multiset of each K binding/Usage profile | The current control derivation clones the checked leaf RecoverySourceSummary and preserves per-child sources, so no drop was observed; the boolean itself is narrower than action/profile equality. Add a source/action multiset audit vector before claiming that flag proves per-action closure. |

No P0 is confirmed. The action-guard probe is P1 because the exposed
conditional-action sidecar is wider, not narrower, than the declared row; it
does not by itself prove a missed possible Effect. Root E/U repetition
algebras showed no independent undercount in the reviewed paths. Root export,
action guard semantics and root-total conditional K aggregation remain P1
design blockers.

## Gate and freeze summary

The baseline, statistics, all existing 58/72/55 production differential
runners, 21 generators, ABI/digest/AFIDL freeze comparison, and final test
results are recorded in `phase-04b2c-readiness.md`. This phase creates reports
only: no production API, rule registry, corpus Expected, ABI, digest, or AFIDL
content is modified.
