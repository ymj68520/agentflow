//! The cross-reference validator (`spec-check`).
//!
//! Validates the registry artifacts against each other, against the
//! normative sources they reference, against the compiled AFIDL modules and
//! against the frozen conformance corpus. It only checks — it never writes
//! or "fixes" anything.

use crate::abi::{self, SchemaBinding};
use crate::anchors;
use crate::error::SpecError;
use crate::fs;
use crate::model;
use crate::tables;
use af_idl::SpecificationId;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// Which registry artifacts live where. Two layouts exist: the real
/// repository and a self-contained spec-registry conformance case.
#[derive(Debug, Clone)]
pub struct Layout {
    /// `specifications.json`, relative to the root.
    pub specifications: &'static str,
    /// `rules.json`, relative to the root.
    pub rules: &'static str,
    /// `abi-numbers.json`, relative to the root.
    pub abi: &'static str,
    /// The rule-area closed table, relative to the root.
    pub rule_areas: &'static str,
    /// The conformance suite table; empty string disables it.
    pub suites: &'static str,
    /// The frozen coverage summary; empty string disables it.
    pub coverage_summary: &'static str,
    /// Directory scanned for conformance case ids; empty string disables it.
    pub conformance_root: &'static str,
    /// Explicit `.afidl` files to compile; empty slice triggers a recursive
    /// walk of [`Layout::idl_walk_dir`] instead.
    pub idl_files: &'static [&'static str],
    /// Directory walked for `.afidl` files when [`Layout::idl_files`] is empty.
    pub idl_walk_dir: &'static str,
    /// Closed-table drift checks against the implementation enums
    /// (hash domains, AFIDL primitives). Only meaningful in repo mode.
    pub drift_checks: bool,
}

impl Layout {
    /// The real repository layout.
    pub fn repo() -> Self {
        Layout {
            specifications: "spec/registry/specifications.json",
            rules: "spec/registry/rules.json",
            abi: "spec/registry/abi-numbers.json",
            rule_areas: "spec/tables/rule-areas.json",
            suites: "spec/conformance/suites.json",
            coverage_summary: "spec/registry/coverage-summary.json",
            conformance_root: "spec/conformance",
            idl_files: &[],
            idl_walk_dir: "spec/idl",
            drift_checks: true,
        }
    }

    /// A self-contained `spec/conformance/spec-registry/` case layout.
    pub fn corpus_case() -> Self {
        Layout {
            specifications: "specifications.json",
            rules: "rules.json",
            abi: "abi-numbers.json",
            rule_areas: "rule-areas.json",
            suites: "",
            coverage_summary: "",
            conformance_root: "conformance",
            idl_files: &["registry.afidl"],
            idl_walk_dir: "",
            drift_checks: false,
        }
    }
}

/// Outcome statistics and findings of one validator run.
#[derive(Debug, Default, Clone)]
pub struct Report {
    pub errors: Vec<SpecError>,
    pub warnings: Vec<String>,
    pub specifications: usize,
    pub rules: usize,
    pub covered: usize,
    pub intentionally_uncovered: usize,
    pub missing_coverage: usize,
    pub abi_active: usize,
    pub abi_retired: usize,
    pub abi_reserved: usize,
    pub modules_checked: usize,
}

impl Report {
    /// `true` when no validation error was found.
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

/// A single validation configuration over one root directory.
#[derive(Debug, Clone)]
pub struct SpecCheck {
    root: PathBuf,
    layout: Layout,
}

impl SpecCheck {
    /// Validates the real repository rooted at `root`.
    pub fn repo(root: impl Into<PathBuf>) -> Self {
        SpecCheck {
            root: root.into(),
            layout: Layout::repo(),
        }
    }

    /// Validates with an explicit layout (conformance corpus cases).
    pub fn with_layout(root: impl Into<PathBuf>, layout: Layout) -> Self {
        SpecCheck {
            root: root.into(),
            layout,
        }
    }

