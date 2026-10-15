# Gate report — Phase4C.1B.4C.3

Start/legacy HEAD:`abc91d533b4d62e91226722e1d1fcaa30bd00883`;legacy tree:`42b548ec66a22813f07b77abb5385707d43256cc`. Full staged code-gate HEAD:`5657ab336fa6fdad1e31572b420fd5d74025f955`. Final documentation HEAD is resolved at delivery after commits;documentation fast gates are not a full Cargo rerun.

## Fresh staged commands at5657ab3

| Command | Actual exit |
| --- | --- |
| cargo fmt --all -- --check |0 |
| cargo clippy -j2 --all-targets --all-features -- -D warnings |0 |
| cargo test --all -j2 |0 |
| cargo run -p af-spec --bin spec-check -- check |0 |

Logs:`target/phase-04c1b4c3-cargo-test-all.log`,`target/phase-04c1b4c3-spec-check.log`. Actual spec-check:46 specifications,436 rules,435 reference-covered,1 intentional exception,0 missing;775 active ABI,0 retired/reserved,17 AFIDL,0 cross-reference errors. HashV1 domains14 unchanged.

Current native edition identity/state tests3/3 and Portable registration/admission/model tests3/3 passed. Stable staged CaseIds:32 registration +9 Portable +10 native=51. Supplemental direct checker suite exercises35 old declaration samples without issuing current Checked (or records legitimate earlier component rejection). No future48+16 design law is marked Executed. Coverage means registered/staged responsibility,not full G01 semantics.

## Fresh pinned legacy commands

A fresh exact Git archive of the start HEAD was compiled/tested using a separate build directory. fmt,clippy,cargo test --all -j2,spec-check all exited0 in this phase. Actual old registry report43/404/403/1/0,775 active ABI,17 AFIDL. Full test log:`target/phase-04c1b4c3-legacy-cargo-test-all.log`.

All512 historical cases remain58/72/55/44/35/47/60/63/78,not new-edition compatibility. Actual old D9 Control22 full,Recovery16 full,D10-group D9 evidence24/26 with two G03 gaps and0 full D10;P30/P31 integration/soundness tests passed in the real old build. These are not replayed prior-phase logs.

Environment:Windows x86_64-pc-windows-msvc;rustc1.92.0(ded5c06cf21d2b93bffd5d884aa6e96934ee4234),cargo1.92.0,Python3.14.2. Archive source and separate CARGO_TARGET_DIR paths are in [legacy record](phase-04c1b4c3-legacy-isolation.md). Every1732 tracked archive file matched its actual Git blob identity after the run.

## Freeze audit

Actual audit:`target/phase-04c1b4c3-audit.json` /helper .py. Old43 Specification and404 Rule full JSON records equal;all43 old normative source bindings/content equal;1144 old conformance files raw-byte equal to retained baseline;old ABI/AFIDL/domain/primitive/rule-area bytes equal to exact legacy source. That retained raw-byte artifact is freeze evidence only,never an isolated-generation substitute for the required gate.

## Original-directory generator gate

PENDING at this intermediate receipt. Actual current inventory31 scripts (old30 +gen-g01-edition2-staging-corpus.py). It must run two consecutive complete rounds in E:/Projects/agent with exit0 and all nonignored path/byte hashes equal. No isolated run or patched-together successes count.

Historical generators now explicitly recalculate from hash-pinned Edition1 registry/declaration inputs and verify immutable physical output bytes rather than overwrite/repair old Expected. This is a reviewed freeze/version-isolation implementation,not a claimed Windows I/O repair. All calculation/validation executes in the original directory. New Edition2 output uses a distinct namespace and an unchanged-byte write guard. Any I/O/mismatch failure is a separate failed attempt requiring a complete fresh double round.

## Completion limit

No OutcomeA until original-directory double-round evidence is recorded. T1/T2 is registered staging only. T3 producers,T4 normalizer,T5 lossless report,T6 full oracle,T7 audit,T8 activation remain blocked;Edition2 positive proof/release NotAuthorized. F01/F02 CLOSED,F03/F04 OPEN/P2;G02 OPEN/P1,G03 OPEN/P2;B01/B02 and B03–B07 blocking. No push or later phase is authorized.
