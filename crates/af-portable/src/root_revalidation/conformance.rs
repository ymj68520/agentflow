//! Test-build-only, getter-based full D9 projection. No checked constructors.

use super::*;

pub(crate) fn project_full_d9_for_conformance(
    workflow: &DerivedWorkflowContractSummary<'_, '_>,
    mut root: Json,
    mut recovery: Json,
) -> Result<(Json, Json), PortableProofError> {
    let root_context = workflow.root().proof_context_reference();
    root["root_control"] = full_node(workflow.root(), root_context, 0)?;
    root["inherited_obligations"] = root["root_control"]["inherited_obligations"].clone();
    for (name, proofs) in dimension_proofs(workflow.root()) {
        root["root_dimensions"][name]["proofs"] = proofs;
    }
    let identity = workflow.resolver_identity();
    root["resolver_identity"] = json!({
        "component_contract_digests":identity.component_contract_digests().iter().map(digest_json).collect::<Vec<_>>(),
        "subflow_targets":identity.subflow_targets().iter().map(|target|json!({
            "semantic_ir_digest":digest_json(target.semantic_ir_digest()),
            "direct_dependencies":target.direct_dependencies().map(|items|items.iter().map(digest_json).collect::<Vec<_>>()),
            "exported_contract_digest":target.exported_contract_digest().map(digest_json),
        })).collect::<Vec<_>>(),
    });
    root["proof_context"] = context_reference(root_context, root_context);
    let k = workflow.root_recovery();
    for (name, proofs) in [
        ("actions", k.actions().proofs()),
        ("action_effects", k.action_effects().proofs()),
        (
            "epsilon_coverage_requirement",
            k.epsilon_coverage_requirement().proofs(),
        ),
        ("recovery_usage", k.recovery_usage().proofs()),
    ] {
        recovery[name]["proofs"] = control_proof_json(proofs);
    }
    Ok((root, recovery))
}

fn dimension_proofs(node: &DerivedControlContractSummary<'_, '_>) -> Vec<(&'static str, Json)> {
    vec![
        ("failure", control_proof_json(node.failures().proofs())),
        ("effect", control_proof_json(node.effects().proofs())),
        ("resource", control_proof_json(node.resources().proofs())),
        (
            "information_flow",
            control_proof_json(node.information_flow().proofs()),
        ),
        ("usage", control_proof_json(node.usage().proofs())),
        (
            "recovery_sources",
            control_proof_json(node.recovery().proofs()),
        ),
        (
            "execution_sources",
            control_proof_json(node.execution().proofs()),
        ),
    ]
}

fn context_reference(
    context: &af_computation::ProofContextReference<'_, '_>,
    root: &af_computation::ProofContextReference<'_, '_>,
) -> Json {
    json!({"same_as_root":context.is_same_context(root),"relation_catalog_pins":context.relation_catalog_pins().iter().map(|pin|json!({
        "schema":pin.schema,"revision":pin.revision,
        "canonical_content_checksum":{"algorithm":pin.canonical_content_checksum.algorithm,"hex":pin.canonical_content_checksum.hex},
    })).collect::<Vec<_>>()})
}

fn full_node(
    node: &DerivedControlContractSummary<'_, '_>,
    root: &af_computation::ProofContextReference<'_, '_>,
    depth: usize,
) -> Result<Json, PortableProofError> {
    if depth > 96 {
        return Err(resource_limit(
            "$.root.control_tree",
            "conformance projection depth exceeded",
        ));
    }
    let mut value = control_node_json(node, depth)?;
    value["children"] = Json::Array(
        node.children()
            .iter()
            .map(|child| full_node(child, root, depth + 1))
            .collect::<Result<Vec<_>, _>>()?,
    );
    let complete = project_root_summary_node_facts(node);
    for (key, item) in complete.as_object().expect("node facts record") {
        value[key] = item.clone();
    }
    value["leaf_summary"] = node
        .leaf_summary()
        .map_or(Json::Null, |leaf| leaf_json(leaf, root));
    value["proof_context"] = context_reference(node.proof_context_reference(), root);
    if let Some(scope) = node.compensation_scope() {
        value["compensation_scope"]["potential_actions"]["proofs"] =
            control_proof_json(scope.potential_actions().proofs());
        value["compensation_scope"]["local_recovery_actions"]["proofs"] =
            control_proof_json(scope.local_recovery_actions().proofs());
    }
    for (name, proofs) in dimension_proofs(node) {
        let key = match name {
            "failure" => "failures",
            "effect" => "effects",
            "resource" => "resources",
            other => other,
        };
        value[key]["proofs"] = proofs;
    }
    Ok(value)
}

