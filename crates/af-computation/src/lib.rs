//! Pure, explicit static analysis for Semantic IR computation leaves and
//! selected structured/advanced control nodes.
//!
//! Leaf resolution handles `INVOKE`, `AWAIT`, and `CALL_SUBFLOW`. The
//! control algebra composes `SEQUENCE`, `SELECT`, `PARALLEL`, `RACE`,
//! `ITERATE`, `RETRY`, `TIMEOUT`, and `HANDLE`, consuming the checked IR,
//! Phase 3C dataflow, resolved leaves, and the same immutable proof context.
//! `SCOPE` and `COMPENSATION_SCOPE` remain explicitly unsupported/Unproven.
//! No summary is a ComponentContract, digest input, runtime binding, grant,
//! handle, plan, or workflow attestation.

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
    EffectLowerBoundAdjustmentReason, ExecutionSourceSummary, HandleClauseSummary,
    HandleMatcherSummary, InheritedControlObligation, IterateMode, RecoverySourceSummary,
    ResourcePortOccurrence, SelectDecisionEvidence, SequenceDataflowDependency, TimeoutOutcome,
    derive_control_contract,
};
pub use proof::{LeafDimensionProof, LeafProofReport, LeafVerdict};
pub use resolution::{
    LeafResolutionError, SnapshotError, StaticComputationResolutionSnapshot,
    StaticComputationResolutionSnapshotBuilder, resolve_computation_leaf,
};
pub use summary::{
    AwaitAuthorityStatus, AwaitInteractionStatus, AwaitLeafSummary, AwaitProvenanceStatus,
    AwaitTimeoutSemantics, ComponentDimensionSummary, EffectOccurrenceFamily,
    EffectOccurrenceFamilyKind, FlowPortMapping, FlowSinkBinding, FlowSourceBinding, LeafKind,
    ProofContextReference, ResolvedComputationLeaf, ResolvedTarget, ResourceAliasGroup,
    ScopedEffectOccurrence, ScopedEffectResidual, ScopedPortBinding, ScopedPortMapping,
    ScopedResourcePortMapping,
};
