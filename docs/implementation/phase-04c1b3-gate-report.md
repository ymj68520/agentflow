# Phase 4C.1B.3 — Gate Report

## Implementation

- D9 fresh Root/RootRecovery: implemented through `derive_workflow_contract_summary` on the same D5/D8 scope.
- D10 full Portable assertion projection: implemented; formal nine-dimension checker owns capability issuance.
- Independent conformance: 78 new CaseIds, plus the frozen historical 434-case set.
- P30/F01: split-row negative and same-row positive run through Portable D0–D10.
- P31/F02: stale target export association and fresh-B upper-bound cases run through Portable D0–D10.
- Cross-process: six isolated Producer/Consumer byte-transfer cases.

## Final gates

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo test --all`: passed, including the historical 434-case production-differential corpus and the portable D9/D10 integration runner.
- `cargo run -p af-spec --bin spec-check -- check`: passed — 43 specifications, 404 normative rules, 403 covered, 1 intentional exception, 0 missing; 775 active ABI assignments; 17 AFIDL modules; 0 cross-reference errors.
- Generator inventory: 27 scripts. Two full rounds produced identical SHA-256 snapshots of all changed and untracked workspace files.
- Frozen existing `spec/` surfaces: no tracked changes under `spec/idl`, `spec/registry`, or prior conformance corpora. The only new conformance tree is `spec/conformance/portable-root-export-revalidation/`.
- New CaseIds: 78. Observed first-stage distribution: D2=4, D3=1, D4=2, D8=2, D10=69. D9 control/recovery inputs all reached formal D10 analysis; early provenance negatives stopped at their first declared rejecting stage.
- Comparator mutations: 20/20 detected. Production input mutations, P30/P31 regressions, context/resolver/compiler provenance cases, and six process-isolated byte-transfer cases passed.

## Closure decision

**Outcome D — Conformance Incomplete.** Production D0–D10 and all listed tests pass, but the independent Python oracle does not independently compute and compare every static fact and proof witness field for every CaseId. The 20 comparator mutations cover the selected independent semantic view, not the complete typed D5–D10 report. Therefore B01/B02 are “implemented code, closure pending,” not closed for this phase. A full-field oracle/comparator expansion and an independent Portable Proof Integration Audit are still required before Release Control integration.

Production locations: `crates/af-portable/src/pipeline.rs`, `crates/af-portable/src/root_revalidation.rs`, `crates/af-portable/src/reconstruction.rs`, `crates/af-computation/src/control.rs`, and `crates/af-computation/src/root_export.rs`.
