# Design-vector input detail — all Not Executed

This is a supplement to the 48 uniquely identified rows in design-vectors.json,
not 48 executed fixtures or additional case credit. Every row's Rust Actual and
assigned_ranks stay null. `P`, `Q`, `x`, `y`, `k`, `v`, `G` and `W` below are
mathematical placeholders, not allocated ModelType names, graph IDs or ranks.
All positive inputs assume a future exact registered profile, complete real
compiler/source/Context inputs, valid applicable domain constraints and genuine
association/admission. Omitting those prerequisites is not a successful seed.
Expected design is derived from the cited frozen rules plus the explicitly
proposed SB clauses, never copied from Rust Actual. No numbering example is given.

## Anonymous / closed nested carriers

| JSON vector suffix | Structural input detail | Required distinction |
|---|---|---|
| same-typed-value | two parent fields expect the same exact nominal/site type P and contain the same complete x | one content equality class, both parent edges |
| different-nominal | equal field shapes and values under different declaring owners P and Q | different types; no shape cast |
| nested-record | P has an anonymous record field whose own field is another closed record | bind every anonymous constructor occurrence, not only P |
| inline-struct | FullSupportManifest.closed_tables contains all four actual byte arrays | all bytes and declared field order, no count/checksum replacement |
| tuple | declared positional fields contain x then y; negative transposes their positions | exact registered tuple-shaped order, not native ABI layout |
| nested-option | None, Some(None), Some(Some(x)) at the same registered nested Option site | three logical values and complete tag/child relationships |
| sequence-order | [x,y] versus [y,x], with x and y distinct complete same-typed values | distinct sequence values; no sorting of sequence data |
| closed-variant | a declared case contains its complete anonymous record payload | exact case, payload type, fields and edges |
| unknown-variant | replace that declared case with an unknown case | UnsupportedModel/closed-shape refusal, not a port result |
| missing-record-field | delete a required anonymous record member | no default, no successful partial carrier |
| same-site-alias | CompleteCompiler resolves the registered Compiler body and its actual child sites | exact ref/owner preserved; body resolution does not mint a new child identity |
| cross-site-shape | two declaring fields have equal anonymous descriptors but no transparent-alias entry | different site identities remain different |

The same-typed-value case also checks replacement of only one parent target
with distinct same-typed y: full target comparison must detect the difference.
That is a refinement, not a new vector. The machine list is the authoritative ID roster.

## Map / Set carriers

| JSON vector suffix | Structural input detail | Required distinction |
|---|---|---|
| insertion-order | identical complete (k,v) pairs supplied in two host insertion orders | one typed canonical map |
| conflicting-key | duplicate canonical k with distinct v and v' | reject before interning; no last writer |
| equal-duplicate-key | duplicate canonical k with equal v | also reject supplied duplicate entries |
| nominal-key | same spelling/width under different nominal key types | exact expected K, no coercion |
| nested-map | Map values include a nested anonymous Map whose values include a Set of closed records | bind outer/inner Map and Set/member sites; Map and Set retain distinct grammar and uniqueness |
| optional-record-map | Map values include None and Some(complete anonymous record) | preserve option tags and full record closure |
| duplicate-set-value | a raw supplied member sequence contains equal complete x twice | reject duplicate membership before interning; genuine native Set storage proves only its resulting members |
| set-order | the same valid member collection appears in different host orders | canonical typed semantic-key/full-payload order |
| numeric-key-order | same U64 map contains data keys two and ten | mathematical numeric order, not decimal text or rank order |
| nfc-key-collision | ordinary string-map domain supplies canonically equivalent Unicode keys | domain NFC collision rejects; not silent normalization plus overwrite |
| raw-byte-key | a registered Raw byte-key domain supplies distinct bytes with equivalent displayed text | preserve byte distinction; do not import string-map NFC |
| empty-map | empty Map and empty Set at distinct exact types | both typed; not one unspecified empty body |

The nested-map refinement explicitly exercises nested Map **and** Set. All
domains/constraints remain descriptor-driven. The notation is a proposed future
typed input, not an invented runtime Map encoder or executable test seed.

## Nodes / ranks / graph references

| JSON vector suffix | Structural input detail | Required distinction |
|---|---|---|
| shared-child | two parents contain the same complete child x | one interned child with both required edges |
| missing-rank | an otherwise required input target has no admitted carrier binding | fail missing binding, never invent a rank |
| wrong-rank | a reference resolves to different type Q or different value y | compare actual full target, not existence alone |
| dangling-reference | a target is absent from its declared carrier | no fallback to another arena |
| unreachable-node | add unrelated x outside the fixed authorized input roots | reject, not synthetic-root rescue |
| duplicate-equal-node | two nodes represent equal complete typed x | reject nonminimal representation |
| alternate-numbering | permute the same logical graph's numbering | reject noncanonical numbering |
| rank-dependency | a local child is not strictly earlier than its parent | reject topology/rank violation |
| root-order | permute the registered subject root list | reject; equal-valued duplicate roots remain permitted |
| both-omit-node | remove one required node from Expected and Actual | independent source/descriptor roster still requires it |
| ordinary-u64 | an ordinary scalar's data spelling coincides with a reference spelling | no reference coercion |
| output-backedge | G requests a Fact/C receipt/Obligation value as a descendant | input/output conflict rejects, no backfill |

## Stage / history / authority

| JSON vector suffix | Structural input detail | Required distinction |
|---|---|---|
| document-occurrence | identical complete documents at two valid different owner/slot occurrences | content sharing allowed; full Source/Origin distinction required |
| same-occurrence | repeated legitimate observations of the same complete scoped fact | same origin; merge only legitimate observations of equal full non-observation payload |
| a-versus-c | same exact A under different actual current module definitions | preserve A equality, distinguish real C input/result observations |
| t3-without-binder | genuine sealed event exists; no full input G | keep local/PendingScope evidence only |
| old-content-only | old local/content-only record lacks complete source/rank binding | no successor relabel/migration |
| cross-edition-ref | target has equal rank spelling/shape in a different exact tuple/carrier | no cross-environment lookup |
| root-capability | assume future structural input binding succeeds | CompleteView/Checked/positive RootExport remain denied |
| c-missing | actual Some slot cannot find its required current module | retain A, C failure and real port verdict; no full Fact |
| split-context | analysis and canonical_validation receive different same-content native objects | fail genuine association, not repair by encoded equality |
| t4-rerank | semantic stage tries to append an input and reuse existing ranks | fail incomplete roster; fresh complete binding/admission/capture required |
| bootstrap-cycle | structural binder asks for final Fact/Scope/Obligation output first | reject stage dependency conflict |
| resource-budget | full construction/validation exceeds any intersected bound | ResourceLimit; no successful prefix or widened budget |

All 48 original IDs remain **Not Executed**. These symbolic refinements do not
increase executed cases, required runtime Nested/Relation denominators, formal
type/rank allocations, or completed Fact comparisons.
