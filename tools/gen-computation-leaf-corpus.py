#!/usr/bin/env python3
"""Independent Phase 4B.2A leaf-resolution oracle and frozen test vectors.

This implementation is intentionally separate from Rust and emits only
test-only JSON inputs/expectations. It defines no wire schema, ACE1 payload,
or HashV1 domain.
"""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "spec" / "conformance" / "computation-leaf"


def verdict(values: list[str]) -> str:
    for preferred in ("violation", "requires_evidence", "unproven"):
        if preferred in values:
            return preferred
    return "proven"


def exact_target(pin: str, checked_contracts: dict[str, dict]) -> dict:
    target = checked_contracts.get(pin)
    if target is None:
        return {"verdict": "unproven", "target": None}
    return {"verdict": "proven", "target": target}


def map_ports(required: list[dict], bindings: list[dict]) -> dict:
    required_numbers = [port["port"] for port in required]
    bound_numbers = [binding["port"] for binding in bindings]
    problems: list[str] = []
    if len(set(required_numbers)) != len(required_numbers) or len(set(bound_numbers)) != len(bound_numbers):
        problems.append("violation")
    if set(required_numbers) != set(bound_numbers):
        problems.append("violation")
    links = []
    for port in required:
        binding = next((item for item in bindings if item["port"] == port["port"]), None)
        if binding is None:
            continue
        actual = binding.get("public_type_ref")
        if actual is None:
            problems.append("unproven")
        elif actual != port["public_type_ref"]:
            problems.append("violation")
        links.append({"port": port["port"], "binding": binding["target"], "type_ref": actual})
    return {"verdict": verdict(problems), "mappings": links}


def map_resources(requirements: list[dict], bindings: list[dict]) -> dict:
    required_numbers = [item["port"] for item in requirements]
    bound_numbers = [item["port"] for item in bindings]
    problems: list[str] = []
    if len(set(bound_numbers)) != len(bound_numbers) or set(required_numbers) != set(bound_numbers):
        problems.append("violation")
    mappings = []
    aliases: dict[str, list[int]] = {}
    for required in requirements:
        binding = next((item for item in bindings if item["port"] == required["port"]), None)
        if binding is None:
            continue
        if binding.get("type_ref") != required["type_ref"]:
            problems.append("violation")
        if required.get("attributes") and not binding.get("attribute_evidence", False):
            problems.append("unproven")
        aliases.setdefault(binding["resource_slot"], []).append(required["port"])
        mappings.append({"port": required["port"], "resource_slot": binding["resource_slot"], "type_ref": binding.get("type_ref")})
    alias_groups = [
        {"resource_slot": slot, "ports": sorted(ports)}
        for slot, ports in sorted(aliases.items()) if len(ports) > 1
    ]
    return {"verdict": verdict(problems), "mappings": mappings, "aliases": alias_groups}


def scoped_effects(entries: list[dict], caller_digest: str, call_sites: list[int], contract_digest: str) -> list[dict]:
    return [
        {
            "caller_semantic_ir_digest": caller_digest,
            "computation": computation,
            "source_contract_digest": contract_digest,
            "source_entry_ordinal": ordinal,
            "declared_atom": entry["atom"],
            "bounds": entry["bounds"],
            "guard": entry.get("guard", "unconditional"),
        }
        for computation in call_sites
        for ordinal, entry in enumerate(entries)
    ]


def check_output_binding(data: dict) -> dict:
    mapped = map_ports(data["required"], data["bindings"])
    output = data["bindings"][0] if data["bindings"] else None
    if output is None or output.get("producer_computation") != data["computation"]:
        mapped["verdict"] = "violation"
    return mapped


def subflow_interface(target: dict, export: dict | None) -> dict:
    if export is None:
        return {
            "verdict": "requires_evidence",
            "interface_verdict": "proven",
            "export_verdict": "unproven",
            "body_verdict": "requires_evidence",
            "dimensions_synthesized": False,
        }
    expected = target["interface"]
    actual = export["ports"]
    checks = [map_ports(expected[direction], actual[direction])["verdict"] for direction in ("inputs", "outputs")]
    checks.append(map_resources(expected["resources"], actual["resources"])["verdict"])
    interface_verdict = verdict(checks)
    return {
        "verdict": verdict([interface_verdict, "requires_evidence"]),
        "interface_verdict": interface_verdict,
        "export_verdict": "proven" if interface_verdict == "proven" else interface_verdict,
        "body_verdict": "requires_evidence",
        "dimensions_synthesized": False,
    }


