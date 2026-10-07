//! The semantic IR static validator.
//!
//! # Validation order (deterministic; the negative corpus freezes one expected
//! kind per isolated violation)
//!
//! 1. `semantic_refs`: `SpecificationId` syntax, then duplicates;
//! 2. the four reused Phase 3A tables via [`SemanticCoreDocument::check`]
//!    (errors pass through as [`IrError::Core`]);
//! 3. `run_parameters`: ids, names (non-empty, NFC, unique after NFC), type
//!    and slot resolution, `parameter.type == slot.type`; registers the
//!    parameter's value-slot producer;
//! 4. `workflow_outputs`: names (non-empty, NFC, unique after NFC), expression
//!    existence (the output type is the expression's result type);
//! 5. `contracts`: ids, digest shape, digest domain per kind, unique contract
//!    identity;
//! 6. `computations`, pass A (local, in canonical id order): ids, output
//!    ports, and per-body static rules — references exist, ordinals unique,
//!    typing (`selector == int64`, `guard == bool`, `timeout == duration`,
//!    `source == list<T>`), max-inflight/iteration bounds, contract kinds,
//!    carry bindings; registers computation-output / iterate-item / carry
//!    producers;
//! 7. producer uniqueness: every declared value slot has exactly one producer
//!    (0 → `unbound_value_slot`, >1 → `duplicate_value_producer`);
//! 8. ownership: root exists (`root == 0` is an id error), structural cycle,
//!    root has no parent, exactly one parent per non-root computation,
//!    everything reachable from root;
//! 9. wrapper interface equality (SEQUENCE == last child, SELECT == every
//!    case, RACE == every branch, RETRY/TIMEOUT/SCOPE/COMPENSATION_SCOPE ==
//!    child, HANDLE == protected child + every handler, PARALLEL by explicit
//!    mapping, ITERATE by carry rules).
//!
//! Dataflow (expression → slot) and ownership (computation → child) are
//! deliberately separate logics sharing no edge type. Dominance and
//! value-availability analysis are Phase 3C and are not approximated here.

use crate::error::{IrError, kind};
use crate::ids::{
    BranchOrdinal, ComputationId, ContractRefId, HandlerOrdinal, PortNo, RunParameterId,
};
use crate::model::{
    ComputationBody, ComputationDef, ContractRefEntry, ContractRefKind, ForeachBody, HandleBody,
    HandleMatch, InputBinding, IterateBody, LoopCarryBinding, ParallelBody, ResourceUseBinding,
    SelectBody, SemanticIrDocument, WhileBody,
};
use af_core::DigestDomain;
use af_semantic::{
    ExpressionDef, ExpressionId, ResourceSlotId, SemanticCoreDocument, SemanticError,
    SemanticTypeId, SemanticTypeNode, SemanticTypeNodeBody, TypeSchemaUniverse, ValueSlot,
    ValueSlotId,
};
use std::collections::{BTreeMap, BTreeSet};
use unicode_normalization::UnicodeNormalization;

/// A validated semantic IR document. Construction only through
/// [`SemanticIrDocument::check`].
#[derive(Debug, Clone)]
pub struct CheckedIrDocument<'d> {
    source: &'d SemanticIrDocument,
}

impl<'d> CheckedIrDocument<'d> {
    /// The validated source document.
    pub fn source(&self) -> &'d SemanticIrDocument {
        self.source
    }

    /// The derived output interface of one computation:
    /// `port_no -> value slot type`, straight from the computation's outputs
    /// and the value-slot table. There is deliberately no second stored
    /// interface table anywhere in the model.
    pub fn interface(&self, id: ComputationId) -> Result<BTreeMap<u32, u32>, IrError> {
        let computation = self.computation(id)?;
        let mut interface = BTreeMap::new();
        for binding in &computation.outputs {
            let slot = self.value_slot(binding.value_slot)?;
            interface.insert(binding.port_no.raw(), slot.type_id.raw());
        }
        Ok(interface)
    }

    fn computation(&self, id: ComputationId) -> Result<&ComputationDef, IrError> {
        self.source
            .computations
            .iter()
            .find(|computation| computation.id == id)
            .ok_or(IrError::UnknownComputation { id })
    }

    fn value_slot(&self, id: ValueSlotId) -> Result<&ValueSlot, IrError> {
        self.source
            .values
            .iter()
            .find(|slot| slot.id == id)
            .ok_or_else(|| {
                IrError::rule(
                    kind::OUTPUT_UNKNOWN_SLOT,
                    format!("value slot {id} is not declared"),
                )
            })
    }
}

impl SemanticIrDocument {
    /// Validates this document against an explicit type schema universe.
    ///
    /// # Errors
    /// The first [`IrError`] in the documented validation order. Semantic
    /// core violations pass through as [`IrError::Core`].
    pub fn check<'d>(
        &'d self,
        universe: &TypeSchemaUniverse,
    ) -> Result<CheckedIrDocument<'d>, IrError> {
        check_semantic_refs(&self.semantic_refs)?;

        // The four reused core tables must be valid in their own right before
        // any IR-level rule reads them.
        let core = SemanticCoreDocument {
            types: self.types.clone(),
            values: self.values.clone(),
            expressions: self.expressions.clone(),
            resources: self.resources.clone(),
        };
        core.check(universe)?;
        check_schema_refs(self, universe)?;

        let types: BTreeMap<u32, &SemanticTypeNode> = self
            .types
            .iter()
            .map(|node| (node.id.raw(), node))
            .collect();
        let values: BTreeMap<u32, &ValueSlot> = self
            .values
            .iter()
            .map(|slot| (slot.id.raw(), slot))
            .collect();
        let expressions: BTreeMap<u32, &ExpressionDef> = self
            .expressions
            .iter()
            .map(|def| (def.id.raw(), def))
            .collect();
        let resources: BTreeSet<u32> = self.resources.iter().map(|slot| slot.id.raw()).collect();
        let computations: BTreeMap<u32, &ComputationDef> = self
            .computations
            .iter()
            .map(|def| (def.id.raw(), def))
            .collect();
        let contracts: BTreeMap<u32, &ContractRefEntry> = self
            .contracts
            .iter()
            .map(|entry| (entry.id.raw(), entry))
            .collect();

        let mut producers: BTreeMap<u32, &'static str> = BTreeMap::new();
        check_run_parameters(self, &types, &values, &mut producers)?;
        check_workflow_outputs(self, &expressions)?;
        check_contracts(self)?;
        check_computations(
            self,
            &types,
            &values,
            &expressions,
            &resources,
            &computations,
            &contracts,
            &mut producers,
        )?;
        check_producer_uniqueness(&values, &producers)?;
        check_ownership(self, &computations)?;
        check_interfaces(self, &values, &computations)?;

        Ok(CheckedIrDocument { source: self })
    }
}

