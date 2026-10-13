# Phase 4C.1B.3 — Proof Context and Compiler Identity

The wire `ProofContextManifest` contains only local context/resolver IDs. D10 does not copy a portable manifest into its own witness. It captures `ProofContextManifest::capture(d5_context, d8_snapshot)` from the newly constructed objects and lets the production RootExport preflight compare that exact typed content.

D5 object construction is in `crates/af-portable/src/reconstruction.rs`; current compiler tuple support is in `crates/af-portable/src/support.rs`; fresh capture and current identity adaptation are in `crates/af-portable/src/root_revalidation.rs` and `crates/af-computation/src/root_export.rs`.

The captured tuple includes modules, schemas, effect catalogs, resource types, flow catalogs, usage meters, recovery operations, execution universes, relation snapshot bytes/pins, and resolver dependency identities. A context content change therefore requires a fresh D5–D8 reconstruction; equality of a digest alone does not replace in-process pointer identity.

D4 accepts only the complete current Profile A tuple. The projected checker identity comes from `current_compiler_semantics_identity()`, not a caller boolean or version-label substitution. Nested assertion compiler manifests are compared to the exact D4-supported root content before their identities are expanded. The projection does not claim complete transitive Rule attribution; F03 remains OPEN/P2.

The `provenance-fresh-context-unit-change` case changes the declared meter unit, reconstructs that changed context, and checks the resulting fresh proof rather than reusing the prior capability. Separate context/resolver/compiler mutations assert their actual first rejection stage.
