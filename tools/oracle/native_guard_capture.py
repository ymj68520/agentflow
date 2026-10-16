"""Independent R3-NG semantics. Inputs are declarations, never Rust facts.

This is a LOCAL observation comparator, not a G01 wire codec, normalizer,
Scope encoder or D10 oracle. Existing independent AFIDL/contract encoders
are reused only for input canonicalization and schema pins. No Rust process
is invoked when constructing Expected. Diagnostic reason is retained in
Actual but excluded explicitly from semantic comparison.
"""
import copy
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
_spec = importlib.util.spec_from_file_location(
    "capture_contract_input", ROOT / "tools/gen-component-contract-corpus.py")
CC = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(CC)
MAX = (1 << 64) - 1
NQ = {"not_queried": None}
ABSENT = {"absent": None}


def present(x):
    return {"present": x}


def ceiling(n):
    return {"unbounded": None} if n is None else {"finite": n}


def finite(c):
    return c.get("finite")


def bounds(lo, hi):
    return {"lower": lo, "upper": ceiling(hi)}


def operation(op, a, b):
    """Finite overflow is an error, NOT infinity; zero precedes infinity."""
    a, b = finite(a), finite(b)
    if op == "mul" and (a == 0 or b == 0):
        return ceiling(0)
    if a is None or b is None:
        return ceiling(None)
    n = a * b if op == "mul" else a + b
    return None if n > MAX else ceiling(n)


def schema(primitive):
    return {"schema": "agentflow.test.guard-types@1", "name": "CaptureInput",
            "body": ("alias", ("primitive", primitive))}


def reference(s):
    return {"specification_id": s["schema"], "type_name": s["name"],
            "schema_digest": {"algorithm": "sha256", "domain": "type-schema",
                              "hex": CC.afidl.type_schema_digest(s)[1]}}


def schema_entry(s):
    return {"specification_id": s["schema"], "type_name": s["name"],
            "digest": reference(s)["schema_digest"],
            "body": json.loads(CC.afidl.canonical_ace1_bytes(s))["body"]}


def condition(primitive="int64", value=0, port=1):
    return {"input_equals": {"port_no": port, "expected": {primitive: value}}}


def base(primitive="int64"):
    """Full independent thirteen-field declaration, not a Rust snapshot."""
    c = CC.empty_contract()
    f = CC.atom(name="charge", qualifier=None)
    a = CC.atom(name="reverse", qualifier=None)
    identity = {"forward_effect": f, "action_kind": "compensation", "action_effect": a}
    c["inputs"] = [{"port_no": 1, "type": reference(schema(primitive))}]
    c["effects"]["row"] = {"closed": [
        {"atom": f, "bounds": bounds(1, 2), "guard": {"unconditional": None}},
        {"atom": a, "bounds": bounds(0, 2), "guard": condition(primitive, None if primitive in ("unit", "null") else 0)}]}
    resource = next(t for t in CC.afidl.compile_file(ROOT / "spec/idl/semantic-core.afidl")
                    if t["name"] == "TypeSchemaRef")
    c["resources"]["requirements"] = [{"port_no": 1, "resource_type": {"schema": reference(resource)},
                                        "required_attributes": []}]
    c["recovery"]["effect_recovery_clauses"] = [{
        "effect": f, "idempotency": "natural", "key_requirement": {"none": None},
        "reconciliation": "none", "compensation": {"required": a["operation"]},
        "unknown_effect_handling": "block"}]
    meter = {"catalog": "agentflow.meters@1", "name": "api_calls"}
    c["usage"]["limits"] = [{"meter": meter, "phase": "recovery", "upper": ceiling(12)}]
    c["recovery_action_bindings"] = [{"identity": identity}]
    c["recovery_usage_bindings"] = [{"action": identity, "activations_per_forward": bounds(1, 1),
                                    "attempts_per_activation": bounds(1, 2), "per_attempt_usage": {
                                        "limits": [{"meter": meter, "phase": "recovery", "upper": ceiling(3)}]}}]
    return copy.deepcopy(c)


