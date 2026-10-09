# Phase 4B.2E.2 — Root Recovery Integration

The checker consumes the existing `DerivedWorkflowContractSummary` and its
`RootRecoverySummary`; it does not implement a second control fold. It reuses
the D2 one-pass root totals, exact action identities, matching
RecoveryUsageBinding profiles, local/root bounds, guards, control paths,
ownership, source closure and K-required Recovery Usage.

For epsilon, the checker proves all three separate obligations:

1. root epsilon fits exported epsilon;
2. K action coverage fits the already-derived root epsilon;
3. K action coverage independently fits exported epsilon.

For Usage, Forward and Recovery are compared separately, then K-required
Recovery Usage is checked against root U.recovery and export U.recovery. The
K projection is not added to E or U a second time. Missing meter/phase is not
finite zero.

K is Proven only for complete source/action/profile closure and the frozen
Recovery compatibility relation. No enum ordering is invented. For a root
with recovery actions, absent action-specific Flow or X relations remain
Unproven even if the child’s ordinary per-child X requirement is known.
Receipt state, reconciliation outcome, provider honoring, action eligibility,
commit order and actual compensation execution are independent evidence.
