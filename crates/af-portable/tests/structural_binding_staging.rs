use af_portable::{DecoderLimits, PortableProofCodec};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn bounded() -> DecoderLimits {
    DecoderLimits {
        max_input_bytes: 128 * 1024 * 1024,
        max_scalar_bytes: 32 * 1024 * 1024,
        ..DecoderLimits::default()
    }
}
#[test]
fn current_old_mixed_unknown_and_default_limits_never_activate() {
    let d = root().join("spec/conformance/g01-structural-binding-staging");
    let m: Value =
        serde_json::from_slice(&std::fs::read(d.join("manifest.json")).unwrap()).unwrap();
    let mut actual = Vec::new();
    for c in m["portable_cases"].as_array().unwrap() {
        let data = std::fs::read(d.join(c["file"].as_str().unwrap())).unwrap();
        let limits = if c["decoder_policy"] == "unchanged_default" {
            DecoderLimits::default()
        } else {
            bounded()
        };
        let err = PortableProofCodec::new(limits)
            .validate_supported(&data)
            .unwrap_err();
        assert_eq!(
            format!("{:?}", err.stage),
            c["stage"].as_str().unwrap(),
            "{}: {err}",
            c["case_id"]
        );
        assert_eq!(
            err.family,
            c["family"].as_str().unwrap(),
            "{}: {err}",
            c["case_id"]
        );
        actual.push(json!({"case_id":c["case_id"],"stage":format!("{:?}",err.stage),"family":err.family,"positive_root_export":false}));
    }
    std::fs::write(
        root().join("target/phase-04c1b4c4r3c2r4g2-portable-actual.json"),
        serde_json::to_vec_pretty(&actual).unwrap(),
    )
    .unwrap();
}
#[test]
fn caller_flags_and_noncanonical_bytes_are_not_source_authority() {
    let data = std::fs::read(
        root().join("spec/conformance/g01-structural-binding-staging/portable/current-denied.ace1"),
    )
    .unwrap();
    let mut v: Value = serde_json::from_slice(&data).unwrap();
    v.as_object_mut()
        .unwrap()
        .insert("supported".into(), json!(true));
    let err = PortableProofCodec::new(bounded())
        .validate_supported(&serde_json::to_vec(&v).unwrap())
        .unwrap_err();
    assert_eq!(format!("{:?}", err.stage), "D0");
    assert_eq!(err.family, "InvalidCanonicalEncoding");
    let mut bad = vec![b' '];
    bad.extend(data);
    let err2 = PortableProofCodec::new(bounded())
        .validate_supported(&bad)
        .unwrap_err();
    assert_eq!(format!("{:?}", err2.stage), "D1");
    assert_eq!(err2.family, "InvalidCanonicalEncoding");
    std::fs::write(root().join("target/phase-04c1b4c4r3c2r4g2-portable-runtime-actual.json"),serde_json::to_vec_pretty(&json!([
      {"case_id":"g01-structural-binding-staging/portable-caller-support-flag","stage":"D0","family":"InvalidCanonicalEncoding"},
      {"case_id":"g01-structural-binding-staging/portable-noncanonical","stage":"D1","family":"InvalidCanonicalEncoding"}
    ])).unwrap()).unwrap();
}
