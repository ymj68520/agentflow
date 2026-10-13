use crate::error::{PortableProofError, ValidationStage};
use af_core::{Shape, Value, encode};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum OrderKey {
    Number(u64),
    Text(String),
    Tuple(Vec<OrderKey>),
}

#[derive(Debug)]
pub(crate) struct SetRule {
    pub owner: &'static str,
    pub field: &'static str,
    pub key: &'static str,
    pub element_type: &'static str,
    pub complete_collection: bool,
}

include!(concat!(env!("OUT_DIR"), "/registered_set_rules.rs"));

pub(crate) fn validate_registered_orders(
    shape: &Shape,
    value: &Value,
) -> Result<(), PortableProofError> {
    walk(shape, value, "$")
}

fn walk(shape: &Shape, value: &Value, path: &str) -> Result<(), PortableProofError> {
    match (shape, value) {
        (Shape::Record(record), Value::Record(fields)) => {
            for declared in &record.fields {
                if let Some((_, child)) = fields.iter().find(|(name, _)| name == &declared.name) {
                    validate_set_field(&record.name, &declared.name, &declared.shape, child, path)?;
                    walk(&declared.shape, child, &format!("{path}.{}", declared.name))?;
                }
            }
        }
        (Shape::Variant(variant), Value::Variant { case, value }) => {
            let Some(declared) = variant.cases.iter().find(|entry| entry.name == *case) else {
                return Err(error(
                    ValidationStage::D0,
                    "UnknownVariant",
                    path,
                    "unknown variant",
                ));
            };
            if variant.name.starts_with("PpiAux") && case == "complete" {
                if let Shape::List(element) = &declared.payload {
                    validate_complete_field(&variant.name, element, value, path)?;
                }
            }
            walk(&declared.payload, value, &format!("{path}.{case}"))?;
        }
        (Shape::Option(inner), Value::Option(Some(child))) => walk(inner, child, path)?,
        (Shape::List(inner), Value::List(items)) => {
            for (index, item) in items.iter().enumerate() {
                walk(inner, item, &format!("{path}[{index}]"))?;
            }
        }
        (Shape::Map(inner), Value::Map(items)) => {
            for (key, item) in items {
                walk(inner, item, &format!("{path}.{key}"))?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_set_field(
    owner: &str,
    field: &str,
    shape: &Shape,
    value: &Value,
    path: &str,
) -> Result<(), PortableProofError> {
    let Some(rule) = REGISTERED_SET_RULES
        .iter()
        .find(|r| r.owner == owner && r.field == field && !r.complete_collection)
    else {
        return Ok(());
    };
    validate_set(shape, value, rule.key, path)
}

fn validate_complete_field(
    aux: &str,
    element: &Shape,
    value: &Value,
    path: &str,
) -> Result<(), PortableProofError> {
    let Some(rule) = REGISTERED_SET_RULES.iter().find(|r| {
        r.complete_collection
            && r.owner.chars().next().is_some()
            && aux.contains(r.owner.rsplit('/').next().unwrap_or_default())
            && element_type(r.element_type, element)
    }) else {
        return Err(error(
            ValidationStage::D4,
            "UnsupportedCurrentSchema",
            path,
            "unregistered CompleteCollection element order",
        ));
    };
    validate_set(
        &Shape::List(Box::new(element.clone())),
        value,
        rule.key,
        path,
    )
}

fn element_type(name: &str, shape: &Shape) -> bool {
    match shape {
        Shape::Record(record) => {
            record.name == name || (name == "SchemaRef" && record.name == "TypeSchemaRef")
        }
        Shape::Enum(enumeration) => enumeration.name == name,
        Shape::Variant(variant) => variant.name == name,
        _ => false,
    }
}

fn validate_set(
    shape: &Shape,
    value: &Value,
    key_kind: &str,
    path: &str,
) -> Result<(), PortableProofError> {
    let Value::List(items) = value else {
        return Ok(());
    };
    let element_shape = match shape {
        Shape::List(element) => element.as_ref(),
        element => element,
    };
    let mut previous: Option<(OrderKey, Vec<u8>)> = None;
    for (index, item) in items.iter().enumerate() {
        let key = primary_key(key_kind, item).ok_or_else(|| {
            error(
                ValidationStage::D4,
                "UnsupportedCurrentSchema",
                path,
                &format!("unsupported registered set key {key_kind}"),
            )
        })?;
        let bytes = encode(element_shape, item).map_err(|e| {
            error(
                ValidationStage::D1,
                "NoncanonicalOrder",
                path,
                &e.to_string(),
            )
        })?;
        if let Some(previous_key) = &previous {
            match previous_key.cmp(&(key.clone(), bytes.clone())) {
                std::cmp::Ordering::Less => {}
                std::cmp::Ordering::Equal => {
                    return Err(error(
                        ValidationStage::D1,
                        "DuplicateSetMember",
                        &format!("{path}[{index}]"),
                        "duplicate set member",
                    ));
                }
                std::cmp::Ordering::Greater => {
                    return Err(error(
                        ValidationStage::D1,
                        "NoncanonicalOrder",
                        &format!("{path}[{index}]"),
                        "set is not in registered order",
                    ));
                }
            }
        }
        previous = Some((key, bytes));
    }
    Ok(())
}

fn primary_key(kind: &str, value: &Value) -> Option<OrderKey> {
    match kind {
        "id" | "numeric" => match value {
            Value::UInt32(number) => Some(OrderKey::Number(u64::from(*number))),
            Value::UInt64(number) => Some(OrderKey::Number(*number)),
            Value::Record(_) if kind == "id" => field(value, "id").and_then(number_key),
            _ => None,
        },
        "specification_id" | "rule_id" => {
            field(value, kind).and_then(scalar_text).map(OrderKey::Text)
        }
        "local_type_name" => field(value, "type_name")
            .and_then(scalar_text)
            .map(OrderKey::Text),
        "utf8" => scalar_text(value).map(OrderKey::Text),
        "RoleRank" => {
            let role = match value {
                Value::Record(_) => scalar_text(field(value, "role")?)?,
                value => scalar_text(value)?,
            };
            let rank = match role.as_str() {
                "static_analysis" => 1,
                "contract_validation" => 2,
                "semantic_ir_validation" => 3,
                "dataflow" => 4,
                "leaf_resolution" => 5,
                "control_algebra" => 6,
                "root_recovery" => 7,
                "root_export" => 8,
                "plan_lowering" => 9,
                "runtime_semantics_profile" => 10,
                _ => return None,
            };
            Some(OrderKey::Number(rank))
        }
        "SchemaRefKey" | "SchemaRef" => schema_key(value),
        "TypedContentRefKey" => typed_key(value),
        "target_digest_key" => field(value, "target_semantic_ir_digest")
            .or_else(|| field(value, "semantic_ir_digest"))
            .and_then(digest_key),
        "DigestKey" => digest_key(value),
        "(catalog,name)" => pair_key(value, "catalog", "name"),
        "(from,to)" => {
            let from = field(value, "from").and_then(|item| pair_key(item, "catalog", "name"))?;
            let to = field(value, "to").and_then(|item| pair_key(item, "catalog", "name"))?;
            Some(OrderKey::Tuple(vec![from, to]))
        }
        "meter" => field(value, "meter").and_then(|v| pair_key(v, "catalog", "name")),
        "capability" => field(value, "capability").and_then(|v| pair_key(v, "catalog", "name")),
        "profile" => field(value, "profile").and_then(|v| pair_key(v, "catalog", "name")),
        "operation" => field(value, "operation").and_then(|v| pair_key(v, "catalog", "name")),
        "key,value" => pair_key(value, "key", "value"),
        "dependent,prerequisite" => pair_key(value, "dependent", "prerequisite"),
        "caller_digest,source_node" => {
            let digest = field(field(value, "caller_ir")?, "digest").and_then(digest_key)?;
            Some(OrderKey::Tuple(vec![
                digest,
                number_key(field(value, "source_node")?)?,
            ]))
        }
        "InputMappingKey" => map_key(
            value,
            &[
                ("export_input_port", Some("port_no")),
                ("run_parameter_id", None),
                ("root_value_slot", None),
            ],
        ),
        "OutputMappingKey" => map_key(
            value,
            &[
                ("export_output_port", Some("port_no")),
                ("workflow_output_name", None),
                ("workflow_output_expression", None),
            ],
        ),
        "ResourceMappingKey" => map_key(
            value,
            &[
                ("export_resource_port", Some("port_no")),
                ("root_resource_slot", None),
            ],
        ),
        "ResourceSourceKey" => map_key(
            value,
            &[
                ("source_node", None),
                ("source_contract_digest", None),
                ("component_resource_port", None),
                ("resource_slot", None),
            ],
        ),
        "ExecutionMappingKey" => map_key(
            value,
            &[("source_node", None), ("source_contract_digest", None)],
        ),
        "SubflowMappingKey" => map_key(
            value,
            &[("target_semantic_ir_digest", None), ("call_site", None)],
        ),
        "InputUseSiteKey" => map_key(
            value,
            &[
                ("computation", None),
                ("source_contract_digest", None),
                ("source_component_port", None),
                ("input_expression", None),
            ],
        ),
        "EffectMappingKey" => effect_key(value),
        "RecoveryMappingKey" => recovery_key(value),
        "FlowMappingKey" => flow_key(value),
        _ => None,
    }
}

fn scalar_text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) | Value::Enum(text) => Some(text.clone()),
        _ => None,
    }
}

fn field<'a>(value: &'a Value, name: &str) -> Option<&'a Value> {
    let Value::Record(fields) = value else {
        return None;
    };
    fields
        .iter()
        .find(|(field, _)| field == name)
        .map(|(_, value)| value)
}

