//! Reachability and available D9 preconditions only. D10 is explicitly NOT closed.
use super::*;

fn materialize(case: &Json) -> Vec<u8> {
    let bytes = std::fs::read(repo().join(case["bundle_file"].as_str().unwrap())).unwrap();
    let changes = case["mutations"].as_array().unwrap();
    assert!(changes.len() <= 128);
    if changes.is_empty() {
        return bytes;
    }
    let mut value: Json = serde_json::from_slice(&bytes).unwrap();
    for change in changes {
        assert_eq!(change["op"], "set");
        let path = change["path"].as_array().unwrap();
        assert!(!path.is_empty() && path.len() <= 96);
        let mut current = &mut value;
        for token in &path[..path.len() - 1] {
            let token = token.as_str().unwrap();
            current = if current.is_array() {
                &mut current.as_array_mut().unwrap()[token.parse::<usize>().unwrap()]
            } else {
                current.as_object_mut().unwrap().get_mut(token).unwrap()
            };
        }
        let token = path.last().unwrap().as_str().unwrap();
        let target = if current.is_array() {
            &mut current.as_array_mut().unwrap()[token.parse::<usize>().unwrap()]
        } else {
            current.as_object_mut().unwrap().get_mut(token).unwrap()
        };
        *target = change["value"].clone();
    }
    serde_json::to_vec(&value).unwrap()
}

#[test]
fn d10_original_reachability_and_available_independent_d9_preconditions() {
    let manifest = read_json(
        "spec/conformance/portable-root-export-revalidation/expected-full/d10-preconditions/manifest.json",
    );
    let contract = read_json(
        "spec/conformance/portable-root-export-revalidation/coverage/field-coverage.json",
    );
    let cases = manifest["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 26);
    assert_eq!(manifest["d9_expected_count"], 24);
    assert_eq!(manifest["d10_full_field_expected"], 0);
    let output = repo().join("target/conformance/d10-gap-audit");
    std::fs::create_dir_all(&output).unwrap();
    let mut results = Vec::new();
    for chunk in cases.chunks(3) {
        let handles = chunk.iter().map(|case| {
            let case = case.clone();
            let contract = contract.clone();
            let output = output.clone();
            std::thread::spawn(move || {
                let bytes = materialize(&case);
                let identity = json!({"canonical_bytes":bytes.len(),"canonical_sha256":hex::encode(sha2::Sha256::digest(&bytes))});
                assert_eq!(identity, case["input_identity"], "exact mutated input");
                let analysis = PortableProofCodec::default().validate_supported(&bytes)
                    .and_then(|supported| supported.revalidate_root_export());
                let slug = case["case_id"].as_str().unwrap().rsplit('/').next().unwrap();
                match analysis {
                    Ok(analysis) => {
                        let actual = json!({"case_id":case["case_id"],"input_identity":identity,"production_stage":"D10","context":analysis.inputs().context(),"d9_root":analysis.root_summary().typed_facts(),"d9_root_recovery":analysis.root_recovery().typed_facts()});
                        // Evidence output is not an Expected-generation input.
                        std::fs::write(output.join(format!("{slug}.actual.json")),serde_json::to_vec_pretty(&json!({"d9_facts":actual,"d10_untrusted_observation":analysis.typed_projection()["d10_root_export"]})).unwrap()).unwrap();
                        if let Some(path) = case["expected_d9_file"].as_str() {
                            let expected = read_json(path);
                            match compare(&expected["facts"], &actual, &contract) {
                                Ok((left,right)) => json!({"case_id":case["case_id"],"input_identity":identity,"actual_first_stage":"D10","d9_status":"passed","expected_required":left.required,"actual_required":right.required,"compared_required":left.required,"diagnostic_instances":left.diagnostic,"d10_status":"NotComparedFormalModelGap","d10_fully_compared":false}),
                                Err(error) => json!({"case_id":case["case_id"],"actual_first_stage":"D10","d9_status":"mismatch","first_difference":error,"d10_status":"NotComparedFormalModelGap","d10_fully_compared":false}),
                            }
                        } else {
                            json!({"case_id":case["case_id"],"input_identity":identity,"actual_first_stage":"D10","d9_status":"OracleImplementationGap","d10_status":"NotComparedFormalModelGap","d10_fully_compared":false})
                        }
                    }
                    Err(error) => json!({"case_id":case["case_id"],"input_identity":identity,"actual_first_stage":format!("{:?}",error.stage),"d9_status":"UnexpectedEarlyRejection","error_family":error.family,"error_path":error.path,"d10_status":"NoD10Report","d10_fully_compared":false}),
                }
            })
        }).collect::<Vec<_>>();
        for handle in handles {
            results.push(handle.join().expect("D10 reachability worker"));
        }
    }
    let reached = results
        .iter()
        .filter(|row| row["actual_first_stage"] == "D10")
        .count();
    let compared = results
        .iter()
        .filter(|row| row["d9_status"] == "passed")
        .count();
    let required: u64 = results
        .iter()
        .filter_map(|row| row["compared_required"].as_u64())
        .sum();
    let diagnostic: u64 = results
        .iter()
        .filter_map(|row| row["diagnostic_instances"].as_u64())
        .sum();
    let report = json!({"outcome":"C - Formal Model Gap; NOT COMPLETE","executed":results.len(),"reached_d10":reached,"d9_fully_compared":compared,"d9_oracle_gaps":26-compared,"d9_compared_required_instances":required,"d9_diagnostic_instances":diagnostic,"d10_fully_compared":0,"d10_formal_gap_cases":26,"skipped":0,"cases":results});
    std::fs::write(
        output.join("report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    assert_eq!(reached, 26, "actual reachability: {report}");
    assert_eq!(compared, 24, "D9 differences: {report}");
    assert_eq!(
        required,
        manifest["d9_generated_instance_totals"]["required"]
            .as_u64()
            .unwrap()
    );
    assert_eq!(
        diagnostic,
        manifest["d9_generated_instance_totals"]["diagnostic"]
            .as_u64()
            .unwrap()
    );
}
