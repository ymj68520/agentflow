//! Structured error types for ACE1 encoding/decoding and digest computation.
//!
//! Error categories are stable (`AceError::kind` / `DigestError::kind`) and are
//! referenced by the negative conformance corpus; they must not be renamed
//! casually.

use std::fmt;

/// Stable error-category names used by `spec/conformance/encoding/negative.json`.
pub mod kind {
    pub const INVALID_INTEGER: &str = "invalid_integer";
    pub const INTEGER_OUT_OF_RANGE: &str = "integer_out_of_range";
    pub const INVALID_DECIMAL: &str = "invalid_decimal";
    pub const NON_FINITE_FLOAT: &str = "non_finite_float";
    pub const INVALID_UNICODE: &str = "invalid_unicode";
    pub const DUPLICATE_NORMALIZED_KEY: &str = "duplicate_normalized_key";
    pub const DUPLICATE_FIELD: &str = "duplicate_field";
    pub const UNKNOWN_FIELD: &str = "unknown_field";
    pub const MISSING_FIELD: &str = "missing_field";
    pub const INVALID_BYTES_ENCODING: &str = "invalid_bytes_encoding";
    pub const UNKNOWN_ENUM_VALUE: &str = "unknown_enum_value";
    pub const UNKNOWN_VARIANT_CASE: &str = "unknown_variant_case";
    pub const INVALID_OPTION_ENCODING: &str = "invalid_option_encoding";
    pub const TYPE_MISMATCH: &str = "type_mismatch";
    pub const NON_CANONICAL_ENCODING: &str = "non_canonical_encoding";
    pub const INVALID_SHAPE: &str = "invalid_shape";
    pub const INVALID_DIGEST: &str = "invalid_digest";
    pub const UNKNOWN_DIGEST_DOMAIN: &str = "unknown_digest_domain";
}

/// ACE1 typed encoding/decoding errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AceError {
    /// Text is not a canonical machine integer (leading zeros, `+` sign, `-0`,
    /// empty, non-ASCII digits, junk characters).
    InvalidInteger { input: String },
    /// Text parses as an integer but does not fit the declared primitive range.
    IntegerOutOfRange { input: String, type_name: String },
    /// Text is not a legal decimal literal, or is not in canonical form where
    /// canonical form is mandatory.
    InvalidDecimal { input: String },
    /// A float64 value is NaN or an infinity; ACE1 forbids both.
    NonFiniteFloat,
    /// Input bytes are not valid UTF-8, contain a lone surrogate escape, or are
    /// not NFC where NFC is mandatory.
    InvalidUnicode { detail: String },
    /// Two map keys (or an equivalent pair) normalize to the same NFC text.
    DuplicateNormalizedKey { key: String },
    /// A record field was supplied more than once.
    DuplicateField { field: String },
    /// A record field is not declared by the shape.
    UnknownField { field: String },
    /// A declared record field was not supplied.
    MissingField { field: String },
    /// Input is not unpadded base64url (bad alphabet, padding, length, or
    /// canonical re-encode mismatch).
    InvalidBytesEncoding { detail: String },
    /// A closed enum received a value outside its declared set.
    UnknownEnumValue { value: String },
    /// A closed variant received a case outside its declared set.
    UnknownVariantCase { case: String },
    /// An option value is not encoded as the mandatory single-case tagged
    /// object `{"none":null}` / `{"some":<v>}` (wrong key, wrong member count,
    /// or a non-null payload under "none").
    InvalidOptionEncoding { detail: String },
    /// The typed value does not match the declared shape (this is how nominal
    /// distinctions such as timestamp vs duration vs int64 are enforced).
    TypeMismatch { expected: String, actual: String },
    /// Bytes are valid-looking but not canonical ACE1 (whitespace, unsorted or
    /// duplicate object keys, non-shortest escapes, trailing data, ...).
    /// Decoder-only: the encoder can never produce these.
    NonCanonicalEncoding { detail: String },
    /// The shape itself is invalid (duplicate names, non-NFC names, ...).
    InvalidShape { detail: String },
}

impl AceError {
    pub fn invalid_shape(detail: impl Into<String>) -> Self {
        AceError::InvalidShape {
            detail: detail.into(),
        }
    }

    pub fn non_canonical(detail: impl Into<String>) -> Self {
        AceError::NonCanonicalEncoding {
            detail: detail.into(),
        }
    }

