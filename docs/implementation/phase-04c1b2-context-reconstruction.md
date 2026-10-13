# Phase 4C.1B.2 — Exact Context Reconstruction

D5 resolves a `context_id` only from D4-supported portable data. It builds a
new owned arena and compares a recaptured projection before returning it.
`ReconstructedStaticContext` has private fields; callers can inspect its
projection or enter `with_proof_context`, which constructs a short-lived
`StaticContractProofContext` borrowing the exact owned arenas.

## Schema compilation

SchemaAssets are decoded with the registered TypeSchema shape, checked against
their exact `TypeSchemaRef` and recomputed TypeSchemaDigest, and indexed by
full `(SpecificationId, TypeName)`. The global schema universe is inserted
from its own complete list, after duplicate prechecks.

For `compiled_modules`, each declared module member is selected by exact pin.
The implementation prints a deterministic AFIDL projection: module IDs are
exact, external imports are sorted and assigned stable aliases, declarations
are emitted by local name, and fields/cases by their existing ordinal. The
existing `afidl::compile_module_set` performs import/name resolution and global
type-cycle validation. Every compiled member list and canonical schema byte
string is compared back to the selected assets. The formal ComponentContract
and Semantic IR root schemas are mandatory.

`TypeSchemaUniverse` and `ResourceTypeUniverse` membership are independent.
The latter now has an exact single-schema insertion method so D5 never imports
all members of a compiled module as a side effect. No new compiler or registry
lookup is introduced.

Before compilation, D5 enforces at most 256 modules and 1,024 selected module
types. Global alias expansion is checked iteratively with a 256-hop depth cap
and a 1,000,000-node work budget, so a hostile alias chain/cycle cannot reach
the formal checker's recursive expansion without an explicit D5 bound.

To bound repeat work, the process-local schema and compiled-module caches use
exact length-delimited input material as their cache identity and retain one
immutable entry each. A cache hit is only a reuse of already reconstructed
typed content; it does not bypass input-byte equality or D5 recapture.
