//! The typed effect-signature model, mirroring
//! `spec/idl/effect-signature.afidl` field-for-field.
//!
//! The model is the declaration side only: an [`EffectSignature`] says what
//! effects a computation MAY cause. It is not a grant, not a receipt and not
//! an authorization — no type in this crate can be executed or exchanged for
//! capability. Occurrence bounds count LOGICAL effect occurrences; physical
//! task attempts appear only in the retry envelope's separate attempt bounds.

use crate::error::{EffectError, kind};
use af_semantic::PrimitiveConstant;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Effect atom
// ---------------------------------------------------------------------------

/// The operation half of an effect atom. `catalog` + `name` is the full
/// logical identity `<catalog>/<name>`; validation resolves it fail-closed
/// against an explicit catalog universe.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectOperationRef {
    pub catalog: String,
    pub name: String,
}

/// One exact logical scope constraint (`key` bound to exactly `value`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopeBinding {
    pub key: String,
    pub value: String,
}

/// The logical acting range of an effect: the requirement port it flows
/// through plus exact-binding scope narrowing. Never a raw resource handle.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceSelector {
    pub requirement_port_no: u32,
    pub scope_bindings: Vec<ScopeBinding>,
}

/// The closed idempotency qualifier family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectIdempotencyQualifier {
    Natural,
    None,
}

impl EffectIdempotencyQualifier {
    /// The AFIDL `value_no` of this member (frozen ABI).
    #[must_use]
    pub fn value_no(self) -> u32 {
        match self {
            EffectIdempotencyQualifier::Natural => 1,
            EffectIdempotencyQualifier::None => 2,
        }
    }
}

/// One qualifier, tagged by family. At most one qualifier per family may
/// appear in an atom's qualifier list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectQualifier {
    Idempotency(EffectIdempotencyQualifier),
}

impl EffectQualifier {
    /// The family case number (`case_no`, frozen ABI): idempotency is 1.
    #[must_use]
    pub fn family_no(self) -> u32 {
        match self {
            EffectQualifier::Idempotency(_) => 1,
        }
    }
}

/// One declared effect: operation + logical selector + closed qualifiers.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectAtom {
    pub operation: EffectOperationRef,
    pub resource_selector: ResourceSelector,
    pub qualifiers: Vec<EffectQualifier>,
}

// ---------------------------------------------------------------------------
// Occurrence bounds
// ---------------------------------------------------------------------------

/// Explicit occurrence ceiling. [`OccurrenceCeiling::Unbounded`] IS the
/// infinity case — a distinct declared state, never an overflow artifact and
/// never encoded as `UINT64_MAX`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OccurrenceCeiling {
    Finite(u64),
    Unbounded,
}

impl OccurrenceCeiling {
    /// `true` for the unbounded (infinite) ceiling.
    #[must_use]
    pub fn is_unbounded(self) -> bool {
        matches!(self, OccurrenceCeiling::Unbounded)
    }

    /// `true` for a finite ceiling of zero (the entry cannot occur).
    #[must_use]
    pub fn is_zero(self) -> bool {
        matches!(self, OccurrenceCeiling::Finite(0))
    }

    /// The greater of two ceilings (`Unbounded` is the greatest element).
    #[must_use]
    pub fn max(self, other: Self) -> Self {
        if self.is_unbounded() || other.is_unbounded() {
            OccurrenceCeiling::Unbounded
        } else {
            OccurrenceCeiling::Finite(self.finite().max(other.finite()))
        }
    }

    /// Whether this ceiling covers (is >=) another.
    #[must_use]
    pub fn covers(self, inner: Self) -> bool {
        match (self, inner) {
            (OccurrenceCeiling::Unbounded, _) => true,
            (OccurrenceCeiling::Finite(_), OccurrenceCeiling::Unbounded) => false,
            (OccurrenceCeiling::Finite(outer), OccurrenceCeiling::Finite(inner)) => outer >= inner,
        }
    }

    /// Checked addition; `Unbounded` absorbs, finite overflow is an error.
    ///
    /// # Errors
    /// [`EffectError`] with kind `effect_arithmetic_overflow` when two finite
    /// ceilings overflow `u64`.
    pub fn checked_add(self, other: Self) -> Result<Self, EffectError> {
        match (self, other) {
            (OccurrenceCeiling::Unbounded, _) | (_, OccurrenceCeiling::Unbounded) => {
                Ok(OccurrenceCeiling::Unbounded)
            }
            (OccurrenceCeiling::Finite(a), OccurrenceCeiling::Finite(b)) => a
                .checked_add(b)
                .map(OccurrenceCeiling::Finite)
                .ok_or_else(|| {
                    EffectError::new(
                        kind::EFFECT_ARITHMETIC_OVERFLOW,
                        format!("occurrence ceiling {a} + {b} overflows u64"),
                    )
                }),
        }
    }

