//! The closed, typed machine value model.
//!
//! `Value` is the only input the ACE1 encoder accepts. It is a closed enum:
//! there is no `Any`, no `serde_json::Value`, and no untyped object escape
//! hatch. Nominal distinctions are enforced here and by the encoder:
//!
//! - `Timestamp` and `Duration` are distinct newtypes over `i64` microseconds;
//!   neither is interchangeable with `Value::Int64`.
//! - `Float64` rejects NaN and infinities at construction; `-0.0` is
//!   canonicalized to `+0.0` when encoded.
//! - `Decimal` lives in [`crate::decimal`] and is exact base-10 text, never a
//!   float.
//! - `Value::Unit`, `Value::Null`, `Value::Option(None)` and an absent record
//!   field are four distinct states; the encoder enforces which one each shape
//!   position accepts.

use crate::decimal::Decimal;
use crate::error::AceError;

/// IEEE-754 binary64 machine float. NaN and infinities are rejected at
/// construction; negative zero is permitted in memory but canonicalizes to
/// positive zero when encoded.
#[derive(Debug, Clone, Copy)]
pub struct Float64(f64);

impl Float64 {
    /// Wraps a finite `f64`. Returns [`AceError::NonFiniteFloat`] for NaN and
    /// infinities.
    pub fn new(value: f64) -> Result<Self, AceError> {
        if !value.is_finite() {
            return Err(AceError::NonFiniteFloat);
        }
        Ok(Self(value))
    }

    /// Wraps the exact IEEE-754 bit pattern. NaN and infinity patterns are
    /// rejected; negative zero is accepted and canonicalized on encode.
    pub fn from_bits(bits: u64) -> Result<Self, AceError> {
        let value = f64::from_bits(bits);
        Self::new(value)
    }

    /// The exact IEEE-754 bit pattern (big-endian serialization of these 64
    /// bits is the ACE1 encoding basis).
    pub fn bits(&self) -> u64 {
        self.0.to_bits()
    }

    /// The wrapped value.
    pub fn get(&self) -> f64 {
        self.0
    }
}

impl PartialEq for Float64 {
    /// Compares canonical (negative-zero-normalized) bit patterns so that
    /// `-0.0` and `0.0` are equal values.
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() & 0x7fff_ffff_ffff_ffff == other.0.to_bits() & 0x7fff_ffff_ffff_ffff
            || self.0 == other.0
    }
}

impl Eq for Float64 {}

/// Nominal int64 microseconds since the Unix epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Timestamp(i64);

impl Timestamp {
    pub fn new(microseconds: i64) -> Self {
        Self(microseconds)
    }

    pub fn as_micros(&self) -> i64 {
        self.0
    }
}

/// Nominal int64 microseconds. Deliberately a distinct type from
/// [`Timestamp`]: the two must never be conflated despite sharing a wire width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Duration(i64);

impl Duration {
    pub fn new(microseconds: i64) -> Self {
        Self(microseconds)
    }

    pub fn as_micros(&self) -> i64 {
        self.0
    }
}

/// The closed typed machine value model.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// The empty product. Encodes as `null` but is a distinct *type* from
    /// [`Value::Null`]; the shapes, not the bytes, disambiguate.
    Unit,
    /// Explicit JSON null.
    Null,
    Bool(bool),
    Int32(i32),
    UInt32(u32),
    Int64(i64),
    UInt64(u64),
    Decimal(Decimal),
    Float64(Float64),
    /// Raw Unicode text; NFC normalization happens in the encoder, so
    /// canonically equivalent inputs produce identical canonical bytes.
    String(String),
    Bytes(Vec<u8>),
    Timestamp(Timestamp),
    Duration(Duration),
    /// Closed enum value; validated against the enum's declared value set.
    Enum(String),
    List(Vec<Value>),
    /// `Some(v)` / `None`. `None` encodes as explicit `null`.
    Option(Option<Box<Value>>),
    /// String-keyed map; entry order in memory is irrelevant, the encoder
    /// canonicalizes (NFC + sort + duplicate rejection).
    Map(Vec<(String, Value)>),
    /// Record instance; entries are (field name, value) pairs validated
    /// against the record shape.
    Record(Vec<(String, Value)>),
    /// Closed variant: one case name plus payload value.
    Variant {
        case: String,
        value: Box<Value>,
    },
}

impl Value {
    /// The nominal type tag used in diagnostics and negative-corpus reporting.
    pub fn type_tag(&self) -> String {
        match self {
            Value::Unit => "unit".into(),
            Value::Null => "null".into(),
            Value::Bool(_) => "bool".into(),
            Value::Int32(_) => "int32".into(),
            Value::UInt32(_) => "uint32".into(),
            Value::Int64(_) => "int64".into(),
            Value::UInt64(_) => "uint64".into(),
            Value::Decimal(_) => "decimal".into(),
            Value::Float64(_) => "float64".into(),
            Value::String(_) => "string".into(),
            Value::Bytes(_) => "bytes".into(),
            Value::Timestamp(_) => "timestamp".into(),
            Value::Duration(_) => "duration".into(),
            Value::Enum(_) => "enum".into(),
            Value::List(items) => format!("list<{} items>", items.len()),
            Value::Option(inner) => match inner {
                Some(v) => format!("option<{}>", v.type_tag()),
                None => "option<none>".into(),
            },
            Value::Map(entries) => format!("map<{} entries>", entries.len()),
            Value::Record(fields) => format!("record<{} fields>", fields.len()),
            Value::Variant { case, .. } => format!("variant case {case:?}"),
        }
    }

    /// Convenience constructor for a finite float.
    pub fn float64(value: f64) -> Result<Self, AceError> {
        Ok(Value::Float64(Float64::new(value)?))
    }
}
