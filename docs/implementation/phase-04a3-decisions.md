# Phase 4A.3 Decision Register — Resource Requirement Row

Phase: 4A.3 (R dimension of `Computation<I, O, Φ, ε, R>`), closed
2026-10-08. Wire authority: `spec/idl/resource-requirement.afidl`;
normative semantics: `spec/resource/resource-requirement.md`; Rust:
`crates/af-resource`; frozen corpus: `spec/conformance/resource-requirement/`
(independent oracle: `tools/gen-resource-requirement-corpus.py`).

## D-1 ResourceTypeRef nominal identity

**Decision:** `ResourceTypeRef { 1 schema: core.TypeSchemaRef }` — the
semantic core's reference shape reused by exact cross-module import,
never a bare string and never a redeclared reference. Identity is the
full triple (SpecificationId, TypeName, pinned TypeSchemaDigest): same
short name under a different specification, or under a different pin, is
a DIFFERENT resource type, in satisfaction and in compatibility alike.

**Why recorded:** the requirement row is consumed by contract evolution
checks; any looser identity would silently merge distinct resource
universes. The corpus freezes `same-short-name-different-family`
(golden), `same-short-name-different-digest` (compatibility) and
`candidate-wrong-type` (negative).

## D-2 Explicit ResourceTypeUniverse

