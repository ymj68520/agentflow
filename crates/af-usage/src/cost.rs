//! Exact static Cost Projection from validated Usage and fixed prices.

use crate::algebra::UsageProof;
use crate::cost_canonical::canonicalize_price_table_snapshot;
use crate::cost_error::{CostError, cost_kind};
use crate::cost_model::{
    MoneyAmount, PriceEntry, PriceTableSnapshot, StaticCostBound, StaticCostProjection,
};
use crate::currency::{CurrencyRef, CurrencyUniverse};
use crate::model::{StaticUsageBound, StaticUsageEnvelope, UsagePhase};
use crate::universe::UsageMeterUniverse;

/// Why a static cost projection could not be proven.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CostUnprovenReason {
    MissingPriceCoverage,
    UnknownUsage,
    CurrencyMismatch,
    DifferentPriceSnapshot,
}

impl CostUnprovenReason {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MissingPriceCoverage => "missing_price_coverage",
            Self::UnknownUsage => "unknown_usage",
            Self::CurrencyMismatch => "currency_mismatch",
            Self::DifferentPriceSnapshot => "different_price_snapshot",
        }
    }
}

/// Cost proof result. It deliberately keeps `Unproven` distinct from
/// `Violation` and `Proven`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CostProof {
    Proven,
    Violation,
    Unproven(CostUnprovenReason),
}

impl CostProof {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Proven => "proven",
            Self::Violation => "violation",
            Self::Unproven(_) => "unproven",
        }
    }

    pub fn require_proven(self) -> Result<(), CostError> {
        match self {
            Self::Proven => Ok(()),
            Self::Violation => Err(CostError::new(
                cost_kind::STATIC_BUDGET_EXCEEDED,
                "static cost ceiling is exceeded",
            )),
            Self::Unproven(reason) => Err(CostError::new(
                cost_kind::STATIC_BUDGET_UNPROVEN,
                format!("static cost proof is unproven: {}", reason.as_str()),
            )),
        }
    }
}

/// Projects an already-derived static Usage envelope through a validated
/// fixed-linear price snapshot.
pub fn project_cost(
    usage: &StaticUsageEnvelope,
    snapshot: &PriceTableSnapshot,
    currencies: &CurrencyUniverse,
    meters: &UsageMeterUniverse,
) -> Result<StaticCostProjection, CostError> {
    let snapshot = canonicalize_price_table_snapshot(snapshot, currencies, meters)?;
    let currency = snapshot.currency.clone();
    if usage.is_globally_unknown() {
        return Ok(unproven_projection(
            currency,
            CostUnprovenReason::UnknownUsage,
        ));
    }
    if usage.is_empty() {
        return Ok(unproven_projection(
            currency,
            CostUnprovenReason::MissingPriceCoverage,
        ));
    }
    let mut forward = StaticCostBound::KnownFinite(MoneyAmount {
        currency: currency.clone(),
        atomic_units: 0,
    });
    let mut recovery = StaticCostBound::KnownFinite(MoneyAmount {
        currency: currency.clone(),
        atomic_units: 0,
    });
    for (coordinate, usage_bound) in usage.iter() {
        let Some(entry) = find_price_entry(&snapshot.entries, &coordinate.meter, coordinate.phase)
        else {
            return Ok(unproven_projection(
                currency,
                CostUnprovenReason::MissingPriceCoverage,
            ));
        };
        let projected = price_one(usage_bound, entry, &currency)?;
        match coordinate.phase {
            UsagePhase::Forward => forward = add_cost_bound(forward, projected)?,
            UsagePhase::Recovery => recovery = add_cost_bound(recovery, projected)?,
        }
    }
    let total = add_cost_bound(forward.clone(), recovery.clone())?;
    Ok(StaticCostProjection {
        currency,
        forward,
        recovery,
        total,
    })
}

/// Checks forward and recovery independently against a static cost ceiling.
pub fn check_static_budget(
    projection: &StaticCostProjection,
    ceiling: &crate::cost_model::StaticCostCeiling,
    currencies: &CurrencyUniverse,
) -> Result<CostProof, CostError> {
    let ceiling = crate::cost_canonical::canonicalize_static_cost_ceiling(ceiling, currencies)?;
    if projection.currency != ceiling.currency {
        return Ok(CostProof::Unproven(CostUnprovenReason::CurrencyMismatch));
    }
    let forward = compare_bound(&projection.forward, ceiling.forward.atomic_units);
    let recovery = compare_bound(&projection.recovery, ceiling.recovery.atomic_units);
    Ok(combine_proofs(forward, recovery))
}

/// Cost non-increase is only comparable when currency and complete price
/// snapshot values are identical. A lower numeric cost under a different
/// price snapshot is not evidence of a cheaper implementation.
pub fn cost_not_increased(
    new_projection: &StaticCostProjection,
    new_snapshot: &PriceTableSnapshot,
    old_projection: &StaticCostProjection,
    old_snapshot: &PriceTableSnapshot,
) -> CostProof {
    if new_projection.currency != old_projection.currency || new_snapshot != old_snapshot {
        return CostProof::Unproven(CostUnprovenReason::DifferentPriceSnapshot);
    }
    let forward = compare_projection_bound(&new_projection.forward, &old_projection.forward);
    let recovery = compare_projection_bound(&new_projection.recovery, &old_projection.recovery);
    combine_proofs(forward, recovery)
}

fn find_price_entry<'a>(
    entries: &'a [PriceEntry],
    meter: &crate::model::UsageMeterRef,
    phase: UsagePhase,
) -> Option<&'a PriceEntry> {
    entries
        .iter()
        .find(|entry| entry.meter == *meter && entry.phase == Some(phase))
        .or_else(|| {
            entries
                .iter()
                .find(|entry| entry.meter == *meter && entry.phase.is_none())
        })
}

