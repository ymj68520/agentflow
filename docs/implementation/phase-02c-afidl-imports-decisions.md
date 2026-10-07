# Phase 2C — AFIDL1 Cross-Module Imports: Decision Register

> Status: Phase 2C implementation record. Decisions here are binding on the
> Phase 2C implementation; they are *not* new frozen specification text. The
> normative source is `spec/idl/afidl1-grammar.md` sections 2 and 5.

Phase 2C adds exactly one capability to AFIDL1: a module may reference a named
type that another module already declares, through an explicit import. Nothing
about the canonical `TypeSchema` model, ACE1, HashV1 or any existing digest
changes.

## D-1 Import is an additive AFIDL1 source capability, not a new language

`AFIDL1_VERSION` stays `1`. The grammar production for `module` gains
`import-decl*` between the schema declaration and the declarations, which is a
pure widening of the accepted source language:

```text
module := "afidl" MAJOR ";" schema-decl import-decl* declaration*
```

Every source that was legal before is still legal with the same meaning, so the
frozen corpus, canonical bytes and digests are unaffected. A language-major
bump would have been a lie: no existing module changes meaning.

*Where:* `crates/af-idl/src/parser.rs` (`Parser::module`),
`spec/idl/afidl1-grammar.md` section 2.

## D-2 The imported specification is named by exact `SpecificationId`

`import <SpecificationId> as <alias>;` where `SpecificationId` is the frozen
`ns-segment("."ns-segment)*"@"MAJOR` form. There is no version range, no `*`,
no path, no URL, and no bare-name import. Version negotiation is not
implemented and cannot be expressed.

*Where:* `Parser::specification_id` is reused unchanged for imports.

## D-3 Qualified references only: no implicit import, no shadowing

An external type is reachable only as `<alias>.<TYPE-NAME>`. Consequences that
are enforced rather than merely intended:

- a bare name never resolves to an imported type, even when the local module
  declares nothing of that name;
- a local declaration and an imported type of the same name never collide,
  because an unqualified name is always local;
- resolution cannot depend on import order or on the number of imported
  modules;
- a qualified reference never falls back to a local lookup.

*Where:* `validate::resolve_refs` treats `QualifiedNamedRef` as opaque,
`resolve::resolve_qualified_against` resolves only the qualified form.

## D-4 One alias per specification; one alias namespace per module

Import aliases are unique within a module (`duplicate_import_alias`) and each
imported specification appears at most once (`duplicate_import_specification`),
so one specification has exactly one source spelling in a module. Self-import
is rejected (`self_import`).

*Where:* `validate::check_module_header`.

## D-5 Import aliases are syntax, never identity

`CanonicalTypeSchema` keeps exactly `schema` / `name` / `body`. There is no
imports member, and no import alias can appear in a `NamedRef`: a qualified
reference lowers to the *target's* full logical identity. This is what makes an
alias rename non-semantic.

*Where:* `canonical::schema_shape` is unchanged;
`canonical::resolve_expr` maps `core.Slot` → `agentflow.semantic.core@1/Slot`.

## D-6 Alias, order and unused imports are all non-semantic

Three frozen equivalence groups prove it:

| Group | Claim |
|---|---|
| `import-alias-rename-{a,b}` | the same type under aliases `core` and `semantic` has one digest |
| `import-order-{a,b}` | reordering two import declarations changes nothing |
| `import-unused-{a,b}` | an unused import does not change a type that never references it |

An unused import is deliberately *legal*: it is a source-level dependency
declaration, and a future linter may warn about it. It is not an error.

*Where:* `spec/conformance/afidl/schema-golden.json` equivalence groups,
`import_equivalence_groups_are_frozen`.

## D-7 Imports do not pin the target's digest

An AFIDL named reference is a **logical type identity**, not
identity-plus-digest. Inlining a target digest would cascade identity through
every upstream schema on any target change. Schema evolution belongs to the
future Schema Version / Compatibility Edge / Upcaster system.

