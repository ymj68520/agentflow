# Phase 4B.2F.2 — Findings

## AUDIT-F01 — Cross-row Effect containment

- **Status / severity:** CLOSED; the prior P0 is fixed and independently
  re-audited.
- **Source:** crates/af-computation/src/root_export.rs:2699-2791;
  authoritative algebra in crates/af-effect/src/algebra.rs:114-211.
- **Normative rule:** one complete effect witness per consumed export slot;
  operation, qualifiers, selector/port/scope, bounds, and guard must be
  compatible on that same row.
- **Formal input:** P30 root {a=1,b=1}, unconditional [0,10]; E1 {a=1},
  p1==0, [0,10]; E2 {b=1}, unconditional, [0,5].
- **Expected / actual:** expected Unproven absent a complete row; actual ε and
  overall Unproven.
- **Capability:** not issued.
- **Independent reproduction:** exact production fixture plus changed row
  order, unrelated row, split witnesses, capacity, and same-row positive.
- **Blocking scope:** no current static RootExport soundness blocker found.

## AUDIT-F02 — Resolver snapshot association

- **Status / severity:** CLOSED; the prior P0 is fixed and independently
  re-audited.
- **Source:** crates/af-computation/src/resolution.rs:107-212;
  crates/af-computation/src/control.rs:321-345,1204-1329;
  crates/af-computation/src/root_export.rs:939-979.
- **Normative rule:** Root and RootRecovery source facts must match the exact
  current resolver snapshot, preserving target/dependency/export associations.
- **Formal input:** two checked targets; same export digest set; swap only
  target-to-export associations while retaining current assertion and top pin.
- **Expected / actual:** expected fail closed; actual proof-context and all
  dimensions Unproven, overall Unproven.
- **Capability:** not issued.
- **Independent reproduction:** association swap, exact-content separate
  snapshots, fresh-root/current-snapshot, content mutations, and exact P31.
- **Blocking scope:** no current static RootExport soundness blocker found.

## AUDIT-F03 — Applied RuleId provenance closure

- **Status / severity:** OPEN, P2.
- **Source:** crates/af-computation/src/root_export.rs:556-760.
- **Normative concern:** the 84 applied IDs are not a complete RuleId dependency
  closure.
- **Evidence:** all 41 specification registry rows and 380 rule registry rows,
  plus 16 complete relevant normative source bundles, are compared as exact
  content identity. The 110 omitted semantic IDs originate in two pinned
  semantic source files. Five content/RuleId mutations fail closed.
- **Expected / actual:** stale content should not reuse static proof; all five
  mutations returned compiler-semantics Unproven.
- **Capability:** not issued for mutated identities.
- **Independent reproduction:** five production input mutations.
- **Blocking scope:** does not block current static use under the documented
  full-content preflight conditions; blocks claiming complete RuleId
  attribution and would block a consumer requiring per-rule closure.

## AUDIT-F04 — Guard-countermodel path sensitivity

- **Status / severity:** OPEN, P2.
- **Source:** crates/af-computation/src/root_export.rs:2523,2699-2801.
- **Normative concern:** a guard contradiction is only decisive with a
  validator-derived countermodel under sound path assumptions.
- **Evidence:** presence of SELECT/RACE/HANDLE anywhere in the root disables
  only the violation countermodel in the fallback. It does not affect the
  normative positive containment path, candidate selection, or slot matching.
- **Expected / actual:** uncertain guard cases may be Unproven, never promoted
  to Proven. SELECT/HANDLE/RACE probes returned Unproven.
- **Capability:** not issued for these negatives.
- **Independent reproduction:** control-sensitive guard variants and cross
  review of every Proven return.
- **Blocking scope:** diagnostic precision only; unresolved cases remain
  rejected as static proof inputs.

## New finding check

No additional confirmed false-Proven or P1 proof-model ambiguity was found.
No new Finding ID is opened.
