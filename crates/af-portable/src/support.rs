use crate::error::{PortableProofError, ValidationStage};
use crate::schema::registered_schema_refs;
use crate::validation::ContentValidatedPortableBundle;
use af_core::Value;
use af_spec::model::{RuleJson, RulesFile, SpecificationsFile};
use std::collections::BTreeSet;

const CURRENT_VERSION: &str = af_computation::ROOT_EXPORT_ANALYSIS_SEMANTICS_VERSION;
const STATIC_SPECS: &[&str] = &[
    "agentflow.computation.structural-binding.semantics@1",
    "agentflow.computation.proof-obligation.semantics@7",
    "agentflow.computation.proof-report.model@7",
    "agentflow.portable.proof.inputs.semantics@8",
    "agentflow.component.relation-snapshot-validation.semantics@1",
    "agentflow.computation.proof-obligation.semantics@6",
    "agentflow.computation.proof-report.model@6",
    "agentflow.portable.proof.inputs.semantics@7",
    "agentflow.component.document-occurrence.semantics@1",
    "agentflow.computation.proof-obligation.semantics@5",
    "agentflow.computation.proof-report.model@5",
    "agentflow.portable.proof.inputs.semantics@6",
    "agentflow.component.raw-validation.semantics@1",
    "agentflow.computation.proof-obligation.semantics@4",
    "agentflow.computation.proof-report.model@4",
    "agentflow.portable.proof.inputs.semantics@5",
    "agentflow.computation.proof-obligation.semantics@3",
    "agentflow.component.native-recovery-guard.semantics@1",
    "agentflow.computation.proof-report.model@3",
    "agentflow.portable.proof.inputs.semantics@4",
    "agentflow.computation.proof-obligation.semantics@2",
    "agentflow.component.substitution-proof.semantics@1",
    "agentflow.computation.proof-report.model@2",
    "agentflow.portable.proof.inputs.semantics@3",
    "agentflow.computation.proof-obligation.semantics@1",
    "agentflow.computation.proof-report.model@1",
    "agentflow.portable.proof.inputs.semantics@1",
    "agentflow.portable.proof.inputs.semantics@2",
    "agentflow.component.contract.semantics@1",
    "agentflow.component.relation-catalogs.semantics@1",
    "agentflow.effect.signature.semantics@1",
    "agentflow.execution.requirements.semantics@1",
    "agentflow.failure.row.semantics@1",
    "agentflow.information.flow.semantics@1",
    "agentflow.recovery.contract.semantics@1",
    "agentflow.resource.requirement.semantics@1",
    "agentflow.semantic.advanced-control-contract-algebra.semantics@1",
    "agentflow.semantic.control-contract-algebra.semantics@1",
    "agentflow.semantic.ir.semantics@1",
    "agentflow.semantic.leaf-contract-resolution.semantics@1",
    "agentflow.semantic.scope-compensation-contract-algebra.semantics@1",
    "agentflow.semantic.types@1",
    "agentflow.usage.signature.semantics@1",
    "agentflow.computation.root-export-contract-proof.semantics@1",
];

#[derive(Debug, Clone)]
pub struct SupportedPortableStaticInputs {
    content: ContentValidatedPortableBundle,
    semantic_version: &'static str,
    supported_schema_type_count: usize,
    supported_source_count: usize,
    applied_rule_count: usize,
}

impl SupportedPortableStaticInputs {
    #[must_use]
    pub fn content(&self) -> &ContentValidatedPortableBundle {
        &self.content
    }
    #[must_use]
    pub fn semantic_version(&self) -> &str {
        self.semantic_version
    }
    #[must_use]
    pub fn supported_schema_type_count(&self) -> usize {
        self.supported_schema_type_count
    }
    #[must_use]
    pub fn supported_source_count(&self) -> usize {
        self.supported_source_count
    }
    #[must_use]
    pub fn applied_rule_count(&self) -> usize {
        self.applied_rule_count
    }

    /// Reconstructs one exact, complete D5 static proof context from the
    /// already D0-D4 validated portable input. Unavailable collections fail
    /// closed and are never interpreted as empty.
    pub fn reconstruct_context(
        &self,
        context_id: u32,
    ) -> Result<crate::ReconstructedStaticContext, PortableProofError> {
        crate::reconstruction::reconstruct_context(self, context_id)
    }

