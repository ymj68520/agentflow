"""Generate registered relation descriptors and input-only conformance.

This generator does not execute a Raw Producer and never reads Rust Actual.
"""
import copy,hashlib,importlib.util,json,re,sys
from pathlib import Path
from relation_schema_registration_data import *
R=Path(__file__).resolve().parent.parent
OUT=R/'spec/conformance/g01-relation-schema-staging'
def module(name,path):
 s=importlib.util.spec_from_file_location(name,R/path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def write(path,raw):
 p=R/path;p.parent.mkdir(parents=True,exist_ok=True)
 if not p.exists() or p.read_bytes()!=raw:p.write_bytes(raw)
def js(path,v):write(path,(json.dumps(v,ensure_ascii=False,sort_keys=True,indent=2)+'\n').encode())
def readmodel(path):return json.loads(re.search(r'```json\s*(.*?)\s*```',(R/path).read_text(encoding='utf8'),re.S)[1])
def ref(n):return {'ref':n}
def rec(**f):return {'kind':'record','fields':list(map(list,f.items()))}
def seq(d):return {'kind':'sequence','item':d,'minimum':0}
def opt(d):return {'kind':'option','item':d,'minimum':0}
def en(**v):return {'kind':'enum','variants':list(map(list,v.items()))}
def node(n):return {'kind':'dag_ref','target':ref(n)}
U={'kind':'unit'};TEXT=ref('$Text');BYTES=ref('$Bytes')

def definitions(old):
 a=module('relation_independent_afidl','tools/gen-afidl-corpus.py')
 _,compiled=a.compile_module_set([(p.relative_to(R).as_posix(),p.read_text(encoding='utf8')) for p in sorted((R/'spec/idl').rglob('*.afidl'))])
 schemas={t['schema']+'/'+t['name']:t for ts in compiled.values() for t in ts}
 d={}
 primitives={'unit':'unit','null':'null','bool':'bool','uint32':'u32','uint64':'u64','int32':'i32','int64':'i64','string':'nfc_text','bytes':'bytes','decimal':'decimal','float64':'finite_f64','timestamp':'timestamp_i64','duration':'duration_i64'}
 def wire(expr):
  k,v=expr
  if k=='primitive':return {'kind':primitives[v]}
  if k=='ref':
   name='RelationWire/'+v
   if name not in d:
    d[name]=None;d[name]=body(schemas[v]['body'])
   return ref(name)
  if k=='list':return seq(wire(v))
  if k=='option':return opt(wire(v))
  if k=='map':return {'kind':'map','key':wire(v[0]),'value':wire(v[1])}
  raise ValueError(expr)
 def body(expr):
  k,v=expr
  if k=='alias':return wire(v)
  if k=='enum':return en(**{r['name']:U for r in v['values']})
  if k=='record':return rec(**{r['name']:wire(r['type']) for r in v['fields']})
  if k=='variant':return en(**{r['name']:wire(r['type']) for r in v['cases']})
  raise ValueError(expr)
 catalogs={s:wire(['ref',family+'/'+root]) for s,root,family in SLOTS}
 d.update({
  'RelationSlot':en(**{s:U for s,_,_ in SLOTS}),
  'RelationSourceFamily':{'kind':'nfc_text','nominal':'RelationSourceFamily','constraint':'strict SpecificationId; exact closed slot family'},
  'RelationPlainChecksum':rec(algorithm=TEXT,hex=TEXT),
  'NativeRelationFamilyPin':rec(family=ref('RelationSourceFamily'),revision=TEXT,canonical_content_checksum=ref('RelationPlainChecksum')),
  'NativeRelationCatalog':en(**catalogs),
  'NativeRelationSnapshotSource':rec(kind=ref('RelationSlot'),schema_family=ref('RelationSourceFamily'),revision=TEXT,pin=ref('NativeRelationFamilyPin'),canonical_content=BYTES,catalog=ref('NativeRelationCatalog')),
  'RelationMissingPart':en(module=U,root=U,definition=U),
  'RelationValidationStep':en(catalog_canonicalization=U,current_encode=U,current_decode=U,native_decode=U,catalog_equality=U,byte_roundtrip=U),
  'RelationInvalidInputSite':en(source_catalog=U,current_schema=U,canonical_content=U),
  'RelationCodecFailure':en(invalid_integer=rec(input=TEXT),integer_out_of_range=rec(input=TEXT,type_name=TEXT),invalid_decimal=rec(input=TEXT),non_finite_float=U,invalid_unicode=rec(detail=TEXT),duplicate_normalized_key=rec(key=TEXT),duplicate_field=rec(field=TEXT),unknown_field=rec(field=TEXT),missing_field=rec(field=TEXT),invalid_bytes_encoding=rec(detail=TEXT),unknown_enum_value=rec(value=TEXT),unknown_variant_case=rec(case=TEXT),invalid_option_encoding=rec(detail=TEXT),type_mismatch=rec(expected=TEXT,actual=TEXT),non_canonical_encoding=rec(detail=TEXT),invalid_shape=rec(detail=TEXT)),
  'RelationPriorFailure':en(invalid_identity=U,invalid_context=U,missing_source=U,invalid_source=U,unsupported_model=U,source_association=U),
  'RelationLookupTarget':rec(slot=ref('RelationSlot'),family=ref('RelationSourceFamily'),root_type=TEXT),
  'RelationFoundSchema':rec(target=ref('RelationLookupTarget'),schema=ref('af-semantic::TypeSchemaRef'),definition=ref('af-idl::CanonicalTypeSchema'),definition_closure=seq(ref('af-idl::CanonicalTypeSchema'))),
  'RelationLookupResult':en(not_reached=rec(cause=ref('RelationPriorFailure')),missing=rec(target=ref('RelationLookupTarget'),part=ref('RelationMissingPart'),missing_identity=TEXT),ambiguous=rec(target=ref('RelationLookupTarget'),definitions=seq(ref('af-idl::CanonicalTypeSchema'))),family_mismatch=rec(target=ref('RelationLookupTarget'),source_family=TEXT),revision_mismatch=rec(actual=ref('NativeRelationFamilyPin'),expected=ref('NativeRelationFamilyPin')),found=ref('RelationFoundSchema')),
  'RelationValidationResult':en(not_reached=U,invalid=rec(found=ref('RelationFoundSchema'),step=ref('RelationValidationStep'),codec_failure=opt(ref('RelationCodecFailure')),input_sites=seq(ref('RelationInvalidInputSite')),completed_steps=seq(ref('RelationValidationStep'))),valid=rec(found=ref('RelationFoundSchema'),validated_catalog=ref('NativeRelationCatalog'),canonical_content=BYTES,completed_steps=seq(ref('RelationValidationStep')))),
  'RelationPartitionAdmission':en(absent=U,incomplete=U,validated=U),
  'RelationMetadataAssurance':en(relation_slot_only=U),
  'RelationAlgorithmIdentity':rec(analysis=TEXT,model=TEXT,lookup_rule=ref('RuleId'),validation_rule=ref('RuleId')),
  'RelationObservedContextRoles':seq(seq(ref('RawContextRole'))),
  'RelationCurrentValidationMetadata':rec(assurance=ref('RelationMetadataAssurance'),algorithm=ref('RelationAlgorithmIdentity'),target=ref('RelationLookupTarget'),source=opt(ref('NativeRelationSnapshotSource')),context_roles=ref('RelationObservedContextRoles'),current_modules=seq(ref('CompiledModuleContents')),lookup=ref('RelationLookupResult'),validation=ref('RelationValidationResult'),admission=ref('RelationPartitionAdmission')),
  'RelationRemainingScope':en(remaining_scope_incomplete=U),
  'RelationContextAdmissionMetadata':rec(effect_resource=ref('RelationCurrentValidationMetadata'),recovery_usage=ref('RelationCurrentValidationMetadata'),flow_execution=ref('RelationCurrentValidationMetadata'),relation_partition_complete=ref('$Bool'),scope_state=ref('RelationRemainingScope')),
  'CurrentContextRelationSchemaValidation':rec(source=node('NativeRelationSnapshotSource'),context=node('ContractProofContextContents'),algorithm=ref('RelationAlgorithmIdentity'),target=ref('RelationLookupTarget'),lookup=ref('RelationLookupResult'),validation=ref('RelationValidationResult')),
  'RelationSubjectAdmission':rec(subject=node('RawValidationSubject'),observations=seq(ref('CurrentContextRelationSchemaValidation')),relation_partition_complete=ref('$Bool')),
  'RelationSourceDagMetadata':rec(nodes=seq(ref('NativeRelationSnapshotSource')),roots=seq(ref('$U64'))),
 })
 assert not set(d)&set(old['types']) and all(v is not None for v in d.values())
 return d

def sections(key):
 return ''.join('\n<a id="'+r['anchor']+'"></a>\n\n## '+r['rule_id']+' — '+r['draft']+'\n\nMUST: '+r['statement']+'\n\nDirect normative dependencies: '+', '.join(r['dependencies'])+'.\nRegistration conformance: '+r['case_id']+'; NOT Raw Full-field Capture.\n' for r in RULES if r['source']==key)

def build():
 old=readmodel('spec/tables/proof-report-model-v5.md');m=copy.deepcopy(old);new=definitions(old)
 m['types'].update(new)
 # Only the successor owner interprets this changed source type. The exact old
 # catalogue/ranks/references are retained in inherited_registered_model.
 m['types']['RelationSnapshotContent']=ref('NativeRelationSnapshotSource')
 m.update(model_id=MODEL,model_edition=SHAPE,obligation_edition=OBLIGATION,analysis_owner=OWNER,profile='relation_source_current_validation',inherited_registered_model=old)
 m['type_ranks']={n:i for i,n in enumerate(m['types'])}
 m['descriptor_roots']=old['descriptor_roots']+list(new)
 m['relation_registration']=dict(authored_types=list(new),changed_types=['RelationSnapshotContent'],frozen_model='agentflow.computation.proof-report.model@5',raw_producer='Incomplete',scope='NotImplemented',normalizer='NotImplemented',positive_capability='NotAuthorized',metadata_scope='relation_slot_and_partition_only',slot_table=[dict(slot=s,native_type='af-contract::'+root,family=f,root_type=root,source_rule=RULES[0]['rule_id'],lookup_rule=RULES[2]['rule_id'],validation_rule=RULES[6]['rule_id']) for s,root,f in SLOTS])
 m['canonical_constraints']=[x.replace('RelationSnapshotContent={pin:{schema:TypeSchemaRef,revision:Text,canonical_content_checksum:{algorithm:Text,hex:Text}},canonical_content:Bytes}.','RelationSnapshotContent=NativeRelationSnapshotSource; its pin is a native family and C is a separate current validation observation.').replace('all62 source bindings here,36 selected specifications and188 applied rules','all66 source bindings here,40 selected specifications and204 applied rules') for x in old['canonical_constraints']]+[
  'Inherited descriptor prose retains old-owner meaning; under this successor RelationSnapshotContent is NativeRelationSnapshotSource (family, not construction TypeSchemaRef). C is a separate output.',
  'RelationWire descriptors are fixed registered lossless Native RelationCatalogWire projections from the17 frozen AFIDL grammars, not a current schema lookup or recovered construction schema.',
  'C uses actual modules only; full unique canonical root and imported closure accompany its current schema pin. Found is not Valid; all applicable steps must execute.',
  'RelationCurrentValidationMetadata is an explicit finite metadata companion: complete modules/source/result, actual native same-context association, not a cross-process complete nine-member Context or Scope.',
  'Formal C source/context Node references point to immutable inputs only. Raw Source/Origin/Basis/Witness/Contradiction/EvidenceRef closure uses successor Context; no output validation is inserted into input identity.',
  'Every actual Some needs Valid for relation partition admission; None remains absent. This cannot close other compiler/context/Fact fields or issue positive authority.']
 return m

def model():
 # Preserve exact prior-version input registries independent of new globals.
 old=json.loads((R/'spec/conformance/g01-document-occurrence-staging/compiler-current.json').read_bytes())
 for field,name in [('registry_hex','specifications'),('rule_registry_hex','rules')]:
  write('spec/conformance/g01-relation-schema-staging/analysis6-'+name+'.json',bytes.fromhex(old[field]))
 for key,(_,_,path,_) in SOURCES.items():
  if key=='RM':continue
  text=(R/path).read_text(encoding='utf8');pos=text.find('\n<a id=');text=text if pos<0 else text[:pos]
  write(path,(text.rstrip()+'\n'+sections(key)).encode())
 m=build();write(SOURCES['RM'][2],('# '+MODEL+'\n\nRegistered relation staging; NOT Raw Producer/Scope/Normalizer/activation. Exact model@5 retained.\n\n```json\n'+json.dumps(m,ensure_ascii=False,separators=(',',':'))+'\n```\n'+sections('RM')).encode())
 return m

def corpus(m):
 helper=module('relation_legacy_shape_samples','tools/gen-document-occurrence-staging.py')
 t=m['types'];cases=[];coverage=[]
 def sample(desc,stack=()):
  if 'ref' in desc:
   n=desc['ref']
   if n=='RelationSourceFamily':return SLOTS[0][2]
   if n=='DocumentSourceKey':return [120]
   if n=='ContextRolePartition':return [['analysis','replacement_checked'],['old_checked']]
   if n in ('ModelRef','ModelType'):return {'owner':MODEL,'name':'NativeRelationSnapshotSource'}
   if n in ('RawContextPartition','NativeGuardContextPartition'):return [['analysis','canonical_validation']]
   if n in stack:raise ValueError(('required recursion',n))
   return sample(t[n],stack+(n,))
  k=desc['kind']
  if k=='record':return {n:sample(d,stack) for n,d in desc['fields']}
  if k=='enum':
   tag,d=desc['variants'][0];return tag if all(v.get('kind')=='unit' for _,v in desc['variants']) else {tag:sample(d,stack)}
  if k in ('sequence','set'):return [sample(desc['item'],stack) for _ in range(desc.get('minimum',0))]
  if k=='map':return []
  if k=='option':return {'none':None}
  if k in ('unit','null'):return None
  if k=='nfc_text':return 'x'
  if k=='bool':return False
  if k=='bytes':return [0]*desc.get('length',0)
  if k in ('decimal',):return '0'
  return 0
 def add(slug,ty,value,failure=None):
  cid=PREFIX+slug;cases.append(dict(case_id=cid,type=ty,value=copy.deepcopy(value),expected_failure=failure,scope='Shape only'));return cid
 def members(ty,payload,value,tag=None):
  for name,_ in payload['fields']:
   c=copy.deepcopy(value);(c[tag] if tag else c).pop(name)
   cid=add('missing-'+ty.replace('/','_')+('-'+tag if tag else '')+'-'+name,ty,c,'IncompleteFacts');coverage.append(dict(type=ty,variant=tag,field=name,case_id=cid))
  c=copy.deepcopy(value);(c[tag] if tag else c)['unknown_extra']=None
  add('extra-'+ty.replace('/','_')+('-'+tag if tag else ''),ty,c,'UnsupportedModel')
 for n in m['relation_registration']['authored_types']:
  d=t[n];v=sample(ref(n));add('shape-'+n.replace('/','_'),n,v)
  if d.get('kind')=='record':members(n,d,v)
  if d.get('kind')=='enum':
   for tag,payload in d['variants']:
    v=tag if all(x.get('kind')=='unit' for _,x in d['variants']) else {tag:sample(payload)}
    cid=add('variant-'+n.replace('/','_')+'-'+tag,n,v);coverage.append(dict(type=n,variant=tag,case_id=cid))
    if payload.get('kind')=='record':members(n,payload,v,tag)
   add('unknown-'+n.replace('/','_'),n,'unregistered','UnsupportedModel')
 for ty in ['RawValidationSubject','Source','RawPortCheckBasis','RawPortRejectionWitness','TypedContradiction','EvidenceRef','Origin','FactKey']:
  add('propagation-'+ty,ty,sample(ref(ty)))
 for ty in ['ModelType','ModelRef']:
  for owner,fail in [(MODEL,None),('agentflow.computation.proof-report.model@5','UnsupportedModel')]:
   add('owner-'+ty+'-'+owner,ty,dict(owner=owner,name='NativeRelationSnapshotSource'),fail)
 js('spec/conformance/g01-relation-schema-staging/shape-cases.json',dict(cases=cases,coverage=coverage,raw_capture_case_ids=0))
 js('spec/conformance/g01-relation-schema-staging/rule-registration.json',dict(cases=RULES))
 js('spec/conformance/g01-relation-schema-staging/slot-matrix.json',dict(slots=m['relation_registration']['slot_table']))
 # Corpus values/Expected for real admission are generated by the independent
 # input-only Python oracle; no Rust result participates.
 oracle=module('relation_metadata_inputs','tools/oracle/relation_schema_admission.py')
 inputs=oracle.cases();js('spec/conformance/g01-relation-schema-staging/admission-cases.json',dict(cases=inputs))
 js('spec/conformance/g01-relation-schema-staging/admission-expected.json',dict(cases=[dict(case_id=c['case_id'],expected=oracle.expected(c)) for c in inputs]))
 dags=oracle.dag_cases();js('spec/conformance/g01-relation-schema-staging/dag-cases.json',dict(cases=dags))
 contexts=oracle.context_cases();js('spec/conformance/g01-relation-schema-staging/context-cases.json',dict(cases=contexts))
 js('spec/conformance/g01-relation-schema-staging/context-expected.json',dict(cases=[dict(case_id=c['case_id'],expected=oracle.context_expected(c)) for c in contexts]))
 codec=module('relation_portable_inputs','tools/gen-portable-proof-codec-corpus.py')
 from portable_proof_python_oracle import canonical_bytes
 old=json.loads((R/'spec/conformance/g01-document-occurrence-staging/manifest.json').read_bytes())
 ids=sorted(old['selected_specifications']+[v[0] for v in SOURCES.values()]);rids=sorted(old['applied_rules']+[r['rule_id'] for r in RULES])
 # Frozen Analysis@7 conformance keeps its own exact Registry after the
 # structural successor is registered. No old Expected is relabelled.
 frozen=R/'spec/conformance/g01-structural-binding-staging'
 sb=(frozen/'analysis7-specifications.json').read_bytes();rb=(frozen/'analysis7-rules.json').read_bytes()
 profile=dict(specification_bytes=sb,rule_bytes=rb,version=OWNER,spec_ids=ids,rule_ids=rids)
 portable=[]
 def pc(slug,p,stage,family):
  file='portable/'+slug+'.ace1';write('spec/conformance/g01-relation-schema-staging/'+file,canonical_bytes(codec.bundle_and_context(p)));portable.append(dict(case_id=PREFIX+'portable-'+slug,file=file,stage=stage,family=family,decoder_policy='bounded_32m_scalar'))
 pc('current-denied',profile,'D4','UnsupportedSemantics')
 for n in range(1,7):pc('old-label-new-content-'+str(n),{**profile,'version':'agentflow.root-export-checker@'+str(n)},'D4','InvalidIdentity')
 oi=json.loads((R/'spec/conformance/g01-document-occurrence-staging/compiler-current.json').read_bytes())
 prior=dict(specification_bytes=bytes.fromhex(oi['registry_hex']),rule_bytes=bytes.fromhex(oi['rule_registry_hex']),version=OWNER,spec_ids=old['selected_specifications'],rule_ids=old['applied_rules'])
 pc('new-label-old-content',prior,'D4','InvalidIdentity');pc('unknown',{**profile,'version':'unknown.checker@999'},'D4','UnsupportedSemantics')
 portable.append(dict(case_id=PREFIX+'portable-default-resource-limit',file='portable/current-denied.ace1',stage='D0',family='ResourceLimit',decoder_policy='unchanged_default'))
 js('spec/conformance/g01-relation-schema-staging/manifest.json',dict(analysis=OWNER,model=MODEL,report=SHAPE,obligation=OBLIGATION,case_ids=[c['case_id'] for c in cases+RULES+inputs+dags+portable+contexts]+[PREFIX+'native-equality-'+s for s,_,_ in SLOTS]+[PREFIX+'portable-caller-support-flag',PREFIX+'portable-noncanonical'],shape_cases=len(cases),admission_cases=len(inputs),context_cases=len(contexts),dag_cases=len(dags),rule_cases=len(RULES),portable_cases=portable,selected_specifications=ids,applied_rules=rids,coverage=coverage,raw_capture_case_ids=0,raw_producer='Incomplete',normalizer='NotImplemented',scope='NotImplemented',positive_capability='NotAuthorized'))
 print('RELATION REGISTRATION',len(t),'descriptors',len(cases),'shape',len(inputs),'admission')

if __name__=='__main__':
 m=model()
 if '--model-only' not in sys.argv:corpus(m)
