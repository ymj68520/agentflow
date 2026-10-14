use af_portable::PortableProofCodec;
use serde_json::Value as Json;
use sha2::Digest as _;
use std::path::Path;
use std::process::Command;

fn fixture(name: &str) -> Vec<u8> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!(
            "../../spec/conformance/portable-root-export-revalidation/{name}"
        ))
        .canonicalize()
        .and_then(std::fs::read)
        .expect("registered portable root/export fixture")
}

fn corpus_manifest() -> Json {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../spec/conformance/portable-root-export-revalidation/manifest.json")
        .canonicalize()
        .and_then(std::fs::read)
        .and_then(|bytes| serde_json::from_slice(&bytes).map_err(std::io::Error::other))
        .expect("independent D9-D10 manifest")
}

fn stage_name(stage: af_portable::ValidationStage) -> &'static str {
    match stage {
        af_portable::ValidationStage::D0 => "D0",
        af_portable::ValidationStage::D1 => "D1",
        af_portable::ValidationStage::D2 => "D2",
        af_portable::ValidationStage::D3 => "D3",
        af_portable::ValidationStage::D4 => "D4",
        af_portable::ValidationStage::D5 => "D5",
        af_portable::ValidationStage::D6 => "D6",
        af_portable::ValidationStage::D7 => "D7",
        af_portable::ValidationStage::D8 => "D8",
        af_portable::ValidationStage::D9 => "D9",
        af_portable::ValidationStage::D10 => "D10",
    }
}

fn apply_bundle_mutations(base: &[u8], mutations: &[Json]) -> Vec<u8> {
    let mut value: Json = serde_json::from_slice(base).expect("canonical portable JSON");
    for mutation in mutations {
        let path = mutation["path"].as_array().expect("mutation path");
        assert_eq!(mutation["op"], "set");
        assert!(!path.is_empty());
        let tokens = path
            .iter()
            .map(|part| part.as_str().expect("text path token"))
            .collect::<Vec<_>>();
        let mut current = &mut value;
        for token in &tokens[..tokens.len() - 1] {
            current = match current {
                Json::Object(fields) => fields.get_mut(*token).expect("mutation object path"),
                Json::Array(items) => &mut items[token.parse::<usize>().expect("array index")],
                _ => panic!("mutation path traverses a scalar"),
            };
        }
        let replacement = mutation["value"].clone();
        match current {
            Json::Object(fields) => {
                fields.insert(tokens[tokens.len() - 1].to_owned(), replacement);
            }
            Json::Array(items) => {
                items[tokens[tokens.len() - 1]
                    .parse::<usize>()
                    .expect("array index")] = replacement;
            }
            _ => panic!("mutation parent is a scalar"),
        }
    }
    serde_json::to_vec(&value).expect("canonical machine JSON bytes")
}

fn positive_oracle_view(projection: &Json) -> Json {
    let root = &projection["d9_root"];
    let report = &projection["d10_root_export"];
    let mut dimension_verdicts = serde_json::Map::new();
    for item in report["dimensions"]
        .as_array()
        .expect("nine formal dimensions")
    {
        dimension_verdicts.insert(
            item["dimension"].as_str().unwrap().to_owned(),
            item["proof"]["verdict"].clone(),
        );
    }
    let mut usage_coordinates = serde_json::Map::new();
    for coordinate in root["root_dimensions"]["usage"]["value"]["coordinates"]
        .as_array()
        .expect("explicit root usage coordinates")
    {
        let key = format!(
            "{}/{}",
            coordinate["phase"].as_str().unwrap(),
            coordinate["meter"]["name"].as_str().unwrap()
        );
        usage_coordinates.insert(key, coordinate["bound"]["upper"].clone());
    }
    serde_json::json!({
        "root_operator":root["root_control"]["operator"],
        "root_effect_entries":root["effect_occurrences"].as_array().unwrap().len(),
        "root_recovery_actions":projection["d9_root_recovery"]["actions"]["actions"].as_array().unwrap().len(),
        "usage_coordinates":Json::Object(usage_coordinates),
        "execution_source_mapping_count":report["assertion_identity"]["execution_mappings"].as_array().unwrap().len(),
        "dimension_verdicts":Json::Object(dimension_verdicts),
        "static_compatibility":report["static_compatibility"],
        "capability_issued":report["checked_static_compatibility_capability_issued"],
        "body_conformance":report["body_conformance"],
        "implementation_conformance":report["implementation_conformance"],
        "publisher_authentication":report["publisher_authentication"],
        "runtime_authority":report["runtime_authority"],
    })
}

