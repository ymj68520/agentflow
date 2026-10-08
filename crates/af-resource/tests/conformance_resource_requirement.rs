//! Resource-requirement conformance harness (Phase 4A.3): consumes the
//! FROZEN corpus under `spec/conformance/resource-requirement/`
//! (generated once by the independent Python implementation
//! `tools/gen-resource-requirement-corpus.py`).
//!
//! Expected canonical ACE1 bytes, error kinds, compatibility verdicts and
//! equivalence groups all come from the corpus; tests never generate
//! expected values from the implementation under test.

use af_resource::{
    ResourceCandidate, ResourceCompatibilityProof, ResourceRequirement, ResourceRequirementRow,
    ResourceTypeDescriptor, ResourceTypeUniverse, requirement_not_stronger,
    satisfies_resource_requirement,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

const CORPUS_REVISION: u64 = 1;

fn case_files(subdir: &str) -> Vec<serde_json::Value> {
    let directory = repo_root()
        .join("spec/conformance/resource-requirement")
        .join(subdir);
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&directory)
        .unwrap_or_else(|e| panic!("cannot list {}: {e}", directory.display()))
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    paths.sort();
    paths
        .iter()
        .map(|path| {
            serde_json::from_str(
                &std::fs::read_to_string(path)
                    .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display())),
            )
            .unwrap_or_else(|e| panic!("cannot parse {}: {e}", path.display()))
        })
        .collect()
}

fn module_set() -> af_idl::CompiledModuleSet {
    let sources = [
        (
            "semantic-core.afidl".to_string(),
            read("spec/idl/semantic-core.afidl"),
        ),
        (
            "resource-requirement.afidl".to_string(),
            read("spec/idl/resource-requirement.afidl"),
        ),
    ];
    af_idl::compile_module_set(&sources).expect("the normative module set compiles")
}

