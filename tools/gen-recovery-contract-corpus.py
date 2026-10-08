#!/usr/bin/env python3
"""Freeze the Phase 4A.6 static Recovery Contract corpus.

Recovery evidence in this oracle is a read-only test fact. It is never a
receipt or a runtime result created by the implementation.
"""

import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CONF = ROOT / "spec" / "conformance" / "recovery-contract"
# Revision 2 (Phase 4B.0A): full typed qualifier identity for clause
# canonicalization and ε matching; includes exact matches and both mismatch
# directions through the independent static Recovery oracle.
CORPUS_REVISION = 2

_sir_spec = importlib.util.spec_from_file_location(
    "sir", Path(__file__).resolve().parent / "gen-semantic-ir-corpus.py")
sir = importlib.util.module_from_spec(_sir_spec)
_sir_spec.loader.exec_module(sir)
afidl = sir.afidl

SEMANTIC_CORE = ROOT / "spec" / "idl" / "semantic-core.afidl"
EFFECT = ROOT / "spec" / "idl" / "effect-signature.afidl"
RECOVERY = ROOT / "spec" / "idl" / "recovery-contract.afidl"

EFFECT_CATALOG = "agentflow.test.recovery.effects@1"
RECOVERY_CATALOG = "agentflow.test.recovery.ops@1"
EFFECT_OPS = ["payment.charge", "payment.refund", "email.send", "inventory.reserve"]
RECOVERY_OPS = ["payment.reconcile", "inventory.reconcile"]

INVALID_IDEMPOTENCY_CONTRACT = "invalid_idempotency_contract"
DUPLICATE_EFFECT_RECOVERY_CLAUSE = "duplicate_effect_recovery_clause"
UNKNOWN_RECOVERY_OPERATION = "unknown_recovery_operation"
RECOVERY_EFFECT_NOT_DECLARED = "recovery_effect_not_declared"
UNSAFE_BLIND_RETRY = "unsafe_blind_retry"
RECOVERY_CONTRACT_UNPROVEN = "recovery_contract_unproven"


def load_types():
    sources = [("semantic-core.afidl", SEMANTIC_CORE.read_text(encoding="utf-8")),
               ("effect-signature.afidl", EFFECT.read_text(encoding="utf-8")),
               ("recovery-contract.afidl", RECOVERY.read_text(encoding="utf-8"))]
    _modules, by_module = afidl.compile_module_set(sources)
    types = {}
    for spec, schemas in by_module.items():
        for schema in schemas:
            types[f"{spec}/{schema['name']}"] = schema
    return types


TYPES = load_types()


def shape_of(identity):
    kind, payload = TYPES[identity]["body"]
    if kind == "alias":
        return expr_shape(payload)
    if kind == "enum":
        return ("enum", identity, {v["name"] for v in payload["values"]})
    if kind == "record":
        return ("record", identity,
                [(f["name"], expr_shape(f["type"])) for f in payload["fields"]])
    if kind == "variant":
        return ("variant", identity,
                [(c["name"], expr_shape(c["type"])) for c in payload["cases"]])
    raise AssertionError(kind)


def expr_shape(expr):
    kind, value = expr
    if kind == "primitive":
        return sir.PRIM_SHAPES[value]
    if kind == "ref":
        return shape_of(value)
    if kind == "list":
        return ("list", expr_shape(value))
    if kind == "option":
        return ("option", expr_shape(value))
    if kind == "map_string":
        return ("map", expr_shape(value))
    raise AssertionError(kind)


CONTRACT_SHAPE = shape_of("agentflow.recovery.contract@1/RecoveryContract")


def to_tree(shape, value):
    kind = shape[0]
    if kind == "variant":
        if isinstance(value, str):
            return {value: None}
        case, payload = next(iter(value.items()))
        return {case: to_tree(dict(shape[2])[case], payload)}
    if kind == "enum":
        return value
    if kind == "record":
        return {name: to_tree(field_shape, value[name])
                for name, field_shape in shape[2]}
    if kind == "list":
        return [to_tree(shape[1], item) for item in value]
    if kind == "option":
        return None if value is None else to_tree(shape[1], value)
    return value


