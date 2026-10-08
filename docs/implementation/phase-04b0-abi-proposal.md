# ComponentContract Canonical ABI — Design Proposal

> **DESIGN PROPOSAL**
> **NON-NORMATIVE**
> **NOT A FROZEN ABI**

This proposal has no AFIDL source, SpecificationId, ABI assignments, RuleIds,
schema digests or production Rust type. It records one candidate for review
after the Phase 4B.0 blockers are resolved.

## Contract and implementation identity

Keep these values separate:

| Object | Candidate identity | In ComponentContract bytes? |
| --- | --- | --- |
| Component logical identity | Exact `catalog/name` stable ID | Yes; prevents a digest from losing the identity it promises |
| Contract version | Publisher/registry version for a logical component | No; version is a lookup/release coordinate, exact content is identified by digest |
| ComponentContractDigest | Hash of the canonical semantic contract body | It is the digest of the bytes, not a recursive body field |
| Implementation reference | Exact package/implementation selection | No; supplied by implementation binding/lock |
| Implementation digest | Digest of the exact implementation artifact under artifact/package identity | No; never substitute for the contract digest |
| Compatibility attestation | Signed evidence about an implementation/contract pair and test/eval profile | No; external evidence object references both exact digests |
| Deployment binding | Environment, resource, identity, model and implementation bindings | No; immutable DeploymentRevision/BindingSnapshot |
| Static proof evidence | Results under named universe/profile snapshots and rules | No; derived proof report, not semantic declaration or authority |
| Runtime admission | Current policy, quota, capacity, attestation and permit results | No; Run-local authority and operational state |

`ComponentIdentity` should be stable and independent of display name. A
registry/contract version may point to an exact `ComponentContractDigest`;
two versions with byte-identical semantic bodies may reuse the digest. The
contract body includes the stable identity so the digest identifies a specific
component promise. Whether that identity should be `(SpecificationId,name)` or
a separately allocated opaque ID remains a P1 review item; the proposal uses
the former because the rest of this repository already has exact
SpecificationId identities.

## Candidate port model

Inputs, outputs and resource requirements use independent 1-based port
namespaces. The stable per-contract port identity is
`(ComponentIdentity, direction, port_no)`. A port number is not reused or
rebound in a component identity line. A change to a port number is a contract
change; callers do not map by display name.

Candidate logical descriptors:

```text
ComponentInputPort  { port_no: PortNo, type: core.TypeSchemaRef }
ComponentOutputPort { port_no: PortNo, type: core.TypeSchemaRef }
```

Each public port pins a named schema or named alias through an exact
`TypeSchemaRef`; a bare built-in primitive cannot be used as a TypeSchemaRef,
so a named AFIDL alias is used for primitive/container public types. The pin
includes SpecificationId, type name and TypeSchemaDigest. Contract comparison
first resolves both pins in the same explicit TypeSchemaUniverse; the
conservative v1 proposal proves only exact pin equality. A different pin or
structure is Unproven unless a future closed subtype/variance rule explicitly
applies. A Component I/O type is not compared by local `SemanticTypeId`.

Subflow's local SemanticTypeIds are expanded for diagnostic/structural checks,
but the current Phase 3D checker preserves exact schema leaves. The Component
checker should use that same exact leaf rule; it must not claim broad
structural subtyping that Phase 3D does not implement. The Phase 3A alias
expansion rule applies to value-level typing; substitution across public
contract ports remains exact-pinned unless the ABI is explicitly revised.

Every declared input is required and every declared output is present on
successful completion. `option<T>` makes the value nullable; it does not make
the port omittable. There are no defaults, variadic ports or implicit extra
field rules in this proposal. Adding/removing a port changes the closed
interface; absent a separately specified evolution mapping, substitution is
Unproven or a Violation according to whether the concrete call domain is
known.

`diagnostic_name` is useful for validation errors and tools, but it must not
define port identity or silently affect compatibility. Keep it in a separate,
closed documentation projection keyed by `(direction, port_no)`, outside the
ComponentContract canonical body. This avoids changing semantic digest for a
rename and avoids an open `metadata: map<string,any>` escape hatch.

The Phase 3A alias rule and Phase 3D exact Subflow pin rule serve different
contexts: the former is value-level typing, while this proposal uses exact
pins for substitution proofs. Alias-versus-target remains Unproven at the
contract boundary even when a value-level assignment is legal. This boundary
must be approved and stated explicitly before freezing the ABI.

## Candidate top-level fields and local field numbers

The numbers below are proposals only. They are not registered and can change
before any schema is published.

The candidate schema family name `agentflow.component.contract@1` is also
provisional and is not a registered SpecificationId.

