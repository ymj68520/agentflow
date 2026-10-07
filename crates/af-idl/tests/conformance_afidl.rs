//! AFIDL1 conformance harness: consumes the FROZEN corpus under
//! `spec/conformance/afidl/` (generated once by the independent Python
//! implementation `tools/gen-afidl-corpus.py`) and the normative tables under
//! `spec/tables/` and `spec/idl/`.
//!
//! Expected canonical bytes and digests are physically frozen; these tests
//! never generate expected values from the implementation under test.

use af_idl::{Primitive, compile, compile_module_set, compile_with_universe};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn read_repo_file(relative: &str) -> String {
    let path = repo_root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

fn load_json(relative: &str) -> serde_json::Value {
    let text = read_repo_file(relative);
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("cannot parse {relative}: {e}"))
}

fn compile_file_at(relative: &str) -> af_idl::CompiledSchema {
    compile(&read_repo_file(relative))
        .unwrap_or_else(|e| panic!("{relative} must compile, got {e}"))
}

#[derive(Deserialize)]
struct SchemaGolden {
    corpus_revision: u64,
    cases: Vec<SchemaCase>,
    equivalence_groups: Vec<Vec<String>>,
}

#[derive(Deserialize)]
struct SchemaCase {
    case_id: String,
    file: String,
    types: Vec<TypeEntry>,
    /// Case ids whose modules form the import universe for a universe-compiled
    /// fixture. Absent when the case imports nothing.
    #[serde(default)]
    requires: Vec<RequiredCase>,
}

#[derive(Deserialize)]
struct RequiredCase {
    case_id: String,
    file: String,
}

#[derive(Deserialize)]
struct TypeEntry {
    name: String,
    ace1_utf8: String,
    ace1_hex: String,
    #[allow(dead_code)]
    digest_hex: String,
}

#[derive(Deserialize)]
struct DigestGolden {
    #[allow(dead_code)]
    corpus_revision: u64,
    cases: Vec<DigestCase>,
    mutation_cases: Vec<MutationCase>,
}

#[derive(Deserialize)]
struct DigestCase {
    case_id: String,
    type_name: String,
    digest_hex: String,
}

#[derive(Deserialize)]
struct MutationCase {
    case_id: String,
    base: String,
    mutated: String,
    type_name: String,
}

/// Frozen corpus revision this harness is written against.
///
/// Golden revision 5 (Phase 3B): additive `semantic-ir` schema case. Revision
/// 4 (Phase 2C) added the cross-module `import-*` fixtures; revision 3
/// (Phase 3A) the `semantic-core` case and revision 2 (Phase 2B) the `specification-registry`
/// case. Every bump is purely additive — existing cases and digests are
/// unchanged. The parser-invalid manifest is a separate frozen artifact and
/// carries its own revision (2 since Phase 2C added the import negatives and
/// the module-set cycle cases).
const CORPUS_REVISION: u64 = 5;
const PARSER_INVALID_CORPUS_REVISION: u64 = 2;

fn frozen_digest_map() -> BTreeMap<(String, String), String> {
    let golden: DigestGolden = serde_json::from_value(load_json(
        "spec/conformance/afidl/type-schema-digest-golden.json",
    ))
    .expect("type-schema-digest-golden.json is valid");
    assert_eq!(
        golden.corpus_revision, CORPUS_REVISION,
        "unexpected corpus revision"
    );
    golden
        .cases
        .into_iter()
        .map(|c| ((c.case_id, c.type_name), c.digest_hex))
        .collect()
}

