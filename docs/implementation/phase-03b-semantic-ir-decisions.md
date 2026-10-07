# Phase 3B — Canonical Semantic IR: Decision Register

> Status: Phase 3B implementation record. Decisions here are binding on the
> Phase 3B implementation; they are *not* new frozen specification text. The
> normative source is `spec/semantic/semantic-ir.md` and the machine schema is
> `spec/idl/semantic-ir.afidl`.

## D-1 Document field model and numbering

`SemanticIrDocument` carries exactly the ten fields recommended by the phase,
with explicit field numbers 1..10 registered in the ABI registry:
`semantic_refs`, `types`, `values`, `expressions`, `resources`, `contracts`,
`run_parameters`, `workflow_outputs`, `computations`, `root`. The four core
tables reuse `agentflow.semantic.core@1` types **by identity** through the
AFIDL import — the schema declares `core.SemanticTypeNode` etc. and never
redeclares a copy. `root` is a `ComputationId` field, not a wrapper record.

## D-2 `SemanticIrDigest` formula and domain

`SemanticIrDigest = HashV1("semantic-ir", ACE1(CanonicalSemanticIrDocument))`
over the canonical form (see D-11). The `semantic-ir` domain is the Phase 1
frozen entry, used from this phase onward. `component-contract` (also frozen
in Phase 1) is used for component contract pins. **No new domain** was added:
`semantic-ir-document`, `computation`, `computation-node`, `workflow-contract`
and `subflow-contract` do not exist, and `TypeSchemaDigest` keeps hashing
schemas under `type-schema`.

The ACE1 bytes are encoded against the schema-derived shape:
`af_idl::CompiledModuleSet::shape_of("agentflow.semantic.ir@1/SemanticIrDocument")`
expands named references across the module set (the Phase 2C import path);
termination is guaranteed by `compile_module_set`'s global type-graph
acyclicity validation.

## D-3 The IR is not an executable plan

No Worker, Region, Endpoint, Credential, Lease, physical implementation,
scheduler placement, interpreter, opcode state machine, runtime command,
journal event, reducer, `SemanticExecutionState` or occurrence identity exists
in `af-ir`. The static-surface test enforces this over the source, the schema
and the dependency set.

## D-4 Exactly 13 opcodes; case numbers are ABI

`ComputationBody` is a closed variant with exactly the 13 cases in the phase's
order; `case_no` 1..13 are derived into the ABI registry from the AFIDL
variant and frozen there. The conformance harness re-reads the registry and
asserts the table. No agent/model/tool/http/sql/shell/human/mcp/a2a case, no
`call_composite`, no `trigger` — the static-surface test rejects all of them
on both the schema and the code.

## D-5 CallComposite / Trigger exclusion

`CallComposite` is authoring/compiler-only: there is no case, no type and no
symbol. `Trigger` is a deployment binding: likewise absent. Both are covered
by suites (`SUITE-SEMANTIC-IR-AUTHORING-ONLY`) because their absence is a
structural property, not a fixture.

## D-6 Six new local identity/numbering types, nominal in Rust

`ComputationId`, `RunParameterId`, `ContractRefId`, `PortNo`, `BranchOrdinal`,
`HandlerOrdinal` are u32 newtypes (`> 0`; `0` representable and rejected by
validation), independent of each other and of the four Phase 3A namespaces.
They are defined by a crate-local `ir_id!` macro rather than by re-exporting
`af-semantic`'s private macro, so `crates/af-semantic` stays byte-identical
(zero-drift proof by empty diff).

## D-7 Run parameters produce; nothing else materializes slots

`RunParameterDecl { id, name, type, value_slot }` is a static input
declaration whose producer is the parameter. The producer set is exactly:
run parameters, computation outputs, iterate item slots, iterate carry slots.
Expressions are never producers. Every declared slot must have exactly one
producer (`unbound_value_slot` / `duplicate_value_producer`). No default
value, admission snapshot, mutable parameter, runtime update, constraint or
secret flag exists in this layer.

Names are user-facing Unicode: non-empty, NFC, unique after NFC; canonical
order of `run_parameters` is by `RunParameterId`, never by name.

## D-8 Workflow outputs derive their type

`WorkflowOutputDecl { name, expression }` — the output type is the
expression's `result_type`, never duplicated. Canonical order by unsigned
UTF-8 bytes of the NFC-normalized name.

## D-9 Closed contract table with two kinds

`ContractRefKind` = { component (1), subflow (2) } — the only kinds whose
contract models are frozen; policy/effect/resource/signal/worker are not
pre-registered. `ContractRefEntry { id, kind, digest: core.DigestRefValue }`.
Domain binding: component → `component-contract`, subflow → `semantic-ir`
(CALL_SUBFLOW pins an exact published Semantic IR). One exact contract
identity has at most one table entry. Validation covers digest shape, domain
and uniqueness only — there is no Contract Store, nothing is loaded by digest,
and component/subflow port compatibility is explicitly Phase 3C.

## D-10 Generic output ports; one derived interface

Every computation declares its own outputs (`port_no`, `value_slot`); port
type = slot type, never stored twice. `ComputationInterface` is a read-only
`port_no → SemanticTypeId` projection computed on demand — no second interface
table exists anywhere (static test). Wrapper equality: SEQUENCE == last child;
SELECT == every case; RACE == every branch; RETRY/TIMEOUT/SCOPE/
COMPENSATION_SCOPE == child; HANDLE == protected child + every handler;
PARALLEL by explicit output mapping; ITERATE by carry rules;
INVOKE/AWAIT/CALL_SUBFLOW are leaves.

## D-11 Canonical ordering: one table, two semantic-order exceptions

