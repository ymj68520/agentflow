//! Outcome-C source-admission diagnostic, not a typed Raw fact producer.
#[allow(dead_code, unused_imports)]
mod fixtures {
    include!("../../af-contract/tests/component_contract_conformance.rs");

    #[test]
    fn independent_equal_documents_have_no_registered_occurrence_discriminator() {
        use af_computation::{
            NativeRawDocument, RawAdmissionFailure, admit_native_raw_source,
            current_compiler_semantics_identity,
        };
        use serde_json::{Value, json};
        use std::io::Write;
        use std::process::{Command, Stdio};

        let audit = repo_root().join("tools/oracle/raw_document_identity_c2.py");
        let vector = Command::new("python")
            .env("PYTHONUTF8", "1")
            .arg(&audit)
            .arg("--vector")
            .output()
            .unwrap();
        assert!(vector.status.success());
        let vector: Value = serde_json::from_slice(&vector.stdout).unwrap();
        // Real, separately owned, two-entry documents supplied before admission.
        // No collector reconstructs a one-entry source from the checked candidate.
        let first: Vec<ComponentContract> =
            serde_json::from_value(vector["native_declarations"].clone()).unwrap();
        let second: Vec<ComponentContract> =
            serde_json::from_value(vector["native_declarations"].clone()).unwrap();
        let document_a = NativeRawDocument {
            declarations: &first,
        };
        let document_b = NativeRawDocument {
            declarations: &second,
        };
        assert_eq!(first.len(), 2);
        assert_eq!(first, second);
        let distinct_documents = !std::ptr::eq(&raw const document_a, &raw const document_b);
        let distinct_storage = !std::ptr::eq(&raw const first[0], &raw const second[0]);
        assert!(distinct_documents);
        assert!(distinct_storage);

        let mut fixture = FixtureContext::new();
        let _ = guard_input_type(&mut fixture, "CaptureInput", Primitive::Int64);
        let context = fixture.view();
        let equal_other_context = fixture.view();
        let compiler = current_compiler_semantics_identity();
        // Exhaustive destructuring prevents silently omitting a native member.
        let StaticContractProofContext {
            modules: _,
            schema_universe: _,
            effects: _,
            resources: _,
            flow: _,
            usage_meters: _,
            recovery: _,
            execution: _,
            relations: _,
        } = &context;
        let mut observations = Vec::new();
        for (document, candidate) in [(&document_a, &first[0]), (&document_b, &second[0])] {
            let admitted = admit_native_raw_source(
                candidate,
                Some(document),
                0,
                &context,
                &context,
                &compiler,
            );
            assert_eq!(admitted, Ok(()));
            assert_eq!(
                admit_native_raw_source(candidate, None, 0, &context, &context, &compiler),
                Err(RawAdmissionFailure::MissingDeclarativeOrigin)
            );
            assert_eq!(
                admit_native_raw_source(
                    candidate,
                    Some(document),
                    1,
                    &context,
                    &context,
                    &compiler
                ),
                Err(RawAdmissionFailure::InvalidSourceAssociation)
            );
            assert_eq!(
                admit_native_raw_source(
                    candidate,
                    Some(document),
                    0,
                    &context,
                    &equal_other_context,
                    &compiler
                ),
                Err(RawAdmissionFailure::InvalidContextBinding)
            );
            let error = af_contract::canonicalize_contract(candidate, &context).unwrap_err();
            assert_eq!(error.rule_id, "AF-CONTRACT-0004");
            assert_eq!(error.code, "invalid_or_duplicate_port");
            let checked = check_contract(candidate, &context).is_ok();
            assert!(!checked);
            let report = af_contract::analyze_contract(candidate, &context);
            assert!(report.action_guard_compatibility.is_empty());
            observations.push(json!({
                "retained_complete_native_input": vector["native_declarations"],
                "retained_selected_candidate_input": vector["native_declarations"][0],
                "all_typed_declarations_equal": document.declarations == first,
                "typed_candidate_equal": candidate == &first[0],
                "selected_entry": 0,
                "admission": "Ok(unit)",
                "missing_document": "MissingDeclarativeOrigin",
                "wrong_entry": "InvalidSourceAssociation",
                "equal_content_other_context": "InvalidContextBinding",
                "physical": "not_supplied",
                "canonical_error": {"rule_id":error.rule_id,"code":error.code},
                "checked":checked,"native_guard_records":report.action_guard_compatibility.len()
            }));
        }
        // Object non-aliasing is diagnostic only. Never serialize an address,
        // use it as Origin, or pretend these projections are registered facts.
        let actual = json!({
            "case_id":"c2-binding/independent-identical-native-documents",
            "distinct_document_objects":distinct_documents,
            "distinct_declaration_storage":distinct_storage,
            "same_actual_context_for_both_roles_and_documents":true,
            "same_compiler_object_for_both_documents":true,
            "actual_context_fields":["compiled_modules","type_schemas","effect_catalogs","resource_types","flow_catalogs","usage_meters","recovery_operations","execution_universes","relation_snapshots"],
            "observations":observations,
            "typed_raw_facts_produced":0,
            "root_export_capability_issued":false
        });
        let payload = serde_json::to_vec_pretty(&actual).unwrap();
        std::fs::write(
            repo_root().join("target/phase-04c1b4c4r3c2-document-actual.json"),
            &payload,
        )
        .unwrap();
        for (arguments, exit_code) in [(vec![], 2), (vec!["--report"], 0)] {
            let mut process = Command::new("python")
                .env("PYTHONUTF8", "1")
                .arg(&audit)
                .args(arguments)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            process.stdin.take().unwrap().write_all(&payload).unwrap();
            let result = process.wait_with_output().unwrap();
            assert_eq!(
                result.status.code(),
                Some(exit_code),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            let report: Value = serde_json::from_slice(&result.stdout).unwrap();
            assert_eq!(report["finding_id"], "C2-GAP-01");
            assert_eq!(report["full_raw_seed_conformance"], false);
            std::fs::write(
                repo_root().join("target/phase-04c1b4c4r3c2-document-audit.json"),
                result.stdout,
            )
            .unwrap();
        }
    }
}
