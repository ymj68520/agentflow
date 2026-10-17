"""Independent C2R1 inputs and raw port semantics; never derives Expected from Rust.

This is a local borrowed-input projection, NOT a Scope encoder or normalizer.
"""
import copy,functools,importlib.util,json,re,sys
from pathlib import Path
R=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('raw_input_cc',R/'tools/oracle/native_guard_capture.py')
NG=importlib.util.module_from_spec(spec);spec.loader.exec_module(NG)
MODEL=json.loads(re.search(r'```json\s*(.*?)\s*```',(R/'spec/tables/proof-report-model-v5.md').read_text(encoding='utf8'),re.S)[1])
T=MODEL['types']
def snake(s):return re.sub(r'(?<!^)(?=[A-Z])','_',s).lower()
def lift(d,v):
 if 'ref' in d:
  n=d['ref']
  if n=='Raw($String)':return list(v.encode('utf8'))
  return lift(T[n],v)
 k=d['kind']
 if k=='record':
  fields=d['fields']
  if len(fields)==1 and fields[0][0]=='0':return {'0':lift(fields[0][1],v)}
  return {n:lift(x,v['type' if n=='type_ref' else n]) for n,x in fields}
 if k=='enum':
  tag,payload=(v,None) if isinstance(v,str) else next(iter(v.items()))
  name,child=next((n,x) for n,x in d['variants'] if snake(n)==tag)
  if child.get('kind')=='unit':return name if all(x.get('kind')=='unit' for _,x in d['variants']) else {name:None}
  return {name:lift(child,payload)}
 if k in ('sequence','set'):return [lift(d['item'],x) for x in v]
 if k=='option':return {'none':None} if v is None else {'some':lift(d['item'],v)}
 return v
def key(s):return list(s.encode('utf8'))
def package(document,kind='declaration_bundle'):
 domain={'namespace':key('C2R1'),'domain_key':key('caller-supplied')}
 anchor={'domain':domain,'owner_key':key('owner-a'),'revision_key':key('revision-1')}
 names={'declaration_bundle':('bundle_name','declarations','declaration_name'),'import_manifest':('manifest_name','imports','import_name'),'workflow_source_collection':('collection_name','sources','source_name')}
 title,entries,label=names[kind]
 return {'domain':domain,'owners':[{'anchor':anchor,'container':{kind:{title:key('actual'),entries:[{label:key('document-a'),'document':document},{label:key('document-b'),'document':copy.deepcopy(document)}]}}}]}
def document(p,owner=0,slot=0):
 kind,body=next(iter(p['owners'][owner]['container'].items()))
 field={'declaration_bundle':'declarations','import_manifest':'imports','workflow_source_collection':'sources'}[kind]
 return body[field][slot]['document']
