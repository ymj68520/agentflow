//! The explicit capability-catalog universe.
//!
//! The final capability catalog is not materialized yet; this phase uses
//! explicit, immutable, in-memory catalogs to resolve operation symbols.
//! A catalog is a NAME SET: presence in the catalog is exactly what makes an
//! operation resolvable, and nothing more is claimed for it. There is no
//! registry lookup, no latest-version fallback, no filesystem/network
//! discovery, and no default catalog — an unresolved operation fails closed.

use crate::error::{EffectError, kind};
use af_idl::SpecificationId;
use std::collections::{BTreeMap, BTreeSet};

/// One explicit capability catalog: an identity plus the operation names it
/// declares. Construction validates both grammars; the value is immutable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityCatalog {
    specification_id: String,
    operations: BTreeSet<String>,
}

impl CapabilityCatalog {
    /// Builds a catalog and checks its identity and operation-name grammars.
    ///
    /// # Errors
    /// [`EffectError`] with kind `invalid_effect_operation` when the
    /// specification id or any operation name fails its grammar.
    pub fn new(
        specification_id: impl Into<String>,
        operations: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, EffectError> {
        let specification_id = specification_id.into();
        Self::check_catalog_id(&specification_id)?;
        let mut set = BTreeSet::new();
        for name in operations {
            let name = name.into();
            Self::check_operation_name(&name)?;
            if !set.insert(name.clone()) {
                return Err(EffectError::new(
                    kind::INVALID_EFFECT_OPERATION,
                    format!("catalog {specification_id} declares {name:?} twice"),
                ));
            }
        }
        Ok(CapabilityCatalog {
            specification_id,
            operations: set,
        })
    }

    /// The catalog's specification identity.
    #[must_use]
    pub fn specification_id(&self) -> &str {
        &self.specification_id
    }

    /// The declared operation names, in canonical (lexicographic) order.
    pub fn operations(&self) -> impl Iterator<Item = &str> {
        self.operations.iter().map(String::as_str)
    }

    /// Whether the catalog declares `name`.
    #[must_use]
    pub fn declares(&self, name: &str) -> bool {
        self.operations.contains(name)
    }

    pub(crate) fn check_catalog_id(identity: &str) -> Result<(), EffectError> {
        SpecificationId::parse(identity).map_err(|error| {
            EffectError::new(
                kind::INVALID_EFFECT_OPERATION,
                format!("catalog identity {identity:?} is not a SpecificationId: {error}"),
            )
        })?;
        Ok(())
    }

    pub(crate) fn check_operation_name(name: &str) -> Result<(), EffectError> {
        let fail = |detail: String| {
            Err(EffectError::new(
                kind::INVALID_EFFECT_OPERATION,
                format!("operation name {name:?} is invalid: {detail}"),
            ))
        };
        if name.is_empty() {
            return fail("empty".into());
        }
        if name.len() > 256 {
            return fail("longer than 256 characters".into());
        }
        if !is_nfc(name) {
            return fail("not NFC-normalized".into());
        }
        for segment in name.split('.') {
            if segment.is_empty() {
                return fail("empty dotted segment".into());
            }
            if !segment
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            {
                return fail(
                    "segments must be ASCII [a-z0-9-] (dotted, no other separators)".into(),
                );
            }
        }
        Ok(())
    }
}

/// The explicit universe of catalogs an effect signature is validated
/// against. Insertion is idempotent for identical catalogs and a collision
/// error for a different catalog under the same identity.
#[derive(Debug, Clone, Default)]
pub struct EffectCatalogUniverse {
    catalogs: BTreeMap<String, CapabilityCatalog>,
}

impl EffectCatalogUniverse {
    /// An empty universe. There is no implicit content: an empty universe
    /// resolves nothing.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts one catalog. Re-inserting an identical catalog is idempotent;
    /// a different catalog under the same identity is a collision.
    ///
    /// # Errors
    /// [`EffectError`] with kind `effect_catalog_collision`.
    pub fn insert(&mut self, catalog: &CapabilityCatalog) -> Result<(), EffectError> {
        match self.catalogs.get(catalog.specification_id()) {
            Some(existing) if existing == catalog => Ok(()),
            Some(_) => Err(EffectError::new(
                kind::EFFECT_CATALOG_COLLISION,
                format!(
                    "catalog {} is already registered with different operations",
                    catalog.specification_id()
                ),
            )),
            None => {
                self.catalogs
                    .insert(catalog.specification_id().to_string(), catalog.clone());
                Ok(())
            }
        }
    }

    /// Whether `catalog` is registered and declares `name`.
    #[must_use]
    pub fn resolves(&self, catalog: &str, name: &str) -> bool {
        self.catalogs
            .get(catalog)
            .is_some_and(|entry| entry.declares(name))
    }

    /// The registered catalog identities, in canonical order.
    pub fn catalog_ids(&self) -> impl Iterator<Item = &str> {
        self.catalogs.keys().map(String::as_str)
    }

    /// Number of registered catalogs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.catalogs.len()
    }

    /// Whether the universe is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.catalogs.is_empty()
    }
}

pub(crate) fn is_nfc(text: &str) -> bool {
    use unicode_normalization::UnicodeNormalization;
    text.nfc().eq(text.chars())
}
