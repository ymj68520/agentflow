//! Typed, static K-action / epsilon-guard compatibility proofs.

use crate::model::{ComponentContract, ComponentIdentity, RecoveryActionIdentity};
use crate::proof::ProofVerdict;
use crate::validate::StaticContractProofContext;
use af_effect::{
    AtomKey, EffectGuard, EffectSignature, OccurrenceBounds, OccurrenceCeiling, guard_contained,
};
use af_semantic::{PrimitiveConstant, StructuralType, TypeSchemaRef};
use std::collections::BTreeMap;

const RULE_RECOVERY_ACTION_GUARD: &str = "AF-CONTRACT-0022";

/// Why the source K action possibility is within the declared action E guard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardCompatibilityBasis {
    FrozenGuardContainment,
    ZeroPossibleActions,
    TypedCountermodel,
    InsufficientEvidence,
}

/// One exact component-input assignment produced by the checker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardInputAssignment {
    port_no: u32,
    type_ref: TypeSchemaRef,
    value: PrimitiveConstant,
}

impl GuardInputAssignment {
    #[must_use]
    pub fn port_no(&self) -> u32 {
        self.port_no
    }
    #[must_use]
    pub fn type_ref(&self) -> &TypeSchemaRef {
        &self.type_ref
    }
    #[must_use]
    pub fn value(&self) -> &PrimitiveConstant {
        &self.value
    }
}

/// A witness that the Forward/K route is possible while its exact action E
/// declaration is false. Instances are constructed only by the validator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedGuardCountermodel {
    source_contract: ComponentContract,
    source_component: ComponentIdentity,
    recovery_action: RecoveryActionIdentity,
    forward_entry_ordinal: usize,
    action_entry_ordinal: usize,
    forward_guard: EffectGuard,
    declared_action_guard: EffectGuard,
    forward_bounds: OccurrenceBounds,
    activation_upper: OccurrenceCeiling,
    attempt_upper: OccurrenceCeiling,
    action_upper: OccurrenceCeiling,
    assignments: Vec<GuardInputAssignment>,
    reachability: GuardCountermodelReachability,
    related_rule_id: &'static str,
}

/// Reachability basis available to the component-local typed countermodel.
/// Component input types in the current schema language have no refinement
/// predicates, so each representable primitive value is a valid input value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardCountermodelReachability {
    WellTypedComponentInputDomain,
}

impl TypedGuardCountermodel {
    #[must_use]
    pub fn source_contract(&self) -> &ComponentContract {
        &self.source_contract
    }
    #[must_use]
    pub fn source_component(&self) -> &ComponentIdentity {
        &self.source_component
    }
    #[must_use]
    pub fn recovery_action(&self) -> &RecoveryActionIdentity {
        &self.recovery_action
    }
    #[must_use]
    pub fn forward_entry_ordinal(&self) -> usize {
        self.forward_entry_ordinal
    }
    #[must_use]
    pub fn action_entry_ordinal(&self) -> usize {
        self.action_entry_ordinal
    }
    #[must_use]
    pub fn forward_guard(&self) -> &EffectGuard {
        &self.forward_guard
    }
    #[must_use]
    pub fn declared_action_guard(&self) -> &EffectGuard {
        &self.declared_action_guard
    }
    #[must_use]
    pub fn forward_bounds(&self) -> &OccurrenceBounds {
        &self.forward_bounds
    }
    #[must_use]
    pub fn activation_upper(&self) -> OccurrenceCeiling {
        self.activation_upper
    }
    #[must_use]
    pub fn attempt_upper(&self) -> OccurrenceCeiling {
        self.attempt_upper
    }
    #[must_use]
    pub fn action_upper(&self) -> OccurrenceCeiling {
        self.action_upper
    }
    #[must_use]
    pub fn assignments(&self) -> &[GuardInputAssignment] {
        &self.assignments
    }
    #[must_use]
    pub fn reachability(&self) -> GuardCountermodelReachability {
        self.reachability
    }
    #[must_use]
    pub fn related_rule_id(&self) -> &'static str {
        self.related_rule_id
    }
}

