//! Pure, explicit static resolution for Semantic IR computation leaves.
//!
//! This crate deliberately resolves only `INVOKE`, `AWAIT`, and
//! `CALL_SUBFLOW`. It consumes checked IR, Phase 3C/3D analysis inputs, exact
//! checked ComponentContracts, and an immutable caller-built snapshot. It
//! creates no runtime bindings, grants, handles, plans, or workflow-level
//! contract algebra.

mod mapping;
mod proof;
mod resolution;
mod summary;

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
