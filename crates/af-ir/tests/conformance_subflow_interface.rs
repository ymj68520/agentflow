//! Subflow interface conformance harness (Phase 3D): consumes the FROZEN
//! corpus under `spec/conformance/subflow-interface/` (generated once by the
//! independent Python implementation `tools/gen-subflow-interface-corpus.py`).
//!
//! The frozen projections are explicitly NON_NORMATIVE CONFORMANCE PROJECTION
//! data: the interface itself is derived-only and never serialized in
//! production. Expected digests are computed from the target documents by
//! the implementation under test and pinned against the corpus; expected
//! error kinds come from the corpus; tests never generate expected values
//! from the checker.

use af_ir::{SemanticDataflowAnalysis, SemanticIrDocument, SubflowIrUniverse};
use af_semantic::TypeSchemaUniverse;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

fn load_json(relative: &str) -> serde_json::Value {
    serde_json::from_str(&read(relative)).unwrap_or_else(|e| panic!("cannot parse {relative}: {e}"))
}

const CORPUS_REVISION: u64 = 1;
const POSITIVE: &str =
    "spec/conformance/subflow-interface/positive/subflow-interface-positive.json";
const NEGATIVE: &str =
    "spec/conformance/subflow-interface/negative/subflow-interface-negative.json";
const EQUIVALENCE: &str =
    "spec/conformance/subflow-interface/equivalence/subflow-interface-equivalence.json";

fn module_set() -> af_idl::CompiledModuleSet {
    let sources = [
        (
            "semantic-core.afidl".to_string(),
            read("spec/idl/semantic-core.afidl"),
        ),
        (
            "semantic-ir.afidl".to_string(),
            read("spec/idl/semantic-ir.afidl"),
        ),
    ];
    af_idl::compile_module_set(&sources).expect("the normative module set compiles")
}

/// The fixture universe: the shared Phase 3A fixture plus any additional
/// fixture schemas the corpus carries (compiled from the frozen sources and
/// pinned against the frozen digests).
fn fixture_universe(corpus: &serde_json::Value) -> TypeSchemaUniverse {
    let golden = load_json("spec/conformance/semantic-ir/golden/semantic-ir-golden.json");
    let compiled = af_idl::compile(&read(golden["fixture_schema"]["file"].as_str().unwrap()))
        .expect("the fixture schema compiles");
    let mut universe = TypeSchemaUniverse::new();
    universe
        .insert_compiled(&compiled)
        .expect("the fixture registers");
    for schema in corpus["fixture_schemas"].as_array().unwrap() {
        let compiled = af_idl::compile(schema["source"].as_str().unwrap())
            .expect("the corpus fixture schema compiles");
        for expected in schema["types"].as_array().unwrap() {
            let digest = compiled
                .digest(expected["name"].as_str().unwrap())
                .unwrap_or_else(|| panic!("fixture type missing"));
            assert_eq!(
                digest.hex(),
                expected["digest_hex"].as_str().unwrap(),
                "corpus fixture digest drifted"
            );
        }
        universe
            .insert_compiled(&compiled)
            .expect("the corpus fixture registers");
    }
    universe
}

/// Builds a universe from the case's target documents. Returns the digests
/// in target order — each digest is always computed from the target itself,
/// never taken from the corpus.
fn build_universe(
    targets: &[SemanticIrDocument],
    universe: &TypeSchemaUniverse,
    set: &af_idl::CompiledModuleSet,
) -> (SubflowIrUniverse, Vec<af_semantic::DigestRefValue>) {
    let mut subflows = SubflowIrUniverse::new();
    let mut digests = Vec::new();
    for target in targets {
        let checked = target
            .check(universe)
            .unwrap_or_else(|e| panic!("target must pass Phase 3B: {e}"));
        let digest = subflows
            .insert(&checked, set)
            .unwrap_or_else(|e| panic!("target must be insertable: {e}"));
        digests.push(digest);
    }
    (subflows, digests)
}

/// The caller side of one case, validated through the whole pass chain.
fn prepare_caller(
    caller: &SemanticIrDocument,
    universe: &TypeSchemaUniverse,
) -> SemanticDataflowAnalysis {
    let checked = caller
        .check(universe)
        .unwrap_or_else(|e| panic!("caller must pass Phase 3B: {e}"));
    checked
        .analyze_dataflow()
        .unwrap_or_else(|e| panic!("caller must pass the Phase 3C analysis: {e}"))
}

