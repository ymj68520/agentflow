# Phase 4B.2B.1A — Initial Coverage Audit

This is the read-only audit captured at the start of Phase 4B.2B.1A, before
the differential harness changes. Baseline: `a00fe81dac530b2ec61b40becbbcecc2c9793f7a`.

## Aggregate findings

- The manifest declares 58 cases and the eight case directories contain 58
  JSON files with 58 unique `case_id` values.
- All 58 files contain an `input` object and an `expected` object. However,
  **0/58** inputs is a complete formal `SemanticIrDocument` plus checked
  Phase 3C evidence, exact checked contracts, snapshot, and proof context.
  The inputs are abstract child/row descriptions or meta-equivalence
  descriptions; fixture digests such as `semantic-ir:caller` and
  `component-contract:aaaa...` are symbolic, not valid computed pins.
- **0/58** expected objects is a complete, schema-declared semantic
  projection. The shapes vary by operation; several omit entire dimensions,
  and none marks omitted dimensions as `NotApplicable`.
- The existing Rust test parses the files, collects the unique case IDs, and
  collects expected verdict strings into a set. It asserts the case count and
  presence of four verdict classes, but does not compare the manifest ID set
  and does not call `derive_control_contract` for any fixture.
- Therefore 58 cases are structurally read, 0 are production-analyzer
  executions, 0 are semantic expected/actual comparisons, and 58 are
  unexecuted for differential-conformance purposes. No per-case tests are
  marked `ignored` or skipped; the missing execution is a missing harness,
  not a test filter.
- The Rust test does not import or invoke Python expected-result code. Python
  and Rust share the frozen JSON artifacts only; there is no shared runtime
  expected calculation. This establishes code separation, not historical
  oracle-first ordering.
- No fixture has yet been shown impossible to express using the frozen IR.
  The four `equivalence` vectors are meta-cases and require two real analyzer
  runs/projections; `parallel_output`, `namespace_alias`, and
  `unknown_control` likewise need explicit formal-IR materialization. Until
  the adapter executes them, representability is unverified, not assumed.

In the per-case table, “abstract only” means a JSON input exists but lacks
formal checked inputs; “partial” means an expected object exists but is not a
complete projection with explicit `NotApplicable` fields. No actual semantic
fields were compared at this baseline.

| CaseId | Category | Input Complete? | Expected Complete? | Rust Actually Executed? | Semantic Fields Compared? | Status |
|---|---|---|---|---|---|---|
| `computation-control/golden/minimal-known-sequence-envelope` | golden | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/negative/effect-overflow` | negative | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/sequence/empty-failure-not-success` | sequence | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/sequence/first-failure-can-skip-next` | sequence | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/sequence/forward-recovery-phases-separate` | sequence | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/sequence/same-atom-occurrences-distinct` | sequence | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/sequence/second-lower-conditional` | sequence | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/sequence/two-effects-accumulate` | sequence | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/sequence/typed-cross-sibling-edge` | sequence | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/sequence/typed-output-last-child` | sequence | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/sequence/unknown-first-child` | sequence | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/sequence/usage-checked-sum` | sequence | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/select/decision-input-influence` | select | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/select/different-guards-lower-zero` | select | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/select/exact-constant-case` | select | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/select/exclusive-effect-join` | select | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/select/qualifier-members-distinct` | select | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/select/taints-union` | select | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/select/typed-merge-mismatch` | select | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/select/unknown-branch-not-pure` | select | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/select/unmatched-ordinal-unproven` | select | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/select/usage-maximum` | select | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/parallel/all-effects-accumulate` | parallel | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/parallel/all-usage-accumulates` | parallel | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/parallel/different-port-namespaces` | parallel | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/parallel/execution-per-child` | parallel | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/parallel/fail-fast-lower-zero` | parallel | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/parallel/fail-fast-sibling-effect-retained` | parallel | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/parallel/max-inflight-not-total-count` | parallel | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/parallel/output-explicit-branch-map` | parallel | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/parallel/same-contract-occurrences` | parallel | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/parallel/shared-slot-alias-explicit` | parallel | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/parallel/unknown-sibling-obligations` | parallel | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/race/all-contestants-not-winner-only` | race | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/race/cancel-not-no-effect` | race | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/race/loser-effect-retained` | race | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/race/loser-recovery-retained` | race | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/race/loser-usage-retained` | race | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/race/physical-timing-not-decision` | race | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/race/same-atom-distinct-occurrences` | race | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/race/unknown-loser-contaminates` | race | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/race/winner-may-fail` | race | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/race/winner-output-taints-union` | race | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/proof/authority-upper-not-permission` | proof | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/proof/cancellation-not-failure` | proof | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/proof/compensation-not-subtraction` | proof | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/proof/execution-not-worker-sum` | proof | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/proof/known-bound-with-evidence` | proof | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/proof/missing-usage-not-zero` | proof | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/proof/open-effect-row-remains-open` | proof | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/proof/requires-evidence-propagates` | proof | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/proof/unknown-effect-row-not-empty` | proof | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/proof/unproven-propagates` | proof | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/proof/unsupported-timeout-not-empty` | proof | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/equivalence/presentation-metadata-only` | equivalence | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/equivalence/same-snapshot-deterministic` | equivalence | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/equivalence/sequence-order-is-semantic` | equivalence | No — abstract only | No — partial | No | None | Inventory-only |
| `computation-control/equivalence/unordered-members-normalized` | equivalence | No — abstract only | No — partial | No | None | Inventory-only |

## Semantic fields not compared at baseline

The inventory-only test does not compare any Production Analyzer output.
Consequently none of the expected Effect identities/bounds/guards/residuals
or source occurrences; Failure identities/provenance; Usage coordinates,
phases, values or overflow; Flow sources, taints, constraints or authority;
Recovery actions; Resource aliases; Execution requirements; child identities
or order; or per-dimension proofs/obligations is verified against Rust.

