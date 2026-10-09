//! Per-dimension static proof outcomes.

use crate::model::ComponentContract;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofVerdict {
    Proven,
    Violation,
    Unproven,
    RequiresEvidence,
}

impl ProofVerdict {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Proven => "proven",
            Self::Violation => "violation",
            Self::Unproven => "unproven",
            Self::RequiresEvidence => "requires_evidence",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DimensionProof {
    pub dimension: &'static str,
    pub verdict: ProofVerdict,
    pub rule_id: &'static str,
    pub reference: Option<String>,
    pub reason: String,
    pub prerequisites: Vec<String>,
}

impl DimensionProof {
    #[must_use]
    pub fn proven(
        dimension: &'static str,
        rule_id: &'static str,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            dimension,
            verdict: ProofVerdict::Proven,
            rule_id,
            reference: None,
            reason: reason.into(),
            prerequisites: vec![],
        }
    }

    #[must_use]
    pub fn unproven(
        dimension: &'static str,
        rule_id: &'static str,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            dimension,
            verdict: ProofVerdict::Unproven,
            rule_id,
            reference: None,
            reason: reason.into(),
            prerequisites: vec![],
        }
    }

    #[must_use]
    pub fn violation(
        dimension: &'static str,
        rule_id: &'static str,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            dimension,
            verdict: ProofVerdict::Violation,
            rule_id,
            reference: None,
            reason: reason.into(),
            prerequisites: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogContentChecksum {
    pub algorithm: &'static str,
    pub hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationCatalogPin {
    pub schema: String,
    pub revision: String,
    pub canonical_content_checksum: CatalogContentChecksum,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofReport {
    pub dimensions: Vec<DimensionProof>,
    pub relation_catalogs: Vec<RelationCatalogPin>,
    /// Typed D1 evidence for each RecoveryActionBinding checked against its
    /// exact Forward and action EffectEntry guards.
    pub action_guard_compatibility: Vec<crate::recovery_guard::ActionGuardCompatibility>,
}

impl ProofReport {
    #[must_use]
    pub fn overall(&self) -> ProofVerdict {
        if self
            .dimensions
            .iter()
            .any(|item| item.verdict == ProofVerdict::Violation)
        {
            ProofVerdict::Violation
        } else if self
            .dimensions
            .iter()
            .any(|item| item.verdict == ProofVerdict::RequiresEvidence)
        {
            ProofVerdict::RequiresEvidence
        } else if self
            .dimensions
            .iter()
            .any(|item| item.verdict == ProofVerdict::Unproven)
        {
            ProofVerdict::Unproven
        } else {
            ProofVerdict::Proven
        }
    }

    #[must_use]
    pub fn is_proven(&self) -> bool {
        self.overall() == ProofVerdict::Proven
    }
}

/// Checked values keep their constructor private to the validator module.
pub struct CheckedComponentContract<'context, 'universe> {
    pub(crate) contract: ComponentContract,
    pub(crate) proof: ProofReport,
    pub(crate) context: &'context crate::validate::StaticContractProofContext<'universe>,
}

impl std::fmt::Debug for CheckedComponentContract<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CheckedComponentContract")
            .field("contract", &self.contract)
            .field("proof", &self.proof)
            .finish_non_exhaustive()
    }
}

impl CheckedComponentContract<'_, '_> {
    #[must_use]
    pub fn contract(&self) -> &ComponentContract {
        &self.contract
    }
    #[must_use]
    pub fn proof(&self) -> &ProofReport {
        &self.proof
    }

    /// The exact immutable proof context under which this checked value was
    /// issued. Callers may compare this reference by identity; an equivalent
    /// but distinct context requires revalidation before its proof is reused.
    #[must_use]
    pub fn proof_context(&self) -> &crate::validate::StaticContractProofContext<'_> {
        self.context
    }
}
