# Phase 4B.2F.2 — Adversarial Production Results

## Counting and status meanings

This run designed 62 fixtures. 58 reached a real RootExport production API;
three were blocked by earlier formal validation, and one duplicate-target
fixture was rejected by the resolver builder before RootExport. The permanent
47-case soundness suite and prior Phase 4B.2F's 31 production probes are
separate and are not included.

All cases in the RootExport tables reached stage root_export_checker. “No”
means the checked static capability was not issued. Blocked cases are not
counted as passing checker behavior.

## F01 — 20 core fixtures

| Case | Formal mutation | Expected | Actual verdict | Capability |
| --- | --- | --- | --- | --- |
| R01 | Original P30 split-row witness | Unproven | Unproven | No |
| R02 | Reverse export row ordering | Unproven | Unproven | No |
| R03 | Add unrelated third export row | Unproven | Unproven | No |
| R04 | Bounds row and guard row split | Unproven | Unproven | No |
| R05 | Guard and selector facts split | Violation or Unproven, never Proven | Violation | No |
| R06 | Qualifier and guard split | Violation or Unproven, never Proven | Violation | No |
| R07 | Resource requirement port differs | Violation | Violation | No |
| R08 | Same operation/AtomKey family, scope value differs | Violation | Violation | No |
| R09 | Two root occurrences compete for one slot | Unproven | Unproven | No |
| R10 | Two roots, two adequate slots | Proven | Proven | Yes |
| R11 | Single-row typed guard countermodel | Violation | Violation | No |
| R12 | Single row has insufficient upper bound | Violation | Violation | No |
| R13 | SELECT-sensitive guard mismatch | Unproven | Unproven | No |
| R14 | HANDLE-sensitive guard mismatch | Unproven | Unproven | No |
| R15 | One-branch RACE fixture | Safe rejection before checker | Blocked: checked IR (race_insufficient_branches) | Not reached |
| R16 | Complete same-row positive witness | Proven | Proven | Yes |
| R17 | Open/unknown effect residual | Safe rejection before checker | Blocked: ComponentValidation requires complete operation set | Not reached |
| R18 | Underdeclared known finite upper | Violation | Violation | No |
| R19 | Two-branch RACE-sensitive guard | Unproven | Unproven | No |
| R20 | HANDLE protected and handler paths | Unproven | Unproven | No |

Rows R15 and R17 are earlier-validator outcomes, not RootExport passes. R05/R06
are diagnostic contradictions from an isolated compatible-row case; the
essential property is that partial facts never produce Proven.

## F01 / Recovery interaction

| Case | Mutation | Actual verdict | Capability |
| --- | --- | --- | --- |
| K01 | Single source Recovery action with guard mismatch | Violation | No |
| K02 | Two co-occurring Recovery source nodes | Violation | No |
| K03 | Forward effect and Recovery action use same atom | Violation | No |

These tests exercise the shared Root K/ε relation and the complete source/action
accounting; they do not establish runtime recovery execution.

## F02 — content and association inputs

| Case | Mutation | Actual stage/result | Capability |
| --- | --- | --- | --- |
| R01 | P31-like Root(A)=1, Snapshot(B)=10, top=1 | RootExport / Unproven | No |
| R02 | Separate snapshots, exact same content | RootExport / Proven | Yes |
| R03 | Fresh Root(B)=10, Snapshot(B)=10, top=1 | RootExport / Violation | No |
| R04 | Stale Root, top widened to 20 | RootExport / Unproven | No |
| R05 | Target upper changes 1→2 with matching top change | RootExport / Unproven | No |
| R06 | Snapshot B adds checked component pin | RootExport / Unproven | No |
| R07 | Snapshot B omits target export association | RootExport / Unproven | No |
| R08 | Distinct checker context object | RootExport / Unproven | No |
| R09 | Forward Usage content changes | RootExport / Unproven | No |
| R10 | Failure row changes | RootExport / Unproven | No |
| R11 | Execution requirement changes | RootExport / Unproven | No |
| R12 | Resource attribute value changes | RootExport / Unproven | No |
| R13 | Information Flow source changes | RootExport / Unproven | No |
| R14 | Public input set changes | RootExport / Unproven | No |
| R15 | Effect qualifier changes | RootExport / Unproven | No |
| R16 | Effect guard changes | RootExport / Unproven | No |
| R17 | Target input is removed | RootExport / Unproven | No |
| R18 | Exact snapshot with wider top export | RootExport / Proven | Yes |
| R19 | Target input type changes | RootExport / Unproven | No |
| R20 | Target output set changes | RootExport / Unproven | No |
| R21 | Target output type changes | RootExport / Unproven | No |
| R22 | Recovery usage profile changes | Blocked: Snapshot B ComponentContract validation returned Violation | Not reached |
| R23 | Effect selector scope changes | RootExport / Unproven | No |

The exact original P31 fixture was also reconstructed separately from the
P31-like R01 variant: RootExport / Unproven, proof-context Unproven, body
RequiresEvidence, no capability.

### Association-preservation probes

The two-target association swap reached RootExport. Both snapshots had the
same set of two checked export digests, but each target pointed at the other
export. The checker returned proof-context Unproven, all nine dimensions
Unproven, overall Unproven, no capability. The reverse-insertion-order
variant preserved the same target/export pairs and observed
proof-context Proven; its overall verdict/capability bit was not retained,
so it is not claimed as a compatibility pass. A duplicated target with two
exports was rejected by the resolver builder before RootExport.

A separate duplicate-source-map mutation reached RootExport: dimension I was
Violation, preflight assertion-input validity remained Proven, and no
capability was issued.

## F01/F02 cross matrix

| Case | Input combination | Overall | ε | Capability |
| --- | --- | --- | --- | --- |
| C01 | Stale snapshot + split-row negative | Unproven | Unproven | No |
| C02 | Exact snapshot + split-row negative | Unproven | Unproven | No |
| C03 | Exact snapshot + complete same-row witness | Proven | Proven | Yes |
| C04 | Stale snapshot + complete same-row witness | Unproven | Unproven | No |
| C05 | Exact snapshot + two root rows competing for one slot | Unproven | Unproven | No |
| C06 | Stale snapshot + execution requirement change | Unproven | Unproven | No |

All six reached RootExport. The exact-snapshot positive establishes that
preflight did not disable valid static proofs. Stale identity prevents use of
the old root even when an independent ε witness would otherwise be complete.

## F03 compiler-semantics input mutations

Each mutation reached RootExport and returned compiler-semantics preflight
Unproven, overall Unproven, no capability:

1. Alter normative short statement while keeping RuleId.
2. Alter byte in full Rule Registry.
3. Alter byte in full Specification Registry.
4. Alter an embedded normative source byte.
5. Remove one applied RuleId.

The fifth result confirms exact identity invalidation, not that the applied
RuleId list is a complete dependency closure.

## Final disposition

- Designed: 62.
- RootExport reached: 58.
- Earlier formal validation blocked: 3.
- Resolver-builder rejection before RootExport: 1.
- Incorrect capability issuance: 0 among 57 runs with a retained capability
  result.
- Unexpected false-Proven: 0 among those 57 fully recorded runs.
- Missing recorded compatibility verdict/capability on ordering-only
  identity probe: 1; it is excluded from positive-result claims.
- All 62 designs were attempted through their construction/validation path;
  no case was silently dropped. The three blocked cases are materially
  non-executable at the requested checker stage and are not called passed.
