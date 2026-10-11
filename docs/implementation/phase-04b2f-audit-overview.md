# Phase 4B.2F — Independent Root Export Audit Overview

## Outcome

**Outcome B — Static False-Proven Found.** Two independent production-entry
probes returned `CheckedRootExportCompatibility` for inputs that are not
statically compatible. RootExport MUST NOT be accepted as a release control
until the defects are fixed in a separate authorized phase and the audit
probes are retained as regressions. This audit did not patch production.

| Finding | Severity | Dimension | Observed |
| --- | --- | --- | --- |
| AUDIT-F01 — Effect containment combines different export rows | P0 | epsilon | All nine dimensions Proven and checked capability issued although a valid input valuation leaves root Effect upper 10 covered only by an export row with upper 5. |
| AUDIT-F02 — Root derivation can be checked against a different subflow resolver snapshot | P0 | epsilon / proof-context lifecycle | All nine dimensions Proven and checked capability issued; root source pin came from subflow export A while the assertion/current resolver snapshot pinned export B. |
| AUDIT-F03 — Applicable RuleId inventory is incomplete | P2 | compiler semantics identity | The 84-ID inventory omits foundational Semantic Core/IR/dataflow RuleIds. Full source/registry bytes are pinned, so no stale-content bypass was reproduced; the completeness claim is unsupported. |
| AUDIT-F04 — Guard-countermodel suppression is root-global | P2 | epsilon / guard precision | Any SELECT/RACE/HANDLE anywhere in the tree disables typed guard countermodels for every effect. This can conservatively lose a Violation witness and yield Unproven, not false Proven. |

No additional P1 model gap was confirmed. P3 body, implementation, publisher,
authority, and runtime evidence remains outside this static phase.

## Independence and method

The audit used four separate evidence lines: frozen 4B.2C/2D/2E.1/2E.2 rules;
direct review of the production Rust paths; independent hand-calculation of
Recovery bounds; and 31 temporary adversarial runs through the real fixture
pipeline and RootExport API. Of those runs, 30 reached the RootExport checker
and one was correctly rejected at ComponentContract validation. Two of the 30
RootExport runs produced confirmed P0 false-Proven results. The temporary tests
and fixture hooks were removed; neither formal Expected nor a production file
was changed.

The existing 38/38 comparator mutations only mutate compared Actual projection
data. They were not counted as production-input probes.

## Baseline and deliverables

Start HEAD: `01f957045fe4a5d52331e8fa01b2ef6c76919118`.
The starting tree was clean. Baseline remains 41 Specifications, 380 Rules,
379 covered, 1 intentional exception, 0 missing coverage, 477 active ABI
assignments, 16 AFIDL modules, and 14 HashV1 domains.

The ten reports in this audit package distinguish confirmed false proofs,
independent checks, designed-but-not-run probes, P0–P3 findings, and readiness.
All changes in this audit are documentation-only. See
[false-Proven analysis](phase-04b2f-false-proven-analysis.md),
[probe log](phase-04b2f-adversarial-probes.md), and
[readiness](phase-04b2f-readiness.md).

## Stop boundary

No AFIDL, registry, conformance Expected, ABI, digest, checker, control
algebra, WorkflowVersion, Publication, Plan, Journal, Reducer, Scheduler,
Worker, or Runtime change was made. Do not proceed to publication design on the
assumption that current RootExport results are sound.
