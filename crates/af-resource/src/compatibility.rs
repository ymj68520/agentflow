//! Static satisfaction and the requirement non-strengthening proof.
//!
//! Two judgments live here, both purely static over explicitly supplied
//! values:
//!
//! - [`satisfies_resource_requirement`]: an abstract candidate descriptor
//!   meets one requirement (exact type pin + all exact attribute
//!   conditions). This proves NOTHING about credentials, live resources,
//!   capacity, authorization or effects — it is a predicate, not a grant.
//!
//! - [`requirement_not_stronger`]: within ONE port namespace, the new
//!   contract's requirement row accepts at least every candidate the old
//!   row accepted. Under the exact-attribute conjunctive model the proof
//!   obligation per retained port is the (key, value) subset
//!   `new.required_attributes ⊆ old.required_attributes`; a removed port
//!   never strengthens; a new port, a changed type or a widened attribute
//!   set fails the proof. Unprovable cases return
//!   [`ResourceCompatibilityProof::Unproven`] and are never auto-judged
//!   compatible.

use crate::canonical::{
    canonical_attributes, canonicalize_resource_requirement_row, check_identity,
};
use crate::error::{ResourceError, kind};
use crate::model::{
    ResourceAttributeConstraint, ResourceRequirement, ResourceRequirementRow, ResourceTypeRef,
};
use crate::universe::ResourceTypeUniverse;
use std::collections::{BTreeMap, BTreeSet};
use unicode_normalization::UnicodeNormalization;

/// Why a non-strengthening judgment could not be proven. The stable
/// names are frozen in the compatibility corpus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnprovenReason {
    /// A retained port's resource type changed nominal identity. There is
    /// no implicit resource-type subtyping: candidates of the old type
    /// cannot be proven to satisfy the new one.
    RetainedPortResourceTypeChanged,
    /// A NEW requirement constrains an attribute key outside its type's
    /// descriptor vocabulary. Unknown-semantics constraints cannot be
    /// assumed conjunctive exact-value, so the subset argument does not
    /// apply. Unknown keys on the OLD side only shrink the old candidate
    /// set and never block the proof.
    UnknownAttributeSemantics,
}

impl UnprovenReason {
    /// The stable machine name used by the corpus.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            UnprovenReason::RetainedPortResourceTypeChanged => {
                "retained_port_resource_type_changed"
            }
            UnprovenReason::UnknownAttributeSemantics => "unknown_attribute_semantics",
        }
    }
}

/// The static compatibility result: `Proven` means the new row provably
/// does not strengthen the old row (every candidate the old row accepted,
/// the new row accepts); `Unproven` fails closed — it is never judged
/// compatible and never enters the wire model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceCompatibilityProof {
    /// Non-strengthening holds.
    Proven,
    /// Non-strengthening could not be proven, with the reason.
    Unproven(UnprovenReason),
}

/// An abstract candidate resource descriptor: an exact nominal type pin
/// plus the attributes it carries. This is analysis input, NOT a runtime
/// resource instance, NOT a credential and NOT a handle; its wire form
/// belongs to the later binding stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceCandidate {
    /// The candidate's exact resource type.
    pub resource_type: ResourceTypeRef,
    /// The attributes the candidate carries (a superset of every
    /// requirement it satisfies is fine).
    pub attributes: Vec<ResourceAttributeConstraint>,
}

/// Normalized (key, value) view of one attribute, for comparisons.
fn normalized_pair(attribute: &ResourceAttributeConstraint) -> (String, String) {
    (
        attribute.key.nfc().collect::<String>(),
        attribute.value.nfc().collect::<String>(),
    )
}

/// `satisfies_resource_requirement`: the candidate's resource type must
/// exact-match the requirement's (all three nominal parts) and the
/// candidate must carry every required exact (key, value) pair. Extra
/// candidate attributes are fine — a requirement is a lower bound.
///
/// # Errors
/// [`ResourceError`] with kinds `invalid_resource_type` (malformed
/// identity), `invalid_resource_attribute` / `duplicate_resource_attribute`
/// (malformed candidate or requirement attribute lists) and
/// `resource_requirement_unsatisfied` (the predicate fails).
pub fn satisfies_resource_requirement(
    candidate: &ResourceCandidate,
    requirement: &ResourceRequirement,
) -> Result<(), ResourceError> {
    check_identity(&candidate.resource_type)?;
    check_identity(&requirement.resource_type)?;
    let candidate_attributes = canonical_attributes(&candidate.attributes)?;
    let required = canonical_attributes(&requirement.required_attributes)?;
    if !candidate
        .resource_type
        .is_same_type(&requirement.resource_type)
    {
        return Err(ResourceError::new(
            kind::RESOURCE_REQUIREMENT_UNSATISFIED,
            format!(
                "candidate resource type {}/{}/{:.8}… does not exact-match \
                 the required {}/{}/{:.8}…: same short name or similar \
                 shape is never accepted",
                candidate.resource_type.specification_id(),
                candidate.resource_type.type_name(),
                candidate.resource_type.schema_hex(),
                requirement.resource_type.specification_id(),
                requirement.resource_type.type_name(),
                requirement.resource_type.schema_hex()
            ),
        ));
    }
    let candidate_pairs: BTreeSet<(String, String)> =
        candidate_attributes.iter().map(normalized_pair).collect();
    for attribute in &required {
        let pair = normalized_pair(attribute);
        if !candidate_pairs.contains(&pair) {
            return Err(ResourceError::new(
                kind::RESOURCE_REQUIREMENT_UNSATISFIED,
                format!(
                    "candidate does not carry required attribute {}={:?}",
                    pair.0, pair.1
                ),
            ));
        }
    }
    Ok(())
}

