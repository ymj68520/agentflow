//! Canonical ordering and the canonical ACE1 bytes of a flow contract.
//!
//! Canonicalization resolves every declared label and every effect sink
//! against the explicit catalogs, NFC-normalizes label text, rejects
//! duplicates, and sorts: restrictions and taints by nominal identity
//! bytes, sources by their typed coordinate, constraints by (kind, sink
//! coordinate). Declaration order never reaches the canonical bytes;
//! `influence_unknown` and the `unknown_external` source are explicit
//! values that are NEVER silently dropped or repaired.
//!
//! The canonical form has ACE1 bytes but deliberately NO digest: no
//! InformationFlowDigest, no ProvenanceDigest, no TaintDigest exists and
//! no HashV1 domain is added; the future contract layer hashes as a whole
//! under the frozen `component-contract` domain.

use crate::error::{FlowError, kind};
use crate::model::{
    FlowKind, FlowSink, InformationFlowContract, SourceCategory, StaticInformationSummary,
};
use crate::propagation::FlowProof;
use crate::universe::{FlowCatalogs, SinkClassification, nfc};
use af_core::Value;
use std::collections::BTreeSet;

/// A summary is structurally valid as-is; a check hook kept separate so
/// the canonicalization of the surrounding contract stays readable.
fn check_summary(summary: &StaticInformationSummary) -> Result<(), FlowError> {
    for source in &summary.sources {
        match source.category {
            SourceCategory::WorkflowInput(port) | SourceCategory::ResourceRequirement(port) => {
                if port == 0 {
                    return Err(FlowError::new(
                        kind::INVALID_FLOW_SOURCE,
                        "source port 0 is invalid: ports are positive",
                    ));
                }
            }
            SourceCategory::UnknownExternal => {}
        }
    }
    Ok(())
}

/// `canonicalize_information_flow_contract`: resolve against the explicit
/// catalogs (unknown labels are errors, never dropped), normalize,
/// deduplicate, sort. Duplicate (kind, sink) constraint pairs are
/// rejected — a declared ceiling list is a set.
///
/// # Errors
/// [`FlowError`] with kinds `invalid_information_label`,
/// `unknown_information_label`, `invalid_flow_source`, `invalid_flow_sink`,
/// `unknown_flow_sink` and `duplicate_flow_constraint`.
pub fn canonicalize_information_flow_contract(
    contract: &InformationFlowContract,
    catalogs: &FlowCatalogs,
) -> Result<InformationFlowContract, FlowError> {
    // Confidentiality restrictions: syntax + universe, NFC-normalized,
    // unique, sorted by nominal identity.
    let mut seen_restrictions = BTreeSet::new();
    let mut restrictions = Vec::new();
    for label in &contract.confidentiality_restrictions {
        catalogs.labels.resolve_confidentiality(label)?;
        let key = (label.catalog.clone(), nfc(&label.name));
        if !seen_restrictions.insert(key.clone()) {
            return Err(FlowError::new(
                kind::DUPLICATE_INFORMATION_LABEL,
                format!(
                    "confidentiality label {}/{} is declared twice (NFC-                     equal spellings are the same label): a declared list                      is a set",
                    key.0, key.1
                ),
            ));
        }
        restrictions.push(key);
    }
    restrictions.sort();
    // Summary: source ports positive; taints resolved against the
    // universe.
    check_summary(&contract.summary)?;
    let mut seen_taints = BTreeSet::new();
    let mut taints = Vec::new();
    for taint in &contract.summary.taints {
        catalogs.labels.resolve_taint(taint)?;
        let key = (taint.catalog.clone(), nfc(&taint.name));
        if !seen_taints.insert(key.clone()) {
            return Err(FlowError::new(
                kind::DUPLICATE_INFORMATION_LABEL,
                format!(
                    "taint {}/{} is declared twice (NFC-equal spellings                      are the same label): a declared list is a set",
                    key.0, key.1
                ),
            ));
        }
        taints.push(key);
    }
    taints.sort();
    // Constraints: syntax + sink resolution, unique (kind, sink), sorted
    // by (kind, sink).
    let mut seen_constraints = BTreeSet::new();
    let mut constraints = Vec::with_capacity(contract.flow_constraints.len());
    for constraint in &contract.flow_constraints {
        check_sink(&constraint.sink, catalogs)?;
        let key = sink_sort_key(&constraint.sink);
        if !seen_constraints.insert((constraint.kind, key.clone())) {
            return Err(FlowError::new(
                kind::DUPLICATE_FLOW_CONSTRAINT,
                "the same (flow kind, sink) ceiling is declared twice: a \
                 constraint list is a set",
            ));
        }
        constraints.push(constraint.clone());
    }
    constraints
        .sort_by(|a, b| (a.kind, sink_sort_key(&a.sink)).cmp(&(b.kind, sink_sort_key(&b.sink))));
    Ok(InformationFlowContract {
        confidentiality_restrictions: restrictions
            .into_iter()
            .map(|(catalog, name)| crate::model::ConfidentialityLabelRef { catalog, name })
            .collect(),
        summary: StaticInformationSummary {
            sources: {
                let mut sources = contract.summary.sources.clone();
                sources.sort();
                sources
            },
            taints: taints
                .into_iter()
                .map(|(catalog, name)| crate::model::TaintRef { catalog, name })
                .collect(),
            instruction_authority: contract.summary.instruction_authority,
            influence_unknown: contract.summary.influence_unknown,
        },
        flow_constraints: constraints,
    })
}

