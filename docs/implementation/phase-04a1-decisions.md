# Phase 4A.1 — Effect Signature: Decision Register

Phase: 4A.1 (Effect Signature / Capability Row / Effect Algebra).
Scope: the ε dimension of `Computation<I, O, Φ, ε, R>` — the static
declaration side only. Frozen boundaries respected: no `ComponentContract`,
no grant/receipt/gateway objects, no runtime, and every pre-existing wire
surface unchanged (semantic-ir.afidl byte-identical, HashV1 domains
untouched, all Phase 1–3D corpora zero-drift).

Each entry states the decision, the alternatives rejected, and why the
chosen option never lets an effect envelope UNDERSTATE possible effects.

---

## D-1 Effect Atom exact identity

**Decision:** an atom is exactly
`(EffectOperationRef, ResourceSelector, qualifier list)` and its canonical
identity is the structural tuple (catalog, name, requirement port, the
sorted exact scope-binding set, the sorted qualifier-family list).

**Rejected:** adding occurrence bounds or guards to the identity (bounds and
guards describe WHEN/HOW OFTEN an atom occurs; two entries with the same
atom and different bounds are the same effect declared twice — a
`duplicate_effect_atom` validation error, never a new identity); a
hash-based identity (no new digest domain this phase, D-14).

**Materialization note:** the frozen docs give
`Effect = Operation + Resource Selector + Qualifiers`; the tuple-to-field
mapping above is this phase's materialization of that definition.

## D-2 Capability operation representation

**Decision:** `EffectOperationRef { catalog: SpecificationId-form, name:
dotted ASCII [a-z0-9-] path }`; the full logical identity is
`<catalog>/<name>`. Resolution goes through an explicit, immutable,
in-memory `EffectCatalogUniverse` whose catalogs are NAME SETS — presence is
exactly what makes an operation resolvable, nothing more is claimed.
Unknown ⇒ `unknown_effect_operation`, fail closed; no default catalog, no
latest fallback, no discovery. There is no operation hierarchy: containment
between DIFFERENT operations is never provable this phase (exact equality
only).

