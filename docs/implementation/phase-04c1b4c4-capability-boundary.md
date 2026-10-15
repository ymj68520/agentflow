# Capability boundary preserved

All production source remains identical to b121ab716bf4e87e33e7433b2b9dc3407699d327. The one `CheckedRootExportCompatibility { report }` construction remains private in root_export.rs and independently guarded by implementation-owned `can_issue_capability`, no admission failure and static Proven. Analyzer and D4 still reject the unsupported edition. No Supported or Authorized state is introduced.

The only executable addition is an external diagnostic under ignored target, plus a read-only Python source auditor. Neither imports or constructs the private RootExport wrapper. No Capture API exists, so it cannot be a new issuance path. Existing51 staging cases retain their current API denial semantics; fresh targeted results are recorded in [the gate report](phase-04c1b4c4-gate-report.md).

This is preservation of T2 denial, not proof of a completed T3 capture boundary. Body/publisher/runtime remain unauthorized. No incomplete fact collection is returned as empty-success or used to promote a verdict.
