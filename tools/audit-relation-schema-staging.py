"""Independent current registration/freeze/receipt audit. NOT a Raw Fact oracle."""
import hashlib,importlib.util,json,re,subprocess,sys
from pathlib import Path
from relation_schema_registration_data import BASELINE,OWNER,MODEL,SHAPE,OBLIGATION,SOURCES,RULES,SLOTS,PREFIX
R=Path(__file__).resolve().parent.parent
D=R/'spec/conformance/g01-relation-schema-staging'
P='phase-04c1b4c4r3c2r3g2-'
ALLOWED={'crates/af-computation/Cargo.toml','crates/af-computation/src/lib.rs','crates/af-computation/src/registered_model.rs','crates/af-computation/src/root_export.rs','crates/af-portable/Cargo.toml','crates/af-portable/src/support.rs','tools/gen-registry-instances.py','tools/gen-document-occurrence-staging.py','spec/registry/specifications.json','spec/registry/rules.json'}
def read(p):return json.loads(p.read_bytes())
def model(p):return json.loads(re.search(r'```json\s*(.*?)\s*```',p.read_text(encoding='utf8'),re.S)[1])
def refs(x):
 if isinstance(x,dict):
  if 'ref' in x:yield x['ref']
  for v in x.values():yield from refs(v)
 elif isinstance(x,list):
  for v in x:yield from refs(v)
