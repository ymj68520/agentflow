use af_portable::{DecoderLimits, PortableProofCodec};
use serde_json::Value;
use std::path::Path;
#[test]
fn occurrence_current_old_mixed_and_unknown_profiles_never_enable_root() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../spec/conformance/g01-document-occurrence-staging");
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
    let mut actual = Vec::new();
    for case in manifest["portable_cases"].as_array().unwrap() {
        let data = std::fs::read(root.join(case["file"].as_str().unwrap())).unwrap();
        let limits = if case["decoder_policy"] == "unchanged_default" {
            DecoderLimits::default()
        } else {
            DecoderLimits {
                max_input_bytes: 128 * 1024 * 1024,
                max_scalar_bytes: 32 * 1024 * 1024,
                ..DecoderLimits::default()
            }
        };
        let error = PortableProofCodec::new(limits)
            .validate_supported(&data)
            .expect_err("registered metadata cannot activate support");
        assert_eq!(
            format!("{:?}", error.stage),
            case["stage"].as_str().unwrap(),
            "{}: {error}",
            case["case_id"]
        );
        assert_eq!(
            error.family,
            case["family"].as_str().unwrap(),
            "{}: {error}",
            case["case_id"]
        );
        actual.push(serde_json::json!({"case_id":case["case_id"],"stage":format!("{:?}",error.stage),"family":error.family}));
    }
    std::fs::write(
        root.join("../../../target/phase-04c1b4c4r3c2g2-portable-actual.json"),
        serde_json::to_vec_pretty(&actual).unwrap(),
    )
    .unwrap();
}
#[test]
fn support_flags_and_noncanonical_tuple_bytes_are_not_authority() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../spec/conformance/g01-document-occurrence-staging/portable/current-denied.ace1",
    );
    let data = std::fs::read(root).unwrap();
    let mut input: Value = serde_json::from_slice(&data).unwrap();
    input
        .as_object_mut()
        .unwrap()
        .insert("supported".into(), Value::Bool(true));
    let limits = DecoderLimits {
        max_input_bytes: 128 * 1024 * 1024,
        max_scalar_bytes: 32 * 1024 * 1024,
        ..DecoderLimits::default()
    };
    let error = PortableProofCodec::new(limits)
        .validate_supported(&serde_json::to_vec(&input).unwrap())
        .unwrap_err();
    assert_eq!(error.family, "InvalidCanonicalEncoding");
    let caller_actual = serde_json::json!({"case_id":"g01-document-occurrence-staging/portable-caller-support-flag","stage":format!("{:?}",error.stage),"family":error.family});
    let mut noncanonical = vec![b' '];
    noncanonical.extend_from_slice(&data);
    let error = PortableProofCodec::new(limits)
        .validate_supported(&noncanonical)
        .unwrap_err();
    assert_eq!(error.family, "InvalidCanonicalEncoding");
    let runtime_actual = vec![
        caller_actual,
        serde_json::json!({"case_id":"g01-document-occurrence-staging/portable-noncanonical-current-tuple","stage":format!("{:?}",error.stage),"family":error.family}),
    ];
    let output = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/phase-04c1b4c4r3c2g2-portable-runtime-actual.json");
    std::fs::write(output, serde_json::to_vec_pretty(&runtime_actual).unwrap()).unwrap();
}
