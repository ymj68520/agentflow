//! Exact, explicit relation catalogs and content-bound snapshots.

use crate::model::RecoveryActionIdentity;
use crate::proof::{CatalogContentChecksum, RelationCatalogPin};
use af_core::{Value, encode};
use af_effect::{EffectAtom, EffectOperationRef, EffectQualifier};
use af_execution::{RegionRef, SandboxLevel, TrustDomainRef};
use af_flow::{ConfidentialityLabelRef, FlowKind, TaintRef};
use af_idl::{CompiledModuleSet, SpecificationId};
use af_resource::ResourceTypeRef;
use af_usage::UsageMeterRef;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeValueRule {
    FiniteExactValues(Vec<String>),
    OpaqueNfcExactString,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectorKeyRule {
    pub selector_key: String,
    pub resource_attribute_key: String,
    pub required: bool,
    pub values: ScopeValueRule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectQualifierProfile {
    /// An empty vector explicitly denotes the unqualified profile.
    pub qualifiers: Vec<EffectQualifier>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportedEffectResourcePair {
    pub operation: EffectOperationRef,
    pub resource_type: ResourceTypeRef,
    pub selector_keys: Vec<SelectorKeyRule>,
    pub allowed_qualifier_profiles: Vec<EffectQualifierProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EffectResourceCompatibilityCatalog {
    pub complete_operations: Vec<EffectOperationRef>,
    pub supported_pairs: Vec<SupportedEffectResourcePair>,
}

impl EffectResourceCompatibilityCatalog {
    /// Rejects duplicate identities and malformed selector literals. It does
    /// not infer support from names, ports, or resource existence.
    pub fn validate(&self) -> Result<(), &'static str> {
        let mut operations = BTreeSet::new();
        for operation in &self.complete_operations {
            if !operations.insert((&operation.catalog, &operation.name)) {
                return Err("duplicate complete operation");
            }
        }
        let mut pairs = BTreeSet::new();
        for pair in &self.supported_pairs {
            if !operations.contains(&(&pair.operation.catalog, &pair.operation.name)) {
                return Err("supported pair operation is not covered by complete_operations");
            }
            if !pairs.insert((
                &pair.operation.catalog,
                &pair.operation.name,
                pair.resource_type.nominal_key(),
            )) {
                return Err("duplicate operation/resource pair");
            }
            let mut selector_keys = BTreeSet::new();
            for rule in &pair.selector_keys {
                if rule.selector_key.is_empty()
                    || rule.resource_attribute_key.is_empty()
                    || !is_nfc(&rule.selector_key)
                    || !is_nfc(&rule.resource_attribute_key)
                    || !selector_keys.insert(&rule.selector_key)
                {
                    return Err("invalid or duplicate selector key mapping");
                }
                if let ScopeValueRule::FiniteExactValues(values) = &rule.values {
                    let mut unique = BTreeSet::new();
                    if values
                        .iter()
                        .any(|value| value.is_empty() || !is_nfc(value) || !unique.insert(value))
                    {
                        return Err("invalid or duplicate finite exact selector value");
                    }
                }
            }
            let mut profiles = BTreeSet::new();
            for profile in &pair.allowed_qualifier_profiles {
                let mut qualifiers = profile.qualifiers.clone();
                qualifiers.sort_by_key(|item| af_effect::EffectQualifierIdentity::of(*item));
                let mut families = BTreeSet::new();
                if qualifiers
                    .iter()
                    .any(|item| !families.insert(item.family_no()))
                {
                    return Err("qualifier profile repeats a full qualifier family");
                }
                if qualifiers.windows(2).any(|items| items[0] == items[1])
                    || !profiles.insert(
                        qualifiers
                            .iter()
                            .copied()
                            .map(af_effect::EffectQualifierIdentity::of)
                            .collect::<Vec<_>>(),
                    )
                {
                    return Err("duplicate or malformed full qualifier profile");
                }
            }
        }
        Ok(())
    }

    fn canonicalized(mut self) -> Result<Self, &'static str> {
        self.complete_operations
            .sort_by(|a, b| (&a.catalog, &a.name).cmp(&(&b.catalog, &b.name)));
        for pair in &mut self.supported_pairs {
            pair.selector_keys
                .sort_by(|a, b| a.selector_key.cmp(&b.selector_key));
            for profile in &mut pair.allowed_qualifier_profiles {
                profile
                    .qualifiers
                    .sort_by_key(|q| af_effect::EffectQualifierIdentity::of(*q));
            }
            pair.allowed_qualifier_profiles.sort_by(|a, b| {
                qualifier_profile_key(&a.qualifiers).cmp(&qualifier_profile_key(&b.qualifiers))
            });
        }
        self.supported_pairs.sort_by(|a, b| {
            (
                &a.operation.catalog,
                &a.operation.name,
                a.resource_type.nominal_key(),
            )
                .cmp(&(
                    &b.operation.catalog,
                    &b.operation.name,
                    b.resource_type.nominal_key(),
                ))
        });
        for pair in &mut self.supported_pairs {
            for selector in &mut pair.selector_keys {
                if let ScopeValueRule::FiniteExactValues(values) = &mut selector.values {
                    values.sort();
                }
            }
            if af_idl::SpecificationId::parse(pair.resource_type.specification_id()).is_err()
                || pair.resource_type.type_name().is_empty()
                || pair
                    .resource_type
                    .type_name()
                    .chars()
                    .any(char::is_whitespace)
                || pair
                    .resource_type
                    .schema
                    .schema_digest
                    .validate_for_type_schema()
                    .is_err()
            {
                return Err("malformed or unpinned exact resource type reference");
            }
        }
        for operation in &self.complete_operations {
            validate_operation(operation)?;
        }
        self.validate()?;
        Ok(self)
    }

    #[must_use]
    pub fn supports(
        &self,
        operation: &EffectOperationRef,
        resource_type: &ResourceTypeRef,
    ) -> Option<&SupportedEffectResourcePair> {
        self.supported_pairs.iter().find(|pair| {
            pair.operation == *operation
                && pair.resource_type.nominal_key() == resource_type.nominal_key()
        })
    }

    #[must_use]
    pub fn covers_operation(&self, operation: &EffectOperationRef) -> bool {
        self.complete_operations
            .iter()
            .any(|item| item == operation)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryUsageCoverageRule {
    pub action: RecoveryActionIdentity,
    pub complete_meters: Vec<UsageMeterRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RecoveryUsageCoverageCatalog {
    pub rules: Vec<RecoveryUsageCoverageRule>,
}

impl RecoveryUsageCoverageCatalog {
    pub fn validate(&self) -> Result<(), &'static str> {
        let mut actions = BTreeSet::new();
        for rule in &self.rules {
            let key = action_key(&rule.action);
            if !actions.insert(key) {
                return Err("duplicate exact recovery action coverage rule");
            }
            let meters: BTreeSet<_> = rule
                .complete_meters
                .iter()
                .map(|meter| (&meter.catalog, &meter.name))
                .collect();
            if meters.len() != rule.complete_meters.len() {
                return Err("duplicate recovery coverage meter");
            }
            if rule
                .complete_meters
                .iter()
                .any(|meter| af_usage::validate::validate_meter(meter).is_err())
            {
                return Err("invalid exact usage meter identity in recovery coverage");
            }
        }
        Ok(())
    }

    fn canonicalized(mut self) -> Result<Self, &'static str> {
        for rule in &mut self.rules {
            rule.complete_meters
                .sort_by(|a, b| (&a.catalog, &a.name).cmp(&(&b.catalog, &b.name)));
            rule.action.forward_effect = normalize_atom(&rule.action.forward_effect)?;
            rule.action.action_effect = normalize_atom(&rule.action.action_effect)?;
        }
        self.rules
            .sort_by(|a, b| action_key(&a.action).cmp(&action_key(&b.action)));
        self.validate()?;
        Ok(self)
    }

    #[must_use]
    pub fn meters_for(&self, action: &RecoveryActionIdentity) -> Option<&[UsageMeterRef]> {
        self.rules
            .iter()
            .find(|rule| action_key(&rule.action) == action_key(action))
            .map(|rule| rule.complete_meters.as_slice())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegionPair {
    pub execution_region: RegionRef,
    pub residency_region: RegionRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegionPairPolicy {
    FiniteExactPairs(Vec<RegionPair>),
    AllPairsInClosedRegionSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrustDomainPolicy {
    ExactAllowlist(Vec<TrustDomainRef>),
    AllDomainsInClosedTrustSnapshot,
    NoAdditionalTrustRequirement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxFloor {
    NoAdditionalMinimum,
    Minimum(SandboxLevel),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfidentialityExecutionRule {
    pub label: ConfidentialityLabelRef,
    pub region_pairs: RegionPairPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaintExecutionRule {
    pub taint: TaintRef,
    pub trust_domains: TrustDomainPolicy,
    pub sandbox: SandboxFloor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorityExecutionRule {
    pub authority: af_flow::InstructionAuthority,
    pub trust_domains: TrustDomainPolicy,
    pub sandbox: SandboxFloor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowExecutionSink {
    WorkflowOutput(u32),
    ExactEffect(EffectAtom),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowExecutionRule {
    pub kind: FlowKind,
    pub sink: FlowExecutionSink,
    pub trust_domains: TrustDomainPolicy,
    pub sandbox: SandboxFloor,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InformationFlowExecutionCatalog {
    pub closed_regions: Vec<RegionRef>,
    pub closed_trust_domains: Vec<TrustDomainRef>,
    pub confidentiality_rules: Vec<ConfidentialityExecutionRule>,
    pub taint_rules: Vec<TaintExecutionRule>,
    pub authority_rules: Vec<AuthorityExecutionRule>,
    pub sink_rules: Vec<FlowExecutionRule>,
}

impl InformationFlowExecutionCatalog {
    pub fn validate(&self) -> Result<(), &'static str> {
        if has_duplicates(self.closed_regions.iter())
            || has_duplicates(self.closed_trust_domains.iter())
        {
            return Err("duplicate closed region or trust-domain identity");
        }
        if self
            .closed_regions
            .iter()
            .any(|item| !valid_named(&item.catalog, &item.name))
            || self
                .closed_trust_domains
                .iter()
                .any(|item| !valid_named(&item.catalog, &item.name))
        {
            return Err("invalid exact region or trust-domain identity");
        }
        if has_duplicates(
            self.confidentiality_rules
                .iter()
                .map(|rule| (&rule.label.catalog, &rule.label.name)),
        ) {
            return Err("duplicate exact confidentiality rule");
        }
        if has_duplicates(
            self.taint_rules
                .iter()
                .map(|rule| (&rule.taint.catalog, &rule.taint.name)),
        ) {
            return Err("duplicate exact taint rule");
        }
        if has_duplicates(self.authority_rules.iter().map(|rule| rule.authority)) {
            return Err("duplicate instruction-authority rule");
        }
        for rule in &self.confidentiality_rules {
            if !valid_named(&rule.label.catalog, &rule.label.name) {
                return Err("invalid confidentiality label identity");
            }
            if let RegionPairPolicy::FiniteExactPairs(pairs) = &rule.region_pairs {
                if has_duplicates(pairs.iter())
                    || pairs.iter().any(|pair| {
                        !self.closed_regions.contains(&pair.execution_region)
                            || !self.closed_regions.contains(&pair.residency_region)
                    })
                {
                    return Err("finite region relation has duplicate or out-of-snapshot pair");
                }
            }
        }
        for rule in &self.taint_rules {
            if !valid_named(&rule.taint.catalog, &rule.taint.name) {
                return Err("invalid taint identity");
            }
            validate_trust_policy(&rule.trust_domains, &self.closed_trust_domains)?;
        }
        for rule in &self.authority_rules {
            validate_trust_policy(&rule.trust_domains, &self.closed_trust_domains)?;
        }
        let mut sinks = BTreeSet::new();
        for rule in &self.sink_rules {
            let sink = match &rule.sink {
                FlowExecutionSink::WorkflowOutput(port) if *port > 0 => {
                    (rule.kind, format!("output:{port}"))
                }
                FlowExecutionSink::WorkflowOutput(_) => {
                    return Err("flow execution output port is 1-based");
                }
                FlowExecutionSink::ExactEffect(atom) => {
                    let normalized = normalize_atom(atom)?;
                    (
                        rule.kind,
                        format!("effect:{:?}", af_effect::AtomKey::of(&normalized)),
                    )
                }
            };
            validate_trust_policy(&rule.trust_domains, &self.closed_trust_domains)?;
            if !sinks.insert(sink) {
                return Err("duplicate exact flow/execution sink rule");
            }
        }
        Ok(())
    }

    fn canonicalized(mut self) -> Result<Self, &'static str> {
        self.closed_regions.sort();
        self.closed_trust_domains.sort();
        for rule in &mut self.confidentiality_rules {
            if let RegionPairPolicy::FiniteExactPairs(pairs) = &mut rule.region_pairs {
                pairs.sort();
            }
        }
        self.confidentiality_rules.sort_by(|a, b| {
            (&a.label.catalog, &a.label.name).cmp(&(&b.label.catalog, &b.label.name))
        });
        self.taint_rules.sort_by(|a, b| {
            (&a.taint.catalog, &a.taint.name).cmp(&(&b.taint.catalog, &b.taint.name))
        });
        self.authority_rules.sort_by_key(|a| a.authority);
        for rule in &mut self.taint_rules {
            canonicalize_trust_policy(&mut rule.trust_domains);
        }
        for rule in &mut self.authority_rules {
            canonicalize_trust_policy(&mut rule.trust_domains);
        }
        for rule in &mut self.sink_rules {
            canonicalize_trust_policy(&mut rule.trust_domains);
            if let FlowExecutionSink::ExactEffect(atom) = &mut rule.sink {
                *atom = normalize_atom(atom)?;
            }
        }
        self.sink_rules
            .sort_by(|a, b| (a.kind, sink_key(&a.sink)).cmp(&(b.kind, sink_key(&b.sink))));
        self.validate()?;
        Ok(self)
    }
}

/// Exact wire contract used by immutable relation snapshots.
pub trait RelationCatalogWire: Clone {
    const SCHEMA_ID: &'static str;
    const ROOT_TYPE: &'static str;
    fn canonicalized(self) -> Result<Self, &'static str>
    where
        Self: Sized;
    fn to_value(&self) -> Value;
}

impl RelationCatalogWire for EffectResourceCompatibilityCatalog {
    const SCHEMA_ID: &'static str = "agentflow.effect.resource.compatibility@1";
    const ROOT_TYPE: &'static str = "EffectResourceCompatibilityCatalog";
    fn canonicalized(self) -> Result<Self, &'static str> {
        EffectResourceCompatibilityCatalog::canonicalized(self)
    }
    fn to_value(&self) -> Value {
        Value::Record(vec![
            ("complete_operations".into(), Value::List(self.complete_operations.iter().map(operation_value).collect())),
            ("supported_pairs".into(), Value::List(self.supported_pairs.iter().map(|pair| Value::Record(vec![
                ("operation".into(), operation_value(&pair.operation)),
                ("resource_type".into(), Value::Record(vec![("schema".into(), schema_ref_value(&pair.resource_type.schema))])),
                ("selector_keys".into(), Value::List(pair.selector_keys.iter().map(|rule| Value::Record(vec![
                    ("selector_key".into(), Value::String(rule.selector_key.clone())),
                    ("resource_attribute_key".into(), Value::String(rule.resource_attribute_key.clone())),
                    ("required".into(), Value::Bool(rule.required)),
                    ("values".into(), match &rule.values {
                        ScopeValueRule::FiniteExactValues(values) => Value::Variant { case: "finite_exact_values".into(), value: Box::new(Value::List(values.iter().cloned().map(Value::String).collect())) },
                        ScopeValueRule::OpaqueNfcExactString => Value::Variant { case: "opaque_nfc_exact_string".into(), value: Box::new(Value::Unit) },
                    }),
                ])).collect())),
                ("allowed_qualifier_profiles".into(), Value::List(pair.allowed_qualifier_profiles.iter().map(|profile| Value::Record(vec![("qualifiers".into(), Value::List(profile.qualifiers.iter().copied().map(effect_qualifier_value).collect()))])).collect())),
            ])).collect())),
        ])
    }
}

impl RelationCatalogWire for RecoveryUsageCoverageCatalog {
    const SCHEMA_ID: &'static str = "agentflow.recovery.usage.coverage@1";
    const ROOT_TYPE: &'static str = "RecoveryUsageCoverageCatalog";
    fn canonicalized(self) -> Result<Self, &'static str> {
        RecoveryUsageCoverageCatalog::canonicalized(self)
    }
    fn to_value(&self) -> Value {
        Value::Record(vec![(
            "rules".into(),
            Value::List(
                self.rules
                    .iter()
                    .map(|rule| {
                        Value::Record(vec![
                            (
                                "action".into(),
                                crate::canonical::recovery_action_value(&rule.action),
                            ),
                            (
                                "complete_meters".into(),
                                Value::List(
                                    rule.complete_meters
                                        .iter()
                                        .map(|meter| {
                                            Value::Record(vec![
                                                (
                                                    "catalog".into(),
                                                    Value::String(meter.catalog.clone()),
                                                ),
                                                ("name".into(), Value::String(meter.name.clone())),
                                            ])
                                        })
                                        .collect(),
                                ),
                            ),
                        ])
                    })
                    .collect(),
            ),
        )])
    }
}

impl RelationCatalogWire for InformationFlowExecutionCatalog {
    const SCHEMA_ID: &'static str = "agentflow.flow.execution.relation@1";
    const ROOT_TYPE: &'static str = "InformationFlowExecutionCatalog";
    fn canonicalized(self) -> Result<Self, &'static str> {
        InformationFlowExecutionCatalog::canonicalized(self)
    }
    fn to_value(&self) -> Value {
        Value::Record(vec![
            (
                "closed_regions".into(),
                Value::List(self.closed_regions.iter().map(named_ref_value).collect()),
            ),
            (
                "closed_trust_domains".into(),
                Value::List(
                    self.closed_trust_domains
                        .iter()
                        .map(named_ref_value)
                        .collect(),
                ),
            ),
            (
                "confidentiality_rules".into(),
                Value::List(
                    self.confidentiality_rules
                        .iter()
                        .map(|rule| {
                            Value::Record(vec![
                                ("label".into(), named_ref_value(&rule.label)),
                                (
                                    "region_pairs".into(),
                                    match &rule.region_pairs {
                                        RegionPairPolicy::FiniteExactPairs(pairs) => {
                                            Value::Variant {
                                                case: "finite_exact_pairs".into(),
                                                value: Box::new(Value::List(
                                                    pairs
                                                        .iter()
                                                        .map(|pair| {
                                                            Value::Record(vec![
                                                                (
                                                                    "execution_region".into(),
                                                                    named_ref_value(
                                                                        &pair.execution_region,
                                                                    ),
                                                                ),
                                                                (
                                                                    "residency_region".into(),
                                                                    named_ref_value(
                                                                        &pair.residency_region,
                                                                    ),
                                                                ),
                                                            ])
                                                        })
                                                        .collect(),
                                                )),
                                            }
                                        }
                                        RegionPairPolicy::AllPairsInClosedRegionSnapshot => {
                                            Value::Variant {
                                                case: "all_pairs_in_closed_region_snapshot".into(),
                                                value: Box::new(Value::Unit),
                                            }
                                        }
                                    },
                                ),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "taint_rules".into(),
                Value::List(
                    self.taint_rules
                        .iter()
                        .map(|rule| {
                            Value::Record(vec![
                                ("taint".into(), named_ref_value(&rule.taint)),
                                (
                                    "trust_domains".into(),
                                    trust_policy_value(&rule.trust_domains),
                                ),
                                ("sandbox".into(), sandbox_floor_value(&rule.sandbox)),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "authority_rules".into(),
                Value::List(
                    self.authority_rules
                        .iter()
                        .map(|rule| {
                            Value::Record(vec![
                                (
                                    "authority".into(),
                                    Value::Enum(rule.authority.as_str().into()),
                                ),
                                (
                                    "trust_domains".into(),
                                    trust_policy_value(&rule.trust_domains),
                                ),
                                ("sandbox".into(), sandbox_floor_value(&rule.sandbox)),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "sink_rules".into(),
                Value::List(
                    self.sink_rules
                        .iter()
                        .map(|rule| {
                            Value::Record(vec![
                                ("kind".into(), Value::Enum(rule.kind.as_str().into())),
                                (
                                    "sink".into(),
                                    match &rule.sink {
                                        FlowExecutionSink::WorkflowOutput(port) => Value::Variant {
                                            case: "workflow_output".into(),
                                            value: Box::new(Value::UInt32(*port)),
                                        },
                                        FlowExecutionSink::ExactEffect(atom) => Value::Variant {
                                            case: "exact_effect".into(),
                                            value: Box::new(crate::canonical::effect_atom_value(
                                                atom,
                                            )),
                                        },
                                    },
                                ),
                                (
                                    "trust_domains".into(),
                                    trust_policy_value(&rule.trust_domains),
                                ),
                                ("sandbox".into(), sandbox_floor_value(&rule.sandbox)),
                            ])
                        })
                        .collect(),
                ),
            ),
        ])
    }
}

trait CatalogNamedRef {
    fn catalog(&self) -> &str;
    fn name(&self) -> &str;
}
impl CatalogNamedRef for RegionRef {
    fn catalog(&self) -> &str {
        &self.catalog
    }
    fn name(&self) -> &str {
        &self.name
    }
}
impl CatalogNamedRef for TrustDomainRef {
    fn catalog(&self) -> &str {
        &self.catalog
    }
    fn name(&self) -> &str {
        &self.name
    }
}
impl CatalogNamedRef for ConfidentialityLabelRef {
    fn catalog(&self) -> &str {
        &self.catalog
    }
    fn name(&self) -> &str {
        &self.name
    }
}
impl CatalogNamedRef for TaintRef {
    fn catalog(&self) -> &str {
        &self.catalog
    }
    fn name(&self) -> &str {
        &self.name
    }
}

fn named_ref_value<T: CatalogNamedRef>(reference: &T) -> Value {
    Value::Record(vec![
        ("catalog".into(), Value::String(reference.catalog().into())),
        ("name".into(), Value::String(reference.name().into())),
    ])
}
fn operation_value(operation: &EffectOperationRef) -> Value {
    Value::Record(vec![
        ("catalog".into(), Value::String(operation.catalog.clone())),
        ("name".into(), Value::String(operation.name.clone())),
    ])
}
fn schema_ref_value(reference: &af_semantic::TypeSchemaRef) -> Value {
    Value::Record(vec![
        (
            "specification_id".into(),
            Value::String(reference.specification_id.clone()),
        ),
        (
            "type_name".into(),
            Value::String(reference.type_name.clone()),
        ),
        (
            "schema_digest".into(),
            Value::Record(vec![
                (
                    "algorithm".into(),
                    Value::String(reference.schema_digest.algorithm.clone()),
                ),
                (
                    "domain".into(),
                    Value::String(reference.schema_digest.domain.clone()),
                ),
                (
                    "hex".into(),
                    Value::String(reference.schema_digest.hex.clone()),
                ),
            ]),
        ),
    ])
}
fn effect_qualifier_value(qualifier: EffectQualifier) -> Value {
    let member = match qualifier {
        EffectQualifier::Idempotency(af_effect::EffectIdempotencyQualifier::Natural) => "natural",
        EffectQualifier::Idempotency(af_effect::EffectIdempotencyQualifier::None) => "none",
    };
    Value::Variant {
        case: "idempotency".into(),
        value: Box::new(Value::Enum(member.into())),
    }
}
fn trust_policy_value(policy: &TrustDomainPolicy) -> Value {
    match policy {
        TrustDomainPolicy::ExactAllowlist(domains) => Value::Variant {
            case: "exact_allowlist".into(),
            value: Box::new(Value::List(domains.iter().map(named_ref_value).collect())),
        },
        TrustDomainPolicy::AllDomainsInClosedTrustSnapshot => Value::Variant {
            case: "all_domains_in_closed_trust_snapshot".into(),
            value: Box::new(Value::Unit),
        },
        TrustDomainPolicy::NoAdditionalTrustRequirement => Value::Variant {
            case: "no_additional_trust_requirement".into(),
            value: Box::new(Value::Unit),
        },
    }
}
fn sandbox_floor_value(floor: &SandboxFloor) -> Value {
    match floor {
        SandboxFloor::NoAdditionalMinimum => Value::Variant {
            case: "no_additional_minimum".into(),
            value: Box::new(Value::Unit),
        },
        SandboxFloor::Minimum(level) => Value::Variant {
            case: "minimum".into(),
            value: Box::new(Value::Enum(
                match level {
                    SandboxLevel::Basic => "basic",
                    SandboxLevel::Strong => "strong",
                }
                .into(),
            )),
        },
    }
}
fn qualifier_profile_key(
    qualifiers: &[EffectQualifier],
) -> Vec<af_effect::EffectQualifierIdentity> {
    qualifiers
        .iter()
        .copied()
        .map(af_effect::EffectQualifierIdentity::of)
        .collect()
}
fn sink_key(sink: &FlowExecutionSink) -> String {
    match sink {
        FlowExecutionSink::WorkflowOutput(port) => format!("0:{port:010}"),
        FlowExecutionSink::ExactEffect(atom) => format!("1:{:?}", af_effect::AtomKey::of(atom)),
    }
}
fn validate_operation(operation: &EffectOperationRef) -> Result<(), &'static str> {
    let test_atom = EffectAtom {
        operation: operation.clone(),
        resource_selector: af_effect::ResourceSelector {
            requirement_port_no: 1,
            scope_bindings: vec![],
        },
        qualifiers: vec![],
    };
    normalize_atom(&test_atom).map(|_| ())
}
fn normalize_atom(atom: &EffectAtom) -> Result<EffectAtom, &'static str> {
    let signature = af_effect::EffectSignature {
        row: af_effect::EffectRow::Closed(vec![af_effect::EffectEntry {
            atom: atom.clone(),
            bounds: af_effect::OccurrenceBounds {
                lower: 1,
                upper: af_effect::OccurrenceCeiling::Finite(1),
            },
            guard: af_effect::EffectGuard::Unconditional,
        }]),
    };
    let canonical = signature
        .canonical()
        .map_err(|_| "invalid exact effect atom")?;
    match canonical.row {
        af_effect::EffectRow::Closed(mut entries) => entries
            .pop()
            .map(|entry| entry.atom)
            .ok_or("empty canonical effect atom"),
        _ => Err("effect atom did not remain closed"),
    }
}
fn valid_named(catalog: &str, name: &str) -> bool {
    SpecificationId::parse(catalog).is_ok()
        && !name.is_empty()
        && is_nfc(name)
        && !name.chars().any(char::is_whitespace)
}
fn validate_trust_policy(
    policy: &TrustDomainPolicy,
    closed: &[TrustDomainRef],
) -> Result<(), &'static str> {
    if let TrustDomainPolicy::ExactAllowlist(domains) = policy {
        if has_duplicates(domains.iter()) || domains.iter().any(|domain| !closed.contains(domain)) {
            return Err("trust allowlist has duplicate or out-of-snapshot identity");
        }
    }
    Ok(())
}
fn canonicalize_trust_policy(policy: &mut TrustDomainPolicy) {
    if let TrustDomainPolicy::ExactAllowlist(domains) = policy {
        domains.sort();
    }
}

/// A catalog snapshot whose private bytes and checksum are immutable after
/// construction. Bytes are generated from the exact typed catalog model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationCatalogSnapshot<T> {
    schema: String,
    revision: String,
    canonical_bytes: Vec<u8>,
    catalog: T,
    pin: RelationCatalogPin,
}

impl<T: RelationCatalogWire> RelationCatalogSnapshot<T> {
    pub fn new(
        revision: impl Into<String>,
        catalog: T,
        modules: &CompiledModuleSet,
    ) -> Result<Self, &'static str> {
        let schema = T::SCHEMA_ID.to_string();
        let revision = revision.into();
        if SpecificationId::parse(&schema).is_err() || revision.is_empty() || !is_nfc(&revision) {
            return Err("invalid exact schema identity or revision token");
        }
        let catalog = catalog.canonicalized()?;
        let shape = modules
            .shape_of(&format!("{schema}/{}", T::ROOT_TYPE))
            .map_err(|_| "unknown relation catalog schema")?;
        let canonical_bytes = encode(&shape, &catalog.to_value())
            .map_err(|_| "catalog model does not encode against its exact AFIDL schema")?;
        let checksum = sha256_hex(&canonical_bytes);
        let pin = RelationCatalogPin {
            schema: schema.clone(),
            revision: revision.clone(),
            canonical_content_checksum: CatalogContentChecksum {
                algorithm: "sha256",
                hex: checksum,
            },
        };
        Ok(Self {
            schema,
            revision,
            canonical_bytes,
            catalog,
            pin,
        })
    }

    #[must_use]
    pub fn pin(&self) -> &RelationCatalogPin {
        &self.pin
    }
    #[must_use]
    pub fn catalog(&self) -> &T {
        &self.catalog
    }
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }
    #[must_use]
    pub fn revision(&self) -> &str {
        &self.revision
    }
}

#[derive(Debug, Clone, Default)]
pub struct RelationSnapshots {
    pub effect_resource: Option<RelationCatalogSnapshot<EffectResourceCompatibilityCatalog>>,
    pub recovery_usage: Option<RelationCatalogSnapshot<RecoveryUsageCoverageCatalog>>,
    pub flow_execution: Option<RelationCatalogSnapshot<InformationFlowExecutionCatalog>>,
}

impl RelationSnapshots {
    #[must_use]
    pub fn pins(&self) -> Vec<RelationCatalogPin> {
        let mut pins: Vec<_> = [
            self.effect_resource
                .as_ref()
                .map(RelationCatalogSnapshot::pin),
            self.recovery_usage
                .as_ref()
                .map(RelationCatalogSnapshot::pin),
            self.flow_execution
                .as_ref()
                .map(RelationCatalogSnapshot::pin),
        ]
        .into_iter()
        .flatten()
        .cloned()
        .collect();
        pins.sort_by(|a, b| a.schema.cmp(&b.schema));
        pins
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn is_nfc(text: &str) -> bool {
    text.nfc().eq(text.chars())
}

fn action_key(action: &RecoveryActionIdentity) -> (af_effect::AtomKey, u32, af_effect::AtomKey) {
    (
        af_effect::AtomKey::of(&action.forward_effect),
        match action.action_kind {
            crate::model::RecoveryActionKind::Reconciliation => 1,
            crate::model::RecoveryActionKind::Compensation => 2,
        },
        af_effect::AtomKey::of(&action.action_effect),
    )
}

fn has_duplicates<T: Ord>(values: impl Iterator<Item = T>) -> bool {
    let mut seen = BTreeSet::new();
    values.into_iter().any(|item| !seen.insert(item))
}