    /// Runs every check, collecting all findings (no fail-fast).
    pub fn run(&self) -> Report {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut report = Report::default();

        // 1. Rule-area closed table.
        let areas = match tables::load_rule_areas(&self.root, self.layout.rule_areas) {
            Ok(loaded) => Some(loaded.areas),
            Err(error) => {
                errors.push(error);
                None
            }
        };
        let areas = areas.unwrap_or_default();

        // 2. Specification registry.
        let mut registered_sources: BTreeSet<String> = BTreeSet::new();
        if let Some(entries) = self.load_specifications(&mut errors) {
            report.specifications = entries.len();
            for entry in &entries {
                registered_sources.insert(entry.normative_source.logical_path.clone());
            }
        }

        // 3. Conformance suite ids + fixture case universe.
        let suite_ids = self.load_suite_ids(&mut errors);
        let case_universe = self.scan_conformance_cases();
        let mut full_universe = case_universe;
        full_universe.extend(suite_ids.iter().cloned());

        // 4. Rule registry + coverage.
        let mut uncovered: BTreeMap<String, String> = BTreeMap::new();
        let mut rule_levels: BTreeMap<String, String> = BTreeMap::new();
        if let Some(rules) = areas_checked(&areas, &mut errors) {
            self.load_rules(
                &rules,
                &registered_sources,
                &full_universe,
                &mut uncovered,
                &mut rule_levels,
                &mut errors,
            );
        }
        report.rules = rule_levels.len();
        report.intentionally_uncovered = uncovered.len();
        report.missing_coverage = errors
            .iter()
            .filter(|e| matches!(e, SpecError::MissingCoverage { .. }))
            .count();

        // 5. ABI registry.
        let assignments = self.load_abi(&mut errors);
        let rows: &[model::AbiAssignmentJson] = assignments.as_deref().unwrap_or(&[]);
        report.abi_active = rows.iter().filter(|a| a.status == "active").count();
        report.abi_retired = rows.iter().filter(|a| a.status == "retired").count();
        report.abi_reserved = rows.iter().filter(|a| a.status == "reserved").count();

        // 6. Compile AFIDL modules and cross-check against the ABI registry.
        let idl_files: Vec<String> = if self.layout.idl_files.is_empty() {
            fs::collect_afidl_files(&self.root, self.layout.idl_walk_dir)
        } else {
            self.layout
                .idl_files
                .iter()
                .map(ToString::to_string)
                .collect()
        };
        let mut families: BTreeMap<String, Vec<SchemaBinding>> = BTreeMap::new();
        for relative in &idl_files {
            match std::fs::read_to_string(self.root.join(relative)) {
                Ok(source) => match af_idl::compile(&source) {
                    Ok(compiled) => {
                        let family = compiled.specification.canonical();
                        let bindings = abi::schema_bindings(&compiled);
                        families.entry(family).or_default().extend(bindings);
                        report.modules_checked += 1;
                    }
                    Err(error) => errors.push(SpecError::SchemaCompile {
                        path: relative.clone(),
                        detail: error.to_string(),
                    }),
                },
                Err(error) => errors.push(SpecError::Io {
                    path: relative.clone(),
                    detail: error.to_string(),
                }),
            }
        }
        if !rows.is_empty() {
            let mut all_families: BTreeSet<String> = families.keys().cloned().collect();
            all_families.extend(rows.iter().map(|row| row.schema_family.clone()));
            for family in &all_families {
                let bindings: &[SchemaBinding] = families.get(family).map_or(&[], Vec::as_slice);
                for error in abi::check_current_schema_against_abi_registry(family, bindings, rows)
                {
                    errors.push(error);
                }
            }
        }

        // 7. Closed-table drift checks (repo mode).
        if self.layout.drift_checks {
            if let Err(error) =
                tables::check_hash_domains(&self.root, "spec/tables/hash-domains.json")
            {
                errors.push(error);
            }
            if let Err(error) =
                tables::check_afidl_primitives(&self.root, "spec/tables/afidl-primitives.json")
            {
                errors.push(error);
            }
        }

        // 8. Frozen coverage summary (repo mode).
        if self.layout.coverage_summary.is_empty() {
            for (rule_id, level) in &rule_levels {
                if uncovered.contains_key(rule_id) && (*level == "must" || *level == "must_not") {
                    warnings.push(format!(
                        "MUST-level rule {rule_id} is intentionally uncovered \
                         (no coverage summary configured to list the exception)"
                    ));
                }
            }
        } else {
            self.check_coverage_summary(&uncovered, &rule_levels, &mut errors);
        }

        report.covered = report.rules - report.intentionally_uncovered - report.missing_coverage;
        report.errors = errors;
        report.warnings = warnings;
        report
    }

