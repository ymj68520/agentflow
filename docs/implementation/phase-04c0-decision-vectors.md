# Phase 4C.0 — Publication Design Discrimination Vectors

All 48 vectors are **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED ABI /
NOT IMPLEMENTED**. Actual execution status for every vector: **Not Executed**.
No Publish API exists, and no production pass count is assigned. These are
independent decision scenarios, not additions to the 311-case corpus or the
historical 62 audit probes.

## Exact symbolic inputs and status notation

T0 denotes a complete hypothetical future valid candidate:

- I0: exact existing SemanticIrDigest and canonical IR.
- C0: exact existing exported ComponentContractDigest and checked contract.
- A0: complete typed RootExport assertion input tuple.
- S0: full exact static/lowering semantic source and registry content.
- X0: complete portable context content, approved origins and exact relations.
- D0: exact source/call-site/target/export/direct/transitive associations.
- LG0 / PC0: valid future LockGraph and PlanCore, with canonical schemas and
  typed pins in the already registered lock-graph / plan-core domains.
- RT0: exact future runtime semantics profile; B0: qualified checker build.
- E0: every mandatory body/fixed-implementation/build evidence subject,
  predicate, issuer and dependency validated under policy.
- V0: future canonical semantic parent and its exact artifact-domain pin.
- P0 / RP0: authenticated publisher and authoritative release policy allowing
  the exact publication namespace, action and required profile.

T0 never claims these models currently exist. Each row inherits T0 except the
stated mutation, so all required exact identities are explicit. “Static” is
the fresh RootExport declaration relation; it does not include Plan, body or
publisher claims. A source-stage rejection with no mintable checked input is
written Unproven/no checker input.

“Admit” is hypothetical eligibility under the selected future gate. Current
production eligibility is NotImplemented for all rows. “Reject” and “Block”
issue no publication capability; every reason remains typed and retained.

Future conceptual entry labels, all unimplemented:
F-REVALIDATE (portable reconstruction/fresh current static APIs),
F-LOCK (LockGraph verifier), F-PLAN (Plan preservation),
F-TRUST (evidence authentication/conformance), F-PUBLISH (release gate),
F-DEPLOY and F-RUN (later boundaries).

## Static proof revalidation — 8

| CaseId | Formal input state | Required exact identities | Expected static status | Expected Publish eligibility | Missing evidence | Rule / decision | Future entry | Failure-closed result |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| SP-01 | Rebuild all fresh checked inputs from T0 | T0, exact local X0 reconstruction | Proven | Admit | None in hypothetical T0 | AF-SEMANTIC-0159..0171; C01 | F-REVALIDATE → F-PUBLISH | Admit only complete conjunction |
| SP-02 | Supply a saved checked-wrapper address/report as proof | T0 identities claimed, checked inputs absent | Unproven | Block | Fresh checked inputs and derivation | C01; D3 | F-REVALIDATE | MissingProofInputs; no pointer restoration |
| SP-03 | Producer report says Proven; root E upper 10, export upper 8 | I1/C1/A1/X0/D0 exact, fresh rederive | Violation | Reject | None; claimed result false | AF-SEMANTIC-0166; F01 | F-REVALIDATE | KnownEffectViolation retained |
| SP-04 | Delete a required effect/resource coordinate witness | T0 except A1 incomplete | Unproven | Block | Typed mapping closure | AF-SEMANTIC-0164/0166; C02 | F-REVALIDATE | UnprovenStaticRelation |
| SP-05 | Assertion pins a different caller IR | I1 bytes, stale I0 in A0 | Violation | Reject | Exact assertion agreement | AF-SEMANTIC-0159; C01 | F-REVALIDATE | ExactPinMismatch |
| SP-06 | Open/unknown effect row offered as complete contract | C1 open, I0/X0 exact | Unproven; no checker input | Block | Complete checked source contract | Frozen Effect completeness; C01 | F-REVALIDATE | Earlier contract rejection, never empty row |
| SP-07 | P30 split bounds/guard rows, no complete same-row witness | Exact scoped E rows in C1/A1/X1 | Unproven | Block | Complete containment witness | AF-SEMANTIC-0166; F01 | F-REVALIDATE | No positive cross-row synthesis |
| SP-08 | Root entries upper 6 and 6 compete for one export slot upper 8 | Exact occurrence identities and C1/A1 | Unproven | Block | One-use slot capacity | Frozen Effect containment; C01 | F-REVALIDATE | No reuse of consumed slot |

