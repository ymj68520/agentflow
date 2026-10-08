//! Static, bottom-up composition for the Phase 4B.2B.1 control operators.
//!
//! The derived values here are analysis summaries, never a ComponentContract
//! or a durable identity. Operators outside Sequence/Select/Parallel/Race are
//! kept explicit as unsupported; they are not treated as empty.

use std::collections::{BTreeMap, BTreeSet};

use af_contract::{RecoveryActionBinding, RecoveryUsageBinding, StaticContractProofContext};
use af_effect::{
    EffectEntry, EffectOpenRow, EffectResidual, EffectRow, EffectSignature, OccurrenceBounds,
};
use af_execution::ExecutionRequirements;
use af_failure::FailureRow;
use af_flow::{InformationFlowContract, SourceCategory, StaticInformationSummary};
use af_ir::{
    AwaitBody, CallSubflowBody, CheckedIrDocument, ComputationBody, ComputationDef, ComputationId,
    InvokeBody, ParallelMode, ProducerLocation, SemanticDataflowAnalysis, SemanticIrDocument,
};
use af_recovery::RecoveryContract;
use af_resource::ResourceRequirement;
use af_semantic::{
    DigestRefValue, ExpressionBody, ExpressionId, PrimitiveConstant, ResourceSlotId,
    SemanticTypeId, ValueSlotId,
};
use af_usage::StaticUsageEnvelope;

use crate::proof::LeafVerdict;
use crate::summary::{
    FlowPortMapping, LeafKind, ProofContextReference, ResolvedComputationLeaf, ResolvedTarget,
    ScopedEffectOccurrence, ScopedEffectResidual, ScopedPortBinding, ScopedResourcePortMapping,
};

const RULE_CONTROL_BOUNDARY: &str = "AF-SEMANTIC-0124";
const RULE_CONTROL_CONTEXT: &str = "AF-SEMANTIC-0125";
const RULE_UNSUPPORTED_CONTROL: &str = "AF-SEMANTIC-0126";
const RULE_SEQUENCE_EFFECT: &str = "AF-SEMANTIC-0127";
const RULE_SELECT_EFFECT: &str = "AF-SEMANTIC-0128";
const RULE_SELECT_DECISION: &str = "AF-SEMANTIC-0129";
const RULE_PARALLEL_EFFECT_USAGE: &str = "AF-SEMANTIC-0130";
const RULE_PARALLEL_FAIL_FAST: &str = "AF-SEMANTIC-0131";
const RULE_RACE_EFFECT_USAGE: &str = "AF-SEMANTIC-0132";
const RULE_RACE_RECOVERY: &str = "AF-SEMANTIC-0133";
const RULE_FAILURE_PLANE: &str = "AF-SEMANTIC-0134";
const RULE_TYPED_INTERFACE: &str = "AF-SEMANTIC-0135";
const RULE_FLOW_PROPAGATION: &str = "AF-SEMANTIC-0136";
const RULE_USAGE_PROPAGATION: &str = "AF-SEMANTIC-0137";
const RULE_RECOVERY_PROPAGATION: &str = "AF-SEMANTIC-0138";
const RULE_RESOURCE_PROVENANCE: &str = "AF-SEMANTIC-0139";
const RULE_EXECUTION_PER_CHILD: &str = "AF-SEMANTIC-0140";
const RULE_OCCURRENCE_IDENTITY: &str = "AF-SEMANTIC-0141";
const RULE_PROOF_PROPAGATION: &str = "AF-SEMANTIC-0142";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlDerivationError {
    UnknownComputation(ComputationId),
    DuplicateLeafSummary(ComputationId),
    LeafSummaryForNonLeaf(ComputationId),
    LeafContextMismatch(ComputationId),
    CallerDigestMismatch(ComputationId),
    SemanticDigest(String),
    Dataflow(String),
}

impl std::fmt::Display for ControlDerivationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownComputation(id) => write!(f, "unknown computation {id}"),
            Self::DuplicateLeafSummary(id) => {
                write!(f, "multiple resolved leaf summaries name {id}")
            }
            Self::LeafSummaryForNonLeaf(id) => {
                write!(f, "resolved leaf summary names non-leaf computation {id}")
            }
            Self::LeafContextMismatch(id) => write!(
                f,
                "leaf {id} was checked under a different proof-context object"
            ),
            Self::CallerDigestMismatch(id) => {
                write!(f, "leaf {id} belongs to a different SemanticIrDigest")
            }
            Self::SemanticDigest(detail) => {
                write!(f, "cannot compute caller SemanticIrDigest: {detail}")
            }
            Self::Dataflow(detail) => write!(f, "Phase 3C dataflow analysis failed: {detail}"),
        }
    }
}

impl std::error::Error for ControlDerivationError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlOperator {
    Invoke,
    Await,
    CallSubflow,
    Sequence,
    Select,
    Parallel {
        mode: ParallelMode,
        max_inflight: Option<u32>,
    },
    RaceFirstTerminal,
    Unsupported {
        opcode: String,
    },
}

impl ControlOperator {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Invoke => "invoke",
            Self::Await => "await",
            Self::CallSubflow => "call_subflow",
            Self::Sequence => "sequence",
            Self::Select => "select",
            Self::Parallel { .. } => "parallel",
            Self::RaceFirstTerminal => "race",
            Self::Unsupported { .. } => "unsupported",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DimensionContribution<T> {
    source_node: ComputationId,
    value: T,
}

impl<T> DimensionContribution<T> {
    #[must_use]
    pub fn source_node(&self) -> ComputationId {
        self.source_node
    }
    #[must_use]
    pub fn value(&self) -> &T {
        &self.value
    }
}

/// Availability is explicit. An empty known row is a declaration; Unknown,
/// Partial and NotApplicable are different analysis states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlDimensionValue<T> {
    Complete(T),
    Partial {
        known_contribution: Option<T>,
        unresolved_sources: Vec<ComputationId>,
    },
    Unknown {
        known_contributions: Vec<DimensionContribution<T>>,
        unresolved_sources: Vec<ComputationId>,
    },
    PerChild {
        contributions: Vec<DimensionContribution<T>>,
        unresolved_sources: Vec<ComputationId>,
    },
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlDimensionProof {
    dimension: &'static str,
    verdict: LeafVerdict,
    rule_id: &'static str,
    source_node: ComputationId,
    reference: Option<String>,
    reason: String,
    prerequisites: Vec<String>,
}

impl ControlDimensionProof {
    fn new(
        dimension: &'static str,
        verdict: LeafVerdict,
        rule_id: &'static str,
        source_node: ComputationId,
        reference: Option<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            dimension,
            verdict,
            rule_id,
            source_node,
            reference,
            reason: reason.into(),
            prerequisites: Vec::new(),
        }
    }