    fn load_specifications(
        &self,
        errors: &mut Vec<SpecError>,
    ) -> Option<Vec<model::SpecificationEntryJson>> {
        let value = match fs::load_canonical_json(&self.root, self.layout.specifications) {
            Ok(value) => value,
            Err(error) => {
                errors.push(error);
                return None;
            }
        };
        let file: model::SpecificationsFile =
            match fs::from_value(value, self.layout.specifications) {
                Ok(file) => file,
                Err(error) => {
                    errors.push(error);
                    return None;
                }
            };
        let mut seen: BTreeSet<String> = BTreeSet::new();
        let mut previous: Option<String> = None;
        for entry in &file.specifications {
            if let Some(prev) = &previous {
                if prev.as_bytes() > entry.specification_id.as_bytes() {
                    errors.push(SpecError::CanonicalForm {
                        path: self.layout.specifications.to_string(),
                        detail: "specifications array is not sorted by specification_id".into(),
                    });
                }
            }
            previous = Some(entry.specification_id.clone());
            if !seen.insert(entry.specification_id.clone()) {
                errors.push(SpecError::DuplicateSpecificationId {
                    specification_id: entry.specification_id.clone(),
                });
            }
            if SpecificationId::parse(&entry.specification_id).is_err() {
                errors.push(SpecError::InvalidSpecificationId {
                    input: entry.specification_id.clone(),
                });
            }
            if !model::SPECIFICATION_KINDS.contains(&entry.kind.as_str()) {
                errors.push(SpecError::UnknownSpecificationKind {
                    value: entry.kind.clone(),
                });
            }
            if !model::SPECIFICATION_STATUSES.contains(&entry.status.as_str()) {
                errors.push(SpecError::UnknownSpecificationStatus {
                    value: entry.status.clone(),
                });
            }
            self.check_normative_source(entry, errors);
            if let Some(digest) = &entry.schema_digest {
                validate_type_schema_digest(digest, errors);
            }
        }
        Some(file.specifications)
    }

