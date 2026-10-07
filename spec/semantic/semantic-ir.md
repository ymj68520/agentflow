# Canonical Semantic IR — v1

> Status: AgentFlow repository normative source for Phase 3B, Phase 3C and Phase 3D. This document
> defines the Canonical Semantic IR: its document structure, the closed
> 13-opcode `ComputationBody` set, structured ownership, value-producer
> rules, static interface rules, canonical ordering and the
> `SemanticIrDigest`. The machine schema is `spec/idl/semantic-ir.afidl`
> (`agentflow.semantic.ir @1`), the first production consumer of the AFIDL1
> cross-module import.
>
> The Semantic IR is **static**. It is not an executable plan, not a runtime
> state, and not a journal:
>
> ```text
> Canonical Semantic IR → later Resolver / Static Analysis
>                      → Resolved / Checked IR
>                      → Executable Plan
> ```
>
> No worker, region, endpoint, credential, lease, physical implementation or
> scheduler placement exists in this layer.

## 1. Scope and boundaries

<a id="ir-scope-static-only"></a>
<a id="ir-no-runtime-semantics"></a>
<a id="ir-occurrence-not-in-ir"></a>

- The IR defines structure, validation, canonicalization and the document
  digest only. No interpreter exists in this layer: there is no
  `execute_sequence()`, `run_retry()`, `choose_select()`, `race_children()` or
  `schedule_timeout()`, and none may be added here.
- There are no opcode state machines (`SequenceState`, `RetryState`,
  `RaceState`, `WaitState`), no runtime commands (`ScheduleTask`, `ArmWait`,
  `ScheduleTimer`) and no journal events (`SelectCaseChosen`,
  `RaceWinnerIteration`, `IterationGuardDecision`). Those belong to the later
  SemanticExecutionState / Reducer / Runtime stages.
- Runtime occurrence identity (`OccurrencePath`, `AttemptId`,
  `InvocationId`) does not exist in the IR. Runtime creates occurrences of
  static definitions; the IR holds only the definitions (e.g. one ITERATE has
  exactly one body computation definition, never a per-iteration copy).

## 2. Frozen boundaries

This phase does not modify: the ACE1 grammar, HashV1, the AFIDL1 canonical
TypeSchema model, AFIDL1 import semantics, `agentflow.semantic.core@1` (its 29
`TypeSchemaDigest`s are reproduced byte-for-byte), the semantic type DAG,
`ValueSlot`, the 15-expression closed set, or `ResourceSlot`. The frozen
HashV1 domain registry is unchanged: this phase *begins using* the
previously frozen `semantic-ir` and `component-contract` domains and
introduces **no** new domain (no `semantic-ir-document`, `computation`,
`computation-node`, `workflow-contract`, `subflow-contract`).

## 3. Document structure

<a id="ir-document-structure"></a>
<a id="ir-semantic-refs"></a>
<a id="ir-reuse-core-tables"></a>
<a id="ir-canonical-ordering"></a>

```text
SemanticIrDocument {
    semantic_refs    : list<SemanticVersionRef>   ; exact SpecificationId pins
    types            : list<core.SemanticTypeNode>
    values           : list<core.ValueSlot>
    expressions      : list<core.ExpressionDef>
    resources        : list<core.ResourceSlot>
    contracts        : list<ContractRefEntry>
    run_parameters   : list<RunParameterDecl>
    workflow_outputs : list<WorkflowOutputDecl>
    computations     : list<ComputationDef>
    root             : ComputationId
}
```

- The four tables `types`/`values`/`expressions`/`resources` reuse the
  `agentflow.semantic.core@1` objects **by identity** (the AFIDL module
  imports `agentflow.semantic.core@1 as core`). Redeclaring copies is
  forbidden. The tables must satisfy the Phase 3A rules; the IR validator
  runs the core check and fails on any core error.
- `SemanticVersionRef.specification_id` must be a canonical `SpecificationId`
  (syntax validated by the IR; registry *knownness* belongs to compiler/spec
  tooling — `af-ir` never reads `specifications.json`). References are unique
  and canonically ordered by the unsigned UTF-8 bytes of the canonical
  SpecificationId.
- Canonical ordering of every table and nested list is fixed; see §10.

## 4. The closed 13-opcode set

<a id="ir-opcode-set-closed"></a>
<a id="ir-case-numbers-frozen"></a>
<a id="ir-call-composite-forbidden"></a>
<a id="ir-trigger-not-computation"></a>

`ComputationBody` is a closed variant with exactly 13 cases. The `case_no`
values are durable ABI, frozen in the Stable ABI Number Registry on first
registration:

