# Core Semantic Types — v1

> Status: AgentFlow workflow semantic core v1 repository normative source
> (materialized in Phase 3A). This document freezes the **semantic data ABI**
> that later layers (Semantic IR, Plan, Task Contract, Run Parameters, Value
> Store, Reducer) reference.
>
> Semantic-shape-changing edits require an explicit specification decision and
> a frozen-corpus update (`spec/conformance/semantic-core/`).

## 1. Scope

Phase 3A defines exactly five machine object families and nothing else:

1. the workflow semantic primitive universe;
2. the semantic type DAG (`SemanticTypeNode`);
3. exact, digest-pinned schema references (`TypeSchemaRef`);
4. value slots and primitive constants;
5. closed pure expressions;
6. logical resource slots.

Not in scope: workflow computation nodes (`INVOKE`, `AWAIT`, `SEQUENCE`,
`SELECT`, `PARALLEL`, `RACE`, `ITERATE`, `RETRY`, `TIMEOUT`, `HANDLE`, `SCOPE`,
`COMPENSATION_SCOPE`, `CALL_SUBFLOW`), `SemanticIRDocument`, dataflow edges,
executable plans, journal, reducer, runtime commands, resource handles and
resource bindings.

## 2. Machine primitives vs workflow primitives

<a id="semantic-workflow-primitive-set-closed"></a>

The AFIDL machine primitive universe has 13 members. The **workflow semantic
primitive universe is exactly 9**:

```text
unit  null  bool  int64  decimal  string  bytes  timestamp  duration
```

`value_no` is frozen in this order, 1..9:

| value_no | primitive |
|---|---|
| 1 | `unit` |
| 2 | `null` |
| 3 | `bool` |
| 4 | `int64` |
| 5 | `decimal` |
| 6 | `string` |
| 7 | `bytes` |
| 8 | `timestamp` |
| 9 | `duration` |

<a id="semantic-machine-only-primitives-prohibited"></a>

`int32`, `uint32`, `uint64` and `float64` are **machine-only**: they may appear
in AFIDL machine schemas (a `field_no: uint32` is fine) but MUST NOT surface as
workflow value types. Every point where a keyword enters the workflow type
layer goes through one closed gate that rejects them
(`unsupported_workflow_primitive`).

The two universes answer different questions and are never conflated:

```text
machine primitive   → how do machine objects express fields?
workflow primitive  → what can a value in a user workflow be?
```

## 3. Semantic type DAG

<a id="semantic-type-dag"></a>

AFIDL1 forbids recursive named schemas, so a self-recursive type expression
cannot be written in AFIDL source. The semantic core therefore represents
composite types as a **document-local indexed DAG**:

```text
SemanticTypeNode { id: SemanticTypeId, body: SemanticTypeNodeBody }

SemanticTypeNodeBody (variant) {
    primitive   : SemanticPrimitiveType
    schema      : TypeSchemaRef
    list        : SemanticTypeId
    map_string  : SemanticTypeId
    option      : SemanticTypeId
}
```

This expresses arbitrary finite nesting (`option<list<map<string,T>>>`) without
recursive schemas, without changing ACE1 and without weakening the Phase 2A
AFIDL recursion restriction.

<a id="semantic-type-dag-ids"></a>

`SemanticTypeId` is a u32 with `id > 0` and unique within the document; the
canonical type table is ordered by id ascending, and source order never affects
identity.

<a id="semantic-type-dag-acyclic"></a>

The DAG MUST be acyclic: `type1 = list<type1>`, directly or transitively, is
rejected (`semantic_type_cycle`). Workflow value types must expand finitely.

There is deliberately **no** `Any`, `Json`, `Dynamic`, `Unknown`, `Object` or
union/`one_of` case. Data that really is arbitrary must be modelled by an
explicit AFIDL schema; workflow unions are AFIDL variants.

## 4. TypeSchemaRef: exact pinning

<a id="semantic-schema-ref-pins-digest"></a>

