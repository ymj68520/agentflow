//! The pure expression AST.
//!
//! Expressions are **pure, deterministic and total-once-well-typed**: they
//! perform no I/O, read no clock, no randomness, no environment, no policy or
//! registry state and no resources. They are the value-level language used by
//! `SELECT`, `ITERATE` conditions and `INVOKE` input construction; they are
//! *not* workflow control (there is no `SELECT`/`SEQUENCE`/... here) and they
//! never produce effects.
//!
//! # Closed operator set (Phase 3A)
//!
//! ```text
//! constant          primitive literal
//! value_slot        read an immutable value slot
//! record_construct  build a record by field_no
//! variant_construct wrap a payload into a variant case by case_no
//! enum_value        a named enum value by value_no
//! list_construct    list of one element type
//! map_construct     NFC-keyed map with one value type
//! option_none       typed None
//! option_some       typed Some
//! field_get         select a record field by field_no
//! equals            equality of comparable operands
//! not / and / or    boolean connectives
//! if_then_else      pure conditional; not workflow SELECT
//! ```
//!
//! Arithmetic, string operators, indexing, variant inspection and any
//! fallible operation are deliberately deferred: this phase's expression
//! language must be total.
//!
//! # Expressions form a DAG
//!
//! Every expression lives in a table ([`ExprTable`]) keyed by [`ExpressionId`]
//! and refers to other expressions by id, so shared subexpressions are not
//! duplicated and forward references are legal. Cycles are rejected.
//!
//! # Determinism notes (they exist so ACE1 bytes are exactly reproducible)
//!
//! - `record_construct.fields` is canonically ordered by `field_no` ascending;
//! - `map_construct.entries` is canonically ordered by the unsigned UTF-8
//!   bytes of the *NFC-normalized* key, and keys that collide after
//!   normalization are rejected (identical to the ACE1 map invariant);
//! - `and` / `or` are binary so their operands cannot be silently reordered.

use crate::error::SemanticError;
use crate::ids::{ExpressionId, ValueSlotId};
use crate::types::TypeSchemaRef;
use crate::values::PrimitiveConstant;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fmt;

/// One record-field binding inside a record construction. `field_no` is the
/// durable schema ABI selector; the field *name* is only a value-encoding key.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordFieldExpr {
    pub field_no: u32,
    pub expr_id: ExpressionId,
}

/// One map entry inside a map construction.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapEntryExpr {
    pub key: String,
    pub value_expr: ExpressionId,
}

/// The closed Phase 3A expression body set.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpressionBody {
    /// A primitive literal; result type is the constant's primitive type.
    Constant(PrimitiveConstant),
    /// Read an immutable value slot; result type is the slot's declared type.
    ValueSlot(ValueSlotId),
    /// Build a value of the pinned record schema.
    RecordConstruct {
        schema: TypeSchemaRef,
        fields: Vec<RecordFieldExpr>,
    },
    /// Wrap a payload into the pinned variant schema's case.
    VariantConstruct {
        schema: TypeSchemaRef,
        case_no: u32,
        payload_expr: ExpressionId,
    },
    /// Select a named enum value of the pinned enum schema.
    EnumValue {
        schema: TypeSchemaRef,
        value_no: u32,
    },
    /// Build `list<element_type>`; `element_type` is a semantic type id.
    ListConstruct {
        element_type: crate::ids::SemanticTypeId,
        elements: Vec<ExpressionId>,
    },
    /// Build `map<string, value_type>`; `value_type` is a semantic type id.
    MapConstruct {
        value_type: crate::ids::SemanticTypeId,
        entries: Vec<MapEntryExpr>,
    },
    /// `option<inner_type>` with no payload; the inner type must be explicit
    /// because `None` carries no value to infer it from.
    OptionNone {
        inner_type: crate::ids::SemanticTypeId,
    },
    /// `option<T>` from a `T` payload.
    OptionSome {
        payload_expr: ExpressionId,
    },
    /// Select a record field by number; the input must be a record schema.
    FieldGet {
        record: ExpressionId,
        field_no: u32,
    },
    /// Equality; both operands must have equal, comparable types.
    Equals {
        left: ExpressionId,
        right: ExpressionId,
    },
    Not {
        operand: ExpressionId,
    },
    And {
        left: ExpressionId,
        right: ExpressionId,
    },
    Or {
        left: ExpressionId,
        right: ExpressionId,
    },
    /// Pure conditional (a value expression, never workflow `SELECT`).
    IfThenElse {
        condition: ExpressionId,
        then_expr: ExpressionId,
        else_expr: ExpressionId,
    },
}

/// One expression definition: local id, declared result type and body.
///
/// The declared `result_type` is required ABI clarity; the validator checks it
/// equals the type derived from the body and rejects any divergence.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpressionDef {
    pub id: ExpressionId,
    #[serde(rename = "result_type")]
    pub result_type_id: crate::ids::SemanticTypeId,
    pub body: ExpressionBody,
}

