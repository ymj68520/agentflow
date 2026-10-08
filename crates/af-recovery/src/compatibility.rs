//! Conservative Recovery Contract compatibility proof.

use crate::canonical::canonicalize_recovery_contract;
use crate::error::RecoveryError;
use crate::model::RecoveryContract;
use crate::universe::RecoveryOperationUniverse;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryCompatibilityProof {
    Proven,
    Weakened,
    Unproven,
}

impl RecoveryCompatibilityProof {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Proven => "proven",
            Self::Weakened => "weakened",
            Self::Unproven => "unproven",
        }
    }
}

/// Only exact-equivalent declarations are proven compatible in this phase.
/// Recovery modes are not an invented total order.
pub fn recovery_not_weakened(
    new_contract: &RecoveryContract,
    old_contract: &RecoveryContract,
    universe: &RecoveryOperationUniverse,
) -> Result<RecoveryCompatibilityProof, RecoveryError> {
    let new = canonicalize_recovery_contract(new_contract, universe)?;
    let old = canonicalize_recovery_contract(old_contract, universe)?;
    if new == old {
        return Ok(RecoveryCompatibilityProof::Proven);
    }
    if new.effect_recovery_clauses.len() < old.effect_recovery_clauses.len() {
        return Ok(RecoveryCompatibilityProof::Weakened);
    }
    Ok(RecoveryCompatibilityProof::Unproven)
}
