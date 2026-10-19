"""R5G1 serial, capped gates and raw-byte freeze. Never registers or edits production.

The scheduler stays idle while the committed Windows Job runner owns each one
workload. Every semantic command is capped; only bounded streaming hashes and
receipt bookkeeping run in this scheduler. No historical rebuild is implied.
"""
import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path

R = Path(__file__).resolve().parents[1]
P = 'phase-04c1b4c4r3c2r5g1-'
G3 = 'phase-04c1b4c4r3c2r4g3-'
G2 = 'phase-04c1b4c4r3c2r4g2-'
OLD = 'phase-04c1b4c4r3c2r5-'
BASE = '774ff6046ec9f3fbd6a2b84101b6e72faf4bbe2f'
os.environ.update(PYTHONUTF8='1', RUST_TEST_THREADS='1', CARGO_BUILD_JOBS='1')


def read(path):
    return json.loads((R / path).read_bytes())


def save(name, value, docs=False):
    (R / ('docs/implementation' if docs else 'target') / (P + name + '.json')).write_bytes(
        (json.dumps(value, indent=2, ensure_ascii=False) + '\n').encode('utf8'))


def git(*args):
    return subprocess.check_output(['git', *args], cwd=R)


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        while chunk := f.read(1048576):
            h.update(chunk)
    return h.hexdigest()


def rawmap():
    names = set(git('ls-files', '--cached', '--others', '--exclude-standard', '-z').decode().split('\0')) - {''}
    return {n: digest(R / n) for n in sorted(names)}


def head():
    return git('rev-parse', 'HEAD').decode().strip()


def run(name, args, expected=0):
    name = 'r5g1-' + name
    start = time.time()
    wrapper = R / 'target' / (P + name + '-wrapper.log')
    with wrapper.open('wb') as f:
        p = subprocess.run([sys.executable, str(R / 'tools/run-production-input-graph-safe.py'),
                            name, *args], cwd=R, stdout=f, stderr=subprocess.STDOUT)
    safe = read('target/' + G3 + 'safe-' + name + '.json')
    row = dict(name=name, command=args, head=head(), exit=p.returncode, expected_exit=expected,
               started_unix=start, seconds=round(time.time() - start, 2), safety=safe,
               log=safe['log'], log_sha256=digest(Path(safe['log'])))
    save(name, row)
    print(json.dumps({k: row[k] for k in ['name', 'exit', 'seconds']}), flush=True)
    assert safe['cancelled'] is None and safe['limits_verified'], row
    assert p.returncode == expected, row
    return row


def py(name, script, *args, expected=0):
    return run(name, [sys.executable, '-X', 'utf8', script, *args], expected)


def freeze(write=False):
    base = read('target/' + P + 'start-raw-map.json')
    now = rawmap()
    assert len(base) == 2343
    changed = [n for n in base if now.get(n) != base[n]]
    assert not changed, changed
    additions = sorted(set(now) - set(base))
    allowed = {'tools/oracle/native_relation_source_design.py', 'tools/audit-native-relation-source-design.py'}
    assert all(n in allowed or n.startswith('docs/implementation/' + P) for n in additions), additions
    result = dict(start_head=BASE, original_files=len(base), original_bytes_unchanged=True,
                  original_raw_sha256=base, added=additions, changed=[], removed=[],
                  production_changed=False, registry_changed=False, historical_oracles_expected_changed=False)
    if write:
        save('source-freeze', result, True)
    return result


def quality():
    freeze()
    assert not git('status', '--porcelain', '--', 'crates', 'spec')
    save('code-head', dict(head=head(), started_unix=time.time(), raw_map=rawmap()))
    rows = []
    for name, argv in [
        ('fmt', ['cargo', 'fmt', '--all', '--', '--check']),
        ('clippy', ['cargo', 'clippy', '-j', '2', '--all-targets', '--all-features', '--', '-D', 'warnings']),
        ('tests', ['cargo', 'test', '--all', '-j', '2']),
        ('spec', ['cargo', 'run', '-p', 'af-spec', '--bin', 'spec-check', '--', 'check']),
    ]:
        rows.append(run(name, argv))
        save('quality', rows)


