# Execution Requirements (X)

Status: normative Phase 4A.7 static contract.

Execution Requirements describe the static conditions under which one
computation or attempt may be considered eligible against an explicitly
provided executor profile. X is a declaration and proof surface only. It does
not create a worker, placement decision, admission, lease, reservation,
credential, runtime authority, or execution evidence.

## 1. Static boundary

`ExecutionRequirements` is a closed AFIDL value. Its hard constraints are
`capabilities`, `quantities`, `sandbox`, `trust_domain`, `region_residency`,
`runtime_profile`, and `implementation_abis`. `soft_preferences` is a separate
closed list and never participates in eligibility.

<a id="execution-static-boundary"></a>
**MUST AF-EXECUTION-0001:** X MUST remain a pure
static declaration/proof layer. It MUST NOT contain WorkerId, endpoint, lease,
credential, heartbeat, live load, live price, reservation, scheduler state,
placement result, or a HashV1 digest.

## 2. Hard and soft constraints

Every hard constraint is conjunctive. A known hard violation is `Rejected`; a
missing fact or an unsupported proof is `Unproven`; only a complete proof
against the supplied descriptor is `ProvenAgainstDescriptor`. A preference
cannot compensate for a hard failure.

<a id="execution-hard-soft-separation"></a>
**MUST AF-EXECUTION-0002:** soft preferences
MUST be ignored by the eligibility predicate. They may only be consumed by a
future Scheduler after all hard constraints have been proven.

## 3. Capability identity and version

`ExecutionCapabilityRef` is the pair `(catalog, name)`. The catalog is a full
`SpecificationId`; the name is nominal and is never compared by a prefix,
provider, or display label. `ExecutionCapabilityUniverse` is an explicit
read-only input. Unknown capabilities fail closed.

V1 uses exact version membership. It has no latest lookup, lexical ordering,
semantic-version inference, or implicit `@2 >= @1` rule. A broader relation
requires an explicit future catalog relation and proof.

`CapabilityAttribute` is a closed key/value pair. Keys and values are NFC
normalized, non-empty, sorted, and checked against the vocabulary supplied by
the explicit capability universe.

<a id="execution-capability-identity"></a>
**MUST AF-EXECUTION-0003:** capability
matching MUST use the complete catalog/name identity and an explicitly
recognized exact version.

<a id="execution-capability-attributes"></a>
**MUST AF-EXECUTION-0004:** required
attributes MUST be normalized and vocabulary-checked; unknown attributes MUST
not silently become compatible attributes.

## 4. Compute quantities

V1 defines three independent unsigned integer units:

* `cpu_millicores`;
* `memory_bytes`;
* `gpu_count`.

Quantities are minimum nominal profile capacity, not free capacity. Missing
quantity evidence is not zero supply. Capability and quantity are independent:
GPU capability does not imply a GPU count, and a GPU count does not imply a
particular compute API. There is no cross-unit arithmetic, float quantity,
negative quantity, reservation, topology, MIG, NUMA, gang, or dynamic permit.

<a id="execution-quantity-units"></a>
**MUST AF-EXECUTION-0005:** quantity comparison
MUST compare only equal closed units using checked unsigned integers; a missing
unit is `Unproven`, never zero.

## 5. Sandbox assurance

`SandboxRequirement.minimum` is the closed V1 assurance scale `basic < strong`.
Matching a supplied trusted descriptor at `strong` is a static conditional
proof only. The enum name is not enforcement and a self-asserted advertisement
is not trusted evidence.

<a id="execution-sandbox-assurance"></a>
**MUST AF-EXECUTION-0006:** a weaker or absent
sandbox descriptor MUST reject or leave the match unproven; preferences MUST
never upgrade it. Enforcement and attestation remain outside X.

## 6. Trust domains

`TrustDomainRef` is a nominal `(catalog, name)` identity resolved through an
explicit `TrustDomainUniverse`. No parent/child, hosting-provider, ambient
trusted, identity, delegation, or authority relationship is inferred. Distinct
domains are not interchangeable without a separately defined proof.

<a id="execution-trust-domain"></a>
**MUST AF-EXECUTION-0007:** trust matching MUST be
exact and MUST NOT imply identity, delegation, authorization, or authority.

## 7. Region and residency

`execution_regions` restricts where the computation may execute. The separate
`residency_regions` list restricts the regions in which the relevant data may
reside. Both are hard constraints. Region identities are nominal and resolved
from `RegionUniverse`; no string-prefix, geography, provider name, or
neighboring-region inference is allowed. Empty lists mean no restriction in
that list. Execution placement region is not canonical Run ownership region.

