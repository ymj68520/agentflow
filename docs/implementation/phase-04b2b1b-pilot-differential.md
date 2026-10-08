# Phase 4B.2B.1B — Pilot Differential Results

This is a test-only verification report: not a wire ABI and not a runtime model.

The report is emitted by the Rust test `control_conformance::production_pilot_reports_stage_aware_conformance_counts` against corpus revision 3. Its input builder and comparator live in `tests/support/control_fixture.rs` and `tests/control_conformance.rs`.

## Actual stage counts

| Category | Manifest | ControlDerivation executed | Expected early rejection executed | Lower-level only executed | Fully compared at stated stage | Unmaterialized |
|---|---:|---:|---:|---:|---:|---:|
| golden | 1 | 1 | 0 | 0 | 1 | 0 |
| negative | 1 | 1 | 0 | 0 | 1 | 0 |
| sequence | 10 | 3 | 0 | 0 | 3 | 7 |
| select | 10 | 3 | 0 | 1 | 4 | 5 |
| parallel | 11 | 3 | 0 | 0 | 3 | 8 |
| race | 10 | 3 | 0 | 0 | 3 | 7 |
| proof | 11 | 1 | 1 | 0 | 2 | 9 |
| equivalence | 4 | 1 | 0 | 0 | 1 | 3 |
| **Total** | **58** | **16** | **1** | **1** | **18** | **39** |

One additional CaseId, `select/typed-merge-mismatch`, is classified as an
expected IRValidation rejection but has not yet been executed; it is separate
from the 39 unmaterialized fixtures. Thus 18 original CaseIds reached an
actual stage, 40 have not. There were zero unexpected mismatches among the
18 executed stage-specific comparisons; this is not a 58/58 result.

The 16 ControlDerivation CaseIds are:

- golden: `golden/minimal-known-sequence-envelope`
- sequence: `sequence/two-effects-accumulate`, `sequence/second-lower-conditional`, `sequence/usage-checked-sum`
- select: `select/exclusive-effect-join`, `select/usage-maximum`, `select/exact-constant-case`
- parallel: `parallel/all-effects-accumulate`, `parallel/fail-fast-sibling-effect-retained`, `parallel/fail-fast-lower-zero`
- race: `race/all-contestants-not-winner-only`, `race/loser-usage-retained`, `race/physical-timing-not-decision`
- proof: `proof/requires-evidence-propagates`
- negative: `negative/effect-overflow`
- equivalence: `equivalence/same-snapshot-deterministic`

The guarded SELECT vector `select/different-guards-lower-zero` reached the
formal `af-effect::control::select_envelope` and its full Effect expected was
compared. It is explicitly lower-level-only, not a SELECT control-tree run.
`proof/open-effect-row-remains-open` ran ContractValidation and was rejected
before a checked component/snapshot could be produced; a separate formal
Effect algebra assertion verified the open residual remains open.

## Compared fields and discovered differences

For 13 ordinary control vectors, the comparator enforces the exact expected
key schema and recursively compares all legacy fields: complete Effect rows,
occurrences, lower-bound adjustments, Failure aliases, Usage values, Flow
summary, K/R/X source rows, unresolved children and overall verdict. The
RequiresEvidence pilot asserts Effect/occurrence/closedness/overall verdict
and the inherited subflow-body obligation; its legacy Flow/Failure/Usage/R/X
fields are explicitly NotApplicable for this materialization. The open-row
case compares ContractValidation stage, verdict, `checked_component_produced
= false`, and the failing dimension/rule/reason.

The richer Actual projection also emits typed failure refs, Flow port/sink
mappings, recovery clauses/action/usage bindings, full scoped resource
requirements and aliases, full ExecutionRequirements, dimension availability,
per-dimension RuleIds and inherited obligations. Those fields are not yet
paired with frozen expected values for every pilot and remain explicit
NotYetRepresentable/NotAsserted work for Phase 4B.2B.1C.

Two revision-2 Oracle expectations disagreed with production and were fixed in
revision 3:

1. `select/exact-constant-case` listed all branch lower-bound adjustments;
   production selects the exact ordinal, so no unselected child is adjusted.
   Expected is now `[]`.
2. `negative/effect-overflow` encoded a synthetic `arithmetic` field. The
   production analyzer returns a summary with Unknown Effect contributions
   and an Effect-envelope Violation, not an error payload. Expected now
   represents the known contributions plus the Violation verdict.

The open-row vector was also intentionally reclassified for reachability. Its
revision-2 Expected described a ControlDerivation result with an open parent
Effect row. The exact raw contract instead stops at ContractValidation: the
resource dimension is Unproven under `AF-CONTRACT-0013`,
`CheckedComponentContract` is not produced, and no snapshot/control call is
permitted. Revision 3 Expected is therefore `stage=contract_validation`,
`verdict=unproven`, `checked_component_produced=false`, with the exact
resources/cross-dimension proofs. The former open-row composition claim is
still checked only at the lower-level formal Effect algebra.

No production algebra defect was found in the pilot. Failure alias order is
normalized in the test projection because Failure rows are sets; complete
nominal TypeSchemaRefs are preserved in the rich projection.

## Comparator mutations

All five isolated mutations were rejected:

| Mutation | Actual differing semantic path |
|---|---|
| Remove RACE loser Effect | `race/all-contestants-not-winner-only.expected.effect_occurrences.length` |
| Change finite Usage upper | `race/loser-usage-retained.expected.usage.coordinates.forward/api_calls` |
| Downgrade RequiresEvidence | `proof/requires-evidence-propagates.expected.verdict` |
| Change Effect qualifier | `sequence/two-effects-accumulate.expected.effects.entries[0].atom.qualifiers[0].member` |
| Remove registered manifest CaseId | manifest/directory CaseId-set mismatch |

Mutation inputs are cloned in memory; the frozen corpus is not modified.

