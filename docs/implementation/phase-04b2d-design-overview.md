# Phase 4B.2D — Design Overview

Status: design decisions recorded; no checker or production change is made by this phase.

Every candidate structure and rule below is **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED ABI / NOT IMPLEMENTED**. The formal inputs remain the frozen Semantic IR, ComponentContract, Effect, Recovery, Usage, and control-algebra models.

## Baseline and scope

The starting revision is 6815928d8571d2b499bfff42e5c16dc4f445dc61, the Phase 4B.2C audit commit. The worktree was clean at inspection. Baseline counters are 40 specifications, 360 normative rules, 477 active ABI assignments, 16 AFIDL modules, and 14 HashV1 domains.

Phase 4B.2C found no confirmed P0 undercount. It identified three P1 design gaps:

| Finding | Design resolution |
| --- | --- |
| AUDIT-10 — epsilon action guard versus K action possibility | D1: the declared Effect row is the ceiling for every occurrence of the exact atom, including K actions. A positive K action possibility must be guard-covered. |
| AUDIT-02 — CompensationScope-local versus root-total action bound | D2: use distinct local/root coordinates, provenance-keyed repetition factors, operator-specific aggregation, and one root traversal of K bindings. |
| AUDIT-01 — root summary versus exported ComponentContract | D3: choose an explicit typed build-time RootExportAssertion; derive coordinates first, then prove each dimension separately. |

All three decisions are accepted as design. This resolves the semantics, not the implementation or any particular workflow's proof result.

## Proof flow

~~~text
Checked SemanticIrDocument + exact ComponentContract export pin
        + complete typed coordinate maps
        + exact static proof-context manifest
                         |
            root derivation under one context
                         |
       I / O / Phi / epsilon / R / F / U / K / X
                         |
 RootExportAssertion report: Proven / Violation / Unproven /
                             RequiresEvidence, per dimension
~~~

The assertion is not a ComponentContract and has no standalone digest. It cannot turn a root summary into a wire contract, body attestation, publisher signature, runtime grant, budget reservation, receipt, Worker placement, or execution authorization.

## Accepted decisions

* **D1:** Action possibility is based on the exact Forward Effect guard plus retained control-path evidence. Receipt eligibility may narrow real runtime actions, but is not assumed for a static upper. The action atom's declared epsilon guard is a ceiling. Prove possibility implies ceiling; emit Violation only with a typed, satisfiable counterexample; otherwise keep Unproven.
* **D2:** A CompScope-local count is per activation of that exact scope. Root counts apply each unique Iterate/Retry factor once and compose actions with the existing operator-specific sum/join rules. Nested scopes assign a binding to its nearest enclosing CompScope for local accounting; outer scopes carry nested contributions transitively, never as a second owner. Root aggregation starts from source bindings once, not from summing nested scope projections.
* **D3:** Option A is selected. Assertion pins exact SemanticIrDigest, exported ComponentContractDigest, compiler semantics, a content-complete proof-context manifest, and fully typed input/output/resource/effect/flow/recovery/execution coordinate witnesses. A future WorkflowVersion owns the assertion inline; its existing canonical artifact identity must commit the complete assertion. No RootContractDigest or new HashV1 domain is introduced.

The detailed decisions and predicates are in phase-04b2d-p1-decisions.md; their counterexamples are in phase-04b2d-decision-vectors.md.

## Frozen-model constraints

1. EffectGuard remains exactly unconditional or input-equals. No conjunction, negation, arbitrary predicate, or new guard ABI is proposed.
2. Component input, output, resource, root ValueSlot, ResourceSlot, and computation-port identities remain separate namespaces. Numeric equality is never a mapping.
3. Root K action bounds are a coverage projection, not an addition to root epsilon or root U. Forward history and Recovery Usage are not refunded.
4. Source node identity alone is not a closure proof. Each source contract digest, full RecoveryActionIdentity, matching RecoveryUsageBinding, and forward occurrence remain accounted.
5. Unknown catalogs, guard domains, expression mappings, or proof-context content remain Unproven. Runtime receipts and body attestations remain RequiresEvidence.
6. Current pointer-identical ProofContextReference is process-local. No serialized assertion may contain a pointer or treat a revision label/checksum as a publisher signature.

## Required follow-up

The 62 decision vectors are audit designs, not entries in the frozen conformance corpus. A separate authorized implementation phase may implement the checker and add production differentials. Until then, this design phase makes no root-export compatibility claim and changes no Rust source, AFIDL, ABI, registry, digest, or Expected file.

## Document map

* phase-04b2d-guard-compatibility.md — D1 guard sources, relation, countermodels, and coordinate bridge.
* phase-04b2d-root-recovery-aggregation.md — D2 local/root action and Usage algebra, factor identity, nesting ownership, and hand calculations.
* phase-04b2d-root-export-assertion.md — D3 assertion ownership, candidate fields, alternatives, and version binding.
* phase-04b2d-coordinate-mapping.md — typed maps for I/O/R/E/F/K/X coordinates.
* phase-04b2d-cross-dimension-proof.md — proof directions and verdict rules for all nine dimensions.
* phase-04b2d-proof-context-lifecycle.md — exact context/compiler identities, invalidation, and subflow evidence.
* phase-04b2d-decision-vectors.md — 62 independent design vectors.
* phase-04b2d-p1-decisions.md — full P1 Decision Register.
* phase-04b2d-readiness.md — gates, risk inventory, P2 plan, and implementation readiness.
