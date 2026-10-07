//! Semantic core conformance harness.
//!
//! Consumes the FROZEN corpus under `spec/conformance/semantic-core/`
//! (generated once by `tools/gen-semantic-core-corpus.py`, an independent
//! Python implementation) and the normative AFIDL module
//! `spec/idl/semantic-core.afidl`.
//!
//! Expected bytes, derived types and error kinds are physically frozen; these
//! tests never generate expected values from the implementation under test.

use af_semantic::{
    ExpressionId, SemanticCoreDocument, SemanticError, StructuralType, TypeSchemaUniverse,
};
use serde::Deserialize;
use std::collections::BTreeMap;
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

fn load<T: serde::de::DeserializeOwned>(relative: &str) -> T {
    let text = read(relative);
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("cannot parse {relative}: {e}"))
}

#[derive(Deserialize)]
struct GoldenFile {
    corpus_revision: u64,
    fixture_schema: FixtureSchema,
    golden: Vec<GoldenCase>,
    negative: Vec<NegativeCase>,
    equivalence_groups: Vec<Vec<String>>,
}

#[derive(Deserialize)]
struct FixtureSchema {
    file: String,
    specification_id: String,
    types: Vec<FixtureType>,
}

#[derive(Deserialize)]
struct FixtureType {
    name: String,
    digest_hex: String,
}

#[derive(Deserialize)]
struct GoldenCase {
    case_id: String,
    expression_id: u32,
    derived_type: String,
    ace1_utf8: String,
    ace1_hex: String,
    #[serde(default)]
    value_bindings: Vec<ValueBinding>,
    document: SemanticCoreDocument,
}

#[derive(Deserialize)]
struct ValueBinding {
    slot: u32,
    constant: af_semantic::PrimitiveConstant,
}

#[derive(Deserialize)]
struct NegativeCase {
    case_id: String,
    error_kind: String,
    /// `"check"` (the document parses and the validator rejects it) or
    /// `"deserialize"` (the document is unrepresentable in the typed model, so
    /// rejection happens at parse time — e.g. a machine-only constant case).
    #[serde(default = "default_rejected_at")]
    rejected_at: String,
    document: serde_json::Value,
}

fn default_rejected_at() -> String {
    "check".to_string()
}

const GOLDEN_PATH: &str = "spec/conformance/semantic-core/semantic-core-golden.json";

/// Word-boundary match: `int32` must not be found inside `uint32`.
fn contains_word(haystack: &str, needle: &str) -> bool {
    let bytes = haystack.as_bytes();
    haystack.match_indices(needle).any(|(index, _)| {
        let before_ok =
            index == 0 || (!bytes[index - 1].is_ascii_alphanumeric() && bytes[index - 1] != b'_');
        let after = index + needle.len();
        let after_ok =
            after >= bytes.len() || (!bytes[after].is_ascii_alphanumeric() && bytes[after] != b'_');
        before_ok && after_ok
    })
}

fn fixture_universe() -> TypeSchemaUniverse {
    let file: GoldenFile = load(GOLDEN_PATH);
    let compiled = af_idl::compile(&read(&file.fixture_schema.file))
        .expect("fixture schema universe must compile");
    let mut universe = TypeSchemaUniverse::new();
    universe
        .insert_compiled(&compiled)
        .expect("insert universe");
    universe
}

#[test]
fn corpus_revision_is_frozen() {
    let file: GoldenFile = load(GOLDEN_PATH);
    assert_eq!(
        file.corpus_revision, 1,
        "semantic-core corpus revision drifted"
    );
    assert!(file.golden.len() >= 30, "golden corpus shrank unexpectedly");
    assert!(
        file.negative.len() >= 50,
        "negative corpus shrank unexpectedly"
    );
}

#[test]
fn fixture_universe_digests_match_the_frozen_values() {
    let file: GoldenFile = load(GOLDEN_PATH);
    let universe = fixture_universe();
    for fixture in &file.fixture_schema.types {
        let identity = format!("{}/{}", file.fixture_schema.specification_id, fixture.name);
        let entry = universe
            .get(&identity)
            .unwrap_or_else(|| panic!("fixture type {identity} missing from the universe"));
        assert_eq!(
            entry.digest.hex(),
            fixture.digest_hex,
            "fixture digest drifted for {identity}"
        );
    }
}

