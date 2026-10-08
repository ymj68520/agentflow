//! Canonical Execution Requirements.
use crate::error::{ExecutionError, kind};
use crate::model::{
    CapabilityRequirement, ComputeQuantityKind, ComputeQuantityRequirement, ExecutionRequirements,
    ImplementationAbiRequirement, PlacementPreference, RegionRef, RegionResidencyRequirement,
    RuntimeProfileRef, RuntimeProfileRequirement, SandboxLevel, SandboxRequirement, TrustDomainRef,
};
use crate::universe::ExecutionUniverses;
use af_core::{Value, encode};
use af_idl::CompiledModuleSet;
use af_semantic::TypeSchemaRef;
use std::collections::BTreeSet;
use unicode_normalization::UnicodeNormalization;

pub fn canonicalize_execution_requirements(
    req: &ExecutionRequirements,
    u: &ExecutionUniverses,
) -> Result<ExecutionRequirements, ExecutionError> {
    let mut caps = Vec::new();
    let mut capkeys = BTreeSet::new();
    for c in &req.capabilities {
        let capability = normalize_capability(c.capability.clone());
        let cap = u.capabilities.resolve(&capability)?;
        let exact_version: String = c.exact_version.nfc().collect();
        if !u
            .capabilities
            .supports_version(&capability, &exact_version)?
        {
            return Err(ExecutionError::new(
                kind::UNSUPPORTED_CAPABILITY_VERSION,
                "capability version is not supported",
            ));
        }
        let mut attrs = c.required_attributes.clone();
        for a in &mut attrs {
            a.key = a.key.nfc().collect();
            a.value = a.value.nfc().collect();
            if a.key.is_empty() || a.value.is_empty() {
                return Err(ExecutionError::new(
                    kind::INVALID_EXECUTION_CAPABILITY,
                    "empty capability attribute",
                ));
            }
        }
        attrs.sort();
        if attrs.iter().any(|attribute| {
            !cap.attributes
                .contains(&(attribute.key.clone(), attribute.value.clone()))
        }) {
            return Err(ExecutionError::new(
                kind::CAPABILITY_REQUIREMENT_UNPROVEN,
                "capability attribute is outside the explicit vocabulary",
            ));
        }
        if !capkeys.insert((capability.clone(), exact_version.clone())) {
            return Err(ExecutionError::new(
                kind::DUPLICATE_EXECUTION_REQUIREMENT,
                "duplicate capability requirement",
            ));
        }
        caps.push(CapabilityRequirement {
            capability,
            exact_version,
            required_attributes: attrs,
        });
    }
    caps.sort_by(|a, b| {
        (&a.capability.catalog, &a.capability.name, &a.exact_version).cmp(&(
            &b.capability.catalog,
            &b.capability.name,
            &b.exact_version,
        ))
    });
    let mut quantities = Vec::new();
    let mut qkeys = BTreeSet::new();
    for q in &req.quantities {
        if !qkeys.insert(q.kind) {
            return Err(ExecutionError::new(
                kind::DUPLICATE_EXECUTION_REQUIREMENT,
                "duplicate compute quantity",
            ));
        }
        quantities.push(q.clone());
    }
    quantities.sort_by_key(|q| q.kind.value_no());
    let sandbox = req.sandbox.clone();
    let trust = req
        .trust_domain
        .clone()
        .map(|x| {
            if u.trust_domains.resolves(&x)? {
                Ok(normalize_trust(x))
            } else {
                Err(ExecutionError::new(
                    kind::UNKNOWN_TRUST_DOMAIN,
                    "trust domain is not registered",
                ))
            }
        })
        .transpose()?;
    let region = req
        .region_residency
        .as_ref()
        .map(|r| canonical_regions(r, u))
        .transpose()?;
    let runtime = req
        .runtime_profile
        .as_ref()
        .map(|r| canonical_runtime(r, u))
        .transpose()?;
    let mut abis = req.implementation_abis.clone();
    let mut abi_keys = BTreeSet::new();
    for abi in &abis {
        validate_abi(&abi.schema)?;
        if !abi_keys.insert(abi_key(&abi.schema)) {
            return Err(ExecutionError::new(
                kind::DUPLICATE_EXECUTION_REQUIREMENT,
                "duplicate implementation ABI requirement",
            ));
        }
    }
    abis.sort_by_key(|x| abi_key(&x.schema));
    let mut prefs = req.soft_preferences.clone();
    prefs.sort_by_key(|p| p.value_no());
    prefs.dedup();
    Ok(ExecutionRequirements {
        capabilities: caps,
        quantities,
        sandbox,
        trust_domain: trust,
        region_residency: region,
        runtime_profile: runtime,
        implementation_abis: abis,
        soft_preferences: prefs,
    })
}
fn normalize_trust(mut x: TrustDomainRef) -> TrustDomainRef {
    x.name = x.name.nfc().collect();
    x
}
fn normalize_capability(
    mut x: crate::model::ExecutionCapabilityRef,
) -> crate::model::ExecutionCapabilityRef {
    x.name = x.name.nfc().collect();
    x
}
fn normalize_region(mut x: RegionRef) -> RegionRef {
    x.name = x.name.nfc().collect();
    x
}
fn canonical_regions(
    r: &RegionResidencyRequirement,
    u: &ExecutionUniverses,
) -> Result<RegionResidencyRequirement, ExecutionError> {
    let mut e: Vec<_> = r
        .execution_regions
        .iter()
        .cloned()
        .map(normalize_region)
        .collect();
    let mut d: Vec<_> = r
        .residency_regions
        .iter()
        .cloned()
        .map(normalize_region)
        .collect();
    for x in e.iter().chain(d.iter()) {
        if !u.regions.resolves(x)? {
            return Err(ExecutionError::new(
                kind::UNKNOWN_REGION,
                "region is not registered",
            ));
        }
    }
    e.sort();
    d.sort();
    e.dedup();
    d.dedup();
    Ok(RegionResidencyRequirement {
        execution_regions: e,
        residency_regions: d,
    })
}
fn canonical_runtime(
    r: &RuntimeProfileRequirement,
    u: &ExecutionUniverses,
) -> Result<RuntimeProfileRequirement, ExecutionError> {
    let profile = normalize_runtime_profile(r.profile.clone());
    let family: String = r.family.nfc().collect();
    let p = u.runtime_profiles.resolve(&profile)?;
    if p.family != family {
        return Err(ExecutionError::new(
            kind::UNKNOWN_RUNTIME_PROFILE,
            "runtime family mismatch",
        ));
    }
    let mut f: Vec<String> = r
        .required_features
        .iter()
        .map(|feature| feature.nfc().collect())
        .collect();
    f.sort();
    f.dedup();
    if !f.iter().all(|x| p.features.contains(x)) {
        return Err(ExecutionError::new(
            kind::UNKNOWN_RUNTIME_PROFILE,
            "runtime feature unavailable",
        ));
    }
    Ok(RuntimeProfileRequirement {
        family,
        profile,
        required_features: f,
    })
}
fn normalize_runtime_profile(mut x: RuntimeProfileRef) -> RuntimeProfileRef {
    x.name = x.name.nfc().collect();
    x
}
pub fn validate_abi(abi: &TypeSchemaRef) -> Result<(), ExecutionError> {
    if af_idl::SpecificationId::parse(&abi.specification_id).is_err()
        || abi.type_name.is_empty()
        || abi.type_name.chars().any(char::is_whitespace)
    {
        return Err(ExecutionError::new(
            kind::RUNTIME_ABI_INCOMPATIBLE,
            "implementation ABI has an invalid schema identity",
        ));
    }
    abi.schema_digest
        .validate_for_type_schema()
        .map_err(|error| {
            ExecutionError::new(
                kind::RUNTIME_ABI_INCOMPATIBLE,
                format!("implementation ABI has an invalid schema digest: {error}"),
            )
        })
}

