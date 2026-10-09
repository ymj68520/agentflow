//! Test-only abstract-fixture materializer. This is not a wire ABI, runtime
//! model, or alternate analyzer: it builds formal ComponentContracts and
//! Semantic IR, then calls the production validators/resolver/analyzer.

use std::collections::{BTreeMap, BTreeSet};

use af_computation::{
    ControlDimensionValue, ControlOperator, DerivedControlContractSummary,
    DerivedWorkflowContractSummary, HandleMatcherSummary, IterateMode, LeafVerdict,
    StaticComputationResolutionSnapshotBuilder, derive_control_contract,
    derive_workflow_contract_summary, resolve_computation_leaf,
};
use af_contract::{
    AuthorityExecutionRule, ComponentContract, ComponentIdentity, ComponentInputPort,
    ComponentOutputPort, EffectQualifierProfile, EffectResourceCompatibilityCatalog,
    InformationFlowExecutionCatalog, RecoveryActionBinding, RecoveryActionIdentity,
    RecoveryActionKind, RecoveryUsageBinding, RecoveryUsageCoverageCatalog,
    RecoveryUsageCoverageRule, RelationCatalogSnapshot, RelationSnapshots, SandboxFloor,
    StaticContractProofContext, SupportedEffectResourcePair, TaintExecutionRule, TrustDomainPolicy,
    analyze_contract, check_contract, component_contract_digest,
};
use af_effect::{
    CapabilityCatalog, EffectAtom, EffectCatalogUniverse, EffectEntry, EffectGuard,
    EffectGuardInputEquals, EffectIdempotencyQualifier, EffectOpenRow, EffectOperationRef,
    EffectQualifier, EffectResidual, EffectRow, EffectSignature, OccurrenceBounds,
    OccurrenceCeiling, ResourceSelector, ScopeBinding,
};
use af_execution::{
    ExecutionRequirements, ExecutionUniverses, RuntimeProfileRef, RuntimeProfileRequirement,
    SandboxLevel, SandboxRequirement,
};
use af_failure::{FailureRow, FailureTypeRef};
use af_flow::{
    FlowCatalogs, FlowSourceRef, InformationFlowContract, InstructionAuthority, SourceCategory,
    StaticInformationSummary, TaintRef,
};
use af_idl::{
    CanonicalBody, CanonicalTypeExpr, CanonicalTypeSchema, CompiledModuleSet, Primitive,
    compile_module_set,
};
use af_ir::model::ComputationOutputBinding;
use af_ir::{
    AwaitBody, CallSubflowBody, ComputationBody, ComputationDef, ComputationId, ContractRefEntry,
    ContractRefId, ContractRefKind, FailureMatch, ForeachBody, HandleBody, HandleClause,
    HandleMatch, InputBinding, InvokeBody, IterateBody, LoopCarryBinding, ParallelBody,
    ParallelBranch, ParallelMode, ParallelOutputBinding, RaceBody, RaceBranch, ResourceUseBinding,
    RetryBody, RunParameterDecl, ScopeBody, SelectBody, SelectCase, SemanticIrDocument,
    SequenceBody, SubflowIrUniverse, TimeoutBody, WhileBody,
};
use af_recovery::{
    CompensationRequirement, EffectRecoveryClause, IdempotencySemantics, KeyStabilityRequirement,
    ReconciliationRequirement, RecoveryContract, RecoveryOperationUniverse, ReplaySemantics,
    UnknownEffectHandling,
};
use af_resource::{
    ResourceRequirement, ResourceRequirementRow, ResourceTypeDescriptor, ResourceTypeRef,
    ResourceTypeUniverse,
};
use af_semantic::{
    DigestRefValue, ExpressionBody, ExpressionDef, PrimitiveConstant, ResourceSlotId,
    SemanticPrimitiveType, SemanticTypeNode, SemanticTypeNodeBody, TypeSchemaRef,
    TypeSchemaUniverse, ValueSlotId,
};
use af_usage::{
    StaticUsageBound, StaticUsageEnvelope, UsageLimit, UsageMeterRef, UsageMeterUniverse,
    UsagePhase, UsageSignature, UsageUnitRef, UsageUpperBound,
};
use serde_json::{Value, json};

const LABEL_CATALOG: &str = "agentflow.test.labels@1";
const METER_CATALOG: &str = "agentflow.meters@1";
const RUNTIME_CATALOG: &str = "agentflow.test.runtimes@1";

fn fixture_u32(value: &Value, label: &str) -> u32 {
    u32::try_from(
        value
            .as_u64()
            .unwrap_or_else(|| panic!("fixture {label} must be an unsigned integer")),
    )
    .unwrap_or_else(|_| panic!("fixture {label} exceeds u32"))
}

fn fixture_index(index: usize) -> u32 {
    u32::try_from(index).expect("fixture table length fits u32")
}

fn guard_type_schema_name(primitive: &str) -> String {
    let mut chars = primitive.chars();
    let first = chars
        .next()
        .unwrap_or_else(|| panic!("empty fixture guard primitive"))
        .to_uppercase()
        .collect::<String>();
    format!("Guard{first}{}", chars.as_str())
}

fn scope_resource_ids<'a>(
    aliases: impl Iterator<Item = &'a Value>,
    slots: &BTreeMap<String, ResourceSlotId>,
) -> Result<Vec<ResourceSlotId>, String> {
    aliases
        .map(|alias| {
            slots
                .get(alias.as_str().unwrap())
                .copied()
                .ok_or_else(|| format!("unmaterialized Scope ResourceSlot {alias}"))
        })
        .collect()
}

/// Test-only, deterministic semantic projection. It has no wire or runtime
/// representation and is built exclusively from a production summary.
#[derive(Debug, Clone, PartialEq)]
pub struct ControlConformanceProjection {
    pub semantic_fields: Value,
}

impl ControlConformanceProjection {
    fn into_value(self) -> Value {
        self.semantic_fields
    }
}

pub struct FixtureContext {
    pub modules: CompiledModuleSet,
    pub schemas: TypeSchemaUniverse,
    pub effects: EffectCatalogUniverse,
    pub resources: ResourceTypeUniverse,
    pub flow: FlowCatalogs,
    pub usage: UsageMeterUniverse,
    pub recovery: RecoveryOperationUniverse,
    pub execution: ExecutionUniverses,
    pub relations: RelationSnapshots,
    resource_type: ResourceTypeRef,
}

impl FixtureContext {
    pub fn for_case(case: &Value) -> Self {
        let modules = modules();
        let mut schemas = TypeSchemaUniverse::new();
        let mut resources = ResourceTypeUniverse::new();
        for module in modules.modules() {
            schemas.insert_compiled(module).unwrap();
            resources.insert_compiled(module).unwrap();
        }
        let children = case["input"]["children"]
            .as_array()
            .cloned()
            .or_else(|| {
                case["input"]
                    .get("contract")
                    .cloned()
                    .map(|contract| vec![contract])
            })
            .unwrap_or_default();
        let mut guard_primitives = BTreeSet::new();
        for child in &children {
            for input_type in child["input_types"].as_array().into_iter().flatten() {
                if let Some(primitive) = input_type["primitive"].as_str() {
                    guard_primitives.insert(primitive.to_owned());
                }
            }
        }
        for name in guard_primitives {
            let primitive = Primitive::from_keyword(&name)
                .unwrap_or_else(|| panic!("unsupported fixture guard primitive {name}"));
            let schema_name = guard_type_schema_name(&name);
            schemas
                .insert_type(&CanonicalTypeSchema {
                    schema: "agentflow.test.guard-types@1".into(),
                    name: schema_name,
                    body: CanonicalBody::Alias(CanonicalTypeExpr::Primitive(primitive)),
                })
                .unwrap();
        }
        let resource_schema = schemas
            .get("agentflow.semantic.core@1/TypeSchemaRef")
            .unwrap()
            .reference();
        resources
            .recognize(ResourceTypeDescriptor {
                specification_id: resource_schema.specification_id.clone(),
                type_name: resource_schema.type_name.clone(),
                schema_hex: resource_schema.schema_digest.hex.clone(),
                recognized_attributes: BTreeSet::new(),
            })
            .unwrap();
        let resource_type = ResourceTypeRef {
            schema: resource_schema,
        };

        let mut effects = EffectCatalogUniverse::new();
        let catalog = CapabilityCatalog::new("agentflow.effects@1", ["write"]).unwrap();
        effects.insert(&catalog).unwrap();
        let mut recovery = RecoveryOperationUniverse::new();
        recovery.effects.insert(&catalog).unwrap();

        let mut flow = FlowCatalogs::new();
        let mut taint_names = BTreeSet::new();
        for child in &children {
            if let Some(taints) = child["flow"]["taints"].as_array() {
                for taint in taints.iter().filter_map(Value::as_str) {
                    flow.labels.recognize_taint(LABEL_CATALOG, taint);
                    taint_names.insert(taint.to_owned());
                }
            }
        }
        flow.sinks
            .recognize_operation("agentflow.effects@1", "write", false);

        let mut usage = UsageMeterUniverse::new();
        usage
            .recognize_additive(
                &UsageMeterRef {
                    catalog: METER_CATALOG.into(),
                    name: "api_calls".into(),
                },
                &UsageUnitRef::new("agentflow.units@1", "calls").unwrap(),
            )
            .unwrap();

        let mut execution = ExecutionUniverses::default();
        execution
            .runtime_profiles
            .recognize(
                RuntimeProfileRef {
                    catalog: RUNTIME_CATALOG.into(),
                    name: "python".into(),
                },
                "language",
                Vec::<String>::new(),
            )
            .unwrap();

        let operation = EffectOperationRef {
            catalog: "agentflow.effects@1".into(),
            name: "write".into(),
        };
        let supported_pair = SupportedEffectResourcePair {
            operation: operation.clone(),
            resource_type: resource_type.clone(),
            selector_keys: vec![],
            allowed_qualifier_profiles: vec![
                EffectQualifierProfile { qualifiers: vec![] },
                EffectQualifierProfile {
                    qualifiers: vec![EffectQualifier::Idempotency(
                        EffectIdempotencyQualifier::Natural,
                    )],
                },
                EffectQualifierProfile {
                    qualifiers: vec![EffectQualifier::Idempotency(
                        EffectIdempotencyQualifier::None,
                    )],
                },
            ],
        };
        let effect_resource = Some(
            RelationCatalogSnapshot::new(
                "fixture-effect-resource-r1",
                EffectResourceCompatibilityCatalog {
                    complete_operations: vec![operation],
                    supported_pairs: vec![supported_pair],
                },
                &modules,
            )
            .unwrap(),
        );
        let flow_execution = Some(
            RelationCatalogSnapshot::new(
                "fixture-flow-execution-r1",
                InformationFlowExecutionCatalog {
                    taint_rules: taint_names
                        .into_iter()
                        .map(|name| TaintExecutionRule {
                            taint: TaintRef {
                                catalog: LABEL_CATALOG.into(),
                                name,
                            },
                            trust_domains: TrustDomainPolicy::NoAdditionalTrustRequirement,
                            sandbox: SandboxFloor::NoAdditionalMinimum,
                        })
                        .collect(),
                    authority_rules: vec![
                        InstructionAuthority::None,
                        InstructionAuthority::ExternalData,
                        InstructionAuthority::DelegatedAgent,
                        InstructionAuthority::User,
                        InstructionAuthority::Workflow,
                        InstructionAuthority::Platform,
                    ]
                    .into_iter()
                    .map(|authority| AuthorityExecutionRule {
                        authority,
                        trust_domains: TrustDomainPolicy::NoAdditionalTrustRequirement,
                        sandbox: SandboxFloor::NoAdditionalMinimum,
                    })
                    .collect(),
                    ..InformationFlowExecutionCatalog::default()
                },
                &modules,
            )
            .unwrap(),
        );
        let mut recovery_rules = Vec::<RecoveryUsageCoverageRule>::new();
        for item in children
            .iter()
            .flat_map(|child| child["recovery_actions"].as_array().into_iter().flatten())
        {
            let identity =
                recovery_action_identity(item).expect("well-formed test recovery action");
            let complete_meters = item["coverage_meters"]
                .as_array()
                .expect("recovery fixture declares exact complete meters")
                .iter()
                .map(|meter| UsageMeterRef {
                    catalog: METER_CATALOG.into(),
                    name: meter.as_str().unwrap().into(),
                })
                .collect::<Vec<_>>();
            if let Some(existing) = recovery_rules.iter().find(|rule| rule.action == identity) {
                assert_eq!(
                    existing.complete_meters, complete_meters,
                    "one exact K identity cannot have two fixture meter profiles"
                );
            } else {
                recovery_rules.push(RecoveryUsageCoverageRule {
                    action: identity,
                    complete_meters,
                });
            }
        }
        let recovery_usage = (!recovery_rules.is_empty()).then(|| {
            RelationCatalogSnapshot::new(
                "fixture-recovery-usage-r1",
                RecoveryUsageCoverageCatalog {
                    rules: recovery_rules,
                },
                &modules,
            )
            .expect("well-formed test RecoveryUsage coverage snapshot")
        });
        let relations = RelationSnapshots {
            effect_resource,
            recovery_usage,
            flow_execution,
        };
        Self {
            modules,
            schemas,
            effects,
            resources,
            flow,
            usage,
            recovery,
            execution,
            relations,
            resource_type,
        }
    }

    pub fn view(&self) -> StaticContractProofContext<'_> {
        StaticContractProofContext {
            modules: &self.modules,
            schema_universe: &self.schemas,
            effects: &self.effects,
            resources: &self.resources,
            flow: &self.flow,
            usage_meters: &self.usage,
            recovery: &self.recovery,
            execution: &self.execution,
            relations: &self.relations,
        }
    }

    pub fn resource_type(&self) -> &ResourceTypeRef {
        &self.resource_type
    }
}

/// Materializes one ordinary structured-control case and runs all production
/// stages. Expected JSON is never used to construct the input.
pub fn analyze_compose_case(case: &Value) -> Result<Value, String> {
    let input = &case["input"];
    let children = input["children"]
        .as_array()
        .ok_or_else(|| "compose input has no children array".to_string())?;
    let env = FixtureContext::for_case(case);
    let view = env.view();
    let mut raws = BTreeMap::<String, ComponentContract>::new();
    let mut unresolved_aliases = BTreeSet::<String>::new();
    let mut pin_by_alias = BTreeMap::<String, DigestRefValue>::new();
    for child in children {
        let alias = child["contract_digest"]
            .as_str()
            .ok_or_else(|| "child has no symbolic contract_digest".to_string())?
            .to_owned();
        let proof_source = child["proof_source"]
            .as_str()
            .unwrap_or("checked_component");
        if proof_source == "unresolved_component_pin" || child["effects"].is_null() {
            unresolved_aliases.insert(alias.clone());
            raws.entry(alias.clone())
                .or_insert_with(|| empty_contract(&alias));
            let hex = alias.rsplit(':').next().unwrap_or_default().to_owned();
            if hex.len() != 64 {
                return Err(format!(
                    "symbolic unresolved pin is not 64 hex chars: {alias}"
                ));
            }
            pin_by_alias.insert(
                alias,
                DigestRefValue {
                    algorithm: "sha256".into(),
                    domain: "component-contract".into(),
                    hex,
                },
            );
            continue;
        }
        if child["effects"]["row"] == "open" || child["effects"]["row"] == "unknown" {
            return Err(format!(
                "{alias}: non-closed Effect rows do not mint checked component contracts"
            ));
        }
        let raw = component_contract(child, &env)?;
        raws.insert(alias.clone(), raw);
    }

    let mut checked_by_pin = BTreeMap::<String, usize>::new();
    let mut checked_values = Vec::new();
    let mut pin_aliases = BTreeMap::<String, String>::new();
    for (alias, raw) in &raws {
        if unresolved_aliases.contains(alias) {
            continue;
        }
        let checked = check_contract(raw, &view).map_err(|report| {
            let details = report
                .dimensions
                .iter()
                .filter(|item| item.verdict != af_contract::ProofVerdict::Proven)
                .map(|item| {
                    format!(
                        "{}={}:{}",
                        item.dimension,
                        item.verdict.as_str(),
                        item.reason
                    )
                })
                .collect::<Vec<_>>()
                .join("; ");
            format!("contract {alias} did not validate: {details}")
        })?;
        let digest = component_contract_digest(&checked, &view).map_err(|e| e.to_string())?;
        let pin = DigestRefValue {
            algorithm: "sha256".into(),
            domain: digest.domain().as_str().into(),
            hex: digest.hex().into(),
        };
        pin_aliases.insert(pin.hex.clone(), alias.clone());
        pin_by_alias.insert(alias.clone(), pin.clone());
        checked_by_pin.insert(pin.hex.clone(), checked_values.len());
        checked_values.push(checked);
    }
    let mut subflows = SubflowIrUniverse::new();
    let mut subflow_targets = BTreeMap::<u32, DigestRefValue>::new();
    let mut subflow_exports = Vec::<(DigestRefValue, usize)>::new();
    for child in children {
        if child["proof_source"] == "subflow_body_evidence" {
            let alias = child["contract_digest"].as_str().unwrap();
            let pin = pin_by_alias
                .get(alias)
                .ok_or("subflow export contract pin missing")?;
            let raw = raws.get(alias).unwrap();
            let target_source = target_subflow_document(pin.clone(), raw, &env);
            let target_ir = target_source
                .check(&env.schemas)
                .map_err(|e| format!("subflow target IR validation: {e}"))?;
            let target_digest = subflows
                .insert(&target_ir, &env.modules)
                .map_err(|e| format!("subflow target digest: {e}"))?;
            let node = fixture_u32(&child["node"], "node id");
            subflow_targets.insert(node, target_digest.clone());
            let checked_index = *checked_by_pin
                .get(&pin.hex)
                .ok_or("checked subflow export missing")?;
            subflow_exports.push((target_digest, checked_index));
        }
    }
    let (document, resource_alias_ids) =
        semantic_ir(input, &raws, &pin_by_alias, &subflow_targets, &env)?;
    let checked_ir = document
        .check(&env.schemas)
        .map_err(|e| format!("IR validation: {e}"))?;
    checked_ir
        .analyze_dataflow()
        .map_err(|e| format!("Phase 3C dataflow: {e}"))?;

    let mut snapshot_builder = StaticComputationResolutionSnapshotBuilder::new(&view, &subflows);
    for index in checked_by_pin.values().copied() {
        snapshot_builder = snapshot_builder
            .component(&checked_values[index])
            .map_err(|e| format!("snapshot: {e}"))?;
    }
    for (target_digest, checked_index) in &subflow_exports {
        snapshot_builder = snapshot_builder
            .subflow_export(target_digest, &checked_values[*checked_index])
            .map_err(|e| format!("subflow snapshot export: {e}"))?;
    }
    let snapshot = snapshot_builder.build();
    let mut resolved = Vec::new();
    for child in children {
        let node = ComputationId::from_raw(fixture_u32(&child["node"], "node id"));
        let leaf = resolve_computation_leaf(&checked_ir, node, &snapshot)
            .map_err(|e| format!("leaf resolution at {node}: {e}"))?;
        resolved.push(leaf);
    }
    let summary =
        derive_control_contract(&checked_ir, ComputationId::from_raw(1), &resolved, &view)
            .map_err(|e| format!("control derivation: {e}"))?;
    let workflow = if input["include_workflow_closure"] == true {
        let workflow = derive_workflow_contract_summary(&checked_ir, &snapshot, &view)
            .map_err(|e| format!("workflow closure: {e}"))?;
        if &summary != workflow.root() {
            return Err(
                "subtree and workflow-root derivations disagree for the checked root".into(),
            );
        }
        Some(workflow)
    } else {
        None
    };
    let first_projection = project(&summary, input, &pin_aliases, &resource_alias_ids).into_value();
    let repeated =
        derive_control_contract(&checked_ir, ComputationId::from_raw(1), &resolved, &view)
            .map_err(|e| format!("repeat control derivation: {e}"))?;
    let repeated_projection =
        project(&repeated, input, &pin_aliases, &resource_alias_ids).into_value();
    if first_projection != repeated_projection {
        return Err("identical checked IR/snapshot produced different projections".into());
    }
    if workflow.is_some() {
        let repeated_workflow = derive_workflow_contract_summary(&checked_ir, &snapshot, &view)
            .map_err(|e| format!("repeat workflow closure: {e}"))?;
        if workflow.as_ref() != Some(&repeated_workflow) {
            return Err("identical checked IR/snapshot produced different workflow closure".into());
        }
    }
    let mut projection = first_projection;
    if let Some(workflow) = workflow.as_ref() {
        let closure_projection = workflow_closure_json(workflow, &pin_aliases, &resource_alias_ids);
        projection["workflow_closure"] = closure_projection.clone();
        projection["_workflow_closure"] = closure_projection;
    }
    projection["_execution_stage"] = json!("control_derivation");
    projection["_resolved_leaf_count"] = json!(resolved.len());
    projection["_same_snapshot_repeat_equal"] = json!(true);
    Ok(projection)
}

