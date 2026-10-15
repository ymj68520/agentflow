"""Static authoring of closed logical descriptors, never a production codec.

Reads frozen declarations, not Rust Actual or runtime layouts. Unknown source
types abort materialization; there is no opaque/any/Other fallback.
"""
import json,re,copy
from pathlib import Path

def ref(n):return {"ref":n}
def atom(k):return {"kind":k}
def rec(**fields):return {"kind":"record","fields":[[k,v] for k,v in fields.items()]}
def enum(cases):return {"kind":"enum","variants":[[k,v] for k,v in cases]}
UNIT=atom("unit")

class Parser:
 def __init__(self,s):
  self.ts=re.findall(r"[A-Za-z_][A-Za-z_0-9@/.-]*(?:::[A-Za-z_][A-Za-z_0-9@/.-]*)*|[{}<>,:|()]",s);self.i=0
 def peek(self):return self.ts[self.i] if self.i<len(self.ts) else None
 def take(self,v=None):
  x=self.peek()
  if v is not None and x!=v:raise ValueError(("expected",v,"found",x,self.ts[max(0,self.i-5):self.i+8]))
  self.i+=1;return x
 def fields(self):
  self.take("{");a=[]
  while self.peek()!="}":
   n=self.take();self.take(":");v=self.expr();a.append([n,v])
   if self.peek()!=",":break
   self.take(",")
  self.take("}");return {"kind":"record","fields":a}
 def term(self):
  if self.peek()=="{":return self.fields()
  n=self.take()
  if self.peek()==":":self.take();return {"_case":n,"payload":self.term()}
  if self.peek()=="{":return {"_case":n,"payload":self.fields()}
  if self.peek()=="<":
   self.take();args=[self.expr()]
   while self.peek()==",":self.take();args.append(self.expr())
   self.take(">");return {"_generic":n,"args":args}
  return ref(n)
 def expr(self):
  a=[self.term()]
  while self.peek()=="|":self.take();a.append(self.term())
  if len(a)>1 or "_case" in a[0]:
   return enum([(x["_case"],x["payload"]) if "_case" in x else (x["ref"],UNIT) for x in a])
  return a[0]
def parse_defs(s):
 s=re.sub(r"//[^\n]*","",s);matches=list(re.finditer(r"(?m)^([A-Za-z_][A-Za-z_0-9]*)\s*=",s));d={}
 for i,m in enumerate(matches):
  p=Parser(s[m.end():matches[i+1].start() if i+1<len(matches) else len(s)]);v=p.expr()
  if p.i!=len(p.ts):raise ValueError(("unconsumed",m[1],p.ts[p.i:]))
  d[m[1]]=v
 return d

