# Pre-capture structural binding — registered contract v1

SpecificationId: `agentflow.computation.structural-binding.semantics@1`.
This is a **Registered / NotImplemented / NotAuthorized** stage contract, not a
production binder. The active tuple is Analysis@8 / Model@7 / Report@7 /
Obligation@7. Old complete sources and meanings remain isolated and unchanged.
The exact machine declarations/site table are in proof-report-model-v7.md.

## 1. Closed declaration sites and nominal identity

The value-eligible input closure starts at RawValidationSubject. The supported
output value closures start at RawPortCheckBasis, RawPortRejectionWitness,
CurrentContextRelationSchemaValidation and RelationSubjectAdmission. Named
references terminate local syntax traversal; referenced declarations are visited
once under their true owner. A declaration alias such as CompleteCompiler
delegates body sites to Compiler, but retains its own nominal node type.

Every inline constructor occurrence has exactly one table row identified by
the complete declaring ModelType and ordered selectors: field, case, sequence
item, set member, map key/value or option some. This includes inline scalar/unit
payloads, not just the earlier container-only audit. Each row binds an opaque
registered value-type name, full descriptor, exact children, source model/type,
eligibility, source stage, reference domain and local-rank dependency. Site
aliases delegate nested sites to the original declaration/path; their generated
bodies do not become new native declaration sites. Missing and extra rows reject.

Opaque StructuralValue names are allocated once in the closed registration
table. They are not runtime names, field-concatenated identifiers, shape hashes,
Rust-layout descriptions, source ordinals or value ranks. Same shape at different
owners/sites is not equal type. Body aliases do not create casts. All97 selected
historical imports and other exact qualified declarations retain original owners,
names and recursive closure. Old type-rank prefixes are unchanged in the new
catalogue; old catalogues retain their original ranks/bytes independently.

Record semantic field order and closed variant order are declaration order.
Positional fields such as `0` are registered tuple payloads, not unwrapped boxes
or Rust ABI positions. Missing/extra fields, unknown cases, None versus Some,
Some(None), unit versus null, sequence order and duplicates stay distinct. Raw
text/claim companions remain exact bytes; no NFC repair or ComponentContract
canonical validation is inferred from structural admission.

## 2. Map and existing Set

The successor NodeBody appends `map:Seq<StructuralMapEntry>` after all seven
existing cases. Each entry is `{key:DagRank,value:DagRank}` metadata, not an
extra value node. Exact K and V come from the registered site descriptor. Validate
keys, values, widths, nominal types and domain constraints before interning.
Reject duplicate complete canonical keys, including equal-value duplicates.
Neither last writer nor silent deduplication is allowed. Sort entries by complete
typed K comparison, never insertion order, rank spelling, Debug or a host hash.
The local child vector flattens key then value in that order; entry count is
nonrecursive metadata. Empty map height is zero; otherwise one plus maximum
local child height. Entry records cannot be synthetic G value nodes.

For actual compiled_modules, enumerate the actual CompiledModuleSet entries.
Parse each native String key as strict SpecificationId and require equality to
the CompiledSchema's full specification identity. Valid SpecificationId spelling
is canonical ASCII, hence injective: two different valid native keys cannot
collapse through Unicode normalization. Invalid keys reject, not normalize. The
value preserves specification_id and every complete canonical type definition,
including numbers, names, body and type expressions in canonical source order.
No global/latest module or schema-digest substitute is permitted. A real native
BTreeMap establishes the resulting unique entries, not earlier insertion history.
A supplied pair sequence can still contain duplicate keys and must reject them.

ACE1 string-map NFC collision rules apply only in their registered string-map
domain. Arbitrary nominal or Raw byte keys do not inherit those rules by shape.
The G01 map above is represented by existing variant/list/record/U64 shapes,
not by recasting arbitrary K as an ACE1 string key. ACE1 grammar v1 is unchanged.

Set keeps its existing body case. Bind its exact site/member type, original
minimum and domain-specific semantic key/full-payload equality/order. Repeated
supplied equal membership is invalid; sequences preserve repetition. Do not
transfer Map's key/overwrite rule to arbitrary record-valued Sets. Conflicting
domain keys obey their existing source rule; full typed membership is not a new
global attribute-key uniqueness rule. Native sets/maps reveal actual resulting
values only, not lost duplicate insertion events.

## 3. Canonical rank contract, not production allocation

Only the future independent pre-capture stage may construct and verify input
ranks. ModelType registration ranks order types; they are never DagRanks.
Node<T> remains a U64 rank resolving to exactly T in the exact carrier. Source
ordinals, addresses, map order and arbitrary counters do not satisfy it.

