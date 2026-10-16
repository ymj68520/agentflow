# Independent native trace Oracle — not full seed conformance

The Python model builds each complete thirteen-field contract independently,
including exact alias-schema pins derived from the independent AFIDL parser.
It applies frozen full AtomKey/action identity, native source lookup, same-atom
peer isolation, checked u64 arithmetic, independent port/schema interpretation,
frozen containment and primitive alternate-assignment semantics. No Rust code,
Actual JSON or report reason is used to create Expected.

Existing Python AFIDL/contract canonical encoders are reused for canonical
input/bytes only. New Expected lives in a separate namespace and is generated
by `python tools/gen-g01-t3-native-capture.py`. Existing generators/Expected are
unchanged. The new generator reads no Actual and runs no Rust process.

The Rust test consumes those independently authored raw declarations through
real explicit native contexts and the real checker/collector. Its projector
uses exhaustive typed matches/getters for returned entries, bounds, schema
bodies, structural types and all countermodel members. It never derives causes
from `reason`. Python compares independently recomputed values against Actual.
Windows subprocesses explicitly use UTF-8; the initial encoding transport
failures are preserved in development logs, not treated as semantic findings.

All 53 inputs execute, including three genuine earlier canonical rejections.
51 traces agree. Two legal Bytes inputs disagree at canonical materialization.
The default independent semantic gate therefore **exits 1**, intentionally:

```powershell
$actual = Get-Content -Raw docs/implementation/phase-04c1b4c4r3-native-trace-actual.json
$actual | python -X utf8 tools/oracle/native_guard_capture.py
```

Use a UTF-8 pipe or `subprocess.run(input=actual_bytes)` when the local
PowerShell's legacy output encoding is not UTF-8. The Rust runner sets the
Python environment explicitly and does not depend on the host pipe code page.

`--report` is a diagnostic report command (exit0), never a passing semantic
gate. Its `outcome=B` and `unexpected_mismatch=2` remain required. The Cargo
test named `independent_native_oracle_reports_formal_bytes_contradiction`
checks detection of the contradiction, not acceptance of the wrong rejection.
The default gate still fails; Expected still says canonical encoding succeeds.

`phase-04c1b4c4r3-native-oracle-report.json` preserves both independent Expected
and Actual for the two failures. `native-trace-actual.json` preserves every
actual case. The first differing admission field is `$.canonical_admission`;
the earliest typed root cause is Bytes versus String at the bytes payload.

No complete seed-schema comparator is claimed. In particular, deleting the
same required field from Expected/Actual is not yet a certified rejection law.
No new40 comparator-mutant gate, complete source/observation coverage or full
T3 field-count closure is claimed. Diagnostic equality of51 traces cannot
establish any of those properties. Required seed gaps remain unclosed even if
the preexisting bytes defect were fixed.
