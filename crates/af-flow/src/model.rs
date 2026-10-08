//! The typed information-flow model, mirroring
//! `spec/idl/information-flow.afidl` field-for-field.
//!
//! Provenance, trust, taint and information flow stay four distinct
//! concepts: no trust score, no single trusted flag. Confidentiality
//! restrictions and taints are structurally separate nominal types;
//! instruction authority is a closed one-directional scale that data
//! transformation can never elevate.
//!
//! Unknown is never empty: an unknown influence set is the explicit
//! `influence_unknown` Top, unknown external origin is an explicit source
//! category, and neither is dropped during canonicalization.

use af_effect::EffectOperationRef;
use serde::{Deserialize, Serialize};

/// One confidentiality restriction: data carrying it may not flow where
/// the restriction is not accepted. Nominal identity
/// `(catalog, name)`; there is NO implicit hierarchy between labels —
/// containment is set membership plus explicit catalog rules, never an
/// invented lattice.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfidentialityLabelRef {
    pub catalog: String,
    pub name: String,
}

impl ConfidentialityLabelRef {
    /// The nominal identity key in byte order (catalog, name).
    #[must_use]
    pub fn identity_key(&self) -> (&str, &str) {
        (&self.catalog, &self.name)
    }
}

/// Shared shape of the two label types: their nominal `(catalog, name)`
/// identity.
pub trait LabelIdentity {
    /// The nominal identity key.
    fn identity(&self) -> (&str, &str);
}

impl LabelIdentity for ConfidentialityLabelRef {
    fn identity(&self) -> (&str, &str) {
        self.identity_key()
    }
}

impl LabelIdentity for TaintRef {
    fn identity(&self) -> (&str, &str) {
        self.identity_key()
    }
}

/// One taint: an influence source that may have affected a value. The
/// integrity/influence dimension, structurally separate from
/// confidentiality. Nominal identity `(catalog, name)`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TaintRef {
    pub catalog: String,
    pub name: String,
}

impl TaintRef {
    /// The nominal identity key in byte order (catalog, name).
    #[must_use]
    pub fn identity_key(&self) -> (&str, &str) {
        (&self.catalog, &self.name)
    }
}

/// Who may issue instructions through a value, ordered from `none`
/// upward. Data transformation can never elevate authority: content
/// claiming to be a system instruction stays `external_data`, and an
/// authentic source is not an instruction authorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InstructionAuthority {
    None,
    ExternalData,
    DelegatedAgent,
    User,
    Workflow,
    Platform,
}

impl InstructionAuthority {
    /// The one-directional comparison: `self` may carry instructions of
    /// strength `other` or weaker. Authority is never inferred upward.
    #[must_use]
    pub fn carries(self, other: Self) -> bool {
        self >= other
    }

    /// The stable serde/AFIDL spelling used by the corpus.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            InstructionAuthority::None => "none",
            InstructionAuthority::ExternalData => "external_data",
            InstructionAuthority::DelegatedAgent => "delegated_agent",
            InstructionAuthority::User => "user",
            InstructionAuthority::Workflow => "workflow",
            InstructionAuthority::Platform => "platform",
        }
    }
}

/// Where a static influence summary believes data may come from. A closed
/// category set of typed logical references; no free text, no ambient
/// source.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum SourceCategory {
    WorkflowInput(u32),
    ResourceRequirement(u32),
    UnknownExternal,
}

/// One static source reference.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FlowSourceRef {
    pub category: SourceCategory,
}

/// What kind of flow one sink rule constrains. The two dimensions are
/// checked separately: no data leak does not imply no decision influence,
/// and vice versa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowKind {
    DataEgress,
    DecisionInfluence,
}

impl FlowKind {
    /// The stable serde/AFIDL spelling used by the corpus.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            FlowKind::DataEgress => "data_egress",
            FlowKind::DecisionInfluence => "decision_influence",
        }
    }
}

/// Where a flow may land: a contract output port or a structured
/// reference to a declared effect operation (reused from the effect
/// signature module — no second string operation id). Referencing an
/// effect is not being granted it. Ordering is by the structured
/// identity (catalog, then name).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum FlowSink {
    WorkflowOutput(u32),
    ExternalEffect(EffectOperationRef),
}

impl PartialOrd for FlowSink {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FlowSink {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (FlowSink::WorkflowOutput(a), FlowSink::WorkflowOutput(b)) => a.cmp(b),
            (FlowSink::WorkflowOutput(_), FlowSink::ExternalEffect(_)) => std::cmp::Ordering::Less,
            (FlowSink::ExternalEffect(_), FlowSink::WorkflowOutput(_)) => {
                std::cmp::Ordering::Greater
            }
            (FlowSink::ExternalEffect(a), FlowSink::ExternalEffect(b)) => {
                (&a.catalog, &a.name).cmp(&(&b.catalog, &b.name))
            }
        }
    }
}

/// One declared flow ceiling: influence of this kind may reach this sink.
/// A declaration — never an observation, never a policy approval.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowConstraint {
    pub kind: FlowKind,
    pub sink: FlowSink,
}

impl FlowConstraint {
    /// The canonical comparison key: (kind, sink coordinate). Effect
    /// sinks compare by their full structured identity.
    #[must_use]
    pub fn sort_key(&self) -> (FlowKind, &FlowSink) {
        (self.kind, &self.sink)
    }
}

/// The conservative static influence summary of one value or contract:
/// sources that may have influenced it, taints it may carry, the
/// instruction authority it may transmit, and the explicit
/// `influence_unknown` Top flag.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StaticInformationSummary {
    pub sources: Vec<FlowSourceRef>,
    pub taints: Vec<TaintRef>,
    pub instruction_authority: InstructionAuthority,
    pub influence_unknown: bool,
}

/// The closed static Information Flow Contract F of one contract.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InformationFlowContract {
    pub confidentiality_restrictions: Vec<ConfidentialityLabelRef>,
    pub summary: StaticInformationSummary,
    pub flow_constraints: Vec<FlowConstraint>,
}

impl InformationFlowContract {
    /// The empty contract: no declared restriction, the conservative
    /// unknown summary, no declared ceiling. Empty is NOT trusted — the
    /// summary still says influence unknown.
    #[must_use]
    pub fn empty() -> Self {
        InformationFlowContract {
            confidentiality_restrictions: Vec::new(),
            summary: StaticInformationSummary {
                sources: Vec::new(),
                taints: Vec::new(),
                instruction_authority: InstructionAuthority::None,
                influence_unknown: true,
            },
            flow_constraints: Vec::new(),
        }
    }
}
