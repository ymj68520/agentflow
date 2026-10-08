//! Static failure propagation for the structured control opcodes.
//!
//! Pure functions over failure rows only — no `af-ir` runtime state, no
//! HANDLE state machine, no retry policy. Each function is the standalone,
//! independently testable semantics a future control-flow analysis will
//! call. The invariant throughout: a control node may only ADD the typed
//! failures its children (or handlers) declare — it never invents a
//! business failure type, never erases one, and never mistakes a control
//! outcome for a failure.

use crate::algebra::{propagate_handler_failures, union};
use crate::error::{FailureError, kind};
use crate::model::FailureRow;

/// SEQUENCE: the union of the failure rows of every child that may run.
#[must_use]
pub fn sequence_row(rows: &[FailureRow]) -> FailureRow {
    union_all(rows)
}

/// SELECT: the union over all selectable branches (exactly one runs, but
/// which one is data-dependent).
#[must_use]
pub fn select_row(rows: &[FailureRow]) -> FailureRow {
    union_all(rows)
}

/// PARALLEL: the union over all branches (all may run).
#[must_use]
pub fn parallel_row(rows: &[FailureRow]) -> FailureRow {
    union_all(rows)
}

/// RACE: the union over all contestants — a loser may have failed before
/// cancellation, so no branch's row is dropped.
#[must_use]
pub fn race_row(rows: &[FailureRow]) -> FailureRow {
    union_all(rows)
}

/// ITERATE: the body's failure row verbatim. Failure TYPES do not multiply
/// with repetitions — a row is a set of nominal types, not an occurrence
/// count.
#[must_use]
pub fn iterate_row(body: &FailureRow) -> FailureRow {
    body.clone()
}

/// RETRY: the child's failure row preserved verbatim. Retry attempts are
/// NOT a proof that business failure is impossible; only a future
/// Recovery/Replay Contract may legitimately narrow this row.
#[must_use]
pub fn retry_row(child: &FailureRow) -> FailureRow {
    child.clone()
}

/// TIMEOUT: the child's failure row preserved; `TimedOut` is a Control
/// Outcome (the frozen `HandleMatch::timed_out`) and never becomes a
/// failure type like `Failure<TimedOut>`.
#[must_use]
pub fn timeout_row(child: &FailureRow) -> FailureRow {
    child.clone()
}

/// SCOPE: the child's failure row verbatim.
#[must_use]
pub fn scope_row(child: &FailureRow) -> FailureRow {
    child.clone()
}

/// COMPENSATION_SCOPE: the child's forward failure row verbatim.
/// Compensation never erases forward facts and never maps recovery states
/// into business failures.
#[must_use]
pub fn compensation_scope_row(child: &FailureRow) -> FailureRow {
    child.clone()
}

/// AWAIT: the empty row. Waiting for a signal invents no business failure
/// type.
#[must_use]
pub fn await_row() -> FailureRow {
    FailureRow::empty()
}

/// HANDLE: `(Φ_child \ H) ∪ Φ_handler_1 ∪ …` — the static propagation
/// formula. `H` may only contain failures the child declares.
///
/// # Errors
/// [`FailureError`] with kind `failure_not_declared` when `handled` is not
/// a subset of `child`.
pub fn handle_row(
    child: &FailureRow,
    handled: &FailureRow,
    handlers: &[FailureRow],
) -> Result<FailureRow, FailureError> {
    propagate_handler_failures(child, handled, handlers)
}

fn union_all(rows: &[FailureRow]) -> FailureRow {
    let mut result = FailureRow::empty();
    for row in rows {
        result = union(&result, row);
    }
    result
}

/// The analysis-level result of a control node whose failure upper bound
/// is not statically known THIS phase: INVOKE resolves from the full
/// ComponentContract and CALL_SUBFLOW from the Subflow Failure Contract,
/// neither of which exists yet. `Unproven` is a first-class analysis
/// result — it is NEVER interpreted as the empty row, and it has no wire
/// representation (the Failure Row itself stays a closed set).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaticFailureEnvelope {
    /// The failure row is proven.
    Proven(FailureRow),
    /// The failure upper bound is unknown to this phase's static analysis.
    Unproven,
}

impl StaticFailureEnvelope {
    /// Builds a proven envelope.
    #[must_use]
    pub fn proven(row: FailureRow) -> Self {
        StaticFailureEnvelope::Proven(row)
    }

    /// Builds the unproven envelope.
    #[must_use]
    pub fn unproven() -> Self {
        StaticFailureEnvelope::Unproven
    }

    /// Demands a provable row.
    ///
    /// # Errors
    /// [`FailureError`] with kind `failure_row_unproven` for the unproven
    /// envelope — the fail-closed answer to "give me this control node's Φ"
    /// when the phase's analysis cannot supply one.
    pub fn require_row(self) -> Result<FailureRow, FailureError> {
        match self {
            StaticFailureEnvelope::Proven(row) => Ok(row),
            StaticFailureEnvelope::Unproven => Err(FailureError::new(
                kind::FAILURE_ROW_UNPROVEN,
                "the failure upper bound of this control node is not \
                 provable in this phase: Unproven is never interpreted as \
                 the empty row, and contract-resolved failure propagation \
                 is deferred to the Component Contract composition stage",
            )),
        }
    }
}
