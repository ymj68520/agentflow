#!/usr/bin/env python3
"""Independent D5-D8 Portable static reconstruction corpus generator.

Expected context and resolver projections are derived from the Portable wire
declarations and the frozen semantic IR shape in Python. This script never
invokes Rust or reads a Rust-produced actual result.
"""

from __future__ import annotations

import base64
import copy
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys_path = str(ROOT / "tools")
import sys
if sys_path not in sys.path:
    sys.path.insert(0, sys_path)

from portable_proof_python_oracle import (  # noqa: E402
    AFIDL,
    b64url,
    canonical_bytes,
    hash_v1,
    sha256,
)

OUT = ROOT / "spec" / "conformance" / "portable-static-reconstruction"
BASE_PATH = "spec/conformance/portable-proof-codec/full-ten-field-bundle.ace1"
MAX_TARGET_CASES = 159

REQUIRED_CONTEXT_FIELDS = [
    "context_id", "modules", "type_schemas", "effect_catalogs",
    "resource_schemas", "resource_descriptors", "confidentiality_labels",
    "taints", "flow_effect_sinks", "usage_meters", "recovery_effect_catalogs",
    "reconciliation_operations", "execution_capabilities", "trust_domains",
    "regions", "region_compatibility", "runtime_profiles",
    "effect_resource_relation", "recovery_usage_relation", "flow_execution_relation",
]
REQUIRED_PIPELINE_FIELDS = [
    "context", "checked_contracts", "checked_irs", "source_associations",
    "resolver_identity.component_contract_digests", "resolver_identity.subflow_targets",
    "resolver_targets", "await_leaf_count", "await_unknown_external_count",
]


def read_base():
    raw = (ROOT / BASE_PATH).read_bytes()
    return raw, json.loads(raw.decode("utf-8"))


def digest_ref(domain: str, content: bytes):
    return {"algorithm": "sha256", "domain": domain, "hex": hash_v1(domain, content)}


def b64decode(value: str) -> bytes:
    return base64.urlsafe_b64decode(value + "=" * ((4 - len(value) % 4) % 4))


def option_some(value):
    return {"some": value}


def option_none():
    return {"none": None}


def complete(value):
    assert set(value) in ({"complete"}, {"unavailable"})
    return value.get("complete")


def object_hash(value) -> str:
    raw = json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return sha256(raw)


def context_projection(bundle, context_id="1"):
    row = next(row for row in bundle["proof_contexts"] if str(row["id"]) == str(context_id))
    context = row["content"]
    labels = context["flow_catalogs"]
    resource = context["resource_types"]
    recovery = context["recovery_operations"]
    execution = context["execution_universes"]
    relations = context["relation_snapshots"]

    def relation(slot):
        option = relations[slot]
        if "none" in option:
            return None
        item = option["some"]
        header = item["header"]
        return {
            "schema": header["schema"],
            "revision": header["revision"],
            "canonical_content_checksum": header["canonical_content_checksum"],
            "canonical_content": list(b64decode(item["canonical_content"])),
        }

    return {
        "context_id": int(context_id),
        "modules": complete(context["compiled_modules"]),
        "type_schemas": complete(context["type_schemas"]),
        "effect_catalogs": complete(context["effect_catalogs"]),
        "resource_schemas": complete(resource["schemas"]),
        "resource_descriptors": complete(resource["descriptors"]),
        "confidentiality_labels": complete(labels["confidentiality_labels"]),
        "taints": complete(labels["taints"]),
        "flow_effect_sinks": complete(labels["effect_sinks"]),
        "usage_meters": complete(context["usage_meters"]),
        "recovery_effect_catalogs": complete(recovery["effect_catalogs"]),
        "reconciliation_operations": complete(recovery["reconciliation_operations"]),
        "execution_capabilities": complete(execution["capabilities"]),
        "trust_domains": complete(execution["trust_domains"]),
        "regions": complete(execution["regions"]),
        "region_compatibility": complete(execution["region_compatibility"]),
        "runtime_profiles": complete(execution["runtime_profiles"]),
        "effect_resource_relation": relation("effect_resource"),
        "recovery_usage_relation": relation("recovery_usage"),
        "flow_execution_relation": relation("flow_execution"),
    }


def diff_values(before, after, path=()):
    if type(before) is not type(after):
        return [{"op": "set", "path": list(path), "value": after}]
    if isinstance(before, dict):
        if before.keys() != after.keys():
            return [{"op": "set", "path": list(path), "value": after}]
        result = []
        for key in sorted(before):
            result.extend(diff_values(before[key], after[key], path + (key,)))
        return result
    if isinstance(before, list):
        if len(before) != len(after):
            return [{"op": "set", "path": list(path), "value": after}]
        result = []
        for index, (left, right) in enumerate(zip(before, after)):
            result.extend(diff_values(left, right, path + (str(index),)))
        return result
    return [] if before == after else [{"op": "set", "path": list(path), "value": after}]


def apply_patch_model(base, patches):
    value = copy.deepcopy(base)
    for patch in patches:
        current = value
        for token in patch["path"][:-1]:
            current = current[int(token)] if isinstance(current, list) else current[token]
        if not patch["path"]:
            value = copy.deepcopy(patch["value"])
        elif isinstance(current, list):
            current[int(patch["path"][-1])] = copy.deepcopy(patch["value"])
        else:
            current[patch["path"][-1]] = copy.deepcopy(patch["value"])
    return value


def asset_ref(bundle, table, name, type_name):
    for row in bundle[table]:
        ref = row["reference"].get("type_schema") if table != "schema_assets" else row["reference"]
        if ref and ref["specification_id"] == name and ref["type_name"] == type_name:
            return copy.deepcopy(ref)
    raise KeyError((name, type_name))


def schema_reference(bundle, specification_id, type_name):
    return next(
        copy.deepcopy(row["reference"])
        for row in bundle["schema_assets"]
        if row["reference"]["specification_id"] == specification_id
        and row["reference"]["type_name"] == type_name
    )


def find_contract(bundle, digest_hex=None):
    if digest_hex is None:
        return bundle["contract_assets"][0]
    return next(row for row in bundle["contract_assets"] if row["reference"]["digest"]["hex"] == digest_hex)


def find_ir(bundle, digest_hex=None):
    if digest_hex is None:
        return bundle["ir_assets"][0]
    return next(row for row in bundle["ir_assets"] if row["reference"]["digest"]["hex"] == digest_hex)


def replace_exact(value, old, new):
    if value == old:
        return copy.deepcopy(new)
    if isinstance(value, list):
        return [replace_exact(item, old, new) for item in value]
    if isinstance(value, dict):
        return {key: replace_exact(item, old, new) for key, item in value.items()}
    return value


def canonical_asset(bundle, table, model, domain, schema_id, root_type):
    content = canonical_bytes(model)
    type_ref = schema_reference(bundle, schema_id, root_type)
    ref = {"digest": digest_ref(domain, content), "type_schema": type_ref}
    return {"reference": ref, "canonical_content": b64url(content)}


def install_contract(bundle, contract):
    old_asset = find_contract(bundle)
    old_ref = copy.deepcopy(old_asset["reference"])
    old_digest = copy.deepcopy(old_ref["digest"])
    new_asset = canonical_asset(
        bundle,
        "contract_assets",
        contract,
        "component-contract",
        "agentflow.component.contract@1",
        "ComponentContract",
    )
    new_ref = copy.deepcopy(new_asset["reference"])
    bundle = replace_exact(bundle, old_ref, new_ref)
    bundle = replace_exact(bundle, old_digest, new_ref["digest"])
    asset = find_contract(bundle, new_ref["digest"]["hex"])
    asset["canonical_content"] = new_asset["canonical_content"]

    # The caller IR contract table pins the checked contract content.
    old_ir = find_ir(bundle)
    old_ir_ref = copy.deepcopy(old_ir["reference"])
    old_ir_digest = copy.deepcopy(old_ir_ref["digest"])
    ir = json.loads(b64decode(old_ir["canonical_content"]).decode("utf-8"))
    for entry in ir["contracts"]:
        if entry["kind"] == "component" and entry["digest"] == old_digest:
            entry["digest"] = copy.deepcopy(new_ref["digest"])
    new_ir_asset = canonical_asset(
        bundle,
        "ir_assets",
        ir,
        "semantic-ir",
        "agentflow.semantic.ir@1",
        "SemanticIrDocument",
    )
    new_ir_ref = copy.deepcopy(new_ir_asset["reference"])
    bundle = replace_exact(bundle, old_ir_ref, new_ir_ref)
    bundle = replace_exact(bundle, old_ir_digest, new_ir_ref["digest"])
    find_ir(bundle, new_ir_ref["digest"]["hex"])["canonical_content"] = new_ir_asset["canonical_content"]
    return bundle, new_ref


