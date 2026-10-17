# G2 — exact source, validation and propagation binding

## A: complete native source, not a construction pin

Owner: agentflow.computation.proof-report.model@6. Native source owner:
af-contract. NativeRelationSnapshotSource is a closed record:

| Field | Actual native source / nominal meaning |
| --- | --- |
| kind | actual named slot and concrete RelationCatalogSnapshot<T> |
| schema_family | snapshot.schema(), strict SpecificationId family |
| revision | snapshot.revision(), actual opaque nonempty NFC token |
| pin.family | snapshot.pin().schema String; must agree with schema_family and T::SCHEMA_ID |
| pin.revision | actual pin revision, agrees with stored revision |
| pin.canonical_content_checksum | actual algorithm/hex; plain SHA256 of full bytes, not HashV1 |
| canonical_content | complete snapshot.canonical_bytes(), never length/hash-only |
| catalog | complete snapshot.catalog().to_value() under the explicit closed per-kind native wire projection |

Native Family grammar is af-idl SpecificationId: lowercase ASCII kebab namespace
segments, exact @positive-u32 major without leading zeros, no normalization,
Unicode/case-folding/whitespace/latest alias. Source admission independently
checks syntax, duplicate stored values, revision NFC/nonempty and plain checksum.
The generic native constructor remains unchanged; its immutable invariants do
not authorize treating a family String as a full historical TypeSchemaRef.

The catalog bridge is explicitly registered, not inferred by identical names:
each concrete native RelationCatalogWire implementation preserves all declared
fields/variants into its exact fixed wire model. RelationWire/* descriptors
encode the full frozen AFIDL/native wire grammar, including imports and typed
primitive distinctions. They describe this projection, not the schema used in
an old constructor call and not a fallback current schema. Round-trip decoding
back into actual T and full equality reject any lossy mapping.

## Three exact associations

| Slot | Actual native T | Family / exact lookup root |
| --- | --- | --- |
| effect_resource | EffectResourceCompatibilityCatalog | agentflow.effect.resource.compatibility@1 / EffectResourceCompatibilityCatalog |
| recovery_usage | RecoveryUsageCoverageCatalog | agentflow.recovery.usage.coverage@1 / RecoveryUsageCoverageCatalog |
| flow_execution | InformationFlowExecutionCatalog | agentflow.flow.execution.relation@1 / InformationFlowExecutionCatalog |

The machine slot-matrix.json binds each to source rule0286, lookup0288 and
validation0292. Expected TypeSchemaRef is the exact af-semantic::TypeSchemaRef
descriptor within the new model owner; no unrelated nominal header is accepted.

## C: actual current modules and actual validation

inspect_relation_metadata borrows the real analysis Context and checks actual
pointer equality with canonical_validation, never content/hash equality. It
checks the exact current CompilerSemanticsIdentity before querying. The receipt
retains that actual Context borrow; its finite metadata view contains complete
source, current module contents, lookup/definition closure and validation result.
Its assurance is relation_slot_only, not a cross-process full Context/Scope.

Lookup is only actual_context.modules.type_by_identity(exact family/root).
Found contains the actual complete canonical root, complete named-reference
definition closure, exact slot target and the current three-field TypeSchemaRef.
specification_id and type_name come from that actual root. schema_digest comes
from its canonical schema ACE1 and HashV1(sha256,type-schema), never a catalog
checksum/family-string digest/default/diagnostic. Complete module contents remain
present because a root digest names imports rather than inlining their bodies.

Validation executes native catalog canonicalization/equality, current encode,
strict current decode, actual T decode/equality and exact byte round-trip. Only
completion of every step emits Valid. Invalid preserves Found, failed typed
step, exact native AceError variant/payload where available, completed preceding
steps and closed input-site selectors. These refer to the retained source
catalog, current schema or canonical bytes. No reason string is parsed. An
Invalid catalog may retain a truthful pin to a valid actual schema definition;
it does not claim that the catalog validation succeeded.

Valid is pointwise validity of this exact value, not universal native-T/schema
ABI equivalence. Equal empty native snapshots under A/B can validate under their
different current schemas and retain different schema pins. Consumers requiring
an exact supported schema still compare that exact pin and closure themselves.

## Closed status applicability

| State | Actual applicability and payload |
| --- | --- |
| NotReached | pre-stage identity/context/source refusal, or actual absent option; no fabricated later check |
| Missing | exact module/root/reference absent in supplied modules; target and missing identity retained |
| Ambiguous | malformed pre-native candidate collection contains multiple actual definitions; all candidates retained |
| FamilyMismatch | an actually supplied comparison family conflicts with the registered slot/source; no fallback |
| RevisionMismatch | actual supplied expected catalog pin has a different revision; full actual/expected pins retained |
| Found | complete current root/reference closure resolved, intermediate only |
| Invalid | an actually reached validation step failed; closed typed evidence and source retained |
| Valid | all applicable checks executed successfully; exact current pin/catalog/bytes and complete step roster retained |

A valid CompiledModuleSet has private unique-key storage, validated import
closure and no cycles. Native ambiguity/missing imported definitions therefore
cannot be manufactured through that API. Actual duplicate pre-context inputs
are independently compiled and rejected; the separate candidate-boundary probe
records the genuine A/B definitions without pretending a native Context existed.
Likewise a foreign family module with the same local root name yields Missing
for the exact requested family, not an arbitrary first match or FamilyMismatch
claim about a definition that was never selected. Shape cases exercise closed
payloads; only real applicable native/boundary calls count as dynamic evidence.

Revision is not schema edition. No expected pin means no invented revision
premise. Same revision with a different content checksum is source-comparison
refusal, not mislabeled RevisionMismatch. Different pins do not become port V.

## Three-slot conjunction, Subject and output separation

inspect_relation_context_metadata runs all three named slots under the same
actual Context object. Every actual Some must be Valid; None remains absent
and creates no catalog or positive relation evidence. A valid effect slot cannot
hide missing recovery or invalid flow. The opaque aggregate retains all three
results, computes conjunction and always reports remaining_scope_incomplete.
Claimed completion/missing-observation mutations reject against the actual receipt.

Nine original Context components and their meanings are unchanged. Native
analysis/canonical_validation must alias even when contents are equal. Neither
single-slot nor aggregate metadata is a full nine-member Context projection,
complete compiler Scope, Fact or G02/T5 cross-process association proof.

Under model@6, RelationSnapshotContent refers to the new A source. Exact parent
references propagate through ContractProofContextContents, RawValidationSubject,
Source.raw_validation, Origin/FactKey, RawPortCheckBasis, rejection witness,
TypedContradiction.raw_validation and EvidenceRef. C is a separate output
CurrentContextRelationSchemaValidation/admission witness, not an input field
creating a source→validation→source cycle. No Raw logical schema premise is
invented before a reached port-zero check. Target remains the actual compiled
ComponentContract schema/rule/port constraint; source/schema-query target and
named slot coordinates are not Raw original/sorted ordinals. RuleRef is not
EvidenceRef; primary still means typed minimum and duplicate witnesses retain
both actual original members.

Existing Root ProofContextManifest pin+byte summaries alone do not contain the
full native T. A future full Source projection must bind their actual same
Context.relations snapshots, not invent T/history from the summary. Current G2
does not implement that full Root/Raw adapter. Formal successor descriptors can
require complete source while current full Producer/Scope stays unsupported.

The model@5 catalogue/probe and all partial Raw/native traces remain historical
metadata only. They are not relabeled model@6 or certified as complete successor
facts. Real port PositiveFresh/Zero/Duplicate verdicts and ordering are unchanged;
Source/Subject admission failure is not a new port contradiction.
