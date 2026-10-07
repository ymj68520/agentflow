//! spec-registry conformance harness: consumes the FROZEN corpus under
//! `spec/conformance/spec-registry/` (generated once by the independent
//! authoring script `tools/gen-spec-registry-corpus.py`). Valid cases must
//! validate cleanly; invalid cases must produce at least one error whose
//! stable kind matches the manifest expectation.

use af_spec::{Layout, SpecCheck};
use serde::Deserialize;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

#[derive(Deserialize)]
struct Manifest {
    #[allow(dead_code)]
    corpus_revision: u64,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[allow(clippy::struct_field_names)] // mirrors the manifest JSON field names
struct Case {
    case_id: String,
    dir: String,
    outcome: String,
    #[serde(default)]
    error_kind: Option<String>,
}

#[test]
fn corpus_revision_is_frozen() {
    let manifest: Manifest = serde_json::from_str(
        &std::fs::read_to_string(repo_root().join("spec/conformance/spec-registry/manifest.json"))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        manifest.corpus_revision, 1,
        "spec-registry corpus revision drifted"
    );
}

#[test]
fn every_corpus_case_matches_its_frozen_expectation() {
    let manifest: Manifest = serde_json::from_str(
        &std::fs::read_to_string(repo_root().join("spec/conformance/spec-registry/manifest.json"))
            .unwrap(),
    )
    .unwrap();
    assert!(manifest.cases.len() >= 18, "corpus shrank unexpectedly");

    for case in &manifest.cases {
        let case_root = repo_root()
            .join("spec/conformance/spec-registry")
            .join(&case.dir);
        let report = SpecCheck::with_layout(&case_root, Layout::corpus_case()).run();
        match case.outcome.as_str() {
            "valid" => {
                assert!(
                    report.is_ok(),
                    "case {} must validate cleanly, got: {:?}",
                    case.case_id,
                    report.errors
                );
            }
            "invalid" => {
                let expected = case.error_kind.as_deref().unwrap_or_else(|| {
                    panic!(
                        "case {} is invalid but declares no error_kind",
                        case.case_id
                    )
                });
                assert!(
                    !report.is_ok(),
                    "case {} must fail validation but passed",
                    case.case_id
                );
                assert!(
                    report.errors.iter().any(|error| error.kind() == expected),
                    "case {} expected error kind {expected:?}, got: {:?}",
                    case.case_id,
                    report.errors.iter().map(af_spec::SpecError::kind).collect::<Vec<_>>()
                );
            }
            other => panic!("case {} has unknown outcome {other:?}", case.case_id),
        }
    }
}

#[test]
fn the_real_repo_rule_refs_resolve_against_the_whole_corpus() {
    // Sanity net: the frozen universe scan must contain the case ids the
    // real rule registry references (spec-registry/*, encoding/*, digest/*,
    // parser-invalid stems, suites).
    let report = SpecCheck::repo(repo_root()).run();
    assert!(
        report
            .errors
            .iter()
            .all(|error| error.kind() != "unknown_conformance_case"),
        "repo rule conformance refs must all resolve: {:?}",
        report
            .errors
            .iter()
            .filter(|e| e.kind() == "unknown_conformance_case")
            .collect::<Vec<_>>()
    );
}
