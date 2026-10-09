# Phase 4B.2B.3 — COMPENSATION_SCOPE static algebra

`CompensationScopeBody` is frozen as `{ child }`; there is no compensation
task-list or receipt field. Forward contracts propagate from that child.
Potential compensation actions are derived only from exact checked
`RecoveryActionBinding` and matching `RecoveryUsageBinding` in leaf K.

Identity remains `(forward EffectAtom, action kind, action EffectAtom)`, with
the complete operation, selector, qualifier, and action kind. For each exact
matching forward occurrence, the static action upper is the checked product of
forward occurrence upper, enclosing occurrence-family uppers, activation
upper, and attempts-per-activation upper. Its lower bound is zero because no
receipt proves that a specific occurrence was applied. Checked overflow fails
closed.

The derived action links to the exact action-Effect `epsilon` occurrence and
its scoped ResourceSlot mapping. Summed K-derived upper bounds must fit that
declared action Effect after the same control-family multiplier; undercoverage
is a Violation. Flow and Execution remain source-contract obligations because
frozen Recovery/K has no per-action Flow/X mapping.

The derived action links to the exact action-Effect `epsilon` occurrence and
its scoped ResourceSlot mapping. Flow and Execution remain source-contract
obligations because frozen Recovery/K has no per-action Flow/X mapping.

The derived conditional action is a separate Effect and Recovery Usage
projection. It never subtracts the forward Effect or refunds Forward Usage.
Unknown or partial receipt state is not treated as NOT_APPLIED; reconciliation
is a prerequisite to a concrete safe decision. Static source order does not
prove committed-effect order. Multi-event compensation order therefore needs
runtime evidence. `RecoveryBlocked` is not added to the business Failure row.

**Static Proven Facts:** exact K identity and declared activation/attempt
bounds when the checked source is complete; forward facts remain present.

**Conservative Upper Bounds:** potential new compensation Effects and matching
Recovery Usage, including all reachable source actions rather than only a
winning branch.

**RequiresEvidence:** nonzero action eligibility depends on receipts; actual
multi-event reverse order depends on committed order.

**Unproven:** missing recovery/usage sources or incomplete static coverage.

**Runtime-Only Obligations:** effect receipt classification, reconciliation,
action dispatch, commit ordering, and recovery completion/blocked outcome.
