//! The validated semantic core document model, its type checker and the ACE1
//! lowering used for golden bytes.
//!
//! This module owns the Phase 3A aggregate — semantic type DAG, value slots,
//! pure expressions and logical resource slots — and the type checker that
//! makes it well-formed:
//!
//! ```text
//! SemanticCoreDocument
//!   ├── types       : list<SemanticTypeNode>   (document-local type DAG)
//!   ├── values      : list<ValueSlot>          (immutable SSA slots)
//!   ├── expressions : list<ExpressionDef>      (pure DAG)
//!   └── resources   : list<ResourceSlot>       (logical requirements)
//! ```
//!
//! It is deliberately **not** a `SemanticIRDocument`: there are no workflow
//! computation nodes, no dataflow edges, no control flow and no document hash.
//! A `SemanticCoreDocument` exists so the core objects can be validated and
//! frozen in golden corpora; Phase 3B builds the real semantic IR on top.
//!
//! # Validation model
//!
//! [`SemanticCoreDocument::check`] runs, in order:
//!
//! 1. local id rules (`> 0`, unique) for all four namespaces;
//! 2. semantic type DAG construction (every `TypeSchemaRef` pinned exactly,
//!    every node reference resolved, no type cycles);
//! 3. value slot type resolution;
//! 4. duplicate expression ids, then expression checks (references exist,
//!    cycles absent, and `derived_type == declared result_type`);
//! 5. resource slot requirements resolve.
//!
//! [`CheckedDocument::infer_expression_type`] exposes the type checker for a
//! single expression of an already-validated document.

use crate::error::SemanticError;
use crate::expressions::{ExprTable, ExpressionBody, ExpressionDef, RecordFieldExpr};
use crate::ids::{ExpressionId, ResourceSlotId, SemanticTypeId, ValueSlotId};
use crate::resources::ResourceSlot;
use crate::types::{SemanticTypeNode, SemanticTypeNodeBody, StructuralType};
use crate::universe::{SchemaEntry, TypeSchemaUniverse};
use crate::values::{PrimitiveConstant, ValueSlot};
use af_core::{Shape, Value};
use af_idl::CanonicalBody;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

/// The Phase 3A semantic core aggregate.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticCoreDocument {
    pub types: Vec<SemanticTypeNode>,
    pub values: Vec<ValueSlot>,
    pub expressions: Vec<ExpressionDef>,
    pub resources: Vec<ResourceSlot>,
}

/// A validated semantic core document: the source object plus the resolved
/// type DAG, slot types, derived expression types and canonical orderings.
/// Construction is only possible through [`SemanticCoreDocument::check`], so
/// any value of this type is well-formed.
#[derive(Debug, Clone)]
pub struct CheckedDocument<'a> {
    source: &'a SemanticCoreDocument,
    universe: &'a TypeSchemaUniverse,
    /// Resolved structural type of every declared type id.
    resolved_types: BTreeMap<SemanticTypeId, StructuralType>,
    /// Type of every value slot.
    slot_types: BTreeMap<ValueSlotId, StructuralType>,
    /// Derived type of every expression.
    expression_types: BTreeMap<ExpressionId, StructuralType>,
    /// Expressions in canonical (id ascending) order.
    canonical_expressions: Vec<ExpressionDef>,
    /// Type nodes in canonical (id ascending) order.
    canonical_types: Vec<SemanticTypeNode>,
}

