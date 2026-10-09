# Advanced Control Contract Algebra (Phase 4B.2B.2)

Status: frozen static-analysis semantics for the existing `ITERATE`, `RETRY`,
`TIMEOUT`, and `HANDLE` Semantic IR opcodes. This specification adds no IR or
AFIDL field, opcode, ABI assignment, digest domain, runtime identity, execution
state, timer, cancellation protocol, retry policy, grant, plan, or journal.

This phase extends the derived-only `DerivedControlContractSummary` introduced
by `spec/computation/control-contract-algebra.md`. All inputs remain the
already-checked Semantic IR, Phase 3C dataflow evidence, resolver-created leaf
summaries, and the exact immutable static proof context. A derived envelope is
not a `ComponentContract`, authorization, runtime observation, or proof that a
workflow completed.

## 1. Shared composition invariants

1. Child contracts are derived recursively in the opcode's owned-child order.
   A missing or unresolved child remains unknown; it is never replaced by a
   pure Effect row, empty Failure row, or zero Usage envelope.
2. Effect arithmetic reuses the checked `af-effect` algebra; Usage arithmetic
   reuses the checked `(phase, meter)` algebra; typed Failure propagation
   reuses `af-failure`; Flow, Resource, Recovery, Execution and typed outputs
   retain their own source-scoped evidence. Overflow is a violation, not wrap,
   saturation, or an unknown smaller bound.
3. Per-leaf Effect occurrences remain distinct by caller digest, source node,
   contract digest, and entry ordinal. A control-family annotation records
   static multiplicity without creating an `InvocationId`, `AttemptId`, or
   runtime occurrence path.
4. Every proof is dimension-specific. A conservative envelope cannot erase an
   inherited `Violation`, `RequiresEvidence`, or `Unproven` obligation.

## 2. ITERATE

<a id="advanced-control-iterate-bounds"></a>

### 2.1 Repetition bounds

`WHILE` has an upper bound `max_iterations` and a conservative lower bound of
zero: the guard may be false before the first body execution, and a later body
failure may stop the loop. Its Effect repetition interval is therefore
`[0, max_iterations]`. Its Usage upper bound is scaled by
`max_iterations`; Usage has no declared lower bound.

`FOREACH` iterates the materialized source collection. A literal
`ListConstruct` provides an exact static source length `n`; the conservative
Effect interval is `[0, n]`, accounting for a body failure before later items.
A data-dependent source has no statically proven count and uses `[0,
unbounded]` for Effect. Its known Usage coordinates become `Unknown`; an absent
coordinate remains unknown. `max_inflight` is concurrency only and never a
total-work multiplier or total-iteration bound.

For either mode, Effect repetition is checked through `iterate_envelope` and
Usage uses the known upper count or an unknown envelope. The Failure row is the
body row verbatim: repetition changes no nominal Failure type. Source and guard
dependencies are retained as control-decision dependencies; no taint or
authority is inferred from an unmodeled value transfer. Resource, Recovery,
and Execution facts remain per source child. A carry output is projected only
when the checked carry/body-output mapping supplies exact type and producer
evidence; otherwise the interface is explicitly unproven.

## 3. RETRY

<a id="advanced-control-retry-envelope-and-recovery"></a>

Given the frozen ordered delay vector `delays`, the maximum number of physical
child-computation attempts is exactly `1 + len(delays)`, with physical bounds
`[1, 1 + len(delays)]`. Delay order is preserved, including zero delays. The
IR contains no retryable-Failure list or retry policy, so this phase does not
invent one.

The conservative logical Effect envelope scales the child's declared Effect
by the maximum attempt count. Physical attempts and logical Effect
multiplicity are both projected, but remain different concepts. An
idempotency qualifier, K declaration, key declaration, or retry delay does not
reduce that upper envelope. A smaller logical bound still requires the
separate recovery/evidence proof and is never asserted by this phase. Effect
arithmetic is checked.

The child's typed Failure row remains unchanged; exhausting attempts is not a
proof that a business Failure is impossible. Usage scales by the maximum
attempt count, with checked arithmetic and no refund. Flow records that retry
decisions depend on child outcomes without choosing an attempt result.
Recovery sources and action/usage bindings remain exact and per child. A
`forbidden` Replay declaration conflicts with a retry of effectful work and is
a static violation. Other K declarations are evaluated using the existing
static Recovery safety analyzer: missing coverage remains Unproven, evidence
requirements remain RequiresEvidence, and a Proven static constraint still
does not create provider evidence, a retry permit, or a narrower logical
Effect envelope.

## 4. TIMEOUT

<a id="advanced-control-timeout-outcomes"></a>

TIMEOUT preserves the child's Effect, Usage, Failure, Flow, Resource, Recovery,
Execution, and typed-output contracts; duration never discounts work that may
already have occurred or may occur before the terminal outcome.

The derived outcome set contains both `child_completed` and `timed_out`.
`TimedOut` is a control outcome, not a member of the typed Failure row. Static
analysis does not infer successful cancellation, zero Effect, refund, or
winner from the duration expression. A constant duration may be reported as
static expression evidence; a dynamic duration retains its Phase 3C value
dependencies. No timer state or wall-clock observation is created.

## 5. HANDLE

<a id="advanced-control-handle-residual"></a>

Each `failure` clause matches its exact pinned `TypeSchemaRef`; `timed_out` is
kept as a distinct control matcher. No subtyping, string-name matching, or
`cancelled` matcher is introduced. The Failure result is
`(protected Failure \ handled exact types) ∪ all possible handler Failures`.
An exact handled type absent from a complete protected Failure row is a
violation. Any unmatched protected failures remain in the residual row.
`timed_out` never enters that Failure row.

Protected-child Effects and Usage occur before a handler path. Handler paths
are conditional and mutually exclusive: Effect is the protected-child
envelope accumulated with the branch join of possible handler envelopes and
the no-handler path; Usage is protected-child Usage accumulated with the
maximum possible handler path. The no-handler path is finite zero only for
coordinates explicitly declared by a handler. A coordinate omitted by a
handler remains unknown; absence is not globally zero. Checked overflow is a
violation.

Every handler's Flow, Resource, Recovery and Execution facts remain
source-scoped. Handler selection is an unknown control influence unless the
current model proves otherwise. Outputs are derived from the exact checked
protected/handler interface evidence; no exception value or runtime handler
state is materialized.

## 6. Stable rule identifiers

- `AF-SEMANTIC-0143`: bounded ITERATE repetition and carry-scoped projection.
- `AF-SEMANTIC-0144`: explicit RETRY attempt bounds, conservative logical
  Effect amplification, and Recovery safety obligations.
- `AF-SEMANTIC-0145`: TIMEOUT preserves child contracts and keeps
  `TimedOut` outside the typed Failure row.
- `AF-SEMANTIC-0146`: HANDLE exact-match residual Failure and conditional
  handler contract composition.

The 72-case independent Python oracle and full production differential live in
`spec/conformance/computation-advanced-control/` and
`tools/gen-advanced-control-corpus.py`. The former Phase 4B.2B.1
`unsupported-timeout-not-empty` vector is retained as a regression identity,
but now proves that the supported TIMEOUT preserves a known child Effect
instead of reporting an unsupported/unknown operator.
