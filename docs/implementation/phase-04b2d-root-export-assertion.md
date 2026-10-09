# Phase 4B.2D — RootExportAssertion Design

All candidate records and rules here are **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED ABI / NOT IMPLEMENTED**.

## Decision D3

Choose **Option A: an explicit typed build-time export assertion** outside the Semantic IR wire document. The assertion is supplied to a future root checker alongside a checked SemanticIrDocument and an exact checked exported ComponentContract. It pins both existing digests and every cross-coordinate witness needed before a dimension can be compared.

It is not a ComponentContract, does not produce a WorkflowContractDigest or RootContractDigest, and does not alter SemanticIrDocument, ComponentContract, ACE1, or HashV1. The assertion's identity is its complete typed input tuple; it has no standalone digest.

## Candidate shape

~~~text
RootExportAssertion {
  semantic_ir_digest: DigestRefValue,                // sha256 / semantic-ir
  exported_component_contract_digest: DigestRefValue, // sha256 / component-contract
  compiler_semantics_identity: CompilerSemanticsIdentity,
  proof_context_manifest: ProofContextManifest,
  input_mappings: Vec<RootInputPortBinding>,
  output_mappings: Vec<RootOutputPortBinding>,
  resource_mappings: Vec<RootResourcePortBinding>,
  effect_mappings: Vec<RootEffectCoordinateBinding>,
  flow_mappings: Vec<RootFlowCoordinateBinding>,
  recovery_mappings: Vec<RootRecoveryCoordinateBinding>,
  execution_mappings: Vec<RootExecutionCoordinateBinding>,
  subflow_dependencies: Vec<SubflowProofDependency>
}

RootExportProofReport {
  exact_assertion_input_identity: RootExportAssertionIdentity,
  dimensions: Vec<DimensionProof>,
  body_conformance: ProofVerdict
}

SubflowProofDependency {
  call_site: ComputationId,
  target_semantic_ir_digest: DigestRefValue,
  target_exported_contract_digest: Option<DigestRefValue>,
  target_root_assertion_identity: Option<RootExportAssertionIdentity>,
  transitive_targets: Vec<SubflowTargetIdentity>
}

SubflowTargetIdentity {
  semantic_ir_digest: DigestRefValue,
  exported_contract_digest: Option<DigestRefValue>
}
~~~

Every field is typed and exact. Canonical ordering is by target namespace and then complete source identity; duplicate targets are rejected. The complete identity includes the two exact DigestRefValues, semantics identity, full context manifest, all maps, and exact dependency pins. No free string map, untyped extra field, or caller-supplied proof boolean is accepted.

The candidate record owner is a compiler/build boundary, not a node in the checked IR. It consumes a successful root derivation from that exact IR and exact context. A missing or changed input means a new assertion identity and a fresh derivation.

RootExportAssertionIdentity is the complete tuple of the two root/export digests, compiler semantics identity, full context manifest, all canonical maps, and sorted subflow dependency pins. DimensionProof is the existing typed diagnostic record; its reason/prerequisite strings are explanations only and never participate as proof identity. Each subflow dependency is keyed by call-site ComputationId and exact target digests. A target assertion identity is a reference to the complete target tuple, not a newly invented digest. TransitiveTargets is sorted by the exact target digest pair and cannot omit a reachable dependency.

SubflowProofDependency carries only exact pins and dependency identities; it cannot carry a caller-supplied verdict. RootExportProofReport.body_conformance is analyzer output. A target RootExportAssertion identity proves at most that target's static envelope comparison; absent a separate trusted body proof, body_conformance remains RequiresEvidence.

## Assertion inputs and outputs

Required inputs:

1. A CheckedIrDocument and its recomputed exact SemanticIrDigest.
2. The exact exported checked ComponentContract and its verified ComponentContractDigest.
3. The exact compiler semantics identity used for derivation and comparisons.
4. A content-complete typed proof-context manifest.
5. Complete typed I/O/resource mappings and source-preserving E/F/K/X mappings.
6. The exact resolution snapshot for component/subflow targets; its proof context must be the same object used to check the export and derive the root.

Outputs are per-dimension proof facts and prerequisites, not a new contract. The report preserves Proven, Violation, Unproven, and RequiresEvidence separately. A static envelope compatibility result may be Proven while body conformance is RequiresEvidence; the two are never merged into a publisher or runtime assertion.

