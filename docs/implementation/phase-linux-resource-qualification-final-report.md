# L2-PIVOT-02 — Outcome D / Partial infrastructure

Starting Linux HEAD: 90756b046a03432f6415148fcc9116a274e84830.
Primary workspace: ssh fwq, ~/projects/agent-r5g2-linux-validation/workspace.
Only Linux workspace source changes and classified local commits; no Push.
Final committed HEAD and clean states are stored in the separate external
phase-linux-resource-qualification-final-runtime-results.json to avoid self-hashing.

Approved and implemented: small cgroup qualification only. Human also accepted
per-process hard RLIMIT_AS as an explicit virtual-address-space metric, but did
NOT authorize Rust compilation or full gates. The immutable candidate policy
records Proposed / Approved / Enforced / Observed separately.

Final version: 60/60 static checks, 11/11 small live fixtures, 78/78 separate
readback checks. CPU quota at32 online CPUs was160000/100000; CPU stress actually
throttled41 periods. OOM memory.peak was100663296 bytes (96MiB), with3 oom_kill
recorded. This is contained local OOM, not server-wide memory exhaustion.
All tested owned cgroups were removed. Original pre-start/LimitDrift/fixture
failures remain, including first suite required11/executed1/failed1/not-executed10.
No early failure is erased or promoted by the repaired suite.

Production executor: NOT yet accepted. The measured small set is narrower than
production safety: PIDs exhaustion, live memory/disk thresholds, guardian death
and production-size budgets remain untested. Disk watchdog scope and sampled
rather than kernel-quota semantics need production review and qualification.
The current fixed heavy-command roster is fail-closed, not an enabled backend.

Pilot and fmt/Clippy/workspace tests/spec-check: Not Authorized / Not Executed.
No build profile/source/Expected was changed to fit resources. No Rust tests were
filtered or deleted; their actual denominator is unmeasured, not zero-required.

Preserved: Linux static252/252 and routing26/26, nine-package venv,25 Cargo locked
packages, ten frozen snapshots, Windows historical Archived0/8 and Old R5G2
Registration Incomplete. No credit toward the eight native historical groups.
Analysis/Model/Report/Obligation and Native Full Seed/Raw Full-field/Total T3/T4 /
RootExport states remain unchanged. No Windows target/release work starts.

Required next approvals: separate pilot compilation scope with CPU5%-of-online,
BuildAS2GiB/tree3GiB, Swap0, PIDs128, reserve8GiB,15min, growth12GiB and free20GiB,
serial/jobs1/testthreads1; and, AFTER pilot/protector qualification, a separate
full-gate scope up to90min each with explicit Build/Test budget-to-command mapping
(TestAS768MiB/tree1536MiB). These remain PROPOSALS, not approvals. Scope must also
bind production disk monitoring and qualified cleanup before execution.
This phase stops here; no automatic retries, quota increases, product work or Push.
