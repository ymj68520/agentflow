# Phase 4A.6 Decision Register — Static Recovery Contract

Phase: 4A.6. Wire authority: `spec/idl/recovery-contract.afidl`; normative
semantics: `spec/recovery/recovery-contract.md`; Rust: `crates/af-recovery`.

## D-K1 Replay identity

Replay is closed at recompute, recorded, reconcile and forbidden. A replay
declaration is not authority; recorded requires durable evidence, reconcile
requires authoritative reconciliation evidence, and forbidden cannot be
lifted by timeout or worker loss.

## D-K2 Idempotency declaration

Natural, keyed, none and reconciled are declarations only. Keyed requires
stable/distinct/operation-bound/provider-honored key obligations; this phase
never generates a key or invocation/attempt identity.

## D-K3 Exact Effect identity

Recovery clauses reuse the complete canonical `EffectAtom` identity, including
operation, ResourceSelector and qualifiers. Short names are never recovery
identity. Duplicate clauses reject; clauses not present in ε are mismatches.

## D-K4 Unknown and partial safety

Unknown Effect state blocks blind retry. `NONE + UNKNOWN` is a static unsafe
violation. Keyed/reconciled unknown state without evidence produces obligations.
`PARTIALLY_APPLIED` requires unresolved-part evidence and is never coerced to
NOT_APPLIED or APPLIED.

## D-K5 Compensation

Compensation is a new EffectOperationRef obligation, not rollback or execution.
Forward Effect history remains present; compensation never refunds Usage/Cost
or erases forward effect/failure facts.

## D-K6 Proof and compatibility

Recovery safety returns ProvenStaticConstraint, RequiresEvidence, Unproven or
Violation. Compatibility proves only exact-equivalent declarations; removed
clauses are weakened and other mode changes are unproven. No enum total order
is invented.

## D-K7 Deferred integration

ε/Φ/U cross-dimension correspondence is checked only from explicit inputs;
there is no temporary ComponentContract. Receipts, ledgers, policy, runtime,
reconciliation, compensation, journal and retry execution remain deferred.
