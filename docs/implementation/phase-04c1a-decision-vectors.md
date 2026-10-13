# Phase 4C.1A — Independent Design Discrimination Vectors

All 60 vectors are **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED ABI /
NOT IMPLEMENTED**. Every Actual Status is **Not Executed**. No portable
production Decoder exists, and no row is reported Passed.

## Exact symbolic inputs

B0 is the complete candidate projection of the current positive
`computation-root-export/positive/exact-input-output` formal fixture, with all
actual context scopes, current full registries/source/table bytes, ten role
entries (eight defined static roles, two ModelMissing) and Profile A.
E0 additionally uses the existing `root-export-soundness/f01/a-full-same-row`
legal effect input: its checked source/export/mappings provide a full witness.
S0 is the exact supported static semantic tuple. These are symbolic **typed
input states**, not already encoded/executed portable fixtures.

I0/C0/schema pins denote their existing HashV1 expressions over exact canonical
objects. No fabricated hex is assigned. ACE(X) denotes the unique candidate
encoding specified in this package. Every row inherits its stated base plus
the explicit typed mutation; remaining prerequisites are unchanged.

Identity notation: S = expanded semantic projection; E = core Bundle evidence
bytes; V = formal static verdict. Equal roundtrip means S/E retained; rejected
bytes have no validated projection. Submission diagnostics/origin claims are
outside E.

Future APIs are conceptual only: F-DECODE (D0–D4), F-RECONSTRUCT (D5),
F-REVALIDATE (D6–D10 using existing validators/checker), F-SUPPORT (local
semantics selection), F-ORIGIN (future B05/B07). Dependencies B01-I/B02-I
mean the still-unimplemented corresponding schema/Decoder/support work.

## Portable Schema / Decoder — 12

| CaseId | Exact typed input | Canonical representation | Expected Decode stage | Expected revalidation stage | Expected verdict / failure | Expected identity relation | Normative basis | Future API | Unimplemented dependency | Actual Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| B01-09 | B0 with unknown EffectGuard case | Unknown single-case object | D0 rejects | None | UnknownVariant; no capability | No accepted S/E | ACE1 closed variants | F-DECODE | B01-I | Not Executed |
| B01-10 | Corrupt one outer ACE escape / trailing byte | Not ACE(B0) | D0/D1 rejects | None | NoncanonicalEncoding | E rejected | ACE1 exact byte grammar | F-DECODE | B01-I | Not Executed |
| B01-11 | Int64 9007199254740993 and decimal 12345678901234567890.1234567890123456789 converted through double | Numeric tokens or rounded asset under old pin | D0 InvalidPrimitive or D3 mismatch | None | InvalidPrimitive/ContentIdentityMismatch; author precision lost | Original values not retained | ACE1 numeric strings/exact Decimal | F-DECODE | B01-I | Not Executed |
| SC-01 | Omit required Option field | Missing record member | D0 rejects | None | MissingField, not None | No accepted S/E | ACE1 required records | F-DECODE | B01-I | Not Executed |
| SC-02 | Encode Some(empty) as bare null | Untagged null | D0 rejects | None | InvalidOption | No accepted S/E | ACE1 option tagging | F-DECODE | B01-I | Not Executed |
| SC-03 | Repeat a record/map key | Duplicate JSON member before map insertion | D0 rejects | None | DuplicateField | No last-write-wins value | ACE1 duplicate rejection | F-DECODE | B01-I | Not Executed |
| SC-04 | Add trusted_context=true to core | Extra boolean member | D0 rejects | None | UnknownField | No accepted S/E | Closed Bundle schema | F-DECODE | B01-I | Not Executed |
| SC-05 | Put IR pin in a ContractPin field | Valid scalar, wrong domain | D2 rejects | None | WrongDigestDomain | Typed identity unequal | HashV1 nominal domains | F-DECODE | B01-I | Not Executed |
| SC-06 | Reverse a declared sorted target/set list | Canonical lexical scalars, noncanonical list | D1 rejects | None | NoncanonicalOrder | E rejected, not auto-sorted | Candidate set order | F-DECODE | B01-I | Not Executed |
| SC-07 | uint32 ID greater than 4294967295 | Exact decimal string outside width | D0 rejects | None | NumericRange | No narrowing | AFIDL widths / ACE1 | F-DECODE | B01-I | Not Executed |
| SC-08 | SchemaRef points to an absent schema asset / cyclic structural table | Canonical records, bad references | D2 rejects | None | MissingReferencedContent/ReferenceCycle | No expanded S | AFIDL / typed DAG rules | F-DECODE | B01-I | Not Executed |
| SC-09 | Non-NFC/invalid UTF-8 text or NFC-colliding authored map keys | Noncanonical Text / duplicate normalized author keys | D0/D1 or encoder rejects | None | InvalidUnicode/DuplicateField | No accepted canonical identity | ACE1 NFC/UTF-8/key uniqueness | F-DECODE | B01-I | Not Executed |

