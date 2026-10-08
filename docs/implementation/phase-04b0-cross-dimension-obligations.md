# Phase 4B.0 — Cross-dimension Proof Obligations

This register describes the future assembly checks. “Required result” is the
result a safe checker must preserve; it is not a claim that the current crates
already implement that check.

| ID | Link | Required static check | Current materialization | Required unresolved result |
| --- | --- | --- | --- | --- |
| C01 | Type table ↔ I/O | Resolve each public port type pin exactly; expand aliases and containers to `StructuralType`; reject unknown/stale schema pins | `TypeSchemaUniverse`, `StructuralType`, `SubflowInterface` exist; no Component port model | Unproven until the explicit type universe and port pins resolve |
| C02 | I ↔ F | Every `workflow_input(port)` source and every ε input guard resolves to a Component I port in the I namespace | F checks positive port syntax; ε checks positive guard syntax; neither has Component I | Violation for a known absent port; Unproven if the I interface is unavailable |
| C03 | O ↔ F | Every `workflow_output(port)` sink resolves to a Component O port | F checks positive syntax only | Violation if a closed O interface lacks the port |
| C04 | R ↔ F | Every `resource_requirement(port)` source resolves to a declared R port | F checks positive syntax only | Violation if a closed R row lacks the port |
| C05 | ε ↔ R | Resolve each selector to R, then separately prove exact operation/resource support, selector vocabulary/value semantics, qualifiers, and consistency with R predicates | B0B-D4 designs an exhaustive exact Operation×ResourceType catalog; checker not implemented | Missing R port or explicit denial is Violation; missing/partial catalog or open ε is Unproven |
| C06 | ε ↔ F | Resolve selector-sensitive permissions to complete EffectAtom keys; legacy operation sink expands to every matching closed ε atom unless narrowed | B0B-D5 adds `EffectFlowPartition(operation,kind,allowed_effects)` without changing old F schema | Dangling/widening partition is Violation; missing selector-specific policy or open ε is Unproven |
| C07 | F ↔ X | Map exact labels/taints/authority/sinks to allowed region pairs, trust policy and sandbox floor | B0B-D8 designs an explicit versioned relation catalog; checker not implemented | Exact known conflict is Violation; missing mapping, snapshot or finite region coverage is Unproven |
| C08 | K ↔ ε | Every recoverable forward EffectAtom has exactly one K clause; no clause exists outside ε; open/unknown ε is not full coverage | Exact AtomKey coverage exists for Closed rows; the Phase 4B.0 qualifier-member omission was closed in Phase 4B.0A | Full atom equality for closed rows; Open/Unknown returns Unproven |
| C09 | K actions ↔ ε | Bind `(forward Atom, action kind, full action Atom)`, require exactly the actions prescribed by K, and require all actions in closed ε | B0B-D6 accepts additive `RecoveryActionBinding`; checker not implemented | Action absent from closed ε/extra binding/cycle is Violation; open ε is Unproven |
| C10 | K ↔ U | Compute per-action recovery usage from explicit meter profiles, activation and attempt bounds; accumulate and compare with U.recovery | B0B-D7 adds per-action profiles and a complete-meter coverage catalog; checker not implemented | Known ceiling excess is Violation; missing profile/coordinate/multiplicity is Unproven, never zero |
| C11 | U ↔ Cost | Project every known U coordinate through an exact immutable price snapshot; check forward/recovery ceilings independently | `project_cost`, `check_static_budget`, `cost_not_increased` exist | Missing usage/price or changed snapshot returns Unproven; never infer free |
| C12 | R ↔ X | Establish whether the chosen execution environment can bind/access the logical R resources under the required trust/residency constraints | R universe knows resource type and attribute vocabulary; X knows executor requirements; no binding/adapter relation | Unproven without deployment/adapter profile; resource satisfaction is not authorization |
| C13 | Φ ↔ Control | Compose known failure rows through controls; preserve unknown INVOKE/CALL_SUBFLOW rows; do not treat empty as a missing summary | `af-failure` has pure row/control helpers and `StaticFailureEnvelope::Unproven`; no IR-wide resolved pass | Unproven when a callee contract or full target closure is absent |
| C14 | ε ↔ Control | Compose upper bounds; preserve RACE loser effects and retry amplification; never use Recovery to narrow without evidence | Effect algebra/control functions exist but are standalone and do not traverse IR | Unproven for calls/subflows without ε; missing control transfer rule cannot be treated pure |
| C15 | U ↔ Control | Compose additive/join/repeat rules; preserve loser/retry/recovery usage and unknown coordinates | Usage control helpers exist, including unknown INVOKE/CALL_SUBFLOW | Unproven for call/subflow without U; ITERATE needs total count, not `max_inflight` |
| C16 | F ↔ dataflow | Derive may-influence sources/taints/authority from all possible input and resource reads; preserve unknown external provenance | `af-flow` propagators exist; `af-ir` has independent dataflow availability analysis; no bridge joins them | Unproven if source coverage or taint propagation is incomplete |
| C17 | X ↔ Control | Keep X scoped to the relevant computation/attempt; do not sum sequential/parallel child requirements onto one worker absent a topology contract | Phase 4A.7 explicitly forbids blind aggregation | No group/affinity/gang claim; per-computation profile proofs only |
| C18 | Universes ↔ proof | Bind each proof to exact immutable universe/catalog snapshots and rules; do not query latest | Existing APIs are explicit in-memory inputs; future report carries snapshot revisions and checksums | Proof remains conditional; checksum identity is not source authentication |
| C19 | Semantic types ↔ ABI pins | Public Component ports compare exact TypeSchemaRef pins; alias expansion is internal only | B0B-D3 accepts nominal public equality; Phase 3D is the precedent | Different pins remain Unproven; malformed/cyclic schema is Violation |

