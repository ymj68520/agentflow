//! Canonical ordering and the canonical ACE1 bytes of the requirement row.
//!
//! Canonicalization normalizes attribute text to NFC, rejects empty
//! attributes, duplicate normalized attribute keys within one requirement
//! and duplicate ports, then sorts requirements by ascending port number
//! and attributes by unsigned UTF-8 key order (then value). Source
//! declaration order never reaches the canonical bytes, and no HashMap
//! iteration order is ever observable.
//!
//! The canonical form has ACE1 bytes but deliberately NO digest: no
//! resource-requirement digest exists, the HashV1 domain registry is
//! untouched, and the future contract layer will hash as a whole under
//! the frozen `component-contract` domain.

use crate::error::{ResourceError, kind};
use crate::model::{
    ResourceAttributeConstraint, ResourceRequirement, ResourceRequirementRow, ResourceTypeRef,
};
use af_core::Value;
use std::collections::BTreeSet;
use unicode_normalization::UnicodeNormalization;

/// NFC normalization for attribute text (keys and values).
#[must_use]
pub fn nfc(text: &str) -> String {
    text.nfc().collect()
}

/// Syntax check of one nominal resource-type reference (no universe
/// needed).
///
/// # Errors
/// [`ResourceError`] with kind `invalid_resource_type`.
pub fn check_identity(resource_type: &ResourceTypeRef) -> Result<(), ResourceError> {
    if af_idl::SpecificationId::parse(resource_type.specification_id()).is_err() {
        return Err(ResourceError::new(
            kind::INVALID_RESOURCE_TYPE,
            format!(
                "specification id {:?} is not a SpecificationId",
                resource_type.specification_id()
            ),
        ));
    }
    let name = resource_type.type_name();
    if name.is_empty() || name.chars().any(char::is_whitespace) {
        return Err(ResourceError::new(
            kind::INVALID_RESOURCE_TYPE,
            format!("type name {name:?} is not a valid AFIDL type name"),
        ));
    }
    // Full digest grammar: sha256, the frozen type-schema domain, 64
    // lowercase hex characters.
    resource_type
        .schema
        .schema_digest
        .validate_for_type_schema()
        .map_err(|error| {
            ResourceError::new(
                kind::INVALID_RESOURCE_TYPE,
                format!("pinned schema digest is not a TypeSchemaDigest: {error}"),
            )
        })
}

/// Normalizes one requirement's attribute list: NFC text, non-empty
/// keys and values, unique normalized keys, sorted by (key, value) in
/// unsigned UTF-8 order.
///
/// # Errors
/// [`ResourceError`] with kinds `invalid_resource_attribute` and
/// `duplicate_resource_attribute`.
pub fn canonical_attributes(
    attributes: &[ResourceAttributeConstraint],
) -> Result<Vec<ResourceAttributeConstraint>, ResourceError> {
    let mut seen = BTreeSet::new();
    let mut canonical = Vec::with_capacity(attributes.len());
    for attribute in attributes {
        let key = nfc(&attribute.key);
        let value = nfc(&attribute.value);
        if key.is_empty() || value.is_empty() {
            return Err(ResourceError::new(
                kind::INVALID_RESOURCE_ATTRIBUTE,
                format!(
                    "attribute key {key:?} / value {value:?} must both be \
                     non-empty NFC text"
                ),
            ));
        }
        if !seen.insert(key.clone()) {
            return Err(ResourceError::new(
                kind::DUPLICATE_RESOURCE_ATTRIBUTE,
                format!(
                    "attribute key {key:?} is declared twice (NFC-equal \
                     spellings are the same key): two different values for \
                     one key are never silently overwritten"
                ),
            ));
        }
        canonical.push(ResourceAttributeConstraint { key, value });
    }
    canonical.sort_by(|a, b| (&a.key, &a.value).cmp(&(&b.key, &b.value)));
    Ok(canonical)
}

/// `canonicalize_resource_requirement_row`: the canonical form of one
/// requirement row — positive unique ports in ascending order, canonical
/// attribute lists, syntax-checked nominal identity.
///
/// # Errors
/// [`ResourceError`] with kinds `invalid_resource_type`,
/// `invalid_resource_port`, `duplicate_resource_port`,
/// `invalid_resource_attribute` and `duplicate_resource_attribute`.
pub fn canonicalize_resource_requirement_row(
    row: &ResourceRequirementRow,
) -> Result<ResourceRequirementRow, ResourceError> {
    let mut seen_ports = BTreeSet::new();
    let mut requirements = Vec::with_capacity(row.requirements.len());
    for requirement in &row.requirements {
        if requirement.port_no == 0 {
            return Err(ResourceError::new(
                kind::INVALID_RESOURCE_PORT,
                "requirement port 0 is invalid: ports are positive",
            ));
        }
        if !seen_ports.insert(requirement.port_no) {
            return Err(ResourceError::new(
                kind::DUPLICATE_RESOURCE_PORT,
                format!(
                    "requirement port {} is declared twice: one contract \
                     interface has at most one requirement per port",
                    requirement.port_no
                ),
            ));
        }
        check_identity(&requirement.resource_type)?;
        requirements.push(ResourceRequirement {
            port_no: requirement.port_no,
            resource_type: requirement.resource_type.clone(),
            required_attributes: canonical_attributes(&requirement.required_attributes)?,
        });
    }
    requirements.sort_by_key(|requirement| requirement.port_no);
    Ok(ResourceRequirementRow { requirements })
}

