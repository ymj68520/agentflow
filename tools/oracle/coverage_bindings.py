"""Reviewed getter/projection bindings for D9's finite type field inventory.

Bindings are declared here, not inferred from Actual JSON. A container field
binds to a typed recursive slot; unused payload branches stay explicitly Pending.
Context references bind to same-arena association plus lossless D5 catalog/pin
recapture. This phase does not bind D10's full ProofContextManifest internals.
"""

OWNERS = {
    "Node":"af-computation::DerivedControlContractSummary",
    "Root":"af-computation::DerivedWorkflowContractSummary",
    "Closure":"af-computation::ControlTreeClosureSummary",
    "Leaf":"af-computation::ResolvedComputationLeaf",
    "LeafDimensions":"af-computation::ComponentDimensionSummary",
    "Proof":"af-computation::ControlDimensionProof",
    "LeafProof":"af-computation::LeafDimensionProof",
    "Obligation":"af-computation::InheritedControlObligation",
    "Interface":"af-computation::ControlInterfaceSummary",
    "Scope":"af-computation::ScopeBoundarySummary",
    "CompensationScope":"af-computation::CompensationScopeSummary",
    "RootRecovery":"af-computation::RootRecoverySummary",
    "SourceClosure":"af-computation::RootRecoverySourceClosureReport",
    "ProofContext":"af-computation::ProofContextReference",
    "ResolverIdentity":"af-computation::ResolverSnapshotIdentity",
    "ResolverTarget":"af-computation::ResolverSubflowIdentity",
    "Flow":"af-computation::ControlFlowSummary",
    "FlowSourceContract":"af-computation::ControlFlowContractSource",
    "RecoverySource":"af-computation::RecoverySourceSummary",
    "ExecutionSource":"af-computation::ExecutionSourceSummary",
    "OutputPort":"af-computation::ControlOutputPort",
    "OutputSource":"af-computation::ControlOutputSource",
    "Dataflow":"af-computation::SequenceDataflowDependency",
    "Occurrence":"af-computation::ScopedEffectOccurrence",
    "ResidualOccurrence":"af-computation::ScopedEffectResidual",
    "Family":"af-computation::EffectOccurrenceFamily",
    "ResourceOccurrence":"af-computation::ResourcePortOccurrence",
    "ResourceMapping":"af-computation::ScopedResourcePortMapping",
    "ResourceAlias":"af-computation::ControlResourceAlias",
    "PortMapping":"af-computation::ScopedPortMapping",
    "Await":"af-computation::AwaitLeafSummary",
    "Context":"af-portable::StaticContextProjection",
    "Digest":"af-semantic::DigestRefValue",
    "SchemaRef":"af-semantic::TypeSchemaRef",
    "Information":"af-flow::StaticInformationSummary",
    "FlowContract":"af-flow::InformationFlowContract",
    "FlowConstraint":"af-flow::FlowConstraint",
    "FlowMapping":"af-computation::FlowPortMapping",
    "FlowPartition":"af-contract::EffectFlowPartition",
    "EffectEntry":"af-effect::EffectEntry",
    "EffectResidual":"af-effect::EffectResidual",
    "Atom":"af-effect::EffectAtom",
    "Selector":"af-effect::ResourceSelector",
    "Attribute":"af-effect::ScopeBinding",
    "Bounds":"af-effect::OccurrenceBounds",
    "Usage":"af-usage::StaticUsageEnvelope",
    "UsageCoordinate":"af-usage::UsageCoordinate",
    "UsageLimit":"af-usage::UsageLimit",
    "Requirement":"af-resource::ResourceRequirement",
    "ResourceType":"af-resource::ResourceTypeRef",
    "Execution":"af-execution::ExecutionRequirements",
    "ExecutionCapability":"af-execution::ExecutionCapabilityRequirement",
    "RelationPin":"af-contract::RelationCatalogPin",
    "Checksum":"af-contract::CatalogContentChecksum",
}
ALIASES = {
    "Node":{"computation":"control_node","failures":"failures","recovery_sources":"recovery","execution_sources":"execution"},
    "Root":{"root_control":"root"},
    "Interface":{"select_decision":"selector"},
    "RootRecovery":{"proof_dimensions":"proof"},
    "ProofContext":{"same_as_root":"context"},
    "RecoverySource":{"replay_semantics":"contract","effect_recovery_clauses":"contract","effect_occurrences":"effects"},
    "Usage":{"coordinates":"bounds"},
    "OutputPort":{"parent_port":"parent_port_no","local_type_id":"local_type_id"},
    "OutputSource":{"child_node":"node","child_port":"port_no","child_value_slot":"value_slot"},
    "ExecutionCapability":{"capability":"capability"},
}

MANDATORY_D9_RECORDS = ["af-computation::DerivedWorkflowContractSummary",
    "af-computation::DerivedControlContractSummary","af-computation::ControlTreeClosureSummary",
    "af-computation::ResolvedComputationLeaf","af-computation::ComponentDimensionSummary",
    "af-computation::RootRecoverySummary","af-computation::RootRecoverySourceClosureReport",
    "af-computation::ControlDimension","af-computation::ControlProofReport",
    "af-computation::LeafProofReport","af-computation::ProofContextReference"]