fn set_projection_path(value: &mut Json, path: &[Json], replacement: Json) {
    let tokens = path
        .iter()
        .map(|item| item.as_str().unwrap())
        .collect::<Vec<_>>();
    let mut current = value;
    for token in &tokens[..tokens.len() - 1] {
        current = current.as_object_mut().unwrap().get_mut(*token).unwrap();
    }
    current
        .as_object_mut()
        .unwrap()
        .insert(tokens[tokens.len() - 1].to_owned(), replacement);
}

fn semantic_path(path: &[String]) -> String {
    let mut rendered = String::from("$");
    for part in path {
        if part.parse::<usize>().is_ok() {
            rendered.push('[');
            rendered.push_str(part);
            rendered.push(']');
        } else {
            rendered.push_str("[\"");
            rendered.push_str(&part.replace('\\', "\\\\").replace('\"', "\\\""));
            rendered.push_str("\"]");
        }
    }
    rendered
}

fn first_semantic_difference(expected: &Json, actual: &Json, path: &str) -> Option<String> {
    match (expected, actual) {
        (Json::Object(expected), Json::Object(actual)) => {
            let mut keys = expected.keys().chain(actual.keys()).collect::<Vec<_>>();
            keys.sort();
            keys.dedup();
            for key in keys {
                let child_path = format!(
                    "{path}[\"{}\"]",
                    key.replace('\\', "\\\\").replace('\"', "\\\"")
                );
                match (expected.get(key), actual.get(key)) {
                    (Some(left), Some(right)) => {
                        if let Some(difference) =
                            first_semantic_difference(left, right, &child_path)
                        {
                            return Some(difference);
                        }
                    }
                    _ => return Some(child_path),
                }
            }
            None
        }
        (Json::Array(expected), Json::Array(actual)) => {
            let common = expected.len().min(actual.len());
            for index in 0..common {
                let child_path = format!("{path}[{index}]");
                if let Some(difference) =
                    first_semantic_difference(&expected[index], &actual[index], &child_path)
                {
                    return Some(difference);
                }
            }
            (expected.len() != actual.len()).then(|| format!("{path}[{common}]"))
        }
        _ if expected == actual => None,
        _ => Some(path.to_owned()),
    }
}

fn collect_semantic_scalars(value: &Json, path: &mut Vec<String>, output: &mut Vec<Vec<String>>) {
    match value {
        Json::Object(fields) => {
            for (key, child) in fields {
                if key == "reason" {
                    continue;
                }
                path.push(key.clone());
                collect_semantic_scalars(child, path, output);
                path.pop();
            }
        }
        Json::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                path.push(index.to_string());
                collect_semantic_scalars(child, path, output);
                path.pop();
            }
        }
        _ => output.push(path.clone()),
    }
}

fn mutate_semantic_path(value: &mut Json, path: &[String]) {
    assert!(!path.is_empty());
    let mut current = value;
    for part in &path[..path.len() - 1] {
        current = match current {
            Json::Object(fields) => fields.get_mut(part).expect("manifest object field exists"),
            Json::Array(items) => &mut items[part.parse::<usize>().expect("manifest array index")],
            _ => panic!("semantic mutation path traverses a scalar"),
        };
    }
    let last = path.last().unwrap();
    let target = match current {
        Json::Object(fields) => fields.get_mut(last).expect("manifest object leaf exists"),
        Json::Array(items) => &mut items[last.parse::<usize>().expect("manifest array leaf")],
        _ => panic!("semantic mutation parent is a scalar"),
    };
    *target = match target {
        Json::Null => serde_json::json!({"mutation":"present"}),
        Json::Bool(value) => Json::Bool(!*value),
        Json::Number(value) => value
            .as_u64()
            .and_then(|number| number.checked_add(1))
            .map_or_else(
                || serde_json::json!("mutated-number"),
                |number| serde_json::json!(number),
            ),
        Json::String(value) => Json::String(format!("{value}#mutated")),
        Json::Array(_) | Json::Object(_) => unreachable!("only scalar paths are collected"),
    };
}

