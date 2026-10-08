# Phase 4B.2B.1C — Comparator Mutation Report

Mutation fixtures are isolated Expected copies. The production result is not changed. The dedicated 58-case test ran **15/15** mutations; each was rejected with a semantic path or a manifest identity-set mismatch.

| Mutation | Detection evidence |
|---|---|
| Remove a RACE loser Effect occurrence and reduce its merged upper | `race/all-contestants-not-winner-only.expected.effect_occurrences.length` |
| Change finite loser Usage upper 300 → 200 | `race/loser-usage-retained.expected.usage.coordinates.forward/api_calls` |
| Downgrade RequiresEvidence to Proven | `proof/requires-evidence-propagates.expected.verdict` |
| Change Effect qualifier member `natural` → `none` | `sequence/two-effects-accumulate.expected.effects.entries[0].atom.qualifiers[0].member` |
| Change Effect lower bound 1 → 0 | `sequence/two-effects-accumulate.expected.effects.entries[0].lower` |
| Delete the open Effect residual | `proof/open-effect-row-remains-open.expected.residuals.length` |
| Delete the Recovery Usage phase coordinate | `sequence/forward-recovery-phases-separate.expected.usage.coordinates.recovery/api_calls` |
| Change FailureTypeRef alias | `parallel/all-effects-accumulate.expected.failures[0]` |
| Remove/change explicit Resource alias | `parallel/shared-slot-alias-explicit.expected.resources_by_child[1].requirements[0].resource_slot` |
| Delete SELECT decision dependency | `select/decision-input-influence.expected.flow.decision_dependencies.length` |
| Change expected typed-merge rejection stage | `select/typed-merge-mismatch.expected.stage` |
| Change known finite contribution at overflow | `negative/effect-overflow.expected.effects.known_contributions[0].entries[0].upper` |
| Delete inherited RequiresEvidence obligation | `proof/requires-evidence-propagates.expected.obligations.length` |
| Change exact FailureTypeRef schema digest | `parallel/all-effects-accumulate.expected.failure_types[0].schema_digest.hex` |
| Remove a manifest CaseId | manifest/discovered CaseId set equality |

The mutation suite remains separate from the formal 58 CaseIds and does not inflate the executed-case count.
