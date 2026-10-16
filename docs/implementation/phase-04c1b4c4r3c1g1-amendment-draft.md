# C1G1 — Proposed normative amendment (UNREGISTERED)

Design recommendation C. All names below are proposed logical authoring names,
not registered SpecificationIds, RuleIds, Edition numbers, ranks, or wire types.
No source in spec/, Registry, AFIDL, ABI, HashV1, Compiler, or Checker is modified.

## RC-SCOPE — smallest exact subject

Propose an independent non-root `raw_contract_validation` fragment. Initially it
has **one** substantive predicate case `positive_unique_io_ports`, parameterized
by the actual port-check occurrence. This is not a renamed generic schema-validity
report carrier. Admission invariants are not fabricated validator Proven facts.
No aggregate canonical-valid result or all-canonical-errors profile is implied.

Closed logical definitions (all references are complete typed contents or future
input-DAG nodes; result/witness nodes are in a separate output DAG):

```text
RawDocument = physical_json_utf8 {bytes:Bytes}
            | declared_native_document {declarations:Seq<RawCandidateContract>}
RawDocumentMember = field {name:RawUtf8Bytes} | entry {ordinal:U64}
RawInputOrigin = {document:Node<RawDocument>, path:Seq<RawDocumentMember>}
RawPhysicalAssociation = not_supplied:unit
                      | supplied {document:Node<RawDocument>,
                                  value_path:Seq<RawDocumentMember>}
RawParsedInput = {candidate:RawCandidateContract,
                 origin:RawInputOrigin,
                 physical:RawPhysicalAssociation}
RawContextRole = analysis | canonical_validation
RawContextBinding = {contents:ContractProofContextContents,
                     roles:Seq<NonemptySeq<RawContextRole>>}
RawValidationSubject = {owner_compiler:CompleteCompiler,
                       input:RawParsedInput,
                       context:RawContextBinding}
```

`RawUtf8Bytes` means exact bytes validated as UTF-8 without NFC normalization;
it is not `$NfcText`. Physical byte content itself can be malformed UTF-8.
`RawCandidateContract` is the existing complete Raw thirteen-field type, including
all raw text bytes/options/numerics/variants. Do not treat it as arbitrary untyped
JSON or a CanonicalComponentContract. RawDocument's `physical_json_utf8` tag names
the frozen transport interpretation; it does not change ACE1 grammar or AFIDL.
Unsupported formats are explicitly UnsupportedSourceFormat, not auto-detected.

A physical source requires complete original bytes and exact parsed field/array
path. Duplicate JSON keys, invalid UTF-8, missing required fields, unknown fields,
wrong numeric widths or malformed structure reject before RawParsedInput exists.
Parsing retains source spans/ordering/unknown material for its typed rejection
diagnostic; it does not normalize, silently drop members, or issue this profile's
port fact. This parser-rejection envelope is separate from the one-case fact
profile; further formal parser predicates require separate registration.

For a native typed input, physical=not_supplied is honest, not known-empty bytes.
Its Raw declaration is preserved exactly as raw field values. An **actual**
declarative native source document and entry path must still be supplied/borrowed
by the future adapter. It must not wrap each runtime call into a synthetic
one-entry document, use a ProbeId, or infer a document from a diagnostic.
If unavailable, return MissingDeclarativeOrigin and emit no portable originating
fact; an ephemeral legacy diagnostic is still possible. The present direct C1
call cannot be retroactively given a historical Origin by this design.

When a caller already owns a real one-declaration document, its root/entry path is
legitimate. Two entries with equal candidate contents in one actual document have
different paths. Separate declarations need their actual enclosing source/path;
absent provenance is not repaired with UUID/call index/timestamp/source line.
No authentication or authority is gained by declaring a document. The future
producer must validate the document->input association, not accept a free string.

The complete physical document participates in raw identity when supplied;
whitespace/NFC/escape-spelling changes in physical input do not become the same
physical source merely because parsing yields the same value. Diagnostic display
is an OUTPUT, not a substitute source or a Scope descendant.

## RC-CONTEXT — stage applicability

