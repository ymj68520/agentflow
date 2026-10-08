//! Conservative static propagation for structured control forms.

use crate::algebra::{accumulate, branch_join, repeat};
use crate::error::UsageError;
use crate::model::StaticUsageEnvelope;

/// SEQUENCE: both children execute, so usage accumulates.
pub fn sequence_envelope(
    a: &StaticUsageEnvelope,
    b: &StaticUsageEnvelope,
) -> Result<StaticUsageEnvelope, UsageError> {
    accumulate(a, b)
}

/// PARALLEL: all branches can consume, so usage accumulates; concurrency is
/// not a total-iteration bound.
pub fn parallel_envelope(
    branches: &[StaticUsageEnvelope],
) -> Result<StaticUsageEnvelope, UsageError> {
    accumulate_all(branches)
}

/// SELECT: one branch runs, so usage joins by maximum.
pub fn select_envelope(
    branches: &[StaticUsageEnvelope],
) -> Result<StaticUsageEnvelope, UsageError> {
    join_all(branches)
}

/// RACE: every contestant may consume before cancellation; losers are kept.
pub fn race_envelope(branches: &[StaticUsageEnvelope]) -> Result<StaticUsageEnvelope, UsageError> {
    accumulate_all(branches)
}

/// ITERATE: a known count scales the body. An unknown count keeps every body
/// coordinate unknown instead of using `max_inflight` as a fake count.
pub fn iterate_envelope(
    repetitions: Option<u64>,
    body: &StaticUsageEnvelope,
) -> Result<StaticUsageEnvelope, UsageError> {
    if let Some(count) = repetitions {
        repeat(count, body)
    } else {
        let mut unknown = StaticUsageEnvelope::empty();
        for coordinate in body.coordinates() {
            unknown.insert(coordinate, crate::model::StaticUsageBound::Unknown)?;
        }
        Ok(unknown)
    }
}

/// RETRY scales metered usage by the maximum possible attempt count. Logical
/// usage and physical attempts remain separate, and no idempotency refund is
/// inferred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryUsageEnvelope {
    pub logical: StaticUsageEnvelope,
    pub max_attempts: u64,
    pub logical_narrowing_proven: bool,
}

/// The obligation attached to every retry envelope in this phase.
pub const RETRY_USAGE_NARROWING_OBLIGATION: &str =
    "retry_logical_narrowing_requires_recovery_contract";

pub fn retry_envelope(
    body: &StaticUsageEnvelope,
    max_attempts: u64,
) -> Result<RetryUsageEnvelope, UsageError> {
    if max_attempts == 0 {
        return Err(UsageError::new(
            crate::error::kind::INVALID_USAGE_PHASE,
            "a retry must have at least one possible attempt",
        ));
    }
    Ok(RetryUsageEnvelope {
        logical: repeat(max_attempts, body)?,
        max_attempts,
        logical_narrowing_proven: false,
    })
}

/// TIMEOUT does not erase already incurred or potentially incurred usage.
pub fn timeout_envelope(body: &StaticUsageEnvelope) -> StaticUsageEnvelope {
    body.clone()
}

/// HANDLE conservatively includes protected child and possible handlers.
pub fn handle_envelope(
    protected: &StaticUsageEnvelope,
    handlers: &[StaticUsageEnvelope],
) -> Result<StaticUsageEnvelope, UsageError> {
    let mut all = Vec::with_capacity(handlers.len() + 1);
    all.push(protected.clone());
    all.extend_from_slice(handlers);
    accumulate_all(&all)
}

/// SCOPE does not make static usage disappear.
pub fn scope_envelope(body: &StaticUsageEnvelope) -> StaticUsageEnvelope {
    body.clone()
}

/// COMPENSATION_SCOPE keeps forward and recovery coordinates side by side;
/// recovery usage never subtracts forward usage.
pub fn compensation_envelope(
    forward: &StaticUsageEnvelope,
    recovery: &StaticUsageEnvelope,
) -> Result<StaticUsageEnvelope, UsageError> {
    accumulate(forward, recovery)
}

/// INVOKE without a later ComponentContract usage envelope is unproven.
pub fn invoke_envelope(declared: Option<&StaticUsageEnvelope>) -> StaticUsageEnvelope {
    declared
        .cloned()
        .unwrap_or_else(StaticUsageEnvelope::unknown)
}

/// CALL_SUBFLOW has the same deferred contract boundary.
pub fn call_subflow_envelope(declared: Option<&StaticUsageEnvelope>) -> StaticUsageEnvelope {
    invoke_envelope(declared)
}

/// AWAIT cannot invent an all-meter zero claim; it preserves any statically
/// known usage supplied by the surrounding analysis.
pub fn await_envelope(existing: &StaticUsageEnvelope) -> StaticUsageEnvelope {
    existing.clone()
}

fn accumulate_all(envelopes: &[StaticUsageEnvelope]) -> Result<StaticUsageEnvelope, UsageError> {
    let Some((first, rest)) = envelopes.split_first() else {
        return Ok(StaticUsageEnvelope::empty());
    };
    let mut result = first.clone();
    for envelope in rest {
        result = accumulate(&result, envelope)?;
    }
    Ok(result)
}

fn join_all(envelopes: &[StaticUsageEnvelope]) -> Result<StaticUsageEnvelope, UsageError> {
    let Some((first, rest)) = envelopes.split_first() else {
        return Ok(StaticUsageEnvelope::empty());
    };
    let mut result = first.clone();
    for envelope in rest {
        result = branch_join(&result, envelope)?;
    }
    Ok(result)
}
