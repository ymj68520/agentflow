"""Materialize reviewed registration declarations/corpus; never a production binder."""
import collections, copy, hashlib, importlib.util, json, re, subprocess, sys
from pathlib import Path
from structural_binding_schema import *
from structural_binding_registration_data import *

R=ROOT;D=R/'spec/conformance/g01-structural-binding-staging'

def write(path,raw):
    p=R/path;p.parent.mkdir(parents=True,exist_ok=True)
    if not p.exists() or p.read_bytes()!=raw:p.write_bytes(raw)

def js(path,value):write(path,(json.dumps(value,indent=2,sort_keys=True,ensure_ascii=False)+'\n').encode())
def selector(part):
    k,v=part;return {k:v}
def sections(key):
    return ''.join('\n<a id="'+r['anchor']+'"></a>\n\n## '+r['rule_id']+'\n\nMUST: '+r['statement']+'\n\nDirect normative dependencies: '+', '.join(r['dependencies'])+'.\nRegistration conformance: '+r['case_id']+'.\n' for r in RULES if r['source']==key)

def registered_model():
    old=model_file('spec/tables/proof-report-model-v6.md');m=copy.deepcopy(old)
    census=site_census(old['types'],PREVIOUS_MODEL)
    defs=stage_definitions();t=copy.deepcopy(old['types']);t.update(defs)
    alias={}
    def skey(parent,path):return json.dumps([parent,path],sort_keys=True)
    for i,row in enumerate(census['sites'],1):
        name='StructuralValue%04d'%i
        assert name not in t
        alias[skey(row['catalogue_key'],row['path'])]=name
        t[name]=copy.deepcopy(row['descriptor'])
    def value_type(parent,path,desc):
        if 'ref' in desc:return nominal(desc['ref'],MODEL)
        if desc.get('kind')=='dag_ref':return nominal(desc['target']['ref'],MODEL)
        return nominal(alias[skey(parent,path)],MODEL)
    rows=[]
    for row in census['sites']:
        parent=row['catalogue_key'];path=row['path'];d=row['descriptor'];is_g=parent in census['input_types']
        children=[]
        for addition,child in descriptor_children(d):
            dom='g_local' if is_g else 'g_external' if child.get('kind')=='dag_ref' else 'w_local'
            children.append(dict(position=selector(addition[0]),value_type=value_type(parent,path+addition,child),domain=dom))
        kind={'enum':'variant','option':'option'}.get(d['kind'],d['kind'] if d['kind'] in ['record','map','sequence','set'] else 'scalar')
        source_stage='declared_input' if is_g else 'raw_fact_capture' if parent in ['RawPortCheckBasis','RawPortRejectionWitness'] else 'c_validation'
        definition=dict(key=dict(parent=nominal(parent,MODEL),path=[selector(p) for p in path]),value_type=value_type(parent,path,d),kind=kind,children=children,eligibility='g_input' if is_g else 'w_output',source_model=PREVIOUS_MODEL,parent_source=nominal(parent,PREVIOUS_MODEL),source_stage=source_stage,reference_policy='g_local' if is_g else 'w_local',rank_dependency='local_child_before_parent')
        rows.append(dict(definition=definition,source_catalogue_key=parent,source_path=path,descriptor=copy.deepcopy(d),alias=definition['value_type']['name']))
    # Existing declaration names/ranks stay in order; only the successor's
    # body/root meanings change. The complete previous object stays nested exact.
    t['NodeBody']['variants'].append(['map',seq(ref('StructuralMapEntry'))])
    raw=dict(t['ScopeSubject']['variants'])
    assert raw['raw_contract_validation']==ref('RawValidationSubject')
    t['ScopeSubject']['variants']=[[n,rec(subject={'kind':'dag_ref','target':ref('RawValidationSubject')}) if n=='raw_contract_validation' else d] for n,d in t['ScopeSubject']['variants']]
    policies=[]
    def policy_scan(parent,d,path):
        if 'ref' in d:return
        if d.get('kind')=='dag_ref':
            target=d['target']['ref'];is_g=parent in census['input_types']
            raw_profile=is_g or target in ['RawValidationSubject','ContractProofContextContents','NativeRelationSnapshotSource']
            uses=['output_record']
            if is_g:uses.append('g_value')
            if parent in census['output_types']:uses.append('w_value')
            for use in uses:
                policies.append(dict(key=dict(parent=nominal(parent,MODEL),path=[selector(p) for p in path]),environment=use,target=nominal(target,MODEL),domain='g_local' if use=='g_value' else 'g_external',profiles=['raw_contract_validation' if raw_profile else 'other_registered_profile']))
            return
        for addition,child in descriptor_children(d):policy_scan(parent,child,path+addition)
    for name,d in t.items():
        if name not in {row['alias'] for row in rows}:policy_scan(name,d,[])
    bindings=[]
    for name in sorted(set(census['input_types'])|set(census['output_types']),key=lambda n:old['type_ranks'][n]):
        origin=name;body=t[name];seen=set()
        while 'ref' in body:
            assert origin not in seen;seen.add(origin);origin=body['ref'];body=t[origin]
        bindings.append(dict(value_type=nominal(name,MODEL),catalogue_key=name,declaration=origin,path=[],eligibility=['g_input','w_output'] if name in census['input_types'] else ['w_output']))
    for row in rows:
        bindings.append(dict(value_type=row['definition']['value_type'],catalogue_key=row['alias'],declaration=row['source_catalogue_key'],path=row['source_path'],eligibility=['g_input','w_output'] if row['definition']['eligibility']=='g_input' else ['w_output']))
    source_closure=closure(t,['StructuralDeclaredInputs'])
    assert not any(v.get('kind')=='dag_ref' for n in source_closure for v in walk(t[n]) if isinstance(v,dict))
    assert all(n in t for n in references(t))
    new_names=list(defs)+[row['alias'] for row in rows]
    ranks={**old['type_ranks'],**{n:len(old['types'])+i for i,n in enumerate(new_names)}}
    assert len(ranks)==len(t)==len(set(ranks.values()))
    status=dict(registration='Registered',production_binder='NotImplemented',raw_full_capture='NotImplemented',normalizer='NotImplemented',scope='NotImplemented',root_export='NotAuthorized',publication='NotAuthorized',runtime='NotAuthorized')
    edges=[('declared_input','source_admission'),('source_admission','pre_capture_binding'),('pre_capture_binding','c_validation'),('c_validation','raw_fact_capture'),('raw_fact_capture','t4_normalization')]
    stage=dict(input_type=nominal('StructuralDeclaredInputs',MODEL),output_type=nominal('StructuralBindingOutputClaim',MODEL),source_stage='source_admission',binding_stage='pre_capture_binding',consumer_stage='raw_fact_capture',dependencies=[dict(source=a,target=b) for a,b in edges],input_roots=[nominal('RawValidationSubject',MODEL)],output_roots=[nominal(n,MODEL) for n in OUTPUT_ROOTS],status=status)
    constraints=[c.replace('all66 source bindings here,40 selected specifications and204 applied rules','all70 source bindings here,44 selected specifications and217 applied rules').replace('| option_none:unit | option_some:DagRank.','| option_none:unit | option_some:DagRank | map:Seq<{key:DagRank,value:DagRank}>.') for c in old['canonical_constraints']]
    constraints += [
      'Explicit successor delta only: ScopeSubject.raw_contract_validation={subject:Node<RawValidationSubject>}; the single ordered G root is that complete candidate input Subject. Fact use additionally requires genuine source and all Some C Valid admission.',
      'Site aliases are exact nominal declaration-site identities; nested sites delegate to the original parent/path, not a new iteration over generated aliases. Actual names/child mappings are the closed site table, never runtime shape names.',
      'ModelType ordering is exact owner unsigned UTF8, active declared type rank, exact name; old774 ranks are unchanged. Explicit foreign qualified names retain their true owners. Type rank is not DagRank.',
      'G uses unchanged0217; new map child order is key/value pairs in full typed key order, duplicate keys reject before interning. Metadata entry records are not extra value nodes. Set retains its old body and domain-specific full typed membership rules.',
      'StructuralWBody has explicit local/input edges, exact declared target domains, local heights and full typed external target comparison. Full envelope equality includes G and exact tuple/profile/root environment.',
      'StructuralDeclaredInputs contains no dag_ref; expanded source and actual native association precede graph binding. Data claims never seal a receipt. No outputs, facts, Scope, report or obligations determine G.',
      'All new fields/sites/variants and rules require independently source-derived coverage and simultaneous Expected/Actual omission rejection. Registration graph probes are bounded manual inputs only; production binder and actual full Raw ranks remain absent.',
      'Legacy Native metadata APIs remain explicitly old-owner metadata, never current Structural admission or upgraded full Facts. A equality, actual C lookup/validation, occurrence identity and port verdicts are unchanged.'
    ]
    m.update(model_id=MODEL,model_edition=SHAPE,obligation_edition=OBLIGATION,analysis_owner=OWNER,types=t,type_ranks=ranks,inherited_registered_model=old,profile='pre_capture_structural_binding',registration_only=True,producer_status='NotImplemented',normalizer_status='NotImplemented',lossless_scope_status='NotImplemented',positive_capability='NotAuthorized',canonical_constraints=constraints,descriptor_roots=old['descriptor_roots']+new_names)
    m['structural_binding_registration']=dict(source_model=PREVIOUS_MODEL,authored_types=new_names,changed_types=['NodeBody','ScopeSubject'],input_types=census['input_types'],output_types=census['output_types'],sites=rows,reference_policies=policies,value_bindings=bindings,stage_contract=stage,source_closure_types=sorted(source_closure),root_profiles=dict(raw_contract_validation=[nominal('RawValidationSubject',MODEL)]),production_binder='NotImplemented',production_capture_case_ids=0,actual_production_dagranks_generated=0,status=status,metadata_limits=dict(nodes=64,work=16384,depth=32),scalar_bodies=dict(unit='unit',null='null',bool='bool',u8='uint32',u32='uint32',u64='uint64',i32='int32',i64='int64',decimal='decimal',finite_f64='float64',nfc_text='string',bytes='bytes',timestamp_i64='timestamp',duration_i64='duration'))
    consumers=['NodeBody','ScopeSubject','Source','Target','Coordinate','Origin','RawValidationOrigin','FactKey','TypedContradiction','EvidenceRef']
    m['structural_binding_registration']['coverage_types']=list(dict.fromkeys(new_names+census['output_types']+consumers))
    return m