## Proof Context Roundtrip — 12

| CaseId | Exact typed input | Canonical representation | Expected Decode stage | Expected revalidation stage | Expected verdict / failure | Expected identity relation | Normative basis | Future API | Unimplemented dependency | Actual Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| B01-01 | Complete E0 context rebuilt in new process | ACE(E0) | D0–D4 accept | D5–D10 | Proven static; trust separate | Roundtrip S/E=; new pointers | C01; AF-SEMANTIC-0160 | F-RECONSTRUCT | B01-I | Not Executed |
| B01-02 | Remove E0's used write catalog member | ACE(new complete set) | D0–D4 accept | D5 then D6 stops | ContractUnproven/unknown operation | S/E differ; old proof invalid | Effect resolution / context closure | F-REVALIDATE | B01-I | Not Executed |
| B01-03 | Same catalog label, add unused declared operation | ACE(new complete set) | D0–D4 accept | D5–D10 | Fresh static may remain Proven; no old reuse | S/E differ, V may equal | Exact content binding | F-REVALIDATE | B01-I | Not Executed |
| B01-04 | Target dependency None versus Some([]) | Tagged none versus some empty | Both decode | D8 unknown stops; known empty can continue | Unproven versus eligible fresh check | S/E unequal | Option semantics / F02 | F-RECONSTRUCT | B01-I | Not Executed |
| B01-07 | Alter relation bytes but keep old checksum | Changed canonical typed blob | D3 rejects | None | ContentIdentityMismatch | Invalid claimed identity | Relation content checksum | F-DECODE | B01-I | Not Executed |
| B01-08 | Resource descriptor selects S1, its own schema scope selects S0 of same name | Full distinct SchemaRefs | D0–D4 accept | D5 rejects | InvalidNominalReference | Pins unequal, label insufficient | Resource nominal recognition | F-RECONSTRUCT | B01-I | Not Executed |
| B01-13 | Context schema set differs from resource private schema set | Two independent complete memberships | D0–D4 accept | D5–D10 | Original valid static result retained | Exact independent scopes roundtrip | Actual ResourceTypeUniverse | F-RECONSTRUCT | B01-I | Not Executed |
| B01-14 | Recovery effects contain additional unused operation | Separate effect-catalog lists | D0–D4 accept | D5–D10 | Fresh static relation retained if needed members valid | Recovery/context sets not collapsed | Actual RecoveryOperationUniverse | F-RECONSTRUCT | B01-I | Not Executed |
| B01-15 | exact_version=v2, supported={v1,v2} | Explicit v2 plus sorted set | D0–D4 accept | D5–D10 | Register v2 first; same semantic projection | exact_version retained | Actual capability constructor | F-RECONSTRUCT | B01-I | Not Executed |
| B01-16 | Directed region compatibility A→B only | Exact pair set | D0–D4 accept | D5–D10 | No inferred B→A | S roundtrip exact | Execution relation data | F-RECONSTRUCT | B01-I | Not Executed |
| B01-17 | Two module imports cycle, named type graph acyclic | Exact canonical schema assets | D0–D4 accept | D5 compiles | Supported module graph; normal static check | Schema pins retained | AF-AFIDL cross-module acyclicity | F-RECONSTRUCT | B01-I | Not Executed |
| B01-18 | Flow label collection Unavailable, not Complete([]) | Known unavailable case | D0–D4 accept | D5 stops | IncompleteContext | Unknown not empty | Closed context field rules | F-RECONSTRUCT | B01-I | Not Executed |

