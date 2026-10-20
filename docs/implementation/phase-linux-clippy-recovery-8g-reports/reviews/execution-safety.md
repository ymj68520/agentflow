# Execution Safety review

Independent coding-agent bounded review of source and raw receipts, not third-party certification.

Initial source review found two inherited blockers: Guardian death after ready
but before release, and a successful controller result despite false cleanup
flags. Before any Cargo launch, the new adapter added final pidfd+identity
checks and required empty/cgroup_removed to be exactly true; otherwise the
controller takes over. Static hardening checks passed 77/77. The frozen core
hash and historical execution artifacts remain unchanged.

The actual run independently read back AS8GiB, tree3GiB, CPU160000/100000 for
32 CPUs, Swap0 and PIDs128. Guardian release identity, policy/freeze/approval
bindings and actual kernel limits agree. No observed memory, Swap, PID, disk,
host reserve or timeout trigger occurred. Cargo101 is distinct from controller1
and GuardianCompleted. Cleanup empty/removed flags are true; post-cleanup PIDs0,
and four owned-unit cleanup commands exited zero.

No P06 process/cgroup, held rendezvous lock or owned unit-file residue was found
by independent readback. Parent-agent live systemctl checks also found no loaded
owned phase02–06 unit. Sampled process trees are not exhaustive fork/exec traces.
The final pidfd check is point-in-time; controller takeover remains the fallback.
B2 disk monitoring has no hard quota/strict overshoot guarantee. Observer costs
are included in the task tree; negligible overhead is not claimed.
