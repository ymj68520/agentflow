"""Approved historical-snapshot + live-declaration double check.

Implementation hooks may change bodies; neither full historical contents nor
the imported declarations/field types/payloads/variant ranks may change.
This module does not regenerate an Expected or a registered source.
"""
import hashlib,json,re
from pathlib import Path
from oracle.rust_type_inventory import clean_comments,balanced,split_top,field
ROOT=Path(__file__).resolve().parent.parent
def parse_named(source,name,path):
 source=clean_comments(source);matches=[]
 for m in re.finditer(r'\b(struct|enum)\s+'+re.escape(name)+r'\b',source):
  pos=m.end()
  while pos<len(source) and source[pos].isspace():pos+=1
  if source[pos:pos+1]=='<':_,pos=balanced(source,pos)
  while pos<len(source) and source[pos].isspace():pos+=1
  if source[pos:pos+1] not in ['{','(']:continue
  body,_=balanced(source,pos);kind=m[1]
  row={'owner_type':Path(path).parts[1]+'::'+name,'kind':kind,'source':path}
  if kind=='struct':row['fields']=[field(p,i) for i,p in enumerate(split_top(body))]
  else:
   row['variants']={}
   for p in split_top(body):
    tag=re.match(r'(\w+)',p);assert tag,(name,p)
    rest=p[tag.end():].strip()
    contents=balanced(rest,0)[0] if rest.startswith(('{','(')) else ''
    row['variants'][tag[1]]=[field(p,i) for i,p in enumerate(split_top(contents))]
  matches.append(row)
 assert len(matches)==1,('ambiguous/missing declaration',path,name,len(matches))
 return matches[0]
def check(model,manifest,overrides=None):
 assert model['historical_selected_imports']==manifest['selected_imports']
 assert model['historical_import_sources']==manifest['sources']
 frozen={}
 for path,row in manifest['sources'].items():
  data=bytes.fromhex(row['content_hex'])
  assert hashlib.sha256(data).hexdigest()==row['sha256'],path
  frozen[path]=data.decode('utf-8')
 compared=[]
 for row in manifest['selected_imports']:
  expected=row['historical_declaration']
  if expected is None:continue
  path=expected['source'];name=row['name'].split('::')[-1]
  historical=parse_named(frozen[path],name,path)
  live=parse_named((overrides or {}).get(path,(ROOT/path).read_text(encoding='utf-8')),name,path)
  assert historical==expected,('historical declaration/content mismatch',row['name'])
  assert live==historical,('live declaration drift',row['name'])
  if historical['kind']=='enum':assert list(live['variants'])==list(historical['variants']),('variant rank drift',row['name'])
  compared.append(row['name'])
 return {'historical_source_bundles':len(frozen),'selected_imports':len(manifest['selected_imports']),'live_declarations_compared':len(compared),'names':compared,'snapshot_content_exact':True,'live_fields_types_payloads_variant_ranks_exact':True}
def main():
 model=json.loads(re.search(r'```json\s*(.*?)\s*```',(ROOT/'spec/tables/proof-report-model-v3.md').read_text(encoding='utf-8'),re.S)[1])
 manifest=json.loads((ROOT/'spec/conformance/g01-native-guard-staging/historical-imports.json').read_bytes())
 print(json.dumps(check(model,manifest),indent=2))
if __name__=='__main__':main()
