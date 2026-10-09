# Phase 4B.2E.2 — Subflow Dependencies and Body Evidence

Every direct CALL_SUBFLOW assertion row fixes the call-site ComputationId,
exact target SemanticIrDigest, exact exported ComponentContractDigest when
present, optional complete target assertion identity, and typed transitive
target/export pins. The checker derives the reachable closure from the exact
resolution snapshot and compares it with the assertion. It does not trust a
caller-supplied transitive list.

Missing targets, stale export associations, duplicate call sites, missing
transitive nodes, and cycles fail closed as Unproven. They are never removed
from the closure to get a positive result.

A checked target export or even a target RootExportAssertion identity proves
only a static envelope relation. This checker has no separate trusted body
attestation input, so `body_conformance` and
`implementation_conformance` remain RequiresEvidence. The 4B.2E.2 corpus
contains a production case where all nine static dimensions are Proven while
subflow/body conformance is still RequiresEvidence.
