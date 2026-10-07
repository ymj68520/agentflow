# AFIDL1 Exact Grammar — v1

> Status: AgentFlow AFIDL1 repository normative source (materialized in
> Phase 2A, superseding the Phase 1A provisional declaration file). This
> document defines the AFIDL1 source grammar, the identity model, and the
> canonical TypeSchema machine representation that `TypeSchemaDigest` hashes.
> Grammar-changing edits require an explicit specification/version decision
> and a corpus update; silent edits are forbidden.
>
> AFIDL1 is a small, closed schema language for machine shapes. It is not a
> programming, macro, expression, or annotation language.

## 1. Lexical rules

- Source encoding: UTF-8, but **syntax and identifiers are ASCII**. Non-ASCII
  bytes outside comments are lex errors. (ACE1 *value* strings are Unicode;
  that rule lives at the value layer, not the schema layer.)
- Whitespace (space, tab, CR, LF) is insignificant and never reaches the AST.
- Comments: `// ...` to end of line, and `/* ... */` (non-nesting; unterminated
  is a lex error). Comments never enter the AST or the canonical schema.
- Integers: `[0-9]+`; leading zeros are lex errors; values must fit `u32`.
- Identifiers (ASCII, no case folding, no normalization):
  - `TYPE-NAME`   : `[A-Z][A-Za-z0-9]*`
  - `lower-name`  : `[a-z][a-z0-9_]*` (fields, cases, enum values)
  - `ns-segment`  : `[a-z][a-z0-9-]*` (schema id namespaces)
- `AFIDL1` is the language major; the grammar version is declared in-file
  (see below) so a future `AFIDL2` can never be confused with a type's
  semantic major.

## 2. Grammar

```text
module        := "afidl" MAJOR ";" schema-decl declaration*

schema-decl   := "schema" SPECIFICATION-ID ";"     ; exactly one per module
declaration   := alias-decl | enum-decl | record-decl | variant-decl

alias-decl    := "alias" TYPE-NAME "=" type-expr ";"

enum-decl     := "enum" TYPE-NAME "{" enum-value* "}"        ; >= 1 value
enum-value    := MAJOR-NUMBER lower-name ";"

record-decl   := "record" TYPE-NAME "{" field* "}"           ; may be empty
field         := MAJOR-NUMBER lower-name ":" type-expr ";"

variant-decl  := "variant" TYPE-NAME "{" variant-case* "}"   ; >= 1 case
variant-case  := MAJOR-NUMBER lower-name ":" type-expr ";"

type-expr     := primitive
               | TYPE-NAME                                   ; named reference
               | "list"   "<" type-expr ">"
               | "map"    "<" "string" "," type-expr ">"
               | "option" "<" type-expr ">"

primitive     := "unit" | "null" | "bool" | "int32" | "uint32" | "int64"
               | "uint64" | "decimal" | "float64" | "string" | "bytes"
               | "timestamp" | "duration"

SPECIFICATION-ID := ns-segment ( "." ns-segment )* "@" MAJOR
ns-segment     := [a-z][a-z0-9-]*
MAJOR          := [1-9][0-9]*                                 ; positive, no leading zeros
MAJOR-NUMBER   := [1-9][0-9]*                                 ; 0 lexes but fails validation
```

Notes:

- `MAJOR` after `afidl` must be exactly `1` in AFIDL1; other values are
  rejected (`invalid_language_version`).
- The schema id is assembled from tokens, so insignificant whitespace inside
  it is tolerated at the source layer; the assembled form must still pass the
  strict `SpecificationId` validation (`invalid_specification_id` otherwise).
- A second `schema` declaration is rejected (`duplicate_schema_declaration`);
  unknown declaration keywords are rejected (`parse_error`); trailing tokens
  after the declarations are rejected.
- `map` keys are exactly `string` — there is no generic `map<K,V>`.
- Field/case/value names may reuse the composite keywords (`list`, `map`,
  `option`) and primitive keywords as *names*; only type-expression positions
  interpret them structurally.

## 3. Identity model

Three distinct layers, never conflated:

```text
AFIDL language major     afidl 1                 (grammar family)
Schema family identity   agentflow.example@1     (SpecificationId: namespace @ semantic major)
Type local name          User                    (UpperCamelCase, unique per compilation unit)
```

- Type full logical identity: `<SpecificationId>/<TypeName>`, e.g.
  `agentflow.example@1/User`. File paths are never part of identity.
- A `SpecificationId` is **not** a HashV1 domain string; the frozen digest
  domain `type-schema` is a separate identifier.
