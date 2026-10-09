# Phase 4B.2B.3 — Proof boundaries

| Claim | Result | Evidence / missing input |
| --- | --- | --- |
| Ownership and no detach | Static Proven by checked IR tree | Phase 3B ownership validation and root walk |
| Scope resource subset | Static Proven when exact Phase 3C sets agree | ResourceSlot visibility only |
| Resource possession/authorization | Not proven | No ResourceHandle or runtime grant |
| Scope authority attenuation | Unproven | No static authority ceiling or grant input |
| Scope budget attenuation | Unproven | No budget ceiling/reservation input |
| Child settles before owner terminal | RequiresEvidence | Static ownership cannot observe runtime terminal state |
| Forward Effect/Failure survives compensation | Static Proven as propagation | No subtraction or outcome rewrite |
| Compensation potential upper | Conservative static upper | Exact K identity and checked activation/attempt bounds |
| Concrete applied state/action eligibility | RequiresEvidence | No EffectReceipt; reconciliation is runtime |
| Multi-event reverse order | RequiresEvidence | Source order is not commit-journal order |
| Root node/source closure | Static Proven when accounting checks pass | Checked IR, resolved leaves, exact source equality |
| Root export compatibility | Unproven / not expressed | No root ComponentContract pin or coordinate mapping |

**Static Proven Facts:** only claims explicitly backed by the table's checked
inputs are Proven.

**Conservative Upper Bounds:** include all possible effects and usage; they are
not execution claims.

**RequiresEvidence:** runtime observations and ordering remain obligations.

**Unproven:** unavailable ceilings, mappings, or incomplete sources remain
visible and cannot be defaulted to zero.

**Runtime-Only Obligations:** authority, resource acquisition, reservations,
receipts, reconciliation, settlement, dispatch, and completion.
