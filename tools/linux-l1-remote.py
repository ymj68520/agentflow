"""Linux L1: bounded source/environment/static audits; NO Cargo build or install.

All writes are confined to the user-authorized new validation root. Existing
toolchain and dependency caches are read-only. No cgroup configuration is changed.
"""
import copy
import hashlib
import importlib.metadata
import importlib.util
import json
import os
from pathlib import Path
import resource
import shutil
import subprocess
import sys
import tarfile
import time
import tomllib

ROOT = Path.home() / 'projects/agent-r5g2-linux-validation'
P0 = 'phase-04c1b4c4r3c2r5g2r4p0-'


def sha(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as stream:
        while chunk := stream.read(1024 * 1024):
            h.update(chunk)
    return h.hexdigest()


def canonical(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def save(name, value):
    path = ROOT / 'evidence' / (name + '.json')
    if path.exists():
        raise RuntimeError('Never overwrite evidence: ' + name)
    path.write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode())
    return value


def command(argv, cwd=None, timeout=60):
    # Fixed callers below permit Git operations and tool version queries only.
    started = time.time()
    process = subprocess.run(argv, cwd=cwd, capture_output=True, timeout=timeout,
        env=dict(os.environ, LC_ALL='C', RUSTUP_AUTO_INSTALL='0', RUSTUP_NO_UPDATE_CHECK='1', PYTHONDONTWRITEBYTECODE='1',
                 GIT_CONFIG_NOSYSTEM='1', GIT_CONFIG_GLOBAL='/dev/null', GIT_ATTR_NOSYSTEM='1', GIT_TERMINAL_PROMPT='0'))
    row = dict(argv=argv, exit=process.returncode, seconds=time.time() - started,
               stdout=process.stdout.decode('utf8', 'replace'), stderr=process.stderr.decode('utf8', 'replace'))
    if row['exit']:
        raise RuntimeError(row)
    return row


def host():
    mem = {k: int(v.strip().split()[0]) * 1024 for k, v in (line.split(':', 1) for line in Path('/proc/meminfo').read_text().splitlines())}
    cgroups = []
    membership = next(line.split(':', 2)[2] for line in Path('/proc/self/cgroup').read_text().splitlines() if line.startswith('0::'))
    cg = Path('/sys/fs/cgroup') / membership.lstrip('/')
    while cg.is_relative_to(Path('/sys/fs/cgroup')):
        fields = {}
        for name in ['memory.max', 'memory.high', 'memory.current', 'memory.swap.max', 'cpu.max', 'pids.max', 'cgroup.controllers', 'cgroup.subtree_control']:
            if (cg / name).is_file():
                fields[name] = (cg / name).read_text().strip()
        cgroups.append(dict(path=str(cg), fields=fields, writable=os.access(cg, os.W_OK)))
        if cg == Path('/sys/fs/cgroup'):
            break
        cg = cg.parent
    binaries = {}
    for name in ['git', 'python3', 'rustc', 'cargo']:
        path = shutil.which(name)
        if path is None and (Path.home() / '.cargo/bin' / name).is_file():
            path = str(Path.home() / '.cargo/bin' / name)
        binaries[name] = dict(path=path, on_default_path=shutil.which(name) is not None)
        if path:
            binaries[name]['version'] = command([path, '-vV'] if name == 'rustc' else [path, '--version'])
            binaries[name]['entry_sha256'] = sha(path)
    rustup = Path.home() / '.cargo/bin/rustup'
    if rustup.is_file():
        for name in ['rustc', 'cargo', 'rustdoc']:
            row = command([str(rustup), 'which', name])
            real = Path(row['stdout'].strip())
            binaries[name] = dict(binaries.get(name, {}), installed_binary=str(real), installed_binary_sha256=sha(real))
    cpu = Path('/proc/cpuinfo').read_text()
    sockets, cores = set(), set()
    for block in cpu.split('\n\n'):
        data = dict(line.split(':', 1) for line in block.splitlines() if ':' in line)
        data = {k.strip(): v.strip() for k, v in data.items()}
        if 'physical id' in data:
            sockets.add(data['physical id'])
            cores.add((data['physical id'], data.get('core id')))
    disk = shutil.disk_usage(Path.home())
    modules = {}
    for name in ['jsonschema', 'cryptography', 'referencing']:
        try:
            modules[name] = importlib.metadata.version(name)
        except importlib.metadata.PackageNotFoundError:
            modules[name] = None
    return dict(os_release=Path('/etc/os-release').read_text(), kernel=command(['uname', '-srm'])['stdout'].strip(),
        logical_cpus=os.cpu_count(), physical_cores=len(cores), sockets=len(sockets),
        memory={n: mem[n] for n in ['MemTotal', 'MemAvailable', 'SwapTotal', 'SwapFree', 'CommitLimit', 'Committed_AS']},
        disk=dict(total=disk.total, used=disk.used, free=disk.free), cgroup_v2=cgroups,
        binaries=binaries, python_modules=modules, current_user_uid=os.getuid(), root_owned=os.stat(ROOT).st_uid == os.getuid(),
        root_mode=oct(ROOT.stat().st_mode & 0o777), physical_identity='User-declared physical server; OS indicators only, not independent hardware attestation',
        compiler_budget_approved=False, heavy_builds=0)


def git(args, cwd):
    return command(['git', '-c', 'core.autocrlf=false', '-c', 'core.eol=lf', '-c', 'core.hooksPath=/dev/null',
                    '-c', 'core.attributesFile=/dev/null', '-c', 'core.fsmonitor=false',
                    '-c', 'core.packedGitWindowSize=8m', '-c', 'core.packedGitLimit=32m', '-c', 'pack.threads=1', *args], cwd)


def materialize():
    plan = json.loads((ROOT / 'evidence/transfer-plan.json').read_bytes())
    bundle = ROOT / 'evidence/source.bundle'
    assert sha(bundle) == plan['bundle_sha256']
    workspace = ROOT / 'workspace'
    assert not list(workspace.iterdir())
    rows = [git(['init', '--template='], workspace), git(['bundle', 'verify', str(bundle)], workspace),
            git(['fetch', '--no-tags', str(bundle), 'HEAD'], workspace),
            git(['checkout', '--detach', plan['head']], workspace)]
    assert git(['rev-parse', 'HEAD'], workspace)['stdout'].strip() == plan['head']
    snapshots = [dict(id='current', head=plan['head'], path=str(workspace))]
    for item in plan['gates']:
        destination = ROOT / 'frozen' / item['id']
        assert not destination.exists()
        git(['cat-file', '-e', item['head'] + '^{commit}'], workspace)
        rows.append(git(['worktree', 'add', '--detach', '--lock', str(destination), item['head']], workspace))
        snapshots.append(dict(id=item['id'], head=item['head'], path=str(destination)))
    manifest = {}
    for snapshot in snapshots:
        folder = Path(snapshot['path'])
        listing = subprocess.check_output(['git', 'ls-tree', '-rz', snapshot['head']], cwd=folder)
        files = {}
        for entry in filter(None, listing.split(b'\0')):
            metadata, name = entry.split(b'\t')
            mode, kind, expected = metadata.split()
            assert mode != b'160000' and kind == b'blob'
            file = folder / name.decode()
            assert file.is_file() and not file.is_symlink()
            blob = hashlib.sha1(b'blob ' + str(file.stat().st_size).encode() + b'\0')
            with file.open('rb') as stream:
                while chunk := stream.read(1024 * 1024):
                    blob.update(chunk)
            assert blob.hexdigest().encode() == expected, str(file)
            files[name.decode()] = dict(git_blob=blob.hexdigest(), raw_sha256=sha(file), byte_length=file.stat().st_size, present=True)
        assert not git(['status', '--porcelain'], folder)['stdout']
        path = ROOT / 'evidence' / ('source-map-' + snapshot['id'] + '.json')
        assert not path.exists()
        path.write_bytes((json.dumps(files, indent=2) + '\n').encode())
        manifest[snapshot['id']] = dict(head=snapshot['head'], paths=len(files), manifest_sha256=canonical(files),
                                         source_map_sha256=sha(path), source_map=path.name, git_blob_and_checkout_raw_exact=True)
        if snapshot['id'] != 'current':
            required = next(g for g in plan['gates'] if g['id'] == snapshot['id'])
            assert manifest[snapshot['id']]['manifest_sha256'] == required['git_raw_manifest_sha256']
    return save('source-transfer-and-freeze', dict(status='PASS — GIT OBJECT AND LINUX CHECKOUT IDENTITY',
        bundle_sha256=sha(bundle), snapshots=manifest, git_commands=rows, original_windows_gate_credit=0,
        windows_worktree_raw_equality='Not asserted; separate original Windows raw manifest required',
        other_projects_modified=False, pushes=0))


def dependencies():
    plan = json.loads((ROOT / 'evidence/transfer-plan.json').read_bytes())
    all_packages = {}
    lockfiles = {}
    for item in [dict(id='current'), *plan['gates']]:
        folder = ROOT / 'workspace' if item['id'] == 'current' else ROOT / 'frozen' / item['id']
        lock = folder / 'Cargo.lock'
        parsed = tomllib.loads(lock.read_text())
        keys = []
        for package in parsed['package']:
            if 'source' in package:
                key = package['name'] + '-' + package['version']
                keys.append(key)
                assert key not in all_packages or all_packages[key] == package
                all_packages[key] = package
        lockfiles[item['id']] = dict(raw_sha256=sha(lock), registry_packages=keys)
    cache = Path.home() / '.cargo/registry/cache'
    sources = Path.home() / '.cargo/registry/src'
    rows = []
    for key, package in sorted(all_packages.items()):
        archives = list(cache.glob('*/' + key + '.crate'))
        roots = list(sources.glob('*/' + key))
        row = dict(package=key, source=package['source'], expected_checksum=package.get('checksum'),
                   archive_found=bool(archives), extracted_found=bool(roots), status='MISSING')
        if archives and roots and sha(archives[0]) == package.get('checksum'):
            archive, source = archives[0], roots[0]
            verified = 0
            expected_files = set()
            with tarfile.open(archive, mode='r|gz') as stream:
                for member in stream:
                    if member.isdir():
                        continue
                    assert member.isfile() and not member.issym() and not member.islnk()
                    relative = Path(member.name).relative_to(key)
                    assert '..' not in relative.parts and not relative.is_absolute()
                    actual = source / relative
                    expected_files.add(relative.as_posix())
                    h = hashlib.sha256()
                    payload = stream.extractfile(member)
                    while chunk := payload.read(1024 * 1024):
                        h.update(chunk)
                    assert actual.is_file() and not actual.is_symlink() and sha(actual) == h.hexdigest(), str(relative)
                    verified += 1
            actual_files = {p.relative_to(source).as_posix() for p in source.rglob('*') if p.is_file()}
            unexpected = sorted(actual_files - expected_files - {'.cargo-ok', '.cargo-checksum.json'})
            row.update(status='ARCHIVE CHECKSUM AND EXTRACTED PAYLOAD MATCH' if not unexpected else 'UNEXPECTED EXTRACTED FILES',
                       verified_payload_files=verified, unexpected_files=unexpected)
        elif archives:
            row['archive_checksum_matches'] = sha(archives[0]) == package.get('checksum')
        rows.append(row)
    return save('linux-dependency-audit', dict(lockfiles=lockfiles, packages=rows,
        complete=all(r['status'] == 'ARCHIVE CHECKSUM AND EXTRACTED PAYLOAD MATCH' for r in rows),
        downloads=0, installs=0, cache_modified=False,
        limitation='Archive payload identity checked; no Cargo resolution/build, network access, or platform-specific dependency selection performed'))


def load(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


def static_diagnostics():
    d = ROOT / 'workspace/docs/implementation'
    def read(name):
        return json.loads((d / name).read_bytes())
    r3 = load(ROOT / 'workspace/tools/evidence-r5g2r3.py', 'linux_frozen_r3')
    expected_r3 = read('phase-04c1b4c4r3c2r5g2r3-design-vector-results.json')
    assert r3.vectors() == expected_r3 and expected_r3['total'] == 110
    validator = load(d / (P0 + 'offline-manifest-validator.py'), 'linux_frozen_p0')
    oracle = load(d / (P0 + 'independent-static-oracle.py'), 'linux_frozen_p0_independent')
    contract, schema = read(P0 + 'validation-contract.json'), read(P0 + 'evidence-manifest-schema.json')
    vectors = read(P0 + 'static-test-vectors.json')['vectors']
    for field in schema['required']:
        vectors.append(dict(id='omit-root/' + field, edits=[], delete=[field], valid=False, error='SCHEMA'))
    for field in read(P0 + 'environment-candidate-schema.json')['required']:
        vectors.append(dict(id='omit-candidate/' + field, edits=[], delete=['candidate.' + field], valid=False, error='SCHEMA'))
    def mutate(value, case):
        def at(path):
            node = value
            parts = path.split('.')
            for key in parts[:-1]:
                node = node[int(key)] if isinstance(node, list) else node[key]
            return node, int(parts[-1]) if isinstance(node, list) else parts[-1]
        for path in case.get('delete', []):
            node, key = at(path); del node[key]
        for source, dest in case.get('copy', []):
            node, key = at(source); target, field = at(dest); target[field] = copy.deepcopy(node[key])
        for path, field in case['edits']:
            node, key = at(path); node[key] = field
        return value
    p0_rows = []
    for case in vectors:
        base = read(P0 + ('preparation-manifest.json' if case.get('base') == 'Preparation' else 'synthetic-manifest.json'))
        value = mutate(base, case)
        a = validator.validate(value, contract, schema)
        b = oracle.evaluate(value, contract, schema)
        assert (a['static_status'] == 'Static PASS') == b['valid'] == case['valid'], case['id']
        if 'error' in case:
            assert case['error'] in a['errors']
        assert not a['positive_authority'] and a['environment_status'] == 'NOT QUALIFIED'
        p0_rows.append(dict(id=case['id'], expected=case['valid'], actual=a['static_status'] == 'Static PASS', independent=b['valid'], errors=a['errors']))
    assert len(p0_rows) == 105
    p1 = 'phase-04c1b4c4r3c2r5g2r4p1a-'
    primary = load(ROOT / 'workspace/tools/review-r5g2r4p1a.py', 'linux_frozen_p1a')
    independent = load(d / (p1 + 'independent-static-oracle.py'), 'linux_frozen_p1a_independent')
    req = read(p1 + 'requirements-reference.json')
    p1_rows = []
    for case in read(p1 + 'authorization-static-vectors.json')['vectors']:
        actual = case.get('base') == 'actual'
        base = read(p1 + 'application-input.json') if actual else primary.synthetic_application(req)
        value = mutate(base, case)
        context = primary.trust_context(not actual)
        a = primary.evaluate(value, req, context)
        b = independent.evaluate(value, req, context, read(P0 + 'environment-candidate-schema.json'))
        assert a['package_complete'] == b['package_complete'] == case['expected_complete'], case['id']
        assert not a['P1_authorized'] and not a['P2_authorized'] and not a['positive_root_export']
        p1_rows.append(dict(id=case['id'], expected=case['expected_complete'], actual=a['package_complete'], independent=b['package_complete']))
    assert len(p1_rows) == 37
    return save('linux-static-diagnostics', dict(status='PASS — ADDITIONAL LINUX STATIC DATA DIAGNOSTICS',
        source_head=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT / 'workspace').decode().strip(),
        r3=110, p0=105, p1a=37, total_static_vectors=252, p0_cases=p0_rows, p1a_cases=p1_rows,
        native_rust_tests=0, original_windows_historical_gates='0/8', original_windows_g3='Not Executed',
        linux_historical_diagnostics='Not Executed', linux_g3_diagnostics='Not Executed', positive_root_export=False,
        independent_executor_attestation=False, source_or_expected_modified=False))


def r3_only():
    source = ROOT / 'workspace'
    module = load(source / 'tools/evidence-r5g2r3.py', 'linux_r3_standalone')
    expected = json.loads((source / 'docs/implementation/phase-04c1b4c4r3c2r5g2r3-design-vector-results.json').read_bytes())
    actual = module.vectors()
    assert actual == expected and actual['total'] == 110
    statuses = {}
    for folder in [source, *(ROOT / 'frozen').iterdir()]:
        if folder.is_dir():
            status = git(['status', '--porcelain'], folder)['stdout']
            assert not status
            statuses[folder.name] = 'CLEAN'
    return save('linux-r3-static-diagnostics', dict(status='PASS — R3 STANDARD-LIBRARY STATIC DIAGNOSTICS ONLY',
        case_ids=[row['id'] for row in actual['vectors']], total=110, passed=110,
        source_head=git(['rev-parse', 'HEAD'], source)['stdout'].strip(),
        expected_raw_sha256=sha(source / 'docs/implementation/phase-04c1b4c4r3c2r5g2r3-design-vector-results.json'),
        frozen_source_status=statuses, complete_combined_252_suite=False, prior_failed_static_receipt_preserved=True,
        P0_105='Not Executed — missing referencing / unqualified JSON Schema runtime',
        P1A_37='Not Executed — depends on missing P0 runtime',
        original_windows_historical_gates='0/8', original_windows_g3='Not Executed', positive_root_export=False))


if __name__ == '__main__':
    assert ROOT.is_dir() and not ROOT.is_symlink() and ROOT.stat().st_uid == os.getuid()
    assert not ROOT.parent.is_symlink() and ROOT.resolve() == Path.home().resolve() / 'projects/agent-r5g2-linux-validation'
    os.umask(0o077)
    os.nice(19)
    # Light audit/transport guard only; not a Linux compiler/test policy and not
    # a claim of Windows Job-equivalent CPU or aggregate memory enforcement.
    resource.setrlimit(resource.RLIMIT_AS, (512 * 1024 ** 2, 512 * 1024 ** 2))
    resource.setrlimit(resource.RLIMIT_CPU, (60, 60))
    started = time.time()
    mode = sys.argv[1]
    try:
        result = save('linux-environment-manifest', host()) if mode == 'host' else materialize() if mode == 'materialize' else dependencies() if mode == 'dependencies' else static_diagnostics() if mode == 'static' else r3_only() if mode == 'r3' else None
        assert result is not None
        save('receipt-' + mode, dict(mode=mode, exit=0, started_unix=started, seconds=time.time() - started,
             max_rss_bytes=resource.getrusage(resource.RUSAGE_SELF).ru_maxrss * 1024,
             cpu_seconds=resource.getrusage(resource.RUSAGE_SELF).ru_utime + resource.getrusage(resource.RUSAGE_SELF).ru_stime,
             light_guard=dict(address_space_bytes=512 * 1024 ** 2, cpu_time_seconds=60, nice=19, compiler_policy_approved=False)))
        print(json.dumps(dict(mode=mode, exit=0, evidence=str(ROOT / 'evidence'))))
    except BaseException as exc:
        save('failure-' + mode, dict(mode=mode, error=repr(exc), exit=1, started_unix=started, seconds=time.time() - started))
        raise
