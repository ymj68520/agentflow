# Phase 4B.2D — Independent Decision Vectors

These **62 design vectors** are **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED ABI / NOT IMPLEMENTED**. They are not registered conformance cases and do not alter any frozen Expected file.

## Vector contract

Every row names an exact input shape and identity namespace, DecisionId, expected semantic relation and static verdict, required proof premises, a counterexample or missing-evidence boundary, and a future production entry point. The source contract digest, SemanticIrDigest, computation node, full EffectAtom/RecoveryActionIdentity, and exact context are part of every applicable test identity even when abbreviated in the table.

## Guard relation vectors

| CaseId | Formal input shape and exact identities | Decision / expected relation / verdict | Preconditions; counterexample or missing evidence | Future production entry point |
| --- | --- | --- | --- | --- |
| G01 | Same source contract and K binding; Forward guard unconditional; exact action E guard unconditional | D1; equal possibility ceilings; Proven | Exact full action atom and positive finite upper; no counterexample | Component-local guard compatibility |
| G02 | Same source; K possibility input p1=0; action E guard unconditional | D1; conditional subset of unconditional; Proven | p1 exists and literal is valid; no missing premise | Frozen guard containment |
| G03 | Same source; K unconditional; action E input p1=0 | D1; possible set exceeds E; Violation with exact typed alternate p1 value | Exact non-singleton primitive domain and reachable alternate value; absent domain witness means Unproven | Guard countermodel check |
| G04 | Same source-local port p1 and same exact constant 0 on K and E | D1; identical guards; Proven | Exact TypeSchemaRef and valid input equality | Frozen guard containment |
| G05 | Same source-local p1; K input=0, E input=1 | D1; K possible while E false; Violation | p1=0 is valid and reachable; otherwise Unproven | Guard countermodel check |
| G06 | Same contract; K p1=0, E p2=0; exact domains admit p1=0 and p2=1 | D1; K can occur while E is false; Violation | Typed witness p1=0,p2=1; no missing premise | Typed input-domain check |
| G07 | K forward guard unknown/open, E unconditional | D1; unknown possibility is not a finite guard; Unproven | Need complete source effect closure; no assumption of empty | Source-closure gate |
| G08 | K guard unknown, E input p1=0 | D1; no comparable typed guard; Unproven | Need exact forward occurrence and guard source | Guard obligation builder |
| G09 | Dynamic SELECT path with exact source guard; E unconditional | D1; all branch possibilities fit unconditional; guard relation Proven | Selector totality is reported separately; no branch is dropped | Control-path/E check |
| G10 | SELECT path proves p1=0; E input p1=0 | D1; K possibility is inside E; Proven | Exact selector-to-input expression identity and branch evidence | Typed control-path implication |
| G11 | Forward guard is covered by E, but receipt status is UNKNOWN | D1; guard relation Proven, actual eligibility RequiresEvidence | No receipt/reconciliation observation; receipt cannot narrow static upper | K eligibility report |
| G12 | Exact RecoveryActionIdentity action atom absent from a closed E row | D1; required action/E coverage fails; Violation | Complete closed row and exact atom lookup; open/unknown row instead Unproven | Contract K/E coverage |
| G13 | Two source components each use numeric input port 1, with different contract digests | D1; coordinates are distinct; Unproven until each is mapped | Need each ScopedPortMapping and exact expression/type; equal integers are no witness | Root guard coordinate map |
| G14 | Exact action and matching usage binding, but action upper is zero | D1; no positive possibility, guard obligation vacuous; Proven for this relation | Checked zero upper; no receipt claim | Zero-bound fast path |
| G15 | SELECT branch is reachable at p1=1; E input p1=0 | D1; K may occur outside E; Violation | Exact branch witness and p1=1 validity | Control-path countermodel |
| G16 | Dynamic selector and E input p1=0, no selector-to-p1 relation | D1; implication/countermodel unavailable; Unproven | Missing selector relation is retained | Control-path proof obligation |

## Local/root action and Recovery Usage vectors

