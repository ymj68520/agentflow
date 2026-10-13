# Phase 4C.1A — Closed Decoder and Rejection Stages

All stages/types/errors are **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED ABI /
NOT IMPLEMENTED**. Errors are structured candidate families, not new existing
Rust error variants.

## Bootstrap versus proof scopes

The qualified Decoder owns a supported finite outer schema selected by
schema_version. This bootstrap schema set is distinct from the Bundle's exact
ProofContext compiled_modules. It must not be inserted into that proof context
automatically: doing so changes the current manifest/provenance input.

No unknown mandatory field or variant is ignored for forward compatibility.
Presentation metadata and diagnostics belong to the separate submission
envelope; unknown semantic Bundle fields are still rejected.

## Stage contract

| Stage | Input → output | Boundary / checks | Error families |
| --- | --- | --- | --- |
| D0 Decode | raw UTF-8 bytes → raw schema-directed typed values | Closed version/record/enum/variant/options; duplicate key detection before host-map insertion; exact scalar widths | InvalidUtf8, UnknownSchemaVersion, UnknownField, MissingField, DuplicateField, UnknownVariant, InvalidOption, NumericRange, InvalidPrimitive |
| D1 Canonical | raw typed value → canonical raw Bundle | Exact escapes/NFC/number/bytes forms, sorted keys/member sets and preserved sequences; re-encode byte equality | NoncanonicalEncoding, NoncanonicalOrder, DuplicateSetMember, InvalidUnicode, InvalidBytes, PrecisionLoss |
| D2 Reference | canonical Bundle → reference-closed raw model | Typed domains, unique local IDs, asset refs, schema scope keys, source identities, finite structural/assertion graphs | WrongDigestDomain, InvalidNominalReference, MissingReferencedContent, DuplicateIdentity, ConflictingAssociation, ReferenceCycle, UnreferencedAsset |
| D3 Content | referenced assets → integrity-checked raw contents | HashV1 IR/contract/schema; plain checksums and exact source/registry consistency | ContentIdentityMismatch, RegistryContentMismatch, SourceContentMismatch |
| D4 Support | integrity-checked inputs + local support snapshot → supported static operation | Reject unsupported attribution selector, then exact semantic tuple, required defined/implemented roles and actual schema/operation path | UnknownSemantics, UnsupportedCurrentSemantics, UnsupportedHistoricalSemantics, ConflictingSemanticContent, MissingSemanticRole, UnsupportedAttributionProfile |
| D5 Reconstruct | complete supported scopes → local immutable contexts/universes | Pure recipes, schema number/ref validation, registered machine-shape consistency, no defaults | IncompleteContext, InvalidSchemaGraph, ContextProjectionMismatch, UnsupportedProjection |
| D6 Contracts | raw canonical contracts + exact local context → new CheckedComponentContracts | Existing formal validator; recomputed checked canonical bytes/digest must equal supplied asset; no saved wrapper | ContractViolation, ContractUnproven, ContractRequiresEvidence, SemanticCanonicalMismatch |
| D7 IR | raw caller/target IR + exact schema scope → new CheckedIrDocument + Phase 3C | Existing structure/type/dataflow validators; normalized canonical bytes/digest must equal supplied asset | IrValidationFailure, DataflowFailure, SemanticCanonicalMismatch |
| D8 Resolver | checked assets + exact manifest → new immutable snapshot | Derive source/target/direct/transitive facts; compare associations and optional states | IncompleteResolverInputs, SourceAssociationMismatch, DependencyAssociationMismatch, UnsupportedDependencyCycle, MissingDependencyTarget |
| D9 Root | fresh checked IR/snapshot/context → Root + RootRecovery | Existing joint derivation, closure and provenance | RootDerivationFailure, RootClosureUnproven |
| D10 Check | fresh root + reconstructed raw assertion → fresh RootExport report and optional local checked wrapper | Existing formal static preflight and every dimension; no submitted verdict | RootExportViolation, RootExportUnproven, RootExportRequiresEvidence |

All stage outputs before D6 are **raw validated data**, not a proof of contract
compatibility or origin approval. D6/D7/D9/D10 trusted objects have private
construction through existing formal production functions.

D8 ordering is fixed: require available target/edge facts; reject a declared
target-graph cycle; derive actual target edges and compare all known lists;
validate derived closure/cycles; compare source associations and build the
snapshot. Thus a caller-declared cyclic negative input can be rejected before
an edge mismatch. This does not claim cyclic hash-pinned IR assets have been
materialized as legal production inputs.

## Specific failure rules

- Missing a required Option field fails D0; explicit None survives decode and
  is interpreted at the appropriate stage.
- Unknown catalog content cannot become Complete([]). Unavailable known case
  decodes but positive reconstruction stops at D5.
- None dependency facts decodes but stops D8 as Unproven. Some([]) is compared
  with actual no-edge target IR.
- Duplicate map keys, including NFC-equivalent keys, fail before a JSON object
  parser can overwrite them. Duplicate aliases are rejected even though the
  current in-process canonicalizer deduplicates its input vector.
- Same target with conflicting export rows fails D2; a changed valid association
  is new input, freshly derived and checked at D8–D10.
- An ordinal represented outside uint64 fails D0. A valid uint64 outside the
  actual source vector fails semantic reference validation; native-size
  inability is ResourceLimit/UnsupportedHostWidth, never truncation.
- Wrong ResourceTypeRef full pin fails D2/D5 nominal resolution or D6 contract
  checking; an absent compatibility relation remains Unproven.
- Hash/checksum-valid content from an untrusted origin can complete D0–D10;
  authentication remains RequiresEvidence in a separate outcome.
- A submitted Proven report is not a D10 input. A forged checked/trusted flag
  inside Bundle is an unknown field and fails D0.

Decode errors do not masquerade as computed RootExport verdicts. A failure
report records the earliest failing stage/family and typed subject/path.
Fresh checker results retain all dimensions/obligations; display reason strings
are diagnostics only.

## Resource and execution restrictions

The local Decoder policy bounds total bytes, table rows, scalar lengths,
nesting, graph expansion and work. Those limits are explicit infrastructure
inputs, not semantic authorization. Exceeding one yields ResourceLimit and
no complete result/capability; it does not produce an empty set or zero bound.

Graph cycle checking and nested assertion expansion must occur before recursive
allocation/stack use. Implementations may use iterative DFS/work lists. No
ambient filesystem, registry, network, clock or environment value supplies a
missing proof asset or changes its meaning.

## Independent implementation obligations

Both implementations consume the same candidate named shapes and existing
frozen nominal schemas, use exact UTF-8/numeric ordering, retain tagged presence,
and compare typed facts. They may present different diagnostic strings.
Normalization, duplicate rejection and re-encoding must precede calls to
constructors that normalize/deduplicate/overwrite.

The future implementation must reject direct deserialization requests for
CheckedComponentContract, CheckedIrDocument, DerivedWorkflowContractSummary,
RootRecoverySummary or CheckedRootExportCompatibility. This phase adds no
Decoder or alternate constructor.
