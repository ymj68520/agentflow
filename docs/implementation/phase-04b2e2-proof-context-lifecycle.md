# Phase 4B.2E.2 — Proof Context and Invalidation

## Exact process-local preconditions

The checked export, immutable `StaticComputationResolutionSnapshot`, root
derivation (including RootRecoverySummary), and caller context must use the
same pointer-identical `StaticContractProofContext`. Equivalent-looking
objects are not interchangeable. Pointer identity is checked only
in-process; it is not part of a durable or cross-process identity.

`ProofContextManifest::capture` separately snapshots complete typed content:

- compiled modules and TypeSchemaUniverse contents;
- complete operation sets for Effect catalogs;
- recognized ResourceTypes, exact schema pins and attribute vocabularies;
- Flow label/taint/sink catalogs;
- Usage meter/unit/aggregation descriptors;
- Recovery operation/reconciliation catalogs;
- Execution capability/version/attribute, trust, region and runtime-profile
  universes;
- all present relation catalog pins plus exact canonical bytes;
- resolver component pins and every subflow target, direct dependency and
  optional export pin.

The manifest is compared structurally with a fresh capture. It does not
contain an ambient registry pointer or a caller-supplied summary checksum.

## Recompute on change

The prior result is not reusable if the IR/export pin, any mapping, compiler
semantics version/RuleId/specification content, context universe, relation
content, resolver pin, subflow closure, control ownership, output producer,
resource visibility, or root recovery source/action/profile changes. This
phase implements no cross-process proof cache. Changed or missing inputs cause
fresh derivation, or Unproven if the evidence is unavailable.

An artifact checksum is not a signature. The checker makes no claim about who
authored the assertion or whether a compiled implementation behaves as
declared.
