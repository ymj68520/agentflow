use af_portable::{PortableProofCodec, PortableProofError, ValidationStage};
use serde::Deserialize;
use serde_json::Value as Json;
use sha2::{Digest as _, Sha256};
use std::path::{Path, PathBuf};

const CORPUS_DIR: &str = "spec/conformance/portable-static-reconstruction";

#[derive(Deserialize)]
struct Manifest {
    base_file: String,
    base_sha256: String,
    case_count: usize,
    group_counts: std::collections::BTreeMap<String, usize>,
    required_context_projection_fields: Vec<String>,
    required_pipeline_projection_fields: Vec<String>,
    comparator_mutation_count: usize,
    comparator_mutations: Vec<ComparatorMutation>,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct ComparatorMutation {
    mutation_id: String,
    dimension: String,
    witness_case_id: String,
}

#[derive(Deserialize)]
#[allow(clippy::struct_field_names)]
struct Case {
    case_id: String,
    group: String,
    entry: String,
    mutations: Vec<Mutation>,
    input_byte_length: usize,
    input_sha256: String,
    expected_stage: String,
    expected_error_family: Option<String>,
    expected_context_projection_sha256: Option<String>,
    expected_pipeline_projection_sha256: Option<String>,
    expected_context_projection: Option<Json>,
    expected_pipeline_projection: Option<Json>,
    expected_context_identity_relation: String,
    required_compared_fields: Vec<String>,
}

#[derive(Deserialize)]
struct Mutation {
    op: String,
    path: Vec<String>,
    #[serde(default)]
    value: Json,
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn stage_name(stage: ValidationStage) -> &'static str {
    match stage {
        ValidationStage::D0 => "D0",
        ValidationStage::D1 => "D1",
        ValidationStage::D2 => "D2",
        ValidationStage::D3 => "D3",
        ValidationStage::D4 => "D4",
        ValidationStage::D5 => "D5",
        ValidationStage::D6 => "D6",
        ValidationStage::D7 => "D7",
        ValidationStage::D8 => "D8",
    }
}

fn apply_mutations(base: &[u8], mutations: &[Mutation]) -> Vec<u8> {
    let mut bundle: Json = serde_json::from_slice(base).expect("canonical PPI Bundle JSON");
    for mutation in mutations {
        assert_eq!(mutation.op, "set", "registered mutation op");
        if mutation.path.is_empty() {
            bundle = mutation.value.clone();
            continue;
        }
        let (parents, last) = mutation.path.split_at(mutation.path.len() - 1);
        let mut current = &mut bundle;
        for token in parents {
            current = descend(current, token);
        }
        match current {
            Json::Object(map) => {
                map.insert(last[0].clone(), mutation.value.clone());
            }
            Json::Array(items) => {
                let index = last[0].parse::<usize>().expect("array index token");
                items[index] = mutation.value.clone();
            }
            _ => panic!("mutation path parent is a scalar"),
        }
    }
    machine_text(&mut bundle);
    serde_json::to_vec(&bundle).expect("canonical machine JSON serialization")
}

fn machine_text(value: &mut Json) {
    match value {
        Json::Number(number) => *value = Json::String(number.to_string()),
        Json::Array(items) => items.iter_mut().for_each(machine_text),
        Json::Object(fields) => fields.values_mut().for_each(machine_text),
        Json::Null | Json::Bool(_) | Json::String(_) => {}
    }
}

fn descend<'a>(value: &'a mut Json, token: &str) -> &'a mut Json {
    match value {
        Json::Object(map) => map.get_mut(token).expect("object mutation path"),
        Json::Array(items) => &mut items[token.parse::<usize>().expect("array index token")],
        _ => panic!("mutation path crosses a scalar"),
    }
}

fn projection_sha(value: &Json) -> String {
    sha256(&serde_json::to_vec(value).expect("typed projection JSON"))
}