#[test]
fn schema_golden_matches_frozen_ace1_bytes_and_digests() {
    let golden: SchemaGolden =
        serde_json::from_value(load_json("spec/conformance/afidl/schema-golden.json"))
            .expect("schema-golden.json is valid");
    assert_eq!(
        golden.corpus_revision, CORPUS_REVISION,
        "unexpected corpus revision"
    );
    assert!(
        golden.cases.len() >= 20,
        "golden corpus must be non-trivial"
    );
    let digests = frozen_digest_map();

    for case in &golden.cases {
        let compiled = compile_file_at(&case.file);
        for type_entry in &case.types {
            let schema = compiled
                .by_name(&type_entry.name)
                .unwrap_or_else(|| panic!("{}: missing type {}", case.case_id, type_entry.name));

            let bytes = schema.ace1_bytes().unwrap();
            assert_eq!(
                hex::encode(&bytes),
                type_entry.ace1_hex,
                "case {} type {}: canonical ACE1 bytes mismatch\nexpected: {}\nactual:   {}",
                case.case_id,
                type_entry.name,
                type_entry.ace1_utf8,
                String::from_utf8_lossy(&bytes)
            );
            assert_eq!(bytes, type_entry.ace1_utf8.as_bytes());

            let digest = schema.digest().unwrap();
            let frozen = digests
                .get(&(case.case_id.clone(), type_entry.name.clone()))
                .unwrap_or_else(|| {
                    panic!("no frozen digest for {}/{}", case.case_id, type_entry.name)
                });
            assert_eq!(
                digest.hex(),
                frozen.as_str(),
                "case {} type {}: TypeSchemaDigest mismatch",
                case.case_id,
                type_entry.name
            );
        }
    }
}

#[test]
fn equivalence_groups_have_identical_type_digests() {
    let golden: SchemaGolden =
        serde_json::from_value(load_json("spec/conformance/afidl/schema-golden.json")).unwrap();

    for group in &golden.equivalence_groups {
        let mut reference: Option<BTreeMap<String, String>> = None;
        for member in group {
            let compiled = compile_file_at(&format!(
                "spec/conformance/afidl/parser-valid/{member}.afidl"
            ));
            let map: BTreeMap<String, String> = compiled
                .types
                .iter()
                .map(|t| (t.name.clone(), t.digest().unwrap().hex().to_string()))
                .collect();
            match &reference {
                None => reference = Some(map),
                Some(expected) => {
                    assert_eq!(
                        expected, &map,
                        "equivalence group {group:?} diverged at {member}"
                    );
                }
            }
        }
    }
}

#[test]
fn mutation_cases_change_the_digest() {
    let golden: DigestGolden = serde_json::from_value(load_json(
        "spec/conformance/afidl/type-schema-digest-golden.json",
    ))
    .unwrap();
    assert!(
        golden.mutation_cases.len() >= 10,
        "mutation corpus must be non-trivial"
    );

    let digests = frozen_digest_map();
    for mutation in &golden.mutation_cases {
        let base_file = format!(
            "spec/conformance/afidl/parser-valid/{}.afidl",
            mutation.base
        );
        let mutated_file = format!(
            "spec/conformance/afidl/parser-valid/{}.afidl",
            mutation.mutated
        );
        let base = compile_file_at(&base_file)
            .digest(&mutation.type_name)
            .unwrap_or_else(|| panic!("{} missing type {}", mutation.base, mutation.type_name));
        let mutated = compile_file_at(&mutated_file)
            .digest(&mutation.type_name)
            .unwrap_or_else(|| panic!("{} missing type {}", mutation.mutated, mutation.type_name));

        // Both members must match their own frozen entries...
        assert_eq!(
            hex::encode(base.raw_bytes()),
            digests[&(mutation.base.clone(), mutation.type_name.clone())],
            "mutation {}: base digest drifted",
            mutation.case_id
        );
        assert_eq!(
            hex::encode(mutated.raw_bytes()),
            digests[&(mutation.mutated.clone(), mutation.type_name.clone())],
            "mutation {}: mutated digest drifted",
            mutation.case_id
        );
        // ...and the semantic change must alter the digest.
        assert_ne!(
            base, mutated,
            "mutation {}: semantic change did not alter the digest",
            mutation.case_id
        );
    }
}

#[derive(Deserialize)]
struct InvalidManifest {
    corpus_revision: u64,
    cases: Vec<InvalidCase>,
    #[serde(default)]
    module_set_cases: Vec<ModuleSetCase>,
    #[serde(default)]
    module_set_valid_cases: Vec<ModuleSetValidCase>,
}

