use af_portable::{DecoderLimits, PortableProofCodec, PortableProofError, ValidationStage};
use base64::Engine as _;
use serde::Deserialize;
use serde_json::Value as Json;
use sha2::Digest as _;
use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[derive(Deserialize)]
struct Manifest {
    case_count: usize,
    #[allow(dead_code)]
    base_file: String,
    base_sha256: String,
    required_bundle_fields: Vec<String>,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct ComparatorManifest {
    mutation_count: usize,
    mutations: Vec<ComparatorMutation>,
}

#[derive(Deserialize)]
struct ComparatorMutation {
    mutation_id: String,
    dimension: String,
    witness_case_id: String,
    expected_stage: String,
    expected_error_family: Option<String>,
}

#[derive(Deserialize)]
#[allow(clippy::struct_field_names)]
struct Case {
    case_id: String,
    #[allow(dead_code)]
    group: String,
    mutations: Vec<Mutation>,
    raw_mutation: Option<String>,
    raw_noncanonical: bool,
    input_byte_length: usize,
    input_sha256: String,
    expected_stage: String,
    expected_error_family: Option<String>,
    decoder_limits: Option<DecoderBound>,
    expected_bundle_sha256: Option<String>,
    expected_projection_sha256: Option<String>,
    expected_bundle_byte_length: Option<usize>,
    expected_typeschema_count: Option<usize>,
    required_semantic_field_paths: Vec<String>,
    trust_boundary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Observation {
    stage: String,
    error_family: Option<String>,
    input_sha256: String,
    canonical_sha256: Option<String>,
    canonical_byte_length: Option<usize>,
    schema_type_count: Option<usize>,
    source_count: Option<usize>,
    applied_rule_count: Option<usize>,
    semantic_version: Option<String>,
    trust_boundary: String,
}

#[derive(Deserialize)]
struct DecoderBound {
    #[serde(default)]
    max_container_items: Option<usize>,
    #[serde(default)]
    max_graph_work: Option<usize>,
}

#[derive(Deserialize)]
struct Mutation {
    op: String,
    path: Vec<Json>,
    #[serde(default)]
    value: Json,
}

fn corpus() -> (Manifest, Vec<u8>) {
    let root = repo().join("spec/conformance/portable-proof-codec");
    let manifest: Manifest =
        serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
    let base = std::fs::read(repo().join(&manifest.base_file)).unwrap();
    (manifest, base)
}

fn path_part(value: &Json) -> &str {
    match value {
        Json::String(text) => text,
        Json::Number(_) => panic!("mutation path index must be serialized as text"),
        _ => panic!("mutation path components are strings or array indexes"),
    }
}

fn descend_mut<'a>(value: &'a mut Json, path: &[Json]) -> &'a mut Json {
    let mut current = value;
    for part in path {
        current = match current {
            Json::Object(map) => map.get_mut(path_part(part)).expect("mutation object path"),
            Json::Array(items) => {
                let index = path_part(part).parse::<usize>().expect("array index");
                &mut items[index]
            }
            _ => panic!("mutation path crosses a scalar"),
        };
    }
    current
}

