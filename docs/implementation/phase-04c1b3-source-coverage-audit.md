# Phase 4C.1B.3 — Source Coverage Audit

## Scope and data flow

This phase adds D9–D10 to the registered D0–D8 pipeline. The checker input is always reconstructed from the same portable Bundle; no Bundle-supplied verdict or checked wrapper is trusted.

| Portable declaration | Fresh consumer | Formal use |
|---|---|---|
| Root assertion ID and IR/export content references | D2–D3, then fresh D6/D7 checks | Exact caller `SemanticIrDigest` and exported `ComponentContractDigest` pins |
| Context and resolver IDs | D5 and D8 | Same newly reconstructed `StaticContractProofContext` and `StaticComputationResolutionSnapshot` |
| Resolver components and target export associations | D6/D8 | Fresh checked contracts and full source/target/export associations |
| Semantic IR control tree | D7/D9 | Formal 13-opcode `derive_workflow_contract_summary` traversal |
| Assertion compiler manifest ID | D4, then exact child-manifest equality | Current compiler semantics identity supplied by the production checker |
| Root structural type IDs | D10 | Bounded recursive conversion to `StructuralType`, with exact schema pins |
| I/O/R/E/F/K/X mappings and subflow rows | D10 | Reconstructed typed assertion passed unchanged in semantic order to the formal checker |
| Full context content in D5/D8 | D10 | `ProofContextManifest::capture` and checker preflight equality |

## Production entry points

- Public entry: `SupportedPortableStaticInputs::revalidate_root_export()` in `crates/af-portable/src/support.rs`.
- D5–D8 and scoped D9–D10 orchestration: `crates/af-portable/src/pipeline.rs`.
- Typed portable assertion projection and D9 diagnostic projection: `crates/af-portable/src/root_revalidation.rs`.
- Root derivation: `af_computation::derive_workflow_contract_summary` in `crates/af-computation/src/control.rs`.
- Nine-dimension proof and capability creation: `af_computation::check_root_export_assertion` in `crates/af-computation/src/root_export.rs`.

## Authority boundary

`CheckedRootExportCompatibility` is populated only from the successful return of the formal checker. D9/D10 diagnostic JSON is ordinary data and has no conversion back to that capability. The schema, ABI, and HashV1 registries are unchanged.
