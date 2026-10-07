//! Canonical ordering, ACE1 lowering and the `SemanticIrDigest`.
//!
//! Canonicalization is a pure function of the source document: every table is
//! sorted by its own identity, every nested list by its own ordinal — except
//! the two lists whose order IS semantics (SEQUENCE children and RETRY
//! delays), which are preserved verbatim. The canonical form is lowered to an
//! ACE1 value tree mirroring `spec/idl/semantic-ir.afidl`, encoded against the
//! schema-derived cross-module `Shape`, and hashed:
//!
//! ```text
//! SemanticIrDigest = HashV1("semantic-ir", ACE1(CanonicalSemanticIrDocument))
//!                  = SHA256("AF\0" || "semantic-ir" || "\0" || canonical_bytes)
//! ```
//!
//! The `semantic-ir` domain is the Phase 1 frozen registry entry; the digest
//! never touches any other domain, and `TypeSchemaDigest` keeps hashing
//! schemas under `type-schema`.

use crate::error::{IrError, kind};
use crate::ids::ComputationId;
use crate::model::{
    ComputationBody, ComputationDef, ContractRefKind, HandleMatch, IterateBody, ParallelMode,
    SemanticIrDocument,
};
use af_core::{DigestDomain, DigestRef, Value};
use af_semantic::{ExpressionBody, PrimitiveConstant, SemanticTypeNodeBody, TypeSchemaRef};
use std::collections::BTreeMap;

impl SemanticIrDocument {
    /// The canonical form: every table and nested list in its frozen order.
    /// SEQUENCE children and RETRY delays are preserved (their order is
    /// semantic).
    #[must_use]
    pub fn canonical(&self) -> SemanticIrDocument {
        let mut document = self.clone();
        document.semantic_refs.sort_by(|a, b| {
            a.specification_id
                .as_bytes()
                .cmp(b.specification_id.as_bytes())
        });
        document.types.sort_by_key(|node| node.id);
        document.values.sort_by_key(|slot| slot.id);
        document.expressions.sort_by_key(|def| def.id);
        document.resources.sort_by_key(|slot| slot.id);
        document.contracts.sort_by_key(|entry| entry.id);
        document
            .run_parameters
            .sort_by_key(|parameter| parameter.id);
        document
            .workflow_outputs
            .sort_by(|a, b| nfc(&a.name).as_bytes().cmp(nfc(&b.name).as_bytes()));
        document.computations.sort_by_key(|def| def.id);
        for computation in &mut document.computations {
            canonicalize_computation(computation);
        }
        document
    }

    /// The canonical ACE1 bytes, encoded against the schema-derived shape of
    /// `agentflow.semantic.ir@1/SemanticIrDocument` within `set` (which must
    /// contain the semantic IR module and the semantic core it imports).
    ///
    /// # Errors
    /// [`IrError::Encoding`] when the shape cannot be derived or the value
    /// tree cannot be encoded.
    pub fn canonical_bytes(&self, set: &af_idl::CompiledModuleSet) -> Result<Vec<u8>, IrError> {
        let shape = set
            .shape_of("agentflow.semantic.ir@1/SemanticIrDocument")
            .map_err(|error| IrError::Encoding {
                detail: format!("cannot derive the document shape: {error}"),
            })?;
        shape.validate().map_err(|error| IrError::Encoding {
            detail: format!("derived shape is invalid: {error}"),
        })?;
        let value = self.canonical().to_value();
        af_core::encode(&shape, &value).map_err(|error| IrError::Encoding {
            detail: error.to_string(),
        })
    }

    /// `SemanticIrDigest = HashV1("semantic-ir", ACE1(CanonicalSemanticIrDocument))`.
    ///
    /// # Errors
    /// [`IrError::Encoding`] when the canonical bytes cannot be computed.
    pub fn digest(&self, set: &af_idl::CompiledModuleSet) -> Result<DigestRef, IrError> {
        let bytes = self.canonical_bytes(set)?;
        Ok(DigestRef::compute(DigestDomain::SemanticIr, &bytes))
    }