fn project_root_summary_node_facts(node: &DerivedControlContractSummary<'_, '_>) -> Json {
    json!({
        "effect_occurrences":node.effect_occurrences().iter().map(scoped_effect_occurrence_json).collect::<Vec<_>>(),
        "effect_residuals":node.effect_residuals().iter().map(scoped_effect_residual_json).collect::<Vec<_>>(),
        "resource_aliases":node.resource_aliases().iter().map(|alias|json!({"resource_slot":alias.resource_slot().raw(),"occurrences":alias.occurrences().iter().map(resource_occurrence_json).collect::<Vec<_>>()})).collect::<Vec<_>>(),
        "effect_lower_bound_adjustments":node.effect_lower_bound_adjustments().iter().map(|item|json!({"source_node":item.source_node().raw(),"reason":match item.reason(){
            EffectLowerBoundAdjustmentReason::SequenceContinuationCanBeSkipped=>"sequence_continuation_can_be_skipped",
            EffectLowerBoundAdjustmentReason::FailFastSiblingCanBeCancelled=>"fail_fast_sibling_can_be_cancelled",
            EffectLowerBoundAdjustmentReason::RaceBranchMayLoseOrBeCancelled=>"race_branch_may_lose_or_be_cancelled",
            EffectLowerBoundAdjustmentReason::SelectMayHaveNoCoveredCase=>"select_may_have_no_covered_case",
        }})).collect::<Vec<_>>(),
        "sequence_dataflow_dependencies":node.sequence_dataflow_dependencies().iter().map(|item|json!({"producer_child":item.producer_child().raw(),"producer_node":item.producer_node().raw(),"value_slot":item.value_slot().raw(),"consumer_child":item.consumer_child().raw(),"consumer_node":item.consumer_node().raw(),"expression":item.expression().raw()})).collect::<Vec<_>>(),
        "inherited_obligations":node.inherited_obligations().iter().map(|proof|json!({"source_node":proof.source_node().raw(),"dimension":proof.dimension(),"verdict":proof.verdict().as_str(),"rule_id":proof.rule_id(),"reference":proof.reference(),"reason":proof.reason()})).collect::<Vec<_>>(),
    })
}