Preserve AF-SEMANTIC-0217: validate complete descriptors/widths/semantic sets,
cycles and reference closure first. Intern complete typed equal values bottom-up
without hash-only equality. Scalar/empty-container height is zero; otherwise
one plus maximum local child height. At each height order by exact descriptor
type identity, nonrecursive body key and ordered child ranks, assigning contiguous
U64 ranks starting zero. Local children are strictly earlier. No alternate
numbering, redundant equal nodes, unreachable nodes or incomplete successful
prefix is admitted. Preserve every parent's real edge to a shared child.

The active type integration key is exact nominal owner under its unsigned UTF8
scalar order, then active declared registration rank, then exact name. The old774
entry prefix is preserved; new entries append. Within a type the descriptor fixes
body kind. Scalar payload comparison is mathematical within exact widths;
Decimal compares exact rationals, finite float uses frozen zero canonicalization,
false precedes true, text/bytes use their registered unsigned order, options use
None before Some, records/cases use declaration order and sequences use prefix
lexicographic order. Map/Set compare complete sorted typed contents, not ranks.

Record nonrecursive keys retain exact field tokens/order; variant keys retain
the registered case rank; containers retain kind/arity. Map's ordered children
are typed-key-ordered pairs. Hashes are only candidate indexes. Full typed
equality always decides interning and comparison. Bounds intersect model@7 and
consumer limits: depth96, nodes1,048,576, work4,194,304, view134,217,728 bytes,
edge262,144 and all relevant key/source limits. Charge scans, numeric limbs,
edges, comparisons and sorting work; overflow/resource exhaustion returns no graph.

## 4. Immutable G and separately typed W

The Raw profile declares one G root, Node<RawValidationSubject>, matching the
successor ScopeSubject.raw_contract_validation.subject field. All required source,
Compiler, Context, occurrence/package/document/evidence values must be reachable
through those declared inputs. No output/report/Fact/Obligation/Scope node is a
descendant of G. A missing real input root is an incomplete profile, not license
for an arbitrary extra root. A fresh approved profile/carrier requires fresh
source admission and capture; T4 cannot append nodes, relocate or backfill ranks.

Every logical dag_ref has an explicit domain entry. G's references are G-local.
Output records' input references resolve only in their enclosing immutable G.
The policy key includes carrier use: g_value, w_value or output_record. The same
RawInputOrigin document-binding field is G-local inside G but external-to-G in W
or a plain output record; declaring-type membership alone cannot choose its arena.
Ordinary U64 data remains scalar. Node type, value, root roster, exact tuple and
complete graph contents establish the environment; no address, UUID, graph number,
digest-only lookup or numeric fallback can replace it.

W uses StructuralWBody, not an unannounced reinterpretation of old NodeBody.
Every body edge is `local:DagRank` or `input:DagRank`; field/type policy determines
which is legal. Ordinary data children are W-local. A registered logical dag_ref
to an input is a direct G edge, never a scalar rank wrapper pretending to be T.
Input edges resolve exact T in G and contribute no W-local height. No local child
means local height zero, otherwise one plus maximum local-child height. External
payload keys compare complete typed targets, not G rank numbers. Local rank order,
minimality, exact roots and reachability apply within W. Complete envelope equality
also compares full G/tuple/profile/root environment. A matching bare W body under
another G is not equal complete evidence. W never determines G identity.

## 5. Rank-free actual source and stage ownership

StructuralDeclaredInputs is a complete rank-free source contract: full Compiler,
expanded SourcePackage metadata, complete OwnerAnchor and actual document entry
ordinal, true member path, physical association path, actual Raw candidate and
full actual Context. Its recursive descriptor closure contains no dag_ref.
RawParsedInput itself cannot bootstrap the binder because its origin/physical
fields already require DocumentOccurrenceBinding nodes. Future binding constructs
those nodes from the actual expanded source; no temporary rank, borrowed Node
substitute or T4 output fills that prerequisite.

Source admission requires actual supplied owner/package/document agreement,
actual selected declaration/physical parse association, implementation-owned full
compiler/support sources/roles and actual analysis/canonical_validation object
association. Complete equal contents or a caller-supplied partition/assurance
enum do not prove native aliasing. Genuine native association receipts are
lifetime-bound and not serialized addresses. The current metadata API does not
create such receipts. Neither provenance nor source events can be reconstructed
from Debug, old reason strings, checksums, current schemas or a new edition.

The future acyclic schedule is declared inputs -> source admission -> binding G
-> actual C/admitted Subject -> complete Raw Fact capture -> future T4. Real port
events may be source-preserved before G; their complete Fact output is not a G
prerequisite. Bad port zero remains legal Raw input and its real port verdict is
unchanged. G consumes no C result, Fact, report, obligation or future Scope.

