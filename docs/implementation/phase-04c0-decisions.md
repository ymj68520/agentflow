# Phase 4C.0 — Decision Register

All chosen designs, candidate identities and future entry points are
**DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED ABI / NOT IMPLEMENTED**.
ACCEPTED denotes an architectural choice. BLOCKING denotes a formal dependency
that has no complete machine model and prevents the dependent implementation.

## DECISION-C01 — Static Proof Revalidation Strategy

- **Status:** ACCEPTED, full re-execution baseline.
- **Problem:** Transfer process-local checked static facts into publication
  without trusting a saved pointer/report.
- **Current implemented facts:** Private checked wrapper, full static checker,
  exact local context and fixed resolver identity.
- **Architecture constraints:** Explicit immutable inputs; no ambient lookup;
  static/body/publisher/run claims separate.
- **Alternatives:** A full check, B independent portable certificate, C
  selective derivation plus attestations.
- **Chosen design:** Option A reconstructs exact inputs and freshly derives Root,
  RootRecovery and all nine comparisons.
- **Rejected options:** Report renamed certificate; pointer persistence;
  positive cached subproof reuse without verifier semantics.
- **Proof obligations:** Complete decoder/context roundtrip, exact closure,
  fresh checked-value ownership, qualified checker and reproducibility.
- **Exact typed identities:** IR/export pins, complete assertion, semantic
  content, context and resolver association tuples.
- **Failure-closed conditions:** Missing inputs, unsupported semantics,
  stale content or any non-Proven static requirement blocks static admission.
- **ABI impact:** None in this phase; future portable schema must be additive,
  versioned and registered before implementation.
- **Unimplemented dependencies:** B01 portable inputs/context and B02 semantic
  content model; downstream release gate B07.
- **Residual risks:** Verifier correctness, availability/cost and untrusted
  input origins; recomputation does not establish implementation truth.
- **Evidence:** [revalidation design](phase-04c0-static-proof-revalidation.md).

## DECISION-C02 — Proof-Carrying Publication Artifact

- **Status:** ACCEPTED, inline full reproducible proof inputs.
- **Problem:** Fix every static proof dependency under an immutable parent
  without inventing a root/assertion digest.
- **Current implemented facts:** Existing IR/contract digests and structural
  assertion identity; artifact hash domain registered.
- **Architecture constraints:** D3 prospective parent ownership; no formal
  WorkflowVersion payload exists.
- **Alternatives:** Inline reproducible inputs; standalone assertion digest;
  report/signature-only artifact.
- **Chosen design:** Versioned canonical semantic parent commits the complete
  assertion/context/association inputs and typed evidence requirements.
- **Rejected options:** Standalone root/assertion digest, opaque report as proof,
  incomplete checksum-only manifest and mutable dependency references.
- **Proof obligations:** Full canonical coverage, typed schema identity,
  exact referenced content, duplicate consistency and acyclic signing subjects.
- **Exact typed identities:** Existing IR/contract pins, future valid
  Plan/LockGraph pins, complete semantic tuple and future parent artifact pin.
- **Failure-closed conditions:** Missing schema/content, conflicting redundant
  fields, integrity mismatch or missing mandatory evidence blocks Publish.
- **ABI impact:** No additions now; B06 must specify canonical payload, ABI and
  goldens before using the existing artifact domain.
- **Unimplemented dependencies:** B01–B06 exact content and evidence models.
- **Residual risks:** Diagnostic/semantic field misclassification, evidence
  circularity and unavailable referenced assets.
- **Evidence:** [proof-carrying artifact](phase-04c0-proof-carrying-artifact.md).

## DECISION-C03 — WorkflowVersion / PlanCore / LockGraph Dependency Order

- **Status:** ACCEPTED, dependency/Plan models precede version materialization.
- **Problem:** A static IR contract proof does not establish executable Plan
  correctness or durable dependency identity.
- **Current implemented facts:** Checked IR/root/static resolver; generic
  plan-core/lock-graph domains only.
- **Architecture constraints:** Plan closes semantic resolution and preserves
  all 13-opcode meanings; LockGraph preserves exact graph associations.
- **Alternatives:** Version from current static result alone; Plan first without
  fixed closure; exact closure then IR proof/lowering/preservation.
- **Chosen design:** Parameter materialization → final checked IR and closed
  LockGraph → RootExport and lowering → independent Plan preservation →
  publication evidence/parent model.
- **Rejected options:** IR digest substituted as Plan pin; flat digest list as
  LockGraph; RootExport substituted for Plan conformance.
- **Proof obligations:** Exact graph projection, acyclicity and source edges;
  formal IR/Plan trace relation; complete control/E/K/U/R/F/X preservation.
