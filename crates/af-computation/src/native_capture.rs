//! Local, non-authoritative T3 native seeds. No Scope, ObligationIdentity,
//! normalized report, or CheckedRootExport constructor exists in this module.
use crate::{
    EditionRegistration, RegisteredAnalysisEdition, current_registered_analysis_edition,
    registered_model_catalogue,
};
use af_contract::guard_observation::{
    CanonicalGuardObservation, ObservedBoundFailure, ObservedGuardBranch, ObservedGuardRole,
    ObservedGuardRun, ObservedPortResult, ObservedResolution,
};
use af_contract::{ProofVerdict, StaticContractProofContext, TypedGuardCountermodel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeContextRole {
    Analysis,
    CanonicalValidation,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeDeclaredStep {
    Field {
        owner: &'static str,
        field: &'static str,
    },
    Entry {
        index: usize,
    },
}
/// Borrowed complete input, not a Scope node/rank or a caller-issued pin.
pub struct NativeSubjectRef<'a, 'c, 'u> {
    source: &'a CanonicalGuardObservation<'c, 'u>,
    compiler: &'a RegisteredAnalysisEdition,
}
impl<'a, 'c, 'u> NativeSubjectRef<'a, 'c, 'u> {
    pub fn declaration(&self) -> &af_contract::ComponentContract {
        self.source.declaration()
    }
    pub fn canonical_bytes(&self) -> &[u8] {
        self.source.canonical_bytes()
    }
    pub fn context(&self) -> &'c StaticContractProofContext<'u> {
        self.source.context()
    }
    pub fn compiler(&self) -> &'a RegisteredAnalysisEdition {
        self.compiler
    }
    pub fn context_roles(&self) -> [[NativeContextRole; 2]; 1] {
        [[
            NativeContextRole::Analysis,
            NativeContextRole::CanonicalValidation,
        ]]
    }
    /// Complete input equality, never digest-only or runtime address identity.
    pub fn same_input(&self, other: &NativeSubjectRef<'_, '_, '_>) -> bool {
        let a = self.context();
        let b = other.context();
        self.compiler == other.compiler
            && self.declaration() == other.declaration()
            && self.canonical_bytes() == other.canonical_bytes()
            && a.modules == b.modules
            && a.schema_universe == b.schema_universe
            && a.effects == b.effects
            && a.resources == b.resources
            && a.flow == b.flow
            && a.usage_meters == b.usage_meters
            && a.recovery == b.recovery
            && a.execution == b.execution
            && a.relations == b.relations
    }
}
pub struct NativeOccurrenceRef<'a> {
    pub action_binding_ordinal: usize,
    pub action_identity: &'a af_contract::RecoveryActionIdentity,
    pub effect_row_kind: af_contract::guard_observation::ObservedEffectRowKind,
    pub forward: &'a ObservedResolution<(usize, af_effect::EffectEntry)>,
    pub action: &'a ObservedResolution<(usize, af_effect::EffectEntry)>,
    pub usage: &'a ObservedResolution<(usize, af_contract::RecoveryUsageBinding)>,
}
impl<'a> From<&'a ObservedGuardRun> for NativeOccurrenceRef<'a> {
    fn from(run: &'a ObservedGuardRun) -> Self {
        Self {
            action_binding_ordinal: run.binding_ordinal,
            action_identity: &run.identity,
            effect_row_kind: run.row_kind,
            forward: &run.forward,
            action: &run.action,
            usage: &run.usage,
        }
    }
}
pub enum NativeSourceRef<'a, 'c, 'u> {
    Subject(NativeSubjectRef<'a, 'c, 'u>),
    Binding {
        subject: NativeSubjectRef<'a, 'c, 'u>,
        occurrence: NativeOccurrenceRef<'a>,
    },
}
pub enum NativeTargetRef<'a, 'c, 'u> {
    Subject(NativeSubjectRef<'a, 'c, 'u>),
    Binding {
        subject: NativeSubjectRef<'a, 'c, 'u>,
        action_binding_ordinal: usize,
        action_identity: &'a af_contract::RecoveryActionIdentity,
    },
}
/// Closed registered basis variants with actual typed operands. These are
/// LOCAL references; no FactKey or Canonical ObligationIdentity is created.
pub enum NativeBasisRef<'a, 'c, 'u> {
    CanonicalSource {
        subject: NativeSubjectRef<'a, 'c, 'u>,
        actual_validation_fact_refs: &'a [LocalNativeRef],
    },
    ContextAssociation {
        partition: [[NativeContextRole; 2]; 1],
    },
    SourceResolution {
        occurrence: NativeOccurrenceRef<'a>,
    },
    Bounds {
        trace: &'a af_contract::guard_observation::ObservedBounds,
    },
    GuardInput {
        role: ObservedGuardRole,
        member_path: NativeLocalMember,
        actual_closed_port_result: &'a ObservedPortResult,
    },
    Containment {
        forward_guard: &'a af_effect::EffectGuard,
        action_guard: &'a af_effect::EffectGuard,
        frozen_result: bool,
    },
    Evaluation {
        native_record: &'a af_contract::ActionGuardCompatibility,
        branch: ObservedGuardBranch,
        causes: Vec<LocalNativeRef>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingScope {
    PendingG02,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeCase {
    CanonicalSource,
    ContextAssociation,
    ActionSources,
    ActionBounds,
    GuardInputs,
    GuardContainment,
    GuardEvaluation,
}
impl NativeCase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CanonicalSource => "canonical_source",
            Self::ContextAssociation => "context_association",
            Self::ActionSources => "action_sources",
            Self::ActionBounds => "action_bounds",
            Self::GuardInputs => "guard_inputs",
            Self::GuardContainment => "guard_containment",
            Self::GuardEvaluation => "guard_evaluation",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalNativeRef(usize);
impl LocalNativeRef {
    pub fn index(self) -> usize {
        self.0
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeLocalOrigin {
    case: NativeCase,
    binding_ordinal: Option<usize>,
    guard_role: Option<ObservedGuardRole>,
}
impl NativeLocalOrigin {
    pub fn case(&self) -> NativeCase {
        self.case
    }
    pub fn binding_ordinal(&self) -> Option<usize> {
        self.binding_ordinal
    }
    pub fn guard_role(&self) -> Option<ObservedGuardRole> {
        self.guard_role
    }
    pub fn declared_path(&self) -> Vec<NativeDeclaredStep> {
        let field = |owner, field| NativeDeclaredStep::Field { owner, field };
        if self.case == NativeCase::ContextAssociation {
            return vec![field("NativeRecoveryGuardSubject", "context")];
        }
        let mut path = vec![field("NativeRecoveryGuardSubject", "declaration")];
        if let Some(index) = self.binding_ordinal {
            path.extend([
                field("CanonicalNativeContractDeclaration", "contract"),
                field("af-contract::ComponentContract", "recovery_action_bindings"),
                NativeDeclaredStep::Entry { index },
            ]);
            if let Some(role) = self.guard_role {
                path.push(field("af-contract::RecoveryActionBinding", "identity"));
                path.push(field(
                    "af-contract::RecoveryActionIdentity",
                    match role {
                        ObservedGuardRole::Forward => "forward_effect",
                        ObservedGuardRole::Action => "action_effect",
                    },
                ));
            }
        }
        path
    }
    pub fn source_ordinal(&self) -> Option<usize> {
        self.binding_ordinal
    }
    pub fn mapping_ordinal(&self) -> Option<usize> {
        None
    }
    /// Exact declared input application;never an audit site ID or call count.
    pub fn declared_fields(&self) -> Vec<&'static str> {
        if self.case == NativeCase::ContextAssociation {
            return vec!["context"];
        }
        if self.binding_ordinal.is_none() {
            return vec!["declaration"];
        }
        let mut p = vec!["declaration", "contract"];
        if self.binding_ordinal.is_some() {
            p.push("recovery_action_bindings");
            if let Some(role) = self.guard_role {
                p.push("identity");
                p.push(match role {
                    ObservedGuardRole::Forward => "forward_effect",
                    ObservedGuardRole::Action => "action_effect",
                });
            }
        }
        p
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeLocalWitness {
    InputCountermodel(Box<TypedGuardCountermodel>),
    MissingPort {
        role: ObservedGuardRole,
        requested_port: u32,
        input_declarations: Vec<af_contract::ComponentInputPort>,
    },
    ConstantTypeMismatch {
        role: ObservedGuardRole,
        port: Box<af_contract::ComponentInputPort>,
        type_ref: af_semantic::TypeSchemaRef,
        expanded_type: Box<af_semantic::StructuralType>,
        constant: af_semantic::PrimitiveConstant,
    },
    BoundFailure(ObservedBoundFailure),
    PropagatedFailure {
        cause: LocalNativeRef,
        witness: Box<NativeLocalWitness>,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativePremise {
    CanonicalSource,
    ContextAssociation,
    ActionSourcesReady,
    ActionBoundsReady,
    GuardInputValid(ObservedGuardRole),
    ContainmentOutcome(bool),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeLocalPrerequisite {
    pub predicate: NativePremise,
    pub support: LocalNativeRef,
    pub established: bool,
    pub mandatory: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeLocalObservation {
    pub owner: &'static str,
    pub record_ordinal: Option<usize>,
    pub field: &'static str,
    /// Pending local fact slot, NOT a normalized FactKey/map-member encoding.
    pub local_fact: Option<LocalNativeRef>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeLocalRuleRef {
    pub specification_id: String,
    pub rule_id: String,
    pub anchor: String,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum NativeLocalMember {
    Subject,
    BindingIdentity {
        ordinal: usize,
    },
    EffectGuard {
        open_known: bool,
        entry_ordinal: usize,
    },
    InputType {
        input_ordinal: usize,
        port_no: u32,
    },
    UsageProfile {
        ordinal: usize,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeLocalCoordinate {
    pub clause: NativeCase,
    pub binding_ordinal: Option<usize>,
    pub guard_role: Option<ObservedGuardRole>,
    pub member: NativeLocalMember,
}
/// All fields private:caller JSON, schema success and trace copies cannot
/// construct or change a trusted seed. References remain LOCAL, not FactKey.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalNativeSeed {
    origin: NativeLocalOrigin,
    verdict: ProofVerdict,
    role: &'static str,
    kind: &'static str,
    rule_id: String,
    clause_family: &'static str,
    clause: &'static str,
    source_variant: &'static str,
    target_variant: &'static str,
    prerequisites: Vec<NativeLocalPrerequisite>,
    witness: Option<NativeLocalWitness>,
    observations: Vec<NativeLocalObservation>,
}
impl LocalNativeSeed {
    pub fn origin(&self) -> &NativeLocalOrigin {
        &self.origin
    }
    pub fn verdict(&self) -> ProofVerdict {
        self.verdict
    }
    pub fn role(&self) -> &str {
        self.role
    }
    pub fn kind(&self) -> &str {
        self.kind
    }
    pub fn rule_id(&self) -> &str {
        &self.rule_id
    }
    pub fn clause(&self) -> (&str, &str) {
        (self.clause_family, self.clause)
    }
    pub fn source_variant(&self) -> &str {
        self.source_variant
    }
    pub fn target_variant(&self) -> &str {
        self.target_variant
    }
    pub fn prerequisites(&self) -> &[NativeLocalPrerequisite] {
        &self.prerequisites
    }
    pub fn witness(&self) -> Option<&NativeLocalWitness> {
        self.witness.as_ref()
    }
    pub fn observations(&self) -> &[NativeLocalObservation] {
        &self.observations
    }
    pub fn stage(&self) -> &'static str {
        "contract_validation"
    }
    pub fn family(&self) -> &'static str {
        "native_recovery_guard"
    }
    pub fn evidence_class(&self) -> &'static str {
        "static_declaration"
    }
    pub fn evaluation(&self) -> &'static str {
        "evaluated"
    }
    pub fn dimension_family(&self) -> &'static str {
        "supporting"
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeCaptureFailure {
    InvalidContextBinding,
    UnsupportedModel,
    IncompleteFacts,
    InvalidFacts,
    ResourceLimit,
}
pub struct NativeLocalCapture<'s, 'c, 'u> {
    source: &'s CanonicalGuardObservation<'c, 'u>,
    compiler: RegisteredAnalysisEdition,
    seeds: Vec<LocalNativeSeed>,
}
impl<'s, 'c, 'u> NativeLocalCapture<'s, 'c, 'u> {
    pub fn source(&self) -> &'s CanonicalGuardObservation<'c, 'u> {
        self.source
    }
    pub fn compiler(&self) -> &RegisteredAnalysisEdition {
        &self.compiler
    }
    pub fn seeds(&self) -> &[LocalNativeSeed] {
        &self.seeds
    }
    pub fn scope(&self) -> PendingScope {
        PendingScope::PendingG02
    }
    pub fn complete_scope(&self) -> bool {
        false
    }
    pub fn can_issue_root_export(&self) -> bool {
        false
    }
    pub fn subject(&self) -> NativeSubjectRef<'_, 'c, 'u> {
        NativeSubjectRef {
            source: self.source,
            compiler: &self.compiler,
        }
    }
    fn owns(&self, seed: &LocalNativeSeed) -> bool {
        self.seeds.iter().any(|own| std::ptr::eq(own, seed))
    }
    pub fn source_for(&self, seed: &LocalNativeSeed) -> Option<NativeSourceRef<'_, 'c, 'u>> {
        if !self.owns(seed) {
            return None;
        }
        Some(if let Some(run) = self.run(seed) {
            NativeSourceRef::Binding {
                subject: self.subject(),
                occurrence: run.into(),
            }
        } else {
            NativeSourceRef::Subject(self.subject())
        })
    }
    pub fn target_for(&self, seed: &LocalNativeSeed) -> Option<NativeTargetRef<'_, 'c, 'u>> {
        if !self.owns(seed) {
            return None;
        }
        Some(if let Some(run) = self.run(seed) {
            NativeTargetRef::Binding {
                subject: self.subject(),
                action_binding_ordinal: run.binding_ordinal,
                action_identity: &run.identity,
            }
        } else {
            NativeTargetRef::Subject(self.subject())
        })
    }
    pub fn basis(&self, seed: &LocalNativeSeed) -> Option<NativeBasisRef<'_, 'c, 'u>> {
        if !self.owns(seed) {
            return None;
        }
        match seed.origin.case {
            NativeCase::CanonicalSource => Some(NativeBasisRef::CanonicalSource {
                subject: self.subject(),
                actual_validation_fact_refs: &[],
            }),
            NativeCase::ContextAssociation => Some(NativeBasisRef::ContextAssociation {
                partition: self.subject().context_roles(),
            }),
            NativeCase::ActionSources => Some(NativeBasisRef::SourceResolution {
                occurrence: self.run(seed)?.into(),
            }),
            NativeCase::ActionBounds => Some(NativeBasisRef::Bounds {
                trace: self.run(seed)?.bounds.as_ref()?,
            }),
            NativeCase::GuardInputs => {
                let run = self.run(seed)?;
                let role = seed.origin.guard_role?;
                let result = match role {
                    ObservedGuardRole::Forward => &run.forward_input,
                    ObservedGuardRole::Action => &run.action_input,
                };
                let ObservedResolution::Present(actual_closed_port_result) = result else {
                    return None;
                };
                Some(NativeBasisRef::GuardInput {
                    role,
                    member_path: self.coordinate(seed)?.member,
                    actual_closed_port_result,
                })
            }
            NativeCase::GuardContainment => {
                let run = self.run(seed)?;
                let ObservedResolution::Present(frozen_result) = run.containment else {
                    return None;
                };
                let ObservedResolution::Present((_, forward)) = &run.forward else {
                    return None;
                };
                let ObservedResolution::Present((_, action)) = &run.action else {
                    return None;
                };
                Some(NativeBasisRef::Containment {
                    forward_guard: &forward.guard,
                    action_guard: &action.guard,
                    frozen_result,
                })
            }
            NativeCase::GuardEvaluation => {
                let run = self.run(seed)?;
                Some(NativeBasisRef::Evaluation {
                    native_record: run.record.as_ref()?,
                    branch: run.branch?,
                    causes: self.cause_refs(seed)?,
                })
            }
        }
    }
    pub fn cause_refs(&self, seed: &LocalNativeSeed) -> Option<Vec<LocalNativeRef>> {
        if !self.owns(seed) {
            return None;
        }
        if seed.origin.case != NativeCase::GuardEvaluation {
            return Some(vec![]);
        }
        let run = self.run(seed)?;
        let cause_case = match run.branch? {
            ObservedGuardBranch::MissingEntries | ObservedGuardBranch::MissingUsage => {
                Some((NativeCase::ActionSources, None))
            }
            ObservedGuardBranch::UnisolatedForward
            | ObservedGuardBranch::SameAtomBoundsFailure
            | ObservedGuardBranch::ActionUpperOverflow => Some((NativeCase::ActionBounds, None)),
            ObservedGuardBranch::MissingPort
            | ObservedGuardBranch::SchemaUnresolved
            | ObservedGuardBranch::TypeMismatch => {
                Some((NativeCase::GuardInputs, run.primary_error))
            }
            ObservedGuardBranch::TypedCountermodel | ObservedGuardBranch::NoCountermodel => {
                Some((NativeCase::GuardContainment, None))
            }
            ObservedGuardBranch::ZeroPotential | ObservedGuardBranch::FrozenContainment => None,
        };
        let Some((case, role)) = cause_case else {
            return Some(vec![]);
        };
        let index = self.seeds.iter().position(|s| {
            s.origin.case == case
                && s.origin.binding_ordinal == seed.origin.binding_ordinal
                && s.origin.guard_role == role
        })?;
        Some(vec![LocalNativeRef(index)])
    }
    pub fn run(&self, seed: &LocalNativeSeed) -> Option<&ObservedGuardRun> {
        if !self.seeds.iter().any(|own| std::ptr::eq(own, seed)) {
            return None;
        }
        seed.origin
            .binding_ordinal
            .and_then(|n| self.source.runs().get(n))
    }
    pub fn rule_ref(&self, seed: &LocalNativeSeed) -> Option<NativeLocalRuleRef> {
        if !self.seeds.iter().any(|own| std::ptr::eq(own, seed)) {
            return None;
        }
        let row = self
            .compiler
            .compiler
            .rules
            .iter()
            .find(|r| r.rule_id == seed.rule_id)?;
        let owner = self
            .compiler
            .registered_sources
            .iter()
            .find(|s| s.logical_path == row.normative_source)?;
        Some(NativeLocalRuleRef {
            specification_id: owner.specification_id.clone(),
            rule_id: row.rule_id.clone(),
            anchor: row.source_anchor.clone(),
        })
    }
    pub fn coordinate(&self, seed: &LocalNativeSeed) -> Option<NativeLocalCoordinate> {
        if !self.seeds.iter().any(|own| std::ptr::eq(own, seed)) {
            return None;
        }
        let member = if let Some(run) = self.run(seed) {
            if let Some(role) = seed.origin.guard_role {
                let res = match role {
                    ObservedGuardRole::Forward => &run.forward,
                    ObservedGuardRole::Action => &run.action,
                };
                let ObservedResolution::Present((ordinal, _)) = res else {
                    return None;
                };
                NativeLocalMember::EffectGuard {
                    open_known: matches!(
                        run.row_kind,
                        af_contract::guard_observation::ObservedEffectRowKind::OpenKnown
                    ),
                    entry_ordinal: *ordinal,
                }
            } else {
                NativeLocalMember::BindingIdentity {
                    ordinal: run.binding_ordinal,
                }
            }
        } else {
            NativeLocalMember::Subject
        };
        Some(NativeLocalCoordinate {
            clause: seed.origin.case,
            binding_ordinal: seed.origin.binding_ordinal,
            guard_role: seed.origin.guard_role,
            member,
        })
    }
    /// Full typed native member references, not RuleIds or parsed sentences.
    /// They remain bound to self.source; no fabricated Scope/DAG rank exists.
    pub fn references(&self, seed: &LocalNativeSeed) -> Option<Vec<NativeLocalMember>> {
        let mut refs = vec![self.coordinate(seed)?.member];
        if let Some(run) = self.run(seed) {
            for res in [&run.forward, &run.action] {
                if let ObservedResolution::Present((i, _)) = res {
                    refs.push(NativeLocalMember::EffectGuard {
                        open_known: matches!(
                            run.row_kind,
                            af_contract::guard_observation::ObservedEffectRowKind::OpenKnown
                        ),
                        entry_ordinal: *i,
                    });
                }
            }
            if let ObservedResolution::Present((i, _)) = &run.usage {
                refs.push(NativeLocalMember::UsageProfile { ordinal: *i });
            }
            if let Some(role) = seed.origin.guard_role {
                let result = match role {
                    ObservedGuardRole::Forward => &run.forward_input,
                    ObservedGuardRole::Action => &run.action_input,
                };
                let port = match result {
                    ObservedResolution::Present(
                        ObservedPortResult::Valid { port, .. }
                        | ObservedPortResult::UnresolvedSchema { port, .. }
                        | ObservedPortResult::TypeMismatch { port, .. },
                    ) => Some(port),
                    _ => None,
                };
                if let Some(port) = port {
                    let i = self
                        .source
                        .declaration()
                        .inputs
                        .iter()
                        .position(|p| p == port)?;
                    refs.push(NativeLocalMember::InputType {
                        input_ordinal: i,
                        port_no: port.port_no,
                    });
                }
            }
        }
        refs.sort();
        refs.dedup();
        Some(refs)
    }
    pub fn primary(&self, seed: &LocalNativeSeed) -> Option<NativeLocalMember> {
        self.references(seed)?.into_iter().next()
    }
    pub fn consumers(&self, support: LocalNativeRef) -> Vec<LocalNativeRef> {
        self.seeds
            .iter()
            .enumerate()
            .filter(|(_, s)| s.prerequisites.iter().any(|p| p.support == support))
            .map(|(n, _)| LocalNativeRef(n))
            .collect()
    }
    fn required_premises(&self, seed: &LocalNativeSeed) -> Option<Vec<NativePremise>> {
        use NativePremise as P;
        match seed.origin.case {
            NativeCase::CanonicalSource => Some(vec![]),
            NativeCase::ContextAssociation => Some(vec![P::CanonicalSource]),
            NativeCase::ActionSources => Some(vec![P::CanonicalSource, P::ContextAssociation]),
            NativeCase::ActionBounds => Some(vec![P::ActionSourcesReady]),
            NativeCase::GuardInputs => Some(vec![P::ActionBoundsReady]),
            NativeCase::GuardContainment | NativeCase::GuardEvaluation => {
                let run = self.run(seed)?;
                let mut out = vec![
                    P::CanonicalSource,
                    P::ContextAssociation,
                    P::ActionSourcesReady,
                ];
                if run.bounds.is_some() {
                    out.push(P::ActionBoundsReady);
                }
                for (role, result) in [
                    (ObservedGuardRole::Forward, &run.forward_input),
                    (ObservedGuardRole::Action, &run.action_input),
                ] {
                    if matches!(result, ObservedResolution::Present(_))
                        && !(role == ObservedGuardRole::Action
                            && run.primary_error == Some(ObservedGuardRole::Forward))
                    {
                        out.push(P::GuardInputValid(role));
                    }
                }
                if seed.origin.case == NativeCase::GuardEvaluation {
                    if let ObservedResolution::Present(value) = run.containment {
                        out.push(P::ContainmentOutcome(value));
                    }
                }
                Some(out)
            }
        }
    }
    /// Enforce the closed local dependency graph before exposing any seeds.
    /// This does not normalize keys or assert complete portable Scope.
    pub fn validate(&self) -> Result<(), NativeCaptureFailure> {
        let model =
            registered_model_catalogue().map_err(|_| NativeCaptureFailure::UnsupportedModel)?;
        let mut applications = Vec::new();
        for (index, seed) in self.seeds.iter().enumerate() {
            if applications.contains(&seed.origin) {
                return Err(NativeCaptureFailure::InvalidFacts);
            }
            applications.push(seed.origin.clone());
            let row = model["native_case_descriptors"]
                .as_array()
                .and_then(|rows| rows.iter().find(|r| r["case"] == seed.origin.case.as_str()))
                .ok_or(NativeCaptureFailure::UnsupportedModel)?;
            let expected_clause = if seed.origin.case == NativeCase::ContextAssociation {
                "proof_context"
            } else {
                seed.origin.case.as_str()
            };
            if row["role"] != seed.role
                || row["kind"] != seed.kind
                || row["stage"] != seed.stage()
                || row["governing_rule"] != seed.rule_id
                || seed.clause != expected_clause
                || seed.clause_family
                    != if seed.origin.case == NativeCase::ContextAssociation {
                        "direct"
                    } else {
                        "native_guard"
                    }
            {
                return Err(NativeCaptureFailure::InvalidFacts);
            }
            let subject = matches!(
                seed.origin.case,
                NativeCase::CanonicalSource | NativeCase::ContextAssociation
            );
            let variant = if subject {
                "native_guard_subject"
            } else {
                "native_guard_binding"
            };
            if seed.source_variant != variant
                || seed.target_variant != variant
                || subject != seed.origin.binding_ordinal.is_none()
                || (seed.origin.case == NativeCase::GuardInputs) != seed.origin.guard_role.is_some()
                || self.rule_ref(seed).is_none()
                || self.basis(seed).is_none()
                || self.coordinate(seed).is_none()
            {
                return Err(NativeCaptureFailure::InvalidFacts);
            }
            let expected = match seed.origin.case {
                NativeCase::CanonicalSource | NativeCase::ContextAssociation => {
                    ProofVerdict::Proven
                }
                NativeCase::ActionSources => {
                    let run = self.run(seed).ok_or(NativeCaptureFailure::InvalidFacts)?;
                    if matches!(run.forward, ObservedResolution::Present(_))
                        && matches!(run.action, ObservedResolution::Present(_))
                        && matches!(run.usage, ObservedResolution::Present(_))
                    {
                        ProofVerdict::Proven
                    } else {
                        ProofVerdict::Unproven
                    }
                }
                NativeCase::ActionBounds => {
                    let bounds = self
                        .run(seed)
                        .and_then(|r| r.bounds.as_ref())
                        .ok_or(NativeCaptureFailure::InvalidFacts)?;
                    if bounds.failure.is_some() {
                        ProofVerdict::Violation
                    } else if bounds.unisolated.is_some() {
                        ProofVerdict::Unproven
                    } else {
                        ProofVerdict::Proven
                    }
                }
                NativeCase::GuardInputs => {
                    let NativeBasisRef::GuardInput {
                        role,
                        actual_closed_port_result,
                        ..
                    } = self.basis(seed).ok_or(NativeCaptureFailure::InvalidFacts)?
                    else {
                        return Err(NativeCaptureFailure::InvalidFacts);
                    };
                    let (status, witness) = input_result(actual_closed_port_result, role);
                    if witness != seed.witness {
                        return Err(NativeCaptureFailure::InvalidFacts);
                    }
                    status
                }
                NativeCase::GuardContainment => {
                    let ObservedResolution::Present(value) = self
                        .run(seed)
                        .ok_or(NativeCaptureFailure::InvalidFacts)?
                        .containment
                    else {
                        return Err(NativeCaptureFailure::InvalidFacts);
                    };
                    if value {
                        ProofVerdict::Proven
                    } else {
                        ProofVerdict::Unproven
                    }
                }
                NativeCase::GuardEvaluation => self
                    .run(seed)
                    .and_then(|r| r.record.as_ref())
                    .ok_or(NativeCaptureFailure::InvalidFacts)?
                    .verdict(),
            };
            if seed.verdict != expected
                || (seed.verdict == ProofVerdict::Violation) != seed.witness.is_some()
            {
                return Err(NativeCaptureFailure::InvalidFacts);
            }
            if let Some(w) = &seed.witness {
                match w {
                    NativeLocalWitness::InputCountermodel(actual) => {
                        let expected = self
                            .run(seed)
                            .and_then(|r| r.record.as_ref())
                            .and_then(af_contract::ActionGuardCompatibility::countermodel)
                            .ok_or(NativeCaptureFailure::InvalidFacts)?;
                        if actual.as_ref() != expected
                            || seed.origin.case != NativeCase::GuardEvaluation
                        {
                            return Err(NativeCaptureFailure::InvalidFacts);
                        }
                    }
                    NativeLocalWitness::PropagatedFailure { cause, witness } => {
                        if cause.0 >= index
                            || matches!(
                                witness.as_ref(),
                                NativeLocalWitness::PropagatedFailure { .. }
                            )
                            || self.seeds[cause.0].witness.as_ref() != Some(witness.as_ref())
                            || !self
                                .cause_refs(seed)
                                .ok_or(NativeCaptureFailure::InvalidFacts)?
                                .contains(cause)
                        {
                            return Err(NativeCaptureFailure::InvalidFacts);
                        }
                    }
                    NativeLocalWitness::BoundFailure(failure) => {
                        if self
                            .run(seed)
                            .and_then(|r| r.bounds.as_ref())
                            .and_then(|b| b.failure.as_ref())
                            != Some(failure)
                        {
                            return Err(NativeCaptureFailure::InvalidFacts);
                        }
                    }
                    NativeLocalWitness::MissingPort { .. }
                    | NativeLocalWitness::ConstantTypeMismatch { .. } => {
                        if seed.origin.case != NativeCase::GuardInputs {
                            return Err(NativeCaptureFailure::InvalidFacts);
                        }
                    }
                }
            }
            let required = self
                .required_premises(seed)
                .ok_or(NativeCaptureFailure::InvalidFacts)?;
            if required.len() != seed.prerequisites.len()
                || required.iter().any(|want| {
                    seed.prerequisites
                        .iter()
                        .filter(|p| &p.predicate == want)
                        .count()
                        != 1
                })
            {
                return Err(NativeCaptureFailure::IncompleteFacts);
            }
            for p in &seed.prerequisites {
                if p.support.0 >= index || !p.mandatory {
                    return Err(NativeCaptureFailure::InvalidFacts);
                }
                let support = &self.seeds[p.support.0];
                let expected_case = match p.predicate {
                    NativePremise::CanonicalSource => NativeCase::CanonicalSource,
                    NativePremise::ContextAssociation => NativeCase::ContextAssociation,
                    NativePremise::ActionSourcesReady => NativeCase::ActionSources,
                    NativePremise::ActionBoundsReady => NativeCase::ActionBounds,
                    NativePremise::GuardInputValid(_) => NativeCase::GuardInputs,
                    NativePremise::ContainmentOutcome(_) => NativeCase::GuardContainment,
                };
                if support.origin.case != expected_case {
                    return Err(NativeCaptureFailure::InvalidFacts);
                }
                if !matches!(
                    expected_case,
                    NativeCase::CanonicalSource | NativeCase::ContextAssociation
                ) && support.origin.binding_ordinal != seed.origin.binding_ordinal
                {
                    return Err(NativeCaptureFailure::InvalidFacts);
                }
                let established = match p.predicate {
                    NativePremise::ContainmentOutcome(expected) => self
                        .run(support)
                        .is_some_and(|r| r.containment == ObservedResolution::Present(expected)),
                    NativePremise::GuardInputValid(role) => {
                        if support.origin.guard_role != Some(role) {
                            return Err(NativeCaptureFailure::InvalidFacts);
                        }
                        support.verdict == ProofVerdict::Proven
                    }
                    _ => support.verdict == ProofVerdict::Proven,
                };
                if established != p.established
                    || (seed.verdict == ProofVerdict::Proven && !established)
                {
                    return Err(NativeCaptureFailure::InvalidFacts);
                }
            }
            if seed.observations.is_empty() {
                return Err(NativeCaptureFailure::IncompleteFacts);
            }
            for observation in &seed.observations {
                let is_record = seed.origin.case == NativeCase::GuardEvaluation;
                if observation.owner != "native_recovery_guard"
                    || observation.field != if is_record { "native_records" } else { "facts" }
                    || observation.record_ordinal
                        != if is_record {
                            seed.origin.binding_ordinal
                        } else {
                            None
                        }
                    || observation.local_fact
                        != if is_record {
                            None
                        } else {
                            Some(LocalNativeRef(index))
                        }
                {
                    return Err(NativeCaptureFailure::InvalidFacts);
                }
            }
        }
        // Derived expected instance roster; mutually missing formal cases do
        // not become success just because both compared outputs omitted them.
        let expected = 2 + self
            .source
            .runs()
            .iter()
            .map(|r| {
                2 + usize::from(r.bounds.is_some())
                    + usize::from(matches!(r.forward_input, ObservedResolution::Present(_)))
                    + usize::from(matches!(r.action_input, ObservedResolution::Present(_)))
                    + usize::from(matches!(r.containment, ObservedResolution::Present(_)))
            })
            .sum::<usize>();
        if self.seeds.len() != expected {
            return Err(NativeCaptureFailure::IncompleteFacts);
        }
        Ok(())
    }
}
fn emit(
    seeds: &mut Vec<LocalNativeSeed>,
    case: NativeCase,
    binding: Option<usize>,
    guard_role: Option<ObservedGuardRole>,
    verdict: ProofVerdict,
    prerequisites: Vec<NativeLocalPrerequisite>,
    witness: Option<NativeLocalWitness>,
) -> Result<LocalNativeRef, NativeCaptureFailure> {
    let model = registered_model_catalogue().map_err(|_| NativeCaptureFailure::UnsupportedModel)?;
    let row = model["native_case_descriptors"]
        .as_array()
        .and_then(|a| a.iter().find(|r| r["case"] == case.as_str()))
        .ok_or(NativeCaptureFailure::UnsupportedModel)?;
    let subject = binding.is_none();
    if subject
        != matches!(
            case,
            NativeCase::CanonicalSource | NativeCase::ContextAssociation
        )
        || (case == NativeCase::GuardInputs) != guard_role.is_some()
    {
        return Err(NativeCaptureFailure::InvalidFacts);
    }
    let context = case == NativeCase::ContextAssociation;
    let expected_role = if subject {
        "mandatory_preflight"
    } else {
        "supporting_fact"
    };
    let expected_kind = if context {
        "proof_context"
    } else {
        "native_recovery_guard_evidence"
    };
    if row["role"] != expected_role
        || row["kind"] != expected_kind
        || row["stage"] != "contract_validation"
    {
        return Err(NativeCaptureFailure::UnsupportedModel);
    }
    if verdict == ProofVerdict::Violation && witness.is_none() {
        return Err(NativeCaptureFailure::IncompleteFacts);
    }
    if verdict == ProofVerdict::Proven
        && prerequisites.iter().any(|p| p.mandatory && !p.established)
    {
        return Err(NativeCaptureFailure::InvalidFacts);
    }
    if prerequisites.iter().any(|p| p.support.0 >= seeds.len()) {
        return Err(NativeCaptureFailure::InvalidFacts);
    }
    let origin = NativeLocalOrigin {
        case,
        binding_ordinal: binding,
        guard_role,
    };
    if seeds.iter().any(|s| s.origin == origin) {
        return Err(NativeCaptureFailure::InvalidFacts);
    }
    if seeds.len() >= 65_536 {
        return Err(NativeCaptureFailure::ResourceLimit);
    }
    let key = LocalNativeRef(seeds.len());
    seeds.push(LocalNativeSeed {
        origin,
        verdict,
        role: expected_role,
        kind: expected_kind,
        rule_id: row["governing_rule"]
            .as_str()
            .ok_or(NativeCaptureFailure::UnsupportedModel)?
            .into(),
        clause_family: if context { "direct" } else { "native_guard" },
        clause: if context {
            "proof_context"
        } else {
            case.as_str()
        },
        source_variant: if subject {
            "native_guard_subject"
        } else {
            "native_guard_binding"
        },
        target_variant: if subject {
            "native_guard_subject"
        } else {
            "native_guard_binding"
        },
        prerequisites,
        witness,
        observations: vec![NativeLocalObservation {
            owner: "native_recovery_guard",
            record_ordinal: if case == NativeCase::GuardEvaluation {
                binding
            } else {
                None
            },
            local_fact: if case == NativeCase::GuardEvaluation {
                None
            } else {
                Some(key)
            },
            field: match case {
                NativeCase::GuardEvaluation => "native_records",
                _ => "facts",
            },
        }],
    });
    Ok(key)
}
fn premise(
    predicate: NativePremise,
    support: LocalNativeRef,
    established: bool,
) -> NativeLocalPrerequisite {
    NativeLocalPrerequisite {
        predicate,
        support,
        established,
        mandatory: true,
    }
}
fn input_result(
    p: &ObservedPortResult,
    role: ObservedGuardRole,
) -> (ProofVerdict, Option<NativeLocalWitness>) {
    match p {
        ObservedPortResult::UnconditionalNotRequired | ObservedPortResult::Valid { .. } => {
            (ProofVerdict::Proven, None)
        }
        ObservedPortResult::UnresolvedSchema { .. } => (ProofVerdict::Unproven, None),
        ObservedPortResult::MissingPort {
            guard,
            input_declarations,
        } => (
            ProofVerdict::Violation,
            Some(NativeLocalWitness::MissingPort {
                role,
                requested_port: guard.port_no,
                input_declarations: input_declarations.clone(),
            }),
        ),
        ObservedPortResult::TypeMismatch {
            port,
            guard,
            expanded,
            ..
        } => (
            ProofVerdict::Violation,
            Some(NativeLocalWitness::ConstantTypeMismatch {
                role,
                port: Box::new(port.clone()),
                type_ref: port.type_ref.clone(),
                expanded_type: Box::new(expanded.clone()),
                constant: guard.expected.clone(),
            }),
        ),
    }
}
pub fn collect_native_guard<'s, 'c, 'u>(
    source: &'s CanonicalGuardObservation<'c, 'u>,
    analysis_context: &StaticContractProofContext<'_>,
) -> Result<NativeLocalCapture<'s, 'c, 'u>, NativeCaptureFailure> {
    if !source.uses_context(analysis_context) {
        return Err(NativeCaptureFailure::InvalidContextBinding);
    }
    // A legacy canonicalizer/encoder success alone cannot admit a physical
    // declaration whose actual guard text is outside the registered String
    // domain. Do not normalize after evaluating: that would invent a source
    // for the already executed native judgment. Preserve that judgment as a
    // diagnostic, but fail closed before issuing any formal local seed.
    let entries = match &source.declaration().effects.row {
        af_effect::EffectRow::Closed(entries) => entries.as_slice(),
        af_effect::EffectRow::Open(open) => open.known.as_slice(),
        af_effect::EffectRow::Unknown => &[],
    };
    if entries.iter().any(|entry| {
        if let af_effect::EffectGuard::InputEquals(g) = &entry.guard {
            if let af_semantic::PrimitiveConstant::String(s) = &g.expected {
                return !unicode_normalization::is_nfc(s);
            }
        }
        false
    }) {
        return Err(NativeCaptureFailure::InvalidFacts);
    }
    let compiler = current_registered_analysis_edition();
    if compiler.registration != EditionRegistration::Registered || compiler.can_issue_capability() {
        return Err(NativeCaptureFailure::UnsupportedModel);
    }
    let mut seeds = Vec::new();
    let canonical = emit(
        &mut seeds,
        NativeCase::CanonicalSource,
        None,
        None,
        ProofVerdict::Proven,
        vec![],
        None,
    )?;
    let context = emit(
        &mut seeds,
        NativeCase::ContextAssociation,
        None,
        None,
        ProofVerdict::Proven,
        vec![premise(NativePremise::CanonicalSource, canonical, true)],
        None,
    )?;
    for run in source.runs() {
        let base = vec![
            premise(NativePremise::CanonicalSource, canonical, true),
            premise(NativePremise::ContextAssociation, context, true),
        ];
        let ready = matches!(run.forward, ObservedResolution::Present(_))
            && matches!(run.action, ObservedResolution::Present(_))
            && matches!(run.usage, ObservedResolution::Present(_));
        let sources = emit(
            &mut seeds,
            NativeCase::ActionSources,
            Some(run.binding_ordinal),
            None,
            if ready {
                ProofVerdict::Proven
            } else {
                ProofVerdict::Unproven
            },
            base.clone(),
            None,
        )?;
        let mut evaluation_premises = base;
        let mut evaluation_witness = None;
        evaluation_premises.push(premise(NativePremise::ActionSourcesReady, sources, ready));
        if let Some(bounds) = &run.bounds {
            let v = if bounds.failure.is_some() {
                ProofVerdict::Violation
            } else if bounds.unisolated.is_some() {
                ProofVerdict::Unproven
            } else {
                ProofVerdict::Proven
            };
            let witness = bounds.failure.clone().map(NativeLocalWitness::BoundFailure);
            let b = emit(
                &mut seeds,
                NativeCase::ActionBounds,
                Some(run.binding_ordinal),
                None,
                v,
                vec![premise(NativePremise::ActionSourcesReady, sources, ready)],
                witness.clone(),
            )?;
            evaluation_premises.push(premise(
                NativePremise::ActionBoundsReady,
                b,
                v == ProofVerdict::Proven,
            ));
            if let Some(w) = witness {
                evaluation_witness = Some(NativeLocalWitness::PropagatedFailure {
                    cause: b,
                    witness: Box::new(w),
                });
            }
            for (role, outcome) in [
                (ObservedGuardRole::Forward, &run.forward_input),
                (ObservedGuardRole::Action, &run.action_input),
            ] {
                if let ObservedResolution::Present(p) = outcome {
                    let (v, w) = input_result(p, role);
                    let input = emit(
                        &mut seeds,
                        NativeCase::GuardInputs,
                        Some(run.binding_ordinal),
                        Some(role),
                        v,
                        vec![premise(NativePremise::ActionBoundsReady, b, true)],
                        w.clone(),
                    )?;
                    // Selecting an Action error also consumed the successful
                    // Forward check. A Forward error does not consume Action's
                    // independently captured eager result as a validity premise.
                    if !(role == ObservedGuardRole::Action
                        && run.primary_error == Some(ObservedGuardRole::Forward))
                    {
                        evaluation_premises.push(premise(
                            NativePremise::GuardInputValid(role),
                            input,
                            v == ProofVerdict::Proven,
                        ));
                        if let Some(w) = w {
                            evaluation_witness = Some(NativeLocalWitness::PropagatedFailure {
                                cause: input,
                                witness: Box::new(w),
                            });
                        }
                    }
                }
            }
        }
        if let ObservedResolution::Present(value) = run.containment {
            let p = evaluation_premises.clone();
            let c = emit(
                &mut seeds,
                NativeCase::GuardContainment,
                Some(run.binding_ordinal),
                None,
                if value {
                    ProofVerdict::Proven
                } else {
                    ProofVerdict::Unproven
                },
                p,
                None,
            )?;
            // Established Boolean observation != supporting compatibility P.
            evaluation_premises.push(premise(NativePremise::ContainmentOutcome(value), c, true));
        }
        let record = run
            .record
            .as_ref()
            .ok_or(NativeCaptureFailure::IncompleteFacts)?;
        if run.branch == Some(ObservedGuardBranch::TypedCountermodel) {
            evaluation_witness = Some(NativeLocalWitness::InputCountermodel(Box::new(
                record
                    .countermodel()
                    .ok_or(NativeCaptureFailure::IncompleteFacts)?
                    .clone(),
            )));
        }
        emit(
            &mut seeds,
            NativeCase::GuardEvaluation,
            Some(run.binding_ordinal),
            None,
            record.verdict(),
            evaluation_premises,
            evaluation_witness,
        )?;
    }
    let capture = NativeLocalCapture {
        source,
        compiler,
        seeds,
    };
    capture.validate()?;
    Ok(capture)
}
