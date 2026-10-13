# Phase 4C.1B.1 — D2 References and D3 Content Identity

D2 resolves the root AssertionEntry, ContextId, ResolverId and Compiler
Manifest ID in their separate tables. The entry's root IR/export references
must match the assertion pins, resolver caller and component inventory.
Schema/IR/contract assets are unique by their complete typed keys, including
digest domain and nominal root type. Zero IDs, duplicate local IDs, dangling
references, conflicting scoped associations, unreachable padding and cyclic
structural/assertion identity graphs fail closed.

Resolver rows retain caller/node, target/export and direct/transitive Option
states. D2 checks the declared references; actual leaf kind/edge derivation,
closure comparison and resolver-cycle analysis belong to D8.

Each ResolverTarget and SourceAssociation is unique by its typed scoped key.
Caller IR, invoke contract, await SchemaRef and call-subflow target/export
association are resolved without collapsing their local namespaces. Assertion
and StructuralType traversal share the configured D2 graph-work budget;
exhaustion returns ResourceLimit and no reference-validated value.

D3 decodes each SchemaAsset using the existing CanonicalTypeSchema grammar,
checks member numbers and names, requires exact canonical re-encoding, and
recomputes HashV1(type-schema). IR and ComponentContract bytes are decoded
against their exact registered roots and re-encoded byte-for-byte before their
existing semantic-ir/component-contract hashes are compared. These raw
content checks never create Checked contracts or IR.

Normative source, registry and table checksums use plain SHA-256 over the
complete inline bytes. Typed registry parsing rejects duplicate object members.
Relation snapshots retain their existing root schema and plain checksum.
Schema/content hashes, file checksums and origin evidence remain distinct.

The production matrix is frozen in
[portable-proof-codec/manifest.json](../../spec/conformance/portable-proof-codec/manifest.json).
