# Effect Signature (agentflow.effect.signature@1)

Status: frozen (Phase 4A.1). The machine schema authority is
`spec/idl/effect-signature.afidl`; this document is the normative semantics.
The Rust implementation is `crates/af-effect`; the frozen conformance corpus
is `spec/conformance/effect-signature/` (independent Python oracle:
`tools/gen-effect-signature-corpus.py`).

## 1. Static positioning

<a id="effect-static-positioning"></a>

The Effect Signature is the **ε dimension** of the frozen computation model
`Computation<I, O, Φ, ε, R>` (docs §"Task 执行语义与 Effect System"): the
statically declared CEILING of the externally observable interactions a
computation may cause. Read interactions are effects.

```text
Effect = Operation + Resource Selector + Qualifiers
Effect Signature = Capability Row + Resource Scope
                 + Occurrence Bounds + Guards + Openness
```

The signature is a declaration. It is not a credential, not a policy
decision and not an execution capability; nothing in this layer can be
invoked, exchanged or redeemed.

<a id="effect-declaration-not-authorization"></a>

The lifecycle separation is normative:

```text
Effect Declaration  →  Effect Intent  →  Policy Decision  →  Effect Grant
                    →  Execution      →  Effect Receipt
Declared Effects ≠ Granted Effects ≠ Dispatched Effects ≠ Observed Effects
Actual Effects ⊆ Granted Effects ⊆ Declared Effects
```

`Dispatched ⊆ Granted ⊆ Declared` is a Runtime / Effect Gateway invariant of
LATER phases. This phase materializes ONLY the declaration end. An effect
signature MUST NOT be treated as authorization, and no API of the
implementation converts a signature into authority. The unknown row (⊤) and
every other row state are static analysis facts, never grants; there is no
EffectGrant, EffectIntent, EffectReceipt, Effect Gateway, Credential Broker
or Provider Dispatch in this layer, and their names MUST NOT appear in the
crate.

## 2. The effect atom

<a id="effect-atom"></a>

An `EffectAtom` is exactly:

```text
EffectAtom = (EffectOperationRef, ResourceSelector, qualifiers)
```

- `EffectOperationRef` — the logical operation identity (section 3);
- `ResourceSelector` — the logical acting range (section 4);
- `qualifiers` — a closed qualifier list (section 2.1).

The atom's canonical identity is the exact structural tuple of these three
parts (operation catalog, operation name, requirement port, the sorted
exact scope-binding set, the sorted qualifier-family list). Two entries with
the same canonical identity declare THE SAME effect.

### 2.1 Qualifiers

<a id="effect-qualifiers"></a>

Qualifiers are a CLOSED model: the only family this phase is
`idempotency`, with members `natural` and `none` (AFIDL
`EffectIdempotencyQualifier`, frozen `value_no`s 1 and 2). At most one
member of a family may appear in one atom's qualifier list; a list carrying
two members of one family is invalid (`invalid_effect_qualifier`). An empty
qualifier list is well formed and means "unqualified".

An idempotency qualifier is a DECLARED behavioral property. This phase gives
it no analytical power: occurrence bounds and retry envelopes are computed
WITHOUT exploiting idempotency (section 10.6). Exploiting it requires the
Replay/Idempotency/Recovery contracts of a later phase.

## 3. Operation identity and the capability catalog

<a id="effect-operation-identity"></a>

An operation identity is `EffectOperationRef = (catalog, name)`:

- `catalog` MUST parse as an AFIDL `SpecificationId` (for example
  `agentflow.test.capability-catalog@1`);
- `name` MUST be NFC text of at most 256 characters formed from ASCII
  lowercase segments `[a-z0-9-]+` joined by `.` with no empty segment;
- the full logical identity of the operation is `<catalog>/<name>`. This
  tuple — not a free-text description — is the operation's logical identity.

Malformed identities are rejected with `invalid_effect_operation`.

<a id="effect-capability-catalog"></a>

The final capability catalog is NOT materialized in this phase. Resolution
happens against an EXPLICIT, immutable, in-memory catalog universe: a
catalog is a named set of operation names, inserted idempotently; inserting
a different catalog under the same identity is a collision
(`effect_catalog_collision`). There is no registry lookup, no
latest-version fallback, no filesystem, network or package discovery, and
no default catalog.

<a id="effect-operation-fail-closed"></a>

Resolution is fail-closed: an operation that does not resolve in the
supplied universe is rejected (`unknown_effect_operation`). An unresolved
operation MUST NOT default to any behavior, and containment between two
DIFFERENT operations MUST NOT be proven: this phase defines no operation
hierarchy, so operation compatibility in a containment proof means exact
identity equality (section 8).