RawValidationSubject uses all nine members of ContractProofContextContents with
exact source contents and the three named optional relation snapshots. Its two
roles each occur once and form exactly one observed alias class:
`[[analysis, canonical_validation]]`. The context is the same actual object in
the original call; equal content from another object is not an alias certificate.
Native addresses never appear in serialized identity. Future G02 must encode the
full contents and verified partition, not invent identity from hashes.

This Subject has **no resolver member**. Resolver absence is a stage property,
not `resolver_dependencies=[]`. Do not change old Context, old contract_validation,
root_export/computation, checked-source or resolver association requirements.
If a later procedure actually consumes a resolver, it must use its own exact
resolver-bearing subject/profile and truthful actual association; this fragment
does not supply that input. An explicitly supplied KnownEmpty resolver is still
not the NotSupplied native call and cannot bypass another profile's checks.

## RC-STAGES — separate states, no successful encoding for rejection

| State | Required actual input/output | Legal statement |
|---|---|---|
| Raw physical input | Original bytes, before successful parse; may be absent for native calls | Exact physical content only; no parsed/canonical/Checked token |
| Parsed Declaration | Successful exact structural deserialization; raw text preserved | u32 zero can parse; schema width admission != positive-port proof |
| Canonical diagnostic presentation | Display projection with explicit provenance to source | May render NFC; never original bytes or canonical validation provenance |
| Canonical Validated Declaration | Actual successful whole canonicalize under the actual context | No such declaration on early failure; no invented ACE1 success or verified digest |
| Checked Contract | Actual full Proven contract report and private wrapper construction | Neither raw-port P nor canonical success alone issues Checked |

Raw parse rejection, canonical validation rejection, cross-dimension contract
rejection and Native Guard evaluation rejection remain separate output locations.
For the replay: Parsed exists; CanonicalValidation rejects at input port0; later
canonicalizers/Contract proof/Native Guard not executed. Legacy carrier U values
are not transformed into evaluated Unproven facts. This profile's applicability
ledger records `not_reached` with the actual earlier stage/cause, or `no_port`
for an empty namespace; neither is a fictional successful comparison.

NFD text can fit the Raw text representation. Whether it is allowed in a specific
validated field follows that field's formal schema and substantive checks. It
must not be silently normalized to repair invalid input. C1's NFD Guard literal
canonical presentation is not lossless historical Raw provenance; its capture
source was rejected. This amendment neither changes that guard verdict nor
retroactively establishes successful canonical encoding for an early rejection.

## RC-PREDICATE — one exact substantive judgment

```text
Family:       raw_contract_validation             [proposed]
Case:         positive_unique_io_ports            [proposed]
Kind:         prerequisite                       [reuse kind; new family mapping]
Requirement:  raw_validation:positive_unique_io_ports [proposed clause]
Role:         supporting_fact
Stage:        contract_validation                 [original stage]
Substage:     canonical_validation/io_port_check  [proposed closed parameter]
Dimension:    supporting(exact PredicateId), not all schema-validity
EvidenceClass: static_declaration
Rule:         AF-CONTRACT-0004 + exact registered RuleContentIdentity
```

The rule Source Anchor is `component-contract-ports`; compiler includes its exact
source and dependencies. The new governing rules are proposed RC-* clauses below,
with **no RuleIds reserved**. Reuse of the substantive rule does not register the
new family or assert full AF-CONTRACT-0004 type-pin conformance.

Applicability is one event for each actual invocation of the positivity/uniqueness
condition in canonical_ports. Input namespace is visited before output; within
each, the actual stable sort by mathematical U32 number is retained, with original
sequence ordinals. For each visited item: test zero/duplicate **before** schema
resolve. An earlier component identity failure yields no port events. An earlier
schema-resolution failure stops later port events. An unvisited item is not P/U/V.

The exact local condition is `number > 0 && number != previous_number`. The
initial sentinel previous_number=0 is not a real predecessor occurrence. For a
fresh positive entry, local status=P, no contradiction. For zero or duplicate,
status=V, typed witness mandatory. The operation always has Evaluation=evaluated
when actually called. No arbitrary U/RequiresEvidence/authority status is invented
for this closed mathematical condition. A missing typed execution/source/context
binding is a capture error, not a new Checker Verdict. No fact is issued on that
error. A successful local port fact says nothing about later type-pin/nested checks.

