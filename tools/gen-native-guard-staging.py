"""G2 Registration Conformance inputs; never Native Typed Capture Expected.

Shape decoder cases are deliberately schema-derived (not an independent
semantic oracle). Owner, identity, roster and native field laws are separately
authored; historical descriptor expectations derive from frozen declarations.
"""
import copy,hashlib,importlib.util,json,re,sys
from pathlib import Path
from native_guard_registration_data import OWNER,SHAPE,OBLIGATION,SOURCES,RULES,MODEL,HISTORICAL,BASELINE
from portable_proof_python_oracle import canonical_bytes
ROOT=Path(__file__).resolve().parent.parent;OUT=ROOT/'spec/conformance/g01-native-guard-staging'
def sha(b):return hashlib.sha256(b).hexdigest()
def save(p,b):
 p=OUT/p;p.parent.mkdir(parents=True,exist_ok=True)
 if not p.exists() or p.read_bytes()!=b:p.write_bytes(b)
def js(p,x):save(p,(json.dumps(x,ensure_ascii=False,sort_keys=True,indent=2)+'\n').encode())
def load(name,path):
 s=importlib.util.spec_from_file_location(name,ROOT/'tools'/path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def read_model():return json.loads(re.search(r'```json\s*(.*?)\s*```',(ROOT/SOURCES['RM'][2]).read_text(encoding='utf-8'),re.S)[1])
def sample(types,d,stack=()):
 if 'ref' in d:
  n=d['ref']
  if n=='NativeGuardContextPartition':return [['analysis','canonical_validation']]
  if n in ('ModelType','ModelRef'):return {'owner':MODEL,'name':'NativeGuardRole'}
  if n=='Sha256Hex':return '0'*64
  if n in stack:raise ValueError(('recursive required value',stack,n))
  return sample(types,types[n],stack+(n,))
 k=d['kind']
 if k in ('unit','null'):return None
 if k=='bool':return False
 if k in ('u8','u32','u64','i32','i64','timestamp_i64','duration_i64','dag_ref'):return 0
 if k=='nfc_text':return 'x'
 if k=='bytes':return [0]*d.get('length',0)
 if k=='decimal':return '0'
 if k=='finite_f64':return 0.0
 if k=='record':return {n:sample(types,v,stack) for n,v in d['fields']}
 if k=='option':return {'none':None}
 if k=='map':return []
 if k in ('sequence','set'):return [sample(types,d['item'],stack) for _ in range(d.get('minimum',0))]
 if k=='enum':
  a,b=d['variants'][0]
  return a if all(v.get('kind')=='unit' for _,v in d['variants']) else {a:sample(types,b,stack)}
 raise ValueError(k)
def main():
 m=read_model();types=m['types'];cases=[];descriptor_cases=[]
 def sc(slug,ty,value,expected=None):cases.append(dict(case_id='g01-native-guard-staging/'+slug,kind='Registration Conformance;NOT Capture',type=ty,value=value,expected_failure=expected))
 def dc(slug,ty,part,expected):descriptor_cases.append(dict(case_id='g01-native-guard-staging/'+slug,type=ty,part=part,expected=expected))
 # Independently written roster and exact native field/order laws.
 roster='canonical_source context_association action_sources action_bounds guard_inputs guard_containment guard_evaluation'.split()
 for i,c in enumerate(roster):
  sc('case-'+c,'NativeGuardCase',c)
  descriptor_cases.append(dict(case_id='g01-native-guard-staging/roster-'+c,part='native_case',case=c,expected={'rank':i,'stage':'contract_validation','role':'mandatory_preflight' if i<2 else 'supporting_fact','kind':'proof_context' if i==1 else 'native_recovery_guard_evidence','requirement':{'family':'direct' if i==1 else 'native_guard','clause':'proof_context' if i==1 else c,'evidence_class':'static_declaration'}}))
 roots={
 'ActionGuardCompatibility':'source_component recovery_action forward_entry_ordinal action_entry_ordinal forward_guard declared_action_guard action_upper verdict basis countermodel reason',
 'TypedGuardCountermodel':'source_contract source_component recovery_action forward_entry_ordinal action_entry_ordinal forward_guard declared_action_guard forward_bounds activation_upper attempt_upper action_upper assignments reachability related_rule_id',
 'GuardInputAssignment':'port_no type_ref value'}
 for name,fields in roots.items():dc('native-fields-'+name,HISTORICAL+'/af-contract::'+name,'fields',fields.split())
 dc('native-bases',HISTORICAL+'/af-contract::GuardCompatibilityBasis','variants','FrozenGuardContainment ZeroPossibleActions TypedCountermodel InsufficientEvidence'.split())
 dc('native-reachability',HISTORICAL+'/af-contract::GuardCountermodelReachability','variants',['WellTypedComponentInputDomain'])
 dc('native-context-nine','ContractProofContextContents','fields','compiled_modules type_schemas effect_catalogs resource_types flow_catalogs usage_meters recovery_operations execution_universes relation_snapshots'.split())
 dc('native-branches','NativeGuardBranch','variants','missing_entries missing_usage unisolated_forward same_atom_bounds_failure action_upper_overflow missing_port schema_unresolved type_mismatch zero_potential frozen_containment typed_countermodel no_countermodel'.split())
 dc('bound-failures','NativeBoundFailure','variants','same_atom_product_overflow same_atom_sum_overflow same_atom_divisor_overflow same_atom_lower_conflict action_upper_overflow'.split())
 dc('port-results','NativeGuardPortResult','variants','unconditional_not_required valid missing_port unresolved_schema type_mismatch'.split())
 dc('prerequisite-clauses','NativeGuardPrerequisiteClause','variants','canonical_source context_association action_sources_ready action_bounds_ready guard_input_valid containment_outcome'.split())
 dc('witness-variants','NativeGuardWitness','variants','input_countermodel missing_port constant_type_mismatch bound_failure propagated_failure'.split())
 dc('native-subject-fields','NativeRecoveryGuardSubject','fields','owner_compiler declaration context context_roles'.split())
 dc('native-fragment-fields','NativeGuardFragment','fields','shape profile scope native_records facts prerequisites obligations identity_strings blockers contradictions'.split())
 # Every native authored field and payload variant receives decoder and
 # missing/extra-field mutations. These are shape laws, not producer facts.
 selected=[n for n in types if n.startswith(('Native','CanonicalNative','Imported',HISTORICAL+'/'))]
 coverage=[]
 for ty in selected:
  d=types[ty];slug=re.sub(r'[^A-Za-z0-9_-]','-',ty)
  value=sample(types,{'ref':ty});sc('shape-'+slug,ty,value)
  if d.get('kind')=='record':
   for field,fd in d['fields']:
    v=copy.deepcopy(value);v.pop(field);sc('missing-'+slug+'-'+field,ty,v,'IncompleteFacts');coverage.append({'type':ty,'field':field,'kind':'required field drop'})
   v=copy.deepcopy(value);v['unregistered_extra']=None;sc('extra-'+slug,ty,v,'UnsupportedModel')
  if d.get('kind')=='enum':
   for variant,payload in d['variants']:
    v=variant if all(v.get('kind')=='unit' for _,v in d['variants']) else {variant:sample(types,payload)}
    sc('variant-'+slug+'-'+variant,ty,v);coverage.append({'type':ty,'variant':variant,'payload':payload})
    if payload.get('kind')=='record':
     for f,_ in payload['fields']:
      x=copy.deepcopy(v);x[variant].pop(f);sc('missing-'+slug+'-'+variant+'-'+f,ty,x,'IncompleteFacts');coverage.append({'type':ty,'variant':variant,'field':f,'kind':'required payload field drop'})
   sc('unknown-'+slug,ty,'unregistered_future_case','UnsupportedModel')
 for n in ['af-contract::ActionGuardCompatibility','af-contract::GuardCompatibilityBasis','af-contract::TypedGuardCountermodel','af-contract::GuardInputAssignment','af-contract::GuardCountermodelReachability']:
  for schema in ['ModelType','ModelRef']:
   for owner,good in [(HISTORICAL,True),(MODEL,False),('unknown.owner@1',False)]:sc('owner-'+schema+'-'+n.split('::')[-1]+'-'+owner,schema,{'owner':owner,'name':n},None if good else 'UnsupportedModel')
 sc('unselected-historical-scope','ModelType',{'owner':HISTORICAL,'name':'Scope'},'UnsupportedModel')
 sc('unknown-current-name','ModelRef',{'owner':MODEL,'name':'future'},'UnsupportedModel')
 sc('current-spoof-qualified-import','ModelType',{'owner':MODEL,'name':HISTORICAL+'/af-contract::TypedGuardCountermodel'},'UnsupportedModel')
 for slug,ty,field,failure in [('valid-field','TypedGuardCountermodel','related_rule_id',None),('unknown-field','TypedGuardCountermodel','future_field','UnsupportedModel'),('enum-not-record','GuardCompatibilityBasis','future_field','UnsupportedModel')]:
  sc('historical-'+slug,'DeclaredStep',{'field':{'owner':{'owner':HISTORICAL,'name':'af-contract::'+ty},'field':field}},failure)
 for slug,value in [('split',[['analysis'],['canonical_validation']]),('reversed',[['canonical_validation','analysis']]),('duplicate',[['analysis','canonical_validation','analysis']]),('missing',[['analysis']])]:sc('context-'+slug,'NativeGuardContextPartition',value,'InvalidFacts')
 sc('resolution-zero','NativeEffectResolution',{'present':{'ordinal':0,'entry':sample(types,types['NativeEffectOccurrence'])['entry']}})
 sc('resolution-absent','NativeEffectResolution',{'absent':None});sc('resolution-not-queried','NativeEffectResolution',{'not_queried':None})
 for parent in ['ScopeSubject','Source','Target','Coordinate','EvidenceRef','TypedContradiction','PrerequisitePredicate','RequirementClause','ClosedPredicateCase']:
  for tag,payload in types[parent]['variants']:
   if tag not in ['native_guard_subject','native_guard_binding','native_guard','native_guard_member','native_recovery_guard']:continue
   v={tag:sample(types,payload)};sc('parent-'+parent+'-'+tag,parent,v)
   if payload.get('kind')=='record':
    for field,_ in payload['fields']:
     x=copy.deepcopy(v);x[tag].pop(field);sc('parent-missing-'+parent+'-'+tag+'-'+field,parent,x,'IncompleteFacts');coverage.append({'type':parent,'variant':tag,'field':field})
 for field in ['native_guard_basis']:
  v=sample(types,{'ref':'Fact'});v.pop(field);sc('parent-Fact-missing-'+field,'Fact',v,'IncompleteFacts')
 js('shape-cases.json',{'cases':cases,'coverage':coverage,'classification':'schema-derived registration decoder tests;NOT independent native semantic Expected'})
 js('descriptor-laws.json',{'cases':descriptor_cases})
 # Complete exact identities / portable inputs from declarations, never Rust.
 codec=load('native_codec','gen-portable-proof-codec-corpus.py')
 oldids=json.loads((ROOT/'spec/conformance/g01-successor-staging/manifest.json').read_bytes())['selected_specifications']
 ids=sorted(oldids+[v[0] for v in SOURCES.values()]);rids=sorted([f'AF-CONTRACT-{n:04}' for n in range(1,23)]+[f'AF-SEMANTIC-{n:04}' for n in list(range(111,188))+[189]+list(range(191,223))+list(range(228,243))+list(range(245,261))])
 # Edition4 input bytes stay frozen when a successor is added.
 frozen=ROOT/'spec/conformance/g01-raw-validation-staging'
 sb=(frozen/'edition4-specifications.json').read_bytes();rb=(frozen/'edition4-rules.json').read_bytes()
 profile=dict(specification_bytes=sb,rule_bytes=rb,version=OWNER,spec_ids=ids,rule_ids=rids)
 def identity(p):
  return {'analysis_semantics_version':p['version'],'registry_hex':p['specification_bytes'].hex(),'rule_registry_hex':p['rule_bytes'].hex(),'specifications':[{'specification_id':x['specification_id'],'logical_path':x['normative_source']['logical_path'],'content_sha256':sha((ROOT/x['normative_source']['logical_path']).read_bytes()),'content_hex':(ROOT/x['normative_source']['logical_path']).read_bytes().hex()} for x in json.loads(p['specification_bytes'])['specifications'] if x['specification_id'] in p['spec_ids']],'rules':[codec.rule_identity(x) for x in json.loads(p['rule_bytes'])['rules'] if x['rule_id'] in p['rule_ids']]}
 js('compiler-current.json',identity(profile));base=codec.bundle_and_context(profile);portable=[]
 def pc(slug,b,stage,family):
  f='portable/'+slug+'.ace1';save(f,canonical_bytes(b));portable.append({'case_id':'g01-native-guard-staging/portable-'+slug,'file':f,'stage':stage,'family':family,'decoder_policy':'bounded_8m_scalar'})
 pc('current-denied',base,'D4','UnsupportedSemantics')
 for version in [1,2,3]:
  pc('old-label-new-registry-'+str(version),codec.bundle_and_context({**profile,'version':'agentflow.root-export-checker@'+str(version)}),'D4','InvalidIdentity')
 oldcompiler=json.loads((ROOT/'spec/conformance/g01-successor-staging/compiler-successor.json').read_bytes())
 oldp=dict(specification_bytes=bytes.fromhex(oldcompiler['registry_hex']),rule_bytes=bytes.fromhex(oldcompiler['rule_registry_hex']),version='agentflow.root-export-checker@3',spec_ids=oldids,rule_ids=[x['rule_id'] for x in oldcompiler['rules']])
 pc('exact-edition3',codec.bundle_and_context(oldp),'D4','UnsupportedSemantics')
 pc('new-label-old-registry',codec.bundle_and_context({**oldp,'version':OWNER}),'D4','InvalidIdentity')
 pc('unknown-owner',codec.bundle_and_context({**profile,'version':'unknown.checker@999'}),'D4','UnsupportedSemantics')
 fake=copy.deepcopy(base);fake['supported']=True;pc('caller-support-flag',fake,'D0','InvalidCanonicalEncoding')
 portable.append({'case_id':'g01-native-guard-staging/default-scalar-resource-denial','file':'portable/current-denied.ace1','stage':'D0','family':'ResourceLimit','decoder_policy':'unchanged_default'})
 js('rule-registration.json',{'cases':[dict(case_id=x['case_id'],rule_id=x['rule_id'],draft=x['draft'],source=SOURCES[x['source']][2],anchor=x['anchor'],statement=x['statement'],dependencies=x['dependencies']) for x in RULES]})
 allids=[x['case_id'] for x in cases+descriptor_cases+portable+RULES]
 assert len(allids)==len(set(allids))
 js('manifest.json',{'kind':'Registration Conformance;NOT Native Typed Capture','source_baseline':BASELINE,'analysis':OWNER,'shape':SHAPE,'obligation':OBLIGATION,'model':MODEL,'case_ids':allids,'case_count':len(allids),'shape_cases':len(cases),'descriptor_laws':len(descriptor_cases),'portable_cases':portable,'rule_cases':len(RULES),'applied_rules':rids,'selected_specifications':ids,'capture_case_ids':0,'native_producers':'NotImplemented','positive_capability':'NotAuthorized','g1_vectors_capture_status':'48 Not Executed','field_variant_inventory':coverage})
 print('G2 REGISTRATION CASES',len(allids),'SHAPE',len(cases),'DESCRIPTOR',len(descriptor_cases),'PORTABLE',len(portable),'RULE',len(RULES))
if __name__=='__main__':main()
