# Phase 4C.1B.2 — Checked Object Lifetime

`CheckedComponentContract` borrows the exact `StaticContractProofContext`, and
the resolver snapshot borrows those checked contracts. A Rust owner cannot
safely contain that context and its own references without a self-referential
representation. This implementation uses scoped construction instead:

1. D5 owns the immutable module and universe arenas.
2. `with_proof_context` creates a short-lived borrowed context.
3. D6 checked contracts, D7 checked IRs, the subflow universe and D8 snapshot
   are built and consumed entirely within the scope.
4. Only owned digest/content/association projections leave the scope.

There is no `Box::leak`, unsafe lifetime extension, deserialized checked
wrapper, caller-filled `checked=true`, or separately supplied Resolver
identity. The public `ReconstructedStaticInputs` projection has no
RootExport/RootRecovery capability and cannot be used as one.
