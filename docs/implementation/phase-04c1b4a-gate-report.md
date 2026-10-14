# Gate report

Scope: the 22 existing D9-Control CaseIds only. D10 independent value comparison remains Pending. Historical CaseIds and Expected are unchanged. Starting HEAD: `5aa4ef7b865b9255a29c83c8375e13803d5e5ce1`.

This evidence log is used only for documentation, never for Expected generation. Normal generation checks the frozen source graph before deriving values. The full static surface and historical Expected remain anchored to the starting HEAD.

```json
{
  "schema_version": 1,
  "starting_head": "5aa4ef7b865b9255a29c83c8375e13803d5e5ce1",
  "rust_workspace_gate_started_at_head": "4bee5e14af281c65740b5cc0a197f6ce8a958cd4",
  "rust_workspace_gate_retry_at_head": "f9ed0b9bcba299c0019cb1b3eb55781a9d596a30",
  "hand_trace_supplement_head": "6627f0cdb5ca884bb661da3d1f17c88c6daf525f",
  "all_gates": "passed",
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
    "status": "passed",
    "meaning": "retained comparator integrity self-tests, not independent semantic proofs"
  },
  "hand_calculations": {
    "count": 37,
    "portable_opcode_trees": 13,
    "incomplete_traces": 0,
    "status": "passed"
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
  "cargo_fmt": "passed",
  "cargo_clippy_all": "passed",
  "cargo_test_all": {
    "status": "passed",
    "committed_retry_head": "f9ed0b9bcba299c0019cb1b3eb55781a9d596a30",
    "command": "cargo test --all -j 2",
    "unit_integration_doc_tests": "passed",
    "historical_case_ids": 512,
    "historical_unique_case_ids": 512,
    "portable_static_surface": "passed unchanged",
    "retry_log": "target/phase-04c1b4a-cargo-test-all-retry.log",
    "d9_full_report": "target/conformance/d9-full/report.json",
    "input_mutation_report": "target/conformance/d9-input-mutations/report.json"
  },
  "initial_workspace_attempt": {
    "status": "failed_static_surface",
    "historical_512_case_differentials": "passed",
    "failing_test": "portable_static_revalidation_is_closed_and_stops_at_d10",
    "cause": "test-only filesystem runner was physically under src and matched the unchanged lexical production scan",
    "repair": "move runner to tests/support with cfg(test) path hook; no production semantics or scanner changes",
    "targeted_static_surface_recheck": "passed"
  },
  "spec_check": {
    "status": "passed",
    "specifications": 43,
    "normative_rules": 404,
    "rules_covered": 403,
    "intentional_exception": 1,
    "missing_coverage": 0,
    "abi_assignments": 775,
    "afidl_modules": 17,
    "hash_v1_domains": 14,
    "cross_reference_errors": 0
  },
  "generator_double_round": {
    "status": "passed",
    "generators": 28,
    "rounds": 2,
    "workspace_files_hashed": 1573,
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
