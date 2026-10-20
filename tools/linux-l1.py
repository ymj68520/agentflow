"""L1 local source/transport auditor. Fixed authorized SSH alias, no builds.

Run under the unchanged Windows safety envelope. Source bundle contains only
the nominated frozen HEAD's reachable Git objects, not working-tree extras.
"""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time
import tomllib

R = Path(__file__).resolve().parents[1]
D = R / 'docs/implementation'
P = 'phase-04c1b4c4r3c2r5g2-linux-l1-'
START = 'dc1d71f30b3a7f06065f4000712e6512f288db16'
P0 = 'phase-04c1b4c4r3c2r5g2r4p0-'
P1 = 'phase-04c1b4c4r3c2r5g2r4p1a-'
A = R / 'target' / (P + 'artifacts')
REMOTE = 'projects/agent-r5g2-linux-validation'
SSH = ['ssh', '-o', 'BatchMode=yes', '-o', 'StrictHostKeyChecking=yes', '-o', 'ConnectTimeout=10', 'fwq']
SCP = ['scp', '-o', 'BatchMode=yes', '-o', 'StrictHostKeyChecking=yes', '-o', 'ConnectTimeout=10']
spec = importlib.util.spec_from_file_location('l1_readonly_helpers', R / 'tools/verify-r5g2r1.py')
V = importlib.util.module_from_spec(spec)
spec.loader.exec_module(V)


def read(path):
    return json.loads((R / path).read_bytes())


def put(path, value):
    assert not path.exists(), str(path)
    path.write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode())


def doc(name, value):
    path = D / (P + name + '.json')
    path.write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode())


def run(name, argv, timeout=240):
    log = A / (name + '.log')
    assert not log.exists()
    started = time.time()
    with log.open('xb') as output:
        process = subprocess.run(argv, cwd=R, stdout=output, stderr=subprocess.STDOUT, timeout=timeout)
    row = dict(command=argv, exit=process.returncode, seconds=time.time() - started,
               log=log.relative_to(R).as_posix(), log_sha256=V.sha(log))
    put(A / (name + '.json'), row)
    assert row['exit'] == 0, row
    return row


def scp_up(name, local, relative):
    assert '..' not in Path(relative).parts
    run(name + '-absent', SSH + ['test ! -e "$HOME/' + REMOTE + '/' + relative + '" && test ! -L "$HOME/' + REMOTE + '/' + relative + '"'])
    return run(name, SCP + [str(local), 'fwq:' + REMOTE + '/' + relative])


def fetch(name):
    path = A / (name + '.json')
    assert not path.exists()
    run('download-' + name, SCP + ['fwq:' + REMOTE + '/evidence/' + name + '.json', str(path)])
    return json.loads(path.read_bytes())


