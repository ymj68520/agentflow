//! Structured information-flow errors with stable category kinds.
//!
//! Categories are stable identifiers referenced by the negative
//! conformance corpus (`spec/conformance/information-flow/`); they must
//! not be renamed casually. A static rejection is a compile-time result
//! of the contract layer — never a `PolicyDenied`, never an
//! `EffectDenied`, never a runtime incident, never a workflow failure
//! value.

use std::fmt;

/// Stable error-kind names used by the negative corpus.
pub mod kind {
    /// Malformed label reference: the catalog id, label name or NFC text
    /// fails its grammar, or a taint reference is malformed.
    pub const INVALID_INFORMATION_LABEL: &str = "invalid_information_label";
    /// The named label (confidentiality restriction or taint) does not
    /// resolve in the supplied explicit universe. Fail closed: no latest
    /// fallback, no case-insensitive matching, no auto-registration.
    pub const UNKNOWN_INFORMATION_LABEL: &str = "unknown_information_label";
    /// The same nominal label is declared twice in one declaration list.
    pub const DUPLICATE_INFORMATION_LABEL: &str = "duplicate_information_label";
    /// A source reference violates its grammar (port 0) or a summary is
    /// structurally invalid.
    pub const INVALID_FLOW_SOURCE: &str = "invalid_flow_source";
    /// A sink reference violates its grammar (port 0) or its effect
    /// operation reference is malformed.
    pub const INVALID_FLOW_SINK: &str = "invalid_flow_sink";
    /// The sink's effect operation does not resolve in the supplied
    /// explicit effect catalog universe.
    pub const UNKNOWN_FLOW_SINK: &str = "unknown_flow_sink";
    /// A flow constraint list declares the same (kind, sink) twice, or a
    /// constraint is structurally invalid.
    pub const DUPLICATE_FLOW_CONSTRAINT: &str = "duplicate_flow_constraint";
    /// A flow was asserted as proven but the static proof does not hold
    /// or was never supplied; Unproven is never judged allowed.
    pub const FLOW_NOT_PROVEN: &str = "flow_not_proven";
    /// A compatibility comparison proved that the new contract relaxes
    /// the old one (a new permitted sink, or a dropped confidentiality
    /// restriction).
    pub const FLOW_CONTRACT_RELAXED: &str = "flow_contract_relaxed";
    /// The flow needs a declassification or endorsement that this static
    /// layer cannot supply: the obligation is recorded, never executed.
    pub const PRIVILEGED_TRANSFORMATION_REQUIRED: &str = "privileged_transformation_required";
    /// The canonical ACE1 lowering failed (shape derivation or encoding).
    pub const FLOW_ENCODING_ERROR: &str = "flow_encoding_error";
}

/// A structured information-flow error: a stable machine kind plus a
/// human-readable detail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowError {
    kind: &'static str,
    detail: String,
}

impl FlowError {
    /// Builds an error of the given stable kind.
    pub fn new(kind: &'static str, detail: impl Into<String>) -> Self {
        FlowError {
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

impl fmt::Display for FlowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.detail)
    }
}

impl std::error::Error for FlowError {}
