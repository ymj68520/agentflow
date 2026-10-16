//! Structural validation, canonical ordering and the canonical ACE1 bytes.
//!
//! Canonicalization is a pure function of the declaration: entries are
//! ordered by their exact atom key, scope bindings by (key, value), qualifiers
//! by family/member/payload, and residuals by (catalog, port, scope). Inputs
//! that differ only in ordering therefore have identical canonical bytes.
//!
//! Two semantic normalizations are part of the canonical form:
//!
//! - an entry whose occurrence bounds cannot occur (zero upper) is dropped —
//!   an effect that occurs zero times is no effect;
//! - an open row whose known list and residual list both end up empty is
//!   normalized to the closed empty row (∅): a bounded-open row that bounds
//!   nothing IS pure.
//!
//! The canonical form has ACE1 bytes (lowered against
//! `agentflow.effect.signature@1/EffectSignature`) but deliberately NO
//! digest: the HashV1 domain registry is untouched, and the future
//! ComponentContract will hash as a whole under the frozen
//! `component-contract` domain.

use crate::catalog::is_nfc;
use crate::error::{EffectError, kind};
use crate::model::{
    AtomKey, EffectAtom, EffectEntry, EffectGuard, EffectIdempotencyQualifier, EffectOpenRow,
    EffectQualifier, EffectQualifierIdentity, EffectResidual, EffectRow, EffectSignature,
    OccurrenceBounds,
};
use af_core::Value;
use af_semantic::PrimitiveConstant;
use std::collections::BTreeSet;

impl EffectSignature {
    /// The canonical form: fully structure-checked, canonically ordered,
    /// zero-occurrence entries dropped, degenerate open rows normalized.
    ///
    /// # Errors
    /// Returns the first structural [`EffectError`] (all kinds except
    /// `unknown_effect_operation`, which requires a catalog universe and is
    /// raised by [`EffectSignature::validate`]).
    pub fn canonical(&self) -> Result<EffectSignature, EffectError> {
        Ok(EffectSignature {
            row: canonical_row(&self.row)?,
        })
    }

    /// Validates structure AND resolves every operation and residual catalog
    /// against the explicit catalog universe, returning the canonical form.
    ///
    /// # Errors
    /// Structural [`EffectError`]s from [`EffectSignature::canonical`], then
    /// `unknown_effect_operation` for the first unresolved operation (in
    /// canonical entry order) or residual catalog.
    pub fn validate(
        &self,
        universe: &crate::catalog::EffectCatalogUniverse,
    ) -> Result<EffectSignature, EffectError> {
        let canonical = self.canonical()?;
        resolve_row(&canonical.row, universe)?;
        Ok(canonical)
    }

    /// `true` exactly for the closed empty row — the ONE representation of
    /// pure. Unknown is not pure; an open row is not pure.
    #[must_use]
    pub fn is_pure(&self) -> bool {
        matches!(&self.row, EffectRow::Closed(entries) if entries.is_empty())
    }

