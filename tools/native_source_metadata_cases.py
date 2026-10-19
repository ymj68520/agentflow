"""Bounded HANDWRITTEN registration subtrees; no native input or producer API.

All ranks are metadata conformance expectations, never production DagRanks.
The small builder is an independent Python calculation for declared test values.
"""
import copy,functools,json,types
from oracle import structural_binding_registration as F
from oracle.native_source_registration import catalogue,MODEL,EDITION,reply
from oracle.native_relation_source_design import native_source
from structural_binding_cases import sample as old_sample

def mt(name):return F.nominal(name,MODEL)
def sample(m,d,route=(),stack=()):
 if d.get('ref')=='StructuralEditionTuple':return dict(EDITION)
 if d.get('ref')=='StructuralSourceType':return F.nominal('$Text','agentflow.computation.proof-report.model@6')
 ns=dict(old_sample.__globals__);ns.update(MODEL=MODEL,OWNER=EDITION['analysis'],SHAPE=EDITION['report'],OBLIGATION=EDITION['obligation'])
 fn=types.FunctionType(old_sample.__code__,ns,old_sample.__name__,old_sample.__defaults__);ns['sample']=fn
 return fn(m,d,route,stack)

def reference():
 """Isolate the frozen independent interpreter in explicit Model8 globals.

 No mutation of its old namespace/cache/files. No Rust Actual is involved.
 """
 ns=dict(F.__dict__)
 for name,obj in F.__dict__.items():
  fn=getattr(obj,'__wrapped__',obj)
  if isinstance(fn,types.FunctionType) and fn.__module__==F.__name__:
   ns[name]=types.FunctionType(fn.__code__,ns,name,fn.__defaults__,fn.__closure__)
 ns.update(catalogue=catalogue,MODEL=MODEL,OWNER=EDITION['analysis'],SHAPE=EDITION['report'],OBLIGATION=EDITION['obligation'],nominal=lambda name,owner=MODEL:F.nominal(name,owner))
 return ns

class HandSubtree:
 def __init__(self,m):
  self.m=m;self.nodes=[];self.intern={};r=m['structural_binding_registration']
  self.bind={b['catalogue_key']:b for b in r['value_bindings']};self.sites={(s['source_catalogue_key'],json.dumps(s['source_path'])):s for s in r['sites']}
 def build(self,key,value):
  b=self.bind[key];parent=b['declaration'];path=b['path'];d=self.m['types'][key]
  while 'ref' in d:parent=d['ref'];path=[];d=self.m['types'][parent]
  def sub(part,desc,v):
   target=desc['ref'] if 'ref' in desc else self.sites[parent,json.dumps(path+[part])]['alias']
   return self.build(target,v)
  k=d['kind'];cs=[];atom=None;base=None
  if k=='record':
   tag='record';atom=[n for n,_ in d['fields']];cs=[sub(['field',n],x,value[n]) for n,x in d['fields']];base=len(cs)
  elif k=='enum':
   tag='variant';atom=value if isinstance(value,str) else next(iter(value));names=[n for n,_ in d['variants']];base=names.index(atom)
   cs=[sub(['case',atom],dict(d['variants'])[atom],None if isinstance(value,str) else value[atom])]
  elif k=='option':
   tag='option_none' if 'none' in value else 'option_some';base=0 if tag=='option_none' else 1
   if tag=='option_some':cs=[sub(['some',None],d['item'],value['some'])]
  elif k in ['sequence','set']:
   assert not value,'Only hand-empty containers in these bounded fixtures';tag=k;base=0;atom=0
  elif k=='map':assert not value;tag='map';base=0;atom=0
  else:
   tag='scalar';atom=value;base=bytes(value) if k=='bytes' else value.encode() if k=='nfc_text' else value
  token=json.dumps([key,tag,atom,cs],sort_keys=True)
  if token in self.intern:return self.intern[token]
  i=len(self.nodes);self.intern[token]=i;self.nodes.append(dict(key=key,tag=tag,kind=k,atom=atom,base=base,children=cs,height=1+max((self.nodes[c]['height'] for c in cs),default=-1)))
  return i
 def finish(self,root):
  ranks={};ordered=[]
  for height in sorted({v['height'] for v in self.nodes}):
   def order(i):
    n=self.nodes[i];typ=mt(n['key']);return (typ['owner'].encode(),self.m['type_ranks'][n['key']],typ['name'].encode(),n['base'],tuple(ranks[c] for c in n['children']))
   for i in sorted((i for i,n in enumerate(self.nodes) if n['height']==height),key=order):ranks[i]=len(ordered);ordered.append(i)
  out=[]
  for i in ordered:
   n=self.nodes[i];tag=n['tag'];cs=[ranks[c] for c in n['children']]
   if tag=='record':v=[dict(field=f,value=c) for f,c in zip(n['atom'],cs)]
   elif tag=='variant':v=dict(case=n['atom'],payload=cs[0])
   elif tag=='scalar':v={self.m['structural_binding_registration']['scalar_bodies'][n['kind']]:n['atom']}
   elif tag=='option_none':v=None
   elif tag=='option_some':v=cs[0]
   else:v=[]
   out.append(dict(type=mt(n['key']),body={tag:v}))
  assert len(out)<=64
  return dict(g=dict(nodes=out,roots=[ranks[root]]))

