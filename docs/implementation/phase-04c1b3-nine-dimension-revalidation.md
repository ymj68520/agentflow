# Phase 4C.1B.3 — Nine-Dimension Revalidation

D10 calls the formal `check_root_export_assertion` with fresh D7 caller IR, D6 export contract, D8 snapshot, D5 context, D9 summary, and the unproved typed assertion. That function calls `analyze_root_export_assertion`, reports each dimension and preflight, and creates `CheckedRootExportCompatibility` only when static compatibility is Proven.

The only capability constructor path is `check_root_export_assertion` in `crates/af-computation/src/root_export.rs`; portable report construction and call wiring are in `crates/af-portable/src/pipeline.rs`.

The report preserves I, O, Phi, epsilon, R, F, U, K, and X with verdict, rule, reason, prerequisites, source facts, typed mapping witnesses, and execution-child evidence. Portable code contains no parallel permissive comparator.

The independent full-static positive proves all nine static dimensions and receives a process-local capability. Removing X source coverage makes X and overall static compatibility Unproven with no capability. The body and implementation states remain RequiresEvidence; publisher authentication and runtime authority remain NotEstablished.

P30 split-row and same-row tests run through the actual Portable pipeline. The split fixture has one root E row with selector `{a=1,b=1}`, lower 0/upper 10, unconditional guard, against two separate export rows: `{a=1}` guarded by `p1 == Int64(0)` at upper 10, and `{b=1}` unconditional at upper 5. With the legal root input domain containing `p1=1`, epsilon is Unproven and no capability is issued. The same-row export positive remains Proven.
