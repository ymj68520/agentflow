"""Independent registration Oracle. NO production graph/rank construction.

Only immutable manually declared registration sources/subgraphs are accepted.
Expected and required rosters never come from Rust Actual or proposed views.
"""
import collections, copy, decimal, functools, hashlib, importlib.util, json, math, re, sys, unicodedata
from pathlib import Path
R=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(R/'tools'))
from structural_binding_registration_data import OWNER,MODEL,SHAPE,OBLIGATION,PREFIX,RULES
D=R/'spec/conformance/g01-structural-binding-staging'
INPUT=['RawValidationSubject']
OUTPUT=['RawPortCheckBasis','RawPortRejectionWitness','CurrentContextRelationSchemaValidation','RelationSubjectAdmission']
CONSUMERS=['NodeBody','ScopeSubject','Source','Target','Coordinate','Origin','RawValidationOrigin','FactKey','TypedContradiction','EvidenceRef']

def read(path):return json.loads(path.read_bytes())
@functools.lru_cache(None)
def catalogue():
    physical=json.loads(re.search(r'```json\s*(.*?)\s*```',(R/'spec/tables/proof-report-model-v7.md').read_text(encoding='utf8'),re.S)[1])
    assert physical['format']=='closed_structural_delta_v1'
    assert set(physical)=={'format','base_source','overrides','append_types','replace_types','append_type_ranks','append_roots'}
    path='spec/tables/proof-report-model-v6.md';raw=(R/path).read_bytes()
    assert physical['base_source']==dict(specification_id='agentflow.computation.proof-report.model@6',logical_path=path,plain_sha256=hashlib.sha256(raw).hexdigest())
    base=json.loads(re.search(r'```json\s*(.*?)\s*```',raw.decode('utf8'),re.S)[1]);m=copy.deepcopy(base)
    assert set(physical['overrides'])=={'model_id','model_edition','obligation_edition','analysis_owner','profile','canonical_constraints','structural_binding_registration'}
    assert set(physical['replace_types'])=={'NodeBody','ScopeSubject'} and not set(physical['append_types'])&set(base['types'])
    m.update(physical['overrides']);m['types'].update(physical['replace_types']);m['types'].update(physical['append_types'])
    m['type_ranks'].update(physical['append_type_ranks']);m['descriptor_roots']+=physical['append_roots'];m['inherited_registered_model']=base
    assert set(m['type_ranks'])==set(m['types']) and set(physical['append_type_ranks'])==set(physical['append_types'])
    return m
def descendants(value):
    if isinstance(value,dict):
        yield value
        for item in value.values():yield from descendants(item)
    elif isinstance(value,list):
        for item in value:yield from descendants(item)
def nominal(name,owner=MODEL):
    a,sep,b=name.partition('/')
    return dict(owner=a,name=b) if sep and re.fullmatch(r'[a-z][a-z0-9.-]*@[1-9][0-9]*',a) else dict(owner=owner,name=name)
def identity(value):return (value['owner'],value['name'])
def closure(types,roots):
    found=set();pending=set(roots)
    while pending:
        n=pending.pop()
        if n in found:continue
        assert n in types,('missing reference',n)
        found.add(n);pending.update(v['ref'] for v in descendants(types[n]) if 'ref' in v)
    return found
def children(d):
    k=d.get('kind')
    if k=='record':return [([['field',n]],v) for n,v in d['fields']]
    if k=='enum':return [([['case',n]],v) for n,v in d['variants']]
    if k in ['sequence','set','option']:return [([[{'sequence':'item','set':'member','option':'some'}[k],None]],d['item'])]
    if k=='map':return [([['key',None]],d['key']),([['value',None]],d['value'])]
    return []

@functools.lru_cache(None)
def required_source_sites():
    # Independent work-list enumeration from exact frozen source, not emitted
    # Site rows, generated Expected or Rust Actual.
    old=json.loads(re.search(r'```json\s*(.*?)\s*```',(R/'spec/tables/proof-report-model-v6.md').read_text(encoding='utf8'),re.S)[1])
    t=old['types'];eligible=closure(t,INPUT+OUTPUT);required={}
    for n in eligible:
        pending=[(t[n],[])]
        while pending:
            d,path=pending.pop()
            if 'ref' in d or d.get('kind')=='dag_ref':continue
            if path:required[(n,json.dumps(path,sort_keys=True))]=d
            pending.extend((v,path+p) for p,v in children(d))
    return old,required

