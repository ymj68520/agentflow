//! Conservative taint propagation and the LLM strong taint mixer.
//!
//! Everything here is a pure function over explicit values; catalogs are
//! parameters, never ambient state. The governing rule is
//! **may-influence**: an output's taint is the union of the taints of all
//! inputs that MAY influence it — not only the observed direct
//! dependencies. Where the influence relation is incomplete, the result
//! is the explicit unknown (`influence_unknown = true`) or the
//! conservative union of all possible inputs. No component author can
//! prove a real influence away by writing an incomplete influence list.
//!
//! The model-mixer rule is the frozen strong taint model: all
//! model-visible inputs may influence all model outputs, unless a
//! stronger verifiable proof exists. Model output therefore inherits the
//! union of every model-visible input's taints, plus the `model_generated`
//! taint, and its influence stays unknown unless proven otherwise.

use crate::error::{FlowError, kind};
use crate::model::{InstructionAuthority, StaticInformationSummary, TaintRef};
use crate::universe::check_label;
use std::collections::BTreeSet;
use unicode_normalization::UnicodeNormalization;

fn normalize(taint: &TaintRef) -> TaintRef {
    TaintRef {
        catalog: taint.catalog.clone(),
        name: taint.name.nfc().collect(),
    }
}

/// `union_taints`: the set union of taint sets, NFC-normalized, sorted by
/// nominal identity, duplicates deduplicated. Commutative, associative,
/// idempotent; the empty set is the identity. Union NEVER loses a taint
/// and never invents one.
#[must_use]
pub fn union_taints(sets: &[Vec<TaintRef>]) -> Vec<TaintRef> {
    let mut seen = BTreeSet::new();
    for set in sets {
        for taint in set {
            seen.insert((normalize(taint).catalog, normalize(taint).name));
        }
    }
    seen.into_iter()
        .map(|(catalog, name)| TaintRef { catalog, name })
        .collect()
}

/// `merge_provenance_summaries`: the conservative merge of influence
/// summaries. Sources union, taints union, instruction authority takes
/// the MAXIMUM (weakest direction first: a flow that may carry `user`
/// instructions may carry `external_data` instructions, never the
/// reverse), and influence_unknown is sticky: unknown stays unknown.
#[must_use]
pub fn merge_provenance_summaries(
    summaries: &[&StaticInformationSummary],
) -> StaticInformationSummary {
    let sources: Vec<_> = summaries.iter().flat_map(|s| s.sources.iter()).collect();
    let mut unique_sources = BTreeSet::new();
    for source in sources {
        unique_sources.insert(source.clone());
    }
    let taints = union_taints(
        &summaries
            .iter()
            .map(|summary| summary.taints.clone())
            .collect::<Vec<_>>(),
    );
    let authority = summaries
        .iter()
        .map(|summary| summary.instruction_authority)
        .max()
        .unwrap_or(InstructionAuthority::None);
    StaticInformationSummary {
        sources: unique_sources.into_iter().collect(),
        taints,
        instruction_authority: authority,
        influence_unknown: summaries.iter().any(|s| s.influence_unknown),
    }
}

/// `propagate_pure_expression`: static propagation through an ordinary
/// data transformation (pure expression, structural copy, serialization,
/// store/load round trip, summarization-by-another-component…). The
/// output inherits the union of ALL listed input taints and the MAXIMUM
/// input authority — inherited, never claimed. `claimed_authority` is
/// the authority the caller believes the output carries; claiming MORE
/// than the inputs carry is instruction-authority elevation and is
/// rejected. `influence_unknown` is sticky.
///
/// # Errors
/// [`FlowError`] with kind `invalid_flow_source` when the claimed
/// authority exceeds every input's (elevation attempt).
pub fn propagate_pure_expression(
    inputs: &[&StaticInformationSummary],
    claimed_authority: InstructionAuthority,
) -> Result<StaticInformationSummary, FlowError> {
    let merged = merge_provenance_summaries(inputs);
    if claimed_authority > merged.instruction_authority {
        return Err(FlowError::new(
            kind::INVALID_FLOW_SOURCE,
            format!(
                "an ordinary data transformation cannot elevate \
                 instruction authority: inputs carry at most {:?} but \
                 the output claims {:?}",
                merged.instruction_authority.as_str(),
                claimed_authority.as_str()
            ),
        ));
    }
    // The inherited maximum stands even when the claim is lower: a
    // claim can never launder authority out of the data.
    Ok(merged)
}

