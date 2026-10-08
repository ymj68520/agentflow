#!/usr/bin/env python3
"""Independent Phase 4B.1 ComponentContract and relation-catalog oracle.

This generator uses only Python's standard library plus the repository's
independent AFIDL1 Python implementation. It never invokes Rust. Expected
ACE1 bytes, TypeSchemaDigests, ComponentContract HashV1 values, relation
content checksums and the fail-closed decision vectors are computed here.
"""

import copy
import hashlib
import importlib.util
import json
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CONF = ROOT / "spec" / "conformance" / "component-contract"

spec = importlib.util.spec_from_file_location("afidl_oracle", ROOT / "tools" / "gen-afidl-corpus.py")
afidl = importlib.util.module_from_spec(spec)
spec.loader.exec_module(afidl)

MODULES = [
    "spec/idl/component-contract.afidl",
    "spec/idl/effect-resource-compatibility.afidl",
    "spec/idl/recovery-usage-coverage.afidl",
    "spec/idl/flow-execution-relation.afidl",
]


def write(rel, value):
    path = CONF / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2) + "\n"
    path.write_text(payload, encoding="utf-8", newline="\n")
    print(f"wrote {path.relative_to(ROOT)}")


def ace1(value):
    """Small independent ACE1 encoder for values used by these fixtures."""
    if value is None:
        return "null"
    if value is True:
        return "true"
    if value is False:
        return "false"
    if isinstance(value, int):
        return json.dumps(str(value), ensure_ascii=False, separators=(",", ":"))
    if isinstance(value, str):
        normalized = unicodedata.normalize("NFC", value)
        return json.dumps(normalized, ensure_ascii=False, separators=(",", ":"))
    if isinstance(value, list):
        return "[" + ",".join(ace1(item) for item in value) + "]"
    if isinstance(value, dict):
        keys = sorted(value, key=lambda key: key.encode("utf-8"))
        return "{" + ",".join(ace1(key) + ":" + ace1(value[key]) for key in keys) + "}"
    raise TypeError(f"unsupported ACE1 fixture value: {type(value)!r}")


def atom_key(atom):
    selector = atom["resource_selector"]
    scope = tuple(sorted((item["key"], item["value"]) for item in selector["scope_bindings"]))
    qualifier_order = {"natural": 1, "none": 2}
    qualifiers = tuple(sorted(("idempotency", qualifier_order[qualifier["idempotency"]]) for qualifier in atom["qualifiers"]))
    operation = atom["operation"]
    return (operation["catalog"], operation["name"], selector["requirement_port_no"], scope, qualifiers)


def action_key(action):
    return (atom_key(action["forward_effect"]),
            {"reconciliation": 1, "compensation": 2}[action["action_kind"]],
            atom_key(action["action_effect"]))


def type_ref_key(ref):
    digest = ref["schema_digest"]
    return (ref["specification_id"], ref["type_name"], digest["algorithm"], digest["domain"], digest["hex"])


def usage_limit_key(limit):
    return ({"forward": 1, "recovery": 2}[limit["phase"]], limit["meter"]["catalog"], limit["meter"]["name"])


def source_key(source):
    category = source["category"]
    name, value = next(iter(category.items()))
    return ({"workflow_input": 1, "resource_requirement": 2, "unknown_external": 3}[name], value if name != "unknown_external" else 0)


def sink_key(sink):
    name, value = next(iter(sink.items()))
    if name == "workflow_output":
        return (1, value)
    return (2, value["catalog"], value["name"])


def normalize_atom(atom):
    atom["resource_selector"]["scope_bindings"].sort(key=lambda item: (item["key"], item["value"]))
    atom["qualifiers"].sort(key=lambda item: (1, next(iter(item.values()))))
    return atom


