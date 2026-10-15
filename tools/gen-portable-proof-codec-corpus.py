#!/usr/bin/env python3
"""Independent portable proof codec expectations, generated without Rust."""

from __future__ import annotations

import copy
from legacy_edition1_inputs import assert_historical_output
import base64
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "tools"))
from portable_proof_python_oracle import (  # noqa: E402
    AFIDL, ROOT, b64url, canonical_bytes, hash_v1, ref, sha256,
)
from legacy_edition1_inputs import pinned_bytes

OUT = ROOT / "spec" / "conformance" / "portable-proof-codec"

STATIC_SPECS = sorted([
    "agentflow.component.contract.semantics@1",
    "agentflow.component.relation-catalogs.semantics@1",
    "agentflow.effect.signature.semantics@1",
    "agentflow.execution.requirements.semantics@1",
    "agentflow.failure.row.semantics@1",
    "agentflow.information.flow.semantics@1",
    "agentflow.recovery.contract.semantics@1",
    "agentflow.resource.requirement.semantics@1",
    "agentflow.semantic.advanced-control-contract-algebra.semantics@1",
    "agentflow.semantic.control-contract-algebra.semantics@1",
    "agentflow.semantic.ir.semantics@1",
    "agentflow.semantic.leaf-contract-resolution.semantics@1",
    "agentflow.semantic.scope-compensation-contract-algebra.semantics@1",
    "agentflow.semantic.types@1",
    "agentflow.usage.signature.semantics@1",
    "agentflow.computation.root-export-contract-proof.semantics@1",
])
STATIC_RULE_IDS = sorted(
    [f"AF-CONTRACT-{number:04}" for number in range(1, 23)]
    + [f"AF-SEMANTIC-{number:04}" for number in range(111, 173)]
)
ROLE_NAMES = [
    "static_analysis", "contract_validation", "semantic_ir_validation", "dataflow",
    "leaf_resolution", "control_algebra", "root_recovery", "root_export",
    "plan_lowering", "runtime_semantics_profile",
]
DEPENDENCIES = {
    "contract_validation": [],
    "semantic_ir_validation": [],
    "dataflow": ["semantic_ir_validation"],
    "leaf_resolution": ["contract_validation", "semantic_ir_validation", "dataflow"],
    "control_algebra": ["leaf_resolution"],
    "root_recovery": ["dataflow", "control_algebra"],
    "root_export": [
        "contract_validation", "semantic_ir_validation", "dataflow",
        "leaf_resolution", "control_algebra", "root_recovery",
    ],
    "static_analysis": [
        "contract_validation", "semantic_ir_validation", "dataflow",
        "leaf_resolution", "control_algebra", "root_recovery", "root_export",
    ],
}


def tagged(case, value):
    return {case: value}


def checksum_blob(data):
    return {"content_sha256": {"algorithm": "sha256", "hex": sha256(data)},
            "content": b64url(data)}


def type_schema_pins(schema_refs):
    return copy.deepcopy(schema_refs)


def empty_contract():
    return {
        "component_identity": {
            "catalog": "agentflow.portable.fixture@1",
            "name": "portable.codec",
        },
        "inputs": [], "outputs": [],
        "failures": {"failures": []},
        "effects": {"row": tagged("closed", [])},
        "resources": {"requirements": []},
        "information_flow": {
            "confidentiality_restrictions": [],
            "summary": {
                "sources": [], "taints": [], "instruction_authority": "none",
                "influence_unknown": False,
            },
            "flow_constraints": [],
        },
        "usage": {"limits": []},
        "recovery": {"replay_semantics": "recompute", "effect_recovery_clauses": []},
        "execution": {
            "capabilities": [], "quantities": [],
            "sandbox": {"none": None}, "trust_domain": {"none": None},
            "region_residency": {"none": None}, "runtime_profile": {"none": None},
            "implementation_abis": [], "soft_preferences": [],
        },
        "effect_flow_partitions": [],
        "recovery_action_bindings": [],
        "recovery_usage_bindings": [],
    }


def source_inventory(specifications):
    rows = []
    for entry in specifications["specifications"]:
        path = entry["normative_source"]["logical_path"]
        raw = (ROOT / path).read_bytes()
        rows.append({
            "specification_id": entry["specification_id"],
            "logical_path": path,
            "content_sha256": {"algorithm": "sha256", "hex": sha256(raw)},
            "content": b64url(raw),
        })
    return rows


def rule_identity(rule):
    return {
        "rule_id": rule["rule_id"],
        "normative_source": rule["normative_source"],
        "source_anchor": rule["source_anchor"],
        "short_statement": rule["short_statement"],
    }


