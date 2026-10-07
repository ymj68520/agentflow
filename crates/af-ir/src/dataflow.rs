//! Phase 3C scoped typed dataflow: value availability, producer dominance,
//! branch isolation and resource lexical visibility.
//!
//! # What this pass is
//!
//! Phase 3B guarantees that every declared [`af_semantic::ValueSlot`] exists
//! and has exactly one producer. Existence is not availability: a use site may
//! only read a slot whose producer **dominates** it — i.e. whose produced
//! value is in the use site's structured environment. This module implements
//! that rule as a deterministic walk over the ownership tree with one
//! opcode-specific availability transfer function per control form.
//!
//! The pass is **derived-only**. It introduces no AFIDL type, no wire object,
//! no new HashV1 domain and no ABI row, and it never enters the
//! `SemanticIrDigest`: the digest hashes the canonical document exactly as in
//! Phase 3B. [`SemanticDataflowAnalysis`] is a plain Rust projection (analysis
//! metadata), canonically ordered only for deterministic testing.
//!
//! # Pass ordering
//!
//! `analyze_dataflow` is a separate pass over an already-validated document:
//!
//! ```text
//! SemanticIrDocument::check          (Phase 3B structural/type rules)
//!         ↓ CheckedIrDocument
//! CheckedIrDocument::analyze_dataflow   (Phase 3C availability analysis)
//! ```
//!
//! Because Phase 3B violations never reach this pass, the frozen Phase 3B
//! negative corpus keeps its exact expected error kinds. A value or resource
//! that is structurally fine but unavailable fails HERE, with the stable
//! kinds `value_not_available` and `resource_not_visible`.
//!
//! # Environments
//!
//! - `EntryValues(root)` = every run-parameter value slot — run parameters are
//!   immutable workflow inputs, live for the whole run and visible in every
//!   descendant computation.
//! - `Exports(C)` = exactly `C.outputs[].value_slot` — only a computation's
//!   own declared outputs ever cross its ownership boundary. Descendant
//!   outputs, branch locals, iterate item/carry slots and loop-body outputs
//!   never escape their owner.
//! - Per opcode (all deterministic; children of sequence in list order,
//!   everything else in ascending ordinal order):
//!   SEQUENCE `EntryValues(Ci) = E ∪ Exports(C1..Ci-1)` (the only multi-child
//!   dataflow order); SELECT/PARALLEL/RACE every branch `= E`; RETRY/TIMEOUT/
//!   SCOPE/COMPENSATION_SCOPE child `= E`; HANDLE protected child and every
//!   handler independently `= E` (a handler must not assume the protected
//!   child's success outputs exist); FOREACH body `= E ∪ {item} ∪ carries`
//!   while `source` and every carry `initial` see only the pre-loop `E`;
//!   WHILE guard and body `= E ∪ carries`.
//! - `EntryResources(root)` = every declared resource slot; non-SCOPE
//!   children inherit the parent environment; a SCOPE's child sees exactly
//!   `ScopeBody.resources` (an attenuation — a subset of the inherited set,
//!   never a union), and the environment is restored after the scope exits.
//!
//! # Use sites
//!
//! Availability is checked **per use site**, never cached per expression: the
//! same `ExpressionDef` may be legal inside its owning branch and illegal
//! outside it. The use-site set is enumerated explicitly
//! ([`ExpressionUseSite`]) — invoke/call_subflow inputs, await correlation,
//! select selector, foreach source, carry initials, while guard, timeout
//! duration and workflow outputs — because each has its own evaluation
//! environment. An expression that is never used at any site is never
//! executed and never fails availability.

use crate::check::CheckedIrDocument;
use crate::error::{IrError, kind};
use crate::ids::ComputationId;
use crate::model::{ComputationBody, IterateBody, SemanticIrDocument};
use af_semantic::{ExprTable, ExpressionId};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Where a value slot's single producer lives. Analysis metadata only — never
/// a wire object, never hashed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProducerLocation {
    /// A run-parameter declaration (visible everywhere, for the whole run).
    RunParameter { parameter: u32 },
    /// One output port of one computation.
    ComputationOutput { computation: u32, port_no: u32 },
    /// The foreach item slot of one ITERATE.
    IterateItem { iterate: u32 },
    /// A loop-carry slot of one ITERATE.
    IterateCarry { iterate: u32 },
}

