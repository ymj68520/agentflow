use crate::error::{PortableProofError, ValidationStage};
use crate::limits::{DecoderLimits, preflight_json};
use crate::orders;
use crate::schema::bundle_shape;
use crate::support::{SupportedPortableStaticInputs, validate_d4};
use crate::validation::{ContentValidatedPortableBundle, validate_d2, validate_d3};
use af_core::{Value, decode, encode};
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone)]
struct RawPortableBundle {
    value: Value,
    limits: DecoderLimits,
}

/// Output of the registered closed shape decoder.
#[derive(Debug, Clone)]
pub struct DecodedPortableBundle(RawPortableBundle);

/// Output of byte canonicality and registered set-order validation.
#[derive(Debug, Clone)]
pub struct CanonicalPortableBundle {
    raw: RawPortableBundle,
    bytes: Vec<u8>,
    pub(crate) limits: DecoderLimits,
}

impl CanonicalPortableBundle {
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.bytes
    }
    #[must_use]
    pub fn value(&self) -> &Value {
        &self.raw.value
    }
}

/// Pure schema-directed ACE1 and D0-D4 entry points for Portable Proof Inputs v1.
#[derive(Debug, Clone, Copy)]
pub struct PortableProofCodec {
    limits: DecoderLimits,
}

impl Default for PortableProofCodec {
    fn default() -> Self {
        Self::new(DecoderLimits::default())
    }
}

impl PortableProofCodec {
    #[must_use]
    pub const fn new(limits: DecoderLimits) -> Self {
        Self { limits }
    }

    /// Author-side canonical ACE1 encoding. It accepts a closed af-core Value
    /// only when that value matches the registered Bundle schema.
    pub fn encode(&self, value: &Value) -> Result<Vec<u8>, PortableProofError> {
        self.check_value_limits(value)?;
        ensure_canonical_text(value, "$")?;
        let shape = bundle_shape()?;
        let bytes = encode(shape, value).map_err(|error| from_ace_error(&error, "$"))?;
        if bytes.len() > self.limits.max_input_bytes {
            return Err(resource("$", "encoded byte limit"));
        }
        Ok(bytes)
    }

    /// D0: closed record/variant/enum/option and exact scalar-width decode.
    pub fn decode_d0(&self, bytes: &[u8]) -> Result<DecodedPortableBundle, PortableProofError> {
        preflight_json(bytes, self.limits)?;
        let shape = bundle_shape()?;
        let value = match decode(shape, bytes) {
            Ok(value) => value,
            Err(error) => {
                if matches!(error, af_core::AceError::NonCanonicalEncoding { .. }) {
                    if contains_malformed_option(bytes, shape) {
                        return Err(PortableProofError::new(
                            ValidationStage::D0,
                            "InvalidOption",
                            "$",
                            "Option requires exactly one tagged none/some case",
                        ));
                    }
                    if contains_primitive_shape_mismatch(bytes, shape) {
                        return Err(PortableProofError::new(
                            ValidationStage::D0,
                            "InvalidPrimitive",
                            "$",
                            "provided JSON value does not have its declared primitive shape",
                        ));
                    }
                }
                return Err(from_ace_error(&error, "$"));
            }
        };
        self.check_value_limits(&value)?;
        Ok(DecodedPortableBundle(RawPortableBundle {
            value,
            limits: self.limits,
        }))
    }

    /// D1: exact canonical bytes plus every registered collection ordering.
    pub fn validate_d1(
        &self,
        decoded: DecodedPortableBundle,
        original_bytes: &[u8],
    ) -> Result<CanonicalPortableBundle, PortableProofError> {
        if let Err(mut error) = ensure_canonical_text(&decoded.0.value, "$") {
            error.stage = ValidationStage::D1;
            error.family = "NoncanonicalString";
            return Err(error);
        }
        let shape = bundle_shape()?;
        let bytes = encode(shape, &decoded.0.value).map_err(|error| from_ace_error(&error, "$"))?;
        if bytes != original_bytes {
            return Err(PortableProofError::new(
                ValidationStage::D1,
                "NoncanonicalEncoding",
                "$",
                "input does not equal canonical ACE1 re-encoding",
            ));
        }
        orders::validate_registered_orders(shape, &decoded.0.value)?;
        let limits = decoded.0.limits;
        Ok(CanonicalPortableBundle {
            raw: decoded.0,
            bytes,
            limits,
        })
    }

    /// D2: validate all Bundle-local IDs, typed asset pins and graph edges.
    pub fn validate_references(
        &self,
        canonical: CanonicalPortableBundle,
    ) -> Result<crate::validation::ReferenceValidatedPortableBundle, PortableProofError> {
        validate_d2(canonical)
    }

