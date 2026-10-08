//! Canonical ordering and the canonical ACE1 bytes.
//!
//! Canonicalization is a pure function of the declared set: refs sorted by
//! their full nominal identity in byte order (specification id, type name,
//! pinned digest hex), duplicates rejected. Input declaration order never
//! reaches the canonical bytes, and no HashMap iteration order is ever
//! observable.
//!
//! The canonical form has ACE1 bytes but deliberately NO digest: no
//! `FailureRowDigest` exists, the HashV1 domain registry is untouched, and
//! the future ComponentContract will hash as a whole under the frozen
//! `component-contract` domain.

use crate::error::{FailureError, kind};
use crate::model::{FailureRow, FailureTypeRef};
use af_core::Value;
use std::collections::BTreeSet;

/// The byte-comparable sort key of one failure type. Used for canonical
/// ordering; ordering is over the full nominal identity only.
#[must_use]
pub fn nominal_sort_key(failure: &FailureTypeRef) -> (&str, &str, &str) {
    failure.nominal_key()
}

/// Canonicalizes the declared set: syntax-checked refs, duplicates
/// rejected, sorted by nominal identity. Does NOT resolve the universe —
/// resolution is [`FailureRow::validate`]'s job (see `validate.rs`).
///
/// # Errors
/// [`FailureError`] with kinds `invalid_failure_type` (malformed identity)
/// and `duplicate_failure_type`.
pub fn canonicalize(row: &FailureRow) -> Result<FailureRow, FailureError> {
    let mut seen = BTreeSet::new();
    let mut failures = Vec::with_capacity(row.failures.len());
    for failure in &row.failures {
        check_identity(failure)?;
        let key = (
            failure.specification_id(),
            failure.type_name(),
            failure.schema_hex(),
        );
        if !seen.insert(key) {
            return Err(FailureError::new(
                kind::DUPLICATE_FAILURE_TYPE,
                format!(
                    "failure type {}/{}/{:.8}… is declared twice: a declared \
                     row is a set, not a multiset (algebra unions deduplicate; \
                     declarations do not)",
                    failure.specification_id(),
                    failure.type_name(),
                    failure.schema_hex()
                ),
            ));
        }
        failures.push(failure.clone());
    }
    failures.sort_by(|a, b| nominal_sort_key(a).cmp(&nominal_sort_key(b)));
    Ok(FailureRow { failures })
}

/// Syntax check of one nominal reference (no universe needed).
///
/// # Errors
/// [`FailureError`] with kind `invalid_failure_type`.
pub fn check_identity(failure: &FailureTypeRef) -> Result<(), FailureError> {
    if af_idl::SpecificationId::parse(failure.specification_id()).is_err() {
        return Err(FailureError::new(
            kind::INVALID_FAILURE_TYPE,
            format!(
                "specification id {:?} is not a SpecificationId",
                failure.specification_id()
            ),
        ));
    }
    let name = failure.type_name();
    if name.is_empty() || name.chars().any(char::is_whitespace) {
        return Err(FailureError::new(
            kind::INVALID_FAILURE_TYPE,
            format!("type name {name:?} is not a valid AFIDL type name"),
        ));
    }
    // Full digest grammar: sha256, the frozen type-schema domain, 64
    // lowercase hex characters.
    failure
        .payload_type
        .schema_digest
        .validate_for_type_schema()
        .map_err(|error| {
            FailureError::new(
                kind::INVALID_FAILURE_TYPE,
                format!("pinned schema digest is not a TypeSchemaDigest: {error}"),
            )
        })
}

impl FailureRow {
    /// `true` for the empty row `Φ = ∅`. The empty row is the algebra's
    /// identity — it does NOT mean the computation succeeds.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.failures.is_empty()
    }

    /// Number of declared failure types.
    #[must_use]
    pub fn len(&self) -> usize {
        self.failures.len()
    }

    /// Whether the exact nominal failure type is declared (all three
    /// identity parts; never the short name alone).
    #[must_use]
    pub fn declares(&self, failure: &FailureTypeRef) -> bool {
        self.failures.iter().any(|f| f.is_same_type(failure))
    }

    /// The canonical form: syntax-checked, duplicate-free, sorted by
    /// nominal identity.
    ///
    /// # Errors
    /// [`FailureError`] with kinds `invalid_failure_type` and
    /// `duplicate_failure_type`.
    pub fn canonical(&self) -> Result<FailureRow, FailureError> {
        canonicalize(self)
    }

    /// The canonical ACE1 bytes, encoded against the schema-derived shape
    /// of `agentflow.failure.row@1/FailureRow` within `set` (which must
    /// contain the failure-row module and the semantic core it imports).
    /// Requires a structurally valid row.
    ///
    /// # Errors
    /// [`FailureError`] with kind `failure_encoding_error` when the
    /// canonical form is invalid or the value tree cannot be encoded.
    pub fn canonical_bytes(
        &self,
        set: &af_idl::CompiledModuleSet,
    ) -> Result<Vec<u8>, FailureError> {
        let value = self.to_value()?;
        af_core::encode(&value_shape(set)?, &value).map_err(|error| {
            FailureError::new(
                kind::FAILURE_ENCODING_ERROR,
                format!("cannot encode the canonical failure row: {error}"),
            )
        })
    }

    /// The ACE1 value tree of the canonical row (mirrors the AFIDL schema
    /// field-for-field; the encoder emits members in byte order).
    ///
    /// # Errors
    /// [`FailureError`] when the row is not structurally valid.
    pub fn to_value(&self) -> Result<Value, FailureError> {
        let canonical = canonicalize(self)?;
        Ok(Value::Record(vec![(
            "failures".into(),
            Value::List(
                canonical
                    .failures
                    .iter()
                    .map(|failure| {
                        Value::Record(vec![(
                            "payload_type".into(),
                            Value::Record(vec![
                                (
                                    "specification_id".into(),
                                    Value::String(failure.specification_id().to_string()),
                                ),
                                (
                                    "type_name".into(),
                                    Value::String(failure.type_name().to_string()),
                                ),
                                (
                                    "schema_digest".into(),
                                    Value::Record(vec![
                                        (
                                            "algorithm".into(),
                                            Value::String(
                                                failure
                                                    .payload_type
                                                    .schema_digest
                                                    .algorithm
                                                    .clone(),
                                            ),
                                        ),
                                        (
                                            "domain".into(),
                                            Value::String(
                                                failure.payload_type.schema_digest.domain.clone(),
                                            ),
                                        ),
                                        (
                                            "hex".into(),
                                            Value::String(failure.schema_hex().to_string()),
                                        ),
                                    ]),
                                ),
                            ]),
                        )])
                    })
                    .collect(),
            ),
        )]))
    }
}

fn value_shape(set: &af_idl::CompiledModuleSet) -> Result<af_core::Shape, FailureError> {
    let shape = set
        .shape_of("agentflow.failure.row@1/FailureRow")
        .map_err(|error| {
            FailureError::new(
                kind::FAILURE_ENCODING_ERROR,
                format!("cannot derive the failure-row shape: {error}"),
            )
        })?;
    shape.validate().map_err(|error| {
        FailureError::new(
            kind::FAILURE_ENCODING_ERROR,
            format!("derived shape is invalid: {error}"),
        )
    })?;
    Ok(shape)
}
