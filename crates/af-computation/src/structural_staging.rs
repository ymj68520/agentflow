//! Non-authoritative structural registration metadata. NO binder/rank producer.
//! Handwritten registration subgraphs cannot create a native source receipt,
//! complete Fact, Scope, normalized report or checked capability.
use crate::RegisteredModelFailure;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

pub const STRUCTURAL_MODEL_ID: &str = "agentflow.computation.proof-report.model@7";
const SOURCE: &str = include_str!("../../../spec/tables/proof-report-model-v7.md");
const PREVIOUS: &str = include_str!("../../../spec/tables/proof-report-model-v6.md");
const FIXTURES: &str = include_str!(
    "../../../spec/conformance/g01-structural-binding-staging/manual-source-fixtures.json"
);

#[derive(Debug, Clone)]
pub struct StructuralMetadataFailure {
    domain: RegisteredModelFailure,
    reason: &'static str,
}
impl StructuralMetadataFailure {
    pub(crate) fn new(domain: RegisteredModelFailure, reason: &'static str) -> Self {
        Self { domain, reason }
    }
    #[must_use]
    pub fn metadata(&self) -> Value {
        json!({"domain":format!("{:?}",self.domain),"reason":self.reason,"stage":"registration_admission","site":{"none":null},"expected_type":{"none":null},"observed_type":{"none":null},"first_input_path":[],"second_input_path":{"none":null}})
    }
}
pub(crate) type MetaResult<T> = Result<T, StructuralMetadataFailure>;
pub(crate) fn bad(reason: &'static str) -> StructuralMetadataFailure {
    StructuralMetadataFailure::new(RegisteredModelFailure::InvalidFacts, reason)
}
pub(crate) fn unsupported(reason: &'static str) -> StructuralMetadataFailure {
    StructuralMetadataFailure::new(RegisteredModelFailure::UnsupportedModel, reason)
}
pub(crate) fn missing(reason: &'static str) -> StructuralMetadataFailure {
    StructuralMetadataFailure::new(RegisteredModelFailure::IncompleteFacts, reason)
}
pub(crate) fn resource() -> StructuralMetadataFailure {
    StructuralMetadataFailure::new(RegisteredModelFailure::ResourceLimit, "resource_bound")
}
fn parse_source(text: &str) -> Result<Value, RegisteredModelFailure> {
    let body = text
        .split_once("```json\n")
        .ok_or(RegisteredModelFailure::IncompleteContent)?
        .1;
    serde_json::from_str(
        body.split_once("\n```")
            .ok_or(RegisteredModelFailure::IncompleteContent)?
            .0,
    )
    .map_err(|_| RegisteredModelFailure::UnsupportedModel)
}
pub fn structural_model_catalogue() -> Result<&'static Value, RegisteredModelFailure> {
    static M: OnceLock<Result<Value, RegisteredModelFailure>> = OnceLock::new();
    M.get_or_init(compose_source).as_ref().map_err(|e| *e)
}
fn compose_source() -> Result<Value, RegisteredModelFailure> {
    let delta = parse_source(SOURCE)?;
    let fail = RegisteredModelFailure::IncompleteContent;
    let expected = BTreeSet::from([
        "format",
        "base_source",
        "overrides",
        "append_types",
        "replace_types",
        "append_type_ranks",
        "append_roots",
    ]);
    if delta
        .as_object()
        .is_none_or(|o| o.keys().map(String::as_str).collect::<BTreeSet<_>>() != expected)
        || delta["format"] != "closed_structural_delta_v1"
        || delta["base_source"]
            != json!({"specification_id":"agentflow.computation.proof-report.model@6","logical_path":"spec/tables/proof-report-model-v6.md","plain_sha256":hex::encode(Sha256::digest(PREVIOUS.as_bytes()))})
    {
        return Err(fail);
    }
    let base = parse_source(PREVIOUS)?;
    let mut m = base.clone();
    let overrides = delta["overrides"].as_object().ok_or(fail)?;
    if overrides
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
        != BTreeSet::from([
            "model_id",
            "model_edition",
            "obligation_edition",
            "analysis_owner",
            "profile",
            "canonical_constraints",
            "structural_binding_registration",
        ])
    {
        return Err(fail);
    }
    m.as_object_mut().ok_or(fail)?.extend(overrides.clone());
    let replace = delta["replace_types"].as_object().ok_or(fail)?;
    if replace.keys().map(String::as_str).collect::<BTreeSet<_>>()
        != BTreeSet::from(["NodeBody", "ScopeSubject"])
    {
        return Err(fail);
    }
    m["types"]
        .as_object_mut()
        .ok_or(fail)?
        .extend(replace.clone());
    for (n, d) in delta["append_types"].as_object().ok_or(fail)? {
        if base["types"].get(n).is_some() {
            return Err(fail);
        }
        m["types"]
            .as_object_mut()
            .ok_or(fail)?
            .insert(n.clone(), d.clone());
    }
    m["type_ranks"]
        .as_object_mut()
        .ok_or(fail)?
        .extend(delta["append_type_ranks"].as_object().ok_or(fail)?.clone());
    if m["type_ranks"]
        .as_object()
        .ok_or(fail)?
        .keys()
        .collect::<BTreeSet<_>>()
        != m["types"]
            .as_object()
            .ok_or(fail)?
            .keys()
            .collect::<BTreeSet<_>>()
        || delta["append_type_ranks"]
            .as_object()
            .ok_or(fail)?
            .keys()
            .collect::<BTreeSet<_>>()
            != delta["append_types"]
                .as_object()
                .ok_or(fail)?
                .keys()
                .collect::<BTreeSet<_>>()
    {
        return Err(fail);
    }
    m["descriptor_roots"]
        .as_array_mut()
        .ok_or(fail)?
        .extend(delta["append_roots"].as_array().ok_or(fail)?.clone());
    m["inherited_registered_model"] = base;
    Ok(m)
}
pub(crate) fn catalogue() -> MetaResult<&'static Value> {
    structural_model_catalogue().map_err(|e| StructuralMetadataFailure::new(e, "missing_site"))
}
pub(crate) fn nominal(key: &str, owner: &str) -> Value {
    if let Some((a, b)) = key.split_once('/') {
        if af_idl::SpecificationId::parse(a).is_ok() {
            return json!({"owner":a,"name":b});
        }
    }
    json!({"owner":owner,"name":key})
}
pub(crate) fn type_key(mt: &Value) -> MetaResult<String> {
    let m = catalogue()?;
    let types = m["types"]
        .as_object()
        .ok_or_else(|| unsupported("unknown_nominal_type"))?;
    types
        .keys()
        .find(|n| nominal(n, STRUCTURAL_MODEL_ID) == *mt)
        .cloned()
        .ok_or_else(|| unsupported("unknown_nominal_type"))
}
fn refs(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Object(o) => {
            if let Some(n) = o.get("ref").and_then(Value::as_str) {
                out.push(n.to_owned());
            }
            for v in o.values() {
                refs(v, out);
            }
        }
        Value::Array(a) => {
            for v in a {
                refs(v, out);
            }
        }
        _ => {}
    }
}
fn closure(types: &Value, roots: &[&str]) -> Result<BTreeSet<String>, RegisteredModelFailure> {
    let mut seen = BTreeSet::new();
    let mut todo = roots.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    while let Some(n) = todo.pop() {
        if !seen.insert(n.clone()) {
            continue;
        }
        let d = types
            .get(&n)
            .ok_or(RegisteredModelFailure::IncompleteContent)?;
        refs(d, &mut todo);
    }
    Ok(seen)
}
pub(crate) fn children(d: &Value) -> Vec<(Value, &Value)> {
    let mut out = Vec::new();
    match d["kind"].as_str() {
        Some("record" | "enum") => {
            let field = if d["kind"] == "record" {
                "fields"
            } else {
                "variants"
            };
            let tag = if field == "fields" { "field" } else { "case" };
            if let Some(rows) = d[field].as_array() {
                for row in rows {
                    out.push((json!({tag:row[0]}), &row[1]));
                }
            }
        }
        Some("sequence" | "set" | "option") => {
            let tag = match d["kind"].as_str() {
                Some("sequence") => "item",
                Some("set") => "member",
                _ => "some",
            };
            out.push((json!({tag:null}), &d["item"]));
        }
        Some("map") => {
            out.push((json!({"key":null}), &d["key"]));
            out.push((json!({"value":null}), &d["value"]));
        }
        _ => {}
    }
    out
}
fn syntax_sites(parent: &str, d: &Value, path: &[Value], out: &mut BTreeMap<String, Value>) {
    if d.get("ref").is_some() || d["kind"] == "dag_ref" {
        return;
    }
    if !path.is_empty() {
        out.insert(json!([parent, path]).to_string(), d.clone());
    }
    for (part, c) in children(d) {
        let mut p = path.to_vec();
        p.push(part);
        syntax_sites(parent, c, &p, out);
    }
}

