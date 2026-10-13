# Phase 4C.1B.3 — P30 / P31 Portable Soundness

## P30 / F01

The Portable P30 input contains an exact source effect row and two separately declared export rows. The formal checker is called over the reconstructed contract/context/snapshot and the full typed assertion. The split-row case reports epsilon Unproven and issues no capability; the same-row case is a Proven positive control. This exercises the fixed `af_effect::contains` path through D10 rather than a test-only comparison.

Fixtures/oracle: `tools/gen-portable-root-export-revalidation-corpus.py`. Production assertions: `crates/af-portable/tests/root_export_revalidation.rs`. The unchanged formal relation is in `crates/af-computation/src/root_export.rs`.

## P31 / F02

The Portable target graph holds the exact target `SemanticIrDigest` while varying its associated export contract digest. When the Bundle assertion declares A but the current D8 target association is B, D10 reports the subflow dependency pin as Unproven and issues no capability. A separate fresh-B case derives Root(B) from Snapshot(B); its root effect upper 10 is compared to top export upper 1 and epsilon is Violation.

Root and resolver facts are never accepted as separately supplied values. Both regressions enter through the registered D0–D10 Bundle path and therefore cannot compose Root(A) with Snapshot(B) in-process.
