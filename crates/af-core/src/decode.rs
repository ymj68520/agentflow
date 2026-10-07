//! The ACE1 strict decoder: canonical bytes → typed value.
//!
//! The decoder is the mirror of the encoder under the same [`Shape`] model and
//! enforces a stronger contract than a permissive JSON parser:
//!
//! ```text
//! decode(canonical bytes) → typed Value
//! re-encode               → exact same bytes
//! ```
//!
//! Anything that is *valid-looking but not canonical* is **rejected**, never
//! auto-repaired: insignificant whitespace, unsorted or duplicate object keys,
//! non-NFC text, non-canonical escapes (`\/`, `\uXXXX` for non-control
//! characters, uppercase hex), bare JSON numbers, padded or standard-alphabet
//! base64url, `-0` float bits, non-canonical decimal text, trailing bytes,
//! unknown record fields, missing record fields, unknown enum values, unknown
//! variant cases.
//!
//! This is deliberately a different concept from a permissive input parser:
//! canonicalization happens in the typed encoder, not here.

use crate::base64url;
use crate::decimal::Decimal;
use crate::error::AceError;
use crate::primitive::{parse_int32_text, parse_int64_text, parse_uint32_text, parse_uint64_text};
use crate::shape::{FieldShape, Shape, is_nfc};
use crate::value::{Duration, Float64, Timestamp, Value};

/// Decodes strict canonical ACE1 bytes into a typed value against `shape`.
///
/// # Errors
/// Returns a structured [`AceError`] for any structural, canonical-form,
/// range, or shape violation. Non-canonical input is rejected, not repaired.
pub fn decode(shape: &Shape, bytes: &[u8]) -> Result<Value, AceError> {
    shape.validate()?;
    let input = std::str::from_utf8(bytes).map_err(|_| AceError::InvalidUnicode {
        detail: "input is not valid UTF-8".into(),
    })?;
    let mut parser = Parser { input, pos: 0 };
    let value = parser.parse_value(shape)?;
    if parser.pos != input.len() {
        return Err(parser.non_canonical("trailing bytes after canonical value"));
    }
    Ok(value)
}

/// Recursive-descent parser over canonical ACE1 text.
struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

/// One parsed object member: the (unescaped, NFC-validated) key plus the byte
/// span of its value inside `input`.
struct Member {
    key: String,
    value: (usize, usize),
}