fn check_semantic_refs(refs: &[crate::model::SemanticVersionRef]) -> Result<(), IrError> {
    let mut seen = BTreeSet::new();
    for reference in refs {
        if af_idl::SpecificationId::parse(&reference.specification_id).is_err() {
            return Err(IrError::rule(
                kind::INVALID_SEMANTIC_REF,
                format!(
                    "{:?} is not a canonical SpecificationId",
                    reference.specification_id
                ),
            ));
        }
        if !seen.insert(reference.specification_id.as_str()) {
            return Err(IrError::rule(
                kind::DUPLICATE_SEMANTIC_REF,
                format!(
                    "semantic ref {:?} declared twice",
                    reference.specification_id
                ),
            ));
        }
    }
    Ok(())
}

fn nfc(text: &str) -> String {
    text.nfc().collect()
}

fn check_name(
    name: &str,
    invalid_kind: &'static str,
    duplicate_kind: &'static str,
    seen: &mut BTreeSet<String>,
) -> Result<(), IrError> {
    if name.is_empty() {
        return Err(IrError::rule(invalid_kind, "name must be non-empty"));
    }
    if nfc(name) != name {
        return Err(IrError::rule(
            invalid_kind,
            format!("name {name:?} is not NFC-normalized"),
        ));
    }
    if !seen.insert(nfc(name)) {
        return Err(IrError::rule(
            duplicate_kind,
            format!("name {name:?} collides after NFC normalization"),
        ));
    }
    Ok(())
}

fn check_run_parameters(
    document: &SemanticIrDocument,
    types: &BTreeMap<u32, &SemanticTypeNode>,
    values: &BTreeMap<u32, &ValueSlot>,
    producers: &mut BTreeMap<u32, &'static str>,
) -> Result<(), IrError> {
    let mut ids: BTreeSet<u32> = BTreeSet::new();
    let mut names: BTreeSet<String> = BTreeSet::new();
    for parameter in &document.run_parameters {
        if !parameter.id.is_valid() {
            return Err(IrError::InvalidLocalId {
                namespace: RunParameterId::namespace(),
                id: parameter.id.raw(),
            });
        }
        if !ids.insert(parameter.id.raw()) {
            return Err(IrError::rule(
                kind::DUPLICATE_RUN_PARAMETER,
                format!("run parameter {} declared twice", parameter.id),
            ));
        }
        check_name(
            &parameter.name,
            kind::INVALID_RUN_PARAMETER_NAME,
            kind::DUPLICATE_RUN_PARAMETER_NAME,
            &mut names,
        )?;
        if !types.contains_key(&parameter.type_id.raw()) {
            return Err(IrError::Core(SemanticError::UnknownSemanticType {
                id: parameter.type_id.raw(),
            }));
        }
        let Some(slot) = values.get(&parameter.value_slot.raw()) else {
            return Err(IrError::Core(SemanticError::UnknownValueSlot {
                id: parameter.value_slot.raw(),
            }));
        };
        if slot.type_id != parameter.type_id {
            return Err(IrError::rule(
                kind::RUN_PARAMETER_TYPE_MISMATCH,
                format!(
                    "run parameter {} has type {}, but its value slot {} has type {}",
                    parameter.id, parameter.type_id, parameter.value_slot, slot.type_id
                ),
            ));
        }
        register_producer(producers, parameter.value_slot.raw(), "run_parameter")?;
    }
    Ok(())
}

fn check_workflow_outputs(
    document: &SemanticIrDocument,
    expressions: &BTreeMap<u32, &ExpressionDef>,
) -> Result<(), IrError> {
    let mut names: BTreeSet<String> = BTreeSet::new();
    for output in &document.workflow_outputs {
        check_name(
            &output.name,
            kind::INVALID_WORKFLOW_OUTPUT_NAME,
            kind::DUPLICATE_WORKFLOW_OUTPUT_NAME,
            &mut names,
        )?;
        if !expressions.contains_key(&output.expression.raw()) {
            return Err(IrError::Core(SemanticError::UnknownExpression {
                id: output.expression.raw(),
            }));
        }
    }
    Ok(())
}

