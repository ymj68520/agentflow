use af_portable::{DecoderLimits, PortableProofCodec};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn limits() -> DecoderLimits {
    DecoderLimits {
        max_input_bytes: 128 * 1024 * 1024,
        max_scalar_bytes: 32 * 1024 * 1024,
        ..DecoderLimits::default()
    }
}
#[test]
fn exact_new_tuple_old_mixed_and_default_resource_limits_never_activate() {
    let dir = root().join("spec/conformance/g01-relation-schema-staging");
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(dir.join("manifest.json")).unwrap()).unwrap();
    let mut actual = Vec::new();
    for c in manifest["portable_cases"].as_array().unwrap() {
        let data = std::fs::read(dir.join(c["file"].as_str().unwrap())).unwrap();
        let policy = if c["decoder_policy"] == "unchanged_default" {
            DecoderLimits::default()
        } else {
            limits()
        };
        let err = PortableProofCodec::new(policy)
            .validate_supported(&data)
            .expect_err("metadata never authorizes RootExport");
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
        root().join("target/phase-04c1b4c4r3c2r3g2-portable-actual.json"),
        serde_json::to_vec_pretty(&actual).unwrap(),
    )
    .unwrap();
}
#[test]
fn caller_flags_and_noncanonical_bytes_cannot_enable_relation_successor() {
    let data = std::fs::read(
        root().join("spec/conformance/g01-relation-schema-staging/portable/current-denied.ace1"),
    )
    .unwrap();
    let mut value: Value = serde_json::from_slice(&data).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .insert("supported".into(), Value::Bool(true));
    let err = PortableProofCodec::new(limits())
        .validate_supported(&serde_json::to_vec(&value).unwrap())
        .unwrap_err();
    assert_eq!(format!("{:?}", err.stage), "D0");
    assert_eq!(err.family, "InvalidCanonicalEncoding");
    let mut extra = vec![b' '];
    extra.extend_from_slice(&data);
    let second = PortableProofCodec::new(limits())
        .validate_supported(&extra)
        .unwrap_err();
    assert_eq!(format!("{:?}", second.stage), "D1");
    assert_eq!(second.family, "InvalidCanonicalEncoding");
    std::fs::write(root().join("target/phase-04c1b4c4r3c2r3g2-portable-runtime-actual.json"),serde_json::to_vec_pretty(&json!([
        {"case_id":"g01-relation-schema-staging/portable-caller-support-flag","stage":"D0","family":"InvalidCanonicalEncoding"},
        {"case_id":"g01-relation-schema-staging/portable-noncanonical","stage":"D1","family":"InvalidCanonicalEncoding"}])).unwrap()).unwrap();
}
