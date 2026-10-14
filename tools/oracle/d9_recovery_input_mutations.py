"""Real re-encoded/re-pinned K mutations; no derived Rust report inputs."""
import copy
import json

from oracle.d9_input_mutations import load, decode, caller, source_contract, update_source, update_caller
from oracle.d9_control_model import STATIC


def fixtures():
    base=load("spec/conformance/portable-root-export-revalidation/d9/recovery/outer-retry-3.ace1")
    rows=[]
    def add(name,bundle,stage="D10",basis="AF-SEMANTIC-0150/0154..0158"):
        for table in ["contract_assets","ir_assets"]:bundle[table].sort(key=lambda item:(item["reference"]["digest"]["domain"],item["reference"]["digest"]["hex"]))
        for resolver in bundle["resolver_dependency_manifests"]:resolver["content"]["components"].sort(key=lambda item:(item["digest"]["domain"],item["digest"]["hex"]))
        rows.append({"vector_id":name,"case_id":"portable-d9-recovery-input/"+name,"bundle":bundle,"expected_stage":stage,"formal_basis":basis})
    def change(name,callback,stage="D10",basis=None):
        b=copy.deepcopy(base);c=decode(source_contract(b));callback(c)
        add(name,update_source(b,c),stage,basis or "AF-CONTRACT-0014..0020; AF-SEMANTIC-0150/0154..0158")
    def entries(c):return c["effects"]["row"]["closed"]
    def update_catalog(b,c,meters=None):
        action=c["recovery_action_bindings"][0]["identity"]
        STATIC.install_relation(b,"recovery_usage","agentflow.recovery.usage.coverage@1","RecoveryUsageCoverageCatalog",{"rules":[{"action":copy.deepcopy(action),"complete_meters":meters or []}]})
    def forward_upper(c):entries(c)[1]["bounds"]["upper"]={"finite":"2"};entries(c)[0]["bounds"]["upper"]={"finite":"2"}
    change("forward-upper-changed",forward_upper)
    for name,field in [("activation-changed","activations_per_forward"),("attempts-changed","attempts_per_activation")]:
        def alter(c,field=field):c["recovery_usage_bindings"][0][field]["upper"]={"finite":"2"};entries(c)[0]["bounds"]["upper"]={"finite":"2"}
        change(name,alter)
    b=copy.deepcopy(base);c=decode(source_contract(b));clause=c["recovery"]["effect_recovery_clauses"][0]
    clause["reconciliation"]="required";clause["compensation"]={"none":None};clause["unknown_effect_handling"]="reconciliation_required"
    c["recovery_action_bindings"][0]["identity"]["action_kind"]="reconciliation"
    c["recovery_usage_bindings"][0]["action"]["action_kind"]="reconciliation"
    update_catalog(b,c);add("action-kind-changed",update_source(b,c))
    b=copy.deepcopy(base);c=decode(source_contract(b));a=c["recovery_action_bindings"][0]["identity"]["action_effect"]
    a["qualifiers"]=[{"idempotency":"natural"}]
    entries(c)[0]["atom"]=copy.deepcopy(a);c["recovery_usage_bindings"][0]["action"]=copy.deepcopy(c["recovery_action_bindings"][0]["identity"])
    update_catalog(b,c);add("action-atom-changed",update_source(b,c))
    change("action-guard-changed",lambda c:entries(c)[0].update(guard={"input_equals":{"port_no":"1","expected":{"bool":True}}}),"D6","AF-CONTRACT-0017: missing actual input port, no invented D9 fact")
    for name,per_attempt,new_meter in [("recovery-meter-changed",1,True),("recovery-profile-changed",3,False),("explicit-zero-profile",0,False)]:
        b=copy.deepcopy(base);c=decode(source_contract(b));catalog=b["proof_contexts"][0]["content"]["usage_meters"]["complete"]
        meter=copy.deepcopy(catalog[0]["meter"])
        if new_meter:
            meter["name"]="recovery_calls";definition=copy.deepcopy(catalog[0]);definition["meter"]=copy.deepcopy(meter);catalog.append(definition);catalog.sort(key=lambda item:(item["meter"]["catalog"],item["meter"]["name"]))
        c["recovery_usage_bindings"][0]["per_attempt_usage"]["limits"]=[{"meter":copy.deepcopy(meter),"phase":"recovery","upper":{"finite":str(per_attempt)}}]
        c["usage"]["limits"]=[{"meter":copy.deepcopy(meter),"phase":"forward","upper":{"finite":"0"}},{"meter":copy.deepcopy(meter),"phase":"recovery","upper":{"finite":str(per_attempt)}}]
        update_catalog(b,c,[meter]);add(name,update_source(b,c))
    b=copy.deepcopy(base);ir=decode(caller(b));ir["computations"][0]["body"]["retry"]["delays"]=["0"]*4;add("retry-factor-changed",update_caller(b,ir))
    b=load("spec/conformance/portable-root-export-revalidation/d9/recovery/outer-iterate-3.ace1");ir=decode(caller(b));ir["computations"][0]["body"]["iterate"]["while"]["max_iterations"]="7";add("iterate-factor-changed",update_caller(b,ir))
    b=copy.deepcopy(base);ir=decode(caller(b));ir["computations"][2]["id"]="4";ir["computations"].insert(2,{"id":"3","outputs":[],"body":{"compensation_scope":{"child":"4"}}})
    b["resolver_dependency_manifests"][0]["content"]["source_associations"][0]["source_node"]="4"
    for mapping in b["root_export_assertions"][0]["assertion"]["execution_mappings"]:mapping["source_node"]="4"
    add("compensation-nesting-changed",update_caller(b,ir))
    change("source-association-changed",lambda c:c["component_identity"].update(name="changed-recovery-source"))
    change("missing-action-binding",lambda c:c.update(recovery_action_bindings=[]),"D6")
    change("missing-usage-binding",lambda c:c.update(recovery_usage_bindings=[]),"D6")
    change("wrong-forward-identity",lambda c:c["recovery_action_bindings"][0]["identity"]["forward_effect"]["operation"].update(name="compensate"),"D6")
    change("wrong-profile-identity",lambda c:c["recovery_usage_bindings"][0]["action"].update(action_kind="reconciliation"),"D6")
    # With an exhaustive EMPTY meter set, these are legitimate D9 epsilon
    # obligations, not necessarily D6 usage rejections. The negative fixtures
    # here declare a real finite meter ceiling so the D6 premise is explicit.
    for name,ceiling,attempt in [("activation-overflow",{"finite":str(2**64-1)},"2"),("unbounded-action-requirement",{"unbounded":None},"1")]:
        b=copy.deepcopy(base);c=decode(source_contract(b));meter=b["proof_contexts"][0]["content"]["usage_meters"]["complete"][0]["meter"]
        binding=c["recovery_usage_bindings"][0];binding["activations_per_forward"]["upper"]=ceiling;binding["attempts_per_activation"]["upper"]={"finite":attempt}
        binding["per_attempt_usage"]["limits"]=[{"meter":copy.deepcopy(meter),"phase":"recovery","upper":{"finite":"1"}}]
        c["usage"]["limits"]=[{"meter":copy.deepcopy(meter),"phase":"recovery","upper":{"finite":"1"}}]
        update_catalog(b,c,[meter]);add(name,update_source(b,c),"D6","AF-CONTRACT-0020: explicit finite meter rejects nonfinite/overflow K usage")
    b=copy.deepcopy(base);c=decode(source_contract(b));meter=b["proof_contexts"][0]["content"]["usage_meters"]["complete"][0]["meter"]
    update_catalog(b,c,[meter]);add("missing-required-meter",b,"D6")
    b=copy.deepcopy(base);c=decode(source_contract(b));c["recovery_usage_bindings"][0]["per_attempt_usage"]["limits"]=[{"meter":copy.deepcopy(meter),"phase":"recovery","upper":{"finite":"1"}}]
    c["usage"]["limits"]=[{"meter":copy.deepcopy(meter),"phase":"recovery","upper":{"finite":"1"}}]
    b["proof_contexts"][0]["content"]["usage_meters"]["complete"][0]["aggregation"]="unsupported"
    update_catalog(b,c,[meter]);add("unsupported-aggregation",update_source(b,c),"D6")
    return rows
