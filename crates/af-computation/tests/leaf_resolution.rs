use af_computation::{
    ControlDerivationError, ControlDimensionValue, LeafVerdict, SnapshotError,
    StaticComputationResolutionSnapshotBuilder, derive_control_contract, resolve_computation_leaf,
};
use af_contract::{
    ComponentContract, ComponentIdentity, ComponentInputPort, ComponentOutputPort,
    EffectFlowPartition, EffectQualifierProfile, EffectResourceCompatibilityCatalog,
    RecoveryActionBinding, RecoveryUsageBinding, RelationCatalogSnapshot, RelationSnapshots,
    StaticContractProofContext, SupportedEffectResourcePair, check_contract,
    component_contract_digest,
};
use af_effect::{
    CapabilityCatalog, EffectAtom, EffectCatalogUniverse, EffectEntry, EffectGuard,
    EffectOperationRef, EffectRow, EffectSignature, OccurrenceBounds, ResourceSelector,
};
use af_execution::ExecutionUniverses;
use af_failure::{FailureRow, FailureTypeRef};
use af_flow::{
    FlowCatalogs, InformationFlowContract, InstructionAuthority, StaticInformationSummary,
};
use af_idl::{CompiledModuleSet, compile_module_set};
use af_ir::{
    AwaitBody, CallSubflowBody, ComputationBody, ComputationDef, ComputationId, ContractRefEntry,
    ContractRefId, ContractRefKind, InputBinding, InvokeBody, ParallelBody, ParallelBranch,
    ParallelMode, ParallelOutputBinding, RaceBody, RaceBranch, ResourceUseBinding,
    RunParameterDecl, SelectBody, SelectCase, SemanticIrDocument, SequenceBody, SubflowIrUniverse,
    TimeoutBody,
};
use af_recovery::{
    CompensationRequirement, EffectRecoveryClause, IdempotencySemantics, ReconciliationRequirement,
    RecoveryContract, RecoveryOperationUniverse, ReplaySemantics, UnknownEffectHandling,
};
use af_resource::{
    ResourceRequirement, ResourceRequirementRow, ResourceTypeDescriptor, ResourceTypeRef,
    ResourceTypeUniverse,
};
use af_semantic::{
    DigestRefValue, ExpressionBody, ExpressionDef, SemanticPrimitiveType, SemanticTypeNode,
    SemanticTypeNodeBody, TypeSchemaUniverse,
};
use af_usage::{
    StaticUsageBound, UsageCoordinate, UsageMeterRef, UsageMeterUniverse, UsagePhase, UsageUnitRef,
};

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

    fn usage_meter(&mut self) -> UsageMeterRef {
        let meter = UsageMeterRef {
            catalog: "agentflow.meters@1".into(),
            name: "api_calls".into(),
        };
        let unit = UsageUnitRef::new("agentflow.units@1", "calls").unwrap();
        self.usage.recognize_additive(&meter, &unit).unwrap();
        meter
    }

    fn effectful_contract(&mut self) -> (ComponentContract, af_semantic::TypeSchemaRef) {
        let resource_type = self.resource_type();
        let operation = EffectOperationRef {
            catalog: "agentflow.effects@1".into(),
            name: "write".into(),
        };
        let catalog = CapabilityCatalog::new("agentflow.effects@1", ["write"]).unwrap();
        self.effects.insert(&catalog).unwrap();
        self.recovery.effects.insert(&catalog).unwrap();
        let atom = EffectAtom {
            operation: operation.clone(),
            resource_selector: ResourceSelector {
                requirement_port_no: 1,
                scope_bindings: vec![],
            },
            qualifiers: vec![],
        };
        let mut contract = empty_contract();
        contract.effects = EffectSignature {
            row: EffectRow::Closed(vec![EffectEntry {
                atom: atom.clone(),
                bounds: OccurrenceBounds::finite(1, 2).unwrap(),
                guard: EffectGuard::Unconditional,
            }]),
        };
        contract.recovery.effect_recovery_clauses = vec![EffectRecoveryClause {
            effect: atom,
            idempotency: IdempotencySemantics::Natural,
            key_requirement: None,
            reconciliation: ReconciliationRequirement::None,
            compensation: CompensationRequirement::None,
            unknown_effect_handling: UnknownEffectHandling::Block,
        }];
        contract.resources = ResourceRequirementRow {
            requirements: vec![ResourceRequirement {
                port_no: 1,
                resource_type: resource_type.clone(),
                required_attributes: vec![],
            }],
        };
        self.relations.effect_resource = Some(
            RelationCatalogSnapshot::new(
                "control-test-effects-r1",
                EffectResourceCompatibilityCatalog {
                    complete_operations: vec![operation.clone()],
                    supported_pairs: vec![SupportedEffectResourcePair {
                        operation,
                        resource_type: resource_type.clone(),
                        selector_keys: vec![],
                        allowed_qualifier_profiles: vec![EffectQualifierProfile {
                            qualifiers: vec![],
                        }],
                    }],
                },
                &self.modules,
            )
            .unwrap(),
        );
        (contract, resource_type.schema)
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

fn usage_contract(meter: UsageMeterRef, upper: u64) -> ComponentContract {
    let mut contract = empty_contract();
    contract.usage = af_usage::UsageSignature {
        limits: vec![af_usage::UsageLimit {
            meter,
            phase: UsagePhase::Forward,
            upper: af_usage::UsageUpperBound::Finite(upper),
        }],
    };
    contract
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

fn two_call_control_document(
    first: DigestRefValue,
    second: DigestRefValue,
    operator: &str,
    resource_type: Option<&af_semantic::TypeSchemaRef>,
) -> SemanticIrDocument {
    let same_contract = first == second;
    let second_contract_ref = if same_contract { 1 } else { 2 };
    let (types, values, expressions, run_parameters, body) = if operator == "select" {
        (
            vec![SemanticTypeNode {
                id: 1.into(),
                body: SemanticTypeNodeBody::Primitive(SemanticPrimitiveType::Int64),
            }],
            vec![af_semantic::ValueSlot {
                id: 1.into(),
                type_id: 1.into(),
            }],
            vec![ExpressionDef {
                id: 1.into(),
                result_type_id: 1.into(),
                body: ExpressionBody::ValueSlot(1.into()),
            }],
            vec![RunParameterDecl {
                id: 1.into(),
                name: "branch".into(),
                type_id: 1.into(),
                value_slot: 1.into(),
            }],
            ComputationBody::Select(SelectBody {
                selector: 1.into(),
                cases: vec![
                    SelectCase {
                        ordinal: 1.into(),
                        child: 2.into(),
                    },
                    SelectCase {
                        ordinal: 2.into(),
                        child: 3.into(),
                    },
                ],
            }),
        )
    } else {
        let body = match operator {
            "sequence" => ComputationBody::Sequence(SequenceBody {
                children: vec![2.into(), 3.into()],
            }),
            "parallel_all" => ComputationBody::Parallel(ParallelBody {
                mode: ParallelMode::All,
                branches: vec![
                    ParallelBranch {
                        ordinal: 1.into(),
                        child: 2.into(),
                    },
                    ParallelBranch {
                        ordinal: 2.into(),
                        child: 3.into(),
                    },
                ],
                max_inflight: Some(1),
                output_bindings: vec![],
            }),
            "parallel_fail_fast" => ComputationBody::Parallel(ParallelBody {
                mode: ParallelMode::FailFast,
                branches: vec![
                    ParallelBranch {
                        ordinal: 1.into(),
                        child: 2.into(),
                    },
                    ParallelBranch {
                        ordinal: 2.into(),
                        child: 3.into(),
                    },
                ],
                max_inflight: Some(1),
                output_bindings: vec![],
            }),
            "race" => ComputationBody::Race(RaceBody {
                branches: vec![
                    RaceBranch {
                        ordinal: 1.into(),
                        child: 2.into(),
                    },
                    RaceBranch {
                        ordinal: 2.into(),
                        child: 3.into(),
                    },
                ],
            }),
            _ => panic!("unsupported test operator"),
        };
        (vec![], vec![], vec![], vec![], body)
    };
    SemanticIrDocument {
        semantic_refs: vec![],
        types,
        values,
        expressions,
        resources: resource_type.map_or_else(Vec::new, |requirement_type| {
            vec![af_semantic::ResourceSlot {
                id: 1.into(),
                requirement_type: requirement_type.clone(),
            }]
        }),
        contracts: if same_contract {
            vec![ContractRefEntry {
                id: 1.into(),
                kind: ContractRefKind::Component,
                digest: first,
            }]
        } else {
            vec![
                ContractRefEntry {
                    id: 1.into(),
                    kind: ContractRefKind::Component,
                    digest: first,
                },
                ContractRefEntry {
                    id: 2.into(),
                    kind: ContractRefKind::Component,
                    digest: second,
                },
            ]
        },
        run_parameters,
        workflow_outputs: vec![],
        computations: vec![
            ComputationDef {
                id: 1.into(),
                outputs: vec![],
                body,
            },
            ComputationDef {
                id: 2.into(),
                outputs: vec![],
                body: ComputationBody::Invoke(InvokeBody {
                    contract: 1.into(),
                    inputs: vec![],
                    resources: resource_type.map_or_else(Vec::new, |_| {
                        vec![ResourceUseBinding {
                            port_no: 1.into(),
                            resource_slot: 1.into(),
                        }]
                    }),
                }),
            },
            ComputationDef {
                id: 3.into(),
                outputs: vec![],
                body: ComputationBody::Invoke(InvokeBody {
                    contract: second_contract_ref.into(),
                    inputs: vec![],
                    resources: resource_type.map_or_else(Vec::new, |_| {
                        vec![ResourceUseBinding {
                            port_no: 1.into(),
                            resource_slot: 1.into(),
                        }]
                    }),
                }),
            },
        ],
        root: 1.into(),
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

fn sequence_output_input_document(
    producer_pin: DigestRefValue,
    consumer_pin: DigestRefValue,
    schema: af_semantic::TypeSchemaRef,
) -> SemanticIrDocument {
    SemanticIrDocument {
        semantic_refs: vec![],
        types: vec![SemanticTypeNode {
            id: 1.into(),
            body: SemanticTypeNodeBody::Schema(schema),
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
        contracts: vec![
            ContractRefEntry {
                id: 1.into(),
                kind: ContractRefKind::Component,
                digest: producer_pin,
            },
            ContractRefEntry {
                id: 2.into(),
                kind: ContractRefKind::Component,
                digest: consumer_pin,
            },
        ],
        run_parameters: vec![],
        workflow_outputs: vec![],
        computations: vec![
            ComputationDef {
                id: 1.into(),
                outputs: vec![],
                body: ComputationBody::Sequence(SequenceBody {
                    children: vec![2.into(), 3.into()],
                }),
            },
            ComputationDef {
                id: 2.into(),
                outputs: vec![af_ir::model::ComputationOutputBinding {
                    port_no: 1.into(),
                    value_slot: 1.into(),
                }],
                body: ComputationBody::Invoke(InvokeBody {
                    contract: 1.into(),
                    inputs: vec![],
                    resources: vec![],
                }),
            },
            ComputationDef {
                id: 3.into(),
                outputs: vec![],
                body: ComputationBody::Invoke(InvokeBody {
                    contract: 2.into(),
                    inputs: vec![InputBinding {
                        port_no: 1.into(),
                        expression: 1.into(),
                    }],
                    resources: vec![],
                }),
            },
        ],
        root: 1.into(),
    }
}

fn parallel_output_document(
    first_pin: DigestRefValue,
    second_pin: DigestRefValue,
    schema: af_semantic::TypeSchemaRef,
) -> SemanticIrDocument {
    SemanticIrDocument {
        semantic_refs: vec![],
        types: vec![SemanticTypeNode {
            id: 1.into(),
            body: SemanticTypeNodeBody::Schema(schema),
        }],
        values: vec![
            af_semantic::ValueSlot {
                id: 1.into(),
                type_id: 1.into(),
            },
            af_semantic::ValueSlot {
                id: 2.into(),
                type_id: 1.into(),
            },
            af_semantic::ValueSlot {
                id: 3.into(),
                type_id: 1.into(),
            },
        ],
        expressions: vec![],
        resources: vec![],
        contracts: vec![
            ContractRefEntry {
                id: 1.into(),
                kind: ContractRefKind::Component,
                digest: first_pin,
            },
            ContractRefEntry {
                id: 2.into(),
                kind: ContractRefKind::Component,
                digest: second_pin,
            },
        ],
        run_parameters: vec![],
        workflow_outputs: vec![],
        computations: vec![
            ComputationDef {
                id: 1.into(),
                outputs: vec![af_ir::model::ComputationOutputBinding {
                    port_no: 1.into(),
                    value_slot: 3.into(),
                }],
                body: ComputationBody::Parallel(ParallelBody {
                    mode: ParallelMode::All,
                    branches: vec![
                        ParallelBranch {
                            ordinal: 1.into(),
                            child: 2.into(),
                        },
                        ParallelBranch {
                            ordinal: 2.into(),
                            child: 3.into(),
                        },
                    ],
                    max_inflight: None,
                    output_bindings: vec![ParallelOutputBinding {
                        parent_port_no: 1.into(),
                        branch_ordinal: 2.into(),
                        child_port_no: 1.into(),
                    }],
                }),
            },
            ComputationDef {
                id: 2.into(),
                outputs: vec![af_ir::model::ComputationOutputBinding {
                    port_no: 1.into(),
                    value_slot: 1.into(),
                }],
                body: ComputationBody::Invoke(InvokeBody {
                    contract: 1.into(),
                    inputs: vec![],
                    resources: vec![],
                }),
            },
            ComputationDef {
                id: 3.into(),
                outputs: vec![af_ir::model::ComputationOutputBinding {
                    port_no: 1.into(),
                    value_slot: 2.into(),
                }],
                body: ComputationBody::Invoke(InvokeBody {
                    contract: 2.into(),
                    inputs: vec![],
                    resources: vec![],
                }),
            },
        ],
        root: 1.into(),
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
fn sequence_records_phase3c_cross_sibling_dataflow_without_claiming_value_level_flow_proof() {
    let fixture = Context::new();
    let schema = fixture
        .schemas
        .get("agentflow.semantic.core@1/TypeSchemaRef")
        .unwrap()
        .reference();
    let mut producer = empty_contract();
    producer.component_identity.name = "acme.producer".into();
    producer.outputs = vec![ComponentOutputPort {
        port_no: 1,
        type_ref: schema.clone(),
    }];
    let mut consumer = empty_contract();
    consumer.component_identity.name = "acme.consumer".into();
    consumer.inputs = vec![ComponentInputPort {
        port_no: 1,
        type_ref: schema.clone(),
    }];
    let context = fixture.view();
    let checked_producer = check_contract(&producer, &context).unwrap();
    let checked_consumer = check_contract(&consumer, &context).unwrap();
    let producer_pin = digest_value(&checked_producer, &context);
    let consumer_pin = digest_value(&checked_consumer, &context);
    let source = sequence_output_input_document(producer_pin, consumer_pin, schema);
    let checked_ir = source.check(&fixture.schemas).unwrap();
    let subflows = SubflowIrUniverse::new();
    let snapshot = StaticComputationResolutionSnapshotBuilder::new(&context, &subflows)
        .component(&checked_producer)
        .unwrap()
        .component(&checked_consumer)
        .unwrap()
        .build();
    let producer_leaf =
        resolve_computation_leaf(&checked_ir, ComputationId::from_raw(2), &snapshot).unwrap();
    let consumer_leaf =
        resolve_computation_leaf(&checked_ir, ComputationId::from_raw(3), &snapshot).unwrap();
    let summary = derive_control_contract(
        &checked_ir,
        ComputationId::from_raw(1),
        &[producer_leaf, consumer_leaf],
        &context,
    )
    .unwrap();
    assert_eq!(summary.sequence_dataflow_dependencies().len(), 1);
    let edge = &summary.sequence_dataflow_dependencies()[0];
    assert_eq!(edge.producer_child(), ComputationId::from_raw(2));
    assert_eq!(edge.consumer_child(), ComputationId::from_raw(3));
    assert_eq!(edge.value_slot(), 1.into());
    assert_eq!(edge.expression(), 1.into());
    assert!(
        summary
            .information_flow()
            .proofs()
            .iter()
            .any(|proof| proof.verdict() == LeafVerdict::Unproven)
    );
}

#[test]
fn parallel_output_uses_the_frozen_explicit_branch_mapping() {
    let fixture = Context::new();
    let schema = fixture
        .schemas
        .get("agentflow.semantic.core@1/TypeSchemaRef")
        .unwrap()
        .reference();
    let mut first = empty_contract();
    first.component_identity.name = "acme.first".into();
    first.outputs = vec![ComponentOutputPort {
        port_no: 1,
        type_ref: schema.clone(),
    }];
    let mut second = empty_contract();
    second.component_identity.name = "acme.second".into();
    second.outputs = vec![ComponentOutputPort {
        port_no: 1,
        type_ref: schema.clone(),
    }];
    let context = fixture.view();
    let checked_first = check_contract(&first, &context).unwrap();
    let checked_second = check_contract(&second, &context).unwrap();
    let source = parallel_output_document(
        digest_value(&checked_first, &context),
        digest_value(&checked_second, &context),
        schema,
    );
    let checked_ir = source.check(&fixture.schemas).unwrap();
    let subflows = SubflowIrUniverse::new();
    let snapshot = StaticComputationResolutionSnapshotBuilder::new(&context, &subflows)
        .component(&checked_first)
        .unwrap()
        .component(&checked_second)
        .unwrap()
        .build();
    let left =
        resolve_computation_leaf(&checked_ir, ComputationId::from_raw(2), &snapshot).unwrap();
    let right =
        resolve_computation_leaf(&checked_ir, ComputationId::from_raw(3), &snapshot).unwrap();
    let summary = derive_control_contract(
        &checked_ir,
        ComputationId::from_raw(1),
        &[left, right],
        &context,
    )
    .unwrap();
    assert_eq!(
        summary
            .proof()
            .dimensions()
            .iter()
            .find(|proof| proof.dimension() == "typed-output-interface")
            .unwrap()
            .verdict(),
        LeafVerdict::Proven
    );
    let ControlDimensionValue::Complete(outputs) = summary.interface().outputs() else {
        panic!("typed output mapping should be available");
    };
    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0].parent_value_slot(), 3.into());
    assert_eq!(outputs[0].sources().len(), 1);
    assert_eq!(
        outputs[0].sources()[0].child_node(),
        ComputationId::from_raw(3)
    );
    assert_eq!(outputs[0].sources()[0].child_value_slot(), 2.into());
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
fn control_derivation_rejects_leaf_summaries_from_another_exact_context_or_ir() {
    let first = Context::new();
    let first_context = first.view();
    let checked_contract = check_contract(&empty_contract(), &first_context).unwrap();
    let pin = digest_value(&checked_contract, &first_context);
    let source_one = document(pin.clone(), 1);
    let ir_one = source_one.check(&first.schemas).unwrap();
    let subflows = SubflowIrUniverse::new();
    let snapshot = StaticComputationResolutionSnapshotBuilder::new(&first_context, &subflows)
        .component(&checked_contract)
        .unwrap()
        .build();
    let leaf = resolve_computation_leaf(&ir_one, ComputationId::from_raw(1), &snapshot).unwrap();

    let second = Context::new();
    let second_context = second.view();
    let source_two = document(pin, 2);
    let ir_two = source_two.check(&second.schemas).unwrap();
    let context_error = derive_control_contract(
        &ir_one,
        ComputationId::from_raw(1),
        std::slice::from_ref(&leaf),
        &second_context,
    )
    .unwrap_err();
    assert!(
        matches!(context_error, ControlDerivationError::LeafContextMismatch(id) if id == ComputationId::from_raw(1))
    );
    let digest_error =
        derive_control_contract(&ir_two, ComputationId::from_raw(2), &[leaf], &first_context)
            .unwrap_err();
    assert!(
        matches!(digest_error, ControlDerivationError::CallerDigestMismatch(id) if id == ComputationId::from_raw(1))
    );
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
    let source = await_document(signal_type.clone());
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

    let checked_contract = check_contract(&empty_contract(), &context).unwrap();
    let component_pin = digest_value(&checked_contract, &context);
    let mut composed_source = await_document(signal_type);
    composed_source.contracts = vec![ContractRefEntry {
        id: 1.into(),
        kind: ContractRefKind::Component,
        digest: component_pin,
    }];
    let await_body = composed_source.computations[0].body.clone();
    composed_source.computations = vec![
        ComputationDef {
            id: 1.into(),
            outputs: vec![],
            body: ComputationBody::Sequence(SequenceBody {
                children: vec![2.into(), 3.into()],
            }),
        },
        ComputationDef {
            id: 2.into(),
            outputs: vec![],
            body: await_body,
        },
        ComputationDef {
            id: 3.into(),
            outputs: vec![],
            body: ComputationBody::Invoke(InvokeBody {
                contract: 1.into(),
                inputs: vec![],
                resources: vec![],
            }),
        },
    ];
    composed_source.root = ComputationId::from_raw(1);
    let composed_ir = composed_source.check(&fixture.schemas).unwrap();
    let composed_snapshot = StaticComputationResolutionSnapshotBuilder::new(&context, &subflows)
        .component(&checked_contract)
        .unwrap()
        .build();
    let await_leaf =
        resolve_computation_leaf(&composed_ir, ComputationId::from_raw(2), &composed_snapshot)
            .unwrap();
    let invoke_leaf =
        resolve_computation_leaf(&composed_ir, ComputationId::from_raw(3), &composed_snapshot)
            .unwrap();
    let composed = derive_control_contract(
        &composed_ir,
        ComputationId::from_raw(1),
        &[await_leaf, invoke_leaf],
        &context,
    )
    .unwrap();
    let ControlDimensionValue::Partial {
        known_contribution: Some(flow),
        ..
    } = composed.information_flow().value()
    else {
        panic!("known child Flow facts and unknown Await influence must both remain explicit");
    };
    assert_eq!(
        flow.unknown_external_children(),
        &[ComputationId::from_raw(2)]
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
fn child_requires_evidence_propagates_without_erasing_a_known_parent_effect_envelope() {
    let fixture = Context::new();
    let context = fixture.view();
    let checked = check_contract(&empty_contract(), &context).unwrap();
    let component_pin = digest_value(&checked, &context);
    let target_source = subflow_target_document(component_pin.clone());
    let target_ir = target_source.check(&fixture.schemas).unwrap();
    let mut subflows = SubflowIrUniverse::new();
    let target_digest = subflows.insert(&target_ir, &fixture.modules).unwrap();
    let caller_source = SemanticIrDocument {
        semantic_refs: vec![],
        types: vec![],
        values: vec![],
        expressions: vec![],
        resources: vec![],
        contracts: vec![
            ContractRefEntry {
                id: 1.into(),
                kind: ContractRefKind::Subflow,
                digest: target_digest.clone(),
            },
            ContractRefEntry {
                id: 2.into(),
                kind: ContractRefKind::Component,
                digest: component_pin,
            },
        ],
        run_parameters: vec![],
        workflow_outputs: vec![],
        computations: vec![
            ComputationDef {
                id: 1.into(),
                outputs: vec![],
                body: ComputationBody::Sequence(SequenceBody {
                    children: vec![2.into(), 3.into()],
                }),
            },
            ComputationDef {
                id: 2.into(),
                outputs: vec![],
                body: ComputationBody::CallSubflow(CallSubflowBody {
                    contract: 1.into(),
                    inputs: vec![],
                    resources: vec![],
                }),
            },
            ComputationDef {
                id: 3.into(),
                outputs: vec![],
                body: ComputationBody::Invoke(InvokeBody {
                    contract: 2.into(),
                    inputs: vec![],
                    resources: vec![],
                }),
            },
        ],
        root: 1.into(),
    };
    let caller_ir = caller_source.check(&fixture.schemas).unwrap();
    let snapshot = StaticComputationResolutionSnapshotBuilder::new(&context, &subflows)
        .component(&checked)
        .unwrap()
        .subflow_export(&target_digest, &checked)
        .unwrap()
        .build();
    let child_subflow =
        resolve_computation_leaf(&caller_ir, ComputationId::from_raw(2), &snapshot).unwrap();
    let child_invoke =
        resolve_computation_leaf(&caller_ir, ComputationId::from_raw(3), &snapshot).unwrap();
    let summary = derive_control_contract(
        &caller_ir,
        ComputationId::from_raw(1),
        &[child_subflow, child_invoke],
        &context,
    )
    .unwrap();
    assert_eq!(summary.proof().overall(), LeafVerdict::RequiresEvidence);
    assert!(
        summary
            .proof()
            .dimensions()
            .iter()
            .any(|proof| proof.source_node() == ComputationId::from_raw(2)
                && proof.dimension() == "subflow-body-conformance"
                && proof.verdict() == LeafVerdict::RequiresEvidence)
    );
    let ControlDimensionValue::Complete(effects) = summary.effects().value() else {
        panic!("known child Effect declarations remain composable")
    };
    assert_eq!(effects.row, EffectRow::Closed(vec![]));
}

#[test]
fn unsupported_control_remains_unknown_instead_of_becoming_an_empty_child() {
    let fixture = Context::new();
    let context = fixture.view();
    let checked = check_contract(&empty_contract(), &context).unwrap();
    let pin = digest_value(&checked, &context);
    let source = SemanticIrDocument {
        semantic_refs: vec![],
        types: vec![SemanticTypeNode {
            id: 1.into(),
            body: SemanticTypeNodeBody::Primitive(SemanticPrimitiveType::Duration),
        }],
        values: vec![],
        expressions: vec![ExpressionDef {
            id: 1.into(),
            result_type_id: 1.into(),
            body: ExpressionBody::Constant(af_semantic::PrimitiveConstant::Duration(
                af_core::Duration::new(10),
            )),
        }],
        resources: vec![],
        contracts: vec![ContractRefEntry {
            id: 1.into(),
            kind: ContractRefKind::Component,
            digest: pin,
        }],
        run_parameters: vec![],
        workflow_outputs: vec![],
        computations: vec![
            ComputationDef {
                id: 1.into(),
                outputs: vec![],
                body: ComputationBody::Timeout(TimeoutBody {
                    child: 2.into(),
                    timeout: 1.into(),
                }),
            },
            ComputationDef {
                id: 2.into(),
                outputs: vec![],
                body: ComputationBody::Invoke(InvokeBody {
                    contract: 1.into(),
                    inputs: vec![],
                    resources: vec![],
                }),
            },
        ],
        root: 1.into(),
    };
    let ir = source.check(&fixture.schemas).unwrap();
    let subflows = SubflowIrUniverse::new();
    let snapshot = StaticComputationResolutionSnapshotBuilder::new(&context, &subflows)
        .component(&checked)
        .unwrap()
        .build();
    let leaf = resolve_computation_leaf(&ir, ComputationId::from_raw(2), &snapshot).unwrap();
    let summary =
        derive_control_contract(&ir, ComputationId::from_raw(1), &[leaf], &context).unwrap();
    assert_eq!(
        summary.operator(),
        &af_computation::ControlOperator::Unsupported {
            opcode: "timeout".into()
        }
    );
    assert_eq!(summary.proof().overall(), LeafVerdict::Unproven);
    assert!(
        matches!(summary.effects().value(), ControlDimensionValue::Unknown { unresolved_sources, .. } if unresolved_sources.contains(&ComputationId::from_raw(1)))
    );
    assert!(
        matches!(summary.children()[0].effects().value(), ControlDimensionValue::Complete(EffectSignature { row: EffectRow::Closed(entries) }) if entries.is_empty())
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

#[test]
fn independent_python_control_oracle_freezes_all_four_verdict_classes() {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../spec/conformance/computation-control/manifest.json"
    ))
    .unwrap();
    assert_eq!(manifest["case_count"].as_u64(), Some(58));
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../spec/conformance/computation-control");
    let groups = [
        "golden",
        "negative",
        "sequence",
        "select",
        "parallel",
        "race",
        "proof",
        "equivalence",
    ];
    let mut ids = std::collections::BTreeSet::new();
    let mut verdicts = std::collections::BTreeSet::new();
    for group in groups {
        for entry in std::fs::read_dir(root.join(group)).unwrap() {
            let path = entry.unwrap().path();
            let value: serde_json::Value =
                serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
            ids.insert(value["case_id"].as_str().unwrap().to_owned());
            verdicts.insert(value["expected"]["verdict"].as_str().unwrap().to_owned());
        }
    }
    assert_eq!(ids.len(), 58);
    assert_eq!(
        verdicts,
        ["proven", "requires_evidence", "unproven", "violation"]
            .into_iter()
            .map(str::to_owned)
            .collect()
    );
}

#[test]
fn four_control_operators_use_their_distinct_static_usage_algebras() {
    let mut fixture = Context::new();
    let meter = fixture.usage_meter();
    let contract_a = usage_contract(meter.clone(), 10);
    let contract_b = usage_contract(meter.clone(), 20);
    let context = fixture.view();
    let checked_a = check_contract(&contract_a, &context).unwrap();
    let checked_b = check_contract(&contract_b, &context).unwrap();
    let pin_a = digest_value(&checked_a, &context);
    let pin_b = digest_value(&checked_b, &context);
    let subflows = SubflowIrUniverse::new();
    let snapshot = StaticComputationResolutionSnapshotBuilder::new(&context, &subflows)
        .component(&checked_a)
        .unwrap()
        .component(&checked_b)
        .unwrap()
        .build();
    let coordinate = UsageCoordinate {
        phase: UsagePhase::Forward,
        meter,
    };

    for (operator, expected) in [
        ("sequence", 30),
        ("select", 20),
        ("parallel_all", 30),
        ("parallel_fail_fast", 30),
        ("race", 30),
    ] {
        let source = two_call_control_document(pin_a.clone(), pin_b.clone(), operator, None);
        let checked_ir = source.check(&fixture.schemas).unwrap();
        let leaf_a =
            resolve_computation_leaf(&checked_ir, ComputationId::from_raw(2), &snapshot).unwrap();
        let leaf_b =
            resolve_computation_leaf(&checked_ir, ComputationId::from_raw(3), &snapshot).unwrap();
        let summary = derive_control_contract(
            &checked_ir,
            ComputationId::from_raw(1),
            &[leaf_a, leaf_b],
            &context,
        )
        .unwrap();
        let ControlDimensionValue::Complete(envelope) = summary.usage().value() else {
            panic!(
                "{operator} should retain a complete usage envelope for complete child declarations"
            );
        };
        assert_eq!(
            envelope.get(&coordinate),
            StaticUsageBound::KnownFinite(expected),
            "{operator}"
        );
        if operator == "select" {
            assert_eq!(
                summary.proof().overall(),
                LeafVerdict::Unproven,
                "the int64 selector has no finite-case exhaustiveness proof"
            );
        }
        if operator == "race" {
            let ControlDimensionValue::PerChild {
                contributions,
                unresolved_sources,
            } = summary.execution().value()
            else {
                panic!("X must remain per-child, never a single-worker aggregate");
            };
            assert!(unresolved_sources.is_empty());
            assert_eq!(contributions.len(), 2);
            assert_eq!(contributions[0].source_node(), ComputationId::from_raw(2));
            assert_eq!(contributions[1].source_node(), ComputationId::from_raw(3));
            assert_eq!(
                contributions[0].value()[0].target(),
                &af_computation::ResolvedTarget::Component {
                    identity: contract_a.component_identity.clone(),
                    contract_digest: pin_a.clone(),
                }
            );
            assert_eq!(
                contributions[1].value()[0].target(),
                &af_computation::ResolvedTarget::Component {
                    identity: contract_b.component_identity.clone(),
                    contract_digest: pin_b.clone(),
                }
            );
        }
    }
}

#[test]
fn all_four_operators_union_typed_failure_rows_without_inventing_control_failures() {
    let fixture = Context::new();
    let failure_a = fixture
        .schemas
        .get("agentflow.semantic.core@1/TypeSchemaRef")
        .unwrap()
        .reference();
    let failure_b = fixture
        .schemas
        .get("agentflow.semantic.core@1/ValueSlot")
        .unwrap()
        .reference();
    let mut body_a = empty_contract();
    body_a.failures = FailureRow {
        failures: vec![FailureTypeRef {
            payload_type: failure_a,
        }],
    };
    let mut body_b = empty_contract();
    body_b.failures = FailureRow {
        failures: vec![FailureTypeRef {
            payload_type: failure_b,
        }],
    };
    let context = fixture.view();
    let checked_a = check_contract(&body_a, &context).unwrap();
    let checked_b = check_contract(&body_b, &context).unwrap();
    let subflows = SubflowIrUniverse::new();
    let snapshot = StaticComputationResolutionSnapshotBuilder::new(&context, &subflows)
        .component(&checked_a)
        .unwrap()
        .component(&checked_b)
        .unwrap()
        .build();
    for operator in [
        "sequence",
        "select",
        "parallel_all",
        "parallel_fail_fast",
        "race",
    ] {
        let source = two_call_control_document(
            digest_value(&checked_a, &context),
            digest_value(&checked_b, &context),
            operator,
            None,
        );
        let ir = source.check(&fixture.schemas).unwrap();
        let left = resolve_computation_leaf(&ir, ComputationId::from_raw(2), &snapshot).unwrap();
        let right = resolve_computation_leaf(&ir, ComputationId::from_raw(3), &snapshot).unwrap();
        let summary =
            derive_control_contract(&ir, ComputationId::from_raw(1), &[left, right], &context)
                .unwrap();
        let ControlDimensionValue::Complete(row) = summary.failures().value() else {
            panic!("typed child Failure rows remain complete")
        };
        assert_eq!(row.failures.len(), 2, "{operator}");
        assert_eq!(
            summary.failures().proofs()[0].verdict(),
            LeafVerdict::Proven
        );
    }
}

#[test]
fn effect_bounds_weaken_conditional_lower_but_retain_all_occurrences_and_resources() {
    let mut fixture = Context::new();
    let (contract, resource_type) = fixture.effectful_contract();
    let context = fixture.view();
    let checked = check_contract(&contract, &context).unwrap();
    let pin = digest_value(&checked, &context);
    let subflows = SubflowIrUniverse::new();
    let snapshot = StaticComputationResolutionSnapshotBuilder::new(&context, &subflows)
        .component(&checked)
        .unwrap()
        .build();

    for (operator, expected_lower, expected_upper) in [
        ("sequence", 1, 4),
        ("parallel_all", 2, 4),
        ("parallel_fail_fast", 0, 4),
        ("race", 0, 4),
        ("select", 0, 2),
    ] {
        let source =
            two_call_control_document(pin.clone(), pin.clone(), operator, Some(&resource_type));
        let checked_ir = source.check(&fixture.schemas).unwrap();
        let leaf_left =
            resolve_computation_leaf(&checked_ir, ComputationId::from_raw(2), &snapshot).unwrap();
        let leaf_right =
            resolve_computation_leaf(&checked_ir, ComputationId::from_raw(3), &snapshot).unwrap();
        let summary = derive_control_contract(
            &checked_ir,
            ComputationId::from_raw(1),
            &[leaf_left, leaf_right],
            &context,
        )
        .unwrap();
        let ControlDimensionValue::Complete(signature) = summary.effects().value() else {
            panic!(
                "complete child Effect rows should produce a typed conservative envelope for {operator}"
            );
        };
        let EffectRow::Closed(entries) = &signature.row else {
            panic!("expected the two exact known effects to remain closed");
        };
        assert_eq!(
            entries.len(),
            1,
            "same full AtomKey is merged only in the algebraic envelope for {operator}"
        );
        assert_eq!(entries[0].bounds.lower, expected_lower, "{operator}");
        assert_eq!(
            entries[0].bounds.upper,
            af_effect::OccurrenceCeiling::Finite(expected_upper),
            "{operator}"
        );
        assert_eq!(
            summary.effect_occurrences().len(),
            2,
            "source occurrence provenance is never merged for {operator}"
        );
        assert_ne!(
            summary.effect_occurrences()[0].computation,
            summary.effect_occurrences()[1].computation
        );
        assert_eq!(
            summary.effect_occurrences()[0].declared.atom.qualifiers,
            summary.effect_occurrences()[1].declared.atom.qualifiers
        );
        assert_eq!(
            summary.resource_aliases().len(),
            1,
            "the same caller slot is recorded as an alias across two distinct call sites"
        );
        assert_eq!(summary.resource_aliases()[0].occurrences().len(), 2);
        if operator == "race" {
            let ControlDimensionValue::PerChild { contributions, .. } = summary.recovery().value()
            else {
                panic!("recovery declarations must remain per child");
            };
            let recovery_sources = contributions
                .iter()
                .flat_map(af_computation::DimensionContribution::value)
                .collect::<Vec<_>>();
            assert_eq!(
                recovery_sources.len(),
                2,
                "RACE loser K declarations remain in the summary"
            );
            assert!(
                recovery_sources.iter().all(|source| source
                    .contract()
                    .effect_recovery_clauses
                    .len()
                    == 1)
            );
        }
    }
}
