"""Independent normative-source census/Expected for successor registration ONLY.

No Rust Actual or authoring model constructor feeds the reference model.
Declaration readback coverage is NOT executed value-branch or production credit.
"""
import argparse,copy,functools,hashlib,json,re,sys
from pathlib import Path
R=Path(__file__).resolve().parents[2];sys.path.insert(0,str(R/'tools'))
from oracle.structural_binding_registration import catalogue as old_catalogue,nominal,descendants,children,closure
MODEL='agentflow.computation.proof-report.model@8';OWNER='agentflow.root-export-checker@9'
EDITION=dict(analysis=OWNER,model=MODEL,report='agentflow.g01.report-shape@8',obligation='agentflow.g01.obligation@8')
D=R/'spec/conformance/g01-native-source-staging';P='phase-04c1b4c4r3c2r5g2-'
SLOTS=['effect_resource','recovery_usage','flow_execution']

def read(p):return json.loads(p.read_bytes())
def js(p,v):
 p.parent.mkdir(parents=True,exist_ok=True);data=(json.dumps(v,ensure_ascii=False,sort_keys=True,indent=2)+'\n').encode()
 if not p.exists() or p.read_bytes()!=data:p.write_bytes(data)
@functools.lru_cache(None)
def catalogue():
 raw=(R/'spec/tables/proof-report-model-v8.md').read_text(encoding='utf8')
 delta=json.loads(re.search(r'```json\s*(.*?)\s*```',raw,re.S)[1]);old=old_catalogue()
 assert delta['base_source']==dict(specification_id=old['model_id'],logical_path='spec/tables/proof-report-model-v7.md',plain_sha256=hashlib.sha256((R/'spec/tables/proof-report-model-v7.md').read_bytes()).hexdigest())
 assert delta['edition']==EDITION
 changed={'ContractProofContextContents','StructuralValue0030','StructuralValue0031','StructuralValue0032','StructuralValue0033'}
 assert set(delta['type_replacements'])==changed
 m=copy.deepcopy(old);m['types'].update(delta['type_replacements']);m.update(model_id=MODEL,analysis_owner=OWNER,model_edition=EDITION['report'],obligation_edition=EDITION['obligation'],structural_binding_registration=delta['registration'],canonical_constraints=delta['canonical_constraints'],inherited_registered_model=old)
 # Independently derive the five permitted edits from actual old declarations.
 for name,d in m['types'].items():
  wanted=copy.deepcopy(old['types'][name])
  if name in changed:
   def rewrite(x):
    if isinstance(x,dict):
     if x=={'ref':'RelationSnapshotContent'}:return {'ref':'NativeRelationSnapshotSource'}
     return {k:rewrite(v) for k,v in x.items()}
    if isinstance(x,list):return [rewrite(v) for v in x]
    return x
   wanted=rewrite(wanted)
  assert d==wanted,('unexpected declaration change',name)
 assert m['type_ranks']==old['type_ranks'] and len(m['types'])==1131
 assert m['types']['NativeRelationSnapshotSource']==old['types']['NativeRelationSnapshotSource']
 assert m['types']['RelationSnapshotContent']==old['types']['RelationSnapshotContent']
 assert len(m['historical_selected_imports'])==97
 assert all(d['ref'] in m['types'] for d in descendants(m['types']) if 'ref' in d)
 r=m['structural_binding_registration'];assert len(r['sites'])==317 and len(r['reference_policies'])==66
 for policy in r['reference_policies']:
  if policy['key']==dict(parent=dict(owner=MODEL,name='CurrentContextRelationSchemaValidation'),path=[{'field':'source'}]):
   assert policy['environment'] in ['output_record','w_value'] and policy['domain']=='g_external' and policy['target']==dict(owner=MODEL,name='NativeRelationSnapshotSource')
 # Alias site descriptors must reflect every exact declaring syntax path.
 for site in r['sites']:
  node=m['types'][site['source_catalogue_key']]
  for kind,value in site['source_path']:
   node=dict(node['fields'])[value] if kind=='field' else dict(node['variants'])[value] if kind=='case' else node['key'] if kind=='key' else node['value'] if kind=='value' else node['item']
  assert site['descriptor']==node==m['types'][site['alias']]
 return m

