//! Stable static Cost / Budget error kinds.

use std::fmt;

/// Machine-readable cost-projection errors.
pub mod cost_kind {
    pub const INVALID_CURRENCY: &str = "invalid_currency";
    pub const UNKNOWN_CURRENCY: &str = "unknown_currency";
    pub const CURRENCY_UNIT_MISMATCH: &str = "currency_unit_mismatch";
    pub const INVALID_PRICE_RATE: &str = "invalid_price_rate";
    pub const DUPLICATE_PRICE_ENTRY: &str = "duplicate_price_entry";
    pub const UNKNOWN_PRICE_METER: &str = "unknown_price_meter";
    pub const PRICE_METER_UNIT_MISMATCH: &str = "price_meter_unit_mismatch";
    pub const UNSUPPORTED_PRICING_MODEL: &str = "unsupported_pricing_model";
    pub const UNSUPPORTED_ROUNDING_RULE: &str = "unsupported_rounding_rule";
    pub const COST_ARITHMETIC_OVERFLOW: &str = "cost_arithmetic_overflow";
    pub const MISSING_PRICE_COVERAGE: &str = "missing_price_coverage";
    pub const COST_PROJECTION_UNPROVEN: &str = "cost_projection_unproven";
    pub const STATIC_BUDGET_EXCEEDED: &str = "static_budget_exceeded";
    pub const STATIC_BUDGET_UNPROVEN: &str = "static_budget_unproven";
    pub const COST_COMPATIBILITY_UNPROVEN: &str = "cost_compatibility_unproven";
    pub const COST_ENCODING_ERROR: &str = "cost_encoding_error";
}

/// Structured static Cost error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostError {
    kind: &'static str,
    detail: String,
}

impl CostError {
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

impl fmt::Display for CostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.detail)
    }
}

impl std::error::Error for CostError {}
