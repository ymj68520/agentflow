//! F1: exact typed payload and existing ACE1 grammar, not new G01 facts.
use af_core::{Shape, Value, decode, encode};
use af_effect::{EffectSignature, kind};
use af_semantic::PrimitiveConstant;
use serde_json::json;

fn modules() -> af_idl::CompiledModuleSet {
    af_idl::compile_module_set(&[
        (
            "semantic-core.afidl".into(),
            include_str!("../../../spec/idl/semantic-core.afidl").into(),
        ),
        (
            "effect-signature.afidl".into(),
            include_str!("../../../spec/idl/effect-signature.afidl").into(),
        ),
    ])
    .unwrap()
}
fn signature(constant: &serde_json::Value) -> EffectSignature {
    serde_json::from_value(json!({"row":{"closed":[{
        "atom":{"operation":{"catalog":"agentflow.effects@1","name":"read"},
            "resource_selector":{"requirement_port_no":1,"scope_bindings":[]},"qualifiers":[]},
        "bounds":{"lower":0,"upper":{"finite":1}},
        "guard":{"input_equals":{"port_no":1,"expected":constant}}
    }]}}))
    .unwrap()
}
fn field<'a>(v: &'a Value, name: &str) -> &'a Value {
    let Value::Record(fields) = v else {
        panic!("expected record")
    };
    &fields.iter().find(|(n, _)| n == name).unwrap().1
}
fn variant<'a>(v: &'a Value, name: &str) -> &'a Value {
    let Value::Variant { case, value } = v else {
        panic!("expected variant")
    };
    assert_eq!(case, name);
    value
}
fn lowered_constant(v: &Value) -> &Value {
    let Value::List(entries) = variant(field(v, "row"), "closed") else {
        panic!("expected list")
    };
    field(
        variant(field(&entries[0], "guard"), "input_equals"),
        "expected",
    )
}

#[test]
fn bytes_payloads_and_encoding_roundtrip_preserve_exact_binary() {
    let m = modules();
    let shape = m
        .shape_of("agentflow.effect.signature@1/EffectSignature")
        .unwrap();
    let mut cases = vec![
        ("empty".to_owned(), String::new(), vec![]),
        ("single".into(), "AQ".into(), vec![1]),
        (
            "multi-with-zero".into(),
            "AAEC_w".into(),
            vec![0, 1, 2, 255],
        ),
        ("zeroes".into(), "AAAA".into(), vec![0, 0, 0]),
        ("non-utf8".into(), "__4".into(), vec![255, 254]),
        ("utf8-bytes".into(), "aGk".into(), b"hi".to_vec()),
        ("url-alphabet".into(), "-_-_".into(), vec![251, 255, 191]),
    ];
    // Base64 block boundaries and the decoded lengths on both sides of the
    // unchanged Portable D0 1 MiB scalar-text limit. Core has no D0 policy.
    for n in [
        2, 3, 255, 256, 257, 4095, 4096, 4097, 786_431, 786_432, 786_433,
    ] {
        let bytes: Vec<u8> = (0..n).map(|i| u8::try_from(i % 256).unwrap()).collect();
        cases.push((
            format!("boundary-{n}"),
            af_core::base64url::encode(&bytes),
            bytes,
        ));
    }
    for (id, text, bytes) in cases {
        let raw = signature(&json!({"bytes":text}));
        let value = raw.to_value().unwrap();
        assert_eq!(
            variant(lowered_constant(&value), "bytes"),
            &Value::Bytes(bytes),
            "{id}"
        );
        let wire = raw.canonical_bytes(&m).unwrap();
        assert_eq!(wire, raw.canonical_bytes(&m).unwrap(), "determinism: {id}");
        let decoded = decode(&shape, &wire).unwrap();
        // Record constructor order is not ACE1 key order; compare the exact
        // typed payload and the canonical reencoding, not Vec field order.
        assert_eq!(
            lowered_constant(&decoded),
            lowered_constant(&value),
            "typed decode: {id}"
        );
        assert_eq!(wire, encode(&shape, &decoded).unwrap(), "reencode: {id}");
        let json: serde_json::Value = serde_json::from_slice(&wire).unwrap();
        assert_eq!(
            json["row"]["closed"][0]["guard"]["input_equals"]["expected"],
            json!({"bytes":text})
        );
    }
}