fn apply_json_mutations(base: &[u8], mutations: &[Mutation], preserve_raw_text: bool) -> Vec<u8> {
    let mut value: Json = serde_json::from_slice(base).expect("frozen canonical base Bundle");
    for mutation in mutations {
        match mutation.op.as_str() {
            "set" => {
                if mutation.path.is_empty() {
                    value = mutation.value.clone();
                    continue;
                }
                let last = path_part(mutation.path.last().unwrap()).to_owned();
                let parent = descend_mut(&mut value, &mutation.path[..mutation.path.len() - 1]);
                match parent {
                    Json::Object(map) => {
                        map.insert(last, mutation.value.clone());
                    }
                    Json::Array(items) => {
                        let index = last.parse::<usize>().expect("array index");
                        items[index] = mutation.value.clone();
                    }
                    _ => panic!("set parent is not a container"),
                }
            }
            "remove" | "remove-index" => {
                let last = path_part(mutation.path.last().unwrap()).to_owned();
                let parent = descend_mut(&mut value, &mutation.path[..mutation.path.len() - 1]);
                match parent {
                    Json::Object(map) => {
                        map.remove(&last).expect("remove field");
                    }
                    Json::Array(items) => {
                        let index = last.parse::<usize>().expect("array index");
                        items.remove(index);
                    }
                    _ => panic!("remove parent is not a container"),
                }
            }
            "reverse" => match descend_mut(&mut value, &mutation.path) {
                Json::Array(items) => items.reverse(),
                _ => panic!("reverse target must be an array"),
            },
            "append-copy" => match descend_mut(&mut value, &mutation.path) {
                Json::Array(items) => {
                    let last = items.last().expect("nonempty array").clone();
                    items.push(last);
                }
                _ => panic!("append target must be an array"),
            },
            "bump_registry_revision" => {
                let content =
                    &mut value["compiler_semantics_manifests"][0]["content"]["registries"]["rules"];
                let encoded = content["content"].as_str().expect("registry bytes");
                let raw = base64::engine::general_purpose::URL_SAFE_NO_PAD
                    .decode(encoded)
                    .expect("base64url registry");
                let mut registry: Json =
                    serde_json::from_slice(&raw).expect("valid embedded RulesFile");
                let revision = registry["registry_revision"]
                    .as_u64()
                    .expect("registry revision");
                registry["registry_revision"] = Json::from(revision + 1);
                let changed = serde_json::to_vec(&registry).expect("canonical JSON instance");
                content["content"] =
                    Json::String(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&changed));
                content["content_sha256"]["hex"] =
                    Json::String(hex::encode(sha2::Sha256::digest(&changed)));
            }
            other => panic!("unknown frozen mutation {other}"),
        }
    }
    // serde_json uses the ordered map representation; this is a test input
    // materializer only. Expected bytes and stage were independently generated
    // by Python and are checked below before production decode.
    let _ = preserve_raw_text;
    serde_json::to_vec(&value).expect("typed mutation serialization")
}

