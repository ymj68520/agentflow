# ComponentContract Canonical ABI — Phase 4B.0B Candidate

> **DESIGN PROPOSAL** · **NON-NORMATIVE** · **NOT A FROZEN ABI**

This proposal is a candidate for the next materialization phase. It creates no
AFIDL file, registered SpecificationId, ABI assignment, RuleId, production
Rust type, digest value, or runtime behavior in Phase 4B.0B. All field/case
numbers below are local design numbers only.

## 1. Identity and release references

The semantic contract and the release/implementation handles are separate:

```text
alias ComponentCatalogId = string        // validator requires SpecificationId
alias ComponentName = string              // NFC lowercase dotted key
alias PackageName = string                // NFC lowercase dotted key
alias ReleaseString = string              // non-empty NFC, opaque metadata
alias LowercaseHex64 = string

ComponentIdentity {
    catalog: ComponentCatalogId
    name: ComponentName
}

ContractVersion {
    major: uint32
    minor: uint32
    patch: uint32
}

ComponentRef {
    identity: ComponentIdentity
    version: ContractVersion
    contract_digest: DigestRefValue
}

PackageIdentity {
    catalog: ComponentCatalogId
    name: PackageName
}

ComponentImplementationRef {
    package: PackageIdentity
    implementation_version: ReleaseString
    artifact_digest: Sha256ArtifactDigest
    target: ComponentRef
}

Sha256ArtifactDigest { 1 algorithm: string, 2 hex: LowercaseHex64 }
CatalogContentChecksum { 1 algorithm: string, 2 hex: LowercaseHex64 }
```

`ContractVersion` is a release coordinate only; numeric ordering is not a
compatibility proof. A registry may publish a later version with the same
semantic body and reuse its digest. The canonical body includes
`ComponentIdentity`, so different lineages do not intentionally share a
semantic digest. The identity is exact and case-sensitive; display name is
not identity. `ComponentName` and `PackageName` are non-empty NFC keys using
closed lowercase dotted segments `[a-z0-9-]+`. The version triple is an exact
release coordinate (all uint32 values, including zero, are representable);
no ordering or compatibility meaning is inferred from its numeric
components. `ReleaseString` is non-empty NFC metadata, never a semantic
version relation.

The digest records require `algorithm == "sha256"` and exactly 64 lowercase
hex digits. `Sha256ArtifactDigest` covers implementation artifact bytes;
`CatalogContentChecksum` covers a relation catalog's canonical ACE1 instance
bytes. Both are ordinary checksum metadata, not HashV1 digests and not
interchangeable with `DigestRefValue`. `ComponentRef.contract_digest` uses
`DigestRefValue` with `algorithm="sha256"` and
`domain="component-contract"`. `ComponentCatalogId` validation uses the
existing SpecificationId grammar; it adds no registry entry.

Candidate local record field numbers (all unregistered): `ComponentIdentity`
uses `1 catalog, 2 name`; `ContractVersion` uses `1 major, 2 minor, 3 patch`;
`ComponentRef` uses `1 identity, 2 version, 3 contract_digest`;
`PackageIdentity` uses `1 catalog, 2 name`; and
`ComponentImplementationRef` uses `1 package, 2 implementation_version,
3 artifact_digest, 4 target`. Each digest record uses `1 algorithm, 2 hex`.

Implementation identity/version/artifact digest live in an implementation
registry, lock, or attestation. `Sha256ArtifactDigest` is not a HashV1
ComponentContractDigest. `CatalogContentChecksum` is only a checksum of a
canonical relation-catalog instance, not an implementation digest or
HashV1 domain. One contract may have multiple compatible
implementations; replacing implementation bytes does not mutate contract
semantics, but requires a separate compatibility/attestation check.

## 2. ComponentContract@1 candidate

Provisional and unregistered schema identity: `agentflow.component.contract@1`.
Candidate imports are the exact existing semantic-core, failure-row,
effect-signature, resource-requirement, information-flow, usage-signature,
recovery-contract, and execution-requirements modules. Cost Projection is
not imported. The relation catalogs in §5 are explicit analysis inputs and
are not imported into the canonical ComponentContract body.

