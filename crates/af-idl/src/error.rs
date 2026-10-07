//! Structured AFIDL1 errors with stable category kinds.
//!
//! Categories are stable identifiers referenced by the negative conformance
//! corpus (`spec/conformance/afidl/parser-invalid/manifest.json`); they must
//! not be renamed casually.

use crate::span::SourceSpan;
use std::fmt;

/// Stable error-kind names used by the negative corpus manifest.
pub mod kind {
    pub const LEX: &str = "lex_error";
    pub const PARSE: &str = "parse_error";
    pub const INVALID_IDENTIFIER: &str = "invalid_identifier";
    pub const INVALID_LANGUAGE_VERSION: &str = "invalid_language_version";
    pub const INVALID_SPECIFICATION_ID: &str = "invalid_specification_id";
    pub const DUPLICATE_SCHEMA_DECLARATION: &str = "duplicate_schema_declaration";
    pub const DUPLICATE_TYPE_DEFINITION: &str = "duplicate_type_definition";
    pub const INVALID_FIELD_NUMBER: &str = "invalid_field_number";
    pub const DUPLICATE_FIELD_NUMBER: &str = "duplicate_field_number";
    pub const DUPLICATE_FIELD_NAME: &str = "duplicate_field_name";
    pub const INVALID_CASE_NUMBER: &str = "invalid_case_number";
    pub const DUPLICATE_CASE_NUMBER: &str = "duplicate_case_number";
    pub const DUPLICATE_CASE_NAME: &str = "duplicate_case_name";
    pub const INVALID_ENUM_NUMBER: &str = "invalid_enum_number";
    pub const DUPLICATE_ENUM_NUMBER: &str = "duplicate_enum_number";
    pub const DUPLICATE_ENUM_NAME: &str = "duplicate_enum_name";
    pub const EMPTY_VARIANT: &str = "empty_variant";
    pub const EMPTY_ENUM: &str = "empty_enum";
    pub const UNKNOWN_TYPE_REFERENCE: &str = "unknown_type_reference";
    pub const ALIAS_CYCLE: &str = "alias_cycle";
    pub const RECURSIVE_TYPE_NOT_SUPPORTED: &str = "recursive_type_not_supported";
}

/// AFIDL1 lexing, parsing and validation errors. Every error carries the
/// source span of the offending construct where one exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AfidlError {
    Lex {
        detail: String,
        span: SourceSpan,
    },
    Parse {
        detail: String,
        span: SourceSpan,
    },
    InvalidIdentifier {
        detail: String,
        span: SourceSpan,
    },
    InvalidLanguageVersion {
        found: u32,
        span: SourceSpan,
    },
    InvalidSpecificationId {
        detail: String,
        span: SourceSpan,
    },
    DuplicateSchemaDeclaration {
        span: SourceSpan,
    },
    DuplicateTypeDefinition {
        name: String,
        span: SourceSpan,
    },
    InvalidFieldNumber {
        no: u32,
        type_name: String,
        span: SourceSpan,
    },
    DuplicateFieldNumber {
        no: u32,
        type_name: String,
        span: SourceSpan,
    },
    DuplicateFieldName {
        name: String,
        type_name: String,
        span: SourceSpan,
    },
    InvalidCaseNumber {
        no: u32,
        type_name: String,
        span: SourceSpan,
    },
    DuplicateCaseNumber {
        no: u32,
        type_name: String,
        span: SourceSpan,
    },
    DuplicateCaseName {
        name: String,
        type_name: String,
        span: SourceSpan,
    },
    InvalidEnumNumber {
        no: u32,
        type_name: String,
        span: SourceSpan,
    },
    DuplicateEnumNumber {
        no: u32,
        type_name: String,
        span: SourceSpan,
    },
    DuplicateEnumName {
        name: String,
        type_name: String,
        span: SourceSpan,
    },
    EmptyVariant {
        name: String,
        span: SourceSpan,
    },
    EmptyEnum {
        name: String,
        span: SourceSpan,
    },
    UnknownTypeReference {
        name: String,
        span: SourceSpan,
    },
    AliasCycle {
        cycle: Vec<String>,
        span: SourceSpan,
    },
    RecursiveTypeNotSupported {
        type_name: String,
        span: SourceSpan,
    },
}

