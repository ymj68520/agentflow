# Phase 4B.2E.1 — Gate Report

## Baseline and categorized commits

Starting HEAD was `8a2c509c7c8a06241074a4a5af3b8de83d61dbf1`; the starting
worktree was clean. The implementation was committed in these categories:

| Category | Commit | Scope |
| --- | --- | --- |
| D1 component guard | `f8b05f2` | Typed K/action-epsilon proof, validator integration, and ComponentContract guard rules |
| D2 root recovery | `5ee8756` | Typed local/root bounds, root source/profile fold, multiplicity and E/U coverage |
| Conformance and registry | `da12c0c` | Independent oracle, 44-case production differential, mutation suite, Rule/Suite registration, and implementation reports |

The final gate-report file is a documentation-only follow-up commit; its full
commit ID and the resulting final HEAD are recorded in the completion message.

## Post-implementation gates

All required gates passed after the three implementation commits:

| Gate | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy --all-targets --all-features -- -D warnings` | Pass |
| `cargo test --all` | Pass; no failed tests |
| `cargo run -p af-spec --bin spec-check -- check` | Pass; 0 cross-reference errors |
| Control production differential | 58/58 |
| Advanced Control production differential | 72/72 |
| Scope/Closure production differential | 55/55 |
| Root Recovery production differential | 44/44 expected-stage executions and full comparisons |
| Root Recovery skipped / mismatch / uncovered required fields | 0 / 0 / 0 |
| Comparator mutations | 18/18 detected |

The Root Recovery manifest stage distribution is 18 ComponentValidation
cases, one IR-stage rejection, and 25 root-recovery cases. AUDIT-10 now stops
at ComponentValidation with a typed countermodel; K20 compares only internal
root facts and explicitly defers its export-epsilon assertion.

## Generator idempotency

All 22 `tools/gen-*.py` generators ran twice. A SHA-256 comparison of all 949
files under `spec/` after pass one and pass two found no changes. The registry
generator was updated to preserve the new RuleIds and suite when regenerating
the canonical registries.

## Registry and frozen-surface comparison

`spec-check` reports:

```text
Specifications: 40
Normative rules: 366
Rules covered: 365
Intentionally uncovered: 1 existing architecture rule
Missing coverage: 0
Active ABI assignments: 477
AFIDL modules checked: 16
Cross-reference errors: 0
```

The final diff against the baseline is empty for `spec/idl/`,
`spec/tables/hash-domains.json`, `spec/registry/abi-numbers.json`, and the
existing Semantic Core, Semantic IR, and ComponentContract golden directories.
The 14 HashV1 domains are unchanged. No Root/Workflow digest, AFIDL field,
ABI assignment, encoding, or old digest golden was added or revised.

## Remaining boundary and phase status

- P0: none identified.
- P1: RootExportAssertion and export-specific epsilon/K/U plus I/O/resource/F/X
  mapping remain Phase 4B.2E.2 work. No export checker was added here.
- P2: control-path guard evidence is retained as typed path and coordinate
  data, but the analyzer does not invent general Boolean implication from a
  selector to an input expression. A relation not proved by component-local
  frozen containment remains Unproven.
- P2: a same-atom shared epsilon ceiling whose contribution includes another
  Forward identity that cannot be isolated remains Unproven rather than
  guessing a Forward seed.
- P3: none recorded.

D1's component-local guard checker and D2's internal root action/Recovery-
Usage aggregation are implemented. Static Proven does not mean receipt
eligibility or runtime safety. Work stops before Phase 4B.2E.2.