def canonical_contract(source):
    body = copy.deepcopy(source)
    body["failures"]["failures"].sort(key=lambda item: type_ref_key(item["payload_type"]))
    row = body["effects"]["row"]
    row_case, row_payload = next(iter(row.items()))
    if row_case == "closed":
        for entry in row_payload:
            normalize_atom(entry["atom"])
        row_payload.sort(key=lambda entry: atom_key(entry["atom"]))
    elif row_case == "open":
        for entry in row_payload["known"]:
            normalize_atom(entry["atom"])
        row_payload["known"].sort(key=lambda entry: atom_key(entry["atom"]))
        for residual in row_payload["residual"]:
            residual["selector"]["scope_bindings"].sort(key=lambda item: (item["key"], item["value"]))
        row_payload["residual"].sort(key=lambda item: (item["catalog"], item["selector"]["requirement_port_no"], tuple((b["key"], b["value"]) for b in item["selector"]["scope_bindings"])))
    body["resources"]["requirements"].sort(key=lambda item: item["port_no"])
    for requirement in body["resources"]["requirements"]:
        requirement["required_attributes"].sort(key=lambda item: (item["key"], item["value"]))
    flow = body["information_flow"]
    flow["confidentiality_restrictions"].sort(key=lambda item: (item["catalog"], item["name"]))
    flow["summary"]["sources"].sort(key=source_key)
    flow["summary"]["taints"].sort(key=lambda item: (item["catalog"], item["name"]))
    flow["flow_constraints"].sort(key=lambda item: ({"data_egress": 1, "decision_influence": 2}[item["kind"]], sink_key(item["sink"])))
    body["usage"]["limits"].sort(key=usage_limit_key)
    for clause in body["recovery"]["effect_recovery_clauses"]:
        normalize_atom(clause["effect"])
    body["recovery"]["effect_recovery_clauses"].sort(key=lambda item: atom_key(item["effect"]))
    execution = body["execution"]
    execution["capabilities"].sort(key=lambda item: (item["capability"]["catalog"], item["capability"]["name"], item["exact_version"]))
    for requirement in execution["capabilities"]:
        requirement["required_attributes"].sort(key=lambda item: (item["key"], item["value"]))
    execution["quantities"].sort(key=lambda item: ({"cpu_millicores": 1, "memory_bytes": 2, "gpu_count": 3}[item["kind"]]))
    if "some" in execution["region_residency"]:
        residency = execution["region_residency"]["some"]
        residency["execution_regions"].sort(key=lambda item: (item["catalog"], item["name"]))
        residency["residency_regions"].sort(key=lambda item: (item["catalog"], item["name"]))
    if "some" in execution["runtime_profile"]:
        execution["runtime_profile"]["some"]["required_features"].sort()
    execution["implementation_abis"].sort(key=lambda item: type_ref_key(item["schema"]))
    execution["soft_preferences"].sort(key=lambda item: {"artifact_locality": 1, "warm_model_cache": 2, "lower_load": 3, "lower_cost": 4}[item])
    body["inputs"].sort(key=lambda port: port["port_no"])
    body["outputs"].sort(key=lambda port: port["port_no"])
    body["effect_flow_partitions"].sort(key=lambda part: (
        part["operation"]["catalog"], part["operation"]["name"],
        {"data_egress": 1, "decision_influence": 2}[part["kind"]]))
    for partition in body["effect_flow_partitions"]:
        for atom in partition["allowed_effects"]:
            normalize_atom(atom)
        partition["allowed_effects"].sort(key=atom_key)
    for binding in body["recovery_action_bindings"]:
        for atom in (binding["identity"]["forward_effect"], binding["identity"]["action_effect"]):
            normalize_atom(atom)
    body["recovery_action_bindings"].sort(key=lambda item: action_key(item["identity"]))
    for binding in body["recovery_usage_bindings"]:
        for atom in (binding["action"]["forward_effect"], binding["action"]["action_effect"]):
            normalize_atom(atom)
        binding["per_attempt_usage"]["limits"].sort(key=lambda limit: (
            {"forward": 1, "recovery": 2}[limit["phase"]],
            limit["meter"]["catalog"], limit["meter"]["name"]))
    body["recovery_usage_bindings"].sort(key=lambda item: action_key(item["action"]))
    return body


def digest(domain, payload):
    return hashlib.sha256(b"AF\x00" + domain.encode("ascii") + b"\x00" + payload).hexdigest()


def atom(operation="agentflow.effects@1", name="read", port=1, qualifier="natural", scope=None):
    return {
        "operation": {"catalog": operation, "name": name},
        "resource_selector": {"requirement_port_no": port, "scope_bindings": scope or []},
        "qualifiers": [] if qualifier is None else [{"idempotency": qualifier}],
    }


