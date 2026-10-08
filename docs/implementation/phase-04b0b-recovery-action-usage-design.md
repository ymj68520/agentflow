# Phase 4B.0B — Recovery Action and Usage Decisions

> **DESIGN PROPOSAL** · **NON-NORMATIVE** · **NOT A FROZEN ABI**

## Decision record B0B-D6 — Complete Recovery action identity

**Status: ACCEPTED (design only).**

### Problem and existing frozen representation

Frozen K binds every forward `EffectRecoveryClause` to a full
`EffectAtom`, but `ReconciliationRequirement` is only none/required and
`CompensationRequirement::Required` carries only `EffectOperationRef`. The
current reconciliation-operation recognition set is not joined to an
EffectAtom by the K model. An operation-only match cannot distinguish a
different R port, exact scope, qualifier member, or payload.

### Candidates and choice

* **A — modify/version the existing RecoveryContract schema.** Rejected for
  this candidate: it changes the frozen K ABI and forces migration of all old
  K declarations.
* **B — add exact Component-level Recovery bindings.** Chosen: preserve K
  bytes and require a typed binding for every action a K clause declares.
* **C — keep recovery action bindings in diagnostics/runtime state.**
  Rejected: then the immutable contract could not express or hash the action
  it claims is available.

### Candidate identity and field shape

```text
enum RecoveryActionKind {
    reconciliation = 1
    compensation   = 2
}

record RecoveryActionIdentity {
    forward_effect: effect.EffectAtom
    action_kind: RecoveryActionKind
    action_effect: effect.EffectAtom
}

record RecoveryActionBinding {
    identity: RecoveryActionIdentity
}
```

`RecoveryActionIdentity` is exactly the triple
`(forward EffectAtom, action kind, exact recovery EffectAtom)`. Every atom
uses Phase 4B.0A's full key: operation catalog/name, R port, sorted exact
scope bindings, qualifier family/member/payload. The action kind is part of
identity, so reconciliation and compensation with otherwise equal atoms are
distinct records. Candidate ComponentContract field 12 is the closed,
canonical list of these bindings.

V1 uses `action_effect.operation` as the reconciliation operation identity;
the full `action_effect` must be declared in ε. It deliberately does not treat
the existing test-only recognition set as an unbound alternate operation
identity. For compensation, `action_effect.operation` must exactly equal the
existing K `CompensationRequirement::Required(EffectOperationRef)` and the
full `action_effect` must also be in ε. There is no “same name” or “same port”
fallback.

The existing `RecoveryOperationUniverse` recognizes reconciliation operation
names, but the frozen `RecoveryContract` carries no reference to one and the
current canonicalizer/proof path does not bind that recognition fact to an
EffectAtom. V1 therefore uses the action EffectAtom's own exact
`EffectOperationRef` as the executable-effect identity; catalog recognition
alone is never treated as action coverage. A future workflow-level
reconciliation service identity would require a separate typed relation, not
an implicit bridge to this binding.

### Identity coverage and proof rules

For every closed ε forward atom with a K clause:

1. `clause.effect` must equal the exact forward EffectAtom (P0 identity is
   reused).
2. `reconciliation=required` requires exactly one reconciliation binding;
   `reconciliation=none` requires none.
3. `compensation=required(op)` requires exactly one compensation binding
   whose action operation is exactly `op`; `compensation=none` requires none.
4. No binding may refer to a forward atom without a K clause or to an action
   not declared as a full ε EffectAtom.
5. The transitive graph from each forward atom to its action effects must be
   acyclic. A cycle is a known Violation; incomplete graph/coverage is
   Unproven.

An action EffectAtom absent from a closed ε is a Violation. A missing action
binding, open/unknown ε, or unavailable catalog snapshot is Unproven. Exact
selector/qualifier mismatch is never repaired by `RecoveryOperationUniverse`
recognition or evidence lookup. Whether an action actually ran and what its
receipt/evidence says remain separate runtime facts.

### Alternatives, migration, and residual risk

The contract-level binding is an additive field in the new ComponentContract
candidate; `recovery-contract.afidl` and existing TypeSchemaDigests remain
unchanged. Old K stays valid as a standalone declaration, but a Component
claim requiring a concrete action is Unproven until the new binding exists.
The exact field-12 action identity is included in future ComponentContract
canonical bytes. No old TypeSchemaDigest or Recovery ABI is revised.
The checker can prove declared coverage only under explicit operation
catalogs; publisher truth, actual action execution, receipts, reconciliation
evidence, and compensation success require later attestation/runtime proof.

