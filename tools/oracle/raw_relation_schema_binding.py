"""Independent C2R3 binding diagnostic. No full-Fact conformance is claimed."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import sys

R = Path(__file__).resolve().parents[2]
FAMILY = 'agentflow.effect.resource.compatibility@1'
NAME = 'EffectResourceCompatibilityCatalog'
REVISION = 'binding-probe'


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, R/path)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


def expected():
    afidl = module('independent_relation_afidl', 'tools/gen-afidl-corpus.py')
    inputs = [(p.relative_to(R).as_posix(), p.read_text(encoding='utf8'))
              for p in sorted((R/'spec/idl').rglob('*.afidl'))]
    assert len(inputs) == 17
    changed = copy.deepcopy(inputs)
    index = next(i for i,(p,_) in enumerate(inputs) if p.endswith('effect-resource-compatibility.afidl'))
    old = '2 supported_pairs: list<SupportedEffectResourcePair>;'
    new = '2 supported_pairs: list<string>;'
    assert changed[index][1].count(old) == 1
    changed[index] = (changed[index][0], changed[index][1].replace(old,new))
    _, types_a = afidl.compile_module_set(inputs)
    _, types_b = afidl.compile_module_set(changed)
    a = next(t for t in types_a[FAMILY] if t['name']==NAME)
    b = next(t for t in types_b[FAMILY] if t['name']==NAME)
    encoded_a, digest_a = afidl.type_schema_digest(a)
    encoded_b, digest_b = afidl.type_schema_digest(b)
    assert digest_a != digest_b
    # Empty lists exercise no element, so both real schema shapes encode the
    # exact same empty native catalog. This is content, not a schema hash.
    catalog = {'complete_operations':[], 'supported_pairs':[]}
    content = afidl.encode_ace1(catalog).encode('utf8')
    original_inputs = module('independent_raw_binding_inputs', 'tools/oracle/raw_port_capture.py')
    v = next(v for v in original_inputs.vectors() if v['case_id']=='c2r1/declaration_bundle/input-zero')
    owner = v['package']['owners'][0]['anchor']
    pin = {'schema':FAMILY,'revision':REVISION,
           'canonical_content_checksum':{'algorithm':'sha256','hex':hashlib.sha256(content).hexdigest()}}
    return {'diagnostic_id':'c2r3/relation-snapshot-schema-pin','family':FAMILY,'type_name':NAME,
            'source_files':[p for p,_ in inputs],
            'original_relation_source':inputs[index][1], 'changed_relation_source':changed[index][1],
            'construction_schema_a':{'algorithm':'sha256','domain':'type-schema','hex':digest_a},
            'construction_schema_b':{'algorithm':'sha256','domain':'type-schema','hex':digest_b},
            'schema_a_ace1':encoded_a.hex(),'schema_b_ace1':encoded_b.hex(),
            'native_snapshots_equal':True,'native_pin_projection_failure':'UnsupportedModel',
            'native_pin':pin,'canonical_content':list(content),
            'same_current_context_values':True,'current_context_without_relation_schema_admitted':True,
            'raw_verdicts':['violation','violation','violation'],
            'source_input':{'package':v['package'],'candidate':v['candidate'],'owner':owner,'slot':0,'internal_entry':0},
            'full_fact_materialized':False,'positive_root_export':False}


def compare(want, actual):
    def equal(a,b):
        if type(a) is not type(b):return False
        if isinstance(a,dict):return set(a)==set(b) and all(equal(a[k],b[k]) for k in a)
        if isinstance(a,list):return len(a)==len(b) and all(equal(x,y) for x,y in zip(a,b))
        return a==b
    assert equal(want,actual),'binding diagnostic typed mismatch'


def report(actual):
    reference = expected()
    compare(reference,actual)
    mutations=[]
    for field in ['construction_schema_a','construction_schema_b','native_pin','canonical_content',
                  'same_current_context_values','current_context_without_relation_schema_admitted',
                  'raw_verdicts','native_pin_projection_failure','positive_root_export']:
        changed=copy.deepcopy(actual);changed.pop(field)
        try:compare(reference,changed)
        except AssertionError:mutations.append({'diagnostic_mutation':field,'rejected':True})
        else:raise AssertionError('missing required diagnostic field accepted')
    return {'diagnostic_conformance':'PASS','full_fact_conformance':False,
            'finding_id':'R3-CTX-REL-01','classification':'FORMAL_BINDING_GAP',
            'native_schema_field_type':'String schema family',
            'registered_schema_field_type':'af-semantic::TypeSchemaRef',
            'first_unbound_field':'RawValidationSubject.context.contents.relation_snapshots.effect_resource.some.pin.schema',
            'raw_checker_contradiction':False,'false_proven':False,
            'diagnostic_mutations':mutations,'new_full_field_mutations':0,
            'new_complete_capture_cases':0,'strict_total_exit':2}


if __name__ == '__main__':
    if sys.argv[1:]==['--expected']:
        print(json.dumps(expected(),ensure_ascii=True));sys.exit(0)
    output = report(json.load(sys.stdin))
    print(json.dumps(output,indent=2))
    # Only a named diagnostic mode may succeed. This tool cannot certify Fact
    # closure or replace the strict full-field gate.
    sys.exit(0 if sys.argv[1:]==['--diagnostic'] else 2)
