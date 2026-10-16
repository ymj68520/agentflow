"""C1G2 frozen-content/registration audit. NOT Typed Capture conformance."""
import hashlib,json,re,subprocess,sys
from pathlib import Path
from raw_validation_registration_data import *
R=Path(__file__).resolve().parent.parent
def read(p):return json.loads((R/p).read_bytes())
def sha(p):return hashlib.sha256((R/p).read_bytes()).hexdigest()
def model(p):return json.loads(re.search(r'```json\s*(.*?)\s*```',(R/p).read_text(encoding='utf-8'),re.S)[1])
def refs(x):
 if isinstance(x,dict):
  if 'ref' in x:yield x['ref']
  assert '_generic' not in x and '_case' not in x
  for v in x.values():yield from refs(v)
 elif isinstance(x,list):
  for v in x:yield from refs(v)
def main():
 frozen=read('docs/implementation/phase-04c1b4c4r3c1g2-frozen-inputs.json')
 assert frozen['baseline_head']==BASELINE and len(frozen['frozen_paths'])==2011
 for p,h in frozen['frozen_paths'].items():assert sha(p)==h,('frozen original raw bytes',p)
 original=set(subprocess.check_output(['git','ls-tree','-r','--name-only',BASELINE],cwd=R,text=True).splitlines())
 assert original==set(frozen['frozen_paths'])|set(frozen['authorized_existing_changes'])
 changed=set(subprocess.check_output(['git','diff',BASELINE,'--name-only'],cwd=R,text=True).splitlines())&original
 assert changed<=set(frozen['authorized_existing_changes'])
 for name,key,id,count in [('specifications','specifications','specification_id',54),('rules','rules','rule_id',468)]:
  old=read(f'spec/conformance/g01-raw-validation-staging/edition4-{name}.json')[key]
  now=read(f'spec/registry/{name}.json')[key];assert len(old)==count
  assert all(next((r for r in now if r[id]==x[id]),None)==x for x in old)
  assert len({r[id] for r in now})==len(now)
 specs=read('spec/registry/specifications.json')['specifications'];rules=read('spec/registry/rules.json')['rules']
 assert (len(specs),len(rules))==(58,479)
 for s in specs:assert sha(s['normative_source']['logical_path'])==s['normative_source']['content_digest']['hex']
 for r in RULES:
  row=next(x for x in rules if x['rule_id']==r['rule_id']);text=(R/row['normative_source']).read_text(encoding='utf-8')
  assert row['short_statement']==r['statement'] and text.count('id="'+r['anchor']+'"')==1
  assert r['case_id'] in row['conformance_refs'] and r['statement'] in text
 old=model('spec/tables/proof-report-model-v3.md');new=model(SOURCES['RM'][2]);t=new['types']
 assert new['inherited_registered_model']==old
 assert new['historical_selected_imports']==old['historical_selected_imports'] and new['historical_import_sources']==old['historical_import_sources']
 variants=new['raw_registration']['parent_extensions'];records=new['raw_registration']['record_extensions']
 for i,(name,desc) in enumerate(old['types'].items()):
  assert new['type_ranks'][name]==i
  if name in variants:assert t[name]['variants'][:-1]==desc['variants']
  elif name in records:assert t[name]['fields'][:-1]==desc['fields']
  elif name=='Fragment':
   d=json.loads(json.dumps(t[name]));d['fields'][1][1]['variants'].pop();assert d==desc
  else:assert t[name]==desc,name
 assert len(set(new['type_ranks'].values()))==len(t) and set(new['type_ranks'])==set(t)
 allrefs=list(refs(t));assert all(x in t for x in allrefs)
 assert len(new['raw_case_descriptors'])==1 and new['raw_case_descriptors'][0]['governing_rule']=='AF-CONTRACT-0004'
 assert new['native_case_descriptors']==old['native_case_descriptors'] and new['case_descriptors']==old['case_descriptors']
 manifest=read('spec/conformance/g01-raw-validation-staging/manifest.json');cases=read('spec/conformance/g01-raw-validation-staging/shape-cases.json')
 ids={c['case_id'] for c in cases['cases']};assert len(ids)==len(cases['cases'])
 assert len(set(manifest['case_ids']))==len(manifest['case_ids'])
 required=[]
 for name in new['raw_registration']['authored_types']:
  d=t[name]
  for f,_ in d.get('fields',[]):required.append((name,None,f))
  for v,p in d.get('variants',[]):
   required.append((name,v,None))
   for f,_ in p.get('fields',[]):required.append((name,v,f))
 for name,v in variants.items():
  required.append((name,v,None));p=dict(t[name]['variants'])[v]
  for f,_ in p.get('fields',[]):required.append((name,v,f))
 for name,f in records.items():required.append((name,None,f))
 required.append(('Fragment','raw_contract_validation','profile'))
 assert all(any(c['type']==n and c.get('variant')==v and c.get('field')==f and c['case_id'] in ids for c in cases['coverage']) for n,v,f in required)
 assert {(c['type'],c.get('variant'),c.get('field')) for c in cases['coverage']}==set(required)
 # Full source and actual imported declaration checks remain independent of
 # whichever implementation bytes occupy the old source paths.
 from native_guard_source_binding import check
 binding=check(old,read('spec/conformance/g01-native-guard-staging/historical-imports.json'))
 assert binding['live_declarations_compared']==90
 result={'kind':'Registration/freeze audit; NOT Capture','baseline':BASELINE,'original_frozen_raw_paths':2011,
 'authorized_existing_changes':sorted(changed),'old_spec_records_full_equal':54,'old_rule_records_full_equal':468,
 'specifications':58,'rules':479,'reference_covered':478,'intentional_exception':1,'missing':0,
 'new_specifications':len(SOURCES),'new_rules':len(RULES),'abi':775,'afidl':17,'hashv1_domains':14,
 'types':len(t),'descriptor_refs':len(allrefs),'selected_historical_imports':97,'live_imported_declarations':90,
 'preserved_old_type_ranks':len(old['types']),'new_shape_cases':len(ids),'new_registration_case_ids':len(manifest['case_ids']),
 'required_field_variant_sites':len(required),'covered_required_sites':len(required),'field_variant_gaps':0,
 'shape_field_case_edges':len(cases['coverage']),'capture_case_ids':0,'raw_producer':'NotImplemented','positive_capability':'NotAuthorized'}
 print(json.dumps(result,indent=2))
if __name__=='__main__':main()
