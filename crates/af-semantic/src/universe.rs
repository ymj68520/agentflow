//! The explicit `TypeSchemaUniverse` the validator resolves against.
//!
//! The semantic core never performs a "latest registry lookup": the validator
//! receives an explicit, read-only universe and a [`TypeSchemaRef`] is only
//! valid when its pinned identity and digest both match an entry. In tests the
//! universe is built from AFIDL compile output; a future compiler stage will
//! build it from a resolution snapshot.

use crate::error::SemanticError;
use crate::types::{DigestRefValue, SemanticPrimitiveType, StructuralType, TypeSchemaRef};
use af_idl::{
    CanonicalBody, CanonicalTypeExpr, CanonicalTypeSchema, CompiledSchema, SpecificationId,
};
use std::collections::BTreeMap;

/// One named canonical schema, indexed by its full logical identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaEntry {
    pub specification_id: String,
    pub type_name: String,
    /// The canonical `TypeSchemaDigest` (`sha256/type-schema/<hex>`).
    pub digest: af_core::DigestRef,
    pub body: CanonicalBody,
}

impl SchemaEntry {
    /// The full logical identity `<specification_id>/<type_name>`.
    pub fn identity(&self) -> String {
        format!("{}/{}", self.specification_id, self.type_name)
    }

    /// The `TypeSchemaRef` that pins this entry exactly.
    pub fn reference(&self) -> TypeSchemaRef {
        TypeSchemaRef {
            specification_id: self.specification_id.clone(),
            type_name: self.type_name.clone(),
            schema_digest: DigestRefValue {
                algorithm: self.digest.alg().as_str().to_string(),
                domain: self.digest.domain().as_str().to_string(),
                hex: self.digest.hex().to_string(),
            },
        }
    }
}

/// A read-only set of canonical named schemas, keyed by full logical identity.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TypeSchemaUniverse {
    entries: BTreeMap<String, SchemaEntry>,
}

impl TypeSchemaUniverse {
    /// An empty universe.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds every named type of a compiled AFIDL module.
    ///
    /// # Errors
    /// Returns a [`SemanticError`] if a type's digest cannot be computed or
    /// the same identity is inserted twice with different content.
    pub fn insert_compiled(&mut self, compiled: &CompiledSchema) -> Result<(), SemanticError> {
        for schema_type in &compiled.types {
            self.insert_type(schema_type)?;
        }
        Ok(())
    }

    /// Adds one canonical named schema.
    ///
    /// # Errors
    /// Returns a [`SemanticError`] if the digest cannot be computed.
    pub fn insert_type(&mut self, schema_type: &CanonicalTypeSchema) -> Result<(), SemanticError> {
        let digest = schema_type
            .digest()
            .map_err(|error| SemanticError::InvalidDigestRef {
                detail: format!("cannot compute TypeSchemaDigest: {error}"),
            })?;
        let entry = SchemaEntry {
            specification_id: schema_type.schema.clone(),
            type_name: schema_type.name.clone(),
            digest,
            body: schema_type.body.clone(),
        };
        self.entries.insert(entry.identity(), entry);
        Ok(())
    }

    /// Number of registered named schemas.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `true` when no schema is registered.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Looks up a schema by full logical identity.
    pub fn get(&self, identity: &str) -> Option<&SchemaEntry> {
        self.entries.get(identity)
    }

    /// Resolves and fully validates a [`TypeSchemaRef`]: the identity must
    /// exist in the universe and the pinned digest must equal the schema's
    /// actual canonical digest. The pinned digest reference must itself be
    /// well-formed (`sha256` / `type-schema` / 64 lowercase hex).
    ///
    /// # Errors
    /// [`SemanticError::UnknownTypeSchema`] for a missing identity,
    /// [`SemanticError::TypeSchemaDigestMismatch`] for a wrong digest, and
    /// [`SemanticError::InvalidDigestRef`] for a malformed digest reference.
    pub fn resolve(&self, reference: &TypeSchemaRef) -> Result<&SchemaEntry, SemanticError> {
        reference.schema_digest.validate_for_type_schema()?;
        // A malformed identity can never be in the universe.
        let identity = reference.identity();
        if SpecificationId::parse(&reference.specification_id).is_err()
            || !is_type_name(&reference.type_name)
        {
            return Err(SemanticError::UnknownTypeSchema { identity });
        }
        let Some(entry) = self.entries.get(&identity) else {
            return Err(SemanticError::UnknownTypeSchema { identity });
        };
        if entry.digest.hex() != reference.schema_digest.hex {
            return Err(SemanticError::TypeSchemaDigestMismatch {
                identity,
                expected: reference.schema_digest.hex.clone(),
                actual: entry.digest.hex().to_string(),
            });
        }
        Ok(entry)
    }

    /// Expands a canonical AFIDL type expression into a workflow structural
    /// type. Named references expand through the universe: aliases resolve to
    /// their target (aliases are schema-level nominal identities; a value of
    /// an alias type is a value of the target type), while records, variants
    /// and enums become `schema` leaves pinning the referenced type.
    ///
    /// # Errors
    /// [`SemanticError::UnsupportedWorkflowPrimitive`] when a machine-only
    /// primitive surfaces, [`SemanticError::UnknownTypeSchema`] for a
    /// reference outside the universe.
    pub fn expand(&self, expr: &CanonicalTypeExpr) -> Result<StructuralType, SemanticError> {
        match expr {
            CanonicalTypeExpr::Primitive(primitive) => Ok(StructuralType::Primitive(
                SemanticPrimitiveType::try_from_keyword(primitive.keyword())?,
            )),
            CanonicalTypeExpr::NamedRef(identity) => self.expand_named(identity),
            CanonicalTypeExpr::List(inner) => {
                Ok(StructuralType::List(Box::new(self.expand(inner)?)))
            }
            CanonicalTypeExpr::MapString(inner) => {
                Ok(StructuralType::MapString(Box::new(self.expand(inner)?)))
            }
            CanonicalTypeExpr::Option(inner) => {
                Ok(StructuralType::Option(Box::new(self.expand(inner)?)))
            }
        }
    }

