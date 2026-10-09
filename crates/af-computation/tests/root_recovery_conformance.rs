//! Independent Python oracle versus the checked production component and
//! workflow-root Recovery projections.
#![recursion_limit = "512"]

#[allow(dead_code)]
mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde_json::{Value, json};
use support::control_fixture::{analyze_compose_case, validate_contract_case};

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../spec/conformance/computation-root-recovery")
}

fn corpus_cases() -> Vec<(PathBuf, Value)> {
    let mut cases = Vec::new();
    for group in ["guard", "root", "audit"] {
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
    match case["stage"].as_str().unwrap() {
        "component_validation" => validate_contract_case(case).unwrap_or_else(|error| {
            panic!(
                "{} did not reach ComponentContract validation: {error}",
                case["case_id"]
            )
        }),
        "root_recovery" => {
            let projection = analyze_compose_case(case).unwrap_or_else(|error| {
                panic!(
                    "{} did not reach workflow-root recovery: {error}",
                    case["case_id"]
                )
            });
            let recovery = &projection["workflow_closure"]["root_recovery"];
            if let Some(actions) = recovery["actions"]["actions"].as_array() {
                let root_digest = actions
                    .first()
                    .map(|action| &action["root_action_bound"]["root_semantic_ir_digest"]);
                for action in actions {
                    assert_eq!(
                        action["source"]["caller_semantic_ir_digest"],
                        action["root_action_bound"]["root_semantic_ir_digest"],
                        "{}: action source must remain in the exact caller SemanticIrDigest namespace",
                        case["case_id"]
                    );
                    if let Some(root_digest) = root_digest {
                        assert_eq!(
                            action["root_action_bound"]["root_semantic_ir_digest"],
                            *root_digest
                        );
                    }
                }
            }
            json!({
                "stage":projection["_execution_stage"],
                "workflow_closure":{"root_recovery":recovery},
            })
        }
        "ir_rejection" => {
            let error = analyze_compose_case(case).err();
            let stage = if error
                .as_deref()
                .is_some_and(|message| message.starts_with("IR validation:"))
            {
                "ir_validation_rejected"
            } else {
                "unexpected_stage_rejection"
            };
            json!({"stage":stage})
        }
        other => panic!("unknown required production stage {other}"),
    }
}

fn assert_required_fields(case: &Value) {
    match case["stage"].as_str().unwrap() {
        "component_validation" => {
            for key in [
                "stage",
                "verdict",
                "checked_component_produced",
                "action_guard_compatibility",
            ] {
                assert!(
                    case["expected"].get(key).is_some(),
                    "{} missing expected component field {key}",
                    case["case_id"]
                );
            }
            for relation in case["expected"]["action_guard_compatibility"]
                .as_array()
                .unwrap()
            {
                assert!(relation.get("verdict").is_some());
                assert!(relation.get("basis").is_some());
            }
        }
        "root_recovery" => {
            let root = &case["expected"]["workflow_closure"]["root_recovery"];
            for key in [
                "actions",
                "action_effects",
                "epsilon_coverage_requirement",
                "recovery_usage",
                "source_closure",
                "overall_verdict",
                "proof_dimensions",
            ] {
                assert!(
                    root.get(key).is_some(),
                    "{} missing expected root field {key}",
                    case["case_id"]
                );
            }
            let actions = root["actions"]["actions"].as_array().unwrap();
            for action in actions {
                for key in [
                    "source",
                    "owner",
                    "local_action_bound",
                    "root_action_bound",
                    "recovery_usage_binding",
                    "root_recovery_usage",
                    "guard_compatibility",
                    "eligibility_evidence",
                    "control_path",
                ] {
                    assert!(
                        action.get(key).is_some(),
                        "{} action omitted required field {key}",
                        case["case_id"]
                    );
                }
            }
        }
        "ir_rejection" => assert_eq!(case["expected"]["stage"], "ir_validation_rejected"),
        _ => panic!("invalid stage"),
    }
}

#[test]
fn every_manifest_case_executes_at_its_declared_production_stage_and_is_fully_compared() {
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
    assert_eq!(manifest["case_count"].as_u64(), Some(cases.len() as u64));
    assert_eq!(cases.len(), 44);
    assert_eq!(
        discovered, declared,
        "manifest must exactly cover every executable case"
    );
    assert_eq!(manifest["cases"].as_array().unwrap().len(), cases.len());

    let case_plan = manifest["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| (item["case_id"].as_str().unwrap().to_owned(), item))
        .collect::<BTreeMap<_, _>>();
    let mut executed = BTreeSet::new();
    let mut fully_compared = BTreeSet::new();
    let skipped = Vec::<String>::new();
    let mut differences = Vec::new();
    for (_, case) in &cases {
        let case_id = case["case_id"].as_str().unwrap();
        let plan = case_plan.get(case_id).unwrap();
        assert_eq!(
            plan["expected_stage"], case["stage"],
            "{case_id}: stage classification drift"
        );
        assert!(plan["production_api"].is_string());
        assert_eq!(plan["oracle"], "independent-python");
        assert!(
            plan["expected_fields"]
                .as_array()
                .is_some_and(|fields| !fields.is_empty())
        );
        for stage in [
            "ComponentValidation",
            "IRValidation",
            "LeafResolution",
            "ControlDerivation",
            "RootRecoveryValidation",
            "LowerLevelEffectAlgebra",
        ] {
            assert!(
                plan["stage_reachability"][stage].is_string(),
                "{case_id} missing reachability classification for {stage}"
            );
        }
        assert!(plan["d3_export_assertion"].is_string());
        assert_required_fields(case);
        let actual = production(case);
        executed.insert(case_id.to_owned());
        compare_value(
            &format!("{case_id}.expected"),
            &case["expected"],
            Some(&actual),
            &mut differences,
        );
        fully_compared.insert(case_id.to_owned());
    }
    assert!(skipped.is_empty(), "skipped CaseIds: {skipped:?}");
    assert_eq!(executed, declared, "stage-executed manifest mismatch");
    assert_eq!(fully_compared, declared, "fully-compared manifest mismatch");
    assert!(
        differences.is_empty(),
        "{} root-recovery oracle differences:\n{}",
        differences.len(),
        differences.join("\n")
    );
}

fn expected(case_id: &str) -> Value {
    let path = corpus_cases()
        .into_iter()
        .find(|(_, case)| case["case_id"] == case_id)
        .unwrap()
        .0;
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

fn mutate_path(value: &mut Value, path: &[&str], replacement: Value) {
    let mut cursor = value;
    for part in &path[..path.len() - 1] {
        cursor = match cursor {
            Value::Array(items) => &mut items[part.parse::<usize>().unwrap()],
            Value::Object(items) => items.get_mut(*part).unwrap(),
            other => panic!("cannot follow mutation path through {other}"),
        };
    }
    let final_part = path[path.len() - 1];
    match cursor {
        Value::Array(items) => items[final_part.parse::<usize>().unwrap()] = replacement,
        Value::Object(items) => *items.get_mut(final_part).unwrap() = replacement,
        other => panic!("cannot mutate value through {other}"),
    }
}

fn actual_for(case_id: &str) -> Value {
    let case = expected(case_id);
    production(&case)
}

fn root_projection(case_id: &str) -> Value {
    actual_for(&format!("computation-root-recovery/root/{case_id}"))["workflow_closure"]["root_recovery"].clone()
}

fn upper_at(root: &Value, action_index: usize, coordinate: &str) -> Value {
    root["actions"]["actions"][action_index][coordinate]["upper"]["upper"].clone()
}

#[test]
fn accepted_arithmetic_and_nested_scope_ownership_have_exact_production_values() {
    let cases = [
        ("K01", 6, 6),
        ("K02", 4, 12),
        ("K03", 12, 12),
        ("K04", 4, 16),
        ("K05", 8, 96),
    ];
    for (case_id, expected_local, expected_root) in cases {
        let root = root_projection(case_id);
        assert_eq!(
            upper_at(&root, 0, "local_action_bound"),
            expected_local,
            "{case_id} local"
        );
        assert_eq!(
            upper_at(&root, 0, "root_action_bound"),
            expected_root,
            "{case_id} root"
        );
    }

    let same_atom = root_projection("K06");
    assert_eq!(upper_at(&same_atom, 0, "root_action_bound"), 6);
    assert_eq!(
        same_atom["epsilon_coverage_requirement"]["value"]["entries"][0]["upper"], 7,
        "Forward 1 and K action 6 remain two occurrences under one atom"
    );
    assert_eq!(same_atom["proof_dimensions"][3]["verdict"], "proven");

    for (case_id, total) in [
        ("K07", 10),
        ("K08", 5),
        ("K09", 8),
        ("K10", 7),
        ("K11", 7),
        ("K12", 7),
        ("K22", 7),
    ] {
        let root = root_projection(case_id);
        let first = &root["action_effects"]["value"]["entries"][0]["upper"];
        assert_eq!(
            first, total,
            "{case_id} operator-specific action aggregation"
        );
    }

    for case_id in ["K13", "K14"] {
        let root = root_projection(case_id);
        assert_eq!(root["actions"]["actions"].as_array().unwrap().len(), 1);
        assert_eq!(root["actions"]["actions"][0]["owner"]["node"], 10);
        assert_eq!(upper_at(&root, 0, "local_action_bound"), 4);
        assert_eq!(upper_at(&root, 0, "root_action_bound"), 4);
        let case = expected(&format!("computation-root-recovery/root/{case_id}"));
        let projection = analyze_compose_case(&case).unwrap();
        let scopes = projection["compensation_scopes_by_node"]
            .as_array()
            .unwrap();
        assert_eq!(scopes.len(), 2);
        assert_eq!(
            scopes[0]["summary"]["local_recovery_actions"]["actions"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        assert_eq!(
            scopes[1]["summary"]["local_recovery_actions"]["actions"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
    }
    let unscoped = root_projection("K15");
    assert_eq!(
        unscoped["actions"]["actions"][0]["owner"]["kind"],
        "root_recovery"
    );
    assert!(unscoped["actions"]["actions"][0]["local_action_bound"].is_null());
    assert_eq!(upper_at(&unscoped, 0, "root_action_bound"), 4);

    let unbounded = root_projection("K20");
    assert_eq!(upper_at(&unbounded, 0, "root_action_bound"), "unbounded");
    assert_eq!(unbounded["proof_dimensions"][4]["verdict"], "unproven");
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(corpus_root().join("manifest.json")).unwrap())
            .unwrap();
    assert!(manifest["d3_export_assertions_deferred"].as_array().unwrap().iter().any(|item| item == "K20 export epsilon ceiling; only internal root K/E/U facts execute in this phase"));
}

#[test]
fn all_eighteen_comparator_mutations_are_detected_on_isolated_actual_copies() {
    let mutations: Vec<(&str, &str, Vec<&str>, Value)> = vec![
        (
            "G02",
            "reverse guard containment",
            vec!["action_guard_compatibility", "0", "verdict"],
            json!("violation"),
        ),
        (
            "G16",
            "false means automatic Violation",
            vec!["action_guard_compatibility", "0", "verdict"],
            json!("violation"),
        ),
        (
            "G03",
            "drop typed countermodel",
            vec!["action_guard_compatibility", "0", "countermodel"],
            Value::Null,
        ),
        (
            "G04",
            "change action guard port",
            vec![
                "action_guard_compatibility",
                "0",
                "declared_action_guard",
                "input_equals",
                "port",
            ],
            json!(2),
        ),
        (
            "K02",
            "drop control path",
            vec![
                "workflow_closure",
                "root_recovery",
                "actions",
                "actions",
                "0",
                "control_path",
            ],
            json!([]),
        ),
        (
            "K02",
            "apply Retry twice",
            vec![
                "workflow_closure",
                "root_recovery",
                "actions",
                "actions",
                "0",
                "root_action_bound",
                "upper",
                "upper",
            ],
            json!(36),
        ),
        (
            "K04",
            "omit outer Iterate",
            vec![
                "workflow_closure",
                "root_recovery",
                "actions",
                "actions",
                "0",
                "root_action_bound",
                "upper",
                "upper",
            ],
            json!(4),
        ),
        (
            "K04",
            "use max_inflight as total",
            vec![
                "workflow_closure",
                "root_recovery",
                "actions",
                "actions",
                "0",
                "root_action_bound",
                "upper",
                "upper",
            ],
            json!(8),
        ),
        (
            "K02",
            "confuse LocalActionBound with RootActionBound",
            vec![
                "workflow_closure",
                "root_recovery",
                "actions",
                "actions",
                "0",
                "root_action_bound",
                "upper",
                "upper",
            ],
            json!(4),
        ),
        (
            "K11",
            "delete RACE loser",
            vec!["workflow_closure", "root_recovery", "actions", "actions"],
            json!([]),
        ),
        (
            "K09",
            "sum exclusive SELECT branches",
            vec![
                "workflow_closure",
                "root_recovery",
                "action_effects",
                "value",
                "entries",
                "0",
                "upper",
            ],
            json!(12),
        ),
        (
            "K06",
            "collapse Forward and same-atom Action",
            vec![
                "workflow_closure",
                "root_recovery",
                "epsilon_coverage_requirement",
                "value",
                "entries",
                "0",
                "upper",
            ],
            json!(6),
        ),
        (
            "K07",
            "drop one RecoveryUsageBinding source profile",
            vec!["workflow_closure", "root_recovery", "actions", "actions"],
            json!([]),
        ),
        (
            "K07",
            "compare source node only",
            vec![
                "workflow_closure",
                "root_recovery",
                "actions",
                "actions",
                "1",
                "source",
                "source_node",
            ],
            json!(2),
        ),
        (
            "K02",
            "add K Usage to Root U a second time",
            vec![
                "workflow_closure",
                "root_recovery",
                "recovery_usage",
                "value",
                "coordinates",
                "recovery/api_calls",
            ],
            json!(24),
        ),
        (
            "K20",
            "replace unknown Usage with zero",
            vec![
                "workflow_closure",
                "root_recovery",
                "recovery_usage",
                "value",
                "coordinates",
                "recovery/api_calls",
            ],
            json!(0),
        ),
        (
            "K25-overflow",
            "convert overflow to Proven",
            vec!["verdict"],
            json!("proven"),
        ),
        (
            "K01",
            "remove RequiresEvidence obligation",
            vec![
                "workflow_closure",
                "root_recovery",
                "actions",
                "actions",
                "0",
                "eligibility_evidence",
            ],
            json!("proven"),
        ),
    ];
    let mut detected = BTreeSet::new();
    for (suffix, label, path, replacement) in mutations {
        let case_id = if suffix.starts_with('G') {
            format!("computation-root-recovery/guard/{suffix}")
        } else {
            format!("computation-root-recovery/root/{suffix}")
        };
        let case = expected(&case_id);
        let mut actual = actual_for(&case_id);
        mutate_path(&mut actual, &path, replacement);
        let mut differences = Vec::new();
        compare_value(
            &format!("{case_id}.expected"),
            &case["expected"],
            Some(&actual),
            &mut differences,
        );
        assert!(
            !differences.is_empty(),
            "comparator failed to detect mutation: {label}"
        );
        detected.insert(label);
    }
    assert_eq!(detected.len(), 18);
}
