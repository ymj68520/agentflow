# Phase 4B.1 — ComponentContract ABI and Typed Assembly

This implementation materializes the accepted Phase 4B.0B ABI in
`spec/idl/component-contract.afidl` and the three standalone relation schemas
in `spec/idl/`. The normative semantics are in `spec/component/`.

## Wire body

`ComponentContract` fields 1–13 are, in order: identity, inputs, outputs,
failures, effects, resources, information flow, usage, recovery, execution,
effect-flow partitions, recovery-action bindings, and recovery-usage
bindings. The identity and all thirteen fields are semantic digest material.
Publication version, implementation/package identity, prices, diagnostics,
proof context and proof output are not.

The digest domain remains the existing `component-contract`. Canonicalization
uses explicit-number port order, exact imported dimension canonicalizers, the
full Effect Atom key, and complete recovery-action identity. The digest API
accepts only a validator-created checked wrapper.

The v1 substitution API is deliberately proof-oriented: public I/O requires
exact port sets and exact pins, while failure/effect/resource/flow/usage/
recovery/execution reuse existing directional static relations. Execution
feasibility and security remain separately visible. Changes to per-action
flow/recovery refinements without an existing directional relation remain
`Unproven`.

## Scope boundary

The crate checks one contract against explicit static universes. It does not
walk Semantic IR control flow and introduces no runtime, worker, scheduler,
grant, handle, receipt, reservation, plan, journal, or reducer surface.
