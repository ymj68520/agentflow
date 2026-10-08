# Phase 4A.2 — Typed Failure Row: Decision Register

Phase: 4A.2 (Typed Failure Row / Failure Signature Algebra). Scope: the Φ
dimension of `Computation<I, O, Φ, ε, R>` — the static declaration side
only. Frozen boundaries respected: no `ComponentContract`, no runtime
`FailureValue`, no new HashV1 domain, every pre-existing wire surface
unchanged (semantic-ir.afidl byte-identical, effect corpus at revision 2,
all Phase 1–4A.1B corpora zero-drift).

---

## D-1 Nominal failure identity

**Decision:** a failure type is the full triple
`(SpecificationId, TypeName, pinned TypeSchemaDigest)`. Same short name
under a different specification, or the same logical name with a different
pinned digest, is a DIFFERENT failure type — never silently compatible, no
implicit upcast. The payload reuses the semantic core's `TypeSchemaRef` by
exact cross-module import; a bare-string identity was rejected outright.

**Rejected:** error-message / error-code substring matching, exception
class-name reflection, provider-specific mapping (later Worker Failure
Mapping); a short-name or record-shape similarity rule (unsound
containment). Any type with a declared AFIDL schema MAY be pinned; it has
a typed failure role exactly by being listed in a row. The AFIDL1
primitive universe is untouched.

## D-2 Closed-row decision

**Decision:** `FailureRow = {F1..Fn}` is a closed set. The Effect Row's
open/residual/unknown wire states are deliberately NOT copied — there is
no `FailureRow::unknown` variant, no residual ceiling, no occurrence
bounds (a row is a set of TYPES; ITERATE does not multiply them).

**Rejected:** reusing the ε wire shape (Φ has no per-type quantification
this phase; introducing it now would freeze granularity the contract
phase may not want); an `unknown` wire variant (unknown is an ANALYSIS
result, see D-10).

## D-3 Schema pinning rule

**Decision:** validation resolves each pin against the explicit
`TypeSchemaUniverse`: syntax first (`invalid_failure_type`), existence
(`unknown_failure_schema`), digest match
(`failure_schema_digest_mismatch`). The distinct kinds exist because
"same name, different version" is a materially different authoring error
from "no such type". No latest-schema lookup exists anywhere in the
dependency set.

## D-4 Canonical duplicate policy

**Decision:** declared rows REJECT duplicates (`duplicate_failure_type` —
a declaration is a set, not a multiset); the algebra's union MAY
deduplicate. Canonical order = full nominal identity in byte order
(specification, name, digest hex); declaration order and map iteration
order never reach the canonical ACE1 bytes; canonicalization is
idempotent. No digest is computed over the row (see D-12).

## D-5 Containment / subtyping boundary

**Decision:** `Φa ⊆ Φb` is exact nominal membership. No subtype relation,
no compatibility inference, no structural similarity — a future
compatibility system requires an explicit Compatibility Proof and is out
of scope. This is also the future component-substitution check
("Failure 不扩张"): `Effects(new) ⊑ Effects(old)` has its ε counterpart in
`af-effect`; the Φ half is `contains`.

## D-6 HANDLE subtraction rule

**Decision:** `remove_handled(Φ, H)` requires `H ⊆ Φ` (each failure
matcher matches an exact `TypeSchemaRef` the child declares) and returns
`Φ \ H`; a handled-but-undeclared type is `failure_not_declared`, never
pretended matched. The operation is a static residual computation after a
full type check — it does NOT mean a HANDLE executed or will succeed. The
frozen `HandleMatch` shape is unchanged and read-only-checked:
`failure: FailureMatch{failure_type: core.TypeSchemaRef}` beside
`timed_out: unit`; no `cancelled` matcher exists; two failure matchers on
one nominal type stay forbidden by the frozen Phase 3B disjointness rule.

## D-7 Handler failure propagation

**Decision:** `Φ_result = (Φ_child \ H) ∪ Φ_handler_1 ∪ Φ_handler_2 ∪ …`.
Handlers may fail themselves; their rows union into the result and are
never treated as "automatically handled". Unhandled child failures
survive verbatim. This is the static propagation formula, not the HANDLE
runtime state machine.

## D-8 Retry failure propagation

**Decision:** RETRY preserves the child row verbatim. Retry attempts are
NOT a proof that business failure is impossible; only a future
Recovery/Replay Contract with explicit guarantees may narrow the row.
TIMEOUT likewise preserves the child row and adds nothing: `TimedOut` is
a Control Outcome, and no `Failure<TimedOut>` is representable.
COMPENSATION_SCOPE passes the forward row through: compensation success
never erases forward failure facts and never maps recovery-blocked states
into business failures. AWAIT contributes ∅ (waiting invents no failure
type). ITERATE passes the body row through (types do not scale with
repetitions).

## D-9 Failure vs Control Outcome distinction

**Decision:** the six planes are separated structurally, not by
convention: the crate and the AFIDL module contain no symbol for
`TimedOut`, `Cancelled`, `RecoveryBlocked`, `TaskAttemptLost` or
`RuntimeIncident` (static-audited, comments excluded), and no fake
`TimedOutFailure` / `WorkerLostFailure` schema was registered for
testing. `Φ = ∅` is documented and frozen in the corpus as "no typed
business failure declared" — never "guaranteed success".

## D-10 Unknown / Unproven representation

**Decision:** an unprovable failure upper bound is the analysis-level
`StaticFailureEnvelope::Unproven` — no wire representation. Demanding a
row from it fails closed (`failure_row_unproven`). INVOKE and CALL_SUBFLOW
have NO propagation function this phase: their Φ resolves from the full
Component/Subflow Failure Contract in a later stage, and "target has no
explicit FailureRow → assume Φ = ∅" is forbidden (Phase 3D derives
inputs/outputs/resources only).

## D-11 Union/join relationship

**Decision:** for closed sets, branch composition IS set union;
`join` exists as the named control-surface alias so the propagation table
reads literally (SEQUENCE/SELECT/PARALLEL/RACE all reduce to ∪, each with
its own function name and corpus case). Union is commutative,
associative, idempotent; ∅ is the identity — all four properties frozen
and cross-checked.

## D-12 No new Hash domain

**Decision:** no `FailureRowDigest`, no `FailureSignatureDigest`; the
HashV1 registry stays at 14 domains (static-audited). Only the new
module's own types carry `TypeSchemaDigest`s (2 ABI rows). The future
ComponentContract will hash as a whole under the frozen
`component-contract` domain.

## D-13 ComponentContract integration deferred

**Decision:** the Φ algebra is parallel and independent of the ε algebra
(`af-failure` does not depend on `af-effect` or `af-ir`); aggregation
into `Computation<I, O, Φ, ε, R>` + F/U/K/X happens in the
ComponentContract stage, which will also supply INVOKE/CALL_SUBFLOW
resolution and the component-substitution checks built on `contains`.
