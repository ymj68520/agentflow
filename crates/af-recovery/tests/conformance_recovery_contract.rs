//! Recovery Contract conformance harness backed by the independent Python
//! corpus oracle.

use af_effect::{CapabilityCatalog, EffectCatalogUniverse, EffectSignature};
use af_recovery::{
    RecoveryCompatibilityProof, RecoveryContract, RecoveryEvidenceState, RecoveryOperationUniverse,
    RecoveryProof, analyze_recovery_safety, canonicalize_recovery_contract, recovery_not_weakened,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn read(relative: &str) -> String {
    std::fs::read_to_string(root().join(relative)).unwrap()
}

fn modules() -> af_idl::CompiledModuleSet {
    af_idl::compile_module_set(&[
        (
            "semantic-core.afidl".into(),
            read("spec/idl/semantic-core.afidl"),
        ),
        (
            "effect-signature.afidl".into(),
            read("spec/idl/effect-signature.afidl"),
        ),
        (
            "recovery-contract.afidl".into(),
            read("spec/idl/recovery-contract.afidl"),
        ),
    ])
    .unwrap()
}

fn cases(subdir: &str) -> Vec<Value> {
    let mut paths: Vec<_> = std::fs::read_dir(
        root()
            .join("spec/conformance/recovery-contract")
            .join(subdir),
    )
    .unwrap()
    .map(|entry| entry.unwrap().path())
    .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
    .collect();
    paths.sort();
    paths
        .into_iter()
        .map(|path| serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap())
        .collect()
}

fn universe_of(case: &Value) -> RecoveryOperationUniverse {
    let mut effects = EffectCatalogUniverse::new();
    let operations: Vec<String> = case["catalogs"]["effects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["name"].as_str().unwrap().to_string())
        .collect();
    let catalog = CapabilityCatalog::new(
        case["catalogs"]["effects"][0]["catalog"].as_str().unwrap(),
        operations,
    )
    .unwrap();
    effects.insert(&catalog).unwrap();
    let mut universe = RecoveryOperationUniverse::new();
    universe.effects = effects;
    for operation in case["catalogs"]["recovery_operations"].as_array().unwrap() {
        universe
            .recognize_reconciliation_operation(
                operation["catalog"].as_str().unwrap(),
                operation["name"].as_str().unwrap(),
            )
            .unwrap();
    }
    universe
}

fn contract_of(case: &Value, key: &str) -> RecoveryContract {
    serde_json::from_value(case[key].clone()).unwrap()
}

fn effects_of(case: &Value) -> Result<EffectSignature, serde_json::Error> {
    serde_json::from_value(case["effect_signature"].clone())
}

fn evidence_of(case: &Value) -> BTreeMap<af_effect::AtomKey, RecoveryEvidenceState> {
    let mut result = BTreeMap::new();
    let state = match case
        .get("evidence")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
    {
        "unknown" => RecoveryEvidenceState::Unknown,
        "still_unknown" => RecoveryEvidenceState::StillUnknown,
        "not_applied" => RecoveryEvidenceState::NotApplied,
        "applied" => RecoveryEvidenceState::Applied,
        "partially_applied" => RecoveryEvidenceState::PartiallyApplied,
        other => panic!("unknown evidence {other}"),
    };
    if let Ok(signature) = effects_of(case) {
        if let af_effect::EffectRow::Closed(entries) = signature.row {
            for entry in entries {
                result.insert(af_effect::AtomKey::of(&entry.atom), state);
            }
        }
    }
    result
}

fn proof_json(proof: &RecoveryProof) -> Value {
    match proof {
        RecoveryProof::ProvenStaticConstraint => json!({"outcome":"proven_static_constraint"}),
        RecoveryProof::RequiresEvidence(obligations) => json!({
            "outcome":"requires_evidence",
            "obligations": obligations.iter().map(|o| o.as_str()).collect::<Vec<_>>(),
        }),
        RecoveryProof::Unproven(reason) => json!({"outcome":"unproven","reason":reason.as_str()}),
        RecoveryProof::Violation(reason) => json!({"outcome":"violation","reason":reason.as_str()}),
    }
}

#[test]
fn golden_cases_match_canonical_bytes_and_proofs() {
    for case in cases("golden") {
        let universe = universe_of(&case);
        let contract = contract_of(&case, "contract");
        let canonical = canonicalize_recovery_contract(&contract, &universe).unwrap();
        let bytes = canonical.canonical_bytes(&universe, &modules()).unwrap();
        assert_eq!(
            String::from_utf8(bytes).unwrap(),
            case["canonical_ace1_utf8"]
        );
        let effect = effects_of(&case).unwrap();
        let proof =
            analyze_recovery_safety(&contract, &effect, &universe, &evidence_of(&case)).unwrap();
        assert_eq!(proof_json(&proof), case["expected"]);
    }
}

#[test]
fn negative_cases_fail_closed() {
    for case in cases("negative") {
        if case["case_id"] == "recovery/negative/invalid-replay" {
            assert!(serde_json::from_value::<RecoveryContract>(case["contract"].clone()).is_err());
            continue;
        }
        let universe = universe_of(&case);
        let contract = contract_of(&case, "contract");
        let effect = effects_of(&case).unwrap();
        let result = analyze_recovery_safety(&contract, &effect, &universe, &evidence_of(&case));
        match case["expected_kind"].as_str().unwrap() {
            "__requires__" | "recovery_contract_unproven" => {
                let proof = result.unwrap();
                assert_eq!(
                    proof.as_str(),
                    if case["expected_kind"] == "__requires__" {
                        "requires_evidence"
                    } else {
                        "unproven"
                    }
                );
            }
            expected => assert_eq!(result.unwrap_err().kind(), expected),
        }
    }
}

#[test]
fn replay_reconciliation_and_compensation_cases_match() {
    for subdir in ["replay", "reconciliation", "compensation"] {
        for case in cases(subdir) {
            let universe = universe_of(&case);
            let contract = contract_of(&case, "contract");
            let effect = effects_of(&case).unwrap();
            let proof = analyze_recovery_safety(&contract, &effect, &universe, &evidence_of(&case))
                .unwrap();
            assert_eq!(proof_json(&proof), case["expected"]);
        }
    }
}

#[test]
fn compatibility_cases_are_conservative() {
    for case in cases("compatibility") {
        let universe = universe_of(&case);
        let proof = recovery_not_weakened(
            &contract_of(&case, "new_contract"),
            &contract_of(&case, "old_contract"),
            &universe,
        )
        .unwrap();
        let actual = match proof {
            RecoveryCompatibilityProof::Proven => "proven",
            RecoveryCompatibilityProof::Weakened => "weakened",
            RecoveryCompatibilityProof::Unproven => "unproven",
        };
        assert_eq!(actual, case["expected"]["outcome"]);
    }
}

#[test]
fn equivalent_declarations_have_identical_bytes() {
    for case in cases("equivalence") {
        let universe = universe_of(&case);
        let mut reference = None;
        for member in case["members"].as_array().unwrap() {
            let contract: RecoveryContract = serde_json::from_value(member.clone()).unwrap();
            let bytes = contract.canonical_bytes(&universe, &modules()).unwrap();
            let text = String::from_utf8(bytes).unwrap();
            match reference.as_ref() {
                None => {
                    assert_eq!(text, case["canonical_ace1_utf8"]);
                    reference = Some(text);
                }
                Some(expected) => assert_eq!(expected, &text),
            }
        }
    }
}
