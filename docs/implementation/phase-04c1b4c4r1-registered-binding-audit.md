# Registered binding audit

**PROPOSED — NOT REGISTERED** applies to recommendations only; this section inventories actual frozen content. Authoritative sources: `spec/computation/proof-obligation-semantics.md` (OS), `spec/tables/proof-report-model-v1.md` (RM), `spec/portable/portable-proof-inputs-v2.md` (PP2), and `spec/component/component-contract.md#component-contract-substitution` /AF-CONTRACT-0020. The RM JSON block, not the earlier planning map, defines the closed roster. Native/AFIDL ComponentIdentity both contain exactly catalog/name.

## Complete actual PredicateFamily/case roster

- root_preflight: assertion_input,exact_pin,proof_context,compiler_identity,subflow_dependency.
- root_dimension: I,O,Phi,epsilon,R,F,U,K,X.
- execution_child: source,feasibility,security.
- root_boundary: body,implementation,scope_authority,scope_budget,runtime,publisher.
- leaf: await-control-semantics,await-signal-pin,contract-pin-validation,effect-and-recovery,failure-usage-execution,information-flow,input-binding,output-binding,proof-context-consistency,reference-resolution,resource-mapping,scoped-dataflow,subflow-body-conformance,subflow-dependency-closure,subflow-export-contract,subflow-interface,target-pin-validation,type-compatibility.
- control: compensation-commit-order,compensation-forward-history,compensation-recovery-coverage,conditional-compensation-actions,control-contract-boundary,control-topology,control-tree-closure,effect-envelope,execution-per-child,failure-row,information-flow,local-k-action-bounds,occurrence-identity,parallel-fail-fast,proof-context-consistency,race-loser-effects,recovery-blocked-outcome,recovery-obligations,resource-port-provenance,retry-logical-effect-narrowing,retry-recovery-safety,root-export-compatibility,root-k-action-effect-envelope,root-k-action-epsilon-coverage,root-k-epsilon-coverage-requirement,root-k-recovery-usage,root-k-recovery-usage-coverage,root-recovery-action-totals,root-recovery-eligibility,root-recovery-guard-compatibility,root-recovery-multiplicity,root-recovery-source-closure,scope-authority-attenuation,scope-budget-attenuation,scope-child-summary,scope-owned-child-settlement,scope-resource-attenuation,select-case-totality,timeout-control-outcomes,typed-output-interface,unsupported-control,usage.
- root_recovery: root-k-action-effect-envelope,root-k-action-epsilon-coverage,root-k-epsilon-coverage-requirement,root-k-recovery-usage,root-k-recovery-usage-coverage,root-recovery-action-totals,root-recovery-eligibility,root-recovery-guard-compatibility,root-recovery-multiplicity,root-recovery-source-closure.
- prerequisite: closed_contract_schema,imported_module_closure,schema_universe,positive_unique_ports,failure_pins,effect_operation_membership,resource_universe,effect_resource_relation,flow_catalogs,io_resource_namespaces,additive_meter_units,recovery_usage_exhaustiveness,closed_effect_action_coverage,exact_k_identity,execution_universes,flow_execution_relation,required_dimensions,exact_relation_pins,epsilon_nonempty,flow_constraints_require_relation. Last2 are applicability-query predicates, not2 additional general missing-premise clauses.

There is no contract_validation or substitution family. Complete existing machine roster is also retained in `phase-04c1b4c4-admission-audit.json`; its values were rechecked against the actual registered source in R1.

## Remaining closed vocabulary (actual)

Kind, ranks1–25: assertion_input,exact_pin,proof_context,resolver_provenance,compiler_identity,subflow_dependency,input_mapping,output_producer,failure_containment,effect_containment,resource_mapping,information_flow,usage_coverage,recovery_coverage,execution_source,execution_feasibility,execution_security,body_conformance,implementation_conformance,scope_authority,scope_budget,runtime_authority,publisher_authentication,control_evidence,prerequisite.

RequirementClause: direct(kind1–13,15–23); recovery(source_closure,action_binding,usage_profile,multiplicity,guard_compatibility,epsilon_coverage,recovery_usage_coverage,eligibility,commit_order); control(resolved_leaf,topology,output_interface,selector_totality,effect_envelope,failure_row,resource_provenance,information_flow,usage,retry_safety,child_settlement,forward_history,compensation_coverage,root_export_boundary,unsupported_control,recovery_blocked_outcome); prerequisite(the18 non-query cases above). Kind/clause must agree. EvidenceClass: static_declaration,typed_counterexample_resolution,trusted_body,implementation_attestation,provider_receipt,authority_attestation,publisher_authentication,control_terminal_state,committed_effect_order,recovery_outcome,provider_honoring,application_idempotency. Role: mandatory_preflight,static_dimension,execution_child,independent_boundary,supporting_fact.

Source variants: root,component,subflow,scope,context,compiler,control. Target: analysis_boundary,export,component,subflow,relation,proof,unresolved_request. Coordinate: whole,input,output,resource,effect,flow,usage,recovery,execution,dependency,control,context_member,pin. Full payloads are RM.core_type_definitions; particularly Source.component requires caller_ir/node/contract, while Coordinate.input/output/resource carry Root* bindings; Coordinate.execution requires a source node. These cannot be supplied by a legal standalone input by inventing a caller or root export. ScopeSubject: root_export,contract_validation,computation. Only contract_validation has optional old_checked, but its raw candidate/one context layout is not an explicit ordered two-checked-contract comparison profile.

## Exact-reuse check against all eight admission conditions

| Candidate | Two checked operands/full lineage | No invented caller/root | Ordered roles/context premise | Actual local+row behavior and12 cases | Conclusion |
| --- | --- | --- | --- | --- | --- |
| root_preflight/exact_pin | IR/export pin relation, not catalog/name | Requires root assertion | Root profile | No standalone12 roster | Not equivalent |
| leaf/type-compatibility or reference-resolution | Call-site I/O/target predicates | Requires leaf/caller | Leaf binding | Not lineage or whole substitution | Not equivalent |
| control/root-export-compatibility | Control boundary, not two contracts | Requires control/root | Wrong subject/profile | No standalone12 roster | Not equivalent |
| prerequisite/required_dimensions | Validity/consumer premise only | Context operands possible | Does not define ordered substitution | Does not name lineage equality or12 results | Not sufficient |
| contract_validation Scope + supporting_fact role | Legal fragment envelope only | No fake caller necessary | old_checked optional, new raw candidate | Neither is PredicateFamily/Case/Requirement | Necessary infrastructure, no exact binding |
| other closed cases | Complete roster above concerns root/control/leaf/K/X/boundary/premises | Their substantive subjects retained | Cannot change by carrier name | No approved substitution mapping | No exact binding found |

AF-CONTRACT-0020 is a governing substantive rule, not a PredicateId. Finding persists by semantic input/subject comparison, not merely a missing spelling. New binding design must preserve both operand ordering and every required comparison; reusing just a dimension name or all-purpose supporting_fact would fail conditions1–8.
