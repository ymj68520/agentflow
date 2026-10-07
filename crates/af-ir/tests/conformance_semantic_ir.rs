//! Semantic IR conformance harness: consumes the FROZEN corpus under
//! `spec/conformance/semantic-ir/` (generated once by the independent Python
//! implementation `tools/gen-semantic-ir-corpus.py`).
//!
//! Expected canonical bytes and digests are physically frozen; these tests
//! never generate expected values from the implementation under test.

use af_ir::SemanticIrDocument;
use af_semantic::TypeSchemaUniverse;
use serde::Deserialize;
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

fn load_json(relative: &str) -> serde_json::Value {
    serde_json::from_str(&read(relative)).unwrap_or_else(|e| panic!("cannot parse {relative}: {e}"))
}

const CORPUS_REVISION: u64 = 1;

/// The normative schema module set: the semantic IR module plus the semantic
/// core it imports, compiled as a set (the Phase 2C import path).
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

/// The type schema universe from the shared Phase 3A fixture, with the frozen
/// fixture digests asserted against the independent expectations.
fn fixture_universe() -> TypeSchemaUniverse {
    let golden = load_json("spec/conformance/semantic-ir/golden/semantic-ir-golden.json");
    let compiled = af_idl::compile(&read(golden["fixture_schema"]["file"].as_str().unwrap()))
        .expect("the fixture schema compiles");
    for expected in golden["fixture_schema"]["types"].as_array().unwrap() {
        let name = expected["name"].as_str().unwrap();
        let digest = compiled
            .digest(name)
            .unwrap_or_else(|| panic!("fixture type {name} missing"));
        assert_eq!(
            digest.hex(),
            expected["digest_hex"].as_str().unwrap(),
            "fixture digest for {name} drifted"
        );
    }
    let mut universe = TypeSchemaUniverse::new();
    universe
        .insert_compiled(&compiled)
        .expect("the fixture registers");
    universe
}

#[derive(Deserialize)]
struct GoldenFile {
    corpus_revision: u64,
    golden: Vec<GoldenCase>,
}

#[derive(Deserialize)]
struct GoldenCase {
    case_id: String,
    document: SemanticIrDocument,
    ace1_utf8: String,
    ace1_hex: String,
    semantic_ir_digest: String,
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
    rejected_at: String,
}

#[derive(Deserialize)]
struct EquivalenceFile {
    corpus_revision: u64,
    ordering_groups: Vec<OrderingGroup>,
    mutations: Vec<Mutation>,
}

#[derive(Deserialize)]
struct OrderingGroup {
    case_id: String,
    #[allow(dead_code)]
    claim: String,
    base_document: SemanticIrDocument,
    member_document: SemanticIrDocument,
    expected_digest: String,
}

#[derive(Deserialize)]
struct Mutation {
    case_id: String,
    #[allow(dead_code)]
    claim: String,
    base_document: SemanticIrDocument,
    mutated_document: SemanticIrDocument,
    base_digest: String,
    mutated_digest: String,
}

#[test]
fn golden_documents_reproduce_frozen_bytes_and_digests() {
    let golden: GoldenFile = serde_json::from_value(load_json(
        "spec/conformance/semantic-ir/golden/semantic-ir-golden.json",
    ))
    .expect("the golden file is valid");
    assert_eq!(golden.corpus_revision, CORPUS_REVISION);
    assert!(golden.golden.len() >= 16, "the golden corpus shrank");

    let universe = fixture_universe();
    let set = module_set();
    let mut covered: BTreeSet<String> = BTreeSet::new();

    for case in &golden.golden {
        let checked = case
            .document
            .check(&universe)
            .unwrap_or_else(|e| panic!("{}: {} must validate", case.case_id, e));
        let bytes = checked
            .source()
            .canonical_bytes(&set)
            .unwrap_or_else(|e| panic!("{}: {e}", case.case_id));
        assert_eq!(
            hex::encode(&bytes),
            case.ace1_hex,
            "{}: canonical bytes drifted\nexpected: {}\nactual:   {}",
            case.case_id,
            case.ace1_utf8,
            String::from_utf8_lossy(&bytes)
        );
        assert_eq!(bytes, case.ace1_utf8.as_bytes());
        let digest = checked
            .source()
            .digest(&set)
            .unwrap_or_else(|e| panic!("{}: {e}", case.case_id));
        assert_eq!(
            digest.hex(),
            case.semantic_ir_digest,
            "{}: SemanticIrDigest drifted",
            case.case_id
        );
        assert_eq!(digest.domain().as_str(), "semantic-ir");
        covered.insert(opcode_of(&case.document).to_string());
    }
    // Every one of the 13 opcodes has at least one golden.
    for opcode in [
        "invoke",
        "await",
        "sequence",
        "select",
        "parallel",
        "race",
        "iterate",
        "retry",
        "timeout",
        "handle",
        "scope",
        "compensation_scope",
        "call_subflow",
    ] {
        assert!(covered.contains(opcode), "opcode {opcode} has no golden");
    }
}

/// The top-level opcode of a document's root computation (for coverage
/// accounting only).
fn opcode_of(document: &SemanticIrDocument) -> &'static str {
    let root = document
        .computations
        .iter()
        .find(|computation| computation.id == document.root)
        .expect("root exists in every golden");
    root.body.case_name()
}

