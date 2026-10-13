# Phase 4C.0 — Readiness and Next-Stage Dependencies

This is **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED ABI / NOT IMPLEMENTED**.
The publication architecture decisions C01–C07 are ACCEPTED. The repository is
**not ready to implement a concrete WorkflowVersion/Publish system from the
current static wrapper alone**: MODEL-B01–B07 remain BLOCKING.

## Capability boundary

Current checked contracts, IR/dataflow, leaf resolution, 13-opcode control
closure, RootRecovery and RootExport provide a useful static foundation.
Phase 4B.2F.2's bounded Outcome A permits considering that foundation for
future release decisions. It does not supply portable schemas, Plan
preservation, durable package closure, trusted body/implementation evidence,
publisher identity or release authorization.

F01/F02 stay CLOSED. F03/F04 stay OPEN/P2 with the triggers and responsibilities
in the compiler-semantics document. This design does not alter the historical
probe accounting.

## Proposed future engineering stages

These stage labels are a dependency proposal, not implementation authorization:

| Order | Stage / deliverable | Readiness condition |
| --- | --- | --- |
| 1 | Portable assertion/context and compiler semantic-content models (B01/B02) | Define versioned exact schemas, closed decoding, context roundtrip, origin approval, semantic source roles; decide attribution profile |
| 2 | LockGraph/resolution model (B03) | Exact dependency/association schema and canonical encoding; fresh versus locked rebuild; closure/cycle/eligibility vectors |
| 3 | PlanCore/lowering/preservation model (B04) | Formal Plan schema/trace relation and exact static/lowering/runtime identities; independent negative/preservation vectors |
| Parallel with 1–3 | Body/implementation/build/publisher evidence and release policy models (B05/B07) | Subject/predicate/issuer scope/freshness and complete mandatory requirement algebra; formally settle deploy-bound implementation claims |
| 4 | WorkflowVersion parent/identity model (B06) | Prior projections stable; specify full canonical payload, logical reference rules, signing subjects and cross-language/digest goldens |
| 5 | Publication verifier implementation and conformance | All model blockers resolved, required registry changes separately authorized, 48 design vectors materialized plus soundness tests |
| Later | DeploymentRevision, Run Admission and runtime models | Exact published subject and independently accepted environment/principal/authority/budget protocols |

Plan/LockGraph design and semantic preservation precede version materialization.
Internal compile analysis may continue before them; real Publish remains
blocked. No certificate ABI or selective positive-proof cache is required for
the chosen option A.

F03 complete Rule Dependency Closure becomes blocking before a certificate
verifier, positive subproof cache, or rule-complete release policy. The basic
full-revalidation profile retains full semantic content checks and explicitly
incomplete per-rule attribution. F04 leaves uncertain guard/path obligations
Unproven until a separately justified precision improvement exists.

## Risk and blocking inventory

| Priority | Item | Responsibility / implication |
| --- | --- | --- |
| P0 | No newly reproduced static false-Proven in this design phase | Historical F01/F02 fixes remain accepted; no new checker audit result is claimed |
| P1 | Portable canonical proof/context model absent | B01; cannot persist Rust objects as a release proof |
| P1 | Plan/LockGraph/full semantic identity absent | B02–B04; cannot build executable/version identity from static declarations |
| P1 | Required evidence/policy/parent models absent | B05–B07; actual Publish blocked, including conditional implementation profiles |
| P2 | F03 incomplete RuleId attribution | Preserve full content checks; hard trigger before rule-step/cached proof claims |
| P2 | F04 conservative guard diagnostics | Unproven remains non-admissible; no unsupported Boolean inference |
| P2 | Historical verifier/content retention and resource-bounded revalidation | Explicit support/availability policy; failure remains incomplete |
| P3 | Deployment, Run authority, Worker/enforcement, receipts and runtime recovery | Later owning models; no static proof creates rights |

P1 entries are known missing formal dependencies, not newly confirmed
RootExport soundness findings. Research “design frozen” does not remove these
engineering blockers.

## Phase completion

This package defines the layer matrix, option-A revalidation, inline
proof-carrying parent, context/dependency content, version identity, Plan
preservation, trust/evidence, configuration and fail-closed decisions. It
contains 48 expected design scenarios and seven accepted architecture
decisions with explicit model blockers.

Phase completion requires the unchanged 311-case differential regression,
Cargo/spec-check gates, generator double-round idempotence, frozen-path
comparison, design-document commit and committed-tree revalidation. No
production type, schema, registry/domain, corpus expectation or checker change
is part of this phase.