def dependency_closure(root: str, graph: dict[str, list[str]]) -> dict:
    state: dict[str, int] = {}
    cycle = False
    missing = False

    def visit(node: str) -> None:
        nonlocal cycle, missing
        if state.get(node) == 1:
            cycle = True
            return
        if state.get(node) == 2:
            return
        if node not in graph:
            missing = True
            return
        state[node] = 1
        for child in graph[node]:
            visit(child)
        state[node] = 2

    visit(root)
    return {"verdict": "violation" if cycle else "unproven" if missing else "proven", "cycle": cycle, "missing": missing}


def context_check(old_context: dict, new_context: dict, canonical_body: str) -> dict:
    # The digest oracle compares canonical body identity, deliberately not a
    # production digest encoding. Proof reuse additionally needs the same
    # context identity and exact catalog content.
    same_body = canonical_body == new_context["canonical_body"]
    same_context = old_context["identity"] == new_context["identity"]
    same_catalog_content = old_context["catalog_content"] == new_context["catalog_content"]
    return {
        "semantic_digest_stable": same_body,
        "old_proof_reusable": same_context and same_catalog_content,
        "verdict": "proven" if same_context and same_catalog_content else "unproven",
    }


def evaluate(case: dict) -> dict:
    operation = case["input"]["operation"]
    data = case["input"]
    if operation == "exact_target":
        return exact_target(data["pin"], data["checked_contracts"])
    if operation == "map_ports":
        return map_ports(data["required"], data["bindings"])
    if operation == "check_output_binding":
        return check_output_binding(data)
    if operation == "map_resources":
        return map_resources(data["requirements"], data["bindings"])
    if operation == "scoped_effects":
        values = scoped_effects(data["entries"], data["caller_digest"], data["call_sites"], data["contract_digest"])
        return {"verdict": "proven", "occurrences": values, "occurrences_distinct": len({(item["caller_semantic_ir_digest"], item["computation"]) for item in values}) == len(values)}
    if operation == "subflow_interface":
        return subflow_interface(data["target"], data.get("export"))
    if operation == "dependency_closure":
        return dependency_closure(data["root"], data["graph"])
    if operation == "context_check":
        return context_check(data["old_context"], data["new_context"], data["canonical_body"])
    if operation == "preservation":
        # A pure projection: preserve every source object byte-for-byte in
        # this JSON model and add only explicit coordinate links.
        return {"verdict": data["verdict"], "preserved": data["source"], "mapping": data["mapping"]}
    raise ValueError(f"unknown oracle operation {operation!r}")


TYPE_A = {"specification_id": "agentflow.test.types@1", "type_name": "RecordA", "schema_digest": "a" * 64}
TYPE_B = {"specification_id": "agentflow.test.types@1", "type_name": "RecordB", "schema_digest": "b" * 64}
CONTRACT_DIGEST = "component-contract:" + "c" * 64
CALLER_DIGEST = "semantic-ir:" + "d" * 64


def case(case_id: str, group: str, operation: str, input_value: dict, note: str) -> dict:
    item = {"case_id": case_id, "group": group, "kind": "agentflow.conformance.computation-leaf.test-vector", "input": {"operation": operation, **input_value}, "note": note}
    item["expected"] = evaluate(item)
    return item


