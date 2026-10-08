# Resource Requirement Row (agentflow.resource.requirement@1)

Status: frozen (Phase 4A.3). The machine schema authority is
`spec/idl/resource-requirement.afidl`; this document is the normative
semantics. The Rust implementation is `crates/af-resource`; the frozen
conformance corpus is `spec/conformance/resource-requirement/`
(independent Python oracle: `tools/gen-resource-requirement-corpus.py`).

## 1. Static positioning

<a id="rr-static-role"></a>

The Resource Requirement Row is the **R dimension** of the frozen
computation model `Computation<I, O, Φ, ε, R>` (docs §14.4, §22.13, §24):
the closed set of logical resources a computation requires from the
outside, under statically provable conditions, to be executable at all.
R answers "what must be PROVIDED"; it never answers "what was OBTAINED in
this Run". The frozen declaration chain is

```text
ResourceType → ResourceSlot → ResourceBinding →
Logical Resource Identity → ResourceHandle → ResourceAdapter →
Physical Resource
```

and exactly one static head of it is materialized here: the
ResourceType and the Requirement Row. ResourceBinding, ResourceHandle,
ResourceAdapter and physical resolution are structurally absent from this
layer. The F, Φ, ε and R dimensions stay independently materialized; the
future contract layer composes them.

## 2. Nominal resource-type identity

<a id="rr-nominal-identity"></a>

A `ResourceTypeRef` is nominal: its identity is the full triple
`(SpecificationId, TypeName, pinned TypeSchemaDigest)`. Consequences:

- same type name + different specification → DIFFERENT resource types;
- same logical name + different pinned digest → different identities,
  never silently compatible.

<a id="rr-exact-pinning"></a>

The schema reference reuses the semantic core's `TypeSchemaRef` by exact
cross-module identity — never a bare string, never a redeclared reference
shape. The pin is exact: validation must reject an unregistered schema
(`unknown_resource_type`) and a wrong digest
(`resource_type_digest_mismatch`).

<a id="rr-no-latest-lookup"></a>

There is no latest-schema lookup, no fallback and no implicit upcast; two
legal versions of the same type keep their full nominal identities.

## 3. The explicit resource-type universe

<a id="rr-universe"></a>

A legal AFIDL TypeSchemaRef does NOT automatically make a resource type.
Recognition is an explicit registry-level fact: the analysis-side
`ResourceTypeUniverse` is an explicit, read-only, in-memory input that
(a) resolves every pin exactly against its schema layer and (b) carries
an explicit descriptor per recognized resource type, fixing that type's
closed attribute vocabulary. An unrecognized schema, or a descriptor that
pins a stale digest, fails closed. The production resource-type registry
is a later stage; test catalogs supply explicit descriptors and are never
masqueraded as a production registry.

## 4. The requirement port namespace

<a id="rr-port-namespace"></a>

`ResourceRequirement.port_no` is a stable requirement port inside ONE
contract interface's namespace. It is not a ResourceSlotId, not a
ResourceHandleId, not a logical resource instance id and not a Phase 3D
derived subflow local id; those coordinate systems relate to requirement
ports only through explicit future binding/mapping.

<a id="rr-port-positive"></a>

The port must be positive: `port_no > 0`, with `port_no = 0` rejected
(`invalid_resource_port`).

<a id="rr-ports-unique"></a>

One row declares at most one requirement per port: duplicate port numbers
are rejected (`duplicate_resource_port`).

<a id="rr-no-blind-union"></a>

Port numbers are LOCAL to a contract interface. Component A's port 1 and
component B's port 1 are different resources by default, so this phase
implements NO cross-component requirement union or merge: aggregating a
control node's R from its children requires an explicit port mapping and
belongs to the later contract analysis. Merging rows by equal port number
is forbidden.

## 5. Exact-value attribute constraints

<a id="rr-exact-attributes"></a>

A `ResourceAttributeConstraint` is an exact-value conjunctive predicate:
a candidate must carry attribute `key` with exactly `value`, for every
declared constraint. The predicate language is closed — no glob, no
regex, no path prefix, no CIDR containment, no dynamic policy expression,
no predicate script. Keys and values are non-empty NFC text.

<a id="rr-canonical-attributes"></a>