impl SemanticCoreDocument {
    /// Validates this document against an explicit type schema universe.
    /// Returns a [`CheckedDocument`] on success.
    ///
    /// # Errors
    /// Returns the first [`SemanticError`] encountered, in the order described
    /// in the module documentation.
    pub fn check<'a>(
        &'a self,
        universe: &'a TypeSchemaUniverse,
    ) -> Result<CheckedDocument<'a>, SemanticError> {
        let resolved_types = self.check_types(universe)?;
        let slot_types = self.check_value_slots(&resolved_types)?;
        let expression_types = self.check_expressions(universe, &resolved_types, &slot_types)?;
        self.check_resources(universe)?;

        let mut canonical_types = self.types.clone();
        canonical_types.sort_by_key(|node| node.id);
        let mut canonical_expressions = self.expressions.clone();
        canonical_expressions.sort_by_key(|def| def.id);
        Ok(CheckedDocument {
            source: self,
            universe,
            resolved_types,
            slot_types,
            expression_types,
            canonical_expressions,
            canonical_types,
        })
    }

    // -- 1 + 2: type DAG -----------------------------------------------------

    fn check_types(
        &self,
        universe: &TypeSchemaUniverse,
    ) -> Result<BTreeMap<SemanticTypeId, StructuralType>, SemanticError> {
        let mut by_id: BTreeMap<SemanticTypeId, &SemanticTypeNode> = BTreeMap::new();
        for node in &self.types {
            if !node.id.is_valid() {
                return Err(SemanticError::InvalidLocalId {
                    namespace: SemanticTypeId::namespace(),
                    id: node.id.raw(),
                });
            }
            if by_id.insert(node.id, node).is_some() {
                return Err(SemanticError::DuplicateSemanticTypeId { id: node.id.raw() });
            }
        }
        let mut resolved: BTreeMap<SemanticTypeId, StructuralType> = BTreeMap::new();
        let mut in_progress: BTreeSet<SemanticTypeId> = BTreeSet::new();
        for id in by_id.keys() {
            resolve_type(universe, &by_id, *id, &mut resolved, &mut in_progress)?;
        }
        Ok(resolved)
    }

    // -- 3: value slots ------------------------------------------------------

    fn check_value_slots(
        &self,
        resolved_types: &BTreeMap<SemanticTypeId, StructuralType>,
    ) -> Result<BTreeMap<ValueSlotId, StructuralType>, SemanticError> {
        let mut slots: BTreeMap<ValueSlotId, StructuralType> = BTreeMap::new();
        for slot in &self.values {
            if !slot.id.is_valid() {
                return Err(SemanticError::InvalidLocalId {
                    namespace: ValueSlotId::namespace(),
                    id: slot.id.raw(),
                });
            }
            let structural =
                resolved_types
                    .get(&slot.type_id)
                    .ok_or(SemanticError::UnknownSemanticType {
                        id: slot.type_id.raw(),
                    })?;
            if slots.insert(slot.id, structural.clone()).is_some() {
                return Err(SemanticError::DuplicateValueSlotId { id: slot.id.raw() });
            }
        }
        Ok(slots)
    }

    // -- 4: expressions ------------------------------------------------------

    fn check_expressions(
        &self,
        universe: &TypeSchemaUniverse,
        resolved_types: &BTreeMap<SemanticTypeId, StructuralType>,
        slot_types: &BTreeMap<ValueSlotId, StructuralType>,
    ) -> Result<BTreeMap<ExpressionId, StructuralType>, SemanticError> {
        let table = ExprTable::from_defs(self.expressions.clone());
        if let Some(id) = table.duplicates().first() {
            return Err(SemanticError::DuplicateExpressionId { id: id.raw() });
        }
        for def in table.iter() {
            if !def.id.is_valid() {
                return Err(SemanticError::InvalidLocalId {
                    namespace: ExpressionId::namespace(),
                    id: def.id.raw(),
                });
            }
            for referenced in ExprTable::references(&def.body) {
                if table.get(referenced).is_none() {
                    return Err(SemanticError::UnknownExpression {
                        id: referenced.raw(),
                    });
                }
            }
        }
        if let Some(id) = table.find_cycle() {
            return Err(SemanticError::ExpressionCycle { id: id.raw() });
        }
        let mut checker = Checker {
            table: &table,
            universe,
            resolved_types,
            slot_types,
            memo: BTreeMap::new(),
        };
        let mut derived: BTreeMap<ExpressionId, StructuralType> = BTreeMap::new();
        for def in table.iter() {
            let actual = checker.expression_type(def.id)?;
            derived.insert(def.id, actual);
        }
        for def in table.iter() {
            let declared = resolved_types.get(&def.result_type_id).ok_or(
                SemanticError::UnknownSemanticType {
                    id: def.result_type_id.raw(),
                },
            )?;
            let actual = derived.get(&def.id).expect("derived for every id");
            if declared != actual {
                return Err(SemanticError::ExpressionResultTypeMismatch {
                    expression: def.id.raw(),
                    declared: declared.canonical_string(),
                    derived: actual.canonical_string(),
                });
            }
        }
        Ok(derived)
    }

    // -- 5: resources --------------------------------------------------------

    fn check_resources(&self, universe: &TypeSchemaUniverse) -> Result<(), SemanticError> {
        let mut seen: BTreeSet<ResourceSlotId> = BTreeSet::new();
        for slot in &self.resources {
            if !slot.id.is_valid() {
                return Err(SemanticError::InvalidLocalId {
                    namespace: ResourceSlotId::namespace(),
                    id: slot.id.raw(),
                });
            }
            if !seen.insert(slot.id) {
                return Err(SemanticError::DuplicateResourceSlotId { id: slot.id.raw() });
            }
            // A resource requirement is a pinned contract schema, not a value
            // type: it resolves through its own `TypeSchemaRef`.
            universe.resolve(&slot.requirement_type)?;
        }
        Ok(())
    }
}

