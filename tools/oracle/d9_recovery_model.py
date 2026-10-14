"""Independent declared K model: AF-SEMANTIC-0150,0154..0158.

Inputs: checked-form declaration profile, never Rust reports. Local/Root
coordinates and source/action/profile/factor multisets stay distinct. Finite
arithmetic is checked; unsupported premises raise OracleGap before output.
"""
from collections import Counter
import json

from oracle.d9_control_model import Model, cp, dimension, option, overall, proof, usage_limits
from oracle.d9_algebra import (OracleGap, atom, checked_add, checked_mul, declared_usage,
    effect_combine, effect_control, guard, key, u64, upper, up_add, up_mul, usage_control, usage_repeat)

EMPTY_E={"row":"closed","entries":[]}
EMPTY_U={"globally_unknown":False,"coordinates":[]}


def identity(raw):
    return {"forward_effect":atom(raw["forward_effect"]),"action_kind":raw["action_kind"],"action_effect":atom(raw["action_effect"])}


def bounds(raw):
    return {"lower":u64(raw["lower"]),"upper":upper(raw["upper"])}


def profile(raw):
    return {"action":identity(raw["action"]),"activations_per_forward":bounds(raw["activations_per_forward"]),"attempts_per_activation":bounds(raw["attempts_per_activation"]),"per_attempt_usage":usage_limits(raw["per_attempt_usage"]["limits"])}


def seed(forward, action_ids, bindings):
    ceiling=forward["bounds"]["upper"]
    equal=[binding for binding in bindings if binding["action"]["forward_effect"]==forward["atom"] and binding["action"]["action_effect"]==forward["atom"]]
    if not equal:return cp(ceiling)
    if any(item["forward_effect"]!=forward["atom"] and item["action_effect"]==forward["atom"] for item in action_ids):
        raise OracleGap("shared same-atom ceiling has unrelated Forward contribution")
    divisor=1
    for binding in equal:
        multiplier=up_mul(binding["activations_per_forward"]["upper"],binding["attempts_per_activation"]["upper"])
        if "finite" not in multiplier:raise OracleGap("unbounded same-atom seed divisor")
        divisor=checked_add(divisor,multiplier["finite"])
    if "finite" not in ceiling:return cp(ceiling)
    inferred=ceiling["finite"]//divisor
    if inferred<forward["bounds"]["lower"]:raise ValueError("same-atom inferred seed violates Forward lower")
    return {"finite":inferred}


def action_base(forward_seed,binding):
    return up_mul(up_mul(forward_seed,binding["activations_per_forward"]["upper"]),binding["attempts_per_activation"]["upper"])


def apply_factors(base,factors):
    seen=set();result=cp(base)
    for factor in factors:
        token=key(factor["identity"])
        if token in seen:raise ValueError("InvalidMultiplicity: duplicate unique factor")
        seen.add(token)
        result=up_mul(result,factor["logical_repetitions"]["upper"])
    return {"state":"known","upper":result}


def usage_for(binding,action_upper):
    return usage_repeat(declared_usage(binding["per_attempt_usage"]),action_upper.get("finite"))


def guard_proof(component, action, forward, effect, forward_index, action_index, action_upper):
    fg,ag=forward["guard"],effect["guard"]
    contained="unconditional" in ag or fg==ag
    vacuous=action_upper.get("finite")==0
    return {"source_component":cp(component),"recovery_action":cp(action),"forward_entry_ordinal":forward_index,"action_entry_ordinal":action_index,"forward_guard":cp(fg),"declared_action_guard":cp(ag),"action_upper":cp(action_upper),"verdict":"proven" if contained or vacuous else "unproven","basis":"zero_possible_actions" if vacuous else "frozen_guard_containment" if contained else "insufficient_evidence","reason":"Independent guard relation; no receipt or action eligibility","countermodel":None}


def effect_entry(action,ceiling,action_guard):
    return {"atom":cp(action),"bounds":{"lower":0,"upper":cp(ceiling)},"guard":cp(action_guard)}


