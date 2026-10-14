"""Independent E/Phi/U models for frozen static control rules 0127-0146.

Inputs are declaration-derived normalized envelopes. Effects retain full atom,
guard and residual identity; Phi contains nominal business failures only; Usage
uses exact phase/meter keys. Missing/unknown/unbounded are separate variants.
Arithmetic checks U64 before producing a value. No Rust execution or Actual is
read by this module. Unsupported facts fail with OracleGap, never truncation.
"""

import copy
import json
import re

U64_MAX = 2**64 - 1


class OracleGap(ValueError):
    pass


def u64(value):
    if type(value) not in {int,str} or isinstance(value,str) and re.fullmatch(r"0|[1-9][0-9]*",value) is None:
        raise ValueError("U64 requires an exact integer/canonical machine numeral")
    value = int(value)
    if not 0 <= value <= U64_MAX:
        raise OverflowError("formal U64 overflow")
    return value


def checked_add(a,b):
    return u64(u64(a)+u64(b))


def checked_mul(a,b):
    return u64(u64(a)*u64(b))


def key(value):
    return json.dumps(value,sort_keys=True,separators=(",",":"))


def unique(values):
    return [copy.deepcopy(value) for _,value in sorted({key(value):value for value in values}.items())]


def upper(raw):
    return {"finite":u64(raw["finite"])} if "finite" in raw else {"unbounded":None}


def up_add(a,b):
    return {"unbounded":None} if "unbounded" in a or "unbounded" in b else {"finite":checked_add(a["finite"],b["finite"])}


def up_max(a,b):
    return {"unbounded":None} if "unbounded" in a or "unbounded" in b else {"finite":max(a["finite"],b["finite"])}


def up_mul(a,b):
    if a.get("finite") == 0 or b.get("finite") == 0:
        return {"finite":0}
    return {"unbounded":None} if "unbounded" in a or "unbounded" in b else {"finite":checked_mul(a["finite"],b["finite"])}


def atom(raw):
    if re.fullmatch(r"[a-z0-9]+(?:[-.][a-z0-9]+)*",raw["operation"]["name"]) is None:
        raise ValueError("invalid frozen EffectOperation name grammar")
    result = copy.deepcopy(raw)
    result["resource_selector"]["requirement_port_no"] = u64(raw["resource_selector"]["requirement_port_no"])
    result["qualifiers"] = [{"family":family,"member":member} for item in raw["qualifiers"] for family,member in item.items()]
    return result


def guard(raw):
    if "unconditional" in raw:
        return {"unconditional":True}
    item = raw["input_equals"]
    constant = copy.deepcopy(item["expected"])
    for kind in {"int64","timestamp","duration"} & constant.keys():
        constant[kind] = int(constant[kind])
    return {"input_equals":{"port_no":u64(item["port_no"]),"expected":constant}}


def declared_effect(raw):
    row = raw["row"]
    if "unknown" in row:
        return {"row":"unknown"}
    def entry(item):
        return {"atom":atom(item["atom"]),"bounds":{"lower":u64(item["bounds"]["lower"]),"upper":upper(item["bounds"]["upper"])},"guard":guard(item["guard"])}
    if "closed" in row:
        return {"row":"closed","entries":[entry(item) for item in row["closed"]]}
    known = [entry(item) for item in row["open"]["known"]]
    residual = copy.deepcopy(row["open"]["residual"])
    for item in residual:
        item["selector"]["requirement_port_no"] = u64(item["selector"]["requirement_port_no"])
        item["bounds"] = {"lower":u64(item["bounds"]["lower"]),"upper":upper(item["bounds"]["upper"])}
    return {"row":"open","known":known,"residual":residual}


def effect_zero_lower(effect):
    value = copy.deepcopy(effect)
    for item in value.get("entries",value.get("known",[])) + value.get("residual",[]):
        item["bounds"]["lower"] = 0
    return value


def effect_combine(left,right,join=False):
    if left["row"] == "unknown" or right["row"] == "unknown":
        return {"row":"unknown"}
    def merge_rows(a,b,identity,guarded):
        aa,bb = {key(identity(item)):item for item in a},{key(identity(item)):item for item in b}
        out = []
        for identity_key in sorted(aa.keys() | bb.keys()):
            x,y = aa.get(identity_key),bb.get(identity_key)
            if x is None or y is None:
                item = copy.deepcopy(x or y)
                if join:
                    item["bounds"]["lower"] = 0
            else:
                item = copy.deepcopy(x)
                same_guard = not guarded or x["guard"] == y["guard"]
                lower = min(x["bounds"]["lower"],y["bounds"]["lower"]) if join else checked_add(x["bounds"]["lower"],y["bounds"]["lower"])
                item["bounds"] = {"lower":lower if same_guard else 0,"upper":(up_max if join else up_add)(x["bounds"]["upper"],y["bounds"]["upper"])}
                if guarded and not same_guard:
                    item["guard"] = {"unconditional":True}
            out.append(item)
        return out
    entries = merge_rows(left.get("entries",left.get("known",[])),right.get("entries",right.get("known",[])),lambda item:item["atom"],True)
    residual = merge_rows(left.get("residual",[]),right.get("residual",[]),lambda item:{"catalog":item["catalog"],"selector":item["selector"]},False)
    return {"row":"open","known":entries,"residual":residual} if left["row"] == "open" or right["row"] == "open" else {"row":"closed","entries":entries}


