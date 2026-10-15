"""Read-only R2 row/descriptor/frozen-byte audit. Not full G01 conformance."""
import hashlib,json,re,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
BASE="51239a0045d7a6fb274bb24cdc8130f2346bf085"
def js(p):return json.loads((ROOT/p).read_bytes())
def main():
 start=js("target/phase-04c1b4c4r2-baseline.json")
 allowed={"Cargo.lock","crates/af-computation/Cargo.toml","crates/af-computation/src/lib.rs","crates/af-computation/src/root_export.rs","crates/af-portable/Cargo.toml","crates/af-portable/src/support.rs","spec/registry/rules.json","spec/registry/specifications.json","tools/gen-g01-edition2-staging-corpus.py","tools/gen-registry-instances.py"}
 changed=[p for p,h in start.items() if not (ROOT/p).is_file() or hashlib.sha256((ROOT/p).read_bytes()).hexdigest()!=h]
 assert not(set(changed)-allowed),set(changed)-allowed
 specs=js("spec/registry/specifications.json")["specifications"];rules=js("spec/registry/rules.json")["rules"]
 for name,key,rows,identifier,count in [("specifications","specifications",specs,"specification_id",46),("rules","rules",rules,"rule_id",436)]:
  old=js(f"spec/conformance/g01-successor-staging/edition2-{name}.json")[key];assert len(old)==count
  by={x[identifier]:x for x in rows};assert all(by[x[identifier]]==x for x in old)
 model_text=(ROOT/"spec/tables/proof-report-model-v2.md").read_text(encoding="utf-8")
 model=json.loads(re.search(r"```json\s*(.*?)\s*```",model_text,re.S)[1]);types=model["types"]
 pending=list(types.values());refs=0
 while pending:
  v=pending.pop()
  if isinstance(v,dict):
   assert "_generic" not in v and "_case" not in v
   if "ref" in v:assert v["ref"] in types,v;refs+=1
   pending.extend(v.values())
  elif isinstance(v,list):pending.extend(v)
 cases=model["case_descriptors"];dims=model["dimension_descriptors"]
 assert len(cases)==27 and len(dims)==12
 assert len({x["case"] for x in cases})==27
 assert len(types["SubstitutionClause"]["variants"])==26
 error_counts={d:len(rows) for d,rows in model["dependency_errors"].items()};assert list(error_counts.values())==[12,11,11,13,22]
 assert all(x[0]!="aggregate" for x in types["NonaggregateSubstitutionWitness"]["variants"])
 ctx=types["ContractProofContextContents"];assert len(ctx["fields"])==9 and all("resolver" not in x[0] for x in ctx["fields"])
 for row in specs:
  source=row["normative_source"];assert hashlib.sha256((ROOT/source["logical_path"]).read_bytes()).hexdigest()==source["content_digest"]["hex"]
 corpus=js("spec/conformance/g01-successor-staging/manifest.json");ids=corpus["case_ids"];assert len(ids)==len(set(ids))==173
 source_review=[]
 for row in corpus["sources"]:
  raw=(ROOT/row["path"]).read_bytes();assert hashlib.sha256(raw).hexdigest()==row["sha256"];source_review.append(row)
 frozen=sum(p.startswith(("spec/conformance/","spec/idl/")) and p not in changed for p in start)
 out={"baseline":BASE,"old_spec_rows_equal":46,"old_rule_rows_equal":436,"original_files":len(start),"authorized_changed_original_paths":changed,"unauthorized_changed_original_paths":[],"unchanged_original_file_count":len(start)-len(changed),"frozen_conformance_and_afidl_files_unchanged":frozen,"actual_specifications":len(specs),"actual_rules":len(rules),"descriptor_types":len(types),"resolved_ref_occurrences":refs,"closed_cases":len(cases),"non_context_clauses":26,"dimensions":len(dims),"error_counts":error_counts,"new_staging_cases":len(ids),"new_source_bindings":source_review,"new_generator_count":len(list((ROOT/"tools").glob("gen-*.py"))),"producer":"NOT IMPLEMENTED","normalizer":"NOT IMPLEMENTED","positive_capability":"NOT AUTHORIZED"}
 print(json.dumps(out,indent=2))
if __name__=="__main__":main()