impl fmt::Display for ProducerLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProducerLocation::RunParameter { parameter } => {
                write!(f, "run_parameter {parameter}")
            }
            ProducerLocation::ComputationOutput {
                computation,
                port_no,
            } => write!(
                f,
                "computation_output computation#{computation} port#{port_no}"
            ),
            ProducerLocation::IterateItem { iterate } => {
                write!(f, "iterate_item iterate#{iterate}")
            }
            ProducerLocation::IterateCarry { iterate } => {
                write!(f, "iterate_carry iterate#{iterate}")
            }
        }
    }
}

/// Why a value slot is not available at a use site. Diagnostic detail only —
/// the stable error kind is always `value_not_available`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvailabilityReason {
    /// The producer has not completed at the use site (own output at own
    /// entry, a future sequence sibling, a parent output inside its child).
    BeforeProducer,
    /// Producer and use live under different branches of the same isolated
    /// wrapper (SELECT case, PARALLEL/RACE branch, HANDLE clause).
    CrossBranch,
    /// The raw output of some non-direct descendant is consumed across an
    /// ownership boundary that does not export it.
    OutsideOwnerBoundary,
    /// A foreach item or iterate carry slot is used outside its loop region.
    IterateLocalOutside,
    /// An ancestor computation's output is used inside its own subtree.
    ParentOutputNotYetAvailable,
}

impl AvailabilityReason {
    /// The stable diagnostic token frozen by the dataflow corpus.
    pub fn as_str(self) -> &'static str {
        match self {
            AvailabilityReason::BeforeProducer => "before_producer",
            AvailabilityReason::CrossBranch => "cross_branch",
            AvailabilityReason::OutsideOwnerBoundary => "outside_owner_boundary",
            AvailabilityReason::IterateLocalOutside => "iterate_local_outside",
            AvailabilityReason::ParentOutputNotYetAvailable => "parent_output_not_yet_available",
        }
    }
}

impl fmt::Display for AvailabilityReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The explicitly enumerated expression use sites. Each variant is a syntactic
/// position with its own evaluation environment; the analysis walks these
/// positions directly instead of reflecting over `ExpressionId` fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpressionUseSite {
    InvokeInput { computation: u32, port_no: u32 },
    CallSubflowInput { computation: u32, port_no: u32 },
    AwaitCorrelation { computation: u32 },
    SelectSelector { computation: u32 },
    ForeachSource { computation: u32 },
    IterateCarryInitial { computation: u32, slot: u32 },
    WhileGuard { computation: u32 },
    TimeoutDuration { computation: u32 },
    WorkflowOutput { name: String },
}

impl fmt::Display for ExpressionUseSite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExpressionUseSite::InvokeInput {
                computation,
                port_no,
            } => write!(f, "invoke input (computation#{computation} port#{port_no})"),
            ExpressionUseSite::CallSubflowInput {
                computation,
                port_no,
            } => write!(
                f,
                "call_subflow input (computation#{computation} port#{port_no})"
            ),
            ExpressionUseSite::AwaitCorrelation { computation } => {
                write!(f, "await correlation (computation#{computation})")
            }
            ExpressionUseSite::SelectSelector { computation } => {
                write!(f, "select selector (computation#{computation})")
            }
            ExpressionUseSite::ForeachSource { computation } => {
                write!(f, "foreach source (computation#{computation})")
            }
            ExpressionUseSite::IterateCarryInitial { computation, slot } => write!(
                f,
                "iterate carry initial (computation#{computation} slot#{slot})"
            ),
            ExpressionUseSite::WhileGuard { computation } => {
                write!(f, "while guard (computation#{computation})")
            }
            ExpressionUseSite::TimeoutDuration { computation } => {
                write!(f, "timeout duration (computation#{computation})")
            }
            ExpressionUseSite::WorkflowOutput { name } => {
                write!(f, "workflow output {name:?}")
            }
        }
    }
}

/// The derived-only Phase 3C analysis artifact. It is a plain Rust projection
/// of the checked document: it is never ACE1-encoded, never hashed into the
/// `SemanticIrDigest` and never enters AFIDL, the ABI registry or the wire
/// model. Maps are keyed by raw local ids in ascending order (a conformance
/// projection ordering, not a hashing rule).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticDataflowAnalysis {
    entry_values: BTreeMap<u32, BTreeSet<u32>>,
    entry_resources: BTreeMap<u32, BTreeSet<u32>>,
    expression_value_dependencies: BTreeMap<u32, BTreeSet<u32>>,
    producer_locations: BTreeMap<u32, ProducerLocation>,
    workflow_exit_values: BTreeSet<u32>,
}

