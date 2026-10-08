# Phase 4B.0B — Component Port and Type Policy

> **DESIGN PROPOSAL** · **NON-NORMATIVE** · **NOT A FROZEN ABI**

This document records B0B-D2 and B0B-D3. Both decisions are accepted as
conservative v1 rules; neither changes Semantic IR or semantic-core.

## Decision record B0B-D2 — I/O port ABI and compatibility

**Status: ACCEPTED (design only).**

### Problem and existing frozen semantics

Semantic IR has document-local `RunParameterId`, `WorkflowOutputDecl.name`,
`ResourceSlotId`, `SemanticTypeId`, and call-site port coordinates. Phase 3D
derives Subflow I/O/R ordinals from one exact SemanticIrDigest; those ordinals
are not global Component IDs. `InvokeBody.inputs`,
`ComputationOutputBinding`, and `ResourceUseBinding` use different caller-side
coordinates. No Component I/O AFIDL exists.

### Candidates

* **A — use local SemanticTypeId and names.** Rejected: IDs are document-local
  and names are editable diagnostics.
* **B — stable positive ports with exact named schema pins.** Chosen: three
  separate closed namespaces; type pins use the existing exact
  `core.TypeSchemaRef` shape.
* **C — implicit subtype/variance or defaulted ports.** Rejected for v1:
  neither subtype proof nor default/call omission semantics is frozen.

### Type and field proposal

```text
ComponentInputPort  { port_no: uint32, type: core.TypeSchemaRef }
ComponentOutputPort { port_no: uint32, type: core.TypeSchemaRef }
```

The candidate ComponentContract body owns `inputs` and `outputs` lists.
Resource ports remain the embedded `resource.ResourceRequirementRow`.
`ComponentPortType` is an alias for the exact TypeSchemaRef shape, not a new
untyped structural union. Primitive/container public types are published as
named AFIDL schemas or aliases and pinned by TypeSchemaRef; they are not
referenced by a SemanticTypeId.

### Identity and namespace rules

* I, O and R are independent, 1-based, duplicate-free namespaces.
* Port identity is `(ComponentIdentity, direction, port_no)`. A number is
  never rebound or reused within a lineage. Renumbering is a semantic contract
  change; names do not remap it.
* Port names/descriptions are outside canonical bytes, in a closed
  documentation projection keyed by `(direction, port_no)`.
* `option<T>` means the port value may be None; it does not make the port
  optional. V1 has no omitted/defaulted, variadic or silently ignored inputs.
* Subflow ordinals remain derived for the exact target SemanticIrDigest.
  `RunParameterId`, output names and `ResourceSlotId` never become Component
  port identity. Explicit call bindings perform the mapping.

### Proof direction and preconditions

Substitution retains the architecture directions:

```text
I_old <: I_new
O_new <: O_old
```

V1 proves a general I/O relation only when the positive port sets are exactly
equal and every corresponding exact TypeSchemaRef is equal. No integer
SemanticTypeId comparison, name lookup, shape resemblance or coercion is a
proof. Required input addition/removal and required output removal are known
call-shape violations when the old closed call surface is available. Any
other unequal port set or type pin is Unproven for general substitution;
there is no implicit port map. Added outputs remain Unproven because INVOKE
does not freeze an “ignore unknown outputs” rule.

The checker must resolve every pin against one explicit TypeSchemaUniverse
and verify its TypeSchemaDigest before comparing it. Unknown/unavailable
universe is Unproven; malformed or stale pins are a closed validation
Violation. Exact pins across two contracts are compared by the full triple,
not by matching short type names.

### Counterexamples and fail-closed cases

| Input | Expected | Reason |
| --- | --- | --- |
| Same I/O ports and same exact TypeSchemaRefs | Proven | Exact closed ABI equality |
| New component adds a required I port | Violation for the old call surface | Old caller cannot provide it |
| Old required O port is absent from new O | Violation | New component cannot supply an old required result |
| Same port number, different resolved TypeSchemaRef | Unproven | No subtype or coercion rule |
| Port number changes but diagnostic name is unchanged | Unproven | Name is not identity or an implicit mapping |
| New extra O port | Unproven | No frozen output-ignore/width-subtyping rule |

