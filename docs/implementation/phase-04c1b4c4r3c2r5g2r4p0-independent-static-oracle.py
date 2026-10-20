"""Independent P0 oracle: no import/call of the manifest validator or its results.

Small independent interpreter for the exact shipped Schema keyword subset, then
orthogonal contract equality checks. This is not a general JSON Schema library.
"""
import json
import re


def equal(a, b):
    return json.dumps(a, sort_keys=True, separators=(',', ':')) == json.dumps(b, sort_keys=True, separators=(',', ':'))


def shape(value, schema, root, depth=0):
    if depth > 32:
        return False
    if '$ref' in schema:
        return shape(value, root['$defs'][schema['$ref'].split('/')[-1]], root, depth + 1)
    if 'anyOf' in schema and not any(shape(value, s, root, depth + 1) for s in schema['anyOf']):
        return False
    if 'const' in schema and not equal(value, schema['const']):
        return False
    if 'enum' in schema and not any(equal(value, e) for e in schema['enum']):
        return False
    types = schema.get('type')
    if types:
        names = types if isinstance(types, list) else [types]
        actual = {dict: 'object', list: 'array', str: 'string', int: 'integer', bool: 'boolean', type(None): 'null'}.get(type(value))
        if actual not in names:
            return False
    if type(value) is dict:
        props = schema.get('properties', {})
        if not set(schema.get('required', [])) <= set(value):
            return False
        if schema.get('additionalProperties') is False and not set(value) <= set(props):
            return False
        if not all(shape(v, props.get(k, {}), root, depth + 1) for k, v in value.items()):
            return False
    if type(value) is list:
        if not schema.get('minItems', 0) <= len(value) <= schema.get('maxItems', 50000):
            return False
        if schema.get('uniqueItems') and len({json.dumps(v, sort_keys=True) for v in value}) != len(value):
            return False
        if not all(shape(v, schema.get('items', {}), root, depth + 1) for v in value):
            return False
    if type(value) is str:
        if not schema.get('minLength', 0) <= len(value) <= schema.get('maxLength', 8192):
            return False
        if 'pattern' in schema and re.search(schema['pattern'], value) is None:
            return False
        if any(ord(c) < 32 for c in value):
            return False
    if type(value) is int and not schema.get('minimum', -(2 ** 63)) <= value <= schema.get('maximum', 2 ** 63 - 1):
        return False
    return type(value) in (dict, list, str, int, bool, type(None))


def evaluate(value, contract, schema):
    valid = shape(value, schema, schema)
    blockers = {'P1_NOT_AUTHORIZED', 'NO_APPROVED_TRUST_ROOTS', 'NO_AUTHENTICATED_EXECUTOR', 'BUILD_BUDGET_PENDING_APPROVAL'}
    if not valid:
        return dict(valid=False, qualification_blockers=sorted(blockers), authority=False)
    env = value['candidate']
    if value['environment_id'] != env['candidate_id']:
        valid = False
    prior = contract['environment_bindings'].get(env['candidate_id'])
    if prior and env['environment_owner'] != prior['environment_owner']:
        valid = False
    for name, target in {'operating_system': 'Windows', 'architecture': 'x86_64', 'target_abi': 'x86_64-pc-windows-msvc'}.items():
        if env[name] not in (None, target):
            valid = False
    def known(value):
        if isinstance(value, dict):
            return all(known(v) for v in value.values())
        return value is not None
    if any(not known(env[n]) for n in 'rust_toolchain cargo_version rustc_version llvm_version msvc_toolset windows_sdk python_runtime'.split()):
        blockers.add('TOOLCHAIN_INCOMPLETE')
    for field, code in [('dependency_sources', 'DEPENDENCY_SOURCE_UNKNOWN'), ('dependency_lockfiles', 'DEPENDENCY_LOCK_UNKNOWN'),
                        ('offline_cache_identity', 'CACHE_UNVERIFIED'), ('approval_reference', 'ENVIRONMENT_APPROVAL_MISSING'),
                        ('trusted_executor_identity', 'EXECUTOR_UNAUTHENTICATED')]:
        if not env[field]:
            blockers.add(code)
    if not equal(value['resource_policy'], contract['resource_policy']):
        valid = False
    if env['build_resource_policy'] and any(env['build_resource_policy'][n] is not None for n in ['P_build', 'J_build']):
        valid = False
    expected_gates = contract['gates']
    names = [g['gate_id'] for g in value['gates']]
    valid &= len(names) == len(set(names)) and set(names) == {g['gate_id'] for g in expected_gates}
    expected = {g['gate_id']: g for g in expected_gates}
    for g in value['gates']:
        want = expected.get(g['gate_id'])
        if want is None:
            valid = False
            continue
        for name in ['frozen_head', 'debug', 'refs', 'case_count', 'commands']:
            valid &= equal(g[name], want[name])
        valid &= g['environment_id'] == value['environment_id'] and all(g['closure'].values())
        valid &= g['artifact_owner'] == want['frozen_head']
        if value['scope'] == 'Preparation':
            valid &= all(g[n] is None for n in ['case_ids', 'expected', 'actual', 'artifacts'])
        else:
            valid &= equal(g['case_ids'], want['synthetic_case_ids'])
            valid &= equal(g['expected'], want['synthetic_records'])
            valid &= equal(g['actual'], want['synthetic_records'])
            valid &= equal(g['artifacts'], want['synthetic_artifacts'])
    valid &= equal(value['lane_b'], contract['lane_b'])
    valid &= equal(value['capability_denial'], contract['capability_denial'])
    valid &= value['execution_id'] not in contract['previous_execution_ids']
    valid &= value['claimed_status'] == value['execution_status']
    valid &= value['trusted_status'] == 'Not Established' and not value['positive_authority']
    state, code, cancel = value['execution_status'], value['actual_exit_code'], value['cancelled']
    times = value['execution_window']
    if any(t is not None for t in times.values()):
        valid &= all(type(t) is int for t in times.values()) and state != 'Not Executed'
        if all(type(t) is int for t in times.values()):
            valid &= times['start_utc_ns'] <= times['end_utc_ns'] and times['start_monotonic_ns'] <= times['end_monotonic_ns']
    valid &= ((state == 'Not Executed' and code is None and cancel is None) or
              (state == 'Failed' and code not in (None, 0) and cancel is None) or
              (state == 'Interrupted By Resource Safety' and code not in (None, 0) and cancel is not None))
    if any(value['signature'].values()):
        # Independent canonical message construction; no import of validator.
        import hashlib
        import base64
        try:
            valid &= all(v is not None for v in value['signature'].values()) and len(base64.b64decode(value['signature']['signature_base64'], validate=True)) == 64
        except (ValueError, TypeError):
            valid = False
        payload = json.loads(json.dumps(value))
        payload['signature']['payload_sha256'] = None
        payload['signature']['signature_base64'] = None
        message = b'R5G2R4-EVIDENCE-v1\0' + json.dumps(payload, sort_keys=True, ensure_ascii=False, separators=(',', ':')).encode()
        valid &= hashlib.sha256(message).hexdigest() == value['signature']['payload_sha256']
    if value['signature']['signature_base64'] is None:
        blockers.add('SIGNATURE_MISSING')
    return dict(valid=bool(valid), qualification_blockers=sorted(blockers), authority=False)
