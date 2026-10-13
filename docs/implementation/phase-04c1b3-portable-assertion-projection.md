# Phase 4C.1B.3 — Portable Assertion Projection

`root_revalidation::project_assertion` turns a registered `PortableRootExportAssertion` into the existing unproved `RootExportAssertion` input. It does not deserialize a checked type or synthesize a proof report.

The projection covers the exact IR/export pins; compiler identity; fresh proof-context manifest; input, output, resource, effect, flow, recovery, execution, and subflow coordinates. It preserves source and target namespaces, list ordering, identity distinctions, option states, and exact digest/type-schema pins. Numeric wire ordinals that become host `usize` values use checked conversions.

Portable structural-type IDs are expanded as a typed DAG into `StructuralType`. Cycles, missing IDs, depth over 96, more than 100,000 table nodes, or more than 100,000 expansion steps fail closed. Each assertion mapping collection is capped at 2,048 rows; the formal comparison budget applies across the reachable assertion graph.

Subflow assertion IDs are resolved from the Bundle table. Missing/cyclic rows, target IR/export pin mismatch, untrusted child compiler content, missing child export inventory membership, and graph/work limits reject the request. The current nested graph bound is 32 identities. No `RootExportAssertionIdentity` is merged merely because two rows look locally similar.

The production projection code is in `crates/af-portable/src/root_revalidation.rs`; the formal model and checker remain in `crates/af-computation/src/root_export.rs`.
