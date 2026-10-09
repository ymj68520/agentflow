use af_contract::{
    ComponentContract, ComponentIdentity, ConfidentialityExecutionRule, EffectQualifierProfile,
    EffectResourceCompatibilityCatalog, RecoveryActionBinding, RecoveryActionIdentity,
    RecoveryActionKind, RecoveryUsageBinding, RecoveryUsageCoverageCatalog,
    RecoveryUsageCoverageRule, RegionPair, RegionPairPolicy, RelationCatalogSnapshot,
    RelationSnapshots, SandboxFloor, ScopeValueRule, SelectorKeyRule, StaticContractProofContext,
    SupportedEffectResourcePair, TaintExecutionRule, TrustDomainPolicy, analyze_contract,
    check_contract, component_contract_digest,
};
use af_effect::{
    CapabilityCatalog, EffectAtom, EffectCatalogUniverse, EffectEntry, EffectGuard,
    EffectGuardInputEquals, EffectIdempotencyQualifier, EffectOperationRef, EffectQualifier,
    EffectRow, EffectSignature, OccurrenceBounds, ResourceSelector, ScopeBinding,
};
use af_execution::ExecutionUniverses;
use af_failure::FailureRow;
use af_flow::{
    FlowCatalogs, InformationFlowContract, InstructionAuthority, StaticInformationSummary,
};
use af_idl::{
    CanonicalBody, CanonicalTypeExpr, CanonicalTypeSchema, CompiledModuleSet, Primitive,
    compile_module_set,
};
use af_recovery::{
    CompensationRequirement, EffectRecoveryClause, IdempotencySemantics, ReconciliationRequirement,
    RecoveryContract, RecoveryOperationUniverse, ReplaySemantics, UnknownEffectHandling,
};
use af_resource::{
    ResourceAttributeConstraint, ResourceRequirement, ResourceRequirementRow,
    ResourceTypeDescriptor, ResourceTypeRef, ResourceTypeUniverse,
};
use af_semantic::{DigestRefValue, PrimitiveConstant, TypeSchemaRef, TypeSchemaUniverse};
use af_usage::{
    UsageLimit, UsageMeterRef, UsageMeterUniverse, UsagePhase, UsageSignature, UsageUnitRef,
    UsageUpperBound,
};
use std::collections::BTreeSet;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
fn modules() -> CompiledModuleSet {
    let sources: Vec<(String, String)> = vec![
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
    .collect();
    compile_module_set(&sources).unwrap()
}

struct FixtureContext {
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

impl FixtureContext {
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

    fn use_fixture_resource(&mut self) -> ResourceTypeRef {
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
                recognized_attributes: BTreeSet::from(["tenant".to_string()]),
            })
            .unwrap();
        ResourceTypeRef { schema }
    }
}

fn recovery_contract_with_usage(
    fixture: &mut FixtureContext,
    recovery_ceiling: Option<u64>,
) -> ComponentContract {
    let resource_type = fixture.use_fixture_resource();
    let forward_operation = EffectOperationRef {
        catalog: "agentflow.effects@1".into(),
        name: "charge".into(),
    };
    let action_operation = EffectOperationRef {
        catalog: "agentflow.effects@1".into(),
        name: "reverse".into(),
    };
    let operations = CapabilityCatalog::new("agentflow.effects@1", ["charge", "reverse"]).unwrap();
    fixture.effects.insert(&operations).unwrap();
    fixture.recovery.effects.insert(&operations).unwrap();
    let forward = EffectAtom {
        operation: forward_operation.clone(),
        resource_selector: ResourceSelector {
            requirement_port_no: 1,
            scope_bindings: vec![],
        },
        qualifiers: vec![],
    };
    let reverse = EffectAtom {
        operation: action_operation.clone(),
        resource_selector: ResourceSelector {
            requirement_port_no: 1,
            scope_bindings: vec![],
        },
        qualifiers: vec![],
    };
    let action = RecoveryActionIdentity {
        forward_effect: forward.clone(),
        action_kind: RecoveryActionKind::Compensation,
        action_effect: reverse.clone(),
    };
    let meter = UsageMeterRef {
        catalog: "agentflow.meters@1".into(),
        name: "api_calls".into(),
    };
    let unit = UsageUnitRef::new("agentflow.units@1", "calls").unwrap();
    fixture.usage.recognize_additive(&meter, &unit).unwrap();

    let mut contract = empty_contract(false);
    contract.effects = EffectSignature {
        row: EffectRow::Closed(vec![
            EffectEntry {
                atom: forward.clone(),
                bounds: OccurrenceBounds::finite(1, 2).unwrap(),
                guard: EffectGuard::Unconditional,
            },
            EffectEntry {
                atom: reverse.clone(),
                bounds: OccurrenceBounds::finite(0, 2).unwrap(),
                guard: EffectGuard::Unconditional,
            },
        ]),
    };
    contract.resources = ResourceRequirementRow {
        requirements: vec![ResourceRequirement {
            port_no: 1,
            resource_type: resource_type.clone(),
            required_attributes: vec![],
        }],
    };
    contract.recovery = RecoveryContract {
        replay_semantics: ReplaySemantics::Recompute,
        effect_recovery_clauses: vec![EffectRecoveryClause {
            effect: forward.clone(),
            idempotency: IdempotencySemantics::Natural,
            key_requirement: None,
            reconciliation: ReconciliationRequirement::None,
            compensation: CompensationRequirement::Required(action_operation.clone()),
            unknown_effect_handling: UnknownEffectHandling::Block,
        }],
    };
    contract.recovery_action_bindings = vec![RecoveryActionBinding {
        identity: action.clone(),
    }];
    contract.recovery_usage_bindings = vec![RecoveryUsageBinding {
        action: action.clone(),
        activations_per_forward: OccurrenceBounds::finite(1, 1).unwrap(),
        attempts_per_activation: OccurrenceBounds::finite(1, 2).unwrap(),
        per_attempt_usage: UsageSignature {
            limits: vec![UsageLimit {
                meter: meter.clone(),
                phase: UsagePhase::Recovery,
                upper: UsageUpperBound::Finite(3),
            }],
        },
    }];
    if let Some(ceiling) = recovery_ceiling {
        contract.usage = UsageSignature {
            limits: vec![UsageLimit {
                meter: meter.clone(),
                phase: UsagePhase::Recovery,
                upper: UsageUpperBound::Finite(ceiling),
            }],
        };
    }
    let pair = |operation| SupportedEffectResourcePair {
        operation,
        resource_type: resource_type.clone(),
        selector_keys: vec![],
        allowed_qualifier_profiles: vec![EffectQualifierProfile { qualifiers: vec![] }],
    };
    fixture.relations.effect_resource = Some(
        RelationCatalogSnapshot::new(
            "fixture-effect-resource-r1",
            EffectResourceCompatibilityCatalog {
                complete_operations: vec![forward_operation, action_operation],
                supported_pairs: vec![
                    pair(forward.operation.clone()),
                    pair(reverse.operation.clone()),
                ],
            },
            &fixture.modules,
        )
        .unwrap(),
    );
    fixture.relations.recovery_usage = Some(
        RelationCatalogSnapshot::new(
            "fixture-recovery-usage-r1",
            RecoveryUsageCoverageCatalog {
                rules: vec![RecoveryUsageCoverageRule {
                    action,
                    complete_meters: vec![meter],
                }],
            },
            &fixture.modules,
        )
        .unwrap(),
    );
    contract
}

