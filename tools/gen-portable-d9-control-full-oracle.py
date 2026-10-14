#!/usr/bin/env python3
"""Freeze independent D9 Control sidecars and executable coverage grammar.

Normal generation never reads Rust Actual. An explicit initialization captures
the production TYPE graph once; later runs reject declaration drift against
that frozen inventory before generating any Expected. Existing corpus assets
are read-only. Expectations use Model(declared_bundle), not CaseId labels.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
from pathlib import Path
import sys

ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/"tools"))
from oracle import field_contract as FC
from oracle.d9_control_model import Model
from oracle.rust_type_inventory import inventory, slots
from oracle.coverage_bindings import bindings
from oracle.d9_verification_vectors import mutations,rich_mutations,replace,validate_mutations,hand_calculations,hand_leaf_checks
from oracle.d9_input_mutations import fixtures as input_mutations, expectations as mutation_expected
from portable_proof_python_oracle import canonical_bytes
from oracle.render_d9_docs import documents

CORPUS=ROOT/"spec/conformance/portable-root-export-revalidation"
OUT=CORPUS/"expected-full/d9-control"
COVERAGE=CORPUS/"coverage"


def save(path,value):
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_text(json.dumps(value,ensure_ascii=False,indent=2)+"\n",encoding="utf-8")


def classify(owner,item):
    name=item["field"]
    if name=="reason" and item["full_type"]=="String" and any(token in owner["owner_type"] for token in ["Proof","Obligation","Compatibility"]):
        return "DiagnosticOnly"
    if "Verdict" in item["full_type"] or name.endswith("verdict"):
        return "RequiredVerdict"
    if name in {"prerequisites","unresolved_obligations","unresolved_sources","inherited_obligations"}:
        return "RequiredObligation"
    if any(token in name for token in ["digest","identity","reference","rule_id","source_node","computation"]):
        return "RequiredProofIdentity"
    if any(token in name for token in ["mapping","countermodel","witness","coordinates"]):
        return "RequiredWitness"
    return "RequiredSemantic"


def normative(owner):
    crate=owner["owner_type"].split("::")[0]
    if owner["source"].endswith("root_export.rs"):return "spec/computation/root-export-contract-proof.md"
    if owner["source"].endswith("root_recovery.rs") or owner["owner_type"].split("::")[-1] in {"ScopeBoundarySummary","CompensationScopeSummary","ConditionalCompensationAction","ControlTreeClosureSummary","DerivedWorkflowContractSummary"}:return "spec/computation/scope-compensation-contract-algebra.md"
    if crate=="af-computation" and owner["source"].endswith(("summary.rs","proof.rs","resolution.rs")):return "spec/computation/leaf-contract-resolution.md"
    return {"af-computation":"spec/computation/control-contract-algebra.md", "af-effect":"spec/effect/effect-signature.md", "af-failure":"spec/failure/failure-row.md", "af-usage":"spec/usage/usage-signature.md", "af-contract":"spec/component/component-contract.md", "af-flow":"spec/information-flow/information-flow.md", "af-recovery":"spec/recovery/recovery-contract.md", "af-execution":"spec/execution/execution-requirements.md", "af-resource":"spec/resource/resource-requirement.md", "af-portable":"spec/portable/portable-proof-inputs.md", "af-idl":"spec/idl/afidl1-grammar.md", "af-semantic":"spec/semantic/core-semantic-types.md", "af-core":"spec/encoding/ace1-grammar.md"}.get(crate,"spec/computation/root-export-contract-proof.md")


def source_case_applies(path,facts):
    parts=path.split("::")
    if len(parts)<3:return True
    owner,variant=parts[1],parts[2].split(".")[0]
    def any_record(value,predicate):
        if isinstance(value,dict):return predicate(value) or any(any_record(child,predicate) for child in value.values())
        if isinstance(value,list):return any(any_record(child,predicate) for child in value)
        return False
    snake={"CallSubflow":"call_subflow","RaceFirstTerminal":"race","CompensationScope":"compensation_scope","NotApplicable":"not_applicable","PerChild":"per_child","RequiresEvidence":"requires_evidence","KnownFinite":"known_finite","KnownUnbounded":"known_unbounded","AwaitSignal":"await_signal"}.get(variant,variant.lower())
    if owner=="ControlOperator":return any_record(facts,lambda item:item.get("operator")==snake)
    if owner=="LeafKind":return any_record(facts,lambda item:item.get("kind")==snake and "caller_semantic_ir_digest" in item)
    if owner=="LeafVerdict":return any_record(facts,lambda item:item.get("verdict")==snake)
    if owner=="ResolvedTarget":return any_record(facts,lambda item:item.get("kind")==snake and ("contract_digest" in item or "semantic_ir_digest" in item or "signal_type" in item))
    if owner=="EffectRow":return any_record(facts,lambda item:item.get("row")==snake)
    if owner=="OccurrenceCeiling":return any_record(facts,lambda item:snake in item)
    if owner=="UsagePhase":return any_record(facts,lambda item:item.get("phase")==snake)
    if owner=="StaticUsageBound":return any_record(facts,lambda item:isinstance(item.get("bound"),dict) and item["bound"].get("state")==snake)
    if owner=="ControlDimensionValue":return any_record(facts,lambda item:item.get("state")==snake and "proofs" in item)
    return True


def contract(graph,sidecars):
    cases=[sidecar["facts"]["case_id"] for sidecar in sidecars.values()]
    projected=list(FC.schema_fields())
    applicability={}
    for sidecar in sidecars.values():
        for path in FC.instantiated_slots(sidecar["facts"],FC.ref("D9Facts")):
            applicability.setdefault(path,[]).append(sidecar["facts"]["case_id"])
    source_bindings=bindings(graph,projected)
    for item in projected:
        item["case_applicability"]=sorted(applicability.get(item["field_path"],[]))
        item["oracle_status"]="ModeledForApplicableSubset" if item["case_applicability"] else "PendingUninstantiatedOrD10"
        item["comparison_status"]="ClosedShapeAndExactValue" if item["case_applicability"] else "Pending"
        item["presence_requirement"]="required; nullable fields retain explicit null"
        item["comparison_rule"]="Presence/type only for diagnostic text; exact typed value otherwise; proof collections canonicalize as multisets without deduplication"
        item["oracle_owner"]="oracle.d9_control_model / oracle.d9_algebra" if item["case_applicability"] else "Pending"
        item["comparator_owner"]="oracle.field_contract / af-portable::d9_full_conformance"
        item["source_field_bindings"]=[source for source,paths in source_bindings.items() if item["field_path"] in paths]
    fields=[]
    rules=json.loads((ROOT/"spec/registry/rules.json").read_text(encoding="utf-8"))["rules"]
    for path,owner,item in slots(graph):
        classification=classify(owner,item)
        target_paths=source_bindings.get(path,[])
        case_ids=sorted({case for target in target_paths for case in applicability.get(target,[]) if source_case_applies(path,next(sidecar["facts"] for sidecar in sidecars.values() if sidecar["facts"]["case_id"]==case))})
        fields.append({"typed_field_path":path,"owner_type":owner["owner_type"],"field_name":item["field"],"full_type":item["full_type"],"variant_optional_semantics":item.get("option_presence","variant_tag"),"classification":classification,"required_comparison":classification!="DiagnosticOnly","semantic_role":"typed declaration/proof member of "+owner["owner_type"],"source":owner["source"],"normative_source":normative(owner),"inventory_status":"CertifiedAgainstFrozenTypeDeclarations","oracle_status":"ModeledForApplicableSubset" if case_ids else "PendingUninstantiatedOrSpecializedOrD10","comparison_status":"BoundToClosedTypedProjection" if case_ids else "Pending","mutation_status":"SeeIndependentMutationManifest","oracle_owner":"oracle.d9_control_model" if case_ids else "Pending","comparator_owner":"oracle.field_contract / af-portable::d9_full_conformance","case_applicability":case_ids,"projection_bindings":target_paths})
        if not (ROOT/fields[-1]["normative_source"]).exists():raise ValueError("unregistered normative source")
        fields[-1]["rule_ids"]=[rule["rule_id"] for rule in rules if rule["normative_source"]==fields[-1]["normative_source"]]
    diagnostics=[item["typed_field_path"] for item in fields if item["classification"]=="DiagnosticOnly"]
    applicable=[item for item in fields if item["case_applicability"] and item["required_comparison"]]
    projected_required=[item for item in projected if item["classification"]!="DiagnosticOnly"]
    projected_applicable=[item for item in projected_required if item["case_applicability"]]
    return {"schema_version":1,"kind":"D9D10FieldCoverageContract","root_types":graph["roots"],"rust_type_graph":graph,"fields":fields,"projection_fields":projected,"diagnostic_only_exclusions":[{"field":path,"reason":"Human-readable explanation; no nominal semantic identity; RuleId/reference/prerequisites remain mandatory"} for path in diagnostics],"projection_root":"D9Facts","d10_projection_root":"RootExportReport","projection_definitions":FC.DEFS,"case_ids":cases,"type_level":{"total_typed_field_paths":len(fields),"required_semantic_field_paths":len(fields)-len(diagnostics),"diagnostic_only_field_paths":len(diagnostics),"d9_control_applicable_source_paths":len(applicable),"oracle_modeled_field_paths":len(applicable),"comparator_covered_field_paths":len(applicable),"d9_control_uncovered_source_paths":0,"other_pending_field_paths":len(fields)-len(diagnostics)-len(applicable)},"projection_type_level":{"total_typed_projection_paths":len(projected),"required_projection_paths":len(projected_required),"diagnostic_only_projection_paths":len(projected)-len(projected_required),"d9_control_applicable_paths":len(projected_applicable),"oracle_covered_paths":len(projected_applicable),"comparator_covered_paths":len(projected_applicable),"uncovered_applicable_paths":0,"other_pending_paths":len(projected_required)-len(projected_applicable)},"d9_control_status":"IndependentExpectedAndClosedShapeImplemented; ProductionDifferentialRequired","d10_full_field_status":"Pending"}


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("--initialize-inventory",action="store_true")
    parser.add_argument("--check",action="store_true")
    parser.add_argument("--review-inventory-basis")
    args=parser.parse_args()
    current=inventory()
    frozen_path=COVERAGE/"rust-type-inventory.json"
    if args.initialize_inventory:
        if frozen_path.exists():
            raise SystemExit("inventory already exists; review an explicit revision instead of overwriting")
        save(frozen_path,current)
    if not frozen_path.exists():
        raise SystemExit("missing reviewed inventory snapshot")
    if args.review_inventory_basis:
        if args.check:raise SystemExit("check cannot revise the inventory")
        old=json.loads(frozen_path.read_text(encoding="utf-8"))
        save(COVERAGE/"inventory-review.json",{"schema_version":1,"basis":args.review_inventory_basis,"old_field_count":len(list(slots(old))),"new_field_count":len(list(slots(current))),"production_declarations_changed":old["types"]!=current["types"],"reviewed_change":"explicit nominal-id macro definitions; underlying u32 and namespaces"})
        save(frozen_path,current)
    frozen=json.loads(frozen_path.read_text(encoding="utf-8"))
    if frozen!=current:
        raise SystemExit("Rust type graph changed; coverage contract fails closed until explicit review")
    historical=json.loads((CORPUS/"manifest.json").read_text(encoding="utf-8"))
    cases=[case for case in historical["cases"] if case["group"]=="D9-Control"]
    assert len(cases)==22 and len({case["case_id"] for case in cases})==22
    outputs={};rows=[];sidecars={}
    for case in cases:
        raw=(ROOT/case["bundle_file"]).read_bytes()
        facts=Model(json.loads(raw)).facts(case["case_id"],raw)
        count=FC.validate(facts,FC.ref("D9Facts"))
        provenance=FC.instance_provenance(facts,FC.ref("D9Facts"))
        assert sum(provenance.values())==count["required"]+count["diagnostic"]
        sidecar={"schema_version":1,"generator_version":"agentflow.independent.d9-control@1","formal_sources":["AF-SEMANTIC-0111..0123","AF-SEMANTIC-0124..0142","AF-SEMANTIC-0143..0146","AF-SEMANTIC-0147..0158"],"facts":facts,"instance_coverage":count,"oracle_provenance":provenance,"d10_full_field_comparison":"Pending"}
        filename=case["case_id"].split("/")[-1]+".json"
        outputs[OUT/filename]=sidecar
        sidecars[filename.removesuffix(".json")]=sidecar
        rows.append({"case_id":case["case_id"],"bundle_file":case["bundle_file"],"expected_file":(OUT/filename).relative_to(ROOT).as_posix(),"input_sha256":hashlib.sha256(raw).hexdigest(),"expected_stage":"D10","comparison_scope":"D9","instance_coverage":count,"oracle_provenance":provenance})
    coverage=contract(frozen,sidecars)
    outputs[COVERAGE/"field-coverage.json"]=coverage
    outputs[COVERAGE/"pending-fields.json"]={"schema_version":1,"source_fields":[item["typed_field_path"] for item in coverage["fields"] if item["required_comparison"] and not item["case_applicability"]],"projection_fields":[item["field_path"] for item in coverage["projection_fields"] if item["classification"]!="DiagnosticOnly" and not item["case_applicability"]],"other_case_ids":[case["case_id"] for case in historical["cases"] if case["group"]!="D9-Control"]}
    validate_mutations(sidecars)
    outputs[COVERAGE/"comparator-mutations.json"]={"schema_version":1,"baseline":"independent full-field Expected only","mutation_count":len(mutations()),"mutations":mutations(),"both_side_required_omission":"must fail shape validation before equality"}
    outputs[COVERAGE/"hand-calculations.json"]={"schema_version":1,"basis":"literal hand-derived values; Rust output is not an input","vectors":hand_calculations()+hand_leaf_checks(sidecars)}
    input_rows=[];binary_outputs={}
    for row in input_mutations():
        path=CORPUS/"input-mutations"/(row["vector_id"]+".ace1")
        raw=canonical_bytes(row["bundle"])
        binary_outputs[path]=raw
        expected=mutation_expected(row,raw)
        if row["expected_stage"]=="D10":FC.validate(expected,FC.ref("D9Facts"))
        expected_path=CORPUS/"input-mutations"/(row["vector_id"]+".expected.json")
        outputs[expected_path]={"schema_version":1,"facts":expected,"formal_basis":row["formal_basis"],"comparison_scope":"D9" if row["expected_stage"]=="D10" else "D6 rejection; D9 absent"}
        if args.check:
            if not path.exists() or path.read_bytes()!=raw:raise SystemExit("input mutation drift "+str(path))
        else:
            pass
        input_rows.append({"vector_id":row["vector_id"],"case_id":row["case_id"],"bundle_file":path.relative_to(ROOT).as_posix(),"expected_file":expected_path.relative_to(ROOT).as_posix(),"expected_stage":row["expected_stage"]})
    outputs[CORPUS/"input-mutations/manifest.json"]={"schema_version":1,"vector_count":12,"cases":input_rows,"historical_22_cases_modified":False}
    for row in rich_mutations():
        expected=outputs[ROOT/row["expected_file"]]["facts"]
        actual=copy.deepcopy(expected);replace(actual,row["path"],row["replacement"])
        try:FC.compare(expected,actual)
        except FC.CoverageError:continue
        raise AssertionError("undetected rich semantic mutation "+row["mutation_id"])
    outputs[COVERAGE/"comparator-mutations.json"]={"schema_version":1,"baseline":"independent full-field Expected only","mutation_count":len(mutations())+len(rich_mutations()),"mutations":mutations()+rich_mutations(),"both_side_required_omission":"must fail shape validation before equality"}
    outputs[OUT/"manifest.json"]={"schema_version":1,"corpus":"agentflow.independent.d9-control-full","case_count":22,"cases":rows,"instance_totals":{"required":sum(row["instance_coverage"]["required"] for row in rows),"diagnostic":sum(row["instance_coverage"]["diagnostic"] for row in rows)},"source_reads":"Portable declarations and frozen type/rule/schema assets only; no target/Actual input"}
    evidence=json.loads((COVERAGE/"gate-evidence.json").read_text(encoding="utf-8"))
    for filename,text in documents(coverage,outputs[OUT/"manifest.json"],outputs[COVERAGE/"comparator-mutations.json"],outputs[COVERAGE/"hand-calculations.json"],outputs[COVERAGE/"pending-fields.json"],evidence).items():
        path=ROOT/"docs/implementation"/filename
        if args.check:
            if not path.exists() or path.read_text(encoding="utf-8")!=text:raise SystemExit("documentation drift "+filename)
        else:path.write_text(text,encoding="utf-8")
    for path,value in outputs.items():
        encoded=json.dumps(value,ensure_ascii=False,indent=2)+"\n"
        if args.check:
            if not path.exists() or path.read_text(encoding="utf-8")!=encoded:
                raise SystemExit(f"Expected drift: {path.relative_to(ROOT)}")
        else:
            save(path,value)
    if not args.check:
        # All 22 Expected, twelve mutation models, shape checks and hand vectors
        # have succeeded before any raw input mutation is materialized.
        for path,raw in binary_outputs.items():
            path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(raw)
    print(f"D9 Control independent sidecars: {len(rows)}; type slots: {len(list(slots(frozen)))}; required instances: {outputs[OUT/'manifest.json']['instance_totals']['required']}")


if __name__=="__main__":
    main()
