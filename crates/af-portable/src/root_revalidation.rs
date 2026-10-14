//! Closed D10 projection from registered PortableRootExportAssertion values
//! to the existing, unproved RootExportAssertion input type.

use crate::error::{PortableProofError, ValidationStage};
use crate::reconstruction::value_to_json;
use af_computation::{
    AppliedMultiplicity, ComponentInputPortIdentity, ComponentOutputPortIdentity, ControlDimension,
    ControlDimensionProof, ControlDimensionValue, ControlFlowSummary, ControlOperator,
    ControlPathKind, ControlPathReachability, ControlPathStep, DerivedControlContractSummary,
    DerivedWorkflowContractSummary, EffectLowerBoundAdjustmentReason, ExecutionSourceSummary,
    FlowSinkBinding, FlowSourceBinding, LocalActionBound, ProofContextManifest,
    RecoveryActionUpper, RecoverySourceSummary, ResourcePortOccurrence,
    RootEffectCoordinateBinding, RootEffectOccurrenceIdentity, RootExecutionCoordinateBinding,
    RootExportAssertion, RootExportAssertionIdentity, RootFlowCoordinateBinding,
    RootInputPortBinding, RootInputUseSite, RootOutputPortBinding, RootOutputProducerEvidence,
    RootRecoveryAction, RootRecoveryCoordinateBinding, RootRecoveryOwner, RootResourcePortBinding,
    ScopeBoundarySummary, ScopedEffectOccurrence, ScopedEffectResidual,
    StaticComputationResolutionSnapshot, SubflowProofDependency, SubflowTargetIdentity,
    current_compiler_semantics_identity,
};
use af_contract::{
    CheckedComponentContract, EffectFlowPartition, RecoveryActionIdentity, RecoveryUsageBinding,
    StaticContractProofContext,
};
use af_core::Value;
use af_effect::{
    EffectAtom, EffectEntry, EffectGuard, EffectIdempotencyQualifier, EffectOpenRow,
    EffectQualifier, EffectResidual, EffectRow, EffectSignature, OccurrenceBounds,
    OccurrenceCeiling, ResourceSelector,
};
use af_execution::{ExecutionRequirements, SandboxLevel};
use af_failure::FailureRow;
use af_flow::{
    FlowConstraint, FlowSink, FlowSourceRef, InformationFlowContract, SourceCategory,
    StaticInformationSummary,
};
use af_ir::{CheckedIrDocument, ComputationId, RunParameterId};
use af_resource::{ResourceRequirement, ResourceTypeRef};
use af_semantic::{
    DigestRefValue, ExpressionId, PrimitiveConstant, ResourceSlotId, SemanticPrimitiveType,
    StructuralType, TypeSchemaRef, ValueSlotId,
};
use af_usage::{StaticUsageBound, StaticUsageEnvelope};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value as Json;
use serde_json::json;
use sha2::Digest as _;
use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
mod conformance;
#[cfg(test)]
pub(crate) use conformance::project_full_d9_for_conformance;

