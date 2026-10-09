# Phase 4B.2D — Readiness and Handoff

Status: **design-ready for a separately authorized Root Export Checker implementation phase**. This document and all candidate material are **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED ABI / NOT IMPLEMENTED**. Phase 4B.2D itself stops here.

## Baseline and outcome

Starting HEAD: 6815928d8571d2b499bfff42e5c16dc4f445dc61. The starting worktree was clean.

Frozen counters verified by spec-check:

~~~text
40 Specifications
360 Normative Rules (359 covered, 1 intentional exception, 0 missing)
477 Active ABI Assignments
16 AFIDL Modules
14 HashV1 Domains
~~~

**P0: none confirmed.** The 4B.2C probes showed a conservative, wider K-action sidecar and a local/root coordinate gap, not a demonstrated root E/U undercount. The current behavior still lacks the new cross-dimension guard proof; accepted decisions below specify the fix but do not implement it.

## P1 Decision Register status

| Decision | Status | Accepted resolution |
| --- | --- | --- |
| AUDIT-10 / D1 — epsilon action guard versus K action possibility | **ACCEPTED** | Declared epsilon guard is the ceiling for every occurrence of the full atom. Prove K possibility within it; use a typed countermodel for Violation and preserve Unproven otherwise. Receipt eligibility remains RequiresEvidence. |
| AUDIT-02 / D2 — local versus root-total K action | **ACCEPTED** | Distinct LocalActionBound and RootActionBound; unique control-family identities; one root traversal and operator-specific branch algebra; nearest CompScope owns local projection; Root K usage is compared with U, never added. |
| AUDIT-01 / D3 — root summary versus export | **ACCEPTED** | Option A typed build-time RootExportAssertion with exact IR/export digest pins, complete I/O/R/E/F/K/X maps, compiler semantics and proof-context manifest. Future WorkflowVersion pins it inline in its existing artifact identity; no RootContractDigest/new domain. |

All three decisions are complete at the design level. No checker, production field, conformance Expected, or publisher schema has been created.

## Required gate checklist

- [x] AUDIT-10 guard relation accepted.
- [x] K-action possibility versus epsilon-ceiling direction fixed.
- [x] Proven implication, witnessed incompatibility, and insufficient evidence have distinct verdicts.
- [x] Component-local guard identity and root expression/value coordinate mapping specified.
- [x] AUDIT-02 local/root action coordinates accepted.
- [x] Control multiplicity identity prevents repeated factors.
- [x] Root fold applies all ancestor repetition and branch algebra exactly once.
- [x] Nested CompensationScope ownership fixed as nearest-scope local owner; outer projection is transitive, not a second owner.
- [x] Same full atom Forward/action counts as two occurrence opportunities against one E ceiling.
- [x] Root K Recovery Usage compares with Root U.recovery and is not added a second time.
- [x] AUDIT-01 RootExportAssertion model accepted.
- [x] Exact SemanticIrDigest and ComponentContractDigest pins defined.
- [x] Complete typed I/O/resource and E/F/K/X source-coordinate maps specified.
- [x] Per-dimension I/O/Phi/epsilon/R/F/U/K/X directions and failure verdicts specified.
- [x] Root conditional K action is checked against root and export epsilon ceilings.
- [x] Root K recovery Usage is checked against root and export U ceilings.
- [x] Proof-context manifest, compiler semantics identity, and invalidation rule specified.
- [x] Static envelope compatibility remains distinct from body conformance and runtime evidence.
- [x] Scope authority/budget remain Unproven without explicit ceilings; no unlimited default.
- [x] Required future typed records and their owners/namespaces/identity/order/version boundaries are described.
- [x] 62 independent design vectors are specified; they are not registered corpus Expected values.
- [x] No unresolved P1 semantic ambiguity remains. Where frozen evidence cannot decide, the accepted rule returns Unproven or RequiresEvidence.
- [x] No AFIDL, ABI, digest, registry, or existing Expected drift.
- [x] Formatting, Clippy, all tests, spec-check, and all 21 generators in two rounds passed.
- [x] Documentation is committed as docs-only; final handoff verifies clean worktree.

## P2 follow-up coverage plan

These are implementation/conformance gaps, not unresolved P1 semantics. Add cases only in a later authorized conformance stage; do not alter frozen Expected files here.