def prepare():
    assert V.git('rev-parse', 'HEAD').decode().strip() == START and not V.git('diff', START, '--name-only')
    assert V.sha(V.RUNNER) == '93672e9d71aaa7fb9a14acc66eeb10e4ef2ca424515fc9eb12fd48cef4daab07'
    A.mkdir()
    baseline = read('target/phase-04c1b4c4r3c2r5g2r2-B-final.json')
    for prefix in ['phase-04c1b4c4r3c2r5g2r3-', P0, P1]:
        ledger = read('target/' + prefix + 'final-runtime-results.json')
        baseline.update(ledger['added_files'])
    tracked = V.git('ls-tree', '-rz', '--name-only', START).decode().strip('\0').split('\0')
    assert set(tracked) == set(baseline) and len(tracked) == 2469
    assert all(V.record(n) == row for n, row in baseline.items())
    put(A / 'windows-worktree-map.json', baseline)
    prior = read('target/' + P1 + 'old-inputs.json')
    prior.update({n: V.record(n) for n in baseline})
    prior['target/' + P1 + 'final-runtime-results.json'] = V.record('target/' + P1 + 'final-runtime-results.json')
    put(A / 'old-inputs.json', prior)
    gates = read('docs/implementation/' + P0 + 'validation-contract.json')['gates']
    selected = []
    for gate in gates:
        head = gate['frozen_head']
        V.git('cat-file', '-e', head + '^{commit}')
        assert subprocess.call(['git', 'merge-base', '--is-ancestor', head, START], cwd=R) == 0
        entries = V.git('ls-tree', '-rz', head).split(b'\0')
        assert all(not entry.startswith((b'160000 ', b'120000 ')) for entry in entries if entry)
        selected.append(dict(id=gate['gate_id'], head=head, debug=gate['debug'], case_count=gate['case_count'],
                             git_raw_manifest_sha256=gate['frozen_source_manifest']['complete_raw_manifest_sha256']))
    objects = {}
    for line in V.git('rev-list', '--objects', START).decode().splitlines():
        oid, _, path = line.partition(' ')
        objects[oid] = path
    forbidden = re.compile(r'(^|/)\.env($|\.)|(^|/)(id_rsa|id_ed25519|credentials)(\.|$)|\.(pem|pfx|p12|key)$', re.I)
    assert not [p for p in objects.values() if forbidden.search(p)], 'Excluded credential-shaped path in required Git ancestry; stop instead of rewriting history'
    types = subprocess.check_output(['git', 'cat-file', '--batch-check'], cwd=R, input=('\n'.join(objects) + '\n').encode()).decode().splitlines()
    private = re.compile(rb'-----BEGIN (?:RSA |OPENSSH |EC |DSA )?PRIVATE KEY-----|(?:ghp_|github_pat_)[A-Za-z0-9_]{30,}|sk-(?:proj-)?[A-Za-z0-9_-]{40,}')
    stream = subprocess.Popen(['git', 'cat-file', '--batch'], cwd=R, stdin=subprocess.PIPE, stdout=subprocess.PIPE)
    blob_hashes = {}
    try:
        for info in types:
            oid, kind, size = info.split()
            if kind != 'blob':
                continue
            stream.stdin.write((oid + '\n').encode()); stream.stdin.flush()
            header = stream.stdout.readline().decode().split()
            assert header == [oid, kind, size]
            remaining = int(size)
            h = hashlib.sha256()
            previous = b''
            while remaining:
                chunk = stream.stdout.read(min(1024 * 1024, remaining))
                assert chunk
                remaining -= len(chunk); h.update(chunk)
                assert not private.search(previous + chunk), ('Possible private credential; transfer denied', objects[oid], oid)
                previous = chunk[-256:]
            assert stream.stdout.read(1) == b'\n'
            blob_hashes[oid] = dict(raw_sha256=h.hexdigest(), byte_length=int(size))
        stream.stdin.close()
        assert stream.wait() == 0
    finally:
        if stream.poll() is None:
            stream.terminate(); stream.wait()
    put(A / 'transfer-screening.json', dict(reachable_objects=len(objects), scanned_blobs=len(blob_hashes),
        credential_paths_found=0, recognized_private_patterns_found=0, untracked_files_included=False,
        private_git_configuration_or_ssh_keys_included=False,
        limitation='Conservative filename and recognized credential pattern screening, not a proof of absence of every possible secret'))
    git_current = {}
    differences = []
    for entry in filter(None, V.git('ls-tree', '-rz', START).split(b'\0')):
        metadata, path = entry.split(b'\t')
        name, oid = path.decode(), metadata.split()[-1].decode()
        row = dict(git_blob=oid, **blob_hashes[oid], present=True)
        git_current[name] = row
        if row['raw_sha256'] != baseline[name]['raw_sha256']:
            file = R / name
            normalized_equal = None
            if file.stat().st_size <= 16 * 1024 * 1024:
                normalized_equal = hashlib.sha256(file.read_bytes().replace(b'\r\n', b'\n')).hexdigest() == row['raw_sha256']
            differences.append(dict(path=name, windows_raw_sha256=baseline[name]['raw_sha256'], git_blob_raw_sha256=row['raw_sha256'],
                                    crlf_to_lf_only_observed=normalized_equal, formal_equivalence_granted=False))
    put(A / 'git-current-map.json', git_current)
    put(A / 'windows-git-raw-differences.json', dict(differences=differences, source_modified=False, general_normalization_rule_created=False))
    run('bundle-create', ['git', '-c', 'pack.threads=1', '-c', 'pack.window=0', '-c', 'pack.windowMemory=16m',
        '-c', 'pack.deltaCacheSize=0', 'bundle', 'create', str(A / 'source.bundle'), 'HEAD'])
    run('bundle-verify-local', ['git', 'bundle', 'verify', str(A / 'source.bundle')])
    put(A / 'transfer-plan.json', dict(head=START, gates=selected, bundle_sha256=V.sha(A / 'source.bundle'),
        bundle_bytes=(A / 'source.bundle').stat().st_size, current_git_map_sha256=V.sha(A / 'git-current-map.json'),
        windows_worktree_map_sha256=V.sha(A / 'windows-worktree-map.json'), included_refs=['HEAD'],
        all_required_heads_are_ancestors=True, scope='Selected frozen commit ancestry, never untracked files or machine logs'))
    print(json.dumps(dict(source_screening='PASS', required_frozen_heads=9, tracked_paths=2469, bundle_bytes=(A / 'source.bundle').stat().st_size)), flush=True)