def field_roster(m=None):
    m=m or catalogue();out=[]
    old,_=required_source_sites()
    required=(set(m['types'])-set(old['types']))|closure(old['types'],OUTPUT)|set(CONSUMERS)
    for name in sorted(required):
        stack=[(m['types'][name],[])]
        while stack:
            d,path=stack.pop()
            if 'ref' in d:continue
            if d.get('kind')=='record':
                out.extend(dict(type=name,path=path,field=n,variant=None) for n,_ in d['fields'])
            if d.get('kind')=='enum':
                out.extend(dict(type=name,path=path,field=None,variant=n) for n,_ in d['variants'])
            stack.extend((v,path+p) for p,v in children(d))
    return sorted(out,key=lambda v:json.dumps(v,sort_keys=True))

def audit_registration():
    m=catalogue();old,required=required_source_sites();reg=m['structural_binding_registration'];t=m['types']
    assert m['inherited_registered_model']==old
    assert (m['model_id'],m['analysis_owner'],m['model_edition'],m['obligation_edition'])==(MODEL,OWNER,SHAPE,OBLIGATION)
    assert m['historical_selected_imports']==old['historical_selected_imports'] and len(m['historical_selected_imports'])==97
    assert m['historical_import_sources']==old['historical_import_sources']
    assert all(m['type_ranks'][n]==r for n,r in old['type_ranks'].items())
    assert sorted(m['type_ranks'].values())==list(range(len(t)))
    assert set(m['type_ranks'])==set(t)
    kinds={'record','enum','sequence','set','map','option','dag_ref','unit','null','bool','nfc_text','bytes','u8','u32','u64','i32','i64','decimal','finite_f64','timestamp_i64','duration_i64'}
    for v in descendants(t):
        if 'kind' not in v:continue
        assert v['kind'] in kinds
        for field in ['fields','variants']:
            if field in v:assert len({r[0] for r in v[field]})==len(v[field]) and all(len(r)==2 for r in v[field])
    assert all(v['ref'] in t for v in descendants(t) if 'ref' in v)
    assert set(reg['input_types'])==closure(old['types'],INPUT)
    assert set(reg['output_types'])==closure(old['types'],OUTPUT)
    assert set(reg['authored_types'])==set(t)-set(old['types']) and len(reg['authored_types'])==len(set(reg['authored_types']))
    assert set(reg['coverage_types'])==(set(t)-set(old['types']))|closure(old['types'],OUTPUT)|set(CONSUMERS)
    for n,d in old['types'].items():
        if n not in ['NodeBody','ScopeSubject']:assert t[n]==d,n
    assert t['NodeBody']['variants'][:7]==old['types']['NodeBody']['variants']
    assert t['NodeBody']['variants'][7:]==[['map',dict(kind='sequence',item={'ref':'StructuralMapEntry'},minimum=0)]]
    before=dict(old['types']['ScopeSubject']['variants']);after=dict(t['ScopeSubject']['variants'])
    assert set(before)==set(after)
    assert all(before[n]==after[n] for n in before if n!='raw_contract_validation')
    assert after['raw_contract_validation']==dict(kind='record',fields=[['subject',dict(kind='dag_ref',target={'ref':'RawValidationSubject'})]])
    actual={(r['source_catalogue_key'],json.dumps(r['source_path'],sort_keys=True)):r for r in reg['sites']}
    assert len(actual)==len(reg['sites']) and set(actual)==set(required),('missing/extra sites',set(required)-set(actual),set(actual)-set(required))
    aliases=set()
    for key,desc in required.items():
        row=actual[key];de=row['definition'];assert row['descriptor']==desc and t[row['alias']]==desc
        assert row['alias'] not in aliases;aliases.add(row['alias'])
        assert de['key']==dict(parent=nominal(key[0]),path=[{k:v} for k,v in row['source_path']])
        assert de['value_type']==nominal(row['alias'])
        assert de['parent_source']==nominal(key[0],old['model_id'])
        stage='declared_input' if key[0] in reg['input_types'] else 'raw_fact_capture' if key[0] in ['RawPortCheckBasis','RawPortRejectionWitness'] else 'c_validation'
        assert de['source_stage']==stage
        assert len(de['children'])==len(children(desc))
        for binding,(part,child) in zip(de['children'],children(desc)):
            if 'ref' in child:wanted=nominal(child['ref'])
            elif child.get('kind')=='dag_ref':wanted=nominal(child['target']['ref'])
            else:wanted=nominal(actual[(key[0],json.dumps(row['source_path']+part,sort_keys=True))]['alias'])
            assert binding['position']==dict(part) and binding['value_type']==wanted
    src=closure(t,['StructuralDeclaredInputs'])
    assert not any(d.get('kind')=='dag_ref' for n in src for d in descendants(t[n]))
    # Reference completeness independently includes all declaring fields and
    # each carrier use; inherited input types reused in W must not stay G-local.
    required_refs={}
    for n,d0 in t.items():
        if n in aliases:continue
        pending=[(d0,[])]
        while pending:
            d,path=pending.pop()
            if 'ref' in d:continue
            if d.get('kind')=='dag_ref':
                uses=['output_record']+(['g_value'] if n in reg['input_types'] else [])+(['w_value'] if n in reg['output_types'] else [])
                for use in uses:required_refs[(n,json.dumps(path,sort_keys=True),use)]=nominal(d['target']['ref'])
            else:pending.extend((c,path+p) for p,c in children(d))
    got={}
    for row in reg['reference_policies']:
        name=next(n for n in t if nominal(n)==row['key']['parent'])
        path=[[k,v] for part in row['key']['path'] for k,v in part.items()]
        key=(name,json.dumps(path,sort_keys=True),row['environment']);assert key not in got
        got[key]=row['target'];assert row['domain']==('g_local' if row['environment']=='g_value' else 'g_external')
    assert got==required_refs
    expected_bindings={identity(nominal(n)) for n in closure(old['types'],OUTPUT)}|{identity(nominal(n)) for n in aliases}
    actual_bindings={identity(r['value_type']) for r in reg['value_bindings']}
    assert actual_bindings==expected_bindings and len(actual_bindings)==len(reg['value_bindings'])
    assert reg['stage_contract']['status']['production_binder']=='NotImplemented'
    assert reg['stage_contract']['status']['root_export']=='NotAuthorized'
    return dict(descriptors=len(t),references=sum('ref' in v for v in descendants(t)),sites=len(required),reference_policies=len(got),source_rank_free=True,old_ranks_preserved=774,old_imports_preserved=97,required_field_variants=len(field_roster(m)),production_capture_case_ids=0)

