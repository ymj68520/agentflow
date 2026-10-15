//! Non-authoritative registration/shape admission. No fact, Scope identity,
//! normalizer, encoder, report, or Checked capability is constructed here.
use serde_json::Value;
use std::collections::BTreeSet;
use std::sync::OnceLock;
use unicode_normalization::UnicodeNormalization;

const SOURCE: &str = include_str!("../../../spec/tables/proof-report-model-v2.md");
pub const SUCCESSOR_MODEL_ID: &str = "agentflow.computation.proof-report.model@2";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisteredModelFailure {
    UnsupportedModel,
    IncompleteContent,
    IncompleteFacts,
    InvalidFacts,
    ResourceLimit,
    NotImplemented,
}

/// The immutable compiled normative catalogue, not caller assertions or Actual.
pub fn registered_model_catalogue() -> Result<&'static Value, RegisteredModelFailure> {
    static MODEL: OnceLock<Result<Value, RegisteredModelFailure>> = OnceLock::new();
    MODEL
        .get_or_init(|| {
            let body = SOURCE
                .split_once("```json\n")
                .ok_or(RegisteredModelFailure::IncompleteContent)?
                .1;
            let json = body
                .split_once("\n```")
                .ok_or(RegisteredModelFailure::IncompleteContent)?
                .0;
            serde_json::from_str(json).map_err(|_| RegisteredModelFailure::UnsupportedModel)
        })
        .as_ref()
        .map_err(|failure| *failure)
}

/// Checks registered descriptor closure only; this is NOT CompleteAndValid.
pub fn registered_descriptor_integrity() -> Result<(), RegisteredModelFailure> {
    let catalogue = registered_model_catalogue()?;
    if catalogue["model_id"] != SUCCESSOR_MODEL_ID
        || catalogue["analysis_owner"] != crate::ROOT_EXPORT_ANALYSIS_SEMANTICS_VERSION
        || catalogue["model_edition"] != crate::REPORT_SHAPE_EDITION
        || catalogue["obligation_edition"] != crate::OBLIGATION_IDENTITY_EDITION
    {
        return Err(RegisteredModelFailure::UnsupportedModel);
    }
    let types = catalogue["types"]
        .as_object()
        .ok_or(RegisteredModelFailure::IncompleteContent)?;
    let mut pending: Vec<&Value> = types.values().collect();
    let mut work = 0usize;
    while let Some(value) = pending.pop() {
        work += 1;
        if work > 262_144 {
            return Err(RegisteredModelFailure::ResourceLimit);
        }
        match value {
            Value::Object(o) => {
                if let Some(kind) = o.get("kind").and_then(Value::as_str) {
                    if !matches!(
                        kind,
                        "record"
                            | "enum"
                            | "sequence"
                            | "set"
                            | "map"
                            | "option"
                            | "dag_ref"
                            | "unit"
                            | "null"
                            | "bool"
                            | "nfc_text"
                            | "bytes"
                            | "u8"
                            | "u32"
                            | "u64"
                            | "i32"
                            | "i64"
                            | "decimal"
                            | "finite_f64"
                            | "timestamp_i64"
                            | "duration_i64"
                    ) {
                        return Err(RegisteredModelFailure::UnsupportedModel);
                    }
                    let member_field = match kind {
                        "record" => Some("fields"),
                        "enum" => Some("variants"),
                        _ => None,
                    };
                    if let Some(field) = member_field {
                        let members = o
                            .get(field)
                            .and_then(Value::as_array)
                            .ok_or(RegisteredModelFailure::IncompleteContent)?;
                        let names = members
                            .iter()
                            .filter_map(|row| row[0].as_str())
                            .collect::<BTreeSet<_>>();
                        if names.len() != members.len()
                            || members
                                .iter()
                                .any(|row| row.as_array().is_none_or(|pair| pair.len() != 2))
                        {
                            return Err(RegisteredModelFailure::UnsupportedModel);
                        }
                    }
                }
                if let Some(name) = o.get("ref") {
                    if !name.as_str().is_some_and(|n| types.contains_key(n)) {
                        return Err(RegisteredModelFailure::IncompleteContent);
                    }
                }
                if o.contains_key("_generic") || o.contains_key("_case") {
                    return Err(RegisteredModelFailure::IncompleteContent);
                }
                pending.extend(o.values());
            }
            Value::Array(array) => pending.extend(array),
            _ => {}
        }
    }
    for (field, count, key) in [
        ("case_descriptors", 27, "case"),
        ("dimension_descriptors", 12, "dimension"),
    ] {
        let rows = catalogue[field]
            .as_array()
            .ok_or(RegisteredModelFailure::IncompleteContent)?;
        let ids = rows
            .iter()
            .filter_map(|row| row[key].as_str())
            .collect::<BTreeSet<_>>();
        if rows.len() != count || ids.len() != count {
            return Err(RegisteredModelFailure::IncompleteContent);
        }
    }
    Ok(())
}