def vectors():
    rows = []

    def add(name, c, primitive="int64", admission=True):
        rows.append({"case_id": "r3-ng/" + name, "schema_primitive": primitive,
                     "raw_declaration": c, "canonical_admission": admission})

    c = base()
    add("typed-int64", c)
    for primitive, values in {
        "unit": [None], "null": [None], "bool": [False, True],
        "int64": [-(1 << 63), -1, (1 << 63) - 1],
        "decimal": ["0", "-1.5"], "string": ["", "甲"],
        "bytes": ["", "AQ"], "timestamp": [0, (1 << 63) - 1],
        "duration": [-1, (1 << 63) - 1],
    }.items():
        for i, value in enumerate(values):
            c = base(primitive)
            c["effects"]["row"]["closed"][1]["guard"] = condition(primitive, value)
            add(f"primitive-{primitive}-{i}", c, primitive)
    for name in ["contained", "equal-guards", "contrary-same-port", "independent-ports",
                 "zero", "zero-missing-port", "zero-times-unbounded", "missing-port",
                 "type-mismatch", "schema-expand", "eager-U-before-V", "eager-V-before-U",
                 "missing-forward", "missing-action", "missing-both", "missing-usage",
                 "first-product-overflow", "second-product-overflow", "unbounded",
                 "empty-bindings", "open-row", "unknown-row", "reordered-input",
                 "bad-schema-pin", "duplicate-input", "bad-bytes"]:
        p = "uint32" if name in ("schema-expand", "eager-U-before-V", "eager-V-before-U") else "int64"
        c = base(p)
        es = c["effects"]["row"]["closed"]
        es[1]["guard"] = condition("int64", 0)
        u = c["recovery_usage_bindings"][0]
        if name == "contained": es[1]["guard"] = {"unconditional": None}
        if name == "equal-guards": es[0]["guard"] = copy.deepcopy(es[1]["guard"])
        if name == "contrary-same-port": es[0]["guard"] = condition(value=1)
        if name == "independent-ports":
            c["inputs"].append({"port_no": 2, "type": reference(schema(p))})
            es[0]["guard"] = condition(port=2)
        if name in ("zero", "zero-missing-port", "zero-times-unbounded"):
            u["attempts_per_activation"] = bounds(0, 0)
        if name == "zero-times-unbounded": es[0]["bounds"] = bounds(0, None)
        if name in ("missing-port", "zero-missing-port", "eager-U-before-V"):
            es[1]["guard"] = condition(port=2)
        if name == "type-mismatch": es[1]["guard"] = condition("bool", True)
        if name == "eager-U-before-V": es[0]["guard"] = condition()
        if name == "eager-V-before-U": es[0]["guard"] = condition(port=2)
        if name == "missing-forward": es.pop(0)
        if name == "missing-action": es.pop(1)
        if name == "missing-both": es.clear()
        if name == "missing-usage": c["recovery_usage_bindings"].clear()
        if name == "first-product-overflow": u["activations_per_forward"] = bounds(1, MAX)
        if name == "second-product-overflow": u["attempts_per_activation"] = bounds(1, MAX)
        if name == "unbounded": u["attempts_per_activation"] = bounds(1, None)
        if name == "empty-bindings":
            c["recovery_action_bindings"].clear(); c["recovery_usage_bindings"].clear()
        if name == "open-row": c["effects"]["row"] = {"open": {"known": es, "residual": []}}
        if name == "unknown-row": c["effects"]["row"] = {"unknown": None}
        if name == "reordered-input": es.reverse()
        if name == "bad-schema-pin": c["inputs"][0]["type"]["schema_digest"]["hex"] = "0" * 64
        if name == "duplicate-input": c["inputs"].append(copy.deepcopy(c["inputs"][0]))
        if name == "bad-bytes": es[1]["guard"] = condition("bytes", "not=base64")
        add(name, c, p, name not in ("bad-schema-pin", "duplicate-input", "bad-bytes"))
    for name in ["ready", "lower-conflict", "product-overflow", "sum-overflow", "divisor-overflow",
                 "foreign-forward", "missing-peer-usage", "infinite-divisor", "infinite-ceiling"]:
        c = base()
        identity = c["recovery_action_bindings"][0]["identity"]
        identity["action_effect"] = copy.deepcopy(identity["forward_effect"])
        u = c["recovery_usage_bindings"][0]
        u["action"] = copy.deepcopy(identity)
        es = c["effects"]["row"]["closed"]
        es.pop(); es[0]["bounds"] = bounds(0, 12)
        if name == "lower-conflict": es[0]["bounds"] = bounds(12, 12)
        if name == "product-overflow": u["activations_per_forward"] = bounds(1, MAX)
        if name == "divisor-overflow":
            u["activations_per_forward"] = bounds(1, MAX); u["attempts_per_activation"] = bounds(1, 1)
        if name == "infinite-divisor": u["attempts_per_activation"] = bounds(1, None)
        if name == "infinite-ceiling": es[0]["bounds"] = bounds(0, None)
        if name in ("sum-overflow", "foreign-forward", "missing-peer-usage"):
            peer = copy.deepcopy(identity); peer["action_kind"] = "reconciliation"
            if name == "foreign-forward": peer["forward_effect"]["operation"]["name"] = "reverse"
            c["recovery_action_bindings"].append({"identity": peer})
            pu = copy.deepcopy(u); pu["action"] = peer
            if name == "sum-overflow":
                u["activations_per_forward"] = bounds(1, MAX); u["attempts_per_activation"] = bounds(1, 1)
                pu["activations_per_forward"] = bounds(1, 1); pu["attempts_per_activation"] = bounds(1, 1)
            if name != "missing-peer-usage": c["recovery_usage_bindings"].append(pu)
        add("same-atom-" + name, c)
    return rows


