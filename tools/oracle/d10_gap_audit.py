"""Declaration-only, explicitly INCOMPLETE D10 precondition audit.

This is not a RootExport proof oracle. In particular it does not select the
checker-specific spelling/order of unresolved obligations from Rust output.
Existing D9 algebra may discharge preconditions; gaps cannot become Expected.
"""

from __future__ import annotations

import copy
import hashlib
import json
from collections import Counter
from pathlib import Path

from oracle import field_contract as FC
from oracle.d9_algebra import OracleGap
from oracle.d9_recovery_model import RecoveryModel
from portable_proof_python_oracle import canonical_bytes

ROOT = Path(__file__).resolve().parents[2]
CORPUS = ROOT / "spec/conformance/portable-root-export-revalidation"
OUT = CORPUS / "expected-full/d10-preconditions"
GAP_ID = "ORACLE-D10-G01"


def materialize(case):
    """Apply ORIGINAL registered input mutations, not output mutations.

    Bounded paths and existing members only. No repair, pin substitution,
    deletion, or production Expected/verdict is consulted.
    """
    raw = (ROOT / case["bundle_file"]).read_bytes()
    value = json.loads(raw)
    changes = case.get("mutations", [])
    if len(changes) > 128:
        raise OracleGap("OracleResourceLimit/input mutations")
    for change in changes:
        path = change["path"]
        if change["op"] != "set" or not 1 <= len(path) <= 96:
            raise OracleGap("unsupported/over-limit registered input mutation")
        current = value
        for token in path[:-1]:
            current = current[int(token)] if isinstance(current, list) else current[token]
        token = int(path[-1]) if isinstance(current, list) else path[-1]
        # Unlike assignment to a new key, lookup must fail for a missing member.
        current[token]
        current[token] = copy.deepcopy(change["value"])
    encoded = canonical_bytes(value) if changes else raw
    if canonical_bytes(value) != encoded:
        raise OracleGap("registered input is not canonical machine JSON")
    return value, encoded


def registration():
    manifest = json.loads((CORPUS / "manifest.json").read_text(encoding="utf-8"))
    groups = Counter(row["group"] for row in manifest["cases"])
    assert groups == {"D9-Control": 22, "D9-Recovery": 16, "D10": 26,
                      "Context-Resolver-Compiler": 8, "Cross-Process": 6}
    cases = [row for row in manifest["cases"] if row["group"] == "D10"]
    assert len({row["case_id"] for row in cases}) == 26
    return cases


def identity(value):
    raw = canonical_bytes(value)
    return {"canonical_bytes": len(raw), "canonical_sha256": hashlib.sha256(raw).hexdigest()}


def require_full_d10_expected(_bundle):
    # This guard is intentionally unconditional until an authority supplies an
    # exact obligation identity/encoding contract. Never return a partial report
    # under the full-field Expected API, even when all static verdicts are known.
    raise OracleGap(GAP_ID + ": RootExportReport.unresolved_obligations exact identity/order is not specified")


def artifacts():
    outputs = {}
    rows = []
    for case in registration():
        bundle, raw = materialize(case)
        entry = next(row for row in bundle["root_export_assertions"]
                     if row["id"] == bundle["root_assertion_id"])
        assertion = entry["assertion"]
        context_ids = assertion["proof_context_manifest"]
        context = next(row["content"] for row in bundle["proof_contexts"]
                       if row["id"] == context_ids["context_id"])
        resolver = next(row["content"] for row in bundle["resolver_dependency_manifests"]
                        if row["id"] == context_ids["resolver_id"])
        slug = case["case_id"].rsplit("/", 1)[-1]
        row = {
            "case_id": case["case_id"], "group": "D10",
            "bundle_file": case["bundle_file"], "mutations": case.get("mutations", []),
            "input_identity": {"canonical_bytes": len(raw), "canonical_sha256": hashlib.sha256(raw).hexdigest()},
            "expected_first_stage": "D10",
            "stage_expectation_basis": "Original D10 group registration; independently tested below, not a new full D0-D9 oracle",
            "actual_first_stage": "PendingProductionRun",
            "checked_ir_digest": entry["ir_asset"]["digest"],
            "top_export_digest": entry["export_contract_asset"]["digest"],
            "context_declaration_identity": identity(context),
            "resolver_declaration_identity": identity(resolver),
            "root_assertion_id": bundle["root_assertion_id"],
            "mapped_source_inventory": {
                family: copy.deepcopy(assertion[family]) for family in
                ["input_mappings", "output_mappings", "resource_mappings", "effect_mappings",
                 "flow_mappings", "recovery_mappings", "execution_mappings", "subflow_dependencies"]},
            "expected_static_verdict": "OracleGapNotFilledFromActual",
            "expected_capability_status": "OracleGapNotFilledFromActual",
            "d10_full_field_status": "FormalModelGap",
            "d10_gap_ids": [GAP_ID, "ORACLE-D10-G02"],
            "expected_d9_file": None, "d9_full_field_status": "PendingProductionRun",
        }
        try:
            facts = RecoveryModel(bundle).facts(case["case_id"], raw)
            count = FC.validate(facts, FC.ref("D9Facts"))
            path = OUT / (slug + ".json")
            outputs[path] = {
                "schema_version": 1,
                "scope": "D9 preconditions only; NOT a D10 Expected",
                "authority": "Portable declarations plus existing independent D9 Control/Recovery formal algebra",
                "facts": facts, "instance_coverage": count,
                "d10_full_field_status": "FormalModelGap",
            }
            row["expected_d9_file"] = path.relative_to(ROOT).as_posix()
            row["instance_coverage"] = count
        except OracleGap as error:
            # An implementation limitation is NOT a new missing formal rule.
            row["d9_full_field_status"] = "OracleImplementationGap"
            row["d9_gap"] = {"id": "ORACLE-D10-G03", "detail": str(error)}
        rows.append(row)
    assert len(rows) == 26
    outputs[OUT / "manifest.json"] = {
        "schema_version": 1, "outcome": "C - Formal Model Gap; NOT COMPLETE",
        "case_count": 26, "d9_expected_count": sum(row["expected_d9_file"] is not None for row in rows),
        "d9_generated_instance_totals": {
            key: sum(row.get("instance_coverage", {}).get(key, 0) for row in rows)
            for key in ["required", "diagnostic"]},
        "d9_comparison_status": "PendingProductionRun",
        "d10_full_field_compared": 0, "d10_full_field_expected": 0,
        "coverage_union_updated": False, "cases": rows,
    }
    outputs[CORPUS / "coverage/d10-gap-ledger.json"] = gap_ledger()
    return outputs


