# Normative amendment draft — structural input binding

**PROPOSED / NOT REGISTERED.** SB-01…SB-16 are draft clause labels only.
No RuleId, source SpecificationId, edition, ABI/AFIDL assignment, type rank or
formal DagRank is allocated. All clauses apply only after a future explicitly
approved exact successor transaction. Existing model@6 and earlier meanings stay
frozen. Dependencies use real source/anchor records in authority-and-field-audit.

## SB-01 — closed structural declarations

The successor MUST materialize a finite complete StructuralSiteBinding table for
every value-eligible anonymous constructor reachable from the authorized profile
input roots. Each row binds the exact declaring nominal owner/name plus closed
field/case/item/key/value/option selector path, complete descriptor/constraints,
declared child types, registered named alias and type-order entry. Names are
resolved at registration, never synthesized from runtime values, hashes or Rust
layout. Nominal references terminate local path expansion; referenced bodies are
reviewed once under their actual declaring owner. Unknown/incomplete/cyclic alias
definitions fail closed. No global current-owner alias of historical imports.
Dependencies:0214/0217/0221, source `model-closed-descriptors`/`model-minimal-value-dag`.

## SB-02 — nominal and structural equality

The binder MUST retain exact expected ModelType identities. Resolving a ref/alias
body for shape checking MUST NOT equate different nominal/site identities merely
because their shapes/bytes agree. Registration MAY specify an explicit reviewed
transparent alias; absence of that entry means no cast. Complete typed equality
compares type and all semantic fields/cases/widths, not hashes or node indices.
Dependencies:0214/0215/0217,0256 `native-imports`.

## SB-03 — records, tuples and closed nested values

Every record field and exact case payload MUST be present under its descriptor;
tuple-shaped fields use their registered positional grammar, never native ABI.
Sequence order/duplicates, nested option tags and variant cases MUST be retained.
Missing, unit, null, None and Some are not interchanged. Raw companion bytes/order
MUST remain exact; canonicalization of a different ordinary string domain MUST
NOT repair a Raw claim or invent successful ComponentContract canonical validation.
Dependencies:ACE1-0001/0003/0013/0014;0214/0219;0261/0263.

## SB-04 — typed Map body and keys

The successor MUST add an explicit Map NodeBody case with ordered entry metadata
`{key:typed child rank,value:typed child rank}`, full K/V types and closed site
identity. Validate all entries and canonical key equality before interning.
Duplicate canonical keys MUST reject, including equal values; no last-write or
silent deduplication. Sort entries by the exact registered typed K comparator.
String-map NFC/key behavior follows ACE1-0005/0006/0007 only in its actual domain;
general nominal/Raw key domains do not inherit that normalization by shape.
No generic runtime Map Encoder or implicit Map-as-JSON-string fallback is permitted.
Dependencies:0214/0215/0217; ACE1 `ace1-map-key-*`/`ace1-duplicate-key-rejection`.

## SB-05 — Set versus sequence

The existing Set body remains distinct from Map and sequence. Actual Set members
MUST meet their exact semantic key/full-payload uniqueness, typed order, minimum
and domain constraints. Repeated supplied membership rejects before interning.
Native Set storage establishes only the actual resulting set, not past insertion
events. Equal sequence items remain repeated occurrences. Anonymous Set/member
sites require explicit closed bindings but not a fictitious missing Set body case.
Dependencies:0214/0215/0217, model canonical_constraints set uniqueness.

## SB-06 — total typed comparison

Comparison MUST implement exact nominal widths/domains, mathematical integer
order, exact rational Decimal, finite float after its existing frozen zero rule,
false<true, each domain's unsigned text/byte order, None<Some, declared record/case
order and lexicographic sequence prefix. Keys/values/sets compare logical typed
contents, not their ranks, decimal spelling, Debug or host hash iteration.
The successor MUST explicitly bind structural type-order entries to the frozen
type-identity/declaration ordering before assigning any input rank; see structural
design for the proposed owner/declared-rank/name total integration key. This
refinement is reviewed successor content, not an assumption of current T3 code.
Dependencies:0215 `model-typed-order`,0217; ACE1 scalar clauses.

## SB-07 — canonical minimal input graph

For the complete declared input root roster, validate widths/types/domain sets,
cycles and closure first; intern equal complete typed values bottom-up without
hash-only equality. Retain0217: scalar/empty-container height zero, otherwise one
plus maximum local-child height; sort each height by declared type identity,
nonrecursive body key and ordered local child ranks; assign contiguous U64 ranks
starting at zero, with strictly smaller local child ranks. Require exact roots
in profile field order, reachability, no redundant equal nodes and no alternate
numbering. Shared children retain every parent edge. No generated output,
unrelated synthetic root, salt or address may rescue an invalid graph.
Dependencies:0217/0218/0219 and current complete model constraints.

## SB-08 — reference environment, not bare integers

Node<T> MUST remain an actual typed DagRank resolving to exact T in the exact
canonical carrier. Ordinary U64 data is not a reference. Every supported profile
and relevant descriptor field MUST explicitly declare its owning input/payload
reference domain. Input graph G has only G-local data references. Direct Fact/C
output input references resolve only in the envelope's immutable G. Separate
witness DAG W has W-local children/ranks plus only expressly declared external
G input references; external references are checked against G and are not W-local
height dependencies. No numeric fallback to another graph/edition, relocation
guess, dangling child or untyped graph-hash ID is permitted. Exact binding domains
and envelope descriptors require registration together; old bare-rank meanings
are not changed retrospectively. Dependencies:0214/0217,0257/0269/0297.

## SB-09 — pure input graph and acyclic admission

