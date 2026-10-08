//! Typed static Usage Signature model and the derived, non-wire usage envelope.
//!
//! A Usage Signature is a declaration of additive upper bounds. It is never
//! a usage estimate, measured evidence, billing record or runtime ledger.

use crate::error::{UsageError, kind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One nominal usage-meter identity. The unit and aggregation semantics are
/// resolved by [`crate::universe::UsageMeterUniverse`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UsageMeterRef {
    pub catalog: String,
    pub name: String,
}

impl UsageMeterRef {
    /// The nominal identity key.
    #[must_use]
    pub fn identity_key(&self) -> (&str, &str) {
        (&self.catalog, &self.name)
    }
}

/// The independent accounting phase of one usage coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UsagePhase {
    Forward,
    Recovery,
}

impl UsagePhase {
    /// The frozen AFIDL case number, also used for canonical ordering.
    #[must_use]
    pub fn value_no(self) -> u32 {
        match self {
            Self::Forward => 1,
            Self::Recovery => 2,
        }
    }

    /// The stable machine spelling.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Forward => "forward",
            Self::Recovery => "recovery",
        }
    }
}

/// A declared finite upper bound or an explicit absence of a finite ceiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageUpperBound {
    Finite(u64),
    Unbounded,
}

impl UsageUpperBound {
    /// Whether this bound is explicitly unbounded.
    #[must_use]
    pub fn is_unbounded(self) -> bool {
        matches!(self, Self::Unbounded)
    }

    /// Whether this is the explicit zero bound.
    #[must_use]
    pub fn is_zero(self) -> bool {
        matches!(self, Self::Finite(0))
    }

    /// Whether this ceiling covers another upper bound.
    #[must_use]
    pub fn covers(self, inner: Self) -> bool {
        match (self, inner) {
            (Self::Unbounded, _) => true,
            (Self::Finite(_), Self::Unbounded) => false,
            (Self::Finite(outer), Self::Finite(inner)) => outer >= inner,
        }
    }

    /// Checked additive composition.
    pub fn checked_add(self, other: Self) -> Result<Self, UsageError> {
        match (self, other) {
            (Self::Unbounded, _) | (_, Self::Unbounded) => Ok(Self::Unbounded),
            (Self::Finite(a), Self::Finite(b)) => {
                a.checked_add(b).map(Self::Finite).ok_or_else(|| {
                    UsageError::new(
                        kind::USAGE_ARITHMETIC_OVERFLOW,
                        format!("usage upper bounds {a} + {b} overflow u64"),
                    )
                })
            }
        }
    }

    /// Checked multiplicative composition.
    pub fn checked_mul(self, other: u64) -> Result<Self, UsageError> {
        if other == 0 {
            return Ok(Self::Finite(0));
        }
        match self {
            Self::Unbounded => Ok(Self::Unbounded),
            Self::Finite(value) => value.checked_mul(other).map(Self::Finite).ok_or_else(|| {
                UsageError::new(
                    kind::USAGE_ARITHMETIC_OVERFLOW,
                    format!("usage upper bound {value} * {other} overflows u64"),
                )
            }),
        }
    }

    /// The greater bound, with `Unbounded` as the greatest element.
    #[must_use]
    pub fn max(self, other: Self) -> Self {
        match (self, other) {
            (Self::Unbounded, _) | (_, Self::Unbounded) => Self::Unbounded,
            (Self::Finite(a), Self::Finite(b)) => Self::Finite(a.max(b)),
        }
    }
}

/// One declared static usage ceiling.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UsageLimit {
    pub meter: UsageMeterRef,
    pub phase: UsagePhase,
    pub upper: UsageUpperBound,
}

/// The closed static Usage Signature U.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UsageSignature {
    pub limits: Vec<UsageLimit>,
}

impl UsageSignature {
    /// The empty declaration. It is not a proof of zero usage for any meter.
    #[must_use]
    pub fn empty() -> Self {
        Self { limits: Vec::new() }
    }
}

/// The coordinate used by the derived static envelope.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UsageCoordinate {
    pub phase: UsagePhase,
    pub meter: UsageMeterRef,
}

/// A computed static upper-bound state. This is analysis-only and is not an
/// AFIDL type, a digest input or runtime usage evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticUsageBound {
    /// A finite, statically proven upper bound.
    KnownFinite(u64),
    /// An explicitly unbounded static upper bound.
    KnownUnbounded,
    /// No finite upper bound was proven for this coordinate.
    Unknown,
}

impl StaticUsageBound {
    /// Stable machine spelling for diagnostics and the corpus.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::KnownFinite(_) => "known_finite",
            Self::KnownUnbounded => "known_unbounded",
            Self::Unknown => "unknown",
        }
    }

    /// Converts a declared upper bound to a known analysis state.
    #[must_use]
    pub fn from_declared(upper: UsageUpperBound) -> Self {
        match upper {
            UsageUpperBound::Finite(value) => Self::KnownFinite(value),
            UsageUpperBound::Unbounded => Self::KnownUnbounded,
        }
    }
}

/// Derived static usage envelope, keyed by the independent phase/meter
/// coordinate. A missing coordinate is deliberately observed as `Unknown`,
/// never as `KnownFinite(0)`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StaticUsageEnvelope {
    bounds: BTreeMap<UsageCoordinate, StaticUsageBound>,
    globally_unknown: bool,
}

impl StaticUsageEnvelope {
    /// A precise envelope with no known coordinates. This is not a zero proof.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// An envelope whose usage cannot be bounded for any coordinate.
    #[must_use]
    pub fn unknown() -> Self {
        Self {
            bounds: BTreeMap::new(),
            globally_unknown: true,
        }
    }

    /// Inserts one coordinate state. Duplicate coordinates are rejected so
    /// analysis code cannot silently overwrite a bound.
    pub fn insert(
        &mut self,
        coordinate: UsageCoordinate,
        bound: StaticUsageBound,
    ) -> Result<(), UsageError> {
        if self.bounds.insert(coordinate, bound).is_some() {
            return Err(UsageError::new(
                kind::DUPLICATE_USAGE_LIMIT,
                "a static usage envelope coordinate was inserted twice",
            ));
        }
        Ok(())
    }

    /// Gets one coordinate; missing coordinates are `Unknown`.
    #[must_use]
    pub fn get(&self, coordinate: &UsageCoordinate) -> StaticUsageBound {
        self.bounds
            .get(coordinate)
            .copied()
            .unwrap_or(StaticUsageBound::Unknown)
    }

    /// Whether the envelope is globally unknown.
    #[must_use]
    pub fn is_globally_unknown(&self) -> bool {
        self.globally_unknown
    }

    /// Stable coordinate order for inspection and corpus projection.
    pub fn iter(&self) -> impl Iterator<Item = (&UsageCoordinate, &StaticUsageBound)> {
        self.bounds.iter()
    }

    /// Number of explicitly represented coordinates.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bounds.len()
    }

    /// Whether no coordinate is explicitly represented.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bounds.is_empty() && !self.globally_unknown
    }

    /// Returns all explicit coordinates, in stable order.
    #[must_use]
    pub fn coordinates(&self) -> Vec<UsageCoordinate> {
        self.bounds.keys().cloned().collect()
    }
}
