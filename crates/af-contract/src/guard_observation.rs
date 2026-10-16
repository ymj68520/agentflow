//! Producer-side local observations, not G01 identities or Checked authority.
//!
//! These engineering traces retain actual operations before legacy reduction.
//! G01's seven registered predicates are bound in af-computation; arithmetic
//! event positions below are NOT predicate cases, coordinates, or origins.
use crate::{
    ActionGuardCompatibility, ComponentContract, ComponentInputPort, RecoveryActionBinding,
    RecoveryActionIdentity, RecoveryUsageBinding, StaticContractProofContext,
};
use af_effect::{EffectEntry, EffectGuardInputEquals, OccurrenceBounds, OccurrenceCeiling};
use af_semantic::{SchemaEntry, StructuralType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObservedResolution<T> {
    NotQueried,
    Absent,
    Present(T),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservedEffectRowKind {
    Closed,
    OpenKnown,
    Unknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservedGuardRole {
    Forward,
    Action,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservedSchemaStep {
    Resolve,
    Expand,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObservedPortResult {
    UnconditionalNotRequired,
    Valid {
        port: ComponentInputPort,
        guard: EffectGuardInputEquals,
        resolved: SchemaEntry,
        expanded: StructuralType,
    },
    MissingPort {
        guard: EffectGuardInputEquals,
        input_declarations: Vec<ComponentInputPort>,
    },
    UnresolvedSchema {
        port: ComponentInputPort,
        guard: EffectGuardInputEquals,
        step: ObservedSchemaStep,
    },
    TypeMismatch {
        port: ComponentInputPort,
        guard: EffectGuardInputEquals,
        resolved: SchemaEntry,
        expanded: StructuralType,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservedGuardBranch {
    MissingEntries,
    MissingUsage,
    UnisolatedForward,
    SameAtomBoundsFailure,
    ActionUpperOverflow,
    MissingPort,
    SchemaUnresolved,
    TypeMismatch,
    ZeroPotential,
    FrozenContainment,
    TypedCountermodel,
    NoCountermodel,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservedMultiplyStep {
    ForwardTimesActivation,
    PartialTimesAttempt,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObservedBoundFailure {
    SameAtomProductOverflow {
        binding_ordinal: usize,
        activation_upper: OccurrenceCeiling,
        attempt_upper: OccurrenceCeiling,
    },
    SameAtomSumOverflow {
        processed_binding_ordinals: Vec<usize>,
        before: OccurrenceCeiling,
        next: OccurrenceCeiling,
    },
    SameAtomDivisorOverflow {
        sum: OccurrenceCeiling,
    },
    SameAtomLowerConflict {
        declared_bounds: OccurrenceBounds,
        sum: OccurrenceCeiling,
        divisor: OccurrenceCeiling,
        effective_upper: OccurrenceCeiling,
    },
    ActionUpperOverflow {
        step: ObservedMultiplyStep,
        left: OccurrenceCeiling,
        right: OccurrenceCeiling,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedPeer {
    pub binding_ordinal: usize,
    pub binding: RecoveryActionBinding,
    pub usage: ObservedResolution<(usize, RecoveryUsageBinding)>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObservedUnisolated {
    ForeignForward(ObservedPeer),
    MissingPeerUsage(ObservedPeer),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservedArithmeticKind {
    PeerProduct,
    PeerSum,
    Divisor,
    ForwardTimesActivation,
    PartialTimesAttempt,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedArithmetic {
    pub kind: ObservedArithmeticKind,
    pub binding_ordinal: Option<usize>,
    pub left: OccurrenceCeiling,
    pub right: OccurrenceCeiling,
    pub result: Option<OccurrenceCeiling>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedBounds {
    pub declared_forward_bounds: OccurrenceBounds,
    pub consulted_peer_bindings: Vec<ObservedPeer>,
    /// Non-identity engineering provenance, compared independently; never
    /// exported as a new registered G01 predicate or a Scope input node.
    pub arithmetic: Vec<ObservedArithmetic>,
    pub sum: ObservedResolution<OccurrenceCeiling>,
    pub divisor: ObservedResolution<OccurrenceCeiling>,
    pub effective_upper: ObservedResolution<OccurrenceCeiling>,
    pub forward_times_activation: ObservedResolution<OccurrenceCeiling>,
    pub partial_times_attempt: ObservedResolution<OccurrenceCeiling>,
    pub action_upper: ObservedResolution<OccurrenceCeiling>,
    pub failure: Option<ObservedBoundFailure>,
    pub unisolated: Option<ObservedUnisolated>,
}
impl ObservedBounds {
    pub(crate) fn new(bounds: &OccurrenceBounds) -> Self {
        Self {
            declared_forward_bounds: bounds.clone(),
            consulted_peer_bindings: vec![],
            arithmetic: vec![],
            sum: ObservedResolution::NotQueried,
            divisor: ObservedResolution::NotQueried,
            effective_upper: ObservedResolution::NotQueried,
            forward_times_activation: ObservedResolution::NotQueried,
            partial_times_attempt: ObservedResolution::NotQueried,
            action_upper: ObservedResolution::NotQueried,
            failure: None,
            unisolated: None,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedGuardRun {
    pub binding_ordinal: usize,
    pub identity: RecoveryActionIdentity,
    pub row_kind: ObservedEffectRowKind,
    pub forward: ObservedResolution<(usize, EffectEntry)>,
    pub action: ObservedResolution<(usize, EffectEntry)>,
    pub usage: ObservedResolution<(usize, RecoveryUsageBinding)>,
    pub bounds: Option<ObservedBounds>,
    pub forward_input: ObservedResolution<ObservedPortResult>,
    pub action_input: ObservedResolution<ObservedPortResult>,
    pub primary_error: Option<ObservedGuardRole>,
    pub containment: ObservedResolution<bool>,
    pub branch: Option<ObservedGuardBranch>,
    pub record: Option<ActionGuardCompatibility>,
}
impl ObservedGuardRun {
    pub(crate) fn new(
        binding_ordinal: usize,
        identity: &RecoveryActionIdentity,
        row_kind: ObservedEffectRowKind,
    ) -> Self {
        Self {
            binding_ordinal,
            identity: identity.clone(),
            row_kind,
            forward: ObservedResolution::NotQueried,
            action: ObservedResolution::NotQueried,
            usage: ObservedResolution::NotQueried,
            bounds: None,
            forward_input: ObservedResolution::NotQueried,
            action_input: ObservedResolution::NotQueried,
            primary_error: None,
            containment: ObservedResolution::NotQueried,
            branch: None,
            record: None,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardObservationFailure {
    ResourceLimit,
    IncompleteFacts,
}
/// Private construction binds fresh actual canonical validation and actual
/// context lifetime. Public trace data alone cannot manufacture this binding.
pub struct CanonicalGuardObservation<'c, 'u> {
    pub(crate) raw: ComponentContract,
    pub(crate) declaration: crate::CanonicalComponentContract,
    pub(crate) bytes: Vec<u8>,
    pub(crate) context: &'c StaticContractProofContext<'u>,
    pub(crate) runs: Vec<ObservedGuardRun>,
}
impl<'c, 'u> CanonicalGuardObservation<'c, 'u> {
    /// The supplied raw declaration from this fresh execution; not a claim
    /// about an unavailable historical validation or originating raw stream.
    pub fn raw_declaration(&self) -> &ComponentContract {
        &self.raw
    }
    /// Actual whole-contract validation is a separate operation from native
    /// diagnostics. It never turns a negative diagnostic into Checked input.
    pub fn check_whole_contract(
        &self,
    ) -> Result<crate::CheckedComponentContract<'c, 'u>, crate::ProofReport> {
        crate::check_contract(&self.raw, self.context)
    }
    pub fn declaration(&self) -> &ComponentContract {
        self.declaration.contract()
    }
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn context(&self) -> &'c StaticContractProofContext<'u> {
        self.context
    }
    pub fn runs(&self) -> &[ObservedGuardRun] {
        &self.runs
    }
    pub fn uses_context(&self, c: &StaticContractProofContext<'_>) -> bool {
        std::ptr::eq(self.context, c)
    }
}
#[derive(Debug)]
pub enum CanonicalGuardObservationError {
    CanonicalValidation(crate::ContractError),
    /// Structural canonical validation succeeded; exact ACE1 materialization
    /// failed. This is NOT an earlier raw/canonical validation rejection.
    CanonicalEncoding(crate::ContractError),
    Capture(GuardObservationFailure),
}
pub fn observe_canonical_guard<'c, 'u>(
    raw: &ComponentContract,
    context: &'c StaticContractProofContext<'u>,
) -> Result<CanonicalGuardObservation<'c, 'u>, CanonicalGuardObservationError> {
    let n = raw.recovery_action_bindings.len();
    // Conservative preflight for the actual same-atom quadratic query path.
    // Resource failure belongs to Capture, never a changed legacy verdict.
    let work = n
        .checked_mul(3)
        .and_then(|v| v.checked_add(32))
        .and_then(|v| n.checked_mul(v));
    if n > 65_536 || work.is_none_or(|v| v > 4_194_304) {
        return Err(CanonicalGuardObservationError::Capture(
            GuardObservationFailure::ResourceLimit,
        ));
    }
    let declaration = crate::canonicalize_contract(raw, context)
        .map_err(CanonicalGuardObservationError::CanonicalValidation)?;
    let bytes = declaration
        .ace1_bytes(context)
        .map_err(CanonicalGuardObservationError::CanonicalEncoding)?;
    if bytes.len() > 134_217_728 {
        return Err(CanonicalGuardObservationError::Capture(
            GuardObservationFailure::ResourceLimit,
        ));
    }
    let mut runs = Vec::new();
    crate::recovery_guard::analyze_observed(declaration.contract(), context, &mut runs);
    if runs.len() != declaration.contract().recovery_action_bindings.len()
        || runs
            .iter()
            .any(|r| r.branch.is_none() || r.record.is_none())
    {
        return Err(CanonicalGuardObservationError::Capture(
            GuardObservationFailure::IncompleteFacts,
        ));
    }
    Ok(CanonicalGuardObservation {
        raw: raw.clone(),
        declaration,
        bytes,
        context,
        runs,
    })
}
