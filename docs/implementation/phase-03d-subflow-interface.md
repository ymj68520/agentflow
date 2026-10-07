# Phase 3D — Subflow Interface Derivation: Implementation Report

> Status: Phase 3D complete. CALL_SUBFLOW now statically verifies
> inputs/outputs/resources against the exact-pinned target Semantic IR.
> Normative text: `spec/semantic/semantic-ir.md` section 15; decisions:
> `phase-03d-subflow-interface-decisions.md`.

## 1. SubflowIrUniverse 模型

`SubflowIrUniverse` — explicit, immutable-during-checking, in-memory:
`DigestRefValue(sha256 / semantic-ir / hex) → SubflowTarget { digest,
interface, canonical_bytes }`. `insert(&CheckedIrDocument,
&CompiledModuleSet)` computes the digest **from the target itself**, runs
the Phase 3C analysis as an insert gate (invalid targets are
uninsertable), accepts identical-bytes re-insertion idempotently and
rejects a digest-over-different-bytes collision
(`subflow_target_collision`). Accessors: `target`, `interface`,
`digests` (sorted projection), `len`/`is_empty`.

## 2. Derived SubflowInterface 结构

`SubflowInterface { inputs: Vec<SubflowValuePort>, outputs:
Vec<SubflowValuePort>, resources: Vec<SubflowResourcePort> }`;
`SubflowValuePort { port_no, name(diagnostic), semantic_type }`,
`SubflowResourcePort { port_no, source_resource_slot(diagnostic),
requirement_type: TypeSchemaRef }`. Plain Rust, no serde, absent from
model/canonical/afidl (static-audited).

## 3. Input port 派生规则

Target `run_parameters` sorted by `RunParameterId` ascending → ports
`1..N`; type = structurally expanded `RunParameterDecl.type`; name rides
along as diagnostic metadata only.

## 4. Output port 派生规则

Target `workflow_outputs` in canonical NFC-name byte order → ports
`1..N`; type = structurally expanded `ExpressionDef.result_type`.

## 5. Resource port 派生规则

**Every** declared target `ResourceSlot` (not just used ones) sorted by
`ResourceSlotId` → ports `1..N`; carries the exact
`requirement_type: TypeSchemaRef`. Value types and resource requirement
types stay separate universes.

## 6. ResolvedSemanticType 模型

Materialized as the existing `af_semantic::StructuralType`
(Primitive/Schema/List/MapString/Option). Expanded per document along the
acyclic type DAG by a memoized walk in `subflow.rs`; frozen
`canonical_string()` rendering for diagnostics.

## 7. Cross-document 类型相等规则

Structural only: primitives by keyword, schema leaves by exact
`TypeSchemaRef` (specification_id + type_name + schema_digest),
containers recursively. Local `SemanticTypeId`s never compare numerically
— pinned by `subflow/same-structure-different-local-type-ids`,
`subflow/caller-and-target-use-different-type-table-numbering` (target
type 3 = caller type 19 → compatible) and `input/type-mismatch` (id 5 =
string in target, int64 in caller → rejected). No SemanticTypeDigest, no
new Hash domain.

## 8. CALL_SUBFLOW exact target lookup 规则

Contract (3B: exists, kind=subflow, domain=semantic-ir) → digest must
resolve exactly in the supplied universe, else `unknown_subflow_target`.
No latest fallback, no filesystem/registry/network/deployment lookup;
fail closed. No recursion into the target's own subflow closure; graph
cycles deferred to Resolver/LockGraph.

## 9. input/output/resource exact checking 规则

Per call, ascending computation id: input port set == `1..N` then
per-port structural type; caller output port set == `1..N` then
per-slot structural type (partial contracts impossible); resource port
set == `1..N` then exact requirement equality. A document without
CALL_SUBFLOW passes against an empty universe.

## 10. 新增 error kinds

`unknown_subflow_target`, `subflow_target_collision`,
`subflow_input_port_mismatch`, `subflow_input_type_mismatch`,
`subflow_output_port_mismatch`, `subflow_output_type_mismatch`,
`subflow_resource_port_mismatch`, `subflow_resource_type_mismatch` —
raised only by `check_subflow_interfaces`. Phase 3B/3C kinds untouched.

## 11. 新增 AF-SEMANTIC rules / suites

15 additive rules `AF-SEMANTIC-0096..0110`, 4 suites
`SUITE-SUBFLOW-{EXACT-TARGET, INTERFACE-DERIVATION,
CROSS-DOCUMENT-TYPES, INTERFACE-PASS-ORDER}`. Totals: **168 rules** (was
153), **28 suites** (was 24), 167 covered + 1 intentionally uncovered,
**missing coverage = 0**.

