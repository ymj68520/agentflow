# Phase 4B.2B.1 — Structured Control Contract Algebra

## Model and API

`af-computation::derive_control_contract` accepts a Phase 3B checked IR,
one node, validator-created `ResolvedComputationLeaf` values, and the exact
`StaticContractProofContext`. It recomputes the caller SemanticIrDigest and
Phase 3C dataflow and rejects any leaf whose digest or proof-context object
does not match. Child ordering is read from the checked IR: SEQUENCE list
order is semantic; SELECT/PARALLEL/RACE use their checked ordinal order.

The returned `DerivedControlContractSummary` is analysis-only and retains the
recursive child summaries, control node/operator, caller digest, typed output
evidence, each dimension's explicit availability state, source-scoped
occurrences, unresolved obligations, inherited proofs, and a direct proof
context reference. Unknown, Partial, PerChild and NotApplicable are separate
states. It is not a ComponentContract, ACE1 object, or digest input.

Only SEQUENCE, SELECT, PARALLEL and RACE get propagation rules. Other
control nodes are represented as Unsupported/Unproven while their child
summaries remain available; their dimensions never become empty by default.

## Operator algebra

- **SEQUENCE:** calls `af-effect` accumulation and `af-usage` checked
  accumulation; failure rows use the existing union rule. A later child may
  be skipped after prior non-success, so its Effect lower bound is weakened
  to zero before the formal algebra, retaining its upper/guard/source. Phase
  3C value dependencies between earlier child exports and later expression
  uses are recorded exactly. Since the current F model is contract-level,
  not per-value, a cross-sibling data edge keeps a conservative may-influence
  union and an Unproven per-value Flow obligation.
- **SELECT:** uses `af-effect::control::select_envelope` / branch join and
  `af-usage::control::select_envelope` / maximum. Full AtomKey qualifiers and
  guards are preserved; differing guards have lower zero per the frozen
  Phase 4A.1B correction. A matching constant int64 selector may select one
  exact child. Otherwise the frozen IR has no default or selector range, so
  coverage is Unproven and Effect lower bounds are conservatively zeroed.
  Dynamic selector value dependencies are retained as decision influence,
  never as effective instruction authority.
- **PARALLEL:** both frozen modes use conservative accumulation for Effect
  and Usage. `max_inflight` does not alter total usage. `all` retains lower
  bounds under its all-branches-complete contract; `fail_fast` weakens child
  lower bounds because sibling work can be cancelled before an Effect. Its
  possible sibling Effect/Failure/Usage facts remain. Parent output ports
  use the exact checked `ParallelOutputBinding` source branch/port mapping.
- **RACE:** calls the existing Effect/Usage accumulation rules over every
  contestant. It is not a branch join: loser Effect and Usage remain; lower
  bounds are weakened where execution to the effect is not guaranteed. Every
  contestant's K, exact action/usage bindings, and effect occurrences remain
  per child. The summary does not choose a winner or primary failure.

## Dimensions and provenance

Failure is the typed union of all potentially contributing child rows;
TimedOut/Cancelled are not added to Phi and empty Phi is not success proof.
Flow may-influence summaries use `af-flow` union semantics (source/taint
union, max may-carry authority, sticky unknown), with child Flow constraints
and EffectFlowPartitions retained by source. R aliases are grouped only by
explicit caller ResourceSlotId and preserve each child port mapping. K and X
remain lists of source-scoped per-child declarations; no global resource-row
union, single-Worker X sum, admission, or capacity calculation occurs.

Effect upper envelopes may combine equal AtomKeys using the existing formal
algebra. The separate occurrence list is never deduplicated: it retains
caller digest, leaf/control node, source contract digest, and source row
ordinal. Recovery compensation adds facts and never subtracts forward
effects.

## Verdicts

Every child non-Proven dimension is copied into the parent report and
`inherited_obligations`. The overall priority is Violation, then
RequiresEvidence, then Unproven, then Proven; all dimension details survive
that projection. A known conservative envelope can coexist with a
RequiresEvidence body or Unproven Flow/Usage relation.

No changes were made to the 13-opcode IR, schemas, ABI, digest domains,
runtime, plan, journal, reducer, or worker/authority surfaces. Phase 4B.2B.2
is not implemented.
