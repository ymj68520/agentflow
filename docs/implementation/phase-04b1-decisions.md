# Phase 4B.1 Decisions

1. Materialize the four-schema Phase 4B.0B design without changing prior
   schemas or HashV1 domains.
2. Keep component identity in the semantic body; keep publication version,
   implementation, price and proof data outside it.
3. Bind proof to exact relation schema, revision and canonical-content
   checksum. The checksum is plain SHA-256 of ACE1 catalog bytes, not a
   signature or a new HashV1 domain.
4. Use exact nominal TypeSchemaRef pins for public I/O. Structural alias
   expansion remains internal and does not widen substitution proof.
5. Preserve full Effect Atom identity in flow partitions and recovery action
   bindings. Operation-only or qualifier-family-only matching is forbidden.
6. Treat open/unknown rows and missing catalog or meter coverage as
   `Unproven`, never as empty, allowed, or zero.
7. Keep static contract compatibility separate from implementation
   attestation and runtime authorization.

No accepted Phase 4B.0B design amendment is introduced by this decision log.
