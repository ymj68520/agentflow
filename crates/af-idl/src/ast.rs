//! The AFIDL1 source AST (parser output).
//!
//! This is the *source* layer: it preserves declaration order and carries
//! `SourceSpan`s. It is deliberately distinct from the canonical TypeSchema
//! model — source spans, comments and source ordering must never leak into
//! schema identity.

use crate::primitives::Primitive;
use crate::span::SourceSpan;
use crate::specification::SpecificationId;

/// A parsed AFIDL1 module: language declaration, schema family identity, the
/// module header's import declarations (source order) and the declarations of
/// the compilation unit (source order).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub language_version: u32,
    pub schema: SchemaDecl,
    /// Import declarations, exactly as written. They are a *source resolution
    /// aid*: they never enter a canonical TypeSchema and never influence a
    /// TypeSchemaDigest (AF-AFIDL-0023, AF-AFIDL-0024).
    pub imports: Vec<ImportDecl>,
    pub declarations: Vec<Decl>,
}

impl Module {
    /// The import alias → specification id table of the module header, in
    /// source order. Duplicate aliases are preserved here; they are a
    /// validation error (`duplicate_import_alias`).
    pub fn import_table(&self) -> Vec<(&str, &SpecificationId)> {
        self.imports
            .iter()
            .map(|import| (import.alias.as_str(), &import.id))
            .collect()
    }
}

/// One `import <SpecificationId> as <alias>;` declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportDecl {
    pub id: SpecificationId,
    pub alias: String,
    pub span: SourceSpan,
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
    /// A qualified reference `<import-alias>.<TYPE-NAME>` to a type declared
    /// in an explicitly imported module. This is a *source* form: after
    /// resolution it becomes the ordinary full logical identity
    /// `<SpecificationId>/<TypeName>` and the import alias disappears.
    QualifiedNamedRef(QualifiedRef),
    List(Box<TypeExpr>),
    MapString(Box<TypeExpr>),
    Option(Box<TypeExpr>),
}

/// A source-level qualified type reference: import alias plus type name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualifiedRef {
    pub import_alias: String,
    pub type_name: String,
    pub span: SourceSpan,
}