| case_no | case | body |
|---|---|---|
| 1 | invoke | InvokeBody |
| 2 | await | AwaitBody |
| 3 | sequence | SequenceBody |
| 4 | select | SelectBody |
| 5 | parallel | ParallelBody |
| 6 | race | RaceBody |
| 7 | iterate | IterateBody |
| 8 | retry | RetryBody |
| 9 | timeout | TimeoutBody |
| 10 | handle | HandleBody |
| 11 | scope | ScopeBody |
| 12 | compensation_scope | CompensationScopeBody |
| 13 | call_subflow | CallSubflowBody |

- No agent, model, tool, http, sql, shell, human, mcp or a2a case exists; the
  variant is closed, so unknown case names are rejected at decoding.
- **CallComposite is authoring/compiler-only** and MUST NOT appear in
  canonical IR: the authoring construct is lowered before this layer.
- **Trigger is not a computation**: triggers belong to deployment binding and
  have no opcode and no node in the IR.

## 5. Local identities and ports

<a id="ir-local-identities"></a>
<a id="ir-computation-outputs"></a>

- `ComputationId`, `RunParameterId` and `ContractRefId` are three new id
  namespaces, independent of each other and of the four Phase 3A namespaces;
  `PortNo`, `BranchOrdinal` and `HandlerOrdinal` are numbering spaces. All
  are u32 underneath with distinct nominal types; values are `> 0`, `0` is
  representable and rejected by validation.