def canonical_ace1(contract):
    tree = to_tree(CONTRACT_SHAPE, contract)
    return sir.encode_value(CONTRACT_SHAPE,
                            sir.json_to_value(CONTRACT_SHAPE, tree))


class RecoveryOracleError(Exception):
    def __init__(self, kind, detail):
        super().__init__(f"{kind}: {detail}")
        self.kind = kind


def atom(name="payment.charge", port=1, qualifier=None, scope=()):
    qualifiers = [] if qualifier is None else [{"idempotency": qualifier}]
    return {
        "operation": {"catalog": EFFECT_CATALOG, "name": name},
        "resource_selector": {
            "requirement_port_no": port,
            "scope_bindings": [{"key": k, "value": v} for k, v in scope],
        },
        "qualifiers": qualifiers,
    }


def key_req(all_true=True):
    return {
        "stable_across_attempts": all_true,
        "distinct_across_invocations": all_true,
        "bound_to_operation": all_true,
        "provider_honors": all_true,
    }


def clause(effect, idem="none", key=None, reconciliation="none",
           compensation=None, unknown="block"):
    return {
        "effect": effect,
        "idempotency": idem,
        "key_requirement": key,
        "reconciliation": reconciliation,
        "compensation": {"none": None} if compensation is None else
        {"required": {"catalog": EFFECT_CATALOG, "name": compensation}},
        "unknown_effect_handling": unknown,
    }


def contract(replay="recompute", clauses=()):
    return {"replay_semantics": replay, "effect_recovery_clauses": list(clauses)}


def fixture_catalogs():
    return {
        "effects": [{"catalog": EFFECT_CATALOG, "name": name}
                    for name in EFFECT_OPS],
        "recovery_operations": [{"catalog": RECOVERY_CATALOG, "name": name}
                                 for name in RECOVERY_OPS],
    }


QUALIFIER_FAMILY_CASES = {"idempotency": 1}
QUALIFIER_MEMBER_CASES = {"idempotency": {"natural": 1, "none": 2}}


def qualifier_identity(value):
    """Full typed qualifier identity: family, member and canonical payload."""
    identities = []
    for qualifier in value["qualifiers"]:
        family, member = next(iter(qualifier.items()))
        payload = json.dumps(member, ensure_ascii=False, sort_keys=True,
                             separators=(",", ":")).encode("utf-8")
        identities.append((QUALIFIER_FAMILY_CASES[family],
                           QUALIFIER_MEMBER_CASES[family][member], payload))
    return tuple(sorted(identities))


def atom_key(value):
    selector = value["resource_selector"]
    scope = tuple(sorted((x["key"], x["value"]) for x in selector["scope_bindings"]))
    return (value["operation"]["catalog"], value["operation"]["name"],
            selector["requirement_port_no"], scope,
            qualifier_identity(value))


