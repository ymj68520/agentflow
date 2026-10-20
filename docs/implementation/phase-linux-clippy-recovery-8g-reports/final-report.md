# L2-PIVOT-06 — Outcome D / Partial

The sole authorized full Clippy invocation exited **101** after **27.701979 seconds**.
The 8 GiB address-space recovery progressed past the earlier `serde_derive` link
failure, but the gate failed on an actual source warning promoted by `-D warnings`:

```text
crates/af-computation/src/control.rs:2842:35
ControlOperator::Handle { clauses, .. } if child_index == 0 => {
error: unused variable: `clauses`
```

No product source was changed, no warning was suppressed, and no retry occurred.
This is a quality-gate failure, not an independently established formal semantic
contradiction. Neither `CLIPPY_GATE_PASSED` nor Round 1 acceptance is claimed.

## Execution identity and authority

- Start HEAD: `e816595ce8cde91b0c87b3e27e217599dd27ae07`.
- Executed HEAD: `f3992f87a0348268c823c81456cf07b3f27de628`.
- Executed tree: `c3bce461c1341976c3a827ff8cf6893160ca7aa7`.
- Run: `fb745d2e-750c-4a9e-96ec-68e5d96a1339`.
- Fresh human A/B/C: `call_756396936da54fa79fb004dc085306a8`.
- Policy SHA256: `97c941f691fed2285da3db6a0866b7d79b212a72db31ec3fb94fba842b8e4200`.
- Exact command: `cargo clippy --locked --offline -j 1 --all-targets --all-features -- -D warnings`.
- Controller exit: **1**. Guardian `Completed` describes supervision, not gate success.
- Complete argv, cwd, environment, code hashes and full log hashes are in
  `source-freeze.json`, `clippy-execution.json` and the immutable raw run receipts.
- stderr SHA256: `d39d12590d3a91645506b5539dceb3d298b3ea429e7e1b392ecbe07d66f05a09`.

Later documentation commits are not represented as the HEAD that ran Clippy.
The final delivered HEAD is recorded separately in `final-runtime-results.json`.

## Actual protections and observations

| Measure | Effective / observed |
| --- | --- |
| Process hard/soft RLIMIT_AS | 8,589,934,592 bytes; virtual address space, not RSS |
| Tree memory.max | 3,221,225,472 bytes; unchanged |
| CPU | 32 online CPUs, cpu.max `160000 100000` (5% host capacity) |
| Swap / PIDs | 0-byte swap cap; pids.max 128 |
| Kernel tree memory.peak | 582,414,336 bytes, about 555.43 MiB |
| Tree CPU usage at final snapshot | 28,497,445 microseconds |
| PID peak | 13 sampled tasks, not a proven lifetime maximum |
| Sampled project growth | 128,983,040 allocated regular-file bytes |
| Minimum sampled filesystem free | 81,373,294,592 bytes |
| Minimum sampled available memory | 19,121,274,880 bytes |
| OOM / Swap use / PID limit events | None observed |
| Timeout / cancellation | None |

The Guardian/controller remain separate 128 MiB-address-space control processes;
workload tree statistics do not include them. The optional observer is a thread
inside the approved workload tree, sharing its CPU, memory and PID limits.

Accepted B2 monitoring is **not a hard disk quota**. It offers no strict maximum
overshoot-byte guarantee, omits some directory/filesystem metadata from regular
file growth, and does not establish isolation of non-project writes. The maximum
Guardian sampling gap was approximately 0.372 seconds. Host free bytes and inodes
were also observed. No broader capacity bound follows from this run.

## Linker observations and failure comparison

The nonprivileged observer completed 1,348 scans and wrote 37 rows. It observed 12
linker/driver process identities, with two racing process reads. For the
`serde_derive` `rust-lld` PID **1934253**, five observations included:

- maximum sampled VmPeak: **2,534,862,848 bytes**;
- maximum sampled VmSize: **2,481,577,984 bytes**;
- maximum sampled VmRSS: **77,697,024 bytes**;
- effective RLIMIT_AS: **8 GiB**; RLIMIT_DATA: **unlimited**.

VmPeak was read from `/proc`, not inferred from cgroup memory. Its final lifetime
maximum is **Not Observed**. These samples exceed the old 2 GiB cap and support
address-space pressure as a plausible explanation for the old failure, but do
not prove the unique cause of that earlier process's ENOMEM. This run retained a
warm cache and had different host state and observation instrumentation.

Old execution `cb9679ea9c4056301d594c5680094765efda2fb3` remains Exit 101 with its
original `serde_derive` linker ENOMEM and complete receipts. All 94 prior evidence
files matched their manifest hashes. No old PASS was substituted for a failure.

## Integrity, credit and remaining work

All ten frozen snapshots / 21,032 paths, 1,677 protected inputs, Cargo.lock and
25 dependency archives plus vendor checksums remained unchanged. Existing
Rust/Cargo 1.91.1 binaries were verified; no installs, toolchain/Profile changes,
Registry/Expected/Oracle changes, or frozen source edits occurred.

Static preparation initially failed one approval check because a mechanical
replacement changed the candidate disk reserve; the validator denied admission.
That failure and source snapshot remain. Corrected static tests passed 59/59;
after independent safety hardening they passed 77/77. Synthetic observer tests
passed 8/8. A failed low-address-space metadata-only version query is also retained;
it was not a Clippy launch. These counts are not native product-test counts.

Independent source review identified inherited final-Guardian-liveness and
cleanup-success gaps. Both were closed in the new adapter before launch, using
fresh pidfd checks and exact cleanup-success requirements. The old qualified
core and its historical receipts were not rewritten. Subsequent independent
readbacks found no observed execution-safety or evidence-integrity failure.
These were coding-agent reviews, not third-party certification.

Historical fmt exit 0 remains credited at its own HEAD. There are 256 equal
tracked formatting inputs and unchanged formatter binaries, but the old freeze
did not capture historical parent-directory configuration absence. Complete
current fmt equivalence is therefore not asserted. Clippy failure independently
prevents Round 1 closure. No tests/doctests ran; no full target-coverage or Linux
Engineering Acceptance claim is made.

Formal Analysis@9 / Model@8 / Report@8 / Obligation@8 and the three Some bindings
remain unchanged. Old Windows history stays 0/8; R5G2 stays Registration Incomplete.
Round 2, Windows gates, Binder/Raw/T3/T4–T8, G02/G03/D10, Positive RootExport,
Publication and Runtime were not started.

The owned task tree became empty and its cgroup was removed. Only twelve empty
directories in the two new owned temporary roots were removed; no data files or
historical evidence were deleted. Cargo Target/cache artifacts and logs remain.
Final live checks and the delivered commit identity are in the runtime ledger.
Only Linux-local classified commits were made. No Push. Stop here.