### Existing AFIDL, digest, and migration effect

No existing schema changes. The future ComponentContract uses candidate
top-level I/O fields 2 and 3; R remains its existing typed row. Port lists
and exact pins enter ComponentContract canonical bytes. Diagnostic names and
derived Subflow ordinals do not. An old→new port map, if a later profile adds
one, is an explicit compatibility input and never rewrites either contract's
identity.

### Deferred obligations and residual risk

There is no nominal subtype, width-subtyping, variance, or optional-input
semantics in v1. A concrete caller may have stronger evidence than the general
compatibility rule, but that evidence must be represented by a separately
specified mapping/proof; it cannot be inferred from these declarations.
Implementation/port behavior still needs compatibility attestation.

## Decision record B0B-D3 — Public Alias comparison

**Status: ACCEPTED (design only).**

### Problem and existing frozen semantics

`TypeSchemaRef` is an exact `(SpecificationId, TypeName, TypeSchemaDigest)`
pin. Phase 3A expands aliases for value-level structural typing; Phase 3D
Subflow interface comparison preserves exact schema leaves. A Component port
must not inherit an accidental helper-specific alias policy.

### Candidates

* **A — public nominal, internal structural.** Chosen: exact TypeSchemaRef
  equality at Component boundaries; controlled alias expansion only inside
  existing internal value typing.
* **B — fully transparent aliases everywhere.** Rejected: it equates distinct
  security/protocol/schema identities and conflicts with Phase 3D's pinned
  leaf behavior.
* **C — explicit per-pair compatibility proof.** Deferred: no such frozen
  proof relation exists. V1 does not invent an opt-in escape.

### Chosen identity and operations

```text
Public Alias Identity = exact TypeSchemaRef
Internal Structural Normalization = validated expansion for pure value typing
Public ABI Compatibility = exact TypeSchemaRef equality only
```

These are deliberately not the same operation. The public pin includes the
schema digest. Equal underlying shapes do not erase nominal identity; equal
schema name with a changed digest is not the same pin. Alias resolution is
used to validate the named schema and perform internal type checks, but the
expanded target is not substituted into the ComponentContract bytes.

### Required cases and expected outcomes

| Case | Public result | Internal rule |
| --- | --- | --- |
| Same alias and exact pin on both ports | Proven | Expanded shape may also be checked for validity |
| Same alias name, different TypeSchemaDigest | Unproven | Pins identify different schema revisions |
| Different aliases with the same primitive target | Unproven | Equal target does not equate nominal public identities |
| Alias of a record vs another structurally identical record alias | Unproven | No record-shape compatibility is inferred |
| Alias of a nominal security object vs its primitive/record target | Unproven | Never erase security/protocol nominal boundary |
| Cross-SpecificationId aliases with equal targets | Unproven | Catalog/name/digest pins differ |
| Acyclic alias chain with the same outer pin | Proven at the public pin; schema validation resolves the chain | The chain does not change public identity |
| Unsupported/cyclic alias resolution | Violation if schema is malformed; otherwise Unproven if the resolver is unavailable | Never normalize to an empty/primitive type |

### Digest, migration, and residual risk

Port TypeSchemaRefs are included verbatim in the future ComponentContract
canonical body. A change to the alias target changes the alias TypeSchemaDigest
under the existing type-schema machinery, so it changes the port pin and
contract digest. There is no old TypeSchemaDigest rewrite and no new hash
domain. A future explicit alias-compatibility relation would be a separately
versioned additive proof input, not a v1 implicit normalization.

Publisher tests/attestation remain necessary to prove a concrete
implementation honors the nominal port type. Structural equality alone is
not behavioral compatibility.
