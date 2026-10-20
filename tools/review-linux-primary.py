"""Separate readback comparator; no production validator imports or execution.

This is an executable evidence review by the same coding agent, NOT third-party
approval or an attested independent resource executor.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess

W = Path(__file__).resolve().parents[1]
E = W.parent / 'evidence/phase-linux-primary-development-migration'
D = W / 'docs/implementation'
P = 'phase-linux-primary-development-migration-'
P0 = 'phase-04c1b4c4r3c2r5g2r4p0-'
P1 = 'phase-04c1b4c4r3c2r5g2r4p1a-'


def sha(p):
    with p.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def read(p):
    return json.loads(p.read_bytes())


def main():
    checks = []
    def check(name, valid):
        checks.append(dict(id=name, passed=bool(valid)))
    summary = read(E / 'static-252-results.json')
    check('static-total', [summary[x] for x in ['Required','Executed','Passed','Failed','NotExecuted']] == [252,252,252,0,0])
    for row in summary['groups']:
        group = row['group']
        payload = E / (group + '-actual.json')
        actual = read(payload)
        check(group + '-transcript-sha-and-exit', sha(payload) == row['ActualSHA256'] and row['ExitCode'] == 0)
        check(group + '-frozen-inputs', all(sha(W / path) == want for path, want in row['InputFileSHA256'].items()))
        if group == 'r3':
            expected = read(D / 'phase-04c1b4c4r3c2r5g2r3-design-vector-results.json')['vectors']
            check('r3-fixed-roster', [r['id'] for r in actual] == [r['id'] for r in expected] and [r['actual'] for r in actual] == expected)
        elif group == 'p0':
            expected = read(D / (P0 + 'static-test-vectors.json'))['vectors']
            for field in read(D / (P0 + 'evidence-manifest-schema.json'))['required']:
                expected.append(dict(id='omit-root/' + field, valid=False))
            for field in read(D / (P0 + 'environment-candidate-schema.json'))['required']:
                expected.append(dict(id='omit-candidate/' + field, valid=False))
            check('p0-fixed-roster', [r['id'] for r in actual] == [r['id'] for r in expected])
            check('p0-independent-transcript', all((a['actual']['static_status'] == 'Static PASS') == a['independent']['valid'] == b['valid'] and not a['actual']['positive_authority'] for a,b in zip(actual,expected)))
        else:
            expected = read(D / (P1 + 'authorization-static-vectors.json'))['vectors']
            check('p1a-fixed-roster', [r['id'] for r in actual] == [r['id'] for r in expected])
            check('p1a-independent-transcript', all(a['actual']['package_complete'] == a['independent']['package_complete'] == b['expected_complete'] and not a['actual']['P1_authorized'] and not a['actual']['P2_authorized'] and not a['actual']['positive_root_export'] for a,b in zip(actual,expected)))
        check(group + '-full-denominator', len(actual) == row['Required'] == row['Executed'] == row['Passed'] and row['Failed'] == row['NotExecuted'] == 0)
    registry = read(W / 'spec/registry/specifications.json')
    check('normative-source-pins', all(sha(W / item['normative_source']['logical_path']) == item['normative_source']['content_digest']['hex'] for item in registry['specifications']))
    git = ['git','-c','core.hooksPath=/dev/null','-c','core.fsmonitor=false','-c','core.packedGitWindowSize=8m','-c','core.packedGitLimit=32m']
    changed = subprocess.check_output(git + ['diff','--name-only','bf5926a25f5fc1a78fe35a0a775b712a62d27ca9'],cwd=W).decode().splitlines()
    allowed = lambda s: s == 'AGENTS.md' or s.startswith('engineering/policies/') or s.startswith('docs/implementation/' + P) or s in ['tools/linux-primary-static.py','tools/linux-primary-plan.py','tools/test-linux-primary-policy.py','tools/review-linux-primary.py']
    check('no-existing-product-or-expected-changes', all(allowed(p) for p in changed))
    preservation = read(E / 'source-preservation-review.json')
    check('ten-raw-frozen-snapshots', len(preservation['frozen_snapshots']) == 10 and all(s['passed'] and not s['changed_paths'] and not s['git_status'] for s in preservation['frozen_snapshots']))
    check('legacy-failure-preserved', all(sha(W.parent / 'evidence' / path) == want for path,want in preservation['legacy_receipts'].items()))
    legacy = read(E / 'legacy-windows-gate-archive.json')
    check('legacy-contracts-unchanged', all(sha(D / path) == want for path,want in legacy['original_contracts'].items()))
    check('no-legacy-promotion', not legacy['formal_superseded'] and legacy['windows_historical_result'] == '0/8' and not legacy['new_linux_development_prerequisite'])
    resource = read(E / 'resource-policy-approval.json')
    qualification = read(E / 'linux-executor-qualification.json')
    check('no-resource-approval-inference', resource['status'] == 'Not Authorized' and resource['human_approved_budget'] is None and not resource['dependency_install_approval_is_resource_approval'])
    check('no-cgroup-or-build-claims', not qualification['controllers_written'] and qualification['heavy_tasks_launched'] == 0 and not qualification['runtime_executor_implemented'])
    gates = read(E / 'linux-cargo-gate-results.json')
    check('four-native-gates-unexecuted', len(gates['commands']) == 4 and all(x['status'] == 'Not Executed' and x['exit_code'] is None for x in gates['commands']))
    for file in sorted(D.glob(P + '*.json')):
        check('json-parse/' + file.name, isinstance(read(file), (dict, list)))
    print(json.dumps(dict(status='Passed' if all(c['passed'] for c in checks) else 'Failed', checks=checks,
        reviewer='Separate readback comparator executed by the same coding agent; not independent human approval',
        scope='Read-only evidence/roster/identity/denial checks; not Rust runtime or executor qualification',
        third_party_attestation=False),indent=2))
    return 0 if all(c['passed'] for c in checks) else 1


if __name__ == '__main__':
    raise SystemExit(main())
