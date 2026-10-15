# Native guard / current model admission reproduction

Diagnostic only;no facts,seeds,origins or positive capabilities are created. It includes the existing unchanged test fixture source and calls real native APIs,then asks the actual current registered ModelType shape API. There is no Rust Actual→Expected generator.

Local execution:

```powershell
cargo run --offline --quiet --manifest-path target/phase-04c1b4c4r3-probe/Cargo.toml --target-dir target
python tools/audit-g01-t3-witness-admission.py target/phase-04c1b4c4r3-probe-final.log
```

Cargo diagnostic exits0. Read-only audit intentionally exits2 for the reviewed blocked binding decision;it is not a failed Rust regression or an empty successful capture. Full ignored logs are retained. Recreate the following two ignored files if needed;the source fixture include is pinned to the current Windows workspace and unchanged repository test helper. The fixture's complete13-field declaration and explicit universes are the existing recovery_contract_with_guards/recovery_contract_with_usage helpers;exact canonical input bytes are embedded as Actual in the audit JSON.

## Manifest

```toml
[package]
name = "g01-r3-witness-admission-diagnostic"
version = "0.0.0"
edition = "2024"

[workspace]

[dependencies]
af-contract = { path = "E:/Projects/agent/crates/af-contract" }
af-computation = { path = "E:/Projects/agent/crates/af-computation" }
af-core = { path = "E:/Projects/agent/crates/af-core" }
af-idl = { path = "E:/Projects/agent/crates/af-idl" }
af-semantic = { path = "E:/Projects/agent/crates/af-semantic" }
af-effect = { path = "E:/Projects/agent/crates/af-effect" }
af-failure = { path = "E:/Projects/agent/crates/af-failure" }
af-resource = { path = "E:/Projects/agent/crates/af-resource" }
af-flow = { path = "E:/Projects/agent/crates/af-flow" }
af-usage = { path = "E:/Projects/agent/crates/af-usage" }
af-recovery = { path = "E:/Projects/agent/crates/af-recovery" }
af-execution = { path = "E:/Projects/agent/crates/af-execution" }
serde_json = "1"
sha2 = "0.10"
hex = "0.4"
```

## Diagnostic source

The version below is the final retained probe source,including historical-owner controls. It is test evidence only;include paths and source locations are never semantic Origin.

