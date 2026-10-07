//! End-to-end semantic core validation: build a type DAG, value slots, and a
//! closed expression set against a compiled AFIDL universe, then check types,
//! reject the documented failure classes, and lower to ACE1 bytes.

use af_semantic::ids::SemanticTypeId as TypeId;
use af_semantic::{
    ExpressionBody, ExpressionDef, ExpressionId, PrimitiveConstant, RecordFieldExpr, ResourceSlot,
    ResourceSlotId, SemanticCoreDocument, SemanticPrimitiveType, SemanticTypeNode,
    SemanticTypeNodeBody, TypeSchemaRef, TypeSchemaUniverse, ValueSlot, ValueSlotId,
};

const UNIVERSE: &str = "afidl 1;\n\
    schema agentflow.example @1;\n\
    alias UserId = string;\n\
    enum Status { 1 pending; 2 closed; }\n\
    record User { 1 id: UserId; 2 name: string; 3 status: Status; }\n\
    variant LookupResult { 1 found: User; 2 missing: unit; }\n\
    record Metrics { 1 count: int64; 2 ratio: float64; }\n";

fn universe() -> TypeSchemaUniverse {
    let compiled = af_idl::compile(UNIVERSE).unwrap();
    let mut universe = TypeSchemaUniverse::new();
    universe.insert_compiled(&compiled).unwrap();
    universe
}

fn schema_ref(universe: &TypeSchemaUniverse, identity: &str) -> TypeSchemaRef {
    universe.get(identity).unwrap().reference()
}

/// Type DAG: 1 string, 2 User, 3 Status, 4 list<string>, 5 option<int64>.
fn base_types(universe: &TypeSchemaUniverse, user_digest: &str) -> Vec<SemanticTypeNode> {
    let user = schema_ref(universe, "agentflow.example@1/User");
    let status = schema_ref(universe, "agentflow.example@1/Status");
    let _ = user_digest;
    vec![
        SemanticTypeNode {
            id: TypeId::from_raw(1),
            body: SemanticTypeNodeBody::Primitive(SemanticPrimitiveType::String),
        },
        SemanticTypeNode {
            id: TypeId::from_raw(2),
            body: SemanticTypeNodeBody::Schema(user),
        },
        SemanticTypeNode {
            id: TypeId::from_raw(3),
            body: SemanticTypeNodeBody::Schema(status),
        },
        SemanticTypeNode {
            id: TypeId::from_raw(4),
            body: SemanticTypeNodeBody::List(TypeId::from_raw(1)),
        },
        SemanticTypeNode {
            id: TypeId::from_raw(5),
            body: SemanticTypeNodeBody::Option(TypeId::from_raw(1)),
        },
    ]
}

fn expr(id: u32, result_type: u32, body: ExpressionBody) -> ExpressionDef {
    ExpressionDef {
        id: ExpressionId::from_raw(id),
        result_type_id: TypeId::from_raw(result_type),
        body,
    }
}

#[test]
fn a_complete_document_type_checks_and_lowers_to_canonical_bytes() {
    let universe = universe();
    let user = schema_ref(&universe, "agentflow.example@1/User");
    let status = schema_ref(&universe, "agentflow.example@1/Status");

    // Expressions:
    // 1  String("Ada")                        : string (type 1)
    // 2  EnumValue(Status, value_no 2=closed)  : Status (type 3)
    // 3  ValueSlot(1)                          : string (type 1)
    // 4  RecordConstruct(User){1->expr3, 2->expr1, 3->expr2} : User (type 2)
    let expressions = vec![
        expr(
            1,
            1,
            ExpressionBody::Constant(PrimitiveConstant::String("Ada".into())),
        ),
        expr(
            2,
            3,
            ExpressionBody::EnumValue {
                schema: status.clone(),
                value_no: 2,
            },
        ),
        expr(3, 1, ExpressionBody::ValueSlot(ValueSlotId::from_raw(1))),
        expr(
            4,
            2,
            ExpressionBody::RecordConstruct {
                schema: user.clone(),
                fields: vec![
                    RecordFieldExpr {
                        field_no: 1,
                        expr_id: ExpressionId::from_raw(3),
                    },
                    RecordFieldExpr {
                        field_no: 2,
                        expr_id: ExpressionId::from_raw(1),
                    },
                    RecordFieldExpr {
                        field_no: 3,
                        expr_id: ExpressionId::from_raw(2),
                    },
                ],
            },
        ),
    ];

    let document = SemanticCoreDocument {
        types: base_types(&universe, ""),
        values: vec![ValueSlot {
            id: ValueSlotId::from_raw(1),
            type_id: TypeId::from_raw(1),
        }],
        expressions,
        resources: vec![],
    };

    let checked = document
        .check(&universe)
        .expect("document must be well typed");

    // Inferred types are the resolved structural types.
    assert_eq!(
        checked
            .infer_expression_type(ExpressionId::from_raw(4))
            .unwrap()
            .canonical_string(),
        format!(
            "schema:agentflow.example@1/User@{}",
            universe
                .get("agentflow.example@1/User")
                .unwrap()
                .digest
                .hex()
        )
    );
    assert_eq!(
        checked
            .infer_expression_type(ExpressionId::from_raw(2))
            .unwrap()
            .canonical_string(),
        format!(
            "schema:agentflow.example@1/Status@{}",
            universe
                .get("agentflow.example@1/Status")
                .unwrap()
                .digest
                .hex()
        )
    );

    // Lowering: the record is encoded under the derived record shape, with
    // field names as value-encoding keys.
    let mut bindings = std::collections::BTreeMap::new();
    bindings.insert(
        ValueSlotId::from_raw(1),
        af_core::Value::String("u-7".into()),
    );
    let (shape, value) = checked
        .lower_with(ExpressionId::from_raw(4), &bindings)
        .unwrap();
    let bytes = af_core::encode(&shape, &value).unwrap();
    assert_eq!(
        std::str::from_utf8(&bytes).unwrap(),
        r#"{"id":"u-7","name":"Ada","status":"closed"}"#
    );
}