impl SemanticDataflowAnalysis {
    /// `EntryValues(C)` for every computation, keyed by raw `ComputationId`.
    pub fn entry_values(&self) -> &BTreeMap<u32, BTreeSet<u32>> {
        &self.entry_values
    }

    /// `EntryResources(C)` for every computation, keyed by raw `ComputationId`.
    pub fn entry_resources(&self) -> &BTreeMap<u32, BTreeSet<u32>> {
        &self.entry_resources
    }

    /// The transitive value-slot dependency closure of every expression,
    /// keyed by raw `ExpressionId`.
    pub fn expression_value_dependencies(&self) -> &BTreeMap<u32, BTreeSet<u32>> {
        &self.expression_value_dependencies
    }

    /// The single producer of every declared value slot, keyed by raw
    /// `ValueSlotId`.
    pub fn producer_locations(&self) -> &BTreeMap<u32, ProducerLocation> {
        &self.producer_locations
    }

    /// `WorkflowExitValues = run parameter slots ∪ Exports(root)`: the only
    /// values workflow-output expressions may read.
    pub fn workflow_exit_values(&self) -> &BTreeSet<u32> {
        &self.workflow_exit_values
    }
}

impl CheckedIrDocument<'_> {
    /// Runs the Phase 3C scoped dataflow analysis over this validated
    /// document: per-use-site value availability, producer dominance, branch
    /// isolation and resource lexical visibility. On success returns the
    /// derived [`SemanticDataflowAnalysis`].
    ///
    /// # Errors
    /// [`IrError`] with kind `value_not_available` or `resource_not_visible`.
    /// Phase 3B violations never reach this pass (validate first with
    /// [`SemanticIrDocument::check`]).
    pub fn analyze_dataflow(&self) -> Result<SemanticDataflowAnalysis, IrError> {
        analyze(self.source())
    }
}

/// Walk state: everything is derived from the checked document, nothing is
/// cached across analyses.
struct Walk<'a> {
    computations: BTreeMap<u32, &'a crate::model::ComputationDef>,
    parents: BTreeMap<u32, u32>,
    dependencies: BTreeMap<u32, BTreeSet<u32>>,
    producers: BTreeMap<u32, ProducerLocation>,
}

