# Frozen dependency error imports

All known constants were read from the five actual error.rs sources; their real error carrier uses closed kind plus diagnostic detail. No structured countermodel is fabricated by parsing that detail. Full source hashes and nominal constant identities are in model@2.dependency_errors and independent error-registration Expected. No currently untouched known branch was omitted.

|Domain|Known kinds|Frozen source SHA-256|
|---|---|---|
|resource|12|`4476ff5065ddb89625e3c388fb4b606dbab6889bec387ba2a16e6bc252f66225`|
|flow|11|`887f4425dccff8093441b7fb599547dcc1fe8b93b25f9b827812af5e3628092a`|
|usage|11|`3b2db1cc8e0f6275dc44d02b88f13516213edd87e9df3fa1870fb3f4ce5fa1a1`|
|recovery|13|`3c5d2aab663e9f65d14930d68ea703f4e253612998b4be5f055fbd18a5583d9a`|
|execution|22|`7ab95b35e7346d6bc80464f97b1c78a95b02908d2bc3059f068f97acb3c8b93d`|

## resource

`af-resource::INVALID_RESOURCE_TYPE` → `invalid_resource_type` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-resource::UNKNOWN_RESOURCE_TYPE` → `unknown_resource_type` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-resource::RESOURCE_TYPE_DIGEST_MISMATCH` → `resource_type_digest_mismatch` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-resource::INVALID_RESOURCE_PORT` → `invalid_resource_port` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-resource::DUPLICATE_RESOURCE_PORT` → `duplicate_resource_port` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-resource::INVALID_RESOURCE_ATTRIBUTE` → `invalid_resource_attribute` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-resource::DUPLICATE_RESOURCE_ATTRIBUTE` → `duplicate_resource_attribute` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-resource::UNKNOWN_RESOURCE_ATTRIBUTE` → `unknown_resource_attribute` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-resource::RESOURCE_REQUIREMENT_UNSATISFIED` → `resource_requirement_unsatisfied` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-resource::RESOURCE_REQUIREMENT_UNPROVEN` → `resource_requirement_unproven` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-resource::RESOURCE_REQUIREMENT_STRENGTHENED` → `resource_requirement_strengthened` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-resource::RESOURCE_ENCODING_ERROR` → `resource_encoding_error` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

## flow

`af-flow::INVALID_INFORMATION_LABEL` → `invalid_information_label` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-flow::UNKNOWN_INFORMATION_LABEL` → `unknown_information_label` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-flow::DUPLICATE_INFORMATION_LABEL` → `duplicate_information_label` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-flow::INVALID_FLOW_SOURCE` → `invalid_flow_source` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-flow::INVALID_FLOW_SINK` → `invalid_flow_sink` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-flow::UNKNOWN_FLOW_SINK` → `unknown_flow_sink` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-flow::DUPLICATE_FLOW_CONSTRAINT` → `duplicate_flow_constraint` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-flow::FLOW_NOT_PROVEN` → `flow_not_proven` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-flow::FLOW_CONTRACT_RELAXED` → `flow_contract_relaxed` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-flow::PRIVILEGED_TRANSFORMATION_REQUIRED` → `privileged_transformation_required` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-flow::FLOW_ENCODING_ERROR` → `flow_encoding_error` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

## usage

`af-usage::INVALID_USAGE_METER` → `invalid_usage_meter` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-usage::UNKNOWN_USAGE_METER` → `unknown_usage_meter` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-usage::INCOMPATIBLE_METER_UNIT` → `incompatible_meter_unit` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-usage::INVALID_USAGE_PHASE` → `invalid_usage_phase` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-usage::DUPLICATE_USAGE_LIMIT` → `duplicate_usage_limit` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-usage::UNSUPPORTED_USAGE_AGGREGATION` → `unsupported_usage_aggregation` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-usage::USAGE_ARITHMETIC_OVERFLOW` → `usage_arithmetic_overflow` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-usage::USAGE_BOUND_UNPROVEN` → `usage_bound_unproven` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-usage::USAGE_CEILING_EXCEEDED` → `usage_ceiling_exceeded` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-usage::USAGE_COMPATIBILITY_UNPROVEN` → `usage_compatibility_unproven` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-usage::USAGE_ENCODING_ERROR` → `usage_encoding_error` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

## recovery

`af-recovery::INVALID_REPLAY_SEMANTICS` → `invalid_replay_semantics` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-recovery::INVALID_IDEMPOTENCY_CONTRACT` → `invalid_idempotency_contract` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-recovery::DUPLICATE_EFFECT_RECOVERY_CLAUSE` → `duplicate_effect_recovery_clause` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-recovery::UNKNOWN_RECOVERY_OPERATION` → `unknown_recovery_operation` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-recovery::RECOVERY_EFFECT_NOT_DECLARED` → `recovery_effect_not_declared` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-recovery::RECONCILIATION_REQUIRED` → `reconciliation_required` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-recovery::RECONCILIATION_EVIDENCE_MISSING` → `reconciliation_evidence_missing` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-recovery::IDEMPOTENCY_EVIDENCE_MISSING` → `idempotency_evidence_missing` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-recovery::COMPENSATION_OBLIGATION_UNRESOLVED` → `compensation_obligation_unresolved` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-recovery::UNSAFE_BLIND_RETRY` → `unsafe_blind_retry` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-recovery::RECOVERY_CONTRACT_UNPROVEN` → `recovery_contract_unproven` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-recovery::RECOVERY_CONTRACT_WEAKENED` → `recovery_contract_weakened` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-recovery::RECOVERY_ENCODING_ERROR` → `recovery_encoding_error` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

## execution

`af-execution::INVALID_EXECUTION_CAPABILITY` → `invalid_execution_capability` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-execution::UNKNOWN_EXECUTION_CAPABILITY` → `unknown_execution_capability` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-execution::UNSUPPORTED_CAPABILITY_VERSION` → `unsupported_capability_version` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-execution::CAPABILITY_REQUIREMENT_UNPROVEN` → `capability_requirement_unproven` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-execution::INVALID_COMPUTE_QUANTITY` → `invalid_compute_quantity` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-execution::QUANTITY_UNIT_MISMATCH` → `quantity_unit_mismatch` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-execution::INVALID_SANDBOX_REQUIREMENT` → `invalid_sandbox_requirement` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-execution::SANDBOX_ASSURANCE_UNPROVEN` → `sandbox_assurance_unproven` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-execution::UNKNOWN_TRUST_DOMAIN` → `unknown_trust_domain` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-execution::TRUST_DOMAIN_MISMATCH` → `trust_domain_mismatch` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-execution::UNKNOWN_REGION` → `unknown_region` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-execution::RESIDENCY_REQUIREMENT_UNPROVEN` → `residency_requirement_unproven` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-execution::UNKNOWN_RUNTIME_PROFILE` → `unknown_runtime_profile` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-execution::RUNTIME_ABI_INCOMPATIBLE` → `runtime_abi_incompatible` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-execution::DUPLICATE_EXECUTION_REQUIREMENT` → `duplicate_execution_requirement` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-execution::CONFLICTING_HARD_CONSTRAINT` → `conflicting_hard_constraint` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-execution::EXECUTION_PROFILE_REJECTED` → `execution_profile_rejected` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-execution::EXECUTION_PROFILE_UNPROVEN` → `execution_profile_unproven` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-execution::EXECUTION_FEASIBILITY_STRENGTHENED` → `execution_feasibility_strengthened` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-execution::EXECUTION_SECURITY_RELAXED` → `execution_security_relaxed` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-execution::EXECUTION_COMPATIBILITY_UNPROVEN` → `execution_compatibility_unproven` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

`af-execution::EXECUTION_ENCODING_ERROR` → `execution_encoding_error` (required diagnostic detail; applicability: existing typed dependency dispatcher only).

DependencyErrorDomain and domain-tagged DependencyErrorKind must agree. Unknown kind=>UnsupportedModel; missing required payload=>IncompleteFacts; domain mismatch=>InvalidFacts.69 executed error registration shapes preserve this boundary. A source error alone never proves a substitution Violation.
