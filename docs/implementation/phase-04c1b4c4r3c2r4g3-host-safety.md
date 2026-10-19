# Host safety and interrupted gates

The user reported two machine freezes. System events confirm abnormal restarts;
those events alone do not establish a complete OS/hardware root cause. Neither
interrupted Cargo run is credited as a passing gate. Original logs are retained.

A concrete unbounded-allocation path was identified in the NEW depth fixture:
it called `TypeSchemaUniverse::insert_type` on a 100-deep List before calling the
Binder. That constructor requests `CanonicalTypeSchema::digest` /
`ace1_bytes`; the frozen `af-idl::canonical::type_expr_shape(depth)` recursively
expands list, map_string and option at every level. Shape construction therefore
has exponential branching, even though the actual selected value is linear.
The Binder's iterative depth guard was never reached. A protected reproduction
hit the 768 MiB process cap and exited abnormally instead of exhausting the host.
This is evidence of the allocation hazard, not proof of every restart's cause.

The depth fixture now compiles an actual AFIDL module through
`compile_module_set` without first requesting its deep schema digest. Its real
Native CompiledModuleSet reaches the Binder, which rejects the unchanged depth
bound at source admission. No fake digest, arbitrary DagRank, manual graph or
private universe mutation is used. The frozen af-idl implementation is not edited.
The old encoder hazard remains an explicitly known, out-of-scope follow-up.

The 70 MiB single-string work fixture is also no longer run. Its replacement
uses 12,000 actual Native confidentiality labels with a 2,048-byte common prefix.
It exercises real typed-order comparison work under the same 4,194,304 limit,
without the giant-scalar projection peak. Case identity, expected failure
domain/reason/stage and all normative limits stay unchanged. Both constructors'
exact source parameters are recorded in boundary-cases.json.

All three boundaries passed together in the protected isolated test:
`production_resource_and_nominal_boundaries`, exit0, 21.81s test time.
Whole-command receipt: 26.01s, peak process committed memory 683,421,696 bytes,
peak job 716,414,976 bytes; at least 15,322,566,656 physical bytes remained free.
This is a targeted development check, not a substituted unfiltered Cargo gate.

Subsequent commands use `tools/run-production-input-graph-safe.py`:

- One active command; RUST_TEST_THREADS=1; no parallel Oracle/Cargo execution.
- Windows Job Object CPU hard cap5%, BelowNormal priority.
- Process committed-memory cap768 MiB; whole job cap1,536 MiB.
- Reserve at least8 GiB physical memory; cancel if physical-memory load reaches80%.
- Worker waits for verified job assignment before starting its command.
- Kill-on-job-close; no orphan workload after parent termination.
- OS limit/cancellation/nonzero exit is NOT a conformance pass.

The safety envelope was checked with a tiny successful child and an allocation
that was refused by a deliberately smaller64 MiB cap. Caps are read back from
Windows before the worker is released. No driver, paging, antivirus or global
Windows setting is changed. Logs/peaks and semantic gate results remain separate.

API references: [CPU hard caps](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_cpu_rate_control_information),
[committed-memory limits](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_extended_limit_information).