fn empty_contract(influence_unknown: bool) -> ComponentContract {
    ComponentContract {
        component_identity: ComponentIdentity {
            catalog: "agentflow.components@1".into(),
            name: "acme.transform".into(),
        },
        inputs: vec![],
        outputs: vec![],
        failures: FailureRow { failures: vec![] },
        effects: EffectSignature {
            row: EffectRow::Closed(vec![]),
        },
        resources: af_resource::ResourceRequirementRow::empty(),
        information_flow: InformationFlowContract {
            confidentiality_restrictions: vec![],
            summary: StaticInformationSummary {
                sources: vec![],
                taints: vec![],
                instruction_authority: InstructionAuthority::None,
                influence_unknown,
            },
            flow_constraints: vec![],
        },
        usage: UsageSignature::empty(),
        recovery: RecoveryContract {
            replay_semantics: ReplaySemantics::Recompute,
            effect_recovery_clauses: vec![],
        },
        execution: af_execution::ExecutionRequirements::empty(),
        effect_flow_partitions: vec![],
        recovery_action_bindings: vec![],
        recovery_usage_bindings: vec![],
    }
}

fn guard_input_type(
    fixture: &mut FixtureContext,
    name: &str,
    primitive: Primitive,
) -> TypeSchemaRef {
    let schema = CanonicalTypeSchema {
        schema: "agentflow.test.guard-types@1".into(),
        name: name.into(),
        body: CanonicalBody::Alias(CanonicalTypeExpr::Primitive(primitive)),
    };
    fixture.schemas.insert_type(&schema).unwrap();
    fixture
        .schemas
        .get(&format!("agentflow.test.guard-types@1/{name}"))
        .unwrap()
        .reference()
}

fn recovery_contract_with_guards(
    fixture: &mut FixtureContext,
    forward_guard: EffectGuard,
    action_guard: EffectGuard,
    input_types: Vec<(u32, TypeSchemaRef)>,
) -> ComponentContract {
    let mut contract = recovery_contract_with_usage(fixture, Some(12));
    contract.inputs = input_types
        .into_iter()
        .map(|(port_no, type_ref)| af_contract::ComponentInputPort { port_no, type_ref })
        .collect();
    let EffectRow::Closed(entries) = &mut contract.effects.row else {
        unreachable!("guard fixture starts from a closed row")
    };
    entries[0].guard = forward_guard;
    entries[1].guard = action_guard;
    contract
}

