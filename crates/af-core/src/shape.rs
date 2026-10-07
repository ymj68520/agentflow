//! Structural type shapes: the machine-readable skeleton used by ACE1 to
//! deterministically encode AgentFlow v1 protocol objects.
//!
//! A `Shape` is *nominal at the boundary* (protocol objects are declared with an
//! explicit name) and *structural inside records*. Field order in a `RecordShape`
//! is declaration order for human readability; ACE1 always emits record fields
//! sorted by unsigned UTF-8 bytes of the NFC-normalized field name, so
//! declaration order never affects canonical bytes.

use crate::error::AceError;

/// A closed ACE1 type shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shape {
    /// The empty product: a single value with no information.
    Unit,
    /// The JSON-null primitive. Distinct from [`Shape::Unit`] and from a missing
    /// record field; the distinction is enforced by the typed encoder.
    Null,
    Bool,
    Int32,
    UInt32,
    Int64,
    UInt64,
    Decimal,
    Float64,
    /// Unicode text. Encoded NFC-normalized UTF-8.
    String,
    /// Opaque bytes. Encoded unpadded base64url.
    Bytes,
    /// Nominal int64 microseconds. Not interchangeable with [`Shape::Int64`] or
    /// [`Shape::Duration`].
    Timestamp,
    /// Nominal int64 microseconds. Not interchangeable with [`Shape::Int64`] or
    /// [`Shape::Timestamp`].
    Duration,
    /// A closed enumeration; values are NFC-normalized names.
    Enum(EnumShape),
    /// An ordered sequence.
    List(Box<Shape>),
    /// `Some(v)` encodes as the tagged object `{"some":v}`; `None` encodes
    /// as the tagged object `{"none":null}`. Any legal AFIDL type may sit
    /// under an option, including unit, null and nested options.
    Option(Box<Shape>),
    /// String-keyed map. Keys are NFC-normalized, must be unique after
    /// normalization, and are emitted sorted by unsigned UTF-8 bytes.
    Map(Box<Shape>),
    /// A closed record with declared fields. All declared fields are mandatory
    /// in canonical bytes; there is no "absent field" state.
    Record(RecordShape),
    /// A closed tagged union with exactly one case selected at a time.
    Variant(VariantShape),
}

/// A closed enumeration shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumShape {
    pub name: String,
    pub values: Vec<String>,
}

/// A record shape with declared fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordShape {
    pub name: String,
    pub fields: Vec<FieldShape>,
}

/// One declared record field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldShape {
    pub name: String,
    pub shape: Shape,
}

/// A closed variant shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantShape {
    pub name: String,
    pub cases: Vec<CaseShape>,
}

/// One variant case and its payload shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseShape {
    pub name: String,
    pub payload: Shape,
}

impl EnumShape {
    pub fn new(
        name: impl Into<String>,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            name: name.into(),
            values: values.into_iter().map(Into::into).collect(),
        }
    }
}

impl RecordShape {
    pub fn new(
        name: impl Into<String>,
        fields: impl IntoIterator<Item = (impl Into<String>, Shape)>,
    ) -> Self {
        Self {
            name: name.into(),
            fields: fields
                .into_iter()
                .map(|(name, shape)| FieldShape {
                    name: name.into(),
                    shape,
                })
                .collect(),
        }
    }
}

impl VariantShape {
    pub fn new(
        name: impl Into<String>,
        cases: impl IntoIterator<Item = (impl Into<String>, Shape)>,
    ) -> Self {
        Self {
            name: name.into(),
            cases: cases
                .into_iter()
                .map(|(name, payload)| CaseShape {
                    name: name.into(),
                    payload,
                })
                .collect(),
        }
    }
}

impl Shape {
    /// Human-facing name used in diagnostics; stable, lowercase, not a wire format.
    pub fn type_name(&self) -> String {
        match self {
            Shape::Unit => "unit".into(),
            Shape::Null => "null".into(),
            Shape::Bool => "bool".into(),
            Shape::Int32 => "int32".into(),
            Shape::UInt32 => "uint32".into(),
            Shape::Int64 => "int64".into(),
            Shape::UInt64 => "uint64".into(),
            Shape::Decimal => "decimal".into(),
            Shape::Float64 => "float64".into(),
            Shape::String => "string".into(),
            Shape::Bytes => "bytes".into(),
            Shape::Timestamp => "timestamp".into(),
            Shape::Duration => "duration".into(),
            Shape::Enum(e) => format!("enum<{}>", e.name),
            Shape::List(item) => format!("list<{}>", item.type_name()),
            Shape::Option(item) => format!("option<{}>", item.type_name()),
            Shape::Map(value) => format!("map<string,{}>", value.type_name()),
            Shape::Record(r) => format!("record<{}>", r.name),
            Shape::Variant(v) => format!("variant<{}>", v.name),
        }
    }

    /// Validates the shape itself: NFC-canonical names, unique names, closed
    /// sets, and representability restrictions (no nested `option`, no
    /// `option<unit>` / `option<null>`).
    pub fn validate(&self) -> Result<(), AceError> {
        match self {
            Shape::Enum(e) => {
                let mut seen = Vec::new();
                for value in &e.values {
                    let normalized = nfc(value);
                    if *value != normalized {
                        return Err(AceError::invalid_shape(format!(
                            "enum<{}> value {:?} is not NFC-normalized",
                            e.name, value
                        )));
                    }
                    if seen.contains(value) {
                        return Err(AceError::invalid_shape(format!(
                            "enum<{}> has duplicate value {:?}",
                            e.name, value
                        )));
                    }
                    seen.push(value.clone());
                }
                Ok(())
            }
            // v1 (Phase 1A ratification): options are explicit tagged objects
            // ({"none":null} / {"some":v}), so ANY legal AFIDL type may sit
            // under an option, including unit, null and nested options.
            Shape::List(item) | Shape::Map(item) | Shape::Option(item) => item.validate(),
            Shape::Record(r) => {
                let mut names: Vec<&str> = Vec::new();
                for field in &r.fields {
                    let normalized = nfc(&field.name);
                    if field.name != normalized {
                        return Err(AceError::invalid_shape(format!(
                            "record<{}> field {:?} is not NFC-normalized",
                            r.name, field.name
                        )));
                    }
                    if names.contains(&field.name.as_str()) {
                        return Err(AceError::invalid_shape(format!(
                            "record<{}> has duplicate field {:?}",
                            r.name, field.name
                        )));
                    }
                    names.push(&field.name);
                    field.shape.validate()?;
                }
                Ok(())
            }
            Shape::Variant(v) => {
                let mut names: Vec<&str> = Vec::new();
                for case in &v.cases {
                    let normalized = nfc(&case.name);
                    if case.name != normalized {
                        return Err(AceError::invalid_shape(format!(
                            "variant<{}> case {:?} is not NFC-normalized",
                            v.name, case.name
                        )));
                    }
                    if names.contains(&case.name.as_str()) {
                        return Err(AceError::invalid_shape(format!(
                            "variant<{}> has duplicate case {:?}",
                            v.name, case.name
                        )));
                    }
                    names.push(&case.name);
                    case.payload.validate()?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

/// NFC-normalizes a string. ACE1 stores and emits only NFC text.
pub(crate) fn nfc(s: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    s.nfc().collect()
}

/// Returns true if the string is already NFC.
pub(crate) fn is_nfc(s: &str) -> bool {
    use unicode_normalization::UnicodeNormalization;
    s.chars().nfc().eq(s.chars())
}
