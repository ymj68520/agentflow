# Phase 4B.0B — Distinguishing Design Vectors

> **DESIGN PROPOSAL** · **NON-NORMATIVE** · **NOT A FROZEN ABI**

These 32 vectors specify required future outcomes; they are not production
conformance cases and were not run through a ComponentContract checker. Each
verdict is conditional on the named exact, immutable inputs. A static `Proven`
does not assert attestation, admission, runtime enforcement, or authorization.

## P1-1 — Identity, version, digest

| ID | Input | Relevant decision | Expected static verdict | Required proof | Reason |
| --- | --- | --- | --- | --- | --- |
| V01 | Same lineage and semantic body published as versions 1.2.0 and 1.3.0 | B0B-D1 | Proven: same canonical bytes/digest | Canonical body excludes release version; exact semantic fields equal | A release label is not semantic content |
| V02 | Same lineage, new semantic ε field, and a correctly recomputed new digest; no compatibility relation supplied | B0B-D1 | Unproven for substitution | Compare the changed dimensions under their frozen directions | A new content digest identifies change; it does not prove compatibility |
| V03 | Two different component lineages have otherwise identical fields | B0B-D1 | Unproven for substitution; distinct digests | Compare identity field and require explicit cross-lineage mapping | Identity participates in canonical bytes |
| V04 | Same ComponentRef, implementation artifact digest changes | B0B-D1 | Contract identity Proven; implementation RequiresEvidence | Verify same contract pin, then separately check implementation compatibility/attestation | Implementation bytes do not alter the promise |

## P1-2 — I/O port ABI and type compatibility

| ID | Input | Relevant decision | Expected static verdict | Required proof | Reason |
| --- | --- | --- | --- | --- | --- |
| V05 | New and old have the same I/O port sets and exact TypeSchemaRefs | B0B-D2 | Proven | Resolve all pins and compare by direction/port | Exact closed ABI is the v1 general proof |
| V06 | New adds a required input port absent from old | B0B-D2 | Violation | Compare closed input sets | Old caller cannot provide the new required value |
| V07 | New removes an output required by old | B0B-D2 | Violation | Compare closed output sets | New cannot supply the old result |
| V08 | Two documents use local SemanticTypeId 1, but it resolves to different exact TypeSchemaRefs at the same public port | B0B-D2 | Unproven | Ignore local IDs; resolve and compare both full pins | Local IDs are document-scoped |

## P1-3 — Alias policy

| ID | Input | Relevant decision | Expected static verdict | Required proof | Reason |
| --- | --- | --- | --- | --- | --- |
| V09 | Both public ports pin the same alias and exact schema digest | B0B-D3 | Proven | Exact TypeSchemaRef equality and schema resolution | Same nominal pin |
| V10 | Alias A and alias B both expand to `string` but have different pins | B0B-D3 | Unproven | Preserve public pins; structural expansion is not compatibility | Equal targets do not erase nominal identity |
| V11 | Security alias and underlying primitive have equal value representation | B0B-D3 | Unproven | Compare exact public pins; never normalize the alias away | Prevents a nominal security object being treated as a primitive |
| V12 | Referenced alias graph contains a detected cycle | B0B-D3 | Violation | Validate the pinned schema graph and reject the cycle | Cyclic type identity cannot be normalized |

## P1-4 — ε↔R operation/resource relation

| ID | Input | Relevant decision | Expected static verdict | Required proof | Reason |
| --- | --- | --- | --- | --- | --- |
| V13 | Exact operation/type pair is covered; selector keys, values, qualifiers, and R predicates agree | B0B-D4 | Proven | Resolve exact OperationRef/TypeSchemaRef and check all three independent predicates | Catalog support is explicit, not inferred |
| V14 | Exhaustive catalog covers the operation but has no rule for this ResourceTypeRef | B0B-D4 | Violation | Check `complete_operations` and exact pair table | Closed-world catalog explicitly excludes the pair |
| V15 | Operation is outside the snapshot's exhaustive coverage | B0B-D4 | Unproven | Require an exact catalog snapshot covering that operation | Missing row in a partial catalog is not a denial or proof |
| V16 | R port exists, but selector uses an unsupported key/value or contradicts an exact R attribute | B0B-D4 | Violation | Map selector key to exact resource attribute and evaluate the conjunction | Port existence alone is insufficient |

## P1-5 — ε↔F selector-grain identity

| ID | Input | Relevant decision | Expected static verdict | Required proof | Reason |
| --- | --- | --- | --- | --- | --- |
| V17 | F has operation-level `data.write`; exact partition allows only atom A on R1; ε is closed with A and B on R1/R2 | B0B-D5 | Proven for the declared refinement | Verify parent coarse sink, exact full AtomKey membership, and closed-set subset | The new binding only narrows the old ceiling |
| V18 | Same operation has R1 and R2 atoms; only the legacy operation sink exists, but author claims “R1 only” | B0B-D5 | Unproven for the claimed selector-specific policy | Require a complete exact EffectFlowPartition | Legacy sink literally covers both atoms |
| V19 | Exact Flow partition includes an EffectAtom absent from ε | B0B-D5 | Violation | Match the complete atom against the closed ε row | A dangling sink is not a declaration of an available effect |
| V20 | ε is Open/Unknown or F summary has `influence_unknown` while a positive selector-specific flow proof is requested | B0B-D5 | Unproven | Prove exhaustive atom and influence coverage | Unknown is never repaired to empty |