def install_ir(bundle, ir):
    ir = prepare_ir_machine_value(ir)
    old_asset = find_ir(bundle)
    old_ref = copy.deepcopy(old_asset["reference"])
    old_digest = copy.deepcopy(old_ref["digest"])
    new_asset = canonical_asset(
        bundle,
        "ir_assets",
        ir,
        "semantic-ir",
        "agentflow.semantic.ir@1",
        "SemanticIrDocument",
    )
    new_ref = copy.deepcopy(new_asset["reference"])
    bundle = replace_exact(bundle, old_ref, new_ref)
    bundle = replace_exact(bundle, old_digest, new_ref["digest"])
    find_ir(bundle, new_ref["digest"]["hex"])["canonical_content"] = new_asset["canonical_content"]
    return bundle, new_ref


def prepare_ir_machine_value(ir):
    """Convert serde Option fields into the registered tagged ACE1 Option."""
    result = copy.deepcopy(ir)
    for computation in result.get("computations", []):
        body = computation.get("body", {})
        if "parallel" in body:
            parallel = body["parallel"]
            value = parallel.get("max_inflight")
            parallel["max_inflight"] = option_none() if value is None else option_some(str(value))
        if "iterate" in body:
            iteration = body["iterate"]
            if "foreach" in iteration:
                foreach = iteration["foreach"]
                value = foreach.get("max_inflight")
                foreach["max_inflight"] = option_none() if value is None else option_some(str(value))
    return result


def empty_contract(name="portable.codec"):
    value = {
        "component_identity": {"catalog": "agentflow.portable.fixture@1", "name": name},
        "inputs": [], "outputs": [],
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
            "capabilities": [], "quantities": [], "sandbox": {"none": None},
            "trust_domain": {"none": None}, "region_residency": {"none": None},
            "runtime_profile": {"none": None}, "implementation_abis": [], "soft_preferences": [],
        },
        "effect_flow_partitions": [], "recovery_action_bindings": [], "recovery_usage_bindings": [],
    }
    return value


def effect_atom(catalog="agentflow.effects.test@1", name="write"):
    return {
        "operation": {"catalog": catalog, "name": name},
        "resource_selector": {"requirement_port_no": "1", "scope_bindings": []},
        "qualifiers": [],
    }


def effect_entry(atom=None):
    return {
        "atom": atom or effect_atom(),
        "bounds": {"lower": "1", "upper": {"finite": "1"}},
        "guard": {"unconditional": None},
    }


def add_context_catalog(bundle, recovery=False, operations=None):
    target = bundle["proof_contexts"][0]["content"]
    table = target["recovery_operations"]["effect_catalogs"] if recovery else target["effect_catalogs"]
    table["complete"].append({
        "specification_id": "agentflow.effects.test@1",
        "operations": operations or ["write"],
    })
    table["complete"].sort(key=lambda row: row["specification_id"].encode("utf-8"))


def install_relation(bundle, slot, schema_id, root_type, content_value, revision="rev-1"):
    content = canonical_bytes(content_value)
    relation = {
        "header": {
            "schema": schema_reference(bundle, schema_id, root_type),
            "revision": revision,
            "canonical_content_checksum": {"algorithm": "sha256", "hex": sha256(content)},
        },
        "canonical_content": b64url(content),
    }
    bundle["proof_contexts"][0]["content"]["relation_snapshots"][slot] = {"some": relation}


def minimal_ir(body, contracts, semantic_refs=None, computations=None):
    return {
        "semantic_refs": semantic_refs or [],
        "types": [], "values": [], "expressions": [], "resources": [],
        "contracts": contracts,
        "run_parameters": [], "workflow_outputs": [],
        "computations": computations or [{"id": "1", "outputs": [], "body": body}],
        "root": "1",
    }


def set_full_resolver(bundle, caller_ir, targets, components, associations, export_ref=None, build_transitive=True):
    # Replace the existing root IR asset, then append exact, independently
    # materialized target assets.
    old_caller = copy.deepcopy(bundle["resolver_dependency_manifests"][0]["content"]["caller_ir"])
    caller_asset = canonical_asset(
        bundle, "ir_assets", caller_ir, "semantic-ir", "agentflow.semantic.ir@1", "SemanticIrDocument"
    )
    caller_ref = caller_asset["reference"]
    bundle = replace_exact(bundle, old_caller, caller_ref)
    existing_ir = find_ir(bundle, caller_ref["digest"]["hex"])
    existing_ir["canonical_content"] = caller_asset["canonical_content"]

    target_specs = []
    for target_ir, target_export in targets:
        target_asset = canonical_asset(
            bundle, "ir_assets", target_ir, "semantic-ir", "agentflow.semantic.ir@1", "SemanticIrDocument"
        )
        target_ref = target_asset["reference"]
        bundle["ir_assets"].append(target_asset)
        deps = sorted({entry["digest"]["hex"]: entry["digest"] for entry in target_ir["contracts"] if entry["kind"] == "subflow"}.values(), key=lambda item: (item["algorithm"], item["domain"], item["hex"]))
        target_specs.append({
            "target_semantic_ir_digest": copy.deepcopy(target_ref["digest"]),
            "target_ir_content": copy.deepcopy(target_ref),
            "exported_contract": copy.deepcopy(target_export),
            "direct_dependencies": deps,
        })

    target_specs.sort(key=lambda row: (row["target_semantic_ir_digest"]["algorithm"], row["target_semantic_ir_digest"]["domain"], row["target_semantic_ir_digest"]["hex"]))
    # Fill transitive closures from the independently declared target IR graph.
    rows_by_digest = {row["target_semantic_ir_digest"]["hex"]: row for row in target_specs}
    target_rows = []
    for row in target_specs:
        closure = set()
        if build_transitive:
            pending = [digest["hex"] for digest in row["direct_dependencies"]]
            while pending:
                digest_hex = pending.pop()
                if digest_hex in closure:
                    continue
                closure.add(digest_hex)
                if digest_hex in rows_by_digest:
                    pending.extend(item["hex"] for item in rows_by_digest[digest_hex]["direct_dependencies"])
        transitive = [
            {
                "semantic_ir_digest": copy.deepcopy(rows_by_digest[item]["target_semantic_ir_digest"]),
                "exported_contract_digest": option_some(copy.deepcopy(rows_by_digest[item]["exported_contract"]["digest"])) if rows_by_digest[item]["exported_contract"] else option_none(),
            }
            for item in sorted(closure)
            if item in rows_by_digest
        ]
        transitive.sort(key=lambda item: (item["semantic_ir_digest"]["algorithm"], item["semantic_ir_digest"]["domain"], item["semantic_ir_digest"]["hex"]))
        target_rows.append({
            "target_semantic_ir_digest": row["target_semantic_ir_digest"],
            "target_ir_content": option_some(row["target_ir_content"]),
            "exported_contract": option_some(row["exported_contract"]) if row["exported_contract"] else option_none(),
            "direct_dependencies": option_some(row["direct_dependencies"]),
            "transitive_dependencies": option_some(transitive),
        })

    components = sorted(components, key=lambda item: (item["digest"]["algorithm"], item["digest"]["domain"], item["digest"]["hex"]))
    associations = [{
        "caller_ir": copy.deepcopy(caller_ref),
        "source_node": item["source_node"],
        "target": copy.deepcopy(item["target"]),
    } for item in associations]
    associations.sort(key=lambda item: (item["source_node"], json.dumps(item["target"], sort_keys=True)))
    bundle["resolver_dependency_manifests"] = [{
        "id": "1",
        "content": {
            "caller_ir": copy.deepcopy(caller_ref), "context_id": "1",
            "components": copy.deepcopy(components),
            "subflow_targets": target_rows,
            "source_associations": associations,
        },
    }]
    bundle["root_export_assertions"][0]["ir_asset"] = copy.deepcopy(caller_ref)
    bundle["root_export_assertions"][0]["assertion"]["exact_semantic_ir_digest"] = copy.deepcopy(caller_ref["digest"])
    if export_ref:
        bundle["root_export_assertions"][0]["export_contract_asset"] = copy.deepcopy(export_ref)
        bundle["root_export_assertions"][0]["assertion"]["exact_exported_component_contract_digest"] = copy.deepcopy(export_ref["digest"])
    bundle["contract_assets"].sort(key=lambda row: (row["reference"]["digest"]["algorithm"], row["reference"]["digest"]["domain"], row["reference"]["digest"]["hex"]))
    bundle["ir_assets"].sort(key=lambda row: (row["reference"]["digest"]["algorithm"], row["reference"]["digest"]["domain"], row["reference"]["digest"]["hex"]))
    return bundle, caller_ref, target_rows