fn assert_projection_mutations(projection: &Json, subtree: &str, count: usize) {
    let expected_subtree = projection.get(subtree).expect("formal projection subtree");
    let mut paths = Vec::new();
    collect_semantic_scalars(expected_subtree, &mut vec![subtree.to_owned()], &mut paths);
    assert!(
        paths.len() >= count,
        "{subtree} has {} scalar fields; mutation contract requires {count}",
        paths.len()
    );
    for path in paths.into_iter().take(count) {
        let mut actual = projection.clone();
        mutate_semantic_path(&mut actual, &path);
        let expected_path = semantic_path(&path);
        let difference = first_semantic_difference(projection, &actual, "$");
        assert_eq!(
            difference.as_deref(),
            Some(expected_path.as_str()),
            "comparator must report the exact mutated semantic field"
        );
    }
}

#[test]
fn recursive_semantic_comparator_is_fail_closed_for_shape_order_and_options() {
    let expected = serde_json::json!({"items":[{"tag":"left"},{"tag":"right"}],"optional":null});
    let missing = serde_json::json!({"items":[{"tag":"left"},{"tag":"right"}]});
    assert_eq!(
        first_semantic_difference(&expected, &missing, "$").as_deref(),
        Some("$[\"optional\"]")
    );

    let extra = serde_json::json!({"items":[{"tag":"left"},{"tag":"right"}],"optional":null,"new_semantic_field":true});
    assert_eq!(
        first_semantic_difference(&expected, &extra, "$").as_deref(),
        Some("$[\"new_semantic_field\"]")
    );

    let reordered = serde_json::json!({"items":[{"tag":"right"},{"tag":"left"}],"optional":null});
    assert_eq!(
        first_semantic_difference(&expected, &reordered, "$").as_deref(),
        Some("$[\"items\"][0][\"tag\"]")
    );

    let wrong_option = serde_json::json!({"items":[{"tag":"left"},{"tag":"right"}],"optional":[]});
    assert_eq!(
        first_semantic_difference(&expected, &wrong_option, "$").as_deref(),
        Some("$[\"optional\"]")
    );
}

#[test]
fn every_registered_opcode_bundle_passes_d0_to_d4() {
    let manifest = corpus_manifest();
    let cases = manifest["cases"].as_array().unwrap();
    let d9_cases = cases
        .iter()
        .filter(|case| case["group"] == "D9-Control" || case["group"] == "D9-Recovery")
        .collect::<Vec<_>>();
    assert_eq!(d9_cases.len(), 38);
    for case in d9_cases {
        let relative = case["bundle_file"].as_str().unwrap();
        let bytes = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(relative)
            .canonicalize()
            .and_then(std::fs::read)
            .expect("registered opcode Bundle bytes");
        let actual = PortableProofCodec::default()
            .validate_supported(&bytes)
            .unwrap_or_else(|error| {
                panic!("{} rejected at {:?}: {error}", case["case_id"], error.stage)
            });
        assert_eq!(actual.semantic_version(), "agentflow.root-export-checker@1");
    }
}

