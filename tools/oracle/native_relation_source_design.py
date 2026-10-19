"""R5G1 DESIGN ONLY. No registration, production graph or capability issuer.

The census follows every frozen descriptor/site/reference policy. Future recipes
are symbolic amendments, NOT accepted model@7 graphs or assigned production ranks.
Required fields come from frozen source even when both compared views omit them.
"""
import argparse
import copy
import hashlib
import json
import sys
from pathlib import Path

R = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(R / 'tools'))
from oracle.structural_binding_registration import (  # noqa: E402
    audit_registration, catalogue, children, descendants, nominal,
)
from oracle import raw_model7_full_field as B  # noqa: E402

P = 'phase-04c1b4c4r3c2r5g1-'
OLD = 'phase-04c1b4c4r3c2r5-'
SLOTS = ['effect_resource', 'recovery_usage', 'flow_execution']
SOURCE = 'NativeRelationSnapshotSource'
CONTENT = 'RelationSnapshotContent'
CONTEXT = 'ContractProofContextContents'
SUBJECT = 'RawValidationSubject'
C = 'CurrentContextRelationSchemaValidation'
DRAFT = 'NB-DRAFT (UNREGISTERED; edition unallocated)'


def save(name, value):
    (R / 'docs/implementation' / (P + name + '.json')).write_bytes(
        (json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode('utf8'))


def refs(descriptor, path=()):
    if 'ref' in descriptor:
        yield dict(path=[list(step) for step in path], target=descriptor['ref'], kind='named_body_ref')
    elif descriptor.get('kind') == 'dag_ref':
        yield dict(path=[list(step) for step in path], target=descriptor['target']['ref'], kind='dag_ref')
    else:
        for part, child in children(descriptor):
            yield from refs(child, path + tuple(tuple(p) for p in part))


def graph_type_closure(m, proposed=False):
    """Enumerate node-producing paths, NOT descriptor alias transitive closure.

    A ref alias delegates body/sites but never emits a second nominal value node.
    The proposed switch changes only three *declared* Some targets in a copy.
    """
    types = m['types']
    reg = m['structural_binding_registration']
    bindings = {b['catalogue_key']: b for b in reg['value_bindings']}
    sites = {(s['source_catalogue_key'], json.dumps(s['source_path'])): s
             for s in reg['sites']}
    reached, edges, pending = set(), [], [SUBJECT]
    while pending:
        key = pending.pop()
        if key in reached:
            continue
        reached.add(key)
        binding = bindings[key]
        assert 'g_input' in binding['eligibility'], key
        parent, path, d = binding['declaration'], binding['path'], types[key]
        while 'ref' in d:
            parent, path = d['ref'], []
            d = types[parent]
        for part, child in children(d):
            full_path = path + part
            if 'ref' in child:
                target = child['ref']
            elif child.get('kind') == 'dag_ref':
                policies = [p for p in reg['reference_policies']
                            if p['key'] == dict(parent=nominal(parent),
                                               path=[{k: v} for k, v in full_path])
                            and p['environment'] == 'g_value']
                assert len(policies) == 1 and policies[0]['domain'] == 'g_local'
                target = child['target']['ref']
            else:
                target = sites[parent, json.dumps(full_path)]['alias']
            switch = (parent == CONTEXT and len(full_path) == 3
                      and full_path[0] == ['field', 'relation_snapshots']
                      and full_path[1][0] == 'field' and full_path[1][1] in SLOTS
                      and full_path[2] == ['some', None])
            if proposed and switch:
                assert target == CONTENT
                target = SOURCE
            edges.append(dict(parent_node=key, declaration=parent, path=full_path,
                              target=target, draft_changed=proposed and switch))
            pending.append(target)
    return dict(types=sorted(reached), edges=edges, proposed_only=proposed)


def census():
    m = catalogue()
    t, reg = m['types'], m['structural_binding_registration']
    counts = audit_registration()
    assert (counts['descriptors'], counts['references'], counts['sites'],
            counts['reference_policies'], counts['required_field_variants']) == (1131, 2559, 317, 66, 1297)
    current = graph_type_closure(m)
    future = graph_type_closure(m, True)
    assert SOURCE not in current['types'] and SOURCE in future['types']
    assert sum(e['draft_changed'] for e in future['edges']) == 3
    all_refs = [dict(declaring=n, **r) for n, d in t.items() for r in refs(d)]
    bindings = {b['catalogue_key']: b for b in reg['value_bindings']}
    records = []
    for n, d in t.items():
        b = bindings.get(n)
        stage = ('declared_input' if n in reg['input_types'] else
                 'c_validation' if n in reg['output_types'] else 'outside_raw_input_profile')
        records.append(dict(exact_type_owner=nominal(n)['owner'], registered_type_name=n,
                            registered_source='spec/tables/proof-report-model-v7.md + exact frozen v6 base',
                            descriptor=d, nominal_identity=nominal(n), model_type_rank=m['type_ranks'][n],
                            reference_kinds=list(refs(d)), node_body='delegated' if 'ref' in d else d.get('kind'),
                            environments=b['eligibility'] if b else [], stage=stage,
                            required_input='complete source, never a C result',
                            actual_native_source='actual Context.relations.<slot>' if n in [SOURCE, CONTENT] else None,
                            available_construction_stage='before G' if n in current['types'] or n == SOURCE else stage,
                            applicable_slots=SLOTS if n in [SOURCE, CONTENT, C, CONTEXT] else [],
                            existing_rules=['AF-SEMANTIC-0302', 'AF-SEMANTIC-0303', 'AF-SEMANTIC-0306', 'AF-SEMANTIC-0308'],
                            reachable_as_nominal_g_node=n in current['types'],
                            missing_binding=n == SOURCE,
                            incoming_references=[r for r in all_refs if r['target'] == n]))
    absent = {n: dict(registered=False, actual_meaning=meaning) for n, meaning in {
        'RawValidationSource': 'Source.raw_validation, not a standalone descriptor',
        'DagRank': 'U64 resolving Node<T> in fixed carrier; not a named descriptor',
        'VerifiedInputGraph': 'private Rust issuer, not a registered ModelType',
        'RawSubjectInputNode': 'private Rust carrier/rank wrapper, not a registered ModelType',
    }.items()}
    policies = []
    for p in reg['reference_policies']:
        policies.append(dict(registered=p, review='exact target + fixed carrier; no implicit alias cast',
                             supplies_source_node=False,
                             c_source=p['key'] == dict(parent=nominal(C), path=[{'field': 'source'}])))
    sites = [dict(registered=s, produces_required_nominal=s['definition']['value_type'] == nominal(SOURCE),
                  review='inline value site; alias child target remains exact') for s in reg['sites']]
    assert not any(s['produces_required_nominal'] for s in sites)
    native_incoming = [r for r in all_refs if r['target'] == SOURCE]
    for row in native_incoming:
        row['reachable_node_producer'] = row['declaring'] in current['types'] and row['declaring'] != CONTENT
        row['exclusion'] = ('body delegation retains CONTENT nominal key' if row['declaring'] == CONTENT else
                            'derived C consumer, not G producer' if row['kind'] == 'dag_ref' else
                            'outside unique Raw Subject G root closure')
    return dict(kind='COMPLETE FROZEN CATALOGUE CENSUS; NOT PRODUCTION EXTENSION', counts=counts,
                descriptors=records, spelling_corrections=absent, structural_sites=sites,
                reference_policies=policies, incoming_native_source=native_incoming,
                current_node_production_closure=current, proposed_node_production_closure=future,
                existing_binding_available=False,
                exhaustion='every node reachable from sole Raw Subject root follows enumerated declared child/site target; '
                           'alias descriptor delegation is not a node-producing edge; no reachable edge targets Source')


def source_fields():
    return [n for n, _ in catalogue()['types'][SOURCE]['fields']]


def future_recipe(source, owner, slot, lookup='Valid'):
    """Symbolic full-field design expectation, no edition/rank allocation."""
    assert slot in SLOTS
    assert list(source) or source
    assert set(source) == set(source_fields())
    assert source['kind'] == slot
    return dict(status='FUTURE DESIGN ONLY; NOT REGISTERED / NOT PRODUCTION',
                nominal=dict(owner=DRAFT, name=SOURCE), source=copy.deepcopy(source),
                source_association=dict(owner=copy.deepcopy(owner), slot=slot,
                                        subject_path=['context', 'contents', 'relation_snapshots', slot, 'some']),
                reachability=[SUBJECT, 'RawContextBinding', CONTEXT,
                              'registered relation_snapshots site', 'registered slot Option site', SOURCE],
                root_types=[SUBJECT], child_target=SOURCE, source_stage='before_G',
                dag_rank='canonical full graph AF-SEMANTIC-0217; UNALLOCATED',
                rank_dependencies='all six exact typed child subtrees < Source < Option < parents <= sole root',
                c_source_domain='g_external', c_source_target=SOURCE, c_observation=lookup,
                complete_raw_fact=False, root_export='NotAuthorized')


def design_compare(required_source, required_owner, required_slot, expected, actual, lookup='Valid'):
    """Both views must independently contain the complete source-derived recipe."""
    ref = future_recipe(required_source, required_owner, required_slot, lookup)
    B.exact(ref, expected)
    B.exact(ref, actual)


GROUPS = [
    ['same-body-different-nominal', 'repeated-same-source', 'different-owner', 'different-slot',
     'missing-source', 'wrong-association', 'content-as-source', 'same-shape-different-type-rank',
     'incomplete-source', 'illegal-alias', 'cross-edition-source', 'complete-native-source'],
    ['reachable-source', 'unreachable-source', 'wrong-child-edge', 'wrong-parent', 'arbitrary-rank',
     'duplicate-equal-node', 'incorrect-interning', 'wrong-root', 'extra-root',
     'source-produced-after-G', 'G-C-G-backedge', 'acyclic-canonical-input'],
    ['effect_resource-valid', 'effect_resource-missing', 'recovery_usage-valid', 'recovery_usage-missing',
     'flow_execution-valid', 'flow_execution-missing', 'all-none', 'same-A-different-C',
     'A-fields-unchanged', 'C-cannot-fill-history', 'wrong-cross-slot', 'existing-G-source-reference',
     'C-missing-no-full-fact'],
    ['old-graph-preserved', 'new-owner-identity-required', 'old-data-no-source', 'unregistered-alias',
     'cross-edition-reference', 'W-local-substitute', 'both-omit-source', 'forged-verified-graph',
     'forged-raw-fact', 'forged-root-export', 'resource-bounds', 'lawful-future-design'],
]


def native_source(slot):
    """A projection from declared native input ONLY; never calls C/expected()."""
    value = B.G.REL.catalog(B.G.REL.case_base(slot))
    B.G.REL.validate_native_catalog(slot, value)
    family = B.G.REL.info(slot)[1]
    data = B.G.REL.CAT.ace1(value).encode('utf8')
    pin = dict(family=family, revision='g2-fixture',
               canonical_content_checksum=dict(algorithm='sha256', hex=hashlib.sha256(data).hexdigest()))
    return dict(kind=slot, schema_family=family, revision='g2-fixture', pin=pin,
                canonical_content=list(data), catalog={slot: value})


def declared_context(slot, mode, present=True):
    """Full actual input definitions, not the result of validating them."""
    modules, _ = B.G.REL.compiled(slot, 'missing_module' if mode == 'Missing' else 'A')
    result = copy.deepcopy(B.G.empty_context_with_actual_modules())
    result['contents']['compiled_modules'] = [
        [name, dict(specification_id=name, types=[B.G.REL.definition(t) for t in modules[name]])]
        for name in sorted(modules)]
    result['contents']['relation_snapshots'] = {s: {'none': None} for s in SLOTS}
    if present:
        result['contents']['relation_snapshots'][slot] = {'some': native_source(slot)}
    return result


def vectors():
    # Original declaration and independent native/AFIDL input, never Rust Actual.
    original = next(v for v in B.G.RAW.vectors() if v['case_id'] == 'c2r1/declaration_bundle/input-zero')
    owner = original['package']['owners'][original['owner']]['anchor']
    association = dict(source_package=original['package'], owner=owner, entry_ordinal=original['slot'],
                       member_path=[['entry', original['path_entry']]])
    # Select an explicitly positive source input, not the first different
    # anchor (which can be the intentionally invalid foreign-domain case).
    other = next(r for r in B.G.RAW.vectors()
                 if r['case_id'] == 'c2r1/declared-namespace-is-not-an-authentication-whitelist')
    assert other['operation'] == 'capture'
    assert other['package']['owners'][other['owner']]['anchor']['domain'] == other['package']['domain']
    assert other['package']['owners'][other['owner']]['anchor'] != owner
    result = []
    positives = {'repeated-same-source', 'different-owner', 'different-slot', 'complete-native-source',
                 'reachable-source', 'acyclic-canonical-input', 'all-none', 'same-A-different-C',
                 'A-fields-unchanged', 'existing-G-source-reference', 'C-missing-no-full-fact',
                 'old-graph-preserved', 'new-owner-identity-required', 'lawful-future-design'}
    for group, names in enumerate(GROUPS, 1):
        for name in names:
            slot = next((s for s in SLOTS if name.startswith(s)), SLOTS[0])
            mode = 'Missing' if name.endswith('-missing') or name == 'C-missing-no-full-fact' else 'Valid'
            native = native_source(slot)
            recipe = future_recipe(native, association, slot, mode)
            admit = name in positives or name.endswith('-valid') or name.endswith('-missing')
            if name == 'all-none':
                recipe = dict(source_nodes=[], observations=[], complete_raw_fact=False,
                              root_export='NotAuthorized', reason='absence is not positive evidence')
            candidate = copy.deepcopy(recipe)
            changes = {
                'same-body-different-nominal': ('nominal', dict(owner=DRAFT, name=CONTENT)),
                'missing-source': ('source', None),
                'wrong-association': ('source_association', dict(owner=None, slot=slot)),
                'content-as-source': ('child_target', CONTENT),
                'same-shape-different-type-rank': ('dag_rank', 'reuse Content registration rank 107 as DagRank'),
                'incomplete-source': ('source', {k: v for k, v in native.items() if k != 'pin'}),
                'illegal-alias': ('nominal', dict(owner=DRAFT, name='UnregisteredSourceAlias')),
                'cross-edition-source': ('nominal', nominal(SOURCE)),
                'unreachable-source': ('reachability', []),
                'wrong-child-edge': ('child_target', CONTENT),
                'wrong-parent': ('reachability', ['CurrentContextRelationSchemaValidation', SOURCE]),
                'arbitrary-rank': ('dag_rank', 42),
                'duplicate-equal-node': ('root_types', [SUBJECT, SOURCE]),
                'incorrect-interning': ('source_association', dict(owner='content-only-merged', slot=slot)),
                'wrong-root': ('root_types', [SOURCE]),
                'extra-root': ('root_types', [SUBJECT, SOURCE]),
                'source-produced-after-G': ('source_stage', 'after_C'),
                'G-C-G-backedge': ('reachability', [SUBJECT, C, SOURCE, SUBJECT]),
                'C-cannot-fill-history': ('source', dict(native, construction_schema='invented-current-schema')),
                'wrong-cross-slot': ('source_association', dict(owner=association, slot='flow_execution')),
                'old-data-no-source': ('source', {'digest_only': native['pin']}),
                'unregistered-alias': ('child_target', 'UnregisteredSourceAlias'),
                'cross-edition-reference': ('nominal', nominal(SOURCE)),
                'W-local-substitute': ('c_source_domain', 'w_local'),
                'forged-verified-graph': ('status', 'VERIFIED BY CALLER BOOLEAN'),
                'forged-raw-fact': ('complete_raw_fact', True),
                'forged-root-export': ('root_export', 'Authorized'),
                'resource-bounds': ('rank_dependencies', 'unbounded/truncated successful prefix'),
            }
            if name in changes:
                field, value = changes[name]
                candidate[field] = value
            elif name == 'both-omit-source':
                del candidate['source']
            comparison_owner = association
            pair = None
            if name == 'different-owner':
                comparison_owner = dict(source_package=other['package'],
                                        owner=other['package']['owners'][other['owner']]['anchor'],
                                        entry_ordinal=other['slot'], member_path=[['entry', other['path_entry']]])
                pair = dict(first=future_recipe(native, association, slot),
                            second=future_recipe(native, comparison_owner, slot))
                recipe = pair['second']
                candidate = copy.deepcopy(recipe)
            elif name == 'different-slot':
                pair = {s: future_recipe(native_source(s),
                                        association, s) for s in SLOTS}
            elif name in ['same-A-different-C', 'A-fields-unchanged']:
                pair = dict(valid=future_recipe(native, association, slot, 'Valid'),
                            missing=future_recipe(native, association, slot, 'Missing'))
                assert pair['valid']['source'] == pair['missing']['source']
            elif name == 'repeated-same-source':
                pair = dict(first=recipe, second=copy.deepcopy(recipe))
            result.append(dict(case_id=f'r5g1/design/{group}/{name}', group=group, scenario=name,
                               status='Not Executed',
                               actual_existing_input=dict(original_declaration=other if name == 'different-owner' else original,
                                                          paired_original_declaration=original if name == 'different-owner' else None,
                                                          native_source=None if name == 'all-none' else native,
                                                          actual_source_association=comparison_owner, slot=slot,
                                                          actual_current_context=declared_context(slot, mode, name != 'all-none'),
                                                          paired_current_contexts={m: declared_context(slot, m) for m in ['Valid', 'Missing']}
                                                          if name in ['same-A-different-C', 'A-fields-unchanged'] else None,
                                                          current_context_mode='absent' if name == 'all-none' else mode),
                               registered_expected=dict(source_node_count=0, some_nominal=CONTENT,
                                                        c_source_binding='UNAVAILABLE' if name != 'all-none' else 'not_applicable',
                                                        complete_raw_fact=False, positive_root_export=False),
                               proposed_future_expected=dict(admissible_as_design=admit, recipe=recipe,
                                                             expected_failure=None if admit else name),
                               proposed_candidate=candidate, paired_design_expected=pair,
                               executed_scope=[],
                               unexecuted_scope=['formal successor registration', 'production successor G',
                                                 'production Source rank', 'complete Raw Fact capture'],
                               applicable_rules=['AF-SEMANTIC-0217', 'AF-SEMANTIC-0286', 'AF-SEMANTIC-0296',
                                                 'AF-SEMANTIC-0303', 'AF-SEMANTIC-0306', 'NB-DRAFT-01..08'],
                               adjudication_conclusion='DESIGN ONLY: ' + ('allow stated symbolic target; no capability' if admit else 'reject')))
    assert len(result) == 49
    return result


def vector_checks(v):
    rows = []
    for row in v:
        a = row['actual_existing_input']
        want = copy.deepcopy(row['proposed_future_expected']['recipe'])
        got = row['proposed_candidate']
        if row['scenario'] == 'both-omit-source':
            del want['source']
        try:
            if row['scenario'] == 'all-none':
                B.exact(dict(source_nodes=[], observations=[], complete_raw_fact=False,
                             root_export='NotAuthorized', reason='absence is not positive evidence'), got)
            else:
                design_compare(a['native_source'], a['actual_source_association'], a['slot'], want, got,
                               a['current_context_mode'])
            accepted = True
        except AssertionError:
            accepted = False
        assert accepted == row['proposed_future_expected']['admissible_as_design'], row['case_id']
        rows.append(dict(case_id=row['case_id'], status='Not Executed',
                         design_recipe_comparison='ACCEPT' if accepted else 'REJECT',
                         executed_scope='symbolic recipe comparison ONLY; no graph construction or native execution',
                         production_scope='Not Executed', future_registration_scope='Not Executed'))
    return rows


def comparator_checks(v):
    a = v[0]['actual_existing_input']
    source, owner, slot = a['native_source'], a['actual_source_association'], a['slot']
    required = future_recipe(source, owner, slot)
    design_compare(source, owner, slot, required, required)
    checks = []
    paths = [('source',), ('nominal',), ('source_association',), ('reachability',),
             ('root_types',), ('child_target',), ('source_stage',), ('dag_rank',),
             ('rank_dependencies',), ('c_source_domain',), ('c_source_target',)]
    paths += [('source', name) for name in source_fields()]
    paths += [('source_association', 'owner'), ('source_association', 'slot')]
    for path in paths:
        for both in [False, True]:
            want, got = copy.deepcopy(required), copy.deepcopy(required)
            for view in ([want, got] if both else [got]):
                obj = view
                for part in path[:-1]:
                    obj = obj[part]
                del obj[path[-1]]
            try:
                design_compare(source, owner, slot, want, got)
            except AssertionError:
                checks.append(dict(path=list(path), both_sides=both, rejected=True,
                                   credit='DESIGN COMPARATOR ONLY'))
            else:
                raise AssertionError(('both-sided omission accepted', path))
    assert len(checks) == 38
    return checks


def diagnostics():
    reference, coverage = B.boundary_expected()
    actual = B.read('target/' + OLD + 'boundary-actual.json')
    B.compare(reference, reference, actual)
    frozen = B.read('docs/implementation/' + OLD + 'raw-model7-full-actual.json')['boundary']
    B.exact(frozen, actual)
    rows = []
    for row, graph_check in zip(reference, coverage):
        graph = B.read(row['graph'])
        selected = row['selected']
        c = row['c_metadata'][selected] if selected != 'none' else None
        rows.append(dict(case_id=row['case_id'], actual_native_source=c['source'] if c else None,
                         actual_current_context=row['c_metadata'][SLOTS[0]]['current_modules'],
                         actual_g_root=graph['roots'], actual_g_nominal_types=sorted({
                             n['type']['owner'] + '/' + n['type']['name'] for n in graph['nodes']}),
                         available_content_node=row['actual_source'], required_source_type=nominal(SOURCE),
                         matching_native_source_nodes=row['matching_target_ranks'],
                         first_missing_g_reference='StructuralSubjectAdmission.observations[].source' if c else None,
                         c_result=c, port_verdict=row['port_event'],
                         raw_fact_admission_result='not issued; C failure and/or nominal boundary; complete producer unavailable',
                         actual_source_context_carrier_borrows=True, g_unchanged_after_C=True,
                         graph_comparison=graph_check))
    assert len(rows) == 7
    return dict(boundary_cases=rows, mutations=B.mutations(reference), complete_raw_field_credit=0,
                c_metadata_identity='frozen relation-only Analysis@7 / Model@6, not current Subject capability')


def materialize(diagnostic=False):
    c = census()
    v = vectors()
    checks = comparator_checks(v)
    results = vector_checks(v)
    save('existing-binding-census', c)
    save('r5g1-native-relation-source-binding-census', c)
    save('design-vectors', v)
    save('design-vector-results', dict(initial_status='Not Executed',
         results=results,
         comparator_checks=checks, production_design_case_ids_executed=0,
         note='Symbolic recipe evaluation is not production/registration execution.'))
    if diagnostic:
        save('boundary-diagnostics', diagnostics())
    print(json.dumps(dict(descriptors=1131, sites=317, policies=66,
                          existing_binding=False, proposed_changed_source_edges=3,
                          design_vectors=len(v), design_omission_checks=len(checks),
                          production_successor_executed=0, root_export='NotAuthorized')))


if __name__ == '__main__':
    cli = argparse.ArgumentParser()
    cli.add_argument('--materialize', action='store_true')
    cli.add_argument('--diagnostics', action='store_true')
    args = cli.parse_args()
    if args.materialize:
        materialize(args.diagnostics)
    else:
        c = census()
        v = vectors()
        comparator_checks(v)
        vector_checks(v)
        B.exact(c, B.read('docs/implementation/' + P + 'existing-binding-census.json'))
        B.exact(v, B.read('docs/implementation/' + P + 'design-vectors.json'))
        if args.diagnostics:
            B.exact(diagnostics(), B.read('docs/implementation/' + P + 'boundary-diagnostics.json'))
        print(json.dumps(dict(design_vectors=len(v), frozen_census='MATCH',
                              production_successor='NOT EXECUTED', positive_root_export=False)))