/// The explicit fixture universe of one case, from its frozen fixture
/// block: schemas compiled and pinned against the frozen digests, plus
/// the frozen recognition descriptors.
fn fixture_universe(case: &serde_json::Value) -> ResourceTypeUniverse {
    let mut universe = ResourceTypeUniverse::new();
    for schema in case["fixture_schemas"].as_array().unwrap() {
        let compiled = af_idl::compile(schema["source"].as_str().unwrap())
            .unwrap_or_else(|e| panic!("the corpus fixture schema compiles: {e}"));
        for expected in schema["types"].as_array().unwrap() {
            let digest = compiled
                .digest(expected["name"].as_str().unwrap())
                .unwrap_or_else(|| panic!("fixture type missing"));
            assert_eq!(
                digest.hex(),
                expected["digest_hex"].as_str().unwrap(),
                "corpus fixture digest drifted"
            );
        }
        universe
            .insert_compiled(&compiled)
            .expect("the corpus fixture registers");
    }
    for descriptor in case["resource_types"].as_array().unwrap() {
        let recognized: BTreeSet<String> = descriptor["recognized_attributes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|key| key.as_str().unwrap().to_string())
            .collect();
        universe
            .recognize(ResourceTypeDescriptor {
                specification_id: descriptor["specification_id"].as_str().unwrap().to_string(),
                type_name: descriptor["type_name"].as_str().unwrap().to_string(),
                schema_hex: descriptor["digest_hex"].as_str().unwrap().to_string(),
                recognized_attributes: recognized,
            })
            .expect("the corpus descriptor recognizes");
    }
    universe
}

fn row_of(case: &serde_json::Value, key: &str) -> ResourceRequirementRow {
    serde_json::from_value(case[key].clone())
        .unwrap_or_else(|e| panic!("{}: requirement row parses: {e}", case["case_id"]))
}

fn candidate_of(value: &serde_json::Value) -> ResourceCandidate {
    ResourceCandidate {
        resource_type: serde_json::from_value(value["resource_type"].clone())
            .unwrap_or_else(|e| panic!("candidate type parses: {e}")),
        attributes: serde_json::from_value(value["attributes"].clone())
            .unwrap_or_else(|e| panic!("candidate attributes parse: {e}")),
    }
}

fn validated(case: &serde_json::Value, key: &str) -> ResourceRequirementRow {
    let universe = fixture_universe(case);
    row_of(case, key)
        .validate(&universe)
        .unwrap_or_else(|e| panic!("{}: validates: {e}", case["case_id"]))
}

fn assert_canonical_bytes(case_id: &str, row: &ResourceRequirementRow, frozen: &str) {
    let bytes = row
        .canonical_bytes(&module_set())
        .unwrap_or_else(|e| panic!("{case_id}: canonical bytes: {e}"));
    assert_eq!(
        String::from_utf8(bytes).unwrap_or_else(|e| panic!("{case_id}: utf8: {e}")),
        frozen,
        "{case_id}: canonical ACE1 bytes drifted from the frozen oracle"
    );
}

// ---------------------------------------------------------------------------
// Golden
// ---------------------------------------------------------------------------

#[test]
fn golden_cases_match_the_frozen_canonical_bytes() {
    let files = case_files("golden");
    assert!(files.len() >= 8, "the golden corpus shrank");
    for case in &files {
        let case_id = case["case_id"].as_str().unwrap();
        assert_eq!(
            case["corpus_revision"].as_u64(),
            Some(CORPUS_REVISION),
            "{case_id}: corpus revision drifted"
        );
        let row = validated(case, "row");
        assert_canonical_bytes(case_id, &row, case["canonical_ace1_utf8"].as_str().unwrap());
        assert_eq!(
            row.is_empty(),
            case["is_empty"].as_bool().unwrap(),
            "{case_id}: is_empty"
        );
        assert_eq!(
            row.len(),
            usize::try_from(case["requirement_count"].as_u64().unwrap()).unwrap(),
            "{case_id}: count"
        );
        let twice = row.canonical().unwrap().canonical().unwrap();
        assert_eq!(twice, row, "{case_id}: canonicalization is not idempotent");
    }
}

// ---------------------------------------------------------------------------
// Negative
// ---------------------------------------------------------------------------

#[test]
fn negative_cases_are_rejected_with_the_frozen_kinds() {
    let files = case_files("negative");
    assert!(files.len() >= 11, "the negative corpus shrank");
    for case in &files {
        let case_id = case["case_id"].as_str().unwrap();
        let expected = case["expected_kind"].as_str().unwrap();
        if case["operation"] == "satisfies" {
            let requirement: ResourceRequirement =
                serde_json::from_value(case["requirement"].clone())
                    .unwrap_or_else(|e| panic!("{case_id}: requirement parses: {e}"));
            let error =
                satisfies_resource_requirement(&candidate_of(&case["candidate"]), &requirement)
                    .expect_err(&format!("{case_id}: must be rejected"));
            assert_eq!(error.kind(), expected, "{case_id}: rejection kind drifted");
            continue;
        }
        let universe = fixture_universe(case);
        let error = row_of(case, "row")
            .validate(&universe)
            .expect_err(&format!("{case_id}: must be rejected"));
        assert_eq!(error.kind(), expected, "{case_id}: rejection kind drifted");
    }
}

// ---------------------------------------------------------------------------
// Compatibility + satisfaction
// ---------------------------------------------------------------------------

#[test]
fn compatibility_cases_reproduce_the_frozen_verdicts() {
    let files = case_files("compatibility");
    assert!(files.len() >= 12, "the compatibility corpus shrank");
    for case in &files {
        let case_id = case["case_id"].as_str().unwrap();
        let universe = fixture_universe(case);
        let expected = &case["expected"];
        if case["operation"] == "satisfies" {
            let requirement: ResourceRequirement =
                serde_json::from_value(case["requirement"].clone())
                    .unwrap_or_else(|e| panic!("{case_id}: requirement parses: {e}"));
            let result =
                satisfies_resource_requirement(&candidate_of(&case["candidate"]), &requirement);
            match expected["outcome"].as_str().unwrap() {
                "satisfied" => {
                    result.unwrap_or_else(|e| panic!("{case_id}: must satisfy: {e}"));
                }
                "unsatisfied" => {
                    let error = result.expect_err(&format!("{case_id}: must not satisfy"));
                    assert_eq!(
                        error.kind(),
                        "resource_requirement_unsatisfied",
                        "{case_id}: rejection kind drifted"
                    );
                }
                other => panic!("{other}: unexpected satisfaction outcome"),
            }
            continue;
        }
        let result = requirement_not_stronger(
            &row_of(case, "new_row"),
            case["new_namespace"].as_str().unwrap(),
            &row_of(case, "old_row"),
            case["old_namespace"].as_str().unwrap(),
            &universe,
        );
        match expected["outcome"].as_str().unwrap() {
            "proven" => {
                assert_eq!(
                    result.expect("{case_id}: must prove"),
                    ResourceCompatibilityProof::Proven,
                    "{case_id}: verdict drifted"
                );
            }
            "unproven" => {
                let reason = match result.expect("{case_id}: must decide") {
                    ResourceCompatibilityProof::Unproven(reason) => reason,
                    ResourceCompatibilityProof::Proven => {
                        panic!("{case_id}: unexpectedly proven")
                    }
                };
                assert_eq!(
                    reason.as_str(),
                    expected["reason"].as_str().unwrap(),
                    "{case_id}: unproven reason drifted"
                );
            }
            "strengthened" => {
                let error = result.expect_err(&format!("{case_id}: must be rejected"));
                assert_eq!(
                    error.kind(),
                    "resource_requirement_strengthened",
                    "{case_id}: rejection kind drifted"
                );
            }
            "cross_namespace" => {
                let error = result.expect_err(&format!("{case_id}: must fail closed"));
                assert_eq!(
                    error.kind(),
                    "resource_requirement_unproven",
                    "{case_id}: rejection kind drifted"
                );
            }
            other => panic!("{other}: unexpected compatibility outcome"),
        }
    }
}

// ---------------------------------------------------------------------------
// Equivalence
// ---------------------------------------------------------------------------

#[test]
fn equivalence_groups_have_identical_canonical_bytes() {
    let files = case_files("equivalence");
    assert!(files.len() >= 3, "the equivalence corpus shrank");
    for case in &files {
        let case_id = case["case_id"].as_str().unwrap();
        let universe = fixture_universe(case);
        let mut reference: Option<String> = None;
        for member in case["members"].as_array().unwrap() {
            let row: ResourceRequirementRow = serde_json::from_value(member.clone())
                .unwrap_or_else(|e| panic!("{case_id}: member parses: {e}"));
            let canonical = row
                .validate(&universe)
                .unwrap_or_else(|e| panic!("{case_id}: member validates: {e}"));
            let bytes = canonical
                .canonical_bytes(&module_set())
                .unwrap_or_else(|e| panic!("{case_id}: bytes: {e}"));
            let text = String::from_utf8(bytes).unwrap();
            match &reference {
                None => {
                    reference = Some(text.clone());
                    assert_eq!(
                        text,
                        case["canonical_ace1_utf8"].as_str().unwrap(),
                        "{case_id}: frozen canonical bytes drifted"
                    );
                }
                Some(expected) => {
                    assert_eq!(&text, expected, "{case_id}: equivalence diverged");
                }
            }
        }
    }
}