#[test]
fn nested_goldens_exist() {
    let golden: GoldenFile = serde_json::from_value(load_json(
        "spec/conformance/semantic-ir/golden/semantic-ir-golden.json",
    ))
    .unwrap();
    let ids: Vec<&str> = golden
        .golden
        .iter()
        .map(|case| case.case_id.as_str())
        .collect();
    for expected in [
        "nested/sequence-invoke-await",
        "nested/retry-timeout-invoke",
        "nested/scope-parallel",
        "nested/compensation-scope-sequence",
        "nested/select-retry-call-subflow",
        "nested/while-sequence",
    ] {
        assert!(ids.contains(&expected), "missing nested golden {expected}");
    }
}

#[test]
fn negative_cases_fail_with_frozen_kinds() {
    let negative: NegativeFile = serde_json::from_value(load_json(
        "spec/conformance/semantic-ir/negative/semantic-ir-negative.json",
    ))
    .expect("the negative file is valid");
    assert_eq!(negative.corpus_revision, CORPUS_REVISION);
    assert!(negative.cases.len() >= 40, "the negative corpus shrank");

    let universe = fixture_universe();
    for case in &negative.cases {
        match case.rejected_at.as_str() {
            "deserialize" => {
                let error = serde_json::from_value::<SemanticIrDocument>(case.document.clone())
                    .expect_err(&format!("{}: must not deserialize", case.case_id));
                let _ = error;
            }
            "check" => {
                let document: SemanticIrDocument = serde_json::from_value(case.document.clone())
                    .unwrap_or_else(|e| panic!("{}: {e}", case.case_id));
                let error = document
                    .check(&universe)
                    .expect_err(&format!("{}: must be rejected", case.case_id));
                assert_eq!(
                    error.kind(),
                    case.error_kind,
                    "{}: error kind mismatch ({error})",
                    case.case_id
                );
            }
            other => panic!("{}: unknown rejected_at {other}", case.case_id),
        }
    }
}

#[test]
fn ordering_groups_share_the_base_digest_and_mutations_diverge() {
    let equivalence: EquivalenceFile = serde_json::from_value(load_json(
        "spec/conformance/semantic-ir/equivalence/semantic-ir-equivalence.json",
    ))
    .expect("the equivalence file is valid");
    assert_eq!(equivalence.corpus_revision, CORPUS_REVISION);

    let universe = fixture_universe();
    let set = module_set();

    for group in &equivalence.ordering_groups {
        let base = group
            .base_document
            .check(&universe)
            .unwrap_or_else(|e| panic!("{}: base must validate: {e}", group.case_id));
        let member = group
            .member_document
            .check(&universe)
            .unwrap_or_else(|e| panic!("{}: member must validate: {e}", group.case_id));
        let base_digest = base.source().digest(&set).unwrap();
        let member_digest = member.source().digest(&set).unwrap();
        assert_eq!(
            base_digest.hex(),
            group.expected_digest,
            "{}: base digest drifted",
            group.case_id
        );
        assert_eq!(
            member_digest.hex(),
            group.expected_digest,
            "{}: ordering changed the digest",
            group.case_id
        );
    }

    for mutation in &equivalence.mutations {
        let base = mutation
            .base_document
            .check(&universe)
            .unwrap_or_else(|e| panic!("{}: base must validate: {e}", mutation.case_id));
        let mutated = mutation
            .mutated_document
            .check(&universe)
            .unwrap_or_else(|e| panic!("{}: mutated must validate: {e}", mutation.case_id));
        let base_digest = base.source().digest(&set).unwrap();
        let mutated_digest = mutated.source().digest(&set).unwrap();
        assert_eq!(
            base_digest.hex(),
            mutation.base_digest,
            "{}",
            mutation.case_id
        );
        assert_eq!(
            mutated_digest.hex(),
            mutation.mutated_digest,
            "{}",
            mutation.case_id
        );
        assert_ne!(
            base_digest, mutated_digest,
            "{}: the semantic order change did not move the digest",
            mutation.case_id
        );
    }
}

/// The frozen ABI table: `ComputationBody` case numbers 1..13 with exactly the
/// frozen names, as derived into the Stable ABI Number Registry.
#[test]
fn the_computation_body_case_table_is_frozen_abi() {
    let table = load_json("spec/registry/abi-numbers.json");
    let mut rows: Vec<(u32, &str)> = table["assignments"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| {
            row["schema_family"] == "agentflow.semantic.ir@1"
                && row["type_name"] == "ComputationBody"
                && row["number_kind"] == "variant_case"
        })
        .map(|row| {
            (
                u32::try_from(row["number"].as_u64().unwrap()).expect("case numbers fit u32"),
                row["symbolic_name"].as_str().unwrap(),
            )
        })
        .collect();
    rows.sort_unstable();
    assert_eq!(
        rows,
        vec![
            (1, "invoke"),
            (2, "await"),
            (3, "sequence"),
            (4, "select"),
            (5, "parallel"),
            (6, "race"),
            (7, "iterate"),
            (8, "retry"),
            (9, "timeout"),
            (10, "handle"),
            (11, "scope"),
            (12, "compensation_scope"),
            (13, "call_subflow"),
        ],
        "the frozen 13-opcode case table drifted"
    );
}

#[test]
fn digests_are_deterministic_across_runs() {
    let golden: GoldenFile = serde_json::from_value(load_json(
        "spec/conformance/semantic-ir/golden/semantic-ir-golden.json",
    ))
    .unwrap();
    let universe = fixture_universe();
    let set = module_set();
    for case in &golden.golden {
        let document = &case.document;
        let first = document.digest(&set).unwrap();
        let second = document.digest(&set).unwrap();
        assert_eq!(first, second, "{}", case.case_id);
        let _ = universe;
    }
}
