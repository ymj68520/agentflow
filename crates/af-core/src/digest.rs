//! HashV1, the frozen digest-domain registry, and [`DigestRef`].
//!
//! Frozen algorithm (v1 normative freeze, §24.1 of the architecture baseline):
//!
//! ```text
//! HashV1(domain, payload) = SHA256("AF\0" || domain || "\0" || payload)
//! ```
//!
//! - the prefix is the exact ASCII bytes `AF` followed by a single NUL;
//! - the domain is one of the frozen registry names below, encoded as its
//!   ASCII bytes, followed by a single NUL;
//! - the payload is the canonical ACE1 bytes of the hashed object;
//! - domain separation guarantees `same payload + different domain →
//!   different digest`.
//!
//! Digest domains are centrally registered in `spec/tables/hash-domains.json`
//! (mirrored by [`DigestDomain`]); implementations must never invent domains
//! by string concatenation. A conformance test asserts the enum and the JSON
//! table never drift apart.

use crate::error::DigestError;
use sha2::{Digest, Sha256};
use std::fmt;

/// Exact ASCII prefix of the HashV1 preimage: `AF` + NUL.
pub const HASH_V1_PREFIX: &[u8] = b"AF\0";

/// The frozen v1 digest-domain registry.
///
/// The string values are normative; do not abbreviate or rename them. This
/// enum must stay in lockstep with `spec/tables/hash-domains.json`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DigestDomain {
    /// Canonical type/schema objects.
    TypeSchema,
    /// Canonical semantic Values.
    Value,
    /// Provenance DAG nodes.
    Provenance,
    /// Immutable artifact content identities.
    Artifact,
    /// Canonical Semantic IR.
    SemanticIr,
    /// Executable Plan core (durable state-machine program).
    PlanCore,
    /// Immutable per-Run admission manifests.
    RunManifest,
    /// Immutable resolver LockGraphs.
    LockGraph,
    /// Canonical Journal event payloads.
    JournalEventPayload,
    /// Canonical Runtime Command payloads.
    RuntimeCommandPayload,
    /// Canonical SemanticExecutionState materializations.
    SemanticExecutionState,
    /// Component contracts.
    ComponentContract,
    /// Resource contracts.
    ResourceContract,
    /// Conformance objects (golden/invariant corpus entries).
    Conformance,
}

impl DigestDomain {
    /// The normative registry string.
    pub fn as_str(&self) -> &'static str {
        match self {
            DigestDomain::TypeSchema => "type-schema",
            DigestDomain::Value => "value",
            DigestDomain::Provenance => "provenance",
            DigestDomain::Artifact => "artifact",
            DigestDomain::SemanticIr => "semantic-ir",
            DigestDomain::PlanCore => "plan-core",
            DigestDomain::RunManifest => "run-manifest",
            DigestDomain::LockGraph => "lock-graph",
            DigestDomain::JournalEventPayload => "journal-event-payload",
            DigestDomain::RuntimeCommandPayload => "runtime-command-payload",
            DigestDomain::SemanticExecutionState => "semantic-execution-state",
            DigestDomain::ComponentContract => "component-contract",
            DigestDomain::ResourceContract => "resource-contract",
            DigestDomain::Conformance => "conformance",
        }
    }

    /// All registry entries, in registry order.
    pub const ALL: &'static [DigestDomain] = &[
        DigestDomain::TypeSchema,
        DigestDomain::Value,
        DigestDomain::Provenance,
        DigestDomain::Artifact,
        DigestDomain::SemanticIr,
        DigestDomain::PlanCore,
        DigestDomain::RunManifest,
        DigestDomain::LockGraph,
        DigestDomain::JournalEventPayload,
        DigestDomain::RuntimeCommandPayload,
        DigestDomain::SemanticExecutionState,
        DigestDomain::ComponentContract,
        DigestDomain::ResourceContract,
        DigestDomain::Conformance,
    ];

    /// Resolves a registry string back to the domain.
    pub fn from_registry_name(name: &str) -> Result<Self, DigestError> {
        DigestDomain::ALL
            .iter()
            .copied()
            .find(|d| d.as_str() == name)
            .ok_or_else(|| DigestError::UnknownDigestDomain {
                domain: name.to_string(),
            })
    }
}

impl fmt::Display for DigestDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The digest algorithm. v1 freezes exactly one algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DigestAlgorithm {
    Sha256,
}

impl DigestAlgorithm {
    /// The normative algorithm name (lowercase).
    pub fn as_str(&self) -> &'static str {
        match self {
            DigestAlgorithm::Sha256 => "sha256",
        }
    }

    /// The digest length in raw bytes.
    pub fn length_bytes(&self) -> usize {
        match self {
            DigestAlgorithm::Sha256 => 32,
        }
    }
}

impl fmt::Display for DigestAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Computes `HashV1(domain, payload) = SHA256("AF\0" || domain || "\0" || payload)`.
///
/// The caller supplies canonical ACE1 bytes as the payload; this function
/// performs the domain separation exactly as frozen, with no string-formatting
/// ambiguity.
pub fn hash_v1(domain: DigestDomain, payload: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(HASH_V1_PREFIX);
    hasher.update(domain.as_str().as_bytes());
    hasher.update([0u8]);
    hasher.update(payload);
    hasher.finalize().into()
}

/// A nominally typed, strictly validated digest identity.
///
/// A `DigestRef` never loses its domain and never degrades to a bare string.
/// The hex is lowercase, exactly 64 characters, `alg` is v1-fixed to `sha256`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DigestRef {
    alg: DigestAlgorithm,
    domain: DigestDomain,
    hex: String,
}

