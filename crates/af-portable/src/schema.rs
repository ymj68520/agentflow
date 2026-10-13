use crate::error::{PortableProofError, ValidationStage};
use af_core::Shape;
use af_idl::{CompiledModuleSet, compile_module_set};
use std::sync::OnceLock;

const SOURCES: &[(&str, &str)] = &[
    (
        "spec/idl/specification-registry.afidl",
        include_str!("../../../spec/idl/specification-registry.afidl"),
    ),
    (
        "spec/idl/semantic-core.afidl",
        include_str!("../../../spec/idl/semantic-core.afidl"),
    ),
    (
        "spec/idl/semantic-ir.afidl",
        include_str!("../../../spec/idl/semantic-ir.afidl"),
    ),
    (
        "spec/idl/effect-signature.afidl",
        include_str!("../../../spec/idl/effect-signature.afidl"),
    ),
    (
        "spec/idl/failure-row.afidl",
        include_str!("../../../spec/idl/failure-row.afidl"),
    ),
    (
        "spec/idl/resource-requirement.afidl",
        include_str!("../../../spec/idl/resource-requirement.afidl"),
    ),
    (
        "spec/idl/information-flow.afidl",
        include_str!("../../../spec/idl/information-flow.afidl"),
    ),
    (
        "spec/idl/usage-signature.afidl",
        include_str!("../../../spec/idl/usage-signature.afidl"),
    ),
    (
        "spec/idl/cost-projection.afidl",
        include_str!("../../../spec/idl/cost-projection.afidl"),
    ),
    (
        "spec/idl/recovery-contract.afidl",
        include_str!("../../../spec/idl/recovery-contract.afidl"),
    ),
    (
        "spec/idl/execution-requirements.afidl",
        include_str!("../../../spec/idl/execution-requirements.afidl"),
    ),
    (
        "spec/idl/component-contract.afidl",
        include_str!("../../../spec/idl/component-contract.afidl"),
    ),
    (
        "spec/idl/effect-resource-compatibility.afidl",
        include_str!("../../../spec/idl/effect-resource-compatibility.afidl"),
    ),
    (
        "spec/idl/recovery-usage-coverage.afidl",
        include_str!("../../../spec/idl/recovery-usage-coverage.afidl"),
    ),
    (
        "spec/idl/flow-execution-relation.afidl",
        include_str!("../../../spec/idl/flow-execution-relation.afidl"),
    ),
    (
        "spec/idl/core/machine-primitives.afidl",
        include_str!("../../../spec/idl/core/machine-primitives.afidl"),
    ),
    (
        "spec/idl/portable-proof-inputs.afidl",
        include_str!("../../../spec/idl/portable-proof-inputs.afidl"),
    ),
];

static COMPILED_MODULES: OnceLock<Result<CompiledModuleSet, String>> = OnceLock::new();
static PORTABLE_BUNDLE_SHAPE: OnceLock<Result<Shape, String>> = OnceLock::new();
static TYPE_SCHEMA_SHAPE: OnceLock<Shape> = OnceLock::new();

pub(crate) fn compiled_modules() -> Result<&'static CompiledModuleSet, PortableProofError> {
    match COMPILED_MODULES.get_or_init(|| {
        let sources = SOURCES
            .iter()
            .map(|(path, source)| ((*path).to_owned(), (*source).to_owned()))
            .collect::<Vec<_>>();
        compile_module_set(&sources).map_err(|error| error.to_string())
    }) {
        Ok(modules) => Ok(modules),
        Err(detail) => Err(PortableProofError::new(
            ValidationStage::D4,
            "UnsupportedCurrentSchema",
            "$.schema_version",
            detail.clone(),
        )),
    }
}

