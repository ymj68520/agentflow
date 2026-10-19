//! R5 boundary reproduction using frozen G3 native fixture constructors.
//! Copied fixture constructors only; no old production test/Expected is changed.
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

fn core_field<'a>(v: &'a af_core::Value, name: &str) -> &'a af_core::Value {
    let af_core::Value::Record(fields) = v else {
        panic!("registered record")
    };
    &fields.iter().find(|(n, _)| n == name).unwrap().1
}
fn core_list(v: &af_core::Value) -> &[af_core::Value] {
    let af_core::Value::List(rows) = v else {
        panic!("registered sequence")
    };
    rows
}
fn core_text(v: &af_core::Value) -> &str {
    let af_core::Value::String(s) = v else {
        panic!("registered text")
    };
    s
}
fn core_rank(v: &af_core::Value) -> usize {
    let af_core::Value::UInt64(n) = v else {
        panic!("registered DagRank")
    };
    usize::try_from(*n).unwrap()
}
fn record_child(nodes: &[af_core::Value], at: usize, field: &str) -> usize {
    let af_core::Value::Variant { case, value } = core_field(&nodes[at], "body") else {
        panic!("registered NodeBody")
    };
    assert_eq!(case, "record");
    let row = core_list(value)
        .iter()
        .find(|r| core_text(core_field(r, "field")) == field)
        .unwrap();
    core_rank(core_field(row, "value"))
}

