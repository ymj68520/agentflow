"""Reviewed comparator mutations and hand-derived algebra cross-checks.

Mutation inputs are independent frozen sidecars. Hand expected bounds and
unknown states below are literal calculations, never generated from Rust.
"""
import copy

from oracle.d9_algebra import (U64_MAX,checked_add,checked_mul,effect_control,
    failure_control,usage_control)
from oracle.field_contract import CoverageError,compare


def mutations():
    root=["d9_root","root_control"]
    leaf=root+["leaf_summary"]
    rows=[
        ("root-node","invoke",root+["computation"],9),
        ("root-opcode","invoke",root+["operator"],"await"),
        ("child-order","sequence",root+["direct_children"],[3,2]),
        ("child-source-node","sequence",root+["children","0","computation"],9),
        ("leaf-contract-pin","invoke",leaf+["target","contract_digest","hex"],"f"*64),
        ("leaf-call-site","invoke",leaf+["computation"],9),
        ("leaf-usage-limit","invoke",leaf+["dimensions","usage","0","upper","finite"],1),
        ("root-usage-upper","invoke",root+["usage","value","coordinates","0","bound","upper"],1),
        ("meter-unit","invoke",["context","usage_meters","0","unit","name"],"changed-unit"),
        ("meter-identity","invoke",root+["usage","value","coordinates","0","meter","name"],"changed-meter"),
        ("recovery-phase","invoke",["d9_root","root_dimensions","usage","value","coordinates","1","phase"],"forward"),
        ("recovery-source-node","invoke",root+["recovery_sources","contributions","0","source_node"],9),
        ("execution-child","invoke",root+["execution_sources","contributions","0","source_node"],9),
        ("execution-sandbox","invoke",leaf+["dimensions","execution","sandbox"],{"minimum":"strong"}),
        ("flow-influence","invoke",root+["information_flow","value","known_may_influence","influence_unknown"],True),
        ("flow-source","invoke",root+["information_flow","value","sources","0","source_node"],9),
        ("context-association","invoke",root+["proof_context","same_as_root"],False),
        ("closure-node","invoke",["d9_root","closure","computation_order"],[9]),
        ("closure-leaf","invoke",["d9_root","closure","leaf_order"],[9]),
        ("closure-occurrences","invoke",["d9_root","closure","known_effect_occurrences_accounted"],False),
        ("root-k-source-closure","invoke",["d9_root_recovery","source_closure","action_bindings_accounted"],"unproven"),
        ("root-k-verdict","invoke",["d9_root_recovery","overall_verdict"],"unproven"),
        ("scope-owner-child","scope",root+["scope_boundary","child"],9),
        ("retry-factor","retry-four-attempts",root+["operator_semantics","max_attempts"],5),
        ("retry-delay-order","retry-four-attempts",root+["operator_semantics","delay_micros_in_order"],[2,1,0]),
        ("selector-witness","select",root+["interface","select_decision","exact_case"],2),
        ("timeout-duration","timeout",root+["operator_semantics","static_duration_micros"],77),
        ("subflow-export-pin","call_subflow",leaf+["target","exported_contract_digest","hex"],"f"*64),
        ("compensation-obligation","compensation_scope",root+["compensation_scope","recovery_blocked_outcome"],"proven"),
        ("await-verdict-promotion","await",root+["effects","proofs","0","verdict"],"proven"),
        ("await-option-presence","await",leaf+["await_summary"],None),
        ("proof-precondition","invoke",root+["effects","proofs","0","prerequisites"],["undeclared-receipt"]),
    ]
    return [{"mutation_id":name,"case_slug":"d9-opcode-"+case,"path":path,"replacement":replacement} for name,case,path,replacement in rows]


def rich_mutations():
    root=["d9_root","root_control"]
    effect=root+["effects","value","entries","0"]
    rows=[
        ("effect-qualifier","effect-operation-changed",effect+["atom","qualifiers","0","member"],"none"),
        ("effect-selector","effect-operation-changed",effect+["atom","resource_selector","scope_bindings","0","value"],"2"),
        ("effect-guard-value","effect-guard-changed",effect+["guard","input_equals","expected","int64"],1),
        ("effect-upper","effect-operation-changed",effect+["bounds","upper","finite"],11),
        ("effect-occurrence-source","effect-operation-changed",root+["effect_occurrences","0","computation"],9),
        ("failure-nominal-pin","failure-member-changed",root+["leaf_summary","dimensions","failures","0","type_name"],"ChangedFailure"),
        ("resource-slot-coordinate","resource-slot-changed",root+["resources","contributions","0","occurrences","0","mapping","resource_slot"],8),
        ("resource-type-pin","resource-slot-changed",root+["resources","contributions","0","occurrences","0","mapping","required_type","schema_digest","hex"],"f"*64),
        ("execution-capability-identity","execution-capability-changed",root+["execution_sources","contributions","0","sources","0","requirements","capabilities","0","capability","name"],"wrong-capability"),
        ("failure-source-call-site","failure-member-changed",root+["leaf_summary","computation"],9),
    ]
    return [{"mutation_id":name,"expected_file":"spec/conformance/portable-root-export-revalidation/input-mutations/"+vector+".expected.json","path":path,"replacement":replacement} for name,vector,path,replacement in rows]