The carrier's `schema-validity Violation` for replay is an observation of this
exact port predicate V. Other error codes using the same carrier require separate
closed predicate mappings. Unknown/unmapped cases return UnsupportedPredicate;
they do not become this port predicate based on a Violation message.

## RC-LOCATION — Source / Target / Coordinate / Origin

```text
RawPortDirection = input | output
RawPortOccurrence = {direction:RawPortDirection, original_ordinal:U64,
                     sorted_ordinal:U64, port_no:U32}
Source.raw_validation = {subject:Node<RawValidationSubject>}
Target.raw_declaration_constraint = {
  subject:Node<RawValidationSubject>,
  schema:ExactCompiledComponentContractSchemaReference,
  rule:ExactRuleContentReference, direction:RawPortDirection,
  constraint:positive_unique_io_ports
}
Coordinate.raw_port = {occurrence:RawPortOccurrence, member:port_no}
RawValidationOrigin = {input_origin:RawInputOrigin,
                      direction:RawPortDirection, original_ordinal:U64}
Origin [successor record delta] = {
          predicate:exact proposed PredicateId,
          declared_path:typed subject.input.candidate.
                        (inputs|outputs)[original_ordinal].port_no path,
          source_ordinal:Some(original_ordinal), mapping_ordinal:None,
          raw_validation_origin:Some(RawValidationOrigin)
}
```

The two exact reference authoring aliases above are records, not opaque labels:
`ExactCompiledComponentContractSchemaReference = {module:SpecificationId,
type_name:RawUtf8Bytes, definition:af-idl::CanonicalTypeSchema}`; the module/name
and full definition must resolve identically in subject.context.contents.
compiled_modules. Imported field dependencies remain reachable in that complete
module set. `ExactRuleContentReference = {identity:af-computation::RuleContentIdentity}`
resolves to the complete rule and normative source content in owner_compiler.
`InputSubjectRef` means a typed input-DAG reference, never a native address or
output-DAG rank. The compiler is already in the Subject; no duplicate native
identity is synthesized.

The target schema reference resolves to the complete compiled
`agentflow.component.contract@1/ComponentContract` body in context.modules and
the exact input/output port field descriptor, not a raw contract digest. Its
semantic constraint comes from AF-CONTRACT-0004; this is not a claim that the
validator performed an ACE1 `shape_of`/encode query at the port branch.
Target rule reference resolves in complete compiler inputs; names/digests alone
are not full equality. Schema lookup for the reference is an adapter association,
not invented validator execution. Missing source schema binding fails capture.

For replay: direction=input, original_ordinal=0, sorted_ordinal=0, port_no=0,
member=port_no. Source is the complete rejected raw input/context, target is the
exact positive-port constraint, not `whole` or a component digest. Invalid zero
must use U32, not the nominal valid PortNo type. Duplicate references include both
actual original positions, even when both declarations and messages are equal.

RawDocumentMember can address source data before validated ModelType fields exist.
It is NOT inserted into the existing DeclaredStep union by an implicit cast.
The proposed optional Origin.raw_validation_origin field binds the full actual
document/path separately; it is Some exactly for this family and None for every
unaffected successor family. Origin.declared_path still contains real typed
DeclaredStep.field/entry members rooted at the new Subject, resolving the complete
Raw candidate's input/output port member. Both paths must resolve to exactly the
same original occurrence, never an arbitrary raw field name. Old Origin bytes
remain unchanged under old tuples. The new required optional-field descriptor,
path owner types and ranks belong to future registration, not a T4 encoder here.

Origin is the predicate application at a declared input member, not the Rust
function or error carrier. Same source/predicate/path/context association copies
have one original fact and union their legitimate observations; contradictory
copies fail InvalidFacts. Same text error at two source paths never merges them.
No runtime call ordinal, pointer, synthetic ComputationId/root or report index is
an originating identity. Full input DAG and role partition are retained separately
from output FactKey/witness DAG to avoid self-reference.

