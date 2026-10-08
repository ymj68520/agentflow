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
| C05 | ε ↔ R | Every EffectAtom selector resolves to the correct R port, and the operation is valid for the pinned resource type and scope vocabulary | `requirement_port_no` and R `port_no` exist, but no cross-check or operation/resource relation exists | Absent R port is Violation; absent catalog relation is Unproven |
| C06 | ε ↔ F | Each F external-effect sink resolves to the intended operation and resource selector of ε | F stores only `EffectOperationRef`; ε stores full Atom | P1: current F cannot distinguish two selectors for one operation; Unproven for selector-specific policy |
| C07 | F ↔ X | Check residency restrictions against execution/residency regions, and influence/confidentiality against trust/sandbox | F labels and X regions/trust/sandbox are separate explicit vocabularies; no relation catalog | Unproven without a trusted, exact catalog relation; never infer by region/label names |
| C08 | K ↔ ε | Every recoverable forward EffectAtom has exactly one K clause; no clause exists outside ε; open/unknown ε is not full coverage | Exact AtomKey coverage exists for Closed rows; P0 AtomKey omits qualifier member value | After P0 repair, full atom equality; Open/Unknown returns Unproven |
| C09 | K actions ↔ ε | Reconciliation and compensation are exact declared Effect actions, including selector and qualifiers | Reconciliation is a boolean enum; compensation is only OperationRef | P1: redesign K action references and prove each action against ε |
| C10 | K ↔ U | Derive the maximum recovery usage envelope for reconciliation/compensation/retry and compare against the recovery U coordinates | U has forward/recovery phases and control algebra; no mapping from K actions to usage profiles | P1: missing recovery meter/cost coverage is Unproven, never zero/free |
| C11 | U ↔ Cost | Project every known U coordinate through an exact immutable price snapshot; check forward/recovery ceilings independently | `project_cost`, `check_static_budget`, `cost_not_increased` exist | Missing usage/price or changed snapshot returns Unproven; never infer free |
| C12 | R ↔ X | Establish whether the chosen execution environment can bind/access the logical R resources under the required trust/residency constraints | R universe knows resource type and attribute vocabulary; X knows executor requirements; no binding/adapter relation | Unproven without deployment/adapter profile; resource satisfaction is not authorization |
| C13 | Φ ↔ Control | Compose known failure rows through controls; preserve unknown INVOKE/CALL_SUBFLOW rows; do not treat empty as a missing summary | `af-failure` has pure row/control helpers and `StaticFailureEnvelope::Unproven`; no IR-wide resolved pass | Unproven when a callee contract or full target closure is absent |
| C14 | ε ↔ Control | Compose upper bounds; preserve RACE loser effects and retry amplification; never use Recovery to narrow without evidence | Effect algebra/control functions exist but are standalone and do not traverse IR | Unproven for calls/subflows without ε; missing control transfer rule cannot be treated pure |
| C15 | U ↔ Control | Compose additive/join/repeat rules; preserve loser/retry/recovery usage and unknown coordinates | Usage control helpers exist, including unknown INVOKE/CALL_SUBFLOW | Unproven for call/subflow without U; ITERATE needs total count, not `max_inflight` |
| C16 | F ↔ dataflow | Derive may-influence sources/taints/authority from all possible input and resource reads; preserve unknown external provenance | `af-flow` propagators exist; `af-ir` has independent dataflow availability analysis; no bridge joins them | Unproven if source coverage or taint propagation is incomplete |
| C17 | X ↔ Control | Keep X scoped to the relevant computation/attempt; do not sum sequential/parallel child requirements onto one worker absent a topology contract | Phase 4A.7 explicitly forbids blind aggregation | No group/affinity/gang claim; per-computation profile proofs only |
| C18 | Universes ↔ proof | Bind each proof to exact, immutable, explicit universe/profile inputs and rule version; do not query latest | APIs take explicit in-memory universes, mostly keyed by SpecificationId or exact schema ref | Current result is conditional on caller-supplied facts; proof snapshot/evidence object is not materialized |
| C19 | Semantic types ↔ ABI pins | Choose whether public schema leaves preserve exact TypeSchemaRef identity or expand aliases to their targets | Phase 3A `TypeSchemaUniverse::expand_named` collapses aliases; Phase 3D `SubflowInterface` preserves pinned schema leaves and rejects alias-vs-target | Unproven across that boundary until one comparator is normative |

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

The minimum additional versioned, explicit relations are:

* `OperationResourceCompatibility`: exact EffectOperationRef, exact
  ResourceTypeRef, accepted scope-binding vocabulary, and any allowed
  qualifier semantics. An operation name alone is not enough.
* `FlowEffectTarget`: exact operation plus selector scope/requirement port
  identity. F permissions need to name the destination granularity they
  constrain.
* `InformationHandlingProfile`: exact confidentiality/taint identities
  and the X region/trust/sandbox requirements that satisfy them. No legal
  inference from names or geography is allowed.
* `RecoveryActionUsageProfile`: exact reconciliation/compensation Effect
  identity and its maximum recovery meter envelope, including bounded
  retries. A missing profile is Unproven.
* `ResourceExecutionCompatibility`: exact logical ResourceType and adapter
  profile relation to runtime capabilities, trust, region and residency.

These relations should be immutable, exact-versioned inputs and their source
identities should be recorded in static proof output. They must not create an
EffectGrant, ResourceHandle, capacity permit or runtime evidence.

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

## Related audit documents

[Audit report](phase-04b0-preassembly-audit.md) ·
[Field matrix](phase-04b0-field-matrix.md) ·
[Port namespaces](phase-04b0-port-namespace-matrix.md) ·
[Compatibility](phase-04b0-compatibility-matrix.md) ·
[Counterexamples](phase-04b0-counterexamples.md) ·
[ABI proposal](phase-04b0-abi-proposal.md) ·
[Decisions](phase-04b0-decisions.md)
