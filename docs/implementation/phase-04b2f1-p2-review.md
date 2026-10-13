# Phase 4B.2F.1 — P2 Review

## AUDIT-F03 — Applied RuleId closure: OPEN, P2

The implementation still names 84 rules: AF-CONTRACT-0001..0022 and
AF-SEMANTIC-0111..0172. This is the declared set, not a demonstrated
transitive closure. RootExport calls checked contract validators, IR and
Phase 3C validation, resolver/leaf resolution, control algebra, Effect,
Failure, Resource, Flow, Usage, Recovery, Execution relations, and digest
recomputation. Those dependencies reach formal rules outside the current
0111..0172 semantic slice, including foundational semantic type, producer,
ownership, and dataflow rules.

Every stored RuleId is paired with its registered normative source, anchor,
and statement; a bare matching RuleId is not accepted. CompilerSemanticsIdentity
also equality-binds the complete `specifications.json` and `rules.json`
contents and embeds the exact bytes of its 16 declared applicable normative
sources. The registry records normative source content digests for the full
specification set. No stale-identity reuse bypass was reproduced in this
phase, but the selected RuleId list is not a dependency-graph proof and must
not be described as complete. The P2 remains open for a separately scoped
closure audit; this phase does not broaden into a registry redesign.

## AUDIT-F04 — Root-global guard countermodel gate: OPEN, P2

`root_has_guard_sensitive_path` still disables typed guard countermodels if
any SELECT, RACE, or HANDLE occurs anywhere in the root tree. This can suppress
a definite counterexample on an unrelated occurrence. It is conservative:
it may return Unproven, and no false-Proven was found. The F01 change does not
weaken this gate or infer branch reachability. Occurrence-local control-path
proof remains future work.

## Severity disposition

F01 and F02 are repaired with permanent production regressions. No additional
P0 or P1 was found in the scoped re-run. The 10 F02 production cases include
same-snapshot acceptance, changed target export association, fresh derivation
under B, wider top export with stale root, a changed component pin set, a
missing target export association, and equal resolver values under a distinct
proof-context object. F03 and F04 remain open P2 findings;
body evidence, publisher authentication, runtime authority, scope attenuation,
and runtime execution/recovery remain P3 or outside this static checker.
