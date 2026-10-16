# Registered local bound trace

NativeBoundTrace retains declared_forward_bounds,ordered consulted_peer_bindings,effective_upper,forward_times_activation,partial_times_attempt,action_upper and closed outcome ready/unisolated/typed_failure. Each peer retains its real binding ordinal/full binding/usage resolution. Unisolated causes are closed foreign_forward or missing_peer_usage with the real peer payload.

For different atoms use declared upper. Same-atom peers derive sum(activation×attempt),divisor1+sum,then effective upper:unbounded ceiling stays unbounded;finite ceiling/unbounded divisor yields0;otherwise floor division. Zero multiplication precedes unbounded handling. Foreign forward contributions and missing peer usage stay unknown/unisolated. Lower conflict and checked overflow retain existing V;no Root retry/iterate factor is applied.

Five NativeBoundFailure variants have full payloads: same_atom_product_overflow{binding_ordinal,activation_upper,attempt_upper};same_atom_sum_overflow{processed_binding_ordinals,before,next};same_atom_divisor_overflow{sum};same_atom_lower_conflict{declared_bounds,sum,divisor,effective_upper};action_upper_overflow{step,left,right}. Step is forward_times_activation/partial_times_attempt. A failed first multiply leaves the second not_queried;an attempted unavailable result is absent with the exact closed failure outcome,not a made-up second error. Capture must precede Err(()) compression;G2 adds no hook.