#[test]
fn registered_d9_cases_rederive_all_thirteen_production_opcodes() {
    let manifest = corpus_manifest();
    let cases = manifest["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["group"] == "D9-Control")
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 22);

    for chunk in cases.chunks(3) {
        let handles = chunk
            .iter()
            .map(|case| {
                let case = case.clone();
                std::thread::spawn(move || {
                    let relative = case["bundle_file"].as_str().unwrap();
                    let bytes = Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../..")
                        .join(relative)
                        .canonicalize()
                        .and_then(std::fs::read)
                        .expect("registered full portable Bundle");
                    let supported = PortableProofCodec::default()
                        .validate_supported(&bytes)
                        .unwrap_or_else(|error| panic!("{} D0-D4: {error}", case["case_id"]));
                    let analysis = supported
                        .revalidate_root_export()
                        .unwrap_or_else(|error| panic!("{} D5-D10: {error}", case["case_id"]));
                    let projection = analysis.typed_projection();
                    assert_eq!(
                        projection["d9_root"]["root_control"]["operator"],
                        case["expected_root_operator"],
                        "{} root opcode",
                        case["case_id"]
                    );
                    assert_eq!(
                        projection["d9_root"]["closure"]["computation_order"],
                        case["expected_computation_order"],
                        "{} exact owned computation closure",
                        case["case_id"]
                    );
                    assert_eq!(
                        projection["d9_root"]["closure"]["leaf_order"], case["expected_leaf_nodes"],
                        "{} exact leaf resolution closure",
                        case["case_id"]
                    );
                    assert_eq!(analysis.report().dimensions.len(), 9);
                    assert!(
                        projection["d9_root_recovery"]["overall_verdict"]
                            .as_str()
                            .is_some(),
                        "{} RootRecovery is from the same formal derivation",
                        case["case_id"]
                    );
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            handle.join().expect("D9 opcode worker panicked");
        }
    }
}

#[test]
fn registered_d9_recovery_cases_preserve_bounds_provenance_and_multiplicity() {
    let cases = corpus_manifest()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["group"] == "D9-Recovery")
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 16);

    for chunk in cases.chunks(3) {
        let handles = chunk
            .iter()
            .map(|case| {
                let case = case.clone();
                std::thread::spawn(move || {
                    let bytes = Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../..")
                        .join(case["bundle_file"].as_str().unwrap())
                        .canonicalize()
                        .and_then(std::fs::read)
                        .expect("recovery Portable Bundle bytes");
                    let supported = PortableProofCodec::default()
                        .validate_supported(&bytes)
                        .unwrap_or_else(|error| panic!("{} D0-D4: {error}", case["case_id"]));
                    let analysis = supported
                        .revalidate_root_export()
                        .unwrap_or_else(|error| panic!("{} D5-D10: {error}", case["case_id"]));
                    let projection = analysis.typed_projection();
                    if case["scenario"] == "outer-retry-1" {
                        assert_projection_mutations(&projection, "d9_root_recovery", 12);
                    }
                    let actions = projection["d9_root_recovery"]["actions"]["actions"]
                        .as_array()
                        .unwrap_or_else(|| {
                            panic!("{} lacks complete D9 action projection", case["case_id"])
                        });
                    assert_eq!(
                        actions.len(),
                        usize::try_from(case["expected_action_count"].as_u64().unwrap()).unwrap()
                    );

                    let actual_nodes = actions
                        .iter()
                        .map(|action| action["source"]["source_node"].as_u64().unwrap())
                        .collect::<Vec<_>>();
                    let expected_nodes = case["expected_source_nodes"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|id| id.as_u64().unwrap())
                        .collect::<Vec<_>>();
                    assert_eq!(actual_nodes, expected_nodes);
                    for (index, action) in actions.iter().enumerate() {
                        assert_eq!(
                            action["source"]["forward_entry_ordinal"],
                            case["expected_forward_entry_ordinal"]
                        );
                        assert_eq!(
                            action["root_bound"]["upper"]["upper"]["finite"],
                            case["expected_root_action_uppers"][index]
                        );
                        assert_eq!(
                            action["root_bound"]["applied_families"]
                                .as_array()
                                .unwrap()
                                .len(),
                            usize::try_from(
                                case["expected_root_factor_counts"][index].as_u64().unwrap()
                            )
                            .unwrap()
                        );
                        let local = action["local_bound"]
                            .as_object()
                            .map(|_| &action["local_bound"]);
                        assert!(
                            local.is_some(),
                            "{} must retain its nearest CompensationScope",
                            case["case_id"]
                        );
                        assert_eq!(
                            local.unwrap()["upper"]["upper"]["finite"],
                            case["expected_local_action_uppers"][index]
                        );
                        assert_eq!(
                            local.unwrap()["applied_families"].as_array().unwrap().len(),
                            usize::try_from(
                                case["expected_local_factor_counts"][index]
                                    .as_u64()
                                    .unwrap()
                            )
                            .unwrap()
                        );
                    }
                    assert_eq!(analysis.report().dimensions.len(), 9);
                    assert!(analysis.checked_compatibility().is_none());
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            handle.join().expect("D9 recovery worker panicked");
        }
    }
}

#[test]
fn full_registered_bundle_revalidates_through_d0_to_d10() {
    let supported = PortableProofCodec::default()
        .validate_supported(&fixture("full-static-positive.ace1"))
        .expect("D0-D4 supported bundle");
    let analysis = supported
        .revalidate_root_export()
        .unwrap_or_else(|error| panic!("D5-D10 revalidation failed: {error}"));

    assert_eq!(analysis.report().dimensions.len(), 9);
    assert!(analysis.checked_compatibility().is_some());
    assert_eq!(analysis.report().static_compatibility.as_str(), "proven");
    assert_eq!(
        analysis.report().body_conformance.as_str(),
        "requires_evidence"
    );
    assert_eq!(
        analysis.report().implementation_conformance.as_str(),
        "requires_evidence"
    );
    assert_eq!(
        analysis.report().runtime_authority_evidence.as_str(),
        "requires_evidence"
    );
    assert!(
        analysis
            .report()
            .dimensions
            .iter()
            .all(|proof| proof.proof.verdict.as_str() == "proven")
    );

    let projection = analysis.typed_projection();
    assert_eq!(
        projection["d10_root_export"]["dimensions"]
            .as_array()
            .unwrap()
            .len(),
        9
    );
    assert_eq!(
        projection["d10_root_export"]["checked_static_compatibility_capability_issued"],
        true
    );
    assert_eq!(projection["d9_root_recovery"]["overall_verdict"], "proven");
    let manifest = corpus_manifest();
    let expected = manifest["positive_expected"].clone();
    let view = positive_oracle_view(&projection);
    assert_eq!(view, expected, "independent Python declaration oracle");
    let comparator_mutations = manifest["comparator_mutations"].as_array().unwrap();
    assert_eq!(
        comparator_mutations.len(),
        usize::try_from(manifest["comparator_mutation_count"].as_u64().unwrap()).unwrap()
    );
    for mutation in comparator_mutations {
        let mut changed = view.clone();
        set_projection_path(
            &mut changed,
            mutation["path"].as_array().unwrap(),
            mutation["value"].clone(),
        );
        assert_ne!(
            changed, expected,
            "comparator mutation {}",
            mutation["mutation_id"]
        );
    }

    assert_projection_mutations(&projection, "d9_root", 12);
    assert_projection_mutations(&projection, "d9_root_recovery", 12);
    assert_projection_mutations(&projection, "d10_root_export", 22);
}

#[test]
fn missing_execution_source_mapping_is_unproven_and_never_issues_capability() {
    let supported = PortableProofCodec::default()
        .validate_supported(&fixture("negative-execution-map-missing.ace1"))
        .expect("D0-D4 supported negative bundle");
    let analysis = supported
        .revalidate_root_export()
        .unwrap_or_else(|error| panic!("D5-D10 analysis failed: {error}"));

    assert_eq!(analysis.report().static_compatibility.as_str(), "unproven");
    assert!(analysis.checked_compatibility().is_none());
    let execution = analysis
        .report()
        .dimension(af_computation::RootExportDimension::Execution)
        .expect("formal X dimension");
    assert_eq!(execution.proof.verdict.as_str(), "unproven");
}

#[test]
fn p30_split_rows_are_not_combined_and_same_row_positive_remains_accepted() {
    let codec = PortableProofCodec::default();
    let split = codec
        .validate_supported(&fixture("p30-split-row-negative.ace1"))
        .expect("P30 D0-D4 exact typed inputs");
    let split = split
        .revalidate_root_export()
        .unwrap_or_else(|error| panic!("P30 D5-D10: {error}"));
    let epsilon = split
        .report()
        .dimension(af_computation::RootExportDimension::Effect)
        .expect("formal epsilon dimension");
    assert_eq!(epsilon.proof.verdict.as_str(), "unproven");
    assert_eq!(split.report().static_compatibility.as_str(), "unproven");
    assert!(split.checked_compatibility().is_none());
    assert_projection_mutations(&split.typed_projection(), "d10_root_export", 10);

    let same_row = codec
        .validate_supported(&fixture("p30-same-row-positive.ace1"))
        .expect("same-row positive D0-D4 inputs");
    let same_row = same_row
        .revalidate_root_export()
        .unwrap_or_else(|error| panic!("same-row positive D5-D10: {error}"));
    assert_eq!(
        same_row
            .report()
            .dimension(af_computation::RootExportDimension::Effect)
            .unwrap()
            .proof
            .verdict
            .as_str(),
        "proven"
    );
    assert_eq!(same_row.report().static_compatibility.as_str(), "proven");
    assert!(same_row.checked_compatibility().is_some());
}

#[test]
fn p31_target_export_identity_drift_is_rejected_and_fresh_b_is_rederived() {
    let stale = PortableProofCodec::default()
        .validate_supported(&fixture("p31-snapshot-a-declaration-b-current.ace1"))
        .expect("P31 D0-D4 typed graph");
    let stale = stale
        .revalidate_root_export()
        .unwrap_or_else(|error| panic!("P31 stale target D5-D10: {error}"));
    assert_eq!(stale.report().static_compatibility.as_str(), "unproven");
    assert!(stale.checked_compatibility().is_none());
    let stale_projection = stale.typed_projection();
    let stale_source_pin =
        &stale_projection["d9_root"]["effect_occurrences"][0]["source_contract_digest"];
    assert_eq!(
        stale_source_pin["hex"],
        corpus_manifest()["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["case_id"]
                == "portable-root-export-revalidation/p31-snapshot-a-declaration-b-current")
            .unwrap()["expected_current_target_export_digest"]["hex"]
    );
    assert_eq!(
        stale
            .report()
            .preflight
            .iter()
            .find(|proof| proof.dimension == "subflow-dependency-pins")
            .unwrap()
            .verdict
            .as_str(),
        "unproven"
    );

    let fresh = PortableProofCodec::default()
        .validate_supported(&fixture("p31-fresh-root-b-top-ceiling-one.ace1"))
        .expect("P31 fresh B D0-D4");
    let fresh = fresh
        .revalidate_root_export()
        .unwrap_or_else(|error| panic!("P31 fresh B D5-D10: {error}"));
    assert_eq!(fresh.report().static_compatibility.as_str(), "violation");
    assert!(fresh.checked_compatibility().is_none());
    assert_eq!(
        fresh
            .report()
            .dimension(af_computation::RootExportDimension::Effect)
            .unwrap()
            .proof
            .verdict
            .as_str(),
        "violation"
    );
    let fresh_projection = fresh.typed_projection();
    assert_eq!(
        fresh_projection["d9_root"]["effect_occurrences"][0]["source_contract_digest"]["hex"],
        corpus_manifest()["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["case_id"]
                == "portable-root-export-revalidation/p31-fresh-root-b-top-ceiling-one")
            .unwrap()["expected_current_target_export_digest"]["hex"]
    );
}

#[test]
fn twenty_d10_execution_identity_mutations_are_all_formally_nonproven() {
    let cases = corpus_manifest()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| {
            case["group"] == "D10" && case.get("mutations").and_then(Json::as_array).is_some()
        })
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 20);

    for chunk in cases.chunks(3) {
        let handles = chunk
            .iter()
            .map(|case| {
                let case = case.clone();
                std::thread::spawn(move || {
                    let base = fixture("full-static-positive.ace1");
                    let bytes =
                        apply_bundle_mutations(&base, case["mutations"].as_array().unwrap());
                    let supported = PortableProofCodec::default()
                        .validate_supported(&bytes)
                        .unwrap_or_else(|error| {
                            panic!("{} rejected before D10: {error}", case["case_id"])
                        });
                    let analysis = supported.revalidate_root_export().unwrap_or_else(|error| {
                        panic!("{} failed before formal report: {error}", case["case_id"])
                    });
                    let execution = analysis
                        .report()
                        .dimension(af_computation::RootExportDimension::Execution)
                        .expect("formal X dimension");
                    assert_ne!(
                        execution.proof.verdict.as_str(),
                        "proven",
                        "{}",
                        case["case_id"]
                    );
                    assert_ne!(analysis.report().static_compatibility.as_str(), "proven");
                    assert!(analysis.checked_compatibility().is_none());
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            handle.join().expect("D10 mutation worker panicked");
        }
    }
}

#[test]
fn context_resolver_and_compiler_provenance_mutations_fail_or_recompute_freshly() {
    let cases = corpus_manifest()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["group"] == "Context-Resolver-Compiler")
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 8);

    for case in cases {
        let bytes = apply_bundle_mutations(
            &fixture("full-static-positive.ace1"),
            case["mutations"].as_array().unwrap(),
        );
        let expected = case["expected_stage"].as_str().unwrap();
        let supported = match PortableProofCodec::default().validate_supported(&bytes) {
            Ok(supported) => supported,
            Err(error) => {
                assert_eq!(
                    stage_name(error.stage),
                    expected,
                    "{}: {error}",
                    case["case_id"]
                );
                assert!(matches!(expected, "D2" | "D3" | "D4"));
                continue;
            }
        };
        let result = supported.revalidate_root_export();
        if expected == "D10" {
            let analysis = result.unwrap_or_else(|error| panic!("{}: {error}", case["case_id"]));
            assert_eq!(analysis.report().static_compatibility.as_str(), "proven");
            assert!(analysis.checked_compatibility().is_some());
            let usage_meters = &analysis.inputs().typed_projection()["context"]["usage_meters"];
            assert_eq!(usage_meters[0]["unit"]["name"], "call-v2");
        } else {
            let error =
                result.expect_err("provenance mismatch must reject in its earliest formal stage");
            assert_eq!(
                stage_name(error.stage),
                expected,
                "{}: {error}",
                case["case_id"]
            );
        }
    }
}

#[test]
fn portable_bundle_producer_process() {
    let Some(output) = std::env::var_os("AGENTFLOW_PORTABLE_TEST_BUNDLE") else {
        return;
    };
    let source = std::env::var_os("AGENTFLOW_PORTABLE_TEST_SOURCE").unwrap_or_else(|| {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../spec/conformance/portable-root-export-revalidation/full-static-positive.ace1")
            .into_os_string()
    });
    let mut input = Path::new(&source)
        .canonicalize()
        .and_then(std::fs::read)
        .expect("independent positive fixture");
    if let Some(mutations) = std::env::var_os("AGENTFLOW_PORTABLE_TEST_MUTATIONS") {
        let mutations: Vec<Json> = serde_json::from_str(&mutations.to_string_lossy()).unwrap();
        input = apply_bundle_mutations(&input, &mutations);
    }
    let codec = PortableProofCodec::default();
    let decoded = codec.decode_d0(&input).expect("producer D0");
    let canonical = codec
        .validate_d1(decoded, &input)
        .expect("producer canonical D1");
    let output = Path::new(&output);
    std::fs::write(output, canonical.canonical_bytes()).expect("immutable byte handoff");
    let metadata = serde_json::json!({
        "pid":std::process::id(),
        "canonical_byte_length":canonical.canonical_bytes().len(),
        "canonical_sha256":hex::encode(sha2::Sha256::digest(canonical.canonical_bytes())),
    });
    std::fs::write(
        output.with_extension("producer.json"),
        serde_json::to_vec(&metadata).unwrap(),
    )
    .expect("producer evidence");
}

#[test]
fn portable_bundle_consumer_process() {
    let Some(bundle_path) = std::env::var_os("AGENTFLOW_PORTABLE_TEST_BUNDLE") else {
        return;
    };
    let bundle_path = Path::new(&bundle_path);
    let output_path = Path::new(
        &std::env::var_os("AGENTFLOW_PORTABLE_TEST_PROJECTION").expect("consumer output path"),
    )
    .to_path_buf();
    let bytes = std::fs::read(bundle_path).expect("consumer receives only portable bytes");
    let wrapper = match PortableProofCodec::default().validate_supported(&bytes) {
        Err(error) => {
            serde_json::json!({"pid":std::process::id(),"stage":stage_name(error.stage),"error_family":error.family,"projection":null})
        }
        Ok(supported) => match supported.revalidate_root_export() {
            Err(error) => {
                serde_json::json!({"pid":std::process::id(),"stage":stage_name(error.stage),"error_family":error.family,"projection":null})
            }
            Ok(analysis) => {
                serde_json::json!({"pid":std::process::id(),"stage":"D10","error_family":null,"projection":analysis.typed_projection()})
            }
        },
    };
    std::fs::write(output_path, serde_json::to_vec(&wrapper).unwrap())
        .expect("consumer semantic projection");
}

#[test]
fn producer_and_fresh_consumer_revalidate_across_process_boundary() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("monotonic test timestamp")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "agentflow-portable-revalidation-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).expect("isolated handoff directory");
    let executable = std::env::current_exe().expect("test executable path");
    let cases = corpus_manifest()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| case["group"] == "Cross-Process")
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 6);
    for (index, case) in cases.iter().enumerate() {
        let bundle_path = directory.join(format!("bundle-{index}.ace1"));
        let projection_path = directory.join(format!("consumer-projection-{index}.json"));
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(case["bundle_file"].as_str().unwrap())
            .canonicalize()
            .unwrap();
        let mut producer_command = Command::new(&executable);
        producer_command
            .arg("--exact")
            .arg("portable_bundle_producer_process")
            .env("AGENTFLOW_PORTABLE_TEST_BUNDLE", &bundle_path)
            .env("AGENTFLOW_PORTABLE_TEST_SOURCE", source);
        if let Some(mutations) = case.get("mutations").and_then(Json::as_array) {
            producer_command.env(
                "AGENTFLOW_PORTABLE_TEST_MUTATIONS",
                serde_json::to_string(mutations).unwrap(),
            );
        }
        let producer = producer_command
            .status()
            .expect("launch independent producer process");
        assert!(
            producer.success(),
            "producer process failed for {}",
            case["case_id"]
        );

        let consumer = Command::new(&executable)
            .arg("--exact")
            .arg("portable_bundle_consumer_process")
            .env("AGENTFLOW_PORTABLE_TEST_BUNDLE", &bundle_path)
            .env("AGENTFLOW_PORTABLE_TEST_PROJECTION", &projection_path)
            .status()
            .expect("launch fresh consumer process");
        assert!(
            consumer.success(),
            "consumer process failed for {}",
            case["case_id"]
        );

        let producer_metadata: Json = serde_json::from_slice(
            &std::fs::read(bundle_path.with_extension("producer.json")).unwrap(),
        )
        .unwrap();
        let consumer_output: Json =
            serde_json::from_slice(&std::fs::read(&projection_path).unwrap()).unwrap();
        assert_ne!(
            producer_metadata["pid"], consumer_output["pid"],
            "{}",
            case["case_id"]
        );
        assert_ne!(producer_metadata["pid"], std::process::id());
        assert_ne!(consumer_output["pid"], std::process::id());
        assert_eq!(
            consumer_output["stage"], case["expected_consumer_stage"],
            "{}",
            case["case_id"]
        );
        if let Some(projection) = consumer_output["projection"].as_object() {
            assert_eq!(
                projection["d10_root_export"]["static_compatibility"],
                case.get("expected_static_compatibility")
                    .cloned()
                    .unwrap_or(Json::String("unproven".into())),
                "{} semantic verdict",
                case["case_id"]
            );
            if case.get("expected_capability_issued").is_some() {
                assert_eq!(
                    projection["d10_root_export"]["checked_static_compatibility_capability_issued"],
                    case["expected_capability_issued"],
                    "{} capability issuance",
                    case["case_id"]
                );
            }
            assert_eq!(
                projection["d10_root_export"]["body_conformance"],
                "requires_evidence"
            );
        } else {
            assert!(consumer_output["error_family"].is_string());
        }
    }

    std::fs::remove_dir_all(directory).expect("remove the exact temporary handoff directory");
}