impl ResourceRequirementRow {
    /// `true` for the empty row `R = ∅` — which does not mean the
    /// computation is pure, succeeds or is authorized.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.requirements.is_empty()
    }

    /// Number of declared resource requirements.
    #[must_use]
    pub fn len(&self) -> usize {
        self.requirements.len()
    }

    /// The requirement declared at one interface port, if any.
    #[must_use]
    pub fn at_port(&self, port_no: u32) -> Option<&ResourceRequirement> {
        self.requirements.iter().find(|r| r.port_no == port_no)
    }

    /// The canonical form: syntax-checked, duplicate-free, canonically
    /// ordered.
    ///
    /// # Errors
    /// [`ResourceError`] with the structural kinds of
    /// [`canonicalize_resource_requirement_row`].
    pub fn canonical(&self) -> Result<ResourceRequirementRow, ResourceError> {
        canonicalize_resource_requirement_row(self)
    }

    /// The canonical ACE1 bytes, encoded against the schema-derived shape
    /// of `agentflow.resource.requirement@1/ResourceRequirementRow` within
    /// `set` (which must contain the resource module and the semantic core
    /// it imports). Requires a structurally valid row.
    ///
    /// # Errors
    /// [`ResourceError`] with kind `resource_encoding_error` when the
    /// canonical form is invalid or the value tree cannot be encoded.
    pub fn canonical_bytes(
        &self,
        set: &af_idl::CompiledModuleSet,
    ) -> Result<Vec<u8>, ResourceError> {
        let value = self.to_value()?;
        af_core::encode(&value_shape(set)?, &value).map_err(|error| {
            ResourceError::new(
                kind::RESOURCE_ENCODING_ERROR,
                format!("cannot encode the canonical requirement row: {error}"),
            )
        })
    }

    /// The ACE1 value tree of the canonical row (mirrors the AFIDL schema
    /// field-for-field; the encoder emits members in byte order).
    ///
    /// # Errors
    /// [`ResourceError`] when the row is not structurally valid.
    pub fn to_value(&self) -> Result<Value, ResourceError> {
        let canonical = canonicalize_resource_requirement_row(self)?;
        Ok(Value::Record(vec![(
            "requirements".into(),
            Value::List(
                canonical
                    .requirements
                    .iter()
                    .map(|requirement| {
                        Value::Record(vec![
                            ("port_no".into(), Value::UInt32(requirement.port_no)),
                            (
                                "resource_type".into(),
                                Value::Record(vec![(
                                    "schema".into(),
                                    Value::Record(vec![
                                        (
                                            "specification_id".into(),
                                            Value::String(
                                                requirement.resource_type.specification_id().into(),
                                            ),
                                        ),
                                        (
                                            "type_name".into(),
                                            Value::String(
                                                requirement.resource_type.type_name().into(),
                                            ),
                                        ),
                                        (
                                            "schema_digest".into(),
                                            Value::Record(vec![
                                                (
                                                    "algorithm".into(),
                                                    Value::String(
                                                        requirement
                                                            .resource_type
                                                            .schema
                                                            .schema_digest
                                                            .algorithm
                                                            .clone(),
                                                    ),
                                                ),
                                                (
                                                    "domain".into(),
                                                    Value::String(
                                                        requirement
                                                            .resource_type
                                                            .schema
                                                            .schema_digest
                                                            .domain
                                                            .clone(),
                                                    ),
                                                ),
                                                (
                                                    "hex".into(),
                                                    Value::String(
                                                        requirement
                                                            .resource_type
                                                            .schema_hex()
                                                            .into(),
                                                    ),
                                                ),
                                            ]),
                                        ),
                                    ]),
                                )]),
                            ),
                            (
                                "required_attributes".into(),
                                Value::List(
                                    requirement
                                        .required_attributes
                                        .iter()
                                        .map(|attribute| {
                                            Value::Record(vec![
                                                (
                                                    "key".into(),
                                                    Value::String(attribute.key.clone()),
                                                ),
                                                (
                                                    "value".into(),
                                                    Value::String(attribute.value.clone()),
                                                ),
                                            ])
                                        })
                                        .collect(),
                                ),
                            ),
                        ])
                    })
                    .collect(),
            ),
        )]))
    }
}

fn value_shape(set: &af_idl::CompiledModuleSet) -> Result<af_core::Shape, ResourceError> {
    let shape = set
        .shape_of("agentflow.resource.requirement@1/ResourceRequirementRow")
        .map_err(|error| {
            ResourceError::new(
                kind::RESOURCE_ENCODING_ERROR,
                format!("cannot derive the requirement-row shape: {error}"),
            )
        })?;
    shape.validate().map_err(|error| {
        ResourceError::new(
            kind::RESOURCE_ENCODING_ERROR,
            format!("derived shape is invalid: {error}"),
        )
    })?;
    Ok(shape)
}
