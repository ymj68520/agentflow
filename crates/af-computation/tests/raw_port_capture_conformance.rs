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
    fn relation_schema_pin_binding_requires_formal_adjudication() {
        use af_contract::{ProofVerdict, RelationCatalogSnapshot, RelationSnapshots};
        fn sources(dir: &std::path::Path, out: &mut Vec<(String, String)>) {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    sources(&path, out);
                } else if path.extension().is_some_and(|ext| ext == "afidl") {
                    out.push((
                        path.to_string_lossy().into_owned(),
                        std::fs::read_to_string(path).unwrap(),
                    ));
                }
            }
        }
        let root = repo_root();
        let mut inputs = Vec::new();
        sources(&root.join("spec/idl"), &mut inputs);
        inputs.sort_by(|a, b| a.0.cmp(&b.0));
        let original = af_idl::compile_module_set(&inputs).unwrap();
        let old_member = "2 supported_pairs: list<SupportedEffectResourcePair>;";
        let new_member = "2 supported_pairs: list<string>;";
        let mut changed_inputs = inputs.clone();
        let relation_source = changed_inputs
            .iter_mut()
            .find(|(path, _)| path.ends_with("effect-resource-compatibility.afidl"))
            .unwrap();
        assert_eq!(relation_source.1.matches(old_member).count(), 1);
        let original_relation_source = relation_source.1.clone();
        relation_source.1 = relation_source.1.replace(old_member, new_member);
        let changed_relation_source = relation_source.1.clone();
        let changed = af_idl::compile_module_set(&changed_inputs).unwrap();
        let family = "agentflow.effect.resource.compatibility@1";
        let type_name = "EffectResourceCompatibilityCatalog";
        let schema_a = original.module(family).unwrap().by_name(type_name).unwrap();
        let schema_b = changed.module(family).unwrap().by_name(type_name).unwrap();
        assert_ne!(schema_a, schema_b);
        assert_ne!(schema_a.digest().unwrap(), schema_b.digest().unwrap());
        let a = RelationCatalogSnapshot::new(
            "binding-probe",
            EffectResourceCompatibilityCatalog::default(),
            &original,
        )
        .unwrap();
        let b = RelationCatalogSnapshot::new(
            "binding-probe",
            EffectResourceCompatibilityCatalog::default(),
            &changed,
        )
        .unwrap();
        assert_eq!(
            a, b,
            "real snapshots do not retain construction schema definition/pin"
        );
        assert_eq!(a.canonical_bytes(), b.canonical_bytes());
        assert_eq!(a.pin(), b.pin());
        let native_projection = json!({"pin":{"schema":a.pin().schema,"revision":a.pin().revision,
            "canonical_content_checksum":{"algorithm":a.pin().canonical_content_checksum.algorithm,"hex":a.pin().canonical_content_checksum.hex}},
            "canonical_content":a.canonical_bytes()});
        let shape_failure =
            validate_registered_model_shape("RelationSnapshotContent", &native_projection)
                .unwrap_err();
        assert_eq!(shape_failure, RegisteredModelFailure::UnsupportedModel);
        let without_relation: Vec<_> = inputs
            .iter()
            .filter(|(path, _)| !path.ends_with("effect-resource-compatibility.afidl"))
            .cloned()
            .collect();
        let absent_modules = af_idl::compile_module_set(&without_relation).unwrap();
        assert!(absent_modules.module(family).is_none());
        let generated = std::process::Command::new("python")
            .env("PYTHONUTF8", "1")
            .args(["tools/oracle/raw_port_capture.py", "--vectors"])
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(generated.status.success());
        let vectors: Vec<Value> = serde_json::from_slice(&generated.stdout).unwrap();
        let input = vectors
            .iter()
            .find(|v| v["case_id"] == "c2r1/declaration_bundle/input-zero")
            .unwrap();
        let package = &input["package"];
        let owner = &package["owners"][0]["anchor"];
        let doc = document(package, 0, 0);
        let path = [RawDocumentMember::Entry(0)];
        let request = DocumentOccurrenceMetadataRequest {
            source_package: Some(package),
            source_owner: Some(owner),
            entry_ordinal: Some(0),
            actual_document: doc,
            candidate: &input["candidate"],
            internal_path: &path,
            physical: OccurrencePhysicalMetadata::NotSupplied,
        };
        let mut left = FixtureContext::new();
        left.modules = original.clone();
        left.relations = RelationSnapshots {
            effect_resource: Some(a.clone()),
            ..RelationSnapshots::default()
        };
        let mut right = FixtureContext::new();
        right.modules = original.clone();
        right.relations = RelationSnapshots {
            effect_resource: Some(b),
            ..RelationSnapshots::default()
        };
        let mut absent = FixtureContext::new();
        absent.modules = absent_modules;
        absent.schemas = TypeSchemaUniverse::new();
        absent.resources = ResourceTypeUniverse::new();
        absent.relations = RelationSnapshots {
            effect_resource: Some(a.clone()),
            ..RelationSnapshots::default()
        };
        let lc = left.view();
        let rc = right.view();
        let ac = absent.view();
        let compiler = current_compiler_semantics_identity();
        let l = capture_raw_ports(&request, &lc, &lc, &compiler).unwrap();
        let r = capture_raw_ports(&request, &rc, &rc, &compiler).unwrap();
        let missing = capture_raw_ports(&request, &ac, &ac, &compiler).unwrap();
        assert!(l.same_subject(&r));
        assert!(!l.same_subject(&missing));
        for capture in [&l, &r, &missing] {
            capture.validate().unwrap();
            assert_eq!(capture.seeds().len(), 1);
            assert_eq!(capture.seeds()[0].status(), ProofVerdict::Violation);
            assert!(matches!(
                capture.seeds()[0].witness(),
                Some(RawRejectionWitness::Zero { .. })
            ));
            assert!(!capture.can_issue_root_export());
        }
        let source_names: Vec<_> = inputs
            .iter()
            .map(|(path, _)| {
                std::path::Path::new(path)
                    .strip_prefix(&root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect();
        let output = json!({"diagnostic_id":"c2r3/relation-snapshot-schema-pin",
            "family":family,"type_name":type_name,"source_files":source_names,
            "original_relation_source":original_relation_source,"changed_relation_source":changed_relation_source,
            "construction_schema_a":{"algorithm":"sha256","domain":"type-schema","hex":schema_a.digest().unwrap().hex()},
            "construction_schema_b":{"algorithm":"sha256","domain":"type-schema","hex":schema_b.digest().unwrap().hex()},
            "schema_a_ace1":hex::encode(schema_a.ace1_bytes().unwrap()),"schema_b_ace1":hex::encode(schema_b.ace1_bytes().unwrap()),
            "native_snapshots_equal":a==right.relations.effect_resource.clone().unwrap(),
            "native_pin_projection_failure":"UnsupportedModel",
            "native_pin":{"schema":a.pin().schema,"revision":a.pin().revision,"canonical_content_checksum":{"algorithm":a.pin().canonical_content_checksum.algorithm,"hex":a.pin().canonical_content_checksum.hex}},
            "canonical_content":a.canonical_bytes(),"same_current_context_values":l.same_subject(&r),
            "current_context_without_relation_schema_admitted":true,
            "raw_verdicts":[l.seeds()[0].status().as_str(),r.seeds()[0].status().as_str(),missing.seeds()[0].status().as_str()],
            "source_input":{"package":package,"candidate":input["candidate"],"owner":owner,"slot":0,"internal_entry":0},
            "full_fact_materialized":false,"positive_root_export":false});
        std::fs::write(
            root.join("target/phase-04c1b4c4r3c2r3-binding-actual.json"),
            serde_json::to_vec_pretty(&output).unwrap(),
        )
        .unwrap();
    }
    #[test]
    fn real_source_and_occurrence_input_sensitivity_subset() {
        let generated = std::process::Command::new("python")
            .env("PYTHONUTF8", "1")
            .args(["tools/oracle/raw_root_subset.py", "--sensitivity-vectors"])
            .current_dir(repo_root())
            .output()
            .unwrap();
        assert!(
            generated.status.success(),
            "{}",
            String::from_utf8_lossy(&generated.stderr)
        );
        let inputs: Vec<Value> = serde_json::from_slice(&generated.stdout).unwrap();
        let mut fixture = FixtureContext::new();
        let _ = guard_input_type(&mut fixture, "CaptureInput", Primitive::Int64);
        let context = fixture.view();
        let other_context = fixture.view();
        let compiler = current_compiler_semantics_identity();
        let mut actual = Vec::new();
        for input in inputs {
            let package = &input["package"];
            let slot = input["slot"].as_u64().unwrap();
            let owner_index = usize::try_from(input["owner"].as_u64().unwrap()).unwrap();
            let owner = &package["owners"][owner_index]["anchor"];
            let doc = document(package, owner_index, usize::try_from(slot).unwrap());
            let candidate = &input["candidate"];
            let path = [RawDocumentMember::Entry(
                input["path_entry"].as_u64().unwrap(),
            )];
            let request = DocumentOccurrenceMetadataRequest {
                source_package: Some(package),
                source_owner: Some(owner),
                entry_ordinal: Some(slot),
                actual_document: doc,
                candidate,
                internal_path: &path,
                physical: if input["physical"] == "matching" {
                    OccurrencePhysicalMetadata::Supplied {
                        source_owner: owner,
                        entry_ordinal: slot,
                        actual_document: doc,
                        value_path: &path,
                    }
                } else {
                    OccurrencePhysicalMetadata::NotSupplied
                },
            };
            let result = capture_raw_ports(
                &request,
                &context,
                if input["operation"] == "other_context" {
                    &other_context
                } else {
                    &context
                },
                &compiler,
            );
            let capture = match result {
                Ok(capture) => capture,
                Err(failure) => {
                    assert_eq!(input["operation"], "other_context");
                    assert_eq!(
                        failure,
                        DocumentOccurrenceAdmissionFailure::InvalidContextBinding
                    );
                    actual.push(json!({"case_id":input["case_id"],"failure":"InvalidContextBinding","events":[]}));
                    continue;
                }
            };
            capture.validate().unwrap();
            let typed: ComponentContract =
                serde_json::from_value(input["wire_candidate"].clone()).unwrap();
            assert_eq!(capture.subject().candidate, &typed);
            let events: Vec<_> = capture.seeds().iter().map(|seed| {
                let event = seed.event();
                json!({"occurrence":occurrence(event.occurrence),
                    "predecessor":event.predecessor.map(|(original_ordinal,port_no)|json!({"original_ordinal":original_ordinal,"port_no":port_no})),
                    "condition":match event.condition { PortCondition::PositiveFresh=>"positive_fresh",PortCondition::Zero=>"zero",PortCondition::Duplicate=>"duplicate" }})
            }).collect();
            let subject = capture.subject();
            assert!(std::ptr::eq(subject.context, &raw const context));
            let source_path: Vec<_> = subject
                .internal_path
                .iter()
                .map(|step| match step {
                    RawDocumentMember::Entry(index) => json!({"entry":index}),
                    RawDocumentMember::Field(_) => {
                        panic!("input corpus uses registered declaration entry paths")
                    }
                })
                .collect();
            actual.push(json!({"case_id":input["case_id"],"failure":null,"events":events,
                "source_package":subject.source_package,"owner":subject.source_owner,"slot":subject.entry_ordinal,
                "document":subject.document,"candidate":subject.candidate_metadata,"internal_path":source_path,
                "context_roles":capture.context_roles(),"root_export":capture.can_issue_root_export(),"complete_scope":capture.complete_scope()}));
        }
        let payload = serde_json::to_vec_pretty(&actual).unwrap();
        std::fs::write(
            repo_root().join("target/phase-04c1b4c4r3c2r2-input-sensitivity-actual.json"),
            &payload,
        )
        .unwrap();
        let mut child = std::process::Command::new("python")
            .env("PYTHONUTF8", "1")
            .args(["tools/oracle/raw_root_subset.py", "--sensitivity-report"])
            .current_dir(repo_root())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(&payload).unwrap();
        let checked = child.wait_with_output().unwrap();
        assert!(
            checked.status.success(),
            "{}",
            String::from_utf8_lossy(&checked.stderr)
        );
        std::fs::write(
            repo_root().join("target/phase-04c1b4c4r3c2r2-input-sensitivity-report.json"),
            checked.stdout,
        )
        .unwrap();
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
        let mut registered_rows = Vec::new();
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
            for seed in capture.seeds() {
                let fields = capture.registered_root_subset(seed).unwrap();
                let path = capture.registered_declared_path(seed).unwrap();
                let bound = capture.bound_rejection_witness(seed).unwrap();
                let witness_subject = bound.as_ref().map(|bound| {
                    assert!(std::ptr::eq(bound.subject.context, &raw const c));
                    assert!(std::ptr::eq(bound.subject.candidate, subject.candidate));
                    assert!(std::ptr::eq(bound.subject.compiler, subject.compiler));
                    assert_eq!(bound.witness, seed.witness().unwrap());
                    json!({"source_package":bound.subject.source_package,
                           "owner":bound.subject.source_owner,"slot":bound.subject.entry_ordinal,
                           "document":bound.subject.document,"candidate":bound.subject.candidate_metadata})
                });
                registered_rows.push(json!({"case_id":v["case_id"],
                    "occurrence":occurrence(seed.event().occurrence),
                    "registered_roots":fields,"registered_declared_path":path,
                    "witness_subject_input":witness_subject,
                    "complete_fact":false,"root_export":capture.can_issue_root_export()}));
            }
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
        let registered_payload = serde_json::to_vec_pretty(&registered_rows).unwrap();
        std::fs::write(
            repo_root().join("target/phase-04c1b4c4r3c2r2-root-subset-actual.json"),
            &registered_payload,
        )
        .unwrap();
        let mut child = std::process::Command::new("python")
            .env("PYTHONUTF8", "1")
            .args(["tools/oracle/raw_root_subset.py", "--report"])
            .current_dir(repo_root())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(&registered_payload)
            .unwrap();
        let checked = child.wait_with_output().unwrap();
        assert!(
            checked.status.success(),
            "{}",
            String::from_utf8_lossy(&checked.stderr)
        );
        std::fs::write(
            repo_root().join("target/phase-04c1b4c4r3c2r2-root-subset-report.json"),
            checked.stdout,
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
