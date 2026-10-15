# ACE1 feasibility, injectivity and size review

**PROPOSED — NOT REGISTERED. Report-local encoder/decoder: NOT IMPLEMENTED.**

## Existing codec feasibility

Inspected `spec/encoding/ace1-grammar.md`, `af-core/src/shape.rs`, existing AFIDL shape lowering and Portable registered shapes. Shape already supports closed Record/Variant/Enum, List, Option and scalar primitives. New report Shape values can be constructed without extending ACE1 or AFIDL. A recursive semantic DAG lowers to a finite list of nonrecursive node records with uint64 child ranks; native recursion is not an encoder feature requirement.

| Required model | Existing ACE1 materialization / rejection |
| --- | --- |
| Closed typed products/sums | RecordShape/VariantShape with exact fields/cases; unknown/duplicate/missing rejects |
| Nominal U32/U64/indices | Exact decimal strings at declared widths; typed wrapper prevents node2/ordinal2 substitution |
| Exact Decimal | Full coefficient/scale canonical decimal, no binary floating intermediary; precision charged to scalar/work budgets |
| DigestRef / TypeSchemaRef | Existing full record shapes and nominal validators; algorithm/domain/all32 digest bytes retained |
| Bool/unit/null | Existing shapes; unit/null differ by descriptor although same null bytes |
| Typed options | none:null / some:T; None != Some([]); no omitted-field sugar |
| Raw bytes/claims | Canonical unpadded base64url; no NFC repair of invalid/NFD claims |
| Semantic sequences | List in declared order; includes invalid assertion list order |
| Sets/maps | Typed-sorted List of complete members or key/value records; duplicate typed identity rejected; ACE1 object-key sort alone is insufficient |
| Complete Value DAG | Finite list of typed node records, checked canonical ranks/reachability/minimality, complete root vector |
| Report text wrapper | Identity is an ACE1 UTF8 string; enclosing report String re-escapes quotes/backslashes exactly once more |

Enum/tag/rank declarations are model meanings, not wire ABI allocations. Report-local scalar ModelType tags preserve nominal equality where wire representations coincide. The complete descriptor is bound by owner content, so ACE1's unit/null or u32/u64 representations cannot be type-confused under the *same fixed shape*. Across editions there is no untyped-byte equality claim.

## Injectivity argument and counterexamples

Let E be schema-directed ACE1 of the canonical complete Identity under one exact supported RM and OS; decode validates all typed invariants, including canonical minimal DAG. Primitive grammars are injective on canonical nominal values; products use fixed fields; sums/options retain tags; sequences retain length/order; sets have unique sorted members. DAG interning identifies precisely equal unfolded typed values, not independent declaration occurrences. Origin paths/ordinals remain payload even when other DAG nodes intern equal. Induction over finite DAG height plus record composition gives decode(E(i))=i. Therefore E(i)=E(j) iff i=j **as complete typed identities**. Status/links are outside Identity and must still be compared separately.

| Adversary | Exact consequence |
| --- | --- |
| Different source/same reason | Source IR/node/contract changes E; reason not read |
| Different ordinal/same atom | Origin/source ordinal changes E, two identities |
| Same context counts/different member | Full context node differs; different Scope/E |
| Same pins/different resolver association | Caller/source/target association and context partition differ |
| None vs Some([]) | Different explicit option tag, hence different bytes |
| Repeated equal source rows | Equal content may intern, but distinct declared origin indices yield different identities |
| Shared DAG vs copied equal DAG | Same canonical minimal reachable DAG, hence same value identity |
| Same ID/different status/observations | Same identity string allowed; full Obligation differs, comparator must catch it |
| Duplicate or alternative DAG rank | Decoder rejects, never renumbers silently |
| Changed ordering/rank under old edition | Unsupported exact model/noncanonical collection; not formatting-only |

Outer Portable Bundle continues its existing registered AFIDL@1/ACE1 shape. The successor's new report shape is **not embedded in or substituted for Bundle**. Compiler manifest *values* necessarily change for a new content edition, but no field grammar/hash meaning changes. Existing ComponentContractDigest, SemanticIrDigest, TypeSchemaDigest and all14 HashV1 domains remain byte-for-byte unchanged for unchanged semantic inputs. There is no ReportDigest/ObligationDigest or digest fixed point.

## Real limits and proposed additional budget

Current code bounds are implementation policy, not proof predicates:

| Stage / source | Current bounds |
| --- | --- |
| D0–D4, limits.rs default | input32MiB; scalar1MiB; container100000; total JSON nodes1000000; nesting96; graph work2000000 |
| D5, reconstruction.rs34–40 | modules256; module types1024; reconstructed schemas16384 (global+resource combined checked against32768); catalog rows100000; schema-cache material32MiB; type expansion depth256/work1000000 |
| D5–D8, pipeline.rs | reconstructed items100000; resolver targets4096; resolver closure work2000000 |
| D9, pipeline.rs31–32 | computations1024; derivation work2000000 |
| D10, root_revalidation.rs56–61 and pipeline.rs | mappings2048; structural nodes/work100000/depth96; assertion graph32; nested identity expansion50000; comparison work2000000 |

G01's1024 **assertion** nodes is an additional report-model ceiling, NOT permission to admit1024 Portable assertion nodes past current32. Effective bounds are the intersection of every earlier limit and G01. No current limit is raised by this plan.

For proposal D13 retain1024 distinct assertion value nodes/depth96, combined originating Fact+Prerequisite+Obligation keys<=65536, work<=4194304, **total complete output<=128MiB**. Add explicit sub-budgets: reachable Value DAG nodes<=1048576; source occurrences<=65536; reference+observation+dependency edges<=262144, all also charged to work/output. These are new report policy proposals, not production values. Per-process stricter declared limits may reject; never alter equality or silently truncate.

A work step is one descriptor/node/edge visit, one compared typed-node pair, one sort comparison, one exact numeric64-bit limb operation, or one64-byte block scanned/copied/encoded. Work is summed across all categories with checked uint64 addition. Scalar-byte costs are counted even when no graph expansion occurs. Memoization avoids re-comparing the same exact validated node pair, not output-byte charging. Arithmetic/allocation overflow is ResourceLimit. This unit definition avoids either free unbounded byte work or a byte-per-step rule that would unexpectedly make ordinary full-scope positives impossible under4M steps.

## Quantitative feasibility, not an executed benchmark

At baseline the43 registered source rows total376255 raw source bytes (row-counted, some paths shared); specifications.json25996 and rules.json236501 bytes. Sum638752 bytes; base64 lengths sum to approximately851672 plus record overhead before adding contexts/mappings. These are measured *old file sizes*, not a promised new owner size or a complete Scope. Exact future RM/PP2/OS bytes do not exist yet, so no future golden byte length is fabricated.

Let S=actual canonical Scope bytes; F=facts; P=prerequisites; O=distinct obligations; V=reachable typed nodes; A=assertion nodes; N=source occurrences; R=refs/observations; L=sum local fact/link payload bytes. Standalone strings each include S. The full view also repeats Scope in typed obligations and scoped prerequisite keys: a conservative output estimate is
`B <= fixed + S + O*(S+Ilocal) + escape(O*(S+Ilocal)) + scopedKeyCopies*S + L`,
where `escape(x)<=2*x` for canonical ACE1 text embedded as String (base64 data generally adds no extra quote escapes). Count the actual encoder's dry-run, not this loose upper bound, for admission. Worst conservative factor for typed identity+its string is3*S per obligation; P keys can add substantially more. No deduplicated document claims its local IDs are standalone strings.

| Worked scenario | Scope / counts | Conservative consequence |
| --- | --- | --- |
| Small static-positive, hypothetical | S=1.25MiB,F=20,P=0,O=6,V=2000,A=1,N=1,R=100 | ~23.75MiB from scope terms(1+3*6)S plus local payload, plausibly below128MiB; must measure actual |
| Moderate causes, hypothetical | S=1.25MiB,F=120,P=0,O=64,V=10000,A=8,N=64,R=2000 | ~241.25MiB scope upper estimate; exact dry-run may exceed128MiB and must return ResourceLimit |
| Near transport input ceiling | S=32MiB,O=6 before keys/escaping | typed+string scopes alone can exceed128MiB; earlier D0 acceptance does not guarantee report acceptance |
| Identity-count ceiling | O=65536,S=1MiB even ignoring typed duplicate | >=64GiB just standalone Scope strings: byte gate binds far before identity count |
| Large shared assertion DAG | A<=32 Portable /<=1024 other profile,V bounded | Equal value nodes intern; does not erase per-origin facts or the repeated output Scope cost |

Comparison is bottom-up DAG validation plus typed set sort: approximately O(V+edges+sort comparisons+unique compared node pairs+output/64+scalar limbs), explicitly bounded by work. No recursive unfolding into exponential strings. Source/ref/Fact counts have separate counters; don't equate them with obligations. An implementation must benchmark actual old-positive analogues and report newly resource-rejected cases at T7. The plan promises fail-closed soundness, not that all historically accepted inputs remain available.

Dry-run validates complete roster, computes exact canonical output length and work before committing output or allocating a private capability. Streaming to a private bounded sink is allowed; on any error discard it. No truncated successful prefix, empty substitute, or `projection_depth_limit:true` sentinel can inhabit CompleteView. Changing descriptor/budgets is reviewed/versioned, never a reason to drop Scope fields.
