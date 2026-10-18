"""Registered-source census/schema helpers; never build value graphs or DagRanks."""
import copy
import hashlib
import json
import re
from pathlib import Path

ROOT=Path(__file__).resolve().parent.parent
BASELINE='8dc4f508c2f97b431053528ab14d2bd9263b3737'
PREVIOUS_MODEL='agentflow.computation.proof-report.model@6'
INPUT_ROOTS=['RawValidationSubject']
OUTPUT_ROOTS=['RawPortCheckBasis','RawPortRejectionWitness','CurrentContextRelationSchemaValidation','RelationSubjectAdmission']

def model_file(path):
    m=json.loads(re.search(r'```json\s*(.*?)\s*```',(ROOT/path).read_text(encoding='utf8'),re.S)[1])
    if m.get('format')!='closed_structural_delta_v1':return m
    assert set(m)=={'format','base_source','overrides','append_types','replace_types','append_type_ranks','append_roots'}
    basepath='spec/tables/proof-report-model-v6.md';raw=(ROOT/basepath).read_bytes()
    assert m['base_source']==dict(specification_id=PREVIOUS_MODEL,logical_path=basepath,plain_sha256=hashlib.sha256(raw).hexdigest())
    base=model_file(basepath);out=copy.deepcopy(base)
    assert set(m['overrides'])=={'model_id','model_edition','obligation_edition','analysis_owner','profile','canonical_constraints','structural_binding_registration'}
    assert not set(m['append_types'])&set(base['types'])
    assert set(m['replace_types'])=={'NodeBody','ScopeSubject'}
    out.update(m['overrides']);out['types']={**base['types'],**m['replace_types'],**m['append_types']}
    out['type_ranks']={**base['type_ranks'],**m['append_type_ranks']}
    out['descriptor_roots']=base['descriptor_roots']+m['append_roots']
    out['inherited_registered_model']=base
    assert set(out['type_ranks'])==set(out['types']) and set(m['append_type_ranks'])==set(m['append_types'])
    return out

def walk(value):
    yield value
    if isinstance(value,dict):
        for v in value.values():yield from walk(v)
    elif isinstance(value,list):
        for v in value:yield from walk(v)

def references(value):
    return [v['ref'] for v in walk(value) if isinstance(v,dict) and 'ref' in v]

def closure(types,roots):
    seen=set();todo=list(roots)
    while todo:
        name=todo.pop()
        if name in seen:continue
        assert name in types,('unresolved nominal reference',name)
        seen.add(name);todo.extend(references(types[name]))
    return seen

def nominal(key,owner):
    prefix,slash,suffix=key.partition('/')
    if slash and re.fullmatch(r'[a-z][a-z0-9.-]*@[1-9][0-9]*',prefix):
        return dict(owner=prefix,name=suffix)
    return dict(owner=owner,name=key)

def descriptor_children(d):
    k=d.get('kind')
    if k=='record':return [([['field',n]],v) for n,v in d['fields']]
    if k=='enum':return [([['case',n]],v) for n,v in d['variants']]
    if k in ('sequence','set','option'):return [([[{'sequence':'item','set':'member','option':'some'}[k],None]],d['item'])]
    if k=='map':return [([['key',None]],d['key']),([['value',None]],d['value'])]
    return []

def site_census(types,owner,input_roots=INPUT_ROOTS,output_roots=OUTPUT_ROOTS):
    inputs=closure(types,input_roots);outputs=closure(types,output_roots)
    eligible=inputs|outputs;sites=[];refsites=[]
    def descend(parent,d,path):
        if 'ref' in d:return
        if d.get('kind')=='dag_ref':
            refsites.append(dict(parent=nominal(parent,owner),catalogue_key=parent,path=path,target=copy.deepcopy(d['target']),domain='G_local' if parent in inputs else 'G_external',profiles=['raw_contract_validation']))
            return
        if path:
            sites.append(dict(parent=nominal(parent,owner),catalogue_key=parent,path=path,kind=d['kind'],descriptor=copy.deepcopy(d),eligibility=['G_input'] if parent in inputs else ['W_output'],source_model=PREVIOUS_MODEL))
        for addition,child in descriptor_children(d):descend(parent,child,path+addition)
    for parent in sorted(eligible,key=lambda n:list(types).index(n)):
        descend(parent,types[parent],[])
    # Nominal references terminate local syntax paths. Every declaration is
    # reviewed once; aliases delegate body sites to their actual declaration.
    keys=[json.dumps([s['parent'],s['path']],sort_keys=True) for s in sites]
    assert len(keys)==len(set(keys))
    return dict(input_types=sorted(inputs),output_types=sorted(outputs),sites=sites,reference_sites=refsites,input_roots=input_roots,output_roots=output_roots)

