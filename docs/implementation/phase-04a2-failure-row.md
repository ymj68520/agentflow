# Phase 4A.2 — Typed Failure Row / Failure Signature Algebra: Report

Date: 2026-10-08. Scope: the Φ dimension of
`Computation<I, O, Φ, ε, R>` — the static declaration side only.
Decisions: `docs/implementation/phase-04a2-decisions.md` (D-1…D-13).

---

## 1. Failure Row AFIDL types and count

New module `spec/idl/failure-row.afidl`
(`schema agentflow.failure.row @1`, importing
`agentflow.semantic.core@1 as core`). Exactly **2 numbered types / 2 ABI
rows**, per the minimal-closed-set mandate:

| type | kind | numbering |
|---|---|---|
| `FailureTypeRef` | record | 1 field (`payload_type: core.TypeSchemaRef`) |
| `FailureRow` | record | 1 field (`failures: list<FailureTypeRef>`) |

No auxiliary types: nothing else carried static semantics. The payload is
the semantic core's `TypeSchemaRef` by exact cross-module import — never a
bare string, never a redeclared reference shape. AFIDL corpus revision
6 → 7 (purely additive; every pre-existing case and digest verified
unchanged).

## 2. Nominal FailureTypeRef model

Identity = `(specification_id, type_name, schema_digest.hex)` — all three
parts, always. `nominal_key()` is the canonical byte-order sort key;
`is_same_type` is the exact equality; matching by short name, similar
schema, record shape, error message, error code or exception class name
does not exist anywhere in the crate (static-audited).

## 3. Closed Failure Row representation

`FailureRow` is a closed set of refs — no open/residual/unknown wire
state (deliberately not copied from the Effect Row). `∅` is the algebra
identity; `is_empty` is "no typed business failure declared", never
"guaranteed success". Unknown upper bounds are the analysis-level
`StaticFailureEnvelope::Unproven` with the fail-closed `require_row()`
(`failure_row_unproven`) — no wire representation.

## 4. Canonicalization and duplicate rules

`canonicalize_failure_row`: syntax check per ref (specification id
grammar, name, digest form) → duplicate rejection by full nominal
identity → sort by nominal key. Declaration order never reaches the
canonical ACE1 bytes; canonicalization is idempotent (frozen).
Declared duplicates → `duplicate_failure_type`; the algebra's union
deduplicates — the two behaviors are deliberately separate.

## 5. Containment / Union / Subtraction algebra

- **⊆ (contains)** — exact nominal membership; the future
  "Failure 不扩张" substitution check. Reflexive and transitive where
  proven (both frozen).
- **∪ (union)** — dedup + canonical order; commutative, associative,
  idempotent, `∅` the identity (all four frozen, cross-validated).
- **join** — the named branch-composition alias of union (SELECT /
  PARALLEL / RACE are exactly set union).
- **remove_handled(Φ, H)** — `H ⊆ Φ` enforced
  (`failure_not_declared`), returns `Φ \ H`; a static residual after a
  full type check, never a claim that a HANDLE executed.

## 6. HANDLE static propagation rule

