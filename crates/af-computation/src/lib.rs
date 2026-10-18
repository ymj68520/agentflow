//! Pure, explicit static analysis for Semantic IR computation leaves and
//! selected structured/advanced control nodes and workflow-root closure.
//!
//! Leaf resolution handles `INVOKE`, `AWAIT`, and `CALL_SUBFLOW`. The
//! control algebra composes `SEQUENCE`, `SELECT`, `PARALLEL`, `RACE`,
//! `ITERATE`, `RETRY`, `TIMEOUT`, `HANDLE`, `SCOPE`, and
//! `COMPENSATION_SCOPE`, consuming the checked IR, Phase 3C dataflow, resolved
//! leaves, and the same immutable proof context. Workflow-root closure accounts
//! for every owned node but never promotes unresolved dimensions to Proven.
//! No summary is a ComponentContract, digest input, runtime binding, grant,
//! handle, plan, or workflow attestation.

mod control;
mod input_binder;
mod input_graph;
mod input_source;
mod mapping;
pub mod native_capture;
mod occurrence_admission;
mod proof;
mod raw_admission;
pub mod raw_capture;
mod registered_model;
mod relation_admission;
mod resolution;
mod root_export;
mod root_recovery;
mod structural_graph_metadata;
mod structural_staging;
mod summary;

pub use control::{
    CompensationScopeSummary, ConditionalCompensationAction, ControlDerivationError,
    ControlDimension, ControlDimensionProof, ControlDimensionValue, ControlFlowContractSource,
    ControlFlowSummary, ControlInterfaceSummary, ControlOperator, ControlOutputPort,
    ControlOutputSource, ControlProofReport, ControlResourceAlias, ControlTreeClosureSummary,
    DerivedControlContractSummary, DerivedWorkflowContractSummary, DimensionContribution,
    EffectLowerBoundAdjustment, EffectLowerBoundAdjustmentReason, ExecutionSourceSummary,
    HandleClauseSummary, HandleMatcherSummary, InheritedControlObligation, IterateMode,
    RecoverySourceSummary, ResourcePortOccurrence, RootExportCompatibilityStatus,
    ScopeBoundarySummary, SelectDecisionEvidence, SequenceDataflowDependency, TimeoutOutcome,
    derive_control_contract, derive_workflow_contract_summary,
};
pub use input_binder::{
    InputBindingFailure, RawSubjectInputNode, VerifiedInputGraph, bind_raw_input_graph,
};
pub use occurrence_admission::{
    DocumentOccurrenceAdmissionFailure, DocumentOccurrenceMetadataRequest,
    OccurrencePhysicalMetadata, admit_document_occurrence_metadata, admit_source_package_metadata,
    check_joint_source_packages_metadata,
};
pub use proof::{LeafDimensionProof, LeafProofReport, LeafVerdict};
pub use raw_admission::{
    NativeRawDocument, RawAdmissionFailure, RawDocumentMember, admit_native_raw_source,
    admit_physical_raw_source,
};
pub use registered_model::{
    RegisteredModelFailure, SUCCESSOR_MODEL_ID, admit_registered_profile,
    registered_descriptor_integrity, registered_model_catalogue, validate_registered_model_shape,
};
pub use relation_admission::{
    RELATION_MODEL_ID, RelationContextMetadataAdmission, RelationMetadataObservation,
    RelationMetadataSlot, admit_relation_context_observation_metadata,
    admit_relation_observation_metadata, inspect_relation_context_metadata,
    inspect_relation_definition_candidates, inspect_relation_metadata,
    relation_descriptor_integrity, relation_model_catalogue, relation_schema_definition_metadata,
    validate_relation_registered_shape, validate_relation_source_dag_metadata,
};
pub use resolution::{
    LeafResolutionError, ResolverSnapshotIdentity, ResolverSubflowIdentity, SnapshotError,
    StaticComputationResolutionSnapshot, StaticComputationResolutionSnapshotBuilder,
    resolve_computation_leaf,
};
pub use root_export::{
    CheckedRootExportCompatibility, CompilerSemanticsIdentity, ComponentInputPortIdentity,
    ComponentOutputPortIdentity, EditionActivation, EditionAdmissionFailure, EditionRegistration,
    EditionRelease, EditionSupport, OBLIGATION_IDENTITY_EDITION, ProofContextManifest,
    REPORT_SHAPE_EDITION, ROOT_EXPORT_ANALYSIS_SEMANTICS_VERSION, RegisteredAnalysisEdition,
    RelationSnapshotContent, ResolverDependencyManifest, ResourceRequirementPortIdentity,
    RootEffectCoordinateBinding, RootEffectOccurrenceIdentity, RootExecutionChildProof,
    RootExecutionCoordinateBinding, RootExportAssertion, RootExportAssertionIdentity,
    RootExportCoordinateWitness, RootExportDimension, RootExportDimensionProof,
    RootExportProofReport, RootExportSourceReference, RootFlowCoordinateBinding,
    RootInputPortBinding, RootInputUseSite, RootOutputPortBinding, RootOutputProducerEvidence,
    RootRecoveryCoordinateBinding, RootResourcePortBinding, RootResourceSourceCoordinate,
    RuleContentIdentity, RuntimeAuthorityStatus, SpecificationContentIdentity,
    SubflowProofDependency, SubflowTargetContextPin, SubflowTargetIdentity,
    analyze_root_export_assertion, check_registered_compiler_identity,
    check_registered_report_editions, check_root_export_assertion,
    current_compiler_semantics_identity, current_registered_analysis_edition,
};
pub use root_recovery::{
    AppliedMultiplicity, ControlMultiplicityIdentity, ControlMultiplicityKind, ControlPathKind,
    ControlPathReachability, ControlPathStep, LocalActionBound, LocalRecoveryAction,
    RecoveryActionSourceIdentity, RecoveryActionUpper, RootActionBound,
    RootGuardCoordinateEvidence, RootRecoveryAction, RootRecoveryOwner,
    RootRecoverySourceClosureReport, RootRecoverySummary,
};
pub use structural_staging::{
    STRUCTURAL_MODEL_ID, StructuralMetadataFailure, inspect_structural_module_map,
    inspect_structural_registration, structural_descriptor_integrity, structural_model_catalogue,
    validate_structural_shape_metadata,
};
pub use summary::{
    AwaitAuthorityStatus, AwaitInteractionStatus, AwaitLeafSummary, AwaitProvenanceStatus,
    AwaitTimeoutSemantics, ComponentDimensionSummary, EffectOccurrenceFamily,
    EffectOccurrenceFamilyKind, FlowPortMapping, FlowSinkBinding, FlowSourceBinding, LeafKind,
    ProofContextReference, ResolvedComputationLeaf, ResolvedTarget, ResourceAliasGroup,
    ScopedEffectOccurrence, ScopedEffectResidual, ScopedPortBinding, ScopedPortMapping,
    ScopedResourcePortMapping,
};
