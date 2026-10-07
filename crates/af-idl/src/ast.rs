//! The AFIDL1 source AST (parser output).
//!
//! This is the *source* layer: it preserves declaration order and carries
//! `SourceSpan`s. It is deliberately distinct from the canonical TypeSchema
//! model — source spans, comments and source ordering must never leak into
//! schema identity.

use crate::primitives::Primitive;
use crate::span::SourceSpan;
use crate::specification::SpecificationId;

/// A parsed AFIDL1 module: language declaration, schema family identity and
/// the declarations of the compilation unit (source order).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub language_version: u32,
    pub schema: SchemaDecl,
    pub declarations: Vec<Decl>,
}

/// The single `schema <id> @<major>;` declaration of the module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaDecl {
    pub id: SpecificationId,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decl {
    Alias(AliasDecl),
    Enum(EnumDecl),
    Record(RecordDecl),
    Variant(VariantDecl),
}

impl Decl {
    /// The declared local type name.
    pub fn name(&self) -> &str {
        match self {
            Decl::Alias(a) => &a.name,
            Decl::Enum(e) => &e.name,
            Decl::Record(r) => &r.name,
            Decl::Variant(v) => &v.name,
        }
    }

    /// The span of the whole declaration.
    pub fn span(&self) -> SourceSpan {
        match self {
            Decl::Alias(a) => a.span,
            Decl::Enum(e) => e.span,
            Decl::Record(r) => r.span,
            Decl::Variant(v) => v.span,
        }
    }

    /// All type expressions appearing directly in this declaration.
    pub fn type_exprs(&self) -> Vec<&TypeExpr> {
        match self {
            Decl::Alias(a) => vec![&a.target],
            Decl::Enum(_) => Vec::new(),
            Decl::Record(r) => r.fields.iter().map(|f| &f.type_expr).collect(),
            Decl::Variant(v) => v.cases.iter().map(|c| &c.payload).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasDecl {
    pub name: String,
    pub target: TypeExpr,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumDecl {
    pub name: String,
    pub values: Vec<EnumValueDecl>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumValueDecl {
    pub no: u32,
    pub name: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordDecl {
    pub name: String,
    pub fields: Vec<FieldDecl>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDecl {
    pub no: u32,
    pub name: String,
    pub type_expr: TypeExpr,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantDecl {
    pub name: String,
    pub cases: Vec<VariantCaseDecl>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantCaseDecl {
    pub no: u32,
    pub name: String,
    pub payload: TypeExpr,
    pub span: SourceSpan,
}

/// A source-level type expression, normalized into explicit variants at parse
/// time (no text re-parsing later).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeExpr {
    pub kind: TypeExprKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeExprKind {
    /// A builtin primitive keyword.
    Primitive(Primitive),
    /// A named reference to a type declared in the same compilation unit.
    /// Unknown lowercase identifiers land here and fail resolution if they
    /// are not declared (no "maybe primitive" acceptance).
    NamedRef(String),
    List(Box<TypeExpr>),
    MapString(Box<TypeExpr>),
    Option(Box<TypeExpr>),
}