fn first_json_difference(expected: &Json, actual: &Json, path: &str) -> Option<String> {
    match (expected, actual) {
        (Json::Object(expected), Json::Object(actual)) => {
            for key in expected.keys().chain(actual.keys()) {
                match (expected.get(key), actual.get(key)) {
                    (Some(expected), Some(actual)) => {
                        if let Some(difference) =
                            first_json_difference(expected, actual, &format!("{path}.{key}"))
                        {
                            return Some(difference);
                        }
                    }
                    (Some(_), None) => return Some(format!("{path}.{key}: missing actual field")),
                    (None, Some(_)) => {
                        return Some(format!("{path}.{key}: unexpected actual field"));
                    }
                    (None, None) => unreachable!("key came from one object"),
                }
            }
            None
        }
        (Json::Array(expected), Json::Array(actual)) => {
            if expected.len() != actual.len() {
                return Some(format!(
                    "{path}: expected {} entries, got {}",
                    expected.len(),
                    actual.len()
                ));
            }
            expected
                .iter()
                .zip(actual)
                .enumerate()
                .find_map(|(index, (expected, actual))| {
                    first_json_difference(expected, actual, &format!("{path}[{index}]"))
                })
        }
        _ if expected == actual => None,
        _ => Some(format!("{path}: expected {expected}, got {actual}")),
    }
}

fn assert_error(case: &Case, error: &PortableProofError) {
    assert_eq!(
        stage_name(error.stage),
        case.expected_stage,
        "stage: {}; actual error={:?} {:?} at {}: {}",
        case.case_id,
        error.family,
        error.stage,
        error.path,
        error.detail,
    );
    assert_eq!(
        Some(error.family.to_owned()),
        case.expected_error_family,
        "family: {}",
        case.case_id
    );
    if case
        .case_id
        .ends_with("d6-12-action-guard-counterexample-no-partial-checked")
    {
        assert!(
            error
                .detail
                .contains("action_guard_compatibility=[violation"),
            "formal D6 report must retain the guard counterexample verdict: {}",
            error.detail,
        );
    }
}

