//! `af-usage` — the static Usage / cost-envelope declaration layer.
//!
//! ```text
//! Usage Meter Identity + Explicit Universe
//!              ↓
//! Static Usage Signature (finite / unbounded)
//!              ↓
//! Canonical ACE1 (no UsageSignatureDigest)
//!              ↓
//! Usage Algebra + Conservative Control Propagation
//!              ↓
//! Static Ceiling / Non-Increase Proof
//! ```
//!
//! This crate has no measurement, billing, pricing, reservation, settlement,
//! quota, scheduler, runtime or ledger surface. A missing meter is never
//! treated as zero, and `Unknown` / `Unproven` never becomes `Proven`.

pub mod algebra;
pub mod budget;
pub mod canonical;
pub mod compatibility;
pub mod control;
pub mod cost;
pub mod cost_canonical;
pub mod cost_error;
pub mod cost_model;
pub mod currency;
pub mod error;
pub mod model;
pub mod universe;
pub mod validate;

pub use algebra::{
    UsageProof, UsageUnprovenReason, accumulate, branch_join, repeat, within_ceiling,
};
pub use canonical::{canonicalize_usage_signature, declared_usage_envelope};
pub use compatibility::usage_not_increased;
pub use control::{
    RETRY_USAGE_NARROWING_OBLIGATION, RetryUsageEnvelope, await_envelope, call_subflow_envelope,
    compensation_envelope, handle_envelope, invoke_envelope, iterate_envelope, parallel_envelope,
    race_envelope, retry_envelope, scope_envelope, select_envelope, sequence_envelope,
    timeout_envelope,
};
pub use cost::{
    CostProof, CostUnprovenReason, check_static_budget, cost_not_increased, project_cost,
};
pub use cost_canonical::{canonicalize_price_table_snapshot, canonicalize_static_cost_ceiling};
pub use cost_error::{CostError, cost_kind};
pub use cost_model::{
    MoneyAmount, PriceEntry, PriceRate, PriceTableSnapshot, PricingModel, RoundingRule,
    StaticCostBound, StaticCostCeiling, StaticCostProjection,
};
pub use currency::{CurrencyDescriptor, CurrencyRef, CurrencyUniverse, MeasurementUnitRef};
pub use error::{UsageError, kind};
pub use model::{
    StaticUsageBound, StaticUsageEnvelope, UsageCoordinate, UsageLimit, UsageMeterRef, UsagePhase,
    UsageSignature, UsageUpperBound,
};
pub use universe::{UsageAggregation, UsageMeterDescriptor, UsageMeterUniverse, UsageUnitRef};
