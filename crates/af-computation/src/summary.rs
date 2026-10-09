//! Derived-only leaf summaries. None of these types is a wire or runtime ABI.

use af_contract::{
    ComponentIdentity, EffectFlowPartition, RecoveryActionBinding, RecoveryUsageBinding,
    RelationCatalogPin, StaticContractProofContext,
};
use af_effect::{EffectEntry, EffectResidual, EffectSignature, OccurrenceBounds};
use af_execution::ExecutionRequirements;
use af_failure::FailureRow;
use af_flow::{FlowConstraint, FlowSourceRef, InformationFlowContract};
use af_ir::ComputationId;
use af_recovery::RecoveryContract;
use af_resource::ResourceRequirementRow;
use af_semantic::{DigestRefValue, ExpressionId, ResourceSlotId, TypeSchemaRef, ValueSlotId};
use af_usage::UsageSignature;

use crate::proof::LeafProofReport;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeafKind {
    Invoke,
    Await,
    CallSubflow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedTarget {
    Component {
        identity: ComponentIdentity,
        contract_digest: DigestRefValue,
    },
    Subflow {
        semantic_ir_digest: DigestRefValue,
        exported_contract_digest: Option<DigestRefValue>,
    },
    AwaitSignal {
        signal_type: TypeSchemaRef,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopedPortBinding {
    InputExpression(ExpressionId),
    OutputValue(ValueSlotId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopedPortMapping {
    pub contract_port_no: u32,
    pub binding: ScopedPortBinding,
    pub exact_type: Option<TypeSchemaRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopedResourcePortMapping {
    pub component_port_no: u32,
    pub resource_slot: ResourceSlotId,
    pub required_type: TypeSchemaRef,
    /// `false` means the caller IR contains no attribute evidence; no
    /// resource identity or authority is thereby inferred.
    pub required_attributes_proven: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceAliasGroup {
    pub resource_slot: ResourceSlotId,
    pub component_ports: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopedEffectOccurrence {
    pub caller_semantic_ir_digest: DigestRefValue,
    pub source_contract_digest: DigestRefValue,
    pub source_component: ComponentIdentity,
    pub computation: ComputationId,
    pub source_entry_ordinal: usize,
    pub declared: EffectEntry,
    /// This is an explicit coordinate translation only. The declared atom
    /// and its full qualifier identity remain unchanged.
    pub mapped_resource_slot: Option<ResourceSlotId>,
    /// Static cardinality scopes introduced by enclosing ITERATE/RETRY nodes.
    /// These are analysis provenance only; they are not invocation or attempt
    /// identifiers and never alter the declared leaf occurrence identity.
    pub control_families: Vec<EffectOccurrenceFamily>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectOccurrenceFamilyKind {
    Iterate,
    Retry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectOccurrenceFamily {
    pub control_node: ComputationId,
    pub kind: EffectOccurrenceFamilyKind,
    /// Conservative logical multiplicity applied by this control node.
    pub logical_repetitions: OccurrenceBounds,
    /// Physical attempts are distinct from the logical effect envelope and
    /// are present only for RETRY.
    pub physical_attempts: Option<OccurrenceBounds>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowSourceBinding {
    InputExpression(ExpressionId),
    ResourceSlot(ResourceSlotId),
    UnknownExternal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowSinkBinding {
    OutputValue(ValueSlotId),
    EffectOccurrences(Vec<usize>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowPortMapping {
    pub source: Option<FlowSourceRef>,
    pub sink: Option<FlowConstraint>,
    pub source_binding: Option<FlowSourceBinding>,
    pub sink_binding: Option<FlowSinkBinding>,
    pub matched_effect_occurrences: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopedEffectResidual {
    pub caller_semantic_ir_digest: DigestRefValue,
    pub source_contract_digest: DigestRefValue,
    pub source_component: ComponentIdentity,
    pub computation: ComputationId,
    pub source_entry_ordinal: usize,
    pub residual: EffectResidual,
    pub mapped_resource_slot: Option<ResourceSlotId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentDimensionSummary {
    pub failures: FailureRow,
    pub effects: EffectSignature,
    pub resources: ResourceRequirementRow,
    pub information_flow: InformationFlowContract,
    pub usage: UsageSignature,
    pub recovery: RecoveryContract,
    pub execution: ExecutionRequirements,
    pub effect_flow_partitions: Vec<EffectFlowPartition>,
    pub recovery_action_bindings: Vec<RecoveryActionBinding>,
    pub recovery_usage_bindings: Vec<RecoveryUsageBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AwaitTimeoutSemantics {
    NoImplicitTimeout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AwaitInteractionStatus {
    ContractUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AwaitProvenanceStatus {
    UnknownExternalInfluence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AwaitAuthorityStatus {
    NotInferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AwaitLeafSummary {
    pub signal_type: TypeSchemaRef,
    pub correlation: ExpressionId,
    pub output_bindings: Vec<(u32, ValueSlotId)>,
    pub timeout: AwaitTimeoutSemantics,
    pub interaction: AwaitInteractionStatus,
    pub provenance: AwaitProvenanceStatus,
    pub authority: AwaitAuthorityStatus,
}

/// Constructor is private: a caller cannot create a trusted checked leaf by
/// populating this data shape. Instances come only from the analyzer.
#[derive(Clone)]
pub struct ProofContextReference<'context, 'universe> {
    context: &'context StaticContractProofContext<'universe>,
    relation_catalog_pins: Vec<RelationCatalogPin>,
}

impl<'context, 'universe> ProofContextReference<'context, 'universe> {
    pub(crate) fn new(context: &'context StaticContractProofContext<'universe>) -> Self {
        Self {
            context,
            relation_catalog_pins: context.relations.pins(),
        }
    }
    #[must_use]
    pub fn context(&self) -> &'context StaticContractProofContext<'universe> {
        self.context
    }
    #[must_use]
    pub fn relation_catalog_pins(&self) -> &[RelationCatalogPin] {
        &self.relation_catalog_pins
    }
    #[must_use]
    pub fn is_same_context(&self, other: &Self) -> bool {
        std::ptr::eq(self.context, other.context)
    }
}

impl std::fmt::Debug for ProofContextReference<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProofContextReference")
            .field("relation_catalog_pins", &self.relation_catalog_pins)
            .finish_non_exhaustive()
    }
}

impl PartialEq for ProofContextReference<'_, '_> {
    fn eq(&self, other: &Self) -> bool {
        self.is_same_context(other) && self.relation_catalog_pins == other.relation_catalog_pins
    }
}
impl Eq for ProofContextReference<'_, '_> {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedComputationLeaf<'context, 'universe> {
    pub(crate) computation: ComputationId,
    pub(crate) caller_semantic_ir_digest: DigestRefValue,
    pub(crate) kind: LeafKind,
    pub(crate) target: Option<ResolvedTarget>,
    pub(crate) input_mappings: Vec<ScopedPortMapping>,
    pub(crate) output_mappings: Vec<ScopedPortMapping>,
    pub(crate) resource_mappings: Vec<ScopedResourcePortMapping>,
    pub(crate) resource_aliases: Vec<ResourceAliasGroup>,
    pub(crate) effect_occurrences: Vec<ScopedEffectOccurrence>,
    pub(crate) effect_residuals: Vec<ScopedEffectResidual>,
    pub(crate) flow_mappings: Vec<FlowPortMapping>,
    pub(crate) dimensions: Option<ComponentDimensionSummary>,
    pub(crate) await_summary: Option<AwaitLeafSummary>,
    pub(crate) proof: LeafProofReport,
    pub(crate) proof_context: ProofContextReference<'context, 'universe>,
}

impl ResolvedComputationLeaf<'_, '_> {
    #[must_use]
    pub fn computation(&self) -> ComputationId {
        self.computation
    }
    #[must_use]
    pub fn caller_semantic_ir_digest(&self) -> &DigestRefValue {
        &self.caller_semantic_ir_digest
    }
    #[must_use]
    pub fn kind(&self) -> LeafKind {
        self.kind
    }
    #[must_use]
    pub fn target(&self) -> Option<&ResolvedTarget> {
        self.target.as_ref()
    }
    #[must_use]
    pub fn input_mappings(&self) -> &[ScopedPortMapping] {
        &self.input_mappings
    }
    #[must_use]
    pub fn output_mappings(&self) -> &[ScopedPortMapping] {
        &self.output_mappings
    }
    #[must_use]
    pub fn resource_mappings(&self) -> &[ScopedResourcePortMapping] {
        &self.resource_mappings
    }
    #[must_use]
    pub fn resource_aliases(&self) -> &[ResourceAliasGroup] {
        &self.resource_aliases
    }
    #[must_use]
    pub fn effect_occurrences(&self) -> &[ScopedEffectOccurrence] {
        &self.effect_occurrences
    }
    #[must_use]
    pub fn effect_residuals(&self) -> &[ScopedEffectResidual] {
        &self.effect_residuals
    }
    #[must_use]
    pub fn flow_mappings(&self) -> &[FlowPortMapping] {
        &self.flow_mappings
    }
    #[must_use]
    pub fn dimensions(&self) -> Option<&ComponentDimensionSummary> {
        self.dimensions.as_ref()
    }
    #[must_use]
    pub fn await_summary(&self) -> Option<&AwaitLeafSummary> {
        self.await_summary.as_ref()
    }
    #[must_use]
    pub fn proof(&self) -> &LeafProofReport {
        &self.proof
    }
    #[must_use]
    pub fn context_relation_pins(&self) -> &[af_contract::RelationCatalogPin] {
        self.proof_context.relation_catalog_pins()
    }
    #[must_use]
    pub fn proof_context_reference(&self) -> &ProofContextReference<'_, '_> {
        &self.proof_context
    }
}
