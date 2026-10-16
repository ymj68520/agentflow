"""Independent F1 binary/nominal/required-field checks; NOT full T3 oracle.

Frozen original native semantics/Expected are read-only. Python base64 and
independent AFIDL/ACE1 authoring determine Expected, never Rust Actual.
"""
import base64
import copy
import importlib.util
import json
import re
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location("f1_frozen_native",ROOT/"tools/oracle/native_guard_capture.py")
NG=importlib.util.module_from_spec(spec);spec.loader.exec_module(NG)
INVALID=["=","AQ==","+w","/w","A","AB","AQ\n","AQ ","é","AA=","AA/","\0"]


def strict_decode(text):
    if not isinstance(text,str) or re.fullmatch(r'[A-Za-z0-9_-]*',text) is None:raise ValueError('alphabet/padding')
    raw=base64.b64decode(text+'='*((-len(text))%4),altchars=b'-_',validate=True)
    if base64.urlsafe_b64encode(raw).decode().rstrip('=')!=text:raise ValueError('noncanonical bits')
    return list(raw)


def binary_inputs():
    rows=[('empty',[]),('single',[1]),('multi-zero',[0,1,2,255]),('zeroes',[0,0,0]),
          ('non-utf8',[255,254]),('utf8-bytes',[104,105]),('url-alphabet',[251,255,191])]
    for n in [2,3,255,256,257]:rows.append((f'block-{n}',[i%256 for i in range(n)]))
    return [(n,base64.urlsafe_b64encode(bytes(b)).decode().rstrip('='),b) for n,b in rows]


def lowering_vectors():
    result=[{'case_id':'bytes-f1/lowering/'+name,'constant':{'bytes':text}} for name,text,_ in binary_inputs()]
    result += [{'case_id':'bytes-f1/lowering/string-'+str(i),'constant':{'string':s}} for i,s in enumerate(['','AQ'])]
    result += [{'case_id':'bytes-f1/lowering/invalid-'+str(i),'constant':{'bytes':s}} for i,s in enumerate(INVALID)]
    return result


def native_vectors():
    result=[]
    def add(name,constant,primitive,mode,valid=True):
        c=NG.base(primitive);es=c['effects']['row']['closed'];es[1]['guard']={'input_equals':{'port_no':1,'expected':constant}}
        if mode=='contained':es[0]['guard']=copy.deepcopy(es[1]['guard'])
        if mode=='zero':c['recovery_usage_bindings'][0]['attempts_per_activation']=NG.bounds(0,0)
        if valid and 'bytes' in constant:strict_decode(constant['bytes'])
        v={'case_id':'bytes-f1/native/'+name,'raw_declaration':c,'schema_primitive':primitive,
           'canonical_admission':valid,'rust_input':NG.rust_input(c)}
        # Fixture-wide Checked is NOT inferred from a Native P. The frozen
        # AF-CONTRACT-0016 validator additionally requires an actual attempt
        # for possible activation (validate.rs check_recovery_usage). The
        # zero-attempt diagnostic deliberately fails that existing premise.
        # This is a reviewed fixture premise, not a full independent D6 oracle.
        usage_ready=all(NG.finite(u['activations_per_forward']['upper'])==0 or u['attempts_per_activation']['lower']>=1
                        for u in c['recovery_usage_bindings'])
        native_ready=valid and all(r['record']['verdict']=='proven' for r in NG.expected(v).get('runs',[]))
        v['contract_checked_expected']=valid and usage_ready and native_ready
        result.append(v)
    for name,text,_ in binary_inputs()[:7]:
        for mode in ['countermodel','contained','zero']:add(name+'-'+mode,{'bytes':text},'bytes',mode)
    add('type-mismatch',{'bytes':'AQ'},'string','type_mismatch')
    add('string-same-printable-countermodel',{'string':'AQ'},'string','countermodel')
    add('string-same-printable-contained',{'string':'AQ'},'string','contained')
    for i,text in enumerate(INVALID):add('invalid-'+str(i),{'bytes':text},'bytes','invalid',False)
    return result


def lower_expected(v):
    source=v['constant'];kind,text=next(iter(source.items()))
    try:payload={'kind':kind,'value':strict_decode(text) if kind=='bytes' else text}
    except (ValueError,base64.binascii.Error):payload=None
    valid=payload is not None
    signature={'row':{'closed':[{'atom':NG.CC.atom(name='read',qualifier=None),'bounds':NG.bounds(0,1),
                                'guard':{'input_equals':{'port_no':1,'expected':source}}}]}}
    return NG.wire_numbers({'case_id':v['case_id'],'input_constant':source,'canonical_admission':valid,
        'typed_payload':payload,'decoded_payload':payload,'scalar_ace1_utf8':json.dumps(text,ensure_ascii=False,separators=(',',':')) if valid else None,
        'signature_ace1_utf8':NG.CC.ace1(signature) if valid else None,
        'rejection_kind':None if valid else 'invalid_effect_guard'})


