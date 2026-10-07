//! The ACE1 typed canonical encoder.
//!
//! ACE1 is AgentFlow v1's deterministic, schema-directed, UTF-8 canonical
//! encoding — a strict JSON subset. It is *not* `JSON.stringify`, not
//! "sorted-key serde_json", and not a generic JSON canonicalizer: every byte
//! is determined by the declared [`Shape`] and the typed [`Value`].
//!
//! Frozen invariants materialized here (see `spec/encoding/ace1-grammar.md`):
//!
//! - machine numeric primitives (`int32/uint32/int64/uint64/decimal`,
//!   `timestamp/duration`) encode as canonical decimal text **inside JSON
//!   strings**; JSON numbers are never the canonical numeric authority and no
//!   value passes through IEEE-754 except `float64` itself;
//! - `float64` encodes as 16 lowercase hex characters of the big-endian
//!   IEEE-754 binary64 bit pattern; NaN/±Inf are rejected; `-0` → `+0`;
//! - strings encode NFC-normalized UTF-8 with a fixed minimal escape table;
//! - map keys are NFC-normalized, must be unique after normalization, and are
//!   emitted sorted by unsigned UTF-8 bytes; record fields are emitted in the
//!   same canonical key order;
//! - `bytes` encode as unpadded base64url;
//! - `option<T>`: `None` is the tagged object `{"none":null}`, `Some(v)` is
//!   the tagged object `{"some":v}` (Phase 1A ratification);
//! - records carry every declared field — there is no "absent field" state in
//!   canonical bytes;
//! - closed enums and closed variants reject unknown values/cases;
//! - no insignificant whitespace anywhere.
//!
//! The encoder is pure: same `(shape, value)` in, byte-for-byte same `Vec<u8>`
//! out, with no hidden state, clock, locale, or randomness.

use crate::base64url;
use crate::error::AceError;
use crate::shape::{FieldShape, Shape, nfc};
use crate::value::Value;

/// Encodes a typed value against its declared shape into canonical ACE1 bytes
/// (UTF-8).
///
/// # Errors
/// Returns a structured [`AceError`] for shape/value mismatches, out-of-range
/// or non-finite values, NFC-colliding or duplicate keys, unknown enum values
/// or variant cases, and invalid shapes.
pub fn encode(shape: &Shape, value: &Value) -> Result<Vec<u8>, AceError> {
    shape.validate()?;
    let mut out = String::new();
    write_value(&mut out, shape, value)?;
    Ok(out.into_bytes())
}

/// Convenience wrapper returning the canonical bytes as a UTF-8 `String`.
pub fn encode_to_string(shape: &Shape, value: &Value) -> Result<String, AceError> {
    shape.validate()?;
    let mut out = String::new();
    write_value(&mut out, shape, value)?;
    Ok(out)
}

