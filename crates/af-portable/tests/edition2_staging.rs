use af_portable::PortableProofCodec;
use serde_json::Value;
use std::path::{Path, PathBuf};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn corpus() -> PathBuf {
    root().join("spec/conformance/g01-edition2-staging")
}
fn json(path: impl AsRef<Path>) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}
#[test]
fn real_portable_admission_matrix_never_returns_supported_inputs_or_capability() {
    let manifest = json(corpus().join("manifest.json"));
    let cases = manifest["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 9);
    for case in cases {
        let raw = std::fs::read(corpus().join(case["file"].as_str().unwrap())).unwrap();
        let failure = PortableProofCodec::default()
            .validate_supported(&raw)
            .expect_err("staging must never support full proof");
        assert_eq!(
            format!("{:?}", failure.stage),
            case["expected_stage"].as_str().unwrap(),
            "{}",
            case["case_id"]
        );
        assert_eq!(
            failure.family,
            case["expected_family"].as_str().unwrap(),
            "{}: {failure}",
            case["case_id"]
        );
    }
}
#[test]
fn actual_registered_rules_sources_and_old_rows_match_independent_expectations() {
    let rules = json(root().join("spec/registry/rules.json"));
    let specs = json(root().join("spec/registry/specifications.json"));
    let registered = json(corpus().join("registration.json"));
    assert_eq!(specs["specifications"].as_array().unwrap().len(), 46);
    assert_eq!(rules["rules"].as_array().unwrap().len(), 436);
    for (name, key, id) in [
        (
            "legacy-specifications.json",
            "specifications",
            "specification_id",
        ),
        ("legacy-rules.json", "rules", "rule_id"),
    ] {
        let old = json(corpus().join(name));
        for entry in old[key].as_array().unwrap() {
            assert_eq!(
                rules_or_specs(&specs, &rules, key)
                    .iter()
                    .find(|r| r[id] == entry[id]),
                Some(entry)
            );
        }
    }
    let cases = registered["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 32);
    for case in cases {
        let row = rules["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["rule_id"] == case["rule_id"])
            .unwrap();
        assert_eq!(row["normative_source"], case["source"]);
        assert_eq!(row["source_anchor"], case["anchor"]);
        assert_eq!(row["short_statement"], case["statement"]);
        assert!(
            row["conformance_refs"]
                .as_array()
                .unwrap()
                .contains(&case["case_id"])
        );
        let text = std::fs::read_to_string(root().join(case["source"].as_str().unwrap())).unwrap();
        assert!(text.contains(&format!("<a id=\"{}\">", case["anchor"].as_str().unwrap())));
        assert!(text.contains(case["statement"].as_str().unwrap()));
        for dependency in case["dependencies"].as_array().unwrap() {
            assert!(
                rules["rules"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|row| row["rule_id"] == *dependency)
            );
            assert!(text.contains(dependency.as_str().unwrap()));
        }
    }
    let report = af_spec::SpecCheck::repo(root()).run();
    assert!(report.is_ok(), "{:?}", report.errors);
    assert_eq!(
        (
            report.specifications,
            report.rules,
            report.covered,
            report.intentionally_uncovered,
            report.missing_coverage
        ),
        (46, 436, 435, 1, 0)
    );
    assert_eq!((report.abi_active, report.modules_checked), (775, 17));
}
fn rules_or_specs<'a>(specs: &'a Value, rules: &'a Value, key: &str) -> &'a Vec<Value> {
    if key == "rules" {
        rules[key].as_array().unwrap()
    } else {
        specs[key].as_array().unwrap()
    }
}
#[test]
fn registered_model_is_closed_nonwire_and_full_imports_not_a_wire_change() {
    let text =
        std::fs::read_to_string(root().join("spec/tables/proof-report-model-v1.md")).unwrap();
    let block = text
        .split_once("```json\n")
        .unwrap()
        .1
        .split_once("\n```")
        .unwrap()
        .0;
    let model: Value = serde_json::from_str(block).unwrap();
    assert_eq!(
        model["model_id"],
        "agentflow.computation.proof-report.model@1"
    );
    assert_eq!(
        model["imported_afidl_declarations"]
            .as_array()
            .unwrap()
            .len(),
        314
    );
    for required in model["required_types"].as_array().unwrap() {
        let name = required.as_str().unwrap();
        assert!(
            model["core_type_definitions"]
                .as_str()
                .unwrap()
                .contains(name)
                || model["semantic_constraints"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|s| s.as_str().unwrap().contains(name)),
            "missing {name}"
        );
    }
    assert_eq!(model["bounds"]["canonical_view_bytes"], 134_217_728_u64);
    // This is registration/schema inspection,NOT a normalizer/encoder execution.
}