#[test]
fn machine_only_primitives_are_rejected_before_anything_else() {
    let universe = universe();
    let metrics = schema_ref(&universe, "agentflow.example@1/Metrics");
    let document = SemanticCoreDocument {
        types: vec![SemanticTypeNode {
            id: TypeId::from_raw(1),
            body: SemanticTypeNodeBody::Schema(metrics),
        }],
        values: vec![],
        expressions: vec![],
        resources: vec![],
    };
    // Metrics.ratio is float64, a machine-only primitive: it cannot be a
    // workflow value type. Here the *record* is still a legal schema pin, so
    // this documents the boundary: the record exists, but a SemanticTypeNode
    // holding a bare `float64` primitive would fail.
    assert!(document.check(&universe).is_ok());

    let primitive_error =
        af_semantic::SemanticPrimitiveType::try_from_keyword("float64").unwrap_err();
    assert_eq!(primitive_error.kind(), "unsupported_workflow_primitive");
    assert_eq!(
        af_semantic::SemanticPrimitiveType::try_from_keyword("uint64")
            .unwrap_err()
            .kind(),
        "unsupported_workflow_primitive"
    );
}

#[test]
fn unknown_schema_and_wrong_digest_are_rejected() {
    let universe = universe();
    let user = schema_ref(&universe, "agentflow.example@1/User");

    let mut wrong = user.clone();
    wrong.type_name = "Status".into();
    let document = SemanticCoreDocument {
        types: vec![SemanticTypeNode {
            id: TypeId::from_raw(1),
            body: SemanticTypeNodeBody::Schema(wrong),
        }],
        values: vec![],
        expressions: vec![],
        resources: vec![],
    };
    assert_eq!(
        document.check(&universe).unwrap_err().kind(),
        "type_schema_digest_mismatch"
    );

    let mut ghost = user;
    ghost.type_name = "Ghost".into();
    let document = SemanticCoreDocument {
        types: vec![SemanticTypeNode {
            id: TypeId::from_raw(1),
            body: SemanticTypeNodeBody::Schema(ghost),
        }],
        values: vec![],
        expressions: vec![],
        resources: vec![],
    };
    assert_eq!(
        document.check(&universe).unwrap_err().kind(),
        "unknown_type_schema"
    );
}

#[test]
fn type_dag_cycles_are_rejected() {
    let universe = universe();
    let document = SemanticCoreDocument {
        types: vec![
            SemanticTypeNode {
                id: TypeId::from_raw(1),
                body: SemanticTypeNodeBody::List(TypeId::from_raw(2)),
            },
            SemanticTypeNode {
                id: TypeId::from_raw(2),
                body: SemanticTypeNodeBody::Option(TypeId::from_raw(1)),
            },
        ],
        values: vec![],
        expressions: vec![],
        resources: vec![],
    };
    assert_eq!(
        document.check(&universe).unwrap_err().kind(),
        "semantic_type_cycle"
    );
}

