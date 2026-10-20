# Execution Safety Review — Checkpoint A scoped evidence, Pilot NOT QUALIFIED

The same coding agent ran a separate read-only comparator; no third-party claim.
PIDs: actual fork EAGAIN and thread-creation failure at pids.current8/max8, with
kernel pids.events increments and no OOM substitution. Task-local bound only.
Disk: shared admission predicate rejects simulated free3MiB/reserve4MiB before
launch; live4MiB maximum producers cross2MiB owned-growth and virtual-free4MiB
thresholds. Real filesystem reserve20GiB and task-growth32MiB remain intact.
Owned payload allocations2,625,536 and3,149,824 bytes were reclaimed after task
reaping; per-file hashes and filesystem before/after metrics remain. Global free
space can change due to other projects, so only owned allocated blocks are exact.
Guardian: orderly handoff cleanup was verified and hash-bound before SIGKILL.
The killed process was an unreaped owned child referenced via pidfd, not a shared
manager. Controller takeover reclaimed every recorded descendant and cgroup.

The old controller's second waitpid on an already reaped guardian could bypass
cleanup. The new version handles ChildProcessError and bounded waits; the legacy
executable stays byte-identical for frozen evidence. Controllers share one lock.
Broader sampled disk roots now cover source workspace (excluding.git), target,
Cargo cache, project tmp and job evidence. Symlink rejection and inode dedup apply.

Hard disk quota remains NOT QUALIFIED: ext4 rw,relatime observation and no quota
utility do not establish current-user usable enforcement or prove kernel lack of
support. No quota activation, privileged queries, fill-disk test, sudo or system
configuration changes occurred. Sampling is not a hard quota and may overshoot.
Residual risk is NOT approved. Production2GiB/3GiB/PIDs128 settings have not been
executed/read back. This is NOT production protector acceptance or Pilot admission.

References: https://docs.kernel.org/admin-guide/cgroup-v2.html and
https://man7.org/linux/man-pages/man2/pidfd_send_signal.2.html.
