# Accepted design decisions

**Design accepted; draft normative only. No registry or production change.** This decision register selects one model and resolves G01's representation choices. [Normative draft](phase-04c1b4c1-g01-normative-contract-draft.md) is the detailed proposed contract; [readiness](phase-04c1b4c1-readiness.md) controls entry to later work.

| Decision | Selected result |
| --- | --- |
| Model | Option A, canonical typed identity + lossless canonical semantic string; not A/C diagnostic-string hybrid |
| Collection | Set of complete identities; canonical sequence view; no sentence-based dedup |
| Repetition | Separate declared source/origin occurrences distinct; inherited observation copies share a fact with all observation edges |
| Identity | Complete exact scope, assertion path, stage, kind, dimension, source, target, coordinate, rule, requirement, origin |
| Status | Required value; no new ProofVerdict, no automatic evidence satisfaction; conflicting simultaneous values reject |
| Unknown/missing | Typed unresolved request or explicit model failure; never unknown-to-None/empty/Proven |
| Failure | Violation retains contradiction and remediation obligation; other obligations remain |
| Preflight | Five mandatory-static predicates; post-gate body item is independent boundary despite old list location |
| Blocked dimensions | Nine blocker records referencing exact failed core facts, not nine invented evaluated obligations |
| Boundaries | Six root facts plus exact applicable call-site body requirements; kept even if static Proven/failed |
| Ordering | Typed tuple/ranks, numeric2<10, source/target/occurrence/nested path retained; no global string sort |
| Encoding | Closed report-local shape using existing ACE1, full snapshots/canonical bounded value DAG, no new digest domain |
| Prerequisites | Typed predicate keys/applicability/discharge and consumer links; established prerequisites may accompany Proven |
| Reference | RequiredSemantic typed evidence coordinate; RuleId remains separate; no generic diagnostic demotion |
| Reasons | Existing DiagnosticOnly classification retained; essential fact data cannot be discarded as prose |
| Capability | Same five mandatory preflight + exactly nine evaluated Proven static dimensions; empty obligations never sufficient |
| Current Rust | Representation/provenance adaptation required; do not patch existing strings/Expected to claim compatibility |
| Compiler migration | Newly identified successor semantics + exact current-only fail-closed support; historical verifier retains old corpus |
| Normative additions | Separately authorize source/RuleId/support/profile registration; local D01–D16 are not allocated official IDs |
| This task | Documentation/design vectors only; all vectors Not Executed, zero new production-case passes |

G01-P/G01-R/G01-ROLE are design-dispositioned but not registered/implemented. G01-MIG records the mandatory future activation/regression-edition decision. All are P1 design/compatibility dependencies, not confirmed production false-Proven. No P0 is declared. G01 remains OPEN/P1 with **FORMAL CONTRACT DESIGN ACCEPTED / NORMATIVE REGISTRATION PENDING** after design/gates complete.

No contradictory collection rule applies elsewhere: "ordered view" always means canonical encoding of a Set; source/control sequences remain semantic inputs and preserve order. Old inherited vectors keep their historical multiset behavior until an explicitly versioned future adapter; the proposal does not silently reinterpret old Expected.
