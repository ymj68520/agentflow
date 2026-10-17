//! ComponentContract canonicalization and ACE1 value construction.

use crate::error::ContractError;
use crate::model::{
    ComponentContract, ComponentInputPort, ComponentOutputPort, EffectFlowPartition,
    RecoveryActionIdentity, RecoveryUsageBinding,
};
use crate::validate::StaticContractProofContext;
use af_core::{Value, encode};
use af_effect::{
    EffectAtom, EffectEntry, EffectGuard, EffectIdempotencyQualifier, EffectQualifier, EffectRow,
    EffectSignature, OccurrenceBounds, OccurrenceCeiling,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalComponentContract {
    pub(crate) contract: ComponentContract,
}

impl CanonicalComponentContract {
    #[must_use]
    pub fn contract(&self) -> &ComponentContract {
        &self.contract
    }

    /// Encodes the canonical semantic body using the exact ComponentContract
    /// shape compiled from the registered AFIDL module set.
    pub fn ace1_bytes(
        &self,
        context: &StaticContractProofContext<'_>,
    ) -> Result<Vec<u8>, ContractError> {
        let shape = context
            .modules
            .shape_of("agentflow.component.contract@1/ComponentContract")
            .map_err(|error| {
                ContractError::new("AF-CONTRACT-0003", "schema_shape", error.to_string())
            })?;
        let value = contract_value(&self.contract, context)?;
        encode(&shape, &value).map_err(|error| {
            ContractError::new("AF-CONTRACT-0003", "ace1_encode", error.to_string())
        })
    }
}

/// Canonicalizes every nested dimension against its explicit universe and
/// applies the ComponentContract-specific ordering rules.
pub fn canonicalize_contract(
    raw: &ComponentContract,
    context: &StaticContractProofContext<'_>,
) -> Result<CanonicalComponentContract, ContractError> {
    canonicalize_observed(
        raw,
        context,
        &mut crate::port_observation::PortTrace::default(),
    )
}

pub(crate) fn canonicalize_observed(
    raw: &ComponentContract,
    context: &StaticContractProofContext<'_>,
    trace: &mut crate::port_observation::PortTrace,
) -> Result<CanonicalComponentContract, ContractError> {
    use crate::port_observation::{PortDirection, PortPrefixStop};
    crate::validate::validate_component_identity(&raw.component_identity).inspect_err(|_| {
        trace.stop = Some(PortPrefixStop::ComponentIdentity);
    })?;
    let inputs = canonical_ports(
        &raw.inputs,
        context.schema_universe,
        PortDirection::Input,
        trace,
    )?;
    let outputs = canonical_ports(
        &raw.outputs,
        context.schema_universe,
        PortDirection::Output,
        trace,
    )?;
    let failures = raw
        .failures
        .validate(context.schema_universe)
        .map_err(|error| ContractError::new("AF-CONTRACT-0005", error.kind(), error.to_string()))?;
    let effects = raw
        .effects
        .validate(context.effects)
        .map_err(|error| ContractError::new("AF-CONTRACT-0006", error.kind(), error.to_string()))?;
    let resources = raw
        .resources
        .validate(context.resources)
        .map_err(|error| ContractError::new("AF-CONTRACT-0007", error.kind(), error.to_string()))?;
    let information_flow = raw
        .information_flow
        .canonical(context.flow)
        .map_err(|error| ContractError::new("AF-CONTRACT-0008", error.kind(), error.to_string()))?;
    let usage = raw
        .usage
        .canonical(context.usage_meters)
        .map_err(|error| ContractError::new("AF-CONTRACT-0009", error.kind(), error.to_string()))?;
    let recovery = raw
        .recovery
        .canonical(context.recovery)
        .map_err(|error| ContractError::new("AF-CONTRACT-0010", error.kind(), error.to_string()))?;
    let execution = raw
        .execution
        .canonical(context.execution)
        .map_err(|error| ContractError::new("AF-CONTRACT-0011", error.kind(), error.to_string()))?;

    let mut partitions = raw.effect_flow_partitions.clone();
    for partition in &mut partitions {
        if !context
            .effects
            .resolves(&partition.operation.catalog, &partition.operation.name)
        {
            return Err(ContractError::new(
                "AF-CONTRACT-0014",
                "unknown_partition_operation",
                "flow partition operation is not in the explicit effect catalog",
            ));
        }
        for atom in &mut partition.allowed_effects {
            *atom = canonical_atom(atom, context)?;
        }
        partition
            .allowed_effects
            .sort_by_key(af_effect::AtomKey::of);
        if partition
            .allowed_effects
            .windows(2)
            .any(|pair| af_effect::AtomKey::of(&pair[0]) == af_effect::AtomKey::of(&pair[1]))
        {
            return Err(ContractError::new(
                "AF-CONTRACT-0014",
                "duplicate_partition_effect",
                "partition repeats a complete EffectAtom identity",
            ));
        }
    }
    partitions.sort_by(|a, b| {
        (&a.operation.catalog, &a.operation.name, a.kind).cmp(&(
            &b.operation.catalog,
            &b.operation.name,
            b.kind,
        ))
    });
    let mut actions = raw.recovery_action_bindings.clone();
    for binding in &mut actions {
        binding.identity.forward_effect =
            canonical_atom(&binding.identity.forward_effect, context)?;
        binding.identity.action_effect = canonical_atom(&binding.identity.action_effect, context)?;
    }
    actions.sort_by(|a, b| action_key(&a.identity).cmp(&action_key(&b.identity)));
    if actions
        .windows(2)
        .any(|pair| action_key(&pair[0].identity) == action_key(&pair[1].identity))
    {
        return Err(ContractError::new(
            "AF-CONTRACT-0015",
            "duplicate_recovery_action",
            "duplicate full recovery action identity",
        ));
    }
    let mut usage_bindings = raw.recovery_usage_bindings.clone();
    for binding in &mut usage_bindings {
        binding.action.forward_effect = canonical_atom(&binding.action.forward_effect, context)?;
        binding.action.action_effect = canonical_atom(&binding.action.action_effect, context)?;
        binding
            .activations_per_forward
            .validate()
            .map_err(|error| {
                ContractError::new(
                    "AF-CONTRACT-0016",
                    "invalid_activation_bound",
                    error.to_string(),
                )
            })?;
        binding
            .attempts_per_activation
            .validate()
            .map_err(|error| {
                ContractError::new(
                    "AF-CONTRACT-0016",
                    "invalid_attempt_bound",
                    error.to_string(),
                )
            })?;
    }
    usage_bindings.sort_by(|a, b| action_key(&a.action).cmp(&action_key(&b.action)));
    if usage_bindings
        .windows(2)
        .any(|pair| action_key(&pair[0].action) == action_key(&pair[1].action))
    {
        return Err(ContractError::new(
            "AF-CONTRACT-0016",
            "duplicate_recovery_usage",
            "more than one usage profile binds the same exact action",
        ));
    }
    for binding in &mut usage_bindings {
        binding.per_attempt_usage = binding
            .per_attempt_usage
            .canonical(context.usage_meters)
            .map_err(|error| {
                ContractError::new("AF-CONTRACT-0010", "recovery_usage", error.to_string())
            })?;
    }

    Ok(CanonicalComponentContract {
        contract: ComponentContract {
            component_identity: raw.component_identity.clone(),
            inputs,
            outputs,
            failures,
            effects,
            resources,
            information_flow,
            usage,
            recovery,
            execution,
            effect_flow_partitions: partitions,
            recovery_action_bindings: actions,
            recovery_usage_bindings: usage_bindings,
        },
    })
}

fn canonical_atom(
    atom: &EffectAtom,
    context: &StaticContractProofContext<'_>,
) -> Result<EffectAtom, ContractError> {
    let signature = EffectSignature {
        row: EffectRow::Closed(vec![EffectEntry {
            atom: atom.clone(),
            bounds: OccurrenceBounds {
                lower: 1,
                upper: OccurrenceCeiling::Finite(1),
            },
            guard: EffectGuard::Unconditional,
        }]),
    };
    let canonical = signature.validate(context.effects).map_err(|error| {
        ContractError::new("AF-CONTRACT-0014", "invalid_effect_atom", error.to_string())
    })?;
    match canonical.row {
        EffectRow::Closed(entries) => entries
            .into_iter()
            .next()
            .map(|entry| entry.atom)
            .ok_or_else(|| {
                ContractError::new(
                    "AF-CONTRACT-0014",
                    "invalid_effect_atom",
                    "canonical atom unexpectedly disappeared",
                )
            }),
        _ => Err(ContractError::new(
            "AF-CONTRACT-0014",
            "invalid_effect_atom",
            "single-atom validation produced a non-closed row",
        )),
    }
}

fn canonical_ports<T>(
    ports: &[T],
    schemas: &af_semantic::TypeSchemaUniverse,
    direction: crate::port_observation::PortDirection,
    trace: &mut crate::port_observation::PortTrace,
) -> Result<Vec<T>, ContractError>
where
    T: PortRef + Clone,
{
    use crate::port_observation::{
        PortCondition, PortDirection, PortEvent, PortOccurrence, PortPrefixStop,
    };
    let mut result: Vec<_> = ports.iter().cloned().enumerate().collect();
    result.sort_by_key(|(_, port)| port.number());
    let mut previous = 0;
    let mut predecessor = None;
    for (sorted_ordinal, (original_ordinal, port)) in result.iter().enumerate() {
        let number = port.number();
        let occurrence = PortOccurrence {
            direction,
            original_ordinal: *original_ordinal,
            sorted_ordinal,
            port_no: number,
        };
        if number == 0 || number == previous {
            trace.events.push(PortEvent {
                occurrence,
                predecessor,
                condition: if number == 0 {
                    PortCondition::Zero
                } else {
                    PortCondition::Duplicate
                },
            });
            trace.stop = Some(PortPrefixStop::PortRejection(occurrence));
            let direction = match direction {
                PortDirection::Input => "input",
                PortDirection::Output => "output",
            };
            return Err(ContractError::new(
                "AF-CONTRACT-0004",
                "invalid_or_duplicate_port",
                format!("{direction} port numbers are positive and unique in their namespace"),
            )
            .at(number.to_string()));
        }
        trace.events.push(PortEvent {
            occurrence,
            predecessor,
            condition: PortCondition::PositiveFresh,
        });
        previous = number;
        predecessor = Some((*original_ordinal, number));
        schemas.resolve(port.type_ref()).map_err(|error| {
            trace.stop = Some(PortPrefixStop::SchemaResolution(occurrence));
            ContractError::new(
                "AF-CONTRACT-0004",
                "unknown_or_wrong_type_pin",
                error.to_string(),
            )
            .at(format!("port {number}"))
        })?;
    }
    Ok(result.into_iter().map(|(_, port)| port).collect())
}

trait PortRef {
    fn number(&self) -> u32;
    fn type_ref(&self) -> &af_semantic::TypeSchemaRef;
}
impl PortRef for ComponentInputPort {
    fn number(&self) -> u32 {
        self.port_no
    }
    fn type_ref(&self) -> &af_semantic::TypeSchemaRef {
        &self.type_ref
    }
}
impl PortRef for ComponentOutputPort {
    fn number(&self) -> u32 {
        self.port_no
    }
    fn type_ref(&self) -> &af_semantic::TypeSchemaRef {
        &self.type_ref
    }
}

fn contract_value(
    contract: &ComponentContract,
    context: &StaticContractProofContext<'_>,
) -> Result<Value, ContractError> {
    let failures = contract
        .failures
        .to_value()
        .map_err(|e| ContractError::new("AF-CONTRACT-0005", "failure_value", e.to_string()))?;
    let effects = contract
        .effects
        .to_value()
        .map_err(|e| ContractError::new("AF-CONTRACT-0006", "effect_value", e.to_string()))?;
    let resources = contract
        .resources
        .to_value()
        .map_err(|e| ContractError::new("AF-CONTRACT-0007", "resource_value", e.to_string()))?;
    let flow = contract
        .information_flow
        .to_value(context.flow)
        .map_err(|e| ContractError::new("AF-CONTRACT-0008", "flow_value", e.to_string()))?;
    let usage = contract
        .usage
        .to_value()
        .map_err(|e| ContractError::new("AF-CONTRACT-0009", "usage_value", e.to_string()))?;
    let recovery = contract
        .recovery
        .to_value()
        .map_err(|e| ContractError::new("AF-CONTRACT-0010", "recovery_value", e.to_string()))?;
    let execution = contract
        .execution
        .to_value(context.execution)
        .map_err(|e| ContractError::new("AF-CONTRACT-0011", "execution_value", e.to_string()))?;
    let recovery_usage = contract
        .recovery_usage_bindings
        .iter()
        .map(recovery_usage_value)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Value::Record(vec![
        (
            "component_identity".into(),
            Value::Record(vec![
                (
                    "catalog".into(),
                    Value::String(contract.component_identity.catalog.clone()),
                ),
                (
                    "name".into(),
                    Value::String(contract.component_identity.name.clone()),
                ),
            ]),
        ),
        (
            "inputs".into(),
            Value::List(contract.inputs.iter().map(input_port_value).collect()),
        ),
        (
            "outputs".into(),
            Value::List(contract.outputs.iter().map(output_port_value).collect()),
        ),
        ("failures".into(), failures),
        ("effects".into(), effects),
        ("resources".into(), resources),
        ("information_flow".into(), flow),
        ("usage".into(), usage),
        ("recovery".into(), recovery),
        ("execution".into(), execution),
        (
            "effect_flow_partitions".into(),
            Value::List(
                contract
                    .effect_flow_partitions
                    .iter()
                    .map(partition_value)
                    .collect(),
            ),
        ),
        (
            "recovery_action_bindings".into(),
            Value::List(
                contract
                    .recovery_action_bindings
                    .iter()
                    .map(|binding| {
                        Value::Record(vec![(
                            "identity".into(),
                            recovery_action_value(&binding.identity),
                        )])
                    })
                    .collect(),
            ),
        ),
        (
            "recovery_usage_bindings".into(),
            Value::List(recovery_usage),
        ),
    ]))
}

