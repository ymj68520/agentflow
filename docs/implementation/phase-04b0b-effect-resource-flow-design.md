# Phase 4B.0B — Effect/Resource and Effect/Flow Decisions

> **DESIGN PROPOSAL** · **NON-NORMATIVE** · **NOT A FROZEN ABI**

## Decision record B0B-D4 — ε↔R Operation/Resource compatibility

**Status: ACCEPTED (design only).**

### Problem and existing semantics

ε identifies an exact `EffectOperationRef` and `ResourceSelector`; R declares
an exact nominal `ResourceTypeRef` and exact-value requirements. Matching the
numeric R port proves only a namespace link. Resource requirement satisfaction,
effect-selector scope, operation support for the resource type, and runtime
authorization are separate facts. The existing crates expose no
operation×resource relation.

### Candidates and choice

* **A — infer by operation/resource short names, port equality, or provider.**
  Rejected: these are not semantic identities.
* **B — add a universal relation field to every old Effect or Resource row.**
  Rejected: that breaks frozen AFIDL semantics and makes capability support
  look like a component-authored permission.
* **C — exact versioned analysis catalog.** Chosen: a new closed
  `EffectResourceCompatibilityCatalog` is an explicit immutable relation
  snapshot supplied to the checker. It is not an EffectGrant and is not
  embedded in an old ε/R schema.

### Candidate typed relation shape

```text
ScopeValueRule =
    finite_exact_values(list<string>)
  | opaque_nfc_exact_string(unit)

SelectorKeyRule {
    selector_key: string
    resource_attribute_key: string
    required: bool
    value_rule: ScopeValueRule
}

SupportedEffectResourcePair {
    operation: effect.EffectOperationRef
    resource_type: resource.ResourceTypeRef
    selector_keys: list<SelectorKeyRule>
    allowed_qualifier_profiles: list<EffectQualifierProfile>
}

EffectQualifierProfile {
    qualifiers: list<effect.EffectQualifier>
}

EffectResourceCompatibilityCatalog {
    complete_operations: list<effect.EffectOperationRef>
    supported_pairs: list<SupportedEffectResourcePair>
}
```

All proposed lists are closed and canonically ordered. `complete_operations`
is the closed-world declaration: if an exact operation is listed there, its
`supported_pairs` are exhaustive. If it is absent from that coverage set, the
checker cannot infer a denial from a missing row.

`allowed_qualifier_profiles` enumerates complete exact qualifier lists, not
independent members that could be combined arbitrarily. An explicit profile
with an empty `qualifiers` list permits an unqualified atom. Each list uses
full family/member/payload identity and rejects duplicate families.

`EffectResourceCompatibilityCatalog` is a candidate additive schema
(`agentflow.effect.resource.compatibility@1`, provisional and unregistered).
Its instance is pinned as an analysis input by exact catalog identity,
revision, and canonical-content checksum. The checksum is plain SHA-256 over
the catalog's canonical bytes, not HashV1 and not a ComponentContractDigest.
The catalog publisher/signature is not authenticated by the static checker.

### Identity, scope, and qualifier rules

* Operation identity is full `(catalog, name)`; resource identity is the full
  TypeSchemaRef triple. No prefix, provider, same-port, short-name, or
  structural-type fallback exists.
* Each selector key maps explicitly to a resource attribute key. A selector
  binding is an exact NFC literal match; no prefix/glob/range behavior exists.
  `finite_exact_values` is a closed allowlist. The explicitly selected
  `opaque_nfc_exact_string` mode means the catalog accepts any NFC literal as
  an exact value for that key; it does not permit wildcard comparison.
* `required=true` requires the key in every selector; an omitted optional key
  has the wider selector meaning already defined by ε.
* If a mapped selector binding and R's required attribute bind the same
  resource attribute to different exact values, the conjunction is empty and
  the contract is a Violation. If R does not constrain that attribute, the
  selector is a further exact scope restriction; it does not prove a runtime
  resource instance exists.
* Qualifiers are matched as complete typed values: family, member
  discriminant, and payload. `natural` never matches `none`; family-only
  matching is prohibited.

### Proof outcomes and distinct checks

The assembly pass keeps three verdicts separate:

1. **Resource satisfies requirement:** exact ResourceTypeRef and candidate
   attributes satisfy R's exact predicates.
2. **Selector fits resource scope:** selector keys/values are permitted by
   the catalog and are consistent with R's attribute constraints.
3. **Operation supports resource type:** the exact operation/type pair is
   explicitly supported by the catalog.

`ResourceRequirementRow` candidate-satisfaction predicate remains its frozen
exact-type/exact-attribute test. Effect selector scope is an ε permission
upper bound. Operation support is a capability-catalog relation. Their
comparison directions are not interchangeable: a narrower effect selector
does not weaken an R requirement, and a weaker R requirement does not broaden
an ε permission.

