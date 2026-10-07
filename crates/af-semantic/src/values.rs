//! Value slots and primitive constants.
//!
//! A [`ValueSlot`] is an immutable, SSA-like binding: one id, one type, and
//! (from Phase 3B on) exactly one producer with zero or more consumers. There
//! is deliberately no mutation operation — loop-carried values will be
//! expressed as new value versions, never by assigning into a slot.

use crate::error::SemanticError;
use crate::ids::{SemanticTypeId, ValueSlotId};
use crate::types::SemanticPrimitiveType;
use af_core::{Decimal, Duration, Timestamp};
use serde::Deserialize;
use serde::de::{Deserializer, Error as DeError};
use std::fmt;

/// One immutable semantic value slot: local identity plus its declared type
/// (a [`SemanticTypeId`] into the document's type DAG — the same semantic type
/// is defined once and referenced by id).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValueSlot {
    pub id: ValueSlotId,
    /// The slot's type, as a `SemanticTypeId`. Renamed from the AFIDL field
    /// name `type` because `type` is a Rust keyword.
    #[serde(rename = "type")]
    pub type_id: SemanticTypeId,
}

/// The only literal form of the semantic core: a value of one of the 9
/// workflow primitives.
///
/// Composite values (records, variants, lists, maps, options) are constructed
/// by pure expressions instead; there is no untyped or `Any` literal. The
/// variant case maps 1:1 onto [`SemanticPrimitiveType`], so a constant carries
/// no redundant type field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveConstant {
    Unit,
    Null,
    Bool(bool),
    Int64(i64),
    Decimal(Decimal),
    String(String),
    /// Canonical unpadded base64url text (validated on check).
    Bytes(String),
    Timestamp(Timestamp),
    Duration(Duration),
}

impl PrimitiveConstant {
    /// The primitive type of this constant (case → type is a total,
    /// unique mapping).
    pub fn semantic_type(&self) -> SemanticPrimitiveType {
        match self {
            PrimitiveConstant::Unit => SemanticPrimitiveType::Unit,
            PrimitiveConstant::Null => SemanticPrimitiveType::Null,
            PrimitiveConstant::Bool(_) => SemanticPrimitiveType::Bool,
            PrimitiveConstant::Int64(_) => SemanticPrimitiveType::Int64,
            PrimitiveConstant::Decimal(_) => SemanticPrimitiveType::Decimal,
            PrimitiveConstant::String(_) => SemanticPrimitiveType::String,
            PrimitiveConstant::Bytes(_) => SemanticPrimitiveType::Bytes,
            PrimitiveConstant::Timestamp(_) => SemanticPrimitiveType::Timestamp,
            PrimitiveConstant::Duration(_) => SemanticPrimitiveType::Duration,
        }
    }

    /// The variant case name (`AFIDL` `PrimitiveConstant` case names are the
    /// primitive keywords).
    pub fn case_name(&self) -> &'static str {
        self.semantic_type().keyword()
    }

    /// Checks the constant's own canonical form. Currently: `bytes` must be
    /// canonical unpadded base64url text.
    ///
    /// # Errors
    /// Returns [`SemanticError::InvalidConstant`] for a non-canonical payload.
    pub fn validate(&self) -> Result<(), SemanticError> {
        if let PrimitiveConstant::Bytes(text) = self {
            af_core::base64url::decode(text).map_err(|error| SemanticError::InvalidConstant {
                detail: format!("bytes constant is not canonical base64url: {error}"),
            })?;
        }
        Ok(())
    }
}

impl fmt::Display for PrimitiveConstant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.case_name())
    }
}

/// Corpus/authoring JSON form: exactly one externally-tagged member whose key
/// is the primitive keyword.
///
/// ```json
/// {"unit": null}  {"null": null}  {"bool": true}  {"int64": 5}
/// {"decimal": "1.5"}  {"string": "x"}  {"bytes": "aGk"}
/// {"timestamp": 0}  {"duration": 1}
/// ```
impl<'de> Deserialize<'de> for PrimitiveConstant {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "snake_case")]
        enum Raw {
            Unit(()),
            Null(()),
            Bool(bool),
            Int64(i64),
            Decimal(String),
            String(String),
            Bytes(String),
            Timestamp(i64),
            Duration(i64),
        }
        let raw = Raw::deserialize(deserializer)?;
        Ok(match raw {
            Raw::Unit(()) => PrimitiveConstant::Unit,
            Raw::Null(()) => PrimitiveConstant::Null,
            Raw::Bool(value) => PrimitiveConstant::Bool(value),
            Raw::Int64(value) => PrimitiveConstant::Int64(value),
            Raw::Decimal(text) => {
                PrimitiveConstant::Decimal(Decimal::parse(&text).map_err(D::Error::custom)?)
            }
            Raw::String(text) => PrimitiveConstant::String(text),
            Raw::Bytes(text) => PrimitiveConstant::Bytes(text),
            Raw::Timestamp(micros) => PrimitiveConstant::Timestamp(Timestamp::new(micros)),
            Raw::Duration(micros) => PrimitiveConstant::Duration(Duration::new(micros)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> PrimitiveConstant {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn constants_map_to_their_primitive_types() {
        assert_eq!(
            parse(r#"{"int64": 5}"#).semantic_type(),
            SemanticPrimitiveType::Int64
        );
        assert_eq!(
            parse(r#"{"unit": null}"#).semantic_type(),
            SemanticPrimitiveType::Unit
        );
        assert_eq!(
            parse(r#"{"null": null}"#).semantic_type(),
            SemanticPrimitiveType::Null
        );
        assert_eq!(
            parse(r#"{"bool": true}"#).semantic_type(),
            SemanticPrimitiveType::Bool
        );
        assert_eq!(
            parse(r#"{"timestamp": -1}"#).semantic_type(),
            SemanticPrimitiveType::Timestamp
        );
    }

    #[test]
    fn decimals_are_canonicalized_on_parse() {
        let constant = parse(r#"{"decimal": "1.250"}"#);
        let PrimitiveConstant::Decimal(decimal) = constant else {
            panic!("decimal expected");
        };
        assert_eq!(decimal.as_str(), "1.25");
        assert!(serde_json::from_str::<PrimitiveConstant>(r#"{"decimal": "01.5"}"#).is_err());
    }

    #[test]
    fn bytes_must_be_canonical_base64url() {
        assert!(parse(r#"{"bytes": "aGk"}"#).validate().is_ok());
        assert_eq!(
            parse(r#"{"bytes": "aGk="}"#).validate().unwrap_err().kind(),
            "invalid_constant"
        );
        assert!(parse(r#"{"bytes": "++"}"#).validate().is_err());
    }

    #[test]
    fn unknown_cases_and_machine_only_payloads_fail_to_parse() {
        assert!(serde_json::from_str::<PrimitiveConstant>(r#"{"float64": 1}"#).is_err());
        assert!(serde_json::from_str::<PrimitiveConstant>(r#"{"int32": 1}"#).is_err());
        assert!(serde_json::from_str::<PrimitiveConstant>(r#"{"wat": 1}"#).is_err());
        assert!(serde_json::from_str::<PrimitiveConstant>("5").is_err());
    }
}
