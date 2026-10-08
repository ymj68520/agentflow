//! Usage Signature conformance harness. Expected canonical bytes, algebra
//! states, proof results and stable errors come from the independent Python
//! corpus generator; Rust consumes them but never derives expectations.

use af_usage::{
    StaticUsageBound, StaticUsageEnvelope, UsageAggregation, UsageMeterRef, UsageMeterUniverse,
    UsagePhase, UsageProof, UsageSignature, UsageUnitRef, accumulate, branch_join,
    canonicalize_usage_signature, compensation_envelope, iterate_envelope, race_envelope, repeat,
    retry_envelope, usage_not_increased, within_ceiling,
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn case_files(subdir: &str) -> Vec<Value> {
    let directory = repo_root()
        .join("spec/conformance/usage-signature")
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

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

fn module_set() -> af_idl::CompiledModuleSet {
    let sources = [
        (
            "semantic-core.afidl".to_string(),
            read("spec/idl/semantic-core.afidl"),
        ),
        (
            "usage-signature.afidl".to_string(),
            read("spec/idl/usage-signature.afidl"),
        ),
    ];
    af_idl::compile_module_set(&sources).expect("usage module set compiles")
}

fn universe_of(value: &Value) -> UsageMeterUniverse {
    let mut universe = UsageMeterUniverse::new();
    for meter in value["meters"].as_array().unwrap() {
        let meter_ref = UsageMeterRef {
            catalog: meter["catalog"].as_str().unwrap().to_string(),
            name: meter["name"].as_str().unwrap().to_string(),
        };
        let unit = UsageUnitRef::new(
            meter["unit"]["catalog"].as_str().unwrap(),
            meter["unit"]["name"].as_str().unwrap(),
        )
        .unwrap();
        let aggregation = match meter["aggregation"].as_str().unwrap() {
            "additive" => UsageAggregation::Additive,
            "latency" => UsageAggregation::Unsupported,
            other => panic!("unknown test aggregation {other}"),
        };
        universe.recognize(&meter_ref, &unit, aggregation).unwrap();
    }
    universe
}

fn signature_of(value: &Value, key: &str) -> UsageSignature {
    serde_json::from_value(value[key].clone())
        .unwrap_or_else(|e| panic!("{}: signature parses: {e}", value["case_id"]))
}

fn envelope_of(value: &Value) -> StaticUsageEnvelope {
    if value["global_unknown"].as_bool().unwrap() {
        return StaticUsageEnvelope::unknown();
    }
    let mut envelope = StaticUsageEnvelope::empty();
    for bound in value["bounds"].as_array().unwrap() {
        let phase: UsagePhase = serde_json::from_value(bound["phase"].clone()).unwrap();
        let meter: UsageMeterRef = serde_json::from_value(bound["meter"].clone()).unwrap();
        let state = match bound["bound"]["kind"].as_str().unwrap() {
            "known_finite" => {
                StaticUsageBound::KnownFinite(bound["bound"]["value"].as_u64().unwrap())
            }
            "known_unbounded" => StaticUsageBound::KnownUnbounded,
            "unknown" => StaticUsageBound::Unknown,
            other => panic!("unknown static bound {other}"),
        };
        envelope
            .insert(af_usage::UsageCoordinate { phase, meter }, state)
            .unwrap();
    }
    envelope
}

fn project_envelope(envelope: &StaticUsageEnvelope) -> Value {
    let bounds: Vec<Value> = envelope
        .iter()
        .map(|(coordinate, bound)| {
            let state = match bound {
                StaticUsageBound::KnownFinite(value) => {
                    json!({"kind": "known_finite", "value": value})
                }
                StaticUsageBound::KnownUnbounded => json!({"kind": "known_unbounded"}),
                StaticUsageBound::Unknown => json!({"kind": "unknown"}),
            };
            json!({
                "phase": coordinate.phase.as_str(),
                "meter": coordinate.meter,
                "bound": state,
            })
        })
        .collect();
    json!({"global_unknown": envelope.is_globally_unknown(), "bounds": bounds})
}

fn assert_proof(case_id: &str, actual: UsageProof, expected: &Value) {
    assert_eq!(
        actual.as_str(),
        expected["outcome"].as_str().unwrap(),
        "{case_id}"
    );
    if let UsageProof::Unproven(reason) = actual {
        assert_eq!(
            reason.as_str(),
            expected["reason"].as_str().unwrap(),
            "{case_id}: unproven reason"
        );
    }
}

#[test]
fn golden_cases_match_canonical_usage_bytes_and_proofs() {
    let files = case_files("golden");
    assert!(files.len() >= 7, "usage golden corpus shrank");
    for case in &files {
        let universe = universe_of(&case["catalogs"]);
        let signature = signature_of(case, "signature");
        let canonical = canonicalize_usage_signature(&signature, &universe)
            .unwrap_or_else(|e| panic!("{}: canonicalizes: {e}", case["case_id"]));
        let bytes = canonical
            .canonical_bytes(&universe, &module_set())
            .unwrap_or_else(|e| panic!("{}: bytes: {e}", case["case_id"]));
        assert_eq!(
            String::from_utf8(bytes).unwrap(),
            case["canonical_ace1_utf8"].as_str().unwrap(),
            "{}: canonical bytes",
            case["case_id"]
        );
        if let Some(proof) = case.get("proof") {
            let observed = StaticUsageEnvelope::empty();
            let declared = signature.static_envelope(&universe).unwrap();
            assert_proof(
                case["case_id"].as_str().unwrap(),
                within_ceiling(&observed, &declared),
                proof,
            );
        }
        assert_eq!(canonical, canonical.canonical(&universe).unwrap());
    }
}

#[test]
fn negative_cases_fail_with_frozen_usage_kinds() {
    let files = case_files("negative");
    assert!(files.len() >= 7, "usage negative corpus shrank");
    for case in &files {
        let expected = case["expected_kind"].as_str().unwrap();
        let result = match case["operation"].as_str().unwrap() {
            "canonicalize" => canonicalize_usage_signature(
                &signature_of(case, "signature"),
                &universe_of(&case["catalogs"]),
            )
            .map(|_| ()),
            "accumulate" => {
                accumulate(&envelope_of(&case["left"]), &envelope_of(&case["right"])).map(|_| ())
            }
            "require_within_ceiling" => {
                let universe = universe_of(&case["catalogs"]);
                let declared = signature_of(case, "declared")
                    .static_envelope(&universe)
                    .unwrap();
                within_ceiling(&envelope_of(&case["observed"]), &declared).require_proven()
            }
            "require_compatibility" => {
                let new_universe =
                    universe_of(case.get("new_catalogs").unwrap_or(&case["catalogs"]));
                let old_universe =
                    universe_of(case.get("old_catalogs").unwrap_or(&case["catalogs"]));
                usage_not_increased(
                    &signature_of(case, "new_signature"),
                    &new_universe,
                    &signature_of(case, "old_signature"),
                    &old_universe,
                )
                .unwrap()
                .require_proven()
            }
            other => panic!("{}: unexpected negative operation {other}", case["case_id"]),
        };
        let error = result.expect_err(&format!("{}: must fail", case["case_id"]));
        assert_eq!(error.kind(), expected, "{}", case["case_id"]);
    }
}

#[test]
fn algebra_cases_match_independent_expected_envelopes() {
    let files = case_files("algebra");
    assert!(files.len() >= 13, "usage algebra corpus shrank");
    for case in &files {
        let actual = match case["operation"].as_str().unwrap() {
            "accumulate" => {
                accumulate(&envelope_of(&case["left"]), &envelope_of(&case["right"])).unwrap()
            }
            "branch_join" => {
                branch_join(&envelope_of(&case["left"]), &envelope_of(&case["right"])).unwrap()
            }
            "repeat" => repeat(
                case["count"].as_u64().unwrap(),
                &envelope_of(&case["input"]),
            )
            .unwrap(),
            "iterate_unknown" => iterate_envelope(None, &envelope_of(&case["body"])).unwrap(),
            "within_ceiling" => {
                let universe = universe_of(&case["catalogs"]);
                let declared = signature_of(case, "declared")
                    .static_envelope(&universe)
                    .unwrap();
                let proof = within_ceiling(&envelope_of(&case["observed"]), &declared);
                assert_proof(case["case_id"].as_str().unwrap(), proof, &case["result"]);
                continue;
            }
            "usage_not_increased" => {
                let new_universe =
                    universe_of(case.get("new_catalogs").unwrap_or(&case["catalogs"]));
                let old_universe =
                    universe_of(case.get("old_catalogs").unwrap_or(&case["catalogs"]));
                let proof = usage_not_increased(
                    &signature_of(case, "new_signature"),
                    &new_universe,
                    &signature_of(case, "old_signature"),
                    &old_universe,
                )
                .unwrap();
                assert_proof(case["case_id"].as_str().unwrap(), proof, &case["result"]);
                continue;
            }
            "race" => race_envelope(
                &case["branches"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(envelope_of)
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
            "retry" => {
                retry_envelope(
                    &envelope_of(&case["body"]),
                    case["max_attempts"].as_u64().unwrap(),
                )
                .unwrap()
                .logical
            }
            "compensation" => compensation_envelope(
                &envelope_of(&case["forward"]),
                &envelope_of(&case["recovery"]),
            )
            .unwrap(),
            other => panic!("{}: unexpected algebra operation {other}", case["case_id"]),
        };
        assert_eq!(
            project_envelope(&actual),
            case["result"],
            "{}",
            case["case_id"]
        );
    }
}

#[test]
fn compatibility_cases_match_usage_non_increase_proofs() {
    let files = case_files("compatibility");
    assert!(files.len() >= 6, "usage compatibility corpus shrank");
    for case in &files {
        let new_universe = universe_of(case.get("new_catalogs").unwrap_or(&case["catalogs"]));
        let old_universe = universe_of(case.get("old_catalogs").unwrap_or(&case["catalogs"]));
        let proof = usage_not_increased(
            &signature_of(case, "new_signature"),
            &new_universe,
            &signature_of(case, "old_signature"),
            &old_universe,
        )
        .unwrap();
        assert_proof(case["case_id"].as_str().unwrap(), proof, &case["expected"]);
    }
}

#[test]
fn equivalence_groups_have_identical_canonical_bytes() {
    for case in &case_files("equivalence") {
        let universe = universe_of(&case["catalogs"]);
        let mut reference = None;
        for member in case["members"].as_array().unwrap() {
            let signature: UsageSignature = serde_json::from_value(member.clone()).unwrap();
            let bytes = signature.canonical_bytes(&universe, &module_set()).unwrap();
            let text = String::from_utf8(bytes).unwrap();
            match reference.as_ref() {
                None => {
                    assert_eq!(text, case["canonical_ace1_utf8"]);
                    reference = Some(text);
                }
                Some(expected) => {
                    assert_eq!(expected, &text);
                }
            }
        }
    }
}
