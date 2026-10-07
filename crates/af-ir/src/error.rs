//! Structured semantic IR errors with stable category kinds.
//!
//! Categories are stable identifiers referenced by the negative conformance
//! corpus (`spec/conformance/semantic-ir/negative/`); they must not be renamed
//! casually. Errors raised by the reused semantic core ([`SemanticError`])
//! pass through with their own stable kinds.

use crate::ids::ComputationId;
use af_semantic::SemanticError;
use std::fmt;

/// Stable error-kind names used by the negative corpus.
pub mod kind {
    pub const INVALID_LOCAL_ID: &str = "invalid_local_id";
    pub const INVALID_PORT_NO: &str = "invalid_port_no";
    pub const INVALID_ORDINAL: &str = "invalid_ordinal";
    pub const INVALID_SEMANTIC_REF: &str = "invalid_semantic_ref";
    pub const DUPLICATE_SEMANTIC_REF: &str = "duplicate_semantic_ref";
    pub const DUPLICATE_RUN_PARAMETER: &str = "duplicate_run_parameter";
    pub const INVALID_RUN_PARAMETER_NAME: &str = "invalid_run_parameter_name";
    pub const DUPLICATE_RUN_PARAMETER_NAME: &str = "duplicate_run_parameter_name";
    pub const RUN_PARAMETER_TYPE_MISMATCH: &str = "run_parameter_type_mismatch";
    pub const INVALID_WORKFLOW_OUTPUT_NAME: &str = "invalid_workflow_output_name";
    pub const DUPLICATE_WORKFLOW_OUTPUT_NAME: &str = "duplicate_workflow_output_name";
    pub const DUPLICATE_CONTRACT_REF: &str = "duplicate_contract_ref";
    pub const INVALID_CONTRACT_DIGEST: &str = "invalid_contract_digest";
    pub const CONTRACT_DIGEST_DOMAIN: &str = "contract_digest_domain";
    pub const DUPLICATE_CONTRACT_IDENTITY: &str = "duplicate_contract_identity";
    pub const INVOKE_UNKNOWN_CONTRACT: &str = "invoke_unknown_contract";
    pub const INVOKE_CONTRACT_KIND: &str = "invoke_contract_kind";
    pub const CALL_SUBFLOW_UNKNOWN_CONTRACT: &str = "call_subflow_unknown_contract";
    pub const CALL_SUBFLOW_CONTRACT_KIND: &str = "call_subflow_contract_kind";
    pub const DUPLICATE_COMPUTATION: &str = "duplicate_computation";
    pub const UNKNOWN_COMPUTATION: &str = "unknown_computation";
    pub const DUPLICATE_OUTPUT_PORT: &str = "duplicate_output_port";
    pub const OUTPUT_UNKNOWN_SLOT: &str = "output_unknown_slot";
    pub const DUPLICATE_INPUT_PORT: &str = "duplicate_input_port";
    pub const DUPLICATE_RESOURCE_PORT: &str = "duplicate_resource_port";
    pub const UNKNOWN_RESOURCE_SLOT: &str = "unknown_resource_slot";
    pub const DUPLICATE_CARRY_SLOT: &str = "duplicate_carry_slot";
    pub const CARRY_UNKNOWN_SLOT: &str = "carry_unknown_slot";
    pub const CARRY_UNKNOWN_PORT: &str = "carry_unknown_port";
    pub const CARRY_TYPE_MISMATCH: &str = "carry_type_mismatch";
    pub const UNBOUND_VALUE_SLOT: &str = "unbound_value_slot";
    pub const DUPLICATE_VALUE_PRODUCER: &str = "duplicate_value_producer";
    pub const SEQUENCE_EMPTY: &str = "sequence_empty";
    pub const DUPLICATE_CHILD_REFERENCE: &str = "duplicate_child_reference";
    pub const MULTIPLE_STRUCTURAL_PARENTS: &str = "multiple_structural_parents";
    pub const COMPUTATION_CYCLE: &str = "computation_cycle";
    pub const UNREACHABLE_COMPUTATION: &str = "unreachable_computation";
    pub const UNKNOWN_ROOT: &str = "unknown_root";
    pub const ROOT_HAS_PARENT: &str = "root_has_parent";
    pub const SELECT_EMPTY: &str = "select_empty";
    pub const SELECTOR_NOT_INT64: &str = "selector_not_int64";
    pub const DUPLICATE_CASE_ORDINAL: &str = "duplicate_case_ordinal";
    pub const SELECT_INTERFACE_MISMATCH: &str = "select_interface_mismatch";
    pub const SEQUENCE_INTERFACE_MISMATCH: &str = "sequence_interface_mismatch";
    pub const PARALLEL_EMPTY: &str = "parallel_empty";
    pub const DUPLICATE_BRANCH_ORDINAL: &str = "duplicate_branch_ordinal";
    pub const PARALLEL_OUTPUT_UNKNOWN_BRANCH: &str = "parallel_output_unknown_branch";
    pub const PARALLEL_OUTPUT_UNKNOWN_PORT: &str = "parallel_output_unknown_port";
    pub const PARALLEL_OUTPUT_TYPE_MISMATCH: &str = "parallel_output_type_mismatch";
    pub const PARALLEL_OUTPUT_DUPLICATE_PORT: &str = "parallel_output_duplicate_port";
    pub const RACE_INSUFFICIENT_BRANCHES: &str = "race_insufficient_branches";
    pub const RACE_INTERFACE_MISMATCH: &str = "race_interface_mismatch";
    pub const INVALID_MAX_INFLIGHT: &str = "invalid_max_inflight";
    pub const INVALID_MAX_ITERATIONS: &str = "invalid_max_iterations";
    pub const FOREACH_SOURCE_NOT_LIST: &str = "foreach_source_not_list";
    pub const FOREACH_ITEM_TYPE_MISMATCH: &str = "foreach_item_type_mismatch";
    pub const WHILE_GUARD_NOT_BOOL: &str = "while_guard_not_bool";
    pub const RETRY_INVALID_DELAY: &str = "retry_invalid_delay";
    pub const RETRY_INTERFACE_MISMATCH: &str = "retry_interface_mismatch";
    pub const TIMEOUT_NOT_DURATION: &str = "timeout_not_duration";
    pub const TIMEOUT_INTERFACE_MISMATCH: &str = "timeout_interface_mismatch";
    pub const DUPLICATE_HANDLER_ORDINAL: &str = "duplicate_handler_ordinal";
    pub const DUPLICATE_TIMED_OUT: &str = "duplicate_timed_out";
    pub const DUPLICATE_FAILURE_MATCH: &str = "duplicate_failure_match";
    pub const HANDLE_INTERFACE_MISMATCH: &str = "handle_interface_mismatch";
    pub const SCOPE_UNKNOWN_RESOURCE: &str = "scope_unknown_resource";
    pub const SCOPE_DUPLICATE_RESOURCE: &str = "scope_duplicate_resource";
    pub const SCOPE_INTERFACE_MISMATCH: &str = "scope_interface_mismatch";
    pub const COMPENSATION_SCOPE_INTERFACE_MISMATCH: &str = "compensation_scope_interface_mismatch";
    pub const ENCODING_ERROR: &str = "encoding_error";
    // Phase 3C scoped dataflow analysis (raised by `analyze_dataflow`, never
    // by `check`; the analysis pass runs strictly after Phase 3B validation).
    pub const VALUE_NOT_AVAILABLE: &str = "value_not_available";
    pub const RESOURCE_NOT_VISIBLE: &str = "resource_not_visible";
}

