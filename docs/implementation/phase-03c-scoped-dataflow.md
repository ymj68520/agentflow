# Phase 3C — Scoped Typed Dataflow: Implementation Report

> Status: Phase 3C complete. The analysis layer is materialized, every gate
> is green and every durable surface is byte-identical to Phase 3B. Normative
> text: `spec/semantic/semantic-ir.md` sections 13–14; decisions:
> `phase-03c-scoped-dataflow-decisions.md`.

## 1. EntryValues / Exports 正式模型

- `EntryValues(C)` — sorted `ValueSlotId` set per computation, recorded
  during the analysis walk for **every** computation (all are reachable per
  Phase 3B ownership). `EntryValues(root)` = all run-parameter slots; run
  parameters stay in every descendant environment by inheritance.
- `Exports(C)` = exactly `C.outputs[].value_slot`. Descendant outputs,
  branch locals, iterate item/carry slots and loop-body outputs never cross
  an owner boundary; every "raw output used after the wrapper" negative in
  the corpus is this one rule failing closed.
- Both models are analysis metadata (`SemanticDataflowAnalysis`), never
  wire objects.

## 2. Expression dependency closure 模型

`referenced_value_slots(ExpressionId) -> BTreeSet<u32>`: transitive
value-slot closure over `af_semantic::ExprTable::references` (the Phase 3A
edge set, reused verbatim — no second cycle semantics), memoized over the
acyclic expression DAG, exposed as
`SemanticDataflowAnalysis::expression_value_dependencies()`.

## 3. 各 Opcode availability transfer 规则

| opcode | transfer |
| --- | --- |
| INVOKE / CALL_SUBFLOW | inputs evaluated at entry over `E`; resource uses over `EntryResources` |
| AWAIT | correlation over `E` |
| SEQUENCE | children in list order; `E_i = E ∪ Exports(c_1..c_{i−1})` |
| SELECT | selector over `E`; every case over `E` (ordinal ascending) |
| PARALLEL | every branch over `E` (ordinal ascending) |
| RACE | every branch over `E` (ordinal ascending) |
| ITERATE foreach | source + carry initials over `E`; body over `E ∪ {item} ∪ carries` |
| ITERATE while | carry initials over `E`; guard + body over `E ∪ carries` |
| RETRY | child over `E` |
| TIMEOUT | duration over `E`; child over `E` |
| HANDLE | protected child and every handler independently over `E` |
| SCOPE | resources narrowed to exactly the listed subset; child values over `E` |
| COMPENSATION_SCOPE | child over `E` |

## 4. Sequence dominance 规则

