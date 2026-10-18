"""Input-only registration fixtures. Numeric DAG ranks below are hand-written.

No real complete Raw input is captured or assigned ranks. No Rust Actual is read.
"""
import copy, importlib.util, json
from pathlib import Path
from structural_binding_schema import ROOT, nominal, descriptor_children
from structural_binding_registration_data import *

D=ROOT/'spec/conformance/g01-structural-binding-staging'
def js(name,value):
    p=D/name;p.parent.mkdir(parents=True,exist_ok=True);raw=(json.dumps(value,indent=2,sort_keys=True,ensure_ascii=False)+'\n').encode()
    if not p.exists() or p.read_bytes()!=raw:p.write_bytes(raw)
def oracle():
    s=importlib.util.spec_from_file_location('structural_independent_reference',ROOT/'tools/oracle/structural_binding_registration.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m

def sample(m,d,route=(),stack=()):
    t=m['types']
    if 'ref' in d:
        n=d['ref']
        if not route:
            if n in ['ModelType','ModelRef']:return nominal('$Text',MODEL)
            if n=='StructuralSourceType':return nominal('$Text','agentflow.computation.proof-report.model@6')
            if n=='StructuralEditionTuple':return dict(analysis=OWNER,model=MODEL,report=SHAPE,obligation=OBLIGATION)
            if n in ['RawUtf8Bytes','DocumentSourceKey']:return [120]
            if n in ['RawContextPartition','NativeGuardContextPartition']:return [['analysis','canonical_validation']]
            if n=='ContextRolePartition':return [['analysis','replacement_checked'],['old_checked']]
        if n in stack and (route or t[n].get('kind') not in ['enum','sequence','set','map','option']):raise ValueError(('required recursive sample',n,route))
        if len(stack)>96:raise ValueError(('unbounded required sample',n))
        return sample(m,t[n],route,stack+(n,))
    k=d['kind'];head=route[0] if route else None;tail=route[1:] if route else ()
    if k=='record':return {n:sample(m,c,tail if head==['field',n] else (),stack) for n,c in d['fields']}
    if k=='enum':
        tag=head[1] if head and head[0]=='case' else d['variants'][0][0];child=dict(d['variants'])[tag]
        if all(c.get('kind')=='unit' for _,c in d['variants']):return tag
        return {tag:sample(m,child,tail if head else (),stack)}
    if k in ['sequence','set']:
        count=max(d.get('minimum',0),1 if route else 0)
        return [sample(m,d['item'],tail if route else (),stack) for _ in range(count)]
    if k=='map':
        if not route:return []
        return [[sample(m,d['key'],tail if head[0]=='key' else (),stack),sample(m,d['value'],tail if head[0]=='value' else (),stack)]]
    if k=='option':return {'some':sample(m,d['item'],tail,stack)} if route else {'none':None}
    if k in ['unit','null']:return None
    if k=='nfc_text':return 'registered.sample@1'
    if k=='bytes':return [0]*d.get('length',0)
    if k=='bool':return False
    if k=='decimal':return '0'
    return 0

def payload_at(m,d,value,route):
    while 'ref' in d:d=m['types'][d['ref']]
    if not route:return d,value
    (kind,name),*rest=route;k=d['kind']
    if k=='record':return payload_at(m,dict(d['fields'])[name],value[name],rest)
    if k=='enum':return payload_at(m,dict(d['variants'])[name],value[name],rest)
    if k in ['sequence','set']:return payload_at(m,d['item'],value[0],rest)
    if k=='map':return payload_at(m,d['key' if kind=='key' else 'value'],value[0][0 if kind=='key' else 1],rest)
    if k=='option':return payload_at(m,d['item'],value['some'],rest)
    raise ValueError((route,d))

def manual_fixtures(m):
    sites=m['structural_binding_registration']['sites']
    def alias(parent,path):return next(r['alias'] for r in sites if r['source_catalogue_key']==parent and r['source_path']==path)
    def n(ty,tag,value):return {'type':nominal(ty,MODEL),'body':{tag:value}}
    def record(ty,items):return n(ty,'record',[dict(field=k,value=v) for k,v in items])
    def variant(ty,case,rank):return n(ty,'variant',dict(case=case,payload=rank))
    def source(ty,value):return dict(type=nominal(ty,MODEL),value=value)
    src={};seeds={}
    def add(name,gs,gnodes,groots,ws=None,wnodes=None,wroots=None):
        src[name]=dict(g=gs,**(dict(w=ws) if ws is not None else {}));seeds[name]=dict(g=dict(nodes=gnodes,roots=groots),**(dict(w=dict(nodes=wnodes,roots=wroots)) if ws is not None else {}))
    add('uint64',[source('$U64',1)],[n('$U64','scalar',{'uint64':1})],[0])
    add('numeric-order',[source('$U64',2),source('$U64',10)],[n('$U64','scalar',{'uint64':2}),n('$U64','scalar',{'uint64':10})],[0,1])
    add('equal-roots',[source('$U64',1),source('$U64',1)],[n('$U64','scalar',{'uint64':1})],[0,0])
    # Float is not reachable in this Raw profile (the native guard constants
    # have no Float64 case); do not add a fake production source to exercise it.
    add('ineligible-floats',[source('af-core::Float64',-1.0),source('af-core::Float64',1.0)],[n('af-core::Float64','scalar',{'float64':-1.0}),n('af-core::Float64','scalar',{'float64':1.0})],[0,1])
    add('ineligible-float-zero',[source('af-core::Float64',0.0)],[n('af-core::Float64','scalar',{'float64':-0.0})],[0])
    add('exact-decimals',[source('Raw(af-core::Decimal)','2.0'),source('Raw(af-core::Decimal)','10.000')],[n('Raw(af-core::Decimal)','scalar',{'decimal':'2'}),n('Raw(af-core::Decimal)','scalar',{'decimal':'10'})],[0,1])
    mapty=alias('ContractProofContextContents',[['field','compiled_modules']]);seqty=alias('CompiledModuleContents',[['field','types']])
    module=dict(specification_id='binding.alpha@1',types=[])
    mapnodes=[n('SpecificationId','scalar',{'string':'binding.alpha@1'}),n(seqty,'sequence',[]),record('CompiledModuleContents',[('specification_id',0),('types',1)]),n(mapty,'map',[dict(key=0,value=2)])]
    add('compiled-map',[source(mapty,[['binding.alpha@1',module]])],mapnodes,[3])
    add('empty-map',[source(mapty,[])],[n(mapty,'map',[])],[0])
    setty=alias('ContractProofContextContents',[['field','recovery_operations'],['field','reconciliation_operations']]);memberty=alias('ContractProofContextContents',[['field','recovery_operations'],['field','reconciliation_operations'],['member',None]])
    setnodes=[n('$Text','scalar',{'string':'a'}),n('$Text','scalar',{'string':'b'}),n('$Text','scalar',{'string':'c'}),record(memberty,[('catalog',0),('name',1)]),record(memberty,[('catalog',0),('name',2)]),n(setty,'set',[3,4])]
    add('set-full-members',[source(setty,[dict(catalog='a',name='b'),dict(catalog='a',name='c')])],setnodes,[5])
    add('empty-map-versus-set',[source(mapty,[]),source(setty,[])],[n(mapty,'map',[]),n(setty,'set',[])],[0,1])
    pathty=alias('RawInputOrigin',[['field','path']]);fieldty=alias('RawDocumentMember',[['case','field']])
    pathnodes=[n('RawUtf8Bytes','scalar',{'bytes':[120]}),record(fieldty,[('name',0)]),variant('RawDocumentMember','field',1),n(pathty,'sequence',[2]),n(pathty,'sequence',[2,2])]
    add('shared-child',[source(pathty,[{'field':{'name':[120]}}]),source(pathty,[{'field':{'name':[120]}},{'field':{'name':[120]}}])],pathnodes,[3,4])
    # Complete, manually linked document occurrence subgraph. Explicit fixed
    # ranks below are examples, not a generalized construction algorithm.
    emptydecl=alias('RawDocument',[['case','declared_native_document'],['field','declarations']])
    docpayload=alias('RawDocument',[['case','declared_native_document']])
    declseq=alias('DocumentOwnerContainer',[['case','declaration_bundle'],['field','declarations']])
    ownerpayload=alias('DocumentOwnerContainer',[['case','declaration_bundle']])
    ownersseq=alias('SourcePackage',[['field','owners']])
    domain=dict(namespace=[120],domain_key=[120]);anchor=dict(domain=domain,owner_key=[120],revision_key=[120]);doc={'declared_native_document':{'declarations':[]}}
    entry=dict(declaration_name=[120],document=doc);container={'declaration_bundle':dict(bundle_name=[120],declarations=[entry])};package=dict(domain=domain,owners=[dict(anchor=anchor,container=container)])
    binding=dict(source_package=package,source_owner=anchor,occurrence_key=dict(entry_ordinal=0),actual_document=doc)
    document_nodes=[n('$U64','scalar',{'uint64':0}),n('DocumentSourceKey','scalar',{'bytes':[120]}),n(emptydecl,'sequence',[]),record('DocumentSourceDomain',[('namespace',1),('domain_key',1)]),record('DocumentOccurrenceKey',[('entry_ordinal',0)]),record(docpayload,[('declarations',2)]),variant('RawDocument','declared_native_document',5),record('DocumentOwnerAnchor',[('domain',3),('owner_key',1),('revision_key',1)]),record('DeclarationDocumentEntry',[('declaration_name',1),('document',6)]),n(declseq,'sequence',[8]),record(ownerpayload,[('bundle_name',1),('declarations',9)]),variant('DocumentOwnerContainer','declaration_bundle',10),record('DocumentOwnerRecord',[('anchor',7),('container',11)]),n(ownersseq,'sequence',[12]),record('SourcePackage',[('domain',3),('owners',13)]),record('DocumentOccurrenceBinding',[('source_package',14),('source_owner',7),('occurrence_key',4),('actual_document',6)])]
    wnodes=[n(pathty,'sequence',[]),record('RawInputOrigin',[('document_binding',{'input':15}),('path',{'local':0})])]
    add('w-input-reference',[source('DocumentOccurrenceBinding',binding)],document_nodes,[15],[source('RawInputOrigin',dict(document_binding=binding,path=[]))],wnodes,[1])
    option=alias('RelationValidationResult',[['case','invalid'],['field','codec_failure']]);unit=alias('RelationCodecFailure',[['case','non_finite_float']])
    add('w-option-none',[source('$U64',1)],[n('$U64','scalar',{'uint64':1})],[0],[source(option,{'none':None})],[n(option,'option_none',None)],[0])
    add('w-option-some',[source('$U64',1)],[n('$U64','scalar',{'uint64':1})],[0],[source(option,{'some':{'non_finite_float':None}})],[n(unit,'scalar',{'unit':None}),n('RelationCodecFailure','variant',dict(case='non_finite_float',payload={'local':0})),n(option,'option_some',{'local':1})],[2])
    # Same complete documents, different legal source-entry ordinals/names.
    entry2=dict(declaration_name=[121],document=doc);package2=dict(domain=domain,owners=[dict(anchor=anchor,container={'declaration_bundle':dict(bundle_name=[120],declarations=[entry,entry2])})])
    b0=dict(source_package=package2,source_owner=anchor,occurrence_key=dict(entry_ordinal=0),actual_document=doc);b1={**b0,'occurrence_key':dict(entry_ordinal=1)}
    two=[n('$U64','scalar',{'uint64':0}),n('$U64','scalar',{'uint64':1}),n('DocumentSourceKey','scalar',{'bytes':[120]}),n('DocumentSourceKey','scalar',{'bytes':[121]}),n(emptydecl,'sequence',[]),record('DocumentSourceDomain',[('namespace',2),('domain_key',2)]),record('DocumentOccurrenceKey',[('entry_ordinal',0)]),record('DocumentOccurrenceKey',[('entry_ordinal',1)]),record(docpayload,[('declarations',4)]),variant('RawDocument','declared_native_document',8),record('DocumentOwnerAnchor',[('domain',5),('owner_key',2),('revision_key',2)]),record('DeclarationDocumentEntry',[('declaration_name',2),('document',9)]),record('DeclarationDocumentEntry',[('declaration_name',3),('document',9)]),n(declseq,'sequence',[11,12]),record(ownerpayload,[('bundle_name',2),('declarations',13)]),variant('DocumentOwnerContainer','declaration_bundle',14),record('DocumentOwnerRecord',[('anchor',10),('container',15)]),n(ownersseq,'sequence',[16]),record('SourcePackage',[('domain',5),('owners',17)]),record('DocumentOccurrenceBinding',[('source_package',18),('source_owner',10),('occurrence_key',6),('actual_document',9)]),record('DocumentOccurrenceBinding',[('source_package',18),('source_owner',10),('occurrence_key',7),('actual_document',9)])]
    add('distinct-document-occurrences',[source('DocumentOccurrenceBinding',b0),source('DocumentOccurrenceBinding',b1)],two,[19,20])
    return src,seeds

def materialize(m):
    O=oracle();cases=[];coverage=[];reg=m['structural_binding_registration']
    sources,seeds=manual_fixtures(m);js('manual-source-fixtures.json',sources);js('manual-dag-inputs.json',seeds)
    def add(category,slug,inp,coverage_item=None):
        cid=PREFIX+slug;cases.append(dict(case_id=cid,category=category,input=copy.deepcopy(inp),expected=O.result(inp),execution_scope='registration_metadata_only',production_capture_case_ids=0))
        if coverage_item:coverage.append(dict(**coverage_item,case_id=cid))
        return cid
    for i,name in enumerate(reg['coverage_types']):
        d=m['types'][name]
        add('shape','shape-%04d'%i,dict(op='shape',type=name,value=sample(m,{'ref':name})))
        stack=[(d,[])]
        while stack:
            child,path=stack.pop()
            if 'ref' in child:continue
            if child.get('kind')=='record':
                for field,_ in child['fields']:
                    value=sample(m,{'ref':name},path);_,at=payload_at(m,d,value,path);at.pop(field)
                    item=dict(type=name,path=path,field=field,variant=None)
                    add('shape','field-%04d'%len(coverage),dict(op='shape',type=name,value=value),item)
                value=sample(m,{'ref':name},path);_,at=payload_at(m,d,value,path);at['unknown_extra']=None
                add('shape','extra-%04d'%len(cases),dict(op='shape',type=name,value=value))
            if child.get('kind')=='enum':
                for tag,_ in child['variants']:
                    route=path+[['case',tag]];value=sample(m,{'ref':name},route)
                    add('shape','variant-%04d'%len(coverage),dict(op='shape',type=name,value=value),dict(type=name,path=path,field=None,variant=tag))
                add('shape','unknown-case-%04d'%len(cases),dict(op='shape',type=name,value={'unknown_case':None}))
            stack.extend((c,path+p) for p,c in descriptor_children(child))
        if d.get('kind') in ['unit','null','bool','nfc_text','bytes','u32','u64','i64','i32','decimal','finite_f64']:
            add('shape','wrong-primitive-%04d'%i,dict(op='shape',type=name,value={'wrong':None}))
    for i,row in enumerate(reg['sites']):
        v=row['definition'];add('structural_site','site-%04d-valid'%i,dict(op='site',value=v))
        bad=copy.deepcopy(v);bad['value_type']=nominal('StructuralStage',MODEL)
        add('structural_site','site-%04d-wrong-type'%i,dict(op='site',value=bad))
        if v['children']:
            bad=copy.deepcopy(v);bad['children'].pop();add('structural_site','site-%04d-missing-child'%i,dict(op='site',value=bad))
    for i,v in enumerate(reg['reference_policies']):
        add('admission_negative','reference-%04d-valid'%i,dict(op='reference',value=v))
        bad=copy.deepcopy(v);bad['domain']='w_local';add('admission_negative','reference-%04d-wrong-arena'%i,dict(op='reference',value=bad))
    for op,values in [('site_registry',[r['definition'] for r in reg['sites']]),('reference_registry',reg['reference_policies']),('field_roster',O.field_roster(m))]:
        add('admission_negative',op+'-complete',dict(op=op,value=values))
        add('admission_negative',op+'-both-omit',dict(op=op,value=values[:-1]))
        extra=copy.deepcopy(values);extra.append(copy.deepcopy(values[0]));add('admission_negative',op+'-extra',dict(op=op,value=extra))
    stage=reg['stage_contract'];add('admission_negative','stage-complete',dict(op='stage',value=stage))
    for field in stage:
        v=copy.deepcopy(stage);v.pop(field);add('admission_negative','stage-missing-'+field,dict(op='stage',value=v))
    wrong=copy.deepcopy(stage);wrong['dependencies'].append(dict(source='raw_fact_capture',target='pre_capture_binding'));add('admission_negative','stage-cycle-backedge',dict(op='stage',value=wrong))
    wrong=copy.deepcopy(stage);wrong['binding_stage']='t4_normalization';add('admission_negative','stage-t4-cannot-own-binder',dict(op='stage',value=wrong))
    for p in ['raw_contract_validation','other_registered_profile','registration_subgraph']:
        add('capability_denial','production-'+p,dict(op='production',profile=p))
    add('capability_denial','caller-supported-flags-do-not-implement',dict(op='production',profile='raw_contract_validation',supported=True,source_assurance='native_associated',graph_checked=True))
    for r in RULES:add('rule','rule-'+r['rule_id'][-4:],dict(op='rule',rule_id=r['rule_id']))
    for fixture,proposal in seeds.items():add('dag_subgraph','dag-'+fixture,dict(op='dag',fixture=fixture,proposal=proposal))
    def mutation(fixture,slug,fn,joint=False):
        p=copy.deepcopy(seeds[fixture]);fn(p)
        inp=dict(op='joint_omission',fixture=fixture,expected_view=p,actual_view=copy.deepcopy(p)) if joint else dict(op='dag',fixture=fixture,proposal=p)
        add('dag_subgraph','dag-'+slug,inp)
    mutation('uint64','wrong-scalar',lambda p:p['g']['nodes'][0]['body']['scalar'].update(uint64=2))
    mutation('uint64','wrong-nominal',lambda p:p['g']['nodes'][0].update(type=nominal('$U32',MODEL)))
    mutation('uint64','wrong-owner',lambda p:p['g']['nodes'][0]['type'].update(owner='agentflow.computation.proof-report.model@6'))
    mutation('uint64','missing-node',lambda p:p['g']['nodes'].clear(),True)
    mutation('uint64','missing-root',lambda p:p['g']['roots'].clear(),True)
    mutation('uint64','duplicate-equal',lambda p:p['g']['nodes'].append(copy.deepcopy(p['g']['nodes'][0])))
    mutation('uint64','unreachable',lambda p:p['g']['nodes'].append(dict(type=nominal('$U64',MODEL),body={'scalar':{'uint64':2}})))
    mutation('uint64','resource-bound',lambda p:p['g']['nodes'].extend([copy.deepcopy(p['g']['nodes'][0])]*64))
    mutation('numeric-order','alternate-numbering',lambda p:p['g']['nodes'].reverse())
    mutation('numeric-order','root-order',lambda p:p['g']['roots'].reverse())
    mutation('ineligible-floats','ineligible-float-remains-refused',lambda p:p['g']['nodes'].pop(),True)
    mutation('compiled-map','map-missing-body',lambda p:p['g']['nodes'][3].update(body={'sequence':[]} ),True)
    mutation('compiled-map','map-missing-value',lambda p:p['g']['nodes'][3]['body']['map'][0].pop('value'),True)
    mutation('compiled-map','map-duplicate-key',lambda p:p['g']['nodes'][3]['body']['map'].append(copy.deepcopy(p['g']['nodes'][3]['body']['map'][0])))
    mutation('compiled-map','map-wrong-key-type',lambda p:p['g']['nodes'][3]['body']['map'][0].update(key=1))
    mutation('compiled-map','same-shape-different-key-type',lambda p:p['g']['nodes'][0].update(type=nominal('$Text',MODEL)))
    alternate_sequence=next(r['alias'] for r in reg['sites'] if r['source_catalogue_key']=='RawInputOrigin' and r['source_path']==[['field','path']])
    mutation('compiled-map','same-shape-different-site',lambda p:p['g']['nodes'][1].update(type=nominal(alternate_sequence,MODEL)))
    mutation('compiled-map','map-dangling-child',lambda p:p['g']['nodes'][3]['body']['map'][0].update(value=90))
    mutation('compiled-map','map-forward-child',lambda p:p['g']['nodes'][3]['body']['map'][0].update(value=3))
    mutation('compiled-map','map-entries-are-not-nodes',lambda p:p['g']['nodes'][3].update(type=nominal('StructuralMapEntry',MODEL)))
    mutation('compiled-map','map-record-field-omitted',lambda p:p['g']['nodes'][2]['body']['record'].pop(),True)
    mutation('set-full-members','set-repeated-member',lambda p:p['g']['nodes'][5].update(body={'set':[3,3]}))
    mutation('set-full-members','set-wrong-order',lambda p:p['g']['nodes'][5]['body']['set'].reverse())
    mutation('shared-child','sequence-duplicates-lost',lambda p:p['g']['nodes'][4]['body']['sequence'].pop())
    mutation('shared-child','wrong-shared-child',lambda p:p['g']['nodes'][4]['body']['sequence'].append(0))
    mutation('w-input-reference','w-wrong-arena',lambda p:p['w']['nodes'][1]['body']['record'][0].update(value={'local':15}))
    mutation('w-input-reference','w-wrong-target',lambda p:p['w']['nodes'][1]['body']['record'][0].update(value={'input':14}))
    mutation('w-input-reference','g-output-backedge',lambda p:p['g']['nodes'][15]['body']['record'][0].update(value={'input':0}))
    mutation('w-input-reference','w-required-reference-omitted',lambda p:p['w']['nodes'][1]['body']['record'].pop(0),True)
    mutation('w-option-some','wrong-option-tag',lambda p:p['w']['nodes'][2].update(body={'option_none':None}))
    mutation('w-option-some','variant-payload-omitted',lambda p:p['w']['nodes'][1]['body']['variant'].pop('payload'),True)
    mutation('distinct-document-occurrences','occurrence-collapse',lambda p:p['g']['roots'].__setitem__(1,19),True)
    mutation('compiled-map','record-order',lambda p:p['g']['nodes'][2]['body']['record'].reverse())
    for ty in ['ModelType','ModelRef']:
        add('admission_negative','old-owner-'+ty,dict(op='shape',type=ty,value=nominal('Compiler','agentflow.computation.proof-report.model@6')))
        add('admission_negative','unknown-name-'+ty,dict(op='shape',type=ty,value=nominal('InventedAnonymousMap',MODEL)))
    current=dict(analysis=OWNER,model=MODEL,report=SHAPE,obligation=OBLIGATION)
    for field in current:
        old=copy.deepcopy(current);old[field]='old.owner@1';add('admission_negative','mixed-tuple-'+field,dict(op='shape',type='StructuralEditionTuple',value=old))
    add('shape','codec-map-existing-grammar',dict(op='codec',kind='map',entries=[[0,2]]))
    add('shape','codec-w-input-existing-grammar',dict(op='codec',kind='input',rank=15))
    alpha=dict(path='alpha.afidl',text='afidl 1; schema binding.alpha @1; record Item { 1 value: uint64; }')
    beta=dict(path='beta.afidl',text='afidl 1; schema binding.beta @1; record Entry { 1 name: string; }')
    add('native_module_source','module-native-source-full',dict(op='module_map',sources=[alpha,beta]))
    add('native_module_source','module-native-insertion-order',dict(op='module_map',sources=[beta,alpha]))
    changed=copy.deepcopy(alpha);changed['text']=changed['text'].replace('uint64','string')
    add('native_module_source','module-native-current-definition-change',dict(op='module_map',sources=[changed,beta]))
    entries=O.module_map(dict(sources=[alpha,beta]))['entries']
    bad=copy.deepcopy(entries);bad[0][1]['types']=[];add('native_module_source','module-missing-definition',dict(op='module_map',sources=[alpha,beta],proposal=bad))
    bad=copy.deepcopy(entries);bad[0][0]='binding.wrong@1';add('native_module_source','module-key-value-mismatch',dict(op='module_map',sources=[alpha,beta],proposal=bad))
    bad=copy.deepcopy(entries);bad[0][0]='binding.alpha@01';add('native_module_source','module-invalid-native-key',dict(op='module_map',sources=[alpha,beta],proposal=bad))
    bad=copy.deepcopy(entries);bad.append(copy.deepcopy(bad[0]));add('native_module_source','module-duplicate-before-interning',dict(op='module_map',sources=[alpha,beta],proposal=bad))
    # CaseIds/coverage are derived from registered source, never Actual.
    ids=[c['case_id'] for c in cases];assert len(ids)==len(set(ids))
    js('cases.json',dict(cases=cases,coverage=coverage,production_capture_case_ids=0))
    js('field-roster.json',O.field_roster(m))
    # Exact current / mixed / old content fixtures, independent of Rust Actual.
    s=importlib.util.spec_from_file_location('structural_portable_source_inputs',ROOT/'tools/gen-portable-proof-codec-corpus.py');codec=importlib.util.module_from_spec(s);s.loader.exec_module(codec)
    from portable_proof_python_oracle import canonical_bytes
    prior=json.loads((ROOT/'spec/conformance/g01-relation-schema-staging/manifest.json').read_bytes())
    spec_ids=sorted(prior['selected_specifications']+[v[0] for v in SOURCES.values()]);rule_ids=sorted(prior['applied_rules']+[r['rule_id'] for r in RULES])
    sb=(ROOT/'spec/registry/specifications.json').read_bytes();rb=(ROOT/'spec/registry/rules.json').read_bytes()
    profile=dict(specification_bytes=sb,rule_bytes=rb,version=OWNER,spec_ids=spec_ids,rule_ids=rule_ids)
    portable=[]
    def pc(slug,p,stage,family,policy='bounded_32m_scalar'):
        file='portable/'+slug+'.ace1';raw=canonical_bytes(codec.bundle_and_context(p));path=D/file;path.parent.mkdir(parents=True,exist_ok=True)
        if not path.exists() or path.read_bytes()!=raw:path.write_bytes(raw)
        portable.append(dict(case_id=PREFIX+'portable-'+slug,file=file,stage=stage,family=family,decoder_policy=policy))
    pc('current-denied',profile,'D4','UnsupportedSemantics')
    for n in range(1,8):pc('old-label-new-content-'+str(n),{**profile,'version':'agentflow.root-export-checker@'+str(n)},'D4','InvalidIdentity')
    pc('unknown', {**profile,'version':'unknown.checker@999'},'D4','UnsupportedSemantics')
    oldprofile=dict(specification_bytes=(D/'analysis7-specifications.json').read_bytes(),rule_bytes=(D/'analysis7-rules.json').read_bytes(),version=OWNER,spec_ids=prior['selected_specifications'],rule_ids=prior['applied_rules'])
    pc('new-label-old-content',oldprofile,'D4','InvalidIdentity')
    portable.append(dict(case_id=PREFIX+'portable-default-resource-limit',file='portable/current-denied.ace1',stage='D0',family='ResourceLimit',decoder_policy='unchanged_default'))
    runtime=[PREFIX+'portable-caller-support-flag',PREFIX+'portable-noncanonical']
    js('compiler-current.json',dict(analysis=OWNER,model=MODEL,report=SHAPE,obligation=OBLIGATION,registry_hex=sb.hex(),rule_registry_hex=rb.hex(),selected_specifications=spec_ids,applied_rules=rule_ids,production_binder='NotImplemented',positive_capability='NotAuthorized'))
    boundary=[]
    for group in ['positive','negative','context','subflow','recovery']:
        for path in sorted((ROOT/'spec/conformance/computation-root-export'/group).glob('*.json')):
            source=json.loads(path.read_bytes())
            for alternate in [False,True]:
                boundary.append(dict(case_id=PREFIX+'native-boundary-'+source['case_id'].replace('/','-')+('-alternate' if alternate else '-default'),source_case_id=source['case_id'],source_file=path.relative_to(ROOT).as_posix(),alternate=alternate,expected_stage='component_validation_rejected' if source['input'].get('expected_stage')=='component_validation_rejected' else 'root_export_checker'))
    assert len(boundary)==70
    js('manifest.json',dict(analysis=OWNER,model=MODEL,report=SHAPE,obligation=OBLIGATION,metadata_case_ids=ids,case_ids=ids+[c['case_id'] for c in portable]+runtime+[c['case_id'] for c in boundary],counts={**dict(__import__('collections').Counter(c['category'] for c in cases)), 'edition_portable':len(portable)+len(runtime),'capability_denial':sum(c['category']=='capability_denial' for c in cases)+len(boundary)},field_coverage=coverage,site_case_count=len(reg['sites']),reference_policy_count=len(reg['reference_policies']),manual_sources=list(sources),portable_cases=portable,portable_runtime_case_ids=runtime,native_boundary_cases=boundary,selected_specifications=spec_ids,applied_rules=rule_ids,production_binder_capture_case_ids=0,production_dagranks_generated=0,positive_root_export='NotAuthorized'))
    print('STRUCTURAL CASES',len(cases),'FIELDS/VARIANTS',len(coverage))
