# Phase 4B.2F.1 — Proof Boundaries

The phase closes two routes to a false static compatibility result:

- epsilon fallback no longer splices guard and bound facts across export
  rows, and it does not reuse finite export slots;
- RootExport preflight requires the supplied root and RootRecovery provenance
  to match the exact current resolver snapshot, in addition to the exact
  proof-context pointer and assertion content manifest.

The positive claim remains exactly static Root/Export envelope compatibility.
All nine static dimensions must be Proven before a
`CheckedRootExportCompatibility` can be issued. P30 and P31 are rejected
without capability issuance. A valid complete same-row positive witness and
same-snapshot positive cases still receive the static capability.

Body/subflow conformance remains `RequiresEvidence`; implementation
conformance remains `RequiresEvidence`; publisher authentication and runtime
authority remain `NotEstablished`. Static Usage is not a budget, resource
mapping is not a handle or grant, and RootRecovery proof is not runtime
recovery authority. No WorkflowVersion, publication, plan, journal, reducer,
scheduler, worker, receipt, reservation, or runtime model is added.

**Release Control Readiness remains PENDING INDEPENDENT RE-AUDIT.** Closing
the two scoped P0s is not an approval to use this checker as release control.
