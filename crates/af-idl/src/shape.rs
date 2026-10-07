//! ACE1 `Shape` derivation from canonical TypeSchemas.
//!
//! `TypeSchemaDigest` hashes the *schema*; to encode an *instance* of a named
//! type with ACE1 this module derives the finite machine `Shape` of that type:
//!
//! - aliases expand to their target (a value of `alias A = string` is a
//!   string value);
//! - named references expand inline (AFIDL1 forbids recursion and has no
//!   cross-module references, so expansion is total and finite);
//! - records, variants and enums become the corresponding closed shape with
//!   their declared member names.
//!
//! An instance value therefore encodes as the fully structural value of its
//! named type; the nominal identity of a named type lives in
//! `TypeSchemaDigest`, not in the value bytes.

use crate::canonical::{CanonicalBody, CanonicalTypeExpr, CompiledSchema};
use crate::error::AfidlError;
use crate::span::SourceSpan;
use af_core::{EnumShape, RecordShape, Shape, VariantShape};

impl CompiledSchema {
    /// Derives the finite ACE1 `Shape` of a named type in this compilation
    /// unit. Named references (including alias chains) expand inline; the
    /// shape is exactly the machine skeleton an instance must conform to.
    ///
    /// # Errors
    /// Returns [`AfidlError::UnknownTypeReference`] when `type_name` is not
    /// declared, or when a named reference inside its expansion cannot be
    /// resolved (which cannot happen for a schema produced by
    /// [`crate::compile`]).
    pub fn shape_of(&self, type_name: &str) -> Result<Shape, AfidlError> {
        let schema_type = self.by_name(type_name).ok_or_else(|| unknown(type_name))?;
        self.body_shape(&schema_type.name, &schema_type.body)
    }

    fn body_shape(&self, name: &str, body: &CanonicalBody) -> Result<Shape, AfidlError> {
        match body {
            CanonicalBody::Alias(target) => self.expr_shape(name, target),
            CanonicalBody::Enum(values) => Ok(Shape::Enum(EnumShape::new(
                name,
                values.iter().map(|value| value.name.clone()),
            ))),
            CanonicalBody::Record(fields) => {
                let mut shapes = Vec::with_capacity(fields.len());
                for field in fields {
                    shapes.push((field.name.clone(), self.expr_shape(name, &field.type_expr)?));
                }
                Ok(Shape::Record(RecordShape::new(name, shapes)))
            }
            CanonicalBody::Variant(cases) => {
                let mut shapes = Vec::with_capacity(cases.len());
                for case in cases {
                    shapes.push((case.name.clone(), self.expr_shape(name, &case.type_expr)?));
                }
                Ok(Shape::Variant(VariantShape::new(name, shapes)))
            }
        }
    }

    fn expr_shape(&self, context: &str, expr: &CanonicalTypeExpr) -> Result<Shape, AfidlError> {
        match expr {
            CanonicalTypeExpr::Primitive(primitive) => Ok(primitive.shape()),
            CanonicalTypeExpr::NamedRef(identity) => {
                let name = identity
                    .rsplit_once('/')
                    .map_or(identity.as_str(), |(_, name)| name);
                let Some(target) = self.by_name(name) else {
                    return Err(unknown(identity));
                };
                // `context` is only used for the diagnostic above; the target
                // type's own name labels the expanded shape.
                let _ = context;
                self.body_shape(&target.name, &target.body)
            }
            CanonicalTypeExpr::List(inner) => {
                Ok(Shape::List(Box::new(self.expr_shape(context, inner)?)))
            }
            CanonicalTypeExpr::MapString(inner) => {
                Ok(Shape::Map(Box::new(self.expr_shape(context, inner)?)))
            }
            CanonicalTypeExpr::Option(inner) => {
                Ok(Shape::Option(Box::new(self.expr_shape(context, inner)?)))
            }
        }
    }
}

fn unknown(name: &str) -> AfidlError {
    AfidlError::UnknownTypeReference {
        name: name.to_string(),
        span: SourceSpan::new(0, 0, 1, 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile;

    const SAMPLE: &str = "afidl 1;\n\
        schema agentflow.example @1;\n\
        alias UserId = string;\n\
        alias AliasUser = User;\n\
        enum Status { 1 pending; 2 closed; }\n\
        record User { 1 id: UserId; 2 name: string; 3 status: Status; }\n\
        variant LookupResult { 1 found: User; 2 missing: unit; }\n";

    #[test]
    fn alias_shapes_expand_to_their_target() {
        let compiled = compile(SAMPLE).unwrap();
        assert_eq!(compiled.shape_of("UserId").unwrap(), Shape::String);
        // alias-to-record expands to the record shape, not a nominal wrapper.
        assert_eq!(
            compiled.shape_of("AliasUser").unwrap(),
            compiled.shape_of("User").unwrap()
        );
    }

    #[test]
    fn record_shape_inlines_named_references() {
        let compiled = compile(SAMPLE).unwrap();
        let expected = Shape::Record(RecordShape::new(
            "User",
            [
                ("id", Shape::String),
                ("name", Shape::String),
                (
                    "status",
                    Shape::Enum(EnumShape::new("Status", ["pending", "closed"])),
                ),
            ],
        ));
        assert_eq!(compiled.shape_of("User").unwrap(), expected);
    }

    #[test]
    fn variant_shape_carries_case_payloads() {
        let compiled = compile(SAMPLE).unwrap();
        let expected = Shape::Variant(VariantShape::new(
            "LookupResult",
            [
                ("found", compiled.shape_of("User").unwrap()),
                ("missing", Shape::Unit),
            ],
        ));
        assert_eq!(compiled.shape_of("LookupResult").unwrap(), expected);
    }

    #[test]
    fn container_shapes_nest() {
        let compiled = compile(
            "afidl 1; schema a@1;\n\
             record R { 1 items: list<string>; 2 index: map<string,int64>; 3 note: option<string>; }\n",
        )
        .unwrap();
        let expected = Shape::Record(RecordShape::new(
            "R",
            [
                ("items", Shape::List(Box::new(Shape::String))),
                ("index", Shape::Map(Box::new(Shape::Int64))),
                ("note", Shape::Option(Box::new(Shape::String))),
            ],
        ));
        assert_eq!(compiled.shape_of("R").unwrap(), expected);
    }

    #[test]
    fn derived_shapes_are_valid_and_encodable() {
        let compiled = compile(SAMPLE).unwrap();
        let shape = compiled.shape_of("User").unwrap();
        shape
            .validate()
            .expect("derived shape must be a valid ACE1 shape");
        let value = af_core::Value::Record(vec![
            ("id".into(), af_core::Value::String("u-1".into())),
            ("name".into(), af_core::Value::String("Ada".into())),
            ("status".into(), af_core::Value::Enum("pending".into())),
        ]);
        let bytes = af_core::encode(&shape, &value).unwrap();
        assert_eq!(
            std::str::from_utf8(&bytes).unwrap(),
            r#"{"id":"u-1","name":"Ada","status":"pending"}"#
        );
    }

    #[test]
    fn unknown_names_are_rejected() {
        let compiled = compile(SAMPLE).unwrap();
        let error = compiled.shape_of("Missing").unwrap_err();
        assert_eq!(error.kind(), crate::error::kind::UNKNOWN_TYPE_REFERENCE);
    }
}
