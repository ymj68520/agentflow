# Gate report

Scope: the 22 existing D9-Control CaseIds only. D10 independent value comparison remains Pending. Historical CaseIds and Expected are unchanged. Starting HEAD: `5aa4ef7b865b9255a29c83c8375e13803d5e5ce1`.

This evidence log is used only for documentation, never for Expected generation. Normal generation checks the frozen source graph before deriving values. The full static surface and historical Expected remain anchored to the starting HEAD.

```json
{
  "schema_version": 1,
  "starting_head": "5aa4ef7b865b9255a29c83c8375e13803d5e5ce1",
  "all_gates": "pending",
  "d9_control": {
    "declared": 22,
    "executed": 22,
    "fully_compared": 22,
    "skipped": 0,
    "unexpected_mismatch": 0
  },
  "field_instances": {
    "expected_required": 120117,
    "actual_required": 120117,
    "compared_required": 120117,
    "diagnostic_only": 2683,
    "missing_expected": 0,
    "missing_actual": 0,
    "unexpected": 0,
    "uncompared": 0
  },
  "new_comparator_mutations": {
    "count": 42,
    "status": "passed"
  },
  "legacy_comparator_mutations": {
    "count": 68,
    "status": "pending_current_workspace_gate"
  },
  "production_input_mutations": {
    "count": 12,
    "d9_full_compared": 11,
    "formal_d6_rejections": 1,
    "status": "passed"
  },
  "type_graph": {
    "records_and_sums": 264,
    "nominal_u32_terminals": 6,
    "member_and_variant_slots": 1079,
    "required": 1074,
    "diagnostic_only": 5
  },
  "d10_independent_values": "Pending",
  "other_56_case_ids": "Pending independent full-field Oracle",
  "cargo_fmt": "pending",
  "cargo_clippy_all": "pending",
  "cargo_test_all": "pending",
  "spec_check": "pending",
  "generator_double_round": {
    "status": "passed",
    "generators": 28,
    "rounds": 2,
    "workspace_files_hashed": 1572,
    "byte_drift": 0
  },
  "frozen_asset_drift": {
    "status": "passed",
    "baseline": "5aa4ef7b865b9255a29c83c8375e13803d5e5ce1",
    "existing_versioned_paths_changed": 0
  },
  "confirmed_new_findings": [],
  "oracle_review_notes": [
    "Partial child Flow must retain child identity even with an empty subtotal unresolved list; corrected by the frozen availability rule",
    "New fixture operation names and re-addressed asset ordering must satisfy frozen input grammar",
    "ResourceTypeRef projects its full schema pin directly; the schema/generator binding was audited"
  ]
}
```

All current generators must run twice and produce identical bytes. Full workspace gates use build parallelism two on Windows. Cargo regression success does not close the still-Pending D10/Recovery proof models.
