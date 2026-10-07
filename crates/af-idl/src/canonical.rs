//! The canonical TypeSchema machine model and `TypeSchemaDigest`.
//!
//! Parser AST is *not* digest material. The pipeline is:
//!
//! ```text
//! AFIDL source → parser AST → validation → CanonicalTypeSchema
//!            → ACE1 bytes → HashV1("type-schema", bytes) → TypeSchemaDigest
//! ```
//!
//! Canonicalization removes everything source-level: comments, whitespace,
//! declaration order, source member order. Members are ordered by their
//! explicit numbers ascending; `NamedRef`s are reified as the referenced
//! type's full logical identity `<SpecificationId>/<TypeName>` (never
//! inlined, so digests stay finite and decoupled).
//!
//! Encoding note: the schema-of-schemas ACE1 `Shape` is a finite tree, while
//! `TypeExpr` is recursive. For each canonical schema the shape is expanded
//! to a depth strictly greater than that schema's own expression depth; the
//! encoder walks shape and value in lockstep, so the emitted bytes are
//! exactly those the ideal (infinite) shape would produce, with no change to
//! the frozen ACE1 wire grammar.

use crate::ast::{Decl, Module, TypeExpr, TypeExprKind};
use crate::primitives::Primitive;
use crate::specification::SpecificationId;
use af_core::{DigestDomain, DigestRef, Shape, Value, encode};
use std::fmt;

/// One canonical named type: schema family identity + local name + body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalTypeSchema {
    /// Canonical `SpecificationId` display form, e.g. `agentflow.example@1`.
    pub schema: String,
    /// Local type name (UpperCamelCase).
    pub name: String,
    pub body: CanonicalBody,
}

