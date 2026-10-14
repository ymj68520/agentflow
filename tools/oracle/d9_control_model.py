"""Declaration-driven D9 model for the registered Control subset.

Formal sources: leaf resolution 0111-0123, control 0124-0142, advanced
0143-0146, scope/root closure 0147-0158. Facts are computed from exact
Portable assets/source associations and independent E/Phi/U operations. Proof
identity labels are the reviewed conformance vocabulary of those obligations;
human-readable explanations are diagnostic only. Unknown leaves remain unknown.
Non-empty K action models outside this subset raise an explicit OracleGap.
"""

from __future__ import annotations

import base64
import copy
import hashlib
import importlib.util
import json
from pathlib import Path

from portable_proof_python_oracle import hash_v1
from oracle.d9_algebra import (OracleGap, atom, checked_add, declared_effect,
    declared_usage, effect_control, failure_control, key, u64, unique, usage_control)

ROOT=Path(__file__).resolve().parents[2]
_spec=importlib.util.spec_from_file_location("d9_portable_declarations",ROOT/"tools/gen-portable-static-reconstruction-corpus.py")
STATIC=importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(STATIC)


def cp(value):
    return copy.deepcopy(value)


def proof(dimension,verdict,number,node,reference=None,leaf=False):
    value={"dimension":dimension,"verdict":verdict,"rule_id":f"AF-SEMANTIC-{number:04d}","reference":reference,"reason":"Independent formal obligation; diagnostic prose excluded","prerequisites":[]}
    if not leaf:
        value["source_node"]=node
    return value


def overall(proofs):
    verdicts={item["verdict"] for item in proofs}
    return next((verdict for verdict in ["violation","requires_evidence","unproven"] if verdict in verdicts),"proven")


def dimension(value,proofs,key_name="value",unknown_node=None):
    if value is None:
        return {"state":"unknown","known_contributions":[],"unresolved_sources":[unknown_node],"proofs":proofs}
    return {"state":"complete",key_name:value,"proofs":proofs}


def per_child(contributions,proofs,key_name="sources",unresolved=()):
    return {"state":"per_child","contributions":[{"source_node":node,key_name:values} for node,values in contributions],"unresolved_sources":sorted(set(unresolved)),"proofs":proofs}


def option(value):
    return value.get("some") if isinstance(value,dict) and "some" in value else None


def execution(raw):
    return {"capabilities":cp(raw["capabilities"]),"quantities":[{"kind_no":{ "cpu_millis":1,"memory_bytes":2,"scratch_bytes":3 }.get(item["kind"],item["kind"]),"minimum":u64(item["minimum"])} for item in raw["quantities"]],"sandbox":option(raw["sandbox"]),"trust_domain":option(raw["trust_domain"]),"region_residency":option(raw["region_residency"]),"runtime_profile":option(raw["runtime_profile"]),"implementation_abis":[item["schema"] for item in raw["implementation_abis"]],"soft_preferences":cp(raw["soft_preferences"])}


def source_ref(raw):
    value=cp(raw.get("category",raw))
    for name in ["workflow_input","resource_requirement"]:
        if name in value:value[name]=u64(value[name])
    return value


def flow_contract(raw):
    value=cp(raw)
    value["summary"]["sources"]=[source_ref(item) for item in raw["summary"]["sources"]]
    return value


def usage_limits(raw):
    return [{"meter":cp(item["meter"]),"phase":item["phase"],"upper":{"finite":u64(item["upper"]["finite"])} if "finite" in item["upper"] else {"unbounded":None}} for item in raw]


