# Phase 4B.2A — Computation Leaf Contract Resolution

## Model and API

`af-computation` consumes a Phase 3B `CheckedIrDocument`, recomputes Phase 3C
dataflow, and accepts a frozen `StaticComputationResolutionSnapshot`. The
snapshot builder requires one exact `StaticContractProofContext`; it admits
only checked contracts whose proof-context object is pointer-identical to
that context and recomputes each existing ComponentContractDigest. After
`build`, the snapshot exposes no mutation API.

The leaf result is derived-only. `ResolvedComputationLeaf` retains the caller
SemanticIrDigest and node id, exact resolved target identity/pin, port and
resource mappings, source-preserving Effect and Flow projections, typed
dimension summaries, a direct reference to the proof context, and one
dimension-preserving proof report. No new wire shape or digest is defined.

## INVOKE

`ContractRefEntry.digest` is the sole target key. A resolved identity is read
from the checked ComponentContract. Input ports match exact port numbers and
only a direct, identical public `TypeSchemaRef`; local primitive or
container types remain `Unproven` where a public named pin is required.
Phase 3C provides expression scope/dominance and output producer evidence.

Resource ports map explicitly to visible caller `ResourceSlotId`s and exact
resource type pins. Because Semantic IR carries no candidate resource
attributes, non-empty contract attribute requirements remain `Unproven`.
Port aliases are materialized and do not reduce any bound.

Known Effect atoms and open-row residuals carry the source contract digest,
caller digest, node id, original selector/qualifiers/bounds, and a separate
mapped slot. Two leaves referencing one component contract stay distinct.
Information Flow records map source/sink coordinates without removing
unknown provenance or modifying authority. Failure, Usage, Recovery and
Execution rows are copied per invocation; no retry scaling or workflow
placement inference occurs.

## CALL_SUBFLOW

The analyzer calls the node-scoped Phase 3D interface checker, so existing
port and structural-type rules are reused rather than shadowed. The explicit
snapshot associates an exact target SemanticIrDigest with a complete checked
export contract; exact I/O/R pins are checked against the derived interface.
This association is explicit analysis input and is not inferred from target
port cardinality.

`SubflowIrUniverse` now stores the sorted/deduplicated exact digest edges
derived from each checked target's CALL_SUBFLOW nodes. The leaf analyzer
checks that explicit closure for cycles and missing targets. It never
recursively expands contracts. Phase 3D interface proof plus an exported
declaration is not body-conformance evidence; without a trusted complete
proof, the body dimension remains `RequiresEvidence`.

## AWAIT

The exact signal `TypeSchemaRef` is resolved in the snapshot schema universe;
correlation scope is checked by Phase 3C. The result records no implicit
timeout, no interaction adapter, unknown external influence, and
uninferred authority. It remains partial/`Unproven` absent a full adapter
contract; no timer, mailbox, accepted-signal, cancellation, or runtime state
is created.

## Boundaries

No control composition or runtime authority is implemented. In particular,
there is no SEQUENCE/SELECT/PARALLEL/RACE/ITERATE/RETRY/TIMEOUT/HANDLE/SCOPE
contract closure, no grant/handle/admission/reservation/receipt, and no
assertion that a checked declaration independently proves implementation
behavior.
