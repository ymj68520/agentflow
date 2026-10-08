//! Static Usage non-increase compatibility.

use crate::algebra::{UsageProof, UsageUnprovenReason};
use crate::canonical::canonicalize_usage_signature;
use crate::error::UsageError;
use crate::model::{UsageSignature, UsageUpperBound};
use crate::universe::UsageMeterUniverse;
use std::collections::BTreeMap;

/// Proves the new static usage declaration does not increase the old one.
/// Meter identity, phase and unit must all match; missing declarations are
/// `Unproven`, never implicit zero.
pub fn usage_not_increased(
    new_signature: &UsageSignature,
    new_universe: &UsageMeterUniverse,
    old_signature: &UsageSignature,
    old_universe: &UsageMeterUniverse,
) -> Result<UsageProof, UsageError> {
    let new = canonicalize_usage_signature(new_signature, new_universe)?;
    let old = canonicalize_usage_signature(old_signature, old_universe)?;
    if new.limits.is_empty() || old.limits.is_empty() {
        return Ok(UsageProof::Unproven(UsageUnprovenReason::MissingMeter));
    }

    let new_by_key: BTreeMap<_, _> = new
        .limits
        .iter()
        .map(|limit| ((limit.phase, limit.meter.clone()), limit))
        .collect();
    let old_by_key: BTreeMap<_, _> = old
        .limits
        .iter()
        .map(|limit| ((limit.phase, limit.meter.clone()), limit))
        .collect();
    if new_by_key.len() != old_by_key.len()
        || new_by_key.keys().any(|key| !old_by_key.contains_key(key))
    {
        return Ok(UsageProof::Unproven(UsageUnprovenReason::MissingMeter));
    }
    for (key, new_limit) in &new_by_key {
        let old_limit = old_by_key[key];
        let new_descriptor = new_universe.resolve(&new_limit.meter)?;
        let old_descriptor = old_universe.resolve(&old_limit.meter)?;
        if new_descriptor.unit != old_descriptor.unit
            || new_descriptor.aggregation != old_descriptor.aggregation
        {
            return Ok(UsageProof::Unproven(
                UsageUnprovenReason::IncompatibleMeterUnit,
            ));
        }
        match (new_limit.upper, old_limit.upper) {
            (UsageUpperBound::Unbounded, UsageUpperBound::Finite(_)) => {
                return Ok(UsageProof::Violation);
            }
            (UsageUpperBound::Finite(new), UsageUpperBound::Finite(old)) if new > old => {
                return Ok(UsageProof::Violation);
            }
            _ => {}
        }
        let _ = key;
    }
    Ok(UsageProof::Proven)
}
