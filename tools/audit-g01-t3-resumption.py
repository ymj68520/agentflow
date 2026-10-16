"""Read-only resumption census. Engineering locators are never Fact Origin.

No producer implementation or coverage success is inferred from constructor
counts. Full source/model/import payloads are parsed, not sampled by name.
"""
import hashlib,json,re,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
BASE='a0ef89e71ac31405ba05df619f585b5830aa8c92'
MODEL='spec/tables/proof-report-model-v3.md'
def text(p):return (ROOT/p).read_text(encoding='utf-8')
def sha(b):return hashlib.sha256(b).hexdigest()
def main():
 m=json.loads(re.search(r'```json\s*(.*?)\s*```',text(MODEL),re.S)[1]);types=m['types']
 specs=json.loads(text('spec/registry/specifications.json'))['specifications'];rules=json.loads(text('spec/registry/rules.json'))['rules']
 assert (len(specs),len(rules),len(types))==(54,468,652)
 pending=list(types.values());refs=0
 while pending:
  x=pending.pop()
  if isinstance(x,dict):
   assert '_generic' not in x and '_case' not in x
   if 'ref' in x:assert x['ref'] in types;refs+=1
   pending.extend(x.values())
  elif isinstance(x,list):pending.extend(x)
 assert refs==1941
 for s in specs:assert sha((ROOT/s['normative_source']['logical_path']).read_bytes())==s['normative_source']['content_digest']['hex']
 imports=m['historical_selected_imports'];assert len(imports)==97
 for row in imports:assert types[row['successor_key']]==row['successor_descriptor']
 historical=[]
 for p,s in m['historical_import_sources'].items():
  b=bytes.fromhex(s['content_hex']);assert sha(b)==s['sha256']
  historical.append({'path':p,'historical_sha256':s['sha256'],'historical_complete_bytes':len(b),'live_sha256':sha((ROOT/p).read_bytes()),'live_bytes_equal':(ROOT/p).read_bytes()==b})
 site_rows=[];categories={}
 def add(group,path,pattern,expected=None):
  source=text(path).split('#[cfg(test)]',1)[0];matches=list(re.finditer(pattern,source));categories[group]=len(matches)
  if expected is not None:assert len(matches)==expected,(group,len(matches),expected)
  for i,x in enumerate(matches,1):
   before=source[:x.start()];fns=list(re.finditer(r'\bfn\s+(\w+)\s*(?:<[^{};]*>)?\s*\(',before));fn=fns[-1][1] if fns else 'module'
   line=before.count('\n')+1
   site_rows.append({'site_id':f'{group}/{i:03}','rust_source_path':path,'function':fn,'byte_offset':x.start(),'line':line,'actual_producer_token':x[0],'registered_predicate':'REQUIRES_PER_BRANCH_BINDING_REVIEW','actual_formal_stage':'contract_validation' if '/af-contract/' in path else 'D8/D9/D10 per producer','applicable_source_category':group,'can_execute_on_valid_input':'PER_BRANCH_NOT_YET_CERTIFIED','can_execute_on_rejected_input':'PER_BRANCH_NOT_YET_CERTIFIED','original_source_data_available':'AT_THIS_BRANCH_BEFORE_REDUCTION;not inferred from report text','earliest_lossless_hook':{'path':path,'function':fn,'line':line,'engineering_locator_not_origin':True},'required_fact_fields':['predicate','role','requirement','stage','source','target','coordinate','origin','evaluation','verdict','typed_basis','witness_if_V','prerequisites','references','observations'],'actual_capture_status':'NOT_IMPLEMENTED','independent_oracle_status':'NOT_IMPLEMENTED'})
 add('root_direct','crates/af-computation/src/root_export.rs',r'unresolved_obligations\.push\(',6)
 add('inherited_constructor','crates/af-computation/src/control.rs',r'(?<!struct )(?<!impl )InheritedControlObligation\s*\{',3)
 add('control_constructor','crates/af-computation/src/control.rs',r'ControlDimensionProof::new\(',65)
 add('leaf_constructor','crates/af-computation/src/resolution.rs',r'LeafDimensionProof::new\(',27)
 add('native_dimension_constructor','crates/af-contract/src/validate.rs',r'DimensionProof::(?:proven|unproven|violation)\(',17)
 add('native_guard_record','crates/af-contract/src/recovery_guard.rs',r'(?<!struct )(?<!impl )ActionGuardCompatibility\s*\{',11)
 add('standalone_set_call','crates/af-contract/src/compatibility.rs',r'(?<!fn )\bset\(')
 add('standalone_return','crates/af-contract/src/compatibility.rs',r'(?<!struct )(?<!impl )CompatibilityReport\s*\{')
 path='crates/af-contract/src/validate.rs';s=text(path);start=s.index('fn attach_prerequisites');end=s.index('fn declared_effects',start);body=s[start:end]
 arms=list(re.finditer(r'(?:"[a-z-]+"|_)\s*=>\s*vec!',body));lits=list(re.finditer(r'"([^"\n]+)"\.into\(\)',body));assert len(arms)==11 and len(lits)==20
 for group,rows in [('prerequisite_arm',arms),('prerequisite_literal',lits)]:
  categories[group]=len(rows)
  for i,x in enumerate(rows,1):
   at=start+x.start();site_rows.append({'site_id':f'{group}/{i:03}','rust_source_path':path,'function':'attach_prerequisites','byte_offset':at,'line':s[:at].count('\n')+1,'actual_producer_token':x[0],'registered_predicate':'prerequisite;actual earlier validator query must establish status','actual_formal_stage':'contract_validation','applicable_source_category':group,'can_execute_on_valid_input':True,'can_execute_on_rejected_input':True,'original_source_data_available':'description carrier here;actual status/support at original check','earliest_lossless_hook':'actual earlier schema/catalog/relation branch,not this literal attachment','required_fact_fields':['prerequisite_key','applicability','status','supporting_fact','consumer_edges','source','target','coordinate','origin','rule','requirement'],'actual_capture_status':'NOT_IMPLEMENTED','independent_oracle_status':'NOT_IMPLEMENTED'})
 anchors={(x['rust_source_path'],x['byte_offset']) for x in site_rows}
 payload={'kind':'G01ProducerSiteInventory;read-only engineering census,NOT Capture','baseline':BASE,'model_id':m['model_id'],'complete_model_bytes':(ROOT/MODEL).stat().st_size,'model_sha256':sha((ROOT/MODEL).read_bytes()),'registered_counts':[54,468,652,1941,97],'historical_source_bindings':historical,'lexical_categories':categories,'unique_reviewed_lexical_anchors':len(anchors),'unique_semantic_producers':'NOT_CLOSED;constructor/factory/observer categories can overlap','sites':site_rows,'actual_capture_sites':0,'independently_compared_capture_sites':0,'capture_case_ids':0,'missing_required_sites':'NOT_ZERO_GAP_CERTIFIED','complete_scope':'PendingG02','normalizer':'NotImplemented','positive_capability':'NotAuthorized'}
 if sys.argv[1:]==['--check']:
  stored=json.loads((ROOT/'docs/implementation/phase-04c1b4c4r3-producer-site-census.json').read_bytes())
  assert stored['lexical_categories']==categories and stored['unique_reviewed_lexical_anchors']==len(anchors)
  assert len(stored['sites'])==len(site_rows)
  for actual,want in zip(stored['sites'],site_rows):
   for k in ['site_id','rust_source_path','function','byte_offset','line','actual_producer_token']:
    assert actual[k]==want[k],(actual['site_id'],k)
  frozen=json.loads((ROOT/'docs/implementation/phase-04c1b4c4r3-frozen-inputs.json').read_bytes())
  for p,h in frozen['paths'].items():assert sha((ROOT/p).read_bytes())==h,('frozen bytes',p)
  print(json.dumps({'lexical_anchors_verified':len(anchors),'frozen_paths_verified':len(frozen['paths']),
                    'formal_tuple':[54,468,652,1941,97],'full_seed_conformance':False,'outcome':'B'}))
 else:print(json.dumps(payload,ensure_ascii=False,indent=2))
if __name__=='__main__':main()
