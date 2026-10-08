//! Canonical Cost Snapshot and Static Cost Ceiling validation/encoding.

use crate::cost_error::{CostError, cost_kind};
use crate::cost_model::{
    MoneyAmount, PriceEntry, PriceTableSnapshot, RoundingRule, StaticCostCeiling,
};
use crate::currency::{CurrencyRef, CurrencyUniverse, MeasurementUnitRef, validate_currency};
use crate::model::UsagePhase;
use crate::universe::UsageMeterUniverse;
use af_core::Value;
use std::collections::BTreeSet;

/// Canonicalizes and validates one immutable price snapshot.
pub fn canonicalize_price_table_snapshot(
    snapshot: &PriceTableSnapshot,
    currencies: &CurrencyUniverse,
    meters: &UsageMeterUniverse,
) -> Result<PriceTableSnapshot, CostError> {
    let currency = validate_currency(&snapshot.currency)?;
    let descriptor = currencies.resolve(&currency)?;
    let atomic_unit = MeasurementUnitRef::new(
        snapshot.atomic_unit.catalog.clone(),
        snapshot.atomic_unit.name.clone(),
    )?;
    if atomic_unit != descriptor.atomic_unit {
        return Err(CostError::new(
            cost_kind::CURRENCY_UNIT_MISMATCH,
            "price snapshot atomic unit does not match the currency universe",
        ));
    }
    if snapshot.model != crate::cost_model::PricingModel::FixedLinear {
        return Err(CostError::new(
            cost_kind::UNSUPPORTED_PRICING_MODEL,
            "only fixed_linear pricing is supported",
        ));
    }
    let mut seen = BTreeSet::new();
    let mut entries = Vec::with_capacity(snapshot.entries.len());
    for entry in &snapshot.entries {
        let meter = crate::universe::normalized_meter(&entry.meter)
            .map_err(|error| CostError::new(cost_kind::UNKNOWN_PRICE_METER, error.detail()))?;
        let meter_descriptor = meters
            .resolve(&meter)
            .map_err(|error| CostError::new(cost_kind::UNKNOWN_PRICE_METER, error.detail()))?;
        let unit = MeasurementUnitRef::new(entry.unit.catalog.clone(), entry.unit.name.clone())?;
        if unit.catalog != meter_descriptor.unit.catalog || unit.name != meter_descriptor.unit.name
        {
            return Err(CostError::new(
                cost_kind::PRICE_METER_UNIT_MISMATCH,
                format!(
                    "price unit does not match meter {}/{}",
                    meter.catalog, meter.name
                ),
            ));
        }
        if entry.rate.denominator == 0 {
            return Err(CostError::new(
                cost_kind::INVALID_PRICE_RATE,
                "price denominator must be greater than zero",
            ));
        }
        if entry.rounding != RoundingRule::PerMeterUnitCeil {
            return Err(CostError::new(
                cost_kind::UNSUPPORTED_ROUNDING_RULE,
                "only per_meter_unit_ceil is supported",
            ));
        }
        let key = (entry.phase, meter.catalog.clone(), meter.name.clone());
        if !seen.insert(key) {
            return Err(CostError::new(
                cost_kind::DUPLICATE_PRICE_ENTRY,
                "the same price meter/phase applicability is declared twice",
            ));
        }
        entries.push(PriceEntry {
            meter,
            phase: entry.phase,
            unit,
            rate: entry.rate,
            rounding: entry.rounding,
        });
    }
    entries.sort_by(|a, b| {
        (phase_sort_key(a.phase), &a.meter.catalog, &a.meter.name).cmp(&(
            phase_sort_key(b.phase),
            &b.meter.catalog,
            &b.meter.name,
        ))
    });
    Ok(PriceTableSnapshot {
        currency,
        atomic_unit,
        model: snapshot.model,
        entries,
    })
}

/// Canonicalizes a static forward/recovery cost ceiling.
pub fn canonicalize_static_cost_ceiling(
    ceiling: &StaticCostCeiling,
    currencies: &CurrencyUniverse,
) -> Result<StaticCostCeiling, CostError> {
    let currency = validate_currency(&ceiling.currency)?;
    currencies.resolve(&currency)?;
    validate_money(&ceiling.forward, &currency)?;
    validate_money(&ceiling.recovery, &currency)?;
    Ok(StaticCostCeiling {
        currency: currency.clone(),
        forward: MoneyAmount {
            currency: currency.clone(),
            atomic_units: ceiling.forward.atomic_units,
        },
        recovery: MoneyAmount {
            currency,
            atomic_units: ceiling.recovery.atomic_units,
        },
    })
}

fn validate_money(money: &MoneyAmount, currency: &CurrencyRef) -> Result<(), CostError> {
    let money_currency = validate_currency(&money.currency)?;
    if &money_currency != currency {
        return Err(CostError::new(
            cost_kind::CURRENCY_UNIT_MISMATCH,
            "money amount currency does not match its ceiling currency",
        ));
    }
    Ok(())
}

fn phase_sort_key(phase: Option<UsagePhase>) -> u32 {
    match phase {
        None => 0,
        Some(UsagePhase::Forward) => 1,
        Some(UsagePhase::Recovery) => 2,
    }
}

