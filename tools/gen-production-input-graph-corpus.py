"""Actual declared input cases. Never reads Rust Actual or chooses DAG ranks."""
import copy,hashlib,importlib.util,json,sys
from pathlib import Path
R=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(R/'tools'))
from structural_binding_schema import model_file,descriptor_children
from structural_binding_cases import sample
from oracle.production_input_graph import REL,RAW,validate_flow_source
D=R/'spec/conformance/production-input-graph'
PREFIX='production-input-g/'
def write(name,value):
    p=D/name;raw=(json.dumps(value,ensure_ascii=False,sort_keys=True,indent=2)+'\n').encode()
    if not p.exists() or p.read_bytes()!=raw:p.write_bytes(raw)
def named_routes(types,root):
    routes={root:[]};queue=[root]
    while queue:
        name=queue.pop(0);todo=[(types[name],routes[name])]
        while todo:
            d,path=todo.pop()
            if 'ref' in d:
                n=d['ref']
                if n not in routes:routes[n]=path;queue.append(n)
            else:todo.extend((ch,path+selectors) for selectors,ch in descriptor_children(d))
    return routes
def wire(types,d,value):
    if 'ref' in d:
        name=d['ref']
        if name=='Raw($String)':return bytes(value).decode('utf8')
        out=wire(types,types[name],value)
        if name in ['Raw(af-contract::ComponentInputPort)','Raw(af-contract::ComponentOutputPort)']:out['type']=out.pop('type_ref')
        if name=='Raw(af-semantic::PrimitiveConstant)' and isinstance(out,str):out={out:None}
        return out
    kind=d['kind']
    if kind=='record':return {n:wire(types,ch,value[n]) for n,ch in d['fields']}
    if kind=='enum':
        tag=value if isinstance(value,str) else next(iter(value));desc=dict(d['variants'])[tag];name=RAW.snake(tag)
        if desc.get('kind')=='unit':return name
        payload=wire(types,desc,value[tag]);payload=payload['0'] if isinstance(payload,dict) and set(payload)=={'0'} else payload
        return {name:payload}
    if kind in ['sequence','set']:return [wire(types,d['item'],v) for v in value]
    if kind=='option':return None if 'none' in value else wire(types,d['item'],value['some'])
    return value
def raw_vocabulary():
    m=model_file('spec/tables/proof-report-model-v7.md');routes=named_routes(m['types'],'RawCandidateContract');rows=[]
    for site in m['structural_binding_registration']['sites']:
        parent=site['source_catalogue_key']
        if parent not in routes:continue
        route=routes[parent]+site['source_path']
        raw=sample(m,{'ref':'RawCandidateContract'},route)
        native=wire(m['types'],{'ref':'RawCandidateContract'},raw)
        rows.append(dict(source_site=site['alias'],candidate=raw,wire_candidate=native))
        kind=site['descriptor']['kind']
        if kind in ['sequence','option']:
            extra=route+[[{'sequence':'item','option':'some'}[kind],None]]
            raw=sample(m,{'ref':'RawCandidateContract'},extra)
            rows.append(dict(source_site=site['alias'],applicability='nonempty' if kind=='sequence' else 'some',candidate=raw,wire_candidate=wire(m['types'],{'ref':'RawCandidateContract'},raw)))
    # Named record fields and cases with named payloads may have NO anonymous
    # site. Cover them explicitly rather than treating the Site count as the
    # complete native type/field/variant denominator.
    for name,route in sorted(routes.items()):
        raw=sample(m,{'ref':'RawCandidateContract'},route)
        rows.append(dict(source_site=None,native_type=name,candidate=raw,wire_candidate=wire(m['types'],{'ref':'RawCandidateContract'},raw)))
        for tag,_ in m['types'][name].get('variants',[]):
            raw=sample(m,{'ref':'RawCandidateContract'},route+[['case',tag]])
            rows.append(dict(source_site=None,native_type=name,native_variant=tag,candidate=raw,wire_candidate=wire(m['types'],{'ref':'RawCandidateContract'},raw)))
    for source in RAW.vectors():
        if source['case_id'].startswith('c2r1/complete-thirteen-fields/') or source['case_id']=='c2r1/declaration_bundle/numeric':
            rows.append(dict(source_site=None,input_condition=source['case_id'],candidate=source['candidate'],wire_candidate=source['wire_candidate']))
    assert rows
    return rows