fn analyze(document: &SemanticIrDocument) -> Result<SemanticDataflowAnalysis, IrError> {
    let computations: BTreeMap<u32, &crate::model::ComputationDef> = document
        .computations
        .iter()
        .map(|computation| (computation.id.raw(), computation))
        .collect();
    let mut parents: BTreeMap<u32, u32> = BTreeMap::new();
    for computation in computations.values() {
        for child in computation.owned_children() {
            parents.insert(child.raw(), computation.id.raw());
        }
    }

    // Expression dependency closure. The graph is a DAG (Phase 3A checked),
    // so the recursion is bounded and memoizable — no second cycle semantics.
    let expression_defs: BTreeMap<u32, &af_semantic::ExpressionDef> = document
        .expressions
        .iter()
        .map(|def| (def.id.raw(), def))
        .collect();
    let mut dependencies: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
    for id in expression_defs.keys() {
        dependency_closure(*id, &expression_defs, &mut dependencies);
    }

    // Producer locations (uniqueness was validated in Phase 3B).
    let mut producers: BTreeMap<u32, ProducerLocation> = BTreeMap::new();
    for parameter in &document.run_parameters {
        producers.insert(
            parameter.value_slot.raw(),
            ProducerLocation::RunParameter {
                parameter: parameter.id.raw(),
            },
        );
    }
    for computation in computations.values() {
        for output in &computation.outputs {
            producers.insert(
                output.value_slot.raw(),
                ProducerLocation::ComputationOutput {
                    computation: computation.id.raw(),
                    port_no: output.port_no.raw(),
                },
            );
        }
        if let ComputationBody::Iterate(iterate) = &computation.body {
            match iterate {
                IterateBody::Foreach(foreach) => {
                    producers.insert(
                        foreach.item_slot.raw(),
                        ProducerLocation::IterateItem {
                            iterate: computation.id.raw(),
                        },
                    );
                    for carry in &foreach.carry {
                        producers.insert(
                            carry.slot.raw(),
                            ProducerLocation::IterateCarry {
                                iterate: computation.id.raw(),
                            },
                        );
                    }
                }
                IterateBody::While(while_body) => {
                    for carry in &while_body.carry {
                        producers.insert(
                            carry.slot.raw(),
                            ProducerLocation::IterateCarry {
                                iterate: computation.id.raw(),
                            },
                        );
                    }
                }
            }
        }
    }

    let walk = Walk {
        computations,
        parents,
        dependencies,
        producers,
    };

    let mut analysis = SemanticDataflowAnalysis {
        entry_values: BTreeMap::new(),
        entry_resources: BTreeMap::new(),
        expression_value_dependencies: walk.dependencies.clone(),
        producer_locations: walk.producers.clone(),
        workflow_exit_values: BTreeSet::new(),
    };

    // Root entry environments: every run parameter, every declared resource.
    let root_values: BTreeSet<u32> = document
        .run_parameters
        .iter()
        .map(|parameter| parameter.value_slot.raw())
        .collect();
    let root_resources: BTreeSet<u32> = document
        .resources
        .iter()
        .map(|slot| slot.id.raw())
        .collect();
    visit(
        &walk,
        &mut analysis,
        document.root.raw(),
        &root_values,
        &root_resources,
    )?;

    // Workflow outputs are evaluated after the root's terminal boundary: the
    // only visible values are the run parameters and the root's own outputs.
    let mut exit_values = root_values;
    for output in exports_of(&walk, document.root.raw()) {
        exit_values.insert(output);
    }
    let mut outputs: Vec<&crate::model::WorkflowOutputDecl> =
        document.workflow_outputs.iter().collect();
    outputs.sort_by_key(|output| output.name.as_bytes());
    for output in outputs {
        check_value_use(
            &walk,
            output.expression,
            &exit_values,
            None,
            &ExpressionUseSite::WorkflowOutput {
                name: output.name.clone(),
            },
        )?;
    }
    analysis.workflow_exit_values = exit_values;
    Ok(analysis)
}

/// The transitive value-slot closure of one expression, memoized. Edges come
/// from [`ExprTable::references`] — the same Phase 3A edge set, never a
/// second graph semantics.
fn dependency_closure(
    id: u32,
    expressions: &BTreeMap<u32, &af_semantic::ExpressionDef>,
    memo: &mut BTreeMap<u32, BTreeSet<u32>>,
) -> BTreeSet<u32> {
    if let Some(closure) = memo.get(&id) {
        return closure.clone();
    }
    let mut closure = BTreeSet::new();
    if let Some(def) = expressions.get(&id) {
        match &def.body {
            af_semantic::ExpressionBody::ValueSlot(slot) => {
                closure.insert(slot.raw());
            }
            body => {
                for reference in ExprTable::references(body) {
                    closure.extend(dependency_closure(reference.raw(), expressions, memo));
                }
            }
        }
    }
    memo.insert(id, closure.clone());
    closure
}

/// `Exports(C)` — exactly the computation's own output slots.
fn exports_of<'a>(walk: &'a Walk<'a>, id: u32) -> impl Iterator<Item = u32> + 'a {
    walk.computations[&id]
        .outputs
        .iter()
        .map(|output| output.value_slot.raw())
}

