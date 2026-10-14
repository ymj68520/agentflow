#!/usr/bin/env python3
"""Declaration-only Recovery full-field Expected. Existing Expected is read-only."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import sys

ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/"tools"))
from oracle.d9_recovery_model import RecoveryModel
from oracle import field_contract as FC
from oracle.rust_type_inventory import inventory
from oracle.coverage_bindings import bindings
from oracle.d9_recovery_input_mutations import fixtures
from oracle.d9_recovery_verification import comparator_mutations, hand_vectors
from oracle.render_d9_recovery_docs import documents
from portable_proof_python_oracle import canonical_bytes

spec=importlib.util.spec_from_file_location("control_coverage",ROOT/"tools/gen-portable-d9-control-full-oracle.py")
A=importlib.util.module_from_spec(spec);spec.loader.exec_module(A)
CORPUS=ROOT/"spec/conformance/portable-root-export-revalidation"
OUT=CORPUS/"expected-full/d9-recovery"


def models():
    historical=json.loads((CORPUS/"manifest.json").read_text(encoding="utf-8"))
    result={};rows=[]
    cases=[case for case in historical["cases"] if case["group"]=="D9-Recovery"]
    assert len(cases)==16
    for case in cases:
        # Only registration/immutable input paths are consumed, never old expected bounds.
        raw=(ROOT/case["bundle_file"]).read_bytes()
        model=RecoveryModel(json.loads(raw));facts=model.facts(case["case_id"],raw)
        count=FC.validate(facts,FC.ref("D9Facts"))
        sidecar={"schema_version":1,"generator_version":"agentflow.independent.d9-recovery@1","formal_sources":["AF-SEMANTIC-0150","AF-SEMANTIC-0154..0158","AF-CONTRACT-0016..0020"],"facts":facts,"instance_coverage":count,"oracle_provenance":FC.instance_provenance(facts,FC.ref("D9Facts")),"d10_full_field_comparison":"Pending"}
        slug=case["case_id"].split("/")[-1]
        result[slug]=sidecar
        rows.append({"case_id":case["case_id"],"bundle_file":case["bundle_file"],"expected_file":(OUT/(slug+".json")).relative_to(ROOT).as_posix(),"input_sha256":hashlib.sha256(raw).hexdigest(),"expected_stage":"D10","comparison_scope":"D9","root_computation":int(model.ir["root"]),"recovery_sources":[item["source"]["source_node"] for item in facts["d9_root_recovery"]["actions"]["actions"]],"instance_coverage":count,"oracle_provenance":sidecar["oracle_provenance"]})
    return result,rows


def main():
    parser=argparse.ArgumentParser();parser.add_argument("--check",action="store_true");args=parser.parse_args()
    graph=inventory()
    frozen=json.loads((CORPUS/"coverage/rust-type-inventory.json").read_text(encoding="utf-8"))
    if graph!=frozen:raise SystemExit("production type graph drift")
    sidecars,rows=models()
    outputs={OUT/(slug+".json"):sidecar for slug,sidecar in sidecars.items()}
    outputs[OUT/"manifest.json"]={"schema_version":1,"case_count":16,"cases":rows,"instance_totals":{"required":sum(item["instance_coverage"]["required"] for item in rows),"diagnostic":sum(item["instance_coverage"]["diagnostic"] for item in rows)},"source_reads":"Portable declarations and normative inputs only"}
    controls={path.stem:json.loads(path.read_text(encoding="utf-8")) for path in (CORPUS/"expected-full/d9-control").glob("*.json") if path.name!="manifest.json"}
    A.bindings=lambda g,p:bindings(g,p,recovery=True)
    coverage=A.contract(graph,{**controls,**sidecars})
    recovery_coverage=A.contract(graph,sidecars)
    baseline=json.loads((CORPUS/"coverage/d9-control-field-coverage.json").read_text(encoding="utf-8"))
    coverage["subsets"]={"D9-Control":{"cases":22,"source_applicable":268,"projection_applicable":332,"required_instances":120117},"D9-Recovery":{"cases":16,"source_applicable":recovery_coverage["type_level"]["d9_control_applicable_source_paths"],"projection_applicable":recovery_coverage["projection_type_level"]["d9_control_applicable_paths"],"required_instances":outputs[OUT/"manifest.json"]["instance_totals"]["required"]}}
    coverage["baseline_type_level"]=baseline["type_level"]
    coverage["baseline_projection_type_level"]=baseline["projection_type_level"]
    coverage["baseline_applicable_source_paths"]=[f["typed_field_path"] for f in baseline["fields"] if f["required_comparison"] and f["case_applicability"]]
    coverage["baseline_applicable_projection_paths"]=[f["field_path"] for f in baseline["projection_fields"] if f["classification"]!="DiagnosticOnly" and f["case_applicability"]]
    coverage["type_level"]["d9_specialized_applicable_source_paths"]=coverage["type_level"].pop("d9_control_applicable_source_paths")
    coverage["projection_type_level"]["d9_specialized_applicable_paths"]=coverage["projection_type_level"].pop("d9_control_applicable_paths")
    coverage["d9_control_status"]="22-case baseline retained; Recovery adds independently derived applicability"
    outputs[CORPUS/"coverage/field-coverage.json"]=coverage
    historical=json.loads((CORPUS/"manifest.json").read_text(encoding="utf-8"))
    outputs[CORPUS/"coverage/pending-fields.json"]={"schema_version":1,"source_fields":[f["typed_field_path"] for f in coverage["fields"] if f["required_comparison"] and not f["case_applicability"]],"projection_fields":[f["field_path"] for f in coverage["projection_fields"] if f["classification"]!="DiagnosticOnly" and not f["case_applicability"]],"other_case_ids":[c["case_id"] for c in historical["cases"] if c["group"] not in {"D9-Control","D9-Recovery"}]}
    binary_outputs={};input_rows=[]
    for row in fixtures():
        raw=canonical_bytes(row["bundle"]);slug=row["vector_id"]
        path=CORPUS/"input-mutations/recovery"/(slug+".ace1")
        binary_outputs[path]=raw
        expected_path=path.with_suffix(".expected.json")
        if row["expected_stage"]=="D10":
            facts=RecoveryModel(row["bundle"]).facts(row["case_id"],raw);FC.validate(facts,FC.ref("D9Facts"))
        else:
            facts={"case_id":row["case_id"],"input_identity":{"canonical_bytes":len(raw),"canonical_sha256":hashlib.sha256(raw).hexdigest()},"production_stage":"D6","error_family":"ContractValidationFailed","error_path":"$.contract_assets","d9_root":None,"capability_issued":False}
        outputs[expected_path]={"schema_version":1,"facts":facts,"formal_basis":row["formal_basis"],"d10_independent_values":"Pending"}
        input_rows.append({"case_id":row["case_id"],"vector_id":slug,"bundle_file":path.relative_to(ROOT).as_posix(),"expected_file":expected_path.relative_to(ROOT).as_posix(),"expected_stage":row["expected_stage"]})
    outputs[CORPUS/"input-mutations/recovery/manifest.json"]={"schema_version":1,"vector_count":len(input_rows),"cases":input_rows,"original_inputs_changed":False}
    by_path={path.relative_to(ROOT).as_posix():value for path,value in outputs.items()}
    mutations=comparator_mutations(sidecars,by_path)
    outputs[CORPUS/"coverage/recovery-comparator-mutations.json"]={"schema_version":1,"baseline":"independent Expected only","mutation_count":len(mutations),"mutations":mutations}
    hands=hand_vectors()
    outputs[CORPUS/"coverage/recovery-hand-calculations.json"]={"schema_version":1,"vector_count":len(hands),"basis":"literal hand premises and independent checked arithmetic; no Actual input","vectors":hands}
    evidence=json.loads((CORPUS/"coverage/recovery-gate-evidence.json").read_text(encoding="utf-8"))
    docs=documents(coverage,outputs[OUT/"manifest.json"],sidecars,input_rows,mutations,hands,outputs[CORPUS/"coverage/pending-fields.json"],evidence)
    for filename,text in docs.items():
        path=ROOT/"docs/implementation"/filename
        if args.check:
            if not path.exists() or path.read_text(encoding="utf-8")!=text:raise SystemExit("Recovery documentation drift: "+filename)
        else:path.write_text(text,encoding="utf-8")
    # All derivations/shape checks finish before any artifact is written.
    for path,value in outputs.items():
        encoded=json.dumps(value,ensure_ascii=False,indent=2)+"\n"
        if args.check:
            if not path.exists() or path.read_text(encoding="utf-8")!=encoded:raise SystemExit("independent Expected drift: "+str(path))
        else:A.save(path,value)
    for path,raw in binary_outputs.items():
        if args.check:
            if not path.exists() or path.read_bytes()!=raw:raise SystemExit("Portable input bytes drift: "+str(path))
        else:path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(raw)
    print("D9 Recovery independent Expected: 16; required visits="+str(outputs[OUT/"manifest.json"]["instance_totals"]["required"]))


if __name__=="__main__":main()
