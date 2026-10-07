//! Structured errors of the specification registry validator.
//!
//! Every variant carries a stable [`SpecError::kind`] string; the frozen
//! spec-registry conformance corpus asserts on kinds, never on prose.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecError {
    Io {
        path: String,
        detail: String,
    },
    Json {
        path: String,
        detail: String,
    },
    CanonicalForm {
        path: String,
        detail: String,
    },
    InvalidSpecificationId {
        input: String,
    },
    DuplicateSpecificationId {
        specification_id: String,
    },
    UnknownSpecificationKind {
        value: String,
    },
    UnknownSpecificationStatus {
        value: String,
    },
    InvalidSourcePath {
        path: String,
        detail: String,
    },
    SourceNotFound {
        path: String,
    },
    ContentDigestMismatch {
        path: String,
        expected: String,
        actual: String,
    },
    InvalidFileDigest {
        path: String,
        detail: String,
    },
    ProvisionalFrozenSource {
        path: String,
    },
    InvalidRuleId {
        rule_id: String,
        detail: String,
    },
    DuplicateRuleId {
        rule_id: String,
    },
    UnknownRuleArea {
        area: String,
    },
    InvalidRequirementLevel {
        value: String,
    },
    AnchorNotFound {
        path: String,
        anchor: String,
    },
    DuplicateAnchorBinding {
        path: String,
        anchor: String,
    },
    SourceSpecificationMissing {
        path: String,
    },
    UnknownConformanceCase {
        case_id: String,
    },
    MissingCoverage {
        rule_id: String,
    },
    IntentionallyUncoveredMustRule {
        rule_id: String,
    },
    CoverageSummaryMismatch {
        detail: String,
    },
    InvalidAbiAssignment {
        detail: String,
    },
    DuplicateAbiAssignment {
        detail: String,
    },
    AbiNumberReassigned {
        detail: String,
    },
    AbiNameRebound {
        detail: String,
    },
    AbiRetiredNumberInUse {
        detail: String,
    },
    AbiReservedNumberInUse {
        detail: String,
    },
    AbiUnregisteredAssignment {
        detail: String,
    },
    AbiActiveAssignmentMissing {
        detail: String,
    },
    TableDrift {
        table: String,
        detail: String,
    },
    SchemaCompile {
        path: String,
        detail: String,
    },
    SchemaDigestInvalid {
        detail: String,
    },
}

impl SpecError {
    /// Stable error category. The spec-registry corpus freezes expectations
    /// against these strings.
    pub fn kind(&self) -> &'static str {
        match self {
            SpecError::Io { .. } => "io",
            SpecError::Json { .. } => "json",
            SpecError::CanonicalForm { .. } => "canonical_form",
            SpecError::InvalidSpecificationId { .. } => "invalid_specification_id",
            SpecError::DuplicateSpecificationId { .. } => "duplicate_specification_id",
            SpecError::UnknownSpecificationKind { .. } => "unknown_specification_kind",
            SpecError::UnknownSpecificationStatus { .. } => "unknown_specification_status",
            SpecError::InvalidSourcePath { .. } => "invalid_source_path",
            SpecError::SourceNotFound { .. } => "source_not_found",
            SpecError::ContentDigestMismatch { .. } => "content_digest_mismatch",
            SpecError::InvalidFileDigest { .. } => "invalid_file_digest",
            SpecError::ProvisionalFrozenSource { .. } => "provisional_frozen_source",
            SpecError::InvalidRuleId { .. } => "invalid_rule_id",
            SpecError::DuplicateRuleId { .. } => "duplicate_rule_id",
            SpecError::UnknownRuleArea { .. } => "unknown_rule_area",
            SpecError::InvalidRequirementLevel { .. } => "invalid_requirement_level",
            SpecError::AnchorNotFound { .. } => "anchor_not_found",
            SpecError::DuplicateAnchorBinding { .. } => "duplicate_anchor_binding",
            SpecError::SourceSpecificationMissing { .. } => "source_specification_missing",
            SpecError::UnknownConformanceCase { .. } => "unknown_conformance_case",
            SpecError::MissingCoverage { .. } => "missing_coverage",
            SpecError::IntentionallyUncoveredMustRule { .. } => "intentionally_uncovered_must_rule",
            SpecError::CoverageSummaryMismatch { .. } => "coverage_summary_mismatch",
            SpecError::InvalidAbiAssignment { .. } => "invalid_abi_assignment",
            SpecError::DuplicateAbiAssignment { .. } => "duplicate_abi_assignment",
            SpecError::AbiNumberReassigned { .. } => "abi_number_reassigned",
            SpecError::AbiNameRebound { .. } => "abi_name_rebound",
            SpecError::AbiRetiredNumberInUse { .. } => "abi_retired_number_in_use",
            SpecError::AbiReservedNumberInUse { .. } => "abi_reserved_number_in_use",
            SpecError::AbiUnregisteredAssignment { .. } => "abi_unregistered_assignment",
            SpecError::AbiActiveAssignmentMissing { .. } => "abi_active_assignment_missing",
            SpecError::TableDrift { .. } => "table_drift",
            SpecError::SchemaCompile { .. } => "schema_compile",
            SpecError::SchemaDigestInvalid { .. } => "schema_digest_invalid",
        }
    }
}