class Refusal(Exception):
    def __init__(self,domain,reason='invalid_shape'):self.domain=domain;self.reason=reason
def fail(domain='UnsupportedModel',reason='invalid_shape'):raise Refusal(domain,reason)
def require(condition,domain='UnsupportedModel',reason='invalid_shape'):
    if not condition:fail(domain,reason)
def key_name(mt,m=None):
    m=m or catalogue()
    for n in m['types']:
        if nominal(n,m['model_id'])==mt:return n
    fail('UnsupportedModel','unknown_nominal_type')

def shape(d,v,depth=0,work=None):
    work=work if work is not None else [0];work[0]+=1
    require(depth<=96 and work[0]<=262144,'ResourceLimit','resource_bound')
    t=catalogue()['types']
    if 'ref' in d:
        name=d['ref'];require(name in t,'IncompleteContent','missing_site');shape(t[name],v,depth+1,work)
        if name in ['ModelType','ModelRef']:key_name(v)
        if name=='SpecificationId':require(valid_spec(v),'InvalidFacts')
        if name=='StructuralSourceType':
            old,_=required_source_sites();require(any(nominal(n,old['model_id'])==v for n in old['types']))
        if name=='StructuralEditionTuple':require(v==dict(analysis=OWNER,model=MODEL,report=SHAPE,obligation=OBLIGATION),'UnsupportedModel','cross_edition')
        if name in ['RawUtf8Bytes','DocumentSourceKey']:
            try:bytes(v).decode('utf8')
            except (ValueError,UnicodeError):fail('InvalidFacts')
            if name=='DocumentSourceKey':require(bool(v),'IncompleteFacts');require(len(v)<=4096,'ResourceLimit','resource_bound')
        if name in ['RawContextPartition','NativeGuardContextPartition']:require(v==[['analysis','canonical_validation']],'InvalidFacts')
        if name=='ContextRolePartition':
            allroles=[r for group in v for r in group];require(sorted(allroles)==['analysis','old_checked','replacement_checked'],'IncompleteFacts');require(any('analysis' in g and 'replacement_checked' in g for g in v),'InvalidFacts')
        if name=='RelationSourceFamily':require(valid_spec(v),'InvalidFacts')
        return
    k=d['kind']
    if k in ['unit','null']:require(v is None)
    elif k=='bool':require(type(v) is bool)
    elif k in ['u8','u32','u64','dag_ref']:require(type(v) is int and 0<=v<(1<<({'u8':8,'u32':32}.get(k,64))))
    elif k in ['i32','i64','timestamp_i64','duration_i64']:
        width=32 if k=='i32' else 64;require(type(v) is int and -(1<<(width-1))<=v<(1<<(width-1)))
    elif k=='finite_f64':require(type(v) in [int,float] and math.isfinite(v))
    elif k=='decimal':require(type(v) is str and re.fullmatch(r'-?(0|[1-9][0-9]*)(\.[0-9]+)?',v) is not None)
    elif k=='nfc_text':require(type(v) is str and unicodedata.normalize('NFC',v)==v)
    elif k=='bytes':require(type(v) is list and all(type(x) is int and 0<=x<=255 for x in v) and ('length' not in d or len(v)==d['length']))
    elif k=='record':
        require(type(v) is dict);wanted={n for n,_ in d['fields']};require(wanted<=set(v),'IncompleteFacts');require(set(v)==wanted)
        for n,c in d['fields']:shape(c,v[n],depth+1,work)
    elif k=='enum':
        cases=dict(d['variants']);unit=all(c.get('kind')=='unit' for c in cases.values())
        if unit:require(type(v) is str and v in cases)
        else:
            require(type(v) is dict and len(v)==1);tag=next(iter(v));require(tag in cases);shape(cases[tag],v[tag],depth+1,work)
    elif k in ['sequence','set']:
        require(type(v) is list);require(len(v)>=d.get('minimum',0),'IncompleteFacts')
        for x in v:shape(d['item'],x,depth+1,work)
        if k=='set':require(len({json.dumps(x,sort_keys=True) for x in v})==len(v),'InvalidFacts','invalid_set_member')
    elif k=='map':
        require(type(v) is list);seen=set()
        for entry in v:
            require(type(entry) is list and len(entry)==2)
            shape(d['key'],entry[0],depth+1,work);shape(d['value'],entry[1],depth+1,work)
            key=json.dumps(entry[0],sort_keys=True);require(key not in seen,'InvalidFacts','duplicate_map_key');seen.add(key)
    elif k=='option':
        require(type(v) is dict and len(v)==1 and next(iter(v)) in ['none','some'])
        if 'none' in v:require(v['none'] is None)
        else:shape(d['item'],v['some'],depth+1,work)
    else:fail()