/// Resolves one type DAG node with memoization and cycle detection.
fn resolve_type(
    universe: &TypeSchemaUniverse,
    by_id: &BTreeMap<SemanticTypeId, &SemanticTypeNode>,
    id: SemanticTypeId,
    resolved: &mut BTreeMap<SemanticTypeId, StructuralType>,
    in_progress: &mut BTreeSet<SemanticTypeId>,
) -> Result<StructuralType, SemanticError> {
    if let Some(existing) = resolved.get(&id) {
        return Ok(existing.clone());
    }
    if !in_progress.insert(id) {
        return Err(SemanticError::SemanticTypeCycle { id: id.raw() });
    }
    let Some(node) = by_id.get(&id) else {
        return Err(SemanticError::UnknownSemanticType { id: id.raw() });
    };
    let structural = match &node.body {
        SemanticTypeNodeBody::Primitive(primitive) => StructuralType::Primitive(*primitive),
        SemanticTypeNodeBody::Schema(reference) => {
            // The pin must resolve exactly; aliases then normalize to their
            // target, because a value of an alias type is a value of the
            // target type.
            universe.resolve(reference)?;
            universe.expand_named(&reference.identity())?
        }
        SemanticTypeNodeBody::List(inner) => StructuralType::List(Box::new(resolve_type(
            universe,
            by_id,
            *inner,
            resolved,
            in_progress,
        )?)),
        SemanticTypeNodeBody::MapString(inner) => StructuralType::MapString(Box::new(
            resolve_type(universe, by_id, *inner, resolved, in_progress)?,
        )),
        SemanticTypeNodeBody::Option(inner) => StructuralType::Option(Box::new(resolve_type(
            universe,
            by_id,
            *inner,
            resolved,
            in_progress,
        )?)),
    };
    in_progress.remove(&id);
    resolved.insert(id, structural.clone());
    Ok(structural)
}

/// The type checker proper: derives the structural type of an expression from
/// the declared types of its subexpressions.
struct Checker<'a> {
    table: &'a ExprTable,
    universe: &'a TypeSchemaUniverse,
    resolved_types: &'a BTreeMap<SemanticTypeId, StructuralType>,
    slot_types: &'a BTreeMap<ValueSlotId, StructuralType>,
    memo: BTreeMap<ExpressionId, StructuralType>,
}

