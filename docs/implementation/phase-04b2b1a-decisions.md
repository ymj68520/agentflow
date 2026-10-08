# Phase 4B.2B.1A Decisions

1. Keep the production control algebra unchanged unless a real production
   differential demonstrates a semantic defect. The current work repairs the
   test Oracle and adds test evidence only.
2. Upgrade the control corpus from revision 1 to revision 2 because the old
   open-row completeness flag and dynamic-SELECT decision inputs were
   semantically wrong/incomplete. CaseIds and case count remain stable.
3. Use explicit fixture evidence-source tags rather than treating an expected
   child verdict as input. Materialization must still run checked IR,
   `resolve_computation_leaf`, and `derive_control_contract`.
4. Model the four former generic `compose` vectors as the frozen
   `SEQUENCE@1` operator. `compose` is not an IR opcode.
5. The bounded-open Effect vector is not resolvable through the existing
   checked-leaf/snapshot capability path. Do not bypass the private
   `CheckedComponentContract` constructor or synthesize a `ResolvedLeaf` in
   the test. Report it as unexecuted unless the user authorizes a formal
   proof-boundary change in a separate phase.
6. Historical Oracle-first ordering cannot be retroactively claimed. The
   independent review is recorded here; all results must distinguish that
   later review from the earlier development sequence.
7. Do not enter Phase 4B.2B.2 from this task.