## Resolver Associations — 10

| CaseId | Exact typed input | Canonical representation | Expected Decode stage | Expected revalidation stage | Expected verdict / failure | Expected identity relation | Normative basis | Future API | Unimplemented dependency | Actual Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| B01-05 | Repeat an identical target row | Duplicate canonical row list | D1 rejects | None | DuplicateSetMember | No accepted association | F02 / manifest uniqueness | F-DECODE | B01-I | Not Executed |
| B01-06 | Same export digest set, swap A→E1/B→E2 to A→E2/B→E1 coherently | Full pair rows differ | D0–D4 accept | D8 rederive then D10 | Old proof unusable; fresh verdict depends on new envelopes | S/E differ despite equal sets | F02; AF-SEMANTIC-0171 | F-REVALIDATE | B01-I | Not Executed |
| RS-01 | Some target content points at absent asset | Dangling typed content ref | D2 rejects | None | MissingReferencedContent | No complete S | Inline asset closure | F-DECODE | B01-I | Not Executed |
| RS-02 | Same target with two different export pins | Conflicting pair rows | D2 rejects | None | ConflictingAssociation | No accepted identity | F02 target association | F-DECODE | B01-I | Not Executed |
| RS-03 | Declared direct edges omit actual target call | Known list differs from checked IR | D0–D4 accept | D8 rejects | DependencyAssociationMismatch | Derived/input graphs unequal | AF-SEMANTIC-0171 | F-RECONSTRUCT | B01-I | Not Executed |
| RS-04 | Complete assets/inventory, omitted reachable transitive closure member | Known incomplete closure | D0–D4 accept | D8 rejects | DependencyAssociationMismatch | No complete derived identity | Subflow closure | F-RECONSTRUCT | B01-I | Not Executed |
| RS-05 | Declared target rows A→B/B→A over complete independently acyclic checked assets | Sorted rows with cyclic declared edges | D0–D4 accept | D8 rejects declared cycle before edge comparison | UnsupportedDependencyCycle | No positive graph; no claim of legal hash-cycle IR | Current closure / B03 boundary | F-RECONSTRUCT | B01-I | Not Executed |
| RS-06 | Source INVOKE pin replaced by same-name contract | Valid new pin but caller still pins old | D0–D4 accept | D8 rejects | SourceAssociationMismatch | Exact source identity differs | AF-SEMANTIC-0118..0121 | F-RECONSTRUCT | B01-I | Not Executed |
| RS-07 | Two caller nodes use same source contract with valid summed envelope | Distinct caller/node rows | D0–D4 accept | D8–D10 | Distinct occurrences, valid fresh proof eligible | Source keys retained separately | Scoped occurrence identity | F-REVALIDATE | B01-I | Not Executed |
| RS-08 | AWAIT signal resolves but external influence remains unknown | await_signal + explicit unknown source data | D0–D4 accept | D7–D10 | Unproven where Flow relation insufficient | Unknown influence retained | Flow unknown provenance | F-REVALIDATE | B01-I | Not Executed |

## Compiler Semantics / Versions — 12

