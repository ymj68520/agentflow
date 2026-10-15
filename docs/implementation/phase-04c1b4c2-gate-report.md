# Fresh gate report — Phase 4C.1B.4C.2

**Outcome A — Registration and Migration Plan Accepted**, now supported by two consecutive complete generator rounds in **E:/Projects/agent itself**, as explicitly required by the user. The previous isolated-evidence acceptance was withdrawn;isolated runs do not count toward this gate. No normative registration or production activation occurred.

## Exact baseline, commits and scope

Full Rust gates launched/completed at HEAD `8b4e3c64ee1d84fc38b6faa9b0b1257deb18873d`;initial tree clean. Only17 new phase-04c1b4c2 planning documents are delivered. Production,registered sources/rules/registries,ABI,AFIDL,HashV1,old Expected,generators and previous-phase documents are unchanged.

Classified documentation commits:
- `87854ba5bd97444698a6a8d25ade630fd69cbd61`: normative allocation,schema,ACE1,capability and edition proposal.
- `c52635ee57a3ebfdb60c1e19cf86c68a57769558`: producer adaptation,migration,isolation,dependencies,vectors,decisions/readiness.
- Final clarification/evidence commit: exact full HEAD is resolved after committing and reported in the delivery receipt,not self-embedded into its own commit.

Generators ran after the first two commits,with all17 documentation files and the three final role/rejection/routing clarifications present. Gate/final-report text was completed afterward. Final documentation fast checks are **not** a second full Rust run. Environment: Windows x86_64-pc-windows-msvc;rustc1.92.0(commit ded5c06cf21d2b93bffd5d884aa6e96934ee4234);cargo1.92.0;Python3.14.2.

## Fresh required commands

| Command | Exit | Evidence |
| --- | --- | --- |
| cargo fmt --all -- --check |0| actual fresh check |
| cargo clippy -j 2 --all-targets --all-features -- -D warnings |0| target/phase-04c1b4c2-clippy.log |
| cargo test --all -j 2 |0| target/phase-04c1b4c2-cargo-test-all.log |
| cargo run -p af-spec --bin spec-check -- check |0| target/phase-04c1b4c2-spec-check.log |

The run includes all512 historical cases (58/72/55/44/35/47/60/63/78),not512 new tests. All13 Portable RootExport integration tests including six fresh producer/consumer cases passed. Static reconstruction's two tests passed,including the independent63-case manifest. Existing D10 gap assertions passed **as gaps**,not full D10 conformance.

| Fresh conformance report under target/conformance | Cases | Required instances | Diagnostic instances | Closure |
| --- | --- | --- | --- | --- |
| d9-full/report.json |22|120117|2683|22 full,0 mismatch/skip |
| d9-recovery-full/report.json |16|164924|2820|16 full,0 mismatch/skip |
| d10-gap-audit/report.json |26 reach D10|118562 independent D9|2112|24 D9 full,2 G03 oracle gaps;**0 full D10**,26 formal gaps |

All48 old design laws and16 migration laws remain **Actual Status: Not Executed**. Markdown/census checking is not execution of those laws.

## Frozen surface and audit

spec-check:43 specifications,404 rules,403 covered,1 intentional exception,0 missing;775 active ABI,0 reserved/retired;17 AFIDL modules;0 cross-reference errors. HashV1 domain count14 unchanged.

Complete-registry collision check:0 proposed SpecificationId/RuleId collisions;all3 candidate source paths absent. RM uses `spec/tables/proof-report-model-v1.md`,not an invalid bare-JSON anchor source. No allocations/reservations happened.

Exact65 Control/27 Leaf source-line census matches the adapter map;6 direct,3 inherited,11 prerequisite arms/20 occurrences and reference/role/substitution carriers mapped.32 allocation rows0197–0228 unique;17 docs' links/fences validate;48+16 vector rows Not Executed;512 old CaseIds unique. No finding ledger or old Expected changed.

## Generator evidence — original I/O failures and equivalent reproduction

Actual inventory:30 tools/gen-*.py. No generator was modified,no process terminated,no permission changed,no fixture replaced as a workaround.

| Original-tree attempt | Observed outcome / retained log under target |
| --- | --- |
| A | Round1 all30,1732 files,0 drift,182.70s;Round2 stopped at codec script18 on full-ten-field-bundle.ace1 with OSError22;phase-04c1b4c2-generators.log |
| B | Round1 all30,1732 files,0 drift,208.16s;Round2 stopped at root script19 on d9/scope.ace1 despite30s pauses;phase-04c1b4c2-generators-retry.log |
| C | Round1 stopped at root script19 on d9/recovery/outer-iterate-4.ace1 with45s pauses/inter-round plan;phase-04c1b4c2-generators-final-attempt.log |