## Context / compiler invalidation — 8

| CaseId | Formal input state | Required exact identities | Expected static status | Expected Publish eligibility | Missing evidence | Rule / decision | Future entry | Failure-closed result |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| CC-01 | New process, same complete semantic/context content | T0; new local pointer, exact X0 projection | Proven | Admit | None | AF-SEMANTIC-0160; C01 | F-REVALIDATE | Fresh local capability only |
| CC-02 | Catalog label matches but full operation set absent | T0 except X1 incomplete | Unproven | Block | Complete catalog content | AF-SEMANTIC-0160; C01 | F-REVALIDATE | MissingArtifact/ContextContent |
| CC-03 | Same RuleId/version label, changed normative source bytes | S1 bytes versus S0 in A0/V0 | Unproven | Block | Current exact semantics | AF-SEMANTIC-0161; C04 | F-REVALIDATE | StaleEvidence |
| CC-04 | Change complete Rule Registry content without updating bundle | Registry S1 versus old S0 | Unproven | Block | Exact registry agreement | AF-SEMANTIC-0161; C04 | F-REVALIDATE | StaleEvidence, no ID-only equality |
| CC-05 | New qualified compiler binary supports exact same semantic tuple | T0 + approved B1; S0 unchanged | Proven after full recheck | Admit | None under RP0 allowing B1 | C01/C04 | F-REVALIDATE / F-TRUST | No automatic cached proof reuse |
| CC-06 | Lowering semantics changed, old PC0 supplied | Static S0, lowering S1/PC0 mismatch | Proven for static subset | Block | Supported matching Plan preservation | C03/C04 | F-PLAN / F-PUBLISH | UnsupportedSemantics/PlanEvidence |
| CC-07 | Archived checker proves old S0; current release policy forbids S0 | Exact historical T0, policy RP1 | Proven under archived S0 | Block | Currently allowed semantics profile | C04/C06 | F-PUBLISH | UnsupportedSemantics, immutable old version retained |
| CC-08 | RP1 demands complete per-rule attribution; only 84 IDs supplied | T0 + rule-complete RP1 | Proven for static subset | Block | Complete rule dependency closure | F03; C04 | F-PUBLISH | UnprovenAttribution, no waiver |

## Resolver / subflow associations — 8

| CaseId | Formal input state | Required exact identities | Expected static status | Expected Publish eligibility | Missing evidence | Rule / decision | Future entry | Failure-closed result |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| RS-01 | Swap two target/export associations with same export digest set | D1 pairs differ from A0/X0/LG0 | Unproven | Block | Source association agreement | AF-SEMANTIC-0160/0171; F02 | F-REVALIDATE / F-LOCK | StaleEvidence, set equality insufficient |
| RS-02 | Fresh current target export upper 10, top export upper 1 | I0/C0, D1/A1/X1 exact and fresh Root | Violation | Reject | None; actual ceiling too narrow | AF-SEMANTIC-0166; C01 | F-REVALIDATE | KnownEffectViolation |
| RS-03 | Direct target exists; one transitive target asset missing | D0 closure and LG0, missing child IR bytes | Unproven | Block | Transitive semantic dependency | AF-SEMANTIC-0171; C03 | F-LOCK / F-REVALIDATE | MissingArtifact; no truncated closure |
| RS-04 | Target graph includes a dependency cycle | Exact cyclic D1/LG1 | Unproven | Block | Acyclic supported dependency closure | AF-SEMANTIC-0171; C03 | F-LOCK | Unsupported/incomplete closure |
| RS-05 | Dependency availability Unknown is encoded as empty | Claimed D1 versus unavailable target facts | Unproven | Block | Known direct edge set | C01/C03 | F-LOCK / F-REVALIDATE | Unknown cannot become Known(empty) |
| RS-06 | Same target is associated with two different exports | D1 includes conflicting exact pairs | Unproven; no checker input | Reject | Valid unique association | F02; C03 | F-LOCK / F-REVALIDATE | DuplicateAssociation contradiction |
| RS-07 | Reorder resolver records but preserve all exact associations | T0 after specified canonical ordering | Proven | Admit | None | C01/C03; F02 | F-REVALIDATE | Order-invariant content, duplicates still rejected |
| RS-08 | Same caller IR requires component Cchild0; resolver replaces it with Cchild1 | I0 exact pin and D1 lacking Cchild0 | Unproven | Block | Exact pinned leaf content | AF-SEMANTIC-0118..0121; C03 | F-REVALIDATE | No compatible-name/latest fallback |

