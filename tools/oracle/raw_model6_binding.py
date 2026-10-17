"""Independent R4-A binding counterexample/coverage audit, NOT a full Fact oracle.

Expected comes from original input vectors, actual fixture input schema sources
and the registered descriptors. Rust output is comparison-only. Strict Root,
Nested, Relation, A/C and Total gates MUST remain nonzero in this Outcome C.
"""
import argparse
import copy
import importlib.util
import json
from pathlib import Path
import sys

R = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(R / 'tools'))
P = 'phase-04c1b4c4r3c2r4-'


def module(name, path):
    s = importlib.util.spec_from_file_location(name, R / path)
    m = importlib.util.module_from_spec(s)
    s.loader.exec_module(m)
    return m


A = module('r4_model_audit', 'tools/audit-raw-model6-closure.py')
B = module('r4_legacy_roster', 'tools/audit-raw-full-field-boundary.py')
C = module('r4_relation_inputs', 'tools/oracle/relation_schema_admission.py')


def input_case():
    inputs, _ = B.load_inputs()
    return next(v for v in inputs if v['case_id'] == 'c2r1/declaration_bundle/input-zero')


def expected():
    # These sixteen exact files are the actual independently declared fixture
    # module inputs; the seventeenth Portable schema is NOT silently inserted.
    sources = [(p.relative_to(R).as_posix(), p.read_text(encoding='utf8'))
               for p in sorted((R / 'spec/idl').rglob('*.afidl'))
               if p.name != 'portable-proof-inputs.afidl']
    assert len(sources) == 16
    _, modules = C.AF.compile_module_set(sources)
    context = dict(compiled_modules=[[k, dict(specification_id=k, types=[C.definition(t) for t in modules[k]])]
                                     for k in sorted(modules)],
                   type_schemas=[], effect_catalogs=[],
                   resource_types=dict(schemas=[], descriptors=[]),
                   flow_catalogs=dict(confidentiality=[], taints=[], sinks=[]), usage_meters=[],
                   recovery_operations=dict(effects=[], reconciliation_operations=[]),
                   execution_universes=dict(capabilities=[], trust_domains=[], regions=[], region_compatibility=[], runtime_profiles=[]),
                   relation_snapshots={s: {'none': None} for s, _, _ in C.SLOTS})
    v = input_case()
    reached, _, _ = B.events(*B.load_inputs())
    event = next(e for e in reached if e['case_id'] == v['case_id'])
    assert event['condition'] == 'zero' and event['predecessor'] is None
    names = ['ContractProofContextContents.compiled_modules',
             'Map<SpecificationId,CompiledModuleContents>',
             'ContractProofContextContents.resource_types']
    return dict(case_id='c2r4/model6-structural-node-binding', source_case_id=v['case_id'], source_input=v,
                context=context, context_roles=[['analysis', 'canonical_validation']],
                native_verdict='Violation', condition='zero', raw_member_value=0, minimum=1,
                occurrence=event['occurrence'],
                native_error_rule='AF-CONTRACT-0004', native_error_kind='invalid_or_duplicate_port',
                context_metadata_shape_admitted=True, relation_partition_complete=True,
                probes=[dict(candidate=dict(owner=C.MODEL, name=n), actual_failure='UnsupportedModel') for n in names],
                invented_map_body_failure='UnsupportedModel', existing_sequence_body_shape_admitted=True,
                complete_typed_dag_admitted=False, complete_raw_subject_materialized=False,
                complete_fact_materialized=False, normalizer_implemented=False, positive_root_export=False)


def typed_equal(a, b):
    if type(a) is not type(b):
        return False
    if isinstance(a, dict):
        return set(a) == set(b) and all(typed_equal(a[k], b[k]) for k in a)
    if isinstance(a, list):
        return len(a) == len(b) and all(typed_equal(x, y) for x, y in zip(a, b))
    return a == b