```text
TypeSchemaRef {
    specification_id : string
    type_name        : string
    schema_digest    : DigestRefValue
}
```

A reference pins the exact `TypeSchemaDigest` it was authored against. A
published reference MUST NOT be re-resolved against "the latest" schema of that
name: `<SpecificationId>/<TypeName>` without a digest is never a workflow value
type.

<a id="semantic-schema-ref-digest-must-match"></a>

Validation resolves the reference against an explicit, read-only
`TypeSchemaUniverse`:

- the identity must exist in the universe (`unknown_type_schema`);
- the pinned digest must equal the canonical schema's actual digest
  (`type_schema_digest_mismatch`); a merely well-formed digest is not enough.

Builtin primitives are expressed through the `primitive` case of the type node;
a `TypeSchemaRef` MUST NOT be used to name a primitive.

## 5. DigestRefValue

<a id="semantic-digest-ref-value-shape"></a>

```text
DigestRefValue { algorithm: string, domain: string, hex: string }
```

This is the single AFIDL representation of a digest reference; every future
Plan/Journal/Value object reuses it, and a bare `digest: string` is forbidden.
Validation requires `algorithm == "sha256"`, `domain` to be a member of the
frozen HashV1 domain registry (for a `TypeSchemaRef`: exactly `type-schema`) and
`hex` to be exactly 64 lowercase hexadecimal characters.

The shape is deliberately a record, not an AFIDL primitive: no digest primitive
is added to the language.

## 6. Value slots

<a id="semantic-value-slot-immutable"></a>

```text
ValueSlot { id: ValueSlotId, type: SemanticTypeId }
```

A value slot is SSA: one immutable binding, one producer (defined in Phase 3B),
zero or more consumers. There is no mutation operation and no loop-carried
assignment; loop-carried values will appear as new value versions.

<a id="semantic-local-ids-positive-unique"></a>

Local ids in every namespace are `> 0` and unique within the document; `0` is
reserved as invalid/unset. Phase 3A adds no "reserved high" rules.

<a id="semantic-namespace-separation"></a>

`SemanticTypeId`, `ValueSlotId`, `ExpressionId` and `ResourceSlotId` are four
independent namespaces: identical numbers in two namespaces denote unrelated
objects. In Rust they are distinct newtypes, never bare `u32`.

## 7. Primitive constants

<a id="semantic-primitive-constants-closed"></a>

```text
PrimitiveConstant (variant) {
    unit, null, bool, int64, decimal, string, bytes, timestamp, duration
}
```

The cases map 1:1 onto `SemanticPrimitiveType`, so a constant carries no
redundant type field: the case determines the type. Machine-only primitives
must not appear as constant cases.

Composite values (records, variants, lists, maps, options) are built by
expressions, not by constants. There is no untyped literal.

`bytes` constants are canonical unpadded base64url text; `decimal` constants are
canonical decimal text (the ACE1 rules, reused verbatim).

## 8. Expressions

<a id="semantic-expression-pure"></a>

Expressions are pure and deterministic: no I/O, no clock, no randomness, no
environment, no policy or registry query, no resource access, no effect. They
are value-level expressions; workflow control is a separate (Phase 3B) concept
with the same name (`if_then_else` never controls task execution).

<a id="semantic-expression-closed-set"></a>

The Phase 3A body set is closed to:

```text
constant           value_slot
record_construct   variant_construct   enum_value
list_construct     map_construct
option_none        option_some
field_get
equals             not   and   or
if_then_else
```

Arithmetic, string operators, regex, indexing, `match_variant` / `is_case` /
`unwrap_case` and every fallible operation are deferred: the Phase 3A
expression language is total once well typed.

<a id="semantic-expression-dag"></a>

Expressions form a DAG: each lives in a table keyed by `ExpressionId`, may
reference later ids (forward references are legal) and MUST NOT cycle
(`expression_cycle`).

<a id="semantic-expression-references-valid"></a>

Every reference (expression id, value slot id, type id) must resolve;
unresolved references are rejected.