def valid_spec(v):
    return type(v) is str and re.fullmatch(r'[a-z][a-z0-9-]*(\.[a-z][a-z0-9-]*)*@[1-9][0-9]*',v) is not None and int(v.rsplit('@',1)[1])<2**32

def module_map(inp):
    spec=importlib.util.spec_from_file_location('structural_afidl_reference',R/'tools/gen-afidl-corpus.py');af=importlib.util.module_from_spec(spec);spec.loader.exec_module(af)
    spec2=importlib.util.spec_from_file_location('structural_native_definition_reference',R/'tools/oracle/relation_schema_admission.py');projection=importlib.util.module_from_spec(spec2);spec2.loader.exec_module(projection)
    _,modules=af.compile_module_set([(s['path'],s['text']) for s in inp['sources']])
    entries=[[key,dict(specification_id=key,types=[projection.definition(t) for t in types])] for key,types in sorted(modules.items())]
    proposed=inp.get('proposal',entries);require(type(proposed) is list,'InvalidFacts','invalid_map_entry');keys=[]
    for row in proposed:
        require(type(row) is list and len(row)==2 and valid_spec(row[0]),'InvalidFacts','invalid_map_entry')
        shape({'ref':'CompiledModuleContents'},row[1]);require(row[0]==row[1]['specification_id'],'InvalidFacts','invalid_map_entry')
        require(row[0] not in keys,'InvalidFacts','duplicate_map_key');keys.append(row[0])
    require(sorted(proposed,key=lambda row:row[0])==entries,'InvalidFacts','source_association')
    return dict(entries=entries,entry_count=len(entries),assurance='module_projection_only')

