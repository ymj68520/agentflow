# Phase 2C — AFIDL1 Cross-Module Imports: Implementation Report

> Scope: AFIDL1 cross-module imports and qualified references. This phase does
> not enter the Workflow Semantic IR: no `SemanticIRDocument`, no plan, no
> journal, no runtime command, no kernel opcode and no resolver/lock graph was
> implemented.
>
> Decision register: [`phase-02c-afidl-imports-decisions.md`](phase-02c-afidl-imports-decisions.md)

## 1. Import grammar

```text
module        := "afidl" MAJOR ";" schema-decl import-decl* declaration*
import-decl   := "import" SPECIFICATION-ID "as" import-alias ";"
type-expr     := primitive
               | TYPE-NAME                        ; local reference
               | import-alias "." TYPE-NAME       ; qualified reference
               | "list" "<" type-expr ">"
               | "map" "<" "string" "," type-expr ">"
               | "option" "<" type-expr ">"
import-alias  := [a-z][a-z0-9_]*
```

Example:

```afidl
afidl 1;

schema agentflow.semantic.ir @1;

import agentflow.semantic.core@1 as core;

record Example {
    1 value: core.ValueSlotId;
    2 expr: core.ExpressionId;
}
```

- The import target is a full `SpecificationId` including the major. There is
  no version range, no `*`, no path form and no URL form: an AFIDL import is
  not a version range.
- Imports are a **header position**: they sit between the `schema` declaration
  and the first type declaration. An import after a declaration is
  `import_after_declaration`.
- `AFIDL1_VERSION` remains `1`. Imports are an additive source capability; a
  module with no imports has exactly its previous meaning.
- The reserved alias set is the closed set of structurally meaningful words
  (`afidl schema import as alias enum record variant`, `list map option` and
  the 13 primitive keywords), so an alias can never be confused with a
  structural token or a builtin.
- A lowercase identifier immediately followed by `.` is a qualified reference;
  local declarations are UpperCamelCase, so there is no ambiguity and no
  lookahead beyond one token.

## 2. Qualified reference model

- An external type is reachable **only** through `<alias>.<TYPE-NAME>`. A bare
  name is always local: it never resolves to an imported type, and a qualified
  reference never falls back to a local one.
- Three distinct, non-overlapping failures:
  | Situation | Error |
  |---|---|
  | alias not declared by the module | `unknown_import_alias` |
  | aliased specification not in the universe | `unknown_import_specification` |
  | module does not declare that type | `unknown_imported_type` |
- A qualified reference names a named declaration only: `core.int64` is
  `qualified_primitive_reference`, rejected at parse time so the diagnostic has
  the right span. Builtin primitives are always spelled bare.
- The parser AST carries `TypeExprKind::QualifiedNamedRef(QualifiedRef { import_alias, type_name, span })`.
  After resolution the form disappears: the canonical model only ever holds
  `CanonicalTypeExpr::NamedRef("<SpecificationId>/<TypeName>")`.
- When an unqualified name fails local resolution but an already-imported
  module declares that type, the diagnostic is upgraded to
  `unqualified_imported_reference`, which names the `alias.TypeName` spelling
  the author should have written. The local-only path (no universe) keeps
  reporting `unknown_type_reference`, because it cannot know the name exists
  elsewhere and must not guess.

## 3. `ResolvedSchemaUniverse` API