def package(rows,identity):
    documents=[{'declared_native_document':{'declarations':[r['candidate'] for r in rows]}}]
    documents.append({'physical_json_utf8':{'bytes':list(json.dumps({'selected':rows[0]['candidate']},ensure_ascii=True,separators=(',',':')).encode())}})
    domain={'namespace':RAW.key('G3 actual declared corpus'),'domain_key':RAW.key('input-domain')};owners=[]
    for kind,title,field,label in [('declaration_bundle','bundle_name','declarations','declaration_name'),('import_manifest','manifest_name','imports','import_name'),('workflow_source_collection','collection_name','sources','source_name')]:
        anchor=dict(domain=copy.deepcopy(domain),owner_key=RAW.key(identity+'/'+kind),revision_key=RAW.key('revision-actual'))
        owners.append(dict(anchor=anchor,container={kind:{title:RAW.key('complete-declared-'+kind),field:[{label:RAW.key('native-slot'),'document':copy.deepcopy(documents[0])},{label:RAW.key('physical-slot'),'document':copy.deepcopy(documents[1])},{label:RAW.key('equal-content-independent-slot'),'document':copy.deepcopy(documents[0])}]}}))
    return dict(domain=domain,owners=owners)
def relation_inputs():
    values={slot:REL.catalog(dict(slot=slot,catalog='nonempty')) for slot,_,_ in REL.SLOTS}
    for row in values['recovery_usage']['rules']:
        for field in ['forward_effect','action_effect']:
            row['action'][field]['resource_selector']['scope_bindings']=[dict(key='tenant',value='acme')]
    flow=values['flow_execution'];domains=flow['closed_trust_domains']
    flow['confidentiality_rules'].append(dict(label=dict(catalog='agentflow.labels@1',name='public'),region_pairs={'all_pairs_in_closed_region_snapshot':None}))
    policies=[{'exact_allowlist':domains},{'all_domains_in_closed_trust_snapshot':None},{'no_additional_trust_requirement':None}]
    floors=[{'no_additional_minimum':None},{'minimum':'basic'},{'minimum':'strong'}]
    flow['taint_rules']=[dict(taint=dict(catalog='agentflow.taints@1',name=f'tag-{i}'),trust_domains=policies[i],sandbox=floors[i]) for i in range(3)]
    flow['authority_rules']=[dict(authority=a,trust_domains=policies[i%3],sandbox=floors[i%3]) for i,a in enumerate(['none','external_data','delegated_agent','user','workflow','platform'])]
    atom=copy.deepcopy(values['recovery_usage']['rules'][0]['action']['forward_effect'])
    flow['sink_rules']=[dict(kind=k,sink=s,trust_domains=policies[i%3],sandbox=floors[i%3]) for i,(k,s) in enumerate([(k,s) for k in ['data_egress','decision_influence'] for s in [{'workflow_output':1},{'exact_effect':atom}]])]
    for slot,value in values.items():
        if slot=='flow_execution':
            value['confidentiality_rules'].sort(key=lambda r:(r['label']['catalog'],r['label']['name']))
            value['sink_rules'].sort(key=lambda r:(['data_egress','decision_influence'].index(r['kind']),0 if 'workflow_output' in r['sink'] else 1))
            validate_flow_source(value)
        else:value=REL.CAT.canonical_relation(REL.KIND[slot],value);REL.validate_native_catalog(slot,value)
        values[slot]=value
    return values
