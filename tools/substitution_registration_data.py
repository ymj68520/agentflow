"""Reviewed R1 -> successor normative allocation. No producer/normalizer."""
BASELINE = "51239a0045d7a6fb274bb24cdc8130f2346bf085"
OWNER = "agentflow.root-export-checker@3"
SHAPE = "agentflow.g01.report-shape@2"
OBLIGATION = "agentflow.g01.obligation@2"
SOURCES = {
 "OS": ("agentflow.computation.proof-obligation.semantics@2", "schema_language", "spec/computation/proof-obligation-semantics-v2.md", "Successor profile applicability, exact scoped facts and independent non-root fragments."),
 "SUB": ("agentflow.component.substitution-proof.semantics@1", "schema_language", "spec/component/standalone-substitution-proof.md", "Closed standalone substitution profile, ordered checked operands, three branches and exact conservative judgments."),
 "RM": ("agentflow.computation.proof-report.model@2", "closed_table", "spec/tables/proof-report-model-v2.md", "Materialized successor non-wire semantic descriptor graph, substitution cases, bases, witnesses and complete error imports."),
 "PP": ("agentflow.portable.proof.inputs.semantics@3", "schema_language", "spec/portable/portable-proof-inputs-v3.md", "Exact successor identity and current-only non-activatable staging, isolated Edition1 and Edition2 evidence."),
}
CASES = "checked_operand context_association whole_body_equality changed_pair_evidence_floor lineage_equality io_relation failure_relation effects_relation resources_relation flow_relation usage_relation recovery_relation execution_feasibility_relation execution_security_relation execution_combined_relation result_identity result_io result_failure result_effects result_resources result_information_flow result_usage result_recovery result_execution result_execution_feasibility result_execution_security result_cross_dimension".split()
DIMENSIONS = "identity io failure effects resources information-flow usage recovery execution execution-feasibility execution-security cross-dimension".split()
LOCAL = ["lineage_equality","io_relation","failure_relation","effects_relation","resources_relation","flow_relation","usage_relation","recovery_relation","execution_combined_relation","execution_feasibility_relation","execution_security_relation"]
# Symbolic draft clauses, not one rule per audit site. Every row has independent
# staging responsibility; full fact/normalization laws remain R3/T4-T8.
ALLOCATIONS = [
 ("OS","successor-profile-separation","Only the root_export profile has the five-core/nine-dimension CompleteView; standalone_substitution is a distinct non-root fragment and cannot issue RootExport capability.",["AF-SEMANTIC-0197","AF-SEMANTIC-0203","AF-SEMANTIC-0212"]),
 ("SUB","substitution-subject","The ordered old and replacement subjects retain all thirteen contract fields, exact canonical bytes and pins, actual validation provenance and complete context-role association without invented caller or resolver.",["AF-CONTRACT-0020","AF-SEMANTIC-0198","AF-SEMANTIC-0200"]),
 ("SUB","substitution-roster","The standalone_substitution profile admits exactly its twenty-seven closed cases, twenty-six non-context clauses and twelve ordered result dimensions with the registered role and applicability mapping.",["AF-SEMANTIC-0199"]),
 ("SUB","substitution-lineage","Complete catalog/name lineage equality is a local Proven or Unproven relation distinct from the final identity judgment; inequality is not Violation.",["AF-CONTRACT-0020"]),
 ("SUB","substitution-context","Analysis aliases the replacement checked context and native context identity is mandatory; a failed association blocks all twelve comparisons without claiming evaluated relation failures.",["AF-CONTRACT-0020","AF-SEMANTIC-0202"]),
 ("SUB","substitution-branches","Context failure, exact-body reflexivity and changed-body comparison are disjoint branches; reflexivity never fabricates directional comparator execution.",["AF-CONTRACT-0020"]),
 ("SUB","substitution-result-fold","Changed-body results retain the Unproven floor and all pre-arbitration candidates; the cross result folds eleven distinct non-cross results without a self dependency.",["AF-CONTRACT-0020","AF-SEMANTIC-0211"]),
 ("SUB","substitution-execution","Execution feasibility and security remain distinct, combined execution preserves Unproven over Violation, and a shared comparison error does not invent three successful independent calls.",["AF-EXECUTION-0012","AF-EXECUTION-0013","AF-EXECUTION-0014"]),
 ("SUB","substitution-usage-recovery","Usage and recovery preserve their exact frozen candidate rules, including the usage fallback without an added recovery-profile equality gate and no newly invented RequiresEvidence candidate.",["AF-USAGE-0013","AF-RECOVERY-0013","AF-CONTRACT-0020"]),
 ("SUB","substitution-origin","Application origin binds the complete declarative document and real entry path; consumption premises are distinct from original validation facts and observations do not create origins.",["AF-SEMANTIC-0200","AF-SEMANTIC-0210"]),
 ("SUB","substitution-reference-links","Typed operand/member references, original-stage prerequisites, actual supporting facts and every consumer/observation edge are retained independently of RuleId and diagnostic text.",["AF-SEMANTIC-0206","AF-SEMANTIC-0208","AF-SEMANTIC-0209"]),
 ("SUB","substitution-basis-witness","Every applicable fact retains its closed typed basis and each Violation its exact valid witness; unsupported lineage, effect, nominal-resource, influence, meter or binding relations are not upgraded to Violation.",["AF-SEMANTIC-0211","AF-SEMANTIC-0222"]),
 ("SUB","substitution-error-closure","Dependency errors use the complete domain-tagged finite frozen error-kind sets and full declared payload; unknown kinds are UnsupportedModel and missing required payload is IncompleteFacts.",["AF-SEMANTIC-0214","AF-SEMANTIC-0219"]),
 ("RM","successor-descriptor-closure","Every reachable successor descriptor has closed fields, variants, nominal widths, option presence, typed ordering and bounded DAG semantics; unknown types reject and no runtime reflection or new ACE1 grammar is permitted.",["AF-SEMANTIC-0214","AF-SEMANTIC-0215","AF-SEMANTIC-0216","AF-SEMANTIC-0217","AF-SEMANTIC-0218","AF-SEMANTIC-0221"]),
 ("PP","successor-exact-identity","The distinct successor owner binds complete source, registry, descriptor, report and obligation identity; known label/content mismatches reject and old exact identities are never relabelled.",["AF-SEMANTIC-0223","AF-SEMANTIC-0225","AF-SEMANTIC-0226"]),
 ("PP","successor-staging-denial","Registered, Supported and Activated are implementation-owned distinct states; incomplete successor producers, normalization or lossless scope cannot issue any RootExport Checked through direct, Portable or private construction paths.",["AF-SEMANTIC-0212","AF-SEMANTIC-0213","AF-SEMANTIC-0227"]),
]
RULES = [dict(source=s,anchor=a,statement=t,dependencies=d,rule_id=f"AF-SEMANTIC-{i:04}",case_id=f"g01-successor-staging/rule-{i:04}") for i,(s,a,t,d) in enumerate(ALLOCATIONS,229)]
OBSOLETE_CURRENT_POLICY = set(range(223,228))
def applied_ids():
 return sorted([f"AF-CONTRACT-{n:04}" for n in range(1,23)] + [f"AF-SEMANTIC-{n:04}" for n in list(range(111,188))+[189]+list(range(191,245)) if n not in OBSOLETE_CURRENT_POLICY])