fn number_key(value: &Value) -> Option<OrderKey> {
    match value {
        Value::UInt32(number) => Some(OrderKey::Number(u64::from(*number))),
        Value::UInt64(number) => Some(OrderKey::Number(*number)),
        _ => None,
    }
}

fn text_key(value: &Value, name: &str) -> Option<OrderKey> {
    scalar_text(field(value, name)?).map(OrderKey::Text)
}

fn pair_key(value: &Value, left: &str, right: &str) -> Option<OrderKey> {
    Some(OrderKey::Tuple(vec![
        text_key(value, left)?,
        text_key(value, right)?,
    ]))
}

fn digest_key(value: &Value) -> Option<OrderKey> {
    let algorithm = text_key(value, "algorithm")?;
    let domain = text_key(value, "domain")?;
    let hash = text_key(value, "hex")?;
    Some(OrderKey::Tuple(vec![algorithm, domain, hash]))
}

fn schema_key(value: &Value) -> Option<OrderKey> {
    // ResourceTypeContent keeps its TypeSchemaRef under `schema`; the other
    // registered SchemaRefKey collections carry the ref directly (or under
    // the common asset `reference` field). All use the same exact nominal
    // tuple and digest tie-breaker.
    let value = field(value, "schema").unwrap_or(value);
    let value = field(value, "reference").unwrap_or(value);
    let specification = text_key(value, "specification_id")?;
    let name = text_key(value, "type_name")?;
    let digest = field(value, "schema_digest").and_then(digest_key)?;
    Some(OrderKey::Tuple(vec![specification, name, digest]))
}

