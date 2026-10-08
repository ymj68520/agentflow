//! Stable, dimension-preserving proof outcomes for one computation leaf.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeafVerdict {
    Proven,
    Violation,
    Unproven,
    RequiresEvidence,
}

impl LeafVerdict {
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
pub struct LeafDimensionProof {
    dimension: &'static str,
    verdict: LeafVerdict,
    rule_id: &'static str,
    reference: Option<String>,
    reason: String,
    prerequisites: Vec<String>,
}

impl LeafDimensionProof {
    pub(crate) fn new(
        dimension: &'static str,
        verdict: LeafVerdict,
        rule_id: &'static str,
        reference: Option<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            dimension,
            verdict,
            rule_id,
            reference,
            reason: reason.into(),
            prerequisites: Vec::new(),
        }
    }

    #[must_use]
    pub fn dimension(&self) -> &'static str {
        self.dimension
    }
    #[must_use]
    pub fn verdict(&self) -> LeafVerdict {
        self.verdict
    }
    #[must_use]
    pub fn rule_id(&self) -> &'static str {
        self.rule_id
    }
    #[must_use]
    pub fn reference(&self) -> Option<&str> {
        self.reference.as_deref()
    }
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
    #[must_use]
    pub fn prerequisites(&self) -> &[String] {
        &self.prerequisites
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeafProofReport {
    dimensions: Vec<LeafDimensionProof>,
}

impl LeafProofReport {
    pub(crate) fn new(dimensions: Vec<LeafDimensionProof>) -> Self {
        Self { dimensions }
    }

    #[must_use]
    pub fn dimensions(&self) -> &[LeafDimensionProof] {
        &self.dimensions
    }

    #[must_use]
    pub fn overall(&self) -> LeafVerdict {
        if self
            .dimensions
            .iter()
            .any(|item| item.verdict == LeafVerdict::Violation)
        {
            LeafVerdict::Violation
        } else if self
            .dimensions
            .iter()
            .any(|item| item.verdict == LeafVerdict::RequiresEvidence)
        {
            LeafVerdict::RequiresEvidence
        } else if self
            .dimensions
            .iter()
            .any(|item| item.verdict == LeafVerdict::Unproven)
        {
            LeafVerdict::Unproven
        } else {
            LeafVerdict::Proven
        }
    }
}
