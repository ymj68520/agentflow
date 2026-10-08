# Failure Row (agentflow.failure.row@1)

Status: frozen (Phase 4A.2). The machine schema authority is
`spec/idl/failure-row.afidl`; this document is the normative semantics.
The Rust implementation is `crates/af-failure`; the frozen conformance
corpus is `spec/conformance/failure-row/` (independent Python oracle:
`tools/gen-failure-row-corpus.py`).

## 1. Static positioning

<a id="fr-positioning"></a>

The Failure Row is the **Φ dimension** of the frozen computation model
`Computation<I, O, Φ, ε, R>` (docs §22.12, §24.15): the CLOSED, NOMINAL
set of typed business failures a computation may surface. `FailureValue`
is Workflow typed ABI; only typed failures enter Φ. The Φ algebra and the
ε algebra (`af-effect`) are parallel and independent; the future
ComponentContract aggregates them.

## 2. The six error planes

<a id="fr-six-planes"></a>

Six planes are strictly separate and never collapsed:

```text
Semantic Failure     ≠ Control Termination   ≠ Recovery State
                     ≠ Operational Condition ≠ Runtime Incident
                     ≠ Platform Degradation
```

Exactly one plane is representable in the Failure Row:

```text
Typed FailureValue → Φ                      (this specification)
TimedOut          → Control Outcome         (SemanticIr HandleMatch)
Cancelled         → Control Outcome         (never a Handle match)
RecoveryBlocked   → Recovery State          (not HANDLE-catchable)
TaskAttemptLost   → Infrastructure Fact     (never Failure<WorkerLost>)
RuntimeIncident   → Incident Plane          (blocks progression)
```

## 3. Nominal failure identity

<a id="fr-nominal-identity"></a>

A `FailureTypeRef` is nominal: its identity is the full triple
`(SpecificationId, TypeName, pinned TypeSchemaDigest)`. Consequences:

- same type name + different specification → DIFFERENT failure types;
- same logical name + different pinned digest → different identities,
  never silently compatible (no implicit upcast).

The payload type reuses the semantic core's `TypeSchemaRef` by exact
cross-module identity — never a bare string, never a redeclared reference
shape. Being a declared AFIDL schema makes nothing a failure: a type has a
typed failure role in a contract exactly by being listed in that
contract's Failure Row. No error-message matching, no error-code substring
matching, no exception class-name reflection and no provider-specific
exception mapping exist in this layer (that is later Worker Failure
Mapping, out of scope). The AFIDL1 primitive universe is untouched.

## 4. Exact schema pinning

<a id="fr-exact-pinning"></a>

Validation resolves every payload pin against an explicit
`TypeSchemaUniverse` (or an equivalent already-verified module set):

- the reference syntax must be valid (`invalid_failure_type`);
- the referenced type must exist (`unknown_failure_schema`);
- the pinned digest must match the universe's computed digest
  (`failure_schema_digest_mismatch`).

There is no latest-schema lookup, no fallback and no implicit upcast; two
legal versions of the same type keep their full nominal identities.

## 5. The closed Failure Row

<a id="fr-closed-row"></a>

`FailureRow = {F1, F2, ... Fn}` is a closed set of nominal failure types.
The row deliberately does NOT copy the Effect Row's open/residual/unknown
wire states — there is no `FailureRow::unknown`, no residual ceiling.

<a id="fr-empty-not-success"></a>

`Φ = ∅` means "no typed business failure is declared". It does NOT mean
the computation succeeds: `TimedOut`, `Cancelled`, `RecoveryBlocked`,
`TaskAttemptLost` and `RuntimeIncident` all remain possible outside Φ.

<a id="fr-unknown-not-empty"></a>

If a static analysis cannot prove a failure upper bound, the result is the
separate analysis-level value **Unproven**
(`StaticFailureEnvelope::Unproven`), never the empty row. Demanding a row
from an unproven envelope fails closed with `failure_row_unproven`.
Unknown is not empty.