def canonicalize(value):
    if value["replay_semantics"] not in {"recompute", "recorded", "reconcile", "forbidden"}:
        raise RecoveryOracleError("invalid_replay_semantics", "unknown replay mode")
    seen = set()
    clauses = []
    for current in value["effect_recovery_clauses"]:
        operation = current["effect"]["operation"]
        if operation["catalog"] != EFFECT_CATALOG or operation["name"] not in EFFECT_OPS:
            raise RecoveryOracleError(UNKNOWN_RECOVERY_OPERATION, "effect operation unknown")
        effect = json.loads(json.dumps(current["effect"]))
        effect["resource_selector"]["scope_bindings"] = sorted(
            effect["resource_selector"]["scope_bindings"],
            key=lambda x: (x["key"], x["value"]))
        effect["qualifiers"] = sorted(
            effect["qualifiers"],
            key=lambda qualifier: qualifier_identity(
                {"qualifiers": [qualifier]}))
        key = atom_key(effect)
        if key in seen:
            raise RecoveryOracleError(DUPLICATE_EFFECT_RECOVERY_CLAUSE, "duplicate effect clause")
        seen.add(key)
        if current["idempotency"] == "keyed" and current["key_requirement"] is None:
            raise RecoveryOracleError(INVALID_IDEMPOTENCY_CONTRACT, "keyed needs key requirement")
        if current["idempotency"] != "keyed" and current["key_requirement"] is not None:
            raise RecoveryOracleError(INVALID_IDEMPOTENCY_CONTRACT, "non-keyed has key requirement")
        if current["unknown_effect_handling"] == "reconciliation_required" \
                and current["reconciliation"] == "none":
            raise RecoveryOracleError(INVALID_IDEMPOTENCY_CONTRACT, "reconciliation handling without requirement")
        comp = current["compensation"]
        if "required" in comp:
            op = comp["required"]
            if (op["catalog"], op["name"]) not in {(EFFECT_CATALOG, x) for x in EFFECT_OPS}:
                raise RecoveryOracleError(UNKNOWN_RECOVERY_OPERATION, "compensation operation unknown")
        clauses.append({**current, "effect": effect})
    clauses.sort(key=lambda x: atom_key(x["effect"]))
    return {"replay_semantics": value["replay_semantics"],
            "effect_recovery_clauses": clauses}


def analyze(value, effect_atoms, evidence="unknown"):
    canonical = canonicalize(value)
    if canonical["replay_semantics"] == "recorded":
        return {"outcome": "requires_evidence", "obligations": ["durable_recorded_result"]}
    if canonical["replay_semantics"] == "forbidden":
        return {"outcome": "proven_static_constraint"}
    if effect_atoms == "unknown":
        return {"outcome": "unproven", "reason": "unknown_effect_coverage"}
    clauses = {atom_key(x["effect"]): x for x in canonical["effect_recovery_clauses"]}
    keys = [atom_key(x) for x in effect_atoms]
    if any(key not in keys for key in clauses):
        raise RecoveryOracleError(RECOVERY_EFFECT_NOT_DECLARED, "clause effect absent from epsilon")
    for key in keys:
        if key not in clauses:
            return {"outcome": "unproven", "reason": "missing_effect_recovery_clause"}
    obligations = set()
    for effect in effect_atoms:
        current = clauses[atom_key(effect)]
        if current["idempotency"] == "natural" and any(
                q.get("idempotency") == "none" for q in effect["qualifiers"]):
            raise RecoveryOracleError(INVALID_IDEMPOTENCY_CONTRACT, "effect qualifier conflict")
        if canonical["replay_semantics"] == "reconcile" and current["reconciliation"] == "none":
            obligations.add("reconciliation_evidence_missing")
        if current["key_requirement"] is not None:
            if not all(current["key_requirement"].values()):
                obligations.add("key_stability_evidence_missing")
        if evidence in ("unknown", "still_unknown"):
            if current["unknown_effect_handling"] == "block" and current["idempotency"] == "none":
                raise RecoveryOracleError(UNSAFE_BLIND_RETRY, "none plus unknown")
            if (current["unknown_effect_handling"] == "reconciliation_required"
                    or current["idempotency"] == "reconciled"
                    or current["reconciliation"] == "required"):
                obligations.add("reconciliation_evidence_missing")
            else:
                obligations.add("idempotency_evidence_missing")
        elif evidence == "partially_applied":
            obligations.add("partial_state_resolution_required")
    return {"outcome": "requires_evidence", "obligations": sorted(obligations)} \
        if obligations else {"outcome": "proven_static_constraint"}