def cmp(a,b):return (a>b)-(a<b)
def compare(d,a,b):
    t=catalogue()['types']
    if 'ref' in d:return compare(t[d['ref']],a,b)
    k=d['kind']
    if k=='dag_ref':return compare(d['target'],a,b)
    if k in ['unit','null']:return 0
    if k=='decimal':return cmp(decimal.Decimal(a),decimal.Decimal(b))
    if k in ['nfc_text','bytes','bool','u8','u32','u64','i32','i64','finite_f64','timestamp_i64','duration_i64']:return cmp(a,b)
    if k=='record':return next((c for n,ch in d['fields'] if (c:=compare(ch,a[n],b[n]))),0)
    if k=='enum':
        unit=all(v.get('kind')=='unit' for _,v in d['variants']);ta=a if unit else next(iter(a));tb=b if unit else next(iter(b));names=[n for n,_ in d['variants']]
        return cmp(names.index(ta),names.index(tb)) or (0 if unit else compare(dict(d['variants'])[ta],a[ta],b[tb]))
    if k=='option':
        if 'none' in a or 'none' in b:return cmp('some' in a,'some' in b)
        return compare(d['item'],a['some'],b['some'])
    for x,y in zip(a,b):
        c=(compare(d['key'],x[0],y[0]) or compare(d['value'],x[1],y[1])) if k=='map' else compare(d['item'],x,y)
        if c:return c
    return cmp(len(a),len(b))

@functools.lru_cache(None)
def binding_index():
    m=catalogue();reg=m['structural_binding_registration']
    return {identity(b['value_type']):b for b in reg['value_bindings']}
def context(mt):
    b=binding_index().get(identity(mt));require(b is not None,'UnsupportedModel','ineligible_value_type')
    return b,catalogue()['types'][b['catalogue_key']]
def slot(parent,path,d):
    if 'ref' in d:return nominal(d['ref'])
    if d.get('kind')=='dag_ref':return nominal(d['target']['ref'])
    for row in catalogue()['structural_binding_registration']['sites']:
        if row['source_catalogue_key']==parent and row['source_path']==path:return row['definition']['value_type']
    fail('IncompleteContent','missing_site')
def body_context(mt):
    b,d=context(mt);parent=b['declaration'];path=b['path']
    while 'ref' in d:parent=d['ref'];path=[];d=catalogue()['types'][parent]
    return b,d,parent,path

