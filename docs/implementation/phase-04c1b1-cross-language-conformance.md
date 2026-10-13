# Phase 4C.1B.1 — Independent Python/Rust Conformance

The standalone standard-library oracle is
[portable_proof_python_oracle.py](/E:/Projects/agent/tools/portable_proof_python_oracle.py);
its frozen corpus generator is
[gen-portable-proof-codec-corpus.py](/E:/Projects/agent/tools/gen-portable-proof-codec-corpus.py).
The oracle implements ACE1 text output, TypeSchema digests and complete Bundle
fixture construction without invoking or parsing Rust output.

The full ten-field Bundle fixture is stored once as canonical ACE1. Each
manifest CaseId carries a deterministic typed mutation script or a bounded
raw-byte edit, plus its independently computed exact input byte length,
SHA-256, expected stage/error and required field set. The Rust test
materializes each script from the frozen base and first compares the input
bytes with the Python hash. That prevents a test helper from quietly changing
an intended negative fixture.

The manifest contains 60 unique cases:

| Group | Count |
| --- | ---: |
| ACE1/closed shape/scalars | 13 |
| Canonical ordering/duplicates | 8 |
| Typed references/graphs | 15 |
| Content/schema/digests | 9 |
| Compiler semantics/profile | 10 |
| Full and positive roundtrip | 5 |

Observed stage distribution is D0=13, D1=8, D2=15, D3=9 and D4=15;
the D4 total includes six accepted current-profile inputs and nine exact
unsupported/conflicting-profile rejections. In particular, the D3 fixture
contains exact int64 `9007199254740993` and decimal
`12345678901234567890.000000000000000001` literals inside the canonical IR
asset. A separate 17-row comparator-mutation matrix lives in
[comparator-mutations.json](../../spec/conformance/portable-proof-codec/comparator-mutations.json);
its mutations are executed as comparison fault injections, distinct from the
60 production-input mutation/positive cases.

Every case executes the public D0, D1, D2, D3 and D4 APIs in sequence. The
runner compares reached stage, stable family, full input identity, complete
Bundle canonical bytes for accepted cases and D4 projection counts. Manifest,
executed and fully compared counts must be equal.
