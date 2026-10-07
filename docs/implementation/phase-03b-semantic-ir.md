# Phase 3B — Canonical Semantic IR: Implementation Report

> Scope: the first materialization of the Canonical Semantic IR — a structured
> control tree over the closed 13-opcode set, a typed dataflow shell over the
> Phase 3A tables, and the closed contract tables — plus validation,
> canonicalization and the `SemanticIrDigest`. No journal, reducer,
> `SemanticExecutionState`, runtime command, worker, scheduler, effect gateway
> or opcode state machine was implemented.
>
> Decision register: [`phase-03b-semantic-ir-decisions.md`](phase-03b-semantic-ir-decisions.md)

## 1. SemanticIrDocument 最终结构

```text
SemanticIrDocument {                       agentflow.semantic.ir @1
    1  semantic_refs    : list<SemanticVersionRef>     ; exact SpecificationId pins
    2  types            : list<core.SemanticTypeNode>  ; reused by identity
    3  values           : list<core.ValueSlot>
    4  expressions      : list<core.ExpressionDef>
    5  resources        : list<core.ResourceSlot>
    6  contracts        : list<ContractRefEntry>       ; closed table
    7  run_parameters   : list<RunParameterDecl>
    8  workflow_outputs : list<WorkflowOutputDecl>
    9  computations     : list<ComputationDef>
    10 root              : ComputationId
}
```

The four core tables are reused **by identity** through
`import agentflow.semantic.core@1 as core;` — the Phase 2C capability's first
production consumer. `semantic-ir.afidl` declares 36 types (35 domain types
plus the document record; 100 ABI assignment rows derived from it), including
6 new identity/numbering aliases (`ComputationId`, `RunParameterId`,
`ContractRefId`, `PortNo`, `BranchOrdinal`, `HandlerOrdinal`).

## 2. 新增 AFIDL 类型数量

36 new types in `agentflow.semantic.ir@1` (documented in §1). Plus the reused
core types by reference: `SemanticTypeNode`, `ValueSlot`, `ExpressionDef`,
`ResourceSlot`, `SemanticTypeId`, `ValueSlotId`, `ExpressionId`,
`ResourceSlotId`, `TypeSchemaRef`, `DigestRefValue`.

## 3. 13 Opcode case_no 与 body 类型表

| case_no | case | body | local children |
|---|---|---|---|
| 1 | invoke | InvokeBody { contract, inputs, resources } | 0 |
| 2 | await | AwaitBody { signal_type, correlation } | 0 |
| 3 | sequence | SequenceBody { children } | N (order = semantics) |
| 4 | select | SelectBody { selector, cases } | N cases |
| 5 | parallel | ParallelBody { mode, branches, max_inflight, output_bindings } | N branches |
| 6 | race | RaceBody { branches } | N branches (≥2) |
| 7 | iterate | IterateBody { foreach \| while } | 1 body |
| 8 | retry | RetryBody { child, delays } | 1 |
| 9 | timeout | TimeoutBody { child, timeout } | 1 |
| 10 | handle | HandleBody { child, clauses } | 1 + N handlers |
| 11 | scope | ScopeBody { child, resources } | 1 |
| 12 | compensation_scope | CompensationScopeBody { child } | 1 |
| 13 | call_subflow | CallSubflowBody { contract, inputs, resources } | 0 |

Case numbers are derived into the ABI registry and asserted there by the
conformance harness. No `call_composite`, no `trigger`, no agent/model/tool
opcode — structurally inexpressible in the closed variant.

## 4. Structured ownership validator 规则

Root exists (`root == 0` is `invalid_local_id`) → structural cycle (DFS from
root, `computation_cycle`) → root-has-parent → exactly-one-parent
(`multiple_structural_parents`; a child listed twice by one parent is
`duplicate_child_reference`) → reachability (`unreachable_computation`). The
per-opcode child table is normative (§3); handlers and iterate bodies are
owned children. Dead nodes are rejected, never dropped.