#[test]
fn golden_cases_reproduce_frozen_bytes_and_derived_types() {
    let file: GoldenFile = load(GOLDEN_PATH);
    let universe = fixture_universe();
    for case in &file.golden {
        let checked = case
            .document
            .check(&universe)
            .unwrap_or_else(|e| panic!("{}: document must check, got {e}", case.case_id));

        // Derived type of the lowered expression.
        let id = ExpressionId::from_raw(case.expression_id);
        let derived: StructuralType = checked
            .infer_expression_type(id)
            .unwrap_or_else(|e| panic!("{}: inference failed: {e}", case.case_id));
        assert_eq!(
            derived.canonical_string(),
            case.derived_type,
            "{}: derived type drifted",
            case.case_id
        );

        // Frozen canonical bytes. Value-slot bindings are supplied by the
        // corpus (Phase 3A has no producers) and are validated by ACE1 itself
        // inside `lower_with`.
        let mut bindings: BTreeMap<af_semantic::ValueSlotId, af_core::Value> = BTreeMap::new();
        for binding in &case.value_bindings {
            let (_, value) = lower_constant(&binding.constant);
            bindings.insert(af_semantic::ValueSlotId::from_raw(binding.slot), value);
        }
        let (shape, value) = checked
            .lower_with(id, &bindings)
            .unwrap_or_else(|e| panic!("{}: lowering failed: {e}", case.case_id));
        let bytes = af_core::encode(&shape, &value)
            .unwrap_or_else(|e| panic!("{}: encoding failed: {e}", case.case_id));
        assert_eq!(
            std::str::from_utf8(&bytes).unwrap(),
            case.ace1_utf8,
            "{}: canonical bytes drifted",
            case.case_id
        );
        assert_eq!(
            hex::encode(&bytes),
            case.ace1_hex,
            "{}: canonical hex drifted",
            case.case_id
        );
    }
}

/// Lowers a primitive constant to `(Shape, Value)` for slot bindings.
fn lower_constant(constant: &af_semantic::PrimitiveConstant) -> (af_core::Shape, af_core::Value) {
    use af_semantic::PrimitiveConstant as C;
    match constant {
        C::Unit => (af_core::Shape::Unit, af_core::Value::Unit),
        C::Null => (af_core::Shape::Null, af_core::Value::Null),
        C::Bool(flag) => (af_core::Shape::Bool, af_core::Value::Bool(*flag)),
        C::Int64(number) => (af_core::Shape::Int64, af_core::Value::Int64(*number)),
        C::Decimal(decimal) => (
            af_core::Shape::Decimal,
            af_core::Value::Decimal(decimal.clone()),
        ),
        C::String(text) => (af_core::Shape::String, af_core::Value::String(text.clone())),
        C::Bytes(text) => (
            af_core::Shape::Bytes,
            af_core::Value::Bytes(af_core::base64url::decode(text).expect("canonical base64url")),
        ),
        C::Timestamp(micros) => (
            af_core::Shape::Timestamp,
            af_core::Value::Timestamp(*micros),
        ),
        C::Duration(micros) => (af_core::Shape::Duration, af_core::Value::Duration(*micros)),
    }
}

#[test]
fn negative_cases_fail_with_their_frozen_error_kinds() {
    let file: GoldenFile = load(GOLDEN_PATH);
    let universe = fixture_universe();
    for case in &file.negative {
        match case.rejected_at.as_str() {
            "deserialize" => {
                // The closed typed model cannot even represent this document;
                // that unrepresentability *is* the rejection.
                let parsed: Result<SemanticCoreDocument, _> =
                    serde_json::from_value(case.document.clone());
                assert!(
                    parsed.is_err(),
                    "{}: document must be unrepresentable in the typed model",
                    case.case_id
                );
            }
            "check" => {
                let document: SemanticCoreDocument = serde_json::from_value(case.document.clone())
                    .unwrap_or_else(|e| {
                        panic!(
                            "{}: document must parse (it only fails validation): {e}",
                            case.case_id
                        )
                    });
                match document.check(&universe) {
                    Ok(_) => panic!(
                        "{}: document must be rejected with {} but validated",
                        case.case_id, case.error_kind
                    ),
                    Err(error) => assert_eq!(
                        error.kind(),
                        case.error_kind,
                        "{}: wrong error kind ({error})",
                        case.case_id
                    ),
                }
            }
            other => panic!("{}: unknown rejected_at {other:?}", case.case_id),
        }
    }
}

