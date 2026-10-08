//! Exact-pinned validation against an explicit schema universe.
//!
//! Validation reuses the semantic core's explicit `TypeSchemaUniverse` (or
//! an equivalent already-verified module set). Every referenced named
//! schema must exist AND its pinned digest must match the universe's
//! computed digest. There is no latest-schema lookup, no fallback and no
//! implicit upcast: two legal versions of the same type keep their full
//! nominal identities and stay different failure types.

use crate::canonical::canonicalize;
use crate::error::{FailureError, kind};
use crate::model::{FailureRow, FailureTypeRef};
use af_semantic::SemanticError;

impl FailureRow {
    /// Validates every failure type against the explicit universe and
    /// returns the canonical row.
    ///
    /// # Errors
    /// [`FailureError`] with the structural kinds of
    /// [`FailureRow::canonical`], then, per ref in canonical order:
    /// `unknown_failure_schema` (the named schema is not in the universe),
    /// `failure_schema_digest_mismatch` (same name, different pinned
    /// identity).
    pub fn validate(
        &self,
        universe: &af_semantic::TypeSchemaUniverse,
    ) -> Result<FailureRow, FailureError> {
        let canonical = canonicalize(self)?;
        for failure in &canonical.failures {
            resolve_failure(failure, universe)?;
        }
        Ok(canonical)
    }
}

/// Resolves one failure type exactly: exists AND pinned digest matches.
///
/// # Errors
/// [`FailureError`] with kinds `unknown_failure_schema` and
/// `failure_schema_digest_mismatch`.
pub fn resolve_failure(
    failure: &FailureTypeRef,
    universe: &af_semantic::TypeSchemaUniverse,
) -> Result<(), FailureError> {
    universe.resolve(&failure.payload_type).map_err(|error| {
        let identity = format!("{}/{}", failure.specification_id(), failure.type_name());
        match &error {
            SemanticError::UnknownTypeSchema { .. } => FailureError::new(
                kind::UNKNOWN_FAILURE_SCHEMA,
                format!(
                    "failure payload type {identity} does not resolve in the \
                     supplied schema universe (no latest-schema fallback)"
                ),
            ),
            SemanticError::TypeSchemaDigestMismatch {
                expected, actual, ..
            } => FailureError::new(
                kind::FAILURE_SCHEMA_DIGEST_MISMATCH,
                format!(
                    "failure payload type {identity} is pinned to \
                         {expected} but the universe computes {actual}: same \
                         logical name, different identity — never silently \
                         compatible"
                ),
            ),
            other => FailureError::new(
                kind::INVALID_FAILURE_TYPE,
                format!("failure payload type {identity} is malformed: {other}"),
            ),
        }
    })?;
    Ok(())
}