## 5. Value producer 模型

Exactly one producer per declared `ValueSlot`: `run_parameter`
(`RunParameterDecl.value_slot`), `computation_output`
(`ComputationDef.outputs[].value_slot`), `iterate_item` (foreach item slot),
`iterate_carry` (carry binding slot). 0 producers → `unbound_value_slot`;
>1 → `duplicate_value_producer`. Expressions are never producers. Run
parameter `type == value_slot.type`; names non-empty/NFC/unique-after-NFC;
canonical order by `RunParameterId`.

## 6. Computation output interface 模型

`ComputationOutputBinding { port_no, value_slot }` — positive, unique ports;
type = `value_slot.type`, never duplicated. `ComputationInterface` is the
read-only projection `port_no → SemanticTypeId`, derived on demand; no second
interface table exists anywhere. Wrapper equality (§10 of the normative doc):
SEQUENCE == last child; SELECT == every case; RACE == every branch;
RETRY/TIMEOUT/SCOPE/COMPENSATION_SCOPE == child; HANDLE == protected child +
every handler; PARALLEL by explicit mapping; ITERATE by carry rules;
INVOKE/AWAIT/CALL_SUBFLOW are leaves.

## 7. RunParameter / WorkflowOutput 模型

Run parameters: static input declarations producing their slots; no default,
snapshot, mutability, update, constraint or secret flag. Workflow outputs:
`{ name, expression }`, type derived from `ExpressionDef.result_type` (never
stored twice), NFC names, canonical order by NFC name bytes.

## 8. Contract table 模型

`ContractRefEntry { id, kind, digest }` with `ContractRefKind` closed at
{ component (1), subflow (2) }. Component pins `sha256/component-contract`;
subflow pins `sha256/semantic-ir` (an exact published Semantic IR). Entries
canonically ordered by `ContractRefId`; duplicate ids and duplicate contract
identities rejected. No Contract Store: shape/domain/uniqueness validation
only; port compatibility is Phase 3C.

## 9. Select / Parallel / Race 结构规则

SELECT: int64 selector produces the exact ordinal (no truthiness/fall-through);
ordinals positive/unique/ascending; ≥1 case; all cases share the parent
interface. PARALLEL: mode closed at all/fail_fast; ≥1 branch; `max_inflight`
option<uint32> with `Some(0)` invalid (semantic bound, never physical);
explicit output mapping with type equality; canonically ordered by ordinal /
parent port. RACE: ≥2 branches; FIRST_TERMINAL only (no winner_policy); one
shared interface.

## 10. Iterate carry 模型

`LoopCarryBinding { slot, initial, body_output_port }`: slot exists,
initial expression exists, body output port exists, and
`slot.type == initial.result_type == body-output-slot.type`; one binding per
slot (canonically ordered by slot id). FOREACH source is `list<T>` with the
item slot produced by the iterate; WHILE guard is bool with
`max_iterations > 0` (no unbounded loop). Loops never mutate — runtime
occurrences, not static copies.

## 11. Retry schedule representation

Explicit finite delay vector `delays: list<duration>` (recorded decision
D-18 / normative rule AF-SEMANTIC-0062). Occurrence count = `1 + len(delays)`;
each delay `>= 0` (0 = immediate retry); list order is the schedule and is
never canonically sorted — reordering changes the digest (frozen mutation
golden). No wall clock, jitter or runtime policy lookup.

## 12. Handle matcher 模型

`HandleMatch` closed at `failure` (exact typed `TypeSchemaRef`) and
`timed_out` (unit). No `cancelled` — the closed enum makes it
undeserializable (frozen negative, `rejected_at: deserialize`). Ordinals
positive/unique/ascending; failure clauses disjoint by exact schema; ≤1
timed_out; all handlers + the protected child share the parent interface.

## 13. SemanticIrDigest 公式

