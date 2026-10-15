# Design gate report — passed, design-only acceptance

Design scope only. No code/registered norm/Expected modification. Start and **Full Code Gate HEAD**: `293066e865e25dad695cd2d774f558b70eac43f3`. Final documentation HEAD and post-commit verification scope are reported in delivery; no new full Rust run will be falsely attributed to that later HEAD.

| Gate | Current evidence |
| --- | --- |
| Exact start / clean tree | Verified293066e865e25dad695cd2d774f558b70eac43f3, clean |
| Complete producer audit |6 direct string sites,3 inherited constructors,11 prerequisite arms/20 literal occurrences, all reference forms,65 Control +27 Leaf constructor-site census |
| Design vectors |48 declarative cases; EVERY Actual Status = Not Executed; zero added production cases |
| Design artifacts |16 requested documents plus38-item final report; unregistered draft, no official allocations |
| `cargo test --all -j 2` | PASS, exit0; full-code HEAD293066e865e25dad695cd2d774f558b70eac43f3; all unit/integration/doc/static-surface regressions |
| fmt / strict clippy / spec-check | PASS;43 specifications/404 rules/403 covered/1 intentional exception/0 missing; ABI775/AFIDL17/domains14/zero cross-reference errors |
| All `tools/gen-*.py` twice | PASS in a fresh complete serial attempt:30 scripts each round,1715 files hashed each round,0 byte drift;2s quiescence between scripts |
| Frozen assets / final tree | Baseline diff: ONLY17 new design Markdown files; no registered/production/Expected drift. Final clean-tree confirmation in delivery |

Commands:

```powershell
cargo fmt --all -- --check
cargo clippy -j 2 --all-targets --all-features -- -D warnings
cargo test --all -j 2
cargo run -p af-spec --bin spec-check -- check
```

Full console log: `target/phase-04c1b4c1-cargo-test-all.log`; SHA-256 `367232f64b4cce5c1880dff5e4968e39f24ef4ee976833bc192345abb7a586e8`. It includes old512 CaseIds' regression, original38 full D9 tests, current26 reachability/24-D9 comparison and the existing D10 gap tests, plus unit/integration/doc/static-surface tests. All passed. Original38 required instances remain285041; new24 remain118562; D10 full-field remains0 and P31 D9 gaps2. The48 new design vectors are not executed by that command, not mislabeled as real fixtures, and add zero to production pass counts.

Generator count is currently30, verified by enumerating `tools/gen-*.py`, not assumed from an older phase. Run every generator serially in each of two rounds; hash all tracked/nonignored workspace files before/after each round and require every exit0 plus zero byte drift. Design docs and existing sources must stay unchanged during that snapshot interval. Final documentation commit receives fast checks and another stable double-round check; expensive full Cargo evidence remains tied to293066e.

Completed fresh double-round evidence: round1 and round2 each ran all30 with every exit0,1715 files hashed,0 changed bytes. The final documentation-only commit is followed by fast checks and a further stable double-round confirmation reported in delivery; no full Rust rerun is falsely attributed to it. Documentation links/fences, the closed D01–D48 vector-ID set and all48 Not Executed labels were separately checked, not executed as semantic vectors.

Initial serial attempt disclosure: round1 completed30/30 with1715 files hashed and0 byte drift. Round2 failed in `gen-portable-root-export-revalidation-corpus.py` opening `d9/select.ace1` with `OSError: [Errno 22] Invalid argument`. No Cargo test was active; this cannot be explained solely by overlap with tests. Exact external cause is unconfirmed. The affected target was byte-equal to baseline Git content (SHA-256 `ce5430ef08fded182bb2aee1c631f8e58ed7dbfaaacd05fc90fccebfd29e3588`), and frozen tracked assets had no drift. Directly rerunning that unmodified generator exited0. The interrupted double-round attempt is NOT counted as pass; a complete fresh double round with short quiescence pauses is required, without changing generator/Expected/semantics or terminating unrelated processes.

Frozen comparison must show only `docs/implementation/phase-04c1b4c1-*.md` additions. This includes no change to old512 inputs/Expected, Control22/Recovery16/additional24 D9 sidecars, current26 manifest, existing coverage classifications, findings ledgers, generators, Rust source, ABI/AFIDL, hash domains, registry/rule tables or golden bytes. Registered stats remain43/404/403/1/0, ABI775, AFIDL17, domains14. No Checked constructor or runtime authority is introduced.

Stage disposition: **Outcome A — G01 Formal Contract Design Accepted / DESIGN COMPLETE**, with **Outcome B current producer incompatibility requiring future adaptation**. G01 registration/production remains open; G02/G03 open; F03/F04 open; B01/B02/B03–B07 blocking. No norm registration, D10 Oracle continuation,4D or downstream publication/runtime work is started. No push is authorized/performed.
