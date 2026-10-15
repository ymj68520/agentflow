# Closed Source/Target/Coordinate additions

**PROPOSED — NOT REGISTERED.** All reused payload types are their complete source-pinned logical types, never Display text or runtime reflection. Additions are only legal in the standalone_substitution profile; arbitrary family/kind/source cross-products reject.

```text
Source += contract_operand {role:OperandRole, subject:OperandSubject}
        | substitution_application {application:ApplicationOrigin,
                                    old:OperandSubject,replacement:OperandSubject}
Target += contract_operand {role:OperandRole, subject:OperandSubject}
        | substitution_pair {old:OperandSubject,replacement:OperandSubject}
Coordinate += substitution {application:ApplicationOrigin,
  dimension:Opt<SubstitutionDimension>, clause:RequirementClause,
  comparison:SubstitutionCoordinate}
SubstitutionCoordinate =
 lineage {old:ComponentIdentity,replacement:ComponentIdentity}
 | io {old_inputs:Seq<ComponentInputPort>,new_inputs:Seq<ComponentInputPort>,
       old_outputs:Seq<ComponentOutputPort>,new_outputs:Seq<ComponentOutputPort>}
 | failure {old:FailureRow,replacement:FailureRow}
 | effects {old:EffectSignature,replacement:EffectSignature}
 | resources {old:ResourceRequirementRow,replacement:ResourceRequirementRow,
              old_namespace:ComponentIdentity,new_namespace:ComponentIdentity}
 | flow {old:InformationFlowContract,replacement:InformationFlowContract,
         old_partitions:Seq<EffectFlowPartition>,new_partitions:Seq<EffectFlowPartition>}
 | usage {old:UsageSignature,replacement:UsageSignature,
          old_profiles:Seq<RecoveryUsageBinding>,new_profiles:Seq<RecoveryUsageBinding>}
 | recovery {old:RecoveryContract,replacement:RecoveryContract,
             old_bindings:Seq<RecoveryActionBinding>,new_bindings:Seq<RecoveryActionBinding>}
 | execution {old:ExecutionRequirements,replacement:ExecutionRequirements,
              aspect:combined|feasibility|security}
 | whole_pair {old:ContractContents,replacement:ContractContents}
 | context_roles {analysis:FullContext,old_checked:FullContext,
                  replacement_checked:FullContext,association:ContextRolePartition}
 | operand_validation {operand:OperandSubject}
```

Pair/application/operand role is identity, not inferred from argument positions in a reason. Whole pair is appropriate for exact-body and cross-dimension subject, not a substitute for twelve distinct coordinates. Every result row uses its matching clause/dimension and the specific coordinate in the roster. Whole-context values are referenced through a future validated typed DAG or borrowed locally; no checksum-only context. Coordinates contain comparison **inputs**, not final FactKeys/Report/outputs in Scope. Full errors/candidate verdicts are separate fact payloads, not identity inputs.

Coordinate.clause accepts exactly the enclosing registered RequirementClause:direct(proof_context) for the context-association case,substitution(the matching non-context clause) otherwise. This is a closed compatibility relation,not permission for unrelated direct/control/recovery clauses.

For12 results and local directional comparisons, Source=actual replacement operand; Target=actual old operand. For checked_operand Source/Target both name that one role; context/whole equality/floor use application Source and pair Target. No caller_ir/computation/root/target-export is required. A missing mandatory source/role/coordinate yields IncompleteFacts/InvalidSourceIdentity/InvalidCoordinate; unknown descriptors yield UnsupportedModel. None is used only for supporting cases with no dimension, never missing comparison data.

References: add `contract_operand{role,subject,member_path}` and `application_declaration{application}` to EvidenceRef in the successor model. They name complete scoped input members. Derive both role-tagged operand refs plus application ref for each result; include exact nominal schema/input-output-resource port/operation/atom/usage/recovery/execution/context members consulted by that predicate. Reuse existing digest/schema/port/scope_member refs where semantically exact; field/index/member paths are closed typed input paths, not free-form delimiters. Required local member sets come from actual structured operands, not a default empty collection. Full ref set/minimum-primary follows0208/0209; standalone facts are not the six root whole-boundary exception.