## RC-WITNESS — exact branch evidence

Proposed closed output types (no generic Error(String), no opaque Debug payload):

```text
RawPortPredecessor = none:unit | some {original_ordinal:U64, port_no:U32}
RawPortCheckBasis = {subject:InputSubjectRef, occurrence:RawPortOccurrence,
                    predecessor:RawPortPredecessor,
                    condition:zero | duplicate | positive_fresh,
                    selected_error:None | InvalidOrDuplicatePort,
                    rule:ExactRuleContentReference}
RawPortRejectionWitness =
  zero {subject:InputSubjectRef, occurrence:RawPortOccurrence,
        raw_member_value:U32, minimum:U32}
 | duplicate {subject:InputSubjectRef, occurrence:RawPortOccurrence,
              previous_original_ordinal:U64, previous_port_no:U32}
```

Zero requires raw_member_value=port_no=0 and minimum=1; no real predecessor for the
initial sentinel. Duplicate requires port_no>0, predecessor with equal number in
the same namespace, predecessor earlier in the actual stable sorted traversal,
distinct original ordinals, and exact corresponding declared values. Input/output
number equality alone is not a duplicate. Basis preserves all executed parameters
for P as well as V. V has exactly the matching zero/duplicate witness; P has None.
No assignments, GuardCountermodel, resolver result, Checked token, receipt, Root
recovery value or runtime evidence is permitted in this witness.

Legacy ContractError rule/code/reference/detail are retained as diagnostic
association output. Detail text is required when preserving the legacy report,
but is neither Fact identity nor the derivation of condition/path/witness.
The future producer must create the typed event at the actual branch, not parse
the string `invalid_or_duplicate_port` to decide zero vs duplicate. The Oracle
independently uses the raw declaration and stage execution specification.

Propose `TypedContradiction.raw_validation{witness:RawPortRejectionWitness}` and
`Fact.raw_validation_basis:Opt<RawPortCheckBasis>` (Some exactly in this family;
both old optional bases None). No result or witness becomes a Subject input.

## RC-LINKS — prerequisites, references and observations

Admission requires actual successful structural parse/native typed association,
complete compiler, origin document association, nine-context binding and actual
branch evidence. These are capture-admission requirements, not fabricated Proven
facts or Root core checks. Missing admission returns explicit failure and no fact.
If actual earlier validator facts are used, retain their own stage/predicate/
source/origin, supporting key and consumers. Do not invent historical P facts for
component identity/schema parse just because the port branch was reached.

For this local port condition, the actual predecessor basis is input, not a new
prerequisite fact. There is no schema resolution premise before the zero test.
The fact has no cause_fact_keys unless an actual typed supporting fact exists;
zero/duplicate witness is direct and nonrecursive. A later consumer may observe
this exact original fact as the registered positive_unique_ports premise only
with an explicit successor adapter, exact source/context/path association and
proof of the required full namespace coverage. One port P does not establish an
all-port premise. A V may block a later stage, not fabricate its evaluated V/U.

Propose EvidenceRef.raw_validation_member{subject,source_member_path} and
ObservationOwner.raw_contract_validation. Reference sets contain the complete
source-member/constraint schema references actually required by this predicate;
derive exact typed minimum primary, not RuleRef-as-EvidenceRef. Duplicate V adds
both original member refs. An observation of a legacy dimension is a separate
output path and keeps the original Raw origin. Root observations require real
caller/Checked associations; rejected Raw cannot establish them by itself.

## RC-HOOK — future contract only, NOT IMPLEMENTED

The earliest sufficient source is inside canonical_ports immediately before the
structured condition/error is reduced to strings; preserve original ordinal while
sorting and the actual predecessor. This is more precise than pretending the
outer `ContractError` already contains a full witness.

Future outer hook: `analyze_contract` before `invalid_contract_report`, borrowing
the actual raw/context and a branch-created typed evidence view. It may assemble
a non-authoritative rejection seed only with a matching original declaration /
document association. The native core need not gain a synthetic source document;
an outer capture adapter supplies an actual declarative source or fails explicitly.

