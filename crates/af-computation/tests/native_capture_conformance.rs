//! Real producer smoke/regression checks; full independent T3 Oracle is a
//! separate gate. Included old fixtures are not counted as new Capture IDs.
#[path = "support/native_capture_projection.rs"]
mod native_projection;
#[allow(dead_code, unused_imports)]
mod fixtures {
    include!("../../af-contract/tests/component_contract_conformance.rs");
    use af_computation::native_capture::{
        NativeCaptureFailure, NativeCase, PendingScope, collect_native_guard,
    };
    use af_contract::guard_observation::{
        ObservedGuardBranch, ObservedGuardRole, ObservedPortResult, ObservedResolution,
        observe_canonical_guard,
    };
    #[test]
    fn actual_native_hooks_preserve_legacy_and_capture_both_checks() {
        for mode in [
            "positive",
            "typed_countermodel",
            "unit_unknown",
            "zero",
            "missing_port",
            "type_mismatch",
            "missing_usage",
            "missing_forward",
            "missing_action",
            "shared_bound_conflict",
            "overflow",
            "first_error_U_then_V",
            "schema_unresolved",
        ] {
            let mut f = FixtureContext::new();
            let primitive = if matches!(mode, "first_error_U_then_V" | "schema_unresolved") {
                Primitive::UInt32
            } else if mode == "unit_unknown" {
                Primitive::Unit
            } else if mode == "type_mismatch" {
                Primitive::Bool
            } else {
                Primitive::Int64
            };
            let ty = guard_input_type(&mut f, "GuardCaptureInput", primitive);
            let action_guard = if matches!(mode, "positive" | "shared_bound_conflict" | "overflow")
            {
                EffectGuard::Unconditional
            } else {
                EffectGuard::InputEquals(EffectGuardInputEquals {
                    port_no: if matches!(mode, "missing_port" | "first_error_U_then_V") {
                        2
                    } else {
                        1
                    },
                    expected: if mode == "unit_unknown" {
                        PrimitiveConstant::Unit
                    } else {
                        PrimitiveConstant::Int64(0)
                    },
                })
            };
            let forward_guard = if mode == "first_error_U_then_V" {
                EffectGuard::InputEquals(EffectGuardInputEquals {
                    port_no: 1,
                    expected: PrimitiveConstant::Int64(0),
                })
            } else {
                EffectGuard::Unconditional
            };
            let mut raw =
                recovery_contract_with_guards(&mut f, forward_guard, action_guard, vec![(1, ty)]);
            if mode == "zero" {
                raw.recovery_usage_bindings[0].attempts_per_activation =
                    OccurrenceBounds::finite(0, 0).unwrap();
            }
            if mode == "missing_usage" {
                raw.recovery_usage_bindings.clear();
            }
            if matches!(mode, "missing_forward" | "missing_action") {
                let EffectRow::Closed(es) = &mut raw.effects.row else {
                    unreachable!()
                };
                es.remove(usize::from(mode == "missing_action"));
            }
            if mode == "shared_bound_conflict" {
                let atom = raw.recovery_action_bindings[0]
                    .identity
                    .forward_effect
                    .clone();
                raw.recovery_action_bindings[0].identity.action_effect = atom;
                raw.recovery_usage_bindings[0].action =
                    raw.recovery_action_bindings[0].identity.clone();
                let EffectRow::Closed(es) = &mut raw.effects.row else {
                    unreachable!()
                };
                es.truncate(1);
                es[0].bounds = OccurrenceBounds::finite(2, 2).unwrap();
            }
            if mode == "overflow" {
                raw.recovery_usage_bindings[0].activations_per_forward =
                    OccurrenceBounds::finite(1, u64::MAX).unwrap();
            }
            let context = f.view();
            let observed = observe_canonical_guard(&raw, &context).unwrap();
            let old =
                af_contract::analyze_action_guard_compatibility(observed.declaration(), &context);
            assert_eq!(observed.runs().len(), 1);
            assert_eq!(observed.runs()[0].record.as_ref(), Some(&old[0]), "{mode}");
            let captured = collect_native_guard(&observed, &context).unwrap();
            assert_eq!(captured.scope(), PendingScope::PendingG02);
            assert!(!captured.can_issue_root_export());
            let eval = captured
                .seeds()
                .iter()
                .find(|s| s.origin().case() == NativeCase::GuardEvaluation)
                .unwrap();
            assert_eq!(eval.verdict(), old[0].verdict());
            if mode == "positive" {
                assert_eq!(captured.seeds().len(), 8);
                assert!(check_contract(&raw, &context).is_ok());
            }
            if mode == "typed_countermodel" {
                assert!(check_contract(&raw, &context).is_err());
                assert!(eval.witness().is_some());
                assert_eq!(
                    old[0].countermodel().unwrap().assignments()[0].value(),
                    &PrimitiveConstant::Int64(1)
                );
            }
            if mode == "zero" {
                assert_eq!(
                    observed.runs()[0].containment,
                    ObservedResolution::NotQueried
                );
                assert!(
                    !captured
                        .seeds()
                        .iter()
                        .any(|s| s.origin().case() == NativeCase::GuardContainment)
                );
            }
            if mode == "first_error_U_then_V" {
                let r = &observed.runs()[0];
                assert_eq!(r.branch, Some(ObservedGuardBranch::SchemaUnresolved));
                assert_eq!(r.primary_error, Some(ObservedGuardRole::Forward));
                assert!(matches!(
                    r.action_input,
                    ObservedResolution::Present(ObservedPortResult::MissingPort { .. })
                ));
                assert_eq!(eval.verdict(), af_contract::ProofVerdict::Unproven);
                assert!(eval.witness().is_none());
            }
            let other_context = f.view();
            assert!(matches!(
                collect_native_guard(&observed, &other_context),
                Err(NativeCaptureFailure::InvalidContextBinding)
            ));
        }
    }

