//! Stable ABI number registry ↔ current AFIDL schema consistency checks.
//!
//! This is the machine enforcement of the "number never reused" principle
//! (spec/registry/registry-spec.md §5): the AFIDL validator only sees the
//! current compilation unit; this module compares it against the historical
//! allocations in `spec/registry/abi-numbers.json`.

use crate::error::SpecError;
use crate::model::AbiAssignmentJson;
use af_idl::{CanonicalBody, CompiledSchema};
use std::collections::{BTreeMap, BTreeSet};

/// One numbered member of a compiled schema: a record field, a variant case
/// or an enum value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaBinding {
    pub type_name: String,
    /// `"field" | "variant_case" | "enum_value"` (registry JSON spelling).
    pub number_kind: &'static str,
    pub number: u32,
    pub symbolic_name: String,
}

/// Extracts every numbered member of a compiled schema, in canonical order
/// (types by name, members by number — the canonicalizer guarantees both).
pub fn schema_bindings(compiled: &CompiledSchema) -> Vec<SchemaBinding> {
    let mut out = Vec::new();
    for schema_type in &compiled.types {
        match &schema_type.body {
            CanonicalBody::Alias(_) => {}
            CanonicalBody::Enum(values) => {
                for value in values {
                    out.push(SchemaBinding {
                        type_name: schema_type.name.clone(),
                        number_kind: "enum_value",
                        number: value.no,
                        symbolic_name: value.name.clone(),
                    });
                }
            }
            CanonicalBody::Record(fields) => {
                for field in fields {
                    out.push(SchemaBinding {
                        type_name: schema_type.name.clone(),
                        number_kind: "field",
                        number: field.no,
                        symbolic_name: field.name.clone(),
                    });
                }
            }
            CanonicalBody::Variant(cases) => {
                for case in cases {
                    out.push(SchemaBinding {
                        type_name: schema_type.name.clone(),
                        number_kind: "variant_case",
                        number: case.no,
                        symbolic_name: case.name.clone(),
                    });
                }
            }
        }
    }
    out
}

type TupleKey = (String, String, u32);

fn tuple_key(row: &AbiAssignmentJson) -> TupleKey {
    (row.type_name.clone(), row.number_kind.clone(), row.number)
}

fn describe(family: &str, binding: &SchemaBinding) -> String {
    format!(
        "{family}/{} {} {} = {:?}",
        binding.type_name, binding.number_kind, binding.number, binding.symbolic_name
    )
}

/// Compiles the registry rows for one schema family against that family's
/// current compiled schema.
///
/// Checks (spec/registry/registry-spec.md §5):
///
/// - every current binding has an `active` row with the identical symbolic
///   name; retired/reserved numbers in use and any number/name divergence
///   are rejected;
/// - a current binding whose symbolic name is registered at a *different*
///   number is a rejected name rebound;
/// - every `active` row must correspond to a current binding; `reserved`
///   rows are exempt.
pub fn check_current_schema_against_abi_registry(
    family: &str,
    bindings: &[SchemaBinding],
    assignments: &[AbiAssignmentJson],
) -> Vec<SpecError> {
    let mut errors = Vec::new();
    let family_rows: Vec<&AbiAssignmentJson> = assignments
        .iter()
        .filter(|row| row.schema_family == family)
        .collect();
    let by_tuple: BTreeMap<TupleKey, &AbiAssignmentJson> = family_rows
        .iter()
        .map(|row| (tuple_key(row), *row))
        .collect();
    let mut matched: BTreeSet<TupleKey> = BTreeSet::new();

    for binding in bindings {
        let key = (
            binding.type_name.clone(),
            binding.number_kind.to_string(),
            binding.number,
        );
        let Some(row) = by_tuple.get(&key) else {
            let rebound = family_rows.iter().any(|candidate| {
                candidate.type_name == binding.type_name
                    && candidate.number_kind == binding.number_kind
                    && candidate.symbolic_name == binding.symbolic_name
            });
            errors.push(if rebound {
                SpecError::AbiNameRebound {
                    detail: format!(
                        "{} is registered at a different number but the schema uses {}",
                        binding.symbolic_name,
                        describe(family, binding)
                    ),
                }
            } else {
                SpecError::AbiUnregisteredAssignment {
                    detail: describe(family, binding),
                }
            });
            continue;
        };
        match (
            row.status.as_str(),
            row.symbolic_name == binding.symbolic_name,
        ) {
            ("active", true) => {
                matched.insert(key);
            }
            ("active" | "retired", false) => {
                errors.push(SpecError::AbiNumberReassigned {
                    detail: format!(
                        "schema uses {}, registry binds that number to {:?}",
                        describe(family, binding),
                        row.symbolic_name
                    ),
                });
            }
            ("retired", true) => {
                errors.push(SpecError::AbiRetiredNumberInUse {
                    detail: describe(family, binding),
                });
            }
            ("reserved", _) => {
                errors.push(SpecError::AbiReservedNumberInUse {
                    detail: describe(family, binding),
                });
            }
            _ => {
                errors.push(SpecError::InvalidAbiAssignment {
                    detail: format!("unknown status {:?}", row.status),
                });
            }
        }
    }

    for row in &family_rows {
        if row.status != "active" || matched.contains(&tuple_key(row)) {
            continue;
        }
        errors.push(SpecError::AbiActiveAssignmentMissing {
            detail: format!(
                "{family}/{} {} {} = {:?} has no counterpart in the current schema",
                row.type_name, row.number_kind, row.number, row.symbolic_name
            ),
        });
    }
    errors
}