fn check_contracts(document: &SemanticIrDocument) -> Result<(), IrError> {
    let mut ids: BTreeSet<u32> = BTreeSet::new();
    let mut identities: BTreeSet<(String, String, String)> = BTreeSet::new();
    for entry in &document.contracts {
        if !entry.id.is_valid() {
            return Err(IrError::InvalidLocalId {
                namespace: ContractRefId::namespace(),
                id: entry.id.raw(),
            });
        }
        if !ids.insert(entry.id.raw()) {
            return Err(IrError::rule(
                kind::DUPLICATE_CONTRACT_REF,
                format!("contract {} declared twice", entry.id),
            ));
        }
        let digest = &entry.digest;
        if digest.algorithm != "sha256" {
            return Err(IrError::rule(
                kind::INVALID_CONTRACT_DIGEST,
                format!(
                    "contract {} algorithm {:?} is not sha256",
                    entry.id, digest.algorithm
                ),
            ));
        }
        let Ok(domain) = DigestDomain::from_registry_name(&digest.domain) else {
            return Err(IrError::rule(
                kind::INVALID_CONTRACT_DIGEST,
                format!(
                    "contract {} domain {:?} is not a HashV1 domain",
                    entry.id, digest.domain
                ),
            ));
        };
        let expected = match entry.kind {
            ContractRefKind::Component => DigestDomain::ComponentContract,
            ContractRefKind::Subflow => DigestDomain::SemanticIr,
        };
        if domain != expected {
            return Err(IrError::rule(
                kind::CONTRACT_DIGEST_DOMAIN,
                format!(
                    "contract {} of kind {:?} must pin the {:?} domain, found {:?}",
                    entry.id,
                    entry.kind.keyword(),
                    expected.as_str(),
                    digest.domain
                ),
            ));
        }
        if digest.hex.len() != 64
            || !digest
                .hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(IrError::rule(
                kind::INVALID_CONTRACT_DIGEST,
                format!(
                    "contract {} hex must be 64 lowercase hex characters",
                    entry.id
                ),
            ));
        }
        if !identities.insert((
            digest.algorithm.clone(),
            digest.domain.clone(),
            digest.hex.clone(),
        )) {
            return Err(IrError::rule(
                kind::DUPLICATE_CONTRACT_IDENTITY,
                format!(
                    "the exact contract {}/{} is referenced by more than one table entry",
                    digest.domain, digest.hex
                ),
            ));
        }
    }
    Ok(())
}