fn visit(
    walk: &Walk<'_>,
    analysis: &mut SemanticDataflowAnalysis,
    current: u32,
    entry_values: &BTreeSet<u32>,
    entry_resources: &BTreeSet<u32>,
) -> Result<(), IrError> {
    analysis.entry_values.insert(current, entry_values.clone());
    analysis
        .entry_resources
        .insert(current, entry_resources.clone());
    let computation = walk.computations[&current];
    let id = ComputationId::from_raw(current);

    match &computation.body {
        ComputationBody::Invoke(body) => {
            let mut inputs: Vec<_> = body.inputs.iter().collect();
            inputs.sort_by_key(|binding| binding.port_no);
            for input in inputs {
                check_value_use(
                    walk,
                    input.expression,
                    entry_values,
                    Some(current),
                    &ExpressionUseSite::InvokeInput {
                        computation: current,
                        port_no: input.port_no.raw(),
                    },
                )?;
            }
            check_resource_uses(&body.resources, entry_resources, "invoke", id)?;
        }
        ComputationBody::CallSubflow(body) => {
            let mut inputs: Vec<_> = body.inputs.iter().collect();
            inputs.sort_by_key(|binding| binding.port_no);
            for input in inputs {
                check_value_use(
                    walk,
                    input.expression,
                    entry_values,
                    Some(current),
                    &ExpressionUseSite::CallSubflowInput {
                        computation: current,
                        port_no: input.port_no.raw(),
                    },
                )?;
            }
            check_resource_uses(&body.resources, entry_resources, "call_subflow", id)?;
        }
        ComputationBody::Await(body) => {
            check_value_use(
                walk,
                body.correlation,
                entry_values,
                Some(current),
                &ExpressionUseSite::AwaitCorrelation {
                    computation: current,
                },
            )?;
        }
        ComputationBody::Sequence(body) => {
            // The one multi-child dataflow order: each child additionally
            // sees every earlier direct sibling's own exports.
            let mut env = entry_values.clone();
            for child in &body.children {
                visit(walk, analysis, child.raw(), &env, entry_resources)?;
                for slot in exports_of(walk, child.raw()) {
                    env.insert(slot);
                }
            }
        }
        ComputationBody::Select(body) => {
            check_value_use(
                walk,
                body.selector,
                entry_values,
                Some(current),
                &ExpressionUseSite::SelectSelector {
                    computation: current,
                },
            )?;
            let mut cases: Vec<_> = body.cases.iter().collect();
            cases.sort_by_key(|case| case.ordinal);
            for case in cases {
                visit(
                    walk,
                    analysis,
                    case.child.raw(),
                    entry_values,
                    entry_resources,
                )?;
            }
        }
        ComputationBody::Parallel(body) => {
            let mut branches: Vec<_> = body.branches.iter().collect();
            branches.sort_by_key(|branch| branch.ordinal);
            for branch in branches {
                visit(
                    walk,
                    analysis,
                    branch.child.raw(),
                    entry_values,
                    entry_resources,
                )?;
            }
        }
        ComputationBody::Race(body) => {
            let mut branches: Vec<_> = body.branches.iter().collect();
            branches.sort_by_key(|branch| branch.ordinal);
            for branch in branches {
                visit(
                    walk,
                    analysis,
                    branch.child.raw(),
                    entry_values,
                    entry_resources,
                )?;
            }
        }
        ComputationBody::Iterate(IterateBody::Foreach(foreach)) => {
            // Source and carry initials evaluate from the same pre-loop
            // environment: loop locals do not exist yet.
            check_value_use(
                walk,
                foreach.source,
                entry_values,
                Some(current),
                &ExpressionUseSite::ForeachSource {
                    computation: current,
                },
            )?;
            let mut carry: Vec<_> = foreach.carry.iter().collect();
            carry.sort_by_key(|binding| binding.slot);
            for binding in &carry {
                check_value_use(
                    walk,
                    binding.initial,
                    entry_values,
                    Some(current),
                    &ExpressionUseSite::IterateCarryInitial {
                        computation: current,
                        slot: binding.slot.raw(),
                    },
                )?;
            }
            let mut body_env = entry_values.clone();
            body_env.insert(foreach.item_slot.raw());
            for binding in &carry {
                body_env.insert(binding.slot.raw());
            }
            visit(
                walk,
                analysis,
                foreach.body.raw(),
                &body_env,
                entry_resources,
            )?;
        }
        ComputationBody::Iterate(IterateBody::While(while_body)) => {
            let mut carry: Vec<_> = while_body.carry.iter().collect();
            carry.sort_by_key(|binding| binding.slot);
            for binding in &carry {
                check_value_use(
                    walk,
                    binding.initial,
                    entry_values,
                    Some(current),
                    &ExpressionUseSite::IterateCarryInitial {
                        computation: current,
                        slot: binding.slot.raw(),
                    },
                )?;
            }
            let mut env = entry_values.clone();
            for binding in &carry {
                env.insert(binding.slot.raw());
            }
            // The guard evaluates at the loop header every iteration: it sees
            // the carry slots but never a body computation output.
            check_value_use(
                walk,
                while_body.guard,
                &env,
                Some(current),
                &ExpressionUseSite::WhileGuard {
                    computation: current,
                },
            )?;
            visit(walk, analysis, while_body.body.raw(), &env, entry_resources)?;
        }
        ComputationBody::Retry(body) => {
            visit(
                walk,
                analysis,
                body.child.raw(),
                entry_values,
                entry_resources,
            )?;
        }
        ComputationBody::Timeout(body) => {
            check_value_use(
                walk,
                body.timeout,
                entry_values,
                Some(current),
                &ExpressionUseSite::TimeoutDuration {
                    computation: current,
                },
            )?;
            visit(
                walk,
                analysis,
                body.child.raw(),
                entry_values,
                entry_resources,
            )?;
        }
        ComputationBody::Handle(body) => {
            visit(
                walk,
                analysis,
                body.child.raw(),
                entry_values,
                entry_resources,
            )?;
            let mut clauses: Vec<_> = body.clauses.iter().collect();
            clauses.sort_by_key(|clause| clause.ordinal);
            for clause in clauses {
                visit(
                    walk,
                    analysis,
                    clause.handler.raw(),
                    entry_values,
                    entry_resources,
                )?;
            }
        }
        ComputationBody::Scope(body) => {
            // Attenuation: the child's resource environment is exactly the
            // listed set, which must be a subset of the inherited one.
            let mut resources: BTreeSet<u32> = BTreeSet::new();
            for resource in &body.resources {
                if !entry_resources.contains(&resource.raw()) {
                    return Err(IrError::rule(
                        kind::RESOURCE_NOT_VISIBLE,
                        format!(
                            "computation {id}: scope declares resource slot {resource}, which \
                             is not visible in its inherited resource environment",
                        ),
                    ));
                }
                resources.insert(resource.raw());
            }
            visit(walk, analysis, body.child.raw(), entry_values, &resources)?;
        }
        ComputationBody::CompensationScope(body) => {
            visit(
                walk,
                analysis,
                body.child.raw(),
                entry_values,
                entry_resources,
            )?;
        }
    }
    Ok(())
}

