# Phase 4B.2B.3 — Decisions

1. **No wire evolution.** Use the exact frozen Scope and CompensationScope
   records. No authority, budget, task, receipt, or root-contract field is
   added.
2. **Reuse one analyzer.** Add the two operators to `derive_control_contract`
   and make workflow closure a wrapper that resolves leaves and validates
   accounting.
3. **Scope resource proof is narrow.** Compare Phase 3C parent/declaration/
   child resource sets; do not equate visibility with possession or permission.
4. **Authority and budget stay Unproven.** Their ceilings are absent from the
   frozen Scope body and the static inputs.
5. **Compensation remains conditional.** Use exact K identity, occurrence
   bounds, activation/attempt bounds, and Recovery Usage. Forward effects and
   outcomes remain unchanged.
6. **Root export is not fabricated.** Keep the root summary derived-only and
   report `NotExpressedInSemanticIr` for compatibility.
7. **Legacy 72-case Scope transition.** The frozen advanced corpus had a
   negative `scope-unsupported` fixture. That expectation cannot remain true
   once this phase implements Scope. Preserve its CaseId and the 72-case
   manifest; revise only that one expected projection and bump the corpus
   revision to 2. The change is explicitly normative, not a tolerance or a
   mismatch mask. The other 71 expected projections remain byte-identical.
   The Phase 4B.2B.1C 58-case revision-5 corpus remains unchanged.

**Static Proven Facts:** the implemented rules are limited to checked IR,
Phase 3C and checked ComponentContract/K inputs.

**Conservative Upper Bounds:** the existing operator algebras plus exact
conditional compensation multiplication.

**RequiresEvidence:** lifecycle, receipts, and actual compensation order.

**Unproven:** absent authority/budget ceilings and root-export mapping.

**Runtime-Only Obligations:** grants, reservations, receipt/reconciliation,
dispatch, and actual terminal state.