def normalize(mt,value,arena,required=None,depth=0):
    require(depth<=32,'ResourceLimit','resource_bound');b,d,parent,path=body_context(mt);k=d['kind']
    require(('g_input' if arena=='g' else 'w_output') in b['eligibility'],'InvalidFacts','input_output_backedge')
    def sub(ch,val,part):
        typ=slot(parent,path+[part],ch)
        # Explicit typed reference targets are expanded source notation only,
        # never admitted as Node<T> or assigned a rank by this Oracle.
        target_arena='g' if ch.get('kind')=='dag_ref' else arena
        return normalize(typ,val,target_arena,required if target_arena==arena else None,depth+1)
    if k in ['unit','null','bool','nfc_text','bytes','u8','u32','u64','i32','i64','decimal','finite_f64','timestamp_i64','duration_i64']:
        shape({'ref':b['catalogue_key']},value)
        if k=='decimal':
            v=value.rstrip('0').rstrip('.') if '.' in value else value
            if decimal.Decimal(v)==0:v='0'
        elif k=='finite_f64':v=0.0 if value==0 else float(value)
        else:v=value
    elif k=='record':
        require(type(value) is dict and set(value)=={n for n,_ in d['fields']},'IncompleteFacts')
        v={n:sub(c,value[n],['field',n]) for n,c in d['fields']}
    elif k=='enum':
        unit=all(c.get('kind')=='unit' for _,c in d['variants']);tag=value if unit else next(iter(value));require(tag in dict(d['variants']))
        payload=None if unit else value[tag];x=sub(dict(d['variants'])[tag],payload,['case',tag]);v=tag if unit else {tag:x}
    elif k in ['sequence','set']:
        require(type(value) is list and len(value)>=d.get('minimum',0),'IncompleteFacts')
        part=['item' if k=='sequence' else 'member',None];v=[sub(d['item'],x,part) for x in value]
        if k=='set':
            v.sort(key=functools.cmp_to_key(lambda a,b:compare(d['item'],a,b)))
            require(all(compare(d['item'],a,b)!=0 for a,b in zip(v,v[1:])),'InvalidFacts','invalid_set_member')
    elif k=='map':
        v=[[sub(d['key'],x,['key',None]),sub(d['value'],y,['value',None])] for x,y in value]
        v.sort(key=functools.cmp_to_key(lambda a,b:compare(d['key'],a[0],b[0])))
        require(all(compare(d['key'],a[0],b[0])!=0 for a,b in zip(v,v[1:])),'InvalidFacts','duplicate_map_key')
    elif k=='option':
        v={'none':None} if 'none' in value else {'some':sub(d['item'],value['some'],['some',None])}
    else:fail()
    if required is not None:required.add(json.dumps([mt,v],sort_keys=True))
    return v