/// NON_NORMATIVE CONFORMANCE PROJECTION: renders the derived interface the
/// way the corpus freezes it. Test projection only, never a production
/// serialization.
fn project_interface(interface: &af_ir::SubflowInterface) -> serde_json::Value {
    serde_json::json!({
        "inputs": interface.inputs().iter().map(|port| serde_json::json!({
            "port_no": port.port_no(),
            "name": port.name(),
            "type": port.semantic_type().canonical_string(),
        })).collect::<Vec<_>>(),
        "outputs": interface.outputs().iter().map(|port| serde_json::json!({
            "port_no": port.port_no(),
            "name": port.name(),
            "type": port.semantic_type().canonical_string(),
        })).collect::<Vec<_>>(),
        "resources": interface.resources().iter().map(|port| serde_json::json!({
            "port_no": port.port_no(),
            "source_resource_slot": port.source_resource_slot(),
            "requirement_type": port.requirement_type().to_string(),
        })).collect::<Vec<_>>(),
    })
}

#[test]
fn positive_cases_check_against_the_frozen_universe() {
    let corpus = load_json(POSITIVE);
    assert_eq!(corpus["corpus_revision"].as_u64(), Some(CORPUS_REVISION));
    let cases = corpus["cases"].as_array().unwrap();
    assert!(cases.len() >= 12, "the positive corpus shrank");

    let universe = fixture_universe(&corpus);
    let set = module_set();
    for case in cases {
        let case_id = case["case_id"].as_str().unwrap();
        let targets: Vec<SemanticIrDocument> = case["targets"]
            .as_array()
            .unwrap()
            .iter()
            .map(|document| serde_json::from_value(document.clone()).unwrap())
            .collect();
        let (subflows, digests) = build_universe(&targets, &universe, &set);

        let frozen: std::collections::BTreeSet<&str> = case["frozen_target_digests"]
            .as_array()
            .unwrap()
            .iter()
            .map(|digest| digest.as_str().unwrap())
            .collect();
        let actual: std::collections::BTreeSet<&str> =
            digests.iter().map(|digest| digest.hex.as_str()).collect();
        assert_eq!(actual, frozen, "{case_id}: target digests drifted");

        for (target_index, digest) in digests.iter().enumerate() {
            let interface = subflows
                .interface(digest)
                .unwrap_or_else(|| panic!("{case_id}: digest must resolve"));
            assert_eq!(
                project_interface(interface),
                case["interfaces"][target_index],
                "{case_id}: derived interface drifted (target {target_index})"
            );
        }

        let caller: SemanticIrDocument = serde_json::from_value(case["caller"].clone()).unwrap();
        let analysis = prepare_caller(&caller, &universe);
        caller
            .check(&universe)
            .unwrap()
            .check_subflow_interfaces(&analysis, &subflows)
            .unwrap_or_else(|e| panic!("{case_id}: {e}"));
    }
}

#[test]
fn negatives_fail_only_at_the_subflow_layer() {
    let corpus = load_json(NEGATIVE);
    assert_eq!(corpus["corpus_revision"].as_u64(), Some(CORPUS_REVISION));
    let cases = corpus["cases"].as_array().unwrap();
    assert!(cases.len() >= 15, "the negative corpus shrank");

    let universe = fixture_universe(&corpus);
    let set = module_set();
    for case in cases {
        let case_id = case["case_id"].as_str().unwrap();
        let targets: Vec<SemanticIrDocument> = case["targets"]
            .as_array()
            .unwrap()
            .iter()
            .map(|document| serde_json::from_value(document.clone()).unwrap())
            .collect();
        let (subflows, _) = build_universe(&targets, &universe, &set);
        let caller: SemanticIrDocument = serde_json::from_value(case["caller"].clone()).unwrap();
        let analysis = prepare_caller(&caller, &universe);
        let error = caller
            .check(&universe)
            .unwrap()
            .check_subflow_interfaces(&analysis, &subflows)
            .expect_err(&format!("{case_id}: must be rejected"));
        assert_eq!(
            error.kind(),
            case["error_kind"].as_str().unwrap(),
            "{case_id}: error kind mismatch ({error})"
        );
    }
}

#[test]
fn table_order_moves_neither_digest_nor_interface() {
    let corpus = load_json(EQUIVALENCE);
    let universe = fixture_universe(&corpus);
    let set = module_set();
    for group in corpus["table_order_groups"].as_array().unwrap() {
        let case_id = group["case_id"].as_str().unwrap();
        let base: SemanticIrDocument = serde_json::from_value(group["base"].clone()).unwrap();
        let member: SemanticIrDocument = serde_json::from_value(group["member"].clone()).unwrap();
        let mut subflows = SubflowIrUniverse::new();
        let digest = subflows
            .insert(&base.check(&universe).unwrap(), &set)
            .unwrap();
        // Same canonical bytes: re-insertion is idempotent, never an
        // overwrite.
        let digest_again = subflows
            .insert(&member.check(&universe).unwrap(), &set)
            .unwrap();
        assert_eq!(digest, digest_again, "{case_id}");
        assert_eq!(subflows.len(), 1, "{case_id}");
        assert_eq!(
            digest.hex,
            group["expected_digest"].as_str().unwrap(),
            "{case_id}: digest drifted"
        );
        assert_eq!(
            project_interface(subflows.interface(&digest).unwrap()),
            group["expected_interface"],
            "{case_id}: interface drifted"
        );
    }
}

