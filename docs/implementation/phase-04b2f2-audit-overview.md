# Phase 4B.2F.2 — Independent RootExport Soundness Re-Audit

## Decision

**Outcome A: soundness re-audit passed for the current static RootExport subset.**
AUDIT-F01 and AUDIT-F02 are independently confirmed closed; no new
false-Proven route was found. The accepted scope is only a possible future
Release Control input. This does not approve a WorkflowVersion, establish
publisher identity, implementation/body conformance, or authorize runtime.

AUDIT-F03 remains P2 for incomplete applied-RuleId provenance. Exact current
identity of the bundled normative sources and both complete registries prevents
stale checker-content reuse within this checker. Therefore this provenance gap
does not block the current static compatibility result, provided a future
consumer requires current compiler-semantics preflight and does not interpret
the 84-ID list as a complete dependency closure. AUDIT-F04 remains P2 diagnostic
precision only: conservative guard handling can lose a Violation witness, but
does not create a Proven witness.

## Baseline and independence

- Start HEAD: fa0ab009530648a89428cf2622ec48a6fae203e6.
- Start worktree: clean before temporary test instrumentation; the temporary
  instrumentation was removed before gates and commit.
- Frozen baseline: 41 specifications, 380 normative rules, 477 active ABI
  assignments, 16 AFIDL modules, 14 HashV1 domains, and 311 historical
  production differential cases.
- Existing 47 RootExport soundness cases remain separately reported. They are
  not counted as this re-audit's probes.

The audit independently derived expected safe results from the frozen
Effect/Resolver rules, inspected the production success and fallback paths,
constructed changed checked contracts/resolvers/assertions, and invoked the
production RootExport APIs. The expected results were authored before reading
the production verdicts. The temporary probe harness was removed; no production
source, formal checker, expected corpus, registry, ABI, or digest input changed.

## Probe accounting

62 audit fixtures/probes were designed across the F01, F02, cross, F03, and
interaction matrices:

- 58 reached the RootExport production entry point.
- 3 were blocked by an earlier formal validator (two F01 cases, one F02
  Recovery-profile case).
- 1 duplicate-target fixture was rejected by the checked resolver builder
  before RootExport.
- The 58 include fresh P30/P31 reproductions; 56 are other current-run
  variants, not 56 claims of unique semantic equivalence classes.
- 57 runs retained overall verdict and capability results. The
  reverse-insertion-order identity probe retained only the observed
  proof-context=Proven assertion in its saved log; overall verdict and
  capability bit were not retained, so it is not counted as a positive
  compatibility result.

This is separate from both the 31 production probes in Phase 4B.2F and the
47 permanent F.1 soundness cases. None of those historical counts was added to
the 58.

## Finding disposition

| Finding | Status | Severity | Blocking scope |
| --- | --- | --- | --- |
| AUDIT-F01: cross-row effect witness synthesis | CLOSED | P0 fixed and independently re-audited | No static RootExport blocker found |
| AUDIT-F02: stale Resolver-derived Root/RootRecovery | CLOSED | P0 fixed and independently re-audited | No static RootExport blocker found |
| AUDIT-F03: applied RuleId provenance closure | OPEN | P2 | Does not block static use while exact full content identity is a mandatory preflight; does block claiming 84 IDs are complete |
| AUDIT-F04: guard-countermodel path sensitivity | OPEN | P2 | Diagnostic precision only; unresolved cases remain Unproven |

Full evidence and gates are in the companion F01, F02, adversarial-results,
rule-closure, findings, readiness, and gate-report documents.

## Explicit non-goals

No WorkflowVersion release approval, Publication, ExecutablePlan, Journal,
Reducer, Scheduler, Worker, runtime admission, publisher authentication, or
body/implementation evidence is implemented or inferred here.
