"""Lightweight R1 Outcome D preservation and post-commit external sealing."""
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys

R = Path(__file__).resolve().parents[1]
s = importlib.util.spec_from_file_location('r1_seal_helpers', R / 'tools/verify-r5g2r1.py')
V = importlib.util.module_from_spec(s)
s.loader.exec_module(V)
P = V.P


def tree():
    names = set(V.git('ls-files', '--cached', '--others', '--exclude-standard', '-z').decode().split('\0')) - {''}
    return {name: V.record(name) for name in sorted(names)}


def preserved():
    original = V.read('target/' + P + 'start-raw-map.json')
    assert all(V.record(name) == row for name, row in original.items())
    return dict(status='PASS', old_paths=len(original), added_to_old_set=0,
                removed=0, changed=0, complete_old_raw_bytes_equal=True)


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def preserve():
    stopped = V.read('target/' + P + 'history-stop.json')
    assert stopped['status'] == 'INTERRUPTED BY RESOURCE SAFETY' and stopped['completed_groups'] == 0
    base = preserved()
    config = V.read('target/' + P + 'historical-configuration.json')
    assert V.sha(Path(config['configuration_file'])) == config['configuration_sha256']
    first = config['groups'][0]
    after = V.blobmap(first['frozen_head'], Path(first['source']))
    before = V.read('target/' + P + 'history-1-before.json')
    assert before == after
    V.save('history-1-after-interrupted', after)
    registry = {n: row for n, row in after.items() if n.startswith('spec/registry/')}
    compiler = {n: row for n, row in after.items() if n.startswith('crates/') or n in ['Cargo.toml', 'Cargo.lock']}
    expected = {n: row for n, row in after.items() if n.startswith(('spec/conformance/', 'tools/oracle/')) or 'expected' in n.lower()}
    history = dict(status='PASS', scope='Source preservation after interruption; no test credit',
                   frozen_head=first['frozen_head'], files=len(after),
                   source_snapshot_sha256=digest(after), registry_identity=digest(registry),
                   compiler_source_identity=digest(compiler), expected_identity=digest(expected),
                   git_blobs_exact=True, raw_bytes_before_after_equal=True,
                   completed_case_ids=0, test_status='INTERRUPTED BY RESOURCE SAFETY')
    V.save('history-1-preservation', history)
    roster = {p.relative_to(R).as_posix(): V.record(p.relative_to(R).as_posix())
              for p in sorted((R / 'tools').glob('gen-*.py'))}
    V.save('generator-roster', dict(status='PASS', scope='Roster only; no generator execution',
                                  scripts=roster, count=len(roster), full_rounds=0))
    V.save('specification-review', dict(status='NOT EXECUTED',
           reason='Resource-safety stop; no new semantic review launched afterward',
           formal_input_byte_identity='PASS - inherited same-source evidence',
           review_executed=False, production_credit=0))
    V.save('closeout-preservation', dict(status='PASS', root_inputs=base,
           interrupted_history=history, source_configuration_unchanged=True,
           generator_rounds='NOT EXECUTED', limits_relaxed=False))
    result = subprocess.call([sys.executable, '-X', 'utf8', str(R / 'tools/audit-r5g2r1.py'), 'quality'], cwd=R)
    assert result == 0
    print(json.dumps(dict(status='PASS', preserved_paths=base['old_paths'],
                          historical_source_files=len(after), generator_roster=len(roster), test_credit=0)))


