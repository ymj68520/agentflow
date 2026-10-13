# Phase 4C.0 — Publication Gate and Downstream Boundaries

All candidate decisions, fact types and entry points are **DESIGN ONLY /
NON-NORMATIVE / NOT REGISTERED ABI / NOT IMPLEMENTED**. Decision C06 defines
distinct positive claims and failure reasons.

## Static admission relation

~~~text
StaticExportAdmissible(inputs) =
  exact supported IR/export/typed witnesses available and checked
  AND exact complete compiler semantic content supported
  AND context reconstructible, exact and valid under local checker scope
  AND exact resolver associations and dependency closure validated
  AND fresh Root/RootRecovery derived from those same inputs
  AND required static preflight and all nine dimensions Proven
~~~

This is a mathematical design predicate, not a production boolean API.
The current RootExport report's separately appended body evidence item is
evaluated by the body gate; it does not make a static compatibility result
equivalent to complete publish readiness.

## Complete Publish decision inputs

| Input predicate | Exact subject / evaluation | Missing or negative result |
| --- | --- | --- |
| StaticExportAdmissible | Full inline proof bundle; option-A fresh check | Violation/Unproven/RequiresEvidence blocks positive static fact |
| Complete semantic baseline | Materialized publish parameters, explicit bounded run slots/profile, no ambient semantic inputs | Unproven or UnsupportedSemantics |
| Semantic source integrity/support | Exact normative sources/registries/schema/profile/lowering meanings | StaleEvidence, MissingArtifact, UnsupportedSemantics |
| Context origin approval | Exact catalogs, schemas and relations from consumer-approved origins | Untrusted context / policy rejection |
| LockGraph closed and valid | All reachable component/subflow/adapter semantic edges and content associations | MissingArtifact, incomplete closure, conflicting pins |
| Plan valid and semantics preserving | Exact PlanCore linked to same IR/LockGraph/semantics | MissingArtifact, NotImplemented, preservation violation |
| Parent artifact intact | Future canonical schema and complete semantic content commitment | Missing model or digest/content conflict |
| Required body evidence | Exact root/target relations and assumptions, transitive evidence closure | RequiresEvidence or invalid/stale evidence |
| Required implementation evidence | Fixed binary predicates or formally permitted conditional binding obligations | RequiresEvidence; unresolved required fact never defaults to pass |
| Qualified build/verifier evidence | Exact checker/lowering binary and claimed build outputs if profile requires it | Untrusted/stale/missing evidence |
| Publisher authentication | Exact parent and logical publication namespace/role/action | UntrustedPublisher or missing authentication |
| Release policy authorization | Authoritative exact policy revision, required profile and evaluated evidence set | Denied, missing policy, unsupported profile |

The future PublishAllowed relation is the conjunction of **all applicable
mandatory publication predicates above**. It is not equal to
StaticExportAdmissible. A policy cannot weaken a static contradiction or
mandatory artifact model into a warning.

The candidate input set is immutable during evaluation. The eventual
publication commit must bind exactly the verified parent, evidence set and
authoritative policy/trust revision. If any of these facts changes between
evaluation and commit, re-evaluate or reject the stale decision. A successful
check against one candidate cannot publish another candidate through a mutable
alias. The transaction/decision schema remains part of MODEL-B07.

## Typed fail-closed algebra

~~~text
PublicationFact (candidate) {
  exact_typed_subject
  responsible_stage
  static_verdict: optional Proven / Violation / Unproven / RequiresEvidence
  availability: Present / MissingArtifact / NotImplemented
  authenticity: Established / Missing / UntrustedPublisher / UntrustedEvidence
  freshness: Current / StaleEvidence / UnsupportedSemantics
  prerequisites: complete typed fact references
  diagnostic: informative only
}

PublishDecision (candidate) =
  Admit(exact_subject, exact_policy, complete_evaluated_inputs)
  | Reject(known_contradictions_or_authentication_denial, all_findings)
  | Block(unproved_or_missing_prerequisites, all_findings)
~~~

No single `is_valid` or `verified` flag covers these states. Proven static
facts, MissingArtifact and NotImplemented remain different dimensions of a
decision. Reject and Block both issue no published-version capability.
Contradictions dominate the displayed reason; all missing/uncertain facts are
retained.

NotApplicable may be computed only from an explicit formal rule and exact
input closure (for example, no subflows in a known complete graph). It cannot
be a publisher-supplied way to skip mandatory body or implementation evidence.
Timeout/resource limits on verification yield an incomplete result, never pass.

RootExport Proven plus required body RequiresEvidence yields **Block**.
Missing PlanCore or LockGraph yields **Block(MissingArtifact)**.
A valid publisher signature plus invalid body proof yields **Reject**.
An unsupported new semantic release or required rule closure missing yields
Block; no current-default fallback is allowed.

## Publication grades

| Prospective artifact/result | Permitted positive claim | Rights |
| --- | --- | --- |
| Internal analysis result | Diagnostics and known static facts | No Publish/Deploy/Run capability |
| Non-executable compiled archive | Exact retained compile inputs/results under its future archive schema | No WorkflowVersion or executable admission |
| Published immutable WorkflowVersion | Every applicable mandatory publish predicate is satisfied | May be considered for Deployment; no environment or run authorization |
| Deployment eligibility / DeploymentRevision | Exact version is compatible with exact EnvironmentContract/BindingSnapshot and concrete required evidence | May be considered for a particular Run |
| Run admission | Exact principal/inputs/parameters/policy/budget/live safety at Run creation | Creates only the specific admitted Run baseline |

Publication with absent required body evidence is not a permitted lower
WorkflowVersion grade. Conditional deploy-bound implementation obligations
need the formal profile described in the trust document; they do not count as
proved implementation conformance.

## Deploy / Run / Runtime separation

A published version need not have an available deployment environment.
EnvironmentContract describes static execution/security/trust/residency/
durability/protocol constraints, not current capacity. BindingSnapshot maps
logical resource/model/identity/component/remote requirements into a closed
compatible deployment baseline. Both formal models are missing here.

Changing a deployment binding creates a new immutable DeploymentRevision,
not a mutated WorkflowVersion. A future mutable active revision pointer only
selects future Runs; an admitted Run keeps its exact baseline.

Run Admission independently checks principal/delegation, policy, typed business
input, materialized bounded RunSemanticParameters, quotas/budget and live safety.
Publisher authority is not run authority. Unknown runtime authority does not
automatically invalidate a fully satisfied Publish gate; it prevents asserting
Run/runtime authorization.

Runtime must enforce each concrete effect/resource/budget/Worker obligation.
EffectGrant, ResourceHandle, BudgetReservation, EffectReceipt and Recovery
Decision are future runtime evidence. No release proof creates these objects.

Existing Scope authority and budget attenuation remain Unproven because frozen
ScopeBody has no such ceilings. They are not promoted to Proven or interpreted
as unlimited. A future publication profile requiring those guarantees must
specify exact Plan/runtime obligations and their verifier; absent such a model
blocks that profile. A known complete runtime-obligation template can be a
publication prerequisite without already establishing concrete Run authority.

## Current decision

The architecture algebra is ACCEPTED. Current production Publish evaluation is
**NotImplemented**; all prospective publication prerequisites lack a complete
machine model. Actual publication is BLOCKING even for a valid current
CheckedRootExportCompatibility.