def compiler_manifest(specifications, rules, module_refs, profile=None):
    version = profile["version"] if profile else "agentflow.root-export-checker@1"
    selected_specs = profile["spec_ids"] if profile else STATIC_SPECS
    selected_ids = profile["rule_ids"] if profile else STATIC_RULE_IDS
    role_rows = []
    for role in ROLE_NAMES:
        if role in ("plan_lowering", "runtime_semantics_profile"):
            declaration = tagged("model_missing", "not_formalized")
        else:
            declaration = tagged("defined", {
                "analysis_semantics_version": version,
                "normative_specifications": selected_specs.copy(),
                "required_schema_pins": type_schema_pins(module_refs),
                "depends_on": DEPENDENCIES[role].copy(),
            })
        role_rows.append({"role": role, "declaration": declaration})
    selected_rules = [rule_identity(row) for row in rules["rules"]
                      if row["rule_id"] in set(selected_ids)]
    if [row["rule_id"] for row in selected_rules] != selected_ids:
        raise ValueError("the frozen applied rule set no longer matches the registered Rules Registry")
    return {
        "registries": {
            "specifications": checksum_blob(profile["specification_bytes"] if profile else pinned_bytes("specifications")),
            "rules": checksum_blob(profile["rule_bytes"] if profile else pinned_bytes("rules")),
        },
        "closed_tables": {
            "hash_domains": checksum_blob((ROOT / "spec/tables/hash-domains.json").read_bytes()),
            "abi_numbers": checksum_blob((ROOT / "spec/registry/abi-numbers.json").read_bytes()),
            "rule_areas": checksum_blob((ROOT / "spec/tables/rule-areas.json").read_bytes()),
            "afidl_primitives": checksum_blob((ROOT / "spec/tables/afidl-primitives.json").read_bytes()),
        },
        "normative_sources": source_inventory(specifications),
        "semantic_roles": role_rows,
        "attribution": tagged("full_content_revalidation", {"listed_rules": selected_rules}),
        "static_identity_projection": {
            "analysis_semantics_version": version,
            "normative_specification_ids": selected_specs.copy(),
            "applied_rules": selected_rules,
        },
    }


def bundle_and_context(profile=None):
    specifications = json.loads(profile["specification_bytes"] if profile else pinned_bytes("specifications"))
    rules = json.loads(profile["rule_bytes"] if profile else pinned_bytes("rules"))
    schema_modules = sorted((ROOT / "spec/idl").rglob("*.afidl"))
    sources = [(path.relative_to(ROOT).as_posix(), path.read_text(encoding="utf-8"))
               for path in schema_modules]
    modules, types_by_module = AFIDL.compile_module_set(sources)
    all_schemas = []
    full_refs = []
    legacy_module_rows = []
    for specification_id in sorted(types_by_module):
        refs = []
        for schema in types_by_module[specification_id]:
            body, digest = AFIDL.type_schema_digest(schema)
            schema_ref = ref(schema["schema"], schema["name"], digest)
            all_schemas.append({"reference": schema_ref, "canonical_content": b64url(body)})
            full_refs.append(schema_ref)
            if schema["schema"] != "agentflow.portable.proof.inputs@1":
                refs.append(schema_ref)
        if refs:
            legacy_module_rows.append({
                "specification_id": specification_id,
                "named_types": sorted(refs, key=lambda item: item["type_name"].encode("utf-8")),
            })
    all_schemas.sort(key=lambda row: (
        row["reference"]["specification_id"].encode("utf-8"),
        row["reference"]["type_name"].encode("utf-8"),
        row["reference"]["schema_digest"]["algorithm"],
        row["reference"]["schema_digest"]["domain"],
        row["reference"]["schema_digest"]["hex"],
    ))
    full_refs.sort(key=lambda row: (
        row["specification_id"].encode("utf-8"),
        row["type_name"].encode("utf-8"),
        row["schema_digest"]["algorithm"],
        row["schema_digest"]["domain"],
        row["schema_digest"]["hex"],
    ))
    root_ir_shape = next(s for s in types_by_module["agentflow.semantic.ir@1"]
                         if s["name"] == "SemanticIrDocument")
    root_contract_shape = next(s for s in types_by_module["agentflow.component.contract@1"]
                               if s["name"] == "ComponentContract")
    _, ir_schema_digest = AFIDL.type_schema_digest(root_ir_shape)
    _, contract_schema_digest = AFIDL.type_schema_digest(root_contract_shape)
    ir_schema_ref = ref("agentflow.semantic.ir@1", "SemanticIrDocument", ir_schema_digest)
    contract_schema_ref = ref("agentflow.component.contract@1", "ComponentContract", contract_schema_digest)

    # A minimal typed IR with canonical int64/decimal PrimitiveConstants and
    # one owned INVOKE leaf. D0-D4 check wire/content identity only; semantic
    # IR checking and expression reachability belong to D7.
    ir_value = {
        "semantic_refs": [],
        "types": [
            {"id": 1, "body": {"primitive": "int64"}},
            {"id": 2, "body": {"primitive": "decimal"}},
        ],
        "values": [],
        "expressions": [
            {"id": 1, "result_type": 1, "body": {"constant": {"int64": "9007199254740993"}}},
            {"id": 2, "result_type": 2, "body": {"constant": {"decimal": "12345678901234567890.000000000000000001"}}},
        ],
        "resources": [],
        "contracts": [{"id": 1, "kind": "component", "digest": {
            "algorithm": "sha256", "domain": "component-contract",
            "hex": hash_v1("component-contract", canonical_bytes(empty_contract())),
        }}],
        "run_parameters": [], "workflow_outputs": [],
        "computations": [{
            "id": 1, "outputs": [],
            "body": {"invoke": {"contract": 1, "inputs": [], "resources": []}},
        }],
        "root": 1,
    }
    ir_bytes = canonical_bytes(ir_value)
    ir_digest = hash_v1("semantic-ir", ir_bytes)
    contract_value = empty_contract()
    contract_bytes = canonical_bytes(contract_value)
    contract_digest = hash_v1("component-contract", contract_bytes)
    ir_content = {
        "digest": {"algorithm": "sha256", "domain": "semantic-ir", "hex": ir_digest},
        "type_schema": ir_schema_ref,
    }
    contract_content = {
        "digest": {"algorithm": "sha256", "domain": "component-contract", "hex": contract_digest},
        "type_schema": contract_schema_ref,
    }
    compiler = compiler_manifest(specifications, rules, full_refs, profile)

    context = {
        "id": 1,
        "content": {
            "compiled_modules": tagged("complete", legacy_module_rows),
            "type_schemas": tagged("complete", full_refs),
            "effect_catalogs": tagged("complete", []),
            "resource_types": {"schemas": tagged("complete", []), "descriptors": tagged("complete", [])},
            "flow_catalogs": {
                "confidentiality_labels": tagged("complete", []),
                "taints": tagged("complete", []),
                "effect_sinks": tagged("complete", []),
            },
            "usage_meters": tagged("complete", []),
            "recovery_operations": {
                "effect_catalogs": tagged("complete", []),
                "reconciliation_operations": tagged("complete", []),
            },
            "execution_universes": {
                "capabilities": tagged("complete", []),
                "trust_domains": tagged("complete", []),
                "regions": tagged("complete", []),
                "region_compatibility": tagged("complete", []),
                "runtime_profiles": tagged("complete", []),
            },
            "relation_snapshots": {
                "effect_resource": {"none": None},
                "recovery_usage": {"none": None},
                "flow_execution": {"none": None},
            },
        },
    }

    ref_key = {
        "digest": ir_content["digest"],
        "type_schema": ir_schema_ref,
    }
    component_ref_key = {
        "digest": contract_content["digest"],
        "type_schema": contract_schema_ref,
    }
    source_association = {
        "caller_ir": ref_key,
        "source_node": 1,
        "target": {"invoke": component_ref_key},
    }
    resolver = {
        "id": 1,
        "content": {
            "caller_ir": ref_key,
            "context_id": 1,
            "components": [component_ref_key],
            "subflow_targets": [],
            "source_associations": [source_association],
        },
    }
    portable_assertion = {
        "exact_semantic_ir_digest": ir_content["digest"],
        "exact_exported_component_contract_digest": contract_content["digest"],
        "compiler_semantics_id": 1,
        "proof_context_manifest": {"context_id": 1, "resolver_id": 1},
        "input_mappings": [], "output_mappings": [], "resource_mappings": [],
        "effect_mappings": [], "flow_mappings": [], "recovery_mappings": [],
        "execution_mappings": [], "subflow_dependencies": [],
    }
    assertion_entry = {
        "id": 1,
        "ir_asset": ref_key,
        "export_contract_asset": component_ref_key,
        "assertion": portable_assertion,
    }
    bundle = {
        "schema_version": "portable_proof_inputs_v1",
        "root_assertion_id": 1,
        "schema_assets": all_schemas,
        "ir_assets": [{"reference": ref_key, "canonical_content": b64url(ir_bytes)}],
        "contract_assets": [{"reference": component_ref_key, "canonical_content": b64url(contract_bytes)}],
        "structural_types": [],
        "proof_contexts": [context],
        "resolver_dependency_manifests": [resolver],
        "compiler_semantics_manifests": [{"id": 1, "content": compiler}],
        "root_export_assertions": [assertion_entry],
    }
    return bundle


