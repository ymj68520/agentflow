# Phase 4B.2E.1 — Independent Conformance

## Corpus and execution stages

`tools/gen-computation-root-recovery-corpus.py` is an independent Python
oracle. Its expected values are computed from input atoms, guards, bounds,
binding/profile data, and a separately implemented control tree fold. It does
not call Rust or read production output. The generated manifest contains 44
CaseIds:

- G01–G16: guard direction, typed countermodels, unknown domains, root input
  coordinates, exact/dynamic selection, receipt separation, and early
  component rejection.
- K01–K24: local/root arithmetic, operator folds, nested owners, same-atom
  accounting, source/profile closure, E/U coverage, unbounded count, and
  incomplete source rejection.
- `G14-root-path`: an exact Select path whose unselected positive component
  action has root upper zero while its component-local guard proof remains
  separate.
- `AUDIT-10`: production reproduction of the Phase 4B.2C guard gap.
- `K06-underdeclared`: a shared same-atom epsilon ceiling that conflicts with
  a mandatory Forward lower.
- `K25-overflow`: checked U64 meter multiplication overflow.

Each case records `expected_stage`, `production_api`, exact required expected
fields, reachability at ComponentValidation / IRValidation / LeafResolution /
ControlDerivation / RootRecoveryValidation / lower-level algebra, and whether
an export assertion is deferred. The runner calls `validate_contract_case`
for component-stage cases, `analyze_compose_case` for root-stage cases, and
the same production pipeline through IR validation for K19. K20 runs only
internal root E/U checks; its export-epsilon assertion is marked deferred to
Phase 4B.2E.2.

## Differential requirements and results

The Rust runner enforces exact manifest/corpus set equality, stage/API
classification, required-field presence, and recursive semantic comparison.
The 43-case gate is designed to require:

```text
Stage-executed = manifest = 44
Fully-compared = manifest = 44
Skipped = 0
Unexpected mismatch = 0
Uncovered required fields = 0
```

Root action comparisons include the source contract alias, computation node,
forward entry ordinal, full RecoveryActionIdentity, exact RecoveryUsageBinding
profile, LocalActionBound, RootActionBound, owner, mapped resource slot,
guard result/coordinates, control-path evidence, root E/Usage projections,
source closure, and proof dimensions. Caller SemanticIrDigest values are
checked for exact equality between each action source and its root coordinate.

The isolated comparator mutation test covers all 18 specified error classes:
reversed guard direction, automatic Violation on failed containment, invalid
typed witness, changed action input, dropped path evidence, retry double
application, missing Iterate factor, max_inflight misuse, local/root coordinate
confusion, deleted Race loser, summed Select alternatives, collapsed
same-atom events, dropped profile, source-node-only closure, double-added K
Usage, unknown meter set to zero, overflow promoted to Proven, and removed
RequiresEvidence.

The prior independent suites remain separate and unchanged: Control 58/58,
Advanced Control 72/72, and Scope/Closure 55/55.
