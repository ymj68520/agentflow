# Phase 4B.2B.1B — Case Reachability

This is a test-only reachability inventory: not a wire ABI and not a runtime model.

Baseline: `1633604cf0e3ae27dc0d9e2592ce466c1c3ee82e`. The classification is
for corpus revision 3. “Executed” means the listed production stage was
actually invoked by the Rust pilot, not merely that a JSON file was read.
`UNMATERIALIZED_FIXTURE` means no conclusion about model reachability has yet
been demonstrated; it is not counted as passed or skipped.

## Aggregate

| Classification | Count | Actually executed |
|---|---:|---:|
| `REACHABLE_CONTROL` | 16 | 16 `derive_control_contract` cases |
| `EXPECTED_EARLY_REJECTION` | 2 | 1 contract validation; 1 IR rejection not yet run |
| `REACHABLE_LOWER_LEVEL_ONLY` | 1 | 1 formal Effect algebra comparison |
| `UNREACHABLE_MODEL_GAP` | 0 | — |
| `UNMATERIALIZED_FIXTURE` | 39 | 0 |
| **Total** | **58** | **18 CaseIds reached their classified stage** |

Of the two early-rejection vectors, the open-row case runs through Contract
Validation. The typed-merge mismatch is expected to terminate at IR
Validation but does not yet have a production IR fixture. Thus 40 CaseIds
have no actual stage execution: 39 unmaterialized fixtures plus that pending
early-rejection fixture.

## Per-CaseId classification

