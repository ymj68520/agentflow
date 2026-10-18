//! Native input projection only. Does not execute C, ports or Fact production.
use crate::input_graph::Failure;
use af_contract::{RelationCatalogSnapshot, RelationCatalogWire, StaticContractProofContext};
use af_core::Value as V;
use serde_json::{Value as J, json};

type Result<T> = std::result::Result<T, Failure>;
fn error() -> Failure {
    Failure {
        domain: "InvalidFacts",
        reason: "source_association",
        path: vec![],
    }
}
fn rec(fields: Vec<(&str, V)>) -> V {
    V::Record(fields.into_iter().map(|(n, v)| (n.to_owned(), v)).collect())
}
fn text(s: &str) -> V {
    V::String(s.to_owned())
}
fn variant(case: &str, value: V) -> V {
    V::Variant {
        case: case.to_owned(),
        value: Box::new(value),
    }
}
fn native_wire(types: &J, d: &J, v: &J) -> Result<J> {
    if let Some(n) = d["ref"].as_str() {
        if n == "Raw($String)" {
            let bytes = v
                .as_array()
                .ok_or_else(error)?
                .iter()
                .map(|n| {
                    n.as_u64()
                        .and_then(|n| u8::try_from(n).ok())
                        .ok_or_else(error)
                })
                .collect::<Result<Vec<_>>>()?;
            return Ok(J::String(String::from_utf8(bytes).map_err(|_| error())?));
        }
        let mut result = native_wire(types, &types[n], v)?;
        if matches!(
            n,
            "Raw(af-contract::ComponentInputPort)" | "Raw(af-contract::ComponentOutputPort)"
        ) {
            let o = result.as_object_mut().ok_or_else(error)?;
            let t = o.remove("type_ref").ok_or_else(error)?;
            o.insert("type".into(), t);
        }
        if n == "Raw(af-semantic::PrimitiveConstant)"
            && let Some(tag) = result.as_str()
        {
            result = json!({tag:null});
        }
        return Ok(result);
    }
    Ok(match d["kind"].as_str().ok_or_else(error)? {
        "record" => J::Object(
            d["fields"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| {
                    let n = r[0].as_str().unwrap();
                    Ok((n.to_owned(), native_wire(types, &r[1], &v[n])?))
                })
                .collect::<Result<_>>()?,
        ),
        "enum" => {
            let (tag, payload) = if let Some(tag) = v.as_str() {
                (tag, &J::Null)
            } else {
                let o = v.as_object().ok_or_else(error)?;
                let (k, v) = o.iter().next().ok_or_else(error)?;
                (k.as_str(), v)
            };
            let desc = &d["variants"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r[0] == tag)
                .ok_or_else(error)?[1];
            let mut lower = String::new();
            for (i, c) in tag.chars().enumerate() {
                if i != 0 && c.is_ascii_uppercase() {
                    lower.push('_');
                }
                lower.push(c.to_ascii_lowercase());
            }
            if desc["kind"] == "unit" {
                J::String(lower)
            } else {
                let mut value = native_wire(types, desc, payload)?;
                if value
                    .as_object()
                    .is_some_and(|o| o.len() == 1 && o.contains_key("0"))
                {
                    value = value["0"].clone();
                }
                json!({lower:value})
            }
        }
        "option" => {
            if v == "none" || v.get("none").is_some() {
                J::Null
            } else {
                native_wire(types, &d["item"], v.get("some").ok_or_else(error)?)?
            }
        }
        "sequence" | "set" => J::Array(
            v.as_array()
                .ok_or_else(error)?
                .iter()
                .map(|v| native_wire(types, &d["item"], v))
                .collect::<Result<_>>()?,
        ),
        _ => v.clone(),
    })
}
/// Pure native representability/parse association, not canonical validation.
/// The full supplied package is not reconstructed from the candidate.
pub(crate) fn native_source(package: &J, candidate: &J) -> Result<af_contract::ComponentContract> {
    let m = crate::structural_model_catalogue().map_err(|_| error())?;
    let parse = |v: &J| {
        serde_json::from_value::<af_contract::ComponentContract>(native_wire(
            &m["types"],
            &m["types"]["RawCandidateContract"],
            v,
        )?)
        .map_err(|_| error())
    };
    for owner in package["owners"].as_array().ok_or_else(error)? {
        let (tag, body) = owner["container"]
            .as_object()
            .ok_or_else(error)?
            .iter()
            .next()
            .ok_or_else(error)?;
        let name = match tag.as_str() {
            "declaration_bundle" => "declarations",
            "import_manifest" => "imports",
            "workflow_source_collection" => "sources",
            _ => return Err(error()),
        };
        for e in body[name].as_array().ok_or_else(error)? {
            if let Some(doc) = e["document"].get("declared_native_document") {
                for declaration in doc["declarations"].as_array().ok_or_else(error)? {
                    parse(declaration)?;
                }
            }
        }
    }
    parse(candidate)
}
pub(crate) fn from_metadata(name: &str, v: &J) -> Result<V> {
    let m = crate::structural_model_catalogue().map_err(|_| error())?;
    convert(&m["types"], &m["types"][name], v, 0)
}
fn convert(types: &J, d: &J, v: &J, depth: u64) -> Result<V> {
    if depth > 96 {
        return Err(Failure {
            domain: "ResourceLimit",
            reason: "resource_bound",
            path: vec![],
        });
    }
    if let Some(n) = d["ref"].as_str() {
        return convert(types, &types[n], v, depth);
    }
    Ok(match d["kind"].as_str().ok_or_else(error)? {
        "dag_ref" => return convert(types, &d["target"], v, depth + 1),
        "record" => {
            let fields = d["fields"].as_array().ok_or_else(error)?;
            if v.as_object().is_none_or(|o| o.len() != fields.len()) {
                return Err(error());
            }
            V::Record(
                fields
                    .iter()
                    .map(|r| {
                        let name = r[0].as_str().ok_or_else(error)?;
                        Ok((
                            name.to_owned(),
                            convert(types, &r[1], v.get(name).ok_or_else(error)?, depth + 1)?,
                        ))
                    })
                    .collect::<Result<_>>()?,
            )
        }
        "enum" => {
            let (tag, payload) = if let Some(tag) = v.as_str() {
                (tag, &J::Null)
            } else {
                let o = v.as_object().filter(|o| o.len() == 1).ok_or_else(error)?;
                let (k, v) = o.iter().next().unwrap();
                (k.as_str(), v)
            };
            let row = d["variants"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r[0] == tag)
                .ok_or_else(error)?;
            variant(tag, convert(types, &row[1], payload, depth + 1)?)
        }
        "option" => {
            if v == "none" || v == &json!({"none":null}) {
                V::Option(None)
            } else {
                if v.as_object().is_none_or(|o| o.len() != 1) {
                    return Err(error());
                }
                V::Option(Some(Box::new(convert(
                    types,
                    &d["item"],
                    v.get("some").ok_or_else(error)?,
                    depth + 1,
                )?)))
            }
        }
        "sequence" | "set" => V::List(
            v.as_array()
                .ok_or_else(error)?
                .iter()
                .map(|v| convert(types, &d["item"], v, depth + 1))
                .collect::<Result<_>>()?,
        ),
        "map" => V::List(
            v.as_array()
                .ok_or_else(error)?
                .iter()
                .map(|r| {
                    let p = r.as_array().filter(|p| p.len() == 2).ok_or_else(error)?;
                    Ok(V::List(vec![
                        convert(types, &d["key"], &p[0], depth + 1)?,
                        convert(types, &d["value"], &p[1], depth + 1)?,
                    ]))
                })
                .collect::<Result<_>>()?,
        ),
        "unit" => {
            if !v.is_null() {
                return Err(error());
            }
            V::Unit
        }
        "null" => {
            if !v.is_null() {
                return Err(error());
            }
            V::Null
        }
        "bool" => V::Bool(v.as_bool().ok_or_else(error)?),
        "u8" | "u32" => V::UInt32(
            v.as_u64()
                .and_then(|n| u32::try_from(n).ok())
                .ok_or_else(error)?,
        ),
        "u64" => V::UInt64(v.as_u64().ok_or_else(error)?),
        "i32" => V::Int32(
            v.as_i64()
                .and_then(|n| i32::try_from(n).ok())
                .ok_or_else(error)?,
        ),
        "i64" => V::Int64(v.as_i64().ok_or_else(error)?),
        "nfc_text" => text(v.as_str().ok_or_else(error)?),
        "bytes" => V::Bytes(
            v.as_array()
                .ok_or_else(error)?
                .iter()
                .map(|v| {
                    v.as_u64()
                        .and_then(|n| u8::try_from(n).ok())
                        .ok_or_else(error)
                })
                .collect::<Result<_>>()?,
        ),
        "decimal" => {
            V::Decimal(af_core::Decimal::parse(v.as_str().ok_or_else(error)?).map_err(|_| error())?)
        }
        "timestamp_i64" => V::Timestamp(af_core::Timestamp::new(v.as_i64().ok_or_else(error)?)),
        "duration_i64" => V::Duration(af_core::Duration::new(v.as_i64().ok_or_else(error)?)),
        _ => return Err(error()),
    })
}