def effect_signature(atoms):
    if atoms == "unknown":
        return {"row": {"unknown": None}}
    return {"row": {"closed": [{
        "atom": item,
        "bounds": {"lower": 1, "upper": {"finite": 1}},
        "guard": {"unconditional": None},
    } for item in atoms]}}


def base(kind, case_id, note):
    return {"kind": "agentflow.conformance.recovery-contract.case",
            "case_kind": kind, "corpus_revision": CORPUS_REVISION,
            "case_id": case_id, "note": note,
            "catalogs": fixture_catalogs()}


CASES = []


def write_case(subdir, payload):
    path = CONF / subdir / f"{payload['case_id'].replace('/', '-')}.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, ensure_ascii=False,
                               sort_keys=True) + "\n", encoding="utf-8",
                    newline="\n")
    CASES.append(payload["case_id"])


def golden(case_id, note, value, atoms, evidence="unknown"):
    canonical = canonicalize(value)
    payload = {**base("golden", case_id, note), "contract": value,
               "effect_signature": effect_signature(atoms),
               "evidence": evidence,
               "canonical_ace1_utf8": canonical_ace1(canonical),
               "expected": analyze(value, atoms, evidence)}
    write_case("golden", payload)


def negative(case_id, note, value, atoms, expected_kind, evidence="unknown"):
    try:
        result = analyze(value, atoms, evidence)
        if expected_kind == "__requires__" and result["outcome"] == "requires_evidence":
            write_case("negative", {**base("negative", case_id, note), "contract": value,
                                     "effect_signature": effect_signature(atoms),
                                     "expected_kind": expected_kind,
                                     "expected": result})
            return
        if expected_kind == RECOVERY_CONTRACT_UNPROVEN and result["outcome"] == "unproven":
            write_case("negative", {**base("negative", case_id, note), "contract": value,
                                     "effect_signature": effect_signature(atoms),
                                     "expected_kind": expected_kind,
                                     "expected": result})
            return
    except RecoveryOracleError as error:
        if error.kind != expected_kind:
            raise AssertionError(f"{case_id}: {error.kind} != {expected_kind}")
    else:
        raise AssertionError(f"{case_id}: expected {expected_kind}")
    write_case("negative", {**base("negative", case_id, note), "contract": value,
                             "effect_signature": effect_signature(atoms),
                             "expected_kind": expected_kind})


def analysis_case(subdir, case_id, note, value, atoms, expected, evidence="unknown"):
    actual = analyze(value, atoms, evidence)
    if actual != expected:
        raise AssertionError(f"{case_id}: {actual} != {expected}")
    write_case(subdir, {**base(subdir, case_id, note), "contract": value,
                        "effect_signature": effect_signature(atoms),
                        "evidence": evidence, "expected": expected})


def compatibility(case_id, note, new, old, expected):
    new_c, old_c = canonicalize(new), canonicalize(old)
    if new_c == old_c:
        actual = {"outcome": "proven"}
    elif len(new_c["effect_recovery_clauses"]) < len(old_c["effect_recovery_clauses"]):
        actual = {"outcome": "weakened"}
    else:
        actual = {"outcome": "unproven"}
    if actual != expected:
        raise AssertionError(f"{case_id}: {actual} != {expected}")
    write_case("compatibility", {**base("compatibility", case_id, note),
                                  "new_contract": new, "old_contract": old,
                                  "expected": expected})


def equivalence(case_id, note, members):
    frozen = canonical_ace1(canonicalize(members[0]))
    for member in members[1:]:
        if canonical_ace1(canonicalize(member)) != frozen:
            raise AssertionError(case_id)
    write_case("equivalence", {**base("equivalence", case_id, note),
                               "members": members, "canonical_ace1_utf8": frozen})


