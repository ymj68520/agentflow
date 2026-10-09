# Phase 4B.2B.3 — Gate report

Status: implementation, conformance, and normative commits are complete. The
post-commit gates and generator runs pass; the final report-only commit is
followed by the confirmation run recorded below.

Baseline HEAD: `96ebbc149f68598abe872698495c133ac657a694`.

Primary checks observed so far:

- Post-commit `cargo fmt --all -- --check` — pass.
- Post-commit `cargo clippy --all-targets --all-features -- -D warnings` — pass.
- Post-commit `cargo test --all` — pass. The 58-case suite is 58/58; advanced
  control is 72/72 with the same CaseIds; Scope/CompensationScope/root closure
  is 55/55, including one underdeclared action-Effect row rejected as Violation.
- New Scope/closure comparator mutations — 4/4 detected; 589 expected fields
  compared across the 55 cases with zero mismatch.
- `cargo run -p af-spec --bin spec-check -- check` — pass: 40 specifications,
  360 rules, 359 covered, one existing intentional exception, 0 missing
  coverage, 477 active ABI rows, 16 AFIDL modules, 0 cross-reference errors.
- Post-commit, all 21 `tools/gen-*.py` generators were run twice. The combined
  SHA-256 of `spec/conformance/` and `spec/registry/` was identical on both rounds:
  `5398CAAF031420ED38D6999845B8B927F448092DFFC96282802E314ED41E1B23`.
- Freeze comparison against the starting HEAD: all 16 AFIDL modules, 477 ABI
  rows, 14 HashV1 domains, existing type/IR/contract digest goldens, and the
  58-case revision-5 corpus are byte-identical. Only the old 72-case
  `scope-unsupported` Expected was revised; its CaseId is preserved and the
  suite revision is 2. The other 71 Expected projections are unchanged.

Categorized commits:

- `dd9f9b3f3b45ebdddf91afaec44b5ef598d5118e` — Scope/CompensationScope
  algebra and workflow closure implementation.
- `c714a10fdf84d5c5a1757d5d54648d0ae22e0155` — independent oracle, 55-case
  corpus, differential runner, and the documented 72-case Scope migration.
- `fd14b1e5506f9189ebfdff239b58c88f4b5df0c4` — normative rules, registry, and
  implementation reports.

P0/P1 remaining: none. P2 boundary: the root has no exported ComponentContract
pin or frozen coordinate mapping, so root-export compatibility remains
Unproven; authority/budget ceilings and runtime receipts remain absent by
design. Static derivation closes every current opcode and known source, but it
does not establish body conformance or export compatibility. A separate
Closure Audit is recommended before making that stronger claim. Runtime/Plan/
Journal/Reducer work was not started.

Final post-report-commit confirmation: fmt, clippy, all tests, spec-check, and
the two-round generator hash above all pass. The worktree is clean at delivery.

**Static Proven Facts:** the post-commit compile, test, registry, and
generator-idempotence results above.

**Conservative Upper Bounds:** verified by the dedicated action-bound vectors.

**RequiresEvidence:** receipt, order, and settlement assertions remain
non-Proven.

**Unproven:** root-export/authority/budget boundaries remain explicit.

**Runtime-Only Obligations:** none are performed by the test gates.
