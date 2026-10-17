# Structural values, Map/Set and canonical equality — proposed design

**DESIGN ONLY.** Symbols here are draft constructs, not registered names, type
ranks or generated value ranks. Baseline owners/fields remain unchanged.
Dependencies: AF-SEMANTIC-0214/0215/0217/0218/0219/0221;
AF-ACE1-0001/0003/0005–0007/0013/0014. See the authority inventory for exact anchors.

## Closed registration-time structural-site bindings

Recommend a successor-only, finite closed **StructuralSiteBinding table**. A site
key comprises the exact declaring nominal owner/name and a declared selector path:
field(name), case(name), item, key, value or option payload. Traverse finite
descriptor syntax; stop at nominal ref boundaries, resolving the referenced
declaration separately. Alias cycles and unknown declarations reject. Do not
unroll recursive type references into an infinite name tree.

For each value-eligible anonymous constructor occurrence the table supplies a
reviewed, registered named alias, its full constructor/constraints, children and
type-order entry. Future registration materializes these names and appends their
type ranks; **none is named/allocated here**. `alias(site)` below is notation,
not an accepted ModelType.name. Parent child-type bindings become explicit only
under the successor owner. Old source/owner/type ranks remain exact.

The identity is declaration-site nominal identity, not a hash of shape. Equal
shapes at different sites/owners remain different unless a separately reviewed
explicit alias entry states otherwise. A ref selects its exact declared nominal
identity; resolving an alias body for field validation never permits a cast to
another same-shaped type. Alias bodies' child sites belong to their actual
declaring nominal body. This prevents a CompleteCompiler body alias from
silently duplicating or renaming Compiler's structural children.

Anonymous records retain every declared field and semantic order. Tuple-shaped
payloads retain the exact registered positional field names/order, not ABI memory
layout. Nested sequences retain order and duplicates; options retain None,
Some(None) and Some(Some(value)); variants retain exact closed case and complete
payload. Empty record/list/set/map/None still require their exact type. Ordinary
string fields obey their registered NFC rules; Raw companions preserve original
UTF8 bytes and invalid claim order without repair.

Register bindings for the complete authorized input-root closure, including
Compiler/FullSupportManifest and all nine Context members, not only the first Map.
The finite static roster is an audit input, not proof that every variant is
runtime applicable. Nodes describing Graph/Fact/Scope/Report metadata are not
silently added as source input values to make a graph self-describing.

## Body grammar and reference distinction

Retain existing Node<T>/DagRank and ModelType owner/name. Proposed successor
NodeBody appends a **map** case containing a sequence of entry records with
`key` and `value` typed child ranks. This is a draft grammar, not an implemented
encoder, an allocated case rank or a new ACE1/AFIDL primitive. Entry records are
map-case metadata, not fabricated value nodes requiring arbitrary type names.

Every actual value node has an exact registered expected type, including a
closed structural alias when needed. Resolve ordinary record/sequence/option
children in the owning graph. For an existing `dag_ref target:T` descriptor,
the reference targets a T node directly; do not invent a scalar-U64 value node
and pretend its numeric value proves that association. Ordinary U64 data is a
scalar node even when its value happens to equal a rank spelling.

The registered successor reference-domain table must distinguish input-data
references, local output-payload edges and input references from output/witness
payloads. An envelope supplies exactly one immutable input carrier G; numbers
cannot be resolved in whichever graph happens to contain that rank. Output DAG W
has its own local child/root/rank space; an expressly declared external input
reference resolves in G and is not a W-local child for height/topology checks.
Cross-graph references and environments require explicit successor descriptors
and field bindings, not ambiguous bare integers or retroactive old meanings.

## Map<K,V> and Set<T>

Validate the exact K/V/member descriptors, nominal identities, widths, constraints
and complete values first. A source list of pairs is not accepted as a Map merely
because it has length-two arrays. Reject duplicate canonical keys **before**
interning: conflicting values and duplicate equal values both reject. No last
writer, silent key collapse or host-container cast supplies uniqueness.

