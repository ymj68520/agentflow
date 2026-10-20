# Three separate evidence reviews

Review identity: the same coding agent using a separate read-only comparator
`tools/verify-linux-resource-receipts.py`; not third-party certification or human
approval. It does not import the runner or launch workloads. All 78 readback checks
passed, within the explicitly limited recorded scope.

## Specification Review

Passed within unchanged-source scope. All registered normative source pins still
match. No Rust product source, Registry, Expected, Oracle, Cargo.lock, Analysis@9 /
Model@8 / Report@8 / Obligation@8, capability or semantic rule is changed.
Frozen source/Expected/Oracle byte checks are separately recorded before/after
commits. No RootExport authorization, Raw Full-field completion or T4 eligibility.

## Execution Safety Review

Partial. Final small suite: 11/11. Real CPU throttling, 256MiB per-process AS
refusal/non-raisability, 96MiB cgroup OOM with machine events, zero Swap accounting,
PID/descendant membership, timeout, explicit cancel, orphan reclamation, repeat
cleanup and controller-SIGKILL guardian cleanup have receipts. Every tested owned
cgroup was empty and removed; a same-user unrelated sentinel remained alive.
This does not attest that all other users were independently observed.

PIDs exhaustion, live host-reserve/disk-threshold crossing, guardian death and
production-sized budget enforcement are NOT verified. Disk protection is sampled
and currently scoped to qualification outputs, not a production aggregate quota.
Thus production executor qualification is not Passed and no Rust admission follows.

## Quality & Evidence Review

Passed within small-fixture scope: exact fixed roster, all child identity files,
raw exits/CPU/memory.events, receipt hashes, actual policy readbacks, cleanup and
approval scope checked. Three earlier failed attempts and the first suite's
1 failed /10 not-executed denominator are preserved; only the separate repaired
11-case suite passes. No Expected or denominator was rewritten to make it pass.
Pilot and four Rust gates remain Not Authorized / Not Executed with null unknown
exit/count/peak fields. Linux engineering results do not promote Windows0/8.