fn price_one(
    usage: &StaticUsageBound,
    entry: &PriceEntry,
    currency: &CurrencyRef,
) -> Result<StaticCostBound, CostError> {
    match usage {
        StaticUsageBound::Unknown => {
            Ok(StaticCostBound::Unproven(CostUnprovenReason::UnknownUsage))
        }
        StaticUsageBound::KnownUnbounded if entry.rate.numerator == 0 => {
            Ok(StaticCostBound::KnownFinite(MoneyAmount {
                currency: currency.clone(),
                atomic_units: 0,
            }))
        }
        StaticUsageBound::KnownUnbounded => Ok(StaticCostBound::KnownUnbounded {
            currency: currency.clone(),
        }),
        StaticUsageBound::KnownFinite(usage) => {
            let unit_upper = ceil_div(entry.rate.numerator, entry.rate.denominator)?;
            let amount = (u128::from(*usage))
                .checked_mul(u128::from(unit_upper))
                .ok_or_else(|| {
                    CostError::new(
                        cost_kind::COST_ARITHMETIC_OVERFLOW,
                        "usage multiplied by per-unit price overflows u128",
                    )
                })?;
            let amount = u64::try_from(amount).map_err(|_| {
                CostError::new(
                    cost_kind::COST_ARITHMETIC_OVERFLOW,
                    "projected atomic cost does not fit u64",
                )
            })?;
            Ok(StaticCostBound::KnownFinite(MoneyAmount {
                currency: currency.clone(),
                atomic_units: amount,
            }))
        }
    }
}

fn ceil_div(numerator: u64, denominator: u64) -> Result<u64, CostError> {
    if denominator == 0 {
        return Err(CostError::new(
            cost_kind::INVALID_PRICE_RATE,
            "price denominator must be greater than zero",
        ));
    }
    let adjusted = u128::from(numerator)
        .checked_add(u128::from(denominator) - 1)
        .ok_or_else(|| {
            CostError::new(
                cost_kind::COST_ARITHMETIC_OVERFLOW,
                "ceil division overflow",
            )
        })?;
    u64::try_from(adjusted / u128::from(denominator)).map_err(|_| {
        CostError::new(
            cost_kind::COST_ARITHMETIC_OVERFLOW,
            "ceil division result does not fit u64",
        )
    })
}

fn add_cost_bound(
    left: StaticCostBound,
    right: StaticCostBound,
) -> Result<StaticCostBound, CostError> {
    match (left, right) {
        (StaticCostBound::Unproven(reason), _) | (_, StaticCostBound::Unproven(reason)) => {
            Ok(StaticCostBound::Unproven(reason))
        }
        (StaticCostBound::KnownUnbounded { currency }, _)
        | (_, StaticCostBound::KnownUnbounded { currency }) => {
            Ok(StaticCostBound::KnownUnbounded { currency })
        }
        (StaticCostBound::KnownFinite(left), StaticCostBound::KnownFinite(right)) => {
            let amount = left
                .atomic_units
                .checked_add(right.atomic_units)
                .ok_or_else(|| {
                    CostError::new(
                        cost_kind::COST_ARITHMETIC_OVERFLOW,
                        "atomic cost addition overflows u64",
                    )
                })?;
            Ok(StaticCostBound::KnownFinite(MoneyAmount {
                currency: left.currency,
                atomic_units: amount,
            }))
        }
    }
}

fn unproven_projection(currency: CurrencyRef, reason: CostUnprovenReason) -> StaticCostProjection {
    let forward = StaticCostBound::Unproven(reason);
    let recovery = StaticCostBound::Unproven(reason);
    StaticCostProjection {
        currency,
        forward: forward.clone(),
        recovery: recovery.clone(),
        total: forward,
    }
}

fn compare_bound(bound: &StaticCostBound, ceiling: u64) -> CostProof {
    match bound {
        StaticCostBound::KnownFinite(amount) if amount.atomic_units <= ceiling => CostProof::Proven,
        StaticCostBound::KnownFinite(_) | StaticCostBound::KnownUnbounded { .. } => {
            CostProof::Violation
        }
        StaticCostBound::Unproven(reason) => CostProof::Unproven(*reason),
    }
}

fn compare_projection_bound(new: &StaticCostBound, old: &StaticCostBound) -> CostProof {
    match (new, old) {
        (StaticCostBound::Unproven(reason), _) | (_, StaticCostBound::Unproven(reason)) => {
            CostProof::Unproven(*reason)
        }
        (StaticCostBound::KnownUnbounded { .. }, StaticCostBound::KnownUnbounded { .. })
        | (StaticCostBound::KnownFinite(_), StaticCostBound::KnownUnbounded { .. }) => {
            CostProof::Proven
        }
        (StaticCostBound::KnownUnbounded { .. }, _) => CostProof::Violation,
        (StaticCostBound::KnownFinite(new), StaticCostBound::KnownFinite(old)) => {
            if new.atomic_units <= old.atomic_units {
                CostProof::Proven
            } else {
                CostProof::Violation
            }
        }
    }
}

fn combine_proofs(left: CostProof, right: CostProof) -> CostProof {
    match (left, right) {
        (CostProof::Violation, _) | (_, CostProof::Violation) => CostProof::Violation,
        (CostProof::Unproven(reason), _) | (_, CostProof::Unproven(reason)) => {
            CostProof::Unproven(reason)
        }
        (CostProof::Proven, CostProof::Proven) => CostProof::Proven,
    }
}

/// Keep the Usage algebra proof type visible to Cost callers without creating
/// a second Usage semantics.
#[allow(dead_code)]
fn _usage_proof_marker(_: UsageProof) {}
