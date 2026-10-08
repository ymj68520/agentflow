# Phase 4B.0B — Design Readiness Report

> **DESIGN PROPOSAL** · **NON-NORMATIVE** · **NOT A FROZEN ABI**

## Status

**Design gate: READY TO IMPLEMENT PHASE 4B.1, subject to review of this
non-normative candidate.** Phase 4B.0B makes design decisions only. It creates
no production ComponentContract type/checker/schema and computes no digest.
This report does not start Phase 4B.1.

Decision date: 2026-10-08.

Audit start: `c2b570a` (Phase 4B.0A; clean worktree). The Phase 4B.0A P0
remains closed. The eight P1s are **ACCEPTED and closed as design blockers**:
their invariants are completely representable by closed typed fields and
explicit relation snapshots. The residual work is P2 schema/checker/conformance
implementation, not an unresolved P1 model question. See the
[P1 decision register](phase-04b0b-p1-decisions.md).

## Eight P1 decisions

| P1 | Status | Accepted disposition | Implementation boundary |
| --- | --- | --- | --- |
| P1-1 Identity/version/digest | ACCEPTED | ComponentIdentity is part of semantic bytes; ContractVersion is release metadata; implementation/deployment/evidence are separate | New ComponentContract schema plus release/implementation registry references |
| P1-2 I/O port ABI | ACCEPTED | Independent I/O/R namespaces, stable positive ports, exact closed sets and exact TypeSchemaRefs | New ComponentInputPort/ComponentOutputPort records |
| P1-3 Alias policy | ACCEPTED | Public nominal exact pin; internal structural alias expansion is not public compatibility | No old schema change; comparator implemented in new checker |
| P1-4 ε↔R | ACCEPTED | Exhaustive exact Operation×ResourceType relation with selector vocabulary and complete qualifier rules | New explicit Operation/Resource compatibility catalog |
| P1-5 ε↔F | ACCEPTED | Existing operation sink retains broad meaning; exact EffectFlowPartition may only narrow it | New typed Component-level Flow partition |
| P1-6 Recovery action identity | ACCEPTED | `(forward Atom, kind, action Atom)` with both atoms exact in closed ε and action graph acyclic | New typed RecoveryActionBinding |
| P1-7 K↔U | ACCEPTED | Per-action, per-attempt profile, explicit activation/attempt bounds, exhaustive meter coverage, checked aggregate ≤ U.recovery | New RecoveryUsageBinding and RecoveryUsageCoverageCatalog |
| P1-8 F↔X | ACCEPTED | Exact versioned mapping from labels/taints/authority/sinks to region pairs/trust/sandbox | New explicit Flow/Execution relation catalog |

`ACCEPTED` does not assert that declarations match an implementation. Missing
catalogs, unresolved exact pins, incomplete action coverage, or unknown
provenance remain Unproven. Known closed contradictions remain Violations.

## Candidate schema set

The ABI proposal is complete enough for a reviewed implementation plan. It
names four provisional, unregistered schemas:

1. `agentflow.component.contract@1` — semantic Component body, I/O/R ports,
   existing Φ/ε/R/F/U/K/X values, exact EffectFlowPartitions, RecoveryAction
   bindings and RecoveryUsage bindings.
2. `agentflow.effect.resource.compatibility@1` — exhaustive exact
   operation/resource/selector/qualifier support catalog.
3. `agentflow.recovery.usage.coverage@1` — exact RecoveryActionIdentity to
   complete meter-coordinate coverage.
4. `agentflow.flow.execution.relation@1` — exact label/taint/authority/sink
   mapping to closed region-pair, trust-domain and sandbox policies.

The three relation catalogs are explicit analysis inputs and are not fields
inside ComponentContract. Their exact schema pins, revisions and canonical
content checksums are included in proof context. This keeps changing catalog
assumptions out of immutable component semantics while preventing latest or
ambient lookup. The coverage catalog imports the exact ComponentContract
RecoveryActionIdentity type; dependency is one-way because ComponentContract
does not import relation catalogs.

No old AFIDL is modified. New local field numbers and provisional
SpecificationIds appear only in the design proposal. They consume no current
ABI assignment and are not registered. Phase 4B.1 must materialize the four
schemas as one reviewed additive set if the chosen relation snapshots cross
language/trust boundaries. There is no required old-schema fix, so a separate
Phase 4B.0C is not necessary.

## Digest scope and proof API

