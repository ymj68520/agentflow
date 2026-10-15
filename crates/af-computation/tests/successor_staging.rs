#![recursion_limit = "512"]
#[allow(dead_code)]
mod support;
use af_computation::*;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn corpus() -> PathBuf {
    root().join("spec/conformance/g01-successor-staging")
}
fn read(name: &str) -> Value {
    serde_json::from_slice(&std::fs::read(corpus().join(name)).unwrap()).unwrap()
}
fn identity(name: &str) -> CompilerSemanticsIdentity {
    let value = read(name);
    CompilerSemanticsIdentity {
        analysis_semantics_version: value["analysis_semantics_version"].as_str().unwrap().into(),
        registry_content: hex::decode(value["registry_hex"].as_str().unwrap()).unwrap(),
        rule_registry_content: hex::decode(value["rule_registry_hex"].as_str().unwrap()).unwrap(),
        specifications: value["specifications"]
            .as_array()
            .unwrap()
            .iter()
            .map(|source| SpecificationContentIdentity {
                specification_id: source["specification_id"].as_str().unwrap().into(),
                logical_path: source["logical_path"].as_str().unwrap().into(),
                content_sha256: source["content_sha256"].as_str().unwrap().into(),
                content: hex::decode(source["content_hex"].as_str().unwrap()).unwrap(),
            })
            .collect(),
        rules: value["rules"]
            .as_array()
            .unwrap()
            .iter()
            .map(|source| RuleContentIdentity {
                rule_id: source["rule_id"].as_str().unwrap().into(),
                normative_source: source["normative_source"].as_str().unwrap().into(),
                source_anchor: source["source_anchor"].as_str().unwrap().into(),
                short_statement: source["short_statement"].as_str().unwrap().into(),
            })
            .collect(),
    }
}
#[test]
fn actual_native_full_identity_source_catalogue_and_staging_states() {
    let stage = current_registered_analysis_edition();
    assert_eq!(stage.compiler, identity("compiler-successor.json"));
    assert_eq!(
        (
            stage.compiler.specifications.len(),
            stage.compiler.rules.len(),
            stage.registered_sources.len()
        ),
        (24, 149, 50)
    );
    assert_eq!(stage.registration, EditionRegistration::Registered);
    assert_eq!(stage.support, EditionSupport::NotImplemented);
    assert_eq!(stage.activation, EditionActivation::NotAuthorized);
    assert_eq!(stage.release, EditionRelease::NotAuthorized);
    assert_eq!(stage.report_shape, "agentflow.g01.report-shape@2");
    assert_eq!(stage.obligation_edition, "agentflow.g01.obligation@2");
    assert!(!stage.can_issue_capability());
    let reg: Value = serde_json::from_slice(&stage.compiler.registry_content).unwrap();
    for old in stage.registered_sources {
        assert_eq!(
            old.content,
            std::fs::read(root().join(&old.logical_path)).unwrap()
        );
        let row = reg["specifications"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["specification_id"] == old.specification_id)
            .unwrap();
        assert_eq!(
            row["normative_source"]["content_digest"]["hex"],
            old.content_sha256
        );
    }
}
#[test]
fn executed_case_dimension_and_error_registration_full_fields() {
    registered_descriptor_integrity().unwrap();
    let catalogue = registered_model_catalogue().unwrap();
    let cases = read("case-registration.json");
    for case in cases["cases"].as_array().unwrap() {
        let expected = &case["expected_case"];
        let actual = catalogue["case_descriptors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["case"] == expected["case"])
            .unwrap();
        assert_eq!(actual, expected, "{}", case["case_id"]);
        validate_registered_model_shape("SubstitutionCase", &expected["case"]).unwrap();
    }
    let dims = read("dimension-registration.json");
    for case in dims["cases"].as_array().unwrap() {
        let array = catalogue["dimension_descriptors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["dimension"] == case["expected_dimension"]["dimension"])
            .unwrap();
        assert_eq!(array, &case["expected_dimension"], "{}", case["case_id"]);
        assert!(
            array["dependencies"]
                .as_array()
                .is_some_and(|x| x.len() >= 4)
        );
    }
    let errors = read("error-registration.json");
    for case in errors["cases"].as_array().unwrap() {
        validate_registered_model_shape(case["probe_type"].as_str().unwrap(), &case["value"])
            .unwrap();
        let rows = catalogue["dependency_errors"][case["domain"].as_str().unwrap()]
            .as_array()
            .unwrap();
        let array = rows.iter().find(|row| row["kind"] == case["kind"]).unwrap();
        assert_eq!(array["constant"], case["constant"]);
        assert_eq!(array["unknown"], "UnsupportedModel");
        assert!(array["payload"].get("detail").is_some());
    }
    assert_eq!(cases["cases"].as_array().unwrap().len(), 27);
    assert_eq!(dims["cases"].as_array().unwrap().len(), 12);
    assert_eq!(errors["cases"].as_array().unwrap().len(), 69);
}
#[test]
fn actual_shape_and_profile_admission_are_non_authoritative() {
    for case in read("shape-registration.json")["cases"].as_array().unwrap() {
        let actual =
            validate_registered_model_shape(case["type"].as_str().unwrap(), &case["value"])
                .err()
                .map(|failure| format!("{failure:?}"));
        assert_eq!(
            actual.as_deref(),
            case["expected_failure"].as_str(),
            "{}",
            case["case_id"]
        );
    }
    for case in read("manifest.json")["profile_cases"].as_array().unwrap() {
        let failure = admit_registered_profile(
            case["profile"].as_str().unwrap(),
            case["as_root"].as_bool().unwrap(),
        )
        .unwrap_err();
        assert_eq!(
            format!("{failure:?}"),
            case["expected_failure"].as_str().unwrap(),
            "{}",
            case["case_id"]
        );
    }
    assert!(validate_registered_model_shape("ProofVerdict", &json!("Proven")).is_ok());
    assert!(!current_registered_analysis_edition().can_issue_capability());
}
#[test]
fn native_current_only_matrix_and_private_boundary_deny() {
    let base = identity("compiler-successor.json");
    let old = identity("compiler-edition2.json");
    for case in read("manifest.json")["native_cases"].as_array().unwrap() {
        let mut value = base.clone();
        let outcome = match case["mutation"].as_str().unwrap() {
            "current" => check_registered_compiler_identity(&value),
            "exact-edition2" => check_registered_compiler_identity(&old),
            "edition2-label-new" => {
                value.analysis_semantics_version = old.analysis_semantics_version.clone();
                check_registered_compiler_identity(&value)
            }
            "successor-label-old" => {
                value = old.clone();
                value.analysis_semantics_version = ROOT_EXPORT_ANALYSIS_SEMANTICS_VERSION.into();
                check_registered_compiler_identity(&value)
            }
            "missing-model-source" => {
                value
                    .specifications
                    .retain(|source| source.specification_id != SUCCESSOR_MODEL_ID);
                check_registered_compiler_identity(&value)
            }
            "changed-model-content" => {
                value
                    .specifications
                    .iter_mut()
                    .find(|source| source.specification_id == SUCCESSOR_MODEL_ID)
                    .unwrap()
                    .content
                    .push(b' ');
                check_registered_compiler_identity(&value)
            }
            "unknown-owner" => {
                value.analysis_semantics_version = "agentflow.root-export-checker@999".into();
                check_registered_compiler_identity(&value)
            }
            "unknown-shape" => check_registered_report_editions(
                "agentflow.g01.report-shape@999",
                OBLIGATION_IDENTITY_EDITION,
            ),
            "unknown-obligation" => check_registered_report_editions(
                REPORT_SHAPE_EDITION,
                "agentflow.g01.obligation@999",
            ),
            "resource-limit" => {
                value.registry_content = vec![0; 32 * 1024 * 1024 + 1];
                check_registered_compiler_identity(&value)
            }
            x => panic!("unimplemented {x}"),
        };
        assert_eq!(
            outcome
                .err()
                .map(|failure| format!("{failure:?}"))
                .as_deref(),
            case["expected_failure"].as_str(),
            "{}",
            case["case_id"]
        );
        assert!(!current_registered_analysis_edition().can_issue_capability());
    }
}
#[test]
fn rules_and_old_complete_rows_frozen_not_relabelled() {
    let specs: Value = serde_json::from_slice(
        &std::fs::read(root().join("spec/registry/specifications.json")).unwrap(),
    )
    .unwrap();
    let rules: Value =
        serde_json::from_slice(&std::fs::read(root().join("spec/registry/rules.json")).unwrap())
            .unwrap();
    for (file, key, id) in [
        (
            "edition2-specifications.json",
            "specifications",
            "specification_id",
        ),
        ("edition2-rules.json", "rules", "rule_id"),
    ] {
        let old = read(file);
        let current = if key == "rules" { &rules } else { &specs };
        for entry in old[key].as_array().unwrap() {
            assert_eq!(
                current[key]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|row| row[id] == entry[id]),
                Some(entry)
            );
        }
    }
    for case in read("rule-registration.json")["cases"].as_array().unwrap() {
        let row = rules["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["rule_id"] == case["rule_id"])
            .unwrap();
        assert_eq!(row["normative_source"], case["normative_source"]);
        assert_eq!(row["source_anchor"], case["anchor"]);
        assert_eq!(row["short_statement"], case["statement"]);
        assert!(
            row["conformance_refs"]
                .as_array()
                .unwrap()
                .contains(&case["case_id"])
        );
        let text = std::fs::read_to_string(root().join(case["normative_source"].as_str().unwrap()))
            .unwrap();
        assert!(text.contains(case["statement"].as_str().unwrap()));
    }
    let manifest = read("manifest.json");
    let ids = manifest["case_ids"].as_array().unwrap();
    assert_eq!(ids.len(), 173);
    assert_eq!(
        ids.iter()
            .map(|x| x.as_str().unwrap())
            .collect::<BTreeSet<_>>()
            .len(),
        173
    );
}
#[test]
fn real_direct_root_checker_never_issues_current_capability() {
    let folder = root().join("spec/conformance/computation-root-export");
    let mut count = 0;
    for group in ["positive", "negative", "context", "subflow", "recovery"] {
        for entry in std::fs::read_dir(folder.join(group)).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|x| x != "json") {
                continue;
            }
            count += 1;
            let input: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
            let array = support::control_fixture::analyze_root_export_case(&input).unwrap();
            if array["stage"] == "component_validation_rejected" {
                assert_eq!(array["checked_component_produced"], false);
                continue;
            }
            assert_eq!(
                array["checked_static_compatibility_capability_issued"],
                false
            );
            assert_ne!(array["static_compatibility"], "proven");
            assert!(array["_edition2_admission_failure"].is_string());
            assert_eq!(array["body_conformance"], "requires_evidence");
            assert_eq!(array["runtime_authority"], "not_established");
        }
    }
    assert_eq!(count, 35);
}
