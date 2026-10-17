//! Independent input + actual-only local projection; never Scope encoding.
#[allow(dead_code, unused_imports)]
mod fixtures {
    include!("../../af-contract/tests/component_contract_conformance.rs");
    use af_computation::raw_capture::*;
    use af_computation::*;
    use af_contract::port_observation::*;
    use serde_json::{Value, json};
    use std::io::Write as _;
    fn direction(d: PortDirection) -> &'static str {
        match d {
            PortDirection::Input => "input",
            PortDirection::Output => "output",
        }
    }
    fn occurrence(o: PortOccurrence) -> Value {
        json!({"direction":direction(o.direction),"original_ordinal":o.original_ordinal,"sorted_ordinal":o.sorted_ordinal,"port_no":o.port_no})
    }
    fn stop(s: Option<PortPrefixStop>) -> Value {
        match s {
            None => Value::Null,
            Some(PortPrefixStop::ComponentIdentity) => json!({"component_identity":null}),
            Some(PortPrefixStop::PortRejection(o)) => json!({"port_rejection":occurrence(o)}),
            Some(PortPrefixStop::SchemaResolution(o)) => json!({"schema_resolution":occurrence(o)}),
        }
    }
    fn declared_path(p: &[af_computation::native_capture::NativeDeclaredStep]) -> Value {
        use af_computation::native_capture::NativeDeclaredStep as S;
        json!(
            p.iter()
                .map(|s| match s {
                    S::Field { owner, field } => json!({"field":{"owner":owner,"field":field}}),
                    S::Entry { index } => json!({"entry":{"index":index}}),
                })
                .collect::<Vec<_>>()
        )
    }
    fn document(p: &Value, owner: usize, slot: usize) -> &Value {
        let (tag, body) = p["owners"][owner]["container"]
            .as_object()
            .unwrap()
            .iter()
            .next()
            .unwrap();
        &body[match tag.as_str() {
            "declaration_bundle" => "declarations",
            "import_manifest" => "imports",
            "workflow_source_collection" => "sources",
            _ => return &Value::Null,
        }][slot]["document"]
    }
    #[test]
    fn actual_occurrence_bound_raw_ports() {
        let output = std::process::Command::new("python")
            .env("PYTHONUTF8", "1")
            .args(["tools/oracle/raw_port_capture.py", "--vectors"])
            .current_dir(repo_root())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let vectors: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
        let mut f = FixtureContext::new();
        let _ = guard_input_type(&mut f, "CaptureInput", Primitive::Int64);
        let c = f.view();
        let other = f.view();
        let compiler = current_compiler_semantics_identity();
        let mut rows = Vec::new();
        for v in vectors {
            println!("CAPTURE {}", v["case_id"]);
            let mut package = v["package"].clone();
            let op = v["operation"].as_str().unwrap();
            if op == "duplicate_owner" {
                let clone = package["owners"][0].clone();
                package["owners"].as_array_mut().unwrap().push(clone);
            }
            if op == "malformed_key" {
                package["owners"][0]["anchor"]["revision_key"] = json!([]);
            }
            if op == "joint_conflict" {
                let mut different = package.clone();
                different["owners"][0]["container"]["declaration_bundle"]["bundle_name"] =
                    json!([122]);
                let failure = check_joint_source_packages_metadata(&[&package, &different]).err();
                rows.push(
                    json!({"case_id":v["case_id"],"failure":format!("{:?}",failure.unwrap())}),
                );
                continue;
            }
            let doc = document(&package, 0, 0);
            let cloned = doc.clone();
            let mut mismatch = doc.clone();
            if op == "binding_conflict" {
                mismatch["declared_native_document"]["declarations"][0]["inputs"][0]["port_no"] =
                    json!(17);
            }
            let actual_document = if op == "clone_document" {
                &cloned
            } else if op == "binding_conflict" {
                &mismatch
            } else {
                doc
            };
            let mut owner = package["owners"][0]["anchor"].clone();
            if op == "wrong_owner" {
                owner["owner_key"] = json!([122]);
            }
            if op == "wrong_revision" {
                owner["revision_key"] = json!([122]);
            }
            let slot = if op == "wrong_slot" { 100 } else { 0 };
            let path = [RawDocumentMember::Entry(if op == "wrong_entry" {
                100
            } else {
                0
            })];
            let mut candidate = v["candidate"].clone();
            if op == "wrong_candidate" {
                candidate["inputs"][0]["port_no"] = json!(123);
            }
            let wrong_physical_path = [RawDocumentMember::Entry(9)];
            let physical = if (v["physical"] == "matching" && op != "physical_not_supplied")
                || op == "native_supplied"
            {
                OccurrencePhysicalMetadata::Supplied {
                    source_owner: &owner,
                    entry_ordinal: slot,
                    actual_document: doc,
                    value_path: if op == "physical_path_mismatch" {
                        &wrong_physical_path
                    } else {
                        &path
                    },
                }
            } else {
                OccurrencePhysicalMetadata::NotSupplied
            };
            let req = DocumentOccurrenceMetadataRequest {
                source_package: if op == "missing_package" {
                    None
                } else {
                    Some(&package)
                },
                source_owner: if op == "missing_owner" {
                    None
                } else {
                    Some(&owner)
                },
                entry_ordinal: if op == "missing_slot" {
                    None
                } else {
                    Some(slot)
                },
                actual_document,
                candidate: &candidate,
                internal_path: &path,
                physical,
            };
            let result = capture_raw_ports(
                &req,
                &c,
                if op == "other_context" { &other } else { &c },
                &compiler,
            );
            let capture = match result {
                Ok(x) => x,
                Err(e) => {
                    assert!(
                        !matches!(
                            op,
                            "capture" | "distinct_owner" | "distinct_slot" | "same_occurrence"
                        ),
                        "required Capture {} failed: {e:?}",
                        v["case_id"]
                    );
                    rows.push(json!({"case_id":v["case_id"],"failure":format!("{e:?}")}));
                    continue;
                }
            };
            capture.validate().unwrap();
            let typed: ComponentContract =
                serde_json::from_value(v["wire_candidate"].clone()).unwrap();
            assert_eq!(capture.subject().candidate, &typed, "{}", v["case_id"]);
            let old = af_contract::canonicalize_contract(&typed, &c);
            assert_eq!(old.is_ok(), capture.canonical_validated());
            let events:Vec<_>=capture.seeds().iter().map(|s| {
                assert!(capture.owns(s));let e=s.event();
                let basis=capture.basis(s).unwrap();let origin=capture.origin(s).unwrap();let target=capture.target(s).unwrap();
                assert!(std::ptr::eq(basis.subject.context,&raw const c) && std::ptr::eq(origin.subject.context,&raw const c) && std::ptr::eq(target.subject.context,&raw const c));
                assert_eq!(basis.subject.source_owner,origin.subject.source_owner);assert_eq!(origin.subject.source_owner,target.subject.source_owner);
                assert_eq!(basis.occurrence,e.occurrence);assert_eq!(target.direction,e.occurrence.direction);
                let rule=json!({"rule_id":basis.rule.rule_id,"normative_source":basis.rule.normative_source,"source_anchor":basis.rule.source_anchor,"short_statement":basis.rule.short_statement});
                let witness=match s.witness(){None=>Value::Null,Some(RawRejectionWitness::Zero{occurrence:o,raw_member_value,minimum})=>json!({"zero":{"occurrence":occurrence(o),"raw_member_value":raw_member_value,"minimum":minimum}}),Some(RawRejectionWitness::Duplicate{occurrence:o,previous_original_ordinal,previous_port_no})=>json!({"duplicate":{"occurrence":occurrence(o),"previous_original_ordinal":previous_original_ordinal,"previous_port_no":previous_port_no}})};
                json!({"occurrence":occurrence(e.occurrence),"predecessor":e.predecessor.map(|(original_ordinal,port_no)|json!({"original_ordinal":original_ordinal,"port_no":port_no})),"condition":match e.condition{PortCondition::PositiveFresh=>"positive_fresh",PortCondition::Zero=>"zero",PortCondition::Duplicate=>"duplicate"},"status":s.status().as_str(),"witness":witness,
                    "role":s.role(),"kind":s.kind(),"stage":s.stage(),"substage":s.substage(),"evidence_class":s.evidence_class(),"predicate":s.predicate(),"evaluated":s.evaluated(),"selected_error":basis.selected_error,"rule":rule,
                    "source_ordinal":origin.source_ordinal,"mapping_ordinal":origin.mapping_ordinal,"declared_path":declared_path(&origin.declared_path),"target_constraint":target.constraint,"target_module":target.schema.schema,"target_name":target.schema.name,"target_definition_ace1_hex":hex::encode(target.schema.ace1_bytes().unwrap()),"target_direction":direction(target.direction)})
            }).collect();
            let subject = capture.subject();
            assert!(std::ptr::eq(subject.context, &raw const c));
            let mut collector = RawLocalCollector::default();
            collector
                .insert(&capture, RawLocalObservation::CaptureFacts)
                .unwrap();
            let first_count = collector.facts().len();
            collector
                .insert(&capture, RawLocalObservation::CaptureFacts)
                .unwrap();
            assert_eq!(first_count, collector.facts().len());
            let diagnostic_observation = collector
                .insert(&capture, RawLocalObservation::ContractDiagnostic)
                .is_ok();
            assert_eq!(
                diagnostic_observation,
                matches!(capture.stop(), Some(PortPrefixStop::PortRejection(_)))
            );
            let observation_counts: Vec<_> = collector
                .facts()
                .iter()
                .map(|s| s.observations().len())
                .collect();
            let mut same_subject = Value::Null;
            let mut combined_count = Value::Null;
            if matches!(op, "distinct_owner" | "distinct_slot" | "same_occurrence") {
                let second_owner = usize::from(op == "distinct_owner");
                let second_slot = usize::from(op == "distinct_slot");
                let second_req = DocumentOccurrenceMetadataRequest {
                    source_package: Some(&package),
                    source_owner: Some(&package["owners"][second_owner]["anchor"]),
                    entry_ordinal: Some(second_slot as u64),
                    actual_document: document(&package, second_owner, second_slot),
                    candidate: &candidate,
                    internal_path: &path,
                    physical: OccurrencePhysicalMetadata::NotSupplied,
                };
                let second = capture_raw_ports(&second_req, &c, &c, &compiler).unwrap();
                same_subject = json!(capture.same_subject(&second));
                let mut joint = RawLocalCollector::default();
                joint
                    .insert(&capture, RawLocalObservation::CaptureFacts)
                    .unwrap();
                joint
                    .insert(&second, RawLocalObservation::CaptureFacts)
                    .unwrap();
                combined_count = json!(joint.facts().len());
                assert_eq!(capture.same_subject(&second), op == "same_occurrence");
                assert_eq!(
                    joint.facts().len(),
                    if op == "same_occurrence" {
                        first_count
                    } else {
                        first_count * 2
                    }
                );
            }
            rows.push(json!({"case_id":v["case_id"],"failure":null,"events":events,"stop":stop(capture.stop()),"canonical_validated":capture.canonical_validated(),"root_export":capture.can_issue_root_export(),"complete_scope":capture.complete_scope(),"source_package":subject.source_package,"owner":subject.source_owner,"slot":subject.entry_ordinal,"document":subject.document,"candidate":subject.candidate_metadata,"same_typed_thirteen_fields":true,"context_roles":capture.context_roles(),"observation_counts":observation_counts,"same_subject":same_subject,"combined_count":combined_count}));
        }
        let payload = serde_json::to_vec_pretty(&rows).unwrap();
        std::fs::write(
            repo_root().join("target/phase-04c1b4c4r3c2r1-raw-actual.json"),
            &payload,
        )
        .unwrap();
        for diagnostic in [true, false] {
            let mut command = std::process::Command::new("python");
            command
                .env("PYTHONUTF8", "1")
                .arg("tools/oracle/raw_port_capture.py")
                .current_dir(repo_root());
            if diagnostic {
                command.arg("--report");
            }
            let mut child = command
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap();
            child.stdin.take().unwrap().write_all(&payload).unwrap();
            let output = child.wait_with_output().unwrap();
            assert_eq!(
                output.status.code(),
                Some(if diagnostic { 0 } else { 2 }),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let report: Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(report["local_projection_conformance"], "PASS");
            assert_eq!(report["full_raw_seed_conformance"], "INCOMPLETE");
            assert_eq!(report["actual_port_capture_case_ids"], 59);
            assert_eq!(report["required_event_coverage"]["compared"], 125);
            if diagnostic {
                std::fs::write(
                    repo_root().join("target/phase-04c1b4c4r3c2r1-raw-report.json"),
                    &output.stdout,
                )
                .unwrap();
            }
        }
    }
}
