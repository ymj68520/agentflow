# Phase 4C.1B.3 — RootRecovery Projection

RootRecovery is not accepted as a caller-supplied summary. The same D9 `DerivedWorkflowContractSummary` instance supplies both the root tree and `root_recovery()`.

The production K model/algebra is in `crates/af-computation/src/root_recovery.rs` and its same-call assembly is in `crates/af-computation/src/control.rs`. Portable projection is in `crates/af-portable/src/root_revalidation.rs`.

The typed projection retains each action's caller digest, source node and contract digest, forward entry ordinal, full `RecoveryActionIdentity`, forward/action occurrences, owner scope, local and root bounds, applied retry/iterate factors, usage binding, guard evidence/coordinates, eligibility verdict, and control path. Unknown, partial, per-child, overflow, and invalid-multiplicity states stay explicit.

The 16 independently declared D9-Recovery bundles exercise outer retry (1–4 attempts), outer bounded iterate (1–4), retry×iterate products, a retry inside its nearest CompensationScope, and multi-leaf sequence/race/parallel cases. The production runner compares action cardinalities, exact source nodes, forward ordinals, local/root bounds, and multiplicity-factor counts.

K action effects and recovery usage remain separate from ordinary root epsilon and forward U. Only the production D9 algebra applies multiplicity. The D9 cases are not claims of receipts, action dispatch, or runtime compensation eligibility.

Resource limits are enforced before formal D10 comparison: 32 nested assertion identities, 50,000 identity-expansion units, and 2,000,000 aggregate assertion comparison units. Any limit failure returns no checked compatibility capability.