| Candidate field_no | Candidate field | Proposed type | Semantic commitment |
| ---: | --- | --- | --- |
| 1 | `component_identity` | `ComponentIdentity` | Stable lineage; enters digest |
| 2 | `inputs` | `list<ComponentInputPort>` | Closed required public I namespace |
| 3 | `outputs` | `list<ComponentOutputPort>` | Closed public O namespace |
| 4 | `failures` | `failure.FailureRow` | Exact nominal Φ |
| 5 | `effects` | `effect.EffectSignature` | ε ceiling; full AtomKey from 4B.0A |
| 6 | `resources` | `resource.ResourceRequirementRow` | Closed logical R requirements |
| 7 | `information_flow` | `flow.InformationFlowContract` | Existing F restrictions, summary and coarse ceilings |
| 8 | `usage` | `usage.UsageSignature` | Existing forward/recovery U ceilings |
| 9 | `recovery` | `recovery.RecoveryContract` | Existing K obligations |
| 10 | `execution` | `execution.ExecutionRequirements` | Existing X hard constraints/preferences |
| 11 | `effect_flow_partitions` | `list<EffectFlowPartition>` | Exact selector/qualifier refinement of old operation-level F ceilings |
| 12 | `recovery_action_bindings` | `list<RecoveryActionBinding>` | Exact reconciliation/compensation Effect identities |
| 13 | `recovery_usage_bindings` | `list<RecoveryUsageBinding>` | Per-action recovery usage upper bounds and multiplicities |

Candidate local fields:

```text
ComponentIdentity       { 1 catalog, 2 name }
ComponentInputPort      { 1 port_no, 2 type: core.TypeSchemaRef }
ComponentOutputPort     { 1 port_no, 2 type: core.TypeSchemaRef }

EffectFlowPartition {
    1 operation: effect.EffectOperationRef
    2 kind: flow.FlowKind
    3 allowed_effects: list<effect.EffectAtom>
}

enum RecoveryActionKind { 1 reconciliation; 2 compensation }
RecoveryActionIdentity {
    1 forward_effect: effect.EffectAtom
    2 action_kind: RecoveryActionKind
    3 action_effect: effect.EffectAtom
}
RecoveryActionBinding { 1 identity: RecoveryActionIdentity }

RecoveryUsageBinding {
    1 action: RecoveryActionIdentity
    2 activations_per_forward: effect.OccurrenceBounds
    3 attempts_per_activation: effect.OccurrenceBounds
    4 per_attempt_usage: usage.UsageSignature
}
```

`RecoveryUsageBinding.per_attempt_usage` is restricted to the `recovery`
phase. A separate exact RecoveryUsageCoverageCatalog is required to prove that
its meter list is complete for the exact action identity. Contract fields
contain explicit upper bounds; missing fields are not defaults.

## 3. Port and alias rules

I, O and R are independent, 1-based, duplicate-free namespaces. Public port
identity is `(ComponentIdentity, direction, port_no)`. Port numbers are never
silently rebound/reused; renumbering is a semantic change. Diagnostic names
and descriptions are kept in a separate closed documentation projection and
are excluded from canonical bytes.

Every I/O port is required; `option<T>` permits a None value but not an omitted
port. No defaults, variadic ports or implicit unknown-output handling exist.
V1 general substitution proves I/O only for exactly equal port sets and exact
TypeSchemaRefs. New required input/removal of an old required input and
removal of an old output are violations when the closed old call surface is
known. Other set or type differences are Unproven; no port-name remapping is
inferred.

Public alias comparison is exact nominal pin equality. A TypeSchemaRef
identity is `(SpecificationId, TypeName, TypeSchemaDigest)`. Phase 3A
structural alias expansion remains for internal pure value typing, not
Component substitution. Different aliases with the same target, alias versus
primitive, and same alias name at a different digest are Unproven.

## 4. Canonical content and digest scope

The unexecuted future formula is:

```text
ComponentContractDigest
  = HashV1("component-contract", ACE1(CanonicalComponentContract))
```