    /// The stable row classification: `pure`, `closed`, `open` or `unknown`.
    #[must_use]
    pub fn classification(&self) -> &'static str {
        match &self.row {
            EffectRow::Closed(entries) if entries.is_empty() => "pure",
            EffectRow::Closed(_) => "closed",
            EffectRow::Open(_) => "open",
            EffectRow::Unknown => "unknown",
        }
    }

    /// The canonical ACE1 bytes, encoded against the schema-derived shape of
    /// `agentflow.effect.signature@1/EffectSignature` within `set` (which
    /// must contain the effect-signature module and the semantic core it
    /// imports). Requires a structurally valid signature.
    ///
    /// # Errors
    /// [`EffectError`] with kind `effect_encoding_error` when the canonical
    /// form is invalid or the value tree cannot be encoded.
    pub fn canonical_bytes(&self, set: &af_idl::CompiledModuleSet) -> Result<Vec<u8>, EffectError> {
        let canonical = self.canonical()?;
        let shape = set
            .shape_of("agentflow.effect.signature@1/EffectSignature")
            .map_err(|error| {
                EffectError::new(
                    kind::EFFECT_ENCODING_ERROR,
                    format!("cannot derive the signature shape: {error}"),
                )
            })?;
        shape.validate().map_err(|error| {
            EffectError::new(
                kind::EFFECT_ENCODING_ERROR,
                format!("derived shape is invalid: {error}"),
            )
        })?;
        let value = canonical.to_value()?;
        af_core::encode(&shape, &value).map_err(|error| {
            EffectError::new(
                kind::EFFECT_ENCODING_ERROR,
                format!("cannot encode the canonical signature: {error}"),
            )
        })
    }

    /// The ACE1 value tree of the canonical signature (mirrors the AFIDL
    /// schema field-for-field; the encoder emits members in byte order).
    ///
    /// # Errors
    /// [`EffectError`] when the signature is not structurally valid.
    pub fn to_value(&self) -> Result<Value, EffectError> {
        Ok(Value::Record(vec![(
            "row".into(),
            row_value(&self.canonical()?.row)?,
        )]))
    }
}

// ---------------------------------------------------------------------------
// Canonical ordering and normalization
// ---------------------------------------------------------------------------

fn canonical_row(row: &EffectRow) -> Result<EffectRow, EffectError> {
    match row {
        EffectRow::Closed(entries) => Ok(EffectRow::Closed(canonical_entries(entries)?)),
        EffectRow::Open(open) => {
            let known = canonical_entries(&open.known)?;
            let residual = canonical_residuals(&open.residual)?;
            if known.is_empty() && residual.is_empty() {
                // A bounded-open row that bounds nothing IS pure.
                Ok(EffectRow::Closed(Vec::new()))
            } else {
                Ok(EffectRow::Open(EffectOpenRow { known, residual }))
            }
        }
        EffectRow::Unknown => Ok(EffectRow::Unknown),
    }
}

fn canonical_entries(entries: &[EffectEntry]) -> Result<Vec<EffectEntry>, EffectError> {
    let mut keyed: Vec<(AtomKey, EffectEntry)> = Vec::with_capacity(entries.len());
    let mut seen = BTreeSet::new();
    for entry in entries {
        check_atom(&entry.atom)?;
        check_bounds(&entry.bounds)?;
        check_guard(&entry.guard)?;
        let key = AtomKey::of(&entry.atom);
        if !seen.insert(key.clone()) {
            return Err(EffectError::new(
                kind::DUPLICATE_EFFECT_ATOM,
                format!(
                    "effect atom {}/{} on port {} is declared twice",
                    key.catalog(),
                    key.name(),
                    key.port_no()
                ),
            ));
        }
        keyed.push((key, canonical_entry(entry)));
    }
    keyed.sort_by(|a, b| a.0.cmp(&b.0));
    // An effect that occurs zero times is no effect: drop it.
    Ok(keyed
        .into_iter()
        .filter(|(_, entry)| !entry.bounds.cannot_occur())
        .map(|(_, entry)| entry)
        .collect())
}

/// Canonical atom internals: scope bindings sorted by (key, value) and
/// qualifiers by family, member number and typed payload — binding or
/// qualifier declaration order is not semantic identity, so it must not
/// reach the canonical bytes.
fn canonical_entry(entry: &EffectEntry) -> EffectEntry {
    let mut atom = entry.atom.clone();
    atom.resource_selector
        .scope_bindings
        .sort_by(|a, b| (&a.key, &a.value).cmp(&(&b.key, &b.value)));
    atom.qualifiers
        .sort_by_key(|qualifier| EffectQualifierIdentity::of(*qualifier));
    EffectEntry {
        atom,
        bounds: entry.bounds.clone(),
        guard: entry.guard.clone(),
    }
}

/// The canonical residual key: (catalog, port, scope bindings).
fn residual_key(residual: &EffectResidual) -> (String, u32, Vec<(String, String)>) {
    let mut scope: Vec<(String, String)> = residual
        .selector
        .scope_bindings
        .iter()
        .map(|binding| (binding.key.clone(), binding.value.clone()))
        .collect();
    scope.sort();
    (
        residual.catalog.clone(),
        residual.selector.requirement_port_no,
        scope,
    )
}

