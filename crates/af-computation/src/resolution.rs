//! Immutable resolution snapshots and leaf-specific analysis.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use af_contract::{
    ActionGuardCompatibility, CheckedComponentContract, ComponentContract,
    StaticContractProofContext, component_contract_digest,
};
use af_effect::{EffectRow, EffectSignature};
use af_flow::{FlowSink, SourceCategory};
use af_ir::{
    AwaitBody, CheckedIrDocument, ComputationBody, ComputationId, ContractRefKind, InvokeBody,
    SemanticIrDocument, SubflowIrUniverse,
};
use af_semantic::{DigestRefValue, ResourceSlotId, SemanticTypeNodeBody, ValueSlotId};

use crate::mapping::{expression_schema_ref, interface_schema_match};
use crate::proof::{LeafDimensionProof, LeafProofReport, LeafVerdict};
use crate::summary::{
    AwaitLeafSummary, ComponentDimensionSummary, FlowPortMapping, FlowSinkBinding,
    FlowSourceBinding, LeafKind, ResolvedComputationLeaf, ResolvedTarget, ResourceAliasGroup,
    ScopedEffectOccurrence, ScopedEffectResidual, ScopedPortBinding, ScopedPortMapping,
    ScopedResourcePortMapping,
};

const RULE_TARGET_PIN: &str = "AF-SEMANTIC-0111";
const RULE_INVOKE_CONTRACT: &str = "AF-SEMANTIC-0112";
const RULE_PUBLIC_TYPE: &str = "AF-SEMANTIC-0113";
const RULE_RESOURCE_MAPPING: &str = "AF-SEMANTIC-0114";
const RULE_EFFECT_SCOPE: &str = "AF-SEMANTIC-0115";
const RULE_FLOW_SCOPE: &str = "AF-SEMANTIC-0116";
const RULE_DIMENSIONS: &str = "AF-SEMANTIC-0117";
const RULE_SUBFLOW_TARGET: &str = "AF-SEMANTIC-0118";
const RULE_SUBFLOW_EXPORT: &str = "AF-SEMANTIC-0119";
const RULE_SUBFLOW_BODY: &str = "AF-SEMANTIC-0120";
const RULE_SUBFLOW_CYCLE: &str = "AF-SEMANTIC-0121";
const RULE_AWAIT: &str = "AF-SEMANTIC-0122";
const RULE_PROOF_CONTEXT: &str = "AF-SEMANTIC-0123";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotError {
    ProofContextMismatch,
    ContractDigest(String),
    DuplicateComponentPin(DigestRefValue),
    DuplicateSubflowExport(DigestRefValue),
    UnknownSubflowTarget(DigestRefValue),
    InvalidSubflowDigest(DigestRefValue),
}

impl std::fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProofContextMismatch => {
                f.write_str("checked contract was proven under a different proof-context object")
            }
            Self::ContractDigest(detail) => write!(
                f,
                "component contract digest could not be computed: {detail}"
            ),
            Self::DuplicateComponentPin(digest) => write!(
                f,
                "duplicate exact component contract pin {}/{}",
                digest.domain, digest.hex
            ),
            Self::DuplicateSubflowExport(digest) => write!(
                f,
                "duplicate exported contract for SemanticIrDigest {}",
                digest.hex
            ),
            Self::UnknownSubflowTarget(digest) => write!(
                f,
                "export association names unresolved SemanticIrDigest {}",
                digest.hex
            ),
            Self::InvalidSubflowDigest(digest) => write!(
                f,
                "invalid subflow target digest {}/{}",
                digest.domain, digest.hex
            ),
        }
    }
}

impl std::error::Error for SnapshotError {}

/// Mutable assembly phase for an explicit resolution snapshot. `build`
/// consumes it; the resulting snapshot has no mutation API.
pub struct StaticComputationResolutionSnapshotBuilder<'a, 'context, 'universe> {
    proof_context: &'context StaticContractProofContext<'universe>,
    subflows: &'a SubflowIrUniverse,
    components: HashMap<DigestRefValue, &'a CheckedComponentContract<'context, 'universe>>,
    exports: HashMap<DigestRefValue, ExportedContract<'a, 'context, 'universe>>,
}

struct ExportedContract<'a, 'context, 'universe> {
    checked: &'a CheckedComponentContract<'context, 'universe>,
    digest: DigestRefValue,
}

pub struct StaticComputationResolutionSnapshot<'a, 'context, 'universe> {
    proof_context: &'context StaticContractProofContext<'universe>,
    subflows: &'a SubflowIrUniverse,
    components: HashMap<DigestRefValue, &'a CheckedComponentContract<'context, 'universe>>,
    exports: HashMap<DigestRefValue, ExportedContract<'a, 'context, 'universe>>,
}

/// Content identity of an immutable resolver snapshot. This process-local
/// value is generated from the snapshot itself and is not a wire or digest
/// identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolverSnapshotIdentity {
    component_contract_digests: Vec<DigestRefValue>,
    subflow_targets: Vec<ResolverSubflowIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolverSubflowIdentity {
    semantic_ir_digest: DigestRefValue,
    direct_dependencies: Option<Vec<DigestRefValue>>,
    exported_contract_digest: Option<DigestRefValue>,
}

impl ResolverSnapshotIdentity {
    #[must_use]
    pub fn component_contract_digests(&self) -> &[DigestRefValue] {
        &self.component_contract_digests
    }

    #[must_use]
    pub fn subflow_targets(&self) -> &[ResolverSubflowIdentity] {
        &self.subflow_targets
    }
}

impl ResolverSubflowIdentity {
    #[must_use]
    pub fn semantic_ir_digest(&self) -> &DigestRefValue {
        &self.semantic_ir_digest
    }

    #[must_use]
    pub fn direct_dependencies(&self) -> Option<&[DigestRefValue]> {
        self.direct_dependencies.as_deref()
    }

    #[must_use]
    pub fn exported_contract_digest(&self) -> Option<&DigestRefValue> {
        self.exported_contract_digest.as_ref()
    }
}