fn source(s: &crate::SpecificationContentIdentity) -> V {
    rec(vec![
        ("specification_id", text(&s.specification_id)),
        ("logical_path", text(&s.logical_path)),
        ("content_sha256", text(&s.content_sha256)),
        ("content", V::Bytes(s.content.clone())),
    ])
}
fn rule(r: &crate::RuleContentIdentity) -> V {
    rec(vec![
        ("rule_id", text(&r.rule_id)),
        ("normative_source", text(&r.normative_source)),
        ("source_anchor", text(&r.source_anchor)),
        ("short_statement", text(&r.short_statement)),
    ])
}
// Registered logical paths are case-sensitive source identity, not host paths.
#[allow(clippy::case_sensitive_file_extension_comparisons)]
pub(crate) fn compiler(edition: &crate::RegisteredAnalysisEdition) -> Result<V> {
    if edition.registration != crate::EditionRegistration::Registered {
        return Err(error());
    }
    let sources: Vec<_> = edition
        .registered_sources
        .iter()
        .filter(|s| s.logical_path.ends_with(".afidl"))
        .map(|s| {
            Ok((
                s.logical_path.clone(),
                String::from_utf8(s.content.clone()).map_err(|_| error())?,
            ))
        })
        .collect::<Result<_>>()?;
    let modules = af_idl::compile_module_set(&sources).map_err(|_| error())?;
    let mut pins = Vec::new();
    for m in modules.modules() {
        for t in &m.types {
            let digest = t.digest().map_err(|_| error())?;
            pins.push(json!({"specification_id":t.schema,"type_name":t.name,"schema_digest":{"algorithm":"sha256","domain":"type-schema","hex":digest.hex()}}));
        }
    }
    pins.sort_by(|a, b| {
        a["specification_id"]
            .as_str()
            .cmp(&b["specification_id"].as_str())
            .then_with(|| a["type_name"].as_str().cmp(&b["type_name"].as_str()))
    });
    let specs: Vec<_> = edition
        .compiler
        .specifications
        .iter()
        .map(|s| s.specification_id.clone())
        .collect();
    let roles = [
        "static_analysis",
        "contract_validation",
        "semantic_ir_validation",
        "dataflow",
        "leaf_resolution",
        "control_algebra",
        "root_recovery",
        "root_export",
        "plan_lowering",
        "runtime_semantics_profile",
    ];
    let mut role_rows = Vec::new();
    for name in roles {
        let dependencies: &[&str] = match name {
            "static_analysis" => &[
                "contract_validation",
                "semantic_ir_validation",
                "dataflow",
                "leaf_resolution",
                "control_algebra",
                "root_recovery",
                "root_export",
            ],
            "dataflow" => &["semantic_ir_validation"],
            "leaf_resolution" => &["contract_validation", "semantic_ir_validation", "dataflow"],
            "control_algebra" => &["leaf_resolution"],
            "root_recovery" => &["dataflow", "control_algebra"],
            "root_export" => &[
                "contract_validation",
                "semantic_ir_validation",
                "dataflow",
                "leaf_resolution",
                "control_algebra",
                "root_recovery",
            ],
            _ => &[],
        };
        let declaration = if matches!(name, "plan_lowering" | "runtime_semantics_profile") {
            json!({"model_missing":"not_formalized"})
        } else {
            json!({"defined":{"analysis_semantics_version":crate::ROOT_EXPORT_ANALYSIS_SEMANTICS_VERSION,"normative_specifications":specs,"required_schema_pins":pins,"depends_on":dependencies}})
        };
        role_rows.push(from_metadata(
            "agentflow.portable.proof.inputs@1/SemanticRoleEntry",
            &json!({"role":name,"declaration":declaration}),
        )?);
    }
    let closed = rec(edition
        .closed_tables
        .iter()
        .map(|(name, bytes)| (*name, V::Bytes(bytes.clone())))
        .collect());
    // Closed-field order is the registered declaration order, not BTreeMap order.
    let V::Record(mut closed) = closed else {
        unreachable!()
    };
    let order = [
        "hash_domains",
        "abi_numbers",
        "rule_areas",
        "afidl_primitives",
    ];
    closed.sort_by_key(|(name, _)| order.iter().position(|n| name == n).unwrap());
    let support = rec(vec![
        (
            "normative_sources",
            V::List(edition.registered_sources.iter().map(source).collect()),
        ),
        ("closed_tables", V::Record(closed)),
        (
            "selected_specifications",
            V::List(specs.iter().map(|s| text(s)).collect()),
        ),
        (
            "applied_rules",
            V::List(edition.compiler.rules.iter().map(rule).collect()),
        ),
        ("semantic_roles", V::List(role_rows)),
        (
            "attribution",
            variant(
                "full_content_revalidation",
                rec(vec![(
                    "listed_rules",
                    V::List(edition.compiler.rules.iter().map(rule).collect()),
                )]),
            ),
        ),
        ("report_shape", text(edition.report_shape)),
        ("obligation_edition", text(edition.obligation_edition)),
    ]);
    Ok(rec(vec![
        (
            "analysis_semantics_version",
            text(&edition.compiler.analysis_semantics_version),
        ),
        (
            "registry_content",
            V::Bytes(edition.compiler.registry_content.clone()),
        ),
        (
            "rule_registry_content",
            V::Bytes(edition.compiler.rule_registry_content.clone()),
        ),
        (
            "specifications",
            V::List(edition.compiler.specifications.iter().map(source).collect()),
        ),
        (
            "rules",
            V::List(edition.compiler.rules.iter().map(rule).collect()),
        ),
        ("support_manifest", support),
    ]))
}

