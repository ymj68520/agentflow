//! Executable field contract and independent sidecars, test build only.
//! Kept outside src so the unchanged production static-surface scan stays closed.

use crate::PortableProofCodec;
use serde_json::{Value as Json, json};
use sha2::Digest as _;
use std::path::{Path, PathBuf};

mod d10_gap_conformance;
mod d9_recovery_conformance;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_json(path: &str) -> Json {
    serde_json::from_slice(&std::fs::read(repo().join(path)).expect("registered conformance asset"))
        .expect("conformance JSON")
}

#[derive(Clone, Default, Debug)]
struct Count {
    required: usize,
    diagnostic: usize,
}

fn validate(
    value: &Json,
    schema: &Json,
    definitions: &Json,
    path: &str,
    depth: usize,
    count: &mut Count,
) -> Result<(), String> {
    if depth > 256 {
        return Err(format!("{path}: CoverageResourceLimit"));
    }
    let kind = schema["kind"]
        .as_str()
        .ok_or_else(|| format!("{path}: unregistered schema kind"))?;
    if kind == "ref" {
        return validate(
            value,
            &definitions[schema["name"].as_str().unwrap()],
            definitions,
            path,
            depth + 1,
            count,
        );
    }
    if kind == "union" {
        let mut errors = Vec::new();
        for alternative in schema["alternatives"].as_array().unwrap() {
            let mut trial = count.clone();
            match validate(value, alternative, definitions, path, depth + 1, &mut trial) {
                Ok(()) => {
                    *count = trial;
                    return Ok(());
                }
                Err(error) => errors.push(error),
            }
        }
        return Err(format!("{path}: wrong variant: {}", errors.join(" | ")));
    }
    if schema["classification"] == "DiagnosticOnly" {
        count.diagnostic += 1;
    } else {
        count.required += 1;
    }
    match kind {
        "record" => {
            let fields = schema["fields"].as_object().unwrap();
            let actual = value
                .as_object()
                .ok_or_else(|| format!("{path}: record required"))?;
            for key in fields.keys() {
                if !actual.contains_key(key) {
                    return Err(format!("{path}.{key}: missing required field"));
                }
            }
            for key in actual.keys() {
                if !fields.contains_key(key) {
                    return Err(format!("{path}.{key}: unexpected field"));
                }
            }
            for (key, field) in fields {
                validate(
                    &actual[key],
                    field,
                    definitions,
                    &format!("{path}.{key}"),
                    depth + 1,
                    count,
                )?;
            }
            if let Some(constraints) = schema["constraints"].as_array() {
                if constraints.contains(&json!("owned_children")) {
                    let children = value["children"].as_array().unwrap();
                    let ids = children
                        .iter()
                        .map(|child| child["computation"].clone())
                        .collect::<Vec<_>>();
                    let declared = value["direct_children"].as_array().unwrap();
                    let unique = declared
                        .iter()
                        .map(|id| id.as_u64().unwrap())
                        .collect::<std::collections::BTreeSet<_>>();
                    if &ids != declared || unique.len() != declared.len() {
                        return Err(format!(
                            "{path}.children: checked ownership identity/order mismatch"
                        ));
                    }
                }
                if constraints.contains(&json!("leaf_identity")) {
                    let leaf = &value["leaf_summary"];
                    let is_leaf = matches!(
                        value["operator"].as_str().unwrap(),
                        "invoke" | "await" | "call_subflow"
                    );
                    if is_leaf == leaf.is_null()
                        || is_leaf
                            && (!value["children"].as_array().unwrap().is_empty()
                                || leaf["computation"] != value["computation"]
                                || leaf["kind"] != value["operator"]
                                || leaf["caller_semantic_ir_digest"]
                                    != value["source_semantic_ir_digest"])
                    {
                        return Err(format!(
                            "{path}.leaf_summary: exact leaf identity/availability mismatch"
                        ));
                    }
                }
            }
        }
        "array" => {
            let items = value
                .as_array()
                .ok_or_else(|| format!("{path}: array required"))?;
            if let Some(length) = schema["exact_length"].as_u64() {
                if items.len() as u64 != length {
                    return Err(format!("{path}: exact typed array length required"));
                }
            }
            if let Some(key) = schema["unique_by"].as_str() {
                let unique = items
                    .iter()
                    .map(|item| serde_json::to_string(&item[key]).unwrap())
                    .collect::<std::collections::BTreeSet<_>>();
                if unique.len() != items.len() {
                    return Err(format!("{path}: duplicated nominal member"));
                }
            }
            for (index, item) in items.iter().enumerate() {
                validate(
                    item,
                    &schema["element"],
                    definitions,
                    &format!("{path}[{index}]"),
                    depth + 1,
                    count,
                )?;
            }
        }
        "tuple" => {
            let items = value
                .as_array()
                .ok_or_else(|| format!("{path}: tuple required"))?;
            let schemas = schema["elements"].as_array().unwrap();
            if items.len() != schemas.len() {
                return Err(format!("{path}: wrong tuple arity"));
            }
            for (index, (item, field)) in items.iter().zip(schemas).enumerate() {
                validate(
                    item,
                    field,
                    definitions,
                    &format!("{path}[{index}]"),
                    depth + 1,
                    count,
                )?;
            }
        }
        "nullable" => {
            if !value.is_null() {
                validate(value, &schema["value"], definitions, path, depth + 1, count)?;
            }
        }
        "literal" if value != &schema["value"] => {
            return Err(format!("{path}: exact literal required"));
        }
        "enum" if !schema["values"].as_array().unwrap().contains(value) => {
            return Err(format!("{path}: exact enum member required"));
        }
        "u64" if value.as_u64().is_none() => {
            return Err(format!("{path}: U64 type/width violation"));
        }
        "u8" if value
            .as_u64()
            .is_none_or(|number| u8::try_from(number).is_err()) =>
        {
            return Err(format!("{path}: U8 type/width violation"));
        }
        "u32"
            if value
                .as_u64()
                .is_none_or(|number| u32::try_from(number).is_err()) =>
        {
            return Err(format!("{path}: U32 type/width violation"));
        }
        "i64" if value.as_i64().is_none() => {
            return Err(format!("{path}: I64 type/width violation"));
        }
        "bool" if !value.is_boolean() => return Err(format!("{path}: bool required")),
        "string" if !value.is_string() => return Err(format!("{path}: string required")),
        "null" if !value.is_null() => return Err(format!("{path}: null required")),
        "string"
            if schema.get("pattern").is_some()
                && (value.as_str().unwrap().len() != 64
                    || !value
                        .as_str()
                        .unwrap()
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))) =>
        {
            return Err(format!("{path}: canonical nominal string pattern mismatch"));
        }
        "literal" | "enum" | "u8" | "u32" | "u64" | "i64" | "bool" | "string" | "null" => {}
        _ => return Err(format!("{path}: schema kind not implemented: {kind}")),
    }
    Ok(())
}