fn workflow_closure_json(
    summary: &DerivedWorkflowContractSummary<'_, '_>,
    pins: &BTreeMap<String, String>,
    resource_alias_ids: &BTreeMap<String, ResourceSlotId>,
) -> Value {
    json!({
        "computation_order":summary.closure().computation_order().iter().map(|node|node.raw()).collect::<Vec<_>>(),
        "leaf_order":summary.closure().leaf_order().iter().map(|node|node.raw()).collect::<Vec<_>>(),
        "known_effect_occurrences_accounted":summary.closure().known_effect_occurrences_accounted(),
        "known_effect_residuals_accounted":summary.closure().known_effect_residuals_accounted(),
        "known_recovery_sources_accounted":summary.closure().known_recovery_sources_accounted(),
        "root_export_compatibility":match summary.closure().root_export_compatibility() {af_computation::RootExportCompatibilityStatus::NotExpressedInSemanticIr=>"not_expressed_in_semantic_ir"},
        "closure_verdict":verdict(summary.closure().proof().overall()),
        "overall_verdict":verdict(summary.overall()),
        "root_recovery":root_recovery_json(summary.root_recovery(),pins,resource_alias_ids),
    })
}

fn root_recovery_json(
    summary: &af_computation::RootRecoverySummary,
    pins: &BTreeMap<String, String>,
    resource_alias_ids: &BTreeMap<String, ResourceSlotId>,
) -> Value {
    let actions = match summary.actions().value() {
        ControlDimensionValue::Complete(actions) => json!({
            "state":"complete",
            "actions":actions.iter().map(|action|root_recovery_action_json(action,pins,resource_alias_ids)).collect::<Vec<_>>(),
        }),
        ControlDimensionValue::Partial {
            known_contribution,
            unresolved_sources,
        } => json!({
            "state":"partial",
            "actions":known_contribution.as_ref().map(|actions|actions.iter().map(|action|root_recovery_action_json(action,pins,resource_alias_ids)).collect::<Vec<_>>()).unwrap_or_default(),
            "unresolved_sources":unresolved_sources.iter().map(|node|node.raw()).collect::<Vec<_>>(),
        }),
        ControlDimensionValue::Unknown {
            known_contributions,
            unresolved_sources,
        } => json!({
            "state":"unknown",
            "actions":known_contributions.iter().flat_map(|item|item.value().iter().map(|action|root_recovery_action_json(action,pins,resource_alias_ids))).collect::<Vec<_>>(),
            "unresolved_sources":unresolved_sources.iter().map(|node|node.raw()).collect::<Vec<_>>(),
        }),
        ControlDimensionValue::PerChild {
            unresolved_sources, ..
        } => json!({
            "state":"per_child",
            "unresolved_sources":unresolved_sources.iter().map(|node|node.raw()).collect::<Vec<_>>(),
        }),
        ControlDimensionValue::NotApplicable => json!({"state":"not_applicable"}),
    };
    let proof = summary
        .proof()
        .dimensions()
        .iter()
        .map(|item| {
            json!({
                "dimension":item.dimension(),
                "verdict":verdict(item.verdict()),
                "rule_id":item.rule_id(),
                "source_node":item.source_node().raw(),
                "reference":item.reference(),
                "prerequisites":item.prerequisites(),
            })
        })
        .collect::<Vec<_>>();
    json!({
        "actions":actions,
        "action_effects":effect_dimension_json(summary.action_effects()),
        "epsilon_coverage_requirement":effect_dimension_json(summary.epsilon_coverage_requirement()),
        "recovery_usage":usage_dimension_json(summary.recovery_usage()),
        "source_closure":{
            "source_nodes_accounted":verdict(summary.source_closure().source_nodes_accounted),
            "action_bindings_accounted":verdict(summary.source_closure().action_bindings_accounted),
            "usage_profiles_accounted":verdict(summary.source_closure().usage_profiles_accounted),
            "multiplicity_factors_accounted":verdict(summary.source_closure().multiplicity_factors_accounted),
            "root_action_totals_sound":verdict(summary.source_closure().root_action_totals_sound),
        },
        "overall_verdict":verdict(summary.overall()),
        "proof_dimensions":proof,
    })
}

