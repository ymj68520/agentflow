# Phase 4C.1A — Portable Static Proof Input Formal Model

Every candidate type, field-number proposal, algorithm and entry point in this
package is **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED ABI / NOT IMPLEMENTED**.
No new module, registry entry, hash domain or Decoder is created.

## Scope and result

The package specifies MODEL-B01/B02 for the **full-content static RootExport
revalidation profile**. It inherits C01 full re-execution, C02 complete inline
inputs and C03's separate LockGraph/PlanCore prerequisites.

MODEL-B01 and MODEL-B02 reach **DESIGN ACCEPTED within this profile**, subject
to the exact definitions in these fifteen documents. Implementation closure is
still BLOCKING: registered schemas, production projection/Decoder, constructor
bridges, cross-language goldens and production differential do not exist.
PlanLoweringSemantics and a full RuntimeSemanticsProfile remain ModelMissing;
their absence is represented, never converted into support. MODEL-B03–B07
remain BLOCKING.

Start HEAD: `89b4f3b41bde82a0bf15240b2b9ddb9013095c01`; initial tree clean.
Baseline: 41 specifications, 380 rules (379 covered, one intentional exception,
zero missing), 477 ABI assignments, 16 AFIDL modules and 14 HashV1 domains.
Historical production differential: 58/72/55/44/35/47 = 311 cases.

## Model ownership

The Bundle owns complete inline canonical IR/contract/schema assets, typed
context data, resolver inventories/associations, compiler content and assertion
witnesses. Local ID tables represent finite graphs rather than recursive wire
records. Callers submit declarations. Closed decoding produces raw model values;
formal validators alone create Checked contracts/IR; fresh derivation and
RootExport checking alone create the local checked static capability.

Catalog classifications and presence variants are raw semantic data, not
approval flags. No field accepts checked=true, trusted_context=true or a
submitted Proven verdict as proof.

## Source checks and inherited constraints

Source review covered:

- `af-computation/src/root_export.rs:55–329,1041–1273`: every assertion
  field, exact sort keys, manifests and dependency checks.
- `af-computation/src/resolution.rs:111–212`: target/export/dependency
  association and Option-preserving resolver identity.
- `af-computation/src/summary.rs` and `root_recovery.rs`: Flow bindings,
  host-size ordinals and action source identities.
- `af-idl/src/canonical.rs` and `resolve.rs`: canonical type grammar,
  immutable compiled module contents and finite shape construction.
- `af-semantic/src/types.rs` and `universe.rs`: structural types and
  nominal schema bodies/pins.
- Effect, Resource, Flow, Usage, Recovery, Execution universe implementations,
  and `af-contract/src/universe.rs`: every catalog field and relation snapshot.

Normative inputs are existing ACE1, HashV1, AFIDL1, semantic-core/IR,
ComponentContract and dimension/relation schemas and rules. C0 revalidation,
compiler identity, inline artifact, decision/readiness reports constrain the
design. F01/F02 remain CLOSED; F03/F04 remain OPEN/P2.

## Implementation-relevant discoveries

| Current representation | Required portable treatment |
| --- | --- |
| CompiledModuleSet / TypeSchemaUniverse private maps | Complete canonical schema assets and explicit per-universe membership; no Rust memory serialization |
| Resource universe's private schema set | Separate membership from context TypeSchemaUniverse |
| RecoveryOperationUniverse.effects | Separate complete effect-catalog set, not inferred equal to context effects |
| Execution capability exact_version | Preserve explicit field and register it first during reconstruction |
| usize source/occurrence ordinals | uint64 in the model, checked host conversion; no narrowing |
| StructuralType recursive Box | Finite typed node table; no recursive AFIDL type |
| Nested target assertion identities | Acyclic assertion ID graph expanded back into complete typed tuples |
| ContextManifest dependency default-to-empty helper | Portable Option preserves unknown versus known-empty; positive reconstruction requires known edges |
| Procedural comparisons | Fixed supported semantic algorithms plus complete typed data; no function pointer/code payload |

Private enumeration APIs and inverse projections are implementation bridges for
4C.1B, not missing data semantics. No present field requires an arbitrary callback
to reconstruct. Unsupported future relationships remain unsupported.

## Package map

| Document | Definition owner |
| --- | --- |
| [Portable schema](phase-04c1a-portable-proof-schema.md) | Bundle, assets, aliases, table ownership and common field rules |
| [Assertion projection](phase-04c1a-root-export-assertion-projection.md) | Complete mapping types and current Rust field correspondence |
| [Context manifest](phase-04c1a-proof-context-manifest.md) | All universes, independent scopes and relation slots |
| [Resolver manifest](phase-04c1a-resolver-dependency-manifest.md) | Source/target/export association graph and derived closure |
| [Canonical encoding](phase-04c1a-canonical-encoding.md) | Exact ACE1 rules, comparators and schema-content grammar |
| [Closed Decoder](phase-04c1a-closed-decoder.md) | Stage-specific rejection and resource limits |
| [Reconstruction](phase-04c1a-context-reconstruction.md) | Pure construction recipes and fresh trusted objects |
| [Compiler identity](phase-04c1a-compiler-semantics-identity.md) | Role/content/attribution schema and current projection |
| [Version support](phase-04c1a-version-support.md) | Exact local support table and historical semantics |
| [Origin boundary](phase-04c1a-origin-trust-boundary.md) | Separate raw claims, build identity and missing trust evidence |
| [Vectors](phase-04c1a-decision-vectors.md) | 60 new design scenarios, all Not Executed |
| [Decisions](phase-04c1a-decisions.md) | Ten machine-model decisions |
| [Readiness](phase-04c1a-readiness.md) | Design versus implementation closure and 4C.1B prerequisites |
| [Gates](phase-04c1a-gate-report.md) | Required source/corpus regression, freeze and commit checks |

The static result, content integrity, origin authentication, binary qualification
and release eligibility remain separate outcomes. This phase stops at model
design.