fn write_value(out: &mut String, shape: &Shape, value: &Value) -> Result<(), AceError> {
    use std::fmt::Write as _;
    match (shape, value) {
        (Shape::Unit, Value::Unit) | (Shape::Null, Value::Null) => out.push_str("null"),
        (Shape::Bool, Value::Bool(b)) => out.push_str(if *b { "true" } else { "false" }),
        (Shape::Int32, Value::Int32(v)) => write_canonical_int(out, *v),
        (Shape::UInt32, Value::UInt32(v)) => write_canonical_int(out, *v),
        (Shape::Int64, Value::Int64(v)) => write_canonical_int(out, *v),
        (Shape::UInt64, Value::UInt64(v)) => write_canonical_int(out, *v),
        (Shape::Decimal, Value::Decimal(d)) => write_escaped_string(out, d.as_str()),
        (Shape::Float64, Value::Float64(f)) => {
            // Canonicalize -0 to +0; emit 16 lowercase hex chars, big-endian.
            let mut bits = f.bits();
            if bits == 0x8000_0000_0000_0000 {
                bits = 0;
            }
            out.push('"');
            write!(out, "{bits:016x}").expect("String formatting is infallible");
            out.push('"');
        }
        (Shape::String, Value::String(s)) => {
            let normalized = nfc(s);
            write_escaped_string(out, &normalized);
        }
        (Shape::Bytes, Value::Bytes(b)) => write_escaped_string(out, &base64url::encode(b)),
        (Shape::Timestamp, Value::Timestamp(t)) => write_canonical_int(out, t.as_micros()),
        (Shape::Duration, Value::Duration(d)) => write_canonical_int(out, d.as_micros()),
        (Shape::Enum(enum_shape), Value::Enum(name)) => {
            if !enum_shape.values.contains(name) {
                return Err(AceError::UnknownEnumValue {
                    value: name.clone(),
                });
            }
            write_escaped_string(out, name);
        }
        (Shape::List(item_shape), Value::List(items)) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_value(out, item_shape, item)?;
            }
            out.push(']');
        }
        // Phase 1A ratification: Option is an explicit single-case tagged
        // object, so None, Some(null), Some(unit) and Some(None) all have
        // distinct wire representations (inner payloads share tokens only
        // where the schema nominally disambiguates).
        (Shape::Option(item_shape), Value::Option(inner)) => match inner {
            None => out.push_str("{\"none\":null}"),
            Some(v) => {
                out.push_str("{\"some\":");
                write_value(out, item_shape, v)?;
                out.push('}');
            }
        },
        (Shape::Map(value_shape), Value::Map(entries)) => {
            // Normalize keys, reject NFC collisions, sort by unsigned UTF-8 bytes.
            let mut normalized: Vec<(String, &Value)> = Vec::with_capacity(entries.len());
            for (key, value) in entries {
                let norm = nfc(key);
                if normalized.iter().any(|(k, _)| *k == norm) {
                    return Err(AceError::DuplicateNormalizedKey { key: norm });
                }
                normalized.push((norm, value));
            }
            normalized.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
            out.push('{');
            for (i, (key, item)) in normalized.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_escaped_string(out, key);
                out.push(':');
                write_value(out, value_shape, item)?;
            }
            out.push('}');
        }
        (Shape::Record(record_shape), Value::Record(entries)) => {
            // Every declared field must be supplied exactly once; nothing else
            // may be supplied. "Missing" is an error state, never an encoding.
            let mut supplied: Vec<(&str, &Value)> = Vec::with_capacity(entries.len());
            for (name, v) in entries {
                if supplied.iter().any(|(n, _)| *n == name.as_str()) {
                    return Err(AceError::DuplicateField {
                        field: name.clone(),
                    });
                }
                if !record_shape.fields.iter().any(|f| f.name == *name) {
                    return Err(AceError::UnknownField {
                        field: name.clone(),
                    });
                }
                supplied.push((name.as_str(), v));
            }
            // Emit in canonical key order over the declared fields.
            let mut ordered: Vec<&FieldShape> = record_shape.fields.iter().collect();
            ordered.sort_by(|a, b| a.name.as_bytes().cmp(b.name.as_bytes()));
            out.push('{');
            for (i, field) in ordered.iter().enumerate() {
                let v = supplied
                    .iter()
                    .find(|(n, _)| *n == field.name.as_str())
                    .map(|(_, v)| *v)
                    .ok_or_else(|| AceError::MissingField {
                        field: field.name.clone(),
                    })?;
                if i > 0 {
                    out.push(',');
                }
                write_escaped_string(out, &field.name);
                out.push(':');
                write_value(out, &field.shape, v)?;
            }
            out.push('}');
        }
        (Shape::Variant(variant_shape), Value::Variant { case, value }) => {
            let case_shape = variant_shape
                .cases
                .iter()
                .find(|c| c.name == *case)
                .ok_or_else(|| AceError::UnknownVariantCase { case: case.clone() })?;
            out.push('{');
            write_escaped_string(out, case);
            out.push(':');
            write_value(out, &case_shape.payload, value)?;
            out.push('}');
        }
        (expected_shape, actual_value) => {
            return Err(AceError::TypeMismatch {
                expected: expected_shape.type_name(),
                actual: actual_value.type_tag(),
            });
        }
    }
    Ok(())
}

/// Canonical machine-integer text: Rust's integer `Display` for the exact
/// integer types is already canonical (no `+`, no leading zeros, ASCII only),
/// and the value never passes through a floating-point type.
fn write_canonical_int<T: std::fmt::Display>(out: &mut String, v: T) {
    out.push('"');
    out.push_str(&v.to_string());
    out.push('"');
}

/// Deterministic JSON string escaping:
/// `"` → `\"`, `\` → `\\`, `\b \t \n \f \r` short forms, other characters
/// below U+0020 as `\u00xx` with lowercase hex; every other character is
/// emitted as raw (NFC) UTF-8.
pub(crate) fn write_escaped_string(out: &mut String, s: &str) {
    use std::fmt::Write as _;
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{0009}' => out.push_str("\\t"),
            '\u{000A}' => out.push_str("\\n"),
            '\u{000C}' => out.push_str("\\f"),
            '\u{000D}' => out.push_str("\\r"),
            c if (c as u32) < 0x20 => {
                write!(out, "\\u{:04x}", c as u32).expect("String formatting is infallible");
            }
            c => out.push(c),
        }
    }
    out.push('"');
}
