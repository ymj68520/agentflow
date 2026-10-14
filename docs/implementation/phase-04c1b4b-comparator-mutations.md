# Phase 4C.1B.4B — comparator mutations

Starting HEAD: `c5b49850d9ba0dce75255f152498468fe8e93762`. This phase targets only 16 original D9-Recovery cases; the 22 Control baseline is retained. Closure depends on the recorded committed gates. D10 independent values are Pending for all 78; B01/B02 Closure and B03–B07 remain BLOCKING. No later phase or runtime/publication capability is created.

Preserved: 68 legacy integrity self-tests and 42 independent 4A mutations. New Recovery mutations start only from independently frozen Expected. Both-side root_bound omission and unexpected fields are rejected by shape, before equality. Arrays retain multiplicity; proof arrays canonicalize without deduplication. Deep paths are reported.

| Mutation | Typed path |
| --- | --- |
| caller-digest | d9_root_recovery.actions.actions.0.source.caller_semantic_ir_digest |
| source-node | d9_root_recovery.actions.actions.0.source.source_node |
| contract-digest | d9_root_recovery.actions.actions.0.source.source_contract_digest.hex |
| forward-ordinal | d9_root_recovery.actions.actions.0.source.forward_entry_ordinal |
| action-kind | d9_root_recovery.actions.actions.0.source.action.action_kind |
| forward-atom | d9_root_recovery.actions.actions.0.source.action.forward_effect.operation.name |
| action-atom | d9_root_recovery.actions.actions.0.source.action.action_effect.qualifiers.0.member |
| action-guard | d9_root_recovery.actions.actions.0.action_effect_occurrence.declared.guard |
| activation | d9_root_recovery.actions.actions.0.recovery_usage_binding.activations_per_forward.upper.finite |
| attempts | d9_root_recovery.actions.actions.0.recovery_usage_binding.attempts_per_activation.upper.finite |
| owner-scope | d9_root_recovery.actions.actions.0.owner.node |
| owner-variant | d9_root_recovery.actions.actions.0.owner |
| local-bound | d9_root_recovery.actions.actions.0.local_bound.upper.upper.finite |
| local-option | d9_root_recovery.actions.actions.0.local_bound |
| root-bound | d9_root_recovery.actions.actions.0.root_bound.upper.upper.finite |
| logical-repeat | d9_root_recovery.actions.actions.0.root_bound.applied_families.0.logical_repetitions.upper.finite |
| physical-attempt | d9_root_recovery.actions.actions.0.root_bound.applied_families.0.physical_attempts.upper.finite |
| factor-identity | d9_root_recovery.actions.actions.0.root_bound.applied_families.0.identity.control_node |
| factor-kind | d9_root_recovery.actions.actions.0.root_bound.applied_families.0.identity.kind |
| factor-missing | d9_root_recovery.actions.actions.0.root_bound.applied_families |
| forward-occurrence | d9_root_recovery.actions.actions.0.forward_occurrence.source_entry_ordinal |
| action-occurrence | d9_root_recovery.actions.actions.0.action_effect_occurrence.computation |
| usage-binding | d9_root_recovery.actions.actions.0.recovery_usage_binding.action.action_kind |
| path-step | d9_root_recovery.actions.actions.0.control_path.0.parent_node |
| path-reachability | d9_root_recovery.actions.actions.0.control_path.0.reachability |
| eligibility | d9_root_recovery.actions.actions.0.eligibility_evidence |
| guard-verdict | d9_root_recovery.actions.actions.0.guard_compatibility.verdict |
| guard-option | d9_root_recovery.actions.actions.0.guard_compatibility |
| countermodel-presence | d9_root_recovery.actions.actions.0.guard_compatibility.countermodel |
| guard-upper | d9_root_recovery.actions.actions.0.guard_compatibility.action_upper.finite |
| factor-duplicate | d9_root_recovery.actions.actions.0.root_bound.applied_families |
| source-closure-verdict | d9_root_recovery.source_closure.usage_profiles_accounted |
| typed-guard-coordinate | d9_root_recovery.actions.actions.0.guard_coordinates |
| epsilon-coverage | d9_root_recovery.epsilon_coverage_requirement.value.entries.0.bounds.upper.finite |
| action-member-missing | d9_root_recovery.actions.actions |
| action-member-extra | d9_root_recovery.actions.actions |
| recovery-meter | d9_root_recovery.actions.actions.0.recovery_usage_binding.per_attempt_usage.0.meter.name |
| recovery-phase | d9_root_recovery.actions.actions.0.recovery_usage_binding.per_attempt_usage.0.phase |
| recovery-unit | context.usage_meters.0.unit.name |
| root-recovery-usage | d9_root_recovery.recovery_usage.value.coordinates.0.bound.upper |
| local-recovery-usage | d9_root_recovery.actions.actions.0.local_recovery_usage.coordinates.0.bound.upper |

