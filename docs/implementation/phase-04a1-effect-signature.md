# Phase 4A.1 — Effect Signature / Capability Row / Effect Algebra: Report

Date: 2026-10-08. Scope: the ε dimension of
`Computation<I, O, Φ, ε, R>` — the static DECLARATION side only. This
report follows the 18-item format requested by the phase task. Decisions:
`docs/implementation/phase-04a1-decisions.md` (D-1…D-16).

---

## 1. Effect Signature AFIDL types and count

New module `spec/idl/effect-signature.afidl`
(`schema agentflow.effect.signature @1`, importing
`agentflow.semantic.core@1 as core`). 20 declared types, 15 of them
numbered (32 ABI rows):

| type | kind | numbering |
|---|---|---|
| `EffectOperationRef` | record | 2 fields |
| `ScopeBinding` | record | 2 fields |
| `ResourceSelector` | record | 2 fields |
| `EffectIdempotencyQualifier` | enum | 2 values |
| `EffectQualifier` | variant | 1 case |
| `OccurrenceCeiling` | variant | 2 cases |
| `OccurrenceBounds` | record | 2 fields |
| `EffectGuardInputEquals` | record | 2 fields |
| `EffectGuard` | variant | 2 cases |
| `EffectAtom` | record | 3 fields |
| `EffectEntry` | record | 3 fields |
| `EffectResidual` | record | 3 fields |
| `EffectOpenRow` | record | 2 fields |
| `EffectRow` | variant | 3 cases |
| `EffectSignature` | record | 1 field |

Plus 5 nominal string aliases (`CatalogId`, `OperationName`,
`RequirementPortNo`, `ScopeKey`, `ScopeValue`). The guard payload reuses
`agentflow.semantic.core@1/PrimitiveConstant` by exact cross-module import.
AFIDL corpus revision 5 → 6 (purely additive: every pre-existing case and
TypeSchemaDigest verified unchanged).

## 2. Effect Atom identity

`EffectAtom = (operation, resource_selector, qualifiers)`; canonical
identity = (catalog, name, requirement_port_no, sorted exact scope
bindings, sorted qualifier families). Occurrence bounds and guards are NOT
part of the identity: same atom with different bounds is a duplicate
declaration (`duplicate_effect_atom`), never a new effect (D-1, D-9).

## 3. ResourceSelector and Guard model

Selector: `requirement_port_no ≥ 1` plus exact-binding scope constraints
(one binding per key, NFC text, exact values, no prefixes, no ambient
resources, no handles/roots/connections/secrets/endpoints). Whether the
port exists in a contract is explicitly not decidable here.

Guards: closed language `unconditional | input_equals{port_no ≥ 1,
expected: PrimitiveConstant}`. Guards narrow when an effect may occur,
never its magnitude; no negation/time/random/policy/free-text;
inexpressible conditions widen to unconditional. Guard containment: any ⊑
unconditional, G ⊑ G, else NOT PROVEN.

## 4. Occurrence Bounds

`[lower: uint64, upper: finite | unbounded]`, `unbounded` IS the explicit
infinity (never `UINT64_MAX`). Validated `upper ≥ lower`; impossible
intervals rejected (`invalid_occurrence_bounds`). Bounds count LOGICAL
occurrences; physical attempts are a separate counter (retry envelope
`attempts`). All arithmetic checked (D-11).

## 5. Closed / Open / Top expression

`EffectRow = closed(entries) | open(known, residual) | unknown` with the
empty closed row as the ONE pure form (∅); zero-occurrence entries are
semantically pure and dropped at canonical form. Open rows carry a residual
ceiling `{catalog, selector, bounds}` that must constrain something (a
scope binding or a finite upper) — otherwise
`unbounded_open_effect_row`; an open row that bounds nothing normalizes to
pure. ⊤ is a first-class analysis state: not pure, upward-contaminating,
contained only in ⊤, canonical bytes the literal `{"row":{"unknown":null}}`
(frozen in the corpus to show it carries no capability material).

## 6. Canonical Effect Row

