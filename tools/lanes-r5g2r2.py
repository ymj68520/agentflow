"""Independent frozen-G3 and original-directory generator lanes, R2 only."""
import importlib.util
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys
import tarfile
import time

R = Path(__file__).resolve().parents[1]
s = importlib.util.spec_from_file_location('r2_lanes_helpers', R / 'tools/verify-r5g2r2.py')
H = importlib.util.module_from_spec(s)
s.loader.exec_module(H)
P, V = H.P, H.V
G3 = 'a9e296407c682a69f596be84ac8974eb870ebc18'
F = R / 'target' / (P + 'frozen-g3/source')


def command(name, args, cwd, stdout_file=None):
    log = R / 'target' / (P + name + '.log')
    start = time.time()
    with log.open('xb') as output:
        code = subprocess.call(args, cwd=cwd, stdout=output, stderr=subprocess.STDOUT)
    row = dict(name=name, command=args, cwd=str(cwd), exit=code,
               status='PASS' if code == 0 else 'FAILED', started_unix=start,
               seconds=time.time() - start, log=str(log), log_sha256=V.sha(log),
               resource_scope='Inherited unchanged enclosing protected Job; no breakaway')
    H.save(name + '-command', row)
    if stdout_file and code == 0:
        with stdout_file.open('xb') as out:
            out.write(log.read_bytes())
    print(json.dumps(dict(command=name, exit=code)), flush=True)
    return row


def generators():
    assert Path.cwd().resolve() == R and not V.git('status', '--porcelain')
    base = H.tree()
    H.save('B-baseline', base)
    scripts = sorted((R / 'tools').glob('gen-*.py'))
    H.save('B-generator-roster', [dict(path=p.relative_to(R).as_posix(), **V.record(p.relative_to(R).as_posix())) for p in scripts])
    rounds = []
    for number in [1, 2]:
        rows = []
        for index, script in enumerate(scripts, 1):
            row = command(f'B-round{number}-{index:02}', [sys.executable, '-X', 'utf8', str(script)], R)
            rows.append(row)
            if row['exit'] != 0:
                H.save('B-failure', dict(status='FAILED', round=number, index=index,
                                        reason='Generator did not exit 0', row=row))
                sys.exit(row['exit'])
        now = H.tree()
        H.save('B-round' + str(number), now)
        delta = dict(added=sorted(now.keys() - base.keys()), removed=sorted(base.keys() - now.keys()),
                     changed=sorted(n for n in now.keys() & base.keys() if now[n] != base[n]))
        rounds.append(dict(round=number, commands=rows, delta=delta))
        H.save('B-round' + str(number) + '-result', rounds[-1])
        if any(delta.values()):
            H.save('B-failure', dict(status='FAILED', reason='Raw-byte/path drift; no restoration performed', delta=delta))
            sys.exit(3)
    final = H.tree()
    H.save('B-final', final)
    assert final == base == H.read('target/' + P + 'B-round1.json') == H.read('target/' + P + 'B-round2.json')
    H.save('B-complete', dict(status='PASS', directory=str(R), generator_count=len(scripts),
           commands=2 * len(scripts), rounds=rounds, raw_maps_equal=True,
           source_paths=len(final), added=0, removed=0, changed=0))


def frozen_g3():
    assert not F.exists(), 'Do not overwrite a previous Lane A source/receipt'
    F.parent.mkdir(parents=True)
    archive = F.parent / 'source.tar'
    subprocess.check_call(['git', 'archive', '--format=tar', '--output', str(archive), G3], cwd=R)
    F.mkdir()
    with tarfile.open(archive, mode='r|') as source:
        for item in source:
            path = PurePosixPath(item.name)
            assert not path.is_absolute() and '..' not in path.parts and not item.issym() and not item.islnk()
            source.extract(item, F, filter='data')
    before = V.blobmap(G3, F)
    H.save('A-source-before', before)
    (F / 'target').mkdir(exist_ok=True)
    rows = []
    # Exact original G3 Debug configuration; no low-debug retry is authorized.
    assert os.environ['CARGO_PROFILE_DEV_DEBUG'] == os.environ['CARGO_PROFILE_TEST_DEBUG'] == '2'
    rows.append(command('A-cargo', ['cargo', 'test', '--all', '-j', '1', '--manifest-path',
                        str(F / 'Cargo.toml'), '--target-dir', str(F.parent / 'build')], F))
    if rows[-1]['exit']:
        H.save('A-failure', dict(status='FAILED', reason='Frozen G3 Cargo did not complete', row=rows[-1]))
        sys.exit(rows[-1]['exit'])
    rows.append(command('A-oracle', [sys.executable, '-X', 'utf8', 'tools/oracle/production_input_graph.py', '--all'], F))
    if rows[-1]['exit']:
        sys.exit(rows[-1]['exit'])
    rows.append(command('A-mutations', [sys.executable, '-X', 'utf8', 'tools/oracle/production_input_graph.py', '--mutations'], F,
                F / 'target/phase-04c1b4c4r3c2r4g3-mutations.json'))
    if rows[-1]['exit']:
        sys.exit(rows[-1]['exit'])
    rows.append(command('A-receipts', [sys.executable, '-X', 'utf8', str(Path(__file__)), 'A-evidence'], F))
    if rows[-1]['exit']:
        sys.exit(rows[-1]['exit'])
    after = V.blobmap(G3, F)
    H.save('A-source-after', after)
    assert after == before
    graph = json.loads((F / 'target/phase-04c1b4c4r3c2r4g3-input-g-oracle-results.json').read_bytes())
    assert graph['case_ids'] == 154 and graph['required'] == graph['compared']
    assert graph['missing'] == graph['unexpected'] == graph['uncompared'] == 0
    mutations = json.loads((F / 'target/phase-04c1b4c4r3c2r4g3-mutations.json').read_bytes())
    assert mutations['mutations'] == 59 and mutations['unexpected_accepted'] == 0
    actual_files = {p.relative_to(F).as_posix(): dict(raw_sha256=V.sha(p), byte_length=p.stat().st_size)
                    for p in (F / 'target').glob('phase-04c1b4c4r3c2r4g3-*') if p.is_file()}
    H.save('A-complete', dict(status='PASS', label='FROZEN G3 REGRESSION — PASS',
           frozen_head=G3, source=str(F), archive_sha256=V.sha(archive),
           debug_profile=2, source_before_after_equal=True, positives=154, negatives=26,
           comparator_mutations=59, commands=rows, runtime=graph, actual_files=actual_files,
           successor_binder_credit=0, raw_full_field_credit=0, root_export='NotAuthorized'))


