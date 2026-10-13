# Phase 4B.2F.2 — Static RootExport Readiness

## Decision: Outcome A, bounded

**Static RootExport Proof Subset: accepted as a possible future Release Control
input, subject to the boundaries below.**

Reasons:

- F01's only positive path is the frozen complete containment algebra; its
  fallback can only report Violation or Unproven.
- F02's private, structurally complete resolver identity binds the same
  target/export/dependency associations to Root and RootRecovery and is checked
  before any of the nine dimensions.
- The independent same-row and exact-snapshot positive cases still issue the
  checked static capability; this is not a permanent-Unproven solution.
- No new false-Proven was observed in the 57 runs with retained overall and
  capability results; the additional identity-order probe is not used as a
  compatibility result.
- F03's provenance list is incomplete, but exact full relevant source and
  registry content identity remains a mandatory fail-closed preflight.
- F04 can lower diagnostic precision only; it does not contribute to a
  positive result.

F03 and F04 remain open P2 findings, not silently closed. A Release Control
consumer must require current compiler-semantics identity, treat the applied
RuleId set as incomplete attribution, and reject every non-Proven or missing
preflight/dimension. If a consumer needs a complete per-rule dependency
proof, F03 must first be closed.

## Capability and evidence boundary

CheckedRootExportCompatibility proves only that the checked Root summary is
statically compatible with the exact checked export under the exact IR,
context, resolver, and assertion. It does not establish:

- component/subflow body conformance;
- trusted implementation or artifact identity;
- publisher authentication or attestations;
- executable plan or deployment admission;
- Worker or Scheduler authorization;
- Effect grant, Resource handle, Usage reservation, or runtime permission.

The report continues to mark body/implementation/runtime evidence as
RequiresEvidence or NotEstablished.

## Required conditions for any future static consumer

1. Consume only a CheckedRootExportCompatibility created by the production
   checker.
2. Require every preflight and all nine dimensions to be Proven.
3. Bind the exact checked IR, top export, proof-context object/content, and
   resolver snapshot used for the check.
4. Preserve current complete normative-source and registry content validation.
5. Do not infer body, publisher, implementation, deployment, or runtime claims.
6. Treat any change in resolver association as requiring fresh Root and
   RootRecovery derivation.

## No progression

This phase does not start WorkflowVersion, Publication, ExecutablePlan,
Journal, Reducer, Scheduler, Worker, or Runtime work.