## 12. Positive / Negative corpus 数量

`spec/conformance/subflow-interface/` (revision 1): **12 positive** (the
12 required scenarios, each freezing target digests + interface
projections), **16 negative** (3 target-resolution with shared
`unknown_subflow_target`, 5 input, 4 output, 4 resource), **3
equivalence proofs** (table order, local type renumbering, canonical
port orders). All projections marked NON_NORMATIVE CONFORMANCE
PROJECTION.

## 13. Independent oracle 范围

`tools/gen-subflow-interface-corpus.py` independently implements: type
expansion + rendering, interface derivation, exact-digest universe
lookup, and the full port/type checking with reason-identical error
kinds. It reuses only the allowed plumbing (3B document builder +
validator + digest computation, 3C availability oracle for the
target/caller validity prerequisite) and never calls the Rust checker.
It also computes the frozen v2 fixture digest independently.

## 14. wire / ABI / hash 零漂移证明

`git diff` over `spec/idl/`, `spec/tables/`, `abi-numbers.json`:
**empty**. `semantic-ir.afidl` byte-identical (sha256
`9fb3292a…616a3`, test-pinned to the registry, spec-check-enforced
against the file) → 36 TypeSchemaDigests and 100 semantic-ir ABI rows
cannot move; ABI total stays **232**; HashV1 stays **14** domains; AFIDL
modules stay **4**; no SemanticTypeDigest exists (static audit).

## 15. Phase 3B corpus 零漂移

`spec/conformance/semantic-ir/` diff-empty: 22 golden
bytes/digests, 79 negative kinds, 11+3 equivalence all re-verified green
by the untouched Phase 3B harness.

## 16. Phase 3C corpus 零漂移

`spec/conformance/semantic-dataflow/` diff-empty: 20 positive
observations and 37 negative kind/reason expectations re-verified green.

## 17. spec-check 结果

exit 0: specifications 14 · rules 168 · covered 167 · intentionally
uncovered 1 · missing coverage 0 · ABI active 232 · AFIDL modules 4 ·
cross-reference errors 0. Only `agentflow.semantic.ir.semantics@1`'s
source digest updated (document edit), as required.

## 18. Workspace 门禁结果

`cargo fmt --all -- --check` pass; `cargo clippy --all-targets
--all-features -- -D warnings` clean; `cargo test --all` **197 passed /
0 failed** (190 → 197: +6 subflow conformance, +1 wire-surface audit
rework; no previous count hardcoded); all **8** generators run twice →
zero diff.

## 19. Explicit deferred 项（Phase 4）

ComponentContract core (complete I/O, Φ, ε, R, F, U, K, X plus
replay/idempotency/timeout/recovery policies) and INVOKE exact contract
checking; Resolver/LockGraph (subflow dependency closure, published-graph
acyclicity, cycle policy); FailureValue runtime ABI; Effect Signature;
Resource Contract meaning/concurrency; Execution Requirements; runtime
layers. No ports-only ComponentContract placeholder was created and no
fake component universe exists.

## 20. 是否存在阻塞 Phase 4A 的问题

None identified. The contract table, the derived-interface machinery and
the structural type comparison are the natural substrate for the Phase 4
contract core; the subflow universe demonstrates the explicit-input
pattern the resolver will generalize.

## 21. Git commits / diff 概览

```text
ca91897 feat(af-ir): subflow interface derivation and CALL_SUBFLOW checking
02be3bb test(subflow-interface): freeze the Phase 3D conformance corpus
aa5ea32 test(af-ir): scope the wire-freeze audit for the derived subflow model
295218e spec(registry): register the 15 subflow-interface rules and suites
<docs>  docs(implementation): phase-03d decision register and report
```

New files: `crates/af-ir/src/subflow.rs` (~660 lines),
`crates/af-ir/tests/conformance_subflow_interface.rs` (6 tests),
`tools/gen-subflow-interface-corpus.py`,
`spec/conformance/subflow-interface/{positive,negative,equivalence}/`,
`docs/implementation/phase-03d-*.md`. Modified: `error.rs` (+8 kinds),
`lib.rs` (module + exports + doc), `static_surface.rs` (audit rescoped),
`semantic-ir.md` (section 15; deferred → Phase 4; conformance +1
paragraph), `gen-registry-instances.py` (+15 rules/+4 suites),
regenerated registry JSONs.
