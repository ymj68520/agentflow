//! Execution Requirements conformance harness.
//!
//! The expected ACE1 bytes, static match outcomes and compatibility verdicts
//! are frozen by the independent Python oracle under the execution corpus.

use af_execution::{
    CapabilityAttribute, ExecutionCapabilityRef, ExecutionRequirements, ExecutionUniverses,
    MatchRejected, MatchUnproven, RegionRef, RuntimeProfileRef, StaticCapability,
    StaticExecutorProfile, StaticPlacementProof, StaticQuantity, TrustDomainRef,
    execution_compatibility, match_execution_requirements,
};
use af_execution::{CompatibilityVerdict, canonicalize_execution_requirements};
use af_semantic::TypeSchemaRef;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn case_files(subdir: &str) -> Vec<Value> {
    let directory = root()
        .join("spec/conformance/execution-requirements")
        .join(subdir);
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&directory)
        .unwrap_or_else(|e| panic!("cannot list {}: {e}", directory.display()))
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            serde_json::from_str(
                &std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display())),
            )
            .unwrap_or_else(|e| panic!("cannot parse {}: {e}", path.display()))
        })
        .collect()
}

fn modules() -> af_idl::CompiledModuleSet {
    af_idl::compile_module_set(&[
        (
            "semantic-core.afidl".into(),
            std::fs::read_to_string(root().join("spec/idl/semantic-core.afidl")).unwrap(),
        ),
        (
            "execution-requirements.afidl".into(),
            std::fs::read_to_string(root().join("spec/idl/execution-requirements.afidl")).unwrap(),
        ),
    ])
    .expect("execution module set compiles")
}

fn universes(value: &Value) -> ExecutionUniverses {
    let mut universes = ExecutionUniverses::default();
    for entry in value["capabilities"].as_array().unwrap() {
        universes
            .capabilities
            .recognize(
                &ExecutionCapabilityRef {
                    catalog: entry["catalog"].as_str().unwrap().into(),
                    name: entry["name"].as_str().unwrap().into(),
                },
                entry["version"].as_str().unwrap(),
                entry["attributes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|attribute| CapabilityAttribute {
                        key: attribute["key"].as_str().unwrap().into(),
                        value: attribute["value"].as_str().unwrap().into(),
                    }),
            )
            .unwrap();
    }
    for entry in value["trust_domains"].as_array().unwrap() {
        universes
            .trust_domains
            .recognize(TrustDomainRef {
                catalog: entry["catalog"].as_str().unwrap().into(),
                name: entry["name"].as_str().unwrap().into(),
            })
            .unwrap();
    }
    for entry in value["regions"].as_array().unwrap() {
        universes
            .regions
            .recognize(RegionRef {
                catalog: entry["catalog"].as_str().unwrap().into(),
                name: entry["name"].as_str().unwrap().into(),
            })
            .unwrap();
    }
    for entry in value["runtime_profiles"].as_array().unwrap() {
        universes
            .runtime_profiles
            .recognize(
                RuntimeProfileRef {
                    catalog: entry["catalog"].as_str().unwrap().into(),
                    name: entry["name"].as_str().unwrap().into(),
                },
                entry["family"].as_str().unwrap(),
                entry["features"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|feature| feature.as_str().unwrap().to_string()),
            )
            .unwrap();
    }
    universes
}

fn profile(value: &Value) -> StaticExecutorProfile {
    StaticExecutorProfile {
        capabilities: value["capabilities"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| StaticCapability {
                capability: serde_json::from_value(entry["capability"].clone()).unwrap(),
                exact_version: entry["version"].as_str().unwrap().into(),
                attributes: entry["attributes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|attribute| {
                        (
                            attribute["key"].as_str().unwrap().into(),
                            attribute["value"].as_str().unwrap().into(),
                        )
                    })
                    .collect::<BTreeSet<_>>(),
            })
            .collect(),
        quantities: value["quantities"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| StaticQuantity {
                kind: serde_json::from_value(entry["kind"].clone()).unwrap(),
                amount: entry["amount"].as_u64().unwrap(),
            })
            .collect(),
        sandbox: optional_from_value(&value["sandbox"]),
        trust_domain: optional_from_value(&value["trust_domain"]),
        execution_region: optional_from_value(&value["execution_region"]),
        residency_regions: value["residency_regions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| serde_json::from_value(entry.clone()).unwrap())
            .collect(),
        runtime_profile: optional_from_value(&value["runtime_profile"]),
        implementation_abis: value["implementation_abis"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| serde_json::from_value(entry.clone()).unwrap())
            .collect::<Vec<TypeSchemaRef>>(),
    }
}

fn optional_from_value<T: serde::de::DeserializeOwned>(value: &Value) -> Option<T> {
    if value.is_null() {
        None
    } else {
        Some(serde_json::from_value(value.clone()).unwrap())
    }
}

fn requirements(value: &Value) -> ExecutionRequirements {
    serde_json::from_value(value["requirements"].clone())
        .unwrap_or_else(|e| panic!("{}: requirements parse: {e}", value["case_id"]))
}

fn assert_match(case_id: &str, actual: &StaticPlacementProof, expected: &Value) {
    match actual {
        StaticPlacementProof::ProvenAgainstDescriptor => {
            assert_eq!(expected["outcome"], "proven", "{case_id}");
        }
        StaticPlacementProof::Rejected(reason) => {
            assert_eq!(expected["outcome"], "rejected", "{case_id}");
            assert_eq!(rejected_reason(*reason), expected["reason"], "{case_id}");
        }
        StaticPlacementProof::Unproven(reason) => {
            assert_eq!(expected["outcome"], "unproven", "{case_id}");
            assert_eq!(unproven_reason(*reason), expected["reason"], "{case_id}");
        }
    }
}

