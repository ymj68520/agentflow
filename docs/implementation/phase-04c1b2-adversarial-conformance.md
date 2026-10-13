# Phase 4C.1B.2 — Adversarial Conformance

`tools/gen-portable-static-reconstruction-corpus.py` is the independent
Python expected-data generator. It reuses the repository's Python ACE1/HashV1
oracle, frozen Semantic IR negative fixtures, and declarative portable
models. It does not invoke Rust or read a Rust result as expected data.
The Rust runner applies patches to the registered complete ten-field Bundle,
then starts at `PortableProofCodec::validate_supported` for each production
case.

The checked-in manifest contains 63 unique CaseIds: D5 16, D6 15, D7 16 and
D8 16. Each case records its input SHA-256/byte length, expected first
production rejection or D8 acceptance, independent complete typed D5/D5-D8
expected projection when available, projection hash, required compared fields
and formal fixture provenance. The Rust runner compares the full JSON value;
on mismatch it reports the first field path rather than relying on a digest-only
assertion.
The D7 negative IR documents are reached only after D0-D6. Two duplicate-set
cases are correctly stopped by the earlier registered D1 duplicate-member
guard and are reported as D1, not mislabeled as D8 execution.

Sixteen separate comparator mutations cover schema/context membership,
resource-private scope, recovery catalog separation, exact execution version,
relation bytes, contract source pin, guard verdict promotion, IR producer
facts, resource alias vocabulary, resolver target/export binding, source-node
identity, None versus Some(empty), dependency edges, AWAIT unknown provenance,
and resource-limit promotion. The runner keeps source-level cases and
comparator mutations as separate counts.

The D8 closure-limit fixture is a 159-target complete DAG. Its typed graph
materialization yields `N = 159`, `E = 12,561`, and the conservative upper
bound `(N + E) * (N + 1) = 2,035,200`, so it must fail closed at D8 before
per-root closure expansion. The input remains under the inherited 32 MiB and
1,000,000-node D0 limits and still passes D0-D7 first.
