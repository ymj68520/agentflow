# Phase 4C.0 — WorkflowVersion / Publication Boundary Architecture

All proposed records, predicates, entry points, and phase names in this package
are **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED ABI / NOT IMPLEMENTED**.
ACCEPTED means an architecture choice has been made; it does not establish a
machine schema, production capability, or permission to publish.

## Result and baseline

The selected publication baseline is **fresh full static proof revalidation**
under a reconstructed exact context. A future versioned parent semantic artifact
owns the complete RootExport assertion and reproducible inputs inline. Publication
also requires closed semantic dependencies, a conforming PlanCore, authenticated
publisher and required conformance/build evidence, and an independent release
policy decision.

WorkflowVersion implementation is **BLOCKED** by the missing formal models
listed below. Current compile results remain useful for internal analysis.

Start HEAD: `92284a764c12550434fc3747e3369146d3a89309`; worktree clean.
The frozen repository has 41 specifications, 380 rules (379 covered, one
intentional exception), 477 active ABI assignments, 16 AFIDL modules, and 14
HashV1 domains. Historical production differential coverage is 311 cases:
58 / 72 / 55 / 44 / 35 / 47.

Phase 4B.2F.2 is accepted as bounded evidence for the current static subset:
F01/F02 CLOSED, F03/F04 OPEN P2. Its 62 designs, 58 RootExport-reaching calls,
and 57 recorded full verdict/capability outcomes retain their historical
meaning. This phase does not reclassify the ordering-only probe as a complete
compatibility comparison.

## Evidence read and precedence

The design uses these repository documents:

- Phase 4B.2D: root-export-assertion, proof-context-lifecycle,
  cross-dimension-proof, p1-decisions.
- Phase 4B.2E.2: root-export-model, nine-dimension-proof,
  proof-context-lifecycle, subflow-dependencies, proof-boundaries.
- Phase 4B.2F: false-proven-analysis and findings.
- Phase 4B.2F.1: f01-effect-containment and f02-snapshot-binding.
- Phase 4B.2F.2: audit-overview, f01-same-row-review,
  f02-resolver-identity-review, rule-closure-review, readiness.
- [Architecture research v6](../agent-workflow-architecture-research-v6.md):
  sections 13.5–13.9, 14.1–14.9, 15.1–15.2, 17.1–17.7, 22.20, 23.1–23.4,
  24.2–24.3, 24.16, and 24.23.

Research terminology such as FROZEN describes architectural decisions, not
implemented AFIDL or Rust. Earlier research's “WorkflowContract digest” and
“effect signature digest” language does not authorize such objects: current
accepted D3 binds the existing exported ComponentContractDigest and the
complete assertion. The current normative registry and implemented models
determine actual capability.

Direct source checks covered `root_export.rs` (manifest capture, assertion,
compiler identity, preflight, aggregate gate, checked wrapper), `resolution.rs`
(snapshot identity and builder), `control.rs` (joint Root/RootRecovery
construction), `af-contract/src/validate.rs` (context/contract checking),
`af-ir/src/subflow.rs` (target validation and dependencies), and the domain
registry. No production WorkflowVersion, PlanCore, LockGraph, publisher
authentication, or publication API exists in this baseline. Generic registered
digest domains are present; their existence does not define canonical payloads.

## Compile / Publish / Deploy / Run responsibilities

| Layer | Inputs and responsibility | Output and authority |
| --- | --- | --- |
| Compile, partly implemented | Exact IR/contracts/context/resolver; type/dataflow validation, 13-opcode closure, Recovery aggregation, nine-dimensional RootExport comparison | Process-local checked static capability and diagnostics; no run authority |
| Publish, not implemented | Full semantic baseline, fresh static proof, Plan preservation, LockGraph closure, artifact integrity, required evidence, publisher and release policy | Immutable WorkflowVersion and publication decision for the exact subject |
| Deploy, not implemented | Exact published version plus EnvironmentContract and compatible BindingSnapshot | Separate immutable DeploymentRevision / environment eligibility |
| Run Admission, not implemented | Concrete revision, principal/delegation, typed inputs/parameters, policy, quota/budget, live safety | A specific Run admission decision and immutable run baseline |
| Runtime Enforcement, not implemented | Run baseline and accepted runtime facts | Per-operation obligations, grants/handles/reservations/receipts and Worker admission |

