"""R5G2R1 verification only. Never changes formal sources or historical inputs."""
import ctypes as C
from ctypes import wintypes as W
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

R = Path(__file__).resolve().parents[1]
P = 'phase-04c1b4c4r3c2r5g2r1-'
OLD = 'phase-04c1b4c4r3c2r5g2-'
START = '0910c1337c6a17100b4ff7a5e775317866b52a20'
CODE = '7a79a5f903172294cfb38296fb7360b8d8c0660c'
RUNNER = R / 'tools/run-production-input-graph-safe.py'
SAFE = 'phase-04c1b4c4r3c2r4g3-safe-'
LOCK = R / 'target' / (SAFE + 'active.json')
GIB = 1024 ** 3
RESERVE = 8 * GIB
ADMISSION = RESERVE + 1536 * 1024 ** 2


def read(path):
    return json.loads((R / path).read_bytes())


def save(name, value):
    path = R / 'target' / (P + name + '.json')
    data = (json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode()
    if path.exists():
        raise RuntimeError(('Refuse overwriting an R1 receipt', str(path)))
    path.write_bytes(data)
    return value


def git(*args):
    return subprocess.check_output(['git', *args], cwd=R)


def sha(path):
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        while chunk := stream.read(1024 * 1024):
            digest.update(chunk)
    return digest.hexdigest()


def record(path):
    file = R / path
    return dict(present=file.is_file(), byte_length=file.stat().st_size,
                raw_sha256=sha(file))


def memory():
    class Status(C.Structure):
        _fields_ = [('length', W.DWORD), ('load', W.DWORD)] + [
            (name, C.c_ulonglong) for name in
            ['total', 'available', 'page_total', 'page_available',
             'virtual_total', 'virtual_available', 'extended']]
    status = Status()
    status.length = C.sizeof(status)
    kernel = C.WinDLL('kernel32', use_last_error=True)
    kernel.GlobalMemoryStatusEx.argtypes = [C.POINTER(Status)]
    kernel.GlobalMemoryStatusEx.restype = W.BOOL
    if not kernel.GlobalMemoryStatusEx(C.byref(status)):
        raise C.WinError(C.get_last_error())
    return dict(time_unix=time.time(), available_bytes=status.available,
                load_percent=status.load, total_bytes=status.total)


def processes():
    command = """$items = @(Get-CimInstance Win32_Process -Filter "Name = 'python.exe' OR Name = 'cargo.exe' OR Name = 'rustc.exe' OR Name LIKE 'af_portable%'"); $owned = @($items | Where-Object { $_.ProcessId -ne __CURRENT_PID__ -and $_.CommandLine -match '(r5g2r1|phase-04c1b4c3-legacy-abc91d5|r5g2-history)' } | Select-Object ProcessId,ParentProcessId,Name,CommandLine); $usage = @(Get-Process | Sort-Object WorkingSet64 -Descending | Select-Object -First 8 Id,ProcessName,WorkingSet64,PrivateMemorySize64); @{owned=$owned; occupation=$usage} | ConvertTo-Json -Depth 5 -Compress""".replace('__CURRENT_PID__', str(os.getpid()))
    return json.loads(subprocess.check_output(
        ['powershell', '-NoProfile', '-Command', command], cwd=R))


def runner_identity():
    prior = read('target/' + OLD + 'final-runtime-results.json')
    digest = sha(RUNNER)
    assert digest == prior['safety_runner_sha256']
    return dict(path=str(RUNNER), raw_sha256=digest, unchanged=True,
                cpu_percent=5, process_mib=768, job_mib=1536,
                foreground_reserve_bytes=RESERVE, test_threads=1,
                build_jobs=1, priority='BelowNormal', top_level_jobs=1)


def preflight(name):
    runner = runner_identity()
    state = processes()
    assert not LOCK.exists(), 'Existing protected job; no launch'
    samples = []
    for index in range(3):
        samples.append(memory())
        if index != 2:
            time.sleep(2)
    admitted = (not state['owned'] and
                all(m['available_bytes'] >= ADMISSION and
                    m['load_percent'] < 80 for m in samples))
    result = dict(status='PASS' if admitted else 'NOT EXECUTED',
                  admitted=admitted, gate=name, samples=samples,
                  launch_headroom_required_bytes=ADMISSION,
                  admission_basis='8 GiB unchanged reserve plus full unchanged 1536 MiB job cap; launch policy only',
                  runner=runner, processes=state,
                  reason=None if admitted else 'Insufficient reliable headroom or existing same-stage process; gate not launched')
    save('preflight-' + name, result)
    print(json.dumps({k: result[k] for k in
                      ['gate', 'status', 'admitted', 'reason']}, ensure_ascii=False), flush=True)
    return admitted


def protected(name, command, environment=None):
    task = 'r5g2r1-' + name
    log = R / 'target' / (SAFE + task + '.log')
    assert not log.exists(), ('Preserve existing log', str(log))
    runner_identity()
    assert not LOCK.exists()
    env = dict(os.environ, PYTHONUTF8='1', RUST_TEST_THREADS='1', CARGO_BUILD_JOBS='1')
    if environment:
        env.update(environment)
    head = git('rev-parse', 'HEAD').decode().strip()
    wrapper = R / 'target' / (P + name + '-wrapper.log')
    assert not wrapper.exists()
    with wrapper.open('wb') as stream:
        exit_code = subprocess.call([sys.executable, '-X', 'utf8', str(RUNNER),
                                     task, *command], cwd=R, env=env,
                                    stdout=stream, stderr=subprocess.STDOUT)
    receipt = read('target/' + SAFE + task + '.json')
    status = ('INTERRUPTED BY RESOURCE SAFETY' if receipt['cancelled'] else
              'PASS' if exit_code == 0 else 'FAILED')
    result = dict(status=status, command=command, exit=exit_code,
                  head=head, ended_head=git('rev-parse', 'HEAD').decode().strip(),
                  log=str(log), log_sha256=sha(log), safety=receipt,
                  verification_script_sha256=sha(Path(__file__)))
    save('run-' + name, result)
    print(json.dumps(dict(gate=name, status=status, exit=exit_code)), flush=True)
    return result


def inherit():
    assert git('rev-parse', 'HEAD').decode().strip() == START
    previous = read('target/' + OLD + 'final-runtime-results.json')
    assert previous['code_gate_head'] == CODE and previous['final_head'] == START
    before = read('target/' + OLD + 'outcome-d-final-raw-map.json')
    files = {name: record(name) for name in sorted(before)}
    assert len(files) == 2395
    assert all(files[n]['raw_sha256'] == expected for n, expected in before.items())
    save('start-raw-map', files)
    changes = git('diff', '--name-only', CODE, START).decode().splitlines()
    assert changes and all(n.startswith('docs/implementation/' + OLD) for n in changes)
    source_files = {n: row for n, row in files.items()
                    if not n.startswith('docs/implementation/' + OLD)}
    proof = []
    rows = (previous['quality'] + previous['regressions'] +
            [previous['registration_oracle'], previous['registration_audit'],
             previous['portable_successor_denial']])
    for row in rows:
        assert row['head'] == CODE and row['exit'] == row['expected_exit']
        assert row['safety']['limits_verified'] and row['safety']['cancelled'] is None
        assert sha(Path(row['log'])) == row['log_sha256']
        proof.append(dict(status='PASS', credit='INHERITED SAME-SOURCE EXECUTION; NOT RERUN IN R1', receipt=row))
    output_paths = set()
    for row in rows:
        for arg in row['command']:
            if arg.startswith('target/') and arg.endswith('.json') and (R / arg).is_file():
                output_paths.add(arg)
    outputs = {p: record(p) for p in sorted(output_paths)}
    save('inheritance', dict(status='PASS', source_inputs_exact=True,
         old_pass_head=CODE, unchanged_source_inputs=source_files,
         all_prior_final_paths_unchanged=len(files), only_code_to_start_changes=changes,
         inherited=proof, current_runtime_output_inventory=outputs,
         runtime_output_inventory_scope='Current readback only; original receipts did not seal separate raw hashes for every ignored output. No claim that this inventory was produced in R1 or was an older output hash ledger.',
         prior_interruption=previous['interrupted_history'], runner=runner_identity()))
    print(json.dumps(dict(status='PASS', original_paths=len(files),
                          inherited_receipts=len(rows), rust_cases_inherited=448,
                          registration_cases_inherited=2880, rerun_cases=0)), flush=True)


def blobmap(head, folder):
    rows = {}
    for entry in filter(None, git('ls-tree', '-rz', head).split(b'\0')):
        meta, name = entry.split(b'\t')
        path = folder / name.decode()
        blob = hashlib.sha1(b'blob ' + str(path.stat().st_size).encode() + b'\0')
        with path.open('rb') as stream:
            while chunk := stream.read(1024 * 1024):
                blob.update(chunk)
        assert blob.hexdigest().encode() == meta.split()[-1], str(path)
        rows[name.decode()] = dict(git_blob=blob.hexdigest(), raw_sha256=sha(path),
                                   byte_length=path.stat().st_size, present=True)
    return rows


def histories():
    prior = read('target/phase-04c1b4c4r3c2r5-histories.json')
    outcomes = []
    for index, old in enumerate(prior, 1):
        name = 'history-' + str(index)
        if not preflight(name):
            save('history-stop', dict(outcome='D', status='NOT EXECUTED',
                 stopped_before_group=index, completed_groups=len(outcomes),
                 downstream='NOT EXECUTED', automatic_retry=False))
            return
        folder = Path(old['source'])
        # Work is inside the original approved protected job, including full
        # source hashing; original profiles and test scope are unchanged.
        debug = '0' if index == 8 else '2'
        row = protected(name, [sys.executable, '-X', 'utf8', str(Path(__file__)),
                        'history-worker', str(index)],
                        dict(CARGO_PROFILE_DEV_DEBUG=debug, CARGO_PROFILE_TEST_DEBUG=debug))
        row.update(frozen_head=old['frozen_head'], source=str(folder),
                   approved_debug=int(debug), required_case_ids=old['case_ids'])
        outcomes.append(row)
        save(name + '-result', row)
        if row['status'] != 'PASS':
            save('history-stop', dict(outcome='D', status=row['status'],
                 stopped_at_group=index, completed_groups=index - 1,
                 downstream='NOT EXECUTED', automatic_retry=False))
            return
    save('histories-complete', dict(status='PASS', completed_groups=8, rows=outcomes))


def history_worker(index):
    old = read('target/phase-04c1b4c4r3c2r5-histories.json')[index - 1]
    folder = Path(old['source'])
    before = blobmap(old['frozen_head'], folder)
    save('history-' + str(index) + '-before', before)
    args = old['command']
    build = args[args.index('--target-dir') + 1]
    command = ['cargo', 'test', '--all', '-j', '1', '--manifest-path',
               str(folder / 'Cargo.toml'), '--target-dir', build]
    exit_code = subprocess.call(command, cwd=folder)
    after = blobmap(old['frozen_head'], folder)
    save('history-' + str(index) + '-after', after)
    assert before == after
    # Historical auditors are the exact original scripts in their own tree.
    if exit_code == 0:
        auditors = {3: ('audit-substitution-staging.py', []),
                    7: ('audit-relation-schema-staging.py', ['--receipts']),
                    8: ('audit-structural-binding-staging.py', ['--receipts'])}
        if index in auditors:
            script, flags = auditors[index]
            exit_code = subprocess.call([sys.executable, '-X', 'utf8',
                                        str(folder / 'tools' / script), *flags], cwd=folder)
        assert after == blobmap(old['frozen_head'], folder)
    sys.exit(exit_code)


if __name__ == '__main__':
    mode = sys.argv[1]
    if mode == 'inherit':
        inherit()
    elif mode == 'preflight':
        sys.exit(0 if preflight(sys.argv[2]) else 2)
    elif mode == 'protected-inherit':
        protected('inheritance-audit', [sys.executable, '-X', 'utf8', str(Path(__file__)), 'inherit'])
    elif mode == 'histories':
        histories()
    elif mode == 'history-worker':
        history_worker(int(sys.argv[2]))
    else:
        raise ValueError(mode)
