"""P0 OFFLINE STATIC validator. No execution/transfer/environment capability.

JSON Schema plus contract checks; cryptographic validity is deliberately separate
from executor trust and authority. There is no production trust root in this file.
"""
import base64
import hashlib
import json
from pathlib import Path
import sys

from jsonschema import Draft202012Validator, validators
from referencing import Registry

P = 'phase-04c1b4c4r3c2r5g2r4p0-'
D = Path(__file__).resolve().parent
MAX_JSON_BYTES = 4 * 1024 * 1024


def canonical(value):
    strict_types(value)
    return json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(',', ':'), allow_nan=False).encode('utf8')


def strict_types(value, depth=0, budget=None):
    if budget is None:
        budget = [50000]
    budget[0] -= 1
    if depth > 32 or budget[0] < 0:
        raise ValueError('JSON complexity limit')
    if type(value) not in (dict, list, str, int, bool, type(None)):
        raise ValueError('Exact JSON types required; floats are forbidden')
    if type(value) is int and not -(2 ** 63) <= value < 2 ** 63:
        raise ValueError('Integer outside int64')
    if type(value) is str and (len(value) > 8192 or any(ord(c) < 32 for c in value)):
        raise ValueError('Invalid/control/oversize string')
    if type(value) is dict:
        for key, child in value.items():
            if type(key) is not str:
                raise ValueError('Non-string object key')
            strict_types(key, depth + 1, budget)
            strict_types(child, depth + 1, budget)
    if type(value) is list:
        for child in value:
            strict_types(child, depth + 1, budget)


def loads(raw):
    if len(raw) > MAX_JSON_BYTES:
        raise ValueError('Manifest exceeds read limit')
    # Reject excessive nesting before json.loads allocates the object graph.
    quoted = escaped = False
    depth = 0
    for c in raw.decode('utf8'):
        if quoted:
            if escaped:
                escaped = False
            elif c == '\\':
                escaped = True
            elif c == '"':
                quoted = False
        elif c == '"':
            quoted = True
        elif c in '[{':
            depth += 1
            if depth > 32:
                raise ValueError('JSON depth limit')
        elif c in ']}':
            depth -= 1
    def pairs(items):
        result = {}
        for key, value in items:
            if key in result:
                raise ValueError('Duplicate JSON key: ' + key)
            result[key] = value
        return result
    def invalid(value):
        raise ValueError('Forbidden numeric value: ' + value)
    value = json.loads(raw, object_pairs_hook=pairs, parse_float=invalid, parse_constant=invalid)
    strict_types(value)
    return value


def read(path):
    with Path(path).open('rb') as stream:
        return loads(stream.read(MAX_JSON_BYTES + 1))


def no_network(uri):
    raise ValueError('External schema retrieval denied: ' + uri)


# Unlike JSON Schema's mathematically integral numbers, this closed evidence
# codec permits only actual integer tokens, never bool or float lookalikes.
StrictSchema = validators.extend(Draft202012Validator, type_checker=
    Draft202012Validator.TYPE_CHECKER.redefine('integer', lambda checker, value: type(value) is int))


def schema_errors(value, schema):
    strict_types(value)
    Draft202012Validator.check_schema(schema)
    # Only internal $defs references are permitted by the shipped schemas.
    def refs(node):
        if isinstance(node, dict):
            if '$ref' in node and not node['$ref'].startswith('#/$defs/'):
                raise ValueError('Non-local schema reference')
            for child in node.values():
                refs(child)
        elif isinstance(node, list):
            for child in node:
                refs(child)
    refs(schema)
    return sorted(('/' + '/'.join(map(str, e.absolute_path)) + ': ' + e.message)
                  for e in StrictSchema(schema, registry=Registry(retrieve=no_network)).iter_errors(value))


def verify_file_reference(root, reference):
    """Streaming integrity only. Does not establish origin or executor trust."""
    root = Path(root).resolve(strict=True)
    name = reference['path']
    if not isinstance(name, str) or '\\' in name or ':' in name or name.startswith('/'):
        raise ValueError('Non-logical path')
    parts = name.split('/')
    if any(p in ('', '.', '..') for p in parts):
        raise ValueError('Traversal/empty path')
    path = root
    for part in parts:
        path = path / part
        if path.is_symlink() or (hasattr(path, 'is_junction') and path.is_junction()):
            raise ValueError('Link/reparse target denied')
    actual = path.resolve(strict=True)
    if not actual.is_relative_to(root) or not actual.is_file():
        raise ValueError('File outside root')
    if type(reference['byte_length']) is not int or actual.stat().st_size != reference['byte_length']:
        return False
    digest = hashlib.sha256()
    with actual.open('rb') as stream:
        while chunk := stream.read(1024 * 1024):
            digest.update(chunk)
    return digest.hexdigest() == reference['sha256']