    /// D3: validate inline Schema/IR/Contract bytes, checksums and registries.
    pub fn validate_contents(
        &self,
        referenced: crate::validation::ReferenceValidatedPortableBundle,
    ) -> Result<ContentValidatedPortableBundle, PortableProofError> {
        validate_d3(referenced)
    }

    /// D4: compare the complete compiler tuple against current local support.
    pub fn validate_semantics(
        &self,
        content: ContentValidatedPortableBundle,
    ) -> Result<SupportedPortableStaticInputs, PortableProofError> {
        validate_d4(content)
    }

    /// Executes D0-D4. The result represents current static semantic support
    /// only; it does not establish reconstruction or a RootExport verdict.
    pub fn validate_supported(
        &self,
        bytes: &[u8],
    ) -> Result<SupportedPortableStaticInputs, PortableProofError> {
        let decoded = self.decode_d0(bytes)?;
        let canonical = self.validate_d1(decoded, bytes)?;
        let referenced = self.validate_references(canonical)?;
        let content = self.validate_contents(referenced)?;
        self.validate_semantics(content)
    }

    fn check_value_limits(&self, root: &Value) -> Result<(), PortableProofError> {
        let mut pending = vec![(root, 0usize)];
        let mut count = 0usize;
        while let Some((value, depth)) = pending.pop() {
            count = count.saturating_add(1);
            if count > self.limits.max_total_nodes {
                return Err(resource("$", "typed node limit"));
            }
            if depth > self.limits.max_nesting {
                return Err(resource("$", "typed nesting limit"));
            }
            match value {
                Value::String(text) | Value::Enum(text) => {
                    if text.len() > self.limits.max_scalar_bytes {
                        return Err(resource("$", "text length limit"));
                    }
                }
                Value::Bytes(bytes) => {
                    if bytes.len() > self.limits.max_scalar_bytes {
                        return Err(resource("$", "byte string length limit"));
                    }
                }
                Value::List(items) => {
                    if items.len() > self.limits.max_container_items {
                        return Err(resource("$", "array item limit"));
                    }
                    pending.extend(items.iter().map(|item| (item, depth + 1)));
                }
                Value::Map(items) | Value::Record(items) => {
                    if items.len() > self.limits.max_container_items {
                        return Err(resource("$", "record member limit"));
                    }
                    pending.extend(items.iter().map(|(_, item)| (item, depth + 1)));
                }
                Value::Option(Some(inner)) | Value::Variant { value: inner, .. } => {
                    pending.push((inner, depth + 1));
                }
                _ => {}
            }
        }
        Ok(())
    }
}

fn contains_malformed_option(bytes: &[u8], shape: &af_core::Shape) -> bool {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(bytes) else {
        return false;
    };
    invalid_option_at(&value, shape)
}

fn contains_primitive_shape_mismatch(bytes: &[u8], shape: &af_core::Shape) -> bool {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(bytes) else {
        return false;
    };
    primitive_shape_mismatch(&value, shape)
}

fn primitive_shape_mismatch(value: &serde_json::Value, shape: &af_core::Shape) -> bool {
    use af_core::Shape;
    match shape {
        Shape::Unit | Shape::Null => !value.is_null(),
        Shape::Bool => !value.is_boolean(),
        Shape::Int32
        | Shape::UInt32
        | Shape::Int64
        | Shape::UInt64
        | Shape::Decimal
        | Shape::Float64
        | Shape::String
        | Shape::Bytes
        | Shape::Timestamp
        | Shape::Duration
        | Shape::Enum(_) => !value.is_string(),
        Shape::Option(inner) => {
            let Some(object) = value.as_object() else {
                return true;
            };
            if object.len() != 1 {
                return true;
            }
            if let Some(payload) = object.get("none") {
                return !payload.is_null();
            }
            object
                .get("some")
                .is_none_or(|payload| primitive_shape_mismatch(payload, inner))
        }
        Shape::Record(record) => {
            let Some(object) = value.as_object() else {
                return true;
            };
            record.fields.iter().any(|field| {
                object
                    .get(&field.name)
                    .is_some_and(|child| primitive_shape_mismatch(child, &field.shape))
            })
        }
        Shape::Variant(variant) => {
            let Some(object) = value.as_object() else {
                return true;
            };
            if object.len() != 1 {
                return false;
            }
            let Some((case, payload)) = object.iter().next() else {
                return false;
            };
            variant
                .cases
                .iter()
                .find(|candidate| candidate.name == *case)
                .is_some_and(|candidate| primitive_shape_mismatch(payload, &candidate.payload))
        }
        Shape::List(inner) => value.as_array().is_none_or(|items| {
            items
                .iter()
                .any(|item| primitive_shape_mismatch(item, inner))
        }),
        Shape::Map(inner) => value.as_object().is_none_or(|items| {
            items
                .values()
                .any(|item| primitive_shape_mismatch(item, inner))
        }),
    }
}