/// `requirement_not_stronger`: proves that `new_row` accepts at least
/// every resource candidate `old_row` accepted, within ONE resource port
/// namespace. Both rows are named by their contract interface namespace;
/// comparing rows of two different namespaces is not a static proof and
/// fails closed — ports of different components are different resources
/// by default.
///
/// Per retained port, in canonical order:
///
/// 1. the resource type must keep its exact nominal identity — a change
///    is [`UnprovenReason::RetainedPortResourceTypeChanged`] (no implicit
///    subtyping);
/// 2. every attribute key the NEW side constrains must be in the type's
///    descriptor vocabulary — otherwise
///    [`UnprovenReason::UnknownAttributeSemantics`];
/// 3. the exact (key, value) conditions must satisfy
///    `new ⊆ old` — a new or changed condition proves strengthening
///    (`resource_requirement_strengthened`).
///
/// A port removed on the new side never strengthens (whether the
/// surrounding contract interface may drop a port at all is a decision of
/// the later full-contract compatibility checker, not of this R-dimension
/// proof).
///
/// # Errors
/// [`ResourceError`] with the structural kinds of
/// [`canonicalize_resource_requirement_row`], the universe resolution
/// kinds, `resource_requirement_unproven` (cross-namespace comparison)
/// and `resource_requirement_strengthened` (proven violation).
pub fn requirement_not_stronger(
    new_row: &ResourceRequirementRow,
    new_namespace: &str,
    old_row: &ResourceRequirementRow,
    old_namespace: &str,
    universe: &ResourceTypeUniverse,
) -> Result<ResourceCompatibilityProof, ResourceError> {
    let new_canonical = canonicalize_resource_requirement_row(new_row)?;
    let old_canonical = canonicalize_resource_requirement_row(old_row)?;
    if new_namespace != old_namespace {
        return Err(ResourceError::new(
            kind::RESOURCE_REQUIREMENT_UNPROVEN,
            format!(
                "resource port namespaces {new_namespace:?} and \
                 {old_namespace:?} are different contract interfaces: \
                 equal port numbers do not name the same resource, and no \
                 cross-component proof exists without an explicit port \
                 mapping"
            ),
        ));
    }
    // Every consulted type must resolve exactly; vocabulary lookups need
    // the descriptor.
    let mut descriptors = BTreeMap::new();
    for requirement in old_canonical
        .requirements
        .iter()
        .chain(new_canonical.requirements.iter())
    {
        let descriptor = universe.resolve(&requirement.resource_type)?;
        descriptors.insert(
            requirement.resource_type.nominal_key(),
            descriptor.recognized_attributes.clone(),
        );
    }
    let old_by_port: BTreeMap<u32, &ResourceRequirement> = old_canonical
        .requirements
        .iter()
        .map(|requirement| (requirement.port_no, requirement))
        .collect();
    for requirement in &new_canonical.requirements {
        let Some(old_requirement) = old_by_port.get(&requirement.port_no) else {
            return Err(ResourceError::new(
                kind::RESOURCE_REQUIREMENT_STRENGTHENED,
                format!(
                    "port {} requires a resource the previous contract did \
                     not require: the new implementation added a mandatory \
                     external resource",
                    requirement.port_no
                ),
            ));
        };
        if !requirement
            .resource_type
            .is_same_type(&old_requirement.resource_type)
        {
            return Ok(ResourceCompatibilityProof::Unproven(
                UnprovenReason::RetainedPortResourceTypeChanged,
            ));
        }
        let vocabulary = &descriptors[&requirement.resource_type.nominal_key()];
        if requirement
            .required_attributes
            .iter()
            .any(|attribute| !vocabulary.contains(&attribute.key))
        {
            return Ok(ResourceCompatibilityProof::Unproven(
                UnprovenReason::UnknownAttributeSemantics,
            ));
        }
        let old_pairs: BTreeSet<(String, String)> = old_requirement
            .required_attributes
            .iter()
            .map(normalized_pair)
            .collect();
        for attribute in &requirement.required_attributes {
            let pair = normalized_pair(attribute);
            if !old_pairs.contains(&pair) {
                return Err(ResourceError::new(
                    kind::RESOURCE_REQUIREMENT_STRENGTHENED,
                    format!(
                        "port {}: required attribute {}={:?} is new or \
                         changed: candidates satisfying the old contract \
                         may lack it",
                        requirement.port_no, pair.0, pair.1
                    ),
                ));
            }
        }
    }
    Ok(ResourceCompatibilityProof::Proven)
}