def recheck():
    base = preserved()
    config = V.read('target/' + P + 'historical-configuration.json')
    first = config['groups'][0]
    actual = V.blobmap(first['frozen_head'], Path(first['source']))
    assert actual == V.read('target/' + P + 'history-1-before.json')
    assert actual == V.read('target/' + P + 'history-1-after-interrupted.json')
    assert digest(actual) == V.read('target/' + P + 'history-1-preservation.json')['source_snapshot_sha256']
    for inherited in V.read('target/' + P + 'inheritance.json')['inherited']:
        row = inherited['receipt']
        assert V.sha(Path(row['log'])) == row['log_sha256']
    failed = V.read('target/' + P + 'run-closeout-preservation.json')
    assert failed['status'] == 'FAILED' and failed['exit'] == 1
    assert V.sha(Path(failed['log'])) == failed['log_sha256']
    result = dict(status='PASS', scope='Lightweight preservation recheck only; not a history/gate retry',
                  root_inputs=base, historical_files=len(actual), test_credit=0,
                  first_attempt_status='FAILED',
                  first_attempt_source_sha256='d8eab8d43b3b04e63993bf14125f785060ce5f63c37fcfdfa3e313392c0f78a4',
                  first_attempt_error="NameError in final progress print: undefined 'original'; no formal data changed",
                  first_attempt_log=failed['log'], first_attempt_log_sha256=failed['log_sha256'],
                  current_sealer_sha256=V.sha(Path(__file__)))
    V.save('closeout-recheck', result)
    print(json.dumps(dict(status='PASS', protected_paths=base['old_paths'], historical_files=len(actual), test_credit=0)))