#[test]
fn recovery_action_guard_compatibility_uses_frozen_direction_and_typed_witnesses() {
    let guard =
        |port_no, expected| EffectGuard::InputEquals(EffectGuardInputEquals { port_no, expected });

    let mut unconditional_fixture = FixtureContext::new();
    let unconditional = recovery_contract_with_guards(
        &mut unconditional_fixture,
        EffectGuard::Unconditional,
        EffectGuard::Unconditional,
        vec![],
    );
    let report = analyze_contract(&unconditional, &unconditional_fixture.view());
    let relation = report.action_guard_compatibility.first().unwrap();
    assert_eq!(relation.verdict(), af_contract::ProofVerdict::Proven);
    assert!(relation.countermodel().is_none());
    assert!(check_contract(&unconditional, &unconditional_fixture.view()).is_ok());

    let mut wider_fixture = FixtureContext::new();
    let wider_int64 = guard_input_type(&mut wider_fixture, "GuardInt64", Primitive::Int64);
    let forward_narrower = recovery_contract_with_guards(
        &mut wider_fixture,
        guard(1, PrimitiveConstant::Int64(0)),
        EffectGuard::Unconditional,
        vec![(1, wider_int64.clone())],
    );
    let report = analyze_contract(&forward_narrower, &wider_fixture.view());
    let relation = report.action_guard_compatibility.first().unwrap();
    assert_eq!(relation.verdict(), af_contract::ProofVerdict::Proven);
    assert!(relation.countermodel().is_none());
    assert!(check_contract(&forward_narrower, &wider_fixture.view()).is_ok());

    let mut equal_fixture = FixtureContext::new();
    let equal_int64 = guard_input_type(&mut equal_fixture, "GuardInt64", Primitive::Int64);
    let equal = recovery_contract_with_guards(
        &mut equal_fixture,
        guard(1, PrimitiveConstant::Int64(0)),
        guard(1, PrimitiveConstant::Int64(0)),
        vec![(1, equal_int64)],
    );
    let report = analyze_contract(&equal, &equal_fixture.view());
    let relation = report.action_guard_compatibility.first().unwrap();
    assert_eq!(relation.verdict(), af_contract::ProofVerdict::Proven);
    assert!(relation.countermodel().is_none());
    assert!(check_contract(&equal, &equal_fixture.view()).is_ok());

    let mut outside_fixture = FixtureContext::new();
    let int64 = guard_input_type(&mut outside_fixture, "GuardInt64", Primitive::Int64);
    let unconditional_outside = recovery_contract_with_guards(
        &mut outside_fixture,
        EffectGuard::Unconditional,
        guard(1, PrimitiveConstant::Int64(0)),
        vec![(1, int64)],
    );
    let report = analyze_contract(&unconditional_outside, &outside_fixture.view());
    let relation = report.action_guard_compatibility.first().unwrap();
    assert_eq!(relation.verdict(), af_contract::ProofVerdict::Violation);
    let witness = relation.countermodel().unwrap();
    assert_eq!(witness.source_contract(), &unconditional_outside);
    assert_eq!(
        witness.reachability(),
        af_contract::GuardCountermodelReachability::WellTypedComponentInputDomain
    );
    assert_eq!(witness.related_rule_id(), "AF-CONTRACT-0022");
    assert_eq!(witness.assignments().len(), 1);
    assert_eq!(witness.assignments()[0].port_no(), 1);
    assert_eq!(witness.assignments()[0].type_ref().type_name, "GuardInt64");
    assert_eq!(
        witness.assignments()[0].value(),
        &PrimitiveConstant::Int64(1)
    );
    assert!(check_contract(&unconditional_outside, &outside_fixture.view()).is_err());

    let mut same_port_fixture = FixtureContext::new();
    let same_port_int64 = guard_input_type(&mut same_port_fixture, "GuardInt64", Primitive::Int64);
    let conflicting_same_port = recovery_contract_with_guards(
        &mut same_port_fixture,
        guard(1, PrimitiveConstant::Int64(0)),
        guard(1, PrimitiveConstant::Int64(1)),
        vec![(1, same_port_int64)],
    );
    let relation = analyze_contract(&conflicting_same_port, &same_port_fixture.view())
        .action_guard_compatibility
        .remove(0);
    assert_eq!(relation.verdict(), af_contract::ProofVerdict::Violation);
    assert_eq!(
        relation.countermodel().unwrap().assignments()[0].value(),
        &PrimitiveConstant::Int64(0)
    );

    let mut different_ports_fixture = FixtureContext::new();
    let different_ports_int64 =
        guard_input_type(&mut different_ports_fixture, "GuardInt64", Primitive::Int64);
    let conflicting_ports = recovery_contract_with_guards(
        &mut different_ports_fixture,
        guard(1, PrimitiveConstant::Int64(0)),
        guard(2, PrimitiveConstant::Int64(0)),
        vec![
            (1, different_ports_int64.clone()),
            (2, different_ports_int64),
        ],
    );
    let relation = analyze_contract(&conflicting_ports, &different_ports_fixture.view())
        .action_guard_compatibility
        .remove(0);
    assert_eq!(relation.verdict(), af_contract::ProofVerdict::Violation);
    let values = relation
        .countermodel()
        .unwrap()
        .assignments()
        .iter()
        .map(|assignment| (assignment.port_no(), assignment.value().clone()))
        .collect::<std::collections::BTreeMap<u32, PrimitiveConstant>>();
    assert_eq!(values[&1], PrimitiveConstant::Int64(0));
    assert_eq!(values[&2], PrimitiveConstant::Int64(1));
}

#[test]
fn guard_mismatch_without_a_primitive_countermodel_stays_unproven() {
    let mut fixture = FixtureContext::new();
    let unit = guard_input_type(&mut fixture, "GuardUnit", Primitive::Unit);
    let contract = recovery_contract_with_guards(
        &mut fixture,
        EffectGuard::Unconditional,
        EffectGuard::InputEquals(EffectGuardInputEquals {
            port_no: 1,
            expected: PrimitiveConstant::Unit,
        }),
        vec![(1, unit)],
    );
    let report = analyze_contract(&contract, &fixture.view());
    let relation = report.action_guard_compatibility.first().unwrap();
    assert_eq!(relation.verdict(), af_contract::ProofVerdict::Unproven);
    assert!(relation.countermodel().is_none());
    assert!(check_contract(&contract, &fixture.view()).is_err());
}

#[test]
fn independent_oracle_freezes_all_four_additive_schema_modules() {
    let set = modules();
    let golden: serde_json::Value = serde_json::from_str(include_str!(
        "../../../spec/conformance/component-contract/schema-golden.json"
    ))
    .unwrap();
    for case in golden["cases"].as_array().unwrap() {
        let module = set
            .module(case["specification_id"].as_str().unwrap())
            .unwrap();
        for ty in case["types"].as_array().unwrap() {
            let schema = module.by_name(ty["name"].as_str().unwrap()).unwrap();
            let bytes = schema.ace1_bytes().unwrap();
            assert_eq!(
                String::from_utf8(bytes.clone()).unwrap(),
                ty["ace1_utf8"].as_str().unwrap()
            );
            assert_eq!(hex(&bytes), ty["ace1_hex"].as_str().unwrap());
            assert_eq!(
                schema.digest().unwrap().hex(),
                ty["digest_hex"].as_str().unwrap()
            );
        }
    }
}