CASES = [
    case("computation-leaf/invoke/exact-contract", "golden", "exact_target", {"pin": CONTRACT_DIGEST, "checked_contracts": {CONTRACT_DIGEST: {"catalog": "agentflow.components@1", "name": "acme.transform"}}}, "Exact digest maps to identity carried by the checked contract."),
    case("computation-leaf/invoke/wrong-contract-digest", "negative", "exact_target", {"pin": "component-contract:" + "e" * 64, "checked_contracts": {CONTRACT_DIGEST: {"catalog": "agentflow.components@1", "name": "acme.transform"}}}, "A stale digest does not fall back to an available target."),
    case("computation-leaf/invoke/exact-input-type", "invoke", "map_ports", {"required": [{"port": 1, "public_type_ref": TYPE_A}], "bindings": [{"port": 1, "target": "expression:8", "public_type_ref": TYPE_A}]}, "Exact public TypeSchemaRef matches."),
    case("computation-leaf/invoke/wrong-public-input-type", "negative", "map_ports", {"required": [{"port": 1, "public_type_ref": TYPE_A}], "bindings": [{"port": 1, "target": "expression:8", "public_type_ref": TYPE_B}]}, "Equal local ids or names cannot replace the pinned public type."),
    case("computation-leaf/invoke/duplicate-required-input", "negative", "map_ports", {"required": [{"port": 1, "public_type_ref": TYPE_A}], "bindings": [{"port": 1, "target": "expression:8", "public_type_ref": TYPE_A}, {"port": 1, "target": "expression:9", "public_type_ref": TYPE_A}]}, "Duplicate binding is a violation."),
    case("computation-leaf/invoke/missing-required-input", "negative", "map_ports", {"required": [{"port": 1, "public_type_ref": TYPE_A}], "bindings": []}, "Required inputs are not defaulted."),
    case("computation-leaf/invoke/unmapped-local-type", "negative", "map_ports", {"required": [{"port": 1, "public_type_ref": TYPE_A}], "bindings": [{"port": 1, "target": "expression:8", "public_type_ref": None}]}, "Local non-schema type cannot be mapped to a public pin."),
    case("computation-leaf/invoke/invalid-output-producer", "negative", "check_output_binding", {"required": [{"port": 1, "public_type_ref": TYPE_A}], "bindings": [{"port": 1, "target": "value-slot:4-from-sibling", "public_type_ref": TYPE_A, "producer_computation": 10}], "computation": 12}, "Output destination must be produced by this exact leaf and pass Phase 3C."),
    case("computation-leaf/invoke/same-contract-two-call-sites", "invoke", "scoped_effects", {"entries": [{"atom": {"operation": "storage/write", "resource_port": 1, "qualifiers": [{"family": "idempotency", "member": "natural"}]}, "bounds": [1, 2]}], "caller_digest": CALLER_DIGEST, "call_sites": [12, 13], "contract_digest": CONTRACT_DIGEST}, "Same contract at two nodes produces two separately scoped occurrences."),
    case("computation-leaf/mapping/resource-alias-explicit", "mapping", "map_resources", {"requirements": [{"port": 1, "type_ref": TYPE_A, "attributes": []}, {"port": 2, "type_ref": TYPE_A, "attributes": []}], "bindings": [{"port": 1, "resource_slot": "resource-slot:7", "type_ref": TYPE_A, "attribute_evidence": True}, {"port": 2, "resource_slot": "resource-slot:7", "type_ref": TYPE_A, "attribute_evidence": True}]}, "Alias group is explicit and has no bound-reduction operation."),
    case("computation-leaf/mapping/numeric-namespaces-distinct", "mapping", "preservation", {"source": {"component_input_port": 1, "resource_slot": 1}, "mapping": {"input_expression": "expression:1", "resource_slot": "resource-slot:1"}, "verdict": "proven"}, "Equal integers in port and resource namespaces remain distinct identities."),
    case("computation-leaf/mapping/wrong-resource-type", "negative", "map_resources", {"requirements": [{"port": 1, "type_ref": TYPE_A, "attributes": []}], "bindings": [{"port": 1, "resource_slot": "resource-slot:3", "type_ref": TYPE_B, "attribute_evidence": True}]}, "A resource slot requires an exact type pin."),
    case("computation-leaf/mapping/required-attributes-without-evidence", "negative", "map_resources", {"requirements": [{"port": 1, "type_ref": TYPE_A, "attributes": [{"key": "tenant", "value": "t1"}]}], "bindings": [{"port": 1, "resource_slot": "resource-slot:3", "type_ref": TYPE_A, "attribute_evidence": False}]}, "Missing attribute evidence is Unproven."),
    case("computation-leaf/mapping/effect-qualifiers-preserved", "mapping", "preservation", {"source": {"operation": "storage/write", "resource_selector": {"requirement_port_no": 1, "scope_bindings": [{"key": "tenant", "value": "t1"}]}, "qualifiers": [{"family": "idempotency", "member": "natural"}]}, "mapping": {"resource_slot": "resource-slot:8"}, "verdict": "proven"}, "Mapping supplements rather than rewrites the full atom."),
    case("computation-leaf/mapping/unknown-external-flow-preserved", "mapping", "preservation", {"source": {"category": "unknown_external", "influence_unknown": True, "instruction_authority": "external_data"}, "mapping": {"preserve_unknown": True, "authority_inferred": False}, "verdict": "proven"}, "Unknown source and may-carry authority remain conservative."),
    case("computation-leaf/mapping/recovery-action-full-effect-key", "mapping", "preservation", {"source": {"forward_effect": {"operation": "storage/write", "resource_port": 1, "qualifiers": [{"family": "idempotency", "member": "natural"}]}, "action_effect": {"operation": "storage/compensate", "resource_port": 1, "qualifiers": []}}, "mapping": {"source_action_identity_retained": True}, "verdict": "proven"}, "Recovery retains exact full EffectAtom identities."),
    case("computation-leaf/mapping/missing-usage-not-zero", "mapping", "preservation", {"source": {"usage": "missing", "recovery": "keyed", "execution": "compatible"}, "mapping": {"usage_defaulted_to": None, "provider_idempotency_evidence": False, "worker_admitted": False}, "verdict": "proven"}, "Usage, recovery, and execution are copied without authority inference."),
    case("computation-leaf/subflow/exact-export-interface", "subflow", "subflow_interface", {"target": {"interface": {"inputs": [{"port": 1, "public_type_ref": TYPE_A}], "outputs": [{"port": 1, "public_type_ref": TYPE_A}], "resources": []}}, "export": {"ports": {"inputs": [{"port": 1, "target": "input:1", "public_type_ref": TYPE_A}], "outputs": [{"port": 1, "target": "output:1", "public_type_ref": TYPE_A}], "resources": []}, "full_dimensions": ["Phi", "Effect", "Flow", "Usage", "Recovery", "Execution"]}}, "Declaration resolves, but body conformance still requires evidence."),
    case("computation-leaf/subflow/derived-io-not-full-contract", "subflow", "subflow_interface", {"target": {"interface": {"inputs": [], "outputs": [], "resources": []}}, "export": None}, "Derived I/O/R alone cannot fill missing dimensions."),
    case("computation-leaf/subflow/wrong-target-version", "negative", "exact_target", {"pin": "semantic-ir:" + "f" * 64, "checked_contracts": {"semantic-ir:" + "0" * 64: {"version": "old"}}}, "Wrong target version is never silently followed."),
    case("computation-leaf/subflow/direct-cycle", "subflow", "dependency_closure", {"root": "ir-A", "graph": {"ir-A": ["ir-A"]}}, "Direct dependency cycle is a violation."),
    case("computation-leaf/subflow/indirect-cycle", "subflow", "dependency_closure", {"root": "ir-A", "graph": {"ir-A": ["ir-B"], "ir-B": ["ir-C"], "ir-C": ["ir-A"]}}, "Indirect dependency cycle is a violation."),
    case("computation-leaf/subflow/missing-target-closure", "subflow", "dependency_closure", {"root": "ir-A", "graph": {"ir-A": ["ir-missing"]}}, "Missing exact dependency target is Unproven."),
    case("computation-leaf/subflow/declaration-needs-body-proof", "subflow", "subflow_interface", {"target": {"interface": {"inputs": [], "outputs": [], "resources": []}}, "export": {"ports": {"inputs": [], "outputs": [], "resources": []}, "full_dimensions": ["Phi", "Effect", "Flow", "Usage", "Recovery", "Execution"]}}, "Export declaration match is not implementation proof."),
    case("computation-leaf/await/typed-signal-partial", "await", "preservation", {"source": {"signal_type_ref": TYPE_A, "correlation_expression": "expression:2"}, "mapping": {"signal_pin_exact": True, "phase3c_correlation_valid": True, "interaction_contract": "missing", "implicit_timeout": False}, "verdict": "unproven"}, "The signal type and correlation are pinned; adapter result remains partial."),
    case("computation-leaf/await/unknown-signal-provenance", "await", "preservation", {"source": {"category": "unknown_external", "influence_unknown": True}, "mapping": {"unknown_origin_retained": True, "authority_inferred": False}, "verdict": "unproven"}, "Signal provenance is not self-elevated."),
    case("computation-leaf/await/no-implicit-timeout", "await", "preservation", {"source": {"opcode": "AWAIT"}, "mapping": {"timeout": "absent", "timer_created": False}, "verdict": "unproven"}, "No timeout semantics are inferred from AWAIT."),
    case("computation-leaf/await/cancellation-not-business-failure", "await", "preservation", {"source": {"control_outcome": "cancelled"}, "mapping": {"failure_row_mutated": False}, "verdict": "unproven"}, "Cancellation remains outside the business Failure row."),
    case("computation-leaf/context/same-body-different-context", "context", "context_check", {"canonical_body": "canonical-contract-body-A", "old_context": {"identity": "ctx-A", "catalog_content": "content-A"}, "new_context": {"identity": "ctx-B", "catalog_content": "content-A", "canonical_body": "canonical-contract-body-A"}}, "Semantic body identity is stable; prior proof is not portable."),
    case("computation-leaf/context/changed-catalog-invalidates-proof", "context", "context_check", {"canonical_body": "canonical-contract-body-A", "old_context": {"identity": "ctx-A", "catalog_content": "content-A"}, "new_context": {"identity": "ctx-B", "catalog_content": "content-B", "canonical_body": "canonical-contract-body-A"}}, "Changed relation content requires revalidation."),
    case("computation-leaf/context/same-version-different-content", "context", "context_check", {"canonical_body": "canonical-contract-body-A", "old_context": {"identity": "ctx-A", "catalog_version": "r1", "catalog_content": "hash-A"}, "new_context": {"identity": "ctx-B", "catalog_version": "r1", "catalog_content": "hash-B", "canonical_body": "canonical-contract-body-A"}}, "A reused revision label cannot hide changed content."),
    case("computation-leaf/equivalence/source-order-unordered", "equivalence", "preservation", {"source": {"ports": [{"port": 1, "type_ref": TYPE_A}, {"port": 2, "type_ref": TYPE_B}]}, "mapping": {"canonical_ports": [{"port": 1, "type_ref": TYPE_A}, {"port": 2, "type_ref": TYPE_B}]}, "verdict": "proven"}, "Semantically unordered declarations keep deterministic output ordering."),
    case("computation-leaf/equivalence/same-snapshot-deterministic", "equivalence", "preservation", {"source": {"snapshot": "exact-A", "node": 9}, "mapping": {"same_output_on_repeat": True}, "verdict": "proven"}, "One exact snapshot yields one deterministic leaf result."),
    case("computation-leaf/equivalence/presentation-metadata-only", "equivalence", "preservation", {"source": {"contract_body": "body-A", "display_metadata": "before"}, "mapping": {"contract_body": "body-A", "digest_changed": False}, "verdict": "proven"}, "Presentation metadata does not alter semantic contract identity."),
]


