use af_computation::{
    LeafVerdict, SnapshotError, StaticComputationResolutionSnapshotBuilder,
    resolve_computation_leaf,
};
use af_contract::{
    ComponentContract, ComponentIdentity, ComponentInputPort, ComponentOutputPort,
    EffectFlowPartition, EffectResourceCompatibilityCatalog, RecoveryActionBinding,
    RecoveryUsageBinding, RelationCatalogSnapshot, RelationSnapshots, StaticContractProofContext,
    check_contract, component_contract_digest,
};
use af_effect::{EffectCatalogUniverse, EffectOperationRef, EffectRow, EffectSignature};
use af_execution::ExecutionUniverses;
use af_failure::FailureRow;
use af_flow::{
    FlowCatalogs, InformationFlowContract, InstructionAuthority, StaticInformationSummary,
};
use af_idl::{CompiledModuleSet, compile_module_set};
use af_ir::{
    AwaitBody, CallSubflowBody, ComputationBody, ComputationDef, ComputationId, ContractRefEntry,
    ContractRefId, ContractRefKind, InputBinding, InvokeBody, ResourceUseBinding, RunParameterDecl,
    SemanticIrDocument, SequenceBody, SubflowIrUniverse,
};
use af_recovery::{RecoveryContract, RecoveryOperationUniverse, ReplaySemantics};
use af_resource::{
    ResourceRequirement, ResourceRequirementRow, ResourceTypeDescriptor, ResourceTypeRef,
    ResourceTypeUniverse,
};
use af_semantic::{
    DigestRefValue, ExpressionBody, ExpressionDef, SemanticPrimitiveType, SemanticTypeNode,
    SemanticTypeNodeBody, TypeSchemaUniverse,
};
use af_usage::UsageMeterUniverse;

fn modules() -> CompiledModuleSet {
    let sources = vec![
        (
            "spec/idl/core/machine-primitives.afidl",
            include_str!("../../../spec/idl/core/machine-primitives.afidl"),
        ),
        (
            "spec/idl/specification-registry.afidl",
            include_str!("../../../spec/idl/specification-registry.afidl"),
        ),
        (
            "spec/idl/semantic-core.afidl",
            include_str!("../../../spec/idl/semantic-core.afidl"),
        ),
        (
            "spec/idl/semantic-ir.afidl",
            include_str!("../../../spec/idl/semantic-ir.afidl"),
        ),
        (
            "spec/idl/effect-signature.afidl",
            include_str!("../../../spec/idl/effect-signature.afidl"),
        ),
        (
            "spec/idl/failure-row.afidl",
            include_str!("../../../spec/idl/failure-row.afidl"),
        ),
        (
            "spec/idl/resource-requirement.afidl",
            include_str!("../../../spec/idl/resource-requirement.afidl"),
        ),
        (
            "spec/idl/information-flow.afidl",
            include_str!("../../../spec/idl/information-flow.afidl"),
        ),
        (
            "spec/idl/usage-signature.afidl",
            include_str!("../../../spec/idl/usage-signature.afidl"),
        ),
        (
            "spec/idl/cost-projection.afidl",
            include_str!("../../../spec/idl/cost-projection.afidl"),
        ),
        (
            "spec/idl/recovery-contract.afidl",
            include_str!("../../../spec/idl/recovery-contract.afidl"),
        ),
        (
            "spec/idl/execution-requirements.afidl",
            include_str!("../../../spec/idl/execution-requirements.afidl"),
        ),
        (
            "spec/idl/component-contract.afidl",
            include_str!("../../../spec/idl/component-contract.afidl"),
        ),
        (
            "spec/idl/effect-resource-compatibility.afidl",
            include_str!("../../../spec/idl/effect-resource-compatibility.afidl"),
        ),
        (
            "spec/idl/recovery-usage-coverage.afidl",
            include_str!("../../../spec/idl/recovery-usage-coverage.afidl"),
        ),
        (
            "spec/idl/flow-execution-relation.afidl",
            include_str!("../../../spec/idl/flow-execution-relation.afidl"),
        ),
    ]
    .into_iter()
    .map(|(path, source)| (path.to_string(), source.to_string()))
    .collect::<Vec<_>>();
    compile_module_set(&sources).unwrap()
}

