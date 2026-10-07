//! Semantic dataflow conformance harness (Phase 3C): consumes the FROZEN
//! corpus under `spec/conformance/semantic-dataflow/` (generated once by the
//! independent Python implementation `tools/gen-semantic-dataflow-corpus.py`).
//!
//! The corpus carries no digests — the analysis is derived-only and never
//! enters the `SemanticIrDigest`. These tests never generate expected values
//! from the implementation under test: positives pin the derived
//! EntryValues/EntryResources/WorkflowExitValues projections, negatives pin
//! the stable error kind and the diagnostic reason, and the frozen Phase 3B
//! golden corpus proves the analysis neither rejects shipped IR nor moves a
//! digest.

use af_ir::{SemanticDataflowAnalysis, SemanticIrDocument};
use af_semantic::TypeSchemaUniverse;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
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

fn load_json(relative: &str) -> serde_json::Value {
    serde_json::from_str(&read(relative)).unwrap_or_else(|e| panic!("cannot parse {relative}: {e}"))
}

const CORPUS_REVISION: u64 = 1;

/// The normative schema module set for the digest identity check.
fn module_set() -> af_idl::CompiledModuleSet {
    let sources = [
        (
            "semantic-core.afidl".to_string(),
            read("spec/idl/semantic-core.afidl"),
        ),
        (
            "semantic-ir.afidl".to_string(),
            read("spec/idl/semantic-ir.afidl"),
        ),
    ];
    af_idl::compile_module_set(&sources).expect("the normative module set compiles")
}

/// The type schema universe from the shared Phase 3A fixture (same file the
/// Phase 3B golden corpus pins).
fn fixture_universe() -> TypeSchemaUniverse {
    let golden = load_json("spec/conformance/semantic-ir/golden/semantic-ir-golden.json");
    let compiled = af_idl::compile(&read(golden["fixture_schema"]["file"].as_str().unwrap()))
        .expect("the fixture schema compiles");
    let mut universe = TypeSchemaUniverse::new();
    universe
        .insert_compiled(&compiled)
        .expect("the fixture registers");
    universe
}

#[derive(Deserialize)]
struct PositiveFile {
    corpus_revision: u64,
    cases: Vec<PositiveCase>,
}

#[derive(Deserialize)]
struct PositiveCase {
    case_id: String,
    document: SemanticIrDocument,
    entry_values: BTreeMap<String, Vec<u32>>,
    entry_resources: BTreeMap<String, Vec<u32>>,
    workflow_exit_values: Vec<u32>,
}

#[derive(Deserialize)]
struct NegativeFile {
    corpus_revision: u64,
    cases: Vec<NegativeCase>,
}

#[derive(Deserialize)]
struct NegativeCase {
    case_id: String,
    document: serde_json::Value,
    error_kind: String,
    reason: Option<String>,
    rejected_at: String,
}

fn sorted(set: &BTreeSet<u32>) -> Vec<u32> {
    set.iter().copied().collect()
}

fn assert_observations_match(
    case_id: &str,
    expected: &PositiveCase,
    analysis: &SemanticDataflowAnalysis,
) {
    let entry_values: BTreeMap<String, Vec<u32>> = analysis
        .entry_values()
        .iter()
        .map(|(computation, slots)| (computation.to_string(), sorted(slots)))
        .collect();
    assert_eq!(
        entry_values, expected.entry_values,
        "{case_id}: EntryValues projection drifted"
    );
    let entry_resources: BTreeMap<String, Vec<u32>> = analysis
        .entry_resources()
        .iter()
        .map(|(computation, slots)| (computation.to_string(), sorted(slots)))
        .collect();
    assert_eq!(
        entry_resources, expected.entry_resources,
        "{case_id}: EntryResources projection drifted"
    );
    assert_eq!(
        sorted(analysis.workflow_exit_values()),
        expected.workflow_exit_values,
        "{case_id}: WorkflowExitValues drifted"
    );
}

#[test]
fn positives_validate_and_reproduce_frozen_observations() {
    let file: PositiveFile = serde_json::from_value(load_json(
        "spec/conformance/semantic-dataflow/positive/semantic-dataflow-positive.json",
    ))
    .expect("the positive file is valid");
    assert_eq!(file.corpus_revision, CORPUS_REVISION);
    assert!(file.cases.len() >= 16, "the positive corpus shrank");

    let universe = fixture_universe();
    for case in &file.cases {
        let checked = case
            .document
            .check(&universe)
            .unwrap_or_else(|e| panic!("{}: {} must pass Phase 3B", case.case_id, e));
        let analysis = checked
            .analyze_dataflow()
            .unwrap_or_else(|e| panic!("{}: {} must pass the analysis", case.case_id, e));
        assert_observations_match(&case.case_id, case, &analysis);
    }
}

