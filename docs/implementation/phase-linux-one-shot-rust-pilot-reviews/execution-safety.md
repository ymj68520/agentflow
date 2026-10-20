# Execution Safety Review — Passed for this B2-approved one-shot Pilot only

Same agent, separate readback process, not independent third-party certification.
Three direct human decisions separately approved numerical resources, B2 disk
risk, and one exact Cargo launch. Immutable qualified PIVOT-03 ownership, cgroup
cleanup, pidfd takeover and measurement helpers were imported by exact hash; no
unnecessary destructive qualification tests were repeated. New admission checks
were statically tested. The single-use phase ledger uses exclusive-create claims.

Before Cargo release, actual online32 CPUs produced cpu.max160000/100000;
memory.max3221225472, memory.swap.max0/current0, pids.max128 and actual kernel
service RLIMIT_AS2147483648 soft+hard were read back. Guardian-ready and service
readiness preceded release. Parent constraints and host capacity were inspected.
Controller/guardian have separate128MiB AS limits outside the workload group.

Cargo exited0 in4.1019s. Kernel tree memory.peak246607872 bytes; CPU3.742497s;
OOM0, swap0, PID-limit events0. Sampled PID maximum8 is NOT an exact kernel peak.
All owned tasks were killed/reaped after completion; service wrapper signal9 is
expected supervisor cleanup, not a Cargo failure. The owned cgroup was removed,
lock released and empty per-job temporary tree removed. Target/cache and logs
are intentionally retained and measured, not deleted or called fully reclaimed.

Disk risk is B2, not a hard quota. Planned sleep100ms produced maximum sample gap
0.339899s. Sampled regular-file block growth14622720 bytes; directories/FS metadata
are outside that file counter, while whole-FS free bytes/inodes remain monitored.
Sampling does not prove an exact maximum overshoot or isolate arbitrary outside
writes. These limitations are disclosed, not hard-disk isolation claims. Known
Cargo/Target/TMPDIR/HOME/XDG/write roots are audited; trusted build scripts could
ignore environment, so no hostile-code filesystem sandbox is asserted.
This disposition expires after this one Pilot; future full gates require new review.