def empty_contract(identity=None):
    return {
        "component_identity": {"catalog": "agentflow.components@1", "name": identity or "acme.transform"},
        "inputs": [],
        "outputs": [],
        "failures": {"failures": []},
        "effects": {"row": {"closed": []}},
        "resources": {"requirements": []},
        "information_flow": {
            "confidentiality_restrictions": [],
            "summary": {"sources": [], "taints": [], "instruction_authority": "none", "influence_unknown": False},
            "flow_constraints": [],
        },
        "usage": {"limits": []},
        "recovery": {"replay_semantics": "recompute", "effect_recovery_clauses": []},
        "execution": {
            "capabilities": [], "quantities": [], "sandbox": {"none": None}, "trust_domain": {"none": None},
            "region_residency": {"none": None}, "runtime_profile": {"none": None},
            "implementation_abis": [], "soft_preferences": [],
        },
        "effect_flow_partitions": [],
        "recovery_action_bindings": [],
        "recovery_usage_bindings": [],
    }


def contract_expectation(case_id, body, metadata=None):
    canonical = canonical_contract(body)
    payload = ace1(canonical).encode("utf-8")
    record = {
        "case_id": case_id,
        "canonical_component_contract": canonical,
        "ace1_utf8": payload.decode("utf-8"),
        "ace1_hex": payload.hex(),
        "component_contract_digest": digest("component-contract", payload),
    }
    if metadata is not None:
        record["non_semantic_metadata"] = metadata
    return record


def schema_golden():
    cases = []
    all_sources = []
    for relative in MODULES:
        source = (ROOT / relative).read_text(encoding="utf-8")
        all_sources.append((relative, source))
        module, _imports, types = afidl.compile_module_from_source(source)
        cases.append({
            "case_id": Path(relative).stem,
            "file": relative,
            "specification_id": f"{module['spec']['namespace']}@{module['spec']['major']}",
            "types": afidl.type_entries(types),
        })
    universe_sources = [
        (path.relative_to(ROOT).as_posix(), path.read_text(encoding="utf-8"))
        for path in sorted((ROOT / "spec" / "idl").rglob("*.afidl"))
    ]
    afidl.compile_module_set(universe_sources)
    write("schema-golden.json", {
        "corpus_revision": 1,
        "kind": "agentflow.conformance.component-contract.schema-golden",
        "note": "Independent Python AFIDL1 type schemas, canonical ACE1 TypeSchema bytes and frozen TypeSchemaDigest expectations for the four additive Phase 4B.1 modules.",
        "cases": cases,
    })