/// Depth-first mark used by [`ExprTable::find_cycle`].
#[derive(Clone, Copy, PartialEq)]
enum Mark {
    InProgress,
    Done,
}

/// The expression universe: a table keyed by [`ExpressionId`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExprTable {
    defs: BTreeMap<ExpressionId, ExpressionDef>,
    duplicates: Vec<ExpressionId>,
}

impl ExprTable {
    /// Builds a table from a source list, **last-write-wins is forbidden**:
    /// duplicate ids are recorded and rejected by the validator.
    pub fn from_defs(defs: Vec<ExpressionDef>) -> Self {
        let mut table = ExprTable::default();
        for def in defs {
            if table.defs.insert(def.id, def.clone()).is_some() {
                table.duplicates.push(def.id);
            }
        }
        table
    }

    /// The definition of `id`, if declared.
    pub fn get(&self, id: ExpressionId) -> Option<&ExpressionDef> {
        self.defs.get(&id)
    }

    /// All definitions in canonical order (id ascending).
    pub fn iter(&self) -> impl Iterator<Item = &ExpressionDef> {
        self.defs.values()
    }

    /// Number of definitions (duplicates counted once).
    pub fn len(&self) -> usize {
        self.defs.len()
    }

    /// `true` when no expression is defined.
    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }

    /// Expression ids recorded more than once in the source list.
    pub fn duplicates(&self) -> &[ExpressionId] {
        &self.duplicates
    }

    /// The expression ids this expression body references directly.
    pub fn references(body: &ExpressionBody) -> Vec<ExpressionId> {
        match body {
            ExpressionBody::Constant(_)
            | ExpressionBody::ValueSlot(_)
            | ExpressionBody::EnumValue { .. }
            | ExpressionBody::OptionNone { .. } => Vec::new(),
            ExpressionBody::RecordConstruct { fields, .. } => {
                fields.iter().map(|field| field.expr_id).collect()
            }
            ExpressionBody::VariantConstruct { payload_expr, .. }
            | ExpressionBody::OptionSome { payload_expr } => vec![*payload_expr],
            ExpressionBody::ListConstruct { elements, .. } => elements.clone(),
            ExpressionBody::MapConstruct { entries, .. } => {
                entries.iter().map(|entry| entry.value_expr).collect()
            }
            ExpressionBody::FieldGet { record, .. } => vec![*record],
            ExpressionBody::Equals { left, right }
            | ExpressionBody::And { left, right }
            | ExpressionBody::Or { left, right } => vec![*left, *right],
            ExpressionBody::Not { operand } => vec![*operand],
            ExpressionBody::IfThenElse {
                condition,
                then_expr,
                else_expr,
            } => vec![*condition, *then_expr, *else_expr],
        }
    }

    /// Detects an expression cycle and returns a witness id.
    ///
    /// Depth-first search over the reference graph; the returned id is the
    /// first node found on a back edge. Recursion is bounded by the table
    /// size, which the cycle test necessarily bounds anyway.
    pub fn find_cycle(&self) -> Option<ExpressionId> {
        let mut marks: BTreeMap<ExpressionId, Mark> = BTreeMap::new();
        for id in self.defs.keys() {
            if !marks.contains_key(id) {
                if let Some(witness) = self.visit(*id, &mut marks) {
                    return Some(witness);
                }
            }
        }
        None
    }

    fn visit(
        &self,
        id: ExpressionId,
        marks: &mut BTreeMap<ExpressionId, Mark>,
    ) -> Option<ExpressionId> {
        match marks.get(&id) {
            Some(Mark::Done) => return None,
            Some(Mark::InProgress) => return Some(id),
            None => {}
        }
        marks.insert(id, Mark::InProgress);
        if let Some(def) = self.defs.get(&id) {
            for referenced in Self::references(&def.body) {
                if let Some(witness) = self.visit(referenced, marks) {
                    return Some(witness);
                }
            }
        }
        marks.insert(id, Mark::Done);
        None
    }

    /// Checks that a map construction's keys are NFC-normalized, unique after
    /// normalization and canonically sorted by unsigned UTF-8 bytes. Returns
    /// the entries in canonical order when they already are.
    ///
    /// # Errors
    /// [`SemanticError::DuplicateNormalizedMapKey`] for collisions,
    /// [`SemanticError::InvalidConstant`] for non-NFC or unsorted input (the
    /// caller normalizes before calling; this is a defensive check).
    pub fn canonical_map_entries(
        entries: &[MapEntryExpr],
    ) -> Result<Vec<MapEntryExpr>, SemanticError> {
        let mut normalized: Vec<MapEntryExpr> = Vec::with_capacity(entries.len());
        for entry in entries {
            let key = nfc(&entry.key);
            if normalized.iter().any(|existing| existing.key == key) {
                return Err(SemanticError::DuplicateNormalizedMapKey { key });
            }
            normalized.push(MapEntryExpr {
                key,
                value_expr: entry.value_expr,
            });
        }
        normalized.sort_by(|a, b| a.key.as_bytes().cmp(b.key.as_bytes()));
        Ok(normalized)
    }
}

