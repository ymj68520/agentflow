"""Planning-only engineering router; deliberately has NO process launch backend.

Caller-supplied booleans in self-tests are models, never runtime authority. A
future executor must independently validate exact-head approval and qualification.
"""
import json
from pathlib import Path

W = Path(__file__).resolve().parents[1]
E = W.parent / 'evidence/phase-linux-primary-development-migration'
POLICY = 'agentflow.engineering.linux-primary-development@1'
COMMANDS = [
    ['cargo', 'fmt', '--all', '--', '--check'],
    ['cargo', 'clippy', '--locked', '--all-targets', '--all-features', '--', '-D', 'warnings'],
    ['cargo', 'test', '--locked', '--all', '-j', '1'],
    ['cargo', 'run', '--locked', '-p', 'af-spec', '--bin', 'spec-check', '--', 'check'],
]


def route_model(state):
    reasons = []
    if state.get('policy_id') != POLICY:
        reasons.append('POLICY_ID')
    for key in ['git_identity_valid', 'dependency_closure_complete', 'static_252_passed',
                'numeric_resource_approval_verified', 'dedicated_executor_qualified']:
        if state.get(key) is not True:
            reasons.append(key)
    # Archived Windows results intentionally do not participate in this decision.
    return dict(policy_id=POLICY, planning_status='Ready for separately authorized executor' if not reasons else 'Blocked',
                reasons=reasons, commands=COMMANDS, env=dict(CARGO_BUILD_JOBS='1', RUST_TEST_THREADS='1', CARGO_NET_OFFLINE='true'),
                launched=False, execution_authorized=False, root_export_authorized=False,
                windows_release_status='Not Executed', caller_state_is_runtime_authority=False)


def main():
    def read(name):
        return json.loads((E / name).read_bytes())
    # This phase has no verified resource approval / qualified executor. Never
    # infer either from a JSON status label or dependency-install authorization.
    state = dict(policy_id=POLICY, git_identity_valid=read('source-migration.json')['ten_frozen_snapshots_preserved'] is True,
                 dependency_closure_complete=all(read(n)['status'] == 'Complete' for n in ['python-dependency-closure.json','cargo-dependency-closure.json']),
                 static_252_passed=read('static-252-results.json')['Passed'] == 252,
                 numeric_resource_approval_verified=False, dedicated_executor_qualified=False,
                 legacy_windows_history='0/8')
    print(json.dumps(route_model(state), indent=2))


if __name__ == '__main__':
    main()