def finalize():
    assert not V.git('status', '--porcelain')
    assert not V.LOCK.exists() or V.read(str(V.LOCK.relative_to(R)))['name'] == 'r5g2r1-final-seal'
    stop = V.read('target/' + P + 'history-stop.json')
    assert stop['outcome'] == 'D' and stop['status'] == 'INTERRUPTED BY RESOURCE SAFETY'
    assert not (R / 'target' / (P + 'histories-complete.json')).exists()
    assert not (R / 'target' / (P + 'g3-complete.json')).exists()
    assert not (R / 'target' / (P + 'generators-complete.json')).exists()
    source = preserved()
    inheritance = V.read('target/' + P + 'inheritance.json')
    for inherited in inheritance['inherited']:
        row = inherited['receipt']
        assert V.sha(Path(row['log'])) == row['log_sha256']
    new_receipts = []
    for file in sorted((R / 'target').glob(P + 'run-*.json')):
        row = json.loads(file.read_bytes())
        assert V.sha(Path(row['log'])) == row['log_sha256']
        new_receipts.append(row)
    history = V.read('target/' + P + 'history-1-result.json')
    assert history['status'] == 'INTERRUPTED BY RESOURCE SAFETY' and history['exit'] == 79
    current = tree()
    V.save('final-raw-map', current)
    old = V.read('target/' + P + 'start-raw-map.json')
    added = sorted(set(current) - set(old))
    assert all(n.startswith('docs/implementation/' + P) or
               (n.startswith('tools/') and 'r5g2r1' in n and n.endswith('.py')) for n in added)
    groups = V.read('target/' + P + 'historical-configuration.json')['groups']
    first_identity = V.read('target/' + P + 'history-1-preservation.json')
    groups = [dict(**r, status='INTERRUPTED BY RESOURCE SAFETY' if r['group'] == 1 else 'NOT EXECUTED',
                   completed_case_ids=0,
                   source_snapshot_sha256=first_identity['source_snapshot_sha256'] if r['group'] == 1 else None,
                   registry_identity=first_identity['registry_identity'] if r['group'] == 1 else None,
                   compiler_source_identity=first_identity['compiler_source_identity'] if r['group'] == 1 else None,
                   expected_identity=first_identity['expected_identity'] if r['group'] == 1 else None) for r in groups]
    result = dict(outcome='D — Mandatory Gates Incomplete', status='Registration Incomplete',
                  start_head=V.START, inherited_code_gate_head=V.CODE,
                  final_head=V.git('rev-parse', 'HEAD').decode().strip(),
                  inherited_execution_only=True, inherited_cargo_passed=448,
                  inherited_cargo_groups=79, inherited_registration_cases=2880,
                  inherited_field_variant_paths=1297, inherited_registration_mutations=16,
                  inherited_receipts=inheritance['inherited'], inherited_native=53,
                  inherited_f1_inputs=62, inherited_f1_mutations=40,
                  inherited_boundary_cases=7, inherited_boundary_mutations=46,
                  strict_model6_exit=2, strict_model7_exit=2,
                  prior_interruption=inheritance['prior_interruption'],
                  new_runs=new_receipts, r1_history=history, histories=groups,
                  completed_histories=0, required_histories=8,
                  frozen_g3=dict(head='a9e296407c682a69f596be84ac8974eb870ebc18', status='NOT EXECUTED'),
                  generators=dict(status='NOT EXECUTED', complete_rounds=0, required_rounds=2,
                                  roster=V.read('target/' + P + 'generator-roster.json'),
                                  baseline=None, round1=None, round2=None,
                                  zero_drift_claim=False),
                  source_preservation=source,
                  preservation_recheck=V.read('target/' + P + 'closeout-recheck.json'),
                  interrupted_source_preservation=V.read('target/' + P + 'history-1-preservation.json'),
                  specification_review=V.read('target/' + P + 'specification-review.json'),
                  quality_review=V.read('target/' + P + 'quality-evidence-review-final.json'),
                  preflight=V.read('target/' + P + 'preflight-history-1.json'),
                  failed_verification_development=V.read('target/' + P + 'preflight-development-failure.json'),
                  new_paths=added, final_nonignored_paths=len(current),
                  final_raw_map='target/' + P + 'final-raw-map.json',
                  no_automatic_retry=True, limits_relaxed=False, unrelated_processes_killed=False,
                  successor_production_binder='NotImplemented', successor_production_dagranks=0,
                  raw_full_field='INCOMPLETE', total_t3='INCOMPLETE', t4='NOT ELIGIBLE',
                  positive_root_export='NotAuthorized', portable_positive='NotAuthorized',
                  complete_view='NotIssued', obligation_identity='NotIssued',
                  successor_runtime_denominators=None, clean_worktree=True, pushed=False,
                  pending_background_jobs='NONE after closing wrapper; independently checked after exit',
                  commits=V.git('log', '--reverse', '--format=%H %s', V.START + '..HEAD').decode().splitlines(),
                  next_stage='No automatic implementation; remaining verification requires renewed authorization',
                  verification_scripts={p.name: V.sha(p) for p in (R / 'tools').glob('*r5g2r1*.py')})
    artifacts = {}
    for file in sorted((R / 'target').iterdir()):
        if not file.is_file():
            continue
        if file.name == P + 'final-runtime-results.json' or 'r5g2r1-final-seal' in file.name or file.name.startswith(P + 'final-seal'):
            continue  # Do not hash the currently writing final log or this ledger.
        if file.name.startswith(P) or file.name.startswith(V.SAFE + 'r5g2r1-'):
            artifacts[file.relative_to(R).as_posix()] = V.record(file.relative_to(R).as_posix())
    result['external_evidence_manifest'] = artifacts
    result['generator_final_map_scope'] = 'Preservation final readback only; generator baseline/round maps NOT EXECUTED'
    V.save('final-runtime-results', result)
    print(json.dumps({k: result[k] for k in ['outcome', 'final_head', 'completed_histories', 'clean_worktree', 'pushed']}, ensure_ascii=False))


if __name__ == '__main__':
    mode = sys.argv[1]
    if mode == 'preserve':
        preserve()
    elif mode == 'finalize':
        finalize()
    elif mode == 'recheck':
        recheck()
    elif mode == 'recheck-protected':
        V.protected('closeout-recheck', [sys.executable, '-X', 'utf8', str(Path(__file__)), 'recheck'])
    elif mode == 'quality-closeout':
        V.protected('quality-closeout-final', [sys.executable, '-X', 'utf8', str(R / 'tools/audit-r5g2r1.py'), 'quality-final'])
    elif mode == 'closeout':
        V.protected('closeout-preservation', [sys.executable, '-X', 'utf8', str(Path(__file__)), 'preserve'])
    elif mode == 'seal':
        V.protected('final-seal', [sys.executable, '-X', 'utf8', str(Path(__file__)), 'finalize'])
    else:
        raise ValueError(mode)
