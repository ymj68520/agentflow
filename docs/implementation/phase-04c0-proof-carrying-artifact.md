# Phase 4C.0 — Proof-Carrying Parent Artifact

All candidate structures below are **DESIGN ONLY / NON-NORMATIVE /
NOT REGISTERED ABI / NOT IMPLEMENTED**. Decision C02 carries reproducible
inputs inline in a future versioned parent, as accepted by D3.

## Candidate ownership

~~~text
PublicationStaticProofBundle (candidate, no wire encoding assigned) {
  exact_semantic_ir_pin: existing SemanticIrDigest
  exact_export_contract_pin: existing ComponentContractDigest
  root_export_assertion: complete versioned portable assertion input
  compiler_semantics_identity: complete semantic content tuple
  proof_context_manifest: complete versioned portable context
  resolver_dependency_manifest: exact source/target/edge/export associations
  body_evidence_dependencies: typed required subjects and evidence commitments
  unresolved_obligations: typed requirement descriptors
  static_proof_result: optional producer diagnostic result, not proof authority
}
~~~

The shape identifies responsibilities; field numbers, type IDs, encoding,
ABI, schema versions and digest payload bytes remain blocking future work.
Redundant pins/manifests inside an assertion and bundle must be exactly equal
or the decoder rejects the candidate; no field “wins” a conflict.

## Field authority and canonical content

| Input/fact | Publisher may supply | Trusted verifier must do | Parent semantic content requirement |
| --- | --- | --- | --- |
| IR/export pins and contents | Exact bytes or immutable referenced assets | Recompute existing canonical digests; validate contracts/IR | Commit exact typed pins and schema versions; referenced content must be available |
| Complete assertion/maps | Candidate typed witnesses | Check every source/target coordinate against fresh checked inputs | Inline the full canonical input tuple; not an assertion digest |
| Compiler semantic identity | Claimed exact semantic bundle | Compare supported static/lowering/profile content and current qualified verifier | Commit sources, registry bytes, semantics versions and their exact roles |
| Context manifest | Proposed schemas/catalogs/relations | Reconstruct, recapture, validate content and approved origins | Inline complete typed content or formally specified exact immutable references |
| Resolver dependency manifest | Claimed nodes and edges | Derive closure from actual pinned IR and checked assets | Commit every source/target association, optional presence and direct/transitive edge |
| Reusable structural witnesses | Ports, use sites, exact type/selector/action/profile facts | Revalidate them; no authority comes from their origin | Inline all witnesses needed for deterministic re-execution |
| Producer proof report/result | Diagnostics | Independently recompute preflight and every dimension | Not a trusted proof input; excluded from semantic core, may be archived separately |
| Body/implementation evidence | Typed exact subjects and evidence envelopes | Check subject, predicate, issuer scope, proof content, freshness and closure | Commit required subjects/predicate types and fixed evidence content commitments |
| Unresolved obligations | Typed proposed obligations plus diagnostic text | Recompute completeness and classify mandatory publish/deploy/runtime roles | Commit semantic obligation templates; reason strings are informative |
| Signature/build provenance/release result | Separate authenticated envelopes | Validate authoritative trust roots and policy | External envelopes bind the finalized parent identity; no self-referential signature field |

The chosen canonical core uses the **reproducible input** alternative in D3,
not caller-authored verdicts. Excluding diagnostics does not remove proof
dependencies: every fact consumed by the derivation remains committed.

## Result handling

A copied ProofReport is not a portable proof certificate. A future verifier
may archive a compact static result with the exact parent subject, semantics
and checker build. Consumers either run option A or verify a separately
authorized evidence predicate; no `static_compatibility=Proven` field can
issue a release permission.

Static preflight Violation/Unproven, a non-Proven static dimension, missing
required asset, unsupported semantics, or stale evidence cannot discharge a
mandatory positive gate. Body RequiresEvidence is a separate unresolved
requirement. Diagnostic strings are never parsed into grants or exceptions.

## Content identity and signing graph

~~~mermaid
flowchart LR
  I["IR / export / PlanCore / LockGraph pins"] --> A["Canonical semantic parent core"]
  Q["Inline assertion + exact contexts + associations"] --> A
  B["Body evidence subjects + stable content commitments"] --> A
  A --> H["Future artifact content identity<br/>schema must first be specified"]
  H --> S["Publisher statement / signature"]
  H --> V["Fresh verifier result / release decision"]
  D["Presentation, source maps, diagnostic archives"] -. "nonsemantic attachments" .-> H
~~~

The existing artifact domain is a candidate for that parent after a separately
authorized schema specifies the full ACE1 payload. PlanCore and LockGraph use
their already registered domains only after their own models exist. No
RootContractDigest, WorkflowContractDigest or RootExportAssertionDigest is
introduced.

Publisher signatures and release decisions refer to the finalized content
identity and typed subject. They are external envelopes, so constructing them
does not recursively change the content they sign. Body evidence fixed inside
the core must concern the exact child or root semantic-input tuple, not its own
enclosing parent digest. If an evidence requirement needs the finalized parent
subject, its authenticated envelope belongs outside the core with a pinned
predicate requirement. A formal subject-reference schema must encode this
acyclic distinction.

## Retrieval and substitution

Retrieval hints, registry aliases, URLs and version labels do not choose
content at verification time. Missing pinned content yields MissingArtifact;
different bytes yield an integrity contradiction. Locked rebuilding may use a
mirror only when it yields the exact pinned content. A new dependency
association or semantic parameter produces a new parent core.

No implementation currently persists this bundle. The missing parent/portable
schemas and evidence commitment encoding remain BLOCKING.
