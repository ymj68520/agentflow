# Phase 4B.2D — Root Export Cross-Dimension Proof Rules

All candidate rules are **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED ABI / NOT IMPLEMENTED**. They compare a checked root derivation with one exact checked exported ComponentContract only after typed coordinate mapping succeeds.

## Global proof rule

No root summary is rewrapped as a ComponentContract. For each dimension the report retains its own proof, exact source references, and prerequisites. The summary verdict precedence remains Violation, RequiresEvidence, Unproven, Proven; choosing an overall value never deletes lower-priority diagnostics.

The selected directions are implementation-within-export:

~~~text
Root required inputs       no stronger than Export I
Root guaranteed outputs    satisfy Export O
Root possible failures     ⊆ Export Phi
Root possible effects      ⊆ Export epsilon
Root resource requirements no stronger than Export R
Root information flow      does not relax Export F
Root forward/recovery U    ≤ Export U, separately by phase and meter
Root K safety              does not weaken Export K
Every child X              satisfies Export X feasibility and security separately
~~~

The exact v1 positive subset uses exact nominal pins and established relations. Unsupported variance is Unproven; no field-name or enum-number ordering is invented.

## Dimension matrix

| Dimension | Preconditions and proof direction | Proven | Violation | Unproven | RequiresEvidence |
| --- | --- | --- | --- | --- | --- |
| **I** | Bijection between root RunParameters and exported input ports; exact source ValueSlot, typed use sites, and exact resolved TypeSchemaRef. Root must not add required public inputs. | Every export port is supplied once, every root RunParameter is advertised once, and all types are exact. | Missing export input, extra root required input, duplicate source alias that forces independent export inputs equal, or a proven incompatible input type. | Structural/schema relation or transformed expression cannot prove exact compatibility; mapping or producer proof missing. | Actual caller value validation/receipt is outside static interface proof. |
| **O** | Every exported output maps to a named root WorkflowOutput and exact expression; Phase 3C availability, dominance, producer, and exact nominal type evidence. | Every export output is produced exactly once with exact TypeSchemaRef and valid scope. | Missing output, invalid producer/dominance, or proven incompatible type. | Extra unmapped root output (no frozen additive rule), uncertain branch/interface, unresolved type alias or producer. | Actual output value/content or implementation behavior requires execution/body evidence. |
| **Phi** | Compare exact pinned TypeSchemaRef identities after output/root coordinates are irrelevant; root possible Failure set must fit export. | Every possible root failure is included in the export row; complete closed rows. | A known root Failure is outside the export row. | Unknown/open/incomplete root or export row; unresolved external Failure source. | Actual Failure occurrence/outcome requires runtime evidence, not static containment. |
| **epsilon** | Exact full EffectAtom/resource mapping, guard facts, occurrence bounds, openness, residuals, and K action projection. Prove Root epsilon ⊑ Export epsilon and separately Root K action upper/guard ⊆ Root epsilon and Export epsilon. | Every forward and K action possibility is covered by exact atom, scope, qualifier, bound, and guard ceilings; root E fits export E. | Known atom/guard/bound outside a closed export ceiling, missing action atom, or typed guard countermodel. | Open/unknown effects, missing resource/guard map, unsupported guard implication, or incomplete action-source closure. | Concrete receipt/action eligibility is RequiresEvidence and does not erase the static E obligation. |
| **R** | Map each root ResourceSlot to exact exported ResourceRequirement port; exact TypeSchemaRef/ResourceTypeRef, attributes, selector constraints, aliases, and Scope visibility. Root requirements must not strengthen export. | Existing resource compatibility relation proves the mapped root requirements are not stronger. | Exact resource type/attribute predicate is proven stronger or slot is outside lexical visibility. | Missing relation row, ambiguous alias, missing attribute evidence, or unsupported constraint variance. | Resource acquisition, handle, binding, or authorization is not established by R. |
| **F** | Map exact FlowSource/FlowSink and EffectFlowPartition identities through I/O/R/E mappings. Root Flow must not relax export restrictions. | Existing flow non-relaxation relation proves exact labels, taints, kinds, sinks, and partitions. | A proven root flow relaxation or a mapped action atom outside a closed allowed partition. | Unknown external provenance/influence, unsupported control-path relation, open partition, or missing typed coordinate. | Trusted external provenance/body data or actual egress requires separate evidence. |
| **U** | Compare exact (phase, meter, unit) coordinates and checked upper bounds. Root Forward and Recovery coordinates are independent; Root K-required Recovery Usage is a coverage check against Root U.recovery, never an addend. | Root forward and recovery uppers each fit exported upper; all K recovery profiles are accounted and covered. | A known root upper exceeds export or checked arithmetic overflows. | Missing meter is not zero; unknown units, open envelope, or unaligned local/root coordinates remain Unproven. | Actual usage, price settlement, reservation, and ledger evidence are runtime-only. |
| **K** | Preserve source/action/profile multiset. Map exact full RecoveryActionIdentity and action kind, full atoms after R mapping, RecoveryContract clause fields, activation/attempt bounds, per-attempt Usage, and root action totals. No total order among Recovery enum members is assumed. | V1 positive proof requires exact mapped Recovery semantics for every root-relevant source, every required action/profile is covered, and Root action/Usage totals fit epsilon/U/export ceilings. Existing recovery_not_weakened proves exact equivalence only. | A required root action/binding is absent, a required identity is contradicted, a binding is extra/invalid under the frozen validator, or exact action/usage total exceeds its ceiling. | Changed replay/idempotency/key/reconciliation/unknown handling without a frozen monotonic relation; source multiset or global Recovery aggregation cannot be proved. | Receipt state, reconciliation result, provider honoring, concrete compensation, and order remain RequiresEvidence. |
| **X** | Compare every per-child ExecutionRequirements against export using exact ExecutionUniverses and preserve feasibility versus security result. Do not aggregate profiles into a Worker. | Every child meets exported feasibility and security-preservation relations. | A child is proven infeasible under the export or weakens an explicit security constraint. | Missing execution catalog, unsupported version/region/trust relation, or absent child profile. | Worker placement/admission, attestation, sandbox enforcement, or runtime capacity remains RequiresEvidence. |

