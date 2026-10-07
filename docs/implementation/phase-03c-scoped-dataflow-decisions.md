# Phase 3C — Scoped Typed Dataflow: Decision Register

> Status: Phase 3C implementation record. Decisions here are binding on the
> Phase 3C implementation; they are *not* new frozen specification text beyond
> what `spec/semantic/semantic-ir.md` (sections 13–14) now states. The wire
> model is untouched: `spec/idl/semantic-ir.afidl` stays byte-identical at its
> Phase 3B registry digest
> (`9fb3292a0fe90b207db037575c50ef3ca50174411b3c5a461c09c37d53f616a3`).

## D-1 The analysis is a separate pass, not a `check()` extension

`SemanticIrDocument::check` keeps its exact Phase 3B behavior — the pass list,
the error precedence and therefore all 79 frozen negative kinds are untouched
by construction. Phase 3C lives in `crates/af-ir/src/dataflow.rs` behind
`CheckedIrDocument::analyze_dataflow()`, which the task explicitly blessed
(建议 API: `analyze_scoped_dataflow(...)` / non-wire analysis on the checked
document). Availability is enforced by the new pass and frozen by the new
corpus, not by re-threading `check`. The task's ordering requirement
(analysis strictly after Phase 3B validation) is structural: the analyzer
consumes `&CheckedIrDocument`, so Phase 3B violations never reach it.

## D-2 Dominance without a CFG

The 13-opcode control structures are already a structured tree, so the
analyzer is one recursive walk with one opcode-specific transfer function per
control form. No dominator bitsets, no worklist, no generic CFG. Per 78 of
the phase, this is still dominance by definition: a producer dominates a use
exactly when the structured transfer places the produced slot into that use
site's environment. `EntryValues`/`EntryResources` are recorded for every
computation during the walk, which doubles as the analysis projection.

## D-3 Environments: `EntryValues` and `Exports`

`EntryValues(root)` = every run-parameter value slot; run parameters are in
every descendant environment by inheritance (no special cases downstream).
`Exports(C)` = exactly `C.outputs[].value_slot` — never descendants, never
branch locals, never iterate locals. This single rule produces all the
leakage rejections: raw case/branch/handler/child outputs consumed after
their wrapper fail because the wrapper's exports are the only thing that
crossed the boundary. SEQUENCE is the only multi-child dataflow order:
`EntryValues(child_i) = E ∪ Exports(child_1..child_{i−1})`, accumulated in
list order (the semantic order frozen in Phase 3B).

## D-4 `ProducerLocation` as analysis metadata

`ProducerLocation { RunParameter, ComputationOutput, IterateItem,
IterateCarry }` is a plain Rust enum in `dataflow.rs`, exported for
consumers and tests but absent from AFIDL, from the ABI registry, from the
canonical form and from the digest. While-loops carry slots too: the first
cross-run of the independent Python oracle caught the Rust (and Python)
analyzer registering only foreach carry producers — while carry slots are
Phase 3B producers just the same. Both were fixed; the corpus now pins the
while-carry behavior from four directions.

## D-5 Expression dependency closure reuses Phase 3A edges

`referenced_value_slots` walks `af_semantic::ExprTable::references` — the
same edge function the Phase 3A type checker and cycle detector use —
memoized over the acyclic expression DAG. No second cycle semantics, no
re-implementation of expression traversal.

## D-6 Per-use-site checking, never per-expression caching

Use sites are enumerated explicitly (`ExpressionUseSite`: invoke/call_subflow
inputs, await correlation, select selector, foreach source, carry initials,
while guard, timeout duration, workflow outputs) — no reflection over
`ExpressionId` fields. The same expression may pass inside its branch and
fail outside it (`expression/reuse-per-use-site`), and an expression no use
site references never fails anything (`expression/unused-branch-local-reference`).

## D-7 Two stable error kinds, reasons are diagnostics

Exactly two new kinds: `value_not_available` and `resource_not_visible`
(plus nothing else). The diagnostic detail names slot, use site, producer
and an internal reason token (`before_producer`, `cross_branch`,
`outside_owner_boundary`, `iterate_local_outside`,
`parent_output_not_yet_available`). Reasons are frozen in the corpus as
diagnostics but are explicitly not ABI; the kind is the contract. No
`resource_shared_in_parallel` error exists — parallel branches may share a
lexically visible resource.

## D-8 Reason classification is structural