def projection_output(bundle):
    resolver = bundle["resolver_dependency_manifests"][0]["content"]
    context = context_projection(bundle, resolver["context_id"])
    components = resolver["components"]
    contract_map = {row["reference"]["digest"]["hex"]: row for row in bundle["contract_assets"]}
    checked_contracts = []
    all_contract_refs = {ref["digest"]["hex"]: ref for ref in components}
    for target in resolver["subflow_targets"]:
        ref = target["exported_contract"].get("some")
        if ref:
            all_contract_refs[ref["digest"]["hex"]] = ref
    for digest_hex in sorted(all_contract_refs, key=lambda h: ("sha256", "component-contract", h)):
        ref = all_contract_refs[digest_hex]
        raw = b64decode(contract_map[digest_hex]["canonical_content"])
        checked_contracts.append({
            "reference": ref["digest"],
            "canonical_content_sha256": sha256(raw),
            "proven_dimension_count": 10,
            "all_dimensions_proven": True,
            "action_guard_compatibility_count": 0,
        })

    ir_map = {row["reference"]["digest"]["hex"]: row for row in bundle["ir_assets"]}
    ir_refs = {resolver["caller_ir"]["digest"]["hex"]: resolver["caller_ir"]}
    for target in resolver["subflow_targets"]:
        target_ir = target["target_ir_content"].get("some")
        if target_ir:
            ir_refs[target_ir["digest"]["hex"]] = target_ir
    checked_irs = []
    for digest_hex in sorted(ir_refs, key=lambda h: ("sha256", "semantic-ir", h)):
        ref = ir_refs[digest_hex]
        ir = json.loads(b64decode(ir_map[digest_hex]["canonical_content"]).decode("utf-8"))
        raw = b64decode(ir_map[digest_hex]["canonical_content"])
        comps = len(ir["computations"])
        checked_irs.append({
            "reference": ref["digest"],
            "canonical_content_sha256": sha256(raw),
            "computation_count": comps,
            "phase3c_computation_entry_count": comps,
            "phase3c_entry_value_count": 0,
            "phase3c_entry_resource_count": 0,
            "phase3c_producer_count": 0,
            "phase3c_workflow_exit_value_count": 0,
        })

    target_by_hex = {row["target_semantic_ir_digest"]["hex"]: row for row in resolver["subflow_targets"]}
    resolver_targets = []
    for row in resolver["subflow_targets"]:
        export_ref = row["exported_contract"].get("some")
        direct = row["direct_dependencies"].get("some")
        transitive = row["transitive_dependencies"].get("some")
        resolver_targets.append({
            "semantic_ir_digest": row["target_semantic_ir_digest"],
            "exported_contract_digest": export_ref["digest"] if export_ref else None,
            "direct_dependencies": direct,
            "transitive_dependencies": [
                {
                    "semantic_ir_digest": item["semantic_ir_digest"],
                    "exported_contract_digest": item["exported_contract_digest"].get("some"),
                }
                for item in transitive
            ],
        })
    resolver_targets.sort(key=lambda row: (row["semantic_ir_digest"]["algorithm"], row["semantic_ir_digest"]["domain"], row["semantic_ir_digest"]["hex"]))

    caller_ir = json.loads(b64decode(ir_map[resolver["caller_ir"]["digest"]["hex"]]["canonical_content"]).decode("utf-8"))
    source_associations = []
    await_count = 0
    for computation in caller_ir["computations"]:
        body = computation["body"]
        if "invoke" in body:
            entry = next(row for row in caller_ir["contracts"] if str(row["id"]) == str(body["invoke"]["contract"]))
            ref = next(ref for ref in components if ref["digest"] == entry["digest"])
            target = {"invoke": ref}
        elif "call_subflow" in body:
            entry = next(row for row in caller_ir["contracts"] if str(row["id"]) == str(body["call_subflow"]["contract"]))
            target_row = target_by_hex[entry["digest"]["hex"]]
            export_ref = target_row["exported_contract"].get("some")
            target = {"call_subflow": {
                "semantic_ir_digest": entry["digest"],
                "exported_contract_digest": export_ref["digest"] if export_ref else None,
            }}
        elif "await" in body:
            target = {"await_signal": body["await"]["signal_type"]}
            await_count += 1
        else:
            continue
        source_associations.append({
            "caller_ir": resolver["caller_ir"]["digest"],
            "source_node": int(computation["id"]),
            "target": target,
        })
    source_associations.sort(key=lambda item: (item["caller_ir"]["algorithm"], item["caller_ir"]["domain"], item["caller_ir"]["hex"], item["source_node"], json.dumps(item["target"], sort_keys=True)))
    return {
        "context": context,
        "checked_contracts": checked_contracts,
        "checked_irs": checked_irs,
        "source_associations": source_associations,
        "resolver_identity": {
            "component_contract_digests": [ref["digest"] for ref in components],
            "subflow_targets": [{
                "semantic_ir_digest": row["target_semantic_ir_digest"],
                "direct_dependencies": row["direct_dependencies"].get("some"),
                "exported_contract_digest": row["exported_contract"].get("some", {}).get("digest"),
            } for row in resolver["subflow_targets"]],
        },
        "resolver_targets": resolver_targets,
        "await_leaf_count": await_count,
        "await_unknown_external_count": await_count,
    }


def add_case(cases, base, case_id, group, entry, expected_stage, expected_family=None, **extra):
    variant = copy.deepcopy(base)
    make_variant = extra.pop("mutate", None)
    if make_variant:
        variant = make_variant(variant)
    mutations = diff_values(base, variant)
    replayed = apply_patch_model(base, mutations)
    if replayed != variant:
        raise AssertionError(f"patch projection does not reproduce {case_id}")
    actual_bytes = canonical_bytes(replayed)
    context_projection_expected = None
    pipeline_projection_expected = None
    d5_hash = None
    pipeline_hash = None
    if expected_stage == "D5" and expected_family is None:
        context_projection_expected = context_projection(variant)
        d5_hash = object_hash(context_projection_expected)
    if expected_stage not in ("D0", "D1", "D2", "D3", "D4", "D5"):
        context_projection_expected = context_projection(variant)
        d5_hash = object_hash(context_projection_expected)
    if expected_stage == "D8" and expected_family is None:
        pipeline_projection_expected = projection_output(variant)
        pipeline_hash = object_hash(pipeline_projection_expected)
    required_fields = extra.pop("required_compared_fields", None)
    if required_fields is None:
        if d5_hash is None:
            required_fields = ["actual_stage", "failure_family"]
        else:
            required_fields = REQUIRED_CONTEXT_FIELDS.copy()
            if pipeline_hash is not None:
                required_fields.extend(REQUIRED_PIPELINE_FIELDS)
            elif entry == "pipeline":
                required_fields.extend(["actual_stage", "failure_family"])
    cases.append({
        "case_id": case_id,
        "group": group,
        "entry": entry,
        "mutations": mutations,
        "input_byte_length": len(actual_bytes),
        "input_sha256": sha256(actual_bytes),
        "expected_stage": expected_stage,
        "expected_error_family": expected_family,
        "expected_context_projection_sha256": d5_hash,
        "expected_pipeline_projection_sha256": pipeline_hash,
        "expected_context_projection": context_projection_expected,
        "expected_pipeline_projection": pipeline_projection_expected,
        "expected_context_identity_relation": extra.pop("context_identity_relation", "not-compared"),
        "required_compared_fields": required_fields,
        **extra,
    })


