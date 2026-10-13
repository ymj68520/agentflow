# Phase 4C.1B.1 — D0–D4 Trust Boundary

The Rust types form one-way capabilities:

`RawPortableBytes`
→ `DecodedPortableBundle`
→ `CanonicalPortableBundle`
→ `ReferenceValidatedPortableBundle`
→ `ContentValidatedPortableBundle`
→ `SupportedPortableStaticInputs`

Each wrapper is privately constructed by its stage validator. Its payload
remains caller-supplied raw data plus progressively verified shape,
canonicality, references, content identity and current-profile support.

`SupportedPortableStaticInputs` does not contain a Checked component contract,
Checked Semantic IR, static proof verdict or resolver snapshot. This phase
does not call D5 Context reconstruction, D6/D7 formal validators, D8 resolver
construction or D9/D10 Root/RootRecovery/RootExport checks. A submitted report,
origin claim, compiler build checksum or local Rust pointer cannot create
those later capabilities.

Profile A is full-content revalidation only. The existing 84 applied RuleIds
remain an incomplete attribution set. Profile B/C, historical dispatcher,
rule-complete cache and certificate verification remain unsupported. Lowering
and full RuntimeSemanticsProfile remain ModelMissing. F03/F04 remain OPEN/P2.

Origin authentication, body/implementation evidence and release authorization
belong to B05/B07. LockGraph, PlanCore, WorkflowVersion, Publication and runtime
remain separate blockers.