struct Context {
    modules: CompiledModuleSet,
    schemas: TypeSchemaUniverse,
    effects: EffectCatalogUniverse,
    resources: ResourceTypeUniverse,
    flow: FlowCatalogs,
    usage: UsageMeterUniverse,
    recovery: RecoveryOperationUniverse,
    execution: ExecutionUniverses,
    relations: RelationSnapshots,
}

impl Context {
    fn new() -> Self {
        let modules = modules();
        let mut schemas = TypeSchemaUniverse::new();
        let mut resources = ResourceTypeUniverse::new();
        for module in modules.modules() {
            schemas.insert_compiled(module).unwrap();
            resources.insert_compiled(module).unwrap();
        }
        Self {
            modules,
            schemas,
            effects: EffectCatalogUniverse::new(),
            resources,
            flow: FlowCatalogs::new(),
            usage: UsageMeterUniverse::new(),
            recovery: RecoveryOperationUniverse::new(),
            execution: ExecutionUniverses::default(),
            relations: RelationSnapshots::default(),
        }
    }
    fn view(&self) -> StaticContractProofContext<'_> {
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

    fn resource_type(&mut self) -> ResourceTypeRef {
        let schema = self
            .schemas
            .get("agentflow.semantic.core@1/TypeSchemaRef")
            .unwrap()
            .reference();
        self.resources
            .recognize(ResourceTypeDescriptor {
                specification_id: schema.specification_id.clone(),
                type_name: schema.type_name.clone(),
                schema_hex: schema.schema_digest.hex.clone(),
                recognized_attributes: std::collections::BTreeSet::new(),
            })
            .unwrap();
        ResourceTypeRef { schema }
    }
}

fn empty_contract() -> ComponentContract {
    ComponentContract {
        component_identity: ComponentIdentity {
            catalog: "agentflow.components@1".into(),
            name: "acme.transform".into(),
        },
        inputs: vec![],
        outputs: vec![],
        failures: FailureRow::empty(),
        effects: EffectSignature {
            row: EffectRow::Closed(vec![]),
        },
        resources: ResourceRequirementRow::empty(),
        information_flow: InformationFlowContract {
            confidentiality_restrictions: vec![],
            summary: StaticInformationSummary {
                sources: vec![],
                taints: vec![],
                instruction_authority: InstructionAuthority::None,
                influence_unknown: false,
            },
            flow_constraints: vec![],
        },
        usage: af_usage::UsageSignature::empty(),
        recovery: RecoveryContract {
            replay_semantics: ReplaySemantics::Recompute,
            effect_recovery_clauses: vec![],
        },
        execution: af_execution::ExecutionRequirements::empty(),
        effect_flow_partitions: Vec::<EffectFlowPartition>::new(),
        recovery_action_bindings: Vec::<RecoveryActionBinding>::new(),
        recovery_usage_bindings: Vec::<RecoveryUsageBinding>::new(),
    }
}

fn digest_value(
    checked: &af_contract::CheckedComponentContract<'_, '_>,
    context: &StaticContractProofContext<'_>,
) -> DigestRefValue {
    let digest = component_contract_digest(checked, context).unwrap();
    DigestRefValue {
        algorithm: "sha256".into(),
        domain: digest.domain().as_str().into(),
        hex: digest.hex().into(),
    }
}

