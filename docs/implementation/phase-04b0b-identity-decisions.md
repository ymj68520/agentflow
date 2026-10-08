# Phase 4B.0B — Component Identity and Digest Decision

> **DESIGN PROPOSAL** · **NON-NORMATIVE** · **NOT A FROZEN ABI**

## Decision record B0B-D1 — Component lineage, release, and content identity

**Status: ACCEPTED (design only).**

### Problem statement

The contract's logical owner, a publisher's release/version coordinate, the
semantic contract bytes, an implementation artifact, and a deployment must
not collapse into one identity or digest. A digest must be stable under
metadata/version-only changes, but two unrelated component lineages must not
become substitutable merely because their declarations happen to match.

### Existing frozen semantics and wire representation

There is no ComponentContract AFIDL or Rust model. The Semantic IR contract
table pins a component by an exact `ComponentContractDigest`; architecture
§7.4 says Component Contract Version and Implementation Digest are separate,
and compatibility is a proof rather than version arithmetic. The existing
`component-contract` HashV1 domain is already reserved; this phase does not
compute a digest or add a domain.

### Candidates

* **A — lineage and release version inside the hashed body.** Rejected:
  changing a release label would change semantic bytes without changing the
  promise.
* **B — ComponentRef separate from canonical contract body.** Chosen:
  stable lineage is a semantic field; release version and digest pin live in a
  separate immutable registry/release reference.
* **C — opaque globally allocated component ID.** Rejected for v1: it adds a
  new allocation/registry authority without improving the exact namespace
  already provided by `(catalog, name)`.

### Chosen type and field proposal

```text
ComponentIdentity {
    catalog: SpecificationId
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

PackageIdentity { catalog: SpecificationId, name: PackageName }

ComponentImplementationRef {
    package: PackageIdentity
    implementation_version: ReleaseString
    artifact_digest: Sha256ArtifactDigest
    target: ComponentRef
}
```

`ComponentName` and `PackageName` are validated NFC non-empty logical keys;
ComponentName uses closed lowercase dotted segments `[a-z0-9-]+`. The package
release string is opaque metadata and is not ordered as semantic compatibility.
`Sha256ArtifactDigest` is `{algorithm="sha256", hex=<64 lowercase hex>}` over
exact implementation artifact bytes; it is not HashV1 and is not a
ComponentContract field.

Only `ComponentIdentity` is in the ComponentContract canonical body. The
version triple is a publication coordinate, distinct from the AFIDL schema
major; it is release metadata, not a compatibility ordering, and no
SemVer-major/minor inference is permitted. A release registry key
`(ComponentIdentity, ContractVersion)` is immutable and unique: republishing
different bytes under the same key is a Violation. `ComponentRef` pins that
exact published release and digest. An implementation ref pins independently
identified implementation bytes and the exact contract release it claims to
implement.

### Identity and namespace rules

`ComponentIdentity` is exact, case-sensitive `(catalog, name)` identity. The
catalog is a full SpecificationId-form logical namespace; the name is a
closed normalized component key, not a display label. Ports are local to this
identity plus direction. Display name, publisher name, package name and
implementation name are not aliases for ComponentIdentity.

### Digest boundary

The future, unexecuted formula remains:

```text
ComponentContractDigest
  = HashV1("component-contract", ACE1(CanonicalComponentContract))
```

The canonical body includes ComponentIdentity and every immutable semantic
field: I, O, Φ, ε, R, F, U, K, X, exact Flow partitions, exact Recovery action
bindings and per-action Recovery Usage profiles. Exact child TypeSchemaRefs
and their TypeSchemaDigests participate by value. Canonical order is the
schema order plus each closed list's declared identity order.

The body excludes ContractVersion, the digest field itself, implementation
reference/digest, publisher attestations, deployment/environment bindings,
region observations, credentials, prices, cost projections, budget
reservations, diagnostics, documentation, proof reports, worker profiles,
capacity and live policy. Names and descriptions that are purely diagnostic
are published in a separate documentation projection keyed by exact
ComponentIdentity and port coordinate.

`UsageSignature` is included because it is a stable component consumption
promise. `PriceTableSnapshot`, `CurrencyUniverse`, and `StaticCostProjection`
are analysis inputs/results and are excluded. Provider price changes therefore
cannot change ComponentContractDigest. A Run-local `StaticCostCeiling` is not
an intrinsic component semantic field.

### Proof direction and fail-closed cases

* Same lineage, same semantic body, different ContractVersion: same canonical
  bytes and same contract digest; release versions do not prove compatibility.
* Same lineage, any semantic field differs: different canonical bytes/digest;
  substitution still requires the independent compatibility proof.
* Different lineage: identity field differs, therefore canonical bytes and
  digest differ (apart from a cryptographic collision). Cross-lineage reuse is
  Unproven without an explicit alias/migration relation; equal field sets do
  not merge identities.
* A release reference whose digest does not match its exact canonical body is
  a Violation. “Latest”, mutable registry pointers, or missing lineage/version
  pins are Unproven/invalid, never fallback-resolved.
* Changing only ImplementationDigest leaves ComponentContractDigest
  unchanged. Whether that implementation is compatible is separately
  Unproven until static compatibility and any required attestation are
  supplied.

Release resolution is a two-step lookup: `(ComponentIdentity, ContractVersion)`
selects one immutable registry release, whose resulting `ComponentRef` pins
the exact ComponentContractDigest. Semantic IR stores the exact digest pin,
not a mutable version constraint or “latest” pointer.

### Existing AFIDL and future ABI effect

No existing AFIDL field, TypeSchemaDigest, SemanticIrDigest, SpecificationId,
or ABI assignment changes. The future candidate adds ComponentIdentity as
ComponentContract field 1; ContractVersion stays outside the body. The
ComponentContract schema may reference a ComponentRef in Semantic IR only
through the existing exact digest pin; it must not recursively embed its own
digest.

### Runtime, migration, and residual risk

Compatibility attestation binds `(ComponentRef, ComponentImplementationRef,
test/evaluation profile)` as separate evidence. A strict deployment may
require an exact implementation digest; a compatible late-binding policy may
select another implementation only after independent compatibility and
attestation checks. Neither result is generated by the contract digest.

Registry retention, publisher authentication, version allocation, revocation,
implementation-to-contract conformance, and collision handling remain
separate P3/registry obligations. This identity model neither authenticates a
publisher nor authorizes execution.
