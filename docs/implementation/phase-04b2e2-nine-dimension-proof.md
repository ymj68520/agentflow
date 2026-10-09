# Phase 4B.2E.2 — Nine-Dimension Proof Rules

The proof direction is implementation-within-export. Each row is reported
independently; one passing dimension never fills another.

| Dimension | Positive proof | Known contradiction | Insufficient evidence |
| --- | --- | --- | --- |
| I | Complete bijection, exact nominal pins, direct checked use sites | Missing/extra required root input or illegal input alias | Incomplete map, transformed input expression, unresolved type |
| O | Every export output maps to exact WorkflowOutput, expression, type, and all-path producer | Known output absent or actual type incompatible | Unknown/incorrect witness, non-total producer, extra root output |
| Phi | Exact nominal root business Failure set is contained in complete export Phi | Known root Failure absent from complete export row | Root Failure availability incomplete |
| epsilon | Mapped closed root Effect row fits export, including atom, qualifier, scope, guard, and bound | Typed root effect/bound/guard is outside a complete export ceiling | Open/unknown row, residual, unmapped coordinate, unsupported guard relation |
| R | Mapped root requirements are not stronger than export R | Proven stronger requirement or actual type mismatch | Missing slot/attribute/alias evidence or unsupported relation |
| F | Mapped root Flow and partitions do not relax export restrictions | Proven relaxation or mapped partition member outside export | Unknown influence, missing source/sink map, unsupported relation |
| U | Forward and Recovery envelopes separately fit exact phase/meter/unit ceilings | Known upper exceeds ceiling or arithmetic overflow | Missing meter/phase/unit is not zero |
| K | Full source/action/profile/factor multiset is accounted; exact frozen Recovery relation holds | Known required action/profile absent or static action/E/U ceiling exceeded | Unsupported Recovery monotonicity, incomplete source/profile map |
| X | Every child separately satisfies both export feasibility and security constraints | Child is infeasible or weakens explicit security | Missing child/catalog facts or action-specific X not expressible |

Root K action and Recovery Usage projections are checked as coverage
obligations against already-derived root/export epsilon and U. They are never
added a second time. Runtime receipt/eligibility may narrow real actions but
does not narrow static bounds.

Business Failure Phi excludes timeout, cancellation, RecoveryBlocked and race
winner choice. Static U is not a budget. Feasibility and security remain
separate X results; no Worker is synthesized or admitted.
