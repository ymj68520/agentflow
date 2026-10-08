//! Effect-signature conformance harness (Phase 4A.1): consumes the FROZEN
//! corpus under `spec/conformance/effect-signature/` (generated once by the
//! independent Python implementation `tools/gen-effect-signature-corpus.py`).
//!
//! Expected canonical ACE1 bytes, classifications, error kinds, algebra
//! results and equivalence groups all come from the corpus; tests never
//! generate expected values from the implementation under test.

use af_effect::{
    AtomKey, CapabilityCatalog, EffectAtom, EffectCatalogUniverse, EffectError, EffectSignature,
    Proof, accumulate, branch_join, compensation_envelope, contains, iterate_envelope,
    parallel_envelope, race_envelope, repeat, retry_envelope, select_envelope, sequence_envelope,
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

fn case_files(subdir: &str) -> Vec<serde_json::Value> {
    let directory = repo_root()
        .join("spec/conformance/effect-signature")
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
            "effect-signature.afidl".to_string(),
            read("spec/idl/effect-signature.afidl"),
        ),
    ];
    af_idl::compile_module_set(&sources).expect("the normative module set compiles")
}

/// The explicit catalog universe of one case, from its frozen fixture block.
fn fixture_universe(case: &serde_json::Value) -> EffectCatalogUniverse {
    let mut universe = EffectCatalogUniverse::new();
    for catalog in case["fixture_catalogs"].as_array().unwrap() {
        let operations: Vec<String> = catalog["operations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|op| op.as_str().unwrap().to_string())
            .collect();
        let catalog = CapabilityCatalog::new(
            catalog["specification_id"].as_str().unwrap().to_string(),
            operations,
        )
        .unwrap_or_else(|e| panic!("fixture catalog is well formed: {e}"));
        universe
            .insert(&catalog)
            .unwrap_or_else(|e| panic!("fixture catalog registers: {e}"));
    }
    universe
}

fn signature_of(case: &serde_json::Value, key: &str) -> EffectSignature {
    serde_json::from_value(case[key].clone())
        .unwrap_or_else(|e| panic!("{}: signature parses: {e}", case["case_id"]))
}

fn validated(case: &serde_json::Value, key: &str) -> EffectSignature {
    let universe = fixture_universe(case);
    signature_of(case, key)
        .validate(&universe)
        .unwrap_or_else(|e| panic!("{}: validates: {e}", case["case_id"]))
}

fn assert_canonical_bytes(case_id: &str, signature: &EffectSignature, frozen: &str) {
    let bytes = signature
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
    assert!(files.len() >= 10, "the golden corpus shrank");
    for case in &files {
        let case_id = case["case_id"].as_str().unwrap();
        let signature = validated(case, "signature");
        assert_canonical_bytes(
            case_id,
            &signature,
            case["canonical_ace1_utf8"].as_str().unwrap(),
        );
        assert_eq!(
            signature.is_pure(),
            case["is_pure"].as_bool().unwrap(),
            "{case_id}: is_pure drifted"
        );
        assert_eq!(
            signature.classification(),
            case["classification"].as_str().unwrap(),
            "{case_id}: classification drifted"
        );
        // Canonicalization is idempotent.
        let twice = signature.canonical().unwrap().canonical().unwrap();
        assert_eq!(
            twice, signature,
            "{case_id}: canonicalization is not idempotent"
        );
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
        let universe = fixture_universe(case);
        let signature = signature_of(case, "signature");
        if case["expect_validation"] == "accept" {
            // A declared state, not a rejection: the top-row observations.
            let canonical = signature
                .validate(&universe)
                .unwrap_or_else(|e| panic!("{case_id}: must validate: {e}"));
            if let Some(expected) = case["classification"].as_str() {
                assert_eq!(canonical.classification(), expected);
            }
            if let Some(expected) = case["is_pure"].as_bool() {
                assert_eq!(canonical.is_pure(), expected);
            }
            if let Some(frozen) = case["canonical_ace1_utf8"].as_str() {
                assert_canonical_bytes(case_id, &canonical, frozen);
            }
            if let Some(limit) = case.get("containment_limit") {
                let limit: EffectSignature = serde_json::from_value(limit.clone()).unwrap();
                let error = canonical
                    .require_contained_in(&limit)
                    .expect_err("{case_id}: containment of the unknown row must fail");
                assert_eq!(
                    error.kind(),
                    case["containment_error_kind"].as_str().unwrap(),
                    "{case_id}: containment error kind drifted"
                );
            }
        } else {
            let error = signature
                .validate(&universe)
                .expect_err(&format!("{case_id}: must be rejected"));
            assert_eq!(
                error.kind(),
                case["expected_kind"].as_str().unwrap(),
                "{case_id}: rejection kind drifted"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Algebra
// ---------------------------------------------------------------------------

fn apply_operation(case: &serde_json::Value, universe: &EffectCatalogUniverse) -> EffectSignature {
    let operation = case["operation"].as_str().unwrap();
    let operands: Vec<EffectSignature> = case["operands"]
        .as_array()
        .unwrap()
        .iter()
        .map(|operand| {
            let signature: EffectSignature = serde_json::from_value(operand.clone())
                .unwrap_or_else(|e| panic!("operand parses: {e}"));
            signature
                .validate(universe)
                .unwrap_or_else(|e| panic!("operand validates: {e}"))
        })
        .collect();
    match operation {
        "accumulate" => accumulate(&operands[0], &operands[1]),
        "branch_join" => branch_join(&operands[0], &operands[1]),
        "repeat" => repeat(
            &serde_json::from_value(case["reps"].clone()).unwrap(),
            &operands[0],
        ),
        "iterate_envelope" => iterate_envelope(
            &serde_json::from_value(case["reps"].clone()).unwrap(),
            &operands[0],
        ),
        "sequence_envelope" => sequence_envelope(&operands[0], &operands[1]),
        "parallel_envelope" => parallel_envelope(&operands[0], &operands[1]),
        "select_envelope" => select_envelope(&operands[0], &operands[1]),
        "race_envelope" => race_envelope(&operands),
        "compensation_envelope" => compensation_envelope(&operands[0], &operands[1]),
        other => panic!("{other}: unexpected algebra operation"),
    }
    .unwrap_or_else(|e| panic!("{}: algebra operation failed: {e}", case["case_id"]))
}

#[test]
fn algebra_cases_reproduce_the_frozen_results() {
    let files = case_files("algebra");
    assert!(files.len() >= 14, "the algebra corpus shrank");
    for case in &files {
        let case_id = case["case_id"].as_str().unwrap();
        let universe = fixture_universe(case);
        let expected = &case["expected"];

        if let Some(error_kind) = expected["error_kind"].as_str() {
            let operation = case["operation"].as_str().unwrap();
            let operands: Vec<EffectSignature> = case["operands"]
                .as_array()
                .unwrap()
                .iter()
                .map(|operand| {
                    let signature: EffectSignature =
                        serde_json::from_value(operand.clone()).unwrap();
                    signature.validate(&universe).unwrap()
                })
                .collect();
            match operation {
                "repeat" => {
                    let reps: af_effect::OccurrenceBounds =
                        serde_json::from_value(case["reps"].clone()).unwrap();
                    let error =
                        repeat(&reps, &operands[0]).expect_err(&format!("{case_id}: must fail"));
                    assert_eq!(error.kind(), error_kind, "{case_id}: error kind drifted");
                }
                "require_contained_in" => {
                    for direction in expected["directions"].as_array().unwrap() {
                        let i = usize::try_from(direction[0].as_u64().unwrap()).unwrap();
                        let j = usize::try_from(direction[1].as_u64().unwrap()).unwrap();
                        let error = operands[i]
                            .require_contained_in(&operands[j])
                            .expect_err(&format!("{case_id}: {i}->{j} must fail closed"));
                        assert_eq!(error.kind(), error_kind, "{case_id}: error kind drifted");
                    }
                }
                other => panic!("{other}: unexpected failing operation"),
            }
            continue;
        }

        if case["operation"] == "contains" {
            let operands: Vec<EffectSignature> = case["operands"]
                .as_array()
                .unwrap()
                .iter()
                .map(|operand| {
                    let signature: EffectSignature =
                        serde_json::from_value(operand.clone()).unwrap();
                    signature.validate(&universe).unwrap()
                })
                .collect();
            if let Some(pairs) = expected["proven_pairs"].as_array() {
                for pair in pairs {
                    let i = usize::try_from(pair[0].as_u64().unwrap()).unwrap();
                    let j = usize::try_from(pair[1].as_u64().unwrap()).unwrap();
                    assert!(
                        contains(&operands[i], &operands[j]).is_proven(),
                        "{case_id}: pair {i}->{j} must be proven"
                    );
                }
            }
            if let Some(pairs) = expected["unproven_pairs"].as_array() {
                for pair in pairs {
                    let i = usize::try_from(pair[0].as_u64().unwrap()).unwrap();
                    let j = usize::try_from(pair[1].as_u64().unwrap()).unwrap();
                    match contains(&operands[i], &operands[j]) {
                        Proof::Unproven(reason) => assert_eq!(
                            reason.as_str(),
                            expected["unproven_reason"].as_str().unwrap(),
                            "{case_id}: unproven reason drifted for {i}->{j}"
                        ),
                        Proof::Proven => {
                            panic!("{case_id}: mismatched identity unexpectedly proved {i}->{j}")
                        }
                    }
                }
            }
            if expected["proven_pairs"].is_null() && expected["unproven_pairs"].is_null() {
                let proof = contains(&operands[0], &operands[0]);
                assert_eq!(
                    proof.is_proven(),
                    expected["proven"].as_bool().unwrap(),
                    "{case_id}: proven drifted"
                );
                if let Some(Proof::Unproven(reason)) = Some(proof) {
                    assert_eq!(
                        reason.as_str(),
                        expected["reason"].as_str().unwrap_or("unknown_row"),
                        "{case_id}: reason drifted"
                    );
                }
            }
            continue;
        }

        if case["operation"] == "retry_envelope" {
            let operand: EffectSignature =
                serde_json::from_value(case["operands"][0].clone()).unwrap();
            let child = operand.validate(&universe).unwrap();
            let envelope = retry_envelope(&child, case["max_attempts"].as_u64().unwrap())
                .unwrap_or_else(|e| panic!("{case_id}: retry envelope: {e}"));
            let attempts: af_effect::OccurrenceBounds =
                serde_json::from_value(expected["attempts"].clone()).unwrap();
            assert_eq!(envelope.attempts, attempts, "{case_id}: attempts drifted");
            assert!(
                !envelope.logical_narrowing_proven,
                "{case_id}: narrowing must not be proven this phase"
            );
            assert_eq!(
                envelope.obligations,
                expected["obligations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|o| o.as_str().unwrap().to_string())
                    .collect::<Vec<String>>(),
                "{case_id}: obligations drifted"
            );
            assert_canonical_bytes(
                case_id,
                &envelope.logical,
                expected["logical_ace1_utf8"].as_str().unwrap(),
            );
            continue;
        }

        // The associativity frozen result belongs to the full grouping, not
        // to a two-operand prefix application.
        if case["property"].as_str() != Some("associativity") {
            let result = apply_operation(case, &universe);
            assert_canonical_bytes(
                case_id,
                &result,
                expected["canonical_ace1_utf8"].as_str().unwrap(),
            );
        }

        // Frozen algebraic properties, re-checked across groupings. The
        // combiner follows the case's operation so guarded-join properties
        // are checked against branch_join exactly as accumulate properties
        // are checked against accumulate.
        let combine = |a: &EffectSignature, b: &EffectSignature| -> EffectSignature {
            match case["operation"].as_str().unwrap() {
                "accumulate" => accumulate(a, b),
                "branch_join" | "select_envelope" => branch_join(a, b),
                other => panic!("{other}: no algebraic property defined"),
            }
            .unwrap_or_else(|e| panic!("{case_id}: property combine failed: {e}"))
        };
        let parse = |index: usize| -> EffectSignature {
            let raw: EffectSignature = serde_json::from_value(case["operands"][index].clone())
                .unwrap_or_else(|e| panic!("{case_id}: operand parses: {e}"));
            raw.validate(&universe)
                .unwrap_or_else(|e| panic!("{case_id}: operand validates: {e}"))
        };
        match case["property"].as_str() {
            Some("identity") => {
                let (pure, x) = (parse(0), parse(1));
                let reversed = combine(&x, &pure);
                assert_canonical_bytes(
                    case_id,
                    &reversed,
                    expected["canonical_ace1_utf8"].as_str().unwrap(),
                );
            }
            Some("associativity") => {
                let parse = |index: usize| -> EffectSignature {
                    let raw: EffectSignature =
                        serde_json::from_value(case["operands"][index].clone()).unwrap();
                    raw.validate(&universe).unwrap()
                };
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
            Some("commutativity") => {
                let parse = |index: usize| -> EffectSignature {
                    let raw: EffectSignature =
                        serde_json::from_value(case["operands"][index].clone()).unwrap();
                    raw.validate(&universe).unwrap()
                };
                let backwards = combine(&parse(1), &parse(0));
                assert_canonical_bytes(
                    case_id,
                    &backwards,
                    expected["canonical_ace1_utf8"].as_str().unwrap(),
                );
            }
            Some("idempotence") => {
                let raw: EffectSignature = serde_json::from_value(case["operands"][0].clone())
                    .unwrap_or_else(|e| panic!("{case_id}: operand parses: {e}"));
                let a = raw.validate(&universe).unwrap();
                let again = combine(&a, &a);
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

#[test]
fn atom_key_uses_full_qualifier_and_selector_identity_only() {
    fn atom(catalog: &str, name: &str, port: u32, scope: &str, qualifier: &str) -> EffectAtom {
        serde_json::from_value(serde_json::json!({
            "operation": {"catalog": catalog, "name": name},
            "resource_selector": {
                "requirement_port_no": port,
                "scope_bindings": [{"key": "tenant", "value": scope}]
            },
            "qualifiers": [{"idempotency": qualifier}]
        }))
        .unwrap()
    }

    let natural = atom(
        "agentflow.test.effects@1",
        "payment.charge",
        1,
        "a",
        "natural",
    );
    let natural_key = AtomKey::of(&natural);
    assert_eq!(natural_key.qualifiers()[0].family_no(), 1);
    assert_eq!(natural_key.qualifiers()[0].member_no(), 1);
    let none_key = AtomKey::of(&atom(
        "agentflow.test.effects@1",
        "payment.charge",
        1,
        "a",
        "none",
    ));
    assert_eq!(none_key.qualifiers()[0].member_no(), 2);
    assert_ne!(natural_key, none_key);
    assert!(
        natural_key < none_key,
        "frozen member numbers define key order"
    );
    assert_eq!(
        natural_key,
        AtomKey::of(&atom(
            "agentflow.test.effects@1",
            "payment.charge",
            1,
            "a",
            "natural"
        ))
    );
    assert_ne!(
        natural_key,
        AtomKey::of(&atom(
            "agentflow.test.other-effects@1",
            "payment.charge",
            1,
            "a",
            "natural"
        ))
    );
    assert_ne!(
        natural_key,
        AtomKey::of(&atom(
            "agentflow.test.effects@1",
            "payment.refund",
            1,
            "a",
            "natural"
        ))
    );
    assert_ne!(
        natural_key,
        AtomKey::of(&atom(
            "agentflow.test.effects@1",
            "payment.charge",
            2,
            "a",
            "natural"
        ))
    );
    assert_ne!(
        natural_key,
        AtomKey::of(&atom(
            "agentflow.test.effects@1",
            "payment.charge",
            1,
            "b",
            "natural"
        ))
    );

    let same_atom_other_entry_semantics: af_effect::EffectEntry =
        serde_json::from_value(serde_json::json!({
            "atom": {
                "operation": {
                    "catalog": "agentflow.test.effects@1",
                    "name": "payment.charge"
                },
                "resource_selector": {
                    "requirement_port_no": 1,
                    "scope_bindings": [{"key": "tenant", "value": "a"}]
                },
                "qualifiers": [{"idempotency": "natural"}]
            },
            "bounds": {"lower": 0, "upper": {"finite": 7}},
            "guard": {"input_equals": {
                "port_no": 2, "expected": {"string": "enabled"}
            }}
        }))
        .unwrap();
    assert_eq!(
        natural_key,
        AtomKey::of(&same_atom_other_entry_semantics.atom)
    );
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
            let signature: EffectSignature = serde_json::from_value(member.clone())
                .unwrap_or_else(|e| panic!("{case_id}: member parses: {e}"));
            let canonical = signature
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

// ---------------------------------------------------------------------------
// Failure-surface hygiene
// ---------------------------------------------------------------------------

#[test]
fn errors_carry_the_stable_kinds() {
    let error = EffectError::new("invalid_resource_selector", "test");
    assert_eq!(error.kind(), "invalid_resource_selector");
    assert_eq!(error.to_string(), "invalid_resource_selector: test");
}