fn document(digest: DigestRefValue, call_count: usize) -> SemanticIrDocument {
    let (root, computations) = if call_count == 1 {
        (
            ComputationId::from_raw(1),
            vec![ComputationDef {
                id: ComputationId::from_raw(1),
                outputs: vec![],
                body: ComputationBody::Invoke(InvokeBody {
                    contract: ContractRefId::from_raw(1),
                    inputs: vec![],
                    resources: vec![],
                }),
            }],
        )
    } else {
        (
            ComputationId::from_raw(1),
            vec![
                ComputationDef {
                    id: ComputationId::from_raw(1),
                    outputs: vec![],
                    body: ComputationBody::Sequence(SequenceBody {
                        children: vec![ComputationId::from_raw(2), ComputationId::from_raw(3)],
                    }),
                },
                ComputationDef {
                    id: ComputationId::from_raw(2),
                    outputs: vec![],
                    body: ComputationBody::Invoke(InvokeBody {
                        contract: ContractRefId::from_raw(1),
                        inputs: vec![],
                        resources: vec![],
                    }),
                },
                ComputationDef {
                    id: ComputationId::from_raw(3),
                    outputs: vec![],
                    body: ComputationBody::Invoke(InvokeBody {
                        contract: ContractRefId::from_raw(1),
                        inputs: vec![],
                        resources: vec![],
                    }),
                },
            ],
        )
    };
    SemanticIrDocument {
        semantic_refs: vec![],
        types: vec![],
        values: vec![],
        expressions: vec![],
        resources: vec![],
        contracts: vec![ContractRefEntry {
            id: ContractRefId::from_raw(1),
            kind: ContractRefKind::Component,
            digest,
        }],
        run_parameters: vec![],
        workflow_outputs: vec![],
        computations,
        root,
    }
}

fn typed_invoke_document(
    digest: DigestRefValue,
    input_type: af_semantic::TypeSchemaRef,
    output_type: af_semantic::TypeSchemaRef,
) -> SemanticIrDocument {
    SemanticIrDocument {
        semantic_refs: vec![],
        types: vec![
            SemanticTypeNode {
                id: 1.into(),
                body: SemanticTypeNodeBody::Schema(input_type),
            },
            SemanticTypeNode {
                id: 2.into(),
                body: SemanticTypeNodeBody::Schema(output_type),
            },
        ],
        values: vec![
            af_semantic::ValueSlot {
                id: 1.into(),
                type_id: 1.into(),
            },
            af_semantic::ValueSlot {
                id: 2.into(),
                type_id: 2.into(),
            },
        ],
        expressions: vec![ExpressionDef {
            id: 1.into(),
            result_type_id: 1.into(),
            body: ExpressionBody::ValueSlot(1.into()),
        }],
        resources: vec![],
        contracts: vec![ContractRefEntry {
            id: 1.into(),
            kind: ContractRefKind::Component,
            digest,
        }],
        run_parameters: vec![RunParameterDecl {
            id: 1.into(),
            name: "request".into(),
            type_id: 1.into(),
            value_slot: 1.into(),
        }],
        workflow_outputs: vec![],
        computations: vec![ComputationDef {
            id: ComputationId::from_raw(1),
            outputs: vec![af_ir::model::ComputationOutputBinding {
                port_no: 1.into(),
                value_slot: 2.into(),
            }],
            body: ComputationBody::Invoke(InvokeBody {
                contract: 1.into(),
                inputs: vec![InputBinding {
                    port_no: 1.into(),
                    expression: 1.into(),
                }],
                resources: vec![],
            }),
        }],
        root: ComputationId::from_raw(1),
    }
}

fn resource_alias_document(
    digest: DigestRefValue,
    resource_type: af_semantic::TypeSchemaRef,
) -> SemanticIrDocument {
    SemanticIrDocument {
        semantic_refs: vec![],
        types: vec![],
        values: vec![],
        expressions: vec![],
        resources: vec![af_semantic::ResourceSlot {
            id: 1.into(),
            requirement_type: resource_type,
        }],
        contracts: vec![ContractRefEntry {
            id: 1.into(),
            kind: ContractRefKind::Component,
            digest,
        }],
        run_parameters: vec![],
        workflow_outputs: vec![],
        computations: vec![ComputationDef {
            id: ComputationId::from_raw(1),
            outputs: vec![],
            body: ComputationBody::Invoke(InvokeBody {
                contract: 1.into(),
                inputs: vec![],
                resources: vec![
                    ResourceUseBinding {
                        port_no: 1.into(),
                        resource_slot: 1.into(),
                    },
                    ResourceUseBinding {
                        port_no: 2.into(),
                        resource_slot: 1.into(),
                    },
                ],
            }),
        }],
        root: ComputationId::from_raw(1),
    }
}

