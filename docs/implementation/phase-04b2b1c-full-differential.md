# Phase 4B.2B.1C — Full Differential Results

The dedicated repeatable entry point is:

```text
cargo test -p af-computation --test control_conformance full_58_case_production_differential_conformance -- --nocapture
```

The runner reads the manifest CaseIds, verifies exact equality with discovered fixture IDs, dispatches each fixture to its classified formal API, captures the result, and compares every required Expected field. It does not infer execution from corpus membership.

## Observed run

```text
Manifest cases                         58
Stage-executed                          58
  ControlDerivation                     55
  ContractValidation early rejection    1
  IRValidation early rejection           1
  EffectAlgebraValidation only           1
Fully compared                           58
Unmaterialized                            0
Skipped                                   0
Unexpected mismatch                      0
Uncovered declared fields                 0
Uncovered full-projection CaseIds         0
Comparator mutations detected            15 / 15
Additional overflow operator checks        3
```

The category totals are generated from the run, not copied from the manifest:

| Category | Manifest | Executed | Fully compared | Mismatch |
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

## Stage-specific checks

- `proof/open-effect-row-remains-open` enters `analyze_contract` and `check_contract`. It returns Unproven under AF-CONTRACT-0013 and AF-CONTRACT-0017 and does not mint a CheckedComponentContract. The separate Effect algebra assertion preserves the open residual; it is not counted as ControlDerivation.
- `select/typed-merge-mismatch` is a real typed PARALLEL output fixture. `SemanticIrDocument::check` rejects it with `parallel_output_type_mismatch`, RuleId AF-SEMANTIC-0056. No ControlDerivation call is made for that CaseId.
- The two positive typed output cases pass checked IR, Phase 3C dataflow, snapshot/leaf resolution, and `derive_control_contract`; their selected source-child/port and parent output type/slot are compared.
- `proof/unsupported-timeout-not-empty` builds a real checked TIMEOUT wrapper and invokes the production analyzer. Its summary remains unknown/Unproven and retains the known child Effect contribution rather than synthesizing an empty row.
- `sequence/typed-cross-sibling-edge` now carries a checked producer output slot into the consumer expression. Phase 3C and the production summary expose the exact producer/consumer/value-slot/expression dependency; Flow transfer remains Unproven.
- The other ordinary structured-control vectors pass checked ComponentContract validation, checked IR, Phase 3C dataflow, snapshot construction, leaf resolution, and production control derivation.

## Semantic checks

The control Expected comparator checks all fields declared by the corpus schema (12 for ordinary compose vectors; 13 for the cross-sibling dependency vector). It compares Effect rows and guards, full AtomKey qualifiers, occurrence provenance, conditional lower-bound adjustments, failure rows, Usage coordinates/phases, Flow summary, Recovery action/usage bindings, per-child Resource aliases, Execution requirements, unresolved children, and overall verdict. The lower-level-only SELECT vector asserts its Effect result; its 11 control-summary fields are explicitly NotApplicable because that API has no control tree or `DerivedControlContractSummary`.

The additional production-projection audit checks 23 semantic projection fields for every ControlDerivation CaseId (55/55). It verifies the checked SemanticIrDigest identity and semantic child order; every occurrence’s exact declared atom/guard/lower/upper, source contract/node/ordinal, caller digest, and mapped resource slot; exact FailureTypeRef schema identities; per-child Usage values without zero-defaulting unresolved leaves; Flow source/taint/authority identities; scoped resources; child-scoped sandbox/runtime demands; named per-dimension proofs and RuleIds; RequiresEvidence inheritance; and Recovery clauses, action identities, and recovery-Usage bindings. The two order-normalization equivalence witnesses each compare both formal derivations.

The key discriminator results are:

- RACE retains all contestant effects and their distinct occurrence identities. `loser-usage-retained` compares the actual finite forward upper **300** (100 + 200), not only a verdict.
- Guarded SELECT is executed only through `select_envelope`; its merged lower is zero, upper is three, and the unconditional guard is explicit. It does not count as a ControlDerivation.
- SEQUENCE weakens a later positive Effect lower bound when continuation is conditional. The cross-sibling case separately confirms the checked Phase 3C dependency.
- RequiresEvidence is retained in the parent verdict and inherited obligation, while known declared Effects remain present.
- Exact-constant SELECT obtains exact ordinal 1 from the checked IR constant expression, has no selector dependency slots, selects that branch, and creates no lower-bound adjustment. Runtime selectors have no `exact_case`, retain value-slot dependencies, and do not use static pruning.
- Sequence overflow is compared in the frozen negative vector. Additional actual Analyzer runs confirm Parallel and Race checked-add overflow preserve each known contribution and produce an Effect Violation. An exact-constant SELECT with a selected `u64::MAX` bound remains finite without an addition overflow.
- All 15 isolated comparator mutations were rejected at semantic paths; see [the mutation report](phase-04b2b1c-mutation-report.md).

No production control algebra, public API, AFIDL, ABI assignment, digest domain, runtime, or later control operator was changed.
