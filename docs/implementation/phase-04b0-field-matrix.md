# Phase 4B.0 — Candidate ComponentContract Field Matrix

This is an audit/design matrix, not a wire specification. `YES` under
“canonical bytes” means the field is a candidate semantic commitment in the
proposal; it does not authorize a schema to be registered. “No” means the
value is metadata, an analysis input/result, or runtime authority and must not
silently enter the future ComponentContract digest.

| Candidate field | Normative source / existing AFIDL | Existing Rust model | Semantics and identity | Universe / canonical order | Contract commitment / bytes | Validation now | Dependencies and open proof |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Component identity | Architecture §13.4, §22.3; no ComponentContract AFIDL | None | Stable logical component identity is distinct from contract version, digest, implementation, deployment and evidence | Registry identity; scalar | Candidate: include stable `catalog/name` in contract body; logical version stays registry metadata; digest pins exact body | None | Decide if identical semantic contracts under two component IDs share a digest; see ABI proposal |
| Input interface I | Architecture §7.4, §22.1; semantic-core `SemanticTypeNode`, semantic-ir `RunParameterDecl` and `InputBinding` | `SemanticTypeNode`, `StructuralType`, `RunParameterDecl`, `InputBinding` | Semantic IR IDs are local; public input identity must be component + input direction + positive port number; diagnostic name is not identity | Exact TypeSchemaRef universe for public pins; port ascending | YES, candidate; input type pins and ports canonicalized by number | Type DAG/schema pin validation exists; no Component input checker | Need I_old <: I_new proof; current implementation has exact structural equality only, no subtype/variance. Alias-vs-target policy differs between Phase 3A and 3D |
| Output interface O | Architecture §7.4, §22.1; semantic-ir `WorkflowOutputDecl`, `ComputationOutputBinding` | `StructuralType`, `WorkflowOutputDecl`, `ComputationOutputBinding` | Workflow output name and local expression are not Component output identity; public output uses component + output direction + port | Exact TypeSchemaRef universe; port ascending | YES, candidate; output types and ports are semantic | Expression/type and computation interface checks exist; no Component output checker | Need O_new <: O_old; no general subtype API. Port additions/removals and alias-vs-target policy need a frozen rule |
| Failure row Φ | failure-row schema / `FailureRow` | `FailureRow`, `FailureTypeRef` | Closed set of exact `(SpecificationId, TypeName, TypeSchemaDigest)` business failures; empty means no failure type declared, not success | `TypeSchemaUniverse`; nominal identity order | YES | Canonicalization, pin resolution, exact containment and control helpers exist | Invoke/Subflow source row is absent until complete target contract is resolved; no empty-row fallback |
| Effect signature ε | effect-signature schema / `EffectSignature` | `EffectSignature`, `EffectAtom`, `EffectOperationRef`, `ResourceSelector`, `AtomKey` | Declared effect ceiling; atom is operation + selector + qualifiers; declaration is not grant | `EffectCatalogUniverse`; canonical atom/scope order | YES | Canonical ACE1, catalog resolution, algebra, control helpers exist | Must link selector to R and input guard to I; Flow currently loses selector grain; P0 AtomKey qualifier-value collision |
| Resource row R | resource-requirement schema / `ResourceRequirementRow` | `ResourceRequirementRow`, `ResourceTypeRef` | External logical needs by contract-local R port; exact type pin and exact attributes; no handle/authorization | `ResourceTypeUniverse`; ports ascending, attributes by normalized key | YES | Exact type recognition, candidate predicate, non-strengthening per same namespace | ε selector link is only numeric; no operation/resource compatibility or explicit cross-contract port mapping |
| Information flow F | information-flow schema / `InformationFlowContract` | F model, `FlowCatalogs`, `FlowProof` | Restrictions, aggregate may-influence summary and per-kind sink ceilings; unknown is sticky; declaration is not observed flow or authorization | Label/taint and effect-sink universes; canonical identity order | YES | Canonicalization, propagation, `check_flow`, non-relaxation exist | Source ports are not resolved against I/R; output ports not resolved against O; effect sink only names operation, not ResourceSelector |
| Usage U | usage-signature schema / `UsageSignature` | `UsageSignature`, meters and static envelopes | Stable additive upper bounds, split into forward/recovery; missing is not finite zero | `UsageMeterUniverse` with unit and aggregation descriptors; phase then meter order | YES | Validation, algebra, control helpers, ceiling and non-increase proofs exist | K recovery actions have no usage profile/link; INVOKE/CALL_SUBFLOW remain unknown without contract |
| Cost / Budget | cost-projection schema / `PriceTableSnapshot`, `StaticCostCeiling` | `PriceTableSnapshot`, `StaticCostProjection`, `CurrencyUniverse` | A cost projection is a result under a fixed pricing assumption; a ceiling is a budget constraint, not actual billing | Currency, meter, fixed price snapshot universes; entries canonicalized | Price snapshot/projection: NO. StaticCostCeiling: recommend Run-local budget input, NO in ComponentContract | Exact rational pricing, projection and budget proof exist | Cost non-increase requires identical complete snapshots. No real reservation; changing prices must not change ComponentContract identity |
| Recovery K | recovery-contract schema / `RecoveryContract` | `RecoveryContract`, `EffectRecoveryClause`, `RecoveryProof` | Replay/idempotency declarations and recovery obligations; not evidence or executable compensation | `RecoveryOperationUniverse` + EffectCatalog; clauses sorted by EffectAtom key | YES, candidate | Exact forward-clause coverage for closed ε; safety yields Proven/RequiresEvidence/Unproven/Violation | Compensation ref only operation-level; reconciliation is only required/none; no recovery-action ε/U envelope |
| Execution X | execution-requirements schema / `ExecutionRequirements` | X model and separate universes/profile | Static eligibility requirements, hard constraints and non-eligibility preferences | Explicit capability/trust/region/runtime universes; canonical closed ordering | YES | Canonical ACE1, descriptor matching, feasibility/security compatibility | No attestation, admission or actual placement. F↔X classification relation absent; no blind Control-tree aggregate |
| Implementation reference/digest | Architecture §13.4; no contract AFIDL | None | Selects concrete implementation and identifies its bytes; not the Component semantic promise | Implementation/package registry and content digest | NO | None in static contracts | Must be pinned/selected by implementation binding or attestation. Never substitute ComponentContractDigest or SemanticIrDigest |
| Compatibility attestation | Architecture §7.4, §12.6, §13.3 | None | Evidence from publisher identity, contract tests and eval gates for a particular implementation/contract pair | Evidence/signature trust store | NO | No API issues or verifies one | Static proofs are conditional; attestation remains separate evidence |
| Deployment binding / environment | Architecture §17.4–17.7, §22.3 | None | Maps logical resources, identity, implementation and environment to a deployable snapshot | DeploymentRevision/BindingSnapshot and Environment | NO | No binding exists in these crates | R/X/resource access and live region/trust evidence cannot be inferred from the contract alone |
| Runtime admission / grants / leases | Architecture §8, §10, §22.9–22.10 | None | Current authority, worker admission, capacity reservation, enforcement and leases | Runtime policy and coordination authorities | NO | Deliberately absent | Not static contract semantics; static proof must never be presented as authorization |
| Static proof record | Architecture §14.6 / §22.17; no schema | Per-dimension proof enums only | Result is conditional on immutable declarations, catalog snapshots and named rules | Exact analysis input snapshots; not globally ordered | NO | Individual APIs exist; no combined result | Future proof output should retain dimension, assumption/snapshot, result and reason; it is not an attestation |

