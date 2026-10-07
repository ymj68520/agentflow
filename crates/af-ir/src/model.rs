//! The typed semantic IR model (source form).
//!
//! Field names mirror `spec/idl/semantic-ir.afidl` exactly — the serde JSON
//! form is the corpus transport, and [`SemanticIrDocument::to_value`] (in
//! [`crate::canonical`]) lowers the *canonical* form to the ACE1 value tree.
//! Ordering of the tables as stored is source order; canonical ordering is a
//! pure function ([`SemanticIrDocument::canonical`]) and is what the digest
//! hashes.
//!
//! The model is deliberately runtime-free: no occurrence identity, no journal,
//! no reducer, no runtime command, no worker, no scheduler, no timer state,
//! no resource handle.

use crate::ids::{
    BranchOrdinal, ComputationId, ContractRefId, HandlerOrdinal, PortNo, RunParameterId,
};
use af_core::Duration;
use af_semantic::{
    DigestRefValue, ExpressionDef, ExpressionId, ResourceSlot, ResourceSlotId, SemanticTypeId,
    SemanticTypeNode, TypeSchemaRef, ValueSlot, ValueSlotId,
};
use serde::Deserialize;
use serde::de::Deserializer;

// ---------------------------------------------------------------------------
// Document header tables
// ---------------------------------------------------------------------------

/// Pins a semantic/profile identity by its exact `SpecificationId`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticVersionRef {
    pub specification_id: String,
}

/// A static workflow input declaration that *produces* its value slot.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunParameterDecl {
    pub id: RunParameterId,
    pub name: String,
    /// The parameter's semantic type. Renamed from the AFIDL field name
    /// `type` because `type` is a Rust keyword.
    #[serde(rename = "type")]
    pub type_id: SemanticTypeId,
    pub value_slot: ValueSlotId,
}

/// A workflow output: a name plus the expression computing the value. The
/// output type is the expression's `result_type` — never stored twice.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowOutputDecl {
    pub name: String,
    pub expression: ExpressionId,
}

/// The closed contract reference kinds. Only the two kinds whose contract
/// models are frozen; policy/effect/resource/signal/worker tables are later
/// stages and are structurally inexpressible here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractRefKind {
    Component,
    Subflow,
}

impl ContractRefKind {
    /// The AFIDL enum member name (the ACE1 value encoding).
    pub fn keyword(self) -> &'static str {
        match self {
            ContractRefKind::Component => "component",
            ContractRefKind::Subflow => "subflow",
        }
    }

    /// The HashV1 domain this kind's digest must live in.
    pub fn digest_domain(self) -> &'static str {
        match self {
            ContractRefKind::Component => "component-contract",
            ContractRefKind::Subflow => "semantic-ir",
        }
    }
}

/// One entry of the closed contract table.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractRefEntry {
    pub id: ContractRefId,
    pub kind: ContractRefKind,
    pub digest: DigestRefValue,
}

// ---------------------------------------------------------------------------
// Ports and bindings
// ---------------------------------------------------------------------------

/// One output port of a computation; its type is the bound slot's type.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComputationOutputBinding {
    pub port_no: PortNo,
    pub value_slot: ValueSlotId,
}

/// One semantic input of INVOKE / CALL_SUBFLOW fed by a pure expression.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputBinding {
    pub port_no: PortNo,
    pub expression: ExpressionId,
}

/// One logical resource requirement of INVOKE / CALL_SUBFLOW.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceUseBinding {
    pub port_no: PortNo,
    pub resource_slot: ResourceSlotId,
}

// ---------------------------------------------------------------------------
// Computation bodies (the closed 13)
// ---------------------------------------------------------------------------

/// INVOKE: call an exact component contract. Zero local children.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvokeBody {
    pub contract: ContractRefId,
    pub inputs: Vec<InputBinding>,
    pub resources: Vec<ResourceUseBinding>,
}

/// AWAIT: a passive durable wait. Zero local children; no built-in timeout
/// (compose TIMEOUT(AWAIT(...))).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AwaitBody {
    pub signal_type: TypeSchemaRef,
    pub correlation: ExpressionId,
}

/// SEQUENCE: sequential composition; the list order IS the semantic
/// execution order and is never canonically re-sorted.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SequenceBody {
    pub children: Vec<ComputationId>,
}

/// One exact-ordinal dispatch case.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectCase {
    pub ordinal: BranchOrdinal,
    pub child: ComputationId,
}

/// SELECT: the selector (type int64) produces the exact case ordinal.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectBody {
    pub selector: ExpressionId,
    pub cases: Vec<SelectCase>,
}

/// The closed v1 parallel completion mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParallelMode {
    All,
    FailFast,
}

impl ParallelMode {
    /// The AFIDL enum member name (the ACE1 value encoding).
    pub fn keyword(self) -> &'static str {
        match self {
            ParallelMode::All => "all",
            ParallelMode::FailFast => "fail_fast",
        }
    }
}

