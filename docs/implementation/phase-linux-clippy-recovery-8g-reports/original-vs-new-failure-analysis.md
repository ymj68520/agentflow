# Original versus 8 GiB Clippy failure

Original execution HEAD `cb9679ea9c4056301d594c5680094765efda2fb3` exited 101
under 2 GiB process RLIMIT_AS. `rust-lld` reported `Cannot allocate memory` while
opening the `serde_derive` shared-library output. The 298,536,960-byte tree memory
peak was not process virtual size; no OOM event was recorded. The failing
process's VmPeak and allocation syscall were not captured.

New execution HEAD `f3992f87a0348268c823c81456cf07b3f27de628` used a hard/soft
8,589,934,592-byte process address-space limit and unchanged 3 GiB task-tree cap.
The exact full Clippy command ran once and exited 101 after 27.701979 seconds,
now on `unused_variables` (`clauses`, `af-computation/src/control.rs:2842:35`),
promoted by the mandated `-D warnings`. It progressed past the dependency link.
No OOM, Swap, PID-limit, timeout, disk-reserve or host-memory-reserve termination
was observed. Thus the current failure is a source quality-gate failure rather
than the previous linker ENOMEM or an observed protection trigger.

The new serde linker PID 1934253 had a directly sampled `/proc` VmPeak of
2,534,862,848 bytes, above the old 2 GiB cap. This supports address-space pressure
as a plausible contributor, but is not a counterfactual reproduction of the old
failing allocation and does not prove unique causation. Final linker lifetime
peak is Not Observed. Host state, retained build cache and instrumentation also
differ. No cap search, further raise, retry, product fix or warning suppression
was performed. Old failure receipts remain unchanged. Outcome D, not A or B.