## Assembly pass order

The proposed pure checker can be staged without changing the existing
dimensions:

1. **Structural and canonical input.** Canonicalize the contract fields and
   validate closed port sets. Reject malformed numbers, duplicates and
   invalid identities before attempting cross-links.
2. **Exact universe resolution.** Resolve TypeSchemaRef, ResourceType,
   EffectOperation, labels, meters, currencies, trust, region and runtime
   identities against explicit immutable inputs.
3. **Port namespace map.** Build typed maps for I, O and R separately; map
   `InvokeBody`, `ComputationOutputBinding`, `ResourceUseBinding`, Flow
   references, effect selectors and Subflow derived ports through the correct
   map.
4. **Local declared-row checks.** Check each Effect selector→R port, each
   Flow source/sink→I/O/R/ε reference, and each K forward/action Effect→ε.
5. **Cross-catalog semantic checks.** Apply explicit operation/resource,
   label/residency, influence/trust/sandbox, recovery-usage and resource/X
   relations. Missing relations return Unproven.
6. **Control-derived summaries.** Traverse the structured IR and compose Φ,
   ε, F, U and K only with their defined rules. Each unresolved callee keeps
   that dimension Unproven. X is checked per computation unless a formal
   placement topology is supplied.
7. **Static proof aggregation.** Return per-dimension and per-obligation
   results with assumptions and reasons. Never convert unknown/missing rows
   into empty rows.
8. **External evidence gates.** A separate attestation/admission layer may
   check publishers, implementation behavior, sandbox attestation, policy,
   current resources and budgets. Those are not static contract proofs.

## Control propagation coverage

This table records which standalone algebra helpers exist. “Missing” means
the full IR-level summary is not materialized; it does not mean the dimension
is empty.

| IR form | Φ | ε | R | F | U | K | X |
| --- | --- | --- | --- | --- | --- | --- | --- |
| INVOKE | Unproven until ComponentContract | Unproven until component ε | Call-site resource mapping only; no contract R check | No bridge from component flow to call | `invoke_envelope(None)` is Unknown | No resolved target action coverage | Check this computation's X only; no implementation attestation |
| AWAIT | `await_row()` = empty business Failure row | No dedicated await helper | No R composition | No F summary composition | `await_envelope` preserves supplied envelope | No K composition | No worker-group aggregation |
| SEQUENCE | `sequence_row` unions | `sequence_envelope` accumulates | No R composition/map | No F summary composition | `sequence_envelope` accumulates | No K composition | Keep each invocation's X local |
| SELECT | `select_row` unions | `select_envelope` joins | No R port map | No per-case F join | `select_envelope` joins | No K join | No cross-branch X aggregate |
| PARALLEL | `parallel_row` unions | `parallel_envelope` accumulates | No gang/resource aggregation | No branch flow summary | `parallel_envelope` accumulates | No per-branch K composition | Explicitly no single-worker sum |
| RACE | `race_row` keeps all branch failures | `race_envelope` accumulates every contestant | No R aggregate | No race flow summary | `race_envelope` keeps loser usage | No recovery relation to winner/losers | No single-worker aggregate |
| ITERATE | Body row unchanged | `iterate_envelope(repetitions, body)` | No resource placement model | No loop flow composition | Known total repeats multiply; unknown stays Unknown; `max_inflight` is not count | No recovery-per-iteration coverage | No whole-loop placement aggregation |
| RETRY | Child row preserved | Retry amplifies effects; narrowing has explicit Recovery obligation | No repeated resource relation | No retry flow summary | Retry scales usage by attempts | K is not composed with a retry tree | Per-attempt requirement only; no aggregate |
| TIMEOUT | Child row preserved; timeout is not Φ | No helper; child effects remain possible | No R composition | No timeout flow summary | `timeout_envelope` preserves usage | No timeout-specific K composition | No placement theorem from timeout |
| HANDLE | `(child \ handled) ∪ handler rows` | No helper for protected+handler effect composition | No R composition | No handler flow summary | Protected+handlers accumulate conservatively | No recovery handler composition | No placement aggregate |
| SCOPE | Child row passes through | No helper; no effect subtraction | IR resource-slot visibility narrows lexically; no R row aggregation | No F scope projection | Child usage passes through | No K scope transform | No profile aggregation |
| COMPENSATION_SCOPE | Forward row preserved | Forward + compensation envelope exists | No R relation for compensation actions | No recovery flow summary | Forward + recovery usage exists; no refund | Clause analysis exists, not a full recovery tree walk | No placement/group aggregate |
| CALL_SUBFLOW | Unproven without target contract | Unproven without target contract | Phase 3D derives exact Subflow R ports only | No transitive flow summary | `call_subflow_envelope(None)` is Unknown | No transitive recovery summary | X intentionally not aggregated from target tree |

