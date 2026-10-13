//! Explicit read-only Usage meter and unit universe.

use crate::error::{UsageError, kind};
use crate::model::UsageMeterRef;
use std::collections::BTreeMap;
use unicode_normalization::UnicodeNormalization;

/// A nominal unit identity. Units are deliberately outside the Usage wire
/// declaration and are supplied by an explicit static universe.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UsageUnitRef {
    pub catalog: String,
    pub name: String,
}

impl UsageUnitRef {
    /// Builds a normalized unit identity.
    pub fn new(catalog: impl Into<String>, name: impl Into<String>) -> Result<Self, UsageError> {
        let unit = Self {
            catalog: catalog.into(),
            name: name.into().nfc().collect(),
        };
        check_identity(&unit.catalog, &unit.name, "unit")?;
        Ok(unit)
    }

    /// The nominal unit identity key.
    #[must_use]
    pub fn identity_key(&self) -> (&str, &str) {
        (&self.catalog, &self.name)
    }
}

/// The only aggregation semantics supported in this phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UsageAggregation {
    Additive,
    Unsupported,
}

/// The static descriptor of one recognized meter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageMeterDescriptor {
    pub meter: UsageMeterRef,
    pub unit: UsageUnitRef,
    pub aggregation: UsageAggregation,
}

/// Explicit in-memory meter vocabulary. It performs no registry lookup,
/// discovery, network access, pricing lookup or runtime measurement.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UsageMeterUniverse {
    descriptors: BTreeMap<UsageMeterRef, UsageMeterDescriptor>,
}

impl UsageMeterUniverse {
    /// Creates an empty universe.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers one descriptor without allowing replacement.
    pub fn recognize(
        &mut self,
        meter: &UsageMeterRef,
        unit: &UsageUnitRef,
        aggregation: UsageAggregation,
    ) -> Result<(), UsageError> {
        let meter = normalized_meter(meter)?;
        let unit = UsageUnitRef::new(unit.catalog.clone(), unit.name.clone())?;
        if self.descriptors.contains_key(&meter) {
            return Err(UsageError::new(
                kind::INVALID_USAGE_METER,
                format!("meter {}/{} is recognized twice", meter.catalog, meter.name),
            ));
        }
        self.descriptors.insert(
            meter.clone(),
            UsageMeterDescriptor {
                meter,
                unit,
                aggregation,
            },
        );
        Ok(())
    }

    /// Convenience registration for the only supported aggregation.
    pub fn recognize_additive(
        &mut self,
        meter: &UsageMeterRef,
        unit: &UsageUnitRef,
    ) -> Result<(), UsageError> {
        self.recognize(meter, unit, UsageAggregation::Additive)
    }

    /// Resolves one meter identity exactly.
    pub fn resolve(&self, meter: &UsageMeterRef) -> Result<&UsageMeterDescriptor, UsageError> {
        let meter = normalized_meter(meter)?;
        let descriptor = self.descriptors.get(&meter).ok_or_else(|| {
            UsageError::new(
                kind::UNKNOWN_USAGE_METER,
                format!("meter {}/{} is not registered", meter.catalog, meter.name),
            )
        })?;
        if descriptor.aggregation != UsageAggregation::Additive {
            return Err(UsageError::new(
                kind::UNSUPPORTED_USAGE_AGGREGATION,
                format!(
                    "meter {}/{} does not have supported additive aggregation",
                    meter.catalog, meter.name
                ),
            ));
        }
        Ok(descriptor)
    }

    /// Returns a descriptor if the identity is present and syntactically valid.
    #[must_use]
    pub fn get(&self, meter: &UsageMeterRef) -> Option<&UsageMeterDescriptor> {
        normalized_meter(meter)
            .ok()
            .and_then(|normalized| self.descriptors.get(&normalized))
    }

    /// Number of recognized meters.
    #[must_use]
    pub fn len(&self) -> usize {
        self.descriptors.len()
    }

    /// Whether the universe is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.descriptors.is_empty()
    }

    /// Every exact meter descriptor in canonical meter identity order.
    pub fn descriptors(&self) -> impl Iterator<Item = &UsageMeterDescriptor> {
        self.descriptors.values()
    }
}

/// Validates and NFC-normalizes a meter identity.
pub fn normalized_meter(meter: &UsageMeterRef) -> Result<UsageMeterRef, UsageError> {
    check_identity(&meter.catalog, &meter.name, "meter")?;
    Ok(UsageMeterRef {
        catalog: meter.catalog.clone(),
        name: meter.name.nfc().collect(),
    })
}

fn check_identity(catalog: &str, name: &str, label: &str) -> Result<(), UsageError> {
    if af_idl::SpecificationId::parse(catalog).is_err() {
        return Err(UsageError::new(
            kind::INVALID_USAGE_METER,
            format!("{label} catalog {catalog:?} is not a SpecificationId"),
        ));
    }
    if name.is_empty() || name.chars().any(char::is_whitespace) {
        return Err(UsageError::new(
            kind::INVALID_USAGE_METER,
            format!("{label} name {name:?} is empty or contains whitespace"),
        ));
    }
    Ok(())
}