fn schema_entry(entry: &af_semantic::SchemaEntry) -> J {
    crate::relation_schema_definition_metadata(&af_idl::CanonicalTypeSchema {
        schema: entry.specification_id.clone(),
        name: entry.type_name.clone(),
        body: entry.body.clone(),
    })
}
fn reference(entry: &af_semantic::SchemaEntry) -> J {
    json!({"specification_id":entry.specification_id,"type_name":entry.type_name,"schema_digest":{"algorithm":entry.digest.alg().as_str(),"domain":entry.digest.domain().as_str(),"hex":entry.digest.hex()}})
}
fn operation(catalog: &str, name: &str) -> J {
    json!({"catalog":catalog,"name":name})
}
fn effects(e: &af_effect::EffectCatalogUniverse) -> J {
    json!(
        e.catalogs()
            .map(|c| json!([c.specification_id(), c.operations().collect::<Vec<_>>()]))
            .collect::<Vec<_>>()
    )
}
fn native_relation<T: RelationCatalogWire>(
    slot: &str,
    s: Option<&RelationCatalogSnapshot<T>>,
) -> V {
    V::Option(s.map(|s| {
        Box::new(rec(vec![
            ("kind", V::Enum(slot.to_owned())),
            ("schema_family", text(s.schema())),
            ("revision", text(s.revision())),
            (
                "pin",
                rec(vec![
                    ("family", text(&s.pin().schema)),
                    ("revision", text(&s.pin().revision)),
                    (
                        "canonical_content_checksum",
                        rec(vec![
                            (
                                "algorithm",
                                text(s.pin().canonical_content_checksum.algorithm),
                            ),
                            ("hex", text(&s.pin().canonical_content_checksum.hex)),
                        ]),
                    ),
                ]),
            ),
            ("canonical_content", V::Bytes(s.canonical_bytes().to_vec())),
            ("catalog", variant(slot, s.catalog().to_value())),
        ]))
    }))
}
pub(crate) fn context(c: &StaticContractProofContext<'_>) -> Result<V> {
    native_context_limits(c)?;
    let mut modules = Vec::new();
    for m in c.modules.modules() {
        let key = m.specification.canonical();
        if !c
            .modules
            .module(&key)
            .is_some_and(|found| std::ptr::eq(found, m))
        {
            return Err(error());
        }
        modules.push(json!([key,{"specification_id":key,"types":m.types.iter().map(crate::relation_schema_definition_metadata).collect::<Vec<_>>()}]));
    }
    if modules.len() != c.modules.len() {
        return Err(error());
    }
    let schemas = |u: &af_semantic::TypeSchemaUniverse| {
        json!(
            u.entries()
                .map(|e| json!([reference(e), schema_entry(e)]))
                .collect::<Vec<_>>()
        )
    };
    let mut resource_descriptors = Vec::new();
    for d in c.resources.descriptors() {
        let schema = c
            .resources
            .schema_universe()
            .get(&format!("{}/{}", d.specification_id, d.type_name))
            .ok_or_else(error)?;
        if schema.digest.hex() != d.schema_hex {
            return Err(error());
        }
        resource_descriptors.push(json!([reference(schema),{"specification_id":d.specification_id,"type_name":d.type_name,"schema_hex":d.schema_hex,"recognized_attributes":d.recognized_attributes}]));
    }
    let execution = &c.execution;
    let value = json!({"compiled_modules":modules,"type_schemas":schemas(c.schema_universe),"effect_catalogs":effects(c.effects),
        "resource_types":{"schemas":schemas(c.resources.schema_universe()),"descriptors":resource_descriptors},
        "flow_catalogs":{"confidentiality":c.flow.labels.confidentiality_labels().map(|(c,n)|operation(c,n)).collect::<Vec<_>>(),"taints":c.flow.labels.taints().map(|(c,n)|operation(c,n)).collect::<Vec<_>>(),"sinks":c.flow.sinks.operations().map(|(c,n,b)|json!([operation(c,n),b])).collect::<Vec<_>>()},
        "usage_meters":c.usage_meters.descriptors().map(|d|json!([operation(&d.meter.catalog,&d.meter.name),{"meter":operation(&d.meter.catalog,&d.meter.name),"unit":operation(&d.unit.catalog,&d.unit.name),"aggregation":match d.aggregation{af_usage::UsageAggregation::Additive=>"Additive",af_usage::UsageAggregation::Unsupported=>"Unsupported"}}])).collect::<Vec<_>>(),
        "recovery_operations":{"effects":effects(&c.recovery.effects),"reconciliation_operations":c.recovery.reconciliation_operations().map(|(c,n)|operation(c,n)).collect::<Vec<_>>()},
        "execution_universes":{"capabilities":execution.capabilities.capabilities.iter().map(|(k,d)|json!([k,{"capability":d.capability,"exact_version":d.exact_version,"supported_versions":d.supported_versions,"attributes":d.attributes.iter().map(|(k,v)|json!({"key":k,"value":v})).collect::<Vec<_>>()}])).collect::<Vec<_>>(),
            "trust_domains":execution.trust_domains.domains().collect::<Vec<_>>(),"regions":execution.regions.regions().collect::<Vec<_>>(),"region_compatibility":execution.regions.compatibility().map(|(f,t)|json!({"from":f,"to":t})).collect::<Vec<_>>(),
            "runtime_profiles":execution.runtime_profiles.profiles().map(|d|json!([d.profile,{"profile":d.profile,"family":d.family,"features":d.features}])).collect::<Vec<_>>()},
        "relation_snapshots":{"effect_resource":{"none":null},"recovery_usage":{"none":null},"flow_execution":{"none":null}}});
    let mut projected = from_metadata("ContractProofContextContents", &value)?;
    let V::Record(fields) = &mut projected else {
        return Err(error());
    };
    let relation = fields
        .iter_mut()
        .find(|(n, _)| n == "relation_snapshots")
        .ok_or_else(error)?;
    relation.1 = rec(vec![
        (
            "effect_resource",
            native_relation("effect_resource", c.relations.effect_resource.as_ref()),
        ),
        (
            "recovery_usage",
            native_relation("recovery_usage", c.relations.recovery_usage.as_ref()),
        ),
        (
            "flow_execution",
            native_relation("flow_execution", c.relations.flow_execution.as_ref()),
        ),
    ]);
    Ok(rec(vec![
        ("contents", projected),
        (
            "roles",
            V::List(vec![V::List(vec![
                V::Enum("analysis".into()),
                V::Enum("canonical_validation".into()),
            ])]),
        ),
    ]))
}

