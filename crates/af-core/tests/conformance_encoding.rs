//! ACE1 conformance harness: consumes the FROZEN golden/negative corpus under
//! `spec/conformance/encoding/`.
//!
//! The expected values in those files were generated once by an independent
//! implementation (`tools/gen-conformance.py`, Python) and are physically
//! frozen in the repository. These tests never generate expected values from
//! the implementation under test; they only compare against the frozen bytes.
//!
//! Note: the fixture *harness* uses `serde_json::Value` to walk the fixture
//! grammar. That is test tooling only — the normative library model
//! (`af_core::value::Value`, `af_core::shape::Shape`) contains no dynamic
//! values whatsoever.

use af_core::{AceError, Decimal, Duration, Shape, Timestamp, Value, decode, encode};
use serde::Deserialize;
use serde_json::Value as Json;
use std::path::{Path, PathBuf};

fn spec_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/conformance/encoding")
}

fn load_json(name: &str) -> Json {
    let path = spec_dir().join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("cannot parse {}: {e}", path.display()))
}

// ---------------------------------------------------------------------------
// Fixture grammar → typed model
// ---------------------------------------------------------------------------

fn shape_from_fx(v: &Json) -> Shape {
    match v {
        Json::String(name) => match name.as_str() {
            "unit" => Shape::Unit,
            "null" => Shape::Null,
            "bool" => Shape::Bool,
            "int32" => Shape::Int32,
            "uint32" => Shape::UInt32,
            "int64" => Shape::Int64,
            "uint64" => Shape::UInt64,
            "decimal" => Shape::Decimal,
            "float64" => Shape::Float64,
            "string" => Shape::String,
            "bytes" => Shape::Bytes,
            "timestamp" => Shape::Timestamp,
            "duration" => Shape::Duration,
            other => panic!("unknown primitive shape {other:?}"),
        },
        Json::Object(map) => {
            let (kind, spec) = map.iter().next().expect("shape object has one key");
            match kind.as_str() {
                "enum" => Shape::Enum(af_core::EnumShape::new(
                    spec["name"].as_str().expect("enum name"),
                    spec["values"]
                        .as_array()
                        .expect("enum values")
                        .iter()
                        .map(|v| v.as_str().expect("enum value")),
                )),
                "list" => Shape::List(Box::new(shape_from_fx(spec))),
                "option" => Shape::Option(Box::new(shape_from_fx(spec))),
                "map" => Shape::Map(Box::new(shape_from_fx(spec))),
                "record" => Shape::Record(af_core::RecordShape::new(
                    spec["name"].as_str().expect("record name"),
                    spec["fields"]
                        .as_array()
                        .expect("record fields")
                        .iter()
                        .map(|f| {
                            (
                                f["field"].as_str().expect("field name"),
                                shape_from_fx(&f["type"]),
                            )
                        }),
                )),
                "variant" => Shape::Variant(af_core::VariantShape::new(
                    spec["name"].as_str().expect("variant name"),
                    spec["cases"]
                        .as_array()
                        .expect("variant cases")
                        .iter()
                        .map(|c| {
                            (
                                c["case"].as_str().expect("case name"),
                                shape_from_fx(&c["type"]),
                            )
                        }),
                )),
                other => panic!("unknown shape kind {other:?}"),
            }
        }
        other => panic!("unknown shape encoding {other:?}"),
    }
}

fn text_of(v: &Json) -> &str {
    v.as_str().expect("numeric fixture values are strings")
}