def main():
    if CONF.exists():
        for path in CONF.rglob("*.json"):
            path.unlink()
    charge = atom("payment.charge", qualifier="natural")
    none_charge = atom("payment.charge", qualifier="none")
    keyed_atom = atom("payment.charge")
    refund = atom("payment.refund")
    assert atom_key(charge) != atom_key(none_charge), \
        "the independent Recovery oracle must distinguish qualifier members"
    try:
        analyze(contract("recompute", [clause(charge, "natural")]),
                [none_charge], "applied")
    except RecoveryOracleError as error:
        assert error.kind == RECOVERY_EFFECT_NOT_DECLARED
    else:
        raise AssertionError("a mismatched qualifier must not be proven")
    # Golden --------------------------------------------------------------
    golden("recovery/golden/pure-recorded", "recorded replay needs durable evidence",
           contract("recorded", []), [], "unknown")
    golden("recovery/golden/recompute", "pure recompute without effects is statically safe",
           contract("recompute", []), [], "unknown")
    golden("recovery/golden/forbidden-replay", "forbidden replay cannot execute effects",
           contract("forbidden", [clause(charge, "none")]), [charge])
    golden("recovery/golden/natural-idempotency", "natural declaration remains a declaration",
           contract("recompute", [clause(charge, "natural")]), [charge], "not_applied")
    golden("recovery/golden/recovery-exact-qualifier-match",
           "the complete qualifier member matches exactly and Applied evidence "
           "closes the static obligation",
           contract("recompute", [clause(charge, "natural")]),
           [charge], "applied")
    golden("recovery/golden/recovery-natural-none-distinct-clauses",
           "natural and none clauses for the same operation and selector are "
           "distinct complete identities, not duplicate clauses",
           contract("recompute", [clause(charge, "natural"),
                                  clause(none_charge, "none")]),
           [charge, none_charge], "applied")
    scoped_charge = atom("payment.charge", qualifier="natural",
                         scope=(("merchant", "acme"),))
    golden("recovery/golden/recovery-clause-complete-atom-identity",
           "exact Recovery matching includes operation, selector port, scope "
           "bindings and full qualifier member",
           contract("recompute", [clause(scoped_charge, "natural")]),
           [scoped_charge], "applied")
    golden("recovery/golden/keyed-idempotency", "keyed replay records key obligations",
           contract("recompute", [clause(keyed_atom, "keyed", key_req())]), [keyed_atom], "not_applied")
    golden("recovery/golden/reconciled-idempotency", "reconciled mode needs authoritative evidence",
           contract("reconcile", [clause(keyed_atom, "reconciled", reconciliation="required", unknown="reconciliation_required")]), [keyed_atom], "not_applied")
    golden("recovery/golden/effect-specific-clause", "clauses use exact effect identity",
           contract("recompute", [clause(charge, "natural"), clause(refund, "none", reconciliation="required", unknown="reconciliation_required")]), [charge, refund], "not_applied")
    golden("recovery/golden/compensation-requirement", "compensation is a new effect operation",
           contract("recompute", [clause(charge, "natural", compensation="payment.refund")]), [charge], "not_applied")
    # Negative ------------------------------------------------------------
    negative("recovery/negative/duplicate-effect-clause", "duplicate exact clauses reject",
             contract("recompute", [clause(charge), clause(charge)]), [charge], DUPLICATE_EFFECT_RECOVERY_CLAUSE)
    negative("recovery/negative/invalid-replay", "invalid replay enum is rejected by parsing", contract("invalid", []), [], "invalid_replay_semantics")
    negative("recovery/negative/unknown-recovery-operation", "unknown compensation operation fails closed",
             contract("recompute", [clause(charge, compensation="provider.unknown")]), [charge], UNKNOWN_RECOVERY_OPERATION)
    negative("recovery/negative/undeclared-effect-identity", "clause absent from epsilon is rejected",
             contract("recompute", [clause(refund)]), [charge], RECOVERY_EFFECT_NOT_DECLARED)
    negative("recovery/negative/recovery-natural-vs-none-mismatch",
             "a natural Recovery clause cannot match an otherwise-identical "
             "none EffectAtom; a positive static proof is forbidden",
             contract("recompute", [clause(charge, "natural")]),
             [none_charge], RECOVERY_EFFECT_NOT_DECLARED, "applied")
    negative("recovery/negative/recovery-none-vs-natural-mismatch",
             "a none Recovery clause cannot match an otherwise-identical "
             "natural EffectAtom in the reverse direction",
             contract("recompute", [clause(none_charge, "none")]),
             [charge], RECOVERY_EFFECT_NOT_DECLARED, "applied")
    negative("recovery/negative/recovery-different-resource-selector",
             "matching operation and qualifier do not erase the requirement "
             "port from EffectAtom identity",
             contract("recompute", [clause(charge, "natural")]),
             [atom("payment.charge", port=2, qualifier="natural")],
             RECOVERY_EFFECT_NOT_DECLARED, "applied")
    negative("recovery/negative/recovery-different-scope-binding",
             "same operation, selector port and qualifier with another exact "
             "scope binding is not a Recovery match",
             contract("recompute", [clause(scoped_charge, "natural")]),
             [atom("payment.charge", qualifier="natural",
                   scope=(("merchant", "other"),))],
             RECOVERY_EFFECT_NOT_DECLARED, "applied")
    negative("recovery/negative/recovery-proof-no-false-proven",
             "end-to-end public proof must reject mismatched qualifier "
             "identity even when the Effect supplies Applied evidence",
             contract("recompute", [clause(charge, "natural")]),
             [none_charge], RECOVERY_EFFECT_NOT_DECLARED, "applied")
    negative("recovery/negative/unknown-effect-coverage", "unknown effect row has no guessed recovery coverage",
             contract("recompute", []), "unknown", RECOVERY_CONTRACT_UNPROVEN)
    negative("recovery/negative/none-unknown-blind-retry", "NONE plus UNKNOWN cannot blind retry",
             contract("recompute", [clause(keyed_atom, "none")]), [keyed_atom], UNSAFE_BLIND_RETRY)
    negative("recovery/negative/keyed-without-evidence", "KEYED unknown state requires evidence",
             contract("recompute", [clause(keyed_atom, "keyed", key_req())]), [keyed_atom], "__requires__")
    negative("recovery/negative/reconciled-without-result", "RECONCILED without result is not safe yet",
             contract("reconcile", [clause(keyed_atom, "reconciled", reconciliation="required", unknown="reconciliation_required")]), [keyed_atom], "__requires__")
    negative("recovery/negative/partially-applied", "partial state requires resolution",
             contract("recompute", [clause(keyed_atom, "keyed", key_req())]), [keyed_atom], "__requires__", "partially_applied")
    negative("recovery/negative/cancelled-not-not-applied", "cancellation is not evidence of no effect",
             contract("recompute", [clause(keyed_atom, "none")]), [keyed_atom], UNSAFE_BLIND_RETRY)
    analysis_case("compensation", "recovery/compensation/compensation-not-rollback", "compensation does not erase forward effect",
                  contract("recompute", [clause(charge, "natural", compensation="payment.refund")]), [charge], {"outcome": "proven_static_constraint"}, "applied")
    # Replay / reconciliation / compensation -----------------------------
    analysis_case("replay", "recovery/replay/recorded-evidence", "recorded mode requires evidence",
                  contract("recorded", []), [], {"outcome": "requires_evidence", "obligations": ["durable_recorded_result"]})
    analysis_case("replay", "recovery/replay/historical-no-dispatch", "historical replay has no effect dispatch",
                  contract("forbidden", [clause(charge)]), [charge], {"outcome": "proven_static_constraint"})
    analysis_case("replay", "recovery/replay/fork-fresh-admission", "fork is outside this proof and remains static only",
                  contract("recompute", []), [], {"outcome": "proven_static_constraint"})
    analysis_case("reconciliation", "recovery/reconciliation/unknown-blocked", "unknown remains blocked",
                  contract("recompute", [clause(keyed_atom, "reconciled", reconciliation="required", unknown="reconciliation_required")]), [keyed_atom], {"outcome": "requires_evidence", "obligations": ["reconciliation_evidence_missing"]})
    analysis_case("reconciliation", "recovery/reconciliation/still-unknown-blocked", "still unknown remains blocked",
                  contract("recompute", [clause(keyed_atom, "reconciled", reconciliation="required", unknown="reconciliation_required")]), [keyed_atom], {"outcome": "requires_evidence", "obligations": ["reconciliation_evidence_missing"]}, "still_unknown")
    analysis_case("reconciliation", "recovery/reconciliation/not-applied-evidence", "not applied is only a necessary static fact",
                  contract("recompute", [clause(keyed_atom, "keyed", key_req())]), [keyed_atom], {"outcome": "proven_static_constraint"}, "not_applied")
    analysis_case("reconciliation", "recovery/reconciliation/partially-applied", "partial state requires resolution",
                  contract("recompute", [clause(keyed_atom, "keyed", key_req())]), [keyed_atom], {"outcome": "requires_evidence", "obligations": ["partial_state_resolution_required"]}, "partially_applied")
    analysis_case("compensation", "recovery/compensation/forward-history-preserved", "forward history remains a separate fact",
                  contract("recompute", [clause(charge, "natural", compensation="payment.refund")]), [charge], {"outcome": "proven_static_constraint"}, "not_applied")
    analysis_case("compensation", "recovery/compensation/recovery-usage-not-refund", "compensation does not refund Usage",
                  contract("recompute", [clause(charge, "natural", compensation="payment.refund")]), [charge], {"outcome": "proven_static_constraint"}, "not_applied")
    # Compatibility -------------------------------------------------------
    compatibility("recovery/compatibility/identical", "identical declarations are compatible",
                  contract("recompute", [clause(charge, "natural")]),
                  contract("recompute", [clause(charge, "natural")]), {"outcome": "proven"})
    compatibility("recovery/compatibility/clause-removed", "removing a recovery clause weakens the contract",
                  contract("recompute", []), contract("recompute", [clause(charge)]), {"outcome": "weakened"})
    compatibility("recovery/compatibility/keyed-not-natural", "keyed is not ordered above natural by enum spelling",
                  contract("recompute", [clause(keyed_atom, "keyed", key_req())]),
                  contract("recompute", [clause(keyed_atom, "natural")]), {"outcome": "unproven"})
    compatibility("recovery/compatibility/reconciliation-capability-removed", "removing reconciliation is not compatible",
                  contract("recompute", [clause(keyed_atom, "reconciled", reconciliation="none")]),
                  contract("recompute", [clause(keyed_atom, "reconciled", reconciliation="required", unknown="reconciliation_required")]), {"outcome": "unproven"})
    compatibility("recovery/compatibility/unknown-recovery-unproven", "unknown recovery claims are not safe recovery",
                  contract("recompute", [clause(charge, "none")]),
                  contract("recompute", [clause(charge, "natural")]), {"outcome": "unproven"})
    # Equivalence ---------------------------------------------------------
    equivalence("recovery/equivalence/effect-clause-order", "clause declaration order is not identity",
                [contract("recompute", [clause(charge), clause(refund)]),
                 contract("recompute", [clause(refund), clause(charge)])])
    equivalence("recovery/equivalence/qualifier-member-order",
                "same-operation clauses with different qualifier members "
                "remain distinct and declaration order is canonicalized",
                [contract("recompute", [clause(charge, "natural"),
                                         clause(none_charge, "none")]),
                 contract("recompute", [clause(none_charge, "none"),
                                         clause(charge, "natural")])])
    print(f"frozen {len(CASES)} cases")


if __name__ == "__main__":
    main()
