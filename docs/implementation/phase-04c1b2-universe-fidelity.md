# Phase 4C.1B.2 — Universe Fidelity

The current `StaticContractProofContext` borrows nine independent inputs. D5
owns each one separately and compares the full member projection after using
the existing constructors.

- Global named schemas and Resource-private schema membership are distinct.
- Context EffectCatalogUniverse and RecoveryOperationUniverse.effects are
  separately populated from separately declared Portable rows.
- Flow confidentiality labels, taints, and effect sink classification are
  distinct sets; an unknown external source is not rewritten to empty.
- Usage aggregation preserves `Unsupported`; units remain exact identities.
- Execution capability `exact_version` is not inferred from sorted versions.
  The exact version is inserted first, then the remaining versions. Attribute
  key duplicates are rejected before the constructor can coalesce them.
- Trust domains and regions are exact sets. Region compatibility remains
  directional; the builder cannot auto-add endpoints absent from the declared
  closed region set.
- Runtime profile family and full feature vocabulary are retained.

Read-only projection accessors were added to semantic, resource, flow, usage,
recovery and execution universes. `ResourceTypeUniverse::insert_schema_type`
is the only new builder operation needed to reconstruct a strict subset.
These Rust APIs do not alter AFIDL or serialized bytes.

Construction limits are explicit: 16,384 SchemaAssets, 100,000 collection
items, 256 compiled modules, 1,024 selected module types, and the D5 alias
expansion cap in the context-reconstruction note. Constructor-induced
normalization is checked by recapture before the context becomes available.