## 4. The resource selector

<a id="effect-resource-selector"></a>

Four different things MUST NOT be conflated:

```text
Declared Resource Requirement   (a future ComponentContract row)
Logical Resource Selector       (this section — the only one defined here)
Runtime ResourceHandle          (Runtime, attenuated)
Physical Resource               (deployment)
```

A `ResourceSelector` is `(requirement_port_no, scope_bindings)`:

- `requirement_port_no` MUST be >= 1 (0 is rejected:
  `invalid_resource_selector`). The port is the future ComponentContract
  resource-requirement port the effect acts through; whether that port
  exists in a component contract is NOT decidable in this phase and MUST
  NOT be claimed.
- `scope_bindings` is a set of EXACT constraints: at most one binding per
  key, keys and values non-empty NFC text, duplicate keys rejected. A
  binding means "bounded to exactly this value", never a prefix pattern;
  containment between scopes is exact-binding set inclusion (section 8).

An ambient resource, a raw filesystem root, a raw database connection,
secret material and a concrete worker/provider endpoint are all
UNREPRESENTABLE in a selector; the selector carries no handle, no address
and no credential.

## 5. Occurrence bounds

<a id="effect-occurrence-bounds"></a>

`OccurrenceBounds = (lower: uint64, upper: finite uint64 | unbounded)`:

- `lower >= 0` and `upper >= lower` (a finite upper below the lower bound is
  rejected: `invalid_occurrence_bounds`);
- `unbounded` IS the infinity case: infinity MUST be expressed as the
  explicit `unbounded` ceiling, never as `UINT64_MAX` and never as an
  overflow artifact;
- bounds count LOGICAL effect occurrences of one atom. Physical task
  attempts are a separate concept and appear only in the retry envelope's
  separate attempt bounds (section 10.6). The two counters MUST NOT be
  merged into one.

All occurrence arithmetic is checked. Overflow is an explicit error
(`effect_arithmetic_overflow`) — no wrap-around, no silent saturation
(section 11).

## 6. Guards

<a id="effect-guards"></a>

A guard is a statically understandable declaration that narrows WHEN an
effect may occur. The guard language is closed:

```text
EffectGuard =
  | unconditional
  | input_equals { port_no >= 1, expected: PrimitiveConstant }
```