def replace(value,path,replacement):
    cursor=value
    for token in path[:-1]:cursor=cursor[int(token)] if isinstance(cursor,list) else cursor[token]
    token=path[-1]
    if isinstance(cursor,list):cursor[int(token)]=copy.deepcopy(replacement)
    else:cursor[token]=copy.deepcopy(replacement)


def validate_mutations(sidecars):
    for row in mutations():
        expected=sidecars[row["case_slug"]]["facts"]
        actual=copy.deepcopy(expected)
        replace(actual,row["path"],row["replacement"])
        try:compare(expected,actual)
        except CoverageError:continue
        raise AssertionError("undetected semantic mutation "+row["mutation_id"])
    both=copy.deepcopy(sidecars["d9-opcode-invoke"]["facts"])
    del both["d9_root"]["root_control"]["operator"]
    try:compare(both,both)
    except CoverageError:return
    raise AssertionError("both-side omission escaped authoritative shape contract")


def hand_calculations():
    atom={"operation":{"catalog":"hand.effects@1","name":"write"},"resource_selector":{"requirement_port_no":1,"scope_bindings":[]},"qualifiers":[{"family":"idempotency","member":"natural"}]}
    def e(lower,upper):return {"row":"closed","entries":[{"atom":copy.deepcopy(atom),"guard":{"unconditional":True},"bounds":{"lower":lower,"upper":{"finite":upper}}}]}
    def u(amount):return {"globally_unknown":False,"coordinates":[{"phase":"forward","meter":{"catalog":"hand.meters@1","name":"calls"},"bound":{"state":"known_finite","upper":amount}}]}
    vectors=[]
    for name,op,body,expected_lower,expected_upper,usage_upper in [
        ("sequence","sequence",{},1,5,5),
        ("select","select",{},0,3,3),
        ("parallel","parallel",{"mode":"all"},3,5,5),
        ("race","race",{},0,5,5),
        ("iterate","iterate",{"while":{"max_iterations":"4"}},0,8,8),
        ("retry","retry",{"delays":["0","1"]},1,6,6),
        ("timeout","timeout",{},1,2,2),
        ("handle","handle",{"clauses":[]},1,5,5),
        ("scope","scope",{},1,2,2),
        ("compensation-scope","compensation_scope",{},1,2,2),
    ]:
        values=[e(1,2),e(2,3)] if op in {"sequence","select","parallel","race","handle"} else [e(1,2)]
        usages=[u(2),u(3)] if len(values)==2 else [u(2)]
        effect=effect_control(op,values,body)
        usage=usage_control(op,usages,body)
        assert effect["entries"][0]["bounds"]=={"lower":expected_lower,"upper":{"finite":expected_upper}},name
        assert usage["coordinates"][0]["bound"]["upper"]==usage_upper,name
        vectors.append({"vector_id":name,"input_effect_bounds":[[item["entries"][0]["bounds"]["lower"],item["entries"][0]["bounds"]["upper"]["finite"]] for item in values],"operator":op,"body":body,"expected_lower":expected_lower,"expected_upper":expected_upper,"expected_usage_upper":usage_upper})
    # HANDLER no-handler path contributes zero lower, hence protected lower 1.
    phi_a={"specification_id":"hand.failure@1","type_name":"A","schema_digest":{"algorithm":"sha256","domain":"type-schema","hex":"a"*64}}
    phi_b={**phi_a,"type_name":"B"}
    handled={"clauses":[{"match":{"failure":{"failure_type":phi_a}}}]}
    assert failure_control("handle",[[phi_a],[phi_b]],handled)==[phi_b]
    vectors.append({"vector_id":"exact-handle-failure","protected":[phi_a],"handled":[phi_a],"handler":[phi_b],"expected":[phi_b]})
    retry=effect_control("retry",[e(1,2)],{"delays":["0","0"]})
    nested=effect_control("iterate",[retry],{"while":{"max_iterations":"4"}})
    assert nested["entries"][0]["bounds"]=={"lower":0,"upper":{"finite":24}}
    vectors.append({"vector_id":"retry-times-iterate","leaf_upper":2,"retry_attempts":3,"iterations":4,"expected_upper":24})
    missing={"globally_unknown":False,"coordinates":[]}
    assert usage_control("sequence",[u(0),missing],{})["coordinates"][0]["bound"]=={"state":"unknown"}
    vectors.append({"vector_id":"missing-is-not-zero","declared_zero":0,"missing":[],"expected_state":"unknown"})
    for operation in [lambda:checked_add(U64_MAX,1),lambda:checked_mul(U64_MAX,2)]:
        try:operation()
        except OverflowError:continue
        raise AssertionError("unchecked U64 arithmetic")
    vectors.append({"vector_id":"u64-overflow","max":U64_MAX,"checked_add":1,"checked_multiply":2,"expected":"OverflowError"})
    sequence_retry=effect_control("sequence",[effect_control("retry",[e(1,2)],{"delays":["0","0"]}),e(2,3)],{})
    assert sequence_retry["entries"][0]["bounds"]=={"lower":1,"upper":{"finite":9}}
    vectors.append({"vector_id":"sequence-plus-retry","inputs":[[1,2],[2,3]],"attempts":3,"expected_lower":1,"expected_upper":9})
    handle=effect_control("handle",[e(1,2),e(2,3)],{"clauses":[]})
    selected=effect_control("select",[handle,e(2,4)],{})
    assert selected["entries"][0]["bounds"]=={"lower":0,"upper":{"finite":5}}
    vectors.append({"vector_id":"select-plus-handle","protected_upper":2,"handler_upper":3,"other_branch_upper":4,"expected_lower":0,"expected_upper":5})
    race=effect_control("race",[e(1,2),e(2,3)],{})
    parallel=effect_control("parallel",[race,e(1,4)],{"mode":"all"})
    assert parallel["entries"][0]["bounds"]=={"lower":1,"upper":{"finite":9}}
    vectors.append({"vector_id":"parallel-plus-race","loser_retained":True,"expected_lower":1,"expected_upper":9})
    scope=effect_control("compensation_scope",[e(1,2)],{})
    repeated=effect_control("iterate",[scope],{"while":{"max_iterations":"4"}})
    assert repeated["entries"][0]["bounds"]=={"lower":0,"upper":{"finite":8}}
    vectors.append({"vector_id":"iterate-plus-compensation-scope","forward_retained":True,"no_action_declared":True,"iterations":4,"expected_upper":8})
    return vectors