impl StaticComputationResolutionSnapshot<'_, '_, '_> {
    /// Whether this immutable resolver snapshot uses the exact proof-context
    /// object supplied to the analysis (not merely equal catalog contents).
    #[must_use]
    pub fn is_same_context(&self, context: &StaticContractProofContext<'_>) -> bool {
        std::ptr::eq(self.proof_context, context)
    }

    /// Exact checked component pins retained by this immutable resolver.
    #[must_use]
    pub fn component_contract_pins(&self) -> Vec<DigestRefValue> {
        let mut pins = self.components.keys().cloned().collect::<Vec<_>>();
        pins.sort_by(|left, right| digest_key(left).cmp(&digest_key(right)));
        pins
    }

    /// Exact target SemanticIrDigest pins retained by this immutable resolver.
    #[must_use]
    pub fn subflow_target_pins(&self) -> Vec<DigestRefValue> {
        self.subflows.digests().into_iter().cloned().collect()
    }

    /// Exact exported ComponentContractDigest associated with one target, if any.
    #[must_use]
    pub fn subflow_export_pin(&self, target: &DigestRefValue) -> Option<&DigestRefValue> {
        self.exports.get(target).map(|entry| &entry.digest)
    }

    /// Exact direct dependencies recorded for one validated target.
    #[must_use]
    pub fn subflow_dependencies(&self, target: &DigestRefValue) -> Option<&[DigestRefValue]> {
        self.subflows
            .target(target)
            .map(af_ir::SubflowTarget::dependencies)
    }

    /// Returns the exact content identity of this immutable snapshot,
    /// including component pins and every target/dependency/export
    /// association. Callers cannot construct or attach an identity to a
    /// different snapshot.
    #[must_use]
    pub fn resolver_identity(&self) -> ResolverSnapshotIdentity {
        let mut subflow_targets = self
            .subflow_target_pins()
            .into_iter()
            .map(|semantic_ir_digest| ResolverSubflowIdentity {
                direct_dependencies: self
                    .subflow_dependencies(&semantic_ir_digest)
                    .map(<[DigestRefValue]>::to_vec),
                exported_contract_digest: self.subflow_export_pin(&semantic_ir_digest).cloned(),
                semantic_ir_digest,
            })
            .collect::<Vec<_>>();
        subflow_targets.sort_by(|left, right| {
            digest_key(&left.semantic_ir_digest).cmp(&digest_key(&right.semantic_ir_digest))
        });
        ResolverSnapshotIdentity {
            component_contract_digests: self.component_contract_pins(),
            subflow_targets,
        }
    }
}

fn digest_key(value: &DigestRefValue) -> (&str, &str, &str) {
    (&value.algorithm, &value.domain, &value.hex)
}

impl<'a, 'context, 'universe> StaticComputationResolutionSnapshotBuilder<'a, 'context, 'universe> {
    #[must_use]
    pub fn new(
        proof_context: &'context StaticContractProofContext<'universe>,
        subflows: &'a SubflowIrUniverse,
    ) -> Self {
        Self {
            proof_context,
            subflows,
            components: HashMap::new(),
            exports: HashMap::new(),
        }
    }

    pub fn component(
        mut self,
        checked: &'a CheckedComponentContract<'context, 'universe>,
    ) -> Result<Self, SnapshotError> {
        let digest = checked_digest(checked, self.proof_context)?;
        if self.components.insert(digest.clone(), checked).is_some() {
            return Err(SnapshotError::DuplicateComponentPin(digest));
        }
        Ok(self)
    }

    /// Associates one complete checked exported contract with one exact
    /// target SemanticIrDigest. The association is explicit snapshot input;
    /// Phase 3D derives only I/O/R and cannot manufacture the other rows.
    pub fn subflow_export(
        mut self,
        target_digest: &DigestRefValue,
        checked: &'a CheckedComponentContract<'context, 'universe>,
    ) -> Result<Self, SnapshotError> {
        if target_digest.algorithm != "sha256" || target_digest.domain != "semantic-ir" {
            return Err(SnapshotError::InvalidSubflowDigest(target_digest.clone()));
        }
        if self.subflows.target(target_digest).is_none() {
            return Err(SnapshotError::UnknownSubflowTarget(target_digest.clone()));
        }
        let contract_digest = checked_digest(checked, self.proof_context)?;
        if self
            .exports
            .insert(
                target_digest.clone(),
                ExportedContract {
                    checked,
                    digest: contract_digest,
                },
            )
            .is_some()
        {
            return Err(SnapshotError::DuplicateSubflowExport(target_digest.clone()));
        }
        Ok(self)
    }

    #[must_use]
    pub fn build(self) -> StaticComputationResolutionSnapshot<'a, 'context, 'universe> {
        StaticComputationResolutionSnapshot {
            proof_context: self.proof_context,
            subflows: self.subflows,
            components: self.components,
            exports: self.exports,
        }
    }
}

