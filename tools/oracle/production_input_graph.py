"""Independent source-to-Input-G model. No Rust process/Actual feeds Expected.

Catalogue and source inputs are fixed before graph construction. Full typed
node bodies, references and every rank are compared, not just counts/digests.
"""
import base64,copy,decimal,functools,hashlib,importlib.util,json,sys
import re,unicodedata
from pathlib import Path
R=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(R/'tools'))
from oracle.structural_binding_registration import catalogue,nominal,audit_registration
from portable_proof_python_oracle import AFIDL,canonical_bytes
P='phase-04c1b4c4r3c2r4g3-'
def load_module(name,path):
    s=importlib.util.spec_from_file_location(name,R/path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
REL=load_module('g3_native_afidl_projection_reference','tools/oracle/relation_schema_admission.py')
PORTABLE=load_module('g3_portable_source_reference','tools/gen-portable-proof-codec-corpus.py')
RAW=load_module('g3_raw_declared_input_reference','tools/oracle/raw_port_capture.py')
def read(path):return json.loads((R/path).read_bytes())
def b64(v):return base64.urlsafe_b64encode(v).rstrip(b'=').decode('ascii')
def order(a,b):return (a>b)-(a<b)

def validate_flow_source(value):
    """G3 declared CANONICAL native snapshot source, not constructor history.

    Frozen CAT.sink_key describes FlowSink, not FlowExecutionSink. Do not
    modify that historical Oracle or reuse it for exact-effect sink inputs.
    These fixtures have at most one exact-effect atom per kind; no Debug-key
    ordering is inferred. Rust additionally asserts constructor output equals
    the complete supplied native catalog before invoking the Binder.
    """
    root,family=REL.info('flow_execution');_,types=REL.compiled('flow_execution','A')
    assert REL.accepts(types[family+'/'+root]['body'],value,types)
    named=lambda v:(v['catalog'],v['name'])
    for field in ['closed_regions','closed_trust_domains']:
        keys=[named(v) for v in value[field]];assert keys==sorted(set(keys))
    for field,key in [('confidentiality_rules','label'),('taint_rules','taint')]:
        keys=[named(v[key]) for v in value[field]];assert keys==sorted(set(keys))
    authorities=['none','external_data','delegated_agent','user','workflow','platform']
    keys=[authorities.index(r['authority']) for r in value['authority_rules']];assert keys==sorted(set(keys))
    for r in value['confidentiality_rules']:
        pairs=r['region_pairs'].get('finite_exact_pairs')
        if pairs is not None:
            keys=[(named(p['execution_region']),named(p['residency_region'])) for p in pairs];assert keys==sorted(set(keys))
            assert all(p['execution_region'] in value['closed_regions'] and p['residency_region'] in value['closed_regions'] for p in pairs)
    for r in value['taint_rules']+value['authority_rules']+value['sink_rules']:
        domains=r['trust_domains'].get('exact_allowlist')
        if domains is not None:
            keys=[named(v) for v in domains];assert keys==sorted(set(keys));assert all(v in value['closed_trust_domains'] for v in domains)
    groups={}
    keys=[]
    for r in value['sink_rules']:
        kind=['data_egress','decision_influence'].index(r['kind']);tag,payload=next(iter(r['sink'].items()))
        if tag=='workflow_output':assert payload>0;key=(kind,0,payload)
        else:
            assert tag=='exact_effect';groups[kind]=groups.get(kind,0)+1;assert groups[kind]<=1
            key=(kind,1,0)
        keys.append(key)
    assert keys==sorted(set(keys))
    return value

@functools.lru_cache(None)
def compiler_source():
    # This frozen G2 INPUT manifest selects sources/rules. Their complete
    # values below derive from actual normative source bytes/Registry records,
    # never from a production graph or its source receipt.
    profile=read('spec/conformance/g01-structural-binding-staging/compiler-current.json')
    specs_bytes=(R/'spec/registry/specifications.json').read_bytes();rules_bytes=(R/'spec/registry/rules.json').read_bytes()
    assert bytes.fromhex(profile['registry_hex'])==specs_bytes and bytes.fromhex(profile['rule_registry_hex'])==rules_bytes
    registry=json.loads(specs_bytes);rules=json.loads(rules_bytes)['rules'];sources=[]
    for row in registry['specifications']:
        path=row['normative_source']['logical_path'];data=(R/path).read_bytes()
        digest=hashlib.sha256(data).hexdigest();assert digest==row['normative_source']['content_digest']['hex']
        sources.append(dict(specification_id=row['specification_id'],logical_path=path,content_sha256=digest,content=data))
    selected=sorted((s for s in sources if s['specification_id'] in profile['selected_specifications']),key=lambda s:s['specification_id'])
    selected_rules=[{k:r[k] for k in ['rule_id','normative_source','source_anchor','short_statement']} for r in rules if r['rule_id'] in profile['applied_rules']]
    selected_rules.sort(key=lambda r:r['rule_id'])
    paths=[s['logical_path'] for s in sources if s['logical_path'].endswith('.afidl')]
    _,modules=AFIDL.compile_module_set([(p,(R/p).read_text(encoding='utf8')) for p in paths])
    pins=[]
    for values in modules.values():
        for t in values:
            _,digest=AFIDL.type_schema_digest(t)
            pins.append(dict(specification_id=t['schema'],type_name=t['name'],schema_digest=dict(algorithm='sha256',domain='type-schema',hex=digest)))
    pins.sort(key=lambda p:(p['specification_id'],p['type_name']))
    role_rows=[]
    for role in PORTABLE.ROLE_NAMES:
        declaration={'model_missing':'not_formalized'} if role in ['plan_lowering','runtime_semantics_profile'] else {'defined':dict(analysis_semantics_version=profile['analysis'],normative_specifications=sorted(profile['selected_specifications']),required_schema_pins=pins,depends_on=PORTABLE.DEPENDENCIES[role])}
        role_rows.append(dict(role=role,declaration=declaration))
    tables={name:(R/path).read_bytes() for name,path in [('hash_domains','spec/tables/hash-domains.json'),('abi_numbers','spec/registry/abi-numbers.json'),('rule_areas','spec/tables/rule-areas.json'),('afidl_primitives','spec/tables/afidl-primitives.json')]}
    support=dict(normative_sources=sources,closed_tables=tables,selected_specifications=sorted(profile['selected_specifications']),applied_rules=selected_rules,semantic_roles=role_rows,attribution={'full_content_revalidation':{'listed_rules':selected_rules}},report_shape=profile['report'],obligation_edition=profile['obligation'])
    assert len(sources)==70 and len(selected)==44 and len(selected_rules)==217
    return dict(analysis_semantics_version=profile['analysis'],registry_content=specs_bytes,rule_registry_content=rules_bytes,specifications=selected,rules=selected_rules,support_manifest=support)

def empty_context_with_actual_modules():
    registry=read('spec/registry/specifications.json')
    paths=[s['normative_source']['logical_path'] for s in registry['specifications'] if s['normative_source']['logical_path'].endswith('.afidl')]
    _,modules=AFIDL.compile_module_set([(p,(R/p).read_text(encoding='utf8')) for p in paths])
    return dict(contents=dict(compiled_modules=[[key,dict(specification_id=key,types=[REL.definition(t) for t in rows])] for key,rows in sorted(modules.items())],
        type_schemas=[],effect_catalogs=[],resource_types=dict(schemas=[],descriptors=[]),flow_catalogs=dict(confidentiality=[],taints=[],sinks=[]),usage_meters=[],
        recovery_operations=dict(effects=[],reconciliation_operations=[]),execution_universes=dict(capabilities=[],trust_domains=[],regions=[],region_compatibility=[],runtime_profiles=[]),
        relation_snapshots={k:{'none':None} for k in ['effect_resource','recovery_usage','flow_execution']}),roles=[['analysis','canonical_validation']])

@functools.lru_cache(None)
def environment_modules(missing=False):
    paths=[s['normative_source']['logical_path'] for s in read('spec/registry/specifications.json')['specifications'] if s['normative_source']['logical_path'].endswith('.afidl')]
    paths.append('spec/conformance/production-input-graph/environment.afidl')
    if missing:paths=[p for p in paths if not p.endswith('effect-resource-compatibility.afidl')]
    _,modules=AFIDL.compile_module_set([(p,(R/p).read_text(encoding='utf8')) for p in paths])
    return modules,{t['schema']+'/'+t['name']:t for rows in modules.values() for t in rows}

def case_context(case):
    cfg=read('spec/conformance/production-input-graph/environment.json');modules,types=environment_modules(case['current_modules']=='missing_relation')
    if case['current_modules']=='empty':modules={}
    elif case['current_modules']=='single':modules={k:v for k,v in modules.items() if k=='binder.fixture@1'}
    _,original_types=environment_modules()
    value=empty_context_with_actual_modules();c=value['contents'];c['compiled_modules']=[[key,dict(specification_id=key,types=[REL.definition(t) for t in rows])] for key,rows in sorted(modules.items())]
    def pin(name):
        t=original_types[name];_,digest=AFIDL.type_schema_digest(t)
        return dict(specification_id=t['schema'],type_name=t['name'],schema_digest=dict(algorithm='sha256',domain='type-schema',hex=digest))
    if case['context']=='rich':
        c['type_schemas']=[[pin(n),REL.definition(original_types[n])] for n in cfg['type_schemas']]
        resource=pin(cfg['resource_schema']);c['resource_types']=dict(schemas=[[resource,REL.definition(original_types[cfg['resource_schema']])]],descriptors=[[resource,dict(specification_id=resource['specification_id'],type_name=resource['type_name'],schema_hex=resource['schema_digest']['hex'],recognized_attributes=cfg['resource_attributes'])]])
        c['effect_catalogs']=[[x['catalog'],x['operations']] for x in cfg['effect_catalogs']]
        c['flow_catalogs']=dict(confidentiality=cfg['confidentiality'],taints=cfg['taints'],sinks=[[{k:r[k] for k in ['catalog','name']},r['decision']] for r in cfg['sinks']])
        c['usage_meters']=[[dict(catalog=cfg['meter_catalog'],name=r['name']),dict(meter=dict(catalog=cfg['meter_catalog'],name=r['name']),unit=cfg['unit'],aggregation=r['aggregation'])] for r in cfg['meters']]
        c['recovery_operations']=dict(effects=c['effect_catalogs'],reconciliation_operations=cfg['reconciliation'])
        c['execution_universes']=dict(capabilities=[[cfg['capability'],dict(capability=cfg['capability'],exact_version=cfg['versions'][0],supported_versions=cfg['versions'],attributes=cfg['attributes'])]],trust_domains=cfg['trust_domains'],regions=cfg['regions'],region_compatibility=[dict(zip(['from','to'],cfg['regions']))],runtime_profiles=[[cfg['profile'],dict(profile=cfg['profile'],family=cfg['family'],features=cfg['features'])]])
    if case['relations']=='nonempty':
        catalogs=read('spec/conformance/production-input-graph/relations.json')
        for slot,catalog in catalogs.items():
            validate_flow_source(catalog) if slot=='flow_execution' else REL.validate_native_catalog(slot,catalog)
            data=REL.CAT.ace1(catalog).encode();family=REL.info(slot)[1]
            source=dict(kind=slot,schema_family=family,revision='g3-actual',pin=dict(family=family,revision='g3-actual',canonical_content_checksum=dict(algorithm='sha256',hex=hashlib.sha256(data).hexdigest())),canonical_content=data,catalog={slot:catalog})
            c['relation_snapshots'][slot]={'some':source}
    return value

def declared_case(case):
    bundle=read('spec/conformance/production-input-graph/'+case['bundle']);row=bundle['declarations'][case['selected_entry']]
    path=[{'field':{'name':list(b'selected')}}] if case['slot']==1 else [{'entry':{'ordinal':case['selected_entry']}}]
    return subject(dict(package=bundle['package'],candidate=row['candidate']),case_context(case),owner=case['owner'],slot=case['slot'],path=path,physical=case['slot']==1)

def subject(source,context,owner=0,slot=0,path=None,physical=False):
    path=path if path is not None else [{'entry':{'ordinal':0}}]
    package=source['package'];actual=RAW.document(package,owner,slot)
    binding=dict(source_package=package,source_owner=package['owners'][owner]['anchor'],occurrence_key={'entry_ordinal':slot},actual_document=actual)
    return dict(owner_compiler=compiler_source(),input=dict(candidate=source['candidate'],origin=dict(document_binding=binding,path=path),
        physical={'supplied':dict(document_binding=binding,value_path=path)} if physical else {'not_supplied':None}),context=context)

class Builder:
    def __init__(self):
        self.m=catalogue();r=self.m['structural_binding_registration'];self.nodes=[];self.intern={};self.sites=set()
        self.bindings={b['catalogue_key']:b for b in r['value_bindings'] if 'g_input' in b['eligibility']}
        self.site_index={(s['source_catalogue_key'],json.dumps(s['source_path'])):s for s in r['sites']}
    def descriptor(self,key):
        assert key in self.bindings,('ineligible input type',key)
        b=self.bindings[key];parent=b['declaration'];path=b['path'];d=self.m['types'][key]
        while 'ref' in d:parent=d['ref'];path=[];d=self.m['types'][parent]
        return parent,path,d
    def child(self,parent,path,part,d):
        if 'ref' in d:return d['ref']
        if d['kind']=='dag_ref':
            formal=[{k:v} for k,v in path+[part]]
            policies=self.m['structural_binding_registration']['reference_policies']
            policy=next(p for p in policies if p['key']==dict(parent=nominal(parent),path=formal) and p['environment']=='g_value')
            assert policy['domain']=='g_local'
            return d['target']['ref']
        row=self.site_index[(parent,json.dumps(path+[part]))];assert row['definition']['eligibility']=='g_input'
        self.sites.add(row['alias']);return row['alias']
    def semantic_order(self,a,b):
        x,y=self.nodes[a],self.nodes[b];assert x['key']==y['key']
        if x['tag']=='scalar':return order(x['semantic'],y['semantic'])
        c=order(x['semantic'],y['semantic']) if x['tag'] in ['variant','option_none','option_some'] else 0
        if c:return c
        for i,j in zip(x['children'],y['children']):
            c=self.semantic_order(i,j)
            if c:return c
        return order(len(x['children']),len(y['children']))
    def build(self,key,value,depth=0):
        assert depth<=96
        parent,path,d=self.descriptor(key);kind=d['kind'];children=[];tag=kind;payload=None;semantic=None
        def sub(part,d,v):return self.build(self.child(parent,path,part,d),v,depth+1)
        scalar=self.m['structural_binding_registration']['scalar_bodies'].get(kind)
        if scalar:
            tag='scalar';payload=value
            if kind=='bytes':payload=value if isinstance(value,bytes) else bytes(value)
            if kind=='decimal':
                v=decimal.Decimal(value);text=format(v,'f');text=text.rstrip('0').rstrip('.') if '.' in text else text
                payload='0' if v==0 else text;semantic=v
            elif kind=='nfc_text':
                import unicodedata
                assert unicodedata.normalize('NFC',value)==value;semantic=value.encode('utf8')
            else:semantic=payload
        elif kind=='record':
            assert set(value)=={n for n,_ in d['fields']};payload=tuple(n for n,_ in d['fields'])
            children=[sub(['field',n],desc,value[n]) for n,desc in d['fields']]
        elif kind=='enum':
            case=value if isinstance(value,str) else next(iter(value));cases=[n for n,_ in d['variants']]
            desc=dict(d['variants'])[case];payload=case;semantic=cases.index(case);tag='variant'
            children=[sub(['case',case],desc,None if isinstance(value,str) else value[case])]
        elif kind=='option':
            tag='option_none' if 'none' in value else 'option_some';semantic=0 if tag=='option_none' else 1
            if tag=='option_some':children=[sub(['some',None],d['item'],value['some'])]
        elif kind in ['sequence','set']:
            assert len(value)>=d.get('minimum',0)
            children=[sub(['member' if kind=='set' else 'item',None],d['item'],v) for v in value]
            if kind=='set':
                children.sort(key=functools.cmp_to_key(self.semantic_order))
                assert all(self.semantic_order(a,b)!=0 for a,b in zip(children,children[1:])),('duplicate set member',key)
            payload=len(children)
        elif kind=='map':
            pairs=[(sub(['key',None],d['key'],k),sub(['value',None],d['value'],v)) for k,v in value]
            pairs.sort(key=functools.cmp_to_key(lambda a,b:self.semantic_order(a[0],b[0])))
            assert all(self.semantic_order(a[0],b[0])!=0 for a,b in zip(pairs,pairs[1:])),('duplicate canonical key',key)
            children=[n for p in pairs for n in p];payload=len(pairs)
        else:raise AssertionError(('unregistered body',key,kind))
        token=(key,tag,payload,tuple(children))
        if token in self.intern:return self.intern[token]
        n=len(self.nodes);self.intern[token]=n
        self.nodes.append(dict(key=key,tag=tag,kind=kind,payload=payload,semantic=semantic,children=children,height=1+max((self.nodes[c]['height'] for c in children),default=-1)))
        return n
    def finish(self,root):
        seen=set();pending=[root]
        while pending:
            n=pending.pop()
            if n not in seen:seen.add(n);pending+=self.nodes[n]['children']
        assert seen==set(range(len(self.nodes)))
        ranks={};ordered=[]
        for height in sorted({n['height'] for n in self.nodes}):
            def key(i):
                n=self.nodes[i];mt=nominal(n['key']);base=n['semantic'] if n['tag'] in ['scalar','variant','option_none','option_some'] else n['payload']
                return (mt['owner'].encode('utf8'),self.m['type_ranks'][n['key']],mt['name'].encode('utf8'),base,tuple(ranks[c] for c in n['children']))
            for i in sorted((i for i,n in enumerate(self.nodes) if n['height']==height),key=key):ranks[i]=len(ordered);ordered.append(i)
        nodes=[]
        for i in ordered:
            n=self.nodes[i];tag=n['tag'];edges=[str(ranks[c]) for c in n['children']];payload=n['payload']
            if tag=='scalar':
                st=self.m['structural_binding_registration']['scalar_bodies'][n['kind']]
                value=b64(payload) if n['kind']=='bytes' else str(payload) if n['kind'] in ['u8','u32','u64','i32','i64','timestamp_i64','duration_i64'] else payload
                body={'scalar':{st:value}}
            elif tag=='record':body={'record':[dict(field=name,value=rank) for name,rank in zip(payload,edges)]}
            elif tag=='variant':body={'variant':dict(case=payload,payload=edges[0])}
            elif tag=='map':body={'map':[dict(key=edges[k],value=edges[k+1]) for k in range(0,len(edges),2)]}
            elif tag=='option_none':body={tag:None}
            elif tag=='option_some':body={tag:edges[0]}
            else:body={tag:edges}
            nodes.append(dict(type=nominal(n['key']),body=body))
        return dict(nodes=nodes,roots=[str(ranks[root])])

def strict_compare(source,want,got):
    # Required roster is rebuilt from SOURCE, even if want and got suffer the
    # same omission. Proposed views are NEVER the denominator.
    builder=Builder();reference=builder.finish(builder.build('RawValidationSubject',source))
    for label,view in [('Expected',want),('Actual',got)]:
        validate_graph(view)
        assert type(view) is dict and set(view)=={'nodes','roots'},(label,'carrier fields')
        assert len(view['nodes'])==len(reference['nodes']),(label,'required node roster')
        for rank,(required,observed) in enumerate(zip(reference['nodes'],view['nodes'])):
            assert set(observed)=={'type','body'},(label,rank,'closed DagNode fields')
            assert required['type']==observed.get('type'),(label,rank,'type',required['type'],observed.get('type'))
            assert required['body']==observed.get('body'),(label,rank,'body')
        assert reference['roots']==view['roots'],(label,'required roots')
    edges=sum(len(n['children']) for n in builder.nodes)
    return dict(required_nodes=len(reference['nodes']),compared_nodes=len(reference['nodes']),required_edges=edges,compared_edges=edges,required_roots=1,compared_roots=1,required_ranks=len(reference['nodes']),compared_ranks=len(reference['nodes']),sites=sorted(builder.sites),missing=0,unexpected=0,uncompared=0)

def integer(token,bits=64,signed=False):
    assert type(token) is str and re.fullmatch(r'-?(0|[1-9][0-9]*)',token) and token!='-0'
    value=int(token);assert -(2**(bits-1))<=value<2**(bits-1) if signed else 0<=value<2**bits
    return value

def validate_graph(view):
    """Independent typed decoder, canonical ranks/minimality and G closure.

    Decodes byte payloads and mathematical numerics. No JSON-text-only,
    digest-only, array-count-only or Expected-supplied roster admission.
    """
    assert type(view) is dict and set(view)=={'nodes','roots'}
    assert type(view['nodes']) is list and len(view['nodes'])<=1_048_576
    d=Builder();by_nominal={(nominal(k)['owner'],nominal(k)['name']):k for k in d.bindings};unique=set();previous=None;total_edges=0
    for rank,node in enumerate(view['nodes']):
        assert type(node) is dict and set(node)=={'type','body'}
        mt=node['type'];assert type(mt) is dict and set(mt)=={'owner','name'}
        key=by_nominal[(mt['owner'],mt['name'])];parent,path,desc=d.descriptor(key);kind=desc['kind']
        body=node['body'];assert type(body) is dict and len(body)==1;tag,payload=next(iter(body.items()));children=[];semantic=None;base=None;atom=None
        def edge(token,part,descriptor):
            target=integer(token);assert target<rank
            expected=d.child(parent,path,part,descriptor);assert d.nodes[target]['key']==expected
            children.append(target);return target
        scalar=d.m['structural_binding_registration']['scalar_bodies'].get(kind)
        if scalar:
            assert tag=='scalar' and type(payload) is dict and set(payload)=={scalar};value=payload[scalar]
            if kind in ['unit','null']:assert value is None;semantic=None
            elif kind=='bool':assert type(value) is bool;semantic=value
            elif kind=='bytes':
                assert type(value) is str and re.fullmatch(r'[A-Za-z0-9_-]*',value)
                semantic=base64.urlsafe_b64decode(value+'='*((-len(value))%4));assert b64(semantic)==value
                assert len(semantic)>=desc.get('minimum_bytes',0) and len(semantic)<=desc.get('maximum_bytes',134_217_728)
                if desc.get('utf8'):semantic.decode('utf8')
            elif kind=='nfc_text':assert type(value) is str and unicodedata.normalize('NFC',value)==value;semantic=value.encode('utf8')
            elif kind=='decimal':
                assert type(value) is str and re.fullmatch(r'-?(0|[1-9][0-9]*)(\.[0-9]*[1-9])?',value) and value!='-0';semantic=decimal.Decimal(value)
            else:semantic=integer(value,8 if kind=='u8' else 32 if kind in ['u32','i32'] else 64,kind in ['i32','i64','timestamp_i64','duration_i64'])
            atom=(scalar,semantic);base=semantic
        elif kind=='record':
            assert tag=='record' and type(payload) is list and len(payload)==len(desc['fields'])
            for row,(field,ch) in zip(payload,desc['fields']):
                assert type(row) is dict and set(row)=={'field','value'} and row['field']==field
                edge(row['value'],['field',field],ch)
            base=tuple(n for n,_ in desc['fields']);atom=base
        elif kind=='enum':
            assert tag=='variant' and type(payload) is dict and set(payload)=={'case','payload'}
            names=[n for n,_ in desc['variants']];case=payload['case'];semantic=names.index(case);base=semantic;atom=case
            edge(payload['payload'],['case',case],dict(desc['variants'])[case])
        elif kind=='option':
            assert tag in ['option_none','option_some'];semantic=0 if tag=='option_none' else 1;base=semantic;atom=tag
            if tag=='option_none':assert payload is None
            else:edge(payload,['some',None],desc['item'])
        elif kind in ['sequence','set']:
            assert tag==kind and type(payload) is list and len(payload)>=desc.get('minimum',0)
            for token in payload:edge(token,['member' if kind=='set' else 'item',None],desc['item'])
            if kind=='set':assert all(d.semantic_order(a,b)<0 for a,b in zip(children,children[1:]))
            base=len(children);atom=base
        elif kind=='map':
            assert tag=='map' and type(payload) is list
            keys=[]
            for row in payload:
                assert type(row) is dict and set(row)=={'key','value'}
                keys.append(edge(row['key'],['key',None],desc['key']));edge(row['value'],['value',None],desc['value'])
            assert all(d.semantic_order(a,b)<0 for a,b in zip(keys,keys[1:]));base=len(keys);atom=base
        else:raise AssertionError(('not input value',key,kind))
        token=(key,tag,atom,tuple(children));assert token not in unique,('duplicate equal typed node',rank);unique.add(token)
        height=1+max((d.nodes[c]['height'] for c in children),default=-1);assert height<=96
        ordering=(height,mt['owner'].encode('utf8'),d.m['type_ranks'][key],mt['name'].encode('utf8'),base,tuple(children))
        assert previous is None or previous<ordering,('noncanonical rank',rank);previous=ordering
        d.nodes.append(dict(key=key,tag=tag,semantic=semantic,children=children,height=height,payload=atom,kind=kind))
        total_edges+=len(children);assert total_edges<=262_144
    assert type(view['roots']) is list and len(view['roots'])==1
    root=integer(view['roots'][0]);assert root<len(d.nodes) and d.nodes[root]['key']=='RawValidationSubject'
    reachable=set();todo=[root]
    while todo:
        n=todo.pop()
        if n not in reachable:reachable.add(n);todo.extend(d.nodes[n]['children'])
    assert reachable==set(range(len(d.nodes)))
    return d

def first():
    source=next(v for v in RAW.vectors() if v['case_id']=='c2r1/declaration_bundle/input-zero')
    declared=subject(source,empty_context_with_actual_modules());builder=Builder();want=builder.finish(builder.build('RawValidationSubject',declared))
    actual=read('target/'+P+'first-actual.ace1')
    result=strict_compare(declared,want,actual)
    raw=canonical_bytes(want);assert raw==(R/'target'/(P+'first-actual.ace1')).read_bytes(), 'canonical bytes differ'
    (R/'target'/(P+'first-expected.ace1')).write_bytes(raw)
    return dict(outcome='FIRST PRODUCTION INPUT-G MATCH; NOT FULL DOMAIN CLOSURE',coverage=result,canonical_bytes=len(raw),registration=audit_registration(),raw_full_field=False,positive_root_export=False)
def check_case(index):
    case=read('spec/conformance/production-input-graph/cases.json')[index];declared=declared_case(case)
    b=Builder();wanted=b.finish(b.build('RawValidationSubject',declared));actual=read('target/'+P+f'case-{index:03}.ace1')
    result=strict_compare(declared,wanted,actual)
    assert canonical_bytes(wanted)==(R/'target'/(P+f'case-{index:03}.ace1')).read_bytes()
    return dict(case_id=case['case_id'],coverage=result,raw_full_field=False,positive_root_export=False)

def body_field_roster(graph):
    rows=[]
    def walk(v,path):
        if type(v) is dict:
            for k,ch in v.items():rows.append(path+(k,));walk(ch,path+(k,))
        elif type(v) is list:
            for i,ch in enumerate(v):walk(ch,path+(i,))
    for i,n in enumerate(graph['nodes']):walk(n['body'],(i,'body'))
    return rows

def source_association_roster(source):
    """Required semantic source links, enumerated from complete source inputs.

    Each row names a source path and the corresponding canonical G node. Its
    full node/subtree is compared by strict_compare, not accepted by hash.
    """
    paths=[['owner_compiler',k] for k in source['owner_compiler']]
    paths += [['input','candidate'],['input','origin','path'],['input','physical']]
    binding=['input','origin','document_binding']
    paths += [binding+[k] for k in source['input']['origin']['document_binding']]
    paths += [['context','contents',k] for k in source['context']['contents']]
    paths += [['context','roles']]
    compiler=source['owner_compiler'];support=compiler['support_manifest']
    paths += [['owner_compiler','specifications',i] for i in range(len(compiler['specifications']))]
    paths += [['owner_compiler','support_manifest','normative_sources',i] for i in range(len(support['normative_sources']))]
    paths += [['owner_compiler','support_manifest','closed_tables',k] for k in support['closed_tables']]
    paths += [['owner_compiler','support_manifest','semantic_roles',i] for i in range(len(support['semantic_roles']))]
    paths += [['context','contents','relation_snapshots',k] for k in source['context']['contents']['relation_snapshots']]
    return paths

def association_nodes(graph,paths,source):
    rows=[]
    for path in paths:
        rank=int(graph['roots'][0]);cursor=source
        for step in path:
            body=graph['nodes'][rank]['body']
            if type(step) is int:
                if 'set' in body:
                    # Source set storage order is not a graph position. These
                    # association roots are SpecificationContentIdentity sets;
                    # locate by the actual unique specification identity.
                    identity=cursor[step]['specification_id'];matches=[]
                    for token in body['set']:
                        record=graph['nodes'][int(token)]['body']['record']
                        target=int(next(r['value'] for r in record if r['field']=='specification_id'))
                        value=next(iter(graph['nodes'][target]['body']['scalar'].values()))
                        if value==identity:matches.append(int(token))
                    assert len(matches)==1;rank=matches[0]
                else:rank=int(body['sequence'][step])
            else:
                rank=int(next(r['value'] for r in body['record'] if r['field']==step))
            cursor=cursor[step]
        rows.append(dict(source_path=path,dag_rank=rank,type=graph['nodes'][rank]['type']))
    return rows

def mutation_gate(source,want,actual):
    cases=[]
    def attack(name,edit,both=False):
        w=copy.deepcopy(want);a=copy.deepcopy(actual)
        edit(a)
        if both:edit(w)
        try:strict_compare(source,w,a)
        except (AssertionError,KeyError,ValueError,TypeError):cases.append(dict(case_id='production-input-g/mutation-'+name,rejected=True,both_sides=both));return
        raise AssertionError(('mutation escaped',name))
    scalar=next(i for i,n in enumerate(want['nodes']) if 'scalar' in n['body'])
    blob=next(i for i,n in enumerate(want['nodes']) if len(n['body'].get('scalar',{}).get('bytes',''))>100_000)
    record_node=next(i for i,n in enumerate(want['nodes']) if n['body'].get('record'))
    map_node=next(i for i,n in enumerate(want['nodes']) if n['body'].get('map'))
    set_node=next(i for i,n in enumerate(want['nodes']) if len(n['body'].get('set',[]))>1)
    variant_node=next(i for i,n in enumerate(want['nodes']) if 'variant' in n['body'])
    seq_node=next(i for i,n in enumerate(want['nodes']) if len(n['body'].get('sequence',[]))>1)
    def node(g,i):return g['nodes'][i]
    def put(field,value):return lambda g:g.__setitem__(field,value)
    edits=[
        ('missing-node',lambda g:g['nodes'].pop()),('missing-node-both',lambda g:g['nodes'].pop(),True),
        ('missing-nodes-field',lambda g:g.pop('nodes')),('missing-nodes-field-both',lambda g:g.pop('nodes'),True),
        ('wrong-owner',lambda g:node(g,scalar)['type'].__setitem__('owner','wrong.owner@1')),
        ('wrong-owner-both',lambda g:node(g,scalar)['type'].__setitem__('owner','wrong.owner@1'),True),
        ('wrong-nominal-type',lambda g:node(g,record_node).__setitem__('type',copy.deepcopy(want['nodes'][scalar]['type']))),
        ('wrong-structural-site',lambda g:node(g,map_node)['type'].__setitem__('name','StructuralValue0317')),
        ('missing-type',lambda g:node(g,scalar).pop('type')),('missing-type-both',lambda g:node(g,scalar).pop('type'),True),
        ('extra-node-field',lambda g:node(g,scalar).__setitem__('trusted',True)),
        ('missing-body',lambda g:node(g,record_node).pop('body')),('missing-body-both',lambda g:node(g,record_node).pop('body'),True),
        ('wrong-body-case',lambda g:node(g,record_node).__setitem__('body',{'scalar':{'unit':None}})),
        ('unknown-body-case',lambda g:node(g,record_node).__setitem__('body',{'witness':None})),
        ('missing-record-field',lambda g:node(g,record_node)['body']['record'].pop()),
        ('missing-record-field-both',lambda g:node(g,record_node)['body']['record'].pop(),True),
        ('extra-record-field',lambda g:node(g,record_node)['body']['record'].append({'field':'future','value':'0'})),
        ('wrong-record-field-token',lambda g:node(g,record_node)['body']['record'][0].__setitem__('field','future')),
        ('wrong-child-edge',lambda g:node(g,record_node)['body']['record'][0].__setitem__('value','0')),
        ('missing-child-both',lambda g:node(g,record_node)['body']['record'][0].pop('value'),True),
        ('forward-child',lambda g:node(g,record_node)['body']['record'][0].__setitem__('value',str(record_node))),
        ('dangling-child',lambda g:node(g,record_node)['body']['record'][0].__setitem__('value',str(len(g['nodes'])))),
        ('child-rank-overflow',lambda g:node(g,record_node)['body']['record'][0].__setitem__('value',str(2**64))),
        ('noncanonical-rank-token',lambda g:node(g,record_node)['body']['record'][0].__setitem__('value','00')),
        ('wrong-map-key',lambda g:node(g,map_node)['body']['map'][0].__setitem__('key','0')),
        ('wrong-map-value',lambda g:node(g,map_node)['body']['map'][0].__setitem__('value','0')),
        ('missing-map-value-both',lambda g:node(g,map_node)['body']['map'][0].pop('value'),True),
        ('duplicate-map-key',lambda g:node(g,map_node)['body']['map'].append(copy.deepcopy(node(g,map_node)['body']['map'][0]))),
        ('wrong-set-order',lambda g:node(g,set_node)['body']['set'].reverse()),
        ('duplicate-set-member',lambda g:node(g,set_node)['body']['set'].append(node(g,set_node)['body']['set'][0])),
        ('missing-set-member-both',lambda g:node(g,set_node)['body']['set'].pop(),True),
        ('wrong-sequence-order',lambda g:node(g,seq_node)['body']['sequence'].reverse()),
        ('missing-sequence-child-both',lambda g:node(g,seq_node)['body']['sequence'].pop(),True),
        ('unknown-variant',lambda g:node(g,variant_node)['body']['variant'].__setitem__('case','future')),
        ('missing-variant-child',lambda g:node(g,variant_node)['body']['variant'].pop('payload')),
        ('compiler-source-byte',lambda g:node(g,blob)['body']['scalar'].__setitem__('bytes','QQ'+node(g,blob)['body']['scalar']['bytes'][2:])),
        ('compiler-source-byte-both',lambda g:node(g,blob)['body']['scalar'].__setitem__('bytes','QQ'+node(g,blob)['body']['scalar']['bytes'][2:]),True),
        ('missing-root',put('roots',[])),('missing-root-both',put('roots',[]),True),
        ('wrong-root',put('roots',['0'])),('root-rank-overflow',put('roots',[str(2**64)])),
        ('extra-root',lambda g:g['roots'].append(g['roots'][0])),('missing-roots-field-both',lambda g:g.pop('roots'),True),
        ('duplicate-equal-node',lambda g:g['nodes'].insert(1,copy.deepcopy(g['nodes'][0]))),
        ('alternate-numbering',lambda g:g['nodes'].reverse()),
        ('g-to-w',lambda g:node(g,record_node)['body']['record'][0].__setitem__('value',{'local':'0'})),
        ('w-to-g-use',lambda g:node(g,record_node)['body']['record'][0].__setitem__('value',{'input':'0'})),
        ('cross-edition-node',lambda g:node(g,scalar)['type'].__setitem__('owner','agentflow.computation.proof-report.model@6')),
        ('undeclared-environment',lambda g:g.__setitem__('witness_graph',{'nodes':[],'roots':[]})),
        ('forged-verified',lambda g:g.__setitem__('verified',True)),('forged-positive-root-export',lambda g:g.__setitem__('positive_root_export',True)),
        ('ignored-resource-result',lambda g:g.__setitem__('resource_limit_ignored',True)),
    ]
    for row in edits:attack(*row)
    # Strong both-side omission: generate another internally canonical graph
    # from a changed complete declaration, with repaired ranks/reachability.
    # The denominator remains the ORIGINAL immutable declared source.
    altered=copy.deepcopy(source)
    package=altered['input']['origin']['document_binding']['source_package']
    for owner in package['owners']:
        _,container=next(iter(owner['container'].items()))
        entries=next(v for k,v in container.items() if k in ['declarations','imports','sources'])
        for entry in entries:
            doc=entry['document'].get('declared_native_document')
            if doc and len(doc['declarations'])>1:doc['declarations'].pop()
    b=Builder();omitted=b.finish(b.build('RawValidationSubject',altered));validate_graph(omitted)
    try:strict_compare(source,omitted,copy.deepcopy(omitted))
    except AssertionError:cases.append(dict(case_id='production-input-g/mutation-both-omit-subtree-valid-reindex',rejected=True,both_sides=True))
    else:raise AssertionError('both-side source-derived roster omission escaped')
    def rebuilt(name,edit):
        altered=copy.deepcopy(source);edit(altered);b=Builder();other=b.finish(b.build('RawValidationSubject',altered));validate_graph(other)
        try:strict_compare(source,other,copy.deepcopy(other))
        except AssertionError:cases.append(dict(case_id='production-input-g/mutation-'+name,rejected=True,both_sides=True,internally_valid_reindexed_graph=True));return
        raise AssertionError(('both-side full source association escaped',name))
    def binding(s):return s['input']['origin']['document_binding']
    rebuilt('both-wrong-owner',lambda s:binding(s)['source_owner'].__setitem__('owner_key',list(b'other')))
    rebuilt('both-wrong-revision',lambda s:binding(s)['source_owner'].__setitem__('revision_key',list(b'other')))
    rebuilt('both-wrong-document-slot',lambda s:binding(s)['occurrence_key'].__setitem__('entry_ordinal',1))
    rebuilt('both-missing-compiler-specification',lambda s:s['owner_compiler']['specifications'].pop())
    rebuilt('both-missing-context-module',lambda s:s['context']['contents']['compiled_modules'].pop())
    assert len(cases)>=40
    return cases

def all_cases():
    cases=read('spec/conformance/production-input-graph/cases.json');manifest=read('spec/conformance/production-input-graph/manifest.json')
    assert [c['case_id'] for c in cases]==manifest['case_ids']
    actual_rows=[r for shard in range(4) for r in read('target/'+P+f'shard-{shard}.json')]
    actual_by_id={r['case_id']:r for r in actual_rows};assert len(actual_by_id)==len(actual_rows)==len(cases)
    assert set(actual_by_id)==set(manifest['case_ids'])
    reports=[];all_sites=set();types={};field_variants={};site_cases={};totals=dict(nodes=0,body_fields=0,edges=0,roots=0,ranks=0,source_associations=0)
    for index,case in enumerate(cases):
        receipt=actual_by_id[case['case_id']];assert receipt['input_case']==case,('stale input execution',case['case_id'])
        declared=declared_case(case);builder=Builder();want=builder.finish(builder.build('RawValidationSubject',declared))
        actual_bytes=(R/receipt['graph']).read_bytes();actual=json.loads(actual_bytes);coverage=strict_compare(declared,want,actual)
        assert receipt['sites']==coverage['sites'] and receipt['nodes']==coverage['required_nodes']
        assert str(receipt['subject_node_rank'])==actual['roots'][0]
        assert receipt['positive_root_export'] is False and receipt['complete_raw_fact'] is False
        assert canonical_bytes(want)==actual_bytes
        actual_decoded=validate_graph(actual)
        for n in actual_decoded.nodes:
            types.setdefault(n['key'],set()).add(case['case_id'])
            required=[]
            if n['tag']=='record':required=[(n['key'],'field',field) for field in n['payload']]
            elif n['tag']=='variant':required=[(n['key'],'variant',n['payload'])]
            for row in required:field_variants.setdefault(row,set()).add(case['case_id'])
        for site in coverage['sites']:site_cases.setdefault(site,[]).append(case['case_id'])
        fields=len(body_field_roster(want));paths=source_association_roster(declared)
        associations_expected=association_nodes(want,paths,declared);assert associations_expected==association_nodes(actual,paths,declared)
        associations=len(paths)
        totals['nodes']+=len(want['nodes']);totals['ranks']+=len(want['nodes']);totals['body_fields']+=fields;totals['edges']+=coverage['required_edges'];totals['roots']+=1;totals['source_associations']+=associations
        all_sites.update(coverage['sites']);reports.append(dict(case_id=case['case_id'],graph=receipt['graph'],actual_raw_sha256=hashlib.sha256(actual_bytes).hexdigest(),**coverage,required_body_fields=fields,compared_body_fields=fields,required_source_associations=associations,compared_source_associations=associations,source_associations=associations_expected))
        if case.get('equal_graph_to'):
            peer=actual_by_id[case['equal_graph_to']];assert actual==read(peer['graph']),('host map order affected G',case['case_id'])
        if case.get('different_graph_from'):
            peer=actual_by_id[case['different_graph_from']];assert actual!=read(peer['graph']),('source identity erased',case['case_id'])
            peer_source=declared_case(peer['input_case'])
            assert declared['owner_compiler']==peer_source['owner_compiler'] and declared['context']==peer_source['context']
            assert declared['input']['candidate']==peer_source['input']['candidate']
            if case['slot']==2:
                # Independently supplied equal-content document slots remain
                # distinct occurrences in one complete declared package.
                assert declared['input']['origin']['document_binding']['actual_document']==peer_source['input']['origin']['document_binding']['actual_document']
        print(json.dumps(dict(case_id=case['case_id'],nodes=len(want['nodes']),sites=len(coverage['sites']),matched=True)),flush=True,file=sys.stderr)
    required_types,required_fields=required_value_shapes()
    assert required_types==set(types),('runtime type closure',sorted(required_types-set(types)),sorted(set(types)-required_types))
    assert required_fields==set(field_variants),('runtime field/variant closure',sorted(required_fields-set(field_variants)),sorted(set(field_variants)-required_fields))
    rows=catalogue()['structural_binding_registration']['sites']
    inactive={'StructuralValue0009','StructuralValue0010','StructuralValue0011','StructuralValue0061'}
    applicable={r['alias'] for r in rows if r['definition']['eligibility']=='g_input'}-inactive
    assert all_sites==applicable,('runtime site closure',sorted(applicable-all_sites),sorted(all_sites-applicable))
    ledger=dict(types=[dict(type=k,cases=sorted(v)) for k,v in sorted(types.items())],field_variants=[dict(type=k[0],kind=k[1],member=k[2],cases=sorted(v)) for k,v in sorted(field_variants.items())],sites=site_cases)
    (R/'target'/(P+'runtime-coverage-ledger.json')).write_text(json.dumps(ledger,indent=2)+'\n',encoding='utf8')
    result=dict(outcome='PRODUCTION INPUT-G DIFFERENTIAL RESULTS; DOMAIN CLOSURE SEPARATE',case_ids=len(cases),required=totals,compared=totals,missing=0,unexpected=0,uncompared=0,executed_sites=sorted(all_sites),required_types=len(required_types),compared_types=len(types),required_field_variants=len(required_fields),compared_field_variants=len(field_variants),cases=reports,raw_full_field=False,positive_root_export=False)
    (R/'target'/(P+'input-g-oracle-results.json')).write_text(json.dumps(result,indent=2)+'\n',encoding='utf8');return result

def mutations():
    case=read('spec/conformance/production-input-graph/cases.json')[1];source=declared_case(case);b=Builder();want=b.finish(b.build('RawValidationSubject',source));actual=read('target/'+P+'case-001.ace1');strict_compare(source,want,actual)
    rows=mutation_gate(source,want,actual);return dict(mutations=len(rows),unexpected_accepted=0,cases=rows)

def required_value_shapes():
    """Independent descriptor work-list, not an Actual/Expected node census."""
    b=Builder();pending=['RawValidationSubject'];seen=set();roster=set()
    from structural_binding_schema import descriptor_children
    while pending:
        key=pending.pop()
        if key in seen:continue
        seen.add(key);parent,path,d=b.descriptor(key)
        if key=='CompleteCompiler':continue
        if d['kind']=='record':roster.update((key,'field',name) for name,_ in d['fields'])
        if d['kind']=='enum':roster.update((key,'variant',name) for name,_ in d['variants'])
        for addition,ch in descriptor_children(d):pending.append(b.child(parent,path,addition[0],ch))
    # The compiler is the exact implementation-owned Profile A, not a
    # caller-selectable arbitrary FullSupportManifest. Add its real fixed
    # fields/cases from independent normative source input, not an Actual.
    root=b.build('CompleteCompiler',compiler_source())
    for n in b.nodes:
        if n['tag']=='record':roster.update((n['key'],'field',field) for field in n['payload'])
        if n['tag']=='variant':roster.add((n['key'],'variant',n['payload']))
    return seen|{n['key'] for n in b.nodes},roster

def planned_coverage():
    required_types,required_fields=required_value_shapes();types=set();fields=set();sites=set();cases=read('spec/conformance/production-input-graph/cases.json')
    # Root selection/owner changes cannot introduce a new source declaration
    # type; this read-only planning check still visits every declared case.
    for case in cases:
        b=Builder();b.build('RawValidationSubject',declared_case(case));sites.update(b.sites)
        for n in b.nodes:
            types.add(n['key'])
            if n['tag']=='record':fields.update((n['key'],'field',field) for field in n['payload'])
            if n['tag']=='variant':fields.add((n['key'],'variant',n['payload']))
    result=dict(planning_only=True,production_credit=0,required_types=len(required_types),planned_types=len(types),missing_types=sorted(required_types-types),required_field_variants=len(required_fields),planned_field_variants=len(fields),missing_field_variants=sorted(required_fields-fields),planned_sites=len(sites))
    return result
if __name__=='__main__':
    result=planned_coverage() if '--plan-coverage' in sys.argv else all_cases() if '--all' in sys.argv else mutations() if '--mutations' in sys.argv else check_case(int(sys.argv[2])) if len(sys.argv)>2 and sys.argv[1]=='--case' else first();print(json.dumps(result,indent=2))
