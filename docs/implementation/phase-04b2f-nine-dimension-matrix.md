# Phase 4B.2F — Nine-Dimension Soundness Matrix

| Dimension | Source path reviewed | Audit result |
| --- | --- | --- |
| I | `prove_inputs`; complete root parameter/export-port bijection and Phase 3C direct use sites | No false-Proven found in source review or executed mutations. Missing/duplicate/extra input probes were Unproven or Violation. |
| O | `prove_outputs`; named output, expression, schema, exit-value producer, complete control interface | No false-Proven found. Missing output was Violation; assertion-only mismatch was Unproven. Dedicated per-opcode branch countermodels remain designed, not run. |
| Phi | Complete root Failure row via nominal `af_failure::contains` | A known failure outside empty export Phi was Violation. Partial/unknown remains Unproven in source. |
| epsilon | Root E plus independent K/E coverage, `remap_effect_signature`, `effect_containment_verdict` | **P0 AUDIT-F01:** two different exported rows can contribute bounds and guard independently and produce Proven although no row covers the root Effect. Also involved in stale-snapshot P0 AUDIT-F02. |
| R | Exact source resource chain, attributes/visibility, existing resource non-strengthening relation | No false-Proven found. Changed root slot/type assertions fail closed. Unsupported non-empty attribute evidence stays Unproven. |
| F | Per-source Flow map, translated sinks/sources, partition membership, unknown-source checks | Flow relaxation was Violation; missing map was Unproven; unknown Flow negative was rejected before RootExport at ComponentValidation. No false-Proven confirmed. |
| U | Separate exact Forward/Recovery phase-meter-unit envelopes plus K coverage | Forward ceiling underdeclared by one was Violation. Root K-required usage is separately compared with Root U and Export U, not added. No overflow-underflow was reproduced. |
| K | `RootRecoverySummary` exact source/action/profile/factor closure, Recovery relation, exported action/E/U checks | Missing mapping/profile was Unproven; guard mutation stayed Unproven; formal recovery-usage underdeclaration was Violation. Independent arithmetic agrees with D2. Stale subflow provenance remains P0 through the source pins used by K. |
| X | Per-child execution mapping and separate feasibility/security compatibility | Missing map was Unproven; exported feasibility/security floor mismatch was Violation. Recovery action-specific X is explicitly Unproven. No Worker/admission claim. |

The aggregate report preserves nine distinct `DimensionProof`s and only issues
the checked capability when all nine report Proven. It also keeps body and
implementation conformance RequiresEvidence and runtime/publisher authority
NotEstablished. The two P0s do not arise from collapsing the final verdict;
they arise because epsilon/source provenance is incorrectly accepted as
Proven before aggregation.

## Control-opcode propagation review

The RootExport checker consumes a derived tree, not a second control fold. The
audited source paths preserve these existing summaries:

| Opcode | Root propagation relevant to this audit |
| --- | --- |
| INVOKE | Exact checked contract and source-scoped I/O/R/E/F/U/K/X facts |
| AWAIT | Exact signal pin; unknown external provenance remains Unproven |
| CALL_SUBFLOW | Exact target IR and snapshot-associated export; body remains RequiresEvidence; snapshot mismatch is AUDIT-F02 |
| SEQUENCE | Ordered accumulation; possible effect/action uppers retained |
| SELECT | Exclusive join; exact constant case can narrow only with typed selector evidence |
| PARALLEL | Accumulates possible work; max_inflight is not total work |
| RACE | Retains every contestant, including loser recovery/effects |
| ITERATE | Applies logical total-count upper once; unknown total remains unavailable |
| RETRY | Applies max-attempt logical multiplier once; physical attempts are not a second multiplier |
| TIMEOUT | Preserves work that may have occurred before timeout |
| HANDLE | Protected work plus conditional handler alternatives; no invented winner |
| SCOPE | Phase 3C lexical resource attenuation; no handle/authority |
| COMPENSATION_SCOPE | Local ownership only; root K fold starts from leaf sources once |

Existing 58/72/55/44/35 suites are regression evidence, not a substitute for
the two direct production false-Proven probes.