- Per-type semantic majors do not exist in the source grammar: a type's major
  is inherited from its schema family. This keeps the grammar small and
  avoids undefined module/type major relationships.

## 4. Numbering: field_no / case_no / value_no

- `field_no`, `case_no`, and `value_no` are explicit `u32` numbers in the
  source; `0` is invalid (`1..=UINT32_MAX`).
- They are durable ABI, NOT source order and NOT display metadata:
  - unique within their declaration (numbers and names independently);
  - canonical member order is **number ascending**, regardless of source
    order;
  - binding a number to a different name/type/payload changes schema
    identity, and reordering source lines with unchanged number bindings does
    not.
- Within one published semantic-major family a retired number MUST NOT be
  reused for different meaning. Phase 2A enforces uniqueness inside a single
  compilation unit; cross-version non-reuse enforcement belongs to the future
  compatibility stage.
- Enum values carry `value_no` (Phase 2A decision, recorded in
  `docs/implementation/phase-02a-decisions.md`): enum evolution gets the same
  stable-identity treatment as fields and cases. Numbers identify the schema;
  they never replace the ACE1 value representation (enum values encode as
  their name strings; variant cases encode as `{"<case>":payload}` with the
  case *name*, per Phase 1A).
- Empty `record` bodies are legal (they encode `{}`). Empty `variant` and
  `enum` are rejected (uninhabited; no value could ever be encoded).

## 5. References, aliases and recursion

- `NamedRef` resolves within the single compilation unit (a set of files
  sharing one schema identity may be merged before compilation). There is no
  registry, package resolver, or remote import in AFIDL1.
- Unknown identifiers in type position become named references and fail
  resolution if undeclared — the parser never speculatively accepts unknown
  primitives.
- `alias Name = TypeExpr;` creates a *new nominal schema identity*; alias
  identity != target identity even when their ACE1 value representations
  coincide. Alias cycles (direct or indirect) are rejected.
- **Phase 2A scoped restriction (not a frozen-document requirement):**
  direct or indirect recursive named references — through record fields,
  variant cases, alias expansion, or any `list`/`map`/`option` wrapper — are
  rejected. Inline structural recursion cannot produce finite ACE1 values,
  and cyclic canonical schema hashing is out of scope for Phase 2A.

## 6. Canonical TypeSchema machine representation

The canonical model is a closed machine object encoded with ACE1 exactly as
frozen in Phase 1/1A (strings, canonical numeric strings, NFC, unsigned-byte
key ordering, explicit tagged options, no whitespace). Conceptual shape:

```text
TypeSchema (record) {
    schema : string                       ; canonical SpecificationId
    name   : string                       ; local type name
    body   : TypeBody                     ; closed variant
}

TypeBody (variant) {
    alias   : TypeExpr
    enum    : record { values : list<EnumValue> }   ; value_no ascending
    record  : record { fields : list<Field>   }     ; field_no ascending
    variant : record { cases  : list<Case>    }     ; case_no  ascending
}

EnumValue (record) { no : uint32, name : string }
Field     (record) { no : uint32, name : string, type : TypeExpr }
Case      (record) { no : uint32, name : string, type : TypeExpr }

TypeExpr (variant) {
    primitive  : string          ; the primitive keyword
    ref        : string          ; "<SpecificationId>/<TypeName>" full identity
    list       : TypeExpr
    map_string : TypeExpr
    option     : TypeExpr
}
```

- ACE1 record members are emitted in unsigned-UTF-8-byte key order by the
  encoder (e.g. `Field` appears as `{"name":...,"no":"N","type":...}`).
- `NamedRef` is reified as the referenced type's full logical identity
  string; targets are never inlined, so digests stay finite and modules stay
  decoupled.
- Source spans, comments, whitespace, declaration order and member source
  order do not exist in this model.

## 7. TypeSchemaDigest

```text
TypeSchemaDigest = HashV1("type-schema", ACE1(CanonicalTypeSchema))
                 = SHA256("AF\0" || "type-schema" || "\0" || canonical_bytes)
```

The domain `type-schema` is the Phase 1 frozen registry entry; Phase 2A begins
using it. Digests are computed per named type. A module-level digest is
deferred (see `docs/implementation/phase-02a-decisions.md`).

## 8. Conformance

Frozen corpus: `spec/conformance/afidl/` — parser-valid sources,
parser-invalid sources with expected error kinds (`parser-invalid/manifest.json`),
frozen canonical ACE1 bytes (`schema-golden.json`) and frozen digests
(`type-schema-digest-golden.json`), all generated by an independent Python
implementation (`tools/gen-afidl-corpus.py`). Tests never generate expected
values at runtime.