impl AfidlError {
    /// Stable category identifier; referenced by the negative corpus manifest.
    pub fn kind(&self) -> &'static str {
        match self {
            AfidlError::Lex { .. } => kind::LEX,
            AfidlError::Parse { .. } => kind::PARSE,
            AfidlError::InvalidIdentifier { .. } => kind::INVALID_IDENTIFIER,
            AfidlError::InvalidLanguageVersion { .. } => kind::INVALID_LANGUAGE_VERSION,
            AfidlError::InvalidSpecificationId { .. } => kind::INVALID_SPECIFICATION_ID,
            AfidlError::DuplicateSchemaDeclaration { .. } => kind::DUPLICATE_SCHEMA_DECLARATION,
            AfidlError::DuplicateTypeDefinition { .. } => kind::DUPLICATE_TYPE_DEFINITION,
            AfidlError::InvalidFieldNumber { .. } => kind::INVALID_FIELD_NUMBER,
            AfidlError::DuplicateFieldNumber { .. } => kind::DUPLICATE_FIELD_NUMBER,
            AfidlError::DuplicateFieldName { .. } => kind::DUPLICATE_FIELD_NAME,
            AfidlError::InvalidCaseNumber { .. } => kind::INVALID_CASE_NUMBER,
            AfidlError::DuplicateCaseNumber { .. } => kind::DUPLICATE_CASE_NUMBER,
            AfidlError::DuplicateCaseName { .. } => kind::DUPLICATE_CASE_NAME,
            AfidlError::InvalidEnumNumber { .. } => kind::INVALID_ENUM_NUMBER,
            AfidlError::DuplicateEnumNumber { .. } => kind::DUPLICATE_ENUM_NUMBER,
            AfidlError::DuplicateEnumName { .. } => kind::DUPLICATE_ENUM_NAME,
            AfidlError::EmptyVariant { .. } => kind::EMPTY_VARIANT,
            AfidlError::EmptyEnum { .. } => kind::EMPTY_ENUM,
            AfidlError::UnknownTypeReference { .. } => kind::UNKNOWN_TYPE_REFERENCE,
            AfidlError::AliasCycle { .. } => kind::ALIAS_CYCLE,
            AfidlError::RecursiveTypeNotSupported { .. } => kind::RECURSIVE_TYPE_NOT_SUPPORTED,
        }
    }

    /// The source span of the offending construct, when available.
    pub fn span(&self) -> Option<SourceSpan> {
        match self {
            AfidlError::Lex { span, .. }
            | AfidlError::Parse { span, .. }
            | AfidlError::InvalidIdentifier { span, .. }
            | AfidlError::InvalidLanguageVersion { span, .. }
            | AfidlError::InvalidSpecificationId { span, .. }
            | AfidlError::DuplicateSchemaDeclaration { span }
            | AfidlError::DuplicateTypeDefinition { span, .. }
            | AfidlError::InvalidFieldNumber { span, .. }
            | AfidlError::DuplicateFieldNumber { span, .. }
            | AfidlError::DuplicateFieldName { span, .. }
            | AfidlError::InvalidCaseNumber { span, .. }
            | AfidlError::DuplicateCaseNumber { span, .. }
            | AfidlError::DuplicateCaseName { span, .. }
            | AfidlError::InvalidEnumNumber { span, .. }
            | AfidlError::DuplicateEnumNumber { span, .. }
            | AfidlError::DuplicateEnumName { span, .. }
            | AfidlError::EmptyVariant { span, .. }
            | AfidlError::EmptyEnum { span, .. }
            | AfidlError::UnknownTypeReference { span, .. }
            | AfidlError::AliasCycle { span, .. }
            | AfidlError::RecursiveTypeNotSupported { span, .. } => Some(*span),
        }
    }
}

