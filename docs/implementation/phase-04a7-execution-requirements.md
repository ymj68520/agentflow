# Phase 4A.7 Implementation Notes

Phase 4A.7 adds the static X dimension as a standalone `af-execution` crate.
The AFIDL module is closed and additive. The crate depends only on `af-core`,
`af-idl`, `af-semantic`, `serde`, and Unicode normalization. Its APIs accept
explicit universes and profile descriptors; no filesystem, network, clock,
worker registry, scheduler, runtime, or policy service is reachable.

The implementation has four proof boundaries:

1. `canonicalize_execution_requirements` resolves nominal identities and
   exact versions, checks closed units/features/ABI pins, rejects duplicates,
   and produces deterministic source-order-independent form.
2. `match_execution_requirements` checks hard requirements conjunctively and
   returns conditional descriptor proof, rejection, or unproven evidence.
3. `execution_compatibility` returns independent feasibility and security
   verdicts plus a combined verdict. It never collapses security relaxation
   into deployment compatibility.
4. `canonical_bytes` lowers the canonical declaration to schema-derived ACE1
   bytes only. It produces no digest.

The independent Python oracle freezes ACE1 bytes, normalization errors,
profile outcomes, compatibility verdicts, and order-equivalence cases under
`spec/conformance/execution-requirements/`. Rust tests consume those files and
never generate expectations from the Rust implementation.

The profile model is deliberately nominal: quantities are not reservations,
sandbox levels are not enforcement, trust is not authority, and a matching
descriptor is not a placement decision. Full ComponentContract integration and
runtime evidence are Phase 4B blockers.