def d5_cases(cases, base):
    add_case(cases, base, "portable-static-reconstruction/d5-01-complete-context", "D5", "d5", "D5")

    def effect_one(bundle):
        bundle["proof_contexts"][0]["content"]["effect_catalogs"]["complete"].append({"specification_id": "agentflow.effects.test@1", "operations": ["audit.read"]})
        bundle["proof_contexts"][0]["content"]["effect_catalogs"]["complete"].sort(key=lambda row: row["specification_id"])
        return bundle
    add_case(cases, base, "portable-static-reconstruction/d5-02-preserve-unused-effect-catalog-member", "D5", "d5", "D5", mutate=effect_one)

    def effect_changed(bundle):
        bundle = effect_one(bundle)
        bundle["proof_contexts"][0]["content"]["effect_catalogs"]["complete"][0]["operations"].append("audit.write")
        bundle["proof_contexts"][0]["content"]["effect_catalogs"]["complete"][0]["operations"].sort()
        return bundle
    add_case(cases, base, "portable-static-reconstruction/d5-03-changed-catalog-member-set", "D5", "d5", "D5", mutate=effect_changed)

    def unavailable(bundle):
        bundle["proof_contexts"][0]["content"]["usage_meters"] = {"unavailable": "not_supplied"}
        return bundle
    add_case(cases, base, "portable-static-reconstruction/d5-04-unavailable-not-empty", "D5", "d5", "D5", "IncompleteContext", mutate=unavailable)

    def private_schema(bundle):
        ref = schema_reference(bundle, "agentflow.portable.proof.inputs@1", "PortableProofInputBundle")
        bundle["proof_contexts"][0]["content"]["type_schemas"]["complete"] = [item for item in bundle["proof_contexts"][0]["content"]["type_schemas"]["complete"] if item != ref]
        bundle["proof_contexts"][0]["content"]["resource_types"]["schemas"]["complete"] = [ref]
        bundle["proof_contexts"][0]["content"]["resource_types"]["descriptors"]["complete"] = [{
            "schema": ref,
            "recognized_attributes": ["bucket", "region"],
        }]
        return bundle
    add_case(cases, base, "portable-static-reconstruction/d5-05-resource-private-schema-scope", "D5", "d5", "D5", mutate=private_schema)

    def global_subset(bundle):
        ref = schema_reference(bundle, "agentflow.portable.proof.inputs@1", "PortableProofInputBundle")
        bundle["proof_contexts"][0]["content"]["type_schemas"]["complete"].remove(ref)
        return bundle
    add_case(cases, base, "portable-static-reconstruction/d5-06-global-schema-membership-exact", "D5", "d5", "D5", mutate=global_subset)

    def recovery_effect(bundle):
        add_context_catalog(bundle, recovery=True, operations=["reconcile.read", "reconcile.write"])
        return bundle
    add_case(cases, base, "portable-static-reconstruction/d5-07-recovery-effect-catalog-independent", "D5", "d5", "D5", mutate=recovery_effect)

    def reconciliation(bundle):
        rows = bundle["proof_contexts"][0]["content"]["recovery_operations"]["reconciliation_operations"]["complete"]
        rows.append({"catalog": "agentflow.recovery.test@1", "name": "reconcile"})
        rows.sort(key=lambda row: (row["catalog"], row["name"]))
        return bundle
    add_case(cases, base, "portable-static-reconstruction/d5-08-reconciliation-operation-kept", "D5", "d5", "D5", mutate=reconciliation)

    def exact_version(bundle):
        caps = bundle["proof_contexts"][0]["content"]["execution_universes"]["capabilities"]["complete"]
        caps.append({
            "capability": {"catalog": "agentflow.executor.test@1", "name": "gpu"},
            "exact_version": "v2", "supported_versions": ["v1", "v2"],
            "attributes": [{"key": "arch", "value": "x64"}],
        })
        caps.sort(key=lambda row: (row["capability"]["catalog"], row["capability"]["name"]))
        return bundle
    add_case(cases, base, "portable-static-reconstruction/d5-09-execution-exact-version-v2", "D5", "d5", "D5", mutate=exact_version)

    def directed_regions(bundle):
        execution = bundle["proof_contexts"][0]["content"]["execution_universes"]
        execution["regions"]["complete"] = [
            {"catalog": "agentflow.region.test@1", "name": "a"},
            {"catalog": "agentflow.region.test@1", "name": "b"},
        ]
        execution["region_compatibility"]["complete"] = [{
            "from": {"catalog": "agentflow.region.test@1", "name": "a"},
            "to": {"catalog": "agentflow.region.test@1", "name": "b"},
        }]
        return bundle
    add_case(cases, base, "portable-static-reconstruction/d5-10-directed-region-pair", "D5", "d5", "D5", mutate=directed_regions)

    def unknown_taint(bundle):
        bundle["proof_contexts"][0]["content"]["flow_catalogs"]["taints"]["complete"] = [
            {"catalog": "agentflow.flow.test@1", "name": "external.unclassified"},
        ]
        return bundle
    add_case(cases, base, "portable-static-reconstruction/d5-11-flow-taint-membership-preserved", "D5", "d5", "D5", mutate=unknown_taint)

    def empty_relation(bundle):
        install_relation(bundle, "effect_resource", "agentflow.effect.resource.compatibility@1", "EffectResourceCompatibilityCatalog", {"complete_operations": [], "supported_pairs": []})
        return bundle
    add_case(cases, base, "portable-static-reconstruction/d5-12-some-empty-relation-retained", "D5", "d5", "D5", mutate=empty_relation)

    def duplicate_relation(bundle):
        atom = effect_atom()
        action = {"forward_effect": atom, "action_kind": "compensation", "action_effect": atom}
        content = {"rules": [{"action": action, "complete_meters": []}, {"action": action, "complete_meters": []}]}
        install_relation(bundle, "recovery_usage", "agentflow.recovery.usage.coverage@1", "RecoveryUsageCoverageCatalog", content)
        return bundle
    add_case(cases, base, "portable-static-reconstruction/d5-13-invalid-typed-relation-rejected", "D5", "d5", "D5", "ReconstructionMismatch", mutate=duplicate_relation)

    def flow_sinks(bundle):
        bundle["proof_contexts"][0]["content"]["flow_catalogs"]["effect_sinks"]["complete"] = [
            {"operation": {"catalog": "agentflow.effects.test@1", "name": "decision.route"}, "decision_capable": True},
        ]
        return bundle
    add_case(cases, base, "portable-static-reconstruction/d5-14-decision-capability-explicit", "D5", "d5", "D5", mutate=flow_sinks)

    add_case(cases, base, "portable-static-reconstruction/d5-15-equal-content-fresh-contexts", "D5", "d5", "D5", context_identity_relation="distinct_for_equal_content")

    def module_budget(bundle):
        rows = bundle["proof_contexts"][0]["content"]["compiled_modules"]["complete"]
        for index in range(241):
            rows.append({"specification_id": f"agentflow.extra{index:03d}@1", "named_types": []})
        rows.sort(key=lambda row: row["specification_id"].encode("utf-8"))
        return bundle
    add_case(cases, base, "portable-static-reconstruction/d5-16-module-work-limit", "D5", "d5", "D5", "ResourceLimit", mutate=module_budget)


