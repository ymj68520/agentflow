"""R4-A descriptor authority audit; descriptor counts are NOT runtime coverage."""
import argparse
import hashlib
import json
import re
from pathlib import Path

R = Path(__file__).resolve().parents[1]
P = 'phase-04c1b4c4r3c2r4-'


def model(edition):
    p = R / f'spec/tables/proof-report-model-v{edition}.md'
    return json.loads(re.search(r'```json\s*(.*?)\s*```', p.read_text(encoding='utf8'), re.S)[1])


def refs(v):
    if isinstance(v, dict):
        if 'ref' in v:
            yield v['ref']
        for x in v.values():
            yield from refs(x)
    elif isinstance(v, list):
        for x in v:
            yield from refs(x)


def closure(types, root):
    todo = [root]
    seen = set()
    while todo:
        n = todo.pop()
        if n in seen:
            continue
        seen.add(n)
        assert n in types, n
        todo.extend(refs(types[n]))
    return sorted(seen)


def report():
    old, new = model(5), model(6)
    assert new['inherited_registered_model'] == old
    a, b = old['types'], new['types']
    assert len(b) == 774
    assert sum(len(list(refs(v))) for v in b.values()) == 2204
    roots = [n for n, _ in b['Fact']['fields']]
    assert roots == [n for n, _ in a['Fact']['fields']]
    direct = [n for n in sorted(a) if a[n] != b[n]]
    rows = []
    for n, d in b['Fact']['fields']:
        old_nested = sorted(set(refs(dict(a['Fact']['fields'])[n])))
        new_nested = sorted(set(refs(d)))
        reachable = sorted(set(x for child in new_nested for x in closure(b, child)))
        rows.append(dict(field=n, old_owner=old['model_id'], new_owner=new['model_id'],
                         old_type=dict(a['Fact']['fields'])[n], new_type=d,
                         old_dependencies=old_nested, new_dependencies=new_nested,
                         reachable_descriptors=reachable,
                         changed_reachable=[x for x in reachable if x in direct],
                         changed_applicability='Some relation inputs require independent C Valid; None executes no C event',
                         changed_source_rules=['AF-SEMANTIC-0286', 'AF-SEMANTIC-0295', 'AF-SEMANTIC-0296', 'AF-SEMANTIC-0297'],
                         expected_method='Original declarative/compiler/native Context inputs plus registered rules, never Rust Actual'))
    structural = dict(b['ContractProofContextContents']['fields'])['compiled_modules']
    # Resolve only registered aliases. No synthetic field-path type names,
    # structural ModelType cases, or map-to-sequence rules are invented here.
    def resolve(d):
        seen = set()
        while 'ref' in d:
            assert d['ref'] not in seen
            seen.add(d['ref'])
            d = b[d['ref']]
        return d
    structural_names = [n for n in b if resolve({'ref': n}) == structural]
    body_cases = [n for n, _ in b['NodeBody']['variants']]
    assert not structural_names and 'map' not in body_cases
    gap = dict(label='R4-DAG-STRUCT-01', classification='FORMAL BINDING / STAGE GAP',
               checkpoint='R4-A', status='REPRODUCIBLE; NOT A REGISTERED RULE OR TYPE',
               path=['RawValidationSubject', 'context', 'contents', 'compiled_modules'],
               field_descriptor=structural, registered_exact_child_names=structural_names,
               node_body_cases=body_cases,
               missing_binding=['Closed ModelType identity for anonymous structural child descriptors',
                                'Registered typed NodeBody representation for map key/value entries'],
               inherited_from_model5=True, introduced_by_g2=False,
               governing_rules=['AF-SEMANTIC-0198','AF-SEMANTIC-0200','AF-SEMANTIC-0214',
                                'AF-SEMANTIC-0217','AF-SEMANTIC-0221','AF-SEMANTIC-0262','AF-SEMANTIC-0269','AF-SEMANTIC-0297'],
               not_claimed=['Not a native port predicate contradiction or false-Proven',
                            'Not a recurrence of native family/construction meaning R3-CTX-REL-01',
                            'Not proof that logical Context metadata cannot be represented',
                            'Not permission to implement G02 or T4',
                            'Not permission to add an implicit general inline Fact companion'],
               stopped='Complete registered Raw Node/Fact materialization; no ad-hoc type/body/rank substitution')
    assert dict(a['ContractProofContextContents']['fields'])['compiled_modules'] == structural
    anonymous = []
    def structural_sites(v, path):
        if isinstance(v, dict):
            if v.get('kind') in ('record', 'enum', 'sequence', 'set', 'map', 'option') and len(path) > 1:
                matches = [n for n in b if resolve({'ref': n}) == v]
                anonymous.append(dict(descriptor_path=path, descriptor=v, matching_registered_names=matches,
                                      note='Static binding site, NOT a runtime field count or an authorization to reuse a structurally equal nominal type'))
            for k, child in v.items():
                structural_sites(child, path + [k])
        elif isinstance(v, list):
            for i, child in enumerate(v):
                structural_sites(child, path + [i])
    for n in closure(b, 'RawValidationSubject'):
        structural_sites(b[n], [n])
    return dict(start_head='3dcad565bb48bcf90a22e09f64ec21123d1e1e3a',
                old_model=old['model_id'], new_model=new['model_id'],
                tuple=dict(analysis=new['analysis_owner'], model=new['model_id'],
                           report=new['model_edition'], obligation=new['obligation_edition']),
                inherited_model_exact=True, descriptors=len(b), references=2204,
                added_descriptors=sorted(set(b)-set(a)), removed_descriptors=sorted(set(a)-set(b)),
                changed_descriptors=[dict(name=n, old=a[n], new=b[n]) for n in direct],
                fact_root_fields=roots, fact_root_count=len(roots), field_diff=rows,
                full_fact_static_roster=[dict(owner=new['model_id'], type=n, descriptor=b[n],
                                             references=list(refs(b[n]))) for n in closure(b, 'Fact')],
                historical_baseline=dict(main_cases=59, reached_events=125, root_instances=1750,
                                         legacy_model5_compared=1000, legacy_model5_incomplete=750,
                                         model6_reaccepted_roots=0, model6_missing_roots=1750),
                runtime_nested_instances=None, runtime_relation_instances=None,
                static_descriptor_count_is_not_runtime_coverage=True,
                representation_gap=gap,
                anonymous_structural_descriptor_sites=anonymous,
                formal_source_sha256={f'model{ed}': hashlib.sha256((R / f'spec/tables/proof-report-model-v{ed}.md').read_bytes()).hexdigest() for ed in (5, 6)})


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--materialize', action='store_true')
    args = parser.parse_args()
    value = report()
    if args.materialize:
        (R / 'docs/implementation' / (P + 'model-diff.json')).write_text(json.dumps(value, indent=2) + '\n', encoding='utf8')
    print(json.dumps({k: v for k, v in value.items() if k not in ('field_diff', 'full_fact_static_roster', 'changed_descriptors')}, indent=2))