An empty row from a helper is meaningful only where its normative rule says
the operation introduces no such member (for example AWAIT's business Failure
row). A missing call summary, missing total iteration count, unknown meter or
unknown effect is not equivalent to that mathematical identity.

## Catalog relations needed before a full proof

Phase 4B.0B accepts these exact, typed analysis relations (all provisional and
unregistered):

* `EffectResourceCompatibilityCatalog`: exact EffectOperationRef ×
  ResourceTypeRef, closed selector-key/value rules, complete-operation
  coverage, and exact complete qualifier allowlists.
* `EffectFlowPartition`: complete exact-effect allowlist per legacy operation
  sink and FlowKind; it may only narrow F.
* `RecoveryActionBinding`: exact forward EffectAtom, action kind and recovery
  EffectAtom; all actions must be in closed ε.
* `RecoveryUsageCoverageCatalog`: exact RecoveryActionIdentity to exhaustive
  meter-coordinate set; per-attempt bounds and multiplicities are declared in
  the ComponentContract binding.
* `InformationFlowExecutionCatalog`: exact confidentiality/taint/authority/
  sink mappings to region pairs, trust policies and sandbox floors.
* `ResourceExecutionCompatibility`: exact logical ResourceType and adapter
  profile relation to runtime capabilities, trust, region and residency.

All relation snapshots are immutable, exact-versioned proof inputs; their
identities/checksums are recorded in static proof output. They must not create
an EffectGrant, ResourceHandle, capacity permit, attestation or runtime
evidence. C12 Resource↔X adapter/access relation remains a P2 implementation
boundary outside the eight P1 design decisions.

## Unified Computation view

The static view is a projection with proof status per field; it is not a new
wire object and it never fills an unavailable field with an empty row.

| Computation kind | Source of facts | Derivable today | Missing proof / closure |
| --- | --- | --- | --- |
| Component call | Exact ComponentContract digest selected by the caller | Nothing complete today; the contract object and resolver are absent | Once materialized, all I/O/Φ/ε/R/F/U/K/X fields come from that exact contract; implementation and runtime evidence remain separate |
| Subflow | Exact pinned SemanticIrDigest and its Semantic IR tables | Phase 3D derives I/O and R ports; Phase 3B/3C validate structure/dataflow | Φ/ε/F/U/K/X require the full pinned contract closure for every INVOKE/CALL_SUBFLOW. Phase 3D deliberately does not traverse transitive targets or decide subflow cycles; Resolver/LockGraph remains necessary |
| Control-derived computation | Canonical Semantic IR child tree plus per-child summaries | I/O interfaces and standalone Φ/ε/U control algebra helpers exist | No single IR traversal composes all dimensions. R mappings, F may-influence/control ceilings, K recovery coverage and call/subflow summaries are missing. X remains per computation unless explicit group topology exists |

The future projection should carry `Proven/Violation/Unproven/RequiresEvidence`
for each dimension and child reference. A missing Component contract, Subflow
target, catalog relation or child summary is `Unproven`; it is not `Φ=∅`,
`ε=∅`, empty F, zero U, no K obligation, or unrestricted X. Any known hard
constraint violation remains a Violation even if another dimension is
Unproven.

## Phase 4B.0B disposition

P1-1 through P1-8 now have accepted design records. Their future validators,
relation-catalog loaders, Semantic IR traversal, Resource↔X adapter proof,
combined proof report, and evidence integration are not implemented here;
those are explicit P2/P3 work. The accepted models require exact typed
bindings and snapshots before those validators may return Proven. No
catalog/port/selector/action identity is guessed from a matching integer or
short name.

## Related audit documents

[Audit report](phase-04b0-preassembly-audit.md) ·
[Field matrix](phase-04b0-field-matrix.md) ·
[Port namespaces](phase-04b0-port-namespace-matrix.md) ·
[Compatibility](phase-04b0-compatibility-matrix.md) ·
[Counterexamples](phase-04b0-counterexamples.md) ·
[ABI proposal](phase-04b0-abi-proposal.md) ·
[P1 register](phase-04b0b-p1-decisions.md) ·
[Proof API](phase-04b0b-proof-api.md) ·
[Readiness report](phase-04b0b-readiness-report.md)
