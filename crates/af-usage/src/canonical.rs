//! Canonical Usage Signature ordering, explicit meter resolution and ACE1.
//!
//! Canonicalization never drops `finite(0)`, rewrites `unbounded` to
//! `u64::MAX`, or treats a missing meter as zero.

use crate::error::{UsageError, kind};
use crate::model::{UsageLimit, UsageSignature, UsageUpperBound};
use crate::universe::UsageMeterUniverse;
use crate::validate::validate_meter;
use af_core::Value;
use std::collections::BTreeSet;

/// Canonicalizes one Usage Signature against an explicit meter universe.
pub fn canonicalize_usage_signature(
    signature: &UsageSignature,
    universe: &UsageMeterUniverse,
) -> Result<UsageSignature, UsageError> {
    let mut seen = BTreeSet::new();
    let mut limits = Vec::with_capacity(signature.limits.len());
    for limit in &signature.limits {
        let meter = validate_meter(&limit.meter)?;
        universe.resolve(&meter)?;
        let key = (limit.phase, meter.catalog.clone(), meter.name.clone());
        if !seen.insert(key) {
            return Err(UsageError::new(
                kind::DUPLICATE_USAGE_LIMIT,
                format!(
                    "usage limit {}/{} in phase {} is declared twice",
                    meter.catalog,
                    meter.name,
                    limit.phase.as_str()
                ),
            ));
        }
        limits.push(UsageLimit {
            meter,
            phase: limit.phase,
            upper: limit.upper,
        });
    }
    limits.sort_by(|a, b| {
        (a.phase.value_no(), &a.meter.catalog, &a.meter.name).cmp(&(
            b.phase.value_no(),
            &b.meter.catalog,
            &b.meter.name,
        ))
    });
    Ok(UsageSignature { limits })
}

/// Derives the known coordinates of a canonical declaration. An empty
/// declaration remains empty and is not a zero envelope for absent meters.
pub fn declared_usage_envelope(
    signature: &UsageSignature,
    universe: &UsageMeterUniverse,
) -> Result<crate::model::StaticUsageEnvelope, UsageError> {
    let canonical = canonicalize_usage_signature(signature, universe)?;
    let mut envelope = crate::model::StaticUsageEnvelope::empty();
    for limit in canonical.limits {
        envelope.insert(
            crate::model::UsageCoordinate {
                phase: limit.phase,
                meter: limit.meter,
            },
            crate::model::StaticUsageBound::from_declared(limit.upper),
        )?;
    }
    Ok(envelope)
}

impl UsageSignature {
    /// The canonical form resolved against the explicit meter universe.
    pub fn canonical(&self, universe: &UsageMeterUniverse) -> Result<UsageSignature, UsageError> {
        canonicalize_usage_signature(self, universe)
    }

    /// The derived static envelope of the declaration.
    pub fn static_envelope(
        &self,
        universe: &UsageMeterUniverse,
    ) -> Result<crate::model::StaticUsageEnvelope, UsageError> {
        declared_usage_envelope(self, universe)
    }

    /// Canonical ACE1 bytes; no UsageSignatureDigest is produced.
    pub fn canonical_bytes(
        &self,
        universe: &UsageMeterUniverse,
        set: &af_idl::CompiledModuleSet,
    ) -> Result<Vec<u8>, UsageError> {
        let shape = set
            .shape_of("agentflow.usage.signature@1/UsageSignature")
            .map_err(|error| {
                UsageError::new(
                    kind::USAGE_ENCODING_ERROR,
                    format!("cannot derive usage signature shape: {error}"),
                )
            })?;
        shape.validate().map_err(|error| {
            UsageError::new(
                kind::USAGE_ENCODING_ERROR,
                format!("derived usage shape is invalid: {error}"),
            )
        })?;
        af_core::encode(&shape, &self.canonical(universe)?.to_value()?).map_err(|error| {
            UsageError::new(
                kind::USAGE_ENCODING_ERROR,
                format!("cannot encode canonical usage signature: {error}"),
            )
        })
    }

    /// The ACE1 value tree of the canonical declaration.
    pub fn to_value(&self) -> Result<Value, UsageError> {
        Ok(Value::Record(vec![(
            "limits".into(),
            Value::List(self.limits.iter().map(limit_value).collect()),
        )]))
    }
}

fn limit_value(limit: &UsageLimit) -> Value {
    Value::Record(vec![
        (
            "meter".into(),
            Value::Record(vec![
                ("catalog".into(), Value::String(limit.meter.catalog.clone())),
                ("name".into(), Value::String(limit.meter.name.clone())),
            ]),
        ),
        (
            "phase".into(),
            Value::Enum(limit.phase.as_str().to_string()),
        ),
        (
            "upper".into(),
            match limit.upper {
                UsageUpperBound::Finite(value) => Value::Variant {
                    case: "finite".into(),
                    value: Box::new(Value::UInt64(value)),
                },
                UsageUpperBound::Unbounded => Value::Variant {
                    case: "unbounded".into(),
                    value: Box::new(Value::Unit),
                },
            },
        ),
    ])
}