## Canonical-content recommendation

Include only the immutable semantic declaration: component logical identity,
public I/O ABI, Φ, ε, R, F, U, K and X. A canonical contract digest must not
depend on publisher display metadata, diagnostic names, documentation, source
spans, visual layout, implementation bytes, deployment region/endpoint,
price table, derived cost projection, attestation, admission result, worker
profile observation, capacity or live policy. The exact proposed identity and
port record shapes are in the [ABI proposal](phase-04b0-abi-proposal.md).

`PriceTableSnapshot` is by-value and immutable for one analysis, but it is an
analysis assumption, not an intrinsic property of a component. A price update
may change a static projection without changing the component's semantic
contract digest. `StaticCostCeiling` is best treated as a Run-local budget
constraint; checking it still does not reserve funds or quota.

## Related audit documents

[Audit report](phase-04b0-preassembly-audit.md) ·
[Port namespaces](phase-04b0-port-namespace-matrix.md) ·
[Compatibility](phase-04b0-compatibility-matrix.md) ·
[Cross-dimension obligations](phase-04b0-cross-dimension-obligations.md) ·
[Counterexamples](phase-04b0-counterexamples.md) ·
[ABI proposal](phase-04b0-abi-proposal.md) ·
[Decisions](phase-04b0-decisions.md)
