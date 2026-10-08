# Static Recovery Contract (agentflow.recovery.contract@1)

Status: frozen for Phase 4A.6. The machine schema authority is
`spec/idl/recovery-contract.afidl`; Rust is `crates/af-recovery`; the
independent corpus oracle is `tools/gen-recovery-contract-corpus.py`.

## 1. Static boundary

<a id="recovery-static-boundary"></a>

K declares replay, idempotency, reconciliation and compensation obligations.
It does not execute recovery, issue a receipt, create an EffectGrant, derive
an idempotency key, inspect a provider, mutate a journal or produce runtime
state.

```text
Invocation ≠ Attempt
Retry ≠ Recovery Replay
Recovery Replay ≠ Historical Replay
Recovery Replay ≠ Fork
Idempotency Declaration ≠ Idempotency Proof
Compensation ≠ Rollback
RecoveryBlocked ≠ Typed FailureValue
```

## 2. Replay modes

<a id="recovery-replay-modes"></a>

`ReplaySemantics` is closed at `recompute`, `recorded`, `reconcile` and
`forbidden`.

- `recompute` permits recomputation only after effect recovery obligations;
- `recorded` requires a durable recorded-result evidence obligation;
- `reconcile` requires reconciliation capability and authoritative evidence;
- `forbidden` disallows replay execution and is not relaxed by timeout or
  worker loss.

<a id="recovery-replay-not-authority"></a>

A replay declaration is not authority or a safe retry proof. Historical replay
reads history without dispatching effects; a fork is a new Run needing fresh
admission and authorization.

## 3. Idempotency and key obligations

<a id="recovery-idempotency"></a>

`IdempotencySemantics` is closed at `natural`, `keyed`, `none` and
`reconciled`. All four are declarations, not provider attestations.

<a id="recovery-key-obligations"></a>

`keyed` requires a `KeyStabilityRequirement`: stable across attempts, distinct
across logical invocations, bound to the intended operation and honored by the
provider/adapter. This phase declares these obligations but never generates an
InvocationId, AttemptId or IdempotencyKey.

## 4. Exact effect recovery clauses

<a id="recovery-effect-identity"></a>

Each `EffectRecoveryClause` uses the complete `effect.EffectAtom` identity:
operation, ResourceSelector and qualifiers. Short operation names never merge
different scopes or qualifier sets. Clauses are sorted by the canonical
EffectAtom key and duplicate identities are rejected.

<a id="recovery-coverage"></a>

Every known EffectAtom requiring recovery must have a matching clause. An
unknown/open Effect row cannot be silently treated as covered. A clause for an
Effect absent from ε is a static mismatch, not an authorization.

## 5. Reconciliation and unknown effects

<a id="recovery-reconciliation"></a>

`reconciliation = required` declares capability and evidence obligations; it
does not provide an actual result. Future evidence values are distinct:

```text
APPLIED | NOT_APPLIED | PARTIALLY_APPLIED | STILL_UNKNOWN
```

<a id="recovery-unknown-effect"></a>

Only `block` and `reconciliation_required` are valid unknown-effect handling.
`UNKNOWN` MUST NOT be blindly retried. `PARTIALLY_APPLIED` MUST NOT be treated
as `NOT_APPLIED`, `APPLIED` or fully compensated. A `NONE + UNKNOWN` retry is
an unsafe blind retry; keyed or reconciled declarations without evidence are
`RequiresEvidence` / `Unproven`.

## 6. Compensation and history

<a id="recovery-compensation"></a>

Compensation is a new explicit EffectOperationRef obligation. It is not a
rollback API, EffectGrant, receipt or executable component.

<a id="recovery-forward-history"></a>

Forward effect history is never erased:

```text
Forward Effect + Conditional Compensation Effect
```

not `Forward Effect - Compensation Effect`. A successful refund does not make
the original charge disappear, and recovery usage is not subtracted from
forward Usage/Cost.

## 7. Static proof and compatibility

<a id="recovery-static-proof"></a>

`analyze_recovery_safety` returns:

```text
ProvenStaticConstraint
RequiresEvidence(obligations)
Unproven(reason)
Violation(reason)
```

`ProvenStaticConstraint` is only a static declaration judgment. It is never a
runtime retry permit, policy approval, budget reservation or provider result.

<a id="recovery-compatibility"></a>

`recovery_not_weakened(new, old)` proves only exact-equivalent declarations
in this phase. Removing a recovery clause is `Weakened`; changing replay,
idempotency, evidence or compensation semantics is `Unproven` unless a future
formal monotonic rule is added. Enum numbering is not a semantic strength
order.

## 8. Cross-dimension and runtime boundaries

<a id="recovery-cross-dimensions"></a>

K must eventually correspond to every recoverable ε Effect, its Φ outcome
plane, and U recovery budget. This phase may validate explicit ε/K exact
identity inputs but does not create a temporary ε+K ComponentContract wire
object. `RecoveryBlocked` is not a business FailureValue.

<a id="recovery-no-runtime"></a>

No EffectReceipt, EffectIntent, EffectGrant, Reconciliation execution,
Compensation execution, Retry Controller, Recovery Journal, Plan, Reducer,
Runtime, Scheduler, Worker, ComponentContract or RecoveryContractDigest is
introduced. HashV1 remains at 14 domains.
