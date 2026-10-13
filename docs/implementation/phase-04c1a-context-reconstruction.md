# Phase 4C.1A — Exact Context Reconstruction and Fresh Checking

All prospective procedures/outputs are **DESIGN ONLY / NON-NORMATIVE /
NOT REGISTERED ABI / NOT IMPLEMENTED**. Construction uses explicit validated
data and existing pure algorithms, not persisted Rust object bytes.

## Preconditions and scope arenas

D0–D4 establish closed canonical shapes, content identity, exact reference
closure and supported static semantics. They establish no publisher/body truth.
All required base collections must be Complete. Schema assets are exact named
bodies/pins; row membership determines which types are visible in each scope.

Allocate immutable owned backing stores for each ContextId, then exactly one
StaticContractProofContext referencing those stores. Contracts, snapshots,
Root and RootRecovery for that context use the same new local context pointer.
Other contexts get separate arenas; equal content does not make their checked
objects pointer-interchangeable.

## Pure reconstruction recipes

| Current object | Exact recipe / prevalidation |
| --- | --- |
| CompiledModuleSet | Decode canonical schema assets; validate each module's selected types and global named-ref graph; compile the deterministic AFIDL projection below; compare every regenerated body/pin and module member list |
| TypeSchemaUniverse | Insert exactly its listed canonical types, pre-reject duplicate logical identities; do not union with all globally available types |
| ResourceTypeUniverse | Independently insert its own listed schema membership grouped by module, then recognize exactly its descriptors; do not clone context TypeSchemaUniverse implicitly |
| EffectCatalogUniverse | Build each complete catalog with existing grammar validation; unique catalogs/operation names |
| FlowCatalogs | Recognize exact separate confidentiality/taint sets and effect sink classification rows |
| UsageMeterUniverse | Register exact meter/unit/aggregation descriptors; Unsupported stays Unsupported |
| RecoveryOperationUniverse | Rebuild its separate EffectCatalogUniverse and exact reconciliation set |
| ExecutionCapabilityUniverse | Register each exact_version first; then remaining supported versions in canonical order with the same complete attributes |
| TrustDomainUniverse | Register exactly supplied nominal members |
| RegionUniverse | Register exact regions first; precheck every directional compatibility endpoint, then register pairs |
| RuntimeProfileUniverse | Register exact profile/family/features; these are static catalog descriptors |
| RelationSnapshots | For each Some slot decode existing typed relation, construct snapshot with exact revision, compare regenerated canonical bytes/plain checksum/header; None stays absent |

Several current constructors normalize, deduplicate or overwrite entries.
The portable validator rejects noncanonical identities/duplicates before
calling them. It never uses that behavior as decoding semantics.

Independent schema universes may contain a subset of named types. A missing
named-reference member is not supplied from another scope: resolution returns
unknown when demanded by the formal validator. CompiledModuleSet itself must
be fully resolved/acyclic as required by AFIDL. Cyclic selected alias/type
expansion is rejected; no infinite shape is constructed.

## Deterministic canonical-schema to AFIDL projection

This recipe enables the existing `compile_module_set` API without restoring
its private map. A future lossless validated constructor is an implementation
alternative only if it produces the same canonical projection.

1. Group the module's exact selected CanonicalTypeSchema values by schema SpecId.
   Verify schema/name fields, unique names/numbers, allowed identifiers and
   full named-ref targets before printing.
2. Collect external specification IDs referenced by its type expressions,
   sort UTF-8, assign aliases m1, m2, ...; self refs use local names.
3. Print `afidl 1;` and the exact schema declaration, then sorted imports.
4. Print declarations by local type name. Fields/cases/enum values appear by
   their existing positive number. Aliases retain their own declaration.
5. Print primitive keywords exactly; ref expressions use TypeName if local,
   otherwise mN.TypeName; list/map<string,T>/option are structural expressions.
6. Use LF, conventional spaces, decimal number tokens and no comments.
   Labels used to call the compiler are derived diagnostic SpecIds only.
7. Compile all module texts together. An import cycle is legal only when the
   complete named type graph is acyclic. Compare resulting canonical ACE1
   schema bytes/digests and full module membership with input assets.

Source formatting/import aliases are not schema identity. Named-reference
bodies retain full identities; they are never expanded into a different
nominal alias or substituted by “latest”.

Known registered machine-family shapes must additionally match the exact
AFIDL normative source set in the selected compiler profile. A caller cannot
redefine a reserved schema family merely by giving its new body a valid hash.
Custom data/resource schema scopes stay explicit and origin-unapproved.

## Resolver and assertion reconstruction

After contracts are newly checked, validate caller and all target IR documents
under their exact schema scopes and run Phase 3C. Populate SubflowIrUniverse
using those checked targets; its insert recomputes target digest and direct
CALL_SUBFLOW references. Compare all supplied known edges/transitive sets.

Before retaining either checked contract or IR as the referenced asset,
recompute its existing semantic canonical bytes/digest and compare with the
supplied content. Generic ACE1 validity alone does not establish semantic table
order, qualifier/scope normalization or zero-effect normalization. A difference
is SemanticCanonicalMismatch; never update the pin or ordinal silently.

Build StaticComputationResolutionSnapshot using the new context, complete
component inventory and exact target/export associations. Recompute its
identity rather than loading one. Validate all caller/node source associations.
Any missing/unknown edge or unsupported cycle stops the positive path.

Expand StructuralTypeId witnesses into finite StructuralType values. Expand
optional target AssertionIds bottom-up into complete boxed typed identities.
Construct raw RootExportAssertion fields from those values, complete compiler
projection, and a newly captured manifest. Recapture must match the input base
context and resolver projection exactly, including optional states.

Explicit None/unknown facts that the current manifest's helper could collapse
are checked **before** capture. They are never accepted by projecting unknown
to the helper's default empty vector.

## Fresh validation state machine

~~~mermaid
flowchart TD
  B["Raw portable bytes: caller constructible"] --> D["D0-D4 closed data/content/support validation"]
  D --> C["D5 exact local context arenas"]
  C --> K["D6 check_contract: new Checked contracts"]
  K --> I["D7 checked caller/targets + Phase 3C"]
  I --> S["D8 fresh exact resolver snapshot"]
  S --> R["D9 derive_workflow_contract_summary"]
  R --> E["Joint Root + RootRecovery + resolver identity"]
  E --> P["D10 check_root_export_assertion"]
  P --> V["Fresh report; local checked capability only if static Proven"]
  V --> O["Body/implementation/origin/runtime: separate obligations"]
~~~

The five static preflight categories and all nine dimensions remain mandatory.
The report's separately appended body evidence item remains RequiresEvidence
unless another future trusted body verifier discharges that separate claim;
it never changes the static Checker result or creates publication rights.

## Roundtrip and failure obligations

For every complete supported v1 input in the projection domain:

~~~text
project(reconstructed contexts/snapshot/raw assertion) =
    expanded Portable Bundle semantic input projection
encode(decode(canonical Bundle bytes)) = same bytes
~~~

No extra bootstrap module, default catalog, inferred alias, hidden region,
current config or resource binding is inserted. A projection mismatch stops
checking. Schema/source identity, relation content, resolver association and
all typed witnesses participate in invalidation. New process pointers and
diagnostic labels do not participate in cross-process equality.

Current read-only snapshot accessors and inverse decoders still need
implementation in 4C.1B. These recipes are a model closure, not a claim that
current Rust already runs this pipeline.