| CaseId | Formal input shape and exact identities | Decision / expected relation / verdict | Preconditions; counterexample or missing evidence | Future production entry point |
| --- | --- | --- | --- | --- |
| K01 | One forward occurrence F=1, no control factors, activation=2, attempts=3, q=1, E=6, U.recovery=6 | D2; local/root upper=6 and K usage fits E/U; Proven | All bounds finite and exact action/usage binding present | Local action-bound calculation |
| K02 | RETRY(3, COMPENSATION_SCOPE(INVOKE(F=1))); activation=2, attempts=2, q=1, root E=12 and U.recovery=12 | D2; local=4, root=12 and root K fits; Proven | Retry control node identity attached once; treating local 4 as root is a counterexample | Root factor conversion |
| K03 | COMPENSATION_SCOPE(RETRY(3, INVOKE(F=1))); activation=2, attempts=2, q=1, E/U=12 | D2; local/root=12 and no second factor; Proven | Inner Retry factor already present; 36 would be double multiplication | Local factor fold |
| K04 | ITERATE(4, COMPENSATION_SCOPE(INVOKE(F=2))); activation=1, attempts=2, q=1, root E/U=16 | D2; local=4, root=16 and root K fits; Proven | Total count is 4; max_inflight is excluded | Root Iterate factor |
| K05 | RETRY(3, ITERATE(4, COMPENSATION_SCOPE(INVOKE(F=2)))); activation=2, attempts=2, q=1, root E/U=96 | D2; local=8, root=96 and root K fits; Proven | Distinct Retry and Iterate node IDs, each factor once; 24/384 are counterexamples | Root multiplicity provenance |
| K06 | Forward and compensation share exact full EffectAtom; F=1, activation=2, attempts=3 | D2; K action=6 and E ceiling must cover F+K=7; E=6 is Violation | Same complete atom including qualifier/scope; collapsing opportunities gives false 6 total | Same-atom K/E coverage |
| K07 | Two distinct source K bindings share one action atom, root co-occurring; action uppers 4 and 6 | D2; aggregate=10; E=9 Violation | Distinct source identities preserved; per-binding checks alone miss 1 | Root atom grouping |
| K08 | SEQUENCE children contribute same action atom uppers 2 and 3; q=1, root E/U=5 | D2; conservative root upper=5 and coverage fits; Proven | Later child lower remains zero; using max=3 undercounts | Sequence action fold |
| K09 | SELECT branches contribute same atom uppers 4 and 8; q=1, root E/U=8 | D2; mutually exclusive join upper=8 and coverage fits; Proven | Branch identity is exact; summing 12 is not the selected formal result | Select branch join |
| K10 | PARALLEL all branches contribute same action atom uppers 2 and 5; q=1, root E/U=7 | D2; root upper=7 and coverage fits; Proven | Both branches may execute; max=5 undercounts | Parallel action fold |
| K11 | RACE contestants contribute possible action uppers 2 and 5; q=1, root E/U=7 | D2; root upper=7 and coverage fits; Proven | No winner observation; dropping loser upper 5 is unsound | Race action fold |
| K12 | HANDLE protected action upper=3; exclusive handler upper=4; no-handler=0; q=1, root E/U=7 | D2; protected plus handler join upper=7 and coverage fits; Proven | Handler branch is conditional; do not sum mutually exclusive handlers | Handle action fold |
| K13 | Nested COMPENSATION_SCOPE( COMPENSATION_SCOPE(A) ), F=1,A=2,T=2, root E/U=4 | D2; inner owns the local projection; root counts binding once; Proven | Checked ownership tree and exact source identity; source-node dedup alone is insufficient | Scope owner assignment |
| K14 | Same nested shape; outer reads inner contribution; F=1,A=2,T=2, root E/U=4 | D2; outer is transitive only; total remains 4; Proven | Counting two copies is the counterexample | Nested-scope projection |
| K15 | One complete action source under no COMPENSATION_SCOPE ancestor; F=1,A=2,T=2, E/U=4 | D2; RootRecovery owner retains action total 4; Proven | Exact K binding and root path present; empty local scope list is not root zero | RootRecovery owner |
| K16 | Per-attempt recovery meter q=2, root action upper=9, Root U.recovery upper=18 | D2; K usage=18 ≤ U=18; Proven; do not add to U | Exact phase/meter/unit and full action profile | Root K/U coverage |
| K17 | Same as K16 but Root U.recovery upper=17 | D2; K exceeds declared recovery ceiling; Violation | Exact meter coordinate and finite bounds; tolerance/rounding is not a premise | Root K/U comparison |
| K18 | Checked source omits a meter required by its exact complete RecoveryUsageCoverage | D2; invalid K profile; Violation before root projection | Complete exact coverage catalog; missing is not zero | Recovery profile closure |
| K19 | Two applied factor records have same IR digest/node/kind but different upper | D2; inconsistent factor identity; Violation | Checked control node is the authority; multiplying both or selecting max is invalid | Factor identity validation |
| K20 | Retry/Iterate factor is unbounded and K action upper is positive; export E finite | D2; root action upper is unbounded and exceeds a finite ceiling; Violation | Positive source/action and exact finite export ceiling; zero action is the vacuous exception | Unbounded root action comparison |
| K21 | Two mutually exclusive branches have per-attempt meter uppers 3 and 5; root U.recovery=5 | D2; root K usage is max 5, fits U; Proven | Exact SELECT/HANDLE branch identities; source remains on both branches | Usage branch fold |
| K22 | Two K sources in parallel use same meter/atom; per-source usage uppers 3 and 4; root E=7 and U=7 | D2; action total=7 and meter total=7; Proven | Exact units and source identities; dropping either source is a counterexample | Root source multiset fold |
| K23 | Same action operation/resource but qualifiers idempotency.natural versus idempotency.none | D2; two exact atom identities remain separate; Proven as an identity/accounting check | Complete qualifier member identity; family-only matching is counterexample | Exact atom grouping |
| K24 | Forward exact occurrence exists but its matching RecoveryUsageBinding is absent | D2; no finite K usage projection; Violation under complete binding rule, never zero | Complete checked source contract; incomplete source context instead Unproven | K binding/profile matcher |