/// One parallel branch.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParallelBranch {
    pub ordinal: BranchOrdinal,
    pub child: ComputationId,
}

/// Explicit merge mapping: parent port takes one branch child's output port.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParallelOutputBinding {
    pub parent_port_no: PortNo,
    pub branch_ordinal: BranchOrdinal,
    pub child_port_no: PortNo,
}

/// PARALLEL: branches under a completion mode with a semantic concurrency
/// bound and an explicit output mapping.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParallelBody {
    pub mode: ParallelMode,
    pub branches: Vec<ParallelBranch>,
    /// Semantic concurrency bound; `Some(0)` is invalid.
    pub max_inflight: Option<u32>,
    pub output_bindings: Vec<ParallelOutputBinding>,
}

/// One first-terminal race branch.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RaceBranch {
    pub ordinal: BranchOrdinal,
    pub child: ComputationId,
}

/// RACE: at least two branches, one output interface, no winner policy.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RaceBody {
    pub branches: Vec<RaceBranch>,
}

/// One loop-carried value template (SSA without mutation).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoopCarryBinding {
    pub slot: ValueSlotId,
    pub initial: ExpressionId,
    pub body_output_port: PortNo,
}

/// FOREACH: bounded map-style iteration; the item slot is produced by this
/// ITERATE.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForeachBody {
    pub source: ExpressionId,
    pub item_slot: ValueSlotId,
    pub body: ComputationId,
    pub carry: Vec<LoopCarryBinding>,
    pub max_inflight: Option<u32>,
}

/// WHILE: guarded iteration with an explicit finite iteration budget.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WhileBody {
    pub guard: ExpressionId,
    pub body: ComputationId,
    pub carry: Vec<LoopCarryBinding>,
    /// `> 0`; v1 has no unbounded while.
    pub max_iterations: u32,
}

/// The two loop modes inside `ComputationBody::iterate` — not a fourteenth
/// kernel opcode.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IterateBody {
    Foreach(ForeachBody),
    While(WhileBody),
}

/// RETRY: an explicit finite delay schedule; maximum occurrence count is
/// exactly `1 + len(delays)`. Delay order is semantic and never re-sorted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryBody {
    pub child: ComputationId,
    /// Nominal duration micros, each `>= 0` (0 is a legal immediate retry).
    pub delays: Vec<Duration>,
}

impl<'de> Deserialize<'de> for RetryBody {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            child: ComputationId,
            /// Nominal duration micros (JSON numbers; ACE1 encodes them as
            /// canonical decimal strings).
            delays: Vec<i64>,
        }
        let raw = Raw::deserialize(deserializer)?;
        Ok(RetryBody {
            child: raw.child,
            delays: raw.delays.into_iter().map(Duration::new).collect(),
        })
    }
}

/// TIMEOUT: bound one child by a duration-valued expression. No timer state.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimeoutBody {
    pub child: ComputationId,
    pub timeout: ExpressionId,
}

/// A typed failure ABI match: the exact pinned failure schema.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureMatch {
    pub failure_type: TypeSchemaRef,
}

/// The closed handle match set. There is deliberately no `cancelled` case:
/// parent cancellation is not an ordinary failure handler.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandleMatch {
    Failure(FailureMatch),
    /// `{"timed_out": null}` — newtype-over-unit serializes as an explicit
    /// tagged null.
    TimedOut(()),
}

/// One handler clause.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandleClause {
    pub ordinal: HandlerOrdinal,
    /// Renamed from the AFIDL field name `match` (a Rust keyword).
    #[serde(rename = "match")]
    pub matcher: HandleMatch,
    pub handler: ComputationId,
}

/// HANDLE: protected child plus ordered handler clauses.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandleBody {
    pub child: ComputationId,
    pub clauses: Vec<HandleClause>,
}

/// SCOPE: one owned child plus statically owned logical resources.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopeBody {
    pub child: ComputationId,
    pub resources: Vec<ResourceSlotId>,
}

/// COMPENSATION_SCOPE: the owned child is compensation-recoverable. No
/// compensation task list lives in the IR.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompensationScopeBody {
    pub child: ComputationId,
}

/// CALL_SUBFLOW: call an exact published subflow by its `semantic-ir` digest.
/// Zero local children; no detach.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CallSubflowBody {
    pub contract: ContractRefId,
    pub inputs: Vec<InputBinding>,
    pub resources: Vec<ResourceUseBinding>,
}