    fn check_normative_source(
        &self,
        entry: &model::SpecificationEntryJson,
        errors: &mut Vec<SpecError>,
    ) {
        let logical = entry.normative_source.logical_path.as_str();
        let path = match fs::safe_join(&self.root, logical) {
            Ok(path) => path,
            Err(error) => {
                errors.push(error);
                return;
            }
        };
        if !path.is_file() {
            errors.push(SpecError::SourceNotFound {
                path: logical.to_string(),
            });
            return;
        }
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) => {
                errors.push(SpecError::Io {
                    path: logical.to_string(),
                    detail: error.to_string(),
                });
                return;
            }
        };
        let file_digest = &entry.normative_source.content_digest;
        if file_digest.algorithm != "sha256" {
            errors.push(SpecError::InvalidFileDigest {
                path: logical.to_string(),
                detail: format!("algorithm {:?} is not sha256", file_digest.algorithm),
            });
        }
        if !is_lower_hex_64(&file_digest.hex) {
            errors.push(SpecError::InvalidFileDigest {
                path: logical.to_string(),
                detail: "hex must be 64 lowercase hexadecimal characters".into(),
            });
        }
        let actual = fs::sha256_hex(&bytes);
        if actual != file_digest.hex {
            errors.push(SpecError::ContentDigestMismatch {
                path: logical.to_string(),
                expected: file_digest.hex.clone(),
                actual,
            });
        }
        if entry.status == "frozen" {
            let text = String::from_utf8_lossy(&bytes);
            if anchors::has_provisional_marker(&text) {
                errors.push(SpecError::ProvisionalFrozenSource {
                    path: logical.to_string(),
                });
            }
        }
    }

    fn load_suite_ids(&self, errors: &mut Vec<SpecError>) -> BTreeSet<String> {
        let mut ids = BTreeSet::new();
        if self.layout.suites.is_empty() {
            return ids;
        }
        let value = match fs::load_canonical_json(&self.root, self.layout.suites) {
            Ok(value) => value,
            Err(error) => {
                errors.push(error);
                return ids;
            }
        };
        let file: model::SuitesFile = match fs::from_value(value, self.layout.suites) {
            Ok(file) => file,
            Err(error) => {
                errors.push(error);
                return ids;
            }
        };
        let mut previous: Option<&str> = None;
        for suite in &file.suites {
            if !suite.suite_id.starts_with("SUITE-") {
                errors.push(SpecError::UnknownConformanceCase {
                    case_id: suite.suite_id.clone(),
                });
            }
            if !ids.insert(suite.suite_id.clone()) {
                errors.push(SpecError::DuplicateRuleId {
                    rule_id: suite.suite_id.clone(),
                });
            }
            if previous.is_some_and(|prev| prev >= suite.suite_id.as_str()) {
                errors.push(SpecError::CanonicalForm {
                    path: self.layout.suites.to_string(),
                    detail: "suites array is not sorted by suite_id".into(),
                });
            }
            previous = Some(suite.suite_id.as_str());
        }
        ids
    }

    /// Scans the conformance root for stable case identities: `case_id`
    /// strings anywhere in corpus JSON, plus manifest-listed `.afidl` file
    /// stems for file-based corpora (objects carrying both `file` and
    /// `error_kind`).
    fn scan_conformance_cases(&self) -> BTreeSet<String> {
        let mut universe = BTreeSet::new();
        if self.layout.conformance_root.is_empty() {
            return universe;
        }
        for relative in fs::collect_json_files(&self.root, self.layout.conformance_root) {
            if let Ok(value) = fs::load_plain_json(&self.root, &relative) {
                collect_case_ids(&value, &mut universe);
            }
        }
        universe
    }

    fn load_rules(
        &self,
        areas: &BTreeSet<String>,
        registered_sources: &BTreeSet<String>,
        universe: &BTreeSet<String>,
        uncovered: &mut BTreeMap<String, String>,
        rule_levels: &mut BTreeMap<String, String>,
        errors: &mut Vec<SpecError>,
    ) -> Option<Vec<model::RuleJson>> {
        let value = match fs::load_canonical_json(&self.root, self.layout.rules) {
            Ok(value) => value,
            Err(error) => {
                errors.push(error);
                return None;
            }
        };
        let file: model::RulesFile = match fs::from_value(value, self.layout.rules) {
            Ok(file) => file,
            Err(error) => {
                errors.push(error);
                return None;
            }
        };
        let mut seen: BTreeSet<String> = BTreeSet::new();
        let mut bound_anchors: BTreeSet<(String, String)> = BTreeSet::new();
        let mut previous: Option<&str> = None;
        for rule in &file.rules {
            if let Some(prev) = previous {
                if prev > rule.rule_id.as_str() {
                    errors.push(SpecError::CanonicalForm {
                        path: self.layout.rules.to_string(),
                        detail: "rules array is not sorted by rule_id".into(),
                    });
                }
            }
            previous = Some(rule.rule_id.as_str());
            rule_levels.insert(rule.rule_id.clone(), rule.requirement_level.clone());
            if !seen.insert(rule.rule_id.clone()) {
                errors.push(SpecError::DuplicateRuleId {
                    rule_id: rule.rule_id.clone(),
                });
            }
            if let Err(error) = anchors::parse_rule_id(&rule.rule_id, areas) {
                errors.push(error);
            }
            // The descriptor's area field must be a table member and agree
            // with the RuleId's own area segment.
            let id_area = rule
                .rule_id
                .strip_prefix("AF-")
                .and_then(|rest| rest.rsplit_once('-'))
                .map(|(area, _)| area);
            if !areas.contains(rule.area.as_str()) {
                errors.push(SpecError::UnknownRuleArea {
                    area: rule.area.clone(),
                });
            }
            if id_area.is_some_and(|area| area != rule.area) {
                errors.push(SpecError::InvalidRuleId {
                    rule_id: rule.rule_id.clone(),
                    detail: format!(
                        "area field {:?} disagrees with the RuleId area segment",
                        rule.area
                    ),
                });
            }
            if !model::REQUIREMENT_LEVELS.contains(&rule.requirement_level.as_str()) {
                errors.push(SpecError::InvalidRequirementLevel {
                    value: rule.requirement_level.clone(),
                });
            }
            if !registered_sources.contains(rule.normative_source.as_str()) {
                errors.push(SpecError::SourceSpecificationMissing {
                    path: rule.normative_source.clone(),
                });
            }
            self.check_rule_anchor(rule, &mut bound_anchors, errors);
            for reference in &rule.conformance_refs {
                if !universe.contains(reference) {
                    errors.push(SpecError::UnknownConformanceCase {
                        case_id: reference.clone(),
                    });
                }
            }
            match (
                rule.conformance_refs.is_empty(),
                rule.intentionally_uncovered_reason
                    .as_deref()
                    .filter(|r| !r.is_empty()),
            ) {
                (false, _) => {}
                (true, Some(reason)) => {
                    uncovered.insert(rule.rule_id.clone(), reason.to_string());
                }
                (true, None) => {
                    errors.push(SpecError::MissingCoverage {
                        rule_id: rule.rule_id.clone(),
                    });
                }
            }
        }
        Some(file.rules)
    }

    fn check_rule_anchor(
        &self,
        rule: &model::RuleJson,
        bound: &mut BTreeSet<(String, String)>,
        errors: &mut Vec<SpecError>,
    ) {
        if !bound.insert((rule.normative_source.clone(), rule.source_anchor.clone())) {
            errors.push(SpecError::DuplicateAnchorBinding {
                path: rule.normative_source.clone(),
                anchor: rule.source_anchor.clone(),
            });
            return;
        }
        let path = match fs::safe_join(&self.root, &rule.normative_source) {
            Ok(path) => path,
            Err(error) => {
                errors.push(error);
                return;
            }
        };
        let Ok(text) = std::fs::read_to_string(&path) else {
            // Belt and braces: a rule source that cannot be read is an error
            // even if the corresponding specification entry was never reached.
            errors.push(SpecError::SourceNotFound {
                path: rule.normative_source.clone(),
            });
            return;
        };
        let is_afidl = std::path::Path::new(&rule.normative_source)
            .extension()
            .is_some_and(|ext| ext == "afidl");
        let exists = if is_afidl {
            anchors::afidl_anchor_exists(&text, &rule.source_anchor)
        } else {
            anchors::markdown_anchor_exists(&text, &rule.source_anchor)
        };
        if !exists {
            errors.push(SpecError::AnchorNotFound {
                path: rule.normative_source.clone(),
                anchor: rule.source_anchor.clone(),
            });
        }
    }

    fn load_abi(&self, errors: &mut Vec<SpecError>) -> Option<Vec<model::AbiAssignmentJson>> {
        let value = match fs::load_canonical_json(&self.root, self.layout.abi) {
            Ok(value) => value,
            Err(error) => {
                errors.push(error);
                return None;
            }
        };
        let file: model::AbiFile = match fs::from_value(value, self.layout.abi) {
            Ok(file) => file,
            Err(error) => {
                errors.push(error);
                return None;
            }
        };
        let mut tuples: BTreeSet<(String, String, String, u32)> = BTreeSet::new();
        let mut names: BTreeSet<(String, String, String, String)> = BTreeSet::new();
        let mut previous: Option<(String, String, String, u32)> = None;
        for row in &file.assignments {
            let invalid = |detail: String| SpecError::InvalidAbiAssignment { detail };
            if SpecificationId::parse(&row.schema_family).is_err() {
                errors.push(SpecError::InvalidSpecificationId {
                    input: row.schema_family.clone(),
                });
            }
            if !model::ABI_NUMBER_KINDS.contains(&row.number_kind.as_str()) {
                errors.push(invalid(format!(
                    "unknown number_kind {:?}",
                    row.number_kind
                )));
            }
            if !model::ABI_NUMBER_STATUSES.contains(&row.status.as_str()) {
                errors.push(invalid(format!("unknown status {:?}", row.status)));
            }
            if row.number == 0 {
                errors.push(invalid("number must be positive".to_string()));
            }
            if !anchors::is_lower_name(&row.symbolic_name) {
                errors.push(invalid(format!(
                    "symbolic_name {:?} must match the AFIDL lower-name grammar",
                    row.symbolic_name
                )));
            }
            if !anchors::is_type_name(&row.type_name) {
                errors.push(invalid(format!(
                    "type_name {:?} must match the AFIDL TYPE-NAME grammar",
                    row.type_name
                )));
            }
            if let Some(digest) = &row.first_seen_schema_digest {
                validate_type_schema_digest(digest, errors);
            }
            let tuple = (
                row.schema_family.clone(),
                row.type_name.clone(),
                row.number_kind.clone(),
                row.number,
            );
            if !tuples.insert(tuple.clone()) {
                errors.push(SpecError::DuplicateAbiAssignment {
                    detail: format!(
                        "{}/{}/{} #{}",
                        row.schema_family, row.type_name, row.number_kind, row.number
                    ),
                });
            }
            let name_key = (
                row.schema_family.clone(),
                row.type_name.clone(),
                row.number_kind.clone(),
                row.symbolic_name.clone(),
            );
            if !names.insert(name_key) {
                errors.push(SpecError::DuplicateAbiAssignment {
                    detail: format!(
                        "symbolic name {:?} bound twice in {}/{}/{}",
                        row.symbolic_name, row.schema_family, row.type_name, row.number_kind
                    ),
                });
            }
            let sort_key = (
                row.schema_family.clone(),
                row.type_name.clone(),
                row.number_kind.clone(),
                row.number,
            );
            if let Some(prev) = &previous {
                if prev >= &sort_key {
                    errors.push(SpecError::CanonicalForm {
                        path: self.layout.abi.to_string(),
                        detail: "assignments array is not sorted by \
                                 (schema_family, type_name, number_kind, number)"
                            .into(),
                    });
                }
            }
            previous = Some(sort_key);
        }
        Some(file.assignments)
    }

    fn check_coverage_summary(
        &self,
        uncovered: &BTreeMap<String, String>,
        rule_levels: &BTreeMap<String, String>,
        errors: &mut Vec<SpecError>,
    ) {
        let value = match fs::load_canonical_json(&self.root, self.layout.coverage_summary) {
            Ok(value) => value,
            Err(error) => {
                errors.push(error);
                return;
            }
        };
        let file: model::CoverageSummaryFile =
            match fs::from_value(value, self.layout.coverage_summary) {
                Ok(file) => file,
                Err(error) => {
                    errors.push(error);
                    return;
                }
            };
        if !file.missing_coverage.is_empty() {
            errors.push(SpecError::CoverageSummaryMismatch {
                detail: "summary lists missing coverage; missing coverage is always \
                         a validator error, never a frozen exception"
                    .into(),
            });
        }
        let listed: BTreeMap<String, String> = file
            .intentionally_uncovered
            .iter()
            .map(|entry| (entry.rule_id.clone(), entry.reason.clone()))
            .collect();
        if listed == *uncovered {
            return;
        }
        let mut must_violations = Vec::new();
        for rule_id in uncovered.keys() {
            let listed_contains = listed.contains_key(rule_id);
            let level = rule_levels.get(rule_id).map_or("", String::as_str);
            if !listed_contains && (level == "must" || level == "must_not") {
                must_violations.push(rule_id.clone());
            }
        }
        for rule_id in must_violations {
            errors.push(SpecError::IntentionallyUncoveredMustRule { rule_id });
        }
        errors.push(SpecError::CoverageSummaryMismatch {
            detail: format!(
                "computed intentionally-uncovered set {:?} != frozen summary {:?}",
                sorted_keys(uncovered),
                sorted_keys(&listed)
            ),
        });
    }
}

