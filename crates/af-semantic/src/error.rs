//! Structured errors of the semantic core validator.
//!
//! Every variant has a stable [`SemanticError::kind`] string; the frozen
//! conformance corpus asserts on kinds, never on prose. These errors are
//! **compile-time semantic validation** failures — they are unrelated to any
//! future workflow `FailureValue` (which belongs to the runtime layer).

use std::fmt;

/// A semantic-core validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticError {
    /// A keyword or schema type outside the 9-primitive workflow universe
    /// surfaced where a workflow value type is required.
    UnsupportedWorkflowPrimitive {
        keyword: String,
        detail: String,
    },
    /// A local id is zero. `0` is reserved as invalid/unset in every
    /// namespace.
    InvalidLocalId {
        namespace: &'static str,
        id: u32,
    },
    DuplicateSemanticTypeId {
        id: u32,
    },
    DuplicateValueSlotId {
        id: u32,
    },
    DuplicateExpressionId {
        id: u32,
    },
    DuplicateResourceSlotId {
        id: u32,
    },
    /// A semantic type node references an id that is not declared.
    UnknownSemanticType {
        id: u32,
    },
    /// The semantic type DAG contains a cycle.
    SemanticTypeCycle {
        id: u32,
    },
    /// An expression references a value slot that is not declared.
    UnknownValueSlot {
        id: u32,
    },
    /// An expression references an expression that is not declared.
    UnknownExpression {
        id: u32,
    },
    /// The expression graph contains a cycle.
    ExpressionCycle {
        id: u32,
    },
    /// A `TypeSchemaRef` does not resolve in the supplied `TypeSchemaUniverse`.
    UnknownTypeSchema {
        identity: String,
    },
    /// The pinned digest differs from the canonical schema's actual digest.
    TypeSchemaDigestMismatch {
        identity: String,
        expected: String,
        actual: String,
    },
    /// A `DigestRefValue` is malformed (algorithm, domain or hex).
    InvalidDigestRef {
        detail: String,
    },
    /// A constant's payload is not in its canonical form (currently: bytes
    /// must be canonical unpadded base64url).
    InvalidConstant {
        detail: String,
    },
    /// Two values of different structural types were combined.
    TypeMismatch {
        context: String,
        expected: String,
        actual: String,
    },
    /// An expression's declared result type differs from the type derived
    /// from its body.
    ExpressionResultTypeMismatch {
        expression: u32,
        declared: String,
        derived: String,
    },
    /// A record-shaped target was required but the pinned schema is not a
    /// record.
    ExpectedRecordSchema {
        identity: String,
    },
    ExpectedVariantSchema {
        identity: String,
    },
    ExpectedEnumSchema {
        identity: String,
    },
    /// A record construction or field selection names a field number that the
    /// pinned record schema does not declare.
    UnknownFieldNumber {
        identity: String,
        field_no: u32,
    },
    DuplicateFieldBinding {
        identity: String,
        field_no: u32,
    },
    MissingFieldBinding {
        identity: String,
        field_no: u32,
    },
    UnknownVariantCase {
        identity: String,
        case_no: u32,
    },
    UnknownEnumValue {
        identity: String,
        value_no: u32,
    },
    /// A map construction has two entries whose keys collide after NFC
    /// normalization.
    DuplicateNormalizedMapKey {
        key: String,
    },
    /// `equals` received an operand whose type is not comparable in Phase 3A
    /// (only primitive types and enum schemas are comparable).
    TypeNotComparable {
        type_repr: String,
    },
    /// Lowering reached a value slot for which the caller supplied no
    /// binding. Phase 3A has no producers, so a slot's value is only known
    /// when a harness binds it explicitly.
    ValueSlotNotBound {
        id: u32,
    },
}