    /// Checked multiplication; `0 × anything = 0`, `Unbounded × positive =
    /// Unbounded`, finite overflow is an error.
    ///
    /// # Errors
    /// [`EffectError`] with kind `effect_arithmetic_overflow` when two finite
    /// ceilings overflow `u64`.
    pub fn checked_mul(self, other: Self) -> Result<Self, EffectError> {
        match (self, other) {
            (a, b) if a.is_zero() || b.is_zero() => Ok(OccurrenceCeiling::Finite(0)),
            (OccurrenceCeiling::Unbounded, _) | (_, OccurrenceCeiling::Unbounded) => {
                Ok(OccurrenceCeiling::Unbounded)
            }
            (OccurrenceCeiling::Finite(a), OccurrenceCeiling::Finite(b)) => a
                .checked_mul(b)
                .map(OccurrenceCeiling::Finite)
                .ok_or_else(|| {
                    EffectError::new(
                        kind::EFFECT_ARITHMETIC_OVERFLOW,
                        format!("occurrence ceiling {a} * {b} overflows u64"),
                    )
                }),
        }
    }

    /// The finite value; panics only on `Unbounded` (internal use on
    /// pre-checked operands).
    fn finite(self) -> u64 {
        match self {
            OccurrenceCeiling::Finite(value) => value,
            OccurrenceCeiling::Unbounded => unreachable!("max() handles unbounded first"),
        }
    }
}

/// Bounds on LOGICAL effect occurrences of one atom: `lower >= 0` and
/// `upper >= lower` (validated), with an explicit infinity.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OccurrenceBounds {
    pub lower: u64,
    pub upper: OccurrenceCeiling,
}

impl OccurrenceBounds {
    /// Builds validated bounds.
    ///
    /// # Errors
    /// [`EffectError`] with kind `invalid_occurrence_bounds` when a finite
    /// upper is below the lower bound.
    pub fn new(lower: u64, upper: OccurrenceCeiling) -> Result<Self, EffectError> {
        let bounds = OccurrenceBounds { lower, upper };
        bounds.validate()?;
        Ok(bounds)
    }

    /// The interval `[lower, upper]`, both finite.
    ///
    /// # Errors
    /// [`EffectError`] with kind `invalid_occurrence_bounds` when the upper
    /// is below the lower.
    pub fn finite(lower: u64, upper: u64) -> Result<Self, EffectError> {
        Self::new(lower, OccurrenceCeiling::Finite(upper))
    }

    /// Checks `upper >= lower`.
    ///
    /// # Errors
    /// [`EffectError`] with kind `invalid_occurrence_bounds`.
    pub fn validate(&self) -> Result<(), EffectError> {
        if let OccurrenceCeiling::Finite(upper) = self.upper {
            if upper < self.lower {
                return Err(EffectError::new(
                    kind::INVALID_OCCURRENCE_BOUNDS,
                    format!(
                        "occurrence bounds [{}, {upper}] have upper < lower",
                        self.lower
                    ),
                ));
            }
        }
        Ok(())
    }

    /// `true` when the entry cannot occur at all (zero upper, hence zero
    /// lower). Such entries are semantically pure.
    #[must_use]
    pub fn cannot_occur(&self) -> bool {
        self.upper.is_zero()
    }

    /// Whether this interval covers another (`self.lower <= inner.lower` and
    /// `self.upper >= inner.upper`).
    #[must_use]
    pub fn covers(&self, inner: &Self) -> bool {
        self.lower <= inner.lower && self.upper.covers(inner.upper)
    }

    /// Checked interval addition (`[l1,u1] ⊕ [l2,u2] = [l1+l2, u1+u2]`).
    ///
    /// # Errors
    /// [`EffectError`] with kind `effect_arithmetic_overflow`.
    pub fn checked_add(&self, other: &Self) -> Result<Self, EffectError> {
        Ok(OccurrenceBounds {
            lower: self.lower.checked_add(other.lower).ok_or_else(|| {
                EffectError::new(
                    kind::EFFECT_ARITHMETIC_OVERFLOW,
                    format!(
                        "occurrence lower bounds {} + {} overflow u64",
                        self.lower, other.lower
                    ),
                )
            })?,
            upper: self.upper.checked_add(other.upper)?,
        })
    }

    /// Checked interval multiplication (`n ⊗ [l,u] = [n_l·l, n_u·u]`).
    ///
    /// # Errors
    /// [`EffectError`] with kind `effect_arithmetic_overflow`.
    pub fn checked_mul(&self, reps: &Self) -> Result<Self, EffectError> {
        Ok(OccurrenceBounds {
            lower: self.lower.checked_mul(reps.lower).ok_or_else(|| {
                EffectError::new(
                    kind::EFFECT_ARITHMETIC_OVERFLOW,
                    format!(
                        "occurrence lower bounds {} * {} overflow u64",
                        self.lower, reps.lower
                    ),
                )
            })?,
            upper: self.upper.checked_mul(reps.upper)?,
        })
    }

