# Phase 4C.1B.2 — Resolver Reconstruction

D8 constructs a fresh `SubflowIrUniverse` by inserting only D7-checked target
IR documents. Each insertion recomputes the target digest and direct
CALL_SUBFLOW dependency set. D8 then invokes the existing
`StaticComputationResolutionSnapshotBuilder`: each resolver component is
inserted as a D6 Checked contract, and each explicit target/export pair is
submitted through `subflow_export`, which checks the target exists and the
export is checked under the same D5 context.

The builder's `ResolverSnapshotIdentity` is freshly read from that snapshot.
Its component inventory, target pins, direct edges and export association are
compared by exact tuple and target key, never by a sorted digest multiset.
Direct and transitive dependency Options require Some to claim a complete
closure; Some([]) differs from None. Transitive closure is recomputed from the
inserted IR graph with cycle, missing-target and graph-work checks.

The resolver accepts at most 4,096 target rows and 2,000,000 counted graph
work units. Before per-root traversal state or expanded digest projections
are allocated, it computes a checked conservative upper bound: with `N`
targets and `W = N + direct-edge-count` already counted while materializing
the typed graph, worst-case closure work is bounded by `W * (N + 1)`. Overflow
or a bound above the budget fails as D8 `ResourceLimit`; the traversal also
retains its exact checked counter. Caller-source resolution separately counts
`subflow-call-count * (N + direct-edge-count)` under the same budget.

Source associations are independently derived by calling
`resolve_computation_leaf` for each caller INVOKE, CALL_SUBFLOW and AWAIT node.
Every source-node occurrence remains distinct, even when two nodes share one
ComponentContractDigest. AWAIT retains the formal unavailable interaction,
UnknownExternalInfluence provenance and NotInferred authority state.

The portable target/export row is an explicit resolver assembly input, not an
origin claim. D8 confirms a well-formed checked association in the existing
snapshot model; it does not establish target-body conformance or publication.
