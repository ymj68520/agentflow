//! The explicit, read-only information-label and effect-catalog universes.
//!
//! Recognition is an explicit registry-level fact: label catalogs and
//! effect catalogs are explicit, immutable, in-memory inputs. There is no
//! registry lookup, no latest-version fallback, no case-insensitive
//! matching, no auto-registration and no invented hierarchy between
//! confidentiality labels.

use crate::error::{FlowError, kind};
use crate::model::{ConfidentialityLabelRef, TaintRef};
use std::collections::BTreeSet;
use unicode_normalization::UnicodeNormalization;

/// NFC normalization for label text.
#[must_use]
pub fn nfc(text: &str) -> String {
    text.nfc().collect()
}

/// Syntax check of one nominal label reference (catalog id grammar,
/// non-empty NFC name, no whitespace). Used for both confidentiality
/// restrictions and taints.
///
/// # Errors
/// [`FlowError`] with kind `invalid_information_label`.
pub fn check_label(catalog: &str, name: &str) -> Result<(), FlowError> {
    if af_idl::SpecificationId::parse(catalog).is_err() {
        return Err(FlowError::new(
            kind::INVALID_INFORMATION_LABEL,
            format!("label catalog {catalog:?} is not a SpecificationId"),
        ));
    }
    let normalized = nfc(name);
    if normalized.is_empty() || normalized.chars().any(char::is_whitespace) {
        return Err(FlowError::new(
            kind::INVALID_INFORMATION_LABEL,
            format!("label name {name:?} is not a valid label path"),
        ));
    }
    Ok(())
}

/// The explicit read-only label universe: two closed vocabularies
/// (confidentiality restrictions and taints), each a set of
/// `(catalog, name)` identities. Membership is exact — case-sensitive,
/// NFC-normalized; unknown labels are rejected, never created.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InformationLabelUniverse {
    confidentiality: BTreeSet<(String, String)>,
    taints: BTreeSet<(String, String)>,
}

impl InformationLabelUniverse {
    /// An empty universe.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers one confidentiality label identity.
    pub fn recognize_confidentiality(&mut self, catalog: &str, name: &str) {
        self.confidentiality
            .insert((catalog.to_string(), nfc(name)));
    }

    /// Registers one taint identity.
    pub fn recognize_taint(&mut self, catalog: &str, name: &str) {
        self.taints.insert((catalog.to_string(), nfc(name)));
    }

    /// Resolves a confidentiality restriction exactly: syntax first, then
    /// membership.
    ///
    /// # Errors
    /// [`FlowError`] with kinds `invalid_information_label` and
    /// `unknown_information_label`.
    pub fn resolve_confidentiality(
        &self,
        label: &ConfidentialityLabelRef,
    ) -> Result<(), FlowError> {
        check_label(&label.catalog, &label.name)?;
        let key = (label.catalog.clone(), nfc(&label.name));
        if !self.confidentiality.contains(&key) {
            return Err(FlowError::new(
                kind::UNKNOWN_INFORMATION_LABEL,
                format!(
                    "confidentiality label {}/{} is not registered in the \
                     supplied universe (case-sensitive, no auto-registration)",
                    label.catalog, label.name
                ),
            ));
        }
        Ok(())
    }

    /// Resolves a taint exactly.
    ///
    /// # Errors
    /// [`FlowError`] with kinds `invalid_information_label` and
    /// `unknown_information_label`.
    pub fn resolve_taint(&self, taint: &TaintRef) -> Result<(), FlowError> {
        check_label(&taint.catalog, &taint.name)?;
        let key = (taint.catalog.clone(), nfc(&taint.name));
        if !self.taints.contains(&key) {
            return Err(FlowError::new(
                kind::UNKNOWN_INFORMATION_LABEL,
                format!(
                    "taint {}/{} is not registered in the supplied universe \
                     (case-sensitive, no auto-registration)",
                    taint.catalog, taint.name
                ),
            ));
        }
        Ok(())
    }