def verify_public_signature(message, signature, public_key):
    """Ed25519 primitive validity only, never a trust/qualification decision."""
    from cryptography.exceptions import InvalidSignature
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey
    try:
        if len(public_key) != 32 or len(signature) != 64:
            return False
        Ed25519PublicKey.from_public_bytes(public_key).verify(signature, message)
        return True
    except (InvalidSignature, ValueError):
        return False


def verify_detached(value, public_key=None):
    """Caller key is merely a verification input, NOT an approved trust root.

    Canonical signing covers all fields except detached signature bytes and its
    own digest; key id/algorithm remain bound. No private-key operation exists.
    """
    envelope = value['signature']
    payload = dict(value, signature=dict(envelope, payload_sha256=None, signature_base64=None))
    message = b'R5G2R4-EVIDENCE-v1\0' + canonical(payload)
    matches = envelope['payload_sha256'] == hashlib.sha256(message).hexdigest()
    valid = False
    if public_key is not None and matches and envelope['algorithm'] == 'Ed25519':
        try:
            signature = base64.b64decode(envelope['signature_base64'], validate=True)
            valid = verify_public_signature(message, signature, public_key)
        except (ValueError, TypeError):
            valid = False
    return dict(hash_integrity=matches, signature_valid=valid,
                cryptographic_trust='Not Established', executor_authority='Not Established',
                formal_authority=False)


