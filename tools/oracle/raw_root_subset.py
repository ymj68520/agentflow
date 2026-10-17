"""Independent registered ROOT SUBSET oracle; never a full Fact oracle.

Default exit remains 2 even when every implemented subset instance matches.
Missing complete compiler/context/key/relation closure is not an empty success.
"""
import copy
import importlib.util
import json
from pathlib import Path
import sys

R = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('root_boundary_audit', R/'tools/audit-raw-full-field-boundary.py')
A = importlib.util.module_from_spec(spec)
spec.loader.exec_module(A)
M = A.model()
T = M['types']
ROOTS = [name for name, _ in T['Fact']['fields']]
IMPLEMENTED = ['role', 'evaluation', 'status', 'prerequisites', 'cause_fact_keys',
               'companion_evidence_status', 'substitution_basis', 'native_guard_basis']
assert set(IMPLEMENTED) < set(ROOTS) and len(ROOTS) == 14


def selected_document(v):
    owner = v['package']['owners'][v['owner']]
    kind, container = next(iter(owner['container'].items()))
    field = {'declaration_bundle': 'declarations', 'import_manifest': 'imports',
             'workflow_source_collection': 'sources'}[kind]
    return owner['anchor'], container[field][v['slot']]['document']


def expected():
    inputs, failures = A.load_inputs()
    roster, _, _ = A.events(inputs, failures)
    source = {v['case_id']: v for v in inputs}
    rows = []
    for e in roster:
        v = source[e['case_id']]
        o = e['occurrence']
        owner, document = selected_document(v)
        def field(name, member):
            assert member in dict(T[name]['fields'])
            return {'field': {'owner': {'owner': M['model_id'], 'name': name}, 'field': member}}
        path = [field('RawValidationSubject', 'input'), field('RawParsedInput', 'candidate'),
                field('Raw(af-contract::ComponentContract)', o['direction']+'s'),
                {'entry': {'index': o['original_ordinal']}},
                field('Raw(af-contract::ComponentInputPort)' if o['direction']=='input'
                      else 'Raw(af-contract::ComponentOutputPort)', 'port_no')]
        roots = {'role': 'supporting_fact', 'evaluation': {'evaluated': None},
                 'status': {'proof': 'Proven' if e['condition']=='positive_fresh' else 'Violation'},
                 'prerequisites': [], 'cause_fact_keys': [],
                 'companion_evidence_status': {'none': None},
                 'substitution_basis': {'none': None}, 'native_guard_basis': {'none': None}}
        subject = None if e['condition']=='positive_fresh' else {
            'source_package': v['package'], 'owner': owner, 'slot': v['slot'],
            'document': document, 'candidate': v['candidate']}
        rows.append({'case_id': e['case_id'], 'occurrence': o, 'registered_roots': roots,
                     'registered_declared_path': path, 'witness_subject_input': subject,
                     'complete_fact': False, 'root_export': False})
    return rows


def compare(reference, want, got):
    # The independent reference roster survives simultaneous deletions in the
    # other two views. Presence/type/value are checked, not same-shape equality.
    assert type(want) is list and type(got) is list
    assert len(reference) == len(want) == len(got) == 125
    for ref, exp, act in zip(reference, want, got):
        assert type(exp) is dict and type(act) is dict
        assert set(ref) == set(exp) == set(act)
        assert set(ref['registered_roots']) == set(exp['registered_roots']) == set(act['registered_roots']) == set(IMPLEMENTED)
        # JSON tokens distinguish bool/int, absent/null and option tags. The
        # comparison is typed recursion, not opaque serialization equality.
        def typed_equal(a, b):
            if type(a) is not type(b): return False
            if isinstance(a, dict): return set(a)==set(b) and all(typed_equal(a[k],b[k]) for k in a)
            if isinstance(a, list): return len(a)==len(b) and all(typed_equal(x,y) for x,y in zip(a,b))
            return a == b
        assert typed_equal(exp, ref), 'independent Expected corrupted'
        assert typed_equal(act, ref), 'typed subset mismatch'