class Model:
    specialized_recovery = False

    def recovery_leaf_facts(self, contract, dims, occurrences, ports):
        return []

    def recovery_control_proofs(self, value, node, children):
        return []

    def compensation_facts(self, value, node):
        raise OracleGap("active compensation bindings require independent K model")

    def __init__(self,bundle):
        self.bundle=bundle
        self.assertion=next(item for item in bundle["root_export_assertions"] if item["id"]==bundle["root_assertion_id"])
        a=self.assertion["assertion"]
        ids=a["proof_context_manifest"]
        self.resolver=next(item["content"] for item in bundle["resolver_dependency_manifests"] if item["id"]==ids["resolver_id"])
        if self.resolver["context_id"] != ids["context_id"] or self.resolver["caller_ir"] != self.assertion["ir_asset"]:
            raise OracleGap("exact Root/context/resolver association mismatch")
        self.context=STATIC.context_projection(bundle,ids["context_id"])
        schemas=[]
        for asset in bundle["schema_assets"]:
            content=base64.urlsafe_b64decode(asset["canonical_content"]+"===")
            reference=asset["reference"]
            definition=json.loads(content)
            if reference["schema_digest"]!={"algorithm":"sha256","domain":"type-schema","hex":hash_v1("type-schema",content)} or reference["specification_id"]!=definition["schema"] or reference["type_name"]!=definition["name"]:
                raise OracleGap("schema content/nominal pin mismatch")
            schemas.append(reference)
        if any(reference not in schemas for reference in self.context["type_schemas"]):raise OracleGap("missing exact context schema content")
        for name in ["effect_resource_relation","recovery_usage_relation","flow_execution_relation"]:
            relation=self.context[name]
            if relation is not None and relation["canonical_content_checksum"]!={"algorithm":"sha256","hex":hashlib.sha256(bytes(relation["canonical_content"])).hexdigest()}:
                raise OracleGap("relation content checksum mismatch")
        self.digest=cp(self.resolver["caller_ir"]["digest"])
        self.ir=self.asset("ir_assets",self.digest)
        self.nodes={int(item["id"]):item for item in self.ir["computations"]}
        self.expressions={int(item["id"]):item for item in self.ir["expressions"]}
        self.types={int(item["id"]):item["body"] for item in self.ir["types"]}
        self.values={int(item["id"]):item for item in self.ir["values"]}
        self.associations={int(item["source_node"]):item["target"] for item in self.resolver["source_associations"] if item["caller_ir"]==self.resolver["caller_ir"]}
        if len(self.nodes)!=len(self.ir["computations"]) or len(self.associations)!=len(self.resolver["source_associations"]):
            raise OracleGap("duplicate/foreign source identity cannot be canonicalized away")
        self.contracts={item["reference"]["digest"]["hex"]:self.asset("contract_assets",item["reference"]["digest"]) for item in bundle["contract_assets"]}
        self.pins=[]
        for name in ["effect_resource_relation","recovery_usage_relation","flow_execution_relation"]:
            item=self.context[name]
            if item is not None:
                self.pins.append({"schema":item["schema"]["specification_id"],"revision":item["revision"],"canonical_content_checksum":cp(item["canonical_content_checksum"])})
        self.pins.sort(key=lambda item:item["schema"])
        self.context_ref={"same_as_root":True,"relation_catalog_pins":self.pins}
        self.order=[]
        self.leaf_order=[]
        self.work=0

    def asset(self,table,digest):
        row=next(item for item in self.bundle[table] if item["reference"]["digest"]==digest)
        raw=base64.urlsafe_b64decode(row["canonical_content"]+"===")
        if hash_v1(digest["domain"],raw) != digest["hex"]:
            raise OracleGap("declared content pin does not equal independent HashV1")
        return json.loads(raw)

    def dependencies(self,expression,visiting=()):
        if expression in visiting or len(visiting)>96:
            raise OracleGap("OracleResourceLimit/cyclic expression")
        raw=self.expressions[expression]["body"]
        kind,item=next(iter(raw.items()))
        if kind in {"value","value_slot"}:
            return [int(item)]
        if kind == "constant":
            return []
        if kind == "list_construct":
            return sorted(set(slot for child in item["elements"] for slot in self.dependencies(int(child),visiting+(expression,))))
        # Original 22-case inputs use constants. Other expression forms are
        # explicit gaps until independently modeled, never clean/default data.
        raise OracleGap("expression dependencies unsupported: "+kind)

    def visible_resources(self,node):
        # Ownership-based lexical resource transfer, independent of Phase 3C.
        def walk(current,visible):
            if current==node:
                return visible
            kind,body=next(iter(self.nodes[current]["body"].items()))
            child_visible=[int(item) for item in body["resources"]] if kind=="scope" else visible
            for child in self.children(kind,body):
                found=walk(child,child_visible)
                if found is not None:
                    return found
            return None
        return walk(int(self.ir["root"]),sorted(int(item["id"]) for item in self.ir["resources"]))

    @staticmethod
    def children(op,body):
        if op=="sequence": return [int(item) for item in body["children"]]
        if op=="select": return [int(item["child"]) for item in body["cases"]]
        if op in {"parallel","race"}: return [int(item["child"]) for item in body["branches"]]
        if op=="iterate": return [int(next(iter(body.values()))["body"])]
        if op=="handle": return [int(body["child"])]+[int(item["handler"]) for item in body["clauses"]]
        if op in {"retry","timeout","scope","compensation_scope"}: return [int(body["child"])]
        return []

    def base(self,node):
        return [proof("proof-context-consistency","proven",125,node,self.digest["hex"]),proof("control-topology","proven",125,node,f"computation:{node}")]

    def leaf(self,node,op,body):
        self.leaf_order.append(node)
        target_decl=self.associations[node]
        p=[proof("proof-context-consistency","proven",123,node,None,True),proof("scoped-dataflow","proven",113,node,f"computation:{node}",True)]
        dims=None;await_summary=None;ports=[];resource_maps=[];resource_aliases=[];occ=[];res=[];flowmaps=[]
        if op=="await":
            schema=cp(body["signal_type"])
            target={"kind":"await_signal","signal_type":schema}
            targetref=f"{schema['specification_id']}/{schema['type_name']}@{schema['schema_digest']['hex']}"
            if target_decl.get("await_signal")!=schema or schema not in self.context["type_schemas"]:
                raise OracleGap("unresolved AWAIT exact signal")
            p += [proof("await-signal-pin","proven",122,node,targetref,True),proof("await-interaction-contract","unproven",122,node,None,True),proof("await-control-semantics","proven",122,node,f"computation:{node}",True)]
            await_summary={"signal_type":schema,"correlation":int(body["correlation"]),"output_bindings":[],"timeout":"no_implicit_timeout","interaction":"contract_unavailable","provenance":"unknown_external_influence","authority":"not_inferred"}
        else:
            cref=next(item for item in self.ir["contracts"] if item["id"]==body["contract"])
            if op=="invoke":
                pin=target_decl["invoke"]["digest"]
                if pin!=cref["digest"]: raise OracleGap("INVOKE source pin mismatch")
                contract=self.contracts[pin["hex"]]
                target={"kind":"component","identity":cp(contract["component_identity"]),"contract_digest":cp(pin)}
                p += [proof("reference-resolution","proven",111,node,f"contract-ref:{int(cref['id'])}",True),proof("contract-pin-validation","proven",112,node,pin["hex"],True)]
            else:
                source=target_decl["call_subflow"]
                target_ir=source["target_semantic_ir_digest"]
                if target_ir!=cref["digest"]: raise OracleGap("subflow source target pin mismatch")
                pin=option(source["exported_contract_digest"])
                if pin is None: raise OracleGap("missing subflow export declaration")
                contract=self.contracts[pin["hex"]]
                target_body=self.asset("ir_assets",target_ir)
                if target_body["run_parameters"] or target_body["resources"] or target_body["workflow_outputs"] or contract["inputs"] or contract["outputs"] or contract["resources"]["requirements"] or body["inputs"] or body["resources"]:
                    raise OracleGap("nonempty subflow interface needs independent Phase 3D interface expansion")
                target={"kind":"subflow","semantic_ir_digest":cp(target_ir),"exported_contract_digest":cp(pin)}
                # Exact graph closure is derived from the pinned declarations.
                declared=next(item for item in self.resolver["subflow_targets"] if item["target_semantic_ir_digest"]==target_ir)
                if option(declared["direct_dependencies"])!=[]: raise OracleGap("nonempty subflow graph beyond registered subset")
                p += [proof("reference-resolution","proven",118,node,target_ir["hex"],True),proof("target-pin-validation","proven",118,node,target_ir["hex"],True),proof("subflow-interface","proven",118,node,f"computation:{node}",True),proof("subflow-dependency-closure","proven",121,node,target_ir["hex"],True),proof("subflow-export-contract","proven",119,node,pin["hex"],True)]
            targetref=pin["hex"]
            if not self.specialized_recovery and (contract["recovery_action_bindings"] or contract["recovery_usage_bindings"]):
                raise OracleGap("nonempty K action needs 4C1B4B model")
            e=declared_effect(contract["effects"])
            f=flow_contract(contract["information_flow"])
            requirements=[]
            def port_status(expected,actual):
                return "proven" if len(expected)==len(set(expected)) and len(actual)==len(set(actual)) and set(expected)==set(actual) else "violation"
            input_status=port_status([int(item["port_no"]) for item in contract["inputs"]],[int(item["port_no"]) for item in body["inputs"]])
            output_status=port_status([int(item["port_no"]) for item in contract["outputs"]],[int(item["port_no"]) for item in self.nodes[node]["outputs"]])
            resource_verdict=port_status([int(item["port_no"]) for item in contract["resources"]["requirements"]],[int(item["port_no"]) for item in body["resources"]])
            for requirement in contract["resources"]["requirements"]:
                item=cp(requirement);item["port_no"]=int(item["port_no"]);item["resource_type"]=cp(requirement["resource_type"]["schema"]);requirements.append(item)
                mapping=next((item for item in body["resources"] if item["port_no"]==requirement["port_no"]),None)
                if mapping is None:continue
                slot=next((item for item in self.ir["resources"] if item["id"]==mapping["resource_slot"]),None)
                if slot is None or slot["requirement_type"]!=requirement["resource_type"]["schema"] or int(mapping["resource_slot"]) not in self.visible_resources(node):resource_verdict="violation"
                elif requirement["required_attributes"] and resource_verdict=="proven":resource_verdict="unproven"
                resource_maps.append({"component_port_no":int(mapping["port_no"]),"resource_slot":int(mapping["resource_slot"]),"required_type":cp(requirement["resource_type"]["schema"]),"required_attributes_proven":not requirement["required_attributes"]})
            for port in contract["inputs"]:
                mapping=next((item for item in body["inputs"] if item["port_no"]==port["port_no"]),None)
                if mapping is None:continue
                expression=self.expressions[int(mapping["expression"])]
                schema=self.types[int(expression["result_type"])].get("schema")
                if schema is None and input_status=="proven":input_status="unproven"
                elif schema is not None and schema!=port["type"]:input_status="violation"
                ports.append({"contract_port_no":int(port["port_no"]),"binding":{"input_expression":int(mapping["expression"])},"exact_type":cp(schema)})
            if contract["outputs"] or self.nodes[node]["outputs"]:
                raise OracleGap("nonempty public output source requires extra independent interface model")
            groups={}
            for mapping in resource_maps:
                groups.setdefault(mapping["resource_slot"],[]).append(mapping["component_port_no"])
            resource_aliases=[{"resource_slot":slot,"component_ports":sorted(values)} for slot,values in sorted(groups.items()) if len(values)>1]
            for ordinal,entry in enumerate(e.get("entries",e.get("known",[]))):
                slot=next((mapping["resource_slot"] for mapping in resource_maps if mapping["component_port_no"]==entry["atom"]["resource_selector"]["requirement_port_no"]),None)
                occ.append({"caller_semantic_ir_digest":cp(self.digest),"source_contract_digest":cp(pin),"source_component":cp(contract["component_identity"]),"computation":node,"source_entry_ordinal":ordinal,"declared":cp(entry),"mapped_resource_slot":slot,"control_families":[]})
            for source in f["summary"]["sources"]:
                if "unknown_external" in source:
                    binding={"unknown_external":None}
                elif "workflow_input" in source:
                    binding=next(item["binding"] for item in ports if item["contract_port_no"]==int(source["workflow_input"]))
                else:
                    binding={"resource_slot":next(item["resource_slot"] for item in resource_maps if item["component_port_no"]==int(source["resource_requirement"]))}
                flowmaps.append({"source":cp(source),"sink":None,"source_binding":binding,"sink_binding":None,"matched_effect_occurrences":[]})
            if f["flow_constraints"] or contract["effect_flow_partitions"] or e["row"]!="closed":
                raise OracleGap("nonempty Flow partitions/open effects require explicit model extension")
            dims={"failures":[cp(item["payload_type"]) for item in contract["failures"]["failures"]],"effects":e,"resources":requirements,"information_flow":f,"usage":usage_limits(contract["usage"]["limits"]),"recovery":cp(contract["recovery"]),"execution":execution(contract["execution"]),"effect_flow_partitions":[],"recovery_action_bindings":[],"recovery_usage_bindings":[]}
            type_status=overall([{"verdict":input_status},{"verdict":output_status}])
            effect_status="unproven" if any(item["mapped_resource_slot"] is None for item in occ) else "proven"
            flow_status="unproven" if any(item["source_binding"] is None for item in flowmaps) else "proven"
            p += [proof("input-binding",input_status,113,node,f"computation:{node}",True),proof("output-binding",output_status,113,node,f"computation:{node}",True),proof("type-compatibility",type_status,113,node,f"computation:{node}",True),proof("resource-mapping",resource_verdict,114,node,f"computation:{node}",True),proof("effect-and-recovery",effect_status,115,node,pin["hex"],True),proof("information-flow",flow_status,116,node,pin["hex"],True),proof("failure-usage-execution","proven",117,node,pin["hex"],True)]
            if op=="call_subflow": p.append(proof("subflow-body-conformance","requires_evidence",120,node,target_ir["hex"],True))
        guards=self.recovery_leaf_facts(contract,dims,occ,ports) if dims is not None else []
        leaf={"computation":node,"caller_semantic_ir_digest":cp(self.digest),"kind":op,"target":target,"input_mappings":ports,"output_mappings":[],"resource_mappings":resource_maps,"resource_aliases":resource_aliases,"effect_occurrences":occ,"effect_residuals":res,"flow_mappings":flowmaps,"dimensions":dims,"action_guard_compatibility":guards,"await_summary":await_summary,"proof":p,"proof_context":cp(self.context_ref)}
        nodevalue=self.empty_node(node,op)
        nodevalue["leaf_summary"]=leaf
        inherited=[self.obligation(item,node) for item in p if item["verdict"]!="proven"]
        nodevalue["inherited_obligations"]=inherited
        nodeproof=self.base(node)+[dict(cp(item),source_node=node) for item in p]+[proof("occurrence-identity","proven",141,node,self.digest["hex"]),proof("typed-output-interface","proven",135,node,f"computation:{node}")]
        def source_verdict(names):
            found=[item for item in p if item["dimension"] in names]
            return overall(found) if found else "unproven"
        status={"failure":source_verdict(["failure-usage-execution"]),"effect":source_verdict(["effect-and-recovery"]),"resource":source_verdict(["resource-mapping"]),"flow":source_verdict(["information-flow","await-interaction-contract"]),"usage":"proven" if dims is not None else "unproven","recovery":source_verdict(["effect-and-recovery","failure-usage-execution"]),"execution":source_verdict(["failure-usage-execution"])}
        dp={name:[proof(label,status[name],rule,node,targetref)] for name,label,rule in [("failure","failure-row",134),("effect","effect-envelope",127),("resource","resource-port-provenance",139),("flow","information-flow",136),("usage","usage",137),("recovery","recovery-obligations",138),("execution","execution-per-child",140)]}
        nodeproof += sum([dp[name] for name in ["resource","flow","usage","recovery","execution","failure","effect"]],[])
        nodevalue["failures"]=dimension(dims["failures"] if dims else None,dp["failure"],unknown_node=node)
        nodevalue["effects"]=dimension(dims["effects"] if dims else None,dp["effect"],unknown_node=node)
        nodevalue["usage"]=dimension(declared_usage(dims["usage"]) if dims else None,dp["usage"],unknown_node=node)
        resource_occ=[{"source_node":node,"target":cp(target),"requirement":cp(next(item for item in dims["resources"] if item["port_no"]==mapping["component_port_no"])),"mapping":cp(mapping)} for mapping in resource_maps]
        nodevalue["resources"]=per_child([(node,resource_occ)] if dims else [],dp["resource"],"occurrences",[] if dims else [node])
        if dims:
            f=dims["information_flow"]
            flow={"known_may_influence":cp(f["summary"]),"unresolved_children":[],"sources":[{"source_node":node,"target":cp(target),"contract":cp(f),"source_mappings":cp(flowmaps),"effect_flow_partitions":[]}],"sequence_dependencies":[],"select_decision_dependencies":[],"control_decision_dependencies":[],"control_decision_influence_unknown":False,"unknown_external_children":[node] if any("unknown_external" in item for item in f["summary"]["sources"]) else []}
            rec_source={"source_node":node,"target":cp(target),"replay_semantics":dims["recovery"]["replay_semantics"],"effect_recovery_clauses":cp(dims["recovery"]["effect_recovery_clauses"]),"action_bindings":cp(dims["recovery_action_bindings"]),"usage_bindings":cp(dims["recovery_usage_bindings"]),"effect_occurrences":cp(occ),"action_guard_compatibility":cp(guards)}
            nodevalue["information_flow"]=dimension(flow,dp["flow"])
            nodevalue["recovery_sources"]=per_child([(node,[rec_source])],dp["recovery"])
            nodevalue["execution_sources"]=per_child([(node,[{"source_node":node,"target":cp(target),"requirements":cp(dims["execution"])}])],dp["execution"])
        else:
            nodevalue["information_flow"]=dimension(None,dp["flow"],unknown_node=node)
            nodevalue["recovery_sources"]=per_child([],dp["recovery"],unresolved=[node])
            nodevalue["execution_sources"]=per_child([],dp["execution"],unresolved=[node])
        nodevalue["effect_occurrences"]=cp(occ);nodevalue["effect_residuals"]=cp(res)
        nodevalue["resource_aliases"]=[{"resource_slot":alias["resource_slot"],"occurrences":[cp(item) for item in resource_occ if item["mapping"]["resource_slot"]==alias["resource_slot"]]} for alias in resource_aliases]
        nodevalue["proof"]=nodeproof
        return nodevalue

    def empty_node(self,node,op):
        return {"source_semantic_ir_digest":cp(self.digest),"computation":node,"operator":op,"operator_semantics":{"kind":op},"direct_children":[],"children":[],"leaf_summary":None,"interface":{"outputs":{"state":"complete","ports":[]},"select_decision":None},"scope_boundary":None,"compensation_scope":None,"effect_occurrences":[],"effect_residuals":[],"resource_aliases":[],"effect_lower_bound_adjustments":[],"sequence_dataflow_dependencies":[],"inherited_obligations":[],"proof_context":cp(self.context_ref)}

    @staticmethod
    def obligation(p,node=None):
        return {"source_node":p.get("source_node",node),**{name:cp(p[name]) for name in ["dimension","verdict","rule_id","reference","reason"]}}

    def derive(self,node,depth=0):
        self.work += 1
        if depth>96 or self.work>1024 or node in self.order: raise OracleGap("OracleResourceLimit/invalid owned tree")
        self.order.append(node)
        op,body=next(iter(self.nodes[node]["body"].items()))
        if op in {"invoke","await","call_subflow"}: return self.leaf(node,op,body)
        value=self.empty_node(node,op)
        ids=self.children(op,body);children=[self.derive(child,depth+1) for child in ids]
        if any(child[name]["state"]!="complete" for child in children for name in ["failures","effects","usage"]):
            raise OracleGap("partial child E/Phi/U composition is outside the registered 22-case subset")
        value["direct_children"]=ids;value["children"]=children
        if self.nodes[node]["outputs"]: raise OracleGap("nonempty control outputs require independent producer model")
        proofs=self.base(node)+[proof("typed-output-interface","proven",135,node,f"computation:{node}")]
        exact=None
        if op=="select":
            expression=int(body["selector"]);ordinals=[int(item["ordinal"]) for item in body["cases"]]
            candidate=self.expressions[expression]["body"].get("constant",{}).get("int64")
            exact_case=int(candidate) if candidate is not None and int(candidate) in ordinals else None
            exact=ordinals.index(exact_case) if exact_case is not None else None
            value["interface"]["select_decision"]={"expression":expression,"dependencies":self.dependencies(expression),"case_ordinals":ordinals,"exact_case":exact_case,"totality":"proven" if exact is not None else "unproven"}
        if op=="parallel": value["operator_semantics"]={"mode":body["mode"],"max_inflight":option(body["max_inflight"])}
        if op=="iterate":
            kind,iteration=next(iter(body.items()))
            count=int(iteration["max_iterations"]) if kind=="while" else None
            if kind=="foreach":
                construct=self.expressions[int(iteration["source"])]["body"].get("list_construct")
                count=len(construct["elements"]) if construct else None
                iteration=cp(iteration);iteration["_static_count"]=count;body={kind:iteration}
            mode={"kind":"while","max_iterations":count} if kind=="while" else {"kind":"foreach","statically_known_item_count":count,"max_inflight":option(iteration["max_inflight"])}
            value["operator_semantics"]={"mode":mode,"effect_repetitions":{"lower":0,"upper":{"finite":count} if count is not None else {"unbounded":None}},"usage_repetitions":count}
        if op=="retry": value["operator_semantics"]={"max_attempts":checked_add(1,len(body["delays"])),"delay_micros_in_order":[int(item) for item in body["delays"]]}
        if op=="timeout":
            expression=int(body["timeout"]);constant=self.expressions[expression]["body"].get("constant",{}).get("duration")
            value["operator_semantics"]={"expression":expression,"static_duration_micros":int(constant) if constant is not None else None,"possible_outcomes":["child_completed","timed_out"]}
            proofs.append(proof("timeout-control-outcomes","proven",145,node,f"computation:{node}"))
        if op=="handle":
            handled=[cp(clause["match"]["failure"]["failure_type"]) for clause in body["clauses"] if "failure" in clause["match"]]
            value["operator_semantics"]={"handled_failures":unique(handled),"clauses":[{"ordinal":int(item["ordinal"]),"matcher":{"timed_out":True} if "timed_out" in item["match"] else {"failure":cp(item["match"]["failure"]["failure_type"])},"handler":int(item["handler"])} for item in body["clauses"]]}
        if op=="scope":
            parent=self.visible_resources(node);declared=[int(item) for item in body["resources"]];child=self.visible_resources(ids[0])
            attenuation="proven" if set(declared)<=set(parent) and declared==child else "violation"
            value["scope_boundary"]={"control_node":node,"child":ids[0],"parent_visible_resources":parent,"declared_resources":declared,"child_visible_resources":child,"resource_attenuation":attenuation,"authority_attenuation":"unproven","budget_attenuation":"unproven","owned_children_settle_before_terminal":"requires_evidence"}
            proofs += [proof("scope-resource-attenuation",attenuation,147,node,f"computation:{ids[0]}"),proof("scope-authority-attenuation","unproven",148,node,f"computation:{node}"),proof("scope-budget-attenuation","unproven",148,node,f"computation:{node}"),proof("scope-owned-child-settlement","requires_evidence",148,node,f"computation:{ids[0]}")]
        for child in children:
            inherited=[cp(item) for item in child["proof"] if item["verdict"]!="proven"]
            proofs += inherited;value["inherited_obligations"] += [self.obligation(item) for item in inherited]
        advanced={"iterate":143,"retry":144,"timeout":145,"handle":146,"scope":148,"compensation_scope":149}
        effect_rule={"sequence":127,"select":128,"parallel":130,"race":132,"scope":147,**{k:v for k,v in advanced.items() if k!="scope"}}[op]
        fp=[proof("failure-row","proven",advanced.get(op,134),node,f"computation:{node}")]
        ep=[proof("effect-envelope","proven",effect_rule,node,f"computation:{node}")]
        up=[proof("usage","proven",151 if op=="compensation_scope" else advanced.get(op,137),node,f"computation:{node}")]
        value["failures"]=dimension(failure_control(op,[child["failures"]["value"] for child in children],body),fp)
        value["effects"]=dimension(effect_control(op,[child["effects"]["value"] for child in children],body,exact),ep)
        value["usage"]=dimension(usage_control(op,[child["usage"]["value"] for child in children],body),up)
        envelope=value["usage"]["value"]
        up[0]["verdict"]="unproven" if not envelope["coordinates"] or envelope["globally_unknown"] or any(item["bound"]["state"]=="unknown" for item in envelope["coordinates"]) else "proven"
        if op=="parallel" and body["mode"]=="fail_fast": ep.append(proof("parallel-fail-fast","proven",131,node,f"computation:{node}"))
        if op=="race": ep.append(proof("race-loser-effects","proven",132,node,f"computation:{node}"))
        if op=="select" and exact is None: ep.append(proof("select-case-totality","unproven",129,node,f"computation:{node}"))
        if op=="retry": ep.append(proof("retry-logical-effect-narrowing","requires_evidence" if any(item["bounds"]["upper"].get("finite",1)>0 for item in value["effects"]["value"].get("entries",[])) else "proven",144,node,f"computation:{node}"))
        recovery_extra=self.recovery_control_proofs(value,node,children)
        if not self.specialized_recovery and op=="retry" and value["operator_semantics"]["max_attempts"]>1 and any(item["bounds"]["upper"].get("finite",1)>0 for item in value["effects"]["value"].get("entries",[])):
            raise OracleGap("active-effect retry safety needs the specialized K model")
        for name,keyname,label,rule in [("resources","occurrences","resource-port-provenance",139),("recovery_sources","sources","recovery-obligations",133 if op=="race" else 138),("execution_sources","sources","execution-per-child",140)]:
            contributions=[cp(item) for child in children for item in child[name]["contributions"]]
            unresolved=[source for child in children for source in child[name]["unresolved_sources"]]
            p=[proof(label,"unproven" if unresolved else "proven",rule,node,f"computation:{node}")]
            if name=="recovery_sources":p += recovery_extra
            value[name]={"state":"per_child","contributions":contributions,"unresolved_sources":sorted(set(unresolved)),"proofs":p}
            proofs += p
        mapped_resources=[occurrence for contribution in value["resources"]["contributions"] for occurrence in contribution["occurrences"]]
        resource_groups={}
        for occurrence in mapped_resources:resource_groups.setdefault(occurrence["mapping"]["resource_slot"],[]).append(occurrence)
        value["resource_aliases"]=[{"resource_slot":slot,"occurrences":cp(occurrences)} for slot,occurrences in sorted(resource_groups.items()) if len(occurrences)>1]
        childflows=[child["information_flow"]["value"] for child in children]
        control_unknown=op in {"race","iterate","retry","timeout","handle"} or op=="parallel" and body["mode"]=="fail_fast"
        control_unknown |= any(item["control_decision_influence_unknown"] for item in childflows)
        summaries=[item["known_may_influence"] for item in childflows if item["known_may_influence"] is not None]
        authority_order=["none","external_data","delegated_agent","user","workflow","platform"]
        merged={"sources":unique([source for item in summaries for source in item["sources"]]),"taints":unique([taint for item in summaries for taint in item["taints"]]),"instruction_authority":max((item["instruction_authority"] for item in summaries),key=authority_order.index),"influence_unknown":control_unknown or any(item["influence_unknown"] for item in summaries)} if summaries else None
        decisions=[]
        if op=="select":decisions=self.dependencies(int(body["selector"]))
        if op=="timeout":decisions=self.dependencies(int(body["timeout"]))
        if op=="iterate":
            mode,item=next(iter(body.items()));decisions=self.dependencies(int(item["guard"] if mode=="while" else item["source"]))
        inherited_decisions=[slot for item in childflows for slot in item["control_decision_dependencies"]]
        unresolved_children=sorted(set([source for item in childflows for source in item["unresolved_children"]]+[child["computation"] for child in children if child["information_flow"]["state"]!="complete"]))
        if unresolved_children and merged is not None:
            merged["influence_unknown"]=True
        select_dynamic=op=="select" and bool(decisions)
        if select_dynamic and merged is not None:merged["influence_unknown"]=True
        flow={"known_may_influence":merged,"unresolved_children":unresolved_children,"sources":[cp(source) for item in childflows for source in item["sources"]],"sequence_dependencies":[cp(edge) for item in childflows for edge in item["sequence_dependencies"]],"select_decision_dependencies":sorted(set([slot for item in childflows for slot in item["select_decision_dependencies"]]+(decisions if op=="select" else []))),"control_decision_dependencies":sorted(set(decisions+inherited_decisions)),"control_decision_influence_unknown":control_unknown,"unknown_external_children":sorted(set(source for item in childflows for source in item["unknown_external_children"]))}
        flow_verdict="unproven" if control_unknown or flow["unresolved_children"] or flow["unknown_external_children"] or merged and merged["influence_unknown"] else "proven"
        flp=[proof("information-flow",flow_verdict,advanced.get(op,136),node,f"computation:{node}")]
        value["information_flow"]={"state":"partial","value":flow,"unresolved_sources":cp(flow["unresolved_children"]),"proofs":flp} if control_unknown or select_dynamic or flow["unresolved_children"] or flow["unknown_external_children"] else dimension(flow,flp)
        proofs += ep+fp+up+flp
        value["effect_occurrences"]=[cp(item) for child in children for item in child["effect_occurrences"]]
        for item in value["effect_occurrences"]:
            if op=="iterate": item["control_families"].append({"control_node":node,"kind":"iterate","logical_repetitions":cp(value["operator_semantics"]["effect_repetitions"]),"physical_attempts":None})
            if op=="retry":
                attempts=value["operator_semantics"]["max_attempts"]
                item["control_families"].append({"control_node":node,"kind":"retry","logical_repetitions":{"lower":1,"upper":{"finite":attempts}},"physical_attempts":{"lower":1,"upper":{"finite":attempts}}})
        value["effect_residuals"]=[cp(item) for child in children for item in child["effect_residuals"]]
        for index,child in enumerate(children):
            value["effect_lower_bound_adjustments"] += cp(child["effect_lower_bound_adjustments"])
            reason={"sequence":"sequence_continuation_can_be_skipped","race":"race_branch_may_lose_or_be_cancelled","parallel":"fail_fast_sibling_can_be_cancelled","select":"select_may_have_no_covered_case"}.get(op)
            weakened=op=="race" or op=="sequence" and index>0 or op=="parallel" and body["mode"]=="fail_fast" or op=="select" and exact is None
            if weakened and any(entry["bounds"]["lower"]>0 for entry in child["effects"]["value"].get("entries",[])):
                value["effect_lower_bound_adjustments"].append({"source_node":child["computation"],"reason":reason})
        if op=="compensation_scope":
            active=any(source["action_bindings"] for item in value["recovery_sources"]["contributions"] for source in item["sources"])
            ap=[proof("conditional-compensation-actions","proven",150,node,f"computation:{node}")];lp=[proof("local-k-action-bounds","proven",155,node,f"computation:{node}")]
            value["compensation_scope"]={"potential_actions":{"state":"complete","actions":[],"proofs":ap},"local_recovery_actions":{"state":"complete","actions":[],"proofs":lp},"compensation_order":"proven","recovery_blocked_outcome":"unproven","forward_effects_retained":True,"forward_outcome_retained":True}
            proofs += [proof("compensation-forward-history","proven",149,node,f"computation:{ids[0]}"),*ap,*lp,proof("compensation-commit-order","proven",151,node,f"computation:{node}"),proof("recovery-blocked-outcome","unproven",150,node,f"computation:{node}")]
            if active:
                value["compensation_scope"], replacement=self.compensation_facts(value,node)
                proofs=proofs[:-5]+replacement
        proofs += [proof("control-contract-boundary","proven",124,node,f"computation:{node}"),proof("occurrence-identity","proven",141,node,self.digest["hex"])]
        value["proof"]=proofs
        return value

    def recovery(self,root):
        node=root["computation"];unknown=root["recovery_sources"]["unresolved_sources"]
        status="unproven" if unknown else "proven"
        proofs=[proof(label,status if label not in {"root-recovery-multiplicity","root-recovery-guard-compatibility"} else "proven",rule,node,f"computation:{node}") for label,rule in [("root-recovery-source-closure",154),("root-recovery-action-totals",155),("root-recovery-multiplicity",155),("root-k-action-epsilon-coverage",157),("root-k-recovery-usage-coverage",158),("root-recovery-guard-compatibility",156),("root-recovery-eligibility",150)]]
        empty_effect={"row":"closed","entries":[]};empty_usage={"globally_unknown":False,"coordinates":[]}
        action_state={"state":"unknown","known_contributions":[],"unresolved_sources":cp(unknown),"proofs":cp(proofs)} if unknown else {"state":"complete","actions":[],"proofs":cp(proofs)}
        def component(value,label,rule):
            p=[proof(label,status,rule,node,f"computation:{node}")]
            return {"state":"unknown","known_contributions":[],"unresolved_sources":cp(unknown),"proofs":p} if unknown else dimension(cp(value),p)
        return {"actions":action_state,"action_effects":component(empty_effect,"root-k-action-effect-envelope",157),"epsilon_coverage_requirement":component(empty_effect,"root-k-epsilon-coverage-requirement",157),"recovery_usage":component(empty_usage,"root-k-recovery-usage",158),"source_closure":{"source_nodes_accounted":status,"action_bindings_accounted":status,"usage_profiles_accounted":status,"multiplicity_factors_accounted":"proven","root_action_totals_sound":status},"proof_dimensions":proofs,"overall_verdict":status}

    def facts(self,case_id,raw_bytes):
        root=self.derive(int(self.ir["root"]))
        if sorted(self.order)!=sorted(self.nodes): raise OracleGap("owned computation closure incomplete")
        node=root["computation"]
        closure_proofs=[proof("control-tree-closure","proven",152,node,f"computation:{node}"),proof("root-export-compatibility","unproven",153,node,f"computation:{node}")]
        recovery=self.recovery(root)
        identity={"component_contract_digests":sorted([cp(item["digest"]) for item in self.resolver["components"]],key=key),"subflow_targets":[{"semantic_ir_digest":cp(item["target_semantic_ir_digest"]),"direct_dependencies":cp(option(item["direct_dependencies"])),"exported_contract_digest":cp(option(item["exported_contract"])["digest"]) if option(item["exported_contract"]) is not None else None} for item in self.resolver["subflow_targets"]]}
        workflow={"source_semantic_ir_digest":cp(self.digest),"overall_verdict":overall(root["proof"]+closure_proofs+recovery["proof_dimensions"]),"root_control":root,"closure":{"computation_order":cp(self.order),"leaf_order":cp(self.leaf_order),"known_effect_occurrences_accounted":True,"known_effect_residuals_accounted":True,"known_recovery_sources_accounted":True,"root_export_compatibility":"not_expressed_in_semantic_ir","proof":closure_proofs},"root_dimensions":{name:cp(root[src]) for name,src in [("failure","failures"),("effect","effects"),("resource","resources"),("information_flow","information_flow"),("usage","usage"),("recovery_sources","recovery_sources"),("execution_sources","execution_sources")]},"proof":cp(root["proof"]),"resolver_identity":identity,"proof_context":cp(self.context_ref)}
        for name in ["effect_occurrences","effect_residuals","resource_aliases","effect_lower_bound_adjustments","sequence_dataflow_dependencies"]: workflow[name]=cp(root[name])
        workflow["inherited_obligations"]=cp(root["inherited_obligations"])
        return {"case_id":case_id,"input_identity":{"canonical_bytes":len(raw_bytes),"canonical_sha256":hashlib.sha256(raw_bytes).hexdigest()},"production_stage":"D10","context":cp(self.context),"d9_root":workflow,"d9_root_recovery":recovery}