<a id="execution-region-residency"></a>
**MUST AF-EXECUTION-0008:** matching MUST check
execution and residency constraints independently and MUST fail closed when a
required region or cross-region compliance relation is not proven.

## 8. Runtime profile and implementation ABI

`RuntimeProfileRequirement` pins a family, a nominal profile identity, and a
closed required-feature set. `RuntimeProfileUniverse` is explicit; `latest`,
free-form ABI strings, process identity, container identity, and endpoint
identity are not runtime compatibility proofs.

`ImplementationAbiRequirement` pins an existing exact `TypeSchemaRef` identity
and digest. It is an ABI obligation, not a ComponentContract digest. X does
not create or compute `ExecutionRequirementsDigest`, `WorkerProfileDigest`,
`SandboxProofDigest`, or a new `component-contract` preimage.

<a id="execution-runtime-abi"></a>
**MUST AF-EXECUTION-0009:** runtime/profile and ABI
compatibility MUST be exact or explicitly proven; absent evidence is
`Unproven`, never an implicit compatible result.

## 9. Canonical form

Canonicalization resolves all explicit universes, rejects unknown/conflicting
hard declarations, NFC-normalizes identity attributes, sorts capabilities by
full identity/version, quantities by their closed unit number, regions by
identity, ABI references by complete pinned identity, and preferences by
closed case number. Duplicate hard declarations are rejected; duplicate soft
preferences are idempotently removed. Canonicalization is idempotent and
source order does not affect ACE1 bytes.

<a id="execution-canonicalization"></a>
**MUST AF-EXECUTION-0010:** equivalent source
orderings MUST produce the same canonical value and ACE1 bytes; canonicalizing
an already canonical value MUST be a no-op.

## 10. Static profile matching

`StaticExecutorProfile` is a non-wire descriptor containing nominal
capabilities, nominal quantities, sandbox level, trust domain, execution
region, residency properties, runtime profile, and exact ABI references. It
has no worker identity or live state.

<a id="execution-profile-matching"></a>
**MUST AF-EXECUTION-0011:** a profile match MUST
evaluate every hard constraint conjunctively and return only
`ProvenAgainstDescriptor`, `Rejected(reason)`, or `Unproven(reason)`.

The result means only that the supplied descriptor meets the declaration under
the stated trust assumption. It does not mean currently available, admitted,
authorized, scheduled, leased, or safe to execute now.

## 11. Compatibility proofs

For `X_new` and `X_old`, feasibility asks whether every profile eligible for
the old declaration remains eligible for the new declaration. A new quantity,
capability, attribute, sandbox level, or narrowed region set can be a
feasibility violation. Removed/relaxed requirements are not silently treated
as security-preserving.

Security preservation is a separate proof. Lowering/removing sandbox or
expanding/removing trust/residency restrictions is a security violation;
replacement relations without a formal order are `Unproven`. A stronger
sandbox may be secure but can still violate feasibility. Runtime and ABI
changes without an explicit relation are feasibility `Unproven`.

<a id="execution-feasibility-direction"></a>
**MUST AF-EXECUTION-0012:** feasibility
MUST be checked in the old-eligible-subset-of-new-eligible direction; a
strengthened new requirement is not automatically compatible.

<a id="execution-security-preservation"></a>
**MUST AF-EXECUTION-0013:** feasibility
improvement MUST NOT be used as proof that sandbox, trust, or residency
security was preserved.

<a id="execution-combined-compatibility"></a>
**MUST AF-EXECUTION-0014:** combined
compatibility MUST require both independent proofs; one boolean MUST NOT hide
the feasibility/security distinction.

## 12. Cross-dimension and deferred boundaries

X remains independent from F, R, ε, U, and K. Future ComponentContract
composition may check F residency/trust against X region/sandbox, R resource
needs against X quantities/runtime, ε effects against future grants and
runtime capabilities, U cost against placement feasibility, and K recovery
obligations against recovery runtime capabilities. Passing X alone proves none
of those cross-dimension facts.

X does not aggregate the requirements of parallel or sequential computations
into a fictitious single-worker requirement. Group placement, affinity, gang
scheduling, admission, worker advertisement trust, capacity, and runtime
enforcement are later contracts.

<a id="execution-no-authority-or-aggregation"></a>
**MUST AF-EXECUTION-0015:** X MUST
not imply execution authority and MUST not implement blind control-tree
aggregation or a Scheduler/Worker/Admission surface.
