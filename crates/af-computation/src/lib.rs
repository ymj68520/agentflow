//! Pure, explicit static analysis for Semantic IR computation leaves and
//! selected structured control nodes.
//!
//! Leaf resolution handles `INVOKE`, `AWAIT`, and `CALL_SUBFLOW`. The
//! control algebra composes only `SEQUENCE`, `SELECT`, `PARALLEL`, and
//! `RACE`, consuming the checked IR, Phase 3C dataflow, resolved leaves, and
//! the same immutable proof context. Other control nodes remain explicitly
//! unsupported/Unproven. No summary is a ComponentContract, digest input,
//! runtime binding, grant, handle, plan, or workflow attestation.

mod control;
mod mapping;
mod proof;
mod resolution;
mod summary;

pub use control::{
    ControlDerivationError, ControlDimension, ControlDimensionProof, ControlDimensionValue,
    ControlFlowContractSource, ControlFlowSummary, ControlInterfaceSummary, ControlOperator,
    ControlOutputPort, ControlOutputSource, ControlProofReport, ControlResourceAlias,
    DerivedControlContractSummary, DimensionContribution, EffectLowerBoundAdjustment,
    EffectLowerBoundAdjustmentReason, ExecutionSourceSummary, InheritedControlObligation,
    RecoverySourceSummary, ResourcePortOccurrence, SelectDecisionEvidence,
    SequenceDataflowDependency, derive_control_contract,
};
pub use proof::{LeafDimensionProof, LeafProofReport, LeafVerdict};
pub use resolution::{
    LeafResolutionError, SnapshotError, StaticComputationResolutionSnapshot,
    StaticComputationResolutionSnapshotBuilder, resolve_computation_leaf,
};
pub use summary::{
    AwaitAuthorityStatus, AwaitInteractionStatus, AwaitLeafSummary, AwaitProvenanceStatus,
    AwaitTimeoutSemantics, ComponentDimensionSummary, FlowPortMapping, FlowSinkBinding,
    FlowSourceBinding, LeafKind, ProofContextReference, ResolvedComputationLeaf, ResolvedTarget,
    ResourceAliasGroup, ScopedEffectOccurrence, ScopedEffectResidual, ScopedPortBinding,
    ScopedPortMapping, ScopedResourcePortMapping,
};
