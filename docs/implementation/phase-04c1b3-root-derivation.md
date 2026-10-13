# Phase 4C.1B.3 — Fresh Root Derivation

D9 runs under the same `with_proof_context` callback as fresh D6–D8 values. The call is `derive_workflow_contract_summary(checked_caller, &snapshot, proof_context)`; there is no Portable-specific control algebra.

The returned `DerivedWorkflowContractSummary` is retained as one source object for `root`, `closure`, `root_recovery`, and `resolver_identity`. Before D10, the pipeline verifies the resolver snapshot context, the root proof-context pointer, and full resolver identity against that exact D8 snapshot.

The scoped call site and pointer checks are in `crates/af-portable/src/pipeline.rs`; the production traversal is `derive_workflow_contract_summary` in `crates/af-computation/src/control.rs`.

The production control traversal covers the registered thirteen opcodes: INVOKE, AWAIT, SEQUENCE, SELECT, PARALLEL, RACE, ITERATE, RETRY, TIMEOUT, HANDLE, SCOPE, COMPENSATION_SCOPE, and CALL_SUBFLOW. The 22 D9-Control bundles exercise each opcode plus ordered sequence, fail-fast parallel, three-way race, extended retry, nested timeout/retry, scope nesting, and selected-branch variants.

The D9 projection preserves dimension states instead of converting Unknown or Partial into empty values. RACE retains all contestants. TIMEOUT does not claim cancellation; RETRY does not create idempotence; SCOPE does not invent authority/budget ceilings; CALL_SUBFLOW does not establish body conformance.

## Work bounds

The integration preflight caps the caller at 1,024 computation records and estimates root derivation work from source leaves, D6 effect rows/residuals, recovery bindings, and tree size (2,000,000 units). Root projection is depth-limited; a bound failure returns `ResourceLimit` before D10 capability issuance.
