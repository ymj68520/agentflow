# Phase 4B.2F — Readiness and Stop Decision

## Explicit audit outcome

**Outcome B — Static False-Proven Found.** RootExport is **not accepted as a
Release Control**. The production checker issued a checked static compatibility
capability in two legal adversarial cases where its proof premises did not
match the actual export semantics/current resolver association.

This decision does not deny that many component-local algebra, recovery,
mapping and static/runtime boundaries are conservative. It means the current
`CheckedRootExportCompatibility` type is not safe to consume as a release
admission signal until both P0s are fixed and re-audited.

## Required next step

Create a separate, narrowly scoped **RootExport Soundness Fix** phase before
any WorkflowVersion/Publication architecture work. It should address:

1. same-row Effect containment for operation/qualifier/scope/bound/guard and a
   typed reachable countermodel;
2. exact binding between the derived root summary and the resolver snapshot
   used by the checker, especially subflow target export pins;
3. permanent production-input regressions P30/P31, retaining the independent
   expected derivation rather than deriving Expected from Rust output;
4. re-run at least the 40-case adversarial design matrix and every existing
   production suite after the fix;
5. revisit the 84-RuleId applicability closure and the root-global guard
   countermodel suppression.

No production fix was made in this audit, consistent with the Audit-only
boundary. This report does not authorize implementing the fix automatically.

## Gate and freeze status

The starting HEAD was `01f957045fe4a5d52331e8fa01b2ef6c76919118`; the starting
worktree was clean. The 31 audit probes used temporary test/helper code, which
was removed before the final documentation commit. Formal specs, AFIDL,
registry, Conformance Expected, ABI, digest domains and existing goldens were
not modified. The final gate report records post-commit fmt, Clippy, full test,
spec-check, generator-idempotence and frozen-surface results.

The 78-case register includes both executed and design-only cases. Unexecuted
cases are marked Not run and are not counted as passed. Existing regression
totals are kept distinct from the independent probe count.

## Stop boundary

Do not enter WorkflowVersion, Publication, ExecutablePlan, Journal, Reducer,
Scheduler, Worker or Runtime. No release-control handoff is safe while either
P0 remains open. After the fix phase and independent re-audit, readiness must
be decided again from a clean, exact final HEAD.