- A computation's output ports (`ComputationOutputBinding { port_no,
  value_slot }`) are unique within the computation and `port_no > 0`. The
  port's type is the bound slot's type — it is never stored a second time
  (no `port/value/type` drift is possible).
- Input bindings (`InputBinding { port_no, expression }`) and resource use
  bindings (`ResourceUseBinding { port_no, resource_slot }`) have unique
  positive ports; the referenced expression/resource must exist. Full
  component-contract input matching is a later (contract checking) stage.

## 6. Value producers

<a id="ir-value-producer-uniqueness"></a>
<a id="ir-expression-not-producer"></a>
<a id="ir-run-parameter-producer"></a>

Phase 3A froze `ValueSlot` as immutable SSA with one producer; Phase 3B
checks producers for the first time. The complete producer set is:

```text
RunParameterDecl.value_slot          run_parameter producer
ComputationDef.outputs[].value_slot  computation_output producer
Iterate foreach item_slot            iterate_item producer
Iterate carry binding slot           iterate_carry producer
```

- Every declared value slot MUST have exactly one producer:
  `0 producers → unbound_value_slot`, `>1 producers → duplicate_value_producer`.
- An `ExpressionDef` is pure derived computation and is **never** a producer:
  expression ids do not materialize value slots.

### 6.1 Run parameters

<a id="ir-run-parameter-decl"></a>
<a id="ir-run-parameter-name"></a>

- `RunParameterDecl { id, name, type, value_slot }` is a static workflow
  input declaration; `parameter.type == value_slot.type` is required.
- No default value, admission snapshot, mutable parameter, runtime update,
  constraint or secret flag exists in this layer (Run Admission /
  RunParameterSnapshot are later stages).
- `name` is non-empty, NFC-normalized and unique after NFC normalization.
  Names are user-facing and MAY be any Unicode; canonical order of
  `run_parameters` is by `RunParameterId`, never by name.

### 6.2 Workflow outputs

<a id="ir-workflow-output"></a>

- `WorkflowOutputDecl { name, expression }`; the output type is the
  expression's `result_type` and is never duplicated on the output. `name`
  follows the run-parameter name rules; canonical order is by the unsigned
  UTF-8 bytes of the NFC-normalized name.

## 7. Closed contract table

<a id="ir-contract-table-closed"></a>
<a id="ir-component-contract-domain"></a>
<a id="ir-subflow-contract-domain"></a>
<a id="ir-contract-identity-unique"></a>

- `ContractRefEntry { id, kind, digest }` with `digest: core.DigestRefValue`.
  `ContractRefKind` is closed at exactly two members: `component` (1) and
  `subflow` (2). Policy/effect/resource/signal/worker contract kinds are not
  designed yet and are not pre-registered.
- `kind == component` requires `digest = sha256 / component-contract`;
  `kind == subflow` requires `digest = sha256 / semantic-ir` — CALL_SUBFLOW
  pins an exact published Semantic IR identity. No `workflow-contract` or
  `subflow-contract` domain exists; if a distinct WorkflowContract object is
  ever introduced, that is an explicit specification upgrade.
- Entries are canonically ordered by `ContractRefId`; duplicate ids are
  rejected, and **duplicate contract identity** is rejected: one exact
  contract (`algorithm`, `domain`, `hex`) has at most one table entry per
  document.
- Validation covers digest shape, expected domain and uniqueness only. There
  is no Contract Store in this layer: nothing is loaded by digest.

## 8. Structured ownership

<a id="ir-ownership-tree"></a>
<a id="ir-child-table"></a>
<a id="ir-child-unique-parent"></a>
<a id="ir-root"></a>
<a id="ir-unreachable"></a>
<a id="ir-structural-cycle"></a>
<a id="ir-dataflow-ownership-separation"></a>

- Child references per opcode (the ownership edge set):

```text
INVOKE             0 local children
AWAIT              0
SEQUENCE           N children (order is semantic, preserved)
SELECT             N cases
PARALLEL           N branches
RACE               N branches
ITERATE            1 body
RETRY              1 child
TIMEOUT            1 child
HANDLE             1 protected child + N handlers
SCOPE              1 child
COMPENSATION_SCOPE 1 child
CALL_SUBFLOW       0 local children
```

- Hard rules: the root exists; the root has no structural parent; every
  non-root computation has exactly one parent; every computation is reachable
  from the root; no ownership cycle (`computation_cycle`); no detached child;
  a child referenced by two parents is `multiple_structural_parents`; a child
  listed twice by the same parent is `duplicate_child_reference`. Handlers
  are owned children of HANDLE; an iterate body is the one owned child of the
  ITERATE (occurrences are runtime, never static copies).
- Unreachable computations are rejected (`unreachable_computation`); the
  compiler must clean or reject dead blocks before canonical IR.
- **Dataflow edge ≠ ownership edge**: expression→slot and computation→child
  are different relations handled by different validator logic. There is no
  universal `Edge` type.
- Validation order inside the ownership phase is fixed so corpus
  expectations are stable: root existence (`root == 0` is an id error), then
  structural cycle (DFS from root), then root-has-parent, then
  multiple-parents, then reachability.

## 9. Static interface rules

<a id="ir-interface-projection"></a>
<a id="ir-wrapper-interfaces"></a>
<a id="ir-leaf-interfaces"></a>

- `ComputationInterface` is a read-only projection `port_no → SemanticTypeId`
  derived from `outputs[].value_slot → ValueSlot.type`. No second interface
  table may be stored anywhere.
- Wrapper equality (parent interface == child interfaces):

```text
Sequence   == last child
Select     == every case
Race       == every branch
Retry      == child
Timeout    == child
Handle     == protected child + every handler
Scope      == child
CompScope  == child
Parallel   by explicit output mapping
Iterate    by carry/output rules
```

- Leaves (`Invoke`, `Await`, `CallSubflow`) declare their own interface only;
  contract-level checking is deferred.
- Dominance, branch-local value leakage, availability at execution point and
  cross-branch merge correctness are **Phase 3C**: this phase checks
  reference existence, producer uniqueness, tree validity and type shapes,
  and deliberately nothing more.

## 10. Per-opcode static schemas

### 10.1 INVOKE / CALL_SUBFLOW / AWAIT

<a id="ir-invoke-static"></a>
<a id="ir-call-subflow"></a>
<a id="ir-await-static"></a>

- INVOKE: `contract` must exist with kind `component`; inputs/resources are
  port lists as in §5. INVOKE states only "call this exact component
  contract with these semantic inputs/resources" — no TaskDefinition,
  TaskBinding, Invocation, Attempt or Worker exists here.
- CALL_SUBFLOW: `contract` must exist with kind `subflow`; zero local
  children; no `detach`/`fire_and_forget`/`owned=false` — v1 has no detached
  children; the subflow is an owned durable child boundary.
- AWAIT: `signal_type` is an exact digest-pinned `TypeSchemaRef`;
  `correlation` is an expression. AWAIT is a passive durable wait; no
  mailbox, timer, webhook or human-approval state exists in this layer. AWAIT
  has no built-in timeout — compose `TIMEOUT(AWAIT(...))` to keep the frozen
  composition semantics.

### 10.2 SEQUENCE / SELECT

<a id="ir-sequence-order"></a>
<a id="ir-select-ordinal"></a>
<a id="ir-select-interfaces"></a>

- SEQUENCE: at least one child; the child list order is the semantic
  execution order and MUST be preserved (never canonically re-sorted). The
  parent's output interface equals the last child's output interface; the
  parent's own output slots are new SSA slots (child→parent value
  availability/passthrough lowering is Phase 3C).
- SELECT: the selector expression has type `int64` and produces the *exact*
  case ordinal to run. There is no truthiness matching, no source-order
  matching and no fall-through: the case ordinal is semantic ABI. Ordinals
  are `> 0`, unique, canonically ascending; at least one case. All case
  children share the parent's output interface (typed merge shell). No
  runtime case selection is implemented in this phase.

### 10.3 PARALLEL / RACE

<a id="ir-parallel-structure"></a>
<a id="ir-parallel-output-mapping"></a>
<a id="ir-parallel-max-inflight"></a>
<a id="ir-race-structure"></a>

- `ParallelMode` is closed at `all` (1) and `fail_fast` (2).
- PARALLEL: at least one branch; ordinals `> 0` and unique, canonically
  ascending. `max_inflight: option<uint32>` is a **semantic** concurrency
  bound; `Some(0)` is invalid. Worker count, tenant quota, region capacity
  and scheduler limits are physical concerns and MUST NOT enter the IR
  (a physical limit may later be narrower).
- Parallel output mapping (`parent_port_no, branch_ordinal, child_port_no`):
  parent ports unique across bindings; branch and child output port must
  exist; the parent slot's type equals the child output slot's type.
  Canonically ordered by `parent_port_no`.
- RACE: at least two branches; ordinals `> 0` and unique. v1 race semantics
  is exactly FIRST_TERMINAL: there is deliberately no `winner_policy`,
  `first_success` or `fastest_success`. All branches and the parent share one
  output interface; the winner is a canonical runtime terminal observation,
  not static information.

### 10.4 ITERATE

<a id="ir-iterate-foreach"></a>
<a id="ir-iterate-while"></a>
<a id="ir-loop-carry"></a>

- `IterateBody` is a closed variant with exactly `foreach` (1) and `while`
  (2). This is a mode selector inside `ComputationBody.iterate`, not a
  fourteenth opcode.
- FOREACH: the source expression has type `list<T>`; the item slot exists,
  has type `T`, and its producer is this ITERATE; the body exists;
  `max_inflight` as in PARALLEL. The source expression identifies the
  collection materialized for iteration; "materialize once" and ordinal
  iteration identity are runtime semantics.
- WHILE: the guard has type `bool`; `max_iterations > 0` — v1 has no
  unbounded while. The guard may reference carry slots; availability and
  dominance analysis is Phase 3C; this phase checks carry typing and producer
  uniqueness only.
- Loop carry (`LoopCarryBinding { slot, initial, body_output_port }`): the
  slot exists; the initial expression exists; the body output port exists;
  and `slot.type == initial.result_type == body-output-slot.type`. One
  binding per carry slot (canonically ordered by slot id). Loops never
  mutate: the runtime creates a new value occurrence per iteration.

### 10.5 RETRY / TIMEOUT

<a id="ir-retry-schedule"></a>
<a id="ir-timeout-expression"></a>

- RETRY retries the owned child **computation** (not a task attempt) under an
  explicit finite delay schedule: attempt #1 runs; on failure wait
  `delays[0]`; attempt #2; on failure wait `delays[1]`; … The maximum
  occurrence count is exactly `1 + len(delays)`. This makes retry finite,
  deterministic and canonical with no wall clock, no random jitter and no
  runtime policy lookup. The delay list order is semantic and never sorted;
  each delay is a duration `>= 0` (0 is a legal immediate retry; no fake
  timer semantics are created for it). The child's output interface equals
  the parent's.
- TIMEOUT: `timeout` is a duration-valued expression; the child's output
  interface equals the parent's. There is no `TimerId`, `TimerState` or
  `TimerCommand` — only the semantic duration expression is stored.
  Timer-vs-child competition is runtime semantics.

### 10.6 HANDLE

<a id="ir-handle-match"></a>

- `HandleMatch` is closed at exactly `failure` (typed exact `TypeSchemaRef`
  match) and `timed_out` (unit payload). There is deliberately no
  `cancelled`: parent cancellation is not an ordinary failure handler; a
  fixture attempting it is rejected at decoding.
- Handler ordinals are `> 0`, unique, canonically ascending; failure clauses
  must be pairwise disjoint by exact `TypeSchemaRef` (there is no subtyping,
  so an exact duplicate is a conflict); at most one `timed_out` clause.
- All handlers and the protected child expose the parent's output interface.
  Detailed outcome semantics are deferred.

### 10.7 SCOPE / COMPENSATION_SCOPE

<a id="ir-scope-resources"></a>
<a id="ir-compensation-scope"></a>

- SCOPE: one owned child plus `resources: list<core.ResourceSlotId>` — the
  logical resources the scope statically owns/allows; they must exist, be
  unique, and are canonically ascending. This freezes the ownership boundary
  and resource lexical boundary only: authority attenuation, budget
  attenuation, deadline state, cancellation state and `ResourceHandle` are
  later, separately specified tables — no placeholder Any fields are added.
- COMPENSATION_SCOPE: one owned child. The IR holds NO compensation task
  list; compensation sources are later derived from committed effect receipts
  plus effect compensation contracts. A normative rule is registered now:
  COMPENSATION_SCOPE MUST NOT erase forward computation or effect history.

## 11. Canonical ordering

<a id="ir-canonical-order-table"></a>
<a id="ir-semantic-order-exceptions"></a>

```text
semantic_refs              canonical SpecificationId bytes
types                      SemanticTypeId
values                     ValueSlotId
expressions                ExpressionId
resources                  ResourceSlotId
contracts                  ContractRefId
run_parameters             RunParameterId
workflow_outputs           unsigned UTF-8 bytes of the NFC name
computations               ComputationId
Computation.outputs        PortNo
Invoke/CallSubflow inputs  PortNo
Invoke/CallSubflow res.    PortNo
Select.cases               ordinal
Parallel.branches          ordinal
Parallel.output_bindings   parent_port_no
Race.branches              ordinal
Handle.clauses             ordinal
Foreach/While.carry        carry slot id
Scope.resources            ResourceSlotId
```

The two **semantic-order exceptions** — never sorted: `Sequence.children`
(execution order) and `Retry.delays` (schedule order).

## 12. SemanticIrDigest

<a id="ir-digest-formula"></a>

```text
SemanticIrDigest = HashV1("semantic-ir", ACE1(CanonicalSemanticIrDocument))
                 = SHA256("AF\0" || "semantic-ir" || "\0" || canonical_bytes)
