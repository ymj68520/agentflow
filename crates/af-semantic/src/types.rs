//! Semantic types: the workflow primitive universe, schema references and the
//! document-local semantic type DAG.
//!
//! # Machine primitives vs workflow primitives
//!
//! The AFIDL machine primitive universe has 13 members (they answer "how do
//! machine objects express fields?"). The *workflow* semantic primitive
//! universe is deliberately narrower — it answers "what can a value in a user
//! workflow be?" — and freezes exactly 9 members:
//!
//! ```text
//! unit  null  bool  int64  decimal  string  bytes  timestamp  duration
//! ```
//!
//! `int32`, `uint32`, `uint64` and `float64` are machine-only: they may appear
//! in AFIDL machine schemas (e.g. `field_no: uint32`) but MUST NOT surface as
//! workflow value types. [`SemanticPrimitiveType::try_from_keyword`] is the
//! single closed gate for that rule.
//!
//! # Type references are an indexed DAG
//!
//! AFIDL1 forbids recursive named schemas (Phase 2A), so a self-recursive
//! `SemanticTypeRef` cannot be expressed in AFIDL source. Instead the semantic
//! core represents composite types as a **document-local DAG**: a table of
//! [`SemanticTypeNode`]s, each carrying an explicit [`SemanticTypeId`] and a
//! body that references other nodes by id. This expresses arbitrary finite
//! nesting (`option<list<map<string,T>>>`) without recursive schemas and
//! without any ACE1 change; the DAG must be acyclic and every reference must
//! resolve.

use crate::error::SemanticError;
use crate::ids::SemanticTypeId;
use serde::Deserialize;
use std::fmt;

/// The 9-member workflow semantic primitive universe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SemanticPrimitiveType {
    Unit,
    Null,
    Bool,
    Int64,
    Decimal,
    String,
    Bytes,
    Timestamp,
    Duration,
}

impl SemanticPrimitiveType {
    /// All workflow primitives, in the frozen `value_no` order.
    pub const ALL: &'static [SemanticPrimitiveType] = &[
        SemanticPrimitiveType::Unit,
        SemanticPrimitiveType::Null,
        SemanticPrimitiveType::Bool,
        SemanticPrimitiveType::Int64,
        SemanticPrimitiveType::Decimal,
        SemanticPrimitiveType::String,
        SemanticPrimitiveType::Bytes,
        SemanticPrimitiveType::Timestamp,
        SemanticPrimitiveType::Duration,
    ];

    /// The AFIDL machine primitives that are deliberately NOT workflow
    /// primitives.
    pub const MACHINE_ONLY: &'static [&'static str] = &["int32", "uint32", "uint64", "float64"];

    /// The lowercase keyword (identical to the AFIDL machine primitive
    /// keyword for the 9 shared primitives).
    pub fn keyword(self) -> &'static str {
        match self {
            SemanticPrimitiveType::Unit => "unit",
            SemanticPrimitiveType::Null => "null",
            SemanticPrimitiveType::Bool => "bool",
            SemanticPrimitiveType::Int64 => "int64",
            SemanticPrimitiveType::Decimal => "decimal",
            SemanticPrimitiveType::String => "string",
            SemanticPrimitiveType::Bytes => "bytes",
            SemanticPrimitiveType::Timestamp => "timestamp",
            SemanticPrimitiveType::Duration => "duration",
        }
    }

    /// The one closed gate into the workflow primitive universe.
    ///
    /// Every AFIDL machine primitive keyword that is not a workflow primitive
    /// (plus every unknown keyword) is rejected with
    /// [`SemanticError::UnsupportedWorkflowPrimitive`].
    pub fn try_from_keyword(keyword: &str) -> Result<Self, SemanticError> {
        Self::ALL
            .iter()
            .copied()
            .find(|primitive| primitive.keyword() == keyword)
            .ok_or_else(|| SemanticError::UnsupportedWorkflowPrimitive {
                keyword: keyword.to_string(),
                detail: if Self::MACHINE_ONLY.contains(&keyword) {
                    "machine-only primitive; not in the workflow semantic universe".into()
                } else {
                    "not a workflow semantic primitive".into()
                },
            })
    }
}

impl fmt::Display for SemanticPrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.keyword())
    }
}

/// The AFIDL machine representation of a `DigestRef` — the single authoritative
/// shape used by every future Plan/Journal/Value object that pins a digest.
///
/// This is deliberately *not* a bare string: algorithm, domain and hex travel
/// together and the validator enforces the frozen HashV1 rules.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DigestRefValue {
    pub algorithm: String,
    pub domain: String,
    pub hex: String,
}

impl DigestRefValue {
    /// Validates the digest reference for a `TypeSchemaRef`: algorithm must be
    /// `sha256`, the domain must be the frozen `type-schema` domain and the hex
    /// must be exactly 64 lowercase hexadecimal characters.
    ///
    /// # Errors
    /// Returns [`SemanticError::InvalidDigestRef`] for any violation.
    pub fn validate_for_type_schema(&self) -> Result<(), SemanticError> {
        if self.algorithm != "sha256" {
            return Err(SemanticError::InvalidDigestRef {
                detail: format!("algorithm {:?} is not sha256", self.algorithm),
            });
        }
        let domain = af_core::DigestDomain::from_registry_name(&self.domain).map_err(|_| {
            SemanticError::InvalidDigestRef {
                detail: format!(
                    "domain {:?} is not in the frozen HashV1 registry",
                    self.domain
                ),
            }
        })?;
        if domain != af_core::DigestDomain::TypeSchema {
            return Err(SemanticError::InvalidDigestRef {
                detail: format!(
                    "domain {:?} is a HashV1 domain but not the type-schema domain",
                    self.domain
                ),
            });
        }
        if !is_lower_hex_64(&self.hex) {
            return Err(SemanticError::InvalidDigestRef {
                detail: "hex must be exactly 64 lowercase hexadecimal characters".into(),
            });
        }
        Ok(())
    }
}