fn await_document(signal_type: af_semantic::TypeSchemaRef) -> SemanticIrDocument {
    SemanticIrDocument {
        semantic_refs: vec![],
        types: vec![SemanticTypeNode {
            id: 1.into(),
            body: SemanticTypeNodeBody::Primitive(SemanticPrimitiveType::String),
        }],
        values: vec![af_semantic::ValueSlot {
            id: 1.into(),
            type_id: 1.into(),
        }],
        expressions: vec![ExpressionDef {
            id: 1.into(),
            result_type_id: 1.into(),
            body: ExpressionBody::ValueSlot(1.into()),
        }],
        resources: vec![],
        contracts: vec![],
        run_parameters: vec![RunParameterDecl {
            id: 1.into(),
            name: "correlation".into(),
            type_id: 1.into(),
            value_slot: 1.into(),
        }],
        workflow_outputs: vec![],
        computations: vec![ComputationDef {
            id: ComputationId::from_raw(1),
            outputs: vec![],
            body: ComputationBody::Await(AwaitBody {
                signal_type,
                correlation: 1.into(),
            }),
        }],
        root: ComputationId::from_raw(1),
    }
}

fn subflow_target_document(component_pin: DigestRefValue) -> SemanticIrDocument {
    SemanticIrDocument {
        semantic_refs: vec![],
        types: vec![],
        values: vec![],
        expressions: vec![],
        resources: vec![],
        contracts: vec![ContractRefEntry {
            id: 1.into(),
            kind: ContractRefKind::Component,
            digest: component_pin,
        }],
        run_parameters: vec![],
        workflow_outputs: vec![],
        computations: vec![ComputationDef {
            id: ComputationId::from_raw(1),
            outputs: vec![],
            body: ComputationBody::Invoke(InvokeBody {
                contract: 1.into(),
                inputs: vec![],
                resources: vec![],
            }),
        }],
        root: ComputationId::from_raw(1),
    }
}

fn call_subflow_document(target_pin: DigestRefValue) -> SemanticIrDocument {
    SemanticIrDocument {
        semantic_refs: vec![],
        types: vec![],
        values: vec![],
        expressions: vec![],
        resources: vec![],
        contracts: vec![ContractRefEntry {
            id: 1.into(),
            kind: ContractRefKind::Subflow,
            digest: target_pin,
        }],
        run_parameters: vec![],
        workflow_outputs: vec![],
        computations: vec![ComputationDef {
            id: ComputationId::from_raw(1),
            outputs: vec![],
            body: ComputationBody::CallSubflow(CallSubflowBody {
                contract: 1.into(),
                inputs: vec![],
                resources: vec![],
            }),
        }],
        root: ComputationId::from_raw(1),
    }
}

fn subflow_dependency_document(dependency_pin: DigestRefValue) -> SemanticIrDocument {
    SemanticIrDocument {
        semantic_refs: vec![],
        types: vec![],
        values: vec![],
        expressions: vec![],
        resources: vec![],
        contracts: vec![ContractRefEntry {
            id: 1.into(),
            kind: ContractRefKind::Subflow,
            digest: dependency_pin,
        }],
        run_parameters: vec![],
        workflow_outputs: vec![],
        computations: vec![ComputationDef {
            id: ComputationId::from_raw(1),
            outputs: vec![],
            body: ComputationBody::CallSubflow(CallSubflowBody {
                contract: 1.into(),
                inputs: vec![],
                resources: vec![],
            }),
        }],
        root: ComputationId::from_raw(1),
    }
}