`Φ_result = (Φ_child \ H) ∪ Φ_handler_1 ∪ …`: `H` may only contain
declared child failures; handler failures union in (never "automatically
handled"); unhandled failures survive verbatim. Frozen cases:
exact removal, unhandled preservation, two-handler propagation. The
frozen `HandleMatch` shape is unchanged and read-only-checked:
`failure: FailureMatch{failure_type: core.TypeSchemaRef}` beside
`timed_out: unit`, no `cancelled` matcher; the Phase 3B disjointness rule
on duplicate failure matchers is untouched.

## 7. Control Failure propagation rules

Pure functions in `af_failure::control` (no `af-ir`): SEQUENCE / SELECT /
PARALLEL / RACE = union of the participating rows (each frozen); ITERATE
= body row verbatim (types never multiply); RETRY = child row verbatim
(attempts ≠ impossibility proof); TIMEOUT = child row (TimedOut never
enters Φ); SCOPE = child; COMPENSATION_SCOPE = child (forward failures
never erased); AWAIT = ∅ (waiting invents nothing); HANDLE = section 6;
INVOKE / CALL_SUBFLOW = deferred to the ComponentContract stage — no
function exists, `Unproven` is the honest analysis result, and "no
explicit FailureRow → assume ∅" is forbidden.

## 8. Six-plane isolation

Structural, not conventional: `TimedOut`, `Cancelled`, `RecoveryBlocked`,
`TaskAttemptLost`, `RuntimeIncident` (and `FailureValue`,
`TerminalOutcome`) appear NOWHERE in the crate's code or the AFIDL module
(static audits, comments stripped); no fake `TimedOutFailure` /
`WorkerLostFailure` test schema was registered. The frozen §22.12/§24.15
plane table is restated normatively in
`spec/failure/failure-row.md` sections 2 and 11.

## 9. Rust crate structure

`crates/af-failure` (deps exactly: af-core, af-idl, af-semantic, serde):
`model.rs` (FailureTypeRef/FailureRow + nominal accessors),
`validate.rs` (exact pinning against `TypeSchemaUniverse`, three distinct
kinds), `canonical.rs` (ordering + canonical ACE1 bytes via
`agentflow.failure.row@1/FailureRow`), `algebra.rs`
(contains/union/join/remove_handled/propagate_handler_failures),
`control.rs` (the propagation table + `StaticFailureEnvelope`),
`error.rs` (7 stable kinds). Unit-tested (canonical ordering, duplicate
rejection, exact pinning, algebra, subtraction failures) plus 4
conformance + 5 static-surface tests.

## 10. Golden / Negative / Algebra corpus counts

37 frozen cases under `spec/conformance/failure-row/` (revision 1), every
file self-contained with its fixture schema block (two frozen test
modules: `agentflow.test.failures@1` with record/enum/variant payloads,
`agentflow.test.failures-alt@1` with the same short name in a different
family):

- golden: **8** — empty row, single, multiple, same-short-name-different-
  family, enum/record/variant-backed, canonical ordering (each freezing
  canonical ACE1 bytes + is_empty + count);
- negative: **8** — duplicate, unknown schema, wrong digest, invalid
  nominal reference, same-name subtraction mismatch, unproven
  containment, handle matcher not in child, ambiguous duplicate handled
  set;
- algebra: **18** — union identity/commutativity/associativity/
  idempotence, containment reflexivity/transitivity, exact removal,
  unhandled preservation, handler propagation, select/parallel/race
  unions, retry preservation, timeout adds nothing, compensation no
  erasure, iterate type preservation, await ∅, scope passthrough;
- equivalence: **3** — declaration ordering, union ≡ declared,
  join ≡ union.

## 11. Independent Python Oracle scope

`tools/gen-failure-row-corpus.py` implements from scratch: nominal
identity triples, the fixture schema digests (via the independent AFIDL1
compiler — plumbing only), exact resolution, canonical sorting,
duplicate rejection, containment, union, subtraction, handler
propagation, all control unions, and the canonical ACE1 encoding (local
shape walker + the shared ACE1 encoder). All 37 expectations are
oracle-frozen; the Rust harness only consumes. (Only the shared ACE1
value encoder from the semantic-ir oracle is reused as plumbing.)

## 12. Specification / Rule / ABI statistics

- Specifications: 16 → **18** (`agentflow.failure.row@1` schema,
  `agentflow.failure.row.semantics@1` semantics).
- Rule area: **FAILURE** added.
- Rules: 186 → **203** (`AF-FAILURE-0001…0017`, each with exactly one
  normative anchor; refs into the corpus + 5 new suites; missing
  coverage 0).
- Suites: 33 → **38** (SUITE-FAILURE-CONTROL-OUTCOME-ISOLATION,
  -DEFERRED-CONTRACT-RESOLUTION, -NOMINAL-NO-STRING-MATCHING,
  -NO-RUNTIME, -WIRE-FREEZE).
- ABI: 264 → **266 active** (+2 failure-row rows); 0 retired/reserved.
- AFIDL modules: 5 → **6**; AFIDL corpus revision **7**; HashV1 domains
  **14** (unchanged). No existing RuleId, ABI assignment or
  SpecificationId changed.

## 13. Zero-drift proof

Against HEAD: `git status` over the legacy corpora (semantic-ir,
semantic-dataflow, subflow-interface, semantic-core,
effect-signature) and the pre-existing AFIDL modules + hash-domains.json:
**0 modified files**. AFIDL corpus diff: all pre-existing cases and
TypeSchemaDigests byte-identical, 1 new `failure-row` case (additive
revision 7). semantic-ir.afidl byte-identical at
`9fb3292a…53f616a3` (pinned in the af-ir and af-failure audits). Effect
corpus revision stays **2** (asserted in the af-failure wire-freeze
audit). All SemanticIrDigests unchanged (the 197-test Phase 3B/3C/3D
suites pass unmodified).

## 14. spec-check results

exit **0**: 18 specifications / 203 rules / 202 covered + 1 intentionally
uncovered (pre-existing ACE1 architectural invariant) / 0 missing
coverage / 266 active ABI / 6 AFIDL modules / 0 cross-reference errors.

## 15. Cargo gate results

- `cargo fmt --all -- --check`: clean.
- `cargo clippy --all-targets --all-features -- -D warnings`: clean.
- `cargo test --all`: **224 passed, 0 failed** (215 before the phase;
  +4 af-failure conformance, +5 af-failure static surface; gate remains
  "0 failed", never a pinned count).
- All 10 generators run twice back-to-back: **zero drift** (sha256 over
  every generated spec artifact identical between passes).

## 16. Decisions / Deferred items

D-1…D-13 in `docs/implementation/phase-04a2-decisions.md`. Explicitly
deferred, structurally absent, static-audited: FailureValue runtime
payloads, TerminalOutcome, RuntimeIncident ABI, worker error transport,
failure mapping adapters, ComponentContract, ResourceRequirementRow,
InformationFlow, Usage, RecoveryContract, ExecutionRequirements,
INVOKE/HANDLE/RETRY runtime, executable plan, journal, reducer,
scheduler, worker, subtype/compatibility proofs, and any modification to
the 13 frozen opcode wire schemas.

## 17. Blockers for the next sub-phase

**None identified.** The ComponentContract stage can now aggregate:
typed inputs/outputs (3A/3B), the derived subflow interface (3D), the
effect envelope (`contains`/`require_contained_in` in af-effect), and the
failure envelope (`contains` in af-failure) — the four dimensions the
frozen substitution rule needs (input contravariance, output covariance,
Failure 不扩张, Effect 不扩张). The `component-contract` HashV1 domain is
reserved and untouched.

## 18. Git commits and diff overview

Five commits, categorized conventional:

1. `feat(af-failure): static typed failure row, algebra and control
   propagation` — new crate + `spec/idl/failure-row.afidl` +
   `spec/failure/failure-row.md` + workspace membership.
2. `test(failure-row): freeze the Phase 4A.2 conformance corpus` — 37
   frozen cases, the independent oracle, the Rust harness and the static
   audits.
3. `test(af-idl,af-ir,af-effect): rescope the additive corpus and
   wire-freeze audits` — AFIDL corpus revision 7 pin, 6-module/266-row
   cumulative audit, effect audit rescope.
4. `spec(registry): register the FAILURE rule area, 17 rules and suites`.
5. `docs(implementation): phase-04a2 decision register and report`.

到此停止。不进入 Phase 4A.3。