def mutations(reference, actual):
    rows=[]
    index=next(i for i,r in enumerate(reference) if r['witness_subject_input'] is not None)
    def reject(name, mutation, both=False):
        exp=copy.deepcopy(reference);act=copy.deepcopy(actual);mutation(act[index])
        if both:mutation(exp[index])
        try:compare(reference, exp, act)
        except AssertionError:
            rows.append({'mutation_id':'c2r2-subset/'+name,'rejected':True,'both_sides':both});return
        raise AssertionError('accepted mutation '+name)
    for name in IMPLEMENTED:
        reject('missing-'+name,lambda r,n=name:r['registered_roots'].pop(n))
        reject('both-missing-'+name,lambda r,n=name:r['registered_roots'].pop(n),True)
        reject('wrong-type-'+name,lambda r,n=name:r['registered_roots'].__setitem__(n,42))
    reject('missing-event',lambda r:r.clear())
    reject('both-missing-event',lambda r:r.clear(),True)
    reject('wrong-owner',lambda r:r['registered_declared_path'][0]['field']['owner'].update(owner='agentflow.computation.proof-report.model@4'))
    reject('wrong-nominal-name',lambda r:r['registered_declared_path'][0]['field']['owner'].update(name='RawParsedInput'))
    reject('path-original-ordinal',lambda r:r['registered_declared_path'][3]['entry'].update(index=999))
    reject('missing-path-owner',lambda r:r['registered_declared_path'][0]['field']['owner'].pop('owner'))
    reject('both-missing-path-owner',lambda r:r['registered_declared_path'][0]['field']['owner'].pop('owner'),True)
    for key in ['owner_key','revision_key']:
        reject('witness-'+key,lambda r,k=key:r['witness_subject_input']['owner'].__setitem__(k,[122]))
    for key in ['namespace','domain_key']:
        reject('witness-'+key,lambda r,k=key:r['witness_subject_input']['owner']['domain'].__setitem__(k,[122]))
    reject('witness-slot',lambda r:r['witness_subject_input'].__setitem__('slot',1))
    reject('witness-subject-missing',lambda r:r.__setitem__('witness_subject_input',None))
    reject('false-complete',lambda r:r.__setitem__('complete_fact',True))
    reject('root-capability',lambda r:r.__setitem__('root_export',True))
    reject('extra-field',lambda r:r['registered_roots'].__setitem__('unregistered',True))
    return rows


def report(actual):
    ref=expected();compare(ref,copy.deepcopy(ref),actual);mut=mutations(ref,actual)
    return {'outcome':'D', 'implemented_subset':'PASS', 'full_field_conformance':False,
            'primary_events':125, 'compared_root_subset_instances':125*len(IMPLEMENTED),
            'required_formal_root_instances':125*len(ROOTS),
            'remaining_root_instances':125*(len(ROOTS)-len(IMPLEMENTED)),
            'implemented_fields':IMPLEMENTED, 'remaining_fields':[n for n in ROOTS if n not in IMPLEMENTED],
            'witness_subject_input_scope':'source package/owner/slot/document/candidate only; complete compiler/context/path projection NOT claimed',
            'mutations':mut, 'new_subset_mutations_rejected':len(mut),
            'new_full_field_mutations':0, 'strict_gate_exit':2,
            'new_full_field_case_ids':0, 'new_real_input_sensitivity_cases':0,
            'full_nested_coverage':False, 'full_relation_coverage':False}


def sensitivity_vectors():
    spec=importlib.util.spec_from_file_location('c2r2_input_lift',R/'tools/oracle/raw_port_capture.py')
    old=importlib.util.module_from_spec(spec);spec.loader.exec_module(old)
    inputs=old.vectors()
    base = copy.deepcopy(next(v for v in inputs if v['case_id']=='c2r1/declaration_bundle/input-duplicate'))
    rows=[]
    def add(label, mutate):
        v=copy.deepcopy(base);v['case_id']='c2r2-input/'+label;v['operation']='capture';mutate(v);rows.append(v)
    def sync(v):
        # Registered Raw lifting is input construction only, not Actual-driven
        # Expected. The old input/oracle module and all old Expected stay intact.
        v['candidate']=old.lift(T['RawCandidateContract'],v['wire_candidate'])
        doc=selected_document(v)[1]
        doc['declared_native_document']['declarations'][v['path_entry']]=copy.deepcopy(v['candidate'])
    add('baseline',lambda v:None)
    for name in ['namespace','domain_key']:
        def change(v,n=name):
            v['package']['domain'][n]=list(('changed-'+n).encode())
            v['package']['owners'][0]['anchor']['domain'][n]=list(('changed-'+n).encode())
        add(name,change)
    for name in ['owner_key','revision_key']:
        add(name,lambda v,n=name:v['package']['owners'][0]['anchor'].__setitem__(n,list(('changed-'+n).encode())))
    add('document-slot',lambda v:v.__setitem__('slot',1))
    def entry(v):
        doc=selected_document(v)[1];doc['declared_native_document']['declarations'].append(copy.deepcopy(v['candidate']));v['path_entry']=1
    add('internal-declaration-entry',entry)
    def ports(v,operation):
        operation(v['wire_candidate']);sync(v)
    add('direction',lambda v:ports(v,lambda w:w.update(inputs=copy.deepcopy(w['outputs']),outputs=copy.deepcopy(w['inputs']))))
    add('original-ordinal',lambda v:ports(v,lambda w:w['inputs'].insert(0,w['inputs'].pop(1))))
    add('sorted-ordinal',lambda v:ports(v,lambda w:w['inputs'].insert(0,dict(copy.deepcopy(w['inputs'][0]),port_no=2))))
    add('port-number',lambda v:ports(v,lambda w:w['inputs'][0].__setitem__('port_no',2)))
    add('predecessor',lambda v:ports(v,lambda w:w['inputs'].insert(0,dict(copy.deepcopy(w['inputs'][0]),port_no=4))))
    add('typed-declaration',lambda v:ports(v,lambda w:w['component_identity'].__setitem__('name','different-component')))
    def physical(v,padding):
        anchor,doc=selected_document(v);doc.clear();doc['physical_json_utf8']={'bytes':list((padding+json.dumps([v['candidate']],separators=(',',':'))+padding).encode())};v['physical']='matching'
    add('physical-base',lambda v:physical(v,''))
    add('physical-bytes-whitespace',lambda v:physical(v,' \n'))
    add('context-association',lambda v:v.__setitem__('operation','other_context'))
    assert len(rows)==16
    return rows