#[test]
fn equivalence_groups_produce_identical_bytes() {
    let file: GoldenFile = load(GOLDEN_PATH);
    let by_id: BTreeMap<&str, &GoldenCase> = file
        .golden
        .iter()
        .map(|case| (case.case_id.as_str(), case))
        .collect();
    for group in &file.equivalence_groups {
        let mut reference: Option<(&str, &str)> = None;
        for case_id in group {
            let case = by_id
                .get(case_id.as_str())
                .unwrap_or_else(|| panic!("equivalence group names unknown case {case_id}"));
            match reference {
                None => reference = Some((case.case_id.as_str(), case.ace1_hex.as_str())),
                Some((first, expected)) => assert_eq!(
                    case.ace1_hex, expected,
                    "{case_id} must be byte-identical to {first}"
                ),
            }
        }
    }
}

#[test]
fn the_normative_module_declares_exactly_the_expected_type_set() {
    let compiled = af_idl::compile(&read("spec/idl/semantic-core.afidl"))
        .expect("semantic-core.afidl must compile");
    assert_eq!(
        compiled.specification.canonical(),
        "agentflow.semantic.core@1"
    );
    let names: Vec<&str> = compiled.types.iter().map(|t| t.name.as_str()).collect();
    for expected in [
        "AndBody",
        "DigestRefValue",
        "EnumValueBody",
        "EqualsBody",
        "ExpressionBody",
        "ExpressionDef",
        "ExpressionId",
        "FieldGetBody",
        "IfThenElseBody",
        "ListConstructBody",
        "MapConstructBody",
        "MapEntryExpr",
        "NotBody",
        "OptionNoneBody",
        "OptionSomeBody",
        "OrBody",
        "PrimitiveConstant",
        "RecordConstructBody",
        "RecordFieldExpr",
        "ResourceSlot",
        "ResourceSlotId",
        "SemanticPrimitiveType",
        "SemanticTypeId",
        "SemanticTypeNode",
        "SemanticTypeNodeBody",
        "TypeSchemaRef",
        "ValueSlot",
        "ValueSlotId",
        "VariantConstructBody",
    ] {
        assert!(
            names.contains(&expected),
            "semantic-core.afidl is missing {expected}"
        );
    }
    assert_eq!(names.len(), 29, "semantic-core.afidl type count drifted");
}

#[test]
fn the_normative_module_says_nothing_about_machine_only_primitives() {
    // Static self-check over *code*: comments may (and do) discuss the
    // machine-only primitives, declarations may not reference them.
    let source = read("spec/idl/semantic-core.afidl");
    let code: String = source
        .lines()
        .map(|line| match line.find("//") {
            Some(index) => &line[..index],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n");

    for machine_only in ["int32", "uint64", "float64"] {
        assert!(
            !contains_word(&code, machine_only),
            "semantic-core.afidl must not declare the machine-only primitive {machine_only}"
        );
    }
    // uint32 is only allowed for local ids and numbering fields.
    for (index, line) in code.lines().enumerate() {
        if !line.contains("uint32") {
            continue;
        }
        let trimmed = line.trim();
        assert!(
            trimmed.starts_with("alias ") || trimmed.contains(": uint32;"),
            "uint32 may only be used for local ids and numbering, found on line {}: {trimmed}",
            index + 1
        );
    }
}

#[test]
fn workflow_primitive_universe_is_the_frozen_nine() {
    use af_semantic::SemanticPrimitiveType as P;
    let keywords: Vec<&str> = P::ALL.iter().map(|p| p.keyword()).collect();
    assert_eq!(
        keywords,
        [
            "unit",
            "null",
            "bool",
            "int64",
            "decimal",
            "string",
            "bytes",
            "timestamp",
            "duration"
        ]
    );
    for machine_only in P::MACHINE_ONLY {
        let error: SemanticError = P::try_from_keyword(machine_only).unwrap_err();
        assert_eq!(error.kind(), "unsupported_workflow_primitive");
    }
    // `uint32` is not even a workflow keyword, and the three remaining machine
    // primitives are rejected by the same closed gate.
    assert!(P::try_from_keyword("uint32").is_err());
    assert!(P::try_from_keyword("int32").is_err());
    assert!(P::try_from_keyword("float64").is_err());
}
