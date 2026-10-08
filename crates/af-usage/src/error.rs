//! Stable errors for the static Usage layer.

use std::fmt;

/// Machine-readable Usage error kinds.
pub mod kind {
    pub const INVALID_USAGE_METER: &str = "invalid_usage_meter";
    pub const UNKNOWN_USAGE_METER: &str = "unknown_usage_meter";
    pub const INCOMPATIBLE_METER_UNIT: &str = "incompatible_meter_unit";
    pub const INVALID_USAGE_PHASE: &str = "invalid_usage_phase";
    pub const DUPLICATE_USAGE_LIMIT: &str = "duplicate_usage_limit";
    pub const UNSUPPORTED_USAGE_AGGREGATION: &str = "unsupported_usage_aggregation";
    pub const USAGE_ARITHMETIC_OVERFLOW: &str = "usage_arithmetic_overflow";
    pub const USAGE_BOUND_UNPROVEN: &str = "usage_bound_unproven";
    pub const USAGE_CEILING_EXCEEDED: &str = "usage_ceiling_exceeded";
    pub const USAGE_COMPATIBILITY_UNPROVEN: &str = "usage_compatibility_unproven";
    pub const USAGE_ENCODING_ERROR: &str = "usage_encoding_error";
}

/// A structured static Usage error. It is never a quota, billing, policy or
/// workflow failure result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageError {
    kind: &'static str,
    detail: String,
}

impl UsageError {
    /// Builds one stable error.
    pub fn new(kind: &'static str, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    /// Returns the stable machine kind.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        self.kind
    }

    /// Returns the human-readable diagnostic.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for UsageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.detail)
    }
}

impl std::error::Error for UsageError {}