**Decision:** recognition is an explicit registry-level fact. The
analysis-side `ResourceTypeUniverse` wraps an exact-pinned
`af_semantic::TypeSchemaUniverse` (schema layer: existence + digest
verification) plus an explicit descriptor table (recognition + closed
attribute vocabulary per type). `resolve` maps
`UnknownTypeSchema → unknown_resource_type`,
`TypeSchemaDigestMismatch → resource_type_digest_mismatch`, other →
`invalid_resource_type`; a registered-but-unrecognized schema is
`unknown_resource_type` ("a legal AFIDL schema alone is not a resource
type"); a descriptor pinning a stale digest is
`resource_type_digest_mismatch`. Re-recognizing an identity is rejected:
recognition is a set and is never silently redefined.

**Why recorded:** the frozen architecture separates "is a schema" from
"is a resource type". The test catalog supplies explicit descriptors and
never masquerades as a production registry (production registry is a
later stage).

## D-3 Resource port identity

**Decision:** `port_no: uint32` with `port_no > 0`
(`invalid_resource_port`), unique per row (`duplicate_resource_port`),
canonical order ascending. The port is a requirement coordinate inside
ONE contract interface namespace: it is not a ResourceSlotId, not a
handle id, not a logical resource instance id and not a Phase 3D derived
subflow local id; relation to those coordinate systems is future explicit
binding/mapping.

## D-4 Required attributes representation

**Decision:** `ResourceAttributeConstraint { key, value }` — exact-value
string pairs, conjunctive. Closed predicate language: no glob, regex,
path prefix, CIDR containment, policy expression or predicate script.
Keys and values are non-empty NFC text. Future dimensions with their own
formal semantics (residency, trust, durability, capacity) get their own
typed shapes and are never overloaded onto this map.

**Why recorded:** this is the only provable predicate class in this
phase; everything richer is deferred rather than faked.

## D-5 Exact predicate semantics

**Decision:** a requirement is a lower bound on candidate attributes:
`satisfies` holds iff the candidate's type exact-matches AND the
candidate carries every required (key, value) pair; extra candidate
attributes are fine. `satisfies_resource_requirement` takes an abstract
`ResourceCandidate` (analysis-side model, NOT in the AFIDL wire model —
its wire form belongs to the later binding stage; recorded here as the
required justification for an auxiliary type outside the four wire
objects).

## D-6 Requirement non-strengthening direction

**Decision:** the formal principle is
`{candidates satisfying old} ⊆ {candidates satisfying new}` — the new
contract's acceptable set must not be narrower. Under the exact-attribute
conjunctive model the per-retained-port obligation is
`new.required_attributes ⊆ old.required_attributes` as full (key, value)
pairs. This direction is deliberately OPPOSITE to the Phase 4A.1 effect
ResourceSelector scope containment (a permission argument); the two are
never interchanged.

**Why recorded:** the task explicitly warns against reusing the effect
containment direction; the spec pins the candidate-set argument and the
corpus freezes add/remove-attribute on both sides of the verdict.

## D-7 Unknown / Unproven classification

**Decision:** `ResourceCompatibilityProof { Proven, Unproven(reason) }`
with reasons `retained_port_resource_type_changed` and
`unknown_attribute_semantics`. Proven violations are NOT Unproven: a new
port, or a new/changed pair on a retained port, is a proven strengthening
returned as `Err(resource_requirement_strengthened)`. Unproven never
enters the wire model and is never auto-judged compatible.

**Why recorded:** conflating "violated" with "unprovable" would let
contract evolution masks pass; the split keeps `strengthened` a hard
rejection and `Unproven` fail-closed.

## D-8 Asymmetric unknown-attribute rule

**Decision:** only attribute keys constrained by the NEW side must be in
the descriptor vocabulary; unknown keys on the OLD side never block the
proof. Rationale: any requirement attribute, whatever its semantics, only
shrinks the old candidate set, so old-side unknowns are safe; a new-side
unknown cannot be assumed conjunctive exact-value, so the subset argument
would not apply → `Unproven(unknown_attribute_semantics)`. Frozen by
`old-unknown-attribute-does-not-block` (Proven) and
`unknown-attribute-implication-unproven` (Unproven).

## D-9 No implicit ResourceType subtyping

**Decision:** a retained port whose nominal type changed — including the
same short name under a different pin — returns
`Unproven(retained_port_resource_type_changed)`; no structural or lexical
compatibility is ever inferred.

## D-10 Requirement vs capability vs grant

**Decision:** requirement (need), capability ceiling (allowance) and
effect grant (authorized action) are three distinct objects; requirement
satisfaction is a predicate and produces no runtime authority. The static
audit forbids every grant/attest/credential symbol in crate code.

## D-11 No blind cross-component union

**Decision:** no union/merge/aggregation operation exists for
requirement rows (static-audited: `fn union|fn merge|fn aggregate`
absent). Aggregating a control node's R requires an explicit port
mapping at the later contract-analysis stage.

## D-12 Namespace parameter on requirement_not_stronger

**Decision:** the signature is
`requirement_not_stronger(new_row, new_namespace, old_row, old_namespace,
universe)`. Comparing rows of two different namespaces fails closed with
`resource_requirement_unproven` — equal port numbers across components do
not name the same resource. The parameter makes the §11 obligation
("comparison objects must live in the same port namespace")
machine-checkable instead of a documentation claim; the frozen
`cross-component-namespaces-not-merged` case exercises it.

## D-13 Removed-port semantics

**Decision:** a removed requirement port never strengthens in the R
dimension (fewer needs cannot narrow the accepted set). Whether the
surrounding contract INTERFACE may drop a port at all is explicitly
deferred to the full-contract compatibility checker; this phase makes no
interface-compatibility claim. Frozen by `new-removes-port` (Proven).

## D-14 NFC normalize-then-reject

**Decision:** attribute text is NFC-normalized BEFORE duplicate
detection and sorting. Consequences frozen in the corpus:
NFC-equivalent member texts produce identical canonical bytes
(`nfc-equivalent-text`), while a duplicate normalized key with two
different values is rejected (`nfc-duplicate-attribute-key`) — NFC
equality is an identity fact, not a legal-equivalence input.

## D-15 Effect selector linkage recorded, not validated

**Decision:** the cross-dimension invariant ("Every declared Effect
resource selector must reference an existing ResourceRequirement port …")
is recorded as a normative MUST in
`spec/resource/resource-requirement.md` §12 and enforced only as a
read-only consistency audit (effect module keeps `requirement_port_no`;
resource module keeps `port_no: uint32`; the spec sentence exists). This
phase performs NO effect-port validation, creates no temporary contract
object between ε and R, and never derives a grant from R. The invariant
is a blocking condition for the contract-integration stage.

## D-16 No new HashV1 domain

**Decision:** no resource-requirement digest of any kind exists
(`ResourceRequirementDigest`/`ResourceRowDigest`/`ResourceBindingDigest`
are static-audited absent); hash-domains.json stays at 14 entries. Only
the new module's own types carry ordinary TypeSchemaDigests; the future
contract layer hashes as a whole under the reserved
`component-contract` domain.

## D-17 Error-kind plane separation

**Decision:** the 12 stable kinds
(`invalid_resource_type`, `unknown_resource_type`,
`resource_type_digest_mismatch`, `invalid_resource_port`,
`duplicate_resource_port`, `invalid_resource_attribute`,
`duplicate_resource_attribute`, `unknown_resource_attribute`,
`resource_requirement_unsatisfied`, `resource_requirement_unproven`,
`resource_requirement_strengthened`, `resource_encoding_error`) are
compile-time results of the contract layer, never mapped onto authority
denials, scheduling conditions, credential states or incidents; the
forbidden plane names are static-audited absent from crate code.

## Deferred by design

ResourceBinding / ResourceHandle / ResourceAdapter / physical resolver /
EffectGrant / CredentialBroker / SecretLease / attenuation chains /
runtime authority evaluation / PolicyEngine / DeploymentRevision /
BindingSnapshot / availability & capacity scheduling / worker placement
/ quota / usage ledger / InformationFlow / RecoveryContract /
ExecutionRequirements / plan / journal / reducer / runtime /
ComponentContract — all structurally absent and static-audited.
