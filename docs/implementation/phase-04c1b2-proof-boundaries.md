# Phase 4C.1B.2 — Proof Boundaries

D5-D8 establish that the supplied static checker inputs can be reconstructed
exactly and freshly validated. They do not authenticate who supplied a body,
prove implementation conformance, verify origin/trust claims, create a grant,
or establish publication eligibility.

No D9 Root/RootRecovery aggregation or D10 RootExport assertion check is
called. No `CheckedRootExportCompatibility` is produced. B01/B02 remain
blocking until D9-D10 provide cross-process proof-input reconstruction and
fresh RootExport validation. Target export resolution is distinct from target
body conformance. AWAIT remains unknown external influence. F03/F04 and the
ModelMissing plan-lowering/full-runtime roles retain their pre-existing
status.

This phase does not implement LockGraph, PlanCore, WorkflowVersion,
Publication, DeploymentRevision, ExecutablePlan, Journal, Reducer, Scheduler,
Worker or Runtime.
