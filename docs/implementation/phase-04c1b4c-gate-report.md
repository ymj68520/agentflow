# Phase 4C.1B.4C — Gate evidence and readiness

Stage outcome: **C — Formal Model Gap, NOT COMPLETE**. Successful regression gates cannot establish the missing independent D10 proof. No4D admission or B01/B02 closure is claimed.

## Verified code and frozen scope

Start `69caaebb6e47d13cafbabdced45fc80a1a4b7d9a` was clean. Implementation commit `c9e13f3e4a0700f8ead8d7d6e7cedce189406600`; corpus and full-test code HEAD `9c6dbd6a9de02f32e50ef4495c18c8c974b05532`. Subsequent changes are evidence and documentation only. The final documentation-only HEAD is printed in the delivery response.

No production `src` change, normative/AFIDL/ABI/HashV1/registry/golden change, original manifest reclassification, historical Expected rewrite or original38 sidecar update is permitted or performed. Source/Projection coverage publications remain at401/474 with673/459 Pending. The new118562 D9 required-instance comparisons are additive evidence, not a silently closed D10 field set.

## Commands and actual status

| Gate | Status / scope |
| --- | --- |
| `python tools/oracle/test_d10_gap_audit.py` | Pass,7 gap/inventory tests; not semantic comparator mutations |
| `python tools/gen-portable-d10-gap-audit.py --check` | Pass; independent declaration/algebra regeneration, never Actual-derived |
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy -j 2 --all-targets --all-features -- -D warnings` | Pass |
| Focused `d10_original_reachability_and_available_independent_d9_preconditions` | Pass:26 reachedD10,24 D9 compared,2 explicit D9 implementation gaps;118562 required/2112 diagnostic |
| `cargo run -p af-spec --bin spec-check -- check` | Pass:43 specs,404 rules,403 covered,1 intentional exception,0 missing;775 active ABI,17 AFIDL modules,0 cross-reference errors |
| `cargo test --all -j 2` | Pass, exit0 at `9c6dbd6a9de02f32e50ef4495c18c8c974b05532`: all unit/integration/doc tests and unchanged static-surface checks; original512 and original38 regressions pass |
| All `tools/gen-*.py`, two consecutive byte-idempotent rounds | Pass AFTER Cargo completed:30/30 exit0 in each round,1698 files hashed each round,0 byte drift |

Full-test console log: `target/phase-04c1b4c-cargo-test-all.log`, SHA-256 `bf21d31993a492420bea220845b39de9f18d35803a327de1f509d1d883dcd18c`. The source/semantic Expected code is not changed after starting that run. Do not attribute a new expensive full Rust run to a later evidence/documentation-only HEAD. Machine summaries are `coverage/d10-gate-evidence.json` and `coverage/d10-stage-evidence.json`.

## Generator interference disclosure

The initial generator attempt overlapped with Cargo tests reading the same fixtures and running generator `--check` checks. Round1 completed with0 byte drift across1694 tracked/nonignored files. In round2, `gen-portable-root-export-revalidation-corpus.py` failed opening `d9/recovery/outer-iterate-4.ace1` with `OSError: [Errno 22] Invalid argument`. A Windows sharing conflict while fixtures were in use is a plausible explanation, not an established root cause or semantic contradiction. The original artifact remains byte-unmodified in Git; it is not repaired from Actual. No success is claimed for that interrupted double-round attempt.

All30 generators were rerun **serially after the complete Cargo suite**, successfully in both rounds. All1698 tracked/nonignored file bytes were snapshotted before/after each round: empty difference sets and every generator exiting0. Expected-producing Python never reads Cargo Actual reports. Final evidence/documentation-only changes receive a further stable double-round confirmation and fast checks, reported in delivery, without rerunning the expensive Rust suite. The interrupted initial attempt remains disclosed above.

## Readiness decision

Original D10 group:26 retained/executed,0 skipped,24 independently full-field D9 compared,2 D9 implementation gaps; **0 independently full-field D10 compared**. G01 is a formal obligation identity/ordering gap. G02 is a separate lossy projection defect. G03 is the known nonempty subflow Oracle implementation limitation.

Not met: full26-case D9/D10 proof, lossless compiler/context projection, seven typed mapping witnesses, preflight and nine dimension/source/obligation/execution proofs, full verdict/capability derivation,40 new semantic comparator mutations,18 new production-input families and27 new hand calculations. Seven integrity tests and20 historical registered input variants are not substitutes.

The original78-case applicable full-field closure remains incomplete, not just the52 CaseIds outside this subset: current26 D10 proofs are also open. This is not a claim that all78 reachD10; earlier rejections require their actual stage/error comparisons, never invented D10 facts. B01/B02 Conformance Closure stays BLOCKING. F01/F02 CLOSED; F03/F04 OPEN/P2. No false-Proven/P0 is established here. No4D, integration audit, LockGraph, PlanCore, WorkflowVersion, Publication, Deployment or Runtime task starts. No push is performed.