pub(crate) fn bundle_shape() -> Result<&'static Shape, PortableProofError> {
    match PORTABLE_BUNDLE_SHAPE.get_or_init(|| {
        compiled_modules()
            .map_err(|error| error.to_string())?
            .shape_of("agentflow.portable.proof.inputs@1/PortableProofInputBundle")
            .map_err(|error| error.to_string())
    }) {
        Ok(shape) => Ok(shape),
        Err(error) => Err({
            PortableProofError::new(
                ValidationStage::D4,
                "UnsupportedCurrentSchema",
                "$.schema_version",
                error.clone(),
            )
        }),
    }
}

pub(crate) fn type_schema_shape() -> &'static Shape {
    TYPE_SCHEMA_SHAPE.get_or_init(build_type_schema_shape)
}

fn build_type_schema_shape() -> Shape {
    // A bounded expansion of the already-registered recursive TypeExpr *value*
    // grammar. The candidate outer AFIDL records stay finite; a TypeSchema blob
    // is accepted only within this explicit expression depth.
    const DEPTH: u32 = 8;
    let expression = type_expr_shape(DEPTH);
    let enum_value = record(
        "EnumValue",
        vec![("no", Shape::UInt32), ("name", Shape::String)],
    );
    let field = record(
        "Field",
        vec![
            ("no", Shape::UInt32),
            ("name", Shape::String),
            ("type", expression.clone()),
        ],
    );
    let case = record(
        "Case",
        vec![
            ("no", Shape::UInt32),
            ("name", Shape::String),
            ("type", expression.clone()),
        ],
    );
    let body = Shape::Variant(af_core::VariantShape::new(
        "TypeBody",
        [
            ("alias", expression),
            (
                "enum",
                record(
                    "EnumBody",
                    vec![("values", Shape::List(Box::new(enum_value)))],
                ),
            ),
            (
                "record",
                record("RecordBody", vec![("fields", Shape::List(Box::new(field)))]),
            ),
            (
                "variant",
                record("VariantBody", vec![("cases", Shape::List(Box::new(case)))]),
            ),
        ],
    ));
    record(
        "TypeSchema",
        vec![
            ("schema", Shape::String),
            ("name", Shape::String),
            ("body", body),
        ],
    )
}

fn type_expr_shape(depth: u32) -> Shape {
    let child = if depth == 0 {
        Shape::Null
    } else {
        type_expr_shape(depth - 1)
    };
    Shape::Variant(af_core::VariantShape::new(
        "TypeExpr",
        [
            ("primitive", Shape::String),
            ("ref", Shape::String),
            ("list", child.clone()),
            ("map_string", child.clone()),
            ("option", child),
        ],
    ))
}

fn record(name: &str, fields: Vec<(&str, Shape)>) -> Shape {
    Shape::Record(af_core::RecordShape::new(name, fields))
}

pub(crate) fn registered_schema_refs() -> Result<Vec<af_semantic::TypeSchemaRef>, PortableProofError>
{
    let mut result = Vec::new();
    for module in compiled_modules()?.modules() {
        for schema in &module.types {
            let digest = schema.digest().map_err(|error| {
                PortableProofError::new(
                    ValidationStage::D4,
                    "UnsupportedCurrentSchema",
                    "$.compiler_semantics_manifests",
                    error.to_string(),
                )
            })?;
            result.push(af_semantic::TypeSchemaRef {
                specification_id: schema.schema.clone(),
                type_name: schema.name.clone(),
                schema_digest: af_semantic::DigestRefValue {
                    algorithm: "sha256".to_owned(),
                    domain: digest.domain().as_str().to_owned(),
                    hex: digest.hex().to_owned(),
                },
            });
        }
    }
    result.sort_by(|left, right| {
        (
            &left.specification_id,
            &left.type_name,
            &left.schema_digest.algorithm,
            &left.schema_digest.domain,
            &left.schema_digest.hex,
        )
            .cmp(&(
                &right.specification_id,
                &right.type_name,
                &right.schema_digest.algorithm,
                &right.schema_digest.domain,
                &right.schema_digest.hex,
            ))
    });
    Ok(result)
}