impl Checker<'_> {
    fn expression_type(&mut self, id: ExpressionId) -> Result<StructuralType, SemanticError> {
        if let Some(existing) = self.memo.get(&id) {
            return Ok(existing.clone());
        }
        let def = self
            .table
            .get(id)
            .ok_or(SemanticError::UnknownExpression { id: id.raw() })?;
        let derived = self.body_type(id, &def.body)?;
        self.memo.insert(id, derived.clone());
        Ok(derived)
    }

    fn operand(&mut self, id: ExpressionId) -> Result<StructuralType, SemanticError> {
        self.expression_type(id)
    }

    fn type_of(&self, id: SemanticTypeId) -> Result<StructuralType, SemanticError> {
        self.resolved_types
            .get(&id)
            .cloned()
            .ok_or(SemanticError::UnknownSemanticType { id: id.raw() })
    }

    fn bool_type() -> StructuralType {
        StructuralType::Primitive(crate::types::SemanticPrimitiveType::Bool)
    }

    fn body_type(
        &mut self,
        self_id: ExpressionId,
        body: &ExpressionBody,
    ) -> Result<StructuralType, SemanticError> {
        match body {
            ExpressionBody::Constant(constant) => {
                constant.validate()?;
                Ok(StructuralType::Primitive(constant.semantic_type()))
            }
            ExpressionBody::ValueSlot(slot) => self
                .slot_types
                .get(slot)
                .cloned()
                .ok_or(SemanticError::UnknownValueSlot { id: slot.raw() }),
            ExpressionBody::RecordConstruct { schema, fields } => {
                let entry = self.universe.resolve(schema)?;
                let CanonicalBody::Record(declared) = &entry.body else {
                    return Err(SemanticError::ExpectedRecordSchema {
                        identity: entry.identity(),
                    });
                };
                let numbers: Vec<u32> = declared.iter().map(|field| field.no).collect();
                let mut seen: BTreeSet<u32> = BTreeSet::new();
                for field in fields {
                    if !seen.insert(field.field_no) {
                        return Err(SemanticError::DuplicateFieldBinding {
                            identity: entry.identity(),
                            field_no: field.field_no,
                        });
                    }
                    let expected = self.universe.field_type(entry, field.field_no)?;
                    let actual = self.operand(field.expr_id)?;
                    if expected != actual {
                        return Err(SemanticError::TypeMismatch {
                            context: format!(
                                "record_construct field {} of {}",
                                field.field_no,
                                entry.identity()
                            ),
                            expected: expected.canonical_string(),
                            actual: actual.canonical_string(),
                        });
                    }
                }
                for field_no in numbers {
                    if !seen.contains(&field_no) {
                        return Err(SemanticError::MissingFieldBinding {
                            identity: entry.identity(),
                            field_no,
                        });
                    }
                }
                Ok(StructuralType::Schema(entry.reference()))
            }
            ExpressionBody::VariantConstruct {
                schema,
                case_no,
                payload_expr,
            } => {
                let entry = self.universe.resolve(schema)?;
                let expected = self.universe.case_payload_type(entry, *case_no)?;
                let actual = self.operand(*payload_expr)?;
                if expected != actual {
                    return Err(SemanticError::TypeMismatch {
                        context: format!(
                            "variant_construct case {case_no} of {}",
                            entry.identity()
                        ),
                        expected: expected.canonical_string(),
                        actual: actual.canonical_string(),
                    });
                }
                Ok(StructuralType::Schema(entry.reference()))
            }
            ExpressionBody::EnumValue { schema, value_no } => {
                let entry = self.universe.resolve(schema)?;
                let CanonicalBody::Enum(values) = &entry.body else {
                    return Err(SemanticError::ExpectedEnumSchema {
                        identity: entry.identity(),
                    });
                };
                if !values.iter().any(|value| value.no == *value_no) {
                    return Err(SemanticError::UnknownEnumValue {
                        identity: entry.identity(),
                        value_no: *value_no,
                    });
                }
                Ok(StructuralType::Schema(entry.reference()))
            }
            ExpressionBody::ListConstruct {
                element_type,
                elements,
            } => {
                let element = self.type_of(*element_type)?;
                for element_expr in elements {
                    let actual = self.operand(*element_expr)?;
                    if actual != element {
                        return Err(SemanticError::TypeMismatch {
                            context: format!("{self_id}.list_construct element"),
                            expected: element.canonical_string(),
                            actual: actual.canonical_string(),
                        });
                    }
                }
                Ok(StructuralType::List(Box::new(element)))
            }
            ExpressionBody::MapConstruct {
                value_type,
                entries,
            } => {
                let value = self.type_of(*value_type)?;
                // Canonical key rules are enforced here, not deferred to value
                // encoding: NFC-normalize, reject post-NFC duplicates, sort by
                // unsigned UTF-8 bytes.
                ExprTable::canonical_map_entries(entries)?;
                for entry in entries {
                    let actual = self.operand(entry.value_expr)?;
                    if actual != value {
                        return Err(SemanticError::TypeMismatch {
                            context: format!("{self_id}.map_construct entry {:?}", entry.key),
                            expected: value.canonical_string(),
                            actual: actual.canonical_string(),
                        });
                    }
                }
                Ok(StructuralType::MapString(Box::new(value)))
            }
            ExpressionBody::OptionNone { inner_type } => {
                Ok(StructuralType::Option(Box::new(self.type_of(*inner_type)?)))
            }
            ExpressionBody::OptionSome { payload_expr } => {
                let payload = self.operand(*payload_expr)?;
                Ok(StructuralType::Option(Box::new(payload)))
            }
            ExpressionBody::FieldGet { record, field_no } => {
                let input = self.operand(*record)?;
                let StructuralType::Schema(reference) = &input else {
                    return Err(SemanticError::ExpectedRecordSchema {
                        identity: input.canonical_string(),
                    });
                };
                let entry = self.universe.resolve(reference)?;
                self.universe.field_type(entry, *field_no)
            }
            ExpressionBody::Equals { left, right } => {
                let left_type = self.operand(*left)?;
                let right_type = self.operand(*right)?;
                if left_type != right_type {
                    return Err(SemanticError::TypeMismatch {
                        context: format!("{self_id}.equals"),
                        expected: left_type.canonical_string(),
                        actual: right_type.canonical_string(),
                    });
                }
                self.require_comparable(&left_type)?;
                Ok(Self::bool_type())
            }
            ExpressionBody::Not { operand } => {
                self.require_bool(*operand, "not")?;
                Ok(Self::bool_type())
            }
            ExpressionBody::And { left, right } => {
                self.require_bool(*left, "and")?;
                self.require_bool(*right, "and")?;
                Ok(Self::bool_type())
            }
            ExpressionBody::Or { left, right } => {
                self.require_bool(*left, "or")?;
                self.require_bool(*right, "or")?;
                Ok(Self::bool_type())
            }
            ExpressionBody::IfThenElse {
                condition,
                then_expr,
                else_expr,
            } => {
                self.require_bool(*condition, "if_then_else condition")?;
                let then_type = self.operand(*then_expr)?;
                let else_type = self.operand(*else_expr)?;
                if then_type != else_type {
                    return Err(SemanticError::TypeMismatch {
                        context: format!("{self_id}.if_then_else branches"),
                        expected: then_type.canonical_string(),
                        actual: else_type.canonical_string(),
                    });
                }
                Ok(then_type)
            }
        }
    }

    fn require_bool(&mut self, id: ExpressionId, context: &str) -> Result<(), SemanticError> {
        let actual = self.operand(id)?;
        let expected = Self::bool_type();
        if actual != expected {
            return Err(SemanticError::TypeMismatch {
                context: format!("{context} expects bool"),
                expected: expected.canonical_string(),
                actual: actual.canonical_string(),
            });
        }
        Ok(())
    }

    fn require_comparable(&self, operand: &StructuralType) -> Result<(), SemanticError> {
        match operand {
            StructuralType::Primitive(_) => Ok(()),
            StructuralType::Schema(reference) => {
                let entry = self.universe.resolve(reference)?;
                if self.universe.is_enum(entry) {
                    Ok(())
                } else {
                    Err(SemanticError::TypeNotComparable {
                        type_repr: operand.canonical_string(),
                    })
                }
            }
            StructuralType::List(_) | StructuralType::MapString(_) | StructuralType::Option(_) => {
                Err(SemanticError::TypeNotComparable {
                    type_repr: operand.canonical_string(),
                })
            }
        }
    }
}