```rust
// The explicit, immutable, in-memory input. No filesystem, no network,
// no registry, no latest-version resolution.
pub struct ResolvedSchemaUniverse { /* SpecificationId -> CompiledSchema */ }
impl ResolvedSchemaUniverse {
    pub fn new() -> Self;
    pub fn insert(&mut self, module: CompiledSchema) -> Result<(), AfidlError>;
    pub fn get(&self, id: &str) -> Option<&CompiledSchema>;
    pub fn contains(&self, id: &str) -> bool;
    pub fn specification_ids(&self) -> impl Iterator<Item = &str>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}

// Additive entry points. `compile(source)` is unchanged for self-contained
// modules.
pub fn compile(source: &str) -> Result<CompiledSchema, AfidlError>;
pub fn compile_with_universe(
    source: &str,
    universe: &ResolvedSchemaUniverse,
) -> Result<CompiledSchema, AfidlError>;
pub fn compile_module_set(
    sources: &[(String, String)],
) -> Result<CompiledModuleSet, AfidlError>;

// The artifact of a module set: validated, acyclic, fully resolved.
pub struct CompiledModuleSet { /* SpecificationId -> CompiledSchema */ }
impl CompiledModuleSet {
    pub fn module(&self, id: &str) -> Option<&CompiledSchema>;
    pub fn modules(&self) -> impl Iterator<Item = &CompiledSchema>;
    pub fn type_by_identity(&self, identity: &str) -> Option<&CanonicalTypeSchema>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

The crate's only dependency is `af-core`, so the absence of a network or
registry lookup is a property of the dependency set, not of a convention.
No new crate was created: the capability lives in `af-idl` and `af-spec`.

## 4. Canonical `NamedRef` lowering rule

```text
source                          canonical NamedRef
------------------------------  ------------------------------------------
Foo                             <this-specification-id>/Foo
core.ValueSlotId                <imported-specification-id>/ValueSlotId
```

The lowering is a pure function of the module's own source: an import names an
exact specification identity, so resolution adds *validity*, never identity.
This has three consequences that the corpus freezes mechanically:

1. `CanonicalTypeSchema` still has exactly three members — `schema`, `name`,
   `body`. There is no imports member.
2. No import alias can appear inside a canonical type expression.
3. A module compiled against a universe produces byte-identical canonical
   schemas to the same module compiled standalone.

`tools/gen-afidl-corpus.py` asserts (3) directly for every import fixture: it
compiles each fixture through its module-set path and compares the resulting
canonical bytes and digests against the single-module values already frozen in
`schema-golden.json`.

## 5. Alias / order / unused identity tests

Three frozen equivalence groups, each proving one claim:

| Group | Members | Claim |
|---|---|---|
| import alias rename | `import-alias-rename-a`, `import-alias-rename-b` | one specification under aliases `core` and `semantic` → identical digest |
| import order | `import-order-a`, `import-order-b` | swapping two import declarations → identical digest |
| unused import | `import-unused-a`, `import-unused-b` | adding an unreferenced import → identical digest |

Both the Python generator and the Rust harness evaluate these groups, so the
claim is checked by two independent implementations. The Rust side additionally
verifies that a universe-compiled fixture reproduces its frozen single-module
bytes (`import_fixtures_compile_against_their_frozen_universe`) and that no
frozen identity contains an alias spelling
(`import_alias_lowering_is_frozen`).

`import-golden.json` freezes, per fixture, the resolvable alias → specification
table and the full logical identities its declarations reference. That table is
the machine statement of "the alias is source spelling and nothing else".

## 6. Cross-module recursion strategy

The authoritative check is the **global named-type dependency graph** over a
module set:

```text
<SpecificationId>/<TypeName>  ->  referenced named types
```

built from *canonical* type expressions, so it covers alias targets, record
fields, variant payloads and every `list`/`map`/`option` wrapper, and spans
modules because a `NamedRef` carries a full identity. Any cycle is
`cross_module_recursive_type`.

- The **module import graph may contain cycles.** Two modules may import each
  other as long as no type in either depends on a type in the other; requiring
  an acyclic import graph would reject legal schemas. The
  `mutual-import-without-type-cycle` corpus case freezes exactly that, and
  `compile_module_set` accepts it.
- A single module compiled against an already-compiled universe verifies its
  own declarations against the dependencies it can see. It **cannot** prove
  that an unknown future module will not refer back to it; the module set
  carries the acyclicity guarantee, and the implementation does not pretend
  otherwise.
- Two unit tests cover the two- and three-module cycles, plus a cycle through
  `list<option<…>>` and `map<string,…>` wrappers.

## 7. AFIDL rule additions

Eleven new rules, `AF-AFIDL-0016` .. `AF-AFIDL-0026`. **No pre-existing rule id
was renumbered, modified or removed** — verified by diffing the registry
against the previous revision (71 rules → 82 rules, all additions).

| Rule | Requirement |
|---|---|
| AF-AFIDL-0016 | exact `SpecificationId` imports only; no ranges/paths/wildcards |
| AF-AFIDL-0017 | explicit immutable in-memory universe; no registry/network/latest lookup |
| AF-AFIDL-0018 | alias grammar `[a-z][a-z0-9_]*`, not reserved, unique |
| AF-AFIDL-0019 | at most one alias per imported specification |
| AF-AFIDL-0020 | self-import rejected |
| AF-AFIDL-0021 | imports are a header position |
| AF-AFIDL-0022 | qualified references name declarations, never primitives |
| AF-AFIDL-0023 | external references must be qualified; no implicit import, no shadowing |
| AF-AFIDL-0024 | imports and aliases never enter canonical identity |
| AF-AFIDL-0025 | the global named-type graph must be acyclic |
| AF-AFIDL-0026 | the module import graph itself may cycle |

Four new conformance suites were registered: `SUITE-AFIDL-IMPORT-EXACT-IDENTITY`,
`SUITE-AFIDL-IMPORT-EXPLICIT-UNIVERSE`,
`SUITE-AFIDL-IMPORT-IDENTITY-ISOLATION` (covering the structural
three-member-record audit plus the equivalence groups) and, alongside them,
`SUITE-DIGEST-DOMAIN-SEPARATION` and the semantic suites are unchanged.

Coverage: 82 rules, 81 covered, 1 intentionally uncovered (the Phase 1 ACE1
architectural invariant), **0 missing coverage**.

## 8. AFIDL corpus revision and new cases

Corpus revision **3 → 4**; the separate parser-invalid manifest is at its own
revision **1 → 2**.

| Artifact | Before | After |
|---|---|---|
| valid cases | 32 | 43 (+11) |
| canonical types | 113 | 127 (+14) |
| digest cases | 113 | 127 (+14) |
| equivalence groups | 4 | 7 (+3) |
| parser-invalid cases | 33 | 45 (+12) |
| module-set cases | — | 3 (2 cycle, 1 mutual-import-valid) |

New valid fixtures: `import-core`, `import-second` (the import targets),
`import-basic`, `import-multiple`, `import-nested-container-ref`,
`import-alias-rename-{a,b}`, `import-order-{a,b}`, `import-unused-{a,b}`.

New negative fixtures (12): `invalid-import-spec-id`,
`unknown-import-specification`, `duplicate-import-alias`,
`duplicate-import-specification`, `self-import`, `reserved-import-alias`,
`qualified-ref-without-import`, `unknown-import-alias`, `unknown-imported-type`,
`qualified-primitive-ref`, `import-after-declaration`, `malformed-import`,
plus the module-set fixtures under `import-cycles/`.

A new frozen artifact, `import-golden.json`, records each fixture's resolved
import table and referenced identities. It exists because "the alias is not
part of the identity" is a claim about *lowering*, which the byte-level golden
alone cannot state.

The manifest gained an optional `mode` field for negatives that need a universe
(`unknown-imported-type`) rather than being weakened into parse errors. The 33
pre-existing cases keep their exact expectations.

## 9. Zero drift on old digests

Mechanically verified by comparing the regenerated corpus against the previous
revision:

```text
pre-existing valid cases          : 32  (all types byte-identical)
pre-existing canonical bytes      : 113 (byte-for-byte unchanged)
pre-existing TypeSchemaDigests    : 113 (all unchanged)
pre-existing mutation cases       : 10  (unchanged)
pre-existing equivalence groups   : 4   (unchanged, order preserved)
new cases                         : 11  (additive)
new canonical types               : 14  (additive)
```

The three universal-invariant diffs are empty:

```text
git diff --stat HEAD -- spec/encoding spec/digest spec/conformance/encoding \
    spec/conformance/digest spec/tables/hash-domains.json                  -> empty
