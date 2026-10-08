//! Structured failure-row errors with stable category kinds.
//!
//! Categories are stable identifiers referenced by the negative conformance
//! corpus (`spec/conformance/failure-row/`); they must not be renamed
//! casually. A static rejection is a compile-time result of the contract
//! layer — never a Workflow `FailureValue`, never a `PolicyDenied`, never
//! a `RuntimeIncident`.

use std::fmt;

/// Stable error-kind names used by the negative corpus.
pub mod kind {
    /// Malformed nominal reference: the specification id, type name or
    /// digest shape fails its grammar.
    pub const INVALID_FAILURE_TYPE: &str = "invalid_failure_type";
    /// The referenced named schema does not exist in the supplied explicit
    /// universe. Fail closed: there is no latest-schema fallback.
    pub const UNKNOWN_FAILURE_SCHEMA: &str = "unknown_failure_schema";
    /// The named schema exists but the pinned digest differs: same logical
    /// name, different identity — never silently compatible.
    pub const FAILURE_SCHEMA_DIGEST_MISMATCH: &str = "failure_schema_digest_mismatch";
    /// The same nominal failure type is declared twice in one row.
    pub const DUPLICATE_FAILURE_TYPE: &str = "duplicate_failure_type";
    /// A handled (or required) failure type is not declared by the child
    /// row the operation applies to.
    pub const FAILURE_NOT_DECLARED: &str = "failure_not_declared";
    /// A provable failure row was demanded from a static analysis whose
    /// failure upper bound is unknown — Unproven is never the empty row.
    pub const FAILURE_ROW_UNPROVEN: &str = "failure_row_unproven";
    /// The canonical ACE1 lowering failed (shape derivation or encoding).
    pub const FAILURE_ENCODING_ERROR: &str = "failure_encoding_error";
}

/// A structured failure-row error: a stable machine kind plus a
/// human-readable detail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureError {
    kind: &'static str,
    detail: String,
}

impl FailureError {
    /// Builds an error of the given stable kind.
    pub fn new(kind: &'static str, detail: impl Into<String>) -> Self {
        FailureError {
            kind,
            detail: detail.into(),
        }
    }

    /// The stable machine-readable kind.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        self.kind
    }

    /// The human-readable detail.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl fmt::Display for FailureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.detail)
    }
}

impl std::error::Error for FailureError {}