Sort map entries by the registered complete typed K comparison. Numerics compare
mathematically (two before ten), bytes/text by their own unsigned-UTF8/byte laws,
records by declared fields and variants by their registered case order. No rank,
HashMap iteration, Debug string or serialized-decimal order stands in for this.
For ordinary ACE1 string-map input only, NFC-equivalent keys collide and reject
per ACE1 0005–0007; Raw-byte key domains are not normalized into text. Do not apply
string-map NFC rules to arbitrary nominal keys or Raw claims.

Set members use the descriptor/domain's registered semantic key/full typed
payload, uniqueness and minimum rules. Reject repeated input membership before
interning; preserve duplicates in sequences. A genuine native BTreeSet already
lost prior insertion history: capture the actual set, never invent duplicate
events. A Set body exists already; preserve it, rather than inventing a Map-like
case for all containers. Nested map/set/optional-record children each retain
their closed site/nominal binding and complete target relationships.

## Canonical graph law (algorithm design, not executed)

After complete shape/domain/association and reference-closure validation, identify
equal complete **typed** values bottom-up; hash buckets may accelerate but never
replace full equality. Preserve sequence and semantic record/case order; normalize
set/map order by their typed comparators. Repeated equal child values share one
node, but every parent's actual typed edge remains. Reject cycles/unreachable
nodes, duplicate equal nodes and alternative numbering at decode/verification.

Keep AF-SEMANTIC-0217 height and contiguous U64 rank rules exactly. The successor
must explicitly register its total descriptor-type order: exact owner under its
registered scalar order, then declared numeric type rank, then exact name as a
collision guard; retain all pre-existing declared type order within each owner.
This is a proposed disambiguation of structural type-order integration, not a
claim that a current T3 implementation already uses that comparison. New type
order entries are appended in future registration, never inferred at runtime.
At equal height, compare type identity, the registered nonrecursive body key,
and ordered child ranks. No successful result is returned before complete
acyclicity/reachability/minimality/canonical ordering checks pass.

For the proposed map body specifically, its local children are the key then
value of each entry in typed-key order. The ordered child-rank vector is the
flattened sequence of these pairs; entry count is nonrecursive metadata. Empty
maps have height zero; otherwise height is one plus the maximum of those local
child heights. Entry metadata is not a synthetic extra value node. Within one
registered map type, constructor and K/V binding are fixed, so count and ordered
complete children determine the body without a hash or insertion-order tie.
The successor appends the map constructor's declared case order without changing
the seven existing cases. External G references in W are separately declared
reference payloads, not extra W-local children or implicit inputs to G. Their
exact typed targets are still checked before W can be accepted. These are draft
body/order rules, not a generated rank example or a current model@6 extension.

Under W's fixed verified G environment, external-reference payload comparison
uses complete typed target values, never mathematical comparison of G rank
spellings. Those target keys participate in nonrecursive payload comparison;
they contribute no W-local height. Envelope equality also compares the complete
G, exact tuple and profile/root environment. An equal numeric W body in two
different G environments does not establish equal complete evidence or source.
For that explicitly registered successor W grammar, no local child means local
height zero; otherwise use one plus the maximum local-child height. The exact
body/reference-domain declarations must classify external payloads versus local
children, not let a caller reinterpret current NodeBody edges. This is proposed
new descriptor/profile content; the input graph G's0217 law stays unchanged.

Profile-declared source roots have fixed field order; repeated equal roots remain
permitted. Source identities retain their declared data; a synthetic root may
not rescue an otherwise unreachable node. Bounds intersect existing D0–D10 and
model bounds (depth96, value nodes1,048,576, work4,194,304, view bytes134,217,728,
edge budget262,144 and relevant key/source limits); none is widened. Charge scans,
limbs, node/edge visits and typed/sort comparisons as already required.

## Canonical bytes versus semantic order

ACE1's byte grammar remains v1. The proposed map case is encoded through explicit
existing record/variant/list/string-numeric shapes. Product members emit unsigned
UTF8 field-name order; body sequences retain their declared semantic order;
map-entry sequences use typed-key order. A G01 Map<U64,V> is **not** recast as an
ACE1 Map<string,V> to obtain sorting. New carrier/report canonical bytes and their
descriptor/content identity necessarily change and require a successor; this
does not edit the global ACE1 grammar, 17 AFIDL declarations or 14 hash domains.
