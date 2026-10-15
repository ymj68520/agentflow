#![recursion_limit = "512"]
#[allow(dead_code)]
mod support;
use af_computation::*;
use serde_json::Value;
use std::path::{Path, PathBuf};
fn corpus() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/conformance/g01-edition2-staging")
}
fn read(name: &str) -> Value {
    serde_json::from_slice(&std::fs::read(corpus().join(name)).unwrap()).unwrap()
}
fn identity(name: &str) -> CompilerSemanticsIdentity {
    let v = read(name);
    CompilerSemanticsIdentity {
        analysis_semantics_version: v["analysis_semantics_version"].as_str().unwrap().to_owned(),
        registry_content: hex::decode(v["registry_hex"].as_str().unwrap()).unwrap(),
        rule_registry_content: hex::decode(v["rule_registry_hex"].as_str().unwrap()).unwrap(),
        specifications: v["specifications"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| SpecificationContentIdentity {
                specification_id: s["specification_id"].as_str().unwrap().to_owned(),
                logical_path: s["logical_path"].as_str().unwrap().to_owned(),
                content_sha256: s["content_sha256"].as_str().unwrap().to_owned(),
                content: hex::decode(s["content_hex"].as_str().unwrap()).unwrap(),
            })
            .collect(),
        rules: v["rules"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| RuleContentIdentity {
                rule_id: r["rule_id"].as_str().unwrap().to_owned(),
                normative_source: r["normative_source"].as_str().unwrap().to_owned(),
                source_anchor: r["source_anchor"].as_str().unwrap().to_owned(),
                short_statement: r["short_statement"].as_str().unwrap().to_owned(),
            })
            .collect(),
    }
}
#[test]
fn exact_independent_edition_identity_and_distinct_staging_states() {
    let stage = current_registered_analysis_edition();
    assert_eq!(stage.compiler, identity("compiler-current.json"));
    assert_eq!(stage.compiler.specifications.len(), 20);
    assert_eq!(stage.compiler.rules.len(), 138);
    assert_eq!(stage.registered_sources.len(), 46);
    let registry: Value = serde_json::from_slice(&stage.compiler.registry_content).unwrap();
    for expected in registry["specifications"].as_array().unwrap() {
        let actual = stage
            .registered_sources
            .iter()
            .find(|source| {
                source.specification_id == expected["specification_id"].as_str().unwrap()
            })
            .unwrap();
        assert_eq!(
            actual.logical_path,
            expected["normative_source"]["logical_path"]
                .as_str()
                .unwrap()
        );
        assert_eq!(
            actual.content_sha256,
            expected["normative_source"]["content_digest"]["hex"]
                .as_str()
                .unwrap()
        );
        assert_eq!(
            actual.content,
            std::fs::read(corpus().join("../../..").join(&actual.logical_path)).unwrap()
        );
    }
    assert_eq!(stage.closed_tables.len(), 4);
    for (key, path) in [
        ("hash_domains", "spec/tables/hash-domains.json"),
        ("abi_numbers", "spec/registry/abi-numbers.json"),
        ("rule_areas", "spec/tables/rule-areas.json"),
        ("afidl_primitives", "spec/tables/afidl-primitives.json"),
    ] {
        assert_eq!(
            stage.closed_tables[key],
            std::fs::read(corpus().join("../../..").join(path)).unwrap()
        );
    }
    assert_eq!(stage.registration, EditionRegistration::Registered);
    assert_eq!(stage.support, EditionSupport::NotImplemented);
    assert_eq!(stage.activation, EditionActivation::NotAuthorized);
    assert_eq!(stage.release, EditionRelease::NotAuthorized);
    assert!(!stage.can_issue_capability());
    assert_eq!(stage.report_shape, "agentflow.g01.report-shape@1");
    assert_eq!(stage.obligation_edition, "agentflow.g01.obligation@1");
}
#[test]
fn independent_native_admission_matrix_is_fail_closed() {
    let base = identity("compiler-current.json");
    let old = identity("compiler-legacy.json");
    let manifest = read("manifest.json");
    let cases = manifest["native_cases"].as_array().unwrap();
    assert_eq!(cases.len(), 10);
    for c in cases {
        let mut claim = base.clone();
        let outcome = match c["mutation"].as_str().unwrap() {
            "current-identity" => check_registered_compiler_identity(&claim),
            "new-label-old-native-content" => {
                claim = old.clone();
                claim.analysis_semantics_version = ROOT_EXPORT_ANALYSIS_SEMANTICS_VERSION.into();
                check_registered_compiler_identity(&claim)
            }
            "old-label-new-native-content" => {
                claim.analysis_semantics_version = old.analysis_semantics_version.clone();
                check_registered_compiler_identity(&claim)
            }
            "old-identity" => check_registered_compiler_identity(&old),
            "unknown-native-edition" => {
                claim.analysis_semantics_version = "agentflow.root-export-checker@999".into();
                check_registered_compiler_identity(&claim)
            }
            "missing-selected-source" => {
                claim.specifications.pop();
                check_registered_compiler_identity(&claim)
            }
            "changed-rule-native-content" => {
                claim.rule_registry_content.push(b' ');
                check_registered_compiler_identity(&claim)
            }
            "identity-resource-limit" => {
                claim.registry_content = vec![0; 32 * 1024 * 1024 + 1];
                check_registered_compiler_identity(&claim)
            }
            "unknown-report-shape" => check_registered_report_editions(
                "agentflow.g01.report-shape@999",
                OBLIGATION_IDENTITY_EDITION,
            ),
            "unknown-obligation-edition" => check_registered_report_editions(
                REPORT_SHAPE_EDITION,
                "agentflow.g01.obligation@999",
            ),
            other => panic!("unimplemented test mutation {other}"),
        };
        let actual = outcome.err().map(|failure| format!("{failure:?}"));
        assert_eq!(
            actual.as_deref(),
            c["expected_failure"].as_str(),
            "{}",
            c["case_id"]
        );
        assert!(!current_registered_analysis_edition().can_issue_capability());
    }
}
#[test]
fn direct_checker_cannot_issue_for_even_a_complete_native_positive_declaration() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../spec/conformance/computation-root-export");
    let mut paths = Vec::new();
    for group in ["positive", "negative", "context", "subflow", "recovery"] {
        for item in std::fs::read_dir(root.join(group)).unwrap() {
            let p = item.unwrap().path();
            if p.extension().is_some_and(|x| x == "json") {
                paths.push(p);
            }
        }
    }
    assert_eq!(paths.len(), 35);
    for path in paths {
        let input: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let actual = support::control_fixture::analyze_root_export_case(&input).unwrap();
        if actual["stage"] == "component_validation_rejected" {
            assert_eq!(
                input["input"]["expected_stage"],
                "component_validation_rejected"
            );
            assert_eq!(actual["checked_component_produced"], false);
            continue;
        }
        assert_eq!(
            actual["checked_static_compatibility_capability_issued"],
            false
        );
        assert_ne!(actual["static_compatibility"], "proven");
        assert!(actual["_edition2_admission_failure"].as_str().is_some());
        assert_eq!(actual["body_conformance"], "requires_evidence");
        assert_eq!(actual["runtime_authority"], "not_established");
        assert_eq!(actual["publisher_authentication"], "not_established");
    }
}
