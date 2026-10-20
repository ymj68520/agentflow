"""R1 source/evidence review. Read-only with respect to every pre-R1 file."""
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys

R = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('r1_verification', R / 'tools/verify-r5g2r1.py')
V = importlib.util.module_from_spec(spec)
spec.loader.exec_module(V)
P = V.P
HISTORIES = [
    ('abc91d533b4d62e91226722e1d1fcaa30bd00883', 512),
    ('51239a0045d7a6fb274bb24cdc8130f2346bf085', 51),
    ('4d3bb59031795d31ecfccf29e3d9f2d89a7d9ce8', 173),
    ('875e6e41f16845c78d4f6d31573826c547d48c07', 889),
    ('e623aab705c5f8269807201e112a32e67e6869bf', 262),
    ('2ad4359410c087462f8366f1ddb5c38bebfef681', 254),
    ('8dc4f508c2f97b431053528ab14d2bd9263b3737', 534),
    ('5ef4e56a3ad5ef9cc3fe050dcbe9b98f678476ea', 3662),
]


def configuration():
    file = R / 'target/phase-04c1b4c4r3c2r5-histories.json'
    entries = json.loads(file.read_bytes())
    assert [(r['frozen_head'], r['case_ids']) for r in entries] == HISTORIES
    rows = []
    for index, row in enumerate(entries, 1):
        source = Path(row['source']).resolve()
        argv = row['command']
        build = Path(argv[argv.index('--target-dir') + 1]).resolve()
        assert source.is_relative_to((R / 'target').resolve())
        assert build.is_relative_to((R / 'target').resolve())
        rows.append(dict(group=index, frozen_head=row['frozen_head'], case_ids=row['case_ids'],
                         source=str(source), build=str(build), approved_debug=0 if index == 8 else 2))
    assert len({r['source'] for r in rows}) == len({r['build'] for r in rows}) == 8
    result = dict(status='PASS', scope='Lightweight configuration identity audit; no test credit',
                  configuration_file=str(file), configuration_sha256=V.sha(file), groups=rows)
    V.save('historical-configuration', result)
    print(json.dumps(dict(status='PASS', groups=8, distinct_builds=8, test_credit=0)))


def rawfreeze():
    before = V.read('target/' + P + 'start-raw-map.json')
    assert all(V.record(n) == old for n, old in before.items())
    return dict(status='PASS', protected_pre_R1_paths=len(before),
                changed=0, removed=0, all_raw_bytes_exact=True)