const MAX_ASSERTION_MAPPINGS: usize = 2_048;
const MAX_STRUCTURAL_NODES: usize = 100_000;
const MAX_STRUCTURAL_EXPANSION_WORK: usize = 100_000;
const MAX_STRUCTURAL_EXPANSION_DEPTH: usize = 96;
const MAX_NESTED_ASSERTION_IDENTITY_WORK: usize = 50_000;
const MAX_ASSERTION_GRAPH_NODES: usize = 32;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct TypedContentReference {
    digest: DigestRefValue,
    type_schema: TypeSchemaRef,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct PortableContextResolverIds {
    context_id: u32,
    resolver_id: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct PortableRootInputMapping {
    export_input_port: ComponentInputPortIdentity,
    run_parameter_id: RunParameterId,
    root_value_slot: ValueSlotId,
    root_type: u32,
    export_type: TypeSchemaRef,
    use_sites: Vec<RootInputUseSite>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct PortableRootOutputMapping {
    export_output_port: ComponentOutputPortIdentity,
    workflow_output_name: String,
    workflow_output_expression: ExpressionId,
    root_result_type: u32,
    producer_evidence: RootOutputProducerEvidence,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct PortableEffectOccurrenceIdentity {
    caller_semantic_ir_digest: DigestRefValue,
    source_node: ComputationId,
    source_contract_digest: DigestRefValue,
    source_entry_ordinal: u64,
    atom: EffectAtom,
    guard: EffectGuard,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct PortableRootEffectMapping {
    source: PortableEffectOccurrenceIdentity,
    root_resource_slot: Option<ResourceSlotId>,
    export_atom: EffectAtom,
    export_guard: EffectGuard,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum PortableFlowSourceBinding {
    InputExpression(ExpressionId),
    ResourceSlot(ResourceSlotId),
    UnknownExternal,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum PortableFlowSinkBinding {
    OutputValue(ValueSlotId),
    EffectOccurrences(Vec<u64>),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct PortableRootFlowMapping {
    source_node: ComputationId,
    source_contract_digest: DigestRefValue,
    flow_mapping_ordinal: u64,
    source: Option<FlowSourceRef>,
    sink: Option<FlowConstraint>,
    source_binding: Option<PortableFlowSourceBinding>,
    sink_binding: Option<PortableFlowSinkBinding>,
    export_source: Option<FlowSourceRef>,
    export_sink: Option<FlowConstraint>,
    source_effect_partition: Option<EffectFlowPartition>,
    export_effect_partition: Option<EffectFlowPartition>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct PortableRecoveryActionSourceIdentity {
    caller_semantic_ir_digest: DigestRefValue,
    source_node: ComputationId,
    source_contract_digest: DigestRefValue,
    forward_entry_ordinal: u64,
    action: RecoveryActionIdentity,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct PortableRootRecoveryMapping {
    source: PortableRecoveryActionSourceIdentity,
    export_action_identity: RecoveryActionIdentity,
    exact_usage_binding: RecoveryUsageBinding,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct PortableSubflowProofDependency {
    call_site: ComputationId,
    target_semantic_ir_digest: DigestRefValue,
    target_exported_contract_digest: Option<DigestRefValue>,
    target_root_assertion_id: Option<u32>,
    transitive_targets: Vec<SubflowTargetIdentity>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct PortableRootExportAssertion {
    exact_semantic_ir_digest: DigestRefValue,
    exact_exported_component_contract_digest: DigestRefValue,
    compiler_semantics_id: u32,
    proof_context_manifest: PortableContextResolverIds,
    input_mappings: Vec<PortableRootInputMapping>,
    output_mappings: Vec<PortableRootOutputMapping>,
    resource_mappings: Vec<RootResourcePortBinding>,
    effect_mappings: Vec<PortableRootEffectMapping>,
    flow_mappings: Vec<PortableRootFlowMapping>,
    recovery_mappings: Vec<PortableRootRecoveryMapping>,
    execution_mappings: Vec<RootExecutionCoordinateBinding>,
    subflow_dependencies: Vec<PortableSubflowProofDependency>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct PortableAssertionEntry {
    id: u32,
    ir_asset: TypedContentReference,
    export_contract_asset: TypedContentReference,
    assertion: PortableRootExportAssertion,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct StructuralTypeNode {
    id: u32,
    body: StructuralTypeBody,
}

pub(crate) struct ProjectedAssertion {
    pub assertion: RootExportAssertion,
    pub expanded_identity_work: usize,
    pub comparison_work: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum StructuralTypeBody {
    Primitive(SemanticPrimitiveType),
    Schema(TypeSchemaRef),
    List(u32),
    MapString(u32),
    Option(u32),
}

/// Convert the complete registered assertion into a fresh unproved checker
/// input. Context and compiler identities are recaptured from the current D5,
/// D8 and D4-supported inputs, never copied from a verdict or cached report.
#[allow(clippy::too_many_arguments)] // These values intentionally form one exact D5-D8 proof scope.
pub(crate) fn project_assertion(
    root: &af_core::Value,
    assertion_id: u32,
    context_id: u32,
    resolver_id: u32,
    trusted_compiler_manifest: &af_core::Value,
    context: &StaticContractProofContext<'_>,
    caller: &CheckedIrDocument<'_>,
    exported: &CheckedComponentContract<'_, '_>,
    snapshot: &StaticComputationResolutionSnapshot<'_, '_, '_>,
    mut capture_child: impl FnMut(
        u32,
    )
        -> Result<(RootExportAssertionIdentity, usize), PortableProofError>,
) -> Result<ProjectedAssertion, PortableProofError> {
    let path = "$.root_export_assertions";
    let row = list_field(root, "root_export_assertions", "$")?
        .iter()
        .find(|entry| u32_field(entry, "id", path).ok() == Some(assertion_id))
        .ok_or_else(|| failure("MissingAssertion", path, "assertion ID does not resolve"))?;
    let entry: PortableAssertionEntry = from_value(row, path)?;
    if entry.id != assertion_id {
        return Err(failure(
            "AssertionIdMismatch",
            path,
            "assertion row ID changed during projection",
        ));
    }
    let caller_digest = caller.source().digest(context.modules).map_err(|error| {
        failure(
            "CallerDigestFailed",
            "$.root_export_assertions[].ir_asset",
            error.to_string(),
        )
    })?;
    let caller_digest = DigestRefValue {
        algorithm: caller_digest.alg().as_str().to_owned(),
        domain: caller_digest.domain().as_str().to_owned(),
        hex: caller_digest.hex().to_owned(),
    };
    if entry.ir_asset.digest != caller_digest
        || entry.assertion.exact_semantic_ir_digest != caller_digest
    {
        return Err(failure(
            "AssertionIrPinMismatch",
            "$.root_export_assertions[].ir_asset",
            "assertion IR asset, D8 caller and exact assertion pin must be identical",
        ));
    }
    if entry.ir_asset.type_schema.specification_id != "agentflow.semantic.ir@1"
        || entry.ir_asset.type_schema.type_name != "SemanticIrDocument"
        || entry.export_contract_asset.type_schema.specification_id
            != "agentflow.component.contract@1"
        || entry.export_contract_asset.type_schema.type_name != "ComponentContract"
    {
        return Err(failure(
            "AssertionAssetTypeMismatch",
            "$.root_export_assertions[]",
            "assertion references must use the registered SemanticIrDocument and ComponentContract schema identities",
        ));
    }
    let export_digest =
        af_contract::component_contract_digest(exported, context).map_err(|error| {
            failure(
                "ExportDigestFailed",
                "$.root_export_assertions[].export_contract_asset",
                error.to_string(),
            )
        })?;
    let export_digest = DigestRefValue {
        algorithm: export_digest.alg().as_str().to_owned(),
        domain: export_digest.domain().as_str().to_owned(),
        hex: export_digest.hex().to_owned(),
    };
    if entry.export_contract_asset.digest != export_digest
        || entry.assertion.exact_exported_component_contract_digest != export_digest
    {
        return Err(failure(
            "AssertionExportPinMismatch",
            "$.root_export_assertions[].export_contract_asset",
            "assertion export asset, fresh D6 contract and exact assertion pin must be identical",
        ));
    }
    if entry.assertion.proof_context_manifest.context_id != context_id
        || entry.assertion.proof_context_manifest.resolver_id != resolver_id
        || !snapshot.is_same_context(context)
        || !std::ptr::eq(exported.proof_context(), context)
    {
        return Err(failure(
            "AssertionContextIdentityMismatch",
            "$.root_export_assertions[].assertion.proof_context_manifest",
            "assertion context/resolver IDs and fresh checked objects must match this D5/D8 scope",
        ));
    }
    let compiler_manifest = find_compiler_manifest(root, entry.assertion.compiler_semantics_id)?;
    if compiler_manifest != trusted_compiler_manifest {
        return Err(failure(
            "CompilerSemanticsMismatch",
            "$.root_export_assertions[].assertion.compiler_semantics_id",
            "this assertion does not select the exact D4-supported compiler semantics content",
        ));
    }

    let mapping_count = assertion_mapping_count(&entry.assertion)?;
    if mapping_count > MAX_ASSERTION_MAPPINGS {
        return Err(resource_limit(
            "$.root_export_assertions[].assertion",
            "assertion mapping expansion limit exceeded",
        ));
    }
    let mapping_lengths = [
        entry.assertion.input_mappings.len(),
        entry.assertion.output_mappings.len(),
        entry.assertion.resource_mappings.len(),
        entry.assertion.effect_mappings.len(),
        entry.assertion.flow_mappings.len(),
        entry.assertion.recovery_mappings.len(),
        entry.assertion.execution_mappings.len(),
        entry.assertion.subflow_dependencies.len(),
    ];
    let comparison_work = mapping_lengths
        .iter()
        .try_fold(0usize, |sum, count| {
            sum.checked_add(count.checked_mul(*count)?)
                .and_then(|total| total.checked_add(*count))
        })
        .ok_or_else(|| resource_limit(path, "assertion mapping comparison work overflow"))?;
    let structural_types = structural_type_table(root)?;
    let mut structural_cache = BTreeMap::new();
    let mut structural_active = BTreeSet::new();
    let mut structural_work = 0usize;
    let input_mappings = entry
        .assertion
        .input_mappings
        .into_iter()
        .map(|mapping| {
            Ok(RootInputPortBinding {
                export_input_port: mapping.export_input_port,
                run_parameter_id: mapping.run_parameter_id,
                root_value_slot: mapping.root_value_slot,
                root_type: expand_structural_type(
                    mapping.root_type,
                    &structural_types,
                    &mut structural_cache,
                    &mut structural_active,
                    &mut structural_work,
                    0,
                )?,
                export_type: mapping.export_type,
                use_sites: mapping.use_sites,
            })
        })
        .collect::<Result<Vec<_>, PortableProofError>>()?;
    let output_mappings = entry
        .assertion
        .output_mappings
        .into_iter()
        .map(|mapping| {
            Ok(RootOutputPortBinding {
                export_output_port: mapping.export_output_port,
                workflow_output_name: mapping.workflow_output_name,
                workflow_output_expression: mapping.workflow_output_expression,
                root_result_type: expand_structural_type(
                    mapping.root_result_type,
                    &structural_types,
                    &mut structural_cache,
                    &mut structural_active,
                    &mut structural_work,
                    0,
                )?,
                producer_evidence: mapping.producer_evidence,
            })
        })
        .collect::<Result<Vec<_>, PortableProofError>>()?;

    let effect_mappings = entry
        .assertion
        .effect_mappings
        .into_iter()
        .map(|mapping| {
            let source_entry_ordinal = usize::try_from(mapping.source.source_entry_ordinal)
                .map_err(|_| {
                    resource_limit(
                        "$.root_export_assertions[].assertion.effect_mappings",
                        "source entry ordinal exceeds host index range",
                    )
                })?;
            Ok(RootEffectCoordinateBinding {
                source: RootEffectOccurrenceIdentity {
                    caller_semantic_ir_digest: mapping.source.caller_semantic_ir_digest,
                    source_node: mapping.source.source_node,
                    source_contract_digest: mapping.source.source_contract_digest,
                    source_entry_ordinal,
                    atom: mapping.source.atom,
                    guard: mapping.source.guard,
                },
                root_resource_slot: mapping.root_resource_slot,
                export_atom: mapping.export_atom,
                export_guard: mapping.export_guard,
            })
        })
        .collect::<Result<Vec<_>, PortableProofError>>()?;
    let flow_mappings = entry
        .assertion
        .flow_mappings
        .into_iter()
        .map(convert_flow_mapping)
        .collect::<Result<Vec<_>, PortableProofError>>()?;
    let recovery_mappings = entry
        .assertion
        .recovery_mappings
        .into_iter()
        .map(|mapping| {
            let forward_entry_ordinal = usize::try_from(mapping.source.forward_entry_ordinal)
                .map_err(|_| {
                    resource_limit(
                        "$.root_export_assertions[].assertion.recovery_mappings",
                        "recovery ordinal exceeds host index range",
                    )
                })?;
            Ok(RootRecoveryCoordinateBinding {
                source: af_computation::RecoveryActionSourceIdentity {
                    caller_semantic_ir_digest: mapping.source.caller_semantic_ir_digest,
                    source_node: mapping.source.source_node,
                    source_contract_digest: mapping.source.source_contract_digest,
                    forward_entry_ordinal,
                    action: mapping.source.action,
                },
                export_action_identity: mapping.export_action_identity,
                exact_usage_binding: mapping.exact_usage_binding,
            })
        })
        .collect::<Result<Vec<_>, PortableProofError>>()?;
    let mut expanded_identity_work = 1usize;
    let subflow_dependencies = entry
        .assertion
        .subflow_dependencies
        .into_iter()
        .map(|dependency| {
            let target_root_assertion_identity = if let Some(target_id) = dependency.target_root_assertion_id {
                let (identity, child_work) = capture_child(target_id)?;
                expanded_identity_work = expanded_identity_work.checked_add(child_work).ok_or_else(|| {
                    resource_limit("$.root_export_assertions", "nested assertion identity work overflow")
                })?;
                if expanded_identity_work > MAX_NESTED_ASSERTION_IDENTITY_WORK {
                    return Err(resource_limit("$.root_export_assertions", "nested assertion identity expansion limit exceeded"));
                }
                Some(Box::new(identity))
            } else {
                None
            };
            if let Some(identity) = &target_root_assertion_identity {
                let child = identity.assertion();
                if child.exact_semantic_ir_digest != dependency.target_semantic_ir_digest
                    || Some(child.exact_exported_component_contract_digest.clone())
                        != dependency.target_exported_contract_digest
                {
                    return Err(failure(
                        "NestedAssertionTargetMismatch",
                        "$.root_export_assertions[].assertion.subflow_dependencies",
                        "nested target assertion identity differs from the declared exact IR/export pins",
                    ));
                }
            }
            Ok(SubflowProofDependency {
                call_site: dependency.call_site,
                target_semantic_ir_digest: dependency.target_semantic_ir_digest,
                target_exported_contract_digest: dependency.target_exported_contract_digest,
                target_root_assertion_identity,
                transitive_targets: dependency.transitive_targets,
            })
        })
        .collect::<Result<Vec<_>, PortableProofError>>()?;

    Ok(ProjectedAssertion {
        assertion: RootExportAssertion {
            exact_semantic_ir_digest: entry.assertion.exact_semantic_ir_digest,
            exact_exported_component_contract_digest: entry
                .assertion
                .exact_exported_component_contract_digest,
            compiler_semantics_identity: current_compiler_semantics_identity(),
            proof_context_manifest: ProofContextManifest::capture(context, snapshot),
            input_mappings,
            output_mappings,
            resource_mappings: entry.assertion.resource_mappings,
            effect_mappings,
            flow_mappings,
            recovery_mappings,
            execution_mappings: entry.assertion.execution_mappings,
            subflow_dependencies,
        },
        expanded_identity_work,
        comparison_work,
    })
}

fn convert_flow_mapping(
    mapping: PortableRootFlowMapping,
) -> Result<RootFlowCoordinateBinding, PortableProofError> {
    let source_binding = mapping.source_binding.map(|binding| match binding {
        PortableFlowSourceBinding::InputExpression(id) => FlowSourceBinding::InputExpression(id),
        PortableFlowSourceBinding::ResourceSlot(id) => FlowSourceBinding::ResourceSlot(id),
        PortableFlowSourceBinding::UnknownExternal => FlowSourceBinding::UnknownExternal,
    });
    let sink_binding = mapping
        .sink_binding
        .map(|binding| match binding {
            PortableFlowSinkBinding::OutputValue(id) => Ok(FlowSinkBinding::OutputValue(id)),
            PortableFlowSinkBinding::EffectOccurrences(ordinals) => ordinals
                .into_iter()
                .map(|ordinal| {
                    usize::try_from(ordinal).map_err(|_| {
                        resource_limit(
                            "$.root_export_assertions[].assertion.flow_mappings.sink_binding",
                            "effect occurrence ordinal exceeds host index range",
                        )
                    })
                })
                .collect::<Result<Vec<_>, _>>()
                .map(FlowSinkBinding::EffectOccurrences),
        })
        .transpose()?;
    Ok(RootFlowCoordinateBinding {
        source_node: mapping.source_node,
        source_contract_digest: mapping.source_contract_digest,
        flow_mapping_ordinal: mapping.flow_mapping_ordinal,
        source: mapping.source,
        sink: mapping.sink,
        source_binding,
        sink_binding,
        export_source: mapping.export_source,
        export_sink: mapping.export_sink,
        source_effect_partition: mapping.source_effect_partition,
        export_effect_partition: mapping.export_effect_partition,
    })
}

fn assertion_mapping_count(
    assertion: &PortableRootExportAssertion,
) -> Result<usize, PortableProofError> {
    let lengths = [
        assertion.input_mappings.len(),
        assertion.output_mappings.len(),
        assertion.resource_mappings.len(),
        assertion.effect_mappings.len(),
        assertion.flow_mappings.len(),
        assertion.recovery_mappings.len(),
        assertion.execution_mappings.len(),
        assertion.subflow_dependencies.len(),
    ];
    let total = lengths
        .iter()
        .try_fold(0usize, |sum, length| sum.checked_add(*length))
        .ok_or_else(|| {
            resource_limit(
                "$.root_export_assertions[].assertion",
                "mapping count overflow",
            )
        })?;
    if lengths
        .iter()
        .any(|length| *length > MAX_ASSERTION_MAPPINGS)
    {
        return Err(resource_limit(
            "$.root_export_assertions[].assertion",
            "one assertion mapping collection exceeds its work limit",
        ));
    }
    Ok(total)
}

fn structural_type_table(
    root: &Value,
) -> Result<BTreeMap<u32, StructuralTypeNode>, PortableProofError> {
    let rows = list_field(root, "structural_types", "$")?;
    if rows.len() > MAX_STRUCTURAL_NODES {
        return Err(resource_limit(
            "$.structural_types",
            "structural type node limit exceeded",
        ));
    }
    let mut nodes = BTreeMap::new();
    for row in rows {
        let node: StructuralTypeNode = from_value(row, "$.structural_types[]")?;
        if nodes.insert(node.id, node.clone()).is_some() {
            return Err(failure(
                "DuplicateStructuralType",
                "$.structural_types",
                "structural type id is duplicated",
            ));
        }
    }
    Ok(nodes)
}

fn expand_structural_type(
    id: u32,
    nodes: &BTreeMap<u32, StructuralTypeNode>,
    cache: &mut BTreeMap<u32, StructuralType>,
    active: &mut BTreeSet<u32>,
    work: &mut usize,
    depth: usize,
) -> Result<StructuralType, PortableProofError> {
    if depth > MAX_STRUCTURAL_EXPANSION_DEPTH {
        return Err(resource_limit(
            "$.structural_types",
            "structural type depth limit exceeded",
        ));
    }
    if let Some(value) = cache.get(&id) {
        return Ok(value.clone());
    }
    if !active.insert(id) {
        return Err(failure(
            "StructuralTypeCycle",
            "$.structural_types",
            "root coordinate type graph is cyclic",
        ));
    }
    *work = work
        .checked_add(1)
        .ok_or_else(|| resource_limit("$.structural_types", "structural type work overflow"))?;
    if *work > MAX_STRUCTURAL_EXPANSION_WORK {
        return Err(resource_limit(
            "$.structural_types",
            "structural type expansion work limit exceeded",
        ));
    }
    let node = nodes.get(&id).ok_or_else(|| {
        failure(
            "MissingStructuralType",
            "$.structural_types",
            "root coordinate references a missing structural type",
        )
    })?;
    let value =
        match &node.body {
            StructuralTypeBody::Primitive(primitive) => StructuralType::Primitive(*primitive),
            StructuralTypeBody::Schema(reference) => StructuralType::Schema(reference.clone()),
            StructuralTypeBody::List(inner) => StructuralType::List(Box::new(
                expand_structural_type(*inner, nodes, cache, active, work, depth + 1)?,
            )),
            StructuralTypeBody::MapString(inner) => StructuralType::MapString(Box::new(
                expand_structural_type(*inner, nodes, cache, active, work, depth + 1)?,
            )),
            StructuralTypeBody::Option(inner) => StructuralType::Option(Box::new(
                expand_structural_type(*inner, nodes, cache, active, work, depth + 1)?,
            )),
        };
    active.remove(&id);
    cache.insert(id, value.clone());
    Ok(value)
}

fn find_compiler_manifest(root: &Value, compiler_id: u32) -> Result<&Value, PortableProofError> {
    list_field(root, "compiler_semantics_manifests", "$")?
        .iter()
        .find(|entry| {
            u32_field(entry, "id", "$.compiler_semantics_manifests[]").ok() == Some(compiler_id)
        })
        .and_then(|entry| field(entry, "content", "$.compiler_semantics_manifests[]").ok())
        .ok_or_else(|| {
            failure(
                "MissingCompilerManifest",
                "$.root_export_assertions[].assertion.compiler_semantics_id",
                "compiler manifest id does not resolve",
            )
        })
}

fn list_field<'a>(
    root: &'a Value,
    name: &str,
    path: &str,
) -> Result<&'a [Value], PortableProofError> {
    let value = field(root, name, path)?;
    match value {
        Value::List(items) => Ok(items),
        _ => Err(failure("InvalidTypedContent", path, "expected a list")),
    }
}

fn field<'a>(value: &'a Value, name: &str, path: &str) -> Result<&'a Value, PortableProofError> {
    let Value::Record(fields) = value else {
        return Err(failure("InvalidTypedContent", path, "expected a record"));
    };
    fields
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value)
        .ok_or_else(|| failure("MissingField", path, "registered field is absent"))
}

fn u32_field(value: &Value, name: &str, path: &str) -> Result<u32, PortableProofError> {
    match field(value, name, path)? {
        Value::UInt32(value) => Ok(*value),
        Value::Int32(value) if *value >= 0 => u32::try_from(*value)
            .map_err(|_| failure("InvalidLocalId", path, "expected an exact uint32 id")),
        _ => Err(failure(
            "InvalidLocalId",
            path,
            "expected an exact uint32 id",
        )),
    }
}

fn from_value<T: DeserializeOwned>(value: &Value, path: &str) -> Result<T, PortableProofError> {
    let json =
        value_to_json(value).map_err(|detail| failure("InvalidTypedContent", path, detail))?;
    serde_json::from_value(json)
        .map_err(|error| failure("InvalidTypedContent", path, error.to_string()))
}

fn failure(
    family: &'static str,
    path: impl Into<String>,
    detail: impl Into<String>,
) -> PortableProofError {
    PortableProofError::new(ValidationStage::D10, family, path, detail)
}

fn resource_limit(path: impl Into<String>, detail: impl Into<String>) -> PortableProofError {
    PortableProofError::new(ValidationStage::D10, "ResourceLimit", path, detail)
}

pub(crate) fn project_root_summary(
    summary: &DerivedWorkflowContractSummary<'_, '_>,
) -> Result<(Json, Json), PortableProofError> {
    let root = summary.root();
    let closure = summary.closure();
    let tree = control_node_json(root, 0)?;
    let root_value = json!({
        "source_semantic_ir_digest":digest_json(root.source_semantic_ir_digest()),
        "overall_verdict":summary.overall().as_str(),
        "root_control":tree,
        "closure":{
            "computation_order":closure.computation_order().iter().map(|id|id.raw()).collect::<Vec<_>>(),
            "leaf_order":closure.leaf_order().iter().map(|id|id.raw()).collect::<Vec<_>>(),
            "known_effect_occurrences_accounted":closure.known_effect_occurrences_accounted(),
            "known_effect_residuals_accounted":closure.known_effect_residuals_accounted(),
            "known_recovery_sources_accounted":closure.known_recovery_sources_accounted(),
            "root_export_compatibility":"not_expressed_in_semantic_ir",
            "proof":control_proof_json(closure.proof().dimensions()),
        },
        "root_dimensions":{
            "failure":failure_dimension_json(root.failures()),
            "effect":effect_dimension_json(root.effects()),
            "resource":resource_dimension_json(root.resources()),
            "information_flow":flow_dimension_json(root.information_flow()),
            "usage":usage_dimension_json(root.usage()),
            "recovery_sources":recovery_sources_dimension_json(root.recovery()),
            "execution_sources":execution_dimension_json(root.execution()),
        },
        "effect_occurrences":root.effect_occurrences().iter().map(scoped_effect_occurrence_json).collect::<Vec<_>>(),
        "effect_residuals":root.effect_residuals().iter().map(scoped_effect_residual_json).collect::<Vec<_>>(),
        "resource_aliases":root.resource_aliases().iter().map(|alias|json!({
            "resource_slot":alias.resource_slot().raw(),
            "occurrences":alias.occurrences().iter().map(resource_occurrence_json).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "effect_lower_bound_adjustments":root.effect_lower_bound_adjustments().iter().map(|adjustment|json!({
            "source_node":adjustment.source_node().raw(),
            "reason":match adjustment.reason() {
                EffectLowerBoundAdjustmentReason::SequenceContinuationCanBeSkipped=>"sequence_continuation_can_be_skipped",
                EffectLowerBoundAdjustmentReason::FailFastSiblingCanBeCancelled=>"fail_fast_sibling_can_be_cancelled",
                EffectLowerBoundAdjustmentReason::RaceBranchMayLoseOrBeCancelled=>"race_branch_may_lose_or_be_cancelled",
                EffectLowerBoundAdjustmentReason::SelectMayHaveNoCoveredCase=>"select_may_have_no_covered_case",
            },
        })).collect::<Vec<_>>(),
        "sequence_dataflow_dependencies":root.sequence_dataflow_dependencies().iter().map(|dependency|json!({
            "producer_child":dependency.producer_child().raw(),
            "producer_node":dependency.producer_node().raw(),
            "value_slot":dependency.value_slot().raw(),
            "consumer_child":dependency.consumer_child().raw(),
            "consumer_node":dependency.consumer_node().raw(),
            "expression":dependency.expression().raw(),
        })).collect::<Vec<_>>(),
        "inherited_obligations":root.inherited_obligations().iter().map(|proof|json!({
            "source_node":proof.source_node().raw(),
            "dimension":proof.dimension(),
            "verdict":proof.verdict().as_str(),
            "rule_id":proof.rule_id(),
            "reference":proof.reference(),
        })).collect::<Vec<_>>(),
        "proof":control_proof_json(root.proof().dimensions()),
    });
    let recovery = root_recovery_json(summary.root_recovery());
    Ok((root_value, recovery))
}

fn control_node_json(
    summary: &DerivedControlContractSummary<'_, '_>,
    depth: usize,
) -> Result<Json, PortableProofError> {
    if depth > 96 {
        return Err(resource_limit(
            "$.root.control_tree",
            "control projection depth limit exceeded",
        ));
    }
    Ok(json!({
        "source_semantic_ir_digest":digest_json(summary.source_semantic_ir_digest()),
        "computation":summary.control_node().raw(),
        "operator":summary.operator().name(),
        "operator_semantics":operator_semantics_json(summary.operator()),
        "direct_children":summary.direct_children().iter().map(|id|id.raw()).collect::<Vec<_>>(),
        "interface":control_interface_json(summary.interface()),
        "effects":effect_dimension_json(summary.effects()),
        "failures":failure_dimension_json(summary.failures()),
        "resources":resource_dimension_json(summary.resources()),
        "information_flow":flow_dimension_json(summary.information_flow()),
        "usage":usage_dimension_json(summary.usage()),
        "recovery_sources":recovery_sources_dimension_json(summary.recovery()),
        "execution_sources":execution_dimension_json(summary.execution()),
        "scope_boundary":summary.scope_boundary().map(scope_boundary_json),
        "compensation_scope":summary.compensation_scope().map(compensation_scope_json),
        "children":summary.children().iter().map(|child|control_node_json(child,depth+1)).collect::<Result<Vec<_>,_>>()?,
        "proof":control_proof_json(summary.proof().dimensions()),
    }))
}

fn operator_semantics_json(operator: &ControlOperator) -> Json {
    match operator {
        ControlOperator::Parallel { mode, max_inflight } => {
            json!({"mode":mode.keyword(),"max_inflight":max_inflight})
        }
        ControlOperator::Iterate {
            mode,
            effect_repetitions,
            usage_repetitions,
        } => {
            let mode = match mode {
                af_computation::IterateMode::Foreach {
                    statically_known_item_count,
                    max_inflight,
                } => {
                    json!({"kind":"foreach","statically_known_item_count":statically_known_item_count,"max_inflight":max_inflight})
                }
                af_computation::IterateMode::While { max_iterations } => {
                    json!({"kind":"while","max_iterations":max_iterations})
                }
            };
            json!({"mode":mode,"effect_repetitions":bounds_json(effect_repetitions),"usage_repetitions":usage_repetitions})
        }
        ControlOperator::Retry {
            delays,
            max_attempts,
        } => {
            json!({"max_attempts":max_attempts,"delay_micros_in_order":delays.iter().map(af_core::Duration::as_micros).collect::<Vec<_>>()})
        }
        ControlOperator::Timeout {
            expression,
            static_duration,
            possible_outcomes,
        } => {
            json!({"expression":expression.raw(),"static_duration_micros":static_duration.map(|duration|duration.as_micros()),"possible_outcomes":possible_outcomes.iter().map(|outcome|match outcome {af_computation::TimeoutOutcome::ChildCompleted=>"child_completed",af_computation::TimeoutOutcome::TimedOut=>"timed_out"}).collect::<Vec<_>>()})
        }
        ControlOperator::Handle { handled, clauses } => {
            json!({"handled_failures":failure_row_json(handled),"clauses":clauses.iter().map(|clause|json!({"ordinal":clause.ordinal(),"matcher":match clause.matcher() {af_computation::HandleMatcherSummary::Failure(reference)=>json!({"failure":schema_ref_json(reference)}),af_computation::HandleMatcherSummary::TimedOut=>json!({"timed_out":true})},"handler":clause.handler().raw()})).collect::<Vec<_>>()})
        }
        ControlOperator::Unsupported { opcode } => json!({"opcode":opcode}),
        other => json!({"kind":other.name()}),
    }
}

fn control_interface_json(interface: &af_computation::ControlInterfaceSummary) -> Json {
    let outputs = match interface.outputs() {
        ControlDimensionValue::Complete(items) => {
            json!({"state":"complete","ports":items.iter().map(output_port_json).collect::<Vec<_>>()})
        }
        ControlDimensionValue::Partial {
            known_contribution,
            unresolved_sources,
        } => {
            json!({"state":"partial","ports":known_contribution.as_ref().map(|items|items.iter().map(output_port_json).collect::<Vec<_>>()),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::Unknown {
            known_contributions,
            unresolved_sources,
        } => {
            json!({"state":"unknown","known_contributions":known_contributions.iter().map(|item|json!({"source_node":item.source_node().raw(),"ports":item.value().iter().map(output_port_json).collect::<Vec<_>>()})).collect::<Vec<_>>(),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::PerChild {
            contributions,
            unresolved_sources,
        } => {
            json!({"state":"per_child","contributions":contributions.iter().map(|item|json!({"source_node":item.source_node().raw(),"ports":item.value().iter().map(output_port_json).collect::<Vec<_>>()})).collect::<Vec<_>>(),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::NotApplicable => json!({"state":"not_applicable"}),
    };
    json!({"outputs":outputs,"select_decision":interface.selector().map(|selector|json!({"expression":selector.expression().raw(),"dependencies":selector.dependency_slots().iter().map(|id|id.raw()).collect::<Vec<_>>(),"case_ordinals":selector.case_ordinals(),"exact_case":selector.exact_case(),"totality":selector.totality_verdict().as_str()}))})
}

fn output_port_json(port: &af_computation::ControlOutputPort) -> Json {
    json!({"parent_port":port.parent_port_no(),"parent_value_slot":port.parent_value_slot().raw(),"local_type_id":port.local_type_id().raw(),"sources":port.sources().iter().map(|source|json!({"child_node":source.child_node().raw(),"child_port":source.child_port_no(),"child_value_slot":source.child_value_slot().raw()})).collect::<Vec<_>>()})
}

fn control_proof_json(proofs: &[ControlDimensionProof]) -> Json {
    json!(proofs.iter().map(|proof|json!({"dimension":proof.dimension(),"verdict":proof.verdict().as_str(),"rule_id":proof.rule_id(),"source_node":proof.source_node().raw(),"reference":proof.reference(),"reason":proof.reason(),"prerequisites":proof.prerequisites()})).collect::<Vec<_>>())
}

fn failure_dimension_json(dimension: &ControlDimension<FailureRow>) -> Json {
    match dimension.value() {
        ControlDimensionValue::Complete(value) => {
            json!({"state":"complete","value":failure_row_json(value)})
        }
        ControlDimensionValue::Partial {
            known_contribution,
            unresolved_sources,
        } => {
            json!({"state":"partial","value":known_contribution.as_ref().map(failure_row_json),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::Unknown {
            known_contributions,
            unresolved_sources,
        } => {
            json!({"state":"unknown","known_contributions":known_contributions.iter().map(|item|json!({"source_node":item.source_node().raw(),"value":failure_row_json(item.value())})).collect::<Vec<_>>(),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::PerChild {
            contributions,
            unresolved_sources,
        } => {
            json!({"state":"per_child","contributions":contributions.iter().map(|item|json!({"source_node":item.source_node().raw(),"value":failure_row_json(item.value())})).collect::<Vec<_>>(),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::NotApplicable => json!({"state":"not_applicable"}),
    }
}

fn effect_dimension_json(dimension: &ControlDimension<EffectSignature>) -> Json {
    match dimension.value() {
        ControlDimensionValue::Complete(value) => {
            json!({"state":"complete","value":effect_signature_json(value)})
        }
        ControlDimensionValue::Partial {
            known_contribution,
            unresolved_sources,
        } => {
            json!({"state":"partial","value":known_contribution.as_ref().map(effect_signature_json),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::Unknown {
            known_contributions,
            unresolved_sources,
        } => {
            json!({"state":"unknown","known_contributions":known_contributions.iter().map(|item|json!({"source_node":item.source_node().raw(),"value":effect_signature_json(item.value())})).collect::<Vec<_>>(),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::PerChild {
            contributions,
            unresolved_sources,
        } => {
            json!({"state":"per_child","contributions":contributions.iter().map(|item|json!({"source_node":item.source_node().raw(),"value":effect_signature_json(item.value())})).collect::<Vec<_>>(),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::NotApplicable => json!({"state":"not_applicable"}),
    }
}

fn resource_dimension_json(dimension: &ControlDimension<Vec<ResourcePortOccurrence>>) -> Json {
    match dimension.value() {
        ControlDimensionValue::Complete(items) => {
            json!({"state":"complete","occurrences":items.iter().map(resource_occurrence_json).collect::<Vec<_>>()})
        }
        ControlDimensionValue::Partial {
            known_contribution,
            unresolved_sources,
        } => {
            json!({"state":"partial","occurrences":known_contribution.as_ref().map(|items|items.iter().map(resource_occurrence_json).collect::<Vec<_>>()),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::Unknown {
            known_contributions,
            unresolved_sources,
        } => {
            json!({"state":"unknown","known_contributions":known_contributions.iter().map(|item|json!({"source_node":item.source_node().raw(),"occurrences":item.value().iter().map(resource_occurrence_json).collect::<Vec<_>>()})).collect::<Vec<_>>(),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::PerChild {
            contributions,
            unresolved_sources,
        } => {
            json!({"state":"per_child","contributions":contributions.iter().map(|item|json!({"source_node":item.source_node().raw(),"occurrences":item.value().iter().map(resource_occurrence_json).collect::<Vec<_>>()})).collect::<Vec<_>>(),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::NotApplicable => json!({"state":"not_applicable"}),
    }
}

fn flow_dimension_json(dimension: &ControlDimension<ControlFlowSummary>) -> Json {
    match dimension.value() {
        ControlDimensionValue::Complete(value) => {
            json!({"state":"complete","value":control_flow_json(value)})
        }
        ControlDimensionValue::Partial {
            known_contribution,
            unresolved_sources,
        } => {
            json!({"state":"partial","value":known_contribution.as_ref().map(control_flow_json),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::Unknown {
            known_contributions,
            unresolved_sources,
        } => {
            json!({"state":"unknown","known_contributions":known_contributions.iter().map(|item|json!({"source_node":item.source_node().raw(),"value":control_flow_json(item.value())})).collect::<Vec<_>>(),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::PerChild {
            contributions,
            unresolved_sources,
        } => {
            json!({"state":"per_child","contributions":contributions.iter().map(|item|json!({"source_node":item.source_node().raw(),"value":control_flow_json(item.value())})).collect::<Vec<_>>(),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::NotApplicable => json!({"state":"not_applicable"}),
    }
}

fn usage_dimension_json(dimension: &ControlDimension<StaticUsageEnvelope>) -> Json {
    match dimension.value() {
        ControlDimensionValue::Complete(value) => {
            json!({"state":"complete","value":usage_envelope_json(value)})
        }
        ControlDimensionValue::Partial {
            known_contribution,
            unresolved_sources,
        } => {
            json!({"state":"partial","value":known_contribution.as_ref().map(usage_envelope_json),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::Unknown {
            known_contributions,
            unresolved_sources,
        } => {
            json!({"state":"unknown","known_contributions":known_contributions.iter().map(|item|json!({"source_node":item.source_node().raw(),"value":usage_envelope_json(item.value())})).collect::<Vec<_>>(),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::PerChild {
            contributions,
            unresolved_sources,
        } => {
            json!({"state":"per_child","contributions":contributions.iter().map(|item|json!({"source_node":item.source_node().raw(),"value":usage_envelope_json(item.value())})).collect::<Vec<_>>(),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::NotApplicable => json!({"state":"not_applicable"}),
    }
}

fn recovery_sources_dimension_json(
    dimension: &ControlDimension<Vec<RecoverySourceSummary>>,
) -> Json {
    match dimension.value() {
        ControlDimensionValue::Complete(items) => {
            json!({"state":"complete","sources":items.iter().map(recovery_source_json).collect::<Vec<_>>()})
        }
        ControlDimensionValue::Partial {
            known_contribution,
            unresolved_sources,
        } => {
            json!({"state":"partial","sources":known_contribution.as_ref().map(|items|items.iter().map(recovery_source_json).collect::<Vec<_>>()),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::Unknown {
            known_contributions,
            unresolved_sources,
        } => {
            json!({"state":"unknown","known_contributions":known_contributions.iter().map(|item|json!({"source_node":item.source_node().raw(),"sources":item.value().iter().map(recovery_source_json).collect::<Vec<_>>()})).collect::<Vec<_>>(),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::PerChild {
            contributions,
            unresolved_sources,
        } => {
            json!({"state":"per_child","contributions":contributions.iter().map(|item|json!({"source_node":item.source_node().raw(),"sources":item.value().iter().map(recovery_source_json).collect::<Vec<_>>()})).collect::<Vec<_>>(),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::NotApplicable => json!({"state":"not_applicable"}),
    }
}

fn execution_dimension_json(dimension: &ControlDimension<Vec<ExecutionSourceSummary>>) -> Json {
    match dimension.value() {
        ControlDimensionValue::Complete(items) => {
            json!({"state":"complete","sources":items.iter().map(execution_source_json).collect::<Vec<_>>()})
        }
        ControlDimensionValue::Partial {
            known_contribution,
            unresolved_sources,
        } => {
            json!({"state":"partial","sources":known_contribution.as_ref().map(|items|items.iter().map(execution_source_json).collect::<Vec<_>>()),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::Unknown {
            known_contributions,
            unresolved_sources,
        } => {
            json!({"state":"unknown","known_contributions":known_contributions.iter().map(|item|json!({"source_node":item.source_node().raw(),"sources":item.value().iter().map(execution_source_json).collect::<Vec<_>>()})).collect::<Vec<_>>(),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::PerChild {
            contributions,
            unresolved_sources,
        } => {
            json!({"state":"per_child","contributions":contributions.iter().map(|item|json!({"source_node":item.source_node().raw(),"sources":item.value().iter().map(execution_source_json).collect::<Vec<_>>()})).collect::<Vec<_>>(),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::NotApplicable => json!({"state":"not_applicable"}),
    }
}

fn failure_row_json(row: &FailureRow) -> Json {
    json!(
        row.failures
            .iter()
            .map(|item| schema_ref_json(&item.payload_type))
            .collect::<Vec<_>>()
    )
}

fn effect_signature_json(signature: &EffectSignature) -> Json {
    match &signature.row {
        EffectRow::Unknown => json!({"row":"unknown"}),
        EffectRow::Closed(entries) => {
            json!({"row":"closed","entries":entries.iter().map(effect_entry_json).collect::<Vec<_>>()})
        }
        EffectRow::Open(EffectOpenRow { known, residual }) => {
            json!({"row":"open","known":known.iter().map(effect_entry_json).collect::<Vec<_>>(),"residual":residual.iter().map(effect_residual_json).collect::<Vec<_>>()})
        }
    }
}

fn effect_entry_json(entry: &EffectEntry) -> Json {
    json!({"atom":effect_atom_json(&entry.atom),"bounds":bounds_json(&entry.bounds),"guard":effect_guard_json(&entry.guard)})
}

fn effect_residual_json(residual: &EffectResidual) -> Json {
    json!({"catalog":residual.catalog,"selector":resource_selector_json(&residual.selector),"bounds":bounds_json(&residual.bounds)})
}

fn effect_atom_json(atom: &EffectAtom) -> Json {
    let qualifiers = atom
        .qualifiers
        .iter()
        .map(|qualifier| match qualifier {
            EffectQualifier::Idempotency(EffectIdempotencyQualifier::Natural) => {
                json!({"family":"idempotency","member":"natural"})
            }
            EffectQualifier::Idempotency(EffectIdempotencyQualifier::None) => {
                json!({"family":"idempotency","member":"none"})
            }
        })
        .collect::<Vec<_>>();
    json!({"operation":{"catalog":atom.operation.catalog,"name":atom.operation.name},"resource_selector":resource_selector_json(&atom.resource_selector),"qualifiers":qualifiers})
}

fn resource_selector_json(selector: &ResourceSelector) -> Json {
    json!({"requirement_port_no":selector.requirement_port_no,"scope_bindings":selector.scope_bindings.iter().map(|binding|json!({"key":binding.key,"value":binding.value})).collect::<Vec<_>>()})
}

fn effect_guard_json(guard: &EffectGuard) -> Json {
    match guard {
        EffectGuard::Unconditional => json!({"unconditional":true}),
        EffectGuard::InputEquals(value) => {
            json!({"input_equals":{"port_no":value.port_no,"expected":primitive_constant_json(&value.expected)}})
        }
    }
}

fn primitive_constant_json(value: &PrimitiveConstant) -> Json {
    match value {
        PrimitiveConstant::Unit => json!({"unit":null}),
        PrimitiveConstant::Null => json!({"null":null}),
        PrimitiveConstant::Bool(value) => json!({"bool":value}),
        PrimitiveConstant::Int64(value) => json!({"int64":value}),
        PrimitiveConstant::Decimal(value) => json!({"decimal":value.as_str()}),
        PrimitiveConstant::String(value) => json!({"string":value}),
        PrimitiveConstant::Bytes(value) => json!({"bytes":value}),
        PrimitiveConstant::Timestamp(value) => json!({"timestamp":value.as_micros()}),
        PrimitiveConstant::Duration(value) => json!({"duration":value.as_micros()}),
    }
}

fn bounds_json(bounds: &OccurrenceBounds) -> Json {
    json!({"lower":bounds.lower,"upper":ceiling_json(bounds.upper)})
}

fn ceiling_json(upper: OccurrenceCeiling) -> Json {
    match upper {
        OccurrenceCeiling::Finite(value) => json!({"finite":value}),
        OccurrenceCeiling::Unbounded => json!({"unbounded":null}),
    }
}

fn resource_occurrence_json(item: &ResourcePortOccurrence) -> Json {
    json!({
        "source_node":item.source_node().raw(),
        "target":resolved_target_json(item.target()),
        "requirement":item.requirement().map(resource_requirement_json),
        "mapping":{
            "component_port_no":item.mapping().component_port_no,
            "resource_slot":item.mapping().resource_slot.raw(),
            "required_type":schema_ref_json(&item.mapping().required_type),
            "required_attributes_proven":item.mapping().required_attributes_proven,
        }
    })
}

fn resource_requirement_json(item: &ResourceRequirement) -> Json {
    json!({"port_no":item.port_no,"resource_type":resource_type_ref_json(&item.resource_type),"required_attributes":item.required_attributes.iter().map(|attribute|json!({"key":attribute.key,"value":attribute.value})).collect::<Vec<_>>()})
}

fn resolved_target_json(target: &af_computation::ResolvedTarget) -> Json {
    match target {
        af_computation::ResolvedTarget::Component {
            identity,
            contract_digest,
        } => {
            json!({"kind":"component","identity":{"catalog":identity.catalog,"name":identity.name},"contract_digest":digest_json(contract_digest)})
        }
        af_computation::ResolvedTarget::Subflow {
            semantic_ir_digest,
            exported_contract_digest,
        } => {
            json!({"kind":"subflow","semantic_ir_digest":digest_json(semantic_ir_digest),"exported_contract_digest":exported_contract_digest.as_ref().map(digest_json)})
        }
        af_computation::ResolvedTarget::AwaitSignal { signal_type } => {
            json!({"kind":"await_signal","signal_type":schema_ref_json(signal_type)})
        }
    }
}

fn control_flow_json(flow: &ControlFlowSummary) -> Json {
    let may_influence = flow.known_may_influence().map(static_information_json);
    json!({
        "known_may_influence":may_influence,
        "unresolved_children":flow.unresolved_children().iter().map(|id|id.raw()).collect::<Vec<_>>(),
        "sources":flow.sources().iter().map(|source|json!({
            "source_node":source.source_node().raw(),
            "target":resolved_target_json(source.target()),
            "contract":information_flow_contract_json(source.contract()),
            "source_mappings":source.source_mappings().iter().map(flow_port_mapping_json).collect::<Vec<_>>(),
            "effect_flow_partitions":source.effect_flow_partitions().iter().map(effect_flow_partition_json).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "sequence_dependencies":flow.sequence_dependencies().iter().map(|item|json!({"producer_child":item.producer_child().raw(),"producer_node":item.producer_node().raw(),"value_slot":item.value_slot().raw(),"consumer_child":item.consumer_child().raw(),"consumer_node":item.consumer_node().raw(),"expression":item.expression().raw()})).collect::<Vec<_>>(),
        "select_decision_dependencies":flow.select_decision_dependencies().iter().map(|id|id.raw()).collect::<Vec<_>>(),
        "control_decision_dependencies":flow.control_decision_dependencies().iter().map(|id|id.raw()).collect::<Vec<_>>(),
        "control_decision_influence_unknown":flow.control_decision_influence_unknown(),
        "unknown_external_children":flow.unknown_external_children().iter().map(|id|id.raw()).collect::<Vec<_>>(),
    })
}

fn static_information_json(summary: &StaticInformationSummary) -> Json {
    json!({"sources":summary.sources.iter().map(flow_source_json).collect::<Vec<_>>(),"taints":summary.taints.iter().map(|item|json!({"catalog":item.catalog,"name":item.name})).collect::<Vec<_>>(),"instruction_authority":summary.instruction_authority.as_str(),"influence_unknown":summary.influence_unknown})
}

fn information_flow_contract_json(contract: &InformationFlowContract) -> Json {
    json!({"confidentiality_restrictions":contract.confidentiality_restrictions.iter().map(|item|json!({"catalog":item.catalog,"name":item.name})).collect::<Vec<_>>(),"summary":static_information_json(&contract.summary),"flow_constraints":contract.flow_constraints.iter().map(flow_constraint_json).collect::<Vec<_>>()})
}

fn flow_source_json(source: &FlowSourceRef) -> Json {
    match source.category {
        SourceCategory::WorkflowInput(port) => json!({"workflow_input":port}),
        SourceCategory::ResourceRequirement(port) => json!({"resource_requirement":port}),
        SourceCategory::UnknownExternal => json!({"unknown_external":null}),
    }
}

fn flow_constraint_json(constraint: &FlowConstraint) -> Json {
    let sink = match &constraint.sink {
        FlowSink::WorkflowOutput(port) => json!({"workflow_output":port}),
        FlowSink::ExternalEffect(operation) => {
            json!({"external_effect":{"catalog":operation.catalog,"name":operation.name}})
        }
    };
    json!({"kind":constraint.kind.as_str(),"sink":sink})
}

fn flow_port_mapping_json(mapping: &af_computation::FlowPortMapping) -> Json {
    json!({
        "source":mapping.source.as_ref().map(flow_source_json),
        "sink":mapping.sink.as_ref().map(flow_constraint_json),
        "source_binding":mapping.source_binding.as_ref().map(|binding|match binding {
            FlowSourceBinding::InputExpression(id)=>json!({"input_expression":id.raw()}),
            FlowSourceBinding::ResourceSlot(id)=>json!({"resource_slot":id.raw()}),
            FlowSourceBinding::UnknownExternal=>json!({"unknown_external":null}),
        }),
        "sink_binding":mapping.sink_binding.as_ref().map(|binding|match binding {
            FlowSinkBinding::OutputValue(id)=>json!({"output_value":id.raw()}),
            FlowSinkBinding::EffectOccurrences(indices)=>json!({"effect_occurrences":indices}),
        }),
        "matched_effect_occurrences":mapping.matched_effect_occurrences,
    })
}

fn effect_flow_partition_json(partition: &EffectFlowPartition) -> Json {
    json!({"operation":{"catalog":partition.operation.catalog,"name":partition.operation.name},"kind":partition.kind.as_str(),"allowed_effects":partition.allowed_effects.iter().map(effect_atom_json).collect::<Vec<_>>()})
}

fn recovery_source_json(source: &RecoverySourceSummary) -> Json {
    json!({
        "source_node":source.source_node().raw(),
        "target":resolved_target_json(source.target()),
        "replay_semantics":source.contract().replay_semantics.as_str(),
        "effect_recovery_clauses":source.contract().effect_recovery_clauses.iter().map(|clause|json!({
            "effect":effect_atom_json(&clause.effect),
            "idempotency":clause.idempotency.as_str(),
            "key_requirement":clause.key_requirement.as_ref().map(|key|json!({"stable_across_attempts":key.stable_across_attempts,"distinct_across_invocations":key.distinct_across_invocations,"bound_to_operation":key.bound_to_operation,"provider_honors":key.provider_honors})),
            "reconciliation":format!("{:?}",clause.reconciliation).to_lowercase(),
            "compensation":match &clause.compensation {af_recovery::CompensationRequirement::None=>json!({"none":null}),af_recovery::CompensationRequirement::Required(operation)=>json!({"required":{"catalog":operation.catalog,"name":operation.name}})},
            "unknown_effect_handling":format!("{:?}",clause.unknown_effect_handling).to_lowercase(),
        })).collect::<Vec<_>>(),
        "action_bindings":source.action_bindings().iter().map(|item|recovery_action_identity_json(&item.identity)).collect::<Vec<_>>(),
        "usage_bindings":source.usage_bindings().iter().map(recovery_usage_binding_json).collect::<Vec<_>>(),
        "effect_occurrences":source.effects().iter().map(scoped_effect_occurrence_json).collect::<Vec<_>>(),
        "action_guard_compatibility":source.action_guard_compatibility().iter().map(action_guard_compatibility_json).collect::<Vec<_>>(),
    })
}

fn execution_source_json(source: &ExecutionSourceSummary) -> Json {
    json!({"source_node":source.source_node().raw(),"target":resolved_target_json(source.target()),"requirements":execution_requirements_json(source.requirements())})
}

fn scope_boundary_json(scope: &ScopeBoundarySummary) -> Json {
    json!({"control_node":scope.control_node().raw(),"child":scope.child().raw(),"parent_visible_resources":scope.parent_visible_resources().iter().map(|id|id.raw()).collect::<Vec<_>>(),"declared_resources":scope.declared_resources().iter().map(|id|id.raw()).collect::<Vec<_>>(),"child_visible_resources":scope.child_visible_resources().iter().map(|id|id.raw()).collect::<Vec<_>>(),"resource_attenuation":scope.resource_attenuation().as_str(),"authority_attenuation":scope.authority_attenuation().as_str(),"budget_attenuation":scope.budget_attenuation().as_str(),"owned_children_settle_before_terminal":scope.owned_children_settle_before_terminal().as_str()})
}

fn digest_json(digest: &DigestRefValue) -> Json {
    json!({"algorithm":digest.algorithm,"domain":digest.domain,"hex":digest.hex})
}

fn schema_ref_json(schema: &TypeSchemaRef) -> Json {
    json!({"specification_id":schema.specification_id,"type_name":schema.type_name,"schema_digest":digest_json(&schema.schema_digest)})
}

fn resource_type_ref_json(resource_type: &ResourceTypeRef) -> Json {
    schema_ref_json(&resource_type.schema)
}

fn scoped_effect_occurrence_json(occurrence: &ScopedEffectOccurrence) -> Json {
    json!({
        "caller_semantic_ir_digest":digest_json(&occurrence.caller_semantic_ir_digest),
        "source_contract_digest":digest_json(&occurrence.source_contract_digest),
        "source_component":{"catalog":occurrence.source_component.catalog,"name":occurrence.source_component.name},
        "computation":occurrence.computation.raw(),
        "source_entry_ordinal":occurrence.source_entry_ordinal,
        "declared":effect_entry_json(&occurrence.declared),
        "mapped_resource_slot":occurrence.mapped_resource_slot.map(ResourceSlotId::raw),
        "control_families":occurrence.control_families.iter().map(|family|json!({
            "control_node":family.control_node.raw(),
            "kind":match family.kind { af_computation::EffectOccurrenceFamilyKind::Iterate=>"iterate", af_computation::EffectOccurrenceFamilyKind::Retry=>"retry" },
            "logical_repetitions":bounds_json(&family.logical_repetitions),
            "physical_attempts":family.physical_attempts.as_ref().map(bounds_json),
        })).collect::<Vec<_>>(),
    })
}

fn scoped_effect_residual_json(residual: &ScopedEffectResidual) -> Json {
    json!({
        "caller_semantic_ir_digest":digest_json(&residual.caller_semantic_ir_digest),
        "source_contract_digest":digest_json(&residual.source_contract_digest),
        "source_component":{"catalog":residual.source_component.catalog,"name":residual.source_component.name},
        "computation":residual.computation.raw(),
        "source_entry_ordinal":residual.source_entry_ordinal,
        "residual":effect_residual_json(&residual.residual),
        "mapped_resource_slot":residual.mapped_resource_slot.map(ResourceSlotId::raw),
    })
}

fn usage_envelope_json(envelope: &StaticUsageEnvelope) -> Json {
    json!({
        "globally_unknown":envelope.is_globally_unknown(),
        "coordinates":envelope.iter().map(|(coordinate,bound)|json!({
            "phase":coordinate.phase.as_str(),
            "meter":{"catalog":coordinate.meter.catalog,"name":coordinate.meter.name},
            "bound":match bound {
                StaticUsageBound::KnownFinite(value)=>json!({"state":"known_finite","upper":value}),
                StaticUsageBound::KnownUnbounded=>json!({"state":"known_unbounded"}),
                StaticUsageBound::Unknown=>json!({"state":"unknown"}),
            },
        })).collect::<Vec<_>>(),
    })
}

fn usage_signature_json(signature: &af_usage::UsageSignature) -> Json {
    json!(signature.limits.iter().map(|limit|json!({
        "meter":{"catalog":limit.meter.catalog,"name":limit.meter.name},
        "phase":limit.phase.as_str(),
        "upper":match limit.upper { af_usage::UsageUpperBound::Finite(value)=>json!({"finite":value}), af_usage::UsageUpperBound::Unbounded=>json!({"unbounded":null}) },
    })).collect::<Vec<_>>())
}

fn recovery_action_identity_json(identity: &RecoveryActionIdentity) -> Json {
    json!({
        "forward_effect":effect_atom_json(&identity.forward_effect),
        "action_kind":identity.action_kind.as_str(),
        "action_effect":effect_atom_json(&identity.action_effect),
    })
}

fn recovery_usage_binding_json(binding: &RecoveryUsageBinding) -> Json {
    json!({
        "action":recovery_action_identity_json(&binding.action),
        "activations_per_forward":bounds_json(&binding.activations_per_forward),
        "attempts_per_activation":bounds_json(&binding.attempts_per_activation),
        "per_attempt_usage":usage_signature_json(&binding.per_attempt_usage),
    })
}

fn action_guard_compatibility_json(proof: &af_contract::ActionGuardCompatibility) -> Json {
    json!({
        "source_component":{"catalog":proof.source_component().catalog,"name":proof.source_component().name},
        "recovery_action":recovery_action_identity_json(proof.recovery_action()),
        "forward_entry_ordinal":proof.forward_entry_ordinal(),
        "action_entry_ordinal":proof.action_entry_ordinal(),
        "forward_guard":proof.forward_guard().map(effect_guard_json),
        "declared_action_guard":proof.declared_action_guard().map(effect_guard_json),
        "action_upper":proof.action_upper().map(ceiling_json),
        "verdict":proof.verdict().as_str(),
        "basis":match proof.basis() {
            af_contract::GuardCompatibilityBasis::FrozenGuardContainment=>"frozen_guard_containment",
            af_contract::GuardCompatibilityBasis::ZeroPossibleActions=>"zero_possible_actions",
            af_contract::GuardCompatibilityBasis::TypedCountermodel=>"typed_countermodel",
            af_contract::GuardCompatibilityBasis::InsufficientEvidence=>"insufficient_evidence",
        },
        "reason":proof.reason(),
        "countermodel":proof.countermodel().map(|countermodel|json!({
            "source_component":{"catalog":countermodel.source_component().catalog,"name":countermodel.source_component().name},
            "recovery_action":recovery_action_identity_json(countermodel.recovery_action()),
            "forward_entry_ordinal":countermodel.forward_entry_ordinal(),
            "action_entry_ordinal":countermodel.action_entry_ordinal(),
            "forward_guard":effect_guard_json(countermodel.forward_guard()),
            "declared_action_guard":effect_guard_json(countermodel.declared_action_guard()),
            "forward_bounds":bounds_json(countermodel.forward_bounds()),
            "activation_upper":ceiling_json(countermodel.activation_upper()),
            "attempt_upper":ceiling_json(countermodel.attempt_upper()),
            "action_upper":ceiling_json(countermodel.action_upper()),
            "assignments":countermodel.assignments().iter().map(|assignment|json!({"port_no":assignment.port_no(),"type":schema_ref_json(assignment.type_ref()),"value":primitive_constant_json(assignment.value())})).collect::<Vec<_>>(),
            "reachability":match countermodel.reachability(){af_contract::GuardCountermodelReachability::WellTypedComponentInputDomain=>"well_typed_component_input_domain"},
            "related_rule_id":countermodel.related_rule_id(),
        })),
    })
}

fn execution_requirements_json(requirements: &ExecutionRequirements) -> Json {
    json!({
        "capabilities":requirements.capabilities.iter().map(|item|json!({"capability":{"catalog":item.capability.catalog,"name":item.capability.name},"exact_version":item.exact_version,"required_attributes":item.required_attributes.iter().map(|attribute|json!({"key":attribute.key,"value":attribute.value})).collect::<Vec<_>>()})).collect::<Vec<_>>(),
        "quantities":requirements.quantities.iter().map(|item|json!({"kind_no":item.kind.value_no(),"minimum":item.minimum})).collect::<Vec<_>>(),
        "sandbox":requirements.sandbox.as_ref().map(|item|json!({"minimum":match item.minimum {SandboxLevel::Basic=>"basic",SandboxLevel::Strong=>"strong"}})),
        "trust_domain":requirements.trust_domain.as_ref().map(|item|json!({"catalog":item.catalog,"name":item.name})),
        "region_residency":requirements.region_residency.as_ref().map(|item|json!({"execution_regions":item.execution_regions.iter().map(|region|json!({"catalog":region.catalog,"name":region.name})).collect::<Vec<_>>(),"residency_regions":item.residency_regions.iter().map(|region|json!({"catalog":region.catalog,"name":region.name})).collect::<Vec<_>>()})),
        "runtime_profile":requirements.runtime_profile.as_ref().map(|item|json!({"family":item.family,"profile":{"catalog":item.profile.catalog,"name":item.profile.name},"required_features":item.required_features})),
        "implementation_abis":requirements.implementation_abis.iter().map(|item|schema_ref_json(&item.schema)).collect::<Vec<_>>(),
        "soft_preferences":requirements.soft_preferences.iter().map(|item|item.value_no()).collect::<Vec<_>>(),
    })
}

fn root_recovery_json(summary: &af_computation::RootRecoverySummary) -> Json {
    let actions = match summary.actions().value() {
        ControlDimensionValue::Complete(items) => {
            json!({"state":"complete","actions":items.iter().map(root_recovery_action_json).collect::<Vec<_>>()})
        }
        ControlDimensionValue::Partial {
            known_contribution,
            unresolved_sources,
        } => {
            json!({"state":"partial","actions":known_contribution.as_ref().map(|items|items.iter().map(root_recovery_action_json).collect::<Vec<_>>()),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::Unknown {
            known_contributions,
            unresolved_sources,
        } => {
            json!({"state":"unknown","known_contributions":known_contributions.iter().map(|item|json!({"source_node":item.source_node().raw(),"actions":item.value().iter().map(root_recovery_action_json).collect::<Vec<_>>()})).collect::<Vec<_>>(),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::PerChild {
            contributions,
            unresolved_sources,
        } => {
            json!({"state":"per_child","contributions":contributions.iter().map(|item|json!({"source_node":item.source_node().raw(),"actions":item.value().iter().map(root_recovery_action_json).collect::<Vec<_>>()})).collect::<Vec<_>>(),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::NotApplicable => json!({"state":"not_applicable"}),
    };
    json!({
        "actions":actions,
        "action_effects":effect_dimension_json(summary.action_effects()),
        "epsilon_coverage_requirement":effect_dimension_json(summary.epsilon_coverage_requirement()),
        "recovery_usage":usage_dimension_json(summary.recovery_usage()),
        "source_closure":{
            "source_nodes_accounted":summary.source_closure().source_nodes_accounted.as_str(),
            "action_bindings_accounted":summary.source_closure().action_bindings_accounted.as_str(),
            "usage_profiles_accounted":summary.source_closure().usage_profiles_accounted.as_str(),
            "multiplicity_factors_accounted":summary.source_closure().multiplicity_factors_accounted.as_str(),
            "root_action_totals_sound":summary.source_closure().root_action_totals_sound.as_str(),
        },
        "proof_dimensions":control_proof_json(summary.proof().dimensions()),
        "overall_verdict":summary.overall().as_str(),
    })
}

fn root_recovery_action_json(action: &RootRecoveryAction) -> Json {
    json!({
        "source":{
            "caller_semantic_ir_digest":digest_json(&action.source.caller_semantic_ir_digest),
            "source_node":action.source.source_node.raw(),
            "source_contract_digest":digest_json(&action.source.source_contract_digest),
            "forward_entry_ordinal":action.source.forward_entry_ordinal,
            "action":recovery_action_identity_json(&action.source.action),
        },
        "forward_seed_upper":ceiling_json(action.forward_seed_upper),
        "target":resolved_target_json(&action.target),
        "owner":match action.owner {RootRecoveryOwner::CompensationScope(node)=>json!({"kind":"compensation_scope","node":node.raw()}),RootRecoveryOwner::RootRecovery=>json!({"kind":"root_recovery"}),},
        "local_bound":action.local_bound.as_ref().map(local_action_bound_json),
        "root_bound":{"root_semantic_ir_digest":digest_json(&action.root_bound.root_semantic_ir_digest),"upper":recovery_action_upper_json(action.root_bound.upper),"applied_families":action.root_bound.applied_families.iter().map(applied_multiplicity_json).collect::<Vec<_>>()},
        "forward_occurrence":scoped_effect_occurrence_json(&action.forward_occurrence),
        "action_effect_occurrence":scoped_effect_occurrence_json(&action.action_effect_occurrence),
        "recovery_usage_binding":recovery_usage_binding_json(&action.recovery_usage_binding),
        "local_recovery_usage":action.local_recovery_usage.as_ref().map(usage_envelope_json),
        "root_recovery_usage":usage_envelope_json(&action.root_recovery_usage),
        "guard_compatibility":action.guard_compatibility.as_ref().map(action_guard_compatibility_json),
        "guard_coordinates":action.guard_coordinates.iter().map(|coordinate|json!({"source_contract_digest":digest_json(&coordinate.source_contract_digest),"component_input_port":coordinate.component_input_port,"input_expression":coordinate.input_expression.raw(),"exact_type":coordinate.exact_type.as_ref().map(schema_ref_json),"dependency_slots":coordinate.dependency_slots.iter().map(|slot|slot.raw()).collect::<Vec<_>>()})).collect::<Vec<_>>(),
        "eligibility_evidence":action.eligibility_evidence.as_str(),
        "control_path":action.control_path.iter().map(control_path_step_json).collect::<Vec<_>>(),
    })
}

fn local_action_bound_json(bound: &LocalActionBound) -> Json {
    json!({"compensation_scope_node":bound.compensation_scope_node.raw(),"upper":recovery_action_upper_json(bound.upper),"applied_families":bound.applied_families.iter().map(applied_multiplicity_json).collect::<Vec<_>>()})
}

fn recovery_action_upper_json(upper: RecoveryActionUpper) -> Json {
    match upper {
        RecoveryActionUpper::Known(value) => json!({"state":"known","upper":ceiling_json(value)}),
        RecoveryActionUpper::Overflow => json!({"state":"overflow"}),
        RecoveryActionUpper::InvalidMultiplicity => json!({"state":"invalid_multiplicity"}),
        RecoveryActionUpper::Unknown => json!({"state":"unknown"}),
    }
}

fn applied_multiplicity_json(factor: &AppliedMultiplicity) -> Json {
    json!({"identity":{"caller_semantic_ir_digest":digest_json(&factor.identity.caller_semantic_ir_digest),"control_node":factor.identity.control_node.raw(),"kind":match factor.identity.kind {af_computation::ControlMultiplicityKind::Iterate=>"iterate",af_computation::ControlMultiplicityKind::Retry=>"retry"}},"logical_repetitions":bounds_json(&factor.logical_repetitions),"physical_attempts":factor.physical_attempts.as_ref().map(bounds_json)})
}

fn control_path_step_json(step: &ControlPathStep) -> Json {
    let (kind, detail) = match &step.kind {
        ControlPathKind::SequenceChild => ("sequence_child", json!({})),
        ControlPathKind::SelectCase {
            case_ordinal,
            selector,
            selector_dependencies,
            exact_case,
        } => (
            "select_case",
            json!({"case_ordinal":case_ordinal,"selector":selector.map(ExpressionId::raw),"selector_dependencies":selector_dependencies.iter().map(|id|id.raw()).collect::<Vec<_>>(),"exact_case":exact_case}),
        ),
        ControlPathKind::ParallelBranch { branch_ordinal } => {
            ("parallel_branch", json!({"branch_ordinal":branch_ordinal}))
        }
        ControlPathKind::RaceBranch { branch_ordinal } => {
            ("race_branch", json!({"branch_ordinal":branch_ordinal}))
        }
        ControlPathKind::IterateBody => ("iterate_body", json!({})),
        ControlPathKind::RetryChild { max_attempts } => {
            ("retry_child", json!({"max_attempts":max_attempts}))
        }
        ControlPathKind::TimeoutChild => ("timeout_child", json!({})),
        ControlPathKind::HandleProtected => ("handle_protected", json!({})),
        ControlPathKind::HandleClause { ordinal } => ("handle_clause", json!({"ordinal":ordinal})),
        ControlPathKind::ScopeChild => ("scope_child", json!({})),
        ControlPathKind::CompensationScopeChild => ("compensation_scope_child", json!({})),
    };
    json!({"parent_node":step.parent_node.raw(),"child_node":step.child_node.raw(),"child_index":step.child_index,"kind":kind,"detail":detail,"reachability":match step.reachability {ControlPathReachability::Possible=>"possible",ControlPathReachability::ExactSelected=>"exact_selected",ControlPathReachability::ProvenUnreachable=>"proven_unreachable"}})
}

pub(crate) fn root_export_report_projection(
    report: &af_computation::RootExportProofReport,
    checked_capability_issued: bool,
    _inputs: &crate::pipeline::ReconstructedStaticInputs,
) -> Json {
    json!({
        "stage":"root_export_checker",
        "assertion_identity":root_export_assertion_identity_json(&report.assertion_identity, 0),
        "preflight":report.preflight.iter().map(dimension_proof_json).collect::<Vec<_>>(),
        "dimensions":report.dimensions.iter().map(|item|json!({
            "dimension":item.dimension.as_str(),
            "proof":dimension_proof_json(&item.proof),
            "source_facts":item.source_facts.iter().map(|source|json!({"semantic_ir_digest":digest_json(&source.semantic_ir_digest),"computation":source.computation.map(ComputationId::raw),"contract_digest":source.contract_digest.as_ref().map(digest_json),"source_ordinal":source.source_ordinal})).collect::<Vec<_>>(),
            "mapped_coordinates":item.mapped_coordinates.iter().map(root_export_witness_json).collect::<Vec<_>>(),
            "mapped_coordinate_count":item.mapped_coordinate_count,
        })).collect::<Vec<_>>(),
        "execution_children":report.execution_children.iter().map(|item|json!({"source_node":item.source_node.raw(),"source_contract_digest":digest_json(&item.source_contract_digest),"feasibility":item.feasibility.as_str(),"security":item.security.as_str()})).collect::<Vec<_>>(),
        "static_compatibility":report.static_compatibility.as_str(),
        "checked_static_compatibility_capability_issued":checked_capability_issued,
        "body_conformance":report.body_conformance.as_str(),
        "implementation_conformance":report.implementation_conformance.as_str(),
        "scope_authority_attenuation":report.scope_authority_attenuation.as_str(),
        "scope_budget_attenuation":report.scope_budget_attenuation.as_str(),
        "runtime_authority":"not_established",
        "runtime_authority_evidence":report.runtime_authority_evidence.as_str(),
        "publisher_authentication":"not_established",
        "unresolved_obligations":report.unresolved_obligations,
    })
}

fn dimension_proof_json(proof: &af_contract::DimensionProof) -> Json {
    json!({"dimension":proof.dimension,"verdict":proof.verdict.as_str(),"rule_id":proof.rule_id,"reference":proof.reference,"reason":proof.reason,"prerequisites":proof.prerequisites})
}

fn root_export_assertion_identity_json(
    identity: &RootExportAssertionIdentity,
    depth: usize,
) -> Json {
    root_export_assertion_json(identity.assertion(), depth)
}

fn root_export_assertion_json(assertion: &RootExportAssertion, depth: usize) -> Json {
    json!({
        "semantic_ir_digest":digest_json(&assertion.exact_semantic_ir_digest),
        "exported_component_contract_digest":digest_json(&assertion.exact_exported_component_contract_digest),
        "compiler_semantics":{
            "analysis_semantics_version":assertion.compiler_semantics_identity.analysis_semantics_version,
            "registry_content_bytes":assertion.compiler_semantics_identity.registry_content.len(),
            "registry_content_sha256":hex::encode(sha2::Sha256::digest(&assertion.compiler_semantics_identity.registry_content)),
            "rule_registry_content_bytes":assertion.compiler_semantics_identity.rule_registry_content.len(),
            "rule_registry_content_sha256":hex::encode(sha2::Sha256::digest(&assertion.compiler_semantics_identity.rule_registry_content)),
            "specifications":assertion.compiler_semantics_identity.specifications.iter().map(|item|json!({"specification_id":item.specification_id,"logical_path":item.logical_path,"content_sha256":item.content_sha256,"content_bytes":item.content.len()})).collect::<Vec<_>>(),
            "rules":assertion.compiler_semantics_identity.rules.iter().map(|item|json!({"rule_id":item.rule_id,"normative_source":item.normative_source,"source_anchor":item.source_anchor,"short_statement":item.short_statement})).collect::<Vec<_>>(),
        },
        "proof_context_manifest":{
            "compiled_module_count":assertion.proof_context_manifest.compiled_modules.len(),
            "type_schema_count":assertion.proof_context_manifest.type_schemas.len(),
            "effect_catalogs":assertion.proof_context_manifest.effect_catalogs.catalogs().map(|catalog|json!({"specification_id":catalog.specification_id(),"operations":catalog.operations().collect::<Vec<_>>()})).collect::<Vec<_>>(),
            "resource_type_count":assertion.proof_context_manifest.resource_types.len(),
            "flow_label_count":assertion.proof_context_manifest.flow_catalogs.labels.len(),
            "flow_sink_count":assertion.proof_context_manifest.flow_catalogs.sinks.len(),
            "usage_meter_count":assertion.proof_context_manifest.usage_meters.len(),
            "recovery_operation_count":assertion.proof_context_manifest.recovery_operations.reconciliation_operations().count(),
            "relation_snapshots":assertion.proof_context_manifest.relation_snapshots.iter().map(|item|json!({"schema":item.pin.schema,"revision":item.pin.revision,"canonical_content_checksum":{"algorithm":item.pin.canonical_content_checksum.algorithm,"hex":item.pin.canonical_content_checksum.hex},"content_bytes":item.canonical_content.len()})).collect::<Vec<_>>(),
            "resolver_dependencies":{"component_contract_digests":assertion.proof_context_manifest.resolver_dependencies.component_contract_digests.iter().map(digest_json).collect::<Vec<_>>(),"subflow_targets":assertion.proof_context_manifest.resolver_dependencies.subflow_targets.iter().map(|item|json!({"semantic_ir_digest":digest_json(&item.semantic_ir_digest),"direct_dependencies":item.direct_dependencies.iter().map(digest_json).collect::<Vec<_>>(),"exported_contract_digest":item.exported_contract_digest.as_ref().map(digest_json)})).collect::<Vec<_>>()},
        },
        "input_mappings":assertion.input_mappings.iter().map(root_input_mapping_json).collect::<Vec<_>>(),
        "output_mappings":assertion.output_mappings.iter().map(root_output_mapping_json).collect::<Vec<_>>(),
        "resource_mappings":assertion.resource_mappings.iter().map(root_resource_mapping_json).collect::<Vec<_>>(),
        "effect_mappings":assertion.effect_mappings.iter().map(root_effect_mapping_json).collect::<Vec<_>>(),
        "flow_mappings":assertion.flow_mappings.iter().map(root_flow_mapping_json).collect::<Vec<_>>(),
        "recovery_mappings":assertion.recovery_mappings.iter().map(root_recovery_mapping_json).collect::<Vec<_>>(),
        "execution_mappings":assertion.execution_mappings.iter().map(root_execution_mapping_json).collect::<Vec<_>>(),
        "subflow_dependencies":assertion.subflow_dependencies.iter().map(|item|json!({
            "call_site":item.call_site.raw(),
            "target_semantic_ir_digest":digest_json(&item.target_semantic_ir_digest),
            "target_exported_contract_digest":item.target_exported_contract_digest.as_ref().map(digest_json),
            "target_root_assertion_identity":item.target_root_assertion_identity.as_ref().map(|identity| {
                if depth >= MAX_ASSERTION_GRAPH_NODES { json!({"projection_depth_limit":true}) }
                else { root_export_assertion_identity_json(identity, depth + 1) }
            }),
            "transitive_targets":item.transitive_targets.iter().map(|target|json!({"semantic_ir_digest":digest_json(&target.semantic_ir_digest),"exported_contract_digest":target.exported_contract_digest.as_ref().map(digest_json)})).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    })
}

fn root_input_mapping_json(item: &RootInputPortBinding) -> Json {
    json!({"export_input_port":{"contract_digest":digest_json(&item.export_input_port.exported_contract_digest),"port_no":item.export_input_port.port_no},"run_parameter_id":item.run_parameter_id.raw(),"root_value_slot":item.root_value_slot.raw(),"root_structural_type":item.root_type.canonical_string(),"export_type":schema_ref_json(&item.export_type),"use_sites":item.use_sites.iter().map(|site|json!({"computation":site.computation.raw(),"source_contract_digest":digest_json(&site.source_contract_digest),"source_component_port":site.source_component_port,"input_expression":site.input_expression.raw(),"exact_type":schema_ref_json(&site.exact_type),"dependency_slots":site.dependency_slots.iter().map(|id|id.raw()).collect::<Vec<_>>()})).collect::<Vec<_>>()})
}

fn root_output_mapping_json(item: &RootOutputPortBinding) -> Json {
    json!({"export_output_port":{"contract_digest":digest_json(&item.export_output_port.exported_contract_digest),"port_no":item.export_output_port.port_no},"workflow_output_name":item.workflow_output_name,"workflow_output_expression":item.workflow_output_expression.raw(),"root_structural_type":item.root_result_type.canonical_string(),"producer_evidence":{"exact_result_type":schema_ref_json(&item.producer_evidence.exact_result_type),"dependency_slots":item.producer_evidence.dependency_slots.iter().map(|id|id.raw()).collect::<Vec<_>>()}})
}

fn root_resource_mapping_json(item: &RootResourcePortBinding) -> Json {
    json!({"export_resource_port":{"contract_digest":digest_json(&item.export_resource_port.exported_contract_digest),"port_no":item.export_resource_port.port_no},"root_resource_slot":item.root_resource_slot.raw(),"root_resource_type":schema_ref_json(&item.root_resource_type),"export_resource_type":resource_type_ref_json(&item.export_resource_type),"source_coordinates":item.source_coordinates.iter().map(|source|json!({"source_node":source.source_node.raw(),"source_contract_digest":digest_json(&source.source_contract_digest),"component_resource_port":source.component_resource_port,"resource_slot":source.resource_slot.raw(),"alias_component_ports":source.alias_component_ports})).collect::<Vec<_>>()})
}

fn root_effect_mapping_json(item: &RootEffectCoordinateBinding) -> Json {
    json!({"source":{"caller_semantic_ir_digest":digest_json(&item.source.caller_semantic_ir_digest),"source_node":item.source.source_node.raw(),"source_contract_digest":digest_json(&item.source.source_contract_digest),"source_entry_ordinal":item.source.source_entry_ordinal,"atom":effect_atom_json(&item.source.atom),"guard":effect_guard_json(&item.source.guard)},"root_resource_slot":item.root_resource_slot.map(ResourceSlotId::raw),"export_atom":effect_atom_json(&item.export_atom),"export_guard":effect_guard_json(&item.export_guard)})
}

fn root_flow_mapping_json(item: &RootFlowCoordinateBinding) -> Json {
    json!({"source_node":item.source_node.raw(),"source_contract_digest":digest_json(&item.source_contract_digest),"flow_mapping_ordinal":item.flow_mapping_ordinal,"source":item.source.as_ref().map(flow_source_json),"sink":item.sink.as_ref().map(flow_constraint_json),"source_binding":item.source_binding.as_ref().map(flow_source_binding_json),"sink_binding":item.sink_binding.as_ref().map(flow_sink_binding_json),"export_source":item.export_source.as_ref().map(flow_source_json),"export_sink":item.export_sink.as_ref().map(flow_constraint_json),"source_effect_partition":item.source_effect_partition.as_ref().map(effect_flow_partition_json),"export_effect_partition":item.export_effect_partition.as_ref().map(effect_flow_partition_json)})
}

fn root_recovery_mapping_json(item: &RootRecoveryCoordinateBinding) -> Json {
    json!({"source":{"caller_semantic_ir_digest":digest_json(&item.source.caller_semantic_ir_digest),"source_node":item.source.source_node.raw(),"source_contract_digest":digest_json(&item.source.source_contract_digest),"forward_entry_ordinal":item.source.forward_entry_ordinal,"action":recovery_action_identity_json(&item.source.action)},"export_action_identity":recovery_action_identity_json(&item.export_action_identity),"exact_usage_binding":recovery_usage_binding_json(&item.exact_usage_binding)})
}

fn root_execution_mapping_json(item: &RootExecutionCoordinateBinding) -> Json {
    json!({"source_node":item.source_node.raw(),"source_contract_digest":digest_json(&item.source_contract_digest),"exact_execution_requirements":execution_requirements_json(&item.exact_execution_requirements)})
}

fn root_export_witness_json(witness: &af_computation::RootExportCoordinateWitness) -> Json {
    match witness {
        af_computation::RootExportCoordinateWitness::Input(item) => {
            json!({"kind":"input","mapping":root_input_mapping_json(item)})
        }
        af_computation::RootExportCoordinateWitness::Output(item) => {
            json!({"kind":"output","mapping":root_output_mapping_json(item)})
        }
        af_computation::RootExportCoordinateWitness::Resource(item) => {
            json!({"kind":"resource","mapping":root_resource_mapping_json(item)})
        }
        af_computation::RootExportCoordinateWitness::Effect(item) => {
            json!({"kind":"effect","mapping":root_effect_mapping_json(item)})
        }
        af_computation::RootExportCoordinateWitness::Flow(item) => {
            json!({"kind":"flow","mapping":root_flow_mapping_json(item)})
        }
        af_computation::RootExportCoordinateWitness::Recovery(item) => {
            json!({"kind":"recovery","mapping":root_recovery_mapping_json(item)})
        }
        af_computation::RootExportCoordinateWitness::Execution(item) => {
            json!({"kind":"execution","mapping":root_execution_mapping_json(item)})
        }
    }
}

fn flow_source_binding_json(binding: &FlowSourceBinding) -> Json {
    match binding {
        FlowSourceBinding::InputExpression(id) => json!({"input_expression":id.raw()}),
        FlowSourceBinding::ResourceSlot(id) => json!({"resource_slot":id.raw()}),
        FlowSourceBinding::UnknownExternal => json!({"unknown_external":null}),
    }
}

fn flow_sink_binding_json(binding: &FlowSinkBinding) -> Json {
    match binding {
        FlowSinkBinding::OutputValue(id) => json!({"output_value":id.raw()}),
        FlowSinkBinding::EffectOccurrences(indices) => json!({"effect_occurrences":indices}),
    }
}

fn compensation_scope_json(scope: &af_computation::CompensationScopeSummary) -> Json {
    let potential_actions = match scope.potential_actions().value() {
        ControlDimensionValue::Complete(actions) => {
            json!({"state":"complete","actions":actions.iter().map(|action|json!({"source_node":action.source_node().raw(),"identity":recovery_action_identity_json(action.identity()),"forward_occurrence":scoped_effect_occurrence_json(action.forward_occurrence()),"action_effect_occurrence":scoped_effect_occurrence_json(action.action_effect_occurrence()),"local_bound":local_action_bound_json(action.local_bound()),"potential_effect":effect_entry_json(action.potential_effect()),"potential_recovery_usage":usage_envelope_json(action.potential_recovery_usage()),"receipt_evidence":action.receipt_evidence().as_str()})).collect::<Vec<_>>()})
        }
        ControlDimensionValue::Partial {
            known_contribution,
            unresolved_sources,
        } => {
            json!({"state":"partial","actions":known_contribution.as_ref().map(|items|items.iter().map(|action|json!({"source_node":action.source_node().raw(),"identity":recovery_action_identity_json(action.identity()),"potential_effect":effect_entry_json(action.potential_effect()),"potential_recovery_usage":usage_envelope_json(action.potential_recovery_usage()),"receipt_evidence":action.receipt_evidence().as_str()})).collect::<Vec<_>>()),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::Unknown {
            known_contributions,
            unresolved_sources,
        } => {
            json!({"state":"unknown","known_contributions":known_contributions.iter().map(|item|json!({"source_node":item.source_node().raw(),"actions":item.value().iter().map(|action|json!({"source_node":action.source_node().raw(),"identity":recovery_action_identity_json(action.identity()),"potential_effect":effect_entry_json(action.potential_effect()),"potential_recovery_usage":usage_envelope_json(action.potential_recovery_usage()),"receipt_evidence":action.receipt_evidence().as_str()})).collect::<Vec<_>>()})).collect::<Vec<_>>(),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::PerChild {
            contributions,
            unresolved_sources,
        } => {
            json!({"state":"per_child","contributions":contributions.iter().map(|item|json!({"source_node":item.source_node().raw(),"actions":item.value().iter().map(|action|json!({"source_node":action.source_node().raw(),"identity":recovery_action_identity_json(action.identity()),"potential_effect":effect_entry_json(action.potential_effect()),"potential_recovery_usage":usage_envelope_json(action.potential_recovery_usage()),"receipt_evidence":action.receipt_evidence().as_str()})).collect::<Vec<_>>()})).collect::<Vec<_>>(),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::NotApplicable => json!({"state":"not_applicable"}),
    };
    let local = match scope.local_recovery_actions().value() {
        ControlDimensionValue::Complete(actions) => {
            json!({"state":"complete","actions":actions.iter().map(|action|json!({"source":{"caller_semantic_ir_digest":digest_json(&action.source.caller_semantic_ir_digest),"source_node":action.source.source_node.raw(),"source_contract_digest":digest_json(&action.source.source_contract_digest),"forward_entry_ordinal":action.source.forward_entry_ordinal,"action":recovery_action_identity_json(&action.source.action)},"forward_seed_upper":ceiling_json(action.forward_seed_upper),"owner_scope":action.owner_scope.raw(),"action_kind":action.action_kind.as_str(),"local_bound":local_action_bound_json(&action.local_bound),"forward_occurrence":scoped_effect_occurrence_json(&action.forward_occurrence),"action_effect_occurrence":scoped_effect_occurrence_json(&action.action_effect_occurrence),"recovery_usage_binding":recovery_usage_binding_json(&action.recovery_usage_binding),"recovery_usage":usage_envelope_json(&action.recovery_usage),"guard_compatibility":action.guard_compatibility.as_ref().map(action_guard_compatibility_json),"eligibility_evidence":action.eligibility_evidence.as_str()})).collect::<Vec<_>>()})
        }
        ControlDimensionValue::Partial {
            known_contribution,
            unresolved_sources,
        } => {
            json!({"state":"partial","actions":known_contribution.as_ref().map(|actions|actions.iter().map(|action|json!({"source_node":action.source.source_node.raw(),"action":recovery_action_identity_json(&action.source.action),"local_bound":local_action_bound_json(&action.local_bound),"recovery_usage":usage_envelope_json(&action.recovery_usage),"eligibility_evidence":action.eligibility_evidence.as_str()})).collect::<Vec<_>>()),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::Unknown {
            known_contributions,
            unresolved_sources,
        } => {
            json!({"state":"unknown","known_contributions":known_contributions.iter().map(|item|json!({"source_node":item.source_node().raw(),"actions":item.value().iter().map(|action|json!({"source_node":action.source.source_node.raw(),"action":recovery_action_identity_json(&action.source.action),"local_bound":local_action_bound_json(&action.local_bound),"recovery_usage":usage_envelope_json(&action.recovery_usage),"eligibility_evidence":action.eligibility_evidence.as_str()})).collect::<Vec<_>>()})).collect::<Vec<_>>(),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::PerChild {
            contributions,
            unresolved_sources,
        } => {
            json!({"state":"per_child","contributions":contributions.iter().map(|item|json!({"source_node":item.source_node().raw(),"actions":item.value().iter().map(|action|json!({"source_node":action.source.source_node.raw(),"action":recovery_action_identity_json(&action.source.action),"local_bound":local_action_bound_json(&action.local_bound),"recovery_usage":usage_envelope_json(&action.recovery_usage),"eligibility_evidence":action.eligibility_evidence.as_str()})).collect::<Vec<_>>()})).collect::<Vec<_>>(),"unresolved_sources":unresolved_sources.iter().map(|id|id.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::NotApplicable => json!({"state":"not_applicable"}),
    };
    json!({"potential_actions":potential_actions,"local_recovery_actions":local,"compensation_order":scope.compensation_order().as_str(),"recovery_blocked_outcome":scope.recovery_blocked_outcome().as_str(),"forward_effects_retained":scope.forward_effects_retained(),"forward_outcome_retained":scope.forward_outcome_retained()})
}
