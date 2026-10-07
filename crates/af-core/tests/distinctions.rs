//! The state-distinction test: `field missing`, `explicit null`, `unit`,
//! `Option::None`, `Option::Some(v)` must never be collapsed by serialization
//! convenience. ACE1 is schema-directed, so the typed encoder — not the bytes
//! alone — resolves these states, and the errors below prove the boundaries.
//!
//! Phase 1A ratification: `Option` is an explicit single-case tagged object
//! (`{"none":null}` / `{"some":v}`), so the Option layer is always visible in
//! bytes; `unit` and `null` share the `null` token, nominally disambiguated
//! by the schema (OQ-2).

use af_core::{AceError, Decimal, Duration, RecordShape, Shape, Timestamp, Value, decode, encode};

#[test]
fn missing_required_field_is_an_error_not_null() {
    let shape = Shape::Record(RecordShape::new("Required", [("value", Shape::String)]));
    // The field is simply not supplied.
    let value = Value::Record(vec![]);
    let err = encode(&shape, &value).expect_err("missing field must be an error");
    assert!(
        matches!(&err, AceError::MissingField { field } if field.as_str() == "value"),
        "{err}"
    );
}

#[test]
fn explicit_null_field_is_present_bytes_while_missing_is_rejected() {
    // A null-typed field encodes the literal null — key present.
    let null_shape = Shape::Record(RecordShape::new("Nullable", [("value", Shape::Null)]));
    let bytes = encode(
        &null_shape,
        &Value::Record(vec![("value".into(), Value::Null)]),
    )
    .unwrap();
    assert_eq!(bytes, b"{\"value\":null}");
}

#[test]
fn option_none_and_some_are_distinct_bytes() {
    let shape = Shape::Record(RecordShape::new(
        "Maybe",
        [("value", Shape::Option(Box::new(Shape::String)))],
    ));

    let none_bytes = encode(
        &shape,
        &Value::Record(vec![("value".into(), Value::Option(None))]),
    )
    .unwrap();
    let some_bytes = encode(
        &shape,
        &Value::Record(vec![(
            "value".into(),
            Value::Option(Some(Box::new(Value::String("x".into())))),
        )]),
    )
    .unwrap();

    assert_eq!(none_bytes, b"{\"value\":{\"none\":null}}");
    assert_eq!(some_bytes, b"{\"value\":{\"some\":\"x\"}}");
    assert_ne!(none_bytes, some_bytes);
}

#[test]
fn unit_and_null_are_distinct_nominal_types() {
    // Byte-level: both render as `null` in their own schema position (OQ-2)...
    let unit_bytes = encode(&Shape::Unit, &Value::Unit).unwrap();
    let null_bytes = encode(&Shape::Null, &Value::Null).unwrap();
    assert_eq!(unit_bytes, b"null");
    assert_eq!(null_bytes, b"null");

    // ...but the typed encoder never lets them be swapped: each nominal type
    // only accepts its own value variant.
    assert!(matches!(
        encode(&Shape::Unit, &Value::Null),
        Err(AceError::TypeMismatch { .. })
    ));
    assert!(matches!(
        encode(&Shape::Null, &Value::Unit),
        Err(AceError::TypeMismatch { .. })
    ));
}

#[test]
fn timestamp_duration_int64_are_distinct_nominal_types() {
    // Same underlying width (int64 microseconds), same canonical text —
    // but the typed encoder enforces the nominal distinction.
    let text =
        |shape: &Shape, value: &Value| String::from_utf8(encode(shape, value).unwrap()).unwrap();

    assert_eq!(
        text(&Shape::Timestamp, &Value::Timestamp(Timestamp::new(5))),
        "\"5\""
    );
    assert_eq!(
        text(&Shape::Duration, &Value::Duration(Duration::new(5))),
        "\"5\""
    );
    assert_eq!(text(&Shape::Int64, &Value::Int64(5)), "\"5\"");

    for (shape, value) in [
        (&Shape::Timestamp, &Value::Duration(Duration::new(5))),
        (&Shape::Duration, &Value::Timestamp(Timestamp::new(5))),
        (&Shape::Timestamp, &Value::Int64(5)),
        (&Shape::Int64, &Value::Timestamp(Timestamp::new(5))),
        (&Shape::Duration, &Value::Int64(5)),
        (&Shape::Int64, &Value::Duration(Duration::new(5))),
    ] {
        assert!(
            matches!(encode(shape, value), Err(AceError::TypeMismatch { .. })),
            "{} must reject {}",
            shape.type_name(),
            value.type_tag()
        );
    }
}