/// A metadata-only shape probe. Success is unit, not a trusted Fact Seed,
/// source validation, canonical identity, Scope binding, or proof capability.
pub fn validate_registered_model_shape(
    name: &str,
    value: &Value,
) -> Result<(), RegisteredModelFailure> {
    registered_descriptor_integrity()?;
    let catalogue = registered_model_catalogue()?;
    let descriptor = catalogue["types"]
        .get(name)
        .ok_or(RegisteredModelFailure::UnsupportedModel)?;
    let mut work = 0;
    check_shape(catalogue, descriptor, value, 0, &mut work)?;
    check_named_constraints(catalogue, name, value)
}

fn check_named_constraints(
    catalogue: &Value,
    name: &str,
    value: &Value,
) -> Result<(), RegisteredModelFailure> {
    if name == "ContextRolePartition" {
        let roles = value
            .as_array()
            .ok_or(RegisteredModelFailure::IncompleteFacts)?
            .iter()
            .flat_map(|case| case.as_array().into_iter().flatten())
            .filter_map(Value::as_str)
            .collect::<Vec<_>>();
        let set = roles.iter().copied().collect::<BTreeSet<_>>();
        if roles.len() != 3
            || set != BTreeSet::from(["analysis", "old_checked", "replacement_checked"])
        {
            return Err(RegisteredModelFailure::IncompleteFacts);
        }
        let classes = value
            .as_array()
            .ok_or(RegisteredModelFailure::IncompleteFacts)?;
        let member_class = |role: &str| {
            classes.iter().position(|case| {
                case.as_array()
                    .is_some_and(|array| array.iter().any(|value| value == role))
            })
        };
        if member_class("analysis") != member_class("replacement_checked") {
            return Err(RegisteredModelFailure::InvalidFacts);
        }
    }
    if name == "DependencyErrorPayload" {
        let domain = value["domain"]
            .as_str()
            .ok_or(RegisteredModelFailure::IncompleteFacts)?;
        if value["kind"]
            .as_object()
            .is_none_or(|o| !o.contains_key(domain))
        {
            return Err(RegisteredModelFailure::InvalidFacts);
        }
    }
    if matches!(name, "ModelRef" | "ModelType") {
        let owner = value["owner"]
            .as_str()
            .ok_or(RegisteredModelFailure::IncompleteFacts)?;
        let ty = value["name"]
            .as_str()
            .ok_or(RegisteredModelFailure::IncompleteFacts)?;
        let qualified = format!("{owner}/{ty}");
        if !((owner == SUCCESSOR_MODEL_ID && catalogue["types"].get(ty).is_some())
            || catalogue["types"].get(&qualified).is_some())
        {
            return Err(RegisteredModelFailure::UnsupportedModel);
        }
    }
    Ok(())
}

