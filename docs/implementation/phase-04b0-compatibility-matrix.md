# Phase 4B.0 — Full Contract Compatibility Matrix

The proposed composite result is a per-dimension proof record, never a single
`compatible: bool`. The required substitution direction follows the frozen
architecture formulas. Existing APIs cover some rows independently; none
currently evaluates this complete matrix.

| Dimension | Replacement condition / proof direction | Existing API and representable outcomes | Proven cases | Unproven cases | Violation cases | Assumptions / evidence | Cross-dimension obligations |
| --- | --- | --- | --- | --- | --- | --- | --- |
| I — inputs | `I_old <: I_new`: the new component accepts every old input | No Component API. `StructuralType` exact Eq; Subflow checks expanded structures while preserving schema leaves | Same port set and equal type identity under one chosen alias policy | Any unequal structure without subtype/variance; alias-vs-target under the unresolved Phase 3A/3D policy | A concrete old call value that the new closed interface rejects, once presence/type rules are frozen | Exact TypeSchemaUniverse; typed port mapping; no implicit coercion | F workflow_input and ε guard input must resolve to I |
| O — outputs | `O_new <: O_old`: every new output fits the old expected type | No Component API. IR/Subflow interface checks compare exact structures with their own alias behavior | Same output set and exact type identity | Different structure/schema pin, alias-vs-target policy, or unresolved port mapping | Missing required old output or a proven value/type mismatch | Exact type pins and closed output-port semantics | F workflow_output sinks must resolve to O; Control parent interfaces consume O |
| Φ — failure row | `Φ_new ⊆ Φ_old` | `af_failure::contains`; boolean exact membership; `StaticFailureEnvelope::{Proven,Unproven}` for incomplete call summaries | Every new failure identity occurs in old | Target/call envelope unavailable; invalid/unresolved target pin | New known failure identity absent from old row | Exact TypeSchemaRef identity; full component/subflow target | HANDLE residuals and handlers must aggregate; timeout/cancel remain Control outcomes |
| ε — effects | `ε_new ⊑ ε_old` | `af_effect::contains` / `require_contained_in`; `Proof::{Proven,Unproven}` only | Exact operation/selector/complete typed qualifier identity, narrower scope, lower occurrence bounds, permitted guard implication | Unknown/open residual, uncovered entry, catalog relation absent | No dedicated `Violation` arm; known uncovered effects currently surface as Unproven | Explicit operation catalog; full AtomKey (qualifier collision fixed in 4B.0A); closed or bounded-open envelopes | Selector→R; input guards→I; F sink must cover intended selectors; K exact effects |
| R — resources | New accepts at least all old candidates, per stable R mapping | `requirement_not_stronger(new, same namespace, old, same namespace, universe)` | Same type pin; new exact-attribute set is a subset of old on each retained port; removal alone does not strengthen R | Retained resource type changes; unknown new attribute semantics; namespaces differ | New R port or additional/changed required attribute | Explicit ResourceTypeUniverse and stable port mapping | ε selectors and F resource sources must resolve; deployment binding stays separate |
| F — information flow | `F_new` must not relax `F_old` | `flow_not_relaxed`; `FlowCompatibilityProof::{Proven,Unproven}`, relaxed ceiling/restriction is an error | Restrictions retained/strengthened, per-kind sink sets narrowed, authority not elevated, taint/source capacity not narrowed | Unknown repaired, taint/source deletion, incomparable influence facts | Dropped confidentiality restriction, new permitted sink, authority elevation | Exact label/sink catalogs; complete I/O/R/ε references | Operation sink must carry enough selector identity; F↔X relation needs a trusted mapping |
| U — usage | No unauthorized usage increase, by exact `(phase,meter)` | `usage_not_increased`; `UsageProof::{Proven,Violation,Unproven}` | Exact meter and phase coordinates, compatible unit/aggregation, each finite ceiling no larger | Missing coordinate on either side, changed unit/aggregation, unknown bound | New finite ceiling larger or finite→unbounded | Explicit UsageMeterUniverses on both sides | K recovery actions must have meter coverage; Control-derived calls/subflows cannot be empty |
| Cost analysis / Run budget | Cost is not a ComponentContract compatibility dimension by itself. Optional derived cost non-increase compares projections only under the same full price snapshot; StaticCostCeiling is checked against a Run-local budget input | `cost_not_increased`; `CostProof::{Proven,Violation,Unproven}`; separate `check_static_budget` | Same currency and complete price snapshot, new forward/recovery projections no greater | Different price snapshot/currency, unknown usage, missing price | Higher comparable projection is a cost non-increase violation; budget ceiling exceeded is a separate Run-budget violation | Explicit CurrencyUniverse, UsageMeterUniverse, immutable price snapshot and Run budget | Cost projection is analysis only; neither check reserves money, billing or quota |
| K — recovery | Recovery guarantees must not weaken; no invented ordinal ranking | `recovery_not_weakened`; `RecoveryCompatibilityProof::{Proven,Weakened,Unproven}`; `analyze_recovery_safety` adds RequiresEvidence/Violation | Exact canonical RecoveryContract equality only | Changed mode/evidence/action details; open/unknown effect coverage; missing action identity | Removed clause yields Weakened; unsafe NONE+UNKNOWN can be a violation | Exact ε AtomKey, provider/key/reconciliation evidence supplied explicitly | Reconciliation/compensation actions must be in ε and their usage in U |
| X — execution | Both old-eligible⊆new-eligible feasibility and security preservation must prove | `execution_compatibility`; independent `feasibility` and `security` verdicts; combined result | Same requirements; proven relaxations only when security is independently preserved | Capability version changes, runtime/ABI changes, incomparable region/trust relations | New hard requirement, weakened sandbox/trust/residency constraint | Explicit capability/trust/region/runtime universes; trusted profile descriptor assumption | F residency/influence and R resource access need explicit relation; no reservation/admission |

