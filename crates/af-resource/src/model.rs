//! The typed resource-requirement model, mirroring
//! `spec/idl/resource-requirement.afidl` field-for-field.
//!
//! A resource type is NOMINAL: the full triple (specification_id,
//! type_name, pinned schema digest). Two refs with the same short name
//! under different specifications — or with different pinned digests —
//! are different resource types and are never silently compatible; there
//! is no implicit resource-type subtyping.
//!
//! A requirement DECLares need; it never grants authority. Requirement
//! satisfaction is a static predicate over explicit descriptors, not an
//! authorization, and no effect is approved by anything in this crate.
//!
//! The row is a CLOSED set of requirements keyed by interface port;
//! unknown requirement upper bounds are an analysis-level result
//! ([`crate::compatibility::ResourceCompatibilityProof::Unproven`]),
//! never an empty row and never a wire state.

use af_semantic::TypeSchemaRef;
use serde::Deserialize;

/// One required resource type: an exact pin of a named AFIDL schema,
/// reused from the semantic core by full cross-module identity. The type
/// is a resource type for a contract exactly by being required in that
/// contract's [`ResourceRequirementRow`] and recognized by the explicit
/// [`crate::universe::ResourceTypeUniverse`].
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceTypeRef {
    pub schema: TypeSchemaRef,
}

impl ResourceTypeRef {
    /// The specification half of the nominal identity.
    #[must_use]
    pub fn specification_id(&self) -> &str {
        &self.schema.specification_id
    }

    /// The type-name half of the nominal identity.
    #[must_use]
    pub fn type_name(&self) -> &str {
        &self.schema.type_name
    }

    /// The pinned TypeSchemaDigest hex half of the nominal identity.
    #[must_use]
    pub fn schema_hex(&self) -> &str {
        &self.schema.schema_digest.hex
    }

    /// The full nominal identity in byte order (specification, type name,
    /// pinned digest hex).
    #[must_use]
    pub fn nominal_key(&self) -> (&str, &str, &str) {
        (self.specification_id(), self.type_name(), self.schema_hex())
    }

    /// Exact nominal equality against another ref (all three identity
    /// parts; never the short name alone).
    #[must_use]
    pub fn is_same_type(&self, other: &Self) -> bool {
        self.nominal_key() == other.nominal_key()
    }
}

/// One exact-value attribute condition: a candidate resource's attribute
/// `key` must carry exactly `value`. The predicate language is closed —
/// exact equality over non-empty NFC text only; no glob, regex, path
/// prefix, CIDR containment, policy expression or predicate script.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceAttributeConstraint {
    pub key: String,
    pub value: String,
}

/// One resource requirement at a stable interface port of ONE component
/// contract. The port is a requirement-port coordinate only: it is not a
/// slot id, not a handle id, not a logical resource instance id and not a
/// derived subflow local id; those coordinate systems relate to it only
/// through explicit future binding/mapping.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceRequirement {
    pub port_no: u32,
    pub resource_type: ResourceTypeRef,
    pub required_attributes: Vec<ResourceAttributeConstraint>,
}

/// The closed Requirement Row `R = {r1, r2, ... rn}` of one contract:
/// exactly the external resources this contract requires.
///
/// `R = ∅` means "no external resource requirement is declared". It does
/// NOT mean the computation is pure (that is ε), succeeds (outcome plane)
/// or is authorized (a future security/effect grant).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceRequirementRow {
    pub requirements: Vec<ResourceRequirement>,
}

impl ResourceRequirementRow {
    /// The empty row `R = ∅` — not a purity, success or authorization
    /// claim.
    #[must_use]
    pub fn empty() -> Self {
        ResourceRequirementRow {
            requirements: Vec::new(),
        }
    }
}
