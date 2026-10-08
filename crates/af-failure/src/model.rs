//! The typed failure-row model, mirroring `spec/idl/failure-row.afidl`
//! field-for-field.
//!
//! A failure type is NOMINAL: the full triple (specification_id, type_name,
//! pinned schema digest). Two refs with the same short name under different
//! specifications — or with different pinned digests — are different
//! failure types and are never silently compatible. Matching is never done
//! on error-message strings, error codes, exception class names or record
//! shapes; provider-specific mapping belongs to a later Worker Failure
//! Mapping stage.
//!
//! The row is a CLOSED set: there is no open, residual or unknown wire
//! state. Unknown failure upper bounds are an analysis-level result
//! ([`crate::control::StaticFailureEnvelope::Unproven`]), never an empty
//! row, and never a wire variant.

use af_semantic::TypeSchemaRef;
use serde::Deserialize;

/// One typed failure: an exact pin of a named AFIDL schema, reused from the
/// semantic core by full cross-module identity. The type has a typed
/// failure role in this contract exactly by being listed in a
/// [`FailureRow`].
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureTypeRef {
    pub payload_type: TypeSchemaRef,
}

impl FailureTypeRef {
    /// The specification half of the nominal identity.
    #[must_use]
    pub fn specification_id(&self) -> &str {
        &self.payload_type.specification_id
    }

    /// The type-name half of the nominal identity.
    #[must_use]
    pub fn type_name(&self) -> &str {
        &self.payload_type.type_name
    }

    /// The pinned TypeSchemaDigest hex half of the nominal identity.
    #[must_use]
    pub fn schema_hex(&self) -> &str {
        &self.payload_type.schema_digest.hex
    }

    /// The canonical sort key: the full nominal identity in byte order
    /// (specification, then type name, then pinned digest hex).
    #[must_use]
    pub fn nominal_key(&self) -> (&str, &str, &str) {
        (self.specification_id(), self.type_name(), self.schema_hex())
    }

    /// Exact nominal equality against another ref (all three identity parts).
    #[must_use]
    pub fn is_same_type(&self, other: &Self) -> bool {
        self.nominal_key() == other.nominal_key()
    }
}

/// The closed Failure Row `Φ = {F1, F2, ... Fn}` of one contract.
///
/// `Φ = ∅` does NOT mean the computation succeeds: `TimedOut` and
/// `Cancelled` are Control Outcomes, `RecoveryBlocked` is a Recovery State,
/// `TaskAttemptLost` is an Infrastructure Fact and `RuntimeIncident` lives
/// on the Incident Plane — none of them is a failure type, and none of them
/// enters this row.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureRow {
    pub failures: Vec<FailureTypeRef>,
}

impl FailureRow {
    /// The empty row `Φ = ∅` — the algebra's identity. It does NOT mean
    /// the computation succeeds (control outcomes, recovery states,
    /// infrastructure facts and incidents all remain possible outside Φ).
    #[must_use]
    pub fn empty() -> Self {
        FailureRow {
            failures: Vec::new(),
        }
    }
}
