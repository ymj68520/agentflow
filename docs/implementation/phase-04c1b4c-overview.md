# Phase 4C.1B.4C — Outcome C: Formal Model Gap

Start: `69caaebb6e47d13cafbabdced45fc80a1a4b7d9a`, initially clean. This stage is **NOT COMPLETE**. D10 core full-field closure is **OPEN**. B01/B02 Conformance Closure remains **BLOCKING**. No admission to 4C.1B.4D is asserted.

## What is preserved

The original manifest still has D9-Control22, D9-Recovery16, D10 26, Context-Resolver-Compiler8 and Cross-Process6: exactly78 CaseIds. Nothing is reclassified. Historical512 input/Expected cases, the original38 full D9 Expected,151 existing comparator mutations and74 hand calculations are not rewritten. All production source, normative semantics, ABI/AFIDL, registry, hash domains and goldens remain frozen.

## Additive work

`tools/oracle/d10_gap_audit.py` independently materializes all26 original D10 inputs, including the20 registered `source_node=2..21` input mutations. Each resulting canonical byte string has its own length/SHA-256. The inventory records full checked-IR/export pins, context and resolver **declaration** checksums, root assertion ID, seven mapping families and subflow rows. Declaration checksums are not misrepresented as complete typed context identities or new HashV1 domains.

Existing independent Control/Recovery algebra derives24 complete D9 precondition sidecars in `spec/conformance/portable-root-export-revalidation/expected-full/d10-preconditions/`. Both P31 cases explicitly retain `OracleImplementationGap`: the prior model deliberately rejects nonempty CALL_SUBFLOW interfaces. This is not a missing formal rule.

The Python full-D10 Expected entry point raises `OracleGap` rather than returning a partial report. No D10 Expected is generated. Seven new Python tests check grouping, mutation materialization, shape validation, fail-closed behavior, required obligation classification, citation anchors and the current projection limitation. They are gap/inventory integrity tests, not40 new D10 comparator mutations or27 semantic hand calculations.

The additive Rust test under `crates/af-portable/tests/support/d10_gap_conformance.rs` runs the real supported codec and fresh D5–D10 pipeline for all26 inputs. It compares every D9 field for available Expected using the unchanged executable grammar and two-sided validation. It records the two unavailable P31 Expected as gaps, not skips or successful comparisons. Production D10 observations go only into ignored `target/conformance/d10-gap-audit/`; no Expected-producing code consumes them.

## Why full-field D10 cannot be claimed

See [formal gap and projection review](phase-04c1b4c-proof-gaps.md). Three findings are deliberately distinct:

| ID | Classification | Consequence |
| --- | --- | --- |
| ORACLE-D10-G01 | P1 formal conformance model gap | Exact required obligation string identity/order is not specified by frozen normative sources; cannot fill from Actual |
| ORACLE-D10-G02 | P1 projection/coverage defect | Legacy compiler/content/manifest projection is lossy; cannot serve as full-content comparison |
| ORACLE-D10-G03 | P2 independent Oracle implementation gap | Two P31 nonempty subflow D9 interfaces need a declaration-driven extension |

No finding here establishes production false-Proven. No P0 is registered. Full-field D10 compared count is0/26, not26/26. The current cumulative38-case coverage contract is intentionally unchanged rather than crediting fields merely because an unverified/partial projection exists.

## Reproduction

Run from the repository root in PowerShell:

```powershell
python tools/gen-portable-d10-gap-audit.py --check
python tools/oracle/test_d10_gap_audit.py
cargo test -p af-portable --lib -j 2 d10_original_reachability_and_available_independent_d9_preconditions -- --nocapture
```

The first two commands never execute Rust or read Actual. The Rust test writes `target/conformance/d10-gap-audit/report.json` with actual stages and precise D9 comparison counts. It does not certify D10.

The [final report](phase-04c1b4c-final-report.md) distinguishes committed code/full-test HEAD from the final documentation-only HEAD and lists all requested acceptance facts, including unmet targets. Work stops at this gap boundary; no integration audit, LockGraph, PlanCore, WorkflowVersion, Publication, Deployment or Runtime work is started. No push is authorized or performed.