#[test]
fn real_g_alias_c_source_nominal_boundary() {
    use af_computation::raw_capture::capture_raw_ports;
    use af_contract::{
        EffectResourceCompatibilityCatalog, InformationFlowExecutionCatalog,
        RecoveryUsageCoverageCatalog, RelationCatalogSnapshot,
    };
    let corpus: Value = serde_json::from_slice(
        &std::fs::read(
            root().join("docs/implementation/phase-04c1b4c4r3c2r1-raw-input-corpus.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let input = corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["case_id"] == "c2r1/declaration_bundle/input-zero")
        .unwrap();
    let package = &input["package"];
    let owner = usize::try_from(input["owner"].as_u64().unwrap()).unwrap();
    let slot = usize::try_from(input["slot"].as_u64().unwrap()).unwrap();
    let document = selected_document(package, owner, slot);
    let path = vec![RawDocumentMember::Entry(
        input["path_entry"].as_u64().unwrap(),
    )];
    let request = DocumentOccurrenceMetadataRequest {
        source_package: Some(package),
        source_owner: Some(&package["owners"][owner]["anchor"]),
        entry_ordinal: Some(slot as u64),
        actual_document: document,
        candidate: &input["candidate"],
        internal_path: &path,
        physical: OccurrencePhysicalMetadata::NotSupplied,
    };
    let compiler = current_compiler_semantics_identity();
    let mut receipts = Vec::new();
    for selected in [
        "effect_resource",
        "recovery_usage",
        "flow_execution",
        "none",
    ] {
        for mode in if selected == "none" {
            vec!["valid"]
        } else {
            vec!["valid", "missing"]
        } {
            let case = json!({"context":"empty","relations":"none","current_modules":"complete"});
            let mut inputs = Inputs::make(&case);
            let mut sources = all_sources();
            sources
                .retain(|(p, _)| p != "spec/conformance/production-input-graph/environment.afidl");
            let original = af_idl::compile_module_set(&sources).unwrap();
            match selected {
                "effect_resource" => {
                    inputs.relations.effect_resource = Some(
                        RelationCatalogSnapshot::new(
                            "g2-fixture",
                            EffectResourceCompatibilityCatalog::default(),
                            &original,
                        )
                        .unwrap(),
                    );
                }
                "recovery_usage" => {
                    inputs.relations.recovery_usage = Some(
                        RelationCatalogSnapshot::new(
                            "g2-fixture",
                            RecoveryUsageCoverageCatalog::default(),
                            &original,
                        )
                        .unwrap(),
                    );
                }
                "flow_execution" => {
                    inputs.relations.flow_execution = Some(
                        RelationCatalogSnapshot::new(
                            "g2-fixture",
                            InformationFlowExecutionCatalog::default(),
                            &original,
                        )
                        .unwrap(),
                    );
                }
                "none" => {}
                _ => unreachable!(),
            }
            if mode == "missing" {
                let name = match selected {
                    "effect_resource" => "effect-resource-compatibility.afidl",
                    "recovery_usage" => "recovery-usage-coverage.afidl",
                    "flow_execution" => "flow-execution-relation.afidl",
                    _ => unreachable!(),
                };
                sources.retain(|(p, _)| !p.ends_with(name));
            }
            inputs.modules = af_idl::compile_module_set(&sources).unwrap();
            let context = inputs.borrow();
            let graph = bind_raw_input_graph(&request, &context, &context, &compiler).unwrap();
            assert!(std::ptr::eq(graph.source(), &raw const request));
            assert!(std::ptr::eq(graph.context(), &raw const context));
            let node = graph.subject_node();
            assert!(std::ptr::eq(node.carrier(), &raw const graph));
            let nodes = core_list(core_field(graph.value(), "nodes"));
            let subject = usize::try_from(node.dag_rank()).unwrap();
            let context_rank =
                record_child(nodes, record_child(nodes, subject, "context"), "contents");
            let snapshots_rank = record_child(nodes, context_rank, "relation_snapshots");
            let actual_type = |rank: usize| json!({"owner":core_text(core_field(core_field(&nodes[rank],"type"),"owner")),"name":core_text(core_field(core_field(&nodes[rank],"type"),"name"))});
            let required =
                json!({"owner":STRUCTURAL_MODEL_ID,"name":"NativeRelationSnapshotSource"});
            let target_ranks: Vec<_> = (0..nodes.len())
                .filter(|r| actual_type(*r) == required)
                .collect();
            assert!(
                target_ranks.is_empty(),
                "frozen minimal G has no nominal NativeRelationSnapshotSource node"
            );
            let actual_source = if selected == "none" {
                Value::Null
            } else {
                let opt = record_child(nodes, snapshots_rank, selected);
                let af_core::Value::Variant { case, value } = core_field(&nodes[opt], "body")
                else {
                    panic!("option body")
                };
                assert_eq!(case, "option_some");
                let rank = core_rank(value);
                assert_eq!(
                    actual_type(rank),
                    json!({"owner":STRUCTURAL_MODEL_ID,"name":"RelationSnapshotContent"})
                );
                json!({"rank":rank,"type":actual_type(rank)})
            };
            // Real C is executed after G. The immutable graph is checked again
            // after C/port observations; no rank/node is added for their needs.
            let graph_before = graph.canonical_bytes().to_vec();
            let c = inspect_relation_context_metadata(&context, &context, &compiler).unwrap();
            assert_eq!(c.relation_partition_complete(), mode == "valid");
            let captured = capture_raw_ports(&request, &context, &context, &compiler).unwrap();
            assert_eq!(captured.seeds().len(), 1);
            let seed = &captured.seeds()[0];
            assert_eq!(seed.status(), af_contract::ProofVerdict::Violation);
            assert_eq!(
                seed.event().condition,
                af_contract::port_observation::PortCondition::Zero
            );
            assert_eq!(graph.canonical_bytes(), graph_before);
            assert!(!captured.can_issue_root_export());
            assert!(!c.can_issue_capability());
            assert!(!graph.can_issue_root_export());
            let id = format!("r5/nominal-boundary/{selected}/{mode}");
            let graph_file = format!("target/phase-04c1b4c4r3c2r5-{selected}-{mode}.ace1");
            std::fs::write(root().join(&graph_file), graph.canonical_bytes()).unwrap();
            receipts.push(json!({"case_id":id,"selected":selected,"mode":mode,"input":input,
                "graph":graph_file,"subject_rank":subject,"context_rank":context_rank,
                "actual_source":actual_source,"required_c_source_type":required,"matching_target_ranks":target_ranks,
                "c_metadata":c.metadata(),"same_source_object":true,"same_context_object":true,
                "same_subject_carrier":true,"g_unchanged_after_c_and_port":true,
                "port_event":{"direction":"input","original_ordinal":seed.event().occurrence.original_ordinal,
                    "sorted_ordinal":seed.event().occurrence.sorted_ordinal,"port_no":seed.event().occurrence.port_no,
                    "condition":"zero","status":"Violation","rule":captured.rule().rule_id},
                "complete_raw_fact_produced":false,"positive_root_export":false}));
        }
    }
    std::fs::write(
        root().join("target/phase-04c1b4c4r3c2r5-boundary-actual.json"),
        serde_json::to_vec_pretty(&receipts).unwrap(),
    )
    .unwrap();
}
