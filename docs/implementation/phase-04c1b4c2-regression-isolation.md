# Executable legacy/new-edition regression isolation plan

**NOT IMPLEMENTED. No old test, input, generator or Expected is changed here.**

## Two independently named evidence domains

Legacy environment pin: complete commit `8b4e3c64ee1d84fc38b6faa9b0b1257deb18873d`, its Cargo.lock, rust-toolchain.toml if present (otherwise capture actual rustc/cargo -Vv), Python version, OS, all generator files and canonical inputs/Expected hashes. The future T0 artifact must record these actual environment versions and retain a read-only git object bundle/source archive plus toolchain/container/Windows runner recipe. A commit SHA alone is not a retained executable environment. No old dispatcher is added to successor production.

New-edition environment: approved T8 commit with separate target directory, corpus namespace and independent oracle dependencies. Both invoke real validators, but never share writable corpus or compiled current-profile source caches.

| Historical corpus | Count | New-build disposition |
| --- | --- | --- |
| computation-control |58| Old algebra-only corpus can run unchanged in new build; has no compiler identity field; keep propagation/source-order laws |
| computation-advanced-control |72| Same, unchanged frozen arithmetic/operator inputs/Expected |
| computation-scope-closure |55| Same, Scope/resource/closure algebra and unknown authority boundaries preserved |
| computation-root-recovery |44| Same, K source/factor/guard/algebra corpus, independent of compiler report edition |
| computation-root-export |35| Full v1 report/obligation representation is edition-sensitive: authoritative old differential runs in legacy job; new edition gets new inputs/Expected |
| computation-root-export-soundness |47| Original full reports run in legacy job; split-row and stale-root safety properties get independently instantiated successor cases, not overwrite47 goldens |
| portable-proof-codec |60| All60 remain legacy. New runner reuses early-stage negative bytes only as below; positives/current-profile cases cannot be forced through new D4 |
| portable-static-reconstruction |63| All63 frozen exact-profile inputs archive+legacy run; new analogues reconstructed from independent new tuple; unchanged old tuple rejects by D4 before D5 |
| portable-root-export-revalidation |78| All78 frozen archive+legacy run; new78 applicable scenario namespace required, no label swaps |

Total512=229 neutral algebra +82 RootExport report-sensitive +201 Portable exact-content-sensitive. "Neutral" refers to these four original algebra corpus assertions, not a permission to keep old report-string tests in any new typed-view comparator.

**Compatibility condition for the229 unchanged rows is explicit:** capture typed facts in a sidecar at original branches while preserving the existing algebra-summary/legacy-carrier values and old versioned corpus projection byte-for-byte. The65/27 constructors may gain fact-sink inputs/links, but their old reference/prerequisite/reason/inherited-carrier outputs used by these four frozen corpora are not overwritten with new canonical strings. The successor exposes a separately typed/versioned G01 view whose required references/prerequisites/identity strings are derived from captured facts, never from that legacy carrier. New typed primary Some can coexist with old carrier None. This is interface versioning with both required contracts retained, not a DiagnosticOnly demotion or an old-profile acceptance path. Label the old projection LegacyCarrierV1 and never advertise it as a complete G01 report. No legacy RootExport capability constructor remains available in the successor. If preserving any229 full-corpus assertion is infeasible, activation is blocked until explicitly approved routing changes; do not silently shrink its comparator to algebra fields and call the original test passed.

Additional unchanged noncompiler suites run in new build: ACE1/Digest, AFIDL registry-independent parser/type checks, Semantic Core/IR/dataflow/subflow-interface, effect/failure/resource/flow/usage/cost/recovery/execution/component algebra and context validators. Registry count/static-surface tests that hard-code43/404 may need **new-profile test routing**, not changes to old corpus facts; old assertions remain legacy evidence. New-profile static-count assertions independently expect approved46/436/775/17/14. Old checker-bound RootExport integration tests (including full report shape and private constructor expectations) route to legacy; corresponding successor semantic safety laws must be re-instantiated, not deleted.

## Per-case routing rather than blanket acceptance

Codec manifest actually has expected stages D0=13,D1=9,D2=14,D3=9,D4=15. Thus45 earlier-stage negatives may be run on the successor unchanged as rejection inputs if the original cause is reached before profile admission; preserve original stage/cause and assert the new outer error category separately. Remaining15 contain6 successful D4 cases and9 D4 policy negatives; keep their original Expected in legacy. New-current-only tests independently state UnsupportedSemantics for complete valid old tuples and InvalidIdentity for mixed known tuples. Do not assume all15 have one rejection class or that a malformed old tuple reaches D4.

