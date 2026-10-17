"""C1G2 static registered descriptors/shape fixtures; NOT a Capture Oracle."""
import copy,hashlib,importlib.util,json,re,sys
from pathlib import Path
from raw_validation_registration_data import *
R=Path(__file__).resolve().parent.parent;OUT=R/'spec/conformance/g01-raw-validation-staging'
def sha(b):return hashlib.sha256(b).hexdigest()
def write(p,b):
 p=R/p;p.parent.mkdir(parents=True,exist_ok=True)
 if not p.exists() or p.read_bytes()!=b:p.write_bytes(b)
def js(p,x):write(p,(json.dumps(x,ensure_ascii=False,sort_keys=True,indent=2)+'\n').encode())
def load(p):return json.loads((R/p).read_bytes())
def ref(n):return {'ref':n}
def rec(**f):return {'kind':'record','fields':[[k,v] for k,v in f.items()]}
def enum(**f):return {'kind':'enum','variants':[[k,v] for k,v in f.items()]}
def seq(x,minimum=0):return {'kind':'sequence','item':x,'minimum':minimum}
def opt(x):return {'kind':'option','item':x,'minimum':0}
def node(n):return {'kind':'dag_ref','target':ref(n)}
U={'kind':'unit'};U32=ref('$U32');U64=ref('$U64');B=ref('$Bytes')
def oldmodel():return json.loads(re.search(r'```json\s*(.*?)\s*```',(R/'spec/tables/proof-report-model-v3.md').read_text(encoding='utf-8'),re.S)[1])
def definitions():
 s=node('RawValidationSubject');o=ref('RawPortOccurrence');origin=ref('RawInputOrigin')
 return {
 'RawUtf8Bytes':{'kind':'bytes','constraint':'valid UTF8; no NFC normalization'},
 'RawDocument':enum(physical_json_utf8=rec(bytes=B),declared_native_document=rec(declarations=seq(ref('RawCandidateContract')))),
 'RawDocumentMember':enum(field=rec(name=ref('RawUtf8Bytes')),entry=rec(ordinal=U64)),
 'RawInputOrigin':rec(document=node('RawDocument'),path=seq(ref('RawDocumentMember'))),
 'RawPhysicalAssociation':enum(not_supplied=U,supplied=rec(document=node('RawDocument'),value_path=seq(ref('RawDocumentMember')))),
 'RawParsedInput':rec(candidate=ref('RawCandidateContract'),origin=origin,physical=ref('RawPhysicalAssociation')),
 'RawContextRole':enum(analysis=U,canonical_validation=U),
 'RawContextPartition':seq(seq(ref('RawContextRole'),1),1),
 'RawContextBinding':rec(contents=ref('ContractProofContextContents'),roles=ref('RawContextPartition')),
 'RawValidationSubject':rec(owner_compiler=ref('CompleteCompiler'),input=ref('RawParsedInput'),context=ref('RawContextBinding')),
 'RawValidationCase':enum(positive_unique_io_ports=U),
 'RawValidationClause':enum(positive_unique_io_ports=U),
 'RawValidationSubstage':enum(**{'canonical_validation/io_port_check':U}),
 'RawPortDirection':enum(input=U,output=U),
 'RawPortMember':enum(port_no=U),
 'RawPortOccurrence':rec(direction=ref('RawPortDirection'),original_ordinal=U64,sorted_ordinal=U64,port_no=U32),
 'RawValidationOrigin':rec(input_origin=origin,direction=ref('RawPortDirection'),original_ordinal=U64),
 'ExactCompiledComponentContractSchemaReference':rec(module=ref('SpecificationId'),type_name=ref('RawUtf8Bytes'),definition=ref('af-idl::CanonicalTypeSchema')),
 'ExactRuleContentReference':rec(identity=ref('af-computation::RuleContentIdentity')),
 'RawPortConstraint':enum(positive_unique_io_ports=U),
 'RawPortPredecessor':enum(none=U,some=rec(original_ordinal=U64,port_no=U32)),
 'RawPortCondition':enum(zero=U,duplicate=U,positive_fresh=U),
 'RawPortSelectedError':enum(invalid_or_duplicate_port=U),
 'RawPortCheckBasis':rec(subject=s,occurrence=o,predecessor=ref('RawPortPredecessor'),condition=ref('RawPortCondition'),selected_error=opt(ref('RawPortSelectedError')),rule=ref('ExactRuleContentReference')),
 'RawPortRejectionWitness':enum(zero=rec(subject=s,occurrence=o,raw_member_value=U32,minimum=U32),duplicate=rec(subject=s,occurrence=o,previous_original_ordinal=U64,previous_port_no=U32)),
 'RawPrefixStop':enum(component_identity=U,schema_resolution=rec(occurrence=o),port_rejection=rec(occurrence=o)),
 'RawPortApplicability':enum(reached=rec(occurrence=o),not_reached=rec(direction=ref('RawPortDirection'),original_ordinal=U64,stopped_by=ref('RawPrefixStop')),stopped_by_earlier_failure=rec(cause=ref('RawPrefixStop')),no_port=rec(direction=ref('RawPortDirection'))),
 'RawValidationStageState':enum(raw_physical=U,parsed_declaration=U,canonical_diagnostic_presentation=U,canonical_validated_declaration=U,checked_contract=U),
 'RawValidationAdmissionFailure':enum(**{k:U for k in ['MissingRequiredSource','MissingDeclarativeOrigin','InvalidSourceAssociation','InvalidContextBinding','IncompleteContent','InvalidPhysicalInput','UnsupportedPredicate','NotImplemented','InvalidIdentity','UnsupportedModel','ResourceLimit']}),
 'RawValidationFragment':rec(shape=ref('ModelEdition'),profile=ref('RawValidationProfile'),scope=ref('Scope'),applicability=seq(ref('RawPortApplicability')),facts={'kind':'map','key':ref('FactKey'),'value':ref('Fact')},prerequisites={'kind':'map','key':ref('PrerequisiteKey'),'value':ref('Prerequisite')},obligations=seq(ref('Obligation'))),
 'RawValidationProfile':enum(raw_contract_validation=U)}
