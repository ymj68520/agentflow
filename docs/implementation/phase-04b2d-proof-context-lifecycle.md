# Phase 4B.2D — Proof Context and Assertion Lifecycle

All candidate identity records and lifecycle rules here are **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED ABI / NOT IMPLEMENTED**.

## Identities that must remain distinct

| Identity | What it commits or names | What it does not commit |
| --- | --- | --- |
| ComponentContractDigest | Exact canonical ComponentContract body under the existing component-contract domain | Static catalogs/relations, compiler semantics, implementation bytes, or a proof result |
| SemanticIrDigest | Exact canonical Semantic IR document under the existing semantic-ir domain | External contract contents beyond exact references, proof context, or body behavior |
| CompilerSemanticsIdentity | Exact set of frozen SpecificationIds and RuleIds applied by the checker | Compiler binary correctness or a publisher signature |
| ProofContextManifest | Complete typed content/identity of static schema, catalog, relation, and execution inputs | Runtime state or trustworthiness of whoever supplied it |
| RootExportAssertionIdentity | The complete two-digest tuple, compiler identity, manifest, coordinate maps, and exact dependencies | A standalone cryptographic digest or public contract |
| StaticProofResultIdentity | Assertion input identity plus all per-dimension verdicts, prerequisites, source references, and witnesses | Body attestation, effect receipt, grant, reservation, or admission |

No two rows above are interchangeable. No new HashV1 domain is created.

## Current process-local boundary

StaticContractProofContext contains explicit references to CompiledModuleSet, TypeSchemaUniverse, EffectCatalogUniverse, ResourceTypeUniverse, FlowCatalogs, UsageMeterUniverse, RecoveryOperationUniverse, ExecutionUniverses, and RelationSnapshots. CheckedComponentContract and the current ProofContextReference use exact object identity; relation catalog pins are retained in reports.

That pointer identity is intentionally process-local. It prevents reuse across separately assembled contexts in the current process, but it cannot be serialized, compared across processes, or treated as a durable content identity. ComponentContractDigest alone does not bind any of these inputs.

RelationCatalogPin already carries an exact schema identity, revision token, and canonical-content checksum. The revision token alone is not enough. The checksum detects changed content but is not a publisher signature or trust proof.

## Candidate content-complete ProofContextManifest

For a future persistent assertion, the manifest is an inline, canonically ordered typed snapshot. It contains:

1. Exact TypeSchemaRefs for every used named schema, including each existing TypeSchemaDigest.
2. Every EffectCatalog identity and its complete sorted operation-name set.
3. Every recognized ResourceTypeRef and exact schema pin, attribute vocabulary, and resource descriptor used by the proof.
4. Flow confidentiality-label, taint, effect-sink, and decision-capable classifications as exact identities and complete sets.
5. Usage meter descriptors: exact meter, unit, and aggregation semantics.
6. Recovery operation/effect catalog content and the complete reconciliation-operation set.
7. Execution capability descriptors, exact and supported versions, attributes, trust domains, regions/compatibility pairs, and runtime-profile descriptors.
8. Every RelationCatalogPin with schema, revision, checksum, plus the exact relation snapshot identity/content bound by that pin.
9. Exact resolver dependencies: each ComponentContractDigest, Subflow SemanticIrDigest, exported ComponentContractDigest, and the context under which it was checked.

Catalogs without a frozen content digest must be embedded as their complete canonical typed member set. A catalog label or revision string is not a substitute. If any context input cannot be serialized canonically and exactly, the assertion remains in-process only and cannot be persisted or reused.

This manifest has an owner, typed namespaces, and deterministic order, but no independent digest. It becomes part of the RootExportAssertionIdentity tuple.

## Compiler semantics identity

The candidate CompilerSemanticsIdentity is the sorted unique list of the exact existing normative SpecificationIds and RuleIds used for root derivation, guard compatibility, control aggregation, dimension comparisons, and context validation. The rule set includes the applicable frozen Effect, Recovery, ComponentContract, Usage, Flow, Resource, Execution, Semantic IR, dataflow, and computation-control rules.

No RuleId or SpecificationId is registered in Phase 4B.2D. A changed normative revision or changed applied rule set changes this identity and invalidates the proof. A compiler binary/build attestation is outside this semantic identity and, if needed, must be a separately trusted release fact.

## Invalidation and recomputation

The following changes invalidate the prior static result:

* SemanticIrDigest, exported ComponentContractDigest, any input/output/resource/effect/flow/recovery/execution map, or subflow target/export pin changes.
* Any schema digest, complete effect/resource/flow/usage/recovery/execution catalog content, relation checksum/content, or applicable catalog identity changes.
* CompilerSemanticsIdentity changes, including a semantic specification or rule-set revision.
* Any control-tree ownership, expression, output mapping, resource-slot visibility, or recovery source/action/profile changes.

The implementation rule is fail-closed: re-run full root derivation and all nine dimension comparisons whenever any member differs or is unavailable. The current process may reuse a CheckedComponentContract only under the exact same context object, as it does today. No cross-process cache reuse is authorized by this design.

## Assertion lifecycle

1. Validate and recompute exact IR/export digests.
2. Assemble the exact StaticContractProofContext and resolution snapshot.
3. Produce a content-complete ProofContextManifest; reject persistence if it cannot be completed.
4. Derive the complete root summary under that exact context.
5. Validate every typed coordinate map, including full action/profile multiset closure.
6. Compare each dimension and retain all verdicts and prerequisites.
7. Bind the report to the complete assertion input identity.
8. For current nonpersistent analysis, keep it ephemeral. For future persistence, store the full assertion inline in a versioned WorkflowVersion/release artifact. Its canonical parent payload must include all pins, maps, manifest, compiler identity, report or reproducible proof input, and subflow dependencies.

The existing artifact HashV1 domain is the only candidate for a future parent content identity because it is already defined for immutable artifact content. The WorkflowVersion schema/canonical encoding does not exist in this phase; a future publication phase must specify it before computing such an identity. No RootExportAssertion digest is created. If there is no suitable parent content identity, do not claim persistence.

An artifact digest or catalog checksum is not a signature. Any claim that the compiler/publisher authored the assertion or that an implementation conforms requires a separate authentication/evidence model.

## Subflow proof dependency

Each CALL_SUBFLOW source retains the exact target SemanticIrDigest and exported ComponentContractDigest. A transitive dependency closure records each target once by the complete digest tuple, and detects missing targets or cycles rather than truncating them.

Root envelope compatibility can use a checked target export where the exact target resolves. Target body conformance is a separate proof: it requires that target's own RootExportAssertion/result under its own exact context and transitive dependency set, or another explicitly trusted body proof. A resolved export digest alone never proves the target body. Missing or stale body proof remains RequiresEvidence or Unproven; it does not upgrade root compatibility.

## Static and runtime boundary

The manifest may contain static catalogs and relation snapshots; it never contains a ResourceHandle, EffectGrant, Worker admission, runtime Usage reading, EffectReceipt, Recovery result, budget reservation, or actual settlement. Static authority attenuation and budget-ceiling checks remain Unproven because frozen ScopeBody has no such fields. Static Usage is a declared upper, not a runtime budget.