| Audit | Required future coverage |
| --- | --- |
| AUDIT-03 — K under nested Retry/Iterate | K02-K05, both nesting orders, positive action, factor IDs, outer and inner factors, no double application. |
| AUDIT-04 — same full Forward/action EffectAtom | K06, including full qualifier/resource identity and F+K sum against one row. |
| AUDIT-05 — full root summary comparison | Compare every root field, per-dimension proof/prerequisite, source provenance, residual, action/profile multiset, and root coordinate map; not only a projection. |
| AUDIT-06 — action-specific Flow/X boundary | Prove only when exact source EffectFlowPartition and child X relation suffice. Otherwise assert Unproven. Any new K-specific Flow/X wire relation requires a separately versioned design. |
| AUDIT-07 — WorkflowOutput to export output | X06-X08: exact name/expression/producer/type, missing output, incompatible type, dominance/scope, and branch alternatives. |
| AUDIT-09 — Recovery source/action/profile multiset | K18, K24, X22: compare each full RecoveryActionBinding and exact RecoveryUsageBinding/profile; source-node accounting boolean alone is insufficient. |
| Nested scope ownership | K13-K15: inner owner, transitive outer view, no-scope root owner, duplicate-source and shared-atom controls. |
| Guard mismatch/coordinates | G03-G10 and G13-G16: same/different ports, typed countermodel, path implication, unknown path, receipt boundary. |
| Root E/Phi/F/U/K/X and resources | X09-X18 plus exact open/unknown, overflow, unit, alias, and per-child execution cases. |
| Context/body proof | X19-X22: changed catalog content, changed compiler semantics, subflow dependency invalidation, and body evidence separate from export compatibility. |

The independent vector set is a design oracle plan only; production comparison and frozen corpus registration remain future work.

## P0–P3 inventory

| Severity | Finding/status |
| --- | --- |
| P0 | None confirmed in the Phase 4B.2C evidence. Reopen only if a concrete undercount/false-Proven contradiction is reproduced. |
| P1 | AUDIT-10, AUDIT-02, AUDIT-01: all ACCEPTED as design in this phase; implementation remains outstanding. |
| P2 | AUDIT-03/04/05/06/07/09 and the cross-product gaps in the table above. Unknown action-specific Flow/X stays Unproven. |
| P3 | Runtime EffectReceipt/reconciliation/compensation/order; actual provider honoring; Scope authority; budget reservation/settlement; ResourceHandle/grant; Worker placement/admission; component/subflow body conformance and publisher authentication. These remain outside static root compatibility. |

## Gate results

| Gate | Result |
| --- | --- |
| cargo fmt --all -- --check | PASS |
| cargo clippy --all-targets --all-features -- -D warnings | PASS |
| cargo test --all | PASS |
| 58-case Control Production Differential | PASS |
| 72-case Advanced Control Production Differential | PASS |
| 55-case Scope/Closure Production Differential | PASS |
| cargo run -p af-spec --bin spec-check -- check | PASS: 40 specs, 360 rules, 359 covered, 1 intentional exception, 0 missing; 477 active ABI rows; 16 AFIDL modules; 0 cross-reference errors |
| Generator idempotency | PASS: all 21 gen-*.py scripts ran in two consecutive rounds |
| Frozen conformance/registry tree | PASS: tracked file-tree manifest matched before, after round 1, and after round 2; manifest SHA-256 1C1AC71AD956BDFF560619407FD8ACB5FB3741EEE3A362CE1D4D230A97971970; git diff showed no corpus or registry drift |

Passing tests validate existing implementations only. They do not establish a RootExportAssertion result, fix the current guard-check omission, or prove body/runtime conformance.

## Implementation readiness and stop boundary

There is no need for a separate guard/recovery semantics micro-stage: D1-D3 settle those semantics sufficiently for a later implementation task. A future checker phase should implement the guard relation, root K action/usage aggregation, complete typed coordinate validation, and all nine dimension reports as one proof pipeline. It must preserve Unproven and RequiresEvidence rather than promoting incomplete mappings.

This is a recommendation only. No Phase 4B.2E, Runtime, Plan, Journal, Reducer, checker, or source edit is started here. Final HEAD and the docs-only commit are reported in the handoff; the post-commit worktree must be clean.
