//! `af-idl` — the AFIDL1 core schema language.
//!
//! Phase 2A foundation: every later long-term ABI (Semantic IR, Plan, Journal
//! Event, Runtime Command, ...) is described in AFIDL1 and hashed through this
//! pipeline:
//!
//! ```text
//! AFIDL1 source → parser AST → validation → CanonicalTypeSchema
//!              → ACE1 bytes → HashV1("type-schema", ...) → TypeSchemaDigest
//! ```
//!
//! Phase 2C adds cross-module reuse as an *additive source-language*
//! capability: `import <SpecificationId> as <alias>;` plus qualified type
//! references (`alias.TypeName`). Imports are a source resolution aid only —
//! they never enter a canonical TypeSchema, so an alias rename, a reordering
//! of import declarations or an unused import leaves every `TypeSchemaDigest`
//! unchanged. A module that imports is compiled with
//! [`compile_with_universe`] / [`compile_module_set`], which take an explicit,
//! immutable [`ResolvedSchemaUniverse`]; there is no registry, latest-version
//! or filesystem lookup anywhere in this crate.
//!
//! Layering:
//!
//! - [`parse`] — hand-written lexer + recursive-descent parser producing the
//!   source AST with [`SourceSpan`]s (diagnostics only, never identity);
//! - [`validate`] — fail-closed structural, resolution, alias-cycle and
//!   recursion checks;
//! - [`canonical`] — the canonical machine model; canonical bytes via ACE1
//!   and `TypeSchemaDigest = HashV1("type-schema", ACE1(schema))`.
//!
//! Normative artifacts: `spec/idl/afidl1-grammar.md` (exact grammar),
//! `spec/idl/core/machine-primitives.afidl` (builtin universe registry),
//! `spec/tables/afidl-primitives.json` (closed primitive table) and the
//! frozen corpus under `spec/conformance/afidl/`.

pub mod ast;
pub mod canonical;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod primitives;
pub mod resolve;
pub mod shape;
pub mod span;
pub mod specification;
pub mod validate;

pub use ast::{
    AliasDecl, Decl, EnumDecl, EnumValueDecl, FieldDecl, ImportDecl, Module, QualifiedRef,
    SchemaDecl, TypeExpr, TypeExprKind, VariantCaseDecl, VariantDecl,
};
pub use canonical::{
    CanonicalBody, CanonicalCase, CanonicalEnumValue, CanonicalField, CanonicalTypeExpr,
    CanonicalTypeSchema, CompiledSchema,
};
pub use error::AfidlError;
pub use parser::{RESERVED_IMPORT_ALIASES, parse};
pub use primitives::{AFIDL1_VERSION, Primitive};
pub use resolve::{
    CompiledModuleSet, DeclaredTypeIndex, ResolvedSchemaUniverse, compile_module_set,
};
pub use span::SourceSpan;
pub use specification::SpecificationId;
pub use validate::validate;

/// Parses, validates and canonicalizes an AFIDL1 source.
///
/// This is the primary entry point for a self-contained module (no imports).
/// Deterministic: the same source (and any source differing only in comments,
/// whitespace or declaration/member ordering with identical number bindings)
/// yields identical canonical schemas and identical digests.
///
/// # Errors
/// Returns a structured [`AfidlError`] for lex, parse and validation
/// failures, each carrying the source span of the offending construct.
pub fn compile(source: &str) -> Result<CompiledSchema, AfidlError> {
    let module = parse(source)?;
    validate(&module)?;
    Ok(canonicalize(&module))
}

/// Parses, validates and canonicalizes an AFIDL1 source against an explicit,
/// immutable schema universe — the import-aware companion of [`compile`].
///
/// Beyond everything [`compile`] checks, this verifies that every import names
/// a specification the universe contains and that every qualified reference
/// resolves to a type that specification actually declares. Local-only
/// modules compile identically under both entry points.
///
/// # Errors
/// Returns a structured [`AfidlError`]; import-resolution failures use
/// [`AfidlError::UnknownImportSpecification`] and
/// [`AfidlError::UnknownImportedType`].
pub fn compile_with_universe(
    source: &str,
    universe: &ResolvedSchemaUniverse,
) -> Result<CompiledSchema, AfidlError> {
    let module = parse(source)?;
    if let Err(error) = validate(&module) {
        if let Some(upgraded) = resolve::explain_unqualified_reference(
            &module,
            &resolve::DeclaredTypeIndex::from_universe(universe),
            &error,
        ) {
            return Err(upgraded);
        }
        return Err(error);
    }
    resolve::resolve_against_universe(&module, universe)?;
    Ok(canonicalize(&module))
}