## P1-6 — Recovery action identity

| ID | Input | Relevant decision | Expected static verdict | Required proof | Reason |
| --- | --- | --- | --- | --- | --- |
| V21 | K requires reconciliation; binding names forward A, kind reconciliation, exact action B; A and B are both in closed ε | B0B-D6 | Proven for action coverage | Exact `RecoveryActionIdentity` lookup and exact ε membership | The action has a complete identity |
| V22 | Compensation operation name matches but its selector/qualifier differs from the declared action EffectAtom | B0B-D6 | Violation | Compare full `action_effect` AtomKey and old operation field | Operation-only matching is insufficient |
| V23 | K requires reconciliation but the bound action EffectAtom is absent from closed ε | B0B-D6 | Violation | Exact action-to-ε check | Recovery action itself is an Effect |
| V24 | Exact RecoveryAction bindings form a cycle A→B→A | B0B-D6 | Violation | Build the exact action graph and run cycle detection | No finite recovery closure is established |

## P1-7 — K↔U action usage coverage

| ID | Input | Relevant decision | Expected static verdict | Required proof | Reason |
| --- | --- | --- | --- | --- | --- |
| V25 | Every action has one exact profile, coverage catalog lists meters M/N, profile explicitly declares M finite(3), N finite(0), and aggregate fits U.recovery | B0B-D7 | Proven under the named catalogs | Exact action coverage, all required meters explicit, checked multiplicity and ceiling comparison | Explicit zero is distinct from omission |
| V26 | Action profile lists meter M but coverage catalog requires M and N | B0B-D7 | Unproven | Compare profile coordinates with exhaustive coverage set | One meter does not cover every action meter |
| V27 | Profile derives positive M use while U explicitly declares M finite(0) | B0B-D7 | Violation | Multiply per-attempt use by activation/attempt bounds, then compare | Explicit ceiling is too small |
| V28 | Retry attempts are unbounded and all required U recovery coordinates are explicitly unbounded | B0B-D7 | Proven as an unbounded declaration, not a finite budget proof | Preserve unbounded through multiply/accumulate and compare exact coordinates | No implicit free or finite usage is claimed |
| V33 | U contains the same meter name as an action profile, but the action has no exact RecoveryUsageBinding/coverage row | B0B-D7 | Unproven | Join by full RecoveryActionIdentity and require its complete profile | A matching meter coordinate is not action-level coverage |

## P1-8 — F↔X relation

| ID | Input | Relevant decision | Expected static verdict | Required proof | Reason |
| --- | --- | --- | --- | --- | --- |
| V29 | Exact label rule allows every pair in closed X execution/residency sets; taint/authority trust and sandbox rules match | B0B-D8 | Proven for static F/X consistency | Exact relation snapshot, exact identities, full allowed-pair subset and sandbox comparison | No string inference is used |
| V30 | X permits an execution/residency pair excluded by the exact confidentiality rule | B0B-D8 | Violation | Check every pair in X's allowed Cartesian product | One unsafe permitted placement invalidates consistency |
| V31 | Label or taint has no rule in the supplied relation snapshot | B0B-D8 | Unproven | Resolve each nominal F identity in the exact catalog snapshot | Missing mapping cannot be inferred from geography or names |
| V32 | Static F/X relation and descriptor match, but worker attestation is absent | B0B-D8 | Static relation Proven; runtime/attestation RequiresEvidence | Preserve separate descriptor, attestation, and authorization reports | Static match is not trusted execution evidence |
| V34 | Closed ε has no `data.write` atom, while F declares an operation-level `data.write` sink | B0B-D5 | Violation | Resolve every declared external-effect sink against the closed ε operation set | A dangling sink is not a usable allowance |
| V35 | Φ is the closed empty FailureRow; no claim is supplied about timeouts, cancellation, incidents, or successful execution | Proof API boundary | Proven only for the Φ-empty declaration; success remains undecided | Keep FailureRow verdict separate from control/runtime outcomes and other dimensions | Empty Φ is not a guarantee of success |
| V36 | All dimensions except one cross-dimension relation are Proven; that relation is Unproven, but a proposed aggregate says “compatible” | Proof API boundary | Overall static compatibility Unproven | Aggregate per-dimension verdicts without a pass-default; retain the failing relation | A single Boolean must not upgrade an Unproven dimension |
| V37 | New semantic body is paired with a digest computed for the old body | B0B-D1 | Violation | Recompute canonical body identity and compare the exact pinned digest | Digest/body mismatch is closed invalidity |

## Coverage summary

There are four vectors per P1, the explicit action-meter-only counterexample,
three cross-dimension aggregation/boundary vectors and one digest-integrity
vector (37 total). The set contains positive,
negative, Unproven, and RequiresEvidence outcomes. The vectors intentionally
do not exercise a production checker: no such checker is implemented in Phase
4B.0B.
