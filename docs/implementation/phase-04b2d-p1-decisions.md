# Phase 4B.2D — P1 Decision Register

All choices and candidate records here are **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED ABI / NOT IMPLEMENTED**. Status ACCEPTED means the semantics are fixed enough for a separately authorized checker implementation; it does not mean production code or corpus coverage already implements them.

## DECISION-D1 — Effect Guard / Recovery Action Guard Semantics

**Status: ACCEPTED**

### Problem

The exact K action identity binds a forward atom, action kind, and action atom, but has no independent action guard. The current ComponentContract validator checks action atom presence; CompScope derives a potential action from the Forward guard and checks numeric upper coverage without proving that this possibility is contained by the declared action E guard.

### Existing normative basis

EffectGuard narrows when an Effect may occur and is closed to unconditional or input-equals. The language has no conjunction, negation, or arbitrary path condition. Frozen Effect containment proves conditional-under-unconditional and exact equality; other cases are not proven. ComponentContract epsilon is the public Effect ceiling. K action binding requires the complete action atom to appear in a closed epsilon row. RecoveryReceipt and recovery execution are not static facts.

### Alternatives

| Alternative | Decision |
| --- | --- |
| A. Declared epsilon guard is the ceiling for all occurrences of its full atom, including K actions | **Chosen.** Complete public Effect accounting remains one ceiling; K cannot create an unreported sidecar. |
| B. K actions form an independent conditional Effect sidecar outside epsilon | Rejected. It would require a second public effect promise, complete epsilon/resource/flow/X/substitution rules, and a versioned schema; otherwise epsilon completeness is ambiguous. |
| C. Treat the action E guard as an execution condition that can narrow K action eligibility without a compatibility proof | Rejected. It can hide a K possibility outside the declared public ceiling and conflates declaration with runtime behavior. |

### Chosen model and formal relation

For each exact action binding:

* KActionPossibleGuard starts from the matched Forward Effect guard. Typed control-path facts are retained separately and may narrow only when exact existing selector/dataflow evidence proves that implication.
* Actual eligibility additionally depends on receipt and recovery state. Those facts may narrow execution but are not assumed when constructing a static upper.
* DeclaredActionEffectGuard is the guard on the exact full action EffectAtom row in the source ComponentContract.
* Require KActionPossible(source, path) ⊆ DeclaredActionEffectPossibility and then repeat the relation after mapping into exported coordinates.
* The frozen guard_contained relation is the positive proof primitive. A false result alone is Unproven. Emit Violation only with a type-valid, reachable valuation that satisfies K possibility and falsifies the E ceiling.
* A positive count with unknown guard/source is Unproven. A zero upper makes the guard relation vacuous. Receipt uncertainty stays RequiresEvidence.

The root action guard is a typed pair of the source EffectGuard and control-path witnesses. The checker does not synthesize a conjunction or alter EffectGuard ABI.

### Proof direction and typed identity

The direction is possible K action contained by the exact declared E action ceiling. Identity includes caller SemanticIrDigest, source ComputationId, source ComponentContractDigest, forward entry ordinal, full RecoveryActionIdentity (forward atom, action kind, action atom), matching RecoveryUsageBinding, guard source, and factor IDs. Root mapping retains exact ScopedPortMapping and expression/type evidence; numeric port equality across contracts is not identity.

### Counterexamples

* Forward unconditional, exact action E input p1=0, and a valid reachable p1 value other than zero: Violation.
* Forward input p1=0, action E input p1=1, with p1=0 valid and reachable: Violation.
* Forward input p1=0, action E unconditional: Proven.
* Different ports with no domain/alias evidence: Unproven, not a guessed Violation.
* Exact guard coverage but receipt UNKNOWN: guard Proven, action eligibility RequiresEvidence.

The Phase 4B.2C isolated action-bound-06 probe remains evidence of the gap, not a P0: the projected sidecar was wider, and the overall result remained RequiresEvidence; no demonstrated false-negative total was found.

### Failure-closed behavior

Missing exact E action atom or a proven countermodel is Violation. Incomplete source, invalid/unresolved port mapping, unknown domain, or unprovable branch implication is Unproven. Receipt, reconciliation, and actual eligibility are RequiresEvidence. No missing guard becomes unconditional permission, and no action sidecar is silently omitted.

### ABI, conformance, and runtime impact

No ABI assignment, AFIDL, EffectGuard, RecoveryContract, or RuleId changes in this design. Future conformance must include exact-equal, E-wider, E-narrower with witness, different same-port condition, different-port coordinates, unknown guard, branch paths, and receipt boundaries (G01-G14). This is static ceiling compatibility only; it creates no receipt, action, or dispatch.

### Residual risks

