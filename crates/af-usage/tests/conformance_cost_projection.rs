//! Cost Projection conformance harness. Expected bytes, projection states and
//! proof verdicts come from the independent Python oracle.

use af_usage::{
    CostProof, CostUnprovenReason, CurrencyRef, CurrencyUniverse, MeasurementUnitRef, MoneyAmount,
    PriceTableSnapshot, StaticCostBound, StaticCostProjection, StaticUsageBound,
    StaticUsageEnvelope, UsageAggregation, UsageMeterRef, UsageMeterUniverse, UsagePhase,
    UsageUnitRef, canonicalize_price_table_snapshot, check_static_budget, cost_not_increased,
    project_cost,
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn read(relative: &str) -> String {
    std::fs::read_to_string(root().join(relative)).unwrap()
}

fn module_set() -> af_idl::CompiledModuleSet {
    af_idl::compile_module_set(&[
        (
            "semantic-core.afidl".into(),
            read("spec/idl/semantic-core.afidl"),
        ),
        (
            "usage-signature.afidl".into(),
            read("spec/idl/usage-signature.afidl"),
        ),
        (
            "cost-projection.afidl".into(),
            read("spec/idl/cost-projection.afidl"),
        ),
    ])
    .unwrap()
}

fn cases(subdir: &str) -> Vec<Value> {
    let mut paths: Vec<_> =
        std::fs::read_dir(root().join("spec/conformance/cost-projection").join(subdir))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
            .collect();
    paths.sort();
    paths
        .into_iter()
        .map(|path| serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap())
        .collect()
}

fn meters(value: &Value) -> UsageMeterUniverse {
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
        let aggregation = if meter["aggregation"] == "additive" {
            UsageAggregation::Additive
        } else {
            UsageAggregation::Unsupported
        };
        universe.recognize(&meter_ref, &unit, aggregation).unwrap();
    }
    universe
}

fn currencies(value: &Value) -> CurrencyUniverse {
    let mut universe = CurrencyUniverse::new();
    for item in value["currencies"].as_array().unwrap() {
        let currency = CurrencyRef {
            catalog: item["currency"]["catalog"].as_str().unwrap().to_string(),
            code: item["currency"]["code"].as_str().unwrap().to_string(),
        };
        let unit = MeasurementUnitRef::new(
            item["atomic_unit"]["catalog"].as_str().unwrap(),
            item["atomic_unit"]["name"].as_str().unwrap(),
        )
        .unwrap();
        universe
            .recognize(
                &currency,
                &unit,
                u8::try_from(item["display_scale"].as_u64().unwrap()).unwrap(),
            )
            .unwrap();
    }
    universe
}

fn snapshot(case: &Value, key: &str) -> PriceTableSnapshot {
    serde_json::from_value(case[key].clone()).unwrap()
}

fn usage_envelope(value: &Value) -> StaticUsageEnvelope {
    if value["global_unknown"].as_bool().unwrap() {
        return StaticUsageEnvelope::unknown();
    }
    let mut envelope = StaticUsageEnvelope::empty();
    for item in value["bounds"].as_array().unwrap() {
        let phase: UsagePhase = serde_json::from_value(item["phase"].clone()).unwrap();
        let meter: UsageMeterRef = serde_json::from_value(item["meter"].clone()).unwrap();
        let bound = match item["bound"]["kind"].as_str().unwrap() {
            "known_finite" => {
                StaticUsageBound::KnownFinite(item["bound"]["value"].as_u64().unwrap())
            }
            "known_unbounded" => StaticUsageBound::KnownUnbounded,
            "unknown" => StaticUsageBound::Unknown,
            kind => panic!("unknown usage bound {kind}"),
        };
        envelope
            .insert(af_usage::UsageCoordinate { phase, meter }, bound)
            .unwrap();
    }
    envelope
}

fn bound_json(bound: &StaticCostBound) -> Value {
    match bound {
        StaticCostBound::KnownFinite(amount) => {
            json!({"kind": "known_finite", "amount": amount.atomic_units})
        }
        StaticCostBound::KnownUnbounded { .. } => json!({"kind": "known_unbounded"}),
        StaticCostBound::Unproven(reason) => {
            json!({"kind": "unproven", "reason": reason.as_str()})
        }
    }
}

fn projection_json(projection: &StaticCostProjection) -> Value {
    json!({
        "currency": projection.currency,
        "forward": bound_json(&projection.forward),
        "recovery": bound_json(&projection.recovery),
        "total": bound_json(&projection.total),
    })
}

fn proof_json(proof: CostProof) -> Value {
    match proof {
        CostProof::Proven => json!({"outcome": "proven"}),
        CostProof::Violation => json!({"outcome": "violation"}),
        CostProof::Unproven(reason) => json!({
            "outcome": "unproven",
            "reason": reason.as_str(),
        }),
    }
}

fn assert_proof(case_id: &str, actual: CostProof, expected: &Value) {
    assert_eq!(proof_json(actual), *expected, "{case_id}: proof");
}