def port_result(c, schemas, guard):
    if "unconditional" in guard: return {"unconditional_not_required": None}
    g = guard["input_equals"]
    p = next((p for p in c["inputs"] if p["port_no"] == g["port_no"]), None)
    if p is None: return {"missing_port": {"guard": g, "input_declarations": c["inputs"]}}
    s = schemas.get((p["type"]["specification_id"], p["type"]["type_name"]))
    if s is None or reference(s) != p["type"]:
        return {"unresolved_schema": {"port": p, "guard": g, "step": "resolve"}}
    primitive = s["body"][1][1]
    if primitive not in ("unit", "null", "bool", "int64", "decimal", "string", "bytes", "timestamp", "duration"):
        return {"unresolved_schema": {"port": p, "guard": g, "step": "expand"}}
    kind = "valid" if primitive == next(iter(g["expected"])) else "type_mismatch"
    return {kind: {"port": p, "guard": g, "resolved": schema_entry(s), "expanded": {"primitive": primitive}}}


def alternate(value):
    k, v = next(iter(value.items()))
    if k in ("unit", "null"): return None
    if k == "bool": n = not v
    elif k in ("int64", "timestamp", "duration"): n = v + 1 if v < (1 << 63) - 1 else v - 1
    elif k == "decimal": n = "1" if v == "0" else "0"
    elif k == "string": n = "x" if v == "" else ""
    elif k == "bytes": n = "AQ" if v == "" else ""
    else: raise ValueError(k)
    return {k: n}