def specification():
    freeze = rawfreeze()
    sys.path.insert(0, str(R / 'tools'))
    from oracle import native_source_registration as O
    model = O.catalogue()
    old = O.old_catalogue()
    delta = json.loads(re.search(r'```json\s*(.*?)\s*```',
        (R / 'spec/tables/proof-report-model-v8.md').read_text(encoding='utf8'), re.S)[1])
    assert delta['edition'] == O.EDITION
    changed = [name for name in model['types'] if model['types'][name] != old['types'][name]]
    assert set(changed) == {'ContractProofContextContents', 'StructuralValue0030',
                           'StructuralValue0031', 'StructuralValue0032', 'StructuralValue0033'}
    contents = dict(model['types']['ContractProofContextContents']['fields'])
    edges = dict(contents['relation_snapshots']['fields'])
    assert set(edges) == set(O.SLOTS)
    assert all(d == {'kind': 'option', 'item': {'ref': 'NativeRelationSnapshotSource'}}
               for d in edges.values())
    assert model['type_ranks'] == old['type_ranks']
    assert model['types']['NativeRelationSnapshotSource'] == old['types']['NativeRelationSnapshotSource']
    assert model['types']['RelationSnapshotContent'] == old['types']['RelationSnapshotContent']
    assert model['types']['NodeBody'] == old['types']['NodeBody']
    reg = model['structural_binding_registration']
    policies = [p for p in reg['reference_policies']
                if p['key'] == dict(parent=dict(owner=O.MODEL, name='CurrentContextRelationSchemaValidation'),
                                    path=[{'field': 'source'}])]
    assert len(policies) == 2
    assert {p['environment'] for p in policies} == {'output_record', 'w_value'}
    assert all(p['domain'] == 'g_external' and p['target'] ==
               dict(owner=O.MODEL, name='NativeRelationSnapshotSource') for p in policies)
    specs = V.read('spec/registry/specifications.json')['specifications']
    rules = V.read('spec/registry/rules.json')['rules']
    old_specs = V.read('spec/conformance/g01-native-source-staging/analysis8-specifications.json')['specifications']
    old_rules = V.read('spec/conformance/g01-native-source-staging/analysis8-rules.json')['rules']
    assert (len(specs), len(rules), len(old_specs), len(old_rules)) == (74, 531, 70, 522)
    sm = {s['specification_id']: s for s in specs}
    rm = {r['rule_id']: r for r in rules}
    assert len(sm) == len(specs) and len(rm) == len(rules)
    assert all(sm[s['specification_id']] == s for s in old_specs)
    assert all(rm[r['rule_id']] == r for r in old_rules)
    for s in specs:
        assert V.sha(R / s['normative_source']['logical_path']) == s['normative_source']['content_digest']['hex']
    formal = [rm[f'AF-SEMANTIC-{n:04}'] for n in range(315, 324)]
    for row in formal:
        source = R / sm[row['normative_source']]['normative_source']['logical_path'] if row['normative_source'] in sm else R / row['normative_source']
        assert ('id="' + row['source_anchor'] + '"') in source.read_text(encoding='utf8')
    result = dict(status='PASS', scope='Specification review only; no historical/runtime execution credit',
                  freeze=freeze, tuple=O.EDITION, changed_declarations=changed,
                  exact_three_some_edges=edges, c_source_policies=policies,
                  native_source_fields=[n for n, _ in model['types']['NativeRelationSnapshotSource']['fields']],
                  descriptors=len(model['types']), structural_sites=len(reg['sites']),
                  reference_policies=len(reg['reference_policies']),
                  old_spec_objects_exact=70, old_rule_objects_exact=522,
                  new_formal_rules=formal, native_snapshot_and_port_sources_unchanged=True,
                  no_new_type_ranks=True, no_nominal_alias_cast=True,
                  no_historical_reconstruction=True, production_credit=0)
    V.save('specification-review', result)
    print(json.dumps({k: result[k] for k in ['status', 'scope', 'descriptors', 'structural_sites', 'reference_policies']}))


def eol():
    prior = V.read('target/' + V.OLD + 'final-runtime-results.json')
    rows = []
    for row in prior['history_audit_input_provenance']:
        assert V.sha(R / row['original']) == row['original_sha256']
        assert V.sha(R / row['destination']) == row['counterpart_sha256']
        original = V.read(row['original'])
        counterpart = V.read(row['destination'])
        assert set(original) == set(counterpart)
        mapped = {r['path']: r for r in row['eol_only']}
        for name, digest in original.items():
            if digest == counterpart[name]:
                assert name not in mapped
            else:
                assert mapped[name]['original_sha256'] == digest
                assert mapped[name]['git_base_sha256'] == counterpart[name]
        rows.append(dict(group=row['group'], status='PASS', original=row['original'],
                         counterpart=row['destination'], original_sha256=row['original_sha256'],
                         counterpart_sha256=row['counterpart_sha256'], paths=len(original),
                         approved_eol_only_paths=len(mapped), new_correspondence_rules=0,
                         old_files_modified=False))
    V.save('historical-eol-review', dict(status='PASS', groups=rows,
           scope='Previously approved per-path correspondence rechecked; not test execution credit'))
    print(json.dumps(dict(status='PASS', groups=len(rows), new_rules=0)))