def d6_cases(cases, base):
    # The two accepted cases prove that both resolver source and export
    # references pass through a fresh formal check_contract call.
    add_case(cases, base, "portable-static-reconstruction/d6-01-valid-exported-contract", "D6", "pipeline", "D8")
    add_case(cases, base, "portable-static-reconstruction/d6-02-valid-source-contract", "D6", "pipeline", "D8")

    def invalid_component_identity(bundle):
        contract = empty_contract()
        contract["component_identity"]["catalog"] = "not-a-spec-id"
        return install_contract(bundle, contract)[0]
    add_case(cases, base, "portable-static-reconstruction/d6-03-invalid-component-identity", "D6", "pipeline", "D6", "ContractValidationFailed", mutate=invalid_component_identity)

    ghost = schema_reference(base, "agentflow.portable.proof.inputs@1", "PortableProofInputBundle")
    ghost["type_name"] = "GhostType"

    def missing_input_schema(bundle):
        contract = empty_contract()
        contract["inputs"] = [{"port_no": "1", "type": ghost}]
        return install_contract(bundle, contract)[0]
    add_case(cases, base, "portable-static-reconstruction/d6-04-unknown-input-schema", "D6", "pipeline", "D6", "ContractValidationFailed", mutate=missing_input_schema)

    def unknown_effect(bundle):
        contract = empty_contract()
        contract["effects"] = {"row": {"closed": [effect_entry(effect_atom("agentflow.missing@1", "unknown.write"))]}}
        return install_contract(bundle, contract)[0]
    add_case(cases, base, "portable-static-reconstruction/d6-05-unresolved-effect-operation", "D6", "pipeline", "D6", "ContractValidationFailed", mutate=unknown_effect)

    def wrong_resource(bundle):
        contract = empty_contract()
        contract["resources"]["requirements"] = [{"port_no": "1", "resource_type": {"schema": ghost}, "required_attributes": []}]
        return install_contract(bundle, contract)[0]
    add_case(cases, base, "portable-static-reconstruction/d6-06-unrecognized-resource-type", "D6", "pipeline", "D6", "ContractValidationFailed", mutate=wrong_resource)

    def unknown_flow_label(bundle):
        contract = empty_contract()
        contract["information_flow"]["confidentiality_restrictions"] = [{"catalog": "agentflow.missing@1", "name": "secret"}]
        return install_contract(bundle, contract)[0]
    add_case(cases, base, "portable-static-reconstruction/d6-07-unresolved-confidentiality-label", "D6", "pipeline", "D6", "ContractValidationFailed", mutate=unknown_flow_label)

    def unknown_meter(bundle):
        contract = empty_contract()
        contract["usage"]["limits"] = [{"meter": {"catalog": "agentflow.missing@1", "name": "tokens"}, "phase": "forward", "upper": {"finite": "3"}}]
        return install_contract(bundle, contract)[0]
    add_case(cases, base, "portable-static-reconstruction/d6-08-unresolved-active-meter", "D6", "pipeline", "D6", "ContractValidationFailed", mutate=unknown_meter)

    def unknown_capability(bundle):
        contract = empty_contract()
        contract["execution"]["capabilities"] = [{"capability": {"catalog": "agentflow.missing@1", "name": "gpu"}, "exact_version": "v1", "required_attributes": []}]
        return install_contract(bundle, contract)[0]
    add_case(cases, base, "portable-static-reconstruction/d6-09-unresolved-execution-capability", "D6", "pipeline", "D6", "ContractValidationFailed", mutate=unknown_capability)

    def flow_port_mismatch(bundle):
        contract = empty_contract()
        contract["information_flow"]["summary"]["sources"] = [{"category": {"workflow_input": "99"}}]
        return install_contract(bundle, contract)[0]
    add_case(cases, base, "portable-static-reconstruction/d6-10-flow-coordinate-counterexample", "D6", "pipeline", "D6", "ContractValidationFailed", mutate=flow_port_mismatch)

    def missing_effect_relation(bundle):
        add_context_catalog(bundle, operations=["write"])
        contract = empty_contract()
        contract["effects"] = {"row": {"closed": [effect_entry()]}}
        return install_contract(bundle, contract)[0]
    add_case(cases, base, "portable-static-reconstruction/d6-11-missing-effect-resource-snapshot", "D6", "pipeline", "D6", "ContractValidationFailed", mutate=missing_effect_relation)

    def failed_check_has_no_output(bundle):
        schema = {
            "schema": "agentflow.guard.fixture@1",
            "name": "GuardInt64",
            "body": {"alias": {"primitive": "int64"}},
        }
        schema_bytes = canonical_bytes(schema)
        schema_ref = {
            "specification_id": schema["schema"],
            "type_name": schema["name"],
            "schema_digest": {"algorithm": "sha256", "domain": "type-schema", "hex": hash_v1("type-schema", schema_bytes)},
        }
        bundle["schema_assets"].append({"reference": schema_ref, "canonical_content": b64url(schema_bytes)})
        bundle["schema_assets"].sort(key=lambda row: (row["reference"]["specification_id"], row["reference"]["type_name"]))
        type_refs = bundle["proof_contexts"][0]["content"]["type_schemas"]["complete"]
        type_refs.append(copy.deepcopy(schema_ref))
        type_refs.sort(key=lambda ref: (ref["specification_id"], ref["type_name"], ref["schema_digest"]["algorithm"], ref["schema_digest"]["domain"], ref["schema_digest"]["hex"]))
        add_context_catalog(bundle, operations=["compensate", "forward"])
        add_context_catalog(bundle, recovery=True, operations=["compensate", "forward"])
        contract = empty_contract()
        contract["inputs"] = [{"port_no": "1", "type": copy.deepcopy(schema_ref)}]
        forward = effect_atom(name="forward")
        compensate = effect_atom(name="compensate")
        contract["effects"] = {"row": {"closed": [
            {
                "atom": compensate,
                "bounds": {"lower": "1", "upper": {"finite": "1"}},
                "guard": {"input_equals": {"port_no": "1", "expected": {"int64": "0"}}},
            },
            effect_entry(forward),
        ]}}
        contract["recovery"] = {
            "replay_semantics": "recompute",
            "effect_recovery_clauses": [{
                "effect": forward,
                "idempotency": "natural",
                "key_requirement": option_none(),
                "reconciliation": "none",
                "compensation": {"required": {"catalog": "agentflow.effects.test@1", "name": "compensate"}},
                "unknown_effect_handling": "block",
            }],
        }
        contract["recovery_action_bindings"] = [{"identity": {
            "forward_effect": forward,
            "action_kind": "compensation",
            "action_effect": compensate,
        }}]
        action_identity = {
            "forward_effect": copy.deepcopy(forward),
            "action_kind": "compensation",
            "action_effect": copy.deepcopy(compensate),
        }
        contract["recovery_usage_bindings"] = [{
            "action": copy.deepcopy(action_identity),
            "activations_per_forward": {"lower": "1", "upper": {"finite": "1"}},
            "attempts_per_activation": {"lower": "1", "upper": {"finite": "2"}},
            "per_attempt_usage": {"limits": []},
        }]
        install_relation(bundle, "recovery_usage", "agentflow.recovery.usage.coverage@1", "RecoveryUsageCoverageCatalog", {
            "rules": [{"action": action_identity, "complete_meters": []}],
        })
        return install_contract(bundle, contract)[0]
    add_case(cases, base, "portable-static-reconstruction/d6-12-action-guard-counterexample-no-partial-checked", "D6", "pipeline", "D6", "ContractValidationFailed", mutate=failed_check_has_no_output)

    def recovery_coverage_mismatch(bundle):
        bundle = failed_check_has_no_output(bundle)
        relation = bundle["proof_contexts"][0]["content"]["relation_snapshots"]["recovery_usage"]["some"]
        catalog = json.loads(b64decode(relation["canonical_content"]).decode("utf-8"))
        catalog["rules"][0]["complete_meters"] = [{"catalog": "agentflow.meters.test@1", "name": "api_calls"}]
        content = canonical_bytes(catalog)
        relation["canonical_content"] = b64url(content)
        relation["header"]["canonical_content_checksum"]["hex"] = sha256(content)
        return bundle
    add_case(cases, base, "portable-static-reconstruction/d6-13-recovery-usage-coverage-mismatch", "D6", "pipeline", "D6", "ContractValidationFailed", mutate=recovery_coverage_mismatch)

    def invalid_effect_resource_relation(bundle):
        add_context_catalog(bundle, operations=["write"])
        schema_ref = schema_reference(bundle, "agentflow.portable.proof.inputs@1", "PortableProofInputBundle")
        resource = bundle["proof_contexts"][0]["content"]["resource_types"]
        resource["schemas"]["complete"] = [copy.deepcopy(schema_ref)]
        resource["descriptors"]["complete"] = [{"schema": copy.deepcopy(schema_ref), "recognized_attributes": []}]
        operation = {"catalog": "agentflow.effects.test@1", "name": "write"}
        install_relation(bundle, "effect_resource", "agentflow.effect.resource.compatibility@1", "EffectResourceCompatibilityCatalog", {
            "complete_operations": [operation],
            "supported_pairs": [],
        })
        contract = empty_contract()
        contract["effects"] = {"row": {"closed": [effect_entry(effect_atom())]}}
        contract["resources"]["requirements"] = [{
            "port_no": "1",
            "resource_type": {"schema": copy.deepcopy(schema_ref)},
            "required_attributes": [],
        }]
        return install_contract(bundle, contract)[0]
    add_case(cases, base, "portable-static-reconstruction/d6-14-effect-resource-relation-counterexample", "D6", "pipeline", "D6", "ContractValidationFailed", mutate=invalid_effect_resource_relation)

    def invalid_flow_execution_relation(bundle):
        label = {"catalog": "agentflow.flow.test@1", "name": "restricted"}
        flow = bundle["proof_contexts"][0]["content"]["flow_catalogs"]
        flow["confidentiality_labels"]["complete"] = [copy.deepcopy(label)]
        execution = bundle["proof_contexts"][0]["content"]["execution_universes"]
        execution["regions"]["complete"] = [
            {"catalog": "agentflow.region.test@1", "name": "a"},
            {"catalog": "agentflow.region.test@1", "name": "b"},
        ]
        install_relation(bundle, "flow_execution", "agentflow.flow.execution.relation@1", "InformationFlowExecutionCatalog", {
            "closed_regions": [
                {"catalog": "agentflow.region.test@1", "name": "a"},
                {"catalog": "agentflow.region.test@1", "name": "b"},
            ],
            "closed_trust_domains": [],
            "confidentiality_rules": [{"label": copy.deepcopy(label), "region_pairs": {"finite_exact_pairs": []}}],
            "taint_rules": [], "authority_rules": [], "sink_rules": [],
        })
        contract = empty_contract()
        contract["information_flow"]["confidentiality_restrictions"] = [copy.deepcopy(label)]
        contract["execution"]["region_residency"] = {"some": {
            "execution_regions": [{"catalog": "agentflow.region.test@1", "name": "a"}],
            "residency_regions": [{"catalog": "agentflow.region.test@1", "name": "b"}],
        }}
        return install_contract(bundle, contract)[0]
    add_case(cases, base, "portable-static-reconstruction/d6-15-flow-execution-relation-counterexample", "D6", "pipeline", "D6", "ContractValidationFailed", mutate=invalid_flow_execution_relation)


