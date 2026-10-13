//! In-process RootExportAssertion and fail-closed static compatibility proof.
//!
//! This module is deliberately outside Semantic IR and ComponentContract.
//! It has no wire encoding, independent digest, persistence, or runtime
//! authority. Its identity is the complete typed assertion input tuple.

use crate::control::{
    ControlDimensionValue, ControlOperator, DerivedControlContractSummary,
    DerivedWorkflowContractSummary, ExecutionSourceSummary, ResourcePortOccurrence,
};
use crate::resolution::StaticComputationResolutionSnapshot;
use crate::root_recovery::RecoveryActionSourceIdentity;
use crate::summary::{FlowSinkBinding, FlowSourceBinding, ResolvedTarget, ScopedEffectOccurrence};
use af_contract::RecoveryUsageBinding;
use af_contract::{
    CheckedComponentContract, ComponentContract, DimensionProof, EffectFlowPartition,
    RelationCatalogPin, StaticContractProofContext, component_contract_digest,
};
use af_effect::{
    AtomKey, EffectAtom, EffectEntry, EffectGuard, EffectRow, EffectSignature, OccurrenceBounds,
    OccurrenceCeiling,
};
use af_execution::ExecutionRequirements;
use af_flow::{FlowConstraint, FlowKind, FlowSink, FlowSourceRef};
use af_ir::{
    CheckedIrDocument, ComputationBody, ComputationId, RunParameterId, SemanticIrDocument,
};
use af_resource::{ResourceRequirement, ResourceRequirementRow, ResourceTypeRef};
use af_semantic::{
    DigestRefValue, ExpressionBody, ExpressionId, ResourceSlotId, StructuralType, TypeSchemaRef,
    ValueSlotId,
};
use af_usage::{StaticUsageBound, StaticUsageEnvelope, UsagePhase};
use sha2::{Digest as _, Sha256};
use std::collections::BTreeMap;

const RULE_ASSERTION_PIN: &str = "AF-SEMANTIC-0159";
const RULE_CONTEXT_MANIFEST: &str = "AF-SEMANTIC-0160";
const RULE_COMPILER_SEMANTICS: &str = "AF-SEMANTIC-0161";
const RULE_INPUT_MAP: &str = "AF-SEMANTIC-0162";
const RULE_OUTPUT_MAP: &str = "AF-SEMANTIC-0163";
const RULE_RESOURCE_MAP: &str = "AF-SEMANTIC-0164";
const RULE_PHI: &str = "AF-SEMANTIC-0165";
const RULE_EPSILON: &str = "AF-SEMANTIC-0166";
const RULE_FLOW: &str = "AF-SEMANTIC-0167";
const RULE_USAGE: &str = "AF-SEMANTIC-0168";
const RULE_RECOVERY: &str = "AF-SEMANTIC-0169";
const RULE_EXECUTION: &str = "AF-SEMANTIC-0170";
const RULE_SUBFLOW: &str = "AF-SEMANTIC-0171";

/// Compiler-side semantics version. Any semantic checker change must bump it.
pub const ROOT_EXPORT_ANALYSIS_SEMANTICS_VERSION: &str = "agentflow.root-export-checker@1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecificationContentIdentity {
    /// Exact SpecificationId, including major version.
    pub specification_id: String,
    /// Exact registered logical source path; not resolved from the filesystem.
    pub logical_path: String,
    /// SHA-256 of `content`, used only as a content checksum, never HashV1.
    pub content_sha256: String,
    /// Complete normative source bytes make this an exact in-process identity.
    pub content: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleContentIdentity {
    pub rule_id: String,
    pub normative_source: String,
    pub source_anchor: String,
    pub short_statement: String,
}