```

- The domain `semantic-ir` is the Phase 1 frozen registry entry, used from
  this phase onward. No other domain is touched; in particular
  `TypeSchemaDigest` continues to hash schemas under `type-schema`.
- The digest hashes the **canonical** document, so table source order never
  affects it, while any semantic-order change (sequence children, retry
  delays, per-body configuration) does.
- The ACE1 shape is derived from the normative schema module set
  (`agentflow.semantic.ir@1` + the imported `agentflow.semantic.core@1`) by
  cross-module expansion; the value tree mirrors the AFIDL schema
  field-for-field.

## 13. Scoped typed dataflow (Phase 3C)

<a id="ir-scoped-dataflow"></a>
<a id="ir-value-availability"></a>
<a id="ir-per-use-site"></a>
<a id="ir-unused-expression"></a>

Phase 3B guarantees that every declared value slot exists and has exactly one
producer. Existence is **not** availability:

```text
A value slot may be read at a use site only if its producer dominates
that use site in the structured ownership tree.
```

- The rule is enforced by a deterministic walk over the ownership tree with
  one opcode-specific availability transfer function per control form. No
  general CFG dominator algorithm exists here: the 13-opcode control
  structures are already a structured tree, so a producer dominates a use
  exactly when the structured transfer places the produced slot into that
  use site's environment.
- Availability is checked **per use site**, never cached per expression. The
  same `ExpressionDef` MAY be legal at a use site inside its owning branch
  and illegal at a use site outside it; the outside use is rejected.
- Expressions carry no scope, owner, region or parent field: whether an
  expression is legal depends only on where its id is used.
- An expression that is referenced by **no** use site creates no use and
  never fails availability: an expression definition is not an execution.
- The stable error kind is `value_not_available`; the diagnostic detail
  carries the value slot, the use site, the producer and an internal reason
  token (`before_producer`, `cross_branch`, `outside_owner_boundary`,
  `iterate_local_outside`, `parent_output_not_yet_available`). Reasons are
  diagnostics, not ABI; the kind is the contract.

### 13.1 Environments

<a id="ir-run-parameter-availability"></a>
<a id="ir-output-boundary"></a>
<a id="ir-export-boundary"></a>

```text
EntryValues(root) = every RunParameterDecl.value_slot
Exports(C)        = exactly C.outputs[].value_slot
```

- Run parameters are immutable workflow inputs: they are available from the
  root entry, in every descendant computation, under every control branch,
  for the whole run.
- A computation's outputs become available only after that computation's
  structured completion boundary. In particular: an output MUST NOT be used
  at its own computation's entry, and a parent output MUST NOT be used
  inside the parent's own subtree.
- `Exports(C)` contains only the computation's **own** outputs. Descendant
  outputs, branch locals, iterate item/carry slots and loop-body outputs
  never cross an owner boundary implicitly: values leave a subtree only
  through the subtree root's declared outputs.

### 13.2 Per-opcode availability transfer

<a id="ir-sequence-dominance"></a>
<a id="ir-branch-isolation"></a>

```text
SEQUENCE  EntryValues(child_i) = E + Exports(child_1..child_{i-1})
          (list order is the only multi-child dataflow order)