def component_corpora():
    base = empty_contract()
    base_atom = atom()
    port = {"specification_id": "agentflow.semantic.types@1", "type_name": "Text", "schema_digest": {
        "algorithm": "sha256", "domain": "type-schema", "hex": "11" * 32}}
    typed = empty_contract()
    typed["inputs"] = [{"port_no": 2, "type": port}, {"port_no": 1, "type": port}]
    typed["outputs"] = [{"port_no": 1, "type": port}]
    typed["information_flow"]["summary"]["sources"] = [{"category": {"workflow_input": 1}}]

    with_effect = empty_contract()
    with_effect["effects"]["row"] = {"closed": [{"atom": base_atom, "bounds": {
        "lower": 1, "upper": {"finite": 2}}, "guard": {"unconditional": None}}]}
    with_effect["resources"]["requirements"] = [{"port_no": 1, "resource_type": {"schema": port}, "required_attributes": []}]
    with_effect["information_flow"]["flow_constraints"] = [{"kind": "data_egress", "sink": {"external_effect": base_atom["operation"]}}]
    with_effect["effect_flow_partitions"] = [{"operation": base_atom["operation"], "kind": "data_egress", "allowed_effects": [base_atom]}]

    meter = {"catalog": "agentflow.meters@1", "name": "api_calls"}
    unit = {"catalog": "agentflow.units@1", "name": "calls"}
    reverse_atom = atom(name="reverse", qualifier=None)
    recovery_clause = {
        "effect": base_atom,
        "idempotency": "natural",
        "key_requirement": {"none": None},
        "reconciliation": "none",
        "compensation": {"required": reverse_atom["operation"]},
        "unknown_effect_handling": "block",
    }
    recovery_identity = {
        "forward_effect": base_atom,
        "action_kind": "compensation",
        "action_effect": reverse_atom,
    }
    recovery_action = empty_contract()
    recovery_action["effects"]["row"] = {"closed": [
        {"atom": base_atom, "bounds": {"lower": 1, "upper": {"finite": 2}}, "guard": {"unconditional": None}},
        {"atom": reverse_atom, "bounds": {"lower": 0, "upper": {"finite": 2}}, "guard": {"unconditional": None}},
    ]}
    recovery_action["resources"]["requirements"] = [{"port_no": 1, "resource_type": {"schema": port}, "required_attributes": []}]
    recovery_action["recovery"] = {"replay_semantics": "recompute", "effect_recovery_clauses": [recovery_clause]}
    recovery_action["recovery_action_bindings"] = [{"identity": recovery_identity}]

    recovery_usage = copy.deepcopy(recovery_action)
    recovery_usage["recovery_usage_bindings"] = [{
        "action": recovery_identity,
        "activations_per_forward": {"lower": 1, "upper": {"finite": 1}},
        "attempts_per_activation": {"lower": 1, "upper": {"finite": 2}},
        "per_attempt_usage": {"limits": [{"meter": meter, "phase": "recovery", "upper": {"finite": 3}}]},
    }]
    recovery_usage["usage"]["limits"] = [
        {"meter": meter, "phase": "forward", "upper": {"finite": 2}},
        {"meter": meter, "phase": "recovery", "upper": {"finite": 12}},
    ]

    execution = empty_contract()
    execution["execution"] = {
        "capabilities": [], "quantities": [{"kind": "cpu_millicores", "minimum": 1000}],
        "sandbox": {"some": {"minimum": "strong"}},
        "trust_domain": {"some": {"catalog": "agentflow.trust@1", "name": "prod"}},
        "region_residency": {"some": {
            "execution_regions": [{"catalog": "agentflow.regions@1", "name": "sg"}],
            "residency_regions": [{"catalog": "agentflow.regions@1", "name": "sg"}],
        }},
        "runtime_profile": {"none": None}, "implementation_abis": [], "soft_preferences": [],
    }

    failures = empty_contract()
    failures["failures"]["failures"] = [{"payload_type": port}]

    all_dimensions = copy.deepcopy(recovery_usage)
    all_dimensions["inputs"] = [{"port_no": 1, "type": port}]
    all_dimensions["outputs"] = [{"port_no": 1, "type": port}]
    all_dimensions["failures"]["failures"] = [{"payload_type": port}]
    all_dimensions["information_flow"] = {
        "confidentiality_restrictions": [{"catalog": "agentflow.labels@1", "name": "pii"}],
        "summary": {
            "sources": [{"category": {"workflow_input": 1}}, {"category": {"resource_requirement": 1}}],
            "taints": [{"catalog": "agentflow.labels@1", "name": "user_input"}],
            "instruction_authority": "user", "influence_unknown": False,
        },
        "flow_constraints": [
            {"kind": "data_egress", "sink": {"workflow_output": 1}},
            {"kind": "decision_influence", "sink": {"external_effect": base_atom["operation"]}},
        ],
    }
    all_dimensions["execution"] = execution["execution"]
    all_dimensions["effect_flow_partitions"] = [{"operation": base_atom["operation"], "kind": "decision_influence", "allowed_effects": [base_atom]}]

    contracts = [
        contract_expectation("contract/golden/minimal-closed", base),
        contract_expectation("contract/golden/typed-io", typed),
        contract_expectation("contract/golden/declared-effects-resource-partition", with_effect),
        contract_expectation("contract/golden/declared-failure-row", failures),
        contract_expectation("contract/golden/recovery-action", recovery_action),
        contract_expectation("contract/golden/recovery-usage", recovery_usage),
        contract_expectation("contract/golden/execution-constraints", execution),
        contract_expectation("contract/golden/all-dimensions", all_dimensions),
        contract_expectation("contract/golden/publication-and-metadata-excluded", base, {
            "contract_version": {"major": 9, "minor": 2, "patch": 1},
            "implementation_digest": "aa" * 32,
            "price_table_snapshot": "prices-v44",
            "diagnostic_name": "friendly label",
        }),
    ]
    write("golden/contracts.json", {"corpus_revision": 1, "kind": "agentflow.conformance.component-contract.golden", "cases": contracts})

    # Mutations are intentionally separated by the first sound static verdict.
    negative_ids = [
        ("unknown-input-type-schema", "violation"), ("wrong-output-type-digest", "violation"),
        ("duplicate-input-port", "violation"), ("duplicate-output-port", "violation"),
        ("zero-port-number", "violation"), ("effect-selector-missing-resource-port", "violation"),
        ("effect-resource-relation-missing", "unproven"), ("effect-resource-explicit-denial", "violation"),
        ("partition-exceeds-legacy-flow-ceiling", "violation"), ("partition-effect-qualifier-mismatch", "violation"),
        ("partition-misses-required-exact-atom", "unproven"), ("flow-sink-undeclared-output", "violation"),
        ("recovery-action-qualifier-mismatch", "violation"), ("recovery-action-undeclared-effect", "violation"),
        ("recovery-action-cycle", "violation"), ("recovery-action-missing-meter-profile", "violation"),
        ("recovery-missing-coverage-catalog", "unproven"), ("recovery-aggregate-exceeds-u", "violation"),
        ("flow-execution-residency-conflict", "violation"), ("changed-catalog-content-stale-proof", "unproven"),
        ("unknown-effect-not-covered", "unproven"), ("missing-usage-not-zero", "unproven"),
        ("unknown-external-provenance", "requires_evidence"), ("open-epsilon-partition-coverage", "unproven"),
    ]
    negative_cases = []
    for case, verdict in negative_ids:
        vector = {"case_id": f"contract/negative/{case}", "input_mutation": case, "expected_verdict": verdict}
        invalid_contract = copy.deepcopy(base)
        if case == "unknown-input-type-schema":
            bad_ref = copy.deepcopy(port)
            bad_ref["specification_id"] = "agentflow.unknown.types@1"
            invalid_contract["inputs"] = [{"port_no": 1, "type": bad_ref}]
        elif case == "wrong-output-type-digest":
            bad_ref = copy.deepcopy(port)
            bad_ref["schema_digest"]["hex"] = "ff" * 32
            invalid_contract["outputs"] = [{"port_no": 1, "type": bad_ref}]
        elif case == "duplicate-input-port":
            invalid_contract["inputs"] = [{"port_no": 1, "type": port}, {"port_no": 1, "type": port}]
        elif case == "duplicate-output-port":
            invalid_contract["outputs"] = [{"port_no": 1, "type": port}, {"port_no": 1, "type": port}]
        elif case == "zero-port-number":
            invalid_contract["inputs"] = [{"port_no": 0, "type": port}]
        else:
            invalid_contract = None
        if invalid_contract is not None:
            vector["wire_input"] = invalid_contract
        negative_cases.append(vector)
    write("negative/cases.json", {"corpus_revision": 1, "kind": "agentflow.conformance.component-contract.negative", "cases": negative_cases})

    reordered = empty_contract()
    reordered["inputs"] = [{"port_no": 2, "type": port}, {"port_no": 1, "type": port}]
    canonical = canonical_contract(reordered)
    expected = contract_expectation("contract/canonical/port-source-order", reordered)
    write("canonical/cases.json", {"corpus_revision": 1, "kind": "agentflow.conformance.component-contract.canonical", "cases": [
        expected,
        {"case_id": "contract/canonical/field-mutation-changes-digest", "base_digest": contract_expectation("base", base)["component_contract_digest"], "mutated_digest": contract_expectation("mutated", empty_contract("acme.other"))["component_contract_digest"]},
        {"case_id": "contract/canonical/full-qualifier-identity", "natural_atom_key": atom_key(atom(qualifier="natural")), "none_atom_key": atom_key(atom(qualifier="none")), "distinct": atom_key(atom(qualifier="natural")) != atom_key(atom(qualifier="none"))},
        {"case_id": "contract/canonical/price-implementation-publication-excluded", "base_digest": contract_expectation("base", base)["component_contract_digest"], "modified_external_metadata_digest": contract_expectation("metadata", base)["component_contract_digest"]},
    ]})

    cross = [
        "flow-input-to-exact-I-port", "flow-resource-to-exact-R-port", "flow-output-to-exact-O-port",
        "effect-selector-to-exact-R-port", "effect-resource-complete-operation-coverage",
        "operation-level-F-ceiling-refined-by-full-atom-partition", "partition-is-subset-not-expansion",
        "K-exact-forward-effect-action-effect", "K-action-dependency-acyclic", "K-to-U-exhaustive-meter-coverage",
        "checked-forward-activation-attempt-meter-product", "F-X-exact-region-pairs", "F-X-exact-taint-trust-sandbox",
        "one-unproven-dimension-blocks-overall-proven",
    ]
    write("cross-dimension/cases.json", {"corpus_revision": 1, "kind": "agentflow.conformance.component-contract.cross-dimension", "cases": [
        {"case_id": f"contract/cross/{case}", "rule_id": "AF-CONTRACT-0017", "expected": "proven" if case == "one-unproven-dimension-blocks-overall-proven" else "requires_exact_context"}
        for case in cross
    ]})

    compat = [
        ("exact-same-contract", "proven"), ("same-semantics-distinct-publication-version", "proven"),
        ("same-lineage-semantic-change", "unproven"), ("implementation-change-contract-unchanged", "proven"),
        ("input-type-pin-mismatch", "unproven"), ("output-type-pin-mismatch", "unproven"),
        ("failure-expansion", "unproven"), ("effect-expansion", "unproven"),
        ("resource-strengthening", "unproven"), ("flow-relaxation", "unproven"),
        ("usage-ceiling-increase", "unproven"), ("recovery-guarantee-weakening", "unproven"),
        ("sandbox-relaxation-feasibility-security-split", "unproven"),
        ("one-unproven-dimension-blocks-proven", "unproven"),
        ("new-required-input", "violation"), ("removed-required-output", "violation"),
    ]
    write("compatibility/cases.json", {"corpus_revision": 1, "kind": "agentflow.conformance.component-contract.compatibility", "cases": [
        {"case_id": f"contract/compatibility/{case}", "expected_verdict": verdict}
        for case, verdict in compat
    ]})

    equivalence = [
        {"case_id": "contract/equivalence/source-port-order", "equivalent": True},
        {"case_id": "contract/equivalence/source-declaration-order", "equivalent": True},
        {"case_id": "contract/equivalence/contract-version", "equivalent": True},
        {"case_id": "contract/equivalence/diagnostic-metadata", "equivalent": True},
        {"case_id": "contract/equivalence/price-snapshot", "equivalent": True},
        {"case_id": "contract/equivalence/implementation-digest", "equivalent": True},
        {"case_id": "contract/equivalence/component-identity", "equivalent": False},
        {"case_id": "contract/equivalence/full-qualifier-member", "equivalent": False},
    ]
    write("equivalence/cases.json", {"corpus_revision": 1, "kind": "agentflow.conformance.component-contract.equivalence", "cases": equivalence})


