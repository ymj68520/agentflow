# Phase 4B.2B.1 Decisions

1. Reuse `af-effect`, `af-failure`, `af-flow`, and `af-usage` control/algebra
   functions; no parallel simplified algebra is introduced.
2. The analyzer checks the exact caller SemanticIrDigest, leaf node IDs and
   proof-context object before traversing. Checked IR ownership/order and
   Phase 3C dataflow remain prerequisites.
3. Child order follows the wire semantics: Sequence list order, and checked
   SELECT/PARALLEL/RACE ordinals. No map iteration determines meaning.
4. Sequence children after index zero have conditional execution. Their
   Effect lower bounds are weakened to zero before formal accumulation;
   upper bounds and occurrence sources remain. Empty Failure is not used as
   a proof of guaranteed success.
5. Frozen SELECT has an int64 selector and case ordinals but no default or
   range field. A direct matching integer constant may resolve a case;
   otherwise case totality is Unproven. Dynamic cases use branch join and
   conservatively zero Effect lower bounds. No IR change or guessed default.
6. PARALLEL ALL uses formal accumulation. FAIL_FAST also retains all sibling
   upper bounds but weakens lower bounds because early cancellation can skip
   a sibling's effect. `max_inflight` is never a total-work multiplier.
7. RACE uses all-contestant accumulation, not Select join. All branch K/X/R/F
   source records are retained; loser cancellation never removes possible
   work, Usage, Effect or Recovery.
8. Typed outputs use checked IR interfaces and the explicit Parallel output
   map. Local SemanticTypeIds remain local to the one caller IR.
9. Per-value taint transfer is not encoded in leaf summaries. Sequence data
   edges and selector dependencies are recorded; known may-influence is
   conservatively merged and unresolved per-value/decision claims remain
   Unproven.
10. R, K and X are represented per child, not flattened to a fictitious
    ComponentContract. Shared resources are aliases only when the same exact
    caller ResourceSlotId is explicitly present.
11. Child verdicts and source references propagate without overwrite. A
    known envelope never downgrades or clears child RequiresEvidence.
12. ITERATE/RETRY/TIMEOUT/HANDLE/SCOPE/COMPENSATION_SCOPE are Unsupported and
    Unproven in this phase. Their descendants may be inspected, but no
    operator-specific empty summary is synthesized.
13. No control digest, persistence ABI, Runtime dispatch, winner selection,
    cancellation, Worker placement, plan, journal, reducer or authority is
    added. Stop before Phase 4B.2B.2.