/// Bound native expansion BEFORE any recursive schema projection or large
/// metadata allocation. These are resource checks, not C schema validation.
fn native_context_limits(c: &StaticContractProofContext<'_>) -> Result<()> {
    struct Budget {
        work: u64,
    }
    impl Budget {
        fn add(&mut self, n: u64) -> Result<()> {
            self.work = self.work.checked_add(n).ok_or_else(|| Failure {
                domain: "ResourceLimit",
                reason: "resource_bound",
                path: vec![],
            })?;
            if self.work > 4_194_304 {
                return Err(Failure {
                    domain: "ResourceLimit",
                    reason: "resource_bound",
                    path: vec![],
                });
            }
            Ok(())
        }
        fn text(&mut self, s: &str) -> Result<()> {
            if s.len() > 134_217_728 {
                return Err(Failure {
                    domain: "ResourceLimit",
                    reason: "resource_bound",
                    path: vec![],
                });
            }
            self.add((s.len() as u64).div_ceil(64) + 1)
        }
        fn array(&mut self, n: usize) -> Result<()> {
            if n > 262_144 {
                return Err(Failure {
                    domain: "ResourceLimit",
                    reason: "resource_bound",
                    path: vec![],
                });
            }
            self.add(n as u64)
        }
        fn body(&mut self, body: &af_idl::CanonicalBody) -> Result<()> {
            use af_idl::{CanonicalBody as B, CanonicalTypeExpr as E};
            let mut todo = Vec::new();
            match body {
                B::Alias(e) => todo.push((e, 0)),
                B::Enum(v) => {
                    self.array(v.len())?;
                    for r in v {
                        self.text(&r.name)?;
                    }
                }
                B::Record(v) => {
                    self.array(v.len())?;
                    for r in v {
                        self.text(&r.name)?;
                        todo.push((&r.type_expr, 0));
                    }
                }
                B::Variant(v) => {
                    self.array(v.len())?;
                    for r in v {
                        self.text(&r.name)?;
                        todo.push((&r.type_expr, 0));
                    }
                }
            }
            while let Some((e, depth)) = todo.pop() {
                self.add(1)?;
                if depth > 96 {
                    return Err(Failure {
                        domain: "ResourceLimit",
                        reason: "resource_bound",
                        path: vec![],
                    });
                }
                match e {
                    E::Primitive(_) => {}
                    E::NamedRef(n) => self.text(n)?,
                    E::List(e) | E::MapString(e) | E::Option(e) => todo.push((e, depth + 1)),
                }
            }
            Ok(())
        }
        fn relation(&mut self, bytes: &[u8]) -> Result<()> {
            if bytes.len() > 134_217_728 {
                return Err(Failure {
                    domain: "ResourceLimit",
                    reason: "resource_bound",
                    path: vec![],
                });
            }
            self.add((bytes.len() as u64).div_ceil(64))?;
            let (mut quoted, mut escaped) = (false, false);
            for b in bytes {
                if quoted {
                    if escaped {
                        escaped = false;
                    } else if *b == b'\\' {
                        escaped = true;
                    } else if *b == b'"' {
                        quoted = false;
                    }
                } else if *b == b'"' {
                    quoted = true;
                    self.add(1)?;
                } else if matches!(*b, b'[' | b'{' | b't' | b'f' | b'n') {
                    self.add(1)?;
                }
            }
            Ok(())
        }
    }
    let mut b = Budget { work: 0 };
    b.array(c.modules.len().checked_mul(2).ok_or_else(error)?)?;
    for m in c.modules.modules() {
        b.text(&m.specification.canonical())?;
        b.array(m.types.len())?;
        for t in &m.types {
            b.text(&t.schema)?;
            b.text(&t.name)?;
            b.body(&t.body)?;
        }
    }
    for u in [c.schema_universe, c.resources.schema_universe()] {
        for e in u.entries() {
            b.add(1)?;
            b.text(&e.specification_id)?;
            b.text(&e.type_name)?;
            b.body(&e.body)?;
        }
    }
    for d in c.resources.descriptors() {
        b.add(1)?;
        b.text(&d.specification_id)?;
        b.text(&d.type_name)?;
        b.text(&d.schema_hex)?;
        b.array(d.recognized_attributes.len())?;
        for s in &d.recognized_attributes {
            b.text(s)?;
        }
    }
    for e in [c.effects, &c.recovery.effects] {
        for cat in e.catalogs() {
            b.add(1)?;
            b.text(cat.specification_id())?;
            for op in cat.operations() {
                b.add(1)?;
                b.text(op)?;
            }
        }
    }
    for (catalog, name) in c
        .flow
        .labels
        .confidentiality_labels()
        .chain(c.flow.labels.taints())
        .chain(c.recovery.reconciliation_operations())
    {
        b.add(1)?;
        b.text(catalog)?;
        b.text(name)?;
    }
    for (catalog, name, _) in c.flow.sinks.operations() {
        b.add(1)?;
        b.text(catalog)?;
        b.text(name)?;
    }
    for d in c.usage_meters.descriptors() {
        b.add(1)?;
        for s in [
            &d.meter.catalog,
            &d.meter.name,
            &d.unit.catalog,
            &d.unit.name,
        ] {
            b.text(s)?;
        }
    }
    b.array(
        c.execution
            .capabilities
            .capabilities
            .len()
            .checked_mul(2)
            .ok_or_else(error)?,
    )?;
    for (k, d) in &c.execution.capabilities.capabilities {
        for s in [
            &k.catalog,
            &k.name,
            &d.capability.catalog,
            &d.capability.name,
            &d.exact_version,
        ] {
            b.text(s)?;
        }
        b.array(d.supported_versions.len())?;
        for s in &d.supported_versions {
            b.text(s)?;
        }
        b.array(d.attributes.len())?;
        for (k, v) in &d.attributes {
            b.text(k)?;
            b.text(v)?;
        }
    }
    for d in c.execution.trust_domains.domains() {
        b.add(1)?;
        b.text(&d.catalog)?;
        b.text(&d.name)?;
    }
    for d in c.execution.regions.regions() {
        b.add(1)?;
        b.text(&d.catalog)?;
        b.text(&d.name)?;
    }
    for (f, t) in c.execution.regions.compatibility() {
        b.add(1)?;
        for s in [&f.catalog, &f.name, &t.catalog, &t.name] {
            b.text(s)?;
        }
    }
    for d in c.execution.runtime_profiles.profiles() {
        b.add(1)?;
        for s in [&d.profile.catalog, &d.profile.name, &d.family] {
            b.text(s)?;
        }
        b.array(d.features.len())?;
        for s in &d.features {
            b.text(s)?;
        }
    }
    if let Some(s) = &c.relations.effect_resource {
        b.relation(s.canonical_bytes())?;
    }
    if let Some(s) = &c.relations.recovery_usage {
        b.relation(s.canonical_bytes())?;
    }
    if let Some(s) = &c.relations.flow_execution {
        b.relation(s.canonical_bytes())?;
    }
    Ok(())
}