def fixtures():
 m=catalogue();sources={};graphs={}
 for slot in ['effect_resource','recovery_usage','flow_execution']:
  value=native_source(slot);b=HandSubtree(m);graphs[slot]=b.finish(b.build('NativeRelationSnapshotSource',value))
  sources[slot]=dict(g=[dict(type=mt('NativeRelationSnapshotSource'),value=value)])
  ref=reference();ref['check_graph']('g',graphs[slot]['g'],sources[slot]['g'])
 return sources,graphs

def field_cases():
 m=catalogue();t=m['types'];rows=[];r=m['structural_binding_registration']
 # Exact independently enumerated named field/variant paths, no proposal roster.
 names=set(r['coverage_types']);required=[]
 for name in sorted(names):
  pending=[(t[name],[])]
  while pending:
   d,path=pending.pop()
   if 'ref' in d:continue
   for key,label in [('fields','field'),('variants','case')]:
    for n,desc in d.get(key,[]):required.append((name,path+[[label,n]]))
   pending.extend((child,path+part) for part,child in F.children(d))
 for i,(name,path) in enumerate(required):
  value=sample(m,{'ref':name},path)
  ref=reference()
  try:ref['shape']({'ref':name},value)
  except F.Refusal:
   # Full named semantic constraints (not construction shape) can deliberately
   # reject samples. Never claim those value branches passed.
   rows.append(dict(name=name,path=path,applicability='REQUIRES SEMANTIC INPUT',credit=0));continue
  rows.append(dict(case_id=f'g01-native-source-staging/field-value/{i:04}',input=dict(edition=EDITION,op='shape',name=name,value=value),expected=dict(ok=True,value=reply(None)),field=dict(name=name,path=path),credit='SHAPE VALUE BRANCH ONLY'))
 return rows

def dag_cases():
 sources,graphs=fixtures();rows=[]
 for slot,g in graphs.items():
  rows.append(dict(case_id='g01-native-source-staging/subtree/'+slot,input=dict(edition=EDITION,op='dag',fixture=slot,value=g),expected=dict(ok=True,value=reply(None))))
  for mutation in ['extra-root','duplicate-node','wrong-type','wrong-rank','omit-body']:
   bad=copy.deepcopy(g)
   if mutation=='extra-root':bad['g']['roots'].append(0)
   elif mutation=='duplicate-node':bad['g']['nodes'].append(copy.deepcopy(bad['g']['nodes'][0]))
   elif mutation=='wrong-type':bad['g']['nodes'][-1]['type']=mt('RelationSnapshotContent')
   elif mutation=='wrong-rank':bad['g']['nodes']=list(reversed(bad['g']['nodes']))
   else:del bad['g']['nodes'][-1]['body']
   try:reference()['check_graph']('g',bad['g'],sources[slot]['g'])
   except (F.Refusal,KeyError,AssertionError):pass
   else:raise AssertionError(('accepted mutation',slot,mutation))
   rows.append(dict(case_id='g01-native-source-staging/subtree/'+slot+'/'+mutation,input=dict(edition=EDITION,op='dag',fixture=slot,value=bad),expected=dict(ok=False,failure='InvalidFacts')))
 return sources,rows
