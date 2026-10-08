# Phase 4A.5B — Fixed Price Snapshot / Exact Cost Projection / Static Budget Ceiling

Status: complete (closed 2026-10-08) for static Cost Projection only.

Artifacts:

| Artifact | Path |
|---|---|
| AFIDL module | `spec/idl/cost-projection.afidl` |
| Rust implementation | `crates/af-usage/src/cost.rs` and related modules |
| Normative semantics | `spec/usage/cost-projection.md` |
| Independent Oracle | `tools/gen-cost-projection-corpus.py` |
| Decision register | `docs/implementation/phase-04a5b-cost-decisions.md` |

The implementation consumes Phase 4A.5A Static Usage Envelopes and supports
only explicit fixed-linear prices with conservative per-meter-unit ceiling
rounding. It does not measure billing or implement Budget/Usage/Quota Ledgers,
reservation, settlement, provider pricing, FX or runtime admission.

The frozen corpus contains 34 cases: 7 golden, 7 pricing, 10 negative,
9 budget and 1 equivalence. The cost AFIDL module contributes 9 numbered
types and 25 ABI rows. Registry totals are 26 specifications, 266 normative
rules, 335 active ABI rows, 10 AFIDL modules and 60 suites; `spec-check`
reports zero missing coverage.