Every table and nested list has a frozen canonical order (normative doc §11).
Exactly two lists are semantic order and never sorted: SEQUENCE children and
RETRY delays. Loop-carry bindings are canonically ordered by carry slot id
(a decision the phase left open — carry is a set of bindings with no
execution order between them, and slot ids make the order total). Parallel
`output_bindings` order by `parent_port_no`; scope resources by
`ResourceSlotId`.

## D-12 Ownership: a tree, checked in a fixed order

Ownership comes only from `owned_children()` (the per-opcode child table;
handlers and iterate bodies count; iterate has one static body). Validation
order inside the ownership phase is fixed so corpus expectations are stable:
root existence (`root == 0` is an id error) → structural cycle (DFS from root,
so a root-anchored cycle reports `computation_cycle` rather than
`root_has_parent`) → root-has-parent → multiple structural parents (a child
listed twice by one parent is the distinct `duplicate_child_reference`) →
reachability (`unreachable_computation`; dead nodes are rejected, never
silently dropped).

## D-13 Dataflow ≠ ownership

Expression→slot references are validated by the Phase 3A core check;
computation→child references by the IR ownership logic. No universal `Edge`
type exists. Dominance, branch-local leakage, availability at execution
points and merge correctness are Phase 3C and are deliberately not
approximated.

## D-14 SELECT ordinals are exact dispatch

The selector (`int64`) produces the exact case ordinal; no truthiness, no
source-order matching, no fall-through. Ordinals are positive, unique,
canonically ascending. All case children share the parent's interface.

## D-15 PARALLEL: mode, semantic bound, explicit mapping

`ParallelMode` closed at { all (1), fail_fast (2) }; at least one branch;
`max_inflight: option<uint32>` with `Some(0)` invalid — a **semantic** bound;
worker/tenant/region/scheduler limits are physical and out of scope. Branch
outputs reach the parent interface only through explicit
`ParallelOutputBinding { parent_port_no, branch_ordinal, child_port_no }`
with type equality.

## D-16 RACE: FIRST_TERMINAL only

At least two branches, one shared interface, no `winner_policy` /
`first_success` / `fastest_success` — the winner is a runtime terminal
observation.

## D-17 ITERATE: two modes, SSA carry

`IterateBody` is a closed variant { foreach (1), while (2) } inside
`ComputationBody.iterate` — not a fourteenth opcode. FOREACH: source
`list<T>`, item slot typed `T` produced by the iterate. WHILE: bool guard,
`max_iterations > 0` (no unbounded loop). Carry binding
`{ slot, initial, body_output_port }` with `slot.type == initial.result_type
== body-output-slot.type`, one binding per slot. Loops never mutate; the
runtime creates per-iteration occurrences.

## D-18 RETRY: explicit finite delay vector

The frozen documents specify finite deterministic backoff without pinning a
wire representation; Phase 3B adopts the **explicit delay vector**
`delays: list<duration>` as the v1 core representation. Attempt count is
exactly `1 + len(delays)`. Each delay is `>= 0` (0 = immediate retry, no fake
timer semantics); the list order is the schedule and is never sorted. RETRY
retries the owned child *computation*, not a task attempt. Decision recorded
per the phase instruction (§68).

## D-19 TIMEOUT: a duration expression, no timer

`timeout` is a duration-valued expression; no `TimerId`/`TimerState`/
`TimerCommand`. Child interface equals the parent's; timer-vs-child
competition is runtime semantics. AWAIT has no built-in timeout —
`TIMEOUT(AWAIT(...))` is the composition.

## D-20 HANDLE: closed match set without `cancelled`

`HandleMatch` = { failure (exact typed `TypeSchemaRef`), timed_out (unit) }.
There is no `cancelled` variant — parent cancellation is not an ordinary
failure handler; a fixture attempting it cannot even deserialize. Handler
ordinals positive/unique/ascending; failure clauses disjoint by exact schema
(no subtyping); at most one timed_out.

## D-21 SCOPE: resources-only static shape

`ScopeBody { child, resources }` with existent, unique, canonically ascending
`ResourceSlotId`s. No authority/budget attenuation, deadline/cancellation
state or `ResourceHandle`, and no placeholder Any fields — future extensions
come as separate tables. COMPENSATION_SCOPE holds no compensation task list;
compensation sources derive later from effect receipts, and the normative
rule "MUST NOT erase forward computation or effect history" is registered now
(`SUITE-SEMANTIC-IR-COMPENSATION-HISTORY`).

## D-22 Schema universe and contract digests

`Await.signal_type` and `Handle.failure_type` resolve exactly against the
explicit `TypeSchemaUniverse` (never "latest"); the digest-pinning machinery
is reused from the Phase 3A universe, not reimplemented. Contract digests are
validated for shape + expected domain + uniqueness only.

## D-23 Python/Rust wire-model discipline

The corpus JSON is the serde form; the ACE1 value tree is derived from it by
a *shape-driven* converter (`json_to_value`) so field names come from the
schema itself. The Python generator independently implements canonicalization,
validation, ownership, producers, interface checks, bytes and digest; it
never calls the Rust implementation. Two real wire-model divergences were
caught by the cross-check during development (enum-vs-string encoding of
`SemanticPrimitiveType`; contract digest domain spelling) and fixed — the
corpus is the arbiter.

## D-24 Deferred to Phase 3C and later (recorded, not omissions)

Full scoped dominance; branch-local value leakage; typed branch merge
availability; ComponentContract port validation; Subflow interface
validation; Await mailbox/matcher runtime semantics; FailureValue runtime
ABI; Effect Signature; Resource Contract meaning; Scope authority/budget
attenuation; runtime occurrence identity; journal decisions; executable
plans; the Resolver/LockGraph; run admission and RunParameterSnapshot.
