//! Exact D5 reconstruction of the explicit static proof context.
//!
//! The portable manifest is treated as input, never as an already-built
//! checker context. Every catalog is independently rebuilt, and the resulting
//! context is reprojected from read-only typed accessors before it is exposed.

use crate::error::{PortableProofError, ValidationStage};
use crate::support::SupportedPortableStaticInputs;
use crate::validation::parse_schema;
use af_contract::{
    EffectResourceCompatibilityCatalog, InformationFlowExecutionCatalog,
    RecoveryUsageCoverageCatalog, RelationCatalogSnapshot, RelationSnapshots,
    StaticContractProofContext,
};
use af_core::{Value, decode};
use af_effect::{CapabilityCatalog, EffectCatalogUniverse};
use af_execution::{
    CapabilityAttribute, ExecutionCapabilityRef, ExecutionUniverses, RegionRef, RuntimeProfileRef,
    TrustDomainRef,
};
use af_flow::{ConfidentialityLabelRef, FlowCatalogs, TaintRef};
use af_idl::{CanonicalBody, CanonicalTypeExpr, CanonicalTypeSchema, CompiledModuleSet};
use af_recovery::RecoveryOperationUniverse;
use af_resource::{ResourceTypeDescriptor, ResourceTypeRef, ResourceTypeUniverse};
use af_semantic::{TypeSchemaRef, TypeSchemaUniverse};
use af_usage::{UsageAggregation, UsageMeterRef, UsageMeterUniverse, UsageUnitRef};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::sync::{Arc, Mutex, OnceLock};
use unicode_normalization::UnicodeNormalization;

const MAX_RECONSTRUCTED_MODULES: usize = 256;
const MAX_RECONSTRUCTED_MODULE_TYPES: usize = 1024;
const MAX_RECONSTRUCTED_SCHEMAS: usize = 16_384;
const MAX_RECONSTRUCTED_CATALOG_ROWS: usize = 100_000;
const MAX_SCHEMA_CACHE_MATERIAL_BYTES: usize = 32 * 1024 * 1024;
const MAX_TYPE_EXPANSION_DEPTH: usize = 256;
const MAX_TYPE_EXPANSION_WORK: usize = 1_000_000;
type SchemaAssetMap = BTreeMap<(String, String), (TypeSchemaRef, CanonicalTypeSchema)>;
type SchemaAssetCacheEntry = (Vec<u8>, Arc<SchemaAssetMap>);
type ModuleSetCacheEntry = (Vec<u8>, Arc<CompiledModuleSet>);
type SchemaAssetCache = Mutex<Option<SchemaAssetCacheEntry>>;
type ModuleSetCache = Mutex<Option<ModuleSetCacheEntry>>;
static SCHEMA_ASSET_CACHE: OnceLock<SchemaAssetCache> = OnceLock::new();
static MODULE_SET_CACHE: OnceLock<ModuleSetCache> = OnceLock::new();