def vectors():
 rows=[]
 shapes=[('empty',[],[]),('input-zero',[0,1],[2]),('output-zero',[1],[0,2]),('input-duplicate',[3,1,3],[4]),('output-duplicate',[1],[2,2]),('namespaces',[1],[1]),('numeric',[10,2,1],[4294967295,10,2]),('stable',[7,3,7,7],[2]),('sentinel',[1,2],[]),('output-only',[],[2,1]),('input-only',[2,1],[]),('zero-duplicate',[0,0],[1]),('reversed-duplicate',[8,4,4],[2]),('max',[4294967295],[]),('all-positive',[4,2,3],[3,1]),('output-sentinel',[],[4294967295])]
 for kind in ['declaration_bundle','import_manifest','workflow_source_collection']:
  for name,ins,outs in shapes:
   wire=NG.rust_input(NG.CC.empty_contract())
   ref=NG.reference(NG.schema('int64'))
   wire['inputs']=[{'port_no':n,'type':copy.deepcopy(ref)} for n in ins]
   wire['outputs']=[{'port_no':n,'type':copy.deepcopy(ref)} for n in outs]
   raw=lift(T['RawCandidateContract'],wire)
   physical=(kind=='import_manifest')
   doc={'physical_json_utf8':{'bytes':list((' \n'+json.dumps([raw],ensure_ascii=True)+'\t').encode())}} if physical else {'declared_native_document':{'declarations':[raw]}}
   rows.append({'case_id':'c2r1/'+kind+'/'+name,'package':package(doc,kind),'owner':0,'slot':0,'path_entry':0,'wire_candidate':wire,'candidate':raw,'physical':'matching' if physical else 'not_supplied','operation':'capture'})
 base=copy.deepcopy(rows[8])
 for name,change in [('bad-identity',lambda v:v['wire_candidate']['component_identity'].update(name='BAD')),('schema-stop',lambda v:v['wire_candidate']['inputs'][0]['type']['schema_digest'].update(hex='0'*64))]:
  v=copy.deepcopy(base);v['case_id']='c2r1/'+name;change(v);v['candidate']=lift(T['RawCandidateContract'],v['wire_candidate']);v['package']=package({'declared_native_document':{'declarations':[v['candidate']]}});rows.append(v)
 for name,op in [('missing-package','missing_package'),('missing-owner','missing_owner'),('missing-slot','missing_slot'),('wrong-slot','wrong_slot'),('wrong-owner','wrong_owner'),('wrong-revision','wrong_revision'),('clone-document','clone_document'),('wrong-candidate','wrong_candidate'),('wrong-entry','wrong_entry'),('different-context-object','other_context'),('native-supplied','native_supplied'),('physical-not-supplied','physical_not_supplied'),('malformed-key','malformed_key'),('duplicate-owner','duplicate_owner'),('joint-conflict','joint_conflict'),('binding-conflict','binding_conflict')]:
  v=copy.deepcopy(base if op!='physical_not_supplied' else rows[16]);v['case_id']='c2r1/source/'+name;v['operation']=op;rows.append(v)
 for name,op in [('distinct-owner','distinct_owner'),('distinct-slot','distinct_slot'),('same-occurrence','same_occurrence')]:
  v=copy.deepcopy(base);v['case_id']='c2r1/reuse/'+name;v['operation']=op
  if op=='distinct_owner':
   other=copy.deepcopy(v['package']['owners'][0]);other['anchor']['owner_key']=key('owner-b');v['package']['owners'].append(other)
  rows.append(v)
 for primitive,value in [('unit',None),('null',None),('bool',True),('int64',-9223372036854775808),('decimal','1.25'),('string','e\u0301'),('bytes','_'),('timestamp',-9223372036854775808),('duration',9223372036854775807)]:
  wire=NG.rust_input(NG.base());wire['inputs'][0]['port_no']=0
  wire['effects']['row']['closed'][1]['guard']['input_equals']['expected']={primitive:value}
  raw=lift(T['RawCandidateContract'],wire)
  rows.append({'case_id':'c2r1/complete-thirteen-fields/'+primitive,'package':package({'declared_native_document':{'declarations':[raw]}}),'owner':0,'slot':0,'path_entry':0,'wire_candidate':wire,'candidate':raw,'physical':'not_supplied','operation':'capture'})
 for name,op in [('duplicate-json-key','duplicate_json'),('invalid-utf8','invalid_utf8'),('unknown-selected-field','unknown_selected_field'),('wrong-u32-width','wrong_u32_width'),('physical-path-mismatch','physical_path_mismatch'),('unknown-package-field','unknown_package_field'),('foreign-domain','foreign_domain'),('unsupported-container','unsupported_container')]:
  v=copy.deepcopy(rows[17]);v['case_id']='c2r1/source/'+name;v['operation']=op
  doc=document(v['package']);b=doc['physical_json_utf8']['bytes']
  if op=='duplicate_json':doc['physical_json_utf8']['bytes']=list(b'[{"component_identity":null,"component_identity":null}]')
  if op=='invalid_utf8':doc['physical_json_utf8']['bytes']=[255]
  if op in ('unknown_selected_field','wrong_u32_width'):
   bad=copy.deepcopy(v['candidate'])
   if op=='unknown_selected_field':bad['not_registered']=True
   else:bad['inputs'][0]['port_no']=4294967296
   doc['physical_json_utf8']['bytes']=list(json.dumps([bad]).encode())
  if op=='unknown_package_field':v['package']['not_registered']=True
  if op=='foreign_domain':
   v['package']['owners'][0]['anchor']['domain']=copy.deepcopy(v['package']['domain'])
   v['package']['owners'][0]['anchor']['domain']['domain_key']=key('foreign')
  if op=='unsupported_container':v['package']['owners'][0]['container']={'durable_handle':{}}
  rows.append(v)
 v=copy.deepcopy(rows[1]);v['case_id']='c2r1/declared-namespace-is-not-an-authentication-whitelist'
 domain={'namespace':key('external-name-is-only-a-declaration'),'domain_key':key('foreign')}
 v['package']['domain']=copy.deepcopy(domain);v['package']['owners'][0]['anchor']['domain']=copy.deepcopy(domain);rows.append(v)
 assert len(rows)==87 and len({v['case_id'] for v in rows})==87
 return rows
