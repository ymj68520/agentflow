#![recursion_limit = "512"]
#[allow(dead_code)]
mod support;
use af_computation::*;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn read(name: &str) -> Value {
    serde_json::from_slice(
        &std::fs::read(
            root()
                .join("spec/conformance/g01-raw-validation-staging")
                .join(name),
        )
        .unwrap(),
    )
    .unwrap()
}
fn identity(v: &Value) -> CompilerSemanticsIdentity {
    CompilerSemanticsIdentity {
        analysis_semantics_version: v["analysis_semantics_version"].as_str().unwrap().into(),
        registry_content: hex::decode(v["registry_hex"].as_str().unwrap()).unwrap(),
        rule_registry_content: hex::decode(v["rule_registry_hex"].as_str().unwrap()).unwrap(),
        specifications: v["specifications"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| SpecificationContentIdentity {
                specification_id: s["specification_id"].as_str().unwrap().into(),
                logical_path: s["logical_path"].as_str().unwrap().into(),
                content_sha256: s["content_sha256"].as_str().unwrap().into(),
                content: hex::decode(s["content_hex"].as_str().unwrap()).unwrap(),
            })
            .collect(),
        rules: v["rules"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| RuleContentIdentity {
                rule_id: s["rule_id"].as_str().unwrap().into(),
                normative_source: s["normative_source"].as_str().unwrap().into(),
                source_anchor: s["source_anchor"].as_str().unwrap().into(),
                short_statement: s["short_statement"].as_str().unwrap().into(),
            })
            .collect(),
    }
}
#[test]
fn every_registered_shape_case_executes_without_trusted_outputs() {
    let corpus = read("shape-cases.json");
    for c in corpus["cases"].as_array().unwrap() {
        let result = validate_registered_model_shape(c["type"].as_str().unwrap(), &c["value"])
            .err()
            .map(|e| format!("{e:?}"));
        assert_eq!(
            result.as_deref(),
            c["expected_failure"].as_str(),
            "{}",
            c["case_id"]
        );
    }
    assert!(!current_registered_analysis_edition().can_issue_capability());
}
#[test]
fn exact_successor_identity_registration_and_all_denials() {
    let e = current_registered_analysis_edition();
    assert_eq!(e.compiler, identity(&read("compiler-current.json")));
    assert_eq!(
        (
            e.compiler.specifications.len(),
            e.compiler.rules.len(),
            e.registered_sources.len()
        ),
        (32, 174, 58)
    );
    assert_eq!(e.registration, EditionRegistration::Registered);
    assert_eq!(e.support, EditionSupport::NotImplemented);
    assert_eq!(e.activation, EditionActivation::NotAuthorized);
    assert!(!e.can_issue_capability());
    assert_eq!(
        admit_registered_profile("raw_contract_validation", false),
        Err(RegisteredModelFailure::NotImplemented)
    );
    assert_eq!(
        admit_registered_profile("raw_contract_validation", true),
        Err(RegisteredModelFailure::UnsupportedModel)
    );
    assert_eq!(
        admit_registered_profile("unknown", false),
        Err(RegisteredModelFailure::UnsupportedModel)
    );
    check_registered_compiler_identity(&e.compiler).unwrap();
    for version in 1..=4 {
        let mut wrong = e.compiler.clone();
        wrong.analysis_semantics_version = format!("agentflow.root-export-checker@{version}");
        assert_eq!(
            check_registered_compiler_identity(&wrong),
            Err(EditionAdmissionFailure::InvalidIdentity)
        );
    }
    let mut missing = e.compiler.clone();
    missing
        .specifications
        .retain(|s| s.specification_id != SUCCESSOR_MODEL_ID);
    assert_eq!(
        check_registered_compiler_identity(&missing),
        Err(EditionAdmissionFailure::IncompleteContent)
    );
    let mut unknown = e.compiler.clone();
    unknown.analysis_semantics_version = "unknown@999".into();
    assert_eq!(
        check_registered_compiler_identity(&unknown),
        Err(EditionAdmissionFailure::UnsupportedSemantics)
    );
    let old: Value = serde_json::from_slice(
        &std::fs::read(
            root().join("spec/conformance/g01-native-guard-staging/compiler-current.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        check_registered_compiler_identity(&identity(&old)),
        Err(EditionAdmissionFailure::UnsupportedSemantics)
    );
    let mut mixed = identity(&old);
    mixed.analysis_semantics_version = ROOT_EXPORT_ANALYSIS_SEMANTICS_VERSION.into();
    assert_eq!(
        check_registered_compiler_identity(&mixed),
        Err(EditionAdmissionFailure::InvalidIdentity)
    );
}
#[test]
fn all_old_registry_records_and_model_are_full_content_equal() {
    for (file, key, id, count) in [
        ("specifications", "specifications", "specification_id", 54),
        ("rules", "rules", "rule_id", 468),
    ] {
        let old = read(&format!("edition4-{file}.json"));
        let now: Value = serde_json::from_slice(
            &std::fs::read(root().join(format!("spec/registry/{file}.json"))).unwrap(),
        )
        .unwrap();
        assert_eq!(old[key].as_array().unwrap().len(), count);
        for row in old[key].as_array().unwrap() {
            assert_eq!(
                now[key]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| r[id] == row[id]),
                Some(row)
            );
        }
    }
    let m = registered_model_catalogue().unwrap();
    let oldfile =
        std::fs::read_to_string(root().join("spec/tables/proof-report-model-v3.md")).unwrap();
    let old: Value = serde_json::from_str(
        oldfile
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
    for (i, name) in old["types"].as_object().unwrap().keys().enumerate() {
        // serde_json maps use lexical order; ranks come from the registered
        // source's explicit successor type_ranks, not this map iteration.
        let _ = i;
        assert!(m["type_ranks"].get(name).is_some());
        if ![
            "ScopeSubject",
            "PredicateFamily",
            "ClosedPredicateCase",
            "RequirementClause",
            "Source",
            "Target",
            "Coordinate",
            "TypedContradiction",
            "EvidenceRef",
            "ObservationOwner",
            "Fact",
            "Origin",
            "Fragment",
        ]
        .contains(&name.as_str())
        {
            assert_eq!(m["types"][name], old["types"][name]);
        }
    }
    assert_eq!(
        m["raw_case_descriptors"][0]["case"],
        "positive_unique_io_ports"
    );
    assert_eq!(
        m["raw_case_descriptors"][0]["governing_rule"],
        "AF-CONTRACT-0004"
    );
    assert_eq!(m["native_case_descriptors"].as_array().unwrap().len(), 7);
    assert_eq!(m["case_descriptors"].as_array().unwrap().len(), 27);
}
#[test]
fn all_rule_anchors_and_new_descriptor_fields_have_executed_case_links() {
    let rules: Value =
        serde_json::from_slice(&std::fs::read(root().join("spec/registry/rules.json")).unwrap())
            .unwrap();
    let rows = read("rule-registration.json");
    for r in rows["cases"].as_array().unwrap() {
        let actual = rules["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["rule_id"] == r["rule_id"])
            .unwrap();
        assert_eq!(actual["source_anchor"], r["anchor"]);
        assert_eq!(actual["short_statement"], r["statement"]);
        assert!(
            actual["conformance_refs"]
                .as_array()
                .unwrap()
                .contains(&r["case_id"])
        );
        let text =
            std::fs::read_to_string(root().join(actual["normative_source"].as_str().unwrap()))
                .unwrap();
        assert_eq!(
            text.matches(&format!("id=\"{}\"", r["anchor"].as_str().unwrap()))
                .count(),
            1
        );
    }
    let corpus = read("shape-cases.json");
    let ids = corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["case_id"].as_str().unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), corpus["cases"].as_array().unwrap().len());
    assert!(ids.len() >= 64);
    let m = registered_model_catalogue().unwrap();
    for name in m["raw_registration"]["authored_types"].as_array().unwrap() {
        let n = name.as_str().unwrap();
        let d = &m["types"][n];
        for field in d["fields"].as_array().into_iter().flatten() {
            assert!(
                corpus["coverage"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|c| c["type"] == n
                        && c["field"] == field[0]
                        && ids.contains(c["case_id"].as_str().unwrap()))
            );
        }
        for variant in d["variants"].as_array().into_iter().flatten() {
            assert!(
                corpus["coverage"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|c| c["type"] == n
                        && c["variant"] == variant[0]
                        && ids.contains(c["case_id"].as_str().unwrap()))
            );
        }
    }
    assert_eq!(read("manifest.json")["capture_case_ids"], 0);
}
#[test]
fn real_root_analyzer_direct_and_private_boundaries_still_deny() {
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
            let v: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
            let a = support::control_fixture::analyze_root_export_case(&v).unwrap();
            if a["stage"] != "component_validation_rejected" {
                assert_eq!(a["checked_static_compatibility_capability_issued"], false);
                assert_ne!(a["static_compatibility"], "proven");
            }
            count += 1;
        }
    }
    assert_eq!(count, 35);
}