    /// The ACE1 value tree of the canonical document (mirrors the AFIDL schema
    /// field-for-field; the encoder emits members in byte order).
    pub fn to_value(&self) -> Value {
        Value::Record(vec![
            (
                "semantic_refs".into(),
                Value::List(
                    self.semantic_refs
                        .iter()
                        .map(|reference| {
                            Value::Record(vec![(
                                "specification_id".into(),
                                Value::String(reference.specification_id.clone()),
                            )])
                        })
                        .collect(),
                ),
            ),
            (
                "types".into(),
                Value::List(self.types.iter().map(type_node_value).collect()),
            ),
            (
                "values".into(),
                Value::List(
                    self.values
                        .iter()
                        .map(|slot| {
                            Value::Record(vec![
                                ("id".into(), Value::UInt32(slot.id.raw())),
                                ("type".into(), Value::UInt32(slot.type_id.raw())),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "expressions".into(),
                Value::List(self.expressions.iter().map(expression_value).collect()),
            ),
            (
                "resources".into(),
                Value::List(
                    self.resources
                        .iter()
                        .map(|slot| {
                            Value::Record(vec![
                                ("id".into(), Value::UInt32(slot.id.raw())),
                                (
                                    "requirement_type".into(),
                                    type_schema_ref_value(&slot.requirement_type),
                                ),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "contracts".into(),
                Value::List(
                    self.contracts
                        .iter()
                        .map(|entry| {
                            Value::Record(vec![
                                ("id".into(), Value::UInt32(entry.id.raw())),
                                ("kind".into(), Value::Enum(entry.kind.keyword().into())),
                                (
                                    "digest".into(),
                                    Value::Record(vec![
                                        (
                                            "algorithm".into(),
                                            Value::String(entry.digest.algorithm.clone()),
                                        ),
                                        (
                                            "domain".into(),
                                            Value::String(entry.digest.domain.clone()),
                                        ),
                                        ("hex".into(), Value::String(entry.digest.hex.clone())),
                                    ]),
                                ),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "run_parameters".into(),
                Value::List(
                    self.run_parameters
                        .iter()
                        .map(|parameter| {
                            Value::Record(vec![
                                ("id".into(), Value::UInt32(parameter.id.raw())),
                                ("name".into(), Value::String(parameter.name.clone())),
                                ("type".into(), Value::UInt32(parameter.type_id.raw())),
                                (
                                    "value_slot".into(),
                                    Value::UInt32(parameter.value_slot.raw()),
                                ),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "workflow_outputs".into(),
                Value::List(
                    self.workflow_outputs
                        .iter()
                        .map(|output| {
                            Value::Record(vec![
                                ("name".into(), Value::String(output.name.clone())),
                                ("expression".into(), Value::UInt32(output.expression.raw())),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "computations".into(),
                Value::List(self.computations.iter().map(computation_value).collect()),
            ),
            ("root".into(), Value::UInt32(self.root.raw())),
        ])
    }
}

fn nfc(text: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    text.nfc().collect()
}

fn canonicalize_computation(computation: &mut ComputationDef) {
    computation
        .outputs
        .sort_by_key(|binding| binding.port_no.raw());
    match &mut computation.body {
        ComputationBody::Invoke(body) => {
            body.inputs.sort_by_key(|binding| binding.port_no.raw());
            body.resources.sort_by_key(|binding| binding.port_no.raw());
        }
        ComputationBody::CallSubflow(body) => {
            body.inputs.sort_by_key(|binding| binding.port_no.raw());
            body.resources.sort_by_key(|binding| binding.port_no.raw());
        }
        ComputationBody::Scope(body) => {
            body.resources.sort_by_key(|slot| slot.raw());
        }
        ComputationBody::Await(_)
        | ComputationBody::Sequence(_)
        | ComputationBody::Retry(_)
        | ComputationBody::Timeout(_)
        | ComputationBody::CompensationScope(_) => {}
        ComputationBody::Select(body) => {
            body.cases.sort_by_key(|case| case.ordinal.raw());
        }
        ComputationBody::Parallel(body) => {
            body.branches.sort_by_key(|branch| branch.ordinal.raw());
            body.output_bindings
                .sort_by_key(|binding| binding.parent_port_no.raw());
        }
        ComputationBody::Race(body) => {
            body.branches.sort_by_key(|branch| branch.ordinal.raw());
        }
        ComputationBody::Iterate(body) => match body {
            IterateBody::Foreach(foreach) => {
                foreach.carry.sort_by_key(|binding| binding.slot.raw());
            }
            IterateBody::While(while_body) => {
                while_body.carry.sort_by_key(|binding| binding.slot.raw());
            }
        },
        ComputationBody::Handle(body) => {
            body.clauses.sort_by_key(|clause| clause.ordinal.raw());
        }
    }
}

fn type_node_value(node: &af_semantic::SemanticTypeNode) -> Value {
    let body = match &node.body {
        SemanticTypeNodeBody::Primitive(primitive) => Value::Variant {
            case: "primitive".into(),
            // SemanticPrimitiveType is an AFIDL enum: it encodes as its
            // member name string through the closed enum shape.
            value: Box::new(Value::Enum(primitive.keyword().into())),
        },
        SemanticTypeNodeBody::Schema(reference) => Value::Variant {
            case: "schema".into(),
            value: Box::new(type_schema_ref_value(reference)),
        },
        SemanticTypeNodeBody::List(inner) => Value::Variant {
            case: "list".into(),
            value: Box::new(Value::UInt32(inner.raw())),
        },
        SemanticTypeNodeBody::MapString(inner) => Value::Variant {
            case: "map_string".into(),
            value: Box::new(Value::UInt32(inner.raw())),
        },
        SemanticTypeNodeBody::Option(inner) => Value::Variant {
            case: "option".into(),
            value: Box::new(Value::UInt32(inner.raw())),
        },
    };
    Value::Record(vec![
        ("id".into(), Value::UInt32(node.id.raw())),
        ("body".into(), body),
    ])
}

fn type_schema_ref_value(reference: &TypeSchemaRef) -> Value {
    Value::Record(vec![
        (
            "specification_id".into(),
            Value::String(reference.specification_id.clone()),
        ),
        (
            "type_name".into(),
            Value::String(reference.type_name.clone()),
        ),
        (
            "schema_digest".into(),
            Value::Record(vec![
                (
                    "algorithm".into(),
                    Value::String(reference.schema_digest.algorithm.clone()),
                ),
                (
                    "domain".into(),
                    Value::String(reference.schema_digest.domain.clone()),
                ),
                (
                    "hex".into(),
                    Value::String(reference.schema_digest.hex.clone()),
                ),
            ]),
        ),
    ])
}

fn expression_value(def: &af_semantic::ExpressionDef) -> Value {
    Value::Record(vec![
        ("id".into(), Value::UInt32(def.id.raw())),
        (
            "result_type".into(),
            Value::UInt32(def.result_type_id.raw()),
        ),
        ("body".into(), expression_body_value(&def.body)),
    ])
}

fn expression_body_value(body: &ExpressionBody) -> Value {
    use ExpressionBody as Body;
    let (case, payload) = match body {
        Body::Constant(constant) => ("constant", constant_value(constant)),
        Body::ValueSlot(slot) => ("value_slot", Value::UInt32(slot.raw())),
        Body::RecordConstruct { schema, fields } => (
            "record_construct",
            Value::Record(vec![
                ("schema".into(), type_schema_ref_value(schema)),
                (
                    "fields".into(),
                    Value::List(
                        fields
                            .iter()
                            .map(|field| {
                                Value::Record(vec![
                                    ("field_no".into(), Value::UInt32(field.field_no)),
                                    ("expr_id".into(), Value::UInt32(field.expr_id.raw())),
                                ])
                            })
                            .collect(),
                    ),
                ),
            ]),
        ),
        Body::VariantConstruct {
            schema,
            case_no,
            payload_expr,
        } => (
            "variant_construct",
            Value::Record(vec![
                ("schema".into(), type_schema_ref_value(schema)),
                ("case_no".into(), Value::UInt32(*case_no)),
                ("payload_expr".into(), Value::UInt32(payload_expr.raw())),
            ]),
        ),
        Body::EnumValue { schema, value_no } => (
            "enum_value",
            Value::Record(vec![
                ("schema".into(), type_schema_ref_value(schema)),
                ("value_no".into(), Value::UInt32(*value_no)),
            ]),
        ),
        Body::ListConstruct {
            element_type,
            elements,
        } => (
            "list_construct",
            Value::Record(vec![
                ("element_type".into(), Value::UInt32(element_type.raw())),
                (
                    "elements".into(),
                    Value::List(elements.iter().map(|id| Value::UInt32(id.raw())).collect()),
                ),
            ]),
        ),
        Body::MapConstruct {
            value_type,
            entries,
        } => (
            "map_construct",
            Value::Record(vec![
                ("value_type".into(), Value::UInt32(value_type.raw())),
                (
                    "entries".into(),
                    Value::List(
                        entries
                            .iter()
                            .map(|entry| {
                                Value::Record(vec![
                                    ("key".into(), Value::String(entry.key.clone())),
                                    ("value_expr".into(), Value::UInt32(entry.value_expr.raw())),
                                ])
                            })
                            .collect(),
                    ),
                ),
            ]),
        ),
        Body::OptionNone { inner_type } => (
            "option_none",
            Value::Record(vec![("inner_type".into(), Value::UInt32(inner_type.raw()))]),
        ),
        Body::OptionSome { payload_expr } => (
            "option_some",
            Value::Record(vec![(
                "payload_expr".into(),
                Value::UInt32(payload_expr.raw()),
            )]),
        ),
        Body::FieldGet { record, field_no } => (
            "field_get",
            Value::Record(vec![
                ("record".into(), Value::UInt32(record.raw())),
                ("field_no".into(), Value::UInt32(*field_no)),
            ]),
        ),
        Body::Equals { left, right } => (
            "equals",
            Value::Record(vec![
                ("left".into(), Value::UInt32(left.raw())),
                ("right".into(), Value::UInt32(right.raw())),
            ]),
        ),
        Body::Not { operand } => (
            "not",
            Value::Record(vec![("operand".into(), Value::UInt32(operand.raw()))]),
        ),
        Body::And { left, right } => (
            "and",
            Value::Record(vec![
                ("left".into(), Value::UInt32(left.raw())),
                ("right".into(), Value::UInt32(right.raw())),
            ]),
        ),
        Body::Or { left, right } => (
            "or",
            Value::Record(vec![
                ("left".into(), Value::UInt32(left.raw())),
                ("right".into(), Value::UInt32(right.raw())),
            ]),
        ),
        Body::IfThenElse {
            condition,
            then_expr,
            else_expr,
        } => (
            "if_then_else",
            Value::Record(vec![
                ("condition".into(), Value::UInt32(condition.raw())),
                ("then_expr".into(), Value::UInt32(then_expr.raw())),
                ("else_expr".into(), Value::UInt32(else_expr.raw())),
            ]),
        ),
    };
    Value::Variant {
        case: case.into(),
        value: Box::new(payload),
    }
}

fn constant_value(constant: &PrimitiveConstant) -> Value {
    use af_core::Value as V;
    let (case, payload) = match constant {
        PrimitiveConstant::Unit => ("unit", V::Unit),
        PrimitiveConstant::Null => ("null", V::Null),
        PrimitiveConstant::Bool(value) => ("bool", V::Bool(*value)),
        PrimitiveConstant::Int64(value) => ("int64", V::Int64(*value)),
        PrimitiveConstant::Decimal(value) => ("decimal", V::Decimal(value.clone())),
        PrimitiveConstant::String(value) => ("string", V::String(value.clone())),
        PrimitiveConstant::Bytes(value) => ("bytes", V::String(value.clone())),
        PrimitiveConstant::Timestamp(value) => ("timestamp", V::Timestamp(*value)),
        PrimitiveConstant::Duration(value) => ("duration", V::Duration(*value)),
    };
    V::Variant {
        case: case.into(),
        value: Box::new(payload),
    }
}

/// Lowers one computation to its ACE1 value.
fn computation_value(computation: &ComputationDef) -> Value {
    Value::Record(vec![
        ("id".into(), Value::UInt32(computation.id.raw())),
        (
            "outputs".into(),
            Value::List(
                computation
                    .outputs
                    .iter()
                    .map(|binding| {
                        Value::Record(vec![
                            ("port_no".into(), Value::UInt32(binding.port_no.raw())),
                            ("value_slot".into(), Value::UInt32(binding.value_slot.raw())),
                        ])
                    })
                    .collect(),
            ),
        ),
        ("body".into(), body_value(&computation.body)),
    ])
}

fn input_bindings_value(inputs: &[crate::model::InputBinding]) -> Value {
    Value::List(
        inputs
            .iter()
            .map(|binding| {
                Value::Record(vec![
                    ("port_no".into(), Value::UInt32(binding.port_no.raw())),
                    ("expression".into(), Value::UInt32(binding.expression.raw())),
                ])
            })
            .collect(),
    )
}

fn resource_bindings_value(resources: &[crate::model::ResourceUseBinding]) -> Value {
    Value::List(
        resources
            .iter()
            .map(|binding| {
                Value::Record(vec![
                    ("port_no".into(), Value::UInt32(binding.port_no.raw())),
                    (
                        "resource_slot".into(),
                        Value::UInt32(binding.resource_slot.raw()),
                    ),
                ])
            })
            .collect(),
    )
}

fn branch_list_value<'a, B: OrdinalBranch + 'a>(
    branches: impl IntoIterator<Item = &'a B>,
) -> Value {
    Value::List(
        branches
            .into_iter()
            .map(|branch| {
                Value::Record(vec![
                    ("ordinal".into(), Value::UInt32(branch.ordinal().raw())),
                    ("child".into(), Value::UInt32(branch.child().raw())),
                ])
            })
            .collect(),
    )
}

trait OrdinalBranch {
    fn ordinal(&self) -> crate::ids::BranchOrdinal;
    fn child(&self) -> ComputationId;
}

impl OrdinalBranch for crate::model::ParallelBranch {
    fn ordinal(&self) -> crate::ids::BranchOrdinal {
        self.ordinal
    }
    fn child(&self) -> ComputationId {
        self.child
    }
}

impl OrdinalBranch for crate::model::RaceBranch {
    fn ordinal(&self) -> crate::ids::BranchOrdinal {
        self.ordinal
    }
    fn child(&self) -> ComputationId {
        self.child
    }
}

fn carry_value(carry: &[crate::model::LoopCarryBinding]) -> Value {
    Value::List(
        carry
            .iter()
            .map(|binding| {
                Value::Record(vec![
                    ("slot".into(), Value::UInt32(binding.slot.raw())),
                    ("initial".into(), Value::UInt32(binding.initial.raw())),
                    (
                        "body_output_port".into(),
                        Value::UInt32(binding.body_output_port.raw()),
                    ),
                ])
            })
            .collect(),
    )
}

fn body_value(body: &ComputationBody) -> Value {
    let (case, payload) = match body {
        ComputationBody::Invoke(body) => (
            "invoke",
            Value::Record(vec![
                ("contract".into(), Value::UInt32(body.contract.raw())),
                ("inputs".into(), input_bindings_value(&body.inputs)),
                ("resources".into(), resource_bindings_value(&body.resources)),
            ]),
        ),
        ComputationBody::Await(body) => (
            "await",
            Value::Record(vec![
                (
                    "signal_type".into(),
                    type_schema_ref_value(&body.signal_type),
                ),
                ("correlation".into(), Value::UInt32(body.correlation.raw())),
            ]),
        ),
        ComputationBody::Sequence(body) => (
            "sequence",
            Value::Record(vec![(
                "children".into(),
                Value::List(
                    body.children
                        .iter()
                        .map(|id| Value::UInt32(id.raw()))
                        .collect(),
                ),
            )]),
        ),
        ComputationBody::Select(body) => (
            "select",
            Value::Record(vec![
                ("selector".into(), Value::UInt32(body.selector.raw())),
                (
                    "cases".into(),
                    Value::List(
                        body.cases
                            .iter()
                            .map(|case| {
                                Value::Record(vec![
                                    ("ordinal".into(), Value::UInt32(case.ordinal.raw())),
                                    ("child".into(), Value::UInt32(case.child.raw())),
                                ])
                            })
                            .collect(),
                    ),
                ),
            ]),
        ),
        ComputationBody::Parallel(body) => (
            "parallel",
            Value::Record(vec![
                ("mode".into(), parallel_mode_value(body.mode)),
                ("branches".into(), branch_list_value(&body.branches)),
                ("max_inflight".into(), option_u32_value(body.max_inflight)),
                (
                    "output_bindings".into(),
                    Value::List(
                        body.output_bindings
                            .iter()
                            .map(|binding| {
                                Value::Record(vec![
                                    (
                                        "parent_port_no".into(),
                                        Value::UInt32(binding.parent_port_no.raw()),
                                    ),
                                    (
                                        "branch_ordinal".into(),
                                        Value::UInt32(binding.branch_ordinal.raw()),
                                    ),
                                    (
                                        "child_port_no".into(),
                                        Value::UInt32(binding.child_port_no.raw()),
                                    ),
                                ])
                            })
                            .collect(),
                    ),
                ),
            ]),
        ),
        ComputationBody::Race(body) => (
            "race",
            Value::Record(vec![("branches".into(), branch_list_value(&body.branches))]),
        ),
        ComputationBody::Iterate(body) => ("iterate", iterate_value(body)),
        ComputationBody::Retry(body) => (
            "retry",
            Value::Record(vec![
                ("child".into(), Value::UInt32(body.child.raw())),
                (
                    "delays".into(),
                    Value::List(
                        body.delays
                            .iter()
                            .map(|delay| Value::Duration(*delay))
                            .collect(),
                    ),
                ),
            ]),
        ),
        ComputationBody::Timeout(body) => (
            "timeout",
            Value::Record(vec![
                ("child".into(), Value::UInt32(body.child.raw())),
                ("timeout".into(), Value::UInt32(body.timeout.raw())),
            ]),
        ),
        ComputationBody::Handle(body) => (
            "handle",
            Value::Record(vec![
                ("child".into(), Value::UInt32(body.child.raw())),
                (
                    "clauses".into(),
                    Value::List(
                        body.clauses
                            .iter()
                            .map(|clause| {
                                Value::Record(vec![
                                    ("ordinal".into(), Value::UInt32(clause.ordinal.raw())),
                                    ("match".into(), handle_match_value(&clause.matcher)),
                                    ("handler".into(), Value::UInt32(clause.handler.raw())),
                                ])
                            })
                            .collect(),
                    ),
                ),
            ]),
        ),
        ComputationBody::Scope(body) => (
            "scope",
            Value::Record(vec![
                ("child".into(), Value::UInt32(body.child.raw())),
                (
                    "resources".into(),
                    Value::List(
                        body.resources
                            .iter()
                            .map(|slot| Value::UInt32(slot.raw()))
                            .collect(),
                    ),
                ),
            ]),
        ),
        ComputationBody::CompensationScope(body) => (
            "compensation_scope",
            Value::Record(vec![("child".into(), Value::UInt32(body.child.raw()))]),
        ),
        ComputationBody::CallSubflow(body) => (
            "call_subflow",
            Value::Record(vec![
                ("contract".into(), Value::UInt32(body.contract.raw())),
                ("inputs".into(), input_bindings_value(&body.inputs)),
                ("resources".into(), resource_bindings_value(&body.resources)),
            ]),
        ),
    };
    Value::Variant {
        case: case.into(),
        value: Box::new(payload),
    }
}

fn iterate_value(body: &IterateBody) -> Value {
    let (case, payload) = match body {
        IterateBody::Foreach(foreach) => (
            "foreach",
            Value::Record(vec![
                ("source".into(), Value::UInt32(foreach.source.raw())),
                ("item_slot".into(), Value::UInt32(foreach.item_slot.raw())),
                ("body".into(), Value::UInt32(foreach.body.raw())),
                ("carry".into(), carry_value(&foreach.carry)),
                (
                    "max_inflight".into(),
                    option_u32_value(foreach.max_inflight),
                ),
            ]),
        ),
        IterateBody::While(while_body) => (
            "while",
            Value::Record(vec![
                ("guard".into(), Value::UInt32(while_body.guard.raw())),
                ("body".into(), Value::UInt32(while_body.body.raw())),
                ("carry".into(), carry_value(&while_body.carry)),
                (
                    "max_iterations".into(),
                    Value::UInt32(while_body.max_iterations),
                ),
            ]),
        ),
    };
    Value::Variant {
        case: case.into(),
        value: Box::new(payload),
    }
}

fn parallel_mode_value(mode: ParallelMode) -> Value {
    Value::Enum(mode.keyword().into())
}

fn option_u32_value(value: Option<u32>) -> Value {
    Value::Option(value.map(|raw| Box::new(Value::UInt32(raw))))
}

fn handle_match_value(matcher: &HandleMatch) -> Value {
    match matcher {
        HandleMatch::Failure(failure) => Value::Variant {
            case: "failure".into(),
            value: Box::new(Value::Record(vec![(
                "failure_type".into(),
                type_schema_ref_value(&failure.failure_type),
            )])),
        },
        HandleMatch::TimedOut(()) => Value::Variant {
            case: "timed_out".into(),
            value: Box::new(Value::Unit),
        },
    }
}

/// A canonical-ordering lookup used by tests and future tooling: the digest of
/// the canonical form must not depend on source order, so this helper exposes
/// the canonical tables as an ordered view.
pub fn canonical_computations(document: &SemanticIrDocument) -> BTreeMap<u32, &ComputationDef> {
    document
        .computations
        .iter()
        .map(|def| (def.id.raw(), def))
        .collect()
}

/// The stable kind re-exported for the encoding error path.
pub fn encoding_error_kind() -> &'static str {
    kind::ENCODING_ERROR
}

/// The contract kind → expected digest domain mapping, as a public table so
/// tooling does not re-derive it.
pub fn contract_digest_domain(kind: ContractRefKind) -> &'static str {
    kind.digest_domain()
}