fn primitive_input_subflow_target(component_pin: DigestRefValue) -> SemanticIrDocument {
    SemanticIrDocument {
        semantic_refs: vec![],
        types: vec![SemanticTypeNode {
            id: 1.into(),
            body: SemanticTypeNodeBody::Primitive(SemanticPrimitiveType::String),
        }],
        values: vec![af_semantic::ValueSlot {
            id: 1.into(),
            type_id: 1.into(),
        }],
        expressions: vec![],
        resources: vec![],
        contracts: vec![ContractRefEntry {
            id: 1.into(),
            kind: ContractRefKind::Component,
            digest: component_pin,
        }],
        run_parameters: vec![RunParameterDecl {
            id: 1.into(),
            name: "query".into(),
            type_id: 1.into(),
            value_slot: 1.into(),
        }],
        workflow_outputs: vec![],
        computations: vec![ComputationDef {
            id: ComputationId::from_raw(1),
            outputs: vec![],
            body: ComputationBody::Invoke(InvokeBody {
                contract: 1.into(),
                inputs: vec![],
                resources: vec![],
            }),
        }],
        root: ComputationId::from_raw(1),
    }
}

fn primitive_input_subflow_caller(target_pin: DigestRefValue) -> SemanticIrDocument {
    SemanticIrDocument {
        semantic_refs: vec![],
        types: vec![SemanticTypeNode {
            id: 1.into(),
            body: SemanticTypeNodeBody::Primitive(SemanticPrimitiveType::String),
        }],
        values: vec![af_semantic::ValueSlot {
            id: 1.into(),
            type_id: 1.into(),
        }],
        expressions: vec![ExpressionDef {
            id: 1.into(),
            result_type_id: 1.into(),
            body: ExpressionBody::ValueSlot(1.into()),
        }],
        resources: vec![],
        contracts: vec![ContractRefEntry {
            id: 1.into(),
            kind: ContractRefKind::Subflow,
            digest: target_pin,
        }],
        run_parameters: vec![RunParameterDecl {
            id: 1.into(),
            name: "query".into(),
            type_id: 1.into(),
            value_slot: 1.into(),
        }],
        workflow_outputs: vec![],
        computations: vec![ComputationDef {
            id: ComputationId::from_raw(1),
            outputs: vec![],
            body: ComputationBody::CallSubflow(CallSubflowBody {
                contract: 1.into(),
                inputs: vec![InputBinding {
                    port_no: 1.into(),
                    expression: 1.into(),
                }],
                resources: vec![],
            }),
        }],
        root: ComputationId::from_raw(1),
    }
}

#[test]
fn exact_invoke_contract_resolves_and_same_target_has_distinct_call_sites() {
    let fixture = Context::new();
    let context = fixture.view();
    let checked_contract = check_contract(&empty_contract(), &context).unwrap();
    let pinned = digest_value(&checked_contract, &context);
    let ir_source = document(pinned, 2);
    let ir = ir_source.check(&fixture.schemas).unwrap();
    let subflows = SubflowIrUniverse::new();
    let snapshot = StaticComputationResolutionSnapshotBuilder::new(&context, &subflows)
        .component(&checked_contract)
        .unwrap()
        .build();

    let left = resolve_computation_leaf(&ir, ComputationId::from_raw(2), &snapshot).unwrap();
    let right = resolve_computation_leaf(&ir, ComputationId::from_raw(3), &snapshot).unwrap();
    assert_eq!(left.proof().overall(), LeafVerdict::Proven);
    assert_eq!(right.proof().overall(), LeafVerdict::Proven);
    assert_ne!(left.computation(), right.computation());
    assert_eq!(left.target(), right.target());
    assert!(
        left.proof_context_reference()
            .is_same_context(right.proof_context_reference())
    );
    assert_eq!(left.caller_semantic_ir_digest().domain, "semantic-ir");
    assert_eq!(left.dimensions().unwrap().failures, FailureRow::empty());
}