fn checked_digest<'universe>(
    checked: &CheckedComponentContract<'_, 'universe>,
    context: &StaticContractProofContext<'universe>,
) -> Result<DigestRefValue, SnapshotError> {
    if !std::ptr::eq(checked.proof_context(), context) || !checked.proof().is_proven() {
        return Err(SnapshotError::ProofContextMismatch);
    }
    let digest = component_contract_digest(checked, context)
        .map_err(|error| SnapshotError::ContractDigest(error.to_string()))?;
    Ok(DigestRefValue {
        algorithm: "sha256".into(),
        domain: digest.domain().as_str().into(),
        hex: digest.hex().to_string(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeafResolutionError {
    UnknownComputation(ComputationId),
    NotAComputationLeaf(ComputationId),
    SemanticIrDigest(String),
}

impl std::fmt::Display for LeafResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownComputation(id) => write!(f, "unknown computation {id}"),
            Self::NotAComputationLeaf(id) => {
                write!(f, "computation {id} is not INVOKE, AWAIT, or CALL_SUBFLOW")
            }
            Self::SemanticIrDigest(detail) => {
                write!(f, "cannot compute caller SemanticIrDigest: {detail}")
            }
        }
    }
}

impl std::error::Error for LeafResolutionError {}

/// Resolves exactly one leaf. Dataflow and Phase 3D checks are recomputed
/// from the checked source and explicit snapshot; callers cannot inject a
/// claimed compatibility boolean or construct the returned checked summary.
pub fn resolve_computation_leaf<'context, 'universe>(
    ir: &CheckedIrDocument<'_>,
    computation_id: ComputationId,
    snapshot: &StaticComputationResolutionSnapshot<'_, 'context, 'universe>,
) -> Result<ResolvedComputationLeaf<'context, 'universe>, LeafResolutionError> {
    let document = ir.source();
    let Some(computation) = document
        .computations
        .iter()
        .find(|item| item.id == computation_id)
    else {
        return Err(LeafResolutionError::UnknownComputation(computation_id));
    };
    let kind = match &computation.body {
        ComputationBody::Invoke(_) => LeafKind::Invoke,
        ComputationBody::Await(_) => LeafKind::Await,
        ComputationBody::CallSubflow(_) => LeafKind::CallSubflow,
        _ => return Err(LeafResolutionError::NotAComputationLeaf(computation_id)),
    };
    let semantic_digest = document
        .digest(snapshot.proof_context.modules)
        .map_err(|error| LeafResolutionError::SemanticIrDigest(error.to_string()))?;
    let caller_digest = DigestRefValue {
        algorithm: "sha256".into(),
        domain: semantic_digest.domain().as_str().into(),
        hex: semantic_digest.hex().to_string(),
    };
    let mut base = EmptyLeaf::new(computation_id, kind, caller_digest);
    base.push(LeafDimensionProof::new(
        "proof-context-consistency",
        LeafVerdict::Proven,
        RULE_PROOF_CONTEXT,
        None,
        "the snapshot and every inserted checked contract use the same exact immutable proof-context object",
    ));

    let dataflow = match ir.analyze_dataflow() {
        Ok(value) => value,
        Err(error) => {
            base.push(LeafDimensionProof::new(
                "scoped-dataflow",
                LeafVerdict::Violation,
                RULE_PUBLIC_TYPE,
                Some(format!("computation:{}", computation_id.raw())),
                format!("Phase 3C scoped dataflow rejected the checked IR use site: {error}"),
            ));
            return Ok(base.finish(snapshot.proof_context));
        }
    };
    base.push(LeafDimensionProof::new(
        "scoped-dataflow",
        LeafVerdict::Proven,
        RULE_PUBLIC_TYPE,
        Some(format!("computation:{}", computation_id.raw())),
        "Phase 3C validated expression availability, producer dominance, branch isolation, and lexical resource visibility",
    ));

    match &computation.body {
        ComputationBody::Invoke(body) => Ok(resolve_invoke(
            ir,
            document,
            computation_id,
            body,
            snapshot,
            &dataflow,
            base,
        )),
        ComputationBody::CallSubflow(body) => Ok(resolve_subflow(
            ir,
            document,
            computation_id,
            body,
            snapshot,
            &dataflow,
            base,
        )),
        ComputationBody::Await(body) => Ok(resolve_await(
            document,
            computation_id,
            body,
            snapshot,
            base,
        )),
        _ => unreachable!("leaf kind checked above"),
    }
}

struct EmptyLeaf {
    computation: ComputationId,
    caller_semantic_ir_digest: DigestRefValue,
    kind: LeafKind,
    target: Option<ResolvedTarget>,
    inputs: Vec<ScopedPortMapping>,
    outputs: Vec<ScopedPortMapping>,
    resources: Vec<ScopedResourcePortMapping>,
    aliases: Vec<ResourceAliasGroup>,
    effects: Vec<ScopedEffectOccurrence>,
    effect_residuals: Vec<ScopedEffectResidual>,
    flows: Vec<FlowPortMapping>,
    dimensions: Option<ComponentDimensionSummary>,
    action_guard_compatibility: Vec<ActionGuardCompatibility>,
    await_summary: Option<AwaitLeafSummary>,
    proofs: Vec<LeafDimensionProof>,
}

impl EmptyLeaf {
    fn new(
        computation: ComputationId,
        kind: LeafKind,
        caller_semantic_ir_digest: DigestRefValue,
    ) -> Self {
        Self {
            computation,
            caller_semantic_ir_digest,
            kind,
            target: None,
            inputs: vec![],
            outputs: vec![],
            resources: vec![],
            aliases: vec![],
            effects: vec![],
            effect_residuals: vec![],
            flows: vec![],
            dimensions: None,
            action_guard_compatibility: vec![],
            await_summary: None,
            proofs: vec![],
        }
    }
    fn push(&mut self, proof: LeafDimensionProof) {
        self.proofs.push(proof);
    }
    fn finish<'context, 'universe>(
        self,
        context: &'context StaticContractProofContext<'universe>,
    ) -> ResolvedComputationLeaf<'context, 'universe> {
        ResolvedComputationLeaf {
            computation: self.computation,
            caller_semantic_ir_digest: self.caller_semantic_ir_digest,
            kind: self.kind,
            target: self.target,
            input_mappings: self.inputs,
            output_mappings: self.outputs,
            resource_mappings: self.resources,
            resource_aliases: self.aliases,
            effect_occurrences: self.effects,
            effect_residuals: self.effect_residuals,
            flow_mappings: self.flows,
            dimensions: self.dimensions,
            action_guard_compatibility: self.action_guard_compatibility,
            await_summary: self.await_summary,
            proof: LeafProofReport::new(self.proofs),
            proof_context: crate::summary::ProofContextReference::new(context),
        }
    }
}

fn resolve_invoke<'context, 'universe>(
    ir: &CheckedIrDocument<'_>,
    document: &SemanticIrDocument,
    computation: ComputationId,
    body: &InvokeBody,
    snapshot: &StaticComputationResolutionSnapshot<'_, 'context, 'universe>,
    dataflow: &af_ir::SemanticDataflowAnalysis,
    mut result: EmptyLeaf,
) -> ResolvedComputationLeaf<'context, 'universe> {
    let entry = document
        .contracts
        .iter()
        .find(|entry| entry.id == body.contract);
    let Some(entry) = entry else {
        result.push(unproven(
            "reference-resolution",
            RULE_TARGET_PIN,
            "the INVOKE contract reference is absent",
        ));
        add_missing_contract_dimensions(&mut result, "the INVOKE target reference is absent");
        return result.finish(snapshot.proof_context);
    };
    if entry.kind != ContractRefKind::Component {
        result.push(LeafDimensionProof::new(
            "reference-resolution",
            LeafVerdict::Violation,
            RULE_TARGET_PIN,
            Some(format!("contract-ref:{}", entry.id.raw())),
            "INVOKE does not reference a ComponentContract entry",
        ));
        add_missing_contract_dimensions(&mut result, "the referenced contract has the wrong kind");
        return result.finish(snapshot.proof_context);
    }
    let Some(checked) = snapshot.components.get(&entry.digest).copied() else {
        result.push(unproven(
            "reference-resolution",
            RULE_TARGET_PIN,
            format!(
                "no checked ComponentContract resolves exact pin sha256/{}/{}",
                entry.digest.domain, entry.digest.hex
            ),
        ));
        add_missing_contract_dimensions(&mut result, "INVOKE target contract is unresolved");
        return result.finish(snapshot.proof_context);
    };
    result.target = Some(ResolvedTarget::Component {
        identity: checked.contract().component_identity.clone(),
        contract_digest: entry.digest.clone(),
    });
    result.push(LeafDimensionProof::new("reference-resolution", LeafVerdict::Proven, RULE_TARGET_PIN, Some(format!("contract-ref:{}", entry.id.raw())), "the IR's exact component-contract digest resolves to a checked contract; identity comes from that checked body"));
    result.push(LeafDimensionProof::new("contract-pin-validation", LeafVerdict::Proven, RULE_INVOKE_CONTRACT, Some(entry.digest.hex.clone()), "snapshot insertion recomputed the contract digest and verified the exact checked proof context"));
    result.dimensions = Some(copy_dimensions(checked.contract()));
    result
        .action_guard_compatibility
        .clone_from(&checked.proof().action_guard_compatibility);
    map_component_contract(
        document,
        computation,
        checked.contract(),
        &entry.digest,
        &body
            .inputs
            .iter()
            .map(|v| {
                (
                    v.port_no.raw(),
                    ScopedPortBinding::InputExpression(v.expression),
                )
            })
            .collect::<Vec<_>>(),
        &ir.source()
            .computations
            .iter()
            .find(|item| item.id == computation)
            .map(|item| {
                item.outputs
                    .iter()
                    .map(|v| (v.port_no.raw(), v.value_slot))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
        &body
            .resources
            .iter()
            .map(|v| (v.port_no.raw(), v.resource_slot))
            .collect::<Vec<_>>(),
        dataflow,
        &checked.contract().component_identity,
        &mut result,
    );
    result.finish(snapshot.proof_context)
}

fn resolve_subflow<'context, 'universe>(
    ir: &CheckedIrDocument<'_>,
    document: &SemanticIrDocument,
    computation: ComputationId,
    body: &af_ir::CallSubflowBody,
    snapshot: &StaticComputationResolutionSnapshot<'_, 'context, 'universe>,
    dataflow: &af_ir::SemanticDataflowAnalysis,
    mut result: EmptyLeaf,
) -> ResolvedComputationLeaf<'context, 'universe> {
    let Some(entry) = document
        .contracts
        .iter()
        .find(|entry| entry.id == body.contract)
    else {
        result.push(unproven(
            "reference-resolution",
            RULE_SUBFLOW_TARGET,
            "CALL_SUBFLOW contract reference is absent",
        ));
        return result.finish(snapshot.proof_context);
    };
    if entry.kind != ContractRefKind::Subflow {
        result.push(LeafDimensionProof::new(
            "reference-resolution",
            LeafVerdict::Violation,
            RULE_SUBFLOW_TARGET,
            Some(format!("contract-ref:{}", entry.id.raw())),
            "CALL_SUBFLOW does not reference a subflow SemanticIrDigest entry",
        ));
        return result.finish(snapshot.proof_context);
    }
    let Some(target) = snapshot.subflows.target(&entry.digest) else {
        result.push(unproven(
            "reference-resolution",
            RULE_SUBFLOW_TARGET,
            format!(
                "exact target SemanticIrDigest {} is absent from the explicit SubflowIrUniverse",
                entry.digest.hex
            ),
        ));
        result.push(unproven(
            "subflow-interface",
            RULE_SUBFLOW_TARGET,
            "target interface cannot be checked without the exact target",
        ));
        result.push(unproven(
            "target-pin-validation",
            RULE_SUBFLOW_TARGET,
            "the exact semantic-ir digest is not present in the target universe",
        ));
        result.push(unproven(
            "subflow-export-contract",
            RULE_SUBFLOW_EXPORT,
            "no target contract can be associated without resolving the exact target",
        ));
        result.push(unproven(
            "subflow-body-conformance",
            RULE_SUBFLOW_BODY,
            "target body is unavailable",
        ));
        return result.finish(snapshot.proof_context);
    };
    result.push(LeafDimensionProof::new(
        "reference-resolution",
        LeafVerdict::Proven,
        RULE_SUBFLOW_TARGET,
        Some(entry.digest.hex.clone()),
        "the IR digest pin resolves exactly in the Phase 3D SubflowIrUniverse",
    ));
    result.push(LeafDimensionProof::new(
        "target-pin-validation",
        LeafVerdict::Proven,
        RULE_SUBFLOW_TARGET,
        Some(entry.digest.hex.clone()),
        "the exact sha256/semantic-ir DigestRef selects this target; no version or latest substitution occurs",
    ));
    result.target = Some(ResolvedTarget::Subflow {
        semantic_ir_digest: entry.digest.clone(),
        exported_contract_digest: snapshot
            .exports
            .get(&entry.digest)
            .map(|export| export.digest.clone()),
    });
    match ir.check_subflow_interface_at(computation, dataflow, snapshot.subflows) {
        Ok(()) => result.push(LeafDimensionProof::new("subflow-interface", LeafVerdict::Proven, RULE_SUBFLOW_TARGET, Some(format!("computation:{}", computation.raw())), "Phase 3D exact target interface checks the caller's input, output, and resource port sets and structural types")),
        Err(error) => result.push(LeafDimensionProof::new("subflow-interface", LeafVerdict::Violation, RULE_SUBFLOW_TARGET, Some(format!("computation:{}", computation.raw())), format!("Phase 3D rejected the exact target interface: {error}"))),
    }
    let (has_cycle, missing_closure) = subflow_closure(snapshot.subflows, &entry.digest);
    if has_cycle {
        result.push(LeafDimensionProof::new(
            "subflow-dependency-closure",
            LeafVerdict::Violation,
            RULE_SUBFLOW_CYCLE,
            Some(entry.digest.hex.clone()),
            "the explicit exact-digest dependency graph contains a direct or indirect cycle",
        ));
    } else if missing_closure {
        result.push(unproven("subflow-dependency-closure", RULE_SUBFLOW_CYCLE, "the explicit target closure contains a dependency digest absent from the snapshot universe"));
    } else {
        result.push(LeafDimensionProof::new(
            "subflow-dependency-closure",
            LeafVerdict::Proven,
            RULE_SUBFLOW_CYCLE,
            Some(entry.digest.hex.clone()),
            "the explicit target closure is complete and acyclic",
        ));
    }
    let Some(export) = snapshot.exports.get(&entry.digest) else {
        result.push(unproven("subflow-export-contract", RULE_SUBFLOW_EXPORT, "the exact target has only derived I/O/R; no full checked exported ComponentContract was supplied"));
        result.push(LeafDimensionProof::new(
            "subflow-body-conformance",
            LeafVerdict::RequiresEvidence,
            RULE_SUBFLOW_BODY,
            Some(entry.digest.hex.clone()),
            "an exported declaration is not evidence that the subflow body satisfies it",
        ));
        return result.finish(snapshot.proof_context);
    };
    let contract = export.checked.contract();
    let interface_types = (
        contract
            .inputs
            .iter()
            .map(|port| (port.port_no, port.type_ref.clone()))
            .collect::<Vec<_>>(),
        contract
            .outputs
            .iter()
            .map(|port| (port.port_no, port.type_ref.clone()))
            .collect::<Vec<_>>(),
        contract
            .resources
            .requirements
            .iter()
            .map(|port| (port.port_no, port.resource_type.schema.clone()))
            .collect::<Vec<_>>(),
    );
    match interface_schema_match(
        target.interface(), &interface_types.0, &interface_types.1, &interface_types.2,
    ) {
        Ok(()) => result.push(LeafDimensionProof::new("subflow-export-contract", LeafVerdict::Proven, RULE_SUBFLOW_EXPORT, Some(export.digest.hex.clone()), "the full checked exported contract has exact port sets and exact public TypeSchemaRef matches against the derived interface")),
        Err((verdict, reason)) => result.push(LeafDimensionProof::new("subflow-export-contract", verdict, RULE_SUBFLOW_EXPORT, Some(export.digest.hex.clone()), reason)),
    }
    result.dimensions = Some(copy_dimensions(contract));
    result
        .action_guard_compatibility
        .clone_from(&export.checked.proof().action_guard_compatibility);
    map_component_contract(
        document,
        computation,
        contract,
        &export.digest,
        &body
            .inputs
            .iter()
            .map(|v| {
                (
                    v.port_no.raw(),
                    ScopedPortBinding::InputExpression(v.expression),
                )
            })
            .collect::<Vec<_>>(),
        &document
            .computations
            .iter()
            .find(|item| item.id == computation)
            .map(|item| {
                item.outputs
                    .iter()
                    .map(|v| (v.port_no.raw(), v.value_slot))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
        &body
            .resources
            .iter()
            .map(|v| (v.port_no.raw(), v.resource_slot))
            .collect::<Vec<_>>(),
        dataflow,
        &contract.component_identity,
        &mut result,
    );
    result.push(LeafDimensionProof::new("subflow-body-conformance", LeafVerdict::RequiresEvidence, RULE_SUBFLOW_BODY, Some(entry.digest.hex.clone()), "Phase 4B.2A resolves the declaration and call boundary only; no trusted complete body-conformance proof is available"));
    result.finish(snapshot.proof_context)
}

fn resolve_await<'context, 'universe>(
    document: &SemanticIrDocument,
    computation: ComputationId,
    body: &AwaitBody,
    snapshot: &StaticComputationResolutionSnapshot<'_, 'context, 'universe>,
    mut result: EmptyLeaf,
) -> ResolvedComputationLeaf<'context, 'universe> {
    result.target = Some(ResolvedTarget::AwaitSignal {
        signal_type: body.signal_type.clone(),
    });
    let signal_valid = snapshot
        .proof_context
        .schema_universe
        .resolve(&body.signal_type)
        .is_ok();
    result.push(LeafDimensionProof::new(
        "await-signal-pin",
        if signal_valid {
            LeafVerdict::Proven
        } else {
            LeafVerdict::Violation
        },
        RULE_AWAIT,
        Some(body.signal_type.to_string()),
        if signal_valid {
            "the exact signal TypeSchemaRef resolves in the explicit schema universe"
        } else {
            "the exact signal TypeSchemaRef does not resolve in the snapshot schema universe"
        },
    ));
    let outputs = document
        .computations
        .iter()
        .find(|item| item.id == computation)
        .map(|item| {
            item.outputs
                .iter()
                .map(|binding| (binding.port_no.raw(), binding.value_slot))
                .collect()
        })
        .unwrap_or_default();
    result.await_summary = Some(AwaitLeafSummary {
        signal_type: body.signal_type.clone(),
        correlation: body.correlation,
        output_bindings: outputs,
        timeout: crate::summary::AwaitTimeoutSemantics::NoImplicitTimeout,
        interaction: crate::summary::AwaitInteractionStatus::ContractUnavailable,
        provenance: crate::summary::AwaitProvenanceStatus::UnknownExternalInfluence,
        authority: crate::summary::AwaitAuthorityStatus::NotInferred,
    });
    result.push(unproven("await-interaction-contract", RULE_AWAIT, "AWAIT provides no interaction-adapter contract; failure, flow, usage, and completion-result obligations cannot default to empty"));
    result.push(LeafDimensionProof::new("await-control-semantics", LeafVerdict::Proven, RULE_AWAIT, Some(format!("computation:{}", computation.raw())), "AWAIT is summarized as passive waiting with no implicit timeout, business Effect dispatch, or business-failure inference; external provenance remains unknown unless separately supplied"));
    result.finish(snapshot.proof_context)
}

fn add_missing_contract_dimensions(result: &mut EmptyLeaf, reason: &str) {
    for dimension in [
        "contract-pin-validation",
        "input-binding",
        "output-binding",
        "resource-mapping",
        "effect-and-recovery",
        "information-flow",
        "failure-usage-execution",
    ] {
        result.push(unproven(dimension, RULE_INVOKE_CONTRACT, reason));
    }
}

fn unproven(
    dimension: &'static str,
    rule: &'static str,
    reason: impl Into<String>,
) -> LeafDimensionProof {
    LeafDimensionProof::new(dimension, LeafVerdict::Unproven, rule, None, reason)
}

fn copy_dimensions(contract: &ComponentContract) -> ComponentDimensionSummary {
    ComponentDimensionSummary {
        failures: contract.failures.clone(),
        effects: contract.effects.clone(),
        resources: contract.resources.clone(),
        information_flow: contract.information_flow.clone(),
        usage: contract.usage.clone(),
        recovery: contract.recovery.clone(),
        execution: contract.execution.clone(),
        effect_flow_partitions: contract.effect_flow_partitions.clone(),
        recovery_action_bindings: contract.recovery_action_bindings.clone(),
        recovery_usage_bindings: contract.recovery_usage_bindings.clone(),
    }
}

#[allow(clippy::too_many_arguments)]
fn map_component_contract(
    document: &SemanticIrDocument,
    computation: ComputationId,
    contract: &ComponentContract,
    contract_digest: &DigestRefValue,
    input_bindings: &[(u32, ScopedPortBinding)],
    output_bindings: &[(u32, ValueSlotId)],
    resource_bindings: &[(u32, ResourceSlotId)],
    dataflow: &af_ir::SemanticDataflowAnalysis,
    identity: &af_contract::ComponentIdentity,
    result: &mut EmptyLeaf,
) {
    let input_expected: Vec<u32> = contract.inputs.iter().map(|port| port.port_no).collect();
    let input_actual: Vec<u32> = input_bindings.iter().map(|(port, _)| *port).collect();
    let mut input_verdict = port_set_verdict(&input_expected, &input_actual);
    for port in &contract.inputs {
        let Some((_, binding)) = input_bindings
            .iter()
            .find(|(number, _)| *number == port.port_no)
        else {
            continue;
        };
        let ScopedPortBinding::InputExpression(expression) = binding else {
            continue;
        };
        let exact_type = expression_schema_ref(document, *expression).cloned();
        match exact_type.as_ref() {
            Some(actual) if actual == &port.type_ref => {}
            Some(_) => input_verdict = merge_verdict(input_verdict, LeafVerdict::Violation),
            None => input_verdict = merge_verdict(input_verdict, LeafVerdict::Unproven),
        }
        result.inputs.push(ScopedPortMapping {
            contract_port_no: port.port_no,
            binding: binding.clone(),
            exact_type,
        });
    }
    result.push(LeafDimensionProof::new(
        "input-binding", input_verdict, RULE_PUBLIC_TYPE, Some(format!("computation:{}", computation.raw())),
        "required component ports are matched by exact port number; public types are proven only by an identical pinned TypeSchemaRef; Phase 3C proves scope and dominance",
    ));

    let output_expected: Vec<u32> = contract.outputs.iter().map(|port| port.port_no).collect();
    let output_actual: Vec<u32> = output_bindings.iter().map(|(port, _)| *port).collect();
    let mut output_verdict = port_set_verdict(&output_expected, &output_actual);
    for port in &contract.outputs {
        let Some((_, slot_id)) = output_bindings
            .iter()
            .find(|(number, _)| *number == port.port_no)
        else {
            continue;
        };
        let slot = document.values.iter().find(|value| value.id == *slot_id);
        let exact_type = slot.and_then(|slot| {
            match document
                .types
                .iter()
                .find(|node| node.id == slot.type_id)
                .map(|node| &node.body)
            {
                Some(SemanticTypeNodeBody::Schema(reference)) => Some(reference.clone()),
                _ => None,
            }
        });
        match exact_type.as_ref() {
            Some(actual) if actual == &port.type_ref => {}
            Some(_) => output_verdict = merge_verdict(output_verdict, LeafVerdict::Violation),
            None => output_verdict = merge_verdict(output_verdict, LeafVerdict::Unproven),
        }
        let own_producer = dataflow.producer_locations().get(&slot_id.raw())
            .is_some_and(|location| matches!(location, af_ir::ProducerLocation::ComputationOutput { computation: owner, port_no } if *owner == computation.raw() && *port_no == port.port_no));
        if !own_producer {
            output_verdict = merge_verdict(output_verdict, LeafVerdict::Violation);
        }
        result.outputs.push(ScopedPortMapping {
            contract_port_no: port.port_no,
            binding: ScopedPortBinding::OutputValue(*slot_id),
            exact_type,
        });
    }
    result.push(LeafDimensionProof::new(
        "output-binding", output_verdict, RULE_PUBLIC_TYPE, Some(format!("computation:{}", computation.raw())),
        "component output ports map to this leaf's declared output slots, with exact pinned public types and the existing single-producer dataflow proof",
    ));
    result.push(LeafDimensionProof::new(
        "type-compatibility",
        merge_verdict(input_verdict, output_verdict),
        RULE_PUBLIC_TYPE,
        Some(format!("computation:{}", computation.raw())),
        "all mapped Component I/O types use exact public TypeSchemaRef pins; any local-only or mismatched type remains non-positive",
    ));

    let requirements = &contract.resources.requirements;
    let resource_expected: Vec<u32> = requirements.iter().map(|item| item.port_no).collect();
    let resource_actual: Vec<u32> = resource_bindings.iter().map(|(port, _)| *port).collect();
    let mut resource_verdict = port_set_verdict(&resource_expected, &resource_actual);
    let visible = dataflow
        .entry_resources()
        .get(&computation.raw())
        .cloned()
        .unwrap_or_default();
    for requirement in requirements {
        let Some((_, slot_id)) = resource_bindings
            .iter()
            .find(|(number, _)| *number == requirement.port_no)
        else {
            continue;
        };
        let slot = document.resources.iter().find(|slot| slot.id == *slot_id);
        let exact_type =
            slot.is_some_and(|slot| slot.requirement_type == requirement.resource_type.schema);
        if !visible.contains(&slot_id.raw()) {
            resource_verdict = merge_verdict(resource_verdict, LeafVerdict::Violation);
        }
        if !exact_type {
            resource_verdict = merge_verdict(resource_verdict, LeafVerdict::Violation);
        }
        if !requirement.required_attributes.is_empty() {
            resource_verdict = merge_verdict(resource_verdict, LeafVerdict::Unproven);
        }
        result.resources.push(ScopedResourcePortMapping {
            component_port_no: requirement.port_no,
            resource_slot: *slot_id,
            required_type: requirement.resource_type.schema.clone(),
            required_attributes_proven: requirement.required_attributes.is_empty(),
        });
    }
    result.aliases = resource_alias_groups(&result.resources);
    result.push(LeafDimensionProof::new(
        "resource-mapping", resource_verdict, RULE_RESOURCE_MAPPING, Some(format!("computation:{}", computation.raw())),
        "each resource requirement port maps explicitly to a visible ResourceSlotId and exact TypeSchemaRef; non-empty attribute requirements remain unproven because IR has no attribute evidence; aliases are recorded without shrinking bounds",
    ));

    let (effect_verdict, occurrences, residuals) = scoped_effects(
        &contract.effects,
        &result.caller_semantic_ir_digest,
        contract_digest,
        identity,
        computation,
        &result.resources,
    );
    result.effects = occurrences;
    result.effect_residuals = residuals;
    result.push(LeafDimensionProof::new(
        "effect-and-recovery", effect_verdict, RULE_EFFECT_SCOPE, Some(contract_digest.hex.clone()),
        "the complete checked effect, recovery, usage, failure, and execution declarations are copied per leaf; known atoms retain full identity and qualifiers, and occurrence bounds are not scaled",
    ));

    let (flow_verdict, mappings) = scoped_flow(
        &contract.information_flow,
        &result.inputs,
        &result.outputs,
        &result.resources,
        &result.effects,
    );
    result.flows = mappings;
    result.push(LeafDimensionProof::new(
        "information-flow", flow_verdict, RULE_FLOW_SCOPE, Some(contract_digest.hex.clone()),
        "source categories, unknown influence, taints, confidentiality restrictions, may-carry authority, flow constraints, and effect-flow partitions are preserved; only explicit port coordinates are linked to this call site",
    ));
    result.push(LeafDimensionProof::new(
        "failure-usage-execution", LeafVerdict::Proven, RULE_DIMENSIONS, Some(contract_digest.hex.clone()),
        "the checked Φ/U/K/X dimensions are carried unchanged as per-invocation declarations; no empty fallback, retry scaling, workflow placement, or admission claim is introduced",
    ));
}

fn port_set_verdict(expected: &[u32], actual: &[u32]) -> LeafVerdict {
    let expected_set: BTreeSet<u32> = expected.iter().copied().collect();
    let actual_set: BTreeSet<u32> = actual.iter().copied().collect();
    if expected.len() != expected_set.len()
        || actual.len() != actual_set.len()
        || expected_set != actual_set
    {
        LeafVerdict::Violation
    } else {
        LeafVerdict::Proven
    }
}

fn merge_verdict(left: LeafVerdict, right: LeafVerdict) -> LeafVerdict {
    match (left, right) {
        (LeafVerdict::Violation, _) | (_, LeafVerdict::Violation) => LeafVerdict::Violation,
        (LeafVerdict::RequiresEvidence, _) | (_, LeafVerdict::RequiresEvidence) => {
            LeafVerdict::RequiresEvidence
        }
        (LeafVerdict::Unproven, _) | (_, LeafVerdict::Unproven) => LeafVerdict::Unproven,
        _ => LeafVerdict::Proven,
    }
}

fn resource_alias_groups(mappings: &[ScopedResourcePortMapping]) -> Vec<ResourceAliasGroup> {
    let mut groups: BTreeMap<ResourceSlotId, Vec<u32>> = BTreeMap::new();
    for mapping in mappings {
        groups
            .entry(mapping.resource_slot)
            .or_default()
            .push(mapping.component_port_no);
    }
    groups
        .into_iter()
        .filter_map(|(resource_slot, mut component_ports)| {
            component_ports.sort_unstable();
            (component_ports.len() > 1).then_some(ResourceAliasGroup {
                resource_slot,
                component_ports,
            })
        })
        .collect()
}

fn scoped_effects(
    signature: &EffectSignature,
    caller_semantic_ir_digest: &DigestRefValue,
    digest: &DigestRefValue,
    identity: &af_contract::ComponentIdentity,
    computation: ComputationId,
    resources: &[ScopedResourcePortMapping],
) -> (
    LeafVerdict,
    Vec<ScopedEffectOccurrence>,
    Vec<ScopedEffectResidual>,
) {
    let mut verdict = LeafVerdict::Proven;
    let mut known = Vec::new();
    let mut residuals = Vec::new();
    let mut add_entry = |entry: &af_effect::EffectEntry, index: usize| {
        let port = entry.atom.resource_selector.requirement_port_no;
        let mapped = resources
            .iter()
            .find(|mapping| mapping.component_port_no == port)
            .map(|mapping| mapping.resource_slot);
        if mapped.is_none() {
            verdict = merge_verdict(verdict, LeafVerdict::Unproven);
        }
        known.push(ScopedEffectOccurrence {
            caller_semantic_ir_digest: caller_semantic_ir_digest.clone(),
            source_contract_digest: digest.clone(),
            source_component: identity.clone(),
            computation,
            source_entry_ordinal: index,
            declared: entry.clone(),
            mapped_resource_slot: mapped,
            control_families: Vec::new(),
        });
    };
    match &signature.row {
        EffectRow::Closed(entries) => {
            for (index, entry) in entries.iter().enumerate() {
                add_entry(entry, index);
            }
        }
        EffectRow::Open(open) => {
            for (index, entry) in open.known.iter().enumerate() {
                add_entry(entry, index);
            }
            for (index, residual) in open.residual.iter().enumerate() {
                let port = residual.selector.requirement_port_no;
                let mapped = resources
                    .iter()
                    .find(|mapping| mapping.component_port_no == port)
                    .map(|mapping| mapping.resource_slot);
                if mapped.is_none() {
                    verdict = merge_verdict(verdict, LeafVerdict::Unproven);
                }
                residuals.push(ScopedEffectResidual {
                    caller_semantic_ir_digest: caller_semantic_ir_digest.clone(),
                    source_contract_digest: digest.clone(),
                    source_component: identity.clone(),
                    computation,
                    source_entry_ordinal: index,
                    residual: residual.clone(),
                    mapped_resource_slot: mapped,
                });
            }
        }
        EffectRow::Unknown => verdict = LeafVerdict::Unproven,
    }
    (verdict, known, residuals)
}

fn scoped_flow(
    contract: &af_flow::InformationFlowContract,
    inputs: &[ScopedPortMapping],
    outputs: &[ScopedPortMapping],
    resources: &[ScopedResourcePortMapping],
    effects: &[ScopedEffectOccurrence],
) -> (LeafVerdict, Vec<FlowPortMapping>) {
    let mut verdict = LeafVerdict::Proven;
    let mut mappings = Vec::new();
    for source in &contract.summary.sources {
        let binding = match &source.category {
            SourceCategory::WorkflowInput(port) => inputs
                .iter()
                .find(|item| item.contract_port_no == *port)
                .and_then(|item| match item.binding {
                    ScopedPortBinding::InputExpression(expression) => {
                        Some(FlowSourceBinding::InputExpression(expression))
                    }
                    ScopedPortBinding::OutputValue(_) => None,
                }),
            SourceCategory::ResourceRequirement(port) => resources
                .iter()
                .find(|item| item.component_port_no == *port)
                .map(|item| FlowSourceBinding::ResourceSlot(item.resource_slot)),
            SourceCategory::UnknownExternal => Some(FlowSourceBinding::UnknownExternal),
        };
        if binding.is_none() {
            verdict = merge_verdict(verdict, LeafVerdict::Unproven);
        }
        mappings.push(FlowPortMapping {
            source: Some(source.clone()),
            sink: None,
            source_binding: binding,
            sink_binding: None,
            matched_effect_occurrences: vec![],
        });
    }
    for constraint in &contract.flow_constraints {
        let (sink_binding, matched) = match &constraint.sink {
            FlowSink::WorkflowOutput(port) => {
                let binding = outputs
                    .iter()
                    .find(|item| item.contract_port_no == *port)
                    .and_then(|item| match item.binding {
                        ScopedPortBinding::OutputValue(slot) => {
                            Some(FlowSinkBinding::OutputValue(slot))
                        }
                        ScopedPortBinding::InputExpression(_) => None,
                    });
                if binding.is_none() {
                    verdict = merge_verdict(verdict, LeafVerdict::Unproven);
                }
                (binding, vec![])
            }
            FlowSink::ExternalEffect(operation) => {
                let indices: Vec<usize> = effects
                    .iter()
                    .enumerate()
                    .filter_map(|(index, occurrence)| {
                        (occurrence.declared.atom.operation == *operation).then_some(index)
                    })
                    .collect();
                if indices.is_empty() {
                    verdict = merge_verdict(verdict, LeafVerdict::Unproven);
                }
                (
                    Some(FlowSinkBinding::EffectOccurrences(indices.clone())),
                    indices,
                )
            }
        };
        mappings.push(FlowPortMapping {
            source: None,
            sink: Some(constraint.clone()),
            source_binding: None,
            sink_binding,
            matched_effect_occurrences: matched,
        });
    }
    (verdict, mappings)
}

fn subflow_closure(universe: &SubflowIrUniverse, root: &DigestRefValue) -> (bool, bool) {
    fn visit(
        universe: &SubflowIrUniverse,
        digest: &DigestRefValue,
        states: &mut HashMap<DigestRefValue, u8>,
        cycle: &mut bool,
        missing: &mut bool,
    ) {
        match states.get(digest).copied() {
            Some(1) => {
                *cycle = true;
                return;
            }
            Some(2) => return,
            _ => {}
        }
        let Some(target) = universe.target(digest) else {
            *missing = true;
            return;
        };
        states.insert(digest.clone(), 1);
        for dependency in target.dependencies() {
            visit(universe, dependency, states, cycle, missing);
        }
        states.insert(digest.clone(), 2);
    }
    let mut states = HashMap::new();
    let (mut cycle, mut missing) = (false, false);
    visit(universe, root, &mut states, &mut cycle, &mut missing);
    (cycle, missing)
}

#[cfg(test)]
mod tests {
    use super::*;
    use af_effect::{
        EffectAtom, EffectEntry, EffectGuard, EffectIdempotencyQualifier, EffectOperationRef,
        EffectQualifier, OccurrenceBounds, ResourceSelector,
    };

    #[test]
    fn same_source_atom_at_two_nodes_yields_two_scoped_occurrences() {
        let atom = EffectAtom {
            operation: EffectOperationRef {
                catalog: "agentflow.effects@1".into(),
                name: "write".into(),
            },
            resource_selector: ResourceSelector {
                requirement_port_no: 4,
                scope_bindings: vec![],
            },
            qualifiers: vec![EffectQualifier::Idempotency(
                EffectIdempotencyQualifier::Natural,
            )],
        };
        let signature = EffectSignature {
            row: EffectRow::Closed(vec![EffectEntry {
                atom: atom.clone(),
                bounds: OccurrenceBounds::finite(1, 2).unwrap(),
                guard: EffectGuard::Unconditional,
            }]),
        };
        let caller = DigestRefValue {
            algorithm: "sha256".into(),
            domain: "semantic-ir".into(),
            hex: "1".repeat(64),
        };
        let contract = DigestRefValue {
            algorithm: "sha256".into(),
            domain: "component-contract".into(),
            hex: "2".repeat(64),
        };
        let identity = af_contract::ComponentIdentity {
            catalog: "agentflow.components@1".into(),
            name: "acme.writer".into(),
        };
        let resources = [];
        let first = scoped_effects(
            &signature,
            &caller,
            &contract,
            &identity,
            ComputationId::from_raw(8),
            &resources,
        );
        let second = scoped_effects(
            &signature,
            &caller,
            &contract,
            &identity,
            ComputationId::from_raw(9),
            &resources,
        );
        assert_eq!(first.0, LeafVerdict::Unproven);
        assert_eq!(first.1.len(), 1);
        assert_eq!(second.1.len(), 1);
        assert_eq!(first.1[0].declared.atom, atom);
        assert_eq!(first.1[0].declared.atom.qualifiers.len(), 1);
        assert_eq!(
            first.1[0].source_contract_digest,
            second.1[0].source_contract_digest
        );
        assert_ne!(first.1[0].computation, second.1[0].computation);
        assert_eq!(
            first.1[0].caller_semantic_ir_digest,
            second.1[0].caller_semantic_ir_digest
        );
    }
}
