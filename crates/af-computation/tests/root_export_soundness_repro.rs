#![recursion_limit = "512"]

#[allow(dead_code)]
mod support;

use serde_json::{Value, json};
use support::control_fixture::analyze_root_export_case;

fn corpus_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../spec/conformance/computation-root-export-soundness")
}

fn read_fixture(relative: &str) -> Value {
    serde_json::from_slice(
        &std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)).unwrap(),
    )
    .unwrap()
}

fn signature(entries: &Value) -> Value {
    json!({"row":"closed","entries":entries})
}

fn snapshot_effect(upper: u64) -> Value {
    json!({"row":"closed","entries":[{
        "atom":{"operation":{"catalog":"agentflow.effects@1","name":"write"},"resource_selector":{"requirement_port_no":1,"scope_bindings":[]},"qualifiers":[{"family":"idempotency","member":"natural"}]},
        "lower":0,"upper":upper,"guard":"unconditional"
    }]})
}

fn materialize(case: &Value) -> Value {
    let mut fixture = match case["kind"].as_str().unwrap() {
        "effect_containment" => read_fixture(
            "../../spec/conformance/computation-root-export/positive/exact-input-output.json",
        ),
        "resolver_snapshot" => read_fixture(
            "../../spec/conformance/computation-root-export/subflow/body-evidence-absent.json",
        ),
        other => panic!("unknown soundness fixture kind {other}"),
    };
    fixture["case_id"] = case["case_id"].clone();
    if case["kind"] == "effect_containment" {
        fixture["input"]["recognized_resource_attributes"] =
            case["input"]["recognized_resource_attributes"].clone();
        fixture["input"]["effect_selector_rules"] = case["input"]["effect_selector_rules"].clone();
        fixture["input"]["children"][0]["input_types"] = json!([{"port":1,"primitive":"int64"}]);
        fixture["input"]["children"][0]["resources"] = json!([]);
        fixture["input"]["children"][0]["effects"] = signature(&case["input"]["root_effects"]);
        fixture["input"]["export_overrides"]["effects"] =
            signature(&case["input"]["export_effects"]);
    } else {
        fixture["input"]["children"][0]["resources"] = json!([
            {"component_port":1,"resource_slot":"resource:1","attributes":[]}
        ]);
        let root_effect = snapshot_effect(
            case["input"]["root_snapshot_export_upper"]
                .as_u64()
                .unwrap(),
        );
        let checker_effect = snapshot_effect(
            case["input"]["checker_snapshot_export_upper"]
                .as_u64()
                .unwrap(),
        );
        let top_effect = snapshot_effect(case["input"]["top_export_upper"].as_u64().unwrap());
        fixture["input"]["export_overrides"]["effects"] = top_effect;
        fixture["input"]["snapshot_a_export_overrides"] = json!({"effects":root_effect});
        fixture["input"]["snapshot_b_export_overrides"] = json!({"effects":checker_effect});
        fixture["input"]["checker_context_variant"] =
            case["input"]["checker_context_variant"].clone();
        fixture["input"]["snapshot_b_without_subflow_export"] =
            case["input"]["snapshot_b_without_subflow_export"].clone();
        if case["input"]["snapshot_b_extra_component"] == true {
            fixture["input"]["snapshot_b_component_extra_overrides"] = json!({
                "usage":{"coordinates":{"forward/api_calls":1,"recovery/api_calls":0}}
            });
        }
    }
    fixture
}

fn canonical_entries(entries: &Value) -> Vec<String> {
    let mut result = entries
        .as_array()
        .unwrap()
        .iter()
        .map(Value::to_string)
        .collect::<Vec<_>>();
    result.sort();
    result
}

fn selector_compatible(root_atom: &Value, export_atom: &Value) -> bool {
    root_atom["operation"] == export_atom["operation"]
        && root_atom["qualifiers"] == export_atom["qualifiers"]
        && root_atom["resource_selector"]["requirement_port_no"]
            == export_atom["resource_selector"]["requirement_port_no"]
        && export_atom["resource_selector"]["scope_bindings"]
            .as_array()
            .unwrap()
            .iter()
            .all(|binding| {
                root_atom["resource_selector"]["scope_bindings"]
                    .as_array()
                    .unwrap()
                    .contains(binding)
            })
}

fn guard_contained(root: &Value, export: &Value) -> bool {
    export == "unconditional" || root == export
}