def transport():
    scp_up('upload-auditor', R / 'tools/linux-l1-remote.py', 'reports/linux-l1-remote.py')
    scp_up('upload-plan', A / 'transfer-plan.json', 'evidence/transfer-plan.json')
    scp_up('upload-bundle', A / 'source.bundle', 'evidence/source.bundle')
    run('remote-host', SSH + ['timeout 180s python3 -B "$HOME/' + REMOTE + '/reports/linux-l1-remote.py" host'])
    fetch('linux-environment-manifest'); fetch('receipt-host')
    run('remote-materialize', SSH + ['timeout 240s python3 -B "$HOME/' + REMOTE + '/reports/linux-l1-remote.py" materialize'])
    fetch('source-transfer-and-freeze'); fetch('receipt-materialize')
    run('remote-dependencies', SSH + ['timeout 180s python3 -B "$HOME/' + REMOTE + '/reports/linux-l1-remote.py" dependencies'])
    fetch('linux-dependency-audit'); fetch('receipt-dependencies')
    run('remote-static', SSH + ['timeout 180s python3 -B "$HOME/' + REMOTE + '/reports/linux-l1-remote.py" static'])
    fetch('linux-static-diagnostics'); fetch('receipt-static')
    run('download-current-map', SCP + ['fwq:' + REMOTE + '/evidence/source-map-current.json', str(A / 'linux-current-map.json')])
    linux = json.loads((A / 'linux-current-map.json').read_bytes())
    assert linux == json.loads((A / 'git-current-map.json').read_bytes())
    print(json.dumps(dict(transfer='PASS', snapshots=10, additional_linux_static_vectors=252, cargo_builds=0)), flush=True)


def unchanged():
    for line in V.git('diff', '--name-status', START).decode().splitlines():
        status, name = line.split('\t')
        assert status == 'A' and (name.startswith('docs/implementation/' + P) or name in ['tools/linux-l1.py', 'tools/linux-l1-remote.py']), line
    baseline = json.loads((A / 'old-inputs.json').read_bytes())
    assert all(V.record(n) == row for n, row in baseline.items())
    return len(baseline)


def complete_light_scope():
    # Preserve the initial whole-suite failure. No dependency installation or
    # original verifier change; standalone R3 has its own distinct denominator.
    fetch('failure-static')
    scp_up('upload-auditor-v2', R / 'tools/linux-l1-remote.py', 'reports/linux-l1-remote-v2.py')
    run('remote-r3-standalone', SSH + ['timeout 180s python3 -B "$HOME/' + REMOTE + '/reports/linux-l1-remote-v2.py" r3'])
    fetch('linux-r3-static-diagnostics'); fetch('receipt-r3')
    run('download-current-map', SCP + ['fwq:' + REMOTE + '/evidence/source-map-current.json', str(A / 'linux-current-map.json')])
    assert json.loads((A / 'linux-current-map.json').read_bytes()) == json.loads((A / 'git-current-map.json').read_bytes())
    print(json.dumps(dict(standalone_r3='PASS', vectors=110, combined_252='FAILED — MISSING DEPENDENCY', original_windows_histories='0/8')), flush=True)


