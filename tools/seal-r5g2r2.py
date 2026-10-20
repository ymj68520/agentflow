"""Post-lane evidence review and external R2 decision. No gate relaxation."""
import importlib.util
import json
from pathlib import Path
import sys

R = Path(__file__).resolve().parents[1]
s = importlib.util.spec_from_file_location('r2_final_helpers', R / 'tools/verify-r5g2r2.py')
H = importlib.util.module_from_spec(s)
s.loader.exec_module(H)
V, P = H.V, H.P


def decision():
    runs = {}
    for file in sorted((R / 'target').glob(P + 'run-*.json')):
        row = json.loads(file.read_bytes())
        if row['log_sha256']:
            assert V.sha(Path(row['log'])) == row['log_sha256']
        if row['safety']:
            safe = row['safety']
            assert safe['limits_verified']
            assert (safe['cpu_hard_cap_percent'], safe['process_memory_mib'], safe['whole_job_memory_mib']) == (5, 768, 1536)
        runs[file.name] = row
    admission_file = R / 'target' / (P + 'history-admission.json')
    admission = json.loads(admission_file.read_bytes()) if admission_file.exists() else dict(status='INTERRUPTED', admitted=False)
    raw_file = R / 'target' / (P + 'memory-observations.jsonl')
    samples = [json.loads(line) for line in raw_file.read_text().splitlines()] if raw_file.exists() else []
    if admission_file.exists():
        assert V.sha(raw_file) == admission['raw_sha256']
        assert len(samples) == admission['samples']
        assert samples[-1]['elapsed_monotonic_seconds'] >= 600
        assert all(a['elapsed_monotonic_seconds'] < b['elapsed_monotonic_seconds'] for a, b in zip(samples, samples[1:]))
        assert admission['minimum_available'] == min(r['available_bytes'] for r in samples)
        selfcheck = H.read('target/' + P + 'memory-source-selfcheck.json')
        assert selfcheck['status'] == 'PASS' and selfcheck['total_physical_exact']
        assert 0 <= selfcheck['wmi_available_bytes'] <= selfcheck['global_before']['total_bytes']
        bracket = [selfcheck['global_before']['available_bytes'], selfcheck['global_after']['available_bytes']]
        distance = max(min(bracket) - selfcheck['wmi_available_bytes'], selfcheck['wmi_available_bytes'] - max(bracket), 0)
        assert distance < 128 * 1024 ** 2, 'Cross-API sampling self-check requires investigation'
        if admission['admitted']:
            assert min(r['available_bytes'] for r in samples) >= 14 * V.GIB
            assert all(a['available_bytes'] - b['available_bytes'] <= V.GIB for a, b in zip(samples, samples[1:]))
    lane_states = {}
    for name in ['A', 'B']:
        file = R / 'target' / (P + 'lane-' + name + '.json')
        row = json.loads(file.read_bytes()) if file.exists() else dict(status='NOT EXECUTED')
        complete = R / 'target' / (P + name + '-complete.json')
        if row['status'] == 'PASS':
            assert complete.exists()
            assert json.loads(complete.read_bytes())['status'] == 'PASS'
        if name == 'A' and (R / 'target' / (P + 'A-resource-adjudication.json')).exists():
            row = dict(H.read('target/' + P + 'A-resource-adjudication.json'), raw_execution_receipt=row)
        lane_states[name] = row
    b_valid = False
    if (R / 'target' / (P + 'B-complete.json')).exists():
        current = H.tree()
        b_valid = all(current == H.read('target/' + P + name + '.json') for name in ['B-baseline', 'B-round1', 'B-round2', 'B-final'])
        assert b_valid, 'Lane B credit invalidated by later protected-tree change'
    history_file = R / 'target' / (P + 'C-complete.json')
    histories = json.loads(history_file.read_bytes()) if history_file.exists() else dict(status='NOT EXECUTED', completed_groups=0)
    spec = H.read('target/' + P + 'specification-review.json')
    source = H.frozen()
    accepted = (spec['status'] == 'PASS' and lane_states['A']['status'] == 'PASS' and
                lane_states['B']['status'] == 'PASS' and b_valid and histories.get('completed_groups') == 8)
    return dict(status='PASS' if accepted else 'FAILED', review_executed=True,
                outcome='A' if accepted else 'D', reason=None if accepted else 'Mandatory lanes incomplete; not a semantic assertion failure',
                lanes=lane_states, resource_admission=admission, histories=histories,
                inherited_current_gates='VALID — INHERITED SAME-SOURCE EXECUTION',
                specification_review=spec, source_preservation=source,
                lane_B_credit_valid_for_final_tree=b_valid, runs=runs)