/// The closed kernel opcode set: exactly these 13 cases, case numbers frozen
/// ABI in the Stable ABI Number Registry.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComputationBody {
    Invoke(InvokeBody),
    Await(AwaitBody),
    Sequence(SequenceBody),
    Select(SelectBody),
    Parallel(ParallelBody),
    Race(RaceBody),
    Iterate(IterateBody),
    Retry(RetryBody),
    Timeout(TimeoutBody),
    Handle(HandleBody),
    Scope(ScopeBody),
    CompensationScope(CompensationScopeBody),
    CallSubflow(CallSubflowBody),
}

impl ComputationBody {
    /// The AFIDL variant case name (the ACE1 value encoding and the frozen
    /// case order key).
    pub fn case_name(&self) -> &'static str {
        match self {
            ComputationBody::Invoke(_) => "invoke",
            ComputationBody::Await(_) => "await",
            ComputationBody::Sequence(_) => "sequence",
            ComputationBody::Select(_) => "select",
            ComputationBody::Parallel(_) => "parallel",
            ComputationBody::Race(_) => "race",
            ComputationBody::Iterate(_) => "iterate",
            ComputationBody::Retry(_) => "retry",
            ComputationBody::Timeout(_) => "timeout",
            ComputationBody::Handle(_) => "handle",
            ComputationBody::Scope(_) => "scope",
            ComputationBody::CompensationScope(_) => "compensation_scope",
            ComputationBody::CallSubflow(_) => "call_subflow",
        }
    }

    /// The frozen `case_no` of this opcode in `ComputationBody`.
    pub fn case_no(&self) -> u32 {
        match self {
            ComputationBody::Invoke(_) => 1,
            ComputationBody::Await(_) => 2,
            ComputationBody::Sequence(_) => 3,
            ComputationBody::Select(_) => 4,
            ComputationBody::Parallel(_) => 5,
            ComputationBody::Race(_) => 6,
            ComputationBody::Iterate(_) => 7,
            ComputationBody::Retry(_) => 8,
            ComputationBody::Timeout(_) => 9,
            ComputationBody::Handle(_) => 10,
            ComputationBody::Scope(_) => 11,
            ComputationBody::CompensationScope(_) => 12,
            ComputationBody::CallSubflow(_) => 13,
        }
    }

    /// The owned child computations, in structural order (with multiplicity —
    /// duplicate references are a validation error). This is the *ownership*
    /// edge set; expression and slot references are dataflow, not ownership.
    pub fn children(&self) -> Vec<ComputationId> {
        match self {
            // The iterate loop body is an owned child too, but it lives
            // inside the IterateBody wrapper; `ComputationDef::owned_children`
            // adds it.
            ComputationBody::Invoke(_)
            | ComputationBody::Await(_)
            | ComputationBody::CallSubflow(_)
            | ComputationBody::Iterate(_) => Vec::new(),
            ComputationBody::Sequence(body) => body.children.clone(),
            ComputationBody::Select(body) => body.cases.iter().map(|case| case.child).collect(),
            ComputationBody::Parallel(body) => {
                body.branches.iter().map(|branch| branch.child).collect()
            }
            ComputationBody::Race(body) => {
                body.branches.iter().map(|branch| branch.child).collect()
            }
            ComputationBody::Retry(body) => vec![body.child],
            ComputationBody::Timeout(body) => vec![body.child],
            ComputationBody::Handle(body) => {
                let mut children = vec![body.child];
                children.extend(body.clauses.iter().map(|clause| clause.handler));
                children
            }
            ComputationBody::Scope(body) => vec![body.child],
            ComputationBody::CompensationScope(body) => vec![body.child],
        }
    }
}

// ---------------------------------------------------------------------------
// The document
// ---------------------------------------------------------------------------

/// One node of the structured control tree.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComputationDef {
    pub id: ComputationId,
    pub outputs: Vec<ComputationOutputBinding>,
    pub body: ComputationBody,
}

impl ComputationDef {
    /// The owned child computations including the iterate loop body.
    pub fn owned_children(&self) -> Vec<ComputationId> {
        let mut children = self.body.children();
        if let ComputationBody::Iterate(body) = &self.body {
            match body {
                IterateBody::Foreach(foreach) => children.push(foreach.body),
                IterateBody::While(while_body) => children.push(while_body.body),
            }
        }
        children
    }
}

/// The canonical Semantic IR document: a structured control tree over the
/// reused Phase 3A tables, plus run parameters, workflow outputs and the
/// closed contract table.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticIrDocument {
    pub semantic_refs: Vec<SemanticVersionRef>,
    /// Reused `agentflow.semantic.core@1` tables — never redeclared.
    pub types: Vec<SemanticTypeNode>,
    pub values: Vec<ValueSlot>,
    pub expressions: Vec<ExpressionDef>,
    pub resources: Vec<ResourceSlot>,
    pub contracts: Vec<ContractRefEntry>,
    pub run_parameters: Vec<RunParameterDecl>,
    pub workflow_outputs: Vec<WorkflowOutputDecl>,
    pub computations: Vec<ComputationDef>,
    pub root: ComputationId,
}
