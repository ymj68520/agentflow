# Phase 4C.1B.4B — gate report

Starting HEAD: `c5b49850d9ba0dce75255f152498468fe8e93762`. This phase targets only 16 original D9-Recovery cases; the 22 Control baseline is retained. Closure depends on the recorded committed gates. D10 independent values are Pending for all 78; B01/B02 Closure and B03–B07 remain BLOCKING. No later phase or runtime/publication capability is created.

This gate log affects documentation only, never Expected semantics. Exact code-vs-document HEADs are distinguished; no costly test is claimed at a later doc-only HEAD. All current generators run twice with byte equality; original512, original4A Expected/37hands and all frozen wire/hash/production surfaces are preserved.

```json
{
  "schema_version": 1,
  "starting_head": "c5b49850d9ba0dce75255f152498468fe8e93762",
  "all_gates": "pending",
  "code_gate_head": "pending",
  "historical_512": "pending",
  "d9_control_22": "pending",
  "d9_recovery_16": "pending_fresh_production_run",
  "recovery_input_mutations": "pending",
  "recovery_comparator_mutations": "pending",
  "hand_calculations": "independent generator checks; pending Rust gate",
  "fmt": "pending",
  "clippy": "pending",
  "cargo_test_all": "pending",
  "spec_check": "pending",
  "generator_double_round": "pending",
  "frozen_asset_drift": "pending",
  "d10_independent_values": "Pending for all 78",
  "B01_B02_conformance_closure": "BLOCKING",
  "B03_B07": "BLOCKING",
  "findings": {
    "F01": "CLOSED",
    "F02": "CLOSED",
    "F03": "OPEN/P2",
    "F04": "OPEN/P2",
    "new_confirmed_production": []
  },
  "oracle_review": [
    "local action dimension preserves positive eligibility evidence, separate from arithmetic certainty",
    "Root-relative occurrences retain enclosing family annotations without applying their multiplier twice",
    "Retry safety is also a Recovery dimension proof; cross-source aliases retain all mapped occurrences",
    "negative Usage fixtures must declare finite meter premises; EMPTY exhaustive meter sets are not missing meters",
    "unsupported aggregation uses the registered unsupported enum, not an invented opaque literal"
  ]
}
```

| Real input vector | Expected stage |
| --- | --- |
| forward-upper-changed | D10 |
| activation-changed | D10 |
| attempts-changed | D10 |
| action-kind-changed | D10 |
| action-atom-changed | D10 |
| action-guard-changed | D6 |
| recovery-meter-changed | D10 |
| recovery-profile-changed | D10 |
| explicit-zero-profile | D10 |
| retry-factor-changed | D10 |
| iterate-factor-changed | D10 |
| compensation-nesting-changed | D10 |
| source-association-changed | D10 |
| missing-action-binding | D6 |
| missing-usage-binding | D6 |
| wrong-forward-identity | D6 |
| wrong-profile-identity | D6 |
| activation-overflow | D6 |
| unbounded-action-requirement | D6 |
| missing-required-meter | D6 |
| unsupported-aggregation | D6 |

