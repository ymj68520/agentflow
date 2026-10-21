# L2-PIVOT-07 — Outcome D / Partial

The approved unused `clauses` binding was removed and committed, with exactly
one product-line change. The sole authorized fmt check on that fix HEAD exited
**1** after **2.592788 seconds**. Clippy was **not started** because the contract
requires stopping on fmt failure. No further edit, automatic formatting, retry,
resource increase, Push or Round2 execution occurred.

## Source identity and minimal change

- Starting HEAD: `1e44ba4a21cfed3f1522063dfeff6dccbfe18412`.
- Starting tree: `3fa4a3b20e82c7fb8a16b70cab4de542b9f53f79`.
- New engineering adapter commit: `27e8130724562526f732dac754988b38d5c5d166`.
- Tree immediately before product fix: `9d8a3dcb1f9702ff8a964e2c110a0f3d06a1c533`.
- Minimal fix commit / actual fmt execution HEAD: `1a5a7c22bf5956d5c96df829e77738b2ea935c8a`.
- Fixed / executed tree: `90f994e4c2552b1fb26884fe325cfdb4ab2bd822`.

`minimal-source-fix.diff` contains only:

```diff
-        ControlOperator::Handle { clauses, .. } if child_index == 0 => {
+        ControlOperator::Handle { .. } if child_index == 0 => {
```

This matches a borrowed operator. The removed local binding was unused in this
guarded arm, which still returns HandleProtected. Variant matching, the
`child_index == 0` guard, result, subsequent clauses-binding arm, ordinals,
reachability, error propagation, ABI and proof rules are unchanged. No lint
allowance, profile or quality threshold was changed. This is a source-level
semantic impact assessment, not executed native semantic-test proof.

## Actual formatting failure and stop

Exact command: `cargo fmt --all -- --check`.
Run: `ca567e1b-a6dc-4b7b-8889-51fa5145a171`.
Cargo exit **1**, controller exit **1**; Guardian `Completed` means supervision
completed, not gate success.

The full original stdout records rustfmt's requested change:

```diff
-        ControlOperator::Handle { .. } if child_index == 0 => {
-            ControlPathKind::HandleProtected
-        }
+        ControlOperator::Handle { .. } if child_index == 0 => ControlPathKind::HandleProtected,
```

The binding removal makes the arm short enough for the formatter's one-line
layout. The authorized check does not rewrite files. The requested layout was
**not applied**, and fmt was **not retried**. The exact failed source remains
committed. Classification: formatting mismatch, not OOM, environment failure,
linker error, a new Clippy result or a formal semantic contradiction.

Stdout SHA256: `536f345ad21ff32cd316ccff5a8d8ac8973c9732866c5a10c1c8bcbeb903050c`.
Stderr is empty, SHA256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

The proposed Clippy freeze uses the same fix HEAD, but it has **zero launches**,
no reservation or consumed claim, no actual exit code, no resource measurements
and no lint coverage in this phase. Admission readback explicitly denies it after
fmt1. Its unused approval is not silently carried into a future phase. Both
original Clippy101 failures remain unchanged; old fmt credit was not substituted.

## Fresh authority and actual resources

The direct human message explicitly approved this minimal fix and independently
approved each exact fmt/Clippy command, numeric policy and B2 risk for this phase.
The verbatim message and SHA256 are retained in the approval evidence. No prior
one-use approval was reused. Cargo jobs and Rust test threads are both1.

Before releasing fmt, the qualified project-owned execution path independently
read back process hard/soft RLIMIT_AS **8,589,934,592 bytes**, task-tree
memory.max **3,221,225,472**, Swap0, PIDs128 and CPU quota **160000/100000** for
32 online CPUs (5% of host logical capacity). Guardian identity/liveness, current
source/dependency freeze and host reserves passed actual admission checks.

Observed fmt tree peak: **43,384,832 bytes** (about41.38MiB); CPU usage at final
snapshot: **2,644,697 microseconds**. No OOM, Swap use or PID-limit event occurred.
Sampled PID peak4; project regular-file allocated growth73,728bytes; minimum
sampled available memory19,749,085,184bytes; minimum sampled free filesystem
space81,201,692,672bytes. Free inodes were recorded. No timeout or cancellation.
Maximum Guardian sampling gap was approximately0.393seconds.

RLIMIT_AS measures virtual address space, not RSS. The unchanged3GiB tree cap
does not stand in for the8GiB process cap. Guardian/controller are separate
128MiB-address-space control processes outside workload statistics. The optional
observer thread is inside the bounded workload and its costs are included there.

Each gate has its own fresh B2 disposition. Sampling is **not a hard quota**,
has no strict maximum overshoot-byte bound, ordinary-file growth omits some
directory/filesystem metadata, and no non-project-write isolation is proven.
The current observations are not full Clippy/test capacity upper bounds.

## Integrity, evidence and remaining boundary

Before/after complete byte audits verified all ten frozen snapshots /21,032
paths unchanged, Cargo.lock unchanged, and25 Cargo archives plus vendor checksums
unchanged. Of1,677 protected current inputs, exactly the approved source file
changed; its entire content equals the original with the single replacement.
The other1,676 inputs, Expected, Oracle and Registry remained unchanged.
All80 preceding PIVOT06 manifest files and94 PIVOT05 manifest files were verified;
both original linker ENOMEM and unused-clauses failure histories remain intact.

Static admission checks passed116/116; separate-process prelaunch review17/17;
runtime evidence/stop-contract readback32/32. These establish engineering evidence
checks, not product tests. Same-agent process review is not third-party certification.
No native tests or doctests ran, and no full Clippy coverage or Round1 acceptance
is claimed. Further formatting repair and gate execution need a new instruction
and explicit new authority; neither is performed as part of this failed attempt.

Analysis@9 / Model@8 / Report@8 / Obligation@8 and three Some bindings are unchanged.
Windows history remains Archived0/8; oldR5G2 Registration Incomplete; RawFull-field
and TotalT3 Incomplete; T4 NotEligible; PositiveRootExport NotAuthorized. No Windows
build, Binder, Publication, Runtime, Round2 or later product phase was started.

Cleanup made the fmt task tree empty and removed its dedicated cgroup. The two
new owned temporary roots contained only empty directories: twelve directories
removed, zero data files removed. Logs, old reports, Target/cache remain. Final
stop checks, final delivery HEAD and backup identities are recorded separately
in `final-runtime-results.json`; later documentation commits are not represented
as the HEAD that ran fmt. Linux-local classified commits only. No Push. Stop here.