fn abi_key(abi: &TypeSchemaRef) -> (String, String, String, String, String) {
    (
        abi.specification_id.clone(),
        abi.type_name.clone(),
        abi.schema_digest.algorithm.clone(),
        abi.schema_digest.domain.clone(),
        abi.schema_digest.hex.clone(),
    )
}

impl ExecutionRequirements {
    /// Returns the canonical declaration resolved against explicit universes.
    pub fn canonical(&self, universes: &ExecutionUniverses) -> Result<Self, ExecutionError> {
        canonicalize_execution_requirements(self, universes)
    }

    /// Returns canonical ACE1 bytes. The bytes are an encoding of the
    /// declaration only; this phase deliberately creates no digest.
    pub fn canonical_bytes(
        &self,
        universes: &ExecutionUniverses,
        modules: &CompiledModuleSet,
    ) -> Result<Vec<u8>, ExecutionError> {
        let shape = modules
            .shape_of("agentflow.execution.requirements@1/ExecutionRequirements")
            .map_err(|error| {
                ExecutionError::new(
                    kind::EXECUTION_ENCODING_ERROR,
                    format!("cannot derive execution requirements shape: {error}"),
                )
            })?;
        shape.validate().map_err(|error| {
            ExecutionError::new(
                kind::EXECUTION_ENCODING_ERROR,
                format!("derived execution requirements shape is invalid: {error}"),
            )
        })?;
        encode(&shape, &self.to_value(universes)?).map_err(|error| {
            ExecutionError::new(
                kind::EXECUTION_ENCODING_ERROR,
                format!("cannot encode execution requirements: {error}"),
            )
        })
    }