def check_graph(arena,graph,sources,g_nodes=None):
    require(type(graph) is dict and set(graph)=={'nodes','roots'})
    nodes=graph['nodes'];roots=graph['roots'];require(type(nodes) is list and len(nodes)<=64,'ResourceLimit','resource_bound')
    normalized=[];heights=[];local_lists=[];nonrecursive=[];units=[]
    reg=catalogue()['structural_binding_registration'];types=catalogue()['types'];ranks=catalogue()['type_ranks'];work=[0]
    required=set();source_values=[normalize(s['type'],s['value'],arena,required) for s in sources]
    for i,node in enumerate(nodes):
        require(type(node) is dict and set(node)=={'type','body'})
        mt=node['type'];key_name(mt);b,d,parent,path=body_context(mt);k=d['kind']
        require(('g_input' if arena=='g' else 'w_output') in b['eligibility'],'InvalidFacts','input_output_backedge')
        body=node['body'];require(type(body) is dict and len(body)==1);tag=next(iter(body));payload=body[tag];locals_=[];externals=[]
        def edge(ch,value,part):
            expected=slot(parent,path+[part],ch);external=arena=='w' and ch.get('kind')=='dag_ref'
            if arena=='w':
                require(type(value) is dict and len(value)==1,'InvalidFacts','invalid_child')
                require(next(iter(value))==('input' if external else 'local'),'InvalidFacts','invalid_child');rank=next(iter(value.values()))
            else:
                require(type(value) is int,'InvalidFacts','input_output_backedge');rank=value
            target=g_nodes if external else normalized
            count=len(g_nodes) if external else len(nodes)
            require(type(rank) is int and 0<=rank<count,'InvalidFacts','invalid_child')
            if not external:require(rank<i,'InvalidFacts','rank_dependency')
            target_type,target_value=target[rank]
            require(target_type==expected,'InvalidFacts','invalid_child')
            if external:externals.append((expected,target_value))
            else:locals_.append(rank)
            work[0]+=measure(target_value);require(work[0]<=16384,'ResourceLimit','resource_bound')
            return target_value
        scalar=reg['scalar_bodies'].get(k)
        if scalar:
            require(tag=='scalar' and type(payload) is dict and set(payload)=={scalar})
            value=normalize(mt,payload[scalar],arena)
            base=('scalar',)
        elif k=='record':
            require(tag=='record' and type(payload) is list)
            require(len(payload)==len(d['fields']),'IncompleteFacts')
            require([r.get('field') for r in payload]==[n for n,_ in d['fields']])
            value={}
            for (n,ch),row in zip(d['fields'],payload):
                require(set(row)=={'field','value'});value[n]=edge(ch,row['value'],['field',n])
            base=('record',)
        elif k=='enum':
            require(tag=='variant' and type(payload) is dict and set(payload)=={'case','payload'})
            case=payload['case'];require(case in dict(d['variants']));val=edge(dict(d['variants'])[case],payload['payload'],['case',case])
            value=case if all(ch.get('kind')=='unit' for _,ch in d['variants']) else {case:val};base=('variant',[n for n,_ in d['variants']].index(case))
        elif k in ['sequence','set']:
            require(tag==k and type(payload) is list)
            require(len(payload)>=d.get('minimum',0),'IncompleteFacts')
            value=[edge(d['item'],r,['item' if k=='sequence' else 'member',None]) for r in payload];base=(k,len(value))
            if k=='set':require(all(compare(d['item'],a,b)<0 for a,b in zip(value,value[1:])),'InvalidFacts','invalid_set_member')
        elif k=='map':
            require(tag=='map' and type(payload) is list,'InvalidFacts','invalid_map_entry');value=[]
            for row in payload:
                require(type(row) is dict and set(row)=={'key','value'},'InvalidFacts','invalid_map_entry')
                value.append([edge(d['key'],row['key'],['key',None]),edge(d['value'],row['value'],['value',None])])
            require(all(compare(d['key'],a[0],b[0])!=0 for a,b in zip(value,value[1:])),'InvalidFacts','duplicate_map_key')
            require(all(compare(d['key'],a[0],b[0])<0 for a,b in zip(value,value[1:])),'InvalidFacts','invalid_map_entry');base=('map',len(value))
        elif k=='option':
            require(tag in ['option_none','option_some'])
            if tag=='option_none':require(payload is None);value={'none':None};base=('option',0)
            else:value={'some':edge(d['item'],payload,['some',None])};base=('option',1)
        else:fail()
        require(not any(mt==ty and value==v for ty,v in normalized),'InvalidFacts','invalid_interning')
        height=0 if not locals_ else 1+max(heights[j] for j in locals_)
        require(height<=32,'ResourceLimit','resource_bound')
        normalized.append((mt,value));heights.append(height);local_lists.append(locals_);nonrecursive.append((base,externals,d))
        units.append(measure(value))
        if i:
            a,b=nodes[i-1]['type'],mt;ak=key_name(a);bk=key_name(b)
            order=cmp(heights[i-1],height) or cmp((a['owner'],ranks[ak],a['name']),(b['owner'],ranks[bk],b['name']))
            if not order:
                oldbase,oldext,oldd=nonrecursive[i-1]
                order=compare(d,normalized[i-1][1],value) if scalar else cmp(oldbase,base)
                if not order:
                    for (ta,va),(tb,vb) in zip(oldext,externals):
                        order=cmp(identity(ta),identity(tb)) or compare({'ref':key_name(ta)},va,vb)
                        if order:break
                if not order:order=cmp(local_lists[i-1],locals_)
            require(order<0,'InvalidFacts','rank_dependency')
    require(type(roots) is list and len(roots)==len(sources),'IncompleteFacts','missing_node')
    for rank,source,value in zip(roots,sources,source_values):
        require(type(rank) is int and 0<=rank<len(nodes),'InvalidFacts','invalid_child')
        require(normalized[rank][0]==source['type'] and compare({'ref':key_name(source['type'])},normalized[rank][1],value)==0,'InvalidFacts','invalid_child')
    reached=set();todo=list(roots)
    while todo:
        i=todo.pop()
        if i in reached:continue
        reached.add(i);todo.extend(local_lists[i])
    require(len(reached)==len(nodes),'InvalidFacts','invalid_interning')
    got={json.dumps([ty,v],sort_keys=True) for ty,v in normalized}
    require(got==required,'IncompleteFacts','missing_node')
    return normalized

def measure(v,depth=0):
    require(depth<=64,'ResourceLimit','resource_bound');n=1
    if isinstance(v,str):n+=len(v.encode('utf8'))
    elif isinstance(v,list):
        for x in v:n+=measure(x,depth+1);require(n<=16384,'ResourceLimit','resource_bound')
    elif isinstance(v,dict):
        for k,x in v.items():n+=len(k.encode('utf8'))+1+measure(x,depth+1);require(n<=16384,'ResourceLimit','resource_bound')
    require(n<=16384,'ResourceLimit','resource_bound');return n

