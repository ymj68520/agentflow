//! Information-flow conformance harness (Phase 4A.4): consumes the FROZEN
//! corpus under `spec/conformance/information-flow/` (generated once by
//! the independent Python implementation
//! `tools/gen-information-flow-corpus.py`).
//!
//! Expected canonical ACE1 bytes, error kinds, flow verdicts, propagation
//! results and compatibility verdicts all come from the corpus; tests
//! never generate expected values from the implementation under test.

use af_flow::{
    FlowCatalogs, FlowCompatibilityProof, FlowKind, FlowProof, FlowSink, InformationFlowContract,
    InstructionAuthority, StaticInformationSummary, TaintRef, check_flow, flow_not_relaxed,
    mix_model_visible_inputs, propagate_pure_expression, propagate_unknown_transformation,
    union_taints,
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
        .join("spec/conformance/information-flow")
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
        (
            "information-flow.afidl".to_string(),
            read("spec/idl/information-flow.afidl"),
        ),
    ];
    af_idl::compile_module_set(&sources).expect("the normative module set compiles")
}

/// The explicit catalog set of one case, from its frozen catalog block.
fn catalogs_of(case: &serde_json::Value) -> FlowCatalogs {
    let mut catalogs = FlowCatalogs::new();
    for label in case["catalogs"]["labels"]["confidentiality"]
        .as_array()
        .unwrap()
    {
        catalogs.labels.recognize_confidentiality(
            label["catalog"].as_str().unwrap(),
            label["name"].as_str().unwrap(),
        );
    }
    for label in case["catalogs"]["labels"]["taints"].as_array().unwrap() {
        catalogs.labels.recognize_taint(
            label["catalog"].as_str().unwrap(),
            label["name"].as_str().unwrap(),
        );
    }
    for operation in case["catalogs"]["effects"].as_array().unwrap() {
        catalogs.sinks.recognize_operation(
            operation["catalog"].as_str().unwrap(),
            operation["name"].as_str().unwrap(),
            operation["decision_capable"].as_bool().unwrap(),
        );
    }
    catalogs
}

fn contract_of(case: &serde_json::Value, key: &str) -> InformationFlowContract {
    serde_json::from_value(case[key].clone())
        .unwrap_or_else(|e| panic!("{}: flow contract parses: {e}", case["case_id"]))
}

fn summary_of(value: &serde_json::Value) -> StaticInformationSummary {
    // Propagation inputs use the compact authoring form (authority /
    // influence_unknown); contract summaries are the plain serde form.
    if value.get("authority").is_some() {
        StaticInformationSummary {
            sources: serde_json::from_value(value["sources"].clone()).unwrap(),
            taints: serde_json::from_value(value["taints"].clone()).unwrap(),
            instruction_authority: serde_json::from_value(value["authority"].clone()).unwrap(),
            influence_unknown: value["influence_unknown"].as_bool().unwrap(),
        }
    } else {
        serde_json::from_value(value.clone()).unwrap_or_else(|e| panic!("summary parses: {e}"))
    }
}

fn kind_of(value: &serde_json::Value) -> FlowKind {
    serde_json::from_value(value.clone()).expect("flow kind parses")
}

fn sink_of(value: &serde_json::Value) -> FlowSink {
    serde_json::from_value(value.clone()).expect("flow sink parses")
}

// ---------------------------------------------------------------------------
// Golden
// ---------------------------------------------------------------------------

#[test]
fn golden_cases_match_the_frozen_canonical_bytes_and_verdicts() {
    let files = case_files("golden");
    assert!(files.len() >= 9, "the golden corpus shrank");
    for case in &files {
        let case_id = case["case_id"].as_str().unwrap();
        assert_eq!(
            case["corpus_revision"].as_u64(),
            Some(CORPUS_REVISION),
            "{case_id}: corpus revision drifted"
        );
        let catalogs = catalogs_of(case);
        let contract = contract_of(case, "contract");
        let canonical = contract
            .canonical(&catalogs)
            .unwrap_or_else(|e| panic!("{case_id}: canonicalizes: {e}"));
        let bytes = canonical
            .canonical_bytes(&catalogs, &module_set())
            .unwrap_or_else(|e| panic!("{case_id}: canonical bytes: {e}"));
        assert_eq!(
            String::from_utf8(bytes).unwrap_or_else(|e| panic!("{case_id}: utf8: {e}")),
            case["canonical_ace1_utf8"].as_str().unwrap(),
            "{case_id}: canonical ACE1 bytes drifted from the frozen oracle"
        );
        assert_eq!(
            canonical.confidentiality_restrictions.is_empty()
                && canonical.flow_constraints.is_empty(),
            case["is_empty"].as_bool().unwrap(),
            "{case_id}: is_empty"
        );
        assert_eq!(
            canonical.confidentiality_restrictions.len(),
            usize::try_from(case["restriction_count"].as_u64().unwrap()).unwrap(),
            "{case_id}: restriction count"
        );
        assert_eq!(
            canonical.flow_constraints.len(),
            usize::try_from(case["constraint_count"].as_u64().unwrap()).unwrap(),
            "{case_id}: constraint count"
        );
        let twice = canonical.canonical(&catalogs).unwrap();
        assert_eq!(
            twice, canonical,
            "{case_id}: canonicalization is not idempotent"
        );

        for verdict in case["verdicts"].as_array().unwrap() {
            let proof = check_flow(
                &contract,
                kind_of(&verdict["kind"]),
                &sink_of(&verdict["sink"]),
                &catalogs,
            )
            .unwrap_or_else(|e| panic!("{case_id}: check_flow: {e}"));
            assert_proof(case_id, &proof, &verdict["expected"]);
        }
    }
}