fn canonical(value: &Json, schema: &Json, defs: &Json) -> Json {
    let kind = schema["kind"].as_str().unwrap();
    if kind == "ref" {
        return canonical(value, &defs[schema["name"].as_str().unwrap()], defs);
    }
    if kind == "union" {
        let alternative = schema["alternatives"]
            .as_array()
            .unwrap()
            .iter()
            .find(|alternative| {
                validate(value, alternative, defs, "$", 0, &mut Count::default()).is_ok()
            })
            .expect("validated variant");
        return canonical(value, alternative, defs);
    }
    if schema["classification"] == "DiagnosticOnly" {
        return json!("DiagnosticOnly");
    }
    match kind {
        "record" => Json::Object(
            schema["fields"]
                .as_object()
                .unwrap()
                .iter()
                .map(|(key, field)| (key.clone(), canonical(&value[key], field, defs)))
                .collect(),
        ),
        "array" => {
            let mut items = value
                .as_array()
                .unwrap()
                .iter()
                .map(|item| canonical(item, &schema["element"], defs))
                .collect::<Vec<_>>();
            if schema["ordering"] == "semantic_multiset" {
                items.sort_by_cached_key(|item| serde_json::to_string(item).unwrap());
            }
            Json::Array(items)
        }
        "nullable" if !value.is_null() => canonical(value, &schema["value"], defs),
        "tuple" => Json::Array(
            value
                .as_array()
                .unwrap()
                .iter()
                .zip(schema["elements"].as_array().unwrap())
                .map(|(item, field)| canonical(item, field, defs))
                .collect(),
        ),
        _ => value.clone(),
    }
}