/// Full source-derived catalogue closure, cached only for compiled immutable bytes.
pub fn structural_descriptor_integrity() -> Result<(), RegisteredModelFailure> {
    static RESULT: OnceLock<Result<(), RegisteredModelFailure>> = OnceLock::new();
    *RESULT.get_or_init(check_catalogue)
}
#[allow(clippy::too_many_lines)]
fn check_catalogue() -> Result<(), RegisteredModelFailure> {
    let m = structural_model_catalogue()?;
    let old = parse_source(PREVIOUS)?;
    let fail = RegisteredModelFailure::IncompleteContent;
    if m["model_id"] != STRUCTURAL_MODEL_ID
        || m["analysis_owner"] != crate::ROOT_EXPORT_ANALYSIS_SEMANTICS_VERSION
        || m["model_edition"] != crate::REPORT_SHAPE_EDITION
        || m["obligation_edition"] != crate::OBLIGATION_IDENTITY_EDITION
        || m["inherited_registered_model"] != old
    {
        return Err(fail);
    }
    if m["historical_selected_imports"] != old["historical_selected_imports"]
        || m["historical_import_sources"] != old["historical_import_sources"]
    {
        return Err(fail);
    }
    let t = m["types"].as_object().ok_or(fail)?;
    let ot = old["types"].as_object().ok_or(fail)?;
    for (n, d) in ot {
        if m["type_ranks"][n] != old["type_ranks"][n] {
            return Err(fail);
        }
        if !matches!(n.as_str(), "NodeBody" | "ScopeSubject") && t.get(n) != Some(d) {
            return Err(fail);
        }
    }
    let variants = m["types"]["NodeBody"]["variants"].as_array().ok_or(fail)?;
    if variants.len() != 8
        || variants[..7]
            != old["types"]["NodeBody"]["variants"]
                .as_array()
                .ok_or(fail)?[..]
        || variants[7]
            != json!(["map",{"kind":"sequence","item":{"ref":"StructuralMapEntry"},"minimum":0}])
    {
        return Err(fail);
    }
    let ranks = m["type_ranks"]
        .as_object()
        .ok_or(fail)?
        .values()
        .filter_map(Value::as_u64)
        .collect::<BTreeSet<_>>();
    if ranks.len() != t.len()
        || ranks
            .iter()
            .copied()
            .ne(0..u64::try_from(t.len()).map_err(|_| fail)?)
    {
        return Err(fail);
    }
    let mut references = Vec::new();
    refs(&m["types"], &mut references);
    if references.iter().any(|n| !t.contains_key(n)) {
        return Err(fail);
    }
    let inputs = closure(&old["types"], &["RawValidationSubject"])?;
    let outputs = closure(
        &old["types"],
        &[
            "RawPortCheckBasis",
            "RawPortRejectionWitness",
            "CurrentContextRelationSchemaValidation",
            "RelationSubjectAdmission",
        ],
    )?;
    let reg = &m["structural_binding_registration"];
    let strings = |v: &Value| {
        v.as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect::<BTreeSet<_>>()
    };
    if strings(&reg["input_types"]) != inputs || strings(&reg["output_types"]) != outputs {
        return Err(fail);
    }
    let mut required = BTreeMap::new();
    for n in inputs.union(&outputs) {
        syntax_sites(n, &old["types"][n], &[], &mut required);
    }
    let rows = reg["sites"].as_array().ok_or(fail)?;
    let mut got = BTreeMap::new();
    let mut aliases = BTreeSet::new();
    for row in rows {
        let parent = row["source_catalogue_key"].as_str().ok_or(fail)?;
        let source_path = row["source_path"]
            .as_array()
            .ok_or(fail)?
            .iter()
            .map(|p| json!({p[0].as_str().unwrap_or(""):p[1]}))
            .collect::<Vec<_>>();
        let k = json!([parent, source_path]).to_string();
        let d = required.get(&k).ok_or(fail)?;
        let def = &row["definition"];
        let alias = row["alias"].as_str().ok_or(fail)?;
        if got.insert(k, d.clone()).is_some()
            || !aliases.insert(alias.to_owned())
            || row["descriptor"] != *d
            || m["types"][alias] != *d
            || def["value_type"] != nominal(alias, STRUCTURAL_MODEL_ID)
            || def["key"]
                != json!({"parent":nominal(parent,STRUCTURAL_MODEL_ID),"path":source_path})
            || def["parent_source"] != nominal(parent, "agentflow.computation.proof-report.model@6")
        {
            return Err(fail);
        }
    }
    if got != required {
        return Err(fail);
    }
    let source = closure(&m["types"], &["StructuralDeclaredInputs"])?;
    let mut pending = source.iter().map(|n| &m["types"][n]).collect::<Vec<_>>();
    while let Some(v) = pending.pop() {
        match v {
            Value::Object(o) => {
                if o.get("kind").is_some_and(|v| v == "dag_ref") {
                    return Err(fail);
                }
                pending.extend(o.values());
            }
            Value::Array(a) => pending.extend(a),
            _ => {}
        }
    }
    if reg["stage_contract"]["status"]["production_binder"] != "NotImplemented"
        || reg["stage_contract"]["status"]["root_export"] != "NotAuthorized"
    {
        return Err(fail);
    }
    Ok(())
}

