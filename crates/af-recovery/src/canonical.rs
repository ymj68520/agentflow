//! Recovery Contract validation, exact EffectAtom canonicalization and ACE1.

use crate::error::{RecoveryError, kind};
use crate::model::{
    CompensationRequirement, EffectRecoveryClause, IdempotencySemantics, KeyStabilityRequirement,
    ReconciliationRequirement, RecoveryContract, UnknownEffectHandling,
};
use crate::universe::RecoveryOperationUniverse;
use af_core::Value;
use af_effect::{
    EffectAtom, EffectEntry, EffectGuard, EffectIdempotencyQualifier, EffectQualifier, EffectRow,
    EffectSignature, OccurrenceBounds, OccurrenceCeiling,
};
use std::collections::BTreeSet;

pub fn canonicalize_recovery_contract(
    contract: &RecoveryContract,
    universe: &RecoveryOperationUniverse,
) -> Result<RecoveryContract, RecoveryError> {
    let mut seen = BTreeSet::new();
    let mut clauses = Vec::with_capacity(contract.effect_recovery_clauses.len());
    for clause in &contract.effect_recovery_clauses {
        let effect = canonical_effect_atom(&clause.effect, universe)?;
        let key = af_effect::AtomKey::of(&effect);
        if !seen.insert(key) {
            return Err(RecoveryError::new(
                kind::DUPLICATE_EFFECT_RECOVERY_CLAUSE,
                "the same complete EffectAtom identity has multiple recovery clauses",
            ));
        }
        validate_clause(clause, universe)?;
        clauses.push(EffectRecoveryClause {
            effect,
            idempotency: clause.idempotency,
            key_requirement: clause.key_requirement.clone(),
            reconciliation: clause.reconciliation,
            compensation: clause.compensation.clone(),
            unknown_effect_handling: clause.unknown_effect_handling,
        });
    }
    clauses.sort_by_key(|clause| af_effect::AtomKey::of(&clause.effect));
    Ok(RecoveryContract {
        replay_semantics: contract.replay_semantics,
        effect_recovery_clauses: clauses,
    })
}

fn canonical_effect_atom(
    atom: &EffectAtom,
    universe: &RecoveryOperationUniverse,
) -> Result<EffectAtom, RecoveryError> {
    let signature = EffectSignature {
        row: EffectRow::Closed(vec![EffectEntry {
            atom: atom.clone(),
            bounds: OccurrenceBounds::new(1, OccurrenceCeiling::Finite(1)).map_err(|error| {
                RecoveryError::new(kind::RECOVERY_CONTRACT_UNPROVEN, error.to_string())
            })?,
            guard: EffectGuard::Unconditional,
        }]),
    };
    let canonical = signature
        .validate(&universe.effects)
        .map_err(|error| RecoveryError::new(kind::UNKNOWN_RECOVERY_OPERATION, error.to_string()))?;
    let EffectRow::Closed(entries) = canonical.row else {
        unreachable!("the recovery atom wrapper is closed")
    };
    Ok(entries.into_iter().next().expect("one recovery atom").atom)
}

fn validate_clause(
    clause: &EffectRecoveryClause,
    universe: &RecoveryOperationUniverse,
) -> Result<(), RecoveryError> {
    if clause.idempotency == IdempotencySemantics::Keyed && clause.key_requirement.is_none() {
        return Err(RecoveryError::new(
            kind::INVALID_IDEMPOTENCY_CONTRACT,
            "KEYED idempotency requires a KeyStabilityRequirement",
        ));
    }
    if clause.idempotency != IdempotencySemantics::Keyed && clause.key_requirement.is_some() {
        return Err(RecoveryError::new(
            kind::INVALID_IDEMPOTENCY_CONTRACT,
            "a key requirement is only valid for KEYED idempotency",
        ));
    }
    if clause.unknown_effect_handling == UnknownEffectHandling::ReconciliationRequired
        && clause.reconciliation == ReconciliationRequirement::None
    {
        return Err(RecoveryError::new(
            kind::INVALID_IDEMPOTENCY_CONTRACT,
            "unknown-effect reconciliation handling requires reconciliation capability",
        ));
    }
    if let CompensationRequirement::Required(operation) = &clause.compensation {
        if !universe
            .effects
            .resolves(&operation.catalog, &operation.name)
        {
            return Err(RecoveryError::new(
                kind::UNKNOWN_RECOVERY_OPERATION,
                format!(
                    "compensation operation {}/{} is not declared",
                    operation.catalog, operation.name
                ),
            ));
        }
    }
    Ok(())
}