def finalize():
    assert not V.git('status', '--porcelain')
    assert not V.LOCK.exists() or H.read(str(V.LOCK.relative_to(R)))['name'] == 'r5g2r2-final-seal'
    result = decision()
    current = H.tree()
    H.save('final-raw-map', current)
    source = H.read('target/' + P + 'start-raw-map.json')
    added = sorted(current.keys() - source.keys())
    assert all(n.startswith('docs/implementation/' + P) or
               (n.startswith('tools/') and 'r5g2r2' in n and n.endswith('.py')) for n in added)
    inheritance = H.read('target/' + P + 'inheritance.json')
    for row in inheritance['receipts']:
        assert V.sha(Path(row['log'])) == row['log_sha256']
    artifacts = {}
    for file in (R / 'target').iterdir():
        if not file.is_file() or 'final-seal' in file.name or file.name == P + 'final-runtime-results.json':
            continue
        if file.name.startswith(P) or file.name.startswith(V.SAFE + 'r5g2r2-'):
            artifacts[file.relative_to(R).as_posix()] = V.record(file.relative_to(R).as_posix())
    result.update(start_head=H.START, inherited_code_gate_head=V.CODE,
                  final_head=V.git('rev-parse', 'HEAD').decode().strip(),
                  registration_status='COMPLETE — REGISTERED STAGING' if result['outcome'] == 'A' else 'Registration Incomplete',
                  inherited_evidence=inheritance, resource_diagnosis=H.read('target/' + P + 'resource-diagnosis.json'),
                  new_paths=added, final_nonignored_paths=len(current), external_evidence_manifest=artifacts,
                  generator_final_map='target/' + P + 'final-raw-map.json',
                  successor_production_binder='NotImplemented', successor_production_dagranks=0,
                  raw_full_field='INCOMPLETE', total_t3='INCOMPLETE', t4='NOT ELIGIBLE',
                  positive_root_export='NotAuthorized', portable_positive='NotAuthorized',
                  complete_view='NotIssued', obligation_identity='NotIssued', successor_runtime_denominators=None,
                  clean_worktree=True, pushed=False, pending_background_tasks='NONE after wrapper closure; checked separately',
                  next_admission='Separate R5G3 authorization only' if result['outcome'] == 'A' else 'Separate verification environment / historical execution strategy review; no unconditional history rerun',
                  commits=V.git('log', '--reverse', '--format=%H %s', H.START + '..HEAD').decode().splitlines())
    H.save('final-runtime-results', result)
    print(json.dumps(dict(outcome=result['outcome'], final_head=result['final_head'],
                         lane_A=result['lanes']['A']['status'], lane_B=result['lanes']['B']['status'],
                         history_admission=result['resource_admission']['status']), ensure_ascii=False))


if __name__ == '__main__':
    mode = sys.argv[1]
    if mode == 'finalize':
        finalize()
    elif mode == 'seal':
        H.run('final-seal', [sys.executable, '-X', 'utf8', str(Path(__file__)), 'finalize'])
    elif mode == 'review':
        result = decision()
        H.save('quality-review', result)
        print(json.dumps(dict(status=result['status'], outcome=result['outcome'])))
    elif mode == 'review-protected':
        H.run('quality-review', [sys.executable, '-X', 'utf8', str(Path(__file__)), 'review'])
    else:
        raise ValueError(mode)
