"""Independent registration/freeze/receipt audit; NOT a Raw Producer Oracle."""
import hashlib,json,re,subprocess
from pathlib import Path
from document_occurrence_registration_data import BASELINE,MODEL,OWNER,SOURCES,RULES
R=Path(__file__).resolve().parent.parent
D=R/'spec/conformance/g01-document-occurrence-staging'
P=R/'target/phase-04c1b4c4r3c2g2-'
ALLOWED={'crates/af-computation/Cargo.toml','crates/af-computation/src/lib.rs','crates/af-computation/src/root_export.rs','crates/af-computation/src/registered_model.rs','crates/af-computation/src/raw_admission.rs','crates/af-portable/Cargo.toml','crates/af-portable/src/support.rs','tools/gen-registry-instances.py','tools/gen-raw-validation-staging.py','spec/registry/specifications.json','spec/registry/rules.json'}
def read(p):return json.loads(p.read_bytes())
def model(p):return json.loads(re.search(r'```json\s*(.*?)\s*```',p.read_text(encoding='utf-8'),re.S)[1])
def refs(x):
 if isinstance(x,dict):
  if 'ref' in x:yield x['ref']
  for v in x.values():yield from refs(v)
 elif isinstance(x,list):
  for v in x:yield from refs(v)
def main():
 frozen_file=R/'docs/implementation/phase-04c1b4c4r3c2g2-frozen-inputs.json'
 frozen=read(frozen_file)['frozen_paths'] if frozen_file.exists() else {k:v for k,v in read(Path(str(P)+'initial-raw-map.json')).items() if k not in ALLOWED}
 assert len(frozen)==2076-len(ALLOWED)
 for path,digest in frozen.items():assert hashlib.sha256((R/path).read_bytes()).hexdigest()==digest,('frozen raw bytes',path)
 original=set(subprocess.check_output(['git','ls-tree','-r','--name-only',BASELINE],cwd=R,text=True).splitlines())
 assert original==set(frozen)|ALLOWED
 modified=set(subprocess.check_output(['git','diff',BASELINE,'--name-only'],cwd=R,text=True).splitlines())&original
 assert modified<=ALLOWED,modified-ALLOWED
 specs=read(R/'spec/registry/specifications.json')['specifications'];rules=read(R/'spec/registry/rules.json')['rules']
 for name,key,count in [('specifications','specification_id',58),('rules','rule_id',479)]:
  old=read(D/f'edition5-{name}.json')[name];now=specs if name=='specifications' else rules
  assert len(old)==count and all(next((n for n in now if n[key]==s[key]),None)==s for s in old)
 for row in specs:assert hashlib.sha256((R/row['normative_source']['logical_path']).read_bytes()).hexdigest()==row['normative_source']['content_digest']['hex']
 m=model(R/SOURCES['RM'][2]);old=model(R/'spec/tables/proof-report-model-v4.md');t=m['types']
 assert m['inherited_registered_model']==old
 assert m['historical_selected_imports']==old['historical_selected_imports'] and len(m['historical_selected_imports'])==97
 assert m['historical_import_sources']==old['historical_import_sources']
 assert m['raw_case_descriptors']==old['raw_case_descriptors'] and m['native_case_descriptors']==old['native_case_descriptors'] and m['case_descriptors']==old['case_descriptors']
 replacement=set(m['document_occurrence_registration']['replaced_types']);authored=m['document_occurrence_registration']['authored_types']
 for name,desc in old['types'].items():
  assert m['type_ranks'][name]==old['type_ranks'][name]
  if name not in replacement:assert t[name]==desc,name
 assert t['RawValidationAdmissionFailure']['variants'][:-1]==old['types']['RawValidationAdmissionFailure']['variants']
 allrefs=list(refs(t));assert all(r in t for r in allrefs)
 # Closed input dependency graph must never lead back into generated output.
 forbidden={'Fact','FactKey','Scope','Obligation','TypedContradiction','ObservationSite','RawPortCheckBasis','RawPortRejectionWitness'}
 reachable=set();active=set()
 def visit(n):
  assert n not in forbidden,('input-output backedge',n)
  assert n not in active,('input descriptor cycle',n)
  if n in reachable:return
  active.add(n)
  for child in refs(t[n]):visit(child)
  active.remove(n);reachable.add(n)
 visit('SourcePackage');visit('DocumentOccurrenceBinding');visit('SourcePackageMetadata')
 shape=read(D/'shape-cases.json');required=[]
 for name in authored+sorted(replacement):
  desc=t[name]
  for field,_ in desc.get('fields',[]):required.append((name,None,field))
  for tag,payload in desc.get('variants',[]):
   required.append((name,tag,None))
   for field,_ in payload.get('fields',[]):required.append((name,tag,field))
 coverage={(x['type'],x.get('variant'),x.get('field')):x['case_id'] for x in shape['coverage']}
 assert set(coverage)==set(required)
 actual=read(Path(str(P)+'shape-actual.json'));assert {x['case_id'] for x in actual}=={x['case_id'] for x in shape['cases']}
 expected={x['case_id']:x['expected_failure'] for x in shape['cases']}
 assert all(x['actual_failure']==expected[x['case_id']] for x in actual)
 assert all(cid in expected for cid in coverage.values())
 admission=read(D/'admission-cases.json')['cases'];seen=read(Path(str(P)+'admission-actual.json'))
 assert {x['case_id'] for x in seen}=={x['case_id'] for x in admission}
 expected={x['case_id']:x['expected'] for x in admission}
 assert all(x['actual']==expected[x['case_id']] and x['raw_facts_produced']==0 and not x['predicate_evaluated'] and not x['root_capability'] for x in seen)
 manifest=read(D/'manifest.json');assert len(manifest['case_ids'])==len(set(manifest['case_ids']))
 for file,group in [('portable-actual.json','portable_cases'),('portable-runtime-actual.json','runtime_portable_cases')]:
  data=read(Path(str(P)+file));expected={r['case_id']:r for r in manifest[group]}
  assert {r['case_id'] for r in data}==set(expected)
  assert all(r['stage']==expected[r['case_id']]['stage'] and r['family']==expected[r['case_id']]['family'] for r in data)
 dags=read(D/'dag-cases.json')['cases'];dag_actual=read(Path(str(P)+'dag-actual.json'))
 expected={r['case_id']:r['expected_failure'] for r in dags}
 assert {r['case_id'] for r in dag_actual}==set(expected)
 assert all(r['actual_failure']==expected[r['case_id']] and not r['full_scope_checked'] and not r['normalized_identity_produced'] for r in dag_actual)
 plan=read(D/'design-conversion-plan.json')['vectors'];assert len(plan)==48
 conversion=[];admitted={r['case_id']:r for r in seen}
 for vector in plan:
  assert vector['original_actual_status']=='Not Executed'
  links=vector['registered_case_ids']
  assert all(cid in admitted for cid in links)
  conversion.append({**vector,'actual_status':'Executed — Metadata Admission Only' if links else 'Not Executed','actual_results':[admitted[cid] for cid in links],'producer_conformance':False})
 Path(str(P)+'design-conversion-actual.json').write_text(json.dumps({'vectors':conversion,'executed_metadata_only':sum(bool(v['registered_case_ids']) for v in plan),'not_executed':sum(not v['registered_case_ids'] for v in plan)},ensure_ascii=True,indent=2)+'\n',encoding='utf-8')
 for r in RULES:
  row=next(x for x in rules if x['rule_id']==r['rule_id']);source=R/SOURCES[r['source']][2]
  assert row['short_statement']==r['statement'] and r['case_id'] in row['conformance_refs']
  s=source.read_text(encoding='utf-8');assert s.count('id="'+r['anchor']+'"')==1 and r['statement'] in s
 rule_actual=read(Path(str(P)+'rule-actual.json'))
 assert {x['case_id'] for x in rule_actual}=={x['case_id'] for x in RULES}
 assert all(x['registration_checked'] and not x['predicate_evaluated'] for x in rule_actual)
 mapping=read(D/'registration-map.json')['clauses'];assert len(mapping)==14
 for link in mapping:
  assert link['model_owner']==MODEL and link['compiler_semantics']==OWNER
  assert all(n in t for n in link['descriptor_names'])
  assert all(cid in manifest['case_ids'] for cid in link['conformance_case_ids'])
 # Existing exact historical declaration/import audit, unchanged semantic Oracle.
 from native_guard_source_binding import check
 historic=check(old,read(R/'spec/conformance/g01-native-guard-staging/historical-imports.json'))
 assert historic['live_declarations_compared']==90
 result={'kind':'Registration/shape/metadata receipt audit; NOT Capture','baseline':BASELINE,
  'frozen_original_raw_paths':len(frozen),'reviewed_existing_changes':sorted(modified),'old_spec_records_full_equal':58,'old_rule_records_full_equal':479,
  'specifications':len(specs),'rules':len(rules),'new_specs':len(specs)-58,'new_rules':len(rules)-479,
  'descriptors':len(t),'descriptor_references':len(allrefs),'new_descriptors':len(t)-683,'new_references':len(allrefs)-2018,
  'old_ranks_preserved':683,'historical_selected_imports':97,'live_declarations_compared':90,'required_field_variant_sites':len(required),'covered_sites':len(coverage),'field_variant_gaps':0,
  'shape_case_ids':len(shape['cases']),'admission_case_ids':len(admission),'registration_case_ids':len(manifest['case_ids']),
  'dag_subgraph_cases':len(dags),'portable_cases':len(manifest['portable_cases'])+len(manifest['runtime_portable_cases']),'design_vectors_metadata_only_executed':sum(bool(v['registered_case_ids']) for v in plan),'design_vectors_not_executed':sum(not v['registered_case_ids'] for v in plan),
  'source_input_descriptor_closure':len(reachable),'new_raw_capture_case_ids':0,'raw_producer':'NotImplemented','positive_root_export':'NotAuthorized'}
 print(json.dumps(result,indent=2))
if __name__=='__main__':main()