def collect():
    unchanged()
    host = json.loads((A / 'linux-environment-manifest.json').read_bytes())
    transfer = json.loads((A / 'source-transfer-and-freeze.json').read_bytes())
    deps = json.loads((A / 'linux-dependency-audit.json').read_bytes())
    r3 = json.loads((A / 'linux-r3-static-diagnostics.json').read_bytes())
    failure = json.loads((A / 'failure-static.json').read_bytes())
    run('remote-python-alias', SSH + ['python --version && readlink -f "$(command -v python)"'])
    run('download-original-auditor', SCP + ['fwq:' + REMOTE + '/reports/linux-l1-remote.py', str(A / 'uploaded-remote-v1.py')])
    doc('ssh-and-host-audit', dict(ssh_alias='fwq', host_key_policy='StrictHostKeyChecking=yes', batch_mode=True,
        authenticated_connections_succeeded=True, initial_uname='Linux 7.0.0-31-generic x86_64',
        new_root='~/'+REMOTE, created_new=True, mode='0700', owner_matches_login=host['root_owned'],
        projects_symlink_rejected=True, no_preexisting_target_overwritten=True,
        physical_server_user_declared=True, corroboration=['systemd-detect-virt: none', 'DMI vendor Suma, product W40P', '32 logical CPUs /16 cores /2 sockets'],
        hardware_identity_independently_attested=False, private_ssh_keys_read=False, remote_pushes=0,
        private_raw_evidence='Dedicated remote evidence directory and ignored local artifacts; no hostname/IP/login/serial recorded in committed summary'))
    doc('linux-environment-manifest', dict(kernel=host['kernel'], distribution='Ubuntu 24.04.4 LTS', architecture='x86_64',
        logical_cpus=host['logical_cpus'], physical_cores=host['physical_cores'], sockets=host['sockets'],
        memory=host['memory'], disk=host['disk'],
        source='Actual read-only SSH inventory; availability is a sample, not sustained headroom qualification',
        cgroup_v2_present=True, session_cgroup_writable=False, user_service_delegated_subtree_writable_observed=True,
        controllers=['cpu','memory','pids'], inherited_observed_cpu_memory_caps='max in audited ancestors; not newly granted unlimited execution',
        resource_enforcement_qualification='Not Executed; no controller files changed'))
    tools = {name: dict(on_default_path=row.get('on_default_path'), version=row.get('version', {}).get('stdout'),
                       binary_sha256=row.get('installed_binary_sha256'), entry_sha256=row.get('entry_sha256')) for name, row in host['binaries'].items()}
    missing = [row for row in deps['packages'] if row['status'] != 'ARCHIVE CHECKSUM AND EXTRACTED PAYLOAD MATCH']
    doc('linux-toolchain-audit', dict(native_tools=tools, target='x86_64-unknown-linux-gnu', installed_python_modules=host['python_modules'],
        windows_reference=dict(rustc='1.92.0 cached self-report', llvm='21.1.3', host='x86_64-pc-windows-msvc', exact_binary_chain='Not qualified in old receipts'),
        linux_reference=dict(rustc='1.91.1', cargo='1.91.1', llvm='21.1.2', python='3.12.3'),
        toolchain_identity_equivalent=False, automatic_upgrade_or_install=False,
        dependencies=deps, missing_or_incomplete_packages=missing,
        python_blockers=['referencing missing', 'jsonschema4.10.3 does not supply the qualified current Registry API runtime'],
        requested_authorization='Approve an isolated project-local pinned Python runtime/wheels and the three locked crate cache gaps; no system/global-cache change authorized here'))
    differences = json.loads((A / 'windows-git-raw-differences.json').read_bytes())
    doc('source-transfer-and-freeze', dict(status=transfer['status'], head=START,
        bundle_sha256=transfer['bundle_sha256'], bundle_bytes=(A/'source.bundle').stat().st_size,
        screening=json.loads((A/'transfer-screening.json').read_bytes()), snapshots=transfer['snapshots'],
        local_git_and_linux_current_manifests_equal=True, windows_worktree_and_git_raw_differences=differences,
        raw_windows_tree_equal_to_linux_tree=False, source_or_expected_normalized_or_modified=False,
        other_projects_modified=False, local_history_modified=False, git_service_pushes=0))
    doc('resource-policy-and-approval', dict(status='PENDING USER APPROVAL — NO HEAVY CARGO ADMISSION',
        approved_linux_compilation_policy=None, approved_linux_test_policy=None, build_protector_installed=False,
        actual_light_audit_guard=dict(address_space_bytes=536870912, cpu_time_seconds=60, nice=19, serial=True,
            note='Per-process light audit/Git transport guard, not Windows Job equivalent or compiler admission'),
        original_windows_limits=dict(cpu_percent=5, process_bytes=805306368, job_bytes=1610612736, reserve_bytes=8589934592),
        proposal_not_applied=dict(P_build_bytes=2147483648, P_build_metric='Dedicated per-process leaf cgroup memory.max; fork containment and delegation require qualification',
            J_build_bytes=3221225472, J_build_metric='Dedicated parent cgroup memory.max, not Windows commit accounting',
            P_test_bytes=805306368, J_test_bytes=1610612736, system_reserve_bytes=8589934592,
            cpu_percent_of_online_host=5, observed_online_cpus=32, example_cpu_max='160000 100000',
            build_jobs=1, test_threads=1, top_level_jobs=1, swap_max=0,
            admission='Fresh sustained MemAvailable >= reserve + proposed J_build + independently approved margin; not inherited old14GiB threshold',
            disk='Approve per-gate/output/time caps and min free disk separately; current filesystem91% occupied',
            wall_time_limit=None, disk_output_limit=None),
        feasibility='Proposed ceilings only; actual compiler peak and policy feasibility unproved; no retry or temporary elevation',
        next_authorization='Explicit values/metrics/time/disk/swap and own delegated-cgroup control, plus fail-closed protector qualification; never cap shared user service or kill unrelated processes',
        runtime_D0_D4_or_root_authority_changed=False))
    census = []
    specs = read('docs/implementation/' + P0 + 'validation-contract.json')['gates']
    targets = [('current', START, R)] + [(g['gate_id'], g['frozen_head'], Path(g['commands']['cwd'])) for g in specs]
    windows = re.compile(r"WinDLL|os\.name\s*==\s*['\"]nt['\"]|std::os::windows|target_os\s*=\s*['\"]windows|target_env\s*=\s*['\"]msvc|\bwinapi\b|windows-sys|extern\s+\"system\"")
    linux = re.compile(r'std::os::linux|target_os\s*=\s*"linux"')
    process = re.compile(r'Command::new|std::process|std::fs::|std::env::|PathBuf|Path::|os\.environ|subprocess\.|platform\.')
    for identifier, head, folder in targets:
        paths = V.git('ls-tree', '-rz', '--name-only', head).decode().strip('\0').split('\0')
        for name in paths:
            if not (name.endswith(('.rs','.py')) or name.endswith('Cargo.toml')):
                continue
            content = (folder / name).read_text(encoding='utf8')
            markers = []
            category = 'Windows-specific' if windows.search(content) else 'Linux-specific' if linux.search(content) else 'Not Yet Classified' if process.search(content) else 'Portable / Cross-platform'
            for number, line in enumerate(content.splitlines(), 1):
                if windows.search(line) or linux.search(line) or (category == 'Not Yet Classified' and process.search(line)):
                    markers.append(dict(line=number, text=line[:200]))
            census.append(dict(snapshot=identifier, head=head, path=name, category=category,
                assurance='Static source-marker candidate only; no Rust test/target execution or dependency closure proof',
                source_markers=markers, exact_target=name if name.endswith('.py') else name.removesuffix('.rs'),
                formal_rule_impact='No semantic waiver or count change; execution-harness/path/process dependency requires profile proof',
                linux_portability_impact='Windows helper cannot be replaced by Wine/VM; use separately qualified native harness' if category == 'Windows-specific' else 'Requires complete unchanged target roster and appropriate native environment'))
    counts = {name: sum(row['category']==name for row in census) for name in ['Portable / Cross-platform','Linux-specific','Windows-specific','Not Yet Classified']}
    doc('portability-census', dict(method='Read-only static scan of current and nine frozen source trees; no filtering or test removal',
        classification_counts=counts, files=census,
        caveats=['String slice .windows() is not a Windows API', 'OS marker may be conditional; file classification is not a proof that an entire target is unbuildable',
                 'Rust requires python command in multiple tests; server python alias exists, but its3.12 runtime differs from Windows3.14 audit runtime',
                 'af-spec rule_area_format_is_enforced writes temp files: future TMPDIR must remain under authorized root',
                 'Case sensitivity, separators, actual FS/process behaviors and cfg-expanded doc/tests remain unproved']))
    doc('linux-native-gate-results', dict(additional_static_r3=r3, attempted_combined_static_suite=dict(status='Failed', actual_exit=1, failure=failure,
        not_a_production_semantic_contradiction=True, expected_not_changed=True),
        P0_105='Not Executed — referencing missing', P1A_37='Not Executed — dependent P0 runtime missing',
        cargo_fmt='Not Executed', cargo_clippy='Not Executed', cargo_test='Not Executed', spec_check='Not Executed',
        reason='No approved heavy Linux resource policy; locked dependency cache incomplete; no dependency/toolchain installations authorized',
        original_windows_historical_gates='0/8', original_windows_g3='Not Executed', positive_root_export=False))
    doc('linux-historical-diagnostics', dict(groups=[dict(id=g['gate_id'], head=g['frozen_head'], debug=g['debug'], required_case_ids=g['case_count'],
        linux_native_diagnostic='Pending — Not Executed', original_windows_gate='Not Executed', original_commands=g['commands']) for g in specs[:8]],
        linux_completed=0, original_windows_historical_gates='0/8'))
    doc('linux-g3-diagnostics', dict(head=specs[8]['frozen_head'], debug=2, required_cases=specs[8]['case_count'],
        linux_native_diagnostic='Pending — Not Executed', original_windows_g3='Not Executed', analysis9_compatibility_not_substituted=True))
    maps = [read('target/phase-04c1b4c4r3c2r5g2r2-B-'+n+'.json') for n in ['baseline','round1','round2','final']]
    assert all(m == maps[0] for m in maps) and len(maps[0])==2403
    assert all(V.record(n)==row for n,row in maps[0].items())
    doc('lane-b-credit-preservation', dict(status='Inherited Windows original-scope credit only', generators=41, actual_original_runs=82, original_paths=2403, byte_drift=0,
        windows_to_linux_generator_credit=False, linux_generator_runs=0, current_windows_source_paths=2469, existing_sources_unchanged=True,
        new_phase_materials='Only new L1 audit scripts/reports; not an expanded generator denominator', linux_root='~/'+REMOTE,
        future='Linux generator suitability/raw output maps need separate approval and receipts; no automatic double-round claim'))
    print(json.dumps(dict(report_data='COMPLETE', primary_contract_gap='C', execution_blockers='D', native_rust_builds=0, actual_r3_static_vectors=110)), flush=True)


