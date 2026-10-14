"""Twelve real Portable declaration mutations, independent of Rust reports.

All changed content is re-encoded and re-pinned with Python ACE1/HashV1. The
original 22 fixtures are read-only. Flow unknown influence has a formally known
D6 RequiresEvidence boundary (AF-CONTRACT-0012), so no D9 result is invented.
"""

import base64
import copy
import json

from oracle.d9_control_model import ROOT, STATIC, Model
from portable_proof_python_oracle import canonical_bytes


def load(relative):
    return json.loads((ROOT/relative).read_bytes())


def decode(asset):
    return json.loads(base64.urlsafe_b64decode(asset["canonical_content"]+"==="))


def caller(bundle):
    pin=bundle["resolver_dependency_manifests"][0]["content"]["caller_ir"]
    return next(item for item in bundle["ir_assets"] if item["reference"]==pin)


def update_caller(bundle,ir):
    old=copy.deepcopy(caller(bundle)["reference"])
    new=STATIC.canonical_asset(bundle,"ir_assets",ir,"semantic-ir","agentflow.semantic.ir@1","SemanticIrDocument")
    bundle=STATIC.replace_exact(bundle,old,new["reference"])
    bundle=STATIC.replace_exact(bundle,old["digest"],new["reference"]["digest"])
    next(item for item in bundle["ir_assets"] if item["reference"]==new["reference"])["canonical_content"]=new["canonical_content"]
    return bundle


def source_contract(bundle):
    source=bundle["resolver_dependency_manifests"][0]["content"]["source_associations"][0]["target"]["invoke"]["digest"]
    return next(item for item in bundle["contract_assets"] if item["reference"]["digest"]==source)


def update_source(bundle,contract):
    old=copy.deepcopy(source_contract(bundle)["reference"])
    ir=decode(caller(bundle))
    new=STATIC.canonical_asset(bundle,"contract_assets",contract,"component-contract","agentflow.component.contract@1","ComponentContract")
    bundle=STATIC.replace_exact(bundle,old,new["reference"])
    bundle=STATIC.replace_exact(bundle,old["digest"],new["reference"]["digest"])
    next(item for item in bundle["contract_assets"] if item["reference"]==new["reference"])["canonical_content"]=new["canonical_content"]
    for entry in ir["contracts"]:
        if entry["digest"]==old["digest"]:entry["digest"]=copy.deepcopy(new["reference"]["digest"])
    return update_caller(bundle,ir)


