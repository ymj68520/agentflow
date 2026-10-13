# Phase 4B.2F.2 — Gate Report

## Baseline and scope

- Frozen start HEAD: fa0ab009530648a89428cf2622ec48a6fae203e6.
- Scope: audit documents only in the final commit.
- Production Rust, AFIDL, ABI, HashV1 domains, digest goldens, normative
  registry, conformance expected corpus, and permanent tests are unchanged.
- Temporary independent harness and its test-support hooks were removed before
  final gates.

## Gate results

| Gate | Result |
| --- | --- |
| cargo fmt --all -- --check | PASS |
| cargo clippy --all-targets --all-features -- -D warnings | PASS |
| cargo test --all | PASS; all workspace tests completed, including the 311 historical production differential cases |
| cargo run -p af-spec --bin spec-check -- check | PASS; 41 specifications, 380 rules, 379 covered, 1 intentional exception, 0 missing, 477 active ABI assignments, 16 AFIDL modules, 0 cross-reference errors |
| All tools/gen-*.py, two consecutive rounds | PASS; 24 generators each ran twice with exit code 0 |
| ABI/digest/expected/specification registry drift vs frozen HEAD | PASS; no tracked change under crates, spec, or tools |
| Commit and post-commit clean-tree gates | PASS; the committed docs-only tree was rechecked with all listed gates, generator rounds, and clean-tree verification |

The existing formal production differential baseline is 311 cases:
Control 58, Advanced Control 72, Scope/Closure 55, Root Recovery 44, Root
Export 35, and Root Export Soundness 47. Audit-only probes remain separately
counted (58 RootExport-reaching; 3 earlier-validation blocks; 1 resolver
builder rejection).

## Frozen path comparison

The final diff against the frozen start contains only the ten
phase-04b2f2-*.md audit reports. Tracked generated and frozen production paths
were unchanged after both generator rounds. After commit, all listed gates
were rerun, the 24 generators again completed two rounds, and git status was
clean. The final report-only commit is followed by the same verification
sequence.
