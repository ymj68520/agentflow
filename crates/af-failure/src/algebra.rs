//! The failure algebra: containment (⊆), union (∪), join and handled-set
//! subtraction — pure functions over closed failure rows, independent of
//! any runtime and of `af-ir`.
//!
//! Failure types compare by EXACT nominal identity only: never by short
//! name, similar schema or equal record fields. A future subtype or
//! compatibility system would require an explicit Compatibility Proof and
//! is deliberately not part of this phase.

use crate::error::{FailureError, kind};
use crate::model::{FailureRow, FailureTypeRef};
use std::collections::BTreeMap;

/// The exact nominal key of a failure type as an owned, comparable tuple.
fn key_of(failure: &FailureTypeRef) -> (String, String, String) {
    (
        failure.specification_id().to_string(),
        failure.type_name().to_string(),
        failure.schema_hex().to_string(),
    )
}

/// Deterministic set semantics: deduplicate by nominal identity and order
/// by the canonical byte order, whatever the input order was.
fn assemble(entries: BTreeMap<(String, String, String), FailureTypeRef>) -> FailureRow {
    FailureRow {
        failures: entries.into_values().collect(),
    }
}

/// `Φa ⊆ Φb` holds iff every nominal failure type declared by `Φa` is
/// declared by `Φb` — all three identity parts. Containment between
/// different types is never inferred (no short-name match, no structural
/// similarity, no subtyping): a future compatibility system needs an
/// explicit proof and does not exist in this phase.
#[must_use]
pub fn contains(a: &FailureRow, b: &FailureRow) -> bool {
    a.failures.iter().all(|failure| b.declares(failure))
}

/// `Φa ∪ Φb` — the union of both declared sets, deduplicated and
/// canonically ordered. Union MAY deduplicate declared duplicates (this is
/// the algebra; declared rows themselves reject duplicates in
/// validation). Commutative, associative, idempotent; `∅` is the identity.
#[must_use]
pub fn union(a: &FailureRow, b: &FailureRow) -> FailureRow {
    let mut merged: BTreeMap<(String, String, String), FailureTypeRef> = BTreeMap::new();
    for failure in a.failures.iter().chain(&b.failures) {
        merged
            .entry(key_of(failure))
            .or_insert_with(|| failure.clone());
    }
    assemble(merged)
}

/// Branch composition (SELECT / PARALLEL / RACE): for closed failure rows
/// the possible failures of either branch is exactly the SET UNION, so
/// `join` IS [`union`] — named separately so the control surface reads
/// like the frozen propagation table.
#[must_use]
pub fn join(a: &FailureRow, b: &FailureRow) -> FailureRow {
    union(a, b)
}

/// `Φ \ H` — removes exactly the handled types from `Φ`.
///
/// This is a static residual computation after a full type check; it does
/// NOT mean a HANDLE has executed or will succeed.
///
/// # Errors
/// [`FailureError`] with kind `failure_not_declared` when `H` contains a
/// type `Φ` never declared: handled sets must be subsets of the declared
/// child row, and pretending an unknown type was handled is never allowed.
pub fn remove_handled(row: &FailureRow, handled: &FailureRow) -> Result<FailureRow, FailureError> {
    for failure in &handled.failures {
        if !row.declares(failure) {
            return Err(FailureError::new(
                kind::FAILURE_NOT_DECLARED,
                format!(
                    "handled failure type {}/{}/{} is not declared by the \
                     child failure row: a HANDLE matcher must match a typed \
                     failure the child actually declares",
                    failure.specification_id(),
                    failure.type_name(),
                    failure.schema_hex()
                ),
            ));
        }
    }
    let mut remaining: BTreeMap<(String, String, String), FailureTypeRef> = BTreeMap::new();
    for failure in &row.failures {
        if !handled.declares(failure) {
            remaining
                .entry(key_of(failure))
                .or_insert_with(|| failure.clone());
        }
    }
    Ok(assemble(remaining))
}

/// Static HANDLE propagation:
///
/// ```text
/// Φ_result = (Φ_child \ H) ∪ Φ_handler_1 ∪ Φ_handler_2 ∪ …
/// ```
///
/// - `H` may only contain typed failures the child declares (checked);
/// - handlers may fail themselves: their rows are unions into the result,
///   never treated as "automatically handled";
/// - unhandled child failures survive verbatim.
///
/// This is the static propagation formula, not the HANDLE runtime state
/// machine.
///
/// # Errors
/// [`FailureError`] with kind `failure_not_declared` when `handled` is not
/// a subset of `child`.
pub fn propagate_handler_failures(
    child: &FailureRow,
    handled: &FailureRow,
    handlers: &[FailureRow],
) -> Result<FailureRow, FailureError> {
    let residual = remove_handled(child, handled)?;
    let mut result = residual;
    for handler in handlers {
        result = union(&result, handler);
    }
    Ok(result)
}