fn difference(expected: &Json, actual: &Json, path: &str) -> Option<String> {
    match (expected, actual) {
        (Json::Object(left), Json::Object(right)) => {
            for (key, item) in left {
                let Some(other) = right.get(key) else {
                    return Some(format!("{path}.{key}: missing Actual variant member"));
                };
                if let Some(diff) = difference(item, other, &format!("{path}.{key}")) {
                    return Some(diff);
                }
            }
            if let Some(key) = right.keys().find(|key| !left.contains_key(*key)) {
                return Some(format!("{path}.{key}: unexpected Actual variant member"));
            }
            None
        }
        (Json::Array(left), Json::Array(right)) => {
            if left.len() != right.len() {
                return Some(format!(
                    "{path}.length expected={} actual={}",
                    left.len(),
                    right.len()
                ));
            }
            for (index, (x, y)) in left.iter().zip(right).enumerate() {
                if let Some(diff) = difference(x, y, &format!("{path}[{index}]")) {
                    return Some(diff);
                }
            }
            None
        }
        _ if expected == actual => None,
        _ => Some(format!("{path} expected={expected} actual={actual}")),
    }
}

fn compare(expected: &Json, actual: &Json, contract: &Json) -> Result<(Count, Count), String> {
    let defs = &contract["projection_definitions"];
    let schema = &defs["D9Facts"];
    let mut left = Count::default();
    let mut right = Count::default();
    validate(expected, schema, defs, "$", 0, &mut left)
        .map_err(|error| format!("Expected: {error}"))?;
    validate(actual, schema, defs, "$", 0, &mut right)
        .map_err(|error| format!("Actual: {error}"))?;
    if let Some(diff) = difference(
        &canonical(expected, schema, defs),
        &canonical(actual, schema, defs),
        "$",
    ) {
        return Err(diff);
    }
    Ok((left, right))
}