def port_events(v):
 w=v['wire_candidate'];events=[];stop=None
 if w['component_identity']['name']=='BAD':return [],{'component_identity':None}
 for direction,field in [('input','inputs'),('output','outputs')]:
  predecessor=None
  for sorted_i,(original,p) in enumerate(sorted(enumerate(w[field]),key=lambda x:x[1]['port_no'])):
   occ={'direction':direction,'original_ordinal':original,'sorted_ordinal':sorted_i,'port_no':p['port_no']}
   n=p['port_no'];condition='zero' if n==0 else 'duplicate' if predecessor and predecessor['port_no']==n else 'positive_fresh'
   witness=None if condition=='positive_fresh' else {'zero':{'occurrence':occ,'raw_member_value':0,'minimum':1}} if condition=='zero' else {'duplicate':{'occurrence':occ,'previous_original_ordinal':predecessor['original_ordinal'],'previous_port_no':predecessor['port_no']}}
   events.append({'occurrence':occ,'predecessor':predecessor,'condition':condition,'status':'proven' if condition=='positive_fresh' else 'violation','witness':witness})
   if condition!='positive_fresh':return events,{'port_rejection':occ}
   if p['type']!=NG.reference(NG.schema('int64')):return events,{'schema_resolution':occ}
   predecessor={'original_ordinal':original,'port_no':n}
 return events,stop
SOURCE_FAILURES={
 'missing_package':'MissingDocumentOccurrenceBinding','missing_owner':'MissingDocumentOccurrenceBinding','missing_slot':'MissingDocumentOccurrenceBinding',
 'wrong_slot':'InvalidOccurrenceAssociation','wrong_owner':'InvalidOccurrenceAssociation','wrong_revision':'InvalidOccurrenceAssociation','clone_document':'InvalidOccurrenceAssociation',
 'wrong_candidate':'InvalidOccurrenceAssociation','wrong_entry':'InvalidOccurrenceAssociation','other_context':'InvalidContextBinding',
 'native_supplied':'UnsupportedPhysicalAssociation','physical_not_supplied':'UnsupportedPhysicalAssociation','malformed_key':'MissingDocumentOccurrenceBinding',
 'duplicate_owner':'MalformedSourcePackage','joint_conflict':'OwnerBindingConflict','binding_conflict':'BindingConflict',
 'duplicate_json':'InvalidPhysicalInput','invalid_utf8':'InvalidPhysicalInput','unknown_selected_field':'InvalidOccurrenceAssociation','wrong_u32_width':'InvalidOccurrenceAssociation',
 'physical_path_mismatch':'InvalidOccurrenceAssociation','unknown_package_field':'UnsupportedModel','foreign_domain':'InvalidOccurrenceAssociation','unsupported_container':'UnsupportedOwnerVariant'}
def rule():
 r=next(r for r in json.loads((R/'spec/registry/rules.json').read_bytes())['rules'] if r['rule_id']=='AF-CONTRACT-0004')
 return {n:r[n] for n in ['rule_id','normative_source','source_anchor','short_statement']}
@functools.lru_cache(maxsize=1)
def schema_hex():
 s=next(s for s in NG.CC.afidl.compile_file(R/'spec/idl/component-contract.afidl') if s['name']=='ComponentContract')
 return NG.CC.afidl.canonical_ace1_bytes(s).hex()
def declared_path(direction,original):
 def field(owner,name):return {'field':{'owner':owner,'field':name}}
 return [field('RawValidationSubject','input'),field('RawParsedInput','candidate'),field('Raw(af-contract::ComponentContract)',direction+'s'),{'entry':{'index':original}},field('Raw(af-contract::ComponentInputPort)' if direction=='input' else 'Raw(af-contract::ComponentOutputPort)','port_no')]
