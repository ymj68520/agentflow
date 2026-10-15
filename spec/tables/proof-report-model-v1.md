# agentflow.computation.proof-report.model@1 — closed non-wire semantic model

Status: normative. The single JSON block under Closed Descriptor Table is the complete registered model declaration. Its generic constructors have the exact closed meanings below;unknown materialization fails closed. Full fixed imported logical declarations are included,not only their names/digests. Native lifetimes/borrows are not wire values;the explicit logical Context/Compiler/Scope definitions override storage layout. Raw(T) is a closed companion-family transformation,not runtime reflection. Every concrete reachable descriptor must be materialized and validated before support can be activated. No AFIDL field/case numbers or HashV1 domains are assigned.

## Canonical interpretation

ACE1 is schema-directed. Products emit all fields in unsigned UTF8 field-name order;semantic comparison follows declaration order. Numerics retain nominal widths and exact canonical text;bytes are unpadded base64url;options retain explicit tags;invalid raw claims remain bytes. Sets use typed keys/full payload;sequences keep declaration order. Identity order is edition,scope,assertion path,stage,derived role,dimension,kind,source,source ordinal,mapping ordinal,target,coordinate,rule,requirement,origin. Role is uniquely derived from predicate roster and full scope. Numeric2 precedes10. Rank order is not a compatibility lattice.

Finite DAG construction validates types/cycles/reachability,interns equal typed values bottom-up,orders by height then type/nonrecursive payload/ordered child ranks,and rejects redundant nodes or alternative numbering. Sharing and equal copies denote the same value;independent declaration origins stay distinct. No Scope/final-report descendant or recursive digest fixed point is allowed. Exact decimals compare as rationals,finite floats mathematically after frozen zero canonicalization;None<Some,text/bytes unsigned lexicographic,records/tuples/lists by declared fields/prefix.

Work charges every node/edge/typed comparison/sort comparison/exact64-bit numeric limb and64-byte scanned/emitted block. The output bound includes every repeated full Scope in standalone identity strings and typed records. All limits intersect earlier D0–D10 limits;no limit raises those limits. Overflow,unknown model,incomplete inventory,invalid graph or overbudget returns an explicit failure and no partial successful view/capability.

## Closed Descriptor Table

```json
{
  "bounds": {
    "assertion_nodes": 1024,
    "canonical_view_bytes": 134217728,
    "combined_fact_prerequisite_obligation_keys": 65536,
    "depth": 96,
    "reference_observation_dependency_edges": 262144,
    "source_occurrences": 65536,
    "value_nodes": 1048576,
    "work": 4194304
  },
  "closed_domains": "- ContextCatalogRole = modules,type_schemas,effect_catalogs,resource_schemas,resource_descriptors,flow_labels,flow_taints,flow_sinks,usage_meters,recovery_effects,reconciliation_operations,execution_capabilities,trust_domains,regions,region_compatibility,runtime_profiles,effect_resource_relation,recovery_usage_relation,flow_execution_relation,resolver.\n- PinRole = caller_ir,root_ir,export_contract,component_contract,subflow_ir,subflow_export,type_schema,relation,compiler.\n- DigestRole = semantic_ir,component_contract,type_schema; only existing registered domain combinations.\n- PortDirection = input,output,resource.\n- SourceRole = component,subflow,await,owned_child,recovery_action,execution_child.\n- ClaimType = pin,identifier,compiler,context,contract,assertion_mapping.\n- UsageCoverageRole = forward_envelope,recovery_envelope,action_recovery_coverage.\n- ExecutionAspect = source_coverage,feasibility,security.\n- ResolverDifference = component_pins,target_absent,direct_dependencies,target_export,additional_targets,association_identity (six current branches, all operands preserved).\n- ControlOperator is the existing13-opcode closed Semantic IR domain; payloads retain actual mode/bound/selector fields, not a newly invented opcode.",
  "closed_predicates": {
    "control": [
      "compensation-commit-order",
      "compensation-forward-history",
      "compensation-recovery-coverage",
      "conditional-compensation-actions",
      "control-contract-boundary",
      "control-topology",
      "control-tree-closure",
      "effect-envelope",
      "execution-per-child",
      "failure-row",
      "information-flow",
      "local-k-action-bounds",
      "occurrence-identity",
      "parallel-fail-fast",
      "proof-context-consistency",
      "race-loser-effects",
      "recovery-blocked-outcome",
      "recovery-obligations",
      "resource-port-provenance",
      "retry-logical-effect-narrowing",
      "retry-recovery-safety",
      "root-export-compatibility",
      "root-k-action-effect-envelope",
      "root-k-action-epsilon-coverage",
      "root-k-epsilon-coverage-requirement",
      "root-k-recovery-usage",
      "root-k-recovery-usage-coverage",
      "root-recovery-action-totals",
      "root-recovery-eligibility",
      "root-recovery-guard-compatibility",
      "root-recovery-multiplicity",
      "root-recovery-source-closure",
      "scope-authority-attenuation",
      "scope-budget-attenuation",
      "scope-child-summary",
      "scope-owned-child-settlement",
      "scope-resource-attenuation",
      "select-case-totality",
      "timeout-control-outcomes",
      "typed-output-interface",
      "unsupported-control",
      "usage"
    ],
    "execution_child": [
      "source",
      "feasibility",
      "security"
    ],
    "leaf": [
      "await-control-semantics",
      "await-signal-pin",
      "contract-pin-validation",
      "effect-and-recovery",
      "failure-usage-execution",
      "information-flow",
      "input-binding",
      "output-binding",
      "proof-context-consistency",
      "reference-resolution",
      "resource-mapping",
      "scoped-dataflow",
      "subflow-body-conformance",
      "subflow-dependency-closure",
      "subflow-export-contract",
      "subflow-interface",
      "target-pin-validation",
      "type-compatibility"
    ],
    "prerequisite": [
      "closed_contract_schema",
      "imported_module_closure",
      "schema_universe",
      "positive_unique_ports",
      "failure_pins",
      "effect_operation_membership",
      "resource_universe",
      "effect_resource_relation",
      "flow_catalogs",
      "io_resource_namespaces",
      "additive_meter_units",
      "recovery_usage_exhaustiveness",
      "closed_effect_action_coverage",
      "exact_k_identity",
      "execution_universes",
      "flow_execution_relation",
      "required_dimensions",
      "exact_relation_pins",
      "epsilon_nonempty",
      "flow_constraints_require_relation"
    ],
    "root_boundary": [
      "body",
      "implementation",
      "scope_authority",
      "scope_budget",
      "runtime",
      "publisher"
    ],
    "root_dimension": [
      "I",
      "O",
      "Phi",
      "epsilon",
      "R",
      "F",
      "U",
      "K",
      "X"
    ],
    "root_preflight": [
      "assertion_input",
      "exact_pin",
      "proof_context",
      "compiler_identity",
      "subflow_dependency"
    ],
    "root_recovery": [
      "root-k-action-effect-envelope",
      "root-k-action-epsilon-coverage",
      "root-k-epsilon-coverage-requirement",
      "root-k-recovery-usage",
      "root-k-recovery-usage-coverage",
      "root-recovery-action-totals",
      "root-recovery-eligibility",
      "root-recovery-guard-compatibility",
      "root-recovery-multiplicity",
      "root-recovery-source-closure"
    ]
  },
  "core_type_definitions": "Scope = {\n  edition: ObligationEdition,\n  analysis_semantics: OwnerVersion,\n  content: CanonicalValueDag,\n  subject: ScopeSubject\n}\nScopeSubject =\n  root_export { caller_ir:DigestRef, top_export:DigestRef,\n    compiler:Node<Compiler>, context:Node<Context>,\n    resolver_associations:Node<ResolverAssociations>,\n    root_provenance:Node<RootProvenance>, associations:ContextPartition,\n    assertion_inputs:Node<AssertionGraph> }\n| contract_validation { compiler:Node<Compiler>, context:Node<Context>,\n    associations:ContextPartition, candidate:Node<RawCandidateContract>,\n    old_checked:Opt<Node<CheckedContractContents>> }\n| computation { compiler:Node<Compiler>, caller_ir:DigestRef,\n    context:Node<Context>, resolver_associations:Node<ResolverAssociations>,\n    root_provenance:Node<RootProvenance>, associations:ContextPartition,\n    owned_root:ComputationId }\n\nRuleRef = { specification_id:SpecificationId, rule_id:RuleId, anchor:Text }\nAssertionPath = Seq<{caller_ir:DigestRef, call_site:ComputationId,\n                    target_ir:DigestRef, target_export:Opt<DigestRef>}>\nPredicateId = {family:PredicateFamily, case:ClosedPredicateCase, rule:RuleRef}\nOrigin = {predicate:PredicateId, declared_path:Seq<DeclaredStep>,\n          source_ordinal:Opt<Ordinal64>, mapping_ordinal:Opt<Ordinal64>}\nDeclaredStep = field {owner:ModelType, field:ClosedField}\n             | entry {index:Ordinal64}\n             | member {key:TypedValue}\nFactKey = {stage:Stage, assertion_path:AssertionPath, kind:Kind,\n  dimension:Dimension, source:Source, target:Target, coordinate:Coordinate,\n  rule:RuleRef, requirement:Requirement, origin:Origin}\nObligationIdentity = {edition:ObligationEdition, scope:Scope,\n  assertion_path:AssertionPath, stage:Stage, kind:Kind, dimension:Dimension,\n  source:Source, target:Target, coordinate:Coordinate,\n  rule:RuleRef, requirement:Requirement, origin:Origin}\n\nFact = {key:FactKey, role:Role, evaluation:Evaluation, status:Status,\n  prerequisites:Set<PrerequisiteKey>, references:Set<EvidenceRef>,\n  primary:Opt<EvidenceRef>, contradiction:Opt<TypedContradiction>,\n  cause_fact_keys:Set<FactKey>, companion_evidence_status:Opt<ProofVerdict>,\n  observations:NonemptySet<ObservationSite>}\nEvaluation = evaluated:unit | blocked_by:NonemptySet<FactKey>\nRole = mandatory_preflight | static_dimension | execution_child\n     | independent_boundary | supporting_fact\nStatus = proof:ProofVerdict | authority:AuthorityStatus\nProofVerdict = Proven | Violation | Unproven | RequiresEvidence\nAuthorityStatus = NotEstablished\nDisposition = missing_static_proof | needs_independent_evidence\n            | contradiction_remains | authority_not_established\nObligation = {identity:ObligationIdentity, status:Status,\n  disposition:Disposition, prerequisite_links:Set<PrerequisiteKey>,\n  references:Set<EvidenceRef>, primary:Opt<EvidenceRef>,\n  contradiction:Opt<TypedContradiction>,\n  companion_evidence_status:Opt<ProofVerdict>,\n  observed_at:NonemptySet<ObservationSite>}\nPrerequisiteKey = {edition:ObligationEdition, scope:Scope,\n  stage:Stage, predicate:PrerequisitePredicate, source:Source, target:Target,\n  coordinate:Coordinate, rule:RuleRef, requirement:Requirement, origin:Origin}\nPrerequisite = {key:PrerequisiteKey, applicability:Applicability,\n  status:PrerequisiteStatus, supporting_fact:Opt<FactKey>,\n  contradiction:Opt<TypedContradiction>, consumers:NonemptySet<ConsumerEdge>}\nApplicability = applies | unknown\nPrerequisiteStatus = established | missing | requires_evidence | contradicted\nConsumerEdge = {consumer:FactKey, mandatory:Bool}\n\nCompleteView = {shape:ModelEdition, scope:Scope,\n facts:Map<FactKey,Fact>, prerequisites:Map<PrerequisiteKey,Prerequisite>,\n obligations:Seq<Obligation>, identity_strings:Seq<Text>,\n blockers:Map<FactKey,NonemptySet<FactKey>>,\n contradictions:Set<TypedContradiction>}\nFragment = {shape:ModelEdition, profile:contract_validation|computation,\n            scope:Scope, facts:Map<FactKey,Fact>, prerequisites:Map<PrerequisiteKey,Prerequisite>, obligations:Seq<Obligation>, identity_strings:Seq<Text>, blockers:Map<FactKey,NonemptySet<FactKey>>, contradictions:Set<TypedContradiction>}\n\nSource =\n root {ir:DigestRef, root_node:ComputationId}\n| component {caller_ir:DigestRef,node:ComputationId,contract:DigestRef,\n             entry_ordinal:Opt<Ordinal64>}\n| subflow {caller_ir:DigestRef,call_site:ComputationId,target_ir:DigestRef,\n           export_state:ExportState}\n| scope {caller_ir:DigestRef,node:ComputationId}\n| context {member:Seq<MemberStep>,catalog_role:Opt<ContextCatalogRole>}\n| compiler {member:Seq<MemberStep>}\n| control {caller_ir:DigestRef,node:ComputationId}\nExportState = provided:DigestRef | absent:unit | unresolved:Request\nTarget =\n analysis_boundary:unit\n| export {digest:DigestRef,port:Opt<PortIdentity>}\n| component {digest:DigestRef}\n| subflow {ir:DigestRef,export_state:ExportState}\n| relation {schema:TypeSchemaRef,revision:Text,content:Bytes}\n| proof {key:FactKey}\n| unresolved_request {caller_ir:DigestRef,node:ComputationId,request:Request}\nRequest = contract_ref {id:ContractRefId,declared_pin:RawClaim}\n        | digest {role:PinRole,claim:RawClaim}\n        | schema {claim:RawClaim}\n        | source_lookup {caller_ir:DigestRef,node:ComputationId,declared_role:SourceRole}\n        | universe_member {catalog:ContextCatalogRole,member:TypedValue}\n\nCoordinate =\n whole:unit\n| input {binding:RootInputPortBinding,export_port:ComponentInputPortIdentity,run_parameter:RunParameterId,\n         slot:ValueSlotId,use_site:Opt<RootInputUseSite>}\n| output {binding:RootOutputPortBinding,export_port:ComponentOutputPortIdentity,name:RawClaim,\n          expression:ExpressionId,path:Seq<MemberStep>}\n| resource {binding:RootResourcePortBinding,source_port:PortIdentity,slot:ResourceSlotId,\n            export_port:ResourceRequirementPortIdentity,type:TypeSchemaRef,\n            attributes:Set<ResourceAttributeConstraint>,aliases:Set<PortIdentity>}\n| effect {binding:RootEffectCoordinateBinding,source:RootEffectOccurrenceIdentity,export_atom:EffectAtom,\n          export_guard:EffectGuard,bounds:OccurrenceBounds}\n| flow {binding:RootFlowCoordinateBinding,source:Opt<FlowSourceRef>,sink:Opt<FlowConstraint>,kind:FlowKind,\n        labels:Set<ConfidentialityLabelRef>,taints:Set<TaintRef>,\n        partition:Opt<EffectFlowPartition>,ordinal:Ordinal64}\n| usage {phase:UsagePhase,meter:UsageMeterRef,unit:UsageUnitRef,\n         aggregation:UsageAggregation,coverage_role:UsageCoverageRole,\n         source_bound:UsageUpperBound,export_bound:UsageUpperBound}\n| recovery {binding:Opt<RootRecoveryCoordinateBinding>,source:RecoveryActionSourceIdentity,action:RecoveryActionIdentity,\n            profile:Opt<RecoveryUsageBinding>,factors:Seq<MultiplicityFactor>,\n            path:Seq<DeclaredStep>}\n| execution {source_node:ComputationId,contract:DigestRef,\n             requirements:ExecutionRequirements,aspect:ExecutionAspect}\n| dependency {call_site:ComputationId,target:DigestRef,edge_ordinal:Ordinal64}\n| control {node:ComputationId,operator:ControlOperator,predicate:PredicateId,\n           edge_path:Seq<DeclaredStep>}\n| context_member {catalog:ContextCatalogRole,member:TypedValue}\n| pin {role:PinRole,expected:ClaimValue,observed:ClaimValue}\nClaimValue = validated {type:ModelType,value:TypedValue}\n           | raw {type:ClaimType,bytes:Bytes}\n\nEvidenceRef =\n computation {caller_ir:DigestRef,node:ComputationId}\n| digest {role:DigestRole,pin:DigestRef}\n| schema {pin:TypeSchemaRef}\n| port {owner:DigestRef,direction:PortDirection,number:PortNo}\n| operation {operation:EffectOperationRef}\n| effect_atom {source:RootEffectOccurrenceIdentity,atom:EffectAtom}\n| contract_ref {caller_ir:DigestRef,id:ContractRefId,\n                resolved:ExportState}\n| control_edge {caller_ir:DigestRef,owner:ComputationId,child:ComputationId,\n                path:Seq<DeclaredStep>}\n| opcode {caller_ir:DigestRef,node:ComputationId,operator:ControlOperator}\n| resolver_difference {case:ResolverDifference,expected:Node<ResolverAssociations>,\n                       observed:Node<ResolverAssociations>,target:Opt<DigestRef>}\n| fact {key:FactKey}\n| scope_member {path:Seq<MemberStep>}\n| unresolved_request {request:Request}\nMemberStep = field {owner:ModelType,field:ClosedField}\n           | member {key:TypedValue} | index {index:Ordinal64}\n\nTypedContradiction =\n pin_mismatch {role:PinRole,expected:ClaimValue,observed:ClaimValue}\n| missing_members {universe:TypedValue,required:Set<TypedValue>,missing:NonemptySet<TypedValue>}\n| noncontainment {relation:PredicateId,actual:TypedValue,ceiling:TypedValue,\n                  witness:TypedValue}\n| invalid_mapping {input:TypedValue,offending:NonemptySeq<DeclaredStep>,\n                   expected:TypedValue}\n| invalid_shape {candidate:ClaimValue,expected_descriptor:ModelType,\n                 offending:NonemptySeq<DeclaredStep>}\n| guard_countermodel {source:RecoveryActionSourceIdentity,source_guard:EffectGuard,\n                      target_guard:EffectGuard,bounds:OccurrenceBounds,\n                      countermodel:TypedValue}\n| source_mismatch {expected:TypedValue,observed:TypedValue}\n| prerequisite_contradiction {key:PrerequisiteKey,witness:TypedValue}\nObservationSite = {report_owner:ObservationOwner,\n field_path:Seq<MemberStep>,exposing_path:AssertionPath,\n exposing_node:Opt<ComputationId>}\nObservationOwner = root_export | contract_validation | leaf | control\n                 | root_recovery | portable_projection",
  "imported_afidl_declarations": [
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "component_identity",
              "no": 1,
              "type": [
                "ref",
                "agentflow.component.contract@1/ComponentIdentity"
              ]
            },
            {
              "name": "inputs",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.component.contract@1/ComponentInputPort"
                ]
              ]
            },
            {
              "name": "outputs",
              "no": 3,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.component.contract@1/ComponentOutputPort"
                ]
              ]
            },
            {
              "name": "failures",
              "no": 4,
              "type": [
                "ref",
                "agentflow.failure.row@1/FailureRow"
              ]
            },
            {
              "name": "effects",
              "no": 5,
              "type": [
                "ref",
                "agentflow.effect.signature@1/EffectSignature"
              ]
            },
            {
              "name": "resources",
              "no": 6,
              "type": [
                "ref",
                "agentflow.resource.requirement@1/ResourceRequirementRow"
              ]
            },
            {
              "name": "information_flow",
              "no": 7,
              "type": [
                "ref",
                "agentflow.information.flow@1/InformationFlowContract"
              ]
            },
            {
              "name": "usage",
              "no": 8,
              "type": [
                "ref",
                "agentflow.usage.signature@1/UsageSignature"
              ]
            },
            {
              "name": "recovery",
              "no": 9,
              "type": [
                "ref",
                "agentflow.recovery.contract@1/RecoveryContract"
              ]
            },
            {
              "name": "execution",
              "no": 10,
              "type": [
                "ref",
                "agentflow.execution.requirements@1/ExecutionRequirements"
              ]
            },
            {
              "name": "effect_flow_partitions",
              "no": 11,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.component.contract@1/EffectFlowPartition"
                ]
              ]
            },
            {
              "name": "recovery_action_bindings",
              "no": 12,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.component.contract@1/RecoveryActionBinding"
                ]
              ]
            },
            {
              "name": "recovery_usage_bindings",
              "no": 13,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.component.contract@1/RecoveryUsageBinding"
                ]
              ]
            }
          ]
        }
      ],
      "name": "ComponentContract",
      "schema": "agentflow.component.contract@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "catalog",
              "no": 1,
              "type": [
                "primitive",
                "string"
              ]
            },
            {
              "name": "name",
              "no": 2,
              "type": [
                "primitive",
                "string"
              ]
            }
          ]
        }
      ],
      "name": "ComponentIdentity",
      "schema": "agentflow.component.contract@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "port_no",
              "no": 1,
              "type": [
                "primitive",
                "uint32"
              ]
            },
            {
              "name": "type",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/TypeSchemaRef"
              ]
            }
          ]
        }
      ],
      "name": "ComponentInputPort",
      "schema": "agentflow.component.contract@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "port_no",
              "no": 1,
              "type": [
                "primitive",
                "uint32"
              ]
            },
            {
              "name": "type",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/TypeSchemaRef"
              ]
            }
          ]
        }
      ],
      "name": "ComponentOutputPort",
      "schema": "agentflow.component.contract@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "operation",
              "no": 1,
              "type": [
                "ref",
                "agentflow.effect.signature@1/EffectOperationRef"
              ]
            },
            {
              "name": "kind",
              "no": 2,
              "type": [
                "ref",
                "agentflow.information.flow@1/FlowKind"
              ]
            },
            {
              "name": "allowed_effects",
              "no": 3,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.effect.signature@1/EffectAtom"
                ]
              ]
            }
          ]
        }
      ],
      "name": "EffectFlowPartition",
      "schema": "agentflow.component.contract@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "identity",
              "no": 1,
              "type": [
                "ref",
                "agentflow.component.contract@1/RecoveryActionIdentity"
              ]
            }
          ]
        }
      ],
      "name": "RecoveryActionBinding",
      "schema": "agentflow.component.contract@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "forward_effect",
              "no": 1,
              "type": [
                "ref",
                "agentflow.effect.signature@1/EffectAtom"
              ]
            },
            {
              "name": "action_kind",
              "no": 2,
              "type": [
                "ref",
                "agentflow.component.contract@1/RecoveryActionKind"
              ]
            },
            {
              "name": "action_effect",
              "no": 3,
              "type": [
                "ref",
                "agentflow.effect.signature@1/EffectAtom"
              ]
            }
          ]
        }
      ],
      "name": "RecoveryActionIdentity",
      "schema": "agentflow.component.contract@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "reconciliation",
              "no": 1
            },
            {
              "name": "compensation",
              "no": 2
            }
          ]
        }
      ],
      "name": "RecoveryActionKind",
      "schema": "agentflow.component.contract@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "action",
              "no": 1,
              "type": [
                "ref",
                "agentflow.component.contract@1/RecoveryActionIdentity"
              ]
            },
            {
              "name": "activations_per_forward",
              "no": 2,
              "type": [
                "ref",
                "agentflow.effect.signature@1/OccurrenceBounds"
              ]
            },
            {
              "name": "attempts_per_activation",
              "no": 3,
              "type": [
                "ref",
                "agentflow.effect.signature@1/OccurrenceBounds"
              ]
            },
            {
              "name": "per_attempt_usage",
              "no": 4,
              "type": [
                "ref",
                "agentflow.usage.signature@1/UsageSignature"
              ]
            }
          ]
        }
      ],
      "name": "RecoveryUsageBinding",
      "schema": "agentflow.component.contract@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "bool"
        ]
      ],
      "name": "Bool",
      "schema": "agentflow.core.machine-primitives@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "bytes"
        ]
      ],
      "name": "Bytes",
      "schema": "agentflow.core.machine-primitives@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "decimal"
        ]
      ],
      "name": "Decimal",
      "schema": "agentflow.core.machine-primitives@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "duration"
        ]
      ],
      "name": "Duration",
      "schema": "agentflow.core.machine-primitives@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "float64"
        ]
      ],
      "name": "Float64",
      "schema": "agentflow.core.machine-primitives@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "int32"
        ]
      ],
      "name": "Int32",
      "schema": "agentflow.core.machine-primitives@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "int64"
        ]
      ],
      "name": "Int64",
      "schema": "agentflow.core.machine-primitives@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "null"
        ]
      ],
      "name": "Null",
      "schema": "agentflow.core.machine-primitives@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "string"
        ]
      ],
      "name": "String",
      "schema": "agentflow.core.machine-primitives@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "timestamp"
        ]
      ],
      "name": "Timestamp",
      "schema": "agentflow.core.machine-primitives@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "uint32"
        ]
      ],
      "name": "UInt32",
      "schema": "agentflow.core.machine-primitives@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "uint64"
        ]
      ],
      "name": "UInt64",
      "schema": "agentflow.core.machine-primitives@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "unit"
        ]
      ],
      "name": "Unit",
      "schema": "agentflow.core.machine-primitives@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "qualifiers",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.effect.signature@1/EffectQualifier"
                ]
              ]
            }
          ]
        }
      ],
      "name": "EffectQualifierProfile",
      "schema": "agentflow.effect.resource.compatibility@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "complete_operations",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.effect.signature@1/EffectOperationRef"
                ]
              ]
            },
            {
              "name": "supported_pairs",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.effect.resource.compatibility@1/SupportedEffectResourcePair"
                ]
              ]
            }
          ]
        }
      ],
      "name": "EffectResourceCompatibilityCatalog",
      "schema": "agentflow.effect.resource.compatibility@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "finite_exact_values",
              "no": 1,
              "type": [
                "list",
                [
                  "primitive",
                  "string"
                ]
              ]
            },
            {
              "name": "opaque_nfc_exact_string",
              "no": 2,
              "type": [
                "primitive",
                "unit"
              ]
            }
          ]
        }
      ],
      "name": "ScopeValueRule",
      "schema": "agentflow.effect.resource.compatibility@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "selector_key",
              "no": 1,
              "type": [
                "primitive",
                "string"
              ]
            },
            {
              "name": "resource_attribute_key",
              "no": 2,
              "type": [
                "primitive",
                "string"
              ]
            },
            {
              "name": "required",
              "no": 3,
              "type": [
                "primitive",
                "bool"
              ]
            },
            {
              "name": "values",
              "no": 4,
              "type": [
                "ref",
                "agentflow.effect.resource.compatibility@1/ScopeValueRule"
              ]
            }
          ]
        }
      ],
      "name": "SelectorKeyRule",
      "schema": "agentflow.effect.resource.compatibility@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "operation",
              "no": 1,
              "type": [
                "ref",
                "agentflow.effect.signature@1/EffectOperationRef"
              ]
            },
            {
              "name": "resource_type",
              "no": 2,
              "type": [
                "ref",
                "agentflow.resource.requirement@1/ResourceTypeRef"
              ]
            },
            {
              "name": "selector_keys",
              "no": 3,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.effect.resource.compatibility@1/SelectorKeyRule"
                ]
              ]
            },
            {
              "name": "allowed_qualifier_profiles",
              "no": 4,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.effect.resource.compatibility@1/EffectQualifierProfile"
                ]
              ]
            }
          ]
        }
      ],
      "name": "SupportedEffectResourcePair",
      "schema": "agentflow.effect.resource.compatibility@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "string"
        ]
      ],
      "name": "CatalogId",
      "schema": "agentflow.effect.signature@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "operation",
              "no": 1,
              "type": [
                "ref",
                "agentflow.effect.signature@1/EffectOperationRef"
              ]
            },
            {
              "name": "resource_selector",
              "no": 2,
              "type": [
                "ref",
                "agentflow.effect.signature@1/ResourceSelector"
              ]
            },
            {
              "name": "qualifiers",
              "no": 3,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.effect.signature@1/EffectQualifier"
                ]
              ]
            }
          ]
        }
      ],
      "name": "EffectAtom",
      "schema": "agentflow.effect.signature@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "atom",
              "no": 1,
              "type": [
                "ref",
                "agentflow.effect.signature@1/EffectAtom"
              ]
            },
            {
              "name": "bounds",
              "no": 2,
              "type": [
                "ref",
                "agentflow.effect.signature@1/OccurrenceBounds"
              ]
            },
            {
              "name": "guard",
              "no": 3,
              "type": [
                "ref",
                "agentflow.effect.signature@1/EffectGuard"
              ]
            }
          ]
        }
      ],
      "name": "EffectEntry",
      "schema": "agentflow.effect.signature@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "unconditional",
              "no": 1,
              "type": [
                "primitive",
                "unit"
              ]
            },
            {
              "name": "input_equals",
              "no": 2,
              "type": [
                "ref",
                "agentflow.effect.signature@1/EffectGuardInputEquals"
              ]
            }
          ]
        }
      ],
      "name": "EffectGuard",
      "schema": "agentflow.effect.signature@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "port_no",
              "no": 1,
              "type": [
                "primitive",
                "uint32"
              ]
            },
            {
              "name": "expected",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/PrimitiveConstant"
              ]
            }
          ]
        }
      ],
      "name": "EffectGuardInputEquals",
      "schema": "agentflow.effect.signature@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "natural",
              "no": 1
            },
            {
              "name": "none",
              "no": 2
            }
          ]
        }
      ],
      "name": "EffectIdempotencyQualifier",
      "schema": "agentflow.effect.signature@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "known",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.effect.signature@1/EffectEntry"
                ]
              ]
            },
            {
              "name": "residual",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.effect.signature@1/EffectResidual"
                ]
              ]
            }
          ]
        }
      ],
      "name": "EffectOpenRow",
      "schema": "agentflow.effect.signature@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "catalog",
              "no": 1,
              "type": [
                "ref",
                "agentflow.effect.signature@1/CatalogId"
              ]
            },
            {
              "name": "name",
              "no": 2,
              "type": [
                "ref",
                "agentflow.effect.signature@1/OperationName"
              ]
            }
          ]
        }
      ],
      "name": "EffectOperationRef",
      "schema": "agentflow.effect.signature@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "idempotency",
              "no": 1,
              "type": [
                "ref",
                "agentflow.effect.signature@1/EffectIdempotencyQualifier"
              ]
            }
          ]
        }
      ],
      "name": "EffectQualifier",
      "schema": "agentflow.effect.signature@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "catalog",
              "no": 1,
              "type": [
                "ref",
                "agentflow.effect.signature@1/CatalogId"
              ]
            },
            {
              "name": "selector",
              "no": 2,
              "type": [
                "ref",
                "agentflow.effect.signature@1/ResourceSelector"
              ]
            },
            {
              "name": "bounds",
              "no": 3,
              "type": [
                "ref",
                "agentflow.effect.signature@1/OccurrenceBounds"
              ]
            }
          ]
        }
      ],
      "name": "EffectResidual",
      "schema": "agentflow.effect.signature@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "closed",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.effect.signature@1/EffectEntry"
                ]
              ]
            },
            {
              "name": "open",
              "no": 2,
              "type": [
                "ref",
                "agentflow.effect.signature@1/EffectOpenRow"
              ]
            },
            {
              "name": "unknown",
              "no": 3,
              "type": [
                "primitive",
                "unit"
              ]
            }
          ]
        }
      ],
      "name": "EffectRow",
      "schema": "agentflow.effect.signature@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "row",
              "no": 1,
              "type": [
                "ref",
                "agentflow.effect.signature@1/EffectRow"
              ]
            }
          ]
        }
      ],
      "name": "EffectSignature",
      "schema": "agentflow.effect.signature@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "lower",
              "no": 1,
              "type": [
                "primitive",
                "uint64"
              ]
            },
            {
              "name": "upper",
              "no": 2,
              "type": [
                "ref",
                "agentflow.effect.signature@1/OccurrenceCeiling"
              ]
            }
          ]
        }
      ],
      "name": "OccurrenceBounds",
      "schema": "agentflow.effect.signature@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "finite",
              "no": 1,
              "type": [
                "primitive",
                "uint64"
              ]
            },
            {
              "name": "unbounded",
              "no": 2,
              "type": [
                "primitive",
                "unit"
              ]
            }
          ]
        }
      ],
      "name": "OccurrenceCeiling",
      "schema": "agentflow.effect.signature@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "string"
        ]
      ],
      "name": "OperationName",
      "schema": "agentflow.effect.signature@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "uint32"
        ]
      ],
      "name": "RequirementPortNo",
      "schema": "agentflow.effect.signature@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "requirement_port_no",
              "no": 1,
              "type": [
                "ref",
                "agentflow.effect.signature@1/RequirementPortNo"
              ]
            },
            {
              "name": "scope_bindings",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.effect.signature@1/ScopeBinding"
                ]
              ]
            }
          ]
        }
      ],
      "name": "ResourceSelector",
      "schema": "agentflow.effect.signature@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "key",
              "no": 1,
              "type": [
                "ref",
                "agentflow.effect.signature@1/ScopeKey"
              ]
            },
            {
              "name": "value",
              "no": 2,
              "type": [
                "ref",
                "agentflow.effect.signature@1/ScopeValue"
              ]
            }
          ]
        }
      ],
      "name": "ScopeBinding",
      "schema": "agentflow.effect.signature@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "string"
        ]
      ],
      "name": "ScopeKey",
      "schema": "agentflow.effect.signature@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "string"
        ]
      ],
      "name": "ScopeValue",
      "schema": "agentflow.effect.signature@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "key",
              "no": 1,
              "type": [
                "primitive",
                "string"
              ]
            },
            {
              "name": "value",
              "no": 2,
              "type": [
                "primitive",
                "string"
              ]
            }
          ]
        }
      ],
      "name": "CapabilityAttribute",
      "schema": "agentflow.execution.requirements@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "capability",
              "no": 1,
              "type": [
                "ref",
                "agentflow.execution.requirements@1/ExecutionCapabilityRef"
              ]
            },
            {
              "name": "exact_version",
              "no": 2,
              "type": [
                "primitive",
                "string"
              ]
            },
            {
              "name": "required_attributes",
              "no": 3,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.execution.requirements@1/CapabilityAttribute"
                ]
              ]
            }
          ]
        }
      ],
      "name": "CapabilityRequirement",
      "schema": "agentflow.execution.requirements@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "cpu_millicores",
              "no": 1
            },
            {
              "name": "memory_bytes",
              "no": 2
            },
            {
              "name": "gpu_count",
              "no": 3
            }
          ]
        }
      ],
      "name": "ComputeQuantityKind",
      "schema": "agentflow.execution.requirements@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "kind",
              "no": 1,
              "type": [
                "ref",
                "agentflow.execution.requirements@1/ComputeQuantityKind"
              ]
            },
            {
              "name": "minimum",
              "no": 2,
              "type": [
                "primitive",
                "uint64"
              ]
            }
          ]
        }
      ],
      "name": "ComputeQuantityRequirement",
      "schema": "agentflow.execution.requirements@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "catalog",
              "no": 1,
              "type": [
                "primitive",
                "string"
              ]
            },
            {
              "name": "name",
              "no": 2,
              "type": [
                "primitive",
                "string"
              ]
            }
          ]
        }
      ],
      "name": "ExecutionCapabilityRef",
      "schema": "agentflow.execution.requirements@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "capabilities",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.execution.requirements@1/CapabilityRequirement"
                ]
              ]
            },
            {
              "name": "quantities",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.execution.requirements@1/ComputeQuantityRequirement"
                ]
              ]
            },
            {
              "name": "sandbox",
              "no": 3,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.execution.requirements@1/SandboxRequirement"
                ]
              ]
            },
            {
              "name": "trust_domain",
              "no": 4,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.execution.requirements@1/TrustDomainRef"
                ]
              ]
            },
            {
              "name": "region_residency",
              "no": 5,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.execution.requirements@1/RegionResidencyRequirement"
                ]
              ]
            },
            {
              "name": "runtime_profile",
              "no": 6,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.execution.requirements@1/RuntimeProfileRequirement"
                ]
              ]
            },
            {
              "name": "implementation_abis",
              "no": 7,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.execution.requirements@1/ImplementationAbiRequirement"
                ]
              ]
            },
            {
              "name": "soft_preferences",
              "no": 8,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.execution.requirements@1/PlacementPreference"
                ]
              ]
            }
          ]
        }
      ],
      "name": "ExecutionRequirements",
      "schema": "agentflow.execution.requirements@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "schema",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/TypeSchemaRef"
              ]
            }
          ]
        }
      ],
      "name": "ImplementationAbiRequirement",
      "schema": "agentflow.execution.requirements@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "artifact_locality",
              "no": 1
            },
            {
              "name": "warm_model_cache",
              "no": 2
            },
            {
              "name": "lower_load",
              "no": 3
            },
            {
              "name": "lower_cost",
              "no": 4
            }
          ]
        }
      ],
      "name": "PlacementPreference",
      "schema": "agentflow.execution.requirements@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "catalog",
              "no": 1,
              "type": [
                "primitive",
                "string"
              ]
            },
            {
              "name": "name",
              "no": 2,
              "type": [
                "primitive",
                "string"
              ]
            }
          ]
        }
      ],
      "name": "RegionRef",
      "schema": "agentflow.execution.requirements@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "execution_regions",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.execution.requirements@1/RegionRef"
                ]
              ]
            },
            {
              "name": "residency_regions",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.execution.requirements@1/RegionRef"
                ]
              ]
            }
          ]
        }
      ],
      "name": "RegionResidencyRequirement",
      "schema": "agentflow.execution.requirements@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "catalog",
              "no": 1,
              "type": [
                "primitive",
                "string"
              ]
            },
            {
              "name": "name",
              "no": 2,
              "type": [
                "primitive",
                "string"
              ]
            }
          ]
        }
      ],
      "name": "RuntimeProfileRef",
      "schema": "agentflow.execution.requirements@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "family",
              "no": 1,
              "type": [
                "primitive",
                "string"
              ]
            },
            {
              "name": "profile",
              "no": 2,
              "type": [
                "ref",
                "agentflow.execution.requirements@1/RuntimeProfileRef"
              ]
            },
            {
              "name": "required_features",
              "no": 3,
              "type": [
                "list",
                [
                  "primitive",
                  "string"
                ]
              ]
            }
          ]
        }
      ],
      "name": "RuntimeProfileRequirement",
      "schema": "agentflow.execution.requirements@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "basic",
              "no": 1
            },
            {
              "name": "strong",
              "no": 2
            }
          ]
        }
      ],
      "name": "SandboxLevel",
      "schema": "agentflow.execution.requirements@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "minimum",
              "no": 1,
              "type": [
                "ref",
                "agentflow.execution.requirements@1/SandboxLevel"
              ]
            }
          ]
        }
      ],
      "name": "SandboxRequirement",
      "schema": "agentflow.execution.requirements@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "catalog",
              "no": 1,
              "type": [
                "primitive",
                "string"
              ]
            },
            {
              "name": "name",
              "no": 2,
              "type": [
                "primitive",
                "string"
              ]
            }
          ]
        }
      ],
      "name": "TrustDomainRef",
      "schema": "agentflow.execution.requirements@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "failures",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.failure.row@1/FailureTypeRef"
                ]
              ]
            }
          ]
        }
      ],
      "name": "FailureRow",
      "schema": "agentflow.failure.row@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "payload_type",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/TypeSchemaRef"
              ]
            }
          ]
        }
      ],
      "name": "FailureTypeRef",
      "schema": "agentflow.failure.row@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "authority",
              "no": 1,
              "type": [
                "ref",
                "agentflow.information.flow@1/InstructionAuthority"
              ]
            },
            {
              "name": "trust_domains",
              "no": 2,
              "type": [
                "ref",
                "agentflow.flow.execution.relation@1/TrustDomainPolicy"
              ]
            },
            {
              "name": "sandbox",
              "no": 3,
              "type": [
                "ref",
                "agentflow.flow.execution.relation@1/SandboxFloor"
              ]
            }
          ]
        }
      ],
      "name": "AuthorityExecutionRule",
      "schema": "agentflow.flow.execution.relation@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "label",
              "no": 1,
              "type": [
                "ref",
                "agentflow.information.flow@1/ConfidentialityLabelRef"
              ]
            },
            {
              "name": "region_pairs",
              "no": 2,
              "type": [
                "ref",
                "agentflow.flow.execution.relation@1/RegionPairPolicy"
              ]
            }
          ]
        }
      ],
      "name": "ConfidentialityExecutionRule",
      "schema": "agentflow.flow.execution.relation@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "kind",
              "no": 1,
              "type": [
                "ref",
                "agentflow.information.flow@1/FlowKind"
              ]
            },
            {
              "name": "sink",
              "no": 2,
              "type": [
                "ref",
                "agentflow.flow.execution.relation@1/FlowExecutionSink"
              ]
            },
            {
              "name": "trust_domains",
              "no": 3,
              "type": [
                "ref",
                "agentflow.flow.execution.relation@1/TrustDomainPolicy"
              ]
            },
            {
              "name": "sandbox",
              "no": 4,
              "type": [
                "ref",
                "agentflow.flow.execution.relation@1/SandboxFloor"
              ]
            }
          ]
        }
      ],
      "name": "FlowExecutionRule",
      "schema": "agentflow.flow.execution.relation@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "workflow_output",
              "no": 1,
              "type": [
                "primitive",
                "uint32"
              ]
            },
            {
              "name": "exact_effect",
              "no": 2,
              "type": [
                "ref",
                "agentflow.effect.signature@1/EffectAtom"
              ]
            }
          ]
        }
      ],
      "name": "FlowExecutionSink",
      "schema": "agentflow.flow.execution.relation@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "closed_regions",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.execution.requirements@1/RegionRef"
                ]
              ]
            },
            {
              "name": "closed_trust_domains",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.execution.requirements@1/TrustDomainRef"
                ]
              ]
            },
            {
              "name": "confidentiality_rules",
              "no": 3,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.flow.execution.relation@1/ConfidentialityExecutionRule"
                ]
              ]
            },
            {
              "name": "taint_rules",
              "no": 4,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.flow.execution.relation@1/TaintExecutionRule"
                ]
              ]
            },
            {
              "name": "authority_rules",
              "no": 5,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.flow.execution.relation@1/AuthorityExecutionRule"
                ]
              ]
            },
            {
              "name": "sink_rules",
              "no": 6,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.flow.execution.relation@1/FlowExecutionRule"
                ]
              ]
            }
          ]
        }
      ],
      "name": "InformationFlowExecutionCatalog",
      "schema": "agentflow.flow.execution.relation@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "execution_region",
              "no": 1,
              "type": [
                "ref",
                "agentflow.execution.requirements@1/RegionRef"
              ]
            },
            {
              "name": "residency_region",
              "no": 2,
              "type": [
                "ref",
                "agentflow.execution.requirements@1/RegionRef"
              ]
            }
          ]
        }
      ],
      "name": "RegionPair",
      "schema": "agentflow.flow.execution.relation@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "finite_exact_pairs",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.flow.execution.relation@1/RegionPair"
                ]
              ]
            },
            {
              "name": "all_pairs_in_closed_region_snapshot",
              "no": 2,
              "type": [
                "primitive",
                "unit"
              ]
            }
          ]
        }
      ],
      "name": "RegionPairPolicy",
      "schema": "agentflow.flow.execution.relation@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "no_additional_minimum",
              "no": 1,
              "type": [
                "primitive",
                "unit"
              ]
            },
            {
              "name": "minimum",
              "no": 2,
              "type": [
                "ref",
                "agentflow.execution.requirements@1/SandboxLevel"
              ]
            }
          ]
        }
      ],
      "name": "SandboxFloor",
      "schema": "agentflow.flow.execution.relation@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "taint",
              "no": 1,
              "type": [
                "ref",
                "agentflow.information.flow@1/TaintRef"
              ]
            },
            {
              "name": "trust_domains",
              "no": 2,
              "type": [
                "ref",
                "agentflow.flow.execution.relation@1/TrustDomainPolicy"
              ]
            },
            {
              "name": "sandbox",
              "no": 3,
              "type": [
                "ref",
                "agentflow.flow.execution.relation@1/SandboxFloor"
              ]
            }
          ]
        }
      ],
      "name": "TaintExecutionRule",
      "schema": "agentflow.flow.execution.relation@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "exact_allowlist",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.execution.requirements@1/TrustDomainRef"
                ]
              ]
            },
            {
              "name": "all_domains_in_closed_trust_snapshot",
              "no": 2,
              "type": [
                "primitive",
                "unit"
              ]
            },
            {
              "name": "no_additional_trust_requirement",
              "no": 3,
              "type": [
                "primitive",
                "unit"
              ]
            }
          ]
        }
      ],
      "name": "TrustDomainPolicy",
      "schema": "agentflow.flow.execution.relation@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "catalog",
              "no": 1,
              "type": [
                "ref",
                "agentflow.information.flow@1/LabelCatalogId"
              ]
            },
            {
              "name": "name",
              "no": 2,
              "type": [
                "ref",
                "agentflow.information.flow@1/LabelName"
              ]
            }
          ]
        }
      ],
      "name": "ConfidentialityLabelRef",
      "schema": "agentflow.information.flow@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "kind",
              "no": 1,
              "type": [
                "ref",
                "agentflow.information.flow@1/FlowKind"
              ]
            },
            {
              "name": "sink",
              "no": 2,
              "type": [
                "ref",
                "agentflow.information.flow@1/FlowSink"
              ]
            }
          ]
        }
      ],
      "name": "FlowConstraint",
      "schema": "agentflow.information.flow@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "data_egress",
              "no": 1
            },
            {
              "name": "decision_influence",
              "no": 2
            }
          ]
        }
      ],
      "name": "FlowKind",
      "schema": "agentflow.information.flow@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "workflow_output",
              "no": 1,
              "type": [
                "primitive",
                "uint32"
              ]
            },
            {
              "name": "external_effect",
              "no": 2,
              "type": [
                "ref",
                "agentflow.effect.signature@1/EffectOperationRef"
              ]
            }
          ]
        }
      ],
      "name": "FlowSink",
      "schema": "agentflow.information.flow@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "category",
              "no": 1,
              "type": [
                "ref",
                "agentflow.information.flow@1/SourceCategory"
              ]
            }
          ]
        }
      ],
      "name": "FlowSourceRef",
      "schema": "agentflow.information.flow@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "confidentiality_restrictions",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.information.flow@1/ConfidentialityLabelRef"
                ]
              ]
            },
            {
              "name": "summary",
              "no": 2,
              "type": [
                "ref",
                "agentflow.information.flow@1/StaticInformationSummary"
              ]
            },
            {
              "name": "flow_constraints",
              "no": 3,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.information.flow@1/FlowConstraint"
                ]
              ]
            }
          ]
        }
      ],
      "name": "InformationFlowContract",
      "schema": "agentflow.information.flow@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "none",
              "no": 1
            },
            {
              "name": "external_data",
              "no": 2
            },
            {
              "name": "delegated_agent",
              "no": 3
            },
            {
              "name": "user",
              "no": 4
            },
            {
              "name": "workflow",
              "no": 5
            },
            {
              "name": "platform",
              "no": 6
            }
          ]
        }
      ],
      "name": "InstructionAuthority",
      "schema": "agentflow.information.flow@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "string"
        ]
      ],
      "name": "LabelCatalogId",
      "schema": "agentflow.information.flow@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "string"
        ]
      ],
      "name": "LabelName",
      "schema": "agentflow.information.flow@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "workflow_input",
              "no": 1,
              "type": [
                "primitive",
                "uint32"
              ]
            },
            {
              "name": "resource_requirement",
              "no": 2,
              "type": [
                "primitive",
                "uint32"
              ]
            },
            {
              "name": "unknown_external",
              "no": 3,
              "type": [
                "primitive",
                "unit"
              ]
            }
          ]
        }
      ],
      "name": "SourceCategory",
      "schema": "agentflow.information.flow@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "sources",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.information.flow@1/FlowSourceRef"
                ]
              ]
            },
            {
              "name": "taints",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.information.flow@1/TaintRef"
                ]
              ]
            },
            {
              "name": "instruction_authority",
              "no": 3,
              "type": [
                "ref",
                "agentflow.information.flow@1/InstructionAuthority"
              ]
            },
            {
              "name": "influence_unknown",
              "no": 4,
              "type": [
                "primitive",
                "bool"
              ]
            }
          ]
        }
      ],
      "name": "StaticInformationSummary",
      "schema": "agentflow.information.flow@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "catalog",
              "no": 1,
              "type": [
                "ref",
                "agentflow.information.flow@1/LabelCatalogId"
              ]
            },
            {
              "name": "name",
              "no": 2,
              "type": [
                "ref",
                "agentflow.information.flow@1/LabelName"
              ]
            }
          ]
        }
      ],
      "name": "TaintRef",
      "schema": "agentflow.information.flow@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "id",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/AssertionId"
              ]
            },
            {
              "name": "ir_asset",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/TypedContentRef"
              ]
            },
            {
              "name": "export_contract_asset",
              "no": 3,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/TypedContentRef"
              ]
            },
            {
              "name": "assertion",
              "no": 4,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/PortableRootExportAssertion"
              ]
            }
          ]
        }
      ],
      "name": "AssertionEntry",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "alias",
        [
          "ref",
          "agentflow.portable.proof.inputs@1/U32"
        ]
      ],
      "name": "AssertionId",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "full_content_revalidation",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/IncompleteAttribution"
              ]
            },
            {
              "name": "rule_complete_attribution",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/CompleteAttributionClaim"
              ]
            },
            {
              "name": "portable_certificate",
              "no": 3,
              "type": [
                "primitive",
                "unit"
              ]
            }
          ]
        }
      ],
      "name": "AttributionProfile",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "bytes"
        ]
      ],
      "name": "Bytes",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "hash_domains",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/RegistryContent"
              ]
            },
            {
              "name": "abi_numbers",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/RegistryContent"
              ]
            },
            {
              "name": "rule_areas",
              "no": 3,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/RegistryContent"
              ]
            },
            {
              "name": "afidl_primitives",
              "no": 4,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/RegistryContent"
              ]
            }
          ]
        }
      ],
      "name": "ClosedTableContentSet",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "id",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/CompilerManifestId"
              ]
            },
            {
              "name": "content",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/CompilerSemanticsManifest"
              ]
            }
          ]
        }
      ],
      "name": "CompilerManifestEntry",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "alias",
        [
          "ref",
          "agentflow.portable.proof.inputs@1/U32"
        ]
      ],
      "name": "CompilerManifestId",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "registries",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/RegistryContentSet"
              ]
            },
            {
              "name": "closed_tables",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/ClosedTableContentSet"
              ]
            },
            {
              "name": "normative_sources",
              "no": 3,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/SourceContentIdentity"
                ]
              ]
            },
            {
              "name": "semantic_roles",
              "no": 4,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/SemanticRoleEntry"
                ]
              ]
            },
            {
              "name": "attribution",
              "no": 5,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/AttributionProfile"
              ]
            },
            {
              "name": "static_identity_projection",
              "no": 6,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/StaticIdentityProjection"
              ]
            }
          ]
        }
      ],
      "name": "CompilerSemanticsManifest",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "direct_rules",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/RuleContentIdentity"
                ]
              ]
            },
            {
              "name": "transitive_rules",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/RuleContentIdentity"
                ]
              ]
            },
            {
              "name": "dependency_edges",
              "no": 3,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/RuleDependencyEdge"
                ]
              ]
            }
          ]
        }
      ],
      "name": "CompleteAttributionClaim",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "exported_contract_digest",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/DigestRefValue"
              ]
            },
            {
              "name": "port_no",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/U32"
              ]
            }
          ]
        }
      ],
      "name": "ComponentInputPortIdentity",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "exported_contract_digest",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/DigestRefValue"
              ]
            },
            {
              "name": "port_no",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/U32"
              ]
            }
          ]
        }
      ],
      "name": "ComponentOutputPortIdentity",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "alias",
        [
          "ref",
          "agentflow.portable.proof.inputs@1/U32"
        ]
      ],
      "name": "ContextId",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "reference",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/TypedContentRef"
              ]
            },
            {
              "name": "canonical_content",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/Bytes"
              ]
            }
          ]
        }
      ],
      "name": "ContractAsset",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "specification_id",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/SpecId"
              ]
            },
            {
              "name": "operations",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/Text"
                ]
              ]
            }
          ]
        }
      ],
      "name": "EffectCatalogContent",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "header",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/RelationContentHeader"
              ]
            },
            {
              "name": "canonical_content",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/Bytes"
              ]
            }
          ]
        }
      ],
      "name": "EffectResourceRelationContent",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "capability",
              "no": 1,
              "type": [
                "ref",
                "agentflow.execution.requirements@1/ExecutionCapabilityRef"
              ]
            },
            {
              "name": "exact_version",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/Text"
              ]
            },
            {
              "name": "supported_versions",
              "no": 3,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/Text"
                ]
              ]
            },
            {
              "name": "attributes",
              "no": 4,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.execution.requirements@1/CapabilityAttribute"
                ]
              ]
            }
          ]
        }
      ],
      "name": "ExecutionCapabilityContent",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "profile",
              "no": 1,
              "type": [
                "ref",
                "agentflow.execution.requirements@1/RuntimeProfileRef"
              ]
            },
            {
              "name": "family",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/Text"
              ]
            },
            {
              "name": "features",
              "no": 3,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/Text"
                ]
              ]
            }
          ]
        }
      ],
      "name": "ExecutionRuntimeProfileContent",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "capabilities",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/PpiAuxExecutionUniverseContentPF1G1"
              ]
            },
            {
              "name": "trust_domains",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/PpiAuxExecutionUniverseContentPF2G1"
              ]
            },
            {
              "name": "regions",
              "no": 3,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/PpiAuxExecutionUniverseContentPF3G1"
              ]
            },
            {
              "name": "region_compatibility",
              "no": 4,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/PpiAuxExecutionUniverseContentPF4G1"
              ]
            },
            {
              "name": "runtime_profiles",
              "no": 5,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/PpiAuxExecutionUniverseContentPF5G1"
              ]
            }
          ]
        }
      ],
      "name": "ExecutionUniverseContent",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "confidentiality_labels",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/PpiAuxFlowCatalogContentPF1G1"
              ]
            },
            {
              "name": "taints",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/PpiAuxFlowCatalogContentPF2G1"
              ]
            },
            {
              "name": "effect_sinks",
              "no": 3,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/PpiAuxFlowCatalogContentPF3G1"
              ]
            }
          ]
        }
      ],
      "name": "FlowCatalogContent",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "operation",
              "no": 1,
              "type": [
                "ref",
                "agentflow.effect.signature@1/EffectOperationRef"
              ]
            },
            {
              "name": "decision_capable",
              "no": 2,
              "type": [
                "primitive",
                "bool"
              ]
            }
          ]
        }
      ],
      "name": "FlowEffectSinkContent",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "header",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/RelationContentHeader"
              ]
            },
            {
              "name": "canonical_content",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/Bytes"
              ]
            }
          ]
        }
      ],
      "name": "FlowExecutionRelationContent",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "output_value",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ValueSlotId"
              ]
            },
            {
              "name": "effect_occurrences",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/Ordinal64"
                ]
              ]
            }
          ]
        }
      ],
      "name": "FlowSinkBinding",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "input_expression",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            },
            {
              "name": "resource_slot",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ResourceSlotId"
              ]
            },
            {
              "name": "unknown_external",
              "no": 3,
              "type": [
                "primitive",
                "unit"
              ]
            }
          ]
        }
      ],
      "name": "FlowSourceBinding",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "listed_rules",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/RuleContentIdentity"
                ]
              ]
            }
          ]
        }
      ],
      "name": "IncompleteAttribution",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "reference",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/TypedContentRef"
              ]
            },
            {
              "name": "canonical_content",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/Bytes"
              ]
            }
          ]
        }
      ],
      "name": "IrAsset",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "alias",
        [
          "ref",
          "agentflow.portable.proof.inputs@1/Text"
        ]
      ],
      "name": "LogicalPath",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "alias",
        [
          "ref",
          "agentflow.portable.proof.inputs@1/Text"
        ]
      ],
      "name": "LowerHex64",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "not_formalized",
              "no": 1
            },
            {
              "name": "future_dependency",
              "no": 2
            }
          ]
        }
      ],
      "name": "ModelMissingReason",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "specification_id",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/SpecId"
              ]
            },
            {
              "name": "named_types",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.core@1/TypeSchemaRef"
                ]
              ]
            }
          ]
        }
      ],
      "name": "ModuleContent",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "alias",
        [
          "ref",
          "agentflow.portable.proof.inputs@1/U64"
        ]
      ],
      "name": "Ordinal64",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "algorithm",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/PpiAuxPlainChecksumPF1"
              ]
            },
            {
              "name": "hex",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/LowerHex64"
              ]
            }
          ]
        }
      ],
      "name": "PlainChecksum",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "compiled_modules",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/PpiAuxPortableContextContentPF1G1"
              ]
            },
            {
              "name": "type_schemas",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/PpiAuxPortableContextContentPF2G1"
              ]
            },
            {
              "name": "effect_catalogs",
              "no": 3,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/PpiAuxPortableContextContentPF3G1"
              ]
            },
            {
              "name": "resource_types",
              "no": 4,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/ResourceUniverseContent"
              ]
            },
            {
              "name": "flow_catalogs",
              "no": 5,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/FlowCatalogContent"
              ]
            },
            {
              "name": "usage_meters",
              "no": 6,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/PpiAuxPortableContextContentPF6G1"
              ]
            },
            {
              "name": "recovery_operations",
              "no": 7,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/RecoveryUniverseContent"
              ]
            },
            {
              "name": "execution_universes",
              "no": 8,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/ExecutionUniverseContent"
              ]
            },
            {
              "name": "relation_snapshots",
              "no": 9,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/RelationSnapshotSlots"
              ]
            }
          ]
        }
      ],
      "name": "PortableContextContent",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "context_id",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/ContextId"
              ]
            },
            {
              "name": "resolver_id",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/ResolverId"
              ]
            }
          ]
        }
      ],
      "name": "PortableProofContextManifest",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "schema_version",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/SchemaVersion"
              ]
            },
            {
              "name": "root_assertion_id",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/AssertionId"
              ]
            },
            {
              "name": "schema_assets",
              "no": 3,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/SchemaAsset"
                ]
              ]
            },
            {
              "name": "ir_assets",
              "no": 4,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/IrAsset"
                ]
              ]
            },
            {
              "name": "contract_assets",
              "no": 5,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/ContractAsset"
                ]
              ]
            },
            {
              "name": "structural_types",
              "no": 6,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/StructuralTypeNode"
                ]
              ]
            },
            {
              "name": "proof_contexts",
              "no": 7,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/ProofContextEntry"
                ]
              ]
            },
            {
              "name": "resolver_dependency_manifests",
              "no": 8,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/ResolverEntry"
                ]
              ]
            },
            {
              "name": "compiler_semantics_manifests",
              "no": 9,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/CompilerManifestEntry"
                ]
              ]
            },
            {
              "name": "root_export_assertions",
              "no": 10,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/AssertionEntry"
                ]
              ]
            }
          ]
        }
      ],
      "name": "PortableProofInputBundle",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "caller_ir",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/TypedContentRef"
              ]
            },
            {
              "name": "context_id",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/ContextId"
              ]
            },
            {
              "name": "components",
              "no": 3,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/TypedContentRef"
                ]
              ]
            },
            {
              "name": "subflow_targets",
              "no": 4,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/ResolverTarget"
                ]
              ]
            },
            {
              "name": "source_associations",
              "no": 5,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/SourceAssociation"
                ]
              ]
            }
          ]
        }
      ],
      "name": "PortableResolverManifest",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "exact_semantic_ir_digest",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/DigestRefValue"
              ]
            },
            {
              "name": "exact_exported_component_contract_digest",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/DigestRefValue"
              ]
            },
            {
              "name": "compiler_semantics_id",
              "no": 3,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/CompilerManifestId"
              ]
            },
            {
              "name": "proof_context_manifest",
              "no": 4,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/PortableProofContextManifest"
              ]
            },
            {
              "name": "input_mappings",
              "no": 5,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/RootInputPortBinding"
                ]
              ]
            },
            {
              "name": "output_mappings",
              "no": 6,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/RootOutputPortBinding"
                ]
              ]
            },
            {
              "name": "resource_mappings",
              "no": 7,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/RootResourcePortBinding"
                ]
              ]
            },
            {
              "name": "effect_mappings",
              "no": 8,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/RootEffectCoordinateBinding"
                ]
              ]
            },
            {
              "name": "flow_mappings",
              "no": 9,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/RootFlowCoordinateBinding"
                ]
              ]
            },
            {
              "name": "recovery_mappings",
              "no": 10,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/RootRecoveryCoordinateBinding"
                ]
              ]
            },
            {
              "name": "execution_mappings",
              "no": 11,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/RootExecutionCoordinateBinding"
                ]
              ]
            },
            {
              "name": "subflow_dependencies",
              "no": 12,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/SubflowProofDependency"
                ]
              ]
            }
          ]
        }
      ],
      "name": "PortableRootExportAssertion",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "complete",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/ExecutionCapabilityContent"
                ]
              ]
            },
            {
              "name": "unavailable",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/UnavailableReason"
              ]
            }
          ]
        }
      ],
      "name": "PpiAuxExecutionUniverseContentPF1G1",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "complete",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.execution.requirements@1/TrustDomainRef"
                ]
              ]
            },
            {
              "name": "unavailable",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/UnavailableReason"
              ]
            }
          ]
        }
      ],
      "name": "PpiAuxExecutionUniverseContentPF2G1",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "complete",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.execution.requirements@1/RegionRef"
                ]
              ]
            },
            {
              "name": "unavailable",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/UnavailableReason"
              ]
            }
          ]
        }
      ],
      "name": "PpiAuxExecutionUniverseContentPF3G1",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "complete",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/RegionCompatibility"
                ]
              ]
            },
            {
              "name": "unavailable",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/UnavailableReason"
              ]
            }
          ]
        }
      ],
      "name": "PpiAuxExecutionUniverseContentPF4G1",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "complete",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/ExecutionRuntimeProfileContent"
                ]
              ]
            },
            {
              "name": "unavailable",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/UnavailableReason"
              ]
            }
          ]
        }
      ],
      "name": "PpiAuxExecutionUniverseContentPF5G1",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "complete",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.information.flow@1/ConfidentialityLabelRef"
                ]
              ]
            },
            {
              "name": "unavailable",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/UnavailableReason"
              ]
            }
          ]
        }
      ],
      "name": "PpiAuxFlowCatalogContentPF1G1",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "complete",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.information.flow@1/TaintRef"
                ]
              ]
            },
            {
              "name": "unavailable",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/UnavailableReason"
              ]
            }
          ]
        }
      ],
      "name": "PpiAuxFlowCatalogContentPF2G1",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "complete",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/FlowEffectSinkContent"
                ]
              ]
            },
            {
              "name": "unavailable",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/UnavailableReason"
              ]
            }
          ]
        }
      ],
      "name": "PpiAuxFlowCatalogContentPF3G1",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "sha256",
              "no": 1
            }
          ]
        }
      ],
      "name": "PpiAuxPlainChecksumPF1",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "complete",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/ModuleContent"
                ]
              ]
            },
            {
              "name": "unavailable",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/UnavailableReason"
              ]
            }
          ]
        }
      ],
      "name": "PpiAuxPortableContextContentPF1G1",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "complete",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.core@1/TypeSchemaRef"
                ]
              ]
            },
            {
              "name": "unavailable",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/UnavailableReason"
              ]
            }
          ]
        }
      ],
      "name": "PpiAuxPortableContextContentPF2G1",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "complete",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/EffectCatalogContent"
                ]
              ]
            },
            {
              "name": "unavailable",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/UnavailableReason"
              ]
            }
          ]
        }
      ],
      "name": "PpiAuxPortableContextContentPF3G1",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "complete",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/UsageMeterContent"
                ]
              ]
            },
            {
              "name": "unavailable",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/UnavailableReason"
              ]
            }
          ]
        }
      ],
      "name": "PpiAuxPortableContextContentPF6G1",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "complete",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/EffectCatalogContent"
                ]
              ]
            },
            {
              "name": "unavailable",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/UnavailableReason"
              ]
            }
          ]
        }
      ],
      "name": "PpiAuxRecoveryUniverseContentPF1G1",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "complete",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/ReconciliationOperationRef"
                ]
              ]
            },
            {
              "name": "unavailable",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/UnavailableReason"
              ]
            }
          ]
        }
      ],
      "name": "PpiAuxRecoveryUniverseContentPF2G1",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "complete",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.core@1/TypeSchemaRef"
                ]
              ]
            },
            {
              "name": "unavailable",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/UnavailableReason"
              ]
            }
          ]
        }
      ],
      "name": "PpiAuxResourceUniverseContentPF1G1",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "complete",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/ResourceTypeContent"
                ]
              ]
            },
            {
              "name": "unavailable",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/UnavailableReason"
              ]
            }
          ]
        }
      ],
      "name": "PpiAuxResourceUniverseContentPF2G1",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "id",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/ContextId"
              ]
            },
            {
              "name": "content",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/PortableContextContent"
              ]
            }
          ]
        }
      ],
      "name": "ProofContextEntry",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "catalog",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/SpecId"
              ]
            },
            {
              "name": "name",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/Text"
              ]
            }
          ]
        }
      ],
      "name": "ReconciliationOperationRef",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "caller_semantic_ir_digest",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/DigestRefValue"
              ]
            },
            {
              "name": "source_node",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ComputationId"
              ]
            },
            {
              "name": "source_contract_digest",
              "no": 3,
              "type": [
                "ref",
                "agentflow.semantic.core@1/DigestRefValue"
              ]
            },
            {
              "name": "forward_entry_ordinal",
              "no": 4,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/Ordinal64"
              ]
            },
            {
              "name": "action",
              "no": 5,
              "type": [
                "ref",
                "agentflow.component.contract@1/RecoveryActionIdentity"
              ]
            }
          ]
        }
      ],
      "name": "RecoveryActionSourceIdentity",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "effect_catalogs",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/PpiAuxRecoveryUniverseContentPF1G1"
              ]
            },
            {
              "name": "reconciliation_operations",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/PpiAuxRecoveryUniverseContentPF2G1"
              ]
            }
          ]
        }
      ],
      "name": "RecoveryUniverseContent",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "header",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/RelationContentHeader"
              ]
            },
            {
              "name": "canonical_content",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/Bytes"
              ]
            }
          ]
        }
      ],
      "name": "RecoveryUsageRelationContent",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "from",
              "no": 1,
              "type": [
                "ref",
                "agentflow.execution.requirements@1/RegionRef"
              ]
            },
            {
              "name": "to",
              "no": 2,
              "type": [
                "ref",
                "agentflow.execution.requirements@1/RegionRef"
              ]
            }
          ]
        }
      ],
      "name": "RegionCompatibility",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "content_sha256",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/PlainChecksum"
              ]
            },
            {
              "name": "content",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/Bytes"
              ]
            }
          ]
        }
      ],
      "name": "RegistryContent",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "specifications",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/RegistryContent"
              ]
            },
            {
              "name": "rules",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/RegistryContent"
              ]
            }
          ]
        }
      ],
      "name": "RegistryContentSet",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "schema",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/TypeSchemaRef"
              ]
            },
            {
              "name": "revision",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/Text"
              ]
            },
            {
              "name": "canonical_content_checksum",
              "no": 3,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/PlainChecksum"
              ]
            }
          ]
        }
      ],
      "name": "RelationContentHeader",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "effect_resource",
              "no": 1,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/EffectResourceRelationContent"
                ]
              ]
            },
            {
              "name": "recovery_usage",
              "no": 2,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/RecoveryUsageRelationContent"
                ]
              ]
            },
            {
              "name": "flow_execution",
              "no": 3,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/FlowExecutionRelationContent"
                ]
              ]
            }
          ]
        }
      ],
      "name": "RelationSnapshotSlots",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "target_semantic_ir_digest",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/DigestRefValue"
              ]
            },
            {
              "name": "exported_contract_digest",
              "no": 2,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.semantic.core@1/DigestRefValue"
                ]
              ]
            }
          ]
        }
      ],
      "name": "ResolvedSubflowAssociation",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "id",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/ResolverId"
              ]
            },
            {
              "name": "content",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/PortableResolverManifest"
              ]
            }
          ]
        }
      ],
      "name": "ResolverEntry",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "alias",
        [
          "ref",
          "agentflow.portable.proof.inputs@1/U32"
        ]
      ],
      "name": "ResolverId",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "target_semantic_ir_digest",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/DigestRefValue"
              ]
            },
            {
              "name": "target_ir_content",
              "no": 2,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/TypedContentRef"
                ]
              ]
            },
            {
              "name": "exported_contract",
              "no": 3,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/TypedContentRef"
                ]
              ]
            },
            {
              "name": "direct_dependencies",
              "no": 4,
              "type": [
                "option",
                [
                  "list",
                  [
                    "ref",
                    "agentflow.semantic.core@1/DigestRefValue"
                  ]
                ]
              ]
            },
            {
              "name": "transitive_dependencies",
              "no": 5,
              "type": [
                "option",
                [
                  "list",
                  [
                    "ref",
                    "agentflow.portable.proof.inputs@1/SubflowTargetIdentity"
                  ]
                ]
              ]
            }
          ]
        }
      ],
      "name": "ResolverTarget",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "exported_contract_digest",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/DigestRefValue"
              ]
            },
            {
              "name": "port_no",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/U32"
              ]
            }
          ]
        }
      ],
      "name": "ResourceRequirementPortIdentity",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "schema",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/TypeSchemaRef"
              ]
            },
            {
              "name": "recognized_attributes",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/Text"
                ]
              ]
            }
          ]
        }
      ],
      "name": "ResourceTypeContent",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "schemas",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/PpiAuxResourceUniverseContentPF1G1"
              ]
            },
            {
              "name": "descriptors",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/PpiAuxResourceUniverseContentPF2G1"
              ]
            }
          ]
        }
      ],
      "name": "ResourceUniverseContent",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "defined",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/SemanticRoleDefinition"
              ]
            },
            {
              "name": "model_missing",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/ModelMissingReason"
              ]
            }
          ]
        }
      ],
      "name": "RoleDeclaration",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "source",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/RootEffectOccurrenceIdentity"
              ]
            },
            {
              "name": "root_resource_slot",
              "no": 2,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.semantic.core@1/ResourceSlotId"
                ]
              ]
            },
            {
              "name": "export_atom",
              "no": 3,
              "type": [
                "ref",
                "agentflow.effect.signature@1/EffectAtom"
              ]
            },
            {
              "name": "export_guard",
              "no": 4,
              "type": [
                "ref",
                "agentflow.effect.signature@1/EffectGuard"
              ]
            }
          ]
        }
      ],
      "name": "RootEffectCoordinateBinding",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "caller_semantic_ir_digest",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/DigestRefValue"
              ]
            },
            {
              "name": "source_node",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ComputationId"
              ]
            },
            {
              "name": "source_contract_digest",
              "no": 3,
              "type": [
                "ref",
                "agentflow.semantic.core@1/DigestRefValue"
              ]
            },
            {
              "name": "source_entry_ordinal",
              "no": 4,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/Ordinal64"
              ]
            },
            {
              "name": "atom",
              "no": 5,
              "type": [
                "ref",
                "agentflow.effect.signature@1/EffectAtom"
              ]
            },
            {
              "name": "guard",
              "no": 6,
              "type": [
                "ref",
                "agentflow.effect.signature@1/EffectGuard"
              ]
            }
          ]
        }
      ],
      "name": "RootEffectOccurrenceIdentity",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "source_node",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ComputationId"
              ]
            },
            {
              "name": "source_contract_digest",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/DigestRefValue"
              ]
            },
            {
              "name": "exact_execution_requirements",
              "no": 3,
              "type": [
                "ref",
                "agentflow.execution.requirements@1/ExecutionRequirements"
              ]
            }
          ]
        }
      ],
      "name": "RootExecutionCoordinateBinding",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "source_node",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ComputationId"
              ]
            },
            {
              "name": "source_contract_digest",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/DigestRefValue"
              ]
            },
            {
              "name": "flow_mapping_ordinal",
              "no": 3,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/U64"
              ]
            },
            {
              "name": "source",
              "no": 4,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.information.flow@1/FlowSourceRef"
                ]
              ]
            },
            {
              "name": "sink",
              "no": 5,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.information.flow@1/FlowConstraint"
                ]
              ]
            },
            {
              "name": "source_binding",
              "no": 6,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/FlowSourceBinding"
                ]
              ]
            },
            {
              "name": "sink_binding",
              "no": 7,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/FlowSinkBinding"
                ]
              ]
            },
            {
              "name": "export_source",
              "no": 8,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.information.flow@1/FlowSourceRef"
                ]
              ]
            },
            {
              "name": "export_sink",
              "no": 9,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.information.flow@1/FlowConstraint"
                ]
              ]
            },
            {
              "name": "source_effect_partition",
              "no": 10,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.component.contract@1/EffectFlowPartition"
                ]
              ]
            },
            {
              "name": "export_effect_partition",
              "no": 11,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.component.contract@1/EffectFlowPartition"
                ]
              ]
            }
          ]
        }
      ],
      "name": "RootFlowCoordinateBinding",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "export_input_port",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/ComponentInputPortIdentity"
              ]
            },
            {
              "name": "run_parameter_id",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/RunParameterId"
              ]
            },
            {
              "name": "root_value_slot",
              "no": 3,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ValueSlotId"
              ]
            },
            {
              "name": "root_type",
              "no": 4,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/StructuralTypeId"
              ]
            },
            {
              "name": "export_type",
              "no": 5,
              "type": [
                "ref",
                "agentflow.semantic.core@1/TypeSchemaRef"
              ]
            },
            {
              "name": "use_sites",
              "no": 6,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/RootInputUseSite"
                ]
              ]
            }
          ]
        }
      ],
      "name": "RootInputPortBinding",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "computation",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ComputationId"
              ]
            },
            {
              "name": "source_contract_digest",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/DigestRefValue"
              ]
            },
            {
              "name": "source_component_port",
              "no": 3,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/U32"
              ]
            },
            {
              "name": "input_expression",
              "no": 4,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            },
            {
              "name": "exact_type",
              "no": 5,
              "type": [
                "ref",
                "agentflow.semantic.core@1/TypeSchemaRef"
              ]
            },
            {
              "name": "dependency_slots",
              "no": 6,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.core@1/ValueSlotId"
                ]
              ]
            }
          ]
        }
      ],
      "name": "RootInputUseSite",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "export_output_port",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/ComponentOutputPortIdentity"
              ]
            },
            {
              "name": "workflow_output_name",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/Text"
              ]
            },
            {
              "name": "workflow_output_expression",
              "no": 3,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            },
            {
              "name": "root_result_type",
              "no": 4,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/StructuralTypeId"
              ]
            },
            {
              "name": "producer_evidence",
              "no": 5,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/RootOutputProducerEvidence"
              ]
            }
          ]
        }
      ],
      "name": "RootOutputPortBinding",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "exact_result_type",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/TypeSchemaRef"
              ]
            },
            {
              "name": "dependency_slots",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.core@1/ValueSlotId"
                ]
              ]
            }
          ]
        }
      ],
      "name": "RootOutputProducerEvidence",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "source",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/RecoveryActionSourceIdentity"
              ]
            },
            {
              "name": "export_action_identity",
              "no": 2,
              "type": [
                "ref",
                "agentflow.component.contract@1/RecoveryActionIdentity"
              ]
            },
            {
              "name": "exact_usage_binding",
              "no": 3,
              "type": [
                "ref",
                "agentflow.component.contract@1/RecoveryUsageBinding"
              ]
            }
          ]
        }
      ],
      "name": "RootRecoveryCoordinateBinding",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "export_resource_port",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/ResourceRequirementPortIdentity"
              ]
            },
            {
              "name": "root_resource_slot",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ResourceSlotId"
              ]
            },
            {
              "name": "root_resource_type",
              "no": 3,
              "type": [
                "ref",
                "agentflow.semantic.core@1/TypeSchemaRef"
              ]
            },
            {
              "name": "export_resource_type",
              "no": 4,
              "type": [
                "ref",
                "agentflow.resource.requirement@1/ResourceTypeRef"
              ]
            },
            {
              "name": "source_coordinates",
              "no": 5,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/RootResourceSourceCoordinate"
                ]
              ]
            }
          ]
        }
      ],
      "name": "RootResourcePortBinding",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "source_node",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ComputationId"
              ]
            },
            {
              "name": "source_contract_digest",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/DigestRefValue"
              ]
            },
            {
              "name": "component_resource_port",
              "no": 3,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/U32"
              ]
            },
            {
              "name": "resource_slot",
              "no": 4,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ResourceSlotId"
              ]
            },
            {
              "name": "alias_component_ports",
              "no": 5,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/U32"
                ]
              ]
            }
          ]
        }
      ],
      "name": "RootResourceSourceCoordinate",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "rule_id",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/Text"
              ]
            },
            {
              "name": "normative_source",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/LogicalPath"
              ]
            },
            {
              "name": "source_anchor",
              "no": 3,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/Text"
              ]
            },
            {
              "name": "short_statement",
              "no": 4,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/Text"
              ]
            }
          ]
        }
      ],
      "name": "RuleContentIdentity",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "dependent",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/Text"
              ]
            },
            {
              "name": "prerequisite",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/Text"
              ]
            }
          ]
        }
      ],
      "name": "RuleDependencyEdge",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "reference",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/TypeSchemaRef"
              ]
            },
            {
              "name": "canonical_content",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/Bytes"
              ]
            }
          ]
        }
      ],
      "name": "SchemaAsset",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "portable_proof_inputs_v1",
              "no": 1
            }
          ]
        }
      ],
      "name": "SchemaVersion",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "static_analysis",
              "no": 1
            },
            {
              "name": "contract_validation",
              "no": 2
            },
            {
              "name": "semantic_ir_validation",
              "no": 3
            },
            {
              "name": "dataflow",
              "no": 4
            },
            {
              "name": "leaf_resolution",
              "no": 5
            },
            {
              "name": "control_algebra",
              "no": 6
            },
            {
              "name": "root_recovery",
              "no": 7
            },
            {
              "name": "root_export",
              "no": 8
            },
            {
              "name": "plan_lowering",
              "no": 9
            },
            {
              "name": "runtime_semantics_profile",
              "no": 10
            }
          ]
        }
      ],
      "name": "SemanticRole",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "analysis_semantics_version",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/Text"
              ]
            },
            {
              "name": "normative_specifications",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/SpecId"
                ]
              ]
            },
            {
              "name": "required_schema_pins",
              "no": 3,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.core@1/TypeSchemaRef"
                ]
              ]
            },
            {
              "name": "depends_on",
              "no": 4,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/SemanticRole"
                ]
              ]
            }
          ]
        }
      ],
      "name": "SemanticRoleDefinition",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "role",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/SemanticRole"
              ]
            },
            {
              "name": "declaration",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/RoleDeclaration"
              ]
            }
          ]
        }
      ],
      "name": "SemanticRoleEntry",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "caller_ir",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/TypedContentRef"
              ]
            },
            {
              "name": "source_node",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ComputationId"
              ]
            },
            {
              "name": "target",
              "no": 3,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/SourceAssociationTarget"
              ]
            }
          ]
        }
      ],
      "name": "SourceAssociation",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "invoke",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/TypedContentRef"
              ]
            },
            {
              "name": "call_subflow",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/ResolvedSubflowAssociation"
              ]
            },
            {
              "name": "await_signal",
              "no": 3,
              "type": [
                "ref",
                "agentflow.semantic.core@1/TypeSchemaRef"
              ]
            }
          ]
        }
      ],
      "name": "SourceAssociationTarget",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "specification_id",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/SpecId"
              ]
            },
            {
              "name": "logical_path",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/LogicalPath"
              ]
            },
            {
              "name": "content_sha256",
              "no": 3,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/PlainChecksum"
              ]
            },
            {
              "name": "content",
              "no": 4,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/Bytes"
              ]
            }
          ]
        }
      ],
      "name": "SourceContentIdentity",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "alias",
        [
          "ref",
          "agentflow.portable.proof.inputs@1/Text"
        ]
      ],
      "name": "SpecId",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "analysis_semantics_version",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/Text"
              ]
            },
            {
              "name": "normative_specification_ids",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/SpecId"
                ]
              ]
            },
            {
              "name": "applied_rules",
              "no": 3,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/RuleContentIdentity"
                ]
              ]
            }
          ]
        }
      ],
      "name": "StaticIdentityProjection",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "primitive",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/SemanticPrimitiveType"
              ]
            },
            {
              "name": "schema",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/TypeSchemaRef"
              ]
            },
            {
              "name": "list",
              "no": 3,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/StructuralTypeId"
              ]
            },
            {
              "name": "map_string",
              "no": 4,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/StructuralTypeId"
              ]
            },
            {
              "name": "option",
              "no": 5,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/StructuralTypeId"
              ]
            }
          ]
        }
      ],
      "name": "StructuralTypeBody",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "alias",
        [
          "ref",
          "agentflow.portable.proof.inputs@1/U32"
        ]
      ],
      "name": "StructuralTypeId",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "id",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/StructuralTypeId"
              ]
            },
            {
              "name": "body",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/StructuralTypeBody"
              ]
            }
          ]
        }
      ],
      "name": "StructuralTypeNode",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "call_site",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ComputationId"
              ]
            },
            {
              "name": "target_semantic_ir_digest",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/DigestRefValue"
              ]
            },
            {
              "name": "target_exported_contract_digest",
              "no": 3,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.semantic.core@1/DigestRefValue"
                ]
              ]
            },
            {
              "name": "target_root_assertion_id",
              "no": 4,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/AssertionId"
                ]
              ]
            },
            {
              "name": "transitive_targets",
              "no": 5,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.portable.proof.inputs@1/SubflowTargetIdentity"
                ]
              ]
            }
          ]
        }
      ],
      "name": "SubflowProofDependency",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "semantic_ir_digest",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/DigestRefValue"
              ]
            },
            {
              "name": "exported_contract_digest",
              "no": 2,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.semantic.core@1/DigestRefValue"
                ]
              ]
            }
          ]
        }
      ],
      "name": "SubflowTargetIdentity",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "string"
        ]
      ],
      "name": "Text",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "alias",
        [
          "ref",
          "agentflow.portable.proof.inputs@1/Text"
        ]
      ],
      "name": "TypeName",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "digest",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/DigestRefValue"
              ]
            },
            {
              "name": "type_schema",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/TypeSchemaRef"
              ]
            }
          ]
        }
      ],
      "name": "TypedContentRef",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "uint32"
        ]
      ],
      "name": "U32",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "uint64"
        ]
      ],
      "name": "U64",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "not_supplied",
              "no": 1
            },
            {
              "name": "content_not_retained",
              "no": 2
            },
            {
              "name": "unsupported_projection",
              "no": 3
            }
          ]
        }
      ],
      "name": "UnavailableReason",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "additive",
              "no": 1
            },
            {
              "name": "unsupported",
              "no": 2
            }
          ]
        }
      ],
      "name": "UsageAggregation",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "meter",
              "no": 1,
              "type": [
                "ref",
                "agentflow.usage.signature@1/UsageMeterRef"
              ]
            },
            {
              "name": "unit",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/UsageUnitRef"
              ]
            },
            {
              "name": "aggregation",
              "no": 3,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/UsageAggregation"
              ]
            }
          ]
        }
      ],
      "name": "UsageMeterContent",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "catalog",
              "no": 1,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/SpecId"
              ]
            },
            {
              "name": "name",
              "no": 2,
              "type": [
                "ref",
                "agentflow.portable.proof.inputs@1/Text"
              ]
            }
          ]
        }
      ],
      "name": "UsageUnitRef",
      "schema": "agentflow.portable.proof.inputs@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "none",
              "no": 1,
              "type": [
                "primitive",
                "unit"
              ]
            },
            {
              "name": "required",
              "no": 2,
              "type": [
                "ref",
                "agentflow.effect.signature@1/EffectOperationRef"
              ]
            }
          ]
        }
      ],
      "name": "CompensationRequirement",
      "schema": "agentflow.recovery.contract@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "effect",
              "no": 1,
              "type": [
                "ref",
                "agentflow.effect.signature@1/EffectAtom"
              ]
            },
            {
              "name": "idempotency",
              "no": 2,
              "type": [
                "ref",
                "agentflow.recovery.contract@1/IdempotencySemantics"
              ]
            },
            {
              "name": "key_requirement",
              "no": 3,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.recovery.contract@1/KeyStabilityRequirement"
                ]
              ]
            },
            {
              "name": "reconciliation",
              "no": 4,
              "type": [
                "ref",
                "agentflow.recovery.contract@1/ReconciliationRequirement"
              ]
            },
            {
              "name": "compensation",
              "no": 5,
              "type": [
                "ref",
                "agentflow.recovery.contract@1/CompensationRequirement"
              ]
            },
            {
              "name": "unknown_effect_handling",
              "no": 6,
              "type": [
                "ref",
                "agentflow.recovery.contract@1/UnknownEffectHandling"
              ]
            }
          ]
        }
      ],
      "name": "EffectRecoveryClause",
      "schema": "agentflow.recovery.contract@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "natural",
              "no": 1
            },
            {
              "name": "keyed",
              "no": 2
            },
            {
              "name": "none",
              "no": 3
            },
            {
              "name": "reconciled",
              "no": 4
            }
          ]
        }
      ],
      "name": "IdempotencySemantics",
      "schema": "agentflow.recovery.contract@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "stable_across_attempts",
              "no": 1,
              "type": [
                "primitive",
                "bool"
              ]
            },
            {
              "name": "distinct_across_invocations",
              "no": 2,
              "type": [
                "primitive",
                "bool"
              ]
            },
            {
              "name": "bound_to_operation",
              "no": 3,
              "type": [
                "primitive",
                "bool"
              ]
            },
            {
              "name": "provider_honors",
              "no": 4,
              "type": [
                "primitive",
                "bool"
              ]
            }
          ]
        }
      ],
      "name": "KeyStabilityRequirement",
      "schema": "agentflow.recovery.contract@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "none",
              "no": 1
            },
            {
              "name": "required",
              "no": 2
            }
          ]
        }
      ],
      "name": "ReconciliationRequirement",
      "schema": "agentflow.recovery.contract@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "replay_semantics",
              "no": 1,
              "type": [
                "ref",
                "agentflow.recovery.contract@1/ReplaySemantics"
              ]
            },
            {
              "name": "effect_recovery_clauses",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.recovery.contract@1/EffectRecoveryClause"
                ]
              ]
            }
          ]
        }
      ],
      "name": "RecoveryContract",
      "schema": "agentflow.recovery.contract@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "recompute",
              "no": 1
            },
            {
              "name": "recorded",
              "no": 2
            },
            {
              "name": "reconcile",
              "no": 3
            },
            {
              "name": "forbidden",
              "no": 4
            }
          ]
        }
      ],
      "name": "ReplaySemantics",
      "schema": "agentflow.recovery.contract@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "block",
              "no": 1
            },
            {
              "name": "reconciliation_required",
              "no": 2
            }
          ]
        }
      ],
      "name": "UnknownEffectHandling",
      "schema": "agentflow.recovery.contract@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "rules",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.recovery.usage.coverage@1/RecoveryUsageCoverageRule"
                ]
              ]
            }
          ]
        }
      ],
      "name": "RecoveryUsageCoverageCatalog",
      "schema": "agentflow.recovery.usage.coverage@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "action",
              "no": 1,
              "type": [
                "ref",
                "agentflow.component.contract@1/RecoveryActionIdentity"
              ]
            },
            {
              "name": "complete_meters",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.usage.signature@1/UsageMeterRef"
                ]
              ]
            }
          ]
        }
      ],
      "name": "RecoveryUsageCoverageRule",
      "schema": "agentflow.recovery.usage.coverage@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "key",
              "no": 1,
              "type": [
                "primitive",
                "string"
              ]
            },
            {
              "name": "value",
              "no": 2,
              "type": [
                "primitive",
                "string"
              ]
            }
          ]
        }
      ],
      "name": "ResourceAttributeConstraint",
      "schema": "agentflow.resource.requirement@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "port_no",
              "no": 1,
              "type": [
                "primitive",
                "uint32"
              ]
            },
            {
              "name": "resource_type",
              "no": 2,
              "type": [
                "ref",
                "agentflow.resource.requirement@1/ResourceTypeRef"
              ]
            },
            {
              "name": "required_attributes",
              "no": 3,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.resource.requirement@1/ResourceAttributeConstraint"
                ]
              ]
            }
          ]
        }
      ],
      "name": "ResourceRequirement",
      "schema": "agentflow.resource.requirement@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "requirements",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.resource.requirement@1/ResourceRequirement"
                ]
              ]
            }
          ]
        }
      ],
      "name": "ResourceRequirementRow",
      "schema": "agentflow.resource.requirement@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "schema",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/TypeSchemaRef"
              ]
            }
          ]
        }
      ],
      "name": "ResourceTypeRef",
      "schema": "agentflow.resource.requirement@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "left",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            },
            {
              "name": "right",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            }
          ]
        }
      ],
      "name": "AndBody",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "algorithm",
              "no": 1,
              "type": [
                "primitive",
                "string"
              ]
            },
            {
              "name": "domain",
              "no": 2,
              "type": [
                "primitive",
                "string"
              ]
            },
            {
              "name": "hex",
              "no": 3,
              "type": [
                "primitive",
                "string"
              ]
            }
          ]
        }
      ],
      "name": "DigestRefValue",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "schema",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/TypeSchemaRef"
              ]
            },
            {
              "name": "value_no",
              "no": 2,
              "type": [
                "primitive",
                "uint32"
              ]
            }
          ]
        }
      ],
      "name": "EnumValueBody",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "left",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            },
            {
              "name": "right",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            }
          ]
        }
      ],
      "name": "EqualsBody",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "constant",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/PrimitiveConstant"
              ]
            },
            {
              "name": "value_slot",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ValueSlotId"
              ]
            },
            {
              "name": "record_construct",
              "no": 3,
              "type": [
                "ref",
                "agentflow.semantic.core@1/RecordConstructBody"
              ]
            },
            {
              "name": "variant_construct",
              "no": 4,
              "type": [
                "ref",
                "agentflow.semantic.core@1/VariantConstructBody"
              ]
            },
            {
              "name": "enum_value",
              "no": 5,
              "type": [
                "ref",
                "agentflow.semantic.core@1/EnumValueBody"
              ]
            },
            {
              "name": "list_construct",
              "no": 6,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ListConstructBody"
              ]
            },
            {
              "name": "map_construct",
              "no": 7,
              "type": [
                "ref",
                "agentflow.semantic.core@1/MapConstructBody"
              ]
            },
            {
              "name": "option_none",
              "no": 8,
              "type": [
                "ref",
                "agentflow.semantic.core@1/OptionNoneBody"
              ]
            },
            {
              "name": "option_some",
              "no": 9,
              "type": [
                "ref",
                "agentflow.semantic.core@1/OptionSomeBody"
              ]
            },
            {
              "name": "field_get",
              "no": 10,
              "type": [
                "ref",
                "agentflow.semantic.core@1/FieldGetBody"
              ]
            },
            {
              "name": "equals",
              "no": 11,
              "type": [
                "ref",
                "agentflow.semantic.core@1/EqualsBody"
              ]
            },
            {
              "name": "not",
              "no": 12,
              "type": [
                "ref",
                "agentflow.semantic.core@1/NotBody"
              ]
            },
            {
              "name": "and",
              "no": 13,
              "type": [
                "ref",
                "agentflow.semantic.core@1/AndBody"
              ]
            },
            {
              "name": "or",
              "no": 14,
              "type": [
                "ref",
                "agentflow.semantic.core@1/OrBody"
              ]
            },
            {
              "name": "if_then_else",
              "no": 15,
              "type": [
                "ref",
                "agentflow.semantic.core@1/IfThenElseBody"
              ]
            }
          ]
        }
      ],
      "name": "ExpressionBody",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "id",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            },
            {
              "name": "result_type",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/SemanticTypeId"
              ]
            },
            {
              "name": "body",
              "no": 3,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionBody"
              ]
            }
          ]
        }
      ],
      "name": "ExpressionDef",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "uint32"
        ]
      ],
      "name": "ExpressionId",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "record",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            },
            {
              "name": "field_no",
              "no": 2,
              "type": [
                "primitive",
                "uint32"
              ]
            }
          ]
        }
      ],
      "name": "FieldGetBody",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "condition",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            },
            {
              "name": "then_expr",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            },
            {
              "name": "else_expr",
              "no": 3,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            }
          ]
        }
      ],
      "name": "IfThenElseBody",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "element_type",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/SemanticTypeId"
              ]
            },
            {
              "name": "elements",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.core@1/ExpressionId"
                ]
              ]
            }
          ]
        }
      ],
      "name": "ListConstructBody",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "value_type",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/SemanticTypeId"
              ]
            },
            {
              "name": "entries",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.core@1/MapEntryExpr"
                ]
              ]
            }
          ]
        }
      ],
      "name": "MapConstructBody",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "key",
              "no": 1,
              "type": [
                "primitive",
                "string"
              ]
            },
            {
              "name": "value_expr",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            }
          ]
        }
      ],
      "name": "MapEntryExpr",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "operand",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            }
          ]
        }
      ],
      "name": "NotBody",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "inner_type",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/SemanticTypeId"
              ]
            }
          ]
        }
      ],
      "name": "OptionNoneBody",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "payload_expr",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            }
          ]
        }
      ],
      "name": "OptionSomeBody",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "left",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            },
            {
              "name": "right",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            }
          ]
        }
      ],
      "name": "OrBody",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "unit",
              "no": 1,
              "type": [
                "primitive",
                "unit"
              ]
            },
            {
              "name": "null",
              "no": 2,
              "type": [
                "primitive",
                "null"
              ]
            },
            {
              "name": "bool",
              "no": 3,
              "type": [
                "primitive",
                "bool"
              ]
            },
            {
              "name": "int64",
              "no": 4,
              "type": [
                "primitive",
                "int64"
              ]
            },
            {
              "name": "decimal",
              "no": 5,
              "type": [
                "primitive",
                "decimal"
              ]
            },
            {
              "name": "string",
              "no": 6,
              "type": [
                "primitive",
                "string"
              ]
            },
            {
              "name": "bytes",
              "no": 7,
              "type": [
                "primitive",
                "bytes"
              ]
            },
            {
              "name": "timestamp",
              "no": 8,
              "type": [
                "primitive",
                "timestamp"
              ]
            },
            {
              "name": "duration",
              "no": 9,
              "type": [
                "primitive",
                "duration"
              ]
            }
          ]
        }
      ],
      "name": "PrimitiveConstant",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "schema",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/TypeSchemaRef"
              ]
            },
            {
              "name": "fields",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.core@1/RecordFieldExpr"
                ]
              ]
            }
          ]
        }
      ],
      "name": "RecordConstructBody",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "field_no",
              "no": 1,
              "type": [
                "primitive",
                "uint32"
              ]
            },
            {
              "name": "expr_id",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            }
          ]
        }
      ],
      "name": "RecordFieldExpr",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "id",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ResourceSlotId"
              ]
            },
            {
              "name": "requirement_type",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/TypeSchemaRef"
              ]
            }
          ]
        }
      ],
      "name": "ResourceSlot",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "uint32"
        ]
      ],
      "name": "ResourceSlotId",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "unit",
              "no": 1
            },
            {
              "name": "null",
              "no": 2
            },
            {
              "name": "bool",
              "no": 3
            },
            {
              "name": "int64",
              "no": 4
            },
            {
              "name": "decimal",
              "no": 5
            },
            {
              "name": "string",
              "no": 6
            },
            {
              "name": "bytes",
              "no": 7
            },
            {
              "name": "timestamp",
              "no": 8
            },
            {
              "name": "duration",
              "no": 9
            }
          ]
        }
      ],
      "name": "SemanticPrimitiveType",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "uint32"
        ]
      ],
      "name": "SemanticTypeId",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "id",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/SemanticTypeId"
              ]
            },
            {
              "name": "body",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/SemanticTypeNodeBody"
              ]
            }
          ]
        }
      ],
      "name": "SemanticTypeNode",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "primitive",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/SemanticPrimitiveType"
              ]
            },
            {
              "name": "schema",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/TypeSchemaRef"
              ]
            },
            {
              "name": "list",
              "no": 3,
              "type": [
                "ref",
                "agentflow.semantic.core@1/SemanticTypeId"
              ]
            },
            {
              "name": "map_string",
              "no": 4,
              "type": [
                "ref",
                "agentflow.semantic.core@1/SemanticTypeId"
              ]
            },
            {
              "name": "option",
              "no": 5,
              "type": [
                "ref",
                "agentflow.semantic.core@1/SemanticTypeId"
              ]
            }
          ]
        }
      ],
      "name": "SemanticTypeNodeBody",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "specification_id",
              "no": 1,
              "type": [
                "primitive",
                "string"
              ]
            },
            {
              "name": "type_name",
              "no": 2,
              "type": [
                "primitive",
                "string"
              ]
            },
            {
              "name": "schema_digest",
              "no": 3,
              "type": [
                "ref",
                "agentflow.semantic.core@1/DigestRefValue"
              ]
            }
          ]
        }
      ],
      "name": "TypeSchemaRef",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "id",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ValueSlotId"
              ]
            },
            {
              "name": "type",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/SemanticTypeId"
              ]
            }
          ]
        }
      ],
      "name": "ValueSlot",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "uint32"
        ]
      ],
      "name": "ValueSlotId",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "schema",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/TypeSchemaRef"
              ]
            },
            {
              "name": "case_no",
              "no": 2,
              "type": [
                "primitive",
                "uint32"
              ]
            },
            {
              "name": "payload_expr",
              "no": 3,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            }
          ]
        }
      ],
      "name": "VariantConstructBody",
      "schema": "agentflow.semantic.core@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "signal_type",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/TypeSchemaRef"
              ]
            },
            {
              "name": "correlation",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            }
          ]
        }
      ],
      "name": "AwaitBody",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "uint32"
        ]
      ],
      "name": "BranchOrdinal",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "contract",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ContractRefId"
              ]
            },
            {
              "name": "inputs",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.ir@1/InputBinding"
                ]
              ]
            },
            {
              "name": "resources",
              "no": 3,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.ir@1/ResourceUseBinding"
                ]
              ]
            }
          ]
        }
      ],
      "name": "CallSubflowBody",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "child",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ComputationId"
              ]
            }
          ]
        }
      ],
      "name": "CompensationScopeBody",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "invoke",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/InvokeBody"
              ]
            },
            {
              "name": "await",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/AwaitBody"
              ]
            },
            {
              "name": "sequence",
              "no": 3,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/SequenceBody"
              ]
            },
            {
              "name": "select",
              "no": 4,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/SelectBody"
              ]
            },
            {
              "name": "parallel",
              "no": 5,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ParallelBody"
              ]
            },
            {
              "name": "race",
              "no": 6,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/RaceBody"
              ]
            },
            {
              "name": "iterate",
              "no": 7,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/IterateBody"
              ]
            },
            {
              "name": "retry",
              "no": 8,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/RetryBody"
              ]
            },
            {
              "name": "timeout",
              "no": 9,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/TimeoutBody"
              ]
            },
            {
              "name": "handle",
              "no": 10,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/HandleBody"
              ]
            },
            {
              "name": "scope",
              "no": 11,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ScopeBody"
              ]
            },
            {
              "name": "compensation_scope",
              "no": 12,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/CompensationScopeBody"
              ]
            },
            {
              "name": "call_subflow",
              "no": 13,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/CallSubflowBody"
              ]
            }
          ]
        }
      ],
      "name": "ComputationBody",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "id",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ComputationId"
              ]
            },
            {
              "name": "outputs",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.ir@1/ComputationOutputBinding"
                ]
              ]
            },
            {
              "name": "body",
              "no": 3,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ComputationBody"
              ]
            }
          ]
        }
      ],
      "name": "ComputationDef",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "uint32"
        ]
      ],
      "name": "ComputationId",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "port_no",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/PortNo"
              ]
            },
            {
              "name": "value_slot",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ValueSlotId"
              ]
            }
          ]
        }
      ],
      "name": "ComputationOutputBinding",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "id",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ContractRefId"
              ]
            },
            {
              "name": "kind",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ContractRefKind"
              ]
            },
            {
              "name": "digest",
              "no": 3,
              "type": [
                "ref",
                "agentflow.semantic.core@1/DigestRefValue"
              ]
            }
          ]
        }
      ],
      "name": "ContractRefEntry",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "uint32"
        ]
      ],
      "name": "ContractRefId",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "component",
              "no": 1
            },
            {
              "name": "subflow",
              "no": 2
            }
          ]
        }
      ],
      "name": "ContractRefKind",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "failure_type",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/TypeSchemaRef"
              ]
            }
          ]
        }
      ],
      "name": "FailureMatch",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "source",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            },
            {
              "name": "item_slot",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ValueSlotId"
              ]
            },
            {
              "name": "body",
              "no": 3,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ComputationId"
              ]
            },
            {
              "name": "carry",
              "no": 4,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.ir@1/LoopCarryBinding"
                ]
              ]
            },
            {
              "name": "max_inflight",
              "no": 5,
              "type": [
                "option",
                [
                  "primitive",
                  "uint32"
                ]
              ]
            }
          ]
        }
      ],
      "name": "ForeachBody",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "child",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ComputationId"
              ]
            },
            {
              "name": "clauses",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.ir@1/HandleClause"
                ]
              ]
            }
          ]
        }
      ],
      "name": "HandleBody",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "ordinal",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/HandlerOrdinal"
              ]
            },
            {
              "name": "match",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/HandleMatch"
              ]
            },
            {
              "name": "handler",
              "no": 3,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ComputationId"
              ]
            }
          ]
        }
      ],
      "name": "HandleClause",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "failure",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/FailureMatch"
              ]
            },
            {
              "name": "timed_out",
              "no": 2,
              "type": [
                "primitive",
                "unit"
              ]
            }
          ]
        }
      ],
      "name": "HandleMatch",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "uint32"
        ]
      ],
      "name": "HandlerOrdinal",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "port_no",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/PortNo"
              ]
            },
            {
              "name": "expression",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            }
          ]
        }
      ],
      "name": "InputBinding",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "contract",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ContractRefId"
              ]
            },
            {
              "name": "inputs",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.ir@1/InputBinding"
                ]
              ]
            },
            {
              "name": "resources",
              "no": 3,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.ir@1/ResourceUseBinding"
                ]
              ]
            }
          ]
        }
      ],
      "name": "InvokeBody",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "foreach",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ForeachBody"
              ]
            },
            {
              "name": "while",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/WhileBody"
              ]
            }
          ]
        }
      ],
      "name": "IterateBody",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "slot",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ValueSlotId"
              ]
            },
            {
              "name": "initial",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            },
            {
              "name": "body_output_port",
              "no": 3,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/PortNo"
              ]
            }
          ]
        }
      ],
      "name": "LoopCarryBinding",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "mode",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ParallelMode"
              ]
            },
            {
              "name": "branches",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.ir@1/ParallelBranch"
                ]
              ]
            },
            {
              "name": "max_inflight",
              "no": 3,
              "type": [
                "option",
                [
                  "primitive",
                  "uint32"
                ]
              ]
            },
            {
              "name": "output_bindings",
              "no": 4,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.ir@1/ParallelOutputBinding"
                ]
              ]
            }
          ]
        }
      ],
      "name": "ParallelBody",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "ordinal",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/BranchOrdinal"
              ]
            },
            {
              "name": "child",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ComputationId"
              ]
            }
          ]
        }
      ],
      "name": "ParallelBranch",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "all",
              "no": 1
            },
            {
              "name": "fail_fast",
              "no": 2
            }
          ]
        }
      ],
      "name": "ParallelMode",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "parent_port_no",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/PortNo"
              ]
            },
            {
              "name": "branch_ordinal",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/BranchOrdinal"
              ]
            },
            {
              "name": "child_port_no",
              "no": 3,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/PortNo"
              ]
            }
          ]
        }
      ],
      "name": "ParallelOutputBinding",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "uint32"
        ]
      ],
      "name": "PortNo",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "branches",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.ir@1/RaceBranch"
                ]
              ]
            }
          ]
        }
      ],
      "name": "RaceBody",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "ordinal",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/BranchOrdinal"
              ]
            },
            {
              "name": "child",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ComputationId"
              ]
            }
          ]
        }
      ],
      "name": "RaceBranch",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "port_no",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/PortNo"
              ]
            },
            {
              "name": "resource_slot",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ResourceSlotId"
              ]
            }
          ]
        }
      ],
      "name": "ResourceUseBinding",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "child",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ComputationId"
              ]
            },
            {
              "name": "delays",
              "no": 2,
              "type": [
                "list",
                [
                  "primitive",
                  "duration"
                ]
              ]
            }
          ]
        }
      ],
      "name": "RetryBody",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "id",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/RunParameterId"
              ]
            },
            {
              "name": "name",
              "no": 2,
              "type": [
                "primitive",
                "string"
              ]
            },
            {
              "name": "type",
              "no": 3,
              "type": [
                "ref",
                "agentflow.semantic.core@1/SemanticTypeId"
              ]
            },
            {
              "name": "value_slot",
              "no": 4,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ValueSlotId"
              ]
            }
          ]
        }
      ],
      "name": "RunParameterDecl",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "uint32"
        ]
      ],
      "name": "RunParameterId",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "child",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ComputationId"
              ]
            },
            {
              "name": "resources",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.core@1/ResourceSlotId"
                ]
              ]
            }
          ]
        }
      ],
      "name": "ScopeBody",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "selector",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            },
            {
              "name": "cases",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.ir@1/SelectCase"
                ]
              ]
            }
          ]
        }
      ],
      "name": "SelectBody",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "ordinal",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/BranchOrdinal"
              ]
            },
            {
              "name": "child",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ComputationId"
              ]
            }
          ]
        }
      ],
      "name": "SelectCase",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "semantic_refs",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.ir@1/SemanticVersionRef"
                ]
              ]
            },
            {
              "name": "types",
              "no": 2,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.core@1/SemanticTypeNode"
                ]
              ]
            },
            {
              "name": "values",
              "no": 3,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.core@1/ValueSlot"
                ]
              ]
            },
            {
              "name": "expressions",
              "no": 4,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.core@1/ExpressionDef"
                ]
              ]
            },
            {
              "name": "resources",
              "no": 5,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.core@1/ResourceSlot"
                ]
              ]
            },
            {
              "name": "contracts",
              "no": 6,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.ir@1/ContractRefEntry"
                ]
              ]
            },
            {
              "name": "run_parameters",
              "no": 7,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.ir@1/RunParameterDecl"
                ]
              ]
            },
            {
              "name": "workflow_outputs",
              "no": 8,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.ir@1/WorkflowOutputDecl"
                ]
              ]
            },
            {
              "name": "computations",
              "no": 9,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.ir@1/ComputationDef"
                ]
              ]
            },
            {
              "name": "root",
              "no": 10,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ComputationId"
              ]
            }
          ]
        }
      ],
      "name": "SemanticIrDocument",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "specification_id",
              "no": 1,
              "type": [
                "primitive",
                "string"
              ]
            }
          ]
        }
      ],
      "name": "SemanticVersionRef",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "children",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.ir@1/ComputationId"
                ]
              ]
            }
          ]
        }
      ],
      "name": "SequenceBody",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "child",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ComputationId"
              ]
            },
            {
              "name": "timeout",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            }
          ]
        }
      ],
      "name": "TimeoutBody",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "guard",
              "no": 1,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            },
            {
              "name": "body",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.ir@1/ComputationId"
              ]
            },
            {
              "name": "carry",
              "no": 3,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.semantic.ir@1/LoopCarryBinding"
                ]
              ]
            },
            {
              "name": "max_iterations",
              "no": 4,
              "type": [
                "primitive",
                "uint32"
              ]
            }
          ]
        }
      ],
      "name": "WhileBody",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "name",
              "no": 1,
              "type": [
                "primitive",
                "string"
              ]
            },
            {
              "name": "expression",
              "no": 2,
              "type": [
                "ref",
                "agentflow.semantic.core@1/ExpressionId"
              ]
            }
          ]
        }
      ],
      "name": "WorkflowOutputDecl",
      "schema": "agentflow.semantic.ir@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "schema_family",
              "no": 1,
              "type": [
                "ref",
                "agentflow.spec.registry@1/SpecificationIdText"
              ]
            },
            {
              "name": "type_name",
              "no": 2,
              "type": [
                "ref",
                "agentflow.spec.registry@1/TypeName"
              ]
            },
            {
              "name": "number_kind",
              "no": 3,
              "type": [
                "ref",
                "agentflow.spec.registry@1/AbiNumberKind"
              ]
            },
            {
              "name": "number",
              "no": 4,
              "type": [
                "primitive",
                "uint32"
              ]
            },
            {
              "name": "symbolic_name",
              "no": 5,
              "type": [
                "ref",
                "agentflow.spec.registry@1/SymbolicName"
              ]
            },
            {
              "name": "status",
              "no": 6,
              "type": [
                "ref",
                "agentflow.spec.registry@1/AbiNumberStatus"
              ]
            },
            {
              "name": "first_seen_schema_digest",
              "no": 7,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.spec.registry@1/DigestRef"
                ]
              ]
            }
          ]
        }
      ],
      "name": "AbiNumberAssignment",
      "schema": "agentflow.spec.registry@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "field",
              "no": 1
            },
            {
              "name": "variant_case",
              "no": 2
            },
            {
              "name": "enum_value",
              "no": 3
            }
          ]
        }
      ],
      "name": "AbiNumberKind",
      "schema": "agentflow.spec.registry@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "active",
              "no": 1
            },
            {
              "name": "reserved",
              "no": 2
            },
            {
              "name": "retired",
              "no": 3
            }
          ]
        }
      ],
      "name": "AbiNumberStatus",
      "schema": "agentflow.spec.registry@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "algorithm",
              "no": 1,
              "type": [
                "primitive",
                "string"
              ]
            },
            {
              "name": "hex",
              "no": 2,
              "type": [
                "ref",
                "agentflow.spec.registry@1/Sha256Hex"
              ]
            }
          ]
        }
      ],
      "name": "ArtifactFileDigest",
      "schema": "agentflow.spec.registry@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "string"
        ]
      ],
      "name": "ConformanceCaseId",
      "schema": "agentflow.spec.registry@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "alg",
              "no": 1,
              "type": [
                "primitive",
                "string"
              ]
            },
            {
              "name": "domain",
              "no": 2,
              "type": [
                "primitive",
                "string"
              ]
            },
            {
              "name": "hex",
              "no": 3,
              "type": [
                "ref",
                "agentflow.spec.registry@1/Sha256Hex"
              ]
            }
          ]
        }
      ],
      "name": "DigestRef",
      "schema": "agentflow.spec.registry@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "string"
        ]
      ],
      "name": "LogicalPath",
      "schema": "agentflow.spec.registry@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "rule_id",
              "no": 1,
              "type": [
                "ref",
                "agentflow.spec.registry@1/RuleId"
              ]
            },
            {
              "name": "area",
              "no": 2,
              "type": [
                "ref",
                "agentflow.spec.registry@1/RuleArea"
              ]
            },
            {
              "name": "normative_source",
              "no": 3,
              "type": [
                "ref",
                "agentflow.spec.registry@1/LogicalPath"
              ]
            },
            {
              "name": "source_anchor",
              "no": 4,
              "type": [
                "ref",
                "agentflow.spec.registry@1/SourceAnchor"
              ]
            },
            {
              "name": "requirement_level",
              "no": 5,
              "type": [
                "ref",
                "agentflow.spec.registry@1/RequirementLevel"
              ]
            },
            {
              "name": "short_statement",
              "no": 6,
              "type": [
                "primitive",
                "string"
              ]
            },
            {
              "name": "conformance_refs",
              "no": 7,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.spec.registry@1/ConformanceCaseId"
                ]
              ]
            },
            {
              "name": "intentionally_uncovered_reason",
              "no": 8,
              "type": [
                "option",
                [
                  "primitive",
                  "string"
                ]
              ]
            }
          ]
        }
      ],
      "name": "NormativeRuleDescriptor",
      "schema": "agentflow.spec.registry@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "must",
              "no": 1
            },
            {
              "name": "must_not",
              "no": 2
            },
            {
              "name": "should",
              "no": 3
            },
            {
              "name": "should_not",
              "no": 4
            },
            {
              "name": "may",
              "no": 5
            }
          ]
        }
      ],
      "name": "RequirementLevel",
      "schema": "agentflow.spec.registry@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "string"
        ]
      ],
      "name": "RuleArea",
      "schema": "agentflow.spec.registry@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "string"
        ]
      ],
      "name": "RuleId",
      "schema": "agentflow.spec.registry@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "string"
        ]
      ],
      "name": "Sha256Hex",
      "schema": "agentflow.spec.registry@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "string"
        ]
      ],
      "name": "SourceAnchor",
      "schema": "agentflow.spec.registry@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "logical_path",
              "no": 1,
              "type": [
                "ref",
                "agentflow.spec.registry@1/LogicalPath"
              ]
            },
            {
              "name": "content_digest",
              "no": 2,
              "type": [
                "ref",
                "agentflow.spec.registry@1/ArtifactFileDigest"
              ]
            }
          ]
        }
      ],
      "name": "SpecificationArtifactRef",
      "schema": "agentflow.spec.registry@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "string"
        ]
      ],
      "name": "SpecificationIdText",
      "schema": "agentflow.spec.registry@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "encoding",
              "no": 1
            },
            {
              "name": "digest",
              "no": 2
            },
            {
              "name": "digest_domain_registry",
              "no": 3
            },
            {
              "name": "schema_language",
              "no": 4
            },
            {
              "name": "schema",
              "no": 5
            },
            {
              "name": "closed_table",
              "no": 6
            },
            {
              "name": "registry",
              "no": 7
            }
          ]
        }
      ],
      "name": "SpecificationKind",
      "schema": "agentflow.spec.registry@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "specification_id",
              "no": 1,
              "type": [
                "ref",
                "agentflow.spec.registry@1/SpecificationIdText"
              ]
            },
            {
              "name": "kind",
              "no": 2,
              "type": [
                "ref",
                "agentflow.spec.registry@1/SpecificationKind"
              ]
            },
            {
              "name": "status",
              "no": 3,
              "type": [
                "ref",
                "agentflow.spec.registry@1/SpecificationStatus"
              ]
            },
            {
              "name": "normative_source",
              "no": 4,
              "type": [
                "ref",
                "agentflow.spec.registry@1/SpecificationArtifactRef"
              ]
            },
            {
              "name": "schema_digest",
              "no": 5,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.spec.registry@1/DigestRef"
                ]
              ]
            },
            {
              "name": "description",
              "no": 6,
              "type": [
                "option",
                [
                  "primitive",
                  "string"
                ]
              ]
            }
          ]
        }
      ],
      "name": "SpecificationRegistryEntry",
      "schema": "agentflow.spec.registry@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "draft",
              "no": 1
            },
            {
              "name": "frozen",
              "no": 2
            },
            {
              "name": "deprecated",
              "no": 3
            },
            {
              "name": "withdrawn",
              "no": 4
            }
          ]
        }
      ],
      "name": "SpecificationStatus",
      "schema": "agentflow.spec.registry@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "string"
        ]
      ],
      "name": "SymbolicName",
      "schema": "agentflow.spec.registry@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "string"
        ]
      ],
      "name": "TypeName",
      "schema": "agentflow.spec.registry@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "catalog",
              "no": 1,
              "type": [
                "primitive",
                "string"
              ]
            },
            {
              "name": "code",
              "no": 2,
              "type": [
                "primitive",
                "string"
              ]
            }
          ]
        }
      ],
      "name": "CurrencyRef",
      "schema": "agentflow.usage.cost@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "catalog",
              "no": 1,
              "type": [
                "primitive",
                "string"
              ]
            },
            {
              "name": "name",
              "no": 2,
              "type": [
                "primitive",
                "string"
              ]
            }
          ]
        }
      ],
      "name": "MeasurementUnitRef",
      "schema": "agentflow.usage.cost@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "currency",
              "no": 1,
              "type": [
                "ref",
                "agentflow.usage.cost@1/CurrencyRef"
              ]
            },
            {
              "name": "atomic_units",
              "no": 2,
              "type": [
                "primitive",
                "uint64"
              ]
            }
          ]
        }
      ],
      "name": "MoneyAmount",
      "schema": "agentflow.usage.cost@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "meter",
              "no": 1,
              "type": [
                "ref",
                "agentflow.usage.signature@1/UsageMeterRef"
              ]
            },
            {
              "name": "phase",
              "no": 2,
              "type": [
                "option",
                [
                  "ref",
                  "agentflow.usage.signature@1/UsagePhase"
                ]
              ]
            },
            {
              "name": "unit",
              "no": 3,
              "type": [
                "ref",
                "agentflow.usage.cost@1/MeasurementUnitRef"
              ]
            },
            {
              "name": "rate",
              "no": 4,
              "type": [
                "ref",
                "agentflow.usage.cost@1/PriceRate"
              ]
            },
            {
              "name": "rounding",
              "no": 5,
              "type": [
                "ref",
                "agentflow.usage.cost@1/RoundingRule"
              ]
            }
          ]
        }
      ],
      "name": "PriceEntry",
      "schema": "agentflow.usage.cost@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "numerator",
              "no": 1,
              "type": [
                "primitive",
                "uint64"
              ]
            },
            {
              "name": "denominator",
              "no": 2,
              "type": [
                "primitive",
                "uint64"
              ]
            }
          ]
        }
      ],
      "name": "PriceRate",
      "schema": "agentflow.usage.cost@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "currency",
              "no": 1,
              "type": [
                "ref",
                "agentflow.usage.cost@1/CurrencyRef"
              ]
            },
            {
              "name": "atomic_unit",
              "no": 2,
              "type": [
                "ref",
                "agentflow.usage.cost@1/MeasurementUnitRef"
              ]
            },
            {
              "name": "model",
              "no": 3,
              "type": [
                "ref",
                "agentflow.usage.cost@1/PricingModel"
              ]
            },
            {
              "name": "entries",
              "no": 4,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.usage.cost@1/PriceEntry"
                ]
              ]
            }
          ]
        }
      ],
      "name": "PriceTableSnapshot",
      "schema": "agentflow.usage.cost@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "fixed_linear",
              "no": 1
            },
            {
              "name": "tiered",
              "no": 2
            },
            {
              "name": "progressive",
              "no": 3
            }
          ]
        }
      ],
      "name": "PricingModel",
      "schema": "agentflow.usage.cost@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "per_meter_unit_ceil",
              "no": 1
            },
            {
              "name": "aggregate_once_ceil",
              "no": 2
            }
          ]
        }
      ],
      "name": "RoundingRule",
      "schema": "agentflow.usage.cost@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "currency",
              "no": 1,
              "type": [
                "ref",
                "agentflow.usage.cost@1/CurrencyRef"
              ]
            },
            {
              "name": "forward",
              "no": 2,
              "type": [
                "ref",
                "agentflow.usage.cost@1/MoneyAmount"
              ]
            },
            {
              "name": "recovery",
              "no": 3,
              "type": [
                "ref",
                "agentflow.usage.cost@1/MoneyAmount"
              ]
            }
          ]
        }
      ],
      "name": "StaticCostCeiling",
      "schema": "agentflow.usage.cost@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "string"
        ]
      ],
      "name": "MeterCatalogId",
      "schema": "agentflow.usage.signature@1"
    },
    {
      "body": [
        "alias",
        [
          "primitive",
          "string"
        ]
      ],
      "name": "MeterName",
      "schema": "agentflow.usage.signature@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "meter",
              "no": 1,
              "type": [
                "ref",
                "agentflow.usage.signature@1/UsageMeterRef"
              ]
            },
            {
              "name": "phase",
              "no": 2,
              "type": [
                "ref",
                "agentflow.usage.signature@1/UsagePhase"
              ]
            },
            {
              "name": "upper",
              "no": 3,
              "type": [
                "ref",
                "agentflow.usage.signature@1/UsageUpperBound"
              ]
            }
          ]
        }
      ],
      "name": "UsageLimit",
      "schema": "agentflow.usage.signature@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "catalog",
              "no": 1,
              "type": [
                "ref",
                "agentflow.usage.signature@1/MeterCatalogId"
              ]
            },
            {
              "name": "name",
              "no": 2,
              "type": [
                "ref",
                "agentflow.usage.signature@1/MeterName"
              ]
            }
          ]
        }
      ],
      "name": "UsageMeterRef",
      "schema": "agentflow.usage.signature@1"
    },
    {
      "body": [
        "enum",
        {
          "values": [
            {
              "name": "forward",
              "no": 1
            },
            {
              "name": "recovery",
              "no": 2
            }
          ]
        }
      ],
      "name": "UsagePhase",
      "schema": "agentflow.usage.signature@1"
    },
    {
      "body": [
        "record",
        {
          "fields": [
            {
              "name": "limits",
              "no": 1,
              "type": [
                "list",
                [
                  "ref",
                  "agentflow.usage.signature@1/UsageLimit"
                ]
              ]
            }
          ]
        }
      ],
      "name": "UsageSignature",
      "schema": "agentflow.usage.signature@1"
    },
    {
      "body": [
        "variant",
        {
          "cases": [
            {
              "name": "finite",
              "no": 1,
              "type": [
                "primitive",
                "uint64"
              ]
            },
            {
              "name": "unbounded",
              "no": 2,
              "type": [
                "primitive",
                "unit"
              ]
            }
          ]
        }
      ],
      "name": "UsageUpperBound",
      "schema": "agentflow.usage.signature@1"
    }
  ],
  "model_edition": "agentflow.g01.report-shape@1",
  "model_id": "agentflow.computation.proof-report.model@1",
  "model_language": {
    "Map": "Set of complete key/value records with unique typed keys",
    "Node": "uint64 canonical value-DAG rank of the exact declared model type",
    "Opt": "explicit none:unit or some:T; missing is not None",
    "Raw": "recursively replace untrusted text positions with exact UTF8 bytes, retain native enum/option/numeric types and all sequence order",
    "Seq": "ordered semantic sequence, duplicate occurrences retained",
    "Set": "unique complete typed values in typed order; conflicting equal keys rejected",
    "imports": "complete frozen declarations below are normative logical shape input; storage references flatten to owned logical values, never native addresses",
    "nominal_ids": "distinct uint32 Computation/Expression/RunParameter/ValueSlot/ResourceSlot/Port/ContractRef; ordinals/indices checked uint64",
    "record": "all named fields required, no extra fields; written declaration order for semantic comparison",
    "unknown": "reject unknown type, field, variant, width, role, clause or edition; no reflection fallback",
    "variant": "one closed case and its complete payload; case declaration order supplies rank"
  },
  "obligation_edition": "agentflow.g01.obligation@1",
  "required_types": [
    "Scope",
    "FactKey",
    "Fact",
    "ObligationIdentity",
    "Obligation",
    "PrerequisiteKey",
    "EvidenceRef",
    "TypedContradiction",
    "ObservationSite",
    "Source",
    "Target",
    "Coordinate",
    "Requirement",
    "CanonicalValueDag"
  ],
  "scope_excluded_descendants": [
    "Fact",
    "Obligation",
    "Scope",
    "CompleteView",
    "final_report",
    "CheckedRootExportCompatibility"
  ],
  "semantic_constraints": [
    "Untrusted assertion descriptors use the deterministic **Raw(T)** companion family: transform each text/string/identifier/digest-spelling scalar position into exact UTF8 bytes, preserving its original nominal field/model tag; keep actual byte fields as bytes, fixed-width native numerics as their exact nominal widths, option tags and actual native enum cases; recursively transform every field/case payload; preserve every Vec as sequence, including bad assertion order. Native set values retain their defined set semantics. Freeze the complete resulting companion descriptors in RM, not a runtime reflection transform. Thus Raw(CompilerClaim),Raw(Context),Raw(binding) retain invalid/NFD compiler labels,paths,pins and names without normalizing them. Checked owner/source subjects use ordinary validated descriptors. Comparing a claimed text scalar to the owner compares exact UTF8 bytes, not raw/checked wrapper tags or normalized text. A truly unknown native model case still fails UnsupportedModel; unknown transport cases fail earlier. This family gives byte-exact claim preservation without an opaque serialized object or a natural-language parser.",
    "Kind ranks1–25, exactly: assertion_input,exact_pin,proof_context,resolver_provenance,compiler_identity,subflow_dependency,input_mapping,output_producer,failure_containment,effect_containment,resource_mapping,information_flow,usage_coverage,recovery_coverage,execution_source,execution_feasibility,execution_security,body_conformance,implementation_conformance,scope_authority,scope_budget,runtime_authority,publisher_authentication,control_evidence,prerequisite.",
    "Dimension = none(rank0), I1,O2,Phi3,epsilon4,R5,F6,U7,K8,X9, or supporting{predicate:PredicateId}. Supporting cases compare after root dimensions by the closed predicate case text. Role ranks follow section2; they are derived and need not appear in Identity again.",
    "Requirement={clause:RequirementClause,evidence_class:EvidenceClass}.\nRequirementClause is a closed tagged sum direct:DirectKind | recovery:RecoveryClause | control:ControlClause | prerequisite:PrerequisitePredicate. DirectKind is precisely kinds1–13 and15–23, in kind-rank order. The other three payload enums are the following ordered lists. The enclosing kind must agree with the clause family and direct payload. This explicit tag avoids conflating identically named control/direct clauses in a context-free codec.\nFor kinds1–13 and15–23, clause is that kind's tag. Recovery14 clauses in order: source_closure,action_binding,usage_profile,multiplicity,guard_compatibility,epsilon_coverage,recovery_usage_coverage,eligibility,commit_order.\nControl24: resolved_leaf,topology,output_interface,selector_totality,effect_envelope,failure_row,resource_provenance,information_flow,usage,retry_safety,child_settlement,forward_history,compensation_coverage,root_export_boundary,unsupported_control,recovery_blocked_outcome.\nPrerequisite25: closed_contract_schema,imported_module_closure,schema_universe,positive_unique_ports,failure_pins,effect_operation_membership,resource_universe,effect_resource_relation,flow_catalogs,io_resource_namespaces,additive_meter_units,recovery_usage_exhaustiveness,closed_effect_action_coverage,exact_k_identity,execution_universes,flow_execution_relation,required_dimensions,exact_relation_pins.",
    "EvidenceClass ranks: static_declaration,typed_counterexample_resolution,trusted_body,implementation_attestation,provider_receipt,authority_attestation,publisher_authentication,control_terminal_state,committed_effect_order,recovery_outcome,provider_honoring,application_idempotency. Ordinary violations KEEP the predicate's evidence class; they do not automatically become typed_counterexample_resolution.",
    "Allowed class relation is predicate-specific, not merely clause-wide: static predicates use static_declaration; body/implementation use trusted_body/implementation_attestation; runtime/publisher use authority_attestation/publisher_authentication; Scope authority/budget static_declaration with Unproven; actual owned-child settlement uses control_terminal_state, while static timeout-outcome separation and passive-AWAIT declarations use static_declaration; commit_order uses committed_effect_order; recovery_blocked_outcome uses recovery_outcome; compensation eligibility uses provider_receipt; retry facts distinguish static declarations, provider_honoring and application_idempotency predicates by their actual evidence query. Counterexample_resolution is only an explicitly declared prerequisite asking to resolve a typed contradiction. No unsupported cross-product is accepted.",
    "PredicateFamily = root_preflight | root_dimension | execution_child | root_boundary | leaf | control | root_recovery | prerequisite. ClosedPredicateCase:\n- root_preflight: assertion_input,exact_pin,proof_context,compiler_identity,subflow_dependency.\n- root_dimension: I,O,Phi,epsilon,R,F,U,K,X.\n- execution_child: source,feasibility,security.\n- root_boundary: body,implementation,scope_authority,scope_budget,runtime,publisher; subflow-body is leaf/body_conformance under its exact call path.\n- prerequisite: the18 clauses above plus conditional-applicability queries epsilon_nonempty and flow_constraints_require_relation (support predicates, not new arbitrary missing categories).\n- leaf/control/root_recovery: the exact cases in closed_predicates; dynamic wrappers keep the original PredicateId, not the wrapper function name. Each map row binds one of the closed clauses above and its frozen governing RuleRef. No site line number is a runtime case.\nThe RM roster must include both successful and non-successful branch variants and their mandatory source prerequisites; proof aggregation priority is never a substitute for this binding.",
    "Compiler={analysis_semantics_version:Text,registry_content:Bytes,rule_registry_content:Bytes,specifications:Seq<SpecificationContentIdentity>,rules:Seq<RuleContentIdentity>,support_manifest:FullSupportManifest}; each SpecificationContentIdentity={specification_id,logical_path,content_sha256,content:Bytes}; each RuleContentIdentity={rule_id,normative_source,source_anchor,short_statement}. These vectors retain original assertion sequence order when untrusted; owner-generated projections use their declared canonical order. All bytes, including complete registry rows not repeated in the four-field rule projection, are retained.",
    "FullSupportManifest={normative_sources:Set<SpecificationContentIdentity>,closed_tables:{hash_domains:Bytes,abi_numbers:Bytes,rule_areas:Bytes,afidl_primitives:Bytes},selected_specifications:Set<SpecificationId>,applied_rules:Set<RuleContentIdentity>,semantic_roles:Seq<SemanticRoleEntry>,attribution:AttributionProfile,report_shape:ModelEdition,obligation_edition:ObligationEdition}. SemanticRoleEntry and AttributionProfile reuse the exact full existing Portable AFIDL types, including all role/dependency/status fields; only the full_content_revalidation case is supported. The owner has all46 source bindings here,20 selected specifications and138 applied rules as specified in the edition plan. This complete support view is recaptured from existing Portable manifest fields/implementation-owned source bytes; it is a new non-wire report model field, NOT a new AFIDL field or a hidden 46->20 content reduction. Owner support is data built from frozen sources, not a report-dependent self-checksum.",
    "CompilerClaim is the closed sum legacy_v1{analysis_semantics_version,registry_content,rule_registry_content,specifications,rules with their original exact types} | successor_v2:Compiler. Assertion input descriptors use Raw(CompilerClaim), rather than demanding a stale v1 claim grow an invented support_manifest or canonical NFC labels. The original full old claim can therefore yield a new-owner compiler-preflight mismatch with its bytes intact. Scope's owner compiler must be successor_v2 and exact supported; these are different tagged comparison operands.",
    "Context={compiled_modules:Map<SpecificationId,CompiledModuleContents>,type_schemas:Map<LogicalTypeIdentity,CanonicalTypeSchema>,effect_catalogs:Map<SpecificationId,Set<Text>>,resource_types:{schemas:Map<LogicalTypeIdentity,CanonicalTypeSchema>,descriptors:Map<LogicalTypeIdentity,ResourceTypeDescriptor>},flow_catalogs:{confidentiality:Set<ConfidentialityLabelRef>,taints:Set<TaintRef>,sinks:Map<EffectOperationRef,Bool>},usage_meters:Map<UsageMeterRef,{meter:UsageMeterRef,unit:UsageUnitRef,aggregation:UsageAggregation}>,recovery_operations:{effects:Map<SpecificationId,Set<Text>>,reconciliation_operations:Set<{catalog:Text,name:Text}>},execution_universes:ExecutionUniverseContents,relation_snapshots:Seq<RelationSnapshotContent>,resolver_dependencies:ResolverDependencyManifest}.",
    "CompiledModuleContents={specification_id,types:Seq<CanonicalTypeSchema>}; CanonicalTypeSchema uses the exact complete TypeSchema machine grammar in AFIDL1 §§6–7, including nominal identities, field/case/value numbers and full type expressions. It is not just TypeSchemaRef. ResourceTypeDescriptor={specification_id,type_name,schema_hex,recognized_attributes:Set<Text>}. ExecutionUniverseContents={capabilities:Map<ExecutionCapabilityRef,{capability,exact_version:Text,supported_versions:Set<Text>,attributes:Set<{key:Text,value:Text}>}>,trust_domains:Set<TrustDomainRef>,regions:Set<RegionRef>,region_compatibility:Set<{from:RegionRef,to:RegionRef}>,runtime_profiles:Map<RuntimeProfileRef,{profile,family:Text,features:Set<Text>}>}. Preserve empty recognized universes and direction of compatibility edges.",
    "RelationSnapshotContent={pin:{schema:TypeSchemaRef,revision:Text,canonical_content_checksum:{algorithm:Text,hex:Text}},canonical_content:Bytes}. ResolverDependencyManifest={component_contract_digests:Seq<DigestRef>,subflow_targets:Seq<{semantic_ir_digest:DigestRef,direct_dependencies:Seq<DigestRef>,exported_contract_digest:Opt<DigestRef>}>}. ResolverAssociations adds exact caller_ir,source_node/request and checked target/export association, complete component/target content, direct/transitive dependencies; its per-association record is {caller_ir,source_node,request:Request,component_contract:Opt<Node<CheckedContractContents>>,target_ir:Opt<Node<CheckedIrContents>>,target_export:ExportState,direct_dependencies:Set<DigestRef>,transitive_dependencies:Set<DigestRef>}. Assert the resolved variant's required fields; no arbitrary all-None resolved row.",
    "RootProvenance={source_ir:DigestRef,root_node:ComputationId,root_resolver_identity:Node<ResolverAssociations>,root_context:Node<Context>,root_k_context:Node<Context>,source_ir_contents:Node<CheckedIrContents>}. CheckedContractContents and CheckedIrContents contain the complete existing ComponentContract/SemanticIrDocument values AND exact canonical input bytes/pins, not a Checked wrapper serialized as authority. RawCandidateContract uses the same named-field descriptor but retains raw claims/order before validation.",
    "AssertionGraph nodes have all fields of Raw(RootExportAssertion) (root_export.rs317–329), all Raw(binding) records (183–313) with their full declared widths/options, and all recursively nested target assertion identities. Seven mapping vectors and dependency vectors are sequences; invalid order is not repaired. Full RootInputUseSite, structural type AST, flow mapping partition/bindings, action/source/usage profiles, execution requirements and transitive targets use their exact source-defined logical models with Raw companions at untrusted text positions. All imported AFIDL payloads are full source-pinned values. A serializer containing only a count, checksum, StructuralType display string without an invertible declared grammar, or omitted universe is not this model.",
    "ContextPartition=Seq<NonemptySeq<ContextRole>>: each applicable role exactly once; members sorted numeric; classes sorted minimum. Root roles analysis0,checked_export1,snapshot2,root3,root_K4; contract roles analysis0,candidate1,old_checked2 iff present; computation roles analysis0,snapshot1,leaves2,root3,root_K4. Equal contents can occupy different classes. Never encode native addresses. This metadata is a comparison of already checked object associations, not evidence that supplied contents share an arena.",
    "TypedValue={model_type:ModelType,value:CanonicalValueDag}, with exactly one root of that model_type, is a value under a **closed ModelType descriptor**, materialized as the following finite DAG. Its self-contained DAG is finite; it never contains Scope/Fact/Obligation/report as a descendant. ModelType is an exact member of RM's closure, bound to source edition/type name and full descriptor; no arbitrary string creates a new model.",
    "ModelType={owner:SpecificationId,name:Text}: owner is an exact source-pinned imported AFIDL specification or RM@1; name must occur in its registered closed descriptor closure. Full descriptors are obtained from the complete owner/source bundle in Scope, not serialized recursively inside each ModelType. ClosedField/ClosedCase are exact text tokens present in that ModelType's approved ordered descriptor. Unknown owner/name/field/case is UnsupportedModel. This is type reference by full supported content, not an unbound local table index or arbitrary model-name string.",
    "DagNode={type:ModelType,body:NodeBody}.\nNodeBody = scalar:ScalarValue\n | record:Seq<{field:ClosedField,value:DagRank}>\n | variant:{case:ClosedCase,payload:DagRank}\n | sequence:Seq<DagRank> | set:Seq<DagRank>\n | option_none:unit | option_some:DagRank.\nScalarValue = unit:unit | null:unit | bool:Bool | int32:I32 | uint32:U32\n | int64:I64 | uint64:U64 | decimal:ExactDecimal | float64:FiniteFloat64\n | string:NfcText | bytes:Bytes | timestamp:MicrosI64 | duration:MicrosI64.\nCanonicalValueDag={nodes:Seq<DagNode>,roots:NonemptySeq<DagRank>}.\nNode<T> is a typed DagRank whose descriptor is exactly T. Roots are in ScopeSubject field order, duplicates allowed for equal-valued roots. Value nodes cannot contain Fact/Obligation/report/Scope as descendants. Identity's Scope is a complete graph; every used member/evidence node must be reachable from its subject roots. Object graphs that need extra evidence values add them as declared input roots in their registered subject, never as final-report roots.",
    "Validate acyclicity, widths/descriptors and semantic sets first. Intern equal typed values bottom-up (no hash-only equality); node height0 for scalars/empty containers, otherwise1+maximum child height. Sort each height by descriptor type identity then nonrecursive payload and ordered child ranks, assign contiguous ranks starting0; child ranks strictly smaller. Sets sort by their domain's typed key/full payload; sequence order is preserved. Require no duplicates,unreachable nodes,alternate node numbering or redundant equal nodes at decode. A duplicated equal subtree and shared subtree yield the same minimal DAG. No recursion through final report or digest fixed point exists.",
    "Identity order is scope,assertion_path,stage,derived role,dimension,kind,source,source_ordinal,mapping_ordinal,target,coordinate,rule,requirement,origin. Ordinals are repeated from validated Origin/Source solely for this key and MUST agree. Edition precedes Scope for differing editions (normalizers admit one edition only). Source/Target/Coordinate/Evidence/contradiction/observation variants use declaration ranks; other records field order above; Requirement clauses use kind-family rank then listed clause order. Rule order=(specification_id,rule_id,anchor). Scalar types are nominal; mathematical numeric comparison, exact rational Decimal, finite float after frozen zero canonicalization, bool false<true, unsigned UTF8/bytes, None<Some, lexicographic tuple/sequence prefix first. Map/set compare sorted complete logical values; DAG ranks are canonical representation, **not a shortcut for semantic numeric order**. Comparisons memoize typed node pairs and charge work.",
    "Set uniqueness: FactKey/PrerequisiteKey by complete key; obligations by complete identity; evidence/observation/contradictions by complete typed value. Conflicting key payload rejects. Semantic sequences preserve duplicates. Unknown fields/cases/type descriptors/role/domain combinations reject; new additions require new model/obligation/analysis editions under0214/0221, not an ignore-unknown option.\n"
  ],
  "semantic_model_declarations": {
    "af-computation::AppliedMultiplicity": {
      "fields": [
        {
          "collection": false,
          "field": "identity",
          "full_type": "ControlMultiplicityIdentity",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "logical_repetitions",
          "full_type": "af_effect::OccurrenceBounds",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "physical_attempts",
          "full_type": "Option<af_effect::OccurrenceBounds>",
          "option_presence": "explicit_none_or_some"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::AppliedMultiplicity",
      "source": "crates/af-computation/src/root_recovery.rs"
    },
    "af-computation::AwaitAuthorityStatus": {
      "kind": "enum",
      "owner_type": "af-computation::AwaitAuthorityStatus",
      "source": "crates/af-computation/src/summary.rs",
      "variants": {
        "NotInferred": []
      }
    },
    "af-computation::AwaitInteractionStatus": {
      "kind": "enum",
      "owner_type": "af-computation::AwaitInteractionStatus",
      "source": "crates/af-computation/src/summary.rs",
      "variants": {
        "ContractUnavailable": []
      }
    },
    "af-computation::AwaitLeafSummary": {
      "fields": [
        {
          "collection": false,
          "field": "signal_type",
          "full_type": "TypeSchemaRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "correlation",
          "full_type": "ExpressionId",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "output_bindings",
          "full_type": "Vec<(u32, ValueSlotId)>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "timeout",
          "full_type": "AwaitTimeoutSemantics",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "interaction",
          "full_type": "AwaitInteractionStatus",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "provenance",
          "full_type": "AwaitProvenanceStatus",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "authority",
          "full_type": "AwaitAuthorityStatus",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::AwaitLeafSummary",
      "source": "crates/af-computation/src/summary.rs"
    },
    "af-computation::AwaitProvenanceStatus": {
      "kind": "enum",
      "owner_type": "af-computation::AwaitProvenanceStatus",
      "source": "crates/af-computation/src/summary.rs",
      "variants": {
        "UnknownExternalInfluence": []
      }
    },
    "af-computation::AwaitTimeoutSemantics": {
      "kind": "enum",
      "owner_type": "af-computation::AwaitTimeoutSemantics",
      "source": "crates/af-computation/src/summary.rs",
      "variants": {
        "NoImplicitTimeout": []
      }
    },
    "af-computation::CheckedRootExportCompatibility": {
      "fields": [
        {
          "collection": false,
          "field": "report",
          "full_type": "RootExportProofReport",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::CheckedRootExportCompatibility",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::CompensationScopeSummary": {
      "fields": [
        {
          "collection": true,
          "field": "potential_actions",
          "full_type": "ControlDimension<Vec<ConditionalCompensationAction>>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "local_recovery_actions",
          "full_type": "ControlDimension<Vec<LocalRecoveryAction>>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "compensation_order",
          "full_type": "LeafVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "recovery_blocked_outcome",
          "full_type": "LeafVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "forward_effects_retained",
          "full_type": "bool",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "forward_outcome_retained",
          "full_type": "bool",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::CompensationScopeSummary",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::CompilerSemanticsIdentity": {
      "fields": [
        {
          "collection": false,
          "field": "analysis_semantics_version",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "registry_content",
          "full_type": "Vec<u8>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "rule_registry_content",
          "full_type": "Vec<u8>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "specifications",
          "full_type": "Vec<SpecificationContentIdentity>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "rules",
          "full_type": "Vec<RuleContentIdentity>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::CompilerSemanticsIdentity",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::ComponentDimensionSummary": {
      "fields": [
        {
          "collection": false,
          "field": "failures",
          "full_type": "FailureRow",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "effects",
          "full_type": "EffectSignature",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "resources",
          "full_type": "ResourceRequirementRow",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "information_flow",
          "full_type": "InformationFlowContract",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "usage",
          "full_type": "UsageSignature",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "recovery",
          "full_type": "RecoveryContract",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "execution",
          "full_type": "ExecutionRequirements",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "effect_flow_partitions",
          "full_type": "Vec<EffectFlowPartition>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "recovery_action_bindings",
          "full_type": "Vec<RecoveryActionBinding>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "recovery_usage_bindings",
          "full_type": "Vec<RecoveryUsageBinding>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ComponentDimensionSummary",
      "source": "crates/af-computation/src/summary.rs"
    },
    "af-computation::ComponentInputPortIdentity": {
      "fields": [
        {
          "collection": false,
          "field": "exported_contract_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "port_no",
          "full_type": "u32",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ComponentInputPortIdentity",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::ComponentOutputPortIdentity": {
      "fields": [
        {
          "collection": false,
          "field": "exported_contract_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "port_no",
          "full_type": "u32",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ComponentOutputPortIdentity",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::ConditionalCompensationAction": {
      "fields": [
        {
          "collection": false,
          "field": "source_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "forward_occurrence",
          "full_type": "ScopedEffectOccurrence",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "action_effect_occurrence",
          "full_type": "ScopedEffectOccurrence",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "identity",
          "full_type": "RecoveryActionIdentity",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "local_bound",
          "full_type": "LocalActionBound",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "potential_effect",
          "full_type": "EffectEntry",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "potential_recovery_usage",
          "full_type": "StaticUsageEnvelope",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "receipt_evidence",
          "full_type": "LeafVerdict",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ConditionalCompensationAction",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::ControlDerivationError": {
      "kind": "enum",
      "owner_type": "af-computation::ControlDerivationError",
      "source": "crates/af-computation/src/control.rs",
      "variants": {
        "ArithmeticOverflow": [
          {
            "collection": false,
            "field": "0",
            "full_type": "&'static str",
            "option_presence": "required"
          }
        ],
        "CallerDigestMismatch": [
          {
            "collection": false,
            "field": "0",
            "full_type": "ComputationId",
            "option_presence": "required"
          }
        ],
        "ClosureMismatch": [
          {
            "collection": false,
            "field": "0",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "Dataflow": [
          {
            "collection": false,
            "field": "0",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "DuplicateLeafSummary": [
          {
            "collection": false,
            "field": "0",
            "full_type": "ComputationId",
            "option_presence": "required"
          }
        ],
        "LeafContextMismatch": [
          {
            "collection": false,
            "field": "0",
            "full_type": "ComputationId",
            "option_presence": "required"
          }
        ],
        "LeafResolution": [
          {
            "collection": false,
            "field": "0",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "LeafSummaryForNonLeaf": [
          {
            "collection": false,
            "field": "0",
            "full_type": "ComputationId",
            "option_presence": "required"
          }
        ],
        "SemanticDigest": [
          {
            "collection": false,
            "field": "0",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "UnknownComputation": [
          {
            "collection": false,
            "field": "0",
            "full_type": "ComputationId",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-computation::ControlDimension": {
      "fields": [
        {
          "collection": false,
          "field": "value",
          "full_type": "ControlDimensionValue<T>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "proofs",
          "full_type": "Vec<ControlDimensionProof>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ControlDimension",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::ControlDimensionProof": {
      "fields": [
        {
          "collection": false,
          "field": "dimension",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "verdict",
          "full_type": "LeafVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "rule_id",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "reference",
          "full_type": "Option<String>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "reason",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "prerequisites",
          "full_type": "Vec<String>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ControlDimensionProof",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::ControlDimensionValue": {
      "kind": "enum",
      "owner_type": "af-computation::ControlDimensionValue",
      "source": "crates/af-computation/src/control.rs",
      "variants": {
        "Complete": [
          {
            "collection": false,
            "field": "0",
            "full_type": "T",
            "option_presence": "required"
          }
        ],
        "NotApplicable": [],
        "Partial": [
          {
            "collection": false,
            "field": "known_contribution",
            "full_type": "Option<T>",
            "option_presence": "explicit_none_or_some"
          },
          {
            "collection": true,
            "field": "unresolved_sources",
            "full_type": "Vec<ComputationId>",
            "option_presence": "required"
          }
        ],
        "PerChild": [
          {
            "collection": true,
            "field": "contributions",
            "full_type": "Vec<DimensionContribution<T>>",
            "option_presence": "required"
          },
          {
            "collection": true,
            "field": "unresolved_sources",
            "full_type": "Vec<ComputationId>",
            "option_presence": "required"
          }
        ],
        "Unknown": [
          {
            "collection": true,
            "field": "known_contributions",
            "full_type": "Vec<DimensionContribution<T>>",
            "option_presence": "required"
          },
          {
            "collection": true,
            "field": "unresolved_sources",
            "full_type": "Vec<ComputationId>",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-computation::ControlFlowContractSource": {
      "fields": [
        {
          "collection": false,
          "field": "source_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "target",
          "full_type": "ResolvedTarget",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "contract",
          "full_type": "InformationFlowContract",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "source_mappings",
          "full_type": "Vec<FlowPortMapping>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "effect_flow_partitions",
          "full_type": "Vec<af_contract::EffectFlowPartition>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ControlFlowContractSource",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::ControlFlowSummary": {
      "fields": [
        {
          "collection": false,
          "field": "known_may_influence",
          "full_type": "Option<StaticInformationSummary>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": true,
          "field": "unresolved_children",
          "full_type": "Vec<ComputationId>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "sources",
          "full_type": "Vec<ControlFlowContractSource>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "sequence_dependencies",
          "full_type": "Vec<SequenceDataflowDependency>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "select_decision_dependencies",
          "full_type": "Vec<ValueSlotId>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "control_decision_dependencies",
          "full_type": "Vec<ValueSlotId>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "control_decision_influence_unknown",
          "full_type": "bool",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "unknown_external_children",
          "full_type": "Vec<ComputationId>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ControlFlowSummary",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::ControlInterfaceSummary": {
      "fields": [
        {
          "collection": true,
          "field": "outputs",
          "full_type": "ControlDimensionValue<Vec<ControlOutputPort>>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "selector",
          "full_type": "Option<SelectDecisionEvidence>",
          "option_presence": "explicit_none_or_some"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ControlInterfaceSummary",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::ControlMultiplicityIdentity": {
      "fields": [
        {
          "collection": false,
          "field": "caller_semantic_ir_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "control_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "kind",
          "full_type": "ControlMultiplicityKind",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ControlMultiplicityIdentity",
      "source": "crates/af-computation/src/root_recovery.rs"
    },
    "af-computation::ControlMultiplicityKind": {
      "kind": "enum",
      "owner_type": "af-computation::ControlMultiplicityKind",
      "source": "crates/af-computation/src/root_recovery.rs",
      "variants": {
        "Iterate": [],
        "Retry": []
      }
    },
    "af-computation::ControlOperator": {
      "kind": "enum",
      "owner_type": "af-computation::ControlOperator",
      "source": "crates/af-computation/src/control.rs",
      "variants": {
        "Await": [],
        "CallSubflow": [],
        "CompensationScope": [],
        "Handle": [
          {
            "collection": false,
            "field": "handled",
            "full_type": "FailureRow",
            "option_presence": "required"
          },
          {
            "collection": true,
            "field": "clauses",
            "full_type": "Vec<HandleClauseSummary>",
            "option_presence": "required"
          }
        ],
        "Invoke": [],
        "Iterate": [
          {
            "collection": false,
            "field": "mode",
            "full_type": "IterateMode",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "effect_repetitions",
            "full_type": "OccurrenceBounds",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "usage_repetitions",
            "full_type": "Option<u64>",
            "option_presence": "explicit_none_or_some"
          }
        ],
        "Parallel": [
          {
            "collection": false,
            "field": "mode",
            "full_type": "ParallelMode",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "max_inflight",
            "full_type": "Option<u32>",
            "option_presence": "explicit_none_or_some"
          }
        ],
        "RaceFirstTerminal": [],
        "Retry": [
          {
            "collection": true,
            "field": "delays",
            "full_type": "Vec<af_core::Duration>",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "max_attempts",
            "full_type": "u64",
            "option_presence": "required"
          }
        ],
        "Scope": [],
        "Select": [],
        "Sequence": [],
        "Timeout": [
          {
            "collection": false,
            "field": "expression",
            "full_type": "ExpressionId",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "static_duration",
            "full_type": "Option<af_core::Duration>",
            "option_presence": "explicit_none_or_some"
          },
          {
            "collection": false,
            "field": "possible_outcomes",
            "full_type": "[TimeoutOutcome; 2]",
            "option_presence": "required"
          }
        ],
        "Unsupported": [
          {
            "collection": false,
            "field": "opcode",
            "full_type": "String",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-computation::ControlOutputPort": {
      "fields": [
        {
          "collection": false,
          "field": "parent_port_no",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "parent_value_slot",
          "full_type": "ValueSlotId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "local_type_id",
          "full_type": "SemanticTypeId",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "sources",
          "full_type": "Vec<ControlOutputSource>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ControlOutputPort",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::ControlOutputSource": {
      "fields": [
        {
          "collection": false,
          "field": "node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "port_no",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "value_slot",
          "full_type": "ValueSlotId",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ControlOutputSource",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::ControlPathKind": {
      "kind": "enum",
      "owner_type": "af-computation::ControlPathKind",
      "source": "crates/af-computation/src/root_recovery.rs",
      "variants": {
        "CompensationScopeChild": [],
        "HandleClause": [
          {
            "collection": false,
            "field": "ordinal",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "HandleProtected": [],
        "IterateBody": [],
        "ParallelBranch": [
          {
            "collection": false,
            "field": "branch_ordinal",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "RaceBranch": [
          {
            "collection": false,
            "field": "branch_ordinal",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "RetryChild": [
          {
            "collection": false,
            "field": "max_attempts",
            "full_type": "u64",
            "option_presence": "required"
          }
        ],
        "ScopeChild": [],
        "SelectCase": [
          {
            "collection": false,
            "field": "case_ordinal",
            "full_type": "u32",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "selector",
            "full_type": "Option<ExpressionId>",
            "option_presence": "explicit_none_or_some"
          },
          {
            "collection": true,
            "field": "selector_dependencies",
            "full_type": "Vec<ValueSlotId>",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "exact_case",
            "full_type": "Option<u32>",
            "option_presence": "explicit_none_or_some"
          }
        ],
        "SequenceChild": [],
        "TimeoutChild": []
      }
    },
    "af-computation::ControlPathReachability": {
      "kind": "enum",
      "owner_type": "af-computation::ControlPathReachability",
      "source": "crates/af-computation/src/root_recovery.rs",
      "variants": {
        "ExactSelected": [],
        "Possible": [],
        "ProvenUnreachable": []
      }
    },
    "af-computation::ControlPathStep": {
      "fields": [
        {
          "collection": false,
          "field": "parent_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "child_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "child_index",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "kind",
          "full_type": "ControlPathKind",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "reachability",
          "full_type": "ControlPathReachability",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ControlPathStep",
      "source": "crates/af-computation/src/root_recovery.rs"
    },
    "af-computation::ControlProofReport": {
      "fields": [
        {
          "collection": true,
          "field": "proofs",
          "full_type": "Vec<ControlDimensionProof>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ControlProofReport",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::ControlResourceAlias": {
      "fields": [
        {
          "collection": false,
          "field": "resource_slot",
          "full_type": "ResourceSlotId",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "occurrences",
          "full_type": "Vec<ResourcePortOccurrence>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ControlResourceAlias",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::ControlTreeClosureSummary": {
      "fields": [
        {
          "collection": true,
          "field": "computation_order",
          "full_type": "Vec<ComputationId>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "leaf_order",
          "full_type": "Vec<ComputationId>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "known_effect_occurrences_accounted",
          "full_type": "bool",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "known_effect_residuals_accounted",
          "full_type": "bool",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "known_recovery_sources_accounted",
          "full_type": "bool",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "root_export_compatibility",
          "full_type": "RootExportCompatibilityStatus",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "proof",
          "full_type": "ControlProofReport",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ControlTreeClosureSummary",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::DerivedControlContractSummary": {
      "fields": [
        {
          "collection": false,
          "field": "source_semantic_ir_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "control_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "operator",
          "full_type": "ControlOperator",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "direct_children",
          "full_type": "Vec<ComputationId>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "children",
          "full_type": "Vec<DerivedControlContractSummary<'context, 'universe>>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "leaf_summary",
          "full_type": "Option<ResolvedComputationLeaf<'context, 'universe>>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "interface",
          "full_type": "ControlInterfaceSummary",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "scope_boundary",
          "full_type": "Option<ScopeBoundarySummary>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "compensation_scope",
          "full_type": "Option<CompensationScopeSummary>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "failures",
          "full_type": "ControlDimension<FailureRow>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "effects",
          "full_type": "ControlDimension<EffectSignature>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "resources",
          "full_type": "ControlDimension<Vec<ResourcePortOccurrence>>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "information_flow",
          "full_type": "ControlDimension<ControlFlowSummary>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "usage",
          "full_type": "ControlDimension<StaticUsageEnvelope>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "recovery",
          "full_type": "ControlDimension<Vec<RecoverySourceSummary>>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "execution",
          "full_type": "ControlDimension<Vec<ExecutionSourceSummary>>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "effect_occurrences",
          "full_type": "Vec<ScopedEffectOccurrence>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "effect_residuals",
          "full_type": "Vec<ScopedEffectResidual>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "resource_aliases",
          "full_type": "Vec<ControlResourceAlias>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "effect_lower_bound_adjustments",
          "full_type": "Vec<EffectLowerBoundAdjustment>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "sequence_dataflow_dependencies",
          "full_type": "Vec<SequenceDataflowDependency>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "inherited_obligations",
          "full_type": "Vec<InheritedControlObligation>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "proof",
          "full_type": "ControlProofReport",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "proof_context",
          "full_type": "ProofContextReference<'context, 'universe>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::DerivedControlContractSummary",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::DerivedWorkflowContractSummary": {
      "fields": [
        {
          "collection": false,
          "field": "root",
          "full_type": "DerivedControlContractSummary<'context, 'universe>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "closure",
          "full_type": "ControlTreeClosureSummary",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "root_recovery",
          "full_type": "RootRecoverySummary",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "resolver_identity",
          "full_type": "ResolverSnapshotIdentity",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::DerivedWorkflowContractSummary",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::DimensionContribution": {
      "fields": [
        {
          "collection": false,
          "field": "source_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "value",
          "full_type": "T",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::DimensionContribution",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::EffectDimensionResult": {
      "fields": [
        {
          "collection": false,
          "field": "dimension",
          "full_type": "ControlDimension<EffectSignature>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "proofs",
          "full_type": "Vec<ControlDimensionProof>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "occurrences",
          "full_type": "Vec<ScopedEffectOccurrence>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "residuals",
          "full_type": "Vec<ScopedEffectResidual>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "adjustments",
          "full_type": "Vec<EffectLowerBoundAdjustment>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::EffectDimensionResult",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::EffectLowerBoundAdjustment": {
      "fields": [
        {
          "collection": false,
          "field": "source_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "reason",
          "full_type": "EffectLowerBoundAdjustmentReason",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::EffectLowerBoundAdjustment",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::EffectLowerBoundAdjustmentReason": {
      "kind": "enum",
      "owner_type": "af-computation::EffectLowerBoundAdjustmentReason",
      "source": "crates/af-computation/src/control.rs",
      "variants": {
        "FailFastSiblingCanBeCancelled": [],
        "RaceBranchMayLoseOrBeCancelled": [],
        "SelectMayHaveNoCoveredCase": [],
        "SequenceContinuationCanBeSkipped": []
      }
    },
    "af-computation::EffectOccurrenceFamily": {
      "fields": [
        {
          "collection": false,
          "field": "control_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "kind",
          "full_type": "EffectOccurrenceFamilyKind",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "logical_repetitions",
          "full_type": "OccurrenceBounds",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "physical_attempts",
          "full_type": "Option<OccurrenceBounds>",
          "option_presence": "explicit_none_or_some"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::EffectOccurrenceFamily",
      "source": "crates/af-computation/src/summary.rs"
    },
    "af-computation::EffectOccurrenceFamilyKind": {
      "kind": "enum",
      "owner_type": "af-computation::EffectOccurrenceFamilyKind",
      "source": "crates/af-computation/src/summary.rs",
      "variants": {
        "Iterate": [],
        "Retry": []
      }
    },
    "af-computation::EmptyLeaf": {
      "fields": [
        {
          "collection": false,
          "field": "computation",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "caller_semantic_ir_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "kind",
          "full_type": "LeafKind",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "target",
          "full_type": "Option<ResolvedTarget>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": true,
          "field": "inputs",
          "full_type": "Vec<ScopedPortMapping>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "outputs",
          "full_type": "Vec<ScopedPortMapping>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "resources",
          "full_type": "Vec<ScopedResourcePortMapping>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "aliases",
          "full_type": "Vec<ResourceAliasGroup>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "effects",
          "full_type": "Vec<ScopedEffectOccurrence>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "effect_residuals",
          "full_type": "Vec<ScopedEffectResidual>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "flows",
          "full_type": "Vec<FlowPortMapping>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "dimensions",
          "full_type": "Option<ComponentDimensionSummary>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": true,
          "field": "action_guard_compatibility",
          "full_type": "Vec<ActionGuardCompatibility>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "await_summary",
          "full_type": "Option<AwaitLeafSummary>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": true,
          "field": "proofs",
          "full_type": "Vec<LeafDimensionProof>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::EmptyLeaf",
      "source": "crates/af-computation/src/resolution.rs"
    },
    "af-computation::ExecutionSourceSummary": {
      "fields": [
        {
          "collection": false,
          "field": "source_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "target",
          "full_type": "ResolvedTarget",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "requirements",
          "full_type": "ExecutionRequirements",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ExecutionSourceSummary",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::ExportedContract": {
      "fields": [
        {
          "collection": false,
          "field": "checked",
          "full_type": "&'a CheckedComponentContract<'context, 'universe>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ExportedContract",
      "source": "crates/af-computation/src/resolution.rs"
    },
    "af-computation::FlowPortMapping": {
      "fields": [
        {
          "collection": false,
          "field": "source",
          "full_type": "Option<FlowSourceRef>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "sink",
          "full_type": "Option<FlowConstraint>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "source_binding",
          "full_type": "Option<FlowSourceBinding>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "sink_binding",
          "full_type": "Option<FlowSinkBinding>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": true,
          "field": "matched_effect_occurrences",
          "full_type": "Vec<usize>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::FlowPortMapping",
      "source": "crates/af-computation/src/summary.rs"
    },
    "af-computation::FlowSinkBinding": {
      "kind": "enum",
      "owner_type": "af-computation::FlowSinkBinding",
      "source": "crates/af-computation/src/summary.rs",
      "variants": {
        "EffectOccurrences": [
          {
            "collection": true,
            "field": "0",
            "full_type": "Vec<usize>",
            "option_presence": "required"
          }
        ],
        "OutputValue": [
          {
            "collection": false,
            "field": "0",
            "full_type": "ValueSlotId",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-computation::FlowSourceBinding": {
      "kind": "enum",
      "owner_type": "af-computation::FlowSourceBinding",
      "source": "crates/af-computation/src/summary.rs",
      "variants": {
        "InputExpression": [
          {
            "collection": false,
            "field": "0",
            "full_type": "ExpressionId",
            "option_presence": "required"
          }
        ],
        "ResourceSlot": [
          {
            "collection": false,
            "field": "0",
            "full_type": "ResourceSlotId",
            "option_presence": "required"
          }
        ],
        "UnknownExternal": []
      }
    },
    "af-computation::HandleClauseSummary": {
      "fields": [
        {
          "collection": false,
          "field": "ordinal",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "matcher",
          "full_type": "HandleMatcherSummary",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "handler",
          "full_type": "ComputationId",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::HandleClauseSummary",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::HandleMatcherSummary": {
      "kind": "enum",
      "owner_type": "af-computation::HandleMatcherSummary",
      "source": "crates/af-computation/src/control.rs",
      "variants": {
        "Failure": [
          {
            "collection": false,
            "field": "0",
            "full_type": "TypeSchemaRef",
            "option_presence": "required"
          }
        ],
        "TimedOut": []
      }
    },
    "af-computation::InheritedControlObligation": {
      "fields": [
        {
          "collection": false,
          "field": "source_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "dimension",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "verdict",
          "full_type": "LeafVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "rule_id",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "reference",
          "full_type": "Option<String>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "reason",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::InheritedControlObligation",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::IterateMode": {
      "kind": "enum",
      "owner_type": "af-computation::IterateMode",
      "source": "crates/af-computation/src/control.rs",
      "variants": {
        "Foreach": [
          {
            "collection": false,
            "field": "statically_known_item_count",
            "full_type": "Option<u64>",
            "option_presence": "explicit_none_or_some"
          },
          {
            "collection": false,
            "field": "max_inflight",
            "full_type": "Option<u32>",
            "option_presence": "explicit_none_or_some"
          }
        ],
        "While": [
          {
            "collection": false,
            "field": "max_iterations",
            "full_type": "u32",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-computation::LeafDimensionProof": {
      "fields": [
        {
          "collection": false,
          "field": "dimension",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "verdict",
          "full_type": "LeafVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "rule_id",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "reference",
          "full_type": "Option<String>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "reason",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "prerequisites",
          "full_type": "Vec<String>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::LeafDimensionProof",
      "source": "crates/af-computation/src/proof.rs"
    },
    "af-computation::LeafKind": {
      "kind": "enum",
      "owner_type": "af-computation::LeafKind",
      "source": "crates/af-computation/src/summary.rs",
      "variants": {
        "Await": [],
        "CallSubflow": [],
        "Invoke": []
      }
    },
    "af-computation::LeafProofReport": {
      "fields": [
        {
          "collection": true,
          "field": "dimensions",
          "full_type": "Vec<LeafDimensionProof>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::LeafProofReport",
      "source": "crates/af-computation/src/proof.rs"
    },
    "af-computation::LeafResolutionError": {
      "kind": "enum",
      "owner_type": "af-computation::LeafResolutionError",
      "source": "crates/af-computation/src/resolution.rs",
      "variants": {
        "NotAComputationLeaf": [
          {
            "collection": false,
            "field": "0",
            "full_type": "ComputationId",
            "option_presence": "required"
          }
        ],
        "SemanticIrDigest": [
          {
            "collection": false,
            "field": "0",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "UnknownComputation": [
          {
            "collection": false,
            "field": "0",
            "full_type": "ComputationId",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-computation::LeafVerdict": {
      "kind": "enum",
      "owner_type": "af-computation::LeafVerdict",
      "source": "crates/af-computation/src/proof.rs",
      "variants": {
        "Proven": [],
        "RequiresEvidence": [],
        "Unproven": [],
        "Violation": []
      }
    },
    "af-computation::LocalActionBound": {
      "fields": [
        {
          "collection": false,
          "field": "compensation_scope_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "upper",
          "full_type": "RecoveryActionUpper",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "applied_families",
          "full_type": "Vec<AppliedMultiplicity>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::LocalActionBound",
      "source": "crates/af-computation/src/root_recovery.rs"
    },
    "af-computation::LocalRecoveryAction": {
      "fields": [
        {
          "collection": false,
          "field": "source",
          "full_type": "RecoveryActionSourceIdentity",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "forward_seed_upper",
          "full_type": "OccurrenceCeiling",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "owner_scope",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "action_kind",
          "full_type": "RecoveryActionKind",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "local_bound",
          "full_type": "LocalActionBound",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "forward_occurrence",
          "full_type": "ScopedEffectOccurrence",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "action_effect_occurrence",
          "full_type": "ScopedEffectOccurrence",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "recovery_usage_binding",
          "full_type": "RecoveryUsageBinding",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "recovery_usage",
          "full_type": "StaticUsageEnvelope",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "guard_compatibility",
          "full_type": "Option<ActionGuardCompatibility>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "eligibility_evidence",
          "full_type": "LeafVerdict",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::LocalRecoveryAction",
      "source": "crates/af-computation/src/root_recovery.rs"
    },
    "af-computation::ProofContextManifest": {
      "fields": [
        {
          "collection": false,
          "field": "compiled_modules",
          "full_type": "af_idl::CompiledModuleSet",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "type_schemas",
          "full_type": "af_semantic::TypeSchemaUniverse",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "effect_catalogs",
          "full_type": "af_effect::EffectCatalogUniverse",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "resource_types",
          "full_type": "af_resource::ResourceTypeUniverse",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "flow_catalogs",
          "full_type": "af_flow::FlowCatalogs",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "usage_meters",
          "full_type": "af_usage::UsageMeterUniverse",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "recovery_operations",
          "full_type": "af_recovery::RecoveryOperationUniverse",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "execution_universes",
          "full_type": "af_execution::ExecutionUniverses",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "relation_snapshots",
          "full_type": "Vec<RelationSnapshotContent>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "resolver_dependencies",
          "full_type": "ResolverDependencyManifest",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ProofContextManifest",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::ProofContextReference": {
      "fields": [
        {
          "collection": false,
          "field": "context",
          "full_type": "&'context StaticContractProofContext<'universe>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "relation_catalog_pins",
          "full_type": "Vec<RelationCatalogPin>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ProofContextReference",
      "source": "crates/af-computation/src/summary.rs"
    },
    "af-computation::RecoveryActionSourceIdentity": {
      "fields": [
        {
          "collection": false,
          "field": "caller_semantic_ir_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_contract_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "forward_entry_ordinal",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "action",
          "full_type": "RecoveryActionIdentity",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RecoveryActionSourceIdentity",
      "source": "crates/af-computation/src/root_recovery.rs"
    },
    "af-computation::RecoveryActionUpper": {
      "kind": "enum",
      "owner_type": "af-computation::RecoveryActionUpper",
      "source": "crates/af-computation/src/root_recovery.rs",
      "variants": {
        "InvalidMultiplicity": [],
        "Known": [
          {
            "collection": false,
            "field": "0",
            "full_type": "OccurrenceCeiling",
            "option_presence": "required"
          }
        ],
        "Overflow": [],
        "Unknown": []
      }
    },
    "af-computation::RecoverySourceSummary": {
      "fields": [
        {
          "collection": false,
          "field": "source_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "target",
          "full_type": "ResolvedTarget",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "contract",
          "full_type": "RecoveryContract",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "action_bindings",
          "full_type": "Vec<RecoveryActionBinding>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "usage_bindings",
          "full_type": "Vec<RecoveryUsageBinding>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "effects",
          "full_type": "Vec<ScopedEffectOccurrence>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "action_guard_compatibility",
          "full_type": "Vec<ActionGuardCompatibility>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RecoverySourceSummary",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::RelationSnapshotContent": {
      "fields": [
        {
          "collection": false,
          "field": "pin",
          "full_type": "RelationCatalogPin",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "canonical_content",
          "full_type": "Vec<u8>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RelationSnapshotContent",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::ResolvedComputationLeaf": {
      "fields": [
        {
          "collection": false,
          "field": "computation",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "caller_semantic_ir_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "kind",
          "full_type": "LeafKind",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "target",
          "full_type": "Option<ResolvedTarget>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": true,
          "field": "input_mappings",
          "full_type": "Vec<ScopedPortMapping>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "output_mappings",
          "full_type": "Vec<ScopedPortMapping>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "resource_mappings",
          "full_type": "Vec<ScopedResourcePortMapping>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "resource_aliases",
          "full_type": "Vec<ResourceAliasGroup>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "effect_occurrences",
          "full_type": "Vec<ScopedEffectOccurrence>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "effect_residuals",
          "full_type": "Vec<ScopedEffectResidual>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "flow_mappings",
          "full_type": "Vec<FlowPortMapping>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "dimensions",
          "full_type": "Option<ComponentDimensionSummary>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": true,
          "field": "action_guard_compatibility",
          "full_type": "Vec<ActionGuardCompatibility>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "await_summary",
          "full_type": "Option<AwaitLeafSummary>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "proof",
          "full_type": "LeafProofReport",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "proof_context",
          "full_type": "ProofContextReference<'context, 'universe>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ResolvedComputationLeaf",
      "source": "crates/af-computation/src/summary.rs"
    },
    "af-computation::ResolvedTarget": {
      "kind": "enum",
      "owner_type": "af-computation::ResolvedTarget",
      "source": "crates/af-computation/src/summary.rs",
      "variants": {
        "AwaitSignal": [
          {
            "collection": false,
            "field": "signal_type",
            "full_type": "TypeSchemaRef",
            "option_presence": "required"
          }
        ],
        "Component": [
          {
            "collection": false,
            "field": "identity",
            "full_type": "ComponentIdentity",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "contract_digest",
            "full_type": "DigestRefValue",
            "option_presence": "required"
          }
        ],
        "Subflow": [
          {
            "collection": false,
            "field": "semantic_ir_digest",
            "full_type": "DigestRefValue",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "exported_contract_digest",
            "full_type": "Option<DigestRefValue>",
            "option_presence": "explicit_none_or_some"
          }
        ]
      }
    },
    "af-computation::ResolverDependencyManifest": {
      "fields": [
        {
          "collection": true,
          "field": "component_contract_digests",
          "full_type": "Vec<DigestRefValue>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "subflow_targets",
          "full_type": "Vec<SubflowTargetContextPin>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ResolverDependencyManifest",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::ResolverSnapshotIdentity": {
      "fields": [
        {
          "collection": true,
          "field": "component_contract_digests",
          "full_type": "Vec<DigestRefValue>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "subflow_targets",
          "full_type": "Vec<ResolverSubflowIdentity>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ResolverSnapshotIdentity",
      "source": "crates/af-computation/src/resolution.rs"
    },
    "af-computation::ResolverSubflowIdentity": {
      "fields": [
        {
          "collection": false,
          "field": "semantic_ir_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "direct_dependencies",
          "full_type": "Option<Vec<DigestRefValue>>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "exported_contract_digest",
          "full_type": "Option<DigestRefValue>",
          "option_presence": "explicit_none_or_some"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ResolverSubflowIdentity",
      "source": "crates/af-computation/src/resolution.rs"
    },
    "af-computation::ResourceAliasGroup": {
      "fields": [
        {
          "collection": false,
          "field": "resource_slot",
          "full_type": "ResourceSlotId",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "component_ports",
          "full_type": "Vec<u32>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ResourceAliasGroup",
      "source": "crates/af-computation/src/summary.rs"
    },
    "af-computation::ResourcePortOccurrence": {
      "fields": [
        {
          "collection": false,
          "field": "source_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "target",
          "full_type": "ResolvedTarget",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "requirement",
          "full_type": "Option<ResourceRequirement>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "mapping",
          "full_type": "ScopedResourcePortMapping",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ResourcePortOccurrence",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::ResourceRequirementPortIdentity": {
      "fields": [
        {
          "collection": false,
          "field": "exported_contract_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "port_no",
          "full_type": "u32",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ResourceRequirementPortIdentity",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::RootActionBound": {
      "fields": [
        {
          "collection": false,
          "field": "root_semantic_ir_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "upper",
          "full_type": "RecoveryActionUpper",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "applied_families",
          "full_type": "Vec<AppliedMultiplicity>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RootActionBound",
      "source": "crates/af-computation/src/root_recovery.rs"
    },
    "af-computation::RootEffectCoordinateBinding": {
      "fields": [
        {
          "collection": false,
          "field": "source",
          "full_type": "RootEffectOccurrenceIdentity",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "root_resource_slot",
          "full_type": "Option<ResourceSlotId>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "export_atom",
          "full_type": "EffectAtom",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "export_guard",
          "full_type": "EffectGuard",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RootEffectCoordinateBinding",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::RootEffectOccurrenceIdentity": {
      "fields": [
        {
          "collection": false,
          "field": "caller_semantic_ir_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_contract_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_entry_ordinal",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "atom",
          "full_type": "EffectAtom",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "guard",
          "full_type": "EffectGuard",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RootEffectOccurrenceIdentity",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::RootExecutionChildProof": {
      "fields": [
        {
          "collection": false,
          "field": "source_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_contract_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "feasibility",
          "full_type": "af_execution::CompatibilityVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "security",
          "full_type": "af_execution::CompatibilityVerdict",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RootExecutionChildProof",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::RootExecutionCoordinateBinding": {
      "fields": [
        {
          "collection": false,
          "field": "source_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_contract_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "exact_execution_requirements",
          "full_type": "ExecutionRequirements",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RootExecutionCoordinateBinding",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::RootExportAssertion": {
      "fields": [
        {
          "collection": false,
          "field": "exact_semantic_ir_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "exact_exported_component_contract_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "compiler_semantics_identity",
          "full_type": "CompilerSemanticsIdentity",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "proof_context_manifest",
          "full_type": "ProofContextManifest",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "input_mappings",
          "full_type": "Vec<RootInputPortBinding>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "output_mappings",
          "full_type": "Vec<RootOutputPortBinding>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "resource_mappings",
          "full_type": "Vec<RootResourcePortBinding>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "effect_mappings",
          "full_type": "Vec<RootEffectCoordinateBinding>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "flow_mappings",
          "full_type": "Vec<RootFlowCoordinateBinding>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "recovery_mappings",
          "full_type": "Vec<RootRecoveryCoordinateBinding>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "execution_mappings",
          "full_type": "Vec<RootExecutionCoordinateBinding>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "subflow_dependencies",
          "full_type": "Vec<SubflowProofDependency>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RootExportAssertion",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::RootExportAssertionIdentity": {
      "fields": [
        {
          "collection": false,
          "field": "0",
          "full_type": "RootExportAssertion",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RootExportAssertionIdentity",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::RootExportCompatibilityStatus": {
      "kind": "enum",
      "owner_type": "af-computation::RootExportCompatibilityStatus",
      "source": "crates/af-computation/src/control.rs",
      "variants": {
        "NotExpressedInSemanticIr": []
      }
    },
    "af-computation::RootExportCoordinateWitness": {
      "kind": "enum",
      "owner_type": "af-computation::RootExportCoordinateWitness",
      "source": "crates/af-computation/src/root_export.rs",
      "variants": {
        "Effect": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Box<RootEffectCoordinateBinding>",
            "option_presence": "required"
          }
        ],
        "Execution": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Box<RootExecutionCoordinateBinding>",
            "option_presence": "required"
          }
        ],
        "Flow": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Box<RootFlowCoordinateBinding>",
            "option_presence": "required"
          }
        ],
        "Input": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Box<RootInputPortBinding>",
            "option_presence": "required"
          }
        ],
        "Output": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Box<RootOutputPortBinding>",
            "option_presence": "required"
          }
        ],
        "Recovery": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Box<RootRecoveryCoordinateBinding>",
            "option_presence": "required"
          }
        ],
        "Resource": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Box<RootResourcePortBinding>",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-computation::RootExportDimension": {
      "kind": "enum",
      "owner_type": "af-computation::RootExportDimension",
      "source": "crates/af-computation/src/root_export.rs",
      "variants": {
        "Effect": [],
        "Execution": [],
        "Failure": [],
        "InformationFlow": [],
        "Input": [],
        "Output": [],
        "Recovery": [],
        "Resource": [],
        "Usage": []
      }
    },
    "af-computation::RootExportDimensionProof": {
      "fields": [
        {
          "collection": false,
          "field": "dimension",
          "full_type": "RootExportDimension",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "proof",
          "full_type": "DimensionProof",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "source_facts",
          "full_type": "Vec<RootExportSourceReference>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "mapped_coordinates",
          "full_type": "Vec<RootExportCoordinateWitness>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "mapped_coordinate_count",
          "full_type": "usize",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RootExportDimensionProof",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::RootExportProofReport": {
      "fields": [
        {
          "collection": false,
          "field": "assertion_identity",
          "full_type": "RootExportAssertionIdentity",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "preflight",
          "full_type": "Vec<DimensionProof>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "dimensions",
          "full_type": "Vec<RootExportDimensionProof>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "execution_children",
          "full_type": "Vec<RootExecutionChildProof>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "body_conformance",
          "full_type": "af_contract::ProofVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "implementation_conformance",
          "full_type": "af_contract::ProofVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "scope_authority_attenuation",
          "full_type": "af_contract::ProofVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "scope_budget_attenuation",
          "full_type": "af_contract::ProofVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "runtime_authority",
          "full_type": "RuntimeAuthorityStatus",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "runtime_authority_evidence",
          "full_type": "af_contract::ProofVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "publisher_authentication",
          "full_type": "RuntimeAuthorityStatus",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "unresolved_obligations",
          "full_type": "Vec<String>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "static_compatibility",
          "full_type": "af_contract::ProofVerdict",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RootExportProofReport",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::RootExportSourceReference": {
      "fields": [
        {
          "collection": false,
          "field": "semantic_ir_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "computation",
          "full_type": "Option<ComputationId>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "contract_digest",
          "full_type": "Option<DigestRefValue>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "source_ordinal",
          "full_type": "Option<u64>",
          "option_presence": "explicit_none_or_some"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RootExportSourceReference",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::RootFlowCoordinateBinding": {
      "fields": [
        {
          "collection": false,
          "field": "source_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_contract_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "flow_mapping_ordinal",
          "full_type": "u64",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source",
          "full_type": "Option<FlowSourceRef>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "sink",
          "full_type": "Option<FlowConstraint>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "source_binding",
          "full_type": "Option<FlowSourceBinding>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "sink_binding",
          "full_type": "Option<FlowSinkBinding>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "export_source",
          "full_type": "Option<FlowSourceRef>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "export_sink",
          "full_type": "Option<FlowConstraint>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "source_effect_partition",
          "full_type": "Option<EffectFlowPartition>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "export_effect_partition",
          "full_type": "Option<EffectFlowPartition>",
          "option_presence": "explicit_none_or_some"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RootFlowCoordinateBinding",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::RootGuardCoordinateEvidence": {
      "fields": [
        {
          "collection": false,
          "field": "source_contract_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "component_input_port",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "input_expression",
          "full_type": "ExpressionId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "exact_type",
          "full_type": "Option<af_semantic::TypeSchemaRef>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": true,
          "field": "dependency_slots",
          "full_type": "Vec<ValueSlotId>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RootGuardCoordinateEvidence",
      "source": "crates/af-computation/src/root_recovery.rs"
    },
    "af-computation::RootInputPortBinding": {
      "fields": [
        {
          "collection": false,
          "field": "export_input_port",
          "full_type": "ComponentInputPortIdentity",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "run_parameter_id",
          "full_type": "RunParameterId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "root_value_slot",
          "full_type": "ValueSlotId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "root_type",
          "full_type": "StructuralType",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "export_type",
          "full_type": "TypeSchemaRef",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "use_sites",
          "full_type": "Vec<RootInputUseSite>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RootInputPortBinding",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::RootInputUseSite": {
      "fields": [
        {
          "collection": false,
          "field": "computation",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_contract_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_component_port",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "input_expression",
          "full_type": "ExpressionId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "exact_type",
          "full_type": "TypeSchemaRef",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "dependency_slots",
          "full_type": "Vec<ValueSlotId>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RootInputUseSite",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::RootOutputPortBinding": {
      "fields": [
        {
          "collection": false,
          "field": "export_output_port",
          "full_type": "ComponentOutputPortIdentity",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "workflow_output_name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "workflow_output_expression",
          "full_type": "ExpressionId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "root_result_type",
          "full_type": "StructuralType",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "producer_evidence",
          "full_type": "RootOutputProducerEvidence",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RootOutputPortBinding",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::RootOutputProducerEvidence": {
      "fields": [
        {
          "collection": false,
          "field": "exact_result_type",
          "full_type": "TypeSchemaRef",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "dependency_slots",
          "full_type": "Vec<ValueSlotId>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RootOutputProducerEvidence",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::RootRecoveryAction": {
      "fields": [
        {
          "collection": false,
          "field": "source",
          "full_type": "RecoveryActionSourceIdentity",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "forward_seed_upper",
          "full_type": "OccurrenceCeiling",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "target",
          "full_type": "ResolvedTarget",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "owner",
          "full_type": "RootRecoveryOwner",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "local_bound",
          "full_type": "Option<LocalActionBound>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "root_bound",
          "full_type": "RootActionBound",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "forward_occurrence",
          "full_type": "ScopedEffectOccurrence",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "action_effect_occurrence",
          "full_type": "ScopedEffectOccurrence",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "recovery_usage_binding",
          "full_type": "RecoveryUsageBinding",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "local_recovery_usage",
          "full_type": "Option<StaticUsageEnvelope>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "root_recovery_usage",
          "full_type": "StaticUsageEnvelope",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "guard_compatibility",
          "full_type": "Option<ActionGuardCompatibility>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": true,
          "field": "guard_coordinates",
          "full_type": "Vec<RootGuardCoordinateEvidence>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "eligibility_evidence",
          "full_type": "LeafVerdict",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "control_path",
          "full_type": "Vec<ControlPathStep>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RootRecoveryAction",
      "source": "crates/af-computation/src/root_recovery.rs"
    },
    "af-computation::RootRecoveryBuilder": {
      "fields": [
        {
          "collection": false,
          "field": "root_digest",
          "full_type": "&'a DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "root_effect_occurrences",
          "full_type": "&'a [ScopedEffectOccurrence]",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "dataflow",
          "full_type": "&'a SemanticDataflowAnalysis",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "context",
          "full_type": "&'context StaticContractProofContext<'universe>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "actions",
          "full_type": "Vec<RootRecoveryAction>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "leaf_sources",
          "full_type": "Vec<RecoverySourceSummary>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "unresolved_sources",
          "full_type": "Vec<ComputationId>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "guard_verdict",
          "full_type": "LeafVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "action_epsilon_verdict",
          "full_type": "LeafVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "factor_verdict",
          "full_type": "LeafVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "binding_verdict",
          "full_type": "LeafVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "usage_profile_verdict",
          "full_type": "LeafVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "arithmetic_violation",
          "full_type": "bool",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RootRecoveryBuilder",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::RootRecoveryCoordinateBinding": {
      "fields": [
        {
          "collection": false,
          "field": "source",
          "full_type": "RecoveryActionSourceIdentity",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "export_action_identity",
          "full_type": "af_contract::RecoveryActionIdentity",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "exact_usage_binding",
          "full_type": "RecoveryUsageBinding",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RootRecoveryCoordinateBinding",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::RootRecoveryFold": {
      "fields": [
        {
          "collection": false,
          "field": "action_effects",
          "full_type": "ControlDimensionValue<EffectSignature>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "epsilon_coverage_requirement",
          "full_type": "ControlDimensionValue<EffectSignature>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "recovery_usage",
          "full_type": "ControlDimensionValue<StaticUsageEnvelope>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "violation",
          "full_type": "bool",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RootRecoveryFold",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::RootRecoveryOwner": {
      "kind": "enum",
      "owner_type": "af-computation::RootRecoveryOwner",
      "source": "crates/af-computation/src/root_recovery.rs",
      "variants": {
        "CompensationScope": [
          {
            "collection": false,
            "field": "0",
            "full_type": "ComputationId",
            "option_presence": "required"
          }
        ],
        "RootRecovery": []
      }
    },
    "af-computation::RootRecoveryPath": {
      "fields": [
        {
          "collection": false,
          "field": "owner_scope",
          "full_type": "Option<ComputationId>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": true,
          "field": "root_factors",
          "full_type": "Vec<AppliedMultiplicity>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "local_factors",
          "full_type": "Vec<AppliedMultiplicity>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "control_path",
          "full_type": "Vec<ControlPathStep>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "root_reachable",
          "full_type": "bool",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "local_reachable",
          "full_type": "bool",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "invalid_factor_identity",
          "full_type": "bool",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RootRecoveryPath",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::RootRecoverySourceClosureReport": {
      "fields": [
        {
          "collection": false,
          "field": "source_nodes_accounted",
          "full_type": "LeafVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "action_bindings_accounted",
          "full_type": "LeafVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "usage_profiles_accounted",
          "full_type": "LeafVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "multiplicity_factors_accounted",
          "full_type": "LeafVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "root_action_totals_sound",
          "full_type": "LeafVerdict",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RootRecoverySourceClosureReport",
      "source": "crates/af-computation/src/root_recovery.rs"
    },
    "af-computation::RootRecoverySummary": {
      "fields": [
        {
          "collection": true,
          "field": "actions",
          "full_type": "ControlDimension<Vec<RootRecoveryAction>>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "action_effects",
          "full_type": "ControlDimension<af_effect::EffectSignature>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "epsilon_coverage_requirement",
          "full_type": "ControlDimension<af_effect::EffectSignature>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "recovery_usage",
          "full_type": "ControlDimension<StaticUsageEnvelope>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_closure",
          "full_type": "RootRecoverySourceClosureReport",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "proof",
          "full_type": "ControlProofReport",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RootRecoverySummary",
      "source": "crates/af-computation/src/root_recovery.rs"
    },
    "af-computation::RootResourcePortBinding": {
      "fields": [
        {
          "collection": false,
          "field": "export_resource_port",
          "full_type": "ResourceRequirementPortIdentity",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "root_resource_slot",
          "full_type": "ResourceSlotId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "root_resource_type",
          "full_type": "TypeSchemaRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "export_resource_type",
          "full_type": "ResourceTypeRef",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "source_coordinates",
          "full_type": "Vec<RootResourceSourceCoordinate>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RootResourcePortBinding",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::RootResourceSourceCoordinate": {
      "fields": [
        {
          "collection": false,
          "field": "source_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_contract_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "component_resource_port",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "resource_slot",
          "full_type": "ResourceSlotId",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "alias_component_ports",
          "full_type": "Vec<u32>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RootResourceSourceCoordinate",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::RuleContentIdentity": {
      "fields": [
        {
          "collection": false,
          "field": "rule_id",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "normative_source",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_anchor",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "short_statement",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::RuleContentIdentity",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::RuntimeAuthorityStatus": {
      "kind": "enum",
      "owner_type": "af-computation::RuntimeAuthorityStatus",
      "source": "crates/af-computation/src/root_export.rs",
      "variants": {
        "NotEstablished": []
      }
    },
    "af-computation::ScopeBoundarySummary": {
      "fields": [
        {
          "collection": false,
          "field": "control_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "child",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "parent_visible_resources",
          "full_type": "Vec<ResourceSlotId>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "declared_resources",
          "full_type": "Vec<ResourceSlotId>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "child_visible_resources",
          "full_type": "Vec<ResourceSlotId>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "resource_attenuation",
          "full_type": "LeafVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "authority_attenuation",
          "full_type": "LeafVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "budget_attenuation",
          "full_type": "LeafVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "owned_children_settle_before_terminal",
          "full_type": "LeafVerdict",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ScopeBoundarySummary",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::ScopedEffectOccurrence": {
      "fields": [
        {
          "collection": false,
          "field": "caller_semantic_ir_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_contract_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_component",
          "full_type": "ComponentIdentity",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "computation",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_entry_ordinal",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "declared",
          "full_type": "EffectEntry",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "mapped_resource_slot",
          "full_type": "Option<ResourceSlotId>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": true,
          "field": "control_families",
          "full_type": "Vec<EffectOccurrenceFamily>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ScopedEffectOccurrence",
      "source": "crates/af-computation/src/summary.rs"
    },
    "af-computation::ScopedEffectResidual": {
      "fields": [
        {
          "collection": false,
          "field": "caller_semantic_ir_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_contract_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_component",
          "full_type": "ComponentIdentity",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "computation",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_entry_ordinal",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "residual",
          "full_type": "EffectResidual",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "mapped_resource_slot",
          "full_type": "Option<ResourceSlotId>",
          "option_presence": "explicit_none_or_some"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ScopedEffectResidual",
      "source": "crates/af-computation/src/summary.rs"
    },
    "af-computation::ScopedPortBinding": {
      "kind": "enum",
      "owner_type": "af-computation::ScopedPortBinding",
      "source": "crates/af-computation/src/summary.rs",
      "variants": {
        "InputExpression": [
          {
            "collection": false,
            "field": "0",
            "full_type": "ExpressionId",
            "option_presence": "required"
          }
        ],
        "OutputValue": [
          {
            "collection": false,
            "field": "0",
            "full_type": "ValueSlotId",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-computation::ScopedPortMapping": {
      "fields": [
        {
          "collection": false,
          "field": "contract_port_no",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "binding",
          "full_type": "ScopedPortBinding",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "exact_type",
          "full_type": "Option<TypeSchemaRef>",
          "option_presence": "explicit_none_or_some"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ScopedPortMapping",
      "source": "crates/af-computation/src/summary.rs"
    },
    "af-computation::ScopedResourcePortMapping": {
      "fields": [
        {
          "collection": false,
          "field": "component_port_no",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "resource_slot",
          "full_type": "ResourceSlotId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "required_type",
          "full_type": "TypeSchemaRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "required_attributes_proven",
          "full_type": "bool",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::ScopedResourcePortMapping",
      "source": "crates/af-computation/src/summary.rs"
    },
    "af-computation::SelectDecisionEvidence": {
      "fields": [
        {
          "collection": false,
          "field": "expression",
          "full_type": "ExpressionId",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "dependency_slots",
          "full_type": "Vec<ValueSlotId>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "case_ordinals",
          "full_type": "Vec<u32>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "exact_case",
          "full_type": "Option<u32>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "totality_verdict",
          "full_type": "LeafVerdict",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::SelectDecisionEvidence",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::SequenceDataflowDependency": {
      "fields": [
        {
          "collection": false,
          "field": "producer_child",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "producer_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "value_slot",
          "full_type": "ValueSlotId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "consumer_child",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "consumer_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "expression",
          "full_type": "ExpressionId",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::SequenceDataflowDependency",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-computation::SnapshotError": {
      "kind": "enum",
      "owner_type": "af-computation::SnapshotError",
      "source": "crates/af-computation/src/resolution.rs",
      "variants": {
        "ContractDigest": [
          {
            "collection": false,
            "field": "0",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "DuplicateComponentPin": [
          {
            "collection": false,
            "field": "0",
            "full_type": "DigestRefValue",
            "option_presence": "required"
          }
        ],
        "DuplicateSubflowExport": [
          {
            "collection": false,
            "field": "0",
            "full_type": "DigestRefValue",
            "option_presence": "required"
          }
        ],
        "InvalidSubflowDigest": [
          {
            "collection": false,
            "field": "0",
            "full_type": "DigestRefValue",
            "option_presence": "required"
          }
        ],
        "ProofContextMismatch": [],
        "UnknownSubflowTarget": [
          {
            "collection": false,
            "field": "0",
            "full_type": "DigestRefValue",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-computation::SpecificationContentIdentity": {
      "fields": [
        {
          "collection": false,
          "field": "specification_id",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "logical_path",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "content_sha256",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "content",
          "full_type": "Vec<u8>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::SpecificationContentIdentity",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::StaticComputationResolutionSnapshot": {
      "fields": [
        {
          "collection": false,
          "field": "proof_context",
          "full_type": "&'context StaticContractProofContext<'universe>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "subflows",
          "full_type": "&'a SubflowIrUniverse",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "components",
          "full_type": "HashMap<DigestRefValue, &'a CheckedComponentContract<'context, 'universe>>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "exports",
          "full_type": "HashMap<DigestRefValue, ExportedContract<'a, 'context, 'universe>>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::StaticComputationResolutionSnapshot",
      "source": "crates/af-computation/src/resolution.rs"
    },
    "af-computation::StaticComputationResolutionSnapshotBuilder": {
      "fields": [
        {
          "collection": false,
          "field": "proof_context",
          "full_type": "&'context StaticContractProofContext<'universe>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "subflows",
          "full_type": "&'a SubflowIrUniverse",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "components",
          "full_type": "HashMap<DigestRefValue, &'a CheckedComponentContract<'context, 'universe>>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "exports",
          "full_type": "HashMap<DigestRefValue, ExportedContract<'a, 'context, 'universe>>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::StaticComputationResolutionSnapshotBuilder",
      "source": "crates/af-computation/src/resolution.rs"
    },
    "af-computation::SubflowProofDependency": {
      "fields": [
        {
          "collection": false,
          "field": "call_site",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "target_semantic_ir_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "target_exported_contract_digest",
          "full_type": "Option<DigestRefValue>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "target_root_assertion_identity",
          "full_type": "Option<Box<RootExportAssertionIdentity>>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": true,
          "field": "transitive_targets",
          "full_type": "Vec<SubflowTargetIdentity>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::SubflowProofDependency",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::SubflowTargetContextPin": {
      "fields": [
        {
          "collection": false,
          "field": "semantic_ir_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "direct_dependencies",
          "full_type": "Vec<DigestRefValue>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "exported_contract_digest",
          "full_type": "Option<DigestRefValue>",
          "option_presence": "explicit_none_or_some"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::SubflowTargetContextPin",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::SubflowTargetIdentity": {
      "fields": [
        {
          "collection": false,
          "field": "semantic_ir_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "exported_contract_digest",
          "full_type": "Option<DigestRefValue>",
          "option_presence": "explicit_none_or_some"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::SubflowTargetIdentity",
      "source": "crates/af-computation/src/root_export.rs"
    },
    "af-computation::TimeoutOutcome": {
      "kind": "enum",
      "owner_type": "af-computation::TimeoutOutcome",
      "source": "crates/af-computation/src/control.rs",
      "variants": {
        "ChildCompleted": [],
        "TimedOut": []
      }
    },
    "af-computation::UsageDimensionResult": {
      "fields": [
        {
          "collection": false,
          "field": "dimension",
          "full_type": "ControlDimension<StaticUsageEnvelope>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "proofs",
          "full_type": "Vec<ControlDimensionProof>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-computation::UsageDimensionResult",
      "source": "crates/af-computation/src/control.rs"
    },
    "af-contract::ActionGuardCompatibility": {
      "fields": [
        {
          "collection": false,
          "field": "source_component",
          "full_type": "ComponentIdentity",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "recovery_action",
          "full_type": "RecoveryActionIdentity",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "forward_entry_ordinal",
          "full_type": "Option<usize>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "action_entry_ordinal",
          "full_type": "Option<usize>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "forward_guard",
          "full_type": "Option<EffectGuard>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "declared_action_guard",
          "full_type": "Option<EffectGuard>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "action_upper",
          "full_type": "Option<OccurrenceCeiling>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "verdict",
          "full_type": "ProofVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "basis",
          "full_type": "GuardCompatibilityBasis",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "countermodel",
          "full_type": "Option<TypedGuardCountermodel>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "reason",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::ActionGuardCompatibility",
      "source": "crates/af-contract/src/recovery_guard.rs"
    },
    "af-contract::AuthorityExecutionRule": {
      "fields": [
        {
          "collection": false,
          "field": "authority",
          "full_type": "af_flow::InstructionAuthority",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "trust_domains",
          "full_type": "TrustDomainPolicy",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "sandbox",
          "full_type": "SandboxFloor",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::AuthorityExecutionRule",
      "source": "crates/af-contract/src/universe.rs"
    },
    "af-contract::CanonicalComponentContract": {
      "fields": [
        {
          "collection": false,
          "field": "contract",
          "full_type": "ComponentContract",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::CanonicalComponentContract",
      "source": "crates/af-contract/src/canonical.rs"
    },
    "af-contract::CatalogContentChecksum": {
      "fields": [
        {
          "collection": false,
          "field": "algorithm",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "hex",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::CatalogContentChecksum",
      "source": "crates/af-contract/src/proof.rs"
    },
    "af-contract::CheckedComponentContract": {
      "fields": [
        {
          "collection": false,
          "field": "contract",
          "full_type": "ComponentContract",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "proof",
          "full_type": "ProofReport",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "context",
          "full_type": "&'context crate::validate::StaticContractProofContext<'universe>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::CheckedComponentContract",
      "source": "crates/af-contract/src/proof.rs"
    },
    "af-contract::CompatibilityReport": {
      "fields": [
        {
          "collection": true,
          "field": "dimensions",
          "full_type": "Vec<DimensionProof>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::CompatibilityReport",
      "source": "crates/af-contract/src/compatibility.rs"
    },
    "af-contract::ComponentContract": {
      "fields": [
        {
          "collection": false,
          "field": "component_identity",
          "full_type": "ComponentIdentity",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "inputs",
          "full_type": "Vec<ComponentInputPort>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "outputs",
          "full_type": "Vec<ComponentOutputPort>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "failures",
          "full_type": "FailureRow",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "effects",
          "full_type": "af_effect::EffectSignature",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "resources",
          "full_type": "ResourceRequirementRow",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "information_flow",
          "full_type": "InformationFlowContract",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "usage",
          "full_type": "UsageSignature",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "recovery",
          "full_type": "RecoveryContract",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "execution",
          "full_type": "ExecutionRequirements",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "effect_flow_partitions",
          "full_type": "Vec<EffectFlowPartition>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "recovery_action_bindings",
          "full_type": "Vec<RecoveryActionBinding>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "recovery_usage_bindings",
          "full_type": "Vec<RecoveryUsageBinding>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::ComponentContract",
      "source": "crates/af-contract/src/model.rs"
    },
    "af-contract::ComponentIdentity": {
      "fields": [
        {
          "collection": false,
          "field": "catalog",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::ComponentIdentity",
      "source": "crates/af-contract/src/model.rs"
    },
    "af-contract::ComponentInputPort": {
      "fields": [
        {
          "collection": false,
          "field": "port_no",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "type_ref",
          "full_type": "TypeSchemaRef",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::ComponentInputPort",
      "source": "crates/af-contract/src/model.rs"
    },
    "af-contract::ComponentOutputPort": {
      "fields": [
        {
          "collection": false,
          "field": "port_no",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "type_ref",
          "full_type": "TypeSchemaRef",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::ComponentOutputPort",
      "source": "crates/af-contract/src/model.rs"
    },
    "af-contract::ConfidentialityExecutionRule": {
      "fields": [
        {
          "collection": false,
          "field": "label",
          "full_type": "ConfidentialityLabelRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "region_pairs",
          "full_type": "RegionPairPolicy",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::ConfidentialityExecutionRule",
      "source": "crates/af-contract/src/universe.rs"
    },
    "af-contract::ContractError": {
      "fields": [
        {
          "collection": false,
          "field": "rule_id",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "code",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "reference",
          "full_type": "Option<String>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "detail",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::ContractError",
      "source": "crates/af-contract/src/error.rs"
    },
    "af-contract::DimensionProof": {
      "fields": [
        {
          "collection": false,
          "field": "dimension",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "verdict",
          "full_type": "ProofVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "rule_id",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "reference",
          "full_type": "Option<String>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "reason",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "prerequisites",
          "full_type": "Vec<String>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::DimensionProof",
      "source": "crates/af-contract/src/proof.rs"
    },
    "af-contract::EffectFlowPartition": {
      "fields": [
        {
          "collection": false,
          "field": "operation",
          "full_type": "EffectOperationRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "kind",
          "full_type": "FlowKind",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "allowed_effects",
          "full_type": "Vec<EffectAtom>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::EffectFlowPartition",
      "source": "crates/af-contract/src/model.rs"
    },
    "af-contract::EffectQualifierProfile": {
      "fields": [
        {
          "collection": true,
          "field": "qualifiers",
          "full_type": "Vec<EffectQualifier>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::EffectQualifierProfile",
      "source": "crates/af-contract/src/universe.rs"
    },
    "af-contract::EffectResourceCompatibilityCatalog": {
      "fields": [
        {
          "collection": true,
          "field": "complete_operations",
          "full_type": "Vec<EffectOperationRef>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "supported_pairs",
          "full_type": "Vec<SupportedEffectResourcePair>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::EffectResourceCompatibilityCatalog",
      "source": "crates/af-contract/src/universe.rs"
    },
    "af-contract::FlowExecutionRule": {
      "fields": [
        {
          "collection": false,
          "field": "kind",
          "full_type": "FlowKind",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "sink",
          "full_type": "FlowExecutionSink",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "trust_domains",
          "full_type": "TrustDomainPolicy",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "sandbox",
          "full_type": "SandboxFloor",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::FlowExecutionRule",
      "source": "crates/af-contract/src/universe.rs"
    },
    "af-contract::FlowExecutionSink": {
      "kind": "enum",
      "owner_type": "af-contract::FlowExecutionSink",
      "source": "crates/af-contract/src/universe.rs",
      "variants": {
        "ExactEffect": [
          {
            "collection": false,
            "field": "0",
            "full_type": "EffectAtom",
            "option_presence": "required"
          }
        ],
        "WorkflowOutput": [
          {
            "collection": false,
            "field": "0",
            "full_type": "u32",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-contract::GuardCompatibilityBasis": {
      "kind": "enum",
      "owner_type": "af-contract::GuardCompatibilityBasis",
      "source": "crates/af-contract/src/recovery_guard.rs",
      "variants": {
        "FrozenGuardContainment": [],
        "InsufficientEvidence": [],
        "TypedCountermodel": [],
        "ZeroPossibleActions": []
      }
    },
    "af-contract::GuardCountermodelReachability": {
      "kind": "enum",
      "owner_type": "af-contract::GuardCountermodelReachability",
      "source": "crates/af-contract/src/recovery_guard.rs",
      "variants": {
        "WellTypedComponentInputDomain": []
      }
    },
    "af-contract::GuardInputAssignment": {
      "fields": [
        {
          "collection": false,
          "field": "port_no",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "type_ref",
          "full_type": "TypeSchemaRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "value",
          "full_type": "PrimitiveConstant",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::GuardInputAssignment",
      "source": "crates/af-contract/src/recovery_guard.rs"
    },
    "af-contract::GuardPortValidationError": {
      "kind": "enum",
      "owner_type": "af-contract::GuardPortValidationError",
      "source": "crates/af-contract/src/recovery_guard.rs",
      "variants": {
        "MissingPort": [],
        "TypeMismatch": [],
        "UnresolvedSchema": []
      }
    },
    "af-contract::InformationFlowExecutionCatalog": {
      "fields": [
        {
          "collection": true,
          "field": "closed_regions",
          "full_type": "Vec<RegionRef>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "closed_trust_domains",
          "full_type": "Vec<TrustDomainRef>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "confidentiality_rules",
          "full_type": "Vec<ConfidentialityExecutionRule>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "taint_rules",
          "full_type": "Vec<TaintExecutionRule>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "authority_rules",
          "full_type": "Vec<AuthorityExecutionRule>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "sink_rules",
          "full_type": "Vec<FlowExecutionRule>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::InformationFlowExecutionCatalog",
      "source": "crates/af-contract/src/universe.rs"
    },
    "af-contract::ProofReport": {
      "fields": [
        {
          "collection": true,
          "field": "dimensions",
          "full_type": "Vec<DimensionProof>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "relation_catalogs",
          "full_type": "Vec<RelationCatalogPin>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "action_guard_compatibility",
          "full_type": "Vec<crate::recovery_guard::ActionGuardCompatibility>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::ProofReport",
      "source": "crates/af-contract/src/proof.rs"
    },
    "af-contract::ProofVerdict": {
      "kind": "enum",
      "owner_type": "af-contract::ProofVerdict",
      "source": "crates/af-contract/src/proof.rs",
      "variants": {
        "Proven": [],
        "RequiresEvidence": [],
        "Unproven": [],
        "Violation": []
      }
    },
    "af-contract::RecoveryActionBinding": {
      "fields": [
        {
          "collection": false,
          "field": "identity",
          "full_type": "RecoveryActionIdentity",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::RecoveryActionBinding",
      "source": "crates/af-contract/src/model.rs"
    },
    "af-contract::RecoveryActionIdentity": {
      "fields": [
        {
          "collection": false,
          "field": "forward_effect",
          "full_type": "EffectAtom",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "action_kind",
          "full_type": "RecoveryActionKind",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "action_effect",
          "full_type": "EffectAtom",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::RecoveryActionIdentity",
      "source": "crates/af-contract/src/model.rs"
    },
    "af-contract::RecoveryActionKind": {
      "kind": "enum",
      "owner_type": "af-contract::RecoveryActionKind",
      "source": "crates/af-contract/src/model.rs",
      "variants": {
        "Compensation": [],
        "Reconciliation": []
      }
    },
    "af-contract::RecoveryUsageBinding": {
      "fields": [
        {
          "collection": false,
          "field": "action",
          "full_type": "RecoveryActionIdentity",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "activations_per_forward",
          "full_type": "OccurrenceBounds",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "attempts_per_activation",
          "full_type": "OccurrenceBounds",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "per_attempt_usage",
          "full_type": "UsageSignature",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::RecoveryUsageBinding",
      "source": "crates/af-contract/src/model.rs"
    },
    "af-contract::RecoveryUsageCoverageCatalog": {
      "fields": [
        {
          "collection": true,
          "field": "rules",
          "full_type": "Vec<RecoveryUsageCoverageRule>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::RecoveryUsageCoverageCatalog",
      "source": "crates/af-contract/src/universe.rs"
    },
    "af-contract::RecoveryUsageCoverageRule": {
      "fields": [
        {
          "collection": false,
          "field": "action",
          "full_type": "RecoveryActionIdentity",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "complete_meters",
          "full_type": "Vec<UsageMeterRef>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::RecoveryUsageCoverageRule",
      "source": "crates/af-contract/src/universe.rs"
    },
    "af-contract::RegionPair": {
      "fields": [
        {
          "collection": false,
          "field": "execution_region",
          "full_type": "RegionRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "residency_region",
          "full_type": "RegionRef",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::RegionPair",
      "source": "crates/af-contract/src/universe.rs"
    },
    "af-contract::RegionPairPolicy": {
      "kind": "enum",
      "owner_type": "af-contract::RegionPairPolicy",
      "source": "crates/af-contract/src/universe.rs",
      "variants": {
        "AllPairsInClosedRegionSnapshot": [],
        "FiniteExactPairs": [
          {
            "collection": true,
            "field": "0",
            "full_type": "Vec<RegionPair>",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-contract::RelationCatalogPin": {
      "fields": [
        {
          "collection": false,
          "field": "schema",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "revision",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "canonical_content_checksum",
          "full_type": "CatalogContentChecksum",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::RelationCatalogPin",
      "source": "crates/af-contract/src/proof.rs"
    },
    "af-contract::RelationCatalogSnapshot": {
      "fields": [
        {
          "collection": false,
          "field": "schema",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "revision",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "canonical_bytes",
          "full_type": "Vec<u8>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "catalog",
          "full_type": "T",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "pin",
          "full_type": "RelationCatalogPin",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::RelationCatalogSnapshot",
      "source": "crates/af-contract/src/universe.rs"
    },
    "af-contract::RelationSnapshots": {
      "fields": [
        {
          "collection": false,
          "field": "effect_resource",
          "full_type": "Option<RelationCatalogSnapshot<EffectResourceCompatibilityCatalog>>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "recovery_usage",
          "full_type": "Option<RelationCatalogSnapshot<RecoveryUsageCoverageCatalog>>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "flow_execution",
          "full_type": "Option<RelationCatalogSnapshot<InformationFlowExecutionCatalog>>",
          "option_presence": "explicit_none_or_some"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::RelationSnapshots",
      "source": "crates/af-contract/src/universe.rs"
    },
    "af-contract::SandboxFloor": {
      "kind": "enum",
      "owner_type": "af-contract::SandboxFloor",
      "source": "crates/af-contract/src/universe.rs",
      "variants": {
        "Minimum": [
          {
            "collection": false,
            "field": "0",
            "full_type": "SandboxLevel",
            "option_presence": "required"
          }
        ],
        "NoAdditionalMinimum": []
      }
    },
    "af-contract::ScopeValueRule": {
      "kind": "enum",
      "owner_type": "af-contract::ScopeValueRule",
      "source": "crates/af-contract/src/universe.rs",
      "variants": {
        "FiniteExactValues": [
          {
            "collection": true,
            "field": "0",
            "full_type": "Vec<String>",
            "option_presence": "required"
          }
        ],
        "OpaqueNfcExactString": []
      }
    },
    "af-contract::SelectorKeyRule": {
      "fields": [
        {
          "collection": false,
          "field": "selector_key",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "resource_attribute_key",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "required",
          "full_type": "bool",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "values",
          "full_type": "ScopeValueRule",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::SelectorKeyRule",
      "source": "crates/af-contract/src/universe.rs"
    },
    "af-contract::StaticContractProofContext": {
      "fields": [
        {
          "collection": false,
          "field": "modules",
          "full_type": "&'a CompiledModuleSet",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "schema_universe",
          "full_type": "&'a af_semantic::TypeSchemaUniverse",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "effects",
          "full_type": "&'a af_effect::EffectCatalogUniverse",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "resources",
          "full_type": "&'a af_resource::ResourceTypeUniverse",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "flow",
          "full_type": "&'a af_flow::FlowCatalogs",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "usage_meters",
          "full_type": "&'a af_usage::UsageMeterUniverse",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "recovery",
          "full_type": "&'a af_recovery::RecoveryOperationUniverse",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "execution",
          "full_type": "&'a af_execution::ExecutionUniverses",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "relations",
          "full_type": "&'a RelationSnapshots",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::StaticContractProofContext",
      "source": "crates/af-contract/src/validate.rs"
    },
    "af-contract::SupportedEffectResourcePair": {
      "fields": [
        {
          "collection": false,
          "field": "operation",
          "full_type": "EffectOperationRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "resource_type",
          "full_type": "ResourceTypeRef",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "selector_keys",
          "full_type": "Vec<SelectorKeyRule>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "allowed_qualifier_profiles",
          "full_type": "Vec<EffectQualifierProfile>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::SupportedEffectResourcePair",
      "source": "crates/af-contract/src/universe.rs"
    },
    "af-contract::TaintExecutionRule": {
      "fields": [
        {
          "collection": false,
          "field": "taint",
          "full_type": "TaintRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "trust_domains",
          "full_type": "TrustDomainPolicy",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "sandbox",
          "full_type": "SandboxFloor",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::TaintExecutionRule",
      "source": "crates/af-contract/src/universe.rs"
    },
    "af-contract::TrustDomainPolicy": {
      "kind": "enum",
      "owner_type": "af-contract::TrustDomainPolicy",
      "source": "crates/af-contract/src/universe.rs",
      "variants": {
        "AllDomainsInClosedTrustSnapshot": [],
        "ExactAllowlist": [
          {
            "collection": true,
            "field": "0",
            "full_type": "Vec<TrustDomainRef>",
            "option_presence": "required"
          }
        ],
        "NoAdditionalTrustRequirement": []
      }
    },
    "af-contract::TypedGuardCountermodel": {
      "fields": [
        {
          "collection": false,
          "field": "source_contract",
          "full_type": "ComponentContract",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_component",
          "full_type": "ComponentIdentity",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "recovery_action",
          "full_type": "RecoveryActionIdentity",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "forward_entry_ordinal",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "action_entry_ordinal",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "forward_guard",
          "full_type": "EffectGuard",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "declared_action_guard",
          "full_type": "EffectGuard",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "forward_bounds",
          "full_type": "OccurrenceBounds",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "activation_upper",
          "full_type": "OccurrenceCeiling",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "attempt_upper",
          "full_type": "OccurrenceCeiling",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "action_upper",
          "full_type": "OccurrenceCeiling",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "assignments",
          "full_type": "Vec<GuardInputAssignment>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "reachability",
          "full_type": "GuardCountermodelReachability",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "related_rule_id",
          "full_type": "&'static str",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-contract::TypedGuardCountermodel",
      "source": "crates/af-contract/src/recovery_guard.rs"
    },
    "af-core::AceError": {
      "kind": "enum",
      "owner_type": "af-core::AceError",
      "source": "crates/af-core/src/error.rs",
      "variants": {
        "DuplicateField": [
          {
            "collection": false,
            "field": "field",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "DuplicateNormalizedKey": [
          {
            "collection": false,
            "field": "key",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "IntegerOutOfRange": [
          {
            "collection": false,
            "field": "input",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "type_name",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "InvalidBytesEncoding": [
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "InvalidDecimal": [
          {
            "collection": false,
            "field": "input",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "InvalidInteger": [
          {
            "collection": false,
            "field": "input",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "InvalidOptionEncoding": [
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "InvalidShape": [
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "InvalidUnicode": [
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "MissingField": [
          {
            "collection": false,
            "field": "field",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "NonCanonicalEncoding": [
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "NonFiniteFloat": [],
        "TypeMismatch": [
          {
            "collection": false,
            "field": "expected",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "actual",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "UnknownEnumValue": [
          {
            "collection": false,
            "field": "value",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "UnknownField": [
          {
            "collection": false,
            "field": "field",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "UnknownVariantCase": [
          {
            "collection": false,
            "field": "case",
            "full_type": "String",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-core::CaseShape": {
      "fields": [
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "payload",
          "full_type": "Shape",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-core::CaseShape",
      "source": "crates/af-core/src/shape.rs"
    },
    "af-core::Decimal": {
      "fields": [
        {
          "collection": false,
          "field": "0",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-core::Decimal",
      "source": "crates/af-core/src/decimal.rs"
    },
    "af-core::DigestAlgorithm": {
      "kind": "enum",
      "owner_type": "af-core::DigestAlgorithm",
      "source": "crates/af-core/src/digest.rs",
      "variants": {
        "Sha256": []
      }
    },
    "af-core::DigestDomain": {
      "kind": "enum",
      "owner_type": "af-core::DigestDomain",
      "source": "crates/af-core/src/digest.rs",
      "variants": {
        "Artifact": [],
        "ComponentContract": [],
        "Conformance": [],
        "JournalEventPayload": [],
        "LockGraph": [],
        "PlanCore": [],
        "Provenance": [],
        "ResourceContract": [],
        "RunManifest": [],
        "RuntimeCommandPayload": [],
        "SemanticExecutionState": [],
        "SemanticIr": [],
        "TypeSchema": [],
        "Value": []
      }
    },
    "af-core::DigestError": {
      "kind": "enum",
      "owner_type": "af-core::DigestError",
      "source": "crates/af-core/src/error.rs",
      "variants": {
        "InvalidDigest": [
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "UnknownDigestDomain": [
          {
            "collection": false,
            "field": "domain",
            "full_type": "String",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-core::DigestRef": {
      "fields": [
        {
          "collection": false,
          "field": "alg",
          "full_type": "DigestAlgorithm",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "domain",
          "full_type": "DigestDomain",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "hex",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-core::DigestRef",
      "source": "crates/af-core/src/digest.rs"
    },
    "af-core::Duration": {
      "fields": [
        {
          "collection": false,
          "field": "0",
          "full_type": "i64",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-core::Duration",
      "source": "crates/af-core/src/value.rs"
    },
    "af-core::EnumShape": {
      "fields": [
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "values",
          "full_type": "Vec<String>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-core::EnumShape",
      "source": "crates/af-core/src/shape.rs"
    },
    "af-core::FieldShape": {
      "fields": [
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "shape",
          "full_type": "Shape",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-core::FieldShape",
      "source": "crates/af-core/src/shape.rs"
    },
    "af-core::Float64": {
      "fields": [
        {
          "collection": false,
          "field": "0",
          "full_type": "f64",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-core::Float64",
      "source": "crates/af-core/src/value.rs"
    },
    "af-core::Member": {
      "fields": [
        {
          "collection": false,
          "field": "key",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "value",
          "full_type": "(usize, usize)",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-core::Member",
      "source": "crates/af-core/src/decode.rs"
    },
    "af-core::Parser": {
      "fields": [
        {
          "collection": false,
          "field": "input",
          "full_type": "&'a str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "pos",
          "full_type": "usize",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-core::Parser",
      "source": "crates/af-core/src/decode.rs"
    },
    "af-core::RecordShape": {
      "fields": [
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "fields",
          "full_type": "Vec<FieldShape>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-core::RecordShape",
      "source": "crates/af-core/src/shape.rs"
    },
    "af-core::Shape": {
      "kind": "enum",
      "owner_type": "af-core::Shape",
      "source": "crates/af-core/src/shape.rs",
      "variants": {
        "Bool": [],
        "Bytes": [],
        "Decimal": [],
        "Duration": [],
        "Enum": [
          {
            "collection": false,
            "field": "0",
            "full_type": "EnumShape",
            "option_presence": "required"
          }
        ],
        "Float64": [],
        "Int32": [],
        "Int64": [],
        "List": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Box<Shape>",
            "option_presence": "required"
          }
        ],
        "Map": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Box<Shape>",
            "option_presence": "required"
          }
        ],
        "Null": [],
        "Option": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Box<Shape>",
            "option_presence": "required"
          }
        ],
        "Record": [
          {
            "collection": false,
            "field": "0",
            "full_type": "RecordShape",
            "option_presence": "required"
          }
        ],
        "String": [],
        "Timestamp": [],
        "UInt32": [],
        "UInt64": [],
        "Unit": [],
        "Variant": [
          {
            "collection": false,
            "field": "0",
            "full_type": "VariantShape",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-core::Timestamp": {
      "fields": [
        {
          "collection": false,
          "field": "0",
          "full_type": "i64",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-core::Timestamp",
      "source": "crates/af-core/src/value.rs"
    },
    "af-core::Value": {
      "kind": "enum",
      "owner_type": "af-core::Value",
      "source": "crates/af-core/src/value.rs",
      "variants": {
        "Bool": [
          {
            "collection": false,
            "field": "0",
            "full_type": "bool",
            "option_presence": "required"
          }
        ],
        "Bytes": [
          {
            "collection": true,
            "field": "0",
            "full_type": "Vec<u8>",
            "option_presence": "required"
          }
        ],
        "Decimal": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Decimal",
            "option_presence": "required"
          }
        ],
        "Duration": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Duration",
            "option_presence": "required"
          }
        ],
        "Enum": [
          {
            "collection": false,
            "field": "0",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "Float64": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Float64",
            "option_presence": "required"
          }
        ],
        "Int32": [
          {
            "collection": false,
            "field": "0",
            "full_type": "i32",
            "option_presence": "required"
          }
        ],
        "Int64": [
          {
            "collection": false,
            "field": "0",
            "full_type": "i64",
            "option_presence": "required"
          }
        ],
        "List": [
          {
            "collection": true,
            "field": "0",
            "full_type": "Vec<Value>",
            "option_presence": "required"
          }
        ],
        "Map": [
          {
            "collection": true,
            "field": "0",
            "full_type": "Vec<(String, Value)>",
            "option_presence": "required"
          }
        ],
        "Null": [],
        "Option": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Option<Box<Value>>",
            "option_presence": "explicit_none_or_some"
          }
        ],
        "Record": [
          {
            "collection": true,
            "field": "0",
            "full_type": "Vec<(String, Value)>",
            "option_presence": "required"
          }
        ],
        "String": [
          {
            "collection": false,
            "field": "0",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "Timestamp": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Timestamp",
            "option_presence": "required"
          }
        ],
        "UInt32": [
          {
            "collection": false,
            "field": "0",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "UInt64": [
          {
            "collection": false,
            "field": "0",
            "full_type": "u64",
            "option_presence": "required"
          }
        ],
        "Unit": [],
        "Variant": [
          {
            "collection": false,
            "field": "case",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "value",
            "full_type": "Box<Value>",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-core::VariantShape": {
      "fields": [
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "cases",
          "full_type": "Vec<CaseShape>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-core::VariantShape",
      "source": "crates/af-core/src/shape.rs"
    },
    "af-core::value": {
      "kind": "enum",
      "owner_type": "af-core::value",
      "source": "crates/af-core/src/error.rs",
      "variants": {
        "value": []
      }
    },
    "af-effect::AtomKey": {
      "fields": [
        {
          "collection": false,
          "field": "catalog",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "port_no",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "scope",
          "full_type": "Vec<(String, String)>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "qualifiers",
          "full_type": "Vec<EffectQualifierIdentity>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-effect::AtomKey",
      "source": "crates/af-effect/src/model.rs"
    },
    "af-effect::CapabilityCatalog": {
      "fields": [
        {
          "collection": false,
          "field": "specification_id",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "operations",
          "full_type": "BTreeSet<String>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-effect::CapabilityCatalog",
      "source": "crates/af-effect/src/catalog.rs"
    },
    "af-effect::CoverKey": {
      "fields": [
        {
          "collection": false,
          "field": "catalog",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "port_no",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "qualifiers",
          "full_type": "Vec<EffectQualifierIdentity>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-effect::CoverKey",
      "source": "crates/af-effect/src/algebra.rs"
    },
    "af-effect::EffectAtom": {
      "fields": [
        {
          "collection": false,
          "field": "operation",
          "full_type": "EffectOperationRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "resource_selector",
          "full_type": "ResourceSelector",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "qualifiers",
          "full_type": "Vec<EffectQualifier>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-effect::EffectAtom",
      "source": "crates/af-effect/src/model.rs"
    },
    "af-effect::EffectCatalogUniverse": {
      "fields": [
        {
          "collection": true,
          "field": "catalogs",
          "full_type": "BTreeMap<String, CapabilityCatalog>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-effect::EffectCatalogUniverse",
      "source": "crates/af-effect/src/catalog.rs"
    },
    "af-effect::EffectEntry": {
      "fields": [
        {
          "collection": false,
          "field": "atom",
          "full_type": "EffectAtom",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "bounds",
          "full_type": "OccurrenceBounds",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "guard",
          "full_type": "EffectGuard",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-effect::EffectEntry",
      "source": "crates/af-effect/src/model.rs"
    },
    "af-effect::EffectError": {
      "fields": [
        {
          "collection": false,
          "field": "kind",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "detail",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-effect::EffectError",
      "source": "crates/af-effect/src/error.rs"
    },
    "af-effect::EffectGuard": {
      "kind": "enum",
      "owner_type": "af-effect::EffectGuard",
      "source": "crates/af-effect/src/model.rs",
      "variants": {
        "InputEquals": [
          {
            "collection": false,
            "field": "0",
            "full_type": "EffectGuardInputEquals",
            "option_presence": "required"
          }
        ],
        "Unconditional": []
      }
    },
    "af-effect::EffectGuardInputEquals": {
      "fields": [
        {
          "collection": false,
          "field": "port_no",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "expected",
          "full_type": "PrimitiveConstant",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-effect::EffectGuardInputEquals",
      "source": "crates/af-effect/src/model.rs"
    },
    "af-effect::EffectIdempotencyQualifier": {
      "kind": "enum",
      "owner_type": "af-effect::EffectIdempotencyQualifier",
      "source": "crates/af-effect/src/model.rs",
      "variants": {
        "Natural": [],
        "None": []
      }
    },
    "af-effect::EffectOpenRow": {
      "fields": [
        {
          "collection": true,
          "field": "known",
          "full_type": "Vec<EffectEntry>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "residual",
          "full_type": "Vec<EffectResidual>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-effect::EffectOpenRow",
      "source": "crates/af-effect/src/model.rs"
    },
    "af-effect::EffectOperationRef": {
      "fields": [
        {
          "collection": false,
          "field": "catalog",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-effect::EffectOperationRef",
      "source": "crates/af-effect/src/model.rs"
    },
    "af-effect::EffectQualifier": {
      "kind": "enum",
      "owner_type": "af-effect::EffectQualifier",
      "source": "crates/af-effect/src/model.rs",
      "variants": {
        "Idempotency": [
          {
            "collection": false,
            "field": "0",
            "full_type": "EffectIdempotencyQualifier",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-effect::EffectQualifierIdentity": {
      "fields": [
        {
          "collection": false,
          "field": "family_no",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "member_no",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "qualifier",
          "full_type": "EffectQualifier",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-effect::EffectQualifierIdentity",
      "source": "crates/af-effect/src/model.rs"
    },
    "af-effect::EffectResidual": {
      "fields": [
        {
          "collection": false,
          "field": "catalog",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "selector",
          "full_type": "ResourceSelector",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "bounds",
          "full_type": "OccurrenceBounds",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-effect::EffectResidual",
      "source": "crates/af-effect/src/model.rs"
    },
    "af-effect::EffectRow": {
      "kind": "enum",
      "owner_type": "af-effect::EffectRow",
      "source": "crates/af-effect/src/model.rs",
      "variants": {
        "Closed": [
          {
            "collection": true,
            "field": "0",
            "full_type": "Vec<EffectEntry>",
            "option_presence": "required"
          }
        ],
        "Open": [
          {
            "collection": false,
            "field": "0",
            "full_type": "EffectOpenRow",
            "option_presence": "required"
          }
        ],
        "Unknown": []
      }
    },
    "af-effect::EffectSignature": {
      "fields": [
        {
          "collection": false,
          "field": "row",
          "full_type": "EffectRow",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-effect::EffectSignature",
      "source": "crates/af-effect/src/model.rs"
    },
    "af-effect::OccurrenceBounds": {
      "fields": [
        {
          "collection": false,
          "field": "lower",
          "full_type": "u64",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "upper",
          "full_type": "OccurrenceCeiling",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-effect::OccurrenceBounds",
      "source": "crates/af-effect/src/model.rs"
    },
    "af-effect::OccurrenceCeiling": {
      "kind": "enum",
      "owner_type": "af-effect::OccurrenceCeiling",
      "source": "crates/af-effect/src/model.rs",
      "variants": {
        "Finite": [
          {
            "collection": false,
            "field": "0",
            "full_type": "u64",
            "option_presence": "required"
          }
        ],
        "Unbounded": []
      }
    },
    "af-effect::Proof": {
      "kind": "enum",
      "owner_type": "af-effect::Proof",
      "source": "crates/af-effect/src/algebra.rs",
      "variants": {
        "Proven": [],
        "Unproven": [
          {
            "collection": false,
            "field": "0",
            "full_type": "UnprovenReason",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-effect::ResidualKey": {
      "fields": [
        {
          "collection": false,
          "field": "catalog",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "port_no",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "scope",
          "full_type": "Vec<(String, String)>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-effect::ResidualKey",
      "source": "crates/af-effect/src/algebra.rs"
    },
    "af-effect::ResourceSelector": {
      "fields": [
        {
          "collection": false,
          "field": "requirement_port_no",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "scope_bindings",
          "full_type": "Vec<ScopeBinding>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-effect::ResourceSelector",
      "source": "crates/af-effect/src/model.rs"
    },
    "af-effect::RetryEnvelope": {
      "fields": [
        {
          "collection": false,
          "field": "logical",
          "full_type": "EffectSignature",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "attempts",
          "full_type": "OccurrenceBounds",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "logical_narrowing_proven",
          "full_type": "bool",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "obligations",
          "full_type": "Vec<&'static str>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-effect::RetryEnvelope",
      "source": "crates/af-effect/src/control.rs"
    },
    "af-effect::ScopeBinding": {
      "fields": [
        {
          "collection": false,
          "field": "key",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "value",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-effect::ScopeBinding",
      "source": "crates/af-effect/src/model.rs"
    },
    "af-effect::UnprovenReason": {
      "kind": "enum",
      "owner_type": "af-effect::UnprovenReason",
      "source": "crates/af-effect/src/algebra.rs",
      "variants": {
        "EntryUncovered": [],
        "ResidualInsideClosed": [],
        "ResidualUncovered": [],
        "UnknownRow": []
      }
    },
    "af-execution::CapabilityAttribute": {
      "fields": [
        {
          "collection": false,
          "field": "key",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "value",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-execution::CapabilityAttribute",
      "source": "crates/af-execution/src/model.rs"
    },
    "af-execution::CapabilityDescriptor": {
      "fields": [
        {
          "collection": false,
          "field": "capability",
          "full_type": "ExecutionCapabilityRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "exact_version",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "supported_versions",
          "full_type": "BTreeSet<String>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "attributes",
          "full_type": "BTreeSet<(String, String)>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-execution::CapabilityDescriptor",
      "source": "crates/af-execution/src/universe.rs"
    },
    "af-execution::CapabilityRequirement": {
      "fields": [
        {
          "collection": false,
          "field": "capability",
          "full_type": "ExecutionCapabilityRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "exact_version",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "required_attributes",
          "full_type": "Vec<CapabilityAttribute>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-execution::CapabilityRequirement",
      "source": "crates/af-execution/src/model.rs"
    },
    "af-execution::CompatibilityVerdict": {
      "kind": "enum",
      "owner_type": "af-execution::CompatibilityVerdict",
      "source": "crates/af-execution/src/compatibility.rs",
      "variants": {
        "Proven": [],
        "Unproven": [],
        "Violation": []
      }
    },
    "af-execution::ComputeQuantityKind": {
      "kind": "enum",
      "owner_type": "af-execution::ComputeQuantityKind",
      "source": "crates/af-execution/src/model.rs",
      "variants": {
        "CpuMillicores": [],
        "GpuCount": [],
        "MemoryBytes": []
      }
    },
    "af-execution::ComputeQuantityRequirement": {
      "fields": [
        {
          "collection": false,
          "field": "kind",
          "full_type": "ComputeQuantityKind",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "minimum",
          "full_type": "u64",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-execution::ComputeQuantityRequirement",
      "source": "crates/af-execution/src/model.rs"
    },
    "af-execution::ExecutionCapabilityRef": {
      "fields": [
        {
          "collection": false,
          "field": "catalog",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-execution::ExecutionCapabilityRef",
      "source": "crates/af-execution/src/model.rs"
    },
    "af-execution::ExecutionCapabilityUniverse": {
      "fields": [
        {
          "collection": true,
          "field": "capabilities",
          "full_type": "BTreeMap<ExecutionCapabilityRef, CapabilityDescriptor>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-execution::ExecutionCapabilityUniverse",
      "source": "crates/af-execution/src/universe.rs"
    },
    "af-execution::ExecutionCompatibility": {
      "fields": [
        {
          "collection": false,
          "field": "feasibility",
          "full_type": "CompatibilityVerdict",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "security",
          "full_type": "CompatibilityVerdict",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-execution::ExecutionCompatibility",
      "source": "crates/af-execution/src/compatibility.rs"
    },
    "af-execution::ExecutionError": {
      "fields": [
        {
          "collection": false,
          "field": "kind",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "detail",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-execution::ExecutionError",
      "source": "crates/af-execution/src/error.rs"
    },
    "af-execution::ExecutionRequirements": {
      "fields": [
        {
          "collection": true,
          "field": "capabilities",
          "full_type": "Vec<CapabilityRequirement>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "quantities",
          "full_type": "Vec<ComputeQuantityRequirement>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "sandbox",
          "full_type": "Option<SandboxRequirement>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "trust_domain",
          "full_type": "Option<TrustDomainRef>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "region_residency",
          "full_type": "Option<RegionResidencyRequirement>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "runtime_profile",
          "full_type": "Option<RuntimeProfileRequirement>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": true,
          "field": "implementation_abis",
          "full_type": "Vec<ImplementationAbiRequirement>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "soft_preferences",
          "full_type": "Vec<PlacementPreference>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-execution::ExecutionRequirements",
      "source": "crates/af-execution/src/model.rs"
    },
    "af-execution::ExecutionUniverses": {
      "fields": [
        {
          "collection": false,
          "field": "capabilities",
          "full_type": "ExecutionCapabilityUniverse",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "trust_domains",
          "full_type": "TrustDomainUniverse",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "regions",
          "full_type": "RegionUniverse",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "runtime_profiles",
          "full_type": "RuntimeProfileUniverse",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-execution::ExecutionUniverses",
      "source": "crates/af-execution/src/universe.rs"
    },
    "af-execution::ImplementationAbiRequirement": {
      "fields": [
        {
          "collection": false,
          "field": "schema",
          "full_type": "TypeSchemaRef",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-execution::ImplementationAbiRequirement",
      "source": "crates/af-execution/src/model.rs"
    },
    "af-execution::MatchRejected": {
      "kind": "enum",
      "owner_type": "af-execution::MatchRejected",
      "source": "crates/af-execution/src/matching.rs",
      "variants": {
        "AbiMismatch": [],
        "CapabilityMissing": [],
        "CapabilityVersion": [],
        "QuantityInsufficient": [],
        "RegionMismatch": [],
        "ResidencyMismatch": [],
        "RuntimeProfileMismatch": [],
        "SandboxTooWeak": [],
        "TrustDomainMismatch": []
      }
    },
    "af-execution::MatchUnproven": {
      "kind": "enum",
      "owner_type": "af-execution::MatchUnproven",
      "source": "crates/af-execution/src/matching.rs",
      "variants": {
        "AbiProfile": [],
        "CapabilityProfile": [],
        "QuantityProfile": [],
        "RegionResidency": [],
        "RuntimeProfile": [],
        "SandboxProfile": [],
        "TrustDomain": []
      }
    },
    "af-execution::PlacementPreference": {
      "kind": "enum",
      "owner_type": "af-execution::PlacementPreference",
      "source": "crates/af-execution/src/model.rs",
      "variants": {
        "ArtifactLocality": [],
        "LowerCost": [],
        "LowerLoad": [],
        "WarmModelCache": []
      }
    },
    "af-execution::RegionDirection": {
      "kind": "enum",
      "owner_type": "af-execution::RegionDirection",
      "source": "crates/af-execution/src/compatibility.rs",
      "variants": {
        "Expanded": [],
        "Replaced": [],
        "Same": [],
        "Tightened": []
      }
    },
    "af-execution::RegionRef": {
      "fields": [
        {
          "collection": false,
          "field": "catalog",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-execution::RegionRef",
      "source": "crates/af-execution/src/model.rs"
    },
    "af-execution::RegionResidencyRequirement": {
      "fields": [
        {
          "collection": true,
          "field": "execution_regions",
          "full_type": "Vec<RegionRef>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "residency_regions",
          "full_type": "Vec<RegionRef>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-execution::RegionResidencyRequirement",
      "source": "crates/af-execution/src/model.rs"
    },
    "af-execution::RegionUniverse": {
      "fields": [
        {
          "collection": true,
          "field": "regions",
          "full_type": "BTreeSet<RegionRef>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "compatibility",
          "full_type": "BTreeSet<(RegionRef, RegionRef)>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-execution::RegionUniverse",
      "source": "crates/af-execution/src/universe.rs"
    },
    "af-execution::RuntimeProfileDescriptor": {
      "fields": [
        {
          "collection": false,
          "field": "profile",
          "full_type": "RuntimeProfileRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "family",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "features",
          "full_type": "BTreeSet<String>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-execution::RuntimeProfileDescriptor",
      "source": "crates/af-execution/src/universe.rs"
    },
    "af-execution::RuntimeProfileRef": {
      "fields": [
        {
          "collection": false,
          "field": "catalog",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-execution::RuntimeProfileRef",
      "source": "crates/af-execution/src/model.rs"
    },
    "af-execution::RuntimeProfileRequirement": {
      "fields": [
        {
          "collection": false,
          "field": "family",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "profile",
          "full_type": "RuntimeProfileRef",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "required_features",
          "full_type": "Vec<String>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-execution::RuntimeProfileRequirement",
      "source": "crates/af-execution/src/model.rs"
    },
    "af-execution::RuntimeProfileUniverse": {
      "fields": [
        {
          "collection": true,
          "field": "profiles",
          "full_type": "BTreeMap<RuntimeProfileRef, RuntimeProfileDescriptor>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-execution::RuntimeProfileUniverse",
      "source": "crates/af-execution/src/universe.rs"
    },
    "af-execution::SandboxLevel": {
      "kind": "enum",
      "owner_type": "af-execution::SandboxLevel",
      "source": "crates/af-execution/src/model.rs",
      "variants": {
        "Basic": [],
        "Strong": []
      }
    },
    "af-execution::SandboxRequirement": {
      "fields": [
        {
          "collection": false,
          "field": "minimum",
          "full_type": "SandboxLevel",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-execution::SandboxRequirement",
      "source": "crates/af-execution/src/model.rs"
    },
    "af-execution::StaticCapability": {
      "fields": [
        {
          "collection": false,
          "field": "capability",
          "full_type": "ExecutionCapabilityRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "exact_version",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "attributes",
          "full_type": "BTreeSet<(String, String)>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-execution::StaticCapability",
      "source": "crates/af-execution/src/profile.rs"
    },
    "af-execution::StaticExecutorProfile": {
      "fields": [
        {
          "collection": true,
          "field": "capabilities",
          "full_type": "Vec<StaticCapability>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "quantities",
          "full_type": "Vec<StaticQuantity>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "sandbox",
          "full_type": "Option<SandboxLevel>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "trust_domain",
          "full_type": "Option<TrustDomainRef>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "execution_region",
          "full_type": "Option<RegionRef>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": true,
          "field": "residency_regions",
          "full_type": "Vec<RegionRef>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "runtime_profile",
          "full_type": "Option<RuntimeProfileRef>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": true,
          "field": "implementation_abis",
          "full_type": "Vec<TypeSchemaRef>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-execution::StaticExecutorProfile",
      "source": "crates/af-execution/src/profile.rs"
    },
    "af-execution::StaticPlacementProof": {
      "kind": "enum",
      "owner_type": "af-execution::StaticPlacementProof",
      "source": "crates/af-execution/src/matching.rs",
      "variants": {
        "ProvenAgainstDescriptor": [],
        "Rejected": [
          {
            "collection": false,
            "field": "0",
            "full_type": "MatchRejected",
            "option_presence": "required"
          }
        ],
        "Unproven": [
          {
            "collection": false,
            "field": "0",
            "full_type": "MatchUnproven",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-execution::StaticQuantity": {
      "fields": [
        {
          "collection": false,
          "field": "kind",
          "full_type": "ComputeQuantityKind",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "amount",
          "full_type": "u64",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-execution::StaticQuantity",
      "source": "crates/af-execution/src/profile.rs"
    },
    "af-execution::StaticSandbox": {
      "fields": [
        {
          "collection": false,
          "field": "level",
          "full_type": "SandboxLevel",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-execution::StaticSandbox",
      "source": "crates/af-execution/src/profile.rs"
    },
    "af-execution::TrustDomainRef": {
      "fields": [
        {
          "collection": false,
          "field": "catalog",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-execution::TrustDomainRef",
      "source": "crates/af-execution/src/model.rs"
    },
    "af-execution::TrustDomainUniverse": {
      "fields": [
        {
          "collection": true,
          "field": "domains",
          "full_type": "BTreeSet<TrustDomainRef>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-execution::TrustDomainUniverse",
      "source": "crates/af-execution/src/universe.rs"
    },
    "af-failure::FailureError": {
      "fields": [
        {
          "collection": false,
          "field": "kind",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "detail",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-failure::FailureError",
      "source": "crates/af-failure/src/error.rs"
    },
    "af-failure::FailureRow": {
      "fields": [
        {
          "collection": true,
          "field": "failures",
          "full_type": "Vec<FailureTypeRef>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-failure::FailureRow",
      "source": "crates/af-failure/src/model.rs"
    },
    "af-failure::FailureTypeRef": {
      "fields": [
        {
          "collection": false,
          "field": "payload_type",
          "full_type": "TypeSchemaRef",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-failure::FailureTypeRef",
      "source": "crates/af-failure/src/model.rs"
    },
    "af-failure::StaticFailureEnvelope": {
      "kind": "enum",
      "owner_type": "af-failure::StaticFailureEnvelope",
      "source": "crates/af-failure/src/control.rs",
      "variants": {
        "Proven": [
          {
            "collection": false,
            "field": "0",
            "full_type": "FailureRow",
            "option_presence": "required"
          }
        ],
        "Unproven": []
      }
    },
    "af-flow::ConfidentialityLabelRef": {
      "fields": [
        {
          "collection": false,
          "field": "catalog",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-flow::ConfidentialityLabelRef",
      "source": "crates/af-flow/src/model.rs"
    },
    "af-flow::EffectSinkUniverse": {
      "fields": [
        {
          "collection": true,
          "field": "operations",
          "full_type": "BTreeSet<(String, String)>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "decision_capable",
          "full_type": "BTreeSet<(String, String)>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-flow::EffectSinkUniverse",
      "source": "crates/af-flow/src/universe.rs"
    },
    "af-flow::FlowCatalogs": {
      "fields": [
        {
          "collection": false,
          "field": "labels",
          "full_type": "InformationLabelUniverse",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "sinks",
          "full_type": "EffectSinkUniverse",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-flow::FlowCatalogs",
      "source": "crates/af-flow/src/universe.rs"
    },
    "af-flow::FlowCompatibilityProof": {
      "kind": "enum",
      "owner_type": "af-flow::FlowCompatibilityProof",
      "source": "crates/af-flow/src/compatibility.rs",
      "variants": {
        "Proven": [],
        "Unproven": [
          {
            "collection": false,
            "field": "0",
            "full_type": "UnprovenReason",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-flow::FlowConstraint": {
      "fields": [
        {
          "collection": false,
          "field": "kind",
          "full_type": "FlowKind",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "sink",
          "full_type": "FlowSink",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-flow::FlowConstraint",
      "source": "crates/af-flow/src/model.rs"
    },
    "af-flow::FlowError": {
      "fields": [
        {
          "collection": false,
          "field": "kind",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "detail",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-flow::FlowError",
      "source": "crates/af-flow/src/error.rs"
    },
    "af-flow::FlowKind": {
      "kind": "enum",
      "owner_type": "af-flow::FlowKind",
      "source": "crates/af-flow/src/model.rs",
      "variants": {
        "DataEgress": [],
        "DecisionInfluence": []
      }
    },
    "af-flow::FlowProof": {
      "kind": "enum",
      "owner_type": "af-flow::FlowProof",
      "source": "crates/af-flow/src/propagation.rs",
      "variants": {
        "ProvenWithinDeclaredCeiling": [],
        "RequiresTrustedTransformation": [
          {
            "collection": false,
            "field": "requires_declassification",
            "full_type": "bool",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "requires_endorsement",
            "full_type": "bool",
            "option_presence": "required"
          }
        ],
        "Unproven": []
      }
    },
    "af-flow::FlowSink": {
      "kind": "enum",
      "owner_type": "af-flow::FlowSink",
      "source": "crates/af-flow/src/model.rs",
      "variants": {
        "ExternalEffect": [
          {
            "collection": false,
            "field": "0",
            "full_type": "EffectOperationRef",
            "option_presence": "required"
          }
        ],
        "WorkflowOutput": [
          {
            "collection": false,
            "field": "0",
            "full_type": "u32",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-flow::FlowSourceRef": {
      "fields": [
        {
          "collection": false,
          "field": "category",
          "full_type": "SourceCategory",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-flow::FlowSourceRef",
      "source": "crates/af-flow/src/model.rs"
    },
    "af-flow::InformationFlowContract": {
      "fields": [
        {
          "collection": true,
          "field": "confidentiality_restrictions",
          "full_type": "Vec<ConfidentialityLabelRef>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "summary",
          "full_type": "StaticInformationSummary",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "flow_constraints",
          "full_type": "Vec<FlowConstraint>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-flow::InformationFlowContract",
      "source": "crates/af-flow/src/model.rs"
    },
    "af-flow::InformationLabelUniverse": {
      "fields": [
        {
          "collection": true,
          "field": "confidentiality",
          "full_type": "BTreeSet<(String, String)>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "taints",
          "full_type": "BTreeSet<(String, String)>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-flow::InformationLabelUniverse",
      "source": "crates/af-flow/src/universe.rs"
    },
    "af-flow::InstructionAuthority": {
      "kind": "enum",
      "owner_type": "af-flow::InstructionAuthority",
      "source": "crates/af-flow/src/model.rs",
      "variants": {
        "DelegatedAgent": [],
        "ExternalData": [],
        "None": [],
        "Platform": [],
        "User": [],
        "Workflow": []
      }
    },
    "af-flow::SinkClassification": {
      "kind": "enum",
      "owner_type": "af-flow::SinkClassification",
      "source": "crates/af-flow/src/universe.rs",
      "variants": {
        "DecisionCapable": [],
        "Plain": [],
        "Unknown": []
      }
    },
    "af-flow::SourceCategory": {
      "kind": "enum",
      "owner_type": "af-flow::SourceCategory",
      "source": "crates/af-flow/src/model.rs",
      "variants": {
        "ResourceRequirement": [
          {
            "collection": false,
            "field": "0",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "UnknownExternal": [],
        "WorkflowInput": [
          {
            "collection": false,
            "field": "0",
            "full_type": "u32",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-flow::StaticInformationSummary": {
      "fields": [
        {
          "collection": true,
          "field": "sources",
          "full_type": "Vec<FlowSourceRef>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "taints",
          "full_type": "Vec<TaintRef>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "instruction_authority",
          "full_type": "InstructionAuthority",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "influence_unknown",
          "full_type": "bool",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-flow::StaticInformationSummary",
      "source": "crates/af-flow/src/model.rs"
    },
    "af-flow::TaintRef": {
      "fields": [
        {
          "collection": false,
          "field": "catalog",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-flow::TaintRef",
      "source": "crates/af-flow/src/model.rs"
    },
    "af-flow::UnprovenReason": {
      "kind": "enum",
      "owner_type": "af-flow::UnprovenReason",
      "source": "crates/af-flow/src/compatibility.rs",
      "variants": {
        "InfluenceRelationUnknown": []
      }
    },
    "af-idl::AfidlError": {
      "kind": "enum",
      "owner_type": "af-idl::AfidlError",
      "source": "crates/af-idl/src/error.rs",
      "variants": {
        "AliasCycle": [
          {
            "collection": true,
            "field": "cycle",
            "full_type": "Vec<String>",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "CrossModuleRecursiveType": [
          {
            "collection": true,
            "field": "cycle",
            "full_type": "Vec<String>",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "DuplicateCaseName": [
          {
            "collection": false,
            "field": "name",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "type_name",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "DuplicateCaseNumber": [
          {
            "collection": false,
            "field": "no",
            "full_type": "u32",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "type_name",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "DuplicateEnumName": [
          {
            "collection": false,
            "field": "name",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "type_name",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "DuplicateEnumNumber": [
          {
            "collection": false,
            "field": "no",
            "full_type": "u32",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "type_name",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "DuplicateFieldName": [
          {
            "collection": false,
            "field": "name",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "type_name",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "DuplicateFieldNumber": [
          {
            "collection": false,
            "field": "no",
            "full_type": "u32",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "type_name",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "DuplicateImportAlias": [
          {
            "collection": false,
            "field": "alias",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "DuplicateImportSpecification": [
          {
            "collection": false,
            "field": "id",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "DuplicateModuleSpecification": [
          {
            "collection": false,
            "field": "id",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "label",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "DuplicateSchemaDeclaration": [
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "DuplicateTypeDefinition": [
          {
            "collection": false,
            "field": "name",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "EmptyEnum": [
          {
            "collection": false,
            "field": "name",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "EmptyVariant": [
          {
            "collection": false,
            "field": "name",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "ImportAfterDeclaration": [
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "InvalidCaseNumber": [
          {
            "collection": false,
            "field": "no",
            "full_type": "u32",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "type_name",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "InvalidEnumNumber": [
          {
            "collection": false,
            "field": "no",
            "full_type": "u32",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "type_name",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "InvalidFieldNumber": [
          {
            "collection": false,
            "field": "no",
            "full_type": "u32",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "type_name",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "InvalidIdentifier": [
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "InvalidImportAlias": [
          {
            "collection": false,
            "field": "alias",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "InvalidLanguageVersion": [
          {
            "collection": false,
            "field": "found",
            "full_type": "u32",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "InvalidSpecificationId": [
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "Lex": [
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "Parse": [
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "QualifiedPrimitiveReference": [
          {
            "collection": false,
            "field": "keyword",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "RecursiveTypeNotSupported": [
          {
            "collection": false,
            "field": "type_name",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "SelfImport": [
          {
            "collection": false,
            "field": "id",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "UnknownImportAlias": [
          {
            "collection": false,
            "field": "alias",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "UnknownImportSpecification": [
          {
            "collection": false,
            "field": "id",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "UnknownImportedType": [
          {
            "collection": false,
            "field": "specification",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "type_name",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "UnknownTypeReference": [
          {
            "collection": false,
            "field": "name",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ],
        "UnqualifiedImportedReference": [
          {
            "collection": false,
            "field": "type_name",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "specification",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "alias",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "span",
            "full_type": "SourceSpan",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-idl::AliasDecl": {
      "fields": [
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "target",
          "full_type": "TypeExpr",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "span",
          "full_type": "SourceSpan",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-idl::AliasDecl",
      "source": "crates/af-idl/src/ast.rs"
    },
    "af-idl::CanonicalBody": {
      "kind": "enum",
      "owner_type": "af-idl::CanonicalBody",
      "source": "crates/af-idl/src/canonical.rs",
      "variants": {
        "Alias": [
          {
            "collection": false,
            "field": "0",
            "full_type": "CanonicalTypeExpr",
            "option_presence": "required"
          }
        ],
        "Enum": [
          {
            "collection": true,
            "field": "0",
            "full_type": "Vec<CanonicalEnumValue>",
            "option_presence": "required"
          }
        ],
        "Record": [
          {
            "collection": true,
            "field": "0",
            "full_type": "Vec<CanonicalField>",
            "option_presence": "required"
          }
        ],
        "Variant": [
          {
            "collection": true,
            "field": "0",
            "full_type": "Vec<CanonicalCase>",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-idl::CanonicalCase": {
      "fields": [
        {
          "collection": false,
          "field": "no",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "type_expr",
          "full_type": "CanonicalTypeExpr",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-idl::CanonicalCase",
      "source": "crates/af-idl/src/canonical.rs"
    },
    "af-idl::CanonicalEnumValue": {
      "fields": [
        {
          "collection": false,
          "field": "no",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-idl::CanonicalEnumValue",
      "source": "crates/af-idl/src/canonical.rs"
    },
    "af-idl::CanonicalField": {
      "fields": [
        {
          "collection": false,
          "field": "no",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "type_expr",
          "full_type": "CanonicalTypeExpr",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-idl::CanonicalField",
      "source": "crates/af-idl/src/canonical.rs"
    },
    "af-idl::CanonicalTypeExpr": {
      "kind": "enum",
      "owner_type": "af-idl::CanonicalTypeExpr",
      "source": "crates/af-idl/src/canonical.rs",
      "variants": {
        "List": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Box<CanonicalTypeExpr>",
            "option_presence": "required"
          }
        ],
        "MapString": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Box<CanonicalTypeExpr>",
            "option_presence": "required"
          }
        ],
        "NamedRef": [
          {
            "collection": false,
            "field": "0",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "Option": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Box<CanonicalTypeExpr>",
            "option_presence": "required"
          }
        ],
        "Primitive": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Primitive",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-idl::CanonicalTypeSchema": {
      "fields": [
        {
          "collection": false,
          "field": "schema",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "body",
          "full_type": "CanonicalBody",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-idl::CanonicalTypeSchema",
      "source": "crates/af-idl/src/canonical.rs"
    },
    "af-idl::CompiledModuleSet": {
      "fields": [
        {
          "collection": true,
          "field": "modules",
          "full_type": "BTreeMap<String, CompiledSchema>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-idl::CompiledModuleSet",
      "source": "crates/af-idl/src/resolve.rs"
    },
    "af-idl::CompiledSchema": {
      "fields": [
        {
          "collection": false,
          "field": "specification",
          "full_type": "SpecificationId",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "types",
          "full_type": "Vec<CanonicalTypeSchema>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-idl::CompiledSchema",
      "source": "crates/af-idl/src/canonical.rs"
    },
    "af-idl::Decl": {
      "kind": "enum",
      "owner_type": "af-idl::Decl",
      "source": "crates/af-idl/src/ast.rs",
      "variants": {
        "Alias": [
          {
            "collection": false,
            "field": "0",
            "full_type": "AliasDecl",
            "option_presence": "required"
          }
        ],
        "Enum": [
          {
            "collection": false,
            "field": "0",
            "full_type": "EnumDecl",
            "option_presence": "required"
          }
        ],
        "Record": [
          {
            "collection": false,
            "field": "0",
            "full_type": "RecordDecl",
            "option_presence": "required"
          }
        ],
        "Variant": [
          {
            "collection": false,
            "field": "0",
            "full_type": "VariantDecl",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-idl::DeclaredTypeIndex": {
      "fields": [
        {
          "collection": true,
          "field": "names",
          "full_type": "BTreeMap<String, BTreeSet<String>>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-idl::DeclaredTypeIndex",
      "source": "crates/af-idl/src/resolve.rs"
    },
    "af-idl::EnumDecl": {
      "fields": [
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "values",
          "full_type": "Vec<EnumValueDecl>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "span",
          "full_type": "SourceSpan",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-idl::EnumDecl",
      "source": "crates/af-idl/src/ast.rs"
    },
    "af-idl::EnumValueDecl": {
      "fields": [
        {
          "collection": false,
          "field": "no",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "span",
          "full_type": "SourceSpan",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-idl::EnumValueDecl",
      "source": "crates/af-idl/src/ast.rs"
    },
    "af-idl::FieldDecl": {
      "fields": [
        {
          "collection": false,
          "field": "no",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "type_expr",
          "full_type": "TypeExpr",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "span",
          "full_type": "SourceSpan",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-idl::FieldDecl",
      "source": "crates/af-idl/src/ast.rs"
    },
    "af-idl::ImportDecl": {
      "fields": [
        {
          "collection": false,
          "field": "id",
          "full_type": "SpecificationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "alias",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "span",
          "full_type": "SourceSpan",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-idl::ImportDecl",
      "source": "crates/af-idl/src/ast.rs"
    },
    "af-idl::Module": {
      "fields": [
        {
          "collection": false,
          "field": "language_version",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "schema",
          "full_type": "SchemaDecl",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "imports",
          "full_type": "Vec<ImportDecl>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "declarations",
          "full_type": "Vec<Decl>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-idl::Module",
      "source": "crates/af-idl/src/ast.rs"
    },
    "af-idl::Parser": {
      "fields": [
        {
          "collection": false,
          "field": "tokens",
          "full_type": "&'a [Token]",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "pos",
          "full_type": "usize",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-idl::Parser",
      "source": "crates/af-idl/src/parser.rs"
    },
    "af-idl::Primitive": {
      "kind": "enum",
      "owner_type": "af-idl::Primitive",
      "source": "crates/af-idl/src/primitives.rs",
      "variants": {
        "Bool": [],
        "Bytes": [],
        "Decimal": [],
        "Duration": [],
        "Float64": [],
        "Int32": [],
        "Int64": [],
        "Null": [],
        "String": [],
        "Timestamp": [],
        "UInt32": [],
        "UInt64": [],
        "Unit": []
      }
    },
    "af-idl::QualifiedRef": {
      "fields": [
        {
          "collection": false,
          "field": "import_alias",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "type_name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "span",
          "full_type": "SourceSpan",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-idl::QualifiedRef",
      "source": "crates/af-idl/src/ast.rs"
    },
    "af-idl::RecordDecl": {
      "fields": [
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "fields",
          "full_type": "Vec<FieldDecl>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "span",
          "full_type": "SourceSpan",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-idl::RecordDecl",
      "source": "crates/af-idl/src/ast.rs"
    },
    "af-idl::ResolvedSchemaUniverse": {
      "fields": [
        {
          "collection": true,
          "field": "modules",
          "full_type": "BTreeMap<String, CompiledSchema>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-idl::ResolvedSchemaUniverse",
      "source": "crates/af-idl/src/resolve.rs"
    },
    "af-idl::SchemaDecl": {
      "fields": [
        {
          "collection": false,
          "field": "id",
          "full_type": "SpecificationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "span",
          "full_type": "SourceSpan",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-idl::SchemaDecl",
      "source": "crates/af-idl/src/ast.rs"
    },
    "af-idl::SourceSpan": {
      "fields": [
        {
          "collection": false,
          "field": "start",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "end",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "line",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "column",
          "full_type": "u32",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-idl::SourceSpan",
      "source": "crates/af-idl/src/span.rs"
    },
    "af-idl::SpecificationId": {
      "fields": [
        {
          "collection": false,
          "field": "namespace",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "major",
          "full_type": "u32",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-idl::SpecificationId",
      "source": "crates/af-idl/src/specification.rs"
    },
    "af-idl::Token": {
      "fields": [
        {
          "collection": false,
          "field": "kind",
          "full_type": "TokenKind",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "text",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "span",
          "full_type": "SourceSpan",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-idl::Token",
      "source": "crates/af-idl/src/lexer.rs"
    },
    "af-idl::TokenKind": {
      "kind": "enum",
      "owner_type": "af-idl::TokenKind",
      "source": "crates/af-idl/src/lexer.rs",
      "variants": {
        "At": [],
        "Colon": [],
        "Comma": [],
        "Dot": [],
        "Equals": [],
        "GreaterThan": [],
        "Int": [],
        "LeftBrace": [],
        "LessThan": [],
        "LowerIdent": [],
        "RightBrace": [],
        "Semicolon": [],
        "UpperIdent": []
      }
    },
    "af-idl::TypeExpr": {
      "fields": [
        {
          "collection": false,
          "field": "kind",
          "full_type": "TypeExprKind",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "span",
          "full_type": "SourceSpan",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-idl::TypeExpr",
      "source": "crates/af-idl/src/ast.rs"
    },
    "af-idl::TypeExprKind": {
      "kind": "enum",
      "owner_type": "af-idl::TypeExprKind",
      "source": "crates/af-idl/src/ast.rs",
      "variants": {
        "List": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Box<TypeExpr>",
            "option_presence": "required"
          }
        ],
        "MapString": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Box<TypeExpr>",
            "option_presence": "required"
          }
        ],
        "NamedRef": [
          {
            "collection": false,
            "field": "0",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "Option": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Box<TypeExpr>",
            "option_presence": "required"
          }
        ],
        "Primitive": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Primitive",
            "option_presence": "required"
          }
        ],
        "QualifiedNamedRef": [
          {
            "collection": false,
            "field": "0",
            "full_type": "QualifiedRef",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-idl::VariantCaseDecl": {
      "fields": [
        {
          "collection": false,
          "field": "no",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "payload",
          "full_type": "TypeExpr",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "span",
          "full_type": "SourceSpan",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-idl::VariantCaseDecl",
      "source": "crates/af-idl/src/ast.rs"
    },
    "af-idl::VariantDecl": {
      "fields": [
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "cases",
          "full_type": "Vec<VariantCaseDecl>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "span",
          "full_type": "SourceSpan",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-idl::VariantDecl",
      "source": "crates/af-idl/src/ast.rs"
    },
    "af-ir::AvailabilityReason": {
      "kind": "enum",
      "owner_type": "af-ir::AvailabilityReason",
      "source": "crates/af-ir/src/dataflow.rs",
      "variants": {
        "BeforeProducer": [],
        "CrossBranch": [],
        "IterateLocalOutside": [],
        "OutsideOwnerBoundary": [],
        "ParentOutputNotYetAvailable": []
      }
    },
    "af-ir::AwaitBody": {
      "fields": [
        {
          "collection": false,
          "field": "signal_type",
          "full_type": "TypeSchemaRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "correlation",
          "full_type": "ExpressionId",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::AwaitBody",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::CallSubflowBody": {
      "fields": [
        {
          "collection": false,
          "field": "contract",
          "full_type": "ContractRefId",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "inputs",
          "full_type": "Vec<InputBinding>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "resources",
          "full_type": "Vec<ResourceUseBinding>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::CallSubflowBody",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::CheckedIrDocument": {
      "fields": [
        {
          "collection": false,
          "field": "source",
          "full_type": "&'d SemanticIrDocument",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::CheckedIrDocument",
      "source": "crates/af-ir/src/check.rs"
    },
    "af-ir::CompensationScopeBody": {
      "fields": [
        {
          "collection": false,
          "field": "child",
          "full_type": "ComputationId",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::CompensationScopeBody",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::ComputationBody": {
      "kind": "enum",
      "owner_type": "af-ir::ComputationBody",
      "source": "crates/af-ir/src/model.rs",
      "variants": {
        "Await": [
          {
            "collection": false,
            "field": "0",
            "full_type": "AwaitBody",
            "option_presence": "required"
          }
        ],
        "CallSubflow": [
          {
            "collection": false,
            "field": "0",
            "full_type": "CallSubflowBody",
            "option_presence": "required"
          }
        ],
        "CompensationScope": [
          {
            "collection": false,
            "field": "0",
            "full_type": "CompensationScopeBody",
            "option_presence": "required"
          }
        ],
        "Handle": [
          {
            "collection": false,
            "field": "0",
            "full_type": "HandleBody",
            "option_presence": "required"
          }
        ],
        "Invoke": [
          {
            "collection": false,
            "field": "0",
            "full_type": "InvokeBody",
            "option_presence": "required"
          }
        ],
        "Iterate": [
          {
            "collection": false,
            "field": "0",
            "full_type": "IterateBody",
            "option_presence": "required"
          }
        ],
        "Parallel": [
          {
            "collection": false,
            "field": "0",
            "full_type": "ParallelBody",
            "option_presence": "required"
          }
        ],
        "Race": [
          {
            "collection": false,
            "field": "0",
            "full_type": "RaceBody",
            "option_presence": "required"
          }
        ],
        "Retry": [
          {
            "collection": false,
            "field": "0",
            "full_type": "RetryBody",
            "option_presence": "required"
          }
        ],
        "Scope": [
          {
            "collection": false,
            "field": "0",
            "full_type": "ScopeBody",
            "option_presence": "required"
          }
        ],
        "Select": [
          {
            "collection": false,
            "field": "0",
            "full_type": "SelectBody",
            "option_presence": "required"
          }
        ],
        "Sequence": [
          {
            "collection": false,
            "field": "0",
            "full_type": "SequenceBody",
            "option_presence": "required"
          }
        ],
        "Timeout": [
          {
            "collection": false,
            "field": "0",
            "full_type": "TimeoutBody",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-ir::ComputationDef": {
      "fields": [
        {
          "collection": false,
          "field": "id",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "outputs",
          "full_type": "Vec<ComputationOutputBinding>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "body",
          "full_type": "ComputationBody",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::ComputationDef",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::ComputationOutputBinding": {
      "fields": [
        {
          "collection": false,
          "field": "port_no",
          "full_type": "PortNo",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "value_slot",
          "full_type": "ValueSlotId",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::ComputationOutputBinding",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::ContractRefEntry": {
      "fields": [
        {
          "collection": false,
          "field": "id",
          "full_type": "ContractRefId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "kind",
          "full_type": "ContractRefKind",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::ContractRefEntry",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::ContractRefKind": {
      "kind": "enum",
      "owner_type": "af-ir::ContractRefKind",
      "source": "crates/af-ir/src/model.rs",
      "variants": {
        "Component": [],
        "Subflow": []
      }
    },
    "af-ir::ExpressionUseSite": {
      "kind": "enum",
      "owner_type": "af-ir::ExpressionUseSite",
      "source": "crates/af-ir/src/dataflow.rs",
      "variants": {
        "AwaitCorrelation": [
          {
            "collection": false,
            "field": "computation",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "CallSubflowInput": [
          {
            "collection": false,
            "field": "computation",
            "full_type": "u32",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "port_no",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "ForeachSource": [
          {
            "collection": false,
            "field": "computation",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "InvokeInput": [
          {
            "collection": false,
            "field": "computation",
            "full_type": "u32",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "port_no",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "IterateCarryInitial": [
          {
            "collection": false,
            "field": "computation",
            "full_type": "u32",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "slot",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "SelectSelector": [
          {
            "collection": false,
            "field": "computation",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "TimeoutDuration": [
          {
            "collection": false,
            "field": "computation",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "WhileGuard": [
          {
            "collection": false,
            "field": "computation",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "WorkflowOutput": [
          {
            "collection": false,
            "field": "name",
            "full_type": "String",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-ir::FailureMatch": {
      "fields": [
        {
          "collection": false,
          "field": "failure_type",
          "full_type": "TypeSchemaRef",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::FailureMatch",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::ForeachBody": {
      "fields": [
        {
          "collection": false,
          "field": "source",
          "full_type": "ExpressionId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "item_slot",
          "full_type": "ValueSlotId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "body",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "carry",
          "full_type": "Vec<LoopCarryBinding>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "max_inflight",
          "full_type": "Option<u32>",
          "option_presence": "explicit_none_or_some"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::ForeachBody",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::HandleBody": {
      "fields": [
        {
          "collection": false,
          "field": "child",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "clauses",
          "full_type": "Vec<HandleClause>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::HandleBody",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::HandleClause": {
      "fields": [
        {
          "collection": false,
          "field": "ordinal",
          "full_type": "HandlerOrdinal",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "matcher",
          "full_type": "HandleMatch",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "handler",
          "full_type": "ComputationId",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::HandleClause",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::HandleMatch": {
      "kind": "enum",
      "owner_type": "af-ir::HandleMatch",
      "source": "crates/af-ir/src/model.rs",
      "variants": {
        "Failure": [
          {
            "collection": false,
            "field": "0",
            "full_type": "FailureMatch",
            "option_presence": "required"
          }
        ],
        "TimedOut": [
          {
            "collection": false,
            "field": "0",
            "full_type": "()",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-ir::InputBinding": {
      "fields": [
        {
          "collection": false,
          "field": "port_no",
          "full_type": "PortNo",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "expression",
          "full_type": "ExpressionId",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::InputBinding",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::InvokeBody": {
      "fields": [
        {
          "collection": false,
          "field": "contract",
          "full_type": "ContractRefId",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "inputs",
          "full_type": "Vec<InputBinding>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "resources",
          "full_type": "Vec<ResourceUseBinding>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::InvokeBody",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::IrError": {
      "kind": "enum",
      "owner_type": "af-ir::IrError",
      "source": "crates/af-ir/src/error.rs",
      "variants": {
        "Core": [
          {
            "collection": false,
            "field": "0",
            "full_type": "SemanticError",
            "option_presence": "required"
          }
        ],
        "Encoding": [
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "InvalidLocalId": [
          {
            "collection": false,
            "field": "namespace",
            "full_type": "&'static str",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "id",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "Rule": [
          {
            "collection": false,
            "field": "kind",
            "full_type": "&'static str",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "UnknownComputation": [
          {
            "collection": false,
            "field": "id",
            "full_type": "ComputationId",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-ir::IterateBody": {
      "kind": "enum",
      "owner_type": "af-ir::IterateBody",
      "source": "crates/af-ir/src/model.rs",
      "variants": {
        "Foreach": [
          {
            "collection": false,
            "field": "0",
            "full_type": "ForeachBody",
            "option_presence": "required"
          }
        ],
        "While": [
          {
            "collection": false,
            "field": "0",
            "full_type": "WhileBody",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-ir::LoopCarryBinding": {
      "fields": [
        {
          "collection": false,
          "field": "slot",
          "full_type": "ValueSlotId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "initial",
          "full_type": "ExpressionId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "body_output_port",
          "full_type": "PortNo",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::LoopCarryBinding",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::ParallelBody": {
      "fields": [
        {
          "collection": false,
          "field": "mode",
          "full_type": "ParallelMode",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "branches",
          "full_type": "Vec<ParallelBranch>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "max_inflight",
          "full_type": "Option<u32>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": true,
          "field": "output_bindings",
          "full_type": "Vec<ParallelOutputBinding>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::ParallelBody",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::ParallelBranch": {
      "fields": [
        {
          "collection": false,
          "field": "ordinal",
          "full_type": "BranchOrdinal",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "child",
          "full_type": "ComputationId",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::ParallelBranch",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::ParallelMode": {
      "kind": "enum",
      "owner_type": "af-ir::ParallelMode",
      "source": "crates/af-ir/src/model.rs",
      "variants": {
        "All": [],
        "FailFast": []
      }
    },
    "af-ir::ParallelOutputBinding": {
      "fields": [
        {
          "collection": false,
          "field": "parent_port_no",
          "full_type": "PortNo",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "branch_ordinal",
          "full_type": "BranchOrdinal",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "child_port_no",
          "full_type": "PortNo",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::ParallelOutputBinding",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::ProducerLocation": {
      "kind": "enum",
      "owner_type": "af-ir::ProducerLocation",
      "source": "crates/af-ir/src/dataflow.rs",
      "variants": {
        "ComputationOutput": [
          {
            "collection": false,
            "field": "computation",
            "full_type": "u32",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "port_no",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "IterateCarry": [
          {
            "collection": false,
            "field": "iterate",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "IterateItem": [
          {
            "collection": false,
            "field": "iterate",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "RunParameter": [
          {
            "collection": false,
            "field": "parameter",
            "full_type": "u32",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-ir::RaceBody": {
      "fields": [
        {
          "collection": true,
          "field": "branches",
          "full_type": "Vec<RaceBranch>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::RaceBody",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::RaceBranch": {
      "fields": [
        {
          "collection": false,
          "field": "ordinal",
          "full_type": "BranchOrdinal",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "child",
          "full_type": "ComputationId",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::RaceBranch",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::Raw": {
      "fields": [
        {
          "collection": false,
          "field": "child",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "delays",
          "full_type": "Vec<i64>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::Raw",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::ResourceUseBinding": {
      "fields": [
        {
          "collection": false,
          "field": "port_no",
          "full_type": "PortNo",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "resource_slot",
          "full_type": "ResourceSlotId",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::ResourceUseBinding",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::RetryBody": {
      "fields": [
        {
          "collection": false,
          "field": "child",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "delays",
          "full_type": "Vec<Duration>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::RetryBody",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::RunParameterDecl": {
      "fields": [
        {
          "collection": false,
          "field": "id",
          "full_type": "RunParameterId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "type_id",
          "full_type": "SemanticTypeId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "value_slot",
          "full_type": "ValueSlotId",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::RunParameterDecl",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::ScopeBody": {
      "fields": [
        {
          "collection": false,
          "field": "child",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "resources",
          "full_type": "Vec<ResourceSlotId>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::ScopeBody",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::SelectBody": {
      "fields": [
        {
          "collection": false,
          "field": "selector",
          "full_type": "ExpressionId",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "cases",
          "full_type": "Vec<SelectCase>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::SelectBody",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::SelectCase": {
      "fields": [
        {
          "collection": false,
          "field": "ordinal",
          "full_type": "BranchOrdinal",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "child",
          "full_type": "ComputationId",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::SelectCase",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::SemanticDataflowAnalysis": {
      "fields": [
        {
          "collection": true,
          "field": "entry_values",
          "full_type": "BTreeMap<u32, BTreeSet<u32>>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "entry_resources",
          "full_type": "BTreeMap<u32, BTreeSet<u32>>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "expression_value_dependencies",
          "full_type": "BTreeMap<u32, BTreeSet<u32>>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "producer_locations",
          "full_type": "BTreeMap<u32, ProducerLocation>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "workflow_exit_values",
          "full_type": "BTreeSet<u32>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::SemanticDataflowAnalysis",
      "source": "crates/af-ir/src/dataflow.rs"
    },
    "af-ir::SemanticIrDocument": {
      "fields": [
        {
          "collection": true,
          "field": "semantic_refs",
          "full_type": "Vec<SemanticVersionRef>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "types",
          "full_type": "Vec<SemanticTypeNode>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "values",
          "full_type": "Vec<ValueSlot>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "expressions",
          "full_type": "Vec<ExpressionDef>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "resources",
          "full_type": "Vec<ResourceSlot>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "contracts",
          "full_type": "Vec<ContractRefEntry>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "run_parameters",
          "full_type": "Vec<RunParameterDecl>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "workflow_outputs",
          "full_type": "Vec<WorkflowOutputDecl>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "computations",
          "full_type": "Vec<ComputationDef>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "root",
          "full_type": "ComputationId",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::SemanticIrDocument",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::SemanticVersionRef": {
      "fields": [
        {
          "collection": false,
          "field": "specification_id",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::SemanticVersionRef",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::SequenceBody": {
      "fields": [
        {
          "collection": true,
          "field": "children",
          "full_type": "Vec<ComputationId>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::SequenceBody",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::SubflowInterface": {
      "fields": [
        {
          "collection": true,
          "field": "inputs",
          "full_type": "Vec<SubflowValuePort>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "outputs",
          "full_type": "Vec<SubflowValuePort>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "resources",
          "full_type": "Vec<SubflowResourcePort>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::SubflowInterface",
      "source": "crates/af-ir/src/subflow.rs"
    },
    "af-ir::SubflowIrUniverse": {
      "fields": [
        {
          "collection": true,
          "field": "targets",
          "full_type": "HashMap<DigestRefValue, SubflowTarget>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::SubflowIrUniverse",
      "source": "crates/af-ir/src/subflow.rs"
    },
    "af-ir::SubflowResourcePort": {
      "fields": [
        {
          "collection": false,
          "field": "port_no",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_resource_slot",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "requirement_type",
          "full_type": "TypeSchemaRef",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::SubflowResourcePort",
      "source": "crates/af-ir/src/subflow.rs"
    },
    "af-ir::SubflowTarget": {
      "fields": [
        {
          "collection": false,
          "field": "digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "interface",
          "full_type": "SubflowInterface",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "dependencies",
          "full_type": "Vec<DigestRefValue>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "canonical_bytes",
          "full_type": "Vec<u8>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::SubflowTarget",
      "source": "crates/af-ir/src/subflow.rs"
    },
    "af-ir::SubflowValuePort": {
      "fields": [
        {
          "collection": false,
          "field": "port_no",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "semantic_type",
          "full_type": "StructuralType",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::SubflowValuePort",
      "source": "crates/af-ir/src/subflow.rs"
    },
    "af-ir::TimeoutBody": {
      "fields": [
        {
          "collection": false,
          "field": "child",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "timeout",
          "full_type": "ExpressionId",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::TimeoutBody",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::Walk": {
      "fields": [
        {
          "collection": true,
          "field": "computations",
          "full_type": "BTreeMap<u32, &'a crate::model::ComputationDef>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "parents",
          "full_type": "BTreeMap<u32, u32>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "dependencies",
          "full_type": "BTreeMap<u32, BTreeSet<u32>>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "producers",
          "full_type": "BTreeMap<u32, ProducerLocation>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::Walk",
      "source": "crates/af-ir/src/dataflow.rs"
    },
    "af-ir::WhileBody": {
      "fields": [
        {
          "collection": false,
          "field": "guard",
          "full_type": "ExpressionId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "body",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "carry",
          "full_type": "Vec<LoopCarryBinding>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "max_iterations",
          "full_type": "u32",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::WhileBody",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-ir::WorkflowOutputDecl": {
      "fields": [
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "expression",
          "full_type": "ExpressionId",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-ir::WorkflowOutputDecl",
      "source": "crates/af-ir/src/model.rs"
    },
    "af-portable::AssertionIdentityState": {
      "fields": [
        {
          "collection": true,
          "field": "active",
          "full_type": "BTreeSet<u32>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "cache",
          "full_type": "BTreeMap<u32, (af_computation::RootExportAssertionIdentity, usize, usize)>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "expansion_work",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "comparison_work",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "visited_count",
          "full_type": "usize",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::AssertionIdentityState",
      "source": "crates/af-portable/src/pipeline.rs"
    },
    "af-portable::AssertionMode": {
      "kind": "enum",
      "owner_type": "af-portable::AssertionMode",
      "source": "crates/af-portable/src/pipeline.rs",
      "variants": {
        "IdentityOnly": [],
        "RootCheck": []
      }
    },
    "af-portable::AssertionRequest": {
      "fields": [
        {
          "collection": false,
          "field": "assertion_id",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "mode",
          "full_type": "AssertionMode",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "trusted_compiler_manifest",
          "full_type": "&'trusted Value",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "identity_state",
          "full_type": "&'state mut AssertionIdentityState",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::AssertionRequest",
      "source": "crates/af-portable/src/pipeline.rs"
    },
    "af-portable::CanonicalPortableBundle": {
      "fields": [
        {
          "collection": false,
          "field": "raw",
          "full_type": "RawPortableBundle",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "bytes",
          "full_type": "Vec<u8>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "limits",
          "full_type": "DecoderLimits",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::CanonicalPortableBundle",
      "source": "crates/af-portable/src/codec.rs"
    },
    "af-portable::CheckedContractProjection": {
      "fields": [
        {
          "collection": false,
          "field": "reference",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "canonical_content_sha256",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "proven_dimension_count",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "all_dimensions_proven",
          "full_type": "bool",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "action_guard_compatibility_count",
          "full_type": "usize",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::CheckedContractProjection",
      "source": "crates/af-portable/src/pipeline.rs"
    },
    "af-portable::CheckedIrProjection": {
      "fields": [
        {
          "collection": false,
          "field": "reference",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "canonical_content_sha256",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "computation_count",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "phase3c_computation_entry_count",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "phase3c_entry_value_count",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "phase3c_entry_resource_count",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "phase3c_producer_count",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "phase3c_workflow_exit_value_count",
          "full_type": "usize",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::CheckedIrProjection",
      "source": "crates/af-portable/src/pipeline.rs"
    },
    "af-portable::ContentReference": {
      "fields": [
        {
          "collection": false,
          "field": "digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "type_schema",
          "full_type": "TypeSchemaRef",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::ContentReference",
      "source": "crates/af-portable/src/pipeline.rs"
    },
    "af-portable::ContentReferenceProjection": {
      "fields": [
        {
          "collection": false,
          "field": "digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "type_schema",
          "full_type": "TypeSchemaRef",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::ContentReferenceProjection",
      "source": "crates/af-portable/src/pipeline.rs"
    },
    "af-portable::ContentValidatedPortableBundle": {
      "fields": [
        {
          "collection": false,
          "field": "referenced",
          "full_type": "ReferenceValidatedPortableBundle",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::ContentValidatedPortableBundle",
      "source": "crates/af-portable/src/validation.rs"
    },
    "af-portable::ContractAsset": {
      "fields": [
        {
          "collection": false,
          "field": "reference",
          "full_type": "ContentReference",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "bytes",
          "full_type": "Vec<u8>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "raw",
          "full_type": "ComponentContract",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::ContractAsset",
      "source": "crates/af-portable/src/pipeline.rs"
    },
    "af-portable::CurrentSource": {
      "fields": [
        {
          "collection": false,
          "field": "specification_id",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "logical_path",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "bytes",
          "full_type": "&'static [u8]",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::CurrentSource",
      "source": "crates/af-portable/src/support.rs"
    },
    "af-portable::DecodedPortableBundle": {
      "fields": [
        {
          "collection": false,
          "field": "0",
          "full_type": "RawPortableBundle",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::DecodedPortableBundle",
      "source": "crates/af-portable/src/codec.rs"
    },
    "af-portable::DecoderLimits": {
      "fields": [
        {
          "collection": false,
          "field": "max_input_bytes",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "max_scalar_bytes",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "max_container_items",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "max_total_nodes",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "max_nesting",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "max_graph_work",
          "full_type": "usize",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::DecoderLimits",
      "source": "crates/af-portable/src/limits.rs"
    },
    "af-portable::EffectCatalogProjection": {
      "fields": [
        {
          "collection": false,
          "field": "specification_id",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "operations",
          "full_type": "Vec<String>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::EffectCatalogProjection",
      "source": "crates/af-portable/src/reconstruction.rs"
    },
    "af-portable::ExecutionCapabilityProjection": {
      "fields": [
        {
          "collection": false,
          "field": "capability",
          "full_type": "ExecutionCapabilityRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "exact_version",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "supported_versions",
          "full_type": "Vec<String>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "attributes",
          "full_type": "Vec<CapabilityAttribute>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::ExecutionCapabilityProjection",
      "source": "crates/af-portable/src/reconstruction.rs"
    },
    "af-portable::FlowSinkProjection": {
      "fields": [
        {
          "collection": false,
          "field": "operation",
          "full_type": "af_effect::EffectOperationRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "decision_capable",
          "full_type": "bool",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::FlowSinkProjection",
      "source": "crates/af-portable/src/reconstruction.rs"
    },
    "af-portable::IrAsset": {
      "fields": [
        {
          "collection": false,
          "field": "reference",
          "full_type": "ContentReference",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "bytes",
          "full_type": "Vec<u8>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "document",
          "full_type": "SemanticIrDocument",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::IrAsset",
      "source": "crates/af-portable/src/pipeline.rs"
    },
    "af-portable::MaybePayload": {
      "kind": "enum",
      "owner_type": "af-portable::MaybePayload",
      "source": "crates/af-portable/src/orders.rs",
      "variants": {
        "Absent": [],
        "Present": [
          {
            "collection": false,
            "field": "0",
            "full_type": "&'a Value",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-portable::ModuleProjection": {
      "fields": [
        {
          "collection": false,
          "field": "specification_id",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "named_types",
          "full_type": "Vec<TypeSchemaRef>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::ModuleProjection",
      "source": "crates/af-portable/src/reconstruction.rs"
    },
    "af-portable::NamedOperationProjection": {
      "fields": [
        {
          "collection": false,
          "field": "catalog",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::NamedOperationProjection",
      "source": "crates/af-portable/src/reconstruction.rs"
    },
    "af-portable::OrderKey": {
      "kind": "enum",
      "owner_type": "af-portable::OrderKey",
      "source": "crates/af-portable/src/orders.rs",
      "variants": {
        "Number": [
          {
            "collection": false,
            "field": "0",
            "full_type": "u64",
            "option_presence": "required"
          }
        ],
        "Text": [
          {
            "collection": false,
            "field": "0",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "Tuple": [
          {
            "collection": true,
            "field": "0",
            "full_type": "Vec<OrderKey>",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-portable::PlainChecksumProjection": {
      "fields": [
        {
          "collection": false,
          "field": "algorithm",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "hex",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::PlainChecksumProjection",
      "source": "crates/af-portable/src/reconstruction.rs"
    },
    "af-portable::PortableAssertionEntry": {
      "fields": [
        {
          "collection": false,
          "field": "id",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "ir_asset",
          "full_type": "TypedContentReference",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "export_contract_asset",
          "full_type": "TypedContentReference",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "assertion",
          "full_type": "PortableRootExportAssertion",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::PortableAssertionEntry",
      "source": "crates/af-portable/src/root_revalidation.rs"
    },
    "af-portable::PortableContextResolverIds": {
      "fields": [
        {
          "collection": false,
          "field": "context_id",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "resolver_id",
          "full_type": "u32",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::PortableContextResolverIds",
      "source": "crates/af-portable/src/root_revalidation.rs"
    },
    "af-portable::PortableEffectOccurrenceIdentity": {
      "fields": [
        {
          "collection": false,
          "field": "caller_semantic_ir_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_contract_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_entry_ordinal",
          "full_type": "u64",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "atom",
          "full_type": "EffectAtom",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "guard",
          "full_type": "EffectGuard",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::PortableEffectOccurrenceIdentity",
      "source": "crates/af-portable/src/root_revalidation.rs"
    },
    "af-portable::PortableFlowSinkBinding": {
      "kind": "enum",
      "owner_type": "af-portable::PortableFlowSinkBinding",
      "source": "crates/af-portable/src/root_revalidation.rs",
      "variants": {
        "EffectOccurrences": [
          {
            "collection": true,
            "field": "0",
            "full_type": "Vec<u64>",
            "option_presence": "required"
          }
        ],
        "OutputValue": [
          {
            "collection": false,
            "field": "0",
            "full_type": "ValueSlotId",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-portable::PortableFlowSourceBinding": {
      "kind": "enum",
      "owner_type": "af-portable::PortableFlowSourceBinding",
      "source": "crates/af-portable/src/root_revalidation.rs",
      "variants": {
        "InputExpression": [
          {
            "collection": false,
            "field": "0",
            "full_type": "ExpressionId",
            "option_presence": "required"
          }
        ],
        "ResourceSlot": [
          {
            "collection": false,
            "field": "0",
            "full_type": "ResourceSlotId",
            "option_presence": "required"
          }
        ],
        "UnknownExternal": []
      }
    },
    "af-portable::PortableProofCodec": {
      "fields": [
        {
          "collection": false,
          "field": "limits",
          "full_type": "DecoderLimits",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::PortableProofCodec",
      "source": "crates/af-portable/src/codec.rs"
    },
    "af-portable::PortableProofError": {
      "fields": [
        {
          "collection": false,
          "field": "stage",
          "full_type": "ValidationStage",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "family",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "path",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "detail",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::PortableProofError",
      "source": "crates/af-portable/src/error.rs"
    },
    "af-portable::PortableRecoveryActionSourceIdentity": {
      "fields": [
        {
          "collection": false,
          "field": "caller_semantic_ir_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_contract_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "forward_entry_ordinal",
          "full_type": "u64",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "action",
          "full_type": "RecoveryActionIdentity",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::PortableRecoveryActionSourceIdentity",
      "source": "crates/af-portable/src/root_revalidation.rs"
    },
    "af-portable::PortableRootEffectMapping": {
      "fields": [
        {
          "collection": false,
          "field": "source",
          "full_type": "PortableEffectOccurrenceIdentity",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "root_resource_slot",
          "full_type": "Option<ResourceSlotId>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "export_atom",
          "full_type": "EffectAtom",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "export_guard",
          "full_type": "EffectGuard",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::PortableRootEffectMapping",
      "source": "crates/af-portable/src/root_revalidation.rs"
    },
    "af-portable::PortableRootExportAssertion": {
      "fields": [
        {
          "collection": false,
          "field": "exact_semantic_ir_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "exact_exported_component_contract_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "compiler_semantics_id",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "proof_context_manifest",
          "full_type": "PortableContextResolverIds",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "input_mappings",
          "full_type": "Vec<PortableRootInputMapping>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "output_mappings",
          "full_type": "Vec<PortableRootOutputMapping>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "resource_mappings",
          "full_type": "Vec<RootResourcePortBinding>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "effect_mappings",
          "full_type": "Vec<PortableRootEffectMapping>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "flow_mappings",
          "full_type": "Vec<PortableRootFlowMapping>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "recovery_mappings",
          "full_type": "Vec<PortableRootRecoveryMapping>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "execution_mappings",
          "full_type": "Vec<RootExecutionCoordinateBinding>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "subflow_dependencies",
          "full_type": "Vec<PortableSubflowProofDependency>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::PortableRootExportAssertion",
      "source": "crates/af-portable/src/root_revalidation.rs"
    },
    "af-portable::PortableRootFlowMapping": {
      "fields": [
        {
          "collection": false,
          "field": "source_node",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_contract_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "flow_mapping_ordinal",
          "full_type": "u64",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source",
          "full_type": "Option<FlowSourceRef>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "sink",
          "full_type": "Option<FlowConstraint>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "source_binding",
          "full_type": "Option<PortableFlowSourceBinding>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "sink_binding",
          "full_type": "Option<PortableFlowSinkBinding>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "export_source",
          "full_type": "Option<FlowSourceRef>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "export_sink",
          "full_type": "Option<FlowConstraint>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "source_effect_partition",
          "full_type": "Option<EffectFlowPartition>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "export_effect_partition",
          "full_type": "Option<EffectFlowPartition>",
          "option_presence": "explicit_none_or_some"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::PortableRootFlowMapping",
      "source": "crates/af-portable/src/root_revalidation.rs"
    },
    "af-portable::PortableRootInputMapping": {
      "fields": [
        {
          "collection": false,
          "field": "export_input_port",
          "full_type": "ComponentInputPortIdentity",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "run_parameter_id",
          "full_type": "RunParameterId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "root_value_slot",
          "full_type": "ValueSlotId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "root_type",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "export_type",
          "full_type": "TypeSchemaRef",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "use_sites",
          "full_type": "Vec<RootInputUseSite>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::PortableRootInputMapping",
      "source": "crates/af-portable/src/root_revalidation.rs"
    },
    "af-portable::PortableRootOutputMapping": {
      "fields": [
        {
          "collection": false,
          "field": "export_output_port",
          "full_type": "ComponentOutputPortIdentity",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "workflow_output_name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "workflow_output_expression",
          "full_type": "ExpressionId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "root_result_type",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "producer_evidence",
          "full_type": "RootOutputProducerEvidence",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::PortableRootOutputMapping",
      "source": "crates/af-portable/src/root_revalidation.rs"
    },
    "af-portable::PortableRootRecoveryMapping": {
      "fields": [
        {
          "collection": false,
          "field": "source",
          "full_type": "PortableRecoveryActionSourceIdentity",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "export_action_identity",
          "full_type": "RecoveryActionIdentity",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "exact_usage_binding",
          "full_type": "RecoveryUsageBinding",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::PortableRootRecoveryMapping",
      "source": "crates/af-portable/src/root_revalidation.rs"
    },
    "af-portable::PortableStaticProofAnalysis": {
      "fields": [
        {
          "collection": false,
          "field": "inputs",
          "full_type": "ReconstructedStaticInputs",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "root",
          "full_type": "RootSummaryProjection",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "recovery",
          "full_type": "RootRecoveryProjection",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "report",
          "full_type": "RootExportProofReport",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "checked_compatibility",
          "full_type": "Option<CheckedRootExportCompatibility>",
          "option_presence": "explicit_none_or_some"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::PortableStaticProofAnalysis",
      "source": "crates/af-portable/src/pipeline.rs"
    },
    "af-portable::PortableSubflowProofDependency": {
      "fields": [
        {
          "collection": false,
          "field": "call_site",
          "full_type": "ComputationId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "target_semantic_ir_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "target_exported_contract_digest",
          "full_type": "Option<DigestRefValue>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "target_root_assertion_id",
          "full_type": "Option<u32>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": true,
          "field": "transitive_targets",
          "full_type": "Vec<SubflowTargetIdentity>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::PortableSubflowProofDependency",
      "source": "crates/af-portable/src/root_revalidation.rs"
    },
    "af-portable::PortableUsageAggregation": {
      "kind": "enum",
      "owner_type": "af-portable::PortableUsageAggregation",
      "source": "crates/af-portable/src/reconstruction.rs",
      "variants": {
        "Additive": [],
        "Unsupported": []
      }
    },
    "af-portable::ProjectedAssertion": {
      "fields": [
        {
          "collection": false,
          "field": "assertion",
          "full_type": "RootExportAssertion",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "expanded_identity_work",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "comparison_work",
          "full_type": "usize",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::ProjectedAssertion",
      "source": "crates/af-portable/src/root_revalidation.rs"
    },
    "af-portable::RawPortableBundle": {
      "fields": [
        {
          "collection": false,
          "field": "value",
          "full_type": "Value",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "limits",
          "full_type": "DecoderLimits",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::RawPortableBundle",
      "source": "crates/af-portable/src/codec.rs"
    },
    "af-portable::ReconstructedStaticContext": {
      "fields": [
        {
          "collection": false,
          "field": "context_id",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "modules",
          "full_type": "CompiledModuleSet",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "schema_universe",
          "full_type": "TypeSchemaUniverse",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "effects",
          "full_type": "EffectCatalogUniverse",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "resources",
          "full_type": "ResourceTypeUniverse",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "flow",
          "full_type": "FlowCatalogs",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "usage_meters",
          "full_type": "UsageMeterUniverse",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "recovery",
          "full_type": "RecoveryOperationUniverse",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "execution",
          "full_type": "ExecutionUniverses",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "relations",
          "full_type": "RelationSnapshots",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "projection",
          "full_type": "StaticContextProjection",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::ReconstructedStaticContext",
      "source": "crates/af-portable/src/reconstruction.rs"
    },
    "af-portable::ReconstructedStaticInputs": {
      "fields": [
        {
          "collection": false,
          "field": "context",
          "full_type": "StaticContextProjection",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "resolver_identity",
          "full_type": "ResolverSnapshotIdentity",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "checked_contracts",
          "full_type": "Vec<CheckedContractProjection>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "checked_irs",
          "full_type": "Vec<CheckedIrProjection>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "source_associations",
          "full_type": "Vec<SourceAssociationProjection>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "resolver_targets",
          "full_type": "Vec<ResolverTargetProjection>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "await_leaf_count",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "await_unknown_external_count",
          "full_type": "usize",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::ReconstructedStaticInputs",
      "source": "crates/af-portable/src/pipeline.rs"
    },
    "af-portable::ReconstructionOutput": {
      "fields": [
        {
          "collection": false,
          "field": "inputs",
          "full_type": "ReconstructedStaticInputs",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "assertion_identity",
          "full_type": "Option<af_computation::RootExportAssertionIdentity>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "assertion_identity_work",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "assertion_comparison_work",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "root_analysis",
          "full_type": "Option<PortableStaticProofAnalysis>",
          "option_presence": "explicit_none_or_some"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::ReconstructionOutput",
      "source": "crates/af-portable/src/pipeline.rs"
    },
    "af-portable::ReferenceValidatedPortableBundle": {
      "fields": [
        {
          "collection": false,
          "field": "canonical",
          "full_type": "CanonicalPortableBundle",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::ReferenceValidatedPortableBundle",
      "source": "crates/af-portable/src/validation.rs"
    },
    "af-portable::RegionCompatibilityProjection": {
      "fields": [
        {
          "collection": false,
          "field": "from",
          "full_type": "RegionRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "to",
          "full_type": "RegionRef",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::RegionCompatibilityProjection",
      "source": "crates/af-portable/src/reconstruction.rs"
    },
    "af-portable::RelationProjection": {
      "fields": [
        {
          "collection": false,
          "field": "schema",
          "full_type": "TypeSchemaRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "revision",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "canonical_content_checksum",
          "full_type": "PlainChecksumProjection",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "canonical_content",
          "full_type": "Vec<u8>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::RelationProjection",
      "source": "crates/af-portable/src/reconstruction.rs"
    },
    "af-portable::ResolvedSubflowAssociation": {
      "fields": [
        {
          "collection": false,
          "field": "target_semantic_ir_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "exported_contract_digest",
          "full_type": "Option<DigestRefValue>",
          "option_presence": "explicit_none_or_some"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::ResolvedSubflowAssociation",
      "source": "crates/af-portable/src/pipeline.rs"
    },
    "af-portable::ResolverManifest": {
      "fields": [
        {
          "collection": false,
          "field": "caller_ir",
          "full_type": "ContentReference",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "context_id",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "components",
          "full_type": "Vec<ContentReference>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "subflow_targets",
          "full_type": "Vec<ResolverTargetManifest>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "source_associations",
          "full_type": "Vec<SourceAssociationManifest>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::ResolverManifest",
      "source": "crates/af-portable/src/pipeline.rs"
    },
    "af-portable::ResolverTargetManifest": {
      "fields": [
        {
          "collection": false,
          "field": "target_semantic_ir_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "target_ir_content",
          "full_type": "Option<ContentReference>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "exported_contract",
          "full_type": "Option<ContentReference>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": true,
          "field": "direct_dependencies",
          "full_type": "Option<Vec<DigestRefValue>>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": true,
          "field": "transitive_dependencies",
          "full_type": "Option<Vec<SubflowTargetIdentity>>",
          "option_presence": "explicit_none_or_some"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::ResolverTargetManifest",
      "source": "crates/af-portable/src/pipeline.rs"
    },
    "af-portable::ResolverTargetProjection": {
      "fields": [
        {
          "collection": false,
          "field": "semantic_ir_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "exported_contract_digest",
          "full_type": "Option<DigestRefValue>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": true,
          "field": "direct_dependencies",
          "full_type": "Vec<DigestRefValue>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "transitive_dependencies",
          "full_type": "Vec<SubflowTargetIdentityProjection>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::ResolverTargetProjection",
      "source": "crates/af-portable/src/pipeline.rs"
    },
    "af-portable::ResourceDescriptorProjection": {
      "fields": [
        {
          "collection": false,
          "field": "schema",
          "full_type": "TypeSchemaRef",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "recognized_attributes",
          "full_type": "Vec<String>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::ResourceDescriptorProjection",
      "source": "crates/af-portable/src/reconstruction.rs"
    },
    "af-portable::RootRecoveryProjection": {
      "fields": [
        {
          "collection": false,
          "field": "value",
          "full_type": "serde_json::Value",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::RootRecoveryProjection",
      "source": "crates/af-portable/src/pipeline.rs"
    },
    "af-portable::RootSummaryProjection": {
      "fields": [
        {
          "collection": false,
          "field": "value",
          "full_type": "serde_json::Value",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::RootSummaryProjection",
      "source": "crates/af-portable/src/pipeline.rs"
    },
    "af-portable::RuntimeProfileProjection": {
      "fields": [
        {
          "collection": false,
          "field": "profile",
          "full_type": "RuntimeProfileRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "family",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "features",
          "full_type": "Vec<String>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::RuntimeProfileProjection",
      "source": "crates/af-portable/src/reconstruction.rs"
    },
    "af-portable::Scan": {
      "fields": [
        {
          "collection": false,
          "field": "bytes",
          "full_type": "&'a [u8]",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "pos",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "nodes",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "limits",
          "full_type": "DecoderLimits",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::Scan",
      "source": "crates/af-portable/src/limits.rs"
    },
    "af-portable::SetRule": {
      "fields": [
        {
          "collection": false,
          "field": "owner",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "field",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "key",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "element_type",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "complete_collection",
          "full_type": "bool",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::SetRule",
      "source": "crates/af-portable/src/orders.rs"
    },
    "af-portable::SourceAssociationManifest": {
      "fields": [
        {
          "collection": false,
          "field": "caller_ir",
          "full_type": "ContentReference",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_node",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "target",
          "full_type": "SourceAssociationTarget",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::SourceAssociationManifest",
      "source": "crates/af-portable/src/pipeline.rs"
    },
    "af-portable::SourceAssociationProjection": {
      "fields": [
        {
          "collection": false,
          "field": "caller_ir",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_node",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "target",
          "full_type": "SourceAssociationTargetProjection",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::SourceAssociationProjection",
      "source": "crates/af-portable/src/pipeline.rs"
    },
    "af-portable::SourceAssociationTarget": {
      "kind": "enum",
      "owner_type": "af-portable::SourceAssociationTarget",
      "source": "crates/af-portable/src/pipeline.rs",
      "variants": {
        "AwaitSignal": [
          {
            "collection": false,
            "field": "0",
            "full_type": "TypeSchemaRef",
            "option_presence": "required"
          }
        ],
        "CallSubflow": [
          {
            "collection": false,
            "field": "0",
            "full_type": "ResolvedSubflowAssociation",
            "option_presence": "required"
          }
        ],
        "Invoke": [
          {
            "collection": false,
            "field": "0",
            "full_type": "ContentReference",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-portable::SourceAssociationTargetProjection": {
      "kind": "enum",
      "owner_type": "af-portable::SourceAssociationTargetProjection",
      "source": "crates/af-portable/src/pipeline.rs",
      "variants": {
        "AwaitSignal": [
          {
            "collection": false,
            "field": "0",
            "full_type": "TypeSchemaRef",
            "option_presence": "required"
          }
        ],
        "CallSubflow": [
          {
            "collection": false,
            "field": "semantic_ir_digest",
            "full_type": "DigestRefValue",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "exported_contract_digest",
            "full_type": "Option<DigestRefValue>",
            "option_presence": "explicit_none_or_some"
          }
        ],
        "Invoke": [
          {
            "collection": false,
            "field": "0",
            "full_type": "ContentReferenceProjection",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-portable::StaticContextProjection": {
      "fields": [
        {
          "collection": false,
          "field": "context_id",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "modules",
          "full_type": "Vec<ModuleProjection>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "type_schemas",
          "full_type": "Vec<TypeSchemaRef>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "effect_catalogs",
          "full_type": "Vec<EffectCatalogProjection>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "resource_schemas",
          "full_type": "Vec<TypeSchemaRef>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "resource_descriptors",
          "full_type": "Vec<ResourceDescriptorProjection>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "confidentiality_labels",
          "full_type": "Vec<ConfidentialityLabelRef>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "taints",
          "full_type": "Vec<TaintRef>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "flow_effect_sinks",
          "full_type": "Vec<FlowSinkProjection>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "usage_meters",
          "full_type": "Vec<UsageMeterProjection>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "recovery_effect_catalogs",
          "full_type": "Vec<EffectCatalogProjection>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "reconciliation_operations",
          "full_type": "Vec<NamedOperationProjection>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "execution_capabilities",
          "full_type": "Vec<ExecutionCapabilityProjection>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "trust_domains",
          "full_type": "Vec<TrustDomainRef>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "regions",
          "full_type": "Vec<RegionRef>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "region_compatibility",
          "full_type": "Vec<RegionCompatibilityProjection>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "runtime_profiles",
          "full_type": "Vec<RuntimeProfileProjection>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "effect_resource_relation",
          "full_type": "Option<RelationProjection>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "recovery_usage_relation",
          "full_type": "Option<RelationProjection>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "flow_execution_relation",
          "full_type": "Option<RelationProjection>",
          "option_presence": "explicit_none_or_some"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::StaticContextProjection",
      "source": "crates/af-portable/src/reconstruction.rs"
    },
    "af-portable::StructuralTypeBody": {
      "kind": "enum",
      "owner_type": "af-portable::StructuralTypeBody",
      "source": "crates/af-portable/src/root_revalidation.rs",
      "variants": {
        "List": [
          {
            "collection": false,
            "field": "0",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "MapString": [
          {
            "collection": false,
            "field": "0",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "Option": [
          {
            "collection": false,
            "field": "0",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "Primitive": [
          {
            "collection": false,
            "field": "0",
            "full_type": "SemanticPrimitiveType",
            "option_presence": "required"
          }
        ],
        "Schema": [
          {
            "collection": false,
            "field": "0",
            "full_type": "TypeSchemaRef",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-portable::StructuralTypeNode": {
      "fields": [
        {
          "collection": false,
          "field": "id",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "body",
          "full_type": "StructuralTypeBody",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::StructuralTypeNode",
      "source": "crates/af-portable/src/root_revalidation.rs"
    },
    "af-portable::SubflowTargetIdentity": {
      "fields": [
        {
          "collection": false,
          "field": "semantic_ir_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "exported_contract_digest",
          "full_type": "Option<DigestRefValue>",
          "option_presence": "explicit_none_or_some"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::SubflowTargetIdentity",
      "source": "crates/af-portable/src/pipeline.rs"
    },
    "af-portable::SubflowTargetIdentityProjection": {
      "fields": [
        {
          "collection": false,
          "field": "semantic_ir_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "exported_contract_digest",
          "full_type": "Option<DigestRefValue>",
          "option_presence": "explicit_none_or_some"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::SubflowTargetIdentityProjection",
      "source": "crates/af-portable/src/pipeline.rs"
    },
    "af-portable::SupportedPortableStaticInputs": {
      "fields": [
        {
          "collection": false,
          "field": "content",
          "full_type": "ContentValidatedPortableBundle",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "semantic_version",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "supported_schema_type_count",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "supported_source_count",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "applied_rule_count",
          "full_type": "usize",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::SupportedPortableStaticInputs",
      "source": "crates/af-portable/src/support.rs"
    },
    "af-portable::TypedContentReference": {
      "fields": [
        {
          "collection": false,
          "field": "digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "type_schema",
          "full_type": "TypeSchemaRef",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::TypedContentReference",
      "source": "crates/af-portable/src/root_revalidation.rs"
    },
    "af-portable::UsageMeterProjection": {
      "fields": [
        {
          "collection": false,
          "field": "meter",
          "full_type": "UsageMeterRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "unit",
          "full_type": "UsageUnitProjection",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "aggregation",
          "full_type": "PortableUsageAggregation",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::UsageMeterProjection",
      "source": "crates/af-portable/src/reconstruction.rs"
    },
    "af-portable::UsageUnitProjection": {
      "fields": [
        {
          "collection": false,
          "field": "catalog",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-portable::UsageUnitProjection",
      "source": "crates/af-portable/src/reconstruction.rs"
    },
    "af-portable::ValidationStage": {
      "kind": "enum",
      "owner_type": "af-portable::ValidationStage",
      "source": "crates/af-portable/src/error.rs",
      "variants": {
        "D0": [],
        "D1": [],
        "D10": [],
        "D2": [],
        "D3": [],
        "D4": [],
        "D5": [],
        "D6": [],
        "D7": [],
        "D8": [],
        "D9": []
      }
    },
    "af-recovery::CompensationRequirement": {
      "kind": "enum",
      "owner_type": "af-recovery::CompensationRequirement",
      "source": "crates/af-recovery/src/model.rs",
      "variants": {
        "None": [],
        "Required": [
          {
            "collection": false,
            "field": "0",
            "full_type": "EffectOperationRef",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-recovery::EffectRecoveryClause": {
      "fields": [
        {
          "collection": false,
          "field": "effect",
          "full_type": "EffectAtom",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "idempotency",
          "full_type": "IdempotencySemantics",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "key_requirement",
          "full_type": "Option<KeyStabilityRequirement>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "reconciliation",
          "full_type": "ReconciliationRequirement",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "compensation",
          "full_type": "CompensationRequirement",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "unknown_effect_handling",
          "full_type": "UnknownEffectHandling",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-recovery::EffectRecoveryClause",
      "source": "crates/af-recovery/src/model.rs"
    },
    "af-recovery::IdempotencySemantics": {
      "kind": "enum",
      "owner_type": "af-recovery::IdempotencySemantics",
      "source": "crates/af-recovery/src/model.rs",
      "variants": {
        "Keyed": [],
        "Natural": [],
        "None": [],
        "Reconciled": []
      }
    },
    "af-recovery::KeyStabilityRequirement": {
      "fields": [
        {
          "collection": false,
          "field": "stable_across_attempts",
          "full_type": "bool",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "distinct_across_invocations",
          "full_type": "bool",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "bound_to_operation",
          "full_type": "bool",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "provider_honors",
          "full_type": "bool",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-recovery::KeyStabilityRequirement",
      "source": "crates/af-recovery/src/model.rs"
    },
    "af-recovery::ReconciliationRequirement": {
      "kind": "enum",
      "owner_type": "af-recovery::ReconciliationRequirement",
      "source": "crates/af-recovery/src/model.rs",
      "variants": {
        "None": [],
        "Required": []
      }
    },
    "af-recovery::RecoveryCompatibilityProof": {
      "kind": "enum",
      "owner_type": "af-recovery::RecoveryCompatibilityProof",
      "source": "crates/af-recovery/src/compatibility.rs",
      "variants": {
        "Proven": [],
        "Unproven": [],
        "Weakened": []
      }
    },
    "af-recovery::RecoveryContract": {
      "fields": [
        {
          "collection": false,
          "field": "replay_semantics",
          "full_type": "ReplaySemantics",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "effect_recovery_clauses",
          "full_type": "Vec<EffectRecoveryClause>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-recovery::RecoveryContract",
      "source": "crates/af-recovery/src/model.rs"
    },
    "af-recovery::RecoveryError": {
      "fields": [
        {
          "collection": false,
          "field": "kind",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "detail",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-recovery::RecoveryError",
      "source": "crates/af-recovery/src/error.rs"
    },
    "af-recovery::RecoveryEvidenceState": {
      "kind": "enum",
      "owner_type": "af-recovery::RecoveryEvidenceState",
      "source": "crates/af-recovery/src/obligations.rs",
      "variants": {
        "Applied": [],
        "NotApplied": [],
        "PartiallyApplied": [],
        "StillUnknown": [],
        "Unknown": []
      }
    },
    "af-recovery::RecoveryObligation": {
      "kind": "enum",
      "owner_type": "af-recovery::RecoveryObligation",
      "source": "crates/af-recovery/src/obligations.rs",
      "variants": {
        "DurableRecordedResult": [],
        "IdempotencyEvidence": [],
        "KeyStabilityEvidence": [],
        "PartialStateResolution": [],
        "ReconciliationEvidence": []
      }
    },
    "af-recovery::RecoveryOperationUniverse": {
      "fields": [
        {
          "collection": false,
          "field": "effects",
          "full_type": "EffectCatalogUniverse",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "reconciliation_operations",
          "full_type": "BTreeSet<(String, String)>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-recovery::RecoveryOperationUniverse",
      "source": "crates/af-recovery/src/universe.rs"
    },
    "af-recovery::RecoveryProof": {
      "kind": "enum",
      "owner_type": "af-recovery::RecoveryProof",
      "source": "crates/af-recovery/src/obligations.rs",
      "variants": {
        "ProvenStaticConstraint": [],
        "RequiresEvidence": [
          {
            "collection": true,
            "field": "0",
            "full_type": "Vec<RecoveryObligation>",
            "option_presence": "required"
          }
        ],
        "Unproven": [
          {
            "collection": false,
            "field": "0",
            "full_type": "RecoveryUnprovenReason",
            "option_presence": "required"
          }
        ],
        "Violation": [
          {
            "collection": false,
            "field": "0",
            "full_type": "RecoveryViolation",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-recovery::RecoveryUnprovenReason": {
      "kind": "enum",
      "owner_type": "af-recovery::RecoveryUnprovenReason",
      "source": "crates/af-recovery/src/obligations.rs",
      "variants": {
        "HistoricalReplayNeedsEvidence": [],
        "MissingClause": [],
        "UnknownEffectCoverage": []
      }
    },
    "af-recovery::RecoveryViolation": {
      "kind": "enum",
      "owner_type": "af-recovery::RecoveryViolation",
      "source": "crates/af-recovery/src/obligations.rs",
      "variants": {
        "EffectQualifierConflict": [],
        "PartiallyAppliedUnresolved": [],
        "UnsafeBlindRetry": []
      }
    },
    "af-recovery::ReplaySemantics": {
      "kind": "enum",
      "owner_type": "af-recovery::ReplaySemantics",
      "source": "crates/af-recovery/src/model.rs",
      "variants": {
        "Forbidden": [],
        "Recompute": [],
        "Reconcile": [],
        "Recorded": []
      }
    },
    "af-recovery::UnknownEffectHandling": {
      "kind": "enum",
      "owner_type": "af-recovery::UnknownEffectHandling",
      "source": "crates/af-recovery/src/model.rs",
      "variants": {
        "Block": [],
        "ReconciliationRequired": []
      }
    },
    "af-resource::ResourceAttributeConstraint": {
      "fields": [
        {
          "collection": false,
          "field": "key",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "value",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-resource::ResourceAttributeConstraint",
      "source": "crates/af-resource/src/model.rs"
    },
    "af-resource::ResourceCandidate": {
      "fields": [
        {
          "collection": false,
          "field": "resource_type",
          "full_type": "ResourceTypeRef",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "attributes",
          "full_type": "Vec<ResourceAttributeConstraint>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-resource::ResourceCandidate",
      "source": "crates/af-resource/src/compatibility.rs"
    },
    "af-resource::ResourceCompatibilityProof": {
      "kind": "enum",
      "owner_type": "af-resource::ResourceCompatibilityProof",
      "source": "crates/af-resource/src/compatibility.rs",
      "variants": {
        "Proven": [],
        "Unproven": [
          {
            "collection": false,
            "field": "0",
            "full_type": "UnprovenReason",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-resource::ResourceError": {
      "fields": [
        {
          "collection": false,
          "field": "kind",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "detail",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-resource::ResourceError",
      "source": "crates/af-resource/src/error.rs"
    },
    "af-resource::ResourceRequirement": {
      "fields": [
        {
          "collection": false,
          "field": "port_no",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "resource_type",
          "full_type": "ResourceTypeRef",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "required_attributes",
          "full_type": "Vec<ResourceAttributeConstraint>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-resource::ResourceRequirement",
      "source": "crates/af-resource/src/model.rs"
    },
    "af-resource::ResourceRequirementRow": {
      "fields": [
        {
          "collection": true,
          "field": "requirements",
          "full_type": "Vec<ResourceRequirement>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-resource::ResourceRequirementRow",
      "source": "crates/af-resource/src/model.rs"
    },
    "af-resource::ResourceTypeDescriptor": {
      "fields": [
        {
          "collection": false,
          "field": "specification_id",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "type_name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "schema_hex",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "recognized_attributes",
          "full_type": "BTreeSet<String>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-resource::ResourceTypeDescriptor",
      "source": "crates/af-resource/src/universe.rs"
    },
    "af-resource::ResourceTypeRef": {
      "fields": [
        {
          "collection": false,
          "field": "schema",
          "full_type": "TypeSchemaRef",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-resource::ResourceTypeRef",
      "source": "crates/af-resource/src/model.rs"
    },
    "af-resource::ResourceTypeUniverse": {
      "fields": [
        {
          "collection": false,
          "field": "schemas",
          "full_type": "af_semantic::TypeSchemaUniverse",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "descriptors",
          "full_type": "BTreeMap<(String, String), ResourceTypeDescriptor>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-resource::ResourceTypeUniverse",
      "source": "crates/af-resource/src/universe.rs"
    },
    "af-resource::UnprovenReason": {
      "kind": "enum",
      "owner_type": "af-resource::UnprovenReason",
      "source": "crates/af-resource/src/compatibility.rs",
      "variants": {
        "RetainedPortResourceTypeChanged": [],
        "UnknownAttributeSemantics": []
      }
    },
    "af-semantic::CheckedDocument": {
      "fields": [
        {
          "collection": false,
          "field": "source",
          "full_type": "&'a SemanticCoreDocument",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "universe",
          "full_type": "&'a TypeSchemaUniverse",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "resolved_types",
          "full_type": "BTreeMap<SemanticTypeId, StructuralType>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "slot_types",
          "full_type": "BTreeMap<ValueSlotId, StructuralType>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "expression_types",
          "full_type": "BTreeMap<ExpressionId, StructuralType>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "canonical_expressions",
          "full_type": "Vec<ExpressionDef>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "canonical_types",
          "full_type": "Vec<SemanticTypeNode>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-semantic::CheckedDocument",
      "source": "crates/af-semantic/src/document.rs"
    },
    "af-semantic::Checker": {
      "fields": [
        {
          "collection": false,
          "field": "table",
          "full_type": "&'a ExprTable",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "universe",
          "full_type": "&'a TypeSchemaUniverse",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "resolved_types",
          "full_type": "&'a BTreeMap<SemanticTypeId, StructuralType>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "slot_types",
          "full_type": "&'a BTreeMap<ValueSlotId, StructuralType>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "memo",
          "full_type": "BTreeMap<ExpressionId, StructuralType>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-semantic::Checker",
      "source": "crates/af-semantic/src/document.rs"
    },
    "af-semantic::DigestRefValue": {
      "fields": [
        {
          "collection": false,
          "field": "algorithm",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "domain",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "hex",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-semantic::DigestRefValue",
      "source": "crates/af-semantic/src/types.rs"
    },
    "af-semantic::ExprTable": {
      "fields": [
        {
          "collection": true,
          "field": "defs",
          "full_type": "BTreeMap<ExpressionId, ExpressionDef>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "duplicates",
          "full_type": "Vec<ExpressionId>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-semantic::ExprTable",
      "source": "crates/af-semantic/src/expressions.rs"
    },
    "af-semantic::ExpressionBody": {
      "kind": "enum",
      "owner_type": "af-semantic::ExpressionBody",
      "source": "crates/af-semantic/src/expressions.rs",
      "variants": {
        "And": [
          {
            "collection": false,
            "field": "left",
            "full_type": "ExpressionId",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "right",
            "full_type": "ExpressionId",
            "option_presence": "required"
          }
        ],
        "Constant": [
          {
            "collection": false,
            "field": "0",
            "full_type": "PrimitiveConstant",
            "option_presence": "required"
          }
        ],
        "EnumValue": [
          {
            "collection": false,
            "field": "schema",
            "full_type": "TypeSchemaRef",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "value_no",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "Equals": [
          {
            "collection": false,
            "field": "left",
            "full_type": "ExpressionId",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "right",
            "full_type": "ExpressionId",
            "option_presence": "required"
          }
        ],
        "FieldGet": [
          {
            "collection": false,
            "field": "record",
            "full_type": "ExpressionId",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "field_no",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "IfThenElse": [
          {
            "collection": false,
            "field": "condition",
            "full_type": "ExpressionId",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "then_expr",
            "full_type": "ExpressionId",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "else_expr",
            "full_type": "ExpressionId",
            "option_presence": "required"
          }
        ],
        "ListConstruct": [
          {
            "collection": false,
            "field": "element_type",
            "full_type": "crate::ids::SemanticTypeId",
            "option_presence": "required"
          },
          {
            "collection": true,
            "field": "elements",
            "full_type": "Vec<ExpressionId>",
            "option_presence": "required"
          }
        ],
        "MapConstruct": [
          {
            "collection": false,
            "field": "value_type",
            "full_type": "crate::ids::SemanticTypeId",
            "option_presence": "required"
          },
          {
            "collection": true,
            "field": "entries",
            "full_type": "Vec<MapEntryExpr>",
            "option_presence": "required"
          }
        ],
        "Not": [
          {
            "collection": false,
            "field": "operand",
            "full_type": "ExpressionId",
            "option_presence": "required"
          }
        ],
        "OptionNone": [
          {
            "collection": false,
            "field": "inner_type",
            "full_type": "crate::ids::SemanticTypeId",
            "option_presence": "required"
          }
        ],
        "OptionSome": [
          {
            "collection": false,
            "field": "payload_expr",
            "full_type": "ExpressionId",
            "option_presence": "required"
          }
        ],
        "Or": [
          {
            "collection": false,
            "field": "left",
            "full_type": "ExpressionId",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "right",
            "full_type": "ExpressionId",
            "option_presence": "required"
          }
        ],
        "RecordConstruct": [
          {
            "collection": false,
            "field": "schema",
            "full_type": "TypeSchemaRef",
            "option_presence": "required"
          },
          {
            "collection": true,
            "field": "fields",
            "full_type": "Vec<RecordFieldExpr>",
            "option_presence": "required"
          }
        ],
        "ValueSlot": [
          {
            "collection": false,
            "field": "0",
            "full_type": "ValueSlotId",
            "option_presence": "required"
          }
        ],
        "VariantConstruct": [
          {
            "collection": false,
            "field": "schema",
            "full_type": "TypeSchemaRef",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "case_no",
            "full_type": "u32",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "payload_expr",
            "full_type": "ExpressionId",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-semantic::ExpressionDef": {
      "fields": [
        {
          "collection": false,
          "field": "id",
          "full_type": "ExpressionId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "result_type_id",
          "full_type": "crate::ids::SemanticTypeId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "body",
          "full_type": "ExpressionBody",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-semantic::ExpressionDef",
      "source": "crates/af-semantic/src/expressions.rs"
    },
    "af-semantic::MapEntryExpr": {
      "fields": [
        {
          "collection": false,
          "field": "key",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "value_expr",
          "full_type": "ExpressionId",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-semantic::MapEntryExpr",
      "source": "crates/af-semantic/src/expressions.rs"
    },
    "af-semantic::Mark": {
      "kind": "enum",
      "owner_type": "af-semantic::Mark",
      "source": "crates/af-semantic/src/expressions.rs",
      "variants": {
        "Done": [],
        "InProgress": []
      }
    },
    "af-semantic::PrimitiveConstant": {
      "kind": "enum",
      "owner_type": "af-semantic::PrimitiveConstant",
      "source": "crates/af-semantic/src/values.rs",
      "variants": {
        "Bool": [
          {
            "collection": false,
            "field": "0",
            "full_type": "bool",
            "option_presence": "required"
          }
        ],
        "Bytes": [
          {
            "collection": false,
            "field": "0",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "Decimal": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Decimal",
            "option_presence": "required"
          }
        ],
        "Duration": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Duration",
            "option_presence": "required"
          }
        ],
        "Int64": [
          {
            "collection": false,
            "field": "0",
            "full_type": "i64",
            "option_presence": "required"
          }
        ],
        "Null": [],
        "String": [
          {
            "collection": false,
            "field": "0",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "Timestamp": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Timestamp",
            "option_presence": "required"
          }
        ],
        "Unit": []
      }
    },
    "af-semantic::Raw": {
      "kind": "enum",
      "owner_type": "af-semantic::Raw",
      "source": "crates/af-semantic/src/values.rs",
      "variants": {
        "Bool": [
          {
            "collection": false,
            "field": "0",
            "full_type": "bool",
            "option_presence": "required"
          }
        ],
        "Bytes": [
          {
            "collection": false,
            "field": "0",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "Decimal": [
          {
            "collection": false,
            "field": "0",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "Duration": [
          {
            "collection": false,
            "field": "0",
            "full_type": "i64",
            "option_presence": "required"
          }
        ],
        "Int64": [
          {
            "collection": false,
            "field": "0",
            "full_type": "i64",
            "option_presence": "required"
          }
        ],
        "Null": [
          {
            "collection": false,
            "field": "0",
            "full_type": "()",
            "option_presence": "required"
          }
        ],
        "String": [
          {
            "collection": false,
            "field": "0",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "Timestamp": [
          {
            "collection": false,
            "field": "0",
            "full_type": "i64",
            "option_presence": "required"
          }
        ],
        "Unit": [
          {
            "collection": false,
            "field": "0",
            "full_type": "()",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-semantic::RecordFieldExpr": {
      "fields": [
        {
          "collection": false,
          "field": "field_no",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "expr_id",
          "full_type": "ExpressionId",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-semantic::RecordFieldExpr",
      "source": "crates/af-semantic/src/expressions.rs"
    },
    "af-semantic::ResourceSlot": {
      "fields": [
        {
          "collection": false,
          "field": "id",
          "full_type": "ResourceSlotId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "requirement_type",
          "full_type": "TypeSchemaRef",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-semantic::ResourceSlot",
      "source": "crates/af-semantic/src/resources.rs"
    },
    "af-semantic::SchemaEntry": {
      "fields": [
        {
          "collection": false,
          "field": "specification_id",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "type_name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "digest",
          "full_type": "af_core::DigestRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "body",
          "full_type": "CanonicalBody",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-semantic::SchemaEntry",
      "source": "crates/af-semantic/src/universe.rs"
    },
    "af-semantic::SemanticCoreDocument": {
      "fields": [
        {
          "collection": true,
          "field": "types",
          "full_type": "Vec<SemanticTypeNode>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "values",
          "full_type": "Vec<ValueSlot>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "expressions",
          "full_type": "Vec<ExpressionDef>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "resources",
          "full_type": "Vec<ResourceSlot>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-semantic::SemanticCoreDocument",
      "source": "crates/af-semantic/src/document.rs"
    },
    "af-semantic::SemanticError": {
      "kind": "enum",
      "owner_type": "af-semantic::SemanticError",
      "source": "crates/af-semantic/src/error.rs",
      "variants": {
        "DuplicateExpressionId": [
          {
            "collection": false,
            "field": "id",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "DuplicateFieldBinding": [
          {
            "collection": false,
            "field": "identity",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "field_no",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "DuplicateNormalizedMapKey": [
          {
            "collection": false,
            "field": "key",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "DuplicateResourceSlotId": [
          {
            "collection": false,
            "field": "id",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "DuplicateSemanticTypeId": [
          {
            "collection": false,
            "field": "id",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "DuplicateValueSlotId": [
          {
            "collection": false,
            "field": "id",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "ExpectedEnumSchema": [
          {
            "collection": false,
            "field": "identity",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "ExpectedRecordSchema": [
          {
            "collection": false,
            "field": "identity",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "ExpectedVariantSchema": [
          {
            "collection": false,
            "field": "identity",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "ExpressionCycle": [
          {
            "collection": false,
            "field": "id",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "ExpressionResultTypeMismatch": [
          {
            "collection": false,
            "field": "expression",
            "full_type": "u32",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "declared",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "derived",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "InvalidConstant": [
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "InvalidDigestRef": [
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "InvalidLocalId": [
          {
            "collection": false,
            "field": "namespace",
            "full_type": "&'static str",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "id",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "MissingFieldBinding": [
          {
            "collection": false,
            "field": "identity",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "field_no",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "SemanticTypeCycle": [
          {
            "collection": false,
            "field": "id",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "TypeMismatch": [
          {
            "collection": false,
            "field": "context",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "expected",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "actual",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "TypeNotComparable": [
          {
            "collection": false,
            "field": "type_repr",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "TypeSchemaDigestMismatch": [
          {
            "collection": false,
            "field": "identity",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "expected",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "actual",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "UnknownEnumValue": [
          {
            "collection": false,
            "field": "identity",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "value_no",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "UnknownExpression": [
          {
            "collection": false,
            "field": "id",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "UnknownFieldNumber": [
          {
            "collection": false,
            "field": "identity",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "field_no",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "UnknownSemanticType": [
          {
            "collection": false,
            "field": "id",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "UnknownTypeSchema": [
          {
            "collection": false,
            "field": "identity",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "UnknownValueSlot": [
          {
            "collection": false,
            "field": "id",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "UnknownVariantCase": [
          {
            "collection": false,
            "field": "identity",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "case_no",
            "full_type": "u32",
            "option_presence": "required"
          }
        ],
        "UnsupportedWorkflowPrimitive": [
          {
            "collection": false,
            "field": "keyword",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "ValueSlotNotBound": [
          {
            "collection": false,
            "field": "id",
            "full_type": "u32",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-semantic::SemanticPrimitiveType": {
      "kind": "enum",
      "owner_type": "af-semantic::SemanticPrimitiveType",
      "source": "crates/af-semantic/src/types.rs",
      "variants": {
        "Bool": [],
        "Bytes": [],
        "Decimal": [],
        "Duration": [],
        "Int64": [],
        "Null": [],
        "String": [],
        "Timestamp": [],
        "Unit": []
      }
    },
    "af-semantic::SemanticTypeNode": {
      "fields": [
        {
          "collection": false,
          "field": "id",
          "full_type": "SemanticTypeId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "body",
          "full_type": "SemanticTypeNodeBody",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-semantic::SemanticTypeNode",
      "source": "crates/af-semantic/src/types.rs"
    },
    "af-semantic::SemanticTypeNodeBody": {
      "kind": "enum",
      "owner_type": "af-semantic::SemanticTypeNodeBody",
      "source": "crates/af-semantic/src/types.rs",
      "variants": {
        "List": [
          {
            "collection": false,
            "field": "0",
            "full_type": "SemanticTypeId",
            "option_presence": "required"
          }
        ],
        "MapString": [
          {
            "collection": false,
            "field": "0",
            "full_type": "SemanticTypeId",
            "option_presence": "required"
          }
        ],
        "Option": [
          {
            "collection": false,
            "field": "0",
            "full_type": "SemanticTypeId",
            "option_presence": "required"
          }
        ],
        "Primitive": [
          {
            "collection": false,
            "field": "0",
            "full_type": "SemanticPrimitiveType",
            "option_presence": "required"
          }
        ],
        "Schema": [
          {
            "collection": false,
            "field": "0",
            "full_type": "TypeSchemaRef",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-semantic::Status": {
      "kind": "enum",
      "owner_type": "af-semantic::Status",
      "source": "crates/af-semantic/src/universe.rs",
      "variants": {
        "1": []
      }
    },
    "af-semantic::StructuralType": {
      "kind": "enum",
      "owner_type": "af-semantic::StructuralType",
      "source": "crates/af-semantic/src/types.rs",
      "variants": {
        "List": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Box<StructuralType>",
            "option_presence": "required"
          }
        ],
        "MapString": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Box<StructuralType>",
            "option_presence": "required"
          }
        ],
        "Option": [
          {
            "collection": false,
            "field": "0",
            "full_type": "Box<StructuralType>",
            "option_presence": "required"
          }
        ],
        "Primitive": [
          {
            "collection": false,
            "field": "0",
            "full_type": "SemanticPrimitiveType",
            "option_presence": "required"
          }
        ],
        "Schema": [
          {
            "collection": false,
            "field": "0",
            "full_type": "TypeSchemaRef",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-semantic::TypeSchemaRef": {
      "fields": [
        {
          "collection": false,
          "field": "specification_id",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "type_name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "schema_digest",
          "full_type": "DigestRefValue",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-semantic::TypeSchemaRef",
      "source": "crates/af-semantic/src/types.rs"
    },
    "af-semantic::TypeSchemaUniverse": {
      "fields": [
        {
          "collection": true,
          "field": "entries",
          "full_type": "BTreeMap<String, SchemaEntry>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-semantic::TypeSchemaUniverse",
      "source": "crates/af-semantic/src/universe.rs"
    },
    "af-semantic::ValueSlot": {
      "fields": [
        {
          "collection": false,
          "field": "id",
          "full_type": "ValueSlotId",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "type_id",
          "full_type": "SemanticTypeId",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-semantic::ValueSlot",
      "source": "crates/af-semantic/src/values.rs"
    },
    "af-spec::AbiAssignmentJson": {
      "fields": [
        {
          "collection": false,
          "field": "schema_family",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "type_name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "number_kind",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "number",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "symbolic_name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "status",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "first_seen_schema_digest",
          "full_type": "Option<DigestRefJson>",
          "option_presence": "explicit_none_or_some"
        }
      ],
      "kind": "struct",
      "owner_type": "af-spec::AbiAssignmentJson",
      "source": "crates/af-spec/src/model.rs"
    },
    "af-spec::AbiFile": {
      "fields": [
        {
          "collection": false,
          "field": "registry_revision",
          "full_type": "u64",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "assignments",
          "full_type": "Vec<AbiAssignmentJson>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-spec::AbiFile",
      "source": "crates/af-spec/src/model.rs"
    },
    "af-spec::CoverageEntryJson": {
      "fields": [
        {
          "collection": false,
          "field": "rule_id",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "reason",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-spec::CoverageEntryJson",
      "source": "crates/af-spec/src/model.rs"
    },
    "af-spec::CoverageSummaryFile": {
      "fields": [
        {
          "collection": false,
          "field": "coverage_revision",
          "full_type": "u64",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "intentionally_uncovered",
          "full_type": "Vec<CoverageEntryJson>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "missing_coverage",
          "full_type": "Vec<CoverageEntryJson>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-spec::CoverageSummaryFile",
      "source": "crates/af-spec/src/model.rs"
    },
    "af-spec::DigestRefJson": {
      "fields": [
        {
          "collection": false,
          "field": "alg",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "domain",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "hex",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-spec::DigestRefJson",
      "source": "crates/af-spec/src/model.rs"
    },
    "af-spec::FileDigestJson": {
      "fields": [
        {
          "collection": false,
          "field": "algorithm",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "hex",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-spec::FileDigestJson",
      "source": "crates/af-spec/src/model.rs"
    },
    "af-spec::Layout": {
      "fields": [
        {
          "collection": false,
          "field": "specifications",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "rules",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "abi",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "rule_areas",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "suites",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "coverage_summary",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "conformance_root",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "idl_files",
          "full_type": "&'static [&'static str]",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "idl_walk_dir",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "drift_checks",
          "full_type": "bool",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-spec::Layout",
      "source": "crates/af-spec/src/check.rs"
    },
    "af-spec::Report": {
      "fields": [
        {
          "collection": true,
          "field": "errors",
          "full_type": "Vec<SpecError>",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "warnings",
          "full_type": "Vec<String>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "specifications",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "rules",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "covered",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "intentionally_uncovered",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "missing_coverage",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "abi_active",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "abi_retired",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "abi_reserved",
          "full_type": "usize",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "modules_checked",
          "full_type": "usize",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-spec::Report",
      "source": "crates/af-spec/src/check.rs"
    },
    "af-spec::RuleAreaJson": {
      "fields": [
        {
          "collection": false,
          "field": "area",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "description",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-spec::RuleAreaJson",
      "source": "crates/af-spec/src/model.rs"
    },
    "af-spec::RuleAreas": {
      "fields": [
        {
          "collection": true,
          "field": "areas",
          "full_type": "BTreeSet<String>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-spec::RuleAreas",
      "source": "crates/af-spec/src/tables.rs"
    },
    "af-spec::RuleAreasFile": {
      "fields": [
        {
          "collection": false,
          "field": "kind",
          "full_type": "Option<String>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "version",
          "full_type": "Option<String>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "note",
          "full_type": "Option<String>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": true,
          "field": "areas",
          "full_type": "Vec<RuleAreaJson>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-spec::RuleAreasFile",
      "source": "crates/af-spec/src/model.rs"
    },
    "af-spec::RuleJson": {
      "fields": [
        {
          "collection": false,
          "field": "rule_id",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "area",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "normative_source",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "source_anchor",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "requirement_level",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "short_statement",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "conformance_refs",
          "full_type": "Vec<String>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "intentionally_uncovered_reason",
          "full_type": "Option<String>",
          "option_presence": "explicit_none_or_some"
        }
      ],
      "kind": "struct",
      "owner_type": "af-spec::RuleJson",
      "source": "crates/af-spec/src/model.rs"
    },
    "af-spec::RulesFile": {
      "fields": [
        {
          "collection": false,
          "field": "registry_revision",
          "full_type": "u64",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "rules",
          "full_type": "Vec<RuleJson>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-spec::RulesFile",
      "source": "crates/af-spec/src/model.rs"
    },
    "af-spec::SchemaBinding": {
      "fields": [
        {
          "collection": false,
          "field": "type_name",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "number_kind",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "number",
          "full_type": "u32",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "symbolic_name",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-spec::SchemaBinding",
      "source": "crates/af-spec/src/abi.rs"
    },
    "af-spec::SourceRefJson": {
      "fields": [
        {
          "collection": false,
          "field": "logical_path",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "content_digest",
          "full_type": "FileDigestJson",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-spec::SourceRefJson",
      "source": "crates/af-spec/src/model.rs"
    },
    "af-spec::SpecCheck": {
      "fields": [
        {
          "collection": false,
          "field": "root",
          "full_type": "PathBuf",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "layout",
          "full_type": "Layout",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-spec::SpecCheck",
      "source": "crates/af-spec/src/check.rs"
    },
    "af-spec::SpecError": {
      "kind": "enum",
      "owner_type": "af-spec::SpecError",
      "source": "crates/af-spec/src/error.rs",
      "variants": {
        "AbiActiveAssignmentMissing": [
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "AbiNameRebound": [
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "AbiNumberReassigned": [
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "AbiReservedNumberInUse": [
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "AbiRetiredNumberInUse": [
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "AbiUnregisteredAssignment": [
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "AnchorNotFound": [
          {
            "collection": false,
            "field": "path",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "anchor",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "CanonicalForm": [
          {
            "collection": false,
            "field": "path",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "ContentDigestMismatch": [
          {
            "collection": false,
            "field": "path",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "expected",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "actual",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "CoverageSummaryMismatch": [
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "DuplicateAbiAssignment": [
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "DuplicateAnchorBinding": [
          {
            "collection": false,
            "field": "path",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "anchor",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "DuplicateModuleFamily": [
          {
            "collection": false,
            "field": "specification_id",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "first",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "second",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "DuplicateRuleId": [
          {
            "collection": false,
            "field": "rule_id",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "DuplicateSpecificationId": [
          {
            "collection": false,
            "field": "specification_id",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "IntentionallyUncoveredMustRule": [
          {
            "collection": false,
            "field": "rule_id",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "InvalidAbiAssignment": [
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "InvalidFileDigest": [
          {
            "collection": false,
            "field": "path",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "InvalidRequirementLevel": [
          {
            "collection": false,
            "field": "value",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "InvalidRuleId": [
          {
            "collection": false,
            "field": "rule_id",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "InvalidSourcePath": [
          {
            "collection": false,
            "field": "path",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "InvalidSpecificationId": [
          {
            "collection": false,
            "field": "input",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "Io": [
          {
            "collection": false,
            "field": "path",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "Json": [
          {
            "collection": false,
            "field": "path",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "MissingCoverage": [
          {
            "collection": false,
            "field": "rule_id",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "ModuleSetCompile": [
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "ProvisionalFrozenSource": [
          {
            "collection": false,
            "field": "path",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "SchemaCompile": [
          {
            "collection": false,
            "field": "path",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "SchemaDigestInvalid": [
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "SourceNotFound": [
          {
            "collection": false,
            "field": "path",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "SourceSpecificationMissing": [
          {
            "collection": false,
            "field": "path",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "TableDrift": [
          {
            "collection": false,
            "field": "table",
            "full_type": "String",
            "option_presence": "required"
          },
          {
            "collection": false,
            "field": "detail",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "UnknownConformanceCase": [
          {
            "collection": false,
            "field": "case_id",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "UnknownRuleArea": [
          {
            "collection": false,
            "field": "area",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "UnknownSpecificationKind": [
          {
            "collection": false,
            "field": "value",
            "full_type": "String",
            "option_presence": "required"
          }
        ],
        "UnknownSpecificationStatus": [
          {
            "collection": false,
            "field": "value",
            "full_type": "String",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-spec::SpecificationEntryJson": {
      "fields": [
        {
          "collection": false,
          "field": "specification_id",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "kind",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "status",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "normative_source",
          "full_type": "SourceRefJson",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "schema_digest",
          "full_type": "Option<DigestRefJson>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "description",
          "full_type": "Option<String>",
          "option_presence": "explicit_none_or_some"
        }
      ],
      "kind": "struct",
      "owner_type": "af-spec::SpecificationEntryJson",
      "source": "crates/af-spec/src/model.rs"
    },
    "af-spec::SpecificationsFile": {
      "fields": [
        {
          "collection": false,
          "field": "registry_revision",
          "full_type": "u64",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "specifications",
          "full_type": "Vec<SpecificationEntryJson>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-spec::SpecificationsFile",
      "source": "crates/af-spec/src/model.rs"
    },
    "af-spec::SuiteJson": {
      "fields": [
        {
          "collection": false,
          "field": "suite_id",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "description",
          "full_type": "Option<String>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": true,
          "field": "targets",
          "full_type": "Vec<String>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-spec::SuiteJson",
      "source": "crates/af-spec/src/model.rs"
    },
    "af-spec::SuitesFile": {
      "fields": [
        {
          "collection": false,
          "field": "kind",
          "full_type": "Option<String>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "suite_revision",
          "full_type": "u64",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "suites",
          "full_type": "Vec<SuiteJson>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-spec::SuitesFile",
      "source": "crates/af-spec/src/model.rs"
    },
    "af-usage::CostError": {
      "fields": [
        {
          "collection": false,
          "field": "kind",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "detail",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-usage::CostError",
      "source": "crates/af-usage/src/cost_error.rs"
    },
    "af-usage::CostProof": {
      "kind": "enum",
      "owner_type": "af-usage::CostProof",
      "source": "crates/af-usage/src/cost.rs",
      "variants": {
        "Proven": [],
        "Unproven": [
          {
            "collection": false,
            "field": "0",
            "full_type": "CostUnprovenReason",
            "option_presence": "required"
          }
        ],
        "Violation": []
      }
    },
    "af-usage::CostUnprovenReason": {
      "kind": "enum",
      "owner_type": "af-usage::CostUnprovenReason",
      "source": "crates/af-usage/src/cost.rs",
      "variants": {
        "CurrencyMismatch": [],
        "DifferentPriceSnapshot": [],
        "MissingPriceCoverage": [],
        "UnknownUsage": []
      }
    },
    "af-usage::CurrencyDescriptor": {
      "fields": [
        {
          "collection": false,
          "field": "currency",
          "full_type": "CurrencyRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "atomic_unit",
          "full_type": "MeasurementUnitRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "display_scale",
          "full_type": "u8",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-usage::CurrencyDescriptor",
      "source": "crates/af-usage/src/currency.rs"
    },
    "af-usage::CurrencyRef": {
      "fields": [
        {
          "collection": false,
          "field": "catalog",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "code",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-usage::CurrencyRef",
      "source": "crates/af-usage/src/currency.rs"
    },
    "af-usage::CurrencyUniverse": {
      "fields": [
        {
          "collection": true,
          "field": "descriptors",
          "full_type": "BTreeMap<CurrencyRef, CurrencyDescriptor>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-usage::CurrencyUniverse",
      "source": "crates/af-usage/src/currency.rs"
    },
    "af-usage::MeasurementUnitRef": {
      "fields": [
        {
          "collection": false,
          "field": "catalog",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-usage::MeasurementUnitRef",
      "source": "crates/af-usage/src/currency.rs"
    },
    "af-usage::MoneyAmount": {
      "fields": [
        {
          "collection": false,
          "field": "currency",
          "full_type": "CurrencyRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "atomic_units",
          "full_type": "u64",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-usage::MoneyAmount",
      "source": "crates/af-usage/src/cost_model.rs"
    },
    "af-usage::PriceEntry": {
      "fields": [
        {
          "collection": false,
          "field": "meter",
          "full_type": "crate::model::UsageMeterRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "phase",
          "full_type": "Option<crate::model::UsagePhase>",
          "option_presence": "explicit_none_or_some"
        },
        {
          "collection": false,
          "field": "unit",
          "full_type": "MeasurementUnitRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "rate",
          "full_type": "PriceRate",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "rounding",
          "full_type": "RoundingRule",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-usage::PriceEntry",
      "source": "crates/af-usage/src/cost_model.rs"
    },
    "af-usage::PriceRate": {
      "fields": [
        {
          "collection": false,
          "field": "numerator",
          "full_type": "u64",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "denominator",
          "full_type": "u64",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-usage::PriceRate",
      "source": "crates/af-usage/src/cost_model.rs"
    },
    "af-usage::PriceTableSnapshot": {
      "fields": [
        {
          "collection": false,
          "field": "currency",
          "full_type": "CurrencyRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "atomic_unit",
          "full_type": "MeasurementUnitRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "model",
          "full_type": "PricingModel",
          "option_presence": "required"
        },
        {
          "collection": true,
          "field": "entries",
          "full_type": "Vec<PriceEntry>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-usage::PriceTableSnapshot",
      "source": "crates/af-usage/src/cost_model.rs"
    },
    "af-usage::PricingModel": {
      "kind": "enum",
      "owner_type": "af-usage::PricingModel",
      "source": "crates/af-usage/src/cost_model.rs",
      "variants": {
        "FixedLinear": [],
        "Progressive": [],
        "Tiered": []
      }
    },
    "af-usage::RetryUsageEnvelope": {
      "fields": [
        {
          "collection": false,
          "field": "logical",
          "full_type": "StaticUsageEnvelope",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "max_attempts",
          "full_type": "u64",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "logical_narrowing_proven",
          "full_type": "bool",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-usage::RetryUsageEnvelope",
      "source": "crates/af-usage/src/control.rs"
    },
    "af-usage::RoundingRule": {
      "kind": "enum",
      "owner_type": "af-usage::RoundingRule",
      "source": "crates/af-usage/src/cost_model.rs",
      "variants": {
        "AggregateOnceCeil": [],
        "PerMeterUnitCeil": []
      }
    },
    "af-usage::StaticCostBound": {
      "kind": "enum",
      "owner_type": "af-usage::StaticCostBound",
      "source": "crates/af-usage/src/cost_model.rs",
      "variants": {
        "KnownFinite": [
          {
            "collection": false,
            "field": "0",
            "full_type": "MoneyAmount",
            "option_presence": "required"
          }
        ],
        "KnownUnbounded": [
          {
            "collection": false,
            "field": "currency",
            "full_type": "CurrencyRef",
            "option_presence": "required"
          }
        ],
        "Unproven": [
          {
            "collection": false,
            "field": "crate",
            "full_type": ":cost::CostUnprovenReason",
            "option_presence": "required"
          }
        ]
      }
    },
    "af-usage::StaticCostCeiling": {
      "fields": [
        {
          "collection": false,
          "field": "currency",
          "full_type": "CurrencyRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "forward",
          "full_type": "MoneyAmount",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "recovery",
          "full_type": "MoneyAmount",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-usage::StaticCostCeiling",
      "source": "crates/af-usage/src/cost_model.rs"
    },
    "af-usage::StaticCostProjection": {
      "fields": [
        {
          "collection": false,
          "field": "currency",
          "full_type": "CurrencyRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "forward",
          "full_type": "StaticCostBound",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "recovery",
          "full_type": "StaticCostBound",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "total",
          "full_type": "StaticCostBound",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-usage::StaticCostProjection",
      "source": "crates/af-usage/src/cost_model.rs"
    },
    "af-usage::StaticUsageBound": {
      "kind": "enum",
      "owner_type": "af-usage::StaticUsageBound",
      "source": "crates/af-usage/src/model.rs",
      "variants": {
        "KnownFinite": [
          {
            "collection": false,
            "field": "0",
            "full_type": "u64",
            "option_presence": "required"
          }
        ],
        "KnownUnbounded": [],
        "Unknown": []
      }
    },
    "af-usage::StaticUsageEnvelope": {
      "fields": [
        {
          "collection": true,
          "field": "bounds",
          "full_type": "BTreeMap<UsageCoordinate, StaticUsageBound>",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "globally_unknown",
          "full_type": "bool",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-usage::StaticUsageEnvelope",
      "source": "crates/af-usage/src/model.rs"
    },
    "af-usage::UsageAggregation": {
      "kind": "enum",
      "owner_type": "af-usage::UsageAggregation",
      "source": "crates/af-usage/src/universe.rs",
      "variants": {
        "Additive": [],
        "Unsupported": []
      }
    },
    "af-usage::UsageCoordinate": {
      "fields": [
        {
          "collection": false,
          "field": "phase",
          "full_type": "UsagePhase",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "meter",
          "full_type": "UsageMeterRef",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-usage::UsageCoordinate",
      "source": "crates/af-usage/src/model.rs"
    },
    "af-usage::UsageError": {
      "fields": [
        {
          "collection": false,
          "field": "kind",
          "full_type": "&'static str",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "detail",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-usage::UsageError",
      "source": "crates/af-usage/src/error.rs"
    },
    "af-usage::UsageLimit": {
      "fields": [
        {
          "collection": false,
          "field": "meter",
          "full_type": "UsageMeterRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "phase",
          "full_type": "UsagePhase",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "upper",
          "full_type": "UsageUpperBound",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-usage::UsageLimit",
      "source": "crates/af-usage/src/model.rs"
    },
    "af-usage::UsageMeterDescriptor": {
      "fields": [
        {
          "collection": false,
          "field": "meter",
          "full_type": "UsageMeterRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "unit",
          "full_type": "UsageUnitRef",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "aggregation",
          "full_type": "UsageAggregation",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-usage::UsageMeterDescriptor",
      "source": "crates/af-usage/src/universe.rs"
    },
    "af-usage::UsageMeterRef": {
      "fields": [
        {
          "collection": false,
          "field": "catalog",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-usage::UsageMeterRef",
      "source": "crates/af-usage/src/model.rs"
    },
    "af-usage::UsageMeterUniverse": {
      "fields": [
        {
          "collection": true,
          "field": "descriptors",
          "full_type": "BTreeMap<UsageMeterRef, UsageMeterDescriptor>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-usage::UsageMeterUniverse",
      "source": "crates/af-usage/src/universe.rs"
    },
    "af-usage::UsagePhase": {
      "kind": "enum",
      "owner_type": "af-usage::UsagePhase",
      "source": "crates/af-usage/src/model.rs",
      "variants": {
        "Forward": [],
        "Recovery": []
      }
    },
    "af-usage::UsageProof": {
      "kind": "enum",
      "owner_type": "af-usage::UsageProof",
      "source": "crates/af-usage/src/algebra.rs",
      "variants": {
        "Proven": [],
        "Unproven": [
          {
            "collection": false,
            "field": "0",
            "full_type": "UsageUnprovenReason",
            "option_presence": "required"
          }
        ],
        "Violation": []
      }
    },
    "af-usage::UsageSignature": {
      "fields": [
        {
          "collection": true,
          "field": "limits",
          "full_type": "Vec<UsageLimit>",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-usage::UsageSignature",
      "source": "crates/af-usage/src/model.rs"
    },
    "af-usage::UsageUnitRef": {
      "fields": [
        {
          "collection": false,
          "field": "catalog",
          "full_type": "String",
          "option_presence": "required"
        },
        {
          "collection": false,
          "field": "name",
          "full_type": "String",
          "option_presence": "required"
        }
      ],
      "kind": "struct",
      "owner_type": "af-usage::UsageUnitRef",
      "source": "crates/af-usage/src/universe.rs"
    },
    "af-usage::UsageUnprovenReason": {
      "kind": "enum",
      "owner_type": "af-usage::UsageUnprovenReason",
      "source": "crates/af-usage/src/algebra.rs",
      "variants": {
        "IncompatibleMeterUnit": [],
        "MissingMeter": [],
        "UnknownBound": []
      }
    },
    "af-usage::UsageUpperBound": {
      "kind": "enum",
      "owner_type": "af-usage::UsageUpperBound",
      "source": "crates/af-usage/src/model.rs",
      "variants": {
        "Finite": [
          {
            "collection": false,
            "field": "0",
            "full_type": "u64",
            "option_presence": "required"
          }
        ],
        "Unbounded": []
      }
    }
  }
}
```

<a id="model-closed-descriptors"></a>

## AF-SEMANTIC-0214

MUST: Every semantic model type has exact closed field, width, option and variant descriptors.

Direct normative dependencies: AF-ACE1-0001, AF-SEMANTIC-0198, AF-SEMANTIC-0199, AF-SEMANTIC-0200. This is not a transitive-closure certificate.

Conformance responsibility:registered/staged case g01-edition2-staging/rule-0214 checks the actual registration/staging boundary;full semantic laws D29,D30,D44 require their separate implementation-layer conformance before T8.

<a id="model-typed-order"></a>

## AF-SEMANTIC-0215

MUST: Sets and identity views use the registered typed total order, preserving semantic sequences.

Direct normative dependencies: AF-SEMANTIC-0214. This is not a transitive-closure certificate.

Conformance responsibility:registered/staged case g01-edition2-staging/rule-0215 checks the actual registration/staging boundary;full semantic laws D12,D13,D38,D45 require their separate implementation-layer conformance before T8.

<a id="model-identity-ace1"></a>

## AF-SEMANTIC-0216

MUST: Canonical identity strings are schema-directed ACE1 of the complete identity and decode bijectively.

Direct normative dependencies: AF-ACE1-0001, AF-ACE1-0002, AF-ACE1-0003, AF-ACE1-0004, AF-ACE1-0005, AF-ACE1-0006, AF-ACE1-0007, AF-ACE1-0008, AF-ACE1-0009, AF-ACE1-0010, AF-ACE1-0011, AF-ACE1-0012, AF-ACE1-0013, AF-ACE1-0014, AF-SEMANTIC-0214, AF-SEMANTIC-0215. This is not a transitive-closure certificate.

Conformance responsibility:registered/staged case g01-edition2-staging/rule-0216 checks the actual registration/staging boundary;full semantic laws D01–08,D30,D37–39,D44 require their separate implementation-layer conformance before T8.

<a id="model-minimal-value-dag"></a>

## AF-SEMANTIC-0217

MUST: Full scope is a finite minimal reachable typed value DAG with canonical bottom-up ranks.

Direct normative dependencies: AF-SEMANTIC-0181, AF-SEMANTIC-0214. This is not a transitive-closure certificate.

Conformance responsibility:registered/staged case g01-edition2-staging/rule-0217 checks the actual registration/staging boundary;full semantic laws D13,D40,M14 require their separate implementation-layer conformance before T8.

<a id="model-no-partial-output"></a>

## AF-SEMANTIC-0218

MUST: Output/work/graph limits fail explicitly without truncated successful reports.

Direct normative dependencies: AF-SEMANTIC-0193, AF-SEMANTIC-0217. This is not a transitive-closure certificate.

Conformance responsibility:registered/staged case g01-edition2-staging/rule-0218 checks the actual registration/staging boundary;full semantic laws D40,M08 require their separate implementation-layer conformance before T8.

<a id="model-closed-decode"></a>

## AF-SEMANTIC-0219

MUST: Unknown, missing, duplicate or noncanonical report fields/variants/identities are rejected without repair.

Direct normative dependencies: AF-SEMANTIC-0214, AF-SEMANTIC-0215, AF-SEMANTIC-0216, AF-SEMANTIC-0217. This is not a transitive-closure certificate.

Conformance responsibility:registered/staged case g01-edition2-staging/rule-0219 checks the actual registration/staging boundary;full semantic laws D15,D29,D30,D38,D41,D44 require their separate implementation-layer conformance before T8.

<a id="model-independent-fullfield"></a>

## AF-SEMANTIC-0220

MUST: Independent Expected compares every required typed field and canonical byte, not strings alone.

Direct normative dependencies: AF-SEMANTIC-0194, AF-SEMANTIC-0211, AF-SEMANTIC-0216, AF-SEMANTIC-0217, AF-SEMANTIC-0218, AF-SEMANTIC-0219. This is not a transitive-closure certificate.

Conformance responsibility:registered/staged case g01-edition2-staging/rule-0220 checks the actual registration/staging boundary;full semantic laws D01–48;M01–16 require their separate implementation-layer conformance before T8.

<a id="model-shape-version"></a>

## AF-SEMANTIC-0221

MUST: Model identity/rank/derivation changes require new shape/obligation/content edition.

Direct normative dependencies: AF-SEMANTIC-0214, AF-SEMANTIC-0215, AF-SEMANTIC-0216, AF-SEMANTIC-0217, AF-SEMANTIC-0218, AF-SEMANTIC-0219. This is not a transitive-closure certificate.

Conformance responsibility:registered/staged case g01-edition2-staging/rule-0221 checks the actual registration/staging boundary;full semantic laws D45,M15,M16 require their separate implementation-layer conformance before T8.

<a id="model-diagnostic-separation"></a>

## AF-SEMANTIC-0222

MUST: Reason is diagnostic only after all equality/decision-bearing content is structurally retained.

Direct normative dependencies: AF-SEMANTIC-0211, AF-SEMANTIC-0214. This is not a transitive-closure certificate.

Conformance responsibility:registered/staged case g01-edition2-staging/rule-0222 checks the actual registration/staging boundary;full semantic laws D39,M05–07 require their separate implementation-layer conformance before T8.
