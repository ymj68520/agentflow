//! Registered relation-only metadata. No Fact, Scope, normalizer or Checked token.
use crate::{CompilerSemanticsIdentity, RegisteredModelFailure};
use af_contract::{
    RelationCatalogPin, RelationCatalogSnapshot, RelationCatalogWire, StaticContractProofContext,
};
use af_core::{Value as CoreValue, decode, encode};
use af_idl::{CanonicalBody, CanonicalTypeExpr, CanonicalTypeSchema, CompiledModuleSet, Primitive};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;
use unicode_normalization::UnicodeNormalization;

pub const RELATION_MODEL_ID: &str = "agentflow.computation.proof-report.model@6";
const SOURCE: &str = include_str!("../../../spec/tables/proof-report-model-v6.md");

pub fn relation_model_catalogue() -> Result<&'static Value, RegisteredModelFailure> {
    static MODEL: OnceLock<Result<Value, RegisteredModelFailure>> = OnceLock::new();
    MODEL
        .get_or_init(|| {
            let body = SOURCE
                .split_once("```json\n")
                .ok_or(RegisteredModelFailure::IncompleteContent)?
                .1;
            serde_json::from_str(
                body.split_once("\n```")
                    .ok_or(RegisteredModelFailure::IncompleteContent)?
                    .0,
            )
            .map_err(|_| RegisteredModelFailure::UnsupportedModel)
        })
        .as_ref()
        .map_err(|e| *e)
}

pub fn relation_descriptor_integrity() -> Result<(), RegisteredModelFailure> {
    let m = relation_model_catalogue()?;
    if m["model_id"] != RELATION_MODEL_ID
        || m["analysis_owner"] != "agentflow.root-export-checker@7"
        || m["model_edition"] != "agentflow.g01.report-shape@6"
        || m["obligation_edition"] != "agentflow.g01.obligation@6"
    {
        return Err(RegisteredModelFailure::UnsupportedModel);
    }
    let types = m["types"]
        .as_object()
        .ok_or(RegisteredModelFailure::IncompleteContent)?;
    let mut todo: Vec<_> = types.values().collect();
    while let Some(v) = todo.pop() {
        match v {
            Value::Object(o) => {
                if let Some(n) = o.get("ref") {
                    if !n.as_str().is_some_and(|n| types.contains_key(n)) {
                        return Err(RegisteredModelFailure::IncompleteContent);
                    }
                }
                if o.contains_key("_generic") || o.contains_key("_case") {
                    return Err(RegisteredModelFailure::UnsupportedModel);
                }
                todo.extend(o.values());
            }
            Value::Array(a) => todo.extend(a),
            _ => {}
        }
    }
    Ok(())
}

pub fn validate_relation_registered_shape(
    name: &str,
    value: &Value,
) -> Result<(), RegisteredModelFailure> {
    relation_descriptor_integrity()?;
    let m = relation_model_catalogue()?;
    let d = m["types"]
        .get(name)
        .ok_or(RegisteredModelFailure::UnsupportedModel)?;
    crate::registered_model::check_shape(m, d, value, 0, &mut 0)?;
    crate::registered_model::check_named_constraints(m, name, value)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationMetadataSlot {
    EffectResource,
    RecoveryUsage,
    FlowExecution,
}
impl RelationMetadataSlot {
    pub fn name(self) -> &'static str {
        match self {
            Self::EffectResource => "effect_resource",
            Self::RecoveryUsage => "recovery_usage",
            Self::FlowExecution => "flow_execution",
        }
    }
    pub fn family(self) -> &'static str {
        match self {
            Self::EffectResource => "agentflow.effect.resource.compatibility@1",
            Self::RecoveryUsage => "agentflow.recovery.usage.coverage@1",
            Self::FlowExecution => "agentflow.flow.execution.relation@1",
        }
    }
    pub fn root_type(self) -> &'static str {
        match self {
            Self::EffectResource => "EffectResourceCompatibilityCatalog",
            Self::RecoveryUsage => "RecoveryUsageCoverageCatalog",
            Self::FlowExecution => "InformationFlowExecutionCatalog",
        }
    }
    fn target(self) -> Value {
        json!({"slot":self.name(),"family":self.family(),"root_type":self.root_type()})
    }
}