fn assert_proof(case_id: &str, proof: &FlowProof, expected: &serde_json::Value) {
    match expected["outcome"].as_str().unwrap() {
        "proven_within_declared_ceiling" => {
            assert_eq!(
                proof,
                &FlowProof::ProvenWithinDeclaredCeiling,
                "{case_id}: verdict drifted"
            );
        }
        "requires_trusted_transformation" => {
            let (dec, endo) = match proof {
                FlowProof::RequiresTrustedTransformation {
                    requires_declassification,
                    requires_endorsement,
                } => (*requires_declassification, *requires_endorsement),
                other => panic!("{case_id}: expected obligation, got {other:?}"),
            };
            assert_eq!(
                dec,
                expected["requires_declassification"].as_bool().unwrap(),
                "{case_id}: declassification obligation drifted"
            );
            assert_eq!(
                endo,
                expected["requires_endorsement"].as_bool().unwrap(),
                "{case_id}: endorsement obligation drifted"
            );
        }
        "unproven" => {
            assert_eq!(proof, &FlowProof::Unproven, "{case_id}: verdict drifted");
        }
        other => panic!("{other}: unexpected verdict outcome"),
    }
}

// ---------------------------------------------------------------------------
// Negative
// ---------------------------------------------------------------------------

#[test]
fn negative_cases_are_rejected_with_the_frozen_kinds() {
    let files = case_files("negative");
    assert!(files.len() >= 10, "the negative corpus shrank");
    for case in &files {
        let case_id = case["case_id"].as_str().unwrap();
        let expected = case["expected_kind"].as_str().unwrap();
        let result = match case["operation"].as_str() {
            Some("check_flow") => check_flow(
                &contract_of(case, "contract"),
                kind_of(&case["kind"]),
                &sink_of(&case["sink"]),
                &catalogs_of(case),
            )
            .map(|_| ()),
            Some("require_proven") => check_flow(
                &contract_of(case, "contract"),
                kind_of(&case["kind"]),
                &sink_of(&case["sink"]),
                &catalogs_of(case),
            )
            .unwrap_or_else(|e| panic!("{case_id}: check_flow itself failed: {e}"))
            .require_proven(),
            Some("propagate_pure_expression") => {
                let inputs: Vec<StaticInformationSummary> = case["inputs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(summary_of)
                    .collect();
                let borrowed: Vec<&StaticInformationSummary> = inputs.iter().collect();
                propagate_pure_expression(
                    &borrowed,
                    serde_json::from_value::<InstructionAuthority>(
                        case["output_authority"].clone(),
                    )
                    .expect("authority parses"),
                )
                .map(|_| ())
            }
            _ => contract_of(case, "contract")
                .canonical(&catalogs_of(case))
                .map(|_| ()),
        };
        let error = result.expect_err(&format!("{case_id}: must be rejected"));
        assert_eq!(error.kind(), expected, "{case_id}: rejection kind drifted");
    }
}

// ---------------------------------------------------------------------------
// Propagation
// ---------------------------------------------------------------------------

#[test]
fn propagation_cases_reproduce_the_frozen_results() {
    let files = case_files("propagation");
    assert!(files.len() >= 12, "the propagation corpus shrank");
    for case in &files {
        let case_id = case["case_id"].as_str().unwrap();
        let catalogs = catalogs_of(case);
        match case["operation"].as_str().unwrap() {
            "union_taints" => {
                let sets: Vec<Vec<TaintRef>> = case["operand_sets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|set| {
                        serde_json::from_value(set.clone())
                            .unwrap_or_else(|e| panic!("{case_id}: taint set parses: {e}"))
                    })
                    .collect();
                let result = union_taints(&sets);
                let frozen = serde_json::to_value(&result).unwrap();
                assert_eq!(
                    frozen, case["taints"],
                    "{case_id}: taint union drifted from the frozen oracle"
                );
            }
            "propagate_pure_expression" => {
                let inputs: Vec<StaticInformationSummary> = case["inputs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(summary_of)
                    .collect();
                let borrowed: Vec<&StaticInformationSummary> = inputs.iter().collect();
                let claimed: InstructionAuthority =
                    serde_json::from_value(case["output_authority"].clone()).unwrap();
                let result = propagate_pure_expression(&borrowed, claimed)
                    .unwrap_or_else(|e| panic!("{case_id}: propagation: {e}"));
                assert_summary(case_id, &result, &case["summary"]);
            }
            "propagate_unknown_transformation" => {
                let inputs: Vec<StaticInformationSummary> = case["inputs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(summary_of)
                    .collect();
                let borrowed: Vec<&StaticInformationSummary> = inputs.iter().collect();
                let result = propagate_unknown_transformation(&borrowed);
                assert_summary(case_id, &result, &case["summary"]);
            }
            "mix_model_visible_inputs" => {
                let inputs: Vec<StaticInformationSummary> = case["inputs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(summary_of)
                    .collect();
                let marker: TaintRef = serde_json::from_value(serde_json::json!({
                    "catalog": "agentflow.infoflow.taints@1",
                    "name": "model_generated"
                }))
                .unwrap();
                let borrowed: Vec<&StaticInformationSummary> = inputs.iter().collect();
                let result = mix_model_visible_inputs(&borrowed, &marker, &catalogs.labels)
                    .unwrap_or_else(|e| panic!("{case_id}: mixer: {e}"));
                assert_summary(case_id, &result, &case["summary"]);
            }
            other => panic!("{other}: unexpected propagation operation"),
        }
    }
}

fn assert_summary(case_id: &str, result: &StaticInformationSummary, frozen: &serde_json::Value) {
    let actual = serde_json::to_value(result).unwrap();
    assert_eq!(
        actual, *frozen,
        "{case_id}: propagation result drifted from the frozen oracle"
    );
}

// ---------------------------------------------------------------------------
// Compatibility
// ---------------------------------------------------------------------------

#[test]
fn compatibility_cases_reproduce_the_frozen_verdicts() {
    let files = case_files("compatibility");
    assert!(files.len() >= 11, "the compatibility corpus shrank");
    for case in &files {
        let case_id = case["case_id"].as_str().unwrap();
        let catalogs = catalogs_of(case);
        let expected = &case["expected"];
        let result = flow_not_relaxed(
            &contract_of(case, "new_contract"),
            &contract_of(case, "old_contract"),
            &catalogs,
        );
        match expected["outcome"].as_str().unwrap() {
            "proven" => {
                assert_eq!(
                    result.expect("{case_id}: must prove"),
                    FlowCompatibilityProof::Proven,
                    "{case_id}: verdict drifted"
                );
            }
            "unproven" => {
                let reason = match result.expect("{case_id}: must decide") {
                    FlowCompatibilityProof::Unproven(reason) => reason,
                    FlowCompatibilityProof::Proven => {
                        panic!("{case_id}: unexpectedly proven")
                    }
                };
                assert_eq!(
                    reason.as_str(),
                    expected["reason"].as_str().unwrap(),
                    "{case_id}: unproven reason drifted"
                );
            }
            "relaxed" => {
                let error = result.expect_err(&format!("{case_id}: must be rejected"));
                assert_eq!(
                    error.kind(),
                    "flow_contract_relaxed",
                    "{case_id}: rejection kind drifted"
                );
            }
            "unknown_label_in_new_catalog" => {
                let error = result.expect_err(&format!("{case_id}: must fail closed"));
                assert_eq!(
                    error.kind(),
                    "unknown_information_label",
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
        let catalogs = catalogs_of(case);
        let mut reference: Option<String> = None;
        for member in case["members"].as_array().unwrap() {
            let contract: InformationFlowContract = serde_json::from_value(member.clone())
                .unwrap_or_else(|e| panic!("{case_id}: member parses: {e}"));
            let canonical = contract
                .canonical(&catalogs)
                .unwrap_or_else(|e| panic!("{case_id}: member validates: {e}"));
            let bytes = canonical
                .canonical_bytes(&catalogs, &module_set())
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