def expected(v):
 op=v['operation']
 if op=='foreign_domain':assert v['package']['owners'][0]['anchor']['domain']!=v['package']['domain'],'ineffective foreign-domain mutation'
 if op in SOURCE_FAILURES:return {'case_id':v['case_id'],'failure':SOURCE_FAILURES[op]}
 events,stop=port_events(v)
 for e in events:
  o=e['occurrence'];e.update(role='supporting_fact',kind='prerequisite',stage='contract_validation',substage='canonical_validation/io_port_check',evidence_class='static_declaration',predicate=['raw_contract_validation','positive_unique_io_ports'],evaluated=True,
      selected_error=None if e['condition']=='positive_fresh' else 'invalid_or_duplicate_port',rule=rule(),source_ordinal=o['original_ordinal'],mapping_ordinal=None,declared_path=declared_path(o['direction'],o['original_ordinal']),
      target_constraint='positive_unique_io_ports',target_module='agentflow.component.contract@1',target_name='ComponentContract',target_definition_ace1_hex=schema_hex(),target_direction=o['direction'])
 observations=[1]*len(events)
 if stop and 'port_rejection'in stop:observations[-1]=2
 reuse=op in ['distinct_owner','distinct_slot','same_occurrence']
 return {'case_id':v['case_id'],'failure':None,'events':events,'stop':stop,'canonical_validated':stop is None,'root_export':False,'complete_scope':False,
   'source_package':v['package'],'owner':v['package']['owners'][0]['anchor'],'slot':0,'document':document(v['package']),'candidate':v['candidate'],
   'same_typed_thirteen_fields':True,'context_roles':[['analysis','canonical_validation']],'observation_counts':observations,
   'same_subject':op=='same_occurrence' if reuse else None,'combined_count':len(events)*(1 if op=='same_occurrence' else 2) if reuse else None}
def paths(v,path=()):
 out=set()
 if isinstance(v,dict):
  for k,x in v.items():out.add(path+(k,));out.update(paths(x,path+(k,)))
 elif isinstance(v,list):
  for i,x in enumerate(v):out.update(paths(x,path+(i,)))
 return out
def event_ids(rows):return {(r['case_id'],e['occurrence']['direction'],e['occurrence']['original_ordinal']) for r in rows for e in r.get('events',[])}
def check_projection(reference,want,got):
 # Reference is regenerated from INPUTS, never from provided Expected/Actual.
 # Thus colluding deletions in both compared views remain missing requirements.
 assert len(want)==len(reference)==len(got),'missing/extra cases'
 assert len({r['case_id'] for r in want})==len(want) and len({r['case_id'] for r in got})==len(got),'duplicate case'
 assert event_ids(reference)==event_ids(want)==event_ids(got),'required event omission/addition'
 for r,w,a in zip(reference,want,got):
  assert r['case_id']==w['case_id']==a['case_id'],'case order/identity'
  assert paths(r)==paths(w)==paths(a),'required local field omission/addition'
  assert w==r,'independent Expected corrupted'
  assert a==w,'semantic mismatch'
def mutation_gate(reference,actual):
 index={r['case_id']:i for i,r in enumerate(reference)};i=index['c2r1/declaration_bundle/input-zero'];j=index['c2r1/declaration_bundle/input-duplicate']
 cases=[]
 def test(name,change,both=False,where=i):
  w=copy.deepcopy(reference);a=copy.deepcopy(actual);change(a[where])
  if both:change(w[where])
  try:check_projection(reference,w,a)
  except AssertionError:cases.append({'mutation_id':'c2r1-mutation/'+name,'rejected':True,'both_sides':both});return
  raise AssertionError('accepted mutation '+name)
 fields=[('direction',lambda r:r['events'][0]['occurrence'].update(direction='output')),
  ('original',lambda r:r['events'][0]['occurrence'].update(original_ordinal=9)),('sorted',lambda r:r['events'][0]['occurrence'].update(sorted_ordinal=9)),
  ('number',lambda r:r['events'][0]['occurrence'].update(port_no=1)),('sentinel',lambda r:r['events'][0].update(predecessor={'original_ordinal':0,'port_no':0})),
  ('condition',lambda r:r['events'][0].update(condition='positive_fresh')),('false-proven',lambda r:r['events'][0].update(status='proven')),
  ('missing-witness',lambda r:r['events'][0].update(witness=None)),('zero-minimum',lambda r:r['events'][0]['witness']['zero'].update(minimum=0)),
  ('stage',lambda r:r['events'][0].update(stage='D10')),('role',lambda r:r['events'][0].update(role='mandatory_preflight')),
  ('rule-content',lambda r:r['events'][0]['rule'].update(short_statement='altered')),('selected-error',lambda r:r['events'][0].update(selected_error=None)),
  ('declared-path',lambda r:r['events'][0]['declared_path'][2]['field'].update(field='outputs')),('source-ordinal',lambda r:r['events'][0].update(source_ordinal=1)),
  ('target-schema',lambda r:r['events'][0].update(target_definition_ace1_hex='00')),('target-direction',lambda r:r['events'][0].update(target_direction='output')),
  ('owner-revision',lambda r:r['owner'].update(revision_key=[122])),('owner-slot',lambda r:r.update(slot=1)),
  ('full-document',lambda r:r['document']['declared_native_document']['declarations'].append(copy.deepcopy(r['candidate']))),
  ('candidate-body',lambda r:r['candidate']['component_identity'].update(name=[122])),('package-domain',lambda r:r['source_package']['domain'].update(namespace=[122])),
  ('context-role',lambda r:r.update(context_roles=[['analysis'],['canonical_validation']])),('scope-capability',lambda r:r.update(complete_scope=True)),('root-capability',lambda r:r.update(root_export=True)),
  ('checked-stage',lambda r:r.update(canonical_validated=True)),('duplicate-observation',lambda r:r.update(observation_counts=[3])),('extra-event',lambda r:r['events'].append(copy.deepcopy(r['events'][0]))),
  ('missing-event',lambda r:r['events'].pop()),('both-missing-event',lambda r:r['events'].pop()),
  ('missing-field',lambda r:r['events'][0].pop('status')),('both-missing-field',lambda r:r['events'][0].pop('status')),('extra-field',lambda r:r['events'][0].update(trusted=True))]
 for name,fn in fields:test(name,fn,both=name.startswith('both-'))
 test('duplicate-previous-original',lambda r:r['events'][-1]['witness']['duplicate'].update(previous_original_ordinal=999),where=j)
 test('duplicate-previous-number',lambda r:r['events'][-1]['witness']['duplicate'].update(previous_port_no=999),where=j)
 return cases