Exact digest pinning exists in the system, but at a different layer:
`TypeSchemaRef` in the Phase 3A workflow value layer pins
`<SpecificationId>/<TypeName>` **plus** an exact `TypeSchemaDigest`, because a
workflow document must not silently re-resolve. The two are consistent: the
schema layer decouples, the workflow layer pins.

*Where:* `spec/idl/afidl1-grammar.md` section 5.3; D-14 below records the same
split for the ABI registry.

## D-8 Import position is a header position

`import-decl*` sits strictly between `schema-decl` and the first declaration.
An import appearing after any `alias`/`enum`/`record`/`variant` is rejected
(`import_after_declaration`). This keeps the module header a fixed, readable
prefix instead of a free-form interleaved block.

*Where:* `Parser::module` (`seen_declaration` flag).

## D-9 Reserved aliases are exactly the structurally meaningful words

The reserved set is closed and frozen:

```text
afidl schema import as
alias enum record variant
list map option
unit null bool int32 uint32 int64 uint64 decimal float64 string bytes
timestamp duration
```

These are precisely the words with meaning in a type or header position.
Reserving them avoids any parser ambiguity and any "alias that looks like a
builtin" confusion. A rejected alias is `invalid_import_alias`.

*Where:* `parser::RESERVED_IMPORT_ALIASES`, mirrored in the Python
implementation and frozen in the grammar document.

## D-10 Resolution takes an explicit, immutable, in-memory universe

`ResolvedSchemaUniverse` is a `SpecificationId → CompiledSchema` map supplied
by the caller. The crate has no filesystem, network, environment or clock
dependency at all (the only dependency is `af-core`), so a registry or
latest-version lookup is not merely discouraged but unrepresentable. An import
of a specification absent from the universe is
`unknown_import_specification`.

*Where:* `crates/af-idl/src/resolve.rs`;
`af-idl`'s dependency set is audited by a static test.

## D-11 This resolver is not a package resolver

Implemented: exact-identity lookup, alias resolution, missing-specification and
missing-type detection, duplicate-family detection, global type-graph
acyclicity.

Explicitly *not* implemented and out of scope: version negotiation, package
download, registry query, dependency solving, lock files, yank/revoke,
compatibility checking. Those belong to the future Resolver / LockGraph stage.

## D-12 The module set is a set: the import graph may cycle

`compile_module_set` parses every module before resolving anything, so two
modules may import each other. What must be acyclic is the **named-type
dependency graph**, not the import graph. Requiring an acyclic import graph
would reject legal schemas (two independent modules that happen to import each
other for unrelated reasons).

A set with a cyclic import graph but an acyclic type graph compiles; the
`mutual-import-without-type-cycle` corpus case freezes exactly that.

*Where:* `compile_module_set`, D-13.

## D-13 The global type graph is the authoritative recursion check

`<SpecificationId>/<TypeName>` → referenced named types, built from *canonical*
type expressions so that it covers alias targets, record fields, variant
payloads and every `list`/`map`/`option` wrapper, spanning modules because a
`NamedRef` carries a full identity. Any cycle is
`cross_module_recursive_type`.

A single module compiled against an already-compiled universe verifies its own
declarations against the dependencies it can see, but cannot prove that an
unknown future module will not refer back to it. The module set therefore
carries the acyclicity guarantee, and the phase does not pretend otherwise.

*Where:* `resolve::check_global_recursion`; corpus cases
`cross-module-direct-cycle` (2 modules) and `cross-module-indirect-cycle`
(3 modules, `A → B → C → A`).

## D-14 ABI numbers are not affected by imports

Import resolution introduces no new normative record or variant, so no ABI
number is added. The ABI registry is derived from AFIDL declarations
(`tools/gen-registry-instances.py`); the import fixtures live in the
conformance corpus, not in `spec/idl`, and are therefore never ABI-scanned.

*Where:* `spec/registry/abi-numbers.json` is unchanged at 132 rows.

## D-15 `spec-check` resolves the normative module set, not files

`spec-check` reads every `.afidl` file under `spec/idl`, parses its header with
the real grammar to obtain its `SpecificationId`, rejects duplicate families,
and then compiles the set as a whole. It never resolves an import by file name
or by a registry lookup, and a duplicate schema family is reported as
`duplicate_module_family`.