class Graph:
 def __init__(self,legacy,defs,root):
  self.legacy=legacy;self.defs=defs;self.root=Path(root);self.types={};self.errors=[];self.raw_pending={}
  self.native=legacy["semantic_model_declarations"]
  self.afidl={x["schema"]+"/"+x["name"]:x for x in legacy["imported_afidl_declarations"]}
  self.primitives={"unit":"unit","null":"null","Bool":"bool","bool":"bool","Bytes":"bytes","Text":"nfc_text","NfcText":"nfc_text","String":"nfc_text","str":"nfc_text","U32":"u32","u32":"u32","I32":"i32","i32":"i32","U64":"u64","u64":"u64","I64":"i64","i64":"i64","Ordinal64":"u64","usize":"u64","uint32":"u32","uint64":"u64","int32":"i32","int64":"i64","boolean":"bool","string":"nfc_text","bytes":"bytes","decimal":"decimal","float64":"finite_f64","timestamp":"timestamp_i64","duration":"duration_i64","f64":"finite_f64","u8":"u8"}
  self.aliases={n:"U32" for n in "ComputationId ExpressionId RunParameterId ValueSlotId ResourceSlotId PortNo ContractRefId SemanticTypeId".split()}
  self.aliases.update({n:"Text" for n in "SpecificationId RuleId OwnerVersion ModelEdition ObligationEdition ClosedField ClosedCase ModelTypeName Sha256Hex".split()})
  self.aliases.update({n:"U32" for n in ("BranchOrdinal","HandlerOrdinal")})
  for n,target in self.aliases.items():self.defs.setdefault(n,{"kind":"u32" if target=="U32" else "nfc_text","nominal":n})
  self.preferred={n:"af-semantic::"+n for n in ["TypeSchemaRef","DigestRefValue","StructuralType","PrimitiveConstant"]}
  self.preferred.update({"ComponentContract":"af-contract::ComponentContract","ComponentIdentity":"af-contract::ComponentIdentity","EffectUnprovenReason":"af-effect::UnprovenReason","ResourceCompatibilityUnprovenReason":"af-resource::UnprovenReason","FlowCompatibilityUnprovenReason":"af-flow::UnprovenReason","UsageUnprovenReason":"af-usage::UsageUnprovenReason","ExecutionCompatibility":"af-execution::ExecutionCompatibility","CanonicalTypeSchema":"af-idl::CanonicalTypeSchema"})
 def resolve(self,n,owner=""):
  if n.startswith(("crate::","self::","super::")):n=n.rsplit("::",1)[-1]
  if n in self.defs:return n
  if n in self.primitives:return "$"+n
  if n in self.aliases:return self.resolve(self.aliases[n],owner)
  if n in self.preferred:n=self.preferred[n]
  n=re.sub(r"^af_([a-z_]+)::",lambda m:"af-"+m[1].replace("_","-")+"::",n)
  if n in self.native or n in self.afidl:return n
  local=owner.split("::")[0]+"::"+n
  if local in self.native:return local
  hits=[k for k in self.native if k.rsplit("::",1)[-1]==n]
  if len(hits)==1:return hits[0]
  raise ValueError(("unresolved",n,"from",owner,"candidates",hits))
 def ensure(self,n,owner=""):
  n=self.resolve(n,owner)
  if n in self.types:return ref(n)
  self.types[n]=None
  if n.startswith("$"):v=atom(self.primitives[n[1:]])
  elif n in self.defs:v=self.compile(self.defs[n],n)
  elif n in self.afidl:v=self.af(self.afidl[n]["body"],n)
  else:
   d=self.native[n]
   if d["kind"]=="struct":v={"kind":"record","fields":[[x["field"],self.rust(x["full_type"],n)] for x in d["fields"]]}
   elif d["kind"]=="enum":
    # JSON maps do not declare semantic order. Reproduce the exact frozen
    # source order via the existing static declaration reader when available.
    variants=list(d["variants"])
    source=(self.root/d["source"]).read_text(encoding="utf-8")
    name=n.rsplit("::",1)[-1]
    match=re.search(r"\benum\s+"+re.escape(name)+r"\b[^{}]*\{",source)
    if match:
     order={x:source.find(x,match.end()) for x in variants};variants.sort(key=lambda x:order[x])
    v=enum([(k,{"kind":"record","fields":[[x["field"],self.rust(x["full_type"],n)] for x in d["variants"][k]]} if d["variants"][k] else UNIT) for k in variants])
   else:raise ValueError(("unsupported declaration",n,d["kind"]))
  self.types[n]=v;return ref(n)
 def compile(self,v,owner=""):
  if "ref" in v:return self.ensure(v["ref"],owner)
  if "_generic" in v:
   k=v["_generic"];args=v["args"]
   if k in ("Seq","Set","NonemptySet","NonemptySeq","Opt"):
    return {"kind":{"Seq":"sequence","Set":"set","NonemptySet":"set","NonemptySeq":"sequence","Opt":"option"}[k],"item":self.compile(args[0],owner),"minimum":1 if k.startswith("Nonempty") else 0}
   if k=="Map":return {"kind":"map","key":self.compile(args[0],owner),"value":self.compile(args[1],owner)}
   if k=="Node":return {"kind":"dag_ref","target":self.compile(args[0],owner)}
   if k=="Raw":
    compiled=self.compile(args[0],owner);return self.raw(compiled)
   raise ValueError(("unknown generic",k))
  return {k:([[a,self.compile(b,owner)] for a,b in val] if k in ("fields","variants") else val) for k,val in v.items()}
 def raw(self,v):
  if "ref" in v:
   n=v["ref"];r="Raw("+n+")"
   if r not in self.types:
    self.types[r]=None
    if self.types[n] is None:self.raw_pending[r]=n
    else:self.types[r]=self.raw(self.types[n])
   return ref(r)
  if v.get("kind")=="nfc_text":return atom("bytes")
  return {k:(self.raw(x) if isinstance(x,dict) else [[a,self.raw(b)] for a,b in x] if k in ("fields","variants") else x) for k,x in v.items()}
 def rust(self,s,owner):
  s=re.sub(r"&(?:'[A-Za-z_]+\s+)?(?:mut\s+)?","",s).strip()
  s=re.sub(r"<'[A-Za-z_, '\s]+>","",s)
  if s=="[u8; 32]":return {"kind":"bytes","length":32}
  a=re.fullmatch(r"\[(.+);\s*(\d+)\]",s)
  if a:return {"kind":"sequence","item":self.rust(a[1],owner),"length":int(a[2]),"minimum":int(a[2])}
  if s.startswith("(") and s.endswith(")"):
   return {"kind":"record","fields":[[str(i),self.rust(t,owner)] for i,t in enumerate(split_args(s[1:-1]))]}
  m=re.fullmatch(r"([\w:]+)<(.*)>",s)
  if m:
   k=m[1].split("::")[-1];args=split_args(m[2])
   if k in ("Box","Arc","Rc"):return self.rust(args[0],owner)
   if k=="Vec" and args==["u8"]:return atom("bytes")
   if k in ("Vec","BTreeSet","HashSet","Option"):
    return {"kind":{"Vec":"sequence","BTreeSet":"set","HashSet":"set","Option":"option"}[k],"item":self.rust(args[0],owner),"minimum":0}
   if k in ("BTreeMap","HashMap"):return {"kind":"map","key":self.rust(args[0],owner),"value":self.rust(args[1],owner)}
   raise ValueError(("unknown native generic",s,owner))
  return self.ensure(s,owner)
 def af(self,v,owner):
  tag,payload=v
  if tag=="ref":return self.ensure(payload,owner)
  if tag=="primitive":return self.ensure(payload,owner)
  if tag in ("list","option"):return {"kind":"sequence" if tag=="list" else "option","item":self.af(payload,owner),"minimum":0}
  if tag=="alias":return self.af(payload,owner)
  if tag=="record":return {"kind":"record","fields":[[x["name"],self.af(x["type"],owner)] for x in payload["fields"]]}
  if tag=="enum":return enum([(x["name"],UNIT) for x in payload["values"]])
  if tag=="variant":return enum([(x["name"],self.af(x["type"],owner)) for x in payload["cases"]])
  if tag=="map":return {"kind":"map","key":self.af(payload[0],owner),"value":self.af(payload[1],owner)}
  raise ValueError(("unknown AFIDL constructor",tag,payload))
 def finish(self,roots):
  for n in roots:self.ensure(n)
  while self.raw_pending:
   pending=self.raw_pending;self.raw_pending={}
   for r,n in pending.items():
    assert self.types[n] is not None,n
    self.types[r]=self.raw(self.types[n])
  assert all(x is not None for x in self.types.values())
  return self.types

def split_args(s):
 a=[];level=0;start=0
 for i,c in enumerate(s):
  if c in "<([":level+=1
  elif c in ">)]":level-=1
  elif c=="," and level==0:a.append(s[start:i].strip());start=i+1
 if s[start:].strip():a.append(s[start:].strip())
 return a