fn invalid_option_at(value: &serde_json::Value, shape: &af_core::Shape) -> bool {
    use af_core::Shape;
    match shape {
        Shape::Option(inner) => {
            let Some(object) = value.as_object() else {
                return true;
            };
            if object.len() != 1 {
                return true;
            }
            if let Some(payload) = object.get("none") {
                return !payload.is_null();
            }
            if let Some(payload) = object.get("some") {
                return invalid_option_at(payload, inner);
            }
            true
        }
        Shape::Record(record) => value.as_object().is_some_and(|object| {
            record.fields.iter().any(|field| {
                object
                    .get(&field.name)
                    .is_some_and(|child| invalid_option_at(child, &field.shape))
            })
        }),
        Shape::Variant(variant) => value.as_object().is_some_and(|object| {
            if object.len() != 1 {
                return false;
            }
            let Some((case, payload)) = object.iter().next() else {
                return false;
            };
            variant
                .cases
                .iter()
                .find(|candidate| candidate.name == *case)
                .is_some_and(|candidate| invalid_option_at(payload, &candidate.payload))
        }),
        Shape::List(inner) => value
            .as_array()
            .is_some_and(|items| items.iter().any(|item| invalid_option_at(item, inner))),
        Shape::Map(inner) => value
            .as_object()
            .is_some_and(|items| items.values().any(|item| invalid_option_at(item, inner))),
        _ => false,
    }
}

fn ensure_canonical_text(value: &Value, path: &str) -> Result<(), PortableProofError> {
    match value {
        Value::String(text) | Value::Enum(text) if !text.nfc().eq(text.chars()) => {
            Err(PortableProofError::new(
                ValidationStage::D0,
                "NoncanonicalIdentity",
                path,
                "v1 identities must already be NFC",
            ))
        }
        Value::List(values) => {
            for (index, item) in values.iter().enumerate() {
                ensure_canonical_text(item, &format!("{path}[{index}]"))?;
            }
            Ok(())
        }
        Value::Map(values) | Value::Record(values) => {
            for (name, item) in values {
                if !name.nfc().eq(name.chars()) {
                    return Err(PortableProofError::new(
                        ValidationStage::D0,
                        "NoncanonicalIdentity",
                        format!("{path}.{name}"),
                        "record/map identity key must already be NFC",
                    ));
                }
                ensure_canonical_text(item, &format!("{path}.{name}"))?;
            }
            Ok(())
        }
        Value::Option(Some(inner)) | Value::Variant { value: inner, .. } => {
            ensure_canonical_text(inner, path)
        }
        _ => Ok(()),
    }
}

fn from_ace_error(error: &af_core::AceError, path: &str) -> PortableProofError {
    use af_core::AceError as E;
    let (stage, family) = match &error {
        E::NonCanonicalEncoding { .. } => (ValidationStage::D1, "NoncanonicalEncoding"),
        E::InvalidUnicode { detail } if detail.contains("not NFC") => {
            (ValidationStage::D1, "NoncanonicalString")
        }
        E::InvalidUnicode { .. } => (ValidationStage::D0, "InvalidUnicode"),
        E::DuplicateField { .. } | E::DuplicateNormalizedKey { .. } => {
            (ValidationStage::D0, "DuplicateField")
        }
        E::UnknownField { .. } => (ValidationStage::D0, "UnknownField"),
        E::MissingField { .. } => (ValidationStage::D0, "MissingField"),
        E::UnknownVariantCase { .. } => (ValidationStage::D0, "UnknownVariant"),
        E::UnknownEnumValue { .. } => (ValidationStage::D0, "UnknownEnumCase"),
        E::InvalidOptionEncoding { .. } => (ValidationStage::D0, "InvalidOption"),
        E::InvalidInteger { input }
            if input == "-0" || (input.len() > 1 && input.starts_with('0')) =>
        {
            (ValidationStage::D1, "NoncanonicalInteger")
        }
        E::InvalidInteger { .. } => (ValidationStage::D0, "InvalidPrimitive"),
        E::IntegerOutOfRange { .. } => (ValidationStage::D0, "NumericRange"),
        E::InvalidDecimal { .. } | E::TypeMismatch { .. } | E::NonFiniteFloat => {
            (ValidationStage::D0, "InvalidPrimitive")
        }
        E::InvalidBytesEncoding { .. } => (ValidationStage::D0, "InvalidBytesEncoding"),
        E::InvalidShape { .. } => (ValidationStage::D0, "InvalidSchemaShape"),
    };
    PortableProofError::new(stage, family, path, error.to_string())
}

fn resource(path: &str, detail: &'static str) -> PortableProofError {
    PortableProofError::new(ValidationStage::D0, "ResourceLimit", path, detail)
}