| CaseId | Exact typed input | Canonical representation | Expected Decode stage | Expected revalidation stage | Expected verdict / failure | Expected identity relation | Normative basis | Future API | Unimplemented dependency | Actual Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| B02-01 | Same RuleId, changed normative bytes under old registry pin | Source Bytes changed | D3 rejects | None | SourceContentMismatch | Claimed S0 invalid | Whole source binding / F03 | F-SUPPORT | B02-I | Not Executed |
| B02-02 | Coherently changed Registry/source rows with the same recognized semantics label, absent from local exact entries | New exact registry content | D0–D3 accept | D4 stops | ConflictingSemanticContent | Compiler identity differs | AF-SEMANTIC-0161 | F-SUPPORT | B02-I | Not Executed |
| B02-03 | Same semantics label, coherently changed sources and registry | Canonical new full tuple | D0–D3 accept | D4 stops absent exact entry | ConflictingSemanticContent | Label equality insufficient | Exact support rules | F-SUPPORT | B02-I | Not Executed |
| B02-04 | Two separately qualified conforming builds, exact same S0 | Core ACE(B0) unchanged; build claims differ | D0–D4 accept | D5–D10 fresh each | Equivalent static facts; no inferred binary correctness | S/E=, build identity differs | C04 / build separation | F-REVALIDATE | B02-I, B05 | Not Executed |
| B02-05 | Recognized historical tuple without implemented support | Exact old tuple | D0–D3 accept | D4 stops | UnsupportedHistoricalSemantics | No relabel to current | Version support | F-SUPPORT | B02-I | Not Executed |
| B02-06 | Exact historical tuple and explicit conforming supported path | Exact historical bytes | D0–D4 accept | D5–D10 fresh | Historical static result under exact old meaning | Historical identity retained | C01/C04 historical support | F-REVALIDATE | B02-I, B05 | Not Executed |
| B02-07 | Plan lowering role explicitly ModelMissing | Required role entry present with missing case | D0–D4 static profile accepts | D5–D10 static only | Static Proven possible; no Plan/Publish support | Missing role remains explicit | B02/B04 boundary | F-SUPPORT | B02-I, B04 | Not Executed |
| B02-08 | Runtime role explicitly ModelMissing | Required entry present | D0–D4 static profile accepts | D5–D10 static only | Static Proven possible; runtime NotImplemented | No runtime profile invented | B02/runtime boundary | F-SUPPORT | B02-I, B04 | Not Executed |
| B02-09 | Rule-complete profile supplied with only old 84-ID set | Known profile B shape | D0–D3 accept structurally | D4 stops | UnsupportedAttributionProfile/Unproven | F03 not discharged | F03 hard trigger | F-SUPPORT | B02-I, rule closure | Not Executed |
| B02-10 | Submitted diagnostic says Proven; root E=10 export E=8 | Core negative canonical input + separate text | D0–D4 accept | D10 | Violation; diagnostic ignored | Fresh V differs from old text | AF-SEMANTIC-0166 / C02 | F-REVALIDATE | B01-I | Not Executed |
| B02-11 | Closed domain table attempts a new unregistered proof domain | Valid JSON Bytes, unsupported table content | D3/D4 rejects | None | RegistryContentMismatch/UnsupportedSemantics | Not a valid supported tuple | Frozen HashV1 domains | F-SUPPORT | B02-I | Not Executed |
| B02-12 | Select portable_certificate profile | Known unit selector; no certificate payload | D0–D3 accept | D4 stops | UnsupportedAttributionProfile | No positive certificate identity | C01 / F03 trigger | F-SUPPORT | B02-I, certificate model | Not Executed |

## Content Origin / Trust — 6

| CaseId | Exact typed input | Canonical representation | Expected Decode stage | Expected revalidation stage | Expected verdict / failure | Expected identity relation | Normative basis | Future API | Unimplemented dependency | Actual Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| B01-12 | Valid E0, claimed catalog origin untrusted | Core ACE(E0), separate raw claim | D0–D4 accept | D5–D10 | Static Proven possible; origin RequiresEvidence | Core S/E unchanged | C05 content/origin separation | F-ORIGIN | B01-I, B05/B07 | Not Executed |
| OR-01 | No OriginClaim supplied | Empty claim set, complete core | D0–D4 accept | D5–D10 | Static result unchanged; origin NotEstablished | S/E= | Origin schema boundary | F-ORIGIN | B05/B07 | Not Executed |
| OR-02 | Valid content checksum with self-claimed issuer | Exact raw subject/checksum | D0–D4 accept | D5–D10 | Authentication not inferred | S/E=, submission differs | C05 trust roots absent | F-ORIGIN | B05/B07 | Not Executed |
| OR-03 | Same binary checksum, no qualification evidence | Binary identity claim only | Core decodes | Fresh checker still required | QualifiedVerifier not established | Content identity not qualification | B02/B05 boundary | F-ORIGIN | B05 | Not Executed |
| OR-04 | OriginClaim names a different context content checksum | Conflicting separate claim | Claim consistency rejects | Core may be checked independently | InvalidOriginSubject; no approval | Core E=, claim unequal | Typed exact subject rules | F-ORIGIN | B05/B07 | Not Executed |
| OR-05 | Build provenance payload hash valid, predicate semantics unavailable | Exact opaque evidence bytes | Claim shape/integrity accept | No positive trust stage | RequiresEvidence/UnsupportedPredicate | Byte identity valid, trust absent | Evidence predicate boundary | F-ORIGIN | B05/B07 | Not Executed |

