# Linux-first engineering

Primary development host: `ssh fwq` (verify the known host key; no sudo).
Primary workspace: `~/projects/agent-r5g2-linux-validation/workspace`.
Windows `E:/Projects/agent` is the preserved audit/backup source, not a daily
Rust build environment. This file does not move the current Codex app executor.
Work from the Linux checkout; if a client is on Windows, use verified SSH for
source edits and execution. Do not edit the ten frozen worktrees.

Read `engineering/policies/linux-primary-development-v1.json` before validation.
The archived Windows 0/8 ledger is not a prerequisite of new Linux engineering.
It remains unpassed and is not formally Superseded. Product normative gates,
Raw full-field / Total T3 / T4 requirements and RootExport denials are unchanged.

Use the existing Rust/Cargo installation, explicit process-local PATH, project
venv `../venvs/linux-primary-v1`, and CARGO_HOME `../cache/cargo-linux-primary-v1`.
Build artifacts belong in `../build/linux-primary-v1`. Do not change global
PATH/toolchains/caches. Dependency restores require exact versions and checksums;
new unpinned dependencies require human approval. No sudo, VM, Wine or auto Push.

Use the graduation-project continuous development workflow: edit, format,
compile/lint, test, fix ordinary problems and revalidate. An ordinary fmt/lint
failure is not a reason to stop or request another phase. Do not delete tests,
weaken lint options or change frozen Expected/Oracle/Registry to obtain a pass.

Resource approval is task-scoped, not per Cargo invocation. For the current
explicitly approved baseline task, `tools/run-thesis-gate.py` uses the existing
project-owned cgroup safety core and the fixed command whitelist in
`engineering/policies/thesis-fast-dev-v1.json`. Selected limits: per-process
RLIMIT_AS 8 GiB (virtual address space), task-tree memory 6 GiB, CPU 10% of online
host capacity, Swap 0, PIDs 128, host memory reserve 8 GiB, minimum free disk
20 GiB, cumulative project growth threshold 12 GiB, jobs 1 / test threads 2, one
heavy task, 90 minutes per command. These are within the user's task maxima.
B2 sampled disk monitoring is accepted for these trusted commands, not a hard
quota or an untrusted-code sandbox. Keep light prechecks and real exit/log data.
Do not rerun destructive qualification tests or change shared system limits.
Do not infer future-task resource authorization after this task ends.

The complete test corpus materializes large conformance graphs. Cargo's test
profile uses opt-level 1 and limited debug info (level 1) to keep full validation practical, while explicitly
retaining debug assertions and overflow checks. This is build performance tuning,
not a relaxation of test assertions or a change to formal execution Profiles.

Development Acceptance, Cross-target Compatibility and Windows Release
Acceptance are separate statuses. Linux passes cannot grant Windows Verified.
Preserve historical Expected, Registry, compiler, profiles, CaseIds and receipts.
Stop at the end of the requested phase; do not automatically resume feature work.
