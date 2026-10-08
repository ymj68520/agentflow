//! `af-effect` — the AgentFlow Effect Signature (Phase 4A.1): the static
//! DECLARATION side of the ε dimension of
//! `Computation<I, O, Φ, ε, R>`.
//!
//! ```text
//! Effect Declaration
//!     ↓
//! Typed Effect Signature   (crates/af-effect, this crate)
//!     ↓
//! Canonical Effect Row     (ACE1 bytes via agentflow.effect.signature@1)
//!     ↓
//! Effect Algebra           (⊑, ⊕, ⊔, n⊗ε — pure functions)
//!     ↓
//! Static Effect Envelope   (control-opcode composition)
//! ```
//!
//! An effect signature is a static capability CEILING, not an authorization:
//!
//! ```text
//! Declared Effects ≠ Granted Effects ≠ Dispatched Effects ≠ Observed Effects
//! Dispatched ⊆ Granted ⊆ Declared   (Runtime / Effect Gateway invariant)
//! ```
//!
//! In particular:
//!
//! - unknown effects are never pure; the unknown row (⊤) is an analysis
//!   result, never a grant;
//! - a bounded-open row needs an explicit, verifiable ceiling;
//! - compensation never subtracts past effects;
//! - there is no credential, provider endpoint or execution capability
//!   anywhere in the model, and no API converts a signature into authority.
//!
//! Layering (dependencies point upward only): `af-core` → `af-idl` /
//! `af-semantic` → `af-effect`. The crate has no I/O, no clock, no
//! randomness, no network, no async runtime and no `af-ir` dependency.
//!
//! Normative artifacts: `spec/idl/effect-signature.afidl` (machine schema),
//! `spec/effect/effect-signature.md` (semantics) and the frozen corpus under
//! `spec/conformance/effect-signature/`.

pub mod algebra;
pub mod canonical;
pub mod catalog;
pub mod control;
pub mod error;
pub mod model;

pub use algebra::{
    Proof, UnprovenReason, accumulate, branch_join, contains, guard_contained, repeat,
};
pub use catalog::{CapabilityCatalog, EffectCatalogUniverse};
pub use control::{
    RETRY_NARROWING_OBLIGATION, RetryEnvelope, compensation_envelope, iterate_envelope,
    parallel_envelope, race_envelope, retry_envelope, select_envelope, sequence_envelope,
};
pub use error::{EffectError, kind};
pub use model::{
    AtomKey, EffectAtom, EffectEntry, EffectGuard, EffectGuardInputEquals,
    EffectIdempotencyQualifier, EffectOpenRow, EffectOperationRef, EffectQualifier,
    EffectQualifierIdentity, EffectResidual, EffectRow, EffectSignature, OccurrenceBounds,
    OccurrenceCeiling, ResourceSelector, ScopeBinding,
};
