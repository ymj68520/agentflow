//! Unit-returning source/metadata admission, never a producer or Fact constructor.
use crate::{
    CompilerSemanticsIdentity, EditionRegistration, check_registered_compiler_identity,
    current_registered_analysis_edition, validate_registered_model_shape,
};
use af_contract::{ComponentContract, StaticContractProofContext};
use serde::de::{Deserialize, Deserializer, Error as _, MapAccess, SeqAccess, Visitor};
use serde_json::Value;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawAdmissionFailure {
    MissingDeclarativeOrigin,
    InvalidSourceAssociation,
    InvalidContextBinding,
    InvalidIdentity,
    InvalidPhysicalInput,
    UnsupportedModel,
    ResourceLimit,
}

/// A borrowed actual declarative document. No implicit per-call document exists.
pub struct NativeRawDocument<'a> {
    pub declarations: &'a [ComponentContract],
}

fn check_source_compiler(compiler: &CompilerSemanticsIdentity) -> Result<(), RawAdmissionFailure> {
    check_registered_compiler_identity(compiler)
        .map_err(|_| RawAdmissionFailure::InvalidIdentity)?;
    if current_registered_analysis_edition().registration != EditionRegistration::Registered {
        return Err(RawAdmissionFailure::InvalidIdentity);
    }
    Ok(())
}

/// Verifies a fresh declared native association, not historical provenance.
/// The public caller gets unit only and cannot obtain a trusted predicate fact.
pub fn admit_native_raw_source(
    candidate: &ComponentContract,
    document: Option<&NativeRawDocument<'_>>,
    original_entry: usize,
    analysis: &StaticContractProofContext<'_>,
    canonical_validation: &StaticContractProofContext<'_>,
    compiler: &CompilerSemanticsIdentity,
) -> Result<(), RawAdmissionFailure> {
    check_source_compiler(compiler)?;
    if !std::ptr::eq(analysis, canonical_validation) {
        return Err(RawAdmissionFailure::InvalidContextBinding);
    }
    let document = document.ok_or(RawAdmissionFailure::MissingDeclarativeOrigin)?;
    if document.declarations.get(original_entry) != Some(candidate) {
        return Err(RawAdmissionFailure::InvalidSourceAssociation);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawDocumentMember {
    Field(Vec<u8>),
    Entry(u64),
}

/// Physical JSON is the registered non-wire Raw metadata representation.
/// It is NOT a ComponentContract wire decoder or successful canonical encoder.
pub fn admit_physical_raw_source(
    candidate: &Value,
    original_bytes: Option<&[u8]>,
    path: &[RawDocumentMember],
    analysis: &StaticContractProofContext<'_>,
    canonical_validation: &StaticContractProofContext<'_>,
    compiler: &CompilerSemanticsIdentity,
) -> Result<(), RawAdmissionFailure> {
    check_source_compiler(compiler)?;
    if !std::ptr::eq(analysis, canonical_validation) {
        return Err(RawAdmissionFailure::InvalidContextBinding);
    }
    let bytes = original_bytes.ok_or(RawAdmissionFailure::MissingDeclarativeOrigin)?;
    if bytes.len() > 8 * 1024 * 1024 || path.len() > 96 {
        return Err(RawAdmissionFailure::ResourceLimit);
    }
    let source =
        std::str::from_utf8(bytes).map_err(|_| RawAdmissionFailure::InvalidPhysicalInput)?;
    let mut decoder = serde_json::Deserializer::from_str(source);
    let parsed = StrictJson::deserialize(&mut decoder)
        .map_err(|_| RawAdmissionFailure::InvalidPhysicalInput)?
        .0;
    decoder
        .end()
        .map_err(|_| RawAdmissionFailure::InvalidPhysicalInput)?;
    let mut selected = &parsed;
    for step in path {
        selected = match step {
            RawDocumentMember::Field(name) => selected.get(
                std::str::from_utf8(name).map_err(|_| RawAdmissionFailure::InvalidPhysicalInput)?,
            ),
            RawDocumentMember::Entry(index) => {
                usize::try_from(*index).ok().and_then(|i| selected.get(i))
            }
        }
        .ok_or(RawAdmissionFailure::InvalidSourceAssociation)?;
    }
    validate_registered_model_shape("RawCandidateContract", selected)
        .map_err(|_| RawAdmissionFailure::UnsupportedModel)?;
    if selected != candidate {
        return Err(RawAdmissionFailure::InvalidSourceAssociation);
    }
    Ok(())
}

// Unlike serde_json::Value's ordinary map decoder, rejects duplicate keys
// recursively BEFORE selection. No normalization or unknown-field dropping.
struct StrictJson(Value);
impl<'de> Deserialize<'de> for StrictJson {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = StrictJson;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("duplicate-free JSON")
            }
            fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(StrictJson(Value::Bool(v)))
            }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(StrictJson(v.into()))
            }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(StrictJson(v.into()))
            }
            fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Self::Value, E> {
                serde_json::Number::from_f64(v)
                    .map(|n| StrictJson(Value::Number(n)))
                    .ok_or_else(|| E::custom("nonfinite JSON"))
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(StrictJson(Value::String(v.into())))
            }
            fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(StrictJson(Value::String(v)))
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(StrictJson(Value::Null))
            }
            fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(StrictJson(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
                let mut v = Vec::new();
                while let Some(x) = a.next_element::<StrictJson>()? {
                    v.push(x.0);
                }
                Ok(StrictJson(Value::Array(v)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
                let mut v = serde_json::Map::new();
                while let Some(k) = a.next_key::<String>()? {
                    if v.contains_key(&k) {
                        return Err(A::Error::custom("duplicate JSON key"));
                    }
                    v.insert(k, a.next_value::<StrictJson>()?.0);
                }
                Ok(StrictJson(Value::Object(v)))
            }
        }
        d.deserialize_any(V)
    }
}
