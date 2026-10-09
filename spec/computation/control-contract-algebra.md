# Structured Control Contract Algebra

Specification ID: `agentflow.semantic.control-contract-algebra.semantics@1`

This specification defines non-wire, static summaries for `SEQUENCE`,
`SELECT`, `PARALLEL`, and `RACE` only. It consumes checked Semantic IR,
Phase 3C dataflow, exact Phase 4B.2A leaf summaries, and the same exact proof
context. The result is a `DerivedControlContractSummary`, not a
`CheckedComponentContract`, implementation proof, or durable identity.

This file records the Phase 4B.2B.1 algebra. The additive Phase 4B.2B.2 rules
for `ITERATE`, `RETRY`, `TIMEOUT`, and `HANDLE` are in
`spec/computation/advanced-control-contract-algebra.md`. Phase 4B.2B.3 adds
`SCOPE`, `COMPENSATION_SCOPE`, and workflow-root closure in
`spec/computation/scope-compensation-contract-algebra.md`. Those specifications
supersede only the corresponding historical unsupported-operator statements
below. No frozen IR or contract schema changes.

## Summary, context, and unsupported operators

<a id="control-summary-is-derived-only"></a>

A control summary MUST retain the source SemanticIrDigest, control node,
operator, direct child identities, typed interface evidence, each dimension's
availability and verdict, provenance/occurrence sources, child summaries,
and unresolved obligations. It MUST NOT be wrapped as a ComponentContract,
used to compute a control/workflow digest, or interpreted as implementation
attestation.

<a id="control-context-and-leaves"></a>

Every supplied `ResolvedComputationLeaf` MUST name the current caller
SemanticIrDigest and the exact proof-context object supplied to the analysis.
The analyzer MUST reuse Phase 3C dataflow, preserve frozen child order, and
reject duplicate or mismatched leaf identities. A missing leaf summary is
`Unproven`, never an empty child contract.

<a id="control-unsupported-not-empty"></a>

Before Phase 4B.2B.3, `SCOPE` and `COMPENSATION_SCOPE` had no propagation rule.
They now use the explicit static algebra in
`scope-compensation-contract-algebra.md`. Any future unknown or unsupported
operator remains `Unproven`; known descendants and obligations remain
traceable, and no unsupported node may silently become an empty row.

## SEQUENCE

<a id="control-sequence-effect"></a>

SEQUENCE children remain in the frozen source order and use the existing
`af-effect` accumulation algebra for the conservative upper envelope. A
later child can be skipped if an earlier child does not reach the required
successful continuation. Because the IR carries no proof that prior
children must succeed, a positive Effect lower bound from child index greater
than zero MUST be weakened to zero before accumulation. Its upper bound,
source atom, guard provenance, and child occurrence MUST remain. This local
weakening MUST NOT change the frozen `af-effect` algebra.

<a id="control-sequence-failure-usage"></a>

The Failure row is the union of typed failures of all children that may run;
empty Phi is not success evidence and control outcomes are not inserted into
Phi. Usage uses checked accumulation over possible children; later-child
usage is not set to zero merely because an earlier failure may skip it.
Forward and Recovery coordinates remain separate.

<a id="control-sequence-dataflow"></a>

The analyzer MUST reuse Phase 3C's sequence availability, producer and
dominance evidence. It MUST retain exact cross-sibling value dependencies
from earlier child exports to later child expression uses. A conservative
union of known may-influence summaries MAY be reported, but without
per-value taint transfer proof the output-specific Flow relation remains
`Unproven`.

## SELECT

<a id="control-select-exclusive"></a>

SELECT is mutually exclusive and MUST use the existing Effect `branch_join`
and Usage maximum/join algebras, never sequential or parallel accumulation.
Failure types union over all selectable cases. Unknown child rows are not
zero branches. Full EffectAtom identity, including every qualifier and
ResourceSelector scope, remains part of branch join.

<a id="control-select-guarded-lower"></a>

Effect lower bounds MUST follow the frozen guarded branch-join rules:
identical guards may retain the minimum lower bound; differing guards widen
to unconditional with lower zero; one-sided atoms/residuals have lower zero.
The analyzer MUST NOT restore a positive lower bound after that join.

<a id="control-select-coverage-and-influence"></a>

The frozen SELECT IR has an int64 selector and exact case ordinals, but no
default/range field. A directly constant selector that matches a case may be
resolved exactly. Otherwise selector totality is `Unproven`; positive Effect
lower bounds are weakened to zero. Selector data dependencies are recorded
as possible decision influence. The analyzer MUST keep content influence
separate from control/decision influence and MUST NOT elevate authority.

<a id="control-select-interface"></a>

<a id="control-typed-interface"></a>

SELECT output evidence MUST use the checked typed interface shared by all
cases and Phase 3C's producer/availability proof. A local SemanticTypeId is
not compared across documents or treated as a public TypeSchemaRef.

## PARALLEL

<a id="control-parallel-all"></a>