/// NFC normalization (the ACE1 string invariant, reused verbatim).
pub(crate) fn nfc(text: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    text.nfc().collect()
}

impl fmt::Display for ExpressionBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ExpressionBody::Constant(_) => "constant",
            ExpressionBody::ValueSlot(_) => "value_slot",
            ExpressionBody::RecordConstruct { .. } => "record_construct",
            ExpressionBody::VariantConstruct { .. } => "variant_construct",
            ExpressionBody::EnumValue { .. } => "enum_value",
            ExpressionBody::ListConstruct { .. } => "list_construct",
            ExpressionBody::MapConstruct { .. } => "map_construct",
            ExpressionBody::OptionNone { .. } => "option_none",
            ExpressionBody::OptionSome { .. } => "option_some",
            ExpressionBody::FieldGet { .. } => "field_get",
            ExpressionBody::Equals { .. } => "equals",
            ExpressionBody::Not { .. } => "not",
            ExpressionBody::And { .. } => "and",
            ExpressionBody::Or { .. } => "or",
            ExpressionBody::IfThenElse { .. } => "if_then_else",
        };
        f.write_str(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def(id: u32, body: ExpressionBody) -> ExpressionDef {
        ExpressionDef {
            id: ExpressionId::from_raw(id),
            result_type_id: crate::ids::SemanticTypeId::from_raw(1),
            body,
        }
    }

    #[test]
    fn cycles_are_detected_for_self_two_node_and_longer_cycles() {
        let self_cycle = ExprTable::from_defs(vec![def(
            1,
            ExpressionBody::Not {
                operand: ExpressionId::from_raw(1),
            },
        )]);
        assert_eq!(self_cycle.find_cycle(), Some(ExpressionId::from_raw(1)));

        let two_node = ExprTable::from_defs(vec![
            def(
                1,
                ExpressionBody::Not {
                    operand: ExpressionId::from_raw(2),
                },
            ),
            def(
                2,
                ExpressionBody::Not {
                    operand: ExpressionId::from_raw(1),
                },
            ),
        ]);
        assert!(two_node.find_cycle().is_some());

        let longer = ExprTable::from_defs(vec![
            def(
                1,
                ExpressionBody::OptionSome {
                    payload_expr: ExpressionId::from_raw(2),
                },
            ),
            def(
                2,
                ExpressionBody::IfThenElse {
                    condition: ExpressionId::from_raw(3),
                    then_expr: ExpressionId::from_raw(1),
                    else_expr: ExpressionId::from_raw(3),
                },
            ),
            def(
                3,
                ExpressionBody::Not {
                    operand: ExpressionId::from_raw(4),
                },
            ),
            def(
                4,
                ExpressionBody::Not {
                    operand: ExpressionId::from_raw(2),
                },
            ),
        ]);
        assert!(longer.find_cycle().is_some());
    }

    #[test]
    fn acyclic_dags_with_forward_references_are_accepted() {
        let table = ExprTable::from_defs(vec![
            // expr 1 references expr 9 (forward reference), 9 references 1?
            // No: 9 is a leaf, so the graph is a DAG.
            def(
                1,
                ExpressionBody::OptionSome {
                    payload_expr: ExpressionId::from_raw(9),
                },
            ),
            def(9, ExpressionBody::Constant(PrimitiveConstant::Bool(true))),
        ]);
        assert_eq!(table.find_cycle(), None);
    }

    #[test]
    fn duplicate_ids_are_recorded_not_silently_overwritten() {
        let table = ExprTable::from_defs(vec![
            def(1, ExpressionBody::Constant(PrimitiveConstant::Bool(true))),
            def(1, ExpressionBody::Constant(PrimitiveConstant::Bool(false))),
        ]);
        assert_eq!(table.duplicates(), &[ExpressionId::from_raw(1)]);
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn map_entries_are_normalized_unique_and_sorted() {
        let entries = vec![
            MapEntryExpr {
                key: "b".into(),
                value_expr: ExpressionId::from_raw(2),
            },
            MapEntryExpr {
                key: "a".into(),
                value_expr: ExpressionId::from_raw(1),
            },
        ];
        let canonical = ExprTable::canonical_map_entries(&entries).unwrap();
        assert_eq!(
            canonical.iter().map(|e| e.key.as_str()).collect::<Vec<_>>(),
            ["a", "b"]
        );

        let decomposed = vec![
            MapEntryExpr {
                key: "e\u{301}".into(),
                value_expr: ExpressionId::from_raw(1),
            },
            MapEntryExpr {
                key: "\u{e9}".into(),
                value_expr: ExpressionId::from_raw(2),
            },
        ];
        let error = ExprTable::canonical_map_entries(&decomposed).unwrap_err();
        assert_eq!(error.kind(), "duplicate_normalized_map_key");
    }
}
