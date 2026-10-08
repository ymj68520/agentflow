# Phase 4B.0B — P1 Decision Register

> **DESIGN PROPOSAL** · **NON-NORMATIVE** · **NOT A FROZEN ABI**

Decision date: 2026-10-08. Baseline: `c2b570a` (Phase 4B.0A committed,
worktree clean). This register
closes the design questions only. It creates no AFIDL module, registered
SpecificationId, ABI assignment, RuleId, ComponentContractDigest, or runtime
proof implementation. “ACCEPTED” means the model is sufficiently explicit to
implement and test; it does not mean a contract is validated or an
implementation is trustworthy.

## Decision summary

| Decision | P1 | Status | Chosen model | Full record |
| --- | --- | --- | --- | --- |
| B0B-D1 | Component identity/version/digest | **ACCEPTED** | Stable lineage is in the semantic body; release version and implementation pins are separate | [Identity decisions](phase-04b0b-identity-decisions.md) |
| B0B-D2 | I/O port ABI and compatibility | **ACCEPTED** | Independent stable I/O/R namespaces; closed required ports; exact pinned type equality is the only general proof | [Type and port policy](phase-04b0b-type-and-port-policy.md) |
| B0B-D3 | Alias comparison | **ACCEPTED** | Public nominal exact TypeSchemaRef pins; structural expansion remains an internal value-typing operation | [Type and port policy](phase-04b0b-type-and-port-policy.md) |
| B0B-D4 | ε↔R operation/resource relation | **ACCEPTED** | Exact, versioned Operation×ResourceType catalog with explicit selector/qualifier rules | [Effect/resource/flow design](phase-04b0b-effect-resource-flow-design.md) |
| B0B-D5 | ε↔F selector-grain sink identity | **ACCEPTED** | Component-level complete allowlist partitions refine legacy operation-level F sinks | [Effect/resource/flow design](phase-04b0b-effect-resource-flow-design.md) |
| B0B-D6 | Complete Recovery action identity | **ACCEPTED** | Bind forward Atom + action kind + exact recovery Atom; all actions must be in closed ε | [Recovery/action-usage design](phase-04b0b-recovery-action-usage-design.md) |
| B0B-D7 | K↔U recovery usage coverage | **ACCEPTED** | Per-action, per-attempt recovery usage profile with exhaustive meter coverage and explicit multiplicity | [Recovery/action-usage design](phase-04b0b-recovery-action-usage-design.md) |
| B0B-D8 | F↔X information/execution relation | **ACCEPTED** | Exact versioned mapping universe injected into analysis; no names or worker claims infer compatibility | [Information/execution design](phase-04b0b-information-execution-design.md) |

The dependency order is respected: exact TypeSchemaRef policy precedes public
port checks; exact EffectAtom identity (including the Phase 4B.0A qualifier
member fix) is reused by ε↔R, ε↔F, Recovery and Usage bindings; Recovery
identity precedes action-usage coverage; F/X rules consume exact F and X
identities.

## Chosen change class

* **Additive extension:** one new, closed `ComponentContract@1` candidate with
  typed ports and typed cross-dimension bindings. Candidate field numbers are
  local design notes only.
* **Versioned new schema:** three separate, exact-versioned relation-catalog
  schemas are proposed for Operation/Resource support, Recovery action/meter
  coverage, and Flow/Execution mapping. They are analysis inputs, not fields
  imported into the ComponentContract body.
* **Contract-level binding model:** used where an existing frozen wire value
  is too coarse but the component must declare a narrower exact relationship
  (selector-granular Flow and Recovery/Usage action bindings).
* **Existing ABI incompatibility:** none is required. The existing F and K
  values remain valid and unchanged. A contract that lacks a required exact
  binding is Unproven; old operation-only claims are never silently upgraded
  to selector/action-specific proof.

All new relation tables are closed, typed, versioned snapshots. The checker
receives them explicitly and records their exact catalog/revision/content pin
in the proof context. They are not ambient lookups and do not enter the
ComponentContract digest. If a relation catalog is needed across a trust or
language boundary, its dedicated additive AFIDL schema is required; it is not
an untyped JSON/`Any` map.

## Proof and authority boundary

The future API keeps schema validity, cross-dimension consistency and
substitution compatibility as separate reports. Every verdict is per
dimension and per relation. `Proven` is conditional on exact declarations and
explicit immutable universes; `Violation`, `Unproven`, and
`RequiresEvidence` remain distinct. No aggregate Boolean hides a missing
proof. A static match is not attestation, worker admission, policy approval,
EffectGrant, ResourceHandle, reservation, or permission to execute.

## Readiness disposition

The eight P1 questions no longer block the *design* of a v1 contract. Their
future schemas, fields, identities, relation snapshots, proof directions and
fail-closed cases are enumerated in the companion documents and ABI proposal;
each invariant is expressible by a closed typed shape. The P1 design blockers
are therefore closed and the remaining work is P2 implementation/conformance.
This document does not perform that work or claim a proof has been executed.

No Phase 4B.0C is required: the old schemas need no repair, and the new
ComponentContract plus three relation-catalog schemas can be reviewed and
materialized together in Phase 4B.1. If a future implementation scope excludes
those catalog schemas, it must be widened before freezing the candidate ABI;
it must not substitute partial or unpinned relation data. The existing P2 and
P3 items remain as listed in the [readiness report](phase-04b0b-readiness-report.md).

## Companion design artifacts

* [Identity decisions](phase-04b0b-identity-decisions.md)
* [Type and port policy](phase-04b0b-type-and-port-policy.md)
* [Effect/resource/flow design](phase-04b0b-effect-resource-flow-design.md)
* [Recovery/action-usage design](phase-04b0b-recovery-action-usage-design.md)
* [Information/execution design](phase-04b0b-information-execution-design.md)
* [Proof API](phase-04b0b-proof-api.md)
* [Decision vectors](phase-04b0b-decision-vectors.md)
* [Readiness report](phase-04b0b-readiness-report.md)
* [Updated candidate ABI proposal](phase-04b0-abi-proposal.md)