def build():
 old=oldmodel();m=copy.deepcopy(old);t=m['types'];new=definitions();assert not set(new)&set(t);t.update(new)
 parents={
 'ScopeSubject':('raw_contract_validation',ref('RawValidationSubject')),
 'PredicateFamily':('raw_contract_validation',U),'ClosedPredicateCase':('raw_contract_validation',ref('RawValidationCase')),
 'RequirementClause':('raw_validation',ref('RawValidationClause')),
 'Source':('raw_validation',rec(subject=node('RawValidationSubject'))),
 'Target':('raw_declaration_constraint',rec(subject=node('RawValidationSubject'),schema=ref('ExactCompiledComponentContractSchemaReference'),rule=ref('ExactRuleContentReference'),direction=ref('RawPortDirection'),constraint=ref('RawPortConstraint'))),
 'Coordinate':('raw_port',rec(occurrence=ref('RawPortOccurrence'),member=ref('RawPortMember'))),
 'TypedContradiction':('raw_validation',rec(witness=ref('RawPortRejectionWitness'))),
 'EvidenceRef':('raw_validation_member',rec(subject=node('RawValidationSubject'),source_member_path=seq(ref('DeclaredStep')))),
 'ObservationOwner':('raw_contract_validation',U)}
 for n,(tag,d) in parents.items():t[n]['variants'].append([tag,d])
 t['Fact']['fields'].append(['raw_validation_basis',opt(ref('RawPortCheckBasis'))])
 t['Origin']['fields'].append(['raw_validation_origin',opt(ref('RawValidationOrigin'))])
 t['Fragment']['fields'][1][1]['variants'].append(['raw_contract_validation',U])
 m.update(model_id=MODEL,model_edition=SHAPE,obligation_edition=OBLIGATION,analysis_owner=OWNER,profile='raw_contract_validation',inherited_registered_model=old)
 m['descriptor_roots']=list(old['descriptor_roots'])+list(new)
 m['closed_predicates']['raw_contract_validation']=['positive_unique_io_ports']
 m['raw_case_descriptors']=[dict(family='raw_contract_validation',case='positive_unique_io_ports',rank=0,kind='prerequisite',role='supporting_fact',stage='contract_validation',substage='canonical_validation/io_port_check',requirement={'family':'raw_validation','clause':'positive_unique_io_ports','evidence_class':'static_declaration'},governing_rule='AF-CONTRACT-0004',source_variant='raw_validation',target_variant='raw_declaration_constraint',coordinate_variant='raw_port',origin_type='RawValidationOrigin',assertion_path='known-empty')]
 m['raw_registration']={'case_count':1,'authored_types':list(new),'parent_extensions':{k:v[0] for k,v in parents.items()},'record_extensions':{'Fact':'raw_validation_basis','Origin':'raw_validation_origin'},'accepted_design':'C1G1 Option C','capture_status':'NotImplemented','full_scope':'PendingG02','normalizer':'NotImplemented','positive_capability':'NotAuthorized'}
 m['type_ranks']={k:i for i,k in enumerate(t)}
 m['canonical_constraints']=[v.replace('all54 source bindings here,28 selected specifications and163 applied rules','all58 source bindings here,32 selected specifications and174 applied rules').replace('hidden 54->28 content reduction','hidden 58->32 content reduction') for v in old['canonical_constraints']]+['Historical enumeration prose describes its frozen prefix; the concrete successor tables append exactly the declared cases/fields/variants without changing old ranks or historical content.','Raw profile has exactly one parameterized port case; canonical, native, standalone and Root judgments are not rewritten.','Raw source requires actual complete document/path association and same native nine-context object, never a synthetic resolver or reconstructed physical bytes.','Origin.raw_validation_origin and Fact.raw_validation_basis are Some exactly for the raw family; other successor families require None.','Raw physical JSON metadata uses the exact closed RawCandidateContract field/byte/option representation; shape success is not validator provenance or CompleteView.']
 return m
