//! Static profile matching; no live placement or scheduler state.
use crate::canonical::canonicalize_execution_requirements;
use crate::error::ExecutionError;
use crate::model::ExecutionRequirements;
use crate::profile::StaticExecutorProfile;
use crate::universe::ExecutionUniverses;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchRejected {
    CapabilityMissing,
    CapabilityVersion,
    QuantityInsufficient,
    SandboxTooWeak,
    TrustDomainMismatch,
    RegionMismatch,
    ResidencyMismatch,
    RuntimeProfileMismatch,
    AbiMismatch,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchUnproven {
    CapabilityProfile,
    QuantityProfile,
    SandboxProfile,
    TrustDomain,
    RegionResidency,
    RuntimeProfile,
    AbiProfile,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaticPlacementProof {
    ProvenAgainstDescriptor,
    Rejected(MatchRejected),
    Unproven(MatchUnproven),
}
impl StaticPlacementProof {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ProvenAgainstDescriptor => "proven_against_descriptor",
            Self::Rejected(_) => "rejected",
            Self::Unproven(_) => "unproven",
        }
    }
}

pub fn match_execution_requirements(
    req: &ExecutionRequirements,
    profile: &StaticExecutorProfile,
    u: &ExecutionUniverses,
) -> Result<StaticPlacementProof, ExecutionError> {
    let req = canonicalize_execution_requirements(req, u)?;
    for required in &req.capabilities {
        let Some(candidate) = profile
            .capabilities
            .iter()
            .find(|x| x.capability == required.capability)
        else {
            return Ok(StaticPlacementProof::Rejected(
                MatchRejected::CapabilityMissing,
            ));
        };
        if candidate.exact_version != required.exact_version {
            return Ok(StaticPlacementProof::Rejected(
                MatchRejected::CapabilityVersion,
            ));
        }
        let attrs = candidate.attributes.clone();
        for a in &required.required_attributes {
            if !attrs.contains(&(a.key.clone(), a.value.clone())) {
                return Ok(StaticPlacementProof::Rejected(
                    MatchRejected::CapabilityMissing,
                ));
            }
        }
    }
    for required in &req.quantities {
        let Some(candidate) = profile.quantities.iter().find(|x| x.kind == required.kind) else {
            return Ok(StaticPlacementProof::Unproven(
                MatchUnproven::QuantityProfile,
            ));
        };
        if candidate.amount < required.minimum {
            return Ok(StaticPlacementProof::Rejected(
                MatchRejected::QuantityInsufficient,
            ));
        }
    }
    if let Some(sandbox) = req.sandbox {
        let Some(candidate) = profile.sandbox else {
            return Ok(StaticPlacementProof::Unproven(
                MatchUnproven::SandboxProfile,
            ));
        };
        if candidate.value_no() < sandbox.minimum.value_no() {
            return Ok(StaticPlacementProof::Rejected(
                MatchRejected::SandboxTooWeak,
            ));
        }
    }
    if let Some(domain) = req.trust_domain {
        let Some(candidate) = &profile.trust_domain else {
            return Ok(StaticPlacementProof::Unproven(MatchUnproven::TrustDomain));
        };
        if candidate != &domain {
            return Ok(StaticPlacementProof::Rejected(
                MatchRejected::TrustDomainMismatch,
            ));
        }
    }
    if let Some(region) = req.region_residency {
        let Some(candidate) = &profile.execution_region else {
            return Ok(StaticPlacementProof::Unproven(
                MatchUnproven::RegionResidency,
            ));
        };
        if !region.execution_regions.is_empty() && !region.execution_regions.contains(candidate) {
            return Ok(StaticPlacementProof::Rejected(
                MatchRejected::RegionMismatch,
            ));
        }
        if !region.residency_regions.is_empty()
            && !profile
                .residency_regions
                .iter()
                .any(|x| region.residency_regions.contains(x))
        {
            return Ok(StaticPlacementProof::Rejected(
                MatchRejected::ResidencyMismatch,
            ));
        }
    }
    if let Some(runtime) = req.runtime_profile {
        let Some(candidate) = &profile.runtime_profile else {
            return Ok(StaticPlacementProof::Unproven(
                MatchUnproven::RuntimeProfile,
            ));
        };
        if candidate != &runtime.profile {
            return Ok(StaticPlacementProof::Rejected(
                MatchRejected::RuntimeProfileMismatch,
            ));
        }
    }
    for abi in &req.implementation_abis {
        if !profile.implementation_abis.contains(&abi.schema) {
            return Ok(StaticPlacementProof::Unproven(MatchUnproven::AbiProfile));
        }
    }
    Ok(StaticPlacementProof::ProvenAgainstDescriptor)
}