/// Borrowed native association is retained, not serialized as an address or
/// inferred from contents. This receipt only covers one selected relation slot.
pub struct RelationMetadataObservation<'c, 'u> {
    context: &'c StaticContractProofContext<'u>,
    metadata: Value,
}
impl<'c, 'u> RelationMetadataObservation<'c, 'u> {
    pub fn metadata(&self) -> &Value {
        &self.metadata
    }
    pub fn context(&self) -> &'c StaticContractProofContext<'u> {
        self.context
    }
    pub fn slot_complete(&self) -> bool {
        matches!(
            self.metadata["admission"].as_str(),
            Some("validated" | "absent")
        )
    }
    pub fn can_issue_capability(&self) -> bool {
        false
    }
}

pub fn admit_relation_observation_metadata(
    proposed: &Value,
    actual: &RelationMetadataObservation<'_, '_>,
) -> Result<(), RegisteredModelFailure> {
    validate_relation_registered_shape("RelationCurrentValidationMetadata", proposed)?;
    if proposed != actual.metadata() {
        return Err(RegisteredModelFailure::InvalidFacts);
    }
    Ok(())
}

/// Three-slot conjunction, still not a complete G01 Subject or proof.
pub struct RelationContextMetadataAdmission<'c, 'u> {
    context: &'c StaticContractProofContext<'u>,
    metadata: Value,
}
impl<'c, 'u> RelationContextMetadataAdmission<'c, 'u> {
    pub fn metadata(&self) -> &Value {
        &self.metadata
    }
    pub fn context(&self) -> &'c StaticContractProofContext<'u> {
        self.context
    }
    pub fn relation_partition_complete(&self) -> bool {
        self.metadata["relation_partition_complete"] == true
    }
    pub fn can_issue_capability(&self) -> bool {
        false
    }
}
pub fn inspect_relation_context_metadata<'c, 'u>(
    analysis: &'c StaticContractProofContext<'u>,
    canonical_validation: &StaticContractProofContext<'_>,
    compiler: &CompilerSemanticsIdentity,
) -> Result<RelationContextMetadataAdmission<'c, 'u>, RegisteredModelFailure> {
    let e = inspect_relation_metadata(
        RelationMetadataSlot::EffectResource,
        analysis,
        canonical_validation,
        compiler,
        None,
    )?;
    let r = inspect_relation_metadata(
        RelationMetadataSlot::RecoveryUsage,
        analysis,
        canonical_validation,
        compiler,
        None,
    )?;
    let f = inspect_relation_metadata(
        RelationMetadataSlot::FlowExecution,
        analysis,
        canonical_validation,
        compiler,
        None,
    )?;
    let metadata = json!({"effect_resource":e.metadata(),"recovery_usage":r.metadata(),"flow_execution":f.metadata(),"relation_partition_complete":e.slot_complete()&&r.slot_complete()&&f.slot_complete(),"scope_state":"remaining_scope_incomplete"});
    validate_relation_registered_shape("RelationContextAdmissionMetadata", &metadata)?;
    Ok(RelationContextMetadataAdmission {
        context: analysis,
        metadata,
    })
}
pub fn admit_relation_context_observation_metadata(
    proposed: &Value,
    actual: &RelationContextMetadataAdmission<'_, '_>,
) -> Result<(), RegisteredModelFailure> {
    validate_relation_registered_shape("RelationContextAdmissionMetadata", proposed)?;
    if proposed != actual.metadata() {
        return Err(RegisteredModelFailure::InvalidFacts);
    }
    Ok(())
}

