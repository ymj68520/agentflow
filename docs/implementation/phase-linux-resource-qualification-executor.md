# Linux controlled runner — qualification implementation, production NOT accepted

Entry: `tools/linux-controlled-runner.py MODE --id UUID`. No shell command or
caller-supplied argv is accepted. Eleven tiny fixture modes are authorized;
`pilot`, `fmt`, `clippy`, `test`, `spec-check` have fixed command rosters but are
hard denied by the current approval validator. They are not enabled by accepting
the process-memory metric or by a successful small fixture.

The SSH session is outside the delegated user manager tree. The kernel requires
write permission at the common ancestor for PID migration, so we do not move a
payload from SSH across that boundary. The ordinary user's systemd manager starts
a fixed service within a UUID-named project slice below user@1001.service.
Only the NEW owned slice's settings are changed. Before any workload starts,
CPU quota, memory.max, memory.swap.max and pids.max are read back. At 32 online
CPUs, 5% means cpu.max=160000 100000 (1.6 CPU capacity), NOT 0.05 CPU.

The service is born inside the constrained tree, with inherited hard LimitAS,
LimitFSIZE, LimitCORE=0, Nice=19, OOMPolicy=kill, RuntimeMaxSec and control-group
termination. The fixed wrapper verifies service membership and the actual AS
hard limit before forking the fixture. Descendants inherit membership and limits.
Per-process AS is virtual address space, not RSS. Aggregate memory comes from
cgroup memory.current/memory.peak/events, never parent RSS. The controller and
independent guardian are outside the workload group. In the actual qualification
harness, controller startup inherits hard256MiB; the guardian uses hard128MiB and
the controller drops to hard128MiB after systemd-run returns. Ownership metadata
uses the shorter steady-state description, not a claim of128MiB during startup.
This control plane is disclosed separately, not charged into the 96MiB workload.

An exclusive flock permits one top-level task. The guardian inherits the lock,
observes controller pipe EOF, and survives a controller SIGKILL. Only the bound
UUID unit names and exact cgroup path/inode are used for cleanup. cgroup.kill
kills that task tree, then the manager stops the owned units and reverts their
runtime property files. No shared user.slice/system.slice budgets, global Swap,
CPU configuration, sudo, packages or unrelated processes are touched. An unlocked
rendezvous lock file is retained; an active lock is not.

CPU/limits are checked during the run; online-CPU change or limit drift aborts.
MemAvailable and host disk reserve are monitored. Disk growth currently measures
the qualification job directory with a sampled watchdog plus per-file LimitFSIZE;
this is NOT a kernel aggregate filesystem quota and does not yet cover production
Cargo target/cache/temp output. Sampling can overshoot; actual samples remain in
receipts. Production approval must address that scope and acceptable semantics.

Eleven final-version small cases passed; 60 final static checks passed. Raw
nonzero exit7, real OOM, timeout, explicit cancel and controller SIGKILL are distinct
results, not manufactured workload success. PIDs exhaustion, host-reserve crossing,
disk threshold crossing, guardian death and production-sized limits remain untested.
No Rust pilot or gate has run. Actual production qualification remains Partial.

Early failures are preserved: unsupported systemd MemoryOOMGroup assignment
(before payload), systemd resetting a manually set slice OOM-group flag (LimitDrift
correctly aborted), and an initial fixture wrongly requiring cpu.max at a child
where the CPU controller acts at its parent. OOMPolicy=kill provides the verified
service memory.oom.group=1; optional child interfaces are not invented. The failed
11-case first suite executed1/failed1/not-executed10; the repaired suite is separate.

References: [kernel cgroup v2](https://docs.kernel.org/admin-guide/cgroup-v2.html),
[Linux getrlimit](https://man7.org/linux/man-pages/man2/getrlimit.2.html),
[systemd resource controls](https://www.freedesktop.org/software/systemd/man/latest/systemd.resource-control.html).
Observed systemd255 behavior is recorded, not inferred from latest docs alone.
