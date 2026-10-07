//! `af-core` — AgentFlow v1 Phase 1 foundation: canonical encoding, digests.
//!
//! This crate implements the machine-deterministic base of AgentFlow v1:
//!
//! 1. **ACE1** ([`encode`], [`decode`]) — a deterministic, schema-directed,
//!    UTF-8 canonical encoding (strict JSON subset) for the AFIDL v1 machine
//!    primitives and containers. See [`shape::Shape`] for the type model and
//!    [`value::Value`] for the closed typed value model.
//! 2. **Primitive validators** ([`primitive`]) — strict canonical text
//!    parsers for machine integers.
//! 3. **HashV1** ([`digest::hash_v1`]) —
//!    `SHA256("AF\0" || domain || "\0" || payload)` with the frozen
//!    digest-domain registry.
//! 4. **DigestRef** ([`digest::DigestRef`]) — nominally typed, strictly
//!    validated digest identity.
//!
//! Design constraints (normative):
//!
//! - no hidden global state, no wall clock, no randomness, no locale, no
//!   platform-dependent output: identical typed inputs always produce
//!   byte-for-byte identical outputs;
//! - JSON numbers are never the canonical authority for machine numerics —
//!   they encode as canonical strings;
//! - `missing != null != unit != Option::None`; the typed encoder enforces
//!   which of these each schema position accepts;
//! - closed enums/variants fail closed on unknown values;
//! - no `Any`/`serde_json::Value` escape hatch exists in the normative model.
//!
//! What this crate deliberately does **not** contain: Semantic IR, Executable
//! Plan, LockGraph, RunManifest, Journal, Reducer, Runtime Commands, workers,
//! schedulers, or any workflow runtime — those are later phases.

pub mod base64url;
pub mod decimal;
pub mod decode;
pub mod digest;
pub mod encode;
pub mod error;
pub mod primitive;
pub mod shape;
pub mod value;

pub use decimal::Decimal;
pub use decode::decode;
pub use digest::{DigestAlgorithm, DigestDomain, DigestRef, hash_v1};
pub use encode::{encode, encode_to_string};
pub use error::{AceError, DigestError};
pub use shape::{CaseShape, EnumShape, FieldShape, RecordShape, Shape, VariantShape};
pub use value::{Duration, Float64, Timestamp, Value};