/// Pins a named AFIDL type: `<specification_id>/<type_name>` plus the exact
/// canonical `TypeSchemaDigest` the reference was authored against.
///
/// A `TypeSchemaRef` never resolves against "the latest registry state": the
/// digest is part of the reference, so a published reference keeps denoting
/// exactly the schema it was compiled against.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeSchemaRef {
    pub specification_id: String,
    pub type_name: String,
    pub schema_digest: DigestRefValue,
}

impl TypeSchemaRef {
    /// The full logical identity `<specification_id>/<type_name>`.
    pub fn identity(&self) -> String {
        format!("{}/{}", self.specification_id, self.type_name)
    }
}

impl fmt::Display for TypeSchemaRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.identity(), self.schema_digest.hex)
    }
}

/// One node body of the semantic type DAG.
///
/// `schema` is a leaf (it pins a named AFIDL type); `list`, `map_string` and
/// `option` reference another node of the same DAG by id.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticTypeNodeBody {
    Primitive(SemanticPrimitiveType),
    Schema(TypeSchemaRef),
    List(SemanticTypeId),
    MapString(SemanticTypeId),
    Option(SemanticTypeId),
}

/// One node of the document-local semantic type DAG.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticTypeNode {
    pub id: SemanticTypeId,
    pub body: SemanticTypeNodeBody,
}

/// A fully expanded structural value type: the result of resolving a
/// [`SemanticTypeId`] through the DAG (and aliases through the schema
/// universe).
///
/// Two structural types are equal when they are structurally identical;
/// schema leaves compare by their full pin (identity *and* digest). There is
/// no nominal identity for containers beyond structure — nominal identity
/// lives in `TypeSchemaRef`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructuralType {
    Primitive(SemanticPrimitiveType),
    Schema(TypeSchemaRef),
    List(Box<StructuralType>),
    MapString(Box<StructuralType>),
    Option(Box<StructuralType>),
}

impl StructuralType {
    /// The canonical, human-readable and frozen-comparable form:
    ///
    /// ```text
    /// primitive:int64
    /// schema:agentflow.example@1/User@<64-hex-digest>
    /// list:<inner>
    /// map_string:<inner>
    /// option:<inner>
    /// ```
    pub fn canonical_string(&self) -> String {
        match self {
            StructuralType::Primitive(primitive) => format!("primitive:{}", primitive.keyword()),
            StructuralType::Schema(reference) => format!("schema:{reference}"),
            StructuralType::List(inner) => format!("list:{}", inner.canonical_string()),
            StructuralType::MapString(inner) => {
                format!("map_string:{}", inner.canonical_string())
            }
            StructuralType::Option(inner) => format!("option:{}", inner.canonical_string()),
        }
    }
}

impl fmt::Display for StructuralType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.canonical_string())
    }
}

/// `true` for exactly 64 lowercase hexadecimal characters.
pub(crate) fn is_lower_hex_64(text: &str) -> bool {
    text.len() == 64
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workflow_universe_is_closed_to_the_nine_primitives() {
        for primitive in SemanticPrimitiveType::ALL {
            assert_eq!(
                SemanticPrimitiveType::try_from_keyword(primitive.keyword()).unwrap(),
                *primitive
            );
        }
        assert_eq!(SemanticPrimitiveType::ALL.len(), 9);
        for keyword in SemanticPrimitiveType::MACHINE_ONLY {
            let error = SemanticPrimitiveType::try_from_keyword(keyword).unwrap_err();
            assert_eq!(error.kind(), "unsupported_workflow_primitive");
        }
        assert!(SemanticPrimitiveType::try_from_keyword("nonsense").is_err());
    }

    #[test]
    fn digest_ref_value_validation() {
        let good = DigestRefValue {
            algorithm: "sha256".into(),
            domain: "type-schema".into(),
            hex: "a".repeat(64),
        };
        assert!(good.validate_for_type_schema().is_ok());

        let mut bad = good.clone();
        bad.algorithm = "sha512".into();
        assert_eq!(
            bad.validate_for_type_schema().unwrap_err().kind(),
            "invalid_digest_ref"
        );
        let mut bad = good.clone();
        bad.domain = "value".into();
        assert!(bad.validate_for_type_schema().is_err());
        let mut bad = good.clone();
        bad.domain = "not-a-domain".into();
        assert!(bad.validate_for_type_schema().is_err());
        let mut bad = good.clone();
        bad.hex = "A".repeat(64);
        assert!(bad.validate_for_type_schema().is_err());
        let mut bad = good;
        bad.hex = "a".repeat(63);
        assert!(bad.validate_for_type_schema().is_err());
    }

    #[test]
    fn canonical_strings_are_stable() {
        let reference = TypeSchemaRef {
            specification_id: "agentflow.example@1".into(),
            type_name: "User".into(),
            schema_digest: DigestRefValue {
                algorithm: "sha256".into(),
                domain: "type-schema".into(),
                hex: "ab".repeat(32),
            },
        };
        assert_eq!(
            StructuralType::Option(Box::new(StructuralType::List(Box::new(
                StructuralType::Primitive(SemanticPrimitiveType::String)
            ))))
            .canonical_string(),
            "option:list:primitive:string"
        );
        assert_eq!(
            StructuralType::Schema(reference.clone()).canonical_string(),
            format!("schema:agentflow.example@1/User@{}", "ab".repeat(32))
        );
        assert_eq!(reference.identity(), "agentflow.example@1/User");
    }
}
