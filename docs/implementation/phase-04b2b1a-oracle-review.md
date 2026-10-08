# Phase 4B.2B.1A — Independent Oracle Review

## Independence and review method

`tools/gen-computation-control-corpus.py` is a self-contained Python model. It
imports no Rust crate, does not read Rust output, and computes expected JSON
before writing the frozen vectors. Rust's former inventory test read those
JSON files but did not share the Python formulas. This confirms code-path
separation; it does not change the historical fact that the Rust adapter
preceded the Oracle.

The review compared the Python operations with the frozen
`spec/computation/control-contract-algebra.md` and the existing formal APIs:
`af_effect::control`, `af_usage::control`, `af_failure::control`, and
`af_flow::merge_provenance_summaries`. Full Atom identity in the Oracle key
includes operation, selector port/scope and canonicalized complete qualifier
members. Sequence retains list order; Select uses exclusive join; Parallel
and Race accumulate possible work; missing/unknown contributions do not
become zero. The formulas remain a separate implementation, not a call into
Rust.

## Corrections and frozen corpus revision

Revision 1 had three verification defects, corrected in revision 2:

1. `effect_row_complete` used `row != "unknown"`, which incorrectly marked a
   bounded-open row complete. It now requires `row == "closed"`.
   `computation-control/proof/open-effect-row-remains-open` therefore changes
   `effect_row_complete` from `true` to `false`; its Effect row and residual
   stay open and unchanged.
2. A dynamic frozen SELECT always has selector data dependencies. Cases that
   did not spell them out now carry an explicit `value-slot:1` decision
   dependency, except the exact-constant case. Expected Flow records the
   decision influence and its Unproven status where applicable. This affects
   dynamic Select inputs and the associated expected Flow fields; it does not
   change their Effect join or Usage maximum.
3. The abstract child `verdict` field purported to be an input fact, and four
   cases used a generic `compose` operator not present in Semantic IR. The
   vectors now say which formal evidence path is materialized
   (`checked_component`, `unresolved_component_pin`, or
   `subflow_body_evidence`) and use the frozen `sequence` operator. The
   unknown-row vector is represented by an unresolved exact component pin,
   whose production leaf summary is Unknown/Unproven rather than a fabricated
   checked `EffectRow::Unknown` contract.

The generator and manifest now identify corpus revision 2. The 58 stable
CaseIds are unchanged. Input/expected changes are generated, not hand-edited;
revision 1 is recoverable from the Phase 4B.2B.1 commit
`a00fe81dac530b2ec61b40becbbcecc2c9793f7a`.

## Six independent normative checks

1. **RACE loser Effect.** Every contestant may have started before a
   cancellation request. The conservative upper envelope therefore includes
   the union/accumulation of both `database.write` and `network.send`; selecting
   one terminal result cannot remove the loser's possible external effect.
2. **RACE loser Usage.** For the same meter and phase, with finite bounds 1000
   and 2000 and both branches able to consume them, the checked accumulation
   is 3000. `max(1000, 2000)` would understate possible pre-cancellation work.
3. **SELECT guarded lower.** For one complete AtomKey under guards `G_a` and
   `G_b`, neither guard is guaranteed when the other branch is selected. The
   frozen branch join widens the guard and sets the unconditional lower to 0;
   the upper is the maximum.
4. **SEQUENCE conditional child.** If A can fail before B starts, B's positive
   lower is not unconditional. B remains in the possible upper bound; its
   lower is weakened before the formal sequence accumulation.
5. **RequiresEvidence child.** A subflow export may provide a known declared
   Effect envelope while body conformance remains RequiresEvidence. The
   envelope is retained, and the inherited evidence obligation prevents the
   parent overall verdict from becoming Proven.
6. **Open Effect row.** At the algebra level, a bounded-open child remains
   open after accumulation with a known sibling; the residual is retained and
   `effect_row_complete` is false. However, this input cannot currently be
   minted as a `ResolvedComputationLeaf`: `check_effect_resource` marks every
   open row Unproven, `check_contract` only mints `CheckedComponentContract`
   for an all-Proven report, and the computation snapshot accepts only that
   checked wrapper. Therefore this vector has a normative expected value but
   no valid current Phase 4B.2A production input path. It must not be counted
   as an executed production differential without an authorized model/API
   change.

These six checks are a human normative review, not a claim that the Rust
production analyzer has been differentially run for them. Per-case production
execution is reported separately in
`phase-04b2b1a-differential-report.md`.

