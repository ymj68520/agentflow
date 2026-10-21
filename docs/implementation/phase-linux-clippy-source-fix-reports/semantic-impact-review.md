# Semantic impact review — L2-PIVOT-07

The subject is `match &parent.operator`: this match borrows the operator. In the
`Handle` arm guarded by `child_index == 0`, the body only constructs
`ControlPathKind::HandleProtected`. The `clauses` name is never referenced there.
Removing its immutable borrowed local binding neither moves nor drops the field.
The variant match, wildcard remainder, guard and returned kind remain identical.
The following unguarded Handle arm still binds and uses clauses to derive its
ordinal, unchanged. Reachability, parent/child IDs and control-path fields are
unchanged. No effects, failure propagation, ABI or proof rules are modified.

The byte comparison asserts exactly one replacement and no other product bytes.
Cargo.lock, Expected, Oracle, Registry and frozen worktrees were not edited.
This is source reasoning and a byte-diff audit, not executed semantic-test proof
or third-party certification. Actual fmt/Clippy results must be recorded separately.