def independent():
    quality_rows = read('target/' + P + 'quality.json')
    assert len(quality_rows) == 4 and all(r['exit'] == 0 for r in quality_rows)
    epoch = read('target/' + P + 'code-head.json')['started_unix']
    rows = []

    def payload(name, script, actual):
        assert (R / actual).stat().st_mtime >= epoch, ('stale Actual', actual)
        return py(name, 'tools/audit-native-relation-source-design.py', '--payload', script, actual)

    rows.append(payload('registration', 'tools/oracle/structural_binding_registration.py',
                        'target/' + G2 + 'registration-actual.json'))
    rows.append(py('registration-audit', 'tools/audit-structural-binding-staging.py', '--receipts'))
    reg = json.loads(Path(rows[-1]['log']).read_bytes())
    assert reg['fresh_execution_receipts']['executed_case_ids'] == 3662
    rows.append(payload('native', 'tools/oracle/native_guard_capture.py',
                        'target/phase-04c1b4c4r3f1-native-actual.json'))
    native = json.loads(Path(rows[-1]['log']).read_bytes())
    assert native['matched'] == 53
    rows.append(payload('f1', 'tools/oracle/bytes_guard_repair.py',
                        'target/phase-04c1b4c4r3f1-supplement-actual.json'))
    f1 = json.loads(Path(rows[-1]['log']).read_bytes())
    assert (f1['lowering_cases'], f1['native_pipeline_cases'], len(f1['mutations'])) == (26, 36, 40)
    assert all((R / 'target' / (G3 + f'shard-{i}.json')).stat().st_mtime >= epoch for i in range(4))
    rows.append(py('g3', 'tools/oracle/production_input_graph.py', '--all'))
    rows.append(py('g3-mutations', 'tools/oracle/production_input_graph.py', '--mutations'))
    assert json.loads(Path(rows[-1]['log']).read_bytes())['mutations'] == 59
    rows.append(py('g3-audit', 'tools/audit-production-input-graph.py', '--receipts'))
    assert (R / 'target' / (OLD + 'boundary-actual.json')).stat().st_mtime >= epoch
    rows.append(py('boundary', 'tools/oracle/raw_model7_full_field.py', '--boundary'))
    b = json.loads(Path(rows[-1]['log']).read_bytes())
    assert (b['boundary_cases'], b['boundary_mutations_rejected']) == (7, 46)
    rows.append(py('design-materialize', 'tools/oracle/native_relation_source_design.py', '--materialize', '--diagnostics'))
    rows.append(py('design-check', 'tools/oracle/native_relation_source_design.py', '--diagnostics'))
    rows.append(py('old-total', 'tools/oracle/raw_model6_binding.py', '--gate', 'total', expected=2))
    rows.append(py('new-total', 'tools/oracle/raw_model7_full_field.py', '--gate', 'total', expected=2))
    save('independent', dict(runs=rows, registration_case_ids=3662, native=native, f1=f1,
                            g3_positive=154, g3_negative=26, g3_mutations=59,
                            boundary_cases=7, boundary_mutations=46, design_vectors=49,
                            complete_raw_case_ids=0, full_field_mutations=0,
                            old_strict_total_exit=2, new_strict_total_exit=2,
                            capability_denial=reg['fresh_execution_receipts']))


def generators():
    assert Path.cwd().resolve() == R and str(R).lower() == 'e:\\projects\\agent'
    assert not git('status', '--porcelain')
    freeze()
    base = rawmap()
    save('generators-baseline', base)
    scripts = sorted((R / 'tools').glob('gen-*.py'))
    results = []
    for rnd in [1, 2]:
        runs = []
        for i, script in enumerate(scripts, 1):
            runs.append(py(f'gen-{rnd}-{i}', str(script)))
        now = rawmap()
        save('generators-round' + str(rnd), now)
        delta = dict(added=sorted(set(now) - set(base)), removed=sorted(set(base) - set(now)),
                     changed=[n for n in base.keys() & now.keys() if base[n] != now[n]])
        results.append(dict(round=rnd, scripts=len(scripts), runs=runs, delta=delta))
        save('generators', results)
        assert not any(delta.values()), delta
    print('TWO COMPLETE ORIGINAL-DIRECTORY ROUNDS', len(scripts), len(base), flush=True)


