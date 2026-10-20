"""Independent application-documentation oracle. No primary P1A code imports.

Uses frozen P0 independent Schema interpreter, plus a separate conjunction of
authorization proof obligations. Never issues real environment/executor trust.
"""
import hashlib
import importlib.util
import json
from pathlib import Path


def complete(record):
    keys = ['claimed_value', 'supplied_by', 'evidence_reference', 'verified_status', 'approval_status']
    return (type(record) is dict and set(record) == set(keys) and record['claimed_value'] is not None and
            bool(record['supplied_by']) and bool(record['evidence_reference']) and
            record['verified_status'] == 'Declared — Not Verified' and record['approval_status'] == 'Documented')


def same(a, b):
    return json.dumps(a, sort_keys=True) == json.dumps(b, sort_keys=True)


def evaluate(value, req, trust, candidate_schema):
    props = req['nominal_fields'] + ['source_transfer_permission', 'evidence_return_permission']
    n = value['nomination']
    valid = set(n) == set(props) and all(complete(n[k]) for k in props)
    claims = {k: n[k]['claimed_value'] for k in props}
    valid &= all(n[k]['verified_status'] in ['Unknown', 'Declared — Not Verified'] for k in props)
    valid &= bool(claims['candidate_id']) and bool(claims['environment_owner'])
    # Reuse only the separately implemented frozen P0 schema interpreter.
    path = Path(__file__).with_name('phase-04c1b4c4r3c2r5g2r4p0-independent-static-oracle.py')
    spec = importlib.util.spec_from_file_location('p1a_prior_independent_schema', path)
    prior = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(prior)
    candidate = {k: claims[k] for k in req['nominal_fields']}
    candidate['qualification_status'] = 'Not Verified'
    valid &= prior.shape(candidate, candidate_schema, candidate_schema)
    valid &= all(claims[k] in (None, wanted) for k, wanted in req['os_abi_requirements'].items())
    evidence = value['toolchain_evidence']
    digest = hashlib.sha256(json.dumps(req['gates'], sort_keys=True, separators=(',', ':')).encode()).hexdigest()
    valid &= evidence is not None
    if evidence:
        valid &= evidence['requirement_digest'] == digest
        valid &= not evidence['changed_components'] or complete(evidence['change_approval'])
    valid &= same(value['gate_claims'], req['gates'])
    valid &= set(value['resources']) == set(req['baseline'])
    for name in req['baseline']:
        row = value['resources'].get(name)
        if row is None or not all(complete(row[k]) for k in ['requested', 'owner_capacity', 'approved']):
            valid = False
            continue
        numbers = [row[k]['claimed_value'] for k in ['requested', 'owner_capacity', 'approved']]
        valid &= all(type(x) is int and x > 0 for x in numbers) and numbers[0] <= numbers[1] and numbers[0] == numbers[2]
        roles = dict.fromkeys(['P_build', 'J_build'], 'build-only')
        roles.update(dict.fromkeys(['P_test', 'J_test'], 'test-only'))
        valid &= row['scope'] == roles.get(name, 'system-protection')
        valid &= row['owner_capacity']['supplied_by'] == claims['environment_owner']
        valid &= row['approved']['supplied_by'] in trust['approvers']
        valid &= row['environment_id'] == claims['candidate_id'] and bool(row['purpose']) and row['approval_reference'] in trust['approval_references']
        valid &= row['measured_peak']['claimed_value'] is None
    valid &= value['requested_phase'] == 'P1'
    valid &= set(value['permissions']) == set(req['permissions'])
    for name, phase in req['permissions'].items():
        grant = value['permissions'].get(name)
        if grant is None:
            valid = False
            continue
        if phase == 'P2':
            valid &= grant['granted'] is False
        else:
            valid &= (grant['granted'] is True and grant['phase'] == 'P1' and grant['authorizer'] in trust['approvers'] and
                      grant['environment_id'] == claims['candidate_id'] and bool(grant['purpose']) and grant['evidence_reference'] in trust['approval_references'])
    t = value['trust']
    valid &= set(t) == set(['executor_id', 'root_id', 'root_approver', 'approval_identity', 'signing_policy', 'verification_policy', 'freshness_policy', 'replay_policy'])
    valid &= all(complete(x) for x in t.values())
    root, approver, executor = [t[k]['claimed_value'] for k in ['root_id', 'root_approver', 'executor_id']]
    identity = claims['trusted_executor_identity']
    roots = claims['approved_trust_roots']
    valid &= isinstance(identity, dict) and identity.get('identity') == executor
    valid &= isinstance(roots, list) and any(r['key_id'] == root and r['approval_reference'] in trust['approval_references'] for r in roots)
    valid &= t['approval_identity']['claimed_value'] in trust['approval_references']
    valid &= root in trust['roots'] and approver in trust['approvers'] and executor in trust['executors']
    valid &= executor is None or executor != approver
    valid &= value['execution_id'] not in trust['old_execution_ids'] and value['signature_id'] not in trust['old_signature_ids']
    valid &= same(value['lane_b'], dict(paths=2403, runs=82, generators=41, byte_drift=0, expanded_tree_pass=False))
    valid &= value['execution_status'] == 'Not Executed' and not value['environment_qualified'] and not value['positive_root_export']
    return dict(package_complete=bool(valid), actual_environment_connected=False, actual_environment_qualified=False,
                actual_executor_attested=False, P1_authorized=False, P2_authorized=False, positive_root_export=False)
