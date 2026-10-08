# Phase 4B.2A Conformance

The new `tools/gen-computation-leaf-corpus.py` is an independent Python
oracle. It implements exact pin lookup, typed port-set matching, resource
mapping and alias recording, source-preserving scoped effects, subflow
interface/export checks, dependency cycle detection, context identity, and
leaf verdict composition. It does not call Rust and does not create a
HashV1/ACE1 persistence format.

Generated test vectors live under `spec/conformance/computation-leaf/` in
the required `golden`, `negative`, `invoke`, `await`, `subflow`, `mapping`,
`context`, and `equivalence` groups. They use a clearly test-only JSON
envelope. Rust end-to-end tests independently exercise exact INVOKE
resolution, no digest fallback, distinct call-site identity, and digest vs
proof-context behavior.

The oracle corpus includes Proven, Violation, Unproven, and RequiresEvidence
expectations. All fixtures are static test vectors, not wire ABI.
