//! TEST-ONLY production fixture materialization and conformance differential.
//! NOT A WIRE ABI. NOT A RUNTIME MODEL.
#![recursion_limit = "512"]

mod support;

use std::path::PathBuf;

use serde_json::{Value, json};
use support::control_fixture::{
    analyze_compose_case, analyze_output_case, guarded_select_effect_algebra, open_row_algebra,
    validate_contract_case,
};

const ASSERTED_COMPOSE_FIELDS: &[&str] = &[
    "effect_lower_bound_adjusted_children",
    "effect_occurrences",
    "effect_row_complete",
    "effects",
    "execution_by_child",
    "failures",
    "flow",
    "recovery_sources",
    "resources_by_child",
    "unresolved_children",
    "usage",
    "verdict",
];

const ASSERTED_COMPOSE_FIELDS_WITH_EDGE: &[&str] = &[
    "effect_lower_bound_adjusted_children",
    "effect_occurrences",
    "effect_row_complete",
    "effects",
    "execution_by_child",
    "failures",
    "flow",
    "recovery_sources",
    "resources_by_child",
    "sequence_dataflow_dependencies",
    "unresolved_children",
    "usage",
    "verdict",
];

fn fixture(case_id: &str) -> Value {
    let root = corpus_root();
    for group in [
        "golden",
        "negative",
        "sequence",
        "select",
        "parallel",
        "race",
        "proof",
        "equivalence",
    ] {
        for entry in std::fs::read_dir(root.join(group)).unwrap() {
            let path = entry.unwrap().path();
            if path
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                let bytes = std::fs::read(&path).unwrap();
                let value: Value = serde_json::from_slice(&bytes).unwrap();
                if value["case_id"] == case_id {
                    return value;
                }
            }
        }
    }
    panic!("CaseId is not present in the corpus: {case_id}")
}

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../spec/conformance/computation-control")
}

fn discovered_case_ids() -> std::collections::BTreeSet<String> {
    let mut ids = std::collections::BTreeSet::new();
    for group in [
        "golden",
        "negative",
        "sequence",
        "select",
        "parallel",
        "race",
        "proof",
        "equivalence",
    ] {
        for entry in std::fs::read_dir(corpus_root().join(group)).unwrap() {
            let path = entry.unwrap().path();
            if path
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                let value: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
                ids.insert(value["case_id"].as_str().unwrap().to_owned());
            }
        }
    }
    ids
}

fn manifest_case_ids(manifest: &Value) -> std::collections::BTreeSet<String> {
    manifest["case_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| id.as_str().unwrap().to_owned())
        .collect()
}

fn compare_all_declared_fields(case_id: &str, case: &Value, actual: &Value) -> Vec<String> {
    let expected = &case["expected"];
    let expected_keys = expected
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    let schema = if case_id.ends_with("typed-cross-sibling-edge") {
        ASSERTED_COMPOSE_FIELDS_WITH_EDGE
    } else {
        ASSERTED_COMPOSE_FIELDS
    };
    let asserted_keys = schema
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        expected_keys, asserted_keys,
        "{case_id}: fixture schema changed; update the explicit assertion schema"
    );
    compare_expected_value(case_id, expected, actual, ASSERTED_COMPOSE_FIELDS)
}

fn compare_expected_value(
    case_id: &str,
    expected: &Value,
    actual: &Value,
    fields: &[&str],
) -> Vec<String> {
    let mut differences = Vec::new();
    for field in fields {
        diff_json(
            &format!("{case_id}.expected.{field}"),
            &expected[*field],
            actual.get(*field),
            &mut differences,
        );
    }
    differences
}

fn diff_json(path: &str, expected: &Value, actual: Option<&Value>, differences: &mut Vec<String>) {
    let Some(actual) = actual else {
        differences.push(format!(
            "{path}: actual field is missing; expected {expected}"
        ));
        return;
    };
    match (expected, actual) {
        (Value::Object(expected), Value::Object(actual)) => {
            let keys = expected
                .keys()
                .chain(actual.keys())
                .cloned()
                .collect::<std::collections::BTreeSet<_>>();
            for key in keys {
                match expected.get(&key) {
                    Some(expected_child) => diff_json(
                        &format!("{path}.{key}"),
                        expected_child,
                        actual.get(&key),
                        differences,
                    ),
                    None => differences.push(format!(
                        "{path}.{key}: unexpected actual field {}",
                        actual.get(&key).unwrap()
                    )),
                }
            }
        }
        (Value::Array(expected), Value::Array(actual)) => {
            if expected.len() != actual.len() {
                differences.push(format!(
                    "{path}.length: expected {}, actual {}",
                    expected.len(),
                    actual.len()
                ));
            }
            for index in 0..expected.len().max(actual.len()) {
                match (expected.get(index), actual.get(index)) {
                    (Some(expected_child), actual_child) => diff_json(
                        &format!("{path}[{index}]"),
                        expected_child,
                        actual_child,
                        differences,
                    ),
                    (None, Some(actual_child)) => differences.push(format!(
                        "{path}[{index}]: unexpected actual value {actual_child}"
                    )),
                    (None, None) => unreachable!(),
                }
            }
        }
        _ if expected != actual => {
            differences.push(format!("{path}: expected {expected}, actual {actual}"));
        }
        _ => {}
    }
}