The current validator does not yet perform the new guard relation. Type/domain and control-path implication must be limited to exact typed evidence; unmodeled expression semantics remain Unproven. Action-specific F/X remains a separate P2 boundary.

## DECISION-D2 — Root-relative Recovery Action Aggregation

**Status: ACCEPTED**

### Problem

ConditionalCompensationAction is computed bottom-up at a CompensationScope node. Its current upper includes factors visible inside that node, but an ancestor Retry/Iterate may be added later. The local number is not a root total. Summing nested scope summaries can double-count; using only the innermost number can omit ancestor repetition.

### Existing normative basis

The frozen local formula is Forward occurrence upper × inner Iterate/Retry families × activation upper × attempt upper, with checked arithmetic. Retry and Iterate have distinct logical factors; max_inflight is not total work. Control algebras use accumulation, exclusive joins, or repetition according to operator. Root U already composes component recovery usage. K source/action/usage bindings remain per child and exact.

### Alternatives

| Alternative | Decision |
| --- | --- |
| A. Keep CompScope node-local and expose a separate root fold over source bindings and the whole checked tree | **Chosen.** It gives local and root totals different typed coordinates and applies every factor/operator once. |
| B. Treat each CompScope upper as root-total | Rejected. It omits ancestor Retry/Iterate factors. |
| C. Multiply every scope upper by every factor seen at root | Rejected. It double-applies factors already present inside the scope. |
| D. Sum all actions across the whole root irrespective of control operator | Rejected as the unique algebra. It misstates mutually exclusive SELECT/HANDLE paths; a safe bound is operator-specific. |

### Chosen model and formal relation

LocalActionBound means one activation of the exact CompensationScope node; descendant Iterate/Retry factors are included, ancestors are not. RootActionBound is derived from leaf K bindings with every unique path factor and every operator algebra applied once.

ControlMultiplicityIdentity is the tuple of SemanticIrDigest, control node, and Iterate-or-Retry kind, plus the checked logical repetition bounds and optional physical-attempt bounds. A factor already present by this identity is not multiplied again. Conflicting bounds for the same identity are a Violation.

SEQUENCE/PARALLEL/RACE use their frozen checked accumulations; SELECT uses exclusive branch join; ITERATE/RETRY apply their logical upper once; TIMEOUT/SCOPE preserve possible work; HANDLE is protected work plus conditional mutually exclusive handler join; CompScope is an ownership boundary, not another multiplier.

The nearest enclosing CompScope owns a binding's local projection. An outer scope carries inner-owned actions transitively and never re-owns them. Root aggregation traverses source/action bindings once. A source outside any CompScope is retained under typed RootRecovery ownership. Source node alone is insufficient: source contract digest, action identity, forward ordinal, matching usage profile, and control path remain part of identity.

Root K-required Recovery Usage is independently projected into root coordinates and compared with Root U.recovery and Export U.recovery. It is not added to either envelope. Same-full-atom Forward and K action occurrences both count against the E ceiling; atom equality is not event equality.

### Proof direction and typed identity

Each source action/profile contributes a root upper and exact factor set; the resulting K action upper must fit Root epsilon and Export epsilon. Each K recovery-usage upper must fit Root U.recovery and Export U.recovery in the same exact phase/meter/unit coordinate.

### Counterexamples and manual results

* Retry(3, CompScope(INVOKE(F=1))), A=2,T=2: local 4, root 12.
* CompScope(Retry(3, INVOKE(F=1))), A=2,T=2: local/root 12; no second Retry.
* Iterate(4, CompScope(INVOKE(F=2))), A=1,T=2: local 4, root 16.
* Retry(3, Iterate(4, CompScope(INVOKE(F=2)))), A=2,T=2: local 8, root 96.
* Same atom F=1 plus K action=6 requires one E ceiling at least 7.
* Same action atom from two co-occurring bindings 4+6 requires root coverage 10; checking each alone against E=9 misses the undercoverage.
* SELECT(4,8) joins to 8; PARALLEL(2,5) and RACE(2,5) conservatively total 7.

### Failure-closed behavior

Overflow, exact underdeclared E/U, conflicting factor identity, or structurally invalid owner is Violation. Missing binding/profile/mapping, open source, or unsupported operator relation is Unproven. Concrete receipt, action eligibility, reconciliation result, or commit order is RequiresEvidence. Missing meter is never zero.

### ABI, conformance, and runtime impact

No CompScope, K, Usage, or Semantic IR ABI changes. Future cases must cover nested Retry/Iterate both orders, factor provenance, same atom, shared action atom, SELECT/HANDLE versus parallel/race, nested scope ownership, full RecoverySource/action/profile multiset, overflow, and K/U coverage. The 62 design vectors propose this coverage; no Expected file changes now.