```text
SemanticIrDigest = HashV1("semantic-ir", ACE1(CanonicalSemanticIrDocument))
                 = SHA256("AF\0" || "semantic-ir" || "\0" || canonical_bytes)
```

The ACE1 shape is derived cross-module from the normative schema set via
`CompiledModuleSet::shape_of` (new Phase 3B extension of the Phase 2C module
set). The value tree mirrors the AFIDL schema field-for-field. Digests are
computed over the canonical form only: `crates/af-ir/src/canonical.rs`
implements the frozen ordering table, and the corpus proves that table source
order never moves the digest while semantic order (sequence children, retry
delays) always does.

## 14. 新增 Specification / Rule / ABI 统计

```text
Specifications : 12 → 14   (+agentflow.semantic.ir@1 schema,
                            +agentflow.semantic.ir.semantics@1 schema_language)
Rules          : 82 → 129  (+47 AF-SEMANTIC-0025..0071; no existing id touched)
Coverage       : 128 covered / 1 intentionally uncovered / 0 missing
ABI rows       : 132 → 232 (+100 derived from semantic-ir.afidl; all
                            pre-existing rows byte-identical)
Suites         : 10 → 16   (+AUTHORING-ONLY, COMPENSATION-HISTORY,
                            AWAIT-COMPOSITION, DATAFLOW-OWNERSHIP-SEPARATION,
                            INTERFACE-PROJECTION, NO-RUNTIME)
```

## 15. Golden / Negative corpus 数量

```text
spec/conformance/semantic-ir/
  golden/semantic-ir-golden.json        22 documents (all 13 opcodes covered
                                        + 6 nested compositions), each with
                                        frozen ace1_utf8 / ace1_hex /
                                        semantic_ir_digest
  negative/semantic-ir-negative.json    79 cases with frozen error kinds
                                        (78 check-layer + 1 decode-layer:
                                         handle/cancelled-matcher)
  equivalence/…equivalence.json         11 ordering-equivalence groups +
                                        3 semantic-order mutations, all with
                                        frozen expected digests
```

Golden coverage: invoke, await, sequence, select, parallel (all + fail-fast),
race, iterate (foreach + while), retry, timeout, handle (failure + timed-out),
scope, compensation-scope, call-subflow, and the six required nested
compositions (`sequence(invoke, await)`, `retry(timeout(invoke))`,
`scope(parallel(...))`, `compensation_scope(sequence(...))`,
`select(retry, call_subflow)`, `while(sequence)`).

The independent Python generator (`tools/gen-semantic-ir-corpus.py`)
implements canonicalization, validation, ownership, producer uniqueness, the
13 body schemas, interface checks, ACE1 bytes and the digest — and asserts
every golden/negative/equivalence expectation before freezing. It never calls
the Rust implementation. The Rust harness (`crates/af-ir/tests/`) reproduces
every frozen value; the cross-check caught two genuine wire-model divergences
during development, which is exactly its job.

## 16. AFIDL corpus revision

**4 → 5**, purely additive: the `semantic-ir` case joins `machine-primitives`,
`specification-registry`, `semantic-core` and the import fixtures. Canonical
type cases 127 → 168 (+41); all pre-existing bytes and digests unchanged
(machine-verified, §17). The parser-invalid manifest stays at revision 2.

## 17. old digest / semantic-core 零漂移证明

```text
git diff --stat HEAD -- spec/encoding spec/digest spec/conformance/encoding
    spec/conformance/digest spec/tables/hash-domains.json
    spec/tables/afidl-primitives.json                        -> empty
git diff --stat HEAD -- spec/idl/semantic-core.afidl spec/idl/core
    spec/conformance/semantic-core crates/af-semantic        -> empty
```

Machine checks against the previous revision:

```text
pre-existing AFIDL digest cases : 127 (all unchanged; 41 additive → 168)
semantic-core digest cases      : 29 (all unchanged)
hash-domains.json               : byte-identical (14 frozen domains)
ACE1 / HashV1 corpora           : byte-identical
semantic-core corpus            : byte-identical
pre-existing ABI rows           : 132 (all unchanged; +100 additive)
pre-existing rule ids           : 82 (all unchanged; +47 additive)
```

`hash-domains.json` is untouched: `semantic-ir` and `component-contract` were
already frozen Phase 1 entries; this phase only *begins using* them.

## 18. spec-check 结果

```text
specifications: 14
normative rules: 129
rules covered: 128
rules intentionally uncovered: 1
rules missing coverage: 0
ABI assignments: active 232 / retired 0 / reserved 0
AFIDL modules checked: 4
cross-reference errors: 0          exit 0
```

`spec-check` now compiles the four normative AFIDL modules as an import-aware
module set (the Phase 2C machinery) — semantic-ir imports semantic-core by
exact identity — and cross-checks all 232 derived ABI rows.

## 19. workspace 门禁结果

```text
cargo fmt --all -- --check                                 OK
cargo clippy --all-targets --all-features -- -D warnings    OK (clean)
cargo test --all                                            181 passed, 0 failed
cargo run -p af-spec --bin spec-check -- check              exit 0
all six corpus generators, two consecutive runs             zero diff
```

Test count moved from 160 to 181 (+21): +7 in `af-ir` unit tests (model serde,
ownership basics, exact pinning), +6 conformance tests (golden reproduction,
nested coverage, negative kinds, ordering/mutation digests, frozen ABI opcode
table, determinism) and +6 static-surface tests (13-opcode closure, forbidden
opcodes, no runtime objects, purity, digest domain, interface projection).
Per the phase instruction the baseline was never hardcoded — the gate is
`cargo test --all → 0 failed`.

## 20. explicit deferred 项

Recorded in the normative doc §13 and the decision register D-24: full scoped
dominance; branch-local value leakage; typed branch merge availability;
value availability at execution points; ComponentContract port validation;
Subflow interface validation; Await mailbox/matcher runtime semantics;
FailureValue runtime ABI; Effect Signature; Resource Contract meaning; Scope
authority/budget attenuation; runtime occurrence identity; journal decisions;
executable plans; the Resolver/LockGraph; run admission and
RunParameterSnapshot. None of these is a Phase 3B defect.

## 21. 是否存在阻塞 Phase 3C 的问题

**None identified.** Everything Phase 3C needs is in place: the checked IR
document with derived interfaces, the producer map, the ownership tree, and
deliberately-unvalidated availability (branch-local leakage, dominance) whose
rule set can now be added without schema changes. The one design constraint
to respect: dominance analysis must extend the *validator*, not the wire
model — the frozen `SemanticIrDigest` and the ABI table must not move when
Phase 3C adds its checks (the same additive discipline Phase 3B followed).

## 22. git commits / diff 概览

Categorized commits follow this report; the working tree is otherwise clean.
Files touched:

```text
crates/af-idl/src/resolve.rs                      +cross-module shape derivation
crates/af-ir/*                                    new crate (model/check/canonical/errors/ids)
crates/af-idl/tests/conformance_afidl.rs          revision pin 5
crates/af-ir/tests/*                              conformance + static surface
spec/idl/semantic-ir.afidl                        new normative schema module
spec/semantic/semantic-ir.md                      new normative semantics
spec/conformance/semantic-ir/**                   new frozen corpus
spec/conformance/afidl/**                         revision 5 (additive)
spec/registry/{rules,specifications,abi-numbers}.json, suites.json  additive
tools/gen-semantic-ir-corpus.py                   new independent generator
tools/gen-afidl-corpus.py, tools/gen-registry-instances.py        extended
docs/implementation/phase-03b-semantic-ir{,-decisions}.md         new
```

到此停止。不进入 Phase 3C。
