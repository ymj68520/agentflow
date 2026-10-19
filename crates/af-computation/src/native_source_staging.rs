//! Analysis9 registration metadata ONLY. Not an input binder or capability issuer.
use crate::{
    CompilerSemanticsIdentity, EditionRegistration, RegisteredAnalysisEdition,
    RegisteredModelFailure,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::sync::OnceLock;

pub const ANALYSIS: &str = "agentflow.root-export-checker@9";
pub const MODEL: &str = "agentflow.computation.proof-report.model@8";
pub const REPORT: &str = "agentflow.g01.report-shape@8";
pub const OBLIGATION: &str = "agentflow.g01.obligation@8";
const DELTA: &str = include_str!("../../../spec/tables/proof-report-model-v8.md");
const BASE: &[u8] = include_bytes!("../../../spec/tables/proof-report-model-v7.md");
const EXTRA: &[(&str, &str, &[u8])] = &[
    (
        "agentflow.computation.native-relation-source-binding.semantics@1",
        "spec/computation/native-relation-source-binding.md",
        include_bytes!("../../../spec/computation/native-relation-source-binding.md"),
    ),
    (
        "agentflow.computation.proof-obligation.semantics@8",
        "spec/computation/proof-obligation-semantics-v8.md",
        include_bytes!("../../../spec/computation/proof-obligation-semantics-v8.md"),
    ),
    (
        MODEL,
        "spec/tables/proof-report-model-v8.md",
        include_bytes!("../../../spec/tables/proof-report-model-v8.md"),
    ),
    (
        "agentflow.portable.proof.inputs.semantics@9",
        "spec/portable/portable-proof-inputs-v9.md",
        include_bytes!("../../../spec/portable/portable-proof-inputs-v9.md"),
    ),
];

/// Expands exact normative source contents. No input values, ranks or receipts.
pub fn native_source_model_catalogue() -> Result<&'static Value, RegisteredModelFailure> {
    static M: OnceLock<Result<Value, RegisteredModelFailure>> = OnceLock::new();
    M.get_or_init(|| {
        let fail = RegisteredModelFailure::IncompleteContent;
        let body = DELTA.split_once("```json\n").ok_or(fail)?.1.split_once("\n```").ok_or(fail)?.0;
        let d: Value = serde_json::from_str(body).map_err(|_| fail)?;
        let expected = ["format", "base_source", "edition", "type_replacements", "registration", "source_contract", "canonical_constraints", "no_new_type_ranks", "no_new_node_body", "production_binder", "production_dagranks_generated", "raw_full_field", "root_export"];
        if d.as_object().is_none_or(|o| o.keys().map(String::as_str).collect::<BTreeSet<_>>() != BTreeSet::from(expected))
            || d["format"] != "closed_native_source_delta_v1"
            || d["base_source"] != json!({"specification_id":"agentflow.computation.proof-report.model@7","logical_path":"spec/tables/proof-report-model-v7.md","plain_sha256":hex::encode(Sha256::digest(BASE))})
            || d["edition"] != json!({"analysis":ANALYSIS,"model":MODEL,"report":REPORT,"obligation":OBLIGATION}) {
            return Err(fail);
        }
        let base = crate::structural_model_catalogue()?;
        let mut m = base.clone();
        let replacements = d["type_replacements"].as_object().ok_or(fail)?;
        if replacements.keys().map(String::as_str).collect::<BTreeSet<_>>() != BTreeSet::from(["ContractProofContextContents", "StructuralValue0030", "StructuralValue0031", "StructuralValue0032", "StructuralValue0033"]) {
            return Err(fail);
        }
        m["types"].as_object_mut().ok_or(fail)?.extend(replacements.clone());
        m["model_id"] = MODEL.into();m["analysis_owner"] = ANALYSIS.into();
        m["model_edition"] = REPORT.into();m["obligation_edition"] = OBLIGATION.into();
        m["structural_binding_registration"] = d["registration"].clone();
        m["canonical_constraints"] = d["canonical_constraints"].clone();
        m["inherited_registered_model"] = base.clone();
        if m["type_ranks"] != base["type_ranks"] || m["types"]["NodeBody"] != base["types"]["NodeBody"] {return Err(fail);}
        Ok(m)
    }).as_ref().map_err(|e| *e)
}

