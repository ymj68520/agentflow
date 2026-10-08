//! Static Recovery Contract model.

use af_effect::{EffectAtom, EffectOperationRef};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplaySemantics {
    Recompute,
    Recorded,
    Reconcile,
    Forbidden,
}

impl ReplaySemantics {
    #[must_use]
    pub fn value_no(self) -> u32 {
        match self {
            Self::Recompute => 1,
            Self::Recorded => 2,
            Self::Reconcile => 3,
            Self::Forbidden => 4,
        }
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Recompute => "recompute",
            Self::Recorded => "recorded",
            Self::Reconcile => "reconcile",
            Self::Forbidden => "forbidden",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdempotencySemantics {
    Natural,
    Keyed,
    None,
    Reconciled,
}

impl IdempotencySemantics {
    #[must_use]
    pub fn value_no(self) -> u32 {
        match self {
            Self::Natural => 1,
            Self::Keyed => 2,
            Self::None => 3,
            Self::Reconciled => 4,
        }
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Natural => "natural",
            Self::Keyed => "keyed",
            Self::None => "none",
            Self::Reconciled => "reconciled",
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyStabilityRequirement {
    pub stable_across_attempts: bool,
    pub distinct_across_invocations: bool,
    pub bound_to_operation: bool,
    pub provider_honors: bool,
}

impl KeyStabilityRequirement {
    #[must_use]
    pub fn all_declared(&self) -> bool {
        self.stable_across_attempts
            && self.distinct_across_invocations
            && self.bound_to_operation
            && self.provider_honors
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconciliationRequirement {
    None,
    Required,
}

impl ReconciliationRequirement {
    #[must_use]
    pub fn value_no(self) -> u32 {
        match self {
            Self::None => 1,
            Self::Required => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompensationRequirement {
    None,
    Required(EffectOperationRef),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnknownEffectHandling {
    Block,
    ReconciliationRequired,
}

impl UnknownEffectHandling {
    #[must_use]
    pub fn value_no(self) -> u32 {
        match self {
            Self::Block => 1,
            Self::ReconciliationRequired => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectRecoveryClause {
    pub effect: EffectAtom,
    pub idempotency: IdempotencySemantics,
    pub key_requirement: Option<KeyStabilityRequirement>,
    pub reconciliation: ReconciliationRequirement,
    pub compensation: CompensationRequirement,
    pub unknown_effect_handling: UnknownEffectHandling,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryContract {
    pub replay_semantics: ReplaySemantics,
    pub effect_recovery_clauses: Vec<EffectRecoveryClause>,
}
