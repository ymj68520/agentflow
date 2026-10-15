# Closed standalone predicate roster

Family/profile: `standalone_substitution`, distinct from contract_validation and the eight inherited families. Scope kind and PredicateFamily are separate. Stage is standalone_substitution. The authoritative full-field descriptors are model@2.case_descriptors; independently authored Expected is [case-registration.json](../../spec/conformance/g01-successor-staging/case-registration.json).

|Rank|Case|Kind|Requirement clause|Proof role|Applicability|
|---|---|---|---|---|---|
|0|checked_operand|substitution_evidence|substitution:checked_operand|mandatory_preflight|all valid subjects|
|1|context_association|proof_context|direct:proof_context|mandatory_preflight|all valid subjects|
|2|whole_body_equality|substitution_evidence|substitution:whole_body_equality|supporting_fact|context Proven|
|3|changed_pair_evidence_floor|substitution_evidence|substitution:changed_pair_evidence_floor|supporting_fact|context Proven and body changed|
|4|lineage_equality|substitution_evidence|substitution:lineage_equality|supporting_fact|context Proven and body changed|
|5|io_relation|substitution_evidence|substitution:io_relation|supporting_fact|context Proven and body changed|
|6|failure_relation|substitution_evidence|substitution:failure_relation|supporting_fact|context Proven and body changed|
|7|effects_relation|substitution_evidence|substitution:effects_relation|supporting_fact|context Proven and body changed|
|8|resources_relation|substitution_evidence|substitution:resources_relation|supporting_fact|context Proven and body changed|
|9|flow_relation|substitution_evidence|substitution:flow_relation|supporting_fact|context Proven and body changed|
|10|usage_relation|substitution_evidence|substitution:usage_relation|supporting_fact|context Proven and body changed|
|11|recovery_relation|substitution_evidence|substitution:recovery_relation|supporting_fact|context Proven and body changed|
|12|execution_feasibility_relation|substitution_evidence|substitution:execution_feasibility_relation|supporting_fact|context Proven and body changed|
|13|execution_security_relation|substitution_evidence|substitution:execution_security_relation|supporting_fact|context Proven and body changed|
|14|execution_combined_relation|substitution_evidence|substitution:execution_combined_relation|supporting_fact|context Proven and body changed|
|15|result_identity|substitution_evidence|substitution:result_identity|static_dimension|all valid subjects|
|16|result_io|substitution_evidence|substitution:result_io|static_dimension|all valid subjects|
|17|result_failure|substitution_evidence|substitution:result_failure|static_dimension|all valid subjects|
|18|result_effects|substitution_evidence|substitution:result_effects|static_dimension|all valid subjects|
|19|result_resources|substitution_evidence|substitution:result_resources|static_dimension|all valid subjects|
|20|result_information_flow|substitution_evidence|substitution:result_information_flow|static_dimension|all valid subjects|
|21|result_usage|substitution_evidence|substitution:result_usage|static_dimension|all valid subjects|
|22|result_recovery|substitution_evidence|substitution:result_recovery|static_dimension|all valid subjects|
|23|result_execution|substitution_evidence|substitution:result_execution|static_dimension|all valid subjects|
|24|result_execution_feasibility|substitution_evidence|substitution:result_execution_feasibility|static_dimension|all valid subjects|
|25|result_execution_security|substitution_evidence|substitution:result_execution_security|static_dimension|all valid subjects|
|26|result_cross_dimension|substitution_evidence|substitution:result_cross_dimension|static_dimension|all valid subjects|

Exactly27 cases. checked_operand has the exact old/replacement parameter, not two invented cases. context_association uses direct:proof_context; the other26 cases form the closed SubstitutionClause. Every row also defines evidence_class, source/target/coordinate/origin, supporting relations, evaluation, witness and ordering in the full table. Unknown cases reject UnsupportedModel. All27 complete rows have distinct executed registration CaseIds; this does not capture facts.
