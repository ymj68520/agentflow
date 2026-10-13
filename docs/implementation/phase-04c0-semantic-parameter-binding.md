# Phase 4C.0 — Semantic Configuration Binding Time

All prospective parameter/binding/secret models are **DESIGN ONLY /
NON-NORMATIVE / NOT REGISTERED ABI / NOT IMPLEMENTED**. Decision C07 adopts
architecture sections 17.6, 22.20 and 24.16's explicit semantic-input boundary.

## Classification

| Class | Binding time | Identity owner and visibility | Changes and authority |
| --- | --- | --- | --- |
| Workflow Input | Run creation | Typed business input schema is published; actual values/provenance belong to future Run baseline | New Run input; no mutation of published semantics |
| Publish-time Semantic Parameter | Before final IR/Plan validation and Publish | Ordinary canonical constants in final semantic core | Change requires new proof/build/version content |
| RunSemanticParameter | Run creation, only when explicitly declared | Published bounded slot/domain; immutable per-Run snapshot | May select/narrow declared behavior only; no hidden widening |
| Deployment Binding | Deploy | Exact version contract mapped to immutable BindingSnapshot/Revision | New compatible revision; existing Run does not drift |
| Operational Config | Bootstrap/live implementation tuning | Outside semantic identity only with justified observational equivalence | Cannot alter workflow-visible semantics or authority |
| Live Safety Overlay | Live safety decision | Separate narrowing policy/evidence and accepted runtime facts | May deny/narrow authority; cannot expand declarations or permissions |
| Secret / Credential | Broker/execution boundary | Typed reference/binding outside ordinary semantic Values; material stays secret | Rotation preserves logical subject/authority or requires a new identity baseline |

Current Semantic IR RunParameters are business inputs. They do not implement
the prospective RunSemanticParameter snapshot or binding slot model.

## Semantic examples

| Input | Selected binding rule |
| --- | --- |
| Retry count / timeout policy | Materialize exact publish constants in current supported IR; a future Run slot needs a bounded-domain proof and formal schema |
| Prompt template / tool contract set | Exact semantic content and dependency closure at Publish |
| Model selection constraints | Publish the allowed semantic/behavioral contract and profile; deployment may select only compatible bindings |
| “Current default model” | Cannot resolve during proof/execute as ambient semantics; materialize or use a formally declared constrained binding |
| Semantic feature flag | Resolve at Publish, or a formally supported bounded Run-start slot; never live-read current value |
| Release routing flag | Select a concrete future DeploymentRevision once at admission; it is not program behavior after admission |
| Operational flag / batching / cache path | Nonsemantic only while the allowed observed trace relation remains unchanged |
| Safety kill switch | Narrows execution authority; its action is a future canonical safety fact, not an invisible new program input |
| Raw secret token | Forbidden in ordinary publication artifacts, Plan constants, input Value or diagnostic archive |

Presentation is determined by semantic influence, not by a file's name.
Model-facing descriptions/tool metadata that affect allowed behavior are
semantic dependencies and must be pinned.

## Materialization and holes

Authoring Draft may contain unresolved parameters/macros/defaults. Before
forming the final proof tuple, resolve all Publish-time semantic values,
validate type/range constraints and materialize them into canonical ordinary
content. Record parameter provenance only as appropriate nonsecret evidence.
No unresolved placeholder or future default lookup remains in published
semantic content.

Normalization, parameter resolution and final dependency selection precede
RootExport and Plan lowering. Any subsequent parameter change invalidates their
proofs and parent content. A logical WorkflowDefinition name does not absorb
that change.

A future RunSemanticParameter domain must be statically bounded and included
in proof inputs. The static envelope must hold for every permitted value.
Admission validates and materializes the actual value once; it cannot add new
tools/packages, arbitrary code, resource requirements or effects. If the
compiler cannot prove the domain relation, it is Unproven. Present IR/ABI must
not be extended by an informal placeholder to simulate this model.

## Ambient reads and late bindings

`env()`, `ctx.config.get()`, `current feature flag` and `current default model`
cannot change fixed semantic behavior without entering the appropriate explicit
baseline. Pure compiler analysis accepts only explicit immutable inputs.
OS environment variables may wire a process or configure proven nonsemantic
implementation behavior, but are not semantic inputs for proof.

Physical Worker/endpoint/replica/credential selection may be compatible late
binding only inside an already fixed logical identity, contract, trust,
residency, authority and behavior envelope. An implementation switch that
changes that envelope is a semantic/binding change, not infrastructure tuning.

## Secrets and safety

The future model distinguishes SecretRef, CredentialBinding and
CredentialLease. Their schema and permissions remain missing. Raw credential
bytes never become ordinary parent/Plan/Value content. Secret material rotation
that changes neither logical principal nor authority does not change semantic
identity; changed principal/delegation is an identity binding change.

Live safety narrows permissions or prevents an operation; it cannot grant a
new effect or tool, widen a run parameter domain or raise a published ceiling.
Future runtime evidence must expose safety actions in the canonical
execution/security record. No Safety Overlay implementation is added.

## Failure closure

Unresolved semantic holes, unknown classifier, unbounded Run slot, undeclared
tool/model lookup or inconsistent parameter provenance blocks the build/publish
baseline. A detected raw secret in ordinary content is a known policy
violation. Calling a semantic field “operational” does not waive the required
identity change.
