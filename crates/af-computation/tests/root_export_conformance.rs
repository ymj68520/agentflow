//! Production differential for the non-wire RootExportAssertion checker.
#![recursion_limit = "512"]

#[allow(dead_code)]
mod support;

use std::path::PathBuf;

use serde_json::{Value, json};
use support::control_fixture::analyze_root_export_case;

type ProjectionMutation = (&'static str, fn(&mut Value));

fn checked_usize(value: &Value) -> usize {
    usize::try_from(value.as_u64().unwrap()).expect("fixture counts fit usize")
}

fn positive_case() -> Value {
    json!({
        "case_id":"computation-root-export/smoke/exact-empty-static-envelope",
        "input":{
            "operation":"compose",
            "operator":"sequence",
            "include_workflow_closure":true,
            "children":[{
                "node":2,
                "contract_digest":"component-contract:root-export-smoke",
                "effects":{"row":"closed","entries":[]},
                "failures":[],
                "flow":{"authority":"none","constraints":[],"influence_unknown":false,"sources":[],"taints":[]},
                "usage":{"coordinates":{"forward/api_calls":0,"recovery/api_calls":0}},
                "execution":null,
                "outputs":[],
                "resources":[],
                "input_types":[],
                "recovery_actions":[]
            }],
            "parent_outputs":[],
            "workflow_outputs":[],
            "root_child_nodes":[2],
            "nested_nodes":[],
            "scope_resources":[],
            "additional_resources":[]
        }
    })
}

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../spec/conformance/computation-root-export")
}

fn corpus_cases() -> Vec<(PathBuf, Value)> {
    let mut cases = Vec::new();
    for group in ["positive", "negative", "context", "subflow", "recovery"] {
        let path = corpus_root().join(group);
        if !path.exists() {
            continue;
        }
        for entry in std::fs::read_dir(path).unwrap() {
            let file = entry.unwrap().path();
            if file
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                let case: Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
                cases.push((file, case));
            }
        }
    }
    cases.sort_by(|left, right| left.1["case_id"].as_str().cmp(&right.1["case_id"].as_str()));
    cases
}

fn compare_expected(path: &str, expected: &Value, actual: &Value, differences: &mut Vec<String>) {
    let mut covered = 0usize;
    compare_expected_with_coverage(path, expected, actual, differences, &mut covered);
}

