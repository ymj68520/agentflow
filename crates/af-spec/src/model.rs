//! Typed models of the registry JSON artifacts.
//!
//! The AFIDL module `spec/idl/specification-registry.afidl`
//! (`agentflow.spec.registry@1`) is the normative shape authority; these Rust
//! models parse the JSON *instances* and the validator enforces the formats
//! the schema layer leaves open (RuleId syntax, digests, enums-as-strings).

use serde::Deserialize;

/// `SpecificationKind`, closed per `agentflow.spec.registry@1`.
pub const SPECIFICATION_KINDS: [&str; 7] = [
    "encoding",
    "digest",
    "digest_domain_registry",
    "schema_language",
    "schema",
    "closed_table",
    "registry",
];

/// `SpecificationStatus`, closed per `agentflow.spec.registry@1`.
pub const SPECIFICATION_STATUSES: [&str; 4] = ["draft", "frozen", "deprecated", "withdrawn"];

/// `RequirementLevel`, closed per `agentflow.spec.registry@1`.
pub const REQUIREMENT_LEVELS: [&str; 5] = ["must", "must_not", "should", "should_not", "may"];

/// `AbiNumberKind`, closed per `agentflow.spec.registry@1`.
pub const ABI_NUMBER_KINDS: [&str; 3] = ["field", "variant_case", "enum_value"];

/// `AbiNumberStatus`, closed per `agentflow.spec.registry@1`.
pub const ABI_NUMBER_STATUSES: [&str; 3] = ["active", "reserved", "retired"];

// ---------------------------------------------------------------------------
// specifications.json
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileDigestJson {
    pub algorithm: String,
    pub hex: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRefJson {
    pub logical_path: String,
    pub content_digest: FileDigestJson,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DigestRefJson {
    pub alg: String,
    pub domain: String,
    pub hex: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecificationEntryJson {
    pub specification_id: String,
    pub kind: String,
    pub status: String,
    pub normative_source: SourceRefJson,
    #[serde(default)]
    pub schema_digest: Option<DigestRefJson>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecificationsFile {
    pub registry_revision: u64,
    pub specifications: Vec<SpecificationEntryJson>,
}

// ---------------------------------------------------------------------------
// rules.json
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleJson {
    pub rule_id: String,
    pub area: String,
    pub normative_source: String,
    pub source_anchor: String,
    pub requirement_level: String,
    pub short_statement: String,
    #[serde(default)]
    pub conformance_refs: Vec<String>,
    #[serde(default)]
    pub intentionally_uncovered_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RulesFile {
    pub registry_revision: u64,
    pub rules: Vec<RuleJson>,
}

// ---------------------------------------------------------------------------
// abi-numbers.json
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AbiAssignmentJson {
    pub schema_family: String,
    pub type_name: String,
    pub number_kind: String,
    pub number: u32,
    pub symbolic_name: String,
    pub status: String,
    #[serde(default)]
    pub first_seen_schema_digest: Option<DigestRefJson>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AbiFile {
    pub registry_revision: u64,
    pub assignments: Vec<AbiAssignmentJson>,
}

// ---------------------------------------------------------------------------
// spec/tables/rule-areas.json
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleAreaJson {
    pub area: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleAreasFile {
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
    pub areas: Vec<RuleAreaJson>,
}

// ---------------------------------------------------------------------------
// spec/conformance/suites.json
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuiteJson {
    pub suite_id: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuitesFile {
    #[serde(default)]
    pub kind: Option<String>,
    pub suite_revision: u64,
    pub suites: Vec<SuiteJson>,
}

// ---------------------------------------------------------------------------
// coverage-summary.json
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageEntryJson {
    pub rule_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageSummaryFile {
    pub coverage_revision: u64,
    pub intentionally_uncovered: Vec<CoverageEntryJson>,
    pub missing_coverage: Vec<CoverageEntryJson>,
}
