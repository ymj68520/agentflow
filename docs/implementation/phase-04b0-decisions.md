# Phase 4B.0 — Decision Register

Status: audit/design decisions only. No decision below freezes a ComponentContract
wire ABI. The candidate schema is separately marked non-normative in the
[ABI proposal](phase-04b0-abi-proposal.md).

Companion artifacts: [audit report](phase-04b0-preassembly-audit.md),
[field matrix](phase-04b0-field-matrix.md),
[port namespace matrix](phase-04b0-port-namespace-matrix.md),
[compatibility matrix](phase-04b0-compatibility-matrix.md),
[cross-dimension obligations](phase-04b0-cross-dimension-obligations.md),
and [counterexamples](phase-04b0-counterexamples.md).

| ID | Decision | Result |
| --- | --- | --- |
| B0-D1 | Contract vs implementation identity | Keep Component logical identity/version/digest separate from implementation reference/digest and compatibility evidence. Contract digest pins semantic promise; it does not select or attest an implementation. |
| B0-D2 | Candidate semantic fields | I, O, Φ, ε, R, F, U, K and X are candidate semantic fields. Cost snapshot/projection, deployment binding, diagnostics, proof artifacts and runtime authority stay outside. |
| B0-D3 | Component port identity | I, O and R are three independent positive, 1-based namespaces. Candidate identity is `(ComponentIdentity, direction, port_no)`; names do not identify ports. |
| B0-D4 | Subflow boundary | `SubflowInterface` remains a derived projection of one exact SemanticIrDigest, not a formal Component I/O schema. RunParameterId/ResourceSlotId never become global Component port IDs. |
| B0-D5 | Type comparison | Exact structural equality may prove compatibility; differing shapes/nominal schema pins are Unproven absent a frozen subtype/variance relation. Alias-vs-target comparison is a normative design blocker because Phase 3A and 3D currently differ. |
| B0-D6 | Input/output direction | Preserve `I_old <: I_new` and `O_new <: O_old`. Exact equality is the only currently reliable general case. Do not invent implicit coercions. |
| B0-D7 | ε↔R | Check selector port existence in the same contract's R namespace. Operation/resource compatibility needs an explicit versioned relation; equal port number is not proof. |
| B0-D8 | ε↔F | Flow's current operation-only sink is too coarse for selector-sensitive policy. Add a reviewed selector-level sink identity before claiming full security compatibility. |
| B0-D9 | F↔I/O/R | Resolve each source/sink through its own namespace, exact port, no renumbering or latest fallback. Unknown provenance stays unknown and cannot become an empty source set. |
| B0-D10 | F↔X | No region, trust, sandbox or taint relation is inferred from names. Missing trusted catalog relation yields Unproven. Catalog assumption, runtime attestation and actual enforcement are three distinct layers. |
| B0-D11 | K↔ε | Forward K clauses require exact full EffectAtom coverage. Reconciliation and compensation references need full action identities and proof that those actions are within ε. |
| B0-D12 | K↔U | Recovery-phase U must cover reconciliation/compensation/retry. Missing recovery usage is Unproven, never zero/free. A future exact RecoveryActionUsageProfile is needed to derive coverage. |
| B0-D13 | R↔X | R remains logical external need; X remains executor environment. Binding/access requires explicit adapter/deployment evidence and never grants authority. |
| B0-D14 | X compatibility | Keep feasibility and security-preservation proofs independent. Full X compatibility requires both; relaxing sandbox/residency cannot be accepted because eligible profile set grows. |
| B0-D15 | U and Cost | U belongs in the stable component semantic body. PriceTableSnapshot/CurrencyUniverse are fixed analysis inputs; StaticCostProjection is derived; StaticCostCeiling is treated as Run-local budget policy. None implies reservation. |
| B0-D16 | Control and Subflow | Compose known dimensions using their own algebra; do not substitute empty rows for missing INVOKE/CALL_SUBFLOW contracts. No blind X aggregation across SEQUENCE/PARALLEL. |
| B0-D17 | Static proof | Use Proven/Violation/Unproven/RequiresEvidence per dimension and obligation. Static proof is conditional on explicit inputs, not implementation attestation or runtime authorization. |
| B0-D18 | Digest scope | Candidate digest contains only immutable semantic contract facts and exact pins. It excludes implementation, deployment, price snapshot, diagnostics, proof, attestation and runtime state. |
| B0-D19 | Existing qualifier key | At the Phase 4B.0 audit baseline, `AtomKey` distinguished only qualifier family, colliding `natural`/`none` and producing a Recovery Proven result for mismatched effects. Phase 4B.0A closes this P0 by keying complete typed qualifier identities; see the dated closure below. |
| B0-D20 | Current phase exit | At the Phase 4B.0 audit, P0 and P1 findings blocked schema freeze. P0 closed in 4B.0A and the eight P1 designs were accepted in 4B.0B. Neither design phase creates component-contract.afidl or registers IDs/ABI. |
| B0-D21 | Unified Computation projection | Component calls source all fields from an exact contract; Subflow derives I/O/R today and needs exact contract closure for the other dimensions; Control summaries compose only where a per-dimension rule exists. Any missing field stays Unproven, never an empty row. |

## Open decisions at Phase 4B.0 audit close (resolved in Phase 4B.0B)

* Resolve alias-vs-target type comparison consistently between Phase 3A value
  typing and Phase 3D Subflow interface checks.
* Freeze a selector-granular Flow sink and its exact relation to an EffectAtom.
* Define exact reconciliation and compensation Effect identities and how they
  are constrained by ε and mapped to U recovery meters.
* Define explicit operation/resource and F/X catalog relations, including
  their identity/version and evidence assumptions.
* Define stable Component identity/version semantics and public port evolution
  before registering any field number.

The audit recorded these as open. Phase 4B.0B resolves them at design level;
the field layout alone was not the resolution. See the per-P1 records in
`phase-04b0b-p1-decisions.md` and its linked design documents.

## Phase 4B.0A P0 closure (2026-10-08)

The shared `af_effect::AtomKey` now retains typed qualifiers ordered by frozen
family/member discriminants and their typed value. The independent Effect and
Recovery oracles freeze the previously colliding cases; the public Recovery
API regression requires a mismatched qualifier to return
`recovery_effect_not_declared`, never `ProvenStaticConstraint`. Effect
canonicalization, containment, `require_contained_in`, accumulation, branch
join, repetition and control propagation now use the same exact identity.
No AFIDL/ABI assignment, TypeSchemaDigest, SemanticIrDigest or HashV1 domain
was changed. The P0 remains closed. Phase 4B.0B accepts the eight P1 designs;
implementation is still outstanding and is separately gated by
`phase-04b0b-readiness-report.md`.

See the full [Phase 4B.0B Decision Register](phase-04b0b-p1-decisions.md),
[candidate ABI](phase-04b0-abi-proposal.md), and
[readiness report](phase-04b0b-readiness-report.md).
