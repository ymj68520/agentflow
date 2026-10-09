# Phase 4B.2B.3 — Gate report

Status: implementation and pre-commit all-target gates pass. Categorized
commits and the required post-commit re-run are pending.

Baseline HEAD: `96ebbc149f68598abe872698495c133ac657a694`.

Primary checks observed so far:

- `cargo fmt --all -- --check` — pass.
- `cargo clippy --all-targets --all-features -- -D warnings` — pass.
- `cargo test --all` — pass. The 58-case suite is 58/58; advanced control is
  72/72 with the same CaseIds; Scope/CompensationScope/root closure is 55/55,
  including one underdeclared action-Effect row rejected as Violation.
- New Scope/closure comparator mutations — 4/4 detected; 589 expected fields
  compared across the 55 cases with zero mismatch.
- `cargo run -p af-spec --bin spec-check -- check` — pass: 40 specifications,
  360 rules, 359 covered, one existing intentional exception, 0 missing
  coverage, 477 active ABI rows, 16 AFIDL modules, 0 cross-reference errors.
- All 21 `tools/gen-*.py` generators were run twice. The combined SHA-256 of
  `spec/conformance/` and `spec/registry/` was identical on both rounds:
  `5398CAAF031420ED38D6999845B8B927F448092DFFC96282802E314ED41E1B23`.
- Freeze comparison against the starting HEAD: all 16 AFIDL modules, 477 ABI
  rows, 14 HashV1 domains, existing type/IR/contract digest goldens, and the
  58-case revision-5 corpus are byte-identical. Only the old 72-case
  `scope-unsupported` Expected was revised; its CaseId is preserved and the
  suite revision is 2. The other 71 Expected projections are unchanged.

The final commit hashes and post-commit worktree/gate status will be appended
after the categorized commits and final re-run.

**Static Proven Facts:** the focused compile and differential results above.

**Conservative Upper Bounds:** verified by the dedicated action-bound vectors.

**RequiresEvidence:** receipt, order, and settlement assertions remain
non-Proven.

**Unproven:** root-export/authority/budget boundaries remain explicit.

**Runtime-Only Obligations:** none are performed by the test gates.
