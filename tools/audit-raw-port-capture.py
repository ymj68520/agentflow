"""C2R1 current registration + source freeze + explicitly partial raw audit.

Does NOT waive or edit C2G2's historical phase-specific whole-file freeze.
Rechecks current receipts under the new explicitly authorized implementation
delta. Full Raw conformance remains a separate, failing (exit 2) gate.
"""
import hashlib,importlib.util,json,re,subprocess,sys
from pathlib import Path
R=Path(__file__).resolve().parent.parent
P='phase-04c1b4c4r3c2r1-'
ALLOWED={'crates/af-computation/Cargo.toml','crates/af-computation/src/lib.rs','crates/af-computation/src/native_capture.rs','crates/af-contract/src/canonical.rs','crates/af-contract/src/lib.rs'}
def read(p):return json.loads(p.read_bytes())
def model(p):return json.loads(re.search(r'```json\s*(.*?)\s*```',p.read_text(encoding='utf8'),re.S)[1])
def run():
 committed=R/'docs/implementation'/(P+'frozen-inputs.json')
 frozen=read(committed)['initial_paths'] if committed.exists() else read(R/'target'/(P+'initial-raw-map.json'))
 assert len(frozen)==2118
 for path,sha in frozen.items():
  if path not in ALLOWED:assert hashlib.sha256((R/path).read_bytes()).hexdigest()==sha,('unauthorized original raw change',path)
 specs=read(R/'spec/registry/specifications.json')['specifications'];rules=read(R/'spec/registry/rules.json')['rules']
 assert (len(specs),len(rules))==(62,493)
 for s in specs:assert hashlib.sha256((R/s['normative_source']['logical_path']).read_bytes()).hexdigest()==s['normative_source']['content_digest']['hex']
 m=model(R/'spec/tables/proof-report-model-v5.md');t=m['types'];refs=[]
 def walk(x):
  if isinstance(x,dict):
   if 'ref'in x:refs.append(x['ref'])
   for v in x.values():walk(v)
  elif isinstance(x,list):
   for v in x:walk(v)
 walk(t);assert (len(t),len(refs),len(m['historical_selected_imports']))==(704,2064,97)
 from native_guard_source_binding import check
 imports=check(m,read(R/'spec/conformance/g01-native-guard-staging/historical-imports.json'));assert imports['live_declarations_compared']==90
 d=R/'spec/conformance/g01-document-occurrence-staging';target=R/'target';g2='phase-04c1b4c4r3c2g2-'
 groups=[]
 for source,actual,expected_key,actual_key in [('shape-cases.json','shape-actual.json','expected_failure','actual_failure'),('admission-cases.json','admission-actual.json','expected','actual'),('dag-cases.json','dag-actual.json','expected_failure','actual_failure')]:
  expected={r['case_id']:r[expected_key] for r in read(d/source)['cases']};seen=read(target/(g2+actual))
  assert len(seen)==len(expected) and {r['case_id'] for r in seen}==set(expected)
  assert all(r[actual_key]==expected[r['case_id']] for r in seen)
  groups.append({'source':source,'case_ids':len(expected),'matched':len(seen)})
  if source=='admission-cases.json':assert all(r['raw_facts_produced']==0 and not r['predicate_evaluated'] and not r['root_capability'] for r in seen)
 manifest=read(d/'manifest.json');assert len(manifest['case_ids'])==254
 for file,key in [('portable-actual.json','portable_cases'),('portable-runtime-actual.json','runtime_portable_cases')]:
  seen=read(target/(g2+file));want={r['case_id']:r for r in manifest[key]};assert len(seen)==len(want)
  assert {r['case_id'] for r in seen}==set(want) and all(r['stage']==want[r['case_id']]['stage'] and r['family']==want[r['case_id']]['family'] for r in seen)
  groups.append({'source':key,'case_ids':len(want),'matched':len(seen)})
 rule_actual=read(target/(g2+'rule-actual.json'));assert len(rule_actual)==14 and all(r['registration_checked'] and not r['predicate_evaluated'] for r in rule_actual)
 expected_rules=read(d/'registration-map.json')['clauses'];assert len(expected_rules)==14
 groups.append({'source':'rule-actual.json','case_ids':14,'matched':14});assert sum(r['matched'] for r in groups)==254
 raw=read(target/(P+'raw-actual.json'))
 spec=importlib.util.spec_from_file_location('raw_port_independent',R/'tools/oracle/raw_port_capture.py');oracle=importlib.util.module_from_spec(spec);spec.loader.exec_module(oracle)
 report=oracle.report(raw);assert report['outcome'].startswith('D') and report['registered_fact_field_contract']['complete'] is False
 return {'kind':'Current registration + authorized raw implementation freeze + partial event conformance','baseline':'23a25b69d65166be3ec870ec1177ddd6ea78dc69','frozen_original_paths':len(frozen)-len(ALLOWED),'reviewed_existing_implementation_paths':sorted(ALLOWED),
  'specifications':62,'rules':493,'descriptors':704,'descriptor_references':2064,'historical_imports':97,'live_declarations':90,'registration_case_ids':254,'registration_receipt_groups':groups,
  'raw':report,'full_raw_conformance':'INCOMPLETE','new_formal_binding_gap':False,'normalizer':'NotImplemented','full_scope':'Pending G02','positive_root_export':'NotAuthorized'}
if __name__=='__main__':print(json.dumps(run(),indent=2,ensure_ascii=True))