## 6. Canonical ordering

<a id="fr-canonical-ordering"></a>

`canonicalize_failure_row` sorts refs by their full nominal identity in
byte order (specification id, then type name, then pinned digest hex).
Input declaration order is not identity and never reaches the canonical
ACE1 bytes; no HashMap iteration order is observable. Canonicalization is
idempotent.

<a id="fr-duplicates-rejected"></a>

A DECLARED row containing the same nominal type twice is rejected
(`duplicate_failure_type`) — a declaration is a set, not a multiset. The
algebra's union MAY deduplicate: the two behaviors are deliberately
separate (validation rejects, algebra normalizes).

## 7. Containment

<a id="fr-containment-exact"></a>

`Φa ⊆ Φb` holds iff every nominal type of `Φa` is declared by `Φb` — all
three identity parts. Containment is never inferred from the same short
name, a similar schema or equal record fields. A future subtype or
compatibility system requires an explicit Compatibility Proof and is not
implemented in this phase (this is also the future component-substitution
check "Failure 不扩张").

## 8. Union

<a id="fr-union-algebra"></a>

`Φa ∪ Φb` deduplicates by nominal identity and returns the canonical
order. It is commutative, associative and idempotent:

```text
A ∪ B = B ∪ A
(A ∪ B) ∪ C = A ∪ (B ∪ C)
A ∪ A = A
```

`Φ ∪ ∅ = Φ` and `∅ ⊆ Φ`: the empty row is the algebra's identity.
Branch composition (SELECT / PARALLEL / RACE) is exactly set union.

## 9. HANDLE residual computation

<a id="fr-handle-residual"></a>

<a id="fr-handled-declared"></a>

`remove_handled(Φ, H)` computes `Φ \ H` as a static residual AFTER a full
type check; it does not mean a HANDLE executed or will succeed. `H` must
be a subset of `Φ`: a HANDLE failure matcher matches an exact
`TypeSchemaRef` the child actually declares; anything else is
`failure_not_declared` — a handled-but-undeclared type is never pretended
matched.

<a id="fr-unhandled-propagate"></a>

Unhandled failures propagate verbatim: `Φ \ H` keeps every declared type
not in `H`.

<a id="fr-handler-propagate"></a>

Handler failures propagate: handlers may fail themselves, and their rows
union into the result:

```text
Φ_result = (Φ_child \ H) ∪ Φ_handler_1 ∪ Φ_handler_2 ∪ …
```

A handler failure is never mistaken for "automatically handled". This is
the static propagation formula, not the HANDLE runtime state machine.

The frozen Semantic IR matcher shape is authoritative and unchanged:
`HandleMatch { 1 failure: FailureMatch; 2 timed_out: unit }` with
`FailureMatch { 1 failure_type: core.TypeSchemaRef }`. The algebra's
failure matcher is exactly that `TypeSchemaRef` — the `timed_out` matcher
carries no payload type and matches no failure.

## 10. Control failure propagation

<a id="fr-control-propagation"></a>

Pure functions over rows (no runtime, no `af-ir` dependency):

| Control | Static Failure Row |
|---|---|
| SEQUENCE | union of all children that may run |
| SELECT | union over all selectable branches |
| PARALLEL | union over all branches |
| RACE | union over all contestants (losers may have failed) |
| ITERATE | body row verbatim — types do not multiply with repetitions |
| RETRY | child row preserved (see below) |
| TIMEOUT | child row preserved; TimedOut never enters Φ |
| HANDLE | `(Φ_child \ H) ∪ Φ_handlers` (section 9) |
| SCOPE | child row |
| COMPENSATION_SCOPE | child row; forward failures are never erased |
| CALL_SUBFLOW | deferred: resolves from the Subflow Failure Contract |
| INVOKE | deferred: resolves from the full ComponentContract |
| AWAIT | ∅ — waiting invents no business failure type |