## Root map and cross-dimension vectors

| CaseId | Formal input shape and exact identities | Decision / expected relation / verdict | Preconditions; counterexample or missing evidence | Future production entry point |
| --- | --- | --- | --- | --- |
| X01 | One export input port maps to one exact RunParameterId/ValueSlotId with exact TypeSchemaRef; all sets biject | D3; root I matches export; Proven | Type DAG and producer available; port numbers need not match | Root input map validator |
| X02 | Export declares input port 2 but no root RunParameter source is mapped | D3; required input missing; Violation | Exact export digest and complete input list | Root input coverage |
| X03 | Root has an extra required RunParameter absent from exported I | D3; root adds a public precondition; Violation | RunParameter is required and exact; unused extra still changes public call arity | Root input coverage |
| X04 | Export ports 1 and 2 both map to the same root source | D3; v1 map is invalid/strengthens independent-input contract; Violation | Export has no alias declaration; equality constraint is counterexample | Root input injectivity |
| X05 | Root type and exported TypeSchemaRef have different nominal pins but similar display names | D3; no exact type proof; Unproven | Need formal subtype/alias rule, which v1 does not supply | Exact input type comparator |
| X06 | Export output port 4 maps explicitly to WorkflowOutput name result and exact expression/type | D3; root O supplies export output; Proven | Expression type, producer, dominance and scope all checked | Root output mapping |
| X07 | Export output port has no WorkflowOutput mapping | D3; required output absent; Violation | Complete export output set | Root output coverage |
| X08 | Mapped root output is proven to have incompatible TypeSchemaRef | D3; output contract mismatch; Violation | Exact schema pins and result type known | Root output type proof |
| X09 | Root ResourceSlot r7 maps to export resource port 3; exact type, attributes, and requirement predicates are equal | D3; explicit R map and relation prove equal requirements; Proven | Scope visibility and resource relation rows present | Root resource comparator |
| X10 | Root ResourceSlotId 3 and export resource port 3, but no mapping record | D3; identity cannot be inferred; Unproven | Numeric coincidence is the counterexample | Root coordinate completeness |
| X11 | One root ResourceSlot maps to two distinct export resource ports without alias rule | D3; no v1 positive proof; Unproven | Export has independent requirement ports; exact alias relation absent | Resource alias validation |
| X12 | Root E upper=12 for exact atom; export E upper=8 after full resource mapping | D3; Root epsilon exceeds export ceiling; Violation | Exact atom/qualifier/scope/guard and closed rows | Root Effect containment |
| X13 | Root known Failure TypeSchemaRef is absent from complete exported Phi | D3; root failure outside interface; Violation | Exact schema pin and closed failure rows | Root failure containment |
| X14 | Root K action total=9 with q=1, Root U.recovery=10, export U.recovery=8 | D2/D3; K usage 9≤Root U 10 but 9>export 8; Violation | Same recovery phase/meter/unit; adding 9 to U is a counterexample | Root K/export U comparison |
| X15 | Root K action possibility guard unconditional; export action E guard input p1=0; p1 has valid alternate | D1/D3; action outside export ceiling; Violation | Exact input map, schema domain, reachable countermodel | Root K/E guard comparison |
| X16 | Root F constraint allows an egress forbidden by exact exported Flow constraint | D3; root relaxes F; Violation | Exact sink/FlowKind/label mapping | Root Flow non-relaxation |
| X17 | Root Flow has unknown external provenance but no exact export source mapping | D3; Flow relation unavailable; Unproven | A trusted provenance object could later resolve; never assume trusted | Root Flow mapping |
| X18 | One child X profile violates an exact exported security constraint while other children pass | D3; per-child X security preservation fails; Violation | Exact execution universe and child source identity | Root X per-child comparison |
| X19 | Same SemanticIrDigest and contract digest, but relation content checksum changed; attempt to reuse old report | D3; stale report is not applicable; Unproven until fresh derivation | Exact old/new proof manifests; revision string alone is insufficient | ProofContextManifest equality |
| X20 | Same maps/context but CompilerSemanticsIdentity changed; attempt to reuse old report | D3; stale report is not applicable; Unproven until rerun | Exact normative SpecificationId/RuleId set | Compiler semantics gate |
| X21 | All static dimensions are Proven; CALL_SUBFLOW target IR/export resolves but trusted target body evidence is absent | D3; static envelope Proven, body conformance RequiresEvidence | Exact target pins and transitive dependency list | Subflow body dependency |
| X22 | Complete source contract requires an action/profile; closure retains its node ID but drops one exact RecoveryUsageBinding | D2/D3; required profile is missing; Violation | Exact action identity, source digest, and complete profile set | Recovery multiset closure |

## Coverage note

There are 16 guard vectors, 24 action/usage vectors, and 22 root-map/cross-dimension vectors: 62 total. This exceeds the required 36 and covers AUDIT-03/04/05/06/07/09 plus guard mismatch, nested-scope ownership, root K/E/U, output/resource maps, context invalidation, and subflow/body evidence. They remain design vectors; a future authorized conformance phase must encode independent oracle expectations before registering them.
