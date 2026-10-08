//! Structured resource-requirement errors with stable category kinds.
//!
//! Categories are stable identifiers referenced by the negative
//! conformance corpus (`spec/conformance/resource-requirement/`); they
//! must not be renamed casually. A static rejection is a compile-time
//! result of the contract layer — never a runtime authority denial, never
//! a scheduling condition, never an incident.

use std::fmt;

/// Stable error-kind names used by the negative corpus.
pub mod kind {
    /// Malformed nominal resource-type reference: the specification id,
    /// type name or digest shape fails its grammar.
    pub const INVALID_RESOURCE_TYPE: &str = "invalid_resource_type";
    /// The referenced schema is not registered in the supplied universe,
    /// or is registered but not explicitly recognized as a resource type.
    /// Fail closed: there is no latest-schema fallback and a legal AFIDL
    /// schema alone is not a resource type.
    pub const UNKNOWN_RESOURCE_TYPE: &str = "unknown_resource_type";
    /// The schema exists but the pinned digest differs — same logical
    /// name, different identity; never silently compatible.
    pub const RESOURCE_TYPE_DIGEST_MISMATCH: &str = "resource_type_digest_mismatch";
    /// A requirement declares port 0 (or otherwise violates the positive
    /// port rule).
    pub const INVALID_RESOURCE_PORT: &str = "invalid_resource_port";
    /// The same port number is declared by two requirements of one row.
    pub const DUPLICATE_RESOURCE_PORT: &str = "duplicate_resource_port";
    /// An attribute key or value is empty (after NFC normalization).
    pub const INVALID_RESOURCE_ATTRIBUTE: &str = "invalid_resource_attribute";
    /// The same normalized attribute key is declared twice within one
    /// requirement's constraint list.
    pub const DUPLICATE_RESOURCE_ATTRIBUTE: &str = "duplicate_resource_attribute";
    /// An attribute key is not part of the resource type's explicit
    /// descriptor vocabulary: its constraint meaning is unknown.
    pub const UNKNOWN_RESOURCE_ATTRIBUTE: &str = "unknown_resource_attribute";
    /// A candidate resource fails a requirement's static predicate.
    pub const RESOURCE_REQUIREMENT_UNSATISFIED: &str = "resource_requirement_unsatisfied";
    /// A static compatibility judgment could not be proven; Unproven is
    /// never auto-judged compatible.
    pub const RESOURCE_REQUIREMENT_UNPROVEN: &str = "resource_requirement_unproven";
    /// A compatibility comparison proved that the new row strengthens the
    /// old one (a new port, or a new/changed exact attribute condition on
    /// a retained port).
    pub const RESOURCE_REQUIREMENT_STRENGTHENED: &str = "resource_requirement_strengthened";
    /// The canonical ACE1 lowering failed (shape derivation or encoding).
    pub const RESOURCE_ENCODING_ERROR: &str = "resource_encoding_error";
}

/// A structured resource error: a stable machine kind plus a
/// human-readable detail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceError {
    kind: &'static str,
    detail: String,
}

impl ResourceError {
    /// Builds an error of the given stable kind.
    pub fn new(kind: &'static str, detail: impl Into<String>) -> Self {
        ResourceError {
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

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.detail)
    }
}

impl std::error::Error for ResourceError {}