def effect_repeat(effect,lower_count,upper_count):
    value = copy.deepcopy(effect)
    for item in value.get("entries",value.get("known",[])) + value.get("residual",[]):
        item["bounds"] = {"lower":checked_mul(item["bounds"]["lower"],lower_count),"upper":up_mul(item["bounds"]["upper"],upper_count)}
    return value


def effect_control(op,values,body,exact=None):
    values = copy.deepcopy(values)
    if not values:
        return None
    if op == "select":
        if exact is not None:
            return values[exact]
        values = [effect_zero_lower(value) for value in values]
    if op == "sequence":
        values = values[:1] + [effect_zero_lower(value) for value in values[1:]]
    if op == "race" or op == "parallel" and body["mode"] == "fail_fast":
        values = [effect_zero_lower(value) for value in values]
    if op == "iterate":
        variant,item = next(iter(body.items()))
        count = u64(item["max_iterations"]) if variant == "while" else item.get("_static_count")
        return effect_repeat(values[0],0,{"finite":count} if count is not None else {"unbounded":None})
    if op == "retry":
        return effect_repeat(values[0],1,{"finite":checked_add(1,len(body["delays"]))})
    if op in {"scope","compensation_scope","timeout"}:
        return values[0]
    if op == "handle":
        handler = {"row":"closed","entries":[]}
        for value in values[1:]:
            handler = effect_combine(handler,value,True)
        return effect_combine(values[0],handler)
    result = values[0]
    for value in values[1:]:
        result = effect_combine(result,value,op == "select")
    return result


def declared_usage(limits):
    coordinates = []
    for item in limits:
        bound = {"state":"known_finite","upper":u64(item["upper"]["finite"])} if "finite" in item["upper"] else {"state":"known_unbounded"}
        coordinates.append({"phase":item["phase"],"meter":copy.deepcopy(item["meter"]),"bound":bound})
    coordinates.sort(key=lambda item:(0 if item["phase"] == "forward" else 1,item["meter"]["catalog"],item["meter"]["name"]))
    return {"globally_unknown":False,"coordinates":coordinates}


def usage_combine(left,right,join=False):
    aa = {(item["phase"],item["meter"]["catalog"],item["meter"]["name"]):item for item in left["coordinates"]}
    bb = {(item["phase"],item["meter"]["catalog"],item["meter"]["name"]):item for item in right["coordinates"]}
    out=[]
    for k in sorted(aa.keys() | bb.keys(),key=lambda k:(0 if k[0]=="forward" else 1,k[1],k[2])):
        x,y=aa.get(k),bb.get(k)
        item=copy.deepcopy(x or y)
        if x is None or y is None or "unknown" in {x["bound"]["state"],y["bound"]["state"]}:
            item["bound"]={"state":"unknown"}
        elif "known_unbounded" in {x["bound"]["state"],y["bound"]["state"]}:
            item["bound"]={"state":"known_unbounded"}
        else:
            item["bound"]={"state":"known_finite","upper":(max if join else checked_add)(x["bound"]["upper"],y["bound"]["upper"])}
        out.append(item)
    return {"globally_unknown":left["globally_unknown"] or right["globally_unknown"],"coordinates":out}


def usage_repeat(value,count):
    result=copy.deepcopy(value)
    for item in result["coordinates"]:
        if count is None:
            item["bound"]={"state":"unknown"}
        elif item["bound"]["state"] == "known_finite":
            item["bound"]["upper"]=checked_mul(item["bound"]["upper"],count)
    return result


def usage_control(op,values,body):
    if not values:
        return None
    if op == "iterate":
        variant,item=next(iter(body.items()))
        return usage_repeat(values[0],u64(item["max_iterations"]) if variant=="while" else item.get("_static_count"))
    if op == "retry":
        return usage_repeat(values[0],checked_add(1,len(body["delays"])))
    if op in {"scope","compensation_scope","timeout"}:
        return copy.deepcopy(values[0])
    if op == "handle":
        coordinates=unique([{"phase":item["phase"],"meter":item["meter"],"bound":{"state":"known_finite","upper":0}} for value in values[1:] for item in value["coordinates"]])
        handler={"globally_unknown":False,"coordinates":coordinates}
        for value in values[1:]:
            handler=usage_combine(handler,value,True)
        return usage_combine(values[0],handler)
    result=copy.deepcopy(values[0])
    for value in values[1:]:
        result=usage_combine(result,value,op=="select")
    return result


def failure_control(op,values,body):
    if not values:
        return None
    if op in {"iterate","retry","scope","compensation_scope","timeout"}:
        return copy.deepcopy(values[0])
    if op == "handle":
        handled=[clause["match"]["failure"]["failure_type"] for clause in body["clauses"] if "failure" in clause["match"]]
        if not all(item in values[0] for item in handled):
            raise ValueError("HANDLE exact failure absent from protected row")
        members=unique([item for item in values[0] if item not in handled]+[item for value in values[1:] for item in value])
    else:
        members=unique([item for value in values for item in value])
    return sorted(members,key=lambda item:(item["specification_id"],item["type_name"],item["schema_digest"]["algorithm"],item["schema_digest"]["domain"],item["schema_digest"]["hex"]))
