# AFIDL1 Exact Grammar — v1

> Status: AgentFlow AFIDL1 repository normative source (materialized in
> Phase 2A, extended with cross-module imports in Phase 2C). This document
> defines the AFIDL1 source grammar, the identity model, and the canonical
> TypeSchema machine representation that `TypeSchemaDigest` hashes.
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
  - `import-alias`: `[a-z][a-z0-9_]*` (never contains `-`)
- `AFIDL1` is the language major; the grammar version is declared in-file
  (see below) so a future `AFIDL2` can never be confused with a type's
  semantic major.
- Imports are an **additive AFIDL1 source-language capability**, not a new
  language major: every module that declares `afidl 1;` and no imports keeps
  its exact previous meaning, canonical bytes and digests.

## 2. Grammar

<a id="afidl-language-major"></a>
<a id="afidl-primitive-universe-closed"></a>

```text
module        := "afidl" MAJOR ";" schema-decl import-decl* declaration*

schema-decl   := "schema" SPECIFICATION-ID ";"     ; exactly one per module
import-decl   := "import" SPECIFICATION-ID "as" import-alias ";"
declaration   := alias-decl | enum-decl | record-decl | variant-decl

alias-decl    := "alias" TYPE-NAME "=" type-expr ";"

enum-decl     := "enum" TYPE-NAME "{" enum-value* "}"        ; >= 1 value
enum-value    := MAJOR-NUMBER lower-name ";"

record-decl   := "record" TYPE-NAME "{" field* "}"           ; may be empty
field         := MAJOR-NUMBER lower-name ":" type-expr ";"

variant-decl  := "variant" TYPE-NAME "{" variant-case* "}"   ; >= 1 case
variant-case  := MAJOR-NUMBER lower-name ":" type-expr ";"

type-expr     := primitive
               | TYPE-NAME                                   ; local reference
               | import-alias "." TYPE-NAME                  ; qualified reference
               | "list"   "<" type-expr ">"
               | "map"    "<" "string" "," type-expr ">"
               | "option" "<" type-expr ">"

primitive     := "unit" | "null" | "bool" | "int32" | "uint32" | "int64"
               | "uint64" | "decimal" | "float64" | "string" | "bytes"
               | "timestamp" | "duration"

SPECIFICATION-ID := ns-segment ( "." ns-segment )* "@" MAJOR
import-alias  := [a-z][a-z0-9_]*
ns-segment    := [a-z][a-z0-9-]*
MAJOR         := [1-9][0-9]*                                 ; positive, no leading zeros
MAJOR-NUMBER  := [1-9][0-9]*                                 ; 0 lexes but fails validation
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
- **Import placement is a header position**: `import-decl*` sits strictly
  between the schema declaration and the first type declaration. Imports may
  not be interleaved with declarations (`import_after_declaration`).
- A lowercase identifier immediately followed by `.` is a qualified
  reference; local declarations are UpperCamelCase, so the two forms never
  collide.

## 3. Identity model

<a id="afidl-specification-id-syntax"></a>

Three distinct layers, never conflated:

```text
AFIDL language major     afidl 1                 (grammar family)
Schema family identity   agentflow.example@1     (SpecificationId: namespace @ semantic major)
Type local name          User                    (UpperCamelCase, unique per compilation unit)
Import alias             core                    (source-only; never identity)
```

- Type full logical identity: `<SpecificationId>/<TypeName>`, e.g.
  `agentflow.example@1/User`. File paths are never part of identity, and
  neither is an import alias: `core.User` and `semantic.User` are two
  spellings of one type when both aliases point at one specification.
- A `SpecificationId` is **not** a HashV1 domain string; the frozen digest
  domain `type-schema` is a separate identifier.
- Per-type semantic majors do not exist in the source grammar: a type's major
  is inherited from its schema family. This keeps the grammar small and
  avoids undefined module/type major relationships.

## 4. Numbering: field_no / case_no / value_no

<a id="afidl-field-numbering-positive"></a>
<a id="afidl-field-numbering-unique"></a>
<a id="afidl-case-numbering-positive"></a>
<a id="afidl-case-numbering-unique"></a>
<a id="afidl-value-numbering-positive"></a>
<a id="afidl-value-numbering-unique"></a>
<a id="afidl-number-binding-semantic"></a>

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

## 5. References, imports, aliases and recursion

<a id="afidl-named-ref-resolution"></a>
<a id="afidl-import-exact-specification-id"></a>
<a id="afidl-import-alias-grammar"></a>
<a id="afidl-import-alias-unique"></a>
<a id="afidl-import-specification-unique"></a>
<a id="afidl-import-no-self"></a>
<a id="afidl-import-header-position"></a>
<a id="afidl-import-explicit-universe"></a>
<a id="afidl-import-no-registry-lookup"></a>
<a id="afidl-import-resolver-not-package-resolver"></a>
<a id="afidl-qualified-ref-resolution"></a>
<a id="afidl-external-ref-qualified"></a>
<a id="afidl-qualified-ref-not-primitive"></a>
<a id="afidl-imports-excluded-from-identity"></a>
<a id="afidl-alias-and-order-nonsemantic"></a>
<a id="afidl-cross-module-type-graph-acyclic"></a>
<a id="afidl-import-graph-may-cycle"></a>
<a id="afidl-alias-cycles"></a>
<a id="afidl-recursion-restriction"></a>

- A local `NamedRef` resolves within the single compilation unit (a set of
  files sharing one schema identity may be merged before compilation). Unknown
  identifiers in type position become named references and fail resolution if
  undeclared — the parser never speculatively accepts unknown primitives.
- `alias Name = TypeExpr;` creates a *new nominal schema identity*; alias
  identity != target identity even when their ACE1 value representations
  coincide. Alias cycles (direct or indirect) are rejected.

### 5.1 Cross-module imports

- Cross-module reuse happens only through an explicit import (no implicit
  import, no wildcard, no path import, no re-export, no per-type alias import):

  ```text
  import <SpecificationId> as <import-alias>;
  ```

- The imported specification is named by its **exact** `SpecificationId`,
  major included. `latest`, `^1`, `>=1`, `1.x` and `*` are not part of the
  language: an AFIDL import is not a version range and AFIDL1 has no package
  resolver.
- An import alias is unique within the module and is not a reserved keyword
  (`afidl`, `schema`, `import`, `as`, the declaration keywords, `list`, `map`,
  `option` and the 13 primitive keywords): `invalid_import_alias` /
  `duplicate_import_alias`.
- A module holds **at most one alias per specification family**
  (`duplicate_import_specification`), so one specification has exactly one
  source spelling. Importing the module's own schema id is rejected
  (`self_import`).
- Imports belong to the module header: an `import` after a declaration is
  rejected (`import_after_declaration`).
- Import resolution reads an **explicit, immutable, in-memory** schema
  universe supplied by the caller — a `SpecificationId → compiled module`
  map. The compiler never queries a registry for the latest version of a name,
  never reads the filesystem and never fetches anything. Version negotiation,
  package download, dependency solving, lock files and yank/revoke belong to a
  future Resolver / LockGraph stage; this resolution is exact-identity lookup
  only.
- An unused import is legal. It is a source-level dependency declaration, and
  it does not affect any digest.

### 5.2 Qualified references

- An external type is referenced **only** through a qualified reference
  `<import-alias>.<TYPE-NAME>`. An unqualified name never resolves to an
  imported type — not even when the local module declares no type of that
  name (`unknown_type_reference`) — so import collisions, shadowing and
  resolution-order dependence cannot arise.
- A qualified reference resolves to exactly one concrete type: the alias must
  be declared by the module (`unknown_import_alias`), the aliased
  specification must be present in the supplied universe
  (`unknown_import_specification`), and that module must declare the type
  (`unknown_imported_type`). There is no fallback to a local reference.
- A qualified reference names a named AFIDL declaration only; qualified
  primitives (`core.int64`) are illegal (`qualified_primitive_reference`):
  builtin primitives are always spelled bare.
- A qualified reference exists in the **parser AST** only. After resolution it
  reduces to the existing canonical `NamedRef` carrying the target's full
  logical identity; the canonical model has no qualified form.

### 5.3 Imports are not identity material

- The canonical `TypeSchema` record has exactly three members — `schema`,
  `name`, `body`. Import declarations never enter it, and neither does an
  import alias.
- A qualified reference lowers to the imported specification's full logical
  identity, so an alias rename, a reordering of import declarations, or an
  unused import leaves every `SourceTypeSchema` byte-identical:
  `same ACE1 bytes, same TypeSchemaDigest`.
- Import declarations do not pin the target's `TypeSchemaDigest`. The frozen
  AFIDL1 named-reference model is a **logical type identity**
  (`<SpecificationId>/<TypeName>`), not identity-plus-digest: schema evolution
  and compatibility are managed by the future Schema Version / Compatibility
  Edge / Upcaster system, and inlining a target digest would cascade identity
  changes through every upstream schema. Exact digest pinning is a *workflow
  value layer* concern (Phase 3A `TypeSchemaRef`), a different layer entirely.

### 5.4 Recursion

- **Phase 2A scoped restriction (not a frozen-document requirement):**
  direct or indirect recursive named references — through record fields,
  variant cases, alias expansion, or any `list`/`map`/`option` wrapper — are
  rejected. Inline structural recursion cannot produce finite ACE1 values, and
  cyclic canonical schema hashing is out of scope.
- Imports must not become a way around that restriction. The authoritative
  check is the **global named-type dependency graph** over a module set —
  `<SpecificationId>/<TypeName>` → referenced named types, covering alias
  targets, record fields, variant payloads and every wrapper — which must be
  acyclic (`cross_module_recursive_type`).
- The *module import graph*, by contrast, is **allowed** to contain cycles: two
  modules may import each other as long as no type in either depends on a type
  in the other. Requiring an acyclic import graph would reject legal schemas.
- A single module compiled against an already-compiled universe verifies its
  own declarations against the dependencies it can see. An isolated module
  cannot prove that a future unknown module will not refer back to it; only
  the module-set validation can, and it is the module set that carries the
  cross-module acyclicity guarantee.

## 6. Canonical TypeSchema machine representation

<a id="afidl-source-order-nonsemantic"></a>

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
  decoupled. A qualified source reference (`core.Slot`) reifies to
  `agentflow.semantic.core@1/Slot`: import aliases are source spelling and
  never appear here.
- Source spans, comments, whitespace, declaration order, member source
  order and import declarations do not exist in this model.

## 7. TypeSchemaDigest

<a id="afidl-type-schema-digest-formula"></a>

```text
TypeSchemaDigest = HashV1("type-schema", ACE1(CanonicalTypeSchema))
                 = SHA256("AF\0" || "type-schema" || "\0" || canonical_bytes)