fn typed_key(value: &Value) -> Option<OrderKey> {
    let value = field(value, "reference").unwrap_or(value);
    let value = field(value, "reference").unwrap_or(value);
    let digest = field(value, "digest").and_then(digest_key)?;
    let schema = field(value, "type_schema").and_then(schema_key)?;
    Some(OrderKey::Tuple(vec![digest, schema]))
}

fn map_key(value: &Value, fields: &[(&str, Option<&str>)]) -> Option<OrderKey> {
    let mut key = Vec::new();
    for (outer, inner) in fields {
        let mut item = field(value, outer)?;
        if let Some(inner) = inner {
            item = field(item, inner)?;
        }
        key.push(
            number_key(item)
                .or_else(|| digest_key(item))
                .or_else(|| scalar_text(item).map(OrderKey::Text))?,
        );
    }
    Some(OrderKey::Tuple(key))
}

fn atom_key(value: &Value) -> Option<OrderKey> {
    let operation = field(value, "operation").and_then(|v| pair_key(v, "catalog", "name"))?;
    let selector = field(value, "resource_selector")?;
    let port = number_key(field(selector, "requirement_port_no")?)?;
    let scopes = field(selector, "scope_bindings")?;
    let qualifiers = field(value, "qualifiers")?;
    let mut scope_keys = Vec::new();
    for scope in match scopes {
        Value::List(v) => v,
        _ => return None,
    } {
        scope_keys.push(pair_key(scope, "key", "value")?);
    }
    let mut qualifier_keys = Vec::new();
    for qualifier in match qualifiers {
        Value::List(v) => v,
        _ => return None,
    } {
        let Value::Variant {
            case,
            value: payload,
        } = qualifier
        else {
            return None;
        };
        let rank = match case.as_str() {
            "natural" => 1,
            "none" => 2,
            _ => 3,
        };
        qualifier_keys.push(OrderKey::Tuple(vec![
            OrderKey::Number(rank),
            OrderKey::Text(format!("{payload:?}")),
        ]));
    }
    Some(OrderKey::Tuple(vec![
        operation,
        port,
        OrderKey::Tuple(scope_keys),
        OrderKey::Tuple(qualifier_keys),
    ]))
}

fn recovery_action_key(value: &Value) -> Option<OrderKey> {
    let forward = atom_key(field(value, "forward_effect")?)?;
    let kind = match scalar_text(field(value, "action_kind")?)?.as_str() {
        "reconciliation" => 1,
        "compensation" => 2,
        _ => return None,
    };
    let action = atom_key(field(value, "action_effect")?)?;
    Some(OrderKey::Tuple(vec![
        forward,
        OrderKey::Number(kind),
        action,
    ]))
}