Design lifetime contract: source document/raw live for 'input; context for
'context; typed evidence is owned by the validation attempt or borrowed only while
that attempt lives. Output local seeds cannot outlive any referenced input/context/
evidence. No public caller-provided witness can create a trusted Proven. No
Checked token, trusted Proven constructor, CompleteView or normalizer output is
returned. Capture/resource errors return no partial success and never alter the
legacy Checker Verdict. Future hooks require separate implementation authority.

## RC-MIGRATION — proposed normative/descriptor delta

Proposed normative source sections: RC-SCOPE, RC-CONTEXT, RC-STAGES, RC-PREDICATE,
RC-LOCATION, RC-WITNESS, RC-LINKS, RC-HOOK, RC-MIGRATION. They depend on the frozen
AF-CONTRACT-0004 and G01 0197–0200/0206–0213/0214–0221 safety laws; new governing
RuleIds and SpecificationId are **unallocated**, not placeholders in Registry.

Proposed descriptor additions: RawValidationSubject, RawDocument/RawDocumentMember,
RawInputOrigin/RawPhysicalAssociation/RawParsedInput, RawContextRole/Binding,
RawPortDirection/Occurrence/Predecessor, RawPortCheckBasis/RejectionWitness,
RawValidationCase(single case), RawValidationClause(single clause),
RawValidationFragment with exact applicability/execution roster and local facts,
and explicit parse/source-admission failures. Monomorphize all referenced types,
nominal widths, options, minimums, DAG roots/ranks and typed total orders in the
future registered table; no `Any`, opaque bytes-for-typed-values or fallback enum.
Bytes here are raw source/physical content, not substitutes for structured evidence.

Proposed parent deltas: append ScopeSubject.raw_contract_validation,
PredicateFamily/ClosedPredicateCase.raw_contract_validation,
RequirementClause.raw_validation, Source.raw_validation,
Target.raw_declaration_constraint, Coordinate.raw_port,
TypedContradiction.raw_validation, EvidenceRef.raw_validation_member,
ObservationOwner.raw_contract_validation, Fact.raw_validation_basis and
Origin.raw_validation_origin:Opt<RawValidationOrigin>.
Update exact profile/case/Kind/Role/Stage/Dimension mapping and source-member path
rules; retain existing Origin members with the explicit successor optional field
described above (None for unaffected families). Add no generic canonical/checked
Subject variants. This field addition changes successor identity payloads and must
not be backported into any current or historical nominal identity.

CompilerSemanticsIdentity impact: new normative model, report, obligation semantics,
governing source/rule content and compiler analysis owner identity require a
separately registered successor tuple; do not change meaning under Analysis@4 /
Model@3 / Report@3 / Obligation@3. No new Edition number is selected here. A later
registration must compute exact descriptors/ranks/counts/source pins and full
compiler dependency changes, not assume today's652/1941 stay true for a successor.

Current-only routing remains on the current exact tuple and denies all proposed
new tags/cases. Unknown edition/old-label-new-content/new-label-old-content and
spoofed support/activation must fail closed. A future successor's non-authoritative
staging still cannot issue RootExport. Historical tuples keep exact old source,
Registry, descriptor ranks, Expected and routing; do not rewrite historical imports.
Wire ABI/AFIDL/ACE1 grammar/HashV1 domains and substantive port/Guard/Substitution
verdicts are unchanged. New raw identity bytes are future model content, not a new
contract digest or HashV1 domain. Formal registration/normalizer/full Scope remain
separate work; this draft does not activate support.

## Decision boundary

C1-GAP-01 may be marked **AMENDMENT DESIGN ACCEPTED / FORMAL REGISTRATION PENDING**
only as design acceptance. It is not CLOSED-by-implementation, not Outcome B
existing binding, not T3 completion. The present direct call with no recorded
physical source/Origin cannot acquire them retroactively. The future adapter must
obtain a genuine declarative input association or fail MissingDeclarativeOrigin.
R3 remains incomplete; T4 not eligible; G02 P1/G03 P2 open; Positive RootExport
NotAuthorized. All design vectors have Actual Status=Not Executed.