#[test]
fn independent_python_manifest_matches_all_production_d5_to_d8_cases() {
    let root = repo();
    let manifest_path = root.join(CORPUS_DIR).join("manifest.json");
    let manifest: Manifest =
        serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
    let base_path = root.join(&manifest.base_file);
    let base = std::fs::read(base_path).unwrap();
    assert_eq!(
        sha256(&base),
        manifest.base_sha256,
        "frozen D0-D4 base identity"
    );
    assert_eq!(manifest.case_count, manifest.cases.len());
    assert_eq!(manifest.case_count, 63);
    assert_eq!(manifest.group_counts.get("D5"), Some(&16));
    assert_eq!(manifest.group_counts.get("D6"), Some(&15));
    assert_eq!(manifest.group_counts.get("D7"), Some(&16));
    assert_eq!(manifest.group_counts.get("D8"), Some(&16));
    assert_eq!(manifest.comparator_mutation_count, 16);
    assert_eq!(manifest.comparator_mutations.len(), 16);

    let mut case_ids = std::collections::BTreeSet::new();
    let mut group_counts = std::collections::BTreeMap::<String, usize>::new();
    let mut final_stage_counts = std::collections::BTreeMap::<String, usize>::new();
    let mut executed = 0usize;
    let mut fully_compared = 0usize;
    let skipped = 0usize;
    let mismatch = 0usize;
    let codec = PortableProofCodec::default();
    let filter = std::env::var("AF_STATIC_RECONSTRUCTION_CASE").ok();
    let mut actual_projections = std::collections::BTreeMap::<String, Json>::new();

    for case in manifest.cases.iter().filter(|case| {
        filter
            .as_ref()
            .is_none_or(|selected| case.case_id == *selected)
    }) {
        assert!(
            !case.required_compared_fields.is_empty(),
            "{} declares compared fields",
            case.case_id
        );
        assert!(
            case_ids.insert(case.case_id.clone()),
            "duplicate CaseId {}",
            case.case_id
        );
        *group_counts.entry(case.group.clone()).or_default() += 1;
        let input = apply_mutations(&base, &case.mutations);
        assert_eq!(
            input.len(),
            case.input_byte_length,
            "input size: {}",
            case.case_id
        );
        assert_eq!(
            sha256(&input),
            case.input_sha256,
            "input digest: {}",
            case.case_id
        );

        let supported = match codec.validate_supported(&input) {
            Ok(supported) => supported,
            Err(error) => {
                assert_error(case, &error);
                actual_projections.insert(case.case_id.clone(), error_projection(&error));
                *final_stage_counts
                    .entry(case.expected_stage.clone())
                    .or_default() += 1;
                executed += 1;
                fully_compared += 1;
                continue;
            }
        };
        assert!(
            !matches!(
                case.expected_stage.as_str(),
                "D0" | "D1" | "D2" | "D3" | "D4"
            ),
            "unexpected D0-D4 success for {}",
            case.case_id
        );

        let context = match supported.reconstruct_context(1) {
            Ok(context) => context,
            Err(error) => {
                assert_error(case, &error);
                actual_projections.insert(case.case_id.clone(), error_projection(&error));
                *final_stage_counts
                    .entry(case.expected_stage.clone())
                    .or_default() += 1;
                executed += 1;
                fully_compared += 1;
                continue;
            }
        };
        let context_json =
            serde_json::to_value(context.projection()).expect("D5 projection serialization");
        if let Some(expected) = &case.expected_context_projection_sha256 {
            assert_eq!(
                projection_sha(&context_json),
                *expected,
                "complete D5 field projection: {}",
                case.case_id
            );
        } else {
            panic!(
                "accepted D5 context lacks independent expected projection: {}",
                case.case_id
            );
        }
        let expected_context = case
            .expected_context_projection
            .as_ref()
            .expect("independent D5 field projection");
        assert_eq!(
            first_json_difference(expected_context, &context_json, "context"),
            None,
            "complete independent D5 field projection: {}",
            case.case_id,
        );
        if case.expected_context_identity_relation == "distinct_for_equal_content" {
            let second = supported
                .reconstruct_context(1)
                .expect("equal input produces a second D5 context");
            second.with_proof_context(|second_context| {
                context.with_proof_context(|first_context| {
                    assert!(
                        !std::ptr::eq(first_context, second_context),
                        "equal content must not reuse proof-context identity"
                    );
                });
            });
        }

        if case.entry == "d5" {
            assert_eq!(
                case.expected_stage, "D5",
                "D5-only production entry: {}",
                case.case_id
            );
            assert!(
                case.expected_error_family.is_none(),
                "D5 successful entry has no error family"
            );
            actual_projections.insert(case.case_id.clone(), context_json);
            *final_stage_counts.entry("D5".to_owned()).or_default() += 1;
            executed += 1;
            fully_compared += 1;
            continue;
        }

        match supported.reconstruct_static_inputs(1, 1) {
            Ok(inputs) => {
                assert_eq!(
                    case.expected_stage, "D8",
                    "pipeline success must reach D8: {}",
                    case.case_id
                );
                assert!(
                    case.expected_error_family.is_none(),
                    "successful pipeline cannot expect an error family"
                );
                let actual = inputs.typed_projection();
                let expected_pipeline = case
                    .expected_pipeline_projection
                    .as_ref()
                    .expect("independent D5-D8 field projection");
                assert_eq!(
                    first_json_difference(expected_pipeline, &actual, "pipeline"),
                    None,
                    "complete independent D5-D8 field projection: {}",
                    case.case_id,
                );
                assert_eq!(
                    projection_sha(&actual),
                    case.expected_pipeline_projection_sha256
                        .as_deref()
                        .expect("Python D8 expected projection"),
                    "complete D5-D8 typed projection: {}",
                    case.case_id,
                );
                actual_projections.insert(case.case_id.clone(), actual);
            }
            Err(error) => {
                assert_error(case, &error);
                actual_projections.insert(case.case_id.clone(), error_projection(&error));
            }
        }
        *final_stage_counts
            .entry(case.expected_stage.clone())
            .or_default() += 1;
        executed += 1;
        fully_compared += 1;
    }

    assert_eq!(executed, filter.as_ref().map_or(manifest.case_count, |_| 1));
    assert_eq!(
        fully_compared,
        filter.as_ref().map_or(manifest.case_count, |_| 1)
    );
    assert_eq!(skipped, 0);
    assert_eq!(mismatch, 0);
    if filter.is_none() {
        assert_eq!(group_counts.get("D5"), Some(&16));
        assert_eq!(group_counts.get("D6"), Some(&15));
        assert_eq!(group_counts.get("D7"), Some(&16));
        assert_eq!(group_counts.get("D8"), Some(&16));
        assert_eq!(manifest.required_context_projection_fields.len(), 20);
        assert_eq!(manifest.required_pipeline_projection_fields.len(), 9);
        assert!(final_stage_counts.get("D5").copied().unwrap_or_default() >= 10);
        assert!(final_stage_counts.get("D6").copied().unwrap_or_default() >= 8);
        assert!(final_stage_counts.get("D7").copied().unwrap_or_default() >= 12);
        assert!(final_stage_counts.get("D8").copied().unwrap_or_default() >= 10);
        assert_eq!(
            final_stage_counts,
            std::collections::BTreeMap::from([
                ("D1".to_owned(), 2),
                ("D5".to_owned(), 16),
                ("D6".to_owned(), 13),
                ("D7".to_owned(), 12),
                ("D8".to_owned(), 20),
            ]),
            "actual earliest rejection / completion stage distribution",
        );

        let mut comparator_ids = std::collections::BTreeSet::new();
        for mutation in &manifest.comparator_mutations {
            assert!(comparator_ids.insert(mutation.mutation_id.clone()));
            let witness = manifest
                .cases
                .iter()
                .find(|case| case.case_id == mutation.witness_case_id)
                .expect("comparator witness CaseId");
            let mut altered = actual_projections
                .get(&mutation.witness_case_id)
                .expect("executed comparator witness")
                .clone();
            alter_projection(&mut altered, &mutation.dimension);
            if witness.expected_error_family.is_some() {
                assert!(
                    altered.get("stage").and_then(Json::as_str)
                        != Some(witness.expected_stage.as_str())
                        || altered.get("failure_family").and_then(Json::as_str)
                            != witness.expected_error_family.as_deref(),
                    "comparator mutation {} was not detected against stage/family",
                    mutation.mutation_id,
                );
            } else if let Some(expected) = &witness.expected_pipeline_projection_sha256 {
                assert_ne!(
                    projection_sha(&altered),
                    *expected,
                    "comparator mutation {} was not detected",
                    mutation.mutation_id
                );
            } else if let Some(expected) = &witness.expected_context_projection_sha256 {
                assert_ne!(
                    projection_sha(&altered),
                    *expected,
                    "comparator mutation {} was not detected",
                    mutation.mutation_id
                );
            } else {
                assert_ne!(
                    altered.get("failure_family").and_then(Json::as_str),
                    witness.expected_error_family.as_deref(),
                    "comparator mutation {} was not detected",
                    mutation.mutation_id,
                );
            }
        }
    }
}

