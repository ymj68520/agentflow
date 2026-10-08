//! Flow non-relaxation: the F-dimension contract-evolution proof.
//!
//! The frozen rule: `F_new MUST NOT relax F_old` — every flow permitted
//! by the new contract must also be permitted by the old one. This is
//! NOT "more labels = safer": the five dimensions (confidentiality
//! restrictions, taint influence, instruction authority, data-egress
//! permissions, decision-influence permissions) are judged separately,
//! each in its own direction.
//!
//! What this phase can prove is the explicit-label / exact-sink subset:
//!
//! - Confidentiality restrictions: the new set must be a SUPERSET of the
//!   old one (dropping a restriction permits flows the old contract
//!   forbade → relaxation).
//! - Declared ceilings: the new constraint set must be a SUBSET of the
//!   old one, per kind (adding a permitted sink is relaxation). The
//!   two kinds are never compared across: a new `data_egress` ceiling
//!   says nothing about `decision_influence`.
//! - Instruction authority: the new summary's authority must not EXCEED
//!   the old one (a higher authority is a stronger claim the new
//!   contract must prove, not inherit).
//! - Unknown influence: `influence_unknown` must not disappear (unknown
//!   repaired into known is a silent widening).
//!
//! Everything else is `Unproven` and fails closed: no invented lattice,
//! no automatic declassification, no cross-catalog subtyping.

use crate::canonical::canonicalize_information_flow_contract;
use crate::error::{FlowError, kind};
use crate::model::{FlowKind, InformationFlowContract, SourceCategory};
use crate::universe::FlowCatalogs;
use std::collections::BTreeSet;

/// Why a non-relaxation judgment could not be proven. The stable names
/// are frozen in the compatibility corpus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnprovenReason {
    /// A taint or influence relationship between two declared states is
    /// not decidable by the closed rules of this phase.
    InfluenceRelationUnknown,
}

impl UnprovenReason {
    /// The stable machine name used by the corpus.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            UnprovenReason::InfluenceRelationUnknown => "influence_relation_unknown",
        }
    }
}

/// The static compatibility result. `Proven` means the new contract
/// provably does not relax the old one; `Unproven` fails closed and is
/// never a substitution approval.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowCompatibilityProof {
    /// Non-relaxation holds.
    Proven,
    /// Non-relaxation could not be proven, with the reason.
    Unproven(UnprovenReason),
}

/// Source-set comparison for the summary dimension: the new summary's
/// source set must not be narrower than the old one's — sources are
/// influence CAPACITY, and claiming fewer sources is a narrowing that
/// this phase cannot prove (it would need a full dataflow proof). The
/// `unknown_external` source makes the set Top: a contract that admits
/// an unknown external source cannot be proven narrower than anything.
fn sources_not_narrowed(
    new_sources: &BTreeSet<SourceCategory>,
    old_sources: &BTreeSet<SourceCategory>,
) -> bool {
    if old_sources.contains(&SourceCategory::UnknownExternal) {
        // The old contract's source set is Top; any new set is inside it.
        return true;
    }
    if new_sources.contains(&SourceCategory::UnknownExternal) {
        // The new contract admits an unknown external source the old one
        // did not: that is a widening of influence capacity, but a
        // WIDENING of sources is conservative-safe, not a relaxation of
        // permissions (permissions are judged per constraint below).
        return true;
    }
    // Both concrete: the new set must be a superset of the old one —
    // dropping a source would be a silent influence-narrowing claim.
    old_sources.is_subset(new_sources)
}

