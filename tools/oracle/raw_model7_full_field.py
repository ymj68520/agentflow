"""Model@7 R5 strict gate. Boundary evidence is NOT full Raw conformance.

Expected is reconstructed from source/AFIDL/registered descriptors, never Actual.
At the demonstrated C-source nominal binding gap, no complete Fact is fabricated.
The strict Total remains nonzero even if every boundary comparison succeeds.
"""
import argparse,copy,hashlib,importlib.util,json,sys
from pathlib import Path
R=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(R/'tools'))
from oracle import production_input_graph as G
from oracle.structural_binding_registration import catalogue,nominal
P='phase-04c1b4c4r3c2r5-'
SLOTS=['effect_resource','recovery_usage','flow_execution']

def read(p):return json.loads((R/p).read_bytes())
def write(name,v):
    p=R/'docs/implementation'/(P+name+'.json')
    p.write_bytes((json.dumps(v,indent=2,ensure_ascii=False)+'\n').encode())
def module(name,path):
    spec=importlib.util.spec_from_file_location(name,R/path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
def exact(a,b,path=()):
    assert type(a) is type(b),('type',path)
    if isinstance(a,dict):
        assert set(a)==set(b),('fields',path,set(a)^set(b))
        for k in a:exact(a[k],b[k],path+(k,))
    elif isinstance(a,list):
        assert len(a)==len(b),('length',path)
        for i,(x,y) in enumerate(zip(a,b)):exact(x,y,path+(i,))
    else:assert a==b,('value',path,a,b)
def plan():
    """Descriptors, not compiler source order or failed Actual, choose this path."""
    m=catalogue();t=m['types'];context=dict(t['ContractProofContextContents']['fields'])
    relation=context['relation_snapshots'];rows=dict(relation['fields'])
    for slot in SLOTS:
        assert rows[slot]['kind']=='option'
        assert rows[slot]['item']=={'ref':'RelationSnapshotContent'}
    assert t['RelationSnapshotContent']=={'ref':'NativeRelationSnapshotSource'}
    required=dict(t['CurrentContextRelationSchemaValidation']['fields'])['source']
    assert required=={'kind':'dag_ref','target':{'ref':'NativeRelationSnapshotSource'}}
    policies=[p for p in m['structural_binding_registration']['reference_policies']
        if p['key']==dict(parent=nominal('CurrentContextRelationSchemaValidation'),path=[{'field':'source'}])]
    assert {p['environment'] for p in policies}=={'output_record','w_value'}
    assert all(p['domain']=='g_external' and p['target']==nominal('NativeRelationSnapshotSource') for p in policies)
    return dict(finding_id='R5-C-REL-NOMINAL-01',first_unbound_field='StructuralSubjectAdmission.observations[0].source',
        required_formal_type=required,required_nominal=nominal('NativeRelationSnapshotSource'),
        actual_source_type='af-contract::RelationCatalogSnapshot<T>',
        registered_g_path='RawValidationSubject.context.contents.relation_snapshots.<slot>.some',
        registered_g_nominal=nominal('RelationSnapshotContent'),
        alias_body=t['RelationSnapshotContent'],reference_policies=policies,
        governing_rules=['AF-SEMANTIC-0295','AF-SEMANTIC-0297','AF-SEMANTIC-0303','AF-SEMANTIC-0306','AF-SEMANTIC-0308'],
        root_profile='one RawValidationSubject root; no extra roots/unreachable/retagged nodes',
        binder_defect_claimed=False,native_false_proven_claimed=False,
        reason='Alias delegates body but retains nominal node identity; no registered nominal alias cast or second-root allocation is permitted.')
def field_node(graph,rank,field):
    return int(next(r['value'] for r in graph['nodes'][rank]['body']['record'] if r['field']==field))
def boundary_expected():
    source=next(v for v in G.RAW.vectors() if v['case_id']=='c2r1/declaration_bundle/input-zero')
    reference=[];counts=[]
    for selected in SLOTS+['none']:
        for mode in (['valid'] if selected=='none' else ['valid','missing']):
            c_results={}
            for slot in SLOTS:
                case=G.REL.case_base(slot);case['present']=selected==slot
                c_results[slot]=G.REL.expected(case,module_selection=(selected if selected!='none' else SLOTS[0], 'A' if mode=='valid' else 'missing_module'))['computed']
            c=dict(**c_results,relation_partition_complete=mode=='valid',scope_state='remaining_scope_incomplete')
            context=G.empty_context_with_actual_modules()
            context=copy.deepcopy(context)
            context['contents']['compiled_modules']=[[row['specification_id'],row] for row in c_results[SLOTS[0]]['current_modules']]
            for slot in SLOTS:context['contents']['relation_snapshots'][slot]=copy.deepcopy(c_results[slot]['source'])
            declared=G.subject(source,context)
            b=G.Builder();graph=b.finish(b.build('RawValidationSubject',declared))
            graph_file=f'target/{P}{selected}-{mode}.ace1'
            actual=read(graph_file)
            print('Comparing complete boundary G:',selected,mode,file=sys.stderr,flush=True)
            compared=G.strict_compare(declared,graph,actual)
            expected_bytes=G.canonical_bytes(graph);actual_bytes=(R/graph_file).read_bytes();assert expected_bytes==actual_bytes
            (R/'target'/(P+f'{selected}-{mode}-expected.ace1')).write_bytes(expected_bytes)
            root=int(graph['roots'][0]);context_rank=field_node(graph,field_node(graph,root,'context'),'contents')
            snapshots=field_node(graph,context_rank,'relation_snapshots')
            found=[i for i,n in enumerate(graph['nodes']) if n['type']==nominal('NativeRelationSnapshotSource')]
            assert found==[]
            actual_source=None
            if selected!='none':
                opt=field_node(graph,snapshots,selected)
                rank=int(graph['nodes'][opt]['body']['option_some'])
                assert graph['nodes'][rank]['type']==nominal('RelationSnapshotContent')
                actual_source=dict(rank=rank,type=nominal('RelationSnapshotContent'))
            ref=dict(case_id=f'r5/nominal-boundary/{selected}/{mode}',selected=selected,mode=mode,input=source,
                graph=graph_file,subject_rank=root,context_rank=context_rank,actual_source=actual_source,
                required_c_source_type=nominal('NativeRelationSnapshotSource'),matching_target_ranks=found,
                c_metadata=c,same_source_object=True,same_context_object=True,same_subject_carrier=True,
                g_unchanged_after_c_and_port=True,
                port_event=dict(direction='input',original_ordinal=0,sorted_ordinal=0,port_no=0,condition='zero',status='Violation',rule='AF-CONTRACT-0004'),
                complete_raw_fact_produced=False,positive_root_export=False)
            reference.append(ref);counts.append(dict(case_id=ref['case_id'],coverage=compared,
                actual_sha256=hashlib.sha256(actual_bytes).hexdigest(),raw_full_field_credit=0))
    return reference,counts
def compare(reference,want,actual):
    exact(reference,want);exact(reference,actual)
def mutations(reference):
    paths=[('case_id',),('subject_rank',),('context_rank',),('actual_source','rank'),('actual_source','type','owner'),
        ('actual_source','type','name'),('required_c_source_type','name'),('matching_target_ranks',),
        ('c_metadata','effect_resource','source'),('c_metadata','effect_resource','lookup'),
        ('c_metadata','effect_resource','validation'),('c_metadata','effect_resource','current_modules'),
        ('same_source_object',),('same_context_object',),('same_subject_carrier',),('g_unchanged_after_c_and_port',),
        ('port_event','original_ordinal'),('port_event','port_no'),('port_event','condition'),('port_event','status'),
        ('complete_raw_fact_produced',),('positive_root_export',)]
    rows=[]
    for path in paths:
        for both in [False,True]:
            want=copy.deepcopy(reference);got=copy.deepcopy(reference)
            for view in ([want,got] if both else [got]):
                at=view[0]
                for p in path[:-1]:at=at[p]
                del at[path[-1]]
            try:compare(reference,want,got)
            except AssertionError:rows.append(dict(case_id='r5/boundary-comparator/'+'/'.join(path)+('/both' if both else '/actual'),rejected=True,both_sides=both))
            else:raise AssertionError(('accepted omission',path))
    for key in ['complete_raw_fact_produced','positive_root_export']:
        got=copy.deepcopy(reference);got[0][key]=True
        try:compare(reference,reference,got)
        except AssertionError:rows.append(dict(case_id='r5/boundary-comparator/forged-'+key,rejected=True,both_sides=False))
        else:raise AssertionError(key)
    return rows
def historical_roster():
    old=module('r5_frozen_raw_event_inputs','tools/audit-raw-full-field-boundary.py')
    inputs,failures=old.load_inputs();main,_,_=old.events(inputs,failures)
    sensitivity=module('r5_frozen_sensitivity_inputs','tools/oracle/raw_root_subset.py')
    development=[dict(case_id=r['case_id'],**e) for r in sensitivity.sensitivity_expected() for e in r.get('events',[])]
    legacy=[dict(case_id='c2r3/relation-snapshot-schema-pin/'+tag,condition='zero',
        occurrence=dict(direction='input',original_ordinal=0,sorted_ordinal=0,port_no=0),
        coordinate_status='SOURCE-DERIVED REQUIRED; old Actual preserved verdict only') for tag in ['left-A','right-A','missing']]
    last=[dict(case_id='c2r4/model6-structural-node-binding',condition='zero',
        occurrence=dict(direction='input',original_ordinal=0,sorted_ordinal=0,port_no=0))]
    assert len(main)==125 and len(development)==48
    rows=main+development+legacy+last;assert len(rows)==177
    return rows
def materialize(reference,coverage):
    finding=plan();legacy=historical_roster();root_names=[n for n,_ in catalogue()['types']['Fact']['fields']]
    new=[dict(case_id=r['case_id'],event=r['port_event'],fact_applicability='C_FORBIDS_FACT' if r['mode']=='missing' else 'OTHERWISE_APPLICABLE',full_field_compared=False) for r in reference]
    applicable=176+sum(r['mode']=='valid' for r in reference)
    ledger=dict(outcome='C — NEW FORMAL BINDING / STAGE GAP',checkpoint='R5-C',
        baseline=dict(real_events=177,otherwise_applicable_facts=176,root_instances=2464,full_field_compared=0),
        events=dict(required=177+len(reference),boundary_compared=len(reference),legacy_not_reaccepted=177,
            old_main_case_ids=59,old_main_events=125,unexpected_boundary=0),
        roots=dict(required=applicable*len(root_names),compared=0,missing_or_partial=applicable*len(root_names),otherwise_applicable_events=applicable),
        nested=dict(required=None,compared=0,missing=None,status='UNKNOWN; stopped at R5-C, not zero'),
        relations=dict(required=None,compared_raw_fact_instances=0,missing=None,status='UNKNOWN Raw Fact denominator; boundary C comparisons separately recorded'),
        observations=dict(required=None,compared_raw_fact_instances=0,missing=None,status='UNKNOWN; no complete Fact issued'),
        source_associations=dict(required_raw_fact_instances=None,compared_raw_fact_instances=0,
            boundary_same_source_context_carrier_checks=7,status='BOUNDARY ONLY'),
        new_complete_capture_case_ids=0,new_full_field_mutations=0,
        unexpected_raw_semantic_mismatches=None,raw_mismatch_status='NOT COMPARED',
        strict_total_exit=2,raw_producer='INCOMPLETE',total_t3='INCOMPLETE',t4='NOT ELIGIBLE',positive_root_export='NotAuthorized')
    write('raw-model7-applicability-ledger',dict(legacy=legacy,new_boundary_cases=new,
        no_event_deleted=True,c_missing_legacy_event='retained; full Fact forbidden',otherwise_applicable_facts=applicable))
    write('raw-model7-full-expected',dict(status='INCOMPLETE; boundary Expected only, not full Fact Expected',facts=None,boundary=reference))
    write('raw-model7-full-actual',dict(status='INCOMPLETE; genuine boundary capture only, not full Fact Actual',facts=None,
        boundary=read('target/'+P+'boundary-actual.json')))
    write('raw-model7-full-coverage-ledger',ledger)
    write('raw-model7-comparator-mutations',dict(boundary_mutations=mutations(reference),raw_full_field_mutations=[],full_field_credit=0))
    write('formal-binding-gap',dict(**finding,actual_graph_checks=coverage,affected_slots=SLOTS,
        minimal_case=reference[0]['case_id'],minimum_some_slots=1,empty_slots_not_blocked_by_this_gap=True,
        complete_raw_facts=0,production_adapter_changed=False))
    return ledger
if __name__=='__main__':
    cli=argparse.ArgumentParser();cli.add_argument('--boundary',action='store_true');cli.add_argument('--materialize',action='store_true');cli.add_argument('--plan',action='store_true');cli.add_argument('--gate',choices=['event','root','nested','relation','admission','total']);args=cli.parse_args()
    if args.plan:print(json.dumps(plan(),indent=2));sys.exit(0)
    ref,coverage=boundary_expected();actual=read('target/'+P+'boundary-actual.json');compare(ref,ref,actual)
    mutation_rows=mutations(ref)
    result=materialize(ref,coverage) if args.materialize else dict(outcome='C',boundary_cases=len(ref),boundary_mutations_rejected=len(mutation_rows),raw_full_field=False,strict_total_exit=2)
    print(json.dumps(result,indent=2))
    sys.exit(0 if args.boundary else 2)