PARALLEL MUST use only the frozen `all` and `fail_fast` modes. Every branch
that may execute contributes its conservative Effect upper bound and Usage
upper bound using the existing checked accumulation APIs. `max_inflight` is
concurrency only; it MUST NOT scale or cap total Effect/Usage, or create a
Worker/placement aggregate.

<a id="control-parallel-fail-fast"></a>

`fail_fast` requests cancellation; it does not erase a sibling's possible
Effect, Usage, Failure, or Recovery fact. Since a sibling may be cancelled
before an individual effect occurs, positive child Effect lower bounds that
depend on full child execution MUST be weakened to zero while upper bounds
remain accumulated. Static analysis MUST NOT designate the physically first
failure as a primary cause.

<a id="control-parallel-output-resource"></a>

PARALLEL parent outputs MUST use the frozen explicit
`(parent_port_no, branch_ordinal, child_port_no)` mapping and existing typed
merge proof. Resource ports remain per-child namespaces. Equal numeric port
numbers do not identify the same requirement; a shared caller ResourceSlot
is recorded only through its explicit slot identity and never reduces an
upper bound.

## RACE

<a id="control-race-effect-usage"></a>

RACE is FIRST_TERMINAL, not SELECT and not `PARALLEL ANY`. The Effect upper
envelope MUST accumulate all contestants, including losers; Usage MUST
accumulate all contestant envelopes. Cancellation is not evidence that a
loser's work or external effect did not occur. Where branch completion is
not guaranteed, positive Effect lower bounds MUST be weakened to zero.

<a id="control-race-recovery"></a>

Every contestant's K contract, exact action bindings, per-action Usage
profiles, and associated effect occurrences MUST remain in the summary,
including losing branches. Not selected does not mean no effect, no
partial application, or no recovery obligation. Compensation adds facts; it
does not subtract the forward Effect.

<a id="control-race-outcome-flow"></a>

The winner is a runtime terminal observation and MUST NOT be selected by
static analysis. Failure rows union over all contestants; cancellation is
not a business Failure. Parent output sources and taints conservatively
cover every possible winner, while loser Effects/Flow remain source-scoped.
Winner-selection influence that cannot be proven remains `Unproven`.

## Per-dimension propagation and identity

<a id="control-flow-may-influence"></a>

<a id="control-dimension-separation"></a>

The analyzer MUST NOT use one generic row union for all dimensions. Failure
uses typed set union; Effect uses operator-specific accumulation/join with
checked bounds and guards; Resource and Recovery remain per-child; Flow
uses conservative may-influence propagation while preserving child
constraints/partitions; Usage uses checked accumulation or exclusive join;
Execution remains per-child. Unknown is never normalized to empty or zero.

<a id="control-usage-propagation"></a>

Usage envelopes MUST use only exact `(phase, meter)` coordinates. Missing
coordinates remain Unknown under the existing algebra; an empty declaration
is not a finite-zero guarantee.

<a id="control-recovery-per-child"></a>

Recovery action and per-action usage bindings MUST remain associated with
their source child and effect occurrences. The analyzer does not compute a
cross-child recovery contract or subtract compensation.

<a id="control-resource-port-provenance"></a>

Resource requirement ports MUST remain attached to their source component
contract and caller ResourceSlotId. Equal numeric port values do not merge
across component namespaces.

<a id="control-execution-per-child"></a>

Execution requirements MUST remain per child. No sum of CPU/quantity values
may be interpreted as one-worker placement demand.

<a id="control-proof-obligation-propagation"></a>

Per-dimension proofs and all inherited child obligations MUST remain
available. `Violation` has overall priority, then `RequiresEvidence`, then
`Unproven`, then `Proven`; this priority MUST NOT delete or hide any
per-dimension result. A computed conservative envelope may be known while
the overall result remains `RequiresEvidence`.

<a id="control-occurrence-identity"></a>

Two uses of the same ComponentContract remain distinct logical occurrences.
Provenance MUST retain caller SemanticIrDigest, control/leaf node identity,
source contract digest, and source entry ordinal. Effect algebra may merge
equal AtomKeys only in the envelope; the occurrence-provenance list MUST NOT
be deduplicated by contract digest or AtomKey. No runtime InvocationId,
AttemptId, UUID, timestamp, WorkerId, or journal path is created.

<a id="control-resource-execution-boundaries"></a>

R and X are retained as explicit child-scoped facts. The analyzer MUST NOT
union different components' resource ports by port number, reduce bounds
because aliases share a slot, or sum child X into one Worker demand. No
resource handle, EffectGrant, placement, admission, or capacity reservation
is produced.

<a id="control-no-runtime-or-digest"></a>

All control analysis is pure and deterministic over explicit checked inputs.
It creates no ControlContractDigest, WorkflowContractDigest, AFIDL/ACE1
object, HashV1 domain, plan, journal, reducer, runtime state, actual branch
selection, cancellation, or recovery execution. This specification adds no
SCOPE or COMPENSATION_SCOPE propagation rule; it does not start Phase 4B.2B.3.