## Proof direction

After coordinate validation, the body must fit the public ceiling:

* Root I/O must implement the exported public interface without adding a required input or omitting an advertised output.
* Root possible Failure types must be within exported Phi.
* Root epsilon and each root K-action projection must be within the exported epsilon ceiling, including exact guards and full atoms.
* Mapped root resource requirements must not be stronger than the export.
* Root information flow must not relax exported restrictions.
* Root Forward and Recovery Usage must each fit their exact exported coordinate.
* Root Recovery safety and full action/profile declarations must not be weakened.
* Every child execution profile must meet exported X feasibility and security constraints, separately.

No direct call to ComponentContract substitution is sufficient. That API compares two ComponentContracts; it does not establish that a workflow body, root output, ValueSlot, ResourceSlot, control path, or K-source multiset maps to a contract.

## Alternatives

| Option | Decision |
| --- | --- |
| A. Explicit build-time assertion | **Accepted.** Keeps the frozen Semantic IR ABI unchanged and makes the missing cross-coordinate evidence an explicit, typed input. A result is valid only for the exact pinned IR, export, semantics, context, and maps. |
| B. Versioned Semantic IR export reference | Rejected for this phase. It changes the frozen wire ABI, adds no intrinsic mapping semantics, and couples a publication relation to the IR object. Reconsider only if a later release model proves an external assertion inadequate. |
| C. Deferred publisher attestation | Rejected as a replacement. It may attest implementation/body facts through a separately trusted mechanism, but cannot replace static root/export comparison or silently become Proven. |

## Identity, canonicalization, and persistence

The two existing digests retain their original meanings: exact canonical Semantic IR content and exact canonical ComponentContract content. Compiler semantics and static proof assumptions are separate identities; neither is implied by those content digests.

The assertion has a deterministic typed identity tuple, but no new hash domain or independent digest. A proof report is reusable only under the same tuple and exact checked context. A result from a different compiler semantics, catalog snapshot, mapping, export, or IR is not reusable.

The current repository has no frozen WorkflowVersion type and the current ProofContextReference is pointer-identical in-process. The selected persistence rule is therefore prospective and explicit:

1. A future additive, versioned WorkflowVersion/release artifact owns the assertion inline, including its complete proof-context manifest and report or reproducible proof inputs.
2. The parent artifact's canonical content identity must commit the complete assertion and manifest. The already registered artifact HashV1 domain is the only candidate domain; the parent schema and its canonical payload must be specified in a later authorized publication phase before it is used.
3. No standalone RootExportAssertion digest is created. If no versioned parent with a content identity exists, the assertion is transient and cannot be reused across processes or releases.
4. A publisher signature is a separate authentication claim. Neither an artifact digest nor a relation checksum is a signature.

This is an accepted design rule, not a claim that the current checkout can persist or authenticate an assertion.

## Body, Subflow, Authority, and Budget boundaries

For each CALL_SUBFLOW dependency, pin the exact target SemanticIrDigest and exact exported ComponentContractDigest, and retain the exact target IR/export resolution and transitive dependency pins. A target body's conformance to its export is a separate ProofVerdict and proof dependency. Missing, stale, cyclic, or unauthenticated body evidence remains Unproven or RequiresEvidence according to the missing fact; resolving the target digest alone is not body conformance. A target RootExportAssertion proves only its static envelope relation, not that an implementation binary conforms.

The assertion contains no authority-ceiling or budget-ceiling field because frozen ScopeBody has neither. Authority attenuation remains Unproven. Static Usage/cost envelopes remain distinct from runtime budget reservations and actual settlement. No absent grant, budget, or profile becomes infinity or authorization.

## Failure closure

* Digest mismatch or a proven public-contract contradiction: Violation.
* Missing coordinate map, unknown relation, incomplete context manifest, or unsupported compatibility direction: Unproven.
* Missing receipt, trusted implementation/body attestation, or runtime admission evidence: RequiresEvidence.
* Complete proof inputs and every required dimension Proven: static root/export compatibility may be Proven, but runtime authority and body conformity are still independent claims.

The ten-document design package specifies the required typed maps and dimensions. No production assertion type or checker is implemented here.
