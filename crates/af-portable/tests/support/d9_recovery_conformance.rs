use super::*;

#[test]
fn independent_recovery_full_fields_match_16_original_cases() {
    let contract = read_json(
        "spec/conformance/portable-root-export-revalidation/coverage/field-coverage.json",
    );
    let manifest = read_json(
        "spec/conformance/portable-root-export-revalidation/expected-full/d9-recovery/manifest.json",
    );
    let cases = manifest["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 16);
    let output = repo().join("target/conformance/d9-recovery-full");
    std::fs::create_dir_all(&output).unwrap();
    let mut results = Vec::new();
    for chunk in cases.chunks(3) {
        let handles=chunk.iter().map(|case| {
            let case=case.clone();let contract=contract.clone();let output=output.clone();
            std::thread::spawn(move || {
                let bytes=std::fs::read(repo().join(case["bundle_file"].as_str().unwrap())).unwrap();
                let analysis=PortableProofCodec::default().validate_supported(&bytes).unwrap().revalidate_root_export().unwrap();
                let actual=json!({"case_id":case["case_id"],"input_identity":{"canonical_bytes":bytes.len(),"canonical_sha256":hex::encode(sha2::Sha256::digest(&bytes))},"production_stage":"D10","context":analysis.inputs().context(),"d9_root":analysis.root_summary().typed_facts(),"d9_root_recovery":analysis.root_recovery().typed_facts()});
                let slug=case["case_id"].as_str().unwrap().rsplit('/').next().unwrap();
                std::fs::write(output.join(format!("{slug}.actual.json")),serde_json::to_vec_pretty(&actual).unwrap()).unwrap();
                let expected=read_json(case["expected_file"].as_str().unwrap());
                match compare(&expected["facts"],&actual,&contract) {
                    Ok((left,right))=>json!({"case_id":case["case_id"],"expected_required":left.required,"actual_required":right.required,"compared_required":left.required,"diagnostic_instances":left.diagnostic,"missing_expected":0,"missing_actual":0,"unexpected":0,"uncompared":0,"d9_full_field":"passed","d10_full_field":"Pending"}),
                    Err(error)=>json!({"case_id":case["case_id"],"d9_full_field":"failed","first_difference":error,"d10_full_field":"Pending"}),
                }
            })
        }).collect::<Vec<_>>();
        for handle in handles {
            results.push(handle.join().expect("Recovery differential worker"));
        }
    }
    let failures = results
        .iter()
        .filter(|row| row["d9_full_field"] != "passed")
        .collect::<Vec<_>>();
    std::fs::write(output.join("report.json"),serde_json::to_vec_pretty(&json!({"executed":results.len(),"fully_compared":results.len()-failures.len(),"skipped":0,"unexpected_mismatch":failures.len(),"cases":results})).unwrap()).unwrap();
    assert!(
        failures.is_empty(),
        "Recovery field differences: {failures:?}"
    );
}

#[test]
fn real_recovery_input_mutations_recompute_or_reject() {
    let contract = read_json(
        "spec/conformance/portable-root-export-revalidation/coverage/field-coverage.json",
    );
    let manifest = read_json(
        "spec/conformance/portable-root-export-revalidation/input-mutations/recovery/manifest.json",
    );
    let cases = manifest["cases"].as_array().unwrap();
    assert!(cases.len() >= 12);
    let output = repo().join("target/conformance/d9-recovery-input");
    std::fs::create_dir_all(&output).unwrap();
    let mut results = Vec::new();
    for chunk in cases.chunks(3) {
        let handles=chunk.iter().map(|case| {
            let case=case.clone();let contract=contract.clone();let output=output.clone();
            std::thread::spawn(move || {
                let bytes=std::fs::read(repo().join(case["bundle_file"].as_str().unwrap())).unwrap();
                let identity=json!({"canonical_bytes":bytes.len(),"canonical_sha256":hex::encode(sha2::Sha256::digest(&bytes))});
                let analysis=PortableProofCodec::default().validate_supported(&bytes).and_then(|supported|supported.revalidate_root_export());
                let actual=match analysis {
                    Ok(analysis)=>json!({"case_id":case["case_id"],"input_identity":identity,"production_stage":"D10","context":analysis.inputs().context(),"d9_root":analysis.root_summary().typed_facts(),"d9_root_recovery":analysis.root_recovery().typed_facts()}),
                    Err(error)=>json!({"case_id":case["case_id"],"input_identity":identity,"production_stage":format!("{:?}",error.stage),"error_family":error.family,"error_path":error.path,"d9_root":null,"capability_issued":false}),
                };
                std::fs::write(output.join(format!("{}.actual.json",case["vector_id"].as_str().unwrap())),serde_json::to_vec_pretty(&actual).unwrap()).unwrap();
                let expected=read_json(case["expected_file"].as_str().unwrap());
                let difference=if case["expected_stage"]=="D10" {compare(&expected["facts"],&actual,&contract).err()} else {
                    let defs=&contract["projection_definitions"];
                    let schema=&defs["EarlyRejection"];
                    validate(&expected["facts"],schema,defs,"$",0,&mut Count::default())
                        .and_then(|()|validate(&actual,schema,defs,"$",0,&mut Count::default())).err()
                        .or_else(||difference(&expected["facts"],&actual,"$"))
                };
                json!({"vector_id":case["vector_id"],"actual_stage":actual["production_stage"],"expected_stage":case["expected_stage"],"first_difference":difference,"result":if difference.is_none(){"passed"}else{"failed"}})
            })
        }).collect::<Vec<_>>();
        for handle in handles {
            results.push(handle.join().expect("Recovery input worker"));
        }
    }
    let failures = results
        .iter()
        .filter(|row| row["result"] != "passed")
        .collect::<Vec<_>>();
    std::fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(
            &json!({"executed":results.len(),"failures":failures.len(),"cases":results}),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(
        failures.is_empty(),
        "Recovery input differences: {failures:?}"
    );
}

#[test]
fn recovery_independent_mutations_and_both_side_omissions_are_detected() {
    let contract = read_json(
        "spec/conformance/portable-root-export-revalidation/coverage/field-coverage.json",
    );
    let manifest = read_json(
        "spec/conformance/portable-root-export-revalidation/coverage/recovery-comparator-mutations.json",
    );
    let rows = manifest["mutations"].as_array().unwrap();
    assert!(rows.len() >= 32);
    for row in rows {
        let path=row["expected_file"].as_str().map_or_else(||format!("spec/conformance/portable-root-export-revalidation/expected-full/d9-recovery/{}.json",row["case_slug"].as_str().unwrap()),str::to_owned);
        let expected = read_json(&path)["facts"].clone();
        let mut actual = expected.clone();
        replace_path(
            &mut actual,
            row["path"].as_array().unwrap(),
            row["replacement"].clone(),
        );
        let error = compare(&expected, &actual, &contract)
            .expect_err("independent Recovery mutation must fail");
        let token = row["path"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()
            .as_str()
            .unwrap();
        assert!(
            error.contains(token),
            "{} exact first difference for {token}: {error}",
            row["mutation_id"]
        );
    }
    let mut both=read_json("spec/conformance/portable-root-export-revalidation/expected-full/d9-recovery/d9-recovery-outer-retry-3.json")["facts"].clone();
    both["d9_root_recovery"]["actions"]["actions"][0]
        .as_object_mut()
        .unwrap()
        .remove("root_bound");
    assert!(
        compare(&both, &both, &contract)
            .unwrap_err()
            .contains("root_bound: missing required field")
    );
    both["d9_root_recovery"]["actions"]["actions"][0]["unregistered_recovery_fact"] = json!(1);
    assert!(compare(&both, &both, &contract).is_err());
}

#[test]
fn recovery_oracle_and_executable_inventory_regenerate_exactly() {
    let result = std::process::Command::new("python")
        .arg(repo().join("tools/gen-portable-d9-recovery-full-oracle.py"))
        .arg("--check")
        .current_dir(repo())
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}