The nearest-scope rule is accounting ownership only. It does not assert runtime commit order, effect eligibility, or which recovery dispatcher executes.

### Residual risks

K does not carry action-specific Flow/X fields. Root K/action coverage can be statically bounded, but such per-action F/X conformance remains Unproven unless existing source-level partitions prove the needed relation. A future runtime model requiring independent nested recovery dispatch must version a replacement ownership rule before implementation.

## DECISION-D3 — Root Export Assertion and Compatibility Proof

**Status: ACCEPTED**

### Problem

The Semantic IR has root outputs but no root exported ComponentContract pin. Root ValueSlots, WorkflowOutputs, ResourceSlots, per-child K/F/X sources, and component ports use different namespaces. Existing ComponentContract substitution compares two contracts and does not prove that a workflow body conforms to an export.

### Existing normative basis

ComponentContract has exact nominal I/O pins, epsilon/resource/flow/usage/recovery/execution dimensions, and a stable component-contract digest. Semantic IR has its own digest and workflow outputs. The current proof context is exact-pointer scoped in-process. There is no WorkflowVersion type in this phase. The artifact HashV1 domain is already reserved for immutable artifact content identities.

### Alternatives

| Alternative | Decision |
| --- | --- |
| A. Explicit typed build-time assertion | **Chosen.** It binds exact IR/export digests, all typed coordinate maps, compiler semantics, and context. No IR ABI change. |
| B. Versioned Semantic IR export pin | Rejected now. It changes frozen IR ABI and still needs all maps and proof rules. |
| C. Deferred publisher attestation | Rejected as a substitute for static comparison. A trusted body attestation may complement static proof, not replace it. |

### Chosen model

RootExportAssertion contains exact SemanticIrDigest, ComponentContractDigest, CompilerSemanticsIdentity, content-complete ProofContextManifest, typed I/O/R/E/F/K/X mappings, subflow dependencies, and per-dimension report. Its owner is the build/publish boundary; it is not serialized inside the frozen IR and is not a ComponentContract.

Input maps are bijective between exported input ports and required root RunParameters in v1, with exact nominal type pins and producer/use-site evidence. Outputs map each exported port explicitly to a named WorkflowOutput and exact expression/producer/type. Resource ports map through exact ResourceSlot and type/attribute/selector/scope evidence. Effect, Flow, Recovery, and Execution maps preserve source identities and exact resource/port namespaces.

The proof directions are implementation-within-export: root epsilon/K actions fit the public ceiling; root failures fit Phi; root resource requirements are not stronger; root Flow is not relaxed; root Forward/Recovery Usage fits each ceiling; K is not weakened; and each child X satisfies feasibility/security separately. Full predicates are in phase-04b2d-cross-dimension-proof.md.

### Typed identity and persistence

Assertion identity is the complete input tuple, not a digest. Semantic IR, ComponentContract, compiler semantics, context, maps, and resolution dependencies remain distinct. The process-local pointer is not serializable.

The selected future version binding is exact: a future additive WorkflowVersion/release artifact includes the entire assertion and manifest inline in its canonical payload; that parent uses the already registered artifact content-identity domain if its later schema specifies the exact payload. No RootContractDigest is created. Until such a parent is formally specified, the assertion is transient, not reusable across processes, and not signed.

### Counterexamples and verdicts

* Missing exported output or extra required root input: Violation.
* Equal port integers without a typed map: Unproven.
* Root E=12, export E=8: Violation.
* Changed relation content or compiler semantics: old result invalid; full derivation required.
* CALL_SUBFLOW target export resolves but body proof is absent: static compatibility may be Proven; body conformance remains RequiresEvidence.
* Missing action-specific Flow/X relation: Unproven, not silently inherited as a K-specific proof.

### Failure-closed behavior

Exact mismatch or known containment failure is Violation. Missing/ambiguous coordinate, context content, unsupported variance, or relation is Unproven. Runtime evidence and trusted body evidence remain RequiresEvidence. All dimension reports and prerequisites are retained.

### ABI, conformance, and runtime impact

No AFIDL, ABI, Semantic IR, hash-domain, or RuleId changes. The 62 design vectors cover input/output/resource mapping, all cross-dimensions, K action/usage, context invalidation, and body evidence; future production conformance is a separate authorized task.

The assertion proves static envelope compatibility only. It does not prove implementation conformance, publisher authenticity, Scope authority/budget attenuation, Worker admission, runtime budget reservation, effect dispatch, or recovery execution.

### Residual risks

The proof-context manifest and WorkflowVersion owner are design requirements only; neither exists as a persistent production schema here. Future publication work must specify canonical encoding and authentication separately. Until then, no durable assertion claim is allowed.
