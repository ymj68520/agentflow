//! Exact-pinned validation against the explicit resource-type universe.
//!
//! Validation is canonicalization (structural) plus, per requirement in
//! canonical port order: the resource type resolves exactly (exists,
//! digest matches, explicitly recognized, descriptor coherent), and every
//! attribute key is part of that type's descriptor vocabulary — an
//! unrecognized key has no known constraint meaning and is rejected
//! rather than guessed at.

use crate::canonical::canonicalize_resource_requirement_row;
use crate::error::{ResourceError, kind};
use crate::model::ResourceRequirementRow;
use crate::universe::ResourceTypeUniverse;

impl ResourceRequirementRow {
    /// Validates every requirement against the explicit universe and
    /// returns the canonical row.
    ///
    /// # Errors
    /// [`ResourceError`] with the structural kinds of
    /// [`canonicalize_resource_requirement_row`], then, per requirement in
    /// canonical port order: `unknown_resource_type`,
    /// `resource_type_digest_mismatch`, `invalid_resource_type` and
    /// `unknown_resource_attribute`.
    pub fn validate(
        &self,
        universe: &ResourceTypeUniverse,
    ) -> Result<ResourceRequirementRow, ResourceError> {
        let canonical = canonicalize_resource_requirement_row(self)?;
        for requirement in &canonical.requirements {
            let descriptor = universe.resolve(&requirement.resource_type)?;
            for attribute in &requirement.required_attributes {
                if !descriptor.recognized_attributes.contains(&attribute.key) {
                    return Err(ResourceError::new(
                        kind::UNKNOWN_RESOURCE_ATTRIBUTE,
                        format!(
                            "attribute key {:?} is not in the descriptor \
                             vocabulary of {}/{}: an unknown attribute has \
                             no known constraint meaning",
                            attribute.key,
                            requirement.resource_type.specification_id(),
                            requirement.resource_type.type_name()
                        ),
                    ));
                }
            }
        }
        Ok(canonical)
    }
}