def d7_cases(cases, base):
    negatives = json.loads((ROOT / "spec/conformance/semantic-ir/negative/semantic-ir-negative.json").read_text(encoding="utf-8"))["cases"]
    golden = json.loads((ROOT / "spec/conformance/semantic-ir/golden/semantic-ir-golden.json").read_text(encoding="utf-8"))

    def add_fixture_schema_scope(bundle):
        source_path = golden["fixture_schema"]["file"]
        source = (ROOT / source_path).read_text(encoding="utf-8")
        _, modules = AFIDL.compile_module_set([(source_path, source)])
        specification_id = golden["fixture_schema"]["specification_id"]
        refs = []
        for schema in modules[specification_id]:
            content, digest = AFIDL.type_schema_digest(schema)
            reference = {
                "specification_id": specification_id,
                "type_name": schema["name"],
                "schema_digest": {"algorithm": "sha256", "domain": "type-schema", "hex": digest},
            }
            refs.append(reference)
            bundle["schema_assets"].append({
                "reference": copy.deepcopy(reference),
                "canonical_content": b64url(content),
            })
        context_refs = bundle["proof_contexts"][0]["content"]["type_schemas"]["complete"]
        context_refs.extend(copy.deepcopy(refs))
        ref_key = lambda reference: (
            reference["specification_id"].encode("utf-8"),
            reference["type_name"].encode("utf-8"),
            reference["schema_digest"]["algorithm"],
            reference["schema_digest"]["domain"],
            reference["schema_digest"]["hex"],
        )
        context_refs.sort(key=ref_key)
        bundle["schema_assets"].sort(key=lambda row: ref_key(row["reference"]))
        return bundle

    selected_ir = [
        "document/root-zero", "document/unknown-root", "document/duplicate-computation-id",
        "document/unreachable-computation", "document/structural-cycle", "producer/unbound-slot",
    ]
    by_id = {row["case_id"]: row for row in negatives}
    for number, fixture_id in enumerate(selected_ir, 1):
        document = copy.deepcopy(by_id[fixture_id]["document"])
        def mutate(bundle, document=document):
            bundle = add_fixture_schema_scope(bundle)
            return install_ir(bundle, document)[0]
        add_case(cases, base, f"portable-static-reconstruction/d7-{number:02d}-{fixture_id.replace('/', '-')}", "D7", "pipeline", "D7", "SemanticIrValidationFailed", mutate=mutate,
                 oracle_fixture=f"spec/conformance/semantic-ir/negative/semantic-ir-negative.json#{fixture_id}")

    dataflow = json.loads((ROOT / "spec/conformance/semantic-dataflow/negative/semantic-dataflow-negative.json").read_text(encoding="utf-8"))["cases"]
    by_id = {row["case_id"]: row for row in dataflow}
    selected_flow = [
        "expression/reuse-per-use-site", "sequence/future-sibling-output",
        "select/case-cross-reference", "parallel/cross-branch-value",
        "resource/scope-child-uses-omitted", "workflow-output/raw-root-descendant-output",
    ]
    for number, fixture_id in enumerate(selected_flow, 7):
        document = copy.deepcopy(by_id[fixture_id]["document"])
        def mutate(bundle, document=document):
            bundle = add_fixture_schema_scope(bundle)
            return install_ir(bundle, document)[0]
        add_case(cases, base, f"portable-static-reconstruction/d7-{number:02d}-{fixture_id.replace('/', '-')}", "D7", "pipeline", "D7", "Phase3CRejected", mutate=mutate,
                 oracle_fixture=f"spec/conformance/semantic-dataflow/negative/semantic-dataflow-negative.json#{fixture_id}")

    add_case(cases, base, "portable-static-reconstruction/d7-13-exact-asset-content-retained", "D7", "pipeline", "D8")
    add_case(cases, base, "portable-static-reconstruction/d7-14-canonical-ir-content-identity", "D7", "pipeline", "D8")
    add_case(cases, base, "portable-static-reconstruction/d7-15-current-module-set-used", "D7", "pipeline", "D8")
    add_case(cases, base, "portable-static-reconstruction/d7-16-phase3c-recomputed", "D7", "pipeline", "D8")


def component_ref(bundle, name="portable.codec"):
    if name == "portable.codec":
        return copy.deepcopy(bundle["contract_assets"][0]["reference"])
    contract = empty_contract(name)
    asset = canonical_asset(
        bundle, "contract_assets", contract, "component-contract",
        "agentflow.component.contract@1", "ComponentContract",
    )
    existing = [row for row in bundle["contract_assets"] if row["reference"] == asset["reference"]]
    if not existing:
        bundle["contract_assets"].append(asset)
        bundle["contract_assets"].sort(key=lambda row: (row["reference"]["digest"]["algorithm"], row["reference"]["digest"]["domain"], row["reference"]["digest"]["hex"]))
    return asset["reference"]


def component_ir(bundle, ref):
    return minimal_ir(
        {"invoke": {"contract": "1", "inputs": [], "resources": []}},
        [{"id": "1", "kind": "component", "digest": copy.deepcopy(ref["digest"])}],
    )


def subflow_ir(bundle, target_digest, semantic_ref):
    return minimal_ir(
        {"call_subflow": {"contract": "1", "inputs": [], "resources": []}},
        [{"id": "1", "kind": "subflow", "digest": copy.deepcopy(target_digest)}],
        semantic_refs=[{"specification_id": semantic_ref}],
    )


def subflow_fanout_ir(bundle, target_refs, semantic_ref):
    references = sorted(
        target_refs,
        key=lambda reference: (
            reference["digest"]["algorithm"],
            reference["digest"]["domain"],
            reference["digest"]["hex"],
        ),
    )
    contracts = [
        {"id": str(index), "kind": "subflow", "digest": copy.deepcopy(reference["digest"])}
        for index, reference in enumerate(references, 1)
    ]
    computations = [{
        "id": "1",
        "outputs": [],
        "body": {"sequence": {"children": [str(index) for index in range(2, len(references) + 2)]}},
    }]
    computations.extend(
        {
            "id": str(index + 1),
            "outputs": [],
            "body": {"call_subflow": {"contract": str(index), "inputs": [], "resources": []}},
        }
        for index in range(1, len(references) + 1)
    )
    return minimal_ir(
        {},
        contracts,
        semantic_refs=[{"specification_id": semantic_ref}],
        computations=computations,
    )


def caller_invoke(bundle, component_refs, sequential=False):
    if sequential and len(component_refs) == 2 and component_refs[0]["digest"] == component_refs[1]["digest"]:
        contracts = [{"id": "1", "kind": "component", "digest": copy.deepcopy(component_refs[0]["digest"])}]
        computations = [{"id": "1", "outputs": [], "body": {"sequence": {"children": ["2", "3"]}}}]
        computations.extend([
            {"id": "2", "outputs": [], "body": {"invoke": {"contract": "1", "inputs": [], "resources": []}}},
            {"id": "3", "outputs": [], "body": {"invoke": {"contract": "1", "inputs": [], "resources": []}}},
        ])
        return minimal_ir({}, contracts, computations=computations)
    contracts = [
        {"id": str(index), "kind": "component", "digest": copy.deepcopy(reference["digest"])}
        for index, reference in enumerate(component_refs, 1)
    ]
    if sequential:
        computations = [{"id": "1", "outputs": [], "body": {"sequence": {"children": ["2", "3"]}}}]
        computations.extend([
            {"id": "2", "outputs": [], "body": {"invoke": {"contract": "1", "inputs": [], "resources": []}}},
            {"id": "3", "outputs": [], "body": {"invoke": {"contract": "2", "inputs": [], "resources": []}}},
        ])
        return minimal_ir({}, contracts, computations=computations)
    return minimal_ir({"invoke": {"contract": "1", "inputs": [], "resources": []}}, contracts)


def caller_subflows(bundle, target_refs):
    contracts = [
        {"id": str(index), "kind": "subflow", "digest": copy.deepcopy(reference["digest"])}
        for index, reference in enumerate(target_refs, 1)
    ]
    if len(target_refs) == 1:
        return minimal_ir({"call_subflow": {"contract": "1", "inputs": [], "resources": []}}, contracts)
    computations = [{"id": "1", "outputs": [], "body": {"sequence": {"children": ["2", "3"]}}}]
    computations.extend([
        {"id": "2", "outputs": [], "body": {"call_subflow": {"contract": "1", "inputs": [], "resources": []}}},
        {"id": "3", "outputs": [], "body": {"call_subflow": {"contract": "2", "inputs": [], "resources": []}}},
    ])
    return minimal_ir({}, contracts, computations=computations)


def invoke_association(source_node, reference):
    return {"source_node": str(source_node), "target": {"invoke": copy.deepcopy(reference)}}


def subflow_association(source_node, digest, export_ref):
    return {
        "source_node": str(source_node),
        "target": {"call_subflow": {
            "target_semantic_ir_digest": copy.deepcopy(digest),
            "exported_contract_digest": option_some(copy.deepcopy(export_ref["digest"])) if export_ref else option_none(),
        }},
    }


def add_two_component_contracts(bundle):
    result = copy.deepcopy(bundle)
    first = component_ref(result, "portable.codec")
    second = component_ref(result, "portable.other")
    return result, first, second


def graph_bundle(base, caller, target_ir_specs, component_refs, associations, root_export=None):
    bundle = copy.deepcopy(base)
    return set_full_resolver(bundle, caller, target_ir_specs, component_refs, associations, root_export)


