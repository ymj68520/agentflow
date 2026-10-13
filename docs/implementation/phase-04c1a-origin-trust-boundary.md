# Phase 4C.1A — Content, Origin and Compiler Build Boundary

All candidate interfaces are **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED
ABI / NOT IMPLEMENTED**. B01/B02 describe identities and raw claims; B05/B07
still own issuer trust, authentication, qualification and release authorization.

## Separate submission envelope

~~~text
PortableProofSubmission = record {
  proof_inputs: PortableProofInputBundle;
  origin_claims: set<OriginClaim, OriginClaimKey>;
  compiler_build_claims: set<CompilerBuildIdentity, BuildIdentityKey>;
  submitted_diagnostic_report: option<SubmittedDiagnosticReport>;
}
OriginSubject = variant {
  schema_asset: SchemaRef;
  ir_asset: TypedContentRef;
  contract_asset: TypedContentRef;
  context_members: ContextMemberSubject;
  relation_slot: RelationSubject;
  normative_source: NormativeSourceSubject;
  compiler_registry: CompilerRegistrySubject;
}
ContextPart = enum {
  compiled_modules; type_schemas; effect_catalogs; resource_schemas;
  resource_descriptors; confidentiality_labels; taints; effect_sinks;
  usage_meters; recovery_effect_catalogs; reconciliation_operations;
  execution_capabilities; trust_domains; regions; region_compatibility;
  execution_runtime_profiles;
}
ContextMemberSubject = record { context_id: ContextId; part: ContextPart; }
RelationKind = enum { effect_resource; recovery_usage; flow_execution; }
RelationSubject = record { context_id: ContextId; kind: RelationKind; }
NormativeSourceSubject = record {
  compiler_id: CompilerManifestId; specification_id: SpecId;
}
CompilerRegistryKind = enum {
  specifications; rules; hash_domains; abi_numbers; rule_areas; afidl_primitives;
}
CompilerRegistrySubject = record {
  compiler_id: CompilerManifestId; kind: CompilerRegistryKind;
}
OriginKind = enum { catalog_provider; publisher; repository; builder; verifier; }
OriginIdentifier = record { kind: OriginKind; identity: Text; }
EvidenceClaimRef = record {
  predicate_type: Text; payload_checksum: PlainChecksum;
}
OriginClaim = record {
  subject: OriginSubject;
  claimed_content_checksum: PlainChecksum;
  claimed_origin: OriginIdentifier;
  evidence_claim: option<EvidenceClaimRef>;
}
~~~

OriginClaimKey is (subject variant rank in declaration order, expanded exact
subject key, OriginKind rank, identity UTF-8), then full ACE1 bytes. Referenced
IDs must resolve within proof_inputs. For a context part/relation, subject
content is the canonical expanded typed value, not just its local ID.
claimed_content_checksum is recomputed plain SHA-256 over that subject's
canonical bytes; for opaque normative/registry sources it is over the exact
raw source bytes. This names content, not an authorized origin.

An evidence checksum alone does not supply trustworthy evidence. EvidenceClaimRef
is an interface for a later typed predicate model; no network lookup, signature
verification or trust-root construction occurs here.

## Compiler/build identity interfaces

~~~text
RawArtifactIdentity = record {
  media_type: Text; byte_length: U64; content_sha256: PlainChecksum;
}
CompilerBinaryArtifactIdentity = RawArtifactIdentity
SourceRevisionIdentity = record {
  origin: OriginIdentifier; revision: Text; source_tree: RawArtifactIdentity;
}
CompilerBuildIdentity = record {
  source_revision: SourceRevisionIdentity;
  build_recipe: RawArtifactIdentity;
  implementation_dependencies: set<RawArtifactIdentity, RawArtifactKey>;
  compiler_binary: CompilerBinaryArtifactIdentity;
}
BuildProvenanceEvidence = record {
  subject: CompilerBuildIdentity; predicate_type: Text;
  payload: Bytes; payload_checksum: PlainChecksum;
}
QualifiedVerifierIdentity = record {
  build: CompilerBuildIdentity;
  declared_semantics: CompilerSemanticsManifest;
  qualification_evidence_requirements: set<EvidenceClaimRef, EvidenceRefKey>;
}
~~~

RawArtifactKey is (media_type, byte_length, checksum.algorithm, checksum.hex).
BuildIdentityKey is ACE1 of the complete build identity. EvidenceRefKey is
(predicate_type, checksum algorithm/hex). Media type/revision/issuer strings
are nonempty NFC declared identifiers, never executable paths or loader commands.
These are **claim identities**, including the name QualifiedVerifierIdentity;
they do not certify that its build is qualified.

Binary byte identity is a plain artifact checksum. It is neither
CompilerSemanticsIdentity nor a new HashV1 domain. A later signed artifact
adapter must explicitly map its transport checksum/nominal content identity;
relabeling a SHA-256 blob hash as an artifact HashV1 reference is invalid.

Same semantic source contents with different binaries requires fresh checking
and separate qualification premises. Same binary checksum does not establish
publisher trust or implementation correctness. No Bundle-selected binary is
loaded/executed by reconstruction.

## Submitted versus fresh reports

~~~text
SubmittedDiagnosticReport = record {
  claimed_root_ir: IrPin; claimed_export_contract: ContractPin;
  format: enum { diagnostic_text_v1; };
  text: Text;
}
~~~

This report is opaque diagnostic text; a statement “overall=Proven” in it
cannot enter the assertion or gate. It may support explanation/audit/performance
comparison. Current typed fresh ProofReport comes only from D10 and is linked
to the full reconstructed assertion tuple. Source spans, reason strings,
formatting and timing remain presentation.

Submission metadata is outside the static Bundle evidence identity. A future
parent may commit claims/evidence separately under its own complete canonical
model; B06 is not implemented. Changing an origin claim or old report does not
change the semantic projection or skip fresh checking.

## Independent states and owners

| State | What can establish it in this model |
| --- | --- |
| ContentComplete | D2/D5 verify complete declared input fields/member scopes; no implied external catalog authority |
| ContentIdentityValid | D3 exact canonical hashes/checksums and regenerated projections |
| OriginClaimed | Caller provides a typed raw OriginClaim |
| OriginAuthenticated | Requires separate B05 trusted authentication evidence |
| OriginAuthorizedForRelease | Requires separate B07 authoritative release policy |

These are distinct facts, not one verified bool. Core may reconstruct an exact
untrusted context and even yield a local static Proven relation under it.
Origin/body/implementation/publisher/runtime obligations remain
RequiresEvidence or NotEstablished.

B05/B07 must later define trusted issuer identities, claim scope, roots,
predicate semantics, timestamps/revocation and authorization. Raw model fields
cannot supply OriginApproved=true. Those missing trust models do not prevent
B01/B02's static input design closure and do not close publication readiness.