fn rejected_reason(reason: MatchRejected) -> Value {
    Value::String(
        match reason {
            MatchRejected::CapabilityMissing => "capability_missing",
            MatchRejected::CapabilityVersion => "capability_version",
            MatchRejected::QuantityInsufficient => "quantity_insufficient",
            MatchRejected::SandboxTooWeak => "sandbox_too_weak",
            MatchRejected::TrustDomainMismatch => "trust_domain_mismatch",
            MatchRejected::RegionMismatch => "region_mismatch",
            MatchRejected::ResidencyMismatch => "residency_mismatch",
            MatchRejected::RuntimeProfileMismatch => "runtime_profile_mismatch",
            MatchRejected::AbiMismatch => "abi_mismatch",
        }
        .into(),
    )
}

fn unproven_reason(reason: MatchUnproven) -> Value {
    Value::String(
        match reason {
            MatchUnproven::CapabilityProfile => "capability_profile",
            MatchUnproven::QuantityProfile => "quantity_profile",
            MatchUnproven::SandboxProfile => "sandbox_profile",
            MatchUnproven::TrustDomain => "trust_domain",
            MatchUnproven::RegionResidency => "region_residency",
            MatchUnproven::RuntimeProfile => "runtime_profile",
            MatchUnproven::AbiProfile => "abi_profile",
        }
        .into(),
    )
}

fn verdict(value: CompatibilityVerdict) -> Value {
    Value::String(value.as_str().into())
}

#[test]
fn golden_cases_match_frozen_ace1_and_static_proofs() {
    let cases = case_files("golden");
    assert!(cases.len() >= 8, "execution golden corpus shrank");
    for case in cases {
        let universes = universes(&case["catalogs"]);
        let req = requirements(&case);
        let canonical = canonicalize_execution_requirements(&req, &universes).unwrap();
        let bytes = canonical.canonical_bytes(&universes, &modules()).unwrap();
        assert_eq!(
            String::from_utf8(bytes).unwrap(),
            case["canonical_ace1_utf8"]
        );
        assert_eq!(
            canonical,
            canonicalize_execution_requirements(&canonical, &universes).unwrap()
        );
        assert_match(
            case["case_id"].as_str().unwrap(),
            &match_execution_requirements(&req, &profile(&case["profile"]), &universes).unwrap(),
            &case["expected"],
        );
    }
}

#[test]
fn negative_cases_fail_closed_with_frozen_kinds() {
    let cases = case_files("negative");
    assert!(cases.len() >= 10, "execution negative corpus shrank");
    for case in cases {
        if case["malformed"].as_bool().unwrap_or(false) {
            assert!(
                serde_json::from_value::<ExecutionRequirements>(case["requirements"].clone())
                    .is_err(),
                "{} must not parse",
                case["case_id"]
            );
            continue;
        }
        let req: ExecutionRequirements =
            serde_json::from_value(case["requirements"].clone()).unwrap();
        let Err(error) = canonicalize_execution_requirements(&req, &universes(&case["catalogs"]))
        else {
            panic!("{} must fail", case["case_id"]);
        };
        assert_eq!(
            error.kind(),
            case["expected_kind"].as_str().unwrap(),
            "{}",
            case["case_id"]
        );
    }
}

#[test]
fn matching_cases_match_frozen_outcomes() {
    let cases = case_files("matching");
    assert!(cases.len() >= 12, "execution matching corpus shrank");
    for case in cases {
        let universes = universes(&case["catalogs"]);
        assert_match(
            case["case_id"].as_str().unwrap(),
            &match_execution_requirements(
                &requirements(&case),
                &profile(&case["profile"]),
                &universes,
            )
            .unwrap(),
            &case["expected"],
        );
    }
}

#[test]
fn compatibility_cases_freeze_separate_verdicts() {
    let cases = case_files("compatibility");
    assert!(cases.len() >= 10, "execution compatibility corpus shrank");
    for case in cases {
        let universes = universes(&case["catalogs"]);
        let new: ExecutionRequirements =
            serde_json::from_value(case["new_requirements"].clone()).unwrap();
        let old: ExecutionRequirements =
            serde_json::from_value(case["old_requirements"].clone()).unwrap();
        let actual = execution_compatibility(&new, &old, &universes).unwrap();
        assert_eq!(
            verdict(actual.feasibility),
            case["expected"]["feasibility"],
            "{}",
            case["case_id"]
        );
        assert_eq!(
            verdict(actual.security),
            case["expected"]["security"],
            "{}",
            case["case_id"]
        );
        assert_eq!(
            verdict(actual.combined()),
            case["expected"]["combined"],
            "{}",
            case["case_id"]
        );
    }
}

#[test]
fn equivalence_cases_have_identical_canonical_ace1() {
    let cases = case_files("equivalence");
    assert!(cases.len() >= 3, "execution equivalence corpus shrank");
    for case in cases {
        let universes = universes(&case["catalogs"]);
        let members = case["members"].as_array().unwrap();
        let mut bytes = None;
        for member in members {
            let req: ExecutionRequirements = serde_json::from_value(member.clone()).unwrap();
            let actual = canonicalize_execution_requirements(&req, &universes)
                .unwrap()
                .canonical_bytes(&universes, &modules())
                .unwrap();
            if let Some(expected) = &bytes {
                assert_eq!(expected, &actual, "{}", case["case_id"]);
            } else {
                bytes = Some(actual);
            }
        }
    }
}