    /// Builds the schema-shaped ACE1 value tree for the canonical declaration.
    pub fn to_value(&self, universes: &ExecutionUniverses) -> Result<Value, ExecutionError> {
        let c = canonicalize_execution_requirements(self, universes)?;
        Ok(Value::Record(vec![
            (
                "capabilities".into(),
                Value::List(c.capabilities.iter().map(capability_value).collect()),
            ),
            (
                "quantities".into(),
                Value::List(c.quantities.iter().map(quantity_value).collect()),
            ),
            (
                "sandbox".into(),
                option_value(c.sandbox.as_ref().map(sandbox_value)),
            ),
            (
                "trust_domain".into(),
                option_value(c.trust_domain.as_ref().map(named_value)),
            ),
            (
                "region_residency".into(),
                option_value(c.region_residency.as_ref().map(region_residency_value)),
            ),
            (
                "runtime_profile".into(),
                option_value(c.runtime_profile.as_ref().map(runtime_value)),
            ),
            (
                "implementation_abis".into(),
                Value::List(c.implementation_abis.iter().map(abi_value).collect()),
            ),
            (
                "soft_preferences".into(),
                Value::List(
                    c.soft_preferences
                        .iter()
                        .map(|p| Value::Enum(preference_name(*p).into()))
                        .collect(),
                ),
            ),
        ]))
    }
}

fn option_value(value: Option<Value>) -> Value {
    Value::Option(value.map(Box::new))
}

fn named_value(reference: &TrustDomainRef) -> Value {
    Value::Record(vec![
        ("catalog".into(), Value::String(reference.catalog.clone())),
        ("name".into(), Value::String(reference.name.clone())),
    ])
}

fn region_value(reference: &RegionRef) -> Value {
    Value::Record(vec![
        ("catalog".into(), Value::String(reference.catalog.clone())),
        ("name".into(), Value::String(reference.name.clone())),
    ])
}