## E, K, and U are checked without double addition

The propagated root epsilon is the declared E ceiling assembled from source contracts. The K projection is a second proof input, not another term to add to epsilon. The checker performs both:

1. Root K action occurrences fit the corresponding source/root epsilon action row, including same-full-atom Forward plus action occurrences.
2. Root epsilon fits exported epsilon.
3. Root K action occurrences independently fit the mapped exported epsilon entry, guard, and upper.
4. Root K-required Recovery Usage fits Root U.recovery and exported U.recovery in exact recovery meter coordinates.

This avoids both failure modes: checking only Forward E and missing K, or adding the K projection to an E/U envelope that already includes the K obligation.

## Phi, F, and K limits

RecoveryBlocked, timeout, cancellation, and winner choice remain separate from the typed Failure row. The exported Phi proof compares business Failure values only.

K carries no action-specific Flow or Execution field. A source-level F/X proof and exact EffectFlowPartition may establish the frozen source relation for a K action atom, but the checker cannot invent an action-specific constraint. Where the export demands a relation not derivable from the source K and E/F/X facts, that dimension is Unproven (or RequiresEvidence if a specified trusted evidence object can resolve it).

For K, the root source/action/profile multiset is part of identity. A boolean such as known_recovery_sources_accounted or equality of source computation IDs is insufficient. A future checker must compare each exact RecoveryActionBinding and RecoveryUsageBinding, including action kind and all typed profile fields.

## Combined report behavior

* A known epsilon Violation and a separate body RequiresEvidence are both retained; overall is Violation.
* Static epsilon Proven plus missing CALL_SUBFLOW body proof yields static-envelope Proven and body-conformance RequiresEvidence; it does not yield full body conformance.
* A dimension that cannot be mapped is Unproven. It cannot be skipped because adjacent dimensions passed.
* A runtime fact never upgrades a static relation unless the assertion names an exact trusted evidence model; no such model is added here.
* Authority and Budget are not among the nine ComponentContract dimensions and have no Scope ceilings in frozen IR. Their attenuation remains Unproven and is not defaulted to unlimited.

## Accepted claim boundary

When all nine static dimensions are Proven, the only permitted claim is: the exact checked root's static declaration fits the exact exported ComponentContract under the pinned semantics and proof context. It is not a behavioral certificate, implementation conformance, release signature, admission decision, or runtime execution authorization.