/// `flow_not_relaxed`: proves that the new contract permits no flow the
/// old contract forbade, within the explicit-catalog subset this phase
/// can decide. Both contracts are canonicalized first (unknown labels
/// fail there, never here).
///
/// # Errors
/// [`FlowError`] with the kinds of
/// [`canonicalize_information_flow_contract`] (propagated from
/// canonicalization) and `flow_not_proven` when the comparison itself is
/// requested across catalogs that make a declared identity unresolvable.
pub fn flow_not_relaxed(
    new_contract: &InformationFlowContract,
    old_contract: &InformationFlowContract,
    catalogs: &FlowCatalogs,
) -> Result<FlowCompatibilityProof, FlowError> {
    let new = canonicalize_information_flow_contract(new_contract, catalogs)?;
    let old = canonicalize_information_flow_contract(old_contract, catalogs)?;

    // 1. Confidentiality restrictions: dropping one permits flows the
    //    old contract forbade. New must be a superset.
    let new_restrictions: BTreeSet<_> = new
        .confidentiality_restrictions
        .iter()
        .map(super::model::ConfidentialityLabelRef::identity_key)
        .collect();
    let old_restrictions: BTreeSet<_> = old
        .confidentiality_restrictions
        .iter()
        .map(super::model::ConfidentialityLabelRef::identity_key)
        .collect();
    if !old_restrictions.is_subset(&new_restrictions) {
        return Err(FlowError::new(
            kind::FLOW_CONTRACT_RELAXED,
            "the new contract dropped a confidentiality restriction: flows \
             the old contract restricted would become unlabelled",
        ));
    }

    // 2. Declared ceilings per kind: adding a permitted sink is
    //    relaxation. New constraints must be a subset of old, per kind.
    for kind_kind in [FlowKind::DataEgress, FlowKind::DecisionInfluence] {
        let new_sinks: BTreeSet<_> = new
            .flow_constraints
            .iter()
            .filter(|constraint| constraint.kind == kind_kind)
            .map(|constraint| &constraint.sink)
            .collect();
        let old_sinks: BTreeSet<_> = old
            .flow_constraints
            .iter()
            .filter(|constraint| constraint.kind == kind_kind)
            .map(|constraint| &constraint.sink)
            .collect();
        if !new_sinks.is_subset(&old_sinks) {
            return Err(FlowError::new(
                kind::FLOW_CONTRACT_RELAXED,
                format!(
                    "the new contract declares a {} sink the old contract \
                     did not permit: a newly permitted flow",
                    match kind_kind {
                        FlowKind::DataEgress => "data-egress",
                        FlowKind::DecisionInfluence => "decision-influence",
                    }
                ),
            ));
        }
    }

    // 3. Instruction authority: the new summary must not claim a higher
    //    authority than the old one. Authority is not an attribute data
    //    acquires by transformation.
    if new.summary.instruction_authority > old.summary.instruction_authority {
        return Err(FlowError::new(
            kind::FLOW_CONTRACT_RELAXED,
            format!(
                "the new contract claims instruction authority {:?} where \
                 the old contract declared {:?}: authority is never elevated \
                 by a contract rewrite",
                new.summary.instruction_authority.as_str(),
                old.summary.instruction_authority.as_str()
            ),
        ));
    }

    // 4. Unknown influence: repaired unknown is a silent widening. If the
    //    old summary declared unknown influence, the new one must too.
    if old.summary.influence_unknown && !new.summary.influence_unknown {
        return Ok(FlowCompatibilityProof::Unproven(
            UnprovenReason::InfluenceRelationUnknown,
        ));
    }

    // 5. Source capacity: the new summary may not claim a narrower source
    //    set (that would be an unproven influence narrowing).
    let new_sources: BTreeSet<_> = new
        .summary
        .sources
        .iter()
        .map(|source| source.category.clone())
        .collect();
    let old_sources: BTreeSet<_> = old
        .summary
        .sources
        .iter()
        .map(|source| source.category.clone())
        .collect();
    if !sources_not_narrowed(&new_sources, &old_sources) {
        return Ok(FlowCompatibilityProof::Unproven(
            UnprovenReason::InfluenceRelationUnknown,
        ));
    }

    Ok(FlowCompatibilityProof::Proven)
}