def canonical_relation(name, value):
    value = copy.deepcopy(value)
    if name == "effect-resource":
        value["complete_operations"].sort(key=lambda op: (op["catalog"], op["name"]))
        for pair in value["supported_pairs"]:
            pair["selector_keys"].sort(key=lambda rule: rule["selector_key"])
            for rule in pair["selector_keys"]:
                choices = rule["values"].get("finite_exact_values")
                if choices is not None:
                    choices.sort()
            for profile in pair["allowed_qualifier_profiles"]:
                profile["qualifiers"].sort(key=lambda qualifier: next(iter(qualifier.values())))
            pair["allowed_qualifier_profiles"].sort(key=lambda profile: tuple(next(iter(q.values())) for q in profile["qualifiers"]))
        value["supported_pairs"].sort(key=lambda pair: (
            pair["operation"]["catalog"], pair["operation"]["name"],
            type_ref_key(pair["resource_type"]["schema"])))
    elif name == "recovery-usage":
        for rule in value["rules"]:
            rule["complete_meters"].sort(key=lambda meter: (meter["catalog"], meter["name"]))
            for atom_value in (rule["action"]["forward_effect"], rule["action"]["action_effect"]):
                normalize_atom(atom_value)
        value["rules"].sort(key=lambda rule: action_key(rule["action"]))
    else:
        value["closed_regions"].sort(key=lambda region: (region["catalog"], region["name"]))
        value["closed_trust_domains"].sort(key=lambda domain: (domain["catalog"], domain["name"]))
        for rule in value["confidentiality_rules"]:
            pairs = rule["region_pairs"].get("finite_exact_pairs")
            if pairs is not None:
                pairs.sort(key=lambda pair: (
                    pair["execution_region"]["catalog"], pair["execution_region"]["name"],
                    pair["residency_region"]["catalog"], pair["residency_region"]["name"]))
        value["confidentiality_rules"].sort(key=lambda rule: (rule["label"]["catalog"], rule["label"]["name"]))
        value["taint_rules"].sort(key=lambda rule: (rule["taint"]["catalog"], rule["taint"]["name"]))
        value["authority_rules"].sort(key=lambda rule: {"none": 1, "external_data": 2, "delegated_agent": 3, "user": 4, "workflow": 5, "platform": 6}[rule["authority"]])
        value["sink_rules"].sort(key=lambda rule: (1 if rule["kind"] == "data_egress" else 2, sink_key(rule["sink"])))
    return value