#[test]
fn nested_option_unit_and_null_are_representable() {
    // Phase 1A: the Phase 1 prohibitions are gone. Any legal AFIDL type may
    // sit under an option because Option is an explicit tagged object.
    assert!(
        Shape::Option(Box::new(Shape::Option(Box::new(Shape::Int32))))
            .validate()
            .is_ok()
    );
    assert!(Shape::Option(Box::new(Shape::Unit)).validate().is_ok());
    assert!(Shape::Option(Box::new(Shape::Null)).validate().is_ok());
}

#[test]
fn none_is_byte_distinguishable_from_every_some_form() {
    let text =
        |shape: &Shape, value: &Value| String::from_utf8(encode(shape, value).unwrap()).unwrap();

    let none = text(
        &Shape::Option(Box::new(Shape::String)),
        &Value::Option(None),
    );
    let some_null = text(
        &Shape::Option(Box::new(Shape::Null)),
        &Value::Option(Some(Box::new(Value::Null))),
    );
    let some_unit = text(
        &Shape::Option(Box::new(Shape::Unit)),
        &Value::Option(Some(Box::new(Value::Unit))),
    );
    let some_none = text(
        &Shape::Option(Box::new(Shape::Option(Box::new(Shape::String)))),
        &Value::Option(Some(Box::new(Value::Option(None)))),
    );

    assert_eq!(none, "{\"none\":null}");
    assert_eq!(some_null, "{\"some\":null}");
    assert_eq!(some_unit, "{\"some\":null}");
    assert_eq!(some_none, "{\"some\":{\"none\":null}}");

    // The §六 assertions: None is byte-distinguishable from every Some form.
    assert_ne!(none, some_null, "None must not share bytes with Some(null)");
    assert_ne!(none, some_unit, "None must not share bytes with Some(unit)");
    assert_ne!(none, some_none, "None must not share bytes with Some(None)");
}

#[test]
fn decoder_accepts_only_tagged_options() {
    let shape = Shape::Option(Box::new(Shape::String));

    // Canonical forms decode.
    let none = decode(&shape, b"{\"none\":null}").unwrap();
    assert_eq!(none, Value::Option(None));
    let some = decode(&shape, b"{\"some\":\"x\"}").unwrap();
    assert_eq!(
        some,
        Value::Option(Some(Box::new(Value::String("x".into()))))
    );

    // Null sugar and malformed tagged objects are rejected, never repaired.
    for (raw, kind) in [
        (&b"null"[..], "non_canonical_encoding"),
        (&b"{}"[..], "invalid_option_encoding"),
        (&b"{\"none\":1}"[..], "non_canonical_encoding"),
        (&b"{\"none\":\"x\"}"[..], "invalid_option_encoding"),
        (&b"{\"foo\":\"x\"}"[..], "invalid_option_encoding"),
        (
            &b"{\"none\":null,\"some\":\"x\"}"[..],
            "invalid_option_encoding",
        ),
        (
            &b"{\"some\":\"x\",\"none\":null}"[..],
            "non_canonical_encoding",
        ),
        (
            &b"{\"none\":null,\"none\":null}"[..],
            "non_canonical_encoding",
        ),
    ] {
        let err = decode(&shape, raw).expect_err("must reject non-canonical option");
        assert_eq!(
            err.kind(),
            kind,
            "decoding {} produced {err} instead of {kind}",
            String::from_utf8_lossy(raw)
        );
    }
}

#[test]
fn nfc_equivalent_values_encode_identically() {
    // Precomposed vs decomposed input — same canonical bytes.
    let pre = encode(&Shape::String, &Value::String("é".into())).unwrap();
    let decomp = encode(&Shape::String, &Value::String("e\u{0301}".into())).unwrap();
    assert_eq!(pre, decomp);

    // NFC-equivalent map keys are rejected instead of last-write-wins.
    let map_shape = Shape::Map(Box::new(Shape::Int32));
    let collided = Value::Map(vec![
        ("é".into(), Value::Int32(1)),
        ("e\u{0301}".into(), Value::Int32(2)),
    ]);
    assert!(matches!(
        encode(&map_shape, &collided),
        Err(AceError::DuplicateNormalizedKey { .. })
    ));
}

/// Keep the decimal import honest: Phase 1A does not touch decimal rules.
#[test]
fn decimal_rules_unchanged() {
    assert_eq!(Decimal::parse("1.250").unwrap().as_str(), "1.25");
    assert_eq!(Decimal::parse("-0").unwrap().as_str(), "0");
}