LOWER_FIELDS={'case_id','input_constant','canonical_admission','typed_payload','decoded_payload','scalar_ace1_utf8','signature_ace1_utf8','rejection_kind'}
PAYLOAD_FIELDS={'kind','value'}
NATIVE_FIELDS={'case_id','actual','pipeline'}
PIPELINE_FIELDS={'canonical_validation_pass','canonical_encoding_pass','contract_check_invoked','contract_checked','native_entry_invoked','native_record_count'}


def record(x,fields):
    if not isinstance(x,dict) or set(x)!=fields:raise ValueError('missing/extra required field')


def lower_shape(x):
    record(x,LOWER_FIELDS)
    if not isinstance(x['case_id'],str):raise ValueError('case id required')
    source=x['input_constant']
    if not isinstance(source,dict) or len(source)!=1 or next(iter(source)) not in ('bytes','string') or not isinstance(next(iter(source.values())),str):raise ValueError('closed source constant required')
    if type(x['canonical_admission']) is not bool:raise ValueError('boolean required')
    for k in ['typed_payload','decoded_payload']:
        p=x[k]
        if x['canonical_admission']:
            record(p,PAYLOAD_FIELDS)
            if p['kind']=='bytes':
                if not isinstance(p['value'],list) or any(not isinstance(v,str) or not re.fullmatch(r'0|[1-9][0-9]*',v) or not 0<=int(v)<=255 for v in p['value']):raise ValueError('byte width')
            elif p['kind']=='string':
                if not isinstance(p['value'],str):raise ValueError('string payload')
            else:raise ValueError('unknown nominal payload')
        elif p is not None:raise ValueError('invented payload on rejection')
    if x['canonical_admission']:
        if not isinstance(x['scalar_ace1_utf8'],str) or not isinstance(x['signature_ace1_utf8'],str) or x['rejection_kind'] is not None:raise ValueError('encoded fields required')
    elif x['scalar_ace1_utf8'] is not None or x['signature_ace1_utf8'] is not None or x['rejection_kind']!='invalid_effect_guard':raise ValueError('invalid rejected record')


def compare_lower(want,got):
    lower_shape(want);lower_shape(got)
    difference=NG.first_difference(want,got)
    if difference:raise ValueError(difference)


def mutations():
    template=lower_expected(next(v for v in lowering_vectors() if v['case_id'].endswith('/single')))
    rows=[]
    def run(name,mutate,both=False):
        a=copy.deepcopy(template);b=copy.deepcopy(template);mutate(b)
        if both:mutate(a)
        try:compare_lower(a,b)
        except (ValueError,AssertionError,KeyError,TypeError):rows.append({'case_id':'bytes-f1/mutation/'+name,'rejected':True});return
        raise AssertionError('accepted mutation '+name)
    run('bytes-to-string-same-scalar',lambda x:x.update(typed_payload={'kind':'string','value':'AQ'}))
    run('payload-a-to-b',lambda x:x['typed_payload'].update(value=['2']))
    run('decoded-tag',lambda x:x.update(decoded_payload={'kind':'string','value':'AQ'}))
    run('byte-out-of-range',lambda x:x['typed_payload'].update(value=['256']))
    run('wrong-scalar-wire',lambda x:x.update(scalar_ace1_utf8='"Ag"'))
    run('wrong-signature-wire',lambda x:x.update(signature_ace1_utf8='{}'))
    run('wrong-source-tag',lambda x:x.update(input_constant={'string':'AQ'}))
    run('wrong-admission',lambda x:x.update(canonical_admission=False))
    for field in sorted(LOWER_FIELDS):run('both-delete-'+field,lambda x,f=field:x.pop(f),True)
    for parent in ['typed_payload','decoded_payload']:
        for field in sorted(PAYLOAD_FIELDS):run('both-delete-'+parent+'-'+field,lambda x,p=parent,f=field:x[p].pop(f),True)
    # Required imported fields are taken from the frozen registered model,
    # not inferred from whichever members survived in Expected/Actual.
    shape_spec=importlib.util.spec_from_file_location('f1_import_shape',ROOT/'tools/native_guard_diagnostic_shape.py')
    shape=importlib.util.module_from_spec(shape_spec);shape_spec.loader.exec_module(shape)
    native_case=next(v for v in NG.vectors() if v['case_id']=='r3-ng/primitive-bytes-1')
    witness=NG.expected(native_case)['runs'][0]['record']['countermodel']
    def compare_witness(a,b):
        shape.convert({'ref':'ImportedTypedGuardCountermodel'},a)
        shape.convert({'ref':'ImportedTypedGuardCountermodel'},b)
        if (d:=NG.first_difference(a,b)):raise ValueError(d)
    def witness_mutation(name,mutate,both=False):
        a=copy.deepcopy(witness);b=copy.deepcopy(witness);mutate(b)
        if both:mutate(a)
        try:compare_witness(a,b)
        except (ValueError,AssertionError,KeyError,TypeError):rows.append({'case_id':'bytes-f1/mutation/'+name,'rejected':True});return
        raise AssertionError('accepted formal mutation '+name)
    owner=shape.m['types']['ImportedTypedGuardCountermodel']['ref']
    fields=[name for name,_ in shape.m['types'][owner]['fields']];assert len(fields)==14
    for field in fields:witness_mutation('both-delete-countermodel-'+field,lambda x,f=field:x.pop(f),True)
    assignment_owner=owner.rsplit('/',1)[0]+'/af-contract::GuardInputAssignment'
    fields=[name for name,_ in shape.m['types'][assignment_owner]['fields']];assert len(fields)==3
    for field in fields:witness_mutation('both-delete-assignment-'+field,lambda x,f=field:x['assignments'][0].pop(f),True)
    witness_mutation('both-delete-assignment-bytes-case',lambda x:x['assignments'][0]['value'].pop('bytes'),True)
    witness_mutation('assignment-bytes-to-string',lambda x:x['assignments'][0].update(value={'string':''}))
    witness_mutation('assignment-wrong-bytes',lambda x:x['assignments'][0].update(value={'bytes':'Ag'}))
    return rows