fn canonical_residuals(residuals: &[EffectResidual]) -> Result<Vec<EffectResidual>, EffectError> {
    type ResidualSortKey = (String, u32, Vec<(String, String)>);
    let mut keyed: Vec<(ResidualSortKey, EffectResidual)> = Vec::with_capacity(residuals.len());
    for residual in residuals {
        crate::catalog::CapabilityCatalog::check_catalog_id(&residual.catalog)?;
        check_selector(&residual.selector)?;
        check_bounds(&residual.bounds)?;
        // The ceiling must constrain something: a scope binding or a finite
        // occurrence upper. A residual that proves nothing is rejected.
        if residual.selector.scope_bindings.is_empty() && residual.bounds.upper.is_unbounded() {
            return Err(EffectError::new(
                kind::UNBOUNDED_OPEN_EFFECT_ROW,
                format!(
                    "residual catalog {} on port {} has no scope binding and an \
                     unbounded occurrence upper: its ceiling proves nothing",
                    residual.catalog, residual.selector.requirement_port_no
                ),
            ));
        }
        keyed.push((residual_key(residual), canonical_residual(residual)));
    }
    keyed.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(keyed
        .into_iter()
        .filter(|(_, residual)| !residual.bounds.cannot_occur())
        .map(|(_, residual)| residual)
        .collect())
}

/// Canonical residual internals: the selector's scope bindings sorted.
fn canonical_residual(residual: &EffectResidual) -> EffectResidual {
    let mut selector = residual.selector.clone();
    selector
        .scope_bindings
        .sort_by(|a, b| (&a.key, &a.value).cmp(&(&b.key, &b.value)));
    EffectResidual {
        catalog: residual.catalog.clone(),
        selector,
        bounds: residual.bounds.clone(),
    }
}

// ---------------------------------------------------------------------------
// Structural checks (shared by canonicalize and validate)
// ---------------------------------------------------------------------------

fn check_atom(atom: &EffectAtom) -> Result<(), EffectError> {
    crate::catalog::CapabilityCatalog::check_catalog_id(&atom.operation.catalog)?;
    crate::catalog::CapabilityCatalog::check_operation_name(&atom.operation.name)?;
    check_selector(&atom.resource_selector)?;
    check_qualifiers(&atom.qualifiers)?;
    Ok(())
}

fn check_selector(selector: &crate::model::ResourceSelector) -> Result<(), EffectError> {
    if selector.requirement_port_no == 0 {
        return Err(EffectError::new(
            kind::INVALID_RESOURCE_SELECTOR,
            "requirement port 0 is invalid: ports are 1-based",
        ));
    }
    let mut seen = BTreeSet::new();
    for binding in &selector.scope_bindings {
        if binding.key.is_empty() || binding.value.is_empty() {
            return Err(EffectError::new(
                kind::INVALID_RESOURCE_SELECTOR,
                format!(
                    "scope binding key {:?} / value {:?} must both be non-empty",
                    binding.key, binding.value
                ),
            ));
        }
        if !is_nfc(&binding.key) || !is_nfc(&binding.value) {
            return Err(EffectError::new(
                kind::INVALID_RESOURCE_SELECTOR,
                format!(
                    "scope binding {:?}={:?} is not NFC-normalized",
                    binding.key, binding.value
                ),
            ));
        }
        if !seen.insert(binding.key.clone()) {
            return Err(EffectError::new(
                kind::INVALID_RESOURCE_SELECTOR,
                format!(
                    "scope key {:?} is bound twice: at most one exact binding per key",
                    binding.key
                ),
            ));
        }
    }
    Ok(())
}

