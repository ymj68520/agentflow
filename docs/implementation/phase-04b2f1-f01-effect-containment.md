# Phase 4B.2F.1 — F01 Same-Row Effect Containment

## Finding and repair

AUDIT-F01 was reproduced before the production edit using checked
ComponentContracts, checked IR, Phase 3C, resolver, root derivation,
RootRecovery, and `check_root_export_assertion`. The former fallback first
searched all selector-compatible export rows for a sufficient occurrence
bound, then independently searched all such rows for a covering guard. That
allowed E1's bound and E2's guard to combine into a proof no declaration had
made.

`effect_containment_verdict` now keeps the fallback facts row-local. An entry
is a complete candidate only when the same export row has exact operation,
full qualifier identity, requirement port and selector scope, covering
bounds, and a guard containing the root guard. The ordinary positive path
remains `af_effect::contains`, whose canonical one-use slot matching is
authoritative. The fallback cannot create a second path to `Proven` after
that algebra returns `Unproven`.

For partial fallback analysis, rows are tracked by their export-entry index.
A fully covering row claimed by an earlier root occurrence cannot be claimed
again. Capacity contention is `Unproven`, consistent with the normative
one-use-slot proof limit; no aggregate ceiling is inferred. Distinct full
AtomKeys, source ordinals, guards, and occurrence bounds remain distinct.

## Guard and bound verdicts

When no complete row exists, a closed export with no atom/selector-compatible
row is a known `Violation`. A single compatible row may prove a bound
contradiction only when that row's guard itself contains the root guard. A
guard contradiction requires a validator-derived typed countermodel and is
disabled on the existing SELECT/RACE/HANDLE-sensitive root path. Multiple
partial rows do not combine into a violation: absent a complete proof or a
row-local conclusive contradiction, the result is `Unproven`.

This deliberately preserves conservative reachability behavior. In
particular, `guard_contained == false` alone is never a violation. Root ε,
Root K-to-root-ε coverage, and Root K-to-export-ε coverage all use the same
corrected relation.

## P30 results

The formal input is root write `{a=1,b=1}`, unconditional `[0,10]`, with
export E1 `{a=1}`, `p1 == Int64(0)`, `[0,10]`, and E2 `{b=1}`, unconditional
`[0,5]`. The context declares both selector keys, and the source has an exact
Int64 input at port 1. At `p1=1`, E1's guard is false and E2's ceiling is too
small.

Before the fix, the production API returned all nine dimensions `Proven`,
overall `Proven`, and issued `CheckedRootExportCompatibility`. After the fix,
the same checked production inputs yield ε `Unproven`, overall `Unproven`,
and no checked capability. The checker does not claim a global multi-row
countermodel; this is the allowed conservative result. The test separately
compares the exact checked source row, both checked export rows, atom/selector
identity, bounds, guard relation, ε verdict, overall verdict, and capability
issuance.

The independent matrix includes single-row positive and negative witnesses,
scope and qualifier mutations, guard variants, multiple root occurrences
competing for finite export slots, and two-root/two-slot matching. Positive
same-row cases remain accepted.