/// Syntax + catalog check of one sink reference. Sink classification is
/// NEVER inferred from the operation name; an unresolvable operation is
/// `unknown_flow_sink`, fail-closed.
///
/// # Errors
/// [`FlowError`] with kinds `invalid_flow_sink` and `unknown_flow_sink`.
pub fn check_sink(sink: &FlowSink, catalogs: &FlowCatalogs) -> Result<(), FlowError> {
    match sink {
        FlowSink::WorkflowOutput(port) => {
            if *port == 0 {
                return Err(FlowError::new(
                    kind::INVALID_FLOW_SINK,
                    "output port 0 is invalid: ports are positive",
                ));
            }
        }
        FlowSink::ExternalEffect(operation) => {
            if operation.catalog.is_empty() || operation.name.is_empty() {
                return Err(FlowError::new(
                    kind::INVALID_FLOW_SINK,
                    "effect sink identity is empty",
                ));
            }
            if !catalogs.sinks.resolves(&operation.catalog, &operation.name) {
                return Err(FlowError::new(
                    kind::UNKNOWN_FLOW_SINK,
                    format!(
                        "effect sink {}/{} does not resolve in the supplied \
                         catalog universe (no latest fallback, no name-based \
                         classification)",
                        operation.catalog, operation.name
                    ),
                ));
            }
        }
    }
    Ok(())
}

fn sink_sort_key(sink: &FlowSink) -> String {
    match sink {
        FlowSink::WorkflowOutput(port) => format!("out:{port}"),
        FlowSink::ExternalEffect(operation) => {
            format!("effect:{}/{}", operation.catalog, operation.name)
        }
    }
}

/// The static flow judgment for one declared (kind, sink) pair of a
/// catalog-valid contract. The verdict semantics:
///
/// 1. no matching constraint → `flow_not_proven`: a MISSING flow proof
///    is never flow allowed;
/// 2. `influence_unknown` → [`FlowProof::Unproven`] (unknown is never
///    read as clean);
/// 3. `data_egress` while the contract declares confidentiality
///    restrictions → the egress carries a declassification obligation;
/// 4. `decision_influence` while the summary carries any taint → the
///    decision carries an endorsement obligation; an external-effect
///    decision sink that the catalog does not classify as
///    decision-capable is [`FlowProof::Unproven`] — classification is
///    never inferred from the operation name.
///
/// # Errors
/// [`FlowError`] with the kinds of
/// [`canonicalize_information_flow_contract`] and `flow_not_proven`.
pub fn check_flow(
    contract: &InformationFlowContract,
    flow_kind: FlowKind,
    sink: &FlowSink,
    catalogs: &FlowCatalogs,
) -> Result<FlowProof, FlowError> {
    let canonical = canonicalize_information_flow_contract(contract, catalogs)?;
    if !canonical
        .flow_constraints
        .iter()
        .any(|constraint| constraint.kind == flow_kind && &constraint.sink == sink)
    {
        return Err(FlowError::new(
            kind::FLOW_NOT_PROVEN,
            "no declared ceiling covers this (flow kind, sink): a missing \
             flow proof is never flow allowed",
        ));
    }
    if canonical.summary.influence_unknown {
        return Ok(FlowProof::Unproven);
    }
    let requires_declassification =
        flow_kind == FlowKind::DataEgress && !canonical.confidentiality_restrictions.is_empty();
    if flow_kind == FlowKind::DecisionInfluence {
        if let FlowSink::ExternalEffect(operation) = sink {
            if catalogs.sink_classification(&operation.catalog, &operation.name)
                != SinkClassification::DecisionCapable
            {
                return Ok(FlowProof::Unproven);
            }
        }
        let requires_endorsement = !canonical.summary.taints.is_empty();
        if requires_endorsement {
            return Ok(FlowProof::RequiresTrustedTransformation {
                requires_declassification: false,
                requires_endorsement: true,
            });
        }
    }
    if requires_declassification {
        return Ok(FlowProof::RequiresTrustedTransformation {
            requires_declassification: true,
            requires_endorsement: false,
        });
    }
    Ok(FlowProof::ProvenWithinDeclaredCeiling)
}

impl InformationFlowContract {
    /// The canonical form: catalogs resolved, duplicates rejected,
    /// deterministic order.
    ///
    /// # Errors
    /// [`FlowError`] with the kinds of
    /// [`canonicalize_information_flow_contract`].
    pub fn canonical(&self, catalogs: &FlowCatalogs) -> Result<InformationFlowContract, FlowError> {
        canonicalize_information_flow_contract(self, catalogs)
    }