The spec-registry corpus case root is now walked rather than read from a fixed
single-file list, so a case may hold multiple modules.

*Where:* `crates/af-spec/src/check.rs` step 6; `Layout::corpus_case`.

## D-16 Qualified references to primitives are rejected at parse time

`core.int64` is not a type expression. Builtin primitives are always spelled
bare, and no module can export one. Rejecting it in `type_expr` (rather than at
resolution) means the error is reported against the offending source position
with the right span.

*Where:* `Parser::qualified_ref` → `qualified_primitive_reference`.

## D-17 An unqualified reference that names an imported type gets a precise error

When the universe is available and a bare name fails local resolution but an
*already-imported* module declares that type, the local-only
`unknown_type_reference` is upgraded to
`unqualified_imported_reference`, which names the qualified spelling the author
should have used. Without a universe, the plain unresolved-reference error is
reported: the local-only path has no way to know the name exists elsewhere and
must not guess.

*Where:* `resolve::explain_unqualified_reference`; used by
`compile_with_universe` and `compile_module_set`.

## D-18 Error kinds are additive and stable

Eleven new stable kinds are introduced, none of which renames an existing one:
`invalid_import_alias`, `duplicate_import_alias`,
`duplicate_import_specification`, `self_import`,
`unknown_import_specification`, `unknown_import_alias`,
`unknown_imported_type`, `qualified_primitive_reference`,
`import_after_declaration`, `unqualified_imported_reference`,
`cross_module_recursive_type`, plus `duplicate_module_specification` for a
module set. The pre-existing `recursive_type_not_supported` continues to mean
intra-module recursion; cross-module cycles use the new, more precise kind.

*Where:* `crates/af-idl/src/error.rs` `kind` module.

## D-19 `compile(source)` is untouched

The pre-existing entry point keeps its exact signature and behaviour. Modules
without imports compile identically through `compile` and
`compile_with_universe` (asserted by test, with an empty universe). The
import-aware entry points are strictly additive.

*Where:* `crates/af-idl/src/lib.rs`; static test
`the_pre_existing_compile_entry_point_is_unchanged`.

## D-20 Imports introduce no HashV1 domain

No `schema-import`, `schema-module` or `schema-universe` domain exists, and the
frozen 14-entry registry is unchanged. The reason is D-5: imports do not enter
identity, so there is nothing new to hash.

*Where:* `spec/tables/hash-domains.json` is byte-identical; static test
`imports_introduce_no_new_hash_domain`.

## D-21 The `parser-invalid` manifest gained a mode, not a shape change

Import negatives that need a universe to be meaningful (`unknown_imported_type`)
cannot be exercised by `compile(source)` alone. Rather than weakening them into
parse errors, the manifest gained an optional `mode` field (`"compile"` default,
`"universe"`) and a separate `module_set_cases` list for the cycle cases. The
pre-existing 33 cases keep their exact expectations; the revision moved 1 → 2
because the corpus grew, not because anything was redefined.

*Where:* `spec/conformance/afidl/parser-invalid/manifest.json`.

## D-22 The Python implementation mirrors the resolver independently

`tools/gen-afidl-corpus.py` implements the lexer/parser extension, the import
table, qualified-reference lowering, the module-set resolver and the global
type graph itself. It never calls the Rust implementation to produce expected
values. It additionally cross-checks that module-set compilation of every
import fixture reproduces the single-module canonical bytes, which is the
mechanical proof of the identity-isolation claim.

*Where:* `tools/gen-afidl-corpus.py`; output `import-golden.json`.

## D-23 Deferred beyond Phase 2C

Explicitly *not* implemented, recorded so the omission is visible:

- a package resolver, lock graph, version solving or compatibility edges;
- module-level (`SchemaModuleDigest`) digests — still per-type only;
- re-exports, wildcards, per-type alias imports and path imports;
- a linter warning for unused imports;
- merging several files into one compilation unit;
- registering the import capability as its own `SpecificationKind` — the
  capability is part of `agentflow.idl.afidl@1`, whose source digest changed,
  matching the phase's expectation that only that entry's digest moves.