impl DigestRef {
    /// Computes the digest of canonical payload bytes under `domain`.
    pub fn compute(domain: DigestDomain, payload: &[u8]) -> Self {
        let raw = hash_v1(domain, payload);
        Self {
            alg: DigestAlgorithm::Sha256,
            domain,
            hex: to_lower_hex(&raw),
        }
    }

    /// Strictly validates and constructs a `DigestRef` from parts.
    ///
    /// The hex must be exactly 64 lowercase hexadecimal characters; the
    /// algorithm must be `sha256`; the domain must be in the frozen registry.
    pub fn from_parts(
        alg: DigestAlgorithm,
        domain: DigestDomain,
        hex: &str,
    ) -> Result<Self, DigestError> {
        if alg != DigestAlgorithm::Sha256 {
            return Err(DigestError::InvalidDigest {
                detail: format!("unsupported digest algorithm {alg:?}; v1 fixes sha256"),
            });
        }
        let bytes = hex.as_bytes();
        if bytes.len() != 64 {
            return Err(DigestError::InvalidDigest {
                detail: format!(
                    "digest hex must be exactly 64 characters, got {}",
                    bytes.len()
                ),
            });
        }
        if !bytes
            .iter()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(b))
        {
            return Err(DigestError::InvalidDigest {
                detail: "digest hex must be lowercase hexadecimal".into(),
            });
        }
        Ok(Self {
            alg,
            domain,
            hex: hex.to_string(),
        })
    }

    /// The digest algorithm (v1: always `sha256`).
    pub fn alg(&self) -> DigestAlgorithm {
        self.alg
    }

    /// The digest domain — never lost, never guessed.
    pub fn domain(&self) -> DigestDomain {
        self.domain
    }

    /// The lowercase 64-character hex digest.
    pub fn hex(&self) -> &str {
        &self.hex
    }

    /// The raw 32 digest bytes.
    pub fn raw_bytes(&self) -> [u8; 32] {
        let mut out = [0u8; 32];
        for (i, byte) in out.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&self.hex[2 * i..2 * i + 2], 16)
                .expect("hex validated at construction");
        }
        out
    }
}

impl fmt::Display for DigestRef {
    /// Human/debug convenience form `sha256/{domain}/{hex}`. This is not a
    /// normative wire format; parse it back with [`DigestRef::from_display`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}/{}",
            self.alg.as_str(),
            self.domain.as_str(),
            self.hex
        )
    }
}

impl DigestRef {
    /// Strictly parses the `sha256/{domain}/{hex}` convenience form.
    pub fn from_display(s: &str) -> Result<Self, DigestError> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 3 {
            return Err(DigestError::InvalidDigest {
                detail: "expected sha256/{domain}/{hex}".into(),
            });
        }
        let alg = match parts[0] {
            "sha256" => DigestAlgorithm::Sha256,
            other => {
                return Err(DigestError::InvalidDigest {
                    detail: format!("unknown digest algorithm {other:?}"),
                });
            }
        };
        let domain = DigestDomain::from_registry_name(parts[1])?;
        DigestRef::from_parts(alg, domain, parts[2])
    }
}

fn to_lower_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        write!(out, "{b:02x}").expect("String formatting is infallible");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formula_is_exact() {
        // Exact expected values are frozen in spec/conformance/digest/, which
        // was generated by an independent implementation (Python hashlib).
        // Here we assert the structural properties of the preimage formula.
        let a = hash_v1(DigestDomain::PlanCore, b"hello");
        let b = hash_v1(DigestDomain::PlanCore, b"hello");
        let c = hash_v1(DigestDomain::PlanCore, b"hellp");
        let d = hash_v1(DigestDomain::SemanticIr, b"hello");
        assert_eq!(a, b, "same domain + payload must be deterministic");
        assert_ne!(a, c, "payload must affect the digest");
        assert_ne!(a, d, "domain must affect the digest");
    }

    #[test]
    fn empty_payload_is_defined() {
        let a = hash_v1(DigestDomain::Value, b"");
        let b = hash_v1(DigestDomain::Value, b"");
        assert_eq!(a, b);
        assert_ne!(a, hash_v1(DigestDomain::Value, b"\0"));
    }

    #[test]
    fn domain_separation_changes_digest() {
        let payload = b"same payload";
        let mut seen = Vec::new();
        for domain in DigestDomain::ALL {
            let d = to_lower_hex(&hash_v1(*domain, payload));
            assert!(
                !seen.contains(&d),
                "digest collision across domains for {domain}"
            );
            seen.push(d);
        }
    }

    #[test]
    fn digest_ref_validation() {
        let computed = DigestRef::compute(DigestDomain::LockGraph, b"x");
        assert_eq!(computed.hex().len(), 64);
        assert_eq!(computed.alg(), DigestAlgorithm::Sha256);
        assert_eq!(computed.domain(), DigestDomain::LockGraph);

        // Round-trip via the convenience form.
        let reparsed = DigestRef::from_display(&computed.to_string()).unwrap();
        assert_eq!(reparsed, computed);

        // Strictness.
        assert!(
            DigestRef::from_parts(
                DigestAlgorithm::Sha256,
                DigestDomain::Value,
                &computed.hex().to_uppercase()
            )
            .is_err()
        );
        assert!(
            DigestRef::from_parts(DigestAlgorithm::Sha256, DigestDomain::Value, "deadbeef")
                .is_err()
        );
        assert!(
            DigestRef::from_parts(
                DigestAlgorithm::Sha256,
                DigestDomain::Value,
                &format!("{}0", computed.hex())
            )
            .is_err()
        );
        assert!(DigestRef::from_display("md5/value/deadbeef").is_err());
        assert!(DigestRef::from_display("sha256/not-a-domain/deadbeef").is_err());
        assert!(matches!(
            DigestDomain::from_registry_name("plan"),
            Err(DigestError::UnknownDigestDomain { .. })
        ));
    }
}
