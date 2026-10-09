//! The explicit, read-only resource-type universe.
//!
//! Recognition is an explicit registry-level fact: a legal AFIDL schema
//! alone is NOT a resource type. The universe wraps an exact-pinned
//! schema universe (the semantic core's `TypeSchemaUniverse`, which
//! verifies every pin against the computed TypeSchemaDigest) plus an
//! explicit descriptor table that (a) declares which schemas are
//! recognized resource types and (b) fixes each type's closed attribute
//! vocabulary.
//!
//! There is no latest-schema lookup, no fallback, no file system, no
//! registry service: the universe is in-memory, explicit input. The
//! production resource-type registry is a later stage; tests supply
//! explicit fixture descriptors and never masquerade them as production.

use crate::error::{ResourceError, kind};
use crate::model::ResourceTypeRef;
use af_semantic::SemanticError;
use std::collections::{BTreeMap, BTreeSet};

/// The explicit descriptor of one recognized resource type: its exact
/// nominal pin and the closed attribute vocabulary its requirements may
/// speak. Attribute keys are NFC text; every key a requirement uses must
/// be part of this vocabulary or the constraint's meaning is unknown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceTypeDescriptor {
    /// The recognized schema's specification id.
    pub specification_id: String,
    /// The recognized schema's type name.
    pub type_name: String,
    /// The exact digest the recognition is pinned to (must equal the
    /// schema universe's computed digest for the same identity).
    pub schema_hex: String,
    /// The closed attribute vocabulary (NFC keys) requirements of this
    /// type may constrain.
    pub recognized_attributes: BTreeSet<String>,
}

impl ResourceTypeDescriptor {
    /// The nominal identity key (specification id, type name).
    #[must_use]
    pub fn identity_key(&self) -> (&str, &str) {
        (&self.specification_id, &self.type_name)
    }
}

/// The explicit resource-type universe: exact schema resolution plus an
/// explicit recognition table.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResourceTypeUniverse {
    schemas: af_semantic::TypeSchemaUniverse,
    descriptors: BTreeMap<(String, String), ResourceTypeDescriptor>,
}

impl ResourceTypeUniverse {
    /// An empty universe.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds every named type of a compiled AFIDL module to the schema
    /// layer (recognition still requires an explicit descriptor).
    ///
    /// # Errors
    /// [`ResourceError`] with kind `invalid_resource_type` when the
    /// schema layer rejects the module.
    pub fn insert_compiled(
        &mut self,
        compiled: &af_idl::CompiledSchema,
    ) -> Result<(), ResourceError> {
        self.schemas.insert_compiled(compiled).map_err(|error| {
            ResourceError::new(
                kind::INVALID_RESOURCE_TYPE,
                format!("cannot register fixture schemas: {error}"),
            )
        })
    }

    /// Explicitly recognizes one resource type. Re-recognizing the same
    /// identity is rejected: recognition is a set, and a silent
    /// vocabulary change would be an implicit redefinition.
    ///
    /// # Errors
    /// [`ResourceError`] with kind `invalid_resource_type` when the
    /// descriptor is malformed or the identity is already recognized.
    pub fn recognize(&mut self, descriptor: ResourceTypeDescriptor) -> Result<(), ResourceError> {
        if af_idl::SpecificationId::parse(&descriptor.specification_id).is_err() {
            return Err(ResourceError::new(
                kind::INVALID_RESOURCE_TYPE,
                format!(
                    "descriptor specification id {:?} is not a SpecificationId",
                    descriptor.specification_id
                ),
            ));
        }
        if descriptor.type_name.is_empty() || descriptor.type_name.chars().any(char::is_whitespace)
        {
            return Err(ResourceError::new(
                kind::INVALID_RESOURCE_TYPE,
                format!(
                    "descriptor type name {:?} is malformed",
                    descriptor.type_name
                ),
            ));
        }
        if descriptor.schema_hex.len() != 64
            || !descriptor
                .schema_hex
                .bytes()
                .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
        {
            return Err(ResourceError::new(
                kind::INVALID_RESOURCE_TYPE,
                "descriptor pin must be 64 lowercase hex characters",
            ));
        }
        let key = (
            descriptor.specification_id.clone(),
            descriptor.type_name.clone(),
        );
        if self.descriptors.contains_key(&key) {
            return Err(ResourceError::new(
                kind::INVALID_RESOURCE_TYPE,
                format!(
                    "resource type {}/{} is already recognized: recognition \
                     is explicit and never silently redefined",
                    descriptor.specification_id, descriptor.type_name
                ),
            ));
        }
        self.descriptors.insert(key, descriptor);
        Ok(())
    }

    /// Resolves one requirement's resource type exactly: the schema must
    /// exist, the pin must match the computed digest, the type must be
    /// explicitly recognized, and the descriptor must pin the same
    /// digest. Returns the descriptor (the attribute vocabulary).
    ///
    /// # Errors
    /// [`ResourceError`] with kinds `invalid_resource_type` (malformed
    /// pin), `unknown_resource_type` (unregistered schema or
    /// unrecognized resource type) and `resource_type_digest_mismatch`.
    pub fn resolve(
        &self,
        resource_type: &ResourceTypeRef,
    ) -> Result<&ResourceTypeDescriptor, ResourceError> {
        let entry = self
            .schemas
            .resolve(&resource_type.schema)
            .map_err(|error| {
                let identity = format!(
                    "{}/{}",
                    resource_type.specification_id(),
                    resource_type.type_name()
                );
                match &error {
                    SemanticError::UnknownTypeSchema { .. } => ResourceError::new(
                        kind::UNKNOWN_RESOURCE_TYPE,
                        format!(
                            "resource type {identity} does not resolve in \
                             the supplied schema universe (no latest-schema \
                             fallback)"
                        ),
                    ),
                    SemanticError::TypeSchemaDigestMismatch {
                        expected, actual, ..
                    } => ResourceError::new(
                        kind::RESOURCE_TYPE_DIGEST_MISMATCH,
                        format!(
                            "resource type {identity} is pinned to \
                             {expected} but the universe computes {actual}: \
                             same logical name, different identity — never \
                             silently compatible"
                        ),
                    ),
                    other => ResourceError::new(
                        kind::INVALID_RESOURCE_TYPE,
                        format!("resource type {identity} is malformed: {other}"),
                    ),
                }
            })?;
        let key = (entry.specification_id.clone(), entry.type_name.clone());
        let Some(descriptor) = self.descriptors.get(&key) else {
            return Err(ResourceError::new(
                kind::UNKNOWN_RESOURCE_TYPE,
                format!(
                    "schema {}/{} is registered but not explicitly \
                     recognized as a resource type: a legal AFIDL schema \
                     alone is not a resource type",
                    entry.specification_id, entry.type_name
                ),
            ));
        };
        if descriptor.schema_hex != entry.digest.hex() {
            return Err(ResourceError::new(
                kind::RESOURCE_TYPE_DIGEST_MISMATCH,
                format!(
                    "the recognition descriptor of {}/{} pins {} but the \
                     universe computes {}: stale recognition, refusing to \
                     interpret it",
                    entry.specification_id,
                    entry.type_name,
                    descriptor.schema_hex,
                    entry.digest.hex()
                ),
            ));
        }
        Ok(descriptor)
    }

    /// Number of recognized resource types.
    #[must_use]
    pub fn len(&self) -> usize {
        self.descriptors.len()
    }

    /// `true` when no resource type is recognized.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.descriptors.is_empty()
    }
}
