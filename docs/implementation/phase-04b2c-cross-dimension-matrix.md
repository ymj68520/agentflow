# Phase 4B.2C — Cross-dimension soundness matrix

This matrix distinguishes the **leaf contract proof** from the **control
composition**. The former is checked against exact catalogs and relations;
the latter preserves source-scoped facts and applies operator-specific E/U
algebras. A per-child summary is not an exported aggregate contract.

| Relation | Leaf/source proof | Control propagation | Audit verdict / boundary |
| --- | --- | --- | --- |
| `epsilon ↔ R` | Component validator checks exact ResourceRequirement/EffectResource relation; resolver maps each effect selector port to its source component resource port and caller ResourceSlotId. | Root retains Effect occurrence mappings and per-child Resource requirements; Scope uses exact Phase 3C visible sets. | Locally Proven where relation evidence exists. Same numeric port across components never merges. ResourceSlot visibility is not a handle or authorization. |
| `epsilon ↔ F` | EffectFlowPartition is checked against exact atom/operation and F ceiling; unknown/open residuals do not become covered. | Flow sources/taints/constraints and effect partitions remain attached to source nodes; decision/winner influence can set Unproven. | Source-scoped; no control branch drops a possible effect or its source. No proof of actual egress/dispatch. |
| `K ↔ epsilon` | Exact action identity uses full forward atom, action kind, action atom; both atoms must exist in a closed E row; required K clauses/bindings are checked. | CompScope derives per-node conditional action bounds and checks summed potential **upper** against the matching action-E occurrence; same atom case includes Forward plus action count. | **P1 design gap:** action-E guard containment is not proved. The separate action sidecar is wider (conservative) than a narrower E guard, but root/export guard semantics must be frozen. Root-total K aggregation is also unspecified. |
| `K ↔ U` | RecoveryUsageCoverageCatalog names complete meter coverage; K per-attempt recovery profile × forward upper × activation × attempts is checked with overflow rejection against U.recovery. | Control U algebra scales/sums/maxes the declared U envelope by operator; K binding facts stay per source. CompScope's `potential_recovery_usage` is separate and is not re-added into root U. | No observed double addition in root U. Root local-vs-outer K action count coordinate remains unspecified; missing meter is not finite zero. |
| `F ↔ X` | Exact FlowExecution relation pairs sink/flow-kind and execution profile constraints under a proof context. | F and X remain per child/source; parallel/race does not synthesize one worker profile. | Relation proof is source-level. No Worker admission, placement, sandbox execution, or runtime attestation. |
| `Phi ↔ control outcomes` | Typed failure refs are exact pinned identities. | Sequence/Select/Parallel/Race/Retry/Timeout/Handle use existing Failure algebra; TimedOut/cancel/RecoveryBlocked are not business Failure types. | Conservative possible Failure row; no physical winner or recovery outcome selected. |
| `R ↔ Scope` | ResourceSlot exists and Phase 3C entry sets enforce lexical visibility. | Scope child sees exactly `ScopeBody.resources`; Resource requirements remain source ports + caller slots. | Proven lexical attenuation only; no resource handle, grant, or action-specific rights. |
| `K ↔ F/X` | Frozen Recovery/K carries action identity and Usage but no per-action Flow/X map. | Full source-contract Flow/X stays on the child; conditional action references source and exact action Effect occurrence. | Action-specific F/X conformance cannot be proven separately; source-level proof is retained, not upgraded. |
| `U ↔ price/budget` | Cost is separately projected from static Usage and fixed pricing; Scope has no budget ceiling. | Controls preserve forward/recovery coordinates and apply checked static algebra. | No reservation, ledger balance, billing receipt, or actual spending assertion. |

## Proof-state product behavior

The control proof report retains each dimension proof and inherited child
obligation. Verdict precedence is `Violation > RequiresEvidence > Unproven >
Proven`; precedence chooses the summary's `overall` value but does not erase
lower-priority diagnostics. A known E/U upper may coexist with Unproven Flow,
unknown child provenance, or RequiresEvidence from recovery. This is the
intended “known fact + unresolved premise” behavior.

The whole-tree closure flags are a source-accounting result: every declared
node, leaf occurrence/residual, and K source was accounted for. They are not
the `Proven` verdict for each dimension, not body conformance, and not root
export compatibility.

## Exact checked arithmetic paths

- E sequence/parallel/race/retry/iterate uses `af-effect` checked algebra.
- U sequence/parallel/race/select/retry/iterate uses `af-usage` checked
  arithmetic over `(phase,meter)`.
- Leaf K usage is checked in `af-contract::check_recovery_usage` before the
  leaf is minted.
- CompScope action count and per-action Usage use checked multiplication;
  undercoverage of action E is a Violation.

The remaining ambiguity is not overflow or a proven dropped root E/U factor;
it is whether CompScope's exposed conditional-action bound is explicitly
local to one activation of that Scope and how a consumer should derive a
whole-workflow K aggregate through ancestor repetition.