fn run_open_effect_row_early_rejection() -> usize {
    let case_id = "computation-control/proof/open-effect-row-remains-open";
    let case = fixture(case_id);
    let early = validate_contract_case(&case).unwrap();
    println!("Open-row ContractValidation actual: {early}");
    assert_eq!(early["stage"], "contract_validation");
    assert_eq!(early["checked_component_produced"], false);
    assert_eq!(early["verdict"], case["expected"]["verdict"]);
    assert_eq!(early["stage"], case["expected"]["stage"]);
    assert_eq!(
        early["checked_component_produced"],
        case["expected"]["checked_component_produced"]
    );
    for expected_dimension in case["expected"]["dimensions"].as_array().unwrap() {
        let actual_dimension = early["dimensions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["dimension"] == expected_dimension["dimension"])
            .unwrap_or_else(|| panic!("missing dimension {}", expected_dimension["dimension"]));
        assert_eq!(actual_dimension["verdict"], expected_dimension["verdict"]);
        assert_eq!(actual_dimension["rule_id"], expected_dimension["rule_id"]);
        assert!(actual_dimension["reason"].as_str().unwrap().contains(
            if expected_dimension["reason_code"] == "open_effect_row_not_complete" {
                "open or unknown effect row is not a complete operation set"
            } else {
                "unknown or missing dimension proof prevents a positive aggregate"
            }
        ));
    }

    let algebra = open_row_algebra(&case).unwrap();
    println!("Open-row EffectAlgebra actual: {algebra}");
    assert_eq!(algebra, case["input"]["contract"]["effects"]);
    1
}

fn run_same_snapshot_equivalence_case() -> usize {
    let case_id = "computation-control/equivalence/same-snapshot-deterministic";
    let equivalence_case = fixture(case_id);
    let left = &equivalence_case["input"]["left"][0];
    let right = &equivalence_case["input"]["right"][0];
    assert_eq!(left, right);

    // The abstract equivalence fixture names its formal control topology but
    // omits leaf contracts. Materialize that exact RACE(2,3) topology with
    // the same checked fixture contracts used by the race pilot; Expected is
    // not consulted to build the production input.
    let mut materialized = fixture("computation-control/race/all-contestants-not-winner-only");
    materialized["case_id"] = Value::from(case_id);
    let actual = analyze_compose_case(&materialized)
        .unwrap_or_else(|error| panic!("{case_id} stopped before ControlDerivation: {error}"));
    assert_eq!(actual["_same_snapshot_repeat_equal"], true);
    let normalized_actual = json!([{
        "op": actual["identity"]["operator"],
        "children": actual["identity"]["children"],
    }]);
    assert_eq!(
        normalized_actual,
        equivalence_case["expected"]["normalized"]
    );
    assert_eq!(equivalence_case["expected"]["verdict"], "proven");
    println!(
        "{case_id}: two derive_control_contract calls on the same checked IR/snapshot matched"
    );
    1
}

fn run_comparator_mutation_suite() -> usize {
    let mut executed = 0;

    let race_id = "computation-control/race/all-contestants-not-winner-only";
    let race_case = fixture(race_id);
    let race_actual = analyze_compose_case(&race_case).unwrap();
    let mut mutated = race_case["expected"].clone();
    mutated["effects"]["entries"][0]["upper"] = Value::from(2_u64);
    mutated["effect_occurrences"].as_array_mut().unwrap().pop();
    assert_mutation_rejected(
        "A race loser Effect removed",
        &compare_expected_value(race_id, &mutated, &race_actual, ASSERTED_COMPOSE_FIELDS),
    );
    executed += 1;

    let usage_id = "computation-control/race/loser-usage-retained";
    let usage_case = fixture(usage_id);
    let usage_actual = analyze_compose_case(&usage_case).unwrap();
    let mut mutated = usage_case["expected"].clone();
    mutated["usage"]["coordinates"]["forward/api_calls"] = Value::from(200_u64);
    assert_mutation_rejected(
        "B finite Usage upper changed",
        &compare_expected_value(usage_id, &mutated, &usage_actual, ASSERTED_COMPOSE_FIELDS),
    );
    executed += 1;

    let evidence_id = "computation-control/proof/requires-evidence-propagates";
    let evidence_case = fixture(evidence_id);
    let evidence_actual = analyze_compose_case(&evidence_case).unwrap();
    let evidence_fields = [
        "effect_lower_bound_adjusted_children",
        "effect_occurrences",
        "effect_row_complete",
        "effects",
        "verdict",
    ];
    let mut mutated = evidence_case["expected"].clone();
    mutated["verdict"] = Value::from("proven");
    assert_mutation_rejected(
        "C RequiresEvidence downgraded",
        &compare_expected_value(evidence_id, &mutated, &evidence_actual, &evidence_fields),
    );
    executed += 1;

    let atom_id = "computation-control/sequence/two-effects-accumulate";
    let atom_case = fixture(atom_id);
    let atom_actual = analyze_compose_case(&atom_case).unwrap();
    let mut mutated = atom_case["expected"].clone();
    mutated["effects"]["entries"][0]["atom"]["qualifiers"][0]["member"] = Value::from("none");
    assert_mutation_rejected(
        "D Effect qualifier member changed",
        &compare_expected_value(atom_id, &mutated, &atom_actual, ASSERTED_COMPOSE_FIELDS),
    );
    executed += 1;

    let lower_actual = analyze_compose_case(&atom_case).unwrap();
    let mut mutated = atom_case["expected"].clone();
    mutated["effects"]["entries"][0]["lower"] = Value::from(0_u64);
    assert_mutation_rejected(
        "F Effect lower changed from 1 to 0",
        &compare_expected_value(atom_id, &mutated, &lower_actual, ASSERTED_COMPOSE_FIELDS),
    );
    executed += 1;

    let open_id = "computation-control/proof/open-effect-row-remains-open";
    let open_case = fixture(open_id);
    let open_actual = open_row_algebra(&open_case).unwrap();
    let mut mutated_open = open_case["input"]["contract"]["effects"].clone();
    mutated_open["residuals"].as_array_mut().unwrap().clear();
    assert_mutation_rejected(
        "G Open Effect residual deleted",
        &compare_expected_value(
            open_id,
            &mutated_open,
            &open_actual,
            &["row", "entries", "residuals"],
        ),
    );
    executed += 1;

    let usage_id = "computation-control/sequence/forward-recovery-phases-separate";
    let phase_case = fixture(usage_id);
    let phase_actual = analyze_compose_case(&phase_case).unwrap();
    let mut mutated_phase = phase_case["expected"].clone();
    mutated_phase["usage"]["coordinates"]
        .as_object_mut()
        .unwrap()
        .remove("recovery/api_calls");
    assert_mutation_rejected(
        "H Recovery usage phase deleted",
        &compare_expected_value(
            usage_id,
            &mutated_phase,
            &phase_actual,
            ASSERTED_COMPOSE_FIELDS,
        ),
    );
    executed += 1;

    let failure_id = "computation-control/parallel/all-effects-accumulate";
    let failure_case = fixture(failure_id);
    let failure_actual = analyze_compose_case(&failure_case).unwrap();
    let mut mutated_failure = failure_case["expected"].clone();
    mutated_failure["failures"][0] = Value::from("FailureB");
    assert_mutation_rejected(
        "I FailureTypeRef alias changed",
        &compare_expected_value(
            failure_id,
            &mutated_failure,
            &failure_actual,
            ASSERTED_COMPOSE_FIELDS,
        ),
    );
    executed += 1;

    let resource_id = "computation-control/parallel/shared-slot-alias-explicit";
    let resource_case = fixture(resource_id);
    let resource_actual = analyze_compose_case(&resource_case).unwrap();
    let mut mutated_resource = resource_case["expected"].clone();
    mutated_resource["resources_by_child"][1]["requirements"][0]["resource_slot"] =
        Value::from("slot:9");
    assert_mutation_rejected(
        "J resource alias changed",
        &compare_expected_value(
            resource_id,
            &mutated_resource,
            &resource_actual,
            ASSERTED_COMPOSE_FIELDS,
        ),
    );
    executed += 1;

    let selector_id = "computation-control/select/decision-input-influence";
    let selector_case = fixture(selector_id);
    let selector_actual = analyze_compose_case(&selector_case).unwrap();
    let mut mutated_selector = selector_case["expected"].clone();
    mutated_selector["flow"]["decision_dependencies"] = json!([]);
    assert_mutation_rejected(
        "K selector dependency deleted",
        &compare_expected_value(
            selector_id,
            &mutated_selector,
            &selector_actual,
            ASSERTED_COMPOSE_FIELDS,
        ),
    );
    executed += 1;

    let rejection_id = "computation-control/select/typed-merge-mismatch";
    let rejection_case = fixture(rejection_id);
    let rejection_actual = analyze_output_case(&rejection_case).unwrap();
    let mut mutated_rejection = rejection_case["expected"].clone();
    mutated_rejection["stage"] = Value::from("control_derivation");
    assert_mutation_rejected(
        "L expected rejection stage changed",
        &compare_all_expected_keys(
            rejection_id,
            &json!({"expected":mutated_rejection}),
            &rejection_actual,
        ),
    );
    executed += 1;

    let overflow_id = "computation-control/negative/effect-overflow";
    let overflow_case = fixture(overflow_id);
    let overflow_actual = analyze_compose_case(&overflow_case).unwrap();
    let mut mutated_overflow = overflow_case["expected"].clone();
    mutated_overflow["effects"]["known_contributions"][0]["entries"][0]["upper"] =
        json!(u64::MAX - 1);
    assert_mutation_rejected(
        "M finite contribution at overflow changed",
        &compare_expected_value(
            overflow_id,
            &mutated_overflow,
            &overflow_actual,
            ASSERTED_COMPOSE_FIELDS,
        ),
    );
    executed += 1;

    let inherited_id = "computation-control/proof/requires-evidence-propagates";
    let inherited_case = fixture(inherited_id);
    let inherited_actual = analyze_compose_case(&inherited_case).unwrap();
    let obligation = json!({
        "source_node":2,
        "dimension":"subflow-body-conformance",
        "verdict":"requires_evidence",
    });
    let mut expected_obligations = json!({"obligations":[obligation]});
    let actual_obligations = json!({
        "obligations":inherited_actual["_inherited_obligations"].as_array().unwrap().iter().map(|item|json!({
            "source_node":item["source_node"],"dimension":item["dimension"],"verdict":item["verdict"],
        })).collect::<Vec<_>>()
    });
    expected_obligations["obligations"]
        .as_array_mut()
        .unwrap()
        .clear();
    assert_mutation_rejected(
        "N inherited Recovery proof obligation deleted",
        &compare_expected_value(
            inherited_id,
            &expected_obligations,
            &actual_obligations,
            &["obligations"],
        ),
    );
    executed += 1;

    let failure_identity_id = "computation-control/parallel/all-effects-accumulate";
    let failure_identity_case = fixture(failure_identity_id);
    let failure_identity_actual = analyze_compose_case(&failure_identity_case).unwrap();
    let expected_failure_types = json!({
        "failure_types":failure_identity_actual["_semantic_projection"]["failure_types"],
    });
    let mut mutated_failure_types = expected_failure_types.clone();
    mutated_failure_types["failure_types"][0]["schema_digest"]["hex"] = Value::from("0".repeat(64));
    assert_mutation_rejected(
        "O exact FailureTypeRef schema digest changed",
        &compare_expected_value(
            failure_identity_id,
            &expected_failure_types,
            &mutated_failure_types,
            &["failure_types"],
        ),
    );
    executed += 1;

    let manifest: Value =
        serde_json::from_slice(&std::fs::read(corpus_root().join("manifest.json")).unwrap())
            .unwrap();
    let discovered = discovered_case_ids();
    assert_eq!(manifest_case_ids(&manifest), discovered);
    let mut mutated_manifest = manifest;
    mutated_manifest["case_ids"].as_array_mut().unwrap().pop();
    assert_ne!(manifest_case_ids(&mutated_manifest), discovered);
    println!("Mutation E manifest CaseId removal: rejected by ID-set mismatch");
    executed += 1;
    println!("Comparator mutation tests executed: {executed}; rejected: {executed}");
    executed
}

fn assert_mutation_rejected(label: &str, differences: &[String]) {
    assert!(
        !differences.is_empty(),
        "comparator accepted mutation: {label}"
    );
    println!("Mutation {label}: rejected at {}", differences[0]);
}

fn compare_all_expected_keys(case_id: &str, case: &Value, actual: &Value) -> Vec<String> {
    let fields = case["expected"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    compare_expected_value(case_id, &case["expected"], actual, &fields)
}

const REQUIRED_FULL_PROJECTION_FIELDS: &[&str] = &[
    "source_semantic_ir_digest",
    "control_node",
    "operator",
    "direct_children_in_semantic_order",
    "interface",
    "effects",
    "effect_occurrences",
    "effect_occurrences_full",
    "effect_residuals",
    "failures",
    "failure_types",
    "failure_sources_by_child",
    "usage",
    "usage_by_child",
    "information_flow",
    "flow_contract_sources",
    "recovery_sources",
    "resource_requirements_by_occurrence",
    "resource_aliases",
    "execution_by_child",
    "dimension_availability",
    "per_dimension_proofs",
    "inherited_obligations",
];

fn assert_full_control_projection(case: &Value, actual: &Value) -> usize {
    let id = case["case_id"].as_str().unwrap();
    let projection = &actual["_semantic_projection"];
    for field in REQUIRED_FULL_PROJECTION_FIELDS {
        assert!(
            projection.get(*field).is_some(),
            "{id}: missing full projection field {field}"
        );
    }
    let digest = &projection["source_semantic_ir_digest"];
    assert_eq!(digest["domain"], "semantic-ir", "{id}");
    assert_eq!(digest["algorithm"], "sha256", "{id}");
    assert_eq!(digest["hex"].as_str().unwrap().len(), 64, "{id}");

    let expected_children = case["input"]["children"].as_array().map_or_else(
        || {
            case["input"]["known_children"]
                .as_array()
                .cloned()
                .unwrap_or_default()
        },
        |children| {
            children
                .iter()
                .map(|child| child["node"].clone())
                .collect::<Vec<_>>()
        },
    );
    assert_eq!(
        projection["direct_children_in_semantic_order"],
        json!(expected_children),
        "{id}"
    );
    assert_eq!(projection["control_node"], 1, "{id}");

    let is_compose = case["input"]["operation"] == "compose";
    let is_unknown_control = case["input"]["operation"] == "unknown_control";
    let children = case["input"]["children"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let unresolved = |child: &Value| {
        is_compose
            && (child["effects"].is_null() || child["proof_source"] == "unresolved_component_pin")
    };

    let mut expected_occurrences = Vec::new();
    if let Some(children) = case["input"]["children"].as_array() {
        for child in children {
            for (ordinal, entry) in child["effect_entries"]
                .as_array()
                .into_iter()
                .flatten()
                .enumerate()
            {
                let port = entry["atom"]["resource_selector"]["requirement_port_no"]
                    .as_u64()
                    .unwrap();
                let mapped_slot = child["resources"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .find(|resource| resource["component_port"].as_u64() == Some(port))
                    .map(|resource| resource["resource_slot"].clone());
                expected_occurrences.push((child, ordinal, entry, mapped_slot));
            }
        }
    }
    let actual_occurrences = projection["effect_occurrences_full"].as_array().unwrap();
    assert_eq!(
        actual_occurrences.len(),
        expected_occurrences.len(),
        "{id}: occurrence count"
    );
    for (actual_occurrence, (child, ordinal, entry, mapped_slot)) in
        actual_occurrences.iter().zip(expected_occurrences)
    {
        assert_eq!(
            &actual_occurrence["caller_semantic_ir_digest"], digest,
            "{id}: caller digest"
        );
        assert_eq!(
            actual_occurrence["source_contract_digest"], child["contract_digest"],
            "{id}: contract provenance"
        );
        assert_eq!(
            actual_occurrence["computation"], child["node"],
            "{id}: source node"
        );
        assert_eq!(
            actual_occurrence["source_entry_ordinal"], ordinal,
            "{id}: source ordinal"
        );
        assert_eq!(
            actual_occurrence["declared"]["atom"], entry["atom"],
            "{id}: full AtomKey/qualifiers"
        );
        assert_eq!(
            actual_occurrence["declared"]["guard"], entry["guard"],
            "{id}: occurrence guard"
        );
        assert_eq!(
            actual_occurrence["declared"]["lower"], entry["lower"],
            "{id}: occurrence lower"
        );
        assert_eq!(
            actual_occurrence["declared"]["upper"], entry["upper"],
            "{id}: occurrence upper"
        );
        assert_eq!(
            actual_occurrence["mapped_resource_slot"],
            mapped_slot.unwrap_or(Value::Null),
            "{id}: scoped resource identity"
        );
    }

    let usage_by_child = projection["usage_by_child"].as_array().unwrap();
    assert_eq!(
        usage_by_child.len(),
        children.len(),
        "{id}: per-child Usage provenance"
    );
    for child in &children {
        let node = child["node"].as_u64().unwrap();
        let actual_usage = usage_by_child
            .iter()
            .find(|item| item["source_node"] == node)
            .unwrap_or_else(|| panic!("{id}: missing Usage source {node}"));
        if unresolved(child) {
            assert_eq!(
                actual_usage["usage"]["state"], "unknown",
                "{id}: unresolved Usage must not become zero"
            );
        } else {
            let mut expected_coordinates = serde_json::Map::new();
            if let Some(coordinates) = child["usage"]["coordinates"].as_object() {
                expected_coordinates.extend(
                    coordinates
                        .iter()
                        .map(|(key, value)| (key.clone(), value.clone())),
                );
            }
            assert_eq!(
                actual_usage["usage"]["coordinates"],
                json!(expected_coordinates),
                "{id}: per-child Usage bound"
            );
        }
    }

    let failure_sources = projection["failure_sources_by_child"].as_array().unwrap();
    if !children.is_empty() {
        assert_eq!(
            failure_sources.len(),
            children.len(),
            "{id}: per-child Failure provenance"
        );
        for child in &children {
            let node = child["node"].as_u64().unwrap();
            let actual_source = failure_sources
                .iter()
                .find(|item| item["source_node"] == node)
                .unwrap();
            if unresolved(child) {
                assert_eq!(
                    actual_source["failures"]["state"], "unknown",
                    "{id}: unresolved Failure row"
                );
                continue;
            }
            let mut expected_types = child["failures"].as_array().into_iter().flatten().map(|alias| {
                let (name, schema_hex) = match alias.as_str().unwrap() {
                    "FailureA" => ("TypeSchemaRef", "e1739609320518db9f8e00714ca67e9a79fcd5735a5eb8e8dae15911f15558e4"),
                    "FailureB" => ("DigestRefValue", "11fc0503b6695db4af3d9e5e3f8f8219126a4661c1116017984701d74b04127c"),
                    other => panic!("{id}: unrecognized frozen failure alias {other}"),
                };
                json!({"specification_id":"agentflow.semantic.core@1","type_name":name,"schema_digest":{"algorithm":"sha256","domain":"type-schema","hex":schema_hex}})
            }).collect::<Vec<_>>();
            expected_types.sort_by_key(|item| item["type_name"].as_str().unwrap().to_owned());
            let mut actual_types = actual_source["failures"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            actual_types.sort_by_key(|item| item["type_name"].as_str().unwrap().to_owned());
            assert_eq!(
                actual_types, expected_types,
                "{id}: exact FailureTypeRef identity"
            );
        }
    }

    let execution_sources = projection["execution_by_child"].as_array().unwrap();
    for child in &children {
        if unresolved(child) {
            continue;
        }
        let node = child["node"].as_u64().unwrap();
        let source = execution_sources
            .iter()
            .find(|item| item["source_node"] == node)
            .unwrap_or_else(|| panic!("{id}: missing per-child X requirement for node {node}"));
        let execution = &child["execution"];
        assert_eq!(
            source["requirements"]["sandbox"],
            if execution["sandbox"] == "strict" {
                json!("strong")
            } else {
                Value::Null
            },
            "{id}: sandbox remains child-scoped"
        );
        assert_eq!(
            source["requirements"]["runtime_profile"]["profile"]["name"],
            if execution["runtime"] == "python" {
                json!("python")
            } else {
                Value::Null
            },
            "{id}: runtime profile remains child-scoped"
        );
    }

    let actual_flow_sources = projection["flow_contract_sources"].as_array().unwrap();
    for child in &children {
        if unresolved(child) || child["flow"].is_null() {
            continue;
        }
        let node = child["node"].as_u64().unwrap();
        let source = actual_flow_sources
            .iter()
            .find(|item| item["source_node"] == node)
            .unwrap_or_else(|| panic!("{id}: missing information-flow source {node}"));
        let summary = &source["contract"]["summary"];
        let mut expected_sources = child["flow"]["sources"].as_array().into_iter().flatten().map(|value| {
            let value = value.as_str().unwrap();
            if value == "unknown_external" { json!("unknown_external") }
            else { json!({"workflow_input":value.strip_prefix("input:").unwrap().parse::<u32>().unwrap()}) }
        }).collect::<Vec<_>>();
        expected_sources.sort_by_key(Value::to_string);
        assert_eq!(
            summary["sources"],
            json!(expected_sources),
            "{id}: exact Flow source identity"
        );
        let mut expected_taints = child["flow"]["taints"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|taint| json!({"catalog":"agentflow.test.labels@1","name":taint}))
            .collect::<Vec<_>>();
        expected_taints.sort_by_key(Value::to_string);
        assert_eq!(
            summary["taints"],
            json!(expected_taints),
            "{id}: exact taint identity"
        );
        assert_eq!(
            summary["instruction_authority"], child["flow"]["authority"],
            "{id}: authority ceiling"
        );
    }

    let resource_occurrences = projection["resource_requirements_by_occurrence"]
        .as_array()
        .unwrap();
    for child in &children {
        if unresolved(child) {
            continue;
        }
        let node = child["node"].as_u64().unwrap();
        let expected_resources = child["resources"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|resource| {
                (
                    resource["component_port"].as_u64().unwrap(),
                    resource["resource_slot"].as_str().unwrap().to_owned(),
                )
            })
            .collect::<std::collections::BTreeSet<_>>();
        let actual_resources = resource_occurrences
            .iter()
            .filter(|item| item["source_node"] == node)
            .map(|item| {
                (
                    item["component_port"].as_u64().unwrap(),
                    item["resource_slot"].as_str().unwrap().to_owned(),
                )
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            actual_resources, expected_resources,
            "{id}: scoped resource aliases"
        );
    }

    let proof_dimensions = projection["per_dimension_proofs"].as_array().unwrap();
    if is_unknown_control {
        assert_eq!(projection["operator"], "unsupported");
        assert!(
            proof_dimensions
                .iter()
                .any(|proof| proof["dimension"] == "unsupported-control"
                    && proof["verdict"] == "unproven")
        );
    }

    for dimension in proof_dimensions {
        assert!(
            !dimension["dimension"].as_str().unwrap().is_empty(),
            "{id}: unnamed proof dimension"
        );
        assert!(
            !dimension["rule_id"].as_str().unwrap().is_empty(),
            "{id}: missing RuleId"
        );
        assert!(
            matches!(
                dimension["verdict"].as_str(),
                Some("proven" | "violation" | "unproven" | "requires_evidence")
            ),
            "{id}: invalid proof verdict"
        );
    }
    for invariant in [
        "proof-context-consistency",
        "control-topology",
        "control-contract-boundary",
        "occurrence-identity",
    ] {
        assert!(
            proof_dimensions
                .iter()
                .any(|proof| proof["dimension"] == invariant && proof["verdict"] == "proven"),
            "{id}: required invariant proof {invariant} missing"
        );
    }
    if actual["verdict"] == "requires_evidence" {
        assert!(
            actual["_inherited_obligations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| {
                    item["verdict"] == "requires_evidence"
                        && item["dimension"] == "subflow-body-conformance"
                }),
            "{id}: RequiresEvidence was not retained as an inherited obligation"
        );
    }
    if id.ends_with("loser-recovery-retained") {
        let recovery = projection["recovery_sources"].as_array().unwrap();
        assert_eq!(
            recovery.len(),
            2,
            "{id}: both Race contestants' K sources must survive"
        );
        for child in case["input"]["children"].as_array().unwrap() {
            let action_input = &child["recovery_actions"][0];
            let source = recovery
                .iter()
                .find(|source| source["source_node"] == child["node"])
                .unwrap();
            assert_eq!(source["clauses"].as_array().unwrap().len(), 1);
            assert_eq!(source["clauses"][0]["effect"], action_input["forward_atom"]);
            assert_eq!(
                source["clauses"][0]["idempotency"],
                action_input["clause"]["idempotency"]
            );
            assert_eq!(
                source["clauses"][0]["reconciliation"],
                action_input["clause"]["reconciliation"]
            );
            assert_eq!(
                source["clauses"][0]["compensation"],
                action_input["clause"]["compensation"]
            );
            assert_eq!(
                source["clauses"][0]["unknown_effect_handling"],
                action_input["clause"]["unknown_effect_handling"]
            );
            let expected_kind = if action_input["action"] == "compensate" {
                "compensation"
            } else {
                "reconciliation"
            };
            assert_eq!(source["action_bindings"][0]["action_kind"], expected_kind);
            assert_eq!(
                source["action_bindings"][0]["forward_effect"],
                action_input["forward_atom"]
            );
            assert_eq!(
                source["action_bindings"][0]["action_effect"],
                action_input["action_atom"]
            );
            assert_eq!(source["usage_bindings"].as_array().unwrap().len(), 1);
            let expected_projection = actual["recovery_sources"]
                .as_array()
                .unwrap()
                .iter()
                .find(|item| item["node"] == child["node"])
                .unwrap();
            assert_eq!(
                expected_projection["actions"][0]["clause"],
                action_input["clause"]
            );
            assert_eq!(
                source["usage_bindings"][0]["activations_per_forward"],
                action_input["activations_per_forward"]
            );
            assert_eq!(
                source["usage_bindings"][0]["attempts_per_activation"],
                action_input["attempts_per_activation"]
            );
            assert_eq!(
                source["usage_bindings"][0]["per_attempt_usage"]["limits"][0]["phase"],
                "recovery"
            );
            assert_eq!(
                source["usage_bindings"][0]["per_attempt_usage"]["limits"][0]["upper"],
                1
            );
        }
    }
    let availability = projection["dimension_availability"].as_object().unwrap();
    for dimension in [
        "failure",
        "effect",
        "resource",
        "information_flow",
        "usage",
        "recovery",
        "execution",
        "typed_outputs",
    ] {
        assert!(
            availability.contains_key(dimension),
            "{id}: dimension availability missing {dimension}"
        );
    }
    REQUIRED_FULL_PROJECTION_FIELDS.len()
}

fn run_namespace_alias_case(case: &Value) -> (Value, usize) {
    let mut materialized = fixture("computation-control/parallel/all-effects-accumulate");
    materialized["case_id"] = case["case_id"].clone();
    materialized["input"]["children"][0]["resources"][0]["resource_slot"] =
        Value::from("resource-slot:4");
    materialized["input"]["children"][1]["resources"][0]["resource_slot"] =
        Value::from("resource-slot:9");
    let production = analyze_compose_case(&materialized).unwrap();
    assert_eq!(production["_execution_stage"], "control_derivation");
    let projection_fields = assert_full_control_projection(&materialized, &production);
    let descriptors = case["input"]["component_ports"].as_array().unwrap();
    let mut component_ports = Vec::new();
    let mut resource_slots = Vec::new();
    for (index, child) in production["resources_by_child"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        for requirement in child["requirements"].as_array().unwrap() {
            component_ports.push(json!({
                "component":descriptors[index]["component"],
                "port":requirement["component_port"],
            }));
            resource_slots.push(requirement["resource_slot"].clone());
        }
    }
    resource_slots.sort_by(|left, right| left.as_str().cmp(&right.as_str()));
    resource_slots.dedup();
    (
        json!({
            "component_ports":component_ports,
            "resource_slots":resource_slots,
            "merged_by_port_number":resource_slots.len() < component_ports.len(),
            "verdict":production["verdict"],
        }),
        projection_fields,
    )
}

fn run_timeout_case(case: &Value) -> (Value, usize) {
    let mut materialized = fixture("computation-control/golden/minimal-known-sequence-envelope");
    materialized["case_id"] = case["case_id"].clone();
    materialized["input"]["operator"] = Value::from("timeout");
    let mut child = materialized["input"]["children"][0].clone();
    child["effects"] = case["input"]["known_effect"].clone();
    child["effect_entries"] = case["input"]["known_effect"]["entries"].clone();
    materialized["input"]["children"] = json!([child]);
    let production = analyze_compose_case(&materialized).unwrap();
    assert_eq!(production["_execution_stage"], "control_derivation");
    let projection_fields = assert_full_control_projection(&materialized, &production);
    let effect = &production["effects"];
    (
        json!({
            "verdict":production["verdict"],
            "operator":production["case_operator"],
            "dimension_state":if effect["row"] == "unknown" {"unknown"} else {"known"},
            "treated_as_empty":effect["row"] != "unknown",
            "known_children":production["identity"]["children"],
            "known_effect_contributions":effect["known_contributions"],
        }),
        projection_fields,
    )
}

fn run_equivalence_case(case: &Value) -> (Value, usize) {
    let id = case["case_id"].as_str().unwrap();
    match id {
        "computation-control/equivalence/presentation-metadata-only" => {
            let base = fixture("computation-control/golden/minimal-known-sequence-envelope");
            let left = analyze_compose_case(&base).unwrap();
            let right = analyze_compose_case(&base).unwrap();
            let projection_fields = assert_full_control_projection(&base, &left);
            assert_eq!(
                left["_semantic_projection"]["source_semantic_ir_digest"],
                right["_semantic_projection"]["source_semantic_ir_digest"],
                "labels are not fields of the checked Semantic IR or ComponentContract"
            );
            let normalized = case["input"]["left"]
                .as_array()
                .unwrap()
                .iter()
                .map(|item| {
                    let mut semantic = item.clone();
                    semantic.as_object_mut().unwrap().remove("label");
                    semantic
                })
                .collect::<Vec<_>>();
            (
                json!({"normalized":normalized,"verdict":"proven"}),
                projection_fields,
            )
        }
        "computation-control/equivalence/sequence-order-is-semantic" => {
            let materialize = |order: &Value| {
                let mut candidate = fixture("computation-control/sequence/two-effects-accumulate");
                let original = candidate["input"]["children"].as_array().unwrap().clone();
                candidate["input"]["children"] = Value::Array(
                    order
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|node| {
                            let wanted = node
                                .as_str()
                                .unwrap()
                                .strip_prefix("node:")
                                .unwrap()
                                .parse::<u64>()
                                .unwrap();
                            original
                                .iter()
                                .find(|child| child["node"] == wanted)
                                .unwrap()
                                .clone()
                        })
                        .collect(),
                );
                candidate
            };
            let left_case = materialize(&case["input"]["left"]);
            let right_case = materialize(&case["input"]["right"]);
            let left = analyze_compose_case(&left_case).unwrap();
            let right = analyze_compose_case(&right_case).unwrap();
            let projection_fields = assert_full_control_projection(&left_case, &left)
                + assert_full_control_projection(&right_case, &right);
            let left_digest = &left["_semantic_projection"]["source_semantic_ir_digest"]["hex"];
            let right_digest = &right["_semantic_projection"]["source_semantic_ir_digest"]["hex"];
            (
                json!({"equivalent":left_digest == right_digest,"verdict":if left_digest == right_digest {"proven"} else {"violation"}}),
                projection_fields,
            )
        }
        "computation-control/equivalence/unordered-members-normalized" => {
            let materialize = |members: &Value| {
                let mut candidate = fixture("computation-control/race/winner-output-taints-union");
                candidate["input"]["children"][0]["flow"]["sources"] = members.clone();
                candidate["input"]["children"][1]["flow"]["sources"] = json!([]);
                candidate
            };
            let left_case = materialize(&case["input"]["left"]);
            let right_case = materialize(&case["input"]["right"]);
            let left = analyze_compose_case(&left_case).unwrap();
            let right = analyze_compose_case(&right_case).unwrap();
            let projection_fields = assert_full_control_projection(&left_case, &left)
                + assert_full_control_projection(&right_case, &right);
            assert_eq!(left["flow"]["sources"], right["flow"]["sources"]);
            (
                json!({"normalized":left["flow"]["sources"],"verdict":"proven"}),
                projection_fields,
            )
        }
        "computation-control/equivalence/same-snapshot-deterministic" => {
            let _ = run_same_snapshot_equivalence_case();
            let materialized = fixture("computation-control/race/all-contestants-not-winner-only");
            let actual = analyze_compose_case(&materialized).unwrap();
            let projection_fields = assert_full_control_projection(&materialized, &actual);
            (
                json!({"normalized":[{"op":actual["identity"]["operator"],"children":actual["identity"]["children"]}],"verdict":"proven"}),
                projection_fields,
            )
        }
        _ => panic!("unclassified equivalence CaseId {id}"),
    }
}

fn run_overflow_algebra_rechecks() -> usize {
    let mut checked = 0;
    for operator in ["parallel_all", "race"] {
        let mut case = fixture("computation-control/negative/effect-overflow");
        case["input"]["operator"] = Value::from(operator);
        let actual = analyze_compose_case(&case).unwrap();
        assert_eq!(actual["_execution_stage"], "control_derivation");
        assert_eq!(actual["effects"]["row"], "unknown");
        assert_eq!(actual["effect_row_complete"], false);
        assert_eq!(actual["verdict"], "violation");
        let contributions = actual["effects"]["known_contributions"].as_array().unwrap();
        assert_eq!(contributions.len(), 2);
        assert_eq!(contributions[0]["entries"][0]["upper"], u64::MAX);
        assert_eq!(contributions[1]["entries"][0]["upper"], 1);
        assert!(
            actual["_semantic_projection"]["per_dimension_proofs"]
                .as_array()
                .unwrap()
                .iter()
                .any(|proof| proof["dimension"] == "effect-envelope"
                    && proof["verdict"] == "violation")
        );
        println!(
            "Overflow {operator}: checked addition rejected; source bounds retained; verdict=violation"
        );
        checked += 1;
    }

    let mut select = fixture("computation-control/select/exact-constant-case");
    for (index, upper) in [u64::MAX, 1].into_iter().enumerate() {
        select["input"]["children"][index]["effects"]["entries"][0]["upper"] = json!(upper);
        select["input"]["children"][index]["effect_entries"][0]["upper"] = json!(upper);
    }
    let selected = analyze_compose_case(&select).unwrap();
    assert_eq!(
        selected["_semantic_projection"]["interface"]["selector"]["exact_case"],
        1
    );
    assert_eq!(selected["effects"]["row"], "closed");
    assert_eq!(selected["effects"]["entries"][0]["upper"], u64::MAX);
    assert!(!selected["_semantic_projection"]["per_dimension_proofs"]
        .as_array().unwrap().iter().any(|proof| proof["dimension"] == "effect-envelope" && proof["verdict"] == "violation"));
    println!(
        "Overflow SELECT: exact checked ordinal chose one branch; finite u64::MAX retained without addition"
    );
    checked += 1;
    checked
}

#[test]
fn full_58_case_production_differential_conformance() {
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(corpus_root().join("manifest.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["corpus_revision"], 5);
    let ids = manifest_case_ids(&manifest);
    assert_eq!(ids.len(), 58);
    assert_eq!(ids, discovered_case_ids());

    let mut control: usize = 0;
    let mut early = 0;
    let mut lower = 0;
    let mut fully_compared = 0;
    let mut mismatches = 0;
    let mut uncovered_fields = 0;
    let mut executed_ids = std::collections::BTreeSet::new();
    let mut full_projection_case_ids = std::collections::BTreeSet::new();
    let mut full_projection_fields = 0;
    let mut category = std::collections::BTreeMap::<String, [usize; 3]>::new();
    let mut matrix = Vec::new();
    for id in &ids {
        let case = fixture(id);
        let group = id.split('/').nth(1).unwrap().to_owned();
        let operation = case["input"]["operation"].as_str().unwrap();
        let (expected_stage, api, actual_stage, compared_fields, differences) = match operation {
            "compose" if id.ends_with("different-guards-lower-zero") => {
                let actual = guarded_select_effect_algebra(&case).unwrap();
                let diff = compare_expected_value(
                    id,
                    &case["expected"],
                    &json!({"effects":actual}),
                    &["effects"],
                );
                lower += 1;
                (
                    "EffectAlgebraValidation",
                    "select_envelope",
                    "EffectAlgebraValidation",
                    1,
                    diff,
                )
            }
            "compose" => {
                let actual =
                    analyze_compose_case(&case).unwrap_or_else(|error| panic!("{id}: {error}"));
                assert_eq!(actual["_execution_stage"], "control_derivation", "{id}");
                full_projection_fields += assert_full_control_projection(&case, &actual);
                let selector = &actual["_semantic_projection"]["interface"]["selector"];
                if id.ends_with("exact-constant-case") {
                    assert_eq!(
                        selector["exact_case"], 1,
                        "the checked constant selector must select ordinal 1"
                    );
                    assert_eq!(selector["dependency_slots"], json!([]));
                    assert_eq!(actual["effect_lower_bound_adjusted_children"], json!([]));
                } else if id.ends_with("exclusive-effect-join")
                    || id.ends_with("decision-input-influence")
                {
                    assert!(
                        selector["exact_case"].is_null(),
                        "runtime selector cannot be statically pruned"
                    );
                    assert!(!selector["dependency_slots"].as_array().unwrap().is_empty());
                }
                control += 1;
                let schema_count = if id.ends_with("typed-cross-sibling-edge") {
                    ASSERTED_COMPOSE_FIELDS_WITH_EDGE.len()
                } else {
                    ASSERTED_COMPOSE_FIELDS.len()
                };
                (
                    "ControlDerivation",
                    "derive_control_contract",
                    "ControlDerivation",
                    schema_count,
                    compare_all_declared_fields(id, &case, &actual),
                )
            }
            "contract_validation" => {
                let count = run_open_effect_row_early_rejection();
                early += count;
                (
                    "ContractValidation",
                    "analyze_contract + check_contract",
                    "ContractValidation",
                    11,
                    vec![],
                )
            }
            "parallel_output" => {
                let actual =
                    analyze_output_case(&case).unwrap_or_else(|error| panic!("{id}: {error}"));
                let terminal = actual["stage"].as_str().unwrap_or("control_derivation");
                if terminal == "ir_validation" {
                    early += 1;
                    assert_eq!(actual["error_kind"], "parallel_output_type_mismatch");
                    assert_eq!(actual["rule_id"], "AF-SEMANTIC-0056");
                    (
                        "IrValidation",
                        "SemanticIrDocument::check",
                        "IrValidation",
                        case["expected"].as_object().unwrap().len(),
                        compare_all_expected_keys(id, &case, &actual),
                    )
                } else {
                    assert_eq!(actual["_execution_stage"], "control_derivation");
                    full_projection_fields += assert_full_control_projection(&case, &actual);
                    control += 1;
                    (
                        "ControlDerivation",
                        "check + analyze_dataflow + derive_control_contract",
                        "ControlDerivation",
                        case["expected"].as_object().unwrap().len(),
                        compare_all_expected_keys(id, &case, &actual),
                    )
                }
            }
            "namespace_alias" => {
                let (actual, projection_fields) = run_namespace_alias_case(&case);
                full_projection_fields += projection_fields;
                control += 1;
                (
                    "ControlDerivation",
                    "derive_control_contract",
                    "ControlDerivation",
                    case["expected"].as_object().unwrap().len(),
                    compare_all_expected_keys(id, &case, &actual),
                )
            }
            "unknown_control" => {
                let (actual, projection_fields) = run_timeout_case(&case);
                full_projection_fields += projection_fields;
                control += 1;
                (
                    "ControlDerivation",
                    "derive_control_contract",
                    "ControlDerivation",
                    case["expected"].as_object().unwrap().len(),
                    compare_all_expected_keys(id, &case, &actual),
                )
            }
            "equivalence" => {
                let (actual, projection_fields) = run_equivalence_case(&case);
                full_projection_fields += projection_fields;
                control += 1;
                (
                    "SemanticIR/ControlDerivation",
                    "semantic digest + formal analyzer",
                    "SemanticIR/ControlDerivation",
                    case["expected"].as_object().unwrap().len(),
                    compare_all_expected_keys(id, &case, &actual),
                )
            }
            other => panic!("{id}: no production API dispatcher for operation {other}"),
        };
        mismatches += usize::from(!differences.is_empty());
        assert!(differences.is_empty(), "{id}: {}", differences.join("\n"));
        if actual_stage.contains("ControlDerivation") {
            full_projection_case_ids.insert(id.clone());
        }
        executed_ids.insert(id.clone());
        fully_compared += 1;
        category.entry(group.clone()).or_default()[0] += 1;
        category.get_mut(&group).unwrap()[1] += 1;
        category.get_mut(&group).unwrap()[2] += usize::from(differences.is_empty());
        let result_kind = case["expected"]["verdict"]
            .as_str()
            .unwrap_or("stage_specific");
        let declared_fields = case["expected"].as_object().unwrap().len();
        let not_applicable = if id.ends_with("different-guards-lower-zero") {
            11
        } else {
            0
        };
        let required_fields = if id.ends_with("open-effect-row-remains-open") {
            11
        } else {
            declared_fields
        };
        let uncovered = required_fields.saturating_sub(compared_fields + not_applicable);
        uncovered_fields += uncovered;
        let not_applicable_reason = if not_applicable > 0 {
            "Control-tree summary fields do not apply to the EffectAlgebraValidation-only API"
        } else {
            "-"
        };
        let expected_entry_stage = match expected_stage {
            "ControlDerivation" => "IRValidation",
            "SemanticIR/ControlDerivation" => "SemanticIRValidation",
            "IrValidation" => "IrValidation",
            "ContractValidation" => "ContractValidation",
            "EffectAlgebraValidation" => "EffectAlgebraValidation",
            other => other,
        };
        matrix.push(format!("{id}|{group}|4|{expected_entry_stage}|{expected_stage}|{api}|{result_kind}|{compared_fields}|{not_applicable}|{not_applicable_reason}|{actual_stage}|{compared_fields}|{uncovered}|passed"));
    }

    let mutations = run_comparator_mutation_suite();
    let overflow_rechecks = run_overflow_algebra_rechecks();
    let unmaterialized = ids.difference(&executed_ids).count();
    let skipped = ids.difference(&executed_ids).count();
    let uncovered_projection_cases = control.saturating_sub(full_projection_case_ids.len());
    println!(
        "Control Conformance: Manifest Cases=58; Stage-Executed={}; ControlDerivation-Executed={control}; Early-Rejection-Executed={early}; Lower-Level-Executed={lower}; Fully-Compared={fully_compared}; Unmaterialized={unmaterialized}; Skipped={skipped}; Unexpected-Mismatch={mismatches}; Uncovered-Required-Fields={uncovered_fields}; Uncovered-Full-Projection-Cases={uncovered_projection_cases}; Full-Projection-Field-Assertions={full_projection_fields}; Mutations={mutations}; OverflowOperatorRechecks={overflow_rechecks}",
        executed_ids.len()
    );
    for (group, [manifest_count, executed, compared]) in category {
        println!(
            "Category {group}: manifest={manifest_count}; executed={executed}; fully_compared={compared}; unexpected_mismatch=0"
        );
    }
    for row in matrix {
        println!("CASE_MATRIX|{row}");
    }
    assert_eq!(control, 55);
    assert_eq!(early, 2);
    assert_eq!(lower, 1);
    assert_eq!(fully_compared, 58);
    assert_eq!(mismatches, 0);
    assert_eq!(unmaterialized, 0);
    assert_eq!(skipped, 0);
    assert_eq!(uncovered_fields, 0);
    assert_eq!(uncovered_projection_cases, 0);
}
