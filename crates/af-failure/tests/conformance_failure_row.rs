//! Failure-row conformance harness (Phase 4A.2): consumes the FROZEN
//! corpus under `spec/conformance/failure-row/` (generated once by the
//! independent Python implementation `tools/gen-failure-row-corpus.py`).
//!
//! Expected canonical ACE1 bytes, error kinds, algebra results and
//! equivalence groups all come from the corpus; tests never generate
//! expected values from the implementation under test.

use af_failure::{
    FailureRow, StaticFailureEnvelope, await_row, compensation_scope_row, contains, handle_row,
    iterate_row, parallel_row, race_row, remove_handled, retry_row, scope_row, select_row,
    sequence_row, timeout_row, union,
};
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
        .join("spec/conformance/failure-row")
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
            "failure-row.afidl".to_string(),
            read("spec/idl/failure-row.afidl"),
        ),
    ];
    af_idl::compile_module_set(&sources).expect("the normative module set compiles")
}

/// The explicit fixture schema universe of one case, from its frozen
/// fixture block: sources compiled and pinned against the frozen digests.
fn fixture_universe(case: &serde_json::Value) -> af_semantic::TypeSchemaUniverse {
    let mut universe = af_semantic::TypeSchemaUniverse::new();
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
    universe
}

fn row_of(case: &serde_json::Value, key: &str) -> FailureRow {
    serde_json::from_value(case[key].clone())
        .unwrap_or_else(|e| panic!("{}: failure row parses: {e}", case["case_id"]))
}

fn validated(case: &serde_json::Value, key: &str) -> FailureRow {
    let universe = fixture_universe(case);
    row_of(case, key)
        .validate(&universe)
        .unwrap_or_else(|e| panic!("{}: validates: {e}", case["case_id"]))
}