For a missing computation output the analyzer classifies by ownership
relationship: self → `before_producer`; strict ancestor of the use →
`parent_output_not_yet_available`; strict descendant of the use →
`before_producer`; otherwise by the lowest common ancestor — isolated
branch wrapper with different sides → `cross_branch`; SEQUENCE with a
**direct-child** producer later in list order → `before_producer`; anything
else (grandchild escapes, cross-wrapper reads) → `outside_owner_boundary`.
Loop locals are always `iterate_local_outside`; workflow-output misses are
`iterate_local_outside` for loop locals and `outside_owner_boundary`
otherwise.

## D-9 Loop locals and the workflow exit boundary

Foreach `source` and every carry `initial` evaluate from the same pre-loop
environment (so canonical carry order by slot id never becomes evaluation
order); foreach body env = `E ∪ {item} ∪ carries`; while guard/body env =
`E ∪ carries`; while guard cannot read a body output. `WorkflowExitValues`
= run parameters ∪ `Exports(root)` — workflow outputs cannot read root
descendants. No FailureValue, no failure slot, no phi/merge construct:
wrapper output slots are the merge boundaries.

## D-10 Resources: attenuation, not union

`EntryResources(root)` = every declared slot; every opcode but SCOPE
inherits unchanged; a SCOPE's child sees exactly the listed set, which must
be a subset of the inherited environment (`resource_not_visible`
otherwise); narrowing ends at the scope boundary. This makes nested-scope
reintroduction structurally impossible without a dedicated rule. Slot
existence stays Phase 3B (`unknown_resource_slot`); visibility is Phase 3C.

## D-11 Corpus layout, transport and the missing digest

New `spec/conformance/semantic-dataflow/{positive,negative}/` (sibling of
`semantic-ir/`, matching the existing directory style) generated by the new
independent `tools/gen-semantic-dataflow-corpus.py` at corpus revision 1.
The corpus carries **no digest** on purpose: the analysis is derived-only
and `SemanticIrDigest = Hash(IR only)` is unchanged. Positives freeze the
`EntryValues`/`EntryResources`/`WorkflowExitValues` projections; negatives
freeze kind + reason + `rejected_at: "dataflow"`, and every negative is
re-validated with the Phase 3B validator to prove the failure belongs to
the analysis layer alone. The generator also cross-checks all 22 Phase 3B
golden documents against its availability oracle — the shipped IR corpus
stays analyzable.

## D-12 Independent oracle, shared plumbing

The dataflow generator imports `gen-semantic-ir-corpus` for document
plumbing only (base tables, builders, the Phase 3B validator, the golden
corpus); every dataflow rule — closure, environments, transfers, resource
visibility — is implemented independently in the new file and never calls
Rust. Defaulted table lists are defensively copied so negative mutations
can never pollute the shared Phase 3B bases (the first run caught exactly
such pollution through expressions 16–19 leaking across documents).

## D-13 Registry shape

24 new rules `AF-SEMANTIC-0072..0095` (area SEMANTIC, source
`spec/semantic/semantic-ir.md`), all additive; existing RuleIds verbatim.
spec-check binds one anchor per rule, so the shared subsections got
dedicated anchors (`ir-sequence-future`, `ir-select-isolation`,
`ir-parallel-isolation`, `ir-race-isolation`, `ir-handle-isolation`,
`ir-iterate-source-pre-loop`, `ir-carry-initial-pre-loop`,
`ir-foreach-body-environment`, `ir-while-environment`,
`ir-iterate-local-escape`, `ir-resource-no-widen`). 8 new suites
(`SUITE-SEMANTIC-DATAFLOW-*`). Totals: 153 rules, 24 suites, 14
specifications, 232 ABI rows, 4 AFIDL modules. The semantics
specification's `content_digest` updated with the document edit (expected);
`agentflow.semantic.ir@1`'s schema digest did not move (required).

## D-14 Wire-freeze regression pins

`static_surface.rs` gained four tests: the registry digest of
`semantic-ir.afidl` pinned to the exact Phase 3B hex (spec-check enforces
file == registry, the test pins registry == Phase 3B), exactly 4 AFIDL
modules / 232 ABI rows (100 semantic IR) / 14 HashV1 domains, the canonical
path containing no analysis reference (plus no `DataflowAnalysisDigest`),
and no Phase 3D placeholder (`FailureValue`, `failure_slot`,
`SubflowInterface`, `ContractPort`, `resource_shared_in_parallel`).

## D-15 Deferred to Phase 3D (explicitly)

Component/Subflow contract port validation against materialized contract
ABIs (no temporary JSON contract format was created); the typed
FailureValue runtime ABI; resource concurrency rules; Scope authority beyond
the lexical subset rule; Effect Signature; anything runtime (liveness,
storage, stack frames, SSA phi). The analysis stays static: no runtime
dominance, no journal, no plan.