impl fmt::Display for AfidlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AfidlError::Lex { detail, span } => write!(f, "lex error at {span}: {detail}"),
            AfidlError::Parse { detail, span } => write!(f, "parse error at {span}: {detail}"),
            AfidlError::InvalidIdentifier { detail, span } => {
                write!(f, "invalid identifier at {span}: {detail}")
            }
            AfidlError::InvalidLanguageVersion { found, span } => {
                write!(
                    f,
                    "unsupported AFIDL language version {found} at {span}; AFIDL1 accepts only 1"
                )
            }
            AfidlError::InvalidSpecificationId { detail, span } => {
                write!(f, "invalid schema specification id at {span}: {detail}")
            }
            AfidlError::DuplicateSchemaDeclaration { span } => {
                write!(
                    f,
                    "duplicate schema declaration at {span}; a module declares exactly one schema"
                )
            }
            AfidlError::DuplicateTypeDefinition { name, span } => {
                write!(f, "duplicate type definition {name:?} at {span}")
            }
            AfidlError::InvalidFieldNumber {
                no,
                type_name,
                span,
            } => {
                write!(
                    f,
                    "invalid field number {no} in record {type_name} at {span}; field numbers start at 1"
                )
            }
            AfidlError::DuplicateFieldNumber {
                no,
                type_name,
                span,
            } => {
                write!(
                    f,
                    "duplicate field number {no} in record {type_name} at {span}"
                )
            }
            AfidlError::DuplicateFieldName {
                name,
                type_name,
                span,
            } => {
                write!(
                    f,
                    "duplicate field name {name:?} in record {type_name} at {span}"
                )
            }
            AfidlError::InvalidCaseNumber {
                no,
                type_name,
                span,
            } => {
                write!(
                    f,
                    "invalid case number {no} in variant {type_name} at {span}; case numbers start at 1"
                )
            }
            AfidlError::DuplicateCaseNumber {
                no,
                type_name,
                span,
            } => {
                write!(
                    f,
                    "duplicate case number {no} in variant {type_name} at {span}"
                )
            }
            AfidlError::DuplicateCaseName {
                name,
                type_name,
                span,
            } => {
                write!(
                    f,
                    "duplicate case name {name:?} in variant {type_name} at {span}"
                )
            }
            AfidlError::InvalidEnumNumber {
                no,
                type_name,
                span,
            } => {
                write!(
                    f,
                    "invalid enum value number {no} in enum {type_name} at {span}; value numbers start at 1"
                )
            }
            AfidlError::DuplicateEnumNumber {
                no,
                type_name,
                span,
            } => {
                write!(
                    f,
                    "duplicate enum value number {no} in enum {type_name} at {span}"
                )
            }
            AfidlError::DuplicateEnumName {
                name,
                type_name,
                span,
            } => {
                write!(
                    f,
                    "duplicate enum value name {name:?} in enum {type_name} at {span}"
                )
            }
            AfidlError::EmptyVariant { name, span } => {
                write!(f, "variant {name} has no cases at {span}")
            }
            AfidlError::EmptyEnum { name, span } => {
                write!(f, "enum {name} has no values at {span}")
            }
            AfidlError::UnknownTypeReference { name, span } => {
                write!(f, "unknown type reference {name:?} at {span}")
            }
            AfidlError::AliasCycle { cycle, span } => {
                write!(f, "alias cycle {} at {span}", cycle.join(" -> "))
            }
            AfidlError::RecursiveTypeNotSupported { type_name, span } => {
                write!(
                    f,
                    "recursive type reference reachable from {type_name:?} at {span}; \
                     AFIDL1 core rejects direct and indirect structural recursion \
                     (Phase 2A scoped restriction)"
                )
            }
        }
    }
}

impl std::error::Error for AfidlError {}
