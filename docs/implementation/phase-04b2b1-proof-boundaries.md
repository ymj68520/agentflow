# Phase 4B.2B.1 Proof Boundaries

## Proven subset

- Exact caller digest/proof-context matching for every supplied leaf, and
  Phase 3C dataflow evidence for the checked IR subtree.
- Typed output interfaces and explicit Parallel branch/port mappings already
  validated by Semantic IR; exact source slots remain available for tracing.
- Conservative Effect and Usage envelopes when each child supplies a typed
  row, using the formal checked algebra; occurrence provenance remains
  separate from algebraic AtomKey merging.
- Typed Failure union and complete per-child R/K/X declarations where those
  leaf dimensions are present.
- Conservative may-influence unions from known F summaries, with child
  constraints and partitions attached to their source contracts.

## Unproven or RequiresEvidence subset

- A missing/unresolved leaf, unknown Effect/Usage, or unsupported control
  operator never becomes empty. Known child contributions remain in the
  summary alongside the unresolved source list.
- Dynamic SELECT case coverage is Unproven because the frozen IR has no
  default/range field. Its branch join remains conservative and lower bounds
  are zeroed unless a direct constant selects an exact case.
- Sequence/SELECT/RACE value-control taint cannot be transferred exactly
  because the current F declaration is contract-level rather than per-value;
  Phase 3C edges and known may-influence remain, and the per-value relation
  stays Unproven where needed.
- A checked exported Subflow declaration is not body-conformance evidence;
  `RequiresEvidence` propagates through parent control nodes.

## Not claimed

The result does not certify external implementations, select a RACE winner,
choose a Parallel primary failure, execute cancellation/recovery, prove
runtime success, produce a global R/X row, or create a single-Worker demand.
It is not a ComponentContract and carries no control digest, grant, handle,
lease, admission, reservation, plan, journal, reducer, or runtime state.