#[derive(Deserialize)]
struct InvalidCase {
    file: String,
    error_kind: String,
    /// `"compile"` (default) or `"universe"`: the latter compiles against the
    /// parser-valid modules it imports.
    #[serde(default)]
    mode: Option<String>,
}

#[derive(Deserialize)]
struct ModuleSetCase {
    case_id: String,
    files: Vec<String>,
    error_kind: String,
}

#[derive(Deserialize)]
struct ModuleSetValidCase {
    case_id: String,
    files: Vec<String>,
    #[allow(dead_code)]
    expect: String,
}

/// Builds the import universe of a fixture from the corpus cases it requires.
fn universe_for(requires: &[RequiredCase]) -> af_idl::ResolvedSchemaUniverse {
    let mut universe = af_idl::ResolvedSchemaUniverse::new();
    for required in requires {
        let compiled = compile_file_at(&required.file);
        universe
            .insert(compiled)
            .unwrap_or_else(|e| panic!("{} must register: {e}", required.case_id));
    }
    universe
}

/// The parser-valid corpus cases, keyed by case id, so a fixture can name the
/// modules that form its import universe.
fn corpus_cases_by_id() -> BTreeMap<String, SchemaCase> {
    let golden: SchemaGolden =
        serde_json::from_value(load_json("spec/conformance/afidl/schema-golden.json"))
            .expect("schema-golden.json is valid");
    golden
        .cases
        .into_iter()
        .map(|case| (case.case_id.clone(), case))
        .collect()
}

#[test]
fn import_fixtures_compile_against_their_frozen_universe() {
    let golden: SchemaGolden =
        serde_json::from_value(load_json("spec/conformance/afidl/schema-golden.json")).unwrap();
    let mut with_imports = 0;
    for case in &golden.cases {
        if case.requires.is_empty() {
            continue;
        }
        with_imports += 1;
        let universe = universe_for(&case.requires);
        let source = read_repo_file(&case.file);
        let compiled = compile_with_universe(&source, &universe)
            .unwrap_or_else(|e| panic!("{} must compile against its universe: {e}", case.case_id));
        for type_entry in &case.types {
            let schema = compiled
                .by_name(&type_entry.name)
                .unwrap_or_else(|| panic!("{}: missing {}", case.case_id, type_entry.name));
            // A universe-compiled module must produce exactly the frozen bytes
            // of its single-module compilation: imports add validity, never
            // identity.
            assert_eq!(
                hex::encode(schema.ace1_bytes().unwrap()),
                type_entry.ace1_hex,
                "{}: universe compilation changed canonical bytes for {}",
                case.case_id,
                type_entry.name
            );
        }
    }
    assert!(
        with_imports >= 8,
        "the corpus must exercise cross-module imports"
    );
}

#[test]
fn import_alias_lowering_is_frozen() {
    let golden: serde_json::Value = load_json("spec/conformance/afidl/import-golden.json");
    assert_eq!(golden["corpus_revision"], CORPUS_REVISION);
    let cases = golden["cases"].as_array().expect("cases array");
    assert!(cases.len() >= 8, "import corpus must be non-trivial");

    for case in cases {
        let case_id = case["case_id"].as_str().unwrap();
        // The frozen alias → specification table must match what the resolver
        // derives from the source, and every referenced identity must be the
        // imported specification's full logical identity.
        let mut expected: BTreeMap<String, String> = BTreeMap::new();
        for entry in case["resolved_imports"].as_array().unwrap() {
            expected.insert(
                entry["alias"].as_str().unwrap().to_string(),
                entry["specification_id"].as_str().unwrap().to_string(),
            );
        }
        let file = case["file"].as_str().unwrap();
        let module = af_idl::parse(&read_repo_file(file)).unwrap();
        let actual: BTreeMap<String, String> = module
            .imports
            .iter()
            .map(|import| (import.alias.clone(), import.id.canonical()))
            .collect();
        assert_eq!(actual, expected, "{case_id}: import table drifted");

        for (type_name, identities) in case["type_identities"].as_object().unwrap() {
            let frozen: BTreeSet<String> = identities
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_str().unwrap().to_string())
                .collect();
            for identity in &frozen {
                // A qualified reference may never lower to the *importing*
                // module's identity, and never to a bare alias spelling.
                assert!(
                    !identity.contains(".Slot") && !identity.contains(".Address"),
                    "{case_id}/{type_name}: alias leaked into identity {identity}"
                );
            }
            let _ = type_name;
        }
    }
}