/// `propagate_unknown_transformation`: the output of a transformation
/// whose influence relation is NOT fully known. Conservative result: the
/// union of every POSSIBLE input's taints plus the explicit unknown flag.
/// This is the only way a component author may "narrow" nothing —
/// unknown influence stays unknown, and no input can be proven
/// uninfluential here.
#[must_use]
pub fn propagate_unknown_transformation(
    possible_inputs: &[&StaticInformationSummary],
) -> StaticInformationSummary {
    let mut merged = merge_provenance_summaries(possible_inputs);
    merged.influence_unknown = true;
    merged
}

/// `mix_model_visible_inputs`: the LLM strong taint mixer. ALL
/// model-visible inputs may influence ALL model outputs (frozen
/// architecture), so the output summary is:
///
/// ```text
/// taints = union(all model-visible input taints) + model_generated
/// influence_unknown = true   (a model is a black box)
/// authority = max(input authorities)   (never elevated by the model)
/// ```
///
/// `model_generated` is supplied by the caller and must resolve in the
/// supplied label universe — the mixer adds no unregistered taint.
///
/// # Errors
/// [`FlowError`] with kinds `invalid_information_label` (malformed
/// reference) and `unknown_information_label` (`model_generated` not
/// registered).
pub fn mix_model_visible_inputs(
    model_visible_inputs: &[&StaticInformationSummary],
    model_generated: &TaintRef,
    universe: &crate::universe::InformationLabelUniverse,
) -> Result<StaticInformationSummary, FlowError> {
    let mut merged = merge_provenance_summaries(model_visible_inputs);
    let marker = normalize(model_generated);
    check_label(&marker.catalog, &marker.name)?;
    universe.resolve_taint(&marker)?;
    let mut taints = union_taints(&[merged.taints.clone(), vec![marker]]);
    taints.sort();
    merged.taints = taints;
    // The model is a black box: its internal influence is never provable
    // from inputs alone. Unknown stays unknown.
    merged.influence_unknown = true;
    Ok(merged)
}

/// The explicit result of a static flow judgment against one declared
/// ceiling. `RequiresTrustedTransformation` is a static OBLIGATION — it
/// never means a declassification or endorsement happened or was
/// approved, and it is not a `PolicyDecision::Allow` substitute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowProof {
    /// The flow is statically proven to stay within the declared
    /// ceiling.
    ProvenWithinDeclaredCeiling,
    /// The flow needs a declassification or endorsement this layer cannot
    /// supply; the requirement is recorded as an obligation.
    RequiresTrustedTransformation {
        /// The flow would carry data under a declared confidentiality
        /// restriction: a declassification obligation.
        requires_declassification: bool,
        /// The flow would carry tainted influence into a decision: an
        /// endorsement obligation.
        requires_endorsement: bool,
    },
    /// The static relationship could not be proven; fail closed.
    Unproven,
}

impl FlowProof {
    /// The stable machine name used by the corpus.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            FlowProof::ProvenWithinDeclaredCeiling => "proven_within_declared_ceiling",
            FlowProof::RequiresTrustedTransformation { .. } => "requires_trusted_transformation",
            FlowProof::Unproven => "unproven",
        }
    }

    /// Demands a proven flow, failing closed: Unproven is
    /// `flow_not_proven` and an unmet obligation is
    /// `privileged_transformation_required`.
    ///
    /// # Errors
    /// [`FlowError`] with kinds `flow_not_proven` and
    /// `privileged_transformation_required`.
    pub fn require_proven(&self) -> Result<(), FlowError> {
        match self {
            FlowProof::ProvenWithinDeclaredCeiling => Ok(()),
            FlowProof::RequiresTrustedTransformation { .. } => Err(FlowError::new(
                kind::PRIVILEGED_TRANSFORMATION_REQUIRED,
                "the declared flow needs a declassification or \
                     endorsement this static layer cannot supply: the \
                     obligation is recorded, never executed",
            )),
            FlowProof::Unproven => Err(FlowError::new(
                kind::FLOW_NOT_PROVEN,
                "the static flow relationship could not be proven: \
                 Unproven is never judged allowed",
            )),
        }
    }
}