/// The closed canonical body of a named type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonicalBody {
    /// Nominal alias: identity != target identity.
    Alias(CanonicalTypeExpr),
    Enum(Vec<CanonicalEnumValue>),
    Record(Vec<CanonicalField>),
    Variant(Vec<CanonicalCase>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalEnumValue {
    pub no: u32,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalField {
    pub no: u32,
    pub name: String,
    pub type_expr: CanonicalTypeExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalCase {
    pub no: u32,
    pub name: String,
    pub type_expr: CanonicalTypeExpr,
}

/// Canonical type expression: explicit variants, no textual composition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonicalTypeExpr {
    Primitive(Primitive),
    /// Full logical identity of the referenced type: `<spec-id>/<TypeName>`.
    NamedRef(String),
    List(Box<CanonicalTypeExpr>),
    MapString(Box<CanonicalTypeExpr>),
    Option(Box<CanonicalTypeExpr>),
}

impl CanonicalTypeSchema {
    /// The full logical identity `<spec-id>/<TypeName>` (never a file path).
    pub fn full_identity(&self) -> String {
        format!("{}/{}", self.schema, self.name)
    }

    /// Maximum nesting depth of the body's type expressions (1 for leaves).
    fn type_expr_depth(expr: &CanonicalTypeExpr) -> u32 {
        match expr {
            CanonicalTypeExpr::Primitive(_) | CanonicalTypeExpr::NamedRef(_) => 1,
            CanonicalTypeExpr::List(inner)
            | CanonicalTypeExpr::MapString(inner)
            | CanonicalTypeExpr::Option(inner) => 1 + Self::type_expr_depth(inner),
        }
    }

    fn body_depth(&self) -> u32 {
        match &self.body {
            CanonicalBody::Alias(target) => Self::type_expr_depth(target),
            CanonicalBody::Enum(_) => 1, // values carry no type expressions
            CanonicalBody::Record(fields) => fields
                .iter()
                .map(|f| Self::type_expr_depth(&f.type_expr))
                .max()
                .unwrap_or(1),
            CanonicalBody::Variant(cases) => cases
                .iter()
                .map(|c| Self::type_expr_depth(&c.type_expr))
                .max()
                .unwrap_or(1),
        }
    }

    /// The ACE1 machine value of this canonical schema.
    pub fn to_value(&self) -> Value {
        Value::Record(vec![
            ("schema".into(), Value::String(self.schema.clone())),
            ("name".into(), Value::String(self.name.clone())),
            ("body".into(), self.body_value()),
        ])
    }

    fn body_value(&self) -> Value {
        let (case, payload) = match &self.body {
            CanonicalBody::Alias(target) => ("alias", type_expr_value(target)),
            CanonicalBody::Enum(values) => (
                "enum",
                Value::Record(vec![(
                    "values".into(),
                    Value::List(
                        values
                            .iter()
                            .map(|v| {
                                Value::Record(vec![
                                    ("no".into(), Value::UInt32(v.no)),
                                    ("name".into(), Value::String(v.name.clone())),
                                ])
                            })
                            .collect(),
                    ),
                )]),
            ),
            CanonicalBody::Record(fields) => (
                "record",
                Value::Record(vec![(
                    "fields".into(),
                    Value::List(
                        fields
                            .iter()
                            .map(|f| {
                                Value::Record(vec![
                                    ("no".into(), Value::UInt32(f.no)),
                                    ("name".into(), Value::String(f.name.clone())),
                                    ("type".into(), type_expr_value(&f.type_expr)),
                                ])
                            })
                            .collect(),
                    ),
                )]),
            ),
            CanonicalBody::Variant(cases) => (
                "variant",
                Value::Record(vec![(
                    "cases".into(),
                    Value::List(
                        cases
                            .iter()
                            .map(|c| {
                                Value::Record(vec![
                                    ("no".into(), Value::UInt32(c.no)),
                                    ("name".into(), Value::String(c.name.clone())),
                                    ("type".into(), type_expr_value(&c.type_expr)),
                                ])
                            })
                            .collect(),
                    ),
                )]),
            ),
        };
        Value::Variant {
            case: case.into(),
            value: Box::new(payload),
        }
    }

    /// The canonical ACE1 bytes of this schema.
    pub fn ace1_bytes(&self) -> Result<Vec<u8>, af_core::AceError> {
        // Depth strictly greater than the value's own expression depth, so
        // the encoder never reaches the unreachable stub.
        let depth = self.body_depth() + 1;
        let shape = schema_shape(depth);
        encode(&shape, &self.to_value())
    }

    /// `TypeSchemaDigest = HashV1("type-schema", ACE1(CanonicalTypeSchema))`.
    pub fn digest(&self) -> Result<DigestRef, af_core::AceError> {
        let bytes = self.ace1_bytes()?;
        Ok(DigestRef::compute(DigestDomain::TypeSchema, &bytes))
    }

    /// HashV1 preimage bytes, exposed for cross-implementation corpus checks.
    pub fn digest_payload(&self) -> Result<Vec<u8>, af_core::AceError> {
        self.ace1_bytes()
    }
}

impl fmt::Display for CanonicalTypeSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.full_identity())
    }
}

fn type_expr_value(expr: &CanonicalTypeExpr) -> Value {
    let (case, payload) = match expr {
        CanonicalTypeExpr::Primitive(p) => ("primitive", Value::String(p.keyword().into())),
        CanonicalTypeExpr::NamedRef(identity) => ("ref", Value::String(identity.clone())),
        CanonicalTypeExpr::List(inner) => ("list", type_expr_value(inner)),
        CanonicalTypeExpr::MapString(inner) => ("map_string", type_expr_value(inner)),
        CanonicalTypeExpr::Option(inner) => ("option", type_expr_value(inner)),
    };
    Value::Variant {
        case: case.into(),
        value: Box::new(payload),
    }
}

/// A validated compilation unit: the schema family plus its canonical types,
/// sorted by local name (declaration order is not identity).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledSchema {
    pub specification: SpecificationId,
    pub types: Vec<CanonicalTypeSchema>,
}