def model():
    # Snapshot bytes come from the exact frozen predecessor, never new globals.
    for n in ['specifications','rules']:
        path=D/('analysis7-'+n+'.json')
        if not path.exists():write(path.relative_to(R),subprocess.check_output(['git','show',BASELINE+':spec/registry/'+n+'.json'],cwd=R))
    m=registered_model()
    for key,(_,_,path,_) in SOURCES.items():
        if key=='RM':continue
        original=(R/path).read_text(encoding='utf8').split('\n<a id=')[0].rstrip()
        write(path,(original+'\n'+sections(key)).encode())
    old=model_file('spec/tables/proof-report-model-v6.md')
    basepath='spec/tables/proof-report-model-v6.md';new_names=m['structural_binding_registration']['authored_types']
    delta=dict(format='closed_structural_delta_v1',base_source=dict(specification_id=PREVIOUS_MODEL,logical_path=basepath,plain_sha256=hashlib.sha256((R/basepath).read_bytes()).hexdigest()),overrides={k:v for k,v in m.items() if k not in ['types','type_ranks','descriptor_roots','inherited_registered_model'] and v!=old.get(k)},append_types={n:m['types'][n] for n in new_names},replace_types={n:m['types'][n] for n in ['NodeBody','ScopeSubject']},append_type_ranks={n:m['type_ranks'][n] for n in new_names},append_roots=new_names)
    intro=('# '+MODEL+'\n\nRegistered structural staging. Production Binder / Full Raw / Normalizer / Scope remain NotImplemented.\n\n'
        '## Complete source composition, not a hash-only model\n\n'
        'The closed_structural_delta_v1 source has exactly format, base_source, overrides, append_types, replace_types, append_type_ranks and append_roots. '
        'Its base_source is the complete exact registered model@6 source, provided in full in Compiler/Portable normative_sources and compiled implementation-owned contents. '
        'The checksum checks those complete bytes; it never substitutes for their availability or full structural comparison. No latest/label-only or external-file fallback exists. '
        'Expand by deep-copying that exact parsed base, applying the listed header/constraint/registration overrides, appending disjoint named types/ranks/roots and replacing only NodeBody and ScopeSubject under the new owner. '
        'The effective inherited_registered_model remains the complete exact base object; all97 imported declarations and774 prior ranks remain exact. '
        'Unknown keys, missing/full-content mismatch, unresolved references, overlapping appended names or other replacements reject. '
        'This is a closed normative source representation, not a new ACE1 value grammar, hash domain or reduced Compiler manifest. '
        'It avoids duplicating existing complete source bytes beyond the unchanged32MiB native identity limit.\n\n')
    write(SOURCES['RM'][2],(intro+'```json\n'+json.dumps(delta,ensure_ascii=False,separators=(',',':'))+'\n```\n'+sections('RM')).encode())
    assert model_file(SOURCES['RM'][2])==m,'lossless source composition failed'
    print(json.dumps(dict(descriptors=len(m['types']),references=len(references(m['types'])),sites=len(m['structural_binding_registration']['sites']),reference_policies=len(m['structural_binding_registration']['reference_policies']),new_types=len(m['structural_binding_registration']['authored_types']),production_dagranks_generated=0),indent=2))
    return m

if __name__=='__main__':
    m=model()
    if '--model-only' not in sys.argv:
        registry=json.loads((R/'spec/registry/specifications.json').read_bytes())['specifications']
        exact=all(any(row['specification_id']==sid and row['normative_source']['content_digest']['hex']==hashlib.sha256((R/path).read_bytes()).hexdigest() for row in registry) for sid,_,path,_ in SOURCES.values())
        if not exact:
            # Authoring dependency only: refresh complete canonical Registry
            # before emitting identity-sensitive fixtures. Old record equality
            # is enforced by that generator; no new IDs are allocated here.
            subprocess.run([sys.executable,str(R/'tools/gen-registry-instances.py')],cwd=R,check=True)
        import structural_binding_cases
        structural_binding_cases.materialize(m)
