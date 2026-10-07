//! `SpecificationId` — the strict machine identity of a schema family.
//!
//! Canonical form: `<ns-segment>("." ns-segment)* "@" <major>`, e.g.
//! `agentflow.core.machine-primitives@1`.
//!
//! - namespace segments are ASCII lowercase kebab: `[a-z][a-z0-9-]*`;
//! - `major` is a positive u32 written without leading zeros;
//! - parsing is strict and total: no whitespace, no Unicode, no case folding.
//!
//! A `SpecificationId` is *not* a HashV1 domain string: the digest domain
//! `type-schema` frozen in Phase 1 is a separate, shorter identifier and the
//! two must never be conflated.

use crate::error::AfidlError;
use crate::span::SourceSpan;
use std::fmt;

/// The schema family identity: namespace + semantic major.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpecificationId {
    namespace: String,
    major: u32,
}

impl SpecificationId {
    /// Strictly parses a canonical specification id.
    pub fn parse(text: &str) -> Result<Self, AfidlError> {
        let span = SourceSpan::new(0, text.len(), 1, 1);
        let (namespace, major) =
            text.rsplit_once('@')
                .ok_or_else(|| AfidlError::InvalidSpecificationId {
                    detail: "missing '@<major>'".into(),
                    span,
                })?;
        if namespace.is_empty() {
            return Err(AfidlError::InvalidSpecificationId {
                detail: "empty namespace".into(),
                span,
            });
        }
        for segment in namespace.split('.') {
            Self::validate_segment(segment)?;
        }
        if major.is_empty() || !major.bytes().all(|b| b.is_ascii_digit()) {
            return Err(AfidlError::InvalidSpecificationId {
                detail: format!("invalid major {major:?}"),
                span,
            });
        }
        if major.len() > 1 && major.starts_with('0') {
            return Err(AfidlError::InvalidSpecificationId {
                detail: format!("major {major:?} has leading zeros"),
                span,
            });
        }
        let major_value: u32 = major
            .parse()
            .map_err(|_| AfidlError::InvalidSpecificationId {
                detail: format!("major {major:?} out of range"),
                span,
            })?;
        if major_value == 0 {
            return Err(AfidlError::InvalidSpecificationId {
                detail: "major must be positive".into(),
                span,
            });
        }
        Ok(Self {
            namespace: namespace.to_string(),
            major: major_value,
        })
    }

    /// Builds from already-validated parts (used by the parser, which
    /// assembles the id from lexed tokens).
    pub fn from_parts(namespace: impl Into<String>, major: u32) -> Result<Self, AfidlError> {
        let namespace = namespace.into();
        let span = SourceSpan::new(0, namespace.len(), 1, 1);
        if namespace.is_empty() {
            return Err(AfidlError::InvalidSpecificationId {
                detail: "empty namespace".into(),
                span,
            });
        }
        for segment in namespace.split('.') {
            Self::validate_segment(segment)?;
        }
        if major == 0 {
            return Err(AfidlError::InvalidSpecificationId {
                detail: "major must be positive".into(),
                span,
            });
        }
        Ok(Self { namespace, major })
    }

    fn validate_segment(segment: &str) -> Result<(), AfidlError> {
        let span = SourceSpan::new(0, 0, 1, 1);
        let invalid = |detail: String| Err(AfidlError::InvalidSpecificationId { detail, span });
        let bytes = segment.as_bytes();
        if bytes.is_empty() {
            return invalid("empty namespace segment".into());
        }
        if !bytes[0].is_ascii_lowercase() {
            return invalid(format!(
                "segment {segment:?} must start with a lowercase ASCII letter"
            ));
        }
        if !bytes[1..]
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
        {
            return invalid(format!(
                "segment {segment:?} may contain only lowercase ASCII letters, digits and '-'"
            ));
        }
        Ok(())
    }

    /// The namespace part (dot-separated lowercase kebab segments).
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// The semantic major of the schema family.
    pub fn major(&self) -> u32 {
        self.major
    }

    /// The canonical display form; identical to the strict parse form.
    pub fn canonical(&self) -> String {
        format!("{}@{}", self.namespace, self.major)
    }
}

impl fmt::Display for SpecificationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.canonical())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_canonical_forms() {
        let id = SpecificationId::parse("agentflow.core.machine-primitives@1").unwrap();
        assert_eq!(id.namespace(), "agentflow.core.machine-primitives");
        assert_eq!(id.major(), 1);
        assert_eq!(id.canonical(), "agentflow.core.machine-primitives@1");

        let id = SpecificationId::parse("a.b@42").unwrap();
        assert_eq!(id.major(), 42);
        assert_eq!(id.canonical(), "a.b@42");
    }

    #[test]
    fn rejects_invalid_forms() {
        for text in [
            "",
            "agentflow",
            "agentflow@",
            "agentflow@0",
            "agentflow@01",
            "agentflow@x",
            "agentflow..example@1",
            ".agentflow@1",
            "agentflow.@1",
            "Agentflow@1",
            "agent_flow@1", // underscore not allowed in namespace segments
        ] {
            assert!(
                SpecificationId::parse(text).is_err(),
                "expected {text:?} to be rejected"
            );
        }
        // kebab segments are legal
        assert!(SpecificationId::parse("agent-flow@1").is_ok());
    }

    #[test]
    fn rejects_unicode_and_whitespace() {
        assert!(SpecificationId::parse("agentflow‑example@1").is_err()); // non-ASCII hyphen
        assert!(SpecificationId::parse("agentflow example@1").is_err());
        assert!(SpecificationId::parse(" agentflow@1").is_err());
    }
}
