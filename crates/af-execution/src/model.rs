//! Typed static Execution Requirements X model.

use af_semantic::TypeSchemaRef;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionCapabilityRef {
    pub catalog: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityAttribute {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityRequirement {
    pub capability: ExecutionCapabilityRef,
    pub exact_version: String,
    pub required_attributes: Vec<CapabilityAttribute>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComputeQuantityKind {
    CpuMillicores,
    MemoryBytes,
    GpuCount,
}

impl ComputeQuantityKind {
    #[must_use]
    pub fn value_no(self) -> u32 {
        match self {
            Self::CpuMillicores => 1,
            Self::MemoryBytes => 2,
            Self::GpuCount => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComputeQuantityRequirement {
    pub kind: ComputeQuantityKind,
    pub minimum: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SandboxLevel {
    Basic,
    Strong,
}

impl SandboxLevel {
    #[must_use]
    pub fn value_no(self) -> u32 {
        match self {
            Self::Basic => 1,
            Self::Strong => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SandboxRequirement {
    pub minimum: SandboxLevel,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrustDomainRef {
    pub catalog: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RegionRef {
    pub catalog: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegionResidencyRequirement {
    pub execution_regions: Vec<RegionRef>,
    pub residency_regions: Vec<RegionRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeProfileRef {
    pub catalog: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeProfileRequirement {
    pub family: String,
    pub profile: RuntimeProfileRef,
    pub required_features: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImplementationAbiRequirement {
    pub schema: TypeSchemaRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlacementPreference {
    ArtifactLocality,
    WarmModelCache,
    LowerLoad,
    LowerCost,
}

impl PlacementPreference {
    #[must_use]
    pub fn value_no(self) -> u32 {
        match self {
            Self::ArtifactLocality => 1,
            Self::WarmModelCache => 2,
            Self::LowerLoad => 3,
            Self::LowerCost => 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionRequirements {
    pub capabilities: Vec<CapabilityRequirement>,
    pub quantities: Vec<ComputeQuantityRequirement>,
    pub sandbox: Option<SandboxRequirement>,
    pub trust_domain: Option<TrustDomainRef>,
    pub region_residency: Option<RegionResidencyRequirement>,
    pub runtime_profile: Option<RuntimeProfileRequirement>,
    pub implementation_abis: Vec<ImplementationAbiRequirement>,
    pub soft_preferences: Vec<PlacementPreference>,
}

impl ExecutionRequirements {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            capabilities: vec![],
            quantities: vec![],
            sandbox: None,
            trust_domain: None,
            region_residency: None,
            runtime_profile: None,
            implementation_abis: vec![],
            soft_preferences: vec![],
        }
    }
}
