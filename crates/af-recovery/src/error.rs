//! Stable static Recovery errors.

use std::fmt;

pub mod kind {
    pub const INVALID_REPLAY_SEMANTICS: &str = "invalid_replay_semantics";
    pub const INVALID_IDEMPOTENCY_CONTRACT: &str = "invalid_idempotency_contract";
    pub const DUPLICATE_EFFECT_RECOVERY_CLAUSE: &str = "duplicate_effect_recovery_clause";
    pub const UNKNOWN_RECOVERY_OPERATION: &str = "unknown_recovery_operation";
    pub const RECOVERY_EFFECT_NOT_DECLARED: &str = "recovery_effect_not_declared";
    pub const RECONCILIATION_REQUIRED: &str = "reconciliation_required";
    pub const RECONCILIATION_EVIDENCE_MISSING: &str = "reconciliation_evidence_missing";
    pub const IDEMPOTENCY_EVIDENCE_MISSING: &str = "idempotency_evidence_missing";
    pub const COMPENSATION_OBLIGATION_UNRESOLVED: &str = "compensation_obligation_unresolved";
    pub const UNSAFE_BLIND_RETRY: &str = "unsafe_blind_retry";
    pub const RECOVERY_CONTRACT_UNPROVEN: &str = "recovery_contract_unproven";
    pub const RECOVERY_CONTRACT_WEAKENED: &str = "recovery_contract_weakened";
    pub const RECOVERY_ENCODING_ERROR: &str = "recovery_encoding_error";
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryError {
    kind: &'static str,
    detail: String,
}

impl RecoveryError {
    pub fn new(kind: &'static str, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    #[must_use]
    pub fn kind(&self) -> &'static str {
        self.kind
    }

    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.detail)
    }
}

impl std::error::Error for RecoveryError {}
