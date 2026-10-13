# Phase 4C.1A — Exact Cross-Language Canonical Encoding

This candidate encoding is **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED ABI /
NOT IMPLEMENTED**. It uses existing ACE1 unchanged. No ordinary host JSON
serialization defines proof identity.

## Deterministic producer contract

1. Obtain the complete typed input values and their exact scopes/assets.
2. Reject noncanonical in-memory identities that cannot be losslessly projected.
3. Canonicalize declared *sets* using the exact keys below; detect duplicates
   before insertion into host maps. Preserve ordered sequences.
4. Validate local references, closed variants and exact numeric types.
5. Encode each field with its selected finite schema Shape using ACE1.

This is author-side construction. A canonical Decoder **rejects**, rather than
repairs, noncanonical received bytes. It never sorts an incoming list or merges
duplicate rows to recover a valid proof.

## Scalar and container rules

| Type | Exact bytes and validation |
| --- | --- |
| bool | true or false; declaration flags only where schema permits |
| uint32/uint64 | JSON string of canonical unsigned decimal digits; full width checked; no sign/leading zero except "0" |
| int64 | Canonical decimal string, -2^63..2^63-1; no Number/double conversion, +, -0 or exponent |
| decimal | Unbounded exact decimal string; no exponent; canonical trailing-zero removal before encoding; canonical Decoder rejects noncanonical text |
| timestamp / duration | Distinct nominal int64 microseconds, not interchangeable integer fields |
| float64 | Existing 16-lowercase-hex IEEE-754 form, finite/+0 only; not a workflow primitive constant |
| Text | Valid UTF-8 NFC; existing minimal ACE1 escapes; no BOM, raw controls, non-NFC repair or alternate escapes |
| Bytes | Unpadded base64url; reject padding, foreign alphabet, impossible length and noncanonical trailing bits |
| record | Every declared field once, sorted by unsigned UTF-8 NFC field-name bytes; unknown/missing/duplicate fields rejected |
| enum / variant | Closed exact name; one variant member only; unknown cases fail |
| option<T> | {"none":null} or {"some":<T>}; never omitted/null-sugar |
| seq<T> | Array in declared semantic order |
| set<T,key> | Array strictly increasing by declared primary key plus canonical element-byte tie-breaker; identical duplicates fail D1, identity conflicts fail D2 |
| map<string,T>, where imported type permits it | NFC key uniqueness, UTF-8 key order; no Any value and no last-write-wins |

unit and null both encode the literal null but remain different nominal types.
PrimitiveConstant uses the existing nine cases, including exact int64,
decimal, bytes, timestamp and duration. Machine int32/uint32/uint64/float64
metadata types do not become additional workflow primitives.

No insignificant whitespace/trailing bytes exist in the outer canonical value.
Canonical embedded IR/contract/schema/relation bytes obey the same rule.
Normative source and registry bytes are instead opaque Bytes: their original
whitespace/newlines/content remain exact file identity, not rewritten ACE1.

## Missing / None / empty matrix

| Representation | Meaning and outcome |
| --- | --- |
| Required record key missing | Shape error |
| Option field = {"none":null} | Explicit absence/unknown per that field's definition |
| Option list = {"some":[]} | Known present empty list |
| Option list = {"some":[...]} | Known present nonempty list |
| CompleteCollection = {"complete":[]} | Entire supplied universe is known empty |
| CompleteCollection = {"unavailable":"not_supplied"} | Required scope unavailable; cannot reconstruct positively |

None relation differs from Some(empty content-bound catalog). Unknown dependency
list differs from Some(empty). A zero occurrence upper differs from unbounded.
No overflow is encoded as infinity.

## Existing canonical schema-content grammar

SchemaAsset.canonical_content is the exact existing CanonicalTypeSchema
payload, not parser AST or AFIDL source. The finite recursive **blob grammar**
is:

~~~text
CanonicalSchema = record { schema: SpecId; name: TypeName; body: Body; }
Body = variant {
  alias: Expr;
  enum: record { values: list<record { no: U32; name: Text; }>; };
  record: record { fields: list<record { no: U32; name: Text; type: Expr; }>; };
  variant: record { cases: list<record { no: U32; name: Text; type: Expr; }>; };
}
Expr = variant {
  primitive: enum { unit; null; bool; int32; uint32; int64; uint64;
                    decimal; float64; string; bytes; timestamp; duration; };
  ref: Text;                 // full "<SpecificationId>/<TypeName>"
  list: Expr; map_string: Expr; option: Expr;
}
~~~

