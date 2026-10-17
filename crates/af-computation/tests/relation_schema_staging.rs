#![recursion_limit = "512"]
use af_computation::*;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn read(file: &str) -> Value {
    serde_json::from_slice(
        &std::fs::read(
            root()
                .join("spec/conformance/g01-relation-schema-staging")
                .join(file),
        )
        .unwrap(),
    )
    .unwrap()
}
#[test]
fn closed_successor_shape_and_every_field_variant_execute() {
    let corpus = read("shape-cases.json");
    let mut actual = Vec::new();
    for c in corpus["cases"].as_array().unwrap() {
        let outcome = validate_relation_registered_shape(c["type"].as_str().unwrap(), &c["value"])
            .err()
            .map(|e| format!("{e:?}"));
        assert_eq!(
            outcome.as_deref(),
            c["expected_failure"].as_str(),
            "{}",
            c["case_id"]
        );
        actual.push(json!({"case_id":c["case_id"],"actual_failure":outcome}));
    }
    std::fs::write(
        root().join("target/phase-04c1b4c4r3c2r3g2-shape-actual.json"),
        serde_json::to_vec_pretty(&actual).unwrap(),
    )
    .unwrap();
}
#[test]
fn exact_successor_registration_never_activates_proof() {
    let e = current_registered_analysis_edition();
    assert_eq!(e.registration, EditionRegistration::Registered);
    assert_eq!(
        e.compiler.analysis_semantics_version,
        "agentflow.root-export-checker@7"
    );
    assert_eq!(
        (
            e.compiler.specifications.len(),
            e.compiler.rules.len(),
            e.registered_sources.len()
        ),
        (40, 204, 66)
    );
    assert!(!e.can_issue_capability());
    check_registered_compiler_identity(&e.compiler).unwrap();
    relation_descriptor_integrity().unwrap();
    // Historical metadata probes remain explicitly model@5, never current tuple admission.
    assert_eq!(
        registered_model_catalogue().unwrap()["model_id"],
        "agentflow.computation.proof-report.model@5"
    );
    assert_eq!(
        relation_model_catalogue().unwrap()["inherited_registered_model"],
        *registered_model_catalogue().unwrap()
    );
    for n in 1..=6 {
        let mut wrong = e.compiler.clone();
        wrong.analysis_semantics_version = format!("agentflow.root-export-checker@{n}");
        assert!(check_registered_compiler_identity(&wrong).is_err());
    }
    assert!(
        check_registered_report_editions(
            "agentflow.g01.report-shape@5",
            "agentflow.g01.obligation@5"
        )
        .is_err()
    );
    for profile in [
        "root_export",
        "raw_contract_validation",
        "native_recovery_guard",
        "standalone_substitution",
    ] {
        assert_eq!(
            admit_registered_profile(profile, false),
            Err(RegisteredModelFailure::NotImplemented)
        );
    }
    let rules: Value = serde_json::from_slice(&e.compiler.rule_registry_content).unwrap();
    let mut actual = Vec::new();
    for r in read("rule-registration.json")["cases"].as_array().unwrap() {
        let rule = rules["rules"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["rule_id"] == r["rule_id"])
            .unwrap();
        assert_eq!(rule["source_anchor"], r["anchor"]);
        assert_eq!(rule["short_statement"], r["statement"]);
        actual.push(json!({"case_id":r["case_id"],"rule_id":r["rule_id"],"actual_registered":true,"positive_capability":false}));
    }
    std::fs::write(
        root().join("target/phase-04c1b4c4r3c2r3g2-rule-actual.json"),
        serde_json::to_vec_pretty(&actual).unwrap(),
    )
    .unwrap();
}

#[test]
fn closed_source_dag_subgraph_is_minimal_and_has_no_output_backedge() {
    let mut actual = Vec::new();
    for c in read("dag-cases.json")["cases"].as_array().unwrap() {
        let failure = validate_relation_source_dag_metadata(&c["input"])
            .err()
            .map(|e| format!("{e:?}"));
        assert_eq!(
            failure.as_deref(),
            c["expected_failure"].as_str(),
            "{}",
            c["case_id"]
        );
        actual.push(json!({"case_id":c["case_id"],"actual_failure":failure}));
    }
    std::fs::write(
        root().join("target/phase-04c1b4c4r3c2r3g2-dag-actual.json"),
        serde_json::to_vec_pretty(&actual).unwrap(),
    )
    .unwrap();
}