def relation_corpora():
    schema_ref = {"specification_id": "agentflow.semantic.core@1", "type_name": "TypeSchemaRef", "schema_digest": {
        "algorithm": "sha256", "domain": "type-schema", "hex": "11" * 32}}
    operation = lambda name: {"catalog": "agentflow.effects@1", "name": name}
    regions = [
        {"catalog": "agentflow.regions@1", "name": "us"},
        {"catalog": "agentflow.regions@1", "name": "sg"},
    ]
    trust = [
        {"catalog": "agentflow.trust@1", "name": "worker-b"},
        {"catalog": "agentflow.trust@1", "name": "worker-a"},
    ]
    action = {
        "forward_effect": atom(name="charge"),
        "action_kind": "compensation",
        "action_effect": atom(name="reverse", qualifier=None),
    }
    catalogs = {
        "effect-resource": (
            "agentflow.effect.resource.compatibility@1", "EffectResourceCompatibilityCatalog",
            {"complete_operations": [], "supported_pairs": []},
            {"complete_operations": [operation("read"), operation("write")], "supported_pairs": [{
                "operation": operation("read"), "resource_type": {"schema": schema_ref},
                "selector_keys": [
                    {"selector_key": "zone", "resource_attribute_key": "region", "required": False, "values": {"finite_exact_values": ["us", "sg"]}},
                    {"selector_key": "tenant", "resource_attribute_key": "tenant", "required": True, "values": {"opaque_nfc_exact_string": None}},
                ],
                "allowed_qualifier_profiles": [{"qualifiers": [{"idempotency": "none"}]}],
            }]},
        ),
        "recovery-usage": (
            "agentflow.recovery.usage.coverage@1", "RecoveryUsageCoverageCatalog",
            {"rules": []},
            {"rules": [
                {"action": action, "complete_meters": [
                    {"catalog": "agentflow.meters@1", "name": "z_calls"},
                    {"catalog": "agentflow.meters@1", "name": "a_calls"},
                ]},
                {"action": {**action, "action_kind": "reconciliation"}, "complete_meters": []},
            ]},
        ),
        "flow-execution": (
            "agentflow.flow.execution.relation@1", "InformationFlowExecutionCatalog",
            {"closed_regions": [], "closed_trust_domains": [], "confidentiality_rules": [], "taint_rules": [], "authority_rules": [], "sink_rules": []},
            {"closed_regions": regions, "closed_trust_domains": trust,
             "confidentiality_rules": [{"label": {"catalog": "agentflow.labels@1", "name": "pii"}, "region_pairs": {"finite_exact_pairs": [
                 {"execution_region": regions[0], "residency_region": regions[1]},
                 {"execution_region": regions[1], "residency_region": regions[0]},
             ]}}], "taint_rules": [], "authority_rules": [], "sink_rules": []},
        ),
    }
    for name, (schema_id, root_type, empty, unordered) in catalogs.items():
        nonempty = canonical_relation(name, unordered)
        empty_bytes = ace1(empty).encode("utf-8")
        nonempty_bytes = ace1(nonempty).encode("utf-8")
        write(f"relation-catalogs/{name}/valid/cases.json", {"corpus_revision": 1, "kind": "agentflow.conformance.component-contract.relation-valid", "cases": [
            {"case_id": f"relation/{name}/valid/empty-explicit-snapshot", "schema": schema_id, "root_type": root_type,
             "revision": "fixture-r1", "canonical_value": empty, "ace1_utf8": empty_bytes.decode(), "content_checksum_sha256": hashlib.sha256(empty_bytes).hexdigest()},
            {"case_id": f"relation/{name}/valid/nonempty-exact-snapshot", "schema": schema_id, "root_type": root_type,
             "revision": "fixture-r2", "canonical_value": nonempty, "ace1_utf8": nonempty_bytes.decode(), "content_checksum_sha256": hashlib.sha256(nonempty_bytes).hexdigest()},
        ]})
        invalids = []
        if name == "effect-resource":
            invalids = [
                {"input_value": {"complete_operations": [operation("read")], "supported_pairs": [{"operation": operation("write"), "resource_type": {"schema": schema_ref}, "selector_keys": [], "allowed_qualifier_profiles": []}]}, "expected_error": "pair_operation_outside_complete_set"},
                {"input_value": {"complete_operations": [operation("read"), operation("read")], "supported_pairs": []}, "expected_error": "duplicate_complete_operation"},
            ]
        elif name == "recovery-usage":
            invalids = [
                {"input_value": {"rules": [{"action": action, "complete_meters": []}, {"action": action, "complete_meters": []}]}, "expected_error": "duplicate_exact_action_rule"},
                {"input_value": {"rules": [{"action": action, "complete_meters": [{"catalog": "agentflow.meters@1", "name": "x"}, {"catalog": "agentflow.meters@1", "name": "x"}]}]}, "expected_error": "duplicate_meter_coordinate"},
            ]
        else:
            invalids = [
                {"input_value": {"closed_regions": [regions[0], regions[0]], "closed_trust_domains": [], "confidentiality_rules": [], "taint_rules": [], "authority_rules": [], "sink_rules": []}, "expected_error": "duplicate_closed_region"},
                {"input_value": {"closed_regions": [], "closed_trust_domains": [], "confidentiality_rules": [], "taint_rules": [], "authority_rules": [], "sink_rules": [{"kind": "data_egress", "sink": {"workflow_output": 0}, "trust_domains": {"no_additional_trust_requirement": None}, "sandbox": {"no_additional_minimum": None}}]}, "expected_error": "zero_output_sink_port"},
            ]
        write(f"relation-catalogs/{name}/invalid/cases.json", {"corpus_revision": 1, "kind": "agentflow.conformance.component-contract.relation-invalid", "cases": [
            {"case_id": f"relation/{name}/invalid/{index}", "schema": schema_id, "input_value": item["input_value"], "expected_error": item["expected_error"], "expected_verdict": "violation"}
            for index, item in enumerate(invalids, start=1)
        ]})
        write(f"relation-catalogs/{name}/canonical/cases.json", {"corpus_revision": 1, "kind": "agentflow.conformance.component-contract.relation-canonical", "cases": [
            {"case_id": f"relation/{name}/canonical/order-independent", "input_value": unordered, "expected_canonical_value": nonempty, "expected_canonical_ace1": nonempty_bytes.decode()}
        ]})
        write(f"relation-catalogs/{name}/equivalence/cases.json", {"corpus_revision": 1, "kind": "agentflow.conformance.component-contract.relation-equivalence", "cases": [
            {"case_id": f"relation/{name}/equivalence/same-revision-different-content", "revision": "fixture-r1", "first_checksum": hashlib.sha256(empty_bytes).hexdigest(), "second_checksum": hashlib.sha256(nonempty_bytes).hexdigest(), "equivalent": False, "proof_reuse": "unproven"},
            {"case_id": f"relation/{name}/equivalence/same-content-different-revision", "revision_a": "fixture-r1", "revision_b": "fixture-r2", "checksum": hashlib.sha256(empty_bytes).hexdigest(), "equivalent": False, "proof_reuse": "unproven"},
        ]})


def main():
    schema_golden()
    component_corpora()
    relation_corpora()


if __name__ == "__main__":
    main()