fn check_resource_uses(
    uses: &[crate::model::ResourceUseBinding],
    entry_resources: &BTreeSet<u32>,
    opcode: &str,
    id: ComputationId,
) -> Result<(), IrError> {
    let mut sorted: Vec<_> = uses.iter().collect();
    sorted.sort_by_key(|binding| binding.port_no);
    for binding in sorted {
        if !entry_resources.contains(&binding.resource_slot.raw()) {
            return Err(IrError::rule(
                kind::RESOURCE_NOT_VISIBLE,
                format!(
                    "computation {}: {} resource port {} names resource slot {}, which is not \
                     lexically visible here",
                    id, opcode, binding.port_no, binding.resource_slot
                ),
            ));
        }
    }
    Ok(())
}

/// Checks one expression use site: every transitively referenced value slot
/// must be in the site's environment. Slots are reported in ascending order,
/// so the first reported violation is deterministic.
fn check_value_use(
    walk: &Walk<'_>,
    expression: ExpressionId,
    env: &BTreeSet<u32>,
    current: Option<u32>,
    site: &ExpressionUseSite,
) -> Result<(), IrError> {
    for slot in walk
        .dependencies
        .get(&expression.raw())
        .expect("use-site expressions exist after Phase 3B validation")
    {
        if !env.contains(slot) {
            let producer = walk
                .producers
                .get(slot)
                .expect("producer uniqueness validated before the analysis");
            let reason = classify(walk, producer, current);
            return Err(IrError::rule(
                kind::VALUE_NOT_AVAILABLE,
                format!(
                    "value slot {slot} is not available at {site}: produced by {producer}; \
                     reason: {reason}"
                ),
            ));
        }
    }
    Ok(())
}