    /// Number of registered identities (both vocabularies).
    #[must_use]
    pub fn len(&self) -> usize {
        self.confidentiality.len() + self.taints.len()
    }

    /// `true` when no label is registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.confidentiality.is_empty() && self.taints.is_empty()
    }

    /// Every recognized confidentiality label in exact identity order.
    pub fn confidentiality_labels(&self) -> impl Iterator<Item = (&str, &str)> {
        self.confidentiality
            .iter()
            .map(|(catalog, name)| (catalog.as_str(), name.as_str()))
    }

    /// Every recognized taint in exact identity order.
    pub fn taints(&self) -> impl Iterator<Item = (&str, &str)> {
        self.taints
            .iter()
            .map(|(catalog, name)| (catalog.as_str(), name.as_str()))
    }
}

/// The explicit read-only effect-catalog universe for sink resolution:
/// the set of `<catalog>/<name>` operation identities a contract's
/// external-effect sinks may reference. Resolution is exact; an
/// unresolvable sink is `unknown_flow_sink`, fail-closed. This proves
/// sink DECLARATION consistency only — never a grant.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EffectSinkUniverse {
    operations: BTreeSet<(String, String)>,
    /// Operations explicitly classified as decision-capable by the
    /// supplied catalog. An external-effect sink whose operation is not
    /// classified cannot be proven safe for `decision_influence`.
    decision_capable: BTreeSet<(String, String)>,
}

impl EffectSinkUniverse {
    /// An empty universe.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers one resolvable operation identity, optionally classified
    /// as decision-capable by the catalog.
    pub fn recognize_operation(&mut self, catalog: &str, name: &str, decision_capable: bool) {
        let key = (catalog.to_string(), name.to_string());
        self.operations.insert(key.clone());
        if decision_capable {
            self.decision_capable.insert(key);
        }
    }

    /// Whether the operation identity resolves at all.
    #[must_use]
    pub fn resolves(&self, catalog: &str, name: &str) -> bool {
        self.operations
            .contains(&(catalog.to_string(), name.to_string()))
    }

    /// Whether the catalog classifies the operation as decision-capable.
    #[must_use]
    pub fn is_decision_capable(&self, catalog: &str, name: &str) -> bool {
        self.decision_capable
            .contains(&(catalog.to_string(), name.to_string()))
    }

    /// Number of registered operations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// `true` when no operation is registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Every recognized operation and its catalog-supplied decision
    /// classification, in exact identity order.
    pub fn operations(&self) -> impl Iterator<Item = (&str, &str, bool)> {
        self.operations.iter().map(|(catalog, name)| {
            (
                catalog.as_str(),
                name.as_str(),
                self.decision_capable
                    .contains(&(catalog.clone(), name.clone())),
            )
        })
    }
}

/// The combined explicit-catalog parameter of every pure analysis entry
/// point: labels + effect sinks. All inputs, no ambient state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FlowCatalogs {
    /// The label vocabulary.
    pub labels: InformationLabelUniverse,
    /// The resolvable effect operations and their classification.
    pub sinks: EffectSinkUniverse,
}

impl FlowCatalogs {
    /// An empty catalog set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Look up the classification of one operation identity.
    #[must_use]
    pub fn sink_classification(&self, catalog: &str, name: &str) -> SinkClassification {
        if !self.sinks.resolves(catalog, name) {
            SinkClassification::Unknown
        } else if self.sinks.is_decision_capable(catalog, name) {
            SinkClassification::DecisionCapable
        } else {
            SinkClassification::Plain
        }
    }
}

/// The explicit catalog classification of one external-effect sink. The
/// classification comes from the supplied catalog only — never from the
/// operation name, its prefix or any naming convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SinkClassification {
    /// The operation does not resolve in the supplied catalog.
    Unknown,
    /// Resolvable; the catalog makes no decision-capability claim.
    Plain,
    /// Resolvable and classified decision-capable by the catalog.
    DecisionCapable,
}
