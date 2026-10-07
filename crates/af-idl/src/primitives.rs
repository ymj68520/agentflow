//! The closed AFIDL1 builtin primitive universe.
//!
//! Exactly the 13 machine primitives frozen in Phase 1 (ACE1). This enum is
//! drift-checked against `spec/tables/afidl-primitives.json` and
//! `spec/idl/core/machine-primitives.afidl` by conformance tests — the three
//! must never diverge.

use af_core::Shape;

/// A builtin AFIDL1 primitive type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Primitive {
    Unit,
    Null,
    Bool,
    Int32,
    UInt32,
    Int64,
    UInt64,
    Decimal,
    Float64,
    String,
    Bytes,
    Timestamp,
    Duration,
}

/// The AFIDL language version implemented by this crate.
pub const AFIDL1_VERSION: u32 = 1;

impl Primitive {
    /// All builtin primitives, in the normative registry order.
    pub const ALL: &'static [Primitive] = &[
        Primitive::Unit,
        Primitive::Null,
        Primitive::Bool,
        Primitive::Int32,
        Primitive::UInt32,
        Primitive::Int64,
        Primitive::UInt64,
        Primitive::Decimal,
        Primitive::Float64,
        Primitive::String,
        Primitive::Bytes,
        Primitive::Timestamp,
        Primitive::Duration,
    ];

    /// The lowercase keyword used in AFIDL source.
    pub fn keyword(self) -> &'static str {
        match self {
            Primitive::Unit => "unit",
            Primitive::Null => "null",
            Primitive::Bool => "bool",
            Primitive::Int32 => "int32",
            Primitive::UInt32 => "uint32",
            Primitive::Int64 => "int64",
            Primitive::UInt64 => "uint64",
            Primitive::Decimal => "decimal",
            Primitive::Float64 => "float64",
            Primitive::String => "string",
            Primitive::Bytes => "bytes",
            Primitive::Timestamp => "timestamp",
            Primitive::Duration => "duration",
        }
    }

    /// Resolves a keyword to the primitive, or `None` for unknown identifiers
    /// (which the parser then treats as named references).
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|p| p.keyword() == keyword)
    }

    /// The matching ACE1 shape of the primitive's values.
    pub fn shape(self) -> Shape {
        match self {
            Primitive::Unit => Shape::Unit,
            Primitive::Null => Shape::Null,
            Primitive::Bool => Shape::Bool,
            Primitive::Int32 => Shape::Int32,
            Primitive::UInt32 => Shape::UInt32,
            Primitive::Int64 => Shape::Int64,
            Primitive::UInt64 => Shape::UInt64,
            Primitive::Decimal => Shape::Decimal,
            Primitive::Float64 => Shape::Float64,
            Primitive::String => Shape::String,
            Primitive::Bytes => Shape::Bytes,
            Primitive::Timestamp => Shape::Timestamp,
            Primitive::Duration => Shape::Duration,
        }
    }
}