def subgraph(fixture,proposal):
    sources=read(D/'manual-source-fixtures.json');require(fixture in sources,'UnsupportedModel','missing_site');source=sources[fixture]
    measure(proposal);require(type(proposal) is dict and set(proposal)==({'g','w'} if source.get('w') else {'g'}))
    g=check_graph('g',proposal['g'],source['g'])
    if source.get('w'):check_graph('w',proposal['w'],source['w'],g)

def inspect(inp):
    op=inp['op'];m=catalogue();reg=m['structural_binding_registration']
    if op=='rule':
        row=next((r for r in read(R/'spec/registry/rules.json')['rules'] if r['rule_id']==inp['rule_id']),None)
        require(row is not None,'IncompleteContent','missing_site')
        return dict(registered_rule=row,compiler_semantics=OWNER,positive_capability='NotAuthorized')
    if op=='shape':
        require(inp['type'] in m['types'],'UnsupportedModel','unknown_nominal_type');shape({'ref':inp['type']},inp['value']);return None
    if op=='site':
        shape({'ref':'StructuralSiteDefinition'},inp['value']);expected=next((r['definition'] for r in reg['sites'] if r['definition']['key']==inp['value']['key']),None)
        require(expected is not None,'UnsupportedModel','missing_site');require(expected==inp['value'],'InvalidFacts','missing_site');return None
    if op=='reference':
        shape({'ref':'StructuralReferencePolicy'},inp['value']);key=inp['value']['key'];use=inp['value']['environment']
        require(any(r==inp['value'] for r in reg['reference_policies'] if r['key']==key and r['environment']==use),'InvalidFacts','invalid_child');return None
    if op in ['site_registry','reference_registry','field_roster','stage']:
        expected={'site_registry':[r['definition'] for r in reg['sites']],'reference_registry':reg['reference_policies'],'field_roster':field_roster(m),'stage':reg['stage_contract']}[op]
        require(inp['value']==expected,'IncompleteContent','missing_site');return None
    if op=='dag':subgraph(inp['fixture'],inp['proposal']);return None
    if op=='joint_omission':subgraph(inp['fixture'],inp['expected_view']);subgraph(inp['fixture'],inp['actual_view']);return None
    if op=='production':fail('NotImplemented','production_not_implemented')
    if op=='module_map':return module_map(inp)
    if op=='codec':
        if inp['kind']=='map':v={'map':[dict(key=str(k),value=str(v)) for k,v in inp['entries']]}
        else:v={inp['kind']:str(inp['rank'])}
        return {'canonical_hex':json.dumps(v,separators=(',',':'),ensure_ascii=False,sort_keys=True).encode().hex()}
    fail()

def result(inp):
    try:return dict(ok=True,value=inspect(inp))
    except Refusal as e:return dict(ok=False,failure=dict(domain=e.domain,reason=e.reason,stage='registration_admission',site={'none':None},expected_type={'none':None},observed_type={'none':None},first_input_path=[],second_input_path={'none':None}))

def report(actual):
    audit=audit_registration();cases=read(D/'cases.json')['cases'];manifest=read(D/'manifest.json')
    expected={c['case_id']:dict(case_id=c['case_id'],result=result(c['input']),production_capture_case_ids=0,production_dagranks_generated=0,positive_root_export=False) for c in cases}
    assert len(expected)==len(cases)
    got={r['case_id']:r for r in actual};assert len(got)==len(actual) and set(got)==set(expected),('missing/extra actual cases',set(expected)-set(got),set(got)-set(expected))
    for cid,row in expected.items():assert row==got[cid],(cid,row,got[cid])
    assert set(expected)==set(manifest['metadata_case_ids'])
    return dict(outcome='REGISTRATION_METADATA_ONLY_MATCH',matched=len(got),unexpected_mismatch=0,registration=audit,production_binder='NotImplemented',production_capture_case_ids=0,actual_production_dagranks_generated=0,raw_verified=False,t4='NOT ELIGIBLE',positive_root_export='NotAuthorized')

if __name__=='__main__':
    if '--audit' in sys.argv:print(json.dumps(audit_registration(),indent=2))
    else:print(json.dumps(report(json.load(sys.stdin)),indent=2))