impl fmt::Display for SpecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpecError::Io { path, detail } => write!(f, "{path}: {detail}"),
            SpecError::Json { path, detail } => write!(f, "{path}: invalid JSON: {detail}"),
            SpecError::CanonicalForm { path, detail } => {
                write!(
                    f,
                    "{path}: registry file is not in canonical form: {detail}"
                )
            }
            SpecError::InvalidSpecificationId { input } => {
                write!(f, "invalid SpecificationId {input:?}")
            }
            SpecError::DuplicateSpecificationId { specification_id } => {
                write!(f, "duplicate SpecificationId {specification_id:?}")
            }
            SpecError::UnknownSpecificationKind { value } => {
                write!(f, "unknown specification kind {value:?}")
            }
            SpecError::UnknownSpecificationStatus { value } => {
                write!(f, "unknown specification status {value:?}")
            }
            SpecError::InvalidSourcePath { path, detail } => {
                write!(f, "invalid normative source path {path:?}: {detail}")
            }
            SpecError::SourceNotFound { path } => write!(f, "normative source not found: {path}"),
            SpecError::ContentDigestMismatch {
                path,
                expected,
                actual,
            } => write!(
                f,
                "content digest mismatch for {path}: registry expects {expected}, file hashes to {actual}"
            ),
            SpecError::InvalidFileDigest { path, detail } => {
                write!(f, "invalid artifact file digest for {path}: {detail}")
            }
            SpecError::ProvisionalFrozenSource { path } => {
                write!(
                    f,
                    "frozen specification source still carries a provisional marker: {path}"
                )
            }
            SpecError::InvalidRuleId { rule_id, detail } => {
                write!(f, "invalid RuleId {rule_id:?}: {detail}")
            }
            SpecError::DuplicateRuleId { rule_id } => write!(f, "duplicate RuleId {rule_id:?}"),
            SpecError::UnknownRuleArea { area } => write!(f, "unknown rule area {area:?}"),
            SpecError::InvalidRequirementLevel { value } => {
                write!(f, "invalid requirement level {value:?}")
            }
            SpecError::AnchorNotFound { path, anchor } => {
                write!(f, "anchor {anchor:?} not found in {path}")
            }
            SpecError::DuplicateAnchorBinding { path, anchor } => {
                write!(
                    f,
                    "anchor {anchor:?} in {path} is bound by more than one rule"
                )
            }
            SpecError::SourceSpecificationMissing { path } => {
                write!(
                    f,
                    "rule source {path} is not the normative source of any registered specification"
                )
            }
            SpecError::UnknownConformanceCase { case_id } => {
                write!(f, "conformance reference {case_id:?} does not resolve")
            }
            SpecError::MissingCoverage { rule_id } => {
                write!(
                    f,
                    "rule {rule_id} has neither conformance refs nor an intentionally-uncovered reason"
                )
            }
            SpecError::IntentionallyUncoveredMustRule { rule_id } => write!(
                f,
                "MUST-level rule {rule_id} is intentionally uncovered but absent from the frozen coverage summary"
            ),
            SpecError::CoverageSummaryMismatch { detail } => {
                write!(f, "coverage summary mismatch: {detail}")
            }
            SpecError::InvalidAbiAssignment { detail } => {
                write!(f, "invalid ABI assignment: {detail}")
            }
            SpecError::DuplicateAbiAssignment { detail } => {
                write!(f, "duplicate ABI assignment: {detail}")
            }
            SpecError::AbiNumberReassigned { detail } => {
                write!(f, "ABI number reassigned to a new meaning: {detail}")
            }
            SpecError::AbiNameRebound { detail } => {
                write!(f, "ABI symbolic name moved to a different number: {detail}")
            }
            SpecError::AbiRetiredNumberInUse { detail } => {
                write!(f, "retired ABI number still in use: {detail}")
            }
            SpecError::AbiReservedNumberInUse { detail } => {
                write!(f, "reserved ABI number used by the schema: {detail}")
            }
            SpecError::AbiUnregisteredAssignment { detail } => {
                write!(f, "unregistered ABI assignment: {detail}")
            }
            SpecError::AbiActiveAssignmentMissing { detail } => {
                write!(
                    f,
                    "active ABI assignment has no schema counterpart: {detail}"
                )
            }
            SpecError::TableDrift { table, detail } => {
                write!(f, "closed table drift in {table}: {detail}")
            }
            SpecError::SchemaCompile { path, detail } => {
                write!(f, "{path}: schema compilation failed: {detail}")
            }
            SpecError::SchemaDigestInvalid { detail } => {
                write!(f, "invalid schema digest: {detail}")
            }
        }
    }
}

impl std::error::Error for SpecError {}