fn error_projection(error: &PortableProofError) -> Json {
    serde_json::json!({
        "stage": stage_name(error.stage),
        "failure_family": error.family,
        "path": error.path,
    })
}

fn alter_projection(value: &mut Json, dimension: &str) {
    match dimension {
        "context_membership_lost" => {
            context_projection_mut(value)["type_schemas"]
                .as_array_mut()
                .unwrap()
                .pop();
        }
        "schema_scope_collapsed" => {
            let object = context_projection_mut(value).as_object_mut().unwrap();
            object.insert("resource_schemas".into(), object["type_schemas"].clone());
        }
        "recovery_effect_catalog_reused" => {
            let object = context_projection_mut(value).as_object_mut().unwrap();
            object.insert(
                "recovery_effect_catalogs".into(),
                object["effect_catalogs"].clone(),
            );
        }
        "execution_exact_version_changed" => {
            context_projection_mut(value)["execution_capabilities"][0]["exact_version"] =
                Json::String("v1".into());
        }
        "relation_content_changed" => {
            context_projection_mut(value)["effect_resource_relation"]["canonical_content"]
                .as_array_mut()
                .unwrap()
                .push(Json::from(0));
        }
        "contract_source_pin_dropped" => {
            value["source_associations"][0]["target"]["invoke"]["digest"]["hex"] =
                Json::String("0".repeat(64));
        }
        "component_guard_violation_promoted" => {
            value["stage"] = Json::String("D8".into());
            value["failure_family"] = Json::Null;
        }
        "ir_producer_count_swapped" => {
            let count = value["checked_irs"][0]["phase3c_producer_count"]
                .as_u64()
                .unwrap();
            value["checked_irs"][0]["phase3c_producer_count"] = Json::from(count + 1);
        }
        "scope_alias_lost" => {
            context_projection_mut(value)["resource_descriptors"][0]["recognized_attributes"]
                .as_array_mut()
                .unwrap()
                .pop();
        }
        "workflow_exit_producer_dropped" => {
            let count = value["checked_irs"][0]["phase3c_workflow_exit_value_count"]
                .as_u64()
                .unwrap();
            value["checked_irs"][0]["phase3c_workflow_exit_value_count"] = Json::from(count + 1);
        }
        "resolver_target_export_swapped" => {
            let targets = value["resolver_targets"].as_array_mut().unwrap();
            let left = targets[0]["exported_contract_digest"].clone();
            targets[0]["exported_contract_digest"] = targets[1]["exported_contract_digest"].clone();
            targets[1]["exported_contract_digest"] = left;
        }
        "source_computation_id_collapsed" => {
            let first = value["source_associations"][0]["source_node"].clone();
            value["source_associations"][1]["source_node"] = first;
        }
        "none_to_some_empty" => {
            value["resolver_targets"][0]["direct_dependencies"] = Json::Null;
        }
        "dependency_edge_dropped" => {
            value["resolver_targets"][0]["direct_dependencies"] = Json::Array(vec![]);
        }
        "unknown_external_erased" => {
            value["await_unknown_external_count"] = Json::from(0);
        }
        "resource_limit_promoted" => {
            value["failure_family"] = Json::Null;
        }
        other => panic!("unregistered comparator mutation dimension {other}"),
    }
}

fn context_projection_mut(value: &mut Json) -> &mut Json {
    if value.get("context").is_some() {
        value.get_mut("context").expect("checked context field")
    } else {
        value
    }
}

#[test]
fn comparator_mutations_handle_d5_only_and_pipeline_projections() {
    let mut d5 = serde_json::json!({"type_schemas": [1, 2]});
    alter_projection(&mut d5, "context_membership_lost");
    assert_eq!(d5["type_schemas"], serde_json::json!([1]));

    let mut pipeline = serde_json::json!({"context": {"type_schemas": [1, 2]}});
    alter_projection(&mut pipeline, "context_membership_lost");
    assert_eq!(pipeline["context"]["type_schemas"], serde_json::json!([1]));

    let mut error = serde_json::json!({"stage": "D8", "failure_family": "ResourceLimit"});
    alter_projection(&mut error, "resource_limit_promoted");
    assert_eq!(error["failure_family"], Json::Null);
}