fn assert_canonical_bytes(case_id: &str, row: &FailureRow, frozen: &str) {
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
        let row = validated(case, "signature");
        assert_canonical_bytes(case_id, &row, case["canonical_ace1_utf8"].as_str().unwrap());
        assert_eq!(
            row.is_empty(),
            case["is_empty"].as_bool().unwrap(),
            "{case_id}: is_empty"
        );
        assert_eq!(
            row.len(),
            usize::try_from(case["failure_count"].as_u64().unwrap()).unwrap(),
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
    assert!(files.len() >= 7, "the negative corpus shrank");
    for case in &files {
        let case_id = case["case_id"].as_str().unwrap();
        let expected = case["expected_kind"].as_str().unwrap();
        if case["operation"] == "require_proven_row" {
            let error = StaticFailureEnvelope::unproven()
                .require_row()
                .expect_err("{case_id}: Unproven must fail closed");
            assert_eq!(error.kind(), expected, "{case_id}: kind drifted");
            continue;
        }
        let universe = fixture_universe(case);
        if let Some(operation) = case["operation"].as_str() {
            let operands: Vec<FailureRow> = case["operands"]
                .as_array()
                .unwrap()
                .iter()
                .map(|operand| {
                    let row: FailureRow = serde_json::from_value(operand.clone())
                        .unwrap_or_else(|e| panic!("{case_id}: operand parses: {e}"));
                    row.validate(&universe)
                        .unwrap_or_else(|e| panic!("{case_id}: operand must validate: {e}"))
                })
                .collect();
            let result = apply_operation(operation, &operands, &[]);
            let error = result.expect_err(&format!("{case_id}: must be rejected"));
            assert_eq!(error.kind(), expected, "{case_id}: rejection kind drifted");
        } else {
            let error = row_of(case, "signature")
                .validate(&universe)
                .expect_err(&format!("{case_id}: must be rejected"));
            assert_eq!(error.kind(), expected, "{case_id}: rejection kind drifted");
        }
    }
}

// ---------------------------------------------------------------------------
// Algebra + control propagation
// ---------------------------------------------------------------------------

fn apply_operation(
    operation: &str,
    operands: &[FailureRow],
    handlers: &[FailureRow],
) -> Result<FailureRow, af_failure::FailureError> {
    match operation {
        "union" => Ok(union(&operands[0], &operands[1])),
        "contains" => unreachable!("contains is asserted, not applied"),
        "remove_handled" => remove_handled(&operands[0], &operands[1]),
        "handle_propagate" => handle_row(&operands[0], &operands[1], handlers),
        "sequence_union" => Ok(sequence_row(operands)),
        "select_union" => Ok(select_row(operands)),
        "parallel_union" => Ok(parallel_row(operands)),
        "race_union" => Ok(race_row(operands)),
        "iterate_row" => Ok(iterate_row(&operands[0])),
        "retry_row" => Ok(retry_row(&operands[0])),
        "timeout_row" => Ok(timeout_row(&operands[0])),
        "scope_row" => Ok(scope_row(&operands[0])),
        "compensation_scope_row" => Ok(compensation_scope_row(&operands[0])),
        "await_row" => Ok(await_row()),
        other => panic!("{other}: unexpected failure-row operation"),
    }
}

#[test]
fn algebra_cases_reproduce_the_frozen_results() {
    let files = case_files("algebra");
    assert!(files.len() >= 15, "the algebra corpus shrank");
    for case in &files {
        let case_id = case["case_id"].as_str().unwrap();
        let universe = fixture_universe(case);
        let expected = &case["expected"];

        let parse = |index: usize| -> FailureRow {
            let raw: FailureRow = serde_json::from_value(case["operands"][index].clone())
                .unwrap_or_else(|e| panic!("{case_id}: operand parses: {e}"));
            raw.validate(&universe)
                .unwrap_or_else(|e| panic!("{case_id}: operand validates: {e}"))
        };

        if case["operation"] == "contains" {
            let operands: Vec<FailureRow> = (0..case["operands"].as_array().unwrap().len())
                .map(parse)
                .collect();
            if let Some(pairs) = expected["chain_pairs"].as_array() {
                for pair in pairs {
                    let i = usize::try_from(pair[0].as_u64().unwrap()).unwrap();
                    let j = usize::try_from(pair[1].as_u64().unwrap()).unwrap();
                    assert!(
                        contains(&operands[i], &operands[j]),
                        "{case_id}: pair {i} -> {j} must be contained"
                    );
                }
            } else {
                assert!(
                    contains(&operands[0], &operands[0]),
                    "{case_id}: reflexivity must hold"
                );
            }
            continue;
        }

        if let Some(error_kind) = expected["error_kind"].as_str() {
            let operands: Vec<FailureRow> = (0..case["operands"].as_array().unwrap().len())
                .map(parse)
                .collect();
            let result = apply_operation(case["operation"].as_str().unwrap(), &operands, &[]);
            let error = result.expect_err(&format!("{case_id}: must fail"));
            assert_eq!(error.kind(), error_kind, "{case_id}: error kind drifted");
            continue;
        }

        let operands: Vec<FailureRow> = (0..case["operands"].as_array().unwrap().len())
            .map(parse)
            .collect();
        let handlers: Vec<FailureRow> = case["handlers"]
            .as_array()
            .unwrap()
            .iter()
            .map(|handler| {
                let row: FailureRow = serde_json::from_value(handler.clone())
                    .unwrap_or_else(|e| panic!("{case_id}: handler parses: {e}"));
                row.validate(&universe)
                    .unwrap_or_else(|e| panic!("{case_id}: handler validates: {e}"))
            })
            .collect();
        // The associativity frozen result belongs to the full grouping,
        // not to a two-operand prefix application.
        if case["property"].as_str() != Some("associativity") {
            let result = apply_operation(case["operation"].as_str().unwrap(), &operands, &handlers)
                .unwrap_or_else(|e| panic!("{case_id}: operation failed: {e}"));
            assert_canonical_bytes(
                case_id,
                &result,
                expected["canonical_ace1_utf8"].as_str().unwrap(),
            );
        }

        // Frozen algebraic properties, re-checked across groupings.
        let combine = |a: &FailureRow, b: &FailureRow| -> FailureRow {
            match case["operation"].as_str().unwrap() {
                "union" => union(a, b),
                other => panic!("{other}: no algebraic property defined"),
            }
        };
        match case["property"].as_str() {
            Some("associativity") => {
                let (a, b, c) = (parse(0), parse(1), parse(2));
                let left = combine(&combine(&a, &b), &c);
                let right = combine(&a, &combine(&b, &c));
                assert_canonical_bytes(
                    case_id,
                    &left,
                    expected["canonical_ace1_utf8"].as_str().unwrap(),
                );
                assert_canonical_bytes(
                    case_id,
                    &right,
                    expected["canonical_ace1_utf8"].as_str().unwrap(),
                );
            }
            Some("identity" | "commutativity") => {
                let backwards = combine(&parse(1), &parse(0));
                assert_canonical_bytes(
                    case_id,
                    &backwards,
                    expected["canonical_ace1_utf8"].as_str().unwrap(),
                );
            }
            Some("idempotence") => {
                let again = combine(&parse(0), &parse(0));
                assert_canonical_bytes(
                    case_id,
                    &again,
                    expected["canonical_ace1_utf8"].as_str().unwrap(),
                );
            }
            _ => {}
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
            let row: FailureRow = serde_json::from_value(member.clone())
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