def finalize():
    assert not git('status', '--porcelain')
    freeze()
    quality_rows = read('target/' + P + 'quality.json')
    assert len(quality_rows) == 4 and all(r['exit'] == 0 for r in quality_rows)
    code = read('target/' + P + 'code-head.json')['head']
    assert not git('diff', '--name-only', code, 'HEAD', '--', 'crates', 'tools', 'spec')
    matches = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',
                         Path(quality_rows[2]['log']).read_text(encoding='utf8'))
    assert matches and all(m[1:] == ('0', '0') for m in matches)
    independent_rows = read('target/' + P + 'independent.json')
    rounds = read('target/' + P + 'generators.json')
    assert len(rounds) == 2 and all(not any(r['delta'].values()) and all(x['exit'] == 0 for x in r['runs']) for r in rounds)
    final = rawmap()
    assert final == read('target/' + P + 'generators-baseline.json') == read('target/' + P + 'generators-round1.json') == read('target/' + P + 'generators-round2.json')
    save('generators-final', final)
    report = (R / 'docs/implementation' / (P + 'final-report.md')).read_text(encoding='utf8')
    assert [int(n) for n in re.findall(r'^(\d+)\. ', report, re.M)] == list(range(1, 43))
    receipt = dict(outcome='A — FORMAL AMENDMENT DESIGN ACCEPTED', start_head=BASE,
                   code_gate_head=code, final_head=head(), quality=quality_rows,
                   cargo_pass_records=sum(int(m[0]) for m in matches), cargo_result_groups=len(matches),
                   independent=independent_rows, generators=dict(scripts=rounds[0]['scripts'], rounds=rounds,
                       raw_files=len(final), baseline_round1_round2_final_equal=True, added=0, removed=0, changed=0),
                   historical_builds='Not Rerun — Referenced R5 Frozen Evidence',
                   historical_receipt_sha256=digest(R / 'target' / (OLD + 'raw-model7-final-runtime-results.json')),
                   source_freeze=read('docs/implementation/' + P + 'source-freeze.json'),
                   formal_registration='PENDING', production_implementation='NOT AUTHORIZED',
                   raw_full_field='INCOMPLETE', native_full_seed='INCOMPLETE', standalone='INCOMPLETE',
                   other_t3='INCOMPLETE', total_t3='INCOMPLETE', t4='NOT ELIGIBLE', root_export='NotAuthorized',
                   clean_worktree=True, push=False, pending_background_jobs='none after this final bounded seal exits',
                   commits=git('log', '--reverse', '--format=%H %s', BASE + '..HEAD').decode().splitlines(),
                   safety_runner_sha256=digest(R / 'tools/run-production-input-graph-safe.py'))
    save('final-runtime-results', receipt)
    print(json.dumps({k: receipt[k] for k in ['outcome', 'code_gate_head', 'final_head', 'cargo_pass_records', 'clean_worktree', 'push']}))


if __name__ == '__main__':
    c = argparse.ArgumentParser()
    c.add_argument('--quality', action='store_true')
    c.add_argument('--independent', action='store_true')
    c.add_argument('--generators', action='store_true')
    c.add_argument('--freeze', action='store_true')
    c.add_argument('--finalize', action='store_true')
    c.add_argument('--payload', nargs=2)
    a = c.parse_args()
    if a.payload:
        script, actual = a.payload
        sys.exit(subprocess.run([sys.executable, '-X', 'utf8', str(R / script)],
                                cwd=R, input=(R / actual).read_bytes()).returncode)
    elif a.quality:
        quality()
    elif a.independent:
        independent()
    elif a.generators:
        generators()
    elif a.freeze:
        print(json.dumps({k: v for k, v in freeze(True).items() if k != 'original_raw_sha256'}))
    elif a.finalize:
        finalize()
    else:
        c.error('explicit mode required')