def reply(value):return dict(registered=value,assurance='registration_metadata_only',production_dagranks_generated=0,complete_view_issued=False,obligation_identity_issued=False,positive_capability='NotAuthorized')
def roster():
 m=catalogue();r=m['structural_binding_registration'];rules=read(R/'spec/registry/rules.json')['rules'];rows=[]
 def add(cid,op,value,**args):rows.append(dict(case_id='g01-native-source-staging/'+cid,input=dict(edition=EDITION,op=op,**args),expected=dict(ok=True,value=reply(value))))
 for n,d in m['types'].items():add('descriptor/'+n,'descriptor',d,name=n)
 for s in r['sites']:add('site/'+s['alias'],'site',s,name=s['alias'])
 for i,p in enumerate(r['reference_policies']):add('reference/'+str(i),'reference',p,index=i)
 for i,p in enumerate(r['reference_policies']):
  if p['key']==dict(parent=dict(owner=MODEL,name='CurrentContextRelationSchemaValidation'),path=[{'field':'source'}]):
   for label in ['w-local','g-local','alias-target','cross-edition']:
    bad=copy.deepcopy(p)
    if label in ['w-local','g-local']:bad['domain']=label.replace('-','_')
    elif label=='alias-target':bad['target']['name']='RelationSnapshotContent'
    else:bad['target']['owner']='agentflow.computation.proof-report.model@7'
    rows.append(dict(case_id=f'g01-native-source-staging/reference-deny/{i}/{label}',input=dict(edition=EDITION,op='reference',index=i,proposal=bad),expected=dict(ok=False,failure='InvalidFacts')))
 for i in range(315,324):
  key=f'AF-SEMANTIC-{i:04}';add(f'rule-{i:04}','rule',next(x for x in rules if x['rule_id']==key),name=key)
 for slot in SLOTS:
  add(slot+'/some','source_edge',dict(slot=slot,target=dict(owner=MODEL,name='NativeRelationSnapshotSource'),source_stage='before_G',c_source_domain='g_external'),slot=slot,presence='some')
  add(slot+'/none','source_edge',dict(slot=slot,source_nodes=0,c_validation='NotApplicable'),slot=slot,presence='none')
 for op in ['production','raw','complete_view','obligation_identity','portable_positive','root_export']:
  rows.append(dict(case_id='g01-native-source-staging/deny/'+op,input=dict(edition=EDITION,op=op),expected=dict(ok=False,failure='NotImplemented')))
 for tuple_name in ['analysis','model','report','obligation']:
  e=copy.deepcopy(EDITION);e[tuple_name]+='-mixed'
  rows.append(dict(case_id='g01-native-source-staging/mixed/'+tuple_name,input=dict(edition=e,op='descriptor',name='NativeRelationSnapshotSource'),expected=dict(ok=False,failure='UnsupportedModel')))
 from native_source_metadata_cases import field_cases,dag_cases
 values=field_cases();_,graphs=dag_cases()
 rows.extend(r for r in values if 'case_id' in r);rows.extend(graphs)
 from oracle.native_relation_source_design import native_source
 for slot in SLOTS:
  value=native_source(slot)
  rows.append(dict(case_id='g01-native-source-staging/source/'+slot,input=dict(edition=EDITION,op='source',slot=slot,source_stage='before_G',value=value),expected=dict(ok=True,value=reply(value))))
  for kind in ['family','slot','checksum','revision','after-C']:
   bad=copy.deepcopy(value);stage='before_G'
   if kind=='family':bad['schema_family']='wrong.family@1'
   elif kind=='slot':bad['kind']=next(s for s in SLOTS if s!=slot)
   elif kind=='checksum':bad['pin']['canonical_content_checksum']['hex']='0'*64
   elif kind=='revision':bad['revision']='not-original'
   else:stage='after_C'
   rows.append(dict(case_id='g01-native-source-staging/source/'+slot+'/'+kind,input=dict(edition=EDITION,op='source',slot=slot,source_stage=stage,value=bad),expected=dict(ok=False,failure='InvalidFacts')))
 return rows

def exact(a,b):
 assert type(a) is type(b)
 if isinstance(a,dict):
  assert set(a)==set(b)
  for k in a:exact(a[k],b[k])
 elif isinstance(a,list):
  assert len(a)==len(b)
  for x,y in zip(a,b):exact(x,y)
 else:assert a==b

def comparator_mutations():
 rows=roster();target=next(r for r in rows if r['input'].get('name')=='NativeRelationSnapshotSource' and r['input']['op']=='descriptor')
 required=target['expected'];out=[]
 for field in [n for n,_ in catalogue()['types']['NativeRelationSnapshotSource']['fields']]:
  for both in [False,True]:
   want=copy.deepcopy(required);got=copy.deepcopy(required)
   for view in ([want,got] if both else [got]):
    view['value']['registered']['fields']=[r for r in view['value']['registered']['fields'] if r[0]!=field]
   try:exact(required,want);exact(required,got)
   except AssertionError:out.append(dict(field=field,both_sides=both,rejected=True,credit='registration comparator only'))
   else:raise AssertionError('accepted required field omission')
 for key,value in [('positive_capability','Authorized'),('production_dagranks_generated',1),('complete_view_issued',True),('obligation_identity_issued',True)]:
  got=copy.deepcopy(required);got['value'][key]=value
  try:exact(required,got)
  except AssertionError:out.append(dict(forged=key,rejected=True,credit='registration comparator only'))
  else:raise AssertionError(key)
 return out

def report(actual):
 want=roster();exact(want,read(D/'cases.json')['cases']);assert len(actual)==len(want)
 for w,a in zip(want,actual):assert w['case_id']==a['case_id'];exact(w['expected'],a['result'])
 return dict(registration_cases=len(want),declaration_descriptors=1131,sites=317,policies=66,shape_field_variants=1297,manual_source_subtrees=3,comparator_mutations=comparator_mutations(),production_capture_credit=0,production_dagranks=0,root_export='NotAuthorized',scope='Registration/Metadata Only; not native source admission or complete producer')

if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--materialize',action='store_true');p.add_argument('--census',action='store_true');a=p.parse_args()
 if a.materialize:
  rows=roster();js(D/'cases.json',dict(cases=rows,scope='declaration registration and metadata ONLY'))
  from native_source_metadata_cases import dag_cases,field_cases
  sources,_=dag_cases();js(D/'manual-source-fixtures.json',sources);js(D/'value-coverage.json',field_cases())
  js(D/'manifest.json',dict(case_ids=[r['case_id'] for r in rows],production_case_ids=0,production_binder='NotImplemented'))
  print(len(rows))
 elif a.census:
  m=catalogue();print(json.dumps(dict(descriptors=len(m['types']),sites=len(m['structural_binding_registration']['sites']),policy_count=len(m['structural_binding_registration']['reference_policies']),unresolved=0)))
 else:print(json.dumps(report(json.load(sys.stdin))))