fn check_qualifiers(qualifiers: &[EffectQualifier]) -> Result<(), EffectError> {
    let mut families = BTreeSet::new();
    for qualifier in qualifiers {
        let family = qualifier.family_no();
        if !families.insert(family) {
            return Err(EffectError::new(
                kind::INVALID_EFFECT_QUALIFIER,
                format!("qualifier family {family} appears twice"),
            ));
        }
    }
    // Within one family at most one member is present (enforced above); the
    // idempotency family has exactly two members, so a single list can never
    // hold both `natural` and `none`.
    Ok(())
}

fn check_bounds(bounds: &OccurrenceBounds) -> Result<(), EffectError> {
    bounds.validate()
}

fn check_guard(guard: &EffectGuard) -> Result<(), EffectError> {
    match guard {
        EffectGuard::Unconditional => Ok(()),
        EffectGuard::InputEquals(input) => {
            if input.port_no == 0 {
                return Err(EffectError::new(
                    kind::INVALID_EFFECT_GUARD,
                    "guard input port 0 is invalid: ports are 1-based",
                ));
            }
            input.expected.validate().map_err(|error| {
                EffectError::new(
                    kind::INVALID_EFFECT_GUARD,
                    format!("guard expected constant is not canonical: {error}"),
                )
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Catalog resolution (canonical order, fail closed)
// ---------------------------------------------------------------------------

fn resolve_row(
    row: &EffectRow,
    universe: &crate::catalog::EffectCatalogUniverse,
) -> Result<(), EffectError> {
    let mut entries: Vec<&EffectEntry> = Vec::new();
    let mut residuals: Vec<&EffectResidual> = Vec::new();
    match row {
        EffectRow::Closed(closed) => entries.extend(closed.iter()),
        EffectRow::Open(open) => {
            entries.extend(open.known.iter());
            residuals.extend(open.residual.iter());
        }
        EffectRow::Unknown => {}
    }
    for entry in entries {
        if !universe.resolves(&entry.atom.operation.catalog, &entry.atom.operation.name) {
            return Err(EffectError::new(
                kind::UNKNOWN_EFFECT_OPERATION,
                format!(
                    "operation {}/{} does not resolve in the supplied catalog universe",
                    entry.atom.operation.catalog, entry.atom.operation.name
                ),
            ));
        }
    }
    for residual in residuals {
        if !universe.catalog_ids().any(|id| id == residual.catalog) {
            return Err(EffectError::new(
                kind::UNKNOWN_EFFECT_OPERATION,
                format!(
                    "residual catalog {} is not registered in the supplied universe",
                    residual.catalog
                ),
            ));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// ACE1 value tree
// ---------------------------------------------------------------------------

fn row_value(row: &EffectRow) -> Result<Value, EffectError> {
    Ok(match row {
        EffectRow::Closed(entries) => Value::Variant {
            case: "closed".into(),
            value: Box::new(Value::List(
                entries.iter().map(entry_value).collect::<Result<_, _>>()?,
            )),
        },
        EffectRow::Open(open) => Value::Variant {
            case: "open".into(),
            value: Box::new(Value::Record(vec![
                (
                    "known".into(),
                    Value::List(
                        open.known
                            .iter()
                            .map(entry_value)
                            .collect::<Result<_, _>>()?,
                    ),
                ),
                (
                    "residual".into(),
                    Value::List(open.residual.iter().map(residual_value).collect()),
                ),
            ])),
        },
        EffectRow::Unknown => Value::Variant {
            case: "unknown".into(),
            value: Box::new(Value::Unit),
        },
    })
}

fn entry_value(entry: &EffectEntry) -> Result<Value, EffectError> {
    Ok(Value::Record(vec![
        ("atom".into(), atom_value(&entry.atom)),
        ("bounds".into(), bounds_value(&entry.bounds)),
        ("guard".into(), guard_value(&entry.guard)?),
    ]))
}

fn residual_value(residual: &EffectResidual) -> Value {
    Value::Record(vec![
        ("catalog".into(), Value::String(residual.catalog.clone())),
        ("selector".into(), selector_value(&residual.selector)),
        ("bounds".into(), bounds_value(&residual.bounds)),
    ])
}

fn atom_value(atom: &EffectAtom) -> Value {
    Value::Record(vec![
        (
            "operation".into(),
            Value::Record(vec![
                (
                    "catalog".into(),
                    Value::String(atom.operation.catalog.clone()),
                ),
                ("name".into(), Value::String(atom.operation.name.clone())),
            ]),
        ),
        (
            "resource_selector".into(),
            selector_value(&atom.resource_selector),
        ),
        (
            "qualifiers".into(),
            Value::List(
                atom.qualifiers
                    .iter()
                    .map(|qualifier| {
                        let member = match qualifier {
                            EffectQualifier::Idempotency(EffectIdempotencyQualifier::Natural) => {
                                "natural"
                            }
                            EffectQualifier::Idempotency(EffectIdempotencyQualifier::None) => {
                                "none"
                            }
                        };
                        Value::Variant {
                            case: "idempotency".into(),
                            value: Box::new(Value::Enum(member.into())),
                        }
                    })
                    .collect(),
            ),
        ),
    ])
}

fn selector_value(selector: &crate::model::ResourceSelector) -> Value {
    Value::Record(vec![
        (
            "requirement_port_no".into(),
            Value::UInt32(selector.requirement_port_no),
        ),
        (
            "scope_bindings".into(),
            Value::List(
                selector
                    .scope_bindings
                    .iter()
                    .map(|binding| {
                        Value::Record(vec![
                            ("key".into(), Value::String(binding.key.clone())),
                            ("value".into(), Value::String(binding.value.clone())),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}

fn bounds_value(bounds: &OccurrenceBounds) -> Value {
    Value::Record(vec![
        ("lower".into(), Value::UInt64(bounds.lower)),
        (
            "upper".into(),
            match bounds.upper {
                crate::model::OccurrenceCeiling::Finite(value) => Value::Variant {
                    case: "finite".into(),
                    value: Box::new(Value::UInt64(value)),
                },
                crate::model::OccurrenceCeiling::Unbounded => Value::Variant {
                    case: "unbounded".into(),
                    value: Box::new(Value::Unit),
                },
            },
        ),
    ])
}

fn guard_value(guard: &EffectGuard) -> Result<Value, EffectError> {
    Ok(match guard {
        EffectGuard::Unconditional => Value::Variant {
            case: "unconditional".into(),
            value: Box::new(Value::Unit),
        },
        EffectGuard::InputEquals(input) => Value::Variant {
            case: "input_equals".into(),
            value: Box::new(Value::Record(vec![
                ("port_no".into(), Value::UInt32(input.port_no)),
                ("expected".into(), constant_value(&input.expected)?),
            ])),
        },
    })
}

fn constant_value(constant: &PrimitiveConstant) -> Result<Value, EffectError> {
    let (case, payload) = match constant {
        PrimitiveConstant::Unit => ("unit", Value::Unit),
        PrimitiveConstant::Null => ("null", Value::Null),
        PrimitiveConstant::Bool(value) => ("bool", Value::Bool(*value)),
        PrimitiveConstant::Int64(value) => ("int64", Value::Int64(*value)),
        PrimitiveConstant::Decimal(value) => ("decimal", Value::Decimal(value.clone())),
        PrimitiveConstant::String(value) => ("string", Value::String(value.clone())),
        // The source String is canonical base64url, NOT the raw byte payload.
        // Keep strict decode failures fallible; never substitute empty/String.
        PrimitiveConstant::Bytes(value) => (
            "bytes",
            Value::Bytes(af_core::base64url::decode(value).map_err(|error| {
                EffectError::new(
                    kind::INVALID_EFFECT_GUARD,
                    format!("guard bytes constant is not canonical: {error}"),
                )
            })?),
        ),
        PrimitiveConstant::Timestamp(value) => ("timestamp", Value::Timestamp(*value)),
        PrimitiveConstant::Duration(value) => ("duration", Value::Duration(*value)),
    };
    Ok(Value::Variant {
        case: case.into(),
        value: Box::new(payload),
    })
}
