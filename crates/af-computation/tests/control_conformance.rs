//! TEST-ONLY production fixture materialization and conformance differential.
//! NOT A WIRE ABI. NOT A RUNTIME MODEL.
#![recursion_limit = "512"]

mod support;

use std::path::PathBuf;

use serde_json::{Value, json};
use support::control_fixture::{
    analyze_compose_case, guarded_select_effect_algebra, open_row_algebra, validate_contract_case,
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

fn fixture(case_id: &str) -> Value {
    let suffix = case_id.strip_prefix("computation-control/").unwrap();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../spec/conformance/computation-control")
        .join(format!("{suffix}.json"));
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
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
    let asserted_keys = ASSERTED_COMPOSE_FIELDS
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

fn compare_with_schema(
    case_id: &str,
    case: &Value,
    actual: &Value,
    asserted: &[&str],
    not_applicable: &[&str],
) -> Vec<String> {
    let expected = case["expected"].as_object().unwrap();
    let declared = asserted
        .iter()
        .chain(not_applicable)
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let expected_keys = expected
        .keys()
        .map(String::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        declared, expected_keys,
        "{case_id}: each Expected field must be Asserted or explicitly NotApplicable"
    );
    let mut differences = Vec::new();
    for field in asserted {
        diff_json(
            &format!("{case_id}.expected.{field}"),
            &expected[*field],
            actual.get(*field),
            &mut differences,
        );
    }
    differences
}

fn run_four_structured_control_operators() -> (usize, usize) {
    let pilots = [
        "computation-control/golden/minimal-known-sequence-envelope",
        "computation-control/sequence/two-effects-accumulate",
        "computation-control/sequence/second-lower-conditional",
        "computation-control/sequence/usage-checked-sum",
        "computation-control/select/exclusive-effect-join",
        "computation-control/select/usage-maximum",
        "computation-control/select/exact-constant-case",
        "computation-control/parallel/all-effects-accumulate",
        "computation-control/parallel/fail-fast-sibling-effect-retained",
        "computation-control/parallel/fail-fast-lower-zero",
        "computation-control/race/all-contestants-not-winner-only",
        "computation-control/race/loser-usage-retained",
        "computation-control/race/physical-timing-not-decision",
    ];
    let mut diffs = Vec::new();
    let mut executed = 0;
    let mut compared = 0;
    for case_id in pilots {
        let case = fixture(case_id);
        let actual = analyze_compose_case(&case)
            .unwrap_or_else(|error| panic!("{case_id} stopped before ControlDerivation: {error}"));
        assert_eq!(
            actual["_execution_stage"], "control_derivation",
            "{case_id}"
        );
        assert_eq!(
            actual["_resolved_leaf_count"].as_u64().unwrap(),
            case["input"]["children"].as_array().unwrap().len() as u64,
            "{case_id}"
        );
        executed += 1;
        diffs.extend(compare_all_declared_fields(case_id, &case, &actual));
        compared += 1;
    }
    println!(
        "Production Control Cases Executed: {executed}; Semantic Comparisons Completed: {compared}; Mismatches: {}",
        diffs.len()
    );
    assert!(diffs.is_empty(), "{}", diffs.join("\n"));
    (executed, compared)
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

fn run_requires_evidence_subflow_case() -> usize {
    let case_id = "computation-control/proof/requires-evidence-propagates";
    let case = fixture(case_id);
    let actual = analyze_compose_case(&case)
        .unwrap_or_else(|error| panic!("{case_id} stopped before ControlDerivation: {error}"));
    assert_eq!(actual["_execution_stage"], "control_derivation");
    assert_eq!(actual["_resolved_leaf_count"], 2);
    let asserted = [
        "effect_lower_bound_adjusted_children",
        "effect_occurrences",
        "effect_row_complete",
        "effects",
        "verdict",
    ];
    let not_applicable = [
        "execution_by_child",
        "failures",
        "flow",
        "recovery_sources",
        "resources_by_child",
        "unresolved_children",
        "usage",
    ];
    let diffs = compare_with_schema(case_id, &case, &actual, &asserted, &not_applicable);
    assert!(diffs.is_empty(), "{}", diffs.join("\n"));
    assert_eq!(actual["verdict"], "requires_evidence");
    assert!(
        actual["_inherited_obligations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| {
                item["source_node"] == 2
                    && item["dimension"] == "subflow-body-conformance"
                    && item["verdict"] == "requires_evidence"
            })
    );
    1
}

fn run_effect_overflow_negative_case() -> usize {
    let case_id = "computation-control/negative/effect-overflow";
    let case = fixture(case_id);
    let actual = analyze_compose_case(&case)
        .unwrap_or_else(|error| panic!("{case_id} stopped before ControlDerivation: {error}"));
    assert_eq!(actual["_execution_stage"], "control_derivation");
    assert_eq!(actual["verdict"], "violation");
    let asserted = [
        "effect_lower_bound_adjusted_children",
        "effect_occurrences",
        "effect_row_complete",
        "effects",
        "execution_by_child",
        "recovery_sources",
        "unresolved_children",
        "verdict",
    ];
    let not_applicable = ["failures", "flow", "resources_by_child", "usage"];
    let diffs = compare_with_schema(case_id, &case, &actual, &asserted, &not_applicable);
    assert!(diffs.is_empty(), "{}", diffs.join("\n"));
    assert!(actual["_semantic_projection"]["per_dimension_proofs"]
        .as_array()
        .unwrap()
        .iter()
        .any(|proof| proof["dimension"] == "effect-envelope" && proof["verdict"] == "violation"));
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

fn run_guarded_select_lower_level_case() -> usize {
    let case_id = "computation-control/select/different-guards-lower-zero";
    let case = fixture(case_id);
    let actual = guarded_select_effect_algebra(&case).unwrap();
    assert_eq!(actual, case["expected"]["effects"]);
    assert_eq!(actual["row"], "closed");
    assert_eq!(actual["entries"][0]["lower"], 0);
    assert_eq!(actual["entries"][0]["upper"], 3);
    assert_eq!(actual["entries"][0]["guard"], "unconditional");
    println!("{case_id}: EffectAlgebraValidation executed; not counted as ControlDerivation");
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

#[test]
fn production_pilot_reports_stage_aware_conformance_counts() {
    let (control_cases, control_comparisons) = run_four_structured_control_operators();
    let evidence_cases = run_requires_evidence_subflow_case();
    let overflow_cases = run_effect_overflow_negative_case();
    let equivalence_cases = run_same_snapshot_equivalence_case();
    let early_rejections = run_open_effect_row_early_rejection();
    let lower_level_only = run_guarded_select_lower_level_case();
    let mutations = run_comparator_mutation_suite();
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(corpus_root().join("manifest.json")).unwrap())
            .unwrap();
    let production_ids = [
        "computation-control/golden/minimal-known-sequence-envelope",
        "computation-control/sequence/two-effects-accumulate",
        "computation-control/sequence/second-lower-conditional",
        "computation-control/sequence/usage-checked-sum",
        "computation-control/select/exclusive-effect-join",
        "computation-control/select/usage-maximum",
        "computation-control/select/exact-constant-case",
        "computation-control/parallel/all-effects-accumulate",
        "computation-control/parallel/fail-fast-sibling-effect-retained",
        "computation-control/parallel/fail-fast-lower-zero",
        "computation-control/race/all-contestants-not-winner-only",
        "computation-control/race/loser-usage-retained",
        "computation-control/race/physical-timing-not-decision",
        "computation-control/proof/requires-evidence-propagates",
        "computation-control/negative/effect-overflow",
        "computation-control/equivalence/same-snapshot-deterministic",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<std::collections::BTreeSet<_>>();
    let early_ids = [
        "computation-control/proof/open-effect-row-remains-open",
        "computation-control/select/typed-merge-mismatch",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<std::collections::BTreeSet<_>>();
    let lower_ids = ["computation-control/select/different-guards-lower-zero"]
        .into_iter()
        .map(str::to_owned)
        .collect::<std::collections::BTreeSet<_>>();
    let classified = production_ids
        .union(&early_ids)
        .cloned()
        .chain(lower_ids.iter().cloned())
        .collect::<std::collections::BTreeSet<_>>();
    let still_unmaterialized = manifest_case_ids(&manifest).difference(&classified).count();
    assert_eq!(
        control_cases + evidence_cases + overflow_cases + equivalence_cases,
        production_ids.len()
    );
    assert_eq!(
        control_comparisons + evidence_cases + overflow_cases + equivalence_cases,
        production_ids.len()
    );
    assert_eq!(early_rejections, 1);
    assert_eq!(lower_level_only, 1);
    println!(
        "Control Conformance: Production Control Cases Executed={}; Expected Early Rejections Executed={}; Lower-level-only Cases Executed={}; Stage-appropriate Comparisons Completed={}; Mutation Tests Executed={}; Unexpected Mismatches=0; Still Unmaterialized={}",
        production_ids.len(),
        early_rejections,
        lower_level_only,
        production_ids.len() + early_rejections + lower_level_only,
        mutations,
        still_unmaterialized
    );
    println!(
        "Expected Early Rejection CaseIds classified={}; executed={}; not yet executed={}",
        early_ids.len(),
        early_rejections,
        early_ids.len() - early_rejections
    );
    assert_eq!(still_unmaterialized, 39);
}
