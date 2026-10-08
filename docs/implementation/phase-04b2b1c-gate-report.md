# Phase 4B.2B.1C — Gate and Frozen-Surface Report

Baseline: `dc497a6ec70d535ca0a31803f25e5a10feec2fc0`. Corpus revision 3 is advanced to revision 4; all 58 CaseIds are unchanged. The required gates passed on the candidate tree and were repeated after the categorized test/corpus and documentation commits. The post-commit rerun passed; the final committed working tree is clean.

## Required gates

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | pass |
| `cargo clippy --all-targets --all-features -- -D warnings` | pass |
| `cargo test --all` | pass; no failed tests |
| `cargo run -p af-spec --bin spec-check -- check` | pass |

Observed `spec-check` summary:

```text
specifications: 38
normative rules: 349
rules covered: 348
rules intentionally uncovered: 1
rules missing coverage: 0
ABI assignments: active 477, retired 0, reserved 0
AFIDL modules checked: 16
cross-reference errors: 0
```

The independent full conformance command passed 58/58 stage executions and comparisons, with 55 ControlDerivation, two stage-specific early rejections, one lower-level-only algebra execution, 15/15 detected mutations, and three supplemental overflow checks.

## Generator idempotence

All **19** `tools/gen-*.py` generators were run twice. After each round the tracked diff was hashed with `git diff --binary | git hash-object --stdin`:

```text
Round 1: f302a9ce5f1ed294c724196b20985fa481afc93f
Round 2: f302a9ce5f1ed294c724196b20985fa481afc93f
```

The identical hashes confirm the second round introduced no additional tracked drift.

After commit, the same 19 generators were run twice again. Both post-commit rounds left the tracked tree unchanged (empty-diff hash `e69de29bb2d1d6434b8b29ae775ad8c2e48c5391`).

## Frozen-surface / baseline comparison

- `spec/idl/`: zero changes.
- `spec/registry/`: zero changes; 477 active assignments remain frozen.
- Non-target `spec/conformance/` corpora: zero changes.
- HashV1 domains: unchanged (14); no digest golden changed.
- Target `spec/conformance/computation-control/`: 26 files changed under revision 4 for explicit fixture inputs, corrected independent Expected values, and the revision marker; manifest CaseId set is unchanged at 58.
- Production `crates/*/src`: zero changes. No public API, AFIDL, ABI, digest, runtime, scheduler, or follow-on control operator was added.

## Repository status and scope

The only implementation files in scope are the existing `af-computation` test and support modules. Conformance reports are under `docs/implementation/phase-04b2b1c-*.md`. No Phase 4B.2B.2 work was started.
