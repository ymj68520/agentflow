//! Root-relative static K action and recovery-usage projection.

use af_contract::{
    ActionGuardCompatibility, RecoveryActionIdentity, RecoveryActionKind, RecoveryUsageBinding,
};
use af_effect::OccurrenceCeiling;
use af_semantic::{DigestRefValue, ExpressionId, ValueSlotId};
use af_usage::StaticUsageEnvelope;

use crate::control::{ControlDimension, ControlProofReport};
use crate::proof::LeafVerdict;
use crate::summary::{ResolvedTarget, ScopedEffectOccurrence};
use af_ir::ComputationId;
use serde::Deserialize;

/// Identity of one repeat factor. Physical retry attempts remain separate
/// metadata and are never multiplied a second time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlMultiplicityIdentity {
    pub caller_semantic_ir_digest: DigestRefValue,
    pub control_node: ComputationId,
    pub kind: ControlMultiplicityKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ControlMultiplicityKind {
    Iterate,
    Retry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedMultiplicity {
    pub identity: ControlMultiplicityIdentity,
    pub logical_repetitions: af_effect::OccurrenceBounds,
    pub physical_attempts: Option<af_effect::OccurrenceBounds>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryActionUpper {
    Known(OccurrenceCeiling),
    Overflow,
    InvalidMultiplicity,
    Unknown,
}

/// The action upper for one activation of the nearest CompensationScope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalActionBound {
    pub compensation_scope_node: ComputationId,
    pub upper: RecoveryActionUpper,
    pub applied_families: Vec<AppliedMultiplicity>,
}

/// The action upper after the complete checked root path has been folded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootActionBound {
    pub root_semantic_ir_digest: DigestRefValue,
    pub upper: RecoveryActionUpper,
    pub applied_families: Vec<AppliedMultiplicity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RecoveryActionSourceIdentity {
    pub caller_semantic_ir_digest: DigestRefValue,
    pub source_node: ComputationId,
    pub source_contract_digest: DigestRefValue,
    pub forward_entry_ordinal: usize,
    pub action: RecoveryActionIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalRecoveryAction {
    pub source: RecoveryActionSourceIdentity,
    pub forward_seed_upper: OccurrenceCeiling,
    pub owner_scope: ComputationId,
    pub action_kind: RecoveryActionKind,
    pub local_bound: LocalActionBound,
    pub forward_occurrence: ScopedEffectOccurrence,
    pub action_effect_occurrence: ScopedEffectOccurrence,
    pub recovery_usage_binding: RecoveryUsageBinding,
    pub recovery_usage: StaticUsageEnvelope,
    pub guard_compatibility: Option<ActionGuardCompatibility>,
    pub eligibility_evidence: LeafVerdict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootRecoveryOwner {
    CompensationScope(ComputationId),
    RootRecovery,
}

/// Typed path provenance. It records checked ownership/control coordinates;
/// it does not claim a runtime branch, receipt, or action execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlPathStep {
    pub parent_node: ComputationId,
    pub child_node: ComputationId,
    pub child_index: usize,
    pub kind: ControlPathKind,
    pub reachability: ControlPathReachability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlPathKind {
    SequenceChild,
    SelectCase {
        case_ordinal: u32,
        selector: Option<ExpressionId>,
        selector_dependencies: Vec<ValueSlotId>,
        exact_case: Option<u32>,
    },
    ParallelBranch {
        branch_ordinal: u32,
    },
    RaceBranch {
        branch_ordinal: u32,
    },
    IterateBody,
    RetryChild {
        max_attempts: u64,
    },
    TimeoutChild,
    HandleProtected,
    HandleClause {
        ordinal: u32,
    },
    ScopeChild,
    CompensationScopeChild,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootGuardCoordinateEvidence {
    pub source_contract_digest: DigestRefValue,
    pub component_input_port: u32,
    pub input_expression: ExpressionId,
    pub exact_type: Option<af_semantic::TypeSchemaRef>,
    pub dependency_slots: Vec<ValueSlotId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlPathReachability {
    Possible,
    ExactSelected,
    ProvenUnreachable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootRecoveryAction {
    pub source: RecoveryActionSourceIdentity,
    pub forward_seed_upper: OccurrenceCeiling,
    pub target: ResolvedTarget,
    pub owner: RootRecoveryOwner,
    pub local_bound: Option<LocalActionBound>,
    pub root_bound: RootActionBound,
    pub forward_occurrence: ScopedEffectOccurrence,
    pub action_effect_occurrence: ScopedEffectOccurrence,
    pub recovery_usage_binding: RecoveryUsageBinding,
    pub local_recovery_usage: Option<StaticUsageEnvelope>,
    pub root_recovery_usage: StaticUsageEnvelope,
    pub guard_compatibility: Option<ActionGuardCompatibility>,
    pub guard_coordinates: Vec<RootGuardCoordinateEvidence>,
    pub eligibility_evidence: LeafVerdict,
    pub control_path: Vec<ControlPathStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootRecoverySourceClosureReport {
    pub source_nodes_accounted: LeafVerdict,
    pub action_bindings_accounted: LeafVerdict,
    pub usage_profiles_accounted: LeafVerdict,
    pub multiplicity_factors_accounted: LeafVerdict,
    pub root_action_totals_sound: LeafVerdict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootRecoverySummary {
    pub(crate) actions: ControlDimension<Vec<RootRecoveryAction>>,
    pub(crate) action_effects: ControlDimension<af_effect::EffectSignature>,
    pub(crate) epsilon_coverage_requirement: ControlDimension<af_effect::EffectSignature>,
    pub(crate) recovery_usage: ControlDimension<StaticUsageEnvelope>,
    pub(crate) source_closure: RootRecoverySourceClosureReport,
    pub(crate) proof: ControlProofReport,
}

impl RootRecoverySummary {
    #[must_use]
    pub fn actions(&self) -> &ControlDimension<Vec<RootRecoveryAction>> {
        &self.actions
    }
    #[must_use]
    pub fn action_effects(&self) -> &ControlDimension<af_effect::EffectSignature> {
        &self.action_effects
    }
    #[must_use]
    pub fn epsilon_coverage_requirement(&self) -> &ControlDimension<af_effect::EffectSignature> {
        &self.epsilon_coverage_requirement
    }
    #[must_use]
    pub fn recovery_usage(&self) -> &ControlDimension<StaticUsageEnvelope> {
        &self.recovery_usage
    }
    #[must_use]
    pub fn source_closure(&self) -> &RootRecoverySourceClosureReport {
        &self.source_closure
    }
    #[must_use]
    pub fn proof(&self) -> &ControlProofReport {
        &self.proof
    }
    #[must_use]
    pub fn overall(&self) -> LeafVerdict {
        self.proof.overall()
    }
}
