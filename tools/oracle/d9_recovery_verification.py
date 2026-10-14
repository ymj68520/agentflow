"""Frozen-literal hand premises and independent-Expected mutations, not Actual."""
from oracle.d9_recovery_model import (action_base, apply_factors, seed, guard_proof,
    EMPTY_E, EMPTY_U)
from oracle.d9_control_model import cp
from oracle.d9_algebra import checked_add, checked_mul, U64_MAX
from oracle.field_contract import compare, CoverageError
from oracle.d9_verification_vectors import replace

DIGEST={"algorithm":"sha256","domain":"semantic-ir","hex":"a"*64}


def factor(node,kind,count):
    repetitions={"lower":1 if kind=="retry" else 0,"upper":{"finite":count}}
    return {"identity":{"caller_semantic_ir_digest":cp(DIGEST),"control_node":node,"kind":kind},"logical_repetitions":repetitions,"physical_attempts":cp(repetitions) if kind=="retry" else None}


def hand_vectors():
    vectors=[]
    def record(name,input_data,arithmetic,local,root,witness,rule="AF-SEMANTIC-0155",factors=(),scope="static possibility only; positive eligibility RequiresEvidence"):
        vectors.append({"vector_id":name,"input":input_data,"normative_rule":rule,"source_identity":{"caller":cp(DIGEST),"source_node":9,"contract_digest":{"algorithm":"sha256","domain":"component-contract","hex":"b"*64},"forward_entry_ordinal":1},"intermediate_arithmetic":arithmetic,"applied_factor_identities":cp(list(factors)),"expected_local_fact":local,"expected_root_fact":root,"expected_witness":witness,"verdict_scope":scope})
    for name,forward,activation,attempts,local_factors,root_factors,expected_local,expected_root in [
        ("retry-outside-scope-4-12",1,2,2,[],[factor(1,"retry",3)],4,12),
        ("retry-inside-scope-12-12",1,2,2,[factor(2,"retry",3)],[factor(2,"retry",3)],12,12),
        ("iterate-outside-scope-4-16",1,2,2,[],[factor(1,"iterate",4)],4,16),
        ("retry-times-iterate-8-96",2,2,2,[],[factor(1,"retry",3),factor(2,"iterate",4)],8,96),
        ("root-owner-single-six",1,2,3,[],[],None,6),
        ("nested-nearest-owner",1,2,2,[factor(4,"retry",2)],[factor(1,"iterate",3),factor(4,"retry",2)],8,24),
        ("zero-activation",5,0,9,[],[factor(1,"retry",3)],0,0),
        ("zero-forward",0,7,9,[],[factor(1,"iterate",4)],0,0),
        ("physical-not-second-factor",1,1,2,[],[factor(1,"retry",3)],2,6),
        ("two-local-factors",1,1,1,[factor(2,"retry",3),factor(3,"iterate",4)],[factor(2,"retry",3),factor(3,"iterate",4)],12,12),
        ("one-attempt-still-identity",1,2,2,[],[factor(1,"retry",1)],4,4),
        ("scope-adds-no-factor",1,2,3,[],[],6,6),
    ]:
        base=checked_mul(checked_mul(forward,activation),attempts)
        local=None if expected_local is None else apply_factors({"finite":base},local_factors)["upper"]["finite"]
        root=apply_factors({"finite":base},root_factors)["upper"]["finite"]
        assert (local,root)==(expected_local,expected_root)
        record(name,{"forward_seed":forward,"activation":activation,"attempts":attempts,"nearest_scope":None if local is None else 3},[forward,activation,attempts,base],local,root,{"local_factors":local_factors,"root_factors":root_factors,"nearest_owner_only":True},factors=root_factors)
    for op in ["sequence","parallel","race"]:
        total=checked_add(4,6);assert total==10
        record(op+"-two-cooccurring-sources",{"tree":op,"source_nodes":[3,5],"action_uppers":[4,6],"same_atom":True},"4 + 6 = 10",[4,6],10,{"source_multiset":[3,5],"deduplicate_contract_or_atom":False},"AF-SEMANTIC-0154/0156")
    record("select-exclusive-max",{"alternatives":[4,6]},"max(4,6) = 6",[4,6],6,{"exclusive":True},"AF-SEMANTIC-0156")
    record("handle-protected-plus-handler",{"protected":4,"handlers":[2,6]},"4 + max(0,2,6) = 10",[4,2,6],10,{"no_handler_path":0},"AF-SEMANTIC-0146/0156")
    record("timeout-does-not-discount",{"action_upper":6,"duration":1},"retain 6",6,6,{"no_cancel_assumption":True},"AF-SEMANTIC-0145/0156")
    # Literal full-atom identity; shared C is an aggregate, not a Forward seed.
    a={"operation":{"catalog":"hand.effects@1","name":"work"},"resource_selector":{"requirement_port_no":1,"scope_bindings":[]},"qualifiers":[{"family":"idempotency","member":"natural"}]}
    identity={"forward_effect":cp(a),"action_kind":"compensation","action_effect":cp(a)}
    binding={"action":identity,"activations_per_forward":{"lower":1,"upper":{"finite":2}},"attempts_per_activation":{"lower":1,"upper":{"finite":3}},"per_attempt_usage":[]}
    entry={"atom":a,"bounds":{"lower":1,"upper":{"finite":7}},"guard":{"unconditional":True}}
    s=seed(entry,[identity],[binding]);k=action_base(s,binding)
    assert s=={"finite":1} and k=={"finite":6} and checked_add(s["finite"],k["finite"])==7
    record("same-atom-forward-one-k-six",{"ceiling":7,"multipliers":[2,3],"full_atom":a},"floor(7/(1+2*3))=1; K=6; required=1+6=7",6,{"forward":1,"K":6,"required_epsilon":7},{"two_opportunities_one_declared_ceiling":True},"AF-SEMANTIC-0150/0157")
    record("different-atom-not-shared",{"forward_atom":"write","K_atom":"undo","forward":1,"K":6},"K requirement stays on undo",6,{"write":1,"undo":6},{"full_atom_identity":True},"AF-SEMANTIC-0157")
    record("different-qualifier-no-coalesce",{"qualifiers":["natural","none"]},"two exact identities",[4,6],[4,6],{"qualifier_family_and_member":True},"AF-SEMANTIC-0157")
    record("different-selector-no-coalesce",{"requirement_ports":[1,2],"same_operation":True},"two exact selectors",[4,6],[4,6],{"source_port_namespace":True},"AF-SEMANTIC-0157")
    record("usage-six-at-three",{"action_upper":6,"per_attempt":3,"Root_U_recovery":18},"6*3 = 18; compare <=18, do not add to U",18,18,{"phase":"recovery","meter":"hand.calls","unit":"call","total_not_36":True},"AF-SEMANTIC-0158")
    record("profile-explicit-zero",{"action_upper":6,"per_attempt":0,"profile_present":True},"6*0=0",0,0,{"complete_meter_presence":True},"AF-SEMANTIC-0158")
    record("profile-missing-is-unproven",{"required_meters":["calls"],"profile":[]},"no multiplication can fill an absent coordinate","unproven","unproven",{"missing_meter":"calls"},"AF-SEMANTIC-0154/0158")
    record("unknown-source-not-empty",{"source_state":"unknown"},"unknown is not an empty multiset","unknown","unproven",{"source_nodes_accounted":"unproven"},"AF-SEMANTIC-0154")
    assert apply_factors({"unbounded":None},[factor(1,"retry",3)])["upper"]=={"unbounded":None}
    record("unbounded-is-not-finite",{"seed":"unbounded","attempts":3},"unbounded*3 remains unbounded","unbounded","unbounded",{"not_finite_zero":True},"AF-SEMANTIC-0155")
    for name,operation in [("activation-overflow",lambda:checked_mul(U64_MAX,2)),("aggregate-overflow",lambda:checked_add(U64_MAX,1))]:
        try:operation()
        except OverflowError:pass
        else:raise AssertionError("unchecked overflow")
        record(name,{"max":U64_MAX},"checked arithmetic rejects",None,"overflow",{"no_saturation":True},"AF-SEMANTIC-0155/0158")
    f=factor(1,"retry",3)
    try:apply_factors({"finite":4},[f,f])
    except ValueError:pass
    else:raise AssertionError("duplicate factor accepted")
    record("duplicate-factor-invalid",{"factors":[f,f]},"one unique identity cannot be applied twice",None,"invalid_multiplicity",{"duplicate_identity":f["identity"]},factors=[f,f])
    record("missing-factor-not-closed",{"required":[f],"applied":[]},"expected path factor multiset differs",4,"unproven",{"missing_identity":f["identity"]},"AF-SEMANTIC-0154/0155")
    record("guard-containment-direction",{"forward":"input_equals(1,true)","action":"unconditional"},"Forward subset Action","proven","proven",{"basis":"frozen_guard_containment","receipt":None},"AF-CONTRACT-0017/AF-SEMANTIC-0156")
    record("guard-no-domain-no-countermodel",{"forward":"unconditional","action":"input_equals(1,false)","primitive_domain":None},"failed containment alone is not a violation","unproven","unproven",{"countermodel":None},"AF-CONTRACT-0017/AF-SEMANTIC-0156")
    record("typed-bool-countermodel",{"forward":"unconditional","action":"input_equals(1,false)","primitive_domain":"Bool"},"true is well-typed; Forward=true; Action=false","violation","violation",{"port_no":1,"value":{"bool":True},"reachability":"well_typed_component_input_domain"},"AF-CONTRACT-0017")
    record("invalid-countermodel-type",{"primitive_domain":"Bool","assignment":{"int64":1}},"not a Bool assignment","unproven","unproven",{"valid_countermodel":False},"AF-CONTRACT-0017")
    record("f04-root-global-sensitive-path",{"root_contains":"race","unrelated_guard_countermodel":True},"preserve frozen global suppression","unproven","unproven",{"no_path_local_repair":True},"AF-SEMANTIC-0156")
    record("source-action-profile-multiset",{"same_contract":True,"sources":[3,5],"profiles":["P","P"]},"two source-scoped copies; multiplicity retained",2,2,{"source":2,"action":2,"profile":2},"AF-SEMANTIC-0154")
    assert len(vectors)>=24
    return vectors