def rule_sections(key):
 return ''.join(f'\n<a id="{r["anchor"]}"></a>\n\n## {r["rule_id"]} — {r["draft"]}\n\nMUST: {r["statement"]}\n\nDirect normative dependencies: '+', '.join(r['dependencies'])+f'.\nRegistration conformance: {r["case_id"]}; NOT typed Capture conformance.\n' for r in RULES if r['source']==key)
def model():
 for k,(_,_,p,_) in SOURCES.items():
  if k=='RM':continue
  src=(R/p).read_text(encoding='utf-8');at=src.find('\n<a id=');base=src if at<0 else src[:at]
  write(p,(base.rstrip()+'\n'+rule_sections(k)).encode())
 m=build();write(SOURCES['RM'][2],('# '+MODEL+'\n\nRegistered Raw staging; NOT Capture or authority. Exact historical model embedded unchanged.\n\n```json\n'+json.dumps(m,ensure_ascii=False,separators=(',',':'))+'\n```\n'+rule_sections('RM')).encode())
 return m
def sample(t,d,stack=()):
 if 'ref' in d:
  n=d['ref']
  if n in ('RawContextPartition','NativeGuardContextPartition'):return [['analysis','canonical_validation']]
  if n in ('ModelType','ModelRef'):return {'owner':MODEL,'name':'RawPortDirection'}
  if n in stack:raise ValueError(('required recursion',n))
  value=sample(t,t[n],stack+(n,))
  if n=='RawPortCheckBasis':value['selected_error']={'some':'invalid_or_duplicate_port'}
  if n=='RawPortRejectionWitness':value['zero']['minimum']=1
  return value
 k=d['kind']
 if k in ('unit','null'):return None
 if k=='bool':return False
 if k in ('u8','u32','u64','i32','i64','timestamp_i64','duration_i64','dag_ref'):return 0
 if k=='nfc_text':return 'x'
 if k=='bytes':return [0]*d.get('length',0)
 if k=='decimal':return '0'
 if k=='finite_f64':return 0.0
 if k=='record':return {n:sample(t,v,stack) for n,v in d['fields']}
 if k=='option':return {'none':None}
 if k=='map':return []
 if k in ('sequence','set'):return [sample(t,d['item'],stack) for _ in range(d.get('minimum',0))]
 if k=='enum':
  a,b=d['variants'][0];return a if all(x.get('kind')=='unit' for _,x in d['variants']) else {a:sample(t,b,stack)}
 raise ValueError(k)