def phase_files():
    files = {p.relative_to(R).as_posix(): V.record(p.relative_to(R)) for p in D.glob(P + '*') if p.is_file()}
    for name in ['tools/linux-l1.py','tools/linux-l1-remote.py']:
        files[name] = V.record(name)
    return files


def raw_correspondence():
    original=json.loads((A/'windows-git-raw-differences.json').read_bytes())
    value=json.loads(json.dumps(original))
    for row in value['differences']:
        if row['crlf_to_lf_only_observed'] is not None:
            continue
        h=hashlib.sha256(); carry=b''
        with (R/row['path']).open('rb') as stream:
            while chunk:=stream.read(1024*1024):
                chunk=carry+chunk
                carry=b'\r' if chunk.endswith(b'\r') else b''
                if carry: chunk=chunk[:-1]
                h.update(chunk.replace(b'\r\n',b'\n'))
        h.update(carry)
        row['crlf_to_lf_only_observed']=h.hexdigest()==row['git_blob_raw_sha256']
    value.update(prior_observation_sha256=V.sha(A/'windows-git-raw-differences.json'),
                 method='Streaming comparison only; zero file bytes rewritten; no semantic or formal byte-equivalence grant')
    put(A/'windows-git-raw-differences-streamed.json',value)
    source=read('docs/implementation/'+P+'source-transfer-and-freeze.json')
    source['windows_worktree_and_git_raw_differences']=value
    source['raw_difference_counts']=dict(total=len(value['differences']), json=sum(r['path'].endswith('.json') for r in value['differences']),
        expected_json=sum(r['path'].endswith('.expected.json') for r in value['differences']),
        all_crlf_to_lf_observed=all(r['crlf_to_lf_only_observed'] for r in value['differences']))
    doc('source-transfer-and-freeze',source)
    print(json.dumps(source['raw_difference_counts']),flush=True)


