# G3 production Input-G binder

Status is provisional until the fresh gate receipts and final runtime report
exist. This is a **non-authoritative structural input** boundary, not a Raw
predicate producer, checked contract, report normalizer or proof issuer.

## Source and stage boundary

`bind_raw_input_graph` accepts the caller's complete declared SourcePackage,
actual owner anchor, actual document-entry ordinal, actual member path and
actual document reference. Existing source admission checks the complete
package and Native C/K object association before a graph exists. Equal content
does not replace actual owner/slot membership or pointer association. All
declared native contracts are genuinely deserialized, including declarations
not selected as the local candidate. Physical documents retain original bytes
and their actual member path; duplicate JSON keys are not silently admitted.

The implementation-owned compiler source is exact current Analysis@8: 70 full
normative sources, four closed tables, 44 selected specifications, 217 applied
rules, ten semantic-role declarations and Profile A full-content attribution.
The full source bytes enter G, not just their checksums. Profile B and the
future_dependency reason are not caller-selectable current compiler sources.

The nine Native Context members are projected from actual typed iterators,
lookups and complete schema bodies. A CompiledModuleSet String key is parsed
and its canonical SpecificationId checked; lookup must return the same actual
CompiledSchema yielded by the private native map. Canonical specification
grammar makes this valid-domain mapping lossless and injective. Logical type
keys include the actual complete schema digest. Public execution-universe Map
keys are retained, never synthesized from descriptor values.

The original first dynamic blocker remains recorded as
`RawValidationSubject.context.contents.compiled_modules`.
`Compiler.specifications` is earlier in a static traversal, but no earlier
dynamic failure was established. Both now have actual complete producers.

Native Relation A snapshots are projected using their full existing wire
content, bytes, revision, family and pin. Test drivers assert each native
snapshot constructor leaves the complete supplied canonical catalog unchanged.
Only after an immutable G is returned does the driver independently execute C
metadata/Raw port validation. C-Valid and C-Missing are both tested. Neither
is a model@7 Subject admission, complete Raw Fact or positive capability.

## Construction and order

Private rank-free expanded source values carry actual logical targets for
registered dag_ref sites. They contain no caller-selected DagRanks. Private
PendingId handles are local construction indices, not published ranks.

The engine reads exact frozen Model@7 descriptors, declaring parents and site
paths. Alias bodies delegate to their actual declaration while the resulting
node retains its nominal alias identity. Anonymous records, positional tuples,
inline records, options, variants, sequences, Maps and Sets therefore use the
actual registered site and exact child type, not Rust layout or a Debug name.

Preflight completes shape/width/NFC/UTF8, complete child and Map/Set duplicate
checks before any value node is interned. SHA256 only indexes candidate buckets;
full typed equality decides membership. All 15 registered Set sites have
separate member semantics, not Map-entry semantics. Equal source contents may
share child nodes but complete owner/occurrence bindings remain in their parent
values. No semantic owner or slot is deduplicated away.

Scalar comparison is mathematical for integers/Decimal and unsigned-byte
lexicographic for text/bytes. Options and variants use registered case order;
container comparison is typed lexicographic. Map keys and Set members are
ordered by complete typed value, never by their eventual DagRank. Interning
uses nominal type plus full body/child equality, even on a hash collision.

Final ranks follow AF-SEMANTIC-0217: dependency height, nominal owner, frozen
ModelType rank/name, nonrecursive typed payload and ordered final child ranks.
Ranks are contiguous U64 and actual children precede parents. The only root is
Node<RawValidationSubject>. All nodes are reachable and no equal duplicate node
or unused extra node is returned. This is not a T4 re-ranking or post-fact
backfill. Temporal scalar NodeBody payloads use the registered I64 microsecond
carrier, preserving the underlying typed timestamp/duration identity.

ModelType ranks (774 inherited + 357 appended), produced value DagRanks,
document entry ordinals and Rust iteration positions remain distinct.

## Policy and trust boundary