`EntryValues(child_i) = E ∪ Exports(child_1..child_{i−1})` — cumulative,
list order is the only multi-child dataflow order. Future reference
(child 1 reading child 2's export) is `value_not_available /
before_producer`; grandchild leakage to a later sibling is
`outside_owner_boundary` (only the wrapper's own export may cross);
parent output read inside the parent's subtree is
`parent_output_not_yet_available`.

## 5. Select / Parallel / Race branch isolation

All branches start from the wrapper's entry environment; ordinals are
identity keys, never execution order. Cross-branch reads are
`cross_branch`. Raw case/branch outputs consumed after the wrapper are
`outside_owner_boundary` — the race rule makes no winner distinction (the
static analysis cannot know the winner, so no branch output escapes raw).

## 6. Retry / Timeout / Handle / Scope 边界规则

Single-child wrappers pass `E` through unchanged; child outputs stay
iteration of the wrapper's boundary (raw use after the wrapper rejected).
The TIMEOUT duration evaluates at the timeout entry (cannot read child
outputs). HANDLE handlers see exactly `E` — not the protected child's
outputs (they run precisely when those may not exist); no FailureValue or
failure-slot placeholder exists. SCOPE narrows resources only (values
unchanged).

## 7. Iterate item/carry 生命周期

`source` and every carry `initial` evaluate from the same pre-loop
environment (initials cannot order among themselves — canonical carry
order by slot id is not evaluation order). Body env =
`E ∪ {item_slot} ∪ carries`. While guard/body env = `E ∪ carries`; the
guard cannot read a body output (only via `LoopCarryBinding`). Item,
carry and body-output slots never escape the iterate
(`iterate_local_outside` / `outside_owner_boundary`); outside an ITERATE
only `Iterate.outputs` are visible. While-carry producers are registered
symmetrically with foreach (a bug the Python oracle caught in the first
cross-run, fixed in both implementations).

## 8. WorkflowOutput availability 规则

`WorkflowExitValues = RunParameterSlots ∪ Exports(root)`, frozen per
positive case. Root-descendant raw outputs, select case outputs and
iterate locals are rejected at the workflow-output use site.

## 9. Resource lexical environment 模型

`EntryResources(root)` = every declared `ResourceSlotId`; every opcode but
SCOPE inherits; recorded per computation as
`SemanticDataflowAnalysis::entry_resources()`. Use sites:
`InvokeBody.resources` and `CallSubflowBody.resources` must be lexically
visible at the computation's entry. Visibility is shared, not owned —
parallel branches may use the same resource, and no concurrency judgment
exists at this layer.

## 10. Scope narrowing 规则

`ScopeBody.resources ⊆ EntryResources(scope)` (else
`resource_not_visible`); the child sees exactly the listed set — never the
union — so nested-scope reintroduction fails structurally. Narrowing ends
at the scope boundary: later siblings continue with the parent
environment (frozen by `resource/scope-exit-restores-sibling`).

## 11. 新增 error kinds

Exactly two, both raised only by `analyze_dataflow`:
`value_not_available` (diagnostic detail carries slot, use site, producer,
reason token) and `resource_not_visible`. Reason tokens
(`before_producer`, `cross_branch`, `outside_owner_boundary`,
`iterate_local_outside`, `parent_output_not_yet_available`) are
diagnostics, not ABI.

## 12. 新增 AF-SEMANTIC 规则与 coverage

24 additive rules `AF-SEMANTIC-0072..0095` (no existing RuleId touched),
8 new suites `SUITE-SEMANTIC-DATAFLOW-*`. Totals: **153 rules** (was 129),
**24 suites** (was 16), **152 covered + 1 intentionally uncovered**,
**missing coverage = 0**. Each rule binds a dedicated anchor; shared
subsections got their own anchors so the one-anchor-per-rule binding
holds.

## 13. Positive / Negative corpus 数量

`spec/conformance/semantic-dataflow/` (revision 1):
- `positive/` — **20** documents: the 17 required scenarios plus the
  expression-reuse inside-use base, the unused-expression fixture and
  cumulative sibling exports; each freezes `entry_values`,
  `entry_resources`, `workflow_exit_values`.
- `negative/` — **37** documents: sequence 4, select 3, parallel 2,
  race 2, retry 1, timeout 2, scope 1, comp-scope 1, handle 3, foreach 7,
  while 4, workflow-output 3, expression reuse 1, resource 3. Every one
  passes Phase 3B and fails only the analysis.

## 14. Independent oracle 实现范围

`tools/gen-semantic-dataflow-corpus.py` independently implements: the
expression dependency closure, producer locations (run parameters,
computation outputs, foreach item, foreach **and while** carry), the full
per-opcode environment walk, loop-local lifetimes, the workflow exit
boundary, resource visibility and the reason classifier. It never calls
Rust. It re-uses only document plumbing (base tables, builders, the
Phase 3B validator, the golden corpus) from the 3B generator, and it
cross-checks all 22 Phase 3B goldens against its oracle.

## 15. SemanticDataflowAnalysis 派生结构

`{ entry_values, entry_resources, expression_value_dependencies,
producer_locations, workflow_exit_values }` — plain Rust, `PartialEq`,
canonically ordered (ascending raw ids) purely for deterministic tests.
It is never ACE1-encoded, never hashed, absent from AFIDL/ABI/registry,
and `canonical.rs` contains no reference to it (static-audited). API:
`CheckedIrDocument::analyze_dataflow()`; `check(...)` unchanged and
compatible.

## 16. Phase 3B golden/digest 零漂移证明

- `git diff` over `spec/conformance/semantic-ir/`: **empty** (22 golden
  bytes + digests, 79 negative kinds, 11+3 equivalence all untouched).
- The Rust golden harness re-reproduced every frozen `ace1_hex` and
  `semantic_ir_digest` (existing test, still green).
- New test: every golden passes `analyze_dataflow` **and** its digest is
  identical before/after the analysis.
- The Python oracle independently accepts all 22 golden documents.

## 17. semantic-core / AFIDL / ABI 零漂移证明

- `git diff` over `spec/idl/`, `spec/tables/`,
  `spec/registry/abi-numbers.json`: **empty**. `semantic-ir.afidl` is
  byte-identical (sha256 still
  `9fb3292a…616a3`, pinned by test to the registry, enforced by
  spec-check against the file) → its 36 `TypeSchemaDigest`s and the 100
  semantic-ir ABI rows cannot move; semantic-core keeps its 29 digests
  and 82 ABI rows; ABI total stays **232** (regenerated, no drift); the
  HashV1 registry stays **14** domains.
- AFIDL corpus revision stays **5**; semantic-core corpus revision stays
  1; both generators re-ran with zero diff.
- Only `specifications.json` changed where required:
  `agentflow.semantic.ir.semantics@1`'s source digest followed the
  semantic-ir.md edit (2bfd1edd…); `agentflow.semantic.ir@1`'s schema
  digest did not move.

## 18. spec-check 结果

`cargo run -p af-spec --bin spec-check -- check` → exit 0:
specifications 14 · normative rules 153 · covered 152 · intentionally
uncovered 1 · missing coverage 0 · ABI active 232 · AFIDL modules 4 ·
cross-reference errors 0.

## 19. Workspace 门禁结果

- `cargo fmt --all -- --check` — pass.
- `cargo clippy --all-targets --all-features -- -D warnings` — clean.
- `cargo test --all` — **190 passed / 0 failed** (181 → 190; +5 dataflow
  conformance, +4 wire-freeze surface tests; no previous test count was
  hardcoded anywhere).
- All 7 generators run twice → zero diff (aggregate sha256 of every
  generated tree identical across rounds).

## 20. Explicit deferred 项（Phase 3D）

Component/Subflow contract port validation against materialized contract
ABIs; the typed FailureValue runtime ABI; resource concurrency rules;
Scope authority beyond the lexical subset rule; Effect Signature; runtime
occurrence identity/journal/plans; any runtime dominance, liveness,
storage, stack-frame or SSA-phi construct. No temporary JSON contract
format was created.

## 21. 是否存在阻塞 Phase 3D 的问题

None identified. The contract table already carries exact digest pins per
kind (`component-contract` / `semantic-ir` domains), interfaces are
derived on demand, and the analysis walk is the natural host for future
contract port checking. Phase 3D can extend the validator + analysis
without touching the wire model, exactly as Phase 3C did.

## 22. Git commits / diff 概览

```text
a7b4e55 feat(af-ir): the scoped typed dataflow analysis pass
3b6744f test(semantic-ir): freeze the Phase 3C dataflow corpus
99e5f24 test(af-ir): audit the Phase 3C wire-model freeze
9659689 spec(registry): register the 24 scoped-dataflow rules and suites
<docs>  docs(implementation): phase-03c decision register and report
```

New files: `crates/af-ir/src/dataflow.rs` (~800 lines),
`crates/af-ir/tests/conformance_semantic_dataflow.rs` (5 tests),
`tools/gen-semantic-dataflow-corpus.py` (independent oracle + corpus),
`spec/conformance/semantic-dataflow/{positive,negative}/`,
`docs/implementation/phase-03c-*.md`. Modified: `error.rs` (+2 kinds),
`lib.rs` (exports/doc), `check.rs` (doc line only), `static_surface.rs`
(+4 tests), `semantic-ir.md` (+2 sections), `gen-registry-instances.py`
(+24 rules/+8 suites), regenerated registry JSONs.
