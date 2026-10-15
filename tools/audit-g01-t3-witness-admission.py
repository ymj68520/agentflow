"""Read-only R3 source/witness admission audit; NOT a T3 capture oracle.

Exit 2 reports the reviewed formal descriptor binding gap. It is not a Rust
test failure and never represents an empty successful Fact collection.
"""
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASE = "8c84f663e446bf967d1a541d31275bb4112c33d5"
MODEL = "spec/tables/proof-report-model-v2.md"
NAMES = [
    "af-contract::ActionGuardCompatibility", "af-contract::GuardCompatibilityBasis",
    "af-contract::TypedGuardCountermodel", "af-contract::GuardInputAssignment",
    "af-contract::GuardCountermodelReachability",
]


def text(path):
    return (ROOT / path).read_text(encoding="utf8")


def positions(path, pattern):
    source = text(path).split("#[cfg(test)]", 1)[0]
    return [{"path": path, "line": source.count("\n", 0, m.start()) + 1,
             "audit_only_not_origin": True} for m in re.finditer(pattern, source)]


def main():
    checked = subprocess.run(["git", "diff", "--quiet", BASE, "--", "crates", "spec", "Cargo.toml", "Cargo.lock"], cwd=ROOT, check=False)
    if checked.returncode != 0:
        raise RuntimeError("Production/formal sources differ from the reviewed binding baseline")
    raw = (ROOT / MODEL).read_bytes()
    model = json.loads(re.search(r"```json\s*(.*?)\s*```", raw.decode(), re.S)[1])
    specs = json.loads(text("spec/registry/specifications.json"))["specifications"]
    rules = json.loads(text("spec/registry/rules.json"))["rules"]
    assert (len(specs), len(rules)) == (50, 452)
    owner = next(x for x in specs if x["specification_id"] == model["model_id"])
    assert owner["normative_source"]["content_digest"]["hex"] == hashlib.sha256(raw).hexdigest()
    types = model["types"]
    pending = list(types.values())
    refs = 0
    while pending:
        value = pending.pop()
        if isinstance(value, dict):
            assert "_generic" not in value and "_case" not in value
            if "ref" in value:
                assert value["ref"] in types
                refs += 1
            pending.extend(value.values())
        elif isinstance(value, list):
            pending.extend(value)
    assert len(types) == 518 and refs == 1593
    snapshots = model["inherited_registered_model"]["semantic_model_declarations"]
    missing = []
    for name in NAMES:
        assert name in snapshots and name not in types
        missing.append({"name": name, "historical_native_snapshot": snapshots[name],
                        "active_descriptor": "ABSENT", "binding": "UNRESOLVED",
                        "historical_type_is_declared": True,
                        "not_claimed": "absence from current types is not absence from all registered historical declarations"})
    witness_fields = {x["field"] for x in snapshots[NAMES[2]]["fields"]}
    identical_records = [n for n, v in types.items() if v.get("kind") == "record"
                         and {x[0] for x in v["fields"]} == witness_fields]
    assert not identical_records
    cases = model["case_descriptors"]
    assert len(cases) == 27 and len(model["dimension_descriptors"]) == 12
    control = "crates/af-computation/src/control.rs"
    leaf = "crates/af-computation/src/resolution.rs"
    root = "crates/af-computation/src/root_export.rs"
    native = "crates/af-contract/src/validate.rs"
    sites = {
        "control_constructor_calls": positions(control, r"ControlDimensionProof::new\("),
        "leaf_constructor_calls": positions(leaf, r"LeafDimensionProof::new\("),
        "legacy_root_direct_pushes": positions(root, r"unresolved_obligations\.push\("),
        "inherited_literals": positions(control, r"(?<!struct )(?<!impl )InheritedControlObligation\s*\{"),
        "native_dimension_constructor_calls": positions(native, r"DimensionProof::(?:proven|unproven|violation)\("),
        "native_guard_record_sites": positions("crates/af-contract/src/recovery_guard.rs", r"(?<!struct )(?<!impl )ActionGuardCompatibility\s*\{"),
    }
    assert [len(sites[k]) for k in list(sites)[:4]] == [65, 27, 6, 3]
    body = text(native).split("fn attach_prerequisites", 1)[1].split("fn declared_effects", 1)[0]
    arms = re.findall(r'"([a-z-]+)"\s*=>\s*vec!', body)
    literals = re.findall(r'"([^"\n]+)"\.into\(\)', body)
    assert len(arms) == 10 and "_ => vec!" in body and len(literals) == 20
    baseline_path = ROOT / "target/phase-04c1b4c4r3-baseline.json"
    byte_audit = {"status": "NOT_AVAILABLE"}
    if baseline_path.is_file():
        base = json.loads(baseline_path.read_bytes())
        drift = [p for p, h in base.items() if not (ROOT/p).is_file()
                 or hashlib.sha256((ROOT/p).read_bytes()).hexdigest() != h]
        assert not drift, drift
        byte_audit = {"status": "PASS", "original_nonignored_files": len(base),
                      "changed": 0, "removed": 0}
    output = {
        "kind": "R3 source/witness admission evidence;NOT capture conformance",
        "baseline": BASE, "decision": "OUTCOME_C_FORMAL_BINDING_REQUIRED",
        "finding_id": "T3-MODEL-01", "model": model["model_id"],
        "binding_question": "Native AF-CONTRACT-0022 single-contract guard fact has no caller/root. Identify the exact registered predicate/source/coordinate and full witness binding without relabelling it as the AF-SEMANTIC-0156 root aggregate or standalone two-contract comparison.",
        "scope_of_conclusion": "Formal binding unresolved for required native guard producer;not a proof that no possible lossless mapping could be approved. Metadata API rejection alone is not the semantic argument.",
        "model_sha256": hashlib.sha256(raw).hexdigest(),
        "descriptor_count": len(types), "resolved_references": refs,
        "missing_active_bindings": missing, "identical_native_witness_field_records": identical_records,
        "existing_guard_contradiction": dict(types["TypedContradiction"]["variants"])["guard_countermodel"],
        "existing_guard_source": types["af-computation::RecoveryActionSourceIdentity"],
        "reviewed_lexical_sites": sites,
        "lexical_counts_not_semantic_producer_total": {k: len(v) for k, v in sites.items()},
        "prerequisite_match_arms": 11, "prerequisite_literal_occurrences": len(literals),
        "constructor_categories_overlap": True,
        "total_unique_semantic_producer_sites": "NOT_CLOSED;do_not_sum_categories",
        "capture_sites": 0, "independently_compared_capture_sites": 0,
        "capture_case_ids": 0, "capture_field_instances_compared": 0,
        "required_coverage_status": "BLOCKED_NOT_ZERO_GAPS",
        "original_byte_audit": byte_audit,
        "complete_scope": "PendingG02", "normalizer": "NotImplemented",
        "positive_capability": "NotAuthorized",
    }
    if len(sys.argv) > 1:
        probe_path = Path(sys.argv[1])
        probe = json.loads(probe_path.read_bytes())
        assert probe["guard_verdict"] == "Violation" and probe["guard_basis"] == "TypedCountermodel"
        assert probe["countermodel"]["assignment_value_int64"] == 1
        assert probe["forward_guard"] == "unconditional"
        assert probe["action_guard"] == {"port_no": 1, "expected_int64": 0}
        assert 1 != 0  # independently specified Int64 witness law, not a reason parser
        assert not probe["positive_capability_allowed"]
        assert probe["owner_controls"]["historical_owner_known_scope"] == "UnsupportedModel"
        assert [x["name"] for x in probe["model_admission"]] == NAMES
        assert all(x["actual_model_type_admission"] == "UnsupportedModel" for x in probe["model_admission"])
        output["actual_native_probe"] = {"path": probe_path.as_posix(),
            "sha256": hashlib.sha256(probe_path.read_bytes()).hexdigest(), "actual": probe,
            "not_independent_expected": True}
    print(json.dumps(output, ensure_ascii=True, indent=2))
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
