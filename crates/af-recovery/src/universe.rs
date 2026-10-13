//! Explicit recovery-operation capability universe.

use crate::error::{RecoveryError, kind};
use af_effect::EffectCatalogUniverse;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecoveryOperationUniverse {
    pub effects: EffectCatalogUniverse,
    reconciliation_operations: BTreeSet<(String, String)>,
}

impl RecoveryOperationUniverse {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn recognize_reconciliation_operation(
        &mut self,
        catalog: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<(), RecoveryError> {
        let catalog = catalog.into();
        let name = name.into();
        if af_idl::SpecificationId::parse(&catalog).is_err()
            || name.is_empty()
            || name.chars().any(char::is_whitespace)
        {
            return Err(RecoveryError::new(
                kind::UNKNOWN_RECOVERY_OPERATION,
                format!("invalid reconciliation operation {catalog}/{name}"),
            ));
        }
        self.reconciliation_operations.insert((catalog, name));
        Ok(())
    }

    #[must_use]
    pub fn resolves_reconciliation(&self, catalog: &str, name: &str) -> bool {
        self.reconciliation_operations
            .contains(&(catalog.to_string(), name.to_string()))
    }

    /// Every explicitly recognized reconciliation operation in exact
    /// identity order.
    pub fn reconciliation_operations(&self) -> impl Iterator<Item = (&str, &str)> {
        self.reconciliation_operations
            .iter()
            .map(|(catalog, name)| (catalog.as_str(), name.as_str()))
    }
}