fn nominal_positions(d: &Value, value: &Value, depth: usize) -> MetaResult<()> {
    if depth > 96 {
        return Err(resource());
    }
    let m = catalogue()?;
    if let Some(n) = d["ref"].as_str() {
        if matches!(n, "ModelType" | "ModelRef") {
            type_key(value)?;
        }
        if n == "SpecificationId"
            && value
                .as_str()
                .is_none_or(|s| af_idl::SpecificationId::parse(s).is_err())
        {
            return Err(bad("invalid_shape"));
        }
        if n == "StructuralSourceType" {
            let old = &m["inherited_registered_model"];
            if !old["types"].as_object().is_some_and(|t| {
                t.keys()
                    .any(|k| nominal(k, "agentflow.computation.proof-report.model@6") == *value)
            }) {
                return Err(unsupported("invalid_shape"));
            }
        }
        if n == "StructuralEditionTuple"
            && *value
                != json!({"analysis":crate::ROOT_EXPORT_ANALYSIS_SEMANTICS_VERSION,"model":STRUCTURAL_MODEL_ID,"report":crate::REPORT_SHAPE_EDITION,"obligation":crate::OBLIGATION_IDENTITY_EDITION})
        {
            return Err(unsupported("cross_edition"));
        }
        return nominal_positions(&m["types"][n], value, depth + 1);
    }
    match d["kind"].as_str() {
        Some("record") => {
            for row in d["fields"].as_array().into_iter().flatten() {
                nominal_positions(&row[1], &value[row[0].as_str().unwrap_or("")], depth + 1)?;
            }
        }
        Some("enum") => {
            if let Some(o) = value.as_object() {
                if let Some((tag, v)) = o.iter().next() {
                    if let Some(row) = d["variants"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .find(|r| r[0] == tag.as_str())
                    {
                        nominal_positions(&row[1], v, depth + 1)?;
                    }
                }
            }
        }
        Some("sequence" | "set") => {
            for v in value.as_array().into_iter().flatten() {
                nominal_positions(&d["item"], v, depth + 1)?;
            }
        }
        Some("map") => {
            for pair in value.as_array().into_iter().flatten() {
                nominal_positions(&d["key"], &pair[0], depth + 1)?;
                nominal_positions(&d["value"], &pair[1], depth + 1)?;
            }
        }
        Some("option") => {
            if let Some(v) = value.get("some") {
                nominal_positions(&d["item"], v, depth + 1)?;
            }
        }
        _ => {}
    }
    Ok(())
}
pub fn validate_structural_shape_metadata(name: &str, value: &Value) -> MetaResult<()> {
    structural_descriptor_integrity()
        .map_err(|e| StructuralMetadataFailure::new(e, "missing_site"))?;
    let m = catalogue()?;
    let d = m["types"]
        .get(name)
        .ok_or_else(|| unsupported("unknown_nominal_type"))?;
    crate::registered_model::check_shape(m, d, value, 0, &mut 0).map_err(|e| {
        StructuralMetadataFailure::new(
            e,
            if e == RegisteredModelFailure::ResourceLimit {
                "resource_bound"
            } else {
                "invalid_shape"
            },
        )
    })?;
    nominal_positions(&json!({"ref":name}), value, 0)
}
fn field_roster() -> MetaResult<Value> {
    let m = catalogue()?;
    let mut rows = Vec::new();
    for name in m["structural_binding_registration"]["coverage_types"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        let mut todo = vec![(&m["types"][name], Vec::<Value>::new())];
        while let Some((d, path)) = todo.pop() {
            if d.get("ref").is_some() {
                continue;
            }
            if let Some(fields) = d["fields"].as_array() {
                for f in fields {
                    rows.push(json!({"type":name,"path":path,"field":f[0],"variant":null}));
                }
            }
            if let Some(cases) = d["variants"].as_array() {
                for c in cases {
                    rows.push(json!({"type":name,"path":path,"field":null,"variant":c[0]}));
                }
            }
            for (part, c) in children(d) {
                let mut p = path.clone();
                if let Some((k, v)) = part.as_object().and_then(|o| o.iter().next()) {
                    p.push(json!([k, v]));
                }
                todo.push((c, p));
            }
        }
    }
    // Python's sorted-key JSON includes spaces only for ordering; key names
    // are fixed and whitespace cannot change their lexicographic relation.
    rows.sort_by_key(Value::to_string);
    Ok(Value::Array(rows))
}

/// Rank-free native map projection only; no source receipt or graph is produced.
pub fn inspect_structural_module_map(
    sources: &[(String, String)],
    proposal: Option<&Value>,
) -> MetaResult<Value> {
    let modules = af_idl::compile_module_set(sources).map_err(|_| bad("invalid_map_entry"))?;
    let mut entries = Vec::new();
    for module in modules.modules() {
        let key = module.specification.to_string();
        af_idl::SpecificationId::parse(&key).map_err(|_| bad("invalid_map_entry"))?;
        if !modules
            .module(&key)
            .is_some_and(|found| std::ptr::eq(found, module))
        {
            return Err(bad("source_association"));
        }
        let types = module
            .types
            .iter()
            .map(crate::relation_admission::relation_schema_definition_metadata)
            .collect::<Vec<_>>();
        entries
            .push(json!([key,{"specification_id":module.specification.to_string(),"types":types}]));
    }
    if entries.len() != modules.len() {
        return Err(bad("source_association"));
    }
    if let Some(claim) = proposal {
        let rows = claim.as_array().ok_or_else(|| bad("invalid_map_entry"))?;
        let mut keys = BTreeSet::new();
        for row in rows {
            let key = row[0].as_str().ok_or_else(|| bad("invalid_map_entry"))?;
            af_idl::SpecificationId::parse(key).map_err(|_| bad("invalid_map_entry"))?;
            if row.as_array().is_none_or(|p| p.len() != 2) || row[1]["specification_id"] != key {
                return Err(bad("invalid_map_entry"));
            }
            validate_structural_shape_metadata("CompiledModuleContents", &row[1])?;
            if !keys.insert(key) {
                return Err(bad("duplicate_map_key"));
            }
        }
        let mut supplied = rows.clone();
        supplied.sort_by(|a, b| a[0].as_str().cmp(&b[0].as_str()));
        if supplied != entries {
            return Err(bad("source_association"));
        }
    }
    Ok(json!({"entry_count":entries.len(),"entries":entries,"assurance":"module_projection_only"}))
}

fn codec_metadata(input: &Value) -> MetaResult<Value> {
    use af_core::{RecordShape, Shape, Value as V, VariantShape};
    let kind = input["kind"]
        .as_str()
        .ok_or_else(|| unsupported("invalid_shape"))?;
    let (shape, value) = if kind == "map" {
        let entry = Shape::Record(RecordShape::new(
            "StructuralMapEntry",
            vec![("key", Shape::UInt64), ("value", Shape::UInt64)],
        ));
        let mut entries = Vec::new();
        for row in input["entries"]
            .as_array()
            .ok_or_else(|| bad("invalid_map_entry"))?
        {
            let k = row[0].as_u64().ok_or_else(|| bad("invalid_map_entry"))?;
            let v = row[1].as_u64().ok_or_else(|| bad("invalid_map_entry"))?;
            entries.push(V::Record(vec![
                ("key".into(), V::UInt64(k)),
                ("value".into(), V::UInt64(v)),
            ]));
        }
        (
            Shape::Variant(VariantShape::new(
                "NodeBodyMap",
                vec![("map", Shape::List(Box::new(entry)))],
            )),
            V::Variant {
                case: "map".into(),
                value: Box::new(V::List(entries)),
            },
        )
    } else {
        if !matches!(kind, "input" | "local") {
            return Err(unsupported("invalid_shape"));
        }
        (
            Shape::Variant(VariantShape::new(
                "StructuralWEdge",
                vec![("local", Shape::UInt64), ("input", Shape::UInt64)],
            )),
            V::Variant {
                case: kind.into(),
                value: Box::new(V::UInt64(
                    input["rank"].as_u64().ok_or_else(|| bad("invalid_child"))?,
                )),
            },
        )
    };
    let bytes = af_core::encode(&shape, &value).map_err(|_| unsupported("invalid_shape"))?;
    Ok(json!({"canonical_hex":hex::encode(bytes)}))
}

/// Explicit finite registration inspector; it cannot bind arbitrary Raw inputs.
#[allow(clippy::too_many_lines)]
pub fn inspect_structural_registration(input: &Value) -> MetaResult<Value> {
    structural_descriptor_integrity()
        .map_err(|e| StructuralMetadataFailure::new(e, "missing_site"))?;
    let m = catalogue()?;
    let reg = &m["structural_binding_registration"];
    let op = input["op"]
        .as_str()
        .ok_or_else(|| unsupported("invalid_shape"))?;
    match op {
        "rule" => {
            static RULES: OnceLock<Value> = OnceLock::new();
            let rules = RULES.get_or_init(|| {
                serde_json::from_str(include_str!("../../../spec/registry/rules.json"))
                    .unwrap_or(Value::Null)
            });
            let row = rules["rules"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|r| r["rule_id"] == input["rule_id"])
                .ok_or_else(|| {
                    StructuralMetadataFailure::new(
                        RegisteredModelFailure::IncompleteContent,
                        "missing_site",
                    )
                })?;
            return Ok(
                json!({"registered_rule":row,"compiler_semantics":crate::ROOT_EXPORT_ANALYSIS_SEMANTICS_VERSION,"positive_capability":"NotAuthorized"}),
            );
        }
        "shape" => {
            validate_structural_shape_metadata(
                input["type"]
                    .as_str()
                    .ok_or_else(|| unsupported("unknown_nominal_type"))?,
                &input["value"],
            )?;
        }
        "site" => {
            validate_structural_shape_metadata("StructuralSiteDefinition", &input["value"])?;
            let expected = reg["sites"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|r| r["definition"]["key"] == input["value"]["key"])
                .ok_or_else(|| unsupported("missing_site"))?;
            if expected["definition"] != input["value"] {
                return Err(bad("missing_site"));
            }
        }
        "reference" => {
            validate_structural_shape_metadata("StructuralReferencePolicy", &input["value"])?;
            if !reg["reference_policies"]
                .as_array()
                .is_some_and(|a| a.contains(&input["value"]))
            {
                return Err(bad("invalid_child"));
            }
        }
        "site_registry" | "reference_registry" | "field_roster" | "stage" => {
            let expected = match op {
                "site_registry" => Value::Array(
                    reg["sites"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .map(|r| r["definition"].clone())
                        .collect(),
                ),
                "reference_registry" => reg["reference_policies"].clone(),
                "field_roster" => field_roster()?,
                _ => reg["stage_contract"].clone(),
            };
            if input["value"] != expected {
                return Err(StructuralMetadataFailure::new(
                    RegisteredModelFailure::IncompleteContent,
                    "missing_site",
                ));
            }
        }
        "production" => {
            return Err(StructuralMetadataFailure::new(
                RegisteredModelFailure::NotImplemented,
                "production_not_implemented",
            ));
        }
        "codec" => return codec_metadata(input),
        "module_map" => {
            let mut sources = Vec::new();
            for s in input["sources"]
                .as_array()
                .ok_or_else(|| bad("invalid_map_entry"))?
            {
                sources.push((
                    s["path"]
                        .as_str()
                        .ok_or_else(|| bad("invalid_map_entry"))?
                        .to_owned(),
                    s["text"]
                        .as_str()
                        .ok_or_else(|| bad("invalid_map_entry"))?
                        .to_owned(),
                ));
            }
            return inspect_structural_module_map(&sources, input.get("proposal"));
        }
        "dag" | "joint_omission" => {
            static SOURCES: OnceLock<Value> = OnceLock::new();
            let sources =
                SOURCES.get_or_init(|| serde_json::from_str(FIXTURES).unwrap_or(Value::Null));
            let fixture = input["fixture"]
                .as_str()
                .ok_or_else(|| unsupported("missing_site"))?;
            let declared = sources
                .get(fixture)
                .ok_or_else(|| unsupported("missing_site"))?;
            if op == "dag" {
                crate::structural_graph_metadata::check_subgraph(declared, &input["proposal"])?;
            } else {
                crate::structural_graph_metadata::check_subgraph(
                    declared,
                    &input["expected_view"],
                )?;
                crate::structural_graph_metadata::check_subgraph(declared, &input["actual_view"])?;
            }
        }
        _ => return Err(unsupported("invalid_shape")),
    }
    Ok(Value::Null)
}
