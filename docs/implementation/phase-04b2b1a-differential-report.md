# Phase 4B.2B.1A — Differential Coverage Status

Baseline: `a00fe81dac530b2ec61b40becbbcecc2c9793f7a`. This report records
the actual status at the end of the current work; it does **not** present the
manifest count as execution evidence.

## Per-category status

The existing Rust inventory test structurally reads all fixture JSON and
checks the number of unique IDs plus the set of expected verdict classes. It
does not construct a `CheckedIrDocument`, resolve checked leaves, call
`derive_control_contract`, or compare an Actual projection with Expected.
Accordingly, all 58 cases remain unexecuted for production differential
purposes. “Failed: not measured” is intentional; no mismatch count can be
claimed when no Actual result exists.

| Category | Manifest | Executed | Fully compared | Failed | Unexecuted |
|---|---:|---:|---:|---:|---:|
| golden | 1 | 0 | 0 | not measured | 1 |
| negative | 1 | 0 | 0 | not measured | 1 |
| sequence | 10 | 0 | 0 | not measured | 10 |
| select | 10 | 0 | 0 | not measured | 10 |
| parallel | 11 | 0 | 0 | not measured | 11 |
| race | 10 | 0 | 0 | not measured | 10 |
| proof | 11 | 0 | 0 | not measured | 11 |
| equivalence | 4 | 0 | 0 | not measured | 4 |
| **Total** | **58** | **0** | **0** | **not measured** | **58** |

There is no runner that marks individual vectors ignored; therefore “0 ignored”
would be misleading. The differential acceptance condition is unmet, not
passed with 58 skips.

## Difference findings

- No Expected/Actual Rust semantic differences were measured because no
  production result was generated for a corpus case.
- Oracle review found and corrected the revision-1 open-row completeness flag
  and missing dynamic-SELECT decision dependencies. Revision 2 records these
  changes; they are Oracle/fixture corrections, not Rust differential fixes.
- `open-effect-row-remains-open` is confirmed not representable as a resolved
  checked leaf through the current API: the checked-contract validator returns
  an Unproven resource/cross-dimension proof for an open row, and snapshot
  insertion accepts only an all-Proven `CheckedComponentContract`. Its
  normative Expected remains frozen, but that CaseId has not passed the
  production Analyzer.
- The other 57 fixtures still need formal input materialization and full
  projection comparison. Their representability has not been established by
  execution. No mismatch is silently waived.

## Mutation tests

Expected-value mutation tests A–D and manifest-removal test E have not run:
there is no Actual-vs-Expected runner/comparator to mutate yet. The Rust test
that proves the open-row checked-contract boundary is a focused validator
test, not a substitute for those runner mutations.