/// Exact checker semantics input: normative content, rules, and analysis version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerSemanticsIdentity {
    pub analysis_semantics_version: String,
    pub registry_content: Vec<u8>,
    pub rule_registry_content: Vec<u8>,
    pub specifications: Vec<SpecificationContentIdentity>,
    pub rules: Vec<RuleContentIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationSnapshotContent {
    pub pin: RelationCatalogPin,
    pub canonical_content: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubflowTargetContextPin {
    pub semantic_ir_digest: DigestRefValue,
    pub direct_dependencies: Vec<DigestRefValue>,
    pub exported_contract_digest: Option<DigestRefValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolverDependencyManifest {
    pub component_contract_digests: Vec<DigestRefValue>,
    pub subflow_targets: Vec<SubflowTargetContextPin>,
}

/// Complete typed content snapshot of the exact static analysis context.
/// Pointer identity is checked separately and is never used as its content identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofContextManifest {
    pub compiled_modules: af_idl::CompiledModuleSet,
    pub type_schemas: af_semantic::TypeSchemaUniverse,
    pub effect_catalogs: af_effect::EffectCatalogUniverse,
    pub resource_types: af_resource::ResourceTypeUniverse,
    pub flow_catalogs: af_flow::FlowCatalogs,
    pub usage_meters: af_usage::UsageMeterUniverse,
    pub recovery_operations: af_recovery::RecoveryOperationUniverse,
    pub execution_universes: af_execution::ExecutionUniverses,
    pub relation_snapshots: Vec<RelationSnapshotContent>,
    pub resolver_dependencies: ResolverDependencyManifest,
}

impl ProofContextManifest {
    /// Captures every explicit static universe and exact resolver pin.
    #[must_use]
    pub fn capture(
        context: &StaticContractProofContext<'_>,
        snapshot: &StaticComputationResolutionSnapshot<'_, '_, '_>,
    ) -> Self {
        let mut relation_snapshots = Vec::new();
        if let Some(value) = &context.relations.effect_resource {
            relation_snapshots.push(RelationSnapshotContent {
                pin: value.pin().clone(),
                canonical_content: value.canonical_bytes().to_vec(),
            });
        }
        if let Some(value) = &context.relations.recovery_usage {
            relation_snapshots.push(RelationSnapshotContent {
                pin: value.pin().clone(),
                canonical_content: value.canonical_bytes().to_vec(),
            });
        }
        if let Some(value) = &context.relations.flow_execution {
            relation_snapshots.push(RelationSnapshotContent {
                pin: value.pin().clone(),
                canonical_content: value.canonical_bytes().to_vec(),
            });
        }
        relation_snapshots.sort_by(|left, right| left.pin.schema.cmp(&right.pin.schema));

        let mut subflow_targets = snapshot
            .subflow_target_pins()
            .into_iter()
            .map(|semantic_ir_digest| SubflowTargetContextPin {
                direct_dependencies: snapshot
                    .subflow_dependencies(&semantic_ir_digest)
                    .unwrap_or_default()
                    .to_vec(),
                exported_contract_digest: snapshot.subflow_export_pin(&semantic_ir_digest).cloned(),
                semantic_ir_digest,
            })
            .collect::<Vec<_>>();
        subflow_targets.sort_by(|left, right| {
            digest_key(&left.semantic_ir_digest).cmp(&digest_key(&right.semantic_ir_digest))
        });

        Self {
            compiled_modules: context.modules.clone(),
            type_schemas: context.schema_universe.clone(),
            effect_catalogs: context.effects.clone(),
            resource_types: context.resources.clone(),
            flow_catalogs: context.flow.clone(),
            usage_meters: context.usage_meters.clone(),
            recovery_operations: context.recovery.clone(),
            execution_universes: context.execution.clone(),
            relation_snapshots,
            resolver_dependencies: ResolverDependencyManifest {
                component_contract_digests: snapshot.component_contract_pins(),
                subflow_targets,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentInputPortIdentity {
    pub exported_contract_digest: DigestRefValue,
    pub port_no: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentOutputPortIdentity {
    pub exported_contract_digest: DigestRefValue,
    pub port_no: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceRequirementPortIdentity {
    pub exported_contract_digest: DigestRefValue,
    pub port_no: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootInputUseSite {
    pub computation: ComputationId,
    pub source_contract_digest: DigestRefValue,
    pub source_component_port: u32,
    pub input_expression: ExpressionId,
    pub exact_type: TypeSchemaRef,
    pub dependency_slots: Vec<ValueSlotId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootInputPortBinding {
    pub export_input_port: ComponentInputPortIdentity,
    pub run_parameter_id: RunParameterId,
    pub root_value_slot: ValueSlotId,
    pub root_type: StructuralType,
    pub export_type: TypeSchemaRef,
    pub use_sites: Vec<RootInputUseSite>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootOutputProducerEvidence {
    pub exact_result_type: TypeSchemaRef,
    pub dependency_slots: Vec<ValueSlotId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootOutputPortBinding {
    pub export_output_port: ComponentOutputPortIdentity,
    pub workflow_output_name: String,
    pub workflow_output_expression: ExpressionId,
    pub root_result_type: StructuralType,
    pub producer_evidence: RootOutputProducerEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootResourceSourceCoordinate {
    pub source_node: ComputationId,
    pub source_contract_digest: DigestRefValue,
    pub component_resource_port: u32,
    pub resource_slot: ResourceSlotId,
    pub alias_component_ports: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootResourcePortBinding {
    pub export_resource_port: ResourceRequirementPortIdentity,
    pub root_resource_slot: ResourceSlotId,
    pub root_resource_type: TypeSchemaRef,
    pub export_resource_type: ResourceTypeRef,
    pub source_coordinates: Vec<RootResourceSourceCoordinate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootEffectOccurrenceIdentity {
    pub caller_semantic_ir_digest: DigestRefValue,
    pub source_node: ComputationId,
    pub source_contract_digest: DigestRefValue,
    pub source_entry_ordinal: usize,
    pub atom: EffectAtom,
    pub guard: EffectGuard,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootEffectCoordinateBinding {
    pub source: RootEffectOccurrenceIdentity,
    pub root_resource_slot: Option<ResourceSlotId>,
    pub export_atom: EffectAtom,
    pub export_guard: EffectGuard,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootFlowCoordinateBinding {
    pub source_node: ComputationId,
    pub source_contract_digest: DigestRefValue,
    pub flow_mapping_ordinal: u64,
    pub source: Option<FlowSourceRef>,
    pub sink: Option<FlowConstraint>,
    pub source_binding: Option<FlowSourceBinding>,
    pub sink_binding: Option<FlowSinkBinding>,
    pub export_source: Option<FlowSourceRef>,
    pub export_sink: Option<FlowConstraint>,
    pub source_effect_partition: Option<EffectFlowPartition>,
    pub export_effect_partition: Option<EffectFlowPartition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootRecoveryCoordinateBinding {
    pub source: RecoveryActionSourceIdentity,
    pub export_action_identity: af_contract::RecoveryActionIdentity,
    pub exact_usage_binding: RecoveryUsageBinding,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootExecutionCoordinateBinding {
    pub source_node: ComputationId,
    pub source_contract_digest: DigestRefValue,
    pub exact_execution_requirements: ExecutionRequirements,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubflowTargetIdentity {
    pub semantic_ir_digest: DigestRefValue,
    pub exported_contract_digest: Option<DigestRefValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubflowProofDependency {
    pub call_site: ComputationId,
    pub target_semantic_ir_digest: DigestRefValue,
    pub target_exported_contract_digest: Option<DigestRefValue>,
    pub target_root_assertion_identity: Option<Box<RootExportAssertionIdentity>>,
    pub transitive_targets: Vec<SubflowTargetIdentity>,
}

/// Caller-supplied typed mapping assertions. No field is a proof verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootExportAssertion {
    pub exact_semantic_ir_digest: DigestRefValue,
    pub exact_exported_component_contract_digest: DigestRefValue,
    pub compiler_semantics_identity: CompilerSemanticsIdentity,
    pub proof_context_manifest: ProofContextManifest,
    pub input_mappings: Vec<RootInputPortBinding>,
    pub output_mappings: Vec<RootOutputPortBinding>,
    pub resource_mappings: Vec<RootResourcePortBinding>,
    pub effect_mappings: Vec<RootEffectCoordinateBinding>,
    pub flow_mappings: Vec<RootFlowCoordinateBinding>,
    pub recovery_mappings: Vec<RootRecoveryCoordinateBinding>,
    pub execution_mappings: Vec<RootExecutionCoordinateBinding>,
    pub subflow_dependencies: Vec<SubflowProofDependency>,
}

impl RootExportAssertion {
    /// Canonicalizes mapping order without deduplicating semantic entries.
    #[must_use]
    pub fn canonicalized(mut self) -> Self {
        self.input_mappings.sort_by(|a, b| {
            (
                a.export_input_port.port_no,
                a.run_parameter_id,
                a.root_value_slot,
            )
                .cmp(&(
                    b.export_input_port.port_no,
                    b.run_parameter_id,
                    b.root_value_slot,
                ))
        });
        self.output_mappings.sort_by(|a, b| {
            (
                a.export_output_port.port_no,
                &a.workflow_output_name,
                a.workflow_output_expression,
            )
                .cmp(&(
                    b.export_output_port.port_no,
                    &b.workflow_output_name,
                    b.workflow_output_expression,
                ))
        });
        self.resource_mappings.sort_by(|a, b| {
            (a.export_resource_port.port_no, a.root_resource_slot)
                .cmp(&(b.export_resource_port.port_no, b.root_resource_slot))
        });
        for mapping in &mut self.resource_mappings {
            for source in &mut mapping.source_coordinates {
                source.alias_component_ports.sort_unstable();
                source.alias_component_ports.dedup();
            }
            mapping.source_coordinates.sort_by(|left, right| {
                (
                    left.source_node,
                    digest_key(&left.source_contract_digest),
                    left.component_resource_port,
                    left.resource_slot,
                )
                    .cmp(&(
                        right.source_node,
                        digest_key(&right.source_contract_digest),
                        right.component_resource_port,
                        right.resource_slot,
                    ))
            });
        }
        self.effect_mappings.sort_by_key(effect_mapping_key);
        self.flow_mappings.sort_by_key(flow_mapping_key);
        self.recovery_mappings.sort_by_key(recovery_mapping_key);
        self.execution_mappings.sort_by(|a, b| {
            (a.source_node, digest_key(&a.source_contract_digest))
                .cmp(&(b.source_node, digest_key(&b.source_contract_digest)))
        });
        self.subflow_dependencies.sort_by(|a, b| {
            (digest_key(&a.target_semantic_ir_digest), a.call_site)
                .cmp(&(digest_key(&b.target_semantic_ir_digest), b.call_site))
        });
        for dependency in &mut self.subflow_dependencies {
            dependency.transitive_targets.sort_by(|left, right| {
                (
                    digest_key(&left.semantic_ir_digest),
                    left.exported_contract_digest.as_ref().map(digest_key),
                )
                    .cmp(&(
                        digest_key(&right.semantic_ir_digest),
                        right.exported_contract_digest.as_ref().map(digest_key),
                    ))
            });
        }
        for mapping in &mut self.input_mappings {
            mapping.use_sites.sort_by(|a, b| {
                (
                    a.computation,
                    digest_key(&a.source_contract_digest),
                    a.source_component_port,
                    a.input_expression,
                )
                    .cmp(&(
                        b.computation,
                        digest_key(&b.source_contract_digest),
                        b.source_component_port,
                        b.input_expression,
                    ))
            });
        }
        self
    }

    #[must_use]
    pub fn identity(&self) -> RootExportAssertionIdentity {
        RootExportAssertionIdentity(self.clone())
    }
}

/// Full typed tuple identity. It is intentionally not a digest or signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootExportAssertionIdentity(RootExportAssertion);

impl RootExportAssertionIdentity {
    #[must_use]
    pub fn assertion(&self) -> &RootExportAssertion {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootExportDimension {
    Input,
    Output,
    Failure,
    Effect,
    Resource,
    InformationFlow,
    Usage,
    Recovery,
    Execution,
}

impl RootExportDimension {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Input => "I",
            Self::Output => "O",
            Self::Failure => "Phi",
            Self::Effect => "epsilon",
            Self::Resource => "R",
            Self::InformationFlow => "F",
            Self::Usage => "U",
            Self::Recovery => "K",
            Self::Execution => "X",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootExportSourceReference {
    pub semantic_ir_digest: DigestRefValue,
    pub computation: Option<ComputationId>,
    pub contract_digest: Option<DigestRefValue>,
    pub source_ordinal: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RootExportCoordinateWitness {
    Input(Box<RootInputPortBinding>),
    Output(Box<RootOutputPortBinding>),
    Resource(Box<RootResourcePortBinding>),
    Effect(Box<RootEffectCoordinateBinding>),
    Flow(Box<RootFlowCoordinateBinding>),
    Recovery(Box<RootRecoveryCoordinateBinding>),
    Execution(Box<RootExecutionCoordinateBinding>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootExportDimensionProof {
    pub dimension: RootExportDimension,
    pub proof: DimensionProof,
    pub source_facts: Vec<RootExportSourceReference>,
    /// Exact typed map rows used in this dimension.
    pub mapped_coordinates: Vec<RootExportCoordinateWitness>,
    /// Redundant count retained for compact corpus projections.
    pub mapped_coordinate_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootExecutionChildProof {
    pub source_node: ComputationId,
    pub source_contract_digest: DigestRefValue,
    pub feasibility: af_execution::CompatibilityVerdict,
    pub security: af_execution::CompatibilityVerdict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootExportProofReport {
    pub assertion_identity: RootExportAssertionIdentity,
    pub preflight: Vec<DimensionProof>,
    pub dimensions: Vec<RootExportDimensionProof>,
    pub execution_children: Vec<RootExecutionChildProof>,
    pub body_conformance: af_contract::ProofVerdict,
    pub implementation_conformance: af_contract::ProofVerdict,
    pub scope_authority_attenuation: af_contract::ProofVerdict,
    pub scope_budget_attenuation: af_contract::ProofVerdict,
    pub runtime_authority: RuntimeAuthorityStatus,
    pub runtime_authority_evidence: af_contract::ProofVerdict,
    pub publisher_authentication: RuntimeAuthorityStatus,
    pub unresolved_obligations: Vec<String>,
    pub static_compatibility: af_contract::ProofVerdict,
}

impl RootExportProofReport {
    #[must_use]
    pub fn dimension(&self, dimension: RootExportDimension) -> Option<&RootExportDimensionProof> {
        self.dimensions
            .iter()
            .find(|proof| proof.dimension == dimension)
    }
}

/// Runtime authorization is explicitly not established by a static proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeAuthorityStatus {
    NotEstablished,
}

/// Capability that the exact static Root/Export compatibility was proven.
/// It carries no body, publisher, budget, or runtime claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedRootExportCompatibility {
    report: RootExportProofReport,
}

impl CheckedRootExportCompatibility {
    #[must_use]
    pub fn report(&self) -> &RootExportProofReport {
        &self.report
    }
}

const APPLICABLE_SPECS: &[(&str, &str, &[u8])] = &[
    (
        "agentflow.component.contract.semantics@1",
        "spec/component/component-contract.md",
        include_bytes!("../../../spec/component/component-contract.md"),
    ),
    (
        "agentflow.component.relation-catalogs.semantics@1",
        "spec/component/relation-catalogs.md",
        include_bytes!("../../../spec/component/relation-catalogs.md"),
    ),
    (
        "agentflow.effect.signature.semantics@1",
        "spec/effect/effect-signature.md",
        include_bytes!("../../../spec/effect/effect-signature.md"),
    ),
    (
        "agentflow.execution.requirements.semantics@1",
        "spec/execution/execution-requirements.md",
        include_bytes!("../../../spec/execution/execution-requirements.md"),
    ),
    (
        "agentflow.failure.row.semantics@1",
        "spec/failure/failure-row.md",
        include_bytes!("../../../spec/failure/failure-row.md"),
    ),
    (
        "agentflow.information.flow.semantics@1",
        "spec/information-flow/information-flow.md",
        include_bytes!("../../../spec/information-flow/information-flow.md"),
    ),
    (
        "agentflow.recovery.contract.semantics@1",
        "spec/recovery/recovery-contract.md",
        include_bytes!("../../../spec/recovery/recovery-contract.md"),
    ),
    (
        "agentflow.resource.requirement.semantics@1",
        "spec/resource/resource-requirement.md",
        include_bytes!("../../../spec/resource/resource-requirement.md"),
    ),
    (
        "agentflow.semantic.advanced-control-contract-algebra.semantics@1",
        "spec/computation/advanced-control-contract-algebra.md",
        include_bytes!("../../../spec/computation/advanced-control-contract-algebra.md"),
    ),
    (
        "agentflow.semantic.control-contract-algebra.semantics@1",
        "spec/computation/control-contract-algebra.md",
        include_bytes!("../../../spec/computation/control-contract-algebra.md"),
    ),
    (
        "agentflow.semantic.ir.semantics@1",
        "spec/semantic/semantic-ir.md",
        include_bytes!("../../../spec/semantic/semantic-ir.md"),
    ),
    (
        "agentflow.semantic.leaf-contract-resolution.semantics@1",
        "spec/computation/leaf-contract-resolution.md",
        include_bytes!("../../../spec/computation/leaf-contract-resolution.md"),
    ),
    (
        "agentflow.semantic.scope-compensation-contract-algebra.semantics@1",
        "spec/computation/scope-compensation-contract-algebra.md",
        include_bytes!("../../../spec/computation/scope-compensation-contract-algebra.md"),
    ),
    (
        "agentflow.semantic.types@1",
        "spec/semantic/core-semantic-types.md",
        include_bytes!("../../../spec/semantic/core-semantic-types.md"),
    ),
    (
        "agentflow.usage.signature.semantics@1",
        "spec/usage/usage-signature.md",
        include_bytes!("../../../spec/usage/usage-signature.md"),
    ),
    (
        "agentflow.computation.root-export-contract-proof.semantics@1",
        "spec/computation/root-export-contract-proof.md",
        include_bytes!("../../../spec/computation/root-export-contract-proof.md"),
    ),
];

const APPLICABLE_RULE_COUNT: usize = 84; // AF-CONTRACT-0001..0022 and AF-SEMANTIC-0111..0172.

fn applicable_rule_ids() -> Vec<String> {
    let mut ids = (1..=22)
        .map(|number| format!("AF-CONTRACT-{number:04}"))
        .chain((111..=172).map(|number| format!("AF-SEMANTIC-{number:04}")))
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

fn rule_content_identities(registry_content: &[u8]) -> Vec<RuleContentIdentity> {
    let Ok(registry) = serde_json::from_slice::<serde_json::Value>(registry_content) else {
        return Vec::new();
    };
    let rows = registry
        .get("rules")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut result = applicable_rule_ids()
        .into_iter()
        .filter_map(|rule_id| {
            let row = rows.iter().find(|row| {
                row.get("rule_id").and_then(serde_json::Value::as_str) == Some(rule_id.as_str())
            })?;
            Some(RuleContentIdentity {
                rule_id,
                normative_source: row.get("normative_source")?.as_str()?.to_owned(),
                source_anchor: row.get("source_anchor")?.as_str()?.to_owned(),
                short_statement: row.get("short_statement")?.as_str()?.to_owned(),
            })
        })
        .collect::<Vec<_>>();
    result.sort_by(|left, right| left.rule_id.cmp(&right.rule_id));
    result
}

/// Current exact compiler semantics pin for this implementation build.
#[must_use]
pub fn current_compiler_semantics_identity() -> CompilerSemanticsIdentity {
    let registry_content = include_bytes!("../../../spec/registry/specifications.json").to_vec();
    let rule_registry_content = include_bytes!("../../../spec/registry/rules.json").to_vec();
    let mut specifications = APPLICABLE_SPECS
        .iter()
        .map(
            |(specification_id, logical_path, content)| SpecificationContentIdentity {
                specification_id: (*specification_id).to_owned(),
                logical_path: (*logical_path).to_owned(),
                content_sha256: sha256_hex(content),
                content: content.to_vec(),
            },
        )
        .collect::<Vec<_>>();
    specifications.sort_by(|left, right| left.specification_id.cmp(&right.specification_id));
    CompilerSemanticsIdentity {
        analysis_semantics_version: ROOT_EXPORT_ANALYSIS_SEMANTICS_VERSION.to_owned(),
        registry_content,
        rule_registry_content: rule_registry_content.clone(),
        specifications,
        rules: rule_content_identities(&rule_registry_content),
    }
}

fn compiler_semantics_is_current(identity: &CompilerSemanticsIdentity) -> bool {
    let current = current_compiler_semantics_identity();
    if identity != &current || identity.rules.len() != APPLICABLE_RULE_COUNT {
        return false;
    }
    if identity.specifications.len() != APPLICABLE_SPECS.len()
        || identity
            .specifications
            .iter()
            .any(|item| sha256_hex(&item.content) != item.content_sha256)
    {
        return false;
    }
    let Ok(spec_registry) = serde_json::from_slice::<serde_json::Value>(&identity.registry_content)
    else {
        return false;
    };
    let Some(spec_rows) = spec_registry
        .get("specifications")
        .and_then(serde_json::Value::as_array)
    else {
        return false;
    };
    for pin in &identity.specifications {
        let Some(row) = spec_rows.iter().find(|row| {
            row.get("specification_id")
                .and_then(serde_json::Value::as_str)
                == Some(pin.specification_id.as_str())
        }) else {
            return false;
        };
        if row
            .pointer("/normative_source/logical_path")
            .and_then(serde_json::Value::as_str)
            != Some(pin.logical_path.as_str())
            || row
                .pointer("/normative_source/content_digest/algorithm")
                .and_then(serde_json::Value::as_str)
                != Some("sha256")
            || row
                .pointer("/normative_source/content_digest/hex")
                .and_then(serde_json::Value::as_str)
                != Some(pin.content_sha256.as_str())
        {
            return false;
        }
    }
    let Ok(rule_registry) =
        serde_json::from_slice::<serde_json::Value>(&identity.rule_registry_content)
    else {
        return false;
    };
    let Some(rule_rows) = rule_registry
        .get("rules")
        .and_then(serde_json::Value::as_array)
    else {
        return false;
    };
    identity.rules.iter().all(|pin| {
        rule_rows.iter().any(|row| {
            row.get("rule_id").and_then(serde_json::Value::as_str) == Some(pin.rule_id.as_str())
                && row
                    .get("normative_source")
                    .and_then(serde_json::Value::as_str)
                    == Some(pin.normative_source.as_str())
                && row.get("source_anchor").and_then(serde_json::Value::as_str)
                    == Some(pin.source_anchor.as_str())
                && row
                    .get("short_statement")
                    .and_then(serde_json::Value::as_str)
                    == Some(pin.short_statement.as_str())
        })
    })
}

fn sha256_hex(content: &[u8]) -> String {
    hex::encode(Sha256::digest(content))
}

/// Produces all nine independent static compatibility proofs for one exact
/// checked IR, export, resolver snapshot, context, and derived root summary.
#[must_use]
pub fn analyze_root_export_assertion(
    ir: &CheckedIrDocument<'_>,
    exported: &CheckedComponentContract<'_, '_>,
    snapshot: &StaticComputationResolutionSnapshot<'_, '_, '_>,
    context: &StaticContractProofContext<'_>,
    root: &DerivedWorkflowContractSummary<'_, '_>,
    assertion: &RootExportAssertion,
) -> RootExportProofReport {
    let mut preflight = preflight(ir, exported, snapshot, context, root, assertion);
    let preflight_ok = preflight
        .iter()
        .all(|proof| proof.verdict == af_contract::ProofVerdict::Proven);
    let (dimensions, execution_children) = if preflight_ok {
        prove_all_dimensions(ir, exported.contract(), root, assertion, context)
    } else {
        (
            unproven_dimensions(
                assertion,
                "root/export dimensions were not evaluated because an exact assertion precondition failed",
            ),
            Vec::new(),
        )
    };
    let static_compatibility = if preflight
        .iter()
        .any(|proof| proof.verdict == af_contract::ProofVerdict::Violation)
        || dimensions
            .iter()
            .any(|proof| proof.proof.verdict == af_contract::ProofVerdict::Violation)
    {
        af_contract::ProofVerdict::Violation
    } else if dimensions
        .iter()
        .all(|proof| proof.proof.verdict == af_contract::ProofVerdict::Proven)
    {
        af_contract::ProofVerdict::Proven
    } else if dimensions
        .iter()
        .any(|proof| proof.proof.verdict == af_contract::ProofVerdict::RequiresEvidence)
    {
        af_contract::ProofVerdict::RequiresEvidence
    } else {
        af_contract::ProofVerdict::Unproven
    };
    if !assertion.subflow_dependencies.is_empty() {
        let mut body = DimensionProof::unproven(
            "subflow-body-evidence",
            RULE_SUBFLOW,
            "resolved target pins do not establish trusted subflow body conformance",
        );
        body.verdict = af_contract::ProofVerdict::RequiresEvidence;
        preflight.push(body);
    }
    let mut unresolved_obligations = Vec::new();
    if dimensions
        .iter()
        .any(|proof| proof.proof.verdict == af_contract::ProofVerdict::Unproven)
    {
        unresolved_obligations.push(
            "one or more static dimensions require missing or unsupported typed evidence"
                .to_owned(),
        );
    }
    if dimensions
        .iter()
        .any(|proof| proof.proof.verdict == af_contract::ProofVerdict::RequiresEvidence)
    {
        unresolved_obligations.push("trusted implementation, subflow body, or runtime evidence is not part of this static checker".to_owned());
    }
    unresolved_obligations.push(
        "Scope authority attenuation is Unproven because frozen ScopeBody has no authority ceiling"
            .to_owned(),
    );
    unresolved_obligations.push(
        "Scope budget attenuation is Unproven because frozen ScopeBody has no budget ceiling"
            .to_owned(),
    );
    unresolved_obligations.push("trusted component implementation/subflow body conformance is RequiresEvidence and separate from static envelope compatibility".to_owned());
    unresolved_obligations.push("static compatibility creates no EffectGrant, ResourceHandle, reservation, receipt, or runtime authorization".to_owned());

    RootExportProofReport {
        assertion_identity: assertion.identity(),
        preflight,
        dimensions,
        execution_children,
        body_conformance: af_contract::ProofVerdict::RequiresEvidence,
        implementation_conformance: af_contract::ProofVerdict::RequiresEvidence,
        scope_authority_attenuation: af_contract::ProofVerdict::Unproven,
        scope_budget_attenuation: af_contract::ProofVerdict::Unproven,
        runtime_authority: RuntimeAuthorityStatus::NotEstablished,
        runtime_authority_evidence: af_contract::ProofVerdict::RequiresEvidence,
        publisher_authentication: RuntimeAuthorityStatus::NotEstablished,
        unresolved_obligations,
        static_compatibility,
    }
}

/// Issues a non-wire compatibility capability only when every static dimension
/// is independently proven. Body/runtime states are intentionally excluded.
pub fn check_root_export_assertion(
    ir: &CheckedIrDocument<'_>,
    exported: &CheckedComponentContract<'_, '_>,
    snapshot: &StaticComputationResolutionSnapshot<'_, '_, '_>,
    context: &StaticContractProofContext<'_>,
    root: &DerivedWorkflowContractSummary<'_, '_>,
    assertion: &RootExportAssertion,
) -> Result<CheckedRootExportCompatibility, Box<RootExportProofReport>> {
    let report = analyze_root_export_assertion(ir, exported, snapshot, context, root, assertion);
    if report.static_compatibility == af_contract::ProofVerdict::Proven {
        Ok(CheckedRootExportCompatibility { report })
    } else {
        Err(Box::new(report))
    }
}

fn preflight(
    ir: &CheckedIrDocument<'_>,
    exported: &CheckedComponentContract<'_, '_>,
    snapshot: &StaticComputationResolutionSnapshot<'_, '_, '_>,
    context: &StaticContractProofContext<'_>,
    root: &DerivedWorkflowContractSummary<'_, '_>,
    assertion: &RootExportAssertion,
) -> Vec<DimensionProof> {
    let mut proofs = Vec::new();
    let canonical = assertion.clone().canonicalized();
    let canonical_order = canonical == *assertion;
    let no_duplicate_targets = assertion_targets_unique(assertion);
    proofs.push(if canonical_order && no_duplicate_targets {
        DimensionProof::proven("assertion-inputs", RULE_ASSERTION_PIN, "typed mappings use deterministic order and contain no duplicate target/source identities")
    } else {
        DimensionProof::violation("assertion-inputs", RULE_ASSERTION_PIN, "typed mapping order is non-canonical or contains a duplicate identity")
    });

    let ir_digest = ir
        .source()
        .digest(context.modules)
        .map(|digest| digest_ref(&digest));
    let export_digest =
        component_contract_digest(exported, context).map(|digest| digest_ref(&digest));
    proofs.push(match (ir_digest, export_digest) {
        (Ok(ir_digest), Ok(export_digest))
            if ir_digest == assertion.exact_semantic_ir_digest
                && &ir_digest == root.root().source_semantic_ir_digest()
                && export_digest == assertion.exact_exported_component_contract_digest =>
        {
            DimensionProof::proven("exact-ir-export-pins", RULE_ASSERTION_PIN, "recomputed SemanticIrDigest and checked ComponentContractDigest exactly match the assertion and derived root")
        }
        (Ok(ir_digest), Ok(export_digest)) => {
            let _ = (ir_digest, export_digest);
            DimensionProof::violation("exact-ir-export-pins", RULE_ASSERTION_PIN, "assertion digest pin does not match the exact checked IR, export, or root derivation")
        }
        _ => DimensionProof::unproven("exact-ir-export-pins", RULE_ASSERTION_PIN, "exact checked digest could not be recomputed under the supplied current context"),
    });

    let same_resolver_snapshot = root.resolver_identity() == &snapshot.resolver_identity();
    let same_context = std::ptr::eq(exported.proof_context(), context)
        && snapshot.is_same_context(context)
        && std::ptr::eq(root.root().proof_context_reference().context(), context)
        && same_resolver_snapshot;
    let captured_manifest = ProofContextManifest::capture(context, snapshot);
    let manifest_matches = assertion.proof_context_manifest == captured_manifest;
    let mut context_proof = if same_context && manifest_matches {
        DimensionProof::proven(
            "proof-context",
            RULE_CONTEXT_MANIFEST,
            "checked export, resolver snapshot, root derivation, and complete typed content manifest use this exact context",
        )
    } else {
        DimensionProof::unproven(
            "proof-context",
            RULE_CONTEXT_MANIFEST,
            "pointer-identical context or content-complete catalog/relation/resolver manifest differs; re-derivation is required",
        )
    };
    if !same_resolver_snapshot {
        context_proof.reference = Some(resolver_identity_difference(
            root.resolver_identity(),
            &snapshot.resolver_identity(),
        ));
    }
    proofs.push(context_proof);

    let semantics_matches = compiler_semantics_is_current(&assertion.compiler_semantics_identity);
    proofs.push(if semantics_matches {
        DimensionProof::proven("compiler-semantics", RULE_COMPILER_SEMANTICS, "exact applicable normative source bytes, registry contents, RuleIds, and analysis semantics version match this checker build")
    } else {
        DimensionProof::unproven("compiler-semantics", RULE_COMPILER_SEMANTICS, "normative content, RuleId set, or checker semantics version is stale, incomplete, or changed")
    });

    let dependencies_match = validate_subflow_dependencies(ir.source(), snapshot, assertion);
    proofs.push(match dependencies_match {
        Some(()) => DimensionProof::proven("subflow-dependency-pins", RULE_SUBFLOW, "direct and transitive subflow digest/export pins match the exact resolution snapshot"),
        None => DimensionProof::unproven("subflow-dependency-pins", RULE_SUBFLOW, "a subflow pin, transitive closure, or cycle/missing-target fact is incomplete or stale"),
    });
    proofs
}

fn resolver_identity_difference(
    root: &crate::resolution::ResolverSnapshotIdentity,
    current: &crate::resolution::ResolverSnapshotIdentity,
) -> String {
    if root.component_contract_digests() != current.component_contract_digests() {
        return "resolver component contract pin set differs".into();
    }
    for root_target in root.subflow_targets() {
        let target = root_target.semantic_ir_digest();
        let Some(current_target) = current
            .subflow_targets()
            .iter()
            .find(|item| item.semantic_ir_digest() == target)
        else {
            return format!(
                "resolver target {} is absent from the current snapshot",
                target.hex
            );
        };
        if root_target.direct_dependencies() != current_target.direct_dependencies() {
            return format!(
                "resolver target {} direct dependency closure differs",
                target.hex
            );
        }
        if root_target.exported_contract_digest() != current_target.exported_contract_digest() {
            return format!(
                "resolver target {} exported contract digest differs: root={}, current={}",
                target.hex,
                optional_digest(root_target.exported_contract_digest()),
                optional_digest(current_target.exported_contract_digest()),
            );
        }
    }
    if current.subflow_targets().len() > root.subflow_targets().len() {
        return "current resolver snapshot has additional subflow targets".into();
    }
    "resolver dependency identity differs".into()
}

fn optional_digest(digest: Option<&DigestRefValue>) -> String {
    digest.map_or_else(
        || "absent".into(),
        |value| format!("{}/{}/{}", value.algorithm, value.domain, value.hex),
    )
}

fn digest_ref(digest: &af_core::DigestRef) -> DigestRefValue {
    DigestRefValue {
        algorithm: digest.alg().as_str().to_owned(),
        domain: digest.domain().as_str().to_owned(),
        hex: digest.hex().to_owned(),
    }
}

fn digest_key(value: &DigestRefValue) -> (&str, &str, &str) {
    (&value.algorithm, &value.domain, &value.hex)
}

fn effect_mapping_key(
    value: &RootEffectCoordinateBinding,
) -> (
    AtomKey,
    String,
    String,
    String,
    u32,
    String,
    String,
    String,
    usize,
) {
    (
        AtomKey::of(&value.export_atom),
        value.source.caller_semantic_ir_digest.algorithm.clone(),
        value.source.caller_semantic_ir_digest.domain.clone(),
        value.source.caller_semantic_ir_digest.hex.clone(),
        value.source.source_node.raw(),
        value.source.source_contract_digest.algorithm.clone(),
        value.source.source_contract_digest.domain.clone(),
        value.source.source_contract_digest.hex.clone(),
        value.source.source_entry_ordinal,
    )
}

fn recovery_mapping_key(
    value: &RootRecoveryCoordinateBinding,
) -> (
    AtomKey,
    u32,
    AtomKey,
    String,
    String,
    String,
    u32,
    String,
    String,
    String,
    usize,
) {
    (
        AtomKey::of(&value.export_action_identity.forward_effect),
        match value.export_action_identity.action_kind {
            af_contract::RecoveryActionKind::Reconciliation => 1,
            af_contract::RecoveryActionKind::Compensation => 2,
        },
        AtomKey::of(&value.export_action_identity.action_effect),
        value.source.caller_semantic_ir_digest.algorithm.clone(),
        value.source.caller_semantic_ir_digest.domain.clone(),
        value.source.caller_semantic_ir_digest.hex.clone(),
        value.source.source_node.raw(),
        value.source.source_contract_digest.algorithm.clone(),
        value.source.source_contract_digest.domain.clone(),
        value.source.source_contract_digest.hex.clone(),
        value.source.forward_entry_ordinal,
    )
}

type FlowMappingSortKey = (
    u8,
    Option<(FlowKind, FlowSink)>,
    Option<FlowSourceRef>,
    String,
    String,
    String,
    u32,
    u64,
);

fn flow_mapping_key(value: &RootFlowCoordinateBinding) -> FlowMappingSortKey {
    let (kind, sink, source) = match (&value.export_sink, &value.export_source) {
        (Some(sink), _) => (0, Some((sink.kind, sink.sink.clone())), None),
        (None, Some(source)) => (1, None, Some(source.clone())),
        (None, None) => (2, None, None),
    };
    (
        kind,
        sink,
        source,
        value.source_contract_digest.algorithm.clone(),
        value.source_contract_digest.domain.clone(),
        value.source_contract_digest.hex.clone(),
        value.source_node.raw(),
        value.flow_mapping_ordinal,
    )
}

fn assertion_targets_unique(assertion: &RootExportAssertion) -> bool {
    fn has_duplicates<T: PartialEq>(items: impl IntoIterator<Item = T>) -> bool {
        let mut seen = Vec::new();
        for item in items {
            if seen.contains(&item) {
                return true;
            }
            seen.push(item);
        }
        false
    }
    !has_duplicates(
        assertion
            .input_mappings
            .iter()
            .map(|item| item.export_input_port.clone()),
    ) && !has_duplicates(
        assertion
            .output_mappings
            .iter()
            .map(|item| item.export_output_port.clone()),
    ) && !has_duplicates(
        assertion
            .resource_mappings
            .iter()
            .map(|item| item.export_resource_port.clone()),
    ) && !has_duplicates(
        assertion
            .effect_mappings
            .iter()
            .map(|item| item.source.clone()),
    ) && !has_duplicates(assertion.flow_mappings.iter().map(|item| {
        (
            item.source_node,
            item.source_contract_digest.clone(),
            item.flow_mapping_ordinal,
        )
    })) && !has_duplicates(
        assertion
            .recovery_mappings
            .iter()
            .map(|item| item.source.clone()),
    ) && !has_duplicates(
        assertion
            .execution_mappings
            .iter()
            .map(|item| (item.source_node, item.source_contract_digest.clone())),
    ) && !has_duplicates(
        assertion
            .subflow_dependencies
            .iter()
            .map(|item| item.call_site),
    )
}

fn validate_subflow_dependencies(
    document: &SemanticIrDocument,
    snapshot: &StaticComputationResolutionSnapshot<'_, '_, '_>,
    assertion: &RootExportAssertion,
) -> Option<()> {
    let mut expected = Vec::new();
    for computation in &document.computations {
        let ComputationBody::CallSubflow(body) = &computation.body else {
            continue;
        };
        let reference = document
            .contracts
            .iter()
            .find(|entry| entry.id == body.contract)?;
        if !matches!(reference.kind, af_ir::ContractRefKind::Subflow) {
            return None;
        }
        let export_pin = snapshot.subflow_export_pin(&reference.digest).cloned();
        let transitive_targets = transitive_subflow_targets(snapshot, &reference.digest)?;
        let dependency = assertion
            .subflow_dependencies
            .iter()
            .find(|item| item.call_site == computation.id)?;
        if dependency.target_semantic_ir_digest != reference.digest
            || dependency.target_exported_contract_digest != export_pin
            || dependency.transitive_targets != transitive_targets
        {
            return None;
        }
        if let Some(target_identity) = &dependency.target_root_assertion_identity {
            let target = target_identity.assertion();
            if target.exact_semantic_ir_digest != reference.digest
                || Some(target.exact_exported_component_contract_digest.clone()) != export_pin
            {
                return None;
            }
        }
        expected.push(computation.id);
    }
    if expected.len() != assertion.subflow_dependencies.len() {
        return None;
    }
    Some(())
}

fn transitive_subflow_targets(
    snapshot: &StaticComputationResolutionSnapshot<'_, '_, '_>,
    direct: &DigestRefValue,
) -> Option<Vec<SubflowTargetIdentity>> {
    fn visit(
        snapshot: &StaticComputationResolutionSnapshot<'_, '_, '_>,
        current: &DigestRefValue,
        path: &mut Vec<DigestRefValue>,
        seen: &mut Vec<DigestRefValue>,
    ) -> Option<()> {
        if path.contains(current) {
            return None;
        }
        if seen.contains(current) {
            return Some(());
        }
        let dependencies = snapshot.subflow_dependencies(current)?;
        path.push(current.clone());
        for dependency in dependencies {
            snapshot.subflow_dependencies(dependency)?;
            visit(snapshot, dependency, path, seen)?;
        }
        path.pop();
        seen.push(current.clone());
        Some(())
    }
    let direct_dependencies = snapshot.subflow_dependencies(direct)?;
    let mut seen = Vec::new();
    let mut path = vec![direct.clone()];
    for dependency in direct_dependencies {
        visit(snapshot, dependency, &mut path, &mut seen)?;
    }
    let mut targets = seen
        .into_iter()
        .map(|semantic_ir_digest| SubflowTargetIdentity {
            exported_contract_digest: snapshot.subflow_export_pin(&semantic_ir_digest).cloned(),
            semantic_ir_digest,
        })
        .collect::<Vec<_>>();
    targets.sort_by(|left, right| {
        digest_key(&left.semantic_ir_digest).cmp(&digest_key(&right.semantic_ir_digest))
    });
    Some(targets)
}

fn unproven_dimensions(
    assertion: &RootExportAssertion,
    reason: &str,
) -> Vec<RootExportDimensionProof> {
    [
        (
            RootExportDimension::Input,
            "root-input-interface",
            RULE_INPUT_MAP,
        ),
        (
            RootExportDimension::Output,
            "root-output-interface",
            RULE_OUTPUT_MAP,
        ),
        (RootExportDimension::Failure, "failure", RULE_PHI),
        (RootExportDimension::Effect, "effects", RULE_EPSILON),
        (
            RootExportDimension::Resource,
            "resources",
            RULE_RESOURCE_MAP,
        ),
        (
            RootExportDimension::InformationFlow,
            "information-flow",
            RULE_FLOW,
        ),
        (RootExportDimension::Usage, "usage", RULE_USAGE),
        (RootExportDimension::Recovery, "recovery", RULE_RECOVERY),
        (RootExportDimension::Execution, "execution", RULE_EXECUTION),
    ]
    .into_iter()
    .map(|(dimension, name, rule)| RootExportDimensionProof {
        dimension,
        proof: DimensionProof::unproven(name, rule, reason),
        source_facts: Vec::new(),
        mapped_coordinates: coordinate_witnesses(assertion, dimension),
        mapped_coordinate_count: mapping_count(assertion, dimension),
    })
    .collect()
}

fn mapping_count(assertion: &RootExportAssertion, dimension: RootExportDimension) -> usize {
    match dimension {
        RootExportDimension::Input => assertion.input_mappings.len(),
        RootExportDimension::Output => assertion.output_mappings.len(),
        RootExportDimension::Failure | RootExportDimension::Usage => 0,
        RootExportDimension::Effect => assertion.effect_mappings.len(),
        RootExportDimension::Resource => assertion.resource_mappings.len(),
        RootExportDimension::InformationFlow => assertion.flow_mappings.len(),
        RootExportDimension::Recovery => assertion.recovery_mappings.len(),
        RootExportDimension::Execution => assertion.execution_mappings.len(),
    }
}

fn prove_all_dimensions(
    ir: &CheckedIrDocument<'_>,
    exported: &ComponentContract,
    root: &DerivedWorkflowContractSummary<'_, '_>,
    assertion: &RootExportAssertion,
    context: &StaticContractProofContext<'_>,
) -> (Vec<RootExportDimensionProof>, Vec<RootExecutionChildProof>) {
    let mut leaves = Vec::new();
    collect_leaf_summaries(root.root(), &mut leaves);
    let dataflow = ir.analyze_dataflow().ok();
    let semantic_ir_digest = root.root().source_semantic_ir_digest().clone();

    let input = prove_inputs(
        ir.source(),
        &leaves,
        dataflow.as_ref(),
        exported,
        assertion,
        context,
        &semantic_ir_digest,
    );
    let output = prove_outputs(
        ir.source(),
        dataflow.as_ref(),
        root,
        exported,
        assertion,
        context,
        &semantic_ir_digest,
    );
    let phi = prove_failures(root, exported, &semantic_ir_digest);
    let resource = prove_resources(
        ir.source(),
        root,
        exported,
        assertion,
        context,
        &semantic_ir_digest,
    );
    let epsilon = prove_effects(root, exported, assertion, context, &semantic_ir_digest);
    let flow = prove_flow(root, exported, assertion, context, &semantic_ir_digest);
    let usage = prove_usage(root, exported, context, &semantic_ir_digest);
    let recovery = prove_recovery(root, exported, assertion, context, &semantic_ir_digest);
    let (execution, execution_children) =
        prove_execution(root, exported, assertion, context, &semantic_ir_digest);
    let mut dimensions = vec![
        input, output, phi, epsilon, resource, flow, usage, recovery, execution,
    ];
    for proof in &mut dimensions {
        proof.mapped_coordinates = coordinate_witnesses(assertion, proof.dimension);
    }
    (dimensions, execution_children)
}

fn coordinate_witnesses(
    assertion: &RootExportAssertion,
    dimension: RootExportDimension,
) -> Vec<RootExportCoordinateWitness> {
    match dimension {
        RootExportDimension::Input => assertion
            .input_mappings
            .iter()
            .cloned()
            .map(|item| RootExportCoordinateWitness::Input(Box::new(item)))
            .collect(),
        RootExportDimension::Output => assertion
            .output_mappings
            .iter()
            .cloned()
            .map(|item| RootExportCoordinateWitness::Output(Box::new(item)))
            .collect(),
        RootExportDimension::Failure | RootExportDimension::Usage => Vec::new(),
        RootExportDimension::Effect => assertion
            .effect_mappings
            .iter()
            .cloned()
            .map(|item| RootExportCoordinateWitness::Effect(Box::new(item)))
            .collect(),
        RootExportDimension::Resource => assertion
            .resource_mappings
            .iter()
            .cloned()
            .map(|item| RootExportCoordinateWitness::Resource(Box::new(item)))
            .collect(),
        RootExportDimension::InformationFlow => assertion
            .flow_mappings
            .iter()
            .cloned()
            .map(|item| RootExportCoordinateWitness::Flow(Box::new(item)))
            .collect(),
        RootExportDimension::Recovery => assertion
            .recovery_mappings
            .iter()
            .cloned()
            .map(|item| RootExportCoordinateWitness::Recovery(Box::new(item)))
            .collect(),
        RootExportDimension::Execution => assertion
            .execution_mappings
            .iter()
            .cloned()
            .map(|item| RootExportCoordinateWitness::Execution(Box::new(item)))
            .collect(),
    }
}

fn collect_leaf_summaries<'context, 'universe>(
    node: &crate::control::DerivedControlContractSummary<'context, 'universe>,
    result: &mut Vec<crate::summary::ResolvedComputationLeaf<'context, 'universe>>,
) {
    if let Some(leaf) = node.leaf_summary() {
        result.push(leaf.clone());
    }
    for child in node.children() {
        collect_leaf_summaries(child, result);
    }
}

fn dimension_report(
    dimension: RootExportDimension,
    proof: DimensionProof,
    source_facts: Vec<RootExportSourceReference>,
    mapped_coordinate_count: usize,
) -> RootExportDimensionProof {
    RootExportDimensionProof {
        dimension,
        proof,
        source_facts,
        mapped_coordinates: Vec::new(),
        mapped_coordinate_count,
    }
}

fn root_source_reference(
    semantic_ir_digest: &DigestRefValue,
    computation: ComputationId,
    contract_digest: &DigestRefValue,
    ordinal: Option<u64>,
) -> RootExportSourceReference {
    RootExportSourceReference {
        semantic_ir_digest: semantic_ir_digest.clone(),
        computation: Some(computation),
        contract_digest: Some(contract_digest.clone()),
        source_ordinal: ordinal,
    }
}

fn merge_verdict(
    left: af_contract::ProofVerdict,
    right: af_contract::ProofVerdict,
) -> af_contract::ProofVerdict {
    use af_contract::ProofVerdict::{Proven, RequiresEvidence, Unproven, Violation};
    match (left, right) {
        (Violation, _) | (_, Violation) => Violation,
        (RequiresEvidence, _) | (_, RequiresEvidence) => RequiresEvidence,
        (Unproven, _) | (_, Unproven) => Unproven,
        (Proven, Proven) => Proven,
    }
}

fn exact_schema_ref(
    document: &SemanticIrDocument,
    type_id: af_semantic::SemanticTypeId,
) -> Option<&TypeSchemaRef> {
    document
        .types
        .iter()
        .find(|node| node.id == type_id)
        .and_then(|node| match &node.body {
            af_semantic::SemanticTypeNodeBody::Schema(schema) => Some(schema),
            _ => None,
        })
}

fn schema_structural_type(reference: &TypeSchemaRef) -> StructuralType {
    StructuralType::Schema(reference.clone())
}

fn leaf_contract_digest(
    leaf: &crate::summary::ResolvedComputationLeaf<'_, '_>,
) -> Option<DigestRefValue> {
    match leaf.target()? {
        ResolvedTarget::Component {
            contract_digest, ..
        } => Some(contract_digest.clone()),
        ResolvedTarget::Subflow {
            exported_contract_digest,
            ..
        } => exported_contract_digest.clone(),
        ResolvedTarget::AwaitSignal { .. } => None,
    }
}

fn prove_inputs(
    document: &SemanticIrDocument,
    leaves: &[crate::summary::ResolvedComputationLeaf<'_, '_>],
    dataflow: Option<&af_ir::SemanticDataflowAnalysis>,
    exported: &ComponentContract,
    assertion: &RootExportAssertion,
    context: &StaticContractProofContext<'_>,
    semantic_ir_digest: &DigestRefValue,
) -> RootExportDimensionProof {
    use af_contract::ProofVerdict::{Proven, Unproven, Violation};
    let mut verdict = Proven;
    let mut reasons = Vec::new();
    let root_parameters = &document.run_parameters;
    if root_parameters.len() != exported.inputs.len() {
        verdict = Violation;
        reasons.push("the exact root RunParameter set and exported public input set have different cardinality".to_owned());
    }
    let mappings = &assertion.input_mappings;
    if mappings.len() != exported.inputs.len() || mappings.len() != root_parameters.len() {
        verdict = merge_verdict(verdict, Unproven);
        reasons.push("input mapping is not a complete bijection".to_owned());
    }
    let mut seen_parameters = Vec::new();
    let mut seen_slots = Vec::new();
    let mut covered_ports = Vec::new();
    for mapping in mappings {
        if mapping.export_input_port.exported_contract_digest
            != assertion.exact_exported_component_contract_digest
            || !exported
                .inputs
                .iter()
                .any(|port| port.port_no == mapping.export_input_port.port_no)
        {
            verdict = merge_verdict(verdict, Unproven);
            reasons.push(
                "an input map target is outside the exact exported contract namespace".to_owned(),
            );
            continue;
        }
        if covered_ports.contains(&mapping.export_input_port.port_no) {
            verdict = Violation;
            reasons.push("duplicate exported input target".to_owned());
            continue;
        }
        covered_ports.push(mapping.export_input_port.port_no);
        if seen_parameters.contains(&mapping.run_parameter_id)
            || seen_slots.contains(&mapping.root_value_slot)
        {
            verdict = Violation;
            reasons.push(
                "two independent exported inputs reuse one root RunParameter or ValueSlot"
                    .to_owned(),
            );
        }
        seen_parameters.push(mapping.run_parameter_id);
        seen_slots.push(mapping.root_value_slot);
        let Some(parameter) = root_parameters
            .iter()
            .find(|item| item.id == mapping.run_parameter_id)
        else {
            verdict = merge_verdict(verdict, Unproven);
            reasons.push("mapped RunParameterId is absent from checked Semantic IR".to_owned());
            continue;
        };
        if parameter.value_slot != mapping.root_value_slot {
            verdict = merge_verdict(verdict, Violation);
            reasons.push(
                "mapped root ValueSlot is not the exact producer of the RunParameter".to_owned(),
            );
        }
        let Some(root_type_ref) = exact_schema_ref(document, parameter.type_id) else {
            verdict = merge_verdict(verdict, Unproven);
            reasons.push("root input type has no exact nominal TypeSchemaRef".to_owned());
            continue;
        };
        if context.schema_universe.resolve(root_type_ref).is_err() {
            verdict = merge_verdict(verdict, Unproven);
            reasons.push(
                "root input TypeSchemaRef does not resolve in the exact type universe".to_owned(),
            );
        }
        let Some(export_port) = exported
            .inputs
            .iter()
            .find(|port| port.port_no == mapping.export_input_port.port_no)
        else {
            continue;
        };
        if root_type_ref != &export_port.type_ref {
            verdict = merge_verdict(verdict, Violation);
            reasons
                .push("root and exported input nominal type pins are not exactly equal".to_owned());
        }
        if mapping.export_type != export_port.type_ref
            || mapping.root_type != schema_structural_type(root_type_ref)
        {
            verdict = merge_verdict(verdict, Unproven);
            reasons.push(
                "input mapping type evidence differs from checked root/export type identities"
                    .to_owned(),
            );
        }
    }

    let mut expected_use_sites = Vec::new();
    for leaf in leaves {
        let Some(source_contract_digest) = leaf_contract_digest(leaf) else {
            if leaf.kind() != crate::summary::LeafKind::Await {
                verdict = merge_verdict(verdict, Unproven);
                reasons.push(
                    "a computation input source has no resolved public contract pin".to_owned(),
                );
            }
            continue;
        };
        for source in leaf.input_mappings() {
            let crate::summary::ScopedPortBinding::InputExpression(expression) = source.binding
            else {
                verdict = merge_verdict(verdict, Violation);
                reasons.push("leaf input mapping has a non-expression producer".to_owned());
                continue;
            };
            let Some(definition) = document
                .expressions
                .iter()
                .find(|item| item.id == expression)
            else {
                verdict = merge_verdict(verdict, Unproven);
                reasons.push("leaf input expression is absent".to_owned());
                continue;
            };
            let ExpressionBody::ValueSlot(root_slot) = definition.body else {
                verdict = merge_verdict(verdict, Unproven);
                reasons.push(
                    "input expression is not an identity-preserving root value mapping".to_owned(),
                );
                continue;
            };
            let Some(root_mapping) = mappings
                .iter()
                .find(|item| item.root_value_slot == root_slot)
            else {
                verdict = merge_verdict(verdict, Unproven);
                reasons.push("leaf input does not map to an asserted root RunParameter".to_owned());
                continue;
            };
            let Some(exact_type) = source.exact_type.as_ref() else {
                verdict = merge_verdict(verdict, Unproven);
                reasons.push("leaf input use-site lacks exact nominal type evidence".to_owned());
                continue;
            };
            if exact_type != &root_mapping.export_type {
                verdict = merge_verdict(verdict, Violation);
                reasons.push(
                    "leaf input use-site type differs from the mapped public root input".to_owned(),
                );
            }
            let Some(dataflow) = dataflow else {
                verdict = merge_verdict(verdict, Unproven);
                reasons.push("Phase 3C use-site dataflow is unavailable".to_owned());
                continue;
            };
            let dependency_slots = dataflow
                .expression_value_dependencies()
                .get(&expression.raw())
                .map(|slots| {
                    slots
                        .iter()
                        .copied()
                        .map(ValueSlotId::from_raw)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if dependency_slots != vec![root_slot] {
                verdict = merge_verdict(verdict, Unproven);
                reasons.push(
                    "Phase 3C expression dependencies do not establish direct root input identity"
                        .to_owned(),
                );
            }
            expected_use_sites.push(RootInputUseSite {
                computation: leaf.computation(),
                source_contract_digest: source_contract_digest.clone(),
                source_component_port: source.contract_port_no,
                input_expression: expression,
                exact_type: exact_type.clone(),
                dependency_slots,
            });
        }
    }
    expected_use_sites.sort_by(|a, b| {
        (
            a.computation,
            digest_key(&a.source_contract_digest),
            a.source_component_port,
            a.input_expression,
        )
            .cmp(&(
                b.computation,
                digest_key(&b.source_contract_digest),
                b.source_component_port,
                b.input_expression,
            ))
    });
    for mapping in mappings {
        let mut actual = mapping.use_sites.clone();
        actual.sort_by(|a, b| {
            (
                a.computation,
                digest_key(&a.source_contract_digest),
                a.source_component_port,
                a.input_expression,
            )
                .cmp(&(
                    b.computation,
                    digest_key(&b.source_contract_digest),
                    b.source_component_port,
                    b.input_expression,
                ))
        });
        let expected = expected_use_sites
            .iter()
            .filter(|site| site.dependency_slots == vec![mapping.root_value_slot])
            .cloned()
            .collect::<Vec<_>>();
        if actual != expected {
            verdict = merge_verdict(verdict, Unproven);
            reasons.push("asserted input use-site set differs from checked leaf expressions and Phase 3C dependencies".to_owned());
        }
    }

    for port in &exported.inputs {
        if !covered_ports.contains(&port.port_no) && root_parameters.len() == exported.inputs.len()
        {
            verdict = merge_verdict(verdict, Unproven);
            reasons.push("an exported input has no typed assertion mapping".to_owned());
        }
    }
    let reason = join_reasons(
        &reasons,
        "all exported inputs bijectively map to exact root parameters, nominal types, and checked use sites",
    );
    dimension_report(
        RootExportDimension::Input,
        proof_for("root-input-interface", RULE_INPUT_MAP, verdict, reason),
        leaves
            .iter()
            .filter_map(|leaf| {
                leaf_contract_digest(leaf).map(|contract| {
                    root_source_reference(semantic_ir_digest, leaf.computation(), &contract, None)
                })
            })
            .collect(),
        mappings.len(),
    )
}

fn prove_outputs(
    document: &SemanticIrDocument,
    dataflow: Option<&af_ir::SemanticDataflowAnalysis>,
    root: &DerivedWorkflowContractSummary<'_, '_>,
    exported: &ComponentContract,
    assertion: &RootExportAssertion,
    context: &StaticContractProofContext<'_>,
    semantic_ir_digest: &DigestRefValue,
) -> RootExportDimensionProof {
    use af_contract::ProofVerdict::{Proven, Unproven, Violation};
    let mut verdict = Proven;
    let mut reasons = Vec::new();
    if document.workflow_outputs.len() < exported.outputs.len() {
        verdict = Violation;
        reasons.push(
            "checked root declares fewer WorkflowOutputs than the export guarantees".to_owned(),
        );
    }
    let mut covered_ports = Vec::new();
    let mut covered_names = Vec::new();
    for mapping in &assertion.output_mappings {
        if mapping.export_output_port.exported_contract_digest
            != assertion.exact_exported_component_contract_digest
        {
            verdict = merge_verdict(verdict, Unproven);
            reasons.push(
                "output mapping target is in a different exported contract namespace".to_owned(),
            );
            continue;
        }
        if covered_ports.contains(&mapping.export_output_port.port_no)
            || covered_names.contains(&mapping.workflow_output_name)
        {
            verdict = Violation;
            reasons.push("duplicate output port or WorkflowOutput target".to_owned());
            continue;
        }
        covered_ports.push(mapping.export_output_port.port_no);
        covered_names.push(mapping.workflow_output_name.clone());
        let Some(export_port) = exported
            .outputs
            .iter()
            .find(|port| port.port_no == mapping.export_output_port.port_no)
        else {
            verdict = merge_verdict(verdict, Unproven);
            reasons.push("mapped output port is absent from the exact export".to_owned());
            continue;
        };
        let Some(output) = document
            .workflow_outputs
            .iter()
            .find(|output| output.name == mapping.workflow_output_name)
        else {
            verdict = merge_verdict(verdict, Unproven);
            reasons.push("asserted WorkflowOutput name is absent; the mapping cannot establish which checked output implements this port".to_owned());
            continue;
        };
        if output.expression != mapping.workflow_output_expression {
            verdict = merge_verdict(verdict, Unproven);
            reasons.push(
                "WorkflowOutput expression differs from the exact assertion mapping".to_owned(),
            );
            continue;
        }
        let Some(expression) = document
            .expressions
            .iter()
            .find(|item| item.id == output.expression)
        else {
            verdict = merge_verdict(verdict, Unproven);
            reasons.push("mapped output expression is absent".to_owned());
            continue;
        };
        let Some(result_type) = exact_schema_ref(document, expression.result_type_id) else {
            verdict = merge_verdict(verdict, Unproven);
            reasons.push("output expression result has no exact nominal TypeSchemaRef".to_owned());
            continue;
        };
        if context.schema_universe.resolve(result_type).is_err() {
            verdict = merge_verdict(verdict, Unproven);
            reasons.push(
                "output TypeSchemaRef does not resolve in the exact type universe".to_owned(),
            );
        }
        if result_type != &export_port.type_ref {
            verdict = merge_verdict(verdict, Violation);
            reasons.push("root output and exported output nominal types differ".to_owned());
        }
        if mapping.producer_evidence.exact_result_type != *result_type
            || mapping.root_result_type != schema_structural_type(result_type)
        {
            verdict = merge_verdict(verdict, Unproven);
            reasons.push(
                "output mapping type witness differs from the exact checked result type".to_owned(),
            );
        }
        let dependency_slots = dataflow
            .and_then(|analysis| {
                analysis
                    .expression_value_dependencies()
                    .get(&output.expression.raw())
            })
            .map(|slots| {
                slots
                    .iter()
                    .copied()
                    .map(ValueSlotId::from_raw)
                    .collect::<Vec<_>>()
            });
        let Some(dependency_slots) = dependency_slots else {
            verdict = merge_verdict(verdict, Unproven);
            reasons.push("Phase 3C producer/dominance evidence is unavailable".to_owned());
            continue;
        };
        if mapping.producer_evidence.dependency_slots != dependency_slots {
            verdict = merge_verdict(verdict, Unproven);
            reasons.push(
                "output producer evidence does not match the checked expression dependency set"
                    .to_owned(),
            );
        }
        let valid_exit_values = dataflow.is_some_and(|analysis| {
            dependency_slots
                .iter()
                .all(|slot| analysis.workflow_exit_values().contains(&slot.raw()))
        });
        if !valid_exit_values {
            verdict = Violation;
            reasons.push(
                "output expression consumes a value not available at workflow exit".to_owned(),
            );
        }
    }
    if assertion.output_mappings.len() != exported.outputs.len() {
        verdict = merge_verdict(verdict, Unproven);
        reasons.push("output mapping does not cover every exported port exactly once".to_owned());
    }
    if document.workflow_outputs.len() > exported.outputs.len() {
        verdict = merge_verdict(verdict, Unproven);
        reasons.push("extra root WorkflowOutput has no frozen additive-interface rule".to_owned());
    }
    if !matches!(
        root.root().interface().outputs(),
        ControlDimensionValue::Complete(_)
    ) {
        verdict = merge_verdict(verdict, Unproven);
        reasons.push(
            "root control interface does not provide a complete all-path output summary".to_owned(),
        );
    }
    let reason = join_reasons(
        &reasons,
        "every exported output maps to an exact checked WorkflowOutput expression, producer path, and nominal type",
    );
    dimension_report(
        RootExportDimension::Output,
        proof_for("root-output-interface", RULE_OUTPUT_MAP, verdict, reason),
        vec![RootExportSourceReference {
            semantic_ir_digest: semantic_ir_digest.clone(),
            computation: None,
            contract_digest: None,
            source_ordinal: None,
        }],
        assertion.output_mappings.len(),
    )
}

fn proof_for(
    dimension: &'static str,
    rule_id: &'static str,
    verdict: af_contract::ProofVerdict,
    reason: String,
) -> DimensionProof {
    match verdict {
        af_contract::ProofVerdict::Proven => DimensionProof::proven(dimension, rule_id, reason),
        af_contract::ProofVerdict::Violation => {
            DimensionProof::violation(dimension, rule_id, reason)
        }
        af_contract::ProofVerdict::Unproven => DimensionProof::unproven(dimension, rule_id, reason),
        af_contract::ProofVerdict::RequiresEvidence => {
            let mut proof = DimensionProof::unproven(dimension, rule_id, reason);
            proof.verdict = af_contract::ProofVerdict::RequiresEvidence;
            proof
        }
    }
}

fn join_reasons(reasons: &[String], success: &str) -> String {
    if reasons.is_empty() {
        success.to_owned()
    } else {
        reasons.join("; ")
    }
}

fn prove_failures(
    root: &DerivedWorkflowContractSummary<'_, '_>,
    exported: &ComponentContract,
    semantic_ir_digest: &DigestRefValue,
) -> RootExportDimensionProof {
    let mut source_facts = Vec::new();
    let verdict = match root.root().failures().value() {
        ControlDimensionValue::Complete(failures) => {
            if af_failure::contains(failures, &exported.failures) {
                af_contract::ProofVerdict::Proven
            } else {
                af_contract::ProofVerdict::Violation
            }
        }
        ControlDimensionValue::Partial { .. }
        | ControlDimensionValue::Unknown { .. }
        | ControlDimensionValue::PerChild { .. }
        | ControlDimensionValue::NotApplicable => af_contract::ProofVerdict::Unproven,
    };
    source_facts.push(RootExportSourceReference {
        semantic_ir_digest: semantic_ir_digest.clone(),
        computation: Some(root.root().control_node()),
        contract_digest: None,
        source_ordinal: None,
    });
    let reason = match verdict {
        af_contract::ProofVerdict::Proven => "complete root business Failure set is contained by the exact nominal exported Phi row".to_owned(),
        af_contract::ProofVerdict::Violation => "a known possible root business Failure is absent from the complete exported Phi row".to_owned(),
        _ => "root Failure row is partial, unknown, or unavailable; timeout/cancellation/recovery outcomes are not treated as Phi".to_owned(),
    };
    dimension_report(
        RootExportDimension::Failure,
        proof_for("failure", RULE_PHI, verdict, reason),
        source_facts,
        0,
    )
}

fn resource_occurrences(
    root: &DerivedWorkflowContractSummary<'_, '_>,
) -> Result<Vec<ResourcePortOccurrence>, af_contract::ProofVerdict> {
    match root.root().resources().value() {
        ControlDimensionValue::Complete(resources) => Ok(resources.clone()),
        ControlDimensionValue::PerChild {
            contributions,
            unresolved_sources,
        } if unresolved_sources.is_empty() => Ok(contributions
            .iter()
            .flat_map(|item| item.value().iter().cloned())
            .collect()),
        ControlDimensionValue::NotApplicable
            if root
                .root()
                .resources()
                .proofs()
                .iter()
                .all(|proof| proof.verdict() == crate::proof::LeafVerdict::Proven) =>
        {
            Ok(Vec::new())
        }
        ControlDimensionValue::Partial { .. }
        | ControlDimensionValue::Unknown { .. }
        | ControlDimensionValue::PerChild { .. }
        | ControlDimensionValue::NotApplicable => Err(af_contract::ProofVerdict::Unproven),
    }
}

fn prove_resources(
    document: &SemanticIrDocument,
    root: &DerivedWorkflowContractSummary<'_, '_>,
    exported: &ComponentContract,
    assertion: &RootExportAssertion,
    context: &StaticContractProofContext<'_>,
    semantic_ir_digest: &DigestRefValue,
) -> RootExportDimensionProof {
    let mut verdict = af_contract::ProofVerdict::Proven;
    let mut reasons = Vec::new();
    let mut mapped_sources = Vec::new();
    let mut root_slots = Vec::new();
    let exported_ports = exported
        .resources
        .requirements
        .iter()
        .map(|item| item.port_no)
        .collect::<Vec<_>>();
    let mut leaves = Vec::new();
    collect_leaf_summaries(root.root(), &mut leaves);
    let mut expected_source_coordinates = Vec::new();
    for leaf in &leaves {
        let Some(source_contract_digest) = leaf_contract_digest(leaf) else {
            if !leaf.resource_mappings().is_empty() {
                verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
                reasons.push("root resource source has no exact source contract pin".to_owned());
            }
            continue;
        };
        for source in leaf.resource_mappings() {
            let mut alias_component_ports = leaf
                .resource_aliases()
                .iter()
                .find(|group| group.resource_slot == source.resource_slot)
                .map_or_else(
                    || vec![source.component_port_no],
                    |group| group.component_ports.clone(),
                );
            alias_component_ports.sort_unstable();
            alias_component_ports.dedup();
            expected_source_coordinates.push(crate::root_export::RootResourceSourceCoordinate {
                source_node: leaf.computation(),
                source_contract_digest: source_contract_digest.clone(),
                component_resource_port: source.component_port_no,
                resource_slot: source.resource_slot,
                alias_component_ports,
            });
        }
    }
    expected_source_coordinates.sort_by(|left, right| {
        (
            left.source_node,
            digest_key(&left.source_contract_digest),
            left.component_resource_port,
            left.resource_slot,
        )
            .cmp(&(
                right.source_node,
                digest_key(&right.source_contract_digest),
                right.component_resource_port,
                right.resource_slot,
            ))
    });
    let mut mapped_ports = Vec::new();
    for mapping in &assertion.resource_mappings {
        let port = mapping.export_resource_port.port_no;
        if mapping.export_resource_port.exported_contract_digest
            != assertion.exact_exported_component_contract_digest
            || !exported_ports.contains(&port)
        {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons.push(
                "resource map target is outside the exact export resource namespace".to_owned(),
            );
            continue;
        }
        if mapped_ports.contains(&port) {
            verdict = af_contract::ProofVerdict::Violation;
            reasons.push("duplicate exported resource requirement port mapping".to_owned());
        }
        if root_slots.contains(&mapping.root_resource_slot) {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons.push("one root ResourceSlot is aliased to distinct exported resource ports without a public alias rule".to_owned());
        }
        mapped_ports.push(port);
        root_slots.push(mapping.root_resource_slot);
        let expected_sources = expected_source_coordinates
            .iter()
            .filter(|source| source.resource_slot == mapping.root_resource_slot)
            .cloned()
            .collect::<Vec<_>>();
        if mapping.source_coordinates != expected_sources {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons.push("resource mapping source-port and alias-group chain differs from the checked leaf mappings".to_owned());
        }
        let Some(slot) = document
            .resources
            .iter()
            .find(|slot| slot.id == mapping.root_resource_slot)
        else {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons
                .push("mapped root ResourceSlotId is absent from checked Semantic IR".to_owned());
            continue;
        };
        let Some(export_requirement) = exported
            .resources
            .requirements
            .iter()
            .find(|item| item.port_no == port)
        else {
            continue;
        };
        if slot.requirement_type != export_requirement.resource_type.schema {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Violation);
            reasons.push("root resource slot and exported requirement use different exact TypeSchemaRef pins".to_owned());
        }
        if mapping.root_resource_type != slot.requirement_type
            || mapping.export_resource_type != export_requirement.resource_type
            || mapping.export_resource_type.schema != export_requirement.resource_type.schema
        {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons.push(
                "resource map type evidence differs from the checked root/export pins".to_owned(),
            );
        }
        if context
            .resources
            .resolve(&mapping.export_resource_type)
            .is_err()
        {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons.push(
                "mapped exported ResourceTypeRef or its attribute vocabulary is unresolved"
                    .to_owned(),
            );
        }
    }
    if assertion.resource_mappings.len() != exported_ports.len() {
        verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
        reasons.push("resource map does not cover every exported public resource port".to_owned());
    }
    let occurrences = match resource_occurrences(root) {
        Ok(value) => value,
        Err(value) => {
            verdict = merge_verdict(verdict, value);
            reasons.push("root resource occurrence closure is incomplete".to_owned());
            Vec::new()
        }
    };
    let mut aggregated: BTreeMap<u32, ResourceRequirement> = BTreeMap::new();
    for occurrence in &occurrences {
        let slot = occurrence.mapping().resource_slot;
        let Some(mapping) = assertion
            .resource_mappings
            .iter()
            .find(|item| item.root_resource_slot == slot)
        else {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons
                .push("root resource occurrence has no explicit exported-port mapping".to_owned());
            continue;
        };
        if !occurrence.mapping().required_attributes_proven {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons.push(
                "leaf resource attributes or scope visibility are not statically proven".to_owned(),
            );
        }
        let Some(requirement) = occurrence.requirement() else {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons.push("resource source has no resolved requirement body".to_owned());
            continue;
        };
        mapped_sources.push(root_source_reference(
            semantic_ir_digest,
            occurrence.source_node(),
            &target_contract_digest(occurrence.target())
                .unwrap_or_else(|| assertion.exact_exported_component_contract_digest.clone()),
            Some(u64::from(occurrence.mapping().component_port_no)),
        ));
        let Some(export_requirement) = exported
            .resources
            .requirements
            .iter()
            .find(|item| item.port_no == mapping.export_resource_port.port_no)
        else {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons.push("mapped root resource port has no exact exported requirement".to_owned());
            continue;
        };
        if requirement.resource_type.schema != export_requirement.resource_type.schema {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Violation);
            reasons.push("source requirement TypeSchemaRef differs from the explicitly mapped root/export resource".to_owned());
        }
        if mapping.export_resource_type != export_requirement.resource_type {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons.push(
                "resource map ResourceTypeRef differs from the exact exported requirement"
                    .to_owned(),
            );
        }
        let target = aggregated
            .entry(mapping.export_resource_port.port_no)
            .or_insert_with(|| ResourceRequirement {
                port_no: mapping.export_resource_port.port_no,
                resource_type: mapping.export_resource_type.clone(),
                required_attributes: Vec::new(),
            });
        if target.resource_type != requirement.resource_type {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons.push(
                "multiple source requirements sharing one root resource have different types"
                    .to_owned(),
            );
        }
        target
            .required_attributes
            .extend(requirement.required_attributes.iter().cloned());
    }
    for requirement in aggregated.values_mut() {
        requirement
            .required_attributes
            .sort_by(|left, right| (&left.key, &left.value).cmp(&(&right.key, &right.value)));
        requirement.required_attributes.dedup();
    }
    let root_row = ResourceRequirementRow {
        requirements: aggregated.into_values().collect(),
    };
    let namespace = format!(
        "{}:{}:resource",
        exported.component_identity.catalog, exported.component_identity.name
    );
    match af_resource::compatibility::requirement_not_stronger(
        &root_row,
        &namespace,
        &exported.resources,
        &namespace,
        context.resources,
    ) {
        Ok(af_resource::compatibility::ResourceCompatibilityProof::Proven) => {}
        Ok(af_resource::compatibility::ResourceCompatibilityProof::Unproven(_)) => {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons.push(
                "existing resource relation cannot prove the mapped requirement direction"
                    .to_owned(),
            );
        }
        Err(error)
            if error.kind() == af_resource::error::kind::RESOURCE_REQUIREMENT_STRENGTHENED =>
        {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Violation);
            reasons.push(
                "mapped root resource requirements are stronger than the exported row".to_owned(),
            );
        }
        Err(_) => {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons.push("resource relation or attribute evidence is unavailable".to_owned());
        }
    }
    let reason = join_reasons(
        &reasons,
        "root ResourceSlots map through exact source requirements to exported ports without strengthening the public R row",
    );
    dimension_report(
        RootExportDimension::Resource,
        proof_for("resources", RULE_RESOURCE_MAP, verdict, reason),
        mapped_sources,
        assertion.resource_mappings.len(),
    )
}

fn target_contract_digest(target: &ResolvedTarget) -> Option<DigestRefValue> {
    match target {
        ResolvedTarget::Component {
            contract_digest, ..
        } => Some(contract_digest.clone()),
        ResolvedTarget::Subflow {
            exported_contract_digest,
            ..
        } => exported_contract_digest.clone(),
        ResolvedTarget::AwaitSignal { .. } => None,
    }
}

fn exact_export_effect<'a>(
    signature: &'a EffectSignature,
    atom: &EffectAtom,
) -> Option<&'a EffectEntry> {
    match &signature.row {
        EffectRow::Closed(entries) => entries.iter().find(|entry| entry.atom == *atom),
        EffectRow::Open(row) => row.known.iter().find(|entry| entry.atom == *atom),
        EffectRow::Unknown => None,
    }
}

fn effect_source_identity(occurrence: &ScopedEffectOccurrence) -> RootEffectOccurrenceIdentity {
    RootEffectOccurrenceIdentity {
        caller_semantic_ir_digest: occurrence.caller_semantic_ir_digest.clone(),
        source_node: occurrence.computation,
        source_contract_digest: occurrence.source_contract_digest.clone(),
        source_entry_ordinal: occurrence.source_entry_ordinal,
        atom: occurrence.declared.atom.clone(),
        guard: occurrence.declared.guard.clone(),
    }
}

fn translate_guard(
    guard: &EffectGuard,
    computation: ComputationId,
    source_contract_digest: &DigestRefValue,
    assertion: &RootExportAssertion,
) -> Option<EffectGuard> {
    match guard {
        EffectGuard::Unconditional => Some(EffectGuard::Unconditional),
        EffectGuard::InputEquals(value) => {
            let root_mapping = assertion.input_mappings.iter().find(|mapping| {
                mapping.use_sites.iter().any(|site| {
                    site.computation == computation
                        && site.source_contract_digest == *source_contract_digest
                        && site.source_component_port == value.port_no
                })
            })?;
            Some(EffectGuard::InputEquals(
                af_effect::EffectGuardInputEquals {
                    port_no: root_mapping.export_input_port.port_no,
                    expected: value.expected.clone(),
                },
            ))
        }
    }
}

fn mapped_effect_target_is_valid(
    source: &EffectAtom,
    root_resource_slot: Option<ResourceSlotId>,
    target: &EffectAtom,
    assertion: &RootExportAssertion,
) -> bool {
    if source.operation != target.operation
        || source.qualifiers != target.qualifiers
        || source.resource_selector.scope_bindings != target.resource_selector.scope_bindings
    {
        return false;
    }
    let Some(slot) = root_resource_slot else {
        return false;
    };
    assertion.resource_mappings.iter().any(|mapping| {
        mapping.root_resource_slot == slot
            && mapping.export_resource_port.port_no == target.resource_selector.requirement_port_no
    })
}

fn prove_effects(
    root: &DerivedWorkflowContractSummary<'_, '_>,
    exported: &ComponentContract,
    assertion: &RootExportAssertion,
    context: &StaticContractProofContext<'_>,
    semantic_ir_digest: &DigestRefValue,
) -> RootExportDimensionProof {
    let mut verdict = af_contract::ProofVerdict::Proven;
    let mut reasons = Vec::new();
    let actual_occurrences = root.root().effect_occurrences();
    let mut coordinate_maps_valid = true;
    if assertion.effect_mappings.len() != actual_occurrences.len() {
        verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
        reasons.push("effect map does not account for every exact source occurrence".to_owned());
    }
    for occurrence in actual_occurrences {
        let source = effect_source_identity(occurrence);
        let Some(mapping) = assertion
            .effect_mappings
            .iter()
            .find(|mapping| mapping.source == source)
        else {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            coordinate_maps_valid = false;
            reasons.push(
                "a root effect occurrence has no exact source-preserving export map".to_owned(),
            );
            continue;
        };
        if mapping.root_resource_slot != occurrence.mapped_resource_slot
            || !mapped_effect_target_is_valid(
                &occurrence.declared.atom,
                occurrence.mapped_resource_slot,
                &mapping.export_atom,
                assertion,
            )
        {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            coordinate_maps_valid = false;
            reasons.push(
                "effect operation, qualifier, resource selector, or root slot mapping is not exact"
                    .to_owned(),
            );
        }
        let Some(guard) = translate_guard(
            &occurrence.declared.guard,
            occurrence.computation,
            &occurrence.source_contract_digest,
            assertion,
        ) else {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons.push(
                "effect guard input cannot be translated through an exact root input use site"
                    .to_owned(),
            );
            continue;
        };
        if mapping.export_guard != guard {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            coordinate_maps_valid = false;
            reasons.push(
                "asserted exported effect guard differs from the exact typed input translation"
                    .to_owned(),
            );
        }
    }
    if !root.root().effect_residuals().is_empty() {
        verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
        reasons.push(
            "one or more open-row effect residuals lack exact export coordinate evidence"
                .to_owned(),
        );
    }

    if !coordinate_maps_valid {
        return dimension_report(
            RootExportDimension::Effect,
            proof_for(
                "effects",
                RULE_EPSILON,
                verdict,
                join_reasons(&reasons, "effect coordinates are valid"),
            ),
            actual_occurrences
                .iter()
                .map(|item| {
                    root_source_reference(
                        semantic_ir_digest,
                        item.computation,
                        &item.source_contract_digest,
                        Some(item.source_entry_ordinal as u64),
                    )
                })
                .collect(),
            assertion.effect_mappings.len(),
        );
    }

    let root_effects = match root.root().effects().value() {
        ControlDimensionValue::Complete(signature) => Some(signature),
        _ => None,
    };
    let Some(root_effects) = root_effects else {
        verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
        reasons.push("root epsilon envelope is not complete".to_owned());
        return dimension_report(
            RootExportDimension::Effect,
            proof_for(
                "effects",
                RULE_EPSILON,
                verdict,
                join_reasons(&reasons, "root epsilon is completely mapped"),
            ),
            actual_occurrences
                .iter()
                .map(|item| {
                    root_source_reference(
                        semantic_ir_digest,
                        item.computation,
                        &item.source_contract_digest,
                        Some(item.source_entry_ordinal as u64),
                    )
                })
                .collect(),
            assertion.effect_mappings.len(),
        );
    };
    let remapped_root = remap_effect_signature(root_effects, &assertion.effect_mappings);
    let allow_guard_countermodel = !root_has_guard_sensitive_path(root.root());
    match remapped_root {
        Ok((mapped, widened)) => {
            match effect_containment_verdict(
                &mapped,
                &exported.effects,
                assertion,
                &exported.inputs,
                context.schema_universe,
                allow_guard_countermodel,
            ) {
                af_contract::ProofVerdict::Proven => {}
                value => {
                    verdict = merge_verdict(verdict, value);
                    reasons.push("mapped root epsilon is not proven inside the exact exported epsilon ceiling".to_owned());
                }
            }
            if widened && verdict != af_contract::ProofVerdict::Proven {
                verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
                reasons.push("control algebra widened guard provenance and the export relation needs narrower path evidence".to_owned());
            }
        }
        Err(value) => {
            verdict = merge_verdict(verdict, value);
            reasons.push("root epsilon row is open/unknown or cannot be mapped without losing source identity".to_owned());
        }
    }

    let recovery = root.root_recovery();
    let root_k_requirement = match recovery.epsilon_coverage_requirement().value() {
        ControlDimensionValue::Complete(signature) => {
            remap_effect_signature(signature, &assertion.effect_mappings)
                .ok()
                .map(|(value, _)| value)
        }
        _ => None,
    };
    if let Some(k_requirement) = root_k_requirement {
        match effect_containment_verdict(
            &k_requirement,
            root_effects,
            assertion,
            &exported.inputs,
            context.schema_universe,
            allow_guard_countermodel,
        ) {
            af_contract::ProofVerdict::Proven => {}
            value => {
                verdict = merge_verdict(verdict, value);
                reasons.push("root K action coverage is not established against the already-derived root epsilon".to_owned());
            }
        }
        match effect_containment_verdict(
            &k_requirement,
            &exported.effects,
            assertion,
            &exported.inputs,
            context.schema_universe,
            allow_guard_countermodel,
        ) {
            af_contract::ProofVerdict::Proven => {}
            value => {
                verdict = merge_verdict(verdict, value);
                reasons.push(
                    "root K action possibility is not separately covered by exported epsilon"
                        .to_owned(),
                );
            }
        }
    } else {
        verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
        reasons.push(
            "root K epsilon-coverage projection is unavailable or cannot be mapped".to_owned(),
        );
    }
    let static_recovery_verdict = recovery
        .proof()
        .dimensions()
        .iter()
        .filter(|proof| !proof.dimension().contains("eligibility"))
        .fold(af_contract::ProofVerdict::Proven, |current, proof| {
            merge_verdict(current, root_recovery_verdict(proof.verdict()))
        });
    if static_recovery_verdict != af_contract::ProofVerdict::Proven {
        verdict = merge_verdict(verdict, static_recovery_verdict);
        reasons.push(
            "Phase 4B.2E.1 root recovery static source/action proof is not complete and Proven"
                .to_owned(),
        );
    }
    dimension_report(
        RootExportDimension::Effect,
        proof_for(
            "effects",
            RULE_EPSILON,
            verdict,
            join_reasons(
                &reasons,
                "root epsilon and the separate root K coverage projection fit the exported epsilon ceiling",
            ),
        ),
        actual_occurrences
            .iter()
            .map(|item| {
                root_source_reference(
                    semantic_ir_digest,
                    item.computation,
                    &item.source_contract_digest,
                    Some(item.source_entry_ordinal as u64),
                )
            })
            .collect(),
        assertion.effect_mappings.len(),
    )
}

fn remap_effect_signature(
    signature: &EffectSignature,
    mappings: &[RootEffectCoordinateBinding],
) -> Result<(EffectSignature, bool), af_contract::ProofVerdict> {
    let entries = match &signature.row {
        EffectRow::Closed(entries) => entries,
        EffectRow::Open(_) | EffectRow::Unknown => return Err(af_contract::ProofVerdict::Unproven),
    };
    let mut widened = false;
    let mut mapped_entries = Vec::with_capacity(entries.len());
    for entry in entries {
        let candidates = mappings
            .iter()
            .filter(|mapping| mapping.source.atom == entry.atom)
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            if entries.is_empty() {
                continue;
            }
            return Err(af_contract::ProofVerdict::Unproven);
        }
        let atoms = candidates
            .iter()
            .map(|mapping| mapping.export_atom.clone())
            .collect::<Vec<_>>();
        if atoms.iter().any(|atom| atom != &atoms[0]) {
            return Err(af_contract::ProofVerdict::Unproven);
        }
        let exact_guard_mappings = candidates
            .iter()
            .filter(|mapping| mapping.source.guard == entry.guard)
            .collect::<Vec<_>>();
        let guards = if !exact_guard_mappings.is_empty() {
            exact_guard_mappings
                .iter()
                .map(|mapping| mapping.export_guard.clone())
                .collect::<Vec<_>>()
        } else if entry.guard == EffectGuard::Unconditional {
            widened = true;
            vec![EffectGuard::Unconditional]
        } else {
            return Err(af_contract::ProofVerdict::Unproven);
        };
        if guards.iter().any(|guard| guard != &guards[0]) {
            return Err(af_contract::ProofVerdict::Unproven);
        }
        mapped_entries.push(EffectEntry {
            atom: atoms[0].clone(),
            bounds: entry.bounds.clone(),
            guard: guards[0].clone(),
        });
    }
    Ok((
        EffectSignature {
            row: EffectRow::Closed(mapped_entries),
        },
        widened,
    ))
}

fn effect_containment_verdict(
    root: &EffectSignature,
    export: &EffectSignature,
    assertion: &RootExportAssertion,
    export_inputs: &[af_contract::ComponentInputPort],
    schemas: &af_semantic::TypeSchemaUniverse,
    allow_guard_countermodel: bool,
) -> af_contract::ProofVerdict {
    if matches!(export.row, EffectRow::Unknown | EffectRow::Open(_))
        || matches!(root.row, EffectRow::Unknown | EffectRow::Open(_))
    {
        return af_contract::ProofVerdict::Unproven;
    }
    if af_effect::contains(root, export) == af_effect::Proof::Proven {
        return af_contract::ProofVerdict::Proven;
    }
    let (EffectRow::Closed(root_entries), EffectRow::Closed(export_entries)) =
        (&root.row, &export.row)
    else {
        return af_contract::ProofVerdict::Unproven;
    };
    let mut claimed_export_slots = vec![false; export_entries.len()];
    for root_entry in root_entries {
        let same_operation_selector = export_entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                entry.atom.operation == root_entry.atom.operation
                    && entry.atom.qualifiers == root_entry.atom.qualifiers
                    && entry.atom.resource_selector.requirement_port_no
                        == root_entry.atom.resource_selector.requirement_port_no
                    && scope_is_covered(&root_entry.atom, &entry.atom)
            })
            .collect::<Vec<_>>();
        if same_operation_selector.is_empty() {
            return af_contract::ProofVerdict::Violation;
        }

        // The normative relation consumes one unused export slot per root
        // occurrence. Keep the fallback's partial analysis on that same
        // one-to-one basis; a slot already claimed by an earlier occurrence
        // cannot be reused to manufacture a second witness.
        if let Some((slot, _)) = same_operation_selector.iter().find(|(slot, entry)| {
            !claimed_export_slots[*slot]
                && entry.bounds.covers(&root_entry.bounds)
                && af_effect::guard_contained(&root_entry.guard, &entry.guard)
        }) {
            claimed_export_slots[*slot] = true;
            continue;
        }

        // If an otherwise complete row exists but was already consumed, the
        // formal algebra's finite-slot refinement is insufficient evidence;
        // do not reinterpret capacity contention as a semantic violation.
        if same_operation_selector.iter().any(|(slot, entry)| {
            claimed_export_slots[*slot]
                && entry.bounds.covers(&root_entry.bounds)
                && af_effect::guard_contained(&root_entry.guard, &entry.guard)
        }) {
            return af_contract::ProofVerdict::Unproven;
        }

        // A local contradiction is decisive only when there is exactly one
        // selector-compatible declaration. With multiple declarations, a
        // partial fact about one row cannot exclude another row; stay
        // conservative unless the normative full containment relation has
        // already produced a complete witness above.
        if same_operation_selector.len() != 1 {
            return af_contract::ProofVerdict::Unproven;
        }
        let (_, entry) = same_operation_selector[0];
        if af_effect::guard_contained(&root_entry.guard, &entry.guard)
            && !entry.bounds.covers(&root_entry.bounds)
        {
            return af_contract::ProofVerdict::Violation;
        }
        if allow_guard_countermodel
            && guard_counterexample(
                &root_entry.guard,
                &entry.guard,
                export_inputs,
                assertion,
                schemas,
            )
        {
            return af_contract::ProofVerdict::Violation;
        }
        return af_contract::ProofVerdict::Unproven;
    }
    // Reaching this path means the normative containment API did not prove
    // the complete signature. The fallback may explain a contradiction or
    // preserve Unproven, but it is never a second route to Proven.
    af_contract::ProofVerdict::Unproven
}

fn root_has_guard_sensitive_path(summary: &DerivedControlContractSummary<'_, '_>) -> bool {
    matches!(
        summary.operator(),
        ControlOperator::Select
            | ControlOperator::RaceFirstTerminal
            | ControlOperator::Handle { .. }
    ) || summary.children().iter().any(root_has_guard_sensitive_path)
}

fn scope_is_covered(inner: &EffectAtom, outer: &EffectAtom) -> bool {
    outer
        .resource_selector
        .scope_bindings
        .iter()
        .all(|binding| inner.resource_selector.scope_bindings.contains(binding))
}

fn guard_counterexample(
    possible: &EffectGuard,
    ceiling: &EffectGuard,
    export_inputs: &[af_contract::ComponentInputPort],
    assertion: &RootExportAssertion,
    schemas: &af_semantic::TypeSchemaUniverse,
) -> bool {
    fn type_of_port(
        port: u32,
        inputs: &[af_contract::ComponentInputPort],
    ) -> Option<&TypeSchemaRef> {
        inputs
            .iter()
            .find(|item| item.port_no == port)
            .map(|item| &item.type_ref)
    }
    fn has_alternate(reference: &TypeSchemaRef, schemas: &af_semantic::TypeSchemaUniverse) -> bool {
        match schemas.expand_named(&reference.identity()) {
            Ok(StructuralType::Primitive(
                af_semantic::SemanticPrimitiveType::Unit | af_semantic::SemanticPrimitiveType::Null,
            )) => false,
            Ok(StructuralType::Primitive(_)) => true,
            _ => false,
        }
    }
    match (possible, ceiling) {
        (EffectGuard::Unconditional, EffectGuard::InputEquals(outer)) => {
            type_of_port(outer.port_no, export_inputs)
                .is_some_and(|reference| has_alternate(reference, schemas))
        }
        (EffectGuard::InputEquals(inner), EffectGuard::InputEquals(outer))
            if inner.port_no == outer.port_no =>
        {
            inner.expected != outer.expected
        }
        (EffectGuard::InputEquals(_), EffectGuard::InputEquals(outer)) => {
            type_of_port(outer.port_no, export_inputs)
                .is_some_and(|reference| has_alternate(reference, schemas))
        }
        _ => {
            let _ = assertion;
            false
        }
    }
}

fn root_recovery_verdict(verdict: crate::proof::LeafVerdict) -> af_contract::ProofVerdict {
    match verdict {
        crate::proof::LeafVerdict::Proven => af_contract::ProofVerdict::Proven,
        crate::proof::LeafVerdict::Violation => af_contract::ProofVerdict::Violation,
        crate::proof::LeafVerdict::Unproven => af_contract::ProofVerdict::Unproven,
        crate::proof::LeafVerdict::RequiresEvidence => af_contract::ProofVerdict::RequiresEvidence,
    }
}

fn recovery_sources(
    root: &DerivedWorkflowContractSummary<'_, '_>,
) -> Result<Vec<crate::control::RecoverySourceSummary>, af_contract::ProofVerdict> {
    match root.root().recovery().value() {
        ControlDimensionValue::Complete(sources) => Ok(sources.clone()),
        ControlDimensionValue::PerChild {
            contributions,
            unresolved_sources,
        } if unresolved_sources.is_empty() => Ok(contributions
            .iter()
            .flat_map(|item| item.value().iter().cloned())
            .collect()),
        ControlDimensionValue::NotApplicable
            if root
                .root()
                .recovery()
                .proofs()
                .iter()
                .all(|proof| proof.verdict() == crate::proof::LeafVerdict::Proven) =>
        {
            Ok(Vec::new())
        }
        _ => Err(af_contract::ProofVerdict::Unproven),
    }
}

fn prove_flow(
    root: &DerivedWorkflowContractSummary<'_, '_>,
    exported: &ComponentContract,
    assertion: &RootExportAssertion,
    context: &StaticContractProofContext<'_>,
    semantic_ir_digest: &DigestRefValue,
) -> RootExportDimensionProof {
    let mut verdict = af_contract::ProofVerdict::Proven;
    let mut reasons = Vec::new();
    let summary = match root.root().information_flow().value() {
        ControlDimensionValue::Complete(summary) => Some(summary),
        _ => None,
    };
    let Some(summary) = summary else {
        return dimension_report(
            RootExportDimension::InformationFlow,
            DimensionProof::unproven(
                "information-flow",
                RULE_FLOW,
                "root Flow summary is incomplete; unknown external influence is not erased",
            ),
            Vec::new(),
            assertion.flow_mappings.len(),
        );
    };
    if !summary.unknown_external_children().is_empty()
        || !summary.unresolved_children().is_empty()
        || summary.control_decision_influence_unknown()
        || summary.known_may_influence().is_some_and(|influence| {
            influence.influence_unknown
                || influence
                    .sources
                    .iter()
                    .any(|source| source.category == af_flow::SourceCategory::UnknownExternal)
        })
    {
        verdict = af_contract::ProofVerdict::Unproven;
        reasons.push(
            "root Flow retains unknown external, unresolved, or control-decision influence"
                .to_owned(),
        );
    }
    let mut expected_mapping_count = 0usize;
    let mut source_facts = Vec::new();
    for source in summary.sources() {
        let Some(source_contract_digest) = target_contract_digest(source.target()) else {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons.push("Flow source has no exact source contract digest".to_owned());
            continue;
        };
        source_facts.push(root_source_reference(
            semantic_ir_digest,
            source.source_node(),
            &source_contract_digest,
            None,
        ));
        let source_mappings = source.source_mappings();
        expected_mapping_count += source_mappings.len();
        let mut source_map = BTreeMap::<FlowSourceRef, FlowSourceRef>::new();
        let mut sink_map = BTreeMap::<(FlowKind, FlowSink), FlowSink>::new();
        for (ordinal, actual) in source_mappings.iter().enumerate() {
            let Some(asserted) = assertion.flow_mappings.iter().find(|item| {
                item.source_node == source.source_node()
                    && item.source_contract_digest == source_contract_digest
                    && item.flow_mapping_ordinal == ordinal as u64
            }) else {
                verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
                reasons.push(
                    "a source Flow mapping lacks a typed root/export coordinate row".to_owned(),
                );
                continue;
            };
            if asserted.source != actual.source
                || asserted.sink != actual.sink
                || asserted.source_binding != actual.source_binding
                || asserted.sink_binding != actual.sink_binding
            {
                verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
                reasons.push(
                    "asserted Flow source/sink identity differs from the checked leaf mapping"
                        .to_owned(),
                );
                continue;
            }
            let export_source = if actual.source.is_some() {
                let Some(value) = translate_flow_source(
                    actual.source.as_ref(),
                    actual.source_binding.as_ref(),
                    source.source_node(),
                    &source_contract_digest,
                    assertion,
                ) else {
                    verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
                    reasons.push("Flow source cannot be translated through an exact input/resource coordinate".to_owned());
                    continue;
                };
                Some(value)
            } else {
                None
            };
            let export_sink = if actual.sink.is_some() {
                let Some(value) = translate_flow_sink(
                    actual.sink.as_ref(),
                    actual.sink_binding.as_ref(),
                    source.source_node(),
                    &source_contract_digest,
                    assertion,
                    root,
                ) else {
                    verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
                    reasons.push(
                        "Flow sink cannot be translated through an exact output/effect coordinate"
                            .to_owned(),
                    );
                    continue;
                };
                Some(value)
            } else {
                None
            };
            if asserted.export_source != export_source || asserted.export_sink != export_sink {
                verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
                reasons.push(
                    "asserted Flow target coordinate differs from the derived typed mapping"
                        .to_owned(),
                );
            }
            if let Some(source_ref) = &actual.source {
                let source_conflict = source_map
                    .get(source_ref)
                    .is_some_and(|previous| export_source.as_ref() != Some(previous));
                if source_conflict {
                    verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
                    reasons
                        .push("Flow coordinate has multiple incompatible translations".to_owned());
                }
                source_map.insert(
                    source_ref.clone(),
                    export_source.clone().expect("mapped Flow source"),
                );
            }
            if let Some(sink) = &actual.sink {
                let sink_key = (sink.kind, sink.sink.clone());
                let sink_conflict = match (sink_map.get(&sink_key), export_sink.as_ref()) {
                    (Some(previous), Some(mapped)) => previous != &mapped.sink,
                    (_, None) => true,
                    (None, Some(_)) => false,
                };
                if sink_conflict {
                    verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
                    reasons.push(
                        "Flow sink coordinate has multiple incompatible translations".to_owned(),
                    );
                }
                sink_map.insert(sink_key, export_sink.expect("mapped Flow sink").sink);
            }
        }
        let mut mapped_contract = source.contract().clone();
        for flow_source in &mut mapped_contract.summary.sources {
            if let Some(mapped) = source_map.get(flow_source) {
                *flow_source = mapped.clone();
            } else {
                verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
                reasons.push("declared Flow summary source lacks an exact map row".to_owned());
            }
        }
        for constraint in &mut mapped_contract.flow_constraints {
            if let Some(mapped) = sink_map.get(&(constraint.kind, constraint.sink.clone())) {
                constraint.sink = mapped.clone();
            } else {
                verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
                reasons.push("declared Flow sink lacks an exact map row".to_owned());
            }
        }
        match af_flow::compatibility::flow_not_relaxed(
            &mapped_contract,
            &exported.information_flow,
            context.flow,
        ) {
            Ok(af_flow::compatibility::FlowCompatibilityProof::Proven) => {}
            Ok(af_flow::compatibility::FlowCompatibilityProof::Unproven(_)) => {
                verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
                reasons.push(
                    "existing information-flow relation cannot prove non-relaxation".to_owned(),
                );
            }
            Err(error) if error.kind() == af_flow::error::kind::FLOW_CONTRACT_RELAXED => {
                verdict = merge_verdict(verdict, af_contract::ProofVerdict::Violation);
                reasons.push("mapped root Flow relaxes a complete exported restriction".to_owned());
            }
            Err(_) => {
                verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
                reasons
                    .push("flow catalog or mapped label/sink evidence is unavailable".to_owned());
            }
        }
        for partition in source.effect_flow_partitions() {
            let mapped = assertion.flow_mappings.iter().find(|item| {
                item.source_node == source.source_node()
                    && item.source_contract_digest == source_contract_digest
                    && item.source_effect_partition.as_ref() == Some(partition)
            });
            let Some(mapped) = mapped else {
                verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
                reasons.push(
                    "EffectFlowPartition has no exact source-preserving root/export map".to_owned(),
                );
                continue;
            };
            let Some(export_partition) = mapped.export_effect_partition.as_ref() else {
                verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
                reasons.push("mapped EffectFlowPartition is absent".to_owned());
                continue;
            };
            if !exported.effect_flow_partitions.contains(export_partition) {
                verdict = merge_verdict(verdict, af_contract::ProofVerdict::Violation);
                reasons.push("mapped EffectFlowPartition is not declared by the export".to_owned());
            }
            if partition.operation != export_partition.operation
                || partition.kind != export_partition.kind
            {
                verdict = merge_verdict(verdict, af_contract::ProofVerdict::Violation);
                reasons.push(
                    "EffectFlowPartition operation or FlowKind changes across the exact map"
                        .to_owned(),
                );
            }
            for atom in &partition.allowed_effects {
                let mapped_atom = assertion.effect_mappings.iter().find(|effect| {
                    effect.source.source_node == source.source_node()
                        && effect.source.source_contract_digest == source_contract_digest
                        && effect.source.atom == *atom
                });
                let Some(mapped_atom) = mapped_atom else {
                    verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
                    reasons.push("EffectFlowPartition membership cannot be mapped by full EffectAtom identity".to_owned());
                    continue;
                };
                if !export_partition
                    .allowed_effects
                    .contains(&mapped_atom.export_atom)
                {
                    verdict = merge_verdict(verdict, af_contract::ProofVerdict::Violation);
                    reasons.push("root EffectFlowPartition allows a mapped EffectAtom absent from the complete export partition".to_owned());
                }
            }
        }
    }
    let root_actions = if let ControlDimensionValue::Complete(actions) =
        root.root_recovery().actions().value()
    {
        actions.as_slice()
    } else {
        verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
        reasons.push("root K action set is incomplete for the action-to-Flow boundary".to_owned());
        &[]
    };
    for action in root_actions {
        let source = summary.sources().iter().find(|source| {
            source.source_node() == action.source.source_node
                && target_contract_digest(source.target()).as_ref()
                    == Some(&action.source.source_contract_digest)
        });
        let covered = source.is_some_and(|source| {
            source.effect_flow_partitions().iter().any(|partition| {
                partition.operation == action.action_effect_occurrence.declared.atom.operation
                    && partition
                        .allowed_effects
                        .contains(&action.action_effect_occurrence.declared.atom)
            })
        });
        if !covered {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons.push("Recovery action has no action-specific Flow field and no exact source EffectFlowPartition proves its action atom".to_owned());
        }
    }
    if assertion.flow_mappings.len() != expected_mapping_count {
        verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
        reasons.push("Flow mapping set has missing or extra source rows".to_owned());
    }
    if summary.known_may_influence().is_none() {
        verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
        reasons.push("root may-influence summary is not complete".to_owned());
    }
    dimension_report(
        RootExportDimension::InformationFlow,
        proof_for(
            "information-flow",
            RULE_FLOW,
            verdict,
            join_reasons(
                &reasons,
                "all mapped source Flow contracts and EffectFlowPartitions preserve the exported restrictions",
            ),
        ),
        source_facts,
        assertion.flow_mappings.len(),
    )
}

fn translate_flow_source(
    source: Option<&FlowSourceRef>,
    binding: Option<&FlowSourceBinding>,
    computation: ComputationId,
    source_contract_digest: &DigestRefValue,
    assertion: &RootExportAssertion,
) -> Option<FlowSourceRef> {
    let source = source?;
    let category = match (&source.category, binding?) {
        (
            af_flow::SourceCategory::WorkflowInput(_),
            FlowSourceBinding::InputExpression(expression),
        ) => {
            let input = assertion.input_mappings.iter().find(|item| {
                item.use_sites.iter().any(|site| {
                    site.computation == computation
                        && site.source_contract_digest == *source_contract_digest
                        && site.input_expression == *expression
                })
            })?;
            af_flow::SourceCategory::WorkflowInput(input.export_input_port.port_no)
        }
        (
            af_flow::SourceCategory::ResourceRequirement(_),
            FlowSourceBinding::ResourceSlot(slot),
        ) => {
            let resource = assertion
                .resource_mappings
                .iter()
                .find(|item| item.root_resource_slot == *slot)?;
            af_flow::SourceCategory::ResourceRequirement(resource.export_resource_port.port_no)
        }
        (af_flow::SourceCategory::UnknownExternal, FlowSourceBinding::UnknownExternal) => {
            af_flow::SourceCategory::UnknownExternal
        }
        _ => return None,
    };
    Some(FlowSourceRef { category })
}

fn translate_flow_sink(
    sink: Option<&FlowConstraint>,
    binding: Option<&FlowSinkBinding>,
    computation: ComputationId,
    source_contract_digest: &DigestRefValue,
    assertion: &RootExportAssertion,
    root: &DerivedWorkflowContractSummary<'_, '_>,
) -> Option<FlowConstraint> {
    let sink = sink?;
    let target = match (&sink.sink, binding?) {
        (FlowSink::WorkflowOutput(_), FlowSinkBinding::OutputValue(slot)) => {
            let outputs = assertion
                .output_mappings
                .iter()
                .filter(|item| item.producer_evidence.dependency_slots.contains(slot))
                .collect::<Vec<_>>();
            if outputs.len() != 1 {
                return None;
            }
            FlowSink::WorkflowOutput(outputs[0].export_output_port.port_no)
        }
        (FlowSink::ExternalEffect(operation), FlowSinkBinding::EffectOccurrences(indices)) => {
            let mut leaves = Vec::new();
            collect_leaf_summaries(root.root(), &mut leaves);
            let leaf = leaves
                .iter()
                .find(|leaf| leaf.computation() == computation)?;
            let mut mapped_operations = Vec::new();
            for index in indices {
                let occurrence = leaf.effect_occurrences().get(*index)?;
                if occurrence.source_contract_digest != *source_contract_digest {
                    return None;
                }
                let mapped = assertion
                    .effect_mappings
                    .iter()
                    .find(|item| item.source == effect_source_identity(occurrence))?;
                mapped_operations.push(mapped.export_atom.operation.clone());
            }
            if mapped_operations.is_empty()
                || mapped_operations
                    .iter()
                    .any(|item| item != &mapped_operations[0])
            {
                return None;
            }
            if &mapped_operations[0] != operation {
                return None;
            }
            FlowSink::ExternalEffect(mapped_operations[0].clone())
        }
        _ => return None,
    };
    Some(FlowConstraint {
        kind: sink.kind,
        sink: target,
    })
}

fn prove_usage(
    root: &DerivedWorkflowContractSummary<'_, '_>,
    exported: &ComponentContract,
    context: &StaticContractProofContext<'_>,
    semantic_ir_digest: &DigestRefValue,
) -> RootExportDimensionProof {
    let mut verdict = af_contract::ProofVerdict::Proven;
    let mut reasons = Vec::new();
    let root_envelope = match root.root().usage().value() {
        ControlDimensionValue::Complete(envelope) => Some(envelope),
        _ => None,
    };
    let export_envelope = exported.usage.static_envelope(context.usage_meters).ok();
    let (Some(root_envelope), Some(export_envelope)) = (root_envelope, export_envelope) else {
        return dimension_report(
            RootExportDimension::Usage,
            DimensionProof::unproven(
                "usage",
                RULE_USAGE,
                "root or export Usage envelope is incomplete or cannot be resolved against the exact meter universe",
            ),
            vec![RootExportSourceReference {
                semantic_ir_digest: semantic_ir_digest.clone(),
                computation: None,
                contract_digest: None,
                source_ordinal: None,
            }],
            0,
        );
    };
    for phase in [UsagePhase::Forward, UsagePhase::Recovery] {
        verdict = merge_verdict(
            verdict,
            usage_phase_within(
                root_envelope,
                &export_envelope,
                phase,
                context,
                &mut reasons,
            ),
        );
    }

    let recovery = root.root_recovery();
    let active_actions = match recovery.actions().value() {
        ControlDimensionValue::Complete(actions) => Some(!actions.is_empty()),
        _ => None,
    };
    match (active_actions, recovery.recovery_usage().value()) {
        (Some(false), ControlDimensionValue::Complete(envelope)) if envelope.is_empty() => {}
        (Some(true), ControlDimensionValue::Complete(k_usage)) => {
            let root_recovery_ceiling = phase_projection(root_envelope, UsagePhase::Recovery);
            verdict = merge_verdict(
                verdict,
                envelope_within(
                    k_usage,
                    &root_recovery_ceiling,
                    UsagePhase::Recovery,
                    &mut reasons,
                ),
            );
            let export_recovery_ceiling = phase_projection(&export_envelope, UsagePhase::Recovery);
            verdict = merge_verdict(
                verdict,
                envelope_within(
                    k_usage,
                    &export_recovery_ceiling,
                    UsagePhase::Recovery,
                    &mut reasons,
                ),
            );
        }
        _ => {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons.push(
                "active root K Recovery Usage or its complete source closure is unavailable"
                    .to_owned(),
            );
        }
    }
    dimension_report(
        RootExportDimension::Usage,
        proof_for(
            "usage",
            RULE_USAGE,
            verdict,
            join_reasons(
                &reasons,
                "Forward and Recovery Usage fit exact exported coordinates; K-required Recovery Usage is checked separately without addition",
            ),
        ),
        vec![RootExportSourceReference {
            semantic_ir_digest: semantic_ir_digest.clone(),
            computation: Some(root.root().control_node()),
            contract_digest: None,
            source_ordinal: None,
        }],
        root_envelope.len() + export_envelope.len(),
    )
}

fn phase_projection(envelope: &StaticUsageEnvelope, phase: UsagePhase) -> StaticUsageEnvelope {
    let mut result = StaticUsageEnvelope::empty();
    for (coordinate, bound) in envelope
        .iter()
        .filter(|(coordinate, _)| coordinate.phase == phase)
    {
        let _ = result.insert(coordinate.clone(), *bound);
    }
    if envelope.is_globally_unknown() {
        StaticUsageEnvelope::unknown()
    } else {
        result
    }
}

fn usage_phase_within(
    observed: &StaticUsageEnvelope,
    declared: &StaticUsageEnvelope,
    phase: UsagePhase,
    context: &StaticContractProofContext<'_>,
    reasons: &mut Vec<String>,
) -> af_contract::ProofVerdict {
    let observed = phase_projection(observed, phase);
    let declared = phase_projection(declared, phase);
    for envelope in [&observed, &declared] {
        if envelope.is_globally_unknown() {
            reasons.push(format!("{phase:?} usage is globally unknown"));
            return af_contract::ProofVerdict::Unproven;
        }
    }
    if observed.is_empty() || declared.is_empty() {
        reasons.push(format!(
            "{phase:?} usage has no explicit meter coordinate; absence is not finite zero"
        ));
        return af_contract::ProofVerdict::Unproven;
    }
    let mut verdict = af_contract::ProofVerdict::Proven;
    for (coordinate, bound) in observed.iter() {
        if context.usage_meters.resolve(&coordinate.meter).is_err() {
            reasons.push(format!(
                "{phase:?} usage meter identity or unit is unresolved"
            ));
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            continue;
        }
        match declared.get(coordinate) {
            StaticUsageBound::Unknown => {
                reasons.push(format!("export omits the exact {phase:?} meter coordinate"));
                verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            }
            ceiling => {
                verdict = merge_verdict(
                    verdict,
                    compare_usage_bound(*bound, ceiling, phase, reasons),
                );
            }
        }
    }
    if declared.iter().any(|(coordinate, _)| {
        coordinate.phase == phase && observed.get(coordinate) == StaticUsageBound::Unknown
    }) {
        reasons.push(format!(
            "root omits an exact exported {phase:?} meter coordinate; omission is not zero"
        ));
        verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
    }
    verdict
}

fn envelope_within(
    observed: &StaticUsageEnvelope,
    declared: &StaticUsageEnvelope,
    phase: UsagePhase,
    reasons: &mut Vec<String>,
) -> af_contract::ProofVerdict {
    if observed.is_globally_unknown() || declared.is_globally_unknown() {
        reasons.push("K-required Recovery Usage or its ceiling is globally unknown".to_owned());
        return af_contract::ProofVerdict::Unproven;
    }
    let observed = phase_projection(observed, phase);
    let declared = phase_projection(declared, phase);
    if observed.is_empty() {
        reasons.push("active K sources have no exact Recovery Usage meter profile".to_owned());
        return af_contract::ProofVerdict::Unproven;
    }
    let mut verdict = af_contract::ProofVerdict::Proven;
    for (coordinate, bound) in observed.iter() {
        let ceiling = declared.get(coordinate);
        if ceiling == StaticUsageBound::Unknown {
            reasons.push(
                "root or export recovery ceiling omits a K-required meter; missing is not zero"
                    .to_owned(),
            );
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
        } else {
            verdict = merge_verdict(
                verdict,
                compare_usage_bound(*bound, ceiling, phase, reasons),
            );
        }
    }
    verdict
}

fn compare_usage_bound(
    observed: StaticUsageBound,
    ceiling: StaticUsageBound,
    phase: UsagePhase,
    reasons: &mut Vec<String>,
) -> af_contract::ProofVerdict {
    match (observed, ceiling) {
        (StaticUsageBound::Unknown, _) | (_, StaticUsageBound::Unknown) => {
            reasons.push(format!("{phase:?} usage bound is unknown"));
            af_contract::ProofVerdict::Unproven
        }
        (StaticUsageBound::KnownFinite(value), StaticUsageBound::KnownFinite(limit))
            if value <= limit =>
        {
            af_contract::ProofVerdict::Proven
        }
        (
            StaticUsageBound::KnownUnbounded | StaticUsageBound::KnownFinite(_),
            StaticUsageBound::KnownUnbounded,
        ) => af_contract::ProofVerdict::Proven,
        (StaticUsageBound::KnownUnbounded, StaticUsageBound::KnownFinite(_)) => {
            reasons.push(format!(
                "unbounded {phase:?} usage exceeds a finite export ceiling"
            ));
            af_contract::ProofVerdict::Violation
        }
        (StaticUsageBound::KnownFinite(value), StaticUsageBound::KnownFinite(limit)) => {
            reasons.push(format!(
                "{phase:?} usage upper {value} exceeds exported upper {limit}"
            ));
            af_contract::ProofVerdict::Violation
        }
    }
}

fn prove_recovery(
    root: &DerivedWorkflowContractSummary<'_, '_>,
    exported: &ComponentContract,
    assertion: &RootExportAssertion,
    context: &StaticContractProofContext<'_>,
    semantic_ir_digest: &DigestRefValue,
) -> RootExportDimensionProof {
    let mut verdict = af_contract::ProofVerdict::Proven;
    let mut reasons = Vec::new();
    let recovery = root.root_recovery();
    let actions = match recovery.actions().value() {
        ControlDimensionValue::Complete(actions) => Some(actions.as_slice()),
        _ => None,
    };
    let Some(actions) = actions else {
        return dimension_report(
            RootExportDimension::Recovery,
            DimensionProof::unproven(
                "recovery",
                RULE_RECOVERY,
                "RootRecoverySummary action multiset is incomplete",
            ),
            Vec::new(),
            assertion.recovery_mappings.len(),
        );
    };
    let closure = recovery.source_closure();
    for (label, fact) in [
        ("source nodes", closure.source_nodes_accounted),
        ("action bindings", closure.action_bindings_accounted),
        ("usage profiles", closure.usage_profiles_accounted),
        (
            "multiplicity factors",
            closure.multiplicity_factors_accounted,
        ),
        ("root action totals", closure.root_action_totals_sound),
    ] {
        if fact != crate::proof::LeafVerdict::Proven {
            verdict = merge_verdict(verdict, root_recovery_verdict(fact));
            reasons.push(format!(
                "RootRecoverySourceClosureReport does not prove {label} accounted"
            ));
        }
    }
    if assertion.recovery_mappings.len() != actions.len() {
        verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
        reasons.push(
            "Recovery mapping does not preserve the complete root source/action/profile multiset"
                .to_owned(),
        );
    }
    let mut recovery_map_complete = assertion.recovery_mappings.len() == actions.len();
    let mut mapped_export_actions: Vec<af_contract::RecoveryActionIdentity> = Vec::new();
    let mut mapped_profiles: Vec<RecoveryUsageBinding> = Vec::new();
    let mut source_facts = Vec::new();
    for action in actions {
        let source = &action.source;
        source_facts.push(root_source_reference(
            semantic_ir_digest,
            source.source_node,
            &source.source_contract_digest,
            Some(source.forward_entry_ordinal as u64),
        ));
        let Some(mapping) = assertion
            .recovery_mappings
            .iter()
            .find(|item| item.source == *source)
        else {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            recovery_map_complete = false;
            reasons
                .push("root recovery action source has no exact typed export mapping".to_owned());
            continue;
        };
        if mapping.exact_usage_binding != action.recovery_usage_binding {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            recovery_map_complete = false;
            reasons.push(
                "RecoveryUsageBinding does not match the checked root action profile".to_owned(),
            );
        }
        let forward_mapping = assertion
            .effect_mappings
            .iter()
            .find(|item| item.source == effect_source_identity(&action.forward_occurrence));
        let action_effect_mapping = assertion
            .effect_mappings
            .iter()
            .find(|item| item.source == effect_source_identity(&action.action_effect_occurrence));
        let (Some(forward_mapping), Some(action_effect_mapping)) =
            (forward_mapping, action_effect_mapping)
        else {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            recovery_map_complete = false;
            reasons.push("Recovery forward/action atoms are not both mapped through exact epsilon coordinates".to_owned());
            continue;
        };
        let exact_export_action = af_contract::RecoveryActionIdentity {
            forward_effect: forward_mapping.export_atom.clone(),
            action_kind: mapping.export_action_identity.action_kind,
            action_effect: action_effect_mapping.export_atom.clone(),
        };
        if mapping.export_action_identity != exact_export_action
            || mapping.export_action_identity.action_kind != source.action.action_kind
            || mapping.export_action_identity.forward_effect != forward_mapping.export_atom
            || mapping.export_action_identity.action_effect != action_effect_mapping.export_atom
        {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            recovery_map_complete = false;
            reasons.push("mapped RecoveryActionIdentity changes the exact forward/action atom or action kind".to_owned());
            continue;
        }
        if !exported
            .recovery_action_bindings
            .iter()
            .any(|binding| binding.identity == mapping.export_action_identity)
        {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Violation);
            reasons
                .push("required root RecoveryActionBinding is absent from exported K".to_owned());
        }
        let mut mapped_profile = action.recovery_usage_binding.clone();
        mapped_profile.action = mapping.export_action_identity.clone();
        mapped_profiles.push(mapped_profile);
        mapped_export_actions.push(mapping.export_action_identity.clone());

        if let Some(effect) = exact_export_effect(
            &exported.effects,
            &mapping.export_action_identity.action_effect,
        ) {
            let possible_guard = translate_guard(
                &action.forward_occurrence.declared.guard,
                source.source_node,
                &source.source_contract_digest,
                assertion,
            );
            let Some(possible_guard) = possible_guard else {
                verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
                reasons.push(
                    "root K action possibility guard lacks an exact exported input translation"
                        .to_owned(),
                );
                continue;
            };
            match action.root_bound.upper {
                crate::root_recovery::RecoveryActionUpper::Known(upper) => {
                    let required = OccurrenceBounds { lower: 0, upper };
                    if !effect.bounds.covers(&required) {
                        verdict = merge_verdict(verdict, af_contract::ProofVerdict::Violation);
                        reasons.push("root K action upper exceeds the mapped exported epsilon action ceiling".to_owned());
                    }
                    if upper != OccurrenceCeiling::Finite(0) {
                        let direct = af_effect::guard_contained(&possible_guard, &effect.guard);
                        let has_path_condition = action.control_path.iter().any(|step| {
                            matches!(
                                step.kind,
                                crate::root_recovery::ControlPathKind::SelectCase { .. }
                                    | crate::root_recovery::ControlPathKind::HandleClause { .. }
                            )
                        });
                        let guard = if direct {
                            af_contract::ProofVerdict::Proven
                        } else if has_path_condition {
                            af_contract::ProofVerdict::Unproven
                        } else {
                            guard_relation_verdict(
                                &possible_guard,
                                &effect.guard,
                                &exported.inputs,
                                assertion,
                                context.schema_universe,
                            )
                        };
                        if guard != af_contract::ProofVerdict::Proven {
                            verdict = merge_verdict(verdict, guard);
                            reasons.push("root K action possibility is not contained by the mapped exported epsilon guard".to_owned());
                        }
                    }
                }
                crate::root_recovery::RecoveryActionUpper::Overflow
                | crate::root_recovery::RecoveryActionUpper::InvalidMultiplicity => {
                    verdict = merge_verdict(verdict, af_contract::ProofVerdict::Violation);
                    reasons.push(
                        "root K action bound overflows or has invalid multiplicity".to_owned(),
                    );
                }
                crate::root_recovery::RecoveryActionUpper::Unknown => {
                    verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
                    reasons.push("root K action upper is unknown".to_owned());
                }
            }
        } else if matches!(exported.effects.row, EffectRow::Closed(_)) {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Violation);
            reasons.push(
                "mapped root K action atom is absent from the complete closed exported epsilon row"
                    .to_owned(),
            );
        } else {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons.push("exported epsilon row is open/unknown and does not provide an exact K action ceiling".to_owned());
        }
    }

    if let Ok(root_sources) = recovery_sources(root) {
        for source in &root_sources {
            match af_recovery::compatibility::recovery_not_weakened(
                source.contract(),
                &exported.recovery,
                context.recovery,
            ) {
                Ok(af_recovery::compatibility::RecoveryCompatibilityProof::Proven) => {}
                Ok(af_recovery::compatibility::RecoveryCompatibilityProof::Weakened) => {
                    verdict = merge_verdict(verdict, af_contract::ProofVerdict::Violation);
                    reasons.push("a root recovery source is proven weaker than the exact exported RecoveryContract".to_owned());
                }
                Ok(af_recovery::compatibility::RecoveryCompatibilityProof::Unproven) | Err(_) => {
                    verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
                    reasons.push(
                        "changed root/export RecoveryContract has no frozen monotonic proof"
                            .to_owned(),
                    );
                }
            }
        }
    } else {
        verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
        reasons.push("root recovery contract source set is incomplete".to_owned());
    }

    let mut unique_mapped_actions = Vec::new();
    for action in mapped_export_actions {
        if !unique_mapped_actions.contains(&action) {
            unique_mapped_actions.push(action);
        }
    }
    for binding in &exported.recovery_action_bindings {
        if !unique_mapped_actions.contains(&binding.identity) {
            verdict = merge_verdict(
                verdict,
                if !recovery_map_complete || actions.is_empty() {
                    af_contract::ProofVerdict::Unproven
                } else {
                    af_contract::ProofVerdict::Violation
                },
            );
            reasons
                .push("exported K action commitment has no mapped root recovery source".to_owned());
        }
    }

    for profile in &mapped_profiles {
        if !exported
            .recovery_usage_bindings
            .iter()
            .any(|declared| declared == profile)
        {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons.push(
                "mapped recovery usage profile is not exactly declared by exported K".to_owned(),
            );
        }
    }
    for profile in &exported.recovery_usage_bindings {
        if !mapped_profiles.contains(profile) {
            let action_has_root_source = unique_mapped_actions.contains(&profile.action);
            verdict = merge_verdict(
                verdict,
                if !recovery_map_complete || actions.is_empty() || action_has_root_source {
                    af_contract::ProofVerdict::Unproven
                } else {
                    af_contract::ProofVerdict::Violation
                },
            );
            reasons.push(
                "exported recovery usage profile has no exact root source/profile witness"
                    .to_owned(),
            );
        }
    }

    dimension_report(
        RootExportDimension::Recovery,
        proof_for(
            "recovery",
            RULE_RECOVERY,
            verdict,
            join_reasons(
                &reasons,
                "root K source/action/profile multisets and exact RecoveryContract semantics are preserved",
            ),
        ),
        source_facts,
        assertion.recovery_mappings.len(),
    )
}

fn guard_relation_verdict(
    possible: &EffectGuard,
    ceiling: &EffectGuard,
    exported_inputs: &[af_contract::ComponentInputPort],
    assertion: &RootExportAssertion,
    schemas: &af_semantic::TypeSchemaUniverse,
) -> af_contract::ProofVerdict {
    if af_effect::guard_contained(possible, ceiling) {
        af_contract::ProofVerdict::Proven
    } else if guard_counterexample(possible, ceiling, exported_inputs, assertion, schemas) {
        af_contract::ProofVerdict::Violation
    } else {
        af_contract::ProofVerdict::Unproven
    }
}

fn execution_verdict(value: af_execution::CompatibilityVerdict) -> af_contract::ProofVerdict {
    match value {
        af_execution::CompatibilityVerdict::Proven => af_contract::ProofVerdict::Proven,
        af_execution::CompatibilityVerdict::Violation => af_contract::ProofVerdict::Violation,
        af_execution::CompatibilityVerdict::Unproven => af_contract::ProofVerdict::Unproven,
    }
}

fn prove_execution(
    root: &DerivedWorkflowContractSummary<'_, '_>,
    exported: &ComponentContract,
    assertion: &RootExportAssertion,
    context: &StaticContractProofContext<'_>,
    semantic_ir_digest: &DigestRefValue,
) -> (RootExportDimensionProof, Vec<RootExecutionChildProof>) {
    let mut sources: Vec<&ExecutionSourceSummary> = Vec::new();
    let mut verdict = af_contract::ProofVerdict::Proven;
    let mut reasons = Vec::new();
    match root.root().execution().value() {
        ControlDimensionValue::Complete(items) => sources.extend(items.iter()),
        ControlDimensionValue::PerChild {
            contributions,
            unresolved_sources,
        } => {
            for item in contributions {
                sources.extend(item.value().iter());
            }
            if !unresolved_sources.is_empty() {
                verdict = af_contract::ProofVerdict::Unproven;
                reasons.push("one or more child X sources are unresolved".to_owned());
            }
        }
        ControlDimensionValue::Partial { .. } | ControlDimensionValue::Unknown { .. } => {
            verdict = af_contract::ProofVerdict::Unproven;
            reasons.push("root per-child X source set is partial or unknown".to_owned());
        }
        ControlDimensionValue::NotApplicable => {}
    }
    if assertion.execution_mappings.len() != sources.len() {
        verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
        reasons.push("execution coordinate maps do not cover every child source".to_owned());
    }
    let mut children = Vec::new();
    let mut source_facts = Vec::new();
    for source in sources {
        let Some(contract_digest) = target_contract_digest(source.target()) else {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons.push("child X source has no exact exported ComponentContract pin".to_owned());
            continue;
        };
        source_facts.push(root_source_reference(
            semantic_ir_digest,
            source.source_node(),
            &contract_digest,
            None,
        ));
        let Some(mapping) = assertion.execution_mappings.iter().find(|item| {
            item.source_node == source.source_node()
                && item.source_contract_digest == contract_digest
        }) else {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons.push("child X source has no exact execution mapping".to_owned());
            continue;
        };
        if mapping.exact_execution_requirements != *source.requirements() {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons.push(
                "asserted child X requirements differ from the checked source summary".to_owned(),
            );
        }
        if let Ok(proof) = af_execution::compatibility::execution_compatibility(
            source.requirements(),
            &exported.execution,
            context.execution,
        ) {
            verdict = merge_verdict(verdict, execution_verdict(proof.feasibility));
            verdict = merge_verdict(verdict, execution_verdict(proof.security));
            children.push(RootExecutionChildProof {
                source_node: source.source_node(),
                source_contract_digest: contract_digest,
                feasibility: proof.feasibility,
                security: proof.security,
            });
        } else {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons
                .push("execution catalog/version/region/trust relation is unavailable".to_owned());
        }
    }
    match root.root_recovery().actions().value() {
        ControlDimensionValue::Complete(actions) if !actions.is_empty() => {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons.push("Recovery actions have no action-specific ExecutionRequirements in the frozen model".to_owned());
        }
        ControlDimensionValue::Complete(_) => {}
        _ => {
            verdict = merge_verdict(verdict, af_contract::ProofVerdict::Unproven);
            reasons
                .push("root K source set is incomplete for per-action X compatibility".to_owned());
        }
    }
    let proof = proof_for(
        "execution",
        RULE_EXECUTION,
        verdict,
        join_reasons(
            &reasons,
            "every exact child X source independently satisfies exported feasibility and security constraints",
        ),
    );
    (
        dimension_report(
            RootExportDimension::Execution,
            proof,
            source_facts,
            assertion.execution_mappings.len(),
        ),
        children,
    )
}