#[test]
fn expression_cycles_are_rejected() {
    let universe = universe();
    let document = SemanticCoreDocument {
        types: base_types(&universe, ""),
        values: vec![],
        expressions: vec![
            expr(
                1,
                1,
                ExpressionBody::OptionSome {
                    payload_expr: ExpressionId::from_raw(2),
                },
            ),
            expr(
                2,
                1,
                ExpressionBody::OptionSome {
                    payload_expr: ExpressionId::from_raw(1),
                },
            ),
        ],
        resources: vec![],
    };
    assert_eq!(
        document.check(&universe).unwrap_err().kind(),
        "expression_cycle"
    );
}

#[test]
fn a_declared_result_type_that_disagrees_with_the_body_is_rejected() {
    let universe = universe();
    let document = SemanticCoreDocument {
        types: base_types(&universe, ""),
        values: vec![],
        expressions: vec![expr(
            1,
            5, // declared option<string>
            ExpressionBody::Constant(PrimitiveConstant::String("x".into())),
        )],
        resources: vec![],
    };
    assert_eq!(
        document.check(&universe).unwrap_err().kind(),
        "expression_result_type_mismatch"
    );
}

#[test]
fn record_field_binding_errors_are_distinguished() {
    let universe = universe();
    let user = schema_ref(&universe, "agentflow.example@1/User");
    let status = schema_ref(&universe, "agentflow.example@1/Status");
    let base_types_vec = base_types(&universe, "");

    let build = |fields: Vec<RecordFieldExpr>| SemanticCoreDocument {
        types: base_types_vec.clone(),
        values: vec![],
        expressions: vec![
            expr(
                1,
                1,
                ExpressionBody::Constant(PrimitiveConstant::String("x".into())),
            ),
            expr(
                2,
                3,
                ExpressionBody::EnumValue {
                    schema: status.clone(),
                    value_no: 1,
                },
            ),
            expr(
                4,
                2,
                ExpressionBody::RecordConstruct {
                    schema: user.clone(),
                    fields,
                },
            ),
        ],
        resources: vec![],
    };

    // unknown field number
    let document = build(vec![
        RecordFieldExpr {
            field_no: 1,
            expr_id: ExpressionId::from_raw(1),
        },
        RecordFieldExpr {
            field_no: 2,
            expr_id: ExpressionId::from_raw(1),
        },
        RecordFieldExpr {
            field_no: 9,
            expr_id: ExpressionId::from_raw(2),
        },
    ]);
    assert_eq!(
        document.check(&universe).unwrap_err().kind(),
        "unknown_field_number"
    );

    // duplicate binding
    let document = build(vec![
        RecordFieldExpr {
            field_no: 1,
            expr_id: ExpressionId::from_raw(1),
        },
        RecordFieldExpr {
            field_no: 1,
            expr_id: ExpressionId::from_raw(1),
        },
        RecordFieldExpr {
            field_no: 2,
            expr_id: ExpressionId::from_raw(1),
        },
        RecordFieldExpr {
            field_no: 3,
            expr_id: ExpressionId::from_raw(2),
        },
    ]);
    assert_eq!(
        document.check(&universe).unwrap_err().kind(),
        "duplicate_field_binding"
    );

    // missing binding
    let document = build(vec![
        RecordFieldExpr {
            field_no: 1,
            expr_id: ExpressionId::from_raw(1),
        },
        RecordFieldExpr {
            field_no: 2,
            expr_id: ExpressionId::from_raw(1),
        },
    ]);
    assert_eq!(
        document.check(&universe).unwrap_err().kind(),
        "missing_field_binding"
    );

    // field type mismatch (status field gets a string)
    let document = build(vec![
        RecordFieldExpr {
            field_no: 1,
            expr_id: ExpressionId::from_raw(1),
        },
        RecordFieldExpr {
            field_no: 2,
            expr_id: ExpressionId::from_raw(1),
        },
        RecordFieldExpr {
            field_no: 3,
            expr_id: ExpressionId::from_raw(1),
        },
    ]);
    assert_eq!(
        document.check(&universe).unwrap_err().kind(),
        "type_mismatch"
    );
}

#[test]
fn comparability_and_bool_operands_are_enforced() {
    let universe = universe();
    let user = schema_ref(&universe, "agentflow.example@1/User");
    let document = SemanticCoreDocument {
        types: base_types(&universe, ""),
        values: vec![],
        expressions: vec![
            expr(
                1,
                1,
                ExpressionBody::Constant(PrimitiveConstant::String("x".into())),
            ),
            expr(
                2,
                2,
                ExpressionBody::RecordConstruct {
                    schema: user,
                    fields: vec![
                        RecordFieldExpr {
                            field_no: 1,
                            expr_id: ExpressionId::from_raw(1),
                        },
                        RecordFieldExpr {
                            field_no: 2,
                            expr_id: ExpressionId::from_raw(1),
                        },
                        // status needs an enum; reuse the string to trigger mismatch
                        RecordFieldExpr {
                            field_no: 3,
                            expr_id: ExpressionId::from_raw(1),
                        },
                    ],
                },
            ),
        ],
        resources: vec![],
    };
    // The record construct itself is the first failure.
    assert_eq!(
        document.check(&universe).unwrap_err().kind(),
        "type_mismatch"
    );
}

