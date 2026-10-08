//! `af-resource` — the AgentFlow Static Resource Requirement Row (Phase
//! 4A.3): the R dimension of `Computation<I, O, Φ, ε, R>`.
//!
//! ```text
//! Resource Type Identity (nominal)
//!         ↓
//! Resource Requirement    (crates/af-resource, this crate)
//!         ↓
//! Closed Requirement Row  (ACE1 bytes via agentflow.resource.requirement@1)
//!         ↓
//! Static Satisfaction     (candidate descriptor ⇸ requirement)
//!         ↓
//! Non-Strengthening Proof (contract evolution, R dimension)
//! ```
//!
//! R answers: which logical resources, under which statically provable
//! exact-value conditions, does this computation need from the outside to
//! execute at all. It does NOT answer what a Run obtained: no binding, no
//! handle, no adapter, no credential, no runtime authority. The
//! declaration chain `ResourceType → ResourceSlot → ResourceBinding →
//! Logical Resource Identity → ResourceHandle → ResourceAdapter →
//! Physical Resource` is frozen architecture; only the static head is
//! materialized here.
//!
//! Three authority concepts stay separate: a resource REQUIREMENT (what a
//! component needs), a capability ceiling (what it may touch) and an
//! effect grant (which exact action may run). Requirement satisfaction
//! never implies authorization.
//!
//! Layering (dependencies point upward only): `af-core` → `af-idl` /
//! `af-semantic` → `af-resource`. The crate has no `af-ir`, `af-effect`,
//! `af-failure` or `af-spec` dependency, no I/O, no clock, no randomness,
//! no network and no async runtime. F, Φ, ε and R stay independently
//! materialized; the future contract layer composes them.
//!
//! Normative artifacts: `spec/idl/resource-requirement.afidl` (machine
//! schema), `spec/resource/resource-requirement.md` (semantics) and the
//! frozen corpus under `spec/conformance/resource-requirement/`.

pub mod canonical;
pub mod compatibility;
pub mod error;
pub mod model;
pub mod universe;
pub mod validate;

pub use canonical::{
    canonical_attributes, canonicalize_resource_requirement_row, check_identity, nfc,
};
pub use compatibility::{
    ResourceCandidate, ResourceCompatibilityProof, UnprovenReason, requirement_not_stronger,
    satisfies_resource_requirement,
};
pub use error::{ResourceError, kind};
pub use model::{
    ResourceAttributeConstraint, ResourceRequirement, ResourceRequirementRow, ResourceTypeRef,
};
pub use universe::{ResourceTypeDescriptor, ResourceTypeUniverse};
