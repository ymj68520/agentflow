//! Static, bottom-up composition for the Phase 4B structured-control operators.
//!
//! The derived values here are analysis summaries, never a ComponentContract
//! or a durable identity. Sequence/Select/Parallel/Race and
//! Iterate/Retry/Timeout/Handle/Scope/CompensationScope have static
//! propagation rules. Operators outside the frozen set remain explicit as
//! unsupported; they are not treated as empty.

use std::collections::{BTreeMap, BTreeSet};

use af_contract::{
    ActionGuardCompatibility, RecoveryActionBinding, RecoveryActionIdentity, RecoveryActionKind,
    RecoveryUsageBinding, StaticContractProofContext,
};
use af_effect::{
    AtomKey, EffectEntry, EffectGuard, EffectOpenRow, EffectResidual, EffectRow, EffectSignature,
    OccurrenceBounds, OccurrenceCeiling,
};
use af_execution::ExecutionRequirements;
use af_failure::FailureRow;
use af_flow::{InformationFlowContract, SourceCategory, StaticInformationSummary};
use af_ir::{
    AwaitBody, CallSubflowBody, CheckedIrDocument, ComputationBody, ComputationDef, ComputationId,
    HandleMatch, InvokeBody, IterateBody, ParallelMode, ProducerLocation, SemanticDataflowAnalysis,
    SemanticIrDocument,
};
use af_recovery::RecoveryContract;
use af_resource::ResourceRequirement;
use af_semantic::{
    DigestRefValue, ExpressionBody, ExpressionId, PrimitiveConstant, ResourceSlotId,
    SemanticTypeId, TypeSchemaRef, ValueSlotId,
};
use af_usage::UsagePhase;
use af_usage::{StaticUsageBound, StaticUsageEnvelope, UsageUpperBound};

