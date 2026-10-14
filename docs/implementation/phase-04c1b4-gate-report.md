# Phase 4C.1B.4 Gate Report (interim Outcome D)

## Baseline and scope

- Starting HEAD: `3eb53e8110c8901ed9f789dda9631eed7a7bd0dc`.
- Starting worktree: clean; branch `master`, three commits ahead of `origin/master`.
- Production D0–D10 source and frozen registries were not changed.
- This report records the comparator/audit increment only; it is not a closure report.

## Checks executed

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy -j 2 --all-targets --all-features -- -D warnings` | Pass |
| `cargo test --all -j 2` | Pass; workspace unit/integration/doc tests all pass |
| `cargo run -p af-spec --bin spec-check -- check` | Pass: 43 specs, 404 rules, 403 covered, 1 intentional exception, 0 missing; 775 active ABI assignments, 17 AFIDL modules, 0 cross-reference errors |
| Recursive comparator shape/order/Option test | Pass |
| Positive D0–D10 report test with 46 comparator mutations | Pass on current source; also included in workspace suite |
| P30 split-row negative and same-row positive with 10 comparator mutations | Pass on current source; also included in workspace suite |
| D9 Recovery 16-case production test with 12 comparator mutations | Pass on current source; also included in workspace suite |
| Six producer/consumer process scenarios | Pass in workspace suite; compares current selected stage/verdict/boundary assertions, not full independent field projections |
| Python corpus generators | All 27 `tools/gen-*.py` scripts exited successfully in both rounds; no generated-file drift is present in the current worktree |

The 512 historical differential cases covered by the registered workspace suites passed: Control 58, Advanced Control 72, Scope/Closure 55, Root Recovery 44, Root Export 35, Root Export Soundness 47, Portable Proof Codec 60, Portable Static Reconstruction 63, and Portable RootExport 78.

The comparator self-tests compare a report projection with a changed copy only to test mismatch detection; this is not an independent semantic expectation.

## Not executed / not established

- Full 78-case independent Expected generation and field comparison were not implemented; therefore 0 skipped / 0 mismatch / 78 fully compared cannot be claimed.
- Complete production-input mutation matrix and six-scenario cross-process full-field comparison were not run as a full-field Oracle.
- Required semantic field totals, independent derivation totals and uncovered field totals are not certified.
- No comparison against frozen Digest goldens was needed by the source changes, but a complete Phase 4C.1B.4 digest audit is not claimed.

## Decision

**Outcome D — Conformance Still Incomplete.** B01/B02 conformance closure remains pending. Do not treat this tree as independently audited or release-control-ready. No Phase 4C.1B.5 or later phase is authorized by this work.