<a id="semantic-expression-static-type-correctness"></a>

The validator implements a real type checker (`infer_expression_type`), not
just a parser:

- `constant` derives its primitive type from the case;
- `value_slot` derives the slot's declared type;
- `record_construct` requires the pin to be a record schema, binds by
  `field_no`, requires exactly the declared field set with matching types, and
  rejects unknown/duplicate/missing bindings;
- `variant_construct` requires a variant schema and a matching payload type;
- `enum_value` requires an enum schema and an existing `value_no`;
- `list_construct` requires all elements to equal the declared element type;
- `map_construct` requires canonical keys and matching value types;
- `option_none` carries its inner type explicitly (nothing else could infer it);
- `option_some` derives `option<T>` from its payload;
- `field_get` requires a record schema input and yields the field's declared
  type by `field_no`;
- `equals` requires both operands to have equal types and that type to be
  comparable — Phase 3A admits primitive types and enum schemas, not records,
  lists, maps or options;
- `not` / `and` / `or` require bool operands and yield bool;
- `if_then_else` requires a bool condition and branch types that are equal.

The declared `result_type` is ABI clarity, not authority: the validator requires
`derived_type == declared_result_type` and rejects any divergence
(`expression_result_type_mismatch`).

## 9. Record and variant selection use numbers, not names

<a id="semantic-field-selection-by-number"></a>

Record construction and `field_get` select fields by `field_no`; variant
construction and enum selection use `case_no` / `value_no`. Numbers are the
durable schema ABI; names remain value-encoding keys (ACE1 still encodes record
members by field name and variant cases by case name).

<a id="semantic-map-key-rules"></a>

Map construction applies the ACE1 key invariants during validation, not at value
encoding time: keys are NFC-normalized, keys colliding after normalization are
rejected (`duplicate_normalized_map_key`), and entries are stored in unsigned
UTF-8 byte order.

## 10. Resource slots

<a id="semantic-resource-slot-separate-namespace"></a>

```text
ResourceSlot { id: ResourceSlotId, requirement_type: TypeSchemaRef }
```

A resource slot is a **logical requirement declaration**, never a database
connection, credential or live handle: values never travel through resource
slots and `ResourceSlotId` is a namespace separate from `ValueSlotId`
(`value slot 1` and `resource slot 1` may coexist).

<a id="semantic-resource-slot-immutable"></a>

A resource slot's definition is immutable for the life of the document.

<a id="semantic-resource-not-runtime-handle"></a>

No `ResourceHandle`, no binding snapshot and no runtime resource object exists
at this layer; deployment-time binding is a later stage.

`requirement_type` pins a resource *contract* schema and is therefore a
`TypeSchemaRef`, not a `SemanticTypeId`: a resource requirement lives outside
the document's value type DAG. The meaning of the contract (authority, effect
permissions, residency, binding) belongs to the Resource/Effect Contract stage.

## 11. Canonical ordering

<a id="semantic-canonical-ordering"></a>

Type nodes, value slots, expressions and resource slots are canonically ordered
by id ascending in the canonical model; source order never enters identity.
Document-level hashing does not exist yet (Phase 3B), but the ordering rules are
already normative so the future semantic IR digest reuses them unchanged.

## 12. No new digest identity

<a id="semantic-no-new-hash-domain"></a>

Phase 3A introduces no new HashV1 domain (`semantic-type`,
`semantic-expression` and similar names are forbidden). Type identity comes from
`TypeSchemaDigest`; constants, expressions and documents are ordinary ACE1
objects and Phase 3A performs no content-addressing of its own.

## 13. Conformance

Frozen corpus: `spec/conformance/semantic-core/` — the fixture schema universe
(`schema-universe.afidl`), golden documents with frozen canonical ACE1 bytes,
and negative documents with frozen expected error kinds, all generated by the
independent Python implementation `tools/gen-semantic-core-corpus.py`. Tests
never generate expected values at runtime.