impl CompiledSchema {
    /// Looks up a canonical type by local name.
    pub fn by_name(&self, name: &str) -> Option<&CanonicalTypeSchema> {
        self.types.iter().find(|t| t.name == name)
    }

    /// `TypeSchemaDigest` for a local type name.
    pub fn digest(&self, type_name: &str) -> Option<DigestRef> {
        self.by_name(type_name)
            .and_then(|schema| schema.digest().ok())
    }

    /// The declared local type names, in canonical (sorted) order.
    pub fn type_names(&self) -> impl Iterator<Item = &str> {
        self.types.iter().map(|t| t.name.as_str())
    }
}

/// Builds the canonical schema of a *validated* module, resolving every type
/// expression through `resolve`.
///
/// `resolve` is the sole extension point between the two resolution modes:
///
/// - the local-only path (no imports), where a `NamedRef` becomes
///   `<this-specification-id>/<TypeName>`;
/// - the import-aware path, where a local `NamedRef` resolves the same way and
///   a `QualifiedNamedRef` becomes `<imported-specification-id>/<TypeName>`.
///
/// Both modes produce the identical wire model: the canonical encoder never
/// sees an import alias. `resolve` may only fail in the import-aware mode.
pub(crate) fn build_schema<F>(
    module: &Module,
    resolve: &F,
) -> Result<CompiledSchema, crate::error::AfidlError>
where
    F: Fn(&TypeExpr) -> Result<CanonicalTypeExpr, crate::error::AfidlError>,
{
    let mut types: Vec<CanonicalTypeSchema> = Vec::with_capacity(module.declarations.len());
    let identity = module.schema.id.canonical();
    for decl in &module.declarations {
        types.push(canonical_type(&identity, decl, resolve)?);
    }
    types.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(CompiledSchema {
        specification: module.schema.id.clone(),
        types,
    })
}

/// Resolves a type expression using only the module's own import table.
///
/// This is the universe-free resolution mode: the *identity* of an imported
/// specification is fully determined by the module's own source (an import
/// names an exact `SpecificationId`), so canonicalization never needs a
/// registry or a resolved universe. Whether the imported module exists and
/// declares the referenced type is a separate, stronger question answered by
/// [`crate::resolve`] against an explicit universe.
pub(crate) fn resolve_from_imports(
    module: &Module,
    expr: &TypeExpr,
) -> Result<CanonicalTypeExpr, crate::error::AfidlError> {
    let this = module.schema.id.canonical();
    resolve_expr(module, &this, expr)
}

fn resolve_expr(
    module: &Module,
    this: &str,
    expr: &TypeExpr,
) -> Result<CanonicalTypeExpr, crate::error::AfidlError> {
    let kind = match &expr.kind {
        TypeExprKind::Primitive(primitive) => CanonicalTypeExpr::Primitive(*primitive),
        TypeExprKind::NamedRef(name) => CanonicalTypeExpr::NamedRef(format!("{this}/{name}")),
        TypeExprKind::QualifiedNamedRef(reference) => {
            let target = module
                .imports
                .iter()
                .find(|import| import.alias == reference.import_alias)
                .ok_or_else(|| crate::error::AfidlError::UnknownImportAlias {
                    alias: reference.import_alias.clone(),
                    span: reference.span,
                })?;
            CanonicalTypeExpr::NamedRef(format!(
                "{}/{}",
                target.id.canonical(),
                reference.type_name
            ))
        }
        TypeExprKind::List(inner) => {
            CanonicalTypeExpr::List(Box::new(resolve_expr(module, this, inner)?))
        }
        TypeExprKind::MapString(inner) => {
            CanonicalTypeExpr::MapString(Box::new(resolve_expr(module, this, inner)?))
        }
        TypeExprKind::Option(inner) => {
            CanonicalTypeExpr::Option(Box::new(resolve_expr(module, this, inner)?))
        }
    };
    Ok(kind)
}

