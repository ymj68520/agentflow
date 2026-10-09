//! Independent advanced-control oracle versus the full production analyzer.
//! These are static Semantic IR fixtures, not runtime simulations.
#![recursion_limit = "512"]

#[allow(dead_code)]
mod support;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use support::control_fixture::analyze_compose_case;

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../spec/conformance/computation-advanced-control")
}

fn corpus_cases() -> Vec<(PathBuf, Value)> {
    let mut cases = Vec::new();
    for group in ["iterate", "retry", "timeout", "handle", "negative"] {
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
                    "{path}: length expected {}, actual {}",
                    expected.len(),
                    actual.len()
                ));
            }
            for (index, expected_item) in expected
                .iter()
                .enumerate()
                .take(expected.len().min(actual.len()))
            {
                compare_value(
                    &format!("{path}[{index}]"),
                    expected_item,
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

fn actual_for(case: &Value) -> Value {
    let case_id = case["case_id"].as_str().unwrap();
    analyze_compose_case(case).unwrap_or_else(|error| {
        panic!("{case_id} did not reach production ControlDerivation: {error}")
    })
}

#[test]
fn all_72_advanced_control_vectors_run_through_production() {
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
    assert_eq!(manifest["case_count"], 72);
    assert_eq!(cases.len(), 72);
    assert_eq!(
        discovered, declared,
        "manifest must exactly cover production fixtures"
    );

    let mut executed_by_opcode = BTreeSet::new();
    let mut compared_fields = 0;
    for (_path, case) in &cases {
        let case_id = case["case_id"].as_str().unwrap();
        let actual = actual_for(case);
        assert_eq!(
            actual["_execution_stage"], "control_derivation",
            "{case_id}"
        );
        assert_eq!(actual["_same_snapshot_repeat_equal"], true, "{case_id}");
        let expected = case["expected"].as_object().unwrap();
        let mut differences = Vec::new();
        for (field, expected_value) in expected {
            compare_value(
                &format!("{case_id}.expected.{field}"),
                expected_value,
                actual.get(field),
                &mut differences,
            );
            compared_fields += 1;
        }
        assert!(
            differences.is_empty(),
            "{case_id}: {}",
            differences.join("\n")
        );
        executed_by_opcode.insert(case["input"]["operator"].as_str().unwrap().to_owned());
    }
    assert_eq!(
        executed_by_opcode,
        BTreeSet::from([
            "handle".to_owned(),
            "iterate_foreach".to_owned(),
            "iterate_while".to_owned(),
            "retry".to_owned(),
            "timeout".to_owned(),
            "scope".to_owned(),
        ])
    );
    println!(
        "Advanced Control Conformance: manifest=72; production=72; full_snapshot_repeat=72; compared_fields={compared_fields}; mismatch=0"
    );
}

fn assert_mutation_rejected(case_id: &str, path: &Path, mutator: impl FnOnce(&mut Value)) {
    let case = corpus_cases()
        .into_iter()
        .map(|(_, case)| case)
        .find(|case| case["case_id"] == case_id)
        .unwrap_or_else(|| panic!("missing corpus case {case_id}"));
    let actual = actual_for(&case);
    let mut mutated = case["expected"].clone();
    mutator(&mut mutated);
    let mut differences = Vec::new();
    compare_value(
        "mutated_expected",
        &mutated,
        Some(&actual),
        &mut differences,
    );
    assert!(
        !differences.is_empty(),
        "mutation at {} was not detected",
        path.display()
    );
}

#[test]
fn advanced_projection_mutations_are_detected() {
    assert_mutation_rejected(
        "computation-advanced-control/retry/schedule-04",
        Path::new("control_semantics.delays_micros_in_semantic_order"),
        |expected| expected["control_semantics"]["delays_micros_in_semantic_order"] = json!([1, 3]),
    );
    assert_mutation_rejected(
        "computation-advanced-control/retry/schedule-03",
        Path::new("control_semantics.physical_attempt_bounds.upper"),
        |expected| expected["control_semantics"]["physical_attempt_bounds"]["upper"] = json!(1),
    );
    assert_mutation_rejected(
        "computation-advanced-control/retry/schedule-03",
        Path::new("effects.entries[0].upper"),
        |expected| expected["effects"]["entries"][0]["upper"] = json!(1),
    );
    assert_mutation_rejected(
        "computation-advanced-control/iterate/foreach-03",
        Path::new("usage.coordinates.forward/api_calls"),
        |expected| expected["usage"]["coordinates"]["forward/api_calls"] = json!(1),
    );
    assert_mutation_rejected(
        "computation-advanced-control/timeout/boundary-04",
        Path::new("control_semantics.possible_outcomes"),
        |expected| expected["control_semantics"]["possible_outcomes"] = json!(["child_completed"]),
    );
    assert_mutation_rejected(
        "computation-advanced-control/handle/typed-clause-01",
        Path::new("failures"),
        |expected| expected["failures"] = json!([]),
    );
    assert_mutation_rejected(
        "computation-advanced-control/iterate/foreach-06",
        Path::new("control_decision_dependencies"),
        |expected| expected["control_decision_dependencies"] = json!([]),
    );
    assert_mutation_rejected(
        "computation-advanced-control/handle/typed-clause-03",
        Path::new("control_semantics.clauses[1].matcher"),
        |expected| {
            expected["control_semantics"]["clauses"][1]["matcher"] =
                json!({"failure_type":"TypeSchemaRef"});
        },
    );
    assert_mutation_rejected(
        "computation-advanced-control/iterate/foreach-carry-interface",
        Path::new("output_interface.ports[0].sources[0].value_slot"),
        |expected| {
            expected["output_interface"]["ports"][0]["sources"][0]["value_slot"] = json!(999);
        },
    );
    assert_mutation_rejected(
        "computation-advanced-control/retry/nested-foreach-amplification",
        Path::new("control_effect_families.families"),
        |expected| {
            expected["control_effect_families"][0]["families"]
                .as_array_mut()
                .unwrap()
                .remove(0);
        },
    );
}
