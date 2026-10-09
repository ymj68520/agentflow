# Phase 4B.2B.3 — Full checked control-tree closure

`derive_workflow_contract_summary(checked_ir, snapshot, proof_context)` is the
root entry point. It verifies snapshot/context identity, walks checked ownership
in semantic order, resolves all reachable INVOKE/AWAIT/CALL_SUBFLOW leaves,
then calls the existing `derive_control_contract` bottom-up. No second control
analyzer or wire contract is introduced.

The closure report retains computation preorder and leaf preorder, then checks
that all known scoped Effect occurrences, residuals, and Recovery/K source
nodes are exactly accounted for. An incomplete source set is an explicit
`ControlDerivationError`; it is never normalized to empty. All 13 opcodes are
closed and exercised through the same derivation path. Phase 3C remains the
authority for scoped Value and Resource checks.

The root summary is derived-only and keeps typed root outputs and unresolved
per-dimension obligations. Since SemanticIrDocument has no exported root
ComponentContract pin or frozen cross-coordinate mapping,
`RootExportCompatibilityStatus::NotExpressedInSemanticIr` is explicit and
overall closure remains Unproven at that boundary.

**Static Proven Facts:** checked node and leaf coverage; deterministic order;
known source accounting; any independently Proven dimensions.

**Conservative Upper Bounds:** the existing operator-specific summaries over
the complete reachable tree.

**RequiresEvidence:** obligations inherited from individual nodes, including
runtime settlement, receipt, or execution facts.

**Unproven:** any unresolved child dimension and root-export compatibility.

**Runtime-Only Obligations:** authorization, dispatch, cancellation, actual
commit/receipt state, and implementation/body conformance.
