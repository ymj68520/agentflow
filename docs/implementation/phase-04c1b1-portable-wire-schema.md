# Phase 4C.1B.1 — Registered Portable Wire Schema

The normative source is
[portable-proof-inputs.md](../../spec/portable/portable-proof-inputs.md);
the numbered AFIDL source is
[portable-proof-inputs.afidl](../../spec/idl/portable-proof-inputs.afidl).
The schema contains 108 named types, including 17 path-specific auxiliary
types, and 298 newly registered members. Existing core, IR, contract and
dimension shapes retain their full imported schema identities.

`PortableProofInputBundle` owns ten required fields: `schema_version`,
`root_assertion_id`, `schema_assets`, `ir_assets`, `contract_assets`,
`structural_types`, `proof_contexts`, `resolver_dependency_manifests`,
`compiler_semantics_manifests` and `root_export_assertions`. All assets are
complete inline canonical bytes. Each content reference retains its
domain-bearing digest and exact nominal SchemaRef.

The registered types preserve all RootExport mappings, independent context
universes, scoped resolver associations and ten compiler-role declarations.
Options use explicit tags. IDs are local to their declared tables. No field
asserts a checked contract or proof verdict.

`PpiAux` is reserved for the deterministic anonymous-shape expansions. Lists
that represent sets carry the separately defined key semantics from the
materialization audit. The proof Context does not acquire the bootstrap schema
automatically.

The independent AFIDL corpus gained one portable schema case and its
108 canonical TypeSchema byte/digest records. Earlier cases retain their
per-type values.