The future digest covers ComponentIdentity plus all immutable semantic
declarations and exact bindings, including per-action usage bounds. It
excludes release version, implementation digest/reference, deployment,
relation snapshot contents, prices, cost projections, budget reservations,
diagnostics, proof reports, attestations, workers, capacity, policy and
runtime evidence. Relation snapshot identity is recorded in proof output, not
folded into ComponentContractDigest.

The future API returns separate `Proven`, `Violation`, `Unproven`, and
`RequiresEvidence` results per dimension and relation. Schema validity,
cross-dimension consistency, static compatibility, implementation
attestation, and execution authorization are distinct. The complete design is
in [phase-04b0b-proof-api.md](phase-04b0b-proof-api.md).

## Design vectors and remaining risks

The decision corpus contains 37 design-level vectors: four per P1, an explicit
“same meter, no action coverage” case, three aggregate/failure-plane boundary
cases, and a digest-integrity case, each with input, expected verdict, required proof and reason.
It covers positive,
negative, Unproven and RequiresEvidence outcomes. These are not production
conformance tests and do not constitute machine verification.

### P2 — implementation/proof work still outstanding

* Implement and conform the four proposed schemas and their canonical order;
  no formal checker or relation-catalog loader exists yet.
* Validate all exact pins and cross-dimension references in the future
  `af-contract` checker; preserve per-relation results and snapshot pins.
* Add IR-wide call/Subflow/control propagation for Φ/ε/F/U/K and keep missing
  summaries Unproven; do not aggregate X without a placement topology.
* Define/prove Resource↔X adapter/access compatibility; this was not one of
  the eight P1 decisions and remains an independent P2 obligation.
* Implement checked action-usage arithmetic, exact meter coverage and
  relation snapshot evolution rules; existing U helpers remain standalone.
* Add production conformance suites, independent oracles, generated ABI rows
  and registry entries only in the authorized materialization phase.

### P3 — runtime/authority work still deferred

Publisher authentication, implementation conformance attestation, trusted
worker evidence, runtime sandbox enforcement, policy authorization,
EffectGrant, ResourceHandle, live resource binding, capacity, metering
receipts, Usage/Cost ledgers, budget reservation, reconciliation/compensation
execution and runtime admissions remain separate authorities. No static
ComponentContract result implies any of them.

## Frozen-surface and phase gate

The design changes only non-normative `docs/implementation` files. Required
materialization constraints remain:

```text
12 current AFIDL modules and their TypeSchemaDigests unchanged
SemanticIrDigest unchanged
400 current ABI assignments unchanged
30 current Specifications and 296 current Rules unchanged
14 HashV1 domains unchanged
Effect corpus revision 3 and Recovery corpus revision 2 unchanged
all other corpus revisions/Expected values unchanged
```

### Verification snapshot

The design-only tree was checked against start HEAD `c2b570a`:

* `git diff --name-only c2b570a -- spec/idl spec/registry spec/tables
  spec/conformance` returned no paths. This includes all TypeSchemaDigest and
  SemanticIrDigest fixtures, ABI rows, registries, domains and corpora.
* Frozen counts remain 12 AFIDL modules, 400 active ABI assignments, 30
  Specifications, 296 Rules, 14 HashV1 domains and 67 conformance suites.
* `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features --
  -D warnings`, and `cargo test --all --quiet` passed (zero failed tests).
* `cargo run -p af-spec --bin spec-check -- check` passed: 30 Specifications,
  296 Rules, 400 active ABI assignments, 12 AFIDL modules, zero cross-reference
  errors.
* All 16 `gen-*.py` corpus/registry generators ran twice; both `spec/` tree
  SHA-256 values were
  `f9970d58b94c9be391461fe6fccafb92972954b80c65ec21883e1a7b5a962bff`.

The same gates are to be rerun on the committed documentation state before
handoff. These tests validate the existing repository and frozen surfaces;
they do not prove the proposed ComponentContract design or relation catalogs.

Phase 4B.0B does not register the proposed SpecificationIds or RuleIds and
does not change the existing audit baseline. The Phase 4B.1 gate is satisfied
at the **design** level only: P0 closed, eight P1 decisions accepted,
candidate fields/identities/digest scope typed, three additive relation
schemas identified, 32 vectors present, and static proof/runtime authority
boundaries explicit. This is “ready to implement Phase 4B.1,” not “already
verified.”
