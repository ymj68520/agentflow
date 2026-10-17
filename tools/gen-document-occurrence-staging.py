"""Closed registration/shape corpus. NOT a Raw Producer or full fact Oracle."""
import copy,hashlib,importlib.util,json,re,sys
from pathlib import Path
from document_occurrence_registration_data import *
R=Path(__file__).resolve().parent.parent
OUT=R/'spec/conformance/g01-document-occurrence-staging'
def read(p):return json.loads((R/p).read_bytes())
def sha(b):return hashlib.sha256(b).hexdigest()
def write(p,b):
 p=R/p;p.parent.mkdir(parents=True,exist_ok=True)
 if not p.exists() or p.read_bytes()!=b:p.write_bytes(b)
def js(p,x):write(p,(json.dumps(x,ensure_ascii=False,sort_keys=True,indent=2)+'\n').encode())
def load(name,path):
 s=importlib.util.spec_from_file_location(name,R/path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def ref(n):return {'ref':n}
def rec(**f):return {'kind':'record','fields':[[k,v] for k,v in f.items()]}
def seq(x):return {'kind':'sequence','item':x,'minimum':0}
def en(**v):return {'kind':'enum','variants':[[k,x] for k,x in v.items()]}
def node(n):return {'kind':'dag_ref','target':ref(n)}
U={'kind':'unit'}
def definitions():
 key=ref('DocumentSourceKey');domain=ref('DocumentSourceDomain');anchor=ref('DocumentOwnerAnchor')
 d={
 'DocumentSourceKey':{'kind':'bytes','minimum_bytes':1,'maximum_bytes':4096,'utf8':True},
 'DocumentSourceDomain':rec(namespace=key,domain_key=key),
 'DocumentOwnerAnchor':rec(domain=domain,owner_key=key,revision_key=key),
 'DeclarationDocumentEntry':rec(declaration_name=key,document=node('RawDocument')),
 'ImportDocumentEntry':rec(import_name=key,document=node('RawDocument')),
 'WorkflowDocumentEntry':rec(source_name=key,document=node('RawDocument')),
 'DocumentOwnerContainer':en(declaration_bundle=rec(bundle_name=key,declarations=seq(ref('DeclarationDocumentEntry'))),import_manifest=rec(manifest_name=key,imports=seq(ref('ImportDocumentEntry'))),workflow_source_collection=rec(collection_name=key,sources=seq(ref('WorkflowDocumentEntry')))),
 'DocumentOwnerRecord':rec(anchor=anchor,container=ref('DocumentOwnerContainer')),
 'SourcePackage':rec(domain=domain,owners=seq(ref('DocumentOwnerRecord'))),
 'DocumentOccurrenceKey':rec(entry_ordinal=ref('$U64')),
 'DocumentOccurrenceBinding':rec(source_package=node('SourcePackage'),source_owner=anchor,occurrence_key=ref('DocumentOccurrenceKey'),actual_document=node('RawDocument')),
 'DocumentSourceAssurance':en(declared_only=U),
 'DocumentOccurrenceAdmissionFailure':en(**{k:U for k in ['MissingDocumentOccurrenceBinding','InvalidOccurrenceAssociation','BindingConflict','OwnerBindingConflict','MalformedSourcePackage','UnsupportedOwnerVariant','UnsupportedSourceDomain','UnsupportedPhysicalAssociation','InvalidPhysicalInput','InvalidContextBinding','InvalidIdentity','IncompleteFacts','UnsupportedModel','ResourceLimit']}),
 'DocumentSourceMigrationState':en(exact_old_content_only=U,unmigratable_missing_occurrence=U,new_declared_occurrence=U)}
 # Explicit finite inline metadata companions. They are NOT a new Node-rank
 # interpretation or Scope encoding. Every full document is preserved inline.
 for n in ['DeclarationDocumentEntry','ImportDocumentEntry','WorkflowDocumentEntry']:
  d['Metadata'+n]=copy.deepcopy(d[n]);d['Metadata'+n]['fields'][1][1]=ref('RawDocument')
 d['MetadataDocumentOwnerContainer']=en(declaration_bundle=rec(bundle_name=key,declarations=seq(ref('MetadataDeclarationDocumentEntry'))),import_manifest=rec(manifest_name=key,imports=seq(ref('MetadataImportDocumentEntry'))),workflow_source_collection=rec(collection_name=key,sources=seq(ref('MetadataWorkflowDocumentEntry'))))
 d['MetadataDocumentOwnerRecord']=rec(anchor=anchor,container=ref('MetadataDocumentOwnerContainer'))
 d['SourcePackageMetadata']=rec(domain=domain,owners=seq(ref('MetadataDocumentOwnerRecord')))
 d['DocumentOccurrenceMetadataBinding']=rec(source_package=ref('SourcePackageMetadata'),source_owner=anchor,occurrence_key=ref('DocumentOccurrenceKey'),actual_document=ref('RawDocument'))
 return d
def sections(k):
 return ''.join('\n<a id="'+r['anchor']+'"></a>\n\n## '+r['rule_id']+' — '+r['draft']+'\n\nMUST: '+r['statement']+'\n\nDirect normative dependencies: '+', '.join(r['dependencies'])+'.\nRegistration conformance: '+r['case_id']+'; NOT Producer Capture.\n' for r in RULES if r['source']==k)
def build():
 old=json.loads(re.search(r'```json\s*(.*?)\s*```',(R/'spec/tables/proof-report-model-v4.md').read_text(encoding='utf-8'),re.S)[1])
 m=copy.deepcopy(old);new=definitions();t=m['types'];assert not set(t)&set(new);t.update(new)
 t['RawInputOrigin']=rec(document_binding=node('DocumentOccurrenceBinding'),path=seq(ref('RawDocumentMember')))
 t['RawPhysicalAssociation']=en(not_supplied=U,supplied=rec(document_binding=node('DocumentOccurrenceBinding'),value_path=seq(ref('RawDocumentMember'))))
 t['RawValidationAdmissionFailure']['variants'].append(['MissingDocumentOccurrenceBinding',U])
 m.update(model_id=MODEL,model_edition=SHAPE,obligation_edition=OBLIGATION,analysis_owner=OWNER,profile='document_occurrence_raw_validation',inherited_registered_model=old)
 m['descriptor_roots']=old['descriptor_roots']+list(new)
 m['type_ranks']={k:i for i,k in enumerate(t)}
 m['document_occurrence_registration']={'authored_types':list(new),'replaced_types':['RawInputOrigin','RawPhysicalAssociation','RawValidationAdmissionFailure'],'draft_clauses':[r['draft'] for r in RULES],'producer':'NotImplemented','full_scope':'NotImplemented','normalizer':'NotImplemented','assurance':'declared_only','positive_capability':'NotAuthorized','metadata_companions':'Finite inline expansions, not Node rank tables or Scope encoding'}
 m['canonical_constraints']=[x.replace('all58 source bindings here,32 selected specifications and174 applied rules','all62 source bindings here,36 selected specifications and188 applied rules').replace('hidden 58->32 content reduction','hidden 62->36 content reduction') for x in old['canonical_constraints']]+[
 'Document occurrence=(complete declared OwnerAnchor,actual owner-container document ordinal); full SourcePackage and document contents are retained. No allocated pointer,call count,UUID,salt,report index or digest replaces this input.',
 'RawInputOrigin.document_binding and RawPhysicalAssociation.supplied.document_binding reference the same actual binding. Source/Origin/Basis/Witness/reference closure follows the new subject. RawDocument variants and all native/substitution/Root predicates are unchanged.',
 'Package domain/owner/revision keys are exact nonempty UTF8 bytes,not normalized,1..4096 bytes. Actual supplied owners are unique within a package; conflicts are checked only in supplied or explicit joint packages,not a global database.',
 'Physical origin uses supplied matching occurrence;native origin uses not_supplied. Other combinations reject UnsupportedPhysicalAssociation. DeclaredOnly is not external authentication or Root authority.',
 'Equal content still interns under AF-SEMANTIC-0217. SourcePackage input closure is finite and has no Fact/Witness/Report/Scope/binding back-edge. Inline metadata companions are explicit complete views,not a G02 encoder or trusted producer.']
 return m
def model():
 for k,(_,_,path,_) in SOURCES.items():
  if k=='RM':continue
  text=(R/path).read_text(encoding='utf-8');at=text.find('\n<a id=');text=text if at<0 else text[:at]
  write(path,(text.rstrip()+'\n'+sections(k)).encode())
 m=build();write(SOURCES['RM'][2],('# '+MODEL+'\n\nRegistered occurrence staging; NOT Producer/Scope/Normalizer/activation. Frozen model@4 embedded exactly.\n\n```json\n'+json.dumps(m,ensure_ascii=False,separators=(',',':'))+'\n```\n'+sections('RM')).encode());return m
def sample(t,d,stack=()):
 if 'ref' in d:
  n=d['ref']
  if n=='DocumentSourceKey':return [120]
  if n in ('RawContextPartition','NativeGuardContextPartition'):return [['analysis','canonical_validation']]
  if n in ('ModelType','ModelRef'):return {'owner':MODEL,'name':'DocumentSourceKey'}
  if n in stack:raise ValueError(('required recursion',n))
  return sample(t,t[n],stack+(n,))
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
  a,b=d['variants'][0];return a if all(v.get('kind')=='unit' for _,v in d['variants']) else {a:sample(t,b,stack)}
 raise ValueError(k)
def corpus(m):
 t=m['types'];cases=[];coverage=[]
 def add(slug,ty,v,fail=None):
  cid=PREFIX+slug;cases.append(dict(case_id=cid,type=ty,value=copy.deepcopy(v),expected_failure=fail,classification='Registration shape; NOT Capture'));return cid
 def fields(ty,d,value,tag=None):
  for name,_ in d['fields']:
   v=copy.deepcopy(value);(v[tag] if tag else v).pop(name)
   cid=add('missing-'+ty+('-'+tag if tag else '')+'-'+name,ty,v,'IncompleteFacts')
   coverage.append(dict(type=ty,variant=tag,field=name,case_id=cid))
  v=copy.deepcopy(value);(v[tag] if tag else v)['unregistered_extra']=None
  add('extra-'+ty+('-'+tag if tag else ''),ty,v,'UnsupportedModel')
 authored=m['document_occurrence_registration']['authored_types']+m['document_occurrence_registration']['replaced_types']
 for ty in authored:
  d=t[ty];v=sample(t,ref(ty));add('shape-'+ty,ty,v)
  if d.get('kind')=='record':fields(ty,d,v)
  if d.get('kind')=='enum':
   for tag,payload in d['variants']:
    v=tag if all(v.get('kind')=='unit' for _,v in d['variants']) else {tag:sample(t,payload)}
    cid=add('variant-'+ty+'-'+tag,ty,v);coverage.append(dict(type=ty,variant=tag,case_id=cid))
    if payload.get('kind')=='record':fields(ty,payload,v,tag)
   add('unknown-'+ty,ty,'future','UnsupportedModel')
 for ty in ['RawParsedInput','RawValidationSubject','RawValidationOrigin','Source','Origin','FactKey','Scope','CanonicalValueDag']:
  add('closure-'+ty,ty,sample(t,ref(ty)))
 add('closure-Source-raw-validation','Source',{'raw_validation':{'subject':0}})
 origin=sample(t,ref('Origin'));origin['raw_validation_origin']={'some':sample(t,ref('RawValidationOrigin'))}
 add('closure-Origin-raw-validation','Origin',origin)
 for label,value in [('missing-roots',{'nodes':[]}),('negative-root',{'nodes':[],'roots':[-1]}),('bad-node',{'nodes':[{'type':{'owner':MODEL,'name':'DocumentSourceKey'}}],'roots':[0]})]:
  add('dag-envelope-'+label,'CanonicalValueDag',value,'IncompleteFacts' if label in ['missing-roots','bad-node'] else 'UnsupportedModel')
 add('key-nfd-preserved','DocumentSourceKey',[101,204,129])
 subject=sample(t,ref('RawValidationSubject'));subject.pop('context');add('source-missing-context','RawValidationSubject',subject,'IncompleteFacts')
 add('context-roles-split','RawContextPartition',[['analysis'],['canonical_validation']],'InvalidFacts')
 add('slot-u64-max','DocumentOccurrenceKey',{'entry_ordinal':18446744073709551615})
 add('slot-u64-negative','DocumentOccurrenceKey',{'entry_ordinal':-1},'UnsupportedModel')
 add('slot-u64-float','DocumentOccurrenceKey',{'entry_ordinal':1.5},'UnsupportedModel')
 for schema in ['ModelRef','ModelType']:
  for owner,fail in [(MODEL,None),('agentflow.computation.proof-report.model@4','UnsupportedModel'),('unknown.owner@999','UnsupportedModel')]:add('owner-'+schema+'-'+owner,schema,{'owner':owner,'name':'DocumentOccurrenceBinding'},fail)
 add('empty-key','DocumentSourceKey',[],'IncompleteFacts');add('invalid-key-utf8','DocumentSourceKey',[255],'InvalidFacts')
 add('key-resource-limit','DocumentSourceKey',[120]*4097,'ResourceLimit')
 js('spec/conformance/g01-document-occurrence-staging/shape-cases.json',{'cases':cases,'coverage':coverage,'capture_case_ids':0})
 js('spec/conformance/g01-document-occurrence-staging/rule-registration.json',{'cases':RULES})
 # Exact successor metadata/Portable inputs, independent of Rust Actual.
 codec=load('occurrence_codec','tools/gen-portable-proof-codec-corpus.py')
 old=read('spec/conformance/g01-raw-validation-staging/manifest.json');ids=sorted(old['selected_specifications']+[v[0] for v in SOURCES.values()]);rids=sorted(old['applied_rules']+[r['rule_id'] for r in RULES])
 frozen=R/'spec/conformance/g01-relation-schema-staging'
 sb=(frozen/'analysis6-specifications.json').read_bytes();rb=(frozen/'analysis6-rules.json').read_bytes()
 profile=dict(specification_bytes=sb,rule_bytes=rb,version=OWNER,spec_ids=ids,rule_ids=rids)
 identity={'analysis_semantics_version':OWNER,'registry_hex':sb.hex(),'rule_registry_hex':rb.hex(),'specifications':[{'specification_id':s['specification_id'],'logical_path':s['normative_source']['logical_path'],'content_sha256':sha((R/s['normative_source']['logical_path']).read_bytes()),'content_hex':(R/s['normative_source']['logical_path']).read_bytes().hex()} for s in json.loads(sb)['specifications'] if s['specification_id'] in ids],'rules':[codec.rule_identity(s) for s in json.loads(rb)['rules'] if s['rule_id'] in rids]}
 js('spec/conformance/g01-document-occurrence-staging/compiler-current.json',identity)
 portable=[]
 from portable_proof_python_oracle import canonical_bytes
 def pc(slug,p,stage,family):
  file='portable/'+slug+'.ace1';write('spec/conformance/g01-document-occurrence-staging/'+file,canonical_bytes(codec.bundle_and_context(p)));portable.append(dict(case_id=PREFIX+'portable-'+slug,file=file,stage=stage,family=family,decoder_policy='bounded_16m_scalar'))
 pc('current-denied',profile,'D4','UnsupportedSemantics')
 for version in range(1,6):pc('old-label-new-content-'+str(version),{**profile,'version':'agentflow.root-export-checker@'+str(version)},'D4','InvalidIdentity')
 oldi=read('spec/conformance/g01-raw-validation-staging/compiler-current.json')
 oldp=dict(specification_bytes=bytes.fromhex(oldi['registry_hex']),rule_bytes=bytes.fromhex(oldi['rule_registry_hex']),version=OWNER,spec_ids=old['selected_specifications'],rule_ids=old['applied_rules'])
 pc('new-label-old-content',oldp,'D4','InvalidIdentity');pc('unknown',{**profile,'version':'unknown.checker@999'},'D4','UnsupportedSemantics')
 portable.append(dict(case_id=PREFIX+'default-resource-limit',file='portable/current-denied.ace1',stage='D0',family='ResourceLimit',decoder_policy='unchanged_default'))
 from document_occurrence_admission_cases import cases as admission_cases
 admission=admission_cases();js('spec/conformance/g01-document-occurrence-staging/admission-cases.json',{'cases':admission,'classification':'Metadata association only; NOT Raw predicate execution'})
 from document_occurrence_admission_cases import dag_cases
 dags=dag_cases();js('spec/conformance/g01-document-occurrence-staging/dag-cases.json',{'cases':dags})
 runtime=[{'case_id':PREFIX+'portable-caller-support-flag','stage':'D0','family':'InvalidCanonicalEncoding'},{'case_id':PREFIX+'portable-noncanonical-current-tuple','stage':'D1','family':'InvalidCanonicalEncoding'}]
 manifest={'kind':'Registration/Metadata Admission ONLY; NOT Producer Capture','analysis':OWNER,'model':MODEL,'shape':SHAPE,'obligation':OBLIGATION,'case_ids':[c['case_id'] for c in cases+RULES+portable+admission+runtime+dags],'shape_cases':len(cases),'admission_cases':len(admission),'dag_cases':len(dags),'rule_cases':len(RULES),'portable_cases':portable,'runtime_portable_cases':runtime,'coverage':coverage,'selected_specifications':ids,'applied_rules':rids,'capture_case_ids':0,'producer':'NotImplemented','scope':'NotImplemented','normalizer':'NotImplemented','positive_capability':'NotAuthorized'}
 js('spec/conformance/g01-document-occurrence-staging/manifest.json',manifest)
 types_by_draft={
  'DO-01':['RawDocument','DocumentOwnerAnchor','DocumentOccurrenceKey','DocumentOccurrenceBinding','ObservationSite'],
  'DO-02':['SourcePackage','SourcePackageMetadata','DocumentOccurrenceBinding','DocumentOccurrenceMetadataBinding'],
  'DO-03':['DocumentOwnerContainer','MetadataDocumentOwnerContainer','DeclarationDocumentEntry','ImportDocumentEntry','WorkflowDocumentEntry'],
  'DO-04':['DocumentSourceKey','DocumentSourceDomain','DocumentOwnerAnchor','DocumentSourceAssurance'],
  'DO-05':['SourcePackage','DocumentOwnerRecord','MetadataDocumentOwnerRecord'],
  'DO-06':['DocumentOccurrenceKey','DocumentOccurrenceBinding','DocumentOccurrenceAdmissionFailure'],
  'DO-07':['RawInputOrigin','RawDocumentMember','RawPortOccurrence','DeclaredStep'],
  'DO-08':['RawDocument','RawPhysicalAssociation','DocumentOccurrenceBinding'],
  'DO-09':['RawInputOrigin','RawParsedInput','RawValidationSubject','Source','RawValidationOrigin','Origin','FactKey','RawPortCheckBasis','RawPortRejectionWitness','EvidenceRef'],
  'DO-10':['SourcePackage','CanonicalValueDag','DocumentOccurrenceBinding'],
  'DO-11':['Origin','FactKey','Fact','ObservationSite','ConsumerEdge'],
  'DO-12':['ContractProofContextContents','RawContextPartition','RawValidationSubject','RawPortOccurrence'],
  'DO-13':['DocumentSourceMigrationState','DocumentOccurrenceBinding','Scope'],
  'DO-14':['DocumentOccurrenceAdmissionFailure','DocumentSourceAssurance','RawValidationFragment']}
 js('spec/conformance/g01-document-occurrence-staging/registration-map.json',{'clauses':[{'draft':r['draft'],'rule_id':r['rule_id'],'source':SOURCES[r['source']][2],'specification_id':SOURCES[r['source']][0],'source_anchor':r['anchor'],'model_owner':MODEL,'descriptor_names':types_by_draft[r['draft']],'conformance_case_ids':[r['case_id']]+[c['case_id'] for c in cases if c['type'] in types_by_draft[r['draft']]],'compiler_semantics':OWNER,'full_producer_conformance':False} for r in RULES]})
 conversions={12:'missing-package',13:'missing-namespace',14:'missing-domain_key',15:'missing-owner_key',16:'missing-revision_key',17:'missing-slot',18:'owner-domain-mismatch',19:'unknown-owner',20:'undeclared-owner',21:'out-of-range',22:'joint-conflict',23:'wrong-content',24:'duplicate-owner',27:'native-declaration_bundle',28:'native-supplied',29:'physical-not-supplied',30:'physical-cross-occurrence',31:'physical-bytes-conflict',33:'context-split',34:'context-digest-is-not-owner',41:'package-output-backedge',44:'missing-slot',45:'native-declaration_bundle',48:'random-uuid-only'}
 designs=read('docs/implementation/phase-04c1b4c4r3c2g1-independent-vectors.json')['vectors']
 js('spec/conformance/g01-document-occurrence-staging/design-conversion-plan.json',{'kind':'Exact metadata-only conversion plan; execution state requires separate real receipts','vectors':[{'design_case_id':v['case_id'],'original_actual_status':v['actual_status'],'registered_case_ids':([PREFIX+'admission-'+conversions[i]] if i in conversions else []),'scope':'Metadata admission ONLY' if i in conversions else 'Not Executed: full source/fact/observation/port/scope or migration implementation not supplied by this phase','new_expected_manual':v['expected_manual'],'formal_rules':[r['rule_id'] for r in RULES]} for i,v in enumerate(designs,1)]})
 print('OCCURRENCE REGISTRATION',len(cases),'shape',len(coverage),'field/variant sites',len(t),'descriptors')
def main():
 m=model()
 if '--model-only' not in sys.argv:corpus(m)
if __name__=='__main__':main()