def main():
    vocabulary=raw_vocabulary();write('raw-vocabulary.json',vocabulary)
    bundles=[vocabulary[i:i+24] for i in range(0,len(vocabulary),24)]
    cases=[]
    # Every case supplies a real complete document container. All embedded
    # declarations are native-deserialized by the Rust driver, not shape-only
    # stand-ins. Explicit owner/slot are actual declared input data, not ranks.
    for b,rows in enumerate(bundles):
        write(f'bundle-{b:02}.json',dict(package=package(rows,'bundle-'+str(b)),declarations=rows))
    for i in range(max(96,len(bundles)*12)):
        b=i%len(bundles);scenario=(i//len(bundles))%12;slot=[0,1,2][scenario%3]
        cases.append(dict(case_id=PREFIX+f'bind-{i:03}',bundle=f'bundle-{b:02}.json',owner=(scenario//3)%3,slot=slot,selected_entry=0 if slot==1 else scenario%len(bundles[b]),
            context='rich' if scenario!=11 else 'empty',relations='none' if scenario in [0,4,8] else 'nonempty',module_order='reverse' if scenario%2 else 'forward',current_modules='missing_relation' if scenario==10 else 'complete'))
    for i,base in enumerate(copy.deepcopy(cases[:4])):
        base.update(case_id=PREFIX+f'map-order-mirror-{i:02}',module_order='forward' if base['module_order']=='reverse' else 'reverse',equal_graph_to=cases[i]['case_id']);cases.append(base)
    for name,owner,slot in [('repeat',0,0),('owner-one',1,0),('owner-two',2,0),('equal-content-slot-two',0,2)]:
        base=copy.deepcopy(cases[0]);base.update(case_id=PREFIX+'sensitivity-'+name,owner=owner,slot=slot)
        base['equal_graph_to' if name=='repeat' else 'different_graph_from']=cases[0]['case_id'];cases.append(base)
    for state in ['empty','single']:
        base=copy.deepcopy(cases[0]);base.update(case_id=PREFIX+'compiled-module-map-'+state,current_modules=state);cases.append(base)
    write('cases.json',cases);write('relations.json',relation_inputs())
    negatives=[('missing-package','IncompleteFacts','missing_node'),('missing-owner','IncompleteFacts','missing_node'),('missing-slot','IncompleteFacts','missing_node'),
        ('wrong-owner','InvalidFacts','source_association'),('wrong-revision','InvalidFacts','source_association'),('clone-document','InvalidFacts','source_association'),('wrong-candidate','InvalidFacts','source_association'),('wrong-entry','InvalidFacts','source_association'),
        ('split-context','InvalidFacts','source_association'),('native-supplied','UnsupportedModel','invalid_shape'),('physical-not-supplied','UnsupportedModel','invalid_shape'),
        ('physical-wrong-path','InvalidFacts','source_association'),('physical-duplicate-json','InvalidFacts','source_association'),('physical-invalid-utf8','InvalidFacts','source_association'),
        ('old-compiler','UnsupportedModel','cross_edition'),('wrong-compiler-content','InvalidFacts','source_association'),('missing-compiler-source','InvalidFacts','source_association'),
        ('duplicate-owner','InvalidFacts','source_association'),('foreign-domain','InvalidFacts','source_association'),('unknown-container','UnsupportedModel','invalid_shape'),
        ('extra-package-field','UnsupportedModel','invalid_shape'),('invalid-key-utf8','UnsupportedModel','invalid_shape'),('key-bound','ResourceLimit','resource_bound')]
    write('negative-cases.json',[dict(case_id=PREFIX+'source-'+name,operation=name,domain=domain,reason=reason,stage='source_admission') for name,domain,reason in negatives])
    write('boundary-cases.json',[dict(case_id=PREFIX+'boundary-'+name,operation=name,domain=domain,reason=reason,stage=stage) for name,domain,reason,stage in [
        ('deep-native-schema','ResourceLimit','resource_bound','source_admission'),('nfd-native-context','UnsupportedModel','invalid_shape','source_admission'),('work-exhaustion','ResourceLimit','resource_bound','pre_capture_binding')]])
    write('environment.json',dict(module='spec/conformance/production-input-graph/environment.afidl',
        type_schemas=['binder.fixture@1/Data','binder.fixture@1/Handle'],
        resource_schema='binder.fixture@1/Handle',resource_attributes=['region','tenant'],
        effect_catalogs=[dict(catalog='binder.effects@1',operations=['act','read']),dict(catalog='binder.empty@1',operations=[])],
        confidentiality=[dict(catalog='binder.labels@1',name='pii'),dict(catalog='binder.labels@1',name='public')],
        taints=[dict(catalog='binder.taints@1',name='tag-a')],
        sinks=[dict(catalog='binder.effects@1',name='act',decision=True),dict(catalog='binder.effects@1',name='read',decision=False)],
        meters=[dict(name='calls',aggregation='Additive'),dict(name='special',aggregation='Unsupported')],meter_catalog='binder.meters@1',unit=dict(catalog='binder.units@1',name='count'),
        reconciliation=[dict(catalog='binder.reconcile@1',name='settle')],
        capability=dict(catalog='binder.capabilities@1',name='compute'),versions=['1','2'],attributes=[dict(key='arch',value='x64'),dict(key='os',value='windows')],
        trust_domains=[dict(catalog='binder.trust@1',name='worker-a'),dict(catalog='binder.trust@1',name='worker-b')],
        regions=[dict(catalog='binder.regions@1',name='sg'),dict(catalog='binder.regions@1',name='us')],
        profile=dict(catalog='binder.runtime@1',name='worker'),family='static',features=['alpha','beta']))
    write('manifest.json',dict(start_head='5ef4e56a3ad5ef9cc3fe050dcbe9b98f678476ea',analysis='agentflow.root-export-checker@8',model='agentflow.computation.proof-report.model@7',case_ids=[c['case_id'] for c in cases],raw_source_vocabulary=len(vocabulary),bundles=len(bundles),production_cases=len(cases),registered_cases_credit=0,manual_subgraph_credit=0,old_boundary_mutations_credit=0))
    print(json.dumps(dict(cases=len(cases),vocabulary=len(vocabulary),bundles=len(bundles))))
if __name__=='__main__':main()