/// Lossless, portable-independent projection used for D5 recapture equality.
/// It contains declaration facts only; it is not a proof or authority token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StaticContextProjection {
    pub context_id: u32,
    pub modules: Vec<ModuleProjection>,
    pub type_schemas: Vec<TypeSchemaRef>,
    pub effect_catalogs: Vec<EffectCatalogProjection>,
    pub resource_schemas: Vec<TypeSchemaRef>,
    pub resource_descriptors: Vec<ResourceDescriptorProjection>,
    pub confidentiality_labels: Vec<ConfidentialityLabelRef>,
    pub taints: Vec<TaintRef>,
    pub flow_effect_sinks: Vec<FlowSinkProjection>,
    pub usage_meters: Vec<UsageMeterProjection>,
    pub recovery_effect_catalogs: Vec<EffectCatalogProjection>,
    pub reconciliation_operations: Vec<NamedOperationProjection>,
    pub execution_capabilities: Vec<ExecutionCapabilityProjection>,
    pub trust_domains: Vec<TrustDomainRef>,
    pub regions: Vec<RegionRef>,
    pub region_compatibility: Vec<RegionCompatibilityProjection>,
    pub runtime_profiles: Vec<RuntimeProfileProjection>,
    pub effect_resource_relation: Option<RelationProjection>,
    pub recovery_usage_relation: Option<RelationProjection>,
    pub flow_execution_relation: Option<RelationProjection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleProjection {
    pub specification_id: String,
    pub named_types: Vec<TypeSchemaRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EffectCatalogProjection {
    pub specification_id: String,
    pub operations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceDescriptorProjection {
    pub schema: TypeSchemaRef,
    pub recognized_attributes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FlowSinkProjection {
    pub operation: af_effect::EffectOperationRef,
    pub decision_capable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UsageMeterProjection {
    pub meter: UsageMeterRef,
    pub unit: UsageUnitProjection,
    pub aggregation: PortableUsageAggregation,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UsageUnitProjection {
    pub catalog: String,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PortableUsageAggregation {
    Additive,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NamedOperationProjection {
    pub catalog: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionCapabilityProjection {
    pub capability: ExecutionCapabilityRef,
    pub exact_version: String,
    pub supported_versions: Vec<String>,
    pub attributes: Vec<CapabilityAttribute>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RegionCompatibilityProjection {
    pub from: RegionRef,
    pub to: RegionRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeProfileProjection {
    pub profile: RuntimeProfileRef,
    pub family: String,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlainChecksumProjection {
    pub algorithm: String,
    pub hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelationProjection {
    pub schema: TypeSchemaRef,
    pub revision: String,
    pub canonical_content_checksum: PlainChecksumProjection,
    pub canonical_content: Vec<u8>,
}

/// Owned D5 arenas. Its proof-context reference is created only in a scoped
/// callback, preventing self-referential checked wrappers.
#[derive(Debug)]
pub struct ReconstructedStaticContext {
    context_id: u32,
    modules: CompiledModuleSet,
    schema_universe: TypeSchemaUniverse,
    effects: EffectCatalogUniverse,
    resources: ResourceTypeUniverse,
    flow: FlowCatalogs,
    usage_meters: UsageMeterUniverse,
    recovery: RecoveryOperationUniverse,
    execution: ExecutionUniverses,
    relations: RelationSnapshots,
    projection: StaticContextProjection,
}

impl ReconstructedStaticContext {
    #[must_use]
    pub fn context_id(&self) -> u32 {
        self.context_id
    }

    /// Complete recaptured typed facts from the newly constructed D5 arenas.
    #[must_use]
    pub fn projection(&self) -> &StaticContextProjection {
        &self.projection
    }

    /// Runs a caller operation under a short-lived formal proof context whose
    /// references all point at these exact owned D5 arenas.
    pub fn with_proof_context<R>(
        &self,
        operation: impl FnOnce(&StaticContractProofContext<'_>) -> R,
    ) -> R {
        let context = StaticContractProofContext {
            modules: &self.modules,
            schema_universe: &self.schema_universe,
            effects: &self.effects,
            resources: &self.resources,
            flow: &self.flow,
            usage_meters: &self.usage_meters,
            recovery: &self.recovery,
            execution: &self.execution,
            relations: &self.relations,
        };
        operation(&context)
    }
}

pub(crate) fn reconstruct_context(
    supported: &SupportedPortableStaticInputs,
    context_id: u32,
) -> Result<ReconstructedStaticContext, PortableProofError> {
    let root = supported.content().value();
    let entry = list_field(root, "proof_contexts", "$")?
        .iter()
        .find(|item| u32_field(item, "id").ok() == Some(context_id))
        .ok_or_else(|| reconstruction_error("$.proof_contexts", "unknown context id"))?;
    let content = field(entry, "content", "$.proof_contexts[]")?;

    let expected = parse_context_projection(content, context_id)?;
    check_reconstruction_limits(&expected)?;

    let schema_assets = parse_schema_assets(root)?;
    let schema_universe = build_schema_universe(&expected.type_schemas, &schema_assets)?;
    let modules = build_module_set(&expected.modules, &schema_assets)?;
    for identity in [
        "agentflow.component.contract@1/ComponentContract",
        "agentflow.semantic.ir@1/SemanticIrDocument",
    ] {
        if modules.type_by_identity(identity).is_none() {
            return Err(reconstruction_error(
                "$.context.compiled_modules",
                format!("required formal checker root schema {identity} is absent"),
            ));
        }
    }
    let effects = build_effect_universe(&expected.effect_catalogs)?;
    let resources = build_resource_universe(
        &expected.resource_schemas,
        &expected.resource_descriptors,
        &schema_assets,
    )?;
    let flow = build_flow_catalogs(
        &expected.confidentiality_labels,
        &expected.taints,
        &expected.flow_effect_sinks,
    )?;
    let usage_meters = build_usage_meters(&expected.usage_meters)?;
    let recovery = build_recovery_universe(
        &expected.recovery_effect_catalogs,
        &expected.reconciliation_operations,
    )?;
    let execution = build_execution_universes(&expected)?;
    let relations = build_relations(&expected, &modules)?;

    // All construction APIs that normalize or coalesce input have already
    // been guarded by duplicate/canonical checks. Recapture is derived only
    // from the resulting typed arenas, never copied from the declaration.
    let actual = recapture_projection(
        context_id,
        &modules,
        &schema_universe,
        &effects,
        &resources,
        &flow,
        &usage_meters,
        &recovery,
        &execution,
        &relations,
    )?;
    if actual != expected {
        return Err(reconstruction_error(
            "$.proof_contexts.content",
            "D5 recaptured typed projection differs from the complete portable declaration",
        ));
    }

    Ok(ReconstructedStaticContext {
        context_id,
        modules,
        schema_universe,
        effects,
        resources,
        flow,
        usage_meters,
        recovery,
        execution,
        relations,
        projection: actual,
    })
}

fn parse_context_projection(
    content: &Value,
    context_id: u32,
) -> Result<StaticContextProjection, PortableProofError> {
    let compiled_modules = complete_vec::<ModuleProjection>(
        field(content, "compiled_modules", "$.context")?,
        "$.context.compiled_modules",
    )?;
    let type_schemas = complete_vec::<TypeSchemaRef>(
        field(content, "type_schemas", "$.context")?,
        "$.context.type_schemas",
    )?;
    let effect_catalogs = complete_vec::<EffectCatalogProjection>(
        field(content, "effect_catalogs", "$.context")?,
        "$.context.effect_catalogs",
    )?;

    let resource = field(content, "resource_types", "$.context")?;
    let resource_schemas = complete_vec::<TypeSchemaRef>(
        field(resource, "schemas", "$.context.resource_types")?,
        "$.context.resource_types.schemas",
    )?;
    let resource_descriptors = complete_vec::<ResourceDescriptorProjection>(
        field(resource, "descriptors", "$.context.resource_types")?,
        "$.context.resource_types.descriptors",
    )?;

    let flow = field(content, "flow_catalogs", "$.context")?;
    let confidentiality_labels = complete_vec::<ConfidentialityLabelRef>(
        field(flow, "confidentiality_labels", "$.context.flow_catalogs")?,
        "$.context.flow_catalogs.confidentiality_labels",
    )?;
    let taints = complete_vec::<TaintRef>(
        field(flow, "taints", "$.context.flow_catalogs")?,
        "$.context.flow_catalogs.taints",
    )?;
    let flow_effect_sinks = complete_vec::<FlowSinkProjection>(
        field(flow, "effect_sinks", "$.context.flow_catalogs")?,
        "$.context.flow_catalogs.effect_sinks",
    )?;
    let usage_meters = complete_vec::<UsageMeterProjection>(
        field(content, "usage_meters", "$.context")?,
        "$.context.usage_meters",
    )?;

    let recovery = field(content, "recovery_operations", "$.context")?;
    let recovery_effect_catalogs = complete_vec::<EffectCatalogProjection>(
        field(recovery, "effect_catalogs", "$.context.recovery_operations")?,
        "$.context.recovery_operations.effect_catalogs",
    )?;
    let reconciliation_operations = complete_vec::<NamedOperationProjection>(
        field(
            recovery,
            "reconciliation_operations",
            "$.context.recovery_operations",
        )?,
        "$.context.recovery_operations.reconciliation_operations",
    )?;

    let execution = field(content, "execution_universes", "$.context")?;
    let execution_capabilities = complete_vec::<ExecutionCapabilityProjection>(
        field(execution, "capabilities", "$.context.execution_universes")?,
        "$.context.execution_universes.capabilities",
    )?;
    let trust_domains = complete_vec::<TrustDomainRef>(
        field(execution, "trust_domains", "$.context.execution_universes")?,
        "$.context.execution_universes.trust_domains",
    )?;
    let regions = complete_vec::<RegionRef>(
        field(execution, "regions", "$.context.execution_universes")?,
        "$.context.execution_universes.regions",
    )?;
    let region_compatibility = complete_vec::<RegionCompatibilityProjection>(
        field(
            execution,
            "region_compatibility",
            "$.context.execution_universes",
        )?,
        "$.context.execution_universes.region_compatibility",
    )?;
    let runtime_profiles = complete_vec::<RuntimeProfileProjection>(
        field(
            execution,
            "runtime_profiles",
            "$.context.execution_universes",
        )?,
        "$.context.execution_universes.runtime_profiles",
    )?;

    let relations = field(content, "relation_snapshots", "$.context")?;
    Ok(StaticContextProjection {
        context_id,
        modules: compiled_modules,
        type_schemas,
        effect_catalogs,
        resource_schemas,
        resource_descriptors,
        confidentiality_labels,
        taints,
        flow_effect_sinks,
        usage_meters,
        recovery_effect_catalogs,
        reconciliation_operations,
        execution_capabilities,
        trust_domains,
        regions,
        region_compatibility,
        runtime_profiles,
        effect_resource_relation: parse_relation_projection(
            field(relations, "effect_resource", "$.context.relation_snapshots")?,
            "$.context.relation_snapshots.effect_resource",
        )?,
        recovery_usage_relation: parse_relation_projection(
            field(relations, "recovery_usage", "$.context.relation_snapshots")?,
            "$.context.relation_snapshots.recovery_usage",
        )?,
        flow_execution_relation: parse_relation_projection(
            field(relations, "flow_execution", "$.context.relation_snapshots")?,
            "$.context.relation_snapshots.flow_execution",
        )?,
    })
}

fn parse_relation_projection(
    value: &Value,
    path: &str,
) -> Result<Option<RelationProjection>, PortableProofError> {
    let Value::Option(value) = value else {
        return Err(reconstruction_error(path, "expected closed Option value"));
    };
    let Some(value) = value else {
        return Ok(None);
    };
    let header = field(value, "header", path)?;
    let checksum = field(header, "canonical_content_checksum", path)?;
    Ok(Some(RelationProjection {
        schema: from_value(field(header, "schema", path)?, path)?,
        revision: string_field(header, "revision", path)?.to_owned(),
        canonical_content_checksum: PlainChecksumProjection {
            algorithm: enum_or_string_field(checksum, "algorithm", path)?.to_owned(),
            hex: string_field(checksum, "hex", path)?.to_owned(),
        },
        canonical_content: bytes_field(value, "canonical_content", path)?.to_vec(),
    }))
}

fn parse_schema_assets(root: &Value) -> Result<Arc<SchemaAssetMap>, PortableProofError> {
    let rows = list_field(root, "schema_assets", "$")?;
    if rows.len() > MAX_RECONSTRUCTED_SCHEMAS {
        return Err(resource_limit(
            "$.schema_assets",
            "schema asset limit exceeded",
        ));
    }
    let mut cache_material = Vec::new();
    for row in rows {
        let reference = field(row, "reference", "$.schema_assets")?;
        for name in ["specification_id", "type_name"] {
            append_cache_part(
                &mut cache_material,
                string_field(reference, name, "$.schema_assets")?.as_bytes(),
            )?;
        }
        let digest = field(reference, "schema_digest", "$.schema_assets")?;
        for name in ["algorithm", "domain", "hex"] {
            append_cache_part(
                &mut cache_material,
                string_field(digest, name, "$.schema_assets")?.as_bytes(),
            )?;
        }
        append_cache_part(
            &mut cache_material,
            bytes_field(row, "canonical_content", "$.schema_assets")?,
        )?;
    }
    if let Some((_, cached)) = SCHEMA_ASSET_CACHE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .filter(|(material, _)| *material == cache_material)
    {
        return Ok(Arc::clone(cached));
    }

    let mut result = BTreeMap::new();
    for (index, row) in rows.iter().enumerate() {
        let path = format!("$.schema_assets[{index}]");
        let reference: TypeSchemaRef = from_value(field(row, "reference", &path)?, &path)?;
        let bytes = bytes_field(row, "canonical_content", &path)?;
        let decoded = decode(crate::schema::type_schema_shape(), bytes)
            .map_err(|error| reconstruction_error(&path, error.to_string()))?;
        let schema = parse_schema(&decoded, &path)
            .map_err(|error| reconstruction_error(&path, error.to_string()))?;
        if schema.schema != reference.specification_id || schema.name != reference.type_name {
            return Err(reconstruction_error(
                &path,
                "schema asset identity differs from its exact TypeSchemaRef",
            ));
        }
        let digest = schema
            .digest()
            .map_err(|error| reconstruction_error(&path, error.to_string()))?;
        if reference.schema_digest.algorithm != "sha256"
            || reference.schema_digest.domain != "type-schema"
            || reference.schema_digest.hex != digest.hex()
        {
            return Err(reconstruction_error(
                &path,
                "schema asset digest differs from its exact TypeSchemaRef",
            ));
        }
        if result
            .insert(
                (
                    reference.specification_id.clone(),
                    reference.type_name.clone(),
                ),
                (reference, schema),
            )
            .is_some()
        {
            return Err(reconstruction_error(
                &path,
                "duplicate schema asset identity",
            ));
        }
    }
    let result = Arc::new(result);
    *SCHEMA_ASSET_CACHE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) =
        Some((cache_material, Arc::clone(&result)));
    Ok(result)
}

fn append_cache_part(buffer: &mut Vec<u8>, part: &[u8]) -> Result<(), PortableProofError> {
    let length = u64::try_from(part.len())
        .map_err(|_| resource_limit("$.schema_assets", "schema cache component length overflow"))?;
    let required = buffer
        .len()
        .checked_add(8)
        .and_then(|size| size.checked_add(part.len()))
        .ok_or_else(|| {
            resource_limit("$.schema_assets", "schema cache material length overflow")
        })?;
    if required > MAX_SCHEMA_CACHE_MATERIAL_BYTES {
        return Err(resource_limit(
            "$.schema_assets",
            "schema asset cache material limit exceeded",
        ));
    }
    buffer.extend_from_slice(&length.to_be_bytes());
    buffer.extend_from_slice(part);
    Ok(())
}

fn build_schema_universe(
    references: &[TypeSchemaRef],
    assets: &BTreeMap<(String, String), (TypeSchemaRef, CanonicalTypeSchema)>,
) -> Result<TypeSchemaUniverse, PortableProofError> {
    let mut seen = BTreeSet::new();
    let mut universe = TypeSchemaUniverse::new();
    for (index, reference) in references.iter().enumerate() {
        let identity = (
            reference.specification_id.clone(),
            reference.type_name.clone(),
        );
        if !seen.insert(identity.clone()) {
            return Err(reconstruction_error(
                "$.context.type_schemas",
                "duplicate exact schema membership",
            ));
        }
        let (asset_ref, schema) = assets.get(&identity).ok_or_else(|| {
            reconstruction_error(
                format!("$.context.type_schemas[{index}]"),
                "declared schema membership has no matching inline SchemaAsset",
            )
        })?;
        if asset_ref != reference {
            return Err(reconstruction_error(
                format!("$.context.type_schemas[{index}]"),
                "TypeSchemaRef pin differs from the matching SchemaAsset",
            ));
        }
        universe
            .insert_type(schema)
            .map_err(|error| reconstruction_error("$.context.type_schemas", error.to_string()))?;
    }
    validate_alias_expansion_budget(&universe)?;
    Ok(universe)
}

fn validate_alias_expansion_budget(
    universe: &TypeSchemaUniverse,
) -> Result<(), PortableProofError> {
    let entries = universe
        .entries()
        .map(|entry| (entry.identity(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut alias_edges = BTreeMap::<String, String>::new();
    for (identity, entry) in &entries {
        let CanonicalBody::Alias(expression) = &entry.body else {
            continue;
        };
        let Some(target_identity) = named_reference(expression) else {
            continue;
        };
        if entries
            .get(target_identity)
            .is_some_and(|target| matches!(target.body, CanonicalBody::Alias(_)))
        {
            alias_edges.insert(identity.clone(), target_identity.to_owned());
        }
    }

    let mut completed_depths = BTreeMap::<String, usize>::new();
    let mut work = 0usize;
    for start in alias_edges.keys() {
        if completed_depths.contains_key(start) {
            continue;
        }
        let mut path = Vec::new();
        let mut active = BTreeSet::new();
        let mut current = start.clone();
        let mut depth: usize;
        loop {
            if let Some(known_depth) = completed_depths.get(&current) {
                depth = *known_depth;
                break;
            }
            if !active.insert(current.clone()) {
                return Err(reconstruction_error(
                    "$.context.type_schemas",
                    "global TypeSchemaUniverse contains a recursive alias expansion cycle",
                ));
            }
            path.push(current.clone());
            work = work.checked_add(1).ok_or_else(|| {
                resource_limit(
                    "$.context.type_schemas",
                    "type expansion work counter overflow",
                )
            })?;
            if work > MAX_TYPE_EXPANSION_WORK || path.len() > MAX_TYPE_EXPANSION_DEPTH {
                return Err(resource_limit(
                    "$.context.type_schemas",
                    "alias expansion work or depth limit exceeded",
                ));
            }
            if let Some(next) = alias_edges.get(&current) {
                current.clone_from(next);
            } else {
                depth = 0;
                break;
            }
        }
        for identity in path.into_iter().rev() {
            depth = depth.checked_add(1).ok_or_else(|| {
                resource_limit("$.context.type_schemas", "type expansion depth overflow")
            })?;
            if depth > MAX_TYPE_EXPANSION_DEPTH {
                return Err(resource_limit(
                    "$.context.type_schemas",
                    "alias expansion depth limit exceeded",
                ));
            }
            completed_depths.insert(identity, depth);
        }
    }
    Ok(())
}

fn named_reference(expression: &CanonicalTypeExpr) -> Option<&str> {
    match expression {
        CanonicalTypeExpr::NamedRef(identity) => Some(identity),
        CanonicalTypeExpr::List(inner)
        | CanonicalTypeExpr::MapString(inner)
        | CanonicalTypeExpr::Option(inner) => named_reference(inner),
        CanonicalTypeExpr::Primitive(_) => None,
    }
}

fn build_module_set(
    declared: &[ModuleProjection],
    assets: &BTreeMap<(String, String), (TypeSchemaRef, CanonicalTypeSchema)>,
) -> Result<CompiledModuleSet, PortableProofError> {
    if declared.len() > MAX_RECONSTRUCTED_MODULES {
        return Err(resource_limit(
            "$.context.compiled_modules",
            "module limit exceeded",
        ));
    }
    let module_type_count = declared
        .iter()
        .try_fold(0usize, |total, module| {
            total.checked_add(module.named_types.len())
        })
        .ok_or_else(|| {
            resource_limit("$.context.compiled_modules", "module type count overflow")
        })?;
    if module_type_count > MAX_RECONSTRUCTED_MODULE_TYPES {
        return Err(resource_limit(
            "$.context.compiled_modules",
            "module type compilation work limit exceeded",
        ));
    }
    let mut module_ids = BTreeSet::new();
    let mut source_rows = Vec::with_capacity(declared.len());
    let mut expected_members = BTreeMap::new();
    let mut cache_material = Vec::new();
    for module in declared {
        if af_idl::SpecificationId::parse(&module.specification_id).is_err()
            || !module_ids.insert(module.specification_id.clone())
        {
            return Err(reconstruction_error(
                "$.context.compiled_modules",
                "invalid or duplicate module SpecificationId",
            ));
        }
        let mut refs = module.named_types.clone();
        refs.sort_by(|a, b| a.type_name.as_bytes().cmp(b.type_name.as_bytes()));
        if refs != module.named_types {
            return Err(reconstruction_error(
                "$.context.compiled_modules",
                "module named type list is not in exact canonical order",
            ));
        }
        let mut schemas = Vec::with_capacity(module.named_types.len());
        let mut member_names = BTreeSet::new();
        append_cache_part(&mut cache_material, module.specification_id.as_bytes())?;
        for reference in &module.named_types {
            if reference.specification_id != module.specification_id
                || !member_names.insert(reference.type_name.clone())
            {
                return Err(reconstruction_error(
                    "$.context.compiled_modules",
                    "module membership contains a foreign or duplicate named type",
                ));
            }
            let (asset_ref, schema) = assets
                .get(&(
                    reference.specification_id.clone(),
                    reference.type_name.clone(),
                ))
                .ok_or_else(|| {
                    reconstruction_error(
                        "$.context.compiled_modules",
                        "module member has no matching exact SchemaAsset",
                    )
                })?;
            if asset_ref != reference {
                return Err(reconstruction_error(
                    "$.context.compiled_modules",
                    "module member TypeSchemaRef differs from its asset pin",
                ));
            }
            append_cache_part(&mut cache_material, reference.specification_id.as_bytes())?;
            append_cache_part(&mut cache_material, reference.type_name.as_bytes())?;
            append_cache_part(
                &mut cache_material,
                reference.schema_digest.algorithm.as_bytes(),
            )?;
            append_cache_part(
                &mut cache_material,
                reference.schema_digest.domain.as_bytes(),
            )?;
            append_cache_part(&mut cache_material, reference.schema_digest.hex.as_bytes())?;
            append_cache_part(
                &mut cache_material,
                &schema.ace1_bytes().map_err(|error| {
                    reconstruction_error("$.context.compiled_modules", error.to_string())
                })?,
            )?;
            schemas.push(schema.clone());
        }
        expected_members.insert(module.specification_id.clone(), module.named_types.clone());
        let source = print_module(&module.specification_id, &schemas)?;
        source_rows.push((module.specification_id.clone(), source));
    }

    if let Some((_, cached)) = MODULE_SET_CACHE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .filter(|(material, _)| *material == cache_material)
    {
        return Ok(cached.as_ref().clone());
    }

    // Every external nominal reference must resolve through a declared exact
    // module row; the compiler then validates imported names and global type
    // acyclicity from the freshly projected source.
    let modules = af_idl::compile_module_set(&source_rows)
        .map_err(|error| reconstruction_error("$.context.compiled_modules", error.to_string()))?;
    if modules.len() != declared.len() {
        return Err(reconstruction_error(
            "$.context.compiled_modules",
            "AFIDL compiler module count differs from declared membership",
        ));
    }
    for module in declared {
        let compiled = modules.module(&module.specification_id).ok_or_else(|| {
            reconstruction_error("$.context.compiled_modules", "compiled module disappeared")
        })?;
        let refs = refs_for_schemas(&compiled.types)?;
        if refs != expected_members[&module.specification_id] {
            return Err(reconstruction_error(
                "$.context.compiled_modules",
                "compiled complete module membership differs from Portable declaration",
            ));
        }
        for schema in &compiled.types {
            let (_, expected) = assets
                .get(&(schema.schema.clone(), schema.name.clone()))
                .ok_or_else(|| {
                    reconstruction_error(
                        "$.context.compiled_modules",
                        "missing compiled schema asset",
                    )
                })?;
            if schema.ace1_bytes().map_err(|error| {
                reconstruction_error("$.context.compiled_modules", error.to_string())
            })? != expected.ace1_bytes().map_err(|error| {
                reconstruction_error("$.context.compiled_modules", error.to_string())
            })? {
                return Err(reconstruction_error(
                    "$.context.compiled_modules",
                    "AFIDL source projection changed canonical named schema bytes",
                ));
            }
        }
    }
    let modules = Arc::new(modules);
    *MODULE_SET_CACHE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) =
        Some((cache_material, Arc::clone(&modules)));
    Ok(modules.as_ref().clone())
}

fn print_module(
    specification_id: &str,
    schemas: &[CanonicalTypeSchema],
) -> Result<String, PortableProofError> {
    let mut external = BTreeSet::new();
    for schema in schemas {
        visit_schema_refs(schema, |identity| {
            let (module, _) = identity.rsplit_once('/').unwrap_or(("", ""));
            if module != specification_id {
                external.insert(module.to_owned());
            }
        });
    }
    let aliases = external
        .into_iter()
        .enumerate()
        .map(|(index, id)| (id, format!("m{}", index + 1)))
        .collect::<BTreeMap<_, _>>();
    let mut source = format!("afidl 1;\nschema {specification_id};\n");
    for (module, alias) in &aliases {
        writeln!(source, "import {module} as {alias};").map_err(|error| {
            reconstruction_error("$.context.compiled_modules", error.to_string())
        })?;
    }
    for schema in schemas {
        source.push_str(&print_schema(schema, specification_id, &aliases)?);
    }
    Ok(source)
}

fn print_schema(
    schema: &CanonicalTypeSchema,
    current: &str,
    aliases: &BTreeMap<String, String>,
) -> Result<String, PortableProofError> {
    let mut output = String::new();
    match &schema.body {
        CanonicalBody::Alias(target) => {
            writeln!(
                output,
                "alias {} = {};",
                schema.name,
                print_expr(target, current, aliases)?
            )
            .map_err(|error| reconstruction_error("$.schema_assets", error.to_string()))?;
        }
        CanonicalBody::Enum(values) => {
            writeln!(output, "enum {} {{", schema.name)
                .map_err(|error| reconstruction_error("$.schema_assets", error.to_string()))?;
            for value in values {
                writeln!(output, "    {} {};", value.no, value.name)
                    .map_err(|error| reconstruction_error("$.schema_assets", error.to_string()))?;
            }
            output.push_str("}\n");
        }
        CanonicalBody::Record(fields) => {
            writeln!(output, "record {} {{", schema.name)
                .map_err(|error| reconstruction_error("$.schema_assets", error.to_string()))?;
            for field in fields {
                writeln!(
                    output,
                    "    {} {}: {};",
                    field.no,
                    field.name,
                    print_expr(&field.type_expr, current, aliases)?
                )
                .map_err(|error| reconstruction_error("$.schema_assets", error.to_string()))?;
            }
            output.push_str("}\n");
        }
        CanonicalBody::Variant(cases) => {
            writeln!(output, "variant {} {{", schema.name)
                .map_err(|error| reconstruction_error("$.schema_assets", error.to_string()))?;
            for case in cases {
                writeln!(
                    output,
                    "    {} {}: {};",
                    case.no,
                    case.name,
                    print_expr(&case.type_expr, current, aliases)?
                )
                .map_err(|error| reconstruction_error("$.schema_assets", error.to_string()))?;
            }
            output.push_str("}\n");
        }
    }
    Ok(output)
}

fn print_expr(
    expression: &CanonicalTypeExpr,
    current: &str,
    aliases: &BTreeMap<String, String>,
) -> Result<String, PortableProofError> {
    Ok(match expression {
        CanonicalTypeExpr::Primitive(primitive) => primitive.keyword().to_owned(),
        CanonicalTypeExpr::NamedRef(identity) => {
            let (module, name) = identity.rsplit_once('/').ok_or_else(|| {
                reconstruction_error("$.schema_assets", "malformed full named type identity")
            })?;
            if module == current {
                name.to_owned()
            } else {
                let alias = aliases.get(module).ok_or_else(|| {
                    reconstruction_error(
                        "$.schema_assets",
                        "external schema reference lacks import",
                    )
                })?;
                format!("{alias}.{name}")
            }
        }
        CanonicalTypeExpr::List(inner) => format!("list<{}>", print_expr(inner, current, aliases)?),
        CanonicalTypeExpr::MapString(inner) => {
            format!("map<string, {}>", print_expr(inner, current, aliases)?)
        }
        CanonicalTypeExpr::Option(inner) => {
            format!("option<{}>", print_expr(inner, current, aliases)?)
        }
    })
}

fn visit_schema_refs(schema: &CanonicalTypeSchema, mut visit: impl FnMut(&str)) {
    fn visit_expr(expression: &CanonicalTypeExpr, visit: &mut impl FnMut(&str)) {
        match expression {
            CanonicalTypeExpr::NamedRef(identity) => visit(identity),
            CanonicalTypeExpr::List(inner)
            | CanonicalTypeExpr::MapString(inner)
            | CanonicalTypeExpr::Option(inner) => visit_expr(inner, visit),
            CanonicalTypeExpr::Primitive(_) => {}
        }
    }
    match &schema.body {
        CanonicalBody::Alias(expression) => visit_expr(expression, &mut visit),
        CanonicalBody::Record(fields) => {
            for field in fields {
                visit_expr(&field.type_expr, &mut visit);
            }
        }
        CanonicalBody::Variant(cases) => {
            for case in cases {
                visit_expr(&case.type_expr, &mut visit);
            }
        }
        CanonicalBody::Enum(_) => {}
    }
}

fn refs_for_schemas(
    schemas: &[CanonicalTypeSchema],
) -> Result<Vec<TypeSchemaRef>, PortableProofError> {
    let mut refs = schemas
        .iter()
        .map(|schema| {
            let digest = schema.digest().map_err(|error| {
                reconstruction_error("$.context.compiled_modules", error.to_string())
            })?;
            Ok(TypeSchemaRef {
                specification_id: schema.schema.clone(),
                type_name: schema.name.clone(),
                schema_digest: af_semantic::DigestRefValue {
                    algorithm: "sha256".to_owned(),
                    domain: "type-schema".to_owned(),
                    hex: digest.hex().to_owned(),
                },
            })
        })
        .collect::<Result<Vec<_>, PortableProofError>>()?;
    refs.sort_by(|a, b| a.type_name.as_bytes().cmp(b.type_name.as_bytes()));
    Ok(refs)
}

fn build_effect_universe(
    catalogs: &[EffectCatalogProjection],
) -> Result<EffectCatalogUniverse, PortableProofError> {
    let mut universe = EffectCatalogUniverse::new();
    let mut identities = BTreeSet::new();
    for catalog in catalogs {
        if !identities.insert(catalog.specification_id.clone()) {
            return Err(reconstruction_error(
                "$.context.effect_catalogs",
                "duplicate catalog id",
            ));
        }
        check_sorted_unique(&catalog.operations, "$.context.effect_catalogs.operations")?;
        let built =
            CapabilityCatalog::new(catalog.specification_id.clone(), catalog.operations.clone())
                .map_err(|error| {
                    reconstruction_error("$.context.effect_catalogs", error.to_string())
                })?;
        universe.insert(&built).map_err(|error| {
            reconstruction_error("$.context.effect_catalogs", error.to_string())
        })?;
    }
    Ok(universe)
}

fn build_resource_universe(
    schemas: &[TypeSchemaRef],
    descriptors: &[ResourceDescriptorProjection],
    assets: &BTreeMap<(String, String), (TypeSchemaRef, CanonicalTypeSchema)>,
) -> Result<ResourceTypeUniverse, PortableProofError> {
    let mut universe = ResourceTypeUniverse::new();
    let mut seen = BTreeSet::new();
    for reference in schemas {
        let key = (
            reference.specification_id.clone(),
            reference.type_name.clone(),
        );
        if !seen.insert(key.clone()) {
            return Err(reconstruction_error(
                "$.context.resource_types.schemas",
                "duplicate resource schema membership",
            ));
        }
        let (asset_ref, schema) = assets.get(&key).ok_or_else(|| {
            reconstruction_error(
                "$.context.resource_types.schemas",
                "resource-private schema has no SchemaAsset",
            )
        })?;
        if asset_ref != reference {
            return Err(reconstruction_error(
                "$.context.resource_types.schemas",
                "resource-private schema pin mismatch",
            ));
        }
        universe.insert_schema_type(schema).map_err(|error| {
            reconstruction_error("$.context.resource_types.schemas", error.to_string())
        })?;
    }
    let mut descriptor_ids = BTreeSet::new();
    for descriptor in descriptors {
        let key = (
            descriptor.schema.specification_id.clone(),
            descriptor.schema.type_name.clone(),
        );
        if !descriptor_ids.insert(key) {
            return Err(reconstruction_error(
                "$.context.resource_types.descriptors",
                "duplicate resource descriptor identity",
            ));
        }
        check_sorted_unique(
            &descriptor.recognized_attributes,
            "$.context.resource_types.descriptors.recognized_attributes",
        )?;
        universe
            .recognize(ResourceTypeDescriptor {
                specification_id: descriptor.schema.specification_id.clone(),
                type_name: descriptor.schema.type_name.clone(),
                schema_hex: descriptor.schema.schema_digest.hex.clone(),
                recognized_attributes: descriptor.recognized_attributes.iter().cloned().collect(),
            })
            .map_err(|error| {
                reconstruction_error("$.context.resource_types.descriptors", error.to_string())
            })?;
        universe
            .resolve(&ResourceTypeRef {
                schema: descriptor.schema.clone(),
            })
            .map_err(|error| {
                reconstruction_error("$.context.resource_types.descriptors", error.to_string())
            })?;
    }
    Ok(universe)
}

fn build_flow_catalogs(
    labels: &[ConfidentialityLabelRef],
    taints: &[TaintRef],
    sinks: &[FlowSinkProjection],
) -> Result<FlowCatalogs, PortableProofError> {
    let mut catalogs = FlowCatalogs::new();
    let mut seen = BTreeSet::new();
    for label in labels {
        if !seen.insert((label.catalog.clone(), label.name.clone())) {
            return Err(reconstruction_error(
                "$.context.flow_catalogs.confidentiality_labels",
                "duplicate label identity",
            ));
        }
        if af_flow::check_label(&label.catalog, &label.name).is_err()
            || label.name.nfc().collect::<String>() != label.name
        {
            return Err(reconstruction_error(
                "$.context.flow_catalogs.confidentiality_labels",
                "invalid or noncanonical label identity",
            ));
        }
        catalogs
            .labels
            .recognize_confidentiality(&label.catalog, &label.name);
    }
    seen.clear();
    for taint in taints {
        if !seen.insert((taint.catalog.clone(), taint.name.clone())) {
            return Err(reconstruction_error(
                "$.context.flow_catalogs.taints",
                "duplicate taint identity",
            ));
        }
        if af_flow::check_label(&taint.catalog, &taint.name).is_err()
            || taint.name.nfc().collect::<String>() != taint.name
        {
            return Err(reconstruction_error(
                "$.context.flow_catalogs.taints",
                "invalid or noncanonical taint identity",
            ));
        }
        catalogs.labels.recognize_taint(&taint.catalog, &taint.name);
    }
    seen.clear();
    for sink in sinks {
        let key = (sink.operation.catalog.clone(), sink.operation.name.clone());
        if !seen.insert(key) {
            return Err(reconstruction_error(
                "$.context.flow_catalogs.effect_sinks",
                "duplicate effect sink identity",
            ));
        }
        catalogs.sinks.recognize_operation(
            &sink.operation.catalog,
            &sink.operation.name,
            sink.decision_capable,
        );
    }
    Ok(catalogs)
}

fn build_usage_meters(
    meters: &[UsageMeterProjection],
) -> Result<UsageMeterUniverse, PortableProofError> {
    let mut universe = UsageMeterUniverse::new();
    let mut seen = BTreeSet::new();
    for meter in meters {
        let key = (meter.meter.catalog.clone(), meter.meter.name.clone());
        if !seen.insert(key) {
            return Err(reconstruction_error(
                "$.context.usage_meters",
                "duplicate usage meter identity",
            ));
        }
        if meter.meter.name.nfc().collect::<String>() != meter.meter.name
            || meter.unit.name.nfc().collect::<String>() != meter.unit.name
        {
            return Err(reconstruction_error(
                "$.context.usage_meters",
                "noncanonical meter or unit identity",
            ));
        }
        let unit = UsageUnitRef::new(meter.unit.catalog.clone(), meter.unit.name.clone())
            .map_err(|error| reconstruction_error("$.context.usage_meters", error.to_string()))?;
        universe
            .recognize(
                &meter.meter,
                &unit,
                match meter.aggregation {
                    PortableUsageAggregation::Additive => UsageAggregation::Additive,
                    PortableUsageAggregation::Unsupported => UsageAggregation::Unsupported,
                },
            )
            .map_err(|error| reconstruction_error("$.context.usage_meters", error.to_string()))?;
    }
    Ok(universe)
}

fn build_recovery_universe(
    catalogs: &[EffectCatalogProjection],
    operations: &[NamedOperationProjection],
) -> Result<RecoveryOperationUniverse, PortableProofError> {
    let mut recovery = RecoveryOperationUniverse::new();
    recovery.effects = build_effect_universe(catalogs)?;
    let mut seen = BTreeSet::new();
    for operation in operations {
        if !seen.insert((operation.catalog.clone(), operation.name.clone())) {
            return Err(reconstruction_error(
                "$.context.recovery_operations.reconciliation_operations",
                "duplicate reconciliation operation",
            ));
        }
        recovery
            .recognize_reconciliation_operation(operation.catalog.clone(), operation.name.clone())
            .map_err(|error| {
                reconstruction_error(
                    "$.context.recovery_operations.reconciliation_operations",
                    error.to_string(),
                )
            })?;
    }
    Ok(recovery)
}

fn build_execution_universes(
    projection: &StaticContextProjection,
) -> Result<ExecutionUniverses, PortableProofError> {
    let mut result = ExecutionUniverses::default();
    let mut capability_ids = BTreeSet::new();
    for capability in &projection.execution_capabilities {
        if !capability_ids.insert((
            capability.capability.catalog.clone(),
            capability.capability.name.clone(),
        )) {
            return Err(reconstruction_error(
                "$.context.execution_universes.capabilities",
                "duplicate capability identity",
            ));
        }
        check_sorted_unique(
            &capability.supported_versions,
            "$.context.execution_universes.capabilities.supported_versions",
        )?;
        if !capability
            .supported_versions
            .contains(&capability.exact_version)
        {
            return Err(reconstruction_error(
                "$.context.execution_universes.capabilities",
                "exact_version is absent from its supported version set",
            ));
        }
        let mut attrs = BTreeSet::new();
        let mut attr_keys = BTreeSet::new();
        for attribute in &capability.attributes {
            if !attrs.insert((attribute.key.clone(), attribute.value.clone()))
                || !attr_keys.insert(attribute.key.clone())
            {
                return Err(reconstruction_error(
                    "$.context.execution_universes.capabilities.attributes",
                    "duplicate attribute key or pair",
                ));
            }
        }
        let mut versions = capability.supported_versions.clone();
        versions.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        versions.dedup();
        let mut ordered = Vec::with_capacity(versions.len());
        ordered.push(capability.exact_version.clone());
        ordered.extend(
            versions
                .into_iter()
                .filter(|version| version != &capability.exact_version),
        );
        for version in ordered {
            result
                .capabilities
                .recognize(
                    &capability.capability,
                    version,
                    capability.attributes.clone(),
                )
                .map_err(|error| {
                    reconstruction_error(
                        "$.context.execution_universes.capabilities",
                        error.to_string(),
                    )
                })?;
        }
    }

    let mut trusts = BTreeSet::new();
    for domain in &projection.trust_domains {
        if !trusts.insert((domain.catalog.clone(), domain.name.clone())) {
            return Err(reconstruction_error(
                "$.context.execution_universes.trust_domains",
                "duplicate trust domain",
            ));
        }
        if domain.name.nfc().collect::<String>() != domain.name {
            return Err(reconstruction_error(
                "$.context.execution_universes.trust_domains",
                "noncanonical trust-domain name",
            ));
        }
        result
            .trust_domains
            .recognize(domain.clone())
            .map_err(|error| {
                reconstruction_error(
                    "$.context.execution_universes.trust_domains",
                    error.to_string(),
                )
            })?;
    }

    let mut regions = BTreeSet::new();
    for region in &projection.regions {
        if !regions.insert((region.catalog.clone(), region.name.clone())) {
            return Err(reconstruction_error(
                "$.context.execution_universes.regions",
                "duplicate region",
            ));
        }
        if region.name.nfc().collect::<String>() != region.name {
            return Err(reconstruction_error(
                "$.context.execution_universes.regions",
                "noncanonical region name",
            ));
        }
        result.regions.recognize(region.clone()).map_err(|error| {
            reconstruction_error("$.context.execution_universes.regions", error.to_string())
        })?;
    }
    let mut pairs = BTreeSet::new();
    for pair in &projection.region_compatibility {
        if !pairs.insert((pair.from.clone(), pair.to.clone()))
            || !regions.contains(&(pair.from.catalog.clone(), pair.from.name.clone()))
            || !regions.contains(&(pair.to.catalog.clone(), pair.to.name.clone()))
        {
            return Err(reconstruction_error(
                "$.context.execution_universes.region_compatibility",
                "duplicate or out-of-snapshot directed region pair",
            ));
        }
        result
            .regions
            .recognize_compatibility(pair.from.clone(), pair.to.clone())
            .map_err(|error| {
                reconstruction_error(
                    "$.context.execution_universes.region_compatibility",
                    error.to_string(),
                )
            })?;
    }

    let mut profiles = BTreeSet::new();
    for profile in &projection.runtime_profiles {
        let key = (
            profile.profile.catalog.clone(),
            profile.profile.name.clone(),
        );
        if !profiles.insert(key) {
            return Err(reconstruction_error(
                "$.context.execution_universes.runtime_profiles",
                "duplicate runtime profile",
            ));
        }
        check_sorted_unique(
            &profile.features,
            "$.context.execution_universes.runtime_profiles.features",
        )?;
        result
            .runtime_profiles
            .recognize(
                profile.profile.clone(),
                profile.family.clone(),
                profile.features.clone(),
            )
            .map_err(|error| {
                reconstruction_error(
                    "$.context.execution_universes.runtime_profiles",
                    error.to_string(),
                )
            })?;
    }
    Ok(result)
}

fn build_relations(
    projection: &StaticContextProjection,
    modules: &CompiledModuleSet,
) -> Result<RelationSnapshots, PortableProofError> {
    Ok(RelationSnapshots {
        effect_resource: projection
            .effect_resource_relation
            .as_ref()
            .map(|relation| {
                build_relation::<EffectResourceCompatibilityCatalog>(
                    relation,
                    modules,
                    "effect_resource",
                )
            })
            .transpose()?,
        recovery_usage: projection
            .recovery_usage_relation
            .as_ref()
            .map(|relation| {
                build_relation::<RecoveryUsageCoverageCatalog>(relation, modules, "recovery_usage")
            })
            .transpose()?,
        flow_execution: projection
            .flow_execution_relation
            .as_ref()
            .map(|relation| {
                build_relation::<InformationFlowExecutionCatalog>(
                    relation,
                    modules,
                    "flow_execution",
                )
            })
            .transpose()?,
    })
}

fn build_relation<T>(
    relation: &RelationProjection,
    modules: &CompiledModuleSet,
    slot: &'static str,
) -> Result<RelationCatalogSnapshot<T>, PortableProofError>
where
    T: af_contract::RelationCatalogWire + for<'de> Deserialize<'de>,
{
    if relation.canonical_content_checksum.algorithm != "sha256"
        || relation.canonical_content_checksum.hex != sha256_hex(&relation.canonical_content)
    {
        return Err(reconstruction_error(
            format!("$.context.relation_snapshots.{slot}"),
            "relation content checksum mismatch",
        ));
    }
    let schema_id = T::SCHEMA_ID;
    let root_type = T::ROOT_TYPE;
    if relation.schema.specification_id != schema_id || relation.schema.type_name != root_type {
        return Err(reconstruction_error(
            format!("$.context.relation_snapshots.{slot}.header.schema"),
            "relation header schema identity differs from the formal catalog schema",
        ));
    }
    let schema = modules
        .type_by_identity(&format!("{schema_id}/{root_type}"))
        .ok_or_else(|| {
            reconstruction_error(
                "$.context.relation_snapshots",
                "relation schema is not in the exact compiled module set",
            )
        })?;
    let digest = schema
        .digest()
        .map_err(|error| reconstruction_error("$.context.relation_snapshots", error.to_string()))?;
    if relation.schema.schema_digest.algorithm != "sha256"
        || relation.schema.schema_digest.domain != "type-schema"
        || relation.schema.schema_digest.hex != digest.hex()
    {
        return Err(reconstruction_error(
            format!("$.context.relation_snapshots.{slot}.header.schema"),
            "relation root schema pin is not the exact compiled TypeSchemaDigest",
        ));
    }
    let shape = modules
        .shape_of(&format!("{schema_id}/{root_type}"))
        .map_err(|error| reconstruction_error("$.context.relation_snapshots", error.to_string()))?;
    let value = decode(&shape, &relation.canonical_content)
        .map_err(|error| reconstruction_error("$.context.relation_snapshots", error.to_string()))?;
    let json = value_to_json(&value)
        .map_err(|error| reconstruction_error("$.context.relation_snapshots", error))?;
    let catalog: T = serde_json::from_value(json)
        .map_err(|error| reconstruction_error("$.context.relation_snapshots", error.to_string()))?;
    let snapshot = RelationCatalogSnapshot::new(relation.revision.clone(), catalog, modules)
        .map_err(|error| reconstruction_error("$.context.relation_snapshots", error))?;
    if snapshot.canonical_bytes() != relation.canonical_content
        || snapshot.pin().schema != schema_id
        || snapshot.pin().revision != relation.revision
        || snapshot.pin().canonical_content_checksum.algorithm != "sha256"
        || snapshot.pin().canonical_content_checksum.hex != relation.canonical_content_checksum.hex
    {
        return Err(reconstruction_error(
            format!("$.context.relation_snapshots.{slot}"),
            "typed relation snapshot re-encoding or pin differs from the declared content",
        ));
    }
    Ok(snapshot)
}

#[allow(clippy::too_many_arguments)]
fn recapture_projection(
    context_id: u32,
    modules: &CompiledModuleSet,
    schemas: &TypeSchemaUniverse,
    effects: &EffectCatalogUniverse,
    resources: &ResourceTypeUniverse,
    flow: &FlowCatalogs,
    usage: &UsageMeterUniverse,
    recovery: &RecoveryOperationUniverse,
    execution: &ExecutionUniverses,
    relations: &RelationSnapshots,
) -> Result<StaticContextProjection, PortableProofError> {
    let module_rows = modules
        .modules()
        .map(|module| {
            Ok(ModuleProjection {
                specification_id: module.specification.canonical(),
                named_types: refs_for_schemas(&module.types)?,
            })
        })
        .collect::<Result<Vec<_>, PortableProofError>>()?;
    let mut type_schemas = schemas
        .entries()
        .map(af_semantic::SchemaEntry::reference)
        .collect::<Vec<_>>();
    type_schemas.sort_by(schema_ref_key);
    let effect_catalogs = project_effects(effects);
    let resource_schemas = resources
        .schema_universe()
        .entries()
        .map(af_semantic::SchemaEntry::reference)
        .collect::<Vec<_>>();
    let resource_descriptors = resources
        .descriptors()
        .map(|descriptor| {
            let schema = resources
                .schema_universe()
                .get(&format!(
                    "{}/{}",
                    descriptor.specification_id, descriptor.type_name
                ))
                .map(af_semantic::SchemaEntry::reference)
                .ok_or_else(|| {
                    reconstruction_error(
                        "$.context.resource_types",
                        "descriptor schema disappeared from private universe",
                    )
                })?;
            Ok(ResourceDescriptorProjection {
                schema,
                recognized_attributes: descriptor.recognized_attributes.iter().cloned().collect(),
            })
        })
        .collect::<Result<Vec<_>, PortableProofError>>()?;
    let confidentiality_labels = flow
        .labels
        .confidentiality_labels()
        .map(|(catalog, name)| ConfidentialityLabelRef {
            catalog: catalog.to_owned(),
            name: name.to_owned(),
        })
        .collect();
    let taints = flow
        .labels
        .taints()
        .map(|(catalog, name)| TaintRef {
            catalog: catalog.to_owned(),
            name: name.to_owned(),
        })
        .collect();
    let flow_effect_sinks = flow
        .sinks
        .operations()
        .map(|(catalog, name, decision_capable)| FlowSinkProjection {
            operation: af_effect::EffectOperationRef {
                catalog: catalog.to_owned(),
                name: name.to_owned(),
            },
            decision_capable,
        })
        .collect();
    let usage_meters = usage
        .descriptors()
        .map(|descriptor| UsageMeterProjection {
            meter: descriptor.meter.clone(),
            unit: UsageUnitProjection {
                catalog: descriptor.unit.catalog.clone(),
                name: descriptor.unit.name.clone(),
            },
            aggregation: match descriptor.aggregation {
                UsageAggregation::Additive => PortableUsageAggregation::Additive,
                UsageAggregation::Unsupported => PortableUsageAggregation::Unsupported,
            },
        })
        .collect();
    let recovery_effect_catalogs = project_effects(&recovery.effects);
    let reconciliation_operations = recovery
        .reconciliation_operations()
        .map(|(catalog, name)| NamedOperationProjection {
            catalog: catalog.to_owned(),
            name: name.to_owned(),
        })
        .collect();
    let execution_capabilities = execution
        .capabilities
        .capabilities
        .values()
        .map(|descriptor| ExecutionCapabilityProjection {
            capability: descriptor.capability.clone(),
            exact_version: descriptor.exact_version.clone(),
            supported_versions: descriptor.supported_versions.iter().cloned().collect(),
            attributes: descriptor
                .attributes
                .iter()
                .map(|(key, value)| CapabilityAttribute {
                    key: key.clone(),
                    value: value.clone(),
                })
                .collect(),
        })
        .collect();
    let trust_domains = execution.trust_domains.domains().cloned().collect();
    let regions = execution.regions.regions().cloned().collect();
    let region_compatibility = execution
        .regions
        .compatibility()
        .map(|(from, to)| RegionCompatibilityProjection {
            from: from.clone(),
            to: to.clone(),
        })
        .collect();
    let runtime_profiles = execution
        .runtime_profiles
        .profiles()
        .map(|descriptor| RuntimeProfileProjection {
            profile: descriptor.profile.clone(),
            family: descriptor.family.clone(),
            features: descriptor.features.iter().cloned().collect(),
        })
        .collect();
    Ok(StaticContextProjection {
        context_id,
        modules: module_rows,
        type_schemas,
        effect_catalogs,
        resource_schemas,
        resource_descriptors,
        confidentiality_labels,
        taints,
        flow_effect_sinks,
        usage_meters,
        recovery_effect_catalogs,
        reconciliation_operations,
        execution_capabilities,
        trust_domains,
        regions,
        region_compatibility,
        runtime_profiles,
        effect_resource_relation: project_relation::<EffectResourceCompatibilityCatalog>(
            relations.effect_resource.as_ref(),
            modules,
        )?,
        recovery_usage_relation: project_relation::<RecoveryUsageCoverageCatalog>(
            relations.recovery_usage.as_ref(),
            modules,
        )?,
        flow_execution_relation: project_relation::<InformationFlowExecutionCatalog>(
            relations.flow_execution.as_ref(),
            modules,
        )?,
    })
}

fn project_effects(universe: &EffectCatalogUniverse) -> Vec<EffectCatalogProjection> {
    universe
        .catalogs()
        .map(|catalog| EffectCatalogProjection {
            specification_id: catalog.specification_id().to_owned(),
            operations: catalog.operations().map(str::to_owned).collect(),
        })
        .collect()
}

fn project_relation<T: af_contract::RelationCatalogWire>(
    snapshot: Option<&RelationCatalogSnapshot<T>>,
    modules: &CompiledModuleSet,
) -> Result<Option<RelationProjection>, PortableProofError> {
    snapshot
        .map(|snapshot| {
            let schema = modules
                .type_by_identity(&format!("{}/{}", T::SCHEMA_ID, T::ROOT_TYPE))
                .ok_or_else(|| {
                    reconstruction_error(
                        "$.context.relation_snapshots",
                        "relation schema missing during recapture",
                    )
                })?;
            let digest = schema.digest().map_err(|error| {
                reconstruction_error("$.context.relation_snapshots", error.to_string())
            })?;
            Ok(RelationProjection {
                schema: TypeSchemaRef {
                    specification_id: T::SCHEMA_ID.to_owned(),
                    type_name: T::ROOT_TYPE.to_owned(),
                    schema_digest: af_semantic::DigestRefValue {
                        algorithm: "sha256".into(),
                        domain: "type-schema".into(),
                        hex: digest.hex().into(),
                    },
                },
                revision: snapshot.revision().to_owned(),
                canonical_content_checksum: PlainChecksumProjection {
                    algorithm: snapshot
                        .pin()
                        .canonical_content_checksum
                        .algorithm
                        .to_owned(),
                    hex: snapshot.pin().canonical_content_checksum.hex.clone(),
                },
                canonical_content: snapshot.canonical_bytes().to_vec(),
            })
        })
        .transpose()
}

fn check_reconstruction_limits(
    projection: &StaticContextProjection,
) -> Result<(), PortableProofError> {
    let schema_count = projection
        .type_schemas
        .len()
        .checked_add(projection.resource_schemas.len())
        .ok_or_else(|| resource_limit("$.context", "schema count overflow"))?;
    if schema_count > MAX_RECONSTRUCTED_SCHEMAS.saturating_mul(2) {
        return Err(resource_limit(
            "$.context",
            "global and resource schema work limit exceeded",
        ));
    }
    let catalog_count = projection
        .effect_catalogs
        .len()
        .checked_add(projection.recovery_effect_catalogs.len())
        .and_then(|value| value.checked_add(projection.usage_meters.len()))
        .and_then(|value| value.checked_add(projection.execution_capabilities.len()))
        .ok_or_else(|| resource_limit("$.context", "catalog row count overflow"))?;
    if catalog_count > MAX_RECONSTRUCTED_CATALOG_ROWS {
        return Err(resource_limit(
            "$.context",
            "catalog construction work limit exceeded",
        ));
    }
    Ok(())
}

fn complete_vec<T: for<'de> Deserialize<'de>>(
    value: &Value,
    path: &str,
) -> Result<Vec<T>, PortableProofError> {
    let Value::Variant { case, value } = value else {
        return Err(reconstruction_error(
            path,
            "expected CompleteCollection variant",
        ));
    };
    if case == "unavailable" {
        return Err(PortableProofError::new(
            ValidationStage::D5,
            "IncompleteContext",
            path,
            "required static proof context collection is explicitly Unavailable",
        ));
    }
    if case != "complete" {
        return Err(reconstruction_error(
            path,
            "unknown collection completeness case",
        ));
    }
    let Value::List(items) = value.as_ref() else {
        return Err(reconstruction_error(path, "Complete payload is not a list"));
    };
    if items.len() > MAX_RECONSTRUCTED_CATALOG_ROWS {
        return Err(resource_limit(
            path,
            "collection construction work limit exceeded",
        ));
    }
    items.iter().map(|item| from_value(item, path)).collect()
}

fn from_value<T: for<'de> Deserialize<'de>>(
    value: &Value,
    path: &str,
) -> Result<T, PortableProofError> {
    let value = value_to_json(value).map_err(|detail| reconstruction_error(path, detail))?;
    serde_json::from_value(value).map_err(|error| reconstruction_error(path, error.to_string()))
}

pub(crate) fn value_to_json(value: &Value) -> Result<serde_json::Value, String> {
    Ok(match value {
        Value::Unit | Value::Null | Value::Option(None) => serde_json::Value::Null,
        Value::Bool(value) => serde_json::Value::Bool(*value),
        Value::Int32(value) => (*value).into(),
        Value::UInt32(value) => (*value).into(),
        Value::Int64(value) => (*value).into(),
        Value::UInt64(value) => (*value).into(),
        Value::Decimal(value) => serde_json::Value::String(value.to_string()),
        Value::Float64(value) => serde_json::Number::from_f64(value.get())
            .map(serde_json::Value::Number)
            .ok_or_else(|| "non-finite float in validated typed value".to_owned())?,
        Value::String(value) | Value::Enum(value) => serde_json::Value::String(value.clone()),
        Value::Bytes(bytes) => {
            serde_json::Value::Array(bytes.iter().copied().map(Into::into).collect())
        }
        Value::Timestamp(value) => value.as_micros().into(),
        Value::Duration(value) => value.as_micros().into(),
        Value::List(values) => serde_json::Value::Array(
            values
                .iter()
                .map(value_to_json)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        Value::Option(Some(value)) => value_to_json(value)?,
        Value::Record(fields) | Value::Map(fields) => {
            let mut object = serde_json::Map::new();
            for (name, value) in fields {
                if object.insert(name.clone(), value_to_json(value)?).is_some() {
                    return Err("duplicate field in validated typed value".to_owned());
                }
            }
            serde_json::Value::Object(object)
        }
        Value::Variant { case, value } => serde_json::Value::Object(
            [(case.clone(), value_to_json(value)?)]
                .into_iter()
                .collect(),
        ),
    })
}

fn schema_ref_key(left: &TypeSchemaRef, right: &TypeSchemaRef) -> std::cmp::Ordering {
    (
        left.specification_id.as_bytes(),
        left.type_name.as_bytes(),
        left.schema_digest.algorithm.as_bytes(),
        left.schema_digest.domain.as_bytes(),
        left.schema_digest.hex.as_bytes(),
    )
        .cmp(&(
            right.specification_id.as_bytes(),
            right.type_name.as_bytes(),
            right.schema_digest.algorithm.as_bytes(),
            right.schema_digest.domain.as_bytes(),
            right.schema_digest.hex.as_bytes(),
        ))
}

fn check_sorted_unique<T: Ord>(values: &[T], path: &str) -> Result<(), PortableProofError> {
    if values.windows(2).any(|items| items[0] >= items[1]) {
        return Err(reconstruction_error(
            path,
            "members must be sorted and unique before constructor normalization",
        ));
    }
    Ok(())
}

fn field<'a>(value: &'a Value, name: &str, path: &str) -> Result<&'a Value, PortableProofError> {
    let Value::Record(fields) = value else {
        return Err(reconstruction_error(path, "expected typed record"));
    };
    fields
        .iter()
        .find(|(field, _)| field == name)
        .map(|(_, value)| value)
        .ok_or_else(|| reconstruction_error(path, format!("missing field {name}")))
}

fn list_field<'a>(
    value: &'a Value,
    name: &str,
    path: &str,
) -> Result<&'a [Value], PortableProofError> {
    match field(value, name, path)? {
        Value::List(items) => Ok(items),
        _ => Err(reconstruction_error(path, format!("{name} is not a list"))),
    }
}

fn string_field<'a>(
    value: &'a Value,
    name: &str,
    path: &str,
) -> Result<&'a str, PortableProofError> {
    match field(value, name, path)? {
        Value::String(text) => Ok(text),
        _ => Err(reconstruction_error(
            path,
            format!("{name} is not a string"),
        )),
    }
}

fn enum_or_string_field<'a>(
    value: &'a Value,
    name: &str,
    path: &str,
) -> Result<&'a str, PortableProofError> {
    match field(value, name, path)? {
        Value::String(text) | Value::Enum(text) => Ok(text),
        _ => Err(reconstruction_error(
            path,
            format!("{name} is not a text enum"),
        )),
    }
}

fn bytes_field<'a>(
    value: &'a Value,
    name: &str,
    path: &str,
) -> Result<&'a [u8], PortableProofError> {
    match field(value, name, path)? {
        Value::Bytes(bytes) => Ok(bytes),
        _ => Err(reconstruction_error(path, format!("{name} is not bytes"))),
    }
}

fn u32_field(value: &Value, name: &str) -> Result<u32, PortableProofError> {
    match field(value, name, "$")? {
        Value::UInt32(number) => Ok(*number),
        _ => Err(reconstruction_error("$", format!("{name} is not uint32"))),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn reconstruction_error(path: impl Into<String>, detail: impl Into<String>) -> PortableProofError {
    PortableProofError::new(ValidationStage::D5, "ReconstructionMismatch", path, detail)
}

fn resource_limit(path: impl Into<String>, detail: impl Into<String>) -> PortableProofError {
    PortableProofError::new(ValidationStage::D5, "ResourceLimit", path, detail)
}