def draft_census():
    old=model_file('spec/tables/proof-report-model-v6.md')
    result=site_census(old['types'],old['model_id'])
    result.update(status='G2-C SOURCE CENSUS; SUCCESSOR IDS/RANKS UNALLOCATED',baseline=BASELINE,type_ranks_allocated=0,production_dagranks_generated=0)
    return result

def ref(n):return {'ref':n}
def rec(**fields):return {'kind':'record','fields':[[n,d] for n,d in fields.items()]}
def seq(d,minimum=0):return {'kind':'sequence','item':d,'minimum':minimum}
def opt(d):return {'kind':'option','item':d,'minimum':0}
def en(**variants):return {'kind':'enum','variants':[[n,d] for n,d in variants.items()]}
UNIT={'kind':'unit'}
TEXT=ref('$Text');U64=ref('$U64');MODELTYPE=ref('ModelType')

def stage_definitions():
    """Reviewed symbolic definitions. No successor ID/rank is allocated here."""
    d={
      'StructuralSelector':en(field=TEXT,case=TEXT,item=UNIT,member=UNIT,key=UNIT,value=UNIT,some=UNIT),
      'StructuralStage':en(declared_input=UNIT,source_admission=UNIT,pre_capture_binding=UNIT,c_validation=UNIT,raw_fact_capture=UNIT,t4_normalization=UNIT,g02_scope=UNIT,registration_admission=UNIT),
      'StructuralProfile':en(raw_contract_validation=UNIT,registration_subgraph=UNIT,other_registered_profile=UNIT),
      'StructuralBodyKind':en(scalar=UNIT,record=UNIT,variant=UNIT,sequence=UNIT,set=UNIT,map=UNIT,option=UNIT),
      'StructuralReferenceDomain':en(g_local=UNIT,g_external=UNIT,w_local=UNIT),
      'StructuralValueEligibility':en(g_input=UNIT,w_output=UNIT),
      'StructuralRankDependency':en(local_child_before_parent=UNIT,external_g_not_w_child=UNIT),
      'StructuralSiteKey':rec(parent=MODELTYPE,path=seq(ref('StructuralSelector'),1)),
      'StructuralChildBinding':rec(position=ref('StructuralSelector'),value_type=MODELTYPE,domain=ref('StructuralReferenceDomain')),
      'StructuralSourceType':rec(owner=ref('SpecificationId'),name=TEXT),
      'StructuralSiteDefinition':rec(key=ref('StructuralSiteKey'),value_type=MODELTYPE,kind=ref('StructuralBodyKind'),children=seq(ref('StructuralChildBinding')),eligibility=ref('StructuralValueEligibility'),source_model=ref('SpecificationId'),parent_source=ref('StructuralSourceType'),source_stage=ref('StructuralStage'),reference_policy=ref('StructuralReferenceDomain'),rank_dependency=ref('StructuralRankDependency')),
      'StructuralReferencePolicy':rec(key=ref('StructuralSiteKey'),environment=ref('StructuralReferenceUse'),target=MODELTYPE,domain=ref('StructuralReferenceDomain'),profiles=seq(ref('StructuralProfile'),1)),
      'StructuralMapEntry':rec(key=U64,value=U64),
      'StructuralEditionTuple':rec(analysis=ref('OwnerVersion'),model=ref('SpecificationId'),report=ref('ModelEdition'),obligation=ref('ObligationEdition')),
      'StructuralWEdge':en(local=U64,input=U64),
      'StructuralWField':rec(field=ref('ClosedField'),value=ref('StructuralWEdge')),
      'StructuralWEntry':rec(key=ref('StructuralWEdge'),value=ref('StructuralWEdge')),
      'StructuralWVariant':rec(case=ref('ClosedCase'),payload=ref('StructuralWEdge')),
      'StructuralWBody':en(scalar=ref('ScalarValue'),record=seq(ref('StructuralWField')),variant=ref('StructuralWVariant'),sequence=seq(ref('StructuralWEdge')),set=seq(ref('StructuralWEdge')),option_none=ref('$unit'),option_some=ref('StructuralWEdge'),map=seq(ref('StructuralWEntry'))),
      'StructuralWNode':rec(type=MODELTYPE,body=ref('StructuralWBody')),
      'StructuralWitnessDag':rec(nodes=seq(ref('StructuralWNode')),roots=seq(U64,1)),
      'StructuralReferenceEnvironment':rec(edition=ref('StructuralEditionTuple'),profile=ref('StructuralProfile'),input_graph=ref('CanonicalValueDag'),witness_graph=opt(ref('StructuralWitnessDag')),root_types=seq(MODELTYPE,1)),
      'StructuralPhysicalSource':en(not_supplied=UNIT,supplied=rec(value_path=seq(ref('RawDocumentMember')))),
      # Complete rank-free source values; RawParsedInput is deliberately NOT
      # used, because its occurrence/physical bindings already require Node<T>.
      'StructuralDeclaredInputs':rec(compiler=ref('CompleteCompiler'),package=ref('SourcePackageMetadata'),owner=ref('DocumentOwnerAnchor'),occurrence=ref('DocumentOccurrenceKey'),document_path=seq(ref('RawDocumentMember')),physical=ref('StructuralPhysicalSource'),candidate=ref('RawCandidateContract'),context=ref('RawContextBinding')),
      'StructuralSourceAssurance':en(declared_only=UNIT,native_associated=UNIT),
      'StructuralBindingOutputClaim':rec(environment=ref('StructuralReferenceEnvironment'),source_assurance=ref('StructuralSourceAssurance')),
      'StructuralSubjectAdmission':rec(subject={'kind':'dag_ref','target':ref('RawValidationSubject')},observations=seq(ref('CurrentContextRelationSchemaValidation')),relation_partition_complete=ref('$Bool'),source_assurance=ref('StructuralSourceAssurance')),
      'StructuralFailureDomain':en(UnsupportedModel=UNIT,IncompleteContent=UNIT,IncompleteFacts=UNIT,InvalidFacts=UNIT,ResourceLimit=UNIT,NotImplemented=UNIT),
      'StructuralFailureReason':en(missing_site=UNIT,unknown_nominal_type=UNIT,invalid_shape=UNIT,missing_node=UNIT,missing_rank_source=UNIT,duplicate_map_key=UNIT,invalid_map_entry=UNIT,invalid_set_member=UNIT,invalid_child=UNIT,rank_dependency=UNIT,invalid_interning=UNIT,input_output_backedge=UNIT,resource_bound=UNIT,cross_edition=UNIT,source_association=UNIT,ineligible_value_type=UNIT,production_not_implemented=UNIT),
      'StructuralInputStep':en(field=TEXT,case=TEXT,index=U64),
      'StructuralBindingFailure':rec(domain=ref('StructuralFailureDomain'),reason=ref('StructuralFailureReason'),stage=ref('StructuralStage'),site=opt(ref('StructuralSiteKey')),expected_type=opt(MODELTYPE),observed_type=opt(MODELTYPE),first_input_path=seq(ref('StructuralInputStep')),second_input_path=opt(seq(ref('StructuralInputStep')))),
      'StructuralAvailability':en(NotImplemented=UNIT),
      'StructuralRegistration':en(Unregistered=UNIT,Registered=UNIT),
      'StructuralCapability':en(NotAuthorized=UNIT),
      'StructuralStageStatus':rec(registration=ref('StructuralRegistration'),production_binder=ref('StructuralAvailability'),raw_full_capture=ref('StructuralAvailability'),normalizer=ref('StructuralAvailability'),scope=ref('StructuralAvailability'),root_export=ref('StructuralCapability'),publication=ref('StructuralCapability'),runtime=ref('StructuralCapability')),
      'StructuralStageEdge':rec(source=ref('StructuralStage'),target=ref('StructuralStage')),
      'StructuralStageContract':rec(input_type=MODELTYPE,output_type=MODELTYPE,source_stage=ref('StructuralStage'),binding_stage=ref('StructuralStage'),consumer_stage=ref('StructuralStage'),dependencies=seq(ref('StructuralStageEdge')),input_roots=seq(MODELTYPE,1),output_roots=seq(MODELTYPE,1),status=ref('StructuralStageStatus')),
    }
    d['StructuralReferenceUse']=en(g_value=UNIT,w_value=UNIT,output_record=UNIT)
    d['StructuralModuleMapAssurance']=en(module_projection_only=UNIT)
    d['StructuralModuleMapInspection']=rec(entries={'kind':'map','key':ref('SpecificationId'),'value':ref('CompiledModuleContents')},entry_count=U64,assurance=ref('StructuralModuleMapAssurance'))
    return d

if __name__=='__main__':
    import collections
    result=draft_census()
    out=ROOT/'docs/implementation/phase-04c1b4c4r3c2r4g2-source-site-census.json'
    if '--emit-census' in __import__('sys').argv:
        raw=(json.dumps(result,indent=2,ensure_ascii=False)+'\n').encode()
        if not out.exists() or out.read_bytes()!=raw:out.write_bytes(raw)
    print(json.dumps(dict(input_types=len(result['input_types']),output_types=len(result['output_types']),sites=len(result['sites']),kinds=dict(collections.Counter(s['kind'] for s in result['sites'])),reference_sites=len(result['reference_sites']),type_ranks_allocated=0,production_dagranks_generated=0),indent=2))
