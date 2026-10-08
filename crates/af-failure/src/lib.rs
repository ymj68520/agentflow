//! `af-failure` — the AgentFlow Static Typed Failure Row (Phase 4A.2): the
//! Φ dimension of `Computation<I, O, Φ, ε, R>`.
//!
//! ```text
//! Failure Type Identity (nominal)
//!         ↓
//! Nominal Failure Row     (crates/af-failure, this crate)
//!         ↓
//! Canonical Failure Set   (ACE1 bytes via agentflow.failure.row@1)
//!         ↓
//! Failure Algebra         (⊆, ∪, subtraction, handler propagation)
//!         ↓
//! Static Failure Propagation (control opcodes)
//! ```
//!
//! A failure type is identified nominally —
//! `(SpecificationId, TypeName, pinned TypeSchemaDigest)` — and the row is
//! a CLOSED set. The six error planes are strictly separate; only typed
//! business failures enter Φ:
//!
//! ```text
//! Typed FailureValue → Φ          TimedOut / Cancelled → Control Outcome
//! RecoveryBlocked → Recovery State  TaskAttemptLost → Infrastructure Fact
//! RuntimeIncident → Incident Plane
//! ```
//!
//! `Φ = ∅` does not mean the computation succeeds, and an unknown failure
//! upper bound is the analysis-level [`control::StaticFailureEnvelope::
//! Unproven`] — never the empty row, and never a wire state.
//!
//! Layering (dependencies point upward only): `af-core` → `af-idl` /
//! `af-semantic` → `af-failure`. The crate has no `af-ir`, `af-effect` or
//! `af-spec` dependency, no I/O, no clock, no randomness, no network and no
//! async runtime. The Φ algebra and the ε algebra (`af-effect`) are
//! deliberately parallel and independent; the future ComponentContract
//! aggregates them.
//!
//! Normative artifacts: `spec/idl/failure-row.afidl` (machine schema),
//! `spec/failure/failure-row.md` (semantics) and the frozen corpus under
//! `spec/conformance/failure-row/`.

pub mod algebra;
pub mod canonical;
pub mod control;
pub mod error;
pub mod model;
pub mod validate;

pub use algebra::{contains, join, propagate_handler_failures, remove_handled, union};
pub use canonical::nominal_sort_key;
pub use control::{
    StaticFailureEnvelope, await_row, compensation_scope_row, handle_row, iterate_row,
    parallel_row, race_row, retry_row, scope_row, select_row, sequence_row, timeout_row,
};
pub use error::{FailureError, kind};
pub use model::{FailureRow, FailureTypeRef};
