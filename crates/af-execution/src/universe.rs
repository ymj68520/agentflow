//! Explicit execution capability/profile universes.
use crate::error::{ExecutionError, kind};
use crate::model::{
    CapabilityAttribute, ExecutionCapabilityRef, RegionRef, RuntimeProfileRef, TrustDomainRef,
};
use std::collections::{BTreeMap, BTreeSet};
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExecutionCapabilityUniverse {
    pub capabilities: BTreeMap<ExecutionCapabilityRef, CapabilityDescriptor>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityDescriptor {
    pub capability: ExecutionCapabilityRef,
    pub exact_version: String,
    pub supported_versions: BTreeSet<String>,
    pub attributes: BTreeSet<(String, String)>,
}
impl ExecutionCapabilityUniverse {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    pub fn recognize(
        &mut self,
        capability: &ExecutionCapabilityRef,
        version: impl Into<String>,
        attributes: impl IntoIterator<Item = CapabilityAttribute>,
    ) -> Result<(), ExecutionError> {
        let c = normalize_ref(capability)?;
        let version = nfc(&version.into());
        if version.is_empty() {
            return Err(ExecutionError::new(
                kind::UNSUPPORTED_CAPABILITY_VERSION,
                "empty capability version",
            ));
        }
        let attrs = attributes
            .into_iter()
            .map(|a| (nfc(&a.key), nfc(&a.value)))
            .collect();
        if let Some(existing) = self.capabilities.get_mut(&c) {
            if existing.attributes != attrs {
                return Err(ExecutionError::new(
                    kind::INVALID_EXECUTION_CAPABILITY,
                    "capability attribute vocabulary changed",
                ));
            }
            if !existing.supported_versions.insert(version) {
                return Err(ExecutionError::new(
                    kind::INVALID_EXECUTION_CAPABILITY,
                    "capability version recognized twice",
                ));
            }
        } else {
            let mut supported_versions = BTreeSet::new();
            supported_versions.insert(version.clone());
            self.capabilities.insert(
                c.clone(),
                CapabilityDescriptor {
                    capability: c,
                    exact_version: version,
                    supported_versions,
                    attributes: attrs,
                },
            );
        }
        Ok(())
    }
    pub fn resolve(
        &self,
        cap: &ExecutionCapabilityRef,
    ) -> Result<&CapabilityDescriptor, ExecutionError> {
        let c = normalize_ref(cap)?;
        self.capabilities.get(&c).ok_or_else(|| {
            ExecutionError::new(
                kind::UNKNOWN_EXECUTION_CAPABILITY,
                "capability is not registered",
            )
        })
    }
    pub fn supports_version(
        &self,
        cap: &ExecutionCapabilityRef,
        version: &str,
    ) -> Result<bool, ExecutionError> {
        Ok(self.resolve(cap)?.supported_versions.contains(version))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrustDomainUniverse {
    domains: BTreeSet<TrustDomainRef>,
}
impl TrustDomainUniverse {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    pub fn recognize(&mut self, domain: TrustDomainRef) -> Result<(), ExecutionError> {
        self.domains.insert(normalize_named(domain)?);
        Ok(())
    }
    pub fn resolves(&self, d: &TrustDomainRef) -> Result<bool, ExecutionError> {
        Ok(self.domains.contains(&normalize_named(d.clone())?))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegionUniverse {
    regions: BTreeSet<RegionRef>,
    compatibility: BTreeSet<(RegionRef, RegionRef)>,
}
impl RegionUniverse {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    pub fn recognize(&mut self, region: RegionRef) -> Result<(), ExecutionError> {
        self.regions.insert(normalize_named(region)?);
        Ok(())
    }
    pub fn recognize_compatibility(
        &mut self,
        from: RegionRef,
        to: RegionRef,
    ) -> Result<(), ExecutionError> {
        let from = normalize_named(from)?;
        let to = normalize_named(to)?;
        self.regions.insert(from.clone());
        self.regions.insert(to.clone());
        self.compatibility.insert((from, to));
        Ok(())
    }
    pub fn resolves(&self, r: &RegionRef) -> Result<bool, ExecutionError> {
        Ok(self.regions.contains(&normalize_named(r.clone())?))
    }
    pub fn compatible(&self, from: &RegionRef, to: &RegionRef) -> Result<bool, ExecutionError> {
        Ok(self
            .compatibility
            .contains(&(normalize_named(from.clone())?, normalize_named(to.clone())?)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeProfileDescriptor {
    pub profile: RuntimeProfileRef,
    pub family: String,
    pub features: BTreeSet<String>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeProfileUniverse {
    profiles: BTreeMap<RuntimeProfileRef, RuntimeProfileDescriptor>,
}
impl RuntimeProfileUniverse {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    pub fn recognize(
        &mut self,
        profile: RuntimeProfileRef,
        family: impl Into<String>,
        features: impl IntoIterator<Item = String>,
    ) -> Result<(), ExecutionError> {
        let profile = normalize_named(profile)?;
        if self.profiles.contains_key(&profile) {
            return Err(ExecutionError::new(
                kind::UNKNOWN_RUNTIME_PROFILE,
                "runtime profile recognized twice",
            ));
        }
        self.profiles.insert(
            profile.clone(),
            RuntimeProfileDescriptor {
                profile,
                family: nfc(&family.into()),
                features: features.into_iter().map(|feature| nfc(&feature)).collect(),
            },
        );
        Ok(())
    }
    pub fn resolve(
        &self,
        p: &RuntimeProfileRef,
    ) -> Result<&RuntimeProfileDescriptor, ExecutionError> {
        self.profiles
            .get(&normalize_named(p.clone())?)
            .ok_or_else(|| {
                ExecutionError::new(
                    kind::UNKNOWN_RUNTIME_PROFILE,
                    "runtime profile is not registered",
                )
            })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExecutionUniverses {
    pub capabilities: ExecutionCapabilityUniverse,
    pub trust_domains: TrustDomainUniverse,
    pub regions: RegionUniverse,
    pub runtime_profiles: RuntimeProfileUniverse,
}
fn nfc(s: &str) -> String {
    s.nfc().collect()
}
fn normalize_ref(r: &ExecutionCapabilityRef) -> Result<ExecutionCapabilityRef, ExecutionError> {
    validate(&r.catalog, &r.name, kind::INVALID_EXECUTION_CAPABILITY).map(|()| {
        ExecutionCapabilityRef {
            catalog: r.catalog.clone(),
            name: nfc(&r.name),
        }
    })
}
fn normalize_named<T>(r: T) -> Result<T, ExecutionError>
where
    T: NamedRef,
{
    validate(r.catalog(), r.name(), T::error_kind()).map(|()| r.normalized())
}
trait NamedRef {
    fn catalog(&self) -> &str;
    fn name(&self) -> &str;
    fn error_kind() -> &'static str;
    fn normalized(self) -> Self;
}
impl NamedRef for TrustDomainRef {
    fn catalog(&self) -> &str {
        &self.catalog
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn error_kind() -> &'static str {
        kind::UNKNOWN_TRUST_DOMAIN
    }
    fn normalized(mut self) -> Self {
        self.name = nfc(&self.name);
        self
    }
}
impl NamedRef for RegionRef {
    fn catalog(&self) -> &str {
        &self.catalog
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn error_kind() -> &'static str {
        kind::UNKNOWN_REGION
    }
    fn normalized(mut self) -> Self {
        self.name = nfc(&self.name);
        self
    }
}
impl NamedRef for RuntimeProfileRef {
    fn catalog(&self) -> &str {
        &self.catalog
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn error_kind() -> &'static str {
        kind::UNKNOWN_RUNTIME_PROFILE
    }
    fn normalized(mut self) -> Self {
        self.name = nfc(&self.name);
        self
    }
}
fn validate(catalog: &str, name: &str, error: &'static str) -> Result<(), ExecutionError> {
    if af_idl::SpecificationId::parse(catalog).is_err()
        || name.is_empty()
        || name.chars().any(char::is_whitespace)
    {
        Err(ExecutionError::new(error, "invalid logical identity"))
    } else {
        Ok(())
    }
}
