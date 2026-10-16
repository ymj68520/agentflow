use af_portable::{DecoderLimits, PortableProofCodec};
use serde_json::Value;
use std::path::Path;
#[test]
fn all_native_current_only_portable_paths_deny_supported_inputs() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../spec/conformance/g01-native-guard-staging");
    let m: Value =
        serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
    for c in m["portable_cases"].as_array().unwrap() {
        let b = std::fs::read(root.join(c["file"].as_str().unwrap())).unwrap();
        // No production/default limit is changed. Exercise exact D4 with an
        // explicitly bounded existing per-process API, and test the unchanged
        // default's earlier ResourceLimit as its own separate case.
        let limits = if c["decoder_policy"] == "unchanged_default" {
            DecoderLimits::default()
        } else {
            DecoderLimits {
                max_scalar_bytes: 8 * 1024 * 1024,
                ..DecoderLimits::default()
            }
        };
        let e = PortableProofCodec::new(limits)
            .validate_supported(&b)
            .expect_err("Registration is not support or activation");
        assert_eq!(
            format!("{:?}", e.stage),
            c["stage"],
            "{}: {e}",
            c["case_id"]
        );
        assert_eq!(e.family, c["family"], "{}: {e}", c["case_id"]);
    }
    assert_eq!(m["portable_cases"].as_array().unwrap().len(), 9);
}
