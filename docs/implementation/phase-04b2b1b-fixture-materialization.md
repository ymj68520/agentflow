# Phase 4B.2B.1B — Production Fixture Materialization

All code and metadata described here are `TEST-ONLY`, are **not a wire ABI**,
and are **not a runtime model**. The support layer lives in
`crates/af-computation/tests/support/control_fixture.rs` and invokes the
existing production APIs; it contains no second control algebra.

## Production construction chain

For the 16 production pilot CaseIds, the harness now performs:

```text
abstract child fields
  → complete ComponentContract fixture
  → check_contract + canonical ComponentContractDigest
  → one FixtureContext / one exact StaticContractProofContext object
  → StaticComputationResolutionSnapshotBuilder
  → formal SemanticIrDocument
  → CheckedIrDocument::check + analyze_dataflow (Phase 3C)
  → resolve_computation_leaf for every child
  → derive_control_contract
  → ControlConformanceProjection
  → frozen Expected comparison
```

The builder does not instantiate `CheckedComponentContract`,
`CheckedIrDocument`, or `ResolvedComputationLeaf`. The production validators
and resolvers mint them. Every control case repeats derivation against the
same checked IR, leaf summaries, snapshot and proof-context object; the
semantic projection is checked for deterministic equality.

## Formal fixture mappings

- Symbolic `component-contract:<hex>` fixture aliases identify test inputs
  only. A canonical digest is computed from the completed 13-field contract;
  the Semantic IR pins that computed digest. Repeated aliases resolve to the
  same checked body in one case.
- `FailureA` and `FailureB` map to two exact registered TypeSchemaRefs; the
  projection maps those exact refs back to fixture labels only after checking.
- Effect atoms, typed qualifiers, bounds and guards are parsed into
  `af-effect` model types. The explicit effect/resource catalog recognizes
  the operation, resource type and complete qualifier profiles.
- Resource selectors materialize typed ResourceRequirements. Explicit slot
  labels in the input are retained; a missing selector mapping receives a
  deterministic child-scoped fixture slot. Such added R facts are not
  silently compared as if present in the old expected: a pilot must either
  assert them or explicitly mark that projection `NotApplicable`.
- Workflow `input:n` sources become typed `WorkflowInput(n)` refs, exact
  ComponentInputPorts and caller RunParameters/ValueSlots/Expressions. Taints
  are registered in the test catalog. Runtime `python` and sandbox `strict`
  become typed X requirements under explicit catalogs.
- SELECT uses a typed int64 ValueSlot expression for dynamic selectors and a
  typed int64 literal for the exact-constant case. The decision dependencies
  are recomputed by Phase 3C, not copied from Expected.
- `RequiresEvidence` is materialized as a real `CALL_SUBFLOW`: a checked
  exported contract is pinned by an exact target SemanticIrDigest, inserted
  into `SubflowIrUniverse`, and associated through the snapshot export API.
  The pilot uses an explicitly empty known Flow summary as test-only input;
  Flow/Failure/Usage/Resource fields outside this case's declared assertion
  schema are marked NotApplicable. It does not use a verdict flag to mint a
  leaf.
- The open-row case is a full ComponentContract validation input. It is
  rejected before SnapshotConstruction because the resource proof is
  Unproven. Separately, `af-effect::control::sequence_envelope` verifies that
  an open residual remains open; that lower-level assertion is not counted as
  ControlDerivation.

## Actual Projection and comparison

`ControlConformanceProjection` is a Rust test-only semantic view. It includes
the caller SemanticIrDigest, control node/operator/child order, typed output
interface, complete Effect identities/guards/bounds/residuals and occurrence
sources, nominal Failure TypeSchemaRefs, Usage phase/meter/numerical bounds,
Flow summaries and source/sink mappings, full Recovery clauses/action/usage
bindings, scoped Resource requirements/aliases, complete per-child Execution
requirements, dimension availability/proofs, inherited obligations and
overall verdict. Stable RuleIds/references are used instead of presentation
diagnostic prose.

The composer pilot checks every legacy expected field for its 13 ordinary
vectors using a fixed expected-key schema. The RequiresEvidence pilot declares
its 5 asserted Effect/proof fields and 7 NotApplicable fields explicitly.
The lower-level SELECT and ContractValidation cases have separate stage
schemas. `diff_json` reports nested semantic paths and fails if an asserted
actual field is missing. Comparator mutations operate on cloned Expected
values, never on the frozen corpus.

This pilot does not yet compare the expanded projection fields against
frozen expected values for all 58 vectors; those remain in the reachability
report as unmaterialized/not asserted, not as silently passed.

