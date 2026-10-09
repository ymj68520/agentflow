# Phase 4B.2D — Effect Guard / K Action Compatibility

All candidate structures and rules in this document are **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED ABI / NOT IMPLEMENTED**.

## Normative basis and decision

The frozen EffectGuard language is exactly unconditional or input-equals(port, PrimitiveConstant). It says when an Effect may occur; it has no conjunction, negation, branch predicate, receipt state, or arbitrary expression. Frozen containment proves conditional-under-unconditional and exact guard equality. Unconditional-under-conditional and distinct conditions are not proven by the current algebra.

RecoveryActionIdentity contains the complete forward atom, action kind, and action atom, but no separate action guard. The action atom must appear in closed ComponentContract epsilon. The selected D1 model treats that declared epsilon entry's guard and upper bound as the ceiling for **all** occurrences of that atom: forward, reconciliation, and compensation. A K sidecar is not a second public Effect declaration and cannot exceed the ceiling.

## Guard sources and relation

Keep these facts separately typed; never flatten them into a string or a fabricated EffectGuard:

| Fact | Source and namespace | Meaning |
| --- | --- | --- |
| ForwardEffectGuard | Exact scoped forward EffectEntry; component input-port namespace owned by source contract digest and computation node | Necessary guard condition for a K action triggered by that Forward occurrence |
| ControlReachability | Exact owned control path: operator node IDs, child/case/handler ordinals, and existing selector/dataflow evidence | May narrow which source occurrence is reachable; it is not an EffectGuard |
| KActionEligibility | Exact K action identity, recovery clause, and matching usage binding | Actual eligibility also depends on applied/partial/unknown receipt and recovery state, which static analysis does not observe |
| DeclaredActionEffectGuard | Exact action EffectEntry for the full action atom in that same source contract | Public epsilon ceiling for every occurrence of that atom |
| RootGuardCoordinates | Exact invocation-port mapping to a root expression/value and its TypeSchemaRef evidence | Translates identities; never inferred from equal port numbers |

For a positive upper, the static possibility is conservatively the ForwardEffectGuard plus the retained control-path provenance. Receipt eligibility is ignored only as a narrowing fact: the static projection is therefore wider than or equal to any runtime-eligible set. A zero K action upper makes the guard obligation vacuous.

The core obligation is:

~~~text
KActionPossible(source, control path)  ⊆  DeclaredActionEffectPossibility
~~~

Use the frozen guard_contained(K, E) relation on the component-local guard when it proves the implication. If a control path is needed to establish a narrower relation, use only exact typed branch/dataflow evidence already available (for example, an exact constant selector or a proven direct-value identity). This is proof evidence, not a new guard expression. Do not add arbitrary Boolean reasoning to EffectGuard.

If the necessary implication is not proven:

* **Violation** only when exact type/identity and reachable-path evidence supplies a valid valuation where the K action is possible and the declared action guard is false.
* **Unproven** when port identity, type domain, alias, path feasibility, or the required implication cannot be established. A false result from guard_contained alone is not a counterexample.
* **RequiresEvidence** remains for concrete receipt/reconciliation/eligibility facts. It cannot replace an unresolved static guard relation.

This preserves the frozen effect algebra's rule that not-proven containment is not proof. It adds no weakening and does not silently classify every failed proof as a Violation.

## Local-to-root coordinate bridge

At one source leaf, Forward and action guards use the same ComponentContract input-port namespace, scoped by exact source_contract_digest and ComputationId. The checker must first validate that a guard port names a real input and that its PrimitiveConstant is coherent with the exact input schema. A missing port with a proven exact contract is a Violation; an unavailable schema/catalog is Unproven.

For root comparison, use the existing scoped binding chain:

~~~text
ComponentInputPort
  -> ScopedPortMapping(contract_port_no, InputExpression, exact_type)
  -> exact expression / Phase 3C producer and dependency evidence
  -> Root RunParameterId / ValueSlotId
  -> exported ComponentInputPort
~~~

The assertion's input map supplies the last typed relation. A repeated integer such as port 1 in two different contracts is not the same input. If an expression transform prevents exact value correspondence, retain the expression and dependencies but return Unproven for a guard implication that depends on it.

The root action guard remains a pair of facts: the source EffectGuard and a list of typed control-path witnesses. It is not canonicalized as an unsupported conjunction. For a comparison that cannot reason over the path, widen conservatively to the known source Forward guard; if that still is not covered, return Unproven unless a reachable countermodel proves Violation.

## Decision matrix

All rows assume a positive potential action unless stated otherwise. The exact source contract digest, computation node, full RecoveryActionIdentity, matching RecoveryUsageBinding, and input schema/domain are part of the case identity.

