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
pub mod shape;
pub mod span;
pub mod specification;
pub mod validate;

pub use ast::{
    AliasDecl, Decl, EnumDecl, EnumValueDecl, FieldDecl, Module, SchemaDecl, TypeExpr,
    TypeExprKind, VariantCaseDecl, VariantDecl,
};
pub use canonical::{
    CanonicalBody, CanonicalCase, CanonicalEnumValue, CanonicalField, CanonicalTypeExpr,
    CanonicalTypeSchema, CompiledSchema,
};
pub use error::AfidlError;
pub use parser::parse;
pub use primitives::{AFIDL1_VERSION, Primitive};
pub use span::SourceSpan;
pub use specification::SpecificationId;
pub use validate::validate;

/// Parses, validates and canonicalizes an AFIDL1 source.
///
/// This is the primary entry point. Deterministic: the same source (and any
/// source differing only in comments, whitespace or declaration/member
/// ordering with identical number bindings) yields identical canonical
/// schemas and identical digests.
///
/// # Errors
/// Returns a structured [`AfidlError`] for lex, parse and validation
/// failures, each carrying the source span of the offending construct.
pub fn compile(source: &str) -> Result<CompiledSchema, AfidlError> {
    let module = parse(source)?;
    validate(&module)?;
    Ok(canonicalize(&module))
}

/// Canonicalizes a *validated* module into its canonical types.
///
/// Canonical member ordering is by explicit number ascending (`field_no`,
/// `case_no`, `value_no`); the type list is sorted by local name. Source
/// declaration order never survives canonicalization.
pub fn canonicalize(module: &Module) -> CompiledSchema {
    let mut types: Vec<CanonicalTypeSchema> = module
        .declarations
        .iter()
        .map(|decl| canonical_type(module.schema.id.canonical(), decl))
        .collect();
    types.sort_by(|a, b| a.name.cmp(&b.name));
    CompiledSchema {
        specification: module.schema.id.clone(),
        types,
    }
}

fn canonical_type(schema_id: String, decl: &Decl) -> CanonicalTypeSchema {
    let name = decl.name().to_string();
    let body = match decl {
        ast::Decl::Alias(alias) => {
            CanonicalBody::Alias(canonical_expr(schema_id.as_str(), &alias.target))
        }
        ast::Decl::Enum(enum_decl) => {
            let mut values: Vec<CanonicalEnumValue> = enum_decl
                .values
                .iter()
                .map(|v| CanonicalEnumValue {
                    no: v.no,
                    name: v.name.clone(),
                })
                .collect();
            values.sort_by_key(|v| v.no);
            CanonicalBody::Enum(values)
        }
        ast::Decl::Record(record) => {
            let mut fields: Vec<CanonicalField> = record
                .fields
                .iter()
                .map(|f| CanonicalField {
                    no: f.no,
                    name: f.name.clone(),
                    type_expr: canonical_expr(schema_id.as_str(), &f.type_expr),
                })
                .collect();
            fields.sort_by_key(|f| f.no);
            CanonicalBody::Record(fields)
        }
        ast::Decl::Variant(variant) => {
            let mut cases: Vec<CanonicalCase> = variant
                .cases
                .iter()
                .map(|c| CanonicalCase {
                    no: c.no,
                    name: c.name.clone(),
                    type_expr: canonical_expr(schema_id.as_str(), &c.payload),
                })
                .collect();
            cases.sort_by_key(|c| c.no);
            CanonicalBody::Variant(cases)
        }
    };
    CanonicalTypeSchema {
        schema: schema_id,
        name,
        body,
    }
}

fn canonical_expr(schema_id: &str, expr: &TypeExpr) -> CanonicalTypeExpr {
    match &expr.kind {
        TypeExprKind::Primitive(primitive) => CanonicalTypeExpr::Primitive(*primitive),
        TypeExprKind::NamedRef(name) => CanonicalTypeExpr::NamedRef(format!("{schema_id}/{name}")),
        TypeExprKind::List(inner) => {
            CanonicalTypeExpr::List(Box::new(canonical_expr(schema_id, inner)))
        }
        TypeExprKind::MapString(inner) => {
            CanonicalTypeExpr::MapString(Box::new(canonical_expr(schema_id, inner)))
        }
        TypeExprKind::Option(inner) => {
            CanonicalTypeExpr::Option(Box::new(canonical_expr(schema_id, inner)))
        }
    }
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
