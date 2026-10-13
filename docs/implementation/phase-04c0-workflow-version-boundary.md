# Phase 4C.0 — WorkflowDefinition / Draft / Version Boundary

All candidate models are **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED ABI /
NOT IMPLEMENTED**. Architecture research sections 17 and 24.3 constrain their roles;
the repository does not implement these resource types.

## Resource identities and mutability

| Concept | Identity/content | Mutability and producer |
| --- | --- | --- |
| WorkflowDefinition | Stable opaque logical identity and lineage/metadata | Mutable authoring root |
| WorkflowDraft | Exact authoring revision; holes and diagnostics allowed | Mutable authoring state, normalized by compiler |
| Compile result | Exact input tuple and static diagnostics/capability | Analysis output; no publication or execution permission |
| WorkflowVersion semantic artifact | Exact immutable canonical semantic core and all publication dependencies | Produced only after future mandatory Publish gate |
| WorkflowVersion logical reference | Definition lineage + immutable version reference + exact parent content pin | Cannot retarget an existing reference to different semantic content |
| DeploymentRevision | Exact version plus environment/binding policy | Separate immutable deployment snapshot |

A draft is not a mutable WorkflowVersion with status DRAFT. Visual Editor,
AI Builder and Authoring SDK submit authoring inputs; they do not create
trusted artifacts or executable Plans directly.

## Candidate semantic core

~~~text
WorkflowVersionSemanticCore (candidate) {
  canonical_schema_identity: future versioned parent schema
  semantic_ir: existing exact SemanticIrDigest + retrievable canonical content
  exported_contract: existing exact ComponentContractDigest
  plan_core: future typed exact pin in registered plan-core domain
  lock_graph: future typed exact pin in registered lock-graph domain
  compiler_semantics: static + lowering semantic content identities
  runtime_semantics_profile: future exact formal profile
  publication_static_proof_bundle: complete reproducible inputs inline
  semantic_parameter_baseline: fully materialized publish-time constants
  declared_run_parameter_constraints: future typed bounded domains
  evidence_requirements: exact body/implementation subject and predicate closure
}
~~~

This is an ownership list, not a canonical schema. The complete encoding,
closed variants, identity prefixes, type-schema association, duplicate rules,
semantic/nonsemantic fields and goldens must be defined before hashing.

`SemanticIrDigest` cannot stand in for `PlanCoreDigest`. A registered digest
domain and a syntactically valid hash do not establish a valid object in that
domain. `RootExportAssertionIdentity` is a structural input tuple, not the
WorkflowVersion artifact identity.

## Semantic identity versus publication identity

The prospective artifact content identity commits the complete semantic core.
Logical Definition/version names, presentation layout and external signature
bytes belong to separate typed records unless a field is observable semantic
input. Same semantic core under two permitted logical publication references
can share content identity; the authenticated publication records remain
distinct and bind their namespace, role, version reference and policy.

Changed title/color/source-map location does not change semantic identity.
Changed retry/timeout/prompt/tool selection constraint or semantic feature flag
does. Model-facing metadata that alters behavior is semantic content and cannot
be classified as presentation by its file name.

An existing logical version reference cannot be silently retargeted. Proposed
conflicting content under that reference is rejected; corrected semantics
requires a new version reference. A policy revocation or new verification
receipt changes current eligibility, not the immutable version's bytes.

## Authoring-to-publication transition

~~~mermaid
flowchart LR
  D["Draft revision"] --> N["Normalization + explicit semantic constants"]
  N --> I["Checked IR + contracts + typed dataflow"]
  I --> L["Exact dependency closure / LockGraph<br/>missing"]
  I --> R["Root + RootRecovery + RootExport<br/>implemented static subset"]
  L --> R
  L --> P["Plan lowering + preservation<br/>missing"]
  R --> G["Release evidence + policy gate<br/>missing"]
  P --> G
  G --> A["Canonical parent artifact construction<br/>missing"]
  A --> W["Publish immutable WorkflowVersion<br/>missing"]
~~~

The preliminary static proof may run while LockGraph is still a design model;
publication must revalidate against the final closed dependency projection.
No transform after proof can change IR, parameter constants or dependency
facts without fresh derivation.

## Artifact levels

| Level | Allowed purpose | Mandatory boundary |
| --- | --- | --- |
| Internal analysis result | Diagnostics, design/debugging | Cannot be supplied as a published version or execute |
| Non-executable compiled archive, prospective | Retain exact compile inputs/results once an archive schema exists | No deployment/run eligibility; missing Plan/Lock/body evidence remains visible |
| Publishable immutable WorkflowVersion, prospective | Stable semantic build for later environment evaluation | All mandatory publish predicates satisfied, including required body evidence |
| Deployable version/revision, prospective | Evaluated combination with exact environment/bindings | A separate result; not a mutable status inside WorkflowVersion |

Renaming an analysis archive “version” does not satisfy the gate. Missing
PlanCore, LockGraph or required body attestation blocks real Publish.
Concrete deployment-selected implementations are addressed by typed
implementation prerequisites: publication cannot claim their conformance before
they are known, and deployment cannot satisfy them with a contract digest alone.

## Current readiness

The boundary and identity ownership choices are ACCEPTED. WorkflowVersion
canonical schema, release record, attestation subjects, context schema,
PlanCore, LockGraph and their validators are absent. Concrete WorkflowVersion
implementation and actual publication remain BLOCKING.