| Case | K possibility / E ceiling | Source and required evidence | Relation and verdict | Counterexample or missing premise | Future checker entry |
| --- | --- | --- | --- | --- | --- |
| G01 | unconditional / unconditional | Same source contract and action atom | Proven; identical full guard | None | Component-local K/E pass |
| G02 | input p1=0 / unconditional | Same p1 TypeSchemaRef and valid literal | Proven; frozen conditional-under-unconditional case | None | guard_contained |
| G03 | unconditional / input p1=0 | Same source, exact non-singleton primitive domain | Violation with a valid p1 value other than 0 | Domain evidence absent: Unproven, not Violation | Guard countermodel check |
| G04 | input p1=0 / input p1=0 | Same source-local p1 identity and exact literal type | Proven; exact equality | None | guard_contained |
| G05 | input p1=0 / input p1=1 | Same source; both literals valid and p1=0 reachable | Violation; p1=0 witnesses K possible and E false | If p1=0 is not proven valid/reachable: Unproven | Guard countermodel check |
| G06 | input p1=0 / input p2=0 | Same component, independent exact p1/p2 domains with an alternate p2 value | Violation; p1=0,p2≠0 is a witness | If domains/alias relation absent: Unproven | Typed local-input solver |
| G07 | unknown forward guard / unconditional | Exact K source but incomplete/open source effects | Unproven; E ceiling cannot repair unknown K possibility | Unknown is not finite zero | Source-closure gate |
| G08 | unknown forward guard / input p1=0 | Exact action atom but missing guard/source proof | Unproven | No typed K guard exists to compare | Guard proof obligation |
| G09 | conditional branch path / unconditional | Dynamic SELECT path retained; source guard exact | Proven for E coverage because any source guard/path is inside unconditional | None for the guard relation; selector totality may remain Unproven separately | Control-path/E check |
| G10 | conditional branch path / input p1=0 | Exact path maps to p1=0, or exact reachable countermodel | Proven if path implication is established; Violation if a reachable path admits p1≠0; otherwise Unproven | Unknown branch selector cannot be guessed | Control-path implication |
| G11 | Forward guard covered / any E guard; receipt UNKNOWN | Exact K and Usage binding, but no receipt | Guard relation may be Proven; action eligibility remains RequiresEvidence | No applied-effect observation | K eligibility proof |
| G12 | any K action / no exact action E atom | Full atom lookup in a complete closed epsilon row | Violation under existing action coverage rule | Open/unknown epsilon instead: Unproven | Contract K/E validator |
| G13 | same numeric p1 in different components | Different source contract digests and expression maps | Unproven until each local port is independently mapped | Numeric equality does not identify value | Root coordinate mapper |
| G14 | any K action / K upper=0 | Exact action binding, zero upper | Guard obligation vacuous; no positive action occurrence | None | Checked-bound fast path |

## Control operators and failure closure

* SEQUENCE, PARALLEL, and RACE retain every potentially executed source; RACE losers are not discarded. The path is retained even if the occurrence algebra widens the merged guard.
* SELECT's exact constant case may remove an impossible branch from the root possibility projection only when existing selector evidence proves it. Dynamic or incomplete case coverage remains Unproven; all checked children still undergo structural/source validation.
* ITERATE and RETRY apply their frozen logical multiplicity; they do not change an EffectGuard. TIMEOUT preserves child effects. HANDLE preserves protected and possible handler sources; handler choice is unknown unless already proven.
* SCOPE and COMPENSATION_SCOPE preserve source identity and resource visibility. A scope boundary does not create a new guard namespace or choose a receipt.
* No operator turns missing K, effect, input, or context evidence into pure/false. Arithmetic overflow is Violation; missing identity/domain/path proof is Unproven; actual receipt is RequiresEvidence.

## Epsilon and K bounds

The guard relation does not change the frozen action-count formula. For each exact source/action:

~~~text
local K upper = forward upper × unique inner Iterate/Retry factors
               × activations-per-forward upper
               × attempts-per-activation upper
~~~

The action row's declared epsilon upper must cover the K action contribution and, if the full action atom equals the Forward atom, the Forward contribution too. Across sources, control algebra composes those contributions once. The action sidecar is a coverage obligation against epsilon; it is not added to epsilon a second time.

## Residual boundary

The current validator does not create a durable root guard identity and the current context is pointer-identical in-process. A future checker may prove only guard relations supported by exact source and root mappings. It must not claim that guard coverage proves a receipt, reconciliation outcome, action dispatch, or publisher/body conformance.