All 66 registered reference policies are retained in the audit manifest.
Only registered g_value / g_local edges may be expanded in this G profile.
W/output/other-profile policy rows are not runtime production coverage and
cannot authorize an input edge. Wrong environments, output backedges, missing
policies, wrong roots and cross-edition types are independently rejected by the
closed graph comparator. The source API cannot submit arbitrary node arrays or
numeric references at all.

VerifiedInputGraph has private fields, no Clone/Deserialize/public constructor,
owns its immutable graph/bytes and retains actual source/context borrows.
RawSubjectInputNode contains a real assigned rank and its exact sealed carrier.
Compile-fail tests reject caller construction and deserialization. The public
registered output claim is plain data only; spelling native_associated cannot
create a native receipt or capability. `can_issue_root_export()` is always false.

## Limits and failure closure

No existing resource limit is increased: depth 96, 1,048,576 value nodes,
262,144 graph edges, 134,217,728 scalar/view bytes and 4,194,304 work units.
Source association retains the previous bounded package/key/compiler limits.
An iterative native source budget precedes recursive schema projection and
large graph allocation. Scans are charged in fixed 64-byte blocks, and numeric
comparison, candidate checks and sorting also consume bounded work.

Failures carry the existing closed StructuralBindingFailure metadata with the
source_admission or pre_capture_binding stage. A failure exposes no partial G,
complete Raw Fact, TypedContradiction or positive capability. A theoretical
U64 DagRank overflow is unreachable before the much smaller default node bound;
overflowing external rank tokens are separately rejected by the typed decoder.
Large/deep Native inputs exercise actual resource failure, not a hand-written
graph stand-in. NFD Native text outside the frozen source domain is fail-closed,
not normalized into a different source identity.

## Coverage meaning

The complete site census has 317 rows: 253 applicable G sites, four inactive
sites in the fixed implementation-owned compiler profile and 60 W-only sites.
All four inactive dispositions are source-proved, not missing tests for a
caller-selectable branch: Profile B direct/transitive/dependency attribution
and ModelMissingReason.future_dependency. All 15 Set sites are applicable G.
The independent descriptor worklist requires 455 actual value types and 538
field/variant members; it is not derived from the proposed Actual/Expected graph.

154 positive production CaseIds, 23 actual source-admission negatives and three
resource/nominal boundaries are separate rosters. The 278 source declarations
are native-deserialized data, not 278 claimed production cases. Comparator
mutations, 3,662 registration cases, old R4's 74 mutations and the original 48
G1 design-only vectors are also separate. G1 receives no automatic conversion
credit: its exact original vector contract is not replaced by a related new
scenario or corrupted-graph mutation.

## Independent verification and reproduction

Python Expected derives only from declared sources, exact registered types,
sites, policies and independently compiled source data. No Rust graph, receipt
or log feeds Expected. `strict_compare` rebuilds the required graph from source
even when both proposed views omit the same material. A separate typed decoder
validates every body, type, edge, root, rank, membership, ordering, minimality and
reachability. Full canonical bytes are checked in addition to typed semantics.

The runtime ledger records full graph instance denominators separately from
type/field/site coverage. Source association rows identify actual source paths
and corresponding canonical graph nodes. Source Set storage ordinals are never
substituted for canonical graph positions. Four map-order mirrors, one exact
repeat and three controlled equal-content owner/slot changes test equality and
input sensitivity independently of changes to compiler or Context.

```powershell
$env:PYTHONUTF8='1'
$env:RUST_TEST_THREADS='2'
cargo fmt --all -- --check
cargo clippy -j 2 --all-targets --all-features -- -D warnings
cargo test --all -j 2
cargo run -p af-spec --bin spec-check -- check
python tools/oracle/production_input_graph.py --all
python tools/oracle/production_input_graph.py --mutations
python tools/audit-production-input-graph.py --receipts
python tools/audit-structural-binding-staging.py --receipts
python tools/oracle/raw_model6_binding.py --gate total
```

Strict Raw Total must remain exit 2, because no complete Raw producer has been
implemented here. Full engineering and seven exact-history results, freshness,
immutable byte maps and original-directory generator rounds belong to the
phase-local gate receipts, not to Cargo success alone. This phase never pushes
and does not authorize any continuation into Raw full-field capture or T4.