def target_labels():
    census=read('docs/implementation/'+P+'portability-census.json')
    gates=read('docs/implementation/'+P0+'validation-contract.json')['gates']
    folders={'current':R,**{g['gate_id']:Path(g['commands']['cwd']) for g in gates}}
    manifests={}
    for row in census['files']:
        name=row['path']; parts=name.split('/')
        if not name.endswith('.rs') or parts[0]!='crates': continue
        key=(row['snapshot'],parts[1])
        if key not in manifests:
            manifests[key]=tomllib.loads((folders[row['snapshot']]/'crates'/parts[1]/'Cargo.toml').read_text(encoding='utf8'))
        data=manifests[key]; package=data['package']; relative='/'.join(parts[2:])
        target=None
        for kind in ['test','bin','example','bench']:
            for declaration in data.get(kind,[]):
                if declaration.get('path',('tests/' if kind=='test' else 'src/bin/')+declaration['name']+'.rs')==relative:
                    target=dict(package=package['name'],kind=kind,name=declaration['name'],selected_by_manifest=True)
        if target is None and parts[2]=='tests' and len(parts)==4:
            target=dict(package=package['name'],kind='integration test',name=Path(name).stem,
                        selected_by_manifest=package.get('autotests',True))
        if target is None and relative.startswith('src/bin/'):
            target=dict(package=package['name'],kind='binary',name=Path(name).stem,selected_by_manifest=package.get('autobins',True))
        if target is None:
            target=dict(package=package['name'],kind='test support module' if relative.startswith('tests/') else 'library source',
                        name=None if relative.startswith('tests/') else data.get('lib',{}).get('name',package['name'].replace('-','_')),
                        selected_by_manifest='Requires module/cfg qualification' if relative.startswith('tests/') else True)
        row['exact_target']=target
        row['literal_test_function_declarations']=re.findall(r'#\[test\]\s*(?:#\[[^\n]*\]\s*)*(?:async\s+)?fn\s+(\w+)',(folders[row['snapshot']]/name).read_text(encoding='utf8'))
        row['actual_cfg_expanded_case_roster']='Not Executed / Not Qualified'
    doc('portability-census',census)
    print(json.dumps(dict(frozen_cargo_target_labels='COMPLETE',rust_execution=0)),flush=True)