#[test]
fn component_public_ports_require_exact_pins_and_reuse_dataflow_scope() {
    let fixture = Context::new();
    let public_type = fixture
        .schemas
        .get("agentflow.semantic.core@1/TypeSchemaRef")
        .unwrap()
        .reference();
    let other_type = fixture
        .schemas
        .get("agentflow.semantic.core@1/ValueSlot")
        .unwrap()
        .reference();
    let mut contract = empty_contract();
    contract.inputs = vec![ComponentInputPort {
        port_no: 1,
        type_ref: public_type.clone(),
    }];
    contract.outputs = vec![ComponentOutputPort {
        port_no: 1,
        type_ref: public_type.clone(),
    }];
    let context = fixture.view();
    let checked_contract = check_contract(&contract, &context).unwrap();
    let pin = digest_value(&checked_contract, &context);
    let good_source = typed_invoke_document(pin.clone(), public_type.clone(), public_type.clone());
    let good_ir = good_source.check(&fixture.schemas).unwrap();
    let subflows = SubflowIrUniverse::new();
    let snapshot = StaticComputationResolutionSnapshotBuilder::new(&context, &subflows)
        .component(&checked_contract)
        .unwrap()
        .build();
    let good = resolve_computation_leaf(&good_ir, ComputationId::from_raw(1), &snapshot).unwrap();
    assert_eq!(good.proof().overall(), LeafVerdict::Proven);
    assert_eq!(
        good.input_mappings()[0].exact_type.as_ref(),
        Some(&public_type)
    );
    assert_eq!(
        good.output_mappings()[0].exact_type.as_ref(),
        Some(&public_type)
    );

    let wrong_source = typed_invoke_document(pin, other_type, public_type);
    let wrong_ir = wrong_source.check(&fixture.schemas).unwrap();
    let wrong = resolve_computation_leaf(&wrong_ir, ComputationId::from_raw(1), &snapshot).unwrap();
    assert_eq!(wrong.proof().overall(), LeafVerdict::Violation);
    assert!(
        wrong
            .proof()
            .dimensions()
            .iter()
            .any(|item| item.dimension() == "type-compatibility"
                && item.verdict() == LeafVerdict::Violation)
    );
}

#[test]
fn resource_port_aliases_are_explicit_and_do_not_change_static_rows() {
    let mut fixture = Context::new();
    let resource_type = fixture.resource_type();
    let resource_type_pin = resource_type.schema.clone();
    let mut contract = empty_contract();
    contract.resources = ResourceRequirementRow {
        requirements: vec![
            ResourceRequirement {
                port_no: 1,
                resource_type: resource_type.clone(),
                required_attributes: vec![],
            },
            ResourceRequirement {
                port_no: 2,
                resource_type,
                required_attributes: vec![],
            },
        ],
    };
    let context = fixture.view();
    let checked_contract = check_contract(&contract, &context).unwrap();
    let pin = digest_value(&checked_contract, &context);
    let source = resource_alias_document(pin, resource_type_pin);
    let ir = source.check(&fixture.schemas).unwrap();
    let subflows = SubflowIrUniverse::new();
    let snapshot = StaticComputationResolutionSnapshotBuilder::new(&context, &subflows)
        .component(&checked_contract)
        .unwrap()
        .build();
    let leaf = resolve_computation_leaf(&ir, ComputationId::from_raw(1), &snapshot).unwrap();
    assert_eq!(leaf.proof().overall(), LeafVerdict::Proven);
    assert_eq!(
        leaf.resource_aliases(),
        &[af_computation::ResourceAliasGroup {
            resource_slot: 1.into(),
            component_ports: vec![1, 2],
        }]
    );
    assert_eq!(leaf.dimensions().unwrap().resources, contract.resources);
}

#[test]
fn wrong_exact_pin_is_unproven_and_never_falls_back_to_an_available_contract() {
    let fixture = Context::new();
    let context = fixture.view();
    let checked_contract = check_contract(&empty_contract(), &context).unwrap();
    let mut pinned = digest_value(&checked_contract, &context);
    pinned.hex.replace_range(
        0..1,
        if pinned.hex.starts_with('0') {
            "1"
        } else {
            "0"
        },
    );
    let ir_source = document(pinned, 1);
    let ir = ir_source.check(&fixture.schemas).unwrap();
    let subflows = SubflowIrUniverse::new();
    let snapshot = StaticComputationResolutionSnapshotBuilder::new(&context, &subflows)
        .component(&checked_contract)
        .unwrap()
        .build();
    let leaf = resolve_computation_leaf(&ir, ComputationId::from_raw(1), &snapshot).unwrap();
    assert_eq!(leaf.proof().overall(), LeafVerdict::Unproven);
    assert!(leaf.dimensions().is_none());
}