`input_equals` states: this effect is possible only when component input
port `port_no` equals the exact primitive constant `expected` (the semantic
core's closed `PrimitiveConstant`, reused by exact cross-module identity).
Its port grammar and constant canonicality are validated
(`invalid_effect_guard`); whether the port exists in a component contract is
a later phase's composition check.

The language has no negation, no conjunction, no time, no randomness, no
current policy lookup, no external callback and no free-text predicate. A
condition this language cannot express MUST be declared conservatively as an
unconditional effect (the wider envelope) — never narrowed by assuming an
unverified condition false.

Guard containment is provable in exactly these cases: any guard is within
`unconditional`, and a guard is within itself. `unconditional` under a
condition, or one condition under a different condition, is NOT PROVEN.

## 7. Row states: ∅, ε, ε | ρ, ⊤

The `EffectRow` has exactly four states, and they are never collapsed.

### 7.1 Pure is the closed empty row

<a id="effect-pure"></a>

The closed row with an EMPTY entry list is the ONE representation of pure
(∅). `is_pure` is true exactly for it. Zero-occurrence entries (bounds whose
upper is the finite value 0) are semantically pure and are dropped by the
canonical form (section 13); a row whose entries all drop becomes pure.

### 7.2 The unknown row is not pure

<a id="effect-top"></a>

⊤ (`unknown`) means "this envelope could not be proven". It is a legitimate
static analysis result and a legitimate declaration state, and:

- it is NOT pure;
- it contaminates every algebra composition upward (any composition with a
  ⊤ operand is ⊤);
- it is contained only in ⊤ (section 8);
- it carries no capability material: its canonical form is the literal
  unknown encoding, and nothing about it can be redeemed, dispatched or
  granted.

Treating ⊤ as pure, or as authorization, is a violation.

### 7.3 The bounded-open row

<a id="effect-open-row"></a>

`ε | ρ` (open) declares the known effects plus a RESIDUAL: unknown
operations, but only from a named catalog, only through a named selector,
only within named occurrence bounds. The ceiling must be VERIFIABLE: a
residual with no scope binding AND an unbounded occurrence upper constrains
nothing and is rejected (`unbounded_open_effect_row`). A residual's catalog
must itself be registered in the supplied universe (fail closed).

An open row whose known list and residual list both end up empty after
canonical normalization IS pure (a bounded-open row that bounds nothing).

## 8. Containment (⊑)

<a id="effect-containment"></a>

`εa ⊑ εb` means: every effect `εa` may cause provably falls within the
envelope `εb` declares. The outcome is `Proven` or `Unproven(reason)` —
NOT PROVEN is a first-class result and MUST NOT be treated as proven.

Provable per entry (each `a` entry must find an unused covering `b` slot;
matching is deterministic over canonical order):

- operation: exact identity equality (catalog and name) — no hierarchy, no
  prefix, no "compatible family";
- requirement port: equal;
- qualifiers: exactly equal;
- scope: `a`'s binding set a SUPERSET of `b`'s (narrower or equal scope,
  exact values only — never a string-prefix inference);
- occurrence bounds: `a`'s interval within `b`'s
  (`b.lower <= a.lower ∧ a.upper <= b.upper`, `unbounded` covering all);
- guard: any `a` guard within an `unconditional` `b` guard, or exactly
  equal guards. Everything else is NOT PROVEN.

A `b` residual ceiling covers an `a` entry when the catalogs are equal, the
ports equal, `a`'s scope is narrower or equal, and the occurrence interval
fits. Open rows: an open row with residuals is never proven inside a closed
row; residuals are covered only by residuals. ⊤: everything is contained in
⊤; ⊤ is contained only in ⊤; ∅ (pure) is contained in everything.

The demand-side assertion `require_contained_in` fails closed with
`effect_envelope_unprovable` when the proof is not `Proven`. Each limit
slot is claimed at most once; joint-budget refinement across several `a`
entries sharing one covering declaration is a documented proof limit (it
can only lose provability, never produce a false proof).

<a id="effect-algebra-safety"></a>

The algebra MUST NOT merge two effects it cannot relate into a smaller safe
envelope: whenever a guard, scope or operation relationship is unprovable,
the composition widens conservatively (sections 9-10) or reports NOT
PROVEN — it never pretends a proof succeeded.

## 9. The composition primitives

### 9.1 Accumulation (⊕)

<a id="effect-accumulation"></a>

`εa ⊕ εb` — both envelopes definitely happen (sequential composition).
Per atom key:

```text
[l1,u1] ⊕ [l2,u2] = [l1+l2, u1+u2]        (checked arithmetic)
```

Distinct atom keys are kept side by side. Guards compose conservatively:

- equal unconditional guards: bounds add fully;
- equal conditional guards: `lower = 0` (the condition may be false), the
  guard is kept, uppers add;
- different guards: the merged entry widens to `unconditional` with
  `lower = 0` and added uppers.

Declared duplicates are a validation error (`duplicate_effect_atom`), never
a silent merge; merging same-key atoms is EXCLUSIVELY the algebra's job.

### 9.2 Branch join (⊔)

<a id="effect-branch-join"></a>

`εa ⊔ εb` — two mutually exclusive branches; exactly one runs. Per atom
key the result is the conservative interval covering both branches' legal
cases. A key declared by BOTH branches keeps the guaranteed minimum lower
bound (whichever branch runs delivers at least its own lower bound, hence
at least the minimum) and the covering ceiling `max(u1, u2)`. A key
declared by only ONE branch counts as zero occurrences when the other
branch runs, so its interval is `[0, u]` with that branch's guard kept
(the guard still describes when it can fire). Equal conditions keep their
guard; different conditions widen to `unconditional` — their union is not
expressible. ⊤ contaminates. Join is commutative and idempotent.

### 9.3 Repetition (n ⊗ ε)

<a id="effect-repetition"></a>

`reps ⊗ ε` scales every occurrence interval:
`[l,u] × [rl,ru] = [l·rl, u·ru]` (checked). Guards are per-occurrence
conditions and do not scale. If the repetition upper is unbounded, every
non-zero effect ceiling becomes unbounded: an unprovable repetition count
MUST NOT yield a finite envelope. `0 ⊗ ε` is pure.

## 10. Control-opcode propagation

<a id="effect-control-propagation"></a>

The composition rules for the structured control opcodes are pure functions
over signatures (no `af-ir` runtime state):

<a id="effect-sequence-accumulation"></a>

- SEQUENCE(A, B): `εA ⊕ εB` (section 9.1).

<a id="effect-parallel-accumulation"></a>

- PARALLEL(A, B): `εA ⊕ εB` — both branches run to completion; concurrency
  limits bound simultaneous work, never total occurrences.

<a id="effect-select-branch-join"></a>

- SELECT(A, B): `εA ⊔ εB` (section 9.2).

<a id="effect-race"></a>

- RACE(branches): CONSERVATIVE ACCUMULATION over all contestants,
  `εB1 ⊕ εB2 ⊕ …`. A loser may already have produced effects before
  cancellation, so loser effects are never discarded and race is NOT a
  branch join.

<a id="effect-iterate"></a>

- ITERATE(reps, body): `reps ⊗ εbody` (section 9.3). A foreach over a
  data-dependent source has no statically provable count and MUST use the
  bounds `[0, unbounded)`.

<a id="effect-retry"></a>

- RETRY(child, max_attempts): the physical attempt bounds are
  `[1, max_attempts]`, tracked separately from the logical envelope. The
  logical envelope is conservatively `[1, max_attempts] ⊗ εchild`: WITHOUT
  the Replay/Idempotency/Recovery contracts this phase MUST NOT assume that
  effectful retries do not amplify business effects, and an idempotency
  qualifier does NOT narrow the result. The result records the explicit
  obligation `retry_logical_narrowing_requires_recovery_contract` and
  `logical_narrowing_proven = false`. `max_attempts = 0` is rejected
  (`invalid_occurrence_bounds`).

<a id="effect-compensation"></a>

- COMPENSATION(forward, counter): `εforward ⊕ εcounter`. Forward effects
  REMAIN RECORDED; the counter-effect ADDS new effects. Compensation never
  subtracts from history — the original effect and the counter-effect are
  both facts, and subtraction is unrepresentable in the algebra.

## 11. Unprovable and overflow

<a id="effect-unprovable"></a>

`effect_envelope_unprovable` is the fail-closed answer to a demand for a
proof the model cannot supply (a containment assertion that came back NOT
PROVEN). It is a static validation outcome — not a Workflow FailureValue
and not a Policy denial.

<a id="effect-overflow"></a>

All occurrence arithmetic (adds, multiplications in ⊕ and ⊗) is checked
`u64`/ceiling arithmetic. Overflow produces the explicit error
`effect_arithmetic_overflow`. There is no wrap-around and no silent
saturation; in particular no operation may turn an overflow into a smaller
or finite envelope.

## 12. Canonicalization

<a id="effect-canonicalization"></a>

`canonicalize_effect_signature` is a pure, deterministic function: entries
ordered by exact atom key (catalog, name, port, sorted scope bindings,
sorted qualifier families), scope bindings by (key, value), qualifiers by
family case number, residuals by (catalog, port, scope). Inputs differing
only in ordering have IDENTICAL canonical ACE1 bytes; canonicalization is
idempotent.

The canonical duplicate policy: a declared row with two entries of the same
canonical atom identity is rejected (`duplicate_effect_atom`) — a
declaration is a set, and silently merging duplicates would widen authoring
intent. Merging is the algebra's explicit job (section 9.1). Zero-occurrence
entries are dropped (section 7.1).

The canonical form has ACE1 bytes but deliberately NO digest: no
`EffectSignatureDigest` exists, no HashV1 domain is added, and the
`component-contract` domain remains reserved for the future whole-contract
object. The new AFIDL module's own types carry ordinary `TypeSchemaDigest`s.

## 13. Static validation

<a id="effect-validation"></a>

The validator checks, in order: operation identity grammar; selector
structure (port, binding keys/values, duplicates); qualifier families;
occurrence bounds; guard grammar and constant canonicality; canonical atom
duplicates; residual structure and ceiling verifiability; then catalog
resolution. It MUST NOT call a policy engine, credential broker, effect
gateway, resource adapter or provider — none of them exist in this layer.

The stable rejection kinds are: `invalid_effect_operation`,
`unknown_effect_operation`, `invalid_resource_selector`,
`invalid_effect_qualifier`, `invalid_occurrence_bounds`,
`duplicate_effect_atom`, `invalid_effect_guard`,
`unbounded_open_effect_row`, `effect_envelope_unprovable`,
`effect_arithmetic_overflow`, `effect_catalog_collision`,
`effect_encoding_error`. A static rejection is not a Workflow FailureValue
and not a PolicyDenied.

## 14. Explicit deferred

<a id="effect-deferred"></a>

Deferred by design in Phase 4A.1 (their names MUST NOT appear as
placeholders): ComponentContract and its Resource Requirement Row (so port
existence is unchecked here), ComponentImplementation, INVOKE exact
component contract checking, FailureValue / FailureRow,
ResourceContract, Information Flow, Usage/Budget, RecoveryContract and
compensation execution, ExecutionRequirements, EffectIntent, EffectGrant,
EffectReceipt, Effect Gateway, Credential Broker, Provider Dispatch, the
production capability catalog, dynamic tool/MCP discovery, executable
plans, journal, reducer, runtime, scheduler, worker. This list is the
authorization boundary: completing the Effect Signature grants nothing.