fn root_recovery_action_json(
    action: &af_computation::RootRecoveryAction,
    pins: &BTreeMap<String, String>,
    resource_alias_ids: &BTreeMap<String, ResourceSlotId>,
) -> Value {
    let owner = match action.owner {
        af_computation::RootRecoveryOwner::CompensationScope(node) => {
            json!({"kind":"compensation_scope","node":node.raw()})
        }
        af_computation::RootRecoveryOwner::RootRecovery => json!({"kind":"root_recovery"}),
    };
    let usage_binding = &action.recovery_usage_binding;
    json!({
        "source":{
            "caller_semantic_ir_digest":digest_ref_json(&action.source.caller_semantic_ir_digest),
            "source_node":action.source.source_node.raw(),
            "source_contract_digest":pins.get(&action.source.source_contract_digest.hex).cloned().unwrap_or_else(||action.source.source_contract_digest.hex.clone()),
            "forward_entry_ordinal":action.source.forward_entry_ordinal,
            "action":recovery_action_identity_json(&action.source.action),
        },
        "forward_seed_upper":ceiling_json(action.forward_seed_upper),
        "target":target_json(&action.target,pins),
        "owner":owner,
        "local_action_bound":action.local_bound.as_ref().map(local_action_bound_json),
        "root_action_bound":{
            "root_semantic_ir_digest":digest_ref_json(&action.root_bound.root_semantic_ir_digest),
            "upper":recovery_action_upper_json(action.root_bound.upper),
            "applied_families":action.root_bound.applied_families.iter().map(applied_multiplicity_json).collect::<Vec<_>>(),
        },
        "forward_occurrence":scoped_effect_occurrence_json(&action.forward_occurrence,resource_alias_ids),
        "action_effect_occurrence":scoped_effect_occurrence_json(&action.action_effect_occurrence,resource_alias_ids),
        "recovery_usage_binding":{
            "action":recovery_action_identity_json(&usage_binding.action),
            "activations_per_forward":bounds_json(&usage_binding.activations_per_forward),
            "attempts_per_activation":bounds_json(&usage_binding.attempts_per_activation),
            "per_attempt_usage":usage_signature_json(&usage_binding.per_attempt_usage),
        },
        "local_recovery_usage":action.local_recovery_usage.as_ref().map(usage_json),
        "root_recovery_usage":usage_json(&action.root_recovery_usage),
        "guard_compatibility":action.guard_compatibility.as_ref().map(guard_compatibility_json),
        "guard_coordinates":action.guard_coordinates.iter().map(|coordinate|json!({
            "source_contract_digest":pins.get(&coordinate.source_contract_digest.hex).cloned().unwrap_or_else(||coordinate.source_contract_digest.hex.clone()),
            "component_input_port":coordinate.component_input_port,
            "input_expression":coordinate.input_expression.raw(),
            "exact_type":coordinate.exact_type.as_ref().map(schema_ref_json),
            "dependency_slots":coordinate.dependency_slots.iter().map(|slot|slot.raw()).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "eligibility_evidence":verdict(action.eligibility_evidence),
        "control_path":action.control_path.iter().map(control_path_step_json).collect::<Vec<_>>(),
    })
}

fn local_action_bound_json(bound: &af_computation::LocalActionBound) -> Value {
    json!({
        "coordinate":"one_compensation_scope_activation",
        "compensation_scope_node":bound.compensation_scope_node.raw(),
        "upper":recovery_action_upper_json(bound.upper),
        "applied_families":bound.applied_families.iter().map(applied_multiplicity_json).collect::<Vec<_>>(),
    })
}

fn recovery_action_upper_json(upper: af_computation::RecoveryActionUpper) -> Value {
    match upper {
        af_computation::RecoveryActionUpper::Known(upper) => {
            json!({"state":"known","upper":ceiling_json(upper)})
        }
        af_computation::RecoveryActionUpper::Overflow => json!({"state":"overflow"}),
        af_computation::RecoveryActionUpper::InvalidMultiplicity => {
            json!({"state":"invalid_multiplicity"})
        }
        af_computation::RecoveryActionUpper::Unknown => json!({"state":"unknown"}),
    }
}

fn applied_multiplicity_json(factor: &af_computation::AppliedMultiplicity) -> Value {
    json!({
        "identity":{
            "caller_semantic_ir_digest":digest_ref_json(&factor.identity.caller_semantic_ir_digest),
            "control_node":factor.identity.control_node.raw(),
            "kind":match factor.identity.kind {af_computation::ControlMultiplicityKind::Iterate=>"iterate",af_computation::ControlMultiplicityKind::Retry=>"retry"},
        },
        "logical_repetitions":bounds_json(&factor.logical_repetitions),
        "physical_attempts":factor.physical_attempts.as_ref().map(bounds_json),
    })
}

fn scoped_effect_occurrence_json(
    occurrence: &af_computation::ScopedEffectOccurrence,
    resource_alias_ids: &BTreeMap<String, ResourceSlotId>,
) -> Value {
    json!({
        "caller_semantic_ir_digest":digest_ref_json(&occurrence.caller_semantic_ir_digest),
        "source_contract_digest":occurrence.source_contract_digest.hex,
        "source_node":occurrence.computation.raw(),
        "entry_ordinal":occurrence.source_entry_ordinal,
        "declared":entry_json(&occurrence.declared),
        "mapped_resource_slot":occurrence.mapped_resource_slot.map(|slot|resource_alias_name(slot,resource_alias_ids)),
    })
}

fn control_path_step_json(step: &af_computation::ControlPathStep) -> Value {
    let (kind, detail) = match &step.kind {
        af_computation::ControlPathKind::SequenceChild => ("sequence_child", json!({})),
        af_computation::ControlPathKind::SelectCase {
            case_ordinal,
            selector,
            selector_dependencies,
            exact_case,
        } => (
            "select_case",
            json!({"case_ordinal":case_ordinal,"selector":selector.map(af_semantic::ExpressionId::raw),"selector_dependencies":selector_dependencies.iter().map(|id|id.raw()).collect::<Vec<_>>(),"exact_case":exact_case}),
        ),
        af_computation::ControlPathKind::ParallelBranch { branch_ordinal } => {
            ("parallel_branch", json!({"branch_ordinal":branch_ordinal}))
        }
        af_computation::ControlPathKind::RaceBranch { branch_ordinal } => {
            ("race_branch", json!({"branch_ordinal":branch_ordinal}))
        }
        af_computation::ControlPathKind::IterateBody => ("iterate_body", json!({})),
        af_computation::ControlPathKind::RetryChild { max_attempts } => {
            ("retry_child", json!({"max_attempts":max_attempts}))
        }
        af_computation::ControlPathKind::TimeoutChild => ("timeout_child", json!({})),
        af_computation::ControlPathKind::HandleProtected => ("handle_protected", json!({})),
        af_computation::ControlPathKind::HandleClause { ordinal } => {
            ("handle_clause", json!({"ordinal":ordinal}))
        }
        af_computation::ControlPathKind::ScopeChild => ("scope_child", json!({})),
        af_computation::ControlPathKind::CompensationScopeChild => {
            ("compensation_scope_child", json!({}))
        }
    };
    json!({
        "parent_node":step.parent_node.raw(),
        "child_node":step.child_node.raw(),
        "child_index":step.child_index,
        "kind":kind,
        "detail":detail,
        "reachability":match step.reachability {af_computation::ControlPathReachability::Possible=>"possible",af_computation::ControlPathReachability::ExactSelected=>"exact_selected",af_computation::ControlPathReachability::ProvenUnreachable=>"proven_unreachable"},
    })
}

fn effect_dimension_json(dimension: &af_computation::ControlDimension<EffectSignature>) -> Value {
    match dimension.value() {
        ControlDimensionValue::Complete(value) => {
            json!({"state":"complete","value":effect_json(value)})
        }
        ControlDimensionValue::Partial {
            known_contribution,
            unresolved_sources,
        } => {
            json!({"state":"partial","value":known_contribution.as_ref().map(effect_json),"unresolved_sources":unresolved_sources.iter().map(|node|node.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::Unknown {
            known_contributions,
            unresolved_sources,
        } => {
            json!({"state":"unknown","known_contributions":known_contributions.iter().map(|item|effect_json(item.value())).collect::<Vec<_>>(),"unresolved_sources":unresolved_sources.iter().map(|node|node.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::PerChild {
            unresolved_sources, ..
        } => {
            json!({"state":"per_child","unresolved_sources":unresolved_sources.iter().map(|node|node.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::NotApplicable => json!({"state":"not_applicable"}),
    }
}

fn usage_dimension_json(
    dimension: &af_computation::ControlDimension<StaticUsageEnvelope>,
) -> Value {
    match dimension.value() {
        ControlDimensionValue::Complete(value) => {
            json!({"state":"complete","value":usage_json(value)})
        }
        ControlDimensionValue::Partial {
            known_contribution,
            unresolved_sources,
        } => {
            json!({"state":"partial","value":known_contribution.as_ref().map(usage_json),"unresolved_sources":unresolved_sources.iter().map(|node|node.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::Unknown {
            known_contributions,
            unresolved_sources,
        } => {
            json!({"state":"unknown","known_contributions":known_contributions.iter().map(|item|usage_json(item.value())).collect::<Vec<_>>(),"unresolved_sources":unresolved_sources.iter().map(|node|node.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::PerChild {
            unresolved_sources, ..
        } => {
            json!({"state":"per_child","unresolved_sources":unresolved_sources.iter().map(|node|node.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::NotApplicable => json!({"state":"not_applicable"}),
    }
}

/// Materialize typed Sequence/PARALLEL output fixtures through checked IR,
/// Phase 3C dataflow, leaf resolution, and the production control analyzer.
/// A type mismatch returns the actual IR-validation error instead.
pub fn analyze_output_case(case: &Value) -> Result<Value, String> {
    let input = &case["input"];
    let children = input["children"]
        .as_array()
        .ok_or("output fixture has no children")?;
    let env = FixtureContext::for_case(case);
    let view = env.view();
    let schema_for_type = |type_id: &Value| -> Result<af_semantic::TypeSchemaRef, String> {
        let name = match type_id.as_str() {
            Some("semantic-type:1") => "TypeSchemaRef",
            Some("semantic-type:2") => "DigestRefValue",
            other => return Err(format!("unsupported output type id {other:?}")),
        };
        env.schemas
            .get(&format!("agentflow.semantic.core@1/{name}"))
            .map(af_semantic::SchemaEntry::reference)
            .ok_or_else(|| format!("missing output schema {name}"))
    };
    let mut checked_by_alias =
        BTreeMap::<String, af_contract::CheckedComponentContract<'_, '_>>::new();
    let mut pin_by_alias = BTreeMap::<String, DigestRefValue>::new();
    for child in children {
        let alias = format!("component-contract:fixture-output:{}", child["node"]);
        let mut raw = empty_contract(&alias);
        raw.information_flow = InformationFlowContract {
            confidentiality_restrictions: vec![],
            summary: StaticInformationSummary {
                sources: vec![],
                taints: vec![],
                instruction_authority: InstructionAuthority::None,
                influence_unknown: false,
            },
            flow_constraints: vec![],
        };
        raw.outputs = child["outputs"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|output| {
                Ok(ComponentOutputPort {
                    port_no: fixture_u32(&output["port"], "output port"),
                    type_ref: schema_for_type(&output["type_id"])?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let checked = check_contract(&raw, &view).map_err(|report| {
            let failures = report
                .dimensions
                .iter()
                .filter(|item| item.verdict != af_contract::ProofVerdict::Proven)
                .map(|item| {
                    format!(
                        "{}={}:{}",
                        item.dimension,
                        item.verdict.as_str(),
                        item.reason
                    )
                })
                .collect::<Vec<_>>()
                .join("; ");
            format!("output child contract validation: {failures}")
        })?;
        let digest =
            component_contract_digest(&checked, &view).map_err(|error| error.to_string())?;
        pin_by_alias.insert(
            alias.clone(),
            DigestRefValue {
                algorithm: "sha256".into(),
                domain: digest.domain().as_str().into(),
                hex: digest.hex().into(),
            },
        );
        checked_by_alias.insert(alias, checked);
    }

    let mut output_slots = BTreeMap::<String, af_semantic::ValueSlotId>::new();
    let mut type_ids = BTreeMap::<u32, af_semantic::TypeSchemaRef>::new();
    let mut values = Vec::new();
    let mut computations = Vec::new();
    let mut refs = BTreeMap::<String, ContractRefId>::new();
    let mut contracts = Vec::new();
    let mut sorted_aliases = pin_by_alias.iter().collect::<Vec<_>>();
    sorted_aliases.sort_by(|left, right| left.0.cmp(right.0));
    for (index, (alias, pin)) in sorted_aliases.iter().enumerate() {
        let id = ContractRefId::from_raw(fixture_index(index) + 1);
        refs.insert((*alias).clone(), id);
        contracts.push(ContractRefEntry {
            id,
            kind: ContractRefKind::Component,
            digest: (*pin).clone(),
        });
    }
    for output in input["parent_outputs"]
        .as_array()
        .into_iter()
        .flatten()
        .chain(
            children
                .iter()
                .flat_map(|child| child["outputs"].as_array().into_iter().flatten()),
        )
    {
        let type_ref = schema_for_type(&output["type_id"])?;
        let type_id = output["type_id"]
            .as_str()
            .unwrap()
            .strip_prefix("semantic-type:")
            .unwrap()
            .parse::<u32>()
            .map_err(|error| error.to_string())?;
        type_ids.insert(type_id, type_ref);
        let slot = output["slot"]
            .as_str()
            .ok_or("output has no value slot")?
            .to_owned();
        let next = fixture_index(output_slots.len()) + 1;
        let slot_id = *output_slots
            .entry(slot)
            .or_insert(af_semantic::ValueSlotId::from_raw(next));
        values.push(af_semantic::ValueSlot {
            id: slot_id,
            type_id: type_id.into(),
        });
    }
    let type_nodes = type_ids
        .into_iter()
        .map(|(id, schema)| SemanticTypeNode {
            id: id.into(),
            body: SemanticTypeNodeBody::Schema(schema),
        })
        .collect::<Vec<_>>();
    let root_id = ComputationId::from_raw(1);
    let child_ids = children
        .iter()
        .map(|child| ComputationId::from_raw(fixture_u32(&child["node"], "child node")))
        .collect::<Vec<_>>();
    let root_outputs = input["parent_outputs"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|output| {
            let slot = output_slots[output["slot"].as_str().unwrap()];
            ComputationOutputBinding {
                port_no: fixture_u32(&output["port"], "parent output port").into(),
                value_slot: slot,
            }
        })
        .collect::<Vec<_>>();
    let root_body = if input["operation"] == "parallel_output" && input["branches"].is_array() {
        let branches = input["branches"].as_array().unwrap();
        let output_bindings = input["bindings"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|binding| {
                let child_index = children
                    .iter()
                    .position(|child| child["node"] == binding["child"])
                    .ok_or("output binding names unknown child")?;
                let ordinal = binding
                    .get("branch_ordinal")
                    .and_then(Value::as_u64)
                    .unwrap_or(child_index as u64 + 1);
                Ok(ParallelOutputBinding {
                    parent_port_no: fixture_u32(&binding["parent_port"], "parent port").into(),
                    branch_ordinal: u32::try_from(ordinal).unwrap().into(),
                    child_port_no: fixture_u32(&binding["child_port"], "child port").into(),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        ComputationBody::Parallel(ParallelBody {
            mode: ParallelMode::All,
            branches: branches
                .iter()
                .map(|branch| ParallelBranch {
                    ordinal: fixture_u32(&branch["ordinal"], "branch ordinal").into(),
                    child: ComputationId::from_raw(fixture_u32(&branch["child"], "branch child")),
                })
                .collect(),
            max_inflight: None,
            output_bindings,
        })
    } else if input["operation"] == "parallel_output" {
        let output_bindings = input["bindings"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|binding| {
                let child_index = children
                    .iter()
                    .position(|child| child["node"] == binding["child"])
                    .ok_or("output binding names unknown child")?;
                Ok(ParallelOutputBinding {
                    parent_port_no: fixture_u32(&binding["parent_port"], "parent port").into(),
                    branch_ordinal: fixture_index(child_index).checked_add(1).unwrap().into(),
                    child_port_no: fixture_u32(&binding["child_port"], "child port").into(),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        ComputationBody::Parallel(ParallelBody {
            mode: ParallelMode::All,
            branches: child_ids
                .iter()
                .enumerate()
                .map(|(index, child)| ParallelBranch {
                    ordinal: fixture_index(index).checked_add(1).unwrap().into(),
                    child: *child,
                })
                .collect(),
            max_inflight: None,
            output_bindings,
        })
    } else {
        ComputationBody::Sequence(SequenceBody {
            children: child_ids.clone(),
        })
    };
    computations.push(ComputationDef {
        id: root_id,
        outputs: root_outputs,
        body: root_body,
    });
    for child in children {
        let node = ComputationId::from_raw(fixture_u32(&child["node"], "child node"));
        let alias = format!("component-contract:fixture-output:{}", child["node"]);
        let outputs = child["outputs"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|output| ComputationOutputBinding {
                port_no: fixture_u32(&output["port"], "child output port").into(),
                value_slot: output_slots[output["slot"].as_str().unwrap()],
            })
            .collect();
        computations.push(ComputationDef {
            id: node,
            outputs,
            body: ComputationBody::Invoke(InvokeBody {
                contract: refs[&alias],
                inputs: vec![],
                resources: vec![],
            }),
        });
    }
    let document = SemanticIrDocument {
        semantic_refs: vec![],
        types: type_nodes,
        values,
        expressions: vec![],
        resources: vec![],
        contracts,
        run_parameters: vec![],
        workflow_outputs: vec![],
        computations,
        root: root_id,
    };
    let checked_ir = match document.check(&env.schemas) {
        Ok(checked) => checked,
        Err(error) => {
            return Ok(json!({
                "stage":"ir_validation",
                "error_kind":error.kind(),
                "rule_id":if error.kind() == "parallel_output_type_mismatch" { "AF-SEMANTIC-0056" } else { "unmapped" },
                "verdict":"violation"
            }));
        }
    };
    checked_ir
        .analyze_dataflow()
        .map_err(|error| format!("DataflowValidation:{}:{error}", error.kind()))?;
    let subflows = SubflowIrUniverse::new();
    let mut snapshot_builder = StaticComputationResolutionSnapshotBuilder::new(&view, &subflows);
    for (alias, checked) in &checked_by_alias {
        snapshot_builder = snapshot_builder
            .component(checked)
            .map_err(|error| format!("snapshot:{error}"))?;
        let _ = alias;
    }
    let snapshot = snapshot_builder.build();
    let leaves = children
        .iter()
        .map(|child| {
            let node = ComputationId::from_raw(fixture_u32(&child["node"], "child node"));
            resolve_computation_leaf(&checked_ir, node, &snapshot)
                .map_err(|error| format!("leaf resolution:{error}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let summary = derive_control_contract(&checked_ir, root_id, &leaves, &view)
        .map_err(|error| error.to_string())?;
    let outputs = summary.interface().outputs();
    let ControlDimensionValue::Complete(ports) = outputs else {
        return Err("control output interface was not complete".into());
    };
    let actual_outputs = ports.iter().flat_map(|port| port.sources().iter().map(move |source| json!({
        "parent_port":port.parent_port_no(),
        "parent_slot":input["parent_outputs"].as_array().unwrap().iter().find(|output| fixture_u32(&output["port"], "port")==port.parent_port_no()).map_or(Value::Null, |output| output["slot"].clone()),
        "source_child":source.child_node().raw(),
        "source_port":source.child_port_no(),
        "type_id":format!("semantic-type:{}",port.local_type_id().raw()),
    }))).collect::<Vec<_>>();
    let output_verdict = if matches!(outputs, ControlDimensionValue::Complete(_)) {
        "proven"
    } else {
        "unproven"
    };
    let pins = pin_by_alias
        .iter()
        .map(|(alias, pin)| (pin.hex.clone(), alias.clone()))
        .collect::<BTreeMap<_, _>>();
    let full_projection = project(&summary, input, &pins, &BTreeMap::new()).into_value();
    Ok(
        json!({"outputs":actual_outputs,"verdict":output_verdict,"_execution_stage":"control_derivation","_semantic_projection":full_projection["_semantic_projection"]}),
    )
}

fn component_contract(child: &Value, env: &FixtureContext) -> Result<ComponentContract, String> {
    let mut contract = empty_contract(child["contract_digest"].as_str().unwrap());
    contract.outputs = child["outputs"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|output| {
            Ok(ComponentOutputPort {
                port_no: fixture_u32(&output["port"], "output port"),
                type_ref: fixture_type_schema(&output["type_id"], &env.schemas)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    contract.effects = parse_effects(&child["effects"])?;
    contract.failures = FailureRow {
        failures: child["failures"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|name| failure_ref(name.as_str().unwrap(), &env.schemas))
            .collect::<Result<Vec<_>, _>>()?,
    };
    let mut input_ports = BTreeSet::new();
    let input_types = child["input_types"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|item| {
            (
                fixture_u32(&item["port"], "component input port"),
                item["primitive"].as_str().unwrap_or_default().to_owned(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    input_ports.extend(input_types.keys().copied());
    if let Some(flow) = child.get("flow").filter(|v| !v.is_null()) {
        if let Some(sources) = flow["sources"].as_array() {
            for source in sources.iter().filter_map(Value::as_str) {
                if let Some(port) = source
                    .strip_prefix("input:")
                    .and_then(|n| n.parse::<u32>().ok())
                {
                    input_ports.insert(port);
                }
            }
        }
    }
    for entry in entries(&contract.effects) {
        if let EffectGuard::InputEquals(guard) = &entry.guard {
            input_ports.insert(guard.port_no);
        }
    }
    contract.inputs = input_ports
        .iter()
        .map(|port_no| ComponentInputPort {
            port_no: *port_no,
            type_ref: input_types.get(port_no).map_or_else(
                || {
                    env.schemas
                        .get("agentflow.semantic.core@1/TypeSchemaRef")
                        .unwrap()
                        .reference()
                },
                |primitive| {
                    env.schemas
                        .get(&format!(
                            "agentflow.test.guard-types@1/{}",
                            guard_type_schema_name(primitive)
                        ))
                        .unwrap_or_else(|| panic!("missing fixture guard schema {primitive}"))
                        .reference()
                },
            ),
        })
        .collect();
    let mut resource_ports = BTreeSet::new();
    if let Some(resources) = child["resources"].as_array() {
        for resource in resources {
            resource_ports.insert(fixture_u32(&resource["component_port"], "component port"));
        }
    }
    for entry in entries(&contract.effects) {
        resource_ports.insert(entry.atom.resource_selector.requirement_port_no);
    }
    contract.resources = ResourceRequirementRow {
        requirements: resource_ports
            .into_iter()
            .map(|port_no| ResourceRequirement {
                port_no,
                resource_type: env.resource_type().clone(),
                required_attributes: vec![],
            })
            .collect(),
    };
    contract.information_flow = if child.get("flow").is_none_or(Value::is_null) {
        InformationFlowContract {
            confidentiality_restrictions: vec![],
            summary: StaticInformationSummary {
                sources: vec![],
                taints: vec![],
                instruction_authority: InstructionAuthority::None,
                influence_unknown: false,
            },
            flow_constraints: vec![],
        }
    } else {
        parse_flow(child.get("flow"), &env.flow)?
    };
    contract.usage = parse_usage(child.get("usage"))?;
    contract.execution = parse_execution(child.get("execution"));
    if let Some(replay_semantics) = child.get("replay_semantics").and_then(Value::as_str) {
        contract.recovery.replay_semantics = match replay_semantics {
            "recompute" => ReplaySemantics::Recompute,
            "recorded" => ReplaySemantics::Recorded,
            "reconcile" => ReplaySemantics::Reconcile,
            "forbidden" => ReplaySemantics::Forbidden,
            other => return Err(format!("unsupported fixture replay semantics {other}")),
        };
    }
    let mut actions = Vec::new();
    let mut usage_bindings = Vec::new();
    for item in child["recovery_actions"].as_array().into_iter().flatten() {
        let forward_effect = parse_atom(&item["forward_atom"])?;
        let identity = recovery_action_identity(item)?;
        let clause = &item["clause"];
        let idempotency = match clause["idempotency"].as_str() {
            Some("natural") => IdempotencySemantics::Natural,
            Some("keyed") => IdempotencySemantics::Keyed,
            Some("none") => IdempotencySemantics::None,
            Some("reconciled") => IdempotencySemantics::Reconciled,
            other => return Err(format!("unsupported recovery idempotency {other:?}")),
        };
        let key_requirement = if clause["key_requirement"].is_null() {
            None
        } else {
            Some(KeyStabilityRequirement {
                stable_across_attempts: clause["key_requirement"]["stable_across_attempts"]
                    .as_bool()
                    .ok_or("missing stable_across_attempts")?,
                distinct_across_invocations:
                    clause["key_requirement"]["distinct_across_invocations"]
                        .as_bool()
                        .ok_or("missing distinct_across_invocations")?,
                bound_to_operation: clause["key_requirement"]["bound_to_operation"]
                    .as_bool()
                    .ok_or("missing bound_to_operation")?,
                provider_honors: clause["key_requirement"]["provider_honors"]
                    .as_bool()
                    .ok_or("missing provider_honors")?,
            })
        };
        let reconciliation = match clause["reconciliation"].as_str() {
            Some("none") => ReconciliationRequirement::None,
            Some("required") => ReconciliationRequirement::Required,
            other => return Err(format!("unsupported reconciliation requirement {other:?}")),
        };
        let compensation = if clause["compensation"] == "none" {
            CompensationRequirement::None
        } else {
            CompensationRequirement::Required(EffectOperationRef {
                catalog: clause["compensation"]["required"]["catalog"]
                    .as_str()
                    .ok_or("missing compensation catalog")?
                    .into(),
                name: clause["compensation"]["required"]["name"]
                    .as_str()
                    .ok_or("missing compensation operation")?
                    .into(),
            })
        };
        let unknown_effect_handling = match clause["unknown_effect_handling"].as_str() {
            Some("block") => UnknownEffectHandling::Block,
            Some("reconciliation_required") => UnknownEffectHandling::ReconciliationRequired,
            other => return Err(format!("unsupported unknown-effect handling {other:?}")),
        };
        contract
            .recovery
            .effect_recovery_clauses
            .push(EffectRecoveryClause {
                effect: forward_effect,
                idempotency,
                key_requirement,
                reconciliation,
                compensation,
                unknown_effect_handling,
            });
        actions.push(RecoveryActionBinding {
            identity: identity.clone(),
        });
        if child["omit_usage_binding"] != true {
            usage_bindings.push(RecoveryUsageBinding {
                action: identity,
                activations_per_forward: parse_bounds(&item["activations_per_forward"])?,
                attempts_per_activation: parse_bounds(&item["attempts_per_activation"])?,
                per_attempt_usage: parse_usage(item.get("per_attempt_usage"))?,
            });
        }
    }
    contract.recovery_action_bindings = actions;
    contract.recovery_usage_bindings = usage_bindings;
    Ok(contract)
}

fn recovery_action_identity(item: &Value) -> Result<RecoveryActionIdentity, String> {
    let action_kind = match item["action"].as_str() {
        Some("compensate") => RecoveryActionKind::Compensation,
        Some("reconcile") => RecoveryActionKind::Reconciliation,
        other => return Err(format!("unsupported recovery action kind {other:?}")),
    };
    Ok(RecoveryActionIdentity {
        forward_effect: parse_atom(&item["forward_atom"])?,
        action_kind,
        action_effect: parse_atom(&item["action_atom"])?,
    })
}

pub fn validate_contract_case(case: &Value) -> Result<Value, String> {
    let input = &case["input"];
    let child = input
        .get("contract")
        .or_else(|| input["children"].as_array().and_then(|items| items.first()))
        .ok_or_else(|| "contract-validation fixture lacks a contract object".to_string())?;
    let env = FixtureContext::for_case(case);
    let raw = component_contract(child, &env)?;
    let view = env.view();
    let report = analyze_contract(&raw, &view);
    let checked_component_produced = check_contract(&raw, &view).is_ok();
    let dimensions = report
        .dimensions
        .iter()
        .filter(|dimension| dimension.verdict != af_contract::ProofVerdict::Proven)
        .map(|dimension| {
            json!({
                "dimension": dimension.dimension,
                "verdict": dimension.verdict.as_str(),
                "rule_id": dimension.rule_id,
                "reason": dimension.reason,
                "reference": dimension.reference,
            })
        })
        .collect::<Vec<_>>();
    let action_guard_compatibility = report
        .action_guard_compatibility
        .iter()
        .map(guard_compatibility_json)
        .collect::<Vec<_>>();
    Ok(json!({
        "stage": "contract_validation",
        "verdict": report.overall().as_str(),
        "checked_component_produced": checked_component_produced,
        "dimensions": dimensions,
        "action_guard_compatibility": action_guard_compatibility,
    }))
}

fn guard_compatibility_json(proof: &af_contract::ActionGuardCompatibility) -> Value {
    let basis = match proof.basis() {
        af_contract::GuardCompatibilityBasis::FrozenGuardContainment => "frozen_guard_containment",
        af_contract::GuardCompatibilityBasis::ZeroPossibleActions => "zero_possible_actions",
        af_contract::GuardCompatibilityBasis::TypedCountermodel => "typed_countermodel",
        af_contract::GuardCompatibilityBasis::InsufficientEvidence => "insufficient_evidence",
    };
    let countermodel = proof.countermodel().map(|witness| {
        json!({
            "source_component":{"catalog":witness.source_component().catalog,"name":witness.source_component().name},
            "recovery_action":recovery_action_identity_json(witness.recovery_action()),
            "forward_entry_ordinal":witness.forward_entry_ordinal(),
            "action_entry_ordinal":witness.action_entry_ordinal(),
            "forward_guard":guard_json(witness.forward_guard()),
            "declared_action_guard":guard_json(witness.declared_action_guard()),
            "forward_bounds":bounds_json(witness.forward_bounds()),
            "activation_upper":ceiling_json(witness.activation_upper()),
            "attempt_upper":ceiling_json(witness.attempt_upper()),
            "action_upper":ceiling_json(witness.action_upper()),
            "assignments":witness.assignments().iter().map(|assignment|json!({
                "port":assignment.port_no(),
                "exact_type":schema_ref_json(assignment.type_ref()),
                "value":primitive_constant_json(assignment.value()),
            })).collect::<Vec<_>>(),
            "reachability":match witness.reachability() {
                af_contract::GuardCountermodelReachability::WellTypedComponentInputDomain => "well_typed_component_input_domain",
            },
            "related_rule_id":witness.related_rule_id(),
            "source_contract_identity":{"catalog":witness.source_contract().component_identity.catalog,"name":witness.source_contract().component_identity.name},
        })
    });
    json!({
        "source_component":{"catalog":proof.source_component().catalog,"name":proof.source_component().name},
        "recovery_action":recovery_action_identity_json(proof.recovery_action()),
        "forward_entry_ordinal":proof.forward_entry_ordinal(),
        "action_entry_ordinal":proof.action_entry_ordinal(),
        "forward_guard":proof.forward_guard().map(guard_json),
        "declared_action_guard":proof.declared_action_guard().map(guard_json),
        "action_upper":proof.action_upper().map(ceiling_json),
        "verdict":proof.verdict().as_str(),
        "basis":basis,
        "countermodel":countermodel,
        "reason":proof.reason(),
    })
}

fn recovery_action_identity_json(identity: &RecoveryActionIdentity) -> Value {
    json!({
        "forward_effect":atom_json(&identity.forward_effect),
        "action_kind":identity.action_kind.as_str(),
        "action_effect":atom_json(&identity.action_effect),
    })
}

fn primitive_constant_json(value: &PrimitiveConstant) -> Value {
    match value {
        PrimitiveConstant::Unit => json!("unit"),
        PrimitiveConstant::Null => json!("null"),
        PrimitiveConstant::Bool(value) => json!(value),
        PrimitiveConstant::Int64(value) => json!(value),
        PrimitiveConstant::Decimal(value) => json!(value.to_string()),
        PrimitiveConstant::String(value) | PrimitiveConstant::Bytes(value) => json!(value),
        PrimitiveConstant::Timestamp(value) => json!({"timestamp_micros":value.as_micros()}),
        PrimitiveConstant::Duration(value) => json!({"duration_micros":value.as_micros()}),
    }
}

pub fn open_row_algebra(case: &Value) -> Result<Value, String> {
    let input = &case["input"];
    let child = input
        .get("contract")
        .or_else(|| input["children"].as_array().and_then(|items| items.first()))
        .ok_or_else(|| "open-row fixture lacks a contract object".to_string())?;
    let env = FixtureContext::for_case(case);
    let raw = component_contract(child, &env)?;
    let closed = EffectSignature {
        row: EffectRow::Closed(vec![]),
    };
    let combined = af_effect::control::sequence_envelope(&raw.effects, &closed)
        .map_err(|error| error.to_string())?;
    Ok(effect_json(&combined))
}

pub fn guarded_select_effect_algebra(case: &Value) -> Result<Value, String> {
    let children = case["input"]["children"]
        .as_array()
        .ok_or_else(|| "SELECT fixture lacks children".to_string())?;
    let signatures = children
        .iter()
        .map(|child| parse_effects(&child["effects"]))
        .collect::<Result<Vec<_>, _>>()?;
    let mut iter = signatures.iter();
    let first = iter.next().ok_or("SELECT needs at least one child")?;
    let mut result = first.clone();
    for next in iter {
        result = af_effect::control::select_envelope(&result, next)
            .map_err(|error| error.to_string())?;
    }
    Ok(effect_json(&result))
}

fn empty_contract(alias: &str) -> ComponentContract {
    let suffix = alias.rsplit(':').next().unwrap_or("component");
    ComponentContract {
        component_identity: ComponentIdentity {
            catalog: "agentflow.components@1".into(),
            name: format!("fixture.{}", &suffix[..suffix.len().min(12)]),
        },
        inputs: vec![],
        outputs: vec![],
        failures: FailureRow::empty(),
        effects: EffectSignature {
            row: EffectRow::Closed(vec![]),
        },
        resources: ResourceRequirementRow::empty(),
        information_flow: InformationFlowContract::empty(),
        usage: UsageSignature::empty(),
        recovery: RecoveryContract {
            replay_semantics: ReplaySemantics::Recompute,
            effect_recovery_clauses: vec![],
        },
        execution: ExecutionRequirements::empty(),
        effect_flow_partitions: vec![],
        recovery_action_bindings: vec![],
        recovery_usage_bindings: vec![],
    }
}

fn parse_effects(value: &Value) -> Result<EffectSignature, String> {
    let row = match value["row"].as_str().unwrap_or("closed") {
        "closed" => EffectRow::Closed(parse_entries(&value["entries"])?),
        "unknown" => EffectRow::Unknown,
        "open" => {
            let residual = value["residuals"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|item| {
                    Ok(EffectResidual {
                        catalog: item["catalog"].as_str().unwrap().into(),
                        selector: parse_selector(&item["selector"]),
                        bounds: parse_bounds(item)?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            EffectRow::Open(EffectOpenRow {
                known: parse_entries(&value["entries"])?,
                residual,
            })
        }
        other => return Err(format!("unknown fixture effect row {other}")),
    };
    Ok(EffectSignature { row })
}

fn parse_entries(value: &Value) -> Result<Vec<EffectEntry>, String> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .map(|entry| {
            Ok(EffectEntry {
                atom: parse_atom(&entry["atom"])?,
                bounds: parse_bounds(entry)?,
                guard: parse_guard(&entry["guard"])?,
            })
        })
        .collect()
}

fn parse_atom(value: &Value) -> Result<EffectAtom, String> {
    let qualifiers = value["qualifiers"]
        .as_array()
        .into_iter()
        .flatten()
        .map(
            |qualifier| match (qualifier["family"].as_str(), qualifier["member"].as_str()) {
                (Some("idempotency"), Some("natural")) => Ok(EffectQualifier::Idempotency(
                    EffectIdempotencyQualifier::Natural,
                )),
                (Some("idempotency"), Some("none")) => Ok(EffectQualifier::Idempotency(
                    EffectIdempotencyQualifier::None,
                )),
                other => Err(format!("unsupported fixture qualifier {other:?}")),
            },
        )
        .collect::<Result<Vec<_>, _>>()?;
    Ok(EffectAtom {
        operation: EffectOperationRef {
            catalog: value["operation"]["catalog"].as_str().unwrap().into(),
            name: value["operation"]["name"].as_str().unwrap().into(),
        },
        resource_selector: parse_selector(&value["resource_selector"]),
        qualifiers,
    })
}

fn parse_selector(value: &Value) -> ResourceSelector {
    ResourceSelector {
        requirement_port_no: fixture_u32(&value["requirement_port_no"], "requirement port"),
        scope_bindings: value["scope_bindings"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|binding| ScopeBinding {
                key: binding["key"].as_str().unwrap().into(),
                value: binding["value"].as_str().unwrap().into(),
            })
            .collect(),
    }
}

fn parse_guard(value: &Value) -> Result<EffectGuard, String> {
    if value == "unconditional" {
        return Ok(EffectGuard::Unconditional);
    }
    let equals = &value["input_equals"];
    if !equals.is_object() {
        return Err(format!("unsupported fixture guard: {value}"));
    }
    let expected = match &equals["value"] {
        Value::Object(value) if value.contains_key("unit") => PrimitiveConstant::Unit,
        Value::Object(value) if value.contains_key("null") => PrimitiveConstant::Null,
        Value::String(value) => PrimitiveConstant::String(value.clone()),
        Value::Number(value) => {
            PrimitiveConstant::Int64(value.as_i64().ok_or("guard int64 out of range")?)
        }
        Value::Bool(value) => PrimitiveConstant::Bool(*value),
        _ => return Err("unsupported guard literal".into()),
    };
    Ok(EffectGuard::InputEquals(EffectGuardInputEquals {
        port_no: fixture_u32(&equals["port"], "guard port"),
        expected,
    }))
}

fn parse_bounds(value: &Value) -> Result<OccurrenceBounds, String> {
    let lower = value["lower"].as_u64().ok_or("missing occurrence lower")?;
    let upper = match &value["upper"] {
        Value::String(s) if s == "unbounded" => OccurrenceCeiling::Unbounded,
        Value::Number(n) => OccurrenceCeiling::Finite(n.as_u64().ok_or("invalid upper")?),
        _ => return Err("invalid occurrence upper".into()),
    };
    Ok(OccurrenceBounds { lower, upper })
}

fn parse_flow(
    value: Option<&Value>,
    catalogs: &FlowCatalogs,
) -> Result<InformationFlowContract, String> {
    let Some(flow) = value.filter(|flow| !flow.is_null()) else {
        return Ok(InformationFlowContract::empty());
    };
    let sources = flow["sources"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|source| {
            let text = source.as_str().unwrap();
            let category = if text == "unknown_external" {
                SourceCategory::UnknownExternal
            } else if let Some(port) = text
                .strip_prefix("input:")
                .and_then(|n| n.parse::<u32>().ok())
            {
                SourceCategory::WorkflowInput(port)
            } else {
                return Err(format!("unmaterializable fixture Flow source {text}"));
            };
            Ok(FlowSourceRef { category })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let taints = flow["taints"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|item| {
            let reference = TaintRef {
                catalog: LABEL_CATALOG.into(),
                name: item.as_str().unwrap().into(),
            };
            catalogs
                .labels
                .resolve_taint(&reference)
                .map_err(|e| e.to_string())?;
            Ok(reference)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let authority = match flow["authority"].as_str().unwrap_or("none") {
        "none" => InstructionAuthority::None,
        "external_data" => InstructionAuthority::ExternalData,
        "delegated_agent" => InstructionAuthority::DelegatedAgent,
        "user" => InstructionAuthority::User,
        "workflow" => InstructionAuthority::Workflow,
        "platform" => InstructionAuthority::Platform,
        other => return Err(format!("unknown instruction authority {other}")),
    };
    Ok(InformationFlowContract {
        confidentiality_restrictions: vec![],
        summary: StaticInformationSummary {
            sources,
            taints,
            instruction_authority: authority,
            influence_unknown: flow["influence_unknown"].as_bool().unwrap_or(false),
        },
        flow_constraints: vec![],
    })
}

fn parse_usage(value: Option<&Value>) -> Result<UsageSignature, String> {
    let mut limits = Vec::new();
    if let Some(coordinates) = value
        .and_then(|v| v.get("coordinates"))
        .and_then(Value::as_object)
    {
        for (coordinate, raw_upper) in coordinates {
            let (phase, meter_name) = coordinate
                .split_once('/')
                .ok_or("malformed usage coordinate")?;
            let phase = match phase {
                "forward" => UsagePhase::Forward,
                "recovery" => UsagePhase::Recovery,
                other => return Err(format!("unknown usage phase {other}")),
            };
            let upper = match raw_upper {
                Value::String(s) if s == "unbounded" => UsageUpperBound::Unbounded,
                Value::Number(n) => {
                    UsageUpperBound::Finite(n.as_u64().ok_or("invalid usage upper")?)
                }
                _ => return Err("invalid usage upper".into()),
            };
            limits.push(UsageLimit {
                meter: UsageMeterRef {
                    catalog: METER_CATALOG.into(),
                    name: meter_name.into(),
                },
                phase,
                upper,
            });
        }
    }
    Ok(UsageSignature { limits })
}

fn parse_execution(value: Option<&Value>) -> ExecutionRequirements {
    let mut requirements = ExecutionRequirements::empty();
    let Some(value) = value.filter(|v| !v.is_null()) else {
        return requirements;
    };
    if value.get("sandbox").and_then(Value::as_str) == Some("strict") {
        requirements.sandbox = Some(SandboxRequirement {
            minimum: SandboxLevel::Strong,
        });
    }
    if value.get("runtime").and_then(Value::as_str) == Some("python") {
        requirements.runtime_profile = Some(RuntimeProfileRequirement {
            family: "language".into(),
            profile: RuntimeProfileRef {
                catalog: RUNTIME_CATALOG.into(),
                name: "python".into(),
            },
            required_features: vec![],
        });
    }
    requirements
}

fn failure_ref(name: &str, schemas: &TypeSchemaUniverse) -> Result<FailureTypeRef, String> {
    let schema = match name {
        "FailureA" => "agentflow.semantic.core@1/TypeSchemaRef",
        "FailureB" => "agentflow.semantic.core@1/DigestRefValue",
        _ => return Err(format!("unknown fixture failure alias {name}")),
    };
    Ok(FailureTypeRef {
        payload_type: schemas
            .get(schema)
            .ok_or_else(|| format!("missing schema {schema}"))?
            .reference(),
    })
}

fn fixture_type_schema(
    type_id: &Value,
    schemas: &TypeSchemaUniverse,
) -> Result<af_semantic::TypeSchemaRef, String> {
    let name = match type_id.as_str() {
        Some("semantic-type:1") => "TypeSchemaRef",
        Some("semantic-type:2") => "DigestRefValue",
        other => return Err(format!("unknown fixture Semantic type {other:?}")),
    };
    schemas
        .get(&format!("agentflow.semantic.core@1/{name}"))
        .map(af_semantic::SchemaEntry::reference)
        .ok_or_else(|| format!("missing fixture output schema {name}"))
}

fn target_subflow_document(
    component_pin: DigestRefValue,
    contract: &ComponentContract,
    env: &FixtureContext,
) -> SemanticIrDocument {
    let resource_slots = contract
        .resources
        .requirements
        .iter()
        .enumerate()
        .map(|(index, requirement)| af_semantic::ResourceSlot {
            id: (fixture_index(index) + 1).into(),
            requirement_type: requirement.resource_type.schema.clone(),
        })
        .collect::<Vec<_>>();
    let resources = contract
        .resources
        .requirements
        .iter()
        .enumerate()
        .map(|(index, requirement)| ResourceUseBinding {
            port_no: requirement.port_no.into(),
            resource_slot: ResourceSlotId::from_raw(fixture_index(index) + 1),
        })
        .collect::<Vec<_>>();
    let _ = env;
    SemanticIrDocument {
        semantic_refs: vec![],
        types: vec![],
        values: vec![],
        expressions: vec![],
        resources: resource_slots,
        contracts: vec![ContractRefEntry {
            id: 1.into(),
            kind: ContractRefKind::Component,
            digest: component_pin,
        }],
        run_parameters: vec![],
        workflow_outputs: vec![],
        computations: vec![ComputationDef {
            id: 1.into(),
            outputs: vec![],
            body: ComputationBody::Invoke(InvokeBody {
                contract: 1.into(),
                inputs: vec![],
                resources,
            }),
        }],
        root: 1.into(),
    }
}

fn entries(signature: &EffectSignature) -> Vec<&EffectEntry> {
    match &signature.row {
        EffectRow::Closed(entries) => entries.iter().collect(),
        EffectRow::Open(open) => open.known.iter().collect(),
        EffectRow::Unknown => vec![],
    }
}

fn semantic_ir(
    input: &Value,
    raws: &BTreeMap<String, ComponentContract>,
    pin_by_alias: &BTreeMap<String, DigestRefValue>,
    subflow_targets: &BTreeMap<u32, DigestRefValue>,
    env: &FixtureContext,
) -> Result<(SemanticIrDocument, BTreeMap<String, ResourceSlotId>), String> {
    let children = input["children"].as_array().unwrap();
    let mut type_nodes = vec![SemanticTypeNode {
        id: 1.into(),
        body: SemanticTypeNodeBody::Schema(
            env.schemas
                .get("agentflow.semantic.core@1/TypeSchemaRef")
                .unwrap()
                .reference(),
        ),
    }];
    let is_select = input["operator"].as_str() == Some("select");
    let is_timeout = input["operator"].as_str() == Some("timeout");
    let is_foreach = input["operator"].as_str() == Some("iterate_foreach");
    let is_while = input["operator"].as_str() == Some("iterate_while");
    let has_nested_foreach = input["nested_nodes"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|node| node["operator"] == "iterate_foreach");
    if is_select {
        type_nodes.push(SemanticTypeNode {
            id: 2.into(),
            body: SemanticTypeNodeBody::Primitive(SemanticPrimitiveType::Int64),
        });
    } else if is_timeout {
        type_nodes.push(SemanticTypeNode {
            id: 2.into(),
            body: SemanticTypeNodeBody::Primitive(SemanticPrimitiveType::Duration),
        });
    } else if is_foreach || has_nested_foreach {
        type_nodes.push(SemanticTypeNode {
            id: 2.into(),
            body: SemanticTypeNodeBody::Primitive(SemanticPrimitiveType::Int64),
        });
        type_nodes.push(SemanticTypeNode {
            id: 3.into(),
            body: SemanticTypeNodeBody::List(2.into()),
        });
    } else if is_while {
        type_nodes.push(SemanticTypeNode {
            id: 2.into(),
            body: SemanticTypeNodeBody::Primitive(SemanticPrimitiveType::Bool),
        });
    }
    let mut input_schema_by_port = BTreeMap::<u32, TypeSchemaRef>::new();
    for child in children {
        let raw = raws
            .get(child["contract_digest"].as_str().unwrap())
            .unwrap();
        for port in &raw.inputs {
            if input_schema_by_port
                .insert(port.port_no, port.type_ref.clone())
                .is_some_and(|previous| previous != port.type_ref)
            {
                return Err(format!(
                    "fixture reuses component input port {} with different exact TypeSchemaRefs",
                    port.port_no
                ));
            }
        }
    }
    let mut input_type_ids = BTreeMap::<u32, u32>::new();
    for (index, (port_no, schema)) in input_schema_by_port.iter().enumerate() {
        let type_id = 1000_u32
            .checked_add(fixture_index(index))
            .ok_or("fixture input type id overflow")?;
        type_nodes.push(SemanticTypeNode {
            id: type_id.into(),
            body: SemanticTypeNodeBody::Schema(schema.clone()),
        });
        input_type_ids.insert(*port_no, type_id);
    }
    let mut values = Vec::new();
    let mut expressions = Vec::new();
    let mut run_parameters = Vec::new();
    let mut component_inputs = BTreeMap::<u32, (u32, u32)>::new();
    let mut output_value_slots = BTreeMap::<String, ValueSlotId>::new();
    for child in children {
        for output in child["outputs"].as_array().into_iter().flatten() {
            let alias = output["slot"]
                .as_str()
                .ok_or("child output lacks a value-slot alias")?
                .to_owned();
            let raw = alias
                .rsplit(':')
                .next()
                .ok_or("malformed output value-slot alias")?
                .parse::<u32>()
                .map_err(|error| error.to_string())?;
            let slot = ValueSlotId::from_raw(raw);
            output_value_slots.insert(alias, slot);
            let type_id = output["type_id"]
                .as_str()
                .unwrap()
                .strip_prefix("semantic-type:")
                .ok_or("malformed output type id")?
                .parse::<u32>()
                .map_err(|error| error.to_string())?;
            values.push(af_semantic::ValueSlot {
                id: slot,
                type_id: type_id.into(),
            });
        }
    }
    let mut root_output_values = BTreeMap::<u32, ValueSlotId>::new();
    for output in input["parent_outputs"].as_array().into_iter().flatten() {
        let alias = output["slot"]
            .as_str()
            .ok_or("parent output lacks a value-slot alias")?;
        let raw = alias
            .rsplit(':')
            .next()
            .ok_or("malformed parent output value-slot alias")?
            .parse::<u32>()
            .map_err(|error| error.to_string())?;
        let slot = ValueSlotId::from_raw(raw);
        let type_id = output["type_id"]
            .as_str()
            .unwrap()
            .strip_prefix("semantic-type:")
            .ok_or("malformed parent output type id")?
            .parse::<u32>()
            .map_err(|error| error.to_string())?;
        values.push(af_semantic::ValueSlot {
            id: slot,
            type_id: type_id.into(),
        });
        root_output_values.insert(fixture_u32(&output["port"], "parent output port"), slot);
    }
    for child in children {
        let raw = raws
            .get(child["contract_digest"].as_str().unwrap())
            .unwrap();
        for port in &raw.inputs {
            component_inputs
                .entry(port.port_no)
                .or_insert((10 + port.port_no, 10 + port.port_no));
        }
    }
    for (port, (slot, expression)) in &component_inputs {
        let type_id = input_type_ids.get(port).copied().unwrap_or(1);
        values.push(af_semantic::ValueSlot {
            id: (*slot).into(),
            type_id: type_id.into(),
        });
        expressions.push(ExpressionDef {
            id: (*expression).into(),
            result_type_id: type_id.into(),
            body: input
                .get("edge")
                .filter(|edge| {
                    edge["consumer_port"] == *port
                        && children
                            .iter()
                            .any(|child| child["node"] == edge["consumer_child"])
                })
                .and_then(|edge| output_value_slots.get(edge["slot"].as_str()?))
                .copied()
                .map_or_else(
                    || ExpressionBody::ValueSlot((*slot).into()),
                    ExpressionBody::ValueSlot,
                ),
        });
        run_parameters.push(RunParameterDecl {
            id: (*port).into(),
            name: format!("input-{port}"),
            type_id: type_id.into(),
            value_slot: (*slot).into(),
        });
    }
    if is_select {
        let exact_child = input.get("exact_child").and_then(Value::as_u64);
        if let Some(index) = exact_child {
            expressions.push(ExpressionDef {
                id: 1.into(),
                result_type_id: 2.into(),
                body: ExpressionBody::Constant(PrimitiveConstant::Int64(
                    i64::try_from(index).expect("SELECT case index fits i64") + 1,
                )),
            });
        } else {
            values.push(af_semantic::ValueSlot {
                id: 1.into(),
                type_id: 2.into(),
            });
            expressions.push(ExpressionDef {
                id: 1.into(),
                result_type_id: 2.into(),
                body: ExpressionBody::ValueSlot(1.into()),
            });
            run_parameters.push(RunParameterDecl {
                id: 100.into(),
                name: "selector".into(),
                type_id: 2.into(),
                value_slot: 1.into(),
            });
        }
    } else if is_timeout {
        if input["timeout_dynamic"].as_bool() == Some(true) {
            values.push(af_semantic::ValueSlot {
                id: 201.into(),
                type_id: 2.into(),
            });
            expressions.push(ExpressionDef {
                id: 1.into(),
                result_type_id: 2.into(),
                body: ExpressionBody::ValueSlot(201.into()),
            });
            run_parameters.push(RunParameterDecl {
                id: 201.into(),
                name: "timeout".into(),
                type_id: 2.into(),
                value_slot: 201.into(),
            });
        } else {
            expressions.push(ExpressionDef {
                id: 1.into(),
                result_type_id: 2.into(),
                body: ExpressionBody::Constant(PrimitiveConstant::Duration(
                    af_core::Duration::new(input["timeout_micros"].as_i64().unwrap_or(1)),
                )),
            });
        }
    } else if is_foreach {
        let item_count = input["item_count"].as_u64().unwrap_or(0);
        if input["item_count"].is_null() {
            values.push(af_semantic::ValueSlot {
                id: 200.into(),
                type_id: 3.into(),
            });
            expressions.push(ExpressionDef {
                id: 1.into(),
                result_type_id: 3.into(),
                body: ExpressionBody::ValueSlot(200.into()),
            });
            run_parameters.push(RunParameterDecl {
                id: 200.into(),
                name: "items".into(),
                type_id: 3.into(),
                value_slot: 200.into(),
            });
        } else {
            let element_expressions = (0..item_count)
                .map(|index| {
                    let expression = u32::try_from(1000_u64 + index)
                        .expect("fixture list expression IDs fit u32");
                    expressions.push(ExpressionDef {
                        id: expression.into(),
                        result_type_id: 2.into(),
                        body: ExpressionBody::Constant(PrimitiveConstant::Int64(
                            i64::try_from(index).expect("fixture list element fits i64"),
                        )),
                    });
                    expression.into()
                })
                .collect::<Vec<_>>();
            expressions.push(ExpressionDef {
                id: 1.into(),
                result_type_id: 3.into(),
                body: ExpressionBody::ListConstruct {
                    element_type: 2.into(),
                    elements: element_expressions,
                },
            });
        }
        values.push(af_semantic::ValueSlot {
            id: 40_000.into(),
            type_id: 2.into(),
        });
    } else if is_while {
        expressions.push(ExpressionDef {
            id: 1.into(),
            result_type_id: 2.into(),
            body: ExpressionBody::Constant(PrimitiveConstant::Bool(
                input["guard_value"].as_bool().unwrap_or(true),
            )),
        });
    }
    if has_nested_foreach && !is_foreach {
        for nested in input["nested_nodes"].as_array().into_iter().flatten() {
            if nested["operator"] != "iterate_foreach" {
                continue;
            }
            let item_count = fixture_u32(&nested["item_count"], "nested foreach item_count");
            let source_expression = fixture_u32(
                &nested["source_expression"],
                "nested foreach source expression",
            );
            let item_slot = fixture_u32(&nested["item_slot"], "nested foreach item slot");
            let element_expressions = (0..item_count)
                .map(|index| {
                    let expression = 30_000_u32
                        .checked_add(index)
                        .expect("nested foreach expression IDs fit u32");
                    expressions.push(ExpressionDef {
                        id: expression.into(),
                        result_type_id: 2.into(),
                        body: ExpressionBody::Constant(PrimitiveConstant::Int64(i64::from(index))),
                    });
                    expression.into()
                })
                .collect::<Vec<_>>();
            expressions.push(ExpressionDef {
                id: source_expression.into(),
                result_type_id: 3.into(),
                body: ExpressionBody::ListConstruct {
                    element_type: 2.into(),
                    elements: element_expressions,
                },
            });
            values.push(af_semantic::ValueSlot {
                id: item_slot.into(),
                type_id: 2.into(),
            });
        }
    }
    let carry_bindings = input["carry"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|carry| {
            let slot = fixture_u32(&carry["slot"], "carry slot");
            let initial_slot = fixture_u32(&carry["initial_slot"], "carry initial slot");
            let type_id = carry["type_id"]
                .as_str()
                .unwrap()
                .strip_prefix("semantic-type:")
                .ok_or("malformed carry type id")?
                .parse::<u32>()
                .map_err(|error| error.to_string())?;
            let initial_expression =
                fixture_u32(&carry["initial_expression"], "carry initial expression");
            values.push(af_semantic::ValueSlot {
                id: slot.into(),
                type_id: type_id.into(),
            });
            values.push(af_semantic::ValueSlot {
                id: initial_slot.into(),
                type_id: type_id.into(),
            });
            expressions.push(ExpressionDef {
                id: initial_expression.into(),
                result_type_id: type_id.into(),
                body: ExpressionBody::ValueSlot(initial_slot.into()),
            });
            run_parameters.push(RunParameterDecl {
                id: initial_slot.into(),
                name: format!("carry-initial-{slot}"),
                type_id: type_id.into(),
                value_slot: initial_slot.into(),
            });
            Ok(LoopCarryBinding {
                slot: slot.into(),
                initial: initial_expression.into(),
                body_output_port: fixture_u32(&carry["body_output_port"], "carry body output port")
                    .into(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let await_children = children
        .iter()
        .filter(|child| child["leaf_kind"] == "await")
        .collect::<Vec<_>>();
    if !await_children.is_empty() {
        type_nodes.push(SemanticTypeNode {
            id: 90_000.into(),
            body: SemanticTypeNodeBody::Primitive(SemanticPrimitiveType::String),
        });
        for child in &await_children {
            let node = fixture_u32(&child["node"], "Await computation id");
            let value_slot = 70_000_u32
                .checked_add(node)
                .ok_or("Await correlation ValueSlotId overflow")?;
            let expression = 80_000_u32
                .checked_add(node)
                .ok_or("Await correlation ExpressionId overflow")?;
            values.push(af_semantic::ValueSlot {
                id: value_slot.into(),
                type_id: 90_000.into(),
            });
            expressions.push(ExpressionDef {
                id: expression.into(),
                result_type_id: 90_000.into(),
                body: ExpressionBody::ValueSlot(value_slot.into()),
            });
            run_parameters.push(RunParameterDecl {
                id: value_slot.into(),
                name: format!("await-correlation-{node}"),
                type_id: 90_000.into(),
                value_slot: value_slot.into(),
            });
        }
    }

    let mut resource_alias_ids = BTreeMap::<String, ResourceSlotId>::new();
    for child in children {
        let mut aliases = BTreeSet::new();
        if let Some(resources) = child["resources"].as_array() {
            for resource in resources {
                let alias = resource["resource_slot"].as_str().unwrap().to_owned();
                aliases.insert((
                    fixture_u32(&resource["component_port"], "component port"),
                    alias,
                ));
            }
        }
        if let Some(effects) = child.get("effects") {
            for entry in entries(&parse_effects(effects)?) {
                let port = entry.atom.resource_selector.requirement_port_no;
                if !aliases.iter().any(|(mapped_port, _)| *mapped_port == port) {
                    aliases.insert((
                        port,
                        format!("fixture:auto:{}:{port}", child["node"].as_u64().unwrap()),
                    ));
                }
            }
        }
        for (_, alias) in aliases {
            let next = fixture_index(resource_alias_ids.len()) + 1;
            resource_alias_ids
                .entry(alias)
                .or_insert(ResourceSlotId::from_raw(next));
        }
    }
    let scope_resource_names = input["scope_resources"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .chain(
            input["additional_resources"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str),
        )
        .chain(
            input["additional_resources"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str),
        )
        .chain(
            input["nested_nodes"]
                .as_array()
                .into_iter()
                .flatten()
                .flat_map(|nested| nested["resources"].as_array().into_iter().flatten())
                .chain(
                    input["nested_nodes"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .flat_map(|nested| {
                            nested["scope_resources"].as_array().into_iter().flatten()
                        }),
                )
                .filter_map(Value::as_str),
        )
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    for alias in scope_resource_names {
        let next = fixture_index(resource_alias_ids.len()) + 1;
        resource_alias_ids
            .entry(alias)
            .or_insert(ResourceSlotId::from_raw(next));
    }
    let mut resource_slots = Vec::new();
    for (alias, slot) in &resource_alias_ids {
        resource_slots.push(af_semantic::ResourceSlot {
            id: *slot,
            requirement_type: env.resource_type().schema.clone(),
        });
        let _ = alias;
    }
    let mut effective_refs = Vec::<(DigestRefValue, ContractRefKind)>::new();
    for child in children {
        let node = fixture_u32(&child["node"], "node id");
        let digest = if let Some(target) = subflow_targets.get(&node) {
            (target.clone(), ContractRefKind::Subflow)
        } else {
            let alias = child["contract_digest"].as_str().unwrap();
            (
                pin_by_alias
                    .get(alias)
                    .ok_or("no checked component pin for child alias")?
                    .clone(),
                ContractRefKind::Component,
            )
        };
        effective_refs.push(digest);
    }
    effective_refs.sort_by(|left, right| {
        let kind_order = |kind: &ContractRefKind| match kind {
            ContractRefKind::Component => 0_u8,
            ContractRefKind::Subflow => 1_u8,
        };
        (&left.0.domain, &left.0.hex, kind_order(&left.1)).cmp(&(
            &right.0.domain,
            &right.0.hex,
            kind_order(&right.1),
        ))
    });
    effective_refs.dedup_by(|left, right| left.0 == right.0 && left.1 == right.1);
    let ref_key = |digest: &DigestRefValue| format!("{}/{}", digest.domain, digest.hex);
    let refs = effective_refs
        .iter()
        .enumerate()
        .map(|(index, (digest, _))| {
            (
                ref_key(digest),
                ContractRefId::from_raw(fixture_index(index) + 1),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let contracts = effective_refs
        .iter()
        .enumerate()
        .map(|(index, (digest, kind))| ContractRefEntry {
            id: (fixture_index(index) + 1).into(),
            kind: *kind,
            digest: digest.clone(),
        })
        .collect::<Vec<_>>();
    let root_id = ComputationId::from_raw(1);
    let child_ids = children
        .iter()
        .map(|c| ComputationId::from_raw(fixture_u32(&c["node"], "node id")))
        .collect::<Vec<_>>();
    let root_child_ids = input["root_child_nodes"].as_array().map_or_else(
        || child_ids.clone(),
        |nodes| {
            nodes
                .iter()
                .map(|node| ComputationId::from_raw(fixture_u32(node, "root child node")))
                .collect::<Vec<_>>()
        },
    );
    let operator = input["operator"].as_str().unwrap();
    let root_body = match operator {
        "sequence" => ComputationBody::Sequence(SequenceBody {
            children: root_child_ids.clone(),
        }),
        "select" => ComputationBody::Select(SelectBody {
            selector: 1.into(),
            cases: root_child_ids
                .iter()
                .enumerate()
                .map(|(index, child)| SelectCase {
                    ordinal: (fixture_index(index) + 1).into(),
                    child: *child,
                })
                .collect(),
        }),
        "parallel_all" | "parallel_fail_fast" => ComputationBody::Parallel(ParallelBody {
            mode: if operator == "parallel_all" {
                ParallelMode::All
            } else {
                ParallelMode::FailFast
            },
            branches: root_child_ids
                .iter()
                .enumerate()
                .map(|(index, child)| ParallelBranch {
                    ordinal: (fixture_index(index) + 1).into(),
                    child: *child,
                })
                .collect(),
            max_inflight: input
                .get("max_inflight")
                .and_then(Value::as_u64)
                .map(|n| u32::try_from(n).expect("fixture max_inflight fits u32")),
            output_bindings: vec![],
        }),
        "race" => ComputationBody::Race(RaceBody {
            branches: root_child_ids
                .iter()
                .enumerate()
                .map(|(index, child)| RaceBranch {
                    ordinal: (fixture_index(index) + 1).into(),
                    child: *child,
                })
                .collect(),
        }),
        "timeout" => ComputationBody::Timeout(TimeoutBody {
            child: *root_child_ids.first().ok_or("TIMEOUT needs one child")?,
            timeout: 1.into(),
        }),
        "iterate_foreach" => ComputationBody::Iterate(IterateBody::Foreach(ForeachBody {
            source: 1.into(),
            item_slot: 40_000.into(),
            body: *root_child_ids
                .first()
                .ok_or("ITERATE FOREACH needs one body")?,
            carry: carry_bindings,
            max_inflight: input["max_inflight"]
                .as_u64()
                .map(|value| u32::try_from(value).expect("fixture max_inflight fits u32")),
        })),
        "iterate_while" => ComputationBody::Iterate(IterateBody::While(WhileBody {
            guard: 1.into(),
            body: *root_child_ids
                .first()
                .ok_or("ITERATE WHILE needs one body")?,
            carry: carry_bindings,
            max_iterations: fixture_u32(&input["max_iterations"], "max_iterations"),
        })),
        "retry" => ComputationBody::Retry(RetryBody {
            child: *root_child_ids.first().ok_or("RETRY needs one child")?,
            delays: input["delays_micros"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|delay| af_core::Duration::new(delay.as_i64().unwrap()))
                .collect(),
        }),
        "handle" => {
            let clauses = input["clauses"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|clause| {
                    let matcher = if clause["match"] == "timed_out" {
                        HandleMatch::TimedOut(())
                    } else {
                        let failure_name = clause["match"]["failure"].as_str().unwrap();
                        HandleMatch::Failure(FailureMatch {
                            failure_type: failure_ref(failure_name, &env.schemas)?.payload_type,
                        })
                    };
                    Ok(HandleClause {
                        ordinal: fixture_u32(&clause["ordinal"], "handler ordinal").into(),
                        matcher,
                        handler: ComputationId::from_raw(fixture_u32(
                            &clause["handler"],
                            "handler node",
                        )),
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            ComputationBody::Handle(HandleBody {
                child: *root_child_ids
                    .first()
                    .ok_or("HANDLE needs a protected child")?,
                clauses,
            })
        }
        "scope" => {
            let mut resources = scope_resource_ids(
                input["scope_resources"].as_array().into_iter().flatten(),
                &resource_alias_ids,
            )?;
            resources.sort_unstable();
            ComputationBody::Scope(ScopeBody {
                child: *root_child_ids.first().ok_or("SCOPE needs one child")?,
                resources,
            })
        }
        "compensation_scope" => ComputationBody::CompensationScope(af_ir::CompensationScopeBody {
            child: *root_child_ids
                .first()
                .ok_or("COMPENSATION_SCOPE needs one child")?,
        }),
        other => return Err(format!("unsupported fixture opcode {other}")),
    };
    let root_outputs = input["parent_outputs"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|output| {
            let port = fixture_u32(&output["port"], "parent output port");
            ComputationOutputBinding {
                port_no: port.into(),
                value_slot: root_output_values[&port],
            }
        })
        .collect();
    let mut computations = vec![ComputationDef {
        id: root_id,
        outputs: root_outputs,
        body: root_body,
    }];
    for child in children {
        let node = ComputationId::from_raw(fixture_u32(&child["node"], "node id"));
        let alias = child["contract_digest"].as_str().unwrap();
        let raw = raws.get(alias).unwrap();
        let subflow_target = subflow_targets.get(&node.raw());
        let pin = if let Some(target) = subflow_target {
            target
        } else {
            pin_by_alias
                .get(alias)
                .ok_or("no checked contract pin for child alias")?
        };
        let input_bindings = raw
            .inputs
            .iter()
            .map(|port| {
                let expression = component_inputs
                    .get(&port.port_no)
                    .map(|(_, expression)| *expression)
                    .ok_or_else(|| {
                        format!(
                            "missing input expression for component port {}",
                            port.port_no
                        )
                    })?;
                Ok(InputBinding {
                    port_no: port.port_no.into(),
                    expression: expression.into(),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let mut mapped_resources = BTreeMap::<u32, String>::new();
        if let Some(resources) = child["resources"].as_array() {
            for resource in resources {
                mapped_resources.insert(
                    fixture_u32(&resource["component_port"], "component port"),
                    resource["resource_slot"].as_str().unwrap().to_owned(),
                );
            }
        }
        if let Some(effects) = child.get("effects") {
            for entry in entries(&parse_effects(effects)?) {
                let port = entry.atom.resource_selector.requirement_port_no;
                mapped_resources.entry(port).or_insert_with(|| {
                    format!("fixture:auto:{}:{port}", child["node"].as_u64().unwrap())
                });
            }
        }
        let resource_bindings = mapped_resources
            .into_iter()
            .map(|(port, alias)| {
                let slot = resource_alias_ids
                    .get(&alias)
                    .copied()
                    .ok_or_else(|| format!("unmaterialized resource slot {alias}"))?;
                Ok(ResourceUseBinding {
                    port_no: port.into(),
                    resource_slot: slot,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let contract_ref = *refs
            .get(&ref_key(pin))
            .ok_or("contract pin not entered in IR")?;
        let body = if child["leaf_kind"] == "await" {
            let node = fixture_u32(&child["node"], "Await computation id");
            let expression = 80_000_u32
                .checked_add(node)
                .ok_or("Await correlation ExpressionId overflow")?;
            ComputationBody::Await(AwaitBody {
                signal_type: env
                    .schemas
                    .get("agentflow.semantic.core@1/TypeSchemaRef")
                    .ok_or("Await signal schema is unavailable")?
                    .reference(),
                correlation: expression.into(),
            })
        } else if subflow_target.is_some() {
            ComputationBody::CallSubflow(CallSubflowBody {
                contract: contract_ref,
                inputs: input_bindings,
                resources: resource_bindings,
            })
        } else {
            ComputationBody::Invoke(InvokeBody {
                contract: contract_ref,
                inputs: input_bindings,
                resources: resource_bindings,
            })
        };
        let output_bindings = child["outputs"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|output| {
                let alias = output["slot"].as_str().unwrap();
                ComputationOutputBinding {
                    port_no: fixture_u32(&output["port"], "output port").into(),
                    value_slot: output_value_slots[alias],
                }
            })
            .collect();
        computations.push(ComputationDef {
            id: node,
            outputs: output_bindings,
            body,
        });
    }
    for nested in input["nested_nodes"].as_array().into_iter().flatten() {
        let node = ComputationId::from_raw(fixture_u32(&nested["node"], "nested node"));
        let body = match nested["operator"].as_str().unwrap() {
            "iterate_foreach" => ComputationBody::Iterate(IterateBody::Foreach(ForeachBody {
                source: fixture_u32(&nested["source_expression"], "nested source expression")
                    .into(),
                item_slot: fixture_u32(&nested["item_slot"], "nested item slot").into(),
                body: ComputationId::from_raw(fixture_u32(&nested["body"], "nested body")),
                carry: vec![],
                max_inflight: nested["max_inflight"]
                    .as_u64()
                    .map(|value| u32::try_from(value).expect("nested max_inflight fits u32")),
            })),
            "sequence" => ComputationBody::Sequence(SequenceBody {
                children: nested["children"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|child| {
                        ComputationId::from_raw(fixture_u32(child, "nested sequence child"))
                    })
                    .collect(),
            }),
            "parallel_all" | "parallel_fail_fast" => ComputationBody::Parallel(ParallelBody {
                mode: if nested["operator"] == "parallel_all" {
                    ParallelMode::All
                } else {
                    ParallelMode::FailFast
                },
                branches: nested["children"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .enumerate()
                    .map(|(index, child)| ParallelBranch {
                        ordinal: (fixture_index(index) + 1).into(),
                        child: ComputationId::from_raw(fixture_u32(child, "nested parallel child")),
                    })
                    .collect(),
                max_inflight: nested["max_inflight"]
                    .as_u64()
                    .map(|value| u32::try_from(value).expect("nested max_inflight fits u32")),
                output_bindings: vec![],
            }),
            "race" => ComputationBody::Race(RaceBody {
                branches: nested["children"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .enumerate()
                    .map(|(index, child)| RaceBranch {
                        ordinal: (fixture_index(index) + 1).into(),
                        child: ComputationId::from_raw(fixture_u32(child, "nested race child")),
                    })
                    .collect(),
            }),
            "retry" => ComputationBody::Retry(RetryBody {
                child: ComputationId::from_raw(fixture_u32(&nested["child"], "nested retry child")),
                delays: nested["delays_micros"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|delay| af_core::Duration::new(delay.as_i64().unwrap()))
                    .collect(),
            }),
            "scope" => {
                let mut resources = scope_resource_ids(
                    nested["scope_resources"].as_array().into_iter().flatten(),
                    &resource_alias_ids,
                )?;
                resources.sort_unstable();
                ComputationBody::Scope(ScopeBody {
                    child: ComputationId::from_raw(fixture_u32(
                        &nested["child"],
                        "nested Scope child",
                    )),
                    resources,
                })
            }
            "compensation_scope" => {
                ComputationBody::CompensationScope(af_ir::CompensationScopeBody {
                    child: ComputationId::from_raw(fixture_u32(
                        &nested["child"],
                        "nested CompensationScope child",
                    )),
                })
            }
            "handle" => {
                let clauses = nested["clauses"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|clause| {
                        let matcher = if clause["match"] == "timed_out" {
                            HandleMatch::TimedOut(())
                        } else {
                            let alias = clause["match"]["failure"].as_str().unwrap();
                            HandleMatch::Failure(FailureMatch {
                                failure_type: failure_ref(alias, &env.schemas)?.payload_type,
                            })
                        };
                        Ok(HandleClause {
                            ordinal: fixture_u32(&clause["ordinal"], "nested handler ordinal")
                                .into(),
                            matcher,
                            handler: ComputationId::from_raw(fixture_u32(
                                &clause["handler"],
                                "nested handler node",
                            )),
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                ComputationBody::Handle(HandleBody {
                    child: ComputationId::from_raw(fixture_u32(
                        &nested["child"],
                        "nested Handle child",
                    )),
                    clauses,
                })
            }
            other => return Err(format!("unsupported nested fixture opcode {other}")),
        };
        computations.push(ComputationDef {
            id: node,
            outputs: vec![],
            body,
        });
    }
    Ok((
        SemanticIrDocument {
            semantic_refs: vec![],
            types: type_nodes,
            values,
            expressions,
            resources: resource_slots,
            contracts,
            run_parameters,
            workflow_outputs: vec![],
            computations,
            root: root_id,
        },
        resource_alias_ids,
    ))
}

fn project(
    summary: &DerivedControlContractSummary<'_, '_>,
    input: &Value,
    pins: &BTreeMap<String, String>,
    resource_alias_ids: &BTreeMap<String, ResourceSlotId>,
) -> ControlConformanceProjection {
    let effects = match summary.effects().value() {
        ControlDimensionValue::Complete(signature) => effect_json(signature),
        ControlDimensionValue::Partial {
            known_contribution, ..
        } => json!({
            "row":"unknown",
            "known_contributions":known_contribution.as_ref().map(effect_json).into_iter().collect::<Vec<_>>(),
        }),
        ControlDimensionValue::Unknown {
            known_contributions,
            ..
        } => json!({
            "row":"unknown",
            "known_contributions":known_contributions.iter().map(|item|effect_json(item.value())).collect::<Vec<_>>(),
        }),
        ControlDimensionValue::PerChild { .. } | ControlDimensionValue::NotApplicable => {
            json!({"row":"unknown"})
        }
    };
    let effect_occurrences = summary.effect_occurrences().iter().map(|occurrence| json!({
        "atom": atom_json(&occurrence.declared.atom),
        "caller_digest": "semantic-ir:caller",
        "contract_digest": pins.get(&occurrence.source_contract_digest.hex).cloned().unwrap_or_else(|| occurrence.source_contract_digest.hex.clone()),
        "entry_ordinal": occurrence.source_entry_ordinal,
        "node": occurrence.computation.raw(),
    })).collect::<Vec<_>>();
    let effects_complete = effects["row"] == "closed";
    let failures = match summary.failures().value() {
        ControlDimensionValue::Complete(row) => {
            let mut names = row.failures.iter().map(failure_alias).collect::<Vec<_>>();
            names.sort();
            Some(names)
        }
        _ => None,
    };
    let usage = match summary.usage().value() {
        ControlDimensionValue::Complete(envelope) => usage_json(envelope),
        ControlDimensionValue::Partial {
            known_contribution: _,
            ..
        }
        | ControlDimensionValue::Unknown { .. } => json!({
            "state":"unknown",
            "known_contributions": usage_known_contributions(summary),
        }),
        _ => json!({"state":"unknown","coordinates":{}}),
    };
    let flow = match summary.information_flow().value() {
        ControlDimensionValue::Complete(flow)
        | ControlDimensionValue::Partial {
            known_contribution: Some(flow),
            ..
        } => flow_json(
            flow,
            summary.information_flow().proofs().first().map_or(
                LeafVerdict::Unproven,
                af_computation::ControlDimensionProof::verdict,
            ),
        ),
        _ => {
            json!({"sources":[],"taints":[],"may_carry_authority":"none","effective_authority_inferred":false,"influence_unknown":true,"decision_dependencies":[],"constraints_by_child":[],"verdict":"unproven"})
        }
    };
    let resources_by_child = resources_json(summary, resource_alias_ids);
    let execution_by_child = execution_json(summary);
    let recovery_sources = recovery_json(summary, pins);
    let flow_sources = flow_sources_json(summary, pins, resource_alias_ids);
    let resource_aliases = resource_aliases_json(summary, resource_alias_ids);
    let resource_requirements = resource_requirements_json(summary, resource_alias_ids);
    let recovery_sources_full = recovery_sources_full_json(summary, pins);
    let execution_by_child_full = execution_full_json(summary);
    let proof_dimensions = summary
        .proof()
        .dimensions()
        .iter()
        .map(|proof| {
            json!({
                "dimension": proof.dimension(),
                "verdict": verdict(proof.verdict()),
                "rule_id": proof.rule_id(),
                "source_node": proof.source_node().raw(),
                "reference": proof.reference(),
                "prerequisites": proof.prerequisites(),
            })
        })
        .collect::<Vec<_>>();
    let unresolved_children = match summary.effects().value() {
        ControlDimensionValue::Partial {
            unresolved_sources, ..
        }
        | ControlDimensionValue::Unknown {
            unresolved_sources, ..
        }
        | ControlDimensionValue::PerChild {
            unresolved_sources, ..
        } => unresolved_sources
            .iter()
            .map(|node| node.raw())
            .collect::<Vec<_>>(),
        _ => vec![],
    };
    let mut semantic_fields = json!({
        "effect_lower_bound_adjusted_children": summary.effect_lower_bound_adjustments().iter().map(|item| item.source_node().raw()).collect::<Vec<_>>(),
        "effect_occurrences": effect_occurrences,
        "effect_row_complete": effects_complete,
        "effects": effects,
        "execution_by_child": execution_by_child,
        "failures": failures,
        "flow": flow,
        "recovery_sources": recovery_sources,
        "resources_by_child": resources_by_child,
        "unresolved_children": unresolved_children,
        "usage": usage,
        "verdict": verdict(summary.proof().overall()),
        "identity": {"operator": summary.operator().name(), "node": summary.control_node().raw(), "children": summary.direct_children().iter().map(|id| id.raw()).collect::<Vec<_>>()},
        "sequence_dataflow_dependencies": summary.sequence_dataflow_dependencies().iter().map(|dependency|json!({"producer_child":dependency.producer_child().raw(),"producer_node":dependency.producer_node().raw(),"value_slot":dependency.value_slot().raw(),"consumer_child":dependency.consumer_child().raw(),"consumer_node":dependency.consumer_node().raw(),"expression":dependency.expression().raw()})).collect::<Vec<_>>(),
        "case_operator": input["operator"],
        "control_semantics": control_semantics_json(summary.operator()),
        "control_decision_dependencies": match summary.information_flow().value() {
            ControlDimensionValue::Complete(flow) | ControlDimensionValue::Partial { known_contribution: Some(flow), .. } => flow.control_decision_dependencies().iter().map(|slot|slot.raw()).collect::<Vec<_>>(),
            _ => Vec::<u32>::new(),
        },
        "control_decision_influence_unknown": match summary.information_flow().value() {
            ControlDimensionValue::Complete(flow) | ControlDimensionValue::Partial { known_contribution: Some(flow), .. } => flow.control_decision_influence_unknown(),
            _ => true,
        },
        "dimension_availability": dimension_availability(summary),
        "typed_outputs": match summary.interface().outputs() {
            ControlDimensionValue::Complete(ports) => json!({"state":"complete","port_count":ports.len()}),
            ControlDimensionValue::Partial { known_contribution, unresolved_sources } => json!({"state":"partial","port_count":known_contribution.as_ref().map_or(0,Vec::len),"unresolved_sources":unresolved_sources.iter().map(|node|node.raw()).collect::<Vec<_>>()}),
            ControlDimensionValue::Unknown { unresolved_sources, .. } => json!({"state":"unknown","unresolved_sources":unresolved_sources.iter().map(|node|node.raw()).collect::<Vec<_>>()}),
            ControlDimensionValue::PerChild { unresolved_sources, .. } => json!({"state":"per_child","unresolved_sources":unresolved_sources.iter().map(|node|node.raw()).collect::<Vec<_>>()}),
            ControlDimensionValue::NotApplicable => json!({"state":"not_applicable"}),
        },
        "output_interface": interface_json(summary)["outputs"].clone(),
        "control_effect_families": summary.effect_occurrences().iter().enumerate().filter(|(_,occurrence)|!occurrence.control_families.is_empty()).map(|(occurrence_index,occurrence)|json!({"occurrence_index":occurrence_index,"source_node":occurrence.computation.raw(),"families":occurrence.control_families.iter().map(|family|json!({"control_node":family.control_node.raw(),"kind":match family.kind {af_computation::EffectOccurrenceFamilyKind::Iterate=>"iterate",af_computation::EffectOccurrenceFamilyKind::Retry=>"retry"},"logical_repetitions":{"lower":family.logical_repetitions.lower,"upper":occurrence_ceiling_json(family.logical_repetitions.upper)},"physical_attempts":family.physical_attempts.as_ref().map(|bounds|json!({"lower":bounds.lower,"upper":occurrence_ceiling_json(bounds.upper)}))})).collect::<Vec<_>>()})).collect::<Vec<_>>(),
        "_inherited_obligations": summary.inherited_obligations().iter().map(|item|json!({"source_node":item.source_node().raw(),"dimension":item.dimension(),"verdict":verdict(item.verdict()),"rule_id":item.rule_id(),"reference":item.reference()})).collect::<Vec<_>>(),
        "_semantic_projection": {
            "source_semantic_ir_digest": digest_ref_json(summary.source_semantic_ir_digest()),
            "control_node": summary.control_node().raw(),
            "operator": summary.operator().name(),
            "direct_children_in_semantic_order": summary.direct_children().iter().map(|id|id.raw()).collect::<Vec<_>>(),
        "interface": interface_json(summary),
            "effects": effects,
            "effect_occurrences": effect_occurrences,
        "effect_occurrences_full": summary.effect_occurrences().iter().map(|occurrence|json!({"caller_semantic_ir_digest":digest_ref_json(&occurrence.caller_semantic_ir_digest),"source_contract_digest":pins.get(&occurrence.source_contract_digest.hex).cloned().unwrap_or_else(||occurrence.source_contract_digest.hex.clone()),"source_component":{"catalog":occurrence.source_component.catalog,"name":occurrence.source_component.name},"computation":occurrence.computation.raw(),"source_entry_ordinal":occurrence.source_entry_ordinal,"declared":entry_json(&occurrence.declared),"mapped_resource_slot":occurrence.mapped_resource_slot.map(|slot|resource_alias_name(slot,resource_alias_ids))})).collect::<Vec<_>>(),
            "effect_residuals": summary.effect_residuals().iter().map(|item|json!({"caller_digest":"semantic-ir:caller","contract_digest":pins.get(&item.source_contract_digest.hex).cloned().unwrap_or_else(||item.source_contract_digest.hex.clone()),"node":item.computation.raw(),"entry_ordinal":item.source_entry_ordinal,"residual":residual_json(&item.residual),"mapped_resource_slot":item.mapped_resource_slot.map(|slot|resource_alias_ids.iter().find(|(_,value)|**value==slot).map_or_else(||format!("resource-slot:{}",slot.raw()),|(alias,_)|alias.clone()))})).collect::<Vec<_>>(),
            "failures": failures,
            "failure_types": failure_types_json(summary),
        "failure_sources_by_child": summary.children().iter().map(|child|json!({"source_node":child.control_node().raw(),"failures":failure_types_json(child)})).collect::<Vec<_>>(),
            "usage": usage,
        "usage_by_child": summary.children().iter().map(|child|json!({"source_node":child.control_node().raw(),"usage":match child.usage().value() { ControlDimensionValue::Complete(envelope)=>usage_json(envelope),ControlDimensionValue::Partial{known_contribution,..}=>known_contribution.as_ref().map(usage_json).unwrap_or(json!({"state":"unknown","coordinates":{}})),_=>json!({"state":"unknown","coordinates":{}}) }})).collect::<Vec<_>>(),
            "information_flow": flow,
            "flow_contract_sources": flow_sources,
            "recovery_sources": recovery_sources_full,
            "resource_requirements_by_occurrence": resource_requirements,
            "resource_aliases": resource_aliases,
            "execution_by_child": execution_by_child_full,
            "dimension_availability": dimension_availability(summary),
            "per_dimension_proofs": proof_dimensions,
            "inherited_obligations": summary.inherited_obligations().iter().map(|item|json!({"source_node":item.source_node().raw(),"dimension":item.dimension(),"verdict":verdict(item.verdict()),"rule_id":item.rule_id(),"reference":item.reference()})).collect::<Vec<_>>(),
        },
    });
    let scope_boundaries = scope_boundaries_json(summary);
    let compensation_scopes = compensation_scopes_json(summary, pins, resource_alias_ids);
    if input["include_scope_details"] == true
        || !scope_boundaries.is_empty()
        || !compensation_scopes.is_empty()
    {
        semantic_fields["scope_boundary"] = summary
            .scope_boundary()
            .map_or(Value::Null, scope_boundary_json);
        semantic_fields["compensation_scope"] =
            summary.compensation_scope().map_or(Value::Null, |scope| {
                compensation_scope_json(scope, pins, resource_alias_ids)
            });
        semantic_fields["scope_boundaries_by_node"] = json!(scope_boundaries);
        semantic_fields["compensation_scopes_by_node"] = json!(compensation_scopes);
    }
    ControlConformanceProjection { semantic_fields }
}

fn occurrence_ceiling_json(ceiling: af_effect::OccurrenceCeiling) -> Value {
    match ceiling {
        af_effect::OccurrenceCeiling::Finite(value) => json!(value),
        af_effect::OccurrenceCeiling::Unbounded => json!("unbounded"),
    }
}

fn control_semantics_json(operator: &ControlOperator) -> Value {
    match operator {
        ControlOperator::Iterate {
            mode,
            effect_repetitions,
            usage_repetitions,
        } => {
            let mode = match mode {
                IterateMode::Foreach {
                    statically_known_item_count,
                    max_inflight,
                } => json!({
                    "kind":"foreach",
                    "statically_known_item_count":statically_known_item_count,
                    "max_inflight":max_inflight,
                }),
                IterateMode::While { max_iterations } => json!({
                    "kind":"while",
                    "max_iterations":max_iterations,
                }),
            };
            json!({
                "kind":"iterate",
                "mode":mode,
                "effect_repetitions":{"lower":effect_repetitions.lower,"upper":occurrence_ceiling_json(effect_repetitions.upper)},
                "usage_repetitions":usage_repetitions,
                "max_inflight_is_not_total_count":true,
            })
        }
        ControlOperator::Retry {
            delays,
            max_attempts,
        } => json!({
            "kind":"retry",
            "delays_micros_in_semantic_order":delays.iter().map(af_core::Duration::as_micros).collect::<Vec<_>>(),
            "maximum_attempts":max_attempts,
            "physical_attempt_bounds":{"lower":1,"upper":max_attempts},
            "logical_narrowing_proven":false,
            "retryable_failure_policy":"not_present_in_frozen_ir",
        }),
        ControlOperator::Timeout {
            expression,
            static_duration,
            possible_outcomes,
        } => json!({
            "kind":"timeout",
            "timeout_expression":expression.raw(),
            "static_duration_micros":static_duration.map(|duration|duration.as_micros()),
            "possible_outcomes":possible_outcomes.iter().map(|outcome|match outcome {af_computation::TimeoutOutcome::ChildCompleted=>"child_completed",af_computation::TimeoutOutcome::TimedOut=>"timed_out"}).collect::<Vec<_>>(),
            "timed_out_is_typed_failure":false,
            "successful_cancellation_inferred":false,
        }),
        ControlOperator::Handle { handled, clauses } => json!({
            "kind":"handle",
            "handled_failure_types":handled.failures.iter().map(failure_alias).collect::<Vec<_>>(),
            "clauses":clauses.iter().map(|clause|json!({
                "ordinal":clause.ordinal(),
                "matcher":match clause.matcher() {HandleMatcherSummary::Failure(schema)=>json!({"failure_type":schema.type_name}),HandleMatcherSummary::TimedOut=>json!({"timed_out":null})},
                "handler":clause.handler().raw(),
            })).collect::<Vec<_>>(),
            "handler_paths_are_conditional":true,
            "unmatched_failures_remain_in_residual":true,
        }),
        other => json!({"kind":other.name()}),
    }
}

fn scope_boundary_json(scope: &af_computation::ScopeBoundarySummary) -> Value {
    json!({
        "control_node":scope.control_node().raw(),
        "child":scope.child().raw(),
        "parent_visible_resources":scope.parent_visible_resources().iter().map(|slot|slot.raw()).collect::<Vec<_>>(),
        "declared_resources":scope.declared_resources().iter().map(|slot|slot.raw()).collect::<Vec<_>>(),
        "child_visible_resources":scope.child_visible_resources().iter().map(|slot|slot.raw()).collect::<Vec<_>>(),
        "resource_attenuation":verdict(scope.resource_attenuation()),
        "authority_attenuation":verdict(scope.authority_attenuation()),
        "budget_attenuation":verdict(scope.budget_attenuation()),
        "owned_children_settle_before_terminal":verdict(scope.owned_children_settle_before_terminal()),
    })
}

fn compensation_scope_json(
    scope: &af_computation::CompensationScopeSummary,
    pins: &BTreeMap<String, String>,
    resource_alias_ids: &BTreeMap<String, ResourceSlotId>,
) -> Value {
    let actions = match scope.potential_actions().value() {
        ControlDimensionValue::Complete(actions) => json!({
            "state":"complete",
            "actions":actions.iter().map(|action|conditional_compensation_action_json(action,pins,resource_alias_ids)).collect::<Vec<_>>(),
        }),
        ControlDimensionValue::Partial {
            known_contribution,
            unresolved_sources,
        } => json!({
            "state":"partial",
            "actions":known_contribution.as_ref().map(|actions|actions.iter().map(|action|conditional_compensation_action_json(action,pins,resource_alias_ids)).collect::<Vec<_>>()).unwrap_or_default(),
            "unresolved_sources":unresolved_sources.iter().map(|node|node.raw()).collect::<Vec<_>>(),
        }),
        ControlDimensionValue::Unknown {
            known_contributions,
            unresolved_sources,
        } => json!({
            "state":"unknown",
            "known_contributions":known_contributions.iter().flat_map(|item|item.value().iter().map(|action|conditional_compensation_action_json(action,pins,resource_alias_ids))).collect::<Vec<_>>(),
            "unresolved_sources":unresolved_sources.iter().map(|node|node.raw()).collect::<Vec<_>>(),
        }),
        ControlDimensionValue::PerChild {
            unresolved_sources, ..
        } => json!({
            "state":"per_child",
            "unresolved_sources":unresolved_sources.iter().map(|node|node.raw()).collect::<Vec<_>>(),
        }),
        ControlDimensionValue::NotApplicable => json!({"state":"not_applicable"}),
    };
    let local_actions = match scope.local_recovery_actions().value() {
        ControlDimensionValue::Complete(actions) => json!({
            "state":"complete",
            "actions":actions.iter().map(local_recovery_action_json).collect::<Vec<_>>(),
        }),
        ControlDimensionValue::Partial {
            known_contribution,
            unresolved_sources,
        } => json!({
            "state":"partial",
            "actions":known_contribution.as_ref().map(|actions|actions.iter().map(local_recovery_action_json).collect::<Vec<_>>()).unwrap_or_default(),
            "unresolved_sources":unresolved_sources.iter().map(|node|node.raw()).collect::<Vec<_>>(),
        }),
        ControlDimensionValue::Unknown {
            known_contributions,
            unresolved_sources,
        } => json!({
            "state":"unknown",
            "actions":known_contributions.iter().flat_map(|item|item.value().iter().map(local_recovery_action_json)).collect::<Vec<_>>(),
            "unresolved_sources":unresolved_sources.iter().map(|node|node.raw()).collect::<Vec<_>>(),
        }),
        ControlDimensionValue::PerChild {
            unresolved_sources, ..
        } => {
            json!({"state":"per_child","unresolved_sources":unresolved_sources.iter().map(|node|node.raw()).collect::<Vec<_>>()})
        }
        ControlDimensionValue::NotApplicable => json!({"state":"not_applicable"}),
    };
    json!({
        "potential_actions":actions,
        "local_recovery_actions":local_actions,
        "compensation_order":verdict(scope.compensation_order()),
        "recovery_blocked_outcome":verdict(scope.recovery_blocked_outcome()),
        "forward_effects_retained":scope.forward_effects_retained(),
        "forward_outcome_retained":scope.forward_outcome_retained(),
    })
}

fn local_recovery_action_json(action: &af_computation::LocalRecoveryAction) -> Value {
    json!({
        "source":{
            "caller_semantic_ir_digest":digest_ref_json(&action.source.caller_semantic_ir_digest),
            "source_node":action.source.source_node.raw(),
            "source_contract_digest":action.source.source_contract_digest.hex,
            "forward_entry_ordinal":action.source.forward_entry_ordinal,
            "action":recovery_action_identity_json(&action.source.action),
        },
        "forward_seed_upper":ceiling_json(action.forward_seed_upper),
        "owner_scope":action.owner_scope.raw(),
        "action_kind":action.action_kind.as_str(),
        "local_action_bound":local_action_bound_json(&action.local_bound),
        "forward_occurrence":entry_json(&action.forward_occurrence.declared),
        "action_effect_occurrence":entry_json(&action.action_effect_occurrence.declared),
        "recovery_usage_binding":{
            "action":recovery_action_identity_json(&action.recovery_usage_binding.action),
            "activations_per_forward":bounds_json(&action.recovery_usage_binding.activations_per_forward),
            "attempts_per_activation":bounds_json(&action.recovery_usage_binding.attempts_per_activation),
            "per_attempt_usage":usage_signature_json(&action.recovery_usage_binding.per_attempt_usage),
        },
        "recovery_usage":usage_json(&action.recovery_usage),
        "guard_compatibility":action.guard_compatibility.as_ref().map(guard_compatibility_json),
        "eligibility_evidence":verdict(action.eligibility_evidence),
    })
}

fn scope_boundaries_json(summary: &DerivedControlContractSummary<'_, '_>) -> Vec<Value> {
    let mut result = Vec::new();
    if let Some(scope) = summary.scope_boundary() {
        result.push(scope_boundary_json(scope));
    }
    for child in summary.children() {
        result.extend(scope_boundaries_json(child));
    }
    result
}

fn compensation_scopes_json(
    summary: &DerivedControlContractSummary<'_, '_>,
    pins: &BTreeMap<String, String>,
    resource_alias_ids: &BTreeMap<String, ResourceSlotId>,
) -> Vec<Value> {
    let mut result = Vec::new();
    if let Some(scope) = summary.compensation_scope() {
        result.push(json!({
            "control_node":summary.control_node().raw(),
            "summary":compensation_scope_json(scope,pins,resource_alias_ids),
        }));
    }
    for child in summary.children() {
        result.extend(compensation_scopes_json(child, pins, resource_alias_ids));
    }
    result
}

fn conditional_compensation_action_json(
    action: &af_computation::ConditionalCompensationAction,
    pins: &BTreeMap<String, String>,
    resource_alias_ids: &BTreeMap<String, ResourceSlotId>,
) -> Value {
    let forward = action.forward_occurrence();
    let action_effect = action.action_effect_occurrence();
    let identity = action.identity();
    json!({
        "source_node":action.source_node().raw(),
        "source_contract":pins.get(&forward.source_contract_digest.hex).cloned().unwrap_or_else(||forward.source_contract_digest.hex.clone()),
        "forward_occurrence":{"node":forward.computation.raw(),"entry_ordinal":forward.source_entry_ordinal,"atom":atom_json(&forward.declared.atom),"families":forward.control_families.iter().map(|family|json!({"control_node":family.control_node.raw(),"kind":match family.kind {af_computation::EffectOccurrenceFamilyKind::Iterate=>"iterate",af_computation::EffectOccurrenceFamilyKind::Retry=>"retry"},"logical_repetitions":{"lower":family.logical_repetitions.lower,"upper":occurrence_ceiling_json(family.logical_repetitions.upper)},"physical_attempts":family.physical_attempts.as_ref().map(|bounds|json!({"lower":bounds.lower,"upper":occurrence_ceiling_json(bounds.upper)}))})).collect::<Vec<_>>()},
        "action_effect_occurrence":{"node":action_effect.computation.raw(),"entry_ordinal":action_effect.source_entry_ordinal,"atom":atom_json(&action_effect.declared.atom),"declared_upper":occurrence_ceiling_json(action_effect.declared.bounds.upper),"mapped_resource_slot":action_effect.mapped_resource_slot.map(|slot|resource_alias_name(slot,resource_alias_ids))},
        "identity":{"forward_effect":atom_json(&identity.forward_effect),"action_kind":identity.action_kind.as_str(),"action_effect":atom_json(&identity.action_effect)},
        "potential_effect":entry_json(action.potential_effect()),
        "potential_recovery_usage":usage_json(action.potential_recovery_usage()),
        "receipt_evidence":verdict(action.receipt_evidence()),
    })
}

fn interface_json(summary: &DerivedControlContractSummary<'_, '_>) -> Value {
    let outputs = match summary.interface().outputs() {
        ControlDimensionValue::Complete(outputs) => json!({
            "state":"complete",
            "ports":outputs.iter().map(|port|json!({"parent_port":port.parent_port_no(),"parent_slot":port.parent_value_slot().raw(),"local_type_id":port.local_type_id().raw(),"sources":port.sources().iter().map(|source|json!({"child":source.child_node().raw(),"port":source.child_port_no(),"value_slot":source.child_value_slot().raw()})).collect::<Vec<_>>() })).collect::<Vec<_>>(),
        }),
        ControlDimensionValue::Partial {
            known_contribution,
            unresolved_sources,
        } => {
            json!({"state":"partial","ports":known_contribution.as_ref().map(|ports|ports.iter().map(|port|json!({"parent_port":port.parent_port_no(),"parent_slot":port.parent_value_slot().raw(),"local_type_id":port.local_type_id().raw(),"sources":port.sources().iter().map(|source|json!({"child":source.child_node().raw(),"port":source.child_port_no(),"value_slot":source.child_value_slot().raw()})).collect::<Vec<_>>() })).collect::<Vec<_>>()).unwrap_or_default(),"unresolved_sources":unresolved_sources.iter().map(|node|node.raw()).collect::<Vec<_>>() })
        }
        ControlDimensionValue::Unknown {
            unresolved_sources, ..
        } => {
            json!({"state":"unknown","unresolved_sources":unresolved_sources.iter().map(|node|node.raw()).collect::<Vec<_>>() })
        }
        ControlDimensionValue::PerChild {
            unresolved_sources, ..
        } => {
            json!({"state":"per_child","unresolved_sources":unresolved_sources.iter().map(|node|node.raw()).collect::<Vec<_>>() })
        }
        ControlDimensionValue::NotApplicable => json!({"state":"not_applicable"}),
    };
    let selector = summary.interface().selector().map(|selector|json!({
        "expression":selector.expression().raw(),
        "dependency_slots":selector.dependency_slots().iter().map(|slot|slot.raw()).collect::<Vec<_>>(),
        "case_ordinals":selector.case_ordinals(),
        "exact_case":selector.exact_case(),
        "totality_verdict":verdict(selector.totality_verdict()),
    }));
    json!({"outputs":outputs,"selector":selector})
}

fn digest_ref_json(digest: &DigestRefValue) -> Value {
    json!({"algorithm":digest.algorithm,"domain":digest.domain,"hex":digest.hex})
}

fn schema_ref_json(schema: &af_semantic::TypeSchemaRef) -> Value {
    json!({"specification_id":schema.specification_id,"type_name":schema.type_name,"schema_digest":digest_ref_json(&schema.schema_digest)})
}

fn failure_types_json(summary: &DerivedControlContractSummary<'_, '_>) -> Value {
    match summary.failures().value() {
        ControlDimensionValue::Complete(row) => json!(
            row.failures
                .iter()
                .map(|failure| schema_ref_json(&failure.payload_type))
                .collect::<Vec<_>>()
        ),
        ControlDimensionValue::Partial { .. } => json!({"state":"partial"}),
        ControlDimensionValue::Unknown { .. } => json!({"state":"unknown"}),
        ControlDimensionValue::PerChild { .. } => json!({"state":"per_child"}),
        ControlDimensionValue::NotApplicable => json!({"state":"not_applicable"}),
    }
}

fn dimension_availability(summary: &DerivedControlContractSummary<'_, '_>) -> Value {
    json!({
        "failure": dimension_state(summary.failures().value()),
        "effect": dimension_state(summary.effects().value()),
        "resource": dimension_state(summary.resources().value()),
        "information_flow": dimension_state(summary.information_flow().value()),
        "usage": dimension_state(summary.usage().value()),
        "recovery": dimension_state(summary.recovery().value()),
        "execution": dimension_state(summary.execution().value()),
        "typed_outputs": dimension_state(summary.interface().outputs()),
    })
}

fn dimension_state<T>(value: &ControlDimensionValue<T>) -> &'static str {
    match value {
        ControlDimensionValue::Complete(_) => "complete",
        ControlDimensionValue::Partial { .. } => "partial",
        ControlDimensionValue::Unknown { .. } => "unknown",
        ControlDimensionValue::PerChild { .. } => "per_child",
        ControlDimensionValue::NotApplicable => "not_applicable",
    }
}

fn resource_alias_name(slot: ResourceSlotId, aliases: &BTreeMap<String, ResourceSlotId>) -> String {
    aliases
        .iter()
        .find(|(_, value)| **value == slot)
        .map_or_else(
            || format!("resource-slot:{}", slot.raw()),
            |(alias, _)| alias.clone(),
        )
}

fn resource_requirements_json(
    summary: &DerivedControlContractSummary<'_, '_>,
    aliases: &BTreeMap<String, ResourceSlotId>,
) -> Vec<Value> {
    let ControlDimensionValue::PerChild { contributions, .. } = summary.resources().value() else {
        return vec![];
    };
    contributions
        .iter()
        .flat_map(|contribution| {
            contribution.value().iter().map(|item| {
                let mapping = item.mapping();
                let requirement = item.requirement();
                json!({
                    "source_node": contribution.source_node().raw(),
                    "component_port": mapping.component_port_no,
                    "resource_slot": resource_alias_name(mapping.resource_slot, aliases),
                    "required_type": requirement.map(|row|schema_ref_json(&row.resource_type.schema)),
                    "required_attributes": requirement.map(|row|row.required_attributes.iter().map(|attribute|json!({"key":attribute.key,"value":attribute.value})).collect::<Vec<_>>()).unwrap_or_default(),
                    "attributes_proven": mapping.required_attributes_proven,
                })
            })
        })
        .collect()
}

fn resource_aliases_json(
    summary: &DerivedControlContractSummary<'_, '_>,
    aliases: &BTreeMap<String, ResourceSlotId>,
) -> Vec<Value> {
    summary
        .resource_aliases()
        .iter()
        .map(|alias| {
            json!({
                "resource_slot": resource_alias_name(alias.resource_slot(), aliases),
                "occurrences": alias.occurrences().iter().map(|occurrence|json!({"source_node":occurrence.source_node().raw(),"component_port":occurrence.mapping().component_port_no,"required_type":occurrence.requirement().map(|row|schema_ref_json(&row.resource_type.schema))})).collect::<Vec<_>>(),
            })
        })
        .collect()
}

fn flow_sources_json(
    summary: &DerivedControlContractSummary<'_, '_>,
    pins: &BTreeMap<String, String>,
    aliases: &BTreeMap<String, ResourceSlotId>,
) -> Vec<Value> {
    let (ControlDimensionValue::Complete(flow)
    | ControlDimensionValue::Partial {
        known_contribution: Some(flow),
        ..
    }) = summary.information_flow().value()
    else {
        return vec![];
    };
    flow.sources()
        .iter()
        .map(|source| {
            json!({
                "source_node": source.source_node().raw(),
                "target": target_json(source.target(), pins),
                "contract": information_flow_contract_json(source.contract()),
                "source_mappings": source.source_mappings().iter().map(flow_mapping_json).collect::<Vec<_>>(),
                "effect_flow_partitions": source.effect_flow_partitions().iter().map(|partition|json!({"operation":{"catalog":partition.operation.catalog,"name":partition.operation.name},"kind":partition.kind.as_str(),"allowed_effects":partition.allowed_effects.iter().map(atom_json).collect::<Vec<_>>()})).collect::<Vec<_>>(),
            })
        })
        .map(|mut value| {
            // Resource identities in Flow source bindings are scoped aliases,
            // not raw numeric ids.
            if let Some(mappings) = value["source_mappings"].as_array_mut() {
                for mapping in mappings {
                    if mapping["source_binding"]["kind"] == "resource_slot" {
                        let raw = u32::try_from(mapping["source_binding"]["resource_slot"].as_u64().unwrap_or_default()).unwrap_or_default();
                        mapping["source_binding"]["resource_slot"] = json!(resource_alias_name(ResourceSlotId::from_raw(raw), aliases));
                    }
                }
            }
            value
        })
        .collect()
}

fn information_flow_contract_json(contract: &InformationFlowContract) -> Value {
    json!({
        "confidentiality_restrictions": contract.confidentiality_restrictions.iter().map(|label|json!({"catalog":label.catalog,"name":label.name})).collect::<Vec<_>>(),
        "summary": {
            "sources": contract.summary.sources.iter().map(|source|match source.category { SourceCategory::WorkflowInput(port)=>json!({"workflow_input":port}),SourceCategory::ResourceRequirement(port)=>json!({"resource_requirement":port}),SourceCategory::UnknownExternal=>json!("unknown_external") }).collect::<Vec<_>>(),
            "taints": contract.summary.taints.iter().map(|taint|json!({"catalog":taint.catalog,"name":taint.name})).collect::<Vec<_>>(),
            "instruction_authority": contract.summary.instruction_authority.as_str(),
            "influence_unknown": contract.summary.influence_unknown,
        },
        "flow_constraints": contract.flow_constraints.iter().map(flow_constraint_json).collect::<Vec<_>>(),
    })
}

fn flow_mapping_json(mapping: &af_computation::FlowPortMapping) -> Value {
    json!({
        "source": mapping.source.as_ref().map(|source|match source.category { SourceCategory::WorkflowInput(port)=>json!({"workflow_input":port}),SourceCategory::ResourceRequirement(port)=>json!({"resource_requirement":port}),SourceCategory::UnknownExternal=>json!("unknown_external") }),
        "sink": mapping.sink.as_ref().map(flow_constraint_json),
        "source_binding": mapping.source_binding.as_ref().map(|binding|match binding { af_computation::FlowSourceBinding::InputExpression(expression)=>json!({"kind":"input_expression","expression":expression.raw()}),af_computation::FlowSourceBinding::ResourceSlot(slot)=>json!({"kind":"resource_slot","resource_slot":slot.raw()}),af_computation::FlowSourceBinding::UnknownExternal=>json!({"kind":"unknown_external"}) }),
        "sink_binding": mapping.sink_binding.as_ref().map(|binding|match binding { af_computation::FlowSinkBinding::OutputValue(slot)=>json!({"kind":"output_value","value_slot":slot.raw()}),af_computation::FlowSinkBinding::EffectOccurrences(indices)=>json!({"kind":"effect_occurrences","indices":indices}) }),
        "matched_effect_occurrences": mapping.matched_effect_occurrences,
    })
}

fn flow_constraint_json(constraint: &af_flow::FlowConstraint) -> Value {
    let sink = match &constraint.sink {
        af_flow::FlowSink::WorkflowOutput(port) => json!({"kind":"workflow_output","port":port}),
        af_flow::FlowSink::ExternalEffect(operation) => {
            json!({"kind":"external_effect","operation":{"catalog":operation.catalog,"name":operation.name}})
        }
    };
    json!({"kind":constraint.kind.as_str(),"sink":sink})
}

fn recovery_sources_full_json(
    summary: &DerivedControlContractSummary<'_, '_>,
    pins: &BTreeMap<String, String>,
) -> Vec<Value> {
    let ControlDimensionValue::PerChild { contributions, .. } = summary.recovery().value() else {
        return vec![];
    };
    contributions
        .iter()
        .flat_map(|contribution| {
            contribution.value().iter().map(|source| {
                json!({
                    "source_node":contribution.source_node().raw(),
                    "target":target_json(source.target(),pins),
                    "replay_semantics":source.contract().replay_semantics.as_str(),
                    "clauses":source.contract().effect_recovery_clauses.iter().map(|clause|json!({"effect":atom_json(&clause.effect),"idempotency":clause.idempotency.as_str(),"key_requirement":clause.key_requirement.as_ref().map(|key|json!({"stable_across_attempts":key.stable_across_attempts,"distinct_across_invocations":key.distinct_across_invocations,"bound_to_operation":key.bound_to_operation,"provider_honors":key.provider_honors})),"reconciliation":match clause.reconciliation { af_recovery::ReconciliationRequirement::None=>"none", af_recovery::ReconciliationRequirement::Required=>"required" },"compensation":match &clause.compensation { af_recovery::CompensationRequirement::None=>json!("none"),af_recovery::CompensationRequirement::Required(operation)=>json!({"required":{"catalog":operation.catalog,"name":operation.name}}) },"unknown_effect_handling":match clause.unknown_effect_handling { af_recovery::UnknownEffectHandling::Block=>"block",af_recovery::UnknownEffectHandling::ReconciliationRequired=>"reconciliation_required" }})).collect::<Vec<_>>(),
                    "action_bindings":source.action_bindings().iter().map(|binding|json!({"forward_effect":atom_json(&binding.identity.forward_effect),"action_kind":binding.identity.action_kind.as_str(),"action_effect":atom_json(&binding.identity.action_effect)})).collect::<Vec<_>>(),
                    "usage_bindings":source.usage_bindings().iter().map(|binding|json!({"action":{"forward_effect":atom_json(&binding.action.forward_effect),"action_kind":binding.action.action_kind.as_str(),"action_effect":atom_json(&binding.action.action_effect)},"activations_per_forward":bounds_json(&binding.activations_per_forward),"attempts_per_activation":bounds_json(&binding.attempts_per_activation),"per_attempt_usage":usage_signature_json(&binding.per_attempt_usage)})).collect::<Vec<_>>(),
                    "effect_occurrences":source.effects().iter().map(|occurrence|json!({"node":occurrence.computation.raw(),"entry_ordinal":occurrence.source_entry_ordinal,"atom":atom_json(&occurrence.declared.atom)})).collect::<Vec<_>>(),
                })
            })
        })
        .collect()
}

fn bounds_json(bounds: &OccurrenceBounds) -> Value {
    json!({"lower":bounds.lower,"upper":ceiling_json(bounds.upper)})
}

fn usage_signature_json(signature: &UsageSignature) -> Value {
    json!({"limits":signature.limits.iter().map(|limit|json!({"meter":{"catalog":limit.meter.catalog,"name":limit.meter.name},"phase":limit.phase.as_str(),"upper":match limit.upper { UsageUpperBound::Finite(value)=>json!(value),UsageUpperBound::Unbounded=>json!("unbounded") }})).collect::<Vec<_>>()})
}

fn execution_full_json(summary: &DerivedControlContractSummary<'_, '_>) -> Vec<Value> {
    let ControlDimensionValue::PerChild { contributions, .. } = summary.execution().value() else {
        return vec![];
    };
    contributions
        .iter()
        .flat_map(|contribution| contribution.value().iter().map(move |source|json!({"source_node":contribution.source_node().raw(),"requirements":execution_requirements_json(source.requirements())})))
        .collect()
}

fn execution_requirements_json(requirements: &ExecutionRequirements) -> Value {
    json!({
        "capabilities":requirements.capabilities.iter().map(|item|json!({"catalog":item.capability.catalog,"name":item.capability.name,"version":item.exact_version,"attributes":item.required_attributes.iter().map(|a|json!({"key":a.key,"value":a.value})).collect::<Vec<_>>()})).collect::<Vec<_>>(),
        "quantities":requirements.quantities.iter().map(|item|json!({"kind_no":item.kind.value_no(),"minimum":item.minimum})).collect::<Vec<_>>(),
        "sandbox":requirements.sandbox.as_ref().map(|item|if item.minimum==SandboxLevel::Basic { "basic" } else { "strong" }),
        "trust_domain":requirements.trust_domain.as_ref().map(|item|json!({"catalog":item.catalog,"name":item.name})),
        "region_residency":requirements.region_residency.as_ref().map(|item|json!({"execution_regions":item.execution_regions.iter().map(|r|json!({"catalog":r.catalog,"name":r.name})).collect::<Vec<_>>(),"residency_regions":item.residency_regions.iter().map(|r|json!({"catalog":r.catalog,"name":r.name})).collect::<Vec<_>>()})),
        "runtime_profile":requirements.runtime_profile.as_ref().map(|item|json!({"family":item.family,"profile":{"catalog":item.profile.catalog,"name":item.profile.name},"required_features":item.required_features})),
        "implementation_abis":requirements.implementation_abis.iter().map(|item|schema_ref_json(&item.schema)).collect::<Vec<_>>(),
        "soft_preferences":requirements.soft_preferences.iter().map(|item|item.value_no()).collect::<Vec<_>>(),
    })
}

fn target_json(target: &af_computation::ResolvedTarget, pins: &BTreeMap<String, String>) -> Value {
    match target {
        af_computation::ResolvedTarget::Component {
            identity,
            contract_digest,
        } => {
            json!({"kind":"component","identity":{"catalog":identity.catalog,"name":identity.name},"contract_digest":pins.get(&contract_digest.hex).cloned().unwrap_or_else(||contract_digest.hex.clone())})
        }
        af_computation::ResolvedTarget::Subflow {
            semantic_ir_digest,
            exported_contract_digest,
        } => {
            json!({"kind":"subflow","semantic_ir_digest":digest_ref_json(semantic_ir_digest),"exported_contract_digest":exported_contract_digest.as_ref().map(|digest|pins.get(&digest.hex).cloned().unwrap_or_else(||digest.hex.clone()))})
        }
        af_computation::ResolvedTarget::AwaitSignal { signal_type } => {
            json!({"kind":"await_signal","signal_type":schema_ref_json(signal_type)})
        }
    }
}

fn effect_json(signature: &EffectSignature) -> Value {
    match &signature.row {
        EffectRow::Unknown => json!({"row":"unknown"}),
        EffectRow::Closed(entries) => {
            json!({"row":"closed","entries":entries.iter().map(entry_json).collect::<Vec<_>>(),"residuals":[]})
        }
        EffectRow::Open(open) => {
            json!({"row":"open","entries":open.known.iter().map(entry_json).collect::<Vec<_>>(),"residuals":open.residual.iter().map(residual_json).collect::<Vec<_>>()})
        }
    }
}

fn entry_json(entry: &EffectEntry) -> Value {
    json!({"atom":atom_json(&entry.atom),"lower":entry.bounds.lower,"upper":ceiling_json(entry.bounds.upper),"guard":guard_json(&entry.guard)})
}

fn residual_json(residual: &EffectResidual) -> Value {
    json!({"catalog":residual.catalog,"selector":selector_json(&residual.selector),"lower":residual.bounds.lower,"upper":ceiling_json(residual.bounds.upper)})
}

fn atom_json(atom: &EffectAtom) -> Value {
    let qualifiers = atom
        .qualifiers
        .iter()
        .map(|q| match q {
            EffectQualifier::Idempotency(EffectIdempotencyQualifier::Natural) => {
                json!({"family":"idempotency","member":"natural"})
            }
            EffectQualifier::Idempotency(EffectIdempotencyQualifier::None) => {
                json!({"family":"idempotency","member":"none"})
            }
        })
        .collect::<Vec<_>>();
    json!({"operation":{"catalog":atom.operation.catalog,"name":atom.operation.name},"resource_selector":selector_json(&atom.resource_selector),"qualifiers":qualifiers})
}

fn selector_json(selector: &ResourceSelector) -> Value {
    json!({"requirement_port_no":selector.requirement_port_no,"scope_bindings":selector.scope_bindings.iter().map(|item|json!({"key":item.key,"value":item.value})).collect::<Vec<_>>()})
}

fn guard_json(guard: &EffectGuard) -> Value {
    match guard {
        EffectGuard::Unconditional => json!("unconditional"),
        EffectGuard::InputEquals(item) => {
            json!({"input_equals":{"port":item.port_no,"value":primitive_json(&item.expected)}})
        }
    }
}

fn primitive_json(value: &PrimitiveConstant) -> Value {
    match value {
        PrimitiveConstant::String(s) => json!(s),
        PrimitiveConstant::Int64(n) => json!(n),
        PrimitiveConstant::Bool(b) => json!(b),
        _ => json!(value.case_name()),
    }
}

fn ceiling_json(value: OccurrenceCeiling) -> Value {
    match value {
        OccurrenceCeiling::Finite(value) => json!(value),
        OccurrenceCeiling::Unbounded => json!("unbounded"),
    }
}

fn usage_json(envelope: &StaticUsageEnvelope) -> Value {
    let coordinates = usage_coordinates_json(envelope);
    let state = if envelope.is_globally_unknown() {
        "unknown"
    } else if coordinates.is_empty() {
        "empty_without_zero_claim"
    } else if coordinates.values().any(|v| v == "unknown") {
        "partial_unknown"
    } else {
        "known"
    };
    json!({"state":state,"coordinates":coordinates})
}

fn usage_coordinates_json(envelope: &StaticUsageEnvelope) -> serde_json::Map<String, Value> {
    let mut coordinates = serde_json::Map::new();
    for (coordinate, bound) in envelope.iter() {
        let key = format!("{}/{}", coordinate.phase.as_str(), coordinate.meter.name);
        let value = match bound {
            StaticUsageBound::KnownFinite(value) => json!(value),
            StaticUsageBound::KnownUnbounded => json!("unbounded"),
            StaticUsageBound::Unknown => json!("unknown"),
        };
        coordinates.insert(key, value);
    }
    coordinates
}

fn usage_known_contributions(summary: &DerivedControlContractSummary<'_, '_>) -> Vec<Value> {
    summary
        .children()
        .iter()
        .filter_map(|child| match child.usage().value() {
            ControlDimensionValue::Complete(envelope) => {
                Some(json!({"coordinates":usage_coordinates_json(envelope)}))
            }
            ControlDimensionValue::Partial {
                known_contribution: Some(envelope),
                ..
            } => Some(json!({"coordinates":usage_coordinates_json(envelope)})),
            _ => None,
        })
        .collect()
}

fn flow_json(flow: &af_computation::ControlFlowSummary, verdict_value: LeafVerdict) -> Value {
    let (sources, taints, authority, unknown) = flow
        .known_may_influence()
        .map(|summary| {
            let sources = summary
                .sources
                .iter()
                .map(|source| match source.category {
                    SourceCategory::WorkflowInput(port) => format!("input:{port}"),
                    SourceCategory::ResourceRequirement(port) => format!("resource:{port}"),
                    SourceCategory::UnknownExternal => "unknown_external".into(),
                })
                .collect::<Vec<_>>();
            let taints = summary
                .taints
                .iter()
                .map(|taint| taint.name.clone())
                .collect::<Vec<_>>();
            (
                sources,
                taints,
                summary.instruction_authority.as_str(),
                summary.influence_unknown,
            )
        })
        .unwrap_or_default();
    let mut constraints = flow.sources().iter().map(|source| json!({"node":source.source_node().raw(),"constraints":source.contract().flow_constraints.iter().map(flow_constraint_json).collect::<Vec<_>>()})).collect::<Vec<_>>();
    constraints.sort_by_key(|item| item["node"].as_u64().unwrap_or_default());
    json!({"sources":sources,"taints":taints,"may_carry_authority":authority,"effective_authority_inferred":false,"influence_unknown":unknown || flow.control_decision_influence_unknown() || !flow.select_decision_dependencies().is_empty() || !flow.unresolved_children().is_empty(),"decision_dependencies":flow.select_decision_dependencies().iter().map(|slot|format!("value-slot:{}",slot.raw())).collect::<Vec<_>>(),"constraints_by_child":constraints,"verdict":verdict(verdict_value)})
}

fn resources_json(
    summary: &DerivedControlContractSummary<'_, '_>,
    aliases: &BTreeMap<String, ResourceSlotId>,
) -> Vec<Value> {
    let ControlDimensionValue::PerChild { contributions, .. } = summary.resources().value() else {
        return vec![];
    };
    let reverse = aliases
        .iter()
        .map(|(alias, slot)| (slot.raw(), alias.clone()))
        .collect::<BTreeMap<_, _>>();
    contributions.iter().flat_map(|contribution| contribution.value().iter().map(|occurrence| {
        let slot=reverse.get(&occurrence.mapping().resource_slot.raw()).cloned().unwrap_or_else(||format!("resource-slot:{}",occurrence.mapping().resource_slot.raw()));
        json!({"node":contribution.source_node().raw(),"requirements":[{"component_port":occurrence.mapping().component_port_no,"resource_slot":slot}]})
    })).collect()
}

fn execution_json(summary: &DerivedControlContractSummary<'_, '_>) -> Vec<Value> {
    let ControlDimensionValue::PerChild { contributions, .. } = summary.execution().value() else {
        return vec![];
    };
    contributions
        .iter()
        .flat_map(|contribution| {
            contribution.value().iter().filter_map(move |source| {
                let requirements = source.requirements();
                let mut x = serde_json::Map::new();
                if requirements
                    .sandbox
                    .as_ref()
                    .is_some_and(|sandbox| sandbox.minimum == SandboxLevel::Strong)
                {
                    x.insert("sandbox".into(), json!("strict"));
                }
                if requirements
                    .runtime_profile
                    .as_ref()
                    .is_some_and(|profile| profile.profile.name == "python")
                {
                    x.insert("runtime".into(), json!("python"));
                }
                (!x.is_empty()).then(|| json!({"node":contribution.source_node().raw(),"x":x}))
            })
        })
        .collect()
}

fn recovery_json(
    summary: &DerivedControlContractSummary<'_, '_>,
    pins: &BTreeMap<String, String>,
) -> Vec<Value> {
    let ControlDimensionValue::PerChild { contributions, .. } = summary.recovery().value() else {
        return vec![];
    };
    contributions.iter().flat_map(|contribution| contribution.value().iter().filter_map(|source| {
        if source.contract().effect_recovery_clauses.is_empty() { return None; }
        let contract_digest = match source.target() {
            af_computation::ResolvedTarget::Component { contract_digest, .. } => pins.get(&contract_digest.hex).cloned().unwrap_or_else(|| contract_digest.hex.clone()),
            af_computation::ResolvedTarget::Subflow { exported_contract_digest: Some(contract_digest), .. } => pins.get(&contract_digest.hex).cloned().unwrap_or_else(|| contract_digest.hex.clone()),
            _ => "unavailable".into(),
        };
        Some(json!({"node":contribution.source_node().raw(),"contract_digest":contract_digest,"actions":source.action_bindings().iter().map(|binding|{
            let usage = source.usage_bindings().iter().find(|usage| usage.action == binding.identity);
            let clause = source.contract().effect_recovery_clauses.first();
            json!({
                "forward_atom":atom_json(&binding.identity.forward_effect),
                "action":if binding.identity.action_kind == RecoveryActionKind::Compensation {"compensate"} else {"reconcile"},
                "action_atom":atom_json(&binding.identity.action_effect),
                "clause":clause.map(recovery_clause_json),
                "activations_per_forward":usage.map(|item|bounds_json(&item.activations_per_forward)),
                "attempts_per_activation":usage.map(|item|bounds_json(&item.attempts_per_activation)),
                "per_attempt_usage":usage.map(|item|usage_signature_json(&item.per_attempt_usage)),
            })
        }).collect::<Vec<_>>() }))
    })).collect()
}

fn recovery_clause_json(clause: &EffectRecoveryClause) -> Value {
    json!({
        "idempotency":clause.idempotency.as_str(),
        "key_requirement":clause.key_requirement.as_ref().map(|key|json!({
            "stable_across_attempts":key.stable_across_attempts,
            "distinct_across_invocations":key.distinct_across_invocations,
            "bound_to_operation":key.bound_to_operation,
            "provider_honors":key.provider_honors,
        })),
        "reconciliation":match clause.reconciliation {
            ReconciliationRequirement::None => "none",
            ReconciliationRequirement::Required => "required",
        },
        "compensation":match &clause.compensation {
            CompensationRequirement::None => json!("none"),
            CompensationRequirement::Required(operation) => json!({"required":{"catalog":operation.catalog,"name":operation.name}}),
        },
        "unknown_effect_handling":match clause.unknown_effect_handling {
            UnknownEffectHandling::Block => "block",
            UnknownEffectHandling::ReconciliationRequired => "reconciliation_required",
        },
    })
}

fn verdict(value: LeafVerdict) -> &'static str {
    match value {
        LeafVerdict::Proven => "proven",
        LeafVerdict::Violation => "violation",
        LeafVerdict::Unproven => "unproven",
        LeafVerdict::RequiresEvidence => "requires_evidence",
    }
}

fn failure_alias(reference: &FailureTypeRef) -> String {
    if reference.type_name() == "TypeSchemaRef" {
        "FailureA"
    } else {
        "FailureB"
    }
    .into()
}

fn modules() -> CompiledModuleSet {
    let sources = vec![
        (
            "spec/idl/core/machine-primitives.afidl",
            include_str!("../../../../spec/idl/core/machine-primitives.afidl"),
        ),
        (
            "spec/idl/specification-registry.afidl",
            include_str!("../../../../spec/idl/specification-registry.afidl"),
        ),
        (
            "spec/idl/semantic-core.afidl",
            include_str!("../../../../spec/idl/semantic-core.afidl"),
        ),
        (
            "spec/idl/semantic-ir.afidl",
            include_str!("../../../../spec/idl/semantic-ir.afidl"),
        ),
        (
            "spec/idl/effect-signature.afidl",
            include_str!("../../../../spec/idl/effect-signature.afidl"),
        ),
        (
            "spec/idl/failure-row.afidl",
            include_str!("../../../../spec/idl/failure-row.afidl"),
        ),
        (
            "spec/idl/resource-requirement.afidl",
            include_str!("../../../../spec/idl/resource-requirement.afidl"),
        ),
        (
            "spec/idl/information-flow.afidl",
            include_str!("../../../../spec/idl/information-flow.afidl"),
        ),
        (
            "spec/idl/usage-signature.afidl",
            include_str!("../../../../spec/idl/usage-signature.afidl"),
        ),
        (
            "spec/idl/cost-projection.afidl",
            include_str!("../../../../spec/idl/cost-projection.afidl"),
        ),
        (
            "spec/idl/recovery-contract.afidl",
            include_str!("../../../../spec/idl/recovery-contract.afidl"),
        ),
        (
            "spec/idl/execution-requirements.afidl",
            include_str!("../../../../spec/idl/execution-requirements.afidl"),
        ),
        (
            "spec/idl/component-contract.afidl",
            include_str!("../../../../spec/idl/component-contract.afidl"),
        ),
        (
            "spec/idl/effect-resource-compatibility.afidl",
            include_str!("../../../../spec/idl/effect-resource-compatibility.afidl"),
        ),
        (
            "spec/idl/recovery-usage-coverage.afidl",
            include_str!("../../../../spec/idl/recovery-usage-coverage.afidl"),
        ),
        (
            "spec/idl/flow-execution-relation.afidl",
            include_str!("../../../../spec/idl/flow-execution-relation.afidl"),
        ),
    ]
    .into_iter()
    .map(|(path, source)| (path.to_owned(), source.to_owned()))
    .collect::<Vec<_>>();
    compile_module_set(&sources).unwrap()
}