    /// The canonical ACE1 bytes, encoded against the schema-derived shape
    /// of `agentflow.information.flow@1/InformationFlowContract` within
    /// `set` (which must contain the flow module, the semantic core and
    /// the effect signature module it imports). Requires a structurally
    /// valid, catalog-resolvable contract.
    ///
    /// # Errors
    /// [`FlowError`] with kind `flow_encoding_error` when the canonical
    /// form is invalid or the value tree cannot be encoded.
    pub fn canonical_bytes(
        &self,
        catalogs: &FlowCatalogs,
        set: &af_idl::CompiledModuleSet,
    ) -> Result<Vec<u8>, FlowError> {
        let value = self.to_value(catalogs)?;
        af_core::encode(&value_shape(set)?, &value).map_err(|error| {
            FlowError::new(
                kind::FLOW_ENCODING_ERROR,
                format!("cannot encode the canonical flow contract: {error}"),
            )
        })
    }

    /// The ACE1 value tree of the canonical contract (mirrors the AFIDL
    /// schema field-for-field; the encoder emits members in byte order).
    ///
    /// # Errors
    /// [`FlowError`] when the contract is not catalog-valid.
    pub fn to_value(&self, catalogs: &FlowCatalogs) -> Result<Value, FlowError> {
        let canonical = canonicalize_information_flow_contract(self, catalogs)?;
        Ok(Value::Record(vec![
            (
                "confidentiality_restrictions".into(),
                Value::List(
                    canonical
                        .confidentiality_restrictions
                        .iter()
                        .map(label_ref_value)
                        .collect(),
                ),
            ),
            ("summary".into(), summary_value(&canonical.summary)),
            (
                "flow_constraints".into(),
                Value::List(
                    canonical
                        .flow_constraints
                        .iter()
                        .map(|constraint| {
                            Value::Record(vec![
                                (
                                    "kind".into(),
                                    Value::Enum(constraint.kind.as_str().to_string()),
                                ),
                                ("sink".into(), sink_value(&constraint.sink)),
                            ])
                        })
                        .collect(),
                ),
            ),
        ]))
    }
}

fn label_ref_value(label: &crate::model::ConfidentialityLabelRef) -> Value {
    Value::Record(vec![
        ("catalog".into(), Value::String(label.catalog.clone())),
        ("name".into(), Value::String(label.name.clone())),
    ])
}

fn taint_ref_value(taint: &crate::model::TaintRef) -> Value {
    Value::Record(vec![
        ("catalog".into(), Value::String(taint.catalog.clone())),
        ("name".into(), Value::String(taint.name.clone())),
    ])
}

fn summary_value(summary: &StaticInformationSummary) -> Value {
    Value::Record(vec![
        (
            "sources".into(),
            Value::List(
                summary
                    .sources
                    .iter()
                    .map(|source| {
                        Value::Record(vec![(
                            "category".into(),
                            match &source.category {
                                SourceCategory::WorkflowInput(port) => Value::Variant {
                                    case: "workflow_input".into(),
                                    value: Box::new(Value::UInt32(*port)),
                                },
                                SourceCategory::ResourceRequirement(port) => Value::Variant {
                                    case: "resource_requirement".into(),
                                    value: Box::new(Value::UInt32(*port)),
                                },
                                SourceCategory::UnknownExternal => Value::Variant {
                                    case: "unknown_external".into(),
                                    value: Box::new(Value::Unit),
                                },
                            },
                        )])
                    })
                    .collect(),
            ),
        ),
        (
            "taints".into(),
            Value::List(summary.taints.iter().map(taint_ref_value).collect()),
        ),
        (
            "instruction_authority".into(),
            Value::Enum(summary.instruction_authority.as_str().to_string()),
        ),
        (
            "influence_unknown".into(),
            Value::Bool(summary.influence_unknown),
        ),
    ])
}

fn sink_value(sink: &FlowSink) -> Value {
    match sink {
        FlowSink::WorkflowOutput(port) => Value::Variant {
            case: "workflow_output".into(),
            value: Box::new(Value::UInt32(*port)),
        },
        FlowSink::ExternalEffect(operation) => Value::Variant {
            case: "external_effect".into(),
            value: Box::new(Value::Record(vec![
                ("catalog".into(), Value::String(operation.catalog.clone())),
                ("name".into(), Value::String(operation.name.clone())),
            ])),
        },
    }
}

fn value_shape(set: &af_idl::CompiledModuleSet) -> Result<af_core::Shape, FlowError> {
    let shape = set
        .shape_of("agentflow.information.flow@1/InformationFlowContract")
        .map_err(|error| {
            FlowError::new(
                kind::FLOW_ENCODING_ERROR,
                format!("cannot derive the flow-contract shape: {error}"),
            )
        })?;
    shape.validate().map_err(|error| {
        FlowError::new(
            kind::FLOW_ENCODING_ERROR,
            format!("derived shape is invalid: {error}"),
        )
    })?;
    Ok(shape)
}