`canonicalize` is deterministic and idempotent: entries ordered by atom
key, scope bindings by (key, value), qualifiers by family case number,
residuals by (catalog, port, scope); atom internals normalized; declared
duplicates rejected (merging is the algebra's exclusive job). Same
semantic signature → identical canonical ACE1 bytes, encoded against
`agentflow.effect.signature@1/EffectSignature` via the schema-derived
shape. NO digest is computed (D-14).

## 7. Containment / Accumulation / Branch Join / Repetition

- **⊑ (contains)** — per-entry proof: exact operation equality, equal
  qualifiers, equal port, scope binding-superset, interval containment,
  guard implication; one limit slot claimed at most once; residual ceilings
  cover entries under catalog+port+scope+interval fit; residuals covered
  only by residuals; ∅ ⊑ everything; everything ⊑ ⊤; ⊤ ⊑ only ⊤. Outcome
  `Proven | Unproven(reason)` with stable reasons
  (`unknown_row`, `residual_inside_closed`, `entry_uncovered`,
  `residual_uncovered`); the demand-side `require_contained_in` fails
  closed as `effect_envelope_unprovable`. Joint-budget refinement across
  shared covering slots is a documented proof limit (can only lose
  provability) — D-10.
- **⊕ (accumulate)** — `[l1,u1] ⊕ [l2,u2] = [l1+l2, u1+u2]` per atom key,
  checked; distinct atoms kept side by side; guards compose conservatively
  (lower bounds add only across unconditional entries; differing guards
  widen to unconditional); ∅ is the two-sided identity (frozen).
- **⊔ (branch_join)** — both-declared keys keep
  `[min(l), max(u)]` (guaranteed-minimum lower), one-sided keys become
  `[0, u]` with the guard kept; differing conditions widen to
  unconditional; commutative and idempotent (both frozen); residuals join
  as `[0, max(u)]`.
- **n ⊗ ε (repeat)** — interval scaling `[l·rl, u·ru]`, checked; guards do
  not scale; unbounded repetition count ⇒ every non-zero ceiling becomes
  unbounded; `0 ⊗ ε` is pure.

## 8. Control Effect propagation

Pure functions in `af_effect::control`, no `af-ir` dependency:
SEQUENCE = ⊕, PARALLEL = ⊕ (concurrency never changes totals), SELECT = ⊔,
RACE = conservative accumulation over ALL contestants (loser effects
retained — frozen), ITERATE = `reps ⊗` (frozen with `[3,5] ⊗ [1,2] =
[3,10]`), RETRY per item 9 below (frozen `[1,3] ⊗ [1,2] = [1,6]`),
COMPENSATION = forward ⊕ counter (frozen `[2,3] ⊕ [1,1] = [3,4]` — no
subtraction).

## 9. Unprovable / Overflow handling

Unprovable: `Proof::Unproven(reason)` in the algebra (never silently
proven); `effect_envelope_unprovable` on the fail-closed assertion
surface. Overflow: checked u64/ceiling arithmetic raising
`effect_arithmetic_overflow` (frozen: `4 ⊗ (2^64−1−1)`), no wrap-around,
no silent saturation, no laundering into `unbounded`.

## 10. Specification / Rule / ABI statistics

- Specifications: 14 → **16** (`agentflow.effect.signature@1` schema,
  `agentflow.effect.signature.semantics@1` semantics).
- Rule area: **EFFECT** added to the closed area table.
- Rules: 168 → **186** (`AF-EFFECT-0001…0018`, all `must`, each with
  exactly one normative anchor; every rule carries conformance refs into
  the corpus or suites; missing coverage 0).
- Suites: 28 → **33** (SUITE-EFFECT-CATALOG-EXPLICIT-UNIVERSE,
  -DECLARATION-NOT-AUTHORIZATION, -NO-RUNTIME, -RETRY-BOUNDARY,
  -WIRE-FREEZE).
- ABI: 232 → **264 active** (+32 effect-signature rows); 0 retired, 0
  reserved. No existing RuleId, ABI assignment or SpecificationId changed.

## 11. Golden / Negative / Algebra corpus counts

42 frozen cases under `spec/conformance/effect-signature/` (corpus
revision 1), every file carrying its fixture catalog block:

- golden: **11** (pure-empty-row, single-read, single-write,
  multiple-distinct, occurrence-bounds, bounded-row, bounded-open-row,
  guarded, resource-scope, zero-occurrence-entry, canonical-reordering) —
  each freezing canonical ACE1 bytes + classification + is_pure;
- negative: **12** (unknown/invalid operation identities, zero port,
  duplicate scope key, lower > upper, conflicting qualifiers, invalid
  guard port, duplicate atom, unbounded open row, and the two top-row
  observations: not-pure, not-authorization with the frozen literal bytes
  and the fail-closed containment error);
- algebra: **15** (empty identity, associativity, join commutativity and
  idempotence, repeat 0/1/n, containment reflexivity and transitivity,
  sequence, parallel, race-loser-retained, retry amplification,
  compensation-no-subtraction, overflow) with frozen results and frozen
  property groupings;
- equivalence: **4** (entry reordering, scope-binding reordering,
  zero-occurrence ≡ pure, residual reordering).

## 12. Independent Oracle implementation scope

`tools/gen-effect-signature-corpus.py` re-implements from scratch (no Rust
calls): grammar checks (SpecificationId, operation names, selectors,
qualifiers, bounds, guards), fail-closed catalog resolution, canonical
identity + canonicalization (including atom-internal normalization and
zero-occurrence dropping), occurrence arithmetic with overflow detection,
the full algebra (accumulate / branch_join / repeat / contains with reason
tokens), control propagation (sequence/parallel/select/race/iterate/
retry/compensation) and the canonical ACE1 encoding (shape walker over the
compiled module set + the shared ACE1 encoder from the semantic-ir oracle —
plumbing only). All 42 frozen expectations were produced by this oracle;
the Rust tests only consume.

## 13. Zero-drift results

- All existing TypeSchemaDigests unchanged (AFIDL corpus diff against
  HEAD: 0 drift, 1 new case).
- semantic-core 29… (all pre-existing digest goldens) unchanged; semantic-ir
  36 digests unchanged; spec-registry digests unchanged.
- All Phase 3B / 3C / 3D corpora and every SemanticIrDigest unchanged
  (`git status` over the legacy corpus directories: 0 modified files).
- Hash domain registry untouched: 14 domains (static-audited).
- semantic-ir.afidl byte-identical at
  `9fb3292a…53f616a3` (pinned in both af-ir and af-effect audits).
- AFIDL corpus revision 6, purely additive.

## 14. spec-check results

`cargo run -p af-spec --bin spec-check -- check`: **exit 0** —
16 specifications / 186 rules / 185 covered + 1 intentionally uncovered
(pre-existing ACE1 architectural invariant) / 0 missing coverage /
264 active ABI / 5 AFIDL modules / 0 cross-reference errors.

## 15. Cargo gate results

- `cargo fmt --all -- --check`: clean.
- `cargo clippy --all-targets --all-features -- -D warnings`: clean.
- `cargo test --all`: **213 passed, 0 failed** (197 before the phase;
  +7 af-effect unit, +5 af-effect conformance, +4 af-effect static
  surface). The gate remains "0 failed", never a pinned count.
- All 9 generators run twice back-to-back: **zero drift** (sha256 over
  every generated spec artifact identical between passes).

## 16. Decisions / Deferred items

Decisions D-1…D-16 are recorded in
`docs/implementation/phase-04a1-decisions.md`, including the two decisions
the task left open (canonical duplicate policy = reject, D-9; overflow =
explicit error, D-11) and the branch-join lower-bound correction made
during implementation (D-15). Explicitly deferred, structurally
unrepresentable and static-audited: ComponentContract + Resource
Requirement Row (port existence unclaimed), ComponentImplementation,
INVOKE exact contract checking, FailureValue/FailureRow, ResourceContract,
Information Flow, Usage/Budget, RecoveryContract and compensation
execution, ExecutionRequirements, EffectIntent/EffectGrant/EffectReceipt/
Effect Gateway/Credential Broker/Provider Dispatch, the production
capability catalog, dynamic MCP/tool discovery, executable plan, journal,
reducer, runtime, scheduler, worker.

## 17. Blockers for the next sub-phase

**None identified.** Phase 4A.2 (the remaining ComponentContract
dimensions) can consume: the typed `EffectSignature`, `contains` /
`require_contained_in` (the future component-substitution check
`Effects(new) ⊑ Effects(old)`), `AtomKey` as the effect identity,
`control::*` for composition, and the canonical-bytes pipeline. The
`component-contract` HashV1 domain is already frozen and reserved for the
whole-contract digest.

## 18. Git commits and diff overview

Five commits, categorized conventional:

1. `feat(af-effect): effect signature model, algebra and control
   propagation` — new crate (model/catalog/canonical/algebra/control/
   error), `spec/idl/effect-signature.afidl`,
   `spec/effect/effect-signature.md`, workspace membership.
2. `test(effect-signature): freeze the Phase 4A.1 conformance corpus` —
   the 42 frozen cases, the independent oracle generator, the Rust
   conformance harness and the static surface audit.
3. `test(af-idl,af-ir): rescope the additive corpus and wire-freeze
   audits` — AFIDL corpus revision 6 pin, 5-module/264-row cumulative
   audit.
4. `spec(registry): register the EFFECT rule area, 18 rules and suites` —
   rule-areas EFFECT, specifications 14→16, rules 168→186, ABI 232→264,
   suites 28→33, AFIDL corpus goldens regenerated.
5. `docs(implementation): phase-04a1 decision register and report` — this
   document and the decision register.

到此停止。不进入 Phase 4A.2。