This is the already existing finite-value type-schema blob grammar
(`af-idl/src/canonical.rs:113–219`), not a new recursive AFIDL declaration.
The new Bundle carries it as Bytes, so its registered wrapper remains finite.
The blob parser recursively validates a finite tree under resource bounds.

Fields/cases/enum values sort by positive number ascending, with unique numbers
and names. Empty record is allowed; empty enum/variant is invalid. Named refs
preserve full logical identity; aliases retain nominal identity. Number
bindings contribute to existing TypeSchemaDigest even though ordinary value
record keys sort by name.

## Universal comparison keys

- Text: unsigned UTF-8 lexicographic, no locale/UTF-16 ordering.
- Numbers: mathematical unsigned/signed numeric ordering, not string ordering.
- DigestKey: (algorithm, domain, hex).
- SchemaRefKey: (specification_id, type_name, DigestKey(schema_digest)).
- TypedContentRefKey: (DigestKey(digest), SchemaRefKey(type_schema)).
- Catalog/operation/named refs: (catalog, name).
- Option keys: None before Some, then payload key.
- Tuple/list keys: lexicographic element comparison; prefix precedes extension.
- Equal primary keys: full canonical record bytes as the final tie-breaker;
  exact duplicate records still fail identity uniqueness.

Local Bundle tables sort by their positive IDs and never renumber on decode.
Independent primitive/type/schema/asset namespaces are not compared by
coincident integers.

## Exact mapping comparators

AtomKey is (operation.catalog, operation.name, requirement_port_no,
sorted scope(key,value), sorted full qualifier identity). Qualifier rank uses
the frozen family/member numbers plus complete payload: idempotency family 1,
natural member 1, none member 2. Unknown families are rejected.

Input/output/resource/effect/recovery/execution/subflow primary keys are the
tuples in the assertion-projection table. Resource source aliases and Phase 3C
dependency_slots are ascending numeric sets. Flow effect occurrence indices
are ascending uint64 sets and remain local to the leaf.

FlowMappingKey is:

~~~text
(class, optional export constraint, optional export source,
 source ContractPin DigestKey, source node, flow ordinal)
class = 0 if export_sink Some, 1 if only export_source Some, 2 if neither
constraint key = (FlowKind rank, FlowSink key)
FlowKind rank = data_egress 1, decision_influence 2
FlowSink key = workflow_output 1 + port, external_effect 2 + operation key
SourceCategory key = workflow_input 1 + port,
                     resource_requirement 2 + port, unknown_external 3
RecoveryActionKind rank = reconciliation 1, compensation 2
~~~

These are explicit schema semantics, not accidental host enum discriminants.
EffectGuard retains exactly unconditional/unit or input_equals with a uint32
port and existing typed PrimitiveConstant. It does not admit negation,
conjunction or program text as a guard. Source/action ordinals retain full
uint64 precision and are checked against the reconstructed vector lengths.

## Identities and equality

| Equality | Exact definition |
| --- | --- |
| Semantic projection equality | Expand table refs; compare full assertion/context/resolver/semantic-content values in their exact scopes |
| Proof verdict equality | Compare structured static prerequisites and nine dimension verdicts under conforming supported semantics |
| Evidence input identity equality | Exact canonical Bundle bytes, including local IDs, assets and full raw content identities |
| Diagnostic presentation equality | Separate text/span/format comparison; not required for static equality |

Isomorphic table relabeling can preserve the expanded semantic tuple while
changing evidence bytes. It is not the same exact typed packaging input.
Every one such typed input has exactly one canonical byte representation.

## Digest rules and decoder laws

Use existing HashV1 only for the actual existing IR/contract/type-schema
objects, with exact domains and full references. Relation/source/table checksums
are plain SHA-256 of their exact bytes. No standalone Bundle, Assertion,
RootContract or ProofContext digest is defined. Future parent identity requires
its own authorized canonical schema and complete inline inputs.

Required laws for both independent implementations:

~~~text
decode(encode(valid_typed_value)) = valid_typed_value
encode(decode(accepted_canonical_bytes)) = accepted_canonical_bytes
project(reconstruct(complete_supported_bundle)) = expanded input projection
~~~

Cross-language conformance must include extrema, large Int64 beyond 2^53,
arbitrary decimal precision, tagged options, NFC collisions, closed variants,
set/sequence differences, nominal aliases, digest domains and ordinal range
checks. A library failing existing ACE1 normalization/escaping is unsupported;
its binary identity does not waive this requirement.
