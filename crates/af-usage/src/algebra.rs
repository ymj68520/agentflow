//! Pure static Usage algebra.

use crate::error::{UsageError, kind};
use crate::model::{StaticUsageBound, StaticUsageEnvelope, UsageCoordinate, UsageUpperBound};
use std::collections::BTreeSet;

/// A conservative proof result. `Unproven` is not `Proven`; `Violation` is
/// a proven bound failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsageProof {
    Proven,
    Unproven(UsageUnprovenReason),
    Violation,
}

impl UsageProof {
    /// Stable machine spelling.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Proven => "proven",
            Self::Unproven(_) => "unproven",
            Self::Violation => "violation",
        }
    }

    /// Demands a proven static bound or compatibility result.
    pub fn require_proven(self) -> Result<(), UsageError> {
        match self {
            Self::Proven => Ok(()),
            Self::Unproven(reason) => Err(UsageError::new(
                kind::USAGE_BOUND_UNPROVEN,
                format!("usage proof is unproven: {}", reason.as_str()),
            )),
            Self::Violation => Err(UsageError::new(
                kind::USAGE_CEILING_EXCEEDED,
                "the static usage ceiling is exceeded",
            )),
        }
    }

    /// Demands a proven non-increase result, using the compatibility-specific
    /// stable error for an unproven comparison.
    pub fn require_compatibility(self) -> Result<(), UsageError> {
        match self {
            Self::Proven => Ok(()),
            Self::Unproven(reason) => Err(UsageError::new(
                kind::USAGE_COMPATIBILITY_UNPROVEN,
                format!("usage compatibility is unproven: {}", reason.as_str()),
            )),
            Self::Violation => Err(UsageError::new(
                kind::USAGE_CEILING_EXCEEDED,
                "the new usage declaration is an increase",
            )),
        }
    }
}

/// Why a static Usage proof could not be established.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsageUnprovenReason {
    MissingMeter,
    UnknownBound,
    IncompatibleMeterUnit,
}

impl UsageUnprovenReason {
    /// Stable machine spelling.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MissingMeter => "missing_meter",
            Self::UnknownBound => "unknown_bound",
            Self::IncompatibleMeterUnit => "incompatible_meter_unit",
        }
    }
}

fn coordinate_set(a: &StaticUsageEnvelope, b: &StaticUsageEnvelope) -> BTreeSet<UsageCoordinate> {
    a.iter()
        .chain(b.iter())
        .map(|(coordinate, _)| coordinate.clone())
        .collect()
}

fn to_upper(bound: StaticUsageBound) -> Option<UsageUpperBound> {
    match bound {
        StaticUsageBound::KnownFinite(value) => Some(UsageUpperBound::Finite(value)),
        StaticUsageBound::KnownUnbounded => Some(UsageUpperBound::Unbounded),
        StaticUsageBound::Unknown => None,
    }
}

/// Accumulates sequential or concurrently completing usage. A missing or
/// unknown coordinate is never treated as zero.
pub fn accumulate(
    a: &StaticUsageEnvelope,
    b: &StaticUsageEnvelope,
) -> Result<StaticUsageEnvelope, UsageError> {
    if a.is_globally_unknown() || b.is_globally_unknown() {
        return Ok(StaticUsageEnvelope::unknown());
    }
    let mut result = StaticUsageEnvelope::empty();
    for coordinate in coordinate_set(a, b) {
        let Some(left) = to_upper(a.get(&coordinate)) else {
            result.insert(coordinate, StaticUsageBound::Unknown)?;
            continue;
        };
        let Some(right) = to_upper(b.get(&coordinate)) else {
            result.insert(coordinate, StaticUsageBound::Unknown)?;
            continue;
        };
        let upper = left.checked_add(right)?;
        result.insert(coordinate, StaticUsageBound::from_declared(upper))?;
    }
    Ok(result)
}

/// Joins mutually exclusive branches by their maximum possible upper bound.
pub fn branch_join(
    a: &StaticUsageEnvelope,
    b: &StaticUsageEnvelope,
) -> Result<StaticUsageEnvelope, UsageError> {
    if a.is_globally_unknown() || b.is_globally_unknown() {
        return Ok(StaticUsageEnvelope::unknown());
    }
    let mut result = StaticUsageEnvelope::empty();
    for coordinate in coordinate_set(a, b) {
        let left = a.get(&coordinate);
        let right = b.get(&coordinate);
        let bound = match (left, right) {
            (StaticUsageBound::KnownFinite(a), StaticUsageBound::KnownFinite(b)) => {
                StaticUsageBound::KnownFinite(a.max(b))
            }
            (StaticUsageBound::KnownUnbounded, _) | (_, StaticUsageBound::KnownUnbounded) => {
                StaticUsageBound::KnownUnbounded
            }
            _ => StaticUsageBound::Unknown,
        };
        result.insert(coordinate, bound)?;
    }
    Ok(result)
}

/// Repeats a statically bounded envelope. `repeat(0, ...)` writes explicit
/// zero states for coordinates present in the input; it does not invent
/// zero states for absent meters.
pub fn repeat(
    count: u64,
    envelope: &StaticUsageEnvelope,
) -> Result<StaticUsageEnvelope, UsageError> {
    let mut result = StaticUsageEnvelope::empty();
    for (coordinate, bound) in envelope.iter() {
        let output = if count == 0 {
            StaticUsageBound::KnownFinite(0)
        } else {
            match bound {
                StaticUsageBound::KnownFinite(value) => StaticUsageBound::from_declared(
                    UsageUpperBound::Finite(*value).checked_mul(count)?,
                ),
                StaticUsageBound::KnownUnbounded => StaticUsageBound::KnownUnbounded,
                StaticUsageBound::Unknown => StaticUsageBound::Unknown,
            }
        };
        result.insert(coordinate.clone(), output)?;
    }
    Ok(result)
}

/// Proves an observed static envelope stays under the declared signature.
/// Every coordinate on either side must be explicitly represented; omission
/// is not zero and produces `Unproven`.
pub fn within_ceiling(
    observed: &StaticUsageEnvelope,
    declared: &StaticUsageEnvelope,
) -> UsageProof {
    if observed.is_globally_unknown() || declared.is_globally_unknown() {
        return UsageProof::Unproven(UsageUnprovenReason::UnknownBound);
    }
    let coordinates = coordinate_set(observed, declared);
    if coordinates.is_empty() {
        return UsageProof::Unproven(UsageUnprovenReason::MissingMeter);
    }
    for coordinate in coordinates {
        let observed = observed.get(&coordinate);
        let ceiling = declared.get(&coordinate);
        let (Some(observed), Some(ceiling)) = (to_upper(observed), to_upper(ceiling)) else {
            return UsageProof::Unproven(UsageUnprovenReason::MissingMeter);
        };
        if !ceiling.covers(observed) {
            return UsageProof::Violation;
        }
    }
    UsageProof::Proven
}