/// Guard proof for one exact RecoveryActionBinding in one ComponentContract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionGuardCompatibility {
    source_component: ComponentIdentity,
    recovery_action: RecoveryActionIdentity,
    forward_entry_ordinal: Option<usize>,
    action_entry_ordinal: Option<usize>,
    forward_guard: Option<EffectGuard>,
    declared_action_guard: Option<EffectGuard>,
    action_upper: Option<OccurrenceCeiling>,
    verdict: ProofVerdict,
    basis: GuardCompatibilityBasis,
    countermodel: Option<TypedGuardCountermodel>,
    reason: String,
}

impl ActionGuardCompatibility {
    #[must_use]
    pub fn source_component(&self) -> &ComponentIdentity {
        &self.source_component
    }
    #[must_use]
    pub fn recovery_action(&self) -> &RecoveryActionIdentity {
        &self.recovery_action
    }
    #[must_use]
    pub fn forward_entry_ordinal(&self) -> Option<usize> {
        self.forward_entry_ordinal
    }
    #[must_use]
    pub fn action_entry_ordinal(&self) -> Option<usize> {
        self.action_entry_ordinal
    }
    #[must_use]
    pub fn forward_guard(&self) -> Option<&EffectGuard> {
        self.forward_guard.as_ref()
    }
    #[must_use]
    pub fn declared_action_guard(&self) -> Option<&EffectGuard> {
        self.declared_action_guard.as_ref()
    }
    #[must_use]
    pub fn action_upper(&self) -> Option<OccurrenceCeiling> {
        self.action_upper
    }
    #[must_use]
    pub fn verdict(&self) -> ProofVerdict {
        self.verdict
    }
    #[must_use]
    pub fn basis(&self) -> GuardCompatibilityBasis {
        self.basis
    }
    #[must_use]
    pub fn countermodel(&self) -> Option<&TypedGuardCountermodel> {
        self.countermodel.as_ref()
    }
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// Analyze every checked K action against its exact forward and action E
/// entries. Missing entries remain explicit Unproven records; they are never
/// synthesized as pure or unconditional.
#[must_use]
pub fn analyze_action_guard_compatibility(
    contract: &ComponentContract,
    context: &StaticContractProofContext<'_>,
) -> Vec<ActionGuardCompatibility> {
    let entries = entries(&contract.effects);
    contract
        .recovery_action_bindings
        .iter()
        .map(|binding| {
            let identity = &binding.identity;
            let forward = entries
                .iter()
                .enumerate()
                .find(|(_, entry)| AtomKey::of(&entry.atom) == AtomKey::of(&identity.forward_effect));
            let action = entries
                .iter()
                .enumerate()
                .find(|(_, entry)| AtomKey::of(&entry.atom) == AtomKey::of(&identity.action_effect));

            let (Some((forward_ordinal, forward_entry)), Some((action_ordinal, action_entry))) =
                (forward, action)
            else {
                return ActionGuardCompatibility {
                    source_component: contract.component_identity.clone(),
                    recovery_action: identity.clone(),
                    forward_entry_ordinal: forward.map(|(ordinal, _)| ordinal),
                    action_entry_ordinal: action.map(|(ordinal, _)| ordinal),
                    forward_guard: forward.map(|(_, entry)| entry.guard.clone()),
                    declared_action_guard: action.map(|(_, entry)| entry.guard.clone()),
                    action_upper: None,
                    verdict: ProofVerdict::Unproven,
                    basis: GuardCompatibilityBasis::InsufficientEvidence,
                    countermodel: None,
                    reason: "exact forward or action EffectEntry is unavailable in the declared row".into(),
                };
            };

            let Some(usage) = contract
                .recovery_usage_bindings
                .iter()
                .find(|candidate| candidate.action == *identity)
            else {
                return ActionGuardCompatibility {
                    source_component: contract.component_identity.clone(),
                    recovery_action: identity.clone(),
                    forward_entry_ordinal: Some(forward_ordinal),
                    action_entry_ordinal: Some(action_ordinal),
                    forward_guard: Some(forward_entry.guard.clone()),
                    declared_action_guard: Some(action_entry.guard.clone()),
                    action_upper: None,
                    verdict: ProofVerdict::Unproven,
                    basis: GuardCompatibilityBasis::InsufficientEvidence,
                    countermodel: None,
                    reason: "matching RecoveryUsageBinding is unavailable, so K action possibility is not finite evidence".into(),
                };
            };

            let forward_seed_upper =
                match crate::validate::recovery_forward_upper(
                    contract,
                    identity,
                    &forward_entry.bounds,
                ) {
                    Ok(Some(upper)) => upper,
                    Ok(None) => {
                        return ActionGuardCompatibility {
                            source_component: contract.component_identity.clone(),
                            recovery_action: identity.clone(),
                            forward_entry_ordinal: Some(forward_ordinal),
                            action_entry_ordinal: Some(action_ordinal),
                            forward_guard: Some(forward_entry.guard.clone()),
                            declared_action_guard: Some(action_entry.guard.clone()),
                            action_upper: None,
                            verdict: ProofVerdict::Unproven,
                            basis: GuardCompatibilityBasis::InsufficientEvidence,
                            countermodel: None,
                            reason: "same-atom epsilon contains other action contributions that cannot be isolated as a Forward upper".into(),
                        };
                    }
                    Err(()) => {
                        return ActionGuardCompatibility {
                            source_component: contract.component_identity.clone(),
                            recovery_action: identity.clone(),
                            forward_entry_ordinal: Some(forward_ordinal),
                            action_entry_ordinal: Some(action_ordinal),
                            forward_guard: Some(forward_entry.guard.clone()),
                            declared_action_guard: Some(action_entry.guard.clone()),
                            action_upper: None,
                            verdict: ProofVerdict::Violation,
                            basis: GuardCompatibilityBasis::InsufficientEvidence,
                            countermodel: None,
                            reason: "same-atom Forward/Action bounds conflict with the shared epsilon ceiling or overflow".into(),
                        };
                    }
                };
            let mut forward_for_proof = forward_entry.clone();
            forward_for_proof.bounds.upper = forward_seed_upper;
            let action_upper = forward_seed_upper
                .checked_mul(usage.activations_per_forward.upper)
                .and_then(|bound| bound.checked_mul(usage.attempts_per_activation.upper));
            let Ok(action_upper) = action_upper else {
                return ActionGuardCompatibility {
                    source_component: contract.component_identity.clone(),
                    recovery_action: identity.clone(),
                    forward_entry_ordinal: Some(forward_ordinal),
                    action_entry_ordinal: Some(action_ordinal),
                    forward_guard: Some(forward_entry.guard.clone()),
                    declared_action_guard: Some(action_entry.guard.clone()),
                    action_upper: None,
                    verdict: ProofVerdict::Violation,
                    basis: GuardCompatibilityBasis::InsufficientEvidence,
                    countermodel: None,
                    reason: "checked K action upper multiplication overflowed".into(),
                };
            };
            let base = ActionGuardCompatibility {
                source_component: contract.component_identity.clone(),
                recovery_action: identity.clone(),
                forward_entry_ordinal: Some(forward_ordinal),
                action_entry_ordinal: Some(action_ordinal),
                forward_guard: Some(forward_entry.guard.clone()),
                declared_action_guard: Some(action_entry.guard.clone()),
                action_upper: Some(action_upper),
                verdict: ProofVerdict::Proven,
                basis: GuardCompatibilityBasis::FrozenGuardContainment,
                countermodel: None,
                reason: String::new(),
            };

            for validation in [
                validate_guard_port(contract, context, &forward_entry.guard),
                validate_guard_port(contract, context, &action_entry.guard),
            ] {
                if let Err(error) = validation {
                    let (verdict, reason) = match error {
                        GuardPortValidationError::MissingPort => (
                            ProofVerdict::Violation,
                            "EffectGuard names an input port absent from the exact source ComponentContract",
                        ),
                        GuardPortValidationError::UnresolvedSchema => (
                            ProofVerdict::Unproven,
                            "the exact input TypeSchemaRef or its primitive expansion is unavailable",
                        ),
                        GuardPortValidationError::TypeMismatch => (
                            ProofVerdict::Violation,
                            "the exact input TypeSchemaRef resolves to a type incompatible with the guard constant",
                        ),
                    };
                    return ActionGuardCompatibility {
                        verdict,
                        basis: GuardCompatibilityBasis::InsufficientEvidence,
                        reason: reason.into(),
                        ..base
                    };
                }
            }
            if action_upper.is_zero() {
                return ActionGuardCompatibility {
                    basis: GuardCompatibilityBasis::ZeroPossibleActions,
                    reason: "the checked forward/activation/attempt upper is zero, so no K action possibility requires guard coverage".into(),
                    ..base
                };
            }
            if guard_contained(&forward_entry.guard, &action_entry.guard) {
                return ActionGuardCompatibility {
                    reason: "the exact K Forward guard is contained by the declared action Effect guard under frozen af-effect rules".into(),
                    ..base
                };
            }
            if let Some(countermodel) = build_countermodel(
                contract,
                identity,
                forward_ordinal,
                action_ordinal,
                &forward_for_proof,
                action_entry,
                usage.activations_per_forward.upper,
                usage.attempts_per_activation.upper,
                action_upper,
            ) {
                return ActionGuardCompatibility {
                    verdict: ProofVerdict::Violation,
                    basis: GuardCompatibilityBasis::TypedCountermodel,
                    countermodel: Some(countermodel),
                    reason: "a type-valid assignment makes the K Forward guard true while the exact declared action E guard is false".into(),
                    ..base
                };
            }
            ActionGuardCompatibility {
                verdict: ProofVerdict::Unproven,
                basis: GuardCompatibilityBasis::InsufficientEvidence,
                reason: "frozen guard containment failed and no type-valid countermodel could be constructed; do not infer either compatibility or incompatibility".into(),
                ..base
            }
        })
        .collect()
}

fn entries(signature: &EffectSignature) -> &[af_effect::EffectEntry] {
    match &signature.row {
        af_effect::EffectRow::Closed(entries) => entries,
        af_effect::EffectRow::Open(open) => &open.known,
        af_effect::EffectRow::Unknown => &[],
    }
}

fn validate_guard_port(
    contract: &ComponentContract,
    context: &StaticContractProofContext<'_>,
    guard: &EffectGuard,
) -> Result<(), GuardPortValidationError> {
    let EffectGuard::InputEquals(equals) = guard else {
        return Ok(());
    };
    let Some(port) = contract
        .inputs
        .iter()
        .find(|port| port.port_no == equals.port_no)
    else {
        return Err(GuardPortValidationError::MissingPort);
    };
    let schema = context
        .schema_universe
        .resolve(&port.type_ref)
        .map_err(|_| GuardPortValidationError::UnresolvedSchema)?;
    let structural = context
        .schema_universe
        .expand_named(&schema.identity())
        .map_err(|_| GuardPortValidationError::UnresolvedSchema)?;
    match structural {
        StructuralType::Primitive(primitive) if primitive == equals.expected.semantic_type() => {
            Ok(())
        }
        _ => Err(GuardPortValidationError::TypeMismatch),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GuardPortValidationError {
    MissingPort,
    UnresolvedSchema,
    TypeMismatch,
}

#[allow(clippy::too_many_arguments)]
fn build_countermodel(
    contract: &ComponentContract,
    identity: &RecoveryActionIdentity,
    forward_entry_ordinal: usize,
    action_entry_ordinal: usize,
    forward_entry: &af_effect::EffectEntry,
    action_entry: &af_effect::EffectEntry,
    activation_upper: OccurrenceCeiling,
    attempt_upper: OccurrenceCeiling,
    action_upper: OccurrenceCeiling,
) -> Option<TypedGuardCountermodel> {
    let mut assignments = BTreeMap::<u32, GuardInputAssignment>::new();
    match &forward_entry.guard {
        EffectGuard::Unconditional => {}
        EffectGuard::InputEquals(equals) => {
            let input = contract
                .inputs
                .iter()
                .find(|port| port.port_no == equals.port_no)?;
            assignments.insert(
                equals.port_no,
                GuardInputAssignment {
                    port_no: equals.port_no,
                    type_ref: input.type_ref.clone(),
                    value: equals.expected.clone(),
                },
            );
        }
    }
    if let EffectGuard::InputEquals(equals) = &action_entry.guard {
        let input = contract
            .inputs
            .iter()
            .find(|port| port.port_no == equals.port_no)?;
        let existing = assignments.get(&equals.port_no);
        if existing.is_none_or(|assignment| assignment.value == equals.expected) {
            let alternate = alternate_primitive(&equals.expected)?;
            assignments.insert(
                equals.port_no,
                GuardInputAssignment {
                    port_no: equals.port_no,
                    type_ref: input.type_ref.clone(),
                    value: alternate,
                },
            );
        } else if existing.is_some_and(|assignment| assignment.value == equals.expected) {
            return None;
        }
    }
    let assignments = assignments.into_values().collect::<Vec<_>>();
    if !guard_holds(&forward_entry.guard, &assignments)
        || guard_holds(&action_entry.guard, &assignments)
    {
        return None;
    }
    Some(TypedGuardCountermodel {
        source_contract: contract.clone(),
        source_component: contract.component_identity.clone(),
        recovery_action: identity.clone(),
        forward_entry_ordinal,
        action_entry_ordinal,
        forward_guard: forward_entry.guard.clone(),
        declared_action_guard: action_entry.guard.clone(),
        forward_bounds: forward_entry.bounds.clone(),
        activation_upper,
        attempt_upper,
        action_upper,
        assignments,
        reachability: GuardCountermodelReachability::WellTypedComponentInputDomain,
        related_rule_id: RULE_RECOVERY_ACTION_GUARD,
    })
}

fn guard_holds(guard: &EffectGuard, assignments: &[GuardInputAssignment]) -> bool {
    match guard {
        EffectGuard::Unconditional => true,
        EffectGuard::InputEquals(equals) => assignments
            .iter()
            .find(|assignment| assignment.port_no == equals.port_no)
            .is_some_and(|assignment| assignment.value == equals.expected),
    }
}

fn alternate_primitive(expected: &PrimitiveConstant) -> Option<PrimitiveConstant> {
    Some(match expected {
        PrimitiveConstant::Unit | PrimitiveConstant::Null => return None,
        PrimitiveConstant::Bool(value) => PrimitiveConstant::Bool(!value),
        PrimitiveConstant::Int64(value) => PrimitiveConstant::Int64(if *value < i64::MAX {
            *value + 1
        } else {
            *value - 1
        }),
        PrimitiveConstant::Decimal(value) => {
            let zero = af_core::Decimal::parse("0").ok()?;
            let alternate = af_core::Decimal::parse("1").ok()?;
            PrimitiveConstant::Decimal(if *value == zero { alternate } else { zero })
        }
        PrimitiveConstant::String(value) => PrimitiveConstant::String(if value.is_empty() {
            "x".into()
        } else {
            String::new()
        }),
        PrimitiveConstant::Bytes(value) => PrimitiveConstant::Bytes(if value.is_empty() {
            "AQ".into()
        } else {
            String::new()
        }),
        PrimitiveConstant::Timestamp(value) => {
            let micros = value.as_micros();
            PrimitiveConstant::Timestamp(af_core::Timestamp::new(if micros < i64::MAX {
                micros + 1
            } else {
                micros - 1
            }))
        }
        PrimitiveConstant::Duration(value) => {
            let micros = value.as_micros();
            PrimitiveConstant::Duration(af_core::Duration::new(if micros < i64::MAX {
                micros + 1
            } else {
                micros - 1
            }))
        }
    })
}