/// Canonicalizes a *validated* module into its canonical types.
///
/// Canonical member ordering is by explicit number ascending (`field_no`,
/// `case_no`, `value_no`); the type list is sorted by local name. Source
/// declaration order never survives canonicalization, and neither does a
/// module's import table: a qualified reference lowers to the imported
/// specification's full logical identity.
///
/// The function is infallible for a module that passed [`validate`], which
/// already rejects a qualified reference whose alias is not declared.
pub fn canonicalize(module: &Module) -> CompiledSchema {
    canonical::build_schema(module, &|expr| {
        canonical::resolve_from_imports(module, expr)
    })
    .expect("a validated module always canonicalizes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::kind;

    const SAMPLE: &str = "afidl 1;\n\
        schema agentflow.example @1;\n\
        alias UserId = string;\n\
        enum Status { 1 pending; 2 complete; }\n\
        record User { 1 id: UserId; 2 name: string; 3 status: Status; }\n\
        variant LookupResult { 1 found: User; 2 missing: unit; }\n";

    #[test]
    fn compiles_the_sample_module() {
        let compiled = compile(SAMPLE).unwrap();
        assert_eq!(compiled.specification.canonical(), "agentflow.example@1");
        assert_eq!(compiled.types.len(), 4);
        let user = compiled.by_name("User").unwrap();
        assert_eq!(user.full_identity(), "agentflow.example@1/User");
        // Canonical field order is field_no ascending, not source order.
        let CanonicalBody::Record(fields) = &user.body else {
            panic!("record body expected");
        };
        let nos: Vec<u32> = fields.iter().map(|f| f.no).collect();
        assert_eq!(nos, vec![1, 2, 3]);
        assert!(user.full_identity().ends_with("/User"));
    }

    #[test]
    fn formatting_and_member_order_do_not_change_digest() {
        let a = compile(SAMPLE).unwrap();
        let reordered = compile(
            "afidl 1;\n// reflowed\nschema  agentflow.example  @1 ;\n\
             variant LookupResult { 2 missing: unit; 1 found: User; }\n\
             record User { 3 status: Status; 1 id: UserId; /* note */ 2 name: string; }\n\
             enum Status { 2 complete; 1 pending; }\n\
             alias UserId = string;\n",
        )
        .unwrap();
        assert_eq!(a.types.len(), reordered.types.len());
        for (x, y) in a.types.iter().zip(reordered.types.iter()) {
            assert_eq!(x.name, y.name);
            assert_eq!(
                x.ace1_bytes().unwrap(),
                y.ace1_bytes().unwrap(),
                "type {}",
                x.name
            );
            assert_eq!(x.digest().unwrap(), y.digest().unwrap(), "type {}", x.name);
        }
    }

    #[test]
    fn semantic_mutations_change_digest() {
        let base = compile(SAMPLE).unwrap();
        let renamed = compile(
            SAMPLE
                .replace("2 name: string;", "2 full_name: string;")
                .as_str(),
        )
        .unwrap();
        let base_user = base.digest("User").unwrap();
        let renamed_user = renamed.digest("User").unwrap();
        assert_ne!(base_user, renamed_user);

        let renumbered = compile(
            SAMPLE
                .replace("2 name: string;", "4 name: string;")
                .as_str(),
        )
        .unwrap();
        assert_ne!(base_user, renumbered.digest("User").unwrap());
    }

    #[test]
    fn digest_uses_the_frozen_type_schema_domain() {
        let compiled = compile(SAMPLE).unwrap();
        let digest = compiled.digest("UserId").unwrap();
        assert_eq!(digest.domain(), af_core::DigestDomain::TypeSchema);
        assert_eq!(digest.hex().len(), 64);
        // Exact expected bytes are frozen in the Python-generated corpus;
        // here we only verify the preimage formula structure.
        let bytes = compiled.by_name("UserId").unwrap().ace1_bytes().unwrap();
        let expected = af_core::hash_v1(af_core::DigestDomain::TypeSchema, &bytes);
        assert_eq!(digest.raw_bytes(), expected);
    }

    #[test]
    fn rejects_unknown_declarations_and_trailing_tokens() {
        let e = compile("afidl 1; schema a@1; service Foo {}").unwrap_err();
        assert_eq!(e.kind(), kind::PARSE);
        let e = compile("afidl 1; schema a@1; record R { 1 x: string; } }}").unwrap_err();
        assert_eq!(e.kind(), kind::PARSE, "trailing tokens must be rejected");
        let e = compile("afidl 1; schema a@1; schema a@1;").unwrap_err();
        assert_eq!(e.kind(), kind::DUPLICATE_SCHEMA_DECLARATION);
    }

    #[test]
    fn empty_module_without_types_is_valid() {
        let compiled = compile("afidl 1; schema a@1;").unwrap();
        assert!(compiled.types.is_empty());
    }

    #[test]
    fn rejects_unknown_type_reference_as_named_ref() {
        // Unknown lowercase identifiers are references, never "maybe
        // primitives": they must fail resolution.
        let e = compile("afidl 1; schema a@1; record R { 1 x: strin; }").unwrap_err();
        assert_eq!(e.kind(), kind::UNKNOWN_TYPE_REFERENCE);
    }

    #[test]
    fn rejects_recursion_and_alias_cycles() {
        let e = compile("afidl 1; schema a@1; record A { 1 x: A; }").unwrap_err();
        assert_eq!(e.kind(), kind::RECURSIVE_TYPE_NOT_SUPPORTED);

        let e =
            compile("afidl 1; schema a@1; record A { 1 b: B; } record B { 1 a: A; }").unwrap_err();
        assert_eq!(e.kind(), kind::RECURSIVE_TYPE_NOT_SUPPORTED);

        let e = compile("afidl 1; schema a@1; alias A = B; alias B = A;").unwrap_err();
        assert_eq!(e.kind(), kind::ALIAS_CYCLE);

        // A reference wrapped in list is still recursion.
        let e = compile("afidl 1; schema a@1; record N { 1 kids: list<N>; }").unwrap_err();
        assert_eq!(e.kind(), kind::RECURSIVE_TYPE_NOT_SUPPORTED);
    }

    #[test]
    fn numbering_rules() {
        let e = compile("afidl 1; schema a@1; record R { 0 x: string; }").unwrap_err();
        assert_eq!(e.kind(), kind::INVALID_FIELD_NUMBER);
        let e = compile("afidl 1; schema a@1; record R { 1 x: string; 1 y: string; }").unwrap_err();
        assert_eq!(e.kind(), kind::DUPLICATE_FIELD_NUMBER);
        let e = compile("afidl 1; schema a@1; record R { 1 x: string; 2 x: int64; }").unwrap_err();
        assert_eq!(e.kind(), kind::DUPLICATE_FIELD_NAME);
        let e = compile("afidl 1; schema a@1; enum E { 0 a; }").unwrap_err();
        assert_eq!(e.kind(), kind::INVALID_ENUM_NUMBER);
        let e = compile("afidl 1; schema a@1; variant V { }").unwrap_err();
        assert_eq!(e.kind(), kind::EMPTY_VARIANT);
        let e = compile("afidl 1; schema a@1; enum E { }").unwrap_err();
        assert_eq!(e.kind(), kind::EMPTY_ENUM);
    }

    #[test]
    fn language_version_and_schema_id() {
        let e = compile("afidl 2; schema a@1;").unwrap_err();
        assert_eq!(e.kind(), kind::INVALID_LANGUAGE_VERSION);
        let e = compile("afidl 1; schema Agentflow@1;").unwrap_err();
        assert_eq!(e.kind(), kind::INVALID_SPECIFICATION_ID);
        let e = compile("afidl 1; schema a@0;").unwrap_err();
        assert_eq!(e.kind(), kind::INVALID_SPECIFICATION_ID);
        let e = compile("afidl 1; schema a@1; schema a@1;").unwrap_err();
        assert_eq!(e.kind(), kind::DUPLICATE_SCHEMA_DECLARATION);
    }

    #[test]
    fn option_nesting_is_fully_supported() {
        // Phase 1A ratified: any legal type may sit under option.
        let compiled = compile(
            "afidl 1; schema a@1;\n\
             alias OptNull = option<null>;\n\
             alias OptUnit = option<unit>;\n\
             alias OptOpt = option<option<string>>;\n",
        )
        .unwrap();
        assert_eq!(compiled.types.len(), 3);
        for name in ["OptNull", "OptUnit", "OptOpt"] {
            assert!(compiled.digest(name).is_some(), "{name} must compile");
        }
        // option<null> and option<unit> differ from None's tagged encoding by
        // construction; their digests differ because the inner primitive
        // keyword differs.
        assert_ne!(
            compiled.digest("OptNull").unwrap(),
            compiled.digest("OptUnit").unwrap()
        );
    }
}
