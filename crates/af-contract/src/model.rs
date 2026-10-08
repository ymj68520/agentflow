//! Typed model mirroring the frozen ComponentContract AFIDL module.

use af_effect::{EffectAtom, EffectOperationRef, OccurrenceBounds};
use af_execution::ExecutionRequirements;
use af_failure::FailureRow;
use af_flow::{FlowKind, InformationFlowContract};
use af_recovery::RecoveryContract;
use af_resource::ResourceRequirementRow;
use af_semantic::TypeSchemaRef;
use af_usage::UsageSignature;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentIdentity {
    pub catalog: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentInputPort {
    pub port_no: u32,
    #[serde(rename = "type")]
    pub type_ref: TypeSchemaRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentOutputPort {
    pub port_no: u32,
    #[serde(rename = "type")]
    pub type_ref: TypeSchemaRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectFlowPartition {
    pub operation: EffectOperationRef,
    pub kind: FlowKind,
    pub allowed_effects: Vec<EffectAtom>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryActionKind {
    Reconciliation,
    Compensation,
}

impl RecoveryActionKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reconciliation => "reconciliation",
            Self::Compensation => "compensation",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryActionIdentity {
    pub forward_effect: EffectAtom,
    pub action_kind: RecoveryActionKind,
    pub action_effect: EffectAtom,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryActionBinding {
    pub identity: RecoveryActionIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryUsageBinding {
    pub action: RecoveryActionIdentity,
    pub activations_per_forward: OccurrenceBounds,
    pub attempts_per_activation: OccurrenceBounds,
    pub per_attempt_usage: UsageSignature,
}

/// The closed thirteen-field semantic contract body. Publication and
/// implementation coordinates intentionally do not appear here.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentContract {
    pub component_identity: ComponentIdentity,
    pub inputs: Vec<ComponentInputPort>,
    pub outputs: Vec<ComponentOutputPort>,
    pub failures: FailureRow,
    pub effects: af_effect::EffectSignature,
    pub resources: ResourceRequirementRow,
    pub information_flow: InformationFlowContract,
    pub usage: UsageSignature,
    pub recovery: RecoveryContract,
    pub execution: ExecutionRequirements,
    pub effect_flow_partitions: Vec<EffectFlowPartition>,
    pub recovery_action_bindings: Vec<RecoveryActionBinding>,
    pub recovery_usage_bindings: Vec<RecoveryUsageBinding>,
}

/// Parsing and deserialization target before canonicalization/proof.
pub type RawComponentContract = ComponentContract;
