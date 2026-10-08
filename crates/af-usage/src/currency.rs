//! Explicit currency identity and atomic-unit universe.

use crate::cost_error::{CostError, cost_kind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use unicode_normalization::UnicodeNormalization;

/// Exact currency identity. No code spelling implies precision or exchange
/// rate; both are supplied by the explicit universe.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CurrencyRef {
    pub catalog: String,
    pub code: String,
}

impl CurrencyRef {
    pub fn new(catalog: impl Into<String>, code: impl Into<String>) -> Result<Self, CostError> {
        let currency = Self {
            catalog: catalog.into(),
            code: code.into().nfc().collect(),
        };
        validate_currency(&currency)
    }
}

/// Identity of the atomic monetary unit used by a currency descriptor and a
/// price snapshot.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MeasurementUnitRef {
    pub catalog: String,
    pub name: String,
}

impl MeasurementUnitRef {
    pub fn new(catalog: impl Into<String>, name: impl Into<String>) -> Result<Self, CostError> {
        let unit = Self {
            catalog: catalog.into(),
            name: name.into().nfc().collect(),
        };
        validate_identity(&unit.catalog, &unit.name)
            .map(|()| unit)
            .map_err(|error| CostError::new(cost_kind::INVALID_CURRENCY, error))
    }
}

/// Static currency descriptor. `display_scale` is presentation metadata and
/// never participates in atomic arithmetic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrencyDescriptor {
    pub currency: CurrencyRef,
    pub atomic_unit: MeasurementUnitRef,
    pub display_scale: u8,
}

/// Explicit read-only currency universe.
#[derive(Debug, Clone, Default)]
pub struct CurrencyUniverse {
    descriptors: BTreeMap<CurrencyRef, CurrencyDescriptor>,
}

impl CurrencyUniverse {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn recognize(
        &mut self,
        currency: &CurrencyRef,
        atomic_unit: &MeasurementUnitRef,
        display_scale: u8,
    ) -> Result<(), CostError> {
        let currency = CurrencyRef::new(currency.catalog.clone(), currency.code.clone())?;
        let atomic_unit =
            MeasurementUnitRef::new(atomic_unit.catalog.clone(), atomic_unit.name.clone())?;
        if self.descriptors.contains_key(&currency) {
            return Err(CostError::new(
                cost_kind::INVALID_CURRENCY,
                format!(
                    "currency {}/{} is recognized twice",
                    currency.catalog, currency.code
                ),
            ));
        }
        self.descriptors.insert(
            currency.clone(),
            CurrencyDescriptor {
                currency,
                atomic_unit,
                display_scale,
            },
        );
        Ok(())
    }

    pub fn resolve(&self, currency: &CurrencyRef) -> Result<&CurrencyDescriptor, CostError> {
        let currency = CurrencyRef::new(currency.catalog.clone(), currency.code.clone())?;
        self.descriptors.get(&currency).ok_or_else(|| {
            CostError::new(
                cost_kind::UNKNOWN_CURRENCY,
                format!(
                    "currency {}/{} is not registered",
                    currency.catalog, currency.code
                ),
            )
        })
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.descriptors.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.descriptors.is_empty()
    }
}

pub fn validate_currency(currency: &CurrencyRef) -> Result<CurrencyRef, CostError> {
    validate_identity(&currency.catalog, &currency.code)
        .map(|()| CurrencyRef {
            catalog: currency.catalog.clone(),
            code: currency.code.nfc().collect(),
        })
        .map_err(|error| CostError::new(cost_kind::INVALID_CURRENCY, error))
}

fn validate_identity(catalog: &str, name: &str) -> Result<(), String> {
    if af_idl::SpecificationId::parse(catalog).is_err() {
        return Err(format!("catalog {catalog:?} is not a SpecificationId"));
    }
    if name.is_empty() || name.chars().any(char::is_whitespace) {
        return Err(format!(
            "identity name {name:?} is empty or contains whitespace"
        ));
    }
    Ok(())
}
