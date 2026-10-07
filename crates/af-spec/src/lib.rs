//! `af-spec` — the AgentFlow specification registry layer.
//!
//! Phase 2B foundation: the stable symbol system of the AgentFlow
//! specification itself. Every long-term ABI — schema, protocol, opcode,
//! event, command, profile, rule, field number, variant number, enum value
//! number — is registered here instead of being named ad hoc in code or
//! Markdown.
//!
//! What lives here:
//!
//! - [`model`] — typed models of the registry JSON instances
//!   (`agentflow.spec.registry@1` in `spec/idl/specification-registry.afidl`
//!   is the normative shape authority);
//! - [`tables`] — the rule-area closed table and implementation-drift checks
//!   for the pre-existing closed tables (hash domains, AFIDL primitives);
//! - [`abi`] — extraction of numbered schema members and the
//!   `check_current_schema_against_abi_registry` consistency check
//!   (number-never-reused enforcement);
//! - [`check`] — the cross-reference validator driving `spec-check`.
//!
//! Layering: `af-core ← af-idl ← af-spec`. This crate is spec tooling; it
//! contains no runtime semantics (no Semantic IR, Plan, Journal, Runtime
//! Command, ...).

pub mod abi;
pub mod anchors;
pub mod check;
pub mod error;
pub mod fs;
pub mod model;
pub mod tables;

pub use check::{Layout, Report, SpecCheck};
pub use error::SpecError;
