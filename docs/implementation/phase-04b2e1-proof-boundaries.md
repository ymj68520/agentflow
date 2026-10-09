# Phase 4B.2E.1 — Static Proof Boundaries

## Facts established by this layer

For the exact checked ComponentContract, checked Semantic IR, immutable
resolution snapshot, and pointer-identical StaticContractProofContext, the
analyzer can establish:

- a component-local K Forward guard is contained by its exact action epsilon
  guard, or a validator-created typed countermodel proves a violation;
- all checked K source/action/usage profiles are retained as exact multisets;
- LocalActionBound and RootActionBound use their correct distinct coordinates;
- checked Iterate/Retry identities are applied once through existing
  operator-specific Effect/Usage algebra;
- root K action potential is covered by the existing root epsilon envelope;
- root K-required Recovery Usage is covered by the existing root
  `U.recovery`, without adding a second charge.

The public result is an in-memory derived summary, not a wire contract or
digest input. `ProofReport.action_guard_compatibility`,
`RootRecoverySummary.proof`, and the source-closure report retain dimension-
specific outcomes instead of collapsing to one boolean.

## Deliberate unknowns

The analyzer does not infer a relation from equal numeric port numbers in
different source contracts. Root guard coordinates keep source digest,
ComponentInputPort, exact ScopedPortMapping, expression, type, and dataflow
dependencies separate. If a typed branch/value relationship is not available
in checked IR evidence, it remains Unproven; arbitrary selector-to-input
predicates are not synthesized.

Same-atom Forward/Action recovery uses the shared epsilon ceiling equation
only when the exact K binding/profile set isolates the Forward seed. A
conflicting Forward lower is Violation. If unrelated Forward identities also
consume that action atom and their contribution cannot be isolated, the
checker returns Unproven. This is not an implicit extra EffectGuard language.

An absent/unknown active-source meter is not finite zero. An empty K leaf may
be omitted only when the complete source closure proves that the leaf has no K
action contribution. An unknown source, profile, resource mapping, or
operator result remains partial/unknown and is not erased to get a positive
aggregate.

## Not established

This phase does not implement RootExportAssertion or root I/O, resource,
failure, F/X, epsilon/K/U export containment. K20's finite-export comparison
is explicitly deferred. Root Recovery `Proven` does not imply workflow
implementation conformance, a trusted subflow body, a receipt state, action
eligibility, runtime recovery safety, or execution authorization.

There is no ExecutablePlan, WorkflowVersion, journal, reducer, scheduler,
worker, EffectGrant, ResourceHandle, BudgetReservation, UsageReceipt,
EffectReceipt, recovery command, reconciliation call, or compensation
execution in this layer.
