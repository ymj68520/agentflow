"""Independent relation Registration/Metadata Admission Oracle ONLY.

Expected derives from frozen AFIDL/native catalog inputs, never Rust Actual.
No complete Raw Fact, Scope identity or positive capability is constructed.
"""
import copy,functools,hashlib,importlib.util,json,re,sys,unicodedata
from pathlib import Path
R=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(R/'tools'))
from relation_schema_registration_data import SLOTS,PREFIX,MODEL,OWNER
def module(name,path):
 s=importlib.util.spec_from_file_location(name,R/path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
AF=module('relation_python_afidl','tools/gen-afidl-corpus.py')
CAT=module('relation_python_catalog','tools/gen-component-contract-corpus.py')
KIND={'effect_resource':'effect-resource','recovery_usage':'recovery-usage','flow_execution':'flow-execution'}
def info(slot):return next((root,family) for s,root,family in SLOTS if s==slot)
def sources(slot,mode):
 result=[(p.relative_to(R).as_posix(),p.read_text(encoding='utf8')) for p in sorted((R/'spec/idl').rglob('*.afidl'))]
 root,family=info(slot);filename={'effect_resource':'effect-resource-compatibility','recovery_usage':'recovery-usage-coverage','flow_execution':'flow-execution-relation'}[slot]
 path='spec/idl/'+filename+'.afidl'
 if mode=='missing_module':return [(p,t) for p,t in result if p!=path]
 for i,(p,t) in enumerate(result):
  if p!=path:continue
  if mode=='wrong_family':
   header='schema '+family.replace('@',' @')+';';assert header in t;t=t.replace(header,'schema wrong.family @1;')
  elif mode=='B':
   old={'effect_resource':'2 supported_pairs: list<SupportedEffectResourcePair>;','recovery_usage':'1 rules: list<RecoveryUsageCoverageRule>;','flow_execution':'1 closed_regions: list<execution.RegionRef>;'}[slot]
   new=re.sub(r'list<[^>]+>','list<string>',old);assert t.count(old)==1;t=t.replace(old,new)
  elif mode=='invalid':
   field={'effect_resource':'complete_operations','recovery_usage':'rules','flow_execution':'closed_regions'}[slot]
   at=t.index('record '+root+' {');head=t[:at];tail=t[at:];tail=tail.replace(' '+field+':',' renamed_'+field+':',1);t=head+tail
  elif mode=='missing_root':
   t=t.replace('record '+root+' {','record Renamed'+root+' {')
  result[i]=(p,t)
 return result
@functools.lru_cache(None)
def compiled(slot,mode):
 _,modules=AF.compile_module_set(sources(slot,mode))
 return modules,{t['schema']+'/'+t['name']:t for rows in modules.values() for t in rows}
PRIMITIVES=dict(zip(['unit','null','bool','int32','uint32','int64','uint64','decimal','float64','string','bytes','timestamp','duration'],['Unit','Null','Bool','Int32','UInt32','Int64','UInt64','Decimal','Float64','String','Bytes','Timestamp','Duration']))
def expr(e):
 k,v=e
 if k=='primitive':return {'Primitive':{'0':PRIMITIVES[v]}}
 if k=='ref':return {'NamedRef':{'0':v}}
 return {{'list':'List','map_string':'MapString','option':'Option'}[k]:{'0':expr(v)}}
def definition(t):
 k,v=t['body']
 if k=='alias':b={'Alias':{'0':expr(v)}}
 elif k=='enum':b={'Enum':{'0':[dict(no=r['no'],name=r['name']) for r in v['values']]}}
 else:
  key='fields' if k=='record' else 'cases'
  b={k.title():{'0':[dict(no=r['no'],name=r['name'],type_expr=expr(r['type'])) for r in v[key]]}}
 return dict(schema=t['schema'],name=t['name'],body=b)
def refs(e):
 k,v=e
 if k=='ref':yield v
 elif k in ('list','option','map_string'):yield from refs(v)
def children(t):
 k,v=t['body']
 if k=='alias':yield from refs(v)
 elif k in ('record','variant'):
  for r in v['fields' if k=='record' else 'cases']:yield from refs(r['type'])
def closure(t,types):
 found={};pending=[t['schema']+'/'+t['name']]
 while pending:
  n=pending.pop()
  if n in found:continue
  if n not in types:raise KeyError(n)
  found[n]=types[n];pending.extend(children(types[n]))
 return [definition(found[n]) for n in sorted(found)]
def accepts(e,value,types):
 k,v=e
 if k=='ref':return accepts(types[v]['body'],value,types)
 if k=='alias':return accepts(v,value,types)
 if k=='list':return type(value) is list and all(accepts(v,x,types) for x in value)
 if k=='option':return value is None or accepts(v,value,types)
 if k=='primitive':
  if v in ('unit','null'):return value is None
  if v=='bool':return type(value) is bool
  if v=='string':return type(value) is str and unicodedata.normalize('NFC',value)==value
  if v in ('uint32','uint64','int32','int64','timestamp','duration'):
   bits=32 if v.endswith('32') else 64;signed=not v.startswith('uint');return type(value) is int and (-(2**(bits-1)) if signed else 0)<=value<2**(bits-1 if signed else bits)
  if v=='bytes':return type(value) is list and all(type(x) is int and 0<=x<=255 for x in value)
  if v=='decimal':return type(value) is str
  if v=='float64':return type(value) in (int,float)
  raise ValueError(v)
 if k=='enum':return type(value) is str and value in [x['name'] for x in v['values']]
 if k=='record':return type(value) is dict and set(value)=={f['name'] for f in v['fields']} and all(accepts(f['type'],value[f['name']],types) for f in v['fields'])
 if k=='variant':
  if type(value) is not dict or len(value)!=1:return False
  tag,payload=next(iter(value.items()));case=next((c for c in v['cases'] if c['name']==tag),None)
  return case is not None and accepts(case['type'],payload,types)
 raise ValueError(k)
def catalog(c):
 slot=c['slot']
 assert c.get('catalog') in ('empty','nonempty'),'unsupported native catalog fixture input'
 if c.get('catalog')=='nonempty':
  p=R/f'spec/conformance/component-contract/relation-catalogs/{KIND[slot]}/canonical/cases.json'
  return CAT.canonical_relation(KIND[slot],json.loads(p.read_bytes())['cases'][0]['input_value'])
 return {'effect_resource':dict(complete_operations=[],supported_pairs=[]),'recovery_usage':dict(rules=[]),'flow_execution':dict(closed_regions=[],closed_trust_domains=[],confidentiality_rules=[],taint_rules=[],authority_rules=[],sink_rules=[])}[slot]
def decode_typed(e,value,types):
 k,v=e
 if k=='ref':return decode_typed(types[v]['body'],value,types)
 if k=='alias':return decode_typed(v,value,types)
 if k=='primitive':
  if v in ('uint32','uint64','int32','int64','timestamp','duration'):
   assert type(value) is str and re.fullmatch(r'-?(0|[1-9][0-9]*)',value)
   return int(value)
  if v=='float64':return float(value)
  return value
 if k=='list':return [decode_typed(v,x,types) for x in value]
 if k=='option':return None if value is None else decode_typed(v,value,types)
 if k=='record':return {f['name']:decode_typed(f['type'],value[f['name']],types) for f in v['fields']}
 if k=='enum':return value
 if k=='variant':
  tag,payload=next(iter(value.items()));case=next(c for c in v['cases'] if c['name']==tag)
  return {tag:decode_typed(case['type'],payload,types)}
 raise ValueError(k)
def validate_native_catalog(slot,value):
 # Independent full fixed native wire shape plus exact canonicalization. The
 # valid constructor fixtures also require the native closed key constraints.
 root,family=info(slot);_,types=compiled(slot,'A')
 assert accepts(types[family+'/'+root]['body'],value,types)
 assert CAT.canonical_relation(KIND[slot],value)==value
 def unique(rows,key):assert len(rows)==len({json.dumps(key(x),sort_keys=True) for x in rows})
 def named(r):assert re.fullmatch(r'[a-z][a-z0-9-]*(\.[a-z][a-z0-9-]*)*@[1-9][0-9]*',r['catalog']) and r['name'] and unicodedata.normalize('NFC',r['name'])==r['name']
 if slot=='effect_resource':
  unique(value['complete_operations'],lambda x:(x['catalog'],x['name']))
  for op in value['complete_operations']:named(op)
  unique(value['supported_pairs'],lambda x:(x['operation'],x['resource_type']))
  for pair in value['supported_pairs']:
   assert pair['operation'] in value['complete_operations'];unique(pair['selector_keys'],lambda x:x['selector_key'])
   unique(pair['allowed_qualifier_profiles'],lambda x:x['qualifiers'])
   for p in pair['allowed_qualifier_profiles']:unique(p['qualifiers'],lambda x:next(iter(x)))
   for r in pair['selector_keys']:
    assert r['selector_key'] and r['resource_attribute_key']
    if 'finite_exact_values' in r['values']:unique(r['values']['finite_exact_values'],lambda x:x);assert all(r['values']['finite_exact_values'])
 elif slot=='recovery_usage':
  unique(value['rules'],lambda x:CAT.action_key(x['action']))
  for r in value['rules']:
   unique(r['complete_meters'],lambda x:(x['catalog'],x['name']))
   for meter in r['complete_meters']:named(meter)
 else:
  unique(value['closed_regions'],lambda x:x);unique(value['closed_trust_domains'],lambda x:x)
  for x in value['closed_regions']+value['closed_trust_domains']:named(x)
  for key,identity in [('confidentiality_rules','label'),('taint_rules','taint'),('authority_rules','authority')]:unique(value[key],lambda x:x[identity])
  unique(value['sink_rules'],lambda x:(x['kind'],x['sink']))
  for r in value['confidentiality_rules']:
   named(r['label'])
   if 'finite_exact_pairs' in r['region_pairs']:
    pairs=r['region_pairs']['finite_exact_pairs'];unique(pairs,lambda x:x)
    assert all(p['execution_region'] in value['closed_regions'] and p['residency_region'] in value['closed_regions'] for p in pairs)
  for r in value['taint_rules']+value['authority_rules']+value['sink_rules']:
   domains=r['trust_domains'].get('exact_allowlist')
   if domains is not None:unique(domains,lambda x:x);assert all(d in value['closed_trust_domains'] for d in domains)
  for r in value['sink_rules']:
   if 'workflow_output' in r['sink']:assert r['sink']['workflow_output']>0
def target(slot):
 root,family=info(slot);return dict(slot=slot,family=family,root_type=root)
def case_base(slot):return dict(slot=slot,current='A',present=True,catalog='empty',association='same',compiler='current',mode='native',expected_pin=None)
def cases():
 rows=[]
 def add(slot,label,**kwargs):
  c=case_base(slot);c.update(kwargs);c['case_id']=PREFIX+'admission-'+slot+'-'+label;rows.append(c)
 for slot,_,_ in SLOTS:
  for label,kw in [('valid',{}),('none',dict(present=False)),('current-B',dict(current='B')),('missing-module',dict(current='missing_module')),('missing-root',dict(current='missing_root')),('invalid-current',dict(current='invalid')),('wrong-family-context',dict(current='wrong_family')),('nonempty',dict(catalog='nonempty')),('nonempty-under-B',dict(catalog='nonempty',current='B')),('wrong-expected-family',dict(expected_pin='family')),('revision-mismatch',dict(expected_pin='revision')),('wrong-compiler',dict(compiler='wrong')),('split-context',dict(association='split')),('ambiguous-boundary',dict(mode='ambiguous'))]:add(slot,label,**kw)
  # Each proposal mutation is a separate metadata admission negative, not an
  # alleged native branch or a modified expected production verdict.
  mutations=[('schema-owner',['algorithm','model'],'set','wrong.model@1'),('schema-name',['validation','valid','found','schema','type_name'],'set','WrongRoot'),('schema-digest',['validation','valid','found','schema','schema_digest','hex'],'set','0'*64),('checksum-as-schema',['validation','valid','found','schema','schema_digest','domain'],'set','catalog-content'),('missing-source',['source'],'set',{'none':None}),('malformed-family',['source','some','schema_family'],'set','INVALID@01'),('slot-swap',['source','some','kind'],'set','recovery_usage' if slot!='recovery_usage' else 'effect_resource'),('missing-result',['validation','valid','found'],'remove',None),('unknown-validation',['validation'],'set',{'future':None}),('fabricated-history',['source','some','construction_schema'],'set',{}),('wrong-stage',['lookup'],'set',{'not_reached':{'cause':'missing_source'}}),('old-model',['algorithm','model'],'set','agentflow.computation.proof-report.model@5'),('false-authority',['positive_root_export'],'set',True)]
  for label,path,op,value in mutations:add(slot,label,mode='proposal',mutation=dict(path=path,op=op,value=value))
  checksum=hashlib.sha256(CAT.ace1(catalog(case_base(slot))).encode()).hexdigest()
  for label,path,value in [
   ('schema-nominal-owner',['validation','valid','found','schema','owner'],'unregistered.schema.owner@1'),
   ('schema-family',['validation','valid','found','schema','specification_id'],'wrong.family@1'),
   ('catalog-checksum-as-schema',['validation','valid','found','schema','schema_digest','hex'],checksum),
   ('source-checksum',['source','some','pin','canonical_content_checksum','hex'],'0'*64),
  ]:add(slot,label,mode='proposal',mutation=dict(path=path,op='set',value=value))
  add(slot,'false-alias-claim',mode='proposal',association='split',mutation=dict(path=['context_roles'],op='set',value=[['analysis','canonical_validation']]))
 return rows

def expected(c,module_selection=None):
 slot=c['slot'];root,family=info(slot);profile_slot,profile_mode=module_selection or (slot,c['current']);mods,types=compiled(profile_slot,profile_mode);v=catalog(c);validate_native_catalog(slot,v)
 data=CAT.ace1(v).encode();p=dict(family=family,revision='g2-fixture',canonical_content_checksum=dict(algorithm='sha256',hex=hashlib.sha256(data).hexdigest()))
 src=dict(kind=slot,schema_family=family,revision='g2-fixture',pin=p,canonical_content=list(data),catalog={slot:v})
 result=dict(assurance='relation_slot_only',algorithm=dict(analysis=OWNER,model=MODEL,lookup_rule='AF-SEMANTIC-0288',validation_rule='AF-SEMANTIC-0292'),target=target(slot),source={'some':src} if c['present'] else {'none':None},context_roles=[['analysis','canonical_validation']] if c['association']=='same' else [['analysis'],['canonical_validation']],current_modules=[dict(specification_id=k,types=[definition(t) for t in mods[k]]) for k in sorted(mods)],lookup={'not_reached':{'cause':'missing_source'}},validation={'not_reached':None},admission='incomplete')
 if c['mode']=='ambiguous':
  _,a=compiled(slot,'A');_,b=compiled(slot,'B');key=family+'/'+root
  return dict(boundary_lookup={'ambiguous':dict(target=target(slot),definitions=[definition(a[key]),definition(b[key])])},raw_capture_complete=False,positive_root_export=False)
 if c['compiler']!='current':result['lookup']={'not_reached':{'cause':'invalid_identity'}}
 elif c['association']!='same':result['lookup']={'not_reached':{'cause':'invalid_context'}}
 elif not c['present']:result['admission']='absent'
 elif c['expected_pin']=='family':result['lookup']={'family_mismatch':dict(target=target(slot),source_family='wrong.family@1')}
 elif c['expected_pin']=='revision':
  other=copy.deepcopy(p);other['revision']='different';result['lookup']={'revision_mismatch':dict(actual=p,expected=other)}
 else:
  key=family+'/'+root
  if key not in types:result['lookup']={'missing':dict(target=target(slot),part='root' if family in mods else 'module',missing_identity=key)}
  else:
   t=types[key];_,digest=AF.type_schema_digest(t)
   found=dict(target=target(slot),schema=dict(specification_id=family,type_name=root,schema_digest=dict(algorithm='sha256',domain='type-schema',hex=digest)),definition=definition(t),definition_closure=closure(t,types))
   result['lookup']={'found':found}
   if accepts(t['body'],v,types):
    decoded=decode_typed(t['body'],json.loads(data),types)
    validate_native_catalog(slot,decoded)
    compare(v,decoded)
    assert CAT.ace1(decoded).encode()==data
    result['validation']={'valid':dict(found=found,validated_catalog={slot:v},canonical_content=list(data),completed_steps=['catalog_canonicalization','current_encode','current_decode','native_decode','catalog_equality','byte_roundtrip'])};result['admission']='validated'
   else:
    if profile_mode=='invalid':
     field={'effect_resource':'complete_operations','recovery_usage':'rules','flow_execution':'closed_regions'}[slot]
     error={'unknown_field':dict(field=field)}
    else:
     field={'effect_resource':'supported_pairs','recovery_usage':'rules','flow_execution':'closed_regions'}[slot]
     error={'type_mismatch':dict(expected='string',actual='record<%d fields>'%len(v[field][0]))}
    result['validation']={'invalid':dict(found=found,step='current_encode',codec_failure={'some':error},input_sites=['source_catalog','current_schema'],completed_steps=['catalog_canonicalization'])}
 response=dict(computed=result,proposal_failure=None,slot_complete=result['admission'] in ('validated','absent'),raw_capture_complete=False,positive_root_export=False)
 if c['mode']=='proposal':
  op=c['mutation'];response['proposal_failure']='IncompleteFacts' if op['op']=='remove' else 'UnsupportedModel' if op['path'] in (['validation'],['positive_root_export'],['source','some','construction_schema'],['validation','valid','found','schema','owner']) else 'InvalidFacts'
 return response
def context_cases():
 return [dict(case_id=PREFIX+'context-'+mode,mode=mode) for mode in ['all-none','all-valid','recovery-missing','flow-invalid','split','wrong-compiler','missing-observation','false-complete']]
def context_expected(c):
 mode=c['mode'];selection=('recovery_usage','missing_module') if mode in ('recovery-missing','false-complete') else ('flow_execution','invalid') if mode=='flow-invalid' else ('effect_resource','A')
 result={}
 for slot,_,_ in SLOTS:
  v=case_base(slot);v.update(present=mode!='all-none',association='split' if mode=='split' else 'same',compiler='wrong' if mode=='wrong-compiler' else 'current')
  result[slot]=expected(v,selection)['computed']
 result.update(relation_partition_complete=all(result[s]['admission'] in ('validated','absent') for s,_,_ in SLOTS),scope_state='remaining_scope_incomplete')
 return dict(computed=result,proposal_failure='IncompleteFacts' if mode=='missing-observation' else 'InvalidFacts' if mode=='false-complete' else None,positive_root_export=False)

def compare(want,got):
 def eq(a,b):
  if type(a) is not type(b):return False
  if type(a) is dict:return set(a)==set(b) and all(eq(a[k],b[k]) for k in a)
  if type(a) is list:return len(a)==len(b) and all(eq(x,y) for x,y in zip(a,b))
  return a==b
 assert eq(want,got),'complete typed relation metadata mismatch'
def dag_cases():
 a=expected(case_base('effect_resource'))['computed']['source']['some']
 b=copy.deepcopy(a);b['revision']='another';b['pin']['revision']='another'
 rows=[]
 def add(name,value,error):rows.append(dict(case_id=PREFIX+'dag-'+name,input=value,expected_failure=error,scope='source-only closed metadata DAG; NOT Scope normalizer'))
 add('empty',dict(nodes=[],roots=[]),None)
 add('single',dict(nodes=[a],roots=[0]),None)
 add('distinct',dict(nodes=[a,b],roots=[0,1]),None)
 add('duplicate-content',dict(nodes=[a,a],roots=[0,1]),'InvalidFacts')
 add('unreachable',dict(nodes=[a,b],roots=[0]),'InvalidFacts')
 add('missing-roots',dict(nodes=[a]),'IncompleteFacts')
 add('out-of-range',dict(nodes=[a],roots=[1]),'InvalidFacts')
 add('wrong-root-order',dict(nodes=[a,b],roots=[1,0]),'InvalidFacts')
 c=copy.deepcopy(a);c['validation_backedge']=0
 add('output-backedge',dict(nodes=[c],roots=[0]),'UnsupportedModel')
 add('output-as-input',dict(nodes=[dict(validation={'valid':None})],roots=[0]),'IncompleteFacts')
 return rows
def report(actual):
 reference=[dict(case_id=c['case_id'],actual=expected(c)) for c in cases()]
 compare(reference,actual)
 # Roster-independent mutations preserve the original input-derived Expected.
 rejected=[]
 for field in ['source','lookup','validation','current_modules','algorithm','target']:
  changed=copy.deepcopy(actual);changed[0]['actual']['computed'].pop(field)
  try:compare(reference,changed)
  except AssertionError:rejected.append(field)
  else:raise AssertionError('omission accepted '+field)
 return dict(scope='Registration/Metadata Admission ONLY',matched=len(reference),slots=[s for s,_,_ in SLOTS],omission_mutations_rejected=rejected,raw_full_field_oracle=False,raw_capture_conformance_cases=0,positive_root_export=False)
if __name__=='__main__':
 if sys.argv[1:]==['--inputs']:print(json.dumps(cases()));sys.exit(0)
 print(json.dumps(report(json.load(sys.stdin)),indent=2))
