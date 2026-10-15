use af_portable::PortableProofCodec;
use serde_json::Value;
use std::path::Path;
#[test]
fn actual_successor_portable_admission_matrix_never_bootstraps_supported_inputs() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/conformance/g01-successor-staging");
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
    for case in manifest["portable_cases"].as_array().unwrap() {
        let raw = std::fs::read(root.join(case["file"].as_str().unwrap())).unwrap();
        let failure = PortableProofCodec::default()
            .validate_supported(&raw)
            .expect_err("incomplete successor cannot support full static proof");
        assert_eq!(
            format!("{:?}", failure.stage),
            case["expected_stage"].as_str().unwrap(),
            "{}: {failure}",
            case["case_id"]
        );
        assert_eq!(
            failure.family,
            case["expected_family"].as_str().unwrap(),
            "{}: {failure}",
            case["case_id"]
        );
    }
    assert_eq!(manifest["portable_cases"].as_array().unwrap().len(), 10);
}