    #[must_use]
    pub fn dimension(&self) -> &'static str {
        self.dimension
    }
    #[must_use]
    pub fn verdict(&self) -> LeafVerdict {
        self.verdict
    }
    #[must_use]
    pub fn rule_id(&self) -> &'static str {
        self.rule_id
    }
    #[must_use]
    pub fn source_node(&self) -> ComputationId {
        self.source_node
    }
    #[must_use]
    pub fn reference(&self) -> Option<&str> {
        self.reference.as_deref()
    }
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
    #[must_use]
    pub fn prerequisites(&self) -> &[String] {
        &self.prerequisites
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlProofReport {
    proofs: Vec<ControlDimensionProof>,
}

impl ControlProofReport {
    fn new(proofs: Vec<ControlDimensionProof>) -> Self {
        Self { proofs }
    }
    #[must_use]
    pub fn dimensions(&self) -> &[ControlDimensionProof] {
        &self.proofs
    }
    #[must_use]
    pub fn overall(&self) -> LeafVerdict {
        if self
            .proofs
            .iter()
            .any(|proof| proof.verdict == LeafVerdict::Violation)
        {
            LeafVerdict::Violation
        } else if self
            .proofs
            .iter()
            .any(|proof| proof.verdict == LeafVerdict::RequiresEvidence)
        {
            LeafVerdict::RequiresEvidence
        } else if self
            .proofs
            .iter()
            .any(|proof| proof.verdict == LeafVerdict::Unproven)
        {
            LeafVerdict::Unproven
        } else {
            LeafVerdict::Proven
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlOutputSource {
    node: ComputationId,
    port_no: u32,
    value_slot: ValueSlotId,
}

impl ControlOutputSource {
    #[must_use]
    pub fn child_node(&self) -> ComputationId {
        self.node
    }
    #[must_use]
    pub fn child_port_no(&self) -> u32 {
        self.port_no
    }
    #[must_use]
    pub fn child_value_slot(&self) -> ValueSlotId {
        self.value_slot
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlOutputPort {
    parent_port_no: u32,
    parent_value_slot: ValueSlotId,
    local_type_id: SemanticTypeId,
    sources: Vec<ControlOutputSource>,
}

impl ControlOutputPort {
    #[must_use]
    pub fn parent_port_no(&self) -> u32 {
        self.parent_port_no
    }
    #[must_use]
    pub fn parent_value_slot(&self) -> ValueSlotId {
        self.parent_value_slot
    }
    #[must_use]
    pub fn local_type_id(&self) -> SemanticTypeId {
        self.local_type_id
    }
    #[must_use]
    pub fn sources(&self) -> &[ControlOutputSource] {
        &self.sources
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlInterfaceSummary {
    outputs: ControlDimensionValue<Vec<ControlOutputPort>>,
    selector: Option<SelectDecisionEvidence>,
}

impl ControlInterfaceSummary {
    #[must_use]
    pub fn outputs(&self) -> &ControlDimensionValue<Vec<ControlOutputPort>> {
        &self.outputs
    }
    #[must_use]
    pub fn selector(&self) -> Option<&SelectDecisionEvidence> {
        self.selector.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectDecisionEvidence {
    expression: ExpressionId,
    dependency_slots: Vec<ValueSlotId>,
    case_ordinals: Vec<u32>,
    exact_case: Option<u32>,
    totality_verdict: LeafVerdict,
}

impl SelectDecisionEvidence {
    #[must_use]
    pub fn expression(&self) -> ExpressionId {
        self.expression
    }
    #[must_use]
    pub fn dependency_slots(&self) -> &[ValueSlotId] {
        &self.dependency_slots
    }
    #[must_use]
    pub fn case_ordinals(&self) -> &[u32] {
        &self.case_ordinals
    }
    #[must_use]
    pub fn exact_case(&self) -> Option<u32> {
        self.exact_case
    }
    #[must_use]
    pub fn totality_verdict(&self) -> LeafVerdict {
        self.totality_verdict
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceDataflowDependency {
    producer_child: ComputationId,
    producer_node: ComputationId,
    value_slot: ValueSlotId,
    consumer_child: ComputationId,
    consumer_node: ComputationId,
    expression: ExpressionId,
}

impl SequenceDataflowDependency {
    #[must_use]
    pub fn producer_child(&self) -> ComputationId {
        self.producer_child
    }
    #[must_use]
    pub fn producer_node(&self) -> ComputationId {
        self.producer_node
    }
    #[must_use]
    pub fn value_slot(&self) -> ValueSlotId {
        self.value_slot
    }
    #[must_use]
    pub fn consumer_child(&self) -> ComputationId {
        self.consumer_child
    }
    #[must_use]
    pub fn consumer_node(&self) -> ComputationId {
        self.consumer_node
    }
    #[must_use]
    pub fn expression(&self) -> ExpressionId {
        self.expression
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectLowerBoundAdjustmentReason {
    SequenceContinuationCanBeSkipped,
    FailFastSiblingCanBeCancelled,
    RaceBranchMayLoseOrBeCancelled,
    SelectMayHaveNoCoveredCase,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectLowerBoundAdjustment {
    source_node: ComputationId,
    reason: EffectLowerBoundAdjustmentReason,
}

impl EffectLowerBoundAdjustment {
    #[must_use]
    pub fn source_node(&self) -> ComputationId {
        self.source_node
    }
    #[must_use]
    pub fn reason(&self) -> EffectLowerBoundAdjustmentReason {
        self.reason
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlFlowContractSource {
    source_node: ComputationId,
    target: ResolvedTarget,
    contract: InformationFlowContract,
    source_mappings: Vec<FlowPortMapping>,
    effect_flow_partitions: Vec<af_contract::EffectFlowPartition>,
}

impl ControlFlowContractSource {
    #[must_use]
    pub fn source_node(&self) -> ComputationId {
        self.source_node
    }
    #[must_use]
    pub fn target(&self) -> &ResolvedTarget {
        &self.target
    }
    #[must_use]
    pub fn contract(&self) -> &InformationFlowContract {
        &self.contract
    }
    #[must_use]
    pub fn source_mappings(&self) -> &[FlowPortMapping] {
        &self.source_mappings
    }
    #[must_use]
    pub fn effect_flow_partitions(&self) -> &[af_contract::EffectFlowPartition] {
        &self.effect_flow_partitions
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlFlowSummary {
    known_may_influence: Option<StaticInformationSummary>,
    unresolved_children: Vec<ComputationId>,
    sources: Vec<ControlFlowContractSource>,
    sequence_dependencies: Vec<SequenceDataflowDependency>,
    select_decision_dependencies: Vec<ValueSlotId>,
    control_decision_influence_unknown: bool,
    unknown_external_children: Vec<ComputationId>,
}

impl ControlFlowSummary {
    #[must_use]
    pub fn known_may_influence(&self) -> Option<&StaticInformationSummary> {
        self.known_may_influence.as_ref()
    }
    #[must_use]
    pub fn unresolved_children(&self) -> &[ComputationId] {
        &self.unresolved_children
    }
    #[must_use]
    pub fn sources(&self) -> &[ControlFlowContractSource] {
        &self.sources
    }
    #[must_use]
    pub fn sequence_dependencies(&self) -> &[SequenceDataflowDependency] {
        &self.sequence_dependencies
    }
    #[must_use]
    pub fn select_decision_dependencies(&self) -> &[ValueSlotId] {
        &self.select_decision_dependencies
    }
    #[must_use]
    pub fn control_decision_influence_unknown(&self) -> bool {
        self.control_decision_influence_unknown
    }
    #[must_use]
    pub fn unknown_external_children(&self) -> &[ComputationId] {
        &self.unknown_external_children
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourcePortOccurrence {
    source_node: ComputationId,
    target: ResolvedTarget,
    requirement: Option<ResourceRequirement>,
    mapping: ScopedResourcePortMapping,
}

impl ResourcePortOccurrence {
    #[must_use]
    pub fn source_node(&self) -> ComputationId {
        self.source_node
    }
    #[must_use]
    pub fn target(&self) -> &ResolvedTarget {
        &self.target
    }
    #[must_use]
    pub fn requirement(&self) -> Option<&ResourceRequirement> {
        self.requirement.as_ref()
    }
    #[must_use]
    pub fn mapping(&self) -> &ScopedResourcePortMapping {
        &self.mapping
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlResourceAlias {
    resource_slot: ResourceSlotId,
    occurrences: Vec<ResourcePortOccurrence>,
}

impl ControlResourceAlias {
    #[must_use]
    pub fn resource_slot(&self) -> ResourceSlotId {
        self.resource_slot
    }
    #[must_use]
    pub fn occurrences(&self) -> &[ResourcePortOccurrence] {
        &self.occurrences
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoverySourceSummary {
    source_node: ComputationId,
    target: ResolvedTarget,
    contract: RecoveryContract,
    action_bindings: Vec<RecoveryActionBinding>,
    usage_bindings: Vec<RecoveryUsageBinding>,
    effects: Vec<ScopedEffectOccurrence>,
}

impl RecoverySourceSummary {
    #[must_use]
    pub fn source_node(&self) -> ComputationId {
        self.source_node
    }
    #[must_use]
    pub fn target(&self) -> &ResolvedTarget {
        &self.target
    }
    #[must_use]
    pub fn contract(&self) -> &RecoveryContract {
        &self.contract
    }
    #[must_use]
    pub fn action_bindings(&self) -> &[RecoveryActionBinding] {
        &self.action_bindings
    }
    #[must_use]
    pub fn usage_bindings(&self) -> &[RecoveryUsageBinding] {
        &self.usage_bindings
    }
    #[must_use]
    pub fn effects(&self) -> &[ScopedEffectOccurrence] {
        &self.effects
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionSourceSummary {
    source_node: ComputationId,
    target: ResolvedTarget,
    requirements: ExecutionRequirements,
}

impl ExecutionSourceSummary {
    #[must_use]
    pub fn source_node(&self) -> ComputationId {
        self.source_node
    }
    #[must_use]
    pub fn target(&self) -> &ResolvedTarget {
        &self.target
    }
    #[must_use]
    pub fn requirements(&self) -> &ExecutionRequirements {
        &self.requirements
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InheritedControlObligation {
    source_node: ComputationId,
    dimension: &'static str,
    verdict: LeafVerdict,
    rule_id: &'static str,
    reference: Option<String>,
    reason: String,
}

impl InheritedControlObligation {
    #[must_use]
    pub fn source_node(&self) -> ComputationId {
        self.source_node
    }
    #[must_use]
    pub fn dimension(&self) -> &'static str {
        self.dimension
    }
    #[must_use]
    pub fn verdict(&self) -> LeafVerdict {
        self.verdict
    }
    #[must_use]
    pub fn rule_id(&self) -> &'static str {
        self.rule_id
    }
    #[must_use]
    pub fn reference(&self) -> Option<&str> {
        self.reference.as_deref()
    }
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// A non-wire, non-digest view of one control node and its recursively
/// derived child summaries. Private fields prevent callers from assembling a
/// trusted summary by hand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedControlContractSummary<'context, 'universe> {
    source_semantic_ir_digest: DigestRefValue,
    control_node: ComputationId,
    operator: ControlOperator,
    direct_children: Vec<ComputationId>,
    children: Vec<DerivedControlContractSummary<'context, 'universe>>,
    leaf_summary: Option<ResolvedComputationLeaf<'context, 'universe>>,
    interface: ControlInterfaceSummary,
    failures: ControlDimension<FailureRow>,
    effects: ControlDimension<EffectSignature>,
    resources: ControlDimension<Vec<ResourcePortOccurrence>>,
    information_flow: ControlDimension<ControlFlowSummary>,
    usage: ControlDimension<StaticUsageEnvelope>,
    recovery: ControlDimension<Vec<RecoverySourceSummary>>,
    execution: ControlDimension<Vec<ExecutionSourceSummary>>,
    effect_occurrences: Vec<ScopedEffectOccurrence>,
    effect_residuals: Vec<ScopedEffectResidual>,
    resource_aliases: Vec<ControlResourceAlias>,
    effect_lower_bound_adjustments: Vec<EffectLowerBoundAdjustment>,
    sequence_dataflow_dependencies: Vec<SequenceDataflowDependency>,
    inherited_obligations: Vec<InheritedControlObligation>,
    proof: ControlProofReport,
    proof_context: ProofContextReference<'context, 'universe>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlDimension<T> {
    value: ControlDimensionValue<T>,
    proofs: Vec<ControlDimensionProof>,
}

impl<T> ControlDimension<T> {
    #[must_use]
    pub fn value(&self) -> &ControlDimensionValue<T> {
        &self.value
    }
    #[must_use]
    pub fn proofs(&self) -> &[ControlDimensionProof] {
        &self.proofs
    }
}

impl DerivedControlContractSummary<'_, '_> {
    #[must_use]
    pub fn source_semantic_ir_digest(&self) -> &DigestRefValue {
        &self.source_semantic_ir_digest
    }
    #[must_use]
    pub fn control_node(&self) -> ComputationId {
        self.control_node
    }
    #[must_use]
    pub fn operator(&self) -> &ControlOperator {
        &self.operator
    }
    #[must_use]
    pub fn direct_children(&self) -> &[ComputationId] {
        &self.direct_children
    }
    #[must_use]
    pub fn children(&self) -> &[DerivedControlContractSummary<'_, '_>] {
        &self.children
    }
    #[must_use]
    pub fn leaf_summary(&self) -> Option<&ResolvedComputationLeaf<'_, '_>> {
        self.leaf_summary.as_ref()
    }
    #[must_use]
    pub fn interface(&self) -> &ControlInterfaceSummary {
        &self.interface
    }
    #[must_use]
    pub fn failures(&self) -> &ControlDimension<FailureRow> {
        &self.failures
    }
    #[must_use]
    pub fn effects(&self) -> &ControlDimension<EffectSignature> {
        &self.effects
    }
    #[must_use]
    pub fn resources(&self) -> &ControlDimension<Vec<ResourcePortOccurrence>> {
        &self.resources
    }
    #[must_use]
    pub fn information_flow(&self) -> &ControlDimension<ControlFlowSummary> {
        &self.information_flow
    }
    #[must_use]
    pub fn usage(&self) -> &ControlDimension<StaticUsageEnvelope> {
        &self.usage
    }
    #[must_use]
    pub fn recovery(&self) -> &ControlDimension<Vec<RecoverySourceSummary>> {
        &self.recovery
    }
    #[must_use]
    pub fn execution(&self) -> &ControlDimension<Vec<ExecutionSourceSummary>> {
        &self.execution
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
    pub fn resource_aliases(&self) -> &[ControlResourceAlias] {
        &self.resource_aliases
    }
    #[must_use]
    pub fn effect_lower_bound_adjustments(&self) -> &[EffectLowerBoundAdjustment] {
        &self.effect_lower_bound_adjustments
    }
    #[must_use]
    pub fn sequence_dataflow_dependencies(&self) -> &[SequenceDataflowDependency] {
        &self.sequence_dataflow_dependencies
    }
    #[must_use]
    pub fn inherited_obligations(&self) -> &[InheritedControlObligation] {
        &self.inherited_obligations
    }
    #[must_use]
    pub fn proof(&self) -> &ControlProofReport {
        &self.proof
    }
    #[must_use]
    pub fn proof_context_reference(&self) -> &ProofContextReference<'_, '_> {
        &self.proof_context
    }
}

/// Derives one checked IR subtree bottom-up. Leaf summaries are capabilities
/// minted by `resolve_computation_leaf`; every supplied leaf is checked for
/// the current SemanticIrDigest and exact proof-context identity.
pub fn derive_control_contract<'context, 'universe>(
    ir: &CheckedIrDocument<'_>,
    control_node: ComputationId,
    resolved_leaves: &[ResolvedComputationLeaf<'context, 'universe>],
    proof_context: &'context StaticContractProofContext<'universe>,
) -> Result<DerivedControlContractSummary<'context, 'universe>, ControlDerivationError> {
    let document = ir.source();
    if !document
        .computations
        .iter()
        .any(|item| item.id == control_node)
    {
        return Err(ControlDerivationError::UnknownComputation(control_node));
    }
    let digest = semantic_ir_ref(document, proof_context)?;
    let mut leaf_map = BTreeMap::new();
    let mut computation_map = BTreeMap::new();
    for computation in &document.computations {
        computation_map.insert(computation.id, computation);
    }
    for leaf in resolved_leaves {
        if leaf.caller_semantic_ir_digest() != &digest {
            return Err(ControlDerivationError::CallerDigestMismatch(
                leaf.computation(),
            ));
        }
        if !leaf
            .proof_context_reference()
            .is_same_context(&ProofContextReference::new(proof_context))
        {
            return Err(ControlDerivationError::LeafContextMismatch(
                leaf.computation(),
            ));
        }
        let Some(source) = computation_map.get(&leaf.computation()) else {
            return Err(ControlDerivationError::UnknownComputation(
                leaf.computation(),
            ));
        };
        if !matches!(
            source.body,
            ComputationBody::Invoke(_)
                | ComputationBody::Await(_)
                | ComputationBody::CallSubflow(_)
        ) {
            return Err(ControlDerivationError::LeafSummaryForNonLeaf(
                leaf.computation(),
            ));
        }
        if leaf_map.insert(leaf.computation(), leaf).is_some() {
            return Err(ControlDerivationError::DuplicateLeafSummary(
                leaf.computation(),
            ));
        }
    }
    let dataflow = ir
        .analyze_dataflow()
        .map_err(|error| ControlDerivationError::Dataflow(error.to_string()))?;
    derive_node(
        document,
        &computation_map,
        &dataflow,
        &leaf_map,
        &digest,
        proof_context,
        control_node,
    )
}

fn semantic_ir_ref(
    document: &SemanticIrDocument,
    context: &StaticContractProofContext<'_>,
) -> Result<DigestRefValue, ControlDerivationError> {
    let digest = document
        .digest(context.modules)
        .map_err(|error| ControlDerivationError::SemanticDigest(error.to_string()))?;
    Ok(DigestRefValue {
        algorithm: "sha256".into(),
        domain: digest.domain().as_str().into(),
        hex: digest.hex().to_owned(),
    })
}

#[allow(clippy::too_many_arguments)]
fn derive_node<'context, 'universe>(
    document: &SemanticIrDocument,
    computations: &BTreeMap<ComputationId, &ComputationDef>,
    dataflow: &SemanticDataflowAnalysis,
    leaves: &BTreeMap<ComputationId, &ResolvedComputationLeaf<'context, 'universe>>,
    digest: &DigestRefValue,
    context: &'context StaticContractProofContext<'universe>,
    node_id: ComputationId,
) -> Result<DerivedControlContractSummary<'context, 'universe>, ControlDerivationError> {
    let Some(computation) = computations.get(&node_id).copied() else {
        return Err(ControlDerivationError::UnknownComputation(node_id));
    };
    let leaf_kind = match computation.body {
        ComputationBody::Invoke(_) => Some(LeafKind::Invoke),
        ComputationBody::Await(_) => Some(LeafKind::Await),
        ComputationBody::CallSubflow(_) => Some(LeafKind::CallSubflow),
        _ => None,
    };
    if let Some(kind) = leaf_kind {
        return Ok(match leaves.get(&node_id).copied() {
            Some(leaf) => {
                derive_leaf_node(document, dataflow, digest, context, computation, kind, leaf)
            }
            None => derive_missing_leaf(document, dataflow, digest, context, computation, kind),
        });
    }

    let (operator, children_ids, select_body) = match &computation.body {
        ComputationBody::Sequence(body) => (ControlOperator::Sequence, body.children.clone(), None),
        ComputationBody::Select(body) => (
            ControlOperator::Select,
            body.cases.iter().map(|case| case.child).collect(),
            Some(body),
        ),
        ComputationBody::Parallel(body) => (
            ControlOperator::Parallel {
                mode: body.mode,
                max_inflight: body.max_inflight,
            },
            body.branches.iter().map(|branch| branch.child).collect(),
            None,
        ),
        ComputationBody::Race(body) => (
            ControlOperator::RaceFirstTerminal,
            body.branches.iter().map(|branch| branch.child).collect(),
            None,
        ),
        _ => (
            ControlOperator::Unsupported {
                opcode: computation.body.case_name().to_owned(),
            },
            computation.owned_children(),
            None,
        ),
    };
    let mut children = Vec::with_capacity(children_ids.len());
    for child in &children_ids {
        children.push(derive_node(
            document,
            computations,
            dataflow,
            leaves,
            digest,
            context,
            *child,
        )?);
    }

    let (interface, output_proofs) = derive_control_interface(
        document,
        dataflow,
        computation,
        &operator,
        &children,
        select_body,
    );
    let own_proofs = base_proofs(node_id, digest);
    let mut proofs = own_proofs;
    proofs.extend(output_proofs);
    if let ControlOperator::Unsupported { opcode } = &operator {
        proofs.push(ControlDimensionProof::new(
            "unsupported-control", LeafVerdict::Unproven, RULE_UNSUPPORTED_CONTROL, node_id,
            Some(opcode.clone()), "the opcode is structurally checked but this phase has no contract propagation rule for it; it is never replaced by empty rows",
        ));
    }
    let mut inherited = Vec::new();
    inherit_child_proofs(&children, &mut proofs, &mut inherited);

    let mut sequence_dependencies = Vec::new();
    if matches!(operator, ControlOperator::Sequence) {
        sequence_dependencies =
            sequence_dataflow_dependencies(document, computations, dataflow, &children_ids);
    }
    let select_exact_child = interface
        .selector()
        .and_then(SelectDecisionEvidence::exact_case)
        .and_then(|ordinal| {
            select_body.and_then(|body| {
                body.cases
                    .iter()
                    .position(|case| case.ordinal.raw() == ordinal)
            })
        });
    let select_totality = interface.selector().map_or(
        LeafVerdict::Proven,
        SelectDecisionEvidence::totality_verdict,
    );
    let effect_result = derive_effect_dimension(
        node_id,
        &operator,
        &children,
        select_exact_child,
        select_totality,
    );
    proofs.extend(effect_result.proofs.clone());
    let failure_result = derive_failure_dimension(node_id, &operator, &children);
    proofs.extend(failure_result.proofs.clone());
    let usage_result = derive_usage_dimension(node_id, &operator, &children);
    proofs.extend(usage_result.proofs.clone());
    let flow_result = derive_flow_dimension(
        node_id,
        &operator,
        &children,
        &sequence_dependencies,
        select_body.map(|body| body.selector),
        document,
        dataflow,
    );
    proofs.extend(flow_result.proofs.clone());
    let (resources, resource_aliases, resource_proofs) =
        derive_resource_dimension(node_id, &children);
    proofs.extend(resource_proofs);
    let (recovery, recovery_proofs) =
        derive_recovery_dimension(node_id, &operator, &children, &effect_result.occurrences);
    proofs.extend(recovery_proofs);
    let (execution, execution_proofs) = derive_execution_dimension(node_id, &children);
    proofs.extend(execution_proofs);
    proofs.push(ControlDimensionProof::new(
        "control-contract-boundary",
        LeafVerdict::Proven,
        RULE_CONTROL_BOUNDARY,
        node_id,
        Some(format!("computation:{}", node_id.raw())),
        "this result is a derived control summary, not a CheckedComponentContract or digest input",
    ));
    proofs.push(ControlDimensionProof::new(
        "occurrence-identity", LeafVerdict::Proven, RULE_OCCURRENCE_IDENTITY,
        node_id, Some(digest.hex.clone()),
        "effect occurrence lists concatenate in checked child order and retain caller digest, source node, contract digest and source ordinal; equal AtomKeys are not used to deduplicate provenance",
    ));

    Ok(DerivedControlContractSummary {
        source_semantic_ir_digest: digest.clone(),
        control_node: node_id,
        operator,
        direct_children: children_ids,
        children,
        leaf_summary: None,
        interface,
        failures: failure_result,
        effects: effect_result.dimension,
        resources,
        information_flow: flow_result,
        usage: usage_result.dimension,
        recovery,
        execution,
        effect_occurrences: effect_result.occurrences,
        effect_residuals: effect_result.residuals,
        resource_aliases,
        effect_lower_bound_adjustments: effect_result.adjustments,
        sequence_dataflow_dependencies: sequence_dependencies,
        inherited_obligations: inherited,
        proof: ControlProofReport::new(proofs),
        proof_context: ProofContextReference::new(context),
    })
}

fn base_proofs(node: ComputationId, digest: &DigestRefValue) -> Vec<ControlDimensionProof> {
    vec![
        ControlDimensionProof::new(
            "proof-context-consistency",
            LeafVerdict::Proven,
            RULE_CONTROL_CONTEXT,
            node,
            Some(digest.hex.clone()),
            "the subtree is analyzed under the explicit current SemanticIrDigest and proof context",
        ),
        ControlDimensionProof::new(
            "control-topology",
            LeafVerdict::Proven,
            RULE_CONTROL_CONTEXT,
            node,
            Some(format!("computation:{}", node.raw())),
            "the source node comes from a Phase 3B checked ownership tree",
        ),
    ]
}

fn derive_leaf_node<'context, 'universe>(
    document: &SemanticIrDocument,
    dataflow: &SemanticDataflowAnalysis,
    digest: &DigestRefValue,
    context: &'context StaticContractProofContext<'universe>,
    computation: &ComputationDef,
    kind: LeafKind,
    leaf: &ResolvedComputationLeaf<'context, 'universe>,
) -> DerivedControlContractSummary<'context, 'universe> {
    let node = computation.id;
    let operator = match kind {
        LeafKind::Invoke => ControlOperator::Invoke,
        LeafKind::Await => ControlOperator::Await,
        LeafKind::CallSubflow => ControlOperator::CallSubflow,
    };
    let mut proofs = base_proofs(node, digest);
    proofs.extend(leaf.proof().dimensions().iter().map(|proof| {
        ControlDimensionProof::new(
            proof.dimension(),
            proof.verdict(),
            proof.rule_id(),
            node,
            proof.reference().map(str::to_owned),
            proof.reason(),
        )
    }));
    proofs.push(ControlDimensionProof::new(
        "occurrence-identity", LeafVerdict::Proven, RULE_OCCURRENCE_IDENTITY,
        node, Some(digest.hex.clone()),
        "one leaf summary represents one source Semantic IR node; no runtime InvocationId or attempt identity is created",
    ));
    let inherited = inherited_from_leaf(leaf);
    let (interface, output_proofs) = leaf_interface(document, dataflow, computation, leaf);
    proofs.extend(output_proofs);

    let dimensions = leaf.dimensions();
    let failures = dimension_from_leaf(dimensions.map(|value| value.failures.clone()), node);
    let effects = dimension_from_leaf(dimensions.map(|value| value.effects.clone()), node);
    let resources = leaf_resource_occurrences(leaf);
    let resource_state = if dimensions.is_some() {
        ControlDimensionValue::PerChild {
            contributions: vec![DimensionContribution {
                source_node: node,
                value: resources.clone(),
            }],
            unresolved_sources: vec![],
        }
    } else {
        ControlDimensionValue::PerChild {
            contributions: vec![],
            unresolved_sources: vec![node],
        }
    };
    let resource_verdict = source_dimension_verdict(leaf, &["resource-mapping"]);
    let resource_proofs = vec![ControlDimensionProof::new(
        "resource-port-provenance",
        resource_verdict,
        RULE_RESOURCE_PROVENANCE,
        node,
        target_reference(leaf),
        "resource requirement ports remain explicitly mapped to caller ResourceSlotIds; no cross-component port union is performed",
    )];
    proofs.extend(resource_proofs.clone());
    let resource_dimension = ControlDimension {
        value: resource_state,
        proofs: resource_proofs,
    };

    let flow = leaf_flow_summary(leaf);
    let flow_verdict =
        source_dimension_verdict(leaf, &["information-flow", "await-interaction-contract"]);
    let flow_proofs = vec![ControlDimensionProof::new(
        "information-flow",
        flow_verdict,
        RULE_FLOW_PROPAGATION,
        node,
        target_reference(leaf),
        "the complete child F summary and constraints are retained without upgrading unknown provenance or may-carry authority",
    )];
    proofs.extend(flow_proofs.clone());
    let flow_dimension = ControlDimension {
        value: flow.map_or_else(
            || ControlDimensionValue::Unknown {
                known_contributions: vec![],
                unresolved_sources: vec![node],
            },
            ControlDimensionValue::Complete,
        ),
        proofs: flow_proofs,
    };

    let usage = dimensions.and_then(|value| {
        af_usage::declared_usage_envelope(&value.usage, context.usage_meters).ok()
    });
    let usage_verdict = if usage.is_some() {
        source_dimension_verdict(leaf, &["failure-usage-execution"])
    } else {
        LeafVerdict::Unproven
    };
    let usage_proofs = vec![ControlDimensionProof::new(
        "usage",
        usage_verdict,
        RULE_USAGE_PROPAGATION,
        node,
        target_reference(leaf),
        "forward and recovery usage coordinates remain in the explicit (phase, meter) envelope; an absent coordinate is not finite zero",
    )];
    proofs.extend(usage_proofs.clone());
    let usage_dimension = ControlDimension {
        value: usage.map_or_else(
            || ControlDimensionValue::Unknown {
                known_contributions: vec![],
                unresolved_sources: vec![node],
            },
            ControlDimensionValue::Complete,
        ),
        proofs: usage_proofs,
    };

    let (recovery, recovery_proofs) = leaf_recovery(leaf);
    proofs.extend(recovery_proofs.clone());
    let recovery_dimension = ControlDimension {
        value: recovery.map_or_else(
            || ControlDimensionValue::PerChild {
                contributions: vec![],
                unresolved_sources: vec![node],
            },
            |value| ControlDimensionValue::PerChild {
                contributions: vec![DimensionContribution {
                    source_node: node,
                    value: vec![value],
                }],
                unresolved_sources: vec![],
            },
        ),
        proofs: recovery_proofs,
    };
    let (execution, execution_proofs) = leaf_execution(leaf);
    proofs.extend(execution_proofs.clone());
    let execution_dimension = ControlDimension {
        value: execution.map_or_else(
            || ControlDimensionValue::PerChild {
                contributions: vec![],
                unresolved_sources: vec![node],
            },
            |value| ControlDimensionValue::PerChild {
                contributions: vec![DimensionContribution {
                    source_node: node,
                    value: vec![value],
                }],
                unresolved_sources: vec![],
            },
        ),
        proofs: execution_proofs,
    };

    let effect_occurrences = leaf.effect_occurrences().to_vec();
    let effect_residuals = leaf.effect_residuals().to_vec();
    let resource_aliases = leaf
        .resource_aliases()
        .iter()
        .map(|alias| ControlResourceAlias {
            resource_slot: alias.resource_slot,
            occurrences: resources
                .iter()
                .filter(|item| item.mapping.resource_slot == alias.resource_slot)
                .cloned()
                .collect(),
        })
        .collect();
    let failure_proofs = vec![ControlDimensionProof::new(
        "failure-row",
        source_dimension_verdict(leaf, &["failure-usage-execution"]),
        RULE_FAILURE_PLANE,
        node,
        target_reference(leaf),
        "the typed Failure row is copied; an empty row is not a success guarantee, and cancellation/timeout remain outside Phi",
    )];
    proofs.extend(failure_proofs.clone());
    let effect_proofs = vec![ControlDimensionProof::new(
        "effect-envelope",
        source_dimension_verdict(leaf, &["effect-and-recovery"]),
        RULE_SEQUENCE_EFFECT,
        node,
        target_reference(leaf),
        "the declared Effect row and per-call occurrence provenance remain separate",
    )];
    proofs.extend(effect_proofs.clone());
    DerivedControlContractSummary {
        source_semantic_ir_digest: digest.clone(),
        control_node: node,
        operator,
        direct_children: vec![],
        children: vec![],
        leaf_summary: Some(leaf.clone()),
        interface,
        failures: ControlDimension {
            value: failures,
            proofs: failure_proofs,
        },
        effects: ControlDimension {
            value: effects,
            proofs: effect_proofs,
        },
        resources: resource_dimension,
        information_flow: flow_dimension,
        usage: usage_dimension,
        recovery: recovery_dimension,
        execution: execution_dimension,
        effect_occurrences,
        effect_residuals,
        resource_aliases,
        effect_lower_bound_adjustments: vec![],
        sequence_dataflow_dependencies: vec![],
        inherited_obligations: inherited,
        proof: ControlProofReport::new(proofs),
        proof_context: ProofContextReference::new(context),
    }
}

fn derive_missing_leaf<'context, 'universe>(
    document: &SemanticIrDocument,
    dataflow: &SemanticDataflowAnalysis,
    digest: &DigestRefValue,
    context: &'context StaticContractProofContext<'universe>,
    computation: &ComputationDef,
    kind: LeafKind,
) -> DerivedControlContractSummary<'context, 'universe> {
    let node = computation.id;
    let operator = match kind {
        LeafKind::Invoke => ControlOperator::Invoke,
        LeafKind::Await => ControlOperator::Await,
        LeafKind::CallSubflow => ControlOperator::CallSubflow,
    };
    let (interface, interface_proofs) =
        leaf_interface_without_summary(document, dataflow, computation);
    let unresolved = vec![node];
    let mut proofs = base_proofs(node, digest);
    proofs.extend(interface_proofs);
    for dimension in [
        "failure-row",
        "effect-envelope",
        "resource-port-provenance",
        "information-flow",
        "usage",
        "recovery-obligations",
        "execution-per-child",
    ] {
        proofs.push(ControlDimensionProof::new(
            dimension, LeafVerdict::Unproven, RULE_PROOF_PROPAGATION, node,
            Some(format!("computation:{}", node.raw())),
            "the required resolved leaf summary is absent; the missing child is not treated as an empty contract",
        ));
    }
    let inherited_obligations = proofs
        .iter()
        .filter(|proof| proof.verdict() != LeafVerdict::Proven)
        .map(|proof| InheritedControlObligation {
            source_node: proof.source_node(),
            dimension: proof.dimension(),
            verdict: proof.verdict(),
            rule_id: proof.rule_id(),
            reference: proof.reference().map(str::to_owned),
            reason: proof.reason().to_owned(),
        })
        .collect();
    let unknown = |dimension_proof: ControlDimensionProof| ControlDimension::<FailureRow> {
        value: ControlDimensionValue::Unknown {
            known_contributions: vec![],
            unresolved_sources: unresolved.clone(),
        },
        proofs: vec![dimension_proof],
    };
    let unknown_resources = ControlDimension {
        value: ControlDimensionValue::PerChild {
            contributions: vec![],
            unresolved_sources: unresolved.clone(),
        },
        proofs: vec![ControlDimensionProof::new(
            "resource-port-provenance",
            LeafVerdict::Unproven,
            RULE_RESOURCE_PROVENANCE,
            node,
            None,
            "missing resolved leaf resource mappings",
        )],
    };
    let unknown_recovery = ControlDimension {
        value: ControlDimensionValue::PerChild {
            contributions: vec![],
            unresolved_sources: unresolved.clone(),
        },
        proofs: vec![ControlDimensionProof::new(
            "recovery-obligations",
            LeafVerdict::Unproven,
            RULE_RECOVERY_PROPAGATION,
            node,
            None,
            "missing resolved leaf Recovery contract",
        )],
    };
    let unknown_execution = ControlDimension {
        value: ControlDimensionValue::PerChild {
            contributions: vec![],
            unresolved_sources: unresolved.clone(),
        },
        proofs: vec![ControlDimensionProof::new(
            "execution-per-child",
            LeafVerdict::Unproven,
            RULE_EXECUTION_PER_CHILD,
            node,
            None,
            "missing resolved leaf Execution requirements",
        )],
    };
    DerivedControlContractSummary {
        source_semantic_ir_digest: digest.clone(),
        control_node: node,
        operator,
        direct_children: vec![],
        children: vec![],
        leaf_summary: None,
        interface,
        failures: unknown(ControlDimensionProof::new(
            "failure-row",
            LeafVerdict::Unproven,
            RULE_FAILURE_PLANE,
            node,
            None,
            "missing typed Failure row",
        )),
        effects: ControlDimension {
            value: ControlDimensionValue::Unknown {
                known_contributions: vec![],
                unresolved_sources: unresolved.clone(),
            },
            proofs: vec![ControlDimensionProof::new(
                "effect-envelope",
                LeafVerdict::Unproven,
                RULE_SEQUENCE_EFFECT,
                node,
                None,
                "missing Effect envelope is not an empty Closed row",
            )],
        },
        resources: unknown_resources,
        information_flow: ControlDimension {
            value: ControlDimensionValue::Unknown {
                known_contributions: vec![],
                unresolved_sources: unresolved.clone(),
            },
            proofs: vec![ControlDimensionProof::new(
                "information-flow",
                LeafVerdict::Unproven,
                RULE_FLOW_PROPAGATION,
                node,
                None,
                "missing Flow/provenance input is not a clean influence summary",
            )],
        },
        usage: ControlDimension {
            value: ControlDimensionValue::Unknown {
                known_contributions: vec![],
                unresolved_sources: unresolved.clone(),
            },
            proofs: vec![ControlDimensionProof::new(
                "usage",
                LeafVerdict::Unproven,
                RULE_USAGE_PROPAGATION,
                node,
                None,
                "missing Usage is not finite(0)",
            )],
        },
        recovery: unknown_recovery,
        execution: unknown_execution,
        effect_occurrences: vec![],
        effect_residuals: vec![],
        resource_aliases: vec![],
        effect_lower_bound_adjustments: vec![],
        sequence_dataflow_dependencies: vec![],
        inherited_obligations,
        proof: ControlProofReport::new(proofs),
        proof_context: ProofContextReference::new(context),
    }
}

fn dimension_from_leaf<T: Clone>(
    value: Option<T>,
    node: ComputationId,
) -> ControlDimensionValue<T> {
    value.map_or_else(
        || ControlDimensionValue::Unknown {
            known_contributions: vec![],
            unresolved_sources: vec![node],
        },
        ControlDimensionValue::Complete,
    )
}

fn leaf_interface(
    document: &SemanticIrDocument,
    dataflow: &SemanticDataflowAnalysis,
    computation: &ComputationDef,
    leaf: &ResolvedComputationLeaf<'_, '_>,
) -> (ControlInterfaceSummary, Vec<ControlDimensionProof>) {
    let outputs: Vec<ControlOutputPort> = computation.outputs.iter().filter_map(|binding| {
        let slot = document.values.iter().find(|slot| slot.id == binding.value_slot)?;
        let expected_node = computation.id.raw();
        let own_producer = dataflow.producer_locations().get(&binding.value_slot.raw())
            .is_some_and(|producer| matches!(producer, ProducerLocation::ComputationOutput { computation: producer_node, port_no } if *producer_node == expected_node && *port_no == binding.port_no.raw()));
        if !own_producer { return None; }
        Some(ControlOutputPort {
            parent_port_no: binding.port_no.raw(), parent_value_slot: binding.value_slot,
            local_type_id: slot.type_id,
            sources: leaf_output_sources(leaf, binding.port_no.raw(), binding.value_slot),
        })
    }).collect::<Vec<_>>();
    let expected = computation.outputs.len();
    let verdict = if outputs.len() == expected {
        if matches!(leaf.kind(), LeafKind::Await) {
            if expected > 0 {
                LeafVerdict::Unproven
            } else {
                LeafVerdict::Proven
            }
        } else {
            source_dimension_verdict(leaf, &["output-binding"])
        }
    } else {
        LeafVerdict::Violation
    };
    let proofs = vec![ControlDimensionProof::new(
        "typed-output-interface",
        verdict,
        RULE_TYPED_INTERFACE,
        computation.id,
        Some(format!("computation:{}", computation.id.raw())),
        "the output slots are read from checked IR and Phase 3C producer evidence; AWAIT output values lack an adapter result contract",
    )];
    (
        ControlInterfaceSummary {
            outputs: ControlDimensionValue::Complete(outputs),
            selector: None,
        },
        proofs,
    )
}

fn leaf_interface_without_summary(
    document: &SemanticIrDocument,
    _dataflow: &SemanticDataflowAnalysis,
    computation: &ComputationDef,
) -> (ControlInterfaceSummary, Vec<ControlDimensionProof>) {
    let outputs: Vec<ControlOutputPort> = computation
        .outputs
        .iter()
        .filter_map(|binding| {
            let slot = document
                .values
                .iter()
                .find(|slot| slot.id == binding.value_slot)?;
            Some(ControlOutputPort {
                parent_port_no: binding.port_no.raw(),
                parent_value_slot: binding.value_slot,
                local_type_id: slot.type_id,
                sources: vec![],
            })
        })
        .collect();
    (
        ControlInterfaceSummary {
            outputs: ControlDimensionValue::Unknown {
                known_contributions: vec![DimensionContribution {
                    source_node: computation.id,
                    value: outputs,
                }],
                unresolved_sources: vec![computation.id],
            },
            selector: None,
        },
        vec![ControlDimensionProof::new(
            "typed-output-interface",
            LeafVerdict::Unproven,
            RULE_TYPED_INTERFACE,
            computation.id,
            Some(format!("computation:{}", computation.id.raw())),
            "output slots exist, but there is no resolved leaf contract/interface evidence",
        )],
    )
}

fn leaf_output_sources(
    leaf: &ResolvedComputationLeaf<'_, '_>,
    port_no: u32,
    value_slot: ValueSlotId,
) -> Vec<ControlOutputSource> {
    let mapped = leaf.output_mappings().iter().any(|mapping| {
        mapping.contract_port_no == port_no
            && mapping.binding == ScopedPortBinding::OutputValue(value_slot)
    });
    if !mapped || matches!(leaf.kind(), LeafKind::Await) {
        return vec![];
    }
    vec![ControlOutputSource {
        node: leaf.computation(),
        port_no,
        value_slot,
    }]
}

fn source_dimension_verdict(
    leaf: &ResolvedComputationLeaf<'_, '_>,
    dimensions: &[&str],
) -> LeafVerdict {
    let proofs = leaf.proof().dimensions();
    let matches = proofs
        .iter()
        .filter(|proof| dimensions.contains(&proof.dimension()))
        .collect::<Vec<_>>();
    if matches.is_empty() {
        return LeafVerdict::Unproven;
    }
    matches
        .into_iter()
        .fold(LeafVerdict::Proven, |current, proof| {
            merge_verdict(current, proof.verdict())
        })
}

fn inherited_from_leaf(leaf: &ResolvedComputationLeaf<'_, '_>) -> Vec<InheritedControlObligation> {
    leaf.proof()
        .dimensions()
        .iter()
        .filter(|proof| proof.verdict() != LeafVerdict::Proven)
        .map(|proof| InheritedControlObligation {
            source_node: leaf.computation(),
            dimension: proof.dimension(),
            verdict: proof.verdict(),
            rule_id: proof.rule_id(),
            reference: proof.reference().map(str::to_owned),
            reason: proof.reason().to_owned(),
        })
        .collect()
}

fn target_reference(leaf: &ResolvedComputationLeaf<'_, '_>) -> Option<String> {
    match leaf.target()? {
        ResolvedTarget::Component {
            contract_digest, ..
        } => Some(contract_digest.hex.clone()),
        ResolvedTarget::Subflow {
            semantic_ir_digest,
            exported_contract_digest,
        } => Some(exported_contract_digest.as_ref().map_or_else(
            || semantic_ir_digest.hex.clone(),
            |digest| digest.hex.clone(),
        )),
        ResolvedTarget::AwaitSignal { signal_type } => Some(signal_type.to_string()),
    }
}

fn leaf_resource_occurrences(
    leaf: &ResolvedComputationLeaf<'_, '_>,
) -> Vec<ResourcePortOccurrence> {
    let Some(target) = leaf.target().cloned() else {
        return vec![];
    };
    leaf.resource_mappings()
        .iter()
        .cloned()
        .map(|mapping| {
            let requirement = leaf.dimensions().and_then(|dimensions| {
                dimensions
                    .resources
                    .requirements
                    .iter()
                    .find(|requirement| requirement.port_no == mapping.component_port_no)
                    .cloned()
            });
            ResourcePortOccurrence {
                source_node: leaf.computation(),
                target: target.clone(),
                requirement,
                mapping,
            }
        })
        .collect()
}

fn leaf_flow_summary(leaf: &ResolvedComputationLeaf<'_, '_>) -> Option<ControlFlowSummary> {
    let dimensions = leaf.dimensions()?;
    let target = leaf.target()?.clone();
    let sources = vec![ControlFlowContractSource {
        source_node: leaf.computation(),
        target,
        contract: dimensions.information_flow.clone(),
        source_mappings: leaf.flow_mappings().to_vec(),
        effect_flow_partitions: dimensions.effect_flow_partitions.clone(),
    }];
    Some(ControlFlowSummary {
        known_may_influence: Some(dimensions.information_flow.summary.clone()),
        unresolved_children: vec![],
        sources,
        sequence_dependencies: vec![],
        select_decision_dependencies: vec![],
        control_decision_influence_unknown: false,
        unknown_external_children: if dimensions
            .information_flow
            .summary
            .sources
            .iter()
            .any(|source| matches!(source.category, SourceCategory::UnknownExternal))
        {
            vec![leaf.computation()]
        } else {
            vec![]
        },
    })
}

fn leaf_recovery(
    leaf: &ResolvedComputationLeaf<'_, '_>,
) -> (Option<RecoverySourceSummary>, Vec<ControlDimensionProof>) {
    let source = leaf.dimensions().and_then(|dimensions| {
        Some(RecoverySourceSummary {
            source_node: leaf.computation(),
            target: leaf.target()?.clone(),
            contract: dimensions.recovery.clone(),
            action_bindings: dimensions.recovery_action_bindings.clone(),
            usage_bindings: dimensions.recovery_usage_bindings.clone(),
            effects: leaf.effect_occurrences().to_vec(),
        })
    });
    let verdict =
        source_dimension_verdict(leaf, &["effect-and-recovery", "failure-usage-execution"]);
    let proofs = vec![ControlDimensionProof::new(
        "recovery-obligations",
        verdict,
        RULE_RECOVERY_PROPAGATION,
        leaf.computation(),
        target_reference(leaf),
        "the source K contract and exact action/usage bindings are retained per child and are never reduced to winner-only recovery",
    )];
    (source, proofs)
}

fn leaf_execution(
    leaf: &ResolvedComputationLeaf<'_, '_>,
) -> (Option<ExecutionSourceSummary>, Vec<ControlDimensionProof>) {
    let source = leaf.dimensions().and_then(|dimensions| {
        Some(ExecutionSourceSummary {
            source_node: leaf.computation(),
            target: leaf.target()?.clone(),
            requirements: dimensions.execution.clone(),
        })
    });
    let verdict = source_dimension_verdict(leaf, &["failure-usage-execution"]);
    let proofs = vec![ControlDimensionProof::new(
        "execution-per-child",
        verdict,
        RULE_EXECUTION_PER_CHILD,
        leaf.computation(),
        target_reference(leaf),
        "X remains the exact per-child requirement; no single-worker aggregation or admission is inferred",
    )];
    (source, proofs)
}

fn derive_control_interface(
    document: &SemanticIrDocument,
    dataflow: &SemanticDataflowAnalysis,
    computation: &ComputationDef,
    operator: &ControlOperator,
    children: &[DerivedControlContractSummary<'_, '_>],
    select_body: Option<&af_ir::SelectBody>,
) -> (ControlInterfaceSummary, Vec<ControlDimensionProof>) {
    let mut proof = ControlDimensionProof::new(
        "typed-output-interface",
        LeafVerdict::Proven,
        RULE_TYPED_INTERFACE,
        computation.id,
        Some(format!("computation:{}", computation.id.raw())),
        "output ports and producer slots are derived from checked Semantic IR; the Phase 3B interface relation and Phase 3C producer/dataflow pass are reused",
    );
    let mut output_sources = BTreeMap::<u32, Vec<ControlOutputSource>>::new();
    match (&computation.body, operator) {
        (ComputationBody::Sequence(_), ControlOperator::Sequence) => {
            let Some(last) = children.last() else {
                proof = ControlDimensionProof::new(
                    "typed-output-interface",
                    LeafVerdict::Violation,
                    RULE_TYPED_INTERFACE,
                    computation.id,
                    None,
                    "a checked SEQUENCE must own at least one child",
                );
                return (
                    ControlInterfaceSummary {
                        outputs: ControlDimensionValue::Unknown {
                            known_contributions: vec![],
                            unresolved_sources: vec![computation.id],
                        },
                        selector: None,
                    },
                    vec![proof],
                );
            };
            for parent in &computation.outputs {
                if let Some(child_port) = interface_output(last, parent.port_no.raw()) {
                    output_sources
                        .entry(parent.port_no.raw())
                        .or_default()
                        .push(ControlOutputSource {
                            node: last.control_node,
                            port_no: child_port.parent_port_no,
                            value_slot: child_port.parent_value_slot,
                        });
                } else {
                    proof = downgrade(
                        proof,
                        LeafVerdict::Unproven,
                        "last-child output evidence is unavailable",
                    );
                }
            }
        }
        (ComputationBody::Select(body), ControlOperator::Select) => {
            for (case, child) in body.cases.iter().zip(children) {
                for parent in &computation.outputs {
                    if let Some(child_port) = interface_output(child, parent.port_no.raw()) {
                        output_sources
                            .entry(parent.port_no.raw())
                            .or_default()
                            .push(ControlOutputSource {
                                node: child.control_node,
                                port_no: child_port.parent_port_no,
                                value_slot: child_port.parent_value_slot,
                            });
                    } else {
                        proof = downgrade(
                            proof,
                            LeafVerdict::Unproven,
                            "a SELECT case output proof is unavailable",
                        );
                    }
                }
                let _ = case;
            }
        }
        (ComputationBody::Parallel(body), ControlOperator::Parallel { .. }) => {
            for binding in &body.output_bindings {
                let Some(child) = body
                    .branches
                    .iter()
                    .position(|branch| branch.ordinal == binding.branch_ordinal)
                    .and_then(|index| children.get(index))
                else {
                    proof = downgrade(
                        proof,
                        LeafVerdict::Violation,
                        "a checked PARALLEL output binding names no branch",
                    );
                    continue;
                };
                if let Some(child_port) = interface_output(child, binding.child_port_no.raw()) {
                    output_sources
                        .entry(binding.parent_port_no.raw())
                        .or_default()
                        .push(ControlOutputSource {
                            node: child.control_node,
                            port_no: child_port.parent_port_no,
                            value_slot: child_port.parent_value_slot,
                        });
                } else {
                    proof = downgrade(
                        proof,
                        LeafVerdict::Unproven,
                        "a PARALLEL mapped child output is unavailable",
                    );
                }
            }
        }
        (ComputationBody::Race(_), ControlOperator::RaceFirstTerminal) => {
            for child in children {
                for parent in &computation.outputs {
                    if let Some(child_port) = interface_output(child, parent.port_no.raw()) {
                        output_sources
                            .entry(parent.port_no.raw())
                            .or_default()
                            .push(ControlOutputSource {
                                node: child.control_node,
                                port_no: child_port.parent_port_no,
                                value_slot: child_port.parent_value_slot,
                            });
                    } else {
                        proof = downgrade(
                            proof,
                            LeafVerdict::Unproven,
                            "a RACE contestant output proof is unavailable",
                        );
                    }
                }
            }
        }
        _ => {
            proof = downgrade(
                proof,
                LeafVerdict::Unproven,
                "this control operator has no supported output-composition rule",
            );
        }
    }

    let mut ports = Vec::with_capacity(computation.outputs.len());
    for output in &computation.outputs {
        let Some(slot) = document
            .values
            .iter()
            .find(|slot| slot.id == output.value_slot)
        else {
            proof = downgrade(
                proof,
                LeafVerdict::Violation,
                "control output names an undeclared value slot",
            );
            continue;
        };
        let own_producer = dataflow.producer_locations().get(&output.value_slot.raw()).is_some_and(|producer| {
            matches!(producer, ProducerLocation::ComputationOutput { computation: producer_node, port_no } if *producer_node == computation.id.raw() && *port_no == output.port_no.raw())
        });
        if !own_producer {
            proof = downgrade(
                proof,
                LeafVerdict::Violation,
                "control output slot lacks this node's exact single-producer evidence",
            );
        }
        let sources = output_sources
            .remove(&output.port_no.raw())
            .unwrap_or_default();
        for source in &sources {
            if let Some(child) = children
                .iter()
                .find(|child| child.control_node == source.child_node())
            {
                if let Some(child_port) = interface_output(child, source.child_port_no()) {
                    if child_port.local_type_id != slot.type_id {
                        proof = downgrade(
                            proof,
                            LeafVerdict::Violation,
                            "parent and child output local types differ under the checked merge mapping",
                        );
                    }
                }
            }
        }
        if sources.is_empty() && !matches!(operator, ControlOperator::Unsupported { .. }) {
            proof = downgrade(
                proof,
                LeafVerdict::Unproven,
                "no exact child output source was retained for this parent port",
            );
        }
        ports.push(ControlOutputPort {
            parent_port_no: output.port_no.raw(),
            parent_value_slot: output.value_slot,
            local_type_id: slot.type_id,
            sources,
        });
    }
    let selector = select_body.map(|body| select_decision_evidence(document, dataflow, body));
    let output_value = if matches!(operator, ControlOperator::Unsupported { .. }) {
        ControlDimensionValue::Unknown {
            known_contributions: vec![],
            unresolved_sources: vec![computation.id],
        }
    } else {
        ControlDimensionValue::Complete(ports)
    };
    (
        ControlInterfaceSummary {
            outputs: output_value,
            selector,
        },
        vec![proof],
    )
}

fn interface_output<'a>(
    summary: &'a DerivedControlContractSummary<'_, '_>,
    port_no: u32,
) -> Option<&'a ControlOutputPort> {
    match summary.interface.outputs() {
        ControlDimensionValue::Complete(ports) => {
            ports.iter().find(|port| port.parent_port_no == port_no)
        }
        ControlDimensionValue::Partial {
            known_contribution: Some(ports),
            ..
        } => ports.iter().find(|port| port.parent_port_no == port_no),
        _ => None,
    }
}

fn select_decision_evidence(
    document: &SemanticIrDocument,
    dataflow: &SemanticDataflowAnalysis,
    body: &af_ir::SelectBody,
) -> SelectDecisionEvidence {
    let dependency_slots = dataflow
        .expression_value_dependencies()
        .get(&body.selector.raw())
        .map(|slots| {
            slots
                .iter()
                .copied()
                .map(ValueSlotId::from_raw)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let case_ordinals = body
        .cases
        .iter()
        .map(|case| case.ordinal.raw())
        .collect::<Vec<_>>();
    let exact_value = document
        .expressions
        .iter()
        .find(|expression| expression.id == body.selector)
        .and_then(|expression| match &expression.body {
            ExpressionBody::Constant(PrimitiveConstant::Int64(value)) => u32::try_from(*value).ok(),
            _ => None,
        });
    let exact_case = exact_value.filter(|value| case_ordinals.contains(value));
    let totality_verdict = if exact_case.is_some() {
        LeafVerdict::Proven
    } else {
        LeafVerdict::Unproven
    };
    SelectDecisionEvidence {
        expression: body.selector,
        dependency_slots,
        case_ordinals,
        exact_case,
        totality_verdict,
    }
}

fn downgrade(
    proof: ControlDimensionProof,
    verdict: LeafVerdict,
    detail: &str,
) -> ControlDimensionProof {
    let verdict = merge_verdict(proof.verdict, verdict);
    ControlDimensionProof::new(
        proof.dimension,
        verdict,
        proof.rule_id,
        proof.source_node,
        proof.reference,
        format!("{}; {detail}", proof.reason),
    )
}

fn merge_verdict(left: LeafVerdict, right: LeafVerdict) -> LeafVerdict {
    match (left, right) {
        (LeafVerdict::Violation, _) | (_, LeafVerdict::Violation) => LeafVerdict::Violation,
        (LeafVerdict::RequiresEvidence, _) | (_, LeafVerdict::RequiresEvidence) => {
            LeafVerdict::RequiresEvidence
        }
        (LeafVerdict::Unproven, _) | (_, LeafVerdict::Unproven) => LeafVerdict::Unproven,
        _ => LeafVerdict::Proven,
    }
}

fn inherit_child_proofs(
    children: &[DerivedControlContractSummary<'_, '_>],
    parent_proofs: &mut Vec<ControlDimensionProof>,
    obligations: &mut Vec<InheritedControlObligation>,
) {
    for child in children {
        for proof in child.proof.dimensions() {
            if proof.verdict == LeafVerdict::Proven {
                continue;
            }
            let reference = proof.reference.clone();
            let reason = format!(
                "inherited from child computation#{}: {}",
                child.control_node.raw(),
                proof.reason
            );
            parent_proofs.push(ControlDimensionProof::new(
                proof.dimension,
                proof.verdict,
                proof.rule_id,
                proof.source_node,
                reference.clone(),
                reason.clone(),
            ));
            obligations.push(InheritedControlObligation {
                source_node: proof.source_node,
                dimension: proof.dimension,
                verdict: proof.verdict,
                rule_id: proof.rule_id,
                reference,
                reason,
            });
        }
    }
}

struct EffectDimensionResult {
    dimension: ControlDimension<EffectSignature>,
    proofs: Vec<ControlDimensionProof>,
    occurrences: Vec<ScopedEffectOccurrence>,
    residuals: Vec<ScopedEffectResidual>,
    adjustments: Vec<EffectLowerBoundAdjustment>,
}

fn derive_effect_dimension(
    node: ComputationId,
    operator: &ControlOperator,
    children: &[DerivedControlContractSummary<'_, '_>],
    select_exact_child: Option<usize>,
    select_totality: LeafVerdict,
) -> EffectDimensionResult {
    let mut occurrences = Vec::new();
    let mut residuals = Vec::new();
    let mut available = Vec::<(usize, ComputationId, EffectSignature)>::new();
    let mut unresolved = Vec::new();
    for (index, child) in children.iter().enumerate() {
        occurrences.extend(child.effect_occurrences.iter().cloned());
        residuals.extend(child.effect_residuals.iter().cloned());
        match &child.effects.value {
            ControlDimensionValue::Complete(signature) => {
                available.push((index, child.control_node, signature.clone()));
            }
            ControlDimensionValue::Partial {
                unresolved_sources, ..
            }
            | ControlDimensionValue::Unknown {
                unresolved_sources, ..
            }
            | ControlDimensionValue::PerChild {
                unresolved_sources, ..
            } => {
                unresolved.push(child.control_node);
                unresolved.extend(unresolved_sources.iter().copied());
            }
            ControlDimensionValue::NotApplicable => unresolved.push(child.control_node),
        }
    }
    unresolved.sort_unstable();
    unresolved.dedup();
    if matches!(operator, ControlOperator::Unsupported { .. }) {
        unresolved.push(node);
    }

    let mut adjustments = Vec::new();
    let mut normalized = Vec::with_capacity(available.len());
    for (index, source_node, signature) in &available {
        let reason = match operator {
            ControlOperator::Sequence if *index > 0 => {
                Some(EffectLowerBoundAdjustmentReason::SequenceContinuationCanBeSkipped)
            }
            ControlOperator::Parallel {
                mode: ParallelMode::FailFast,
                ..
            } => Some(EffectLowerBoundAdjustmentReason::FailFastSiblingCanBeCancelled),
            ControlOperator::RaceFirstTerminal => {
                Some(EffectLowerBoundAdjustmentReason::RaceBranchMayLoseOrBeCancelled)
            }
            ControlOperator::Select if select_exact_child.is_none() => {
                Some(EffectLowerBoundAdjustmentReason::SelectMayHaveNoCoveredCase)
            }
            _ => None,
        };
        if let Some(reason) = reason {
            if has_positive_effect_lower_bound(signature) {
                adjustments.push(EffectLowerBoundAdjustment {
                    source_node: *source_node,
                    reason,
                });
            }
            normalized.push((*index, *source_node, zero_effect_lower_bounds(signature)));
        } else {
            normalized.push((*index, *source_node, signature.clone()));
        }
    }

    let all_children_known = unresolved.is_empty() && available.len() == children.len();
    let combination = combine_effect_signatures(operator, &normalized, select_exact_child);
    let (value, verdict, reason) = match combination {
        Ok(Some(signature)) if all_children_known => {
            let verdict = if matches!(signature.row, EffectRow::Unknown) {
                LeafVerdict::Unproven
            } else {
                LeafVerdict::Proven
            };
            let reason = match operator {
                ControlOperator::Sequence => {
                    "SEQUENCE uses the existing checked accumulation algebra; every later child's positive lower bounds are weakened because prior success is not proven"
                }
                ControlOperator::Select => {
                    "SELECT uses the existing exclusive branch_join; different guards retain zero lower bounds, and unproven selector coverage forces conservative zero lower bounds"
                }
                ControlOperator::Parallel {
                    mode: ParallelMode::FailFast,
                    ..
                } => {
                    "PARALLEL fail_fast conservatively accumulates every branch upper bound and weakens lower bounds because siblings may be cancelled"
                }
                ControlOperator::Parallel {
                    mode: ParallelMode::All,
                    ..
                } => {
                    "PARALLEL all accumulates every branch; max_inflight is not a total work multiplier"
                }
                ControlOperator::RaceFirstTerminal => {
                    "RACE accumulates every contestant including losers and weakens unguaranteed lower bounds"
                }
                _ => "the selected control algebra is not available",
            };
            (ControlDimensionValue::Complete(signature), verdict, reason)
        }
        Ok(Some(known)) => (
            ControlDimensionValue::Partial {
                known_contribution: Some(known),
                unresolved_sources: unresolved.clone(),
            },
            LeafVerdict::Unproven,
            "known child Effect contributions are retained, but at least one child Effect row is unavailable; the subtotal is not the parent upper envelope",
        ),
        Ok(None) => (
            ControlDimensionValue::Unknown {
                known_contributions: available
                    .iter()
                    .map(|(_, source_node, value)| DimensionContribution {
                        source_node: *source_node,
                        value: value.clone(),
                    })
                    .collect(),
                unresolved_sources: unresolved.clone(),
            },
            LeafVerdict::Unproven,
            "no supported complete Effect composition is available; missing rows remain unknown, not empty",
        ),
        Err(_error) => (
            ControlDimensionValue::Unknown {
                known_contributions: available
                    .iter()
                    .map(|(_, source_node, value)| DimensionContribution {
                        source_node: *source_node,
                        value: value.clone(),
                    })
                    .collect(),
                unresolved_sources: unresolved.clone(),
            },
            LeafVerdict::Violation,
            "the existing Effect algebra rejected the composition (for example, checked occurrence arithmetic overflow)",
        ),
    };
    let mut proofs = vec![ControlDimensionProof::new(
        "effect-envelope",
        verdict,
        effect_rule(operator),
        node,
        Some(format!("computation:{}", node.raw())),
        reason,
    )];
    if let ControlOperator::Select = operator {
        if select_totality != LeafVerdict::Proven {
            proofs.push(ControlDimensionProof::new(
                "select-case-totality", select_totality, RULE_SELECT_DECISION, node,
                Some(format!("computation:{}", node.raw())),
                "the frozen SELECT schema has no range or default field, so exhaustive selector coverage is not inferred",
            ));
        }
    }
    if let ControlOperator::Parallel {
        mode: ParallelMode::FailFast,
        ..
    } = operator
    {
        proofs.push(ControlDimensionProof::new(
            "parallel-fail-fast", LeafVerdict::Proven, RULE_PARALLEL_FAIL_FAST, node,
            Some(format!("computation:{}", node.raw())),
            "fail_fast does not erase any sibling's possible Effect; only unsupported lower guarantees are weakened",
        ));
    }
    if let ControlOperator::RaceFirstTerminal = operator {
        proofs.push(ControlDimensionProof::new(
            "race-loser-effects", verdict, RULE_RACE_EFFECT_USAGE, node,
            Some(format!("computation:{}", node.raw())),
            "all contestant upper bounds remain in the conservative accumulation; cancellation is not evidence of no Effect",
        ));
    }
    let _ = reason;
    EffectDimensionResult {
        dimension: ControlDimension {
            value,
            proofs: proofs.clone(),
        },
        proofs,
        occurrences,
        residuals,
        adjustments,
    }
}

fn effect_rule(operator: &ControlOperator) -> &'static str {
    match operator {
        ControlOperator::Sequence => RULE_SEQUENCE_EFFECT,
        ControlOperator::Select => RULE_SELECT_EFFECT,
        ControlOperator::Parallel { .. } => RULE_PARALLEL_EFFECT_USAGE,
        ControlOperator::RaceFirstTerminal => RULE_RACE_EFFECT_USAGE,
        _ => RULE_UNSUPPORTED_CONTROL,
    }
}

fn combine_effect_signatures(
    operator: &ControlOperator,
    signatures: &[(usize, ComputationId, EffectSignature)],
    select_exact_child: Option<usize>,
) -> Result<Option<EffectSignature>, af_effect::EffectError> {
    if signatures.is_empty() || matches!(operator, ControlOperator::Unsupported { .. }) {
        return Ok(None);
    }
    match operator {
        ControlOperator::Sequence => {
            let mut ordered = signatures.to_vec();
            ordered.sort_by_key(|(index, _, _)| *index);
            let Some((_, _, first)) = ordered.first() else {
                return Ok(None);
            };
            let mut result = first.clone();
            for (index, _, next) in ordered.iter().skip(1) {
                let next = if *index > 0 {
                    zero_effect_lower_bounds(next)
                } else {
                    next.clone()
                };
                result = af_effect::control::sequence_envelope(&result, &next)?;
            }
            Ok(Some(result))
        }
        ControlOperator::Select => {
            if let Some(selected_child) = select_exact_child {
                return Ok(signatures
                    .iter()
                    .find(|(index, _, _)| *index == selected_child)
                    .map(|(_, _, signature)| signature.clone()));
            }
            let mut ordered = signatures.to_vec();
            ordered.sort_by_key(|(index, _, _)| *index);
            let mut iter = ordered.into_iter();
            let Some((_, _, first)) = iter.next() else {
                return Ok(None);
            };
            let mut result = first;
            for (_, _, next) in iter {
                result = af_effect::control::select_envelope(&result, &next)?;
            }
            // The current IR has no default/range field. A dynamic selector
            // may name no listed ordinal; the envelope's upper remains safe,
            // but no positive occurrence lower can be unconditional.
            Ok(Some(zero_effect_lower_bounds(&result)))
        }
        ControlOperator::Parallel { mode, .. } => {
            let mut ordered = signatures.to_vec();
            ordered.sort_by_key(|(index, _, _)| *index);
            let mut iter = ordered.into_iter();
            let Some((_, _, first)) = iter.next() else {
                return Ok(None);
            };
            let first = if *mode == ParallelMode::FailFast {
                zero_effect_lower_bounds(&first)
            } else {
                first
            };
            let mut result = first;
            for (_, _, next) in iter {
                let next = if *mode == ParallelMode::FailFast {
                    zero_effect_lower_bounds(&next)
                } else {
                    next
                };
                result = af_effect::control::parallel_envelope(&result, &next)?;
            }
            Ok(Some(result))
        }
        ControlOperator::RaceFirstTerminal => {
            let ordered = signatures
                .iter()
                .map(|(_, _, signature)| zero_effect_lower_bounds(signature))
                .collect::<Vec<_>>();
            Ok(Some(af_effect::control::race_envelope(&ordered)?))
        }
        _ => Ok(None),
    }
}

fn has_positive_effect_lower_bound(signature: &EffectSignature) -> bool {
    match &signature.row {
        EffectRow::Closed(entries) => entries.iter().any(|entry| entry.bounds.lower > 0),
        EffectRow::Open(open) => {
            open.known.iter().any(|entry| entry.bounds.lower > 0)
                || open.residual.iter().any(|entry| entry.bounds.lower > 0)
        }
        EffectRow::Unknown => false,
    }
}

fn zero_effect_lower_bounds(signature: &EffectSignature) -> EffectSignature {
    let zero_entry = |entry: &EffectEntry| EffectEntry {
        atom: entry.atom.clone(),
        guard: entry.guard.clone(),
        bounds: OccurrenceBounds {
            lower: 0,
            upper: entry.bounds.upper,
        },
    };
    let row = match &signature.row {
        EffectRow::Closed(entries) => EffectRow::Closed(entries.iter().map(zero_entry).collect()),
        EffectRow::Open(open) => EffectRow::Open(EffectOpenRow {
            known: open.known.iter().map(zero_entry).collect(),
            residual: open
                .residual
                .iter()
                .map(|entry| EffectResidual {
                    catalog: entry.catalog.clone(),
                    selector: entry.selector.clone(),
                    bounds: OccurrenceBounds {
                        lower: 0,
                        upper: entry.bounds.upper,
                    },
                })
                .collect(),
        }),
        EffectRow::Unknown => EffectRow::Unknown,
    };
    EffectSignature { row }
}

fn derive_failure_dimension(
    node: ComputationId,
    operator: &ControlOperator,
    children: &[DerivedControlContractSummary<'_, '_>],
) -> ControlDimension<FailureRow> {
    let mut rows = Vec::<(usize, ComputationId, FailureRow)>::new();
    let mut unresolved = Vec::new();
    for (index, child) in children.iter().enumerate() {
        match &child.failures.value {
            ControlDimensionValue::Complete(row) => {
                rows.push((index, child.control_node, row.clone()));
            }
            ControlDimensionValue::Partial {
                unresolved_sources, ..
            }
            | ControlDimensionValue::Unknown {
                unresolved_sources, ..
            }
            | ControlDimensionValue::PerChild {
                unresolved_sources, ..
            } => {
                unresolved.push(child.control_node);
                unresolved.extend(unresolved_sources.iter().copied());
            }
            ControlDimensionValue::NotApplicable => unresolved.push(child.control_node),
        }
    }
    unresolved.sort_unstable();
    unresolved.dedup();
    let known_rows = rows
        .iter()
        .map(|(_, _, row)| row.clone())
        .collect::<Vec<_>>();
    let composed = compose_failure_rows(operator, &known_rows);
    let (value, verdict, reason) = if matches!(operator, ControlOperator::Unsupported { .. }) {
        (
            ControlDimensionValue::Unknown {
                known_contributions: rows
                    .iter()
                    .map(|(_, source_node, value)| DimensionContribution {
                        source_node: *source_node,
                        value: value.clone(),
                    })
                    .collect(),
                unresolved_sources: vec![node],
            },
            LeafVerdict::Unproven,
            "the operator has no supported Failure propagation rule; known child rows remain available below",
        )
    } else if unresolved.is_empty() && rows.len() == children.len() {
        (
            ControlDimensionValue::Complete(composed.unwrap_or_else(FailureRow::empty)),
            LeafVerdict::Proven,
            "the existing Failure control algebra conservatively unions all child rows that may contribute; an empty row does not guarantee success",
        )
    } else if let Some(known) = composed {
        (
            ControlDimensionValue::Partial {
                known_contribution: Some(known),
                unresolved_sources: unresolved.clone(),
            },
            LeafVerdict::Unproven,
            "known child Failure types are retained, but at least one child row is unresolved and was not treated as empty",
        )
    } else {
        (
            ControlDimensionValue::Unknown {
                known_contributions: rows
                    .iter()
                    .map(|(_, source_node, value)| DimensionContribution {
                        source_node: *source_node,
                        value: value.clone(),
                    })
                    .collect(),
                unresolved_sources: unresolved.clone(),
            },
            LeafVerdict::Unproven,
            "no complete child Failure rows are available; no empty Failure row is synthesized",
        )
    };
    let proof = ControlDimensionProof::new(
        "failure-row",
        verdict,
        RULE_FAILURE_PLANE,
        node,
        Some(format!("computation:{}", node.raw())),
        reason,
    );
    ControlDimension {
        value,
        proofs: vec![proof],
    }
}

fn compose_failure_rows(operator: &ControlOperator, rows: &[FailureRow]) -> Option<FailureRow> {
    match operator {
        ControlOperator::Sequence => Some(af_failure::control::sequence_row(rows)),
        ControlOperator::Select => Some(af_failure::control::select_row(rows)),
        ControlOperator::Parallel { .. } => Some(af_failure::control::parallel_row(rows)),
        ControlOperator::RaceFirstTerminal => Some(af_failure::control::race_row(rows)),
        _ => None,
    }
}

struct UsageDimensionResult {
    dimension: ControlDimension<StaticUsageEnvelope>,
    proofs: Vec<ControlDimensionProof>,
}

fn derive_usage_dimension(
    node: ComputationId,
    operator: &ControlOperator,
    children: &[DerivedControlContractSummary<'_, '_>],
) -> UsageDimensionResult {
    let mut envelopes = Vec::<(usize, ComputationId, StaticUsageEnvelope)>::new();
    let mut unresolved = Vec::new();
    for (index, child) in children.iter().enumerate() {
        match &child.usage.value {
            ControlDimensionValue::Complete(envelope) => {
                envelopes.push((index, child.control_node, envelope.clone()));
            }
            ControlDimensionValue::Partial {
                unresolved_sources, ..
            }
            | ControlDimensionValue::Unknown {
                unresolved_sources, ..
            }
            | ControlDimensionValue::PerChild {
                unresolved_sources, ..
            } => {
                unresolved.push(child.control_node);
                unresolved.extend(unresolved_sources.iter().copied());
            }
            ControlDimensionValue::NotApplicable => unresolved.push(child.control_node),
        }
    }
    unresolved.sort_unstable();
    unresolved.dedup();
    if matches!(operator, ControlOperator::Unsupported { .. }) {
        unresolved.push(node);
    }
    let composed = compose_usage(operator, &envelopes);
    let (value, mut verdict, reason) = match composed {
        Ok(Some(envelope)) if unresolved.is_empty() && envelopes.len() == children.len() => {
            let no_coordinates = envelope.is_empty();
            let unknown_bound = envelope.is_globally_unknown()
                || envelope
                    .iter()
                    .any(|(_, bound)| matches!(bound, af_usage::StaticUsageBound::Unknown));
            let status = if no_coordinates || unknown_bound {
                LeafVerdict::Unproven
            } else {
                LeafVerdict::Proven
            };
            (
                ControlDimensionValue::Complete(envelope),
                status,
                "Usage combines by checked accumulation for SEQUENCE/PARALLEL/RACE and exclusive maximum join for SELECT; meters retain (phase, identity), and max_inflight never scales totals",
            )
        }
        Ok(Some(known_contribution)) => (
            ControlDimensionValue::Partial {
                known_contribution: Some(known_contribution),
                unresolved_sources: unresolved.clone(),
            },
            LeafVerdict::Unproven,
            "known child Usage contributions are retained, but an unresolved child/meter is not treated as zero",
        ),
        Ok(None) => (
            ControlDimensionValue::Unknown {
                known_contributions: envelopes
                    .iter()
                    .map(|(_, source_node, value)| DimensionContribution {
                        source_node: *source_node,
                        value: value.clone(),
                    })
                    .collect(),
                unresolved_sources: unresolved.clone(),
            },
            LeafVerdict::Unproven,
            "no supported Usage algebra is available for this node; child values are retained and no empty envelope is substituted",
        ),
        Err(_) => (
            ControlDimensionValue::Unknown {
                known_contributions: envelopes
                    .iter()
                    .map(|(_, source_node, value)| DimensionContribution {
                        source_node: *source_node,
                        value: value.clone(),
                    })
                    .collect(),
                unresolved_sources: unresolved.clone(),
            },
            LeafVerdict::Violation,
            "the existing Usage algebra rejected checked arithmetic; overflow never wraps or saturates",
        ),
    };
    if matches!(operator, ControlOperator::Unsupported { .. }) {
        verdict = LeafVerdict::Unproven;
    }
    let proof = ControlDimensionProof::new(
        "usage",
        verdict,
        RULE_USAGE_PROPAGATION,
        node,
        Some(format!("computation:{}", node.raw())),
        reason,
    );
    let proofs = vec![proof];
    UsageDimensionResult {
        dimension: ControlDimension {
            value,
            proofs: proofs.clone(),
        },
        proofs,
    }
}

fn compose_usage(
    operator: &ControlOperator,
    envelopes: &[(usize, ComputationId, StaticUsageEnvelope)],
) -> Result<Option<StaticUsageEnvelope>, af_usage::UsageError> {
    if envelopes.is_empty() || matches!(operator, ControlOperator::Unsupported { .. }) {
        return Ok(None);
    }
    let mut ordered = envelopes.to_vec();
    ordered.sort_by_key(|(index, _, _)| *index);
    let values = ordered
        .iter()
        .map(|(_, _, value)| value.clone())
        .collect::<Vec<_>>();
    match operator {
        ControlOperator::Sequence => {
            let Some((_, _, first)) = ordered.first() else {
                return Ok(None);
            };
            let mut result = first.clone();
            for (_, _, next) in ordered.iter().skip(1) {
                result = af_usage::control::sequence_envelope(&result, next)?;
            }
            Ok(Some(result))
        }
        ControlOperator::Select => Ok(Some(af_usage::control::select_envelope(&values)?)),
        ControlOperator::Parallel { .. } => {
            Ok(Some(af_usage::control::parallel_envelope(&values)?))
        }
        ControlOperator::RaceFirstTerminal => Ok(Some(af_usage::control::race_envelope(&values)?)),
        _ => Ok(None),
    }
}

fn sequence_dataflow_dependencies(
    _document: &SemanticIrDocument,
    computations: &BTreeMap<ComputationId, &ComputationDef>,
    dataflow: &SemanticDataflowAnalysis,
    children: &[ComputationId],
) -> Vec<SequenceDataflowDependency> {
    let subtrees = children
        .iter()
        .map(|child| subtree_nodes(computations, *child))
        .collect::<Vec<_>>();
    let mut dependencies = BTreeSet::new();
    for consumer_index in 1..children.len() {
        let mut uses = Vec::new();
        for consumer_node in &subtrees[consumer_index] {
            if let Some(computation) = computations.get(consumer_node) {
                uses.extend(expression_uses(&computation.body));
            }
        }
        uses.sort_unstable();
        uses.dedup();
        for expression in uses {
            let Some(slots) = dataflow
                .expression_value_dependencies()
                .get(&expression.raw())
            else {
                continue;
            };
            let consumer_nodes = subtrees[consumer_index]
                .iter()
                .copied()
                .filter(|candidate| {
                    computations.get(candidate).is_some_and(|computation| {
                        expression_uses(&computation.body).contains(&expression)
                    })
                })
                .collect::<Vec<_>>();
            for consumer_node in consumer_nodes {
                for raw_slot in slots {
                    let Some(ProducerLocation::ComputationOutput {
                        computation: producer_node,
                        ..
                    }) = dataflow.producer_locations().get(raw_slot)
                    else {
                        continue;
                    };
                    let Some(producer_index) =
                        subtrees[..consumer_index].iter().position(|subtree| {
                            subtree.contains(&ComputationId::from_raw(*producer_node))
                        })
                    else {
                        continue;
                    };
                    let entry_contains = dataflow
                        .entry_values()
                        .get(&consumer_node.raw())
                        .is_some_and(|values| values.contains(raw_slot));
                    if !entry_contains {
                        continue;
                    }
                    dependencies.insert((
                        consumer_index,
                        expression.raw(),
                        *raw_slot,
                        producer_index,
                        *producer_node,
                        consumer_node.raw(),
                    ));
                }
            }
        }
    }
    dependencies
        .into_iter()
        .map(
            |(consumer_index, expression, slot, producer_index, producer_node, consumer_node)| {
                SequenceDataflowDependency {
                    producer_child: children[producer_index],
                    producer_node: ComputationId::from_raw(producer_node),
                    value_slot: ValueSlotId::from_raw(slot),
                    consumer_child: children[consumer_index],
                    consumer_node: ComputationId::from_raw(consumer_node),
                    expression: ExpressionId::from_raw(expression),
                }
            },
        )
        .collect()
}

fn subtree_nodes(
    computations: &BTreeMap<ComputationId, &ComputationDef>,
    root: ComputationId,
) -> BTreeSet<ComputationId> {
    fn visit(
        computations: &BTreeMap<ComputationId, &ComputationDef>,
        node: ComputationId,
        out: &mut BTreeSet<ComputationId>,
    ) {
        if !out.insert(node) {
            return;
        }
        if let Some(computation) = computations.get(&node) {
            for child in computation.owned_children() {
                visit(computations, child, out);
            }
        }
    }
    let mut result = BTreeSet::new();
    visit(computations, root, &mut result);
    result
}

fn expression_uses(body: &ComputationBody) -> Vec<ExpressionId> {
    match body {
        ComputationBody::Invoke(InvokeBody { inputs, .. })
        | ComputationBody::CallSubflow(CallSubflowBody { inputs, .. }) => {
            inputs.iter().map(|input| input.expression).collect()
        }
        ComputationBody::Await(AwaitBody { correlation, .. }) => vec![*correlation],
        ComputationBody::Select(body) => vec![body.selector],
        ComputationBody::Iterate(af_ir::IterateBody::Foreach(body)) => std::iter::once(body.source)
            .chain(body.carry.iter().map(|carry| carry.initial))
            .collect(),
        ComputationBody::Iterate(af_ir::IterateBody::While(body)) => std::iter::once(body.guard)
            .chain(body.carry.iter().map(|carry| carry.initial))
            .collect(),
        ComputationBody::Timeout(body) => vec![body.timeout],
        _ => vec![],
    }
}

fn derive_flow_dimension(
    node: ComputationId,
    operator: &ControlOperator,
    children: &[DerivedControlContractSummary<'_, '_>],
    sequence_dependencies: &[SequenceDataflowDependency],
    select_selector: Option<ExpressionId>,
    document: &SemanticIrDocument,
    dataflow: &SemanticDataflowAnalysis,
) -> ControlDimension<ControlFlowSummary> {
    let mut known_summaries = Vec::new();
    let mut unresolved = Vec::new();
    let mut sources = Vec::new();
    let mut child_sequence_edges = Vec::new();
    let mut selector_edges = Vec::new();
    let mut unknown_external_children = Vec::new();
    let mut control_decision_unknown = matches!(operator, ControlOperator::RaceFirstTerminal)
        || matches!(
            operator,
            ControlOperator::Parallel {
                mode: ParallelMode::FailFast,
                ..
            }
        );
    let mut child_has_unknown = matches!(operator, ControlOperator::Unsupported { .. });
    if matches!(operator, ControlOperator::Unsupported { .. }) {
        unresolved.push(node);
    }
    for child in children {
        if child
            .leaf_summary
            .as_ref()
            .and_then(ResolvedComputationLeaf::await_summary)
            .is_some_and(|await_summary| {
                await_summary.provenance
                    == crate::summary::AwaitProvenanceStatus::UnknownExternalInfluence
            })
        {
            unknown_external_children.push(child.control_node);
        }
        match &child.information_flow.value {
            ControlDimensionValue::Complete(flow) => {
                if let Some(summary) = &flow.known_may_influence {
                    known_summaries.push(summary);
                }
                sources.extend(flow.sources.iter().cloned());
                unknown_external_children.extend(flow.unknown_external_children.iter().copied());
                child_sequence_edges.extend(flow.sequence_dependencies.iter().cloned());
                selector_edges.extend(flow.select_decision_dependencies.iter().copied());
                control_decision_unknown |= flow.control_decision_influence_unknown;
                if !flow.unresolved_children.is_empty() {
                    child_has_unknown = true;
                    unresolved.extend(flow.unresolved_children.iter().copied());
                }
            }
            ControlDimensionValue::Partial {
                known_contribution,
                unresolved_sources,
            } => {
                if let Some(flow) = known_contribution {
                    if let Some(summary) = &flow.known_may_influence {
                        known_summaries.push(summary);
                    }
                    sources.extend(flow.sources.iter().cloned());
                    unknown_external_children
                        .extend(flow.unknown_external_children.iter().copied());
                    child_sequence_edges.extend(flow.sequence_dependencies.iter().cloned());
                    selector_edges.extend(flow.select_decision_dependencies.iter().copied());
                    control_decision_unknown |= flow.control_decision_influence_unknown;
                }
                child_has_unknown = true;
                unresolved.push(child.control_node);
                unresolved.extend(unresolved_sources.iter().copied());
            }
            ControlDimensionValue::Unknown {
                known_contributions,
                unresolved_sources,
            } => {
                for contribution in known_contributions {
                    if let Some(flow) = &contribution.value.known_may_influence {
                        known_summaries.push(flow);
                    }
                    sources.extend(contribution.value.sources.iter().cloned());
                    unknown_external_children
                        .extend(contribution.value.unknown_external_children.iter().copied());
                    child_sequence_edges
                        .extend(contribution.value.sequence_dependencies.iter().cloned());
                    selector_edges.extend(
                        contribution
                            .value
                            .select_decision_dependencies
                            .iter()
                            .copied(),
                    );
                    control_decision_unknown |=
                        contribution.value.control_decision_influence_unknown;
                }
                child_has_unknown = true;
                unresolved.push(child.control_node);
                unresolved.extend(unresolved_sources.iter().copied());
            }
            ControlDimensionValue::PerChild {
                unresolved_sources, ..
            } => {
                child_has_unknown = true;
                unresolved.push(child.control_node);
                unresolved.extend(unresolved_sources.iter().copied());
            }
            ControlDimensionValue::NotApplicable => {
                child_has_unknown = true;
                unresolved.push(child.control_node);
            }
        }
    }
    unresolved.sort_unstable();
    unresolved.dedup();
    unknown_external_children.sort_unstable();
    unknown_external_children.dedup();
    let has_unknown_external = !unknown_external_children.is_empty();
    unresolved.extend(unknown_external_children.iter().copied());
    unresolved.sort_unstable();
    unresolved.dedup();
    if matches!(operator, ControlOperator::Unsupported { .. }) {
        unresolved.push(node);
    }
    child_sequence_edges.extend(sequence_dependencies.iter().cloned());
    let select_decision_dependencies = if matches!(operator, ControlOperator::Select) {
        select_selector
            .and_then(|expression| {
                dataflow
                    .expression_value_dependencies()
                    .get(&expression.raw())
            })
            .map(|slots| {
                slots
                    .iter()
                    .copied()
                    .map(ValueSlotId::from_raw)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    } else {
        vec![]
    };
    selector_edges.extend(select_decision_dependencies.iter().copied());
    selector_edges.sort_unstable();
    selector_edges.dedup();
    let mut merged = if known_summaries.is_empty() {
        None
    } else {
        Some(af_flow::merge_provenance_summaries(&known_summaries))
    };
    let select_dynamic =
        matches!(operator, ControlOperator::Select) && !select_decision_dependencies.is_empty();
    let sequence_flow_unknown = !sequence_dependencies.is_empty();
    let decision_unknown =
        select_dynamic || control_decision_unknown || sequence_flow_unknown || child_has_unknown;
    if decision_unknown {
        if let Some(summary) = &mut merged {
            summary.influence_unknown = true;
        }
    }
    let flow = ControlFlowSummary {
        known_may_influence: merged,
        unresolved_children: unresolved.clone(),
        sources,
        sequence_dependencies: child_sequence_edges,
        select_decision_dependencies: selector_edges,
        control_decision_influence_unknown: control_decision_unknown,
        unknown_external_children,
    };
    let verdict = if child_has_unknown
        || decision_unknown
        || has_unknown_external
        || flow
            .known_may_influence
            .as_ref()
            .is_some_and(|summary| summary.influence_unknown)
    {
        LeafVerdict::Unproven
    } else {
        LeafVerdict::Proven
    };
    let reason = if select_dynamic {
        "SELECT data dependencies may influence which branch is chosen; source/taint/authority from value slots is not a Phase 3C output and is retained as unknown"
    } else if sequence_flow_unknown {
        "Phase 3C producer-to-use edges are retained and child may-influence facts are unioned, but this model has no per-value taint transfer proof"
    } else if control_decision_unknown {
        "PARALLEL fail-fast cancellation/terminal cause or RACE first-terminal winner may influence outcomes; all child F summaries remain and no winner/primary cause is selected"
    } else {
        "child may-influence summaries are conservatively merged; per-child Flow constraints and EffectFlowPartitions remain source-scoped"
    };
    let proof = ControlDimensionProof::new(
        "information-flow",
        verdict,
        RULE_FLOW_PROPAGATION,
        node,
        Some(format!("computation:{}", node.raw())),
        reason,
    );
    let value = if child_has_unknown
        || select_dynamic
        || sequence_flow_unknown
        || control_decision_unknown
        || has_unknown_external
    {
        ControlDimensionValue::Partial {
            known_contribution: Some(flow),
            unresolved_sources: unresolved,
        }
    } else {
        ControlDimensionValue::Complete(flow)
    };
    let _ = document;
    ControlDimension {
        value,
        proofs: vec![proof],
    }
}

fn derive_resource_dimension(
    node: ComputationId,
    children: &[DerivedControlContractSummary<'_, '_>],
) -> (
    ControlDimension<Vec<ResourcePortOccurrence>>,
    Vec<ControlResourceAlias>,
    Vec<ControlDimensionProof>,
) {
    let mut contributions = Vec::new();
    let mut unresolved = Vec::new();
    for child in children {
        match &child.resources.value {
            ControlDimensionValue::PerChild {
                contributions: child_contributions,
                unresolved_sources,
            } => {
                contributions.extend(child_contributions.iter().cloned());
                unresolved.extend(unresolved_sources.iter().copied());
            }
            ControlDimensionValue::Complete(occurrences) => {
                contributions.push(DimensionContribution {
                    source_node: child.control_node,
                    value: occurrences.clone(),
                });
            }
            ControlDimensionValue::Partial {
                known_contribution,
                unresolved_sources,
            } => {
                if let Some(occurrences) = known_contribution {
                    contributions.push(DimensionContribution {
                        source_node: child.control_node,
                        value: occurrences.clone(),
                    });
                }
                unresolved.push(child.control_node);
                unresolved.extend(unresolved_sources.iter().copied());
            }
            ControlDimensionValue::Unknown {
                known_contributions,
                unresolved_sources,
            } => {
                contributions.extend(known_contributions.iter().cloned());
                unresolved.push(child.control_node);
                unresolved.extend(unresolved_sources.iter().copied());
            }
            ControlDimensionValue::NotApplicable => {
                unresolved.push(child.control_node);
            }
        }
    }
    unresolved.sort_unstable();
    unresolved.dedup();
    let occurrences = contributions
        .iter()
        .flat_map(|item| item.value.iter().cloned())
        .collect::<Vec<_>>();
    let aliases = group_resource_aliases(&occurrences);
    let verdict = if unresolved.is_empty() {
        LeafVerdict::Proven
    } else {
        LeafVerdict::Unproven
    };
    let proof = ControlDimensionProof::new(
        "resource-port-provenance",
        verdict,
        RULE_RESOURCE_PROVENANCE,
        node,
        Some(format!("computation:{}", node.raw())),
        "resource ports remain per-child namespace coordinates; only equal caller ResourceSlotIds are recorded as explicit aliases and no row union or bound reduction occurs",
    );
    let dimension = ControlDimension {
        value: ControlDimensionValue::PerChild {
            contributions,
            unresolved_sources: unresolved,
        },
        proofs: vec![proof.clone()],
    };
    (dimension, aliases, vec![proof])
}

fn group_resource_aliases(occurrences: &[ResourcePortOccurrence]) -> Vec<ControlResourceAlias> {
    let mut by_slot = BTreeMap::<ResourceSlotId, Vec<ResourcePortOccurrence>>::new();
    for occurrence in occurrences {
        by_slot
            .entry(occurrence.mapping.resource_slot)
            .or_default()
            .push(occurrence.clone());
    }
    by_slot
        .into_iter()
        .filter_map(|(resource_slot, occurrences)| {
            (occurrences.len() > 1).then_some(ControlResourceAlias {
                resource_slot,
                occurrences,
            })
        })
        .collect()
}

fn derive_recovery_dimension(
    node: ComputationId,
    operator: &ControlOperator,
    children: &[DerivedControlContractSummary<'_, '_>],
    occurrences: &[ScopedEffectOccurrence],
) -> (
    ControlDimension<Vec<RecoverySourceSummary>>,
    Vec<ControlDimensionProof>,
) {
    let mut contributions = Vec::new();
    let mut unresolved = Vec::new();
    for child in children {
        match &child.recovery.value {
            ControlDimensionValue::PerChild {
                contributions: child_contributions,
                unresolved_sources,
            } => {
                contributions.extend(child_contributions.iter().cloned());
                unresolved.extend(unresolved_sources.iter().copied());
            }
            ControlDimensionValue::Complete(sources) => contributions.push(DimensionContribution {
                source_node: child.control_node,
                value: sources.clone(),
            }),
            ControlDimensionValue::Partial {
                known_contribution,
                unresolved_sources,
            } => {
                if let Some(sources) = known_contribution {
                    contributions.push(DimensionContribution {
                        source_node: child.control_node,
                        value: sources.clone(),
                    });
                }
                unresolved.push(child.control_node);
                unresolved.extend(unresolved_sources.iter().copied());
            }
            ControlDimensionValue::Unknown {
                known_contributions,
                unresolved_sources,
            } => {
                contributions.extend(known_contributions.iter().cloned());
                unresolved.push(child.control_node);
                unresolved.extend(unresolved_sources.iter().copied());
            }
            ControlDimensionValue::NotApplicable => {
                unresolved.push(child.control_node);
            }
        }
    }
    unresolved.sort_unstable();
    unresolved.dedup();
    let verdict = if unresolved.is_empty() {
        LeafVerdict::Proven
    } else {
        LeafVerdict::Unproven
    };
    let proof = ControlDimensionProof::new(
        "recovery-obligations",
        verdict,
        if matches!(operator, ControlOperator::RaceFirstTerminal) {
            RULE_RACE_RECOVERY
        } else {
            RULE_RECOVERY_PROPAGATION
        },
        node,
        Some(format!("computation:{}", node.raw())),
        if occurrences.is_empty() {
            "all known child K declarations and recovery bindings remain source-scoped; an empty K row is not a statement that the computation succeeds"
        } else {
            "recovery declarations and bindings for every possible Effect occurrence are retained per source child; RACE losers are not removed and compensation never subtracts forward effects"
        },
    );
    let dimension = ControlDimension {
        value: ControlDimensionValue::PerChild {
            contributions,
            unresolved_sources: unresolved,
        },
        proofs: vec![proof.clone()],
    };
    (dimension, vec![proof])
}

fn derive_execution_dimension(
    node: ComputationId,
    children: &[DerivedControlContractSummary<'_, '_>],
) -> (
    ControlDimension<Vec<ExecutionSourceSummary>>,
    Vec<ControlDimensionProof>,
) {
    let mut contributions = Vec::new();
    let mut unresolved = Vec::new();
    for child in children {
        match &child.execution.value {
            ControlDimensionValue::PerChild {
                contributions: child_contributions,
                unresolved_sources,
            } => {
                contributions.extend(child_contributions.iter().cloned());
                unresolved.extend(unresolved_sources.iter().copied());
            }
            ControlDimensionValue::Complete(sources) => contributions.push(DimensionContribution {
                source_node: child.control_node,
                value: sources.clone(),
            }),
            ControlDimensionValue::Partial {
                known_contribution,
                unresolved_sources,
            } => {
                if let Some(sources) = known_contribution {
                    contributions.push(DimensionContribution {
                        source_node: child.control_node,
                        value: sources.clone(),
                    });
                }
                unresolved.push(child.control_node);
                unresolved.extend(unresolved_sources.iter().copied());
            }
            ControlDimensionValue::Unknown {
                known_contributions,
                unresolved_sources,
            } => {
                contributions.extend(known_contributions.iter().cloned());
                unresolved.push(child.control_node);
                unresolved.extend(unresolved_sources.iter().copied());
            }
            ControlDimensionValue::NotApplicable => {
                unresolved.push(child.control_node);
            }
        }
    }
    unresolved.sort_unstable();
    unresolved.dedup();
    let verdict = if unresolved.is_empty() {
        LeafVerdict::Proven
    } else {
        LeafVerdict::Unproven
    };
    let proof = ControlDimensionProof::new(
        "execution-per-child",
        verdict,
        RULE_EXECUTION_PER_CHILD,
        node,
        Some(format!("computation:{}", node.raw())),
        "X requirements remain per child; concurrency, branch selection and race winner do not create a single-worker aggregate, capacity reservation or admission result",
    );
    let dimension = ControlDimension {
        value: ControlDimensionValue::PerChild {
            contributions,
            unresolved_sources: unresolved,
        },
        proofs: vec![proof.clone()],
    };
    (dimension, vec![proof])
}

#[cfg(test)]
mod tests {
    use super::*;
    use af_effect::{
        EffectAtom, EffectEntry, EffectGuard, EffectGuardInputEquals, EffectIdempotencyQualifier,
        EffectOperationRef, EffectQualifier, OccurrenceBounds, OccurrenceCeiling, ResourceSelector,
    };

    fn entry(
        qualifier: EffectIdempotencyQualifier,
        lower: u64,
        upper: u64,
        guard: EffectGuard,
    ) -> EffectEntry {
        EffectEntry {
            atom: EffectAtom {
                operation: EffectOperationRef {
                    catalog: "agentflow.effects@1".into(),
                    name: "write".into(),
                },
                resource_selector: ResourceSelector {
                    requirement_port_no: 1,
                    scope_bindings: vec![],
                },
                qualifiers: vec![EffectQualifier::Idempotency(qualifier)],
            },
            bounds: OccurrenceBounds {
                lower,
                upper: OccurrenceCeiling::Finite(upper),
            },
            guard,
        }
    }

    fn signature(entries: Vec<EffectEntry>) -> EffectSignature {
        EffectSignature {
            row: EffectRow::Closed(entries),
        }
    }

    fn single_entry(signature: &EffectSignature) -> &EffectEntry {
        let EffectRow::Closed(entries) = &signature.row else {
            panic!("closed expected")
        };
        assert_eq!(entries.len(), 1);
        &entries[0]
    }

    #[test]
    fn sequence_and_race_weaken_only_lower_bounds_the_operator_cannot_guarantee() {
        let unconditional = EffectGuard::Unconditional;
        let a = signature(vec![entry(
            EffectIdempotencyQualifier::Natural,
            1,
            2,
            unconditional.clone(),
        )]);
        let b = signature(vec![entry(
            EffectIdempotencyQualifier::Natural,
            1,
            3,
            unconditional,
        )]);
        let sequence = combine_effect_signatures(
            &ControlOperator::Sequence,
            &[(0, 2.into(), a.clone()), (1, 3.into(), b.clone())],
            None,
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            single_entry(&sequence).bounds,
            OccurrenceBounds {
                lower: 1,
                upper: OccurrenceCeiling::Finite(5)
            }
        );

        let race = combine_effect_signatures(
            &ControlOperator::RaceFirstTerminal,
            &[(0, 2.into(), a.clone()), (1, 3.into(), b.clone())],
            None,
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            single_entry(&race).bounds,
            OccurrenceBounds {
                lower: 0,
                upper: OccurrenceCeiling::Finite(5)
            }
        );

        let all = combine_effect_signatures(
            &ControlOperator::Parallel {
                mode: ParallelMode::All,
                max_inflight: Some(1),
            },
            &[(0, 2.into(), a.clone()), (1, 3.into(), b.clone())],
            None,
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            single_entry(&all).bounds,
            OccurrenceBounds {
                lower: 2,
                upper: OccurrenceCeiling::Finite(5)
            }
        );

        let fail_fast = combine_effect_signatures(
            &ControlOperator::Parallel {
                mode: ParallelMode::FailFast,
                max_inflight: Some(1),
            },
            &[(0, 2.into(), a), (1, 3.into(), b)],
            None,
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            single_entry(&fail_fast).bounds,
            OccurrenceBounds {
                lower: 0,
                upper: OccurrenceCeiling::Finite(5)
            }
        );
    }

    #[test]
    fn select_join_does_not_promote_different_guards_or_merge_qualifiers() {
        let guard_a = EffectGuard::InputEquals(EffectGuardInputEquals {
            port_no: 1,
            expected: af_semantic::PrimitiveConstant::String("a".into()),
        });
        let guard_b = EffectGuard::InputEquals(EffectGuardInputEquals {
            port_no: 1,
            expected: af_semantic::PrimitiveConstant::String("b".into()),
        });
        let guarded_a = signature(vec![entry(
            EffectIdempotencyQualifier::Natural,
            1,
            2,
            guard_a,
        )]);
        let guarded_b = signature(vec![entry(
            EffectIdempotencyQualifier::Natural,
            1,
            3,
            guard_b,
        )]);
        let guarded_join = combine_effect_signatures(
            &ControlOperator::Select,
            &[(0, 2.into(), guarded_a), (1, 3.into(), guarded_b)],
            None,
        )
        .unwrap()
        .unwrap();
        let joined_entry = single_entry(&guarded_join);
        assert_eq!(joined_entry.bounds.lower, 0);
        assert_eq!(joined_entry.bounds.upper, OccurrenceCeiling::Finite(3));
        assert_eq!(joined_entry.guard, EffectGuard::Unconditional);

        let natural = signature(vec![entry(
            EffectIdempotencyQualifier::Natural,
            1,
            2,
            EffectGuard::Unconditional,
        )]);
        let none = signature(vec![entry(
            EffectIdempotencyQualifier::None,
            1,
            3,
            EffectGuard::Unconditional,
        )]);
        let qualifier_join = combine_effect_signatures(
            &ControlOperator::Select,
            &[(0, 2.into(), natural), (1, 3.into(), none)],
            None,
        )
        .unwrap()
        .unwrap();
        let EffectRow::Closed(entries) = qualifier_join.row else {
            panic!("closed expected")
        };
        assert_eq!(
            entries.len(),
            2,
            "complete qualifier identity remains in AtomKey"
        );
        assert!(entries.iter().all(|item| item.bounds.lower == 0));
    }
}