fn effect_key(value: &Value) -> Option<OrderKey> {
    let export_atom = field(value, "export_atom").and_then(atom_key)?;
    let source = field(value, "source")?;
    Some(OrderKey::Tuple(vec![
        export_atom,
        digest_key(field(source, "caller_semantic_ir_digest")?)?,
        number_key(field(source, "source_node")?)?,
        digest_key(field(source, "source_contract_digest")?)?,
        number_key(field(source, "source_entry_ordinal")?)?,
    ]))
}

fn recovery_key(value: &Value) -> Option<OrderKey> {
    let export = recovery_action_key(field(value, "export_action_identity")?)?;
    let source = field(value, "source")?;
    let action = recovery_action_key(field(source, "action")?)?;
    Some(OrderKey::Tuple(vec![
        export,
        digest_key(field(source, "caller_semantic_ir_digest")?)?,
        number_key(field(source, "source_node")?)?,
        digest_key(field(source, "source_contract_digest")?)?,
        number_key(field(source, "forward_entry_ordinal")?)?,
        action,
    ]))
}

fn flow_key(value: &Value) -> Option<OrderKey> {
    let sink = option_payload(field(value, "export_sink")?)?;
    let source = option_payload(field(value, "export_source")?)?;
    let class = match (sink, source) {
        (MaybePayload::Present(_), _) => 0,
        (MaybePayload::Absent, MaybePayload::Present(_)) => 1,
        (MaybePayload::Absent, MaybePayload::Absent) => 2,
    };
    let constraint = if let MaybePayload::Present(sink) = sink {
        let kind = field(value, "export_sink")?;
        let Value::Option(Some(kind)) = kind else {
            return None;
        };
        let flow_kind = field(kind, "kind").and_then(scalar_text)?;
        let rank = if flow_kind == "data_egress" { 1 } else { 2 };
        Some(OrderKey::Tuple(vec![
            OrderKey::Number(rank),
            sink_key(sink)?,
        ]))
    } else {
        None
    };
    let source = match source {
        MaybePayload::Present(value) => source_key(value)?,
        MaybePayload::Absent => OrderKey::Tuple(Vec::new()),
    };
    let row = field(value, "source_contract_digest")?;
    Some(OrderKey::Tuple(vec![
        OrderKey::Number(class),
        constraint.unwrap_or(OrderKey::Tuple(Vec::new())),
        source,
        digest_key(row)?,
        number_key(field(value, "source_node")?)?,
        number_key(field(value, "flow_mapping_ordinal")?)?,
    ]))
}

#[derive(Clone, Copy)]
enum MaybePayload<'a> {
    Absent,
    Present(&'a Value),
}

fn option_payload(value: &Value) -> Option<MaybePayload<'_>> {
    match value {
        Value::Option(None) => Some(MaybePayload::Absent),
        Value::Option(Some(v)) => Some(MaybePayload::Present(v)),
        _ => None,
    }
}
fn source_key(value: &Value) -> Option<OrderKey> {
    let category = field(value, "category").unwrap_or(value);
    match category {
        Value::Variant { case, value } => {
            let rank = match case.as_str() {
                "workflow_input" => 1,
                "resource_requirement" => 2,
                "unknown_external" => 3,
                _ => return None,
            };
            Some(OrderKey::Tuple(vec![
                OrderKey::Number(rank),
                match value.as_ref() {
                    Value::UInt32(v) => OrderKey::Number(u64::from(*v)),
                    Value::Unit => OrderKey::Number(0),
                    _ => return None,
                },
            ]))
        }
        _ => None,
    }
}
fn sink_key(value: &Value) -> Option<OrderKey> {
    if let Value::Record(_) = value {
        let kind = scalar_text(field(value, "kind")?)?;
        let rank = if kind == "data_egress" {
            1
        } else if kind == "decision_influence" {
            2
        } else {
            return None;
        };
        let sink = field(value, "sink")?;
        let sink_key = match sink {
            Value::Variant { case, value } if case == "workflow_output" => {
                OrderKey::Tuple(vec![OrderKey::Number(1), number_key(value)?])
            }
            Value::Variant { case, value } if case == "external_effect" => OrderKey::Tuple(vec![
                OrderKey::Number(2),
                pair_key(value, "catalog", "name")?,
            ]),
            _ => return None,
        };
        return Some(OrderKey::Tuple(vec![OrderKey::Number(rank), sink_key]));
    }
    match value {
        Value::Variant { case, value } if case == "workflow_output" => Some(OrderKey::Tuple(vec![
            OrderKey::Number(1),
            number_key(value)?,
        ])),
        Value::Variant { case, value } if case == "external_effect" => Some(OrderKey::Tuple(vec![
            OrderKey::Number(2),
            pair_key(value, "catalog", "name")?,
        ])),
        _ => None,
    }
}

fn error(
    stage: ValidationStage,
    family: &'static str,
    path: &str,
    detail: &str,
) -> PortableProofError {
    PortableProofError::new(stage, family, path, detail)
}
