//! Static, typed ComponentContract assembly and cross-dimension proof.
//!
//! This crate has no runtime or authorization surface. It consumes explicit
//! immutable universes and reports static constraints only.

pub mod canonical;
pub mod compatibility;
pub mod digest;
pub mod effect_flow;
pub mod effect_resource;
pub mod error;
pub mod flow_execution;
pub mod model;
pub mod ports;
pub mod proof;
pub mod recovery_effect;
pub mod recovery_guard;
pub mod recovery_usage;
pub mod universe;
pub mod validate;

pub use canonical::{CanonicalComponentContract, canonicalize_contract};
pub use compatibility::{CompatibilityReport, check_substitution};
pub use digest::component_contract_digest;
pub use error::ContractError;
pub use model::{
    ComponentContract, ComponentIdentity, ComponentInputPort, ComponentOutputPort,
    EffectFlowPartition, RawComponentContract, RecoveryActionBinding, RecoveryActionIdentity,
    RecoveryActionKind, RecoveryUsageBinding,
};
pub use proof::CheckedComponentContract;
pub use proof::{
    CatalogContentChecksum, DimensionProof, ProofReport, ProofVerdict, RelationCatalogPin,
};
pub use recovery_guard::{
    ActionGuardCompatibility, GuardCompatibilityBasis, GuardCountermodelReachability,
    GuardInputAssignment, TypedGuardCountermodel, analyze_action_guard_compatibility,
};
pub use universe::{
    AuthorityExecutionRule, ConfidentialityExecutionRule, EffectQualifierProfile,
    EffectResourceCompatibilityCatalog, FlowExecutionRule, FlowExecutionSink,
    InformationFlowExecutionCatalog, RecoveryUsageCoverageCatalog, RecoveryUsageCoverageRule,
    RegionPair, RegionPairPolicy, RelationCatalogSnapshot, RelationCatalogWire, RelationSnapshots,
    SandboxFloor, ScopeValueRule, SelectorKeyRule, SupportedEffectResourcePair, TaintExecutionRule,
    TrustDomainPolicy,
};
pub use validate::{
    StaticContractProofContext, analyze_contract, check_contract, validate_raw_contract,
};