The binder MUST consume actual declared source/Compiler/Context inputs and
verified native associations, not complete Facts, C results, obligations, reports
or capabilities. G MUST contain only profile-declared input closure; outputs
reference prior inputs and never occur as descendants determining input identity.
Structural candidate Subject data MAY be bound before C in the successor only;
it is not an admitted Subject usable by Fact consumers. The successor MUST amend
the old `full Subject materialization` wording explicitly to preserve this
data/admission distinction. The actual C checks and all Some C Valid results are
required before any complete Raw Fact is emitted. No C receipt/boolean is added
to source Context/A identity and no rank is backfilled after Fact generation.
Dependencies:0198/0200/0257/0269/0295/0297.

## SB-10 — occurrence and native associations

Source O remains complete OwnerAnchor plus actual document-entry ordinal, with
full package/doc/candidate and true member paths. Equal content nodes MAY share;
different legal O values MUST retain different complete Source/Origin bindings.
Observation does not create source. Same-content native Contexts do not establish
analysis/canonical_validation object aliasing. The binder MUST consume a genuine
native association receipt checked at actual use, not a supplied boolean/encoded
partition or serialized address. Equal carrier data under another native object
requires fresh needed association/admission checks. No UUID/time/call-count origin.
Dependencies:0200/0210/0247/0262/0272/0273/0275/0277/0278/0280–0283.

## SB-11 — native A and derived C

Native A MUST keep kind/family/revision/pin/full bytes/checksum/full T projection
and unchanged equality. C MUST use actual current modules and independent exact
slot/family/root/definition closure/schema-digest/current validation stages.
Found is not Valid; None executes no snapshot validation; each Some requires C
Valid for complete Subject admission. C Missing/Invalid retains A, failure and
the already executed port event but forbids a complete Fact. C is outside G and
cannot infer construction history or become a port predicate premise/verdict.
Dependencies:0286–0298, especially0295 `subject-admission` and0297 `input-output-dag`.

## SB-12 — errors and bounded failure

The future binding diagnostic SHOULD reuse existing genuinely applicable failure
domains, with a separately registered closed structural reason/site payload;
the payload names below are descriptions, not allocated enum cases.

| Condition | Applicable failure domain / handling |
|---|---|
| known required node/rank/source binding absent | IncompleteFacts; no placeholder or successful partial carrier |
| binder not implemented/authorized for supported profile | NotImplemented/staging refusal, not evaluated port failure |
| missing exact normative descriptor/source contents | IncompleteContent; stop before source/rank association |
| unknown nominal/site/shape/case/key domain | UnsupportedModel; no same-shaped/byte fallback |
| known value violates descriptor/domain, duplicate canonical key/member | InvalidFacts; retain exact typed site/conflicting values |
| invalid child, wrong target type/value, unreachable node, nonminimal equality | InvalidFacts, no graph repair |
| local rank/topological order violation, input/output backedge | InvalidFacts; no backfill or reordered successful graph |
| old/mixed exact tuple | external current-only PP taxonomy InvalidIdentity/UnsupportedSemantics as applicable; binder does not relabel |
| actual document/Context association failure | retain actual registered DO/Raw admission category, not a fabricated structural port Violation |
| work/depth/node/edge/key/output bound/overflow | ResourceLimit under intersected limits; no widening |

These diagnostics confer neither Proven nor Violation. RegisteredModelFailure's
actual Rust variants are diagnostic implementation evidence, not proof that a new
binding error payload is already a registered type. Official semantic failure
principles come from0218/0219/0225 and the existing DO/Raw source clauses. New
structured failure descriptors require separate future registration.

## SB-13 — stage authority and immutability

Only the separately registered pre-capture binder may construct/verify canonical
input ranks under its exact supported source tuple/profile. T3 preservation keeps
genuine input/event evidence; complete Raw assembly consumes admitted G references.
T4 owns semantic Fact consistency/obligation/identity/report normalization and
MUST NOT mutate/extend/re-rank G. Missing future evidence roots require failure,
a new complete carrier, fresh admissions and capture—not pointer/rank relocation.
G02 lossless Scope/cross-process reconstruction remains separately required.
No LocalTypedFactSeed borrowed view is promoted to Node<T>, complete Fact,
CompleteView or Checked. Existing other-profile PendingScope rules are unchanged.
Dependencies:0197/0198/0212/0255/0257/0268/0269/0300.

## SB-14 — exact edition and historical isolation

Any new structural names/body cases/reference domains/stage ownership require a
future exact source/rule/model/report/obligation/analysis transaction under0221
and0223–0227. All successor numbers/IDs are unallocated here. Preserve current
66/509/508/1/0,774/2204/97,ABI775/AFIDL17/HashV1-14 and all historical sources,
type ranks and Expected. An old rank is not migrated by numeric reuse; absent
source/provenance/binding stays nonmigratable, not reconstructed history.

## SB-15 — independent Oracle and coverage

Before implementation acceptance, independent source/descriptor/stage-derived
Required Nodes, types, fields, child/ref domains, roots, equality classes/ranks
and applicability MUST survive simultaneous Expected/Actual deletion. Rust
Actual is comparison-only. Compare full typed targets/values and canonical bytes,
not identities/counts/hashes alone. The48 G1 design vectors are Not Executed;
R4's74 boundary comparator mutations do not execute them or close full Fact gaps.
Dependencies:0220 `model-independent-fullfield`,0268,0301 `closed-model`.

## SB-16 — capability and separate authorization

Input binding success is non-authoritative structural availability, not a proof
predicate result, complete G01 Fact or Root capability. D4 positive proof,
CompleteView, CheckedRootExportCompatibility, publication/runtime/activation and
T4 admission MUST remain denied until their separately authorized complete gates.
This draft performs no registration, rank allocation, binder/producer/normalizer/
Scope/Oracle implementation or activation. Dependencies:0212/0213/0227/0300.