    #[test]
    fn independent_native_oracle_reports_formal_bytes_contradiction() {
        use serde_json::{Value, json};
        use std::io::Write;
        use std::process::{Command, Stdio};
        let oracle = repo_root().join("tools/oracle/native_guard_capture.py");
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
        let mut actual = Vec::new();
        for input in inputs {
            let mut fixture = FixtureContext::new();
            // Catalog setup is explicit, unrelated to the independent body.
            let _ = recovery_contract_with_usage(&mut fixture, Some(12));
            let primitive =
                Primitive::from_keyword(input["schema_primitive"].as_str().unwrap()).unwrap();
            let _ = guard_input_type(&mut fixture, "CaptureInput", primitive);
            let raw: ComponentContract =
                serde_json::from_value(input["rust_input"].clone()).unwrap();
            let context = fixture.view();
            let result = match observe_canonical_guard(&raw, &context) {
                Ok(observed) => {
                    let capture = collect_native_guard(&observed, &context).unwrap();
                    assert!(!capture.complete_scope() && !capture.can_issue_root_export());
                    let legacy = af_contract::analyze_action_guard_compatibility(observed.declaration(), &context);
                    assert_eq!(legacy, observed.runs().iter().map(|r|r.record.clone().unwrap()).collect::<Vec<_>>());
                    super::native_projection::observation(&observed)
                }
                Err(af_contract::guard_observation::CanonicalGuardObservationError::CanonicalValidation(error)) => {
                    eprintln!("{} canonical rejection: {error:?}", input["case_id"]);
                    json!({"canonical_admission":false})
                },
                Err(af_contract::guard_observation::CanonicalGuardObservationError::CanonicalEncoding(error)) => {
                    eprintln!("{} canonical validation passed; encoding rejection: {error:?}", input["case_id"]);
                    json!({"canonical_admission":false})
                },
                Err(e) => panic!("unexpected capture rejection: {e:?}"),
            };
            actual.push(json!({"case_id":input["case_id"],"actual":result}));
        }
        let mut actual = json!(actual);
        super::native_projection::wire_numbers(&mut actual);
        std::fs::write(
            repo_root().join("target/phase-04c1b4c4r3-native-semantics-actual.json"),
            serde_json::to_vec_pretty(&actual).unwrap(),
        )
        .unwrap();
        let mut process = Command::new("python")
            .env("PYTHONUTF8", "1")
            .arg(&oracle)
            .arg("--report")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        process
            .stdin
            .take()
            .unwrap()
            .write_all(&serde_json::to_vec(&actual).unwrap())
            .unwrap();
        let result = process.wait_with_output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let report: Value = serde_json::from_slice(&result.stdout).unwrap();
        // This is a contradiction DETECTION regression, not T3 conformance.
        // Neither independently generated Expected nor the default failing
        // semantic gate is changed to accept the production rejection.
        assert_eq!(report["outcome"], "B");
        assert_eq!(report["native_semantic_cases_executed"], 53);
        assert_eq!(report["matched"], 51);
        assert_eq!(report["unexpected_mismatch"], 2);
        assert_eq!(
            report["mismatches"]
                .as_array()
                .unwrap()
                .iter()
                .map(|m| m["case_id"].as_str().unwrap())
                .collect::<Vec<_>>(),
            ["r3-ng/primitive-bytes-0", "r3-ng/primitive-bytes-1"]
        );
        assert!(!report["r3_ng_complete"].as_bool().unwrap());
        std::fs::write(
            repo_root().join("target/phase-04c1b4c4r3-native-semantics-report.json"),
            &result.stdout,
        )
        .unwrap();
        println!(
            "Native TRACE diagnostic: 53 executed, 51 matched, 2 formal contradictions; NOT T3 conformance"
        );
    }

    #[test]
    fn minimal_unchanged_effect_bytes_lowering_contradiction() {
        for (text, bytes) in [("", vec![]), ("AQ", vec![1])] {
            let mut f = FixtureContext::new();
            let mut raw = recovery_contract_with_guards(
                &mut f,
                EffectGuard::Unconditional,
                EffectGuard::InputEquals(EffectGuardInputEquals {
                    port_no: 1,
                    expected: PrimitiveConstant::Bytes(text.into()),
                }),
                vec![],
            );
            let EffectRow::Closed(entries) = &mut raw.effects.row else {
                unreachable!()
            };
            entries.remove(0); // one atom, one bytes guard; no K/collector call
            // Independent typed core encoding succeeds; effect structural
            // validation succeeds. The untouched effect lowering alone fails.
            assert_eq!(
                af_core::encode(&af_core::Shape::Bytes, &af_core::Value::Bytes(bytes)).unwrap(),
                serde_json::to_vec(text).unwrap()
            );
            assert!(raw.effects.validate(&f.effects).is_ok());
            let error = raw.effects.canonical_bytes(&f.modules).unwrap_err();
            assert!(error.to_string().contains("expected bytes, got string"));
        }
    }
}
