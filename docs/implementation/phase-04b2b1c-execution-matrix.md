# Phase 4B.2B.1C — 58-Case Execution Matrix

Baseline: `dc497a6ec70d535ca0a31803f25e5a10feec2fc0`. Corpus revision: **4**. Stable CaseId count: **58**. The table is emitted from `full_58_case_production_differential_conformance`; the `asserted/N/A` column records fields actually compared and explicit stage-specific non-applicability, not merely fields found in JSON.

## Aggregate

| Category | Manifest | Stage-executed | Fully compared | Unexpected mismatch |
|---|---:|---:|---:|---:|
| golden | 1 | 1 | 1 | 0 |
| negative | 1 | 1 | 1 | 0 |
| sequence | 10 | 10 | 10 | 0 |
| select | 10 | 10 | 10 | 0 |
| parallel | 11 | 11 | 11 | 0 |
| race | 10 | 10 | 10 | 0 |
| proof | 11 | 11 | 11 | 0 |
| equivalence | 4 | 4 | 4 | 0 |
| **Total** | **58** | **58** | **58** | **0** |

| Terminal API class | CaseIds | Actual executions |
|---|---:|---:|
| `derive_control_contract` (including formal IR validation, Phase 3C, snapshot, leaf resolution) | 55 | 55 |
| ContractValidation early rejection | 1 | 1 |
| IRValidation early rejection | 1 | 1 |
| EffectAlgebraValidation only | 1 | 1 |

## Per-CaseId matrix

`asserted/N/A` is `actually compared fields / explicitly not-applicable fields`. All rows have zero uncovered fields and passed their stage-specific comparison. Corpus revision is 4 for every row.

