# Phase 4C.1B.4 — Oracle Coverage Audit (interim)

## Result

**Outcome D — Conformance Still Incomplete.** This audit does not close B01/B02 and is not an Independent Portable Proof Integration Audit.

The checkout began at `3eb53e8110c8901ed9f789dda9631eed7a7bd0dc`, with a clean worktree. The production D0–D10 path was not changed.

The existing Python generator independently constructs Portable declarations, ACE1 bytes and some fixture-level facts. Its RootExport expected projection is intentionally small: it checks an empty-root positive (and missing-execution variant), including root opcode, empty effect/recovery totals, two explicit zero usage coordinates, execution mapping count, nine verdict labels and authority boundaries. Existing D9 tests additionally check selected opcode/closure fields and recovery bounds. These are useful local checks, not a Full-field Oracle.

The Rust report projection contains substantially more information than those expectations: recursive control facts, provenance, effect and failure rows, usage states, recovery source/action identities and multiplicity paths, typed mapping witnesses, preflight proofs, per-dimension source facts, execution child verdicts, capability issuance and unresolved obligations. The independent generator does not currently derive these complete values for every applicable CaseId. The per-type gap inventory is in [the field coverage manifest](phase-04c1b4-field-coverage-manifest.md); that inventory is not yet a closed, executable coverage contract.

## What changed in this interim pass

The `root_export_revalidation` integration test now has a recursive exact JSON comparator. It checks object key presence in both directions, scalar type/value, explicit `null`, array length, array order and each array member, and returns the first differing field path. Comparator self-tests make 68 one-field mutations across representative D9 root, non-empty D9 recovery, D10 positive, and P30 report projections. Each mutation is required to produce its exact field path.

This is comparator integrity evidence only. Its mutation baseline is a production projection and is never treated as an independent Expected. The test does not establish Rust/Python conformance, nor does it turn a projection-derived value into a normative fact.

## Unclosed requirements

- Independent expected documents for all 78 CaseIds at their actual reached stages.
- A complete D9 control/effect/failure/usage/flow/execution/recovery model over the declared Portable inputs.
- A declaration-derived full `RootExportAssertionIdentity`, including every typed mapping member, all content identities and explicit option presence.
- Independent preflight and nine-dimension proofs including source facts, mapping witnesses, prerequisites, missing evidence, violation witnesses and rule attribution.
- Full-field comparison of the six producer/consumer scenarios and the 69 D10-reaching cases.
- A closed, executable field inventory with numeric required/derived/compared/diagnostic-only/uncovered counts. The checked-in inventory is only a gap audit, so no zero-gap count is claimed.
- Human-derived cross-check vectors for all requested control operators and every proof dimension.
- The requested complete production-input mutation matrix as part of an independent full-field Oracle.

No Rust output was copied into Python Expected. Unknown proof facts remain gaps; they are not defaulted to empty, zero, or Proven. No production checker contradiction was observed in this audit, but the missing independent oracle means that absence is not evidence of soundness.

## Finding status carried forward

- AUDIT-F01: CLOSED; this pass adds no independent evidence sufficient to reopen or broaden it.
- AUDIT-F02: CLOSED; this pass adds no independent evidence sufficient to reopen or broaden it.
- AUDIT-F03: OPEN / P2; the known applied-rule set is not a complete transitive dependency closure.
- AUDIT-F04: OPEN / P2; conservative guard countermodel reachability can still leave obligations Unproven.
- New P0/P1/P2/P3 findings: none confirmed by the work performed here. This is not a complete soundness re-audit.

## Frozen boundaries

No changes were made to RootExport semantics, D0–D10 trust boundaries, AFIDL, ABI assignments, HashV1 domains, or digest goldens. B01/B02 remain implementation-complete but conformance-closure-pending. The work is not ready to enter the independent integration audit or any later phase.