def review():
    count = unchanged()
    plan = json.loads((A/'transfer-plan.json').read_bytes())
    transferred = json.loads((A/'source-transfer-and-freeze.json').read_bytes())
    assert transferred['bundle_sha256'] == plan['bundle_sha256'] == V.sha(A/'source.bundle')
    assert len(transferred['snapshots']) == 10
    for gate in plan['gates']:
        assert transferred['snapshots'][gate['id']]['head'] == gate['head']
        assert transferred['snapshots'][gate['id']]['manifest_sha256'] == gate['git_raw_manifest_sha256']
    assert [g['debug'] for g in plan['gates']] == [2]*7+[0,2]
    native = read('docs/implementation/'+P+'linux-native-gate-results.json')
    assert native['additional_static_r3']['passed'] == 110
    assert native['attempted_combined_static_suite']['actual_exit'] == 1
    assert native['cargo_test'] == 'Not Executed' and native['original_windows_historical_gates']=='0/8'
    policy = read('docs/implementation/'+P+'resource-policy-and-approval.json')
    assert policy['approved_linux_compilation_policy'] is None and not policy['build_protector_installed']
    assert not policy['runtime_D0_D4_or_root_authority_changed']
    body=(D/(P+'formal-acceptance-amendment-draft.md')).read_text(encoding='utf8')
    assert 'NOT REGISTERED' in body and '0/8' in body
    equivalent=(D/(P+'cross-platform-equivalence-analysis.md')).read_text(encoding='utf8')
    assert 'AF-ACE1-0002' in equivalent and 'AF-DIGEST-0001' in equivalent
    result=dict(status='PASS — INDEPENDENT STATIC READBACK', method='Same agent, independent readback assertions; not external executor/hardware attestation',
        source_inputs_exact=count, checks=['Trusted SSH and newly created dedicated root; no unrelated project writes',
            'Git/source identity exact for current plus nine frozen snapshots; old raw differences separately retained',
            '110 actually executed standalone static vectors; first exit1 and unexecuted105/37/cargo/history/G3 not inflated',
            'No Linux resource approval or enforcement qualification; draft only, Windows limits/Expected/Oracle untouched',
            'Existing semantic byte rules do not supersede Windows acceptance ABI; proposed profile is unregistered; deficit remains0/8'],
        decision='C — cross-platform acceptance contract gap; D execution/dependency/resource blockers also open',
        inputs=phase_files())
    put(A/'independent-review.json',result)
    print(json.dumps(dict(review='PASS', outcome='C', resource_execution_blockers='D', original_windows_histories='0/8')),flush=True)


def publish(final_only=False):
    files=[A/'final-runtime-results.json'] if final_only else sorted(p for p in D.glob(P+'*') if p.suffix in ['.json','.md'])
    destinations=[P+'final-runtime-results.json'] if final_only else [p.name for p in files]
    check=' && '.join('test ! -e "$HOME/'+REMOTE+'/reports/'+n+'" && test ! -L "$HOME/'+REMOTE+'/reports/'+n+'"' for n in destinations)
    tag='publish-final' if final_only else 'publish-reports'
    run(tag+'-absent',SSH+[check])
    destination='fwq:'+REMOTE+'/reports/'+(destinations[0] if final_only else '')
    run(tag,SCP+[*(str(p) for p in files),destination])
    command="python3 - <<'PY'\nimport hashlib,json,pathlib\nr=pathlib.Path.home()/'"+REMOTE+"/reports'\nnames="+repr(destinations)+"\nprint(json.dumps({n:hashlib.sha256((r/n).read_bytes()).hexdigest() for n in names},sort_keys=True))\nPY"
    run(tag+'-hashes',SSH+[command])
    hashes=json.loads((A/(tag+'-hashes.log')).read_bytes())
    assert hashes==dict(zip(destinations,[V.sha(p) for p in files]))
    put(A/(tag+'-receipt.json'),dict(actual_files=hashes,source_transfer_permission='Explicit L1 user authorization',copies_confined_to_dedicated_root=True))


