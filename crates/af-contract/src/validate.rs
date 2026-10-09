//! Single-contract cross-dimension validation with explicit proof inputs.

use crate::canonical::{CanonicalComponentContract, canonicalize_contract};
use crate::error::ContractError;
use crate::model::{ComponentContract, RecoveryActionIdentity, RecoveryActionKind};
use crate::proof::{CheckedComponentContract, DimensionProof, ProofReport, ProofVerdict};
use crate::universe::{
    EffectResourceCompatibilityCatalog, FlowExecutionSink, InformationFlowExecutionCatalog,
    RegionPairPolicy, RelationSnapshots, SandboxFloor, ScopeValueRule, TrustDomainPolicy,
};
use af_effect::{AtomKey, EffectAtom, EffectRow, EffectSignature, OccurrenceCeiling};
use af_flow::{FlowSink, SourceCategory};
use af_idl::CompiledModuleSet;
use af_usage::{UsagePhase, UsageUpperBound};
use std::collections::{BTreeMap, BTreeSet};
use unicode_normalization::UnicodeNormalization;

/// All inputs are explicit references. This context is read-only, contains
/// no I/O or resolver, and relation pins are captured in every report.
pub struct StaticContractProofContext<'a> {
    pub modules: &'a CompiledModuleSet,
    pub schema_universe: &'a af_semantic::TypeSchemaUniverse,
    pub effects: &'a af_effect::EffectCatalogUniverse,
    pub resources: &'a af_resource::ResourceTypeUniverse,
    pub flow: &'a af_flow::FlowCatalogs,
    pub usage_meters: &'a af_usage::UsageMeterUniverse,
    pub recovery: &'a af_recovery::RecoveryOperationUniverse,
    pub execution: &'a af_execution::ExecutionUniverses,
    pub relations: &'a RelationSnapshots,
}

/// Builds the validated canonical body and a dimension-preserving report.
/// Only a fully `Proven` report produces the checked digest-capable wrapper.
pub fn check_contract<'context, 'universe>(
    raw: &ComponentContract,
    context: &'context StaticContractProofContext<'universe>,
) -> Result<CheckedComponentContract<'context, 'universe>, ProofReport> {
    let report = analyze_contract(raw, context);
    if report.is_proven() {
        let canonical = match canonicalize_contract(raw, context) {
            Ok(value) => value,
            Err(error) => return Err(invalid_contract_report(&error, context)),
        };
        Ok(CheckedComponentContract {
            contract: canonical.contract,
            proof: report,
            context,
        })
    } else {
        Err(report)
    }
}

/// Checks and canonicalizes without asserting that every cross-dimension
/// relation has enough evidence for a complete positive proof.
pub fn validate_raw_contract(
    raw: &ComponentContract,
    context: &StaticContractProofContext<'_>,
) -> Result<CanonicalComponentContract, ContractError> {
    canonicalize_contract(raw, context)
}

/// Produces the complete dimension-preserving report without issuing a
/// checked wrapper or digest capability.
pub fn analyze_contract(
    raw: &ComponentContract,
    context: &StaticContractProofContext<'_>,
) -> ProofReport {
    match canonicalize_contract(raw, context) {
        Ok(canonical) => prove_canonical_contract(&canonical, context),
        Err(error) => invalid_contract_report(&error, context),
    }
}

pub(crate) fn analyze_dimension(
    raw: &ComponentContract,
    context: &StaticContractProofContext<'_>,
    dimension: &'static str,
) -> DimensionProof {
    analyze_contract(raw, context)
        .dimensions
        .into_iter()
        .find(|item| item.dimension == dimension)
        .unwrap_or_else(|| {
            DimensionProof::unproven(
                dimension,
                "AF-CONTRACT-0001",
                "dimension report was not produced",
            )
        })
}

