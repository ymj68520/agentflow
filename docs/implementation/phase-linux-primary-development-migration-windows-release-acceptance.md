# Windows Release Acceptance v1 — Not Executed

This is a separate release-stage contract, not a daily Linux prerequisite.
A release owner must preserve executable hash, source HEAD, Cargo.lock, target,
compiler/linker/ABI, runtime dependencies, machine identity, case roster, Expected,
Actual, exit codes and resource receipts. Linux results cannot fill these fields.

Required real Windows x64 evidence:

1. Successful Windows x64 executable build, with GNU/MSVC ABI explicitly named.
2. Successful launch on an actual Windows x64 machine (no VM/Wine in this workflow).
3. Successful principal Agent workflows with their complete fixed case roster.
4. Correct paths, Unicode, newline and temporary-directory behavior.
5. Contract-conforming persisted data and canonical bytes against independent Oracle.
6. Correct Windows process and permission API behavior, including negative cases.
7. Reproducible distribution with necessary runtime dependencies and deployment checks.
8. No new unauthorized RootExport capability or weakened semantic gate.

All eight criteria: Not Executed. Overall Windows Compatibility Pending; not
Windows Verified. Cross-build Complete may only be claimed after an actual build
receipt and does not establish any of criteria2–8 by itself.