Attribute text is NFC-normalized before comparison; the canonical order
of one requirement's constraints is unsigned UTF-8 order by key (then
value). Semantically identical requirement texts therefore produce
identical canonical ACE1 bytes.

<a id="rr-attributes-duplicates"></a>

Two constraints whose normalized keys are equal are rejected
(`duplicate_resource_attribute`) — different values for one key are never
silently overwritten, and NFC-equivalent spellings are the same key, not
a legal equivalence input. An attribute key outside the resource type's
descriptor vocabulary has no known constraint meaning and is rejected
(`unknown_resource_attribute`); future dimensions with their own formal
semantics (residency, trust, durability, capacity) get their own typed
shapes and are never overloaded onto this string map.

## 6. The closed Requirement Row

<a id="rr-closed-row"></a>

`R = {r1, r2, ... rn}` is a closed set of requirements; the wire model
has no open, residual or unknown state. Empty is a plain set:

<a id="rr-empty-not-requirement"></a>

`R = ∅` means "no external resource requirement is declared". It does NOT
mean the computation is pure (purity is ε), does not mean it succeeds
(that is the outcome plane) and does not mean it is authorized (that is a
future security/effect grant).

<a id="rr-unknown-not-empty"></a>

A requirement upper bound this phase cannot derive is an analysis-level
unknown — it is never represented by an empty row, because empty would
falsely claim "needs nothing".

## 7. Canonical ordering

<a id="rr-canonical-ordering"></a>

`canonicalize_resource_requirement_row` normalizes attribute text to NFC,
rejects the duplicates of section 5, sorts requirements by ascending port
number and constraints by unsigned UTF-8 key order. Source declaration
order is not identity and never reaches the canonical ACE1 bytes; no hash
map iteration order is observable. Canonicalization is idempotent, and
the same semantic requirements always produce the same canonical bytes.

## 8. Static satisfaction

<a id="rr-static-satisfaction"></a>

`satisfies_resource_requirement(candidate, requirement)` is a pure static
predicate over explicitly supplied values: `candidate` is an abstract
resource descriptor, not a runtime handle. It holds iff

```text
candidate.resource_type == requirement.resource_type   (exact nominal)
candidate attributes ⊇ requirement.required_attributes (exact values)
```

Extra candidate attributes are fine: a requirement is a lower bound.

<a id="rr-satisfaction-exact-type"></a>

The type match is exact — the same short name, the same record shape or
a similar provider is never accepted. This predicate proves ONLY static
attribute matching. It does not prove credential availability, live
resource existence, capacity, authorization or effect permission; those
belong to the later binding/admission/runtime checks.

## 9. Requirement non-strengthening

<a id="rr-non-strengthening"></a>

The frozen architecture requires requirement non-strengthening for
contract evolution. Formal principle:

```text
{candidates satisfying the old row}
        ⊆
{candidates satisfying the new row}
```

i.e. the new contract's acceptable resource set must not be narrower than
the old one's. `requirement_not_stronger(new, old)` decides this within
ONE port namespace; comparing rows of two different namespaces is not a
static proof and fails closed with `resource_requirement_unproven`.

Under the exact-attribute conjunctive model the per-port proof obligation
is the subset comparison

```text
new.required_attributes ⊆ old.required_attributes   (full (key,value) pairs)
```

for every port retained by the new row. The direction is the OPPOSITE of
the Phase 4A.1 effect-scope containment: effect scope containment is a
permission argument (a narrower scope proves a bounded effect); resource
non-strengthening is a need argument (fewer or equal requirements prove
the new implementation is at least as deployable). The two must never be
interchanged.

<a id="rr-ns-ports"></a>

Port rules: a NEW port (`R_new` requires a port `R_old` does not) is
strengthening (`resource_requirement_strengthened`). A REMOVED port never
strengthens in the R dimension — whether the surrounding contract
interface may drop a port at all is decided by the later full-contract
compatibility checker, not here.

<a id="rr-no-subtyping"></a>

Type rules: a retained port must keep its exact nominal resource type. A
changed type — including the same short name under a different pin — is
not provable and returns `Unproven(retained_port_resource_type_changed)`;
no implicit resource-type subtyping exists.

<a id="rr-unknown-attrs-compat"></a>