Each failed target still exactly equals its committed8b4e3c64 blob:
- full-ten-field-bundle.ace1:2245830 bytes;SHA25607937cbf704355d670559bae5fb0d07b6acd360400acc230ff1d27dd358b1459.
- d9/scope.ace1:2246414 bytes;SHA256f3d4c28b9916021669f3f768ae331e0d1a9887a7a80f77cd75533d47eca08bed.
- d9/recovery/outer-iterate-4.ace1:2253385 bytes;SHA256a8d3d22fc5a21b704da99ab539495cdc46f783e4faf17636f71da4a350591bb4.

Root cause undetermined. Repository-command-line-filtered process inspection found no attributable competing process;this does not rule out a system reader/mapping. Failed attempts are not combined into consecutive successes.

### Successful isolated verification

Every tracked/nonignored file from `git ls-files --cached --others --exclude-standard -z` was byte-copied (not hardlinked) into ignored `target/g01-regeneration-0y4aqu39`. Initial mirror hashes equal all1732 original file hashes. All30 unchanged scripts run with the mirror repository root.

| Complete consecutive round | Scripts | Files | Changed/added/removed | Time |
| --- | --- | --- | --- | --- |
| Isolated1 |30|1732|0|145.05s |
| Isolated2 |30|1732|0|145.59s |

Final assertion: **mirror Round1 = mirror Round2 = original baseline = original after verification**,over all1732 path/byte hashes. This proves deterministic reproduction and no source/registry/ABI/hash/Expected/production drift for current scripts/exact inputs. It does **not** claim original-directory I/O was fixed.

Evidence: `target/phase-04c1b4c2-generators-isolated.log`,helper `target/phase-04c1b4c2-generators-isolated.py`,retained ignored mirror. These are validation artifacts,not production changes or new conformance cases.

The user explicitly requires original-directory consecutive double-round success. Isolated results are supplementary diagnostics only and do NOT satisfy that gate. Original I/O remains unresolved at reopening;no isolated result is substituted for an original-directory pass.

### Required original-directory verification — PASS

After the explicit user clarification,a fresh fail-fast double-round run executed in `E:/Projects/agent`,not a mirror/worktree/copy. All30 actual `tools/gen-*.py` scripts were unmodified and executed with the original Python3.14.2 at `D:/Software/ProgramEvm/Python3-14-2/python.exe`. No individual-script retries or earlier successful rounds were spliced into this run.

| Consecutive original-directory round | Scripts successfully completed | Nonignored files | Changed/added/removed | Time |
| --- | --- | --- | --- | --- |
| Required1 |30/30|1732|0|243.89s |
| Required2 |30/30|1732|0|243.92s |

Process exit0;assertion **original baseline = original Round1 = original Round2** over all1732 file path/SHA256 pairs. Inventory uses `git ls-files --cached --others --exclude-standard -z` and includes all tracked/nonignored documentation/source/corpus files. Full successful log:`target/phase-04c1b4c2-generators-original-required.log`;orchestration helper:`target/phase-04c1b4c2-generators.py`(ignored validation artifact,not an edited generator).

Verification started with HEAD`50d082612933a469311465aece357c9ef0e88559` and only three phase-local documentation edits recording the reopened gate;those files were included unchanged in both round snapshots. Final receipt updates happen after the run. The helper waits45s before each large Portable generator and between rounds,and2s between scripts;these are idle intervals,not skipped/replaced generator calls.

Read-only diagnostics before this run:Windows Restart Manager reported no affected processes for the three earlier failed resources;non-truncating ReadWrite opens succeeded. The existing bundled Python3.12.14 was discovered but **not used for the successful required run**. No process was killed,no permissions/configuration changed,no fixture replaced,no generator monkey-patched. Earlier OSError22 failures remain preserved;their root cause is still undetermined,and success is not claimed as an environmental repair. The actual user-required in-place two-round gate is now satisfied independently of the isolated evidence.

## Delivery controls

Final checks:docs-only baseline diff,unchanged frozen paths,candidate collision/source/vector census,local links/fences,diff --check and fmt/clippy/spec-check fast pass. Delivery receipt gives final exact HEAD and actual clean-tree status. No push. No G01/G02/G03 code or D10 Expected/oracle changes. Registration READY FOR SEPARATE APPROVAL only;production adaptation/oracle NOT IMPLEMENTED;T1–T8 unexecuted.
