# Phase 4B.2F.1 — Gate Report

## Scope and source state

Starting HEAD: `964853529521ea4d22614759afb49493854e20bf` (clean worktree).
The code/test commits before this documentation commit are `951ac3f`
(`fix(computation): bind root export proof evidence`) and `721972f`
(`test(computation): add root export soundness corpus`). The exact final HEAD
is reported in the handoff.

## Findings and production corpus

```text
AUDIT-F01: FIXED — P30 safe as epsilon Unproven; no checked capability
AUDIT-F02: FIXED — P31 proof-context preflight Unproven; no capability
P30 pre-fix: all nine Proven; checked capability issued
P31 pre-fix: all nine Proven; checked capability issued; body RequiresEvidence
New independent production corpus: 47/47 executed and fully compared
  F01: 37   F02: 10   unexpected mismatches: 0   skipped: 0
```

The exact pins and production results are in
[`phase-04b2f1-adversarial-conformance.md`](phase-04b2f1-adversarial-conformance.md).

## Remaining checks

Gates pass on the final code/test tree: `cargo fmt --all -- --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all`,
and `cargo run -p af-spec --bin spec-check -- check`. The 24 generators passed
two complete rounds; generated `spec`/`tools` tree SHA-256 after both rounds was
`249dc0db4ff4b6610eff295506ef1c3d16f1aa1207e0548ef5d534be75315b2d`. The old
production differential remains Control 58/58, Advanced Control 72/72,
Scope/Closure 55/55, Root Recovery 44/44, and Root Export 35/35 (264 total).
`spec-check` reports 41 specifications, 380 rules,
379 covered, 1 intentional exception, 0 missing coverage, 477 active ABI
assignments, 16 AFIDL modules, and no cross-reference errors.

The frozen-path diff from the starting HEAD is empty for AFIDL, ABI
assignments, HashV1 domains, digest rules, schema goldens, and digest goldens.
All gates are rerun after the final documentation commit; the exact final HEAD
and clean-tree state are reported in the handoff. No Release Control approval
is implied; readiness stays **PENDING INDEPENDENT RE-AUDIT**.