## I/O compatibility decision table

The table applies to the closed Component interface proposal. Every listed
input is required; an `option<T>` value is still a required port whose value
may be None. There are no defaulted, variadic or silently ignored ports.

| Old I / O | New I / O | Input result | Output result | Reason |
| --- | --- | --- | --- | --- |
| Same numbered ports and exact pinned types | Same numbered ports and exact pinned types | Proven | Proven | Equal closed interface is the only general case currently supported |
| Old I port is absent from new I | New component cannot receive an input required by the old caller | Violation | — | Old call domain is not accepted |
| New I has an additional required port | Old caller does not provide the required value | Violation | — | Required input domain narrowed |
| Same input port number, different TypeSchemaRef | Relation unknown | Unproven | — | No subtype, variance or coercion rule |
| Old O port is absent from new O | — | — | Violation | New component cannot supply an old required result |
| New O adds a port | — | — | Unproven | Whether callers may ignore extra outputs is not materialized in INVOKE ABI |
| Same O number, different TypeSchemaRef | — | — | Unproven | No result covariance/subtyping relation is materialized |
| Port number changes but name stays the same | No implicit mapping | Unproven | Unproven | Name is diagnostic, and port ordinal is the binding coordinate |
| Alias TypeSchemaRef changes to its target or another alias | No automatic equivalence | Unproven | Unproven | Component compatibility uses exact pins unless a later rule proves more |

The architecture's general `I_old <: I_new` / `O_new <: O_old` direction remains
the target relation. This table exposes that current schemas do not define the
subtype relation needed to prove that direction beyond exact interface
equality and obvious closed-port failures.

## Outcome vocabulary

The composite proof should retain these states per dimension:

* **Proven** — the named static relation is proven under named exact inputs.
* **Violation** — a closed rule proves the relation false.
* **Unproven** — available facts do not establish either safe substitution or
  a concrete violation.
* **RequiresEvidence** — the declaration is structurally acceptable but a
  named independent evidence item is needed (for example K idempotency or
  reconciliation evidence).

`RequiresEvidence` is not a compatibility pass. A future aggregate is Proven
only when every mandatory dimension is Proven and every required evidence item
is separately satisfied under its own authority. `Violation` must not be
collapsed into `Unproven`, and `Unproven` must never be upgraded by a default.

## Whole-contract rule

The necessary conditions are conjunctive:

```text
I_old <: I_new
AND O_new <: O_old
AND Φ_new ⊆ Φ_old
AND ε_new ⊑ ε_old
AND R_new does not strengthen R_old
AND F_new does not relax F_old
AND U_new does not increase U_old
AND K_new does not weaken K_old
AND X feasibility proven
AND X security preservation proven
AND every cross-dimension reference and relation proven
```

The comparison also requires an explicit old→new port mapping when the
contract identity or port coordinates change. Matching port numbers across
different Component identities does not provide that mapping.

Publisher declarations, contract tests and eval gates create separate
Compatibility Attestation evidence. They do not change the static relation or
turn an unsupported subtype/catalog relation into a normative proof.

## Related audit documents

[Audit report](phase-04b0-preassembly-audit.md) ·
[Field matrix](phase-04b0-field-matrix.md) ·
[Port namespaces](phase-04b0-port-namespace-matrix.md) ·
[Cross-dimension obligations](phase-04b0-cross-dimension-obligations.md) ·
[Counterexamples](phase-04b0-counterexamples.md) ·
[ABI proposal](phase-04b0-abi-proposal.md) ·
[Decisions](phase-04b0-decisions.md)