def write_case(item: dict) -> None:
    group = item["group"]
    bucket = "golden" if group == "golden" else group
    path = OUT / bucket / (item["case_id"].split("/")[-1] + ".json")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(item, indent=2, ensure_ascii=False, sort_keys=True) + "\n", encoding="utf-8", newline="\n")


def main() -> None:
    expected_ids = set()
    for item in CASES:
        if item["case_id"] in expected_ids:
            raise ValueError(f"duplicate case id: {item['case_id']}")
        expected_ids.add(item["case_id"])
        write_case(item)
    observed = {item["expected"].get("verdict") for item in CASES}
    required = {"proven", "violation", "unproven", "requires_evidence"}
    if not required.issubset(observed):
        raise ValueError(f"corpus does not cover all proof verdicts: {sorted(required - observed)}")
    manifest = {
        "kind": "agentflow.conformance.computation-leaf.test-corpus",
        "corpus_revision": 1,
        "format_note": "test vectors only; not an AFIDL, ACE1, or HashV1 ABI",
        "case_count": len(CASES),
        "case_ids": sorted(expected_ids),
    }
    path = OUT / "manifest.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False, sort_keys=True) + "\n", encoding="utf-8", newline="\n")
    print(f"generated {len(CASES)} computation-leaf vectors")


if __name__ == "__main__":
    main()
