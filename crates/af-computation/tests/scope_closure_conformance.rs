//! Independent Python oracle versus the production Scope/CompensationScope
//! summaries and full checked control-tree closure.
#![recursion_limit = "512"]

#[allow(dead_code)]
mod support;

use std::collections::BTreeSet;
use std::path::PathBuf;

use serde_json::{Value, json};
use support::control_fixture::analyze_compose_case;

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../spec/conformance/computation-scope-closure")
}

fn corpus_cases() -> Vec<(PathBuf, Value)> {
    let mut cases = Vec::new();
    for group in ["scope", "compensation", "nested", "negative"] {
        for entry in std::fs::read_dir(corpus_root().join(group)).unwrap() {
            let path = entry.unwrap().path();
            if path
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                let case: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
                cases.push((path, case));
            }
        }
    }
    cases.sort_by(|left, right| left.1["case_id"].as_str().cmp(&right.1["case_id"].as_str()));
    cases
}

fn compare_value(
    path: &str,
    expected: &Value,
    actual: Option<&Value>,
    differences: &mut Vec<String>,
) {
    let Some(actual) = actual else {
        differences.push(format!("{path}: actual field missing; expected {expected}"));
        return;
    };
    match (expected, actual) {
        (Value::Object(expected), Value::Object(actual)) => {
            for (key, value) in expected {
                compare_value(
                    &format!("{path}.{key}"),
                    value,
                    actual.get(key),
                    differences,
                );
            }
        }
        (Value::Array(expected), Value::Array(actual)) => {
            if expected.len() != actual.len() {
                differences.push(format!(
                    "{path}: expected {} items, actual {}",
                    expected.len(),
                    actual.len()
                ));
            }
            for (index, value) in expected
                .iter()
                .enumerate()
                .take(expected.len().min(actual.len()))
            {
                compare_value(
                    &format!("{path}[{index}]"),
                    value,
                    actual.get(index),
                    differences,
                );
            }
        }
        _ if expected != actual => {
            differences.push(format!("{path}: expected {expected}, actual {actual}"));
        }
        _ => {}
    }
}

fn production(case: &Value) -> Value {
    analyze_compose_case(case).unwrap_or_else(|error| {
        panic!(
            "{} did not reach production workflow closure: {error}",
            case["case_id"]
        )
    })
}

fn opcode_name(name: &str) -> &str {
    match name {
        "parallel_all" | "parallel_fail_fast" => "parallel",
        "iterate_foreach" | "iterate_while" => "iterate",
        other => other,
    }
}

#[test]
fn all_scope_and_compensation_scope_vectors_match_python_oracle() {
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(corpus_root().join("manifest.json")).unwrap())
            .unwrap();
    let cases = corpus_cases();
    let discovered = cases
        .iter()
        .map(|(_, case)| case["case_id"].as_str().unwrap().to_owned())
        .collect::<BTreeSet<_>>();
    let declared = manifest["case_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| id.as_str().unwrap().to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(manifest["case_count"], 55);
    assert_eq!(cases.len(), 55);
    assert_eq!(
        discovered, declared,
        "manifest must exactly cover production fixtures"
    );

    let mut compared_fields = 0;
    let mut scope_cases = 0;
    let mut compensation_cases = 0;
    let mut nested_cases = 0;
    let mut negative_cases = 0;
    let mut opcodes = BTreeSet::new();
    for (_, case) in &cases {
        let actual = production(case);
        let expected = case["expected"].as_object().unwrap();
        let mut differences = Vec::new();
        for (field, value) in expected {
            compare_value(
                &format!("{}.expected.{field}", case["case_id"]),
                value,
                actual.get(field),
                &mut differences,
            );
            compared_fields += 1;
        }
        assert!(
            differences.is_empty(),
            "{}: {}",
            case["case_id"],
            differences.join("\n")
        );
        assert_eq!(
            actual["workflow_closure"], actual["_workflow_closure"],
            "{}",
            case["case_id"]
        );
        assert_eq!(
            actual["_same_snapshot_repeat_equal"], true,
            "{}",
            case["case_id"]
        );
        match case["group"].as_str().unwrap() {
            "scope" => scope_cases += 1,
            "compensation" => compensation_cases += 1,
            "nested" => nested_cases += 1,
            "negative" => negative_cases += 1,
            other => panic!("unexpected corpus group {other}"),
        }
        opcodes.insert(opcode_name(case["input"]["operator"].as_str().unwrap()).to_owned());
        for node in case["input"]["nested_nodes"]
            .as_array()
            .into_iter()
            .flatten()
        {
            opcodes.insert(opcode_name(node["operator"].as_str().unwrap()).to_owned());
        }
        for child in case["input"]["children"].as_array().unwrap() {
            opcodes.insert(
                match child["leaf_kind"].as_str() {
                    Some("await") => "await",
                    _ if child["proof_source"] == "subflow_body_evidence" => "call_subflow",
                    _ => "invoke",
                }
                .to_owned(),
            );
        }
    }
    assert_eq!(
        (
            scope_cases,
            compensation_cases,
            nested_cases,
            negative_cases
        ),
        (18, 18, 18, 1)
    );
    assert_eq!(
        opcodes,
        BTreeSet::from([
            "invoke".to_owned(),
            "await".to_owned(),
            "sequence".to_owned(),
            "select".to_owned(),
            "parallel".to_owned(),
            "race".to_owned(),
            "iterate".to_owned(),
            "retry".to_owned(),
            "timeout".to_owned(),
            "handle".to_owned(),
            "scope".to_owned(),
            "compensation_scope".to_owned(),
            "call_subflow".to_owned(),
        ])
    );
    println!(
        "Scope/CompensationScope conformance: manifest=55; production=55; full-tree closure=55; compared_fields={compared_fields}; mismatch=0"
    );
}

fn assert_expected_mutation_detected(case_id: &str, mutate: impl FnOnce(&mut Value)) {
    let case = corpus_cases()
        .into_iter()
        .map(|(_, case)| case)
        .find(|case| case["case_id"] == case_id)
        .unwrap();
    let actual = production(&case);
    let mut expected = case["expected"].clone();
    mutate(&mut expected);
    let mut differences = Vec::new();
    compare_value(
        "mutated_expected",
        &expected,
        Some(&actual),
        &mut differences,
    );
    assert!(
        !differences.is_empty(),
        "oracle mutation was not detected for {case_id}"
    );
}

#[test]
fn scope_compensation_and_closure_mutations_are_detected() {
    assert_expected_mutation_detected(
        "computation-scope-closure/scope/resource-boundary-04",
        |expected| expected["scope_boundary"]["resource_attenuation"] = json!("violation"),
    );
    assert_expected_mutation_detected(
        "computation-scope-closure/compensation/action-bound-06",
        |expected| {
            expected["compensation_scope"]["potential_actions"]["actions"][0]["potential_effect"]
                ["upper"] = json!(999);
        },
    );
    assert_expected_mutation_detected("computation-scope-closure/nested/tree-01", |expected| {
        expected["workflow_closure"]["computation_order"] = json!([1, 3, 2]);
    });
    assert_expected_mutation_detected("computation-scope-closure/nested/tree-15", |expected| {
        expected["workflow_closure"]["known_recovery_sources_accounted"] = json!(false);
    });
}
