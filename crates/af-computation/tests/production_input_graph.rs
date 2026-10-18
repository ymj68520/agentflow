//! Genuine native source/context calls. Registration fixture ranks are not used.
use af_computation::*;
use af_contract::{RelationSnapshots, StaticContractProofContext};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn read(name: &str) -> Value {
    serde_json::from_slice(
        &std::fs::read(
            root()
                .join("spec/conformance/production-input-graph")
                .join(name),
        )
        .unwrap(),
    )
    .unwrap()
}
#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn all_sources() -> Vec<(String, String)> {
    let registry: Value = serde_json::from_slice(
        &std::fs::read(root().join("spec/registry/specifications.json")).unwrap(),
    )
    .unwrap();
    let mut sources: Vec<_> = registry["specifications"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|s| {
            let path = s["normative_source"]["logical_path"].as_str().unwrap();
            path.ends_with(".afidl").then(|| {
                (
                    path.to_owned(),
                    std::fs::read_to_string(root().join(path)).unwrap(),
                )
            })
        })
        .collect();
    let name = "spec/conformance/production-input-graph/environment.afidl";
    sources.push((
        name.into(),
        std::fs::read_to_string(root().join(name)).unwrap(),
    ));
    sources
}
struct Inputs {
    modules: af_idl::CompiledModuleSet,
    schemas: af_semantic::TypeSchemaUniverse,
    effects: af_effect::EffectCatalogUniverse,
    resources: af_resource::ResourceTypeUniverse,
    flow: af_flow::FlowCatalogs,
    usage: af_usage::UsageMeterUniverse,
    recovery: af_recovery::RecoveryOperationUniverse,
    execution: af_execution::ExecutionUniverses,
    relations: RelationSnapshots,
}
impl Inputs {
    fn make(case: &Value) -> Self {
        let cfg = read("environment.json");
        let mut sources = all_sources();
        if case["module_order"] == "reverse" {
            sources.reverse();
        }
        let original = af_idl::compile_module_set(&sources).unwrap();
        let mut this = Self {
            modules: original.clone(),
            schemas: af_semantic::TypeSchemaUniverse::new(),
            effects: af_effect::EffectCatalogUniverse::new(),
            resources: af_resource::ResourceTypeUniverse::new(),
            flow: af_flow::FlowCatalogs::new(),
            usage: af_usage::UsageMeterUniverse::new(),
            recovery: af_recovery::RecoveryOperationUniverse::new(),
            execution: af_execution::ExecutionUniverses::default(),
            relations: RelationSnapshots::default(),
        };
        if case["context"] == "rich" {
            for schema in cfg["type_schemas"].as_array().unwrap() {
                this.schemas
                    .insert_type(original.type_by_identity(schema.as_str().unwrap()).unwrap())
                    .unwrap();
            }
            let schema = original
                .type_by_identity(cfg["resource_schema"].as_str().unwrap())
                .unwrap();
            this.resources.insert_schema_type(schema).unwrap();
            this.resources
                .recognize(af_resource::ResourceTypeDescriptor {
                    specification_id: schema.schema.clone(),
                    type_name: schema.name.clone(),
                    schema_hex: schema.digest().unwrap().hex().into(),
                    recognized_attributes: cfg["resource_attributes"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|s| s.as_str().unwrap().to_owned())
                        .collect(),
                })
                .unwrap();
            for cat in cfg["effect_catalogs"].as_array().unwrap() {
                let c = af_effect::CapabilityCatalog::new(
                    cat["catalog"].as_str().unwrap(),
                    cat["operations"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_str().unwrap()),
                )
                .unwrap();
                this.effects.insert(&c).unwrap();
                this.recovery.effects.insert(&c).unwrap();
            }
            for v in cfg["confidentiality"].as_array().unwrap() {
                this.flow.labels.recognize_confidentiality(
                    v["catalog"].as_str().unwrap(),
                    v["name"].as_str().unwrap(),
                );
            }
            for v in cfg["taints"].as_array().unwrap() {
                this.flow
                    .labels
                    .recognize_taint(v["catalog"].as_str().unwrap(), v["name"].as_str().unwrap());
            }
            for v in cfg["sinks"].as_array().unwrap() {
                this.flow.sinks.recognize_operation(
                    v["catalog"].as_str().unwrap(),
                    v["name"].as_str().unwrap(),
                    v["decision"].as_bool().unwrap(),
                );
            }
            let unit = af_usage::UsageUnitRef::new(
                cfg["unit"]["catalog"].as_str().unwrap(),
                cfg["unit"]["name"].as_str().unwrap(),
            )
            .unwrap();
            for v in cfg["meters"].as_array().unwrap() {
                this.usage
                    .recognize(
                        &af_usage::UsageMeterRef {
                            catalog: cfg["meter_catalog"].as_str().unwrap().into(),
                            name: v["name"].as_str().unwrap().into(),
                        },
                        &unit,
                        if v["aggregation"] == "Additive" {
                            af_usage::UsageAggregation::Additive
                        } else {
                            af_usage::UsageAggregation::Unsupported
                        },
                    )
                    .unwrap();
            }
            for v in cfg["reconciliation"].as_array().unwrap() {
                this.recovery
                    .recognize_reconciliation_operation(
                        v["catalog"].as_str().unwrap(),
                        v["name"].as_str().unwrap(),
                    )
                    .unwrap();
            }
            let capability: af_execution::ExecutionCapabilityRef =
                serde_json::from_value(cfg["capability"].clone()).unwrap();
            let attributes: Vec<af_execution::CapabilityAttribute> =
                serde_json::from_value(cfg["attributes"].clone()).unwrap();
            for version in cfg["versions"].as_array().unwrap() {
                this.execution
                    .capabilities
                    .recognize(&capability, version.as_str().unwrap(), attributes.clone())
                    .unwrap();
            }
            for v in cfg["trust_domains"].as_array().unwrap() {
                this.execution
                    .trust_domains
                    .recognize(serde_json::from_value(v.clone()).unwrap())
                    .unwrap();
            }
            for v in cfg["regions"].as_array().unwrap() {
                this.execution
                    .regions
                    .recognize(serde_json::from_value(v.clone()).unwrap())
                    .unwrap();
            }
            this.execution
                .regions
                .recognize_compatibility(
                    serde_json::from_value(cfg["regions"][0].clone()).unwrap(),
                    serde_json::from_value(cfg["regions"][1].clone()).unwrap(),
                )
                .unwrap();
            this.execution
                .runtime_profiles
                .recognize(
                    serde_json::from_value(cfg["profile"].clone()).unwrap(),
                    cfg["family"].as_str().unwrap(),
                    cfg["features"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_str().unwrap().to_owned()),
                )
                .unwrap();
        }
        if case["relations"] == "nonempty" {
            let rel = read("relations.json");
            this.relations.effect_resource = Some(
                af_contract::RelationCatalogSnapshot::new(
                    "g3-actual",
                    serde_json::from_value::<af_contract::EffectResourceCompatibilityCatalog>(
                        rel["effect_resource"].clone(),
                    )
                    .unwrap(),
                    &original,
                )
                .unwrap(),
            );
            this.relations.recovery_usage = Some(
                af_contract::RelationCatalogSnapshot::new(
                    "g3-actual",
                    serde_json::from_value::<af_contract::RecoveryUsageCoverageCatalog>(
                        rel["recovery_usage"].clone(),
                    )
                    .unwrap(),
                    &original,
                )
                .unwrap(),
            );
            this.relations.flow_execution = Some(
                af_contract::RelationCatalogSnapshot::new(
                    "g3-actual",
                    serde_json::from_value::<af_contract::InformationFlowExecutionCatalog>(
                        rel["flow_execution"].clone(),
                    )
                    .unwrap(),
                    &original,
                )
                .unwrap(),
            );
            // Source data is already the complete native A catalog. A constructor
            // normalization must not silently redefine the independent input.
            assert_eq!(
                this.relations.effect_resource.as_ref().unwrap().catalog(),
                &serde_json::from_value::<af_contract::EffectResourceCompatibilityCatalog>(
                    rel["effect_resource"].clone()
                )
                .unwrap()
            );
            assert_eq!(
                this.relations.recovery_usage.as_ref().unwrap().catalog(),
                &serde_json::from_value::<af_contract::RecoveryUsageCoverageCatalog>(
                    rel["recovery_usage"].clone()
                )
                .unwrap()
            );
            assert_eq!(
                this.relations.flow_execution.as_ref().unwrap().catalog(),
                &serde_json::from_value::<af_contract::InformationFlowExecutionCatalog>(
                    rel["flow_execution"].clone()
                )
                .unwrap()
            );
        }
        if case["current_modules"] == "missing_relation" {
            sources.retain(|(p, _)| !p.ends_with("effect-resource-compatibility.afidl"));
            this.modules = af_idl::compile_module_set(&sources).unwrap();
        }
        if case["current_modules"] == "empty" {
            this.modules = af_idl::compile_module_set(&[]).unwrap();
        } else if case["current_modules"] == "single" {
            sources
                .retain(|(p, _)| p == "spec/conformance/production-input-graph/environment.afidl");
            this.modules = af_idl::compile_module_set(&sources).unwrap();
        }
        this
    }
    fn borrow(&self) -> StaticContractProofContext<'_> {
        StaticContractProofContext {
            modules: &self.modules,
            schema_universe: &self.schemas,
            effects: &self.effects,
            resources: &self.resources,
            flow: &self.flow,
            usage_meters: &self.usage,
            recovery: &self.recovery,
            execution: &self.execution,
            relations: &self.relations,
        }
    }
}
fn selected_document(package: &Value, owner: usize, slot: usize) -> &Value {
    let (tag, body) = package["owners"][owner]["container"]
        .as_object()
        .unwrap()
        .iter()
        .next()
        .unwrap();
    &body[match tag.as_str() {
        "declaration_bundle" => "declarations",
        "import_manifest" => "imports",
        "workflow_source_collection" => "sources",
        _ => panic!("unknown input container"),
    }][slot]["document"]
}
fn production_cases(shard: usize, only_first: bool) {
    let cases = read("cases.json");
    let compiler = current_compiler_semantics_identity();
    let mut receipt = Vec::new();
    for (index, case) in cases
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(i, _)| i % 4 == shard)
    {
        let bundle = read(case["bundle"].as_str().unwrap());
        let package = &bundle["package"];
        let row = &bundle["declarations"]
            [usize::try_from(case["selected_entry"].as_u64().unwrap()).unwrap()];
        let native: af_contract::ComponentContract =
            serde_json::from_value(row["wire_candidate"].clone()).unwrap();
        let owner = usize::try_from(case["owner"].as_u64().unwrap()).unwrap();
        let slot = usize::try_from(case["slot"].as_u64().unwrap()).unwrap();
        let document = selected_document(package, owner, slot);
        let path = if slot == 1 {
            vec![RawDocumentMember::Field(b"selected".to_vec())]
        } else {
            vec![RawDocumentMember::Entry(
                case["selected_entry"].as_u64().unwrap(),
            )]
        };
        let inputs = Inputs::make(case);
        let context = inputs.borrow();
        let physical = if slot == 1 {
            OccurrencePhysicalMetadata::Supplied {
                source_owner: &package["owners"][owner]["anchor"],
                entry_ordinal: slot as u64,
                actual_document: document,
                value_path: &path,
            }
        } else {
            OccurrencePhysicalMetadata::NotSupplied
        };
        let source = DocumentOccurrenceMetadataRequest {
            source_package: Some(package),
            source_owner: Some(&package["owners"][owner]["anchor"]),
            entry_ordinal: Some(slot as u64),
            actual_document: document,
            candidate: &row["candidate"],
            internal_path: &path,
            physical,
        };
        admit_document_occurrence_metadata(&source, &context, &context, &compiler)
            .unwrap_or_else(|e| panic!("native source admission {}: {e:?}", case["case_id"]));
        let graph = bind_raw_input_graph(&source, &context, &context, &compiler)
            .unwrap_or_else(|e| panic!("{}: {e:?}", case["case_id"]));
        assert!(!graph.can_issue_root_export());
        assert!(std::ptr::eq(graph.source().actual_document, document));
        let subject_node = graph.subject_node();
        assert!(std::ptr::eq(subject_node.carrier(), &raw const graph));
        assert!(subject_node.dag_rank() < u64::try_from(graph.node_count()).unwrap());
        let file = if only_first {
            format!("target/phase-04c1b4c4r3c2r4g3-development-case-{index:03}.ace1")
        } else {
            format!("target/phase-04c1b4c4r3c2r4g3-case-{index:03}.ace1")
        };
        std::fs::write(root().join(&file), graph.canonical_bytes()).unwrap();
        // Validation is outside the Binder and produces no complete Raw Fact.
        let port_result = af_contract::validate_raw_contract(&native, &context);
        let c_compatibility = if case["relations"] == "nonempty"
            && case["bundle"] == "bundle-00.json"
            && case["slot"] == 1
            && case["owner"] == 0
        {
            let observed =
                inspect_relation_context_metadata(graph.context(), &context, &compiler).unwrap();
            assert_eq!(
                observed.relation_partition_complete(),
                case["current_modules"] == "complete"
            );
            assert!(!observed.can_issue_capability());
            json!({"executed_outside_binder":true,"complete_partition":observed.relation_partition_complete(),"metadata":observed.metadata(),"model7_subject_admission":false})
        } else {
            json!({"executed_outside_binder":false,"model7_subject_admission":false})
        };
        receipt.push(json!({"case_id":case["case_id"],"input_case":case,"graph":file,"nodes":graph.node_count(),"subject_node_rank":subject_node.dag_rank(),"sites":graph.structural_sites(),"positive_root_export":false,"complete_raw_fact":false,"raw_validation_succeeded":port_result.is_ok(),"c_compatibility":c_compatibility}));
        if only_first {
            break;
        }
    }
    let file = if only_first {
        format!("target/phase-04c1b4c4r3c2r4g3-development-shard-{shard}.json")
    } else {
        format!("target/phase-04c1b4c4r3c2r4g3-shard-{shard}.json")
    };
    std::fs::write(
        root().join(file),
        serde_json::to_vec_pretty(&receipt).unwrap(),
    )
    .unwrap();
}
#[test]
fn first_rich_production_source() {
    production_cases(1, true);
}
#[test]
fn production_source_failures_return_no_partial_carrier() {
    let case = read("cases.json")[0].clone();
    let bundle = read(case["bundle"].as_str().unwrap());
    let inputs = Inputs::make(&case);
    let context = inputs.borrow();
    let other_context = inputs.borrow();
    let current = current_compiler_semantics_identity();
    let mut receipts = Vec::new();
    for negative in read("negative-cases.json").as_array().unwrap() {
        let op = negative["operation"].as_str().unwrap();
        let mut package = bundle["package"].clone();
        let mut compiler = current.clone();
        let mut alternate_owner = package["owners"][0]["anchor"].clone();
        match op {
            "wrong-owner" => alternate_owner["owner_key"] = json!([120]),
            "wrong-revision" => alternate_owner["revision_key"] = json!([120]),
            "old-compiler" => {
                compiler.analysis_semantics_version = "agentflow.root-export-checker@7".into();
            }
            "wrong-compiler-content" => compiler.specifications[0].content.push(0),
            "missing-compiler-source" => {
                compiler.specifications.pop();
            }
            "duplicate-owner" => {
                let owner = package["owners"][0].clone();
                package["owners"].as_array_mut().unwrap().push(owner);
            }
            "foreign-domain" => {
                package["owners"][0]["anchor"]["domain"]["domain_key"] = json!([120]);
            }
            "unknown-container" => package["owners"][0]["container"] = json!({"future":{}}),
            "extra-package-field" => package["future"] = json!(true),
            "invalid-key-utf8" => package["owners"][0]["anchor"]["owner_key"] = json!([255]),
            "key-bound" => package["owners"][0]["anchor"]["owner_key"] = json!(vec![120; 4097]),
            "physical-duplicate-json" => {
                package["owners"][0]["container"]["declaration_bundle"]["declarations"][1]["document"]
                    ["physical_json_utf8"]["bytes"] =
                    json!(b"{\"selected\":null,\"selected\":null}".to_vec());
            }
            "physical-invalid-utf8" => {
                package["owners"][0]["container"]["declaration_bundle"]["declarations"][1]["document"]
                    ["physical_json_utf8"]["bytes"] = json!([255]);
            }
            _ => {}
        }
        let slot = usize::from(op.starts_with("physical-"));
        let document = if op == "unknown-container" {
            selected_document(&bundle["package"], 0, 0)
        } else {
            selected_document(&package, 0, slot)
        };
        let cloned_document = document.clone();
        let mut wrong_candidate = bundle["declarations"][0]["candidate"].clone();
        wrong_candidate["component_identity"]["name"] = json!([120]);
        let path = if slot == 1 {
            vec![RawDocumentMember::Field(b"selected".to_vec())]
        } else {
            vec![RawDocumentMember::Entry(if op == "wrong-entry" {
                u64::MAX
            } else {
                0
            })]
        };
        let wrong_path = vec![RawDocumentMember::Entry(u64::MAX)];
        let owner = if matches!(op, "wrong-owner" | "wrong-revision") {
            &alternate_owner
        } else {
            &package["owners"][0]["anchor"]
        };
        let physical = if op == "physical-not-supplied" {
            OccurrencePhysicalMetadata::NotSupplied
        } else if slot == 1 || op == "native-supplied" {
            OccurrencePhysicalMetadata::Supplied {
                source_owner: owner,
                entry_ordinal: slot as u64,
                actual_document: document,
                value_path: if op == "physical-wrong-path" {
                    &wrong_path
                } else {
                    &path
                },
            }
        } else {
            OccurrencePhysicalMetadata::NotSupplied
        };
        let request = DocumentOccurrenceMetadataRequest {
            source_package: if op == "missing-package" {
                None
            } else {
                Some(&package)
            },
            source_owner: if op == "missing-owner" {
                None
            } else {
                Some(owner)
            },
            entry_ordinal: if op == "missing-slot" {
                None
            } else {
                Some(slot as u64)
            },
            actual_document: if op == "clone-document" {
                &cloned_document
            } else {
                document
            },
            candidate: if op == "wrong-candidate" {
                &wrong_candidate
            } else {
                &bundle["declarations"][0]["candidate"]
            },
            internal_path: &path,
            physical,
        };
        let result = bind_raw_input_graph(
            &request,
            &context,
            if op == "split-context" {
                &other_context
            } else {
                &context
            },
            &compiler,
        );
        let Err(failure) = result else {
            panic!("source negative issued a carrier: {op}");
        };
        assert_eq!(
            failure.domain(),
            negative["domain"].as_str().unwrap(),
            "{op}"
        );
        assert_eq!(
            failure.reason(),
            negative["reason"].as_str().unwrap(),
            "{op}"
        );
        assert_eq!(failure.stage(), negative["stage"].as_str().unwrap(), "{op}");
        validate_structural_shape_metadata("StructuralBindingFailure", &failure.metadata())
            .unwrap();
        receipts.push(json!({"case_id":negative["case_id"],"failure":failure.metadata(),"graph_returned":false,"capability":false,"raw_fact":false}));
    }
    std::fs::write(
        root().join("target/phase-04c1b4c4r3c2r4g3-source-negative-actual.json"),
        serde_json::to_vec_pretty(&receipts).unwrap(),
    )
    .unwrap();
}
#[test]
fn production_resource_and_nominal_boundaries() {
    let case = read("cases.json")[0].clone();
    let bundle = read(case["bundle"].as_str().unwrap());
    let package = &bundle["package"];
    let document = selected_document(package, 0, 0);
    let path = [RawDocumentMember::Entry(0)];
    let compiler = current_compiler_semantics_identity();
    let mut receipts = Vec::new();
    for boundary in read("boundary-cases.json").as_array().unwrap() {
        let mut inputs = Inputs::make(&case);
        let op = boundary["operation"].as_str().unwrap();
        match op {
            "deep-native-schema" => {
                let mut expr = af_idl::CanonicalTypeExpr::Primitive(af_idl::Primitive::Bool);
                for _ in 0..100 {
                    expr = af_idl::CanonicalTypeExpr::List(Box::new(expr));
                }
                inputs
                    .schemas
                    .insert_type(&af_idl::CanonicalTypeSchema {
                        schema: "binder.deep@1".into(),
                        name: "Deep".into(),
                        body: af_idl::CanonicalBody::Alias(expr),
                    })
                    .unwrap();
            }
            "nfd-native-context" => {
                inputs
                    .flow
                    .sinks
                    .recognize_operation("binder.effects@1", "e\u{301}", false);
            }
            "work-exhaustion" => inputs.flow.sinks.recognize_operation(
                "binder.effects@1",
                &"x".repeat(70 * 1024 * 1024),
                false,
            ),
            _ => panic!("unknown declared boundary"),
        }
        let context = inputs.borrow();
        let source = DocumentOccurrenceMetadataRequest {
            source_package: Some(package),
            source_owner: Some(&package["owners"][0]["anchor"]),
            entry_ordinal: Some(0),
            actual_document: document,
            candidate: &bundle["declarations"][0]["candidate"],
            internal_path: &path,
            physical: OccurrencePhysicalMetadata::NotSupplied,
        };
        let Err(failure) = bind_raw_input_graph(&source, &context, &context, &compiler) else {
            panic!("boundary issued a graph: {op}");
        };
        assert_eq!(
            failure.domain(),
            boundary["domain"].as_str().unwrap(),
            "{op}"
        );
        assert_eq!(
            failure.reason(),
            boundary["reason"].as_str().unwrap(),
            "{op}"
        );
        assert_eq!(failure.stage(), boundary["stage"].as_str().unwrap(), "{op}");
        validate_structural_shape_metadata("StructuralBindingFailure", &failure.metadata())
            .unwrap();
        receipts.push(json!({"case_id":boundary["case_id"],"failure":failure.metadata(),"graph_returned":false,"capability":false,"raw_fact":false}));
    }
    std::fs::write(
        root().join("target/phase-04c1b4c4r3c2r4g3-boundary-actual.json"),
        serde_json::to_vec_pretty(&receipts).unwrap(),
    )
    .unwrap();
}
#[test]
fn production_cases_0() {
    production_cases(0, false);
}
#[test]
fn production_cases_1() {
    production_cases(1, false);
}
#[test]
fn production_cases_2() {
    production_cases(2, false);
}
#[test]
fn production_cases_3() {
    production_cases(3, false);
}
#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn modules() -> af_idl::CompiledModuleSet {
    let registry: Value = serde_json::from_slice(
        &std::fs::read(root().join("spec/registry/specifications.json")).unwrap(),
    )
    .unwrap();
    let sources: Vec<_> = registry["specifications"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|s| {
            let path = s["normative_source"]["logical_path"].as_str().unwrap();
            path.ends_with(".afidl").then(|| {
                (
                    path.to_owned(),
                    std::fs::read_to_string(root().join(path)).unwrap(),
                )
            })
        })
        .collect();
    af_idl::compile_module_set(&sources).unwrap()
}
#[test]
fn every_declared_vocabulary_member_is_an_actual_native_contract() {
    let rows: Vec<Value> = serde_json::from_slice(
        &std::fs::read(root().join("spec/conformance/production-input-graph/raw-vocabulary.json"))
            .unwrap(),
    )
    .unwrap();
    for row in rows {
        let native =
            serde_json::from_value::<af_contract::ComponentContract>(row["wire_candidate"].clone());
        assert!(
            native.is_ok(),
            "site {}: {}",
            row["source_site"],
            native.unwrap_err()
        );
    }
}
#[test]
fn first_actual_complete_native_input_graph() {
    let modules = modules();
    let schemas = af_semantic::TypeSchemaUniverse::new();
    let effects = af_effect::EffectCatalogUniverse::new();
    let resources = af_resource::ResourceTypeUniverse::new();
    let flow = af_flow::FlowCatalogs::new();
    let usage = af_usage::UsageMeterUniverse::new();
    let recovery = af_recovery::RecoveryOperationUniverse::new();
    let execution = af_execution::ExecutionUniverses::default();
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
        .current_dir(root())
        .output()
        .unwrap();
    assert!(generated.status.success());
    let inputs: Vec<Value> = serde_json::from_slice(&generated.stdout).unwrap();
    let input = inputs
        .iter()
        .find(|v| v["case_id"] == "c2r1/declaration_bundle/input-zero")
        .unwrap();
    let package = &input["package"];
    let document =
        &package["owners"][0]["container"]["declaration_bundle"]["declarations"][0]["document"];
    let path = [RawDocumentMember::Entry(0)];
    let source = DocumentOccurrenceMetadataRequest {
        source_package: Some(package),
        source_owner: Some(&package["owners"][0]["anchor"]),
        entry_ordinal: Some(0),
        actual_document: document,
        candidate: &input["candidate"],
        internal_path: &path,
        physical: OccurrencePhysicalMetadata::NotSupplied,
    };
    let compiler = current_compiler_semantics_identity();
    let graph = bind_raw_input_graph(&source, &context, &context, &compiler).unwrap();
    assert!(!graph.can_issue_root_export());
    assert!(std::ptr::eq(graph.context(), &raw const context));
    let af_core::Value::Record(claim) = graph.registered_output_claim() else {
        panic!("closed registered output claim");
    };
    assert_eq!(
        claim.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>(),
        vec!["environment", "source_assurance"]
    );
    assert_eq!(claim[1].1, af_core::Value::Enum("native_associated".into()));
    let af_core::Value::Record(environment) = &claim[0].1 else {
        panic!("closed environment");
    };
    assert_eq!(
        environment
            .iter()
            .find(|(k, _)| k == "witness_graph")
            .unwrap()
            .1,
        af_core::Value::Option(None)
    );
    assert_eq!(
        &environment
            .iter()
            .find(|(k, _)| k == "input_graph")
            .unwrap()
            .1,
        graph.value()
    );
    std::fs::write(
        root().join("target/phase-04c1b4c4r3c2r4g3-first-actual.ace1"),
        graph.canonical_bytes(),
    )
    .unwrap();
    std::fs::write(root().join("target/phase-04c1b4c4r3c2r4g3-first-actual.json"),serde_json::to_vec_pretty(&json!({"nodes":graph.node_count(),"sites":graph.structural_sites(),"input":input,"capability":false})).unwrap()).unwrap();
}