## Artifact identity — 6

| CaseId | Formal input state | Required exact identities | Expected static status | Expected Publish eligibility | Missing evidence | Rule / decision | Future entry | Failure-closed result |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| AI-01 | Same semantic core, different UI coordinates/title | T0; presentation attachment changes only | Proven | Admit same semantic content | None | Architecture canonicalization; C02/C07 | F-PUBLISH | Presentation excluded from semantic identity |
| AI-02 | Same logical version name, retry constant changes | New I1/A1/PC1/V1, old logical reference fixed to V0 | Proven if rederived for I1 | Reject conflicting reference | New immutable logical version reference | C02/C07 | F-PUBLISH | Never retarget old version |
| AI-03 | Claimed artifact pin unchanged but canonical parent bytes altered | Claimed V0 versus different core bytes | Proven on separately intact I0/C0 | Reject | Valid parent integrity | HashV1/ACE1; C02 | F-PUBLISH | ContentIdentityMismatch |
| AI-04 | SemanticIrDigest is supplied as PlanCore pin | I0 used in wrong PC field/domain | Proven for RootExport | Reject | Valid typed PlanCore asset | C02/C03 | F-PLAN / F-PUBLISH | WrongDigestDomain |
| AI-05 | RootExportAssertionIdentity is supplied as version content identity | A0 tuple, missing parent schema/V0 | Proven for RootExport | Block | Canonical parent identity/model | D3; C02 | F-PUBLISH | NotImplementedArtifactIdentity |
| AI-06 | Bundle's top export pin differs from inline assertion export pin | C0 versus C1 redundant fields | Unproven; reconstruction rejected | Reject | Consistent exact input tuple | C01/C02 | F-REVALIDATE | DuplicateFieldIdentityConflict |

## Body / publisher evidence — 6

| CaseId | Formal input state | Required exact identities | Expected static status | Expected Publish eligibility | Missing evidence | Rule / decision | Future entry | Failure-closed result |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| BE-01 | RootExport valid, required target body attestation absent | I0/C0/D0, required Ebody0 unavailable | Proven; body RequiresEvidence | Block | Mandatory body predicate | AF-SEMANTIC-0171; C05 | F-TRUST / F-PUBLISH | RequiresEvidence |
| BE-02 | Publisher signature authentic, body proof invalid | P0/V0 valid; Ebody1 invalid subject/predicate | Proven for declarations | Reject | Sound body conformance | C05/C06 | F-TRUST | KnownEvidenceViolation |
| BE-03 | Body evidence references old export after target association change | Current exact D1, stale evidence subject Ebody0 | Proven after current recheck if ceiling fits | Block | Current exact body evidence | C03/C05 | F-TRUST | StaleEvidence |
| BE-04 | Caller chooses a self-authored catalog; checker content internally exact | I0/C0/X1/D1 exact; RP0 disallows its origin | Proven for that context | Reject | Approved context origin | C01/C05 | F-TRUST / F-PUBLISH | UntrustedEvidenceOrigin |
| BE-05 | Fixed implementation binary changes, contract label unchanged | Same C0, different exact binary artifact, old evidence | Proven for declarations | Block | Exact implementation predicate | C05 | F-TRUST | StaleEvidence; contract is not binary proof |
| BE-06 | Publisher declaration unsigned / unauthenticated | V0 exact, P0 authentication absent | Proven | Block | Publisher authentication and authorization | C05/C06 | F-TRUST / F-PUBLISH | UntrustedPublisher; no content-hash promotion |

