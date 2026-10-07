//! `af-ir` — the AgentFlow canonical Semantic IR (Phase 3B).
//!
//! This crate freezes the first materialization of the Canonical Semantic IR:
//! a **structured control tree** (ownership via [`ComputationId`] child
//! references over the closed 13-opcode [`ComputationBody`] set), a **typed
//! dataflow shell** (SSA [`ValueSlot`] producers over the Phase 3A tables and
//! the closed pure-expression language) and **closed contract tables**
//! (component / subflow pins).
//!
//! Responsibilities — exactly:
//!
//! 1. the typed model ([`model`]);
//! 2. static validation ([`SemanticIrDocument::check`]): references, producer
//!    uniqueness, ownership tree, wrapper interface equality, contract and
//!    schema-reference pinning;
//! 3. canonical ordering ([`SemanticIrDocument::canonical`]);
//! 4. ACE1 lowering ([`SemanticIrDocument::canonical_bytes`]);
//! 5. `SemanticIrDigest = HashV1("semantic-ir", ACE1(canonical document))`
//!    ([`SemanticIrDocument::digest`]).
//!
//! The crate reuses `agentflow.semantic.core@1` objects by identity — the
//! four Phase 3A tables are validated by [`SemanticCoreDocument::check`] and
//! never redeclared. It does **not** depend on `af-spec`, reads no
//! filesystem, no registry, no clock and no network; the schema universe and
//! the module set are explicit inputs.
//!
//! Deliberately absent (this is a static model, not a runtime):
//! `CallComposite` (authoring-only, lowered before this layer), Trigger
//! (deployment binding), any agent/model/tool opcode, journal events, reducer,
//! `SemanticExecutionState`, runtime commands, occurrence identity
//! (`OccurrencePath`/`AttemptId`/`InvocationId`), workers, schedulers,
//! regions, leases, timers, resource handles and any opcode state machine.
//! Dominance and value-availability analysis are deferred to Phase 3C.

pub mod canonical;
pub mod check;
pub mod error;
pub mod ids;
pub mod model;

pub use canonical::canonical_computations;
pub use check::CheckedIrDocument;
pub use error::{IrError, kind};
pub use ids::{
    BranchOrdinal, ComputationId, ContractRefId, HandlerOrdinal, PortNo, RunParameterId,
};
pub use model::{
    AwaitBody, CallSubflowBody, CompensationScopeBody, ComputationBody, ComputationDef,
    ContractRefEntry, ContractRefKind, FailureMatch, ForeachBody, HandleBody, HandleClause,
    HandleMatch, InputBinding, InvokeBody, IterateBody, LoopCarryBinding, ParallelBody,
    ParallelBranch, ParallelMode, ParallelOutputBinding, RaceBody, RaceBranch, ResourceUseBinding,
    RetryBody, RunParameterDecl, ScopeBody, SelectBody, SelectCase, SemanticIrDocument,
    SemanticVersionRef, SequenceBody, TimeoutBody, WhileBody, WorkflowOutputDecl,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::kind;
    use af_semantic::{DigestRefValue, TypeSchemaRef, TypeSchemaUniverse};

    const FIXTURE_SCHEMA: &str = "afidl 1;
schema agentflow.test.semantic@1;
record User { 1 name: string; }
";

    /// A universe containing the fixture schema, plus the exact
    /// `TypeSchemaRef` that pins `User` within it.
    fn fixture_universe() -> (TypeSchemaUniverse, TypeSchemaRef) {
        let compiled = af_idl::compile(FIXTURE_SCHEMA).expect("fixture compiles");
        let digest = compiled.digest("User").expect("fixture digest");
        let mut universe = TypeSchemaUniverse::new();
        universe
            .insert_compiled(&compiled)
            .expect("fixture registers");
        let reference = TypeSchemaRef {
            specification_id: "agentflow.test.semantic@1".into(),
            type_name: "User".into(),
            schema_digest: DigestRefValue {
                algorithm: "sha256".into(),
                domain: "type-schema".into(),
                hex: digest.hex().to_string(),
            },
        };
        (universe, reference)
    }

    fn minimal_document(user: &TypeSchemaRef) -> SemanticIrDocument {
        let user_ref_json = format!(
            r#"{{"specification_id": {:?}, "type_name": {:?}, "schema_digest":
               {{"algorithm": "sha256", "domain": "type-schema", "hex": {:?}}}}}"#,
            user.specification_id, user.type_name, user.schema_digest.hex
        );
        let template = r#"
            {
              "semantic_refs": [],
              "types": [
                {"id": 1, "body": {"primitive": "int64"}},
                {"id": 2, "body": {"primitive": "bool"}}
              ],
              "values": [
                {"id": 1, "type": 1},
                {"id": 2, "type": 2}
              ],
              "expressions": [
                {"id": 1, "result_type": 1, "body": {"value_slot": 1}},
                {"id": 2, "result_type": 2, "body": {"value_slot": 2}}
              ],
              "resources": [],
              "contracts": [],
              "run_parameters": [
                {"id": 1, "name": "limit", "type": 1, "value_slot": 1},
                {"id": 2, "name": "flag", "type": 2, "value_slot": 2}
              ],
              "workflow_outputs": [
                {"name": "done", "expression": 2}
              ],
              "computations": [
                {"id": 1, "outputs": [], "body": {"sequence": {"children": [2]}}},
                {"id": 2, "outputs": [], "body": {"scope": {"child": 3, "resources": []}}},
                {"id": 3, "outputs": [], "body": {"compensation_scope": {"child": 4}}},
                {"id": 4, "outputs": [], "body": {"await": {
                    "signal_type": USER_REF_PLACEHOLDER,
                    "correlation": 1}}}
              ],
              "root": 1
            }
            "#;
        let source = template.replace("USER_REF_PLACEHOLDER", &user_ref_json);
        serde_json::from_str(&source).expect("minimal document deserializes")
    }

    #[test]
    fn structural_shapes_round_trip_through_serde() {
        let (_universe, user) = fixture_universe();
        let document = minimal_document(&user);
        assert_eq!(document.computations.len(), 4);
        assert_eq!(document.root, ComputationId::from_raw(1));
        // FailureMatch requires a full TypeSchemaRef object; a bare number is
        // not a match.
        assert!(
            serde_json::from_str::<crate::model::FailureMatch>(r#"{"failure_type": 1}"#).is_err()
        );
    }

    #[test]
    fn zero_ids_are_representable_and_rejected() {
        let (universe, user) = fixture_universe();
        let mut document = minimal_document(&user);
        document.root = ComputationId::from_raw(0);
        let error = document.check(&universe).unwrap_err();
        assert_eq!(error.kind(), kind::INVALID_LOCAL_ID);
    }

    #[test]
    fn digest_pinning_is_exact() {
        let (universe, user) = fixture_universe();
        let mut document = minimal_document(&user);
        let mutated = TypeSchemaRef {
            schema_digest: DigestRefValue {
                hex: "1".repeat(64),
                ..user.schema_digest.clone()
            },
            ..user.clone()
        };
        document.computations[3].body = ComputationBody::Await(AwaitBody {
            signal_type: mutated,
            correlation: af_semantic::ExpressionId::from_raw(1),
        });
        let error = document.check(&universe).unwrap_err();
        assert_eq!(error.kind(), "type_schema_digest_mismatch");
    }

    #[test]
    fn the_minimal_document_validates() {
        let (universe, user) = fixture_universe();
        let document = minimal_document(&user);
        document
            .check(&universe)
            .expect("minimal document is valid");
    }
}
