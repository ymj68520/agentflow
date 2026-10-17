#![recursion_limit = "512"]
use af_computation::*;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
#[allow(dead_code)]
mod support;
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn read(name: &str) -> Value {
    serde_json::from_slice(
        &std::fs::read(
            root()
                .join("spec/conformance/g01-document-occurrence-staging")
                .join(name),
        )
        .unwrap(),
    )
    .unwrap()
}
#[test]
fn every_occurrence_shape_and_bidirectional_field_case_executes() {
    let corpus = read("shape-cases.json");
    let mut actual = Vec::new();
    for case in corpus["cases"].as_array().unwrap() {
        let outcome =
            validate_registered_model_shape(case["type"].as_str().unwrap(), &case["value"])
                .err()
                .map(|e| format!("{e:?}"));
        assert_eq!(
            outcome.as_deref(),
            case["expected_failure"].as_str(),
            "{}",
            case["case_id"]
        );
        actual.push(json!({"case_id":case["case_id"],"actual_failure":outcome}));
    }
    for site in corpus["coverage"].as_array().unwrap() {
        assert!(actual.iter().any(|a| a["case_id"] == site["case_id"]));
    }
    std::fs::write(
        root().join("target/phase-04c1b4c4r3c2g2-shape-actual.json"),
        serde_json::to_vec_pretty(&actual).unwrap(),
    )
    .unwrap();
}
#[test]
fn exact_registration_identity_and_all_complete_support_gates_deny() {
    let e = current_registered_analysis_edition();
    assert_eq!(
        e.compiler.analysis_semantics_version,
        "agentflow.root-export-checker@6"
    );
    assert_eq!(
        (
            e.compiler.specifications.len(),
            e.compiler.rules.len(),
            e.registered_sources.len()
        ),
        (36, 188, 62)
    );
    assert_eq!(e.registration, EditionRegistration::Registered);
    assert_eq!(e.support, EditionSupport::NotImplemented);
    assert_eq!(e.activation, EditionActivation::NotAuthorized);
    assert_eq!(e.release, EditionRelease::NotAuthorized);
    assert!(!e.can_issue_capability());
    check_registered_compiler_identity(&e.compiler).unwrap();
    for n in 1..=5 {
        let mut wrong = e.compiler.clone();
        wrong.analysis_semantics_version = format!("agentflow.root-export-checker@{n}");
        assert!(check_registered_compiler_identity(&wrong).is_err());
    }
    for profile in [
        "raw_contract_validation",
        "root_export",
        "native_recovery_guard",
        "standalone_substitution",
    ] {
        assert_eq!(
            admit_registered_profile(profile, false),
            Err(RegisteredModelFailure::NotImplemented)
        );
    }
    assert_eq!(
        admit_registered_profile("raw_contract_validation", true),
        Err(RegisteredModelFailure::UnsupportedModel)
    );
    assert!(
        check_registered_report_editions(
            "agentflow.g01.report-shape@4",
            "agentflow.g01.obligation@4"
        )
        .is_err()
    );
}
#[test]
fn frozen_model_and_raw_identity_reference_closure_are_exact() {
    let m = registered_model_catalogue().unwrap();
    let oldtext =
        std::fs::read_to_string(root().join("spec/tables/proof-report-model-v4.md")).unwrap();
    let old: Value = serde_json::from_str(
        oldtext
            .split_once("```json\n")
            .unwrap()
            .1
            .split_once("\n```")
            .unwrap()
            .0,
    )
    .unwrap();
    assert_eq!(m["inherited_registered_model"], old);
    assert_eq!(
        m["historical_selected_imports"],
        old["historical_selected_imports"]
    );
    assert_eq!(
        m["historical_import_sources"],
        old["historical_import_sources"]
    );
    assert_eq!(m["types"]["RawDocument"], old["types"]["RawDocument"]);
    assert_eq!(m["raw_case_descriptors"], old["raw_case_descriptors"]);
    assert_eq!(m["native_case_descriptors"], old["native_case_descriptors"]);
    assert_eq!(m["case_descriptors"], old["case_descriptors"]);
    assert_eq!(
        m["types"]["RawInputOrigin"]["fields"][0],
        json!(["document_binding",{"kind":"dag_ref","target":{"ref":"DocumentOccurrenceBinding"}}])
    );
    assert_eq!(
        m["types"]["RawValidationOrigin"]["fields"][0],
        json!(["input_origin",{"ref":"RawInputOrigin"}])
    );
    for name in [
        "RawParsedInput",
        "RawValidationSubject",
        "Source",
        "Origin",
        "FactKey",
        "RawPortCheckBasis",
        "RawPortRejectionWitness",
        "ObservationSite",
    ] {
        assert_eq!(
            m["types"][name], old["types"][name],
            "{name} retains its exact references into the new closure"
        );
    }
    assert_eq!(
        m["types"]["DocumentSourceAssurance"]["variants"],
        json!([["declared_only",{"kind":"unit"}]])
    );
    assert!(!m["types"]["FactKey"].to_string().contains("Observation"));
    registered_descriptor_integrity().unwrap();
}
#[test]
fn complete_old_registry_records_and_registered_rules_are_unchanged() {
    for (file, key) in [("specifications", "specifications"), ("rules", "rules")] {
        let old = read(&format!("edition5-{file}.json"));
        let now: Value = serde_json::from_slice(
            &std::fs::read(root().join(format!("spec/registry/{file}.json"))).unwrap(),
        )
        .unwrap();
        for row in old[key].as_array().unwrap() {
            let id = if file == "rules" {
                "rule_id"
            } else {
                "specification_id"
            };
            assert!(now[key].as_array().unwrap().contains(row), "{}", row[id]);
        }
    }
    let rules: Value =
        serde_json::from_slice(&std::fs::read(root().join("spec/registry/rules.json")).unwrap())
            .unwrap();
    let mut rule_actual = Vec::new();
    for r in read("rule-registration.json")["cases"].as_array().unwrap() {
        let row = rules["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["rule_id"] == r["rule_id"])
            .unwrap();
        assert_eq!(row["short_statement"], r["statement"]);
        let source =
            std::fs::read_to_string(root().join(row["normative_source"].as_str().unwrap()))
                .unwrap();
        assert!(source.contains(r["statement"].as_str().unwrap()));
        assert!(source.contains(&format!("id=\"{}\"", r["anchor"].as_str().unwrap())));
        assert!(
            row["conformance_refs"]
                .as_array()
                .unwrap()
                .contains(&r["case_id"])
        );
        rule_actual.push(json!({"case_id":r["case_id"],"rule_id":r["rule_id"],"registration_checked":true,"predicate_evaluated":false}));
    }
    std::fs::write(
        root().join("target/phase-04c1b4c4r3c2g2-rule-actual.json"),
        serde_json::to_vec_pretty(&rule_actual).unwrap(),
    )
    .unwrap();
}

#[test]
fn real_native_direct_private_and_alternate_root_boundaries_still_deny() {
    let mut count = 0;
    for group in ["positive", "negative", "context", "subflow", "recovery"] {
        for entry in std::fs::read_dir(
            root()
                .join("spec/conformance/computation-root-export")
                .join(group),
        )
        .unwrap()
        {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|x| x != "json") {
                continue;
            }
            let input: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
            let actual = support::control_fixture::analyze_root_export_case(&input).unwrap();
            if actual["stage"] != "component_validation_rejected" {
                assert_eq!(
                    actual["checked_static_compatibility_capability_issued"],
                    false
                );
                assert_ne!(actual["static_compatibility"], "proven");
            }
            count += 1;
        }
    }
    assert_eq!(count, 35);
}