fn check_shape(
    catalogue: &Value,
    descriptor: &Value,
    value: &Value,
    depth: usize,
    work: &mut usize,
) -> Result<(), RegisteredModelFailure> {
    *work += 1;
    if depth > 96 || *work > 262_144 {
        return Err(RegisteredModelFailure::ResourceLimit);
    }
    if let Some(name) = descriptor["ref"].as_str() {
        check_shape(
            catalogue,
            catalogue["types"]
                .get(name)
                .ok_or(RegisteredModelFailure::IncompleteContent)?,
            value,
            depth + 1,
            work,
        )?;
        return check_named_constraints(catalogue, name, value);
    }
    let bad = RegisteredModelFailure::UnsupportedModel;
    let missing = RegisteredModelFailure::IncompleteFacts;
    match descriptor["kind"].as_str().ok_or(bad)? {
        "unit" | "null" => {
            if !value.is_null() {
                return Err(bad);
            }
        }
        "bool" => {
            if !value.is_boolean() {
                return Err(bad);
            }
        }
        "u8" | "u32" | "u64" | "dag_ref" => {
            let n = value.as_u64().ok_or(bad)?;
            if (descriptor["kind"] == "u8" && n > 255)
                || (descriptor["kind"] == "u32" && n > u64::from(u32::MAX))
            {
                return Err(bad);
            }
        }
        "i32" | "i64" | "timestamp_i64" | "duration_i64" => {
            let n = value.as_i64().ok_or(bad)?;
            if descriptor["kind"] == "i32" && i32::try_from(n).is_err() {
                return Err(bad);
            }
        }
        "finite_f64" => {
            if !value.as_f64().is_some_and(f64::is_finite) {
                return Err(bad);
            }
        }
        "decimal" => {
            if value
                .as_str()
                .is_none_or(|source| af_core::Decimal::parse(source).is_err())
            {
                return Err(bad);
            }
        }
        "nfc_text" => {
            let source = value.as_str().ok_or(bad)?;
            if !source.nfc().eq(source.chars()) {
                return Err(bad);
            }
        }
        // Metadata probes represent bytes as an explicit array, not an ACE1
        // serializer. Canonical G01 bytes are deliberately NOT produced here.
        "bytes" => {
            let array = value.as_array().ok_or(bad)?;
            if array.iter().any(|x| x.as_u64().is_none_or(|n| n > 255)) {
                return Err(bad);
            }
            if let Some(n) = descriptor["length"].as_u64() {
                if array.len() as u64 != n {
                    return Err(bad);
                }
            }
        }
        "record" => {
            let o = value.as_object().ok_or(bad)?;
            let fields = descriptor["fields"].as_array().ok_or(bad)?;
            if fields
                .iter()
                .any(|f| !f[0].as_str().is_some_and(|source| o.contains_key(source)))
            {
                return Err(missing);
            }
            if o.len() != fields.len() {
                return Err(bad);
            }
            for f in fields {
                check_shape(
                    catalogue,
                    &f[1],
                    &o[f[0].as_str().ok_or(bad)?],
                    depth + 1,
                    work,
                )?;
            }
        }
        "enum" => {
            let rows = descriptor["variants"].as_array().ok_or(bad)?;
            let simple = rows.iter().all(|row| row[1]["kind"] == "unit");
            if simple {
                let source = value.as_str().ok_or(bad)?;
                if !rows.iter().any(|row| row[0] == source) {
                    return Err(bad);
                }
            } else {
                let o = value.as_object().ok_or(bad)?;
                if o.len() != 1 {
                    return Err(bad);
                }
                let (tag, payload) = o.iter().next().ok_or(bad)?;
                let row = rows.iter().find(|row| row[0] == tag.as_str()).ok_or(bad)?;
                check_shape(catalogue, &row[1], payload, depth + 1, work)?;
            }
        }
        "option" => {
            let o = value.as_object().ok_or(bad)?;
            if o.len() != 1 {
                return Err(bad);
            }
            if let Some(n) = o.get("none") {
                if !n.is_null() {
                    return Err(bad);
                }
            } else if let Some(source) = o.get("some") {
                check_shape(catalogue, &descriptor["item"], source, depth + 1, work)?;
            } else {
                return Err(bad);
            }
        }
        "sequence" | "set" => {
            let array = value.as_array().ok_or(bad)?;
            if (array.len() as u64) < descriptor["minimum"].as_u64().unwrap_or(0) {
                return Err(missing);
            }
            if let Some(n) = descriptor["length"].as_u64() {
                if array.len() as u64 != n {
                    return Err(bad);
                }
            }
            for (i, x) in array.iter().enumerate() {
                if descriptor["kind"] == "set" && array[..i].contains(x) {
                    return Err(RegisteredModelFailure::InvalidFacts);
                }
                check_shape(catalogue, &descriptor["item"], x, depth + 1, work)?;
            }
        }
        "map" => {
            let array = value.as_array().ok_or(bad)?;
            let mut keys = Vec::new();
            for pair in array {
                let path = pair.as_array().filter(|path| path.len() == 2).ok_or(bad)?;
                if keys.contains(&&path[0]) {
                    return Err(RegisteredModelFailure::InvalidFacts);
                }
                keys.push(&path[0]);
                check_shape(catalogue, &descriptor["key"], &path[0], depth + 1, work)?;
                check_shape(catalogue, &descriptor["value"], &path[1], depth + 1, work)?;
            }
        }
        _ => return Err(bad),
    }
    Ok(())
}

/// Recognizing array registered profile still refuses all complete proof support.
/// A standalone request is never array RootExport CompleteView conversion.
pub fn admit_registered_profile(
    profile: &str,
    as_root: bool,
) -> Result<(), RegisteredModelFailure> {
    registered_descriptor_integrity()?;
    if !matches!(
        profile,
        "root_export" | "contract_validation" | "computation" | "standalone_substitution"
    ) || (profile == "standalone_substitution" && as_root)
    {
        return Err(RegisteredModelFailure::UnsupportedModel);
    }
    Err(RegisteredModelFailure::NotImplemented)
}