Only the conjunction is a proven static relation. `Compatible` means the
exact pair and selector pass all three checks. `Incompatible` is a known
closed-catalog denial, unsupported key/value/qualifier, required binding
missing, or contradictory R/selector predicate. `Unproven` means the
operation is not covered by an exhaustive snapshot, the snapshot is absent,
the ResourceTypeRef cannot be resolved, or the Effect row is Open/Unknown.
An Open residual admits unknown operations, so its complete relation cannot
be checked by enumerating only visible atoms.

### Authority and migration boundary

An operation/resource relation proves only catalog-declared semantic support
under the named snapshot. It does not prove that a worker has the capability,
that a resource was bound, that a credential exists, or that the action is
authorized. Those remain admission/attestation/EffectGrant/ResourceHandle
obligations. Existing AFIDL and TypeSchemaDigests are unchanged; the future
ComponentContract digest includes ε and R, not this analysis snapshot. A
proof report records the exact catalog revision/checksum; comparisons under
different snapshots are Unproven unless a separate catalog-compatibility rule
is supplied.

The operation/resource catalog is a separate additive schema and introduces
no old-field migration. The catalog snapshot is excluded from
ComponentContractDigest; changes require re-validation under the new exact
snapshot but do not rewrite the component's declared ε or R.

### Rejected alternatives and residual risk

Equal port numbers are only local links. Resource requirement predicates are
not capability ceilings. A catalog that claims exhaustive support is still a
caller-supplied trust assumption. Static compatibility cannot establish
actual provider behavior, availability, or authorization.

## Decision record B0B-D5 — ε↔F exact sink grain

**Status: ACCEPTED (design only).**

### Problem and existing wire

Frozen `FlowSink::ExternalEffect` contains only `EffectOperationRef`, while
ε identity is `(operation, ResourceSelector, complete qualifier set)`. A
single operation-level sink therefore cannot express a selector-specific
distinction. The frozen F schema is preserved byte-for-byte; no existing
FlowConstraint is reinterpreted as implicitly naming one selector.

### Candidates and choice

* **A — revise FlowConstraint to carry EffectAtom.** Rejected for v1 because
  it changes the existing Flow schema and its canonical bytes.
* **B — add a versioned F schema.** Kept as a possible later migration, not
  selected for this candidate because the new ComponentContract can add a
  typed refinement without changing old F.
* **C — typed Component-level exact partition.** Chosen.

### Candidate type and semantics

```text
EffectFlowPartition {
    operation: effect.EffectOperationRef
    kind: flow.FlowKind
    allowed_effects: list<effect.EffectAtom>
}
```

The future ComponentContract's `effect_flow_partitions` list contains at most
one complete partition per exact `(operation, FlowKind)`. `allowed_effects` is
an exhaustive closed allowlist for that pair; an omitted matching atom is not
allowed by the refined policy. Each listed atom must match ε by the complete
AtomKey, including resource port, exact scope bindings, qualifier member and
payload. The list is sorted by full AtomKey and rejects duplicates.

The partition may only **narrow** an existing operation-level
`FlowConstraint`: an exact partition without a matching coarse sink is a
Violation because it would introduce a permission absent from frozen F. If
there is no exact partition, the old operation-level constraint keeps its
literal coarse meaning: every declared ε atom with that operation is in its
potential sink ceiling. It is never silently narrowed by a port number or
caller convention. An empty exact allowlist is a valid explicit denial of
all matching effect atoms, subject to the rest of F's static analysis.

### Source influence and proof direction

F currently has one aggregate `StaticInformationSummary`, not per-sink
provenance. V1 applies that entire may-influence upper bound to every allowed
exact sink; it never filters sources/taints/authority separately for one
selector. This is conservative and may over-restrict. Source-specific
partitioning requires a separately reviewed F schema/profile and a proven
dataflow relation; it is not inferred from this binding.

Every allowed atom must exist in closed ε. A missing F sink is Violation for
an attempted new allowance; a dangling allowed atom is Violation; open or
unknown ε makes partition completeness Unproven. Unknown provenance,
`unknown_external`, or `influence_unknown` remains Unproven for positive flow
proof.

For F non-relaxation, compute the effective exact-atom allowed set per
`(operation, kind)` on each closed contract. A coarse old sink expands to all
matching atoms in that old ε; a partitioned sink is its explicit exact set.
The new effective set must be a subset of the old set, with all atoms compared
by full identity. If either side is Open/Unknown or expansion lacks complete
coverage, return Unproven. A selector-specific F policy never proves
authorization or actual information-flow enforcement.

### Existing ABI, digest, and residual risk

The new partition is a candidate ComponentContract field (field 11), so its
typed atoms enter the future contract digest. Frozen `InformationFlowContract`
and its TypeSchemaDigest do not change. The partition cannot broaden F; exact
member values use the Phase 4B.0A AtomKey rule. Operation-level-only contracts
remain expressible but mean “all matching atoms,” not a selector-specific
permission. Actual sources may be more precise than the global summary, but
that precision cannot be claimed by this v1 binding. Old F values remain
parseable; they are not retroactively reinterpreted and cannot gain a
selector-specific denial without the new typed partition.
