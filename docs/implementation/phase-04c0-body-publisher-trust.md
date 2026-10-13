# Phase 4C.0 — Body, Implementation, Build and Publisher Evidence

All proposed subjects, evidence predicates and validators are **DESIGN ONLY /
NON-NORMATIVE / NOT REGISTERED ABI / NOT IMPLEMENTED**. Decision C05 separates
authentication from the meaning and soundness of a claim.

## Claims and owners

| Claim | Exact subject and required verifier | Current repository status |
| --- | --- | --- |
| Root declarations fit export | IR/export/assertion/context/resolver tuple; fresh RootExport checker | Implemented in process |
| Target semantic body fits its export | Target IR/export plus complete leaf/subflow proof assumptions; future body verifier | RequiresEvidence |
| Plan implements the semantic body | Exact PlanCore, source IR, LockGraph and lowering/profile tuple; future preservation verifier | NotImplemented |
| Component implementation conforms | Exact implementation artifact/package, contract and execution/profile assumptions; trusted scoped conformance predicate | RequiresEvidence |
| Publisher authenticated | Exact parent content subject and logical publication namespace/action; approved authentication roots | NotEstablished |
| Artifact intact | Exact schema and canonical content hash, plus transport bytes if different | Core hashing exists; publication artifact model missing |
| Build provenance | Exact compiler/build outputs, inputs/recipe and qualified builder identity | NotImplemented |
| Release authorized | Exact subject plus all evaluated inputs under authoritative release policy | NotImplemented |

Resolving a CALL_SUBFLOW export does not prove its body. Verifying a signature
does not verify the body predicate. Reproducible build equality establishes
agreement of outputs under recorded inputs; it does not establish behavioral
conformance without the relevant verifier.

## Candidate body dependency

~~~text
BodyConformanceDependency (candidate, no registered schema) {
  target_semantic_ir_pin: exact existing SemanticIrDigest
  target_export_contract_pin: exact existing ComponentContractDigest
  target_proof_inputs: exact portable assertion/context/semantic-input tuple
  direct_dependencies: Known exact association rows, or Unavailable
  transitive_closure: complete exact target/export/evidence subject graph
  evidence_predicate: exact required body relation and version
  evidence_content_commitment: optional exact immutable evidence payload
  trusted_issuer_requirement: predicate-specific identity/role/scope policy
  freshness_requirements: semantic/version/revocation applicability rules
}
~~~

Publication validates the target's own pinned IR and evidence assumptions
recursively. A bare RootExport target assertion proves only declared envelope
compatibility. Full body evidence must either discharge its implementation
assumptions or explicitly carry them as typed obligations under the chosen
claim. A conditional composition theorem must not be labeled unconditional
implementation conformance.

The semantic dependency graph is acyclic for this prospective v1 boundary.
Cycles, truncated transitive closure, unavailable targets and conflicting
subject associations block the gate. Known empty leaf closure is valid.
Evidence graph validation must prevent mutually self-supporting “trusted”
attestations from discharging missing premises.

## Required versus deploy-bound evidence

The chosen boundary permits internal non-executable analysis archives with
unresolved body evidence; it does **not** permit a publishable WorkflowVersion
to omit a mandatory body predicate.

At Publish, every reachable subflow body relation must be established for its
exact IR/export/context/semantic closure, and every fixed implementation used
by the semantic build must meet required conformance predicates. A signature
over an unverified predicate is still missing conformance evidence.

An implementation that is intentionally selected at deployment is a typed
binding variable with exact contract, allowed compatibility/profile constraints
and mandatory implementation evidence requirements. It has no conformance
claim at publication before a concrete binary is bound. Such deferral is
allowed only when a future formal release profile explicitly establishes the
conditional semantic claim and requires the exact binary evidence at Deploy;
the ordinary “required body evidence missing” case cannot be relabeled as
deferred. Without that quantified/conditional obligation model, abstract
implementation deferral is NotImplemented and blocks the chosen release profile.

This preserves architecture sections 13.5 and 17.7's separation between semantic
dependencies and deployment-selected implementations while keeping missing
mandatory evidence visible.

## Verification and invalidation

For each evidence envelope, independently check:

1. Its schema/predicate version and exact subject including typed content
   identity, contract, context/semantics and applicable dependency projection.
2. The authenticated issuer/key or workload identity and its authority for this
   predicate, package/tenant/namespace and assurance scope.
3. The predicate's proof content or trusted claim semantics; a producer-supplied
   `verified` bit is not evidence.
4. All evidence dependencies and their source associations.
5. Applicable issuance time, trusted timestamp, revocation and policy freshness.
   Expiring certificates alone do not define historical signature semantics;
   the selected authentication scheme must specify verification time rules.
6. The current authoritative release policy's acceptance of the claim.

Changed body, export, implementation binary, context, normative content,
dependency edge or required predicate invalidates the applicable evidence.
Changed trust/revocation policy can change current eligibility without changing
the immutable program. Unavailable evidence stays RequiresEvidence/Unproven;
contradictory subject or invalid proof is a known rejection.

## Authentication alternatives

| Direction | Useful guarantee | Required additional facts |
| --- | --- | --- |
| Publisher signs a typed declaration of exact parent and publication action | Authenticates the claimed publisher/subject relation | Approved identity roots, namespace/role authorization, fresh release policy |
| Independent qualified builder/verifier produces provenance | Authenticated build inputs/outputs and verification execution | Trust in builder/verifier, exact semantic support and proof soundness |
| Reproducible verification under option A | Consumers recompute the static relation | Approved input origins, exact semantic model and qualified checker |
| Scoped conformance attestations | A distinct body/implementation predicate | Trusted predicate-specific issuer and verified assumptions/subject closure |

Sigstore verification checks identity/issuer, signature and artifact claims;
this is a suitable authentication direction, not a behavioral verifier.
[Sigstore verification documentation](https://docs.sigstore.dev/cosign/verifying/verify/)

SLSA verification compares artifact/provenance subjects, trusted builders and
expected build inputs. The selected AgentFlow closure requirement is stricter
than treating available dependency provenance as sufficient.
[SLSA v1.2 artifact verification](https://slsa.dev/spec/v1.2/verifying-artifacts)

An in-toto Statement binds immutable subjects to a typed predicate; the
consumer must separately interpret and verify that predicate.
[in-toto Statement v1](https://github.com/in-toto/attestation/blob/main/spec/v1/statement.md)

No external service, credential integration or standards compliance is
implemented or claimed.

## Digest adaptation and signing subjects

Plain SHA-256 of transport/blob bytes and HashV1(artifact, canonical payload)
are different computations. An external signature adapter must bind the exact
canonical schema, semantic parent identity, and actual transport subject and
recompute their association. Changing a domain label on a blob checksum is not
a valid conversion.

Publisher statements also bind the intended logical Definition/version
reference and action/role to prevent reuse as a different tenant's release
authorization. Signatures and release results are outside the canonical
semantic parent to avoid recursive identities. Evidence issued before parent
construction binds exact semantic-input tuples; evidence issued afterwards may
bind the parent. Authoritative trust roots never come solely from the
publisher-provided bundle.