git diff --stat HEAD -- crates/af-semantic                                 -> empty
git diff --stat HEAD -- spec/conformance/encoding/*.json (ACE1 corpus)      -> empty
```

The frozen encoder region of `canonical.rs` (`type_expr_shape` / `schema_shape`
— the ACE1 wire model of a canonical schema) is **not touched by the diff at
all**. Only the resolution layer around it changed, which is why the old bytes
cannot move.

## 10. Zero drift on semantic core

`spec/idl/semantic-core.afidl`, `spec/conformance/semantic-core/` and the whole
`crates/af-semantic` tree are byte-identical to the previous revision:

```text
git diff --stat HEAD -- spec/idl/semantic-core.afidl \
    spec/conformance/semantic-core crates/af-semantic   -> empty
```

All 29 `agentflow.semantic.core@1` `TypeSchemaDigest` values are reproduced
unchanged, and all 82 ABI assignment rows for that family are unchanged
(132 rows total, unchanged). The semantic crate's 26 tests pass untouched, and
its static surface checks (frozen 14-domain HashV1 registry, no new domain)
still pass.

The semantic core is *consumable* as an import target — the phase's fixtures
import a test module, and a static test imports `agentflow.semantic.core@1`
without modifying it.

## 11. `spec-check` results

```text
specifications: 12
normative rules: 82
rules covered: 81
rules intentionally uncovered: 1
rules missing coverage: 0
ABI assignments:
  active: 132
  retired: 0
  reserved: 0
AFIDL modules checked: 3
cross-reference errors: 0
```

Exit code `0`. `spec-check` is now import-aware and resolves the normative
module set rather than assuming each file is independently compilable:

1. discover every `.afidl` under `spec/idl`;
2. parse each header with the real grammar to obtain its `SpecificationId`
   (never guessed from a file name or a registry entry);
3. reject duplicate schema families (`duplicate_module_family`);
4. compile the set as a whole, resolving imports and validating the global type
   graph;
5. only then extract ABI assignments and cross-check them.

It remains read-only: it never rewrites or auto-fixes a registry file.

## 12. Workspace gates

```text
cargo fmt --all -- --check                                 OK
cargo clippy --all-targets --all-features -- -D warnings    OK (clean)
cargo test --all                                            159 passed, 0 failed
cargo run -p af-spec --bin spec-check -- check              exit 0
all five corpus generators, two consecutive runs            zero diff
```

Test count moved from 136 to 159: +10 in `af-idl` unit tests (import header,
alias grammar, qualified references, module sets, cycles), +4 in
`conformance_afidl` (universe compilation, lowering, equivalence groups,
module-set cycles) and +5 in the new `import_surface` static/behavioral suite
(no I/O capability, no new hash domain, three-member canonical record,
unchanged `compile`, no implicit import).

The registry grew additively: 71 → 82 rules, 132 ABI rows unchanged (imports
introduce no numbered member), 7 → 10 conformance suites, 12 specifications
(only the two entries backed by `spec/idl/afidl1-grammar.md` moved their
`ArtifactFileDigest`, as the phase anticipated).

## 13. Explicitly deferred

- **Package resolver / lock graph / version solving / compatibility edges.**
  Import resolution is exact-identity lookup only; version negotiation,
  download, dependency solving, lock files and yank/revoke are a later stage.
- **Module-level digest (`SchemaModuleDigest`).** Digests remain per named type,
  as in Phase 2A. Adding a module digest would be a new identity and is not
  needed for imports.
- **Re-exports, wildcards, per-type alias imports, path imports.** Each is
  syntactically absent, not merely rejected.
- **Merging several source files into one compilation unit.** Still not
  implemented; a module is one file in this phase's tooling.
- **Unused-import linter warning.** Unused imports are legal (and provably
  non-semantic); surfacing them as a warning is a linting concern.
- **`semantic_spec` as a `SpecificationKind`.** Unchanged from Phase 3A: adding
  a kind member would drift two frozen registry digests. The import capability
  is part of `agentflow.idl.afidl@1` instead, whose source digest moved as the
  phase expects.
- **Cross-module compatibility checking.** Which schema versions remain
  compatible across a reference is future work; this phase only guarantees
  that the graph is acyclic and every reference resolves.

## 14. Blockers for Phase 3B

None identified. The semantic IR will describe a machine schema that imports
`agentflow.semantic.core@1` by exact `SpecificationId` and refers to its types
through qualified references, which is now expressible end to end:

```afidl
afidl 1;

schema agentflow.semantic.ir @1;

import agentflow.semantic.core@1 as core;

record SemanticIrDocument {
    1 types: list<core.SemanticTypeNode>;
    2 values: list<core.ValueSlot>;
    3 expressions: list<core.ExpressionDef>;
    4 resources: list<core.ResourceSlot>;
}
```

The digest of such a module is a pure function of its own local declarations
plus the *identities* (not the contents) of the imported types, so the IR schema
can be frozen independently of any future change inside the semantic core. The
one thing Phase 3B must not assume is that a NamedRef pins a target digest: it
does not, and the `TypeSchemaRef` mechanism in the semantic core is the layer
that does pin.

## 15. Git overview

Categorized commits for this phase follow this report; the working tree is
otherwise clean. Files touched:

```text
crates/af-idl/src/{ast,canonical,error,lib,parser,validate}.rs   modified
crates/af-idl/src/resolve.rs                                     new
crates/af-idl/tests/{conformance_afidl.rs,import_surface.rs}      modified / new
crates/af-spec/src/{check,error}.rs                              modified
crates/af-spec/tests/conformance_spec_registry.rs                modified
spec/idl/afidl1-grammar.md                                       modified
spec/conformance/afidl/{schema-golden,type-schema-digest-golden}.json  regenerated (additive)
spec/conformance/afidl/import-golden.json                        new
spec/conformance/afidl/parser-valid/import-*.afidl               new (11)
spec/conformance/afidl/parser-invalid/*import*.afidl             new (9)
spec/conformance/afidl/import-cycles/*.afidl                     new (7)
spec/conformance/spec-registry/**                                regenerated (revision 2)
spec/registry/{rules,specifications}.json                        regenerated (additive)
tools/gen-afidl-corpus.py                                        modified
tools/gen-registry-instances.py                                  modified
tools/gen-spec-registry-corpus.py                                modified
docs/implementation/phase-02c-afidl-imports{,-decisions}.md      new
```

到此停止。不进入 Phase 3B。
