# Seven native cases

|Rank|Case|Kind / clause|Role|
|---|---|---|---|
|0|canonical_source|native_recovery_guard_evidence / native_guard.canonical_source|mandatory_preflight|
|1|context_association|proof_context / direct.proof_context|mandatory_preflight|
|2|action_sources|native_recovery_guard_evidence / native_guard.action_sources|supporting_fact|
|3|action_bounds|native_recovery_guard_evidence / native_guard.action_bounds|supporting_fact|
|4|guard_inputs|native_recovery_guard_evidence / native_guard.guard_inputs|supporting_fact|
|5|guard_containment|native_recovery_guard_evidence / native_guard.guard_containment|supporting_fact|
|6|guard_evaluation|native_recovery_guard_evidence / native_guard.guard_evaluation|supporting_fact|

Every case is contract_validation and static_declaration,with supporting(exact PredicateId) dimension. Guard inputs parameterize forward/action,not extra cases. Subject cases require genuine admitted canonical/context facts;per-binding cases require the actual occurrence. A skipped kernel produces no containment fact. Native mandatory preflights do not alter RootExport's original five.