def hand_leaf_checks(sidecars):
    invoke=sidecars["d9-opcode-invoke"]["facts"]["d9_root"]["root_control"]
    assert invoke["effects"]["value"]=={"row":"closed","entries":[]}
    assert invoke["failures"]["value"]==[]
    assert [item["bound"]["upper"] for item in invoke["usage"]["value"]["coordinates"]]==[0,0]
    await_node=sidecars["d9-opcode-await"]["facts"]["d9_root"]["root_control"]
    assert all(await_node[name]["state"]=="unknown" for name in ["effects","failures","usage","information_flow"])
    assert await_node["leaf_summary"]["await_summary"]["provenance"]=="unknown_external_influence"
    call=sidecars["d9-opcode-call_subflow"]["facts"]["d9_root"]["root_control"]
    assert call["leaf_summary"]["target"]["kind"]=="subflow"
    assert any(item["dimension"]=="subflow-body-conformance" and item["verdict"]=="requires_evidence" for item in call["proof"])
    sequence=sidecars["d9-opcode-sequence"]["facts"]["d9_root"]["root_control"]
    for name in ["resources","recovery_sources","execution_sources"]:
        assert [item["source_node"] for item in sequence[name]["contributions"]]==[2,3]
    assert sequence["children"][0]["leaf_summary"]["target"]["contract_digest"]==sequence["children"][1]["leaf_summary"]["target"]["contract_digest"]
    race=sidecars["d9-opcode-race"]["facts"]["d9_root"]["root_control"]
    assert race["information_flow"]["state"]=="partial" and race["information_flow"]["value"]["control_decision_influence_unknown"] is True
    scope=sidecars["d9-opcode-scope"]["facts"]["d9_root"]["root_control"]["scope_boundary"]
    assert scope["parent_visible_resources"]==scope["declared_resources"]==scope["child_visible_resources"]==[]
    assert (scope["resource_attenuation"],scope["authority_attenuation"],scope["budget_attenuation"],scope["owned_children_settle_before_terminal"])==("proven","unproven","unproven","requires_evidence")
    return [
        {"vector_id":"invoke-leaf","declared_effect":"closed empty","declared_failure":[],"declared_usage":{"forward":0,"recovery":0},"source_instances":1,"rule":"AF-SEMANTIC-0112/0117"},
        {"vector_id":"await-leaf","adapter_contract":"absent","expected_effect_failure_usage_flow":"Unknown","external_influence":"unknown_external_influence","rule":"AF-SEMANTIC-0122"},
        {"vector_id":"call-subflow-leaf","target_and_export":"exact declaration pins","body_conformance":"requires_evidence","rule":"AF-SEMANTIC-0118/0119/0120"},
        {"vector_id":"source-multiset-and-per-child-r-x-k","same_contract":True,"child_ids":[2,3],"expected_source_instances":2,"requirements":"retained per child","rule":"AF-SEMANTIC-0138/0139/0140/0141"},
        {"vector_id":"race-flow-decision","content_authority":"none","expected_decision_influence_unknown":True,"expected_state":"partial","rule":"AF-SEMANTIC-0132/0136"},
        {"vector_id":"scope-static-resource-and-runtime-obligations","parent_declared_child_resources":[],"resource":"proven","authority":"unproven","budget":"unproven","settlement":"requires_evidence","rule":"AF-SEMANTIC-0147/0148"},
    ]
