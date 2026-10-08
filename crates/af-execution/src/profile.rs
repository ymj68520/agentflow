//! Non-wire static executor profile descriptor.
use crate::model::{
    ComputeQuantityKind, ExecutionCapabilityRef, RegionRef, RuntimeProfileRef, SandboxLevel,
    TrustDomainRef,
};
use af_semantic::TypeSchemaRef;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticQuantity {
    pub kind: ComputeQuantityKind,
    pub amount: u64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticCapability {
    pub capability: ExecutionCapabilityRef,
    pub exact_version: String,
    pub attributes: BTreeSet<(String, String)>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticSandbox {
    pub level: SandboxLevel,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticExecutorProfile {
    pub capabilities: Vec<StaticCapability>,
    pub quantities: Vec<StaticQuantity>,
    pub sandbox: Option<SandboxLevel>,
    pub trust_domain: Option<TrustDomainRef>,
    pub execution_region: Option<RegionRef>,
    pub residency_regions: Vec<RegionRef>,
    pub runtime_profile: Option<RuntimeProfileRef>,
    pub implementation_abis: Vec<TypeSchemaRef>,
}