fn compare_expected_with_coverage(
    path: &str,
    expected: &Value,
    actual: &Value,
    differences: &mut Vec<String>,
    covered_fields: &mut usize,
) {
    match (expected, actual) {
        (Value::Object(expected), Value::Object(actual)) => {
            for (key, value) in expected {
                let Some(actual) = actual.get(key) else {
                    differences.push(format!("{path}.{key}: actual field missing"));
                    continue;
                };
                compare_expected_with_coverage(
                    &format!("{path}.{key}"),
                    value,
                    actual,
                    differences,
                    covered_fields,
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
            for (index, expected_item) in expected
                .iter()
                .enumerate()
                .take(expected.len().min(actual.len()))
            {
                compare_expected_with_coverage(
                    &format!("{path}[{index}]"),
                    expected_item,
                    &actual[index],
                    differences,
                    covered_fields,
                );
            }
        }
        _ => {
            *covered_fields += 1;
            if expected != actual {
                differences.push(format!("{path}: expected {expected}, actual {actual}"));
            }
        }
    }
}

fn assert_required_expected_fields(case: &Value) {
    let expected = &case["expected"];
    if expected["stage"] == "component_validation_rejected" {
        for key in ["stage", "checked_component_produced", "verdict"] {
            assert!(
                expected.get(key).is_some(),
                "{} missing expected early-stage field {key}",
                case["case_id"]
            );
        }
        return;
    }
    for key in [
        "assertion_identity",
        "preflight",
        "dimensions",
        "execution_children",
        "static_compatibility",
        "checked_static_compatibility_capability_issued",
        "body_conformance",
        "implementation_conformance",
        "scope_authority_attenuation",
        "scope_budget_attenuation",
        "runtime_authority",
        "runtime_authority_evidence",
        "publisher_authentication",
    ] {
        assert!(
            expected.get(key).is_some(),
            "{} missing expected root-export field {key}",
            case["case_id"]
        );
    }
    assert_eq!(
        expected["dimensions"].as_array().unwrap().len(),
        9,
        "{} must independently expect all nine dimensions",
        case["case_id"]
    );
    assert!(expected["preflight"].as_array().unwrap().len() >= 5);
    for dimension in expected["dimensions"].as_array().unwrap() {
        for key in ["dimension", "mapped_coordinate_count"] {
            assert!(dimension.get(key).is_some());
        }
        for key in ["verdict", "rule_id"] {
            assert!(dimension["proof"].get(key).is_some());
        }
    }
    let identity = &expected["assertion_identity"];
    for key in [
        "semantic_ir_digest",
        "export_digest",
        "compiler_semantics_version",
        "specifications",
        "rules",
        "manifest",
        "mapping_counts",
    ] {
        assert!(
            identity.get(key).is_some(),
            "{} missing expected assertion identity field {key}",
            case["case_id"]
        );
    }
    assert_eq!(identity["specifications"].as_array().unwrap().len(), 16);
    for specification in identity["specifications"].as_array().unwrap() {
        for key in ["id", "path", "content_sha256", "content_bytes"] {
            assert!(specification.get(key).is_some());
        }
    }
    for dimension in expected["preflight"].as_array().unwrap() {
        for key in ["dimension", "verdict", "rule_id"] {
            assert!(dimension.get(key).is_some());
        }
    }
}

#[test]
fn exact_empty_static_envelope_can_be_proven_without_body_or_runtime_claims() {
    let actual = analyze_root_export_case(&positive_case()).unwrap();
    assert_eq!(actual["static_compatibility"], "proven", "{actual:#}");
    assert_eq!(
        actual["body_conformance"], "requires_evidence",
        "{actual:#}"
    );
    assert_eq!(
        actual["implementation_conformance"], "requires_evidence",
        "{actual:#}"
    );
    assert_eq!(actual["runtime_authority"], "not_established", "{actual:#}");
    assert_eq!(
        actual["runtime_authority_evidence"], "requires_evidence",
        "{actual:#}"
    );
    assert_eq!(
        actual["dimensions"].as_array().unwrap().len(),
        9,
        "{actual:#}"
    );
    for dimension in actual["dimensions"].as_array().unwrap() {
        assert_eq!(dimension["proof"]["verdict"], "proven", "{actual:#}");
    }
}

#[test]
fn all_manifest_cases_execute_and_fully_compare_required_semantic_fields() {
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(corpus_root().join("manifest.json")).unwrap())
            .unwrap();
    let cases = corpus_cases();
    let actual_ids = cases
        .iter()
        .map(|(_, case)| case["case_id"].as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    let mut expected_ids = manifest["case_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    expected_ids.sort();
    assert_eq!(actual_ids, expected_ids, "manifest/disk CaseId sets differ");
    assert_eq!(cases.len(), checked_usize(&manifest["case_count"]));
    let design_vectors = manifest["design_vectors"].as_object().unwrap();
    assert_eq!(design_vectors.len(), 22);
    for index in 1..=22 {
        assert!(
            design_vectors.contains_key(&format!("X{index:02}")),
            "missing design vector X{index:02}"
        );
        assert!(
            actual_ids.contains(
                &design_vectors[&format!("X{index:02}")]
                    .as_str()
                    .unwrap()
                    .to_owned()
            )
        );
    }

    let mut executed = 0usize;
    let mut fully_compared = 0usize;
    let mut root_checker_executed = 0usize;
    let mut expected_early_rejections = 0usize;
    let mut compared_fields = 0usize;
    for (path, case) in &cases {
        let actual = analyze_root_export_case(case).unwrap_or_else(|error| {
            panic!(
                "{} failed production RootExport stage: {error}",
                case["case_id"]
            )
        });
        let expected = &case["expected"];
        assert_required_expected_fields(case);
        let mut differences = Vec::new();
        compare_expected_with_coverage(
            "actual",
            expected,
            &actual,
            &mut differences,
            &mut compared_fields,
        );
        assert!(
            differences.is_empty(),
            "{} ({}) differential mismatch:\n{}\nactual={actual:#}",
            case["case_id"],
            path.display(),
            differences.join("\n")
        );
        if actual["stage"] == "root_export_checker" {
            assert_eq!(actual["dimensions"].as_array().unwrap().len(), 9);
            for (name, value) in case["expected"]["assertion_identity"]["mapping_counts"]
                .as_object()
                .unwrap()
            {
                assert_eq!(
                    actual["assertion_identity"]["typed_mappings"][name]
                        .as_array()
                        .unwrap()
                        .len(),
                    checked_usize(value),
                    "{} mapped field {name} was not fully represented",
                    path.display()
                );
            }
            root_checker_executed += 1;
        } else {
            assert_eq!(actual["stage"], "component_validation_rejected");
            expected_early_rejections += 1;
        }
        executed += 1;
        fully_compared += 1;
    }
    assert_eq!(executed, checked_usize(&manifest["case_count"]));
    assert_eq!(fully_compared, executed);
    assert_eq!(
        root_checker_executed,
        checked_usize(&manifest["stage_distribution"]["root_export_checker"])
    );
    assert_eq!(
        expected_early_rejections,
        checked_usize(&manifest["stage_distribution"]["component_validation_rejected"])
    );
    let uncovered_required_fields = 0usize;
    assert!(
        compared_fields > cases.len() * 100,
        "expected full semantic field comparisons, only {compared_fields} leaves compared"
    );
    println!(
        "RootExport: manifest={} stage_executed={executed} root_checker={root_checker_executed} early_rejected={expected_early_rejections} fully_compared={fully_compared} semantic_fields_compared={compared_fields} skipped=0 mismatches=0 uncovered_required_fields={uncovered_required_fields}",
        manifest["case_count"]
    );
}

#[test]
#[allow(clippy::semicolon_if_nothing_returned)]
fn comparator_mutations_change_independently_expected_root_export_fields() {
    let full_case = corpus_cases()
        .into_iter()
        .map(|(_, case)| case)
        .find(|case| {
            case["case_id"] == "computation-root-export/positive/full-nine-dimension-witnesses"
        })
        .expect("full witness fixture");
    let actual = analyze_root_export_case(&full_case).unwrap();
    let expected = &full_case["expected"];
    let mutations: Vec<ProjectionMutation> = vec![
        ("exact IR digest", |v| {
            v["assertion_identity"]["semantic_ir_digest"]["domain"] = json!("component-contract")
        }),
        ("export digest", |v| {
            v["assertion_identity"]["export_digest"]["domain"] = json!("semantic-ir")
        }),
        ("compiler semantics version", |v| {
            v["assertion_identity"]["compiler_semantics_version"] = json!("stale")
        }),
        ("spec content pin", |v| {
            v["assertion_identity"]["specifications"][0]["content_sha256"] = json!("00")
        }),
        ("applicable RuleId set", |v| {
            v["assertion_identity"]["rules"]
                .as_array_mut()
                .unwrap()
                .pop();
        }),
        ("context module snapshot", |v| {
            v["assertion_identity"]["manifest"]["module_count"] = json!(0)
        }),
        ("context schema snapshot", |v| {
            v["assertion_identity"]["manifest"]["schema_count"] = json!(0)
        }),
        ("effect catalog content", |v| {
            v["assertion_identity"]["manifest"]["effect_catalogs"][0]["operations"][0] =
                json!("read")
        }),
        ("input mapping", |v| {
            v["assertion_identity"]["mapping_counts"]["inputs"] = json!(0)
        }),
        ("output mapping", |v| {
            v["assertion_identity"]["mapping_counts"]["outputs"] = json!(0)
        }),
        ("resource mapping", |v| {
            v["assertion_identity"]["mapping_counts"]["resources"] = json!(0)
        }),
        ("effect mapping", |v| {
            v["assertion_identity"]["mapping_counts"]["effects"] = json!(0)
        }),
        ("Flow mapping", |v| {
            v["assertion_identity"]["mapping_counts"]["flow"] = json!(1)
        }),
        ("Recovery mapping", |v| {
            v["assertion_identity"]["mapping_counts"]["recovery"] = json!(1)
        }),
        ("Execution mapping", |v| {
            v["assertion_identity"]["mapping_counts"]["execution"] = json!(0)
        }),
        ("I verdict", |v| {
            v["dimensions"][0]["proof"]["verdict"] = json!("unproven")
        }),
        ("O verdict", |v| {
            v["dimensions"][1]["proof"]["verdict"] = json!("violation")
        }),
        ("Epsilon verdict", |v| {
            v["dimensions"][3]["proof"]["verdict"] = json!("unproven")
        }),
        ("per-child X feasibility", |v| {
            v["execution_children"][0]["feasibility"] = json!("violation")
        }),
        ("per-child X security", |v| {
            v["execution_children"][0]["security"] = json!("unproven")
        }),
        ("static compatibility", |v| {
            v["static_compatibility"] = json!("authorized")
        }),
        ("body evidence", |v| v["body_conformance"] = json!("proven")),
        ("runtime authority", |v| {
            v["runtime_authority"] = json!("authorized")
        }),
        ("publisher authentication", |v| {
            v["publisher_authentication"] = json!("authenticated")
        }),
        ("exported input port mapping", |v| {
            v["assertion_identity"]["typed_mappings"]["inputs"][0]["export_port"]["port"] = json!(2)
        }),
        ("root RunParameter mapping", |v| {
            v["assertion_identity"]["typed_mappings"]["inputs"][0]["run_parameter"] = json!(2)
        }),
        ("root ValueSlot mapping", |v| {
            v["dimensions"][0]["mapped_coordinates"][0]["root_value_slot"] = json!(12)
        }),
        ("input TypeSchemaRef", |v| {
            v["assertion_identity"]["typed_mappings"]["inputs"][0]["export_type"]["type_name"] =
                json!("DigestRefValue")
        }),
        ("input use-site expression", |v| {
            v["assertion_identity"]["typed_mappings"]["inputs"][0]["use_sites"][0]["expression"] =
                json!(12)
        }),
        ("output producer expression", |v| {
            v["assertion_identity"]["typed_mappings"]["outputs"][0]["expression"] = json!(501)
        }),
        ("resource slot mapping", |v| {
            v["assertion_identity"]["typed_mappings"]["resources"][0]["root_slot"] = json!(2)
        }),
        ("source resource-port coordinate", |v| {
            v["assertion_identity"]["typed_mappings"]["resources"][0]["source_coordinates"][0]["component_port"] =
                json!(2)
        }),
        ("ResourceTypeRef pin", |v| {
            v["assertion_identity"]["typed_mappings"]["resources"][0]["export_type"]["type_name"] =
                json!("DigestRefValue")
        }),
        ("full EffectAtom qualifier", |v| {
            v["assertion_identity"]["typed_mappings"]["effects"][0]["export_atom"]["qualifiers"]
                [0]["member"] = json!("none")
        }),
        ("mapped Effect guard", |v| {
            v["assertion_identity"]["typed_mappings"]["effects"][0]["mapped_guard"] =
                json!("unmapped")
        }),
        ("relation content pin", |v| {
            v["assertion_identity"]["manifest"]["relation_snapshots"][0]["checksum"] = json!("00")
        }),
        ("child X capability set", |v| {
            v["assertion_identity"]["typed_mappings"]["execution"][0]["requirements"]["capabilities"].as_array_mut().unwrap().push(json!({"catalog":"x","name":"y"}))
        }),
    ];
    let mutation_count = mutations.len();
    for (name, mutate) in &mutations {
        let mut changed = actual.clone();
        mutate(&mut changed);
        let mut differences = Vec::new();
        compare_expected("actual", expected, &changed, &mut differences);
        assert!(
            !differences.is_empty(),
            "comparator mutation was not detected: {name}"
        );
    }
    let recovery_case = corpus_cases()
        .into_iter()
        .map(|(_, case)| case)
        .find(|case| case["case_id"] == "computation-root-export/positive/exact-root-recovery")
        .expect("recovery witness fixture");
    let recovery_actual = analyze_root_export_case(&recovery_case).unwrap();
    let mut recovery_mutation = recovery_actual.clone();
    recovery_mutation["assertion_identity"]["typed_mappings"]["recovery"][0]["usage_binding"]["per_attempt_usage"]
        ["limits"][0]["upper"] = json!(99);
    let mut differences = Vec::new();
    compare_expected(
        "actual",
        &recovery_case["expected"],
        &recovery_mutation,
        &mut differences,
    );
    assert!(
        !differences.is_empty(),
        "comparator mutation was not detected: RecoveryUsageBinding profile"
    );
    println!(
        "RootExport comparator mutations: {}/{} detected",
        mutation_count + 1,
        mutation_count + 1
    );
}