def sensitivity_expected():
    # This separate independent prefix calculation supports the new real input
    # corpus without modifying the frozen 87-case Expected or its roster.
    result=[]
    for v in sensitivity_vectors():
        if v['operation']=='other_context':
            result.append({'case_id':v['case_id'],'failure':'InvalidContextBinding','events':[]});continue
        events=[];stop=False
        for direction,field in [('input','inputs'),('output','outputs')]:
            previous=None
            if stop:break
            for sorted_i,(original,port) in enumerate(sorted(enumerate(v['wire_candidate'][field]),key=lambda pair:pair[1]['port_no'])):
                number=port['port_no'];condition='zero' if number==0 else 'duplicate' if previous and previous['port_no']==number else 'positive_fresh'
                events.append({'occurrence':{'direction':direction,'original_ordinal':original,'sorted_ordinal':sorted_i,'port_no':number},'predecessor':previous,'condition':condition})
                if condition!='positive_fresh':stop=True;break
                previous={'original_ordinal':original,'port_no':number}
        owner,document=selected_document(v)
        result.append({'case_id':v['case_id'],'failure':None,'events':events,
                       'source_package':v['package'],'owner':owner,'slot':v['slot'],
                       'document':document,'candidate':v['candidate'],
                       'internal_path':[{'entry':v['path_entry']}],
                       'context_roles':[['analysis','canonical_validation']],
                       'root_export':False,'complete_scope':False})
    return result


def sensitivity_report(actual):
    ref=sensitivity_expected()
    def flatten(value,path=()):
        out={path:('object' if isinstance(value,dict) else 'array' if isinstance(value,list) else type(value).__name__,None if isinstance(value,(dict,list)) else value)}
        for k,v in (value.items() if isinstance(value,dict) else enumerate(value) if isinstance(value,list) else []):out.update(flatten(v,path+(k,)))
        return out
    assert len(actual)==len(ref)==16
    for e,a in zip(ref,actual):assert flatten(e)==flatten(a),e['case_id']
    baseline=flatten(ref[0]);observed=flatten(actual[0]);rows=[]
    for e,a in zip(ref[1:],actual[1:]):
        exp=flatten(e);got=flatten(a);keys=set(baseline)|set(exp)
        changed={p for p in keys if p!=('case_id',) and baseline.get(p)!=exp.get(p)}
        unchanged={p for p in keys if p!=('case_id',) and baseline.get(p)==exp.get(p)}
        actual_keys=set(observed)|set(got)
        achanged={p for p in actual_keys if p!=('case_id',) and observed.get(p)!=got.get(p)}
        aunchanged={p for p in actual_keys if p!=('case_id',) and observed.get(p)==got.get(p)}
        assert changed==achanged and unchanged==aunchanged and changed
        path_list=lambda values:[list(p) for p in sorted(values,key=lambda p:tuple(str(x) for x in p))]
        rows.append({'case_id':e['case_id'],'expected_changed_fields':path_list(changed),'actual_changed_fields':path_list(achanged),
                     'expected_unchanged_fields':path_list(unchanged),'actual_unchanged_fields':path_list(aunchanged),'matched':True})
    # Isolate bytes alone too: same parsed declaration, changed original bytes.
    assert ref[13]['candidate']==ref[14]['candidate'] and ref[13]['document']!=ref[14]['document']
    return {'new_input_cases':16,'sensitivity_comparisons':15,'comparisons':rows,
            'physical_bytes_only_pair':[ref[13]['case_id'],ref[14]['case_id']],
            'compiled_constraint_sensitivity':'NOT_EXECUTED',
            'complete_context_contents_sensitivity':'NOT_EXECUTED',
            'scope':'LOCAL_SOURCE_AND_EVENT_FIELDS_NOT_FULL_FORMAL_FACT',
            'full_field_conformance':False}


if __name__=='__main__':
    if sys.argv[1:]==['--sensitivity-vectors']:
        print(json.dumps(sensitivity_vectors(),ensure_ascii=True));sys.exit(0)
    if sys.argv[1:]==['--sensitivity-report']:
        print(json.dumps(sensitivity_report(json.load(sys.stdin)),ensure_ascii=True,indent=2));sys.exit(0)
    result=report(json.load(sys.stdin))
    print(json.dumps(result,ensure_ascii=True,indent=2))
    sys.exit(0 if sys.argv[1:]==['--report'] else 2)
