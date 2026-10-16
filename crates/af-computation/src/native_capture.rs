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
            record_ordinal: binding,
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
                    if run.primary_error.is_none() || run.primary_error == Some(role) {
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
    Ok(NativeLocalCapture {
        source,
        compiler,
        seeds,
    })
}