T4 owns semantic Fact consistency/obligations/normalization, never input repair.
G02 still owns complete lossless Scope and remote integration; it is not completed
by this delegation. Other producer profiles and all T4–T8 stages remain separate.
The structural stage enum is not a new G01 predicate-evaluation Stage case.

## 6. Failures and non-authoritative checks

Use the registered StructuralFailureDomain and closed reason/stage/path payloads.
Unknown type/site/shape cases use UnsupportedModel; absent normative content uses
IncompleteContent; a required known field/node/rank binding is IncompleteFacts;
known invalid references, duplicate canonical keys/members, rank/interning or
backedge violations are InvalidFacts; bounds are ResourceLimit; an unavailable
production stage is NotImplemented. Exact edition transport taxonomy remains
Portable's existing InvalidIdentity/UnsupportedSemantics, not a relabeling rule.
The reason/source/stage matrix specifies applicability; diagnostic text is not
typed evidence. Keep conflicting source paths and full declared inputs. No failure
becomes a Raw Port Violation or an evaluated Unproven predicate.

Current APIs only check registration/catalogue/site/shape claims and bounded
manually supplied conformance subgraphs. They do not build graphs, assign actual
production ranks, seal source receipts, emit Raw Facts or authorize downstream
consumers. Even `native_associated` or a structurally valid graph in caller data
is a claim, not successful native admission. Production entry is NotImplemented.
Handwritten registration ranks are not production Binder Capture CaseIds.

<!-- Generated rule index follows; statements are part of this frozen source. -->

<a id="sites"></a>

## AF-SEMANTIC-0302

MUST: Every eligible inline value site has one exact owner/parent/selector binding, complete descriptor and child types; missing or extra source-less sites reject.

Direct normative dependencies: AF-SEMANTIC-0214, AF-SEMANTIC-0221.
Registration conformance: g01-structural-binding-staging/rule-0302.

<a id="nominal-order"></a>

## AF-SEMANTIC-0303

MUST: Nominal types and site identities remain exact; aliases delegate declaration bodies without casts and type-order integration preserves all old ranks.

Direct normative dependencies: AF-SEMANTIC-0215, AF-SEMANTIC-0256.
Registration conformance: g01-structural-binding-staging/rule-0303.

<a id="map-body"></a>

## AF-SEMANTIC-0304

MUST: Map uses typed key/value child entries with canonical typed order and duplicate canonical key rejection before interning; native module mapping is full and source-exact.

Direct normative dependencies: AF-SEMANTIC-0214, AF-SEMANTIC-0215, AF-ACE1-0007.
Registration conformance: g01-structural-binding-staging/rule-0304.

<a id="canonical-ranks"></a>

## AF-SEMANTIC-0305

MUST: Only the future pre-capture binder constructs bounded minimal input graphs under0217 full typed equality, canonical height/type/body/child ordering and exact roots.

Direct normative dependencies: AF-SEMANTIC-0217, AF-SEMANTIC-0218, AF-SEMANTIC-0219.
Registration conformance: g01-structural-binding-staging/rule-0305.

<a id="input-environment"></a>

## AF-SEMANTIC-0306

MUST: G references resolve exact typed inputs in their fixed complete carrier; ordinary integers, other arenas, output backedges and synthetic roots never repair missing bindings.

Direct normative dependencies: AF-SEMANTIC-0269, AF-SEMANTIC-0297.
Registration conformance: g01-structural-binding-staging/rule-0306.

<a id="witness-environment"></a>

## AF-SEMANTIC-0307

MUST: W local/input edges are explicit; external G targets use full typed comparison without W-local height and complete envelope equality includes G and exact tuple.

Direct normative dependencies: AF-SEMANTIC-0217, AF-SEMANTIC-0297.
Registration conformance: g01-structural-binding-staging/rule-0307.

<a id="source-stage"></a>

## AF-SEMANTIC-0308

MUST: Complete rank-free declared inputs and genuine native source associations precede immutable G, actual C admission and complete Fact assembly; T4 cannot backfill input ranks.

Direct normative dependencies: AF-SEMANTIC-0198, AF-SEMANTIC-0262, AF-SEMANTIC-0268, AF-SEMANTIC-0278.
Registration conformance: g01-structural-binding-staging/rule-0308.

<a id="failures"></a>

## AF-SEMANTIC-0309

MUST: Structural diagnostics use closed applicable domain/reason/stage/source paths, preserve complete source and never become evaluated port predicates or partial successful graphs.

Direct normative dependencies: AF-SEMANTIC-0218, AF-SEMANTIC-0219, AF-SEMANTIC-0225.
Registration conformance: g01-structural-binding-staging/rule-0309.