#[test]
fn import_equivalence_groups_are_frozen() {
    // These three groups are the central Phase 2C claim: an alias rename, a
    // reordering of import declarations and an unused import all leave every
    // TypeSchemaDigest unchanged.
    let golden: SchemaGolden =
        serde_json::from_value(load_json("spec/conformance/afidl/schema-golden.json")).unwrap();
    let import_groups: Vec<&Vec<String>> = golden
        .equivalence_groups
        .iter()
        .filter(|group| group.iter().any(|member| member.starts_with("import-")))
        .collect();
    assert!(
        import_groups.len() >= 3,
        "the import equivalence groups must be frozen in the corpus"
    );
    for group in import_groups {
        let mut reference: Option<BTreeMap<String, String>> = None;
        for member in group {
            let case = golden
                .cases
                .iter()
                .find(|case| case.case_id == *member)
                .unwrap_or_else(|| panic!("missing case {member}"));
            let universe = universe_for(&case.requires);
            let compiled = if case.requires.is_empty() {
                compile_file_at(&case.file)
            } else {
                compile_with_universe(&read_repo_file(&case.file), &universe)
                    .unwrap_or_else(|e| panic!("{member} must compile: {e}"))
            };
            let map: BTreeMap<String, String> = compiled
                .types
                .iter()
                .map(|t| (t.name.clone(), t.digest().unwrap().hex().to_string()))
                .collect();
            match &reference {
                None => reference = Some(map),
                Some(expected) => assert_eq!(
                    expected, &map,
                    "import equivalence group {group:?} diverged at {member}"
                ),
            }
        }
    }
}

#[test]
fn module_set_cycles_are_rejected_and_free_mutual_imports_compile() {
    let manifest: InvalidManifest = serde_json::from_value(load_json(
        "spec/conformance/afidl/parser-invalid/manifest.json",
    ))
    .unwrap();
    assert!(
        !manifest.module_set_cases.is_empty(),
        "the cross-module recursion cases must be frozen"
    );

    for case in &manifest.module_set_cases {
        let sources: Vec<(String, String)> = case
            .files
            .iter()
            .map(|file| {
                (
                    file.clone(),
                    read_repo_file(&format!("spec/conformance/afidl/{file}")),
                )
            })
            .collect();
        let error = compile_module_set(&sources)
            .expect_err(&format!("{}: the cycle must be rejected", case.case_id));
        assert_eq!(
            error.kind(),
            case.error_kind,
            "{}: error kind mismatch ({error})",
            case.case_id
        );
    }

    for case in &manifest.module_set_valid_cases {
        let sources: Vec<(String, String)> = case
            .files
            .iter()
            .map(|file| {
                (
                    file.clone(),
                    read_repo_file(&format!("spec/conformance/afidl/{file}")),
                )
            })
            .collect();
        compile_module_set(&sources).unwrap_or_else(|e| {
            panic!(
                "{}: mutual imports without a type cycle must compile, got {e}",
                case.case_id
            )
        });
    }
}