impl PriceTableSnapshot {
    /// Canonicalizes this snapshot against explicit currency and meter universes.
    pub fn canonical(
        &self,
        currencies: &CurrencyUniverse,
        meters: &UsageMeterUniverse,
    ) -> Result<Self, CostError> {
        canonicalize_price_table_snapshot(self, currencies, meters)
    }

    /// Canonical ACE1 bytes; no price snapshot digest exists.
    pub fn canonical_bytes(
        &self,
        currencies: &CurrencyUniverse,
        meters: &UsageMeterUniverse,
        set: &af_idl::CompiledModuleSet,
    ) -> Result<Vec<u8>, CostError> {
        let shape = set
            .shape_of("agentflow.usage.cost@1/PriceTableSnapshot")
            .map_err(|error| CostError::new(cost_kind::COST_ENCODING_ERROR, error.to_string()))?;
        shape
            .validate()
            .map_err(|error| CostError::new(cost_kind::COST_ENCODING_ERROR, error.to_string()))?;
        af_core::encode(&shape, &self.canonical(currencies, meters)?.to_value()?)
            .map_err(|error| CostError::new(cost_kind::COST_ENCODING_ERROR, error.to_string()))
    }

    pub fn to_value(&self) -> Result<Value, CostError> {
        Ok(Value::Record(vec![
            ("currency".into(), currency_value(&self.currency)),
            ("atomic_unit".into(), unit_value(&self.atomic_unit)),
            ("model".into(), Value::Enum(model_str(self.model).into())),
            (
                "entries".into(),
                Value::List(self.entries.iter().map(price_entry_value).collect()),
            ),
        ]))
    }
}

impl StaticCostCeiling {
    pub fn canonical(&self, currencies: &CurrencyUniverse) -> Result<Self, CostError> {
        canonicalize_static_cost_ceiling(self, currencies)
    }

    pub fn canonical_bytes(
        &self,
        currencies: &CurrencyUniverse,
        set: &af_idl::CompiledModuleSet,
    ) -> Result<Vec<u8>, CostError> {
        let shape = set
            .shape_of("agentflow.usage.cost@1/StaticCostCeiling")
            .map_err(|error| CostError::new(cost_kind::COST_ENCODING_ERROR, error.to_string()))?;
        shape
            .validate()
            .map_err(|error| CostError::new(cost_kind::COST_ENCODING_ERROR, error.to_string()))?;
        af_core::encode(&shape, &self.canonical(currencies)?.to_value()?)
            .map_err(|error| CostError::new(cost_kind::COST_ENCODING_ERROR, error.to_string()))
    }

    pub fn to_value(&self) -> Result<Value, CostError> {
        Ok(Value::Record(vec![
            ("currency".into(), currency_value(&self.currency)),
            ("forward".into(), money_value(&self.forward)),
            ("recovery".into(), money_value(&self.recovery)),
        ]))
    }
}

fn currency_value(currency: &CurrencyRef) -> Value {
    Value::Record(vec![
        ("catalog".into(), Value::String(currency.catalog.clone())),
        ("code".into(), Value::String(currency.code.clone())),
    ])
}

fn unit_value(unit: &MeasurementUnitRef) -> Value {
    Value::Record(vec![
        ("catalog".into(), Value::String(unit.catalog.clone())),
        ("name".into(), Value::String(unit.name.clone())),
    ])
}

fn money_value(money: &MoneyAmount) -> Value {
    Value::Record(vec![
        ("currency".into(), currency_value(&money.currency)),
        ("atomic_units".into(), Value::UInt64(money.atomic_units)),
    ])
}

fn price_entry_value(entry: &PriceEntry) -> Value {
    Value::Record(vec![
        (
            "meter".into(),
            Value::Record(vec![
                ("catalog".into(), Value::String(entry.meter.catalog.clone())),
                ("name".into(), Value::String(entry.meter.name.clone())),
            ]),
        ),
        (
            "phase".into(),
            Value::Option(
                entry
                    .phase
                    .map(|phase| Box::new(Value::Enum(phase.as_str().to_string()))),
            ),
        ),
        ("unit".into(), unit_value(&entry.unit)),
        (
            "rate".into(),
            Value::Record(vec![
                ("numerator".into(), Value::UInt64(entry.rate.numerator)),
                ("denominator".into(), Value::UInt64(entry.rate.denominator)),
            ]),
        ),
        (
            "rounding".into(),
            Value::Enum(rounding_str(entry.rounding).into()),
        ),
    ])
}

fn model_str(model: crate::cost_model::PricingModel) -> &'static str {
    match model {
        crate::cost_model::PricingModel::FixedLinear => "fixed_linear",
        crate::cost_model::PricingModel::Tiered => "tiered",
        crate::cost_model::PricingModel::Progressive => "progressive",
    }
}

fn rounding_str(rounding: RoundingRule) -> &'static str {
    match rounding {
        RoundingRule::PerMeterUnitCeil => "per_meter_unit_ceil",
        RoundingRule::AggregateOnceCeil => "aggregate_once_ceil",
    }
}