Attribute rules: every attribute key the NEW side constrains must be in
the type's descriptor vocabulary — an unknown-semantics constraint cannot
be assumed conjunctive exact-value, so the subset argument would not
apply; the result is `Unproven(unknown_attribute_semantics)`. Unknown
keys on the OLD side only shrink the old candidate set and never block
the proof. Proven violations (new port, new or changed `(key,value)`
condition on a retained port) are `resource_requirement_strengthened`;
unprovable outcomes are returned as `Unproven(reason)` and are never
auto-judged compatible.

## 10. Unknown / Unproven

<a id="rr-unproven"></a>

`ResourceCompatibilityProof` is `{Proven, Unproven(reason)}`. Unproven is
a fail-closed analysis result: it never enters the wire model, is never
interpreted as the empty row and is never judged compatible. A demand for
a proof from a cross-namespace comparison fails with
`resource_requirement_unproven`.

## 11. Requirement vs capability vs grant

<a id="rr-vs-capability"></a>

Three authority concepts are strictly separate and never collapsed:

```text
Resource requirement:      what a component NEEDS
Resource capability ceiling: what it is ALLOWED to access
EffectGrant:               which exact external action may execute
```

A resource satisfying a requirement grants nothing: requirement
satisfaction is a predicate, not an authorization. No code path in this
layer derives a grant, a capability ceiling or an attenuation from a
requirement row.

<a id="rr-authority-distinction"></a>

The ResourceHandle capability ceiling, EffectGrant and
ResourceRequirement are distinct objects with distinct lifetimes and
distinct authorities; declaring a requirement does not produce any
runtime authority, and no `requirement satisfied → grant capability`
path exists.

## 12. Effect ResourceSelector linkage

<a id="rr-effect-selector-linkage"></a>

Phase 4A.1's `EffectAtom.resource_selector.requirement_port_no` and this
phase's `ResourceRequirement.port_no` are the future connection points.
The cross-dimension invariant to be enforced at contract integration is:

```text
Every declared Effect resource selector must reference an existing
ResourceRequirement port, and the referenced requirement's resource type
must carry the operation capability and authorization ceiling the effect
declares.
```

Until that stage exists this phase claims NO effect-port validation: an
effect selector referencing a positive integer proves nothing about any
row, no temporary contract object connects ε and R, and R never derives
an effect grant.

## 13. Type / slot / binding / handle boundary

<a id="rr-slot-binding-handle-boundary"></a>

This layer ends at the requirement. It contains no resource slot table,
no binding, no handle, no adapter, no credential, no lease, no physical
resolver, no availability or capacity scheduling, no quota, no usage
ledger and no placement logic; none of those names exists anywhere in
the model of this phase, and no temporary context object combines
declaration, binding and authority.

## 14. Error model

<a id="rr-error-model"></a>

Stable rejection kinds: `invalid_resource_type`, `unknown_resource_type`,
`resource_type_digest_mismatch`, `invalid_resource_port`,
`duplicate_resource_port`, `invalid_resource_attribute`,
`duplicate_resource_attribute`, `unknown_resource_attribute`,
`resource_requirement_unsatisfied`, `resource_requirement_unproven`,
`resource_requirement_strengthened`, `resource_encoding_error`. These are
compile-time results of the contract layer and are never mapped onto
runtime authority denials, scheduling conditions, credential states or
incidents of other semantic planes.

## 15. Digest boundary and deferred

<a id="rr-hash-boundary"></a>

No resource-requirement digest of any kind exists; the HashV1 domain
registry stays at its frozen set. Only the new AFIDL module's own types
carry ordinary `TypeSchemaDigest`s. The future contract layer will hash
as a whole under the already-frozen `component-contract` domain.

<a id="rr-contract-integration"></a>

The future contract layer composes F (interface), Φ, ε and R; that is
where requirement ports meet component ports, effect selectors meet
requirement ports, and requirement non-strengthening becomes one input
of the full substitution check. None of that exists yet.

<a id="rr-deferred"></a>

Deferred by design (structurally absent, static-audited): the contract
object itself, ResourceBinding, ResourceHandle, ResourceAdapter,
physical resource resolution, grants, credentials, leases, attenuation
chains, runtime authority evaluation, policy engines, deployment
revisions, binding snapshots, availability and capacity scheduling,
worker placement, quota, usage ledgers, information flow, recovery
contracts, execution requirements, plan, journal, reducer and runtime.