Candidate local field numbers for nested records are `ComponentIdentity`
`1 catalog, 2 name`; `ComponentInputPort` and `ComponentOutputPort`
`1 port_no, 2 type`. These assignments are design notes only and consume no
registered ABI number.

| Proposed field_no | Candidate field | Proposed type | Semantic commitment |
| ---: | --- | --- | --- |
| 1 | `component_identity` | `ComponentIdentity` | Stable logical identity |
| 2 | `inputs` | `list<ComponentInputPort>` | Required public input ABI |
| 3 | `outputs` | `list<ComponentOutputPort>` | Public output ABI |
| 4 | `failures` | `failure.FailureRow` | Typed business Failure ceiling |
| 5 | `effects` | `effect.EffectSignature` | Declared effect ceiling |
| 6 | `resources` | `resource.ResourceRequirementRow` | Logical external resource requirements |
| 7 | `information_flow` | `flow.InformationFlowContract` | Flow restrictions, may-influence summary and ceilings |
| 8 | `usage` | `usage.UsageSignature` | Stable forward/recovery usage upper bounds |
| 9 | `recovery` | `recovery.RecoveryContract` | Replay/reconciliation/compensation obligations |
| 10 | `execution` | `execution.ExecutionRequirements` | Static hard constraints and soft preferences |

Proposed port-local fields are `1 port_no` and `2 type: core.TypeSchemaRef`;
the numbered values are local to each proposed record and are not registered.

Proposed closed imports are exactly the existing semantic core, failure,
effect, resource, information-flow, usage, recovery and execution
specifications. Cost Projection is not imported into the contract body.

Canonical order: the top-level record follows the closed schema; input,
output and R lists sort by their own port number; failure/effect/resource/F/U/K/X
subobjects use their already frozen canonicalizers. The type DAG must be
canonicalized so local table order cannot change bytes. No input list may be
silently re-numbered to repair a bad reference.

## Cost and budget placement

* **UsageSignature** is the component's stable resource-consumption promise
  and belongs in the semantic contract body.
* **PriceTableSnapshot** is an explicit immutable analysis input. Provider
  pricing may change without changing component semantics.
* **StaticCostProjection** is derived from U and a price snapshot; it is a
  proof result, never a contract field.
* **CurrencyUniverse** is an explicit analysis universe, not semantic body
  content.
* **StaticCostCeiling** is best treated as a Run-local budget constraint,
  outside ComponentContract. `check_static_budget` does not reserve money,
  quota or capacity. A future publisher cost SLA would need its own exact
  semantics and must not be smuggled in as a price-snapshot-derived field.

## Diagnostic, publisher and runtime fields excluded from the body

Do not put the following in canonical ComponentContract bytes:

* port display names, descriptions, docs, examples, tags, visual/editor data,
  source spans or publisher profile metadata;
* implementation reference/digest, SBOM, build provenance, tests or
  Compatibility Attestation;
* deployment bindings, environment instance, endpoint, worker, region
  observation, runtime load, current price or capacity;
* proof outputs, profile match results, policy/approval results, credential,
  lease, ResourceHandle, EffectGrant, reservation or runtime evidence.

If any such value changes a semantic guarantee, it must be moved into a
formally named semantic field with explicit normalization and compatibility
rules before schema freeze.

## Digest input

The future formula remains the already frozen domain and is **not executed in
Phase 4B.0**:

```text
ComponentContractDigest
= HashV1("component-contract", ACE1(CanonicalComponentContract))
```

The body must pin exact child TypeSchemaRefs and must not contain latest
references. It must include each semantic fact once: no duplicated port type,
parallel resource binding table that repeats an authoritative R row, or
precomputed cost projection. The component identity is included; registry
version, implementation digest, deployment binding, pricing snapshot and
proof/attestation are excluded. `SemanticIrDigest`, `ComponentContractDigest`
and `ComponentImplementationDigest` remain distinct identities.

Before registration, resolve the P0 qualifier collision, Flow selector grain,
Recovery action identities, public port/type policy, and catalog relations
listed in the [audit](phase-04b0-preassembly-audit.md). The proposal cannot
make those missing proofs true by assigning field numbers.

## Related audit documents

[Field matrix](phase-04b0-field-matrix.md) ·
[Port namespaces](phase-04b0-port-namespace-matrix.md) ·
[Compatibility](phase-04b0-compatibility-matrix.md) ·
[Cross-dimension obligations](phase-04b0-cross-dimension-obligations.md) ·
[Counterexamples](phase-04b0-counterexamples.md) ·
[Decisions](phase-04b0-decisions.md)
