# agentflow.component.document-occurrence.semantics@1

Registered non-wire metadata staging, NOT Raw Producer, Scope encoder, Normalizer,
external authentication or positive capability. C2G1 DO-01..DO-14 are concretized
by the exact model@5 table and the rules below; native/substitution/Root meanings
and the one positive_unique_io_ports predicate remain frozen.

## Complete source and four identities

Content is the full nominal RawDocument: exact original physical_json_utf8 bytes
or the complete declared_native_document sequence of thirteen-field Raw candidates.
No digest, length, NFC presentation or reserialization replaces that content.
OwnerAnchor={domain:{namespace,domain_key},owner_key,revision_key}. All four keys
are exact nonempty UTF8 bytes,1..4096 bytes,without normalization or case folding.
Identity compares full typed values; key byte order is unsigned lexicographic,
entry ordinals are mathematical U64. Occurrence=(complete anchor,actual owner
container document-entry ordinal). Member identity additionally retains the true
internal document path and actual port coordinates. Observation is output exposure,
not a source. Equal content may have distinct occurrences; repeated observation
of one complete scoped fact does not create a new occurrence.

## Finite closed source package

The table defines SourcePackage{domain,owners:Seq<DocumentOwnerRecord>}, with each
record {anchor,container}. Container is exactly one of:

- declaration_bundle{bundle_name,declarations:Seq<{declaration_name,document}>}
- import_manifest{manifest_name,imports:Seq<{import_name,document}>}
- workflow_source_collection{collection_name,sources:Seq<{source_name,document}>}

Each document is a typed Node<RawDocument>; the complete actual package, full owner
container and document are input values, not an opaque JSON blob or report hash.
The occurrence binding retains source_package,source_owner,occurrence_key{entry_ordinal},
actual_document. Owner ordinal is NOT a document internal declaration entry, port
original/sorted ordinal, source_ordinal/mapping_ordinal or report position.

The caller must actually supply this declaration before checking. No adapter may
invent a default domain,per-call one-document package,UUID,pointer,clock,ProbeId or
allocation count. A freshly explicitly submitted complete package can establish
new declared provenance,never recover an unknown past invocation's provenance.
An arbitrary larger third-party manifest cannot be truncated into this schema
and claimed complete; unknown fields,schemas,domains or owner variants reject.
Durable external handles and authenticated-source variants are NOT supported.
Namespace is a declared scope,not a whitelist of trusted organization names.

## Admission and conflicts

Check exact current compiler/content and closed shapes. Missing package,anchor,
revision or entry key is MissingDocumentOccurrenceBinding (unformable required
Portable fields can fail earlier IncompleteFacts). Validate the entire submitted
package: every owner domain equals package domain,anchors are unique,owners<=4096,
total actual document entries<=262144. Duplicate owner rows are MalformedSourcePackage.
Key or work/depth/input bounds may fail ResourceLimit; no unbounded search.

Select the unique actual owner and proper container sequence; out-of-range U64 or
absent owner is InvalidOccurrenceAssociation. The selected complete document must
equal binding.actual_document. Same anchor/ordinal with different content is
BindingConflict; same anchor with different full owner records in explicitly joint
packages is OwnerBindingConflict in that conflict family. Joint checking examines
only supplied packages and never claims a global uniqueness ledger. Same exact
owner/key/content reuses one declared occurrence across reloads. Different real
owner keys or actual slots can distinguish equal documents.

Local metadata admission requires the actual_document borrow to be the document
selected from the actual submitted package; a different same-valued document
cannot be passed under another owner's selection using Eq alone. Native addresses
are local association checks only and are never serialized or identity inputs.
Portable revalidation would reselect from complete inputs,not authenticate an old
process's pointer or historical origin. No caller supported/trusted bool suffices.

## Physical/native and context

Physical uses supplied association to the exact same occurrence,retains original
bytes and selects the true parsed member with duplicate-key-free strict UTF8 JSON
metadata decoding. Unknown fields,invalid widths or wrong selected candidate reject.
Native uses complete typed Raw metadata declarations and physical=not_supplied.
Native+supplied and physical+not_supplied fail UnsupportedPhysicalAssociation.
NotSupplied means absent physical history,not empty bytes. No conversion receipt
or new general Contract wire decoder is introduced.