    /// The conservative branch-join interval: `[min(l1,l2), max(u1,u2)]`.
    #[must_use]
    pub fn join(&self, other: &Self) -> Self {
        OccurrenceBounds {
            lower: self.lower.min(other.lower),
            upper: self.upper.max(other.upper),
        }
    }
}

impl std::fmt::Display for OccurrenceBounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.upper {
            OccurrenceCeiling::Finite(upper) => write!(f, "[{}, {upper}]", self.lower),
            OccurrenceCeiling::Unbounded => write!(f, "[{}, unbounded)", self.lower),
        }
    }
}

// ---------------------------------------------------------------------------
// Guards
// ---------------------------------------------------------------------------

/// Payload of the typed input-equality guard: a component input port (>= 1)
/// and the exact primitive value the input must equal.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectGuardInputEquals {
    pub port_no: u32,
    pub expected: PrimitiveConstant,
}

/// Closed guard language: a guard narrows WHEN an effect may occur, never
/// the envelope's magnitude. No negation, no time, no randomness, no policy
/// lookup, no free-text predicate.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectGuard {
    Unconditional,
    InputEquals(EffectGuardInputEquals),
}

// ---------------------------------------------------------------------------
// Rows
// ---------------------------------------------------------------------------

/// One row entry: an atom, its occurrence bounds and its guard.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectEntry {
    pub atom: EffectAtom,
    pub bounds: OccurrenceBounds,
    pub guard: EffectGuard,
}

/// The residual ceiling of a bounded-open row: unknown operations, but only
/// from this catalog, only through this selector, only within these bounds.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectResidual {
    pub catalog: String,
    pub selector: ResourceSelector,
    pub bounds: OccurrenceBounds,
}

/// A bounded-open row: known effects plus an explicitly bounded residual.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectOpenRow {
    pub known: Vec<EffectEntry>,
    pub residual: Vec<EffectResidual>,
}

/// The capability row. The four states are distinct and never collapsed:
/// empty `Closed` is the one representation of pure (∅), non-empty `Closed`
/// is a known closed signature (ε), `Open` is a bounded-open row (ε | ρ),
/// and `Unknown` is ⊤ — an unprovable envelope, not pure, never authority.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectRow {
    Closed(Vec<EffectEntry>),
    Open(EffectOpenRow),
    Unknown,
}

/// The declared effect envelope of a computation: exactly one row.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectSignature {
    pub row: EffectRow,
}

// ---------------------------------------------------------------------------
// Canonical atom identity
// ---------------------------------------------------------------------------

/// The exact canonical identity of an effect atom: operation identity,
/// requirement port, the sorted exact scope-binding set and the sorted
/// qualifier-family list. Two entries share an atom key exactly when they
/// declare the same effect on the same range with the same qualifiers; this
/// is the merge key of the algebra and the duplicate key of validation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AtomKey {
    catalog: String,
    name: String,
    port_no: u32,
    scope: Vec<(String, String)>,
    qualifiers: Vec<u32>,
}

impl AtomKey {
    /// Builds the canonical key of an atom. Scope bindings are sorted by
    /// (key, value); duplicate keys are NOT detected here — structural
    /// validation rejects them before a key is built.
    #[must_use]
    pub fn of(atom: &EffectAtom) -> Self {
        let mut scope: Vec<(String, String)> = atom
            .resource_selector
            .scope_bindings
            .iter()
            .map(|binding| (binding.key.clone(), binding.value.clone()))
            .collect();
        scope.sort();
        let mut qualifiers: Vec<u32> = atom.qualifiers.iter().map(|q| q.family_no()).collect();
        qualifiers.sort_unstable();
        AtomKey {
            catalog: atom.operation.catalog.clone(),
            name: atom.operation.name.clone(),
            port_no: atom.resource_selector.requirement_port_no,
            scope,
            qualifiers,
        }
    }

    /// The catalog identity half of the operation reference.
    #[must_use]
    pub fn catalog(&self) -> &str {
        &self.catalog
    }

    /// The operation name half of the operation reference.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The requirement port the atom acts through.
    #[must_use]
    pub fn port_no(&self) -> u32 {
        self.port_no
    }

    /// The sorted exact scope-binding set.
    #[must_use]
    pub fn scope(&self) -> &[(String, String)] {
        &self.scope
    }

    /// The sorted qualifier-family case numbers (part of the exact identity).
    #[must_use]
    pub fn qualifiers(&self) -> &[u32] {
        &self.qualifiers
    }

    /// Whether this atom's scope is narrower than or equal to `outer`'s:
    /// every constraint of `outer` is present here with the exact same value
    /// (more bindings = narrower scope). Exact value equality only — no
    /// prefix patterns. An unconstrained (empty) scope is the WIDEST scope;
    /// only the empty scope is narrower than it.
    #[must_use]
    pub fn scope_narrower_or_equal(&self, outer: &Self) -> bool {
        outer
            .scope
            .iter()
            .all(|binding| self.scope.contains(binding))
    }
}
