//! The five-way distinction test: `field missing`, `explicit null`, `unit`,
//! `Option::None`, `Option::Some(v)` must never be collapsed by serialization
//! convenience. ACE1 is schema-directed, so the typed encoder — not the bytes
//! alone — resolves these states, and the errors below prove the boundaries.

use af_core::{AceError, Duration, RecordShape, Shape, Timestamp, Value, encode};

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

    assert_eq!(none_bytes, b"{\"value\":null}");
    assert_eq!(some_bytes, b"{\"value\":\"x\"}");
    assert_ne!(none_bytes, some_bytes);
}

#[test]
fn unit_and_null_are_distinct_nominal_types() {
    // Byte-level: both render as `null` in their own schema position (see
    // open question OQ-2)...
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
fn option_unit_and_option_null_are_unrepresentable() {
    assert!(
        Shape::Option(Box::new(Shape::Option(Box::new(Shape::Int32))))
            .validate()
            .is_err()
    );
    assert!(Shape::Option(Box::new(Shape::Unit)).validate().is_err());
    assert!(Shape::Option(Box::new(Shape::Null)).validate().is_err());
    // A valid option position is fine.
    assert!(Shape::Option(Box::new(Shape::Int32)).validate().is_ok());
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
