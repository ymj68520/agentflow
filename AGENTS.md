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

Resource approval is currently pending. No fmt/Clippy/test/spec-check build gate
may start before a numeric Linux resource policy is approved AND a dedicated
cgroup executor is actually qualified. A routing plan is not an executor or an
authorization receipt. Never change shared user-service cgroup limits.

Development Acceptance, Cross-target Compatibility and Windows Release
Acceptance are separate statuses. Linux passes cannot grant Windows Verified.
Preserve historical Expected, Registry, compiler, profiles, CaseIds and receipts.
Stop at the end of the requested phase; do not automatically resume feature work.
