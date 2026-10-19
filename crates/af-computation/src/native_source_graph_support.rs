//! Explicit Model8 context for the unchanged bounded registration verifier.
pub(crate) use crate::structural_staging::{
    MetaResult, StructuralMetadataFailure, bad, missing, nominal, resource, unsupported,
};
use serde_json::Value;

pub(crate) fn catalogue() -> MetaResult<&'static Value> {
    crate::native_source_staging::native_source_model_catalogue()
        .map_err(|e| StructuralMetadataFailure::new(e, "missing_site"))
}
pub(crate) fn type_key(mt: &Value) -> MetaResult<String> {
    catalogue()?["types"]
        .as_object()
        .ok_or_else(|| unsupported("unknown_nominal_type"))?
        .keys()
        .find(|n| nominal(n, crate::native_source_staging::MODEL) == *mt)
        .cloned()
        .ok_or_else(|| unsupported("unknown_nominal_type"))
}
pub(crate) fn validate_structural_shape_metadata(name: &str, value: &Value) -> MetaResult<()> {
    let m = catalogue()?;
    let d = m["types"]
        .get(name)
        .ok_or_else(|| unsupported("unknown_nominal_type"))?;
    crate::registered_model::check_shape(m, d, value, 0, &mut 0)
        .map_err(|e| StructuralMetadataFailure::new(e, "invalid_shape"))
}