    /// Expands a named reference by full logical identity.
    ///
    /// # Errors
    /// As [`TypeSchemaUniverse::expand`].
    pub fn expand_named(&self, identity: &str) -> Result<StructuralType, SemanticError> {
        let Some(entry) = self.entries.get(identity) else {
            return Err(SemanticError::UnknownTypeSchema {
                identity: identity.to_string(),
            });
        };
        match &entry.body {
            CanonicalBody::Alias(target) => self.expand(target),
            CanonicalBody::Record(_) | CanonicalBody::Variant(_) | CanonicalBody::Enum(_) => {
                Ok(StructuralType::Schema(entry.reference()))
            }
        }
    }

    /// The expanded type of a record field, identified by `field_no` (the
    /// durable ABI numbering — names are value-encoding keys, not schema
    /// identity).
    ///
    /// # Errors
    /// [`SemanticError::ExpectedRecordSchema`] when the entry is not a
    /// record, [`SemanticError::UnknownFieldNumber`] for an undeclared number.
    pub fn field_type(
        &self,
        entry: &SchemaEntry,
        field_no: u32,
    ) -> Result<StructuralType, SemanticError> {
        let CanonicalBody::Record(fields) = &entry.body else {
            return Err(SemanticError::ExpectedRecordSchema {
                identity: entry.identity(),
            });
        };
        let Some(field) = fields.iter().find(|field| field.no == field_no) else {
            return Err(SemanticError::UnknownFieldNumber {
                identity: entry.identity(),
                field_no,
            });
        };
        self.expand(&field.type_expr)
    }

    /// The expanded payload type of a variant case, identified by `case_no`.
    ///
    /// # Errors
    /// [`SemanticError::ExpectedVariantSchema`] / [`SemanticError::UnknownVariantCase`].
    pub fn case_payload_type(
        &self,
        entry: &SchemaEntry,
        case_no: u32,
    ) -> Result<StructuralType, SemanticError> {
        let CanonicalBody::Variant(cases) = &entry.body else {
            return Err(SemanticError::ExpectedVariantSchema {
                identity: entry.identity(),
            });
        };
        let Some(case) = cases.iter().find(|case| case.no == case_no) else {
            return Err(SemanticError::UnknownVariantCase {
                identity: entry.identity(),
                case_no,
            });
        };
        self.expand(&case.type_expr)
    }

    /// `true` when the entry denotes an enum (the only schema kind comparable
    /// with `equals` besides primitives in Phase 3A).
    pub fn is_enum(&self, entry: &SchemaEntry) -> bool {
        matches!(entry.body, CanonicalBody::Enum(_))
    }
}

/// AFIDL1 `TYPE-NAME` grammar: `[A-Z][A-Za-z0-9]*`.
fn is_type_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) if first.is_ascii_uppercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;
    use af_idl::compile;

    const UNIVERSE: &str = "afidl 1;\n\
        schema agentflow.example @1;\n\
        alias UserId = string;\n\
        enum Status { 1 pending; 2 closed; }\n\
        record User { 1 id: UserId; 2 status: Status; }\n\
        record Metrics { 1 count: int64; 2 ratio: float64; }\n";

    fn universe() -> TypeSchemaUniverse {
        let compiled = compile(UNIVERSE).unwrap();
        let mut universe = TypeSchemaUniverse::new();
        universe.insert_compiled(&compiled).unwrap();
        universe
    }

    #[test]
    fn resolves_pinned_references_exactly() {
        let universe = universe();
        let entry = universe.get("agentflow.example@1/User").unwrap();
        let reference = entry.reference();
        assert!(universe.resolve(&reference).is_ok());

        let mut wrong = reference.clone();
        wrong.type_name = "Status".into();
        assert_eq!(
            universe.resolve(&wrong).unwrap_err().kind(),
            "type_schema_digest_mismatch"
        );

        let mut missing = reference;
        missing.type_name = "Ghost".into();
        assert_eq!(
            universe.resolve(&missing).unwrap_err().kind(),
            "unknown_type_schema"
        );
    }

    #[test]
    fn alias_expands_to_target_and_machine_only_primitives_are_rejected() {
        let universe = universe();
        let user = universe.get("agentflow.example@1/User").unwrap();
        assert_eq!(
            universe.field_type(user, 1).unwrap(),
            StructuralType::Primitive(SemanticPrimitiveType::String),
            "UserId is an alias of string"
        );
        assert_eq!(
            universe.field_type(user, 2).unwrap().canonical_string(),
            format!(
                "schema:agentflow.example@1/Status@{}",
                universe
                    .get("agentflow.example@1/Status")
                    .unwrap()
                    .digest
                    .hex()
            )
        );

        let metrics = universe.get("agentflow.example@1/Metrics").unwrap();
        let error = universe.field_type(metrics, 2).unwrap_err();
        assert_eq!(error.kind(), "unsupported_workflow_primitive");
        assert_eq!(
            universe.field_type(metrics, 9).unwrap_err().kind(),
            "unknown_field_number"
        );
    }

    #[test]
    fn records_are_schema_leaves_and_fields_use_field_no() {
        let universe = universe();
        let status = universe.get("agentflow.example@1/Status").unwrap();
        assert!(universe.is_enum(status));
        assert_eq!(
            universe.field_type(status, 1).unwrap_err().kind(),
            "expected_record_schema"
        );
    }
}