def check_original(actual,stages):
    frozen=json.loads((ROOT/'spec/conformance/g01-t3-native-capture/expected-traces.json').read_bytes())
    assert len(actual)==len(frozen)==len(stages)==53
    want={v['case_id']:v['expected_trace'] for v in frozen};seen=set()
    for r in actual:
        assert r['case_id'] not in seen;seen.add(r['case_id'])
        got=copy.deepcopy(r['actual'])
        for run in got.get('runs',[]):run['record'].pop('reason')
        assert NG.first_difference(want[r['case_id']],got) is None,r['case_id']
    # Full original independent semantics remain unchanged and must also pass.
    independent=NG.compare_all(actual);assert independent['unexpected_mismatch']==0
    for case_id in ['r3-ng/primitive-bytes-0','r3-ng/primitive-bytes-1']:
        stage=next(s for s in stages if s['case_id']==case_id)
        assert stage['canonical_validation_pass'] and stage['canonical_encoding_pass'] and stage['contract_check_invoked']
        assert not stage['contract_checked'] and stage['contract_rejection']['overall']=='violation'
        row=next(r for r in actual if r['case_id']==case_id)
        assert row['actual']['runs'][0]['branch']=='typed_countermodel'
    return independent


def check_supplement(actual):
    assert set(actual)=={'lowering','native'}
    low={v['case_id']:v for v in lowering_vectors()};native={v['case_id']:v for v in native_vectors()}
    assert len(actual['lowering'])==len(low) and len(actual['native'])==len(native)
    seen=set()
    for r in actual['lowering']:
        assert r['case_id'] not in seen;seen.add(r['case_id']);compare_lower(lower_expected(low[r['case_id']]),r)
    seen=set()
    for r in actual['native']:
        record(r,NATIVE_FIELDS);record(r['pipeline'],PIPELINE_FIELDS)
        assert r['case_id'] not in seen;seen.add(r['case_id']);v=native[r['case_id']]
        got=copy.deepcopy(r['actual'])
        for run in got.get('runs',[]):run['record'].pop('reason')
        assert NG.first_difference(NG.expected(v),got) is None,r['case_id']
        p=r['pipeline'];valid=v['canonical_admission']
        assert p['canonical_validation_pass']==valid and p['canonical_encoding_pass']==valid
        assert p['native_entry_invoked']==valid and p['contract_check_invoked']
        assert p['contract_checked']==v['contract_checked_expected'],r['case_id']+' checked'
        assert p['native_record_count']==('1' if valid else '0')
    return {'lowering_cases':len(low),'native_pipeline_cases':len(native),'mutations':mutations(),
            'unexpected_mismatch':0,'full_t3_seed_conformance':False,'t4_eligible':False,'positive_root_export':'NotAuthorized'}


def main():
    import sys
    if sys.argv[1:]==['--vectors']:
        print(json.dumps({'lowering':lowering_vectors(),'native':native_vectors()},ensure_ascii=True));return
    actual=json.load(sys.stdin)
    if sys.argv[1:]==['--original-53']:
        result=check_original(actual['actual'],actual['stages'])
    else:result=check_supplement(actual)
    print(json.dumps(result,ensure_ascii=True,sort_keys=True))


if __name__=='__main__':main()
