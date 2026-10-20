# L2-PIVOT-05 — Outcome D / Partial

Round1 executed exactly once per item at the SAME frozen source:
HEAD cb9679ea9c4056301d594c5680094765efda2fb3
Tree9caacbf482685a9e004892559fd9d77c81920075
Starting HEAD4b08031f4c8d54074b3c2f90e6361e5ccf9b376a.

fmt: cargo fmt --all -- --check — PASS, original Cargo exit0,2.228423811s.
Clippy: cargo clippy --locked --offline -j1 --all-targets --all-features -- -D warnings
— FAILED, original Cargo exit101,8.193976853s. It failed linking serde_derive's
proc-macro shared library: rust-lld reported Cannot allocate memory. Complete
workspace lint coverage was NOT reached. FORMAT_AND_CLIPPY_GATES_PASSED is NOT
issued; only formatting-gate Linux credit is earned.

Raw group memory peak: fmt44048384 bytes (~42.01MiB); Clippy298536960 bytes
(~284.71MiB). OOM/Swap/PID-limit events0.2GiB per-process AS and3GiB cgroup tree
memory are different metrics. A linker AS-limit interaction is plausible but NOT
proven; no failing syscall/VmPeak trace was captured. No arbitrary new limit is
justified by this report, and no automatic budget change or retry occurred.
This is an observed compilation/link allocation failure, not a RuleId-backed
product contradiction, nor a mandatory formal execution-contract gap; D applies.

Round2: NOT AUTHORIZED / NOT EXECUTED and blocked by Round1 failure. No cargo test,
spec-check, full engineering acceptance, Windows historical/cross-build, Binder,
Raw Full-field/TotalT3/T4–T8/RootExport/Publication/Runtime work or Push. A generic
stage grant and consumed prior Pilot grants do not authorize Round2. The mixed
build/runtime memory and complete test/doctest coverage contract remain pending.

Each command had fresh per-item resource and B2 approval; neither reused Pilot
risk acceptance. Production readbacks, raw Cargo exits, timestamps, CPU/memory/
PIDs/disk/free/inode samples, streaming log hashes, guardian/controller cleanup
and phase-level immutable evidence are preserved.89 static checks and24 receipt
readback checks passed; same coding agent, separate processes, not third party.
No product source, Profile, Expected/Oracle, dependency lock or gate strictness
was changed to gain a pass. Old failed/interrupted/not-executed evidence remains.

Before/after checks retain21,032 frozen paths,1,677 protected inputs and25 package
identities. Formal9/8/8/8 and three Some bindings unchanged; old Windows0/8 remains
Archived, Old R5G2 Registration Incomplete. Pilot OutcomeA stays scoped to its old
af-core check and does not establish full acceptance of this or final source HEAD.

Temporary task directories were reclaimed after verified tree cleanup; Target
and cache, including failed build outputs, and all raw evidence retained. Final
HEAD/clean tree/no processes/cgroups/active locks and backup hashes are in the
separate external final-runtime-results ledger. Stop here. Any investigation,
policy revision or fresh Clippy attempt needs new explicit scope; Round2 cannot
start on this failed Round1 and cannot borrow any of its consumed approvals.