The nine-member context and actual shared analysis/canonical_validation object
partition remain unchanged. Equal contents do not imply object identity. No
resolver_dependencies or Context role/digest substitutes for source Owner.
Port validation remains input-before-output,stable mathematical U32 sort,original
ordinal preservation and real predecessor/prefix. Fresh positive=P,zero=V,duplicate=V;
admission failures are not Predicate Evaluated Unproven/Violation or whole schema
judgments. canonical_ports and invalid_contract_report are not modified.

## Explicit inline metadata companions

The closed MetadataDeclaration/Import/WorkflowDocumentEntry,MetadataDocumentOwnerContainer,
MetadataDocumentOwnerRecord,SourcePackageMetadata and DocumentOccurrenceMetadataBinding
are exact finite expansions replacing ONLY the above Node<RawDocument>/Node<SourcePackage>
references with full inline target values. Their field/variant semantics are otherwise
identical. They provide a complete non-wire admission view,not new DAG rank meanings,
runtime reflection,Scope construction,normalization or proof serialization.
Success returns unit: admitted declared association only. No trusted predicate,
canonical contract,Checked,FactKey,TypedFactSeed or external authority is constructed.

DeclaredOnly is the only assurance. Names and distinct keys do not prove globally
unique real-world ownership,publisher authentication,Scope authority,budget or runtime
authorization. Unknown domains mean unknown structural source schemas,not an unknown
namespace string. Incomplete/unsupported/resource failures never fall back to content-only
success or an allegedly complete empty fact collection.

<a id="document-identities"></a>

## AF-SEMANTIC-0272 — DO-01

MUST: Complete document content, declared occurrence, internal member and output observation are distinct identities; equal content does not imply equal occurrence.

Direct normative dependencies: AF-SEMANTIC-0200, AF-SEMANTIC-0210.
Registration conformance: g01-document-occurrence-staging/rule-0272; NOT Producer Capture.

<a id="actual-source"></a>

## AF-SEMANTIC-0273 — DO-02

MUST: Occurrence admission requires an actual supplied complete SourcePackage and real owner revision and document slot; missing provenance cannot be synthesized from a candidate or invocation.

Direct normative dependencies: AF-SEMANTIC-0261.
Registration conformance: g01-document-occurrence-staging/rule-0273; NOT Producer Capture.

<a id="declared-authority"></a>

## AF-SEMANTIC-0275 — DO-04

MUST: Exact UTF8 namespace, domain, owner and revision keys establish only declared source identity and never external authentication or global ownership.

Direct normative dependencies: AF-SEMANTIC-0213.
Registration conformance: g01-document-occurrence-staging/rule-0275; NOT Producer Capture.

<a id="owner-conflicts"></a>

## AF-SEMANTIC-0276 — DO-05

MUST: All submitted owners must match the package domain and have unique complete anchors; conflicting complete records in explicit joint packages are owner binding conflicts.

Direct normative dependencies: AF-SEMANTIC-0200.
Registration conformance: g01-document-occurrence-staging/rule-0276; NOT Producer Capture.

<a id="occurrence-conflicts"></a>

## AF-SEMANTIC-0277 — DO-06

MUST: Owner anchor and actual U64 document-entry ordinal define occurrence; equal content reuses it and unequal content at the same key fails BindingConflict.

Direct normative dependencies: AF-SEMANTIC-0210.
Registration conformance: g01-document-occurrence-staging/rule-0277; NOT Producer Capture.

<a id="member-paths"></a>

## AF-SEMANTIC-0278 — DO-07

MUST: External document slot, internal document path, port original and sorted ordinals and observation site remain separate and all selected input members must agree.

Direct normative dependencies: AF-SEMANTIC-0265.
Registration conformance: g01-document-occurrence-staging/rule-0278; NOT Producer Capture.

<a id="physical-native"></a>

## AF-SEMANTIC-0279 — DO-08

MUST: Physical sources require original bytes and supplied matching occurrence; native sources require complete declarations and not_supplied physical history; other combinations reject UnsupportedPhysicalAssociation.

Direct normative dependencies: AF-SEMANTIC-0263.
Registration conformance: g01-document-occurrence-staging/rule-0279; NOT Producer Capture.

<a id="frozen-port-context"></a>

## AF-SEMANTIC-0283 — DO-12

MUST: The one Raw port predicate, actual execution prefix and nine-member same-object context partition remain unchanged; no context role, resolver or generic schema profile supplies document identity.

Direct normative dependencies: AF-SEMANTIC-0262, AF-SEMANTIC-0264.
Registration conformance: g01-document-occurrence-staging/rule-0283; NOT Producer Capture.
