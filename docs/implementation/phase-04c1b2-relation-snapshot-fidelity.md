# Phase 4C.1B.2 — Relation Snapshot Fidelity

Each relation Option is decoded independently. None stays None; Some(empty)
stays Some(empty). For a Some relation, D5 checks the exact root TypeSchemaRef
against the compiled module set, verifies the plain SHA-256 checksum, decodes
through that exact schema, deserializes the typed catalog, and invokes the
existing `RelationCatalogSnapshot::new` with the declared revision.

Acceptance requires byte-for-byte equality between the declared canonical
content and the catalog's fresh canonical ACE1 encoding, plus equality of
schema, revision, checksum algorithm and checksum hex. The three typed catalog
models gained Deserialize implementations only; no relation schema or wire
field changed. A revision token alone never establishes content equality.

The checksum is an ordinary content checksum, not source authentication and
not a new HashV1 domain. Relation construction failures return D5 and never
silently replace a revision or relation body.
