use af_computation::native_source_staging::{
    ANALYSIS, MODEL, OBLIGATION, REPORT, check_native_source_registration_identity,
    inspect_native_source_registration, native_source_registered_analysis_edition,
};
use serde_json::{Value, json};
use std::path::Path;

#[test]
fn registration_declarations_are_not_production_or_capabilities() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let corpus: Value = serde_json::from_slice(
        &std::fs::read(root.join("spec/conformance/g01-native-source-staging/cases.json")).unwrap(),
    )
    .unwrap();
    let mut actual = Vec::new();
    for row in corpus["cases"].as_array().unwrap() {
        let result = match inspect_native_source_registration(&row["input"]) {
            Ok(value) => json!({"ok":true,"value":value}),
            Err(error) => json!({"ok":false,"failure":format!("{error:?}")}),
        };
        assert_eq!(result, row["expected"], "{}", row["case_id"]);
        actual.push(json!({"case_id":row["case_id"],"result":result}));
    }
    std::fs::write(
        root.join("target/phase-04c1b4c4r3c2r5g2-registration-actual.json"),
        serde_json::to_vec_pretty(&actual).unwrap(),
    )
    .unwrap();
}

#[test]
fn explicit_successor_is_registered_but_unimplemented_and_legacy8_is_exact() {
    let new = native_source_registered_analysis_edition();
    assert_eq!(new.compiler.analysis_semantics_version, ANALYSIS);
    assert_eq!(
        (new.report_shape, new.obligation_edition),
        (REPORT, OBLIGATION)
    );
    assert_eq!(
        new.registration,
        af_computation::EditionRegistration::Registered
    );
    assert_eq!(
        (
            new.registered_sources.len(),
            new.compiler.specifications.len(),
            new.compiler.rules.len()
        ),
        (74, 48, 226)
    );
    assert!(!new.can_issue_capability());
    check_native_source_registration_identity(&new.compiler).unwrap();
    let legacy = af_computation::current_registered_analysis_edition();
    assert_eq!(
        legacy.compiler.analysis_semantics_version,
        "agentflow.root-export-checker@8"
    );
    assert_eq!(legacy.registered_sources.len(), 70);
    assert_eq!(
        legacy.registration,
        af_computation::EditionRegistration::Registered
    );
    assert!(check_native_source_registration_identity(&legacy.compiler).is_err());
    assert!(af_computation::check_registered_compiler_identity(&new.compiler).is_err());
    let denied = inspect_native_source_registration(
        &json!({"edition":{"analysis":ANALYSIS,"model":MODEL,"report":REPORT,"obligation":OBLIGATION},"op":"production","supported":true,"activated":true,"released":true}),
    );
    assert_eq!(
        denied,
        Err(af_computation::RegisteredModelFailure::NotImplemented)
    );
}
