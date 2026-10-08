//! Structured static contract diagnostics.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractError {
    pub rule_id: &'static str,
    pub code: &'static str,
    pub reference: Option<String>,
    pub detail: String,
}

impl ContractError {
    #[must_use]
    pub fn new(rule_id: &'static str, code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            rule_id,
            code,
            reference: None,
            detail: detail.into(),
        }
    }

    #[must_use]
    pub fn at(mut self, reference: impl Into<String>) -> Self {
        self.reference = Some(reference.into());
        self
    }
}

impl std::fmt::Display for ContractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}]: {}", self.rule_id, self.code, self.detail)
    }
}

impl std::error::Error for ContractError {}