## Required publication facts and model gaps

| Publication fact | Existing formal evidence | Missing model / dependency |
| --- | --- | --- |
| Exact program and public envelope | SemanticIrDigest, checked IR, ComponentContractDigest, checked contract | Parent artifact's typed canonical references |
| Root envelope fits export | RootExportAssertion, CheckedRootExportCompatibility | Portable schema/decoder, complete context reconstruction, fresh verifier boundary |
| Exact semantic inputs | Explicit process-local context and resolver | Durable context/catalog bundle with trusted origin and exact optional associations |
| Dependency closure | Checked targets and static direct/transitive checks | LockGraph canonical schema, package/adapter edges, eligibility and integrity rules |
| Executable semantics preserve IR | Research description only | PlanCore schema, lowering semantics, semantic preservation verifier and corpus |
| Compiler/runtime meanings | Current RootExport source/registry identity; execution profile catalogs | Full build/lowering semantics identity and formal runtime semantics profile |
| Body/implementation truth | RequiresEvidence in current report | Typed evidence predicates, exact subjects, trusted issuers, invalidation rules |
| Publisher and release authorization | No implementation | Authentication, authoritative trust roots and release policy decision model |
| Immutable published identity | ACE1 and registered artifact domain | WorkflowVersion parent schema, canonical content and logical-reference rules |

## Stage dependency graph

~~~mermaid
flowchart TD
  S["Existing static semantic proof<br/>IMPLEMENTED"] --> R["Portable proof revalidation model<br/>DESIGN ACCEPTED / NOT IMPLEMENTED"]
  R --> C["Compiler + complete semantic source identity<br/>formal portable model BLOCKING"]
  R --> X["Exact context reconstruction + approved origins<br/>BLOCKING"]
  X --> L["Resolver / LockGraph closure<br/>BLOCKING"]
  C --> P["PlanCore + lowering semantics + preservation<br/>BLOCKING"]
  L --> P
  R --> E["Body / implementation / publisher / build evidence<br/>formal predicates and trust policy BLOCKING"]
  C --> E
  L --> E
  X --> E
  P --> A["Canonical parent artifact model<br/>BLOCKING"]
  E --> A
  R --> A
  A --> W["WorkflowVersion + Publish gate<br/>NOT IMPLEMENTED"]
  W --> D["DeploymentRevision<br/>NOT IMPLEMENTED"]
  D --> N["Run Admission<br/>NOT IMPLEMENTED"]
  N --> T["Runtime Enforcement<br/>NOT IMPLEMENTED"]
~~~

The arrows denote prerequisite models, not authorization to implement them in
this phase. LockGraph needs an exact context/semantic dependency projection;
it does not wait for PlanCore. RootExport can run before Plan lowering, but
publication requires both proofs against the same final IR and closure.

## Package index

| Document | Main responsibility |
| --- | --- |
| [Static proof revalidation](phase-04c0-static-proof-revalidation.md) | A/B/C comparison, reconstruction and fresh checking |
| [Proof-carrying artifact](phase-04c0-proof-carrying-artifact.md) | Inline inputs, identity ownership and report treatment |
| [WorkflowVersion boundary](phase-04c0-workflow-version-boundary.md) | Definition/Draft/Version and canonical identity |
| [Plan / LockGraph](phase-04c0-plan-lockgraph-dependencies.md) | Closure, lowering preservation, engineering order |
| [Compiler semantics](phase-04c0-compiler-semantics-identity.md) | Content versus binary identity, version reuse, F03/F04 |
| [Body / publisher trust](phase-04c0-body-publisher-trust.md) | Evidence subjects, issuers, authenticity and conformance |
| [Publication / deployment](phase-04c0-publication-deployment-separation.md) | Gate algebra, mandatory inputs and downstream authority |
| [Parameter binding](phase-04c0-semantic-parameter-binding.md) | Semantic inputs, bindings, safety, operational config and secrets |
| [Decision vectors](phase-04c0-decision-vectors.md) | 48 design scenarios; no production Publish tests claimed |
| [Decision register](phase-04c0-decisions.md) | Seven accepted architecture choices and blocking formal models |
| [Readiness](phase-04c0-readiness.md) | Blocking inventory and proposed future stages |
| [Gate report](phase-04c0-gate-report.md) | Baseline, required checks, freeze and commit verification |