fn leaf_json(
    leaf: &af_computation::ResolvedComputationLeaf<'_, '_>,
    root: &af_computation::ProofContextReference<'_, '_>,
) -> Json {
    json!({
        "computation":leaf.computation().raw(),"caller_semantic_ir_digest":digest_json(leaf.caller_semantic_ir_digest()),
        "kind":match leaf.kind(){af_computation::LeafKind::Invoke=>"invoke",af_computation::LeafKind::Await=>"await",af_computation::LeafKind::CallSubflow=>"call_subflow"},
        "target":leaf.target().map(resolved_target_json),
        "input_mappings":leaf.input_mappings().iter().map(port_mapping_json).collect::<Vec<_>>(),
        "output_mappings":leaf.output_mappings().iter().map(port_mapping_json).collect::<Vec<_>>(),
        "resource_mappings":leaf.resource_mappings().iter().map(|item|json!({"component_port_no":item.component_port_no,"resource_slot":item.resource_slot.raw(),"required_type":schema_ref_json(&item.required_type),"required_attributes_proven":item.required_attributes_proven})).collect::<Vec<_>>(),
        "resource_aliases":leaf.resource_aliases().iter().map(|item|json!({"resource_slot":item.resource_slot.raw(),"component_ports":item.component_ports})).collect::<Vec<_>>(),
        "effect_occurrences":leaf.effect_occurrences().iter().map(scoped_effect_occurrence_json).collect::<Vec<_>>(),
        "effect_residuals":leaf.effect_residuals().iter().map(scoped_effect_residual_json).collect::<Vec<_>>(),
        "flow_mappings":leaf.flow_mappings().iter().map(flow_port_mapping_json).collect::<Vec<_>>(),
        "dimensions":leaf.dimensions().map(|d|json!({"failures":failure_row_json(&d.failures),"effects":effect_signature_json(&d.effects),"resources":d.resources.requirements.iter().map(resource_requirement_json).collect::<Vec<_>>(),"information_flow":information_flow_contract_json(&d.information_flow),"usage":usage_signature_json(&d.usage),"recovery":recovery_contract_json(&d.recovery),"execution":execution_requirements_json(&d.execution),"effect_flow_partitions":d.effect_flow_partitions.iter().map(effect_flow_partition_json).collect::<Vec<_>>(),"recovery_action_bindings":d.recovery_action_bindings.iter().map(|b|recovery_action_identity_json(&b.identity)).collect::<Vec<_>>(),"recovery_usage_bindings":d.recovery_usage_bindings.iter().map(recovery_usage_binding_json).collect::<Vec<_>>()})),
        "action_guard_compatibility":leaf.action_guard_compatibility().iter().map(action_guard_compatibility_json).collect::<Vec<_>>(),
        "await_summary":leaf.await_summary().map(|a|json!({"signal_type":schema_ref_json(&a.signal_type),"correlation":a.correlation.raw(),"output_bindings":a.output_bindings.iter().map(|(port,slot)|json!([port,slot.raw()])).collect::<Vec<_>>(),"timeout":"no_implicit_timeout","interaction":"contract_unavailable","provenance":"unknown_external_influence","authority":"not_inferred"})),
        "proof":leaf.proof().dimensions().iter().map(|p|json!({"dimension":p.dimension(),"verdict":p.verdict().as_str(),"rule_id":p.rule_id(),"reference":p.reference(),"reason":p.reason(),"prerequisites":p.prerequisites()})).collect::<Vec<_>>(),
        "proof_context":context_reference(leaf.proof_context_reference(),root),
    })
}

fn port_mapping_json(port: &af_computation::ScopedPortMapping) -> Json {
    json!({"contract_port_no":port.contract_port_no,"binding":match port.binding{af_computation::ScopedPortBinding::InputExpression(id)=>json!({"input_expression":id.raw()}),af_computation::ScopedPortBinding::OutputValue(id)=>json!({"output_value":id.raw()})},"exact_type":port.exact_type.as_ref().map(schema_ref_json)})
}

fn recovery_contract_json(contract: &af_recovery::RecoveryContract) -> Json {
    json!({"replay_semantics":contract.replay_semantics.as_str(),"effect_recovery_clauses":contract.effect_recovery_clauses.iter().map(|clause|json!({
        "effect":effect_atom_json(&clause.effect),"idempotency":clause.idempotency.as_str(),
        "key_requirement":clause.key_requirement.as_ref().map(|key|json!({"stable_across_attempts":key.stable_across_attempts,"distinct_across_invocations":key.distinct_across_invocations,"bound_to_operation":key.bound_to_operation,"provider_honors":key.provider_honors})),
        "reconciliation":match clause.reconciliation{af_recovery::ReconciliationRequirement::None=>"none",af_recovery::ReconciliationRequirement::Required=>"required"},
        "compensation":match &clause.compensation{af_recovery::CompensationRequirement::None=>json!({"none":null}),af_recovery::CompensationRequirement::Required(operation)=>json!({"required":{"catalog":operation.catalog,"name":operation.name}})},
        "unknown_effect_handling":match clause.unknown_effect_handling{af_recovery::UnknownEffectHandling::Block=>"block",af_recovery::UnknownEffectHandling::ReconciliationRequired=>"reconciliationrequired"},
    })).collect::<Vec<_>>()})
}