OPERANDS = {
 "identity": ["component_identity"], "io":["inputs","outputs"],
 "failure":["failures"],"effects":["effects"],"resources":["resources","component_identity"],
 "information-flow":["information_flow","effect_flow_partitions"],
 "usage":["usage","recovery_usage_bindings"],"recovery":["recovery","recovery_action_bindings"],
 "execution":["execution"],"execution-feasibility":["execution"],"execution-security":["execution"],
 "cross-dimension":["component_identity","inputs","outputs","failures","effects","resources","information_flow","usage","recovery","execution","effect_flow_partitions","recovery_action_bindings","recovery_usage_bindings"],
}
SUBORDINATE = {
 "identity":[],"io":[],"failure":["AF-FAILURE-0008"],"effects":["AF-EFFECT-0009"],
 "resources":["AF-RESOURCE-0012","AF-RESOURCE-0013","AF-RESOURCE-0018"],
 "information-flow":["AF-INFOFLOW-0017"],"usage":["AF-USAGE-0013"],"recovery":["AF-RECOVERY-0013"],
 "execution":["AF-EXECUTION-0014"],"execution-feasibility":["AF-EXECUTION-0012"],"execution-security":["AF-EXECUTION-0013"],"cross-dimension":[],
}
COORDINATES = ["lineage","io","failure","effects","resources","flow","usage","recovery","execution","execution","execution","whole_pair"]
DECISIONS = [
 "exact catalog AND name equality:P;otherwise:U;never:V",
 "equal I/O port sets and pins:P;added required input or missing old output:V;otherwise:U",
 "new nominal failure set subset old:P;otherwise:V",
 "existing exact effect containment:P or typed U;never manufacture V",
 "existing non-strengthening Ok:P/U;strengthened error:V;other error:U",
 "flow_relaxed error:V;Ok(P) and partitions equal:P;otherwise:U",
 "equal usage and profiles:P;otherwise existing usage_not_increased P/V/U;Err:U;no extra fallback profile gate",
 "K equivalent and action bindings equal:P;Weakened:V;otherwise:U",
 "either aspect U:U;else either V:V;else:P;shared Err:U",
 "existing feasibility P/V/U;shared Err:U",
 "existing security P/V/U;shared Err:U",
 "outer fold of eleven distinct non-cross final results;V>RE>U>P;no self edge",
]