def d8_cases(cases, base):
    first = component_ref(base)
    base_caller = json.loads(b64decode(base["ir_assets"][0]["canonical_content"]).decode("utf-8"))
    base_assoc = [{"source_node": "1", "target": {"invoke": copy.deepcopy(first)}}]
    add_case(cases, base, "portable-static-reconstruction/d8-01-valid-invoke-source", "D8", "pipeline", "D8")

    shared_caller = caller_invoke(base, [first, first], sequential=True)
    shared_bundle, _, _ = graph_bundle(base, shared_caller, [], [first], [invoke_association(2, first), invoke_association(3, first)], first)
    add_case(cases, base, "portable-static-reconstruction/d8-02-shared-contract-distinct-occurrences", "D8", "pipeline", "D8", mutate=lambda _: shared_bundle)

    target_b = component_ir(base, first)
    target_b_ref = canonical_asset(base, "ir_assets", target_b, "semantic-ir", "agentflow.semantic.ir@1", "SemanticIrDocument")["reference"]
    caller = caller_subflows(base, [target_b_ref])
    subflow_bundle, caller_ref, target_rows = graph_bundle(base, caller, [(target_b, first)], [first], [subflow_association(1, target_b_ref["digest"], first)], first)
    add_case(cases, base, "portable-static-reconstruction/d8-03-valid-subflow-target-export", "D8", "pipeline", "D8", mutate=lambda _: subflow_bundle)

    two_contract_base, first_export, second_export = add_two_component_contracts(base)
    t1 = component_ir(two_contract_base, first_export)
    t2 = component_ir(two_contract_base, first_export)
    t1["semantic_refs"] = [{"specification_id": "agentflow.target.one@1"}]
    t2["semantic_refs"] = [{"specification_id": "agentflow.target.two@1"}]
    t1_ref = canonical_asset(two_contract_base, "ir_assets", t1, "semantic-ir", "agentflow.semantic.ir@1", "SemanticIrDocument")["reference"]
    t2_ref = canonical_asset(two_contract_base, "ir_assets", t2, "semantic-ir", "agentflow.semantic.ir@1", "SemanticIrDocument")["reference"]
    caller = caller_subflows(two_contract_base, [t1_ref, t2_ref])
    mapping_a, _, _ = graph_bundle(two_contract_base, caller, [(t1, first_export), (t2, second_export)], [first_export, second_export], [subflow_association(2, t1_ref["digest"], first_export), subflow_association(3, t2_ref["digest"], second_export)], first_export)
    add_case(cases, base, "portable-static-reconstruction/d8-04-target-export-association-a", "D8", "pipeline", "D8", mutate=lambda _: mapping_a)
    mapping_b, _, _ = graph_bundle(two_contract_base, caller, [(t1, second_export), (t2, first_export)], [first_export, second_export], [subflow_association(2, t1_ref["digest"], second_export), subflow_association(3, t2_ref["digest"], first_export)], first_export)
    add_case(cases, base, "portable-static-reconstruction/d8-05-target-export-association-swapped", "D8", "pipeline", "D8", mutate=lambda _: mapping_b)

    no_deps = copy.deepcopy(subflow_bundle)
    no_deps["resolver_dependency_manifests"][0]["content"]["subflow_targets"][0]["direct_dependencies"] = option_none()
    add_case(cases, base, "portable-static-reconstruction/d8-06-none-not-some-empty-dependencies", "D8", "pipeline", "D8", "IncompleteResolver", mutate=lambda _: no_deps)

    # A -> B: the target IR proves one direct edge, while the declaration
    # claims none. The mismatch is caught against freshly inserted IR facts.
    leaf_b = component_ir(base, first)
    leaf_b["semantic_refs"] = [{"specification_id": "agentflow.target.leaf@1"}]
    ref_b = canonical_asset(base, "ir_assets", leaf_b, "semantic-ir", "agentflow.semantic.ir@1", "SemanticIrDocument")["reference"]
    node_a = subflow_ir(base, ref_b["digest"], "agentflow.target.parent@1")
    ref_a = canonical_asset(base, "ir_assets", node_a, "semantic-ir", "agentflow.semantic.ir@1", "SemanticIrDocument")["reference"]
    edge_ref_a = copy.deepcopy(ref_a)
    edge_ref_b = copy.deepcopy(ref_b)
    caller = caller_subflows(base, [ref_a])
    edge_bundle, _, _ = graph_bundle(base, caller, [(node_a, first), (leaf_b, first)], [first], [subflow_association(1, ref_a["digest"], first)], first)
    row_a = next(row for row in edge_bundle["resolver_dependency_manifests"][0]["content"]["subflow_targets"] if row["target_semantic_ir_digest"] == ref_a["digest"])
    row_a["direct_dependencies"] = option_some([])
    add_case(cases, base, "portable-static-reconstruction/d8-07-missing-direct-dependency", "D8", "pipeline", "D8", "DirectDependencyMismatch", mutate=lambda _: edge_bundle)

    # A -> B -> C; direct edges match, but A's submitted closure omits C.
    leaf_c = component_ir(base, first)
    leaf_c["semantic_refs"] = [{"specification_id": "agentflow.target.c@1"}]
    ref_c = canonical_asset(base, "ir_assets", leaf_c, "semantic-ir", "agentflow.semantic.ir@1", "SemanticIrDocument")["reference"]
    node_b = subflow_ir(base, ref_c["digest"], "agentflow.target.b@1")
    ref_b = canonical_asset(base, "ir_assets", node_b, "semantic-ir", "agentflow.semantic.ir@1", "SemanticIrDocument")["reference"]
    node_a = subflow_ir(base, ref_b["digest"], "agentflow.target.a@1")
    ref_a = canonical_asset(base, "ir_assets", node_a, "semantic-ir", "agentflow.semantic.ir@1", "SemanticIrDocument")["reference"]
    caller = caller_subflows(base, [ref_a])
    chain, _, _ = graph_bundle(base, caller, [(node_a, first), (node_b, first), (leaf_c, first)], [first], [subflow_association(1, ref_a["digest"], first)], first)
    row_a = next(row for row in chain["resolver_dependency_manifests"][0]["content"]["subflow_targets"] if row["target_semantic_ir_digest"] == ref_a["digest"])
    row_a["transitive_dependencies"]["some"] = [item for item in row_a["transitive_dependencies"]["some"] if item["semantic_ir_digest"] != ref_c["digest"]]
    add_case(cases, base, "portable-static-reconstruction/d8-08-missing-transitive-dependency", "D8", "pipeline", "D8", "TransitiveDependencyMismatch", mutate=lambda _: chain)

    # A -> B is a valid independently derived edge and closure fixture.
    edge_leaf = component_ir(base, first)
    edge_leaf["semantic_refs"] = [{"specification_id": "agentflow.target.edgeleaf@1"}]
    edge_leaf_ref = canonical_asset(base, "ir_assets", edge_leaf, "semantic-ir", "agentflow.semantic.ir@1", "SemanticIrDocument")["reference"]
    edge_parent = subflow_ir(base, edge_leaf_ref["digest"], "agentflow.target.edgeparent@1")
    edge_parent_ref = canonical_asset(base, "ir_assets", edge_parent, "semantic-ir", "agentflow.semantic.ir@1", "SemanticIrDocument")["reference"]
    edge_caller = caller_subflows(base, [edge_parent_ref])
    edge_positive, _, _ = graph_bundle(base, edge_caller, [(edge_parent, first), (edge_leaf, first)], [first], [subflow_association(1, edge_parent_ref["digest"], first)], first)
    add_case(cases, base, "portable-static-reconstruction/d8-09-valid-direct-transitive-closure", "D8", "pipeline", "D8", mutate=lambda _: edge_positive)

    duplicate_source = copy.deepcopy(base)
    duplicate_source["resolver_dependency_manifests"][0]["content"]["source_associations"].append(copy.deepcopy(duplicate_source["resolver_dependency_manifests"][0]["content"]["source_associations"][0]))
    duplicate_source["resolver_dependency_manifests"][0]["content"]["source_associations"].sort(key=lambda row: (row["caller_ir"]["digest"]["hex"], row["source_node"]))
    add_case(cases, base, "portable-static-reconstruction/d8-10-duplicate-source-early-d1-guard", "D8", "pipeline", "D1", "DuplicateSetMember", mutate=lambda _: duplicate_source, required_compared_fields=["resolver.source_associations", "actual_stage", "failure_family"])

    duplicate_target = copy.deepcopy(subflow_bundle)
    duplicate_target["resolver_dependency_manifests"][0]["content"]["subflow_targets"].append(copy.deepcopy(duplicate_target["resolver_dependency_manifests"][0]["content"]["subflow_targets"][0]))
    duplicate_target["resolver_dependency_manifests"][0]["content"]["subflow_targets"].sort(key=lambda row: row["target_semantic_ir_digest"]["hex"])
    add_case(cases, base, "portable-static-reconstruction/d8-11-duplicate-target-early-d1-guard", "D8", "pipeline", "D1", "DuplicateSetMember", mutate=lambda _: duplicate_target, required_compared_fields=["resolver.subflow_targets", "actual_stage", "failure_family"])

    declared_cycle = copy.deepcopy(edge_bundle)
    rows = declared_cycle["resolver_dependency_manifests"][0]["content"]["subflow_targets"]
    row_a = next(row for row in rows if row["target_semantic_ir_digest"] == edge_ref_a["digest"])
    row_b = next(row for row in rows if row["target_semantic_ir_digest"] == edge_ref_b["digest"])
    row_a["direct_dependencies"] = option_some([copy.deepcopy(edge_ref_b["digest"])])
    row_b["direct_dependencies"] = option_some([copy.deepcopy(edge_ref_a["digest"])])
    add_case(cases, base, "portable-static-reconstruction/d8-12-declared-cycle-disagrees-with-ir", "D8", "pipeline", "D8", "DirectDependencyMismatch", mutate=lambda _: declared_cycle)

    missing_content = copy.deepcopy(subflow_bundle)
    missing_content["resolver_dependency_manifests"][0]["content"]["subflow_targets"][0]["target_ir_content"] = option_none()
    missing_target_digest = missing_content["resolver_dependency_manifests"][0]["content"]["subflow_targets"][0]["target_semantic_ir_digest"]
    missing_content["ir_assets"] = [
        asset for asset in missing_content["ir_assets"]
        if asset["reference"]["digest"] != missing_target_digest
    ]
    add_case(cases, base, "portable-static-reconstruction/d8-13-missing-target-content-option", "D8", "pipeline", "D8", "IncompleteResolver", mutate=lambda _: missing_content)

    wrong_source = copy.deepcopy(base)
    second = component_ref(wrong_source, "portable.other")
    resolver = wrong_source["resolver_dependency_manifests"][0]["content"]
    resolver["components"].append(second)
    resolver["components"].sort(key=lambda row: row["digest"]["hex"])
    resolver["source_associations"][0]["target"] = {"invoke": second}
    wrong_source["contract_assets"].sort(key=lambda row: row["reference"]["digest"]["hex"])
    add_case(cases, base, "portable-static-reconstruction/d8-14-changed-source-contract-association", "D8", "pipeline", "D8", "SourceAssociationMismatch", mutate=lambda _: wrong_source)

    await_type = schema_reference(base, "agentflow.semantic.ir@1", "SemanticIrDocument")
    await_ir = minimal_ir(
        {"await": {"signal_type": await_type, "correlation": "1"}}, [],
    )
    await_ir["types"] = [{"id": "1", "body": {"primitive": "int64"}}]
    await_ir["expressions"] = [{"id": "1", "result_type": "1", "body": {"constant": {"int64": "1"}}}]
    await_bundle, _, _ = graph_bundle(base, await_ir, [], [first], [{"source_node": "1", "target": {"await_signal": await_type}}], first)
    add_case(cases, base, "portable-static-reconstruction/d8-15-await-unknown-external-preserved", "D8", "pipeline", "D8", mutate=lambda _: await_bundle)

    bounded = copy.deepcopy(base)
    target_specs = []
    target_refs = [None] * MAX_TARGET_CASES
    leaf_index = MAX_TARGET_CASES - 1
    leaf_ir = component_ir(bounded, first)
    leaf_ir["semantic_refs"] = [{"specification_id": f"agentflow.target.limit{leaf_index:04d}@1"}]
    leaf_ref = canonical_asset(bounded, "ir_assets", leaf_ir, "semantic-ir", "agentflow.semantic.ir@1", "SemanticIrDocument")["reference"]
    target_refs[leaf_index] = leaf_ref
    target_specs.append((leaf_ir, None))
    for index in reversed(range(MAX_TARGET_CASES - 1)):
        parent = subflow_fanout_ir(
            bounded,
            target_refs[index + 1:],
            f"agentflow.target.limit{index:04d}@1",
        )
        parent_ref = canonical_asset(bounded, "ir_assets", parent, "semantic-ir", "agentflow.semantic.ir@1", "SemanticIrDocument")["reference"]
        target_refs[index] = parent_ref
        target_specs.append((parent, None))
    root_target_ref = target_refs[0]
    caller = caller_subflows(bounded, [root_target_ref])
    too_much_work, _, _ = set_full_resolver(
        bounded,
        caller,
        target_specs,
        [first],
        [subflow_association(1, root_target_ref["digest"], None)],
        first,
        build_transitive=False,
    )
    add_case(cases, base, "portable-static-reconstruction/d8-16-resolver-closure-work-limit", "D8", "pipeline", "D8", "ResourceLimit", mutate=lambda _: too_much_work)