/// Closed source-only DAG probe. No edge to a validation output can occur in
/// its node grammar. This does not certify a required input roster or Scope.
pub fn validate_relation_source_dag_metadata(value: &Value) -> Result<(), RegisteredModelFailure> {
    validate_relation_registered_shape("RelationSourceDagMetadata", value)?;
    let nodes = value["nodes"]
        .as_array()
        .ok_or(RegisteredModelFailure::IncompleteFacts)?;
    let roots = value["roots"]
        .as_array()
        .ok_or(RegisteredModelFailure::IncompleteFacts)?;
    if roots.len() != nodes.len()
        || roots
            .iter()
            .enumerate()
            .any(|(i, r)| r.as_u64() != Some(i as u64))
        || nodes
            .iter()
            .enumerate()
            .any(|(i, n)| nodes[..i].contains(n))
    {
        return Err(RegisteredModelFailure::InvalidFacts);
    }
    Ok(())
}
fn primitive_name(p: Primitive) -> &'static str {
    match p {
        Primitive::Unit => "Unit",
        Primitive::Null => "Null",
        Primitive::Bool => "Bool",
        Primitive::Int32 => "Int32",
        Primitive::UInt32 => "UInt32",
        Primitive::Int64 => "Int64",
        Primitive::UInt64 => "UInt64",
        Primitive::Decimal => "Decimal",
        Primitive::Float64 => "Float64",
        Primitive::String => "String",
        Primitive::Bytes => "Bytes",
        Primitive::Timestamp => "Timestamp",
        Primitive::Duration => "Duration",
    }
}
fn expression(e: &CanonicalTypeExpr) -> Value {
    match e {
        CanonicalTypeExpr::Primitive(p) => json!({"Primitive":{"0":primitive_name(*p)}}),
        CanonicalTypeExpr::NamedRef(n) => json!({"NamedRef":{"0":n}}),
        CanonicalTypeExpr::List(e) => json!({"List":{"0":expression(e)}}),
        CanonicalTypeExpr::MapString(e) => json!({"MapString":{"0":expression(e)}}),
        CanonicalTypeExpr::Option(e) => json!({"Option":{"0":expression(e)}}),
    }
}
/// Exact native canonical schema projection; no Debug/error-string parsing.
pub fn relation_schema_definition_metadata(s: &CanonicalTypeSchema) -> Value {
    let body = match &s.body {
        CanonicalBody::Alias(e) => json!({"Alias":{"0":expression(e)}}),
        CanonicalBody::Enum(v) => {
            json!({"Enum":{"0":v.iter().map(|r|json!({"no":r.no,"name":r.name})).collect::<Vec<_>>()}})
        }
        CanonicalBody::Record(v) => {
            json!({"Record":{"0":v.iter().map(|r|json!({"no":r.no,"name":r.name,"type_expr":expression(&r.type_expr)})).collect::<Vec<_>>()}})
        }
        CanonicalBody::Variant(v) => {
            json!({"Variant":{"0":v.iter().map(|r|json!({"no":r.no,"name":r.name,"type_expr":expression(&r.type_expr)})).collect::<Vec<_>>()}})
        }
    };
    json!({"schema":s.schema,"name":s.name,"body":body})
}
fn schema_refs(e: &CanonicalTypeExpr, out: &mut Vec<String>) {
    match e {
        CanonicalTypeExpr::NamedRef(n) => out.push(n.clone()),
        CanonicalTypeExpr::List(e)
        | CanonicalTypeExpr::MapString(e)
        | CanonicalTypeExpr::Option(e) => schema_refs(e, out),
        CanonicalTypeExpr::Primitive(_) => {}
    }
}
fn definition_closure<'a>(
    root: &'a CanonicalTypeSchema,
    modules: &'a CompiledModuleSet,
) -> Result<Vec<&'a CanonicalTypeSchema>, String> {
    let mut pending = vec![root.full_identity()];
    let mut found = BTreeMap::new();
    while let Some(id) = pending.pop() {
        if found.contains_key(&id) {
            continue;
        }
        let s = modules.type_by_identity(&id).ok_or(id.clone())?;
        match &s.body {
            CanonicalBody::Alias(e) => schema_refs(e, &mut pending),
            CanonicalBody::Record(v) => {
                for f in v {
                    schema_refs(&f.type_expr, &mut pending);
                }
            }
            CanonicalBody::Variant(v) => {
                for f in v {
                    schema_refs(&f.type_expr, &mut pending);
                }
            }
            CanonicalBody::Enum(_) => {}
        }
        found.insert(id, s);
    }
    Ok(found.into_values().collect())
}
fn value_json(v: &CoreValue, metadata: bool) -> Value {
    match v {
        CoreValue::Unit | CoreValue::Null => Value::Null,
        CoreValue::Bool(v) => json!(v),
        CoreValue::Int32(v) => json!(v),
        CoreValue::UInt32(v) => json!(v),
        CoreValue::Int64(v) => json!(v),
        CoreValue::UInt64(v) => json!(v),
        CoreValue::Decimal(v) => json!(v.to_string()),
        CoreValue::Float64(v) => json!(v.get()),
        CoreValue::String(v) | CoreValue::Enum(v) => json!(v),
        CoreValue::Bytes(v) => json!(v),
        CoreValue::Timestamp(v) => json!(v.as_micros()),
        CoreValue::Duration(v) => json!(v.as_micros()),
        CoreValue::List(v) => json!(
            v.iter()
                .map(|v| value_json(v, metadata))
                .collect::<Vec<_>>()
        ),
        CoreValue::Option(None) => {
            if metadata {
                json!({"none":null})
            } else {
                Value::Null
            }
        }
        CoreValue::Option(Some(v)) => {
            if metadata {
                json!({"some":value_json(v,metadata)})
            } else {
                value_json(v, metadata)
            }
        }
        CoreValue::Record(v) | CoreValue::Map(v) => Value::Object(
            v.iter()
                .map(|(k, v)| (k.clone(), value_json(v, metadata)))
                .collect(),
        ),
        CoreValue::Variant { case, value } => json!({case:value_json(value,metadata)}),
    }
}
fn pin(p: &RelationCatalogPin) -> Value {
    json!({"family":p.schema,"revision":p.revision,"canonical_content_checksum":{"algorithm":p.canonical_content_checksum.algorithm,"hex":p.canonical_content_checksum.hex}})
}
fn source<T: RelationCatalogWire>(
    slot: RelationMetadataSlot,
    s: &RelationCatalogSnapshot<T>,
) -> Value {
    json!({"kind":slot.name(),"schema_family":s.schema(),"revision":s.revision(),"pin":pin(s.pin()),"canonical_content":s.canonical_bytes(),"catalog":{slot.name():value_json(&s.catalog().to_value(),true)}})
}
fn native_source(slot: RelationMetadataSlot, c: &StaticContractProofContext<'_>) -> Option<Value> {
    match slot {
        RelationMetadataSlot::EffectResource => c
            .relations
            .effect_resource
            .as_ref()
            .map(|s| source(slot, s)),
        RelationMetadataSlot::RecoveryUsage => {
            c.relations.recovery_usage.as_ref().map(|s| source(slot, s))
        }
        RelationMetadataSlot::FlowExecution => {
            c.relations.flow_execution.as_ref().map(|s| source(slot, s))
        }
    }
}
fn source_integrity(slot: RelationMetadataSlot, source: &Value) -> bool {
    let Some(family) = source["schema_family"].as_str() else {
        return false;
    };
    let Some(revision) = source["revision"].as_str() else {
        return false;
    };
    let Some(array) = source["canonical_content"].as_array() else {
        return false;
    };
    let bytes: Option<Vec<u8>> = array
        .iter()
        .map(|v| v.as_u64().and_then(|n| u8::try_from(n).ok()))
        .collect();
    let Some(bytes) = bytes else {
        return false;
    };
    af_idl::SpecificationId::parse(family).is_ok()
        && family == slot.family()
        && source["pin"]["family"] == family
        && source["pin"]["revision"] == revision
        && !revision.is_empty()
        && revision.nfc().eq(revision.chars())
        && source["pin"]["canonical_content_checksum"]["algorithm"] == "sha256"
        && source["pin"]["canonical_content_checksum"]["hex"] == hex::encode(Sha256::digest(bytes))
        && source["kind"] == slot.name()
        && source["catalog"]
            .as_object()
            .is_some_and(|o| o.len() == 1 && o.contains_key(slot.name()))
}
type RelationValidationError = (&'static str, Option<af_core::AceError>);
fn codec_failure(e: &af_core::AceError) -> Value {
    use af_core::AceError as E;
    match e {
        E::InvalidInteger { input } => json!({"invalid_integer":{"input":input}}),
        E::IntegerOutOfRange { input, type_name } => {
            json!({"integer_out_of_range":{"input":input,"type_name":type_name}})
        }
        E::InvalidDecimal { input } => json!({"invalid_decimal":{"input":input}}),
        E::NonFiniteFloat => json!({"non_finite_float":null}),
        E::InvalidUnicode { detail } => json!({"invalid_unicode":{"detail":detail}}),
        E::DuplicateNormalizedKey { key } => json!({"duplicate_normalized_key":{"key":key}}),
        E::DuplicateField { field } => json!({"duplicate_field":{"field":field}}),
        E::UnknownField { field } => json!({"unknown_field":{"field":field}}),
        E::MissingField { field } => json!({"missing_field":{"field":field}}),
        E::InvalidBytesEncoding { detail } => json!({"invalid_bytes_encoding":{"detail":detail}}),
        E::UnknownEnumValue { value } => json!({"unknown_enum_value":{"value":value}}),
        E::UnknownVariantCase { case } => json!({"unknown_variant_case":{"case":case}}),
        E::InvalidOptionEncoding { detail } => json!({"invalid_option_encoding":{"detail":detail}}),
        E::TypeMismatch { expected, actual } => {
            json!({"type_mismatch":{"expected":expected,"actual":actual}})
        }
        E::NonCanonicalEncoding { detail } => json!({"non_canonical_encoding":{"detail":detail}}),
        E::InvalidShape { detail } => json!({"invalid_shape":{"detail":detail}}),
    }
}
fn revalidate<T>(
    s: &RelationCatalogSnapshot<T>,
    modules: &CompiledModuleSet,
    slot: RelationMetadataSlot,
) -> Result<(), RelationValidationError>
where
    T: RelationCatalogWire + PartialEq + for<'de> Deserialize<'de>,
{
    let canonical = s
        .catalog()
        .clone()
        .canonicalized()
        .map_err(|_| ("catalog_canonicalization", None))?;
    if canonical != *s.catalog() {
        return Err(("catalog_canonicalization", None));
    }
    let shape = modules
        .shape_of(&format!("{}/{}", slot.family(), slot.root_type()))
        .map_err(|_| ("current_encode", None))?;
    let bytes = encode(&shape, &s.catalog().to_value()).map_err(|e| ("current_encode", Some(e)))?;
    let decoded = decode(&shape, s.canonical_bytes()).map_err(|e| ("current_decode", Some(e)))?;
    let catalog: T =
        serde_json::from_value(value_json(&decoded, false)).map_err(|_| ("native_decode", None))?;
    if catalog != *s.catalog() {
        return Err(("catalog_equality", None));
    }
    if bytes != s.canonical_bytes() {
        return Err(("byte_roundtrip", None));
    }
    Ok(())
}

pub fn inspect_relation_metadata<'c, 'u>(
    slot: RelationMetadataSlot,
    analysis: &'c StaticContractProofContext<'u>,
    canonical_validation: &StaticContractProofContext<'_>,
    compiler: &CompilerSemanticsIdentity,
    expected_pin: Option<&RelationCatalogPin>,
) -> Result<RelationMetadataObservation<'c, 'u>, RegisteredModelFailure> {
    let actual = native_source(slot, analysis);
    let same = std::ptr::eq(analysis, canonical_validation);
    let modules:Vec<_>=analysis.modules.modules().map(|m|json!({"specification_id":m.specification.canonical(),"types":m.types.iter().map(relation_schema_definition_metadata).collect::<Vec<_>>()})).collect();
    // This is the explicitly frozen relation-only metadata algorithm, not a
    // current structural Subject/Fact carrier. Its native A/C operations and
    // payload remain unchanged; current compiler admission remains separate.
    let mut view = json!({"assurance":"relation_slot_only","algorithm":{"analysis":"agentflow.root-export-checker@7","model":RELATION_MODEL_ID,"lookup_rule":"AF-SEMANTIC-0288","validation_rule":"AF-SEMANTIC-0292"},"target":slot.target(),"source":actual.as_ref().map_or_else(||json!({"none":null}),|s|json!({"some":s})),"context_roles":if same {json!([["analysis","canonical_validation"]])} else {json!([["analysis"],["canonical_validation"]])},"current_modules":modules,"lookup":{"not_reached":{"cause":"missing_source"}},"validation":{"not_reached":null},"admission":"incomplete"});
    let cause = if crate::check_registered_compiler_identity(compiler).is_err() {
        Some("invalid_identity")
    } else if !same {
        Some("invalid_context")
    } else {
        None
    };
    if let Some(cause) = cause {
        view["lookup"] = json!({"not_reached":{"cause":cause}});
    } else if let Some(s) = actual {
        let actual_pin = match slot {
            RelationMetadataSlot::EffectResource => analysis
                .relations
                .effect_resource
                .as_ref()
                .map(RelationCatalogSnapshot::pin),
            RelationMetadataSlot::RecoveryUsage => analysis
                .relations
                .recovery_usage
                .as_ref()
                .map(RelationCatalogSnapshot::pin),
            RelationMetadataSlot::FlowExecution => analysis
                .relations
                .flow_execution
                .as_ref()
                .map(RelationCatalogSnapshot::pin),
        }
        .ok_or(RegisteredModelFailure::IncompleteFacts)?;
        if !source_integrity(slot, &s) {
            view["lookup"] = json!({"not_reached":{"cause":"invalid_source"}});
        } else if let Some(expected) = expected_pin.filter(|p| p.schema != slot.family()) {
            view["lookup"] =
                json!({"family_mismatch":{"target":slot.target(),"source_family":expected.schema}});
        } else if let Some(expected) = expected_pin.filter(|p| p.revision != actual_pin.revision) {
            view["lookup"] =
                json!({"revision_mismatch":{"actual":pin(actual_pin),"expected":pin(expected)}});
        } else if expected_pin
            .is_some_and(|p| p.canonical_content_checksum != actual_pin.canonical_content_checksum)
        {
            view["lookup"] = json!({"not_reached":{"cause":"invalid_source"}});
        } else {
            let identity = format!("{}/{}", slot.family(), slot.root_type());
            if let Some(root) = analysis.modules.type_by_identity(&identity) {
                match definition_closure(root, analysis.modules) {
                    Err(missing) => {
                        view["lookup"] = json!({"missing":{"target":slot.target(),"part":"definition","missing_identity":missing}});
                    }
                    Ok(closure) => {
                        let digest = root
                            .digest()
                            .map_err(|_| RegisteredModelFailure::IncompleteContent)?;
                        let found = json!({"target":slot.target(),"schema":{"specification_id":root.schema,"type_name":root.name,"schema_digest":{"algorithm":"sha256","domain":"type-schema","hex":digest.hex()}},"definition":relation_schema_definition_metadata(root),"definition_closure":closure.into_iter().map(relation_schema_definition_metadata).collect::<Vec<_>>()});
                        view["lookup"] = json!({"found":found});
                        let result = match slot {
                            RelationMetadataSlot::EffectResource => revalidate(
                                analysis
                                    .relations
                                    .effect_resource
                                    .as_ref()
                                    .ok_or(RegisteredModelFailure::IncompleteFacts)?,
                                analysis.modules,
                                slot,
                            ),
                            RelationMetadataSlot::RecoveryUsage => revalidate(
                                analysis
                                    .relations
                                    .recovery_usage
                                    .as_ref()
                                    .ok_or(RegisteredModelFailure::IncompleteFacts)?,
                                analysis.modules,
                                slot,
                            ),
                            RelationMetadataSlot::FlowExecution => revalidate(
                                analysis
                                    .relations
                                    .flow_execution
                                    .as_ref()
                                    .ok_or(RegisteredModelFailure::IncompleteFacts)?,
                                analysis.modules,
                                slot,
                            ),
                        };
                        match result {
                            Ok(()) => {
                                view["validation"] = json!({"valid":{"found":found,"validated_catalog":s["catalog"],"canonical_content":s["canonical_content"],"completed_steps":["catalog_canonicalization","current_encode","current_decode","native_decode","catalog_equality","byte_roundtrip"]}});
                                view["admission"] = json!("validated");
                            }
                            Err((step, error)) => {
                                let steps = [
                                    "catalog_canonicalization",
                                    "current_encode",
                                    "current_decode",
                                    "native_decode",
                                    "catalog_equality",
                                    "byte_roundtrip",
                                ];
                                let completed = &steps[..steps
                                    .iter()
                                    .position(|s| *s == step)
                                    .ok_or(RegisteredModelFailure::InvalidFacts)?];
                                let sites = match step {
                                    "catalog_canonicalization" => vec!["source_catalog"],
                                    "current_decode" | "native_decode" => {
                                        vec!["canonical_content", "current_schema"]
                                    }
                                    _ => vec!["source_catalog", "current_schema"],
                                };
                                view["validation"] = json!({"invalid":{"found":found,"step":step,"codec_failure":error.as_ref().map_or_else(||json!({"none":null}),|e|json!({"some":codec_failure(e)})),"input_sites":sites,"completed_steps":completed}});
                            }
                        }
                    }
                }
            } else {
                view["lookup"] = json!({"missing":{"target":slot.target(),"part":if analysis.modules.module(slot.family()).is_some(){"root"}else{"module"},"missing_identity":identity}});
            }
        }
    } else if expected_pin.is_none() {
        view["admission"] = json!("absent");
    }
    validate_relation_registered_shape("RelationCurrentValidationMetadata", &view)?;
    Ok(RelationMetadataObservation {
        context: analysis,
        metadata: view,
    })
}

/// Malformed pre-context candidates are a separate boundary. Valid native
/// CompiledModuleSet never contains duplicates. This function cannot validate
/// a Context or produce C Valid; it records exact conflicting real definitions.
pub fn inspect_relation_definition_candidates(
    slot: RelationMetadataSlot,
    definitions: &[CanonicalTypeSchema],
) -> Value {
    let selected: Vec<_> = definitions
        .iter()
        .filter(|s| s.schema == slot.family() && s.name == slot.root_type())
        .collect();
    let identities: BTreeSet<_> = selected.iter().map(|s| s.full_identity()).collect();
    if selected.len() > identities.len() {
        json!({"ambiguous":{"target":slot.target(),"definitions":selected.into_iter().map(relation_schema_definition_metadata).collect::<Vec<_>>()}})
    } else if selected.is_empty() {
        json!({"missing":{"target":slot.target(),"part":"root","missing_identity":format!("{}/{}",slot.family(),slot.root_type())}})
    } else {
        json!({"not_reached":{"cause":"invalid_context"}})
    }
}