```

The domain `type-schema` is the Phase 1 frozen registry entry; Phase 2A begins
using it. Digests are computed per named type. A module-level digest is
deferred (see `docs/implementation/phase-02a-decisions.md`).

Phase 2C adds no domain and changes no digest material: `HashV1`'s domain
registry is untouched, and because imports are excluded from the canonical
`TypeSchema`, every digest that existed before imports existed is still
reproduced byte-for-byte.

## 8. Conformance

Frozen corpus: `spec/conformance/afidl/` — parser-valid sources,
parser-invalid sources with expected error kinds (`parser-invalid/manifest.json`),
frozen canonical ACE1 bytes (`schema-golden.json`) and frozen digests
(`type-schema-digest-golden.json`), all generated by an independent Python
implementation (`tools/gen-afidl-corpus.py`). Tests never generate expected
values at runtime.

Cross-module conformance lives alongside it: `parser-valid/import-*.afidl`
fixtures with their import-target source named by a `requires` field, the
alias/order/unused equivalence groups, and the import negative cases in
`parser-invalid/` (`invalid-import-spec-id`, `unknown-import-specification`,
`duplicate-import-alias`, `duplicate-import-specification`, `self-import`,
`reserved-import-alias`, `qualified-ref-without-import`, `unknown-import-alias`,
`unknown-imported-type`, `qualified-primitive-ref`, `import-after-declaration`,
`malformed-import`) plus the multi-module cycle fixtures under
`spec/conformance/afidl/import-cycles/`.
