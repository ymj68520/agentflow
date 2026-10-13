use af_portable::PortableProofCodec;

const COMPLETE_BUNDLE: &[u8] =
    include_bytes!("../../../spec/conformance/portable-proof-codec/full-ten-field-bundle.ace1");

#[test]
fn d5_reconstructs_and_recaptures_the_complete_registered_context() {
    let supported = PortableProofCodec::default()
        .validate_supported(COMPLETE_BUNDLE)
        .expect("registered full ten-field bundle passes D0-D4");
    let context = supported
        .reconstruct_context(1)
        .expect("complete empty catalog declarations reconstruct exactly");
    let projection = context.projection();
    assert_eq!(projection.context_id, 1);
    assert_eq!(projection.modules.len(), 16);
    assert_eq!(projection.type_schemas.len(), 314);
    assert!(projection.effect_catalogs.is_empty());
    assert!(projection.resource_schemas.is_empty());
    assert!(projection.reconciliation_operations.is_empty());
    assert!(projection.effect_resource_relation.is_none());

    context.with_proof_context(|proof_context| {
        assert_eq!(proof_context.modules.len(), 16);
        assert_eq!(proof_context.schema_universe.len(), 314);
        assert!(proof_context.effects.is_empty());
        assert!(proof_context.resources.is_empty());
    });
    let result = supported
        .reconstruct_static_inputs(1, 1)
        .expect("fresh D5-D8 reconstruction succeeds");
    assert_eq!(result.context().context_id, 1);
    assert_eq!(result.checked_contracts().len(), 1);
    assert_eq!(result.checked_irs().len(), 1);
    assert_eq!(result.source_associations().len(), 1);
    assert!(matches!(
        result.source_associations()[0].target,
        af_portable::SourceAssociationTargetProjection::Invoke(_)
    ));
    assert!(result.resolver_identity().subflow_targets().is_empty());
    assert_eq!(result.await_leaf_count(), 0);

    let second_context = supported
        .reconstruct_context(1)
        .expect("an equal complete declaration reconstructs a second context");
    context.with_proof_context(|first| {
        second_context.with_proof_context(|second| {
            assert!(!std::ptr::eq(first, second));
        });
    });

    let mut unavailable: serde_json::Value = serde_json::from_slice(COMPLETE_BUNDLE).unwrap();
    unavailable["proof_contexts"][0]["content"]["usage_meters"] =
        serde_json::json!({"unavailable": "not_supplied"});
    let unavailable_bytes = serde_json::to_vec(&unavailable).unwrap();
    let unavailable = PortableProofCodec::default()
        .validate_supported(&unavailable_bytes)
        .expect("D4 supports the schema's explicit availability state");
    let error = unavailable.reconstruct_context(1).unwrap_err();
    assert_eq!(error.stage, af_portable::ValidationStage::D5);
    assert_eq!(error.family, "IncompleteContext");
}