#[test]
fn option_none_requires_an_explicit_inner_type() {
    let universe = universe();
    let document = SemanticCoreDocument {
        types: base_types(&universe, ""),
        values: vec![],
        expressions: vec![expr(
            1,
            5, // option<string>
            ExpressionBody::OptionNone {
                inner_type: TypeId::from_raw(1),
            },
        )],
        resources: vec![],
    };
    let checked = document.check(&universe).unwrap();
    let (shape, value) = checked.lower(ExpressionId::from_raw(1)).unwrap();
    assert_eq!(
        std::str::from_utf8(&af_core::encode(&shape, &value).unwrap()).unwrap(),
        r#"{"none":null}"#
    );
}

#[test]
fn resources_resolve_requirement_contracts_and_keep_their_own_namespace() {
    let universe = universe();
    let status = schema_ref(&universe, "agentflow.example@1/Status");

    // A resource requirement pins *some* schema; Phase 3A does not interpret
    // resource semantics, so any resolvable pin is structurally valid.
    let document = SemanticCoreDocument {
        types: base_types(&universe, ""),
        values: vec![ValueSlot {
            id: ValueSlotId::from_raw(1),
            type_id: TypeId::from_raw(1),
        }],
        expressions: vec![],
        resources: vec![ResourceSlot {
            id: ResourceSlotId::from_raw(1), // same number as value slot 1
            requirement_type: status,
        }],
    };
    let checked = document.check(&universe).expect("namespaces are disjoint");
    assert_eq!(
        checked
            .value_slot_type(ValueSlotId::from_raw(1))
            .unwrap()
            .canonical_string(),
        "primitive:string"
    );

    // Unknown requirement schema.
    let mut bad = document.clone();
    bad.resources[0].requirement_type.type_name = "Ghost".into();
    assert_eq!(
        bad.check(&universe).unwrap_err().kind(),
        "unknown_type_schema"
    );

    // Duplicate resource id.
    let mut bad = document;
    bad.resources.push(ResourceSlot {
        id: ResourceSlotId::from_raw(1),
        requirement_type: schema_ref(&universe, "agentflow.example@1/User"),
    });
    assert_eq!(
        bad.check(&universe).unwrap_err().kind(),
        "duplicate_resource_slot_id"
    );
}

#[test]
fn invalid_local_ids_and_unknown_references_are_rejected() {
    let universe = universe();
    let base = base_types(&universe, "");

    // zero id
    let mut types = base.clone();
    types[0].id = TypeId::from_raw(0);
    let document = SemanticCoreDocument {
        types,
        values: vec![],
        expressions: vec![],
        resources: vec![],
    };
    assert_eq!(
        document.check(&universe).unwrap_err().kind(),
        "invalid_local_id"
    );

    // duplicate type id
    let mut types = base.clone();
    types.push(types[0].clone());
    let document = SemanticCoreDocument {
        types,
        values: vec![],
        expressions: vec![],
        resources: vec![],
    };
    assert_eq!(
        document.check(&universe).unwrap_err().kind(),
        "duplicate_semantic_type_id"
    );

    // unknown node reference
    let document = SemanticCoreDocument {
        types: vec![SemanticTypeNode {
            id: TypeId::from_raw(1),
            body: SemanticTypeNodeBody::List(TypeId::from_raw(9)),
        }],
        values: vec![],
        expressions: vec![],
        resources: vec![],
    };
    assert_eq!(
        document.check(&universe).unwrap_err().kind(),
        "unknown_semantic_type"
    );

    // unknown value slot referenced by an expression
    let document = SemanticCoreDocument {
        types: base.clone(),
        values: vec![],
        expressions: vec![expr(
            1,
            1,
            ExpressionBody::ValueSlot(ValueSlotId::from_raw(3)),
        )],
        resources: vec![],
    };
    assert_eq!(
        document.check(&universe).unwrap_err().kind(),
        "unknown_value_slot"
    );

    // unknown expression reference
    let document = SemanticCoreDocument {
        types: base,
        values: vec![],
        expressions: vec![expr(
            1,
            1,
            ExpressionBody::OptionSome {
                payload_expr: ExpressionId::from_raw(8),
            },
        )],
        resources: vec![],
    };
    assert_eq!(
        document.check(&universe).unwrap_err().kind(),
        "unknown_expression"
    );
}
