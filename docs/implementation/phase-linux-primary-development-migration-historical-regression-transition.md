# Separate historical ledgers

Windows Historical Ledger: Archived, original 0/8. Original profiles, CaseIds,
Compiler, Registry, Expected, Oracle and receipts are preserved. The eight frozen
history trees, frozen G3, and original dc1d current tree remain independent locked
checkouts. Full raw-source rehash checks are in source-preservation-review.json.
The original L1 missing-referencing failure remains unchanged.

Linux Historical Regression Ledger: Not Executed. Corresponding denominators
are retained as 512, 51, 173, 889, 262, 254, 534, 3662. Original Debug2 is retained
for histories1–7; history8 retains Debug0. Frozen G3's 154/26/59 counts and Debug2
are separate. No static replay is credited toward any of these Rust tests.

Linux Development Gate: separate acceptance under engineering policy v1; Partial,
not yet Passed. The 252 static vectors are fully executed with separate group
exit codes and input/expected/actual hashes. No historical Rust denominator is
replaced by 252. Future Linux historical runs retain original source/CaseIds and
Expected; actual platform-exclusive cases must be individually registered, never
silently deleted or written back to the Windows ledger.

Original Windows Lane B 41 generators x2 =82 executions /2403 paths /0 drift
retains its original Windows-only scope, not an expanded-tree or Linux PASS.
