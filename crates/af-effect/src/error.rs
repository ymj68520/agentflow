//! Structured effect-signature errors with stable category kinds.
//!
//! Categories are stable identifiers referenced by the negative conformance
//! corpus (`spec/conformance/effect-signature/`); they must not be renamed
//! casually. A static rejection is a validation result of the declaration
//! layer — never a Workflow `FailureValue` and never a Policy
//! `PermissionDenied`.

use std::fmt;

/// Stable error-kind names used by the negative corpus.
pub mod kind {
    /// Malformed operation identity (catalog or name fails its grammar).
    pub const INVALID_EFFECT_OPERATION: &str = "invalid_effect_operation";
    /// The operation does not resolve in the supplied explicit catalog
    /// universe. Fail closed: there is no default catalog and no fallback.
    pub const UNKNOWN_EFFECT_OPERATION: &str = "unknown_effect_operation";
    /// Malformed resource selector: zero port, duplicate scope key, empty or
    /// non-NFC key/value text.
    pub const INVALID_RESOURCE_SELECTOR: &str = "invalid_resource_selector";
    /// Malformed qualifier list: the same qualifier family twice, or two
    /// conflicting members of one family.
    pub const INVALID_EFFECT_QUALIFIER: &str = "invalid_effect_qualifier";
    /// Impossible occurrence bounds (finite upper below lower) or an
    /// out-of-range attempt count.
    pub const INVALID_OCCURRENCE_BOUNDS: &str = "invalid_occurrence_bounds";
    /// The same canonical effect atom is declared twice in one row.
    pub const DUPLICATE_EFFECT_ATOM: &str = "duplicate_effect_atom";
    /// Malformed guard: zero input port or a non-canonical expected constant.
    pub const INVALID_EFFECT_GUARD: &str = "invalid_effect_guard";
    /// A residual ceiling that constrains nothing (no scope binding and an
    /// unbounded occurrence upper) — an open row without a provable ceiling.
    pub const UNBOUNDED_OPEN_EFFECT_ROW: &str = "unbounded_open_effect_row";
    /// A provable envelope was demanded (containment assertion, closed-row
    /// requirement) and the model could not prove it.
    pub const EFFECT_ENVELOPE_UNPROVABLE: &str = "effect_envelope_unprovable";
    /// Checked occurrence arithmetic overflowed. Never wraps, never silently
    /// saturates.
    pub const EFFECT_ARITHMETIC_OVERFLOW: &str = "effect_arithmetic_overflow";
    /// Two different catalogs claim the same catalog identity in one
    /// explicit universe.
    pub const EFFECT_CATALOG_COLLISION: &str = "effect_catalog_collision";
    /// The canonical ACE1 lowering failed (shape derivation or encoding).
    pub const EFFECT_ENCODING_ERROR: &str = "effect_encoding_error";
}

/// A structured effect-signature error: a stable machine kind plus a
/// human-readable detail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectError {
    kind: &'static str,
    detail: String,
}

impl EffectError {
    /// Builds an error of the given stable kind.
    pub fn new(kind: &'static str, detail: impl Into<String>) -> Self {
        EffectError {
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

impl fmt::Display for EffectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.detail)
    }
}

impl std::error::Error for EffectError {}
