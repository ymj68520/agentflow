//! Static effect composition for the structured control opcodes.
//!
//! Pure functions over effect signatures only — no `af-ir` runtime state, no
//! journal, no plan, no scheduler. Each function is the standalone,
//! independently testable semantics a future control-flow analysis will
//! call. The IR-level structural rules (branch counts, ownership) stay in
//! `af-ir`; this module consumes already-composed branch envelopes.
//!
//! The invariant that shapes every rule: an envelope must never UNDERSTATE
//! possible effects. Where a fact that would legitimately narrow the result
//! is missing (replay/idempotency/recovery contracts), the result stays wide
//! and the missing fact is recorded as an explicit obligation.

use crate::algebra::{accumulate, branch_join, repeat};
use crate::error::{EffectError, kind};
use crate::model::{EffectSignature, OccurrenceBounds, OccurrenceCeiling};

/// The obligation recorded on every retry envelope this phase: narrowing the
/// LOGICAL effect bounds below the physical attempt scaling requires the
/// Replay/Idempotency/Recovery contracts, which do not exist yet. Until then
/// an effectful retry is conservatively treated as amplifying.
pub const RETRY_NARROWING_OBLIGATION: &str = "retry_logical_narrowing_requires_recovery_contract";

/// SEQUENCE(A, B): `εA ⊕ εB` — both children definitely run, occurrence
/// intervals add.
///
/// # Errors
/// See [`accumulate`].
pub fn sequence_envelope(
    a: &EffectSignature,
    b: &EffectSignature,
) -> Result<EffectSignature, EffectError> {
    accumulate(a, b)
}

/// PARALLEL(A, B): `εA ⊕ εB` — both branches run to completion; concurrency
/// limits (max_inflight) bound simultaneous work, never total occurrences.
///
/// # Errors
/// See [`accumulate`].
pub fn parallel_envelope(
    a: &EffectSignature,
    b: &EffectSignature,
) -> Result<EffectSignature, EffectError> {
    accumulate(a, b)
}

/// SELECT(A, B): `εA ⊔ εB` — exactly one branch runs; the join keeps the
/// possible effects of both with zero lower bounds.
///
/// # Errors
/// See [`branch_join`].
pub fn select_envelope(
    a: &EffectSignature,
    b: &EffectSignature,
) -> Result<EffectSignature, EffectError> {
    branch_join(a, b)
}

/// RACE(branches): conservative accumulation over ALL contestants. A loser
/// may already have produced effects before cancellation, so its effects are
/// never discarded — race is NOT a join.
///
/// # Errors
/// See [`accumulate`].
pub fn race_envelope(branches: &[EffectSignature]) -> Result<EffectSignature, EffectError> {
    let mut envelope = EffectSignature {
        row: crate::model::EffectRow::Closed(Vec::new()),
    };
    for branch in branches {
        envelope = accumulate(&envelope, branch)?;
    }
    Ok(envelope)
}

/// ITERATE(reps, body): `reps ⊗ ε_body` — bounded repetition scaling. A
/// foreach over a data-dependent source has no statically provable count and
/// must be given the bounds `[0, unbounded)`.
///
/// # Errors
/// See [`repeat`].
pub fn iterate_envelope(
    reps: &OccurrenceBounds,
    body: &EffectSignature,
) -> Result<EffectSignature, EffectError> {
    repeat(reps, body)
}

/// The retry result: the conservative logical envelope plus the separate
/// PHYSICAL attempt bounds, and the explicit obligation that narrows the
/// logical bound only with a future recovery contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryEnvelope {
    /// The conservative logical effect envelope: every attempt is assumed to
    /// produce fresh logical effects (no idempotency assumption this phase).
    pub logical: EffectSignature,
    /// The physical attempt bounds: at least one attempt, at most
    /// `max_attempts`.
    pub attempts: OccurrenceBounds,
    /// Always `false` this phase: without Replay/Idempotency/Recovery
    /// contracts the logical bound is NOT proven below the physical scaling.
    /// Retries of effectful work may amplify business effects.
    pub logical_narrowing_proven: bool,
    /// The static obligations attached to this envelope; exactly
    /// [`RETRY_NARROWING_OBLIGATION`] this phase.
    pub obligations: Vec<&'static str>,
}

/// RETRY(child, max_attempts): the logical envelope is conservatively
/// `attempts ⊗ ε_child`, the attempt bounds are tracked separately. An
/// idempotency qualifier declared on an atom does NOT narrow the result:
/// exploiting it requires the recovery contracts of a later phase.
///
/// # Errors
/// [`EffectError`] with kind `invalid_occurrence_bounds` when
/// `max_attempts` is zero; other kinds via [`repeat`].
pub fn retry_envelope(
    child: &EffectSignature,
    max_attempts: u64,
) -> Result<RetryEnvelope, EffectError> {
    if max_attempts == 0 {
        return Err(EffectError::new(
            kind::INVALID_OCCURRENCE_BOUNDS,
            "a retry makes at least one attempt",
        ));
    }
    let attempts = OccurrenceBounds::new(1, OccurrenceCeiling::Finite(max_attempts))?;
    let logical = repeat(&attempts, child)?;
    Ok(RetryEnvelope {
        logical,
        attempts,
        logical_narrowing_proven: false,
        obligations: vec![RETRY_NARROWING_OBLIGATION],
    })
}

/// Compensation: the forward effects REMAIN RECORDED and the counter-effect
/// ADDS to the envelope. Subtraction is unrepresentable — the original
/// effect and the counter-effect are both facts.
///
/// # Errors
/// See [`accumulate`].
pub fn compensation_envelope(
    forward: &EffectSignature,
    counter: &EffectSignature,
) -> Result<EffectSignature, EffectError> {
    accumulate(forward, counter)
}