impl CheckedDocument<'_> {
    /// The structural type of one expression of this validated document.
    ///
    /// # Errors
    /// [`SemanticError::UnknownExpression`] when the id is not declared.
    pub fn infer_expression_type(&self, id: ExpressionId) -> Result<StructuralType, SemanticError> {
        self.expression_types
            .get(&id)
            .cloned()
            .ok_or(SemanticError::UnknownExpression { id: id.raw() })
    }

    /// The structural type of one value slot.
    ///
    /// # Errors
    /// [`SemanticError::UnknownValueSlot`] when the id is not declared.
    pub fn value_slot_type(&self, id: ValueSlotId) -> Result<StructuralType, SemanticError> {
        self.slot_types
            .get(&id)
            .cloned()
            .ok_or(SemanticError::UnknownValueSlot { id: id.raw() })
    }

    /// The structural type of one declared type id.
    ///
    /// # Errors
    /// [`SemanticError::UnknownSemanticType`] when the id is not declared.
    pub fn type_of(&self, id: SemanticTypeId) -> Result<StructuralType, SemanticError> {
        self.resolved_types
            .get(&id)
            .cloned()
            .ok_or(SemanticError::UnknownSemanticType { id: id.raw() })
    }

    /// Expressions in canonical order (id ascending).
    pub fn canonical_expressions(&self) -> &[ExpressionDef] {
        &self.canonical_expressions
    }

    /// Type nodes in canonical order (id ascending).
    pub fn canonical_types(&self) -> &[SemanticTypeNode] {
        &self.canonical_types
    }

    /// The source document.
    pub fn source(&self) -> &SemanticCoreDocument {
        self.source
    }

    /// Lowers one well-typed expression into an ACE1 `(Shape, Value)` pair,
    /// treating value slots as unbound.
    ///
    /// # Errors
    /// As [`CheckedDocument::lower_with`].
    pub fn lower(&self, id: ExpressionId) -> Result<(Shape, Value), SemanticError> {
        self.lower_with(id, &BTreeMap::new())
    }

    /// Lowers one well-typed expression into an ACE1 `(Shape, Value)` pair.
    ///
    /// This is the bridge that makes golden bytes possible without inventing a
    /// new hash domain: the result is an ordinary ACE1 encoding of the value
    /// the expression denotes, under the shape derived from the referenced
    /// AFIDL schema.
    ///
    /// `bindings` supplies the concrete values of value slots (Phase 3A has no
    /// producers, so a slot's value is only known when a harness supplies it).
    /// A binding is validated against the slot's declared type through ACE1
    /// itself.
    ///
    /// # Errors
    /// - [`SemanticError::UnknownExpression`] for an undeclared expression;
    /// - [`SemanticError::ValueSlotNotBound`] when a reached slot has no binding;
    /// - [`SemanticError::TypeMismatch`] when a binding does not conform to the
    ///   slot's declared type.
    pub fn lower_with(
        &self,
        id: ExpressionId,
        bindings: &BTreeMap<ValueSlotId, Value>,
    ) -> Result<(Shape, Value), SemanticError> {
        let def = self
            .source
            .expressions
            .iter()
            .find(|def| def.id == id)
            .ok_or(SemanticError::UnknownExpression { id: id.raw() })?;
        self.lower_body(&def.body, bindings)
    }

    fn lower_body(
        &self,
        body: &ExpressionBody,
        bindings: &BTreeMap<ValueSlotId, Value>,
    ) -> Result<(Shape, Value), SemanticError> {
        match body {
            ExpressionBody::Constant(constant) => lower_constant(constant),
            ExpressionBody::ValueSlot(slot_id) => {
                let structural = self.value_slot_type(*slot_id)?;
                let shape = self.shape_of_structural(&structural)?;
                let Some(value) = bindings.get(slot_id) else {
                    return Err(SemanticError::ValueSlotNotBound { id: slot_id.raw() });
                };
                // Validate the binding through ACE1 itself: the supplied value
                // must encode under the slot's declared shape.
                af_core::encode(&shape, value).map_err(|error| SemanticError::TypeMismatch {
                    context: format!("value slot {slot_id} binding"),
                    expected: shape.type_name(),
                    actual: error.to_string(),
                })?;
                Ok((shape, value.clone()))
            }
            ExpressionBody::RecordConstruct { schema, fields } => {
                let entry = self.universe.resolve(schema)?;
                let CanonicalBody::Record(declared) = &entry.body else {
                    return Err(SemanticError::ExpectedRecordSchema {
                        identity: entry.identity(),
                    });
                };
                let shape = self.shape_of_named(entry)?;
                let Shape::Record(record_shape) = &shape else {
                    return Err(SemanticError::ExpectedRecordSchema {
                        identity: entry.identity(),
                    });
                };
                debug_assert_eq!(record_shape.fields.len(), declared.len());
                let mut ordered: Vec<&RecordFieldExpr> = fields.iter().collect();
                ordered.sort_by_key(|field| field.field_no);
                let mut values = Vec::with_capacity(ordered.len());
                for binding in ordered {
                    let index = declared
                        .iter()
                        .position(|field| field.no == binding.field_no)
                        .ok_or(SemanticError::UnknownFieldNumber {
                            identity: entry.identity(),
                            field_no: binding.field_no,
                        })?;
                    let (_, value) = self.lower_with(binding.expr_id, bindings)?;
                    values.push((record_shape.fields[index].name.clone(), value));
                }
                Ok((shape, Value::Record(values)))
            }
            ExpressionBody::VariantConstruct {
                schema,
                case_no,
                payload_expr,
            } => {
                let entry = self.universe.resolve(schema)?;
                let CanonicalBody::Variant(cases) = &entry.body else {
                    return Err(SemanticError::ExpectedVariantSchema {
                        identity: entry.identity(),
                    });
                };
                let case = cases.iter().find(|case| case.no == *case_no).ok_or(
                    SemanticError::UnknownVariantCase {
                        identity: entry.identity(),
                        case_no: *case_no,
                    },
                )?;
                let shape = self.shape_of_named(entry)?;
                let (_, payload) = self.lower_with(*payload_expr, bindings)?;
                Ok((
                    shape,
                    Value::Variant {
                        case: case.name.clone(),
                        value: Box::new(payload),
                    },
                ))
            }
            ExpressionBody::EnumValue { schema, value_no } => {
                let entry = self.universe.resolve(schema)?;
                let CanonicalBody::Enum(values) = &entry.body else {
                    return Err(SemanticError::ExpectedEnumSchema {
                        identity: entry.identity(),
                    });
                };
                let value = values.iter().find(|value| value.no == *value_no).ok_or(
                    SemanticError::UnknownEnumValue {
                        identity: entry.identity(),
                        value_no: *value_no,
                    },
                )?;
                let shape = self.shape_of_named(entry)?;
                Ok((shape, Value::Enum(value.name.clone())))
            }
            ExpressionBody::ListConstruct {
                element_type,
                elements,
            } => {
                let shape = self.shape_of_type(*element_type)?;
                let mut values = Vec::with_capacity(elements.len());
                for element in elements {
                    let (_, value) = self.lower_with(*element, bindings)?;
                    values.push(value);
                }
                Ok((Shape::List(Box::new(shape)), Value::List(values)))
            }
            ExpressionBody::MapConstruct {
                value_type,
                entries,
            } => {
                let shape = self.shape_of_type(*value_type)?;
                let canonical = ExprTable::canonical_map_entries(entries)?;
                let mut values = Vec::with_capacity(canonical.len());
                for entry in canonical {
                    let (_, value) = self.lower_with(entry.value_expr, bindings)?;
                    values.push((entry.key, value));
                }
                Ok((Shape::Map(Box::new(shape)), Value::Map(values)))
            }
            ExpressionBody::OptionNone { inner_type } => {
                let shape = self.shape_of_type(*inner_type)?;
                Ok((Shape::Option(Box::new(shape)), Value::Option(None)))
            }
            ExpressionBody::OptionSome { payload_expr } => {
                let (shape, value) = self.lower_with(*payload_expr, bindings)?;
                Ok((
                    Shape::Option(Box::new(shape)),
                    Value::Option(Some(Box::new(value))),
                ))
            }
            ExpressionBody::FieldGet { record, field_no } => {
                // The input's derived type is a pinned schema reference; the
                // field is selected by number and its name comes from the
                // schema declaration (names are value-encoding keys only).
                let input_type = self.infer_expression_type(*record)?;
                let StructuralType::Schema(reference) = &input_type else {
                    return Err(SemanticError::ExpectedRecordSchema {
                        identity: input_type.canonical_string(),
                    });
                };
                let entry = self.universe.resolve(reference)?;
                let CanonicalBody::Record(declared) = &entry.body else {
                    return Err(SemanticError::ExpectedRecordSchema {
                        identity: entry.identity(),
                    });
                };
                let field = declared.iter().find(|field| field.no == *field_no).ok_or(
                    SemanticError::UnknownFieldNumber {
                        identity: entry.identity(),
                        field_no: *field_no,
                    },
                )?;
                let field_shape =
                    self.shape_of_structural(&self.universe.expand(&field.type_expr)?)?;
                let (_, value) = self.lower_with(*record, bindings)?;
                let Value::Record(members) = value else {
                    return Err(SemanticError::ExpectedRecordSchema {
                        identity: entry.identity(),
                    });
                };
                let selected = members
                    .into_iter()
                    .find(|(name, _)| *name == field.name)
                    .map(|(_, field_value)| field_value)
                    .ok_or(SemanticError::UnknownFieldNumber {
                        identity: entry.identity(),
                        field_no: *field_no,
                    })?;
                Ok((field_shape, selected))
            }
            ExpressionBody::Equals { left, right } => {
                let (_, left_value) = self.lower_with(*left, bindings)?;
                let (_, right_value) = self.lower_with(*right, bindings)?;
                Ok((Shape::Bool, Value::Bool(left_value == right_value)))
            }
            ExpressionBody::Not { operand } => {
                let value = self.lower_bool(*operand, bindings, "not")?;
                Ok((Shape::Bool, Value::Bool(!value)))
            }
            ExpressionBody::And { left, right } => {
                let a = self.lower_bool(*left, bindings, "and")?;
                let b = self.lower_bool(*right, bindings, "and")?;
                Ok((Shape::Bool, Value::Bool(a && b)))
            }
            ExpressionBody::Or { left, right } => {
                let a = self.lower_bool(*left, bindings, "or")?;
                let b = self.lower_bool(*right, bindings, "or")?;
                Ok((Shape::Bool, Value::Bool(a || b)))
            }
            ExpressionBody::IfThenElse {
                condition,
                then_expr,
                else_expr,
            } => {
                let condition = self.lower_bool(*condition, bindings, "if_then_else")?;
                if condition {
                    self.lower_with(*then_expr, bindings)
                } else {
                    self.lower_with(*else_expr, bindings)
                }
            }
        }
    }

    fn lower_bool(
        &self,
        id: ExpressionId,
        bindings: &BTreeMap<ValueSlotId, Value>,
        context: &str,
    ) -> Result<bool, SemanticError> {
        let (_, value) = self.lower_with(id, bindings)?;
        let Value::Bool(flag) = value else {
            return Err(SemanticError::TypeMismatch {
                context: context.into(),
                expected: "bool".into(),
                actual: value.type_tag(),
            });
        };
        Ok(flag)
    }

    fn shape_of_type(&self, id: SemanticTypeId) -> Result<Shape, SemanticError> {
        let structural = self.type_of(id)?;
        self.shape_of_structural(&structural)
    }

    fn shape_of_structural(&self, structural: &StructuralType) -> Result<Shape, SemanticError> {
        match structural {
            StructuralType::Primitive(primitive) => Ok(primitive_shape(*primitive)),
            StructuralType::Schema(reference) => {
                let entry = self.universe.resolve(reference)?;
                match &entry.body {
                    CanonicalBody::Alias(target) => {
                        let structural = self.universe.expand(target)?;
                        self.shape_of_structural(&structural)
                    }
                    _ => self.shape_of_named(entry),
                }
            }
            StructuralType::List(inner) => {
                Ok(Shape::List(Box::new(self.shape_of_structural(inner)?)))
            }
            StructuralType::MapString(inner) => {
                Ok(Shape::Map(Box::new(self.shape_of_structural(inner)?)))
            }
            StructuralType::Option(inner) => {
                Ok(Shape::Option(Box::new(self.shape_of_structural(inner)?)))
            }
        }
    }

    /// Builds the finite ACE1 shape of a named non-alias type by walking the
    /// canonical body through the universe.
    fn shape_of_named(&self, entry: &SchemaEntry) -> Result<Shape, SemanticError> {
        match &entry.body {
            CanonicalBody::Enum(values) => Ok(Shape::Enum(af_core::EnumShape::new(
                entry.type_name.clone(),
                values.iter().map(|value| value.name.clone()),
            ))),
            CanonicalBody::Record(fields) => {
                let mut shapes = Vec::with_capacity(fields.len());
                for field in fields {
                    let structural = self.universe.expand(&field.type_expr)?;
                    shapes.push((field.name.clone(), self.shape_of_structural(&structural)?));
                }
                Ok(Shape::Record(af_core::RecordShape::new(
                    entry.type_name.clone(),
                    shapes,
                )))
            }
            CanonicalBody::Variant(cases) => {
                let mut shapes = Vec::with_capacity(cases.len());
                for case in cases {
                    let structural = self.universe.expand(&case.type_expr)?;
                    shapes.push((case.name.clone(), self.shape_of_structural(&structural)?));
                }
                Ok(Shape::Variant(af_core::VariantShape::new(
                    entry.type_name.clone(),
                    shapes,
                )))
            }
            CanonicalBody::Alias(target) => {
                let structural = self.universe.expand(target)?;
                self.shape_of_structural(&structural)
            }
        }
    }
}