def quality(final=False):
    inheritance = V.read('target/' + P + 'inheritance.json')
    assert inheritance['source_inputs_exact']
    for item in inheritance['inherited']:
        row = item['receipt']
        assert V.sha(Path(row['log'])) == row['log_sha256']
        assert row['exit'] == row['expected_exit'] and row['safety']['cancelled'] is None
    runs = []
    for file in sorted((R / 'target').glob(P + 'run-*.json')):
        row = json.loads(file.read_bytes())
        assert V.sha(Path(row['log'])) == row['log_sha256']
        safety = row['safety']
        assert (safety['cpu_hard_cap_percent'], safety['process_memory_mib'], safety['whole_job_memory_mib']) == (5, 768, 1536)
        assert safety['limits_verified'] and safety['test_threads'] == safety['cargo_build_jobs_env'] == safety['active_top_level_jobs'] == 1
        runs.append(dict(receipt=file.name, status=row['status'], exit=row['exit'],
                         log_sha256=row['log_sha256'], cancellation=safety['cancelled']))
    history = []
    for i, old in enumerate(V.read('target/phase-04c1b4c4r3c2r5-histories.json'), 1):
        receipt = R / 'target' / (P + f'history-{i}-result.json')
        row = json.loads(receipt.read_bytes()) if receipt.exists() else None
        history.append(dict(group=i, frozen_head=old['frozen_head'], required_case_ids=old['case_ids'],
                            status=row['status'] if row else 'NOT EXECUTED',
                            exit=row['exit'] if row else None,
                            completed_case_ids=old['case_ids'] if row and row['status'] == 'PASS' else 0))
    gates_closed = (all(h['status'] == 'PASS' for h in history) and
                    (R / 'target' / (P + 'g3-complete.json')).exists() and
                    (R / 'target' / (P + 'generators-complete.json')).exists())
    result = dict(status='PASS' if gates_closed else 'FAILED', review_completed=True,
                  reason=None if gates_closed else 'Mandatory gates incomplete; not a semantic test assertion failure',
                  scope='Quality/evidence review; inherited execution distinguished from R1 execution',
                  source_preservation=rawfreeze(), inherited_receipts=len(inheritance['inherited']),
                  current_runs=runs, histories=history, all_mandatory_gates_closed=gates_closed,
                  outcome='A' if gates_closed else 'D',
                  frozen_g3='PASS' if (R / 'target' / (P + 'g3-complete.json')).exists() else 'NOT EXECUTED',
                  generator_rounds='PASS' if (R / 'target' / (P + 'generators-complete.json')).exists() else 'NOT EXECUTED',
                  capability='NotAuthorized', production_credit=0)
    result['reviewer_source_sha256'] = V.sha(Path(__file__))
    V.save('quality-evidence-review-final' if final else 'quality-evidence-review', result)
    print(json.dumps(dict(status=result['status'], outcome=result['outcome'], completed_histories=sum(h['status'] == 'PASS' for h in history))))


def historical_identities():
    rows = []
    configuration = V.read('target/' + P + 'historical-configuration.json')
    assert V.sha(Path(configuration['configuration_file'])) == configuration['configuration_sha256']
    entries = V.read('target/phase-04c1b4c4r3c2r5-histories.json')
    assert [(r['frozen_head'], r['case_ids']) for r in entries] == HISTORIES
    for index, old in enumerate(entries, 1):
        folder = Path(old['source'])
        snapshot = V.blobmap(old['frozen_head'], folder)
        def identity(values):
            return hashlib.sha256(json.dumps(values, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
        compiler = {n: r for n, r in snapshot.items() if n.startswith('crates/') or n in ['Cargo.toml', 'Cargo.lock']}
        expected = {n: r for n, r in snapshot.items() if n.startswith(('spec/conformance/', 'tools/oracle/')) or 'expected' in n.lower()}
        registry = {n: r for n, r in snapshot.items() if n.startswith('spec/registry/')}
        before_path = R / 'target' / (P + f'history-{index}-before.json')
        before = json.loads(before_path.read_bytes()) if before_path.exists() else None
        if before is not None:
            assert snapshot == before
        build_args = old['command']
        row = dict(group=index, frozen_head=old['frozen_head'], snapshot_status='PASS',
                   source=str(folder), source_files=len(snapshot),
                   source_snapshot_sha256=identity(snapshot), registry_identity=identity(registry),
                   registry_files=registry, compiler_source_identity=identity(compiler),
                   expected_identity=identity(expected), expected_paths=len(expected),
                   approved_debug=0 if index == 8 else 2,
                   build_directory=build_args[build_args.index('--target-dir') + 1],
                   before_captured=before is not None, before_after_raw_and_git_exact=before is not None,
                   test_execution_credit=0, note='Identity review alone allocates no test credit')
        V.save(f'history-{index}-identity-map', snapshot)
        rows.append(row)
    V.save('historical-source-identities', dict(status='PASS', groups=rows,
           unique_build_directories=len({r['build_directory'] for r in rows})))
    assert len({r['build_directory'] for r in rows}) == 8
    print(json.dumps(dict(status='PASS', snapshots=len(rows), test_credit=0)))


if __name__ == '__main__':
    mode = sys.argv[1]
    if mode == 'specification':
        specification()
    elif mode == 'configuration':
        configuration()
    elif mode == 'eol':
        eol()
    elif mode == 'quality':
        quality()
    elif mode == 'quality-final':
        quality(final=True)
    elif mode == 'historical-identities':
        historical_identities()
    else:
        raise ValueError(mode)