/// Explicit successor registration identity; legacy `current_*` APIs remain
/// exact Analysis8 compatibility APIs. No caller flag can change these states.
#[must_use]
pub fn native_source_registered_analysis_edition() -> RegisteredAnalysisEdition {
    let mut e = crate::current_registered_analysis_edition();
    e.compiler.analysis_semantics_version = ANALYSIS.into();
    e.compiler.registry_content =
        include_bytes!("../../../spec/registry/specifications.json").to_vec();
    e.compiler.rule_registry_content = include_bytes!("../../../spec/registry/rules.json").to_vec();
    for &(id, path, content) in EXTRA {
        let source = crate::SpecificationContentIdentity {
            specification_id: id.into(),
            logical_path: path.into(),
            content_sha256: hex::encode(Sha256::digest(content)),
            content: content.to_vec(),
        };
        e.compiler.specifications.push(source.clone());
        e.registered_sources.push(source);
    }
    e.compiler
        .specifications
        .sort_by(|a, b| a.specification_id.cmp(&b.specification_id));
    e.registered_sources
        .sort_by(|a, b| a.specification_id.cmp(&b.specification_id));
    let rule_registry: Value =
        serde_json::from_slice(&e.compiler.rule_registry_content).unwrap_or(Value::Null);
    for row in rule_registry["rules"].as_array().into_iter().flatten() {
        if row["rule_id"]
            .as_str()
            .is_some_and(|id| (315..=323).any(|n| id == format!("AF-SEMANTIC-{n:04}")))
        {
            e.compiler.rules.push(crate::RuleContentIdentity {
                rule_id: row["rule_id"].as_str().unwrap_or("").into(),
                normative_source: row["normative_source"].as_str().unwrap_or("").into(),
                source_anchor: row["source_anchor"].as_str().unwrap_or("").into(),
                short_statement: row["short_statement"].as_str().unwrap_or("").into(),
            });
        }
    }
    e.compiler.rules.sort_by(|a, b| a.rule_id.cmp(&b.rule_id));
    let specs: Value = serde_json::from_slice(&e.compiler.registry_content).unwrap_or(Value::Null);
    let complete = specs["specifications"].as_array().is_some_and(|rows| {
        rows.len() == 74
            && rows.iter().all(|r| {
                e.registered_sources.iter().any(|s| {
                    r["specification_id"] == s.specification_id
                        && r["normative_source"]["logical_path"] == s.logical_path
                        && r["normative_source"]["content_digest"]["hex"] == s.content_sha256
                })
            })
    });
    e.registration =
        if complete && e.compiler.rules.len() == 226 && native_source_model_catalogue().is_ok() {
            EditionRegistration::Registered
        } else {
            EditionRegistration::Unregistered
        };
    e.report_shape = REPORT;
    e.obligation_edition = OBLIGATION;
    e
}

pub fn check_native_source_registration_identity(
    input: &CompilerSemanticsIdentity,
) -> Result<(), RegisteredModelFailure> {
    if input.analysis_semantics_version != ANALYSIS {
        return Err(RegisteredModelFailure::UnsupportedModel);
    }
    if input != &native_source_registered_analysis_edition().compiler {
        return Err(RegisteredModelFailure::InvalidFacts);
    }
    Ok(())
}