impl SemanticError {
    /// Stable error category; the conformance corpus freezes expectations
    /// against these strings.
    pub fn kind(&self) -> &'static str {
        match self {
            SemanticError::UnsupportedWorkflowPrimitive { .. } => "unsupported_workflow_primitive",
            SemanticError::InvalidLocalId { .. } => "invalid_local_id",
            SemanticError::DuplicateSemanticTypeId { .. } => "duplicate_semantic_type_id",
            SemanticError::DuplicateValueSlotId { .. } => "duplicate_value_slot_id",
            SemanticError::DuplicateExpressionId { .. } => "duplicate_expression_id",
            SemanticError::DuplicateResourceSlotId { .. } => "duplicate_resource_slot_id",
            SemanticError::UnknownSemanticType { .. } => "unknown_semantic_type",
            SemanticError::SemanticTypeCycle { .. } => "semantic_type_cycle",
            SemanticError::UnknownValueSlot { .. } => "unknown_value_slot",
            SemanticError::UnknownExpression { .. } => "unknown_expression",
            SemanticError::ExpressionCycle { .. } => "expression_cycle",
            SemanticError::UnknownTypeSchema { .. } => "unknown_type_schema",
            SemanticError::TypeSchemaDigestMismatch { .. } => "type_schema_digest_mismatch",
            SemanticError::InvalidDigestRef { .. } => "invalid_digest_ref",
            SemanticError::InvalidConstant { .. } => "invalid_constant",
            SemanticError::TypeMismatch { .. } => "type_mismatch",
            SemanticError::ExpressionResultTypeMismatch { .. } => "expression_result_type_mismatch",
            SemanticError::ExpectedRecordSchema { .. } => "expected_record_schema",
            SemanticError::ExpectedVariantSchema { .. } => "expected_variant_schema",
            SemanticError::ExpectedEnumSchema { .. } => "expected_enum_schema",
            SemanticError::UnknownFieldNumber { .. } => "unknown_field_number",
            SemanticError::DuplicateFieldBinding { .. } => "duplicate_field_binding",
            SemanticError::MissingFieldBinding { .. } => "missing_field_binding",
            SemanticError::UnknownVariantCase { .. } => "unknown_variant_case",
            SemanticError::UnknownEnumValue { .. } => "unknown_enum_value",
            SemanticError::DuplicateNormalizedMapKey { .. } => "duplicate_normalized_map_key",
            SemanticError::TypeNotComparable { .. } => "type_not_comparable",
            SemanticError::ValueSlotNotBound { .. } => "value_slot_not_bound",
        }
    }
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SemanticError::UnsupportedWorkflowPrimitive { keyword, detail } => write!(
                f,
                "{keyword:?} is not a workflow semantic primitive ({detail})"
            ),
            SemanticError::InvalidLocalId { namespace, id } => {
                write!(f, "{namespace} id {id} is invalid: local ids must be > 0")
            }
            SemanticError::DuplicateSemanticTypeId { id } => {
                write!(f, "duplicate semantic type id {id}")
            }
            SemanticError::DuplicateValueSlotId { id } => write!(f, "duplicate value slot id {id}"),
            SemanticError::DuplicateExpressionId { id } => {
                write!(f, "duplicate expression id {id}")
            }
            SemanticError::DuplicateResourceSlotId { id } => {
                write!(f, "duplicate resource slot id {id}")
            }
            SemanticError::UnknownSemanticType { id } => {
                write!(f, "semantic type id {id} is not declared")
            }
            SemanticError::SemanticTypeCycle { id } => {
                write!(f, "semantic type id {id} participates in a type cycle")
            }
            SemanticError::UnknownValueSlot { id } => {
                write!(f, "value slot id {id} is not declared")
            }
            SemanticError::UnknownExpression { id } => {
                write!(f, "expression id {id} is not declared")
            }
            SemanticError::ExpressionCycle { id } => {
                write!(f, "expression id {id} participates in an expression cycle")
            }
            SemanticError::UnknownTypeSchema { identity } => {
                write!(f, "unknown type schema {identity:?}")
            }
            SemanticError::TypeSchemaDigestMismatch {
                identity,
                expected,
                actual,
            } => write!(
                f,
                "type schema {identity:?} digest mismatch: pinned {expected}, actual {actual}"
            ),
            SemanticError::InvalidDigestRef { detail } => {
                write!(f, "invalid digest reference: {detail}")
            }
            SemanticError::InvalidConstant { detail } => write!(f, "invalid constant: {detail}"),
            SemanticError::TypeMismatch {
                context,
                expected,
                actual,
            } => write!(
                f,
                "type mismatch in {context}: expected {expected}, got {actual}"
            ),
            SemanticError::ExpressionResultTypeMismatch {
                expression,
                declared,
                derived,
            } => write!(
                f,
                "expression {expression} declares result type {declared} but its body derives {derived}"
            ),
            SemanticError::ExpectedRecordSchema { identity } => {
                write!(f, "{identity:?} is not a record schema")
            }
            SemanticError::ExpectedVariantSchema { identity } => {
                write!(f, "{identity:?} is not a variant schema")
            }
            SemanticError::ExpectedEnumSchema { identity } => {
                write!(f, "{identity:?} is not an enum schema")
            }
            SemanticError::UnknownFieldNumber { identity, field_no } => {
                write!(f, "{identity:?} has no field number {field_no}")
            }
            SemanticError::DuplicateFieldBinding { identity, field_no } => {
                write!(
                    f,
                    "{identity:?} field number {field_no} is bound more than once"
                )
            }
            SemanticError::MissingFieldBinding { identity, field_no } => {
                write!(f, "{identity:?} field number {field_no} is not bound")
            }
            SemanticError::UnknownVariantCase { identity, case_no } => {
                write!(f, "{identity:?} has no case number {case_no}")
            }
            SemanticError::UnknownEnumValue { identity, value_no } => {
                write!(f, "{identity:?} has no value number {value_no}")
            }
            SemanticError::DuplicateNormalizedMapKey { key } => {
                write!(f, "duplicate map key {key:?} after NFC normalization")
            }
            SemanticError::TypeNotComparable { type_repr } => {
                write!(f, "type {type_repr} is not comparable in Phase 3A")
            }
            SemanticError::ValueSlotNotBound { id } => {
                write!(f, "value slot {id} has no binding supplied for lowering")
            }
        }
    }
}

impl std::error::Error for SemanticError {}