fn register_producer(
    producers: &mut BTreeMap<u32, &'static str>,
    slot: u32,
    producer: &'static str,
) -> Result<(), IrError> {
    if producers.insert(slot, producer).is_some() {
        return Err(IrError::rule(
            kind::DUPLICATE_VALUE_PRODUCER,
            format!("value slot {slot} has more than one producer"),
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn check_computations(
    document: &SemanticIrDocument,
    types: &BTreeMap<u32, &SemanticTypeNode>,
    values: &BTreeMap<u32, &ValueSlot>,
    expressions: &BTreeMap<u32, &ExpressionDef>,
    resources: &BTreeSet<u32>,
    computations: &BTreeMap<u32, &ComputationDef>,
    contracts: &BTreeMap<u32, &ContractRefEntry>,
    producers: &mut BTreeMap<u32, &'static str>,
) -> Result<(), IrError> {
    let mut ids: BTreeSet<u32> = BTreeSet::new();
    for computation in &document.computations {
        if !computation.id.is_valid() {
            return Err(IrError::InvalidLocalId {
                namespace: ComputationId::namespace(),
                id: computation.id.raw(),
            });
        }
        if !ids.insert(computation.id.raw()) {
            return Err(IrError::rule(
                kind::DUPLICATE_COMPUTATION,
                format!("computation {} declared twice", computation.id),
            ));
        }
        // Output ports: positive, unique, bound to a declared slot of the
        // document (the port's type is the slot's type — never duplicated).
        let mut ports: BTreeSet<u32> = BTreeSet::new();
        for binding in &computation.outputs {
            if !binding.port_no.is_valid() {
                return Err(IrError::InvalidLocalId {
                    namespace: PortNo::namespace(),
                    id: binding.port_no.raw(),
                });
            }
            if !ports.insert(binding.port_no.raw()) {
                return Err(IrError::rule(
                    kind::DUPLICATE_OUTPUT_PORT,
                    format!(
                        "computation {} declares output port {} twice",
                        computation.id, binding.port_no
                    ),
                ));
            }
            if !values.contains_key(&binding.value_slot.raw()) {
                return Err(IrError::Core(SemanticError::UnknownValueSlot {
                    id: binding.value_slot.raw(),
                }));
            }
            register_producer(producers, binding.value_slot.raw(), "computation_output")?;
        }
    }

    // Per-body static rules, in canonical (id ascending) order.
    for computation in computations.values() {
        check_body(
            computation,
            types,
            values,
            expressions,
            resources,
            computations,
            contracts,
            producers,
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn check_body(
    computation: &ComputationDef,
    types: &BTreeMap<u32, &SemanticTypeNode>,
    values: &BTreeMap<u32, &ValueSlot>,
    expressions: &BTreeMap<u32, &ExpressionDef>,
    resources: &BTreeSet<u32>,
    computations: &BTreeMap<u32, &ComputationDef>,
    contracts: &BTreeMap<u32, &ContractRefEntry>,
    producers: &mut BTreeMap<u32, &'static str>,
) -> Result<(), IrError> {
    // Every owned child reference must resolve, and no child may be listed
    // twice by the same parent.
    let children = computation.owned_children();
    let mut seen_children: BTreeSet<u32> = BTreeSet::new();
    for child in &children {
        if !computations.contains_key(&child.raw()) {
            return Err(IrError::UnknownComputation { id: *child });
        }
        if !seen_children.insert(child.raw()) {
            return Err(IrError::rule(
                kind::DUPLICATE_CHILD_REFERENCE,
                format!(
                    "computation {} owns child {} more than once",
                    computation.id, child
                ),
            ));
        }
    }

    match &computation.body {
        ComputationBody::Invoke(body) => {
            check_contract_use(
                body.contract,
                ContractRefKind::Component,
                contracts,
                kind::INVOKE_UNKNOWN_CONTRACT,
                kind::INVOKE_CONTRACT_KIND,
            )?;
            check_call_bindings(&body.inputs, &body.resources, resources, expressions)?;
        }
        ComputationBody::Await(body) => {
            // The signal schema must resolve exactly in the supplied universe;
            // that happens in the document-level check via `await_schema_refs`.
            let _ = body;
        }
        ComputationBody::Sequence(body) => {
            if body.children.is_empty() {
                return Err(IrError::rule(
                    kind::SEQUENCE_EMPTY,
                    format!(
                        "computation {}: a sequence has at least one child",
                        computation.id
                    ),
                ));
            }
        }
        ComputationBody::Select(body) => check_select(computation, body, types, expressions)?,
        ComputationBody::Parallel(body) => check_parallel(computation, body, values, computations)?,
        ComputationBody::Race(body) => {
            if body.branches.len() < 2 {
                return Err(IrError::rule(
                    kind::RACE_INSUFFICIENT_BRANCHES,
                    format!(
                        "computation {}: a race has at least two branches, found {}",
                        computation.id,
                        body.branches.len()
                    ),
                ));
            }
            check_branch_ordinals(computation, body.branches.iter().map(|b| b.ordinal))?;
        }
        ComputationBody::Iterate(body) => match body {
            IterateBody::Foreach(foreach) => check_foreach(
                computation,
                foreach,
                types,
                values,
                expressions,
                computations,
                producers,
            )?,
            IterateBody::While(while_body) => check_while(
                computation,
                while_body,
                types,
                values,
                expressions,
                computations,
                producers,
            )?,
        },
        ComputationBody::Retry(body) => {
            for delay in &body.delays {
                if delay.as_micros() < 0 {
                    return Err(IrError::rule(
                        kind::RETRY_INVALID_DELAY,
                        format!(
                            "computation {}: retry delays are non-negative, found {} micros",
                            computation.id,
                            delay.as_micros()
                        ),
                    ));
                }
            }
        }
        ComputationBody::Timeout(body) => {
            let timeout_type = result_type(body.timeout, expressions)?;
            if !is_primitive(types, timeout_type, "duration") {
                return Err(IrError::rule(
                    kind::TIMEOUT_NOT_DURATION,
                    format!(
                        "computation {}: the timeout expression must have type duration, found {}",
                        computation.id, timeout_type
                    ),
                ));
            }
        }
        ComputationBody::Handle(body) => check_handle(computation, body, values, computations)?,
        ComputationBody::Scope(body) => {
            check_scope_resources(computation, &body.resources, resources)?;
        }
        ComputationBody::CompensationScope(_) | ComputationBody::CallSubflow(_) => {}
    }

    if let ComputationBody::CallSubflow(body) = &computation.body {
        check_contract_use(
            body.contract,
            ContractRefKind::Subflow,
            contracts,
            kind::CALL_SUBFLOW_UNKNOWN_CONTRACT,
            kind::CALL_SUBFLOW_CONTRACT_KIND,
        )?;
        check_call_bindings(&body.inputs, &body.resources, resources, expressions)?;
    }
    Ok(())
}

fn check_contract_use(
    contract: ContractRefId,
    expected: ContractRefKind,
    contracts: &BTreeMap<u32, &ContractRefEntry>,
    unknown_kind: &'static str,
    mismatch_kind: &'static str,
) -> Result<(), IrError> {
    let Some(entry) = contracts.get(&contract.raw()) else {
        return Err(IrError::rule(
            unknown_kind,
            format!("contract {contract} is not declared in the contract table"),
        ));
    };
    if entry.kind != expected {
        return Err(IrError::rule(
            mismatch_kind,
            format!(
                "contract {} has kind {:?}, but this opcode requires {:?}",
                contract,
                entry.kind.keyword(),
                expected.keyword()
            ),
        ));
    }
    Ok(())
}

fn check_call_bindings(
    inputs: &[InputBinding],
    resource_uses: &[ResourceUseBinding],
    resources: &BTreeSet<u32>,
    expressions: &BTreeMap<u32, &ExpressionDef>,
) -> Result<(), IrError> {
    let mut ports: BTreeSet<u32> = BTreeSet::new();
    for binding in inputs {
        if !binding.port_no.is_valid() {
            return Err(IrError::InvalidLocalId {
                namespace: PortNo::namespace(),
                id: binding.port_no.raw(),
            });
        }
        if !ports.insert(binding.port_no.raw()) {
            return Err(IrError::rule(
                kind::DUPLICATE_INPUT_PORT,
                format!("duplicate input port {}", binding.port_no),
            ));
        }
        if !expressions.contains_key(&binding.expression.raw()) {
            return Err(IrError::Core(SemanticError::UnknownExpression {
                id: binding.expression.raw(),
            }));
        }
    }
    let mut resource_ports: BTreeSet<u32> = BTreeSet::new();
    for binding in resource_uses {
        if !binding.port_no.is_valid() {
            return Err(IrError::InvalidLocalId {
                namespace: PortNo::namespace(),
                id: binding.port_no.raw(),
            });
        }
        if !resource_ports.insert(binding.port_no.raw()) {
            return Err(IrError::rule(
                kind::DUPLICATE_RESOURCE_PORT,
                format!("duplicate resource port {}", binding.port_no),
            ));
        }
        if !resources.contains(&binding.resource_slot.raw()) {
            return Err(IrError::rule(
                kind::UNKNOWN_RESOURCE_SLOT,
                format!("resource slot {} is not declared", binding.resource_slot),
            ));
        }
    }
    Ok(())
}

fn result_type(
    expression: ExpressionId,
    expressions: &BTreeMap<u32, &ExpressionDef>,
) -> Result<SemanticTypeId, IrError> {
    expressions
        .get(&expression.raw())
        .map(|def| def.result_type_id)
        .ok_or_else(|| {
            IrError::Core(SemanticError::UnknownExpression {
                id: expression.raw(),
            })
        })
}

fn is_primitive(
    types: &BTreeMap<u32, &SemanticTypeNode>,
    type_id: SemanticTypeId,
    keyword: &str,
) -> bool {
    types.get(&type_id.raw()).is_some_and(|node| {
        matches!(&node.body, SemanticTypeNodeBody::Primitive(primitive)
            if primitive.keyword() == keyword)
    })
}

fn check_select(
    computation: &ComputationDef,
    body: &SelectBody,
    types: &BTreeMap<u32, &SemanticTypeNode>,
    expressions: &BTreeMap<u32, &ExpressionDef>,
) -> Result<(), IrError> {
    if body.cases.is_empty() {
        return Err(IrError::rule(
            kind::SELECT_EMPTY,
            format!(
                "computation {}: a select has at least one case",
                computation.id
            ),
        ));
    }
    let selector_type = result_type(body.selector, expressions)?;
    if !is_primitive(types, selector_type, "int64") {
        return Err(IrError::rule(
            kind::SELECTOR_NOT_INT64,
            format!(
                "computation {}: the select selector must have type int64, found {}",
                computation.id, selector_type
            ),
        ));
    }
    check_branch_ordinals(computation, body.cases.iter().map(|case| case.ordinal))?;
    Ok(())
}

fn check_branch_ordinals<I: Iterator<Item = BranchOrdinal>>(
    computation: &ComputationDef,
    ordinals: I,
) -> Result<(), IrError> {
    let mut seen: BTreeSet<u32> = BTreeSet::new();
    for ordinal in ordinals {
        if !ordinal.is_valid() {
            return Err(IrError::InvalidLocalId {
                namespace: BranchOrdinal::namespace(),
                id: ordinal.raw(),
            });
        }
        if !seen.insert(ordinal.raw()) {
            return Err(IrError::rule(
                kind::DUPLICATE_BRANCH_ORDINAL,
                format!(
                    "computation {}: duplicate ordinal {}",
                    computation.id, ordinal
                ),
            ));
        }
    }
    Ok(())
}

fn check_max_inflight(
    computation: &ComputationDef,
    max_inflight: Option<u32>,
) -> Result<(), IrError> {
    if max_inflight.is_some_and(|bound| bound == 0) {
        return Err(IrError::rule(
            kind::INVALID_MAX_INFLIGHT,
            format!(
                "computation {}: max_inflight must be positive when present",
                computation.id
            ),
        ));
    }
    Ok(())
}

fn check_parallel(
    computation: &ComputationDef,
    body: &ParallelBody,
    values: &BTreeMap<u32, &ValueSlot>,
    computations: &BTreeMap<u32, &ComputationDef>,
) -> Result<(), IrError> {
    if body.branches.is_empty() {
        return Err(IrError::rule(
            kind::PARALLEL_EMPTY,
            format!(
                "computation {}: a parallel has at least one branch",
                computation.id
            ),
        ));
    }
    check_max_inflight(computation, body.max_inflight)?;
    check_branch_ordinals(computation, body.branches.iter().map(|b| b.ordinal))?;

    // The explicit output mapping is the only path from branch outputs into
    // the parent interface.
    let mut parent_ports: BTreeSet<u32> = BTreeSet::new();
    for binding in &body.output_bindings {
        if !parent_ports.insert(binding.parent_port_no.raw()) {
            return Err(IrError::rule(
                kind::PARALLEL_OUTPUT_DUPLICATE_PORT,
                format!(
                    "computation {}: parent port {} is bound by two output bindings",
                    computation.id, binding.parent_port_no
                ),
            ));
        }
        let Some(branch) = body
            .branches
            .iter()
            .find(|branch| branch.ordinal == binding.branch_ordinal)
        else {
            return Err(IrError::rule(
                kind::PARALLEL_OUTPUT_UNKNOWN_BRANCH,
                format!(
                    "computation {}: output binding names branch {}, which does not exist",
                    computation.id, binding.branch_ordinal
                ),
            ));
        };
        let child = computations
            .get(&branch.child.raw())
            .expect("child existence validated before body rules");
        let child_port = child
            .outputs
            .iter()
            .find(|output| output.port_no == binding.child_port_no)
            .ok_or_else(|| {
                IrError::rule(
                    kind::PARALLEL_OUTPUT_UNKNOWN_PORT,
                    format!(
                        "computation {}: branch {} child {} has no output port {}",
                        computation.id, binding.branch_ordinal, branch.child, binding.child_port_no
                    ),
                )
            })?;
        let Some(parent_output) = computation
            .outputs
            .iter()
            .find(|output| output.port_no == binding.parent_port_no)
        else {
            return Err(IrError::rule(
                kind::PARALLEL_OUTPUT_UNKNOWN_PORT,
                format!(
                    "computation {}: output binding names parent port {}, which does not exist",
                    computation.id, binding.parent_port_no
                ),
            ));
        };
        let parent_slot = values
            .get(&parent_output.value_slot.raw())
            .expect("output slots validated before body rules");
        let child_slot = values
            .get(&child_port.value_slot.raw())
            .expect("child output slots validated before body rules");
        if parent_slot.type_id != child_slot.type_id {
            return Err(IrError::rule(
                kind::PARALLEL_OUTPUT_TYPE_MISMATCH,
                format!(
                    "computation {}: output binding for parent port {} maps type {} \
                     onto branch type {}",
                    computation.id, binding.parent_port_no, parent_slot.type_id, child_slot.type_id
                ),
            ));
        }
    }
    Ok(())
}

fn check_foreach(
    computation: &ComputationDef,
    body: &ForeachBody,
    types: &BTreeMap<u32, &SemanticTypeNode>,
    values: &BTreeMap<u32, &ValueSlot>,
    expressions: &BTreeMap<u32, &ExpressionDef>,
    computations: &BTreeMap<u32, &ComputationDef>,
    producers: &mut BTreeMap<u32, &'static str>,
) -> Result<(), IrError> {
    check_max_inflight(computation, body.max_inflight)?;
    let source_type = result_type(body.source, expressions)?;
    let Some(item_type) = list_element_type(types, source_type) else {
        return Err(IrError::rule(
            kind::FOREACH_SOURCE_NOT_LIST,
            format!(
                "computation {}: the foreach source must have type list<T>, found {}",
                computation.id, source_type
            ),
        ));
    };
    let Some(slot) = values.get(&body.item_slot.raw()) else {
        return Err(IrError::Core(SemanticError::UnknownValueSlot {
            id: body.item_slot.raw(),
        }));
    };
    if slot.type_id != item_type {
        return Err(IrError::rule(
            kind::FOREACH_ITEM_TYPE_MISMATCH,
            format!(
                "computation {}: the item slot has type {}, but the source element type is {}",
                computation.id, slot.type_id, item_type
            ),
        ));
    }
    // The item slot's producer is this ITERATE.
    register_producer(producers, body.item_slot.raw(), "iterate_item")?;
    check_carry(
        computation,
        &body.carry,
        body.body,
        types,
        values,
        expressions,
        computations,
        producers,
    )
}

fn check_while(
    computation: &ComputationDef,
    body: &WhileBody,
    types: &BTreeMap<u32, &SemanticTypeNode>,
    values: &BTreeMap<u32, &ValueSlot>,
    expressions: &BTreeMap<u32, &ExpressionDef>,
    computations: &BTreeMap<u32, &ComputationDef>,
    producers: &mut BTreeMap<u32, &'static str>,
) -> Result<(), IrError> {
    if body.max_iterations == 0 {
        return Err(IrError::rule(
            kind::INVALID_MAX_ITERATIONS,
            format!(
                "computation {}: max_iterations must be positive",
                computation.id
            ),
        ));
    }
    let guard_type = result_type(body.guard, expressions)?;
    if !is_primitive(types, guard_type, "bool") {
        return Err(IrError::rule(
            kind::WHILE_GUARD_NOT_BOOL,
            format!(
                "computation {}: the while guard must have type bool, found {}",
                computation.id, guard_type
            ),
        ));
    }
    check_carry(
        computation,
        &body.carry,
        body.body,
        types,
        values,
        expressions,
        computations,
        producers,
    )
}

#[allow(clippy::too_many_arguments)]
fn check_carry(
    computation: &ComputationDef,
    carry: &[LoopCarryBinding],
    body: ComputationId,
    _types: &BTreeMap<u32, &SemanticTypeNode>,
    values: &BTreeMap<u32, &ValueSlot>,
    expressions: &BTreeMap<u32, &ExpressionDef>,
    computations: &BTreeMap<u32, &ComputationDef>,
    producers: &mut BTreeMap<u32, &'static str>,
) -> Result<(), IrError> {
    let mut slots: BTreeSet<u32> = BTreeSet::new();
    for binding in carry {
        if !slots.insert(binding.slot.raw()) {
            return Err(IrError::rule(
                kind::DUPLICATE_CARRY_SLOT,
                format!(
                    "computation {}: carry slot {} is bound twice",
                    computation.id, binding.slot
                ),
            ));
        }
        let Some(slot) = values.get(&binding.slot.raw()) else {
            return Err(IrError::rule(
                kind::CARRY_UNKNOWN_SLOT,
                format!(
                    "computation {}: carry slot {} is not declared",
                    computation.id, binding.slot
                ),
            ));
        };
        let initial_type = result_type(binding.initial, expressions)?;
        // The body's output port supplies the next iteration's value.
        let body_computation = computations
            .get(&body.raw())
            .expect("child existence validated before body rules");
        let Some(body_port) = body_computation
            .outputs
            .iter()
            .find(|output| output.port_no == binding.body_output_port)
        else {
            return Err(IrError::rule(
                kind::CARRY_UNKNOWN_PORT,
                format!(
                    "computation {}: the loop body {} has no output port {}",
                    computation.id, body, binding.body_output_port
                ),
            ));
        };
        let next_slot = values
            .get(&body_port.value_slot.raw())
            .expect("output slots validated before body rules");
        let consistent = slot.type_id == initial_type && slot.type_id == next_slot.type_id;
        if !consistent {
            return Err(IrError::rule(
                kind::CARRY_TYPE_MISMATCH,
                format!(
                    "computation {}: carry slot {} has type {}, initial {} has type {}, \
                     the next-iteration port has type {}",
                    computation.id,
                    binding.slot,
                    slot.type_id,
                    binding.initial,
                    initial_type,
                    next_slot.type_id
                ),
            ));
        }
        // The carry slot's producer is this ITERATE (per-iteration values are
        // new occurrences of the template, never mutations).
        register_producer(producers, binding.slot.raw(), "iterate_carry")?;
    }
    Ok(())
}

fn list_element_type(
    types: &BTreeMap<u32, &SemanticTypeNode>,
    type_id: SemanticTypeId,
) -> Option<SemanticTypeId> {
    match &types.get(&type_id.raw())?.body {
        SemanticTypeNodeBody::List(inner) => Some(*inner),
        _ => None,
    }
}

fn check_handle(
    computation: &ComputationDef,
    body: &HandleBody,
    values: &BTreeMap<u32, &ValueSlot>,
    computations: &BTreeMap<u32, &ComputationDef>,
) -> Result<(), IrError> {
    let _ = values;
    let mut ordinals: BTreeSet<u32> = BTreeSet::new();
    let mut failure_matches: Vec<af_semantic::TypeSchemaRef> = Vec::new();
    let mut timed_out_clauses = 0u32;
    for clause in &body.clauses {
        if !clause.ordinal.is_valid() {
            return Err(IrError::InvalidLocalId {
                namespace: HandlerOrdinal::namespace(),
                id: clause.ordinal.raw(),
            });
        }
        if !ordinals.insert(clause.ordinal.raw()) {
            return Err(IrError::rule(
                kind::DUPLICATE_HANDLER_ORDINAL,
                format!(
                    "computation {}: duplicate handler ordinal {}",
                    computation.id, clause.ordinal
                ),
            ));
        }
        match &clause.matcher {
            HandleMatch::Failure(failure) => {
                if failure_matches.contains(&failure.failure_type) {
                    return Err(IrError::rule(
                        kind::DUPLICATE_FAILURE_MATCH,
                        format!(
                            "computation {}: two failure clauses match the exact schema {}",
                            computation.id,
                            failure.failure_type.identity()
                        ),
                    ));
                }
                failure_matches.push(failure.failure_type.clone());
            }
            HandleMatch::TimedOut(()) => {
                timed_out_clauses += 1;
                if timed_out_clauses > 1 {
                    return Err(IrError::rule(
                        kind::DUPLICATE_TIMED_OUT,
                        format!(
                            "computation {}: a handle has at most one timed_out clause",
                            computation.id
                        ),
                    ));
                }
            }
        }
    }
    let _ = computations;
    Ok(())
}

fn check_scope_resources(
    computation: &ComputationDef,
    scope_resources: &[ResourceSlotId],
    resources: &BTreeSet<u32>,
) -> Result<(), IrError> {
    let mut seen: BTreeSet<u32> = BTreeSet::new();
    for resource in scope_resources {
        if !resources.contains(&resource.raw()) {
            return Err(IrError::rule(
                kind::SCOPE_UNKNOWN_RESOURCE,
                format!("resource slot {resource} is not declared"),
            ));
        }
        if !seen.insert(resource.raw()) {
            return Err(IrError::rule(
                kind::SCOPE_DUPLICATE_RESOURCE,
                format!(
                    "computation {}: resource slot {} is listed twice",
                    computation.id, resource
                ),
            ));
        }
    }
    Ok(())
}

fn check_producer_uniqueness(
    values: &BTreeMap<u32, &ValueSlot>,
    producers: &BTreeMap<u32, &'static str>,
) -> Result<(), IrError> {
    for raw in values.keys() {
        if producers.get(raw).is_none() {
            return Err(IrError::rule(
                kind::UNBOUND_VALUE_SLOT,
                format!("value slot {raw} has no producer"),
            ));
        }
    }
    Ok(())
}

/// Ownership tree validation. Dataflow edges (expression → slot) never enter
/// here: ownership comes only from `owned_children`.
fn check_ownership(
    document: &SemanticIrDocument,
    computations: &BTreeMap<u32, &ComputationDef>,
) -> Result<(), IrError> {
    if document.root.raw() == 0 {
        return Err(IrError::InvalidLocalId {
            namespace: ComputationId::namespace(),
            id: 0,
        });
    }
    if !computations.contains_key(&document.root.raw()) {
        return Err(IrError::rule(
            kind::UNKNOWN_ROOT,
            format!("root {} is not a declared computation", document.root),
        ));
    }

    // Child existence and same-parent duplicates were validated per body; here
    // we build the parent map and the children adjacency.
    let mut parents: BTreeMap<u32, u32> = BTreeMap::new();
    let mut children_of: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for computation in computations.values() {
        for child in computation.owned_children() {
            children_of
                .entry(computation.id.raw())
                .or_default()
                .push(child.raw());
        }
    }
    for (parent, children) in &children_of {
        for child in children {
            match parents.insert(*child, *parent) {
                None => {}
                Some(previous) if previous == *parent => {}
                Some(previous) => {
                    return Err(IrError::rule(
                        kind::MULTIPLE_STRUCTURAL_PARENTS,
                        format!(
                            "computation {child} is owned by computations {previous} and {parent}"
                        ),
                    ));
                }
            }
        }
    }

    // Structural cycle: DFS from the root; entering a node that is still on
    // the stack is a cycle. (With the exactly-one-parent rule a reachable
    // cycle always involves the root or a repeated child, but the dedicated
    // walk produces the precise error.)
    let mut state: BTreeMap<u32, u8> = BTreeMap::new();
    let cycle = dfs_cycle(document.root.raw(), &children_of, &mut state);
    if cycle {
        return Err(IrError::rule(
            kind::COMPUTATION_CYCLE,
            "the ownership graph contains a directed cycle",
        ));
    }

    if parents.contains_key(&document.root.raw()) {
        return Err(IrError::rule(
            kind::ROOT_HAS_PARENT,
            format!(
                "the root computation {} has a structural parent",
                document.root
            ),
        ));
    }

    // Reachability: every computation must be owned, directly or transitively,
    // by the root. Dead detached nodes are an authoring/compiler defect and
    // are rejected, not silently dropped.
    let mut reachable: BTreeSet<u32> = BTreeSet::new();
    let mut stack = vec![document.root.raw()];
    while let Some(current) = stack.pop() {
        if reachable.insert(current) {
            if let Some(children) = children_of.get(&current) {
                stack.extend(children.iter().copied());
            }
        }
    }
    let detached = computations
        .keys()
        .copied()
        .find(|id| !reachable.contains(id));
    if let Some(id) = detached {
        return Err(IrError::rule(
            kind::UNREACHABLE_COMPUTATION,
            format!("computation {id} is not reachable from the root"),
        ));
    }
    Ok(())
}

/// `0` = unvisited, `1` = on stack, `2` = done.
fn dfs_cycle(
    current: u32,
    children_of: &BTreeMap<u32, Vec<u32>>,
    state: &mut BTreeMap<u32, u8>,
) -> bool {
    match state.get(&current) {
        Some(1) => true,
        Some(_) => false,
        None => {
            state.insert(current, 1);
            let cyclic = children_of.get(&current).is_some_and(|children| {
                children
                    .iter()
                    .any(|child| dfs_cycle(*child, children_of, state))
            });
            state.insert(current, 2);
            cyclic
        }
    }
}

/// Wrapper interface equality (Phase 3B static rules): SEQUENCE == last child,
/// SELECT == every case, RACE == every branch, RETRY/TIMEOUT/SCOPE/
/// COMPENSATION_SCOPE == child, HANDLE == protected child + every handler.
/// PARALLEL is checked by its explicit output mapping; ITERATE by the carry
/// rules; INVOKE/AWAIT/CALL_SUBFLOW are leaves whose interface only its own
/// outputs declare.
fn check_interfaces(
    document: &SemanticIrDocument,
    values: &BTreeMap<u32, &ValueSlot>,
    computations: &BTreeMap<u32, &ComputationDef>,
) -> Result<(), IrError> {
    let checked = CheckedIrDocument { source: document };
    for computation in computations.values() {
        let parent = checked.interface(computation.id)?;
        let equal = |child: ComputationId| -> Result<bool, IrError> {
            Ok(checked.interface(child)? == parent)
        };
        match &computation.body {
            ComputationBody::Sequence(body) => {
                let last = *body.children.last().expect("non-empty validated");
                if !equal(last)? {
                    return Err(IrError::rule(
                        kind::SEQUENCE_INTERFACE_MISMATCH,
                        format!(
                            "computation {}: the output interface must equal the last child's ({last})",
                            computation.id
                        ),
                    ));
                }
            }
            ComputationBody::Select(body) => {
                for case in &body.cases {
                    if !equal(case.child)? {
                        return Err(IrError::rule(
                            kind::SELECT_INTERFACE_MISMATCH,
                            format!(
                                "computation {}: case {} child {} has a different output interface",
                                computation.id, case.ordinal, case.child
                            ),
                        ));
                    }
                }
            }
            ComputationBody::Parallel(body) => {
                // The explicit mapping was fully checked in pass A.
                let _ = body;
            }
            ComputationBody::Race(body) => {
                for branch in &body.branches {
                    if !equal(branch.child)? {
                        return Err(IrError::rule(
                            kind::RACE_INTERFACE_MISMATCH,
                            format!(
                                "computation {}: branch {} child {} has a different output interface",
                                computation.id, branch.ordinal, branch.child
                            ),
                        ));
                    }
                }
            }
            ComputationBody::Iterate(body) => {
                // Carry typing is the iterate's static interface rule; the
                // loop body keeps its own interface.
                let _ = body;
            }
            ComputationBody::Retry(body) => {
                if !equal(body.child)? {
                    return Err(IrError::rule(
                        kind::RETRY_INTERFACE_MISMATCH,
                        format!(
                            "computation {}: the output interface must equal the child's ({})",
                            computation.id, body.child
                        ),
                    ));
                }
            }
            ComputationBody::Timeout(body) => {
                if !equal(body.child)? {
                    return Err(IrError::rule(
                        kind::TIMEOUT_INTERFACE_MISMATCH,
                        format!(
                            "computation {}: the output interface must equal the child's ({})",
                            computation.id, body.child
                        ),
                    ));
                }
            }
            ComputationBody::Handle(body) => {
                let mut handlers: Vec<ComputationId> =
                    body.clauses.iter().map(|clause| clause.handler).collect();
                handlers.push(body.child);
                for owned in handlers {
                    if !equal(owned)? {
                        return Err(IrError::rule(
                            kind::HANDLE_INTERFACE_MISMATCH,
                            format!(
                                "computation {}: computation {owned} has a different output interface",
                                computation.id
                            ),
                        ));
                    }
                }
            }
            ComputationBody::Scope(body) => {
                if !equal(body.child)? {
                    return Err(IrError::rule(
                        kind::SCOPE_INTERFACE_MISMATCH,
                        format!(
                            "computation {}: the output interface must equal the child's ({})",
                            computation.id, body.child
                        ),
                    ));
                }
            }
            ComputationBody::CompensationScope(body) => {
                if !equal(body.child)? {
                    return Err(IrError::rule(
                        kind::COMPENSATION_SCOPE_INTERFACE_MISMATCH,
                        format!(
                            "computation {}: the output interface must equal the child's ({})",
                            computation.id, body.child
                        ),
                    ));
                }
            }
            ComputationBody::Invoke(_)
            | ComputationBody::Await(_)
            | ComputationBody::CallSubflow(_) => {}
        }
        let _ = values;
    }
    Ok(())
}

/// The `Await`/`Handle` schema references must resolve exactly against the
/// supplied universe (never "latest"). Split out so the document check can
/// run it right after the core tables are validated.
pub(crate) fn check_schema_refs(
    document: &SemanticIrDocument,
    universe: &TypeSchemaUniverse,
) -> Result<(), IrError> {
    for computation in &document.computations {
        match &computation.body {
            ComputationBody::Await(body) => {
                universe.resolve(&body.signal_type)?;
            }
            ComputationBody::Handle(body) => {
                for clause in &body.clauses {
                    if let HandleMatch::Failure(failure) = &clause.matcher {
                        universe.resolve(&failure.failure_type)?;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}