def compare(reference, want, got):
    # Immutable independent reference rejects simultaneous Expected/Actual
    # deletion at ANY depth. This comparison concerns this boundary probe,
    # never credits complete Raw Fact field coverage.
    assert typed_equal(reference, want), 'independent Expected corrupted'
    assert typed_equal(reference, got), 'binding probe differs from independent inputs'


def mutations(ref):
    results = []
    paths = [(k,) for k in ref]
    paths += [('context', k) for k in ref['context']]
    paths += [('context', 'resource_types', k) for k in ('schemas', 'descriptors')]
    paths += [('context', 'relation_snapshots', s) for s, _, _ in C.SLOTS]
    paths += [('context', 'compiled_modules', 0, 1, 'types')]
    for path in paths:
        for both in (False, True):
            want, got = copy.deepcopy(ref), copy.deepcopy(ref)
            for value in ([want, got] if both else [got]):
                at = value
                for p in path[:-1]:
                    at = at[p]
                at.pop(path[-1])
            try:
                compare(ref, want, got)
            except AssertionError:
                results.append(dict(case_id='c2r4/binding-comparator/' + '/'.join(map(str, path)) + ('/both' if both else '/actual'),
                                    rejected=True, both_sides=both))
            else:
                raise AssertionError('accepted deletion ' + str(path))
    return results


def event_roster_and_actual():
    inputs, failures = B.load_inputs()
    main, _, _ = B.events(inputs, failures)
    local = json.loads((R / 'target/phase-04c1b4c4r3c2r1-raw-actual.json').read_bytes())
    actual = []
    for row in local:
        for e in row.get('events', []):
            actual.append(dict(case_id=row['case_id'], occurrence=e['occurrence'],
                               condition=e['condition'], predecessor=e['predecessor']))
    compare(main, copy.deepcopy(main), actual)
    S = module('r4_sensitivity_inputs', 'tools/oracle/raw_root_subset.py')
    sensitivity = json.loads((R / 'target/phase-04c1b4c4r3c2r2-input-sensitivity-actual.json').read_bytes())
    sensitivity_report = S.sensitivity_report(sensitivity)
    # The existing sensitivity oracle independently verifies all48 local events.
    sensitivity_events = [dict(case_id=row['case_id'], **e) for row in S.sensitivity_expected() for e in row.get('events', [])]
    assert len(sensitivity_events) == 48
    binding = json.loads((R / 'target/phase-04c1b4c4r3c2r3-binding-actual.json').read_bytes())
    assert binding['raw_verdicts'] == ['violation'] * 3
    old_binding = [dict(case_id='c2r3/relation-snapshot-schema-pin/' + x,
                        occurrence=copy.deepcopy(expected()['occurrence']), condition='zero', predecessor=None)
                   for x in ('left-A', 'right-A', 'missing')]
    probe = json.loads((R / 'target' / (P + 'binding-actual.json')).read_bytes())
    compare(expected(), expected(), probe)
    fresh = dict(case_id=probe['case_id'], occurrence=probe['occurrence'], condition=probe['condition'], predecessor=None)
    port_roster = main + sensitivity_events + old_binding + [fresh]
    assert len(port_roster) == 177
    # Under model@6, the original missing-current-schema relation Context
    # requires an explicit failed C admission and MUST NOT issue a full Fact.
    # Its already executed zero event remains required event evidence. It is
    # NOT fourteen missing positive Fact fields. The other two contexts use A.
    applicable = main + sensitivity_events + old_binding[:2] + [fresh]
    assert len(applicable) == 176
    non_fact = [dict(event=old_binding[2], expected_full_fact='FORBIDDEN',
                    reason='Some relation snapshot with exact current module missing',
                    required_output='A source + C Missing + fail-closed Subject admission; preserve zero Violation',
                    actual_complete_c_admission='NOT MATERIALIZED',
                    rule='AF-SEMANTIC-0295')]
    return applicable, dict(main=125, main_case_ids=59, sensitivity=48, legacy_binding=3,
                        new_binding=1, actual_events=177, expected_events=177, missing=0, unexpected=0,
                        otherwise_applicable_full_fact_events=176,
                        forbidden_full_fact_events=non_fact,
                        complete_event_coordinate_comparisons=174,
                        legacy_binding_events_with_verdict_only=3,
                        legacy_binding_coordinates_not_inferred_from_expected=True,
                        aggregate_strict_exit=2, original_main_event_gate_exit=0,
                        legacy_sensitivity_receipt=dict(new_input_cases=sensitivity_report['new_input_cases'],
                                                        sensitivity_comparisons=sensitivity_report['sensitivity_comparisons'],
                                                        matched=all(r['matched'] for r in sensitivity_report['comparisons'])))