## Plan / LockGraph prerequisites — 6

| CaseId | Formal input state | Required exact identities | Expected static status | Expected Publish eligibility | Missing evidence | Rule / decision | Future entry | Failure-closed result |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PL-01 | Valid RootExport, PlanCore missing | I0/C0/A0/D0/LG0, PC unavailable | Proven | Block | PlanCore model/content and preservation | C03/C06 | F-PLAN / F-PUBLISH | MissingArtifact |
| PL-02 | Valid RootExport, LockGraph missing | I0/C0/A0/D0, LG unavailable | Proven in process | Block | Durable semantic closure | C03/C06 | F-LOCK / F-PUBLISH | MissingArtifact |
| PL-03 | Lowering adds a declared effect possibility absent in IR | I0/C0/LG0, altered PC1 | Proven for IR declarations | Reject | Semantic-preserving Plan | C03 | F-PLAN | PlanEffectPreservationViolation |
| PL-04 | Lowering drops a required Recovery/cancellation obligation | I0/C0/LG0, altered PC1 K/termination | Proven for IR declarations | Reject | Recovery/control preservation | C03 | F-PLAN | PlanSemanticPreservationViolation |
| PL-05 | Plan consumes semantic tool/prompt/adapter content omitted from lock | PC1 refers asset Z, LG0 omits Z | Proven for declared contracts | Block | Complete reachable semantic edge | C03/C07 | F-LOCK / F-PLAN | UnprovenLockClosure |
| PL-06 | Plan generated from LG0 is packaged with LG1 | PC0 source tuple differs from parent LG1 | Proven for rechecked IR relation | Reject | Exact lowering input association | C03/C04 | F-PLAN / F-PUBLISH | PlanSourceIdentityMismatch |

## Publish / Deploy / Run separation — 6

| CaseId | Formal input state | Required exact identities | Expected static status | Expected Publish eligibility | Missing evidence | Rule / decision | Future entry | Failure-closed result |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PD-01 | All mandatory Publish predicates met; no deployment environment | T0; no EnvironmentContract/BindingSnapshot yet | Proven | Admit WorkflowVersion | Deployment inputs | C06 | F-PUBLISH then F-DEPLOY | Publication succeeds hypothetically; deployment remains unavailable |
| PD-02 | T0 valid; concrete runtime authority unknown | T0; no Run principal/grant facts | Proven | Admit WorkflowVersion | Run admission/authority | C06 | F-RUN | No run/runtime authorization inferred |
| PD-03 | Compatible physical binding changes inside declared envelope | T0 plus different prospective BindingSnapshot | Proven | Existing version content unchanged | New deployment eligibility evaluation | C06/C07 | F-DEPLOY | New immutable revision; admitted Runs do not drift |
| PD-04 | Implementation reads current env/flag/default model to alter tool behavior | Declared T0, hidden behavior not committed or attested | Proven for declarations only | Reject semantic baseline violation | Explicit semantic input and valid body evidence | C05/C07 | F-TRUST / F-PUBLISH | Ambient semantics cannot be accepted |
| PD-05 | Published T0 valid; concrete principal/quota/policy denies Run | V0 and exact future run policy/principal facts | Proven | Existing Publish unaffected | Successful specific Run admission | C06 | F-RUN | Run rejected independently |
| PD-06 | Live safety overlay attempts to widen authority/effect envelope | V0 baseline, proposed widening overlay | Proven for declarations | Existing Publish unaffected | Valid narrowing-only overlay | C06/C07 | F-RUN / future enforcement | Reject widening; no added runtime permission |

## Counting and use

8 + 8 + 8 + 6 + 6 + 6 + 6 = **48 independent design vectors**. Every row
specifies formal state, exact identities, static expectation, publication
expectation, missing evidence, decision/rule, future entry and safe result.
None has an Actual production Publish outcome.

Future conformance must materialize these inputs after the relevant schemas
exist, author independent expected results and run actual production validators.
Until then, “Admit” rows demonstrate intended distinctions, not current
readiness. Valid declaration proofs plus missing Plan/Lock/body/identity facts
remain blocked; valid publication plus unavailable deployment remains a
different, intentionally supported boundary.