def loadmod(name,path):
 s=importlib.util.spec_from_file_location(name,R/path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def frozen():
 baseline=R/'target'/(P+'start-raw-map.json')
 if baseline.exists():original=read(baseline)
 else:
  files=subprocess.check_output(['git','ls-tree','-r','--name-only',BASELINE],cwd=R,text=True).splitlines()
  original={p:hashlib.sha256(subprocess.check_output(['git','show',BASELINE+':'+p],cwd=R)).hexdigest() for p in files}
 changed=[p for p,h in original.items() if p not in ALLOWED and hashlib.sha256((R/p).read_bytes()).hexdigest()!=h]
 assert not changed,changed
 return dict(original_paths=len(original),allowed_existing=sorted(ALLOWED),unauthorized_changes=changed,all_other_original_raw_bytes_equal=True)
def registration():
 old=model(R/'spec/tables/proof-report-model-v5.md');m=model(R/SOURCES['RM'][2]);t=m['types']
 assert m['inherited_registered_model']==old
 assert m['historical_selected_imports']==old['historical_selected_imports'] and len(m['historical_selected_imports'])==97
 assert m['historical_import_sources']==old['historical_import_sources']
 assert m['analysis_owner']==OWNER and m['model_id']==MODEL and m['model_edition']==SHAPE and m['obligation_edition']==OBLIGATION
 assert len(old['types'])==704 and len(list(refs(old['types'])))==2064
 for name,d in old['types'].items():
  assert m['type_ranks'][name]==old['type_ranks'][name],name
  if name!='RelationSnapshotContent':assert t[name]==d,name
 assert len(set(m['type_ranks'].values()))==len(t)
 edges=list(refs(t));assert all(n in t for n in edges)
 assert list(dict(t['ContractProofContextContents']['fields']))==['compiled_modules','type_schemas','effect_catalogs','resource_types','flow_catalogs','usage_meters','recovery_operations','execution_universes','relation_snapshots']
 forbidden={'Fact','FactKey','Scope','CurrentContextRelationSchemaValidation','RelationSubjectAdmission','RelationValidationResult','RelationCurrentValidationMetadata'}
 seen=set();pending=['ContractProofContextContents','NativeRelationSnapshotSource']
 while pending:
  n=pending.pop()
  if n in seen:continue
  assert n not in forbidden,('output backedge',n)
  seen.add(n);pending.extend(refs(t[n]))
 for file,key,count in [('specifications','specification_id',62),('rules','rule_id',493)]:
  before=read(D/f'analysis6-{file}.json')[file];after=read(R/f'spec/registry/{file}.json')[file]
  assert len(before)==count
  assert all(next((v for v in after if v[key]==r[key]),None)==r for r in before)
 specs=read(R/'spec/registry/specifications.json')['specifications'];rules=read(R/'spec/registry/rules.json')['rules']
 assert len(specs)==66 and len(rules)==509
 for r in specs:
  raw=(R/r['normative_source']['logical_path']).read_bytes()
  assert hashlib.sha256(raw).hexdigest()==r['normative_source']['content_digest']['hex']
 assert len([r for r in rules if r['conformance_refs']])==508
 assert read(R/'spec/registry/coverage-summary.json')['missing_coverage']==[]
 assert sum(r['status']=='active' for r in read(R/'spec/registry/abi-numbers.json')['assignments'])==775
 assert len(list((R/'spec/idl').rglob('*.afidl')))==17
 assert len(read(R/'spec/tables/hash-domains.json')['domains'])==14
 return dict(tuple=dict(analysis=OWNER,model=MODEL,report=SHAPE,obligation=OBLIGATION),specifications=66,new_specifications=4,rules=509,new_rules=16,reference_covered=508,intentional_exception=1,missing=0,old_spec_records_equal=62,old_rule_records_equal=493,old_descriptor_identities=704,old_references=2064,old_ranks_preserved=True,historical_imports=97,descriptors=len(t),references=len(edges),added_descriptors=len(t)-704,successor_reference_delta=len(edges)-2064,input_closure_types=len(seen),input_output_separation=True,abi=775,afidl=17,hash_domains=14)
def field_coverage(m):
 manifest=read(D/'manifest.json');shape=read(D/'shape-cases.json');ids=manifest['case_ids']
 assert len(ids)==len(set(ids)) and len(ids)>=72
 cases={c['case_id']:c for c in shape['cases']};sites=shape['coverage']
 required=[]
 for name in m['relation_registration']['authored_types']:
  d=m['types'][name]
  if d.get('kind')=='record':required.extend((name,None,f) for f,_ in d['fields'])
  if d.get('kind')=='enum':
   for tag,payload in d['variants']:
    required.append((name,tag,None))
    if payload.get('kind')=='record':required.extend((name,tag,f) for f,_ in payload['fields'])
 got={(c['type'],c.get('variant'),c.get('field')) for c in sites}
 assert set(required)==got
 assert all(s['case_id'] in cases for s in sites)
 assert set(m['relation_registration']['authored_types'])=={c['type'] for c in shape['cases'] if '/shape-' in c['case_id']}
 return dict(stable_case_ids=len(ids),shape_cases=len(cases),field_variant_sites=len(sites),required_sites=len(required),missing_sites=[],real_slot_admission_cases=manifest['admission_cases'],aggregate_context_cases=manifest['context_cases'],dag_cases=manifest['dag_cases'],rule_cases=manifest['rule_cases'],portable_cases=len(manifest['portable_cases']),native_equality_cases=3,portable_runtime_cases=2,actual_raw_producer_capture_cases=0)
def receipts():
 oracle=loadmod('independent_relation_admission','tools/oracle/relation_schema_admission.py')
 actual=read(R/'target'/(P+'admission-actual.json'));result=oracle.report(actual)
 contexts=read(R/'target'/(P+'context-actual.json'))
 oracle.compare([dict(case_id=c['case_id'],actual=oracle.context_expected(c)) for c in oracle.context_cases()],contexts)
 shape=read(D/'shape-cases.json');actualshape=read(R/'target'/(P+'shape-actual.json'))
 oracle.compare([dict(case_id=c['case_id'],actual_failure=c['expected_failure']) for c in shape['cases']],actualshape)
 dags=read(R/'target'/(P+'dag-actual.json'))
 oracle.compare([dict(case_id=c['case_id'],actual_failure=c['expected_failure']) for c in oracle.dag_cases()],dags)
 eq=read(R/'target'/(P+'equality-actual.json'));wanted=[]
 for slot,root,family in SLOTS:
  _,a=oracle.compiled(slot,'A');_,b=oracle.compiled(slot,'B');key=family+'/'+root
  digest=lambda ts:dict(algorithm='sha256',domain='type-schema',hex=oracle.AF.type_schema_digest(ts[key])[1])
  wanted.append(dict(case_id=PREFIX+'native-equality-'+slot,native_equal=True,schema_a=digest(a),schema_b=digest(b),ambiguous_native_context_rejected=True,construction_history_recovered=False,positive_root_export=False))
 oracle.compare(wanted,eq)
 ruleactual=read(R/'target'/(P+'rule-actual.json'))
 oracle.compare([dict(case_id=r['case_id'],rule_id=r['rule_id'],actual_registered=True,positive_capability=False) for r in RULES],ruleactual)
 manifest=read(D/'manifest.json');portable=read(R/'target'/(P+'portable-actual.json'))
 oracle.compare([dict(case_id=c['case_id'],stage=c['stage'],family=c['family'],positive_root_export=False) for c in manifest['portable_cases']],portable)
 runtime=read(R/'target'/(P+'portable-runtime-actual.json'))
 oracle.compare([dict(case_id=PREFIX+'portable-caller-support-flag',stage='D0',family='InvalidCanonicalEncoding'),dict(case_id=PREFIX+'portable-noncanonical',stage='D1',family='InvalidCanonicalEncoding')],runtime)
 executed={c['case_id'] for c in actual+contexts+actualshape+dags+eq+ruleactual+portable+runtime}
 assert executed==set(manifest['case_ids']),('unexecuted registration cases',set(manifest['case_ids'])-executed)
 return dict(independent=result,executed_case_ids=len(executed),actual_shape=len(actualshape),actual_admission=len(actual),actual_context=len(contexts),actual_dag=len(dags),actual_equality=len(eq),actual_rules=len(ruleactual),actual_portable=len(portable)+len(runtime),raw_full_field_oracle=False,raw_producer_complete=False,positive_root_export=False)
def conversion(executed):
 original=read(R/'docs/implementation/phase-04c1b4c4r3c2r3g1-design-vectors.json')['vectors']
 assert len(original)==48 and all(v['ActualStatus']=='Not Executed' and v['actual'] is None for v in original)
 # Only exact metadata scenarios, never a similar name or Full Raw condition.
 exact={13:['admission-effect_resource-valid'],18:['admission-effect_resource-nonempty-under-B'],19:['admission-effect_resource-current-B'],22:['admission-effect_resource-split-context'],25:['admission-effect_resource-valid'],26:['admission-recovery_usage-valid'],27:['admission-flow_execution-valid'],28:['context-all-none'],30:['admission-effect_resource-schema-name'],31:['admission-effect_resource-schema-digest'],32:['admission-effect_resource-catalog-checksum-as-schema'],33:['admission-effect_resource-split-context','admission-effect_resource-false-alias-claim'],35:['context-recovery-missing']}
 subset={1:['native-equality-effect_resource'],2:['native-equality-effect_resource'],3:['native-equality-effect_resource'],4:['admission-effect_resource-fabricated-history'],6:['admission-effect_resource-fabricated-history'],7:['admission-effect_resource-schema-name'],8:['admission-effect_resource-malformed-family'],10:['admission-effect_resource-source-checksum'],12:['native-equality-effect_resource'],14:['admission-effect_resource-missing-module'],15:['admission-effect_resource-ambiguous-boundary'],16:['admission-effect_resource-wrong-expected-family'],17:['admission-effect_resource-revision-mismatch'],20:['admission-effect_resource-valid','admission-effect_resource-current-B'],24:['admission-effect_resource-invalid-current'],29:['admission-effect_resource-schema-nominal-owner'],34:['admission-effect_resource-slot-swap'],36:['context-all-valid'],38:['portable-new-label-old-content'],39:['admission-effect_resource-unknown-validation'],40:['dag-output-backedge'],41:['dag-output-as-input'],42:['admission-effect_resource-missing-module'],43:['admission-effect_resource-invalid-current'],44:['admission-effect_resource-nonempty'],45:['portable-current-denied'],46:['portable-old-label-new-content-6'],47:['portable-current-denied'],48:['portable-current-denied']}
 manifest=set(read(D/'manifest.json')['case_ids']);rows=[]
 for i,v in enumerate(original,1):
  targets=[PREFIX+x for x in exact.get(i,subset.get(i,[]))];assert set(targets)<=manifest
  rows.append(dict(design_vector=v['id'],original_actual_status='Not Executed',registered_case_ids=targets,rules=[r['rule_id'] for r in RULES],expected_source='independent frozen AFIDL/native inputs; never Rust Actual',new_status='Executed — exact metadata conditions' if executed and i in exact else 'Not Executed — full design condition; registered subset only' if targets else 'Not Executed — out of current implementation scope',subset_executed=bool(executed and targets),full_raw_capture_credit=0))
 return dict(original_file_frozen=True,original_vectors=48,exact_metadata_vectors_executed=sum(r['new_status'].startswith('Executed') for r in rows),vectors=rows)
def main():
 m=model(R/SOURCES['RM'][2]);out=dict(registration=registration(),field_coverage=field_coverage(m),freeze=frozen())
 if '--receipts' in sys.argv:out['fresh_execution_receipts']=receipts()
 if '--materialize' in sys.argv:
  (R/'docs/implementation'/(P+'registration-audit.json')).write_text(json.dumps(out,indent=2)+'\n',encoding='utf8')
  (R/'docs/implementation'/(P+'design-conversion-receipt.json')).write_text(json.dumps(conversion('--receipts' in sys.argv),indent=2,ensure_ascii=False)+'\n',encoding='utf8')
  mapping=[dict(accepted_design=r['draft'],rule_id=r['rule_id'],normative_source=SOURCES[r['source']][2],source_anchor=r['anchor'],specification_id=SOURCES[r['source']][0],model_owner=MODEL,authored_descriptors=m['relation_registration']['authored_types'],registration_case_id=r['case_id'],compiler_semantics=OWNER,stage='Registration/Metadata Admission ONLY',full_raw_capture=False) for r in RULES]
  (R/'docs/implementation'/(P+'design-rule-descriptor-map.json')).write_text(json.dumps(mapping,indent=2)+'\n',encoding='utf8')
 print(json.dumps(out,indent=2))
if __name__=='__main__':main()