def report():
    want = expected()
    got = json.loads((R / 'target' / (P + 'binding-actual.json')).read_bytes())
    compare(want, copy.deepcopy(want), got)
    audit = A.report()
    gap = audit['representation_gap']
    assert not gap['registered_exact_child_names']
    assert 'map' not in gap['node_body_cases']
    roster, events = event_roster_and_actual()
    root_names = audit['fact_root_fields']
    required = [dict(event=[e['case_id'], e['occurrence']['direction'], e['occurrence']['original_ordinal']],
                     owner=C.MODEL, type='Fact', field=f, status='NOT MATERIALIZED / NOT COMPARED')
                for e in roster for f in root_names]
    assert len(required) == 2464
    return dict(outcome='C — FORMAL BINDING / STAGE GAP', checkpoint='R4-A', finding=gap,
                descriptor_diff=audit, events=events,
                root_coverage=dict(required_instances=len(required), compared_full_field_instances=0,
                                   missing_or_partial_instances=len(required), full_fact_comparisons=0,
                                   main_required=1750, additional_required=714,
                                   legacy_model5_subset_compared=1000, legacy_model5_incomplete=750,
                                   legacy_credit_is_not_model6_full_field_reacceptance=True),
                required_root_roster=required,
                nested_coverage=dict(required_instances=None, compared_full_field_instances=0,
                                     missing_instances=None, complete=False, unknown_is_not_zero=True),
                relation_coverage=dict(required_instances=None, compared_full_field_instances=0,
                                       missing_instances=None, complete=False, unknown_is_not_zero=True),
                ac_context_admission=dict(all_none_probe_metadata_partition=True,
                                          complete_registered_raw_subject=False, strict_exit=2,
                                          g2_registration_not_reopened=True),
                independent_boundary_comparison='PASS',
                boundary_comparator_mutations=mutations(want), full_field_comparator_mutations=0,
                independent_full_field_oracle='NOT IMPLEMENTED — STOPPED AT REGISTERED NODE BINDING GAP',
                gate_exits=dict(event=events['aggregate_strict_exit'], root=2, nested=2, relation=2, ac=2, total=2),
                raw_producer_verified=False, positive_root_export='NotAuthorized',
                total_t3='INCOMPLETE', t4='NOT ELIGIBLE')


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--gate', choices=['event', 'root', 'nested', 'relation', 'ac', 'total'])
    parser.add_argument('--materialize', action='store_true')
    parser.add_argument('--expected', action='store_true')
    args = parser.parse_args()
    if args.expected:
        print(json.dumps(expected(), indent=2))
        sys.exit(0)
    result = report()
    if args.materialize:
        out = R / 'docs/implementation'
        (out / (P + 'coverage-receipt.json')).write_text(json.dumps(result, indent=2) + '\n', encoding='utf8')
        (out / (P + 'binding-expected.json')).write_text(json.dumps(expected(), indent=2) + '\n', encoding='utf8')
    summary = {k: v for k, v in result.items() if k not in ('descriptor_diff', 'required_root_roster', 'boundary_comparator_mutations')}
    summary['boundary_mutations_rejected'] = len(result['boundary_comparator_mutations'])
    print(json.dumps(summary, indent=2))
    # Diagnostic materialization is not strict success. The original main
    # event gate passes independently; the expanded aggregate also retains
    # three historical binding-event coordinate gaps rather than inventing them.
    sys.exit(result['gate_exits'].get(args.gate, 2))
