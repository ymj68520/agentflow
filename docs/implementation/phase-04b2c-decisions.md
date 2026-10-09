# Phase 4B.2C — Audit decisions

1. **P1 guard relation is unresolved, not a confirmed P0 undercount.** An unconditional Forward plus a
   K compensation can coexist with a narrower `input_equals` guard on the
   action Effect row. Production derives an unconditional conditional action,
   checks numeric upper coverage, and returns RequiresEvidence. The sidecar is
   wider than the E row, so the probe does not demonstrate a false-negative
   total; it demonstrates missing guard-compatibility semantics. No fix was
   made here.
2. **Level 1 accepted.** Checked IR already rejects cycles, multiple parents,
   root parentage, and unreachable computations. The workflow API additionally
   compares every declared node against its owned preorder and checks resolved
   leaf occurrence/residual/K-source accounting.
3. **Level 2 accepted for the declared envelopes.** E/Phi/U composition is
   conservative under the current operator rules; R/F/K/X remain per-source.
   No generic component ports, Flow sources, or execution profiles are
   flattened by numeric coincidence.
4. **Conditional K action bounds are node-scoped in practice.** An inner
   Retry/Iterate is already attached to the forward occurrence before
   CompScope computes its action. An outer Retry/Iterate is added to the
   parent/root occurrence and E/U algebra after the inner CompScope summary
   exists. Root E/U remain amplified. The exposed CompScope action upper is
   therefore not a documented root-total K envelope. No add/max rule is
   selected in this audit; formalize the coordinate before root export proof.
5. **Recovery Usage is not added twice to root U.** Component validation
   checks K per-attempt usage against U.recovery. Control U composition
   amplifies the declared U. CompScope's action-specific potential recovery
   usage is a separate projection and must not be summed again over root U.
6. **Root Export remains Unproven.** Recommend a future typed, non-wire
   build-time assertion with exact digests and explicit I/O/R mappings (Option
   A in `phase-04b2c-root-export-options.md`). Do not reuse ComponentContract
   substitution without mapping. Persistent proof-context identity must be
   designed separately.
7. **No ABI migration.** Option B is not justified by this audit and is not
   started. Option C can supply implementation attestation but cannot replace
   static compatibility proof.
8. **Authority/Budget are not filled with defaults.** Current Scope remains
   Unproven on those dimensions; actual grants, reservations and receipts are
   runtime/context obligations (P3) unless a later static policy model is
   explicitly specified.
9. **Follow-up coverage is needed.** The committed 55-case corpus tests action
   bounds and an underdeclared E row, but does not put nonempty K actions under
   outer Retry/Iterate, use identical Forward/action Atom identity, or combine
   loser-branch K with CompScope. These are P2 conformance gaps; do not change
   frozen Expected values in this audit.
10. **Outcome C — Design Incompleteness.** No P0 false-negative total was
    confirmed. Structural closure is accepted, while root-total K-action
    coordinates and E/K action-guard semantics remain P1 design questions.
    Root export proof is also absent. Recommend a separate Phase 4B.2D design
    stage; this audit does not begin it.
