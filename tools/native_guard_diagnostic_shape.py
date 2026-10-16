"""Diagnostic metadata presentation ONLY. Not G01 encoding or Fact Capture.

Converts the isolated getter observations to the existing metadata shape
probe's arrays/closed enum spelling. No Scope, identity, Fact, prerequisite,
Checked value or Expected is generated. Unknown/ambiguous spellings abort.
"""
import json,re,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
m=json.loads(re.search(r'```json\s*(.*?)\s*```',(ROOT/'spec/tables/proof-report-model-v3.md').read_text(encoding='utf-8'),re.S)[1])
def key(s):return s.replace('_','').lower()
def convert(d,x,path='$'):
 if 'ref' in d:
  n=d['ref']
  if n.endswith(('::ComponentInputPort','::ComponentOutputPort')) and isinstance(x,dict) and 'type' in x:
   x={('type_ref' if k=='type' else k):v for k,v in x.items()}
  return convert(m['types'][n],x,path+'<'+n+'>')
 kind=d['kind']
 if kind=='option':
  if x is None or x=={'none':None}:return {'none':None}
  return {'some':convert(d['item'],x['some'] if isinstance(x,dict) and set(x)=={'some'} else x,path+'.some')}
 if kind=='record':
  fields=d['fields']
  if len(fields)==1 and fields[0][0]=='0' and not (isinstance(x,dict) and set(x)=={'0'}):x={'0':x}
  assert isinstance(x,dict) and set(x)=={k for k,_ in fields},(path,'record',x)
  return {k:convert(v,x[k],path+'.'+k) for k,v in fields}
 if kind=='enum':
  if isinstance(x,str):tag=x;value=None
  else:assert isinstance(x,dict) and len(x)==1,(path,x);tag,value=next(iter(x.items()))
  matches=[(n,v) for n,v in d['variants'] if key(n)==key(tag)];assert len(matches)==1,(path,tag,[n for n,_ in d['variants']])
  name,payload=matches[0]
  return name if all(v.get('kind')=='unit' for _,v in d['variants']) else {name:convert(payload,value,path+'.'+name)}
 if kind in ['sequence','set']:return [convert(d['item'],v,path+'[]') for v in x]
 if kind=='map':return [[convert(d['key'],a),convert(d['value'],b)] for a,b in x]
 if kind=='unit':assert x is None,(path,x)
 if kind in ['u8','u32','u64','i32','i64','timestamp_i64','duration_i64'] and isinstance(x,str):
  assert re.fullmatch(r'-?(0|[1-9][0-9]*)',x),(path,x)
  return int(x)
 return x
def main():
 rows=json.loads(Path(sys.argv[1]).read_text(encoding='utf-8'));out=[]
 for r in rows:
  record={
   'source_component':r['native_source_component'],'recovery_action':r['native_recovery_action'],
   'forward_entry_ordinal':r['forward_entry_ordinal'],'action_entry_ordinal':r['action_entry_ordinal'],
   'forward_guard':r['forward_guard'],'declared_action_guard':r['declared_action_guard'],'action_upper':r['action_upper'],
   'verdict':r['guard_verdict'],'basis':r['basis'],'countermodel':r['countermodel'],'reason':r['diagnostic_reason']}
  value=convert({'ref':'ImportedActionGuardCompatibility'},record)
  out.append({'diagnostic_id':r['diagnostic_id'],'canonical_valid':r['canonical_valid'],'component_checked':r['component_checked'],'metadata_type':'ImportedActionGuardCompatibility','metadata_value':value,'capture':False})
 print(json.dumps(out,ensure_ascii=False,sort_keys=True,indent=2))
if __name__=='__main__':main()
