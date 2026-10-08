//! `af-recovery` — static Recovery Contract K.
//!
//! This crate declares replay/idempotency/reconciliation/compensation
//! obligations and performs pure static proofs. It creates no receipt,
//! grant, key, journal, runtime state or recovery execution.

pub mod canonical;
pub mod compatibility;
pub mod error;
pub mod model;
pub mod obligations;
pub mod universe;

pub use canonical::canonicalize_recovery_contract;
pub use compatibility::{RecoveryCompatibilityProof, recovery_not_weakened};
pub use error::{RecoveryError, kind};
pub use model::{
    CompensationRequirement, EffectRecoveryClause, IdempotencySemantics, KeyStabilityRequirement,
    ReconciliationRequirement, RecoveryContract, ReplaySemantics, UnknownEffectHandling,
};
pub use obligations::{
    RecoveryEvidenceState, RecoveryObligation, RecoveryProof, RecoveryUnprovenReason,
    RecoveryViolation, analyze_recovery_safety,
};
pub use universe::RecoveryOperationUniverse;