fn input_port_value(port: &ComponentInputPort) -> Value {
    Value::Record(vec![
        ("port_no".into(), Value::UInt32(port.port_no)),
        ("type".into(), type_ref_value(&port.type_ref)),
    ])
}
fn output_port_value(port: &ComponentOutputPort) -> Value {
    Value::Record(vec![
        ("port_no".into(), Value::UInt32(port.port_no)),
        ("type".into(), type_ref_value(&port.type_ref)),
    ])
}
fn type_ref_value(reference: &af_semantic::TypeSchemaRef) -> Value {
    Value::Record(vec![
        (
            "specification_id".into(),
            Value::String(reference.specification_id.clone()),
        ),
        (
            "type_name".into(),
            Value::String(reference.type_name.clone()),
        ),
        (
            "schema_digest".into(),
            Value::Record(vec![
                (
                    "algorithm".into(),
                    Value::String(reference.schema_digest.algorithm.clone()),
                ),
                (
                    "domain".into(),
                    Value::String(reference.schema_digest.domain.clone()),
                ),
                (
                    "hex".into(),
                    Value::String(reference.schema_digest.hex.clone()),
                ),
            ]),
        ),
    ])
}

fn partition_value(partition: &EffectFlowPartition) -> Value {
    Value::Record(vec![
        ("operation".into(), operation_value(&partition.operation)),
        ("kind".into(), Value::Enum(partition.kind.as_str().into())),
        (
            "allowed_effects".into(),
            Value::List(
                partition
                    .allowed_effects
                    .iter()
                    .map(effect_atom_value)
                    .collect(),
            ),
        ),
    ])
}
fn operation_value(operation: &af_effect::EffectOperationRef) -> Value {
    Value::Record(vec![
        ("catalog".into(), Value::String(operation.catalog.clone())),
        ("name".into(), Value::String(operation.name.clone())),
    ])
}
pub(crate) fn effect_atom_value(atom: &EffectAtom) -> Value {
    Value::Record(vec![
        ("operation".into(), operation_value(&atom.operation)),
        (
            "resource_selector".into(),
            Value::Record(vec![
                (
                    "requirement_port_no".into(),
                    Value::UInt32(atom.resource_selector.requirement_port_no),
                ),
                (
                    "scope_bindings".into(),
                    Value::List(
                        atom.resource_selector
                            .scope_bindings
                            .iter()
                            .map(|binding| {
                                Value::Record(vec![
                                    ("key".into(), Value::String(binding.key.clone())),
                                    ("value".into(), Value::String(binding.value.clone())),
                                ])
                            })
                            .collect(),
                    ),
                ),
            ]),
        ),
        (
            "qualifiers".into(),
            Value::List(
                atom.qualifiers
                    .iter()
                    .map(|qualifier| {
                        let member = match qualifier {
                            EffectQualifier::Idempotency(EffectIdempotencyQualifier::Natural) => {
                                "natural"
                            }
                            EffectQualifier::Idempotency(EffectIdempotencyQualifier::None) => {
                                "none"
                            }
                        };
                        Value::Variant {
                            case: "idempotency".into(),
                            value: Box::new(Value::Enum(member.into())),
                        }
                    })
                    .collect(),
            ),
        ),
    ])
}
pub(crate) fn recovery_action_value(action: &RecoveryActionIdentity) -> Value {
    Value::Record(vec![
        (
            "forward_effect".into(),
            effect_atom_value(&action.forward_effect),
        ),
        (
            "action_kind".into(),
            Value::Enum(action.action_kind.as_str().into()),
        ),
        (
            "action_effect".into(),
            effect_atom_value(&action.action_effect),
        ),
    ])
}
fn recovery_usage_value(binding: &RecoveryUsageBinding) -> Result<Value, ContractError> {
    let per_attempt_usage = binding.per_attempt_usage.to_value().map_err(|error| {
        ContractError::new(
            "AF-CONTRACT-0016",
            "recovery_usage_value",
            error.to_string(),
        )
    })?;
    Ok(Value::Record(vec![
        ("action".into(), recovery_action_value(&binding.action)),
        (
            "activations_per_forward".into(),
            occurrence_bounds_value(&binding.activations_per_forward),
        ),
        (
            "attempts_per_activation".into(),
            occurrence_bounds_value(&binding.attempts_per_activation),
        ),
        ("per_attempt_usage".into(), per_attempt_usage),
    ]))
}
fn occurrence_bounds_value(bounds: &OccurrenceBounds) -> Value {
    Value::Record(vec![
        ("lower".into(), Value::UInt64(bounds.lower)),
        (
            "upper".into(),
            match bounds.upper {
                af_effect::OccurrenceCeiling::Finite(value) => Value::Variant {
                    case: "finite".into(),
                    value: Box::new(Value::UInt64(value)),
                },
                af_effect::OccurrenceCeiling::Unbounded => Value::Variant {
                    case: "unbounded".into(),
                    value: Box::new(Value::Unit),
                },
            },
        ),
    ])
}

fn action_key(action: &RecoveryActionIdentity) -> (af_effect::AtomKey, u32, af_effect::AtomKey) {
    (
        af_effect::AtomKey::of(&action.forward_effect),
        match action.action_kind {
            crate::model::RecoveryActionKind::Reconciliation => 1,
            crate::model::RecoveryActionKind::Compensation => 2,
        },
        af_effect::AtomKey::of(&action.action_effect),
    )
}