fn capability_value(requirement: &CapabilityRequirement) -> Value {
    Value::Record(vec![
        (
            "capability".into(),
            Value::Record(vec![
                (
                    "catalog".into(),
                    Value::String(requirement.capability.catalog.clone()),
                ),
                (
                    "name".into(),
                    Value::String(requirement.capability.name.clone()),
                ),
            ]),
        ),
        (
            "exact_version".into(),
            Value::String(requirement.exact_version.clone()),
        ),
        (
            "required_attributes".into(),
            Value::List(
                requirement
                    .required_attributes
                    .iter()
                    .map(|a| {
                        Value::Record(vec![
                            ("key".into(), Value::String(a.key.clone())),
                            ("value".into(), Value::String(a.value.clone())),
                        ])
                    })
                    .collect(),
            ),
        ),
    ])
}

fn quantity_value(requirement: &ComputeQuantityRequirement) -> Value {
    Value::Record(vec![
        (
            "kind".into(),
            Value::Enum(quantity_name(requirement.kind).into()),
        ),
        ("minimum".into(), Value::UInt64(requirement.minimum)),
    ])
}

fn sandbox_value(requirement: &SandboxRequirement) -> Value {
    Value::Record(vec![(
        "minimum".into(),
        Value::Enum(sandbox_name(requirement.minimum).into()),
    )])
}

fn region_residency_value(requirement: &RegionResidencyRequirement) -> Value {
    Value::Record(vec![
        (
            "execution_regions".into(),
            Value::List(
                requirement
                    .execution_regions
                    .iter()
                    .map(region_value)
                    .collect(),
            ),
        ),
        (
            "residency_regions".into(),
            Value::List(
                requirement
                    .residency_regions
                    .iter()
                    .map(region_value)
                    .collect(),
            ),
        ),
    ])
}

fn runtime_value(requirement: &RuntimeProfileRequirement) -> Value {
    Value::Record(vec![
        ("family".into(), Value::String(requirement.family.clone())),
        (
            "profile".into(),
            Value::Record(vec![
                (
                    "catalog".into(),
                    Value::String(requirement.profile.catalog.clone()),
                ),
                (
                    "name".into(),
                    Value::String(requirement.profile.name.clone()),
                ),
            ]),
        ),
        (
            "required_features".into(),
            Value::List(
                requirement
                    .required_features
                    .iter()
                    .map(|x| Value::String(x.clone()))
                    .collect(),
            ),
        ),
    ])
}

fn abi_value(requirement: &ImplementationAbiRequirement) -> Value {
    let schema = &requirement.schema;
    Value::Record(vec![(
        "schema".into(),
        Value::Record(vec![
            (
                "specification_id".into(),
                Value::String(schema.specification_id.clone()),
            ),
            ("type_name".into(), Value::String(schema.type_name.clone())),
            (
                "schema_digest".into(),
                Value::Record(vec![
                    (
                        "algorithm".into(),
                        Value::String(schema.schema_digest.algorithm.clone()),
                    ),
                    (
                        "domain".into(),
                        Value::String(schema.schema_digest.domain.clone()),
                    ),
                    (
                        "hex".into(),
                        Value::String(schema.schema_digest.hex.clone()),
                    ),
                ]),
            ),
        ]),
    )])
}

fn quantity_name(kind: ComputeQuantityKind) -> &'static str {
    match kind {
        ComputeQuantityKind::CpuMillicores => "cpu_millicores",
        ComputeQuantityKind::MemoryBytes => "memory_bytes",
        ComputeQuantityKind::GpuCount => "gpu_count",
    }
}

fn sandbox_name(level: SandboxLevel) -> &'static str {
    match level {
        SandboxLevel::Basic => "basic",
        SandboxLevel::Strong => "strong",
    }
}

fn preference_name(preference: PlacementPreference) -> &'static str {
    match preference {
        PlacementPreference::ArtifactLocality => "artifact_locality",
        PlacementPreference::WarmModelCache => "warm_model_cache",
        PlacementPreference::LowerLoad => "lower_load",
        PlacementPreference::LowerCost => "lower_cost",
    }
}
