//! Fresh D6-D8 validation scoped to one D5 proof-context lifetime.

use crate::error::{PortableProofError, ValidationStage};
use crate::reconstruction::{ReconstructedStaticContext, StaticContextProjection, value_to_json};
use crate::support::SupportedPortableStaticInputs;
use af_computation::{
    AwaitAuthorityStatus, AwaitInteractionStatus, AwaitProvenanceStatus, ResolvedTarget,
    ResolverSnapshotIdentity, StaticComputationResolutionSnapshotBuilder, resolve_computation_leaf,
};
use af_contract::{
    CheckedComponentContract, ComponentContract, StaticContractProofContext, canonicalize_contract,
    check_contract, component_contract_digest,
};
use af_core::{DigestRef, Value, decode};
use af_ir::{
    CheckedIrDocument, ComputationBody, SemanticDataflowAnalysis, SemanticIrDocument,
    SubflowIrUniverse,
};
use af_semantic::{DigestRefValue, TypeSchemaRef};
use serde::{Deserialize, Serialize};
use sha2::Digest as _;
use std::collections::{BTreeMap, BTreeSet, HashMap};

const MAX_RESOLVER_TARGETS: usize = 4096;
const MAX_RESOLVER_GRAPH_WORK: usize = 2_000_000;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContentReference {
    digest: DigestRefValue,
    type_schema: TypeSchemaRef,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResolverManifest {
    caller_ir: ContentReference,
    context_id: u32,
    components: Vec<ContentReference>,
    subflow_targets: Vec<ResolverTargetManifest>,
    source_associations: Vec<SourceAssociationManifest>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResolverTargetManifest {
    target_semantic_ir_digest: DigestRefValue,
    target_ir_content: Option<ContentReference>,
    exported_contract: Option<ContentReference>,
    direct_dependencies: Option<Vec<DigestRefValue>>,
    transitive_dependencies: Option<Vec<SubflowTargetIdentity>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct SubflowTargetIdentity {
    semantic_ir_digest: DigestRefValue,
    exported_contract_digest: Option<DigestRefValue>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceAssociationManifest {
    caller_ir: ContentReference,
    source_node: u32,
    target: SourceAssociationTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SourceAssociationTarget {
    Invoke(ContentReference),
    CallSubflow(ResolvedSubflowAssociation),
    AwaitSignal(TypeSchemaRef),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResolvedSubflowAssociation {
    target_semantic_ir_digest: DigestRefValue,
    exported_contract_digest: Option<DigestRefValue>,
}

#[derive(Debug, Clone)]
struct ContractAsset {
    reference: ContentReference,
    bytes: Vec<u8>,
    raw: ComponentContract,
}

#[derive(Debug, Clone)]
struct IrAsset {
    reference: ContentReference,
    bytes: Vec<u8>,
    document: SemanticIrDocument,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CheckedContractProjection {
    pub reference: DigestRefValue,
    pub canonical_content_sha256: String,
    pub proven_dimension_count: usize,
    pub all_dimensions_proven: bool,
    pub action_guard_compatibility_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CheckedIrProjection {
    pub reference: DigestRefValue,
    pub canonical_content_sha256: String,
    pub computation_count: usize,
    pub phase3c_computation_entry_count: usize,
    pub phase3c_entry_value_count: usize,
    pub phase3c_entry_resource_count: usize,
    pub phase3c_producer_count: usize,
    pub phase3c_workflow_exit_value_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceAssociationProjection {
    pub caller_ir: DigestRefValue,
    pub source_node: u32,
    pub target: SourceAssociationTargetProjection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceAssociationTargetProjection {
    Invoke(ContentReferenceProjection),
    CallSubflow {
        semantic_ir_digest: DigestRefValue,
        exported_contract_digest: Option<DigestRefValue>,
    },
    AwaitSignal(TypeSchemaRef),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContentReferenceProjection {
    pub digest: DigestRefValue,
    pub type_schema: TypeSchemaRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolverTargetProjection {
    pub semantic_ir_digest: DigestRefValue,
    pub exported_contract_digest: Option<DigestRefValue>,
    pub direct_dependencies: Vec<DigestRefValue>,
    pub transitive_dependencies: Vec<SubflowTargetIdentityProjection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SubflowTargetIdentityProjection {
    pub semantic_ir_digest: DigestRefValue,
    pub exported_contract_digest: Option<DigestRefValue>,
}

/// Owned D8 result. Checked wrappers and the resolver snapshot are deliberately
/// scoped to construction; this projection carries no D9/D10 capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconstructedStaticInputs {
    context: StaticContextProjection,
    resolver_identity: ResolverSnapshotIdentity,
    checked_contracts: Vec<CheckedContractProjection>,
    checked_irs: Vec<CheckedIrProjection>,
    source_associations: Vec<SourceAssociationProjection>,
    resolver_targets: Vec<ResolverTargetProjection>,
    await_leaf_count: usize,
    await_unknown_external_count: usize,
}

impl ReconstructedStaticInputs {
    #[must_use]
    pub fn context(&self) -> &StaticContextProjection {
        &self.context
    }

    #[must_use]
    pub fn resolver_identity(&self) -> &ResolverSnapshotIdentity {
        &self.resolver_identity
    }

    #[must_use]
    pub fn checked_contracts(&self) -> &[CheckedContractProjection] {
        &self.checked_contracts
    }

    #[must_use]
    pub fn checked_irs(&self) -> &[CheckedIrProjection] {
        &self.checked_irs
    }

    #[must_use]
    pub fn source_associations(&self) -> &[SourceAssociationProjection] {
        &self.source_associations
    }

    #[must_use]
    pub fn resolver_targets(&self) -> &[ResolverTargetProjection] {
        &self.resolver_targets
    }

    #[must_use]
    pub fn await_leaf_count(&self) -> usize {
        self.await_leaf_count
    }

    #[must_use]
    pub fn await_unknown_external_count(&self) -> usize {
        self.await_unknown_external_count
    }

    /// A deterministic, ordinary data projection for independent conformance
    /// comparison. It contains no Checked wrappers, pointer identities or
    /// RootExport verdict.
    #[must_use]
    pub fn typed_projection(&self) -> serde_json::Value {
        let context = serde_json::to_value(&self.context)
            .expect("the closed D5 projection has a Serialize implementation");
        let checked_contracts = serde_json::to_value(&self.checked_contracts)
            .expect("the closed D6 projection has a Serialize implementation");
        let checked_irs = serde_json::to_value(&self.checked_irs)
            .expect("the closed D7 projection has a Serialize implementation");
        let source_associations = serde_json::to_value(&self.source_associations)
            .expect("the closed source association projection has a Serialize implementation");
        let resolver_targets = serde_json::to_value(&self.resolver_targets)
            .expect("the closed D8 target projection has a Serialize implementation");
        let resolver_components = self.resolver_identity.component_contract_digests();
        let resolver_subflows = self
            .resolver_identity
            .subflow_targets()
            .iter()
            .map(|target| {
                serde_json::json!({
                    "semantic_ir_digest": target.semantic_ir_digest(),
                    "direct_dependencies": target.direct_dependencies(),
                    "exported_contract_digest": target.exported_contract_digest(),
                })
            })
            .collect::<Vec<_>>();
        serde_json::json!({
            "context": context,
            "checked_contracts": checked_contracts,
            "checked_irs": checked_irs,
            "source_associations": source_associations,
            "resolver_identity": {
                "component_contract_digests": resolver_components,
                "subflow_targets": resolver_subflows,
            },
            "resolver_targets": resolver_targets,
            "await_leaf_count": self.await_leaf_count,
            "await_unknown_external_count": self.await_unknown_external_count,
        })
    }
}

pub(crate) fn reconstruct_static_inputs(
    supported: &SupportedPortableStaticInputs,
    context_id: u32,
    resolver_id: u32,
) -> Result<ReconstructedStaticInputs, PortableProofError> {
    let context = supported.reconstruct_context(context_id)?;
    let root = supported.content().value();
    let resolver = find_resolver(root, resolver_id)?;
    if resolver.context_id != context_id {
        return Err(error(
            ValidationStage::D8,
            "ResolverContextMismatch",
            "$.resolver_dependency_manifests",
            "resolver and D5 context use different context ids",
        ));
    }

    context.with_proof_context(|proof_context| {
        reconstruct_with_context(root, &context, proof_context, &resolver)
    })
}

fn reconstruct_with_context(
    root: &Value,
    context_owner: &ReconstructedStaticContext,
    proof_context: &StaticContractProofContext<'_>,
    resolver: &ResolverManifest,
) -> Result<ReconstructedStaticInputs, PortableProofError> {
    let mut contract_refs = resolver.components.clone();
    for target in &resolver.subflow_targets {
        if let Some(export) = &target.exported_contract {
            contract_refs.push(export.clone());
        }
    }
    contract_refs.sort_by(|left, right| digest_key(&left.digest).cmp(&digest_key(&right.digest)));
    let mut unique_contract_refs = Vec::with_capacity(contract_refs.len());
    for reference in contract_refs {
        if let Some(previous) = unique_contract_refs.last() {
            let previous: &ContentReference = previous;
            if previous.digest == reference.digest {
                if previous != &reference {
                    return Err(error(
                        ValidationStage::D6,
                        "ConflictingContractReference",
                        "$.resolver",
                        "one contract digest is associated with conflicting type pins",
                    ));
                }
                continue;
            }
        }
        unique_contract_refs.push(reference);
    }
    let contract_assets = load_contract_assets(root, proof_context, &unique_contract_refs)?;
    let (checked_contracts, checked_contract_projections, contracts_by_digest) =
        check_contracts(&contract_assets, proof_context)?;
    let ir_assets = load_ir_assets(root, proof_context, resolver)?;
    let (checked_irs, dataflows, checked_ir_projections, ir_by_digest) =
        check_irs(&ir_assets, proof_context)?;

    if resolver.subflow_targets.len() > MAX_RESOLVER_TARGETS {
        return Err(resource_limit(
            "$.resolver.subflow_targets",
            "resolver target count limit exceeded after D5-D7 validation",
        ));
    }

    let mut subflows = SubflowIrUniverse::new();
    let mut target_set = BTreeSet::new();
    for target in &resolver.subflow_targets {
        if !valid_digest(&target.target_semantic_ir_digest, "semantic-ir")
            || !target_set.insert(digest_key(&target.target_semantic_ir_digest))
        {
            return Err(error(
                ValidationStage::D8,
                "DuplicateOrInvalidTarget",
                "$.resolver.subflow_targets",
                "target pins must be unique exact SemanticIrDigest values",
            ));
        }
        let target_ref = target.target_ir_content.as_ref().ok_or_else(|| {
            error(
                ValidationStage::D8,
                "IncompleteResolver",
                "$.resolver.subflow_targets.target_ir_content",
                "formal SubflowIrUniverse reconstruction requires exact target IR content; None is not an empty target",
            )
        })?;
        if target_ref.digest != target.target_semantic_ir_digest {
            return Err(error(
                ValidationStage::D8,
                "TargetPinMismatch",
                "$.resolver.subflow_targets.target_ir_content",
                "target IR content reference and target semantic digest differ",
            ));
        }
        let index = *ir_by_digest
            .get(&digest_key(&target_ref.digest))
            .ok_or_else(|| {
                error(
                    ValidationStage::D8,
                    "MissingTargetContent",
                    "$.resolver.subflow_targets.target_ir_content",
                    "target IR reference does not resolve to a validated D7 asset",
                )
            })?;
        let inserted = subflows
            .insert(&checked_irs[index], proof_context.modules)
            .map_err(|failure| {
                error(
                    ValidationStage::D8,
                    "TargetInsertionFailed",
                    "$.resolver.subflow_targets",
                    failure.to_string(),
                )
            })?;
        if inserted != target.target_semantic_ir_digest {
            return Err(error(
                ValidationStage::D8,
                "TargetPinMismatch",
                "$.resolver.subflow_targets",
                "formal target insertion recomputed a different SemanticIrDigest",
            ));
        }
    }

    for (index, ir) in checked_irs.iter().enumerate() {
        ir.check_subflow_interfaces(&dataflows[index], &subflows)
            .map_err(|failure| {
                error(
                    ValidationStage::D8,
                    "SubflowInterfaceRejected",
                    "$.ir_assets",
                    failure.to_string(),
                )
            })?;
    }

    let mut export_digest_by_target = HashMap::new();
    for target in &resolver.subflow_targets {
        if let Some(export_ref) = &target.exported_contract {
            let contract_index = *contracts_by_digest
                .get(&digest_key(&export_ref.digest))
                .ok_or_else(|| {
                    error(
                        ValidationStage::D8,
                        "MissingExportContract",
                        "$.resolver.subflow_targets.exported_contract",
                        "export association does not resolve to a freshly D6-checked contract",
                    )
                })?;
            if contract_assets[contract_index].reference != *export_ref {
                return Err(error(
                    ValidationStage::D8,
                    "ExportContentReferenceMismatch",
                    "$.resolver.subflow_targets.exported_contract",
                    "exported contract's exact typed content reference differs from the checked asset",
                ));
            }
            export_digest_by_target.insert(
                digest_key(&target.target_semantic_ir_digest),
                export_ref.digest.clone(),
            );
        }
    }

    let caller_index = *ir_by_digest
        .get(&digest_key(&resolver.caller_ir.digest))
        .ok_or_else(|| {
            error(
                ValidationStage::D7,
                "MissingCallerIr",
                "$.resolver.caller_ir",
                "caller IR is not a validated D7 asset",
            )
        })?;
    let caller_asset = &ir_assets[caller_index];
    if caller_asset.reference != resolver.caller_ir {
        return Err(error(
            ValidationStage::D7,
            "CallerContentReferenceMismatch",
            "$.resolver.caller_ir",
            "caller IR typed content reference differs from validated asset",
        ));
    }

    let mut builder = StaticComputationResolutionSnapshotBuilder::new(proof_context, &subflows);
    for reference in &resolver.components {
        let index = contracts_by_digest
            .get(&digest_key(&reference.digest))
            .ok_or_else(|| {
                error(
                    ValidationStage::D8,
                    "MissingComponent",
                    "$.resolver.components",
                    "component inventory entry did not produce a D6 Checked contract",
                )
            })?;
        let checked = &checked_contracts[*index];
        if !std::ptr::eq(checked.proof_context(), proof_context) {
            return Err(error(
                ValidationStage::D6,
                "ProofContextIdentityMismatch",
                "$.contract_assets",
                "D6 checked contract belongs to a different context object",
            ));
        }
        builder = builder.component(checked).map_err(|failure| {
            error(
                ValidationStage::D8,
                "ComponentInventoryRejected",
                "$.resolver.components",
                failure.to_string(),
            )
        })?;
    }
    for target in &resolver.subflow_targets {
        if let Some(export_ref) = &target.exported_contract {
            let contract_index = *contracts_by_digest
                .get(&digest_key(&export_ref.digest))
                .ok_or_else(|| {
                    error(
                        ValidationStage::D8,
                        "MissingExportContract",
                        "$.resolver.subflow_targets.exported_contract",
                        "exported contract is not checked",
                    )
                })?;
            builder = builder
                .subflow_export(
                    &target.target_semantic_ir_digest,
                    &checked_contracts[contract_index],
                )
                .map_err(|failure| {
                    error(
                        ValidationStage::D8,
                        "ExportAssociationRejected",
                        "$.resolver.subflow_targets.exported_contract",
                        failure.to_string(),
                    )
                })?;
        }
    }
    let snapshot = builder.build();
    if !snapshot.is_same_context(proof_context) {
        return Err(error(
            ValidationStage::D8,
            "ProofContextIdentityMismatch",
            "$.resolver",
            "fresh resolver snapshot does not retain the D5 context pointer",
        ));
    }
    let resolver_identity = snapshot.resolver_identity();
    compare_component_inventory(resolver, &resolver_identity)?;
    let resolver_targets =
        compare_target_facts(resolver, &snapshot, &subflows, &export_digest_by_target)?;

    let (source_associations, await_leaf_count, await_unknown_external_count) =
        derive_source_associations(
            resolver,
            &ir_assets[caller_index],
            &checked_irs[caller_index],
            proof_context.modules,
            proof_context,
            &snapshot,
            &contracts_by_digest,
            &contract_assets,
            &checked_contracts,
        )?;
    compare_source_associations(
        &resolver.source_associations,
        &source_associations,
        &resolver.caller_ir.digest,
    )?;

    Ok(ReconstructedStaticInputs {
        context: context_owner.projection().clone(),
        resolver_identity,
        checked_contracts: checked_contract_projections,
        checked_irs: checked_ir_projections,
        source_associations,
        resolver_targets,
        await_leaf_count,
        await_unknown_external_count,
    })
}

fn find_resolver(root: &Value, resolver_id: u32) -> Result<ResolverManifest, PortableProofError> {
    let row = list_field(root, "resolver_dependency_manifests", "$")?
        .iter()
        .find(|row| u32_field(row, "id").ok() == Some(resolver_id))
        .ok_or_else(|| {
            error(
                ValidationStage::D8,
                "UnknownResolver",
                "$.resolver_dependency_manifests",
                "resolver id does not resolve",
            )
        })?;
    let content = field(row, "content", "$.resolver_dependency_manifests")?;
    from_value(content, "$.resolver_dependency_manifests.content")
}

fn load_contract_assets(
    root: &Value,
    context: &StaticContractProofContext<'_>,
    references: &[ContentReference],
) -> Result<Vec<ContractAsset>, PortableProofError> {
    if references.len() > MAX_RECONSTRUCTED_ITEMS {
        return Err(resource_limit(
            "$.resolver.components",
            "component inventory limit exceeded",
        ));
    }
    let shape = context
        .modules
        .shape_of("agentflow.component.contract@1/ComponentContract")
        .map_err(|failure| {
            error(
                ValidationStage::D6,
                "UnsupportedCurrentSchema",
                "$.contract_assets",
                failure.to_string(),
            )
        })?;
    let expected_schema = schema_ref(
        context.modules,
        "agentflow.component.contract@1",
        "ComponentContract",
    )?;
    let mut seen = BTreeSet::new();
    let mut assets = Vec::with_capacity(references.len());
    for reference in references {
        if !seen.insert(digest_key(&reference.digest)) {
            return Err(error(
                ValidationStage::D6,
                "DuplicateComponentReference",
                "$.resolver.components",
                "component inventory contains a duplicate exact digest",
            ));
        }
        if reference.type_schema != expected_schema
            || !valid_digest(&reference.digest, "component-contract")
        {
            return Err(error(
                ValidationStage::D6,
                "InvalidComponentReference",
                "$.resolver.components",
                "component ref must use the exact registered root schema and ComponentContractDigest",
            ));
        }
        let bytes = find_asset_bytes(root, "contract_assets", reference)?;
        let value = decode(&shape, bytes).map_err(|failure| {
            error(
                ValidationStage::D6,
                "InvalidContractContent",
                "$.contract_assets",
                failure.to_string(),
            )
        })?;
        let raw: ComponentContract = from_value(&value, "$.contract_assets.canonical_content")?;
        assets.push(ContractAsset {
            reference: reference.clone(),
            bytes: bytes.to_vec(),
            raw,
        });
    }
    Ok(assets)
}

fn check_contracts<'context, 'universe>(
    assets: &[ContractAsset],
    context: &'context StaticContractProofContext<'universe>,
) -> Result<FreshContractSet<'context, 'universe>, PortableProofError> {
    let mut checked = Vec::with_capacity(assets.len());
    let mut projections = Vec::with_capacity(assets.len());
    let mut by_digest = BTreeMap::new();
    for (index, asset) in assets.iter().enumerate() {
        let checked_contract = check_contract(&asset.raw, context).map_err(|report| {
            let failed = report
                .dimensions
                .iter()
                .filter(|dimension| dimension.verdict != af_contract::ProofVerdict::Proven)
                .map(|dimension| format!("{}={:?}", dimension.dimension, dimension.verdict))
                .collect::<Vec<_>>()
                .join(", ");
            let guards = report
                .action_guard_compatibility
                .iter()
                .map(|guard| guard.verdict().as_str())
                .collect::<Vec<_>>()
                .join(",");
            error(
                ValidationStage::D6,
                "ContractValidationFailed",
                "$.contract_assets",
                if failed.is_empty() {
                    format!("formal check_contract rejected the component; action_guard_compatibility=[{guards}]")
                } else {
                    format!("{failed}; action_guard_compatibility=[{guards}]")
                },
            )
        })?;
        if !std::ptr::eq(checked_contract.proof_context(), context) {
            return Err(error(
                ValidationStage::D6,
                "ProofContextIdentityMismatch",
                "$.contract_assets",
                "formal checked wrapper is not bound to this D5 context",
            ));
        }
        let canonical = canonicalize_contract(&asset.raw, context).map_err(|failure| {
            error(
                ValidationStage::D6,
                "ContractCanonicalizationFailed",
                "$.contract_assets",
                failure.to_string(),
            )
        })?;
        let canonical_bytes = canonical.ace1_bytes(context).map_err(|failure| {
            error(
                ValidationStage::D6,
                "ContractEncodingFailed",
                "$.contract_assets",
                failure.to_string(),
            )
        })?;
        if canonical_bytes != asset.bytes {
            return Err(error(
                ValidationStage::D6,
                "SemanticCanonicalMismatch",
                "$.contract_assets.canonical_content",
                "freshly validated contract does not reproduce the exact pinned canonical bytes",
            ));
        }
        let digest = component_contract_digest(&checked_contract, context).map_err(|failure| {
            error(
                ValidationStage::D6,
                "ContractDigestFailed",
                "$.contract_assets",
                failure.to_string(),
            )
        })?;
        if !digest_matches(&asset.reference.digest, &digest) {
            return Err(error(
                ValidationStage::D6,
                "ContractDigestMismatch",
                "$.contract_assets.reference.digest",
                "fresh canonical contract digest differs from the pinned digest",
            ));
        }
        if by_digest
            .insert(digest_key(&asset.reference.digest), index)
            .is_some()
        {
            return Err(error(
                ValidationStage::D6,
                "DuplicateComponentDigest",
                "$.resolver.components",
                "two checked component assets share one exact digest",
            ));
        }
        projections.push(CheckedContractProjection {
            reference: asset.reference.digest.clone(),
            canonical_content_sha256: sha256_hex(&canonical_bytes),
            proven_dimension_count: checked_contract.proof().dimensions.len(),
            all_dimensions_proven: checked_contract
                .proof()
                .dimensions
                .iter()
                .all(|dimension| dimension.verdict == af_contract::ProofVerdict::Proven),
            action_guard_compatibility_count: checked_contract
                .proof()
                .action_guard_compatibility
                .len(),
        });
        checked.push(checked_contract);
    }
    Ok((checked, projections, by_digest))
}

fn load_ir_assets(
    root: &Value,
    context: &StaticContractProofContext<'_>,
    resolver: &ResolverManifest,
) -> Result<Vec<IrAsset>, PortableProofError> {
    let shape = context
        .modules
        .shape_of("agentflow.semantic.ir@1/SemanticIrDocument")
        .map_err(|failure| {
            error(
                ValidationStage::D7,
                "UnsupportedCurrentSchema",
                "$.ir_assets",
                failure.to_string(),
            )
        })?;
    let expected_schema = schema_ref(
        context.modules,
        "agentflow.semantic.ir@1",
        "SemanticIrDocument",
    )?;
    let mut refs = vec![resolver.caller_ir.clone()];
    let mut targets = BTreeSet::new();
    for target in &resolver.subflow_targets {
        if !targets.insert(digest_key(&target.target_semantic_ir_digest)) {
            return Err(error(
                ValidationStage::D8,
                "DuplicateTargetAssociation",
                "$.resolver.subflow_targets",
                "duplicate target identity",
            ));
        }
        if let Some(target_ir) = &target.target_ir_content {
            refs.push(target_ir.clone());
        }
    }
    refs.sort_by(|a, b| digest_key(&a.digest).cmp(&digest_key(&b.digest)));
    refs.dedup_by(|a, b| a.digest == b.digest);
    if refs.len() > MAX_RECONSTRUCTED_ITEMS {
        return Err(resource_limit("$.resolver", "IR asset set limit exceeded"));
    }
    let mut assets = Vec::with_capacity(refs.len());
    for reference in refs {
        if reference.type_schema != expected_schema
            || !valid_digest(&reference.digest, "semantic-ir")
        {
            return Err(error(
                ValidationStage::D7,
                "InvalidIrReference",
                "$.resolver",
                "IR ref must use the exact registered root schema and SemanticIrDigest",
            ));
        }
        let bytes = find_asset_bytes(root, "ir_assets", &reference)?;
        let value = decode(&shape, bytes).map_err(|failure| {
            error(
                ValidationStage::D7,
                "InvalidIrContent",
                "$.ir_assets",
                failure.to_string(),
            )
        })?;
        let document: SemanticIrDocument = from_value(&value, "$.ir_assets.canonical_content")?;
        assets.push(IrAsset {
            reference,
            bytes: bytes.to_vec(),
            document,
        });
    }
    Ok(assets)
}

fn check_irs<'ir>(
    assets: &'ir [IrAsset],
    context: &StaticContractProofContext<'_>,
) -> Result<FreshIrSet<'ir>, PortableProofError> {
    let checked = assets
        .iter()
        .map(|asset| {
            asset
                .document
                .check(context.schema_universe)
                .map_err(|failure| {
                    error(
                        ValidationStage::D7,
                        "SemanticIrValidationFailed",
                        "$.ir_assets",
                        failure.to_string(),
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut dataflows = Vec::with_capacity(checked.len());
    let mut projections = Vec::with_capacity(checked.len());
    let mut by_digest = BTreeMap::new();
    for (index, (asset, checked_ir)) in assets.iter().zip(&checked).enumerate() {
        let dataflow = checked_ir.analyze_dataflow().map_err(|failure| {
            error(
                ValidationStage::D7,
                "Phase3CRejected",
                "$.ir_assets",
                failure.to_string(),
            )
        })?;
        let canonical = checked_ir
            .source()
            .canonical_bytes(context.modules)
            .map_err(|failure| {
                error(
                    ValidationStage::D7,
                    "IrCanonicalizationFailed",
                    "$.ir_assets",
                    failure.to_string(),
                )
            })?;
        if canonical != asset.bytes {
            return Err(error(
                ValidationStage::D7,
                "SemanticCanonicalMismatch",
                "$.ir_assets.canonical_content",
                "fresh Phase 3B/3C IR does not preserve the exact pinned canonical bytes",
            ));
        }
        let digest = checked_ir
            .source()
            .digest(context.modules)
            .map_err(|failure| {
                error(
                    ValidationStage::D7,
                    "IrDigestFailed",
                    "$.ir_assets",
                    failure.to_string(),
                )
            })?;
        if !digest_matches(&asset.reference.digest, &digest) {
            return Err(error(
                ValidationStage::D7,
                "IrDigestMismatch",
                "$.ir_assets.reference.digest",
                "fresh canonical SemanticIrDigest differs from the pinned digest",
            ));
        }
        if by_digest
            .insert(digest_key(&asset.reference.digest), index)
            .is_some()
        {
            return Err(error(
                ValidationStage::D7,
                "DuplicateIrDigest",
                "$.ir_assets",
                "duplicate exact SemanticIrDigest asset",
            ));
        }
        projections.push(CheckedIrProjection {
            reference: asset.reference.digest.clone(),
            canonical_content_sha256: sha256_hex(&canonical),
            computation_count: checked_ir.source().computations.len(),
            phase3c_computation_entry_count: dataflow.entry_values().len(),
            phase3c_entry_value_count: dataflow.entry_values().values().map(BTreeSet::len).sum(),
            phase3c_entry_resource_count: dataflow
                .entry_resources()
                .values()
                .map(BTreeSet::len)
                .sum(),
            phase3c_producer_count: dataflow.producer_locations().len(),
            phase3c_workflow_exit_value_count: dataflow.workflow_exit_values().len(),
        });
        dataflows.push(dataflow);
    }
    Ok((checked, dataflows, projections, by_digest))
}

fn compare_component_inventory(
    manifest: &ResolverManifest,
    identity: &ResolverSnapshotIdentity,
) -> Result<(), PortableProofError> {
    let mut expected = manifest
        .components
        .iter()
        .map(|reference| reference.digest.clone())
        .collect::<Vec<_>>();
    expected.sort_by_key(digest_key);
    let mut actual = identity.component_contract_digests().to_vec();
    actual.sort_by_key(digest_key);
    if expected != actual {
        return Err(error(
            ValidationStage::D8,
            "ComponentInventoryMismatch",
            "$.resolver.components",
            "fresh Builder component pins differ from the declared exact inventory",
        ));
    }
    Ok(())
}

fn compare_target_facts(
    manifest: &ResolverManifest,
    snapshot: &af_computation::StaticComputationResolutionSnapshot<'_, '_, '_>,
    subflows: &SubflowIrUniverse,
    exports: &HashMap<DigestKey, DigestRefValue>,
) -> Result<Vec<ResolverTargetProjection>, PortableProofError> {
    let mut output = Vec::with_capacity(manifest.subflow_targets.len());
    let mut graph_work = 0usize;
    let target_digests = subflows.digests().into_iter().cloned().collect::<Vec<_>>();
    let target_indices = target_digests
        .iter()
        .enumerate()
        .map(|(index, digest)| (digest_key(digest), index))
        .collect::<BTreeMap<_, _>>();
    let mut direct_edges = Vec::with_capacity(target_digests.len());
    for digest in &target_digests {
        add_graph_work(&mut graph_work, 1)?;
        let dependencies = subflows
            .target(digest)
            .map(af_ir::SubflowTarget::dependencies)
            .ok_or_else(|| {
                error(
                    ValidationStage::D8,
                    "MissingTarget",
                    "$.resolver.subflow_targets",
                    "fresh target disappeared while materializing the resolver graph",
                )
            })?;
        let mut edges = Vec::with_capacity(dependencies.len());
        for dependency in dependencies {
            add_graph_work(&mut graph_work, 1)?;
            let index = target_indices
                .get(&digest_key(dependency))
                .copied()
                .ok_or_else(|| {
                    error(
                        ValidationStage::D8,
                        "MissingTarget",
                        "$.resolver.subflow_targets.direct_dependencies",
                        "direct dependency target is absent from exact target inventory",
                    )
                })?;
            edges.push(index);
        }
        direct_edges.push(edges);
    }
    // Bound worst-case closure work before allocating per-root visited/active
    // bitmaps or materializing any transitive digest projections. For each of
    // N roots, DFS can visit at most N nodes and scan the entire E-edge graph;
    // include the one-time graph materialization already counted in graph_work.
    let per_root_bound = direct_edges.len().checked_add(1).ok_or_else(|| {
        resource_limit(
            "$.resolver.subflow_targets",
            "resolver closure bound overflow",
        )
    })?;
    let worst_case_closure_work = graph_work.checked_mul(per_root_bound).ok_or_else(|| {
        resource_limit(
            "$.resolver.subflow_targets",
            "resolver closure bound overflow",
        )
    })?;
    if worst_case_closure_work > MAX_RESOLVER_GRAPH_WORK {
        return Err(resource_limit(
            "$.resolver.subflow_targets",
            "resolver worst-case closure work limit exceeded",
        ));
    }
    for target in &manifest.subflow_targets {
        let digest = &target.target_semantic_ir_digest;
        let direct_declared = target.direct_dependencies.as_ref().ok_or_else(|| {
            error(
                ValidationStage::D8,
                "IncompleteResolver",
                "$.resolver.subflow_targets.direct_dependencies",
                "None is not equivalent to a known empty direct dependency set",
            )
        })?;
        let transitive_declared = target.transitive_dependencies.as_ref().ok_or_else(|| {
            error(
                ValidationStage::D8,
                "IncompleteResolver",
                "$.resolver.subflow_targets.transitive_dependencies",
                "None is not equivalent to a known empty transitive closure",
            )
        })?;
        let target_index = target_indices
            .get(&digest_key(digest))
            .copied()
            .ok_or_else(|| {
                error(
                    ValidationStage::D8,
                    "MissingTarget",
                    "$.resolver.subflow_targets",
                    "fresh SubflowIrUniverse does not contain the declared target",
                )
            })?;
        let direct_fresh = direct_edges[target_index]
            .iter()
            .map(|index| target_digests[*index].clone())
            .collect::<Vec<_>>();
        if *direct_declared != direct_fresh {
            return Err(error(
                ValidationStage::D8,
                "DirectDependencyMismatch",
                "$.resolver.subflow_targets.direct_dependencies",
                "fresh IR-derived direct edge set differs from the manifest",
            ));
        }
        let export_fresh = snapshot.subflow_export_pin(digest).cloned();
        let export_declared = target
            .exported_contract
            .as_ref()
            .map(|reference| reference.digest.clone());
        if export_declared != export_fresh
            || exports.get(&digest_key(digest)).cloned() != export_fresh
        {
            return Err(error(
                ValidationStage::D8,
                "TargetExportMismatch",
                "$.resolver.subflow_targets.exported_contract",
                "target-to-export association differs from the fresh formal snapshot",
            ));
        }
        let _ = transitive_declared;
        output.push(ResolverTargetProjection {
            semantic_ir_digest: digest.clone(),
            exported_contract_digest: export_fresh,
            direct_dependencies: direct_fresh,
            transitive_dependencies: Vec::new(),
        });
    }
    output
        .sort_by(|a, b| digest_key(&a.semantic_ir_digest).cmp(&digest_key(&b.semantic_ir_digest)));

    // Derive every closure under one shared budget before comparing any
    // submitted transitive list. Keep only integer node indices until the
    // whole graph fits the budget; a rejected oversized graph therefore
    // never allocates/copies its expanded digest projection.
    let mut closure_indices = Vec::with_capacity(output.len());
    for target in &output {
        closure_indices.push(transitive_closure(
            target_indices[&digest_key(&target.semantic_ir_digest)],
            &direct_edges,
            &mut graph_work,
        )?);
    }
    for (target, mut indices) in output.iter_mut().zip(closure_indices) {
        indices.sort_unstable();
        target.transitive_dependencies = indices
            .into_iter()
            .map(|index| {
                let semantic_ir_digest = target_digests[index].clone();
                SubflowTargetIdentityProjection {
                    exported_contract_digest: snapshot
                        .subflow_export_pin(&semantic_ir_digest)
                        .cloned(),
                    semantic_ir_digest,
                }
            })
            .collect();
    }
    for target in &output {
        let declared = manifest
            .subflow_targets
            .iter()
            .find(|item| item.target_semantic_ir_digest == target.semantic_ir_digest)
            .ok_or_else(|| {
                error(
                    ValidationStage::D8,
                    "MissingTarget",
                    "$.resolver.subflow_targets",
                    "fresh resolver target has no declared target row",
                )
            })?;
        let declared_transitive = declared.transitive_dependencies.as_ref().ok_or_else(|| {
            error(
                ValidationStage::D8,
                "IncompleteResolver",
                "$.resolver.subflow_targets.transitive_dependencies",
                "None is not equivalent to a known empty transitive closure",
            )
        })?;
        let declared_transitive = declared_transitive
            .iter()
            .map(|entry| SubflowTargetIdentityProjection {
                semantic_ir_digest: entry.semantic_ir_digest.clone(),
                exported_contract_digest: entry.exported_contract_digest.clone(),
            })
            .collect::<Vec<_>>();
        if declared_transitive != target.transitive_dependencies {
            return Err(error(
                ValidationStage::D8,
                "TransitiveDependencyMismatch",
                "$.resolver.subflow_targets.transitive_dependencies",
                "fresh graph closure differs from the complete declared target/export associations",
            ));
        }
    }

    let identity = snapshot.resolver_identity();
    if identity.subflow_targets().len() != output.len() {
        return Err(error(
            ValidationStage::D8,
            "TargetSetMismatch",
            "$.resolver.subflow_targets",
            "formal snapshot target set differs from reconstructed target set",
        ));
    }
    for (actual, expected) in identity.subflow_targets().iter().zip(&output) {
        if actual.semantic_ir_digest() != &expected.semantic_ir_digest
            || actual.direct_dependencies() != Some(expected.direct_dependencies.as_slice())
            || actual.exported_contract_digest() != expected.exported_contract_digest.as_ref()
        {
            return Err(error(
                ValidationStage::D8,
                "ResolverIdentityMismatch",
                "$.resolver.subflow_targets",
                "fresh resolver identity association differs from its typed target projection",
            ));
        }
    }
    Ok(output)
}

fn transitive_closure(
    root: usize,
    direct_edges: &[Vec<usize>],
    total_work: &mut usize,
) -> Result<Vec<usize>, PortableProofError> {
    let mut visited = vec![false; direct_edges.len()];
    let mut active = vec![false; direct_edges.len()];
    let mut visited_nodes = Vec::new();
    let mut stack = vec![(root, false)];
    while let Some((node, exiting)) = stack.pop() {
        if exiting {
            active[node] = false;
            continue;
        }
        if active[node] {
            return Err(error(
                ValidationStage::D8,
                "DependencyCycle",
                "$.resolver.subflow_targets",
                "direct target dependency graph contains a cycle",
            ));
        }
        if visited[node] {
            continue;
        }
        add_graph_work(total_work, 1)?;
        visited[node] = true;
        active[node] = true;
        visited_nodes.push(node);
        stack.push((node, true));
        let direct = direct_edges.get(node).ok_or_else(|| {
            error(
                ValidationStage::D8,
                "MissingTarget",
                "$.resolver.subflow_targets",
                "target graph edge row is missing",
            )
        })?;
        for dependency in direct.iter().rev() {
            add_graph_work(total_work, 1)?;
            if active[*dependency] {
                return Err(error(
                    ValidationStage::D8,
                    "DependencyCycle",
                    "$.resolver.subflow_targets",
                    "direct target dependency graph contains a cycle",
                ));
            }
            stack.push((*dependency, false));
        }
    }
    visited_nodes.retain(|index| *index != root);
    Ok(visited_nodes)
}

fn add_graph_work(work: &mut usize, amount: usize) -> Result<(), PortableProofError> {
    *work = work.checked_add(amount).ok_or_else(|| {
        resource_limit("$.resolver.subflow_targets", "graph work counter overflow")
    })?;
    if *work > MAX_RESOLVER_GRAPH_WORK {
        return Err(resource_limit(
            "$.resolver.subflow_targets",
            "resolver closure work limit exceeded",
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn derive_source_associations(
    manifest: &ResolverManifest,
    caller_asset: &IrAsset,
    caller: &CheckedIrDocument<'_>,
    modules: &af_idl::CompiledModuleSet,
    proof_context: &StaticContractProofContext<'_>,
    snapshot: &af_computation::StaticComputationResolutionSnapshot<'_, '_, '_>,
    contracts_by_digest: &BTreeMap<DigestKey, usize>,
    contract_assets: &[ContractAsset],
    checked_contracts: &[CheckedComponentContract<'_, '_>],
) -> Result<(Vec<SourceAssociationProjection>, usize, usize), PortableProofError> {
    let mut associations = Vec::new();
    let mut await_leaf_count = 0usize;
    let mut await_unknown_external_count = 0usize;
    let mut occurrences = BTreeSet::new();
    let caller_subflow_calls = caller
        .source()
        .computations
        .iter()
        .filter(|computation| matches!(computation.body, ComputationBody::CallSubflow(_)))
        .count();
    let resolver_identity = snapshot.resolver_identity();
    let target_count = resolver_identity.subflow_targets().len();
    let direct_edge_count = resolver_identity
        .subflow_targets()
        .iter()
        .filter_map(|target| target.direct_dependencies())
        .map(<[DigestRefValue]>::len)
        .sum::<usize>();
    let per_call_work = target_count.checked_add(direct_edge_count).ok_or_else(|| {
        resource_limit(
            "$.resolver.source_associations",
            "source-resolution work counter overflow",
        )
    })?;
    let source_graph_work = caller_subflow_calls
        .checked_mul(per_call_work)
        .ok_or_else(|| {
            resource_limit(
                "$.resolver.source_associations",
                "source-resolution work counter overflow",
            )
        })?;
    if source_graph_work > MAX_RESOLVER_GRAPH_WORK {
        return Err(resource_limit(
            "$.resolver.source_associations",
            "source leaf closure work limit exceeded",
        ));
    }
    let caller_digest = caller.source().digest(modules).map_err(|failure| {
        error(
            ValidationStage::D8,
            "CallerDigestFailed",
            "$.resolver.caller_ir",
            failure.to_string(),
        )
    })?;
    let caller_digest = to_digest_ref(&caller_digest);
    for computation in &caller.source().computations {
        if !matches!(
            computation.body,
            ComputationBody::Invoke(_)
                | ComputationBody::CallSubflow(_)
                | ComputationBody::Await(_)
        ) {
            continue;
        }
        if !occurrences.insert(computation.id.raw()) {
            return Err(error(
                ValidationStage::D8,
                "DuplicateSourceOccurrence",
                "$.resolver.source_associations",
                "caller IR contains duplicate source-node occurrence",
            ));
        }
        let resolved =
            resolve_computation_leaf(caller, computation.id, snapshot).map_err(|failure| {
                error(
                    ValidationStage::D8,
                    "SourceResolutionFailed",
                    "$.resolver.source_associations",
                    failure.to_string(),
                )
            })?;
        if !std::ptr::eq(resolved.proof_context_reference().context(), proof_context) {
            return Err(error(
                ValidationStage::D8,
                "ProofContextIdentityMismatch",
                "$.resolver.source_associations",
                "resolved source occurrence is not bound to the D5 context",
            ));
        }
        let target = resolved.target().ok_or_else(|| {
            error(
                ValidationStage::D8,
                "UnresolvedSourceAssociation",
                "$.resolver.source_associations",
                "fresh leaf resolution did not produce an exact target kind and identity",
            )
        })?;
        let target = match target {
            ResolvedTarget::Component {
                contract_digest, ..
            } => {
                let index = contracts_by_digest
                    .get(&digest_key(contract_digest))
                    .ok_or_else(|| {
                        error(
                            ValidationStage::D8,
                            "MissingSourceContract",
                            "$.resolver.source_associations",
                            "INVOKE source does not resolve to a checked component asset",
                        )
                    })?;
                if !std::ptr::eq(
                    checked_contracts
                        .get(*index)
                        .ok_or_else(|| {
                            error(
                                ValidationStage::D8,
                                "MissingSourceContract",
                                "$.resolver.source_associations",
                                "checked component index is invalid",
                            )
                        })?
                        .proof_context(),
                    resolved.proof_context_reference().context(),
                ) {
                    return Err(error(
                        ValidationStage::D8,
                        "ProofContextIdentityMismatch",
                        "$.resolver.source_associations",
                        "source ComponentContract belongs to another proof context",
                    ));
                }
                SourceAssociationTargetProjection::Invoke(ContentReferenceProjection {
                    digest: contract_assets[*index].reference.digest.clone(),
                    type_schema: contract_assets[*index].reference.type_schema.clone(),
                })
            }
            ResolvedTarget::Subflow {
                semantic_ir_digest,
                exported_contract_digest,
            } => SourceAssociationTargetProjection::CallSubflow {
                semantic_ir_digest: semantic_ir_digest.clone(),
                exported_contract_digest: exported_contract_digest.clone(),
            },
            ResolvedTarget::AwaitSignal { signal_type } => {
                let summary = resolved.await_summary().ok_or_else(|| {
                    error(
                        ValidationStage::D8,
                        "AwaitSummaryMissing",
                        "$.resolver.source_associations",
                        "fresh AWAIT resolution omitted its explicit unknown-provenance summary",
                    )
                })?;
                if summary.signal_type != *signal_type
                    || summary.interaction != AwaitInteractionStatus::ContractUnavailable
                    || summary.provenance != AwaitProvenanceStatus::UnknownExternalInfluence
                    || summary.authority != AwaitAuthorityStatus::NotInferred
                {
                    return Err(error(
                        ValidationStage::D8,
                        "AwaitAuthorityEscalation",
                        "$.resolver.source_associations",
                        "AWAIT facts were not preserved as unavailable interaction, unknown external influence, and uninferred authority",
                    ));
                }
                await_leaf_count += 1;
                await_unknown_external_count += 1;
                SourceAssociationTargetProjection::AwaitSignal(signal_type.clone())
            }
        };
        associations.push(SourceAssociationProjection {
            caller_ir: caller_digest.clone(),
            source_node: computation.id.raw(),
            target,
        });
    }
    if caller_asset.reference.digest != caller_digest || manifest.caller_ir.digest != caller_digest
    {
        return Err(error(
            ValidationStage::D8,
            "CallerDigestMismatch",
            "$.resolver.caller_ir",
            "fresh source associations were derived from a different caller SemanticIrDigest",
        ));
    }
    associations.sort_by_key(source_association_key);
    Ok((associations, await_leaf_count, await_unknown_external_count))
}

fn compare_source_associations(
    declared: &[SourceAssociationManifest],
    fresh: &[SourceAssociationProjection],
    caller_digest: &DigestRefValue,
) -> Result<(), PortableProofError> {
    let mut normalized = declared
        .iter()
        .map(|item| SourceAssociationProjection {
            caller_ir: item.caller_ir.digest.clone(),
            source_node: item.source_node,
            target: match &item.target {
                SourceAssociationTarget::Invoke(reference) => {
                    SourceAssociationTargetProjection::Invoke(ContentReferenceProjection {
                        digest: reference.digest.clone(),
                        type_schema: reference.type_schema.clone(),
                    })
                }
                SourceAssociationTarget::CallSubflow(association) => {
                    SourceAssociationTargetProjection::CallSubflow {
                        semantic_ir_digest: association.target_semantic_ir_digest.clone(),
                        exported_contract_digest: association.exported_contract_digest.clone(),
                    }
                }
                SourceAssociationTarget::AwaitSignal(schema) => {
                    SourceAssociationTargetProjection::AwaitSignal(schema.clone())
                }
            },
        })
        .collect::<Vec<_>>();
    if declared
        .iter()
        .any(|item| item.caller_ir.digest != *caller_digest)
    {
        return Err(error(
            ValidationStage::D8,
            "ForeignSourceAssociation",
            "$.resolver.source_associations",
            "source association names a caller other than the resolver's exact caller IR",
        ));
    }
    let mut seen = BTreeSet::new();
    for association in &normalized {
        if !seen.insert((digest_key(&association.caller_ir), association.source_node)) {
            return Err(error(
                ValidationStage::D8,
                "DuplicateSourceAssociation",
                "$.resolver.source_associations",
                "duplicate caller/source-node occurrence",
            ));
        }
    }
    normalized.sort_by_key(source_association_key);
    if normalized != fresh {
        return Err(error(
            ValidationStage::D8,
            "SourceAssociationMismatch",
            "$.resolver.source_associations",
            "freshly resolved caller/node/target tuples differ from the declared manifest",
        ));
    }
    Ok(())
}

fn source_association_key(item: &SourceAssociationProjection) -> (DigestKey, u32, String) {
    let target = match &item.target {
        SourceAssociationTargetProjection::Invoke(reference) => {
            format!("0:{}", digest_text(&reference.digest))
        }
        SourceAssociationTargetProjection::CallSubflow {
            semantic_ir_digest,
            exported_contract_digest,
        } => format!(
            "1:{}:{}",
            digest_text(semantic_ir_digest),
            exported_contract_digest
                .as_ref()
                .map_or_else(|| "-".to_owned(), digest_text),
        ),
        SourceAssociationTargetProjection::AwaitSignal(reference) => format!(
            "2:{}/{}/{}",
            reference.specification_id, reference.type_name, reference.schema_digest.hex,
        ),
    };
    (digest_key(&item.caller_ir), item.source_node, target)
}

fn find_asset_bytes<'a>(
    root: &'a Value,
    table: &str,
    expected: &ContentReference,
) -> Result<&'a [u8], PortableProofError> {
    let mut found = None;
    for row in list_field(root, table, "$")? {
        let reference: ContentReference = from_value(field(row, "reference", table)?, table)?;
        if reference == *expected {
            if found.is_some() {
                return Err(error(
                    ValidationStage::D6,
                    "DuplicateAssetReference",
                    table,
                    "duplicate exact typed content reference",
                ));
            }
            found = Some(bytes_field(row, "canonical_content", table)?);
        }
    }
    found.ok_or_else(|| {
        error(
            ValidationStage::D6,
            "MissingAsset",
            table,
            "exact typed content reference has no inline asset",
        )
    })
}

fn schema_ref(
    modules: &af_idl::CompiledModuleSet,
    specification_id: &str,
    type_name: &str,
) -> Result<TypeSchemaRef, PortableProofError> {
    let schema = modules
        .type_by_identity(&format!("{specification_id}/{type_name}"))
        .ok_or_else(|| {
            error(
                ValidationStage::D6,
                "UnsupportedCurrentSchema",
                "$.schema_assets",
                "registered root schema is absent from D5 module set",
            )
        })?;
    let digest = schema.digest().map_err(|failure| {
        error(
            ValidationStage::D6,
            "UnsupportedCurrentSchema",
            "$.schema_assets",
            failure.to_string(),
        )
    })?;
    Ok(TypeSchemaRef {
        specification_id: specification_id.to_owned(),
        type_name: type_name.to_owned(),
        schema_digest: af_semantic::DigestRefValue {
            algorithm: "sha256".to_owned(),
            domain: "type-schema".to_owned(),
            hex: digest.hex().to_owned(),
        },
    })
}

fn valid_digest(reference: &DigestRefValue, domain: &str) -> bool {
    reference.algorithm == "sha256"
        && reference.domain == domain
        && reference.hex.len() == 64
        && reference
            .hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn digest_matches(reference: &DigestRefValue, digest: &DigestRef) -> bool {
    reference.algorithm == "sha256"
        && reference.domain == digest.domain().as_str()
        && reference.hex == digest.hex()
}

type DigestKey = (String, String, String);
type FreshContractSet<'context, 'universe> = (
    Vec<CheckedComponentContract<'context, 'universe>>,
    Vec<CheckedContractProjection>,
    BTreeMap<DigestKey, usize>,
);
type FreshIrSet<'ir> = (
    Vec<CheckedIrDocument<'ir>>,
    Vec<SemanticDataflowAnalysis>,
    Vec<CheckedIrProjection>,
    BTreeMap<DigestKey, usize>,
);

fn digest_key(reference: &DigestRefValue) -> DigestKey {
    (
        reference.algorithm.clone(),
        reference.domain.clone(),
        reference.hex.clone(),
    )
}

fn digest_text(reference: &DigestRefValue) -> String {
    format!(
        "{}/{}/{}",
        reference.algorithm, reference.domain, reference.hex
    )
}

fn to_digest_ref(digest: &DigestRef) -> DigestRefValue {
    DigestRefValue {
        algorithm: "sha256".to_owned(),
        domain: digest.domain().as_str().to_owned(),
        hex: digest.hex().to_owned(),
    }
}

const MAX_RECONSTRUCTED_ITEMS: usize = 100_000;

fn from_value<T: for<'de> Deserialize<'de>>(
    value: &Value,
    path: &str,
) -> Result<T, PortableProofError> {
    let json = value_to_json(value)
        .map_err(|detail| error(ValidationStage::D6, "InvalidTypedContent", path, detail))?;
    serde_json::from_value(json).map_err(|failure| {
        error(
            ValidationStage::D6,
            "InvalidTypedContent",
            path,
            failure.to_string(),
        )
    })
}

fn field<'a>(value: &'a Value, name: &str, path: &str) -> Result<&'a Value, PortableProofError> {
    let Value::Record(fields) = value else {
        return Err(error(
            ValidationStage::D6,
            "InvalidTypedContent",
            path,
            "expected record",
        ));
    };
    fields
        .iter()
        .find(|(field, _)| field == name)
        .map(|(_, value)| value)
        .ok_or_else(|| {
            error(
                ValidationStage::D6,
                "InvalidTypedContent",
                path,
                format!("missing field {name}"),
            )
        })
}

fn list_field<'a>(
    value: &'a Value,
    name: &str,
    path: &str,
) -> Result<&'a [Value], PortableProofError> {
    match field(value, name, path)? {
        Value::List(items) => Ok(items),
        _ => Err(error(
            ValidationStage::D6,
            "InvalidTypedContent",
            path,
            format!("{name} is not a list"),
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
        _ => Err(error(
            ValidationStage::D6,
            "InvalidTypedContent",
            path,
            format!("{name} is not bytes"),
        )),
    }
}

fn u32_field(value: &Value, name: &str) -> Result<u32, PortableProofError> {
    match field(value, name, "$")? {
        Value::UInt32(number) => Ok(*number),
        _ => Err(error(
            ValidationStage::D8,
            "InvalidIdentifier",
            "$",
            format!("{name} is not uint32"),
        )),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(sha2::Sha256::digest(bytes))
}

fn error(
    stage: ValidationStage,
    family: &'static str,
    path: impl Into<String>,
    detail: impl Into<String>,
) -> PortableProofError {
    PortableProofError::new(stage, family, path, detail)
}

fn resource_limit(path: impl Into<String>, detail: impl Into<String>) -> PortableProofError {
    error(ValidationStage::D8, "ResourceLimit", path, detail)
}