class RecoveryModel(Model):
    specialized_recovery=True

    def recovery_leaf_facts(self,contract,dims,occurrences,ports):
        clauses=[]
        for raw in contract["recovery"]["effect_recovery_clauses"]:
            clauses.append({"effect":atom(raw["effect"]),"idempotency":raw["idempotency"],"key_requirement":cp(option(raw["key_requirement"])),"reconciliation":raw["reconciliation"],"compensation":cp(raw["compensation"]),"unknown_effect_handling":raw["unknown_effect_handling"].replace("_","")})
        dims["recovery"]["effect_recovery_clauses"]=clauses
        actions=[identity(raw["identity"]) for raw in contract["recovery_action_bindings"]]
        profiles=[profile(raw) for raw in contract["recovery_usage_bindings"]]
        dims["recovery_action_bindings"]=actions;dims["recovery_usage_bindings"]=profiles
        if Counter(key(item) for item in actions)!=Counter(key(item["action"]) for item in profiles):
            raise ValueError("exact action/profile multiset mismatch; D6 rejection required")
        if actions:
            relation=self.context["recovery_usage_relation"]
            if relation is None:raise OracleGap("missing exact RecoveryUsage coverage catalog")
            rules=json.loads(bytes(relation["canonical_content"]))["rules"]
            for binding in profiles:
                rule=next((item for item in rules if identity(item["action"])==binding["action"]),None)
                if rule is None:raise OracleGap("missing exact action usage rule")
                if Counter(key(meter) for meter in rule["complete_meters"])!=Counter(key(item["meter"]) for item in binding["per_attempt_usage"]):raise OracleGap("missing/unexpected exhaustive Recovery meter")
                for limit in binding["per_attempt_usage"]:
                    definition=next((item for item in self.context["usage_meters"] if item["meter"]==limit["meter"]),None)
                    if limit["phase"]!="recovery" or definition is None or definition["aggregation"]!="additive":raise OracleGap("unsupported phase/unit/aggregation in K profile")
        result=[]
        for action in actions:
            binding=next(item for item in profiles if item["action"]==action)
            fo=next(item for item in occurrences if item["declared"]["atom"]==action["forward_effect"])
            ao=next(item for item in occurrences if item["declared"]["atom"]==action["action_effect"])
            s=seed(fo["declared"],actions,profiles)
            result.append(guard_proof(contract["component_identity"],action,fo["declared"],ao["declared"],fo["source_entry_ordinal"],ao["source_entry_ordinal"],action_base(s,binding)))
        return result

    def recovery_control_proofs(self,value,node,children):
        if value["operator"]!="retry" or value["operator_semantics"]["max_attempts"]<=1:return []
        if not any(entry["bounds"]["upper"].get("finite",1)>0 for entry in value["effects"]["value"].get("entries",[])):return []
        sources=[source for child in children for contribution in child["recovery_sources"]["contributions"] for source in contribution["sources"]]
        verdicts=[]
        for source in sources:
            if source["replay_semantics"]=="forbidden":verdicts.append("violation");continue
            clauses=source["effect_recovery_clauses"]
            active=[occurrence for occurrence in source["effect_occurrences"] if occurrence["declared"]["bounds"]["upper"].get("finite")!=0]
            if any(not any(clause["effect"]==occurrence["declared"]["atom"] for clause in clauses) for occurrence in active):
                verdicts.append("unproven");continue
            for occurrence in active:
                if occurrence["declared"]["bounds"]["upper"].get("finite")==0:continue
                clause=next((item for item in clauses if item["effect"]==occurrence["declared"]["atom"]),None)
                if clause is None:verdicts.append("unproven")
                else:verdicts.append("requires_evidence")
        verdict=overall([{"verdict":item} for item in verdicts]) if verdicts else "unproven"
        return [proof("retry-recovery-safety",verdict,144,node,f"computation:{node}")]

    def source_actions(self,tree):
        """Walk declarations/derived Python nodes; never coalesce source pins."""
        actions=[]
        occurrences={(item["computation"],item["source_entry_ordinal"]):item for item in tree["effect_occurrences"]}
        def walk(current,owner,local_factors,root_factors,path,reachable=True):
            node=current["computation"];op=current["operator"]
            if op=="compensation_scope":owner=node;local_factors=[]
            if op in {"retry","iterate"}:
                repetitions={"lower":1,"upper":{"finite":current["operator_semantics"]["max_attempts"]}} if op=="retry" else cp(current["operator_semantics"]["effect_repetitions"])
                factor={"identity":{"caller_semantic_ir_digest":cp(self.digest),"control_node":node,"kind":op},"logical_repetitions":repetitions,"physical_attempts":cp(repetitions) if op=="retry" else None}
                root_factors=root_factors+[factor]
                if owner is not None:local_factors=local_factors+[factor]
            leaf=current["leaf_summary"]
            if leaf is not None and leaf["dimensions"] is not None:
                dims=leaf["dimensions"];bindings=dims["recovery_usage_bindings"]
                for action in dims["recovery_action_bindings"]:
                    binding=next(item for item in bindings if item["action"]==action)
                    fo=next(cp(item) for item in leaf["effect_occurrences"] if item["declared"]["atom"]==action["forward_effect"])
                    ao=next(cp(item) for item in leaf["effect_occurrences"] if item["declared"]["atom"]==action["action_effect"])
                    # The independently derived subtree occurrence carries the
                    # descriptive enclosing families. They are not multiplied
                    # here; the distinct bound coordinate applies each once.
                    fo=cp(occurrences[(node,fo["source_entry_ordinal"])])
                    ao=cp(occurrences[(node,ao["source_entry_ordinal"])])
                    forward_seed=seed(fo["declared"],dims["recovery_action_bindings"],bindings)
                    base=action_base(forward_seed,binding)
                    root_bound=apply_factors(base,root_factors)
                    if not reachable:root_bound["upper"]={"finite":0}
                    local_bound=None if owner is None else {"compensation_scope_node":owner,"upper":apply_factors(base,local_factors),"applied_families":cp(local_factors)}
                    coords=[]
                    for g in [fo["declared"]["guard"],ao["declared"]["guard"]]:
                        if "input_equals" in g:
                            port=g["input_equals"]["port_no"]
                            if any(item["component_input_port"]==port for item in coords):continue
                            mapping=next((item for item in leaf["input_mappings"] if item["contract_port_no"]==port),None)
                            if mapping is None:raise OracleGap("guard coordinate not mapped")
                            expression=mapping["binding"]["input_expression"]
                            coords.append({"source_contract_digest":cp(fo["source_contract_digest"]),"component_input_port":port,"input_expression":expression,"exact_type":cp(mapping["exact_type"]),"dependency_slots":self.dependencies(expression)})
                    guard_fact=next((cp(item) for item in leaf["action_guard_compatibility"] if item["recovery_action"]==action),None)
                    actions.append({"source":{"caller_semantic_ir_digest":cp(self.digest),"source_node":node,"source_contract_digest":cp(fo["source_contract_digest"]),"forward_entry_ordinal":fo["source_entry_ordinal"],"action":cp(action)},"forward_seed_upper":forward_seed,"target":cp(leaf["target"]),"owner":{"kind":"root_recovery"} if owner is None else {"kind":"compensation_scope","node":owner},"local_bound":local_bound,"root_bound":{"root_semantic_ir_digest":cp(self.digest),"upper":root_bound,"applied_families":cp(root_factors)},"forward_occurrence":fo,"action_effect_occurrence":ao,"recovery_usage_binding":cp(binding),"local_recovery_usage":None if local_bound is None else usage_for(binding,local_bound["upper"]["upper"]),"root_recovery_usage":usage_for(binding,root_bound["upper"]),"guard_compatibility":guard_fact,"guard_coordinates":coords,"eligibility_evidence":"proven" if root_bound["upper"].get("finite")==0 else "requires_evidence","control_path":cp(path)})
            for index,child in enumerate(current["children"]):
                detail={};reach="possible"
                kind={"sequence":"sequence_child","select":"select_case","parallel":"parallel_branch","race":"race_branch","iterate":"iterate_body","retry":"retry_child","timeout":"timeout_child","handle":"handle_protected" if index==0 else "handle_clause","scope":"scope_child","compensation_scope":"compensation_scope_child"}[op]
                body=next(iter(self.nodes[node]["body"].values()))
                if op=="retry":detail={"max_attempts":current["operator_semantics"]["max_attempts"]}
                if op in {"race","parallel"}:detail={"branch_ordinal":int(body["branches"][index]["ordinal"])}
                if op=="handle" and index>0:detail={"ordinal":int(body["clauses"][index-1]["ordinal"])}
                if op=="select":
                    decision=current["interface"]["select_decision"];ordinal=int(body["cases"][index]["ordinal"])
                    detail={"case_ordinal":ordinal,"selector":decision["expression"],"selector_dependencies":cp(decision["dependencies"]),"exact_case":decision["exact_case"]}
                    if decision["exact_case"] is not None:reach="exact_selected" if ordinal==decision["exact_case"] else "proven_unreachable"
                step={"parent_node":node,"child_node":child["computation"],"child_index":index,"kind":kind,"detail":detail,"reachability":reach}
                walk(child,owner,cp(local_factors),cp(root_factors),path+[step],reachable and reach!="proven_unreachable")
        walk(tree,None,[],[],[])
        return actions

    def compensation_facts(self,value,node):
        actions=self.source_actions(value)
        locals=[];potential=[]
        for action in actions:
            if action["owner"].get("node")!=node:continue
            local={"source":cp(action["source"]),"forward_seed_upper":cp(action["forward_seed_upper"]),"owner_scope":node,"action_kind":action["source"]["action"]["action_kind"],"local_bound":cp(action["local_bound"]),"forward_occurrence":cp(action["forward_occurrence"]),"action_effect_occurrence":cp(action["action_effect_occurrence"]),"recovery_usage_binding":cp(action["recovery_usage_binding"]),"recovery_usage":cp(action["local_recovery_usage"]),"guard_compatibility":cp(action["guard_compatibility"]),"eligibility_evidence":action["eligibility_evidence"]}
            locals.append(local)
            if local["action_kind"]=="compensation":
                ceiling=local["local_bound"]["upper"]["upper"]
                potential.append({"source_node":action["source"]["source_node"],"identity":cp(action["source"]["action"]),"forward_occurrence":cp(action["forward_occurrence"]),"action_effect_occurrence":cp(action["action_effect_occurrence"]),"local_bound":cp(local["local_bound"]),"potential_effect":effect_entry(action["source"]["action"]["action_effect"],ceiling,action["forward_occurrence"]["declared"]["guard"]),"potential_recovery_usage":cp(action["local_recovery_usage"]),"receipt_evidence":action["eligibility_evidence"]})
        ap=[proof("conditional-compensation-actions","requires_evidence" if any(item["receipt_evidence"]!="proven" for item in potential) else "proven",150,node,f"computation:{node}")]
        # The dimension contains conditional LocalRecoveryAction objects, not
        # just the arithmetic ceiling. Exact integers do not discharge their
        # explicit eligibility_evidence (0150 / 0155 proof boundary).
        lp=[proof("local-k-action-bounds",overall([{"verdict":item["eligibility_evidence"]} for item in locals]),155,node,f"computation:{node}")]
        total={"finite":0}
        for item in potential:total=up_add(total,item["local_bound"]["upper"]["upper"])
        order="proven" if total.get("finite",2)<=1 else "requires_evidence"
        result={"potential_actions":{"state":"complete","actions":potential,"proofs":ap},"local_recovery_actions":{"state":"complete","actions":locals,"proofs":lp},"compensation_order":order,"recovery_blocked_outcome":"requires_evidence" if potential else "unproven","forward_effects_retained":True,"forward_outcome_retained":True}
        proofs=[proof("compensation-forward-history","proven",149,node,f"computation:{value['direct_children'][0]}"),*ap,*lp,proof("compensation-commit-order",order,151,node,f"computation:{node}"),proof("recovery-blocked-outcome",result["recovery_blocked_outcome"],150,node,f"computation:{node}")]
        return result,proofs

    def recovery(self,root):
        actions=self.source_actions(root)
        if not actions:return super().recovery(root)
        by_node={}
        for action in actions:by_node.setdefault(action["source"]["source_node"],[]).append(action)
        def fold(node,which):
            if node["leaf_summary"] is not None:
                e=cp(EMPTY_E);u=cp(EMPTY_U);forward_added=set()
                for action in by_node.get(node["computation"],[]):
                    binding=action["recovery_usage_binding"];base=action_base(action["forward_seed_upper"],binding)
                    entry=effect_entry(action["source"]["action"]["action_effect"],base,action["forward_occurrence"]["declared"]["guard"])
                    e=effect_combine(e,{"row":"closed","entries":[entry]})
                    forward_key=action["source"]["forward_entry_ordinal"]
                    if which=="requirement" and action["source"]["action"]["forward_effect"]==action["source"]["action"]["action_effect"] and forward_key not in forward_added:
                        e=effect_combine(e,{"row":"closed","entries":[effect_entry(entry["atom"],action["forward_seed_upper"],entry["guard"])]})
                        forward_added.add(forward_key)
                    contribution=usage_for(binding,base)
                    if contribution["coordinates"]:u=contribution if not u["coordinates"] else usage_control("sequence",[u,contribution],{})
                return u if which=="usage" else e
            children=[fold(child,which) for child in node["children"]]
            op=node["operator"];body=cp(next(iter(self.nodes[node["computation"]]["body"].values())))
            exact=None
            if op=="select" and node["interface"]["select_decision"]["exact_case"] is not None:
                exact=[int(item["ordinal"]) for item in body["cases"]].index(node["interface"]["select_decision"]["exact_case"])
            if which=="usage":
                # Only closed action-free leaves have the empty-contribution exception.
                nonempty=[child for child in children if child["coordinates"]]
                if not nonempty:return cp(EMPTY_U)
                if op in {"sequence","parallel","race"}:return usage_control(op,nonempty,body)
                return usage_control(op,children,body)
            return effect_control(op,children,body,exact)
        effects=fold(root,"action");required=fold(root,"requirement");usage=fold(root,"usage")
        declared=root["effects"]["value"]
        epsilon_verdict="proven"
        for entry in required["entries"]:
            witness=next((candidate for candidate in declared.get("entries",[]) if candidate["atom"]==entry["atom"] and ("unconditional" in candidate["guard"] or candidate["guard"]==entry["guard"])),None)
            if witness is None:epsilon_verdict="unproven"
            elif "finite" in witness["bounds"]["upper"] and ("finite" not in entry["bounds"]["upper"] or entry["bounds"]["upper"]["finite"]>witness["bounds"]["upper"]["finite"]):epsilon_verdict="violation"
        usage_verdict="proven"
        for coordinate in usage["coordinates"]:
            candidate=next((item for item in root["usage"]["value"]["coordinates"] if item["phase"]==coordinate["phase"] and item["meter"]==coordinate["meter"]),None)
            if candidate is None or coordinate["bound"]["state"]=="unknown":usage_verdict="unproven"
            elif candidate["bound"]["state"]=="known_finite" and (coordinate["bound"]["state"]!="known_finite" or coordinate["bound"]["upper"]>candidate["bound"]["upper"]):usage_verdict="violation"
        guard_verdict=overall([{"verdict":action["guard_compatibility"]["verdict"] if action["guard_compatibility"] else "unproven"} for action in actions])
        eligibility=overall([{"verdict":action["eligibility_evidence"]} for action in actions])
        sources=[source for item in root["recovery_sources"]["contributions"] for source in item["sources"]]
        expected_bindings=Counter((source["source_node"],key(binding)) for source in sources for binding in source["action_bindings"])
        actual_bindings=Counter((action["source"]["source_node"],key(action["source"]["action"])) for action in actions)
        source_counter=Counter(source["source_node"] for source in sources)
        leaf_counter=Counter(self.leaf_order)
        expected_profiles=Counter((source["source_node"],key(binding)) for source in sources for binding in source["usage_bindings"])
        actual_profiles=Counter((action["source"]["source_node"],key(action["recovery_usage_binding"])) for action in actions)
        sources_accounted="proven" if source_counter==leaf_counter and not root["recovery_sources"]["unresolved_sources"] else "unproven"
        bindings_accounted="proven" if expected_bindings==actual_bindings and sources_accounted=="proven" else "unproven"
        profiles_accounted="proven" if expected_profiles==actual_profiles and sources_accounted=="proven" else "unproven"
        factors_accounted="proven"
        for action in actions:
            expected=Counter((step["parent_node"],"retry" if step["kind"]=="retry_child" else "iterate") for step in action["control_path"] if step["kind"] in {"retry_child","iterate_body"})
            actual=Counter((family["identity"]["control_node"],family["identity"]["kind"]) for family in action["root_bound"]["applied_families"])
            if expected!=actual or any(family["identity"]["caller_semantic_ir_digest"]!=self.digest for family in action["root_bound"]["applied_families"]):factors_accounted="unproven"
        accounted=overall([{"verdict":verdict} for verdict in [sources_accounted,bindings_accounted,profiles_accounted,factors_accounted]])
        node=root["computation"]
        statuses=[("root-recovery-source-closure",accounted,154),("root-recovery-action-totals",accounted,155),("root-recovery-multiplicity",factors_accounted,155),("root-k-action-epsilon-coverage",epsilon_verdict,157),("root-k-recovery-usage-coverage",usage_verdict,158),("root-recovery-guard-compatibility",guard_verdict,156),("root-recovery-eligibility",eligibility,150)]
        proofs=[proof(label,status,rule,node,f"computation:{node}") for label,status,rule in statuses]
        def projection(value,label,status,rule):return dimension(value,[proof(label,status,rule,node,f"computation:{node}")])
        return {"actions":{"state":"complete","actions":actions,"proofs":cp(proofs)},"action_effects":projection(effects,"root-k-action-effect-envelope",epsilon_verdict,157),"epsilon_coverage_requirement":projection(required,"root-k-epsilon-coverage-requirement",epsilon_verdict,157),"recovery_usage":projection(usage,"root-k-recovery-usage",usage_verdict,158),"source_closure":{"source_nodes_accounted":sources_accounted,"action_bindings_accounted":bindings_accounted,"usage_profiles_accounted":profiles_accounted,"multiplicity_factors_accounted":factors_accounted,"root_action_totals_sound":accounted},"proof_dimensions":proofs,"overall_verdict":overall(proofs)}
