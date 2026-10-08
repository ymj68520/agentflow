//! `af-flow` — the AgentFlow Static Information Flow Contract (Phase
//! 4A.4): the F dimension of `Computation<I, O, Φ, ε, R>`.
//!
//! ```text
//! Information Source
//!         ↓
//! Security Labels + Taint + Instruction Authority
//!         ↓
//! Static Influence Summary   (crates/af-flow, this crate)
//!         ↓
//! Information Flow Contract  (ACE1 bytes via agentflow.information.flow@1)
//!         ↓
//! Conservative Propagation   (may-influence union; LLM strong mixer)
//!         ↓
//! Flow Non-Relaxation Proof  (contract evolution, F dimension)
//! ```
//!
//! Provenance, trust, taint and information flow stay four distinct
//! concepts. The frozen security invariants hold structurally: unknown
//! provenance is not trusted, unknown taint is not the empty set, a
//! missing flow proof is not flow allowed, an authentic source is not a
//! trusted instruction, sanitized data is not endorsed data, persistence
//! is not a trust upgrade, and a declared flow contract is not runtime
//! authorization.
//!
//! Layering: `af-core` → `af-idl` / `af-semantic` → `af-effect` →
//! `af-flow` (the one-directional effect dependency exists only to reuse
//! the structured `EffectOperationRef` for external-effect sinks — never
//! a second operation-id vocabulary; referencing an effect is not being
//! granted it). No `af-ir`, `af-failure`, `af-resource` or `af-spec`
//! dependency, no I/O, no clock, no randomness, no network. All analysis
//! entry points are pure functions; catalogs and contracts are explicit
//! read-only parameters.
//!
//! Normative artifacts: `spec/idl/information-flow.afidl` (machine
//! schema), `spec/information-flow/information-flow.md` (semantics) and
//! the frozen corpus under `spec/conformance/information-flow/`.

pub mod canonical;
pub mod compatibility;
pub mod error;
pub mod model;
pub mod propagation;
pub mod universe;

pub use canonical::{canonicalize_information_flow_contract, check_flow, check_sink};
pub use compatibility::{FlowCompatibilityProof, UnprovenReason, flow_not_relaxed};
pub use error::{FlowError, kind};
pub use model::{
    ConfidentialityLabelRef, FlowConstraint, FlowKind, FlowSink, FlowSourceRef,
    InformationFlowContract, InstructionAuthority, LabelIdentity, SourceCategory,
    StaticInformationSummary, TaintRef,
};
pub use propagation::{
    FlowProof, merge_provenance_summaries, mix_model_visible_inputs, propagate_pure_expression,
    propagate_unknown_transformation, union_taints,
};
pub use universe::{
    EffectSinkUniverse, FlowCatalogs, InformationLabelUniverse, SinkClassification, check_label,
    nfc,
};
