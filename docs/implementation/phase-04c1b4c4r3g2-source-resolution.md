# Exact source resolution

NativeEffectRowKind is closed/open_known/unknown. NativeEffectOccurrence{ordinal:U64,entry:EffectEntry},NativeUsageOccurrence{ordinal:U64,binding:RecoveryUsageBinding},NativeGuardOccurrence retains the separate action_binding_ordinal and full action_identity,plus row kind and forward/action/usage resolutions. Each concrete resolution is the closed not_queried/absent/present(T) sum. present ordinal0 is not absence.

The frozen analyzer executes both full-AtomKey E lookups before the missing-entry return,then actual usage lookup. Both executed E results must survive future capture before arbitration. Unknown row/missing entry/usage is U/InsufficientEvidence,not pure E,unconditional guard,zero usage or V. No later reason-string parser supplies resolution facts. Hooks remain unimplemented.