Every case-routing row in future `migration-routing-v2.json` (NOT IMPLEMENTED, new namespace) must contain oldCaseId,oldInputHash,oldExpectedHash,originalStage,legacyJob,newEarlyRejectJob flag,newUnsupportedJob flag,newAnalogueCaseId,reason. Coverage gate: exactly512 unique legacy rows;229 unchanged neutral rows;82 report-sensitive rows;201 portable rows; all applicable new scenario counterparts enumerated. If fixture inspection contradicts a proposed neutral row, move the *new-build routing* with explicit reason; never mutate its old Expected. Legacy512 must still execute.

## Operator recipe for the future implementation branch

All paths below are proposed isolated CI workspaces, not commands executed this phase.

1. Materialize legacy checkout at T0 in a separately named exact directory, e.g. `E:/AgentFlowEvidence/legacy-8b4e3c64`; new checkout in `E:/AgentFlowEvidence/g01-v1`. Verify each resolved path and requested commit before creating a checkout. Never reset this user's working tree.
2. Use separate `CARGO_TARGET_DIR` values per environment. Pin Python imports to each tree. Run old complete cargo gates and all30 generators twice in legacy; before/after whole nonignored file hashes must match. Mount/copy old corpus read-only for the new build.
3. Future new test runners partition frozen full-report tests into legacy-only jobs/features and new-edition tests into owner@2 jobs. Default new-build `cargo test --all` runs all appropriate neutral/new tests and migration-admission tests, not a silently altered old-profile success assertion. This harness routing itself requires explicit implementation review.
4. New suite/generator directories: `spec/conformance/g01-v1/`, `spec/conformance/portable-root-export-v2/`, new model/compiler/routing manifests and new independent Expected. Proposed generator `tools/gen-g01-v1-conformance.py` and runner `af-portable/tests/g01_v1_conformance.rs`: **NOT IMPLEMENTED**. No writes to any original512 inputs/Expected; generator --check cannot repair them.
5. CI publishes separate counters: LegacyFrozen512/512; NewNeutral229/229; OldTupleAdmission (expected Unsupported count); NewEditionActual (real executed cases only); G01Design64 NotExecuted until materialized; G02/G03 readiness; Full78Closure. Never sum duplicate neutral runs as new production cases.
6. Enforce deny-write paths and hash comparison against T0 for all legacy corpora. Fail if generator output escapes its new namespace, oldCaseId is relabeled as new, legacy hashes drift, or independentExpected imports Rust Actual/old report strings.

The plan is executable routing and environment preservation, not a fabricated current v2 all-green result. A new-profile cargo job passing old unsupported assertions is not evidence of accepting old positives. A legacy job failing remains a real frozen-evidence regression. Old-profile UnsupportedSemantics in the correctly labeled admission job is an intended **pass**, not an excuse to skip testing.

## Future generator routing, not frozen-output overwriting

The future generator-routing-v2 manifest must list all30 old script names and every new script explicitly. Eight old content/report-sensitive tools run against the old tree only: gen-computation-root-export-corpus.py,gen-root-export-soundness-corpus.py,gen-portable-proof-codec-corpus.py,gen-portable-root-export-revalidation-corpus.py,gen-portable-static-reconstruction-corpus.py,gen-portable-d10-gap-audit.py,gen-portable-d9-control-full-oracle.py,gen-portable-d9-recovery-full-oracle.py. New independently versioned generators cover their successor inputs/Expected in new directories. The registry authoring generator runs once per appropriate isolated edition with its approved descriptors;the other21 tools are candidates for both jobs subject to unchanged-byte proof. No old content-sensitive generator is allowed to read a live new registry then write an old corpus path.

T7's two-round gate means all30 frozen scripts in the legacy job plus every applicable/new registered generator in the successor job, each with explicit routed status and byte guards. An unsupported old generator on the new tree must fail safely before writing and is reported as routed-to-legacy,not silently counted as executed/pass. New CI does not loop the old glob destructively. Routing,write guards and version-specific entry points are required implementation deltas,NOT IMPLEMENTED here. This phase itself still runs all actual current30 twice without changing them.