#[test]
fn checked_component_digest_is_context_independent_but_checked_value_is_not_reusable() {
    let first = Context::new();
    let first_context = first.view();
    let first_checked = check_contract(&empty_contract(), &first_context).unwrap();
    let first_digest = digest_value(&first_checked, &first_context);

    let mut second = Context::new();
    second.relations.effect_resource = Some(
        RelationCatalogSnapshot::new(
            "relation-r1",
            EffectResourceCompatibilityCatalog {
                complete_operations: vec![EffectOperationRef {
                    catalog: "agentflow.effects@1".into(),
                    name: "read".into(),
                }],
                supported_pairs: vec![],
            },
            &second.modules,
        )
        .unwrap(),
    );
    let second_context = second.view();
    let second_checked = check_contract(&empty_contract(), &second_context).unwrap();
    let second_digest = digest_value(&second_checked, &second_context);
    assert_eq!(first_digest, second_digest);

    let subflows = SubflowIrUniverse::new();
    let Err(error) = StaticComputationResolutionSnapshotBuilder::new(&second_context, &subflows)
        .component(&first_checked)
    else {
        panic!("a checked proof from a distinct context must not enter this snapshot");
    };
    assert_eq!(error, SnapshotError::ProofContextMismatch);
}

#[test]
fn await_is_typed_but_partial_and_keeps_unknown_external_provenance() {
    let fixture = Context::new();
    let context = fixture.view();
    let signal_type = fixture
        .schemas
        .get("agentflow.semantic.core@1/TypeSchemaRef")
        .unwrap()
        .reference();
    let source = await_document(signal_type);
    let ir = source.check(&fixture.schemas).unwrap();
    let subflows = SubflowIrUniverse::new();
    let snapshot = StaticComputationResolutionSnapshotBuilder::new(&context, &subflows).build();
    let leaf = resolve_computation_leaf(&ir, ComputationId::from_raw(1), &snapshot).unwrap();
    assert_eq!(leaf.proof().overall(), LeafVerdict::Unproven);
    let await_summary = leaf.await_summary().unwrap();
    assert_eq!(
        await_summary.timeout,
        af_computation::AwaitTimeoutSemantics::NoImplicitTimeout
    );
    assert_eq!(
        await_summary.provenance,
        af_computation::AwaitProvenanceStatus::UnknownExternalInfluence
    );
    assert_eq!(
        await_summary.authority,
        af_computation::AwaitAuthorityStatus::NotInferred
    );
}

#[test]
fn subflow_export_is_exact_but_body_conformance_remains_requires_evidence() {
    let fixture = Context::new();
    let context = fixture.view();
    let checked_contract = check_contract(&empty_contract(), &context).unwrap();
    let component_pin = digest_value(&checked_contract, &context);
    let target_source = subflow_target_document(component_pin);
    let target_ir = target_source.check(&fixture.schemas).unwrap();
    let mut subflows = SubflowIrUniverse::new();
    let target_digest = subflows.insert(&target_ir, &fixture.modules).unwrap();

    let caller_source = call_subflow_document(target_digest.clone());
    let caller_ir = caller_source.check(&fixture.schemas).unwrap();
    let snapshot = StaticComputationResolutionSnapshotBuilder::new(&context, &subflows)
        .subflow_export(&target_digest, &checked_contract)
        .unwrap()
        .build();
    let leaf = resolve_computation_leaf(&caller_ir, ComputationId::from_raw(1), &snapshot).unwrap();
    assert_eq!(leaf.proof().overall(), LeafVerdict::RequiresEvidence);
    assert!(
        leaf.proof()
            .dimensions()
            .iter()
            .any(|item| item.dimension() == "subflow-export-contract"
                && item.verdict() == LeafVerdict::Proven)
    );
    assert!(
        leaf.proof()
            .dimensions()
            .iter()
            .any(|item| item.dimension() == "subflow-body-conformance"
                && item.verdict() == LeafVerdict::RequiresEvidence)
    );
}

