//! Exact, namespace-aware mapping helpers shared by leaf analyzers.

use crate::proof::LeafVerdict;
use af_ir::{SemanticIrDocument, SubflowInterface};
use af_semantic::{ExpressionId, SemanticTypeId, SemanticTypeNodeBody, TypeSchemaRef};

pub(crate) fn schema_ref_for_type(
    document: &SemanticIrDocument,
    type_id: SemanticTypeId,
) -> Option<&TypeSchemaRef> {
    document
        .types
        .iter()
        .find(|node| node.id == type_id)
        .and_then(|node| match &node.body {
            SemanticTypeNodeBody::Schema(schema) => Some(schema),
            SemanticTypeNodeBody::Primitive(_)
            | SemanticTypeNodeBody::List(_)
            | SemanticTypeNodeBody::MapString(_)
            | SemanticTypeNodeBody::Option(_) => None,
        })
}

pub(crate) fn expression_schema_ref(
    document: &SemanticIrDocument,
    expression: ExpressionId,
) -> Option<&TypeSchemaRef> {
    let expression = document
        .expressions
        .iter()
        .find(|item| item.id == expression)?;
    schema_ref_for_type(document, expression.result_type_id)
}

pub(crate) fn interface_schema_match(
    interface: &SubflowInterface,
    input_types: &[(u32, TypeSchemaRef)],
    output_types: &[(u32, TypeSchemaRef)],
    resource_types: &[(u32, TypeSchemaRef)],
) -> Result<(), (LeafVerdict, &'static str)> {
    if interface.inputs().len() != input_types.len()
        || interface.outputs().len() != output_types.len()
        || interface.resources().len() != resource_types.len()
    {
        return Err((
            LeafVerdict::Violation,
            "exported contract port cardinalities differ from the derived target interface",
        ));
    }
    for port in interface.inputs() {
        let Some((_, expected)) = input_types
            .iter()
            .find(|(number, _)| *number == port.port_no())
        else {
            return Err((
                LeafVerdict::Violation,
                "exported contract input port set differs from target interface",
            ));
        };
        match port.semantic_type() {
            af_semantic::StructuralType::Schema(actual) if actual == expected => {}
            af_semantic::StructuralType::Schema(_) => {
                return Err((
                    LeafVerdict::Violation,
                    "exported contract input TypeSchemaRef differs from target interface",
                ));
            }
            _ => {
                return Err((
                    LeafVerdict::Unproven,
                    "target interface input has no exact public TypeSchemaRef mapping",
                ));
            }
        }
    }
    for port in interface.outputs() {
        let Some((_, expected)) = output_types
            .iter()
            .find(|(number, _)| *number == port.port_no())
        else {
            return Err((
                LeafVerdict::Violation,
                "exported contract output port set differs from target interface",
            ));
        };
        match port.semantic_type() {
            af_semantic::StructuralType::Schema(actual) if actual == expected => {}
            af_semantic::StructuralType::Schema(_) => {
                return Err((
                    LeafVerdict::Violation,
                    "exported contract output TypeSchemaRef differs from target interface",
                ));
            }
            _ => {
                return Err((
                    LeafVerdict::Unproven,
                    "target interface output has no exact public TypeSchemaRef mapping",
                ));
            }
        }
    }
    for port in interface.resources() {
        let Some((_, expected)) = resource_types
            .iter()
            .find(|(number, _)| *number == port.port_no())
        else {
            return Err((
                LeafVerdict::Violation,
                "exported contract resource port set differs from target interface",
            ));
        };
        if port.requirement_type() != expected {
            return Err((
                LeafVerdict::Violation,
                "exported contract resource TypeSchemaRef differs from target interface",
            ));
        }
    }
    Ok(())
}