impl RecoveryContract {
    pub fn canonical(&self, universe: &RecoveryOperationUniverse) -> Result<Self, RecoveryError> {
        canonicalize_recovery_contract(self, universe)
    }

    pub fn canonical_bytes(
        &self,
        universe: &RecoveryOperationUniverse,
        set: &af_idl::CompiledModuleSet,
    ) -> Result<Vec<u8>, RecoveryError> {
        let shape = set
            .shape_of("agentflow.recovery.contract@1/RecoveryContract")
            .map_err(|error| {
                RecoveryError::new(kind::RECOVERY_ENCODING_ERROR, error.to_string())
            })?;
        shape.validate().map_err(|error| {
            RecoveryError::new(kind::RECOVERY_ENCODING_ERROR, error.to_string())
        })?;
        af_core::encode(&shape, &self.canonical(universe)?.to_value()?)
            .map_err(|error| RecoveryError::new(kind::RECOVERY_ENCODING_ERROR, error.to_string()))
    }

    pub fn to_value(&self) -> Result<Value, RecoveryError> {
        Ok(Value::Record(vec![
            (
                "replay_semantics".into(),
                Value::Enum(self.replay_semantics.as_str().into()),
            ),
            (
                "effect_recovery_clauses".into(),
                Value::List(
                    self.effect_recovery_clauses
                        .iter()
                        .map(clause_value)
                        .collect(),
                ),
            ),
        ]))
    }
}

fn clause_value(clause: &EffectRecoveryClause) -> Value {
    Value::Record(vec![
        ("effect".into(), effect_atom_value(&clause.effect)),
        (
            "idempotency".into(),
            Value::Enum(clause.idempotency.as_str().into()),
        ),
        (
            "key_requirement".into(),
            Value::Option(
                clause
                    .key_requirement
                    .as_ref()
                    .map(|requirement| Box::new(key_requirement_value(requirement))),
            ),
        ),
        (
            "reconciliation".into(),
            Value::Enum(
                match clause.reconciliation {
                    ReconciliationRequirement::None => "none",
                    ReconciliationRequirement::Required => "required",
                }
                .into(),
            ),
        ),
        (
            "compensation".into(),
            match &clause.compensation {
                CompensationRequirement::None => Value::Variant {
                    case: "none".into(),
                    value: Box::new(Value::Unit),
                },
                CompensationRequirement::Required(operation) => Value::Variant {
                    case: "required".into(),
                    value: Box::new(operation_value(operation)),
                },
            },
        ),
        (
            "unknown_effect_handling".into(),
            Value::Enum(
                match clause.unknown_effect_handling {
                    UnknownEffectHandling::Block => "block",
                    UnknownEffectHandling::ReconciliationRequired => "reconciliation_required",
                }
                .into(),
            ),
        ),
    ])
}

fn key_requirement_value(requirement: &KeyStabilityRequirement) -> Value {
    Value::Record(vec![
        (
            "stable_across_attempts".into(),
            Value::Bool(requirement.stable_across_attempts),
        ),
        (
            "distinct_across_invocations".into(),
            Value::Bool(requirement.distinct_across_invocations),
        ),
        (
            "bound_to_operation".into(),
            Value::Bool(requirement.bound_to_operation),
        ),
        (
            "provider_honors".into(),
            Value::Bool(requirement.provider_honors),
        ),
    ])
}

fn operation_value(operation: &af_effect::EffectOperationRef) -> Value {
    Value::Record(vec![
        ("catalog".into(), Value::String(operation.catalog.clone())),
        ("name".into(), Value::String(operation.name.clone())),
    ])
}

fn effect_atom_value(atom: &EffectAtom) -> Value {
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
                    .map(|qualifier| qualifier_value(*qualifier))
                    .collect(),
            ),
        ),
    ])
}

fn qualifier_value(qualifier: EffectQualifier) -> Value {
    match qualifier {
        EffectQualifier::Idempotency(value) => Value::Variant {
            case: "idempotency".into(),
            value: Box::new(Value::Enum(
                match value {
                    EffectIdempotencyQualifier::Natural => "natural",
                    EffectIdempotencyQualifier::None => "none",
                }
                .into(),
            )),
        },
    }
}
