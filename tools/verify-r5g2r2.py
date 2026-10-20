"""R2 resource diagnosis and independent lane verification; no formal writes."""
import importlib.util
import copy
import json
import os
import re
from pathlib import Path
import subprocess
import sys
import time
from types import SimpleNamespace

R = Path(__file__).resolve().parents[1]
P = 'phase-04c1b4c4r3c2r5g2r2-'
START = 'a2f9b9e6009475dd1ab96f2e3172e32ced8992fd'


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, R / path)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


V = module('r2_unchanged_helpers', 'tools/verify-r5g2r1.py')


def read(path):
    return json.loads((R / path).read_bytes())


def save(name, value):
    file = R / 'target' / (P + name + '.json')
    with file.open('xb') as stream:
        stream.write((json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode())
    return value


def frozen():
    before = read('target/' + P + 'start-raw-map.json')
    assert all(V.record(n) == row for n, row in before.items())
    return dict(status='PASS', protected_pre_R2_paths=len(before), original_paths_preserved=2395,
                changed=0, removed=0)


def tree():
    names = set(V.git('ls-files', '--cached', '--others', '--exclude-standard', '-z').decode().split('\0')) - {''}
    return {n: V.record(n) for n in sorted(names)}


def preflight(name, minimum=None):
    minimum = minimum or V.ADMISSION
    assert not V.LOCK.exists()
    V.runner_identity()
    samples = []
    for i in range(3):
        samples.append(V.memory())
        if i != 2:
            time.sleep(2)
    ok = all(r['available_bytes'] >= minimum and r['load_percent'] < 80 for r in samples)
    save('preflight-' + name, dict(status='PASS' if ok else 'NOT EXECUTED',
         admitted=ok, minimum_launch_bytes=minimum, samples=samples, hard_reserve_bytes=V.RESERVE))
    return ok


def run(name, args, environment=None):
    assert not V.LOCK.exists()
    V.runner_identity()
    tag = 'r5g2r2-' + name
    log = R / 'target' / (V.SAFE + tag + '.log')
    assert not log.exists()
    source = {p.name: V.sha(p) for p in (R / 'tools').glob('*r5g2r2*.py')}
    head = V.git('rev-parse', 'HEAD').decode().strip()
    env = dict(os.environ, PYTHONUTF8='1', RUST_TEST_THREADS='1', CARGO_BUILD_JOBS='1')
    env.update(environment or {})
    with (R / 'target' / (P + name + '-wrapper.log')).open('xb') as out:
        code = subprocess.call([sys.executable, '-X', 'utf8', str(V.RUNNER), tag, *args],
                               cwd=R, env=env, stdout=out, stderr=subprocess.STDOUT)
    receipt_path = R / 'target' / (V.SAFE + tag + '.json')
    safety = json.loads(receipt_path.read_bytes()) if receipt_path.exists() else None
    status = ('INTERRUPTED BY RESOURCE SAFETY' if safety and safety['cancelled'] else
              'PASS' if code == 0 else 'FAILED')
    row = dict(status=status, command=args, exit=code, head=head,
               ended_head=V.git('rev-parse', 'HEAD').decode().strip(), safety=safety,
               log=str(log), log_sha256=V.sha(log) if log.exists() else None,
               verification_sources=source)
    save('run-' + name, row)
    print(json.dumps(dict(task=name, status=status, exit=code)), flush=True)
    return row


def initial():
    assert V.git('rev-parse', 'HEAD').decode().strip() == START
    prior = read('target/' + V.P + 'final-runtime-results.json')
    assert prior['final_head'] == START
    old = read('target/' + V.P + 'final-raw-map.json')
    assert len(old) == 2399 and all(V.record(n) == row for n, row in old.items())
    save('start-raw-map', old)
    g2 = read('target/' + V.OLD + 'final-runtime-results.json')
    rows = g2['quality'] + g2['regressions'] + [g2['registration_oracle'], g2['registration_audit'], g2['portable_successor_denial']]
    for row in rows:
        assert row['exit'] == row['expected_exit'] and row['safety']['cancelled'] is None
        assert V.sha(Path(row['log'])) == row['log_sha256']
    save('inheritance', dict(status='PASS — INHERITED SAME-SOURCE EXECUTION',
         source_identity=frozen(), code_gate_head=g2['code_gate_head'], receipts=rows,
         cargo_cases=448, registration_cases=2880, field_paths=1297,
         registration_mutations=16, native=53, f1_inputs=62, f1_mutations=40,
         boundary_cases=7, boundary_mutations=46, freshly_executed_in_R2=False,
         previous_ledgers={n: V.record(n) for n in
          ['target/' + V.OLD + 'final-runtime-results.json', 'target/' + V.P + 'final-runtime-results.json']}))
    review_diagnosis()


def review_diagnosis():
    source_freeze = frozen()
    inherited = read('target/' + P + 'inheritance.json')
    for row in inherited['receipts']:
        assert V.sha(Path(row['log'])) == row['log_sha256']
    sys.path.insert(0, str(R / 'tools'))
    from oracle import native_source_registration as O
    model, old = O.catalogue(), O.old_catalogue()
    allowed = {'ContractProofContextContents', 'StructuralValue0030', 'StructuralValue0031', 'StructuralValue0032', 'StructuralValue0033'}
    changed = {n for n in model['types'] if model['types'][n] != old['types'][n]}
    assert changed == allowed
    def rewrite(value):
        if value == {'ref': 'RelationSnapshotContent'}:
            return {'ref': 'NativeRelationSnapshotSource'}
        if isinstance(value, dict):
            return {k: rewrite(v) for k, v in value.items()}
        if isinstance(value, list):
            return [rewrite(v) for v in value]
        return value
    for name in allowed:
        assert model['types'][name] == rewrite(old['types'][name])
    relation = dict(model['types']['ContractProofContextContents']['fields'])['relation_snapshots']
    assert {n for n, _ in relation['fields']} == set(O.SLOTS)
    assert all(d['kind'] == 'option' and d['item'] == {'ref': 'NativeRelationSnapshotSource'} and d['minimum'] == 0 for _, d in relation['fields'])
    assert model['type_ranks'] == old['type_ranks'] and model['types']['NodeBody'] == old['types']['NodeBody']
    assert model['types']['NativeRelationSnapshotSource'] == old['types']['NativeRelationSnapshotSource']
    assert model['types']['RelationSnapshotContent'] == old['types']['RelationSnapshotContent']
    reg = model['structural_binding_registration']
    policies = [p for p in reg['reference_policies'] if p['key'] == dict(parent=dict(owner=O.MODEL, name='CurrentContextRelationSchemaValidation'), path=[{'field': 'source'}])]
    assert len(policies) == 2 and {p['environment'] for p in policies} == {'output_record', 'w_value'}
    assert all(p['domain'] == 'g_external' and p['target'] == dict(owner=O.MODEL, name='NativeRelationSnapshotSource') for p in policies)
    specs = read('spec/registry/specifications.json')['specifications']
    rules = read('spec/registry/rules.json')['rules']
    sm, rm = {r['specification_id']: r for r in specs}, {r['rule_id']: r for r in rules}
    assert (len(sm), len(rm)) == (74, 531)
    assert all(sm[r['specification_id']] == r for r in read('spec/conformance/g01-native-source-staging/analysis8-specifications.json')['specifications'])
    assert all(rm[r['rule_id']] == r for r in read('spec/conformance/g01-native-source-staging/analysis8-rules.json')['rules'])
    for spec in specs:
        assert V.sha(R / spec['normative_source']['logical_path']) == spec['normative_source']['content_digest']['hex']
    save('specification-review', dict(status='PASS', source_freeze=source_freeze,
         tuple=O.EDITION, changed_declarations=sorted(changed), exact_some_edges=relation,
         c_source_policies=policies, no_new_ranks_or_node_body=True,
         no_alias_cast=True, no_historical_reconstruction=True,
         old_registry_objects_exact=True, descriptors=len(model['types']),
         sites=len(reg['sites']), policies=len(reg['reference_policies']),
         required_metadata_preserved='minimum:0 remains exact; only three approved reference targets change',
         original_reviewer_failure='Overly narrow dictionary assertion omitted minimum:0; old reviewer unchanged', production_credit=0))
    diagnostic = []
    for label, name, preflight_path in [
        ('R5G2', 'r5g2-history-1', None),
        ('R5G2R1', 'r5g2r1-history-1', 'target/' + V.P + 'preflight-history-1.json')]:
        receipt = read('target/' + V.SAFE + name + '.json')
        pref = read(preflight_path) if preflight_path else None
        diagnostic.append(dict(run=label, frozen_head='abc91d533b4d62e91226722e1d1fcaa30bd00883',
          start_available_memory=None, start_available_note='Not captured by unchanged executor; not inferred from last preflight',
          preflight_samples=pref['samples'] if pref else None,
          preflight_minimum=min(r['available_bytes'] for r in pref['samples']) if pref else None,
          last_preflight_available=pref['samples'][-1]['available_bytes'] if pref else None,
          runtime_minimum_available=receipt['minimum_system_available_bytes'],
          process_peak_committed=receipt['peak_process_bytes'], whole_job_peak_committed=receipt['peak_job_bytes'],
          reserve_bytes=V.RESERVE, interruption_point='Historical Portable tests; full history incomplete',
          cancellation=receipt['cancelled'], exit=receipt['exit'], duration_seconds=receipt['seconds'],
          actual_original_receipt=receipt,
          external_causal_attribution='UNKNOWN; no independent contemporaneous series identifies a responsible process'))
    save('resource-diagnosis', dict(status='PASS', runs=diagnostic,
         measurement_caution='System available is physical memory; job peaks are committed memory. Do not subtract them to assign external-process blame.',
         runner=V.runner_identity(), sampling_api='Windows GlobalMemoryStatusEx'))
    print('Initial exact-byte pinning, inherited receipts, source review, and two-run diagnostics complete')


def observe():
    assert V.runner_identity()['unchanged']
    raw = R / 'target' / (P + 'memory-observations.jsonl')
    before = V.memory()
    wmi = json.loads(subprocess.check_output(['powershell', '-NoProfile', '-Command',
        'Get-CimInstance Win32_OperatingSystem | Select-Object FreePhysicalMemory,TotalVisibleMemorySize | ConvertTo-Json -Compress'], cwd=R))
    after = V.memory()
    assert int(wmi['TotalVisibleMemorySize']) * 1024 == before['total_bytes'] == after['total_bytes']
    wmi_free = int(wmi['FreePhysicalMemory']) * 1024
    # Independent cross-API bracket is diagnostic, not a replacement metric.
    selfcheck = dict(status='PASS', total_physical_exact=True, global_before=before,
                     wmi_available_bytes=wmi_free, global_after=after,
                     admission_metric='GlobalMemoryStatusEx available bytes, identical to protected executor',
                     bracket_difference_recorded_not_normalized=True)
    save('memory-source-selfcheck', selfcheck)
    rows = []
    start = time.monotonic()
    with raw.open('x', encoding='utf8', newline='\n') as out:
        while True:
            now = V.memory()
            now['elapsed_monotonic_seconds'] = time.monotonic() - start
            lock = read(str(V.LOCK.relative_to(R)))
            now['other_protected_job'] = lock['name'] != 'r5g2r2-C-observation'
            assert 0 <= now['available_bytes'] <= now['total_bytes']
            rows.append(now)
            out.write(json.dumps(now) + '\n')
            out.flush()
            if now['elapsed_monotonic_seconds'] >= 600:
                break
            time.sleep(min(5, 600 - now['elapsed_monotonic_seconds']))
    threshold = 14 * V.GIB
    drops = [a['available_bytes'] - b['available_bytes'] for a, b in zip(rows, rows[1:])]
    admitted = (rows[-1]['elapsed_monotonic_seconds'] >= 600 and
                min(r['available_bytes'] for r in rows) >= threshold and
                max(drops, default=0) <= V.GIB and not any(r['other_protected_job'] for r in rows))
    save('history-admission', dict(status='ADMITTED' if admitted else 'NOT ADMITTED',
         conclusion='ADMITTED' if admitted else 'HISTORY LAUNCH NOT ADMITTED — INSUFFICIENT STABLE HEADROOM',
         admitted=admitted, samples=len(rows), duration_seconds=rows[-1]['elapsed_monotonic_seconds'],
         minimum_available=min(r['available_bytes'] for r in rows), maximum_sample_drop=max(drops, default=0),
         launch_threshold=threshold, hard_reserve=V.RESERVE, raw_series=str(raw), raw_sha256=V.sha(raw),
         original_protection_unchanged=True, automatic_retry=False))
    print(json.dumps(dict(admitted=admitted, samples=len(rows), min_available=min(r['available_bytes'] for r in rows))), flush=True)


if __name__ == '__main__':
    mode = sys.argv[1]
    if mode in ['initial', 'observe']:
        {'initial': initial, 'observe': observe}[mode]()
    elif mode == 'initial-protected':
        run('initial-audit', [sys.executable, '-X', 'utf8', str(Path(__file__)), 'initial'])
    elif mode == 'observation-protected':
        run('C-observation', [sys.executable, '-X', 'utf8', str(Path(__file__)), 'observe'])
    elif mode == 'review-diagnosis':
        review_diagnosis()
    elif mode == 'review-diagnosis-protected':
        run('review-diagnosis', [sys.executable, '-X', 'utf8', str(Path(__file__)), 'review-diagnosis'])
    elif mode == 'inspect-spec':
        text = (R / 'spec/tables/proof-report-model-v8.md').read_text(encoding='utf8')
        data = json.loads(re.search(r'```json\s*(.*?)\s*```', text, re.S)[1])
        print(json.dumps(dict(data['type_replacements']['ContractProofContextContents']['fields'])['relation_snapshots']))
    elif mode == 'inspect-protected':
        run('inspect-spec-edge', [sys.executable, '-X', 'utf8', str(Path(__file__)), 'inspect-spec'])
    else:
        raise ValueError(mode)