fn guard_holds(guard: &Value, value: i64) -> bool {
    guard == "unconditional"
        || guard["input_equals"]["port"] == 1
            && guard["input_equals"]["value"].as_i64() == Some(value)
}

fn checked_usize(value: &Value) -> usize {
    usize::try_from(value.as_u64().unwrap()).expect("fixture counts fit usize")
}

fn p30_case() -> Value {
    let mut case: Value = serde_json::from_slice(
        &std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../spec/conformance/computation-root-export/positive/exact-input-output.json",
        ))
        .unwrap(),
    )
    .unwrap();
    case["input"]["recognized_resource_attributes"] = json!(["a", "b"]);
    case["input"]["effect_selector_rules"] = json!([
        {"selector_key":"a", "resource_attribute_key":"a", "required":false},
        {"selector_key":"b", "resource_attribute_key":"b", "required":false}
    ]);
    case["input"]["children"][0]["input_types"] = json!([{"port":1,"primitive":"int64"}]);
    case["input"]["children"][0]["resources"] = json!([]);
    case["input"]["children"][0]["effects"] = json!({"row":"closed","entries":[{
        "atom":{"operation":{"catalog":"agentflow.effects@1","name":"write"},"resource_selector":{"requirement_port_no":1,"scope_bindings":[{"key":"a","value":"1"},{"key":"b","value":"1"}]},"qualifiers":[{"family":"idempotency","member":"natural"}]},
        "lower":0,"upper":10,"guard":"unconditional"
    }]});
    case["input"]["export_overrides"]["effects"] = json!({"row":"closed","entries":[
        {"atom":{"operation":{"catalog":"agentflow.effects@1","name":"write"},"resource_selector":{"requirement_port_no":1,"scope_bindings":[{"key":"a","value":"1"}]},"qualifiers":[{"family":"idempotency","member":"natural"}]},"lower":0,"upper":10,"guard":{"input_equals":{"port":1,"value":0}}},
        {"atom":{"operation":{"catalog":"agentflow.effects@1","name":"write"},"resource_selector":{"requirement_port_no":1,"scope_bindings":[{"key":"b","value":"1"}]},"qualifiers":[{"family":"idempotency","member":"natural"}]},"lower":0,"upper":5,"guard":"unconditional"}
    ]});
    case
}

fn p31_effect(upper: u64) -> Value {
    json!({"row":"closed","entries":[{
        "atom":{"operation":{"catalog":"agentflow.effects@1","name":"write"},"resource_selector":{"requirement_port_no":1,"scope_bindings":[]},"qualifiers":[{"family":"idempotency","member":"natural"}]},
        "lower":0,"upper":upper,"guard":"unconditional"
    }]})
}

fn p31_case() -> Value {
    let mut case: Value = serde_json::from_slice(
        &std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../spec/conformance/computation-root-export/subflow/body-evidence-absent.json",
        ))
        .unwrap(),
    )
    .unwrap();
    case["input"]["children"][0]["resources"] = json!([
        {"component_port":1,"resource_slot":"resource:1","attributes":[]}
    ]);
    case["input"]["export_overrides"]["effects"] = p31_effect(1);
    case["input"]["snapshot_a_export_overrides"] = json!({"effects":p31_effect(1)});
    case["input"]["snapshot_b_export_overrides"] = json!({"effects":p31_effect(10)});
    case
}