| CaseId | Category | Rev | Expected entry → terminal; required API | Expected result | Asserted/N/A | Actual stage | Status |
|---|---|---:|---|---|---:|---|---|
| `computation-control/equivalence/presentation-metadata-only` | equivalence | 4 | SemanticIRValidation → SemanticIR/ControlDerivation; digest + formal analyzer | proven | 2/0 | SemanticIR/ControlDerivation | passed |
| `computation-control/equivalence/same-snapshot-deterministic` | equivalence | 4 | SemanticIRValidation → SemanticIR/ControlDerivation; digest + `derive_control_contract` twice | proven | 2/0 | SemanticIR/ControlDerivation | passed |
| `computation-control/equivalence/sequence-order-is-semantic` | equivalence | 4 | SemanticIRValidation → SemanticIR/ControlDerivation; digest + formal analyzer | violation | 2/0 | SemanticIR/ControlDerivation | passed |
| `computation-control/equivalence/unordered-members-normalized` | equivalence | 4 | SemanticIRValidation → SemanticIR/ControlDerivation; formal analyzer | proven | 2/0 | SemanticIR/ControlDerivation | passed |
| `computation-control/golden/minimal-known-sequence-envelope` | golden | 4 | IRValidation → ControlDerivation; `derive_control_contract` | proven | 12/0 | ControlDerivation | passed |
| `computation-control/negative/effect-overflow` | negative | 4 | IRValidation → ControlDerivation; `derive_control_contract` | violation | 12/0 | ControlDerivation | passed |
| `computation-control/parallel/all-effects-accumulate` | parallel | 4 | IRValidation → ControlDerivation; `derive_control_contract` | proven | 12/0 | ControlDerivation | passed |
| `computation-control/parallel/all-usage-accumulates` | parallel | 4 | IRValidation → ControlDerivation; `derive_control_contract` | proven | 12/0 | ControlDerivation | passed |
| `computation-control/parallel/different-port-namespaces` | parallel | 4 | IRValidation → ControlDerivation; `derive_control_contract` | proven | 4/0 | ControlDerivation | passed |
| `computation-control/parallel/execution-per-child` | parallel | 4 | IRValidation → ControlDerivation; `derive_control_contract` | proven | 12/0 | ControlDerivation | passed |
| `computation-control/parallel/fail-fast-lower-zero` | parallel | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/parallel/fail-fast-sibling-effect-retained` | parallel | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/parallel/max-inflight-not-total-count` | parallel | 4 | IRValidation → ControlDerivation; `derive_control_contract` | proven | 12/0 | ControlDerivation | passed |
| `computation-control/parallel/output-explicit-branch-map` | parallel | 4 | IRValidation → ControlDerivation; `SemanticIrDocument::check` + `analyze_dataflow` + `derive_control_contract` | proven | 2/0 | ControlDerivation | passed |
| `computation-control/parallel/same-contract-occurrences` | parallel | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/parallel/shared-slot-alias-explicit` | parallel | 4 | IRValidation → ControlDerivation; `derive_control_contract` | proven | 12/0 | ControlDerivation | passed |
| `computation-control/parallel/unknown-sibling-obligations` | parallel | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/proof/authority-upper-not-permission` | proof | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/proof/cancellation-not-failure` | proof | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/proof/compensation-not-subtraction` | proof | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/proof/execution-not-worker-sum` | proof | 4 | IRValidation → ControlDerivation; `derive_control_contract` | proven | 12/0 | ControlDerivation | passed |
| `computation-control/proof/known-bound-with-evidence` | proof | 4 | IRValidation → ControlDerivation; `derive_control_contract` | requires_evidence | 12/0 | ControlDerivation | passed |
| `computation-control/proof/missing-usage-not-zero` | proof | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/proof/open-effect-row-remains-open` | proof | 4 | ContractValidation → ContractValidation; `analyze_contract` + `check_contract` | unproven | 11/0 | ContractValidation | passed |
| `computation-control/proof/requires-evidence-propagates` | proof | 4 | IRValidation → ControlDerivation; `derive_control_contract` | requires_evidence | 12/0 | ControlDerivation | passed |
| `computation-control/proof/unknown-effect-row-not-empty` | proof | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/proof/unproven-propagates` | proof | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/proof/unsupported-timeout-not-empty` | proof | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 6/0 | ControlDerivation | passed |
| `computation-control/race/all-contestants-not-winner-only` | race | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/race/cancel-not-no-effect` | race | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/race/loser-effect-retained` | race | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/race/loser-recovery-retained` | race | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/race/loser-usage-retained` | race | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/race/physical-timing-not-decision` | race | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/race/same-atom-distinct-occurrences` | race | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/race/unknown-loser-contaminates` | race | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/race/winner-may-fail` | race | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/race/winner-output-taints-union` | race | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/select/decision-input-influence` | select | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/select/different-guards-lower-zero` | select | 4 | EffectAlgebraValidation → EffectAlgebraValidation; `select_envelope` | unproven | 1/11* | EffectAlgebraValidation | passed |
| `computation-control/select/exact-constant-case` | select | 4 | IRValidation → ControlDerivation; `derive_control_contract` | proven | 12/0 | ControlDerivation | passed |
| `computation-control/select/exclusive-effect-join` | select | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/select/qualifier-members-distinct` | select | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/select/taints-union` | select | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/select/typed-merge-mismatch` | select | 4 | IrValidation → IrValidation; `SemanticIrDocument::check` | violation | 4/0 | IrValidation | passed |
| `computation-control/select/unknown-branch-not-pure` | select | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/select/unmatched-ordinal-unproven` | select | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/select/usage-maximum` | select | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/sequence/empty-failure-not-success` | sequence | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/sequence/first-failure-can-skip-next` | sequence | 4 | IRValidation → ControlDerivation; `derive_control_contract` | proven | 12/0 | ControlDerivation | passed |
| `computation-control/sequence/forward-recovery-phases-separate` | sequence | 4 | IRValidation → ControlDerivation; `derive_control_contract` | proven | 12/0 | ControlDerivation | passed |
| `computation-control/sequence/same-atom-occurrences-distinct` | sequence | 4 | IRValidation → ControlDerivation; `derive_control_contract` | proven | 12/0 | ControlDerivation | passed |
| `computation-control/sequence/second-lower-conditional` | sequence | 4 | IRValidation → ControlDerivation; `derive_control_contract` | proven | 12/0 | ControlDerivation | passed |
| `computation-control/sequence/two-effects-accumulate` | sequence | 4 | IRValidation → ControlDerivation; `derive_control_contract` | proven | 12/0 | ControlDerivation | passed |
| `computation-control/sequence/typed-cross-sibling-edge` | sequence | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 13/0 | ControlDerivation | passed |
| `computation-control/sequence/typed-output-last-child` | sequence | 4 | IRValidation → ControlDerivation; `check` + `analyze_dataflow` + `derive_control_contract` | proven | 2/0 | ControlDerivation | passed |
| `computation-control/sequence/unknown-first-child` | sequence | 4 | IRValidation → ControlDerivation; `derive_control_contract` | unproven | 12/0 | ControlDerivation | passed |
| `computation-control/sequence/usage-checked-sum` | sequence | 4 | IRValidation → ControlDerivation; `derive_control_contract` | proven | 12/0 | ControlDerivation | passed |

\* The lower-level SELECT case asserts the full Effect result. The other 11 Control-summary fields are explicitly NotApplicable because `EffectAlgebraValidation` does not produce a checked IR, leaf resolution, or `DerivedControlContractSummary`; it is not counted as a ControlDerivation run.

The typed-merge rejection is checked against `parallel_output_type_mismatch` and normative RuleId `AF-SEMANTIC-0056`; the case is counted at IRValidation and does not enter ControlDerivation. The open-row case separately checks ContractValidation and the formal open-row Effect algebra operation.