def b64(data):
    import base64
    return base64.urlsafe_b64encode(data).rstrip(b"=").decode("ascii")


def walk_pointer(value, path, create=False):
    current = value
    for token in path[:-1]:
        current = current[int(token)] if isinstance(current, list) else current[token]
    return current, path[-1]


def machine_text(item):
    if isinstance(item, bool):
        return item
    if isinstance(item, int):
        return str(item)
    if isinstance(item, list):
        return [machine_text(child) for child in item]
    if isinstance(item, dict):
        return {name: machine_text(child) for name, child in item.items()}
    return item


def apply_mutations(value, mutations):
    # JSON values have tree identity. deepcopy preserves Python object aliases,
    # which would make an edit to a reused fixture object change several wire
    # occurrences at once. Round-tripping first materializes the actual tree.
    value = json.loads(json.dumps(value, ensure_ascii=False))
    mutations = machine_text(list(mutations))
    for mutation in mutations:
        operation = mutation["op"]
        path = mutation.get("path", [])
        if operation == "set":
            parent, leaf = walk_pointer(value, path)
            if isinstance(parent, list):
                parent[int(leaf)] = machine_text(copy.deepcopy(mutation["value"]))
            else:
                parent[leaf] = machine_text(copy.deepcopy(mutation["value"]))
        elif operation == "remove":
            parent, leaf = walk_pointer(value, path)
            if isinstance(parent, list):
                del parent[int(leaf)]
            else:
                del parent[leaf]
        elif operation == "reverse":
            parent, leaf = walk_pointer(value, path)
            target = parent[int(leaf)] if isinstance(parent, list) else parent[leaf]
            target.reverse()
        elif operation == "append-copy":
            target = value
            for part in path:
                target = target[int(part)] if isinstance(target, list) else target[part]
            target.append(copy.deepcopy(target[-1]))
        elif operation == "remove-index":
            target = value
            for part in path[:-1]:
                target = target[int(part)] if isinstance(target, list) else target[part]
            del target[int(path[-1])]
        elif operation == "bump_registry_revision":
            registry = value["compiler_semantics_manifests"][0]["content"]["registries"]["rules"]
            encoded = registry["content"]
            raw = base64.urlsafe_b64decode(encoded + "=" * ((4 - len(encoded) % 4) % 4))
            document = json.loads(raw)
            document["registry_revision"] += 1
            changed = json.dumps(document, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")
            registry["content"] = b64(changed)
            registry["content_sha256"]["hex"] = sha256(changed)
        else:
            raise ValueError(f"unknown typed fixture mutation {operation!r}")
    return value


def raw_ace1(value, *, preserve_non_nfc=False):
    if preserve_non_nfc:
        def emit(item):
            if item is None: return "null"
            if item is True: return "true"
            if item is False: return "false"
            if isinstance(item, int): return json.dumps(str(item))
            if isinstance(item, str): return json.dumps(item, ensure_ascii=False, separators=(",", ":"))
            if isinstance(item, list): return "[" + ",".join(emit(child) for child in item) + "]"
            if isinstance(item, dict):
                return "{" + ",".join(json.dumps(key, ensure_ascii=False) + ":" + emit(item[key])
                                      for key in sorted(item, key=lambda key: key.encode("utf-8"))) + "}"
            raise TypeError(type(item).__name__)
        return emit(value).encode("utf-8")
    return canonical_bytes(value)


def digest_value(domain, value):
    return {"algorithm": "sha256", "domain": domain,
            "hex": hash_v1(domain, canonical_bytes(value))}


def effect_atom():
    return {
        "operation": {"catalog": "agentflow.portable.fixture@1", "name": "write"},
        "resource_selector": {"requirement_port_no": 1, "scope_bindings": []},
        "qualifiers": [],
    }


def effect_mapping(bundle, guard):
    assertion = bundle["root_export_assertions"][0]["assertion"]
    atom = effect_atom()
    caller = bundle["ir_assets"][0]["reference"]["digest"]
    contract = bundle["contract_assets"][0]["reference"]["digest"]
    identity = {
        "caller_semantic_ir_digest": caller,
        "source_node": 1,
        "source_contract_digest": contract,
        "source_entry_ordinal": 0,
        "atom": atom,
        "guard": {"unconditional": None},
    }
    assertion["effect_mappings"] = [{
        "source": identity, "root_resource_slot": {"none": None},
        "export_atom": atom, "export_guard": guard,
    }]


def flow_mapping(bundle, source_binding):
    contract = bundle["contract_assets"][0]["reference"]["digest"]
    bundle["root_export_assertions"][0]["assertion"]["flow_mappings"] = [{
        "source_node": 1,
        "source_contract_digest": contract,
        "flow_mapping_ordinal": 0,
        "source": {"none": None}, "sink": {"none": None},
        "source_binding": source_binding, "sink_binding": {"none": None},
        "export_source": {"none": None}, "export_sink": {"none": None},
        "source_effect_partition": {"none": None},
        "export_effect_partition": {"none": None},
    }]


def make_case(case_id, group, base, mutations=(), expected="D4", error_family=None,
              *, raw=False, note="", required_fields=None, raw_mutation=None,
              decoder_limits=None):
    typed = apply_mutations(base, mutations)
    payload = raw_ace1(typed, preserve_non_nfc=raw)
    if raw_mutation == "invalid_utf8":
        payload = b"\xff"
    elif raw_mutation == "duplicate_key":
        encoded = raw_ace1(base).decode("utf-8")
        needle = '"schema_version":"portable_proof_inputs_v1"'
        payload = encoded.replace(
            needle,
            needle + ',"schema_version":"portable_proof_inputs_v1"',
            1,
        ).encode("utf-8")
    elif raw_mutation == "leading_whitespace":
        payload = b" " + raw_ace1(base)
    elif raw_mutation == "trailing_whitespace":
        payload = raw_ace1(base) + b" "
    elif raw_mutation == "unnecessary_escape":
        payload = raw_ace1(base).replace(
            b'"portable_proof_inputs_v1"', b'"portable_proof_inputs\\u005fv1"', 1,
        )
    accepted = expected == "D4" and error_family is None
    result = {
        "case_id": case_id,
        "group": group,
        "mutations": machine_text(list(mutations)),
        "raw_mutation": raw_mutation,
        "raw_noncanonical": raw,
        "decoder_limits": decoder_limits,
        "input_byte_length": len(payload),
        "input_sha256": sha256(payload),
        "expected_stage": expected,
        "expected_error_family": error_family,
        "required_semantic_field_paths": required_fields or [
            "$.schema_version", "$.root_assertion_id", "$.schema_assets",
            "$.ir_assets", "$.contract_assets", "$.structural_types",
            "$.proof_contexts", "$.resolver_dependency_manifests",
            "$.compiler_semantics_manifests", "$.root_export_assertions",
        ],
        "expected_bundle_sha256": sha256(payload) if accepted else None,
        "expected_bundle_byte_length": len(payload) if accepted else None,
        "expected_typeschema_count": len(base["schema_assets"]) if accepted else None,
        "trust_boundary": "current-static-inputs-only" if accepted else "no-capability",
        "mutation_description": note,
    }
    return result


def generate():
    base = bundle_and_context()
    base_bytes = canonical_bytes(base)
    OUT.mkdir(parents=True, exist_ok=True)
    assert_historical_output(OUT / "full-ten-field-bundle.ace1", base_bytes)
    cases = []
    add = cases.append
    root = ["root_assertion_id"]

    add(make_case("portable-proof-codec/positive/full-ten-field-d4", "roundtrip", base,
                  expected="D4", note="Full inline Bundle, every field and role; current Profile A."))
    add(make_case("portable-proof-codec/d0-unknown-required-field", "ace1-shape", base,
                  ({"op": "set", "path": ["origin_approved"], "value": True},),
                  "D0", "UnknownField"))
    add(make_case("portable-proof-codec/d0-missing-required-field", "ace1-shape", base,
                  ({"op": "remove", "path": ["schema_version"]},), "D0", "MissingField"))
    add(make_case("portable-proof-codec/d0-invalid-enum", "ace1-shape", base,
                  ({"op": "set", "path": ["schema_version"], "value": "portable_v9"},),
                  "D0", "UnknownEnumCase"))
    add(make_case("portable-proof-codec/d0-invalid-option", "ace1-shape", base,
                  ({"op": "set", "path": ["proof_contexts", "0", "content", "relation_snapshots", "effect_resource"], "value": None},),
                  "D0", "InvalidOption"))
    add(make_case("portable-proof-codec/d0-u32-overflow", "ace1-shape", base,
                  ({"op": "set", "path": root, "value": str(2**32)},), "D0", "NumericRange"))
    add(make_case("portable-proof-codec/d0-u64-overflow", "ace1-shape", base,
                  ({"op": "set", "path": ["root_export_assertions", "0", "assertion", "flow_mappings"],
                    "value": [{
                        "source_node": "1", "source_contract_digest": base["contract_assets"][0]["reference"]["digest"],
                        "flow_mapping_ordinal": "18446744073709551616", "source": {"none": None}, "sink": {"none": None},
                        "source_binding": {"some": {"unknown_external": None}}, "sink_binding": {"none": None},
                        "export_source": {"none": None}, "export_sink": {"none": None},
                        "source_effect_partition": {"none": None}, "export_effect_partition": {"none": None},
                    }]},), "D0", "NumericRange"))
    add(make_case("portable-proof-codec/d0-invalid-width-type", "ace1-shape", base,
                  ({"op": "set", "path": root, "value": True},), "D0", "InvalidPrimitive"))
    add(make_case("portable-proof-codec/d0-invalid-utf8", "ace1-shape", base,
                  expected="D0", error_family="InvalidUtf8", raw_mutation="invalid_utf8"))
    add(make_case("portable-proof-codec/d0-invalid-base64url", "ace1-shape", base,
                  ({"op": "set", "path": ["schema_assets", "0", "canonical_content"], "value": "bad=="},),
                  "D0", "InvalidBytesEncoding"))
    unknown_guard = copy.deepcopy(base)
    effect_mapping(unknown_guard, {"unknown_guard": {"port": "1"}})
    add(make_case("portable-proof-codec/d0-unknown-effect-guard", "ace1-shape", base,
                  ({"op": "set", "path": ["root_export_assertions", "0", "assertion", "effect_mappings"],
                    "value": unknown_guard["root_export_assertions"][0]["assertion"]["effect_mappings"]},),
                  "D0", "UnknownVariant"))
    unknown_binding = copy.deepcopy(base)
    flow_mapping(unknown_binding, {"some": {"unknown_binding": None}})
    add(make_case("portable-proof-codec/d0-unknown-flow-binding", "ace1-shape", base,
                  ({"op": "set", "path": ["root_export_assertions", "0", "assertion", "flow_mappings"],
                    "value": unknown_binding["root_export_assertions"][0]["assertion"]["flow_mappings"]},),
                  "D0", "UnknownVariant"))
    add(make_case("portable-proof-codec/d0-duplicate-object-key", "ace1-shape", base,
                  expected="D0", error_family="DuplicateField", raw_mutation="duplicate_key"))
    add(make_case("portable-proof-codec/d0-resource-limit", "ace1-shape", base,
                  expected="D0", error_family="ResourceLimit",
                  decoder_limits={"max_container_items": 8},
                  note="A strict local item cap fails closed before returning a partial Bundle."))

    add(make_case("portable-proof-codec/d1-leading-whitespace", "canonical-order", base,
                  expected="D1", error_family="NoncanonicalEncoding", raw_mutation="leading_whitespace"))
    add(make_case("portable-proof-codec/d1-trailing-whitespace", "canonical-order", base,
                  expected="D1", error_family="NoncanonicalEncoding", raw_mutation="trailing_whitespace"))
    add(make_case("portable-proof-codec/d1-unnecessary-escape", "canonical-order", base,
                  expected="D1", error_family="NoncanonicalEncoding", raw_mutation="unnecessary_escape"))
    add(make_case("portable-proof-codec/d1-unsorted-schema-set", "canonical-order", base,
                  ({"op": "reverse", "path": ["schema_assets"]},),
                  "D1", "NoncanonicalOrder"))
    add(make_case("portable-proof-codec/d1-duplicate-set-member", "canonical-order", base,
                  ({"op": "append-copy", "path": ["schema_assets"]},),
                  "D1", "DuplicateSetMember"))
    add(make_case("portable-proof-codec/d1-noncanonical-integer", "canonical-order", base,
                  ({"op": "set", "path": root, "value": "01"},),
                  "D1", "NoncanonicalInteger"))
    add(make_case("portable-proof-codec/d1-non-nfc-text", "canonical-order", base,
                  ({"op": "set", "path": ["compiler_semantics_manifests", "0", "content", "static_identity_projection", "analysis_semantics_version"],
                    "value": "agentflow.root-export-checke\u0301r@1"},),
                  "D1", "NoncanonicalString", raw=True))
    add(make_case("portable-proof-codec/d1-noncanonical-byte-order", "canonical-order", base,
                  ({"op": "set", "path": ["schema_assets"], "value": list(reversed(base["schema_assets"]))},),
                  "D1", "NoncanonicalOrder"))

    add(make_case("portable-proof-codec/d2-dangling-root", "references", base,
                  ({"op": "set", "path": root, "value": "4294967295"},),
                  "D2", "MissingAssertion"))
    add(make_case("portable-proof-codec/d2-dangling-context", "references", base,
                  ({"op": "set", "path": ["root_export_assertions", "0", "assertion", "proof_context_manifest", "context_id"], "value": "99"},),
                  "D2", "MissingContext"))
    add(make_case("portable-proof-codec/d2-dangling-resolver", "references", base,
                  ({"op": "set", "path": ["root_export_assertions", "0", "assertion", "proof_context_manifest", "resolver_id"], "value": "99"},),
                  "D2", "MissingResolver"))
    add(make_case("portable-proof-codec/d2-dangling-compiler", "references", base,
                  ({"op": "set", "path": ["root_export_assertions", "0", "assertion", "compiler_semantics_id"], "value": "99"},),
                  "D2", "MissingCompilerManifest"))
    add(make_case("portable-proof-codec/d2-dangling-schema", "references", base,
                  ({"op": "remove-index", "path": ["schema_assets", "0"]},),
                  "D2", "MissingReferencedContent"))
    add(make_case("portable-proof-codec/d2-wrong-digest-domain", "references", base,
                  ({"op": "set", "path": ["root_export_assertions", "0", "assertion", "exact_semantic_ir_digest", "domain"], "value": "component-contract"},),
                  "D2", "ConflictingAssociation"))
    add(make_case("portable-proof-codec/d2-wrong-schema-domain", "references", base,
                  ({"op": "set", "path": ["schema_assets", "0", "reference", "schema_digest", "domain"], "value": "semantic-ir"},),
                  "D2", "WrongDigestDomain"))
    wrong_ir_schema = next(row["reference"] for row in base["schema_assets"]
                           if row["reference"]["specification_id"] == "agentflow.semantic.ir@1"
                           and row["reference"]["type_name"] == "RunParameterDecl")
    wrong_nominal_mutations = tuple(
        {"op": "set", "path": path, "value": wrong_ir_schema}
        for path in (
            ["ir_assets", "0", "reference", "type_schema"],
            ["root_export_assertions", "0", "ir_asset", "type_schema"],
            ["resolver_dependency_manifests", "0", "content", "caller_ir", "type_schema"],
            ["resolver_dependency_manifests", "0", "content", "source_associations", "0", "caller_ir", "type_schema"],
        )
    )
    add(make_case("portable-proof-codec/d2-wrong-nominal-ir-schema", "references", base,
                  wrong_nominal_mutations, "D2", "InvalidNominalReference"))

    duplicate_target_digest = base["ir_assets"][0]["reference"]["digest"]
    resolver_target_none = {
        "target_semantic_ir_digest": duplicate_target_digest,
        "target_ir_content": {"none": None}, "exported_contract": {"none": None},
        "direct_dependencies": {"none": None}, "transitive_dependencies": {"none": None},
    }
    resolver_target_some = {
        "target_semantic_ir_digest": duplicate_target_digest,
        "target_ir_content": {"some": base["ir_assets"][0]["reference"]},
        "exported_contract": {"none": None},
        "direct_dependencies": {"none": None}, "transitive_dependencies": {"none": None},
    }
    duplicate_targets = sorted([resolver_target_none, resolver_target_some], key=canonical_bytes)
    add(make_case("portable-proof-codec/d2-duplicate-resolver-target-identity", "references", base,
                  ({"op": "set", "path": ["resolver_dependency_manifests", "0", "content", "subflow_targets"],
                    "value": duplicate_targets},), "D2", "DuplicateIdentity"))

    invoke_association = copy.deepcopy(base["resolver_dependency_manifests"][0]["content"]["source_associations"][0])
    await_association = {
        "caller_ir": base["ir_assets"][0]["reference"], "source_node": 1,
        "target": {"await_signal": base["schema_assets"][0]["reference"]},
    }
    duplicate_sources = sorted([invoke_association, await_association], key=canonical_bytes)
    add(make_case("portable-proof-codec/d2-duplicate-source-association", "references", base,
                  ({"op": "set", "path": ["resolver_dependency_manifests", "0", "content", "source_associations"],
                    "value": duplicate_sources},), "D2", "DuplicateIdentity"))

    mismatched_target = {
        "target_semantic_ir_digest": duplicate_target_digest,
        "target_ir_content": {"none": None},
        "exported_contract": {"some": base["contract_assets"][0]["reference"]},
        "direct_dependencies": {"none": None}, "transitive_dependencies": {"none": None},
    }
    mismatched_source = {
        "caller_ir": base["ir_assets"][0]["reference"], "source_node": 1,
        "target": {"call_subflow": {
            "target_semantic_ir_digest": duplicate_target_digest,
            "exported_contract_digest": {"none": None},
        }},
    }
    add(make_case("portable-proof-codec/d2-target-export-association-swap", "references", base,
                  ({"op": "set", "path": ["resolver_dependency_manifests", "0", "content", "subflow_targets"],
                    "value": [mismatched_target]},
                   {"op": "set", "path": ["resolver_dependency_manifests", "0", "content", "source_associations"],
                    "value": [mismatched_source]}), "D2", "MissingResolverTarget"))
    add(make_case("portable-proof-codec/d2-structural-cycle", "references", base,
                  ({"op": "set", "path": ["structural_types"], "value": [{"id": "1", "body": {"list": "1"}}]},
                   {"op": "set", "path": ["root_export_assertions", "0", "assertion", "input_mappings"],
                    "value": [{"export_input_port": {"exported_contract_digest": base["contract_assets"][0]["reference"]["digest"], "port_no": "1"},
                               "run_parameter_id": "1", "root_value_slot": "1", "root_type": "1",
                               "export_type": base["schema_assets"][0]["reference"], "use_sites": []}]}),
                  "D2", "ReferenceCycle"))
    add(make_case("portable-proof-codec/d2-graph-work-resource-limit", "references", base,
                  expected="D2", error_family="ResourceLimit",
                  decoder_limits={"max_graph_work": 0},
                  note="A D2 graph-work cap rejects before returning a reference-validated Bundle."))
    self_edge = {
        "call_site": "1",
        "target_semantic_ir_digest": base["ir_assets"][0]["reference"]["digest"],
        "target_exported_contract_digest": {"none": None},
        "target_root_assertion_id": {"some": "1"},
        "transitive_targets": [],
    }
    add(make_case("portable-proof-codec/d2-assertion-cycle", "references", base,
                  ({"op": "set", "path": ["root_export_assertions", "0", "assertion", "subflow_dependencies"], "value": [self_edge]},),
                  "D2", "ReferenceCycle"))
    add(make_case("portable-proof-codec/d2-unreferenced-asset", "references", base,
                  ({"op": "append-copy", "path": ["contract_assets"]},),
                  "D1", "DuplicateSetMember"))

    import base64
    ir_bytes = base64.urlsafe_b64decode(
        base["ir_assets"][0]["canonical_content"]
        + "=" * ((4 - len(base["ir_assets"][0]["canonical_content"]) % 4) % 4)
    )
    changed_ir = json.loads(ir_bytes)
    changed_ir["root"] = "2"  # Still a canonical SemanticIrDocument; only the old pin is stale.
    add(make_case("portable-proof-codec/d3-corrupt-ir-under-old-pin", "content", base,
                  ({"op": "set", "path": ["ir_assets", "0", "canonical_content"],
                    "value": b64(canonical_bytes(changed_ir))},),
                  "D3", "ContentIdentityMismatch"))
    corrupted_schema = base["schema_assets"][0]["canonical_content"]
    raw_schema = bytearray(base64.urlsafe_b64decode(corrupted_schema + "=" * ((4 - len(corrupted_schema) % 4) % 4)))
    raw_schema[0] ^= 1
    add(make_case("portable-proof-codec/d3-corrupt-schema-under-old-pin", "content", base,
                  ({"op": "set", "path": ["schema_assets", "0", "canonical_content"], "value": b64(raw_schema)},),
                  "D3", "InvalidTypeSchema"))
    add(make_case("portable-proof-codec/d3-source-checksum-mismatch", "content", base,
                  ({"op": "set", "path": ["compiler_semantics_manifests", "0", "content", "normative_sources", "0", "content"], "value": b64(b"changed source")},),
                  "D3", "SourceContentMismatch"))
    add(make_case("portable-proof-codec/d3-source-plain-checksum", "content", base,
                  ({"op": "set", "path": ["compiler_semantics_manifests", "0", "content", "normative_sources", "0", "content_sha256", "hex"], "value": "0"*64},),
                  "D3", "SourceContentMismatch"))
    add(make_case("portable-proof-codec/d3-registry-checksum-mismatch", "content", base,
                  ({"op": "set", "path": ["compiler_semantics_manifests", "0", "content", "registries", "rules", "content_sha256", "hex"], "value": "0"*64},),
                  "D3", "RegistryContentMismatch"))
    add(make_case("portable-proof-codec/d3-plain-not-hashv1", "content", base,
                  ({"op": "set", "path": ["compiler_semantics_manifests", "0", "content", "closed_tables", "abi_numbers", "content_sha256", "hex"], "value": "a"*64},),
                  "D3", "RegistryContentMismatch"))
    bad_relation = {
        "some": {
            "header": {
                "schema": base["schema_assets"][0]["reference"],
                "revision": "v1",
                "canonical_content_checksum": {"algorithm": "sha256", "hex": "0" * 64},
            },
            "canonical_content": b64(b"{}"),
        }
    }
    add(make_case(
        "portable-proof-codec/d3-relation-checksum-mismatch", "content", base,
        ({"op": "set",
          "path": ["proof_contexts", "0", "content", "relation_snapshots", "effect_resource"],
          "value": bad_relation},),
        "D3", "ContentIdentityMismatch"))
    add(make_case("portable-proof-codec/d3-invalid-schema-blob", "content", base,
                  ({"op": "set", "path": ["schema_assets", "0", "canonical_content"], "value": b64(b"{}")},),
                  "D3", "InvalidTypeSchema"))
    add(make_case("portable-proof-codec/d3-closed-table-bytes", "content", base,
                  ({"op": "set", "path": ["compiler_semantics_manifests", "0", "content", "closed_tables", "hash_domains", "content"], "value": b64(b"{}")},),
                  "D3", "RegistryContentMismatch"))

    add(make_case("portable-proof-codec/positive/profile-a-exact", "positive-roundtrip", base, expected="D4",
                  note="Current Profile A, exact 84 applied rule identities."))
    add(make_case("portable-proof-codec/d4-version-label-change", "semantics", base,
                  ({"op": "set", "path": ["compiler_semantics_manifests", "0", "content", "static_identity_projection", "analysis_semantics_version"], "value": "agentflow.old-checker@1"},),
                  "D4", "ConflictingSemanticContent"))
    profile_b = {"rule_complete_attribution": {
        "direct_rules": [], "transitive_rules": [], "dependency_edges": []}}
    add(make_case("portable-proof-codec/d4-profile-b-unsupported", "semantics", base,
                  ({"op": "set", "path": ["compiler_semantics_manifests", "0", "content", "attribution"], "value": profile_b},),
                  "D4", "UnsupportedAttributionProfile"))
    add(make_case("portable-proof-codec/d4-profile-c-unsupported", "semantics", base,
                  ({"op": "set", "path": ["compiler_semantics_manifests", "0", "content", "attribution"], "value": {"portable_certificate": None}},),
                  "D4", "UnsupportedAttributionProfile"))
    add(make_case("portable-proof-codec/d4-missing-runtime-model", "semantics", base,
                  ({"op": "remove-index", "path": ["compiler_semantics_manifests", "0", "content", "semantic_roles", "9"]},),
                  "D4", "MissingSemanticRole"))
    add(make_case("portable-proof-codec/d4-runtime-claimed-defined", "semantics", base,
                  ({"op": "set", "path": ["compiler_semantics_manifests", "0", "content", "semantic_roles", "9", "declaration"], "value": {"defined": {
                      "analysis_semantics_version": "agentflow.root-export-checker@1",
                      "normative_specifications": STATIC_SPECS, "required_schema_pins": [],
                      "depends_on": []}}},),
                  "D4", "UnsupportedSemantics"))
    add(make_case("portable-proof-codec/d4-registry-bytes-conflict", "semantics", base,
                  ({"op": "bump_registry_revision", "path": []},),
                  "D4", "ConflictingSemanticContent"))
    add(make_case("portable-proof-codec/d4-changed-role-dependency", "semantics", base,
                  ({"op": "set", "path": ["compiler_semantics_manifests", "0", "content", "semantic_roles", "3", "declaration", "defined", "depends_on"], "value": []},),
                  "D4", "UnsupportedSemantics"))
    add(make_case("portable-proof-codec/d4-lowering-role-defined", "semantics", base,
                  ({"op": "set", "path": ["compiler_semantics_manifests", "0", "content", "semantic_roles", "8", "declaration"], "value": {"defined": {
                      "analysis_semantics_version": "agentflow.root-export-checker@1",
                      "normative_specifications": STATIC_SPECS, "required_schema_pins": [],
                      "depends_on": []}}},),
                  "D4", "UnsupportedSemantics"))
    add(make_case("portable-proof-codec/d4-rule-complete-old-84-only", "semantics", base,
                  ({"op": "set", "path": ["compiler_semantics_manifests", "0", "content", "attribution"], "value": profile_b},),
                  "D4", "UnsupportedAttributionProfile"))

    add(make_case("portable-proof-codec/positive-none-and-empty-distinct", "positive-roundtrip", base,
                  ({"op": "set", "path": ["resolver_dependency_manifests", "0", "content", "subflow_targets"],
                    "value": [{"target_semantic_ir_digest": {"algorithm":"sha256","domain":"semantic-ir","hex":"1"*64},
                               "target_ir_content":{"none":None},"exported_contract":{"none":None},
                               "direct_dependencies":{"some":[]},"transitive_dependencies":{"some":[]}}]},),
                  expected="D4", note="None remains absence; some-empty direct/transitive lists are known-empty values."))
    add(make_case("portable-proof-codec/positive-id-identity-preserved", "positive-roundtrip", base,
                  expected="D4", note="Portable table IDs are preserved and are local to their declared tables."))
    add(make_case("portable-proof-codec/positive-distinct-local-id-namespaces", "positive-roundtrip", base,
                  ({"op": "set", "path": ["root_assertion_id"], "value": "7"},
                   {"op": "set", "path": ["root_export_assertions", "0", "id"], "value": "7"},
                   {"op": "set", "path": ["proof_contexts", "0", "id"], "value": "11"},
                   {"op": "set", "path": ["resolver_dependency_manifests", "0", "id"], "value": "13"},
                   {"op": "set", "path": ["compiler_semantics_manifests", "0", "id"], "value": "17"},
                   {"op": "set", "path": ["root_export_assertions", "0", "assertion", "proof_context_manifest", "context_id"], "value": "11"},
                   {"op": "set", "path": ["root_export_assertions", "0", "assertion", "proof_context_manifest", "resolver_id"], "value": "13"},
                   {"op": "set", "path": ["root_export_assertions", "0", "assertion", "compiler_semantics_id"], "value": "17"},
                   {"op": "set", "path": ["resolver_dependency_manifests", "0", "content", "context_id"], "value": "11"}),
                  expected="D4", note="Assertion, context, resolver and compiler IDs remain table-local namespaces."))
    add(make_case("portable-proof-codec/d4-host-width-scalar-remains-uint64", "semantics", base,
                  ({"op": "set", "path": ["root_export_assertions", "0", "assertion", "flow_mappings"], "value": [{
                      "source_node":"1","source_contract_digest":base["contract_assets"][0]["reference"]["digest"],
                      "flow_mapping_ordinal":"9007199254740993","source":{"none":None},"sink":{"none":None},
                      "source_binding":{"some":{"unknown_external":None}},"sink_binding":{"none":None},
                      "export_source":{"none":None},"export_sink":{"none":None},
                      "source_effect_partition":{"none":None},"export_effect_partition":{"none":None}}]},),
                  expected="D4", note="Full U64 flow ordinal preserves values above 2^53 without floating-point conversion."))

    for case in cases:
        case["expected_projection_sha256"] = (
            case["input_sha256"]
            if case["expected_stage"] == "D4" and case["expected_error_family"] is None
            else None
        )
    ids = [case["case_id"] for case in cases]
    if len(ids) < 48 or len(set(ids)) != len(ids):
        raise ValueError("portable codec corpus needs 48+ unique CaseIds")
    base_bytes = canonical_bytes(base)
    OUT.mkdir(parents=True, exist_ok=True)
    assert_historical_output(OUT / "full-ten-field-bundle.ace1", base_bytes)
    corpus = {
        "kind": "agentflow.conformance.portable-proof-codec",
        "corpus_revision": 1,
        "case_count": len(cases),
        "base_case_id": "portable-proof-codec/positive/full-ten-field-d4",
        "base_file": "spec/conformance/portable-proof-codec/full-ten-field-bundle.ace1",
        "base_sha256": sha256(base_bytes),
        "required_bundle_fields": [
            "schema_version", "root_assertion_id", "schema_assets", "ir_assets",
            "contract_assets", "structural_types", "proof_contexts",
            "resolver_dependency_manifests", "compiler_semantics_manifests",
            "root_export_assertions",
        ],
        "cases": cases,
    }
    assert_historical_output(OUT / "manifest.json", json.dumps(corpus, ensure_ascii=False, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")
    print(f"portable codec vectors: {len(cases)}; oracle-generated")


if __name__ == "__main__":
    generate()