fn cost_bound_from_json(value: &Value, currency: &CurrencyRef) -> StaticCostBound {
    match value["kind"].as_str().unwrap() {
        "known_finite" => StaticCostBound::KnownFinite(MoneyAmount {
            currency: currency.clone(),
            atomic_units: value["amount"].as_u64().unwrap(),
        }),
        "known_unbounded" => StaticCostBound::KnownUnbounded {
            currency: currency.clone(),
        },
        "unproven" => StaticCostBound::Unproven(match value["reason"].as_str().unwrap() {
            "unknown_usage" => CostUnprovenReason::UnknownUsage,
            "missing_price_coverage" => CostUnprovenReason::MissingPriceCoverage,
            "currency_mismatch" => CostUnprovenReason::CurrencyMismatch,
            _ => CostUnprovenReason::DifferentPriceSnapshot,
        }),
        other => panic!("unknown cost bound {other}"),
    }
}

fn projection_from_json(value: &Value) -> StaticCostProjection {
    let currency: CurrencyRef = serde_json::from_value(value["currency"].clone()).unwrap();
    StaticCostProjection {
        currency: currency.clone(),
        forward: cost_bound_from_json(&value["forward"], &currency),
        recovery: cost_bound_from_json(&value["recovery"], &currency),
        total: cost_bound_from_json(&value["total"], &currency),
    }
}

#[test]
fn golden_and_pricing_cases_match_projection_and_ace1() {
    for subdir in ["golden", "pricing"] {
        for case in cases(subdir) {
            let meter_universe = meters(&case["catalogs"]);
            let currency_universe = currencies(&case["catalogs"]);
            let snapshot = snapshot(&case, "snapshot");
            let canonical =
                canonicalize_price_table_snapshot(&snapshot, &currency_universe, &meter_universe)
                    .unwrap();
            if let Some(frozen) = case.get("canonical_ace1_utf8") {
                let bytes = canonical
                    .canonical_bytes(&currency_universe, &meter_universe, &module_set())
                    .unwrap();
                assert_eq!(
                    String::from_utf8(bytes).unwrap(),
                    frozen.as_str().unwrap(),
                    "{}: canonical bytes",
                    case["case_id"]
                );
            }
            let projection = project_cost(
                &usage_envelope(&case["usage"]),
                &snapshot,
                &currency_universe,
                &meter_universe,
            )
            .unwrap();
            assert_eq!(
                projection_json(&projection),
                case["projection"],
                "{}",
                case["case_id"]
            );
        }
    }
}

#[test]
fn negative_cases_fail_closed() {
    for case in cases("negative") {
        let meter_universe = meters(&case["catalogs"]);
        let currency_universe = currencies(&case["catalogs"]);
        let result = match case["operation"].as_str().unwrap() {
            "canonicalize" => canonicalize_price_table_snapshot(
                &snapshot(&case, "snapshot"),
                &currency_universe,
                &meter_universe,
            )
            .map(|_| ()),
            "project" => {
                let result = project_cost(
                    &usage_envelope(&case["usage"]),
                    &snapshot(&case, "snapshot"),
                    &currency_universe,
                    &meter_universe,
                );
                match result {
                    Ok(projection) => {
                        assert!(matches!(projection.forward, StaticCostBound::Unproven(_)));
                        Err(af_usage::CostError::new(
                            af_usage::cost_kind::COST_PROJECTION_UNPROVEN,
                            "projection state is unproven",
                        ))
                    }
                    Err(error) => Err(error),
                }
            }
            other => panic!(
                "{}: unsupported negative operation {other}",
                case["case_id"]
            ),
        };
        let error = result.expect_err(&format!("{}: must fail", case["case_id"]));
        assert_eq!(error.kind(), case["expected_kind"].as_str().unwrap());
    }
}

#[test]
fn budget_cases_match_static_proofs() {
    for case in cases("budget") {
        if case["case_id"] == "cost/budget/cost-non-increase"
            || case["case_id"] == "cost/budget/different-price-separated"
        {
            let old = projection_from_json(&case["old_projection"]);
            let new = projection_from_json(&case["new_projection"]);
            let old_snapshot = snapshot(&case, "old_snapshot");
            let new_snapshot = snapshot(&case, "new_snapshot");
            assert_proof(
                case["case_id"].as_str().unwrap(),
                cost_not_increased(&new, &new_snapshot, &old, &old_snapshot),
                &case["expected"],
            );
            continue;
        }
        let meter_universe = meters(&case["catalogs"]);
        let currency_universe = currencies(&case["catalogs"]);
        let projection = project_cost(
            &usage_envelope(&case["usage"]),
            &snapshot(&case, "snapshot"),
            &currency_universe,
            &meter_universe,
        )
        .unwrap();
        let ceiling: af_usage::StaticCostCeiling =
            serde_json::from_value(case["ceiling"].clone()).unwrap();
        assert_proof(
            case["case_id"].as_str().unwrap(),
            check_static_budget(&projection, &ceiling, &currency_universe).unwrap(),
            &case["expected"],
        );
    }
}

#[test]
fn price_order_equivalence_has_identical_bytes() {
    for case in cases("equivalence") {
        let meter_universe = meters(&case["catalogs"]);
        let currency_universe = currencies(&case["catalogs"]);
        let mut reference = None;
        for member in case["members"].as_array().unwrap() {
            let snapshot: PriceTableSnapshot = serde_json::from_value(member.clone()).unwrap();
            let bytes = snapshot
                .canonical_bytes(&currency_universe, &meter_universe, &module_set())
                .unwrap();
            let text = String::from_utf8(bytes).unwrap();
            match reference.as_ref() {
                None => {
                    assert_eq!(text, case["canonical_ace1_utf8"]);
                    reference = Some(text);
                }
                Some(expected) => assert_eq!(expected, &text),
            }
        }
    }
}