fn areas_checked(
    areas: &BTreeSet<String>,
    errors: &mut Vec<SpecError>,
) -> Option<BTreeSet<String>> {
    // The area table load error was already recorded; without a table no
    // rule-level checks can run.
    if areas.is_empty() {
        errors.push(SpecError::TableDrift {
            table: "rule-areas.json".to_string(),
            detail: "rule-area table is empty or failed to load".into(),
        });
        None
    } else {
        Some(areas.clone())
    }
}

fn sorted_keys(map: &BTreeMap<String, String>) -> Vec<String> {
    map.keys().cloned().collect()
}

fn is_lower_hex_64(text: &str) -> bool {
    text.len() == 64
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Validates an embedded `DigestRef` object: real HashV1 identity, domain
/// `type-schema`, strict lowercase 64-hex.
fn validate_type_schema_digest(digest: &model::DigestRefJson, errors: &mut Vec<SpecError>) {
    if digest.alg != "sha256" {
        errors.push(SpecError::SchemaDigestInvalid {
            detail: format!("alg {:?} is not sha256", digest.alg),
        });
    }
    let domain = af_core::DigestDomain::from_registry_name(&digest.domain);
    match domain {
        Ok(af_core::DigestDomain::TypeSchema) => {}
        Ok(_) => errors.push(SpecError::SchemaDigestInvalid {
            detail: format!(
                "domain {:?} is a HashV1 domain but not type-schema",
                digest.domain
            ),
        }),
        Err(_) => errors.push(SpecError::SchemaDigestInvalid {
            detail: format!("domain {:?} is not in the frozen registry", digest.domain),
        }),
    }
    if af_core::DigestRef::from_parts(
        af_core::DigestAlgorithm::Sha256,
        af_core::DigestDomain::TypeSchema,
        &digest.hex,
    )
    .is_err()
    {
        errors.push(SpecError::SchemaDigestInvalid {
            detail: "hex must be 64 lowercase hexadecimal characters".into(),
        });
    }
}

fn collect_case_ids(value: &serde_json::Value, out: &mut BTreeSet<String>) {
    match value {
        serde_json::Value::Object(map) => {
            let mut file_name: Option<String> = None;
            let mut has_error_kind = false;
            for (key, nested) in map {
                match key.as_str() {
                    "case_id" => {
                        if let Some(id) = nested.as_str() {
                            out.insert(id.to_string());
                        }
                    }
                    "file" => file_name = nested.as_str().map(ToString::to_string),
                    "error_kind" => has_error_kind = nested.is_string(),
                    _ => {}
                }
                collect_case_ids(nested, out);
            }
            if let (Some(file), true) = (file_name, has_error_kind) {
                if let Some(stem) = file.strip_suffix(".afidl") {
                    out.insert(stem.to_string());
                }
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_case_ids(item, out);
            }
        }
        _ => {}
    }
}