#[test]
fn negatives_fail_only_at_the_analysis_layer() {
    let file: NegativeFile = serde_json::from_value(load_json(
        "spec/conformance/semantic-dataflow/negative/semantic-dataflow-negative.json",
    ))
    .expect("the negative file is valid");
    assert_eq!(file.corpus_revision, CORPUS_REVISION);
    assert!(file.cases.len() >= 30, "the negative corpus shrank");

    let universe = fixture_universe();
    for case in &file.cases {
        assert_eq!(
            case.rejected_at, "dataflow",
            "{}: every dataflow negative must pass Phase 3B first",
            case.case_id
        );
        let document: SemanticIrDocument = serde_json::from_value(case.document.clone())
            .unwrap_or_else(|e| panic!("{}: {e}", case.case_id));
        checked_or_panic(&document, &universe, &case.case_id);
        let error = document
            .check(&universe)
            .ok()
            .and_then(|checked| checked.analyze_dataflow().err())
            .unwrap_or_else(|| panic!("{}: must be rejected by the analysis", case.case_id));
        assert_eq!(
            error.kind(),
            case.error_kind,
            "{}: error kind mismatch ({error})",
            case.case_id
        );
        if let Some(reason) = &case.reason {
            let detail = match &error {
                af_ir::IrError::Rule { detail, .. } => detail.clone(),
                other => panic!("{}: unexpected error shape {other:?}", case.case_id),
            };
            assert!(
                detail.contains(&format!("reason: {reason}")),
                "{}: expected reason {reason} in {detail:?}",
                case.case_id
            );
        }
    }
}

fn checked_or_panic(document: &SemanticIrDocument, universe: &TypeSchemaUniverse, case_id: &str) {
    if let Err(error) = document.check(universe) {
        panic!("{case_id}: must be Phase 3B-valid (the analysis runs later): {error}");
    }
}

#[test]
fn phase3b_goldens_pass_the_analysis_with_unchanged_digests() {
    let golden: serde_json::Value =
        load_json("spec/conformance/semantic-ir/golden/semantic-ir-golden.json");
    let universe = fixture_universe();
    let set = module_set();
    let cases = golden["golden"].as_array().unwrap();
    assert!(cases.len() >= 16, "the golden corpus shrank");
    for case in cases {
        let case_id = case["case_id"].as_str().unwrap();
        let document: SemanticIrDocument = serde_json::from_value(case["document"].clone())
            .unwrap_or_else(|e| panic!("{case_id}: {e}"));
        let digest_before = document.digest(&set).unwrap();
        let checked = document
            .check(&universe)
            .unwrap_or_else(|e| panic!("{case_id}: {e}"));
        checked
            .analyze_dataflow()
            .unwrap_or_else(|e| panic!("{case_id}: the shipped golden must stay analyzable: {e}"));
        let digest_after = document.digest(&set).unwrap();
        assert_eq!(
            digest_before, digest_after,
            "{case_id}: running the analysis moved the SemanticIrDigest"
        );
        assert_eq!(
            digest_after.hex(),
            case["semantic_ir_digest"].as_str().unwrap(),
            "{case_id}: golden digest drifted"
        );
    }
}

#[test]
fn the_analysis_is_deterministic_across_runs() {
    let file: PositiveFile = serde_json::from_value(load_json(
        "spec/conformance/semantic-dataflow/positive/semantic-dataflow-positive.json",
    ))
    .unwrap();
    let universe = fixture_universe();
    for case in &file.cases {
        let checked = case.document.check(&universe).unwrap();
        let first = checked.analyze_dataflow().unwrap();
        let second = checked.analyze_dataflow().unwrap();
        assert_eq!(first, second, "{}", case.case_id);
    }
}

/// The producer location projection is complete: every declared value slot of
/// every positive document has exactly one location (Phase 3B uniqueness,
/// re-derived by the analysis).
#[test]
fn producer_locations_cover_every_declared_slot() {
    let file: PositiveFile = serde_json::from_value(load_json(
        "spec/conformance/semantic-dataflow/positive/semantic-dataflow-positive.json",
    ))
    .unwrap();
    let universe = fixture_universe();
    for case in &file.cases {
        let checked = case.document.check(&universe).unwrap();
        let analysis = checked.analyze_dataflow().unwrap();
        let declared: BTreeSet<u32> = case
            .document
            .values
            .iter()
            .map(|slot| slot.id.raw())
            .collect();
        let located: BTreeSet<u32> = analysis.producer_locations().keys().copied().collect();
        assert_eq!(declared, located, "{}", case.case_id);
    }
}