It uses the already reserved domain; no new HashV1 domain is proposed. The
body includes ComponentIdentity, all 13 semantic fields, exact child pins,
all action/flow bindings, their bounds and per-attempt usage. Inputs and
outputs sort by port number; existing rows use their frozen canonicalizers;
Flow partitions sort by `(operation, FlowKind)` and then full AtomKey; Recovery
action/usage bindings sort by full RecoveryActionIdentity. The complete
EffectAtom identity remains operation + full ResourceSelector + complete
qualifier family/member/payload.

Exact canonical keys for the new records are: ComponentIdentity by catalog
then name; ports by ascending port number; Flow partitions by operation then
FlowKind; allowed EffectAtoms by full AtomKey; Recovery action/usage rows by
the complete RecoveryActionIdentity tuple; resource relation operations and
types by their complete nominal pins, selector keys by NFC UTF-8 key, and
qualifier profiles by their sorted full typed qualifier set; coverage rows by
action identity and meter by `(catalog,name)`; Flow/Execution rows by exact
label/taint/authority/sink identity, with region/trust sets sorted by complete
catalog/name. Duplicates are rejected; no map iteration order is semantic.

Exclude ContractVersion, the digest field itself, implementation/package
identity/digest, Compatibility Attestation, DeploymentRevision,
Environment/Worker, live region observations, credentials, price snapshots,
cost projections, budget reservations, diagnostics/documentation, proof
reports, profile-match results, capacity, live policy and runtime evidence.
`UsageSignature` is semantic and included; mutable provider pricing is not.
`StaticCostCeiling` remains a Run-local budget input, not a component promise.

The three relation-catalog snapshots described below are explicit proof
assumptions, not contract fields. Their exact snapshot identities/checksums
are recorded in proof output. Changing a relation snapshot may change a proof
without changing ComponentContractDigest; comparisons across different
snapshots are Unproven unless a separate evolution relation is defined.

## 5. Required additive relation schemas

The following schemas are proposed as standalone versioned analysis catalogs.
Their IDs are provisional and must not be registered in Phase 4B.0B. They
use ordinary TypeSchemaDigest under the existing `type-schema` domain; their
instance checksums are plain SHA-256 of canonical catalog bytes and are not
HashV1 domains.

### 5.1 Operation/resource support catalog

Provisional schema: `agentflow.effect.resource.compatibility@1`.

```text
variant ScopeValueRule {
    1 finite_exact_values: list<string>
    2 opaque_nfc_exact_string: unit
}
record SelectorKeyRule {
    1 selector_key: string
    2 resource_attribute_key: string
    3 required: bool
    4 values: ScopeValueRule
}
record SupportedEffectResourcePair {
    1 operation: effect.EffectOperationRef
    2 resource_type: resource.ResourceTypeRef
    3 selector_keys: list<SelectorKeyRule>
    4 allowed_qualifier_profiles: list<EffectQualifierProfile>
}
record EffectQualifierProfile {
    1 qualifiers: list<effect.EffectQualifier>
}
record EffectResourceCompatibilityCatalog {
    1 complete_operations: list<effect.EffectOperationRef>
    2 supported_pairs: list<SupportedEffectResourcePair>
}
```

The closed `complete_operations` set distinguishes exhaustive denial from
missing evidence. Relation identity includes exact operation catalog/name
and exact ResourceTypeRef. `allowed_qualifier_profiles` is an exhaustive set
of complete qualifier lists; an empty list inside one profile explicitly
allows an unqualified EffectAtom. Selector scope is exact NFC literal matching only;
no prefixes, globs, provider names or same-port proof. A binding to a resource
attribute key is explicit. The catalog does not alter R's exact candidate
predicate and does not grant a ResourceHandle or EffectGrant.

### 5.2 Recovery usage coverage catalog

Provisional schema: `agentflow.recovery.usage.coverage@1`.

```text
record RecoveryUsageCoverageRule {
    1 action: component.RecoveryActionIdentity
    2 complete_meters: list<usage.UsageMeterRef>
}
record RecoveryUsageCoverageCatalog {
    1 rules: list<RecoveryUsageCoverageRule>
}
```

This catalog imports the exact ComponentContract action identity type and
therefore has a one-way schema dependency. A rule is exhaustive for its
exact action identity; an explicit empty set is different from a missing rule.
Each contract per-attempt profile must contain exactly the declared meter
coordinates, with explicit finite zero where applicable. The separate
UsageMeterUniverse still resolves units and additive aggregation.