**Rejected:** a registry/network/package lookup (unrepresentable in the
crate's dependency set, mirroring the af-idl/af-ir universe pattern);
free-text operation descriptions (unverifiable); a hierarchy or wildcard
scheme (would create "provable" containments with no frozen semantics
behind them — an understatement risk); embedding mechanism capabilities
(`network.egress`, `credential.use`) as a separate subsystem — they are
ordinary catalog entries.

**Materialization note:** the production capability catalog is not
materialized (explicitly deferred); the corpus's
`agentflow.test.capability-catalog@1` is a TEST catalog and is never
presented as a production universe.

## D-3 ResourceSelector representation

**Decision:** `{ requirement_port_no ≥ 1, scope_bindings }` where bindings
are an exact-value set: at most one binding per key, non-empty NFC text,
duplicate keys rejected. Scope containment in a proof = binding-set
superset ("more bindings = narrower"), exact value equality only — never a
string-prefix inference.

**Rejected:** raw ResourceHandle / filesystem root / DB connection / secret
/ worker endpoint (not representable — the selector is the second of the
four resource concepts; requirement row, runtime handle and physical
resource stay future); unconstrained ambient resources (port ≥ 1 is
mandatory); prefix patterns (unverifiable containment).

**Boundary:** whether the port exists in a ComponentContract is NOT
decidable here and is not claimed anywhere (the Resource Requirement Row is
a later phase).

## D-4 Qualifier model

**Decision:** one closed family this phase — `idempotency` with members
`natural` and `none` (frozen `value_no`s 1, 2), wrapped in the closed
variant `EffectQualifier` so further families are additive cases. At most
one member per family per atom. An empty qualifier list is well formed
("unqualified"). An idempotency qualifier has NO analytical power this
phase: occurrence bounds, retry envelopes and containment treat qualifier
sets as exact-identity material only.

**Rejected:** `idempotency_keyed` (its semantics require a key-space
contract that does not exist; declaring it would invite premature
non-amplification assumptions); `Map<String, Any>` qualifiers (forbidden by
the task and by the wire-model style).

## D-5 Occurrence bounds representation

**Decision:** `OccurrenceBounds { lower: uint64, upper:
OccurrenceCeiling }` with `OccurrenceCeiling = finite(uint64) |
unbounded(unit)`. `unbounded` IS infinity — never `UINT64_MAX`, never an
overflow artifact. Bounds count LOGICAL effect occurrences; physical task
attempts appear only in the retry envelope's separate `attempts` bounds
(D-12).

**Rejected:** `option<uint64>` (None-as-infinity conflates "maybe" with
"infinity" and makes `UINT64_MAX` ambiguity tempting); saturating
arithmetic (silently shrinks an upper bound — an understatement, D-11).

## D-6 Guard model

**Decision:** closed language `unconditional | input_equals{port_no ≥ 1,
expected: core.PrimitiveConstant}` — the semantic core's constant model is
reused by exact cross-module import (the first effect-layer use of the
Phase 2C import on the declaration side). Guard containment: any guard ⊑
`unconditional`, G ⊑ G; everything else NOT PROVEN. Guards narrow WHEN an
effect may occur, never its magnitude; conditions the language cannot
express must be declared as unconditional (wider) effects.

**Rejected:** free-string predicates (indeterminable); negation/conjunction
(expressing `A ∧ ¬B` pairs would invite assuming an unverified condition
false to shrink upper bounds); time/random/policy guards (non-static).

## D-7 Open-row ceiling

**Decision:** a bounded-open row = known entries + residual list
`{catalog, selector, bounds}`. A residual's ceiling must CONSTRAIN
something: at least one scope binding OR a finite occurrence upper; a
residual with neither proves nothing and is rejected
(`unbounded_open_effect_row`). A residual's catalog must itself be
registered (fail closed). An open row whose known list and residual list
both normalize away IS pure.

**Rejected:** requiring BOTH a binding and a finite upper (would outlaw the
honest declaration "any operation of this catalog on exactly this repo,
infinitely often" — bounded scope, unbounded count); accepting
unconstrained+unbounded residuals (an unbounded grant-shaped object with
zero verifiable content).

## D-8 Top / unprovable distinction

**Decision:** ⊤ (`EffectRow::unknown`) is a declarable analysis state:
`is_pure` is false, it contaminates every composition upward, it is
contained only in ⊤, and its canonical form is the literal
`{"row":{"unknown":null}}` — frozen in the corpus precisely to show it
carries no capability material. `require_contained_in` maps NOT PROVEN to
the structured error `effect_envelope_unprovable`; the pure algebra returns
`Proof::Proven/Unproven(reason)` and never throws on unprovable
relationships. The four row states ∅ / ε / ε|ρ / ⊤ are distinct enum cases
and never collapse.

**Rejected:** modeling ⊤ as an error (unprovable envelopes are legitimate
static analysis RESULTS — e.g. an open-ended discovery process); modeling
it as an empty row (the classic "unknown ⇒ pure" hole the task forbids).

## D-9 Canonical duplicate policy

**Decision:** a DECLARED row with two entries of the same canonical atom
identity is REJECTED (`duplicate_effect_atom`) — a declaration is a set and
silently merging duplicates would widen authoring intent. Merging same-key
atoms is exclusively the algebra's explicit job (⊕/⊔). Canonicalization is
otherwise order-erasing: entries by atom key, scope bindings by (key,
value), qualifiers by family case number, residuals by (catalog, port,
scope); atom INTERNALS are normalized too (binding/qualifier input order
never reaches the canonical bytes). Zero-occurrence entries are
semantically pure and dropped; a row that drops empty becomes ∅.

**Rejected:** merging duplicates into a union interval at validation time
(conflates declaration with algebra and hides authoring mistakes); keeping
input order in the canonical bytes (would break "same semantic signature →
same canonical bytes").

**Materialization note:** the task allowed "merge OR reject"; this is the
recorded choice.

## D-10 Containment proof limits

**Decision:** entry covering requires exact operation equality, equal
qualifiers, equal port, scope binding-superset, interval containment and
guard implication; each limit slot is claimed AT MOST ONCE (deterministic
first-match over canonical order). A `b` residual ceiling covers an `a`
entry with catalog+port equality, scope containment and interval fit.
Open-with-residual ⊑ closed is never provable; residuals are covered only
by residuals.

**Rejected:** summing budgets across several `a` entries sharing one
covering declaration (would produce proofs like "2+2 ≤ 3" — false proofs);
reusing one `b` entry for many `a` entries without accounting. The 1:1 rule
can only LOSE provability (returns NOT PROVEN where a joint-budget proof
would succeed), never fabricate one — joint-budget refinement is recorded
as a future refinement, not a silent hole.

## D-11 Algebra overflow policy

**Decision:** all occurrence arithmetic (⊕ interval addition, ⊗ interval
multiplication, ceiling arithmetic) is checked; overflow raises the
explicit `effect_arithmetic_overflow` error. No wrap-around, no silent
saturation, and no "conservative infinity" fallback — an overflow is a
signature the analyzer refuses to guess about.

**Rejected:** saturating to `u64::MAX` (that constant must remain
meaningless — D-5); silently widening to `unbounded` (would let authors
launder absurd bounds into unbounded envelopes without noticing).

## D-12 Retry amplification deferred boundary

**Decision:** `retry_envelope(child, max_attempts)` returns a struct with
(1) the conservative LOGICAL envelope `[1, max_attempts] ⊗ ε_child`, (2) the
separate PHYSICAL `attempts` bounds `[1, max_attempts]`, (3)
`logical_narrowing_proven = false` (always, this phase), and (4) the
explicit obligation
`retry_logical_narrowing_requires_recovery_contract`. An idempotency
qualifier does NOT narrow the result. `max_attempts = 0` is rejected.

**Rejected:** exploiting `natural`/`none` qualifiers to keep logical bounds
flat (exactly the assumption the task forbids before Replay/Idempotency/
Recovery contracts exist); collapsing attempts into the logical bounds (the
frozen docs require the two cardinals to stay distinct).

## D-13 Authorization non-equivalence

**Decision:** the declaration/authorization boundary is enforced
structurally: there is no EffectGrant/EffectIntent/EffectReceipt/Effect
Gateway/Credential Broker/Provider Dispatch symbol, no execution surface,
and no API converts a signature into authority (`contains` /
`require_contained_in` are pure analysis; a future ComponentContract stage
consumes them). `Dispatched ⊆ Granted ⊆ Declared` is recorded as a
Runtime/Gateway invariant of later phases. The static surface audit fails
the build if any forbidden symbol appears.

**Rejected:** a "capability token" view of signatures, any redeem/claim
API, and embedding credentials or endpoints in atoms (all structurally
unrepresentable, D-2/D-3).

## D-14 No new HashV1 domain

**Decision:** the canonical effect signature has canonical ACE1 bytes and
deliberately NO digest: no `EffectSignatureDigest`, no new HashV1 domain
(the registry stays at 14 frozen domains), no digest call anywhere in
`af-effect` (static-audited). The new AFIDL module's own types carry
ordinary TypeSchemaDigests, and the future ComponentContract will hash as a
whole under the already-frozen `component-contract` domain.

**Rejected:** introducing an `effect-signature` domain now (it would
pre-register identity for an object — the whole contract — that does not
exist yet, and would freeze a granularity the contract phase may not want).

## D-15 Branch-join lower bounds (recorded during implementation)

**Decision:** in `εa ⊔ εb`, a key declared by BOTH branches keeps
`[min(l1,l2), max(u1,u2)]` (whichever branch runs delivers at least its own
lower, hence at least the minimum); a key declared by only ONE branch
becomes `[0, u]` with its guard kept. Equal conditions keep their guard;
different conditions widen to `unconditional`. Join is commutative and
idempotent. Residual ceilings join as `[0, max(u1,u2)]`.

**Why recorded:** an earlier draft zeroed ALL lower bounds in joins; that
is sound but understates guaranteed minima for both-declared keys and breaks
idempotence. The corrected rule is the tightest sound interval and is
frozen in `effect/algebra/join-idempotence` and `join-commutativity`.

## D-16 Control-propagation surface

**Decision:** control rules are pure functions over signatures in
`af_effect::control`: SEQUENCE/PARALLEL = ⊕, SELECT = ⊔, RACE =
conservative accumulation over ALL contestants (losers may have acted —
race is NOT a join), ITERATE = `reps ⊗` (a data-dependent foreach must use
`[0, unbounded)`), RETRY per D-12, COMPENSATION = forward ⊕ counter
(subtraction is unrepresentable). No `af-ir` dependency: the IR-level
structural rules stay in `af-ir`; this module consumes already-composed
branch envelopes.

**Rejected:** wiring the propagation into `af-ir`'s computation walk now
(the task scopes 4A.1 to the standalone, independently testable semantics;
the walk belongs to the ComponentContract/control analysis phase).