- **Exact typed identities:** IR, export, package/adapter content, direct target
  associations, LockGraph, PlanCore, lowering/runtime profiles.
- **Failure-closed conditions:** Missing Plan/Lock, altered lowering facts,
  ambient resolution, unsupported transitions or preservation failure.
- **ABI impact:** None now; future B03/B04 schemas use registered domains after
  canonical models are authorized.
- **Unimplemented dependencies:** B03 LockGraph, B04 Plan/preservation, B02
  lowerer/profile semantics.
- **Residual risks:** Declared envelope compatibility is weaker than behavioral
  preservation; optimization proof scope must be explicit.
- **Evidence:** [Plan and LockGraph](phase-04c0-plan-lockgraph-dependencies.md).

## DECISION-C04 — Compiler Semantics and Normative Content Identity

- **Status:** ACCEPTED; F03 remains OPEN/P2.
- **Problem:** Semantic source meaning, checker build and RuleId attribution
  have different identity/trust requirements.
- **Current implemented facts:** Exact 16 source bundles, complete registries,
  checker version and 84 applied RuleIds; no lowering/binary identity model.
- **Architecture constraints:** Explicit semantic versions; no ID-only
  comparison or silent cross-version proof reuse.
- **Alternatives:** Version string only; 84 IDs assumed complete; full content
  tuple plus separate build identity and explicit attribution profile.
- **Chosen design:** Full source/registry/static/lowering/runtime meanings
  pinned; qualified binary/provenance verified separately; fresh revalidation
  under exact supported historical or current semantics.
- **Rejected options:** Automatic compatibility by matching labels; rewriting
  identity fields on a saved proof; closing F03 without a justified closure.
- **Proof obligations:** Exact role coverage, support policy, deterministic
  rebuilding and rule closure before certificates/cached positive proofs or a
  rule-complete release profile.
- **Exact typed identities:** Source bytes/registered specification/rule
  content, analysis/lowering/profile versions, exact checker build.
- **Failure-closed conditions:** Unsupported/stale content or mandatory
  attribution missing blocks the corresponding profile.
- **ABI impact:** No registry changes now; B02 later defines the portable tuple.
- **Unimplemented dependencies:** B02; optional complete Rule Dependency Closure
  becomes mandatory at its specified hard triggers.
- **Residual risks:** Checker implementation correctness, archived verifier
  availability and F04 conservative guard precision.
- **Evidence:** [compiler semantics](phase-04c0-compiler-semantics-identity.md).

## DECISION-C05 — Body / Implementation / Publisher Evidence Boundary

- **Status:** ACCEPTED; trust/evidence models remain BLOCKING.
- **Problem:** Digest equality and authenticated reports cannot establish
  body or implementation conformance.
- **Current implemented facts:** Static report keeps body/implementation
  RequiresEvidence and publisher/runtime NotEstablished.
- **Architecture constraints:** Separate artifact, issuer, predicate, build,
  behavioral and release claims; exact transitive subject graph.
- **Alternatives:** Publisher signature implies conformance; arbitrary
  self-signed report; separate scoped evidence and independent verification.
- **Chosen design:** Exact body/implementation predicates and dependency
  subjects, approved scoped issuers, explicit freshness/invalidation; publisher
  statement signs exact parent and logical publication action.
- **Rejected options:** Resolved subflow export as body proof, standards
  adoption as correctness, raw secret content, publisher-provided trust root.
- **Proof obligations:** Exact subjects, predicate soundness/assumptions,
  authenticated issuer authority, complete closure, no cyclic claims and
  typed deferred implementation obligations where formally supported.
- **Exact typed identities:** Target IR/export, implementation artifacts,
  context/semantic content, evidence payloads, parent subject and issuer/policy.
- **Failure-closed conditions:** Required body evidence absent, stale subjects,
  invalid predicates or untrusted publisher/issuer.
- **ABI impact:** None now; B05/B07 require versioned evidence/authentication
  models and no signature bytes in the semantic parent.
- **Unimplemented dependencies:** B05 evidence; B07 conditional profiles and
  authoritative trust/release policy.
- **Residual risks:** Trusted issuer/verifier compromise and claims exceeding
  their explicit assumptions.
- **Evidence:** [trust boundary](phase-04c0-body-publisher-trust.md).

## DECISION-C06 — Publish / Deploy / Run Admission Separation

- **Status:** ACCEPTED; production release decisions remain NotImplemented.
- **Problem:** Static compatibility must not grant release or runtime rights.
- **Current implemented facts:** RootExport issues only a static wrapper and
  retains independent body/publisher/runtime states.
