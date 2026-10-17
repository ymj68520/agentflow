//! R4-A real-input representation probe. NOT a new producer or DAG codec.
#[allow(dead_code, unused_imports)]
mod fixtures {
    include!("../../af-contract/tests/component_contract_conformance.rs");
    use af_computation::*;
    use serde_json::{Value, json};

    #[test]
    fn raw_zero_retains_verdict_while_complete_node_binding_is_not_invented() {
        let modules = modules();
        let schemas = TypeSchemaUniverse::new();
        let effects = EffectCatalogUniverse::new();
        let resources = ResourceTypeUniverse::new();
        let flow = FlowCatalogs::new();
        let usage = UsageMeterUniverse::new();
        let recovery = RecoveryOperationUniverse::new();
        let execution = ExecutionUniverses::default();
        let relations = RelationSnapshots::default();
        let context = StaticContractProofContext {
            modules: &modules,
            schema_universe: &schemas,
            effects: &effects,
            resources: &resources,
            flow: &flow,
            usage_meters: &usage,
            recovery: &recovery,
            execution: &execution,
            relations: &relations,
        };
        let generated = std::process::Command::new("python")
            .env("PYTHONUTF8", "1")
            .args(["tools/oracle/raw_port_capture.py", "--vectors"])
            .current_dir(repo_root())
            .output()
            .unwrap();
        assert!(generated.status.success());
        let inputs: Vec<Value> = serde_json::from_slice(&generated.stdout).unwrap();
        let input = inputs
            .iter()
            .find(|v| v["case_id"] == "c2r1/declaration_bundle/input-zero")
            .unwrap();
        let package = &input["package"];
        let owner = &package["owners"][0]["anchor"];
        let document =
            &package["owners"][0]["container"]["declaration_bundle"]["declarations"][0]["document"];
        let path = [RawDocumentMember::Entry(0)];
        let request = DocumentOccurrenceMetadataRequest {
            source_package: Some(package),
            source_owner: Some(owner),
            entry_ordinal: Some(0),
            actual_document: document,
            candidate: &input["candidate"],
            internal_path: &path,
            physical: OccurrencePhysicalMetadata::NotSupplied,
        };
        let compiler = current_compiler_semantics_identity();
        let capture =
            raw_capture::capture_raw_ports(&request, &context, &context, &compiler).unwrap();
        capture.validate().unwrap();
        assert_eq!(capture.seeds().len(), 1);
        assert_eq!(
            capture.seeds()[0].status(),
            af_contract::ProofVerdict::Violation
        );
        assert!(std::ptr::eq(capture.subject().context, &raw const context));
        let relation = inspect_relation_context_metadata(&context, &context, &compiler).unwrap();
        assert!(relation.relation_partition_complete());
        // This is a genuine complete *logical metadata value* of the Context,
        // not an input DAG/Scope, registered Raw Subject or complete Fact.
        // Every non-module universe above was actually constructed empty.
        let compiled_modules: Vec<_> = relation.metadata()["effect_resource"]["current_modules"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| json!([m["specification_id"], m]))
            .collect();
        let context_value = json!({
            "compiled_modules":compiled_modules,"type_schemas":[],"effect_catalogs":[],
            "resource_types":{"schemas":[],"descriptors":[]},
            "flow_catalogs":{"confidentiality":[],"taints":[],"sinks":[]},
            "usage_meters":[],"recovery_operations":{"effects":[],"reconciliation_operations":[]},
            "execution_universes":{"capabilities":[],"trust_domains":[],"regions":[],"region_compatibility":[],"runtime_profiles":[]},
            "relation_snapshots":{"effect_resource":{"none":null},"recovery_usage":{"none":null},"flow_execution":{"none":null}}
        });
        validate_relation_registered_shape("ContractProofContextContents", &context_value).unwrap();
        let mut probes = Vec::new();
        for name in [
            "ContractProofContextContents.compiled_modules",
            "Map<SpecificationId,CompiledModuleContents>",
            "ContractProofContextContents.resource_types",
        ] {
            let value = json!({"owner":RELATION_MODEL_ID,"name":name});
            let failure = validate_relation_registered_shape("ModelType", &value).unwrap_err();
            assert_eq!(failure, RegisteredModelFailure::UnsupportedModel);
            probes.push(json!({"candidate":value,"actual_failure":"UnsupportedModel"}));
        }
        let invented_map = json!({"map":[]});
        assert_eq!(
            validate_relation_registered_shape("NodeBody", &invented_map),
            Err(RegisteredModelFailure::UnsupportedModel)
        );
        // A registered outer type/name cannot supply the missing child type.
        // Shape admission of an existing body case is NOT typed DAG admission.
        validate_relation_registered_shape("NodeBody", &json!({"sequence":[]})).unwrap();
        let m = relation_model_catalogue().unwrap();
        let outer_as_child = &m["types"]["ContractProofContextContents"];
        assert_eq!(outer_as_child["kind"], "record");
        let raw: ComponentContract =
            serde_json::from_value(input["wire_candidate"].clone()).unwrap();
        let error = af_contract::validate_raw_contract(&raw, &context).unwrap_err();
        let occurrence = capture.seeds()[0].event().occurrence;
        assert!(!capture.complete_scope());
        assert!(!capture.can_issue_root_export());
        let result = json!({"case_id":"c2r4/model6-structural-node-binding",
            "source_case_id":input["case_id"],"source_input":input,
            "context":context_value,"context_roles":capture.context_roles(),
            "native_verdict":"Violation","condition":"zero","raw_member_value":0,"minimum":1,
            "occurrence":{"direction":"input","original_ordinal":occurrence.original_ordinal,
                "sorted_ordinal":occurrence.sorted_ordinal,"port_no":occurrence.port_no},
            "native_error_rule":error.rule_id,"native_error_kind":error.code,
            "context_metadata_shape_admitted":true,
            "relation_partition_complete":relation.relation_partition_complete(),
            "probes":probes,"invented_map_body_failure":"UnsupportedModel",
            "existing_sequence_body_shape_admitted":true,"complete_typed_dag_admitted":false,
            "complete_raw_subject_materialized":false,"complete_fact_materialized":false,
            "normalizer_implemented":false,"positive_root_export":false});
        std::fs::write(
            repo_root().join("target/phase-04c1b4c4r3c2r4-binding-actual.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .unwrap();
    }
}