fn classify(
    walk: &Walk<'_>,
    producer: &ProducerLocation,
    current: Option<u32>,
) -> AvailabilityReason {
    let Some(use_id) = current else {
        // A workflow output sees only run parameters and root exports; a
        // missing loop local escapes its iterate, everything else escapes
        // some owner boundary.
        return match producer {
            ProducerLocation::IterateItem { .. } | ProducerLocation::IterateCarry { .. } => {
                AvailabilityReason::IterateLocalOutside
            }
            _ => AvailabilityReason::OutsideOwnerBoundary,
        };
    };
    match producer {
        // Run parameters are in every environment; this is unreachable for a
        // validated document and classified defensively.
        ProducerLocation::RunParameter { .. } => AvailabilityReason::BeforeProducer,
        ProducerLocation::IterateItem { .. } | ProducerLocation::IterateCarry { .. } => {
            AvailabilityReason::IterateLocalOutside
        }
        ProducerLocation::ComputationOutput {
            computation: producer_id,
            ..
        } => {
            if producer_id == &use_id {
                return AvailabilityReason::BeforeProducer;
            }
            if is_ancestor(walk, *producer_id, use_id) {
                return AvailabilityReason::ParentOutputNotYetAvailable;
            }
            if is_ancestor(walk, use_id, *producer_id) {
                return AvailabilityReason::BeforeProducer;
            }
            let Some(lowest) = lowest_common_ancestor(walk, use_id, *producer_id) else {
                return AvailabilityReason::OutsideOwnerBoundary;
            };
            let use_side = child_under(walk, lowest, use_id);
            let producer_side = child_under(walk, lowest, *producer_id);
            match (use_side, producer_side) {
                (Some(use_side), Some(producer_side)) => {
                    if use_side == producer_side {
                        AvailabilityReason::OutsideOwnerBoundary
                    } else if is_isolated_branch_wrapper(walk, lowest) {
                        AvailabilityReason::CrossBranch
                    } else if is_later_sequence_child(walk, lowest, use_side, producer_side) {
                        AvailabilityReason::BeforeProducer
                    } else {
                        AvailabilityReason::OutsideOwnerBoundary
                    }
                }
                _ => AvailabilityReason::OutsideOwnerBoundary,
            }
        }
    }
}

/// `true` when `ancestor` is a strict structural ancestor of `node`.
fn is_ancestor(walk: &Walk<'_>, ancestor: u32, node: u32) -> bool {
    let mut current = node;
    while let Some(parent) = walk.parents.get(&current) {
        if *parent == ancestor {
            return true;
        }
        current = *parent;
    }
    false
}

/// The lowest computation that structurally contains both nodes (possibly one
/// of them, never a leaf with two distinct sides).
fn lowest_common_ancestor(walk: &Walk<'_>, a: u32, b: u32) -> Option<u32> {
    let mut ancestors: BTreeSet<u32> = BTreeSet::new();
    let mut current = Some(a);
    while let Some(node) = current {
        ancestors.insert(node);
        current = walk.parents.get(&node).copied();
    }
    let mut current = Some(b);
    while let Some(node) = current {
        if ancestors.contains(&node) {
            return Some(node);
        }
        current = walk.parents.get(&node).copied();
    }
    None
}

/// The direct child of `root` on the path down to `node` (`node` itself when
/// it is a direct child).
fn child_under(walk: &Walk<'_>, root: u32, node: u32) -> Option<u32> {
    let mut current = node;
    while let Some(parent) = walk.parents.get(&current) {
        if *parent == root {
            return Some(current);
        }
        current = *parent;
    }
    None
}

/// SELECT cases, PARALLEL branches, RACE branches and HANDLE clauses are the
/// isolated multi-child wrappers: two different children never see each
/// other's values.
fn is_isolated_branch_wrapper(walk: &Walk<'_>, id: u32) -> bool {
    matches!(
        &walk.computations[&id].body,
        ComputationBody::Select(_)
            | ComputationBody::Parallel(_)
            | ComputationBody::Race(_)
            | ComputationBody::Handle(_)
    )
}

/// `true` when `producer_side` is a **direct** child of a SEQUENCE that comes
/// after `use_side` — a plain use-before-producer (the value would exist only
/// later in the sequence's execution order).
fn is_later_sequence_child(
    walk: &Walk<'_>,
    sequence: u32,
    use_side: u32,
    producer_side: u32,
) -> bool {
    let ComputationBody::Sequence(body) = &walk.computations[&sequence].body else {
        return false;
    };
    // Only a direct child's own output is "produced later"; a grandchild's
    // output inside an earlier sibling is an owner-boundary escape instead.
    if body
        .children
        .iter()
        .any(|child| child.raw() == producer_side)
    {
        let use_position = body
            .children
            .iter()
            .position(|child| child.raw() == use_side);
        let producer_position = body
            .children
            .iter()
            .position(|child| child.raw() == producer_side);
        match (use_position, producer_position) {
            (Some(use_position), Some(producer_position)) => producer_position > use_position,
            _ => false,
        }
    } else {
        false
    }
}