pub(crate) fn validate_component_identity(
    identity: &crate::model::ComponentIdentity,
) -> Result<(), ContractError> {
    if af_idl::SpecificationId::parse(&identity.catalog).is_err() {
        return Err(ContractError::new(
            "AF-CONTRACT-0002",
            "invalid_component_identity",
            "catalog is not an exact SpecificationId",
        ));
    }
    if identity.name.is_empty()
        || !identity.name.nfc().eq(identity.name.chars())
        || identity.name.split('.').any(|segment| {
            segment.is_empty()
                || !segment
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
    {
        return Err(ContractError::new(
            "AF-CONTRACT-0002",
            "invalid_component_identity",
            "name must be a non-empty NFC lowercase dotted key using [a-z0-9-] segments",
        ));
    }
    Ok(())
}

fn invalid_contract_report(
    error: &ContractError,
    context: &StaticContractProofContext<'_>,
) -> ProofReport {
    let unresolved_dimension = unresolved_reference_dimension(error.code);
    let mut dimensions = vec![if unresolved_dimension.is_some() {
        DimensionProof::proven(
            "schema-validity",
            error.rule_id,
            "wire shape is valid but an explicit proof universe is incomplete",
        )
    } else {
        DimensionProof::violation(
            "schema-validity",
            error.rule_id,
            format!("{}: {}", error.code, error.detail),
        )
    }];
    if let Some(reference) = &error.reference {
        dimensions[0].reference = Some(reference.clone());
    }
    for name in [
        "io",
        "failure",
        "effects",
        "resources",
        "information-flow",
        "usage",
        "recovery",
        "execution",
        "cross-dimension",
    ] {
        dimensions.push(DimensionProof::unproven(
            name,
            "AF-CONTRACT-0001",
            "not evaluated because canonical input validation failed",
        ));
    }
    if let Some(name) = unresolved_dimension {
        if let Some(item) = dimensions.iter_mut().find(|item| item.dimension == name) {
            item.rule_id = error.rule_id;
            item.reference.clone_from(&error.reference);
            item.reason.clone_from(&error.detail);
        }
    }
    attach_prerequisites(&mut dimensions);
    ProofReport {
        dimensions,
        relation_catalogs: context.relations.pins(),
        action_guard_compatibility: vec![],
    }
}

fn unresolved_reference_dimension(code: &str) -> Option<&'static str> {
    match code {
        "unknown_failure_schema" => Some("failure"),
        "unknown_effect_operation" => Some("effects"),
        "unknown_resource_type" | "unknown_resource_attribute" => Some("resources"),
        "unknown_information_label" | "unknown_flow_sink" => Some("information-flow"),
        "unknown_usage_meter" => Some("usage"),
        "unknown_recovery_operation" => Some("recovery"),
        "unknown_execution_capability"
        | "unknown_trust_domain"
        | "unknown_region"
        | "unknown_runtime_profile" => Some("execution"),
        _ => None,
    }
}

fn prove_canonical_contract(
    canonical: &CanonicalComponentContract,
    context: &StaticContractProofContext<'_>,
) -> ProofReport {
    let contract = &canonical.contract;
    let mut dimensions = vec![
        DimensionProof::proven(
            "schema-validity",
            "AF-CONTRACT-0001",
            "all typed declarations and exact pins passed their existing validators",
        ),
        DimensionProof::proven(
            "io",
            "AF-CONTRACT-0004",
            "public port namespaces are positive, unique, exact-pinned and canonical",
        ),
        DimensionProof::proven(
            "failure",
            "AF-CONTRACT-0005",
            "failure row resolved against the explicit schema universe",
        ),
        DimensionProof::proven(
            "effects",
            "AF-CONTRACT-0006",
            "effect row resolved against the explicit operation universe",
        ),
        DimensionProof::proven(
            "resources",
            "AF-CONTRACT-0007",
            "resource requirements resolved against the explicit resource universe",
        ),
        DimensionProof::proven(
            "information-flow",
            "AF-CONTRACT-0008",
            "information-flow declaration is structurally and catalog valid",
        ),
        DimensionProof::proven(
            "usage",
            "AF-CONTRACT-0009",
            "usage declaration resolved against the explicit meter universe",
        ),
        DimensionProof::proven(
            "recovery",
            "AF-CONTRACT-0010",
            "recovery declaration resolved against the explicit operation universe",
        ),
        DimensionProof::proven(
            "execution",
            "AF-CONTRACT-0011",
            "execution requirements resolved against explicit capability universes",
        ),
    ];

    check_flow_ports(contract, &mut dimensions);
    check_effect_resource(
        contract,
        context
            .relations
            .effect_resource
            .as_ref()
            .map(crate::universe::RelationCatalogSnapshot::catalog),
        &mut dimensions,
    );
    check_effect_flow(contract, &mut dimensions);
    let action_guard_compatibility = check_recovery_effects(contract, context, &mut dimensions);
    for relation in &action_guard_compatibility {
        if relation.verdict() != ProofVerdict::Proven {
            set_dimension(
                &mut dimensions,
                "recovery",
                relation.verdict(),
                "AF-CONTRACT-0022",
                relation.reason(),
            );
        }
    }
    check_recovery_usage(contract, context, &mut dimensions);
    check_flow_execution(
        contract,
        context
            .relations
            .flow_execution
            .as_ref()
            .map(crate::universe::RelationCatalogSnapshot::catalog),
        &mut dimensions,
    );

    let cross_verdict = if dimensions
        .iter()
        .any(|item| item.verdict == ProofVerdict::Violation)
    {
        DimensionProof::violation(
            "cross-dimension",
            "AF-CONTRACT-0017",
            "one or more cross-dimension constraints are violated",
        )
    } else if dimensions
        .iter()
        .any(|item| item.verdict == ProofVerdict::RequiresEvidence)
    {
        DimensionProof {
            verdict: ProofVerdict::RequiresEvidence,
            ..DimensionProof::unproven(
                "cross-dimension",
                "AF-CONTRACT-0017",
                "a required proof depends on external evidence",
            )
        }
    } else if dimensions
        .iter()
        .any(|item| item.verdict == ProofVerdict::Unproven)
    {
        DimensionProof::unproven(
            "cross-dimension",
            "AF-CONTRACT-0017",
            "an unknown or missing dimension proof prevents a positive aggregate",
        )
    } else {
        DimensionProof::proven(
            "cross-dimension",
            "AF-CONTRACT-0017",
            "all required cross-dimension constraints are proven",
        )
    };
    dimensions.push(cross_verdict);
    attach_prerequisites(&mut dimensions);
    ProofReport {
        dimensions,
        relation_catalogs: context.relations.pins(),
        action_guard_compatibility,
    }
}

fn attach_prerequisites(dimensions: &mut [DimensionProof]) {
    for item in dimensions {
        item.prerequisites = match item.dimension {
            "schema-validity" => vec![
                "closed ComponentContract@1 schema".into(),
                "all exact imported schema modules compiled".into(),
            ],
            "io" => vec![
                "explicit TypeSchemaUniverse".into(),
                "positive unique per-direction ports".into(),
            ],
            "failure" => vec![
                "exact failure TypeSchemaRef pins".into(),
                "explicit TypeSchemaUniverse".into(),
            ],
            "effects" => vec!["exact EffectOperationRef catalog coverage".into()],
            "resources" => vec![
                "exact ResourceTypeUniverse".into(),
                "EffectResourceCompatibilityCatalog snapshot when epsilon is non-empty".into(),
            ],
            "information-flow" => vec![
                "explicit label and effect-sink catalogs".into(),
                "exact I/O/R port namespaces".into(),
            ],
            "usage" => vec![
                "explicit additive meter/unit universe".into(),
                "exhaustive recovery coverage snapshot for every action".into(),
            ],
            "recovery" => vec![
                "closed effect row for complete action coverage".into(),
                "exact K clause and action identities".into(),
            ],
            "execution" => vec![
                "explicit execution capability/profile universes".into(),
                "exact F/X relation snapshot when flow constraints require it".into(),
            ],
            "cross-dimension" => vec![
                "every required dimension is Proven".into(),
                "all relation catalog pins match this proof context".into(),
            ],
            _ => vec!["explicit proof context".into()],
        };
    }
}

fn declared_effects(signature: &EffectSignature) -> (Vec<&EffectAtom>, bool) {
    match &signature.row {
        EffectRow::Closed(entries) => (entries.iter().map(|entry| &entry.atom).collect(), true),
        EffectRow::Open(open) => (open.known.iter().map(|entry| &entry.atom).collect(), false),
        EffectRow::Unknown => (Vec::new(), false),
    }
}

fn check_flow_ports(contract: &ComponentContract, dimensions: &mut [DimensionProof]) {
    let inputs: BTreeSet<_> = contract.inputs.iter().map(|port| port.port_no).collect();
    let outputs: BTreeSet<_> = contract.outputs.iter().map(|port| port.port_no).collect();
    let resources: BTreeSet<_> = contract
        .resources
        .requirements
        .iter()
        .map(|item| item.port_no)
        .collect();
    let mut failure = None;
    for source in &contract.information_flow.summary.sources {
        match source.category {
            SourceCategory::WorkflowInput(port) if !inputs.contains(&port) => {
                failure = Some((
                    ProofVerdict::Violation,
                    "AF-CONTRACT-0012",
                    format!("flow source references undeclared input port {port}"),
                ));
            }
            SourceCategory::ResourceRequirement(port) if !resources.contains(&port) => {
                failure = Some((
                    ProofVerdict::Violation,
                    "AF-CONTRACT-0012",
                    format!("flow source references undeclared resource port {port}"),
                ));
            }
            _ => {}
        }
    }
    for constraint in &contract.information_flow.flow_constraints {
        match &constraint.sink {
            FlowSink::WorkflowOutput(port) if !outputs.contains(port) => {
                failure = Some((
                    ProofVerdict::Violation,
                    "AF-CONTRACT-0012",
                    format!("flow sink references undeclared output port {port}"),
                ));
            }
            FlowSink::ExternalEffect(operation)
                if !contract.effects.row_has_operation(operation) =>
            {
                let (_, closed) = declared_effects(&contract.effects);
                failure = Some((
                    if closed {
                        ProofVerdict::Violation
                    } else {
                        ProofVerdict::Unproven
                    },
                    "AF-CONTRACT-0012",
                    format!(
                        "flow sink operation {}/{} has no known exact atom in the declared effect row",
                        operation.catalog, operation.name
                    ),
                ));
            }
            _ => {}
        }
    }
    if let Some((verdict, rule, reason)) = failure {
        set_dimension(dimensions, "information-flow", verdict, rule, reason);
    } else if contract.information_flow.summary.influence_unknown
        || contract
            .information_flow
            .summary
            .sources
            .iter()
            .any(|source| matches!(source.category, SourceCategory::UnknownExternal))
    {
        set_dimension(
            dimensions,
            "information-flow",
            ProofVerdict::RequiresEvidence,
            "AF-CONTRACT-0012",
            "unknown external origin or influence cannot yield a positive information-flow proof",
        );
    }
}

trait EffectRowLookup {
    fn row_has_operation(&self, operation: &af_effect::EffectOperationRef) -> bool;
}
impl EffectRowLookup for EffectSignature {
    fn row_has_operation(&self, operation: &af_effect::EffectOperationRef) -> bool {
        let (atoms, _) = declared_effects(self);
        atoms.iter().any(|atom| atom.operation == *operation)
    }
}

fn check_effect_resource(
    contract: &ComponentContract,
    catalog: Option<&EffectResourceCompatibilityCatalog>,
    dimensions: &mut [DimensionProof],
) {
    let (atoms, closed) = declared_effects(&contract.effects);
    if atoms.is_empty() && closed {
        return;
    }
    let Some(catalog) = catalog else {
        set_dimension(
            dimensions,
            "resources",
            ProofVerdict::Unproven,
            "AF-CONTRACT-0013",
            "missing exact EffectResourceCompatibilityCatalog snapshot",
        );
        return;
    };
    if catalog.validate().is_err() {
        set_dimension(
            dimensions,
            "resources",
            ProofVerdict::Violation,
            "AF-CONTRACT-0013",
            "effect/resource relation catalog contains duplicate or malformed rows",
        );
        return;
    }
    if !closed {
        set_dimension(
            dimensions,
            "resources",
            ProofVerdict::Unproven,
            "AF-CONTRACT-0013",
            "open or unknown effect row is not a complete operation set",
        );
    }
    for atom in atoms {
        let Some(requirement) = contract
            .resources
            .at_port(atom.resource_selector.requirement_port_no)
        else {
            set_dimension_at(
                dimensions,
                "resources",
                ProofVerdict::Violation,
                "AF-CONTRACT-0013",
                format!("effect-atom:{:?}", AtomKey::of(atom)),
                format!(
                    "effect selector refers to missing resource port {}",
                    atom.resource_selector.requirement_port_no
                ),
            );
            continue;
        };
        if !catalog.covers_operation(&atom.operation) {
            set_dimension_at(
                dimensions,
                "resources",
                ProofVerdict::Unproven,
                "AF-CONTRACT-0013",
                format!(
                    "operation:{}/{}",
                    atom.operation.catalog, atom.operation.name
                ),
                format!(
                    "operation {}/{} is outside relation catalog complete_operations",
                    atom.operation.catalog, atom.operation.name
                ),
            );
            continue;
        }
        let Some(pair) = catalog.supports(&atom.operation, &requirement.resource_type) else {
            set_dimension_at(
                dimensions,
                "resources",
                ProofVerdict::Violation,
                "AF-CONTRACT-0013",
                format!(
                    "operation:{}/{};resource:{}/{}/{}",
                    atom.operation.catalog,
                    atom.operation.name,
                    requirement.resource_type.specification_id(),
                    requirement.resource_type.type_name(),
                    requirement.resource_type.schema_hex()
                ),
                format!(
                    "catalog explicitly does not support operation {}/{} with resource type {}/{}",
                    atom.operation.catalog,
                    atom.operation.name,
                    requirement.resource_type.specification_id(),
                    requirement.resource_type.type_name()
                ),
            );
            continue;
        };
        let bindings: BTreeMap<_, _> = atom
            .resource_selector
            .scope_bindings
            .iter()
            .map(|item| (item.key.as_str(), item.value.as_str()))
            .collect();
        for rule in &pair.selector_keys {
            match bindings.get(rule.selector_key.as_str()) {
                None if rule.required => set_dimension(
                    dimensions,
                    "resources",
                    ProofVerdict::Violation,
                    "AF-CONTRACT-0013",
                    format!("required selector key {:?} is missing", rule.selector_key),
                ),
                Some(value) => {
                    let allowed = match &rule.values {
                        ScopeValueRule::FiniteExactValues(values) => {
                            values.iter().any(|item| item == value)
                        }
                        ScopeValueRule::OpaqueNfcExactString => value.nfc().eq(value.chars()),
                    };
                    if !allowed {
                        set_dimension(
                            dimensions,
                            "resources",
                            ProofVerdict::Violation,
                            "AF-CONTRACT-0013",
                            format!(
                                "selector value for {:?} is outside the exact relation",
                                rule.selector_key
                            ),
                        );
                    }
                    if let Some(attribute) = requirement
                        .required_attributes
                        .iter()
                        .find(|item| item.key == rule.resource_attribute_key)
                    {
                        if attribute.value != *value {
                            set_dimension(
                                dimensions,
                                "resources",
                                ProofVerdict::Violation,
                                "AF-CONTRACT-0013",
                                format!(
                                    "selector scope conflicts with resource attribute {:?}",
                                    rule.resource_attribute_key
                                ),
                            );
                        }
                    }
                }
                None => {}
            }
        }
        if bindings.keys().any(|key| {
            !pair
                .selector_keys
                .iter()
                .any(|rule| rule.selector_key == **key)
        }) {
            set_dimension(
                dimensions,
                "resources",
                ProofVerdict::Violation,
                "AF-CONTRACT-0013",
                "selector contains an unmapped scope key",
            );
        }
        if !pair
            .allowed_qualifier_profiles
            .iter()
            .any(|profile| exact_qualifiers(&profile.qualifiers, &atom.qualifiers))
        {
            set_dimension_at(
                dimensions,
                "resources",
                ProofVerdict::Violation,
                "AF-CONTRACT-0013",
                format!("effect-atom:{:?}", AtomKey::of(atom)),
                "full typed qualifier profile is not allowed by the exact operation/resource relation",
            );
        }
    }
}

fn exact_qualifiers(
    left: &[af_effect::EffectQualifier],
    right: &[af_effect::EffectQualifier],
) -> bool {
    let mut a = left.to_vec();
    let mut b = right.to_vec();
    a.sort_by_key(|value| af_effect::EffectQualifierIdentity::of(*value));
    b.sort_by_key(|value| af_effect::EffectQualifierIdentity::of(*value));
    a == b
}

fn check_effect_flow(contract: &ComponentContract, dimensions: &mut [DimensionProof]) {
    let (atoms, closed) = declared_effects(&contract.effects);
    let mut partition_keys = BTreeSet::new();
    for partition in &contract.effect_flow_partitions {
        let key = (
            partition.operation.catalog.as_str(),
            partition.operation.name.as_str(),
            partition.kind,
        );
        if !partition_keys.insert(key) {
            set_dimension(
                dimensions,
                "information-flow",
                ProofVerdict::Violation,
                "AF-CONTRACT-0014",
                "duplicate operation/FlowKind partition",
            );
        }
        let legacy = contract.information_flow.flow_constraints.iter().any(|item| item.kind == partition.kind && matches!(&item.sink, FlowSink::ExternalEffect(op) if op == &partition.operation));
        if !legacy {
            set_dimension(
                dimensions,
                "information-flow",
                ProofVerdict::Violation,
                "AF-CONTRACT-0014",
                "partition has no matching legacy operation-level flow ceiling",
            );
        }
        let mut allowed = BTreeSet::new();
        for effect in &partition.allowed_effects {
            let identity = AtomKey::of(effect);
            if !allowed.insert(identity.clone()) {
                set_dimension(
                    dimensions,
                    "information-flow",
                    ProofVerdict::Violation,
                    "AF-CONTRACT-0014",
                    "duplicate exact effect atom in partition",
                );
            }
            if !atoms
                .iter()
                .any(|candidate| AtomKey::of(candidate) == identity)
            {
                if closed {
                    set_dimension(
                        dimensions,
                        "information-flow",
                        ProofVerdict::Violation,
                        "AF-CONTRACT-0014",
                        "partition atom is absent from declared closed effect row",
                    );
                } else {
                    set_dimension(
                        dimensions,
                        "information-flow",
                        ProofVerdict::Unproven,
                        "AF-CONTRACT-0014",
                        "open/unknown effect row cannot prove complete partition coverage",
                    );
                }
            }
            if effect.operation != partition.operation {
                set_dimension(
                    dimensions,
                    "information-flow",
                    ProofVerdict::Violation,
                    "AF-CONTRACT-0014",
                    "partition contains an atom for a different operation",
                );
            }
        }
    }
}

fn check_recovery_effects(
    contract: &ComponentContract,
    context: &StaticContractProofContext<'_>,
    dimensions: &mut [DimensionProof],
) -> Vec<crate::recovery_guard::ActionGuardCompatibility> {
    let clauses = &contract.recovery.effect_recovery_clauses;
    let bindings = &contract.recovery_action_bindings;
    let (declared, closed) = declared_effects(&contract.effects);
    let declared_keys: BTreeSet<_> = declared.iter().map(|atom| AtomKey::of(atom)).collect();
    let mut identities = BTreeSet::new();
    let mut action_slots = BTreeSet::new();
    let mut graph: BTreeMap<AtomKey, Vec<AtomKey>> = BTreeMap::new();
    for binding in bindings {
        let identity = &binding.identity;
        let key = action_key(identity);
        if !identities.insert(key.clone()) {
            set_dimension(
                dimensions,
                "recovery",
                ProofVerdict::Violation,
                "AF-CONTRACT-0015",
                "duplicate exact recovery action binding",
            );
        }
        let forward = AtomKey::of(&identity.forward_effect);
        let action_kind_no = match identity.action_kind {
            RecoveryActionKind::Reconciliation => 1,
            RecoveryActionKind::Compensation => 2,
        };
        if !action_slots.insert((forward.clone(), action_kind_no)) {
            set_dimension(
                dimensions,
                "recovery",
                ProofVerdict::Violation,
                "AF-CONTRACT-0015",
                "more than one action binding occupies the same exact forward/action-kind slot",
            );
        }
        let action = AtomKey::of(&identity.action_effect);
        if !declared_keys.contains(&forward) {
            set_dimension(
                dimensions,
                "recovery",
                if closed {
                    ProofVerdict::Violation
                } else {
                    ProofVerdict::Unproven
                },
                "AF-CONTRACT-0015",
                "recovery action forward effect is not in the declared closed effect set",
            );
        }
        if !declared_keys.contains(&action) {
            set_dimension(
                dimensions,
                "recovery",
                if closed {
                    ProofVerdict::Violation
                } else {
                    ProofVerdict::Unproven
                },
                "AF-CONTRACT-0015",
                "recovery action effect is not in the declared closed effect set",
            );
        }
        // A same-atom Forward/Action relation is one atom with two distinct
        // occurrence opportunities, not a dependency edge back into the
        // recovery graph. Its count is checked against the shared epsilon
        // ceiling by the computation recovery fold.
        if forward != action {
            graph.entry(forward).or_default().push(action);
        }
        if !clauses
            .iter()
            .any(|clause| AtomKey::of(&clause.effect) == AtomKey::of(&identity.forward_effect))
        {
            set_dimension(
                dimensions,
                "recovery",
                ProofVerdict::Violation,
                "AF-CONTRACT-0015",
                "binding has no exact Recovery clause",
            );
        }
        let matching_clause = clauses
            .iter()
            .find(|clause| AtomKey::of(&clause.effect) == AtomKey::of(&identity.forward_effect));
        if identity.action_kind == RecoveryActionKind::Reconciliation
            && matching_clause.is_none_or(|clause| {
                clause.reconciliation != af_recovery::ReconciliationRequirement::Required
            })
        {
            set_dimension(
                dimensions,
                "recovery",
                ProofVerdict::Violation,
                "AF-CONTRACT-0015",
                "reconciliation binding is not required by the exact K clause",
            );
        }
        if identity.action_kind == RecoveryActionKind::Compensation {
            let required = clauses
                .iter()
                .find(|clause| AtomKey::of(&clause.effect) == AtomKey::of(&identity.forward_effect))
                .and_then(|clause| match &clause.compensation {
                    af_recovery::CompensationRequirement::Required(operation) => Some(operation),
                    af_recovery::CompensationRequirement::None => None,
                });
            if required.is_none_or(|operation| operation != &identity.action_effect.operation) {
                set_dimension(
                    dimensions,
                    "recovery",
                    ProofVerdict::Violation,
                    "AF-CONTRACT-0015",
                    "compensation action operation does not equal the exact K requirement",
                );
            }
        }
    }
    for clause in clauses {
        let forward = AtomKey::of(&clause.effect);
        if !declared_keys.contains(&forward) {
            set_dimension(
                dimensions,
                "recovery",
                if closed {
                    ProofVerdict::Violation
                } else {
                    ProofVerdict::Unproven
                },
                "AF-CONTRACT-0015",
                "Recovery clause does not bind to an exact declared forward effect",
            );
        }
        let reconciliation_count = bindings
            .iter()
            .filter(|binding| {
                binding.identity.action_kind == RecoveryActionKind::Reconciliation
                    && AtomKey::of(&binding.identity.forward_effect) == forward
            })
            .count();
        let reconciliation_required =
            clause.reconciliation == af_recovery::ReconciliationRequirement::Required;
        if reconciliation_count != usize::from(reconciliation_required) {
            set_dimension(
                dimensions,
                "recovery",
                ProofVerdict::Violation,
                "AF-CONTRACT-0015",
                "reconciliation binding count does not exactly match the K requirement",
            );
        }
        let compensation_required = matches!(
            clause.compensation,
            af_recovery::CompensationRequirement::Required(_)
        );
        let compensation_count = bindings
            .iter()
            .filter(|binding| {
                binding.identity.action_kind == RecoveryActionKind::Compensation
                    && AtomKey::of(&binding.identity.forward_effect) == forward
            })
            .count();
        if compensation_count != usize::from(compensation_required) {
            set_dimension(
                dimensions,
                "recovery",
                ProofVerdict::Violation,
                "AF-CONTRACT-0015",
                "compensation binding count does not exactly match the K requirement",
            );
        }
    }
    if graph_has_cycle(&graph) {
        set_dimension(
            dimensions,
            "recovery",
            ProofVerdict::Violation,
            "AF-CONTRACT-0015",
            "recovery action dependency graph contains a cycle",
        );
    }
    if !closed && !bindings.is_empty() {
        set_dimension(
            dimensions,
            "recovery",
            ProofVerdict::Unproven,
            "AF-CONTRACT-0015",
            "open/unknown epsilon cannot prove exact recovery action coverage",
        );
    }
    crate::recovery_guard::analyze_action_guard_compatibility(contract, context)
}

fn graph_has_cycle(graph: &BTreeMap<AtomKey, Vec<AtomKey>>) -> bool {
    fn visit(
        node: &AtomKey,
        graph: &BTreeMap<AtomKey, Vec<AtomKey>>,
        visiting: &mut BTreeSet<AtomKey>,
        done: &mut BTreeSet<AtomKey>,
    ) -> bool {
        if done.contains(node) {
            return false;
        }
        if !visiting.insert(node.clone()) {
            return true;
        }
        if graph.get(node).is_some_and(|edges| {
            edges
                .iter()
                .any(|target| visit(target, graph, visiting, done))
        }) {
            return true;
        }
        visiting.remove(node);
        done.insert(node.clone());
        false
    }
    let mut visiting = BTreeSet::new();
    let mut done = BTreeSet::new();
    graph
        .keys()
        .any(|node| visit(node, graph, &mut visiting, &mut done))
}

fn check_recovery_usage(
    contract: &ComponentContract,
    context: &StaticContractProofContext<'_>,
    dimensions: &mut [DimensionProof],
) {
    let actions: Vec<_> = contract
        .recovery_action_bindings
        .iter()
        .map(|binding| &binding.identity)
        .collect();
    if contract.recovery_usage_bindings.iter().any(|binding| {
        !actions
            .iter()
            .any(|action| action_key(action) == action_key(&binding.action))
    }) {
        set_dimension(
            dimensions,
            "usage",
            ProofVerdict::Violation,
            "AF-CONTRACT-0016",
            "orphan recovery usage binding has no exact recovery action",
        );
    }
    if actions.is_empty() {
        return;
    }
    let Some(snapshot) = context.relations.recovery_usage.as_ref() else {
        set_dimension(
            dimensions,
            "usage",
            ProofVerdict::Unproven,
            "AF-CONTRACT-0016",
            "missing exact RecoveryUsageCoverageCatalog snapshot",
        );
        return;
    };
    let coverage = snapshot.catalog();
    if coverage.validate().is_err() {
        set_dimension(
            dimensions,
            "usage",
            ProofVerdict::Violation,
            "AF-CONTRACT-0016",
            "recovery coverage catalog has duplicate identities",
        );
        return;
    }
    let mut aggregate: BTreeMap<(String, String), UsageUpperBound> = BTreeMap::new();
    for action in actions {
        let Some(binding) = contract
            .recovery_usage_bindings
            .iter()
            .find(|binding| action_key(&binding.action) == action_key(action))
        else {
            set_dimension(
                dimensions,
                "usage",
                ProofVerdict::Violation,
                "AF-CONTRACT-0016",
                "recovery action has no exact usage binding",
            );
            continue;
        };
        if binding.activations_per_forward.validate().is_err()
            || binding.attempts_per_activation.validate().is_err()
        {
            set_dimension(
                dimensions,
                "usage",
                ProofVerdict::Violation,
                "AF-CONTRACT-0016",
                "invalid activation or attempt occurrence interval",
            );
            continue;
        }
        let can_activate = !binding.activations_per_forward.upper.is_zero();
        if can_activate && binding.attempts_per_activation.lower == 0 {
            set_dimension(
                dimensions,
                "usage",
                ProofVerdict::Violation,
                "AF-CONTRACT-0016",
                "action with possible activation must declare at least one attempt",
            );
        }
        if (action.action_kind == RecoveryActionKind::Reconciliation
            || action.action_kind == RecoveryActionKind::Compensation)
            && binding.activations_per_forward.upper.is_zero()
        {
            let required = contract
                .recovery
                .effect_recovery_clauses
                .iter()
                .any(|clause| {
                    AtomKey::of(&clause.effect) == AtomKey::of(&action.forward_effect)
                        && ((action.action_kind == RecoveryActionKind::Reconciliation
                            && clause.reconciliation
                                == af_recovery::ReconciliationRequirement::Required)
                            || (action.action_kind == RecoveryActionKind::Compensation
                                && matches!(
                                    clause.compensation,
                                    af_recovery::CompensationRequirement::Required(_)
                                )))
                });
            if required {
                set_dimension(
                    dimensions,
                    "usage",
                    ProofVerdict::Violation,
                    "AF-CONTRACT-0016",
                    "K-required action must have activation upper bound >= 1",
                );
            }
        }
        let Some(complete_meters) = coverage.meters_for(action) else {
            set_dimension(
                dimensions,
                "usage",
                ProofVerdict::Unproven,
                "AF-CONTRACT-0016",
                "missing exhaustive meter coverage rule for exact recovery action",
            );
            continue;
        };
        let mut profile_meters: Vec<_> = binding
            .per_attempt_usage
            .limits
            .iter()
            .filter(|limit| limit.phase == UsagePhase::Recovery)
            .map(|limit| limit.meter.clone())
            .collect();
        profile_meters.sort();
        let mut expected_meters = complete_meters.to_vec();
        expected_meters.sort();
        if profile_meters != expected_meters {
            set_dimension(
                dimensions,
                "usage",
                ProofVerdict::Unproven,
                "AF-CONTRACT-0016",
                "per-attempt recovery profile does not exactly match the catalog's complete meter set",
            );
            continue;
        }
        if binding
            .per_attempt_usage
            .limits
            .iter()
            .any(|limit| limit.phase != UsagePhase::Recovery)
        {
            set_dimension(
                dimensions,
                "usage",
                ProofVerdict::Violation,
                "AF-CONTRACT-0016",
                "per_attempt_usage may contain recovery phase only",
            );
        }
        let declared_forward_bounds = find_effect_bounds(&contract.effects, &action.forward_effect);
        let Some(declared_forward_bounds) = declared_forward_bounds else {
            set_dimension(
                dimensions,
                "usage",
                ProofVerdict::Unproven,
                "AF-CONTRACT-0016",
                "forward effect bound is not explicitly known",
            );
            continue;
        };
        let forward_upper = match recovery_forward_upper(contract, action, &declared_forward_bounds)
        {
            Ok(Some(upper)) => upper,
            Ok(None) => {
                set_dimension(
                    dimensions,
                    "usage",
                    ProofVerdict::Unproven,
                    "AF-CONTRACT-0016",
                    "same-atom Recovery action contribution cannot be isolated from the exact epsilon ceiling",
                );
                continue;
            }
            Err(()) => {
                set_dimension(
                    dimensions,
                    "usage",
                    ProofVerdict::Violation,
                    "AF-CONTRACT-0016",
                    "same-atom Forward/Action bounds conflict with the shared epsilon ceiling or overflow",
                );
                continue;
            }
        };
        for limit in &binding.per_attempt_usage.limits {
            if context.usage_meters.resolve(&limit.meter).is_err() {
                set_dimension(
                    dimensions,
                    "usage",
                    ProofVerdict::Violation,
                    "AF-CONTRACT-0016",
                    "recovery profile references an unresolved meter",
                );
                continue;
            }
            let Ok(amount) = multiply_usage(
                limit.upper,
                forward_upper,
                binding.activations_per_forward.upper,
                binding.attempts_per_activation.upper,
            ) else {
                set_dimension(
                    dimensions,
                    "usage",
                    ProofVerdict::Violation,
                    "AF-CONTRACT-0016",
                    "checked usage multiplication overflowed u64",
                );
                continue;
            };
            let key = (limit.meter.catalog.clone(), limit.meter.name.clone());
            let next = match aggregate.get(&key).copied() {
                Some(prior) => add_usage(prior, amount),
                None => Ok(amount),
            };
            match next {
                Ok(value) => {
                    aggregate.insert(key, value);
                }
                Err(()) => set_dimension(
                    dimensions,
                    "usage",
                    ProofVerdict::Violation,
                    "AF-CONTRACT-0016",
                    "checked usage arithmetic overflowed u64",
                ),
            }
        }
    }
    for ((catalog, name), required) in aggregate {
        let ceiling = contract.usage.limits.iter().find(|limit| {
            limit.phase == UsagePhase::Recovery
                && limit.meter.catalog == catalog
                && limit.meter.name == name
        });
        match ceiling {
            None => set_dimension(
                dimensions,
                "usage",
                ProofVerdict::Unproven,
                "AF-CONTRACT-0016",
                format!("U.recovery omits meter {catalog}/{name}; absence is not zero"),
            ),
            Some(limit) if !limit.upper.covers(required) => set_dimension(
                dimensions,
                "usage",
                ProofVerdict::Violation,
                "AF-CONTRACT-0016",
                format!("recovery aggregate exceeds U.recovery for {catalog}/{name}"),
            ),
            Some(_) => {}
        }
    }
}

fn find_effect_bounds(
    signature: &EffectSignature,
    wanted: &EffectAtom,
) -> Option<af_effect::OccurrenceBounds> {
    match &signature.row {
        EffectRow::Closed(entries) => entries
            .iter()
            .find(|entry| AtomKey::of(&entry.atom) == AtomKey::of(wanted))
            .map(|entry| entry.bounds.clone()),
        EffectRow::Open(open) => open
            .known
            .iter()
            .find(|entry| AtomKey::of(&entry.atom) == AtomKey::of(wanted))
            .map(|entry| entry.bounds.clone()),
        EffectRow::Unknown => None,
    }
}

/// When Forward and K Action use the same full atom, the single epsilon entry
/// is a ceiling for both occurrence opportunities. Recover the greatest
/// finite Forward seed compatible with that shared ceiling and every exact
/// same-atom K multiplier; do not reinterpret the combined ceiling as the
/// Forward count itself.
pub(crate) fn recovery_forward_upper(
    contract: &ComponentContract,
    action: &RecoveryActionIdentity,
    declared_bounds: &af_effect::OccurrenceBounds,
) -> Result<Option<OccurrenceCeiling>, ()> {
    let atom = AtomKey::of(&action.forward_effect);
    if atom != AtomKey::of(&action.action_effect) {
        return Ok(Some(declared_bounds.upper));
    }
    let mut multiplier = OccurrenceCeiling::Finite(0);
    for binding in &contract.recovery_action_bindings {
        let identity = &binding.identity;
        if AtomKey::of(&identity.action_effect) != atom {
            continue;
        }
        if AtomKey::of(&identity.forward_effect) != atom {
            // Another source K action consumes the same shared epsilon
            // ceiling but is based on a different Forward atom. Its exact
            // contribution cannot be subtracted from this local equation.
            return Ok(None);
        }
        let Some(usage) = contract
            .recovery_usage_bindings
            .iter()
            .find(|candidate| candidate.action == *identity)
        else {
            return Ok(None);
        };
        let action_multiplier = usage
            .activations_per_forward
            .upper
            .checked_mul(usage.attempts_per_activation.upper)
            .map_err(|_| ())?;
        multiplier = multiplier.checked_add(action_multiplier).map_err(|_| ())?;
    }
    let divisor = OccurrenceCeiling::Finite(1)
        .checked_add(multiplier)
        .map_err(|_| ())?;
    let forward_upper = match (declared_bounds.upper, divisor) {
        (OccurrenceCeiling::Unbounded, _) => OccurrenceCeiling::Unbounded,
        (OccurrenceCeiling::Finite(_), OccurrenceCeiling::Unbounded) => {
            OccurrenceCeiling::Finite(0)
        }
        (OccurrenceCeiling::Finite(ceiling), OccurrenceCeiling::Finite(divisor)) => {
            OccurrenceCeiling::Finite(ceiling / divisor)
        }
    };
    if matches!(forward_upper, OccurrenceCeiling::Finite(upper) if upper < declared_bounds.lower) {
        return Err(());
    }
    Ok(Some(forward_upper))
}

fn multiply_usage(
    bound: UsageUpperBound,
    effect: OccurrenceCeiling,
    activations: OccurrenceCeiling,
    attempts: OccurrenceCeiling,
) -> Result<UsageUpperBound, ()> {
    if [effect, activations, attempts]
        .iter()
        .any(|value| value.is_zero())
    {
        return Ok(UsageUpperBound::Finite(0));
    }
    let mut result = bound;
    for multiplier in [effect, activations, attempts] {
        match multiplier {
            OccurrenceCeiling::Unbounded => {
                if !matches!(result, UsageUpperBound::Finite(0)) {
                    result = UsageUpperBound::Unbounded;
                }
            }
            OccurrenceCeiling::Finite(value) => {
                result = match result {
                    UsageUpperBound::Finite(amount) => amount
                        .checked_mul(value)
                        .map(UsageUpperBound::Finite)
                        .ok_or(())?,
                    UsageUpperBound::Unbounded => UsageUpperBound::Unbounded,
                }
            }
        }
    }
    Ok(result)
}
fn add_usage(left: UsageUpperBound, right: UsageUpperBound) -> Result<UsageUpperBound, ()> {
    match (left, right) {
        (UsageUpperBound::Unbounded, _) | (_, UsageUpperBound::Unbounded) => {
            Ok(UsageUpperBound::Unbounded)
        }
        (UsageUpperBound::Finite(a), UsageUpperBound::Finite(b)) => {
            a.checked_add(b).map(UsageUpperBound::Finite).ok_or(())
        }
    }
}

fn check_flow_execution(
    contract: &ComponentContract,
    relation: Option<&InformationFlowExecutionCatalog>,
    dimensions: &mut [DimensionProof],
) {
    let has_constraints = !contract
        .information_flow
        .confidentiality_restrictions
        .is_empty()
        || !contract.information_flow.summary.taints.is_empty()
        || contract.information_flow.summary.instruction_authority
            != af_flow::InstructionAuthority::None
        || !contract.information_flow.flow_constraints.is_empty();
    if !has_constraints {
        return;
    }
    let Some(relation) = relation else {
        set_dimension(
            dimensions,
            "execution",
            ProofVerdict::Unproven,
            "AF-CONTRACT-0018",
            "missing exact InformationFlowExecutionCatalog snapshot",
        );
        return;
    };
    if relation.validate().is_err() {
        set_dimension(
            dimensions,
            "execution",
            ProofVerdict::Violation,
            "AF-CONTRACT-0018",
            "flow/execution relation contains duplicate exact keys",
        );
        return;
    }
    for label in &contract.information_flow.confidentiality_restrictions {
        let Some(rule) = relation
            .confidentiality_rules
            .iter()
            .find(|item| item.label == *label)
        else {
            set_dimension(
                dimensions,
                "execution",
                ProofVerdict::Unproven,
                "AF-CONTRACT-0018",
                "confidentiality label has no exact execution relation",
            );
            continue;
        };
        let Some(regions) = contract.execution.region_residency.as_ref() else {
            set_dimension(
                dimensions,
                "execution",
                ProofVerdict::Unproven,
                "AF-CONTRACT-0018",
                "confidentiality restriction requires explicit region/residency constraints",
            );
            continue;
        };
        if regions.execution_regions.is_empty() || regions.residency_regions.is_empty() {
            set_dimension(
                dimensions,
                "execution",
                ProofVerdict::Unproven,
                "AF-CONTRACT-0018",
                "empty region set is unrestricted and does not prove residency compatibility",
            );
            continue;
        }
        for execution in &regions.execution_regions {
            for residency in &regions.residency_regions {
                let allowed = match &rule.region_pairs {
                    RegionPairPolicy::FiniteExactPairs(pairs) => pairs.iter().any(|pair| {
                        pair.execution_region == *execution && pair.residency_region == *residency
                    }),
                    RegionPairPolicy::AllPairsInClosedRegionSnapshot => {
                        relation.closed_regions.contains(execution)
                            && relation.closed_regions.contains(residency)
                    }
                };
                if !allowed {
                    set_dimension(
                        dimensions,
                        "execution",
                        ProofVerdict::Violation,
                        "AF-CONTRACT-0018",
                        "execution/residency Cartesian pair is not allowed for the exact confidentiality label",
                    );
                }
            }
        }
    }
    for taint in &contract.information_flow.summary.taints {
        if let Some(rule) = relation
            .taint_rules
            .iter()
            .find(|item| item.taint == *taint)
        {
            check_trust_sandbox(
                &rule.trust_domains,
                &rule.sandbox,
                contract,
                relation,
                dimensions,
            );
        } else {
            set_dimension(
                dimensions,
                "execution",
                ProofVerdict::Unproven,
                "AF-CONTRACT-0018",
                "taint has no exact trust/sandbox relation",
            );
        }
    }
    let authority = contract.information_flow.summary.instruction_authority;
    if authority != af_flow::InstructionAuthority::None {
        if let Some(rule) = relation
            .authority_rules
            .iter()
            .find(|item| item.authority == authority)
        {
            check_trust_sandbox(
                &rule.trust_domains,
                &rule.sandbox,
                contract,
                relation,
                dimensions,
            );
        } else {
            set_dimension(
                dimensions,
                "execution",
                ProofVerdict::Unproven,
                "AF-CONTRACT-0018",
                "instruction-authority level has no exact trust/sandbox relation",
            );
        }
    }
    for constraint in &contract.information_flow.flow_constraints {
        match &constraint.sink {
            FlowSink::WorkflowOutput(port) => check_exact_sink_rule(
                relation,
                constraint.kind,
                &FlowExecutionSink::WorkflowOutput(*port),
                contract,
                dimensions,
            ),
            FlowSink::ExternalEffect(operation) => {
                let (atoms, closed) = declared_effects(&contract.effects);
                let matching_atoms: Vec<_> = atoms
                    .iter()
                    .filter(|atom| atom.operation == *operation)
                    .collect();
                if !closed {
                    set_dimension(
                        dimensions,
                        "execution",
                        ProofVerdict::Unproven,
                        "AF-CONTRACT-0018",
                        "operation-level flow sink with open epsilon lacks exact sink coverage",
                    );
                }
                if matching_atoms.is_empty() {
                    set_dimension(
                        dimensions,
                        "execution",
                        ProofVerdict::Unproven,
                        "AF-CONTRACT-0018",
                        "operation-level sink has no known exact effect atom to resolve",
                    );
                    continue;
                }
                let partition = contract
                    .effect_flow_partitions
                    .iter()
                    .find(|part| part.operation == *operation && part.kind == constraint.kind);
                if let Some(partition) = partition {
                    for atom in &partition.allowed_effects {
                        check_exact_sink_rule(
                            relation,
                            constraint.kind,
                            &FlowExecutionSink::ExactEffect(atom.clone()),
                            contract,
                            dimensions,
                        );
                    }
                } else {
                    // A coarse legacy sink applies conservatively to every
                    // declared exact atom for the operation.
                    for atom in matching_atoms {
                        check_exact_sink_rule(
                            relation,
                            constraint.kind,
                            &FlowExecutionSink::ExactEffect((**atom).clone()),
                            contract,
                            dimensions,
                        );
                    }
                }
            }
        }
    }
}

fn check_exact_sink_rule(
    relation: &InformationFlowExecutionCatalog,
    kind: af_flow::FlowKind,
    sink: &FlowExecutionSink,
    contract: &ComponentContract,
    dimensions: &mut [DimensionProof],
) {
    if let Some(rule) = relation
        .sink_rules
        .iter()
        .find(|item| item.kind == kind && item.sink == *sink)
    {
        check_trust_sandbox(
            &rule.trust_domains,
            &rule.sandbox,
            contract,
            relation,
            dimensions,
        );
    } else {
        set_dimension(
            dimensions,
            "execution",
            ProofVerdict::Unproven,
            "AF-CONTRACT-0018",
            "flow sink lacks an exact kind/sink execution relation",
        );
    }
}

fn check_trust_sandbox(
    policy: &TrustDomainPolicy,
    floor: &SandboxFloor,
    contract: &ComponentContract,
    relation: &InformationFlowExecutionCatalog,
    dimensions: &mut [DimensionProof],
) {
    match policy {
        TrustDomainPolicy::NoAdditionalTrustRequirement => {}
        TrustDomainPolicy::ExactAllowlist(allowlist) => match &contract.execution.trust_domain {
            Some(domain) if allowlist.contains(domain) => {}
            Some(_) => set_dimension(
                dimensions,
                "execution",
                ProofVerdict::Violation,
                "AF-CONTRACT-0018",
                "X trust domain is outside exact relation allowlist",
            ),
            None => set_dimension(
                dimensions,
                "execution",
                ProofVerdict::Unproven,
                "AF-CONTRACT-0018",
                "relation requires a trust domain but X is unrestricted",
            ),
        },
        TrustDomainPolicy::AllDomainsInClosedTrustSnapshot => {
            match &contract.execution.trust_domain {
                Some(domain) if relation.closed_trust_domains.contains(domain) => {}
                Some(_) => set_dimension(
                    dimensions,
                    "execution",
                    ProofVerdict::Violation,
                    "AF-CONTRACT-0018",
                    "X trust domain is outside the exact closed snapshot",
                ),
                None => set_dimension(
                    dimensions,
                    "execution",
                    ProofVerdict::Unproven,
                    "AF-CONTRACT-0018",
                    "unrestricted X trust does not prove membership in a closed trust snapshot",
                ),
            }
        }
    }
    match floor {
        SandboxFloor::NoAdditionalMinimum => {}
        SandboxFloor::Minimum(minimum) => match &contract.execution.sandbox {
            Some(sandbox) if sandbox.minimum >= *minimum => {}
            Some(_) => set_dimension(
                dimensions,
                "execution",
                ProofVerdict::Violation,
                "AF-CONTRACT-0018",
                "X sandbox minimum is below relation floor",
            ),
            None => set_dimension(
                dimensions,
                "execution",
                ProofVerdict::Unproven,
                "AF-CONTRACT-0018",
                "X has no sandbox minimum required by the relation",
            ),
        },
    }
}

fn set_dimension(
    dimensions: &mut [DimensionProof],
    name: &'static str,
    verdict: ProofVerdict,
    rule: &'static str,
    reason: impl Into<String>,
) {
    if let Some(item) = dimensions.iter_mut().find(|item| item.dimension == name) {
        if item.verdict == ProofVerdict::Violation {
            return;
        }
        if verdict == ProofVerdict::Violation
            || (item.verdict == ProofVerdict::Proven && verdict != ProofVerdict::Proven)
        {
            item.verdict = verdict;
            item.rule_id = rule;
            item.reason = reason.into();
        }
    }
}

fn set_dimension_at(
    dimensions: &mut [DimensionProof],
    name: &'static str,
    verdict: ProofVerdict,
    rule: &'static str,
    reference: impl Into<String>,
    reason: impl Into<String>,
) {
    let reference = reference.into();
    let reason = reason.into();
    set_dimension(dimensions, name, verdict, rule, reason.clone());
    if let Some(item) = dimensions.iter_mut().find(|item| item.dimension == name)
        && item.rule_id == rule
        && item.reason == reason
    {
        item.reference = Some(reference);
    }
}

fn action_key(action: &RecoveryActionIdentity) -> (AtomKey, u32, AtomKey) {
    (
        AtomKey::of(&action.forward_effect),
        match action.action_kind {
            RecoveryActionKind::Reconciliation => 1,
            RecoveryActionKind::Compensation => 2,
        },
        AtomKey::of(&action.action_effect),
    )
}

#[cfg(test)]
mod tests {
    use super::{add_usage, graph_has_cycle, multiply_usage};
    use af_effect::{AtomKey, EffectAtom, EffectOperationRef, OccurrenceCeiling, ResourceSelector};
    use af_usage::UsageUpperBound;
    use std::collections::BTreeMap;

    fn atom(name: &str) -> AtomKey {
        AtomKey::of(&EffectAtom {
            operation: EffectOperationRef {
                catalog: "agentflow.effects@1".into(),
                name: name.into(),
            },
            resource_selector: ResourceSelector {
                requirement_port_no: 1,
                scope_bindings: vec![],
            },
            qualifiers: vec![],
        })
    }

    #[test]
    fn recovery_graph_cycle_detection_uses_full_atom_keys() {
        let forward = atom("charge");
        let reverse = atom("reverse");
        let mut acyclic = BTreeMap::new();
        acyclic.insert(forward.clone(), vec![reverse.clone()]);
        assert!(!graph_has_cycle(&acyclic));
        acyclic.insert(reverse, vec![forward]);
        assert!(graph_has_cycle(&acyclic));
    }

    #[test]
    fn recovery_usage_multiplication_and_addition_never_wrap() {
        assert_eq!(
            multiply_usage(
                UsageUpperBound::Finite(3),
                OccurrenceCeiling::Finite(2),
                OccurrenceCeiling::Finite(1),
                OccurrenceCeiling::Finite(2),
            ),
            Ok(UsageUpperBound::Finite(12)),
        );
        assert!(
            multiply_usage(
                UsageUpperBound::Finite(u64::MAX),
                OccurrenceCeiling::Finite(2),
                OccurrenceCeiling::Finite(1),
                OccurrenceCeiling::Finite(1),
            )
            .is_err()
        );
        assert_eq!(
            multiply_usage(
                UsageUpperBound::Unbounded,
                OccurrenceCeiling::Finite(0),
                OccurrenceCeiling::Unbounded,
                OccurrenceCeiling::Unbounded,
            ),
            Ok(UsageUpperBound::Finite(0)),
        );
        assert!(
            add_usage(
                UsageUpperBound::Finite(u64::MAX),
                UsageUpperBound::Finite(1)
            )
            .is_err()
        );
    }
}