| CaseId | Expected terminal stage | Reachability | Executed? | Fully compared? | Blocker / note |
|---|---|---|---:|---:|---|
| `computation-control/equivalence/presentation-metadata-only` | ControlDerivation pair | UNMATERIALIZED_FIXTURE | No | No | No paired formal documents yet |
| `computation-control/equivalence/same-snapshot-deterministic` | ControlDerivation | REACHABLE_CONTROL | Yes | Yes | Two derivations use the same checked IR, snapshot and proof-context object |
| `computation-control/equivalence/sequence-order-is-semantic` | ControlDerivation pair | UNMATERIALIZED_FIXTURE | No | No | Reversed formal Sequence documents not built |
| `computation-control/equivalence/unordered-members-normalized` | ControlDerivation pair | UNMATERIALIZED_FIXTURE | No | No | Paired normalized contract fixtures not built |
| `computation-control/golden/minimal-known-sequence-envelope` | ControlDerivation | REACHABLE_CONTROL | Yes | Yes | — |
| `computation-control/negative/effect-overflow` | ControlDerivation | REACHABLE_CONTROL | Yes | Yes | Valid checked children; production summary reports Effect arithmetic Violation |
| `computation-control/parallel/all-effects-accumulate` | ControlDerivation | REACHABLE_CONTROL | Yes | Yes | — |
| `computation-control/parallel/all-usage-accumulates` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | — |
| `computation-control/parallel/different-port-namespaces` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | Scoped two-resource-slot IR not built |
| `computation-control/parallel/execution-per-child` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | — |
| `computation-control/parallel/fail-fast-lower-zero` | ControlDerivation | REACHABLE_CONTROL | Yes | Yes | — |
| `computation-control/parallel/fail-fast-sibling-effect-retained` | ControlDerivation | REACHABLE_CONTROL | Yes | Yes | — |
| `computation-control/parallel/max-inflight-not-total-count` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | — |
| `computation-control/parallel/output-explicit-branch-map` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | Typed output IR adapter not built |
| `computation-control/parallel/same-contract-occurrences` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | Source contract declares missing Flow as unknown; no checked ComponentContract fixture yet |
| `computation-control/parallel/shared-slot-alias-explicit` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | — |
| `computation-control/parallel/unknown-sibling-obligations` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | Unresolved exact-pin case adapter not yet compared |
| `computation-control/proof/authority-upper-not-permission` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | — |
| `computation-control/proof/cancellation-not-failure` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | — |
| `computation-control/proof/compensation-not-subtraction` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | Exact recovery actions/bindings not yet materialized |
| `computation-control/proof/execution-not-worker-sum` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | — |
| `computation-control/proof/known-bound-with-evidence` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | — |
| `computation-control/proof/missing-usage-not-zero` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | — |
| `computation-control/proof/open-effect-row-remains-open` | ContractValidation | EXPECTED_EARLY_REJECTION | Yes | Yes | Open row is Unproven at AF-CONTRACT-0013; no CheckedComponentContract is produced |
| `computation-control/proof/requires-evidence-propagates` | ControlDerivation | REACHABLE_CONTROL | Yes | Yes | Real CALL_SUBFLOW/export path; non-asserted Flow/R/U fields explicitly N/A in pilot schema |
| `computation-control/proof/unknown-effect-row-not-empty` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | Unresolved exact pin needs full expected comparison |
| `computation-control/proof/unproven-propagates` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | — |
| `computation-control/proof/unsupported-timeout-not-empty` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | TIMEOUT is checked IR but unsupported control algebra |
| `computation-control/race/all-contestants-not-winner-only` | ControlDerivation | REACHABLE_CONTROL | Yes | Yes | — |
| `computation-control/race/cancel-not-no-effect` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | — |
| `computation-control/race/loser-effect-retained` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | — |
| `computation-control/race/loser-recovery-retained` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | Exact recovery actions and coverage catalog not yet materialized |
| `computation-control/race/loser-usage-retained` | ControlDerivation | REACHABLE_CONTROL | Yes | Yes | Actual finite upper compared as 300 |
| `computation-control/race/physical-timing-not-decision` | ControlDerivation | REACHABLE_CONTROL | Yes | Yes | — |
| `computation-control/race/same-atom-distinct-occurrences` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | — |
| `computation-control/race/unknown-loser-contaminates` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | Unresolved exact-pin case adapter not yet compared |
| `computation-control/race/winner-may-fail` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | — |
| `computation-control/race/winner-output-taints-union` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | Input aliases `input:a/b` need explicit formal source mapping |
| `computation-control/select/decision-input-influence` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | — |
| `computation-control/select/different-guards-lower-zero` | EffectAlgebraValidation | REACHABLE_LOWER_LEVEL_ONLY | Yes | Yes, Effect only | Formal `af-effect::control::select_envelope`; no claim of ControlDerivation |
| `computation-control/select/exact-constant-case` | ControlDerivation | REACHABLE_CONTROL | Yes | Yes | Constant int64 ordinal is checked in IR |
| `computation-control/select/exclusive-effect-join` | ControlDerivation | REACHABLE_CONTROL | Yes | Yes | Dynamic selector obligation comes from checked IR/dataflow |
| `computation-control/select/qualifier-members-distinct` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | — |
| `computation-control/select/taints-union` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | — |
| `computation-control/select/typed-merge-mismatch` | IrValidation | EXPECTED_EARLY_REJECTION | No | No | Typed mismatch IR materializer/error comparison pending |
| `computation-control/select/unknown-branch-not-pure` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | — |
| `computation-control/select/unmatched-ordinal-unproven` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | — |
| `computation-control/select/usage-maximum` | ControlDerivation | REACHABLE_CONTROL | Yes | Yes | Actual maximum compared |
| `computation-control/sequence/empty-failure-not-success` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | — |
| `computation-control/sequence/first-failure-can-skip-next` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | — |
| `computation-control/sequence/forward-recovery-phases-separate` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | — |
| `computation-control/sequence/same-atom-occurrences-distinct` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | — |
| `computation-control/sequence/second-lower-conditional` | ControlDerivation | REACHABLE_CONTROL | Yes | Yes | Child 3 lower is compared after production composition |
| `computation-control/sequence/two-effects-accumulate` | ControlDerivation | REACHABLE_CONTROL | Yes | Yes | Full Atom/occurrence and upper/lower comparison |
| `computation-control/sequence/typed-cross-sibling-edge` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | Producer/output/input SSA fixture not yet built |
| `computation-control/sequence/typed-output-last-child` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | Typed output fixture not yet built |
| `computation-control/sequence/unknown-first-child` | ControlDerivation | UNMATERIALIZED_FIXTURE | No | No | Unresolved exact-pin input needs full expected comparison |
| `computation-control/sequence/usage-checked-sum` | ControlDerivation | REACHABLE_CONTROL | Yes | Yes | Exact finite upper compared |

No current CaseId is classified `UNREACHABLE_MODEL_GAP`: the former open-row
parent-composition expectation was explicitly reclassified in corpus revision
3 as a ContractValidation early rejection, while the old algebra claim is
tested separately at EffectAlgebraValidation. This does not establish
reachability for the 39 `UNMATERIALIZED_FIXTURE` cases.