def corpus(m):
 t=m['types'];cases=[];coverage=[];prefix='g01-raw-validation-staging/'
 def add(slug,ty,value,fail=None):
  cid=prefix+slug;cases.append(dict(case_id=cid,type=ty,value=copy.deepcopy(value),expected_failure=fail,classification='Registration Conformance;NOT Capture'));return cid
 def field_cases(ty,d,base,variant=None):
  for n,_ in d['fields']:
   v=copy.deepcopy(base);(v[variant] if variant else v).pop(n)
   cid=add('missing-'+ty+'-'+(variant+'-' if variant else '')+n,ty,v,'IncompleteFacts')
   coverage.append(dict(type=ty,variant=variant,field=n,case_id=cid))
  v=copy.deepcopy(base);(v[variant] if variant else v)['unregistered_extra']=None
  add('extra-'+ty+('-'+variant if variant else ''),ty,v,'UnsupportedModel')
 for ty in m['raw_registration']['authored_types']:
  d=t[ty];value=sample(t,ref(ty));add('shape-'+ty,ty,value)
  if d.get('kind')=='record':field_cases(ty,d,value)
  if d.get('kind')=='enum':
   for tag,p in d['variants']:
    v=tag if all(x.get('kind')=='unit' for _,x in d['variants']) else {tag:sample(t,p)}
    if ty=='RawPortRejectionWitness' and tag=='zero':v[tag]['minimum']=1
    if ty=='RawPortRejectionWitness' and tag=='duplicate':
     v[tag]['occurrence'].update(original_ordinal=1,sorted_ordinal=1,port_no=1);v[tag]['previous_port_no']=1
    cid=add('variant-'+ty+'-'+tag,ty,v);coverage.append(dict(type=ty,variant=tag,case_id=cid))
    if p.get('kind')=='record':field_cases(ty,p,v,tag)
   add('unknown-'+ty,ty,'unregistered_variant','UnsupportedModel')
 for ty,tag in m['raw_registration']['parent_extensions'].items():
  d=dict(t[ty]['variants'])[tag];v=tag if all(x.get('kind')=='unit' for _,x in t[ty]['variants']) else {tag:sample(t,d)}
  cid=add('parent-'+ty+'-'+tag,ty,v);coverage.append(dict(type=ty,variant=tag,case_id=cid))
  if d.get('kind')=='record':field_cases(ty,d,v,tag)
 for ty,field in m['raw_registration']['record_extensions'].items():
  v=sample(t,ref(ty));cid=add('parent-'+ty,ty,v);coverage.append(dict(type=ty,field=field,case_id=cid));v.pop(field);add('parent-missing-'+ty,ty,v,'IncompleteFacts')
  v=sample(t,ref(ty));v[field]={'some':sample(t,t[ty]['fields'][-1][1]['item'])};add('parent-some-'+ty,ty,v)
 for field,_ in t['ContractProofContextContents']['fields']:
  v=sample(t,ref('RawContextBinding'));v['contents'].pop(field);add('context-missing-'+field,'RawContextBinding',v,'IncompleteFacts')
 for field in ['effect_resource','recovery_usage','flow_execution']:
  v=sample(t,ref('RawContextBinding'));v['contents']['relation_snapshots'].pop(field);add('relation-slot-missing-'+field,'RawContextBinding',v,'IncompleteFacts')
 # Fragment's new profile field is a required inline closed-variant extension.
 v=sample(t,ref('Fragment'));v['profile']='raw_contract_validation';cid=add('parent-Fragment-profile','Fragment',v);coverage.append(dict(type='Fragment',field='profile',variant='raw_contract_validation',case_id=cid))
 for tag,val in [('missing',[['analysis']]),('split',[['analysis'],['canonical_validation']]),('duplicate',[['analysis','canonical_validation','analysis']]),('reversed',[['canonical_validation','analysis']])]:add('partition-'+tag,'RawContextPartition',val,'InvalidFacts')
 value=sample(t,ref('RawContextBinding'));add('resolver-not-supplied','RawContextBinding',value)
 value['contents']['resolver_dependencies']=[];add('resolver-known-empty-not-native-absence','RawContextBinding',value,'UnsupportedModel')
 add('nfd-utf8-bytes','RawUtf8Bytes',[101,204,129]);add('bad-utf8','RawUtf8Bytes',[255],'InvalidFacts')
 add('u32-zero','RawPortOccurrence',{'direction':'input','original_ordinal':0,'sorted_ordinal':0,'port_no':0})
 add('distinct-original-sorted','RawPortOccurrence',{'direction':'input','original_ordinal':10,'sorted_ordinal':2,'port_no':3})
 add('u32-overflow','RawPortOccurrence',{'direction':'input','original_ordinal':0,'sorted_ordinal':0,'port_no':4294967296},'UnsupportedModel')
 for schema in ['ModelRef','ModelType']:
  for owner,expected in [(MODEL,None),('agentflow.computation.proof-report.model@3','UnsupportedModel'),('unknown.owner@99','UnsupportedModel')]:
   add('owner-'+schema+'-'+owner,schema,{'owner':owner,'name':'RawValidationSubject'},expected)
 js('spec/conformance/g01-raw-validation-staging/shape-cases.json',{'cases':cases,'coverage':coverage,'claim':'Schema-derived shape coverage only; never validator-produced evidence'})
 js('spec/conformance/g01-raw-validation-staging/rule-registration.json',{'cases':RULES})
 # Independent native/portable compiler inputs derive from frozen declarations.
 spec=importlib.util.spec_from_file_location('raw_codec',R/'tools/gen-portable-proof-codec-corpus.py');codec=importlib.util.module_from_spec(spec);spec.loader.exec_module(codec)
 old=load('spec/conformance/g01-native-guard-staging/manifest.json');ids=sorted(old['selected_specifications']+[v[0] for v in SOURCES.values()]);rids=sorted(old['applied_rules']+[v['rule_id'] for v in RULES])
 frozen=R/'spec/conformance/g01-document-occurrence-staging'
 sb=(frozen/'edition5-specifications.json').read_bytes();rb=(frozen/'edition5-rules.json').read_bytes();profile=dict(specification_bytes=sb,rule_bytes=rb,version=OWNER,spec_ids=ids,rule_ids=rids)
 identity={'analysis_semantics_version':OWNER,'registry_hex':sb.hex(),'rule_registry_hex':rb.hex(),'specifications':[{'specification_id':x['specification_id'],'logical_path':x['normative_source']['logical_path'],'content_sha256':sha((R/x['normative_source']['logical_path']).read_bytes()),'content_hex':(R/x['normative_source']['logical_path']).read_bytes().hex()} for x in json.loads(sb)['specifications'] if x['specification_id'] in ids],'rules':[codec.rule_identity(x) for x in json.loads(rb)['rules'] if x['rule_id'] in rids]}
 js('spec/conformance/g01-raw-validation-staging/compiler-current.json',identity)
 from portable_proof_python_oracle import canonical_bytes
 portable=[]
 def pc(slug,p,stage,family):
  file='portable/'+slug+'.ace1';write('spec/conformance/g01-raw-validation-staging/'+file,canonical_bytes(codec.bundle_and_context(p)));portable.append(dict(case_id=prefix+'portable-'+slug,file=file,stage=stage,family=family,decoder_policy='bounded_16m_scalar'))
 pc('current-denied',profile,'D4','UnsupportedSemantics')
 for n in [1,2,3,4]:pc('old-label-new-content-'+str(n),{**profile,'version':'agentflow.root-export-checker@'+str(n)},'D4','InvalidIdentity')
 oldi=load('spec/conformance/g01-native-guard-staging/compiler-current.json');oldp=dict(specification_bytes=bytes.fromhex(oldi['registry_hex']),rule_bytes=bytes.fromhex(oldi['rule_registry_hex']),version=OWNER,spec_ids=old['selected_specifications'],rule_ids=old['applied_rules'])
 pc('new-label-old-content',oldp,'D4','InvalidIdentity');pc('unknown-edition',{**profile,'version':'unknown.checker@999'},'D4','UnsupportedSemantics')
 portable.append(dict(case_id=prefix+'default-resource-limit',file='portable/current-denied.ace1',stage='D0',family='ResourceLimit',decoder_policy='unchanged_default'))
 mutations=[dict(case_id=prefix+'portable-caller-support-flag',test='caller_flags_and_noncanonical_new_tuple_bytes_never_gain_support',family='InvalidCanonicalEncoding',stage='D0'),dict(case_id=prefix+'portable-noncanonical-current-tuple',test='caller_flags_and_noncanonical_new_tuple_bytes_never_gain_support',family='InvalidCanonicalEncoding',stage='D1')]
 js('spec/conformance/g01-raw-validation-staging/manifest.json',{'kind':'Registration Conformance;NOT Typed Capture','analysis':OWNER,'model':MODEL,'shape':SHAPE,'obligation':OBLIGATION,'case_ids':[c['case_id'] for c in cases+RULES+portable+mutations],'shape_cases':len(cases),'rule_cases':len(RULES),'portable_cases':portable,'runtime_portable_mutations':mutations,'selected_specifications':ids,'applied_rules':rids,'capture_case_ids':0,'raw_producer':'NotImplemented','positive_capability':'NotAuthorized','coverage':coverage})
 print('RAW REGISTRATION',len(cases),'shape',len(coverage),'coverage edges',len(m['types']),'types')
def main():
 m=model()
 if '--model-only' not in sys.argv:corpus(m)
if __name__=='__main__':main()