## Cross-process Revalidation — 8

| CaseId | Exact typed input | Canonical representation | Expected Decode stage | Expected revalidation stage | Expected verdict / failure | Expected identity relation | Normative basis | Future API | Unimplemented dependency | Actual Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| CROSS-01 | Same E0 decoded in two processes | Same exact ACE(E0) | D0–D4 accept both | D5–D10 fresh each | Equivalent static Proven facts | S/E/V=, pointers differ | C01 full reconstruction | F-REVALIDATE | B01-I | Not Executed |
| CROSS-02 | Coherent new resolver B, old saved report/root A proposed for reuse | New Bundle + old separate diagnostic | Core accepts; saved Root field would reject | D8–D10 fresh B | Old proof invalid; fresh result only | Resolver S/E differ | F02 | F-REVALIDATE | B01-I | Not Executed |
| CROSS-03 | Valid context, source/export same port number but incorrect mapping owner | Canonical typed wrong witness | D0–D4 accepts if refs exist | D10 | Unproven/Violation by formal relation | Numeric coincidence not identity | Typed I/R/E mapping rules | F-REVALIDATE | B01-I | Not Executed |
| CROSS-04 | All static relations Proven, body evidence absent | Complete E0 without external body evidence | D0–D4 accept | D10 | Static Proven; body RequiresEvidence, Publish blocked | Static identity unchanged | C05/C06; AF-SEMANTIC-0171 | F-REVALIDATE | B01-I, B05–B07 | Not Executed |
| CROSS-05 | Checksums valid, origin unauthenticated in both processes | Same core, no trusted evidence | D0–D4 accept | D5–D10 | Static facts equal; no origin authorization | S/E/V=; trust missing | Content versus origin | F-ORIGIN | B05/B07 | Not Executed |
| CROSS-06-P30 | Portable original split guard/bounds/selector shape | Exact typed P30 IR/source/export/maps | D0–D4 accept | D10 | Epsilon/overall Unproven, no capability | Same full-row predicate | F01 / AF-SEMANTIC-0166 | F-REVALIDATE | B01-I | Not Executed |
| CROSS-06-P31 | Root derivation formerly A=1; coherent portable current B=10, top=1 | Exact B inputs; no saved Root field | D0–D4 accept coherent B | D8–D10 fresh B | Violation; no stale A acceptance | B association bound throughout | F02 / AF-SEMANTIC-0160 | F-REVALIDATE | B01-I | Not Executed |
| CROSS-07 | Diagnostics/spans/format differ while exact core and conforming semantics match | Same core, changed diagnostic envelope | D0–D4 accept | D5–D10 | Typed verdicts equal; diagnostics may differ | S/E/V=, diagnostic presentation differs | Equality separation / C02 | F-REVALIDATE | B01-I | Not Executed |

## Counting and future execution

12 + 12 + 10 + 12 + 6 + 8 = **60 design vectors**. CROSS-06-P30 and
CROSS-06-P31 are separate explicit portable reconstruction designs; they do
not reuse historical executions as new results.

Each future implementation must independently author expected typed outcomes,
materialize full canonical inputs and run actual stage APIs. Historical 311
formal CaseIds remain a separate regression requirement. No row above
increments production pass totals or closes implementation readiness.