```rust
//! Diagnostic only. No seeds, origins, normalizer or Checked factory.
#[allow(dead_code, unused_imports)]
mod native {
    include!("E:/Projects/agent/crates/af-contract/tests/component_contract_conformance.rs");

    pub fn run() -> serde_json::Value {
        let mut fixture = FixtureContext::new();
        let input_type = guard_input_type(&mut fixture, "GuardInt64", Primitive::Int64);
        let contract = recovery_contract_with_guards(
            &mut fixture,
            EffectGuard::Unconditional,
            EffectGuard::InputEquals(EffectGuardInputEquals {
                port_no: 1,
                expected: PrimitiveConstant::Int64(0),
            }),
            vec![(1, input_type)],
        );
        let context = fixture.view();
        let canonical = af_contract::canonicalize_contract(&contract, &context).unwrap();
        let bytes = canonical.ace1_bytes(&context).unwrap();
        let report = analyze_contract(&contract, &context);
        let relation = report.action_guard_compatibility.first().unwrap();
        let witness = relation.countermodel().unwrap();
        assert_eq!(relation.verdict(), af_contract::ProofVerdict::Violation);
        assert_eq!(relation.basis(), af_contract::GuardCompatibilityBasis::TypedCountermodel);
        assert_eq!(witness.related_rule_id(), "AF-CONTRACT-0022");
        assert_eq!(witness.source_contract(), &contract);
        assert_eq!(witness.assignments().len(), 1);
        assert_eq!(witness.assignments()[0].value(), &PrimitiveConstant::Int64(1));
        assert!(check_contract(&contract, &context).is_err());
        let mut control = contract.clone();
        let EffectRow::Closed(entries) = &mut control.effects.row else { unreachable!() };
        entries[1].guard = EffectGuard::Unconditional;
        assert!(check_contract(&control, &context).is_ok());
        serde_json::json!({
            "kind":"native admission diagnostic;NOT T3 Capture Conformance",
            "input_canonical_ace1_hex":hex::encode(bytes),
            "canonical_validation":"passed",
            "full_check":"rejected_at_contract_validation",
            "guard_verdict":"Violation",
            "guard_basis":"TypedCountermodel",
            "forward_guard":"unconditional",
            "action_guard":{"port_no":1,"expected_int64":0},
            "countermodel":{"assignment_port_no":witness.assignments()[0].port_no(),
                "assignment_value_int64":1,"assignment_schema":witness.assignments()[0].type_ref().specification_id,
                "assignment_type":witness.assignments()[0].type_ref().type_name,
                "assignment_schema_digest":witness.assignments()[0].type_ref().schema_digest.hex,
                "forward_entry_ordinal":witness.forward_entry_ordinal(),
                "action_entry_ordinal":witness.action_entry_ordinal(),
                "reachability":"WellTypedComponentInputDomain","rule":witness.related_rule_id()},
            "control_unconditional_action":"Checked",
            "caller_ir_supplied":false,"root_node_supplied":false,"source_node_supplied":false
        })
    }
}
fn main() {
    let mut output = native::run();
    let names = ["af-contract::ActionGuardCompatibility", "af-contract::GuardCompatibilityBasis",
        "af-contract::TypedGuardCountermodel", "af-contract::GuardInputAssignment",
        "af-contract::GuardCountermodelReachability"];
    let model = af_computation::registered_model_catalogue().unwrap();
    let mut rows = Vec::new();
    for name in names {
        assert!(model["inherited_registered_model"]["semantic_model_declarations"].get(name).is_some());
        assert!(model["types"].get(name).is_none());
        let claim = serde_json::json!({"owner":af_computation::SUCCESSOR_MODEL_ID,"name":name});
        let failure = af_computation::validate_registered_model_shape("ModelType", &claim).unwrap_err();
        assert_eq!(failure,af_computation::RegisteredModelFailure::UnsupportedModel);
        let old_claim = serde_json::json!({"owner":"agentflow.computation.proof-report.model@1","name":name});
        let old_failure = af_computation::validate_registered_model_shape("ModelType", &old_claim).unwrap_err();
        assert_eq!(old_failure,af_computation::RegisteredModelFailure::UnsupportedModel);
        rows.push(serde_json::json!({"name":name,"historical_native_snapshot":true,
            "active_materialized_descriptor":false,"actual_model_type_admission":"UnsupportedModel",
            "historical_owner_actual_admission":"UnsupportedModel"}));
    }
    let known_current = serde_json::json!({"owner":af_computation::SUCCESSOR_MODEL_ID,"name":"Scope"});
    assert!(af_computation::validate_registered_model_shape("ModelType", &known_current).is_ok());
    let known_historical = serde_json::json!({"owner":"agentflow.computation.proof-report.model@1","name":"Scope"});
    assert_eq!(af_computation::validate_registered_model_shape("ModelType", &known_historical),Err(af_computation::RegisteredModelFailure::UnsupportedModel));
    output["owner_controls"] = serde_json::json!({"current_owner_known_scope":"accepted_metadata_only",
        "historical_owner_known_scope":"UnsupportedModel","interpretation":"API rejection alone is NOT proof that the historical declared type is unregistered;explicit complete binding must be reviewed"});
    output["model_admission"] = serde_json::json!(rows);
    output["positive_capability_allowed"] = serde_json::json!(af_computation::current_registered_analysis_edition().can_issue_capability());
    assert_eq!(output["positive_capability_allowed"],false);
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
```