    /// Reconstructs D5 and runs fresh D6 contract checks, D7 IR/Phase 3C
    /// checks, and D8 resolver assembly and comparison under the same exact
    /// proof-context object. The returned value is an owned analysis
    /// projection, not a RootExport or publication capability.
    pub fn reconstruct_static_inputs(
        &self,
        context_id: u32,
        resolver_id: u32,
    ) -> Result<crate::ReconstructedStaticInputs, PortableProofError> {
        crate::pipeline::reconstruct_static_inputs(self, context_id, resolver_id)
    }

    /// Reconstructs D5-D8 inputs, derives a fresh Root/RootRecovery summary,
    /// projects the root assertion, and calls the formal D10 RootExport
    /// checker under the same D5 context lifetime.
    pub fn revalidate_root_export(
        &self,
    ) -> Result<crate::PortableStaticProofAnalysis, PortableProofError> {
        crate::pipeline::revalidate_root_export(self)
    }
}

#[derive(Debug)]
pub(crate) struct CurrentSource {
    pub specification_id: &'static str,
    pub logical_path: &'static str,
    pub bytes: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/current_profile_sources.rs"));

pub(crate) fn validate_d4(
    content: ContentValidatedPortableProofBundleAlias,
) -> Result<SupportedPortableStaticInputs, PortableProofError> {
    let compiler = root_compiler(&content)?;
    let version = string_field(
        field(compiler, "static_identity_projection")?,
        "analysis_semantics_version",
    )?;
    if version != CURRENT_VERSION {
        let family = if matches!(
            version,
            "agentflow.root-export-checker@1"
                | "agentflow.root-export-checker@2"
                | "agentflow.root-export-checker@3"
                | "agentflow.root-export-checker@4"
                | "agentflow.root-export-checker@5"
                | "agentflow.root-export-checker@6"
                | "agentflow.root-export-checker@7"
        ) && embedded_blob(compiler, "registries", "specifications")?
            == SPECIFICATION_REGISTRY_BYTES
        {
            "InvalidIdentity"
        } else {
            "UnsupportedSemantics"
        };
        return Err(PortableProofError::new(
            ValidationStage::D4,
            family,
            "$.compiler_semantics_manifests.static_identity_projection.analysis_semantics_version",
            "current-only successor staging never upgrades an old or unknown edition",
        ));
    }
    let (profile, _) = variant(field(compiler, "attribution")?)?;
    if profile != "full_content_revalidation" {
        return Err(PortableProofError::new(
            ValidationStage::D4,
            "UnsupportedAttributionProfile",
            "$.compiler_semantics_manifests.attribution",
            "only full-content Profile A is implemented",
        ));
    }
    validate_current_tuple(compiler)?;
    let specifications_bytes = embedded_blob(compiler, "registries", "specifications")?;
    let embedded_rule_registry = embedded_blob(compiler, "registries", "rules")?;
    let specification_file: SpecificationsFile = serde_json::from_slice(specifications_bytes)
        .map_err(|error| {
            d4(
                "$.compiler_semantics_manifests.registries.specifications",
                error.to_string(),
            )
        })?;
    let rules_file: RulesFile =
        serde_json::from_slice(embedded_rule_registry).map_err(|error| {
            d4(
                "$.compiler_semantics_manifests.registries.rules",
                error.to_string(),
            )
        })?;
    if specification_file.specifications.len() != CURRENT_NORMATIVE_SOURCES.len()
        || CURRENT_NORMATIVE_SOURCES.len() != 70
    {
        return Err(d4(
            "$.compiler_semantics_manifests.normative_sources",
            "current support profile source count differs",
        ));
    }

    let rows = list(
        compiler,
        "normative_sources",
        "$.compiler_semantics_manifests",
    )?;
    if rows.len() != CURRENT_NORMATIVE_SOURCES.len() {
        return Err(d4(
            "$.compiler_semantics_manifests.normative_sources",
            "source set is incomplete",
        ));
    }
    for expected in CURRENT_NORMATIVE_SOURCES {
        let row = rows
            .iter()
            .find(|row| {
                string_field(row, "specification_id").ok() == Some(expected.specification_id)
            })
            .ok_or_else(|| {
                d4(
                    "$.compiler_semantics_manifests.normative_sources",
                    "a current source row is absent",
                )
            })?;
        if string_field(row, "logical_path")? != expected.logical_path
            || bytes_field(row, "content")? != expected.bytes
            || plain_checksum(bytes_field(row, "content")?)
                != string_field(field(row, "content_sha256")?, "hex")?
        {
            return Err(d4(
                "$.compiler_semantics_manifests.normative_sources",
                "source identity differs from the implementation-owned current profile",
            ));
        }
        let registered = specification_file
            .specifications
            .iter()
            .find(|specification| specification.specification_id == expected.specification_id)
            .ok_or_else(|| {
                d4(
                    "$.compiler_semantics_manifests.normative_sources",
                    "source row is absent from exact current registry",
                )
            })?;
        if registered.normative_source.logical_path != expected.logical_path
            || registered.normative_source.content_digest.hex != plain_checksum(expected.bytes)
        {
            return Err(d4(
                "$.compiler_semantics_manifests.normative_sources",
                "embedded registry row differs from compiled source identity",
            ));
        }
    }
    let normalized_ids = STATIC_SPECS
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<BTreeSet<_>>();
    let projection = field(compiler, "static_identity_projection")?;
    if string_field(projection, "analysis_semantics_version")? != CURRENT_VERSION
        || text_set(projection, "normative_specification_ids")? != normalized_ids
    {
        return Err(PortableProofError::new(
            ValidationStage::D4,
            "ConflictingSemanticContent",
            "$.compiler_semantics_manifests.static_identity_projection",
            "current semantic version or source projection differs",
        ));
    }

    let expected_rules = expected_rule_rows(&rules_file.rules)?;
    let applied = list(
        projection,
        "applied_rules",
        "$.compiler_semantics_manifests.static_identity_projection",
    )?;
    if applied.len() != expected_rules.len()
        || applied
            .iter()
            .zip(expected_rules.iter())
            .any(|(wire, row)| !value_rule_matches(wire, row))
    {
        return Err(d4(
            "$.compiler_semantics_manifests.static_identity_projection.applied_rules",
            "the exact current 174 applied RuleContentIdentity rows are required; not a dependency closure",
        ));
    }
    let (profile_name, profile_payload) = variant(field(compiler, "attribution")?)?;
    if profile_name != "full_content_revalidation" {
        return Err(PortableProofError::new(
            ValidationStage::D4,
            "UnsupportedAttributionProfile",
            "$.compiler_semantics_manifests.attribution",
            "only full-content Profile A is implemented",
        ));
    }
    let listed = list(
        profile_payload,
        "listed_rules",
        "$.compiler_semantics_manifests.attribution",
    )?;
    if listed.len() != expected_rules.len()
        || listed
            .iter()
            .zip(expected_rules.iter())
            .any(|(wire, row)| !value_rule_matches(wire, row))
    {
        return Err(d4(
            "$.compiler_semantics_manifests.attribution",
            "Profile A rows must exactly equal the current applied set",
        ));
    }

    let roles = list(compiler, "semantic_roles", "$.compiler_semantics_manifests")?;
    validate_roles(roles)?;
    let expected_schema_refs = registered_schema_refs()?;
    for role in roles.iter().filter_map(|entry| {
        let declaration = field(entry, "declaration").ok()?;
        let (name, value) = variant(declaration).ok()?;
        (name == "defined").then_some(value)
    }) {
        let pins = list(role, "required_schema_pins", "$.semantic_role")?;
        if pins.len() != expected_schema_refs.len()
            || pins
                .iter()
                .zip(expected_schema_refs.iter())
                .any(|(wire, expected)| !equal_schema_ref(wire, expected))
        {
            return Err(d4(
                "$.compiler_semantics_manifests.semantic_roles.required_schema_pins",
                "defined role must bind all current registered named schemas",
            ));
        }
    }

    let registry_bytes = embedded_blob(compiler, "registries", "specifications")?;
    let rule_bytes = embedded_blob(compiler, "registries", "rules")?;
    if registry_bytes != SPECIFICATION_REGISTRY_BYTES || rule_bytes != RULE_REGISTRY_BYTES {
        return Err(PortableProofError::new(
            ValidationStage::D4,
            "ConflictingSemanticContent",
            "$.compiler_semantics_manifests.registries",
            "registry bytes differ from the exact implementation-owned current content",
        ));
    }
    let expected_tables = [
        ("hash_domains", HASH_DOMAIN_TABLE_BYTES),
        ("abi_numbers", ABI_TABLE_BYTES),
        ("rule_areas", RULE_AREA_TABLE_BYTES),
        ("afidl_primitives", AFIDL_PRIMITIVE_TABLE_BYTES),
    ];
    for (name, expected) in expected_tables {
        if embedded_blob(compiler, "closed_tables", name)? != expected {
            return Err(PortableProofError::new(
                ValidationStage::D4,
                "ConflictingSemanticContent",
                format!("$.compiler_semantics_manifests.closed_tables.{name}"),
                "closed table differs from the implementation-owned current bytes",
            ));
        }
    }

    let supported_source_count = rows.len();
    let applied_rule_count = expected_rules.len();
    let stage = af_computation::current_registered_analysis_edition();
    if !stage.can_issue_capability() {
        return Err(PortableProofError::new(
            ValidationStage::D4,
            "UnsupportedSemantics",
            "$.compiler_semantics_manifests",
            "Successor is Registered/NotImplemented/NotAuthorized; R3/T4-T8 are incomplete",
        ));
    }
    Ok(SupportedPortableStaticInputs {
        content,
        semantic_version: CURRENT_VERSION,
        supported_schema_type_count: expected_schema_refs.len(),
        supported_source_count,
        applied_rule_count,
    })
}

type ContentValidatedPortableProofBundleAlias = ContentValidatedPortableBundle;

fn root_compiler(content: &ContentValidatedPortableBundle) -> Result<&Value, PortableProofError> {
    let bundle = content.value();
    let root_id = u32_value(field(bundle, "root_assertion_id")?)
        .ok_or_else(|| d4("$.root_assertion_id", "invalid root assertion ID"))?;
    let assertions = list(bundle, "root_export_assertions", "$")?;
    let root = assertions
        .iter()
        .find(|entry| u32_value(field(entry, "id").unwrap_or(&Value::Null)) == Some(root_id))
        .ok_or_else(|| d4("$.root_assertion_id", "root assertion ID did not resolve"))?;
    let assertion = field(root, "assertion")?;
    let manifest_id = u32_value(field(assertion, "compiler_semantics_id")?).ok_or_else(|| {
        d4(
            "$.root_assertion.compiler_semantics_id",
            "invalid compiler manifest ID",
        )
    })?;
    list(bundle, "compiler_semantics_manifests", "$")?
        .iter()
        .find(|entry| u32_value(field(entry, "id").unwrap_or(&Value::Null)) == Some(manifest_id))
        .map(|entry| field(entry, "content").unwrap_or(&Value::Null))
        .ok_or_else(|| {
            d4(
                "$.root_assertion.compiler_semantics_id",
                "compiler manifest ID did not resolve",
            )
        })
}

fn validate_current_tuple(compiler: &Value) -> Result<(), PortableProofError> {
    let expected: [(&str, &[u8]); 6] = [
        ("specifications", SPECIFICATION_REGISTRY_BYTES),
        ("rules", RULE_REGISTRY_BYTES),
        ("hash_domains", HASH_DOMAIN_TABLE_BYTES),
        ("abi_numbers", ABI_TABLE_BYTES),
        ("rule_areas", RULE_AREA_TABLE_BYTES),
        ("afidl_primitives", AFIDL_PRIMITIVE_TABLE_BYTES),
    ];
    let registries = field(compiler, "registries")?;
    let tables = field(compiler, "closed_tables")?;
    for (name, bytes) in expected {
        let container = if matches!(name, "specifications" | "rules") {
            registries
        } else {
            tables
        };
        let content = field(container, name)?;
        let actual = bytes_field(content, "content")?;
        let checksum = field(content, "content_sha256")?;
        if actual != bytes || string_field(checksum, "hex")? != plain_checksum(bytes) {
            return Err(PortableProofError::new(
                ValidationStage::D4,
                "InvalidIdentity",
                "$.compiler_semantics_manifests",
                "compiler content tuple differs from the implementation-owned current profile",
            ));
        }
    }
    Ok(())
}

fn validate_roles(roles: &[Value]) -> Result<(), PortableProofError> {
    const RANKED: [&str; 10] = [
        "static_analysis",
        "contract_validation",
        "semantic_ir_validation",
        "dataflow",
        "leaf_resolution",
        "control_algebra",
        "root_recovery",
        "root_export",
        "plan_lowering",
        "runtime_semantics_profile",
    ];
    if roles.len() != RANKED.len() {
        return Err(PortableProofError::new(
            ValidationStage::D4,
            "MissingSemanticRole",
            "$.compiler_semantics_manifests.semantic_roles",
            "all ten closed role declarations are required",
        ));
    }
    let mut actual = Vec::new();
    for entry in roles {
        let role = string_field(entry, "role")?;
        actual.push(role);
        let declaration = field(entry, "declaration")?;
        let (case, payload) = variant(declaration)?;
        match (role, case) {
            ("plan_lowering" | "runtime_semantics_profile", "model_missing") => {
                if value_name(payload)? != "not_formalized" {
                    return Err(d4(
                        "$.compiler_semantics_manifests.semantic_roles",
                        "missing roles use the registered not_formalized reason",
                    ));
                }
            }
            ("plan_lowering" | "runtime_semantics_profile", _) => {
                return Err(d4(
                    "$.compiler_semantics_manifests.semantic_roles",
                    "lowering and full runtime roles remain ModelMissing",
                ));
            }
            (_, "defined") => {
                if string_field(payload, "analysis_semantics_version")? != CURRENT_VERSION {
                    return Err(PortableProofError::new(
                        ValidationStage::D4,
                        "ConflictingSemanticContent",
                        "$.compiler_semantics_manifests.semantic_roles",
                        "defined role version differs from the exact current profile",
                    ));
                }
                if text_set(payload, "normative_specifications")?
                    != STATIC_SPECS
                        .iter()
                        .map(|value| (*value).to_owned())
                        .collect()
                {
                    return Err(d4(
                        "$.compiler_semantics_manifests.semantic_roles",
                        "defined role source bindings are incomplete",
                    ));
                }
                check_role_dependencies(role, list(payload, "depends_on", "$")?)?;
            }
            _ => {
                return Err(d4(
                    "$.compiler_semantics_manifests.semantic_roles",
                    "static role is not defined",
                ));
            }
        }
    }
    if actual != RANKED {
        return Err(d4(
            "$.compiler_semantics_manifests.semantic_roles",
            "exactly one role per enum rank is required",
        ));
    }
    Ok(())
}

fn check_role_dependencies(role: &str, dependencies: &[Value]) -> Result<(), PortableProofError> {
    let expected: &[&str] = match role {
        "contract_validation" | "semantic_ir_validation" => &[],
        "dataflow" => &["semantic_ir_validation"],
        "leaf_resolution" => &["contract_validation", "semantic_ir_validation", "dataflow"],
        "control_algebra" => &["leaf_resolution"],
        "root_recovery" => &["dataflow", "control_algebra"],
        "root_export" => &[
            "contract_validation",
            "semantic_ir_validation",
            "dataflow",
            "leaf_resolution",
            "control_algebra",
            "root_recovery",
        ],
        "static_analysis" => &[
            "contract_validation",
            "semantic_ir_validation",
            "dataflow",
            "leaf_resolution",
            "control_algebra",
            "root_recovery",
            "root_export",
        ],
        _ => {
            return Err(d4(
                "$.compiler_semantics_manifests.semantic_roles",
                "unknown defined role",
            ));
        }
    };
    let actual = dependencies
        .iter()
        .map(value_name)
        .collect::<Result<Vec<_>, _>>()?;
    if actual.iter().map(String::as_str).collect::<Vec<_>>() != expected {
        return Err(d4(
            "$.compiler_semantics_manifests.semantic_roles.depends_on",
            "role dependency graph differs from the exact current profile",
        ));
    }
    Ok(())
}

fn expected_rule_rows(rules: &[RuleJson]) -> Result<Vec<&RuleJson>, PortableProofError> {
    let mut ids = (1..=22)
        .map(|n| format!("AF-CONTRACT-{n:04}"))
        .chain((111..=172).map(|n| format!("AF-SEMANTIC-{n:04}")))
        .chain(
            (173..=187)
                .chain([189])
                .chain(191..=222)
                .chain(228..=242)
                .chain(245..=260)
                .chain(261..=271)
                .chain(272..=301)
                .chain(302..=314)
                .map(|n| format!("AF-SEMANTIC-{n:04}")),
        )
        .collect::<Vec<_>>();
    ids.sort();
    ids.into_iter()
        .map(|id| {
            rules.iter().find(|row| row.rule_id == id).ok_or_else(|| {
                d4(
                    "$.compiler_semantics_manifests.applied_rules",
                    "current applied rule row missing from full registry",
                )
            })
        })
        .collect()
}

fn value_rule_matches(value: &Value, row: &RuleJson) -> bool {
    [
        ("rule_id", row.rule_id.as_str()),
        ("normative_source", row.normative_source.as_str()),
        ("source_anchor", row.source_anchor.as_str()),
        ("short_statement", row.short_statement.as_str()),
    ]
    .into_iter()
    .all(|(field, expected)| string_field(value, field).is_ok_and(|actual| actual == expected))
}

fn text_set(value: &Value, name: &str) -> Result<BTreeSet<String>, PortableProofError> {
    let values = list(value, name, "$")?;
    let mut result = BTreeSet::new();
    for item in values {
        let Value::String(text) = item else {
            return Err(d4("$.compiler_semantics_manifests", "expected text set"));
        };
        result.insert(text.clone());
    }
    Ok(result)
}

fn equal_schema_ref(value: &Value, expected: &af_semantic::TypeSchemaRef) -> bool {
    let Ok(specification_id) = string_field(value, "specification_id") else {
        return false;
    };
    let Ok(type_name) = string_field(value, "type_name") else {
        return false;
    };
    let Ok(digest) = field(value, "schema_digest") else {
        return false;
    };
    let Ok(algorithm) = string_field(digest, "algorithm") else {
        return false;
    };
    let Ok(domain) = string_field(digest, "domain") else {
        return false;
    };
    let Ok(hex) = string_field(digest, "hex") else {
        return false;
    };
    specification_id == expected.specification_id
        && type_name == expected.type_name
        && algorithm == expected.schema_digest.algorithm
        && domain == expected.schema_digest.domain
        && hex == expected.schema_digest.hex
}

fn embedded_blob<'a>(
    value: &'a Value,
    parent: &str,
    name: &str,
) -> Result<&'a [u8], PortableProofError> {
    let container = field(value, parent)?;
    let body = field(container, name)?;
    bytes_field(body, "content")
}

