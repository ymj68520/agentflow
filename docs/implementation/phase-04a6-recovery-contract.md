# Phase 4A.6 — Recovery Contract / Replay & Idempotency / Static Recovery Proof

Status: complete (closed 2026-10-08) for the static K dimension only.

Artifacts:

| Artifact | Path |
|---|---|
| AFIDL module | `spec/idl/recovery-contract.afidl` |
| Rust crate | `crates/af-recovery` |
| Normative semantics | `spec/recovery/recovery-contract.md` |
| Independent Oracle | `tools/gen-recovery-contract-corpus.py` |
| Decision register | `docs/implementation/phase-04a6-decisions.md` |

The implementation declares replay/idempotency/reconciliation/compensation
obligations and performs pure static proof only. No receipt, grant, key,
runtime state, journal, compensation execution or ComponentContract is
created. The AFIDL module has 8 numbered types and 26 ABI rows. The corpus
contains 34 cases: 8 golden, 10 negative, 3 replay, 4 reconciliation, 3
compensation, 5 compatibility and 1 equivalence. Registry totals are 28
specifications, 281 normative rules, 361 active ABI rows, 11 AFIDL modules
and 63 suites; `spec-check` reports zero missing coverage.