#[test]
fn local_type_numbering_never_changes_the_interface_shape() {
    let corpus = load_json(EQUIVALENCE);
    let universe = fixture_universe(&corpus);
    let set = module_set();
    for group in corpus["type_id_groups"].as_array().unwrap() {
        let case_id = group["case_id"].as_str().unwrap();
        let target_a: SemanticIrDocument =
            serde_json::from_value(group["target_a"].clone()).unwrap();
        let target_b: SemanticIrDocument =
            serde_json::from_value(group["target_b"].clone()).unwrap();
        let mut subflows = SubflowIrUniverse::new();
        let digest_a = subflows
            .insert(&target_a.check(&universe).unwrap(), &set)
            .unwrap();
        let digest_b = subflows
            .insert(&target_b.check(&universe).unwrap(), &set)
            .unwrap();
        assert_ne!(
            digest_a.hex, digest_b.hex,
            "{case_id}: the renumbered target must have its own digest"
        );
        assert_eq!(digest_a.hex, group["expected_digest_a"].as_str().unwrap());
        assert_eq!(digest_b.hex, group["expected_digest_b"].as_str().unwrap());
        assert_eq!(
            project_interface(subflows.interface(&digest_a).unwrap()),
            project_interface(subflows.interface(&digest_b).unwrap()),
            "{case_id}: the interface shape moved with local numbering"
        );
        assert_eq!(
            project_interface(subflows.interface(&digest_a).unwrap()),
            group["expected_interface"],
            "{case_id}"
        );
    }
}

#[test]
fn port_ordinals_follow_the_frozen_canonical_keys() {
    let corpus = load_json(EQUIVALENCE);
    let universe = fixture_universe(&corpus);
    let set = module_set();
    for proof in corpus["port_order_proofs"].as_array().unwrap() {
        let case_id = proof["case_id"].as_str().unwrap();
        let target: SemanticIrDocument = serde_json::from_value(proof["target"].clone()).unwrap();
        let mut subflows = SubflowIrUniverse::new();
        let digest = subflows
            .insert(&target.check(&universe).unwrap(), &set)
            .unwrap();
        let interface = subflows.interface(&digest).unwrap();

        // Input ports enumerate the run parameters in RunParameterId order.
        let mut parameters: Vec<_> = target.run_parameters.clone();
        parameters.sort_by_key(|parameter| parameter.id);
        let expected_inputs: Vec<(u64, u64)> = proof["expected_inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|expected| {
                (
                    expected["port_no"].as_u64().unwrap(),
                    expected["parameter_id"].as_u64().unwrap(),
                )
            })
            .collect();
        for (index, (port, parameter)) in interface.inputs().iter().zip(&parameters).enumerate() {
            assert_eq!(
                (u64::from(port.port_no()), u64::from(parameter.id.raw())),
                (expected_inputs[index].0, expected_inputs[index].1),
                "{case_id}: input port {index} drifted"
            );
        }
        // Output ports follow the canonical NFC-name order; resource ports
        // the ResourceSlotId order — both frozen in the corpus projection.
        for (index, port) in interface.outputs().iter().enumerate() {
            let expected = &proof["expected_outputs"][index];
            assert_eq!(
                (u64::from(port.port_no()), port.name()),
                (
                    expected["port_no"].as_u64().unwrap(),
                    expected["name"].as_str().unwrap()
                ),
                "{case_id}: output port {index} drifted"
            );
        }
        for (index, port) in interface.resources().iter().enumerate() {
            let expected = &proof["expected_resources"][index];
            assert_eq!(
                (
                    u64::from(port.port_no()),
                    u64::from(port.source_resource_slot())
                ),
                (
                    expected["port_no"].as_u64().unwrap(),
                    expected["source_resource_slot"].as_u64().unwrap()
                ),
                "{case_id}: resource port {index} drifted"
            );
        }
    }
}

#[test]
fn interface_derivation_is_deterministic_and_never_hashed() {
    let corpus = load_json(POSITIVE);
    let cases = corpus["cases"].as_array().unwrap();
    for case in cases {
        let targets: Vec<SemanticIrDocument> = case["targets"]
            .as_array()
            .unwrap()
            .iter()
            .map(|document| serde_json::from_value(document.clone()).unwrap())
            .collect();
        for target in &targets {
            let first = af_ir::derive_subflow_interface(target);
            let second = af_ir::derive_subflow_interface(target);
            assert_eq!(first, second, "{}", case["case_id"].as_str().unwrap());
        }
    }
}