#[allow(dead_code, unused_imports)]
mod fixtures {
    include!("../../af-contract/tests/component_contract_conformance.rs");
    use af_computation::{RelationMetadataSlot as S, *};
    use af_contract::InformationFlowExecutionCatalog;
    use serde_json::{Value, json};
    fn files(dir: &std::path::Path, out: &mut Vec<(String, String)>) {
        for e in std::fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                files(&p, out);
            } else if p.extension().is_some_and(|x| x == "afidl") {
                out.push((
                    p.to_string_lossy().into(),
                    std::fs::read_to_string(p).unwrap(),
                ));
            }
        }
    }
    fn relation_modules(slot: S, mode: &str) -> CompiledModuleSet {
        let mut inputs = Vec::new();
        files(&repo_root().join("spec/idl"), &mut inputs);
        inputs.sort_by(|a, b| a.0.cmp(&b.0));
        let filename = match slot {
            S::EffectResource => "effect-resource-compatibility.afidl",
            S::RecoveryUsage => "recovery-usage-coverage.afidl",
            S::FlowExecution => "flow-execution-relation.afidl",
        };
        if mode == "missing_module" {
            inputs.retain(|(p, _)| !p.ends_with(filename));
        }
        for (path, text) in &mut inputs {
            if !path.ends_with(filename) {
                continue;
            }
            if mode == "wrong_family" {
                let header = format!("schema {};", slot.family().replace('@', " @"));
                assert!(text.contains(&header));
                *text = text.replace(&header, "schema wrong.family @1;");
            } else if mode == "B" {
                let (old, new) = match slot {
                    S::EffectResource => (
                        "2 supported_pairs: list<SupportedEffectResourcePair>;",
                        "2 supported_pairs: list<string>;",
                    ),
                    S::RecoveryUsage => (
                        "1 rules: list<RecoveryUsageCoverageRule>;",
                        "1 rules: list<string>;",
                    ),
                    S::FlowExecution => (
                        "1 closed_regions: list<execution.RegionRef>;",
                        "1 closed_regions: list<string>;",
                    ),
                };
                assert!(text.contains(old));
                *text = text.replace(old, new);
            } else if mode == "invalid" {
                let field = match slot {
                    S::EffectResource => "complete_operations",
                    S::RecoveryUsage => "rules",
                    S::FlowExecution => "closed_regions",
                };
                let at = text
                    .find(&format!("record {} {{", slot.root_type()))
                    .unwrap();
                let (head, tail) = text.split_at(at);
                *text = format!(
                    "{head}{}",
                    tail.replacen(&format!(" {field}:"), &format!(" renamed_{field}:"), 1)
                );
            } else if mode == "missing_root" {
                *text = text.replace(
                    &format!("record {} {{", slot.root_type()),
                    &format!("record Renamed{} {{", slot.root_type()),
                );
            }
        }
        compile_module_set(&inputs).unwrap()
    }
    fn source_relations(slot: S, nonempty: bool, modules: &CompiledModuleSet) -> RelationSnapshots {
        let input = if nonempty {
            let family = match slot {
                S::EffectResource => "effect-resource",
                S::RecoveryUsage => "recovery-usage",
                S::FlowExecution => "flow-execution",
            };
            let v:Value=serde_json::from_slice(&std::fs::read(repo_root().join(format!("spec/conformance/component-contract/relation-catalogs/{family}/canonical/cases.json"))).unwrap()).unwrap();
            v["cases"][0]["input_value"].clone()
        } else {
            match slot {
                S::EffectResource => json!({"complete_operations":[],"supported_pairs":[]}),
                S::RecoveryUsage => json!({"rules":[]}),
                S::FlowExecution => {
                    json!({"closed_regions":[],"closed_trust_domains":[],"confidentiality_rules":[],"taint_rules":[],"authority_rules":[],"sink_rules":[]})
                }
            }
        };
        let mut r = RelationSnapshots::default();
        match slot {
            S::EffectResource => {
                r.effect_resource = Some(
                    RelationCatalogSnapshot::new(
                        "g2-fixture",
                        serde_json::from_value::<EffectResourceCompatibilityCatalog>(input)
                            .unwrap(),
                        modules,
                    )
                    .unwrap(),
                );
            }
            S::RecoveryUsage => {
                r.recovery_usage = Some(
                    RelationCatalogSnapshot::new(
                        "g2-fixture",
                        serde_json::from_value::<RecoveryUsageCoverageCatalog>(input).unwrap(),
                        modules,
                    )
                    .unwrap(),
                );
            }
            S::FlowExecution => {
                r.flow_execution = Some(
                    RelationCatalogSnapshot::new(
                        "g2-fixture",
                        serde_json::from_value::<InformationFlowExecutionCatalog>(input).unwrap(),
                        modules,
                    )
                    .unwrap(),
                );
            }
        }
        r
    }
    #[test]
    fn real_three_slot_sources_lookup_revalidation_and_metadata_negatives() {
        let corpus = super::read("admission-cases.json");
        let expected = super::read("admission-expected.json");
        let compiler = current_compiler_semantics_identity();
        let mut actual = Vec::new();
        for c in corpus["cases"].as_array().unwrap() {
            let slot = match c["slot"].as_str().unwrap() {
                "effect_resource" => S::EffectResource,
                "recovery_usage" => S::RecoveryUsage,
                "flow_execution" => S::FlowExecution,
                _ => panic!("unknown slot"),
            };
            let construction = relation_modules(slot, "A");
            let current = relation_modules(slot, c["current"].as_str().unwrap());
            let mut fixture = FixtureContext::new();
            fixture.modules = current;
            if c["present"] == true {
                fixture.relations =
                    source_relations(slot, c["catalog"] == "nonempty", &construction);
            } else {
                fixture.relations = RelationSnapshots::default();
            }
            let output = if c["mode"] == "ambiguous" {
                let b = relation_modules(slot, "B");
                let id = format!("{}/{}", slot.family(), slot.root_type());
                json!({"boundary_lookup":inspect_relation_definition_candidates(slot,&[construction.type_by_identity(&id).unwrap().clone(),b.type_by_identity(&id).unwrap().clone()]),"raw_capture_complete":false,"positive_root_export":false})
            } else {
                let context = fixture.view();
                let separate = fixture.view();
                let mut supplied = compiler.clone();
                if c["compiler"] == "wrong" {
                    supplied.analysis_semantics_version = "wrong.checker@999".into();
                }
                let expected_pin = if c["expected_pin"].is_string() {
                    let mut p = fixture.relations.pins()[0].clone();
                    if c["expected_pin"] == "family" {
                        p.schema = "wrong.family@1".into();
                    } else {
                        p.revision = "different".into();
                    }
                    Some(p)
                } else {
                    None
                };
                let receipt = inspect_relation_metadata(
                    slot,
                    &context,
                    if c["association"] == "split" {
                        &separate
                    } else {
                        &context
                    },
                    &supplied,
                    expected_pin.as_ref(),
                )
                .unwrap();
                assert!(std::ptr::eq(receipt.context(), &raw const context));
                assert!(!receipt.can_issue_capability());
                let mut proposed = receipt.metadata().clone();
                if c["mode"] == "proposal" {
                    let mut at = &mut proposed;
                    let path = c["mutation"]["path"].as_array().unwrap();
                    for key in &path[..path.len() - 1] {
                        at = &mut at[key.as_str().unwrap()];
                    }
                    let key = path.last().unwrap().as_str().unwrap();
                    if c["mutation"]["op"] == "remove" {
                        at.as_object_mut().unwrap().remove(key);
                    } else {
                        at[key] = c["mutation"]["value"].clone();
                    }
                }
                let failure = admit_relation_observation_metadata(&proposed, &receipt)
                    .err()
                    .map(|x| format!("{x:?}"));
                json!({"computed":receipt.metadata(),"proposal_failure":failure,"slot_complete":receipt.slot_complete(),"raw_capture_complete":false,"positive_root_export":false})
            };
            let want = &expected["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|x| x["case_id"] == c["case_id"])
                .unwrap()["expected"];
            if &output != want {
                std::fs::write(
                    repo_root().join("target/phase-04c1b4c4r3c2r3g2-dev-mismatch-actual.json"),
                    serde_json::to_vec_pretty(&output).unwrap(),
                )
                .unwrap();
                std::fs::write(
                    repo_root().join("target/phase-04c1b4c4r3c2r3g2-dev-mismatch-expected.json"),
                    serde_json::to_vec_pretty(want).unwrap(),
                )
                .unwrap();
                panic!(
                    "independent relation metadata mismatch {} (full values retained in target)",
                    c["case_id"]
                );
            }
            actual.push(json!({"case_id":c["case_id"],"actual":output}));
        }
        std::fs::write(
            repo_root().join("target/phase-04c1b4c4r3c2r3g2-admission-actual.json"),
            serde_json::to_vec_pretty(&actual).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn equal_native_sources_do_not_recover_construction_schema() {
        let mut actual = Vec::new();
        for slot in [S::EffectResource, S::RecoveryUsage, S::FlowExecution] {
            let a = relation_modules(slot, "A");
            let b = relation_modules(slot, "B");
            let left = source_relations(slot, false, &a);
            let right = source_relations(slot, false, &b);
            assert_eq!(left, right);
            let id = format!("{}/{}", slot.family(), slot.root_type());
            let ad = a.type_by_identity(&id).unwrap().digest().unwrap();
            let bd = b.type_by_identity(&id).unwrap().digest().unwrap();
            assert_ne!(ad, bd);
            // Real malformed pre-context inputs cannot be assembled into the
            // private unique-key native module set; ambiguity is not invented.
            let minimal = format!(
                "afidl 1; schema {} ; record Probe {{ 1 value: string; }}",
                slot.family().replace('@', " @")
            );
            assert!(
                compile_module_set(&[("one".into(), minimal.clone()), ("two".into(), minimal)])
                    .is_err()
            );
            actual.push(json!({"case_id":format!("g01-relation-schema-staging/native-equality-{}",slot.name()),"native_equal":true,"schema_a":{"algorithm":"sha256","domain":"type-schema","hex":ad.hex()},"schema_b":{"algorithm":"sha256","domain":"type-schema","hex":bd.hex()},"ambiguous_native_context_rejected":true,"construction_history_recovered":false,"positive_root_export":false}));
        }
        std::fs::write(
            repo_root().join("target/phase-04c1b4c4r3c2r3g2-equality-actual.json"),
            serde_json::to_vec_pretty(&actual).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn same_actual_context_requires_every_some_slot_to_validate() {
        let expected = super::read("context-expected.json");
        let compiler = current_compiler_semantics_identity();
        let mut actual = Vec::new();
        for c in super::read("context-cases.json")["cases"]
            .as_array()
            .unwrap()
        {
            let mode = c["mode"].as_str().unwrap();
            let construction = relation_modules(S::EffectResource, "A");
            let (slot, profile) = match mode {
                "recovery-missing" | "false-complete" => (S::RecoveryUsage, "missing_module"),
                "flow-invalid" => (S::FlowExecution, "invalid"),
                _ => (S::EffectResource, "A"),
            };
            let mut fixture = FixtureContext::new();
            fixture.modules = relation_modules(slot, profile);
            fixture.relations = RelationSnapshots::default();
            if mode != "all-none" {
                fixture.relations.effect_resource =
                    source_relations(S::EffectResource, false, &construction).effect_resource;
                fixture.relations.recovery_usage =
                    source_relations(S::RecoveryUsage, false, &construction).recovery_usage;
                fixture.relations.flow_execution =
                    source_relations(S::FlowExecution, false, &construction).flow_execution;
            }
            let context = fixture.view();
            let split = fixture.view();
            let mut input = compiler.clone();
            if mode == "wrong-compiler" {
                input.analysis_semantics_version = "wrong.checker@999".into();
            }
            let receipt = inspect_relation_context_metadata(
                &context,
                if mode == "split" { &split } else { &context },
                &input,
            )
            .unwrap();
            assert!(std::ptr::eq(receipt.context(), &raw const context));
            assert!(!receipt.can_issue_capability());
            let mut proposed = receipt.metadata().clone();
            if mode == "missing-observation" {
                proposed.as_object_mut().unwrap().remove("recovery_usage");
            }
            if mode == "false-complete" {
                proposed["relation_partition_complete"] = json!(true);
            }
            let failure = admit_relation_context_observation_metadata(&proposed, &receipt)
                .err()
                .map(|e| format!("{e:?}"));
            let value = json!({"computed":receipt.metadata(),"proposal_failure":failure,"positive_root_export":false});
            let want = &expected["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|v| v["case_id"] == c["case_id"])
                .unwrap()["expected"];
            assert_eq!(&value, want, "{}", c["case_id"]);
            actual.push(json!({"case_id":c["case_id"],"actual":value}));
        }
        std::fs::write(
            repo_root().join("target/phase-04c1b4c4r3c2r3g2-context-actual.json"),
            serde_json::to_vec_pretty(&actual).unwrap(),
        )
        .unwrap();
    }
}
