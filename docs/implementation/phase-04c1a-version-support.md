# Phase 4C.1A — Exact Version Support and Invalidation

All candidate policies/types are **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED
ABI / NOT IMPLEMENTED**. Support is selected from a local verifier-controlled
snapshot, never from a Bundle's claimed supported=true flag.

## Closed support types

~~~text
OperationKind = enum { static_root_export_revalidation; }
ImplementationSupportStatus = enum {
  implemented; defined_but_not_implemented; model_missing; unsupported;
}
SemanticSupportClass = enum {
  current_supported; current_unsupported; historical_supported; historical_unsupported;
  unknown_semantics; conflicting_semantic_content;
}
SupportedProfileEntry = record {
  schema_version: SchemaVersion;
  operation: OperationKind;
  expected_compiler_content: CompilerSemanticsManifest;
  required_roles: set<SemanticRole, RoleRank>;
  implementation_statuses: set<RoleImplementation, RoleRank>;
}
RoleImplementation = record {
  role: SemanticRole; status: ImplementationSupportStatus;
}
VerifierSupportSnapshot = record {
  current: list<SupportedProfileEntry>;
  historical: list<SupportedProfileEntry>;
}
~~~

This snapshot is **local configuration/output**, outside portable proof inputs.
Its identities and actual support must be established by the selected verifier
implementation and separately qualified trust process. It is not a certificate.
An entry does not load a binary or execute producer-supplied code.

Lists are unique by the full schema/operation/expanded semantic-content tuple,
canonicalized by ACE1(entry identity) byte order. The same tuple cannot be both
current and historical. Each entry recognizes an exact tuple and records all
ten local role statuses, including absent implementations. Its name is not a
positive support flag: only an entry whose required roles are all implemented
and whose schema/operation path actually exists is eligible for success.
No missing status defaults to support. Recognized historical entries with an
absent path remain available for HistoricalUnsupported classification.

## Selection algorithm

1. Verify the declared source/registry/table checksums and all internal
   content/role/attribution references. Reject a known attribution selector
   unsupported by this operation as UnsupportedAttributionProfile before
   tuple matching (initial static operation supports Profile A only).
2. Expand compiler identity refs and compare the complete tuple with current
   exact entries. A match with the required implemented role set and an actual
   schema/operation path is CurrentSupported. A recognized current tuple with
   an absent role/path is CurrentUnsupported, retaining the
   ModelMissing/DefinedButNotImplemented/Unsupported reasons; it never falls
   through to a more convenient historical entry.
3. Otherwise compare with explicit historical entries. A match with an
   implemented verifier path is HistoricalSupported; an exact historical
   match without the required path/statuses is HistoricalUnsupported.
4. If no full tuple matches, compare the exact
   static_identity_projection.analysis_semantics_version with every recognized
   current/historical entry. A known label with any changed content/role/
   attribution tuple is ConflictingSemanticContent. Multiple explicit tuples
   may intentionally share a label; exact matching precedes this check.
5. If neither exact tuple nor label is recognized, return UnknownSemantics.
6. Every non-supported outcome stops before minting a checked static capability.
   A missing mandatory role may additionally report ModelMissing or
   DefinedButNotImplemented; all reasons are retained.

Matching labels, majors or rule counts is insufficient. Version strings have
no implicit SemVer ordering/substitution relation. An optional future approved
historical binary is selected **outside** the Bundle and revalidates exact old
inputs; it does not return a restored historical Rust pointer.

## Initial production support boundary

Current repository RootExport checks only its build's current exact compiler
identity. It has no multi-version dispatcher or portable Decoder.
The future initial portable implementation may advertise only the exact
post-registration profile it actually implements. Historical support remains
empty unless an explicit conforming path is implemented and qualified.

The portable v1 imported machine-shape families are fixed. A historical
semantic tuple also needs a supported representation/shape path; a semantic
label cannot make an unknown nested variant or schema decode. Different wire
shapes require a separately versioned Decoder/explicit compatibility model.

The pre-registration HEAD profile and post-registration profile are different
tuples because whole registries are bound. An old report remains historical
data. Revalidation under a new intended semantic profile produces new inputs,
derivation and result; changing stored labels is never proof reuse.

## Upcasting and diagnostics

No upcaster is part of v1. Candidate future interface, still blocked:

~~~text
UpcasterDefinition = record {
  source_schema: SchemaRef; target_schema: SchemaRef;
  conversion_semantics_version: Text;
  compatibility_evidence_requirement: Text;
}
~~~

It would need separately specified pure deterministic conversion, exact source/
target schemas, semantic compatibility proof and closed failure cases.
Network, clock, environment and latest lookup are forbidden conversion inputs.
A future upcast cannot discharge static compatibility without fresh checking.

Diagnostic strings, spans, source paths used only as display labels and
performance counters may differ across conforming verifiers. Formal compared
facts are the typed semantic projection, static prerequisites/dimensions,
verdicts and structured failure families. Evidence identity binds exact
source/registry content and witnesses; presentation equality is separate.

## Hard invalidation table

| Change | Required action |
| --- | --- |
| IR/export bytes or pin | Recheck canonical content and all formal inputs |
| Context members/relations | Reconstruct exact new context; recheck contracts and derive anew |
| Resolver source/target/export/edge | Rebuild exact snapshot and Root/RootRecovery |
| Typed coordinate witness | Fresh complete assertion validation/check |
| Source/registry/table/version | New supported semantic tuple or unsupported failure |
| Compiler binary/build | Fresh checking by the selected qualified build; no inferred equivalent behavior |
| Diagnostic-only text | No change to static input identity/result |
| Origin authentication/policy | Separate evidence/release outcome; does not rewrite static content facts |

F04 remains OPEN/P2. Unknown path implication stays Unproven for every supported
version of the current relation; changing diagnostic suppression into positive
guard inference needs an independently authorized semantic version and audit.