use crate::proof::LeafVerdict;
use crate::resolution::resolve_computation_leaf;
use crate::root_recovery::{
    AppliedMultiplicity, ControlMultiplicityIdentity, ControlMultiplicityKind, ControlPathKind,
    ControlPathReachability, ControlPathStep, LocalActionBound, LocalRecoveryAction,
    RecoveryActionSourceIdentity, RecoveryActionUpper, RootActionBound,
    RootGuardCoordinateEvidence, RootRecoveryAction, RootRecoveryOwner,
    RootRecoverySourceClosureReport, RootRecoverySummary,
};
use crate::summary::{
    EffectOccurrenceFamily, EffectOccurrenceFamilyKind, FlowPortMapping, LeafKind,
    ProofContextReference, ResolvedComputationLeaf, ResolvedTarget, ScopedEffectOccurrence,
    ScopedEffectResidual, ScopedPortBinding, ScopedResourcePortMapping,
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
const RULE_ITERATION_ENVELOPE: &str = "AF-SEMANTIC-0143";
const RULE_RETRY_ENVELOPE: &str = "AF-SEMANTIC-0144";
const RULE_TIMEOUT_OUTCOMES: &str = "AF-SEMANTIC-0145";
const RULE_HANDLE_RESIDUAL: &str = "AF-SEMANTIC-0146";
const RULE_SCOPE_RESOURCE_BOUNDARY: &str = "AF-SEMANTIC-0147";
const RULE_SCOPE_RUNTIME_OBLIGATIONS: &str = "AF-SEMANTIC-0148";
const RULE_COMPENSATION_FORWARD_HISTORY: &str = "AF-SEMANTIC-0149";
const RULE_COMPENSATION_ACTIONS: &str = "AF-SEMANTIC-0150";
const RULE_COMPENSATION_USAGE_ORDER: &str = "AF-SEMANTIC-0151";
const RULE_RECOVERY_SOURCE_CLOSURE: &str = "AF-SEMANTIC-0154";
const RULE_RECOVERY_ROOT_MULTIPLICITY: &str = "AF-SEMANTIC-0155";
const RULE_RECOVERY_ROOT_ALGEBRA: &str = "AF-SEMANTIC-0156";
const RULE_RECOVERY_ROOT_EFFECT_COVERAGE: &str = "AF-SEMANTIC-0157";
const RULE_RECOVERY_ROOT_USAGE_COVERAGE: &str = "AF-SEMANTIC-0158";
const RULE_CONTROL_TREE_CLOSURE: &str = "AF-SEMANTIC-0152";
const RULE_ROOT_EXPORT_BOUNDARY: &str = "AF-SEMANTIC-0153";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IterateMode {
    Foreach {
        statically_known_item_count: Option<u64>,
        max_inflight: Option<u32>,
    },
    While {
        max_iterations: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandleMatcherSummary {
    Failure(TypeSchemaRef),
    TimedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeoutOutcome {
    ChildCompleted,
    TimedOut,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandleClauseSummary {
    ordinal: u32,
    matcher: HandleMatcherSummary,
    handler: ComputationId,
}

impl HandleClauseSummary {
    #[must_use]
    pub fn ordinal(&self) -> u32 {
        self.ordinal
    }
    #[must_use]
    pub fn matcher(&self) -> &HandleMatcherSummary {
        &self.matcher
    }
    #[must_use]
    pub fn handler(&self) -> ComputationId {
        self.handler
    }
}

/// Derived evidence for the frozen SCOPE resource-visibility boundary. The
/// authority, budget, and settle-before-terminal properties are obligations,
/// not runtime facts supplied by Semantic IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeBoundarySummary {
    control_node: ComputationId,
    child: ComputationId,
    parent_visible_resources: Vec<ResourceSlotId>,
    declared_resources: Vec<ResourceSlotId>,
    child_visible_resources: Vec<ResourceSlotId>,
    resource_attenuation: LeafVerdict,
    authority_attenuation: LeafVerdict,
    budget_attenuation: LeafVerdict,
    owned_children_settle_before_terminal: LeafVerdict,
}

impl ScopeBoundarySummary {
    #[must_use]
    pub fn control_node(&self) -> ComputationId {
        self.control_node
    }
    #[must_use]
    pub fn child(&self) -> ComputationId {
        self.child
    }
    #[must_use]
    pub fn parent_visible_resources(&self) -> &[ResourceSlotId] {
        &self.parent_visible_resources
    }
    #[must_use]
    pub fn declared_resources(&self) -> &[ResourceSlotId] {
        &self.declared_resources
    }
    #[must_use]
    pub fn child_visible_resources(&self) -> &[ResourceSlotId] {
        &self.child_visible_resources
    }
    #[must_use]
    pub fn resource_attenuation(&self) -> LeafVerdict {
        self.resource_attenuation
    }
    #[must_use]
    pub fn authority_attenuation(&self) -> LeafVerdict {
        self.authority_attenuation
    }
    #[must_use]
    pub fn budget_attenuation(&self) -> LeafVerdict {
        self.budget_attenuation
    }
    #[must_use]
    pub fn owned_children_settle_before_terminal(&self) -> LeafVerdict {
        self.owned_children_settle_before_terminal
    }
}

/// One possible compensation Action reconstructed from exact K bindings and
/// scoped forward/action Effect occurrences. Its upper must fit the declared
/// source Effect row. This is never an EffectReceipt or dispatch instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionalCompensationAction {
    source_node: ComputationId,
    forward_occurrence: ScopedEffectOccurrence,
    action_effect_occurrence: ScopedEffectOccurrence,
    identity: RecoveryActionIdentity,
    local_bound: LocalActionBound,
    potential_effect: EffectEntry,
    potential_recovery_usage: StaticUsageEnvelope,
    receipt_evidence: LeafVerdict,
}

impl ConditionalCompensationAction {
    #[must_use]
    pub fn source_node(&self) -> ComputationId {
        self.source_node
    }
    #[must_use]
    pub fn forward_occurrence(&self) -> &ScopedEffectOccurrence {
        &self.forward_occurrence
    }
    #[must_use]
    pub fn action_effect_occurrence(&self) -> &ScopedEffectOccurrence {
        &self.action_effect_occurrence
    }
    #[must_use]
    pub fn identity(&self) -> &RecoveryActionIdentity {
        &self.identity
    }
    #[must_use]
    pub fn local_bound(&self) -> &LocalActionBound {
        &self.local_bound
    }
    #[must_use]
    pub fn potential_effect(&self) -> &EffectEntry {
        &self.potential_effect
    }
    #[must_use]
    pub fn potential_recovery_usage(&self) -> &StaticUsageEnvelope {
        &self.potential_recovery_usage
    }
    #[must_use]
    pub fn receipt_evidence(&self) -> LeafVerdict {
        self.receipt_evidence
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompensationScopeSummary {
    potential_actions: ControlDimension<Vec<ConditionalCompensationAction>>,
    local_recovery_actions: ControlDimension<Vec<LocalRecoveryAction>>,
    compensation_order: LeafVerdict,
    recovery_blocked_outcome: LeafVerdict,
    forward_effects_retained: bool,
    forward_outcome_retained: bool,
}

impl CompensationScopeSummary {
    #[must_use]
    pub fn potential_actions(&self) -> &ControlDimension<Vec<ConditionalCompensationAction>> {
        &self.potential_actions
    }
    #[must_use]
    pub fn local_recovery_actions(&self) -> &ControlDimension<Vec<LocalRecoveryAction>> {
        &self.local_recovery_actions
    }
    #[must_use]
    pub fn compensation_order(&self) -> LeafVerdict {
        self.compensation_order
    }
    #[must_use]
    pub fn recovery_blocked_outcome(&self) -> LeafVerdict {
        self.recovery_blocked_outcome
    }
    #[must_use]
    pub fn forward_effects_retained(&self) -> bool {
        self.forward_effects_retained
    }
    #[must_use]
    pub fn forward_outcome_retained(&self) -> bool {
        self.forward_outcome_retained
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootExportCompatibilityStatus {
    NotExpressedInSemanticIr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlTreeClosureSummary {
    computation_order: Vec<ComputationId>,
    leaf_order: Vec<ComputationId>,
    known_effect_occurrences_accounted: bool,
    known_effect_residuals_accounted: bool,
    known_recovery_sources_accounted: bool,
    root_export_compatibility: RootExportCompatibilityStatus,
    proof: ControlProofReport,
}

impl ControlTreeClosureSummary {
    #[must_use]
    pub fn computation_order(&self) -> &[ComputationId] {
        &self.computation_order
    }
    #[must_use]
    pub fn leaf_order(&self) -> &[ComputationId] {
        &self.leaf_order
    }
    #[must_use]
    pub fn known_effect_occurrences_accounted(&self) -> bool {
        self.known_effect_occurrences_accounted
    }
    #[must_use]
    pub fn known_effect_residuals_accounted(&self) -> bool {
        self.known_effect_residuals_accounted
    }
    #[must_use]
    pub fn known_recovery_sources_accounted(&self) -> bool {
        self.known_recovery_sources_accounted
    }
    #[must_use]
    pub fn root_export_compatibility(&self) -> RootExportCompatibilityStatus {
        self.root_export_compatibility
    }
    #[must_use]
    pub fn proof(&self) -> &ControlProofReport {
        &self.proof
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedWorkflowContractSummary<'context, 'universe> {
    root: DerivedControlContractSummary<'context, 'universe>,
    closure: ControlTreeClosureSummary,
    root_recovery: RootRecoverySummary,
}

impl DerivedWorkflowContractSummary<'_, '_> {
    #[must_use]
    pub fn root(&self) -> &DerivedControlContractSummary<'_, '_> {
        &self.root
    }
    #[must_use]
    pub fn closure(&self) -> &ControlTreeClosureSummary {
        &self.closure
    }
    #[must_use]
    pub fn root_recovery(&self) -> &RootRecoverySummary {
        &self.root_recovery
    }
    #[must_use]
    pub fn overall(&self) -> LeafVerdict {
        merge_verdict(
            merge_verdict(self.root.proof().overall(), self.closure.proof().overall()),
            self.root_recovery.overall(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlDerivationError {
    UnknownComputation(ComputationId),
    DuplicateLeafSummary(ComputationId),
    LeafSummaryForNonLeaf(ComputationId),
    LeafContextMismatch(ComputationId),
    CallerDigestMismatch(ComputationId),
    SemanticDigest(String),
    Dataflow(String),
    LeafResolution(String),
    ClosureMismatch(String),
    ArithmeticOverflow(&'static str),
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
            Self::LeafResolution(detail) => write!(f, "leaf resolution failed: {detail}"),
            Self::ClosureMismatch(detail) => write!(f, "control-tree closure failed: {detail}"),
            Self::ArithmeticOverflow(dimension) => {
                write!(f, "static {dimension} cardinality overflows u64")
            }
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
    Iterate {
        mode: IterateMode,
        effect_repetitions: OccurrenceBounds,
        usage_repetitions: Option<u64>,
    },
    Retry {
        delays: Vec<af_core::Duration>,
        max_attempts: u64,
    },
    Timeout {
        expression: ExpressionId,
        static_duration: Option<af_core::Duration>,
        possible_outcomes: [TimeoutOutcome; 2],
    },
    Handle {
        handled: FailureRow,
        clauses: Vec<HandleClauseSummary>,
    },
    Scope,
    CompensationScope,
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
            Self::Iterate { .. } => "iterate",
            Self::Retry { .. } => "retry",
            Self::Timeout { .. } => "timeout",
            Self::Handle { .. } => "handle",
            Self::Scope => "scope",
            Self::CompensationScope => "compensation_scope",
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
    control_decision_dependencies: Vec<ValueSlotId>,
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
    pub fn control_decision_dependencies(&self) -> &[ValueSlotId] {
        &self.control_decision_dependencies
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
    action_guard_compatibility: Vec<ActionGuardCompatibility>,
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
    #[must_use]
    pub fn action_guard_compatibility(&self) -> &[ActionGuardCompatibility] {
        &self.action_guard_compatibility
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
    scope_boundary: Option<ScopeBoundarySummary>,
    compensation_scope: Option<CompensationScopeSummary>,
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
    pub fn scope_boundary(&self) -> Option<&ScopeBoundarySummary> {
        self.scope_boundary.as_ref()
    }
    #[must_use]
    pub fn compensation_scope(&self) -> Option<&CompensationScopeSummary> {
        self.compensation_scope.as_ref()
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

/// Resolves every reachable leaf in a checked Semantic IR and derives one
/// bottom-up root summary plus an explicit static tree-closure report.
/// `RootExportCompatibilityStatus::NotExpressedInSemanticIr` is intentional:
/// the frozen document has root outputs, but no root ComponentContract pin.
pub fn derive_workflow_contract_summary<'context, 'universe>(
    ir: &CheckedIrDocument<'_>,
    snapshot: &crate::resolution::StaticComputationResolutionSnapshot<'_, 'context, 'universe>,
    proof_context: &'context StaticContractProofContext<'universe>,
) -> Result<DerivedWorkflowContractSummary<'context, 'universe>, ControlDerivationError> {
    let document = ir.source();
    let recovery_dataflow = ir
        .analyze_dataflow()
        .map_err(|error| ControlDerivationError::Dataflow(error.to_string()))?;
    if !snapshot.is_same_context(proof_context) {
        return Err(ControlDerivationError::LeafContextMismatch(document.root));
    }
    let computations = document
        .computations
        .iter()
        .map(|computation| (computation.id, computation))
        .collect::<BTreeMap<_, _>>();
    let mut computation_order = Vec::new();
    let mut visiting = BTreeSet::new();
    collect_owned_computations(
        &computations,
        document.root,
        &mut visiting,
        &mut computation_order,
    )?;
    let all_declared = computations.keys().copied().collect::<BTreeSet<_>>();
    let visited = computation_order.iter().copied().collect::<BTreeSet<_>>();
    if all_declared != visited || computation_order.len() != visited.len() {
        return Err(ControlDerivationError::ClosureMismatch(
            "checked root walk did not account for every declared computation exactly once".into(),
        ));
    }
    let leaf_order = computation_order
        .iter()
        .copied()
        .filter(|node| {
            computations.get(node).is_some_and(|computation| {
                matches!(
                    computation.body,
                    ComputationBody::Invoke(_)
                        | ComputationBody::Await(_)
                        | ComputationBody::CallSubflow(_)
                )
            })
        })
        .collect::<Vec<_>>();
    let leaves = leaf_order
        .iter()
        .map(|node| {
            resolve_computation_leaf(ir, *node, snapshot)
                .map_err(|error| ControlDerivationError::LeafResolution(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let root = derive_control_contract(ir, document.root, &leaves, proof_context)?;

    let mut derived_order = Vec::new();
    collect_derived_control_order(&root, &mut derived_order);
    if derived_order != computation_order {
        return Err(ControlDerivationError::ClosureMismatch(
            "bottom-up contract derivation did not visit nodes in checked owned-child order".into(),
        ));
    }
    let mut expected_occurrences = leaves
        .iter()
        .flat_map(|leaf| leaf.effect_occurrences().iter().cloned())
        .collect::<Vec<_>>();
    let mut actual_occurrences = root.effect_occurrences.clone();
    for occurrence in &mut expected_occurrences {
        occurrence.control_families.clear();
    }
    for occurrence in &mut actual_occurrences {
        occurrence.control_families.clear();
    }
    let effects_accounted = expected_occurrences == actual_occurrences;
    let expected_residuals = leaves
        .iter()
        .flat_map(|leaf| leaf.effect_residuals().iter().cloned())
        .collect::<Vec<_>>();
    let residuals_accounted = expected_residuals == root.effect_residuals;
    let expected_recovery_sources = leaves
        .iter()
        .filter(|leaf| leaf.dimensions().is_some() && leaf.target().is_some())
        .map(ResolvedComputationLeaf::computation)
        .collect::<Vec<_>>();
    let mut actual_recovery_sources = Vec::new();
    collect_recovery_source_nodes(root.recovery.value(), &mut actual_recovery_sources);
    let recovery_sources_accounted = expected_recovery_sources == actual_recovery_sources;
    if !effects_accounted || !residuals_accounted || !recovery_sources_accounted {
        return Err(ControlDerivationError::ClosureMismatch(format!(
            "source coverage mismatch: occurrences={effects_accounted}, residuals={residuals_accounted}, recovery={recovery_sources_accounted}"
        )));
    }
    let proof = ControlProofReport::new(vec![
        ControlDimensionProof::new(
            "control-tree-closure",
            LeafVerdict::Proven,
            RULE_CONTROL_TREE_CLOSURE,
            document.root,
            Some(format!("computation:{}", document.root.raw())),
            "every checked owned computation and reachable INVOKE/AWAIT/CALL_SUBFLOW leaf was visited exactly once in semantic order; known ε occurrences/residuals and K sources match the resolved leaves",
        ),
        ControlDimensionProof::new(
            "root-export-compatibility",
            LeafVerdict::Unproven,
            RULE_ROOT_EXPORT_BOUNDARY,
            document.root,
            Some(format!("computation:{}", document.root.raw())),
            "SemanticIrDocument expresses typed root outputs but has no root ComponentContract pin or frozen cross-coordinate mapping; no export compatibility assertion is invented",
        ),
    ]);
    let closure = ControlTreeClosureSummary {
        computation_order,
        leaf_order,
        known_effect_occurrences_accounted: effects_accounted,
        known_effect_residuals_accounted: residuals_accounted,
        known_recovery_sources_accounted: recovery_sources_accounted,
        root_export_compatibility: RootExportCompatibilityStatus::NotExpressedInSemanticIr,
        proof,
    };
    let root_recovery = derive_root_recovery_summary(&root, &recovery_dataflow, proof_context);
    Ok(DerivedWorkflowContractSummary {
        root,
        closure,
        root_recovery,
    })
}

fn collect_owned_computations(
    computations: &BTreeMap<ComputationId, &ComputationDef>,
    node: ComputationId,
    visiting: &mut BTreeSet<ComputationId>,
    order: &mut Vec<ComputationId>,
) -> Result<(), ControlDerivationError> {
    if !visiting.insert(node) {
        return Err(ControlDerivationError::ClosureMismatch(format!(
            "computation {node} occurs more than once in the checked owned-child traversal"
        )));
    }
    let Some(computation) = computations.get(&node) else {
        return Err(ControlDerivationError::UnknownComputation(node));
    };
    order.push(node);
    for child in computation.owned_children() {
        collect_owned_computations(computations, child, visiting, order)?;
    }
    Ok(())
}

fn collect_derived_control_order(
    summary: &DerivedControlContractSummary<'_, '_>,
    order: &mut Vec<ComputationId>,
) {
    order.push(summary.control_node);
    for child in &summary.children {
        collect_derived_control_order(child, order);
    }
}

fn collect_recovery_source_nodes(
    value: &ControlDimensionValue<Vec<RecoverySourceSummary>>,
    output: &mut Vec<ComputationId>,
) {
    match value {
        ControlDimensionValue::Complete(sources) => {
            output.extend(sources.iter().map(RecoverySourceSummary::source_node));
        }
        ControlDimensionValue::PerChild { contributions, .. } => {
            output.extend(
                contributions
                    .iter()
                    .flat_map(|contribution| contribution.value.iter())
                    .map(RecoverySourceSummary::source_node),
            );
        }
        ControlDimensionValue::Partial {
            known_contribution: Some(sources),
            ..
        } => output.extend(sources.iter().map(RecoverySourceSummary::source_node)),
        ControlDimensionValue::Unknown {
            known_contributions,
            ..
        } => output.extend(
            known_contributions
                .iter()
                .flat_map(|contribution| contribution.value.iter())
                .map(RecoverySourceSummary::source_node),
        ),
        ControlDimensionValue::Partial { .. } | ControlDimensionValue::NotApplicable => {}
    }
}

struct RootRecoveryFold {
    action_effects: ControlDimensionValue<EffectSignature>,
    epsilon_coverage_requirement: ControlDimensionValue<EffectSignature>,
    recovery_usage: ControlDimensionValue<StaticUsageEnvelope>,
    violation: bool,
}

fn derive_root_recovery_summary<'context, 'universe>(
    root: &DerivedControlContractSummary<'context, 'universe>,
    dataflow: &SemanticDataflowAnalysis,
    context: &'context StaticContractProofContext<'universe>,
) -> RootRecoverySummary {
    let root_digest = root.source_semantic_ir_digest.clone();
    let mut builder = RootRecoveryBuilder {
        root_digest: &root_digest,
        root_effect_occurrences: &root.effect_occurrences,
        dataflow,
        context,
        actions: Vec::new(),
        leaf_sources: Vec::new(),
        unresolved_sources: Vec::new(),
        guard_verdict: LeafVerdict::Proven,
        action_epsilon_verdict: LeafVerdict::Proven,
        factor_verdict: LeafVerdict::Proven,
        binding_verdict: LeafVerdict::Proven,
        usage_profile_verdict: LeafVerdict::Proven,
        arithmetic_violation: false,
    };
    let folded = derive_root_recovery_node(root, &RootRecoveryPath::default(), &mut builder);

    let (root_sources, root_unresolved) = recovery_sources_from_child(root);
    let root_sources = root_sources
        .into_iter()
        .flat_map(|contribution| contribution.value)
        .collect::<Vec<_>>();
    let source_nodes_accounted =
        if !root_unresolved.is_empty() || !builder.unresolved_sources.is_empty() {
            LeafVerdict::Unproven
        } else if multiset_equal(&root_sources, &builder.leaf_sources) {
            LeafVerdict::Proven
        } else {
            LeafVerdict::Violation
        };

    let mut expected_action_bindings = Vec::new();
    let mut expected_usage_profiles = Vec::new();
    for source in &builder.leaf_sources {
        for binding in source.action_bindings() {
            let matching_usage = source
                .usage_bindings()
                .iter()
                .find(|candidate| candidate.action == binding.identity);
            for forward in source
                .effects()
                .iter()
                .filter(|occurrence| occurrence.declared.atom == binding.identity.forward_effect)
            {
                let key = (
                    forward.caller_semantic_ir_digest.clone(),
                    source.source_node(),
                    forward.source_contract_digest.clone(),
                    forward.source_entry_ordinal,
                    binding.identity.clone(),
                );
                expected_action_bindings.push(key.clone());
                if let Some(usage) = matching_usage {
                    expected_usage_profiles.push((key, usage.clone()));
                }
            }
        }
    }
    let actual_action_bindings = builder
        .actions
        .iter()
        .map(|action| {
            (
                action.source.caller_semantic_ir_digest.clone(),
                action.source.source_node,
                action.source.source_contract_digest.clone(),
                action.source.forward_entry_ordinal,
                action.source.action.clone(),
            )
        })
        .collect::<Vec<_>>();
    let actual_usage_profiles = builder
        .actions
        .iter()
        .map(|action| {
            (
                (
                    action.source.caller_semantic_ir_digest.clone(),
                    action.source.source_node,
                    action.source.source_contract_digest.clone(),
                    action.source.forward_entry_ordinal,
                    action.source.action.clone(),
                ),
                action.recovery_usage_binding.clone(),
            )
        })
        .collect::<Vec<_>>();
    let closure_has_unknown = !root_unresolved.is_empty() || !builder.unresolved_sources.is_empty();
    let action_bindings_accounted = if closure_has_unknown {
        LeafVerdict::Unproven
    } else if multiset_equal(&expected_action_bindings, &actual_action_bindings) {
        builder.binding_verdict
    } else {
        LeafVerdict::Violation
    };
    let usage_profiles_accounted = if closure_has_unknown {
        LeafVerdict::Unproven
    } else if multiset_equal(&expected_usage_profiles, &actual_usage_profiles) {
        builder.usage_profile_verdict
    } else {
        LeafVerdict::Violation
    };
    let root_action_totals_sound = if builder.arithmetic_violation
        || builder.factor_verdict == LeafVerdict::Violation
        || source_nodes_accounted == LeafVerdict::Violation
        || action_bindings_accounted == LeafVerdict::Violation
        || usage_profiles_accounted == LeafVerdict::Violation
    {
        LeafVerdict::Violation
    } else if !matches!(&folded.action_effects, ControlDimensionValue::Complete(_))
        || action_bindings_accounted != LeafVerdict::Proven
        || source_nodes_accounted != LeafVerdict::Proven
        || usage_profiles_accounted != LeafVerdict::Proven
        || builder.factor_verdict != LeafVerdict::Proven
    {
        LeafVerdict::Unproven
    } else {
        LeafVerdict::Proven
    };
    let source_closure = RootRecoverySourceClosureReport {
        source_nodes_accounted,
        action_bindings_accounted,
        usage_profiles_accounted,
        multiplicity_factors_accounted: builder.factor_verdict,
        root_action_totals_sound,
    };

    let action_state = if !root_unresolved.is_empty() || !builder.unresolved_sources.is_empty() {
        if builder.actions.is_empty() {
            ControlDimensionValue::Unknown {
                known_contributions: vec![],
                unresolved_sources: unique_nodes(
                    root_unresolved
                        .iter()
                        .copied()
                        .chain(builder.unresolved_sources.iter().copied()),
                ),
            }
        } else {
            ControlDimensionValue::Partial {
                known_contribution: Some(builder.actions.clone()),
                unresolved_sources: unique_nodes(
                    root_unresolved
                        .iter()
                        .copied()
                        .chain(builder.unresolved_sources.iter().copied()),
                ),
            }
        }
    } else {
        ControlDimensionValue::Complete(builder.actions.clone())
    };
    let action_effect_proof = if builder.arithmetic_violation {
        LeafVerdict::Violation
    } else {
        merge_verdict(
            builder.action_epsilon_verdict,
            root_action_effect_coverage(&folded.epsilon_coverage_requirement, root.effects.value()),
        )
    };
    let recovery_usage_proof = if builder.arithmetic_violation {
        LeafVerdict::Violation
    } else {
        root_recovery_usage_coverage(&folded.recovery_usage, root.usage.value())
    };
    let eligibility_verdict = builder
        .actions
        .iter()
        .map(|action| action.eligibility_evidence)
        .fold(
            if source_nodes_accounted == LeafVerdict::Proven {
                LeafVerdict::Proven
            } else {
                LeafVerdict::Unproven
            },
            merge_verdict,
        );

    let proofs = vec![
        ControlDimensionProof::new(
            "root-recovery-source-closure",
            merge_verdict(
                merge_verdict(
                    source_nodes_accounted,
                    merge_verdict(action_bindings_accounted, usage_profiles_accounted),
                ),
                root_action_totals_sound,
            ),
            RULE_RECOVERY_SOURCE_CLOSURE,
            root.control_node,
            Some(format!("computation:{}", root.control_node.raw())),
            "the root RecoverySource/Action/Usage profile multisets are compared exactly; source-node equality alone is not accepted as action/profile closure",
        ),
        ControlDimensionProof::new(
            "root-recovery-action-totals",
            root_action_totals_sound,
            RULE_RECOVERY_ROOT_MULTIPLICITY,
            root.control_node,
            Some(format!("computation:{}", root.control_node.raw())),
            "root action totals require a complete leaf-seed fold, exact source bindings and one application of every checked factor identity",
        ),
        ControlDimensionProof::new(
            "root-recovery-multiplicity",
            builder.factor_verdict,
            RULE_RECOVERY_ROOT_MULTIPLICITY,
            root.control_node,
            Some(format!("computation:{}", root.control_node.raw())),
            "each Repeat factor is keyed by caller digest, exact control node and kind, and is applied once by the root fold",
        ),
        ControlDimensionProof::new(
            "root-k-action-epsilon-coverage",
            action_effect_proof,
            RULE_RECOVERY_ROOT_EFFECT_COVERAGE,
            root.control_node,
            Some(format!("computation:{}", root.control_node.raw())),
            "the Root K action projection is checked against the already-derived Root epsilon ceiling and is not added to it",
        ),
        ControlDimensionProof::new(
            "root-k-recovery-usage-coverage",
            recovery_usage_proof,
            RULE_RECOVERY_ROOT_USAGE_COVERAGE,
            root.control_node,
            Some(format!("computation:{}", root.control_node.raw())),
            "Root K-required Recovery Usage is compared with the recovery coordinates of Root U and is never added to U a second time",
        ),
        ControlDimensionProof::new(
            "root-recovery-guard-compatibility",
            builder.guard_verdict,
            RULE_RECOVERY_ROOT_ALGEBRA,
            root.control_node,
            Some(format!("computation:{}", root.control_node.raw())),
            "each source K action retains its validator-created component-local guard proof plus typed scoped input and control-path coordinates",
        ),
        ControlDimensionProof::new(
            "root-recovery-eligibility",
            eligibility_verdict,
            RULE_COMPENSATION_ACTIONS,
            root.control_node,
            Some(format!("computation:{}", root.control_node.raw())),
            "positive static action potential is not receipt eligibility or a recovery command; actual eligibility remains evidence-dependent",
        ),
    ];
    RootRecoverySummary {
        actions: ControlDimension {
            value: action_state,
            proofs: proofs.clone(),
        },
        action_effects: ControlDimension {
            value: folded.action_effects,
            proofs: vec![ControlDimensionProof::new(
                "root-k-action-effect-envelope",
                action_effect_proof,
                RULE_RECOVERY_ROOT_EFFECT_COVERAGE,
                root.control_node,
                Some(format!("computation:{}", root.control_node.raw())),
                "action effects are folded with the existing checked Effect algebra",
            )],
        },
        epsilon_coverage_requirement: ControlDimension {
            value: folded.epsilon_coverage_requirement,
            proofs: vec![ControlDimensionProof::new(
                "root-k-epsilon-coverage-requirement",
                action_effect_proof,
                RULE_RECOVERY_ROOT_EFFECT_COVERAGE,
                root.control_node,
                Some(format!("computation:{}", root.control_node.raw())),
                "the K-only projection remains separate; same-atom Forward opportunities are added once in this comparison requirement, never to the declared epsilon envelope",
            )],
        },
        recovery_usage: ControlDimension {
            value: folded.recovery_usage,
            proofs: vec![ControlDimensionProof::new(
                "root-k-recovery-usage",
                recovery_usage_proof,
                RULE_RECOVERY_ROOT_USAGE_COVERAGE,
                root.control_node,
                Some(format!("computation:{}", root.control_node.raw())),
                "per-attempt Recovery Usage is scaled at the leaf and composed by the existing checked Usage algebra exactly once",
            )],
        },
        source_closure,
        proof: ControlProofReport::new(proofs),
    }
}

fn unique_nodes(nodes: impl IntoIterator<Item = ComputationId>) -> Vec<ComputationId> {
    nodes
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn multiset_equal<T: Clone + PartialEq>(expected: &[T], actual: &[T]) -> bool {
    if expected.len() != actual.len() {
        return false;
    }
    let mut remaining = actual.to_vec();
    for item in expected {
        let Some(index) = remaining.iter().position(|candidate| candidate == item) else {
            return false;
        };
        remaining.remove(index);
    }
    remaining.is_empty()
}

#[derive(Clone)]
struct RootRecoveryPath {
    owner_scope: Option<ComputationId>,
    root_factors: Vec<AppliedMultiplicity>,
    local_factors: Vec<AppliedMultiplicity>,
    control_path: Vec<ControlPathStep>,
    root_reachable: bool,
    local_reachable: bool,
    invalid_factor_identity: bool,
}

impl Default for RootRecoveryPath {
    fn default() -> Self {
        Self {
            owner_scope: None,
            root_factors: vec![],
            local_factors: vec![],
            control_path: vec![],
            root_reachable: true,
            local_reachable: true,
            invalid_factor_identity: false,
        }
    }
}

struct RootRecoveryBuilder<'a, 'context, 'universe> {
    root_digest: &'a DigestRefValue,
    root_effect_occurrences: &'a [ScopedEffectOccurrence],
    dataflow: &'a SemanticDataflowAnalysis,
    context: &'context StaticContractProofContext<'universe>,
    actions: Vec<RootRecoveryAction>,
    leaf_sources: Vec<RecoverySourceSummary>,
    unresolved_sources: Vec<ComputationId>,
    guard_verdict: LeafVerdict,
    action_epsilon_verdict: LeafVerdict,
    factor_verdict: LeafVerdict,
    binding_verdict: LeafVerdict,
    usage_profile_verdict: LeafVerdict,
    arithmetic_violation: bool,
}

fn derive_root_recovery_node(
    summary: &DerivedControlContractSummary<'_, '_>,
    inherited: &RootRecoveryPath,
    builder: &mut RootRecoveryBuilder<'_, '_, '_>,
) -> RootRecoveryFold {
    if summary.children.is_empty() {
        return derive_root_recovery_leaf(summary, inherited, builder);
    }
    let mut path = inherited.clone();
    if matches!(&summary.operator, ControlOperator::CompensationScope) {
        path.owner_scope = Some(summary.control_node);
        path.local_factors.clear();
        // A LocalActionBound is per activation of this owner, even when the
        // entire owner is unreachable from a statically selected root branch.
        path.local_reachable = true;
    }
    if let Some(family) = effect_occurrence_family(summary.control_node, &summary.operator) {
        let applied = AppliedMultiplicity {
            identity: ControlMultiplicityIdentity {
                caller_semantic_ir_digest: summary.source_semantic_ir_digest.clone(),
                control_node: family.control_node,
                kind: match family.kind {
                    EffectOccurrenceFamilyKind::Iterate => ControlMultiplicityKind::Iterate,
                    EffectOccurrenceFamilyKind::Retry => ControlMultiplicityKind::Retry,
                },
            },
            logical_repetitions: family.logical_repetitions,
            physical_attempts: family.physical_attempts,
        };
        if path
            .root_factors
            .iter()
            .any(|existing| existing.identity == applied.identity)
        {
            builder.factor_verdict = LeafVerdict::Violation;
            path.invalid_factor_identity = true;
        } else {
            path.root_factors.push(applied.clone());
        }
        if path.owner_scope.is_some() {
            if path
                .local_factors
                .iter()
                .any(|existing| existing.identity == applied.identity)
            {
                builder.factor_verdict = LeafVerdict::Violation;
                path.invalid_factor_identity = true;
            } else {
                path.local_factors.push(applied);
            }
        }
    }
    let exact_select_child = exact_select_child_index(summary);
    let mut child_folds = Vec::with_capacity(summary.children.len());
    for (child_index, child) in summary.children.iter().enumerate() {
        let child_reachable = path.root_reachable
            && exact_select_child.is_none_or(|selected| selected == child_index);
        let child_path_reachability = if !child_reachable {
            ControlPathReachability::ProvenUnreachable
        } else if exact_select_child == Some(child_index) {
            ControlPathReachability::ExactSelected
        } else {
            ControlPathReachability::Possible
        };
        let mut next = path.clone();
        next.root_reachable = child_reachable;
        if path.owner_scope.is_some()
            && exact_select_child.is_some_and(|selected| selected != child_index)
        {
            next.local_reachable = false;
        }
        next.control_path.push(control_path_step(
            summary,
            child_index,
            child.control_node,
            child_path_reachability,
        ));
        child_folds.push(derive_root_recovery_node(child, &next, builder));
    }
    combine_root_recovery_children(summary, &child_folds, exact_select_child, builder)
}

fn derive_root_recovery_leaf(
    summary: &DerivedControlContractSummary<'_, '_>,
    path: &RootRecoveryPath,
    builder: &mut RootRecoveryBuilder<'_, '_, '_>,
) -> RootRecoveryFold {
    let (contributions, unresolved) = recovery_sources_from_child(summary);
    builder
        .unresolved_sources
        .extend(unresolved.iter().copied());
    let sources = contributions
        .into_iter()
        .flat_map(|contribution| contribution.value)
        .collect::<Vec<_>>();
    builder.leaf_sources.extend(sources.iter().cloned());
    if !unresolved.is_empty() {
        builder
            .unresolved_sources
            .extend(unresolved.iter().copied());
    }

    let mut effect = EffectSignature {
        row: EffectRow::Closed(vec![]),
    };
    let mut epsilon_requirement = EffectSignature {
        row: EffectRow::Closed(vec![]),
    };
    let mut same_atom_forwards = Vec::<(ScopedEffectOccurrence, OccurrenceCeiling)>::new();
    let mut usage = None::<StaticUsageEnvelope>;
    let mut unknown = !unresolved.is_empty();
    let mut leaf_violation = false;
    for source in &sources {
        for binding in source.action_bindings() {
            let identity = &binding.identity;
            let Some(usage_binding) = source
                .usage_bindings()
                .iter()
                .find(|candidate| candidate.action == *identity)
            else {
                builder.binding_verdict =
                    merge_verdict(builder.binding_verdict, LeafVerdict::Violation);
                builder.usage_profile_verdict =
                    merge_verdict(builder.usage_profile_verdict, LeafVerdict::Violation);
                builder.unresolved_sources.push(source.source_node());
                unknown = true;
                continue;
            };
            let forwards = source
                .effects()
                .iter()
                .filter(|occurrence| occurrence.declared.atom == identity.forward_effect)
                .collect::<Vec<_>>();
            if forwards.is_empty() {
                builder.binding_verdict =
                    merge_verdict(builder.binding_verdict, LeafVerdict::Unproven);
                builder.usage_profile_verdict =
                    merge_verdict(builder.usage_profile_verdict, LeafVerdict::Unproven);
                builder.unresolved_sources.push(source.source_node());
                unknown = true;
                continue;
            }
            for forward_source in forwards {
                let Some(forward) =
                    find_root_occurrence(builder.root_effect_occurrences, forward_source)
                else {
                    builder.binding_verdict =
                        merge_verdict(builder.binding_verdict, LeafVerdict::Unproven);
                    builder.usage_profile_verdict =
                        merge_verdict(builder.usage_profile_verdict, LeafVerdict::Unproven);
                    builder.unresolved_sources.push(source.source_node());
                    unknown = true;
                    continue;
                };
                let Some(action_effect) = builder.root_effect_occurrences.iter().find(|entry| {
                    entry.computation == source.source_node()
                        && entry.source_contract_digest == forward.source_contract_digest
                        && entry.declared.atom == identity.action_effect
                }) else {
                    builder.binding_verdict =
                        merge_verdict(builder.binding_verdict, LeafVerdict::Unproven);
                    builder.usage_profile_verdict =
                        merge_verdict(builder.usage_profile_verdict, LeafVerdict::Unproven);
                    builder.unresolved_sources.push(source.source_node());
                    unknown = true;
                    continue;
                };
                let guard_proof = source
                    .action_guard_compatibility()
                    .iter()
                    .find(|proof| proof.recovery_action() == identity)
                    .cloned();
                let guard_coordinates = summary.leaf_summary().map_or_else(Vec::new, |leaf| {
                    root_guard_coordinate_evidence(
                        leaf,
                        &forward.source_contract_digest,
                        guard_proof.as_ref(),
                        builder.dataflow,
                        &mut builder.guard_verdict,
                    )
                });
                let guard_verdict = guard_proof.as_ref().map_or(LeafVerdict::Unproven, |proof| {
                    match proof.verdict() {
                        af_contract::ProofVerdict::Proven => LeafVerdict::Proven,
                        af_contract::ProofVerdict::Violation => LeafVerdict::Violation,
                        af_contract::ProofVerdict::Unproven => LeafVerdict::Unproven,
                        af_contract::ProofVerdict::RequiresEvidence => {
                            LeafVerdict::RequiresEvidence
                        }
                    }
                });
                builder.guard_verdict = merge_verdict(builder.guard_verdict, guard_verdict);
                let Some(forward_seed_upper) = recovery_forward_seed_upper(
                    source,
                    identity,
                    forward_source.declared.bounds.lower,
                    forward_source.declared.bounds.upper,
                ) else {
                    builder.binding_verdict =
                        merge_verdict(builder.binding_verdict, LeafVerdict::Unproven);
                    builder.action_epsilon_verdict =
                        merge_verdict(builder.action_epsilon_verdict, LeafVerdict::Unproven);
                    builder.unresolved_sources.push(source.source_node());
                    unknown = true;
                    continue;
                };
                let base_upper = if let Ok(value) = forward_seed_upper
                    .checked_mul(usage_binding.activations_per_forward.upper)
                    .and_then(|bound| {
                        bound.checked_mul(usage_binding.attempts_per_activation.upper)
                    }) {
                    RecoveryActionUpper::Known(value)
                } else {
                    builder.arithmetic_violation = true;
                    leaf_violation = true;
                    RecoveryActionUpper::Overflow
                };
                let local_upper = if path.invalid_factor_identity {
                    RecoveryActionUpper::InvalidMultiplicity
                } else if !path.local_reachable {
                    RecoveryActionUpper::Known(OccurrenceCeiling::Finite(0))
                } else {
                    match base_upper {
                        RecoveryActionUpper::Known(value) => apply_multiplicity_factors(
                            value,
                            &path.local_factors,
                            &mut builder.factor_verdict,
                        )
                        .map_or(RecoveryActionUpper::Overflow, RecoveryActionUpper::Known),
                        other => other,
                    }
                };
                let root_upper = if path.invalid_factor_identity {
                    RecoveryActionUpper::InvalidMultiplicity
                } else if !path.root_reachable {
                    RecoveryActionUpper::Known(OccurrenceCeiling::Finite(0))
                } else {
                    match base_upper {
                        RecoveryActionUpper::Known(value) => apply_multiplicity_factors(
                            value,
                            &path.root_factors,
                            &mut builder.factor_verdict,
                        )
                        .map_or(RecoveryActionUpper::Overflow, RecoveryActionUpper::Known),
                        other => other,
                    }
                };
                if matches!(
                    local_upper,
                    RecoveryActionUpper::Overflow | RecoveryActionUpper::InvalidMultiplicity
                ) || matches!(
                    root_upper,
                    RecoveryActionUpper::Overflow | RecoveryActionUpper::InvalidMultiplicity
                ) {
                    leaf_violation = true;
                    builder.arithmetic_violation = true;
                }
                if forward.mapped_resource_slot.is_none()
                    || action_effect.mapped_resource_slot.is_none()
                {
                    builder.binding_verdict =
                        merge_verdict(builder.binding_verdict, LeafVerdict::Unproven);
                    builder.unresolved_sources.push(source.source_node());
                    unknown = true;
                }
                let forward_factors = applied_multiplicities(forward);
                let action_factors = applied_multiplicities(action_effect);
                if !multiset_equal(&path.root_factors, &forward_factors)
                    || !multiset_equal(&path.root_factors, &action_factors)
                {
                    builder.factor_verdict =
                        merge_verdict(builder.factor_verdict, LeafVerdict::Violation);
                    builder.arithmetic_violation = true;
                    leaf_violation = true;
                }
                let declared_action_upper =
                    if let Ok(upper) = scoped_occurrence_upper(action_effect) {
                        Some(upper)
                    } else {
                        builder.arithmetic_violation = true;
                        builder.action_epsilon_verdict =
                            merge_verdict(builder.action_epsilon_verdict, LeafVerdict::Violation);
                        leaf_violation = true;
                        None
                    };
                let required_epsilon_upper = match root_upper {
                    RecoveryActionUpper::Known(action_upper)
                        if AtomKey::of(&identity.forward_effect)
                            == AtomKey::of(&identity.action_effect) =>
                    {
                        let mut factor_verdict = LeafVerdict::Proven;
                        let forward_root_upper = if path.root_reachable {
                            apply_multiplicity_factors(
                                forward_seed_upper,
                                &path.root_factors,
                                &mut factor_verdict,
                            )
                        } else {
                            Ok(OccurrenceCeiling::Finite(0))
                        };
                        if let Ok(required) = forward_root_upper.and_then(|forward_upper| {
                            forward_upper.checked_add(action_upper).map_err(|_| ())
                        }) {
                            Some(required)
                        } else {
                            builder.arithmetic_violation = true;
                            builder.action_epsilon_verdict = merge_verdict(
                                builder.action_epsilon_verdict,
                                LeafVerdict::Violation,
                            );
                            leaf_violation = true;
                            None
                        }
                    }
                    RecoveryActionUpper::Known(action_upper) => Some(action_upper),
                    RecoveryActionUpper::Overflow
                    | RecoveryActionUpper::InvalidMultiplicity
                    | RecoveryActionUpper::Unknown => None,
                };
                match (required_epsilon_upper, declared_action_upper) {
                    (Some(required), Some(declared)) if !declared.covers(required) => {
                        builder.action_epsilon_verdict =
                            merge_verdict(builder.action_epsilon_verdict, LeafVerdict::Violation);
                        leaf_violation = true;
                    }
                    (Some(_), Some(_)) => {}
                    _ => {
                        builder.action_epsilon_verdict =
                            merge_verdict(builder.action_epsilon_verdict, LeafVerdict::Unproven);
                    }
                }
                let root_usage = match root_upper {
                    RecoveryActionUpper::Known(upper) => conditional_recovery_usage(
                        &usage_binding.per_attempt_usage,
                        upper,
                        builder.context,
                    )
                    .unwrap_or_else(|()| {
                        builder.arithmetic_violation = true;
                        StaticUsageEnvelope::unknown()
                    }),
                    RecoveryActionUpper::Overflow
                    | RecoveryActionUpper::InvalidMultiplicity
                    | RecoveryActionUpper::Unknown => StaticUsageEnvelope::unknown(),
                };
                let fold_seed_usage = match base_upper {
                    RecoveryActionUpper::Known(base) => conditional_recovery_usage(
                        &usage_binding.per_attempt_usage,
                        base,
                        builder.context,
                    )
                    .unwrap_or_else(|()| {
                        builder.arithmetic_violation = true;
                        StaticUsageEnvelope::unknown()
                    }),
                    RecoveryActionUpper::Overflow
                    | RecoveryActionUpper::InvalidMultiplicity
                    | RecoveryActionUpper::Unknown => StaticUsageEnvelope::unknown(),
                };
                if fold_seed_usage.is_globally_unknown() {
                    unknown = true;
                }
                let local_usage = match local_upper {
                    RecoveryActionUpper::Known(upper) => Some(
                        conditional_recovery_usage(
                            &usage_binding.per_attempt_usage,
                            upper,
                            builder.context,
                        )
                        .unwrap_or_else(|()| {
                            builder.arithmetic_violation = true;
                            StaticUsageEnvelope::unknown()
                        }),
                    ),
                    _ => None,
                };
                if let RecoveryActionUpper::Known(base) = base_upper {
                    let action_entry = EffectEntry {
                        atom: identity.action_effect.clone(),
                        bounds: OccurrenceBounds {
                            lower: 0,
                            upper: base,
                        },
                        guard: forward_source.declared.guard.clone(),
                    };
                    let singleton = EffectSignature {
                        row: EffectRow::Closed(vec![action_entry]),
                    };
                    if let Ok(next) = af_effect::control::sequence_envelope(&effect, &singleton) {
                        effect = next;
                    } else {
                        builder.arithmetic_violation = true;
                        leaf_violation = true;
                        unknown = true;
                    }
                    if let Ok(next) =
                        af_effect::control::sequence_envelope(&epsilon_requirement, &singleton)
                    {
                        epsilon_requirement = next;
                    } else {
                        builder.arithmetic_violation = true;
                        leaf_violation = true;
                        unknown = true;
                    }
                    if AtomKey::of(&forward_source.declared.atom)
                        == AtomKey::of(&identity.action_effect)
                        && !same_atom_forwards
                            .iter()
                            .any(|(occurrence, _)| occurrence == forward_source)
                    {
                        same_atom_forwards.push((forward_source.clone(), forward_seed_upper));
                    }
                    if fold_seed_usage.is_globally_unknown() {
                        unknown = true;
                    } else {
                        match usage.as_ref() {
                            None => usage = Some(fold_seed_usage.clone()),
                            Some(current) => {
                                if let Ok(next) =
                                    af_usage::control::sequence_envelope(current, &fold_seed_usage)
                                {
                                    usage = Some(next);
                                } else {
                                    builder.arithmetic_violation = true;
                                    leaf_violation = true;
                                    unknown = true;
                                }
                            }
                        }
                    }
                } else {
                    unknown = true;
                }
                let source_identity = RecoveryActionSourceIdentity {
                    caller_semantic_ir_digest: forward.caller_semantic_ir_digest.clone(),
                    source_node: source.source_node(),
                    source_contract_digest: forward.source_contract_digest.clone(),
                    forward_entry_ordinal: forward.source_entry_ordinal,
                    action: identity.clone(),
                };
                let local_bound = path.owner_scope.map(|scope| LocalActionBound {
                    compensation_scope_node: scope,
                    upper: local_upper,
                    applied_families: path.local_factors.clone(),
                });
                builder.actions.push(RootRecoveryAction {
                    source: source_identity,
                    forward_seed_upper,
                    target: source.target().clone(),
                    owner: path.owner_scope.map_or(
                        RootRecoveryOwner::RootRecovery,
                        RootRecoveryOwner::CompensationScope,
                    ),
                    local_bound,
                    root_bound: RootActionBound {
                        root_semantic_ir_digest: builder.root_digest.clone(),
                        upper: root_upper,
                        applied_families: path.root_factors.clone(),
                    },
                    forward_occurrence: forward.clone(),
                    action_effect_occurrence: action_effect.clone(),
                    recovery_usage_binding: usage_binding.clone(),
                    local_recovery_usage: local_usage,
                    root_recovery_usage: root_usage,
                    guard_compatibility: guard_proof,
                    guard_coordinates,
                    eligibility_evidence: match root_upper {
                        RecoveryActionUpper::Known(OccurrenceCeiling::Finite(0)) => {
                            LeafVerdict::Proven
                        }
                        RecoveryActionUpper::Known(_) => LeafVerdict::RequiresEvidence,
                        RecoveryActionUpper::Overflow
                        | RecoveryActionUpper::InvalidMultiplicity => LeafVerdict::Violation,
                        RecoveryActionUpper::Unknown => LeafVerdict::Unproven,
                    },
                    control_path: path.control_path.clone(),
                });
            }
        }
    }
    let usage = usage.unwrap_or_else(StaticUsageEnvelope::empty);
    for (forward, forward_seed_upper) in same_atom_forwards {
        let singleton = EffectSignature {
            row: EffectRow::Closed(vec![EffectEntry {
                atom: forward.declared.atom.clone(),
                bounds: OccurrenceBounds {
                    lower: 0,
                    upper: forward_seed_upper,
                },
                guard: forward.declared.guard.clone(),
            }]),
        };
        if let Ok(next) = af_effect::control::sequence_envelope(&epsilon_requirement, &singleton) {
            epsilon_requirement = next;
        } else {
            builder.arithmetic_violation = true;
            leaf_violation = true;
            unknown = true;
        }
    }
    builder.unresolved_sources.sort_unstable();
    builder.unresolved_sources.dedup();
    if unknown {
        RootRecoveryFold {
            action_effects: if effect.row == EffectRow::Closed(vec![]) {
                ControlDimensionValue::Unknown {
                    known_contributions: vec![],
                    unresolved_sources: unique_nodes(
                        builder
                            .unresolved_sources
                            .iter()
                            .copied()
                            .chain([summary.control_node]),
                    ),
                }
            } else {
                ControlDimensionValue::Partial {
                    known_contribution: Some(effect),
                    unresolved_sources: unique_nodes(
                        builder
                            .unresolved_sources
                            .iter()
                            .copied()
                            .chain([summary.control_node]),
                    ),
                }
            },
            epsilon_coverage_requirement: if epsilon_requirement.row == EffectRow::Closed(vec![]) {
                ControlDimensionValue::Unknown {
                    known_contributions: vec![],
                    unresolved_sources: unique_nodes(
                        builder
                            .unresolved_sources
                            .iter()
                            .copied()
                            .chain([summary.control_node]),
                    ),
                }
            } else {
                ControlDimensionValue::Partial {
                    known_contribution: Some(epsilon_requirement),
                    unresolved_sources: unique_nodes(
                        builder
                            .unresolved_sources
                            .iter()
                            .copied()
                            .chain([summary.control_node]),
                    ),
                }
            },
            recovery_usage: if usage.is_empty() {
                ControlDimensionValue::Unknown {
                    known_contributions: vec![],
                    unresolved_sources: unique_nodes(
                        builder
                            .unresolved_sources
                            .iter()
                            .copied()
                            .chain([summary.control_node]),
                    ),
                }
            } else {
                ControlDimensionValue::Partial {
                    known_contribution: Some(usage),
                    unresolved_sources: unique_nodes(
                        builder
                            .unresolved_sources
                            .iter()
                            .copied()
                            .chain([summary.control_node]),
                    ),
                }
            },
            violation: leaf_violation,
        }
    } else if leaf_violation {
        RootRecoveryFold {
            action_effects: ControlDimensionValue::Partial {
                known_contribution: Some(effect),
                unresolved_sources: vec![summary.control_node],
            },
            epsilon_coverage_requirement: ControlDimensionValue::Partial {
                known_contribution: Some(epsilon_requirement),
                unresolved_sources: vec![summary.control_node],
            },
            recovery_usage: ControlDimensionValue::Partial {
                known_contribution: Some(usage),
                unresolved_sources: vec![summary.control_node],
            },
            violation: true,
        }
    } else {
        RootRecoveryFold {
            action_effects: ControlDimensionValue::Complete(effect),
            epsilon_coverage_requirement: ControlDimensionValue::Complete(epsilon_requirement),
            recovery_usage: ControlDimensionValue::Complete(usage),
            violation: false,
        }
    }
}

fn root_guard_coordinate_evidence(
    leaf: &ResolvedComputationLeaf<'_, '_>,
    source_contract_digest: &DigestRefValue,
    guard_proof: Option<&ActionGuardCompatibility>,
    dataflow: &SemanticDataflowAnalysis,
    verdict: &mut LeafVerdict,
) -> Vec<RootGuardCoordinateEvidence> {
    let Some(guard_proof) = guard_proof else {
        *verdict = merge_verdict(*verdict, LeafVerdict::Unproven);
        return vec![];
    };
    let mut result = Vec::new();
    for guard in [
        guard_proof.forward_guard(),
        guard_proof.declared_action_guard(),
    ]
    .into_iter()
    .flatten()
    {
        let EffectGuard::InputEquals(equals) = guard else {
            continue;
        };
        if result.iter().any(|item: &RootGuardCoordinateEvidence| {
            item.component_input_port == equals.port_no
                && item.source_contract_digest == *source_contract_digest
        }) {
            continue;
        }
        let Some(mapping) = leaf
            .input_mappings()
            .iter()
            .find(|mapping| mapping.contract_port_no == equals.port_no)
        else {
            *verdict = merge_verdict(*verdict, LeafVerdict::Unproven);
            continue;
        };
        let ScopedPortBinding::InputExpression(expression) = mapping.binding else {
            *verdict = merge_verdict(*verdict, LeafVerdict::Unproven);
            continue;
        };
        let Some(exact_type) = mapping.exact_type.clone() else {
            *verdict = merge_verdict(*verdict, LeafVerdict::Unproven);
            continue;
        };
        let dependency_slots = dataflow
            .expression_value_dependencies()
            .get(&expression.raw())
            .map(|slots| slots.iter().copied().map(ValueSlotId::from_raw).collect())
            .unwrap_or_default();
        result.push(RootGuardCoordinateEvidence {
            source_contract_digest: (*source_contract_digest).clone(),
            component_input_port: equals.port_no,
            input_expression: expression,
            exact_type: Some(exact_type),
            dependency_slots,
        });
    }
    result
}

fn apply_multiplicity_factors(
    base: OccurrenceCeiling,
    factors: &[AppliedMultiplicity],
    verdict: &mut LeafVerdict,
) -> Result<OccurrenceCeiling, ()> {
    let mut seen = Vec::new();
    let mut result = base;
    for factor in factors {
        if seen.contains(&factor.identity) {
            *verdict = merge_verdict(*verdict, LeafVerdict::Violation);
            return Err(());
        }
        seen.push(factor.identity.clone());
        result = result
            .checked_mul(factor.logical_repetitions.upper)
            .map_err(|_| {
                *verdict = merge_verdict(*verdict, LeafVerdict::Violation);
            })?;
    }
    Ok(result)
}

fn find_root_occurrence<'a>(
    root_occurrences: &'a [ScopedEffectOccurrence],
    source_occurrence: &ScopedEffectOccurrence,
) -> Option<&'a ScopedEffectOccurrence> {
    root_occurrences.iter().find(|candidate| {
        candidate.caller_semantic_ir_digest == source_occurrence.caller_semantic_ir_digest
            && candidate.source_contract_digest == source_occurrence.source_contract_digest
            && candidate.computation == source_occurrence.computation
            && candidate.source_entry_ordinal == source_occurrence.source_entry_ordinal
            && candidate.declared.atom == source_occurrence.declared.atom
            && candidate.mapped_resource_slot == source_occurrence.mapped_resource_slot
    })
}

fn combine_root_recovery_children(
    summary: &DerivedControlContractSummary<'_, '_>,
    children: &[RootRecoveryFold],
    exact_select_child: Option<usize>,
    builder: &mut RootRecoveryBuilder<'_, '_, '_>,
) -> RootRecoveryFold {
    let violation = children.iter().any(|child| child.violation);
    if let Some(selected) = exact_select_child {
        let Some(child) = children.get(selected) else {
            builder.factor_verdict = merge_verdict(builder.factor_verdict, LeafVerdict::Violation);
            return RootRecoveryFold {
                action_effects: ControlDimensionValue::Unknown {
                    known_contributions: vec![],
                    unresolved_sources: vec![summary.control_node],
                },
                epsilon_coverage_requirement: ControlDimensionValue::Unknown {
                    known_contributions: vec![],
                    unresolved_sources: vec![summary.control_node],
                },
                recovery_usage: ControlDimensionValue::Unknown {
                    known_contributions: vec![],
                    unresolved_sources: vec![summary.control_node],
                },
                violation: true,
            };
        };
        return RootRecoveryFold {
            action_effects: child.action_effects.clone(),
            epsilon_coverage_requirement: child.epsilon_coverage_requirement.clone(),
            recovery_usage: child.recovery_usage.clone(),
            violation,
        };
    }

    let action_effects = combine_root_effect_children(summary, children, builder);
    let epsilon_children = children
        .iter()
        .map(|child| RootRecoveryFold {
            action_effects: child.epsilon_coverage_requirement.clone(),
            epsilon_coverage_requirement: child.epsilon_coverage_requirement.clone(),
            recovery_usage: child.recovery_usage.clone(),
            violation: child.violation,
        })
        .collect::<Vec<_>>();
    let epsilon_coverage_requirement =
        combine_root_effect_children(summary, &epsilon_children, builder);
    let recovery_usage = combine_root_usage_children(summary, children, builder);
    RootRecoveryFold {
        action_effects,
        epsilon_coverage_requirement,
        recovery_usage,
        violation: violation || builder.arithmetic_violation,
    }
}

fn combine_root_effect_children(
    summary: &DerivedControlContractSummary<'_, '_>,
    children: &[RootRecoveryFold],
    builder: &mut RootRecoveryBuilder<'_, '_, '_>,
) -> ControlDimensionValue<EffectSignature> {
    let mut signatures = Vec::new();
    let mut unresolved = Vec::new();
    for (index, child) in children.iter().enumerate() {
        match &child.action_effects {
            ControlDimensionValue::Complete(signature) => {
                signatures.push((
                    index,
                    summary.children[index].control_node,
                    signature.clone(),
                ));
            }
            ControlDimensionValue::Partial {
                known_contribution: Some(signature),
                unresolved_sources,
            } => {
                signatures.push((
                    index,
                    summary.children[index].control_node,
                    signature.clone(),
                ));
                unresolved.push(summary.children[index].control_node);
                unresolved.extend(unresolved_sources.iter().copied());
            }
            ControlDimensionValue::Unknown {
                known_contributions,
                unresolved_sources,
            } => {
                signatures.extend(
                    known_contributions
                        .iter()
                        .map(|item| (index, item.source_node(), item.value().clone())),
                );
                unresolved.push(summary.children[index].control_node);
                unresolved.extend(unresolved_sources.iter().copied());
            }
            ControlDimensionValue::Partial {
                known_contribution: None,
                unresolved_sources,
            }
            | ControlDimensionValue::PerChild {
                unresolved_sources, ..
            } => {
                unresolved.push(summary.children[index].control_node);
                unresolved.extend(unresolved_sources.iter().copied());
            }
            ControlDimensionValue::NotApplicable => {
                unresolved.push(summary.children[index].control_node);
            }
        }
    }
    unresolved.sort_unstable();
    unresolved.dedup();
    match combine_effect_signatures(&summary.operator, &signatures, None) {
        Ok(Some(signature)) if unresolved.is_empty() => ControlDimensionValue::Complete(signature),
        Ok(Some(signature)) => ControlDimensionValue::Partial {
            known_contribution: Some(signature),
            unresolved_sources: unresolved,
        },
        Ok(None) => ControlDimensionValue::Unknown {
            known_contributions: signatures
                .into_iter()
                .map(|(_, source_node, value)| DimensionContribution { source_node, value })
                .collect(),
            unresolved_sources: if unresolved.is_empty() {
                vec![summary.control_node]
            } else {
                unresolved
            },
        },
        Err(_) => {
            builder.arithmetic_violation = true;
            ControlDimensionValue::Unknown {
                known_contributions: signatures
                    .into_iter()
                    .map(|(_, source_node, value)| DimensionContribution { source_node, value })
                    .collect(),
                unresolved_sources: unique_nodes(
                    unresolved.into_iter().chain([summary.control_node]),
                ),
            }
        }
    }
}

fn combine_root_usage_children(
    summary: &DerivedControlContractSummary<'_, '_>,
    children: &[RootRecoveryFold],
    builder: &mut RootRecoveryBuilder<'_, '_, '_>,
) -> ControlDimensionValue<StaticUsageEnvelope> {
    let mut envelopes = Vec::new();
    let mut unresolved = Vec::new();
    for (index, child) in children.iter().enumerate() {
        match &child.recovery_usage {
            ControlDimensionValue::Complete(envelope) => {
                envelopes.push((
                    index,
                    summary.children[index].control_node,
                    envelope.clone(),
                ));
            }
            ControlDimensionValue::Partial {
                known_contribution: Some(envelope),
                unresolved_sources,
            } => {
                envelopes.push((
                    index,
                    summary.children[index].control_node,
                    envelope.clone(),
                ));
                unresolved.push(summary.children[index].control_node);
                unresolved.extend(unresolved_sources.iter().copied());
            }
            ControlDimensionValue::Unknown {
                known_contributions,
                unresolved_sources,
            } => {
                envelopes.extend(
                    known_contributions
                        .iter()
                        .map(|item| (index, item.source_node(), item.value().clone())),
                );
                unresolved.push(summary.children[index].control_node);
                unresolved.extend(unresolved_sources.iter().copied());
            }
            ControlDimensionValue::Partial {
                known_contribution: None,
                unresolved_sources,
            }
            | ControlDimensionValue::PerChild {
                unresolved_sources, ..
            } => {
                unresolved.push(summary.children[index].control_node);
                unresolved.extend(unresolved_sources.iter().copied());
            }
            ControlDimensionValue::NotApplicable => {
                unresolved.push(summary.children[index].control_node);
            }
        }
    }
    unresolved.sort_unstable();
    unresolved.dedup();
    envelopes.retain(|(_, _, envelope)| !envelope.is_empty());
    if envelopes.is_empty() && unresolved.is_empty() {
        return ControlDimensionValue::Complete(StaticUsageEnvelope::empty());
    }
    match compose_usage(&summary.operator, &envelopes) {
        Ok(Some(envelope)) if unresolved.is_empty() => ControlDimensionValue::Complete(envelope),
        Ok(Some(envelope)) => ControlDimensionValue::Partial {
            known_contribution: Some(envelope),
            unresolved_sources: unresolved,
        },
        Ok(None) => ControlDimensionValue::Unknown {
            known_contributions: envelopes
                .into_iter()
                .map(|(_, source_node, value)| DimensionContribution { source_node, value })
                .collect(),
            unresolved_sources: if unresolved.is_empty() {
                vec![summary.control_node]
            } else {
                unresolved
            },
        },
        Err(_) => {
            builder.arithmetic_violation = true;
            ControlDimensionValue::Unknown {
                known_contributions: envelopes
                    .into_iter()
                    .map(|(_, source_node, value)| DimensionContribution { source_node, value })
                    .collect(),
                unresolved_sources: unique_nodes(
                    unresolved.into_iter().chain([summary.control_node]),
                ),
            }
        }
    }
}

fn root_action_effect_coverage(
    actions: &ControlDimensionValue<EffectSignature>,
    root_effects: &ControlDimensionValue<EffectSignature>,
) -> LeafVerdict {
    let ControlDimensionValue::Complete(actions) = actions else {
        return LeafVerdict::Unproven;
    };
    let action_entries = match &actions.row {
        EffectRow::Closed(entries) => entries,
        EffectRow::Open(open) if open.residual.is_empty() => &open.known,
        EffectRow::Open(_) | EffectRow::Unknown => return LeafVerdict::Unproven,
    };
    if action_entries.is_empty() {
        return LeafVerdict::Proven;
    }
    let ControlDimensionValue::Complete(root_effects) = root_effects else {
        return LeafVerdict::Unproven;
    };
    let (root_entries, root_residuals) = match &root_effects.row {
        EffectRow::Closed(entries) => (entries.as_slice(), &[][..]),
        EffectRow::Open(open) => (open.known.as_slice(), open.residual.as_slice()),
        EffectRow::Unknown => return LeafVerdict::Unproven,
    };
    let mut verdict = LeafVerdict::Proven;
    for action in action_entries
        .iter()
        .filter(|entry| !entry.bounds.upper.is_zero())
    {
        if let Some(declared) = root_entries
            .iter()
            .find(|entry| AtomKey::of(&entry.atom) == AtomKey::of(&action.atom))
        {
            if !declared.bounds.upper.covers(action.bounds.upper) {
                verdict = merge_verdict(verdict, LeafVerdict::Violation);
            } else if !af_effect::guard_contained(&action.guard, &declared.guard) {
                verdict = merge_verdict(verdict, LeafVerdict::Unproven);
            }
            continue;
        }
        if matches!(&root_effects.row, EffectRow::Closed(_)) {
            verdict = merge_verdict(verdict, LeafVerdict::Violation);
            continue;
        }
        let residual_covers = root_residuals.iter().any(|residual| {
            residual.catalog == action.atom.operation.catalog
                && residual.selector.requirement_port_no
                    == action.atom.resource_selector.requirement_port_no
                && residual.bounds.upper.covers(action.bounds.upper)
                && scope_contains(
                    &action.atom.resource_selector.scope_bindings,
                    &residual.selector.scope_bindings,
                )
        });
        if !residual_covers {
            verdict = merge_verdict(verdict, LeafVerdict::Unproven);
        }
    }
    verdict
}

fn scope_contains(inner: &[af_effect::ScopeBinding], outer: &[af_effect::ScopeBinding]) -> bool {
    outer.iter().all(|binding| inner.contains(binding))
}

fn root_recovery_usage_coverage(
    required: &ControlDimensionValue<StaticUsageEnvelope>,
    declared: &ControlDimensionValue<StaticUsageEnvelope>,
) -> LeafVerdict {
    let ControlDimensionValue::Complete(required) = required else {
        return LeafVerdict::Unproven;
    };
    if required.is_globally_unknown() {
        return LeafVerdict::Unproven;
    }
    if required.is_empty() {
        return LeafVerdict::Proven;
    }
    let ControlDimensionValue::Complete(declared) = declared else {
        return LeafVerdict::Unproven;
    };
    if declared.is_globally_unknown() {
        return LeafVerdict::Unproven;
    }
    let mut verdict = LeafVerdict::Proven;
    for (coordinate, required_bound) in required.iter() {
        if coordinate.phase != UsagePhase::Recovery {
            verdict = merge_verdict(verdict, LeafVerdict::Violation);
            continue;
        }
        let declared_bound = declared.get(coordinate);
        match (required_bound, declared_bound) {
            (StaticUsageBound::KnownFinite(required), StaticUsageBound::KnownFinite(declared))
                if required > &declared =>
            {
                verdict = merge_verdict(verdict, LeafVerdict::Violation);
            }
            (
                StaticUsageBound::KnownFinite(_),
                StaticUsageBound::KnownFinite(_) | StaticUsageBound::KnownUnbounded,
            )
            | (StaticUsageBound::KnownUnbounded, StaticUsageBound::KnownUnbounded) => {}
            (StaticUsageBound::KnownUnbounded, StaticUsageBound::KnownFinite(_)) => {
                verdict = merge_verdict(verdict, LeafVerdict::Violation);
            }
            _ => verdict = merge_verdict(verdict, LeafVerdict::Unproven),
        }
    }
    verdict
}

fn exact_select_child_index(summary: &DerivedControlContractSummary<'_, '_>) -> Option<usize> {
    let selector = summary.interface.selector()?;
    let exact_case = selector.exact_case()?;
    selector
        .case_ordinals()
        .iter()
        .position(|ordinal| *ordinal == exact_case)
}

fn control_path_step(
    parent: &DerivedControlContractSummary<'_, '_>,
    child_index: usize,
    child_node: ComputationId,
    reachability: ControlPathReachability,
) -> ControlPathStep {
    let kind = match &parent.operator {
        ControlOperator::Select => {
            let evidence = parent.interface.selector();
            let case_ordinal = evidence
                .and_then(|item| item.case_ordinals().get(child_index).copied())
                .unwrap_or_else(|| u32::try_from(child_index).unwrap_or(u32::MAX));
            ControlPathKind::SelectCase {
                case_ordinal,
                selector: evidence.map(SelectDecisionEvidence::expression),
                selector_dependencies: evidence
                    .map(|item| item.dependency_slots().to_vec())
                    .unwrap_or_default(),
                exact_case: evidence.and_then(SelectDecisionEvidence::exact_case),
            }
        }
        ControlOperator::Parallel { .. } => ControlPathKind::ParallelBranch {
            branch_ordinal: u32::try_from(child_index + 1).unwrap_or(u32::MAX),
        },
        ControlOperator::RaceFirstTerminal => ControlPathKind::RaceBranch {
            branch_ordinal: u32::try_from(child_index + 1).unwrap_or(u32::MAX),
        },
        ControlOperator::Iterate { .. } => ControlPathKind::IterateBody,
        ControlOperator::Retry { max_attempts, .. } => ControlPathKind::RetryChild {
            max_attempts: *max_attempts,
        },
        ControlOperator::Timeout { .. } => ControlPathKind::TimeoutChild,
        ControlOperator::Handle { clauses, .. } if child_index == 0 => {
            ControlPathKind::HandleProtected
        }
        ControlOperator::Handle { clauses, .. } => ControlPathKind::HandleClause {
            ordinal: clauses.get(child_index - 1).map_or_else(
                || u32::try_from(child_index).unwrap_or(u32::MAX),
                HandleClauseSummary::ordinal,
            ),
        },
        ControlOperator::Scope => ControlPathKind::ScopeChild,
        ControlOperator::CompensationScope => ControlPathKind::CompensationScopeChild,
        _ => ControlPathKind::SequenceChild,
    };
    ControlPathStep {
        parent_node: parent.control_node,
        child_node,
        child_index,
        kind,
        reachability,
    }
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
        ComputationBody::Iterate(IterateBody::Foreach(body)) => {
            let known_count = document
                .expressions
                .iter()
                .find(|expression| expression.id == body.source)
                .and_then(|expression| match &expression.body {
                    ExpressionBody::ListConstruct { elements, .. } => {
                        u64::try_from(elements.len()).ok()
                    }
                    _ => None,
                });
            let upper = known_count.map_or(
                af_effect::OccurrenceCeiling::Unbounded,
                af_effect::OccurrenceCeiling::Finite,
            );
            (
                ControlOperator::Iterate {
                    mode: IterateMode::Foreach {
                        statically_known_item_count: known_count,
                        max_inflight: body.max_inflight,
                    },
                    effect_repetitions: OccurrenceBounds { lower: 0, upper },
                    usage_repetitions: known_count,
                },
                vec![body.body],
                None,
            )
        }
        ComputationBody::Iterate(IterateBody::While(body)) => (
            ControlOperator::Iterate {
                mode: IterateMode::While {
                    max_iterations: body.max_iterations,
                },
                effect_repetitions: OccurrenceBounds {
                    lower: 0,
                    upper: af_effect::OccurrenceCeiling::Finite(u64::from(body.max_iterations)),
                },
                usage_repetitions: Some(u64::from(body.max_iterations)),
            },
            vec![body.body],
            None,
        ),
        ComputationBody::Retry(body) => {
            let retry_count = u64::try_from(body.delays.len())
                .map_err(|_| ControlDerivationError::ArithmeticOverflow("retry"))?;
            let max_attempts = retry_count
                .checked_add(1)
                .ok_or(ControlDerivationError::ArithmeticOverflow("retry"))?;
            (
                ControlOperator::Retry {
                    delays: body.delays.clone(),
                    max_attempts,
                },
                vec![body.child],
                None,
            )
        }
        ComputationBody::Timeout(body) => {
            let duration = document
                .expressions
                .iter()
                .find(|expression| expression.id == body.timeout)
                .and_then(|expression| match &expression.body {
                    ExpressionBody::Constant(PrimitiveConstant::Duration(value)) => Some(*value),
                    _ => None,
                });
            (
                ControlOperator::Timeout {
                    expression: body.timeout,
                    static_duration: duration,
                    possible_outcomes: [TimeoutOutcome::ChildCompleted, TimeoutOutcome::TimedOut],
                },
                vec![body.child],
                None,
            )
        }
        ComputationBody::Handle(body) => {
            let clauses = body
                .clauses
                .iter()
                .map(|clause| HandleClauseSummary {
                    ordinal: clause.ordinal.raw(),
                    matcher: match &clause.matcher {
                        HandleMatch::Failure(failure) => {
                            HandleMatcherSummary::Failure(failure.failure_type.clone())
                        }
                        HandleMatch::TimedOut(()) => HandleMatcherSummary::TimedOut,
                    },
                    handler: clause.handler,
                })
                .collect::<Vec<_>>();
            let handled = FailureRow {
                failures: body
                    .clauses
                    .iter()
                    .filter_map(|clause| match &clause.matcher {
                        HandleMatch::Failure(failure) => Some(af_failure::FailureTypeRef {
                            payload_type: failure.failure_type.clone(),
                        }),
                        HandleMatch::TimedOut(()) => None,
                    })
                    .collect(),
            };
            let mut children = vec![body.child];
            children.extend(body.clauses.iter().map(|clause| clause.handler));
            (ControlOperator::Handle { handled, clauses }, children, None)
        }
        ComputationBody::Scope(body) => (ControlOperator::Scope, vec![body.child], None),
        ComputationBody::CompensationScope(body) => {
            (ControlOperator::CompensationScope, vec![body.child], None)
        }
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
    let (scope_boundary, scope_proofs) =
        derive_scope_boundary(computation, &operator, &children, dataflow);
    let own_proofs = base_proofs(node_id, digest);
    let mut proofs = own_proofs;
    proofs.extend(output_proofs);
    proofs.extend(scope_proofs);
    if let ControlOperator::Timeout {
        possible_outcomes, ..
    } = &operator
    {
        proofs.push(ControlDimensionProof::new(
            "timeout-control-outcomes",
            LeafVerdict::Proven,
            RULE_TIMEOUT_OUTCOMES,
            node_id,
            Some(format!("computation:{}", node_id.raw())),
            format!(
                "the child-completed and timed-out outcomes are represented separately from the typed Failure row; no successful cancellation or zero-effect conclusion is inferred ({possible_outcomes:?})"
            ),
        ));
    }
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
        &control_decision_expressions(&computation.body),
        document,
        dataflow,
    );
    proofs.extend(flow_result.proofs.clone());
    let (resources, resource_aliases, resource_proofs) =
        derive_resource_dimension(node_id, &children);
    proofs.extend(resource_proofs);
    let (recovery, recovery_proofs) = derive_recovery_dimension(
        node_id,
        &operator,
        &children,
        &effect_result.occurrences,
        context,
    );
    proofs.extend(recovery_proofs);
    let (compensation_scope, compensation_proofs) =
        derive_compensation_scope(node_id, &operator, &children, context);
    proofs.extend(compensation_proofs);
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
        scope_boundary,
        compensation_scope,
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
        scope_boundary: None,
        compensation_scope: None,
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
        scope_boundary: None,
        compensation_scope: None,
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
        control_decision_dependencies: vec![],
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
            action_guard_compatibility: leaf.action_guard_compatibility().to_vec(),
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
        (ComputationBody::Iterate(body), ControlOperator::Iterate { .. }) => {
            let (body_node, carry) = match body {
                IterateBody::Foreach(body) => (body.body, &body.carry),
                IterateBody::While(body) => (body.body, &body.carry),
            };
            if let Some(child) = children
                .iter()
                .find(|child| child.control_node == body_node)
            {
                for parent in &computation.outputs {
                    let Some(binding) = carry
                        .iter()
                        .find(|binding| binding.body_output_port.raw() == parent.port_no.raw())
                    else {
                        proof = downgrade(
                            proof,
                            LeafVerdict::Unproven,
                            "no exact carry/body-output binding matches this Iterate output port",
                        );
                        continue;
                    };
                    if let Some(child_port) =
                        interface_output(child, binding.body_output_port.raw())
                    {
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
                            "the carry's loop-body output interface is unavailable",
                        );
                    }
                }
            } else {
                proof = downgrade(
                    proof,
                    LeafVerdict::Unproven,
                    "the owned loop body summary is unavailable",
                );
            }
        }
        (ComputationBody::Retry(_), ControlOperator::Retry { .. })
        | (ComputationBody::Timeout(_), ControlOperator::Timeout { .. })
        | (ComputationBody::Scope(_), ControlOperator::Scope)
        | (ComputationBody::CompensationScope(_), ControlOperator::CompensationScope) => {
            let Some(child) = children.first() else {
                proof = downgrade(
                    proof,
                    LeafVerdict::Unproven,
                    "the wrapper's owned child summary is unavailable",
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
                        "the wrapper child output evidence is unavailable",
                    );
                }
            }
        }
        (ComputationBody::Handle(_), ControlOperator::Handle { .. }) => {
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
                            "a protected or handler output proof is unavailable",
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
        let mut child_occurrences = child.effect_occurrences.clone();
        if let Some(family) = effect_occurrence_family(node, operator) {
            for occurrence in &mut child_occurrences {
                occurrence.control_families.push(family.clone());
            }
        }
        occurrences.extend(child_occurrences);
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
    if let ControlOperator::Retry { .. } = operator {
        let needs_narrowing_evidence = match &value {
            ControlDimensionValue::Complete(signature) => signature_has_possible_effects(signature),
            _ => true,
        };
        proofs.push(ControlDimensionProof::new(
            "retry-logical-effect-narrowing",
            if needs_narrowing_evidence {
                LeafVerdict::RequiresEvidence
            } else {
                LeafVerdict::Proven
            },
            RULE_RETRY_ENVELOPE,
            node,
            Some(format!("computation:{}", node.raw())),
            "the logical Effect upper envelope is conservatively scaled by the maximum physical attempts; declared idempotency alone does not narrow it, and replay/recovery evidence is required for a smaller logical bound",
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

fn signature_has_possible_effects(signature: &EffectSignature) -> bool {
    match &signature.row {
        EffectRow::Closed(entries) => entries.iter().any(|entry| !entry.bounds.upper.is_zero()),
        EffectRow::Open(open) => {
            open.known.iter().any(|entry| !entry.bounds.upper.is_zero())
                || open
                    .residual
                    .iter()
                    .any(|entry| !entry.bounds.upper.is_zero())
        }
        EffectRow::Unknown => true,
    }
}

fn effect_rule(operator: &ControlOperator) -> &'static str {
    match operator {
        ControlOperator::Sequence => RULE_SEQUENCE_EFFECT,
        ControlOperator::Select => RULE_SELECT_EFFECT,
        ControlOperator::Parallel { .. } => RULE_PARALLEL_EFFECT_USAGE,
        ControlOperator::RaceFirstTerminal => RULE_RACE_EFFECT_USAGE,
        ControlOperator::Iterate { .. } => RULE_ITERATION_ENVELOPE,
        ControlOperator::Retry { .. } => RULE_RETRY_ENVELOPE,
        ControlOperator::Timeout { .. } => RULE_TIMEOUT_OUTCOMES,
        ControlOperator::Handle { .. } => RULE_HANDLE_RESIDUAL,
        ControlOperator::Scope => RULE_SCOPE_RESOURCE_BOUNDARY,
        ControlOperator::CompensationScope => RULE_COMPENSATION_FORWARD_HISTORY,
        _ => RULE_UNSUPPORTED_CONTROL,
    }
}

fn effect_occurrence_family(
    node: ComputationId,
    operator: &ControlOperator,
) -> Option<EffectOccurrenceFamily> {
    match operator {
        ControlOperator::Iterate {
            effect_repetitions, ..
        } => Some(EffectOccurrenceFamily {
            control_node: node,
            kind: EffectOccurrenceFamilyKind::Iterate,
            logical_repetitions: effect_repetitions.clone(),
            physical_attempts: None,
        }),
        ControlOperator::Retry { max_attempts, .. } => {
            let attempts = OccurrenceBounds {
                lower: 1,
                upper: af_effect::OccurrenceCeiling::Finite(*max_attempts),
            };
            Some(EffectOccurrenceFamily {
                control_node: node,
                kind: EffectOccurrenceFamilyKind::Retry,
                logical_repetitions: attempts.clone(),
                physical_attempts: Some(attempts),
            })
        }
        _ => None,
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
        ControlOperator::Iterate {
            effect_repetitions, ..
        } => signatures
            .first()
            .map(|(_, _, signature)| {
                af_effect::control::iterate_envelope(effect_repetitions, signature)
            })
            .transpose(),
        ControlOperator::Retry { max_attempts, .. } => signatures
            .first()
            .map(|(_, _, signature)| {
                af_effect::control::retry_envelope(signature, *max_attempts)
                    .map(|envelope| envelope.logical)
            })
            .transpose(),
        ControlOperator::Timeout { .. } => Ok(signatures
            .first()
            .map(|(_, _, signature)| signature.clone())),
        ControlOperator::Scope | ControlOperator::CompensationScope => Ok(signatures
            .first()
            .map(|(_, _, signature)| signature.clone())),
        ControlOperator::Handle { .. } => {
            let Some(protected) = signatures.iter().find(|(index, _, _)| *index == 0) else {
                return Ok(None);
            };
            let mut handlers = signatures
                .iter()
                .filter(|(index, _, _)| *index > 0)
                .map(|(_, _, signature)| signature.clone())
                .collect::<Vec<_>>();
            handlers.push(EffectSignature {
                row: EffectRow::Closed(Vec::new()),
            });
            let mut iter = handlers.into_iter();
            let Some(first) = iter.next() else {
                return Ok(Some(protected.2.clone()));
            };
            let mut handler_join = first;
            for next in iter {
                handler_join = af_effect::control::select_envelope(&handler_join, &next)?;
            }
            Ok(Some(af_effect::control::sequence_envelope(
                &protected.2,
                &handler_join,
            )?))
        }
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
    let composed = compose_failure_rows(operator, &rows);
    let composition_failed = composed.is_err();
    let known_composed = composed.ok().flatten();
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
    } else if composition_failed {
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
            LeafVerdict::Violation,
            "HANDLE names an exact Failure type absent from the protected child's closed Failure row; the invalid removal is not applied",
        )
    } else if unresolved.is_empty() && rows.len() == children.len() {
        (
            ControlDimensionValue::Complete(known_composed.unwrap_or_else(FailureRow::empty)),
            LeafVerdict::Proven,
            "the existing Failure control algebra conservatively unions all child rows that may contribute; an empty row does not guarantee success",
        )
    } else if let Some(known) = known_composed {
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
        failure_rule(operator),
        node,
        Some(format!("computation:{}", node.raw())),
        reason,
    );
    ControlDimension {
        value,
        proofs: vec![proof],
    }
}

fn compose_failure_rows(
    operator: &ControlOperator,
    indexed_rows: &[(usize, ComputationId, FailureRow)],
) -> Result<Option<FailureRow>, af_failure::FailureError> {
    let rows = indexed_rows
        .iter()
        .map(|(_, _, row)| row.clone())
        .collect::<Vec<_>>();
    match operator {
        ControlOperator::Sequence => Ok(Some(af_failure::control::sequence_row(&rows))),
        ControlOperator::Select => Ok(Some(af_failure::control::select_row(&rows))),
        ControlOperator::Parallel { .. } => Ok(Some(af_failure::control::parallel_row(&rows))),
        ControlOperator::RaceFirstTerminal => Ok(Some(af_failure::control::race_row(&rows))),
        ControlOperator::Iterate { .. } => Ok(rows.first().map(af_failure::control::iterate_row)),
        ControlOperator::Retry { .. } => Ok(rows.first().map(af_failure::control::retry_row)),
        ControlOperator::Timeout { .. } => Ok(rows.first().map(af_failure::control::timeout_row)),
        ControlOperator::Scope => Ok(rows.first().map(af_failure::control::scope_row)),
        ControlOperator::CompensationScope => Ok(rows
            .first()
            .map(af_failure::control::compensation_scope_row)),
        ControlOperator::Handle { handled, .. } => {
            let Some((_, _, protected)) = indexed_rows.iter().find(|(index, _, _)| *index == 0)
            else {
                return Ok(None);
            };
            let handlers = indexed_rows
                .iter()
                .filter(|(index, _, _)| *index > 0)
                .map(|(_, _, row)| row.clone())
                .collect::<Vec<_>>();
            af_failure::control::handle_row(protected, handled, &handlers).map(Some)
        }
        _ => Ok(None),
    }
}

fn failure_rule(operator: &ControlOperator) -> &'static str {
    match operator {
        ControlOperator::Iterate { .. } => RULE_ITERATION_ENVELOPE,
        ControlOperator::Retry { .. } => RULE_RETRY_ENVELOPE,
        ControlOperator::Timeout { .. } => RULE_TIMEOUT_OUTCOMES,
        ControlOperator::Handle { .. } => RULE_HANDLE_RESIDUAL,
        ControlOperator::Scope => RULE_SCOPE_RUNTIME_OBLIGATIONS,
        ControlOperator::CompensationScope => RULE_COMPENSATION_FORWARD_HISTORY,
        _ => RULE_FAILURE_PLANE,
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
        usage_rule(operator),
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
        ControlOperator::Iterate {
            usage_repetitions, ..
        } => {
            let Some((_, _, body)) = ordered.first() else {
                return Ok(None);
            };
            af_usage::control::iterate_envelope(*usage_repetitions, body).map(Some)
        }
        ControlOperator::Retry { max_attempts, .. } => {
            let Some((_, _, body)) = ordered.first() else {
                return Ok(None);
            };
            af_usage::control::retry_envelope(body, *max_attempts)
                .map(|envelope| Some(envelope.logical))
        }
        ControlOperator::Timeout { .. } => Ok(ordered
            .first()
            .map(|(_, _, body)| af_usage::control::timeout_envelope(body))),
        ControlOperator::Scope => Ok(ordered
            .first()
            .map(|(_, _, body)| af_usage::control::scope_envelope(body))),
        ControlOperator::CompensationScope => Ok(ordered.first().map(|(_, _, body)| body.clone())),
        ControlOperator::Handle { .. } => {
            let Some((_, _, protected)) = ordered.iter().find(|(index, _, _)| *index == 0) else {
                return Ok(None);
            };
            let mut handlers = ordered
                .iter()
                .filter(|(index, _, _)| *index > 0)
                .map(|(_, _, envelope)| envelope.clone())
                .collect::<Vec<_>>();
            let handler_coordinates = handlers
                .iter()
                .flat_map(StaticUsageEnvelope::coordinates)
                .collect::<BTreeSet<_>>();
            let mut no_handler = StaticUsageEnvelope::empty();
            for coordinate in handler_coordinates {
                no_handler.insert(coordinate, af_usage::StaticUsageBound::KnownFinite(0))?;
            }
            handlers.push(no_handler);
            let handler_join = af_usage::control::select_envelope(&handlers)?;
            af_usage::control::sequence_envelope(protected, &handler_join).map(Some)
        }
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

fn usage_rule(operator: &ControlOperator) -> &'static str {
    match operator {
        ControlOperator::Iterate { .. } => RULE_ITERATION_ENVELOPE,
        ControlOperator::Retry { .. } => RULE_RETRY_ENVELOPE,
        ControlOperator::Timeout { .. } => RULE_TIMEOUT_OUTCOMES,
        ControlOperator::Handle { .. } => RULE_HANDLE_RESIDUAL,
        ControlOperator::Scope => RULE_SCOPE_RUNTIME_OBLIGATIONS,
        ControlOperator::CompensationScope => RULE_COMPENSATION_USAGE_ORDER,
        _ => RULE_USAGE_PROPAGATION,
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
        ComputationBody::Iterate(IterateBody::Foreach(body)) => std::iter::once(body.source)
            .chain(body.carry.iter().map(|carry| carry.initial))
            .collect(),
        ComputationBody::Iterate(IterateBody::While(body)) => std::iter::once(body.guard)
            .chain(body.carry.iter().map(|carry| carry.initial))
            .collect(),
        ComputationBody::Timeout(body) => vec![body.timeout],
        _ => vec![],
    }
}

fn control_decision_expressions(body: &ComputationBody) -> Vec<ExpressionId> {
    match body {
        ComputationBody::Iterate(IterateBody::Foreach(body)) => vec![body.source],
        ComputationBody::Iterate(IterateBody::While(body)) => vec![body.guard],
        ComputationBody::Timeout(body) => vec![body.timeout],
        _ => vec![],
    }
}

#[allow(clippy::too_many_arguments)]
fn derive_flow_dimension(
    node: ComputationId,
    operator: &ControlOperator,
    children: &[DerivedControlContractSummary<'_, '_>],
    sequence_dependencies: &[SequenceDataflowDependency],
    select_selector: Option<ExpressionId>,
    control_decision_expressions: &[ExpressionId],
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
    control_decision_unknown |= matches!(
        operator,
        ControlOperator::Iterate { .. }
            | ControlOperator::Retry { .. }
            | ControlOperator::Timeout { .. }
            | ControlOperator::Handle { .. }
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
    let mut control_decision_dependencies = control_decision_expressions
        .iter()
        .filter_map(|expression| {
            dataflow
                .expression_value_dependencies()
                .get(&expression.raw())
        })
        .flatten()
        .copied()
        .map(ValueSlotId::from_raw)
        .collect::<Vec<_>>();
    control_decision_dependencies.extend(
        children
            .iter()
            .filter_map(|child| match &child.information_flow.value {
                ControlDimensionValue::Complete(flow)
                | ControlDimensionValue::Partial {
                    known_contribution: Some(flow),
                    ..
                } => Some(flow.control_decision_dependencies.iter().copied()),
                _ => None,
            })
            .flatten(),
    );
    control_decision_dependencies.sort_unstable();
    control_decision_dependencies.dedup();
    control_decision_dependencies.extend(select_decision_dependencies.iter().copied());
    control_decision_dependencies.sort_unstable();
    control_decision_dependencies.dedup();
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
        control_decision_dependencies,
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
        "iteration guards/items, retry failure decisions, timeout completion, handler matches, fail-fast cancellation or race winner may influence outcomes; value dependencies and all child may-influence facts remain, with no physical outcome selected"
    } else {
        "child may-influence summaries are conservatively merged; per-child Flow constraints and EffectFlowPartitions remain source-scoped"
    };
    let proof = ControlDimensionProof::new(
        "information-flow",
        verdict,
        flow_rule(operator),
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

fn flow_rule(operator: &ControlOperator) -> &'static str {
    match operator {
        ControlOperator::Iterate { .. } => RULE_ITERATION_ENVELOPE,
        ControlOperator::Retry { .. } => RULE_RETRY_ENVELOPE,
        ControlOperator::Timeout { .. } => RULE_TIMEOUT_OUTCOMES,
        ControlOperator::Handle { .. } => RULE_HANDLE_RESIDUAL,
        ControlOperator::Scope => RULE_SCOPE_RUNTIME_OBLIGATIONS,
        ControlOperator::CompensationScope => RULE_COMPENSATION_FORWARD_HISTORY,
        _ => RULE_FLOW_PROPAGATION,
    }
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

fn derive_scope_boundary(
    computation: &ComputationDef,
    operator: &ControlOperator,
    children: &[DerivedControlContractSummary<'_, '_>],
    dataflow: &SemanticDataflowAnalysis,
) -> (Option<ScopeBoundarySummary>, Vec<ControlDimensionProof>) {
    let (ComputationBody::Scope(body), ControlOperator::Scope) = (&computation.body, operator)
    else {
        return (None, vec![]);
    };
    let parent = dataflow
        .entry_resources()
        .get(&computation.id.raw())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(ResourceSlotId::from_raw)
        .collect::<Vec<_>>();
    let declared = body.resources.clone();
    let child_visible = dataflow
        .entry_resources()
        .get(&body.child.raw())
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(ResourceSlotId::from_raw)
        .collect::<Vec<_>>();
    let parent_set = parent.iter().copied().collect::<BTreeSet<_>>();
    let declared_set = declared.iter().copied().collect::<BTreeSet<_>>();
    let child_set = child_visible.iter().copied().collect::<BTreeSet<_>>();
    let resource_attenuation = if parent_set.is_superset(&declared_set) && declared_set == child_set
    {
        LeafVerdict::Proven
    } else {
        LeafVerdict::Violation
    };
    let summary = ScopeBoundarySummary {
        control_node: computation.id,
        child: body.child,
        parent_visible_resources: parent,
        declared_resources: declared,
        child_visible_resources: child_visible,
        resource_attenuation,
        // The frozen ScopeBody contains no authority or budget ceiling.
        authority_attenuation: LeafVerdict::Unproven,
        budget_attenuation: LeafVerdict::Unproven,
        // Phase 3B proves ownership, not actual child terminal state.
        owned_children_settle_before_terminal: LeafVerdict::RequiresEvidence,
    };
    let child_available = children
        .iter()
        .any(|child| child.control_node == body.child);
    let mut proofs = vec![
        ControlDimensionProof::new(
            "scope-resource-attenuation",
            resource_attenuation,
            RULE_SCOPE_RESOURCE_BOUNDARY,
            computation.id,
            Some(format!("computation:{}", body.child.raw())),
            "the checked Phase 3C entry-resource environment is compared with the exact frozen SCOPE.resources set; this proves lexical visibility only, not a ResourceHandle",
        ),
        ControlDimensionProof::new(
            "scope-authority-attenuation",
            LeafVerdict::Unproven,
            RULE_SCOPE_RUNTIME_OBLIGATIONS,
            computation.id,
            Some(format!("computation:{}", computation.id.raw())),
            "ScopeBody has no authority ceiling and no RunAuthorityEnvelope or DelegationGrant is an input; no effective grant is inferred",
        ),
        ControlDimensionProof::new(
            "scope-budget-attenuation",
            LeafVerdict::Unproven,
            RULE_SCOPE_RUNTIME_OBLIGATIONS,
            computation.id,
            Some(format!("computation:{}", computation.id.raw())),
            "ScopeBody has no budget ceiling; child Usage remains an upper-bound declaration, not a reservation or ledger balance",
        ),
        ControlDimensionProof::new(
            "scope-owned-child-settlement",
            LeafVerdict::RequiresEvidence,
            RULE_SCOPE_RUNTIME_OBLIGATIONS,
            computation.id,
            Some(format!("computation:{}", body.child.raw())),
            "the checked ownership edge is retained, but this static pass cannot observe that every owned child has settled before owner terminal",
        ),
    ];
    if !child_available {
        proofs.push(ControlDimensionProof::new(
            "scope-child-summary",
            LeafVerdict::Unproven,
            RULE_PROOF_PROPAGATION,
            computation.id,
            Some(format!("computation:{}", body.child.raw())),
            "the owned Scope child has no recursively derived summary",
        ));
    }
    (Some(summary), proofs)
}

fn nearest_scope_owner_for_action(
    summary: &DerivedControlContractSummary<'_, '_>,
    inherited_owner: Option<ComputationId>,
    source_node: ComputationId,
    identity: &RecoveryActionIdentity,
) -> Option<ComputationId> {
    let owner = if matches!(&summary.operator, ControlOperator::CompensationScope) {
        Some(summary.control_node)
    } else {
        inherited_owner
    };
    if summary.control_node == source_node {
        for contribution in recovery_sources_from_child(summary).0 {
            for source in contribution.value() {
                if source.source_node() == source_node
                    && source
                        .action_bindings()
                        .iter()
                        .any(|binding| binding.identity == *identity)
                {
                    return owner;
                }
            }
        }
    }
    summary
        .children
        .iter()
        .find_map(|child| nearest_scope_owner_for_action(child, owner, source_node, identity))
}

fn applied_multiplicities(occurrence: &ScopedEffectOccurrence) -> Vec<AppliedMultiplicity> {
    occurrence
        .control_families
        .iter()
        .map(|family| AppliedMultiplicity {
            identity: ControlMultiplicityIdentity {
                caller_semantic_ir_digest: occurrence.caller_semantic_ir_digest.clone(),
                control_node: family.control_node,
                kind: match family.kind {
                    EffectOccurrenceFamilyKind::Iterate => ControlMultiplicityKind::Iterate,
                    EffectOccurrenceFamilyKind::Retry => ControlMultiplicityKind::Retry,
                },
            },
            logical_repetitions: family.logical_repetitions.clone(),
            physical_attempts: family.physical_attempts.clone(),
        })
        .collect()
}

fn recovery_forward_seed_upper(
    source: &RecoverySourceSummary,
    action: &RecoveryActionIdentity,
    declared_lower: u64,
    declared_upper: OccurrenceCeiling,
) -> Option<OccurrenceCeiling> {
    let atom = AtomKey::of(&action.forward_effect);
    if atom != AtomKey::of(&action.action_effect) {
        return Some(declared_upper);
    }
    let mut action_multiplier = OccurrenceCeiling::Finite(0);
    for binding in source.action_bindings() {
        let identity = &binding.identity;
        if AtomKey::of(&identity.action_effect) != atom {
            continue;
        }
        if AtomKey::of(&identity.forward_effect) != atom {
            return None;
        }
        let usage = source
            .usage_bindings()
            .iter()
            .find(|candidate| candidate.action == *identity)?;
        let multiplier = usage
            .activations_per_forward
            .upper
            .checked_mul(usage.attempts_per_activation.upper)
            .ok()?;
        action_multiplier = action_multiplier.checked_add(multiplier).ok()?;
    }
    let divisor = OccurrenceCeiling::Finite(1)
        .checked_add(action_multiplier)
        .ok()?;
    let forward_upper = match (declared_upper, divisor) {
        (OccurrenceCeiling::Unbounded, _) => OccurrenceCeiling::Unbounded,
        (OccurrenceCeiling::Finite(_), OccurrenceCeiling::Unbounded) => {
            OccurrenceCeiling::Finite(0)
        }
        (OccurrenceCeiling::Finite(ceiling), OccurrenceCeiling::Finite(divisor)) => {
            OccurrenceCeiling::Finite(ceiling / divisor)
        }
    };
    if matches!(forward_upper, OccurrenceCeiling::Finite(upper) if upper < declared_lower) {
        return None;
    }
    Some(forward_upper)
}

fn derive_compensation_scope(
    node: ComputationId,
    operator: &ControlOperator,
    children: &[DerivedControlContractSummary<'_, '_>],
    context: &StaticContractProofContext<'_>,
) -> (Option<CompensationScopeSummary>, Vec<ControlDimensionProof>) {
    if !matches!(operator, ControlOperator::CompensationScope) {
        return (None, vec![]);
    }
    let Some(child) = children.first() else {
        let proof = ControlDimensionProof::new(
            "compensation-forward-history",
            LeafVerdict::Unproven,
            RULE_COMPENSATION_FORWARD_HISTORY,
            node,
            None,
            "the checked COMPENSATION_SCOPE child summary is unavailable",
        );
        return (
            Some(CompensationScopeSummary {
                potential_actions: ControlDimension {
                    value: ControlDimensionValue::Unknown {
                        known_contributions: vec![],
                        unresolved_sources: vec![node],
                    },
                    proofs: vec![proof.clone()],
                },
                local_recovery_actions: ControlDimension {
                    value: ControlDimensionValue::Unknown {
                        known_contributions: vec![],
                        unresolved_sources: vec![node],
                    },
                    proofs: vec![proof.clone()],
                },
                compensation_order: LeafVerdict::Unproven,
                recovery_blocked_outcome: LeafVerdict::Unproven,
                forward_effects_retained: true,
                forward_outcome_retained: true,
            }),
            vec![proof],
        );
    };

    let (recovery_sources, mut unresolved) = recovery_sources_from_child(child);
    let mut actions = Vec::new();
    let mut local_actions = Vec::new();
    let mut action_verdict = if unresolved.is_empty() {
        LeafVerdict::Proven
    } else {
        LeafVerdict::Unproven
    };
    let mut local_action_verdict = action_verdict;
    let mut arithmetic_violation = false;
    for contribution in &recovery_sources {
        for source in contribution.value() {
            for binding in source.action_bindings() {
                let identity = &binding.identity;
                if nearest_scope_owner_for_action(child, Some(node), source.source_node(), identity)
                    != Some(node)
                {
                    continue;
                }
                let usage_binding = source
                    .usage_bindings()
                    .iter()
                    .find(|candidate| candidate.action == *identity);
                let Some(usage_binding) = usage_binding else {
                    action_verdict = merge_verdict(action_verdict, LeafVerdict::Violation);
                    unresolved.push(source.source_node());
                    continue;
                };
                let mut found_forward = false;
                for leaf_occurrence in source
                    .effects()
                    .iter()
                    .filter(|occurrence| occurrence.declared.atom == identity.forward_effect)
                {
                    let Some(scoped_occurrence) =
                        child.effect_occurrences.iter().find(|candidate| {
                            candidate.computation == leaf_occurrence.computation
                                && candidate.source_contract_digest
                                    == leaf_occurrence.source_contract_digest
                                && candidate.source_entry_ordinal
                                    == leaf_occurrence.source_entry_ordinal
                                && candidate.declared.atom == identity.forward_effect
                        })
                    else {
                        action_verdict = merge_verdict(action_verdict, LeafVerdict::Unproven);
                        unresolved.push(source.source_node());
                        continue;
                    };
                    let Some(forward_seed_upper) = recovery_forward_seed_upper(
                        source,
                        identity,
                        scoped_occurrence.declared.bounds.lower,
                        scoped_occurrence.declared.bounds.upper,
                    ) else {
                        action_verdict = merge_verdict(action_verdict, LeafVerdict::Unproven);
                        unresolved.push(source.source_node());
                        continue;
                    };
                    found_forward = true;
                    let Some(action_effect_occurrence) =
                        child.effect_occurrences.iter().find(|candidate| {
                            candidate.computation == source.source_node()
                                && candidate.source_contract_digest
                                    == scoped_occurrence.source_contract_digest
                                && candidate.declared.atom == identity.action_effect
                        })
                    else {
                        action_verdict = merge_verdict(action_verdict, LeafVerdict::Unproven);
                        unresolved.push(source.source_node());
                        continue;
                    };
                    let Ok(bounds) = conditional_compensation_bounds(
                        scoped_occurrence,
                        usage_binding,
                        forward_seed_upper,
                    ) else {
                        action_verdict = merge_verdict(action_verdict, LeafVerdict::Violation);
                        arithmetic_violation = true;
                        unresolved.push(source.source_node());
                        continue;
                    };
                    let receipt_evidence = if bounds.upper.is_zero() {
                        LeafVerdict::Proven
                    } else {
                        LeafVerdict::RequiresEvidence
                    };
                    let Ok(usage) = conditional_recovery_usage(
                        &usage_binding.per_attempt_usage,
                        bounds.upper,
                        context,
                    ) else {
                        action_verdict = merge_verdict(action_verdict, LeafVerdict::Violation);
                        arithmetic_violation = true;
                        unresolved.push(source.source_node());
                        continue;
                    };
                    let guard_compatibility = source
                        .action_guard_compatibility()
                        .iter()
                        .find(|proof| proof.recovery_action() == identity)
                        .cloned();
                    let guard_verdict =
                        guard_compatibility
                            .as_ref()
                            .map_or(LeafVerdict::Unproven, |proof| match proof.verdict() {
                                af_contract::ProofVerdict::Proven => LeafVerdict::Proven,
                                af_contract::ProofVerdict::Violation => LeafVerdict::Violation,
                                af_contract::ProofVerdict::Unproven => LeafVerdict::Unproven,
                                af_contract::ProofVerdict::RequiresEvidence => {
                                    LeafVerdict::RequiresEvidence
                                }
                            });
                    local_action_verdict = merge_verdict(local_action_verdict, guard_verdict);
                    let local_bound = LocalActionBound {
                        compensation_scope_node: node,
                        upper: crate::root_recovery::RecoveryActionUpper::Known(bounds.upper),
                        applied_families: applied_multiplicities(scoped_occurrence),
                    };
                    let source_identity = RecoveryActionSourceIdentity {
                        caller_semantic_ir_digest: scoped_occurrence
                            .caller_semantic_ir_digest
                            .clone(),
                        source_node: source.source_node(),
                        source_contract_digest: scoped_occurrence.source_contract_digest.clone(),
                        forward_entry_ordinal: scoped_occurrence.source_entry_ordinal,
                        action: identity.clone(),
                    };
                    local_actions.push(LocalRecoveryAction {
                        source: source_identity,
                        forward_seed_upper,
                        owner_scope: node,
                        action_kind: identity.action_kind,
                        local_bound: local_bound.clone(),
                        forward_occurrence: scoped_occurrence.clone(),
                        action_effect_occurrence: action_effect_occurrence.clone(),
                        recovery_usage_binding: usage_binding.clone(),
                        recovery_usage: usage.clone(),
                        guard_compatibility,
                        eligibility_evidence: receipt_evidence,
                    });
                    if identity.action_kind == RecoveryActionKind::Compensation {
                        actions.push(ConditionalCompensationAction {
                            source_node: source.source_node(),
                            forward_occurrence: scoped_occurrence.clone(),
                            action_effect_occurrence: action_effect_occurrence.clone(),
                            identity: identity.clone(),
                            local_bound,
                            potential_effect: EffectEntry {
                                atom: identity.action_effect.clone(),
                                bounds,
                                // The declared forward guard still gates whether
                                // this recovery route can become relevant.
                                guard: scoped_occurrence.declared.guard.clone(),
                            },
                            potential_recovery_usage: usage,
                            receipt_evidence,
                        });
                    }
                }
                if !found_forward {
                    action_verdict = merge_verdict(action_verdict, LeafVerdict::Unproven);
                    unresolved.push(source.source_node());
                }
            }
        }
    }
    let mut required_action_rows = Vec::<(
        ComputationId,
        DigestRefValue,
        af_effect::EffectAtom,
        OccurrenceCeiling,
        OccurrenceCeiling,
        BTreeSet<usize>,
    )>::new();
    for action in &local_actions {
        let occurrence = &action.action_effect_occurrence;
        let crate::root_recovery::RecoveryActionUpper::Known(action_upper) =
            action.local_bound.upper
        else {
            action_verdict = merge_verdict(action_verdict, LeafVerdict::Violation);
            arithmetic_violation = true;
            unresolved.push(action.source.source_node);
            continue;
        };
        let same_atom_forward =
            action.source.action.forward_effect == action.source.action.action_effect;
        let forward_upper = if same_atom_forward {
            let Ok(upper) = scoped_occurrence_upper_from_seed(
                &action.forward_occurrence,
                action.forward_seed_upper,
            ) else {
                action_verdict = merge_verdict(action_verdict, LeafVerdict::Violation);
                arithmetic_violation = true;
                unresolved.push(action.source.source_node);
                continue;
            };
            upper
        } else {
            OccurrenceCeiling::Finite(0)
        };
        if let Some((_, _, _, action_total, forward_total, forward_ordinals)) = required_action_rows
            .iter_mut()
            .find(|(node, digest, atom, _, _, _)| {
                *node == occurrence.computation
                    && *digest == occurrence.source_contract_digest
                    && *atom == occurrence.declared.atom
            })
        {
            let Ok(total) = action_total.checked_add(action_upper) else {
                action_verdict = merge_verdict(action_verdict, LeafVerdict::Violation);
                arithmetic_violation = true;
                unresolved.push(action.source.source_node);
                continue;
            };
            *action_total = total;
            if same_atom_forward
                && forward_ordinals.insert(action.forward_occurrence.source_entry_ordinal)
            {
                if let Ok(total) = forward_total.checked_add(forward_upper) {
                    *forward_total = total;
                } else {
                    action_verdict = merge_verdict(action_verdict, LeafVerdict::Violation);
                    arithmetic_violation = true;
                    unresolved.push(action.source.source_node);
                }
            }
        } else {
            let mut forward_ordinals = BTreeSet::new();
            let mut forward_total = OccurrenceCeiling::Finite(0);
            if same_atom_forward {
                forward_ordinals.insert(action.forward_occurrence.source_entry_ordinal);
                forward_total = forward_upper;
            }
            required_action_rows.push((
                occurrence.computation,
                occurrence.source_contract_digest.clone(),
                occurrence.declared.atom.clone(),
                action_upper,
                forward_total,
                forward_ordinals,
            ));
        }
    }
    for (source_node, contract_digest, atom, action_total, forward_total, _) in required_action_rows
    {
        let Ok(required) = action_total.checked_add(forward_total) else {
            action_verdict = merge_verdict(action_verdict, LeafVerdict::Violation);
            arithmetic_violation = true;
            unresolved.push(source_node);
            continue;
        };
        let Some(declared) = child.effect_occurrences.iter().find(|occurrence| {
            occurrence.computation == source_node
                && occurrence.source_contract_digest == contract_digest
                && occurrence.declared.atom == atom
        }) else {
            action_verdict = merge_verdict(action_verdict, LeafVerdict::Unproven);
            unresolved.push(source_node);
            continue;
        };
        match scoped_occurrence_upper(declared) {
            Ok(declared_upper) if declared_upper.covers(required) => {}
            Ok(_) => {
                action_verdict = merge_verdict(action_verdict, LeafVerdict::Violation);
                unresolved.push(source_node);
            }
            Err(()) => {
                action_verdict = merge_verdict(action_verdict, LeafVerdict::Violation);
                arithmetic_violation = true;
                unresolved.push(source_node);
            }
        }
    }
    unresolved.sort_unstable();
    unresolved.dedup();
    let possible_events = actions
        .iter()
        .try_fold(OccurrenceCeiling::Finite(0), |total, action| {
            total.checked_add(action.potential_effect.bounds.upper)
        });
    let compensation_order = match possible_events {
        Ok(OccurrenceCeiling::Finite(0 | 1)) => LeafVerdict::Proven,
        Ok(_) => LeafVerdict::RequiresEvidence,
        Err(_) => LeafVerdict::Violation,
    };
    let actions_value = if unresolved.is_empty() {
        ControlDimensionValue::Complete(actions.clone())
    } else if !actions.is_empty() {
        ControlDimensionValue::Partial {
            known_contribution: Some(actions.clone()),
            unresolved_sources: unresolved.clone(),
        }
    } else {
        ControlDimensionValue::Unknown {
            known_contributions: vec![],
            unresolved_sources: unresolved.clone(),
        }
    };
    local_action_verdict = merge_verdict(local_action_verdict, action_verdict);
    let local_actions_value = if unresolved.is_empty() {
        ControlDimensionValue::Complete(local_actions.clone())
    } else if !local_actions.is_empty() {
        ControlDimensionValue::Partial {
            known_contribution: Some(local_actions.clone()),
            unresolved_sources: unresolved.clone(),
        }
    } else {
        ControlDimensionValue::Unknown {
            known_contributions: vec![],
            unresolved_sources: unresolved.clone(),
        }
    };
    let local_receipt_required = local_actions
        .iter()
        .any(|action| action.eligibility_evidence == LeafVerdict::RequiresEvidence);
    let local_proof_verdict = if arithmetic_violation {
        LeafVerdict::Violation
    } else if local_receipt_required && local_action_verdict == LeafVerdict::Proven {
        LeafVerdict::RequiresEvidence
    } else {
        local_action_verdict
    };
    let local_actions_proof = ControlDimensionProof::new(
        "local-k-action-bounds",
        local_proof_verdict,
        RULE_RECOVERY_ROOT_MULTIPLICITY,
        node,
        Some(format!("computation:{}", node.raw())),
        "each locally owned K action is expressed per activation of this exact CompensationScope, with descendant Repeat identities applied once and nested owners excluded",
    );
    let local_recovery_actions = ControlDimension {
        value: local_actions_value,
        proofs: vec![local_actions_proof.clone()],
    };
    let receipt_required = actions
        .iter()
        .any(|action| action.receipt_evidence == LeafVerdict::RequiresEvidence);
    let actions_proof_verdict = if arithmetic_violation {
        LeafVerdict::Violation
    } else if receipt_required && action_verdict == LeafVerdict::Proven {
        LeafVerdict::RequiresEvidence
    } else {
        action_verdict
    };
    let actions_proof = ControlDimensionProof::new(
        "conditional-compensation-actions",
        actions_proof_verdict,
        RULE_COMPENSATION_ACTIONS,
        node,
        Some(format!("computation:{}", node.raw())),
        "exact compensation K bindings and activation/attempt bounds are retained as conditional new Effect and Recovery-Usage possibilities; EffectReceipt state is required to know eligibility",
    );
    let action_dimension = ControlDimension {
        value: actions_value,
        proofs: vec![actions_proof.clone()],
    };
    let order_proof = ControlDimensionProof::new(
        "compensation-commit-order",
        compensation_order,
        RULE_COMPENSATION_USAGE_ORDER,
        node,
        Some(format!("computation:{}", node.raw())),
        "static source order does not establish actual committed Effect order for compensation; a multi-event reverse order requires committed receipt evidence",
    );
    let blocked_verdict = if !unresolved.is_empty() {
        LeafVerdict::Unproven
    } else if receipt_required {
        LeafVerdict::RequiresEvidence
    } else {
        LeafVerdict::Unproven
    };
    let blocked_proof = ControlDimensionProof::new(
        "recovery-blocked-outcome",
        blocked_verdict,
        RULE_COMPENSATION_ACTIONS,
        node,
        Some(format!("computation:{}", node.raw())),
        "RecoveryBlocked is a possible recovery outcome distinct from typed Failure; this summary has no runtime reconciliation/compensation result and reports no actual blocked state",
    );
    let forward_proof = ControlDimensionProof::new(
        "compensation-forward-history",
        LeafVerdict::Proven,
        RULE_COMPENSATION_FORWARD_HISTORY,
        node,
        Some(format!("computation:{}", child.control_node.raw())),
        "the child's forward Effect envelope and typed Failure outcome are propagated unchanged; potential compensation is a separate conditional projection and never subtracts/refunds forward history",
    );
    let summary = CompensationScopeSummary {
        potential_actions: action_dimension,
        local_recovery_actions,
        compensation_order,
        recovery_blocked_outcome: blocked_verdict,
        forward_effects_retained: true,
        forward_outcome_retained: true,
    };
    let mut proofs = vec![
        forward_proof,
        actions_proof,
        local_actions_proof,
        order_proof,
        blocked_proof,
    ];
    if recovery_sources.is_empty()
        && matches!(
            &child.recovery.value,
            ControlDimensionValue::PerChild { .. }
        )
    {
        proofs.push(ControlDimensionProof::new(
            "compensation-recovery-coverage",
            LeafVerdict::Unproven,
            RULE_COMPENSATION_ACTIONS,
            node,
            Some(format!("computation:{}", child.control_node.raw())),
            "no resolved K Recovery source was available from the owned child",
        ));
    }
    (Some(summary), proofs)
}

fn recovery_sources_from_child(
    child: &DerivedControlContractSummary<'_, '_>,
) -> (
    Vec<DimensionContribution<Vec<RecoverySourceSummary>>>,
    Vec<ComputationId>,
) {
    match &child.recovery.value {
        ControlDimensionValue::PerChild {
            contributions,
            unresolved_sources,
        } => (contributions.clone(), unresolved_sources.clone()),
        ControlDimensionValue::Complete(sources) => (
            vec![DimensionContribution {
                source_node: child.control_node,
                value: sources.clone(),
            }],
            vec![],
        ),
        ControlDimensionValue::Partial {
            known_contribution,
            unresolved_sources,
        } => (
            known_contribution
                .as_ref()
                .map(|sources| {
                    vec![DimensionContribution {
                        source_node: child.control_node,
                        value: sources.clone(),
                    }]
                })
                .unwrap_or_default(),
            unresolved_sources.clone(),
        ),
        ControlDimensionValue::Unknown {
            known_contributions,
            unresolved_sources,
        } => (known_contributions.clone(), unresolved_sources.clone()),
        ControlDimensionValue::NotApplicable => (vec![], vec![child.control_node]),
    }
}

fn conditional_compensation_bounds(
    forward: &ScopedEffectOccurrence,
    binding: &RecoveryUsageBinding,
    forward_seed_upper: OccurrenceCeiling,
) -> Result<OccurrenceBounds, ()> {
    let forward_upper = scoped_occurrence_upper_from_seed(forward, forward_seed_upper)?;
    let upper = forward_upper
        .checked_mul(binding.activations_per_forward.upper)
        .and_then(|bound| bound.checked_mul(binding.attempts_per_activation.upper))
        .map_err(|_| ())?;
    Ok(OccurrenceBounds { lower: 0, upper })
}

fn scoped_occurrence_upper(occurrence: &ScopedEffectOccurrence) -> Result<OccurrenceCeiling, ()> {
    scoped_occurrence_upper_from_seed(occurrence, occurrence.declared.bounds.upper)
}

fn scoped_occurrence_upper_from_seed(
    occurrence: &ScopedEffectOccurrence,
    seed_upper: OccurrenceCeiling,
) -> Result<OccurrenceCeiling, ()> {
    let mut upper = seed_upper;
    let mut seen = Vec::new();
    for factor in applied_multiplicities(occurrence) {
        if seen.contains(&factor.identity) {
            return Err(());
        }
        seen.push(factor.identity);
        upper = upper
            .checked_mul(factor.logical_repetitions.upper)
            .map_err(|_| ())?;
    }
    Ok(upper)
}

fn conditional_recovery_usage(
    per_attempt: &af_usage::UsageSignature,
    action_upper: OccurrenceCeiling,
    context: &StaticContractProofContext<'_>,
) -> Result<StaticUsageEnvelope, ()> {
    let envelope =
        af_usage::declared_usage_envelope(per_attempt, context.usage_meters).map_err(|_| ())?;
    if envelope.is_globally_unknown() {
        return Ok(StaticUsageEnvelope::unknown());
    }
    let mut result = StaticUsageEnvelope::empty();
    for (coordinate, bound) in envelope.iter() {
        let scaled = match (action_upper, bound) {
            (OccurrenceCeiling::Finite(0), _) => StaticUsageBound::KnownFinite(0),
            (OccurrenceCeiling::Finite(count), StaticUsageBound::KnownFinite(value)) => {
                let scaled = UsageUpperBound::Finite(*value)
                    .checked_mul(count)
                    .map_err(|_| ())?;
                StaticUsageBound::from_declared(scaled)
            }
            (OccurrenceCeiling::Unbounded, StaticUsageBound::KnownFinite(0)) => {
                StaticUsageBound::KnownFinite(0)
            }
            (
                OccurrenceCeiling::Finite(_) | OccurrenceCeiling::Unbounded,
                StaticUsageBound::KnownUnbounded,
            )
            | (OccurrenceCeiling::Unbounded, StaticUsageBound::KnownFinite(_)) => {
                StaticUsageBound::KnownUnbounded
            }
            (_, StaticUsageBound::Unknown) => StaticUsageBound::Unknown,
        };
        result.insert(coordinate.clone(), scaled).map_err(|_| ())?;
    }
    Ok(result)
}

fn derive_recovery_dimension(
    node: ComputationId,
    operator: &ControlOperator,
    children: &[DerivedControlContractSummary<'_, '_>],
    occurrences: &[ScopedEffectOccurrence],
    context: &StaticContractProofContext<'_>,
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
    let mut dimension_proofs = vec![proof.clone()];
    let mut all_proofs = vec![proof.clone()];
    let retry_effect_possible = children.iter().any(|child| match &child.effects.value {
        ControlDimensionValue::Complete(signature)
        | ControlDimensionValue::Partial {
            known_contribution: Some(signature),
            ..
        } => signature_has_possible_effects(signature),
        ControlDimensionValue::Unknown { .. }
        | ControlDimensionValue::Partial { .. }
        | ControlDimensionValue::PerChild { .. }
        | ControlDimensionValue::NotApplicable => true,
    });
    if retry_effect_possible
        && matches!(operator, ControlOperator::Retry { max_attempts, .. } if *max_attempts > 1)
    {
        let retry_verdict = retry_recovery_safety(&contributions, context);
        let retry_proof = ControlDimensionProof::new(
            "retry-recovery-safety",
            retry_verdict,
            RULE_RETRY_ENVELOPE,
            node,
            Some(format!("computation:{}", node.raw())),
            match retry_verdict {
                LeafVerdict::Proven => {
                    "the child's exact K declaration is statically compatible with retry, but it does not narrow the conservative logical Effect envelope or create runtime evidence"
                }
                LeafVerdict::RequiresEvidence => {
                    "the child's exact K declaration requires provider/application evidence across attempts; no runtime evidence or retry permit is created"
                }
                LeafVerdict::Unproven => {
                    "the child's exact K declaration is retained, but missing or unknown recovery coverage prevents a static retry-safety proof"
                }
                LeafVerdict::Violation => {
                    "the child's K declaration conflicts with retry or is statically unsafe; the violation is retained and no retry permit is created"
                }
            },
        );
        dimension_proofs.push(retry_proof.clone());
        all_proofs.push(retry_proof);
    }
    let dimension = ControlDimension {
        value: ControlDimensionValue::PerChild {
            contributions,
            unresolved_sources: unresolved,
        },
        proofs: dimension_proofs,
    };
    (dimension, all_proofs)
}

fn retry_recovery_safety(
    contributions: &[DimensionContribution<Vec<RecoverySourceSummary>>],
    context: &StaticContractProofContext<'_>,
) -> LeafVerdict {
    let mut found_source = false;
    let mut verdict = LeafVerdict::Proven;
    for source in contributions
        .iter()
        .flat_map(|contribution| contribution.value.iter())
    {
        found_source = true;
        if source.contract.replay_semantics == af_recovery::ReplaySemantics::Forbidden {
            return LeafVerdict::Violation;
        }
        let effect = EffectSignature {
            row: EffectRow::Closed(
                source
                    .effects
                    .iter()
                    .map(|occurrence| occurrence.declared.clone())
                    .collect(),
            ),
        };
        let evidence = BTreeMap::new();
        let result = af_recovery::analyze_recovery_safety(
            &source.contract,
            &effect,
            context.recovery,
            &evidence,
        );
        let source_verdict = match result {
            Ok(af_recovery::RecoveryProof::ProvenStaticConstraint) => LeafVerdict::Proven,
            Ok(af_recovery::RecoveryProof::RequiresEvidence(_)) => LeafVerdict::RequiresEvidence,
            Ok(af_recovery::RecoveryProof::Unproven(_)) => LeafVerdict::Unproven,
            Ok(af_recovery::RecoveryProof::Violation(_)) | Err(_) => LeafVerdict::Violation,
        };
        verdict = merge_verdict(verdict, source_verdict);
        if verdict == LeafVerdict::Violation {
            return verdict;
        }
    }
    if found_source {
        verdict
    } else {
        LeafVerdict::Unproven
    }
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
