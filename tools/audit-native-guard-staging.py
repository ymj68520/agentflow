"""Read-only G2 registration/frozen-byte/coverage audit; NOT capture oracle."""
import hashlib,json,re,subprocess,sys
from pathlib import Path
from native_guard_registration_data import BASELINE,MODEL,HISTORICAL,SOURCES,RULES,NATIVE_NAMES
ROOT=Path(__file__).resolve().parent.parent
def read(p):return json.loads((ROOT/p).read_bytes())
def sha(b):return hashlib.sha256(b).hexdigest()
def gitraw(p):return subprocess.check_output(['git','show',BASELINE+':'+p],cwd=ROOT)
def model(p):return json.loads(re.search(r'```json\s*(.*?)\s*```',(ROOT/p).read_text(encoding='utf-8'),re.S)[1])
def refs(x):
 if isinstance(x,dict):
  if 'ref' in x:yield x['ref']
  for v in x.values():yield from refs(v)
 elif isinstance(x,list):
  for v in x:yield from refs(v)
def main():
 oldspec=json.loads(gitraw('spec/registry/specifications.json'))['specifications'];oldrules=json.loads(gitraw('spec/registry/rules.json'))['rules']
 specs=read('spec/registry/specifications.json')['specifications'];rules=read('spec/registry/rules.json')['rules']
 assert len(oldspec)==50 and len(oldrules)==452
 for key,old,new in [('specification_id',oldspec,specs),('rule_id',oldrules,rules)]:
  assert len({x[key] for x in new})==len(new)
  assert all(next((y for y in new if y[key]==x[key]),None)==x for x in old)
 for row in specs:
  p=row['normative_source']['logical_path'];assert sha((ROOT/p).read_bytes())==row['normative_source']['content_digest']['hex'],p
 for row in oldspec:
  p=row['normative_source']['logical_path'];assert (ROOT/p).read_bytes()==gitraw(p),p
 m=model(SOURCES['RM'][2]);old=model('spec/tables/proof-report-model-v2.md');assert m['inherited_registered_model']==old
 assert old['inherited_registered_model']==model('spec/tables/proof-report-model-v1.md')
 types=m['types'];rs=list(refs(types));assert all(n in types for n in rs)
 # The ONLY changes to existing logical types are approved append-only
 # parent extensions. Embedded old editions remain complete and identical.
 parents={'ScopeSubject','Source','Target','Coordinate','EvidenceRef','TypedContradiction','PrerequisitePredicate','RequirementClause','PredicateFamily','Kind','ObservationOwner','ClosedPredicateCase','Fragment','Fact'}
 for n,d in old['types'].items():
  if n not in parents:assert types[n]==d,n
  elif n=='Fragment':
   x=json.loads(json.dumps(types[n]));x['fields'][1][1]['variants'].pop();assert x==d,n
  else:
   field='fields' if n=='Fact' else 'variants';assert types[n][field][:len(d[field])]==d[field],n
 assert len(m['native_case_descriptors'])==7
 selected=m['historical_selected_imports'];assert len(selected)==97 and sum(x['root'] for x in selected)==5
 hist=old['inherited_registered_model']
 for row in selected:
  assert row['owner']==HISTORICAL and row['successor_key']==HISTORICAL+'/'+row['name']
  assert types[row['successor_key']]==row['successor_descriptor']
  if row['historical_declaration'] is not None:assert row['historical_declaration']==hist['semantic_model_declarations'][row['name']]
  assert all(HISTORICAL+'/'+dep in types for dep in row['recursive_dependencies'])
 for p,v in m['historical_import_sources'].items():
  b=(ROOT/p).read_bytes();assert b.hex()==v['content_hex'] and sha(b)==v['sha256']
 projection=read('docs/implementation/phase-04c1b4c4r3g1-projection-contract.json')
 native14=types[HISTORICAL+'/af-contract::TypedGuardCountermodel']['fields']
 pfields=[x['field'] if 'field' in x else x['field_name'] for x in projection['fields']]
 assert [x[0] for x in native14]==pfields and len(set(pfields))==14
 for rule in RULES:
  p=SOURCES[rule['source']][2];text=(ROOT/p).read_text(encoding='utf-8')
  assert text.count('id="'+rule['anchor']+'"')==1
  assert 'MUST: '+rule['statement'] in text
  assert all(any(r['rule_id']==dep for r in rules) for dep in rule['dependencies'])
 corpus=read('spec/conformance/g01-native-guard-staging/manifest.json');ids=corpus['case_ids'];assert len(ids)==len(set(ids))==corpus['case_count']
 assert corpus['capture_case_ids']==0
 inv=read('spec/conformance/g01-native-guard-staging/shape-cases.json')['coverage']
 coverage={(x['type'],x.get('variant'),x.get('field')) for x in inv}
 for n,d in types.items():
  if n.startswith(('Native','CanonicalNative','Imported',HISTORICAL+'/')):
   if d.get('kind')=='record':assert all((n,None,k) in coverage for k,_ in d['fields']),n
   if d.get('kind')=='enum':
    assert all((n,k,None) in coverage for k,_ in d['variants']),n
    assert all((n,k,f) in coverage for k,v in d['variants'] if v.get('kind')=='record' for f,_ in v['fields']),n
 assert sum(1 for _ in (ROOT/'spec/idl').rglob('*.afidl'))==17
 for p in ['spec/registry/abi-numbers.json','spec/tables/hash-domains.json','Cargo.lock']:
  assert (ROOT/p).read_bytes()==gitraw(p),p
 known_changes={'crates/af-computation/Cargo.toml','crates/af-computation/src/registered_model.rs','crates/af-computation/src/root_export.rs','crates/af-portable/Cargo.toml','crates/af-portable/src/support.rs','spec/registry/rules.json','spec/registry/specifications.json','tools/gen-registry-instances.py','tools/gen-substitution-successor-staging.py'}
 changed=subprocess.check_output(['git','diff',BASELINE,'--name-only'],cwd=ROOT,text=True).splitlines()
 # Newly committed paths show in this diff too; non-baseline files are allowed.
 original=set(subprocess.check_output(['git','ls-tree','-r','--name-only',BASELINE],cwd=ROOT,text=True).splitlines())
 assert set(changed)&original<=known_changes,(set(changed)&original)-known_changes
 print(json.dumps({'kind':'G2 read-only Registration Conformance audit;NOT native semantic Oracle','baseline':BASELINE,'specifications':len(specs),'rules':len(rules),'old_rows_exact':[len(oldspec),len(oldrules)],'old_sources_byte_equal':50,'model':MODEL,'types':len(types),'resolved_refs':len(rs),'selected_imports':len(selected),'five_guard_roots':NATIVE_NAMES,'countermodel_fields':pfields,'registration_case_ids':len(ids),'native_field_variant_inventory':len(inv),'capture_case_ids':0,'positive_capability':'NotAuthorized','frozen_assets':'ABI/AFIDL/HashV1/old models/old Expected preserved'},indent=2))
if __name__=='__main__':main()
