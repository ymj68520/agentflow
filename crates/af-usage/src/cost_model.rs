//! Cost Projection wire model and non-wire static cost results.

use crate::currency::{CurrencyRef, MeasurementUnitRef};
use serde::{Deserialize, Serialize};

/// Non-negative atomic money amount.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MoneyAmount {
    pub currency: CurrencyRef,
    pub atomic_units: u64,
}

/// Exact non-negative rational price per one usage-unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PriceRate {
    pub numerator: u64,
    pub denominator: u64,
}

/// Pricing model is closed in the wire shape, but only FixedLinear is
/// supported by Phase 4A.5B.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PricingModel {
    FixedLinear,
    Tiered,
    Progressive,
}

/// Rounding model. Only per-meter-unit ceiling is safe without a billing
/// grouping contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RoundingRule {
    PerMeterUnitCeil,
    AggregateOnceCeil,
}

/// One fixed price entry. `phase = None` applies independently to both
/// phases; a phase-specific entry takes precedence for that phase.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PriceEntry {
    pub meter: crate::model::UsageMeterRef,
    pub phase: Option<crate::model::UsagePhase>,
    pub unit: MeasurementUnitRef,
    pub rate: PriceRate,
    pub rounding: RoundingRule,
}

/// Complete by-value immutable price snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PriceTableSnapshot {
    pub currency: CurrencyRef,
    pub atomic_unit: MeasurementUnitRef,
    pub model: PricingModel,
    pub entries: Vec<PriceEntry>,
}

/// Static forward/recovery budget ceiling.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StaticCostCeiling {
    pub currency: CurrencyRef,
    pub forward: MoneyAmount,
    pub recovery: MoneyAmount,
}

/// A derived static cost state, intentionally not an AFIDL type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaticCostBound {
    KnownFinite(MoneyAmount),
    KnownUnbounded { currency: CurrencyRef },
    Unproven(crate::cost::CostUnprovenReason),
}

/// Derived cost projection with forward/recovery accounting retained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticCostProjection {
    pub currency: CurrencyRef,
    pub forward: StaticCostBound,
    pub recovery: StaticCostBound,
    pub total: StaticCostBound,
}
