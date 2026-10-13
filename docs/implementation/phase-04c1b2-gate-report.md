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
- `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features --
  -D warnings`, and `cargo run -p af-spec --bin spec-check -- check` passed.
  Registry counts remained Specifications 43, Rules 404 (403 covered, one
  intentional exception, zero missing), ABI assignments 775, AFIDL modules
  17, HashV1 domains 14.
- All 26 `tools/gen-*.py` generators ran twice. Changed/untracked file hashes
  matched between rounds; no generated registry or frozen expected drift.
- The static-surface test was updated to permit the required D6/D7 wrappers
  while continuing to forbid D9/D10 and runtime capabilities; its focused
  post-fix test passed.

The full workspace `cargo test --all` is the post-commit gate and is recorded
after that run. No D9/D10, RootExport capability, or runtime/publication work
is part of this phase.
