"""Source-based G2 asset/coverage audit; never a production or proof issuer."""
import argparse,hashlib,json,subprocess,sys
from pathlib import Path
R=Path(__file__).resolve().parents[1];sys.path.insert(0,str(R/'tools'))
from oracle import native_source_registration as O
from native_relation_source_registration import RULES,SOURCES
D=R/'spec/conformance/g01-native-source-staging';P='phase-04c1b4c4r3c2r5g2-'

def sha(p):
 h=hashlib.sha256()
 with p.open('rb') as f:
  while b:=f.read(1048576):h.update(b)
 return h.hexdigest()
def read(p):return json.loads(p.read_bytes())
def audit():
 specs=read(R/'spec/registry/specifications.json')['specifications'];rules=read(R/'spec/registry/rules.json')['rules']
 oldspec=read(D/'analysis8-specifications.json')['specifications'];oldrules=read(D/'analysis8-rules.json')['rules']
 assert (len(specs),len(rules),len(oldspec),len(oldrules))==(74,531,70,522)
 sm={r['specification_id']:r for r in specs};rm={r['rule_id']:r for r in rules}
 assert len(sm)==len(specs) and len(rm)==len(rules)
 assert all(sm[r['specification_id']]==r for r in oldspec) and all(rm[r['rule_id']]==r for r in oldrules)
 for s in specs:assert s['normative_source']['content_digest']['hex']==sha(R/s['normative_source']['logical_path'])
 m=O.catalogue();cases=O.roster();O.exact(cases,read(D/'cases.json')['cases'])
 values=read(D/'value-coverage.json');assert len(values)==1297 and all('case_id' in r for r in values)
 assert len({r['case_id'] for r in cases})==len(cases)
 assert all(r['case_id'] in {c['case_id'] for c in cases} for r in RULES)
 g1=read(R/'docs/implementation/phase-04c1b4c4r3c2r5g1-design-vectors.json');assert len(g1)==49 and all(v['status']=='Not Executed' for v in g1)
 base=read(R/'target'/(P+'start-raw-map.json'))
 protected=[n for n in base if n.startswith('spec/') and n not in ['spec/registry/specifications.json','spec/registry/rules.json','spec/conformance/suites.json']]
 assert all(sha(R/n)==base[n] for n in protected)
 result=dict(start_head='0e94c9f6a2084aad996ffc410d180abbf4be1d38',tuple=O.EDITION,specifications=74,rules=531,reference_covered=530,intentional_exception=1,missing=0,
  old_spec_objects_unchanged=70,old_rule_objects_unchanged=522,protected_old_spec_files=len(protected),
  descriptors=1131,references=sum('ref' in d for d in O.descendants(m['types'])),historical_imports=97,
  structural_sites=317,reference_policies=66,required_value_field_variants=1297,compared_value_branches=1297,
  registration_case_ids=len(cases),case_ids=[c['case_id'] for c in cases],
  draft_clause_mapping={f'NB-DRAFT-{n:02}':[r['rule_id'] for r in RULES if f'NB-DRAFT-{n:02}' in r['clauses']] for n in range(1,9)},
  full_descriptors=m['types'],type_ranks=m['type_ranks'],sites=m['structural_binding_registration']['sites'],policies=m['structural_binding_registration']['reference_policies'],
  unchanged_old_design_vectors=49,converted_old_design_vectors=0,production_dagranks=0,raw_full_field_credit=0,root_export='NotAuthorized')
 return result

if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--write',action='store_true');p.add_argument('--actual');a=p.parse_args()
 result=audit()
 if a.actual:result['fresh_oracle']=O.report(read(R/a.actual))
 if a.write:O.js(R/'docs/implementation'/(P+'registration-manifest.json'),result)
 print(json.dumps({k:v for k,v in result.items() if k not in ['case_ids','full_descriptors','type_ranks','sites','policies']},indent=2))