#[test]
fn bytes_and_string_have_distinct_nominal_identity() {
    let bytes = signature(&json!({"bytes":"AQ"})).to_value().unwrap();
    let string = signature(&json!({"string":"AQ"})).to_value().unwrap();
    assert_eq!(
        variant(lowered_constant(&bytes), "bytes"),
        &Value::Bytes(vec![1])
    );
    assert_eq!(
        variant(lowered_constant(&string), "string"),
        &Value::String("AQ".into())
    );
    assert_ne!(lowered_constant(&bytes), lowered_constant(&string));
    let m = modules();
    assert_ne!(
        signature(&json!({"bytes":"AQ"}))
            .canonical_bytes(&m)
            .unwrap(),
        signature(&json!({"string":"AQ"}))
            .canonical_bytes(&m)
            .unwrap()
    );
    assert!(encode(&Shape::Bytes, &Value::String("AQ".into())).is_err());
    assert!(encode(&Shape::String, &Value::Bytes(vec![1])).is_err());
    // Same printable ACE1 scalar is legal under either *different* shape;
    // nominal schema/tag distinguishes them, not a new wire grammar.
    assert_eq!(
        encode(&Shape::Bytes, &Value::Bytes(vec![1])).unwrap(),
        encode(&Shape::String, &Value::String("AQ".into())).unwrap()
    );
}

#[test]
fn bounded_open_known_bytes_and_invalid_values_use_the_same_strict_path() {
    let m = modules();
    for (text, valid) in [("AAEC_w", true), ("AQ==", false)] {
        let mut raw = signature(&json!({"bytes":text}));
        let af_effect::EffectRow::Closed(known) = raw.row else {
            unreachable!()
        };
        raw.row = af_effect::EffectRow::Open(af_effect::EffectOpenRow {
            known,
            residual: vec![],
        });
        if valid {
            let tree = raw.to_value().unwrap();
            let open = variant(field(&tree, "row"), "open");
            let Value::List(entries) = field(open, "known") else {
                panic!("known list")
            };
            let payload = variant(
                field(
                    variant(field(&entries[0], "guard"), "input_equals"),
                    "expected",
                ),
                "bytes",
            );
            assert_eq!(payload, &Value::Bytes(vec![0, 1, 2, 255]));
            let wire = raw.canonical_bytes(&m).unwrap();
            let shape = m
                .shape_of("agentflow.effect.signature@1/EffectSignature")
                .unwrap();
            assert_eq!(
                wire,
                encode(&shape, &decode(&shape, &wire).unwrap()).unwrap()
            );
        } else {
            assert_eq!(
                raw.to_value().unwrap_err().kind(),
                kind::INVALID_EFFECT_GUARD
            );
            assert_eq!(
                raw.canonical_bytes(&m).unwrap_err().kind(),
                kind::INVALID_EFFECT_GUARD
            );
        }
    }
}

#[test]
fn malformed_base64url_remains_a_structural_rejection() {
    let m = modules();
    for text in [
        "=", "AQ==", "+w", "/w", "A", "AB", "AQ\n", "AQ ", "é", "AA=", "AA/", "\0",
    ] {
        let c = PrimitiveConstant::Bytes(text.into());
        assert!(c.validate().is_err(), "{text:?}");
        let raw = signature(&json!({"bytes":text}));
        assert_eq!(
            raw.to_value().unwrap_err().kind(),
            kind::INVALID_EFFECT_GUARD
        );
        assert_eq!(
            raw.canonical_bytes(&m).unwrap_err().kind(),
            kind::INVALID_EFFECT_GUARD
        );
    }
}

#[test]
fn other_constant_variants_remain_exactly_typed() {
    for (source, case, payload) in [
        (json!({"unit":null}), "unit", Value::Unit),
        (json!({"null":null}), "null", Value::Null),
        (json!({"bool":true}), "bool", Value::Bool(true)),
        (json!({"int64":-1}), "int64", Value::Int64(-1)),
        (
            json!({"decimal":"1.5"}),
            "decimal",
            Value::Decimal(af_core::Decimal::parse("1.5").unwrap()),
        ),
        (json!({"string":"AQ"}), "string", Value::String("AQ".into())),
        (
            json!({"timestamp":-1}),
            "timestamp",
            Value::Timestamp(af_core::Timestamp::new(-1)),
        ),
        (
            json!({"duration":1}),
            "duration",
            Value::Duration(af_core::Duration::new(1)),
        ),
    ] {
        assert_eq!(
            variant(
                lowered_constant(&signature(&source).to_value().unwrap()),
                case
            ),
            &payload
        );
    }
}
