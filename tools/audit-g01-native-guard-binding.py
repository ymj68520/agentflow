"""G1 read-only normative/native/ownership census, not a capture oracle.

No formal source, registry, admission rule, producer or Expected is written.
The proposed binding decision is reviewed in the G1 design documents; source
and metadata observations below are not an impossibility proof from spellings.
"""
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASE = "5bee948a033cfbf1233c65448e39d0c344993b52"
M1 = "spec/tables/proof-report-model-v1.md"
M2 = "spec/tables/proof-report-model-v2.md"
NAMES = ["ActionGuardCompatibility", "GuardCompatibilityBasis", "TypedGuardCountermodel",
         "GuardInputAssignment", "GuardCountermodelReachability"]


def read(path):
    return (ROOT/path).read_text(encoding="utf8")


def sha(path):
    return hashlib.sha256((ROOT/path).read_bytes()).hexdigest()


def model(path):
    return json.loads(re.search(r"```json\s*(.*?)\s*```", read(path), re.S)[1])


def main():
    check = subprocess.run(["git", "diff", "--quiet", BASE, "--", "crates", "spec", "Cargo.toml", "Cargo.lock"], cwd=ROOT, check=False)
    if check.returncode:
        raise RuntimeError("Reviewed production/formal baseline changed")
    old = model(M1)
    current = model(M2)
    assert current["inherited_registered_model"] == old
    specs = json.loads(read("spec/registry/specifications.json"))["specifications"]
    rules = json.loads(read("spec/registry/rules.json"))["rules"]
    assert (len(specs),len(rules)) == (50,452)
    for row in specs:
        src = row["normative_source"]
        assert sha(src["logical_path"]) == src["content_digest"]["hex"]
    types = current["types"]
    refs = 0
    todo = list(types.values())
    while todo:
        x = todo.pop()
        if isinstance(x, dict):
            if "ref" in x:
                assert x["ref"] in types
                refs += 1
            todo.extend(x.values())
        elif isinstance(x, list):
            todo.extend(x)
    assert len(types)==518 and refs==1593
    reachable = set()
    todo = list(current["descriptor_roots"])
    while todo:
        n = todo.pop()
        if n in reachable:
            continue
        reachable.add(n)
        pending = [types[n]]
        while pending:
            value = pending.pop()
            if isinstance(value, dict):
                if "ref" in value:
                    todo.append(value["ref"])
                pending.extend(value.values())
            elif isinstance(value,list):
                pending.extend(value)
    source = "crates/af-contract/src/recovery_guard.rs"
    native = read(source)
    rows = []
    for name in NAMES:
        q = "af-contract::"+name
        declaration = old["semantic_model_declarations"][q]
        assert q not in types and q not in reachable
        if declaration["kind"] == "struct":
            body = re.search(r"pub struct "+name+r"\s*\{(.*?)\n\}",native,re.S)[1]
            fields = re.findall(r"^\s*(\w+):\s*(.+),\s*$",body,re.M)
            assert fields == [(f["field"],f["full_type"]) for f in declaration["fields"]]
            order = [x[0] for x in fields]
        else:
            body = re.search(r"pub enum "+name+r"\s*\{(.*?)\n\}",native,re.S)[1]
            order = re.findall(r"^\s*(\w+),\s*$",body,re.M)
            assert set(order)==set(declaration["variants"])
        rows.append({"name":q,"historical_owner":old["model_id"],
                     "declared":True,"registered_source":True,"included_as_historical_payload":True,
                     "active_materialized":False,"reachable_from_active_roots":False,
                     "metadata_claimable":"See actual owner/schema/name matrix,not inferred from name absence",
                     "native_semantically_applicable":True,"complete_current_projection":"NOT ESTABLISHED",
                     "native_declaration_order":order,"historical_descriptor":declaration})
    assert len(rows[2]["historical_descriptor"]["fields"])==14
    producer_sites=[]
    for pattern,role in [(r"pub fn analyze_action_guard_compatibility\(","entry"),
                         (r"(?<!struct )(?<!impl )ActionGuardCompatibility\s*\{","native_record_site"),
                         (r"fn build_countermodel\(","countermodel_builder"),
                         (r"Some\(TypedGuardCountermodel\s*\{","typed_countermodel_constructor")]:
        for m in re.finditer(pattern,native):
            producer_sites.append({"path":source,"line":native.count("\n",0,m.start())+1,"role":role,"audit_locator_only":True})
    pinned_paths=[M1,M2,source,"crates/af-contract/src/validate.rs","crates/af-contract/src/canonical.rs",
                  "crates/af-computation/src/registered_model.rs","crates/af-computation/src/control.rs",
                  "crates/af-computation/src/root_recovery.rs","spec/component/component-contract.md",
                  "spec/registry/specifications.json","spec/registry/rules.json"]
    result={"kind":"G1 source/owner design adjudication evidence;NOT T3 Capture",
            "baseline":BASE,"registered_counts":{"specifications":len(specs),"rules":len(rules)},
            "active_descriptors":len(types),"resolved_references":refs,"active_static_named_ref_reachable_names":len(reachable),
            "reachability_note":"Static named-ref reachability is distinct from ModelType dispatch over all518 catalogued descriptors;this is not a claim that30 definitions are illegal or unusable.",
            "old_model_exactly_inherited":True,"five_native_types":rows,"producer_locations":producer_sites,
            "existing_source":types["Source"],"existing_target":types["Target"],
            "existing_coordinate":types["Coordinate"],"existing_predicate_families":types["PredicateFamily"],
            "closed_predicates":current["closed_predicates"],
            "guard_contradiction":dict(types["TypedContradiction"]["variants"])["guard_countermodel"],
            "root_guard_source":types["af-computation::RecoveryActionSourceIdentity"],
            "model_type_constraints":[x for x in current["canonical_constraints"] if x.startswith("ModelType=")],
            "source_bindings":[{"path":p,"sha256":sha(p)} for p in pinned_paths],
            "scope_of_conclusion":"Complete exact reusable binding not established;generic containers and metadata rejection are not a semantic proof by themselves.",
            "capture":"NOT IMPLEMENTED","positive_capability":"NOT AUTHORIZED"}
    baseline = ROOT/"target/phase-04c1b4c4r3g1-baseline.json"
    if baseline.is_file():
        before=json.loads(baseline.read_bytes())
        drift=[p for p,h in before.items() if not (ROOT/p).is_file() or sha(p)!=h]
        assert not drift,drift
        result["original_file_byte_audit"]={"files":len(before),"changed":0,"removed":0}
    if len(sys.argv)>1:
        diag_path=Path(sys.argv[1])
        actual=json.loads(diag_path.read_bytes())
        assert len(actual["matrix"])==56
        assert sum(x["actual"]=="Ok(())" for x in actual["matrix"])==8
        assert len(actual["native"])==11 and not actual["capability_allowed"]
        witness=next(x for x in actual["native"] if x["diagnostic_id"]=="typed_countermodel")["countermodel"]
        assert set(witness)=={x["field"] for x in rows[2]["historical_descriptor"]["fields"]}
        assert witness["assignments"][0]["value"]=={"int64":1}
        result["actual_diagnostics"]={"path":diag_path.as_posix(),"sha256":hashlib.sha256(diag_path.read_bytes()).hexdigest(),"value":actual,"not_design_vector_execution":True}
    print(json.dumps(result,ensure_ascii=True,indent=2))


if __name__=="__main__":
    main()
