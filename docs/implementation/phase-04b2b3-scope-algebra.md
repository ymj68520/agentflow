# Phase 4B.2B.3 — SCOPE static algebra

`ScopeBody` is frozen as `{ child, resources }`. It is an ownership and lexical
resource-visibility boundary, not an authority service, budget reservation, or
compensation operator. Phase 3B validates the ownership tree; Phase 3C is reused
for the exact parent and child `EntryResources` sets.

The derived `ScopeBoundarySummary` reports control node, child, parent-visible
slots, declared slots, and child-visible slots. Resource attenuation is Proven
only if the declared set is a subset of the parent set and the child sees
exactly that declared set. The implementation does not infer a ResourceHandle
or access grant. A malformed boundary is a Violation, not an empty set.

Scope recursively preserves child Effect occurrences, guards, bounds, Failure,
Flow, Usage, K/Recovery, Resource provenance, Execution/X and typed outputs.
It creates no compensation action, rollback, reconciliation, or usage refund.

**Static Proven Facts:** checked child ownership; Phase 3C lexical visibility;
child contract facts for dimensions with resolved evidence.

**Conservative Upper Bounds:** all child Effects and Usage remain possible;
cancellation does not discount either.

**RequiresEvidence:** owned children must settle before owner terminal; static
ownership is not a runtime terminal observation.

**Unproven:** authority attenuation and budget attenuation. The frozen Scope
body has no ceiling fields; no authority or budget is inferred from absence.

**Runtime-Only Obligations:** effective grant evaluation, resource acquisition,
budget reservation, cancellation, and observation that every child settled.