def main():
    base_raw, base = read_base()
    cases = []
    d5_cases(cases, base)
    d6_cases(cases, base)
    d7_cases(cases, base)
    d8_cases(cases, base)
    comparator_mutations = [
        ("context-membership-lost", "context_membership_lost", "portable-static-reconstruction/d5-01-complete-context"),
        ("global-resource-scope-collapsed", "schema_scope_collapsed", "portable-static-reconstruction/d5-05-resource-private-schema-scope"),
        ("recovery-catalog-reused", "recovery_effect_catalog_reused", "portable-static-reconstruction/d5-07-recovery-effect-catalog-independent"),
        ("execution-exact-version-changed", "execution_exact_version_changed", "portable-static-reconstruction/d5-09-execution-exact-version-v2"),
        ("relation-content-changed", "relation_content_changed", "portable-static-reconstruction/d5-12-some-empty-relation-retained"),
        ("contract-source-pin-dropped", "contract_source_pin_dropped", "portable-static-reconstruction/d8-01-valid-invoke-source"),
        ("component-guard-violation-promoted", "component_guard_violation_promoted", "portable-static-reconstruction/d6-12-action-guard-counterexample-no-partial-checked"),
        ("ir-producer-count-swapped", "ir_producer_count_swapped", "portable-static-reconstruction/d7-13-exact-asset-content-retained"),
        ("scope-alias-lost", "scope_alias_lost", "portable-static-reconstruction/d5-05-resource-private-schema-scope"),
        ("workflow-exit-producer-dropped", "workflow_exit_producer_dropped", "portable-static-reconstruction/d7-13-exact-asset-content-retained"),
        ("resolver-target-export-swapped", "resolver_target_export_swapped", "portable-static-reconstruction/d8-04-target-export-association-a"),
        ("source-computation-id-collapsed", "source_computation_id_collapsed", "portable-static-reconstruction/d8-02-shared-contract-distinct-occurrences"),
        ("none-to-some-empty", "none_to_some_empty", "portable-static-reconstruction/d8-03-valid-subflow-target-export"),
        ("dependency-edge-dropped", "dependency_edge_dropped", "portable-static-reconstruction/d8-09-valid-direct-transitive-closure"),
        ("unknown-external-erased", "unknown_external_erased", "portable-static-reconstruction/d8-15-await-unknown-external-preserved"),
        ("resource-limit-promoted", "resource_limit_promoted", "portable-static-reconstruction/d8-16-resolver-closure-work-limit"),
    ]
    ids = [row["case_id"] for row in cases]
    if len(cases) < 56 or len(ids) != len(set(ids)):
        raise SystemExit("reconstruction corpus must contain 56+ unique CaseIds")
    if sum(row["group"] == "D5" for row in cases) != 16:
        raise SystemExit("D5 group must retain its 16-case allocation")
    if sum(row["group"] == "D6" for row in cases) != 15:
        raise SystemExit("D6 group must retain its original 12 cases plus relation counterexamples")
    if sum(row["group"] == "D7" for row in cases) != 16:
        raise SystemExit("D7 group must retain its 12+4-case allocation")
    if sum(row["group"] == "D8" for row in cases) != 16:
        raise SystemExit("D8 group must retain its 16-case allocation")
    OUT.mkdir(parents=True, exist_ok=True)
    manifest = {
        "corpus": "agentflow.portable.static-reconstruction",
        "revision": 1,
        "base_file": BASE_PATH,
        "base_sha256": sha256(base_raw),
        "case_count": len(cases),
        "group_counts": {group: sum(row["group"] == group for row in cases) for group in ["D5", "D6", "D7", "D8"]},
        "required_context_projection_fields": REQUIRED_CONTEXT_FIELDS,
        "required_pipeline_projection_fields": REQUIRED_PIPELINE_FIELDS,
        "comparator_mutation_count": len(comparator_mutations),
        "comparator_mutations": [
            {"mutation_id": mutation_id, "dimension": dimension, "witness_case_id": witness}
            for mutation_id, dimension, witness in comparator_mutations
        ],
        "cases": cases,
    }
    (OUT / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"generated {len(cases)} independent D5-D8 cases in {OUT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