## Decision record B0B-D7 — K↔U Recovery Usage coverage

**Status: ACCEPTED (design only).**

### Problem and candidates

K may require reconciliation/compensation and recovery may retry, while U
separates forward and recovery meters. A matching meter name, an empty row, or
missing action profile does not prove zero use. The candidate compares:

* **A — one per-action profile and exact coverage relation.** Chosen.
* **B — one aggregate “complete” envelope with a self-asserted Boolean.**
  Rejected: a Boolean does not demonstrate which action/meter pairs were
  covered.
* **C — infer the profile from the existence of any recovery meter.**
  Rejected: one meter does not cover all actions or coordinates.

### Candidate type and field shape

```text
record RecoveryUsageBinding {
    action: RecoveryActionIdentity
    activations_per_forward: effect.OccurrenceBounds
    attempts_per_activation: effect.OccurrenceBounds
    per_attempt_usage: usage.UsageSignature
}

record RecoveryUsageCoverageRule {
    action: RecoveryActionIdentity
    complete_meters: list<usage.UsageMeterRef>
}

record RecoveryUsageCoverageCatalog {
    rules: list<RecoveryUsageCoverageRule>
}
```

`RecoveryUsageBinding` is candidate ComponentContract field 13. It is
canonicalized by exact RecoveryActionIdentity. Its `per_attempt_usage` may
contain only `UsagePhase::Recovery`. Both multiplicities use the existing
closed `effect.OccurrenceBounds` count type; no missing field, retry default,
or inferred single attempt exists. The maximum attempts lower bound must be
at least one whenever the action can activate, and a K-required action must
have an activation upper bound of at least one. Lower bounds describe
guaranteed occurrences only; ceiling arithmetic uses upper bounds exclusively.
`RecoveryUsageCoverageCatalog` is a separate additive
analysis-catalog schema (`agentflow.recovery.usage.coverage@1`, provisional
and unregistered), versioned and pinned as an explicit analysis input.
The recovery binding, both multiplicity bounds and per-attempt UsageSignature
are included in future ComponentContract canonical bytes; the external meter
coverage snapshot is proof context, not a price or cost field.

The binding list must contain exactly one profile for each
`RecoveryActionBinding` identity and no orphan profiles. Duplicate identities
reject. A missing profile is Unproven, even if another action uses the same
meter.

For each exact action identity, the coverage catalog declares the exhaustive
set of meter coordinates that must be reported. The contract profile must
contain exactly one explicit bound for each required meter, including
`finite(0)` when the action declares no consumption for that meter. Missing
catalog rule or coordinate is Unproven, never zero. An empty complete meter
set is allowed only as an explicit catalog rule for that exact action.

### Proof direction and arithmetic

For each recovery action, the checker computes a conservative upper bound:

```text
forward_effect_occurrences.upper
× activations_per_forward.upper
× attempts_per_activation.upper
× per_attempt_usage[recovery meter]
```

Every multiplication and sum is checked. Any unbounded factor produces an
unbounded action bound. The checker accumulates every reachable action profile
(it does not assume mutually exclusive actions) and compares the resulting
recovery-phase coordinates with `U.recovery` using the exact additive meter
universe.

* Exact complete profile and aggregate ≤ explicit U recovery ceiling: Proven
  under the named coverage/meter snapshots.
* Missing action profile, missing required meter, unknown meter unit,
  incomplete coverage catalog, or missing U coordinate: Unproven.
* Explicit U finite ceiling below a known action aggregate, or checked
  arithmetic overflow: Violation.
* Unbounded action aggregate with U explicitly unbounded for every required
  coordinate: Proven as an unbounded declaration, not a finite budget proof.

This conservative accumulation may overestimate mutually exclusive recovery
paths. A tighter branch/join result requires a future closed recovery-control
graph; it cannot be inferred from an action list.

### Identity, digest, and runtime boundary

Profiles join only by the full RecoveryActionIdentity, not by operation name
or meter name. The binding, multiplicities, and per-attempt recovery meter
limits enter ComponentContract canonical bytes. Coverage and meter catalogs
are analysis inputs whose exact catalog revisions and canonical-content
checksums are recorded in the proof report; proofs compared under different
snapshots are Unproven absent a catalog evolution relation. No PriceTable or
CostProjection enters the contract body. Cost is projected later under an
explicit fixed PriceTableSnapshot; no profile reserves funds or quota.

These are declared upper bounds, not actual measurements. A provider may
understate them; compatibility tests, implementation attestation, runtime
metering, and BudgetReservation remain separate evidence/authority layers.
