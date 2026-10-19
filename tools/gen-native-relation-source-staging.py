"""Generate reviewed successor registration sources, never a production binder."""
import json,sys
from native_relation_source_registration import R,D,SOURCES,TEXT,source_delta,sections

def write(path,raw):
 p=R/path;p.parent.mkdir(parents=True,exist_ok=True)
 if not p.exists() or p.read_bytes()!=raw:p.write_bytes(raw)

def main():
 for key in ['NB','OS','PP']:write(SOURCES[key][2],(TEXT[key]+sections(key)).encode('utf8'))
 delta=source_delta()
 intro='''# Closed Native Source Binding Model8

Normative closed delta. The exact complete Model7 source (and its exact Model6 dependency) is required as content, not replaced by a checksum. Deep-copy its effective model, retain the entire exact old object as inherited_registered_model, replace ONLY the five listed declarations, apply the exact successor edition header and complete registration object. Existing type ranks,97 historical imports and NodeBody are unchanged. Unknown keys, missing base, bad digest, wrong replacements or altered old declarations reject. This is not a new ACE1 grammar or a production graph constructor.

The registration table inventories all exact nominal owners/fields/sites/children and both C.source environments. All unchanged old normative meanings remain in their old editions. Source fields/stages, failures and non-activation are governed by the new normative sources and rule sections below.

```json
'''
 write(SOURCES['RM'][2],(intro+json.dumps(delta,ensure_ascii=False,separators=(',',':'))+'\n```\n'+sections('RM')).encode('utf8'))
 print(json.dumps(dict(types_replaced=len(delta['type_replacements']),new_type_ranks=0,production_dagranks=0)))
 if '--model-only' not in sys.argv:
  from oracle.native_source_registration import roster,js,D as CASES
  rows=roster();js(CASES/'cases.json',dict(cases=rows,scope='declaration registration and metadata ONLY'))
  from native_source_metadata_cases import dag_cases,field_cases
  sources,_=dag_cases();js(CASES/'manual-source-fixtures.json',sources);js(CASES/'value-coverage.json',field_cases())
  js(CASES/'manifest.json',dict(case_ids=[r['case_id'] for r in rows],production_case_ids=0,production_binder='NotImplemented'))

if __name__=='__main__':main()
