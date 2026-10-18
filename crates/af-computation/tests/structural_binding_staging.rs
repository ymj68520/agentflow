#![recursion_limit = "512"]
use af_computation::*;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
#[allow(dead_code)]
mod support;

#[test]
fn current_analyzer_direct_private_and_alternate_boundaries_deny() {
    let manifest: Value = serde_json::from_slice(
        &std::fs::read(
            root().join("spec/conformance/g01-structural-binding-staging/manifest.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let mut observed = Vec::new();
    for case in manifest["native_boundary_cases"].as_array().unwrap() {
        let mut input: Value = serde_json::from_slice(
            &std::fs::read(root().join(case["source_file"].as_str().unwrap())).unwrap(),
        )
        .unwrap();
        input["input"]["checker_context_variant"] = case["alternate"].clone();
        let actual = support::control_fixture::analyze_root_export_case(&input).unwrap();
        assert_eq!(
            actual["stage"], case["expected_stage"],
            "{}",
            case["case_id"]
        );
        if actual["stage"] == "component_validation_rejected" {
            assert_eq!(actual["checked_component_produced"], false);
        } else {
            assert_eq!(
                actual["checked_static_compatibility_capability_issued"],
                false
            );
            assert_ne!(actual["static_compatibility"], "proven");
            assert!(actual["_edition2_admission_failure"].is_string());
        }
        observed.push(json!({"case_id":case["case_id"],"actual":actual,"positive_root_export":false,"production_capture_case_ids":0}));
    }
    assert_eq!(observed.len(), 70);
    std::fs::write(
        root().join("target/phase-04c1b4c4r3c2r4g2-native-boundary-actual.json"),
        serde_json::to_vec_pretty(&observed).unwrap(),
    )
    .unwrap();
}

#[test]
fn every_registration_input_executes_without_binder_or_capability() {
    structural_descriptor_integrity().unwrap();
    let corpus: Value = serde_json::from_slice(
        &std::fs::read(root().join("spec/conformance/g01-structural-binding-staging/cases.json"))
            .unwrap(),
    )
    .unwrap();
    let mut actual = Vec::new();
    for c in corpus["cases"].as_array().unwrap() {
        let result = match inspect_structural_registration(&c["input"]) {
            Ok(v) => json!({"ok":true,"value":v}),
            Err(e) => json!({"ok":false,"failure":e.metadata()}),
        };
        assert_eq!(result, c["expected"], "{}", c["case_id"]);
        actual.push(json!({"case_id":c["case_id"],"result":result,"production_capture_case_ids":0,"production_dagranks_generated":0,"positive_root_export":false}));
    }
    std::fs::write(
        root().join("target/phase-04c1b4c4r3c2r4g2-registration-actual.json"),
        serde_json::to_vec_pretty(&actual).unwrap(),
    )
    .unwrap();
}

#[test]
fn exact_current_registration_is_not_implementation_activation_or_release() {
    let e = current_registered_analysis_edition();
    assert_eq!(e.registration, EditionRegistration::Registered);
    assert_eq!(
        (
            e.compiler.specifications.len(),
            e.compiler.rules.len(),
            e.registered_sources.len()
        ),
        (44, 217, 70)
    );
    assert!(!e.can_issue_capability());
    assert_eq!(e.support, EditionSupport::NotImplemented);
    assert_eq!(e.activation, EditionActivation::NotAuthorized);
    assert_eq!(e.release, EditionRelease::NotAuthorized);
    check_registered_compiler_identity(&e.compiler).unwrap();
    for n in 1..=7 {
        let mut old = e.compiler.clone();
        old.analysis_semantics_version = format!("agentflow.root-export-checker@{n}");
        assert_eq!(
            check_registered_compiler_identity(&old),
            Err(EditionAdmissionFailure::InvalidIdentity)
        );
    }
    assert!(
        check_registered_report_editions(
            "agentflow.g01.report-shape@6",
            "agentflow.g01.obligation@6"
        )
        .is_err()
    );
    // Frozen old catalogues remain old, not current-owner aliases.
    assert_eq!(
        relation_model_catalogue().unwrap()["model_id"],
        "agentflow.computation.proof-report.model@6"
    );
    assert_eq!(
        registered_model_catalogue().unwrap()["model_id"],
        "agentflow.computation.proof-report.model@5"
    );
    assert_eq!(
        structural_model_catalogue().unwrap()["model_id"],
        STRUCTURAL_MODEL_ID
    );
}
