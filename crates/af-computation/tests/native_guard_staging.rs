#![recursion_limit = "512"]
#[allow(dead_code)]
mod support;
use af_computation::*;
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn read(name: &str) -> Value {
    serde_json::from_slice(
        &std::fs::read(
            root()
                .join("spec/conformance/g01-native-guard-staging")
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
fn exact_current_full_identity_and_four_independent_denials() {
    let e = current_registered_analysis_edition();
    assert_eq!(e.compiler, identity(&read("compiler-current.json")));
    assert_eq!(
        (
            e.compiler.specifications.len(),
            e.compiler.rules.len(),
            e.registered_sources.len()
        ),
        (28, 163, 54)
    );
    assert_eq!(e.registration, EditionRegistration::Registered);
    assert_eq!(e.support, EditionSupport::NotImplemented);
    assert_eq!(e.activation, EditionActivation::NotAuthorized);
    assert_eq!(e.release, EditionRelease::NotAuthorized);
    assert_eq!(e.report_shape, "agentflow.g01.report-shape@3");
    assert_eq!(e.obligation_edition, "agentflow.g01.obligation@3");
    assert!(!e.can_issue_capability());
    for (profile, as_root, expected) in [
        (
            "native_recovery_guard",
            false,
            RegisteredModelFailure::NotImplemented,
        ),
        (
            "native_recovery_guard",
            true,
            RegisteredModelFailure::UnsupportedModel,
        ),
        (
            "standalone_substitution",
            true,
            RegisteredModelFailure::UnsupportedModel,
        ),
        ("root_export", true, RegisteredModelFailure::NotImplemented),
        ("unknown", false, RegisteredModelFailure::UnsupportedModel),
    ] {
        assert_eq!(admit_registered_profile(profile, as_root), Err(expected));
    }
}
#[test]
fn every_registration_shape_case_executes_no_fact_or_capability() {
    let cases = read("shape-cases.json");
    for c in cases["cases"].as_array().unwrap() {
        let actual = validate_registered_model_shape(c["type"].as_str().unwrap(), &c["value"])
            .err()
            .map(|e| format!("{e:?}"));
        assert_eq!(
            actual.as_deref(),
            c["expected_failure"].as_str(),
            "{}",
            c["case_id"]
        );
    }
    assert!(!current_registered_analysis_edition().can_issue_capability());
}
#[test]
fn independent_ordered_roster_native_field_and_variant_laws() {
    registered_descriptor_integrity().unwrap();
    let m = registered_model_catalogue().unwrap();
    for c in read("descriptor-laws.json")["cases"].as_array().unwrap() {
        if c["part"] == "native_case" {
            let actual = m["native_case_descriptors"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["case"] == c["case"])
                .unwrap();
            for (k, v) in c["expected"].as_object().unwrap() {
                assert_eq!(&actual[k], v, "{} {k}", c["case_id"]);
            }
        } else {
            let key = c["part"].as_str().unwrap();
            let actual = m["types"][c["type"].as_str().unwrap()][key]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| p[0].clone())
                .collect::<Vec<_>>();
            assert_eq!(json!(actual), c["expected"], "{}", c["case_id"]);
        }
    }
    assert_eq!(m["types"].as_object().unwrap().len(), 652);
}
#[test]
fn historical_imports_full_fields_recursive_edges_and_original_source_contents() {
    let m = registered_model_catalogue().unwrap();
    let manifest = read("historical-imports.json");
    assert_eq!(
        m["historical_selected_imports"],
        manifest["selected_imports"]
    );
    let old = &m["inherited_registered_model"]["inherited_registered_model"];
    let rows = manifest["selected_imports"].as_array().unwrap();
    assert_eq!(rows.len(), 97);
    assert_eq!(rows.iter().filter(|r| r["root"] == true).count(), 5);
    for row in rows {
        assert_eq!(row["owner"], "agentflow.computation.proof-report.model@1");
        assert_eq!(
            m["types"][row["successor_key"].as_str().unwrap()],
            row["successor_descriptor"]
        );
        if !row["historical_declaration"].is_null() {
            assert_eq!(
                old["semantic_model_declarations"][row["name"].as_str().unwrap()],
                row["historical_declaration"]
            );
        }
    }
    for (path, s) in manifest["sources"].as_object().unwrap() {
        // Exact historical owner resolves its immutable registered snapshot,
        // never whichever implementation currently occupies the source path.
        assert_eq!(&m["historical_import_sources"][path], s);
        let bytes = hex::decode(s["content_hex"].as_str().unwrap()).unwrap();
        assert_eq!(hex::encode(&bytes), s["content_hex"]);
        assert_eq!(hex::encode(Sha256::digest(&bytes)), s["sha256"]);
    }
    let status = std::process::Command::new("python")
        .arg(root().join("tools/native_guard_source_binding.py"))
        .current_dir(root())
        .status()
        .expect("live imported declaration audit");
    assert!(
        status.success(),
        "live declarations must retain exact fields/types/variant ranks"
    );
}
#[test]
fn old_complete_registry_rows_and_sources_stay_frozen() {
    for (name, key, id, count) in [
        ("specifications", "specifications", "specification_id", 50),
        ("rules", "rules", "rule_id", 452),
    ] {
        let old = read(&format!("edition3-{name}.json"));
        let now: Value = serde_json::from_slice(
            &std::fs::read(root().join(format!("spec/registry/{name}.json"))).unwrap(),
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
            if name == "specifications" {
                let b = std::fs::read(
                    root().join(row["normative_source"]["logical_path"].as_str().unwrap()),
                )
                .unwrap();
                assert_eq!(
                    hex::encode(Sha256::digest(b)),
                    row["normative_source"]["content_digest"]["hex"]
                );
            }
        }
    }
    let rules: Value =
        serde_json::from_slice(&std::fs::read(root().join("spec/registry/rules.json")).unwrap())
            .unwrap();
    for c in read("rule-registration.json")["cases"].as_array().unwrap() {
        let row = rules["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["rule_id"] == c["rule_id"])
            .unwrap();
        assert_eq!(row["normative_source"], c["source"]);
        assert_eq!(row["source_anchor"], c["anchor"]);
        assert_eq!(row["short_statement"], c["statement"]);
        assert!(
            row["conformance_refs"]
                .as_array()
                .unwrap()
                .contains(&c["case_id"])
        );
        let text = std::fs::read_to_string(root().join(c["source"].as_str().unwrap())).unwrap();
        assert!(text.contains(c["statement"].as_str().unwrap()));
        assert_eq!(
            text.matches(&format!("id=\"{}\"", c["anchor"].as_str().unwrap()))
                .count(),
            1
        );
    }
    let m = read("manifest.json");
    let ids = m["case_ids"].as_array().unwrap();
    assert_eq!(
        ids.len(),
        usize::try_from(m["case_count"].as_u64().unwrap()).unwrap()
    );
    assert_eq!(
        ids.iter()
            .map(|v| v.as_str().unwrap())
            .collect::<BTreeSet<_>>()
            .len(),
        ids.len()
    );
    assert_eq!(m["capture_case_ids"], 0);
}
#[test]
fn exact_old_content_current_mixtures_and_spoofed_support_reject() {
    let current = identity(&read("compiler-current.json"));
    check_registered_compiler_identity(&current).unwrap();
    let old: Value = serde_json::from_slice(
        &std::fs::read(
            root().join("spec/conformance/g01-successor-staging/compiler-successor.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        check_registered_compiler_identity(&identity(&old)),
        Err(EditionAdmissionFailure::UnsupportedSemantics)
    );
    for version in [1, 2, 3] {
        let mut x = current.clone();
        x.analysis_semantics_version = format!("agentflow.root-export-checker@{version}");
        assert_eq!(
            check_registered_compiler_identity(&x),
            Err(EditionAdmissionFailure::InvalidIdentity)
        );
    }
    let mut mixed = identity(&old);
    mixed.analysis_semantics_version = ROOT_EXPORT_ANALYSIS_SEMANTICS_VERSION.into();
    assert_eq!(
        check_registered_compiler_identity(&mixed),
        Err(EditionAdmissionFailure::InvalidIdentity)
    );
    let mut missing = current.clone();
    missing
        .specifications
        .retain(|s| s.specification_id != SUCCESSOR_MODEL_ID);
    assert_eq!(
        check_registered_compiler_identity(&missing),
        Err(EditionAdmissionFailure::IncompleteContent)
    );
    let mut changed = current.clone();
    changed
        .specifications
        .last_mut()
        .unwrap()
        .content
        .push(b' ');
    assert_eq!(
        check_registered_compiler_identity(&changed),
        Err(EditionAdmissionFailure::InvalidIdentity)
    );
    assert_eq!(
        check_registered_report_editions(
            "agentflow.g01.report-shape@2",
            OBLIGATION_IDENTITY_EDITION
        ),
        Err(EditionAdmissionFailure::UnsupportedSemantics)
    );
    assert!(!current_registered_analysis_edition().can_issue_capability());
}
#[test]
fn actual_direct_analyzer_checker_and_private_issuance_boundary_deny() {
    let folder = root().join("spec/conformance/computation-root-export");
    let mut count = 0;
    for group in ["positive", "negative", "context", "subflow", "recovery"] {
        for entry in std::fs::read_dir(folder.join(group)).unwrap() {
            let p = entry.unwrap().path();
            if p.extension().is_none_or(|e| e != "json") {
                continue;
            }
            count += 1;
            let v: Value = serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap();
            let a = support::control_fixture::analyze_root_export_case(&v).unwrap();
            if a["stage"] == "component_validation_rejected" {
                assert_eq!(a["checked_component_produced"], false);
                continue;
            }
            assert_eq!(a["checked_static_compatibility_capability_issued"], false);
            assert_ne!(a["static_compatibility"], "proven");
            assert_eq!(a["body_conformance"], "requires_evidence");
            assert_eq!(a["runtime_authority"], "not_established");
        }
    }
    assert_eq!(count, 35);
}
