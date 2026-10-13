use std::fmt;

/// The first validator stage that rejected portable input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationStage {
    D0,
    D1,
    D2,
    D3,
    D4,
}

/// Stable, path-bearing failure for D0-D4. Diagnostic text is not an ABI field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortableProofError {
    pub stage: ValidationStage,
    pub family: &'static str,
    pub path: String,
    pub detail: String,
}

impl PortableProofError {
    pub(crate) fn new(
        stage: ValidationStage,
        family: &'static str,
        path: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            stage,
            family,
            path: path.into(),
            detail: detail.into(),
        }
    }
}

impl fmt::Display for PortableProofError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} {} at {}: {}",
            self.stage, self.family, self.path, self.detail
        )
    }
}

impl std::error::Error for PortableProofError {}
