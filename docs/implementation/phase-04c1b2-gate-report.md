# Phase 4C.1B.2 — Gate Report

This report records measured Phase 4C.1B.2 gates. The implementation started
from `51ae18d5731c4c354fc3a3d6cfe1549dfcf1ce4d`.

## Scope invariants

- D5 exact context; D6 fresh formal contracts; D7 fresh IR and Phase 3C; D8
  exact resolver snapshot and source/target comparisons only.
- No AFIDL, ABI assignment, HashV1 domain, or existing digest meaning changed.
- No RootRecovery, RootExport, Publication or runtime capability is emitted.
- The original 60-case D0-D4 production corpus remains frozen.

## Results

- D0-D4 frozen Portable Proof Codec corpus: 60/60; no schema, ABI, AFIDL or
  HashV1 registration changes.
- New independent D5-D8 corpus: 63 unique CaseIds (D5 16, D6 15, D7 16, D8
  16). The successful full runner measured first rejection/completion as:
  `D1=2, D5=16, D6=13, D7=12, D8=20`. All 63 inputs were executed from the
  registered ten-field Bundle through `validate_supported`; full typed D5 or
  D5-D8 projections were compared where applicable. `skipped=0`,
  `mismatch=0`; all 16 comparator mutations passed.
- Historical formal production suites: Control 58/58, Advanced Control
  72/72, Scope/Closure 55/55, Root Recovery 44/44, Root Export 35/35,
  Root Export Soundness 47/47, and Portable Codec 60/60 (371 total).
- `cargo fmt --all -- --check`,
  `cargo clippy --all-targets --all-features -- -D warnings`, and
  `cargo run -p af-spec --bin spec-check -- check` passed.
  Registry counts remained Specifications 43, Rules 404 (403 covered, one
  intentional exception, zero missing), ABI assignments 775, AFIDL modules
  17, HashV1 domains 14.
- All 26 `tools/gen-*.py` generators ran twice. Changed/untracked file hashes
  matched between rounds; no generated registry or frozen expected drift.
- The static-surface test was updated to permit the required D6/D7 wrappers
  while continuing to forbid D9/D10 and runtime capabilities; its focused
  post-fix test passed.

Post-commit at implementation HEAD `1a377efac6432383367e84942c7cc1ebcc082c1d`,
`cargo test --all` passed, including both new Portable tests and all workspace
unit, integration, static-surface and doc tests. The new independent runner
completed in 685.97 seconds; its final result was 2 passed, 0 failed
(63-case production corpus plus the standalone comparator-shape test).
Post-commit `cargo fmt --all -- --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, and
`cargo run -p af-spec --bin spec-check -- check` also passed.

The D0-D4 baseline remains 43 specifications, 404 rules (403 covered, one
intentional exception, zero missing), 775 active ABI assignments, 17 AFIDL
modules and 14 HashV1 domains. The fixture still has 314 SchemaAssets, 16
static modules, 108 registered Portable schema types (17 auxiliary); no
registered schema, ABI assignment, digest golden, semantic digest or old
371-case expected value changed. No new P0/P1/P2/P3 finding was introduced.
Existing AUDIT-F03 and AUDIT-F04 remain OPEN/P2. MODEL-B01 through B07 remain
BLOCKING. This is Outcome A for D5-D8 and qualifies the work to enter
Phase 4C.1B.3, but it does not close B01/B02 because D9-D10 are still required.
There is no RootExport capability, body-conformance claim, LockGraph,
publication or runtime result.

Implementation commit: `1a377efac6432383367e84942c7cc1ebcc082c1d`. The only
follow-up commit records this post-commit gate report; no production code
changed after the post-commit test run.