/// Builds a typed `Value` from the fixture grammar. Values are self-describing
/// (the tag carries the nominal type), so nominal cross-assignment errors
/// surface later in the encoder — exactly what the negative corpus asserts.
///
/// Integer/decimal/float text is parsed through `af-core`'s own strict
/// primitive parsers, so range and canonicality violations surface with the
/// structured error kinds the negative corpus expects.
fn value_from_fx(v: &Json) -> Result<Value, AceError> {
    let obj = v.as_object().expect("value is a tagged object");
    let (tag, inner) = obj.iter().next().expect("value has one tag");
    Ok(match tag.as_str() {
        "unit" => Value::Unit,
        "null" => Value::Null,
        "bool" => Value::Bool(inner.as_bool().expect("bool")),
        "int32" => Value::Int32(af_core::primitive::parse_int32_text(text_of(inner))?),
        "uint32" => Value::UInt32(af_core::primitive::parse_uint32_text(text_of(inner))?),
        "int64" => Value::Int64(af_core::primitive::parse_int64_text(text_of(inner))?),
        "uint64" => Value::UInt64(af_core::primitive::parse_uint64_text(text_of(inner))?),
        "decimal" => Value::Decimal(Decimal::parse(text_of(inner))?),
        "float64" => {
            let bits =
                u64::from_str_radix(text_of(inner), 16).expect("fixture float bits are valid hex");
            Value::Float64(af_core::Float64::from_bits(bits)?)
        }
        "string" => Value::String(inner.as_str().expect("string").to_string()),
        "bytes" => Value::Bytes(
            inner
                .as_array()
                .expect("byte array")
                .iter()
                .map(|b| u8::try_from(b.as_u64().expect("byte")).expect("fixture byte"))
                .collect(),
        ),
        "timestamp" => Value::Timestamp(Timestamp::new(af_core::primitive::parse_int64_text(
            text_of(inner),
        )?)),
        "duration" => Value::Duration(Duration::new(af_core::primitive::parse_int64_text(
            text_of(inner),
        )?)),
        "enum" => Value::Enum(inner.as_str().expect("enum value").to_string()),
        "list" => Value::List(
            inner
                .as_array()
                .expect("list items")
                .iter()
                .map(value_from_fx)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        "some" => Value::Option(Some(Box::new(value_from_fx(inner)?))),
        "none" => Value::Option(None),
        "map" => Value::Map(
            inner
                .as_array()
                .expect("map entries")
                .iter()
                .map(|e| {
                    (
                        e["key"].as_str().expect("map key").to_string(),
                        value_from_fx(&e["value"]).expect("map value"),
                    )
                })
                .collect(),
        ),
        "record" => Value::Record(
            inner
                .as_array()
                .expect("record fields")
                .iter()
                .map(|e| {
                    (
                        e["field"].as_str().expect("field name").to_string(),
                        value_from_fx(&e["value"]).expect("record value"),
                    )
                })
                .collect(),
        ),
        "variant" => Value::Variant {
            case: inner["case"].as_str().expect("variant case").to_string(),
            value: Box::new(value_from_fx(&inner["value"])?),
        },
        other => panic!("unknown value tag {other:?}"),
    })
}

// ---------------------------------------------------------------------------
// Golden corpus runner
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct GoldenFile {
    #[allow(dead_code)]
    kind: String,
    cases: Vec<GoldenCase>,
}

#[derive(Deserialize)]
struct GoldenCase {
    case_id: String,
    #[allow(dead_code)]
    description: String,
    shape: Json,
    value: Json,
    expected_ace1_utf8: String,
    expected_bytes_hex: String,
    /// When present, names another case whose frozen bytes must be identical —
    /// the machine-checkable form of "equivalent textual input → same bytes".
    #[serde(default)]
    equivalent_to: Option<String>,
}

fn run_golden_file(file_name: &str) {
    let file: GoldenFile = serde_json::from_value(load_json(file_name))
        .unwrap_or_else(|e| panic!("{file_name} is not a valid golden file: {e}"));
    assert!(!file.cases.is_empty(), "{file_name} must not be empty");

    // Determinism: the whole corpus is executed twice; results must be
    // byte-identical across runs.
    let mut first_run: Vec<(String, Vec<u8>)> = Vec::new();

    for case in &file.cases {
        let shape = shape_from_fx(&case.shape);
        let value = value_from_fx(&case.value).unwrap_or_else(|e| panic!("{}: {e}", case.case_id));

        let encoded = encode(&shape, &value)
            .unwrap_or_else(|e| panic!("case {} failed to encode: {e}", case.case_id));

        assert_eq!(
            hex::encode(&encoded),
            case.expected_bytes_hex,
            "case {}: canonical bytes mismatch\nexpected utf8: {}\nactual utf8:   {}",
            case.case_id,
            case.expected_ace1_utf8,
            String::from_utf8_lossy(&encoded)
        );
        assert_eq!(
            encoded,
            case.expected_ace1_utf8.as_bytes(),
            "case {}: expected_ace1_utf8 and expected_bytes_hex disagree with each other",
            case.case_id
        );

        // Round-trip: decode(canonical) → re-encode → exact same bytes.
        let decoded = decode(&shape, &encoded).unwrap_or_else(|e| {
            panic!(
                "case {} failed to decode its own golden bytes: {e}",
                case.case_id
            )
        });
        let re_encoded = encode(&shape, &decoded)
            .unwrap_or_else(|e| panic!("case {} failed to re-encode: {e}", case.case_id));
        assert_eq!(
            re_encoded, encoded,
            "case {}: decode(encode(v)) is not byte-identical",
            case.case_id
        );

        if let Some(equivalent_id) = &case.equivalent_to {
            let expected_equivalent = file
                .cases
                .iter()
                .find(|c| &c.case_id == equivalent_id)
                .unwrap_or_else(|| {
                    panic!(
                        "case {}: equivalent_to references unknown case {equivalent_id}",
                        case.case_id
                    )
                });
            assert_eq!(
                case.expected_bytes_hex, expected_equivalent.expected_bytes_hex,
                "case {}: frozen bytes differ from equivalent case {equivalent_id}",
                case.case_id
            );
        }

        first_run.push((case.case_id.clone(), encoded));
    }

    // Second run must produce identical bytes for every case.
    for case in &file.cases {
        let shape = shape_from_fx(&case.shape);
        let value = value_from_fx(&case.value).unwrap();
        let encoded = encode(&shape, &value).unwrap();
        let (_, previous) = first_run
            .iter()
            .find(|(id, _)| *id == case.case_id)
            .expect("case recorded in first run");
        assert_eq!(
            &encoded, previous,
            "case {}: encoder is not deterministic across runs",
            case.case_id
        );
    }
}

#[test]
fn golden_primitives() {
    run_golden_file("primitive-golden.json");
}

#[test]
fn golden_strings() {
    run_golden_file("string-golden.json");
}

#[test]
fn golden_collections() {
    run_golden_file("collection-golden.json");
}

// ---------------------------------------------------------------------------
// Negative corpus runner
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct NegativeFile {
    #[allow(dead_code)]
    kind: String,
    cases: Vec<NegativeCase>,
}

#[derive(Deserialize)]
struct NegativeCase {
    case_id: String,
    #[allow(dead_code)]
    description: String,
    shape: Json,
    #[serde(default)]
    value: Option<Json>,
    #[serde(default)]
    bytes: Option<String>,
    error_class: String,
    target: String,
    requires_decoder: bool,
}

#[test]
fn negative_encoding() {
    let file: NegativeFile = serde_json::from_value(load_json("negative.json"))
        .expect("negative.json is a valid negative file");
    assert!(!file.cases.is_empty());

    let mut encoder_cases = 0usize;
    let mut decoder_cases = 0usize;

    for case in &file.cases {
        let shape = shape_from_fx(&case.shape);
        match case.target.as_str() {
            "encoder" => {
                encoder_cases += 1;
                assert!(
                    !case.requires_decoder,
                    "encoder case must not require a decoder"
                );
                let value = match value_from_fx(case.value.as_ref().expect("encoder value")) {
                    Ok(v) => v,
                    Err(e) => {
                        assert_eq!(
                            e.kind(),
                            case.error_class,
                            "case {}: construction error kind mismatch ({}: {})",
                            case.case_id,
                            e.kind(),
                            e
                        );
                        continue;
                    }
                };
                match encode(&shape, &value) {
                    Err(e) => assert_eq!(
                        e.kind(),
                        case.error_class,
                        "case {}: encode error kind mismatch ({e})",
                        case.case_id
                    ),
                    Ok(bytes) => panic!(
                        "case {}: expected error {} but encoding succeeded: {:?}",
                        case.case_id,
                        case.error_class,
                        String::from_utf8_lossy(&bytes)
                    ),
                }
            }
            "decoder" => {
                decoder_cases += 1;
                let raw = case.bytes.as_ref().expect("decoder case carries raw bytes");
                match decode(&shape, raw.as_bytes()) {
                    Err(e) => assert_eq!(
                        e.kind(),
                        case.error_class,
                        "case {}: decode error kind mismatch ({e})",
                        case.case_id
                    ),
                    Ok(value) => panic!(
                        "case {}: expected error {} but decoding succeeded: {value:?}",
                        case.case_id, case.error_class
                    ),
                }
            }
            other => panic!("case {}: unknown target {other:?}", case.case_id),
        }
    }

    assert!(
        encoder_cases >= 20,
        "negative corpus must cover encoder cases"
    );
    assert!(
        decoder_cases >= 20,
        "negative corpus must cover decoder cases"
    );
}