def fixtures():
    simple=load("spec/conformance/portable-root-export-revalidation/d9/invoke.ace1")
    rich=load("spec/conformance/portable-root-export-revalidation/p30-same-row-positive.ace1")
    rows=[]
    def add(slug,bundle,stage="D10",basis=None):
        digest_key=lambda item:(item["digest"]["algorithm"],item["digest"]["domain"],item["digest"]["hex"])
        for table in ["contract_assets","ir_assets"]:
            bundle[table].sort(key=lambda item:digest_key(item["reference"]))
        for resolver in bundle["resolver_dependency_manifests"]:
            resolver["content"]["components"].sort(key=digest_key)
            resolver["content"]["subflow_targets"].sort(key=lambda item:(item["target_semantic_ir_digest"]["algorithm"],item["target_semantic_ir_digest"]["domain"],item["target_semantic_ir_digest"]["hex"]))
        rows.append({"vector_id":slug,"case_id":"portable-d9-input-mutation/"+slug,"bundle":bundle,"expected_stage":stage,"formal_basis":basis})
    b=copy.deepcopy(simple);c=decode(source_contract(b));c["component_identity"]["name"]="changed-source-contract";add("source-contract-changed",update_source(b,c))
    b=copy.deepcopy(simple);ir=decode(caller(b));ir["root"]="7";ir["computations"][0]["id"]="7"
    b["resolver_dependency_manifests"][0]["content"]["source_associations"][0]["source_node"]="7"
    b["root_export_assertions"][0]["assertion"]["execution_mappings"][0]["source_node"]="7"
    add("computation-node-changed",update_caller(b,ir))
    b=copy.deepcopy(rich);c=decode(source_contract(b));c["effects"]["row"]["closed"][0]["atom"]["operation"]["name"]="changed-operation"
    b=update_source(b,c)
    catalog=next(item for item in b["proof_contexts"][0]["content"]["effect_catalogs"]["complete"] if item["specification_id"]=="agentflow.effects.test@1")
    catalog["operations"]=sorted(set(catalog["operations"]+["changed-operation"]))
    relation=b["proof_contexts"][0]["content"]["relation_snapshots"]["effect_resource"]["some"]
    contents=json.loads(base64.urlsafe_b64decode(relation["canonical_content"]+"==="))
    extra=copy.deepcopy(contents["supported_pairs"][0]);extra["operation"]["name"]="changed-operation"
    contents["supported_pairs"].append(extra);contents["supported_pairs"].sort(key=lambda item:(item["operation"]["catalog"],item["operation"]["name"]))
    contents["complete_operations"].append(copy.deepcopy(extra["operation"]));contents["complete_operations"].sort(key=lambda item:(item["catalog"],item["name"]))
    STATIC.install_relation(b,"effect_resource","agentflow.effect.resource.compatibility@1","EffectResourceCompatibilityCatalog",contents)
    add("effect-operation-changed",b)
    b=copy.deepcopy(rich);c=decode(source_contract(b));c["effects"]["row"]["closed"][0]["guard"]={"input_equals":{"port_no":"1","expected":{"int64":"0"}}};add("effect-guard-changed",update_source(b,c))
    b=copy.deepcopy(simple);c=decode(source_contract(b));schema=STATIC.schema_reference(b,"agentflow.semantic.ir@1","SemanticIrDocument");c["failures"]["failures"]=[{"payload_type":schema}];add("failure-member-changed",update_source(b,c))
    b=copy.deepcopy(simple);c=decode(source_contract(b));
    for item in c["usage"]["limits"]:item["meter"]["name"]="changed_calls"
    b["proof_contexts"][0]["content"]["usage_meters"]["complete"][0]["meter"]["name"]="changed_calls"
    add("usage-meter-changed",update_source(b,c))
    b=copy.deepcopy(rich);ir=decode(caller(b));ir["resources"][0]["id"]="7";ir["computations"][0]["body"]["invoke"]["resources"][0]["resource_slot"]="7"
    assertion=b["root_export_assertions"][0]["assertion"]
    assertion["resource_mappings"][0]["root_resource_slot"]="7"
    for item in assertion["resource_mappings"][0]["source_coordinates"]:item["resource_slot"]="7"
    assertion["effect_mappings"][0]["root_resource_slot"]={"some":"7"}
    add("resource-slot-changed",update_caller(b,ir))
    b=copy.deepcopy(simple);c=decode(source_contract(b));c["information_flow"]["summary"]["influence_unknown"]=True
    add("flow-influence-changed",update_source(b,c),"D6","AF-CONTRACT-0012: unknown influence RequiresEvidence; no Checked contract/D9 fact")
    b=copy.deepcopy(simple);c=decode(source_contract(b));cap={"catalog":"agentflow.execution.test@1","name":"changed-capability"}
    c["execution"]["capabilities"]=[{"capability":cap,"exact_version":"1","required_attributes":[]}]
    b["proof_contexts"][0]["content"]["execution_universes"]["capabilities"]["complete"]=[{"capability":copy.deepcopy(cap),"exact_version":"1","supported_versions":["1"],"attributes":[]}]
    add("execution-capability-changed",update_source(b,c))
    b=load("spec/conformance/portable-root-export-revalidation/d9/sequence.ace1");ir=decode(caller(b));ir["computations"][0]["body"]["sequence"]["children"]=["3","2"];add("control-child-reordered",update_caller(b,ir))
    b=load("spec/conformance/portable-root-export-revalidation/d9/retry.ace1");ir=decode(caller(b));ir["computations"][0]["body"]["retry"]["delays"]=["0","1","2","3"];add("retry-bound-changed",update_caller(b,ir))
    b=copy.deepcopy(simple);old=copy.deepcopy(b["contract_assets"][0]["reference"]);c=decode(b["contract_assets"][0]);c["component_identity"]["name"]="changed-subflow-export"
    asset=STATIC.canonical_asset(b,"contract_assets",c,"component-contract","agentflow.component.contract@1","ComponentContract");b["contract_assets"].append(asset);new=asset["reference"]
    target_ir=STATIC.component_ir(b,new);target_ir["semantic_refs"]=[{"specification_id":"agentflow.target.d9-mutation@1"}]
    target_asset=STATIC.canonical_asset(b,"ir_assets",target_ir,"semantic-ir","agentflow.semantic.ir@1","SemanticIrDocument")
    call=STATIC.subflow_ir(b,target_asset["reference"]["digest"],"agentflow.caller.d9-mutation@1")
    b,_,_=STATIC.set_full_resolver(b,call,[(target_ir,new)],[old,new],[STATIC.subflow_association(1,target_asset["reference"]["digest"],new)],old)
    add("subflow-target-export-changed",b)
    assert len(rows)==12
    return rows


def expectations(row,raw):
    if row["expected_stage"]=="D10":return Model(row["bundle"]).facts(row["case_id"],raw)
    import hashlib
    return {"case_id":row["case_id"],"input_identity":{"canonical_bytes":len(raw),"canonical_sha256":hashlib.sha256(raw).hexdigest()},"production_stage":"D6","error_family":"ContractValidationFailed","error_path":"$.contract_assets","d9_root":None,"capability_issued":False}