pub(crate) fn primitive_shape(primitive: crate::types::SemanticPrimitiveType) -> Shape {
    use crate::types::SemanticPrimitiveType as P;
    match primitive {
        P::Unit => Shape::Unit,
        P::Null => Shape::Null,
        P::Bool => Shape::Bool,
        P::Int64 => Shape::Int64,
        P::Decimal => Shape::Decimal,
        P::String => Shape::String,
        P::Bytes => Shape::Bytes,
        P::Timestamp => Shape::Timestamp,
        P::Duration => Shape::Duration,
    }
}

fn lower_constant(constant: &PrimitiveConstant) -> Result<(Shape, Value), SemanticError> {
    let shape = primitive_shape(constant.semantic_type());
    let value = match constant {
        PrimitiveConstant::Unit => Value::Unit,
        PrimitiveConstant::Null => Value::Null,
        PrimitiveConstant::Bool(flag) => Value::Bool(*flag),
        PrimitiveConstant::Int64(number) => Value::Int64(*number),
        PrimitiveConstant::Decimal(decimal) => Value::Decimal(decimal.clone()),
        PrimitiveConstant::String(text) => Value::String(text.clone()),
        PrimitiveConstant::Bytes(text) => {
            let bytes = af_core::base64url::decode(text).map_err(|error| {
                SemanticError::InvalidConstant {
                    detail: format!("bytes constant is not canonical base64url: {error}"),
                }
            })?;
            Value::Bytes(bytes)
        }
        PrimitiveConstant::Timestamp(micros) => Value::Timestamp(*micros),
        PrimitiveConstant::Duration(micros) => Value::Duration(*micros),
    };
    Ok((shape, value))
}
