"""Independent fixed expected routing cases. No commands or cgroups launched."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path

W = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('linux_primary_planner', W / 'tools/linux-primary-plan.py')
planner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(planner)
BASE = dict(policy_id='agentflow.engineering.linux-primary-development@1', git_identity_valid=True,
            dependency_closure_complete=True, static_252_passed=True, numeric_resource_approval_verified=True,
            dedicated_executor_qualified=True, legacy_windows_history='0/8')
rows = []
def case(name, state, expected):
    actual = planner.route_model(state)
    passed = actual['planning_status'] == expected and actual['launched'] is False and actual['execution_authorized'] is False
    passed = passed and actual['root_export_authorized'] is False and actual['windows_release_status'] == 'Not Executed'
    rows.append(dict(id=name, state=state, expected=expected, actual=actual, passed=passed))
case('archived-windows-0of8-not-a-development-prerequisite', BASE, 'Ready for separately authorized executor')
for legacy in ['0/8', 'Failed', None]:
    state = dict(BASE, legacy_windows_history=legacy)
    case('legacy-is-not-promoted-or-consumed/' + str(legacy), state, 'Ready for separately authorized executor')
for key in ['git_identity_valid','dependency_closure_complete','static_252_passed','numeric_resource_approval_verified','dedicated_executor_qualified']:
    for invalid in [False, None, 'Approved', 1]:
        state = dict(BASE); state[key] = invalid
        case('fail-closed/' + key + '/' + repr(invalid), state, 'Blocked')
state = dict(BASE, policy_id='unknown')
case('unknown-policy', state, 'Blocked')
state = dict(BASE, numeric_resource_approval_verified=False, dedicated_executor_qualified=False)
case('actual-pending-resource-and-executor', state, 'Blocked')
print(json.dumps(dict(Required=26, Executed=len(rows), Passed=sum(r['passed'] for r in rows), Failed=sum(not r['passed'] for r in rows),
    cases=rows, scope='Engineering routing model only; synthetic state never authorizes runtime execution'),indent=2))
assert len(rows) == 26 and all(r['passed'] for r in rows)