    /// Stable category identifier; referenced by the negative conformance corpus.
    pub fn kind(&self) -> &'static str {
        match self {
            AceError::InvalidInteger { .. } => kind::INVALID_INTEGER,
            AceError::IntegerOutOfRange { .. } => kind::INTEGER_OUT_OF_RANGE,
            AceError::InvalidDecimal { .. } => kind::INVALID_DECIMAL,
            AceError::NonFiniteFloat => kind::NON_FINITE_FLOAT,
            AceError::InvalidUnicode { .. } => kind::INVALID_UNICODE,
            AceError::DuplicateNormalizedKey { .. } => kind::DUPLICATE_NORMALIZED_KEY,
            AceError::DuplicateField { .. } => kind::DUPLICATE_FIELD,
            AceError::UnknownField { .. } => kind::UNKNOWN_FIELD,
            AceError::MissingField { .. } => kind::MISSING_FIELD,
            AceError::InvalidBytesEncoding { .. } => kind::INVALID_BYTES_ENCODING,
            AceError::UnknownEnumValue { .. } => kind::UNKNOWN_ENUM_VALUE,
            AceError::UnknownVariantCase { .. } => kind::UNKNOWN_VARIANT_CASE,
            AceError::InvalidOptionEncoding { .. } => kind::INVALID_OPTION_ENCODING,
            AceError::TypeMismatch { .. } => kind::TYPE_MISMATCH,
            AceError::NonCanonicalEncoding { .. } => kind::NON_CANONICAL_ENCODING,
            AceError::InvalidShape { .. } => kind::INVALID_SHAPE,
        }
    }
}

impl fmt::Display for AceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AceError::InvalidInteger { input } => {
                write!(f, "invalid canonical integer text: {input:?}")
            }
            AceError::IntegerOutOfRange { input, type_name } => {
                write!(f, "integer {input:?} out of range for {type_name}")
            }
            AceError::InvalidDecimal { input } => {
                write!(f, "invalid or non-canonical decimal text: {input:?}")
            }
            AceError::NonFiniteFloat => {
                write!(
                    f,
                    "float64 must be finite: NaN and infinities are forbidden"
                )
            }
            AceError::InvalidUnicode { detail } => write!(f, "invalid unicode: {detail}"),
            AceError::DuplicateNormalizedKey { key } => {
                write!(f, "map keys collide after NFC normalization: {key:?}")
            }
            AceError::DuplicateField { field } => write!(f, "duplicate record field {field:?}"),
            AceError::UnknownField { field } => write!(f, "unknown record field {field:?}"),
            AceError::MissingField { field } => write!(f, "missing record field {field:?}"),
            AceError::InvalidBytesEncoding { detail } => {
                write!(f, "invalid base64url encoding: {detail}")
            }
            AceError::UnknownEnumValue { value } => {
                write!(f, "unknown enum value {value:?} for closed enum")
            }
            AceError::UnknownVariantCase { case } => {
                write!(f, "unknown variant case {case:?} for closed variant")
            }
            AceError::InvalidOptionEncoding { detail } => {
                write!(f, "invalid option encoding: {detail}")
            }
            AceError::TypeMismatch { expected, actual } => {
                write!(f, "type mismatch: expected {expected}, got {actual}")
            }
            AceError::NonCanonicalEncoding { detail } => {
                write!(f, "input is not canonical ACE1: {detail}")
            }
            AceError::InvalidShape { detail } => write!(f, "invalid shape: {detail}"),
        }
    }
}

impl std::error::Error for AceError {}

/// Digest (HashV1 / DigestRef) errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DigestError {
    /// Digest text/structure is invalid: wrong algorithm, wrong length,
    /// non-hex or uppercase hex characters, malformed composite form.
    InvalidDigest { detail: String },
    /// A digest domain outside the frozen v1 registry was requested.
    UnknownDigestDomain { domain: String },
}

impl DigestError {
    /// Stable category identifier; referenced by the negative conformance corpus.
    pub fn kind(&self) -> &'static str {
        match self {
            DigestError::InvalidDigest { .. } => kind::INVALID_DIGEST,
            DigestError::UnknownDigestDomain { .. } => kind::UNKNOWN_DIGEST_DOMAIN,
        }
    }
}

impl fmt::Display for DigestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DigestError::InvalidDigest { detail } => write!(f, "invalid digest: {detail}"),
            DigestError::UnknownDigestDomain { domain } => {
                write!(f, "unknown digest domain {domain:?}")
            }
        }
    }
}

impl std::error::Error for DigestError {}