#[test]
fn p30_cross_row_effect_containment_is_not_checked() {
    let actual = analyze_root_export_case(&p30_case()).unwrap();
    let epsilon = actual["dimensions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|dimension| dimension["dimension"] == "epsilon")
        .unwrap();
    println!(
        "P30 post-fix static={} capability={} epsilon={}",
        actual["static_compatibility"],
        actual["checked_static_compatibility_capability_issued"],
        epsilon["proof"]["verdict"]
    );
    assert_ne!(epsilon["proof"]["verdict"], "proven");
    assert_ne!(actual["static_compatibility"], "proven");
    assert_eq!(
        actual["checked_static_compatibility_capability_issued"],
        false
    );
}

#[test]
fn p31_stale_root_snapshot_is_not_checked() {
    let actual = analyze_root_export_case(&p31_case()).unwrap();
    let source_export = &actual["_production_fixture_inputs"]["snapshot_a_export_digest"]["hex"];
    let current_target_export =
        &actual["assertion_identity"]["typed_mappings"]["subflows"][0]["export"]["hex"];
    println!(
        "P31 post-fix static={} capability={} body={} source-export-A={} resolver-export-B={} top-export-C={}",
        actual["static_compatibility"],
        actual["checked_static_compatibility_capability_issued"],
        actual["body_conformance"],
        source_export,
        current_target_export,
        actual["assertion_identity"]["export_digest"]["hex"]
    );
    assert_ne!(actual["static_compatibility"], "proven");
    assert_eq!(
        actual["checked_static_compatibility_capability_issued"],
        false
    );
    assert_eq!(actual["body_conformance"], "requires_evidence");
    assert_eq!(actual["preflight"][2]["verdict"], "unproven");
    assert!(
        actual["preflight"][2]["reference"]
            .as_str()
            .unwrap()
            .contains("exported contract digest differs")
    );
}

#[test]
fn independent_soundness_corpus_executes_formal_production_inputs() {
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(corpus_root().join("manifest.json")).unwrap())
            .unwrap();
    let cases: Vec<Value> =
        serde_json::from_slice(&std::fs::read(corpus_root().join("cases.json")).unwrap()).unwrap();
    let ids = cases
        .iter()
        .map(|case| case["case_id"].as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        manifest["case_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| id.as_str().unwrap().to_owned())
            .collect::<Vec<_>>()
    );
    assert_eq!(cases.len(), checked_usize(&manifest["case_count"]));

    let mut compared = 0usize;
    let mut capabilities = 0usize;
    for case in &cases {
        let fixture = materialize(case);
        let actual = analyze_root_export_case(&fixture).unwrap_or_else(|error| {
            panic!(
                "{} failed in formal production fixture: {error}",
                case["case_id"]
            )
        });
        let expected = &case["expected"];
        assert_eq!(actual["stage"], expected["stage"], "{}", case["case_id"]);
        assert_eq!(
            actual["static_compatibility"], expected["static_compatibility"],
            "{}",
            case["case_id"]
        );
        assert_eq!(
            actual["checked_static_compatibility_capability_issued"],
            expected["checked_capability"],
            "{}",
            case["case_id"]
        );
        assert_eq!(
            actual["body_conformance"], expected["body_conformance"],
            "{}",
            case["case_id"]
        );
        assert_eq!(
            actual["runtime_authority"], "not_established",
            "{}",
            case["case_id"]
        );
        let dimensions = actual["dimensions"].as_array().unwrap();
        assert_eq!(dimensions.len(), 9, "{}", case["case_id"]);
        let epsilon = dimensions
            .iter()
            .find(|item| item["dimension"] == "epsilon")
            .unwrap();
        assert_eq!(
            epsilon["proof"]["verdict"], expected["epsilon"],
            "{}",
            case["case_id"]
        );
        let other_verdict = if expected["preflight"] == "unproven" {
            "unproven"
        } else {
            "proven"
        };
        for dimension in dimensions
            .iter()
            .filter(|item| item["dimension"] != "epsilon")
        {
            assert_eq!(
                dimension["proof"]["verdict"], other_verdict,
                "{} {}",
                case["case_id"], dimension["dimension"]
            );
        }
        assert_eq!(
            actual["preflight"][2]["verdict"], expected["preflight"],
            "{}",
            case["case_id"]
        );

        let checked_inputs = &actual["_production_fixture_inputs"];
        if case["kind"] == "effect_containment" {
            let input_type = &fixture["input"]["children"][0]["input_types"][0];
            assert_eq!(input_type["port"], expected["input_type"]["port"]);
            assert_eq!(input_type["primitive"], expected["input_type"]["primitive"]);
            let source_entries = checked_inputs["source_effects"]["entries"]
                .as_array()
                .unwrap();
            let export_entries = checked_inputs["top_export_effects"]["entries"]
                .as_array()
                .unwrap();
            assert_eq!(
                canonical_entries(&checked_inputs["source_effects"]["entries"]),
                canonical_entries(&case["input"]["root_effects"]),
                "{} source rows",
                case["case_id"]
            );
            assert_eq!(
                canonical_entries(&checked_inputs["top_export_effects"]["entries"]),
                canonical_entries(&case["input"]["export_effects"]),
                "{} export rows",
                case["case_id"]
            );
            assert_eq!(
                actual["assertion_identity"]["typed_mappings"]["effects"]
                    .as_array()
                    .unwrap()
                    .len(),
                checked_usize(&expected["root_effect_count"]),
                "{} source atom identity count",
                case["case_id"]
            );
            assert_eq!(
                expected["row_analysis"].as_array().unwrap().len(),
                source_entries.len()
            );
            for (root_ordinal, analysis) in expected["row_analysis"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
            {
                let root_entry = &source_entries[root_ordinal];
                assert_eq!(analysis["root_ordinal"], root_ordinal);
                assert_eq!(
                    analysis["attempted_export_rows"].as_array().unwrap().len(),
                    export_entries.len()
                );
                for (export_ordinal, row) in analysis["attempted_export_rows"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .enumerate()
                {
                    let export_entry = &export_entries[export_ordinal];
                    assert_eq!(row["export_ordinal"], export_ordinal);
                    assert_eq!(row["root_atom"], root_entry["atom"]);
                    assert_eq!(row["export_atom"], export_entry["atom"]);
                    assert_eq!(
                        row["operation_qualifier_selector_compatible"],
                        selector_compatible(&root_entry["atom"], &export_entry["atom"])
                    );
                    assert_eq!(
                        row["bounds_covered"],
                        export_entry["upper"].as_u64().unwrap()
                            >= root_entry["upper"].as_u64().unwrap()
                    );
                    assert_eq!(
                        row["guard_contained"],
                        guard_contained(&root_entry["guard"], &export_entry["guard"])
                    );
                    if !row["typed_countermodel_candidate"].is_null() {
                        let candidate = &row["typed_countermodel_candidate"];
                        assert_eq!(candidate["port"], input_type["port"]);
                        assert_eq!(candidate["primitive"], input_type["primitive"]);
                        let value = candidate["value"].as_i64().unwrap();
                        assert!(guard_holds(&root_entry["guard"], value));
                        assert!(!guard_holds(&export_entry["guard"], value));
                    }
                }
            }
            if case["case_id"] == "root-export-soundness/f01/p30-split-guard-bound-selector" {
                let rows = &expected["row_analysis"][0]["attempted_export_rows"];
                assert_eq!(rows[0]["bounds_covered"], true);
                assert_eq!(rows[0]["guard_contained"], false);
                assert_eq!(
                    rows[0]["typed_countermodel_candidate"],
                    json!({"port":1,"primitive":"int64","value":1})
                );
                assert_eq!(rows[1]["bounds_covered"], false);
                assert_eq!(rows[1]["guard_contained"], true);
                assert_eq!(expected["epsilon"], "unproven");
            }
        } else {
            let root_target = &checked_inputs["root_resolver_identity"]["subflow_targets"][0]["exported_contract_digest"];
            let checker_target =
                &actual["assertion_identity"]["typed_mappings"]["subflows"][0]["export"];
            assert_eq!(
                root_target, &checked_inputs["snapshot_a_export_digest"],
                "{} Root must retain Snapshot A pin",
                case["case_id"]
            );
            assert_eq!(
                checker_target.is_null(),
                case["input"]["snapshot_b_without_subflow_export"] == true,
                "{} current snapshot export presence",
                case["case_id"]
            );
            if !checker_target.is_null() {
                assert_eq!(
                    checker_target, &checked_inputs["snapshot_b_export_digest"],
                    "{} assertion must pin Snapshot B",
                    case["case_id"]
                );
            }
            assert_eq!(
                actual["assertion_identity"]["export_digest"], checked_inputs["top_export_digest"],
                "{} top export pin",
                case["case_id"]
            );
            let root_component_pins =
                &checked_inputs["root_resolver_identity"]["component_contract_digests"];
            let checker_component_pins =
                &actual["assertion_identity"]["manifest"]["component_contract_pins"];
            assert_eq!(
                root_component_pins == checker_component_pins,
                case["input"]["snapshot_b_extra_component"] != true,
                "{} complete component pin set",
                case["case_id"]
            );
        }
        if expected["checked_capability"] == true {
            capabilities += 1;
        }
        compared += 1;
    }
    assert_eq!(compared, checked_usize(&manifest["case_count"]));
    assert_eq!(compared, 47);
    assert_eq!(
        capabilities,
        checked_usize(&manifest["expected_checked_capabilities"])
    );
    println!(
        "RootExport soundness corpus: manifest={compared} stage_executed={compared} fully_compared={compared} mismatches=0 skipped=0 checked_capabilities={capabilities}"
    );
}