#[test]
fn independent_d9_control_full_fields_match_22_production_cases() {
    let contract = read_json(
        "spec/conformance/portable-root-export-revalidation/coverage/field-coverage.json",
    );
    let manifest = read_json(
        "spec/conformance/portable-root-export-revalidation/expected-full/d9-control/manifest.json",
    );
    let cases = manifest["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 22);
    let output = repo().join("target/conformance/d9-full");
    std::fs::create_dir_all(&output).unwrap();
    let mut results = Vec::new();
    for chunk in cases.chunks(3) {
        let handles=chunk.iter().map(|case|{
            let case=case.clone();let contract=contract.clone();let output=output.clone();
            std::thread::spawn(move ||{
                let bytes=std::fs::read(repo().join(case["bundle_file"].as_str().unwrap())).unwrap();
                let analysis=PortableProofCodec::default().validate_supported(&bytes).unwrap_or_else(|error|panic!("{} D0-D4: {error}",case["case_id"]))
                    .revalidate_root_export().unwrap_or_else(|error|panic!("{} D5-D10: {error}",case["case_id"]));
                let actual=json!({"case_id":case["case_id"],"input_identity":{"canonical_bytes":bytes.len(),"canonical_sha256":hex::encode(sha2::Sha256::digest(&bytes))},"production_stage":"D10","context":analysis.inputs().context(),"d9_root":analysis.root_summary().typed_facts(),"d9_root_recovery":analysis.root_recovery().typed_facts()});
                let d10=analysis.typed_projection()["d10_root_export"].clone();
                validate(&d10,&contract["projection_definitions"]["RootExportReport"],&contract["projection_definitions"],"$.d10",0,&mut Count::default()).unwrap_or_else(|error|panic!("D10 shape inventory (value Oracle Pending): {error}"));
                let slug=case["case_id"].as_str().unwrap().rsplit('/').next().unwrap();
                std::fs::write(output.join(format!("{slug}.actual.json")),serde_json::to_vec_pretty(&actual).unwrap()).unwrap();
                let expected=read_json(case["expected_file"].as_str().unwrap());
                let (left,right)=compare(&expected["facts"],&actual,&contract).unwrap_or_else(|error|panic!("case={} field={error}",case["case_id"]));
                assert_eq!(left.required,usize::try_from(expected["instance_coverage"]["required"].as_u64().unwrap()).unwrap());
                assert_eq!(left.diagnostic,usize::try_from(expected["instance_coverage"]["diagnostic"].as_u64().unwrap()).unwrap());
                json!({"case_id":case["case_id"],"expected_required":left.required,"actual_required":right.required,"compared_required":left.required,"diagnostic_instances":left.diagnostic,"missing_expected":0,"missing_actual":0,"unexpected":0,"uncompared":0,"d9_full_field":"passed","d10_full_field":"Pending"})
            })
        }).collect::<Vec<_>>();
        for handle in handles {
            results.push(handle.join().expect("D9 differential worker"));
        }
    }
    assert_eq!(results.len(), 22);
    std::fs::write(output.join("report.json"),serde_json::to_vec_pretty(&json!({"executed":22,"fully_compared":22,"skipped":0,"unexpected_mismatch":0,"cases":results})).unwrap()).unwrap();
}

#[test]
fn executable_contract_rejects_both_sides_missing_and_unregistered_fields() {
    let contract = read_json(
        "spec/conformance/portable-root-export-revalidation/coverage/field-coverage.json",
    );
    let expected=read_json("spec/conformance/portable-root-export-revalidation/expected-full/d9-control/d9-opcode-invoke.json")["facts"].clone();
    let mut both = expected.clone();
    both["d9_root"]["root_control"]
        .as_object_mut()
        .unwrap()
        .remove("operator");
    let failure = compare(&both, &both, &contract).unwrap_err();
    assert!(
        failure.contains("operator: missing required field"),
        "{failure}"
    );
    let mut extra = expected.clone();
    extra["d9_root"]["root_control"]["new_semantic_field"] = json!(1);
    let failure = compare(&extra, &extra, &contract).unwrap_err();
    assert!(
        failure.contains("new_semantic_field: unexpected field"),
        "{failure}"
    );
    let mut wide = expected.clone();
    wide["d9_root"]["root_control"]["computation"] = json!(1u64 << 40);
    assert!(
        compare(&wide, &wide, &contract)
            .unwrap_err()
            .contains("U32")
    );
    let mut byte = read_json("spec/conformance/portable-root-export-revalidation/input-mutations/effect-operation-changed.expected.json")["facts"].clone();
    byte["context"]["effect_resource_relation"]["canonical_content"][0] = json!(256);
    assert!(compare(&byte, &byte, &contract).unwrap_err().contains("U8"));
    let mut timeout=read_json("spec/conformance/portable-root-export-revalidation/expected-full/d9-control/d9-opcode-timeout.json")["facts"].clone();
    timeout["d9_root"]["root_control"]["operator_semantics"]["possible_outcomes"]
        .as_array_mut()
        .unwrap()
        .pop();
    assert!(
        compare(&timeout, &timeout, &contract)
            .unwrap_err()
            .contains("array length")
    );
}

fn replace_path(value: &mut Json, path: &[Json], replacement: Json) {
    let mut current = value;
    for token in &path[..path.len() - 1] {
        current = match current {
            Json::Object(fields) => fields
                .get_mut(token.as_str().unwrap())
                .expect("declared field"),
            Json::Array(items) => &mut items[token.as_str().unwrap().parse::<usize>().unwrap()],
            _ => panic!("mutation path traverses scalar"),
        };
    }
    let token = path.last().unwrap().as_str().unwrap();
    match current {
        Json::Object(fields) => {
            *fields.get_mut(token).expect("declared field") = replacement;
        }
        Json::Array(items) => items[token.parse::<usize>().unwrap()] = replacement,
        _ => panic!("mutation parent scalar"),
    }
}

#[test]
fn independent_expected_detects_42_new_semantic_mutations() {
    let contract = read_json(
        "spec/conformance/portable-root-export-revalidation/coverage/field-coverage.json",
    );
    let vectors = read_json(
        "spec/conformance/portable-root-export-revalidation/coverage/comparator-mutations.json",
    );
    let rows = vectors["mutations"].as_array().unwrap();
    assert_eq!(rows.len(), 42);
    for row in rows {
        let path=row["expected_file"].as_str().map_or_else(||format!("spec/conformance/portable-root-export-revalidation/expected-full/d9-control/{}.json",row["case_slug"].as_str().unwrap()),str::to_owned);
        let expected = read_json(&path)["facts"].clone();
        let mut actual = expected.clone();
        replace_path(
            &mut actual,
            row["path"].as_array().unwrap(),
            row["replacement"].clone(),
        );
        let error = compare(&expected, &actual, &contract)
            .expect_err("independent semantic mutation must be detected");
        let final_token = row["path"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()
            .as_str()
            .unwrap();
        assert!(
            error.contains(final_token)
                || error.contains("length")
                || error.contains("leaf_summary")
                || error.contains("children"),
            "{} precise difference: {error}",
            row["mutation_id"]
        );
    }
}

#[test]
fn frozen_type_inventory_and_python_expected_regenerate_exactly() {
    let result = std::process::Command::new("python")
        .arg(repo().join("tools/gen-portable-d9-control-full-oracle.py"))
        .arg("--check")
        .current_dir(repo())
        .output()
        .expect("independent Python oracle runtime");
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn twelve_portable_input_mutations_recompute_or_reject_at_formal_stage() {
    let contract = read_json(
        "spec/conformance/portable-root-export-revalidation/coverage/field-coverage.json",
    );
    let manifest = read_json(
        "spec/conformance/portable-root-export-revalidation/input-mutations/manifest.json",
    );
    let cases = manifest["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 12);
    let output = repo().join("target/conformance/d9-input-mutations");
    std::fs::create_dir_all(&output).unwrap();
    let mut results = Vec::new();
    for chunk in cases.chunks(3) {
        let handles=chunk.iter().map(|case|{
            let case=case.clone();let contract=contract.clone();let output=output.clone();
            std::thread::spawn(move ||{
                let bytes=std::fs::read(repo().join(case["bundle_file"].as_str().unwrap())).unwrap();
                let supported=PortableProofCodec::default().validate_supported(&bytes).unwrap_or_else(|error|panic!("{} D0-D4: {error}",case["case_id"]));
                let identity=json!({"canonical_bytes":bytes.len(),"canonical_sha256":hex::encode(sha2::Sha256::digest(&bytes))});
                let expected=read_json(case["expected_file"].as_str().unwrap());
                let actual=match supported.revalidate_root_export() {
                    Ok(analysis)=>json!({"case_id":case["case_id"],"input_identity":identity,"production_stage":"D10","context":analysis.inputs().context(),"d9_root":analysis.root_summary().typed_facts(),"d9_root_recovery":analysis.root_recovery().typed_facts()}),
                    Err(error)=>{
                        assert_eq!(error.stage,crate::ValidationStage::D6,"{} unexpected rejection: {error}",case["case_id"]);
                        json!({"case_id":case["case_id"],"input_identity":identity,"production_stage":"D6","error_family":error.family,"error_path":error.path,"d9_root":null,"capability_issued":false})
                    }
                };
                std::fs::write(output.join(format!("{}.actual.json",case["vector_id"].as_str().unwrap())),serde_json::to_vec_pretty(&actual).unwrap()).unwrap();
                if case["expected_stage"]=="D10" {
                    compare(&expected["facts"],&actual,&contract).unwrap_or_else(|error|panic!("input mutation {} field={error}",case["case_id"]));
                } else {
                    let defs=&contract["projection_definitions"];
                    let schema=&defs["EarlyRejection"];
                    validate(&expected["facts"],schema,defs,"$",0,&mut Count::default()).unwrap();
                    validate(&actual,schema,defs,"$",0,&mut Count::default()).unwrap_or_else(|error|panic!("input mutation {} field={error}",case["case_id"]));
                    assert_eq!(expected["facts"],actual);
                }
                json!({"vector_id":case["vector_id"],"stage":case["expected_stage"],"result":"passed"})
            })
        }).collect::<Vec<_>>();
        for handle in handles {
            results.push(handle.join().expect("input mutation worker"));
        }
    }
    std::fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(&json!({"executed":12,"cases":results})).unwrap(),
    )
    .unwrap();
}