#[allow(dead_code, unused_imports)]
mod native_behavior {
    include!("../../af-contract/tests/component_contract_conformance.rs");
    #[test]
    fn source_admission_and_real_port_behavior_never_create_new_capture() {
        use af_computation::*;
        use serde_json::{Value, json};
        use std::process::Command;
        let output = Command::new("python")
            .env("PYTHONUTF8", "1")
            .arg(repo_root().join("tools/oracle/native_raw_binding_c1.py"))
            .arg("--vector")
            .output()
            .unwrap();
        let v: Value = serde_json::from_slice(&output.stdout).unwrap();
        let base: ComponentContract = serde_json::from_value(v["rust_input"].clone()).unwrap();
        let mut f = FixtureContext::new();
        let _ = guard_input_type(&mut f, "CaptureInput", Primitive::Int64);
        let c = f.view();
        let other = f.view();
        let compiler = current_compiler_semantics_identity();
        let declarations = vec![base.clone(), base.clone()];
        let document = NativeRawDocument {
            declarations: &declarations,
        };
        assert_eq!(
            admit_native_raw_source(&base, None, 0, &c, &c, &compiler),
            Err(RawAdmissionFailure::MissingDeclarativeOrigin)
        );
        assert_eq!(
            admit_native_raw_source(&base, Some(&document), 0, &c, &other, &compiler),
            Err(RawAdmissionFailure::InvalidContextBinding)
        );
        admit_native_raw_source(&base, Some(&document), 0, &c, &c, &compiler).unwrap();
        admit_native_raw_source(&base, Some(&document), 1, &c, &c, &compiler).unwrap();
        assert_eq!(
            admit_native_raw_source(&base, Some(&document), 2, &c, &c, &compiler),
            Err(RawAdmissionFailure::InvalidSourceAssociation)
        );
        let mut cases = Vec::new();
        for (name, inputs, outputs, identity_bad, pin_bad) in [
            ("zero", vec![0], vec![], false, false),
            ("duplicate-input", vec![2, 1, 2], vec![], false, false),
            ("duplicate-output", vec![], vec![1, 1], false, false),
            ("separate-namespaces", vec![1], vec![1], false, false),
            ("positive", vec![2, 1], vec![], false, false),
            ("early-identity", vec![0], vec![], true, false),
            ("early-schema", vec![1, 2], vec![], false, true),
            ("empty", vec![], vec![], false, false),
        ] {
            let mut raw = base.clone();
            raw.inputs = inputs
                .iter()
                .map(|n| {
                    let mut p = base.inputs[0].clone();
                    p.port_no = *n;
                    p
                })
                .collect();
            raw.outputs = outputs
                .iter()
                .map(|n| af_contract::ComponentOutputPort {
                    port_no: *n,
                    type_ref: base.inputs[0].type_ref.clone(),
                })
                .collect();
            if identity_bad {
                raw.component_identity.name = "BAD".into();
            }
            if pin_bad {
                raw.inputs[0].type_ref.schema_digest.hex = "0".repeat(64);
            }
            let canonical = af_contract::canonicalize_contract(&raw, &c);
            let report = af_contract::analyze_contract(&raw, &c);
            let error = canonical.as_ref().err();
            match name {
                "zero" | "duplicate-input" | "duplicate-output" => {
                    assert_eq!(error.unwrap().code, "invalid_or_duplicate_port");
                }
                "early-identity" => assert_eq!(error.unwrap().code, "invalid_component_identity"),
                "early-schema" => assert_eq!(error.unwrap().code, "unknown_or_wrong_type_pin"),
                _ => assert!(canonical.is_ok()),
            }
            if error.is_some() {
                assert!(report.action_guard_compatibility.is_empty());
                assert!(check_contract(&raw, &c).is_err());
            }
            if let Ok(value) = &canonical {
                let numbers = value
                    .contract()
                    .inputs
                    .iter()
                    .map(|p| p.port_no)
                    .collect::<Vec<_>>();
                let mut sorted = inputs.clone();
                sorted.sort_unstable();
                assert_eq!(numbers, sorted);
            }
            cases.push(json!({"case_id":format!("g01-raw-validation-staging/behavior-{name}"),
                "original_input_numbers":inputs,"original_output_numbers":outputs,
                "canonical_input_numbers":canonical.as_ref().ok().map(|v|v.contract().inputs.iter().map(|p|p.port_no).collect::<Vec<_>>()),
                "canonical_admitted":canonical.is_ok(),"error":error.map(|e|json!({"code":e.code,"rule_id":e.rule_id,"reference":e.reference,"detail":e.detail})),
                "schema_carrier_verdict":report.dimensions[0].verdict.as_str(),
                "native_guard_record_count":report.action_guard_compatibility.len(),
                "separate_legacy_whole_check_produced_checked":check_contract(&raw,&c).is_ok(),
                "new_typed_capture":"NotImplemented"}));
        }
        std::fs::write(
            repo_root().join("target/phase-04c1b4c4r3c1g2-port-behavior.json"),
            serde_json::to_vec_pretty(&cases).unwrap(),
        )
        .unwrap();
    }
    #[test]
    fn physical_sources_are_strict_original_documents_not_normalized_displays() {
        use af_computation::*;
        use serde_json::{Value, json};
        let f = FixtureContext::new();
        let c = f.view();
        let d = f.view();
        let compiler = current_compiler_semantics_identity();
        let cases: Value = serde_json::from_slice(
            &std::fs::read(
                repo_root().join("spec/conformance/g01-raw-validation-staging/shape-cases.json"),
            )
            .unwrap(),
        )
        .unwrap();
        let candidate = cases["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["type"] == "RawParsedInput" && x["expected_failure"].is_null())
            .unwrap()["value"]["candidate"]
            .clone();
        let nfd_document =
            serde_json::to_vec(&json!({"declaration":candidate,"label":"e\u{301}"})).unwrap();
        let display_form =
            serde_json::to_vec_pretty(&json!({"declaration":candidate,"label":"é"})).unwrap();
        assert_ne!(nfd_document, display_form);
        let path = [RawDocumentMember::Field(b"declaration".to_vec())];
        admit_physical_raw_source(&candidate, Some(&nfd_document), &path, &c, &c, &compiler)
            .unwrap();
        admit_physical_raw_source(&candidate, Some(&display_form), &path, &c, &c, &compiler)
            .unwrap();
        assert_eq!(
            admit_physical_raw_source(&candidate, None, &path, &c, &c, &compiler),
            Err(RawAdmissionFailure::MissingDeclarativeOrigin)
        );
        assert_eq!(
            admit_physical_raw_source(&candidate, Some(&nfd_document), &path, &c, &d, &compiler),
            Err(RawAdmissionFailure::InvalidContextBinding)
        );
        assert_eq!(
            admit_physical_raw_source(&candidate, Some(&[255]), &[], &c, &c, &compiler),
            Err(RawAdmissionFailure::InvalidPhysicalInput)
        );
        assert_eq!(
            admit_physical_raw_source(
                &candidate,
                Some(br#"{"x":1,"x":2}"#),
                &[],
                &c,
                &c,
                &compiler
            ),
            Err(RawAdmissionFailure::InvalidPhysicalInput)
        );
        assert_eq!(
            admit_physical_raw_source(&candidate, Some(b"{"), &[], &c, &c, &compiler),
            Err(RawAdmissionFailure::InvalidPhysicalInput)
        );
        assert_eq!(
            admit_physical_raw_source(
                &candidate,
                Some(&nfd_document),
                &[RawDocumentMember::Field(b"absent".to_vec())],
                &c,
                &c,
                &compiler
            ),
            Err(RawAdmissionFailure::InvalidSourceAssociation)
        );
        let mut bad = candidate.clone();
        bad.as_object_mut()
            .unwrap()
            .insert("unknown_field".into(), Value::Null);
        let badbytes = serde_json::to_vec(&bad).unwrap();
        assert_eq!(
            admit_physical_raw_source(&bad, Some(&badbytes), &[], &c, &c, &compiler),
            Err(RawAdmissionFailure::UnsupportedModel)
        );
        let mut missing = candidate.clone();
        missing.as_object_mut().unwrap().remove("inputs");
        let missingbytes = serde_json::to_vec(&missing).unwrap();
        assert_eq!(
            admit_physical_raw_source(&missing, Some(&missingbytes), &[], &c, &c, &compiler),
            Err(RawAdmissionFailure::UnsupportedModel)
        );
        let mut malformed = candidate.clone();
        malformed["inputs"] = json!("not-array");
        let malformedbytes = serde_json::to_vec(&malformed).unwrap();
        assert_eq!(
            admit_physical_raw_source(&malformed, Some(&malformedbytes), &[], &c, &c, &compiler),
            Err(RawAdmissionFailure::UnsupportedModel)
        );
        // Successful association returns unit only; no typed validator output.
        assert!(!current_registered_analysis_edition().can_issue_capability());
    }
}