def frozen_evidence():
    # Use the exact frozen auditor's semantic/receipt checks, not its old
    # CRLF worktree ledger. Full frozen HEAD Git-Blob/raw equality is stronger
    # and checked separately; no historical ledger/Expected is rewritten.
    sys.path.insert(0, str(F / 'tools'))
    spec = importlib.util.spec_from_file_location('exact_frozen_g3_auditor', F / 'tools/audit-production-input-graph.py')
    audit = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(audit)
    result = audit.receipts(audit.manifests())
    assert result['production_case_ids'] == 154
    assert result['source_negative_case_ids'] + result['resource_nominal_boundaries'] == 26
    assert result['comparator_mutations'] == 59
    print(json.dumps(result))


def lane(name):
    if name == 'B':
        assert not V.git('status', '--porcelain')
    if not H.preflight(name):
        H.save('lane-' + name, dict(status='NOT EXECUTED', reason='Lane launch preflight lacked safe headroom', automatic_retry=False))
        return
    environment = dict(CARGO_PROFILE_DEV_DEBUG='2', CARGO_PROFILE_TEST_DEBUG='2') if name == 'A' else None
    row = H.run('lane-' + name, [sys.executable, '-X', 'utf8', str(Path(__file__)), name + '-worker'], environment)
    H.save('lane-' + name, row)


def postcheck_a():
    before = H.read('target/' + P + 'A-source-before.json')
    after = V.blobmap(G3, F)
    assert before == after
    H.save('A-source-after-resource-stop', after)
    raw = H.read('target/' + P + 'lane-A.json')
    text = (R / 'target' / (P + 'A-cargo.log')).read_text(encoding='utf8')
    safe = raw['safety']
    cap = safe['process_memory_mib'] * 1024 ** 2
    assert raw['exit'] == 101 and 'rustc-LLVM ERROR: out of memory' in text
    assert 0 <= cap - safe['peak_process_bytes'] <= 128 * 1024
    assert safe['cancelled'] is None and safe['minimum_system_available_bytes'] > V.RESERVE
    H.save('A-resource-adjudication', dict(status='INTERRUPTED BY RESOURCE SAFETY',
         raw_execution_status=raw['status'], raw_exit=101, raw_receipt='target/' + P + 'lane-A.json',
         observed='LLVM allocation failure during af-computation compilation; no tests executed',
         diagnosis='Strongly consistent with unchanged per-process hard memory limit; not a semantic assertion failure',
         supervisor_foreground_cancellation=None, process_limit_bytes=cap,
         process_peak_bytes=safe['peak_process_bytes'], distance_to_cap=cap - safe['peak_process_bytes'],
         source_git_blob_and_raw_unchanged=True, debug_profile=2, profile_changed=False,
         positive_credit=0, negative_credit=0, comparator_credit=0, automatic_retry=False))
    print(json.dumps(dict(status='PASS', source_files=len(after), lane_status='INTERRUPTED BY RESOURCE SAFETY', raw_exit=101)))


if __name__ == '__main__':
    mode = sys.argv[1]
    if mode in ['A', 'B']:
        lane(mode)
    elif mode == 'A-worker':
        frozen_g3()
    elif mode == 'A-evidence':
        frozen_evidence()
    elif mode == 'B-worker':
        generators()
    elif mode == 'A-postcheck':
        postcheck_a()
    elif mode == 'A-postcheck-protected':
        H.run('A-source-resource-postcheck', [sys.executable, '-X', 'utf8', str(Path(__file__)), 'A-postcheck'])
    else:
        raise ValueError(mode)