def run(c, schemas, ordinal):
    identity = c["recovery_action_bindings"][ordinal]["identity"]
    row_kind, payload = next(iter(c["effects"]["row"].items()))
    es = payload if row_kind == "closed" else payload["known"] if row_kind == "open" else []
    fi = next((i for i, e in enumerate(es) if CC.atom_key(e["atom"]) == CC.atom_key(identity["forward_effect"])), None)
    ai = next((i for i, e in enumerate(es) if CC.atom_key(e["atom"]) == CC.atom_key(identity["action_effect"])), None)
    occurrence = lambda i: ABSENT if i is None else present({"ordinal": i, "entry": es[i]})
    r = {"binding_ordinal": ordinal, "identity": identity, "row_kind": "open_known" if row_kind == "open" else row_kind,
         "forward": occurrence(fi), "action": occurrence(ai), "usage": NQ, "bounds": None,
         "forward_input": NQ, "action_input": NQ, "primary_error": None, "containment": NQ,
         "branch": None, "record": {"source_component": c["component_identity"], "recovery_action": identity,
             "forward_entry_ordinal": fi, "action_entry_ordinal": ai,
             "forward_guard": None if fi is None else es[fi]["guard"],
             "declared_action_guard": None if ai is None else es[ai]["guard"],
             "action_upper": None, "verdict": "unproven", "basis": "insufficient_evidence", "countermodel": None}}

    def end(branch, verdict="unproven", basis="insufficient_evidence"):
        r["branch"] = branch; r["record"]["verdict"] = verdict; r["record"]["basis"] = basis
        return r

    if fi is None or ai is None: return end("missing_entries")
    us = c["recovery_usage_bindings"]
    ui = next((i for i, u in enumerate(us) if u["action"] == identity), None)
    r["usage"] = ABSENT if ui is None else present({"ordinal": ui, "binding": us[ui]})
    if ui is None: return end("missing_usage")
    u = us[ui]; declared = es[fi]["bounds"]
    b = {"declared_forward_bounds": declared, "consulted_peer_bindings": [], "arithmetic": [],
         "sum": NQ, "divisor": NQ, "effective_upper": NQ, "forward_times_activation": NQ,
         "partial_times_attempt": NQ, "action_upper": NQ, "failure": None, "unisolated": None}
    r["bounds"] = b

    def arithmetic(kind, left, right, peer=None):
        result = operation("mul" if kind in ("peer_product", "forward_times_activation", "partial_times_attempt") else "add", left, right)
        b["arithmetic"].append({"kind": kind, "binding_ordinal": peer, "left": left, "right": right, "result": result})
        return result

    upper = declared["upper"]
    if CC.atom_key(identity["forward_effect"]) == CC.atom_key(identity["action_effect"]):
        total = ceiling(0); processed = []; b["sum"] = present(total)
        for i, peer in enumerate(c["recovery_action_bindings"]):
            ident = peer["identity"]
            if CC.atom_key(ident["action_effect"]) != CC.atom_key(identity["forward_effect"]): continue
            q = {"binding_ordinal": i, "binding": peer, "usage": NQ}
            b["consulted_peer_bindings"].append(q)
            if CC.atom_key(ident["forward_effect"]) != CC.atom_key(identity["forward_effect"]):
                b["unisolated"] = {"foreign_forward": q}; b["effective_upper"] = ABSENT
                return end("unisolated_forward")
            j = next((j for j, pu in enumerate(us) if pu["action"] == ident), None)
            q["usage"] = ABSENT if j is None else present({"ordinal": j, "binding": us[j]})
            if j is None:
                b["unisolated"] = {"missing_peer_usage": q}; b["effective_upper"] = ABSENT
                return end("unisolated_forward")
            activation = us[j]["activations_per_forward"]["upper"]; attempt = us[j]["attempts_per_activation"]["upper"]
            product = arithmetic("peer_product", activation, attempt, i)
            if product is None:
                b["failure"] = {"same_atom_product_overflow": {"binding_ordinal": i, "activation_upper": activation, "attempt_upper": attempt}}
                b["effective_upper"] = ABSENT
                return end("same_atom_bounds_failure", "violation")
            new_total = arithmetic("peer_sum", total, product, i)
            if new_total is None:
                b["failure"] = {"same_atom_sum_overflow": {"processed_binding_ordinals": processed[:], "before": total, "next": product}}
                b["sum"] = ABSENT; b["effective_upper"] = ABSENT
                return end("same_atom_bounds_failure", "violation")
            total = new_total; processed.append(i); b["sum"] = present(total)
        divisor = arithmetic("divisor", ceiling(1), total)
        b["divisor"] = ABSENT if divisor is None else present(divisor)
        if divisor is None:
            b["failure"] = {"same_atom_divisor_overflow": {"sum": total}}; b["effective_upper"] = ABSENT
            return end("same_atom_bounds_failure", "violation")
        upper = ceiling(None if finite(upper) is None else 0 if finite(divisor) is None else finite(upper) // finite(divisor))
        b["effective_upper"] = present(upper)
        if finite(upper) is not None and finite(upper) < declared["lower"]:
            b["failure"] = {"same_atom_lower_conflict": {"declared_bounds": declared, "sum": total, "divisor": divisor, "effective_upper": upper}}
            return end("same_atom_bounds_failure", "violation")
    else: b["effective_upper"] = present(upper)
    effective = upper
    for step, factor in [("forward_times_activation", u["activations_per_forward"]["upper"]),
                         ("partial_times_attempt", u["attempts_per_activation"]["upper"])]:
        result = arithmetic(step, upper, factor)
        b[step] = ABSENT if result is None else present(result)
        if result is None:
            b["failure"] = {"action_upper_overflow": {"step": step, "left": upper, "right": factor}}
            b["action_upper"] = ABSENT
            return end("action_upper_overflow", "violation")
        upper = result
    b["action_upper"] = present(upper); r["record"]["action_upper"] = upper
    fg, ag = es[fi]["guard"], es[ai]["guard"]
    for role, g in [("forward", fg), ("action", ag)]: r[role + "_input"] = present(port_result(c, schemas, g))
    for role in ("forward", "action"):
        result = r[role + "_input"]["present"]; tag = next(iter(result))
        if tag in ("missing_port", "unresolved_schema", "type_mismatch"):
            r["primary_error"] = role
            return end("schema_unresolved" if tag == "unresolved_schema" else tag, "unproven" if tag == "unresolved_schema" else "violation")
    if finite(upper) == 0: return end("zero_potential", "proven", "zero_possible_actions")
    contained = "unconditional" in ag or fg == ag
    r["containment"] = present(contained)
    if contained: return end("frozen_containment", "proven", "frozen_guard_containment")
    assignments = {}
    if "input_equals" in fg:
        g = fg["input_equals"]; p = next(p for p in c["inputs"] if p["port_no"] == g["port_no"])
        assignments[g["port_no"]] = {"port_no": g["port_no"], "type_ref": p["type"], "value": g["expected"]}
    g = ag["input_equals"]; existing = assignments.get(g["port_no"])
    if existing is None or existing["value"] == g["expected"]:
        value = alternate(g["expected"])
        if value is None: return end("no_countermodel")
        p = next(p for p in c["inputs"] if p["port_no"] == g["port_no"])
        assignments[g["port_no"]] = {"port_no": g["port_no"], "type_ref": p["type"], "value": value}
    holds = lambda guard: "unconditional" in guard or assignments.get(guard["input_equals"]["port_no"], {}).get("value") == guard["input_equals"]["expected"]
    if not holds(fg) or holds(ag): return end("no_countermodel")
    r["record"]["countermodel"] = {"source_contract": c, "source_component": c["component_identity"],
        "recovery_action": identity, "forward_entry_ordinal": fi, "action_entry_ordinal": ai,
        "forward_guard": fg, "declared_action_guard": ag, "forward_bounds": {"lower": declared["lower"], "upper": effective},
        "activation_upper": u["activations_per_forward"]["upper"], "attempt_upper": u["attempts_per_activation"]["upper"],
        "action_upper": upper, "assignments": [assignments[k] for k in sorted(assignments)],
        "reachability": "well_typed_component_input_domain", "related_rule_id": "AF-CONTRACT-0022"}
    return end("typed_countermodel", "violation", "typed_countermodel")


def wire_numbers(x):
    if isinstance(x, bool): return x
    if isinstance(x, int): return str(x)
    if isinstance(x, dict): return {k: wire_numbers(v) for k, v in x.items()}
    if isinstance(x, (list, tuple)): return [wire_numbers(v) for v in x]
    return x


def rust_input(x):
    if isinstance(x, dict):
        if x == {"none": None}: return None
        if x in ({"unconditional": None}, {"unbounded": None}, {"unknown": None}): return next(iter(x))
        return {k: rust_input(v) for k, v in x.items()}
    if isinstance(x, list): return [rust_input(v) for v in x]
    return x


def expected(v):
    if not v["canonical_admission"]: return {"canonical_admission": False}
    c = CC.canonical_contract(v["raw_declaration"])
    s = schema(v["schema_primitive"])
    schemas = {(s["schema"], s["name"]): s}
    return wire_numbers({"canonical_admission": True, "canonical_declaration": c,
                         "canonical_bytes_hex": CC.ace1(c).encode().hex(),
                         "runs": [run(c, schemas, i) for i in range(len(c["recovery_action_bindings"]))]})


def first_difference(a, b, path="$"):
    if type(a) is not type(b): return path + ": type"
    if isinstance(a, dict):
        if a.keys() != b.keys(): return path + ": keys " + repr(sorted(a.keys() ^ b.keys()))
        for key in a:
            if (d := first_difference(a[key], b[key], path + "." + key)): return d
    elif isinstance(a, list):
        if len(a) != len(b): return path + ": length"
        for i, (x, y) in enumerate(zip(a, b)):
            if (d := first_difference(x, y, path + f"[{i}]")): return d
    elif a != b: return path + ": " + repr(a) + " != " + repr(b)
    return None


def compare_all(actual):
    """Always examine every case, including failures; never waive a mismatch."""
    source = {v["case_id"]: v for v in vectors()}
    assert len(actual) == len(source), "missing/extra actual cases"
    ids = set(); mismatches = []; matched = 0; branches = set(); countermodels = 0
    for row in actual:
        case_id = row["case_id"]
        assert case_id not in ids and case_id in source; ids.add(case_id)
        want = expected(source[case_id]); got = copy.deepcopy(row["actual"])
        for r in got.get("runs", []):
            r["record"].pop("reason")
            branches.add(r["branch"])
            countermodels += r["record"]["countermodel"] is not None
        difference = first_difference(want, got)
        if difference:
            first = "$.canonical_admission" if want["canonical_admission"] != got["canonical_admission"] else difference
            mismatches.append({"case_id": case_id, "first_differing_field": first,
                               "difference": difference, "expected": want, "actual": row["actual"]})
        else: matched += 1
    return {"outcome": "B" if mismatches else "NATIVE_TRACE_ONLY_MATCH",
            "native_semantic_cases_executed": len(ids), "matched": matched,
            "unexpected_mismatch": len(mismatches), "mismatches": mismatches,
            "actual_branches": sorted(branches), "actual_countermodels": countermodels,
            "countermodel_fields_per_instance": 14, "assignment_fields_per_instance": 3,
            "scope": "PendingG02", "full_seed_conformance": False,
            "r3_ng_complete": False, "r3_all_complete": False, "positive_capability": "NotAuthorized"}


def main():
    import sys
    if sys.argv[1:] == ["--vectors"]:
        print(json.dumps([{**v, "rust_input": rust_input(v["raw_declaration"])} for v in vectors()], ensure_ascii=True))
        return
    report = compare_all(json.load(sys.stdin))
    print(json.dumps(report, ensure_ascii=True))
    # --report is a diagnostic report command, NEVER a passing conformance
    # gate. The default semantic gate exits 1 for any disagreement.
    if report["unexpected_mismatch"] and sys.argv[1:] != ["--report"]: sys.exit(1)


if __name__ == "__main__": main()
