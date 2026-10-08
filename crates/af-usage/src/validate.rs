//! Structural Usage Signature validation helpers.

use crate::error::UsageError;
use crate::model::UsageMeterRef;
use crate::universe::normalized_meter;

/// Validates and normalizes a meter reference.
pub fn validate_meter(meter: &UsageMeterRef) -> Result<UsageMeterRef, UsageError> {
    normalized_meter(meter)
}