SELECT    EntryValues(case_i)   = E        (every case independently)
PARALLEL  EntryValues(branch_i) = E        (every branch independently)
RACE      EntryValues(branch_i) = E        (every branch independently)
RETRY / TIMEOUT / SCOPE / COMPENSATION_SCOPE
          EntryValues(child)    = E
HANDLE    EntryValues(protected) = EntryValues(handler_i) = E
INVOKE / CALL_SUBFLOW / AWAIT
          inputs / correlation evaluate at the computation entry, over E
TIMEOUT   the timeout duration evaluates at the timeout entry, over E
FOREACH   source and every carry initial evaluate pre-loop, over E
          body over E + {item_slot} + {carry slots}
WHILE     carry initials over E; guard and body over E + {carry slots}
```

<a id="ir-sequence-future"></a>

- SEQUENCE is the one multi-child control form whose child order is
  dataflow: a later direct sibling reads an earlier direct sibling's export.
  A child MUST NOT read a later sibling's export (use-before-producer), a
  grandchild's raw output MUST NOT reach its parent's later sibling (only
  the wrapper's own export may), and case/branch/clause ordinals are
  identity keys, never execution order: SELECT cases, PARALLEL branches,
  RACE branches and HANDLE clauses are mutually isolated, and a raw
  child/case/branch/handler output consumed after the wrapper is rejected —
  only the wrapper's own outputs are visible outside.
<a id="ir-select-isolation"></a>
<a id="ir-parallel-isolation"></a>
<a id="ir-race-isolation"></a>

- A HANDLE handler MUST NOT read the protected child's outputs: the handler
  runs precisely when the protected child may have failed or timed out, so
  its success outputs cannot be assumed. No `FailureValue` or failure-slot
  placeholder exists in this phase.
<a id="ir-handle-isolation"></a>

- A WHILE guard evaluates at the loop header: it sees the carry slots but
  never a body computation output; a body output reaches the next iteration
  only through its `LoopCarryBinding`.
- Carry initials all evaluate from the same pre-loop environment: no initial
  may read a carry slot or the item slot, so the canonical carry order (by
  slot id) never becomes an evaluation order.

### 13.3 Iterate loop-local values

<a id="ir-iterate-local-values"></a>

<a id="ir-iterate-source-pre-loop"></a>
<a id="ir-carry-initial-pre-loop"></a>
<a id="ir-foreach-body-environment"></a>
<a id="ir-while-environment"></a>

- A foreach `item_slot`, every carry slot and every loop-body output is
  iteration-internal: none of them MAY be read outside their iterate
  computation. Outside an ITERATE only `Iterate.outputs` are visible.

<a id="ir-iterate-local-escape"></a>

### 13.4 Workflow output boundary

<a id="ir-workflow-output-boundary"></a>

```text
WorkflowExitValues = RunParameterSlots + Exports(root)
```

- Every workflow-output expression is evaluated after the root computation's
  terminal boundary and MUST read only `WorkflowExitValues`. A raw
  root-descendant output (however deep in the root's interior it was used)
  is rejected; run parameters remain legal because their lifetime covers
  the whole run.

### 13.5 The analysis artifact and the Phase 3C freeze

<a id="ir-analysis-derived"></a>
<a id="ir-wire-freeze"></a>

- The analysis is **derived-only**: `SemanticDataflowAnalysis`
  (`EntryValues`, `EntryResources`, expression value-slot dependency
  closures, producer locations, `WorkflowExitValues`) is a plain
  compile-time projection. It is never ACE1-encoded, never enters the
  `SemanticIrDigest` (which still hashes the canonical document only), and
  is absent from AFIDL, the ABI registry and the wire model.
- Phase 3C freeze: `spec/idl/semantic-ir.afidl` stays byte-identical (36
  TypeSchemaDigests), the 13 `ComputationBody` case numbers stay frozen, no
  AFIDL module, no ABI row, no HashV1 domain and no Rule Area is added.
  The analysis runs strictly **after** Phase 3B validation, so every Phase
  3B error kind is unchanged; the analysis adds only `value_not_available`
  and `resource_not_visible`.
- The expression dependency closure reuses the Phase 3A expression edge set;
  no second cycle semantics exists.

## 14. Resource lexical visibility (Phase 3C)

<a id="ir-resource-visibility"></a>
<a id="ir-resource-attenuation"></a>
<a id="ir-resource-restore"></a>
<a id="ir-resource-use-sites"></a>
<a id="ir-resource-no-concurrency"></a>

```text
EntryResources(root) = every declared ResourceSlotId
EntryResources(child) = EntryResources(parent)          (every opcode but SCOPE)
EntryResources(scope.child) = exactly ScopeBody.resources
```

- A SCOPE is an attenuation boundary: its listed resource set MUST be a
  subset of the inherited environment, and its child sees exactly the listed
  set — never the union. A nested scope cannot reintroduce a resource the
  outer scope omitted.
<a id="ir-resource-no-widen"></a>

- Narrowing ends at the scope boundary: a sibling after a SCOPE (in a
  SEQUENCE or anywhere else) continues with the parent's resource
  environment.
- The resource use sites are `InvokeBody.resources` and
  `CallSubflowBody.resources`: each used slot MUST be lexically visible at
  the computation's entry environment (`resource_not_visible` otherwise).
  Resource slot existence is Phase 3B (`unknown_resource_slot`); visibility
  is Phase 3C.
- Lexical visibility is shared, not owned: two PARALLEL/RACE branches MAY
  use the same visible resource slot, and whether concurrent use is safe
  belongs to the later Resource Contract / effect / scheduler layers. No
  `resource_shared_in_parallel` error exists and none may be derived here.
- Visibility is not mutable state: no resource mutation ordering, locking
  or transactions are modeled in this phase.

## 15. Subflow interface derivation (Phase 3D)

<a id="ir-subflow-interface-derivation"></a>
<a id="ir-subflow-interface-derived"></a>
<a id="ir-subflow-port-ordinals"></a>

A CALL_SUBFLOW pins its target by an exact `SemanticIrDigest` (the contract
table enforces `kind == subflow` and `digest.domain == semantic-ir`); the
target Semantic IR **is** the subflow's semantic identity. Its public surface
is therefore not copied into a new contract object — it is derived:

```text
target run_parameters    → input ports   (RunParameterId ascending)
target workflow_outputs  → output ports  (canonical NFC-name byte order)
target resources         → resource ports (ResourceSlotId ascending)
```

- The derived `SubflowInterface` is a plain compile-time projection. It
  MUST NOT enter AFIDL, the ABI registry, the `SemanticIrDigest` or any
  wire serialization; no `SemanticTypeDigest` or interface digest exists —
  structural equality is the comparison, nothing here is hashed.
- Each mapping is **1-based and canonical-key ordered**, so source table
  order never matters: reordering any non-semantic table moves neither the
  digest nor the derived interface. The three port namespaces are
  independent: input port 1, output port 1 and resource port 1 have no
  relation to each other.
- Port ordinals are **derived external interface coordinates, not aliases**
  for `RunParameterId` / `WorkflowOutput` name / `ResourceSlotId`. Target
  names are carried as diagnostic/inspection metadata only; bindings use
  `port_no` exclusively, and no call wire field is added.
<a id="ir-subflow-interface-immutable"></a>

- As long as the `SemanticIrDigest` is fixed, the derived interface is
  fixed (deterministic and immutable); any change to a parameter type, an
  output type, a resource requirement or order-relevant identity produces a
  new digest, and a caller pinned to the old digest never follows silently.

### 15.1 Input, output and resource ports

<a id="ir-subflow-input-ports"></a>
<a id="ir-subflow-output-ports"></a>
<a id="ir-subflow-resource-ports"></a>

<a id="ir-subflow-inputs-exact"></a>

- Every target run parameter is a required static input: there is no
  default, optional omission or variadic input, so the caller's
  `CallSubflowBody.inputs` port set must exactly equal the derived input
  port set `1..N`.
<a id="ir-subflow-outputs-exact"></a>

- Every workflow output is part of the output interface: the caller's
  `ComputationDef.outputs` must exactly equal the derived output port set.
  An unconsumed output stays an unused value slot — the call's own contract
  interface never becomes partial.
<a id="ir-subflow-resources-exact"></a>

- **Every declared target `ResourceSlot` is part of the resource
  requirement interface**, not only the ones that happen to be used: the
  slot is the contract surface (Phase 3A), and an unused slot is an
  authoring defect to fix before publishing, never something this layer
  hides. The caller's `CallSubflowBody.resources` port set must exactly
  equal `1..N`, and each bound caller slot's `requirement_type` must equal
  the target port's `requirement_type` as an exact `TypeSchemaRef`.
- A resource requirement is a pinned contract schema, not a value type:
  value types and resource requirement types are never conflated.

### 15.2 Exact target lookup

<a id="ir-subflow-exact-target"></a>
<a id="ir-subflow-no-fallback"></a>
<a id="ir-subflow-no-transitive-closure"></a>

- The pinned digest must resolve **exactly** in an explicit, immutable,
  in-memory target universe (`SubflowIrUniverse`); otherwise the call is
  rejected as `unknown_subflow_target`. There is no same-workflow-name
  latest fallback, no filesystem, registry, network, package or deployment
  lookup — fail closed.
- The universe accepts only fully validated targets: Phase 3B by the
  checked-document type, Phase 3C because insertion runs the scoped
  dataflow analysis itself. An invalid target is uninsertable.
- The digest is always computed **from** the target document; a caller
  never passes a trusted digest string alongside a document. Re-inserting
  the same canonical bytes is idempotent; the same digest over different
  bytes is an invariant violation and is rejected, never silently
  overwritten.
- Interface derivation does **not** recursively validate the target's own
  transitive subflow closure, and subflow-graph cycles (A → B → A) are not
  judged here: dependency closure and graph acyclicity belong to the later
  Resolver / LockGraph stage. This phase is not a partial resolver.

### 15.3 Cross-document type equivalence

<a id="ir-cross-document-type-equivalence"></a>
<a id="ir-schema-leaf-exact-equality"></a>

A `SemanticTypeId` is a document-local DAG coordinate: numeric equality
across caller and target proves nothing and numeric difference proves
nothing. Ports and bindings are therefore compared as structurally
resolved types:

```text
primitive p        == primitive p'        iff p == p'
schema r           == schema r'           iff r is the exact TypeSchemaRef
list A             == list B              iff A == B
map<string, A>     == map<string, B>      iff A == B
option A           == option B            iff A == B
```

- A schema leaf compares the complete pin — `specification_id`,
  `type_name` **and** `schema_digest`. Comparing the name alone is
  forbidden; the same record name under a different specification version
  is a different type.
- The expansion terminates because the Phase 3A type DAG is acyclic; the
  comparison needs no new digest and no new Hash domain.

### 15.4 CALL_SUBFLOW interface checking

<a id="ir-call-subflow-interface-checking"></a>
<a id="ir-subflow-pass-order"></a>

- The pass order is frozen:
  `Phase 3B check → Phase 3C analysis → Phase 3D subflow check`. The
  subflow pass consumes the checked document and its own analysis; it
  never runs before them and never changes a Phase 3B/3C error kind.
  Existing Phase 3B kinds (unknown contract, wrong contract kind,
  duplicate ports, unknown expression/resource) stay exactly where they
  are.
- Per call, in ascending computation id order, the stable error kinds are:
  `unknown_subflow_target`; `subflow_input_port_mismatch` /
  `subflow_input_type_mismatch`; `subflow_output_port_mismatch` /
  `subflow_output_type_mismatch`; `subflow_resource_port_mismatch` /
  `subflow_resource_type_mismatch`. Diagnostics carry the computation,
  the port and the expected/actual resolved types; the rendering is not
  durable ABI.
- A document with no CALL_SUBFLOW passes against an empty universe.
  INVOKE keeps its Phase 3B state: no component port checking exists in
  this phase, and no ports-only `ComponentContract` placeholder is
  created — the real contract core (I/O, failure row, effect signature,
  resource requirements, information flow, usage, recovery, execution
  requirements, replay/idempotency/timeout policies) is materialized
  once, in Phase 4, and INVOKE exact checking returns there.

## 16. Explicitly deferred (Phase 4 and later)

The following are **not** defects of this document; they are later layers:

- the Component/Component-Contract core (Phase 4): ports over the complete
  I / O / F / R / U / K / X dimensions plus replay, idempotency, timeout
  and recovery policies — no ports-only shrinkage may occupy the
  `component-contract` domain's meaning first; INVOKE exact contract
  checking returns there;
- the Resolver / LockGraph: subflow dependency closure, published-graph
  acyclicity and cycle policy (Phase 3D deliberately does not judge
  A → B → A);
- Await mailbox/matcher runtime semantics; the typed FailureValue runtime
  ABI (handlers therefore see no protected-child outputs);
- Effect Signature; Resource Contract meaning; resource concurrency rules;
  Scope authority/budget attenuation beyond the lexical subset rule;
- runtime occurrence identity; journal decisions; executable plans; any
  runtime dominance, liveness, storage, stack frame or SSA-phi construct
  (the control wrappers' own output slots are the merge boundaries).

## 17. Conformance

Frozen corpus: `spec/conformance/semantic-ir/` — `golden/` (per-opcode and
nested documents with canonical `ace1_utf8`/`ace1_hex`/`semantic_ir_digest`),
`negative/` (documents with expected stable error kinds, including decode-time
rejections), and `equivalence/` (ordering-equivalence groups and semantic-order
mutations). All values are generated by the independent Python implementation
`tools/gen-semantic-ir-corpus.py`; tests never generate expected values at
runtime. The AFIDL corpus carries the `semantic-ir` schema module additively
(revision 5): all pre-existing cases, bytes and digests are unchanged.

Phase 3C adds `spec/conformance/semantic-dataflow/` — `positive/` (documents
that pass Phase 3B **and** the availability analysis, with frozen
`EntryValues`/`EntryResources`/`WorkflowExitValues` projections, the
per-use-site expression reuse fixture and the unused-expression fixture) and
`negative/` (documents that pass Phase 3B and are rejected only by the
analysis, with frozen `value_not_available`/`resource_not_visible` kinds and
diagnostic reason tokens). It is generated by the independent Python
implementation `tools/gen-semantic-dataflow-corpus.py` and carries no digest:
the analysis is never hashed. The same generator cross-checks all Phase 3B
golden documents against its availability oracle.

Phase 3D adds `spec/conformance/subflow-interface/` — `positive/` (targets
plus callers that pass Phase 3B + Phase 3C and check clean against the
explicit universe, with the target digests and the derived interface
projections frozen), `negative/` (callers rejected only by the subflow
check, with frozen kinds) and `equivalence/` (table order moves neither the
digest nor the interface; local type renumbering moves neither the interface
shape; port ordinals follow the frozen canonical key orders). It is
generated by the independent Python implementation
`tools/gen-subflow-interface-corpus.py`; the interface projections are
marked NON_NORMATIVE CONFORMANCE PROJECTION — the analysis is never hashed
and never serialized in production.