fn raw_mutation(base: &[u8], recipe: &str) -> Vec<u8> {
    match recipe {
        "invalid_utf8" => vec![0xff],
        "duplicate_key" => {
            let text = std::str::from_utf8(base).unwrap();
            text.replace(
                "\"schema_version\":\"portable_proof_inputs_v1\"",
                "\"schema_version\":\"portable_proof_inputs_v1\",\"schema_version\":\"portable_proof_inputs_v1\"",
            )
            .into_bytes()
        }
        "leading_whitespace" => [b" ".as_slice(), base].concat(),
        "trailing_whitespace" => [base, b" ".as_slice()].concat(),
        "unnecessary_escape" => std::str::from_utf8(base)
            .unwrap()
            .replacen(
                "\"portable_proof_inputs_v1\"",
                "\"portable_proof_inputs\\u005fv1\"",
                1,
            )
            .into_bytes(),
        other => panic!("unknown raw mutation {other}"),
    }
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

fn expected_observation(case: &Case) -> Observation {
    let accepted = case.expected_stage == "D4" && case.expected_error_family.is_none();
    if accepted {
        assert_eq!(
            case.expected_projection_sha256, case.expected_bundle_sha256,
            "{} typed projection identity",
            case.case_id
        );
    }
    Observation {
        stage: case.expected_stage.clone(),
        error_family: case.expected_error_family.clone(),
        input_sha256: case.input_sha256.clone(),
        canonical_sha256: case.expected_projection_sha256.clone(),
        canonical_byte_length: case.expected_bundle_byte_length,
        schema_type_count: case.expected_typeschema_count,
        source_count: accepted.then_some(43),
        applied_rule_count: accepted.then_some(84),
        semantic_version: accepted.then(|| "agentflow.root-export-checker@1".to_owned()),
        trust_boundary: case.trust_boundary.clone(),
    }
}

fn actual_observation(
    result: Result<af_portable::SupportedPortableStaticInputs, PortableProofError>,
    input: &[u8],
) -> Observation {
    use sha2::Digest as _;
    match result {
        Ok(supported) => {
            let bytes = supported.content().canonical_bytes();
            Observation {
                stage: "D4".to_owned(),
                error_family: None,
                input_sha256: hex::encode(sha2::Sha256::digest(input)),
                canonical_sha256: Some(hex::encode(sha2::Sha256::digest(bytes))),
                canonical_byte_length: Some(bytes.len()),
                schema_type_count: Some(supported.supported_schema_type_count()),
                source_count: Some(supported.supported_source_count()),
                applied_rule_count: Some(supported.applied_rule_count()),
                semantic_version: Some(supported.semantic_version().to_owned()),
                trust_boundary: "current-static-inputs-only".to_owned(),
            }
        }
        Err(error) => Observation {
            stage: stage_name(error.stage).to_owned(),
            error_family: Some(error.family.to_owned()),
            input_sha256: hex::encode(sha2::Sha256::digest(input)),
            canonical_sha256: None,
            canonical_byte_length: None,
            schema_type_count: None,
            source_count: None,
            applied_rule_count: None,
            semantic_version: None,
            trust_boundary: "no-capability".to_owned(),
        },
    }
}

fn mutate_comparator_projection(projection: &mut Observation, dimension: &str) {
    match dimension {
        "stage" => {
            if projection.stage == "D4" { "D3" } else { "D4" }.clone_into(&mut projection.stage);
        }
        "error_family" => projection.error_family = Some("ComparatorMutationAccepted".to_owned()),
        "canonical_bytes" => projection.canonical_sha256 = Some("0".repeat(64)),
        "trust_boundary" => "trusted-origin".clone_into(&mut projection.trust_boundary),
        other => panic!("unregistered comparator mutation dimension {other}"),
    }
}

fn compare_observation(
    result: Result<af_portable::SupportedPortableStaticInputs, PortableProofError>,
    input: &[u8],
    case: &Case,
) -> Observation {
    let actual = actual_observation(result, input);
    assert_eq!(
        actual,
        expected_observation(case),
        "full production projection for {}",
        case.case_id
    );
    actual
}

fn run_group(group: &str, expected_count: usize) {
    use sha2::Digest as _;

    let (manifest, base) = corpus();
    assert_eq!(manifest.case_count, manifest.cases.len());
    assert!(manifest.case_count >= 48);
    assert_eq!(
        hex::encode(sha2::Sha256::digest(&base)),
        manifest.base_sha256
    );
    assert!(String::from_utf8_lossy(&base).starts_with('{'));
    assert_eq!(manifest.required_bundle_fields.len(), 10);
    let comparator_manifest: ComparatorManifest = serde_json::from_slice(
        &std::fs::read(
            repo().join("spec/conformance/portable-proof-codec/comparator-mutations.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        comparator_manifest.mutation_count,
        comparator_manifest.mutations.len()
    );
    let codec = PortableProofCodec::default();
    let mut executed = 0usize;
    let mut compared = 0usize;
    for case in manifest.cases.iter().filter(|case| case.group == group) {
        let input = match case.raw_mutation.as_deref() {
            Some(recipe) => raw_mutation(&base, recipe),
            None => apply_json_mutations(&base, &case.mutations, case.raw_noncanonical),
        };
        assert_eq!(
            input.len(),
            case.input_byte_length,
            "{} materialized length",
            case.case_id
        );
        assert_eq!(
            hex::encode(sha2::Sha256::digest(&input)),
            case.input_sha256,
            "{} independently expected bytes",
            case.case_id
        );
        assert!(
            !case.required_semantic_field_paths.is_empty(),
            "{}",
            case.case_id
        );
        let test_codec = match &case.decoder_limits {
            Some(limits) => {
                let defaults = DecoderLimits::default();
                PortableProofCodec::new(DecoderLimits {
                    max_container_items: limits
                        .max_container_items
                        .unwrap_or(defaults.max_container_items),
                    max_graph_work: limits.max_graph_work.unwrap_or(defaults.max_graph_work),
                    ..defaults
                })
            }
            None => codec,
        };
        let result = test_codec.validate_supported(&input);
        let actual = match result {
            Ok(supported) => {
                assert_eq!(
                    supported.content().canonical_bytes(),
                    input,
                    "{}",
                    case.case_id
                );
                for field in &case.required_semantic_field_paths {
                    let name = field.rsplit('.').next().unwrap();
                    assert!(
                        input
                            .windows(name.len() + 3)
                            .any(|window| window == format!("\"{name}\":").as_bytes()),
                        "{} required field {field} absent from canonical bytes",
                        case.case_id
                    );
                }
                compare_observation(Ok(supported), &input, case)
            }
            Err(error) => {
                assert!(
                    case.expected_stage != "D4" || case.expected_error_family.is_some(),
                    "{} unexpectedly rejected: {error}",
                    case.case_id
                );
                assert_eq!(
                    stage_name(error.stage),
                    case.expected_stage,
                    "{}: {error}",
                    case.case_id
                );
                assert_eq!(
                    error.family,
                    case.expected_error_family.as_deref().unwrap(),
                    "{}",
                    case.case_id
                );
                compare_observation(Err(error), &input, case)
            }
        };
        let expected = expected_observation(case);
        for mutation in comparator_manifest
            .mutations
            .iter()
            .filter(|mutation| mutation.witness_case_id == case.case_id)
        {
            assert_eq!(
                mutation.expected_stage, case.expected_stage,
                "{} witness stage",
                mutation.mutation_id
            );
            assert_eq!(
                mutation.expected_error_family, case.expected_error_family,
                "{} witness family",
                mutation.mutation_id
            );
            let mut corrupted_projection = expected.clone();
            mutate_comparator_projection(&mut corrupted_projection, &mutation.dimension);
            assert_ne!(
                actual, corrupted_projection,
                "comparator mutant survived: {}",
                mutation.mutation_id
            );
            compared += 1;
        }
        executed += 1;
    }
    assert_eq!(executed, expected_count, "executed cases for {group}");
    let expected_mutations = comparator_manifest
        .mutations
        .iter()
        .filter(|mutation| {
            manifest
                .cases
                .iter()
                .any(|case| case.group == group && case.case_id == mutation.witness_case_id)
        })
        .count();
    assert_eq!(
        compared, expected_mutations,
        "comparator mutants killed for {group}"
    );
}

#[test]
fn corpus_partition_has_unique_ids_and_no_unexecuted_groups() {
    let (manifest, _) = corpus();
    let expected = [
        ("ace1-shape", 13),
        ("canonical-order", 8),
        ("references", 15),
        ("content", 9),
        ("semantics", 10),
        ("roundtrip", 1),
        ("positive-roundtrip", 4),
    ];
    let mut ids = std::collections::BTreeSet::new();
    for case in &manifest.cases {
        assert!(
            ids.insert(&case.case_id),
            "duplicate CaseId {}",
            case.case_id
        );
        assert!(
            !case.required_semantic_field_paths.is_empty(),
            "{}",
            case.case_id
        );
        assert!(
            expected.iter().any(|(group, _)| *group == case.group),
            "unexecuted group {}",
            case.group
        );
    }
    for (group, count) in expected {
        assert_eq!(
            manifest
                .cases
                .iter()
                .filter(|case| case.group == group)
                .count(),
            count,
            "group {group}"
        );
    }
    assert_eq!(ids.len(), manifest.case_count);
    assert_eq!(
        expected.iter().map(|(_, count)| count).sum::<usize>(),
        manifest.case_count
    );

    let comparator: ComparatorManifest = serde_json::from_slice(
        &std::fs::read(
            repo().join("spec/conformance/portable-proof-codec/comparator-mutations.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(comparator.mutation_count, 17);
    assert_eq!(comparator.mutation_count, comparator.mutations.len());
    let mut mutation_ids = std::collections::BTreeSet::new();
    for mutation in &comparator.mutations {
        assert!(
            mutation_ids.insert(&mutation.mutation_id),
            "duplicate comparator mutation {}",
            mutation.mutation_id
        );
        let witness = manifest
            .cases
            .iter()
            .find(|case| case.case_id == mutation.witness_case_id)
            .unwrap_or_else(|| {
                panic!(
                    "unregistered comparator witness {}",
                    mutation.witness_case_id
                )
            });
        assert_eq!(
            witness.expected_stage, mutation.expected_stage,
            "{}",
            mutation.mutation_id
        );
        assert_eq!(
            witness.expected_error_family, mutation.expected_error_family,
            "{}",
            mutation.mutation_id
        );
    }
}

#[test]
fn python_oracle_ace1_shape_cases() {
    run_group("ace1-shape", 13);
}

#[test]
fn python_oracle_canonical_order_cases() {
    run_group("canonical-order", 8);
}

#[test]
fn python_oracle_reference_cases() {
    run_group("references", 15);
}

#[test]
fn python_oracle_content_cases() {
    run_group("content", 9);
}

#[test]
fn python_oracle_semantics_cases() {
    run_group("semantics", 10);
}

#[test]
fn python_oracle_positive_cases() {
    run_group("roundtrip", 1);
    run_group("positive-roundtrip", 4);
}

#[test]
fn independent_base_bundle_roundtrips_through_public_encoder() {
    let (manifest, bytes) = corpus();
    let codec = PortableProofCodec::default();
    let decoded = codec.decode_d0(&bytes).expect("D0 base");
    let canonical = codec.validate_d1(decoded, &bytes).expect("D1 base");
    let encoded = codec.encode(canonical.value()).expect("public encoder");
    assert_eq!(encoded, bytes);
    assert!(manifest.case_count >= 48);
}

#[test]
fn resource_bounds_fail_without_partial_output() {
    let (_manifest, bytes) = corpus();
    let limits = DecoderLimits {
        max_container_items: 8,
        ..DecoderLimits::default()
    };
    let codec = PortableProofCodec::new(limits);
    let error = codec
        .decode_d0(&bytes)
        .expect_err("large complete asset inventory exceeds this local bound");
    assert_eq!(error.family, "ResourceLimit");
    assert_eq!(error.stage, ValidationStage::D0);
}