/// Canonicalizes one declaration. Every type expression is lowered by
/// `resolve`; nothing else about the declaration varies between the two
/// resolution modes.
fn canonical_type<F>(
    identity: &str,
    decl: &Decl,
    resolve: &F,
) -> Result<CanonicalTypeSchema, crate::error::AfidlError>
where
    F: Fn(&TypeExpr) -> Result<CanonicalTypeExpr, crate::error::AfidlError>,
{
    let name = decl.name().to_string();
    let body = match decl {
        Decl::Alias(alias) => CanonicalBody::Alias(resolve(&alias.target)?),
        Decl::Enum(enum_decl) => {
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
        Decl::Record(record) => {
            let mut fields: Vec<CanonicalField> = Vec::with_capacity(record.fields.len());
            for field in &record.fields {
                fields.push(CanonicalField {
                    no: field.no,
                    name: field.name.clone(),
                    type_expr: resolve(&field.type_expr)?,
                });
            }
            fields.sort_by_key(|f| f.no);
            CanonicalBody::Record(fields)
        }
        Decl::Variant(variant) => {
            let mut cases: Vec<CanonicalCase> = Vec::with_capacity(variant.cases.len());
            for case in &variant.cases {
                cases.push(CanonicalCase {
                    no: case.no,
                    name: case.name.clone(),
                    type_expr: resolve(&case.payload)?,
                });
            }
            cases.sort_by_key(|c| c.no);
            CanonicalBody::Variant(cases)
        }
    };
    // The canonical schema carries the *module's* identity, never an import
    // alias: imports are structurally absent from this model (AF-AFIDL-0024).
    Ok(CanonicalTypeSchema {
        schema: identity.to_string(),
        name,
        body,
    })
}

// ---------------------------------------------------------------------------
// Schema-of-schemas shape (finite depth expansion)
// ---------------------------------------------------------------------------

fn type_expr_shape(depth: u32) -> Shape {
    if depth == 0 {
        // Unreachable guard: the encoder pairs this shape only with a value
        // position that would have to be deeper than the declared depth.
        return Shape::Null;
    }
    Shape::Variant(af_core::VariantShape::new(
        "TypeExpr",
        [
            ("primitive", Shape::String),
            ("ref", Shape::String),
            ("list", type_expr_shape(depth - 1)),
            ("map_string", type_expr_shape(depth - 1)),
            ("option", type_expr_shape(depth - 1)),
        ],
    ))
}

fn schema_shape(depth: u32) -> Shape {
    let type_expr = type_expr_shape(depth);
    let enum_value = Shape::Record(af_core::RecordShape::new(
        "EnumValue",
        [("no", Shape::UInt32), ("name", Shape::String)],
    ));
    let field = Shape::Record(af_core::RecordShape::new(
        "Field",
        [
            ("no", Shape::UInt32),
            ("name", Shape::String),
            ("type", type_expr.clone()),
        ],
    ));
    let case = Shape::Record(af_core::RecordShape::new(
        "Case",
        [
            ("no", Shape::UInt32),
            ("name", Shape::String),
            ("type", type_expr.clone()),
        ],
    ));
    let body = af_core::VariantShape::new(
        "TypeBody",
        [
            ("alias", type_expr),
            (
                "enum",
                Shape::Record(af_core::RecordShape::new(
                    "EnumBody",
                    [("values", Shape::List(Box::new(enum_value)))],
                )),
            ),
            (
                "record",
                Shape::Record(af_core::RecordShape::new(
                    "RecordBody",
                    [("fields", Shape::List(Box::new(field)))],
                )),
            ),
            (
                "variant",
                Shape::Record(af_core::RecordShape::new(
                    "VariantBody",
                    [("cases", Shape::List(Box::new(case)))],
                )),
            ),
        ],
    );
    Shape::Record(af_core::RecordShape::new(
        "TypeSchema",
        [
            ("schema", Shape::String),
            ("name", Shape::String),
            ("body", Shape::Variant(body)),
        ],
    ))
}