def report(actual):
 source=vectors();reference=[expected(v) for v in source];check_projection(reference,copy.deepcopy(reference),actual)
 mutations=mutation_gate(reference,actual)
 # This release deliberately has an executable DENIAL for the missing formal
 # Fact layer. A local event comparator does not silently become full G01.
 fields=[n for n,_ in T['Fact']['fields']]
 mapped={'role','evaluation','status','raw_validation_basis'}
 gaps=[n for n in fields if n not in mapped]
 assert len(fields)==14 and gaps
 events=len(event_ids(reference));local_fields=sum(len(paths(r)) for r in reference)
 return {'outcome':'D — CONFORMANCE INCOMPLETE','local_projection_conformance':'PASS','full_raw_seed_conformance':'INCOMPLETE',
  'input_case_ids':len(source),'accepted_source_case_ids':sum(r['failure'] is None for r in reference),'actual_port_capture_case_ids':sum(bool(r.get('events')) for r in reference),'source_negative_case_ids':sum(r['failure'] is not None for r in reference),
  'required_event_coverage':{'applicable':events,'executed':events,'captured':events,'independent_expected':events,'compared':events,'missing_expected':0,'missing_actual':0,'unexpected_extra':0},
  'local_projection_fields':{'required_instances':local_fields,'expected':local_fields,'actual':local_fields,'compared':local_fields,'missing_expected':0,'missing_actual':0,'unexpected_extra':0,'mismatch':0},
  'registered_fact_field_contract':{'model':MODEL['model_id'],'required_fields':fields,'locally_mapped_fields':sorted(mapped),'missing_formal_fields':gaps,'required_instances':events*len(fields),'unmaterialized_instances':events*len(gaps),'complete':False},
  'additional_required_field_gaps':['CompleteCompiler full-field independent projection','ContractProofContextContents nine complete member values independent projection','Raw witness subject/full TypedContradiction materialization','Exact typed evidence set and primary minimum','Formal ObservationSite paths and consumer/prerequisite references','DeclaredStep full ModelType owner/name projection'],
  'mutations':mutations,'comparator_mutations_rejected':len(mutations),'both_sides_event_deletion_rejected':True,'both_sides_field_deletion_rejected':True,
  'remaining_raw_capture':'INCOMPLETE','positive_root_export':'NotAuthorized','t4':'NOT ELIGIBLE'}
if __name__=='__main__':
 if sys.argv[1:]==['--vectors']:print(json.dumps(vectors(),ensure_ascii=True))
 elif sys.argv[1:] in ([],['--report']):
  result=report(json.load(sys.stdin));print(json.dumps(result,ensure_ascii=True,indent=2))
  if not sys.argv[1:]:sys.exit(2)
 else:raise ValueError(sys.argv[1:])
