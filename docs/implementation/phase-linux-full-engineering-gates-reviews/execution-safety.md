# Execution Safety Review — Executed resource contract respected

Same agent, separate process; no third-party or universal executor certification.
Each command had independent resource, B2 and exact-command approval. No previous
Pilot B2 grant was reused. CPU quota used fresh32 online CPUs:160000/100000;
per-process hard AS2GiB (virtual address space, not RSS), tree3GiB,Swap0,PIDs128,
host reserve8GiB,each5400s,growth12GiB/free20GiB,jobs1/testthreads1/serial1.
Actual cgroup and service/kernel RLIMIT readbacks preceded both releases.

Both guardians ended Completed with owned trees empty/removed. No cgroup OOM,
Swap usage, PID-limit event, cancellation or timeout occurred. fmt kernel tree
peak44048384 bytes; Clippy peak298536960 bytes (~284.71MiB), far below the3GiB tree
cap. This does NOT rule out virtual-address-space allocation failure in a linker.
RLIMIT_AS affects mmap/brk-type allocation; however the failing linker syscall
and VmPeak were not traced, so causation by2GiB AS remains an inference, not proof.
[Linux getrlimit reference](https://man7.org/linux/man-pages/man2/getrlimit.2.html).

Monitoring is sampled, not a hard aggregate quota; file-block growth excludes
all directory/FS metadata and does not prove arbitrary outside-write isolation.
Whole-FS free bytes/inodes are retained. B2 was accepted separately for fmt and
Clippy only. Streaming sample/process logs bound monitor memory for5400s tasks;
89 static checks include real JSONL encoding and prevent the old10000-sample abort.
Only this task's empty temporary directories were removed; cache/Target, raw logs
and failed build artifacts retained. No quota increase or unconstrained fallback.
Round2 never admitted or launched. Existing qualified cleanup core is unchanged.