def bindings(graph,projection_fields,recovery=False):
    slots={name+"."+item["field"] for name,owner in graph["types"].items() for item in owner.get("fields",[])}
    slots.update(name+"::"+variant+".$tag" for name,owner in graph["types"].items() for variant in owner.get("variants",{}))
    slots.update(name+"::"+variant+"."+item["field"] for name,owner in graph["types"].items() for variant,items in owner.get("variants",{}).items() for item in items)
    result={}
    owners=dict(OWNERS)
    if recovery:
        owners_guard_type="af-computation::RootGuardCoordinateEvidence"
        owners.update({"ActionIdentity":"af-contract::RecoveryActionIdentity","UsageBinding":"af-contract::RecoveryUsageBinding","ActionSource":"af-computation::RecoveryActionSourceIdentity","LocalBound":"af-computation::LocalActionBound","RootBound":"af-computation::RootActionBound","Multiplicity":"af-computation::AppliedMultiplicity","MultiplicityIdentity":"af-computation::ControlMultiplicityIdentity","RootAction":"af-computation::RootRecoveryAction","LocalAction":"af-computation::LocalRecoveryAction","PotentialAction":"af-computation::ConditionalCompensationAction","ControlPath":"af-computation::ControlPathStep","GuardCoordinate":"af-computation::RecoveryGuardCoordinate","ActionGuard":"af-contract::ActionGuardCompatibility","Countermodel":"af-contract::TypedGuardCountermodel","GuardAssignment":"af-contract::GuardInputAssignment","RecoveryClause":"af-recovery::EffectRecoveryClause"})
        owners["GuardCoordinate"]=owners_guard_type
    for item in projection_fields:
        path=item["field_path"]
        definition,_,field=path.partition(".")
        if definition not in owners or "." in field or "[]" in field or "?" in field:
            continue
        source=owners[definition]+"."+ALIASES.get(definition,{}).get(field,field)
        if source in slots:
            result.setdefault(source,[]).append(path)
    # Generic wrappers retain their fields through every concrete dimension;
    # these mappings are schema references, not unbounded expanded tree paths.
    manual={
        "af-computation::DerivedWorkflowContractSummary.root_recovery":["D9Facts.d9_root_recovery"],
        "af-computation::ControlDimension.value":["Node.effects","Node.failures","Node.usage","Node.resources","Node.information_flow","Node.recovery_sources","Node.execution_sources","RootRecovery.actions"],
        "af-computation::ControlDimension.proofs":["EffectDimension::variant[0].proofs","FailureDimension::variant[0].proofs","UsageDimension::variant[0].proofs"],
        "af-computation::ControlProofReport.proofs":["Node.proof","Closure.proof","RootRecovery.proof_dimensions"],
        "af-computation::LeafProofReport.dimensions":["Leaf.proof"],
        "af-computation::DimensionContribution.source_node":["ResourceDimension::variant[3].contributions[].source_node","RecoveryDimension::variant[3].contributions[].source_node","ExecutionDimension::variant[3].contributions[].source_node"],
        "af-computation::DimensionContribution.value":["ResourceDimension::variant[3].contributions[].occurrences","RecoveryDimension::variant[3].contributions[].sources","ExecutionDimension::variant[3].contributions[].sources"],
        "af-computation::SelectDecisionEvidence.expression":["Interface.select_decision?.expression"],
        "af-computation::SelectDecisionEvidence.dependency_slots":["Interface.select_decision?.dependencies"],
        "af-computation::SelectDecisionEvidence.case_ordinals":["Interface.select_decision?.case_ordinals"],
        "af-computation::SelectDecisionEvidence.exact_case":["Interface.select_decision?.exact_case"],
        "af-computation::SelectDecisionEvidence.totality_verdict":["Interface.select_decision?.totality"],
        "af-computation::RecoverySourceSummary.contract":["RecoverySource.replay_semantics","RecoverySource.effect_recovery_clauses"],
        "af-resource::ResourceTypeRef.schema":["Requirement.resource_type","ResourceCoordinate.export_resource_type"],
        "af-computation::IterateMode::While.$tag":["OperatorSemantics::variant[2].mode::variant[0].kind"],
        "af-computation::IterateMode::While.max_iterations":["OperatorSemantics::variant[2].mode::variant[0].max_iterations"],
        "af-computation::IterateMode::Foreach.$tag":["OperatorSemantics::variant[2].mode::variant[1].kind"],
        "af-computation::IterateMode::Foreach.statically_known_item_count":["OperatorSemantics::variant[2].mode::variant[1].statically_known_item_count"],
        "af-computation::IterateMode::Foreach.max_inflight":["OperatorSemantics::variant[2].mode::variant[1].max_inflight"],
        "af-computation::TimeoutOutcome::ChildCompleted.$tag":["OperatorSemantics::variant[4].possible_outcomes"],
        "af-computation::TimeoutOutcome::TimedOut.$tag":["OperatorSemantics::variant[4].possible_outcomes"],
        "af-computation::HandleClauseSummary.ordinal":["OperatorSemantics::variant[5].clauses[].ordinal"],
        "af-computation::HandleClauseSummary.matcher":["OperatorSemantics::variant[5].clauses[].matcher"],
        "af-computation::HandleClauseSummary.handler":["OperatorSemantics::variant[5].clauses[].handler"],
    }
    for source,targets in manual.items():
        if source not in slots:
            raise ValueError("stale declared getter binding "+source)
        result[source]=targets
    # Closed sum types keep both the discriminator and every typed payload.
    dimensions=[name for name in ["FailureDimension","EffectDimension","ResourceDimension","FlowDimension","UsageDimension","RecoveryDimension","ExecutionDimension","RootActions","PotentialActions","LocalActions"]]
    for variant,index in [("Complete",0),("Partial",1),("Unknown",2),("PerChild",3),("NotApplicable",4)]:
        result["af-computation::ControlDimensionValue::"+variant+".$tag"]=[name+f"::variant[{index}].state" for name in dimensions]
        for field in graph["types"]["af-computation::ControlDimensionValue"]["variants"][variant]:
            suffix={"0":"value","known_contribution":"value"}.get(field["field"],field["field"])
            candidates=[item["field_path"] for item in projection_fields if any(item["field_path"].startswith(name+f"::variant[{index}].") for name in dimensions) and item["field_path"].rsplit(".",1)[-1] in {suffix,"actions","sources","occurrences"}]
            result["af-computation::ControlDimensionValue::"+variant+"."+field["field"]]=candidates
    for variant,fields in graph["types"]["af-computation::ControlOperator"]["variants"].items():
        result["af-computation::ControlOperator::"+variant+".$tag"]=["Node.operator"]
        for field in fields:
            result["af-computation::ControlOperator::"+variant+"."+field["field"]]=["Node.operator_semantics"]
    for owner,definition in [("af-computation::ResolvedTarget","Target"),("af-effect::EffectRow","Effect"),("af-effect::OccurrenceCeiling","Upper"),("af-usage::StaticUsageBound","UsageBound"),("af-computation::LeafKind","Leaf"),("af-computation::LeafVerdict","Proof"),("af-usage::UsagePhase","UsageCoordinate")]:
        schema_fields=[item["field_path"] for item in projection_fields if item["field_path"].startswith(definition+".") or item["field_path"].startswith(definition+"::")]
        for variant,fields in graph["types"][owner]["variants"].items():
            result[owner+"::"+variant+".$tag"]=schema_fields
            for field in fields:result[owner+"::"+variant+"."+field["field"]]=schema_fields
    for name in MANDATORY_D9_RECORDS:
        for item in graph["types"][name]["fields"]:
            source=name+"."+item["field"]
            if source not in result:
                raise ValueError("unbound mandatory D9 Rust field: "+source)
    if recovery:
        for owner,definition,variants in [
            ("RootRecoveryOwner","RecoveryOwner",["CompensationScope","RootRecovery"]),
            ("RecoveryActionUpper","ActionUpper",["Known","Overflow","InvalidMultiplicity","Unknown"]),
        ]:
            qualified="af-computation::"+owner
            for index,variant in enumerate(variants):
                prefix=definition+f"::variant[{index}]."
                targets=[item["field_path"] for item in projection_fields if item["field_path"].startswith(prefix)]
                result[qualified+"::"+variant+".$tag"]=targets
                for field in graph["types"][qualified]["variants"][variant]:result[qualified+"::"+variant+"."+field["field"]]=targets
        for variant,fields in graph["types"]["af-computation::ControlPathKind"]["variants"].items():
            result["af-computation::ControlPathKind::"+variant+".$tag"]=["ControlPath.kind"]
            for field in fields:result["af-computation::ControlPathKind::"+variant+"."+field["field"]]=[item["field_path"] for item in projection_fields if item["field_path"].startswith("PathDetail::") and item["field_path"].rsplit(".",1)[-1]==field["field"]]
        for variant in ["Iterate","Retry"]:result["af-computation::ControlMultiplicityKind::"+variant+".$tag"]=["MultiplicityIdentity.kind"]
        for owner in ["af-computation::RootRecoveryAction","af-computation::LocalRecoveryAction","af-computation::ConditionalCompensationAction","af-computation::RecoveryActionSourceIdentity","af-computation::LocalActionBound","af-computation::RootActionBound","af-computation::AppliedMultiplicity","af-computation::ControlMultiplicityIdentity","af-computation::ControlPathStep","af-computation::RootGuardCoordinateEvidence","af-contract::RecoveryActionIdentity","af-contract::RecoveryUsageBinding","af-contract::ActionGuardCompatibility"]:
            for item in graph["types"][owner]["fields"]:
                if owner+"."+item["field"] not in result:raise ValueError("unbound required Recovery getter: "+owner+"."+item["field"])
    return result
