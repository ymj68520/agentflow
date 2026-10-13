# Phase 4B.2F.1 — Proof Context Invalidation

RootExport preflight now binds three distinct facts rather than conflating
them:

1. **Exact process context:** checked export, snapshot, root, and caller use
   the same pointer-identical `StaticContractProofContext`.
2. **Resolver content provenance:** the root summary's resolver identity,
   generated during derivation, structurally equals the current immutable
   snapshot identity.
3. **Assertion content manifest:** a fresh capture of catalogs, schemas,
   relation contents, component pins, subflow targets, direct dependencies,
   and target export pins equals the assertion manifest.

The root identity includes every component digest plus every resolver
subflow association: target SemanticIrDigest, whether direct dependency data
is present, its complete digest list, and whether an exported contract is
associated with that target. Sort order is deterministic. Snapshot digests
come from `StaticComputationResolutionSnapshotBuilder` after validation of
the checked contract and its exact context.

`DerivedWorkflowContractSummary` is privately constructed by
`derive_workflow_contract_summary`; it stores the identity with both the
control root and RootRecovery summary. RootRecovery is folded from that same
root in the same derivation. There is no API that accepts a separately
supplied RootRecovery summary or a caller-created resolver identity.

When Snapshot A and Snapshot B share the same proof-context pointer but differ
in a target export digest, the stored root identity and fresh snapshot
identity differ. Preflight becomes Unproven before the nine dimensions can
consume old Root or RootRecovery data. The assertion may correctly match B;
that does not authenticate Root(A). Re-deriving under B creates a new summary
and a new identity.

This invalidation metadata is in-process only. It adds no WorkflowVersion,
wire field, assertion digest, HashV1 domain, or cross-process cache behavior.