- **Architecture constraints:** Immutable version distinct from immutable
  deployment/run snapshots; authority assessed at its owning stage.
- **Alternatives:** PublishAllowed=static Proven; mutable version status;
  typed conjunction of stage-specific predicates.
- **Chosen design:** Full mandatory Publish input matrix with Admit/Reject/Block
  results; separate Deploy, Run Admission and enforcement. Missing required
  body/Plan/Lock evidence only permits internal non-executable analysis.
- **Rejected options:** Verified boolean, unknown runtime authority promoted,
  unavailable deployment treated as failed static proof, named analysis
  artifact bypassing Publish.
- **Proof obligations:** Mandatory requirement closure, exact policy/subject
  binding through publication commit, all findings retained and typed
  conditional implementation roles.
- **Exact typed identities:** Parent/version, release policy and evidence set;
  future environment/bindings/revision and specific Run principal/inputs.
- **Failure-closed conditions:** Every mandatory nonpositive/absent fact blocks
  its owning gate; known contradiction rejects; no capability from failure.
- **ABI impact:** None now; B07 specifies future policy/decision schemas.
- **Unimplemented dependencies:** All publication models B01–B07; later
  DeploymentRevision/Run admission models.
- **Residual risks:** Policy waiver loopholes and accidental reuse of a
  capability outside its exact stage/subject.
- **Evidence:** [gate and stage boundaries](phase-04c0-publication-deployment-separation.md).

## DECISION-C07 — Semantic Configuration Binding Time

- **Status:** ACCEPTED; machine parameter/binding models remain missing.
- **Problem:** Mutable ambient configuration can change a fixed semantic build.
- **Current implemented facts:** IR uses explicit typed values/RunParameters
  and constant control policies; no general config/RunSemanticParameter API.
- **Architecture constraints:** Every semantic input belongs to published,
  run-start or accepted runtime facts; no ambient fourth path.
- **Alternatives:** Environment lookup at execution; publish constants plus
  formal bounded run slots and compatible deployment bindings.
- **Chosen design:** Resolve publish semantic parameters before final proof;
  future run slots require bounded domains and immutable snapshots; separate
  deployment, operational, narrowing safety and secret mechanisms.
- **Rejected options:** Current default model/toolset/feature flag, unresolved
  publish placeholders, raw credentials and operational labels hiding semantics.
- **Proof obligations:** Semantic influence classification, domain proof,
  materialization completeness and compatibility/authority preservation.
- **Exact typed identities:** Final IR/Plan/parent content, future slot/domain
  schema and Run snapshot, exact deployment binding and security baseline.
- **Failure-closed conditions:** Unresolved/undeclared semantic input,
  unsupported/unbounded slot, widening late binding or forbidden secret.
- **ABI impact:** None now; future slots/bindings need dedicated versioned
  machine schemas before implementation.
- **Unimplemented dependencies:** B02/B04 parameter/profile semantics, B06
  parent content; later Run/Deployment/credential models.
- **Residual risks:** Misclassifying model-facing metadata or timing-sensitive
  operational changes as nonsemantic.
- **Evidence:** [parameter binding](phase-04c0-semantic-parameter-binding.md).

## Formal model blockers

| ID | Status | Missing exact model | Blocks |
| --- | --- | --- | --- |
| MODEL-B01 | BLOCKING | Portable assertion/context schemas, complete decoder, reconstruction and roundtrip/cross-language vectors | Cross-process production proof revalidation |
| MODEL-B02 | BLOCKING | Full compiler/lowering/runtime semantic content tuple and support/version rules | Plan/release semantic identity |
| MODEL-B03 | BLOCKING | LockGraph canonical schema, semantic source edges, exact package/adapter/target associations and eligibility rules | Durable closure and locked rebuilding |
| MODEL-B04 | BLOCKING | PlanCore canonical schema, lowering semantics and independent preservation relation/verifier | Executable semantic build |
| MODEL-B05 | BLOCKING | Body/implementation/build evidence subjects, predicates, dependency closure and trusted issuer scope | Required conformance/authenticity |
| MODEL-B06 | BLOCKING | WorkflowVersion semantic parent canonical schema, logical reference rules and evidence commitment encoding | Valid immutable artifact identity |
| MODEL-B07 | BLOCKING | Release policy/typed gate/authentication schemas and explicit conditional deployment implementation profile | Actual Publish admission |

None is filled with a guessed Digest value, untyped map, arbitrary signature,
default successful state or invented existing API. Their architecture direction
is accepted; exact fields/ABI/encoding/verifier implementations require separately
authorized future phases.
