# Phase 4B.2E.1 — Guard Compatibility Implementation

## D1 implementation mapping

`af-contract::analyze_action_guard_compatibility` is the component-local
checker. `analyze_contract` adds every per-action relation to `ProofReport`;
`check_contract` issues `CheckedComponentContract` only if the complete report
remains Proven. The new `AF-CONTRACT-0022` recovery-dimension obligation does
not modify ComponentContract fields, canonicalization, or its digest.

For each exact `RecoveryActionIdentity`, the checker finds the full forward
and action `EffectEntry` by `AtomKey`, the matching `RecoveryUsageBinding`, and
the component-local port/type of every `InputEquals` guard. The static
possibility uses the Forward guard; the action Effect guard is its public
epsilon ceiling. It reuses the frozen `af_effect::guard_contained` direction:
conditional under unconditional and exact guard equality can prove
containment. A failed containment result is not itself a counterexample.

## Typed countermodel and verdicts

`TypedGuardCountermodel` is validator-created and retains the complete source
ComponentContract, component identity, full K identity, both EffectEntry
ordinals and guards, Forward/action bounds, exact input `TypeSchemaRef`s,
primitive assignments, reachability basis, and related RuleId. The current
type universe has no refinement predicates, so an assignment is reachable
only when the exact input schema expands to the same primitive type as the
constant. An alternate well-typed primitive value can prove that the Forward
is possible while the action epsilon guard is false. Unit/Null have no
alternate in this checker and therefore do not yield a false Violation.

The result is:

- `Proven` when frozen containment succeeds or a checked zero K upper makes
  the obligation vacuous.
- `Violation` when a typed countermodel exists, the input pin/type is
  contradicted, or checked action-bound arithmetic is inconsistent.
- `Unproven` when the source, type/domain, or guard relation cannot be
  established and no countermodel exists.
- `RequiresEvidence` remains independent for receipt, reconciliation, and
  actual action eligibility; this checker does not promote it.

No caller-provided `reachable` or `compatible` boolean is accepted. A positive
guard proof creates no receipt, recovery permit, action command, or dispatch.

## AUDIT-10 production reproduction

`computation-root-recovery/audit/AUDIT-10` uses a typed Int64 input, an
unconditional Forward, and a narrower action epsilon guard `p1 = 0`, while
retaining the exact K binding, A/T bounds, recovery meter profile, and resource
mapping. The production ComponentContract validator returns Violation with a
typed `p1 = 1` countermodel and does not issue a CheckedComponentContract.
This closes the specific Phase 4B.2C AUDIT-10 gap that previously returned
RequiresEvidence while exposing an unconditional K sidecar beneath a
conditional epsilon row.

At workflow root, source component input ports are retained as
`RootGuardCoordinateEvidence` with the exact source-contract digest, checked
`ScopedPortMapping`, exact type, input expression, and Phase 3C dependency
slots. The control path is separate typed evidence (`ControlPathStep`); it is
never forged into a new EffectGuard. If no exact component-local proof or
typed countermodel exists, the root relation stays Unproven. This phase does
not implement arbitrary selector-to-input Boolean reasoning.

## Frozen surface

No EffectGuard constructor, guard canonicalization, AFIDL module, ABI
assignment, digest domain, ComponentContract digest, or SemanticIr digest
changed. The production entry points are `analyze_contract`,
`check_contract`, and the derived-only root proof; there is no compatibility
shortcut that bypasses the formal validator.
