# Phase 4B.2F.2 — Metamorphic Soundness Results

| Invariant | Independent input and observation | Result |
| --- | --- | --- |
| M1 Same-row integrity | Add/reorder incomplete rows around split-row P30; ε stayed Unproven | Pass |
| M2 Candidate ordering | Reverse export order; same non-Proven result. Reverse resolver insertion retained proof-context Proven for identical associations | Pass for observed verdict/property; ordering-only case's overall/capability bit was not retained |
| M3 Slot capacity | Two roots/one export slot was Unproven; two roots/two full slots remained Proven | Pass |
| M4 Selector identity | Change scope, port, or qualifier; no old witness was reused | Pass |
| M5 Snapshot association | Same export digest set with swapped target associations failed proof-context preflight | Pass |
| M6 Snapshot equality | Independent Snapshot A/B with exact structural content remained eligible and one exact positive issued capability | Pass |
| M7 Snapshot mutation | Changing one target export association invalidated the stored Root identity | Pass |
| M8 Fresh derivation | Fresh Root under current changed snapshot was re-evaluated and reported the too-narrow top export as Violation | Pass |
| M9 Recovery consistency | RootRecovery is stored inside the privately constructed Root summary from the same derivation; changed recovery profile either invalidated identity or was rejected by contract validation | Pass, with R22 blocked before RootExport |
| M10 Context/compiler invalidation | Different context and mutations to sources/registries/RuleIds produced preflight Unproven and no capability | Pass |
| M11 Static/runtime separation | P31 remains body RequiresEvidence; report marks implementation/runtime/publisher authority not established | Pass |

M1 does not assume every incomplete export row must yield a Violation. Where
the global relation cannot establish a complete witness or a decisive local
countermodel, the required result is conservative Unproven.

The ordering-only resolver probe asserted only the preflight identity
dimension, not full compatibility. It is recorded as a limited observation,
not as evidence that an unchecked path issues a capability.
