# L2-PIVOT-01 — Partial / Outcome D

Linux-first engineering migration and isolated dependency/static baseline are
established. Full Linux Development Acceptance is NOT ready: explicit numeric
resource approval and live dedicated cgroup qualification are absent.

Source base: `bf5926a25f5fc1a78fe35a0a775b712a62d27ca9`. New source changes exist only in the Linux workspace,
branch `codex/linux-primary-migration`. Original Windows worktree remains at that
base. Full committed HEAD, clean-status and backup identities will be recorded
post-commit in external `final-runtime-results.json`, avoiding a self-hash cycle.

Complete: verified incremental Git transfer; all ten frozen snapshots retained;
engineering policy v1 plus AGENTS routing; exact approved nine-package Python venv;
25-package Cargo checksum directory source; offline resolution of both lockfiles;
actual static R3=110/P0=105/P1A=37, total252/252, failed0, not-executed0; 26/26
planning-model tests. Static execution HEAD was the base with the new harness
uncommitted; its exact bytes are bound by static-execution-context.json. These are
static design/manifest/authorization cases, not Rust product test coverage.

Not Authorized / Not Executed: fmt, Clippy, Rust tests, spec-check, cgroup live
qualification, Windows cross-build and real Windows release acceptance. Zero heavy
builds, zero limit increases, zero Push, no RootExport authority. Test-count and
peak-resource fields for unexecuted Rust runs are null, not invented values.

Old Windows0/8 is Archived, not Passed and not formally Superseded. Old R5G2
registration remains incomplete. New Linux engineering excludes that legacy route
without changing product Spec/Registry/Expected/Compiler/Edition. Raw Full-field /
Total T3 remain incomplete and T4 is not eligible. No feature work starts here.

Remaining work: independently approve exact numeric build/test/host CPU-memory-
swap-disk-time policy; implement/qualify the owned-only dedicated cgroup executor
with real OOM/over-limit/cancel/cleanup receipts; then run the four full native
Rust gates at a recorded source HEAD. Windows target tools and real release
validation need separately scoped preparation later. Do not return to the Windows
historical build loop as an unconditional prerequisite.