impl Parser<'_> {
    fn non_canonical(&self, detail: impl Into<String>) -> AceError {
        AceError::NonCanonicalEncoding {
            detail: format!("{} (at byte {})", detail.into(), self.pos),
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.as_bytes().get(self.pos).copied()
    }

    fn next_char(&mut self) -> Option<char> {
        let c = self.input[self.pos..].chars().next()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn expect_byte(&mut self, b: u8) -> Result<(), AceError> {
        if self.peek() == Some(b) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.non_canonical(format!("expected {:?}", b as char)))
        }
    }

    fn expect_literal(&mut self, literal: &str) -> Result<(), AceError> {
        if self.input[self.pos..].starts_with(literal) {
            self.pos += literal.len();
            Ok(())
        } else {
            Err(self.non_canonical(format!("expected literal {literal:?}")))
        }
    }

    fn parse_value(&mut self, shape: &Shape) -> Result<Value, AceError> {
        match shape {
            Shape::Unit => {
                self.expect_literal("null")?;
                Ok(Value::Unit)
            }
            Shape::Null => {
                self.expect_literal("null")?;
                Ok(Value::Null)
            }
            Shape::Bool => {
                if self.input[self.pos..].starts_with("true") {
                    self.pos += 4;
                    Ok(Value::Bool(true))
                } else if self.input[self.pos..].starts_with("false") {
                    self.pos += 5;
                    Ok(Value::Bool(false))
                } else {
                    Err(self.non_canonical("expected bool literal true/false"))
                }
            }
            Shape::Int32 => {
                let s = self.parse_string()?;
                Ok(Value::Int32(parse_int32_text(&s)?))
            }
            Shape::UInt32 => {
                let s = self.parse_string()?;
                Ok(Value::UInt32(parse_uint32_text(&s)?))
            }
            Shape::Int64 => {
                let s = self.parse_string()?;
                Ok(Value::Int64(parse_int64_text(&s)?))
            }
            Shape::UInt64 => {
                let s = self.parse_string()?;
                Ok(Value::UInt64(parse_uint64_text(&s)?))
            }
            Shape::Decimal => {
                let s = self.parse_string()?;
                let d = Decimal::parse(&s)?;
                if d.as_str() != s {
                    return Err(
                        self.non_canonical(format!("decimal {s:?} is not in canonical form"))
                    );
                }
                Ok(Value::Decimal(d))
            }
            Shape::Float64 => {
                let s = self.parse_string()?;
                self.parse_float_hex(&s)
            }
            Shape::String => Ok(Value::String(self.parse_string()?)),
            Shape::Bytes => {
                let s = self.parse_string()?;
                Ok(Value::Bytes(base64url::decode(&s)?))
            }
            Shape::Timestamp => {
                let s = self.parse_string()?;
                Ok(Value::Timestamp(Timestamp::new(parse_int64_text(&s)?)))
            }
            Shape::Duration => {
                let s = self.parse_string()?;
                Ok(Value::Duration(Duration::new(parse_int64_text(&s)?)))
            }
            Shape::Enum(enum_shape) => {
                let s = self.parse_string()?;
                if !enum_shape.values.contains(&s) {
                    return Err(AceError::UnknownEnumValue { value: s });
                }
                Ok(Value::Enum(s))
            }
            Shape::Option(inner) => {
                if self.input[self.pos..].starts_with("null") {
                    self.pos += 4;
                    Ok(Value::Option(None))
                } else {
                    Ok(Value::Option(Some(Box::new(self.parse_value(inner)?))))
                }
            }
            Shape::List(item_shape) => {
                self.expect_byte(b'[')?;
                let mut items = Vec::new();
                if self.peek() == Some(b']') {
                    self.pos += 1;
                    return Ok(Value::List(items));
                }
                loop {
                    items.push(self.parse_value(item_shape)?);
                    match self.peek() {
                        Some(b',') => self.pos += 1,
                        Some(b']') => {
                            self.pos += 1;
                            return Ok(Value::List(items));
                        }
                        _ => return Err(self.non_canonical("expected ',' or ']' in list")),
                    }
                }
            }
            Shape::Map(value_shape) => {
                let members = self.parse_members()?;
                let mut entries = Vec::with_capacity(members.len());
                for m in &members {
                    let value = self.parse_value_span(m.value, value_shape)?;
                    entries.push((m.key.clone(), value));
                }
                Ok(Value::Map(entries))
            }
            Shape::Record(record_shape) => {
                let members = self.parse_members()?;
                // Canonical record bytes carry exactly the declared fields in
                // canonical (unsigned UTF-8 byte) key order.
                let mut sorted_fields: Vec<&FieldShape> = record_shape.fields.iter().collect();
                sorted_fields.sort_by(|a, b| a.name.as_bytes().cmp(b.name.as_bytes()));
                if members.len() != sorted_fields.len() {
                    if members.len() < sorted_fields.len() {
                        let missing = sorted_fields
                            .iter()
                            .find(|f| !members.iter().any(|m| m.key == f.name))
                            .unwrap();
                        return Err(AceError::MissingField {
                            field: missing.name.clone(),
                        });
                    }
                    let extra = members
                        .iter()
                        .find(|m| !sorted_fields.iter().any(|f| f.name == m.key))
                        .unwrap();
                    return Err(AceError::UnknownField {
                        field: extra.key.clone(),
                    });
                }
                let mut entries = Vec::with_capacity(members.len());
                for (m, field) in members.iter().zip(sorted_fields.iter()) {
                    if m.key != field.name {
                        return Err(AceError::UnknownField {
                            field: m.key.clone(),
                        });
                    }
                    let value = self.parse_value_span(m.value, &field.shape)?;
                    entries.push((m.key.clone(), value));
                }
                Ok(Value::Record(entries))
            }
            Shape::Variant(variant_shape) => {
                let members = self.parse_members()?;
                if members.len() != 1 {
                    return Err(self.non_canonical("variant must encode exactly one case member"));
                }
                let m = &members[0];
                let case_shape = variant_shape
                    .cases
                    .iter()
                    .find(|c| c.name == m.key)
                    .ok_or_else(|| AceError::UnknownVariantCase {
                        case: m.key.clone(),
                    })?;
                let value = self.parse_value_span(m.value, &case_shape.payload)?;
                Ok(Value::Variant {
                    case: m.key.clone(),
                    value: Box::new(value),
                })
            }
        }
    }

    /// Parses a float64 hex literal: exactly 16 lowercase hex characters,
    /// finite, canonical (no `-0`).
    fn parse_float_hex(&mut self, s: &str) -> Result<Value, AceError> {
        let bytes = s.as_bytes();
        if bytes.len() != 16
            || !bytes
                .iter()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return Err(self.non_canonical("float64 must be exactly 16 lowercase hex characters"));
        }
        let bits = u64::from_str_radix(s, 16)
            .map_err(|_| self.non_canonical("float64 hex literal is invalid"))?;
        if bits & 0x7FF0_0000_0000_0000 == 0x7FF0_0000_0000_0000 {
            return Err(AceError::NonFiniteFloat);
        }
        if bits == 0x8000_0000_0000_0000 {
            return Err(self.non_canonical("float64 -0 must be canonicalized to +0"));
        }
        Ok(Value::Float64(Float64::from_bits(bits)?))
    }

    /// Parses `"..."` with canonical escapes only and NFC-mandatory content.
    #[allow(
        clippy::many_single_char_names,
        reason = "c/e/h/d/v are the natural names in escape scanning"
    )]
    fn parse_string(&mut self) -> Result<String, AceError> {
        self.expect_byte(b'"')?;
        let mut out = String::new();
        loop {
            let c = self
                .next_char()
                .ok_or_else(|| self.non_canonical("unterminated string"))?;
            match c {
                '"' => break,
                '\\' => {
                    let e = self
                        .next_char()
                        .ok_or_else(|| self.non_canonical("unterminated escape"))?;
                    match e {
                        '"' => out.push('"'),
                        '\\' => out.push('\\'),
                        'b' => out.push('\u{0008}'),
                        'f' => out.push('\u{000C}'),
                        'n' => out.push('\u{000A}'),
                        'r' => out.push('\u{000D}'),
                        't' => out.push('\u{0009}'),
                        'u' => {
                            let mut v: u32 = 0;
                            for _ in 0..4 {
                                let h = self
                                    .next_char()
                                    .ok_or_else(|| self.non_canonical("unterminated \\u escape"))?;
                                let d = match h {
                                    '0'..='9' => h as u32 - '0' as u32,
                                    'a'..='f' => h as u32 - 'a' as u32 + 10,
                                    _ => {
                                        return Err(self.non_canonical(
                                            "\\u escapes require lowercase hex digits",
                                        ));
                                    }
                                };
                                v = v * 16 + d;
                            }
                            if v >= 0x20 {
                                return Err(self.non_canonical(
                                    "\\u escapes are only canonical for control characters",
                                ));
                            }
                            if matches!(v, 0x8 | 0x9 | 0xA | 0xC | 0xD) {
                                return Err(self.non_canonical(
                                    "control characters with short escapes must use them",
                                ));
                            }
                            out.push(char::from_u32(v).expect("v < 0x20 is a valid scalar"));
                        }
                        _ => return Err(self.non_canonical("invalid escape sequence")),
                    }
                }
                c if (c as u32) < 0x20 => {
                    return Err(self.non_canonical("raw control character inside string"));
                }
                c => out.push(c),
            }
        }
        if !is_nfc(&out) {
            return Err(AceError::InvalidUnicode {
                detail: format!("string {out:?} is not NFC-normalized"),
            });
        }
        Ok(out)
    }

    /// Parses an object's members (after consuming nothing; the caller has
    /// positioned at `{`). Enforces canonical member structure: string keys,
    /// no whitespace, strictly ascending unsigned-byte key order (which also
    /// rejects duplicates), NFC keys.
    fn parse_members(&mut self) -> Result<Vec<Member>, AceError> {
        self.expect_byte(b'{')?;
        let mut members: Vec<Member> = Vec::new();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(members);
        }
        loop {
            let key = self.parse_string()?;
            if let Some(prev) = members.last() {
                if prev.key.as_bytes() >= key.as_bytes() {
                    return Err(self.non_canonical(
                        "object keys are not in strictly ascending unsigned UTF-8 byte order",
                    ));
                }
            }
            self.expect_byte(b':')?;
            let start = self.pos;
            self.skip_value()?;
            let value = (start, self.pos);
            members.push(Member { key, value });
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(members);
                }
                _ => return Err(self.non_canonical("expected ',' or '}' in object")),
            }
        }
    }

    /// Structurally skips one canonical value (validating canonical form) to
    /// locate its byte span. Bare JSON numbers are rejected: ACE1 encodes all
    /// machine numerics as strings.
    fn skip_value(&mut self) -> Result<(), AceError> {
        match self.peek() {
            Some(b'"') => {
                self.parse_string()?;
                Ok(())
            }
            Some(b't') => self.expect_literal("true"),
            Some(b'f') => self.expect_literal("false"),
            Some(b'n') => self.expect_literal("null"),
            Some(b'[') => {
                self.pos += 1;
                if self.peek() == Some(b']') {
                    self.pos += 1;
                    return Ok(());
                }
                loop {
                    self.skip_value()?;
                    match self.peek() {
                        Some(b',') => self.pos += 1,
                        Some(b']') => {
                            self.pos += 1;
                            return Ok(());
                        }
                        _ => return Err(self.non_canonical("expected ',' or ']' in list")),
                    }
                }
            }
            Some(b'{') => {
                self.parse_members()?;
                Ok(())
            }
            Some(c) if c == b'-' || c.is_ascii_digit() => Err(self.non_canonical(
                "bare JSON numbers are not part of ACE1; machine numerics are strings",
            )),
            _ => Err(self.non_canonical("expected a canonical ACE1 value")),
        }
    }

    /// Parses a value previously spanned by [`Parser::parse_members`].
    fn parse_value_span(&mut self, span: (usize, usize), shape: &Shape) -> Result<Value, AceError> {
        let saved = self.pos;
        self.pos = span.0;
        let value = self.parse_value(shape)?;
        if self.pos != span.1 {
            return Err(self.non_canonical("value did not consume exactly its span"));
        }
        self.pos = saved;
        Ok(value)
    }
}
