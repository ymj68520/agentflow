# Phase 4B.2E.1 — Root Recovery Implementation

## D2 implementation mapping

`derive_workflow_contract_summary` now creates one `RootRecoverySummary` from
the existing checked workflow root, leaf-resolved source summaries, exact
proof context, and already-derived root Effect/Usage envelopes. It does not
create a second control analyzer or accept a caller-supplied checked summary.
`DerivedWorkflowContractSummary::overall` preserves the separate root
Recovery proofs alongside the pre-existing root and tree-closure proofs.

The non-wire API is in `crates/af-computation/src/root_recovery.rs` and
`control.rs`:

- `LocalActionBound` means one activation of one exact CompensationScope.
- `RootActionBound` means the complete checked root tree.
- `ControlMultiplicityIdentity` is caller SemanticIrDigest + exact control
  node + Iterate/Retry kind; logical repetitions and physical attempts remain
  separate.
- `RootRecoveryAction` retains exact source/target, action kind and full
  RecoveryActionIdentity, Forward/action occurrences, matching full usage
  profile, local/root bounds, guard proof/coordinates, ownership, and typed
  control path.
- `RootRecoverySourceClosureReport` separately reports source nodes, action
  bindings, usage profiles, multiplicity factors, and root action totals.

## One-pass leaf-seed fold

Each checked K action starts from a factor-free Forward seed × activation ×
attempt upper. The root walk applies each checked Iterate/Retry family once,
then composes the resulting action and Recovery-Usage projections with the
existing Effect and Usage operator algebra. It never folds a pre-scaled root
bound through the same Repeat node again. Local bounds are computed in a
separate nearest-owner coordinate; the root fold does not add local summaries.

The production differential includes these exact bounds:

| Case | LocalActionBound | RootActionBound |
| --- | ---: | ---: |
| K01: F=1, A=2, T=3 | 6 | 6 |
| K02: Retry3 outside CompScope, F=1, A=2, T=2 | 4 | 12 |
| K03: Retry3 inside CompScope, F=1, A=2, T=2 | 12 | 12 |
| K04: Iterate4 outside CompScope, F=2, A=1, T=2 | 4 | 16 |
| K05: Iterate4 × Retry3 outside CompScope, F=2, A=2, T=2 | 8 | 96 |

Retry physical attempts are preserved as metadata but not multiplied a second
time. K04 uses a statically known four-item FOREACH with `max_inflight=2`; the
root action count remains 16, not 8.

## Same atom, multiple sources, and owners

For a single full atom used by both Forward and K action, the shared epsilon
row is an aggregate ceiling. The checked same-atom inference computes the
greatest Forward seed compatible with `C >= F × (1 + Σ(A×T))`, retains that
seed separately, and projects Forward+K exactly once for epsilon comparison.
K06 therefore reports K=6 and required epsilon=7; epsilon=7 passes. The
K06-underdeclared companion has a mandatory Forward lower of one with epsilon
ceiling six and is rejected at ComponentContract validation before a checked
leaf or root summary can be created.

K07 retains two distinct source bindings sharing one action atom and aggregates
4+6=10. K08 Sequence gives 5; K09 exclusive Select gives 8; K10 Parallel and
K11 Race each give 7, including the Race loser; K12 keeps protected+handler
possibility. K13/K14 show the nested inner scope owns the action once and the
outer scope's local owned list stays empty. K15 keeps an unscoped source under
`RootRecovery`, not an empty local scope.

## Cross-dimension coverage

`action_effects` exposes only the K action projection. A distinct
`epsilon_coverage_requirement` includes a same-atom Forward opportunity once
and is compared against the already-derived root epsilon envelope. It is not
added to epsilon. Root K-required Recovery Usage is independently composed
from the factor-free leaf profile and compared against root `U.recovery`; it
is not added to U. The proof retains exact Recovery phase/meter/unit
coordinates. Missing active-source coverage remains Unproven, and checked
arithmetic overflow is a Violation.

K20's dynamic FOREACH yields an unbounded root action. The internal root E
comparison runs, while the dynamic Usage algebra remains Unproven. Its
export-epsilon assertion is explicitly deferred; no RootExportAssertion or
exported F/I/O/R/epsilon/K/U/X checker is part of this phase.

## No runtime authority

Positive action potential, guard compatibility, and root E/U coverage do not
prove an applied effect, receipt state, successful reconciliation, action
eligibility, commit order, runtime recovery safety, or execution
authorization. No plan, journal, reducer, scheduler, worker, grant,
reservation, receipt, or command was added.
