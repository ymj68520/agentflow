//! Separate placement-feasibility and security-preservation compatibility.
use crate::canonical::canonicalize_execution_requirements;
use crate::error::ExecutionError;
use crate::model::{
    CapabilityRequirement, ComputeQuantityKind, ExecutionCapabilityRef, ExecutionRequirements,
    RegionRef, RegionResidencyRequirement,
};
use crate::universe::ExecutionUniverses;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityVerdict {
    Proven,
    Violation,
    Unproven,
}
impl CompatibilityVerdict {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Proven => "proven",
            Self::Violation => "violation",
            Self::Unproven => "unproven",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionCompatibility {
    pub feasibility: CompatibilityVerdict,
    pub security: CompatibilityVerdict,
}
impl ExecutionCompatibility {
    #[must_use]
    pub fn combined(&self) -> CompatibilityVerdict {
        if self.feasibility == CompatibilityVerdict::Unproven
            || self.security == CompatibilityVerdict::Unproven
        {
            CompatibilityVerdict::Unproven
        } else if self.feasibility == CompatibilityVerdict::Violation
            || self.security == CompatibilityVerdict::Violation
        {
            CompatibilityVerdict::Violation
        } else {
            CompatibilityVerdict::Proven
        }
    }
}

/// Conservative closed-rule comparison. Feasibility asks whether every
/// profile eligible for the old declaration remains eligible for the new
/// declaration. Security preservation is judged independently: a wider
/// placement set can still be a security relaxation.
pub fn execution_compatibility(
    new: &ExecutionRequirements,
    old: &ExecutionRequirements,
    u: &ExecutionUniverses,
) -> Result<ExecutionCompatibility, ExecutionError> {
    let new = canonicalize_execution_requirements(new, u)?;
    let old = canonicalize_execution_requirements(old, u)?;
    let mut feasibility = CompatibilityVerdict::Proven;
    let mut security = CompatibilityVerdict::Proven;

    let old_caps = capability_map(&old);
    let new_caps = capability_map(&new);
    for (key, required) in &new_caps {
        let Some(previous) = old_caps.get(key) else {
            feasibility = merge(feasibility, CompatibilityVerdict::Violation);
            continue;
        };
        if required.exact_version != previous.exact_version {
            feasibility = merge(feasibility, CompatibilityVerdict::Unproven);
            continue;
        }
        let previous_attributes: BTreeSet<_> = previous.required_attributes.iter().collect();
        let required_attributes: BTreeSet<_> = required.required_attributes.iter().collect();
        if !required_attributes.is_subset(&previous_attributes) {
            feasibility = merge(feasibility, CompatibilityVerdict::Violation);
        }
    }
    let old_quantities = quantities(&old);
    let new_quantities = quantities(&new);
    for kind in old_quantities.keys().chain(new_quantities.keys()) {
        match (new_quantities.get(kind), old_quantities.get(kind)) {
            (Some(new_value), Some(old_value)) if new_value > old_value => {
                feasibility = merge(feasibility, CompatibilityVerdict::Violation);
            }
            (Some(_), None) => feasibility = merge(feasibility, CompatibilityVerdict::Violation),
            _ => {}
        }
    }

    match (new.sandbox.as_ref(), old.sandbox.as_ref()) {
        (Some(new_value), Some(old_value))
            if new_value.minimum.value_no() > old_value.minimum.value_no() =>
        {
            feasibility = merge(feasibility, CompatibilityVerdict::Violation);
        }
        (Some(new_value), Some(old_value))
            if new_value.minimum.value_no() < old_value.minimum.value_no() =>
        {
            security = merge(security, CompatibilityVerdict::Violation);
        }
        (Some(_), None) => feasibility = merge(feasibility, CompatibilityVerdict::Violation),
        (None, Some(_)) => security = merge(security, CompatibilityVerdict::Violation),
        _ => {}
    }

    if new.trust_domain != old.trust_domain {
        security = merge(
            security,
            if new.trust_domain.is_none() {
                CompatibilityVerdict::Violation
            } else {
                CompatibilityVerdict::Unproven
            },
        );
    }
    let (new_execution_regions, new_residency_regions) =
        region_lists(new.region_residency.as_ref());
    let (old_execution_regions, old_residency_regions) =
        region_lists(old.region_residency.as_ref());
    apply_region_direction(
        region_direction(&new_execution_regions, &old_execution_regions),
        &mut feasibility,
        &mut security,
    );
    apply_region_direction(
        region_direction(&new_residency_regions, &old_residency_regions),
        &mut feasibility,
        &mut security,
    );
    if new.runtime_profile != old.runtime_profile
        || new.implementation_abis != old.implementation_abis
    {
        feasibility = merge(feasibility, CompatibilityVerdict::Unproven);
    }
    Ok(ExecutionCompatibility {
        feasibility,
        security,
    })
}

fn capability_map(
    requirements: &ExecutionRequirements,
) -> BTreeMap<ExecutionCapabilityRef, &CapabilityRequirement> {
    requirements
        .capabilities
        .iter()
        .map(|requirement| (requirement.capability.clone(), requirement))
        .collect()
}

fn quantities(requirements: &ExecutionRequirements) -> BTreeMap<ComputeQuantityKind, u64> {
    requirements
        .quantities
        .iter()
        .map(|quantity| (quantity.kind, quantity.minimum))
        .collect()
}

fn merge(left: CompatibilityVerdict, right: CompatibilityVerdict) -> CompatibilityVerdict {
    if left == CompatibilityVerdict::Violation || right == CompatibilityVerdict::Violation {
        CompatibilityVerdict::Violation
    } else if left == CompatibilityVerdict::Unproven || right == CompatibilityVerdict::Unproven {
        CompatibilityVerdict::Unproven
    } else {
        CompatibilityVerdict::Proven
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegionDirection {
    Same,
    Tightened,
    Expanded,
    Replaced,
}

fn region_direction(new: &[RegionRef], old: &[RegionRef]) -> RegionDirection {
    let new: BTreeSet<_> = new.iter().collect();
    let old: BTreeSet<_> = old.iter().collect();
    if new == old {
        RegionDirection::Same
    } else if new.is_empty() {
        RegionDirection::Expanded
    } else if old.is_empty() || new.is_subset(&old) {
        RegionDirection::Tightened
    } else if old.is_subset(&new) {
        RegionDirection::Expanded
    } else {
        RegionDirection::Replaced
    }
}

fn region_lists(
    requirement: Option<&RegionResidencyRequirement>,
) -> (Vec<RegionRef>, Vec<RegionRef>) {
    requirement.map_or_else(
        || (Vec::new(), Vec::new()),
        |requirement| {
            (
                requirement.execution_regions.clone(),
                requirement.residency_regions.clone(),
            )
        },
    )
}

fn apply_region_direction(
    direction: RegionDirection,
    feasibility: &mut CompatibilityVerdict,
    security: &mut CompatibilityVerdict,
) {
    match direction {
        RegionDirection::Same => {}
        RegionDirection::Tightened => {
            *feasibility = merge(*feasibility, CompatibilityVerdict::Violation);
        }
        RegionDirection::Expanded => *security = merge(*security, CompatibilityVerdict::Violation),
        RegionDirection::Replaced => {
            *feasibility = merge(*feasibility, CompatibilityVerdict::Unproven);
            *security = merge(*security, CompatibilityVerdict::Unproven);
        }
    }
}