def validate(value, contract, schema):
    result = dict(static_status='Failed', errors=[], schema_errors=[],
                  qualification_blockers=['P1_NOT_AUTHORIZED', 'NO_APPROVED_TRUST_ROOTS', 'NO_AUTHENTICATED_EXECUTOR'],
                  crypto_schema='Validatable', cryptographic_trust='Not Established',
                  executor_authority='Not Established', environment_status='NOT QUALIFIED',
                  real_environment_qualification='Not Executed', real_executor_attestation='Not Executed',
                  real_historical_gate_execution='Not Executed', execution_authorization='Not Authorized',
                  historical_builds='0/8', positive_authority=False, execution_credit=0)
    errors = result['errors']
    try:
        problems = schema_errors(value, schema)
    except (ValueError, TypeError, RecursionError) as exc:
        problems = [str(exc)]
    if problems:
        result.update(errors=['SCHEMA'], schema_errors=problems, crypto_schema='Failed')
        return result
    env = value['candidate']
    if value['environment_id'] != env['candidate_id']:
        errors.append('ENVIRONMENT_ID')
    prior_environment = contract['environment_bindings'].get(env['candidate_id'])
    if prior_environment is not None and env['environment_owner'] != prior_environment['environment_owner']:
        errors.append('ENVIRONMENT_CONFLICT')
    for field, want in [('operating_system', 'Windows'), ('architecture', 'x86_64'),
                        ('target_abi', 'x86_64-pc-windows-msvc')]:
        if env[field] is not None and env[field] != want:
            errors.append('OS_ABI')
    toolfields = ['rust_toolchain', 'cargo_version', 'rustc_version', 'llvm_version',
                  'msvc_toolset', 'windows_sdk', 'python_runtime']
    def known(value):
        return value is not None and (not isinstance(value, dict) or all(known(v) for v in value.values()))
    if any(not known(env[n]) for n in toolfields):
        result['qualification_blockers'].append('TOOLCHAIN_INCOMPLETE')
    for name, code in [('dependency_sources', 'DEPENDENCY_SOURCE_UNKNOWN'),
                       ('dependency_lockfiles', 'DEPENDENCY_LOCK_UNKNOWN'),
                       ('offline_cache_identity', 'CACHE_UNVERIFIED'),
                       ('approval_reference', 'ENVIRONMENT_APPROVAL_MISSING'),
                       ('trusted_executor_identity', 'EXECUTOR_UNAUTHENTICATED')]:
        if not env[name]:
            result['qualification_blockers'].append(code)
    if any(env[n] is None for n in ['candidate_id', 'environment_owner', 'execution_location',
                                    'execution_authority', 'host_or_vm_identity', 'os_build',
                                    'cpu_topology', 'physical_memory', 'available_memory',
                                    'commit_limit', 'filesystem_capacity']):
        result['qualification_blockers'].append('CONCRETE_ENVIRONMENT_MISSING')
    policy = value['resource_policy']
    if policy != contract['resource_policy']:
        errors.append('UNAPPROVED_RESOURCE_POLICY')
    for which in ['build_resource_policy', 'test_resource_policy', 'foreground_reserve_policy']:
        if env[which] is not None:
            # Candidate-declared policy cannot approve itself.
            result['qualification_blockers'].append('CANDIDATE_POLICY_NOT_APPROVED')
    result['qualification_blockers'].append('BUILD_BUDGET_PENDING_APPROVAL')
    if env['build_resource_policy'] is not None and any(env['build_resource_policy'].get(k) is not None for k in ['P_build', 'J_build']):
        errors.append('UNAPPROVED_RESOURCE_POLICY')
    trusted = {g['gate_id']: g for g in contract['gates']}
    actual = {g['gate_id']: g for g in value['gates']}
    if set(actual) != set(trusted) or len(actual) != len(value['gates']):
        errors.append('GATE_ROSTER')
    for gate in value['gates']:
        required = trusted.get(gate['gate_id'])
        if required is None:
            continue
        if gate['environment_id'] != value['environment_id']:
            errors.append('ENVIRONMENT_ID')
        if gate['frozen_head'] != required['frozen_head']:
            errors.append('FROZEN_HEAD')
        if gate['debug'] != required['debug']:
            errors.append('DEBUG_PROFILE')
        if gate['refs'] != required['refs']:
            errors.append('REFERENCE_CLOSURE')
        if gate['case_count'] != required['case_count']:
            errors.append('CASE_DENOMINATOR')
        if gate['commands'] != required['commands']:
            errors.append('COMMAND_CONTRACT')
        if not all(gate['closure'].values()):
            errors.append('TARGET_CLOSURE')
        if value['scope'] == 'Synthetic':
            names = required['synthetic_case_ids']
            for side in ['expected', 'actual']:
                if gate[side] != required['synthetic_records']:
                    errors.append('CASE_FIELD_CLOSURE')
            if gate['case_ids'] != names:
                errors.append('CASE_DENOMINATOR')
            if gate['artifacts'] != required['synthetic_artifacts']:
                errors.append('ARTIFACT_CLOSURE')
        elif any(gate[n] is not None for n in ['expected', 'actual', 'case_ids', 'artifacts']):
            errors.append('PREPARATION_IS_NOT_EXECUTION')
        if gate['artifact_owner'] != gate['frozen_head']:
            errors.append('CROSS_EDITION_ARTIFACT')
    if value['lane_b'] != contract['lane_b']:
        errors.append('LANE_B_SCOPE')
    if value['execution_id'] in contract['previous_execution_ids']:
        errors.append('EXECUTION_REPLAY')
    status = value['execution_status']
    window = [value['execution_window'][k] for k in ['start_utc_ns', 'end_utc_ns', 'start_monotonic_ns', 'end_monotonic_ns']]
    if (any(v is not None for v in window) and
        (not all(type(v) is int for v in window) or window[0] > window[1] or window[2] > window[3])) or (status == 'Not Executed' and any(v is not None for v in window)):
        errors.append('EXECUTION_WINDOW')
    if value['claimed_status'] != status:
        errors.append('STATUS_REWRITE')
    if status == 'Not Executed' and (value['actual_exit_code'] is not None or value['cancelled'] is not None):
        errors.append('RAW_EXIT_STATUS')
    if status == 'Failed' and (value['actual_exit_code'] in (None, 0) or value['cancelled'] is not None):
        errors.append('RAW_EXIT_STATUS')
    if status == 'Interrupted By Resource Safety' and (value['actual_exit_code'] in (None, 0) or value['cancelled'] is None):
        errors.append('RAW_EXIT_STATUS')
    if status == 'PASS':
        errors.append('P0_EXECUTION_CLAIM')
    if value['trusted_status'] != 'Not Established':
        errors.append('UNESTABLISHED_TRUST')
    if value['positive_authority']:
        errors.append('POSITIVE_AUTHORITY')
    if value['capability_denial'] != contract['capability_denial']:
        errors.append('CAPABILITY_DENIAL')
    signature = value['signature']
    if any(signature.values()):
        try:
            if any(v is None for v in signature.values()) or len(base64.b64decode(signature['signature_base64'], validate=True)) != 64:
                raise ValueError('Incomplete/invalid detached signature envelope')
        except (ValueError, TypeError):
            errors.append('CRYPTO_ENVELOPE')
            result['crypto_schema'] = 'Failed'
        checked = verify_detached(value)
        if not checked['hash_integrity']:
            errors.append('SIGNATURE_PAYLOAD_HASH')
    if signature['signature_base64'] is None:
        result['qualification_blockers'].append('SIGNATURE_MISSING')
    result['errors'] = sorted(set(errors))
    result['qualification_blockers'] = sorted(set(result['qualification_blockers']))
    result['static_status'] = 'Static PASS' if not errors else 'Failed'
    return result


if __name__ == '__main__':
    try:
        if len(sys.argv) != 2:
            raise ValueError('Usage: python offline-manifest-validator.py LOCAL_MANIFEST.json')
        result = validate(read(sys.argv[1]), read(D / (P + 'validation-contract.json')),
                          read(D / (P + 'evidence-manifest-schema.json')))
    except Exception as exc:
        result = dict(static_status='Failed', error=str(exc), execution_authorization='Not Authorized')
    print(json.dumps(result, ensure_ascii=False, indent=2))
    sys.exit(0 if result['static_status'] == 'Static PASS' else 2)
