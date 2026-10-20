"""L2 static replay. Frozen implementations/oracles/Expected are read-only.

No build, qualification, connection authorization or RootExport capability is
issued. Each group is a separate process; immutable receipts use exclusive writes.
"""
import copy
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import resource
import subprocess
import sys
import time

W = Path(__file__).resolve().parents[1]
D = W / 'docs/implementation'
E = W.parent / 'evidence/phase-linux-primary-development-migration'
P0 = 'phase-04c1b4c4r3c2r5g2r4p0-'
P1 = 'phase-04c1b4c4r3c2r5g2r4p1a-'
R3 = 'phase-04c1b4c4r3c2r5g2r3-'
COUNTS = {'r3': 110, 'p0': 105, 'p1a': 37}


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def read(name):
    return json.loads((D / name).read_bytes())


def save(path, value):
    with path.open('x', encoding='utf8') as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write('\n')


def load(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


def mutate(value, case):
    def at(path):
        node = value
        parts = path.split('.')
        for key in parts[:-1]:
            node = node[int(key)] if isinstance(node, list) else node[key]
        return node, int(parts[-1]) if isinstance(node, list) else parts[-1]
    for path in case.get('delete', []):
        node, key = at(path)
        del node[key]
    for source, dest in case.get('copy', []):
        node, key = at(source)
        target, field = at(dest)
        target[field] = copy.deepcopy(node[key])
    for path, value_field in case['edits']:
        node, key = at(path)
        node[key] = value_field
    return value


def group(name):
    rows = []
    if name == 'r3':
        impl = load(W / 'tools/evidence-r5g2r3.py', 'l2_r3')
        expected = read(R3 + 'design-vector-results.json')
        actual = impl.vectors()
        assert actual['total'] == expected['total'] == 110
        assert len(actual['vectors']) == 110
        for a, b in zip(actual['vectors'], expected['vectors']):
            rows.append(dict(id=b['id'], expected=b, actual=a, passed=a == b))
        assert {k:v for k,v in actual.items() if k != 'vectors'} == {k:v for k,v in expected.items() if k != 'vectors'}
    elif name == 'p0':
        impl = load(D / (P0 + 'offline-manifest-validator.py'), 'l2_p0')
        oracle = load(D / (P0 + 'independent-static-oracle.py'), 'l2_p0_oracle')
        contract, schema = read(P0 + 'validation-contract.json'), read(P0 + 'evidence-manifest-schema.json')
        cases = read(P0 + 'static-test-vectors.json')['vectors']
        for field in schema['required']:
            cases.append(dict(id='omit-root/' + field, edits=[], delete=[field], valid=False, error='SCHEMA'))
        for field in read(P0 + 'environment-candidate-schema.json')['required']:
            cases.append(dict(id='omit-candidate/' + field, edits=[], delete=['candidate.' + field], valid=False, error='SCHEMA'))
        assert len(cases) == 105
        for case in cases:
            value = mutate(read(P0 + ('preparation-manifest.json' if case.get('base') == 'Preparation' else 'synthetic-manifest.json')), case)
            a, b = impl.validate(value, contract, schema), oracle.evaluate(value, contract, schema)
            passed = (a['static_status'] == 'Static PASS') == b['valid'] == case['valid']
            passed = passed and ('error' not in case or case['error'] in a['errors'])
            passed = passed and not a['positive_authority'] and a['environment_status'] == 'NOT QUALIFIED'
            rows.append(dict(id=case['id'], expected=case, input_sha256=digest(value), actual=a, independent=b, passed=passed))
    elif name == 'p1a':
        impl = load(W / 'tools/review-r5g2r4p1a.py', 'l2_p1a')
        oracle = load(D / (P1 + 'independent-static-oracle.py'), 'l2_p1a_oracle')
        req = read(P1 + 'requirements-reference.json')
        cases = read(P1 + 'authorization-static-vectors.json')['vectors']
        assert len(cases) == 37
        for case in cases:
            actual = case.get('base') == 'actual'
            base = read(P1 + 'application-input.json') if actual else impl.synthetic_application(req)
            value = mutate(base, case)
            context = impl.trust_context(not actual)
            a = impl.evaluate(value, req, context)
            b = oracle.evaluate(value, req, context, read(P0 + 'environment-candidate-schema.json'))
            passed = a['package_complete'] == b['package_complete'] == case['expected_complete']
            passed = passed and not a['P1_authorized'] and not a['P2_authorized'] and not a['positive_root_export']
            rows.append(dict(id=case['id'], expected=case, input_sha256=digest(value), actual=a, independent=b, passed=passed))
    assert len(rows) == COUNTS[name] and len({r['id'] for r in rows}) == COUNTS[name]
    return rows


def main():
    os.nice(19)
    resource.setrlimit(resource.RLIMIT_AS, (512 * 1024**2,) * 2)
    resource.setrlimit(resource.RLIMIT_CPU, (60,) * 2)
    if len(sys.argv) == 3 and sys.argv[1] == '--group':
        name = sys.argv[2]
        rows = group(name)
        print(json.dumps(rows, ensure_ascii=False, sort_keys=True))
        return 0 if all(r['passed'] for r in rows) else 1
    assert len(sys.argv) == 1
    assert Path(sys.prefix).resolve() == W.parent / 'venvs/linux-primary-v1'
    assert not (E / 'static-252-results.json').exists()
    inputs = {p.relative_to(W).as_posix():sha(p) for p in sorted(D.iterdir()) if p.is_file() and p.name.startswith((R3, P0, P1))}
    for name in ['evidence-r5g2r3.py', 'review-r5g2r4p1a.py', 'verify-r5g2r1.py', 'linux-primary-static.py']:
        inputs['tools/' + name] = sha(W / 'tools' / name)
    records = []
    for name, count in COUNTS.items():
        started = time.time()
        p = subprocess.run([sys.executable, '-B', str(Path(__file__)), '--group', name], cwd=W,
            env=dict(os.environ, PYTHONDONTWRITEBYTECODE='1', PYTHONNOUSERSITE='1'), capture_output=True, timeout=120)
        for suffix, payload in [('actual.json', p.stdout), ('stderr.log', p.stderr)]:
            with (E / (name + '-' + suffix)).open('xb') as stream:
                stream.write(payload)
        try:
            rows = json.loads(p.stdout)
        except (ValueError, TypeError):
            rows = []
        passed = sum(r['passed'] is True for r in rows)
        expected = [r['expected'] for r in rows]
        records.append(dict(group=name, Required=count, Executed=len(rows), Passed=passed, Failed=len(rows)-passed,
            NotExecuted=count-len(rows), InputSHA256=digest(inputs), InputFileSHA256=inputs,
            ExpectedSHA256=digest(expected) if rows else None, ActualSHA256=sha(E / (name + '-actual.json')),
            ExitCode=p.returncode, seconds=time.time()-started,
            oracle_scope='Frozen independent Expected reference; not a new independent executable oracle' if name == 'r3' else 'Frozen separate executable oracle plus frozen expected vector descriptors'))
    unchanged = all(sha(W / path) == wanted for path, wanted in inputs.items())
    good = unchanged and all(x['Passed'] == x['Required'] and x['ExitCode'] == 0 for x in records)
    result = dict(status='Passed' if good else 'Failed', groups=records,
        Required=252, Executed=sum(x['Executed'] for x in records), Passed=sum(x['Passed'] for x in records),
        Failed=sum(x['Failed'] for x in records), NotExecuted=sum(x['NotExecuted'] for x in records),
        unchanged_inputs=unchanged, python=sys.executable, build_credit=0, root_export_authority=False,
        legacy_windows_history='Archived original 0/8; no new execution credit', independent_executor_attestation=False)
    save(E / 'static-252-results.json', result)
    print(json.dumps({k:v for k,v in result.items() if k != 'groups'}))
    return 0 if good else 1


if __name__ == '__main__':
    raise SystemExit(main())