fn plain_checksum(bytes: &[u8]) -> String {
    use sha2::{Digest as _, Sha256};
    hex::encode(Sha256::digest(bytes))
}

fn field<'a>(value: &'a Value, name: &str) -> Result<&'a Value, PortableProofError> {
    let Value::Record(fields) = value else {
        return Err(d4("$", "expected typed record"));
    };
    fields
        .iter()
        .find(|(field, _)| field == name)
        .map(|(_, value)| value)
        .ok_or_else(|| d4("$", "required typed field missing"))
}
fn list<'a>(value: &'a Value, name: &str, _path: &str) -> Result<&'a [Value], PortableProofError> {
    match field(value, name)? {
        Value::List(values) => Ok(values),
        _ => Err(d4("$", "expected typed list")),
    }
}
fn string_field<'a>(value: &'a Value, name: &str) -> Result<&'a str, PortableProofError> {
    match field(value, name)? {
        Value::Enum(text) | Value::String(text) => Ok(text),
        _ => Err(d4("$", "expected text")),
    }
}
fn value_name(value: &Value) -> Result<String, PortableProofError> {
    match value {
        Value::Enum(name) | Value::String(name) => Ok(name.clone()),
        _ => Err(d4("$", "expected closed enum or text")),
    }
}
fn bytes_field<'a>(value: &'a Value, name: &str) -> Result<&'a [u8], PortableProofError> {
    match field(value, name)? {
        Value::Bytes(bytes) => Ok(bytes),
        _ => Err(d4("$", "expected inline bytes")),
    }
}
fn u32_value(value: &Value) -> Option<u32> {
    match value {
        Value::UInt32(number) => Some(*number),
        _ => None,
    }
}
fn variant(value: &Value) -> Result<(&str, &Value), PortableProofError> {
    match value {
        Value::Variant { case, value } => Ok((case, value)),
        _ => Err(d4("$", "expected closed variant")),
    }
}
fn d4(path: &str, detail: impl Into<String>) -> PortableProofError {
    PortableProofError::new(ValidationStage::D4, "UnsupportedSemantics", path, detail)
}
