# Gate report — Phase4C.1B.4C.3

**Outcome A — Registered Edition-2 Staging Accepted.** All required fresh code/legacy/original-directory generator gates passed. Positive proof/release remains NOT ACTIVATED;this is T1/T2 only.

Start/legacy HEAD:`abc91d533b4d62e91226722e1d1fcaa30bd00883`;legacy tree:`42b548ec66a22813f07b77abb5385707d43256cc`. Initial full staged code gate5657ab336fa6fdad1e31572b420fd5d74025f955 passed;after the registry-writer refinement,the complete four-command gate was freshly rerun and passed at **bbee6fbb41c31cda868604827b73ff557fc40eed**. Final documentation HEAD is resolved at delivery;fast documentation checks are not a full Cargo rerun.

## Fresh staged commands at final code HEAD bbee6fb

| Command | Actual exit |
| --- | --- |
| cargo fmt --all -- --check |0 |
| cargo clippy -j2 --all-targets --all-features -- -D warnings |0 |
| cargo test --all -j2 |0 |
| cargo run -p af-spec --bin spec-check -- check |0 |

Logs:`target/phase-04c1b4c3-cargo-test-all.log`,`target/phase-04c1b4c3-spec-check.log`. Actual spec-check:46 specifications,436 rules,435 reference-covered,1 intentional exception,0 missing;775 active ABI,0 retired/reserved,17 AFIDL,0 cross-reference errors. HashV1 domains14 unchanged.

The final-code full test log is `target/phase-04c1b4c3-cargo-test-all-final-code.log`;all four final-code commands exited0 after the writer refinement,not inferred from the earlier run.

Current native edition identity/state tests3/3 and Portable registration/admission/model tests3/3 passed. Stable staged CaseIds:32 registration +9 Portable +10 native=51. Supplemental direct checker suite exercises35 old declaration samples without issuing current Checked (or records legitimate earlier component rejection). No future48+16 design law is marked Executed. Coverage means registered/staged responsibility,not full G01 semantics.

## Fresh pinned legacy commands

A fresh exact Git archive of the start HEAD was compiled/tested using a separate build directory. fmt,clippy,cargo test --all -j2,spec-check all exited0 in this phase. Actual old registry report43/404/403/1/0,775 active ABI,17 AFIDL. Full test log:`target/phase-04c1b4c3-legacy-cargo-test-all.log`.

All512 historical cases remain58/72/55/44/35/47/60/63/78,not new-edition compatibility. Actual old D9 Control22 full,Recovery16 full,D10-group D9 evidence24/26 with two G03 gaps and0 full D10;P30/P31 integration/soundness tests passed in the real old build. These are not replayed prior-phase logs.

Environment:Windows x86_64-pc-windows-msvc;rustc1.92.0(ded5c06cf21d2b93bffd5d884aa6e96934ee4234),cargo1.92.0,Python3.14.2. Archive source and separate CARGO_TARGET_DIR paths are in [legacy record](phase-04c1b4c3-legacy-isolation.md). Every1732 tracked archive file matched its actual Git blob identity after the run.

## Freeze audit

Actual audit:`target/phase-04c1b4c3-audit.json` /helper .py. Old43 Specification and404 Rule full JSON records equal;all43 old normative source bindings/content equal;1144 old conformance files raw-byte equal to retained baseline;old ABI/AFIDL/domain/primitive/rule-area bytes equal to exact legacy source. That retained raw-byte artifact is freeze evidence only,never an isolated-generation substitute for the required gate.

## Original-directory generator gate

PASS after a fresh complete retry. Actual current inventory31 scripts (old30 +gen-g01-edition2-staging-corpus.py). Both consecutive complete rounds executed in E:/Projects/agent with exit0 and all nonignored path/byte hashes equal. No isolated run or patched-together successes count.

First original-directory attempt at5657ab3/documentation0797bed:Round1 all31,1774 files,zero drift,164.55s;Round2 stopped at script23 gen-registry-instances.py with OSError22 opening spec/registry/rules.json. Failed attempt log:`target/phase-04c1b4c3-generators-original.log`. Target250239 bytes/SHA2568f93ec2e54bbeb248b518f2f60152695b2708a7bf57d3239415f588463d7ff3f still exactly matched the code-gate blob;no truncation/drift occurred. This interrupted attempt is not counted as a double round.

Reviewed refinement bbee6fb:the registry author still computes complete canonical bytes and asserts old row equality,but verifies/returns when existing output bytes already exactly match,instead of redundant truncation. Different desired bytes still take the normal write path;exceptions are not swallowed and mismatches are not ignored. This is a generator idempotence/freeze refinement,not an identified fix of the unknown Windows root cause. Fresh full Rust gates passed afterward;the required generator retry must start from Round1.

Historical generators now explicitly recalculate from hash-pinned Edition1 registry/declaration inputs and verify immutable physical output bytes rather than overwrite/repair old Expected. This is a reviewed freeze/version-isolation implementation,not a claimed Windows I/O repair. All calculation/validation executes in the original directory. New Edition2 output uses a distinct namespace and an unchanged-byte write guard. Any I/O/mismatch failure is a separate failed attempt requiring a complete fresh double round.

| Fresh consecutive original-directory round | Actual scripts exit0 | Nonignored files | Changed/added/removed | Seconds |
| --- | --- | --- | --- | --- |
| Final1 |31/31|1774|0|159.35 |
| Final2 |31/31|1774|0|160.42 |

Full successful log:`target/phase-04c1b4c3-generators-original-final.log`;process exit0;assertion original baseline=Round1=Round2. Helper:`target/phase-04c1b4c3-generators.py`. Source HEAD bbee6fbb41c31cda868604827b73ff557fc40eed,with only gate/final-receipt documentation edits present and included in both snapshots. No individual generator retry is hidden inside a round. The earlier interrupted attempt remains separate evidence and is not counted.

## Completion limit

OutcomeA is limited to registered staging after the actual original-directory pass above. T3 producers,T4 normalizer,T5 lossless report,T6 full oracle,T7 audit,T8 activation remain blocked;Edition2 positive proof/release NotAuthorized. F01/F02 CLOSED,F03/F04 OPEN/P2;G02 OPEN/P1,G03 OPEN/P2;B01/B02 and B03–B07 blocking. No push or later phase is authorized.