/// Semantic IR validation, canonicalization and digest errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrError {
    /// An id or number is `0` (or a namespace/numbering rule on the raw value
    /// is violated). `namespace` names the offending space.
    InvalidLocalId { namespace: &'static str, id: u32 },
    /// A rule violation with a stable kind and a human-readable detail.
    Rule { kind: &'static str, detail: String },
    /// A reference to a computation that does not exist.
    UnknownComputation { id: ComputationId },
    /// The semantic core tables (types/values/expressions/resources) failed
    /// their own Phase 3A validation; the core error passes through.
    Core(SemanticError),
    /// Canonical bytes or the digest could not be computed.
    Encoding { detail: String },
}

impl IrError {
    /// A rule violation with the given stable kind.
    pub fn rule(kind: &'static str, detail: impl Into<String>) -> Self {
        IrError::Rule {
            kind,
            detail: detail.into(),
        }
    }

    /// Stable category identifier; referenced by the negative corpus.
    pub fn kind(&self) -> &'static str {
        match self {
            IrError::InvalidLocalId { .. } => kind::INVALID_LOCAL_ID,
            IrError::UnknownComputation { .. } => kind::UNKNOWN_COMPUTATION,
            IrError::Rule { kind, .. } => kind,
            IrError::Core(error) => error.kind(),
            IrError::Encoding { .. } => kind::ENCODING_ERROR,
        }
    }
}

impl fmt::Display for IrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrError::InvalidLocalId { namespace, id } => {
                write!(f, "invalid {namespace} identity {id}: ids are positive u32")
            }
            IrError::Rule { kind, detail } => write!(f, "{kind}: {detail}"),
            IrError::UnknownComputation { id } => {
                write!(f, "unknown computation {id}")
            }
            IrError::Core(error) => write!(f, "semantic core: {error}"),
            IrError::Encoding { detail } => write!(f, "encoding error: {detail}"),
        }
    }
}

impl std::error::Error for IrError {}

impl From<SemanticError> for IrError {
    fn from(error: SemanticError) -> Self {
        IrError::Core(error)
    }
}