#[test]
fn checked_digest_matches_independent_component_contract_oracle() {
    let fixture = FixtureContext::new();
    let context = fixture.view();
    let contract = empty_contract(false);
    let checked = check_contract(&contract, &context).unwrap();
    let digest = component_contract_digest(&checked, &context).unwrap();
    let golden: serde_json::Value = serde_json::from_str(include_str!(
        "../../../spec/conformance/component-contract/golden/contracts.json"
    ))
    .unwrap();
    let expected = golden["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["case_id"] == "contract/golden/minimal-closed")
        .unwrap();
    let canonical = af_contract::validate_raw_contract(&contract, &context).unwrap();
    assert_eq!(
        canonical.ace1_bytes(&context).unwrap(),
        expected["ace1_utf8"].as_str().unwrap().as_bytes()
    );
    assert_eq!(
        digest.hex(),
        expected["component_contract_digest"].as_str().unwrap()
    );
    assert_eq!(digest.domain().as_str(), "component-contract");
}

#[test]
fn every_contract_golden_is_valid_schema_shaped_ace1_and_hashes_independently() {
    let compiled = modules();
    let shape = compiled
        .shape_of("agentflow.component.contract@1/ComponentContract")
        .unwrap();
    let corpus: serde_json::Value = serde_json::from_str(include_str!(
        "../../../spec/conformance/component-contract/golden/contracts.json"
    ))
    .unwrap();
    for case in corpus["cases"].as_array().unwrap() {
        let bytes = from_hex(case["ace1_hex"].as_str().unwrap());
        let value = af_core::decode(&shape, &bytes).unwrap_or_else(|error| {
            panic!(
                "{} is not a ComponentContract ACE1 value: {error}",
                case["case_id"]
            )
        });
        assert_eq!(af_core::encode(&shape, &value).unwrap(), bytes);
        assert_eq!(
            af_core::DigestRef::compute(af_core::DigestDomain::ComponentContract, &bytes).hex(),
            case["component_contract_digest"].as_str().unwrap(),
            "{}",
            case["case_id"],
        );
    }
}

#[test]
fn unproven_summary_never_creates_checked_digest_capability() {
    let fixture = FixtureContext::new();
    let context = fixture.view();
    let report = check_contract(&empty_contract(true), &context).unwrap_err();
    assert!(!report.is_proven());
    assert!(
        report
            .dimensions
            .iter()
            .any(|dimension| dimension.verdict == af_contract::ProofVerdict::RequiresEvidence)
    );
}

#[test]
fn relation_snapshot_pin_is_typed_and_uses_plain_content_sha256() {
    let fixture = FixtureContext::new();
    let snapshot = af_contract::RelationCatalogSnapshot::new(
        "fixture-r1",
        af_contract::EffectResourceCompatibilityCatalog::default(),
        &fixture.modules,
    )
    .unwrap();
    assert_eq!(
        snapshot.pin().schema,
        "agentflow.effect.resource.compatibility@1"
    );
    assert_eq!(
        snapshot.pin().canonical_content_checksum.algorithm,
        "sha256"
    );
    assert_eq!(
        snapshot.pin().canonical_content_checksum.hex,
        sha256_hex(snapshot.canonical_bytes())
    );
    assert_ne!(
        snapshot.pin().canonical_content_checksum.hex,
        af_core::DigestRef::compute(
            af_core::DigestDomain::ComponentContract,
            snapshot.canonical_bytes()
        )
        .hex()
    );
}

#[test]
fn effect_resource_pair_requires_exact_operation_resource_selector_and_qualifier() {
    let mut fixture = FixtureContext::new();
    let resource_type = fixture.use_fixture_resource();
    let operation = EffectOperationRef {
        catalog: "agentflow.effects@1".into(),
        name: "read".into(),
    };
    let capability = CapabilityCatalog::new("agentflow.effects@1", ["read"]).unwrap();
    fixture.effects.insert(&capability).unwrap();
    fixture.recovery.effects.insert(&capability).unwrap();

    let atom = EffectAtom {
        operation: operation.clone(),
        resource_selector: ResourceSelector {
            requirement_port_no: 1,
            scope_bindings: vec![ScopeBinding {
                key: "tenant".into(),
                value: "acme".into(),
            }],
        },
        qualifiers: vec![EffectQualifier::Idempotency(
            EffectIdempotencyQualifier::Natural,
        )],
    };
    let mut contract = empty_contract(false);
    contract.effects = EffectSignature {
        row: EffectRow::Closed(vec![EffectEntry {
            atom: atom.clone(),
            bounds: OccurrenceBounds::finite(1, 2).unwrap(),
            guard: EffectGuard::Unconditional,
        }]),
    };
    contract.resources = ResourceRequirementRow {
        requirements: vec![ResourceRequirement {
            port_no: 1,
            resource_type: resource_type.clone(),
            required_attributes: vec![ResourceAttributeConstraint {
                key: "tenant".into(),
                value: "acme".into(),
            }],
        }],
    };
    fixture.relations.effect_resource = Some(
        RelationCatalogSnapshot::new(
            "fixture-effects-r1",
            EffectResourceCompatibilityCatalog {
                complete_operations: vec![operation.clone()],
                supported_pairs: vec![SupportedEffectResourcePair {
                    operation: operation.clone(),
                    resource_type: resource_type.clone(),
                    selector_keys: vec![SelectorKeyRule {
                        selector_key: "tenant".into(),
                        resource_attribute_key: "tenant".into(),
                        required: true,
                        values: ScopeValueRule::FiniteExactValues(vec!["acme".into()]),
                    }],
                    allowed_qualifier_profiles: vec![EffectQualifierProfile {
                        qualifiers: atom.qualifiers.clone(),
                    }],
                }],
            },
            &fixture.modules,
        )
        .unwrap(),
    );

    let context = fixture.view();
    assert!(check_contract(&contract, &context).is_ok());

    contract.effects = EffectSignature {
        row: EffectRow::Closed(vec![EffectEntry {
            atom: EffectAtom {
                qualifiers: vec![EffectQualifier::Idempotency(
                    EffectIdempotencyQualifier::None,
                )],
                ..atom
            },
            bounds: OccurrenceBounds::finite(1, 2).unwrap(),
            guard: EffectGuard::Unconditional,
        }]),
    };
    let report = check_contract(&contract, &context).unwrap_err();
    assert_eq!(
        report
            .dimensions
            .iter()
            .find(|item| item.dimension == "resources")
            .unwrap()
            .verdict,
        af_contract::ProofVerdict::Violation
    );
}

#[test]
fn missing_effect_resource_catalog_is_unproven_not_allowed() {
    let mut fixture = FixtureContext::new();
    let resource_type = fixture.use_fixture_resource();
    let operation = EffectOperationRef {
        catalog: "agentflow.effects@1".into(),
        name: "read".into(),
    };
    let capability = CapabilityCatalog::new("agentflow.effects@1", ["read"]).unwrap();
    fixture.effects.insert(&capability).unwrap();
    let atom = EffectAtom {
        operation,
        resource_selector: ResourceSelector {
            requirement_port_no: 1,
            scope_bindings: vec![],
        },
        qualifiers: vec![],
    };
    let mut contract = empty_contract(false);
    contract.effects = EffectSignature {
        row: EffectRow::Closed(vec![EffectEntry {
            atom,
            bounds: OccurrenceBounds::finite(1, 1).unwrap(),
            guard: EffectGuard::Unconditional,
        }]),
    };
    contract.resources = ResourceRequirementRow {
        requirements: vec![ResourceRequirement {
            port_no: 1,
            resource_type,
            required_attributes: vec![],
        }],
    };
    let context = fixture.view();
    let report = check_contract(&contract, &context).unwrap_err();
    assert_eq!(
        report
            .dimensions
            .iter()
            .find(|item| item.dimension == "resources")
            .unwrap()
            .verdict,
        af_contract::ProofVerdict::Unproven
    );
}

#[test]
fn io_pin_and_namespace_negatives_are_rejected_before_digest_issuance() {
    let fixture = FixtureContext::new();
    let type_ref = fixture
        .schemas
        .get("agentflow.semantic.core@1/TypeSchemaRef")
        .unwrap()
        .reference();
    let context = fixture.view();
    let mut duplicate = empty_contract(false);
    duplicate.inputs = vec![
        af_contract::ComponentInputPort {
            port_no: 1,
            type_ref: type_ref.clone(),
        },
        af_contract::ComponentInputPort {
            port_no: 1,
            type_ref: type_ref.clone(),
        },
    ];
    let report = check_contract(&duplicate, &context).unwrap_err();
    assert_eq!(report.overall(), af_contract::ProofVerdict::Violation);

    let mut wrong_digest = empty_contract(false);
    let mut bad_pin = type_ref;
    bad_pin.schema_digest.hex = "ff".repeat(32);
    wrong_digest.outputs = vec![af_contract::ComponentOutputPort {
        port_no: 1,
        type_ref: bad_pin,
    }];
    let report = check_contract(&wrong_digest, &context).unwrap_err();
    assert_eq!(report.overall(), af_contract::ProofVerdict::Violation);
}

#[test]
fn malformed_relation_catalog_rows_are_rejected_not_canonicalized_away() {
    let fixture = FixtureContext::new();
    let operation = EffectOperationRef {
        catalog: "agentflow.effects@1".into(),
        name: "read".into(),
    };
    let invalid = EffectResourceCompatibilityCatalog {
        complete_operations: vec![operation.clone(), operation],
        supported_pairs: vec![],
    };
    assert!(RelationCatalogSnapshot::new("fixture-invalid", invalid, &fixture.modules).is_err());
}

#[test]
fn recovery_meter_product_is_checked_and_must_fit_u_recovery() {
    let mut fixture = FixtureContext::new();
    let contract = recovery_contract_with_usage(&mut fixture, Some(100));
    {
        let context = fixture.view();
        assert!(check_contract(&contract, &context).is_ok());
    }

    let mut too_small = FixtureContext::new();
    let contract = recovery_contract_with_usage(&mut too_small, Some(11));
    let context = too_small.view();
    let report = check_contract(&contract, &context).unwrap_err();
    assert_eq!(
        report
            .dimensions
            .iter()
            .find(|item| item.dimension == "usage")
            .unwrap()
            .verdict,
        af_contract::ProofVerdict::Violation,
        "2 forward occurrences x 1 activation x 2 attempts x 3 units = 12",
    );
}

#[test]
fn missing_recovery_coverage_and_missing_u_coordinate_are_unproven() {
    let mut missing_coverage = FixtureContext::new();
    let mut contract = recovery_contract_with_usage(&mut missing_coverage, Some(100));
    missing_coverage.relations.recovery_usage = Some(
        RelationCatalogSnapshot::new(
            "fixture-recovery-usage-r2",
            RecoveryUsageCoverageCatalog { rules: vec![] },
            &missing_coverage.modules,
        )
        .unwrap(),
    );
    {
        let context = missing_coverage.view();
        let report = check_contract(&contract, &context).unwrap_err();
        assert_eq!(
            report
                .dimensions
                .iter()
                .find(|item| item.dimension == "usage")
                .unwrap()
                .verdict,
            af_contract::ProofVerdict::Unproven
        );
    }

    let mut missing_ceiling = FixtureContext::new();
    contract = recovery_contract_with_usage(&mut missing_ceiling, None);
    let context = missing_ceiling.view();
    let report = check_contract(&contract, &context).unwrap_err();
    assert_eq!(
        report
            .dimensions
            .iter()
            .find(|item| item.dimension == "usage")
            .unwrap()
            .verdict,
        af_contract::ProofVerdict::Unproven
    );
}

#[test]
fn flow_execution_relation_proves_exact_pairs_and_rejects_sandbox_relaxation() {
    let mut fixture = FixtureContext::new();
    let label = af_flow::ConfidentialityLabelRef {
        catalog: "agentflow.labels@1".into(),
        name: "pii".into(),
    };
    let taint = af_flow::TaintRef {
        catalog: "agentflow.labels@1".into(),
        name: "user_input".into(),
    };
    fixture
        .flow
        .labels
        .recognize_confidentiality(&label.catalog, &label.name);
    fixture
        .flow
        .labels
        .recognize_taint(&taint.catalog, &taint.name);
    let sg = af_execution::RegionRef {
        catalog: "agentflow.regions@1".into(),
        name: "sg".into(),
    };
    let us = af_execution::RegionRef {
        catalog: "agentflow.regions@1".into(),
        name: "us".into(),
    };
    fixture.execution.regions.recognize(sg.clone()).unwrap();
    fixture.execution.regions.recognize(us.clone()).unwrap();
    let trust = af_execution::TrustDomainRef {
        catalog: "agentflow.trust@1".into(),
        name: "trusted-worker".into(),
    };
    fixture
        .execution
        .trust_domains
        .recognize(trust.clone())
        .unwrap();
    fixture.relations.flow_execution = Some(
        RelationCatalogSnapshot::new(
            "fixture-fx-r1",
            af_contract::InformationFlowExecutionCatalog {
                closed_regions: vec![sg.clone(), us.clone()],
                closed_trust_domains: vec![trust.clone()],
                confidentiality_rules: vec![ConfidentialityExecutionRule {
                    label: label.clone(),
                    region_pairs: RegionPairPolicy::FiniteExactPairs(vec![RegionPair {
                        execution_region: sg.clone(),
                        residency_region: us.clone(),
                    }]),
                }],
                taint_rules: vec![TaintExecutionRule {
                    taint: taint.clone(),
                    trust_domains: TrustDomainPolicy::ExactAllowlist(vec![trust.clone()]),
                    sandbox: SandboxFloor::Minimum(af_execution::SandboxLevel::Strong),
                }],
                ..af_contract::InformationFlowExecutionCatalog::default()
            },
            &fixture.modules,
        )
        .unwrap(),
    );
    let mut contract = empty_contract(false);
    contract.information_flow.confidentiality_restrictions = vec![label];
    contract.information_flow.summary.taints = vec![taint];
    contract.execution.region_residency = Some(af_execution::RegionResidencyRequirement {
        execution_regions: vec![sg],
        residency_regions: vec![us],
    });
    contract.execution.trust_domain = Some(trust);
    contract.execution.sandbox = Some(af_execution::SandboxRequirement {
        minimum: af_execution::SandboxLevel::Strong,
    });
    let context = fixture.view();
    assert!(check_contract(&contract, &context).is_ok());

    contract.execution.sandbox = Some(af_execution::SandboxRequirement {
        minimum: af_execution::SandboxLevel::Basic,
    });
    let report = check_contract(&contract, &context).unwrap_err();
    assert_eq!(
        report
            .dimensions
            .iter()
            .find(|item| item.dimension == "execution")
            .unwrap()
            .verdict,
        af_contract::ProofVerdict::Violation
    );
}

#[test]
fn generated_relation_catalog_corpora_match_typed_canonical_encodings() {
    let fixture = FixtureContext::new();
    let modules = &fixture.modules;
    let effect_resource = RelationCatalogSnapshot::new(
        "fixture-r1",
        EffectResourceCompatibilityCatalog::default(),
        modules,
    )
    .unwrap();
    let recovery_usage = RelationCatalogSnapshot::new(
        "fixture-r1",
        af_contract::RecoveryUsageCoverageCatalog::default(),
        modules,
    )
    .unwrap();
    let flow_execution = RelationCatalogSnapshot::new(
        "fixture-r1",
        af_contract::InformationFlowExecutionCatalog::default(),
        modules,
    )
    .unwrap();
    let cases = [
        (
            "effect-resource",
            effect_resource.canonical_bytes().to_vec(),
            effect_resource.pin().canonical_content_checksum.hex.clone(),
        ),
        (
            "recovery-usage",
            recovery_usage.canonical_bytes().to_vec(),
            recovery_usage.pin().canonical_content_checksum.hex.clone(),
        ),
        (
            "flow-execution",
            flow_execution.canonical_bytes().to_vec(),
            flow_execution.pin().canonical_content_checksum.hex.clone(),
        ),
    ];
    for (name, bytes, checksum) in cases {
        let file = repo_root().join(format!(
            "spec/conformance/component-contract/relation-catalogs/{name}/valid/cases.json"
        ));
        let corpus: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();
        assert_eq!(
            bytes,
            corpus["cases"][0]["ace1_utf8"].as_str().unwrap().as_bytes()
        );
        assert_eq!(
            checksum,
            corpus["cases"][0]["content_checksum_sha256"]
                .as_str()
                .unwrap()
        );
    }
}

#[test]
fn nonempty_relation_catalog_corpora_freeze_canonical_ordering() {
    let fixture = FixtureContext::new();
    let modules = &fixture.modules;
    let type_ref = TypeSchemaRef {
        specification_id: "agentflow.semantic.core@1".into(),
        type_name: "TypeSchemaRef".into(),
        schema_digest: DigestRefValue {
            algorithm: "sha256".into(),
            domain: "type-schema".into(),
            hex: "11".repeat(32),
        },
    };
    let resource_type = ResourceTypeRef { schema: type_ref };
    let operation = |name: &str| EffectOperationRef {
        catalog: "agentflow.effects@1".into(),
        name: name.into(),
    };
    let effect_resource = RelationCatalogSnapshot::new(
        "fixture-r2",
        EffectResourceCompatibilityCatalog {
            complete_operations: vec![operation("read"), operation("write")],
            supported_pairs: vec![SupportedEffectResourcePair {
                operation: operation("read"),
                resource_type,
                selector_keys: vec![
                    SelectorKeyRule {
                        selector_key: "zone".into(),
                        resource_attribute_key: "region".into(),
                        required: false,
                        values: ScopeValueRule::FiniteExactValues(vec!["us".into(), "sg".into()]),
                    },
                    SelectorKeyRule {
                        selector_key: "tenant".into(),
                        resource_attribute_key: "tenant".into(),
                        required: true,
                        values: ScopeValueRule::OpaqueNfcExactString,
                    },
                ],
                allowed_qualifier_profiles: vec![EffectQualifierProfile {
                    qualifiers: vec![EffectQualifier::Idempotency(
                        EffectIdempotencyQualifier::None,
                    )],
                }],
            }],
        },
        modules,
    )
    .unwrap();

    let action = RecoveryActionIdentity {
        forward_effect: EffectAtom {
            operation: operation("charge"),
            resource_selector: ResourceSelector {
                requirement_port_no: 1,
                scope_bindings: vec![],
            },
            qualifiers: vec![EffectQualifier::Idempotency(
                EffectIdempotencyQualifier::Natural,
            )],
        },
        action_kind: RecoveryActionKind::Compensation,
        action_effect: EffectAtom {
            operation: operation("reverse"),
            resource_selector: ResourceSelector {
                requirement_port_no: 1,
                scope_bindings: vec![],
            },
            qualifiers: vec![],
        },
    };
    let recovery_usage = RelationCatalogSnapshot::new(
        "fixture-r2",
        RecoveryUsageCoverageCatalog {
            rules: vec![
                RecoveryUsageCoverageRule {
                    action: action.clone(),
                    complete_meters: vec![
                        UsageMeterRef {
                            catalog: "agentflow.meters@1".into(),
                            name: "z_calls".into(),
                        },
                        UsageMeterRef {
                            catalog: "agentflow.meters@1".into(),
                            name: "a_calls".into(),
                        },
                    ],
                },
                RecoveryUsageCoverageRule {
                    action: RecoveryActionIdentity {
                        action_kind: RecoveryActionKind::Reconciliation,
                        ..action
                    },
                    complete_meters: vec![],
                },
            ],
        },
        modules,
    )
    .unwrap();

    let region = |name: &str| af_execution::RegionRef {
        catalog: "agentflow.regions@1".into(),
        name: name.into(),
    };
    let flow_execution = RelationCatalogSnapshot::new(
        "fixture-r2",
        af_contract::InformationFlowExecutionCatalog {
            closed_regions: vec![region("us"), region("sg")],
            closed_trust_domains: vec![
                af_execution::TrustDomainRef {
                    catalog: "agentflow.trust@1".into(),
                    name: "worker-b".into(),
                },
                af_execution::TrustDomainRef {
                    catalog: "agentflow.trust@1".into(),
                    name: "worker-a".into(),
                },
            ],
            confidentiality_rules: vec![ConfidentialityExecutionRule {
                label: af_flow::ConfidentialityLabelRef {
                    catalog: "agentflow.labels@1".into(),
                    name: "pii".into(),
                },
                region_pairs: RegionPairPolicy::FiniteExactPairs(vec![
                    RegionPair {
                        execution_region: region("us"),
                        residency_region: region("sg"),
                    },
                    RegionPair {
                        execution_region: region("sg"),
                        residency_region: region("us"),
                    },
                ]),
            }],
            ..af_contract::InformationFlowExecutionCatalog::default()
        },
        modules,
    )
    .unwrap();

    for (name, bytes) in [
        ("effect-resource", effect_resource.canonical_bytes()),
        ("recovery-usage", recovery_usage.canonical_bytes()),
        ("flow-execution", flow_execution.canonical_bytes()),
    ] {
        let path = repo_root().join(format!(
            "spec/conformance/component-contract/relation-catalogs/{name}/valid/cases.json"
        ));
        let corpus: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(
            bytes,
            corpus["cases"][1]["ace1_utf8"].as_str().unwrap().as_bytes(),
            "{name}"
        );
    }
}

#[test]
fn changed_relation_snapshot_cannot_reuse_a_checked_proof_or_digest() {
    let first = FixtureContext::new();
    let first_context = first.view();
    let checked = check_contract(&empty_contract(false), &first_context).unwrap();

    let mut changed = FixtureContext::new();
    changed.relations.effect_resource = Some(
        RelationCatalogSnapshot::new(
            "fixture-r1",
            EffectResourceCompatibilityCatalog {
                complete_operations: vec![EffectOperationRef {
                    catalog: "agentflow.effects@1".into(),
                    name: "read".into(),
                }],
                supported_pairs: vec![],
            },
            &changed.modules,
        )
        .unwrap(),
    );
    let changed_context = changed.view();
    assert!(component_contract_digest(&checked, &changed_context).is_err());
    let same_body = check_contract(&empty_contract(false), &changed_context).unwrap();
    assert_eq!(
        af_contract::check_substitution(&same_body, &checked).overall(),
        af_contract::ProofVerdict::Unproven,
    );
}

#[test]
fn compatibility_proves_exact_io_and_rejects_a_new_required_input() {
    let fixture = FixtureContext::new();
    let type_ref = fixture
        .schemas
        .get("agentflow.semantic.core@1/TypeSchemaRef")
        .unwrap()
        .reference();
    let old = empty_contract(false);
    let mut exact = old.clone();
    let context = fixture.view();
    let checked_old = check_contract(&old, &context).unwrap();
    let checked_exact = check_contract(&exact, &context).unwrap();
    assert_eq!(
        af_contract::check_substitution(&checked_exact, &checked_old).overall(),
        af_contract::ProofVerdict::Proven
    );

    exact.inputs.push(af_contract::ComponentInputPort {
        port_no: 1,
        type_ref,
    });
    let checked_with_new_input = check_contract(&exact, &context).unwrap();
    let report = af_contract::check_substitution(&checked_with_new_input, &checked_old);
    assert_eq!(report.overall(), af_contract::ProofVerdict::Violation);
    assert_eq!(
        report
            .dimensions
            .iter()
            .find(|item| item.dimension == "io")
            .unwrap()
            .verdict,
        af_contract::ProofVerdict::Violation
    );
}

#[test]
fn compatibility_reports_failure_row_expansion_as_a_violation() {
    let fixture = FixtureContext::new();
    let payload_type = fixture
        .schemas
        .get("agentflow.semantic.core@1/TypeSchemaRef")
        .unwrap()
        .reference();
    let old = empty_contract(false);
    let mut expanded = old.clone();
    expanded
        .failures
        .failures
        .push(af_failure::FailureTypeRef { payload_type });
    let context = fixture.view();
    let checked_old = check_contract(&old, &context).unwrap();
    let checked_new = check_contract(&expanded, &context).unwrap();
    let report = af_contract::check_substitution(&checked_new, &checked_old);
    assert_eq!(report.overall(), af_contract::ProofVerdict::Violation);
    assert_eq!(
        report
            .dimensions
            .iter()
            .find(|item| item.dimension == "failure")
            .unwrap()
            .verdict,
        af_contract::ProofVerdict::Violation
    );
}

#[test]
fn full_effect_qualifier_member_is_canonical_digest_material() {
    let mut fixture = FixtureContext::new();
    let operations = CapabilityCatalog::new("agentflow.effects@1", ["read"]).unwrap();
    fixture.effects.insert(&operations).unwrap();
    let context = fixture.view();
    let base_atom = |qualifier| EffectAtom {
        operation: EffectOperationRef {
            catalog: "agentflow.effects@1".into(),
            name: "read".into(),
        },
        resource_selector: ResourceSelector {
            requirement_port_no: 1,
            scope_bindings: vec![
                ScopeBinding {
                    key: "z".into(),
                    value: "last".into(),
                },
                ScopeBinding {
                    key: "a".into(),
                    value: "first".into(),
                },
            ],
        },
        qualifiers: vec![EffectQualifier::Idempotency(qualifier)],
    };
    let make_contract = |qualifier| {
        let mut contract = empty_contract(false);
        contract.effects = EffectSignature {
            row: EffectRow::Closed(vec![EffectEntry {
                atom: base_atom(qualifier),
                bounds: OccurrenceBounds::finite(1, 1).unwrap(),
                guard: EffectGuard::Unconditional,
            }]),
        };
        contract
    };
    let natural = af_contract::validate_raw_contract(
        &make_contract(EffectIdempotencyQualifier::Natural),
        &context,
    )
    .unwrap();
    let none = af_contract::validate_raw_contract(
        &make_contract(EffectIdempotencyQualifier::None),
        &context,
    )
    .unwrap();
    let natural_bytes = natural.ace1_bytes(&context).unwrap();
    let none_bytes = none.ace1_bytes(&context).unwrap();
    assert_ne!(natural_bytes, none_bytes);
    assert!(
        String::from_utf8(natural_bytes)
            .unwrap()
            .contains("natural")
    );
    assert!(String::from_utf8(none_bytes).unwrap().contains("none"));
}

#[test]
fn overall_proof_composition_preserves_every_unproven_dimension() {
    let fixture = FixtureContext::new();
    let context = fixture.view();
    let report = af_contract::analyze_contract(&empty_contract(true), &context);
    assert_eq!(
        report.overall(),
        af_contract::ProofVerdict::RequiresEvidence
    );
    assert!(
        report
            .dimensions
            .iter()
            .any(|item| item.dimension == "information-flow"
                && item.verdict == af_contract::ProofVerdict::RequiresEvidence)
    );
    assert!(
        report
            .dimensions
            .iter()
            .any(|item| item.dimension == "cross-dimension"
                && item.verdict == af_contract::ProofVerdict::RequiresEvidence)
    );
}

fn hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}
fn from_hex(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let pair = std::str::from_utf8(pair).unwrap();
            u8::from_str_radix(pair, 16).unwrap()
        })
        .collect()
}
fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::Digest as _;
    hex::encode(sha2::Sha256::digest(bytes))
}
