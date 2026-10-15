# Executed legacy isolation and current test routing

Exact baseline HEAD `abc91d533b4d62e91226722e1d1fcaa30bd00883`;Git tree `42b548ec66a22813f07b77abb5385707d43256cc`.

A fresh `git archive` from that commit was extracted to `target/phase-04c1b4c3-legacy-abc91d5/source`. Every one of1732 archived tracked files was verified by its actual Git blob hash against that tree. Separate build directory `target/phase-04c1b4c3-legacy-abc91d5/build` is selected with process-local CARGO_TARGET_DIR. No new registry/source was substituted.

Actual environment:Windows x86_64-pc-windows-msvc;rustc1.92.0 commit ded5c06cf21d2b93bffd5d884aa6e96934ee4234;cargo1.92.0;Python3.14.2. The fresh legacy job executed fmt --check,clippy(all targets/features,-D warnings),cargo test --all -j2,and spec-check;all exited0 this phase. Full test log:`target/phase-04c1b4c3-legacy-cargo-test-all.log`. No old log is used as a fresh run claim.

| Frozen historical corpus | Cases | Current treatment |
| --- | --- | --- |
| Control /advanced /Scope /RootRecovery |58/72/55/44|229 algebra cases execute unchanged in current and old builds |
| RootExport /soundness |35/47|82 full old report-sensitive cases execute in pinned legacy build;current tests separately enforce no Checked |
| Portable codec /static /RootExport |60/63/78|201 exact old-profile cases execute in legacy build;current admission explicitly rejects old positive bundles |

All512 old CaseIds are unique and preserved. The old job really executes512;current229 neutral/current staging51 and supplemental direct observations are reported separately,not pooled into “512/512 Edition2 compatibility”.

Explicit Cargo test manifests route current tests:229 algebra +leaf/other intrinsic suites remain;old two RootExport and old Portable identity-sensitive runners/full-field module remain unchanged in source and execute in the archived code,not falsely under@2. Current Edition2 runners are additive and actually call production admission/checker APIs. This is not disabling production validators or replacing old Expected.

Old43/404 full registry records and all43 old source bindings compare equal. All1144 pre-existing conformance files compare raw-byte equal with the retained byte baseline artifact;old ABI/AFIDL/domains also match the exact legacy archive. P30 split-row negative/same-row positive and P31 stale/fresh snapshot cases passed in the actual fresh old job.

Historical generators run in the **original directory**,reading hash-pinned old registry/declaration inputs and recalculating old Expected. Their output guard verifies exact physical bytes and refuses overwrite/repair;it does not copy archived Expected into a new edition. The new namespace is generated independently from current source fields. This does not count isolated generation toward the original-directory generator gate.

The pinned archive is evidence/CI input,not a multi-version production dispatcher. T8 must not release this staging artifact as a complete positive checker.