#[test]
fn missing_subflow_dependency_is_retained_as_unproven_closure() {
    let fixture = Context::new();
    let context = fixture.view();
    let missing = DigestRefValue {
        algorithm: "sha256".into(),
        domain: "semantic-ir".into(),
        hex: "f".repeat(64),
    };
    let target_source = subflow_dependency_document(missing.clone());
    let target_ir = target_source.check(&fixture.schemas).unwrap();
    let mut subflows = SubflowIrUniverse::new();
    let target_digest = subflows.insert(&target_ir, &fixture.modules).unwrap();
    assert_eq!(
        subflows.target(&target_digest).unwrap().dependencies(),
        &[missing]
    );

    let caller_source = call_subflow_document(target_digest);
    let caller_ir = caller_source.check(&fixture.schemas).unwrap();
    let snapshot = StaticComputationResolutionSnapshotBuilder::new(&context, &subflows).build();
    let leaf = resolve_computation_leaf(&caller_ir, ComputationId::from_raw(1), &snapshot).unwrap();
    assert!(
        leaf.proof()
            .dimensions()
            .iter()
            .any(|item| item.dimension() == "subflow-dependency-closure"
                && item.verdict() == LeafVerdict::Unproven)
    );
}

#[test]
fn subflow_primitive_interface_is_not_guessed_into_public_schema_type() {
    let fixture = Context::new();
    let public_schema = fixture
        .schemas
        .get("agentflow.semantic.core@1/TypeSchemaRef")
        .unwrap()
        .reference();
    let context = fixture.view();
    let mut export_body = empty_contract();
    export_body.inputs = vec![ComponentInputPort {
        port_no: 1,
        type_ref: public_schema,
    }];
    let checked_export = check_contract(&export_body, &context).unwrap();
    let component_pin = digest_value(&checked_export, &context);
    let target_source = primitive_input_subflow_target(component_pin);
    let target_ir = target_source.check(&fixture.schemas).unwrap();
    let mut subflows = SubflowIrUniverse::new();
    let target_digest = subflows.insert(&target_ir, &fixture.modules).unwrap();
    let caller_source = primitive_input_subflow_caller(target_digest.clone());
    let caller_ir = caller_source.check(&fixture.schemas).unwrap();
    let snapshot = StaticComputationResolutionSnapshotBuilder::new(&context, &subflows)
        .subflow_export(&target_digest, &checked_export)
        .unwrap()
        .build();
    let leaf = resolve_computation_leaf(&caller_ir, ComputationId::from_raw(1), &snapshot).unwrap();
    assert_eq!(
        leaf.proof()
            .dimensions()
            .iter()
            .find(|item| item.dimension() == "subflow-interface")
            .unwrap()
            .verdict(),
        LeafVerdict::Proven
    );
    assert_eq!(
        leaf.proof()
            .dimensions()
            .iter()
            .find(|item| item.dimension() == "subflow-export-contract")
            .unwrap()
            .verdict(),
        LeafVerdict::Unproven
    );
}

#[test]
fn independent_python_oracle_corpus_covers_all_leaf_verdicts() {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../spec/conformance/computation-leaf/manifest.json"
    ))
    .unwrap();
    assert_eq!(manifest["case_count"].as_u64(), Some(34));
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../spec/conformance/computation-leaf");
    let mut verdicts = std::collections::BTreeSet::new();
    let mut case_ids = std::collections::BTreeSet::new();
    for group in [
        "golden",
        "negative",
        "invoke",
        "await",
        "subflow",
        "mapping",
        "context",
        "equivalence",
    ] {
        let directory = root.join(group);
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|extension| extension != "json") {
                continue;
            }
            let value: serde_json::Value =
                serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
            if let Some(case_id) = value["case_id"].as_str() {
                case_ids.insert(case_id.to_owned());
                if let Some(verdict) = value["expected"]["verdict"].as_str() {
                    verdicts.insert(verdict.to_owned());
                }
            }
        }
    }
    assert_eq!(case_ids.len(), 34);
    assert_eq!(
        verdicts,
        ["proven", "requires_evidence", "unproven", "violation"]
            .into_iter()
            .map(str::to_owned)
            .collect()
    );
}
