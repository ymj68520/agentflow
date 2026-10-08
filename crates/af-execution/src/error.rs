//! Static Execution Requirement errors.
use std::fmt;
pub mod kind {
    pub const INVALID_EXECUTION_CAPABILITY: &str = "invalid_execution_capability";
    pub const UNKNOWN_EXECUTION_CAPABILITY: &str = "unknown_execution_capability";
    pub const UNSUPPORTED_CAPABILITY_VERSION: &str = "unsupported_capability_version";
    pub const CAPABILITY_REQUIREMENT_UNPROVEN: &str = "capability_requirement_unproven";
    pub const INVALID_COMPUTE_QUANTITY: &str = "invalid_compute_quantity";
    pub const QUANTITY_UNIT_MISMATCH: &str = "quantity_unit_mismatch";
    pub const INVALID_SANDBOX_REQUIREMENT: &str = "invalid_sandbox_requirement";
    pub const SANDBOX_ASSURANCE_UNPROVEN: &str = "sandbox_assurance_unproven";
    pub const UNKNOWN_TRUST_DOMAIN: &str = "unknown_trust_domain";
    pub const TRUST_DOMAIN_MISMATCH: &str = "trust_domain_mismatch";
    pub const UNKNOWN_REGION: &str = "unknown_region";
    pub const RESIDENCY_REQUIREMENT_UNPROVEN: &str = "residency_requirement_unproven";
    pub const UNKNOWN_RUNTIME_PROFILE: &str = "unknown_runtime_profile";
    pub const RUNTIME_ABI_INCOMPATIBLE: &str = "runtime_abi_incompatible";
    pub const DUPLICATE_EXECUTION_REQUIREMENT: &str = "duplicate_execution_requirement";
    pub const CONFLICTING_HARD_CONSTRAINT: &str = "conflicting_hard_constraint";
    pub const EXECUTION_PROFILE_REJECTED: &str = "execution_profile_rejected";
    pub const EXECUTION_PROFILE_UNPROVEN: &str = "execution_profile_unproven";
    pub const EXECUTION_FEASIBILITY_STRENGTHENED: &str = "execution_feasibility_strengthened";
    pub const EXECUTION_SECURITY_RELAXED: &str = "execution_security_relaxed";
    pub const EXECUTION_COMPATIBILITY_UNPROVEN: &str = "execution_compatibility_unproven";
    pub const EXECUTION_ENCODING_ERROR: &str = "execution_encoding_error";
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionError {
    kind: &'static str,
    detail: String,
}
impl ExecutionError {
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
impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.detail)
    }
}
impl std::error::Error for ExecutionError {}
