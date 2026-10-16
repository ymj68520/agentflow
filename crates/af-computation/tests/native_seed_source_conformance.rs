#[path = "support/native_capture_projection.rs"]
mod projection;
#[allow(dead_code, unused_imports)]
mod fixtures {
    include!("../../af-contract/tests/component_contract_conformance.rs");
    #[test]
    fn c1_native_source_admission_rejects_noncanonical_text() {
        use serde_json::{Value, json};
        use std::io::Write;
        use std::process::{Command, Stdio};
        let oracle = repo_root().join("tools/oracle/native_source_c1.py");
        let inputs = Command::new("python")
            .env("PYTHONUTF8", "1")
            .arg(&oracle)
            .arg("--vectors")
            .output()
            .unwrap();
        assert!(
            inputs.status.success(),
            "{}",
            String::from_utf8_lossy(&inputs.stderr)
        );
        let inputs: Vec<Value> = serde_json::from_slice(&inputs.stdout).unwrap();
        let mut f = FixtureContext::new();
        let _ = recovery_contract_with_usage(&mut f, Some(12));
        let _ = guard_input_type(&mut f, "CaptureInput", Primitive::String);
        let c = f.view();
        let mut rows = Vec::new();
        for input in inputs {
            let raw: ComponentContract =
                serde_json::from_value(input["rust_input"].clone()).unwrap();
            let observed =
                af_contract::guard_observation::observe_canonical_guard(&raw, &c).unwrap();
            let capture = af_computation::native_capture::collect_native_guard(&observed, &c);
            let EffectRow::Closed(entries) = &observed.declaration().effects.row else {
                unreachable!()
            };
            let guards = entries
                .iter()
                .map(|e| {
                    let EffectGuard::InputEquals(g) = &e.guard else {
                        unreachable!()
                    };
                    let PrimitiveConstant::String(value) = &g.expected else {
                        unreachable!()
                    };
                    json!({"input_equals":{"port_no":g.port_no,"expected":{"string":value}}})
                })
                .collect::<Vec<_>>();
            rows.push(json!({"case_id":input["case_id"],"stored_guards":guards,
                "observation":super::projection::observation(&observed),"context_same_object":observed.uses_context(&c),
                "capture_admitted":capture.is_ok(),
                "capture_failure":capture.as_ref().err().map(|e|match e {af_computation::native_capture::NativeCaptureFailure::InvalidFacts=>"invalid_facts",_=>"unexpected_capture_failure"}),
                "canonical_source_status":capture.as_ref().ok().map(|c|c.seeds()[0].verdict().as_str()),
                "whole_contract_checked":check_contract(&raw,&c).is_ok(),"root_export_capability":capture.as_ref().is_ok_and(af_computation::native_capture::NativeLocalCapture::can_issue_root_export)}));
        }
        let mut actual = json!(rows);
        super::projection::wire_numbers(&mut actual);
        let payload = serde_json::to_vec_pretty(&actual).unwrap();
        std::fs::write(
            repo_root().join("target/phase-04c1b4c4r3c1-native-source-actual.json"),
            &payload,
        )
        .unwrap();
        let mut p = Command::new("python")
            .env("PYTHONUTF8", "1")
            .arg(&oracle)
            .arg("--report")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        p.stdin.take().unwrap().write_all(&payload).unwrap();
        let result = p.wait_with_output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        std::fs::write(
            repo_root().join("target/phase-04c1b4c4r3c1-native-source-report.json"),
            &result.stdout,
        )
        .unwrap();
        let report: Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(report["unexpected_mismatch"], 0);
        println!(
            "source probe outcome={} matched={} mismatch={} equal_bytes={}",
            report["outcome"],
            report["matched"],
            report["unexpected_mismatch"],
            report["equal_canonical_bytes"]
        );
    }
    #[test]
    fn c1_direct_raw_native_validation_subject_binding_gap() {
        use serde_json::{Value, json};
        use std::io::Write;
        use std::process::{Command, Stdio};
        let oracle = repo_root().join("tools/oracle/native_raw_binding_c1.py");
        let input = Command::new("python")
            .env("PYTHONUTF8", "1")
            .arg(&oracle)
            .arg("--vector")
            .output()
            .unwrap();
        assert!(input.status.success());
        let input: Value = serde_json::from_slice(&input.stdout).unwrap();
        let mut f = FixtureContext::new();
        let _ = guard_input_type(&mut f, "CaptureInput", Primitive::Int64);
        let c = f.view();
        let raw: ComponentContract = serde_json::from_value(input["rust_input"].clone()).unwrap();
        let error = af_contract::canonicalize_contract(&raw, &c).unwrap_err();
        let report = af_contract::analyze_contract(&raw, &c);
        let checked = check_contract(&raw, &c).is_ok();
        // Exhaustive native context destructuring: no hidden resolver or IR.
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
        } = &c;
        let actual = json!({"case_id":input["case_id"],"canonical_validation":{"passed":false,"rule_id":error.rule_id,"code":error.code,"reference":error.reference,"detail":error.detail},
            "actual_context_fields":["compiled_modules","type_schemas","effect_catalogs","resource_types","flow_catalogs","usage_meters","recovery_operations","execution_universes","relation_snapshots"],
            "checked":checked,"native_observation_admitted":af_contract::guard_observation::observe_canonical_guard(&raw,&c).is_ok(),
            "schema_fact":{"verdict":report.dimensions[0].verdict.as_str(),"rule_id":report.dimensions[0].rule_id,"reference":report.dimensions[0].reference,"reason":report.dimensions[0].reason},
            "native_records":report.action_guard_compatibility.len(),"report_dimensions":report.dimensions.iter().map(|d|json!({"dimension":d.dimension,"verdict":d.verdict.as_str(),"rule_id":d.rule_id,"reference":d.reference,"reason":d.reason})).collect::<Vec<_>>()});
        let data = serde_json::to_vec_pretty(&actual).unwrap();
        std::fs::write(
            repo_root().join("target/phase-04c1b4c4r3c1-raw-binding-actual.json"),
            &data,
        )
        .unwrap();
        let mut p = Command::new("python")
            .env("PYTHONUTF8", "1")
            .arg(&oracle)
            .arg("--report")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        p.stdin.take().unwrap().write_all(&data).unwrap();
        let result = p.wait_with_output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let audit: Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(audit["finding_id"], "C1-GAP-01");
        assert_eq!(audit["subject_alternatives"].as_array().unwrap().len(), 5);
        std::fs::write(
            repo_root().join("target/phase-04c1b4c4r3c1-raw-binding-report.json"),
            &result.stdout,
        )
        .unwrap();
        println!(
            "C1-GAP-01: real raw rejection; nine-member native Context; all five registered Subject alternatives checked"
        );
    }
}