def comparator_mutations(sidecars,input_expected):
    base="d9-recovery-outer-retry-3"
    prefix=["d9_root_recovery","actions","actions","0"]
    digest={"algorithm":"sha256","domain":"semantic-ir","hex":"f"*64}
    rows=[]
    def add(name,path,replacement,slug=base,expected_file=None):
        row={"mutation_id":name,"path":path,"replacement":replacement}
        if expected_file:row["expected_file"]=expected_file
        else:row["case_slug"]=slug
        rows.append(row)
    for name,path,replacement in [
        ("caller-digest",["source","caller_semantic_ir_digest"],digest),
        ("source-node",["source","source_node"],8),
        ("contract-digest",["source","source_contract_digest","hex"],"e"*64),
        ("forward-ordinal",["source","forward_entry_ordinal"],0),
        ("action-kind",["source","action","action_kind"],"reconciliation"),
        ("forward-atom",["source","action","forward_effect","operation","name"],"changed"),
        ("action-atom",["source","action","action_effect","qualifiers","0","member"],"natural"),
        ("action-guard",["action_effect_occurrence","declared","guard"],{"input_equals":{"port_no":1,"expected":{"bool":False}}}),
        ("activation",["recovery_usage_binding","activations_per_forward","upper","finite"],2),
        ("attempts",["recovery_usage_binding","attempts_per_activation","upper","finite"],2),
        ("owner-scope",["owner","node"],8),
        ("owner-variant",["owner"],{"kind":"root_recovery"}),
        ("local-bound",["local_bound","upper","upper","finite"],2),
        ("local-option",["local_bound"],None),
        ("root-bound",["root_bound","upper","upper","finite"],4),
        ("logical-repeat",["root_bound","applied_families","0","logical_repetitions","upper","finite"],4),
        ("physical-attempt",["root_bound","applied_families","0","physical_attempts","upper","finite"],4),
        ("factor-identity",["root_bound","applied_families","0","identity","control_node"],8),
        ("factor-kind",["root_bound","applied_families","0","identity","kind"],"iterate"),
        ("factor-missing",["root_bound","applied_families"],[]),
        ("forward-occurrence",["forward_occurrence","source_entry_ordinal"],0),
        ("action-occurrence",["action_effect_occurrence","computation"],8),
        ("usage-binding",["recovery_usage_binding","action","action_kind"],"reconciliation"),
        ("path-step",["control_path","0","parent_node"],8),
        ("path-reachability",["control_path","0","reachability"],"proven_unreachable"),
        ("eligibility",["eligibility_evidence"],"proven"),
        ("guard-verdict",["guard_compatibility","verdict"],"unproven"),
        ("guard-option",["guard_compatibility"],None),
        ("countermodel-presence",["guard_compatibility","countermodel"],True),
        ("guard-upper",["guard_compatibility","action_upper","finite"],4),
    ]:add(name,prefix+path,replacement)
    factors=sidecars[base]["facts"]["d9_root_recovery"]["actions"]["actions"][0]["root_bound"]["applied_families"]
    add("factor-duplicate",prefix+["root_bound","applied_families"],cp(factors+factors))
    add("source-closure-verdict",["d9_root_recovery","source_closure","usage_profiles_accounted"],"unproven")
    action=sidecars[base]["facts"]["d9_root_recovery"]["actions"]["actions"][0]
    coordinate={"source_contract_digest":cp(action["source"]["source_contract_digest"]),"component_input_port":1,"input_expression":1,"exact_type":None,"dependency_slots":[]}
    add("typed-guard-coordinate",prefix+["guard_coordinates"],[coordinate])
    add("epsilon-coverage",["d9_root_recovery","epsilon_coverage_requirement","value","entries","0","bounds","upper","finite"],4)
    add("action-member-missing",["d9_root_recovery","actions","actions"],[])
    actions=sidecars[base]["facts"]["d9_root_recovery"]["actions"]["actions"]
    add("action-member-extra",["d9_root_recovery","actions","actions"],cp(actions+actions))
    rich="spec/conformance/portable-root-export-revalidation/input-mutations/recovery/recovery-profile-changed.expected.json"
    for name,path,value in [
        ("recovery-meter",prefix+["recovery_usage_binding","per_attempt_usage","0","meter","name"],"wrong"),
        ("recovery-phase",prefix+["recovery_usage_binding","per_attempt_usage","0","phase"],"forward"),
        ("recovery-unit",["context","usage_meters","0","unit","name"],"wrong-unit"),
        ("root-recovery-usage",["d9_root_recovery","recovery_usage","value","coordinates","0","bound","upper"],10),
        ("local-recovery-usage",prefix+["local_recovery_usage","coordinates","0","bound","upper"],4),
    ]:add(name,path,value,expected_file=rich)
    for row in rows:
        expected=input_expected[row["expected_file"]]["facts"] if "expected_file" in row else sidecars[row["case_slug"]]["facts"]
        actual=cp(expected);replace(actual,row["path"],row["replacement"])
        try:compare(expected,actual)
        except CoverageError:continue
        raise AssertionError("undetected independent mutation: "+row["mutation_id"])
    return rows
