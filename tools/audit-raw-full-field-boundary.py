"""C2R2 read-only descriptor/instance audit, NOT a full-field conformance oracle.

The default strict gate fails closed. --materialize saves diagnostic manifests;
it does not grant conformance, produce a FactKey, or change a frozen Expected.
"""
import argparse
import functools
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys
import unicodedata

ROOT = Path(__file__).resolve().parents[1]
PREFIX = "phase-04c1b4c4r3c2r2-"
BASE = "cee9b10e965dfd0bd16ed73ded3e72eca16cb7d8"
MODEL_PATH = "spec/tables/proof-report-model-v5.md"
AUTHORIZED_EXISTING = {
    "crates/af-computation/src/raw_capture.rs",
    "crates/af-computation/src/registered_model.rs",
    "crates/af-computation/tests/raw_port_capture_conformance.rs",
}


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT)


def model():
    raw = (ROOT / MODEL_PATH).read_text(encoding="utf8")
    return json.loads(re.search(r"```json\s*(.*?)\s*```", raw, re.S)[1])


@functools.lru_cache(maxsize=1)
def input_module():
    spec = importlib.util.spec_from_file_location(
        "unchanged_c2r1_inputs", ROOT / "tools/oracle/raw_port_capture.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def load_inputs():
    module = input_module()
    return module.vectors(), module.SOURCE_FAILURES


def events(inputs, failures):
    """Independently enumerate the reached prefix, not from an Actual/report."""
    output = []
    no_events = []
    rejected = []
    fixture = input_module().NG
    # Derived from the independently constructed Int64 schema in the actual
    # fixture context, never from a CaseId, error string or Rust output.
    accepted_reference = fixture.reference(fixture.schema('int64'))
    for case in inputs:
        if case["operation"] in failures:
            rejected.append({"case_id": case["case_id"],
                             "failure": failures[case["operation"]]})
            continue
        candidate = case["wire_candidate"]
        reached = []
        name = candidate["component_identity"]["name"]
        catalog = candidate["component_identity"]["catalog"]
        parsed_catalog = re.fullmatch(r'[a-z][a-z0-9-]*(?:\.[a-z][a-z0-9-]*)*@([1-9][0-9]*)', catalog)
        stop = (unicodedata.normalize('NFC', name) != name
                or re.fullmatch(r'[a-z0-9-]+(?:\.[a-z0-9-]+)*', name) is None
                or parsed_catalog is None
                or (parsed_catalog is not None and int(parsed_catalog[1]) > 4294967295))
        for direction, field in [("input", "inputs"), ("output", "outputs")]:
            previous = None
            if stop:
                break
            ordered = sorted(enumerate(candidate[field]),
                             key=lambda pair: pair[1]["port_no"])
            for ordinal, (original, port) in enumerate(ordered):
                number = port["port_no"]
                condition = ("zero" if number == 0 else "duplicate"
                             if previous and previous["port_no"] == number
                             else "positive_fresh")
                occurrence = {"direction": direction,
                              "original_ordinal": original,
                              "sorted_ordinal": ordinal, "port_no": number}
                row = {"case_id": case["case_id"], "occurrence": occurrence,
                       "condition": condition, "predecessor": previous}
                reached.append(row)
                if condition != "positive_fresh" or port['type'] != accepted_reference:
                    stop = True
                    break
                previous = {"original_ordinal": original, "port_no": number}
        if not reached:
            no_events.append({"case_id": case["case_id"], "reason":
                              "component_identity_before_ports" if stop
                              else "both_namespaces_empty"})
        output.extend(reached)
    assert len(inputs) == 87 and len(output) == 125
    assert len({e["case_id"] for e in output}) == 59
    assert len(no_events) == 4 and len(rejected) == 24
    return output, no_events, rejected


def refs(value):
    if isinstance(value, dict):
        if "ref" in value:
            yield value["ref"]
        for child in value.values():
            yield from refs(child)
    elif isinstance(value, list):
        for child in value:
            yield from refs(child)


def event_id(event):
    o = event["occurrence"]
    return [event["case_id"], o["direction"], o["original_ordinal"]]


def descriptor_members(types):
    """Finite descriptor graph, NOT a fabricated runtime-value expansion."""
    pending = ["Fact"]
    seen = set()
    result = []
    while pending:
        name = pending.pop(0)
        if name in seen:
            continue
        seen.add(name)
        descriptor = types[name]
        children = sorted(set(refs(descriptor)))
        pending.extend(n for n in children if n not in seen)
        result.append({"descriptor_owner": "agentflow.computation.proof-report.model@5",
                       "source_descriptor_path": MODEL_PATH,
                       "registered_type": name,
                       "descriptor": descriptor,
                       "references": children,
                       "value_instances": "NOT_ENUMERATED_BY_SHAPE_AUDIT",
                       "conformance": "NOT_PROVED"})
    return result


def root_manifest(m):
    source = {
        "key": "RawLocalCapture.{subject,target,origin}; LocalRawSeed.{event,predicate,kind,stage}",
        "role": "LocalRawSeed::role (local label only)",
        "evaluation": "LocalRawSeed::evaluated (bool, not Evaluation tagged value)",
        "status": "LocalRawSeed::status (ProofVerdict, not Status.proof wrapper)",
        "raw_validation_basis": "RawLocalCapture::basis; borrowed input, not full nominal projection",
    }
    conditions = {
        "contradiction": "Some raw_validation witness for zero/duplicate; None for positive_fresh",
        "raw_validation_basis": "Some for every raw_contract_validation fact",
        "substitution_basis": "None for this family, but option field is required",
        "native_guard_basis": "None for this family, but option field is required",
        "observations": "Set minimum 1; legitimate actual output observations only",
        "primary": "Some typed minimum required reference; not a six-root-boundary predicate",
        "prerequisites": "Actual queried premises only; no invented schema-resolution premise",
        "cause_fact_keys": "Actual supporting facts only; no invented pre-port Proven fact",
        "companion_evidence_status": "No independent authority/evidence companion for this local predicate",
        "references": "All actually required typed member references; duplicate includes both original ports",
    }
    output = []
    for name, descriptor in m["types"]["Fact"]["fields"]:
        output.append({
            "exact_field_name": name,
            "descriptor_owner": m["model_id"],
            "nominal_identity": {"owner": m["model_id"], "name": "Fact"},
            "exact_registered_type": descriptor,
            "source_descriptor": {"path": MODEL_PATH, "json_path": ["types", "Fact", "fields", name]},
            "required_optional_applicability": conditions.get(name, "Required for every reached event"),
            "actual_rust_source": "crates/af-computation/src/raw_capture.rs: " + source.get(name, "NOT MATERIALIZED"),
            "required_producer_stage": ("T3 typed inputs required; T4 canonical encoding deferred; descriptive carrier paths do not require a FactKey selector"
                                        if name in ("key", "observations") else
                                        "T3 typed semantic payload; T4 normalization is not implemented"),
            "current_local_mapping": "PARTIAL_LOCAL_MAPPING" if name in source and name != "key" else "NOT_MATERIALIZED",
            "nested_dependencies": sorted(set(refs(descriptor))),
            "independent_expected_method": "Registered descriptor + original source/compiler/context + reached branch; NOT Rust Actual",
            "comparator_status": "NO_COMPLETE_FORMAL_COMPARISON",
        })
    assert len(output) == 14
    assert sum(r["current_local_mapping"] == "NOT_MATERIALIZED" for r in output) == 10
    return output


def direct_instances(types, name, descriptor, path, selected=None):
    """Enumerate immediate registered slots, never pretend unknown lengths=0."""
    if "ref" in descriptor:
        name = descriptor["ref"]
        descriptor = types[name]
    kind = descriptor.get("kind")
    if kind == "record":
        return [{"registered_type": name, "field_path": path + [field],
                 "exact_registered_type": child,
                 "applicability": "REQUIRED_RECORD_SLOT",
                 "status": "NOT_FULLY_COMPARED"}
                for field, child in descriptor["fields"]]
    if kind == "enum":
        return [{"registered_type": name, "field_path": path + ["variant:" + case],
                 "exact_registered_type": child,
                 "applicability": "SELECTED" if case == selected else "CONDITIONAL_UNRESOLVED",
                 "status": "NOT_FULLY_COMPARED"}
                for case, child in descriptor["variants"] if selected is None or case == selected]
    if kind in ("set", "sequence", "map", "option", "dag_ref"):
        return [{"registered_type": name, "field_path": path,
                 "exact_registered_type": descriptor,
                 "applicability": "VALUE_EXPANSION_REQUIRED_NOT_ASSUMED_EMPTY",
                 "status": "NOT_FULLY_COMPARED"}]
    return []


def boundary(m):
    t = m["types"]
    fields = lambda n: dict(t[n]["fields"])
    variants = lambda n: dict(t[n]["variants"])
    chain = [
        ["Fact.observations", fields("Fact")["observations"]],
        ["ObservationSite.field_path", fields("ObservationSite")["field_path"]],
        ["RawValidationFragment.facts", fields("RawValidationFragment")["facts"]],
        ["MemberStep.member", variants("MemberStep")["member"]],
        ["TypedValue", t["TypedValue"]],
        ["CanonicalValueDag", t["CanonicalValueDag"]],
        ["FactKey", t["FactKey"]],
    ]
    assert fields("Fact")["observations"]["minimum"] == 1
    assert fields("RawValidationFragment")["facts"]["kind"] == "map"
    assert fields("RawValidationFragment")["facts"]["key"] == {"ref": "FactKey"}
    member_key = dict(variants("MemberStep")["member"]["fields"])["key"]
    assert member_key == {"ref": "TypedValue"}
    assert fields("TypedValue")["value"] == {"ref": "CanonicalValueDag"}
    # Distinguish a stage handoff question from an unexpressible raw source.
    assert set(fields("RawValidationSubject")) == {"owner_compiler", "input", "context"}
    assert "source_package" in fields("DocumentOccurrenceBinding")
    return {"candidate_id": "R2-STAGE-CANDIDATE-01", "outcome": "NOT_A_CONFIRMED_FORMAL_GAP",
            "classification": "CANONICAL_MAP_SELECTOR_IS_NOT_THE_ONLY_OBSERVATION_SITE",
            "first_affected_root": "observations",
            "first_unresolved_typed_field": "ObservationSite.field_path[].member.key.value",
            "exact_descriptor_chain": chain,
            "resolution": "Accepted typed-schema materialization section 3 allows descriptive report-carrier paths. Therefore the descriptor chain alone does not prove a mandatory canonical FactKey dependency in T3.",
            "not_claimed": ["model cannot express SourcePackage/Occurrence", "all observations must use a map selector",
                            "empty field_path is structurally forbidden", "a checker false-Proven", "new edition required"],
            "possible_implementation_routes_not_chosen": [
                "Explicitly bind a noncanonical borrowed FactKey selector as T3 input to T4's TypedValue construction, and define what full T3 comparison covers",
                "Identify a real registered observation output site that reaches this actual fact without a canonical map key, with complete source/site correspondence",
                "Explicitly authorize the needed canonical selector construction and clarify its boundary with the prohibited canonical FactKey encoding"],
            "prohibited_workarounds": ["empty path with no demonstrated actual site", "report index/UUID as origin",
                                        "Rust-only semantic replacement variant", "relabel Pending as Complete",
                                        "implement T4/G02 under a diagnostic adapter"]}


def build():
    m = model()
    inputs, failures = load_inputs()
    roster, no_events, rejected = events(inputs, failures)
    old = json.loads((ROOT / "docs/implementation/phase-04c1b4c4r3c2r1-expected-event-manifest.json").read_text(encoding="utf8"))
    old_events = {(e["case_id"], e["expected"]["occurrence"]["direction"],
                   e["expected"]["occurrence"]["original_ordinal"]): e["expected"] for e in old["events"]}
    assert set(old_events) == {tuple(event_id(e)) for e in roster}
    for e in roster:
        assert old_events[tuple(event_id(e))]["occurrence"] == e["occurrence"]
        assert old_events[tuple(event_id(e))]["condition"] == e["condition"]
        assert old_events[tuple(event_id(e))]["predecessor"] == e["predecessor"]
    roots = root_manifest(m)
    root_gaps = []
    nested_gaps = []
    for event in roster:
        for root in roots:
            name = root["exact_field_name"]
            root_gaps.append({"event_id": event_id(event), "root_field": name,
                              "exact_registered_type": root["exact_registered_type"],
                              "field_presence_required": True,
                              "known_option_case": ("some" if name in ("raw_validation_basis", "primary")
                                                    or (name == "contradiction" and event["condition"] != "positive_fresh")
                                                    else "none" if name in ("contradiction", "companion_evidence_status", "substitution_basis", "native_guard_basis")
                                                    else None),
                              "initial_status": root["current_local_mapping"],
                              "final_status": root["current_local_mapping"],
                              "closed_in_c2r2": False})
            selected = {"role": "supporting_fact", "evaluation": "evaluated", "status": "proof"}.get(name)
            for slot in direct_instances(m["types"], "Fact", root["exact_registered_type"], [name], selected):
                nested_gaps.append({"event_id": event_id(event), **slot})
        # Expand the direct borrowed basis as well as its required subject's
        # compiler/context. These are concrete per-event obligations, not a
        # claim to have enumerated every collection element in the full closure.
        for name, path in [
            ("RawPortCheckBasis", ["raw_validation_basis", "some"]),
            ("RawValidationSubject", ["raw_validation_basis", "some", "subject"]),
            ("Compiler", ["raw_validation_basis", "some", "subject", "owner_compiler"]),
            ("FullSupportManifest", ["raw_validation_basis", "some", "subject", "owner_compiler", "support_manifest"]),
            ("RawContextBinding", ["raw_validation_basis", "some", "subject", "context"]),
            ("ContractProofContextContents", ["raw_validation_basis", "some", "subject", "context", "contents"]),
        ]:
            for slot in direct_instances(m["types"], name, m["types"][name], path):
                nested_gaps.append({"event_id": event_id(event), **slot})
    missing = [r for r in root_gaps if r["initial_status"] == "NOT_MATERIALIZED"]
    partial = [r for r in root_gaps if r["initial_status"] == "PARTIAL_LOCAL_MAPPING"]
    assert len(missing) == 1250 and len(partial) == 500
    relation = [{"event_id": event_id(e), "field": name,
                 "expected": "NOT_MATERIALIZED_NO_FULL_ORACLE",
                 "actual": "NO_REGISTERED_RELATION_PROJECTION",
                 "required_applicability": "UNRESOLVED_NOT_EMPTY_SUCCESS"}
                for e in roster for name in ("prerequisites", "references", "primary", "observations")]
    return {
        "schema": "c2r2-diagnostic-audit-v1-not-a-formal-model",
        "start_head": BASE, "model": m["model_id"],
        "RawFactRootFieldManifest": roots,
        "RootFieldGapBaseline": {"required_instances": len(root_gaps),
                                 "unmaterialized_instances": len(missing),
                                 "partially_mapped_instances": len(partial),
                                 "closed_instances": 0, "instances": root_gaps},
        "NestedFieldGapBaseline": {"enumerated_direct_slot_obligations": len(nested_gaps),
                                   "direct_slot_categories_not_full_closure": {
                                       "compiler_and_support_manifest": 1750,
                                       "context_binding_and_contents": 1375,
                                       "raw_subject_direct_fields": 375,
                                       "raw_basis_direct_fields": 750,
                                       "root_referenced_direct_slots": 2875,
                                   },
                                   "full_value_closure_count": None,
                                   "full_value_closure_count_status": "UNDETERMINED_NOT_ZERO",
                                   "closed_instances": 0, "instances": nested_gaps},
        "RawNestedFieldManifest": descriptor_members(m["types"]),
        "RawEventExpectedManifest": {"scope": "unchanged primary C2R1 corpus",
                                     "cases": 59, "events": roster,
                                     "accepted_without_event": no_events,
                                     "rejected_before_port": rejected},
        "RawRootExpectedManifest": {"complete": False, "value_instances": [],
                                    "required_contract": "RawFactRootFieldManifest + RootFieldGapBaseline"},
        "RawNestedExpectedManifest": {"complete": False, "value_instances": [],
                                      "reason": "Stage handoff unresolved; descriptor census is not an Expected value oracle"},
        "RawReferenceExpectedManifest": {"complete": False, "instances": relation},
        "RawActualComparisonManifest": {"complete": False, "full_field_compared_instances": 0,
                                        "source": "Existing local capture is not a registered Fact projection"},
        "MutationManifest": {"new_full_field_mutations": [], "new_real_input_mutations": [],
                             "reason": "Stopped at R2-A/stage binding; no false conformance credit"},
        "StageBoundaryCandidateReview": boundary(m),
        "StrictGateResults": {"RawLocalTraceGate": "SEPARATELY_EXECUTED_SEE_RUNTIME_RECEIPT",
                              "RawRootFieldGate": "INCOMPLETE", "RawNestedFieldGate": "INCOMPLETE",
                              "RawRelationGate": "INCOMPLETE", "RawTotalStrictGate": 2,
                              "outcome": "D", "positive_root_export": "NotAuthorized"},
        "HistoricalIsolationReceipt": {"status": "PENDING_CURRENT_PHASE_EXECUTION",
                                       "historical_sources_modified": False},
        "OriginalDirectoryGeneratorByteMaps": {"status": "PENDING_FINAL_TREE_EXECUTION",
                                               "two_round_pass_claimed": False},
    }


def verify_preservation():
    """Every initial tracked byte must remain equal to its initial worktree byte."""
    baseline = ROOT / "target" / (PREFIX + "initial-byte-map.json")
    current_paths = git("ls-files", "-z").decode().split("\0")[:-1]
    if baseline.exists():
        rows = json.loads(baseline.read_text(encoding="utf8"))["files"]
    else:
        assert git("rev-parse", "HEAD").decode().strip() == BASE
        assert not git("diff", "--name-only")
        rows = {p: hashlib.sha256((ROOT / p).read_bytes()).hexdigest() for p in current_paths}
        baseline.parent.mkdir(exist_ok=True)
        baseline.write_text(json.dumps({"head": BASE, "files": rows}, indent=2) + "\n", encoding="utf8")
    changed = [p for p, digest in rows.items() if not (ROOT / p).is_file()
               or hashlib.sha256((ROOT / p).read_bytes()).hexdigest() != digest]
    assert not set(changed) - AUTHORIZED_EXISTING, changed
    return {"initial_tracked_paths": len(rows), "changed_initial_paths": changed,
            "authorized_existing_implementation_paths": sorted(AUTHORIZED_EXISTING),
            "unchanged_initial_paths": len(rows)-len(changed),
            "unauthorized_changed_paths": [],
            "initial_byte_map": str(baseline), "scope": "ALL initial tracked paths, not just normative files"}


def attach_subset(result):
    actual_path = ROOT / "target" / (PREFIX + "root-subset-actual.json")
    if not actual_path.exists():
        return
    spec = importlib.util.spec_from_file_location("c2r2_subset_oracle", ROOT / "tools/oracle/raw_root_subset.py")
    oracle = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(oracle)
    actual = json.loads(actual_path.read_text(encoding="utf8"))
    comparison = oracle.report(actual)
    expected = oracle.expected()
    implemented = set(oracle.IMPLEMENTED)
    assert comparison["compared_root_subset_instances"] == 1000
    for root in result["RawFactRootFieldManifest"]:
        name = root["exact_field_name"]
        root["final_mapping"] = "REGISTERED_SUBSET_COMPARED" if name in implemented else "INCOMPLETE"
        if name in implemented:
            root["comparator_status"] = "INPUT_DERIVED_TYPED_VALUE_COMPARISON_PASS"
    instances = result["RootFieldGapBaseline"]["instances"]
    for row in instances:
        if row["root_field"] in implemented:
            row["final_status"] = "REGISTERED_SUBSET_COMPARED"
            row["closed_in_c2r2"] = True
    result["RootFieldClosureLedger"] = {
        "required_instances": 1750,
        "compared_registered_subset_instances": 1000,
        "initially_missing_now_compared": sum(r["closed_in_c2r2"] and r["initial_status"] == "NOT_MATERIALIZED" for r in instances),
        "initially_partial_now_compared": sum(r["closed_in_c2r2"] and r["initial_status"] == "PARTIAL_LOCAL_MAPPING" for r in instances),
        "remaining_incomplete_instances": 750,
        "remaining_unmaterialized_instances": 625,
        "remaining_partial_basis_instances": 125,
        "complete": False,
    }
    result["RawRootExpectedManifest"] = {
        "complete": False, "source": "independent inputs + descriptors, NOT Rust Actual",
        "compared_registered_subset_instances": 1000,
        "value_instances": [{"event_id": [r["case_id"], r["occurrence"]["direction"], r["occurrence"]["original_ordinal"]],
                             "registered_roots": r["registered_roots"]} for r in expected],
        "missing_formal_roots": comparison["remaining_fields"],
    }
    result["RawNestedExpectedManifest"] = {
        "complete": False,
        "value_instances": [{"event_id": [r["case_id"], r["occurrence"]["direction"], r["occurrence"]["original_ordinal"]],
                             "registered_declared_path": r["registered_declared_path"],
                             "witness_subject_input": r["witness_subject_input"]} for r in expected],
        "not_claimed": "complete compiler/context/reference DAG or full formal Witness/Contradiction",
    }
    result["RawActualComparisonManifest"] = comparison
    result["MutationManifest"] = {"new_subset_mutations": comparison["mutations"],
                                  "new_full_field_mutations": [], "new_real_input_mutations": [],
                                  "does_not_close_full_field_conformance": True}
    result["StrictGateResults"]["RawRootFieldGate"] = "1000_SUBSET_INSTANCES_PASS_750_INCOMPLETE"
    sensitivity_path = ROOT / "target" / (PREFIX + "input-sensitivity-actual.json")
    if sensitivity_path.exists():
        actual_sensitivity = json.loads(sensitivity_path.read_text(encoding="utf8"))
        result["RealInputSensitivity"] = {
            "inputs": oracle.sensitivity_vectors(),
            "expected": oracle.sensitivity_expected(),
            "comparison": oracle.sensitivity_report(actual_sensitivity),
            "complete_formal_fact_sensitivity": False,
        }


def current_registration_receipts(m):
    """Current 254 cases, independently of older phases' whole-file freezes."""
    def read(path):
        return json.loads((ROOT / path).read_bytes())
    specifications = read("spec/registry/specifications.json")["specifications"]
    rules = read("spec/registry/rules.json")["rules"]
    assert (len(specifications), len(rules)) == (62, 493)
    for source in specifications:
        p = ROOT / source["normative_source"]["logical_path"]
        assert hashlib.sha256(p.read_bytes()).hexdigest() == source["normative_source"]["content_digest"]["hex"]
    assert (len(m["types"]), len(list(refs(m["types"]))), len(m["historical_selected_imports"])) == (704, 2064, 97)
    spec = importlib.util.spec_from_file_location("frozen_native_source_binding", ROOT/"tools/native_guard_source_binding.py")
    imported = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(imported)
    historical = imported.check(m, read("spec/conformance/g01-native-guard-staging/historical-imports.json"))
    assert historical["live_declarations_compared"] == 90
    d = "spec/conformance/g01-document-occurrence-staging/"
    target = "target/phase-04c1b4c4r3c2g2-"
    groups = []
    for source, output, ekey, akey in [
        ("shape-cases.json", "shape-actual.json", "expected_failure", "actual_failure"),
        ("admission-cases.json", "admission-actual.json", "expected", "actual"),
        ("dag-cases.json", "dag-actual.json", "expected_failure", "actual_failure"),
    ]:
        expected = {r["case_id"]: r[ekey] for r in read(d+source)["cases"]}
        actual = read(target+output)
        assert len(actual) == len(expected)
        assert {r["case_id"] for r in actual} == set(expected)
        assert all(r[akey] == expected[r["case_id"]] for r in actual)
        if source == "admission-cases.json":
            assert all(r["raw_facts_produced"] == 0 and not r["predicate_evaluated"] and not r["root_capability"] for r in actual)
        groups.append({"source": source, "matched": len(actual)})
    manifest = read(d+"manifest.json")
    assert len(manifest["case_ids"]) == 254
    for output, field in [("portable-actual.json", "portable_cases"), ("portable-runtime-actual.json", "runtime_portable_cases")]:
        actual = read(target+output)
        expected = {r["case_id"]: r for r in manifest[field]}
        assert len(actual) == len(expected) and {r["case_id"] for r in actual} == set(expected)
        assert all(r["stage"] == expected[r["case_id"]]["stage"] and r["family"] == expected[r["case_id"]]["family"] for r in actual)
        groups.append({"source": field, "matched": len(actual)})
    actual = read(target+"rule-actual.json")
    assert len(actual) == 14 and all(r["registration_checked"] and not r["predicate_evaluated"] for r in actual)
    groups.append({"source": "rule-actual.json", "matched": 14})
    assert sum(g["matched"] for g in groups) == 254
    return {"registration_case_ids": 254, "groups": groups,
            "specifications": 62, "rules": 493, "descriptors": 704,
            "references": 2064, "historical_imports": 97,
            "live_declarations_compared": 90,
            "is_full_field_capture_evidence": False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--materialize", action="store_true")
    parser.add_argument("--registration", action="store_true")
    args = parser.parse_args()
    preservation = verify_preservation()
    result = build()
    attach_subset(result)
    if args.registration:
        result["CurrentRegistrationReceipt"] = current_registration_receipts(model())
    result["FrozenInputPreservation"] = preservation
    if args.materialize:
        path = ROOT / "docs/implementation" / (PREFIX + "field-and-stage-audit.json")
        path.write_text(json.dumps(result, ensure_ascii=True, indent=2) + "\n", encoding="utf8")
    print(json.dumps({"outcome": "D", "stage_gap_confirmed": False,
                      "required_root_instances": result["RootFieldGapBaseline"]["required_instances"],
                      "initial_missing_root_instances": 1250, "initial_partial_root_instances": 500,
                      "final_incomplete_root_instances": result.get("RootFieldClosureLedger", {}).get("remaining_incomplete_instances", 1750),
                      "nested_direct_slots": result["NestedFieldGapBaseline"]["enumerated_direct_slot_obligations"],
                      "full_nested_count": None, "full_field_conformance": False,
                      "strict_exit": 2, "preservation": preservation}, indent=2))
    return 2


if __name__ == "__main__":
    sys.exit(main())