/// Bounded, non-authoritative inspection of registered declarations only.
/// Success returns data, never VerifiedInputGraph/CompleteView/ObligationIdentity.
pub fn inspect_native_source_registration(input: &Value) -> Result<Value, RegisteredModelFailure> {
    if input["edition"]
        != json!({"analysis":ANALYSIS,"model":MODEL,"report":REPORT,"obligation":OBLIGATION})
    {
        return Err(RegisteredModelFailure::UnsupportedModel);
    }
    let m = native_source_model_catalogue()?;
    let r = &m["structural_binding_registration"];
    let value = match input["op"].as_str() {
        Some("source") => {
            let family = match input["slot"].as_str() {
                Some("effect_resource") => "agentflow.effect.resource.compatibility@1",
                Some("recovery_usage") => "agentflow.recovery.usage.coverage@1",
                Some("flow_execution") => "agentflow.flow.execution.relation@1",
                _ => return Err(RegisteredModelFailure::InvalidFacts),
            };
            let source = &input["value"];
            crate::registered_model::check_shape(
                m,
                &m["types"]["NativeRelationSnapshotSource"],
                source,
                0,
                &mut 0,
            )?;
            let bytes = source["canonical_content"]
                .as_array()
                .ok_or(RegisteredModelFailure::InvalidFacts)?
                .iter()
                .map(|n| {
                    n.as_u64()
                        .and_then(|n| u8::try_from(n).ok())
                        .ok_or(RegisteredModelFailure::InvalidFacts)
                })
                .collect::<Result<Vec<_>, _>>()?;
            if source["catalog"]
                .get(input["slot"].as_str().unwrap_or(""))
                .is_none()
                || source["kind"] != input["slot"]
                || source["schema_family"] != family
                || source["pin"]["family"] != family
                || source["revision"] != source["pin"]["revision"]
                || source["pin"]["canonical_content_checksum"]["algorithm"] != "sha256"
                || source["pin"]["canonical_content_checksum"]["hex"]
                    != hex::encode(Sha256::digest(bytes))
                || input["source_stage"] != "before_G"
            {
                return Err(RegisteredModelFailure::InvalidFacts);
            }
            source.clone()
        }
        Some("shape") => {
            let name = input["name"]
                .as_str()
                .ok_or(RegisteredModelFailure::UnsupportedModel)?;
            let d = m["types"]
                .get(name)
                .ok_or(RegisteredModelFailure::UnsupportedModel)?;
            crate::registered_model::check_shape(m, d, &input["value"], 0, &mut 0)?;
            Value::Null
        }
        Some("dag") => {
            static SOURCES: OnceLock<Value> = OnceLock::new();
            let sources=SOURCES.get_or_init(||serde_json::from_str(include_str!("../../../spec/conformance/g01-native-source-staging/manual-source-fixtures.json")).unwrap_or(Value::Null));
            let source = sources
                .get(
                    input["fixture"]
                        .as_str()
                        .ok_or(RegisteredModelFailure::UnsupportedModel)?,
                )
                .ok_or(RegisteredModelFailure::IncompleteContent)?;
            if crate::native_source_graph_metadata::check_subgraph(source, &input["value"]).is_err()
            {
                return Err(RegisteredModelFailure::InvalidFacts);
            }
            Value::Null
        }
        Some("descriptor") => m["types"]
            .get(
                input["name"]
                    .as_str()
                    .ok_or(RegisteredModelFailure::UnsupportedModel)?,
            )
            .ok_or(RegisteredModelFailure::UnsupportedModel)?
            .clone(),
        Some("site") => r["sites"]
            .as_array()
            .and_then(|a| a.iter().find(|v| v["alias"] == input["name"]))
            .ok_or(RegisteredModelFailure::IncompleteContent)?
            .clone(),
        Some("reference") => r["reference_policies"]
            .as_array()
            .and_then(|a| {
                input["index"]
                    .as_u64()
                    .and_then(|n| usize::try_from(n).ok())
                    .and_then(|n| a.get(n))
            })
            .ok_or(RegisteredModelFailure::IncompleteContent)?
            .clone(),
        Some("rule") => {
            let rules: Value =
                serde_json::from_slice(include_bytes!("../../../spec/registry/rules.json"))
                    .map_err(|_| RegisteredModelFailure::IncompleteContent)?;
            rules["rules"]
                .as_array()
                .and_then(|a| a.iter().find(|r| r["rule_id"] == input["name"]))
                .ok_or(RegisteredModelFailure::IncompleteContent)?
                .clone()
        }
        Some("source_edge") => {
            let slot = input["slot"]
                .as_str()
                .ok_or(RegisteredModelFailure::InvalidFacts)?;
            if !["effect_resource", "recovery_usage", "flow_execution"].contains(&slot) {
                return Err(RegisteredModelFailure::InvalidFacts);
            }
            match input["presence"].as_str() {
                Some("none") => {
                    json!({"slot":slot,"source_nodes":0,"c_validation":"NotApplicable"})
                }
                Some("some") => {
                    json!({"slot":slot,"target":{"owner":MODEL,"name":"NativeRelationSnapshotSource"},"source_stage":"before_G","c_source_domain":"g_external"})
                }
                _ => return Err(RegisteredModelFailure::InvalidFacts),
            }
        }
        Some(
            "production"
            | "raw"
            | "complete_view"
            | "obligation_identity"
            | "portable_positive"
            | "root_export",
        ) => return Err(RegisteredModelFailure::NotImplemented),
        _ => return Err(RegisteredModelFailure::UnsupportedModel),
    };
    if input.get("proposal").is_some_and(|p| p != &value) {
        return Err(RegisteredModelFailure::InvalidFacts);
    }
    Ok(
        json!({"registered":value,"assurance":"registration_metadata_only","production_dagranks_generated":0,"complete_view_issued":false,"obligation_identity_issued":false,"positive_capability":"NotAuthorized"}),
    )
}