<a id="fr-retry-preserves"></a>

`retry attempts ≠ proof that business failure is impossible`. RETRY
preserves the child's row verbatim; only a future Recovery/Replay Contract
with explicit guarantees may narrow it. COMPENSATION_SCOPE never maps a
recovery-blocked state into a business failure and never deletes a forward
failure because compensation succeeded.

## 11. Control-outcome exclusion

<a id="fr-timedout-excluded"></a>

`TimedOut` is a Control Outcome (the frozen `HandleMatch::timed_out`).
It is never modeled as a failure type — no `Failure<TimedOut>` exists —
and never enters Φ. TIMEOUT composition therefore adds no type to the
child row.

<a id="fr-cancelled-excluded"></a>

`Cancelled` is a Control Outcome. Parent cancellation is never a normal
failure, no `cancelled` matcher exists in the frozen HANDLE set, and the
algebra provides no way to introduce a cancellation failure type.

<a id="fr-recoveryblocked-excluded"></a>

`RecoveryBlocked` is a Recovery State. It is not a Workflow failure and
cannot be caught by an ordinary HANDLE; it never enters Φ and is never
mapped into a business failure type by compensation or retry logic.

<a id="fr-incidents-excluded"></a>

`TaskAttemptLost` is an Infrastructure Fact (never automatically
`Failure<WorkerLost>`) and `RuntimeIncident` (invariant violations,
journal mismatches, protocol conflicts, durability violations) lives on
the Incident Plane, blocking semantic progression instead of disguising
itself as a Workflow Failure. Operational conditions (capacity, placement)
and platform degradation are likewise outside Φ. None of these names
exists as a failure type anywhere in the model, and no fake
`TimedOutFailure` / `WorkerLostFailure` schema is registered for testing.

## 12. Unknown / Unproven

See section 5: the closed row has no unknown wire state; Unproven is the
analysis-level result with the `failure_row_unproven` fail-closed demand,
and INVOKE / CALL_SUBFLOW resolution is deferred, not assumed empty.

<a id="fr-deferred-contract-resolution"></a>

Phase 3D derives a subflow's inputs/outputs/resources but the
SemanticIrDocument carries no Failure Signature; it is therefore NOT
claimed that a full Φ can be derived from an arbitrary target IR, and
`target has no explicit FailureRow → assume Φ = ∅` is forbidden.
CALL_SUBFLOW and INVOKE failure propagation waits for the Component
Contract composition stage. Phase 3D's entire static interface stays
unchanged.

## 13. Error model

<a id="fr-error-model"></a>

Stable rejection kinds: `invalid_failure_type`,
`unknown_failure_schema`, `failure_schema_digest_mismatch`,
`duplicate_failure_type`, `failure_not_declared`,
`failure_row_unproven`, `failure_encoding_error`. These are compile-time
results of the contract layer — never Workflow `FailureValue`, never
`PolicyDenied`, never `RuntimeIncident`.

## 14. Digest boundary and deferred

<a id="fr-hash-boundary"></a>

No `FailureRowDigest` and no `FailureSignatureDigest` exist; the HashV1
domain registry is unchanged. Only the new AFIDL module's own types carry
ordinary `TypeSchemaDigest`s. The future ComponentContract will hash as a
whole under the frozen `component-contract` domain.

<a id="fr-deferred"></a>

Deferred by design (structurally absent, static-audited):
FailureValue runtime payloads, TerminalOutcome, RuntimeIncident ABI,
worker error transport, failure mapping adapters, ComponentContract,
ResourceRequirementRow, InformationFlow, Usage, RecoveryContract,
ExecutionRequirements, INVOKE/HANDLE/RETRY runtime, executable plan,
journal, reducer, scheduler, worker, subtype/compatibility proofs, and
any modification to the 13 frozen opcode wire schemas.