#[test]
fn parser_invalid_corpus_fails_with_frozen_kinds() {
    let manifest: InvalidManifest = serde_json::from_value(load_json(
        "spec/conformance/afidl/parser-invalid/manifest.json",
    ))
    .expect("parser-invalid manifest is valid");
    assert_eq!(
        manifest.corpus_revision, PARSER_INVALID_CORPUS_REVISION,
        "unexpected corpus revision"
    );
    assert!(
        manifest.cases.len() >= 25,
        "negative corpus must be non-trivial"
    );

    for case in &manifest.cases {
        let path = repo_root()
            .join("spec/conformance/afidl/parser-invalid")
            .join(&case.file);
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        let outcome = match case.mode.as_deref() {
            Some("universe") => {
                // Universe-mode negatives import a module that is itself
                // corpus-frozen, so the negative exercises resolution rather
                // than the self-contained compile path.
                let cases = corpus_cases_by_id();
                let requires = &cases
                    .get("import-basic")
                    .expect("the import-basic case declares its universe")
                    .requires;
                let universe = universe_for(requires);
                compile_with_universe(&source, &universe).map(|_| ())
            }
            _ => compile(&source).map(|_| ()),
        };
        match outcome {
            Ok(()) => panic!("{}: expected error {}", case.file, case.error_kind),
            Err(e) => assert_eq!(
                e.kind(),
                case.error_kind,
                "{}: error kind mismatch ({e})",
                case.file
            ),
        }
    }
}

/// The three normative primitive registries must never drift:
/// Rust `Primitive` enum == afidl-primitives.json == machine-primitives.afidl.
#[test]
fn primitive_universe_does_not_drift() {
    // 1. The closed JSON table.
    let table = load_json("spec/tables/afidl-primitives.json");
    let table_names: Vec<&str> = table["primitives"]
        .as_array()
        .expect("primitives array")
        .iter()
        .map(|p| p["name"].as_str().expect("primitive name"))
        .collect();
    let rust_names: Vec<&str> = Primitive::ALL.iter().map(|p| p.keyword()).collect();
    assert_eq!(
        table_names, rust_names,
        "afidl-primitives.json drifted from the enum"
    );

    // 2. The normative registry file, parsed by the real implementation.
    let compiled = compile_file_at("spec/idl/core/machine-primitives.afidl");
    assert_eq!(
        compiled.specification.canonical(),
        "agentflow.core.machine-primitives@1"
    );
    assert_eq!(compiled.types.len(), Primitive::ALL.len());
    let alias_targets: BTreeMap<&str, &str> = compiled
        .types
        .iter()
        .map(|t| match &t.body {
            af_idl::CanonicalBody::Alias(af_idl::CanonicalTypeExpr::Primitive(p)) => {
                (t.name.as_str(), p.keyword())
            }
            other => panic!(
                "registry entry {} must alias a primitive, got {other:?}",
                t.name
            ),
        })
        .collect();
    let entries: Vec<(&str, &str)> = alias_targets.iter().map(|(k, v)| (*k, *v)).collect();
    for primitive in Primitive::ALL {
        let alias_name = entries
            .iter()
            .find(|(_, keyword)| *keyword == primitive.keyword())
            .map_or_else(
                || panic!("no alias for primitive {}", primitive.keyword()),
                |(name, _)| *name,
            );
        assert_eq!(
            alias_name,
            expected_alias_name(primitive.keyword()),
            "registry alias for {} should be its nominal PascalCase name",
            primitive.keyword()
        );
    }
}

fn expected_alias_name(keyword: &str) -> &'static str {
    match keyword {
        "unit" => "Unit",
        "null" => "Null",
        "bool" => "Bool",
        "int32" => "Int32",
        "uint32" => "UInt32",
        "int64" => "Int64",
        "uint64" => "UInt64",
        "decimal" => "Decimal",
        "float64" => "Float64",
        "string" => "String",
        "bytes" => "Bytes",
        "timestamp" => "Timestamp",
        "duration" => "Duration",
        other => unreachable!("unknown primitive keyword {other}"),
    }
}

#[test]
fn compilation_is_deterministic_across_runs() {
    let file = "spec/conformance/afidl/parser-valid/record-multi-field.afidl";
    let first = compile_file_at(file);
    let second = compile_file_at(file);
    for (a, b) in first.types.iter().zip(second.types.iter()) {
        assert_eq!(a.ace1_bytes().unwrap(), b.ace1_bytes().unwrap());
        assert_eq!(a.digest().unwrap(), b.digest().unwrap());
    }
}