// A deliberately bounded read-only conformance checker for three registered
// scalar/ordinal types. It does not encode/normalize a SourcePackage or Scope.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum DagPayloadOrder {
    Unsigned(u64),
    Bytes(Vec<u8>),
    Child(u64),
}
fn check_scalar_ordinal_dag(v: &Value) -> Result<(), RegisteredModelFailure> {
    use RegisteredModelFailure::{InvalidFacts, ResourceLimit, UnsupportedModel};
    validate_registered_model_shape("CanonicalValueDag", v)?;
    let nodes = v["nodes"].as_array().ok_or(UnsupportedModel)?;
    if nodes.len() > 4096 {
        return Err(ResourceLimit);
    }
    let mut edges = Vec::<Vec<usize>>::new();
    let mut heights = Vec::new();
    let mut prior = None;
    for (rank, node) in nodes.iter().enumerate() {
        if nodes[..rank].contains(node) {
            return Err(InvalidFacts);
        }
        if node["type"]["owner"] != SUCCESSOR_MODEL_ID {
            return Err(UnsupportedModel);
        }
        let name = node["type"]["name"].as_str().ok_or(UnsupportedModel)?;
        let (children, payload) = match name {
            "$U64" => (
                vec![],
                DagPayloadOrder::Unsigned(
                    node["body"]["scalar"]["uint64"]
                        .as_u64()
                        .ok_or(UnsupportedModel)?,
                ),
            ),
            "DocumentSourceKey" => {
                let value = &node["body"]["scalar"]["bytes"];
                validate_registered_model_shape("DocumentSourceKey", value)?;
                let b = value
                    .as_array()
                    .ok_or(UnsupportedModel)?
                    .iter()
                    .map(|x| {
                        u8::try_from(x.as_u64().ok_or(UnsupportedModel)?)
                            .map_err(|_| UnsupportedModel)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                (vec![], DagPayloadOrder::Bytes(b))
            }
            "DocumentOccurrenceKey" => {
                let fields = node["body"]["record"].as_array().ok_or(UnsupportedModel)?;
                if fields.len() != 1 || fields[0]["field"] != "entry_ordinal" {
                    return Err(UnsupportedModel);
                }
                let index = fields[0]["value"].as_u64().ok_or(UnsupportedModel)?;
                let child = usize::try_from(index).map_err(|_| InvalidFacts)?;
                if child >= rank || nodes[child]["type"]["name"] != "$U64" {
                    return Err(InvalidFacts);
                }
                (vec![child], DagPayloadOrder::Child(index))
            }
            _ => return Err(UnsupportedModel),
        };
        let height = children
            .iter()
            .map(|i| heights[*i] + 1)
            .max()
            .unwrap_or(0usize);
        let key = (height, name.to_owned(), payload);
        if prior.as_ref().is_some_and(|p| p >= &key) {
            return Err(InvalidFacts);
        }
        prior = Some(key);
        heights.push(height);
        edges.push(children);
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut pending = v["roots"]
        .as_array()
        .ok_or(UnsupportedModel)?
        .iter()
        .map(|x| usize::try_from(x.as_u64().ok_or(UnsupportedModel)?).map_err(|_| InvalidFacts))
        .collect::<Result<Vec<_>, _>>()?;
    while let Some(i) = pending.pop() {
        if i >= nodes.len() {
            return Err(InvalidFacts);
        }
        if seen.insert(i) {
            pending.extend(&edges[i]);
        }
    }
    if seen.len() != nodes.len() {
        return Err(InvalidFacts);
    }
    Ok(())
}
#[test]
fn registered_scalar_ordinal_dag_negative_laws_are_not_scope_normalization() {
    let cases = read("dag-cases.json");
    let mut actual = Vec::new();
    for c in cases["cases"].as_array().unwrap() {
        let failure = check_scalar_ordinal_dag(&c["value"])
            .err()
            .map(|e| format!("{e:?}"));
        assert_eq!(
            failure.as_deref(),
            c["expected_failure"].as_str(),
            "{}",
            c["case_id"]
        );
        actual.push(json!({"case_id":c["case_id"],"actual_failure":failure,"full_scope_checked":false,"normalized_identity_produced":false}));
    }
    std::fs::write(
        root().join("target/phase-04c1b4c4r3c2g2-dag-actual.json"),
        serde_json::to_vec_pretty(&actual).unwrap(),
    )
    .unwrap();
}

#[allow(dead_code, unused_imports)]
mod fixtures {
    include!("../../af-contract/tests/component_contract_conformance.rs");
    use af_computation::*;
    use serde_json::{Value, json};
    fn document(package: &Value, owner: usize, slot: usize) -> &Value {
        let c = package["owners"][owner]["container"].as_object().unwrap();
        let (kind, body) = c.iter().next().unwrap();
        let field = match kind.as_str() {
            "declaration_bundle" => "declarations",
            "import_manifest" => "imports",
            _ => "sources",
        };
        &body[field][slot]["document"]
    }
    #[test]
    fn real_borrowed_owner_slot_and_context_admission_cases_are_not_capture() {
        let cases = super::read("admission-cases.json");
        let mut f = FixtureContext::new();
        let _ = guard_input_type(&mut f, "CaptureInput", Primitive::Int64);
        let c = f.view();
        let other_c = f.view();
        let compiler = current_compiler_semantics_identity();
        let mut actual = Vec::new();
        for case in cases["cases"].as_array().unwrap() {
            let package = &case["package"];
            let mut joint = package.clone();
            let operation = case["operation"].as_str().unwrap();
            let outcome = if operation.starts_with("joint") {
                if operation == "joint_conflict" {
                    joint["owners"][0]["container"]["declaration_bundle"]["bundle_name"] =
                        json!([121]);
                }
                check_joint_source_packages_metadata(&[package, &joint])
            } else {
                let borrowed = document(
                    package,
                    usize::try_from(case["borrow_owner"].as_u64().unwrap()).unwrap(),
                    usize::try_from(case["borrow_slot"].as_u64().unwrap()).unwrap(),
                );
                let cloned = borrowed.clone();
                let doc = if !case["external_document"].is_null() {
                    &case["external_document"]
                } else if case["clone_document"] == true {
                    &cloned
                } else {
                    borrowed
                };
                let owner = if case["source_owner"].is_null() {
                    &package["owners"][0]["anchor"]
                } else {
                    &case["source_owner"]
                };
                let path = [RawDocumentMember::Entry(
                    case["path_entry"].as_u64().unwrap(),
                )];
                let decoded: Value = if let Some(p) = doc.get("physical_json_utf8") {
                    let bytes = p["bytes"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|x| u8::try_from(x.as_u64().unwrap()).unwrap())
                        .collect::<Vec<_>>();
                    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
                } else {
                    Value::Null
                };
                let candidate = if !case["candidate_override"].is_null() {
                    &case["candidate_override"]
                } else if doc.get("declared_native_document").is_some() {
                    &doc["declared_native_document"]["declarations"][0]
                } else {
                    &decoded[0]
                };
                let physical = match case["physical"].as_str().unwrap() {
                    "matching" => OccurrencePhysicalMetadata::Supplied {
                        source_owner: owner,
                        entry_ordinal: case["slot"].as_u64().unwrap_or(0),
                        actual_document: if case["physical_document_override"].is_null() {
                            doc
                        } else {
                            &case["physical_document_override"]
                        },
                        value_path: &path,
                    },
                    "other" => OccurrencePhysicalMetadata::Supplied {
                        source_owner: &package["owners"][1]["anchor"],
                        entry_ordinal: 0,
                        actual_document: document(package, 1, 0),
                        value_path: &path,
                    },
                    _ => OccurrencePhysicalMetadata::NotSupplied,
                };
                let req = DocumentOccurrenceMetadataRequest {
                    source_package: if case["missing_package"] == true {
                        None
                    } else {
                        Some(package)
                    },
                    source_owner: if case["missing_owner"] == true {
                        None
                    } else {
                        Some(owner)
                    },
                    entry_ordinal: case["slot"].as_u64(),
                    actual_document: doc,
                    candidate,
                    internal_path: &path,
                    physical,
                };
                let mut wrong = compiler.clone();
                if case["compiler_wrong"] == true {
                    wrong.analysis_semantics_version = "agentflow.root-export-checker@5".into();
                }
                let result = admit_document_occurrence_metadata(
                    &req,
                    &c,
                    if case["context_split"] == true {
                        &other_c
                    } else {
                        &c
                    },
                    &wrong,
                );
                if result.is_ok() && operation == "repeat" {
                    admit_document_occurrence_metadata(&req, &c, &c, &compiler).unwrap();
                }
                if result.is_ok() && matches!(operation, "different_slots" | "different_owners") {
                    let (next_owner, next_slot) = if operation == "different_slots" {
                        (0, 1)
                    } else {
                        (1, 0)
                    };
                    let second_doc = document(package, next_owner, next_slot);
                    let second_owner = &package["owners"][next_owner]["anchor"];
                    let second = DocumentOccurrenceMetadataRequest {
                        source_package: Some(package),
                        source_owner: Some(second_owner),
                        entry_ordinal: Some(next_slot as u64),
                        actual_document: second_doc,
                        candidate: &second_doc["declared_native_document"]["declarations"][0],
                        internal_path: &path,
                        physical: OccurrencePhysicalMetadata::NotSupplied,
                    };
                    admit_document_occurrence_metadata(&second, &c, &c, &compiler).unwrap();
                    assert_eq!(doc, second_doc);
                    assert_ne!(
                        json!([owner, case["slot"]]),
                        json!([second_owner, next_slot])
                    ); // Admitted input projection, NOT emitted FactKey.
                }
                result
            };
            let status = outcome
                .err()
                .map_or_else(|| "Admitted".to_owned(), |e| format!("{e:?}"));
            assert_eq!(
                status,
                case["expected"].as_str().unwrap(),
                "{}",
                case["case_id"]
            );
            actual.push(json!({"case_id":case["case_id"],"actual":status,"predicate_evaluated":false,"raw_facts_produced":0,"root_capability":false}));
        }
        std::fs::write(
            super::root().join("target/phase-04c1b4c4r3c2g2-admission-actual.json"),
            serde_json::to_vec_pretty(&actual).unwrap(),
        )
        .unwrap();
    }
    #[test]
    fn legacy_anonymous_source_and_all_root_boundaries_fail_closed() {
        let output = std::process::Command::new("python")
            .env("PYTHONUTF8", "1")
            .arg(super::root().join("tools/oracle/native_raw_binding_c1.py"))
            .arg("--vector")
            .output()
            .unwrap();
        assert!(output.status.success());
        let v: Value = serde_json::from_slice(&output.stdout).unwrap();
        let raw: ComponentContract = serde_json::from_value(v["rust_input"].clone()).unwrap();
        let mut f = FixtureContext::new();
        let _ = guard_input_type(&mut f, "CaptureInput", Primitive::Int64);
        let c = f.view();
        let declarations = vec![raw.clone()];
        let doc = NativeRawDocument {
            declarations: &declarations,
        };
        assert_eq!(
            admit_native_raw_source(
                &raw,
                Some(&doc),
                0,
                &c,
                &c,
                &current_compiler_semantics_identity()
            ),
            Err(RawAdmissionFailure::MissingDocumentOccurrenceBinding)
        );
        assert!(af_contract::canonicalize_contract(&raw, &c).is_err());
        assert!(check_contract(&raw, &c).is_err());
        assert!(!current_registered_analysis_edition().can_issue_capability());
    }
}