def seal():
    assert not V.git('status','--porcelain')
    count=unchanged()
    reviewed=json.loads((A/'independent-review.json').read_bytes())
    assert all(V.record(n)==row for n,row in reviewed['inputs'].items())
    receipts=[]
    for file in sorted((R/'target').glob('phase-04c1b4c4r3c2r4g3-safe-linux-l1-*.json')):
        row=json.loads(file.read_bytes())
        assert row['limits_verified'] and row['cancelled'] is None
        if row['exit']:
            assert row['name'] in ['linux-l1-transfer-static','linux-l1-target-labels'] and row['exit']==1
        receipts.append(row)
    def local_raw(name): return json.loads((A/(name+'.json')).read_bytes())
    result=dict(outcome='C — Cross-platform Acceptance Contract Gap', secondary_status='D — Linux dependency/resource/execution prerequisites incomplete',
        review_topic='R5G2-LINUX-VERIFICATION-EQUIVALENCE', registered_rule=False,
        source_head=START, local_final_head=V.git('rev-parse','HEAD').decode().strip(), remote_source_head=START,
        commit_list=V.git('log','--format=%H %s',START+'..HEAD').decode().splitlines(),
        ssh_trusted=True, new_remote_directory='~/'+REMOTE, snapshots=10, source_inputs_exact=count,
        linux_static_r3=dict(actual=110,passed=110), linux_combined252=dict(status='Failed',actual_exit=1,reason='Missing referencing; original failure preserved'),
        linux_P0_105='Not Executed',linux_P1A_37='Not Executed',linux_cargo_builds=0,linux_rust_tests=0,
        original_windows_histories='0/8',original_windows_g3='Not Executed',linux_historical_diagnostics='Not Executed',linux_g3_diagnostics='Not Executed',
        heavy_linux_policy_approved=False, resource_amendments_applied=False, dependency_installs=0,
        positive_root_export=False,r5g2_registration='INCOMPLETE',formal_semantics_or_expected_modified=False,
        vm_or_wine_created=False,unrelated_projects_modified=False,local_or_remote_pushes=0,
        remote_receipts={name:local_raw('receipt-'+name) for name in ['host','materialize','dependencies','r3']},
        failed_remote_attempt=local_raw('failure-static'),local_protection_receipts=receipts,
        preserved_local_argument_failure='linux-l1-target-labels: underscore mode rejected before work; correct hyphenated mode subsequently passed',
        raw_source_windows_vs_git_differences=local_raw('windows-git-raw-differences'),
        extended_streaming_raw_correspondence=local_raw('windows-git-raw-differences-streamed'),
        added_files=phase_files(),independent_review=reviewed,clean_worktree=True,
        evidence_mirror='Committed reports and this external ledger copied to dedicated reports; separate hash receipts avoid self-reference')
    put(A/'pending-final-seal.json',result)
    print(json.dumps(dict(outcome=result['outcome'],head=result['local_final_head'],old_histories='0/8')),flush=True)


def closeout():
    assert not V.LOCK.exists() and not V.git('status','--porcelain')
    result=json.loads((A/'pending-final-seal.json').read_bytes())
    receipt=read('target/phase-04c1b4c4r3c2r4g3-safe-linux-l1-final-seal.json')
    assert receipt['exit']==0 and receipt['cancelled'] is None
    assert V.git('rev-parse','HEAD').decode().strip()==result['local_final_head']
    query="@(Get-CimInstance Win32_Process -Filter \"Name = 'python.exe' OR Name = 'ssh.exe' OR Name = 'scp.exe'\" | Where-Object { $_.ProcessId -ne "+str(os.getpid())+" -and $_.CommandLine -match 'tools[\\/]linux-l1|linux-l1-remote' } | Select-Object ProcessId,Name) | ConvertTo-Json -Compress"
    raw=subprocess.check_output(['powershell','-NoProfile','-Command',query],cwd=R)
    assert not raw.strip() or json.loads(raw)==[]
    result.update(final_local_protection_receipt=receipt,pending_local_background_jobs=[],protected_lock_present=False,
                  post_job_closeout='Tiny metadata-only final check outside ended protected Job; remote check/hash mirror receipts separate')
    put(A/'final-runtime-results.json',result)
    # Required external ledger has its conventional stage-prefixed path too.
    put(R/'target'/(P+'final-runtime-results.json'),result)
    print(json.dumps(dict(outcome=result['outcome'],head=result['local_final_head'],clean=True,pushes=0)),flush=True)


if __name__ == '__main__':
    if sys.argv[1:] == ['prepare']:
        prepare()
    elif sys.argv[1:] == ['transport']:
        transport()
    elif sys.argv[1:] == ['complete-light-scope']:
        complete_light_scope()
    elif sys.argv[1:] == ['collect']:
        collect()
    elif sys.argv[1:] == ['review']:
        review()
    elif sys.argv[1:] == ['raw-correspondence']:
        raw_correspondence()
    elif sys.argv[1:] == ['target-labels']:
        target_labels()
    elif sys.argv[1:] == ['publish']:
        publish()
    elif sys.argv[1:] == ['publish-final']:
        publish(True)
    elif sys.argv[1:] == ['seal']:
        seal()
    elif sys.argv[1:] == ['closeout']:
        closeout()
    else:
        raise SystemExit('Unknown local L1 audit mode; no build mode exists')
