//! Unit tests for the ABI number consistency check: each rejection class of
//! `check_current_schema_against_abi_registry` exercised directly against a
//! compiled fixture schema (independent of the file-based corpus).

use af_spec::abi::{check_current_schema_against_abi_registry, schema_bindings};
use af_spec::model::AbiAssignmentJson;

const MINI_SCHEMA: &str = "afidl 1;\n\
     schema agentflow.test.mini @1;\n\
     record Widget { 1 id: string; 2 count: uint32; }\n\
     variant Lookup { 1 found: Widget; 2 missing: unit; }\n\
     enum Color { 1 red; 2 green; }\n";

fn row(type_name: &str, kind: &str, number: u32, name: &str, status: &str) -> AbiAssignmentJson {
    AbiAssignmentJson {
        schema_family: "agentflow.test.mini@1".into(),
        type_name: type_name.into(),
        number_kind: kind.into(),
        number,
        symbolic_name: name.into(),
        status: status.into(),
        first_seen_schema_digest: None,
    }
}

fn bindings() -> Vec<af_spec::abi::SchemaBinding> {
    let compiled = af_idl::compile(MINI_SCHEMA).unwrap();
    schema_bindings(&compiled)
}

/// The complete, fully active registry matching the fixture schema.
fn full_rows() -> Vec<AbiAssignmentJson> {
    vec![
        row("Widget", "field", 1, "id", "active"),
        row("Widget", "field", 2, "count", "active"),
        row("Lookup", "variant_case", 1, "found", "active"),
        row("Lookup", "variant_case", 2, "missing", "active"),
        row("Color", "enum_value", 1, "red", "active"),
        row("Color", "enum_value", 2, "green", "active"),
    ]
}

fn kinds(errors: &[af_spec::SpecError]) -> Vec<&'static str> {
    errors.iter().map(af_spec::SpecError::kind).collect()
}

#[test]
fn active_registry_matching_the_schema_is_clean() {
    let errors = check_current_schema_against_abi_registry(
        "agentflow.test.mini@1",
        &bindings(),
        &full_rows(),
    );
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn a_number_bound_to_a_new_name_is_a_reassignment() {
    let mut rows = full_rows();
    rows[0] = row("Widget", "field", 1, "legacy_id", "active");
    let errors =
        check_current_schema_against_abi_registry("agentflow.test.mini@1", &bindings(), &rows);
    // The reassignment is the headline finding; the renamed active row
    // necessarily also lacks a schema counterpart.
    assert!(
        kinds(&errors).contains(&"abi_number_reassigned"),
        "{errors:?}"
    );
}

#[test]
fn a_retired_number_still_in_use_is_rejected() {
    let mut rows = full_rows();
    rows[0] = row("Widget", "field", 1, "id", "retired");
    let errors =
        check_current_schema_against_abi_registry("agentflow.test.mini@1", &bindings(), &rows);
    assert_eq!(kinds(&errors), ["abi_retired_number_in_use"]);
}

#[test]
fn a_retired_number_reused_for_new_meaning_is_rejected() {
    let mut rows = full_rows();
    rows[0] = row("Widget", "field", 1, "legacy_id", "retired");
    let errors =
        check_current_schema_against_abi_registry("agentflow.test.mini@1", &bindings(), &rows);
    assert_eq!(kinds(&errors), ["abi_number_reassigned"]);
}

#[test]
fn a_reserved_number_used_by_the_schema_is_rejected() {
    let mut rows = full_rows();
    rows[0] = row("Widget", "field", 1, "reserved_slot", "reserved");
    let errors =
        check_current_schema_against_abi_registry("agentflow.test.mini@1", &bindings(), &rows);
    assert_eq!(kinds(&errors), ["abi_reserved_number_in_use"]);
}

#[test]
fn an_unregistered_binding_is_rejected() {
    let mut rows = full_rows();
    rows.pop(); // Color enum_value 2 "green" loses its row
    let errors =
        check_current_schema_against_abi_registry("agentflow.test.mini@1", &bindings(), &rows);
    assert_eq!(kinds(&errors), ["abi_unregistered_assignment"]);
}

#[test]
fn a_name_moved_to_a_new_number_is_a_rebind() {
    // `id` is registered (retired) at field 1, but the schema now uses `id`
    // at a brand-new number 3 with no row: name rebound, not unregistered.
    let compiled = af_idl::compile(
        "afidl 1;\n schema agentflow.test.mini @1;\n record Widget { 2 count: uint32; 3 id: string; }\n",
    )
    .unwrap();
    let rows = vec![
        row("Widget", "field", 1, "id", "retired"),
        row("Widget", "field", 2, "count", "active"),
    ];
    let errors = check_current_schema_against_abi_registry(
        "agentflow.test.mini@1",
        &schema_bindings(&compiled),
        &rows,
    );
    assert_eq!(kinds(&errors), ["abi_name_rebound"]);
}

#[test]
fn an_active_row_without_schema_counterpart_is_rejected() {
    let mut rows = full_rows();
    rows.push(row("Widget", "field", 7, "ghost", "active"));
    let errors =
        check_current_schema_against_abi_registry("agentflow.test.mini@1", &bindings(), &rows);
    assert_eq!(kinds(&errors), ["abi_active_assignment_missing"]);
}

#[test]
fn a_family_without_a_module_loses_all_active_rows() {
    // Empty bindings (module deleted) + active rows -> every row missing.
    let errors =
        check_current_schema_against_abi_registry("agentflow.test.mini@1", &[], &full_rows());
    assert_eq!(kinds(&errors), ["abi_active_assignment_missing"; 6]);
}

#[test]
fn schema_bindings_cover_all_three_number_kinds() {
    let all = bindings();
    assert_eq!(all.iter().filter(|b| b.number_kind == "field").count(), 2);
    assert_eq!(
        all.iter()
            .filter(|b| b.number_kind == "variant_case")
            .count(),
        2
    );
    assert_eq!(
        all.iter().filter(|b| b.number_kind == "enum_value").count(),
        2
    );
}
