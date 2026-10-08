# Phase 4B.2B.1 Conformance

The independent oracle is `tools/gen-computation-control-corpus.py`. It
implements guard-sensitive Effect accumulation/join, Sequence/Select/
Parallel/Race lower/upper rules, checked and maximum Usage algebra, Failure
union, Flow may-influence and sticky unknown, Resource aliases, per-child
Recovery/X, occurrence identity and verdict composition. It never calls
Rust and generates only a test-vector JSON envelope.

The frozen corpus is `spec/conformance/computation-control/` (revision 1,
58 cases) with the required golden, negative, sequence, select, parallel,
race, proof and equivalence groups. Rust integration tests exercise all four
operator Usage rules, Effect lower-bound normalization, exact parallel
output mapping, sequence cross-sibling dataflow, child RequiresEvidence
propagation, unsupported-node fail-closed behavior, R aliases, per-child X,
and loser Recovery source retention.

The oracle's expected outputs are computed in Python before serialization;
no Rust result is copied into the expected values. These files are analysis
test vectors, not AFIDL, ACE1 or HashV1 objects.
