# Phase 4B.2F — Effect and Recovery Accounting Audit

## Independent hand calculations

These are direct integer calculations from the accepted D2 equations
`base = Forward × activation × attempts`, with each logical Repeat factor
applied once to the root path. No Rust output was used as the arithmetic
oracle.

| Shape | Local action upper | Root action upper | Recovery Usage at q=2 |
| --- | ---: | ---: | ---: |
| Retry3 outside CompScope; F=1,A=2,T=2 | 1×2×2 = 4 | 4×3 = 12 | 24 |
| Retry3 inside CompScope; F=1,A=2,T=2 | 1×3×2×2 = 12 | 12 | 24 |
| Iterate4 outside CompScope; F=2,A=1,T=2 | 2×1×2 = 4 | 4×4 = 16 | 32 |
| Retry3 × Iterate4 outside CompScope; F=2,A=2,T=2 | 2×2×2 = 8 | 8×4×3 = 96 | 192 |
| Same full atom, F=1,A=2,T=3 | — | action=6; shared E row must cover 1+6=7 | — |
| Two co-occurring K sources with action uppers 4 and 6 | — | 4+6 = 10 for the shared atom row | — |

`max_inflight` does not enter any total. K Action and K Usage are coverage
inputs, not additional additions to the already-derived epsilon/U envelopes.
Source review of `derive_root_recovery_node` confirms factor identities are
keyed by caller digest, node and Retry/Iterate kind; checked multiplication
and operator-specific effect/usage composition preserve overflow/unknown.
This does not cure the RootExport comparison P0 below.

## D1 guard relation

The source K possibility starts from the Forward guard and the declared action
epsilon row is the ceiling for that full atom. Root guard translation uses
input use-site maps. A failed `guard_contained` alone is Unproven unless a
typed reachable countermodel exists. The temporary `narrow_action_guard`
production mutation returned Unproven; no false violation was observed.

`f011eef` disables guard countermodels when the root tree contains any
SELECT/RACE/HANDLE. This is fail-closed but broader than path-local analysis:
an unrelated branch can suppress a counterexample for another effect and
leave Unproven. Record as AUDIT-F04 (P2 precision/coverage), not a soundness
defect.

## AUDIT-F01 interaction

The E containment fallback does not preserve the D1 “same exact ceiling” rule.
The typed source-to-root map itself was valid, the export was a checked
ComponentContract, and the two scope rows were legal distinct EffectAtoms.
`af_effect::contains` correctly failed to find one row satisfying all
predicates. The fallback’s independent existential queries then spliced E1's
bound and E2's guard, skipping the reachable `port 1 = 1` countermodel. The
production result was epsilon Proven, nine dimensions Proven, and a checked
capability. See AUDIT-F01 in
[false-Proven analysis](phase-04b2f-false-proven-analysis.md).

## AUDIT-F02 interaction

Root K/E/U facts are summarized from the resolved child contract. If that
target export was read from resolver snapshot A but RootExport validates only
the assertion against snapshot B, the source K/E/U multiset is stale relative
to the pinned dependency. The test used no K action, so epsilon directly
reproduced it; the same missing snapshot binding would affect K source facts
when present. Root source export A upper 1, current asserted target export B
upper 10, root export C upper 1 still yielded checked compatibility. No
independent K data was dropped in this fixture, but K is not protected from
the provenance mismatch. See AUDIT-F02.