def gap_ledger():
    return {
        "schema_version": 1, "stage_outcome": "C", "complete": False,
        "findings": [
            {
                "id": GAP_ID, "severity": "P1", "kind": "FormalModelGap",
                "field": "RootExportReport.unresolved_obligations",
                "applicable_case_ids": [row["case_id"] for row in registration()],
                "normative_sources": [
                    "spec/computation/root-export-contract-proof.md#subflow-body-separation",
                    "spec/computation/root-export-contract-proof.md#static-runtime-boundary",
                    "spec/component/component-contract.md#component-contract-proof-composition"],
                "known_facts": [
                    "Static compatibility does not prove implementation conformance",
                    "Static proof establishes no publisher authentication or runtime authority",
                    "Without trusted body proof, body conformance RequiresEvidence",
                    "Scope authority/budget attenuation Unproven without declared ceilings"],
                "missing_fact": "Exact nominal obligation identifiers, string encoding, enumeration/aggregation conditions, order and duplicate policy for this required Vec<String>",
                "why_not_diagnostic": "The frozen executable field contract marks the entire array RequiredSemantic with exact ordering, unlike reason prose",
                "non_uniqueness": {
                    "premise": "Implementation conformance requires independent evidence",
                    "possible_encodings_not_expected": ["implementation:requires_evidence", "implementation_conformance:requires_evidence"],
                    "claim": "The normative premise selects neither exact spelling. These are illustrative encodings, not fabricated production reports or Expected. Exact comparison cannot choose by observing Actual."},
                "resolution_required": "A separately authorized formal conformance vocabulary/typed obligation identity and ordering contract. No field may be silently dropped or reclassified DiagnosticOnly.",
                "production_soundness_claim": "None; no false-Proven established",
            },
            {
                "id": "ORACLE-D10-G02", "severity": "P1", "kind": "ProjectionCoverageDefect",
                "field": "Assertion.compiler_semantics / Assertion.proof_context_manifest",
                "evidence": [
                    "CompilerIdentity registry_content_bytes and specification content_bytes are u64 lengths, not bytes",
                    "ContextManifest omits execution universes and reduces multiple universes to counts",
                    "RelationSnapshot content_bytes retains length/checksum but not canonical bytes"],
                "normative_sources": ["spec/computation/root-export-contract-proof.md#compiler-semantics-identity",
                                      "spec/computation/root-export-contract-proof.md#proof-context-manifest"],
                "production_soundness_claim": "None; typed production identities retain complete content; legacy report projection is lossy",
                "resolution_required": "Lossless test-only getters and reviewed source/projection bindings before any full D10 closure claim; not a production semantics edit",
                "status": "OPEN - repairable within test-only scope, not itself a FormalModelGap",
            },
            {
                "id": "ORACLE-D10-G03", "severity": "P2", "kind": "OracleImplementationGap",
                "field": "D9 nonempty CALL_SUBFLOW interface",
                "applicable_case_ids": [row["case_id"] for row in registration() if "/p31-" in row["case_id"]],
                "resolution_required": "Extend existing independent Phase 3D interface model; normative interface rules exist. Do not confuse this with missing formal semantics.",
                "status": "OPEN",
            }],
        "audit_findings": {"AUDIT-F01": "CLOSED", "AUDIT-F02": "CLOSED", "AUDIT-F03": "OPEN/P2", "AUDIT-F04": "OPEN/P2"},
        "b01_b02_conformance_closure": "BLOCKING", "phase_4d_admission": False,
    }