### 5.3 Information-flow/execution relation catalog

Provisional schema: `agentflow.flow.execution.relation@1`.

```text
record RegionPair {
    1 execution_region: execution.RegionRef
    2 residency_region: execution.RegionRef
}
variant RegionPairPolicy {
    1 finite_exact_pairs: list<RegionPair>
    2 all_pairs_in_closed_region_snapshot: unit
}
variant TrustDomainPolicy {
    1 exact_allowlist: list<execution.TrustDomainRef>
    2 all_domains_in_closed_trust_snapshot: unit
    3 no_additional_trust_requirement: unit
}
variant SandboxFloor {
    1 no_additional_minimum: unit
    2 minimum: execution.SandboxLevel
}
record ConfidentialityExecutionRule {
    1 label: flow.ConfidentialityLabelRef
    2 region_pairs: RegionPairPolicy
}
record TaintExecutionRule {
    1 taint: flow.TaintRef
    2 trust_domains: TrustDomainPolicy
    3 sandbox: SandboxFloor
}
record AuthorityExecutionRule {
    1 authority: flow.InstructionAuthority
    2 trust_domains: TrustDomainPolicy
    3 sandbox: SandboxFloor
}
variant FlowExecutionSink {
    1 workflow_output: uint32
    2 exact_effect: effect.EffectAtom
}
record FlowExecutionRule {
    1 kind: flow.FlowKind
    2 sink: FlowExecutionSink
    3 trust_domains: TrustDomainPolicy
    4 sandbox: SandboxFloor
}
record InformationFlowExecutionCatalog {
    1 closed_regions: list<execution.RegionRef>
    2 closed_trust_domains: list<execution.TrustDomainRef>
    3 confidentiality_rules: list<ConfidentialityExecutionRule>
    4 taint_rules: list<TaintExecutionRule>
    5 authority_rules: list<AuthorityExecutionRule>
    6 sink_rules: list<FlowExecutionRule>
}
```

There is no operation-name-only sink key. `all_*_in_closed_*_snapshot` is a
typed, closed policy over the exact snapshot, not a latest or ambient
wildcard. The two closed identity sets are fields of this exact catalog
snapshot; an X reference outside them is unrecognized and remains Unproven.
Adding a region/trust domain requires a new catalog revision. The catalog is
injected into analysis; changing it causes
revalidation. It is not an execution authorization.

## 6. Analysis inputs and proof output

The future analysis context is a closed typed record of explicit snapshots:
TypeSchemaUniverse, EffectCatalogUniverse, ResourceTypeUniverse,
UsageMeterUniverse, the three relation catalogs above, InformationLabel/
EffectSink catalogs, and ExecutionUniverses. Each relation snapshot is
identified by exact schema/catalog identity, immutable revision and
canonical-content checksum. The checker verifies canonical bytes/checksum
consistency but cannot authenticate the snapshot publisher.

Proof output records these pins and per-dimension `Proven`, `Violation`,
`Unproven`, or `RequiresEvidence` results. Static assumptions, descriptor
matches, worker attestation and runtime authorization stay separate. See the
[proof API](phase-04b0b-proof-api.md) and [decision vectors](phase-04b0b-decision-vectors.md).

## 7. Alternatives and migration

No existing AFIDL module is revised. P1-5, P1-6 and P1-7 use additive
ComponentContract-level typed bindings rather than changing the old F/K/U
wire layouts. P1-4 and P1-8 use exact versioned analysis catalogs. There is
no open `extensions`, `map<string,string>`, `Any`, latest pointer, or implicit
port/type/region relation. Old contracts remain parseable in their existing
schemas but cannot claim new fine-grained proofs without the new Component
bindings.

Candidate module names and field/case numbers above are design-only. Phase
4B.1 must register new SpecificationIds/ABI rows only after reviewing this
proposal, generating conformance vectors, and confirming all frozen surfaces
remain unchanged. No Phase 4B.0C is needed if the relation catalogs and
ComponentContract are materialized together as the planned additive schema
set. No schema has been materialized here.
