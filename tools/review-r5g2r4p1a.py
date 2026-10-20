"""LOCAL APPROVAL REVIEW ONLY. No connection, transfer, build or key operations.

Old contracts/receipts are read-only. Only new P1A documents/target receipts may
be written. A complete synthetic application never grants real P1/P2 authority.
"""
import copy
import ast
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

R = Path(__file__).resolve().parents[1]
D = R / 'docs/implementation'
P = 'phase-04c1b4c4r3c2r5g2r4p1a-'
P0 = 'phase-04c1b4c4r3c2r5g2r4p0-'
START = 'd2e734ef3cb8a59f05487baa928f85bda4779e5a'
SAFE = 'phase-04c1b4c4r3c2r4g3-safe-'
RUNNER_SHA = '93672e9d71aaa7fb9a14acc66eeb10e4ef2ca424515fc9eb12fd48cef4daab07'
ATTACHMENT = Path('C:/Users/Administrator/.codex/attachments/754748d3-efb7-4022-849c-02067c51e365/已粘贴的文本.txt')


def module(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


V = module(R / 'tools/verify-r5g2r1.py', 'p1a_readonly_helpers')


def read(path):
    return json.loads((R / path).read_bytes())


def doc(name, prefix=P):
    return read('docs/implementation/' + prefix + name + '.json')


def write(name, value, target=False):
    path = (R / 'target' if target else D) / (P + name + '.json')
    path.write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode('utf8'))
    return path


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def unknown():
    return dict(claimed_value=None, supplied_by=None, evidence_reference=None,
                verified_status='Unknown', approval_status='Not Provided')


def supplied(record):
    return (type(record) is dict and set(record) == {'claimed_value', 'supplied_by', 'evidence_reference', 'verified_status', 'approval_status'} and
            record['claimed_value'] is not None and bool(record['supplied_by']) and bool(record['evidence_reference']) and
            record['verified_status'] == 'Declared — Not Verified' and record['approval_status'] == 'Documented')


def contract():
    gates = doc('historical-gate-contracts', P0)['groups'] + [doc('frozen-g3-gate-contract', P0)['gate']]
    compact = [{k: g[k] for k in ['gate_id', 'frozen_head', 'refs', 'debug', 'case_count', 'commands']} for g in gates]
    baseline = doc('resource-policy-analysis', P0)['current_baseline']
    parsed = ast.parse((D / (P0 + 'offline-manifest-validator.py')).read_text(encoding='utf8'))
    compatibility = [ast.literal_eval(n.iter) for n in ast.walk(parsed) if isinstance(n, ast.For)
                     and isinstance(n.target, ast.Tuple) and [a.id for a in n.target.elts if isinstance(a, ast.Name)] == ['field', 'want']]
    assert len(compatibility) == 1
    return dict(gates=compact, nominal_fields=doc('environment-candidate-schema', P0)['required'],
                os_abi_requirements=dict(compatibility[0]),
                baseline={n: baseline[k] for n, k in {'P_build':'P_build', 'J_build':'J_build', 'P_test':'P_test', 'J_test':'J_test',
                    'SystemReserve':'SystemReserve', 'CPU Policy':'cpu_percent', 'Build Jobs':'cargo_jobs', 'Test Threads':'test_threads', 'Top-level Job Count':'top_level_jobs'}.items()},
                permissions={'Environment Connection':'P1', 'Source Transfer':'P1', 'Artifact Transfer':'P1', 'Evidence Return':'P1',
                    'Remote Command Execution':'P1', 'Environment Inspection':'P1', 'Build Resource Policy Amendment':'P1',
                    'Historical Build Launch':'P2', 'Frozen G3 Build Launch':'P2'},
                compatibility_source='Frozen P0 environment-equivalence-audit + candidate schema + offline validator; no new compatibility specification',
                p0_source_head=START)


def evaluate(value, requirements, trust):
    """Adjudicates application documentation, not signatures or actual machines."""
    errors = []
    nomination = value['nomination']
    fields = requirements['nominal_fields'] + ['source_transfer_permission', 'evidence_return_permission']
    if set(nomination) != set(fields) or not all(supplied(nomination[n]) for n in fields):
        errors.append('NOMINATION_INCOMPLETE')
    for field in fields:
        if field not in nomination:
            continue
        if nomination[field]['verified_status'] not in ['Unknown', 'Declared — Not Verified']:
            errors.append('DECLARATION_IS_NOT_MEASUREMENT')
    claims = {n: nomination.get(n, unknown())['claimed_value'] for n in fields}
    if not claims['candidate_id']:
        errors.append('ENVIRONMENT_ID_MISSING')
    if not claims['environment_owner']:
        errors.append('OWNER_MISSING')
    # Use the actual frozen P0 implementation/schema for OS/ABI and structural
    # compatibility, instead of inventing a second compatibility specification.
    p0 = module(D / (P0 + 'offline-manifest-validator.py'), 'p1a_frozen_p0_validator')
    projected = {n: claims[n] for n in requirements['nominal_fields']}
    projected['qualification_status'] = 'Not Verified'
    if p0.schema_errors(projected, doc('environment-candidate-schema', P0)) or any(
            claims[field] is not None and claims[field] != want for field, want in requirements['os_abi_requirements'].items()):
        errors.append('P0_COMPATIBILITY')
    evidence = value['toolchain_evidence']
    if evidence is None or evidence['requirement_digest'] != digest(requirements['gates']):
        errors.append('TOOLCHAIN_EVIDENCE_INCOMPLETE')
    elif evidence['changed_components'] and not supplied(evidence['change_approval']):
        errors.append('UNAPPROVED_TOOLCHAIN_CHANGE')
    if json.dumps(value['gate_claims'], sort_keys=True) != json.dumps(requirements['gates'], sort_keys=True):
        errors.append('FROZEN_GATE_CONTRACT')
    resources = value['resources']
    if set(resources) != set(requirements['baseline']):
        errors.append('RESOURCE_ROSTER')
    for name in requirements['baseline']:
        row = resources.get(name)
        if row is None or not all(supplied(row[k]) for k in ['requested', 'owner_capacity', 'approved']):
            errors.append('RESOURCE_APPROVAL_INCOMPLETE')
            continue
        wanted, capacity, approved = [row[k]['claimed_value'] for k in ['requested', 'owner_capacity', 'approved']]
        if any(type(v) is not int or v <= 0 for v in [wanted, capacity, approved]) or wanted > capacity or approved != wanted:
            errors.append('RESOURCE_VALUE_BINDING')
        expected_scope = 'build-only' if name in ['P_build', 'J_build'] else 'test-only' if name in ['P_test', 'J_test'] else 'system-protection'
        if row['scope'] != expected_scope:
            errors.append('BUILD_TEST_BUDGET_MIX')
        if row['owner_capacity']['supplied_by'] != claims['environment_owner'] or row['approved']['supplied_by'] not in trust['approvers']:
            errors.append('RESOURCE_APPROVER_BINDING')
        if row['environment_id'] != claims['candidate_id'] or not row['purpose'] or row['approval_reference'] not in trust['approval_references']:
            errors.append('RESOURCE_APPROVER_BINDING')
        if row['measured_peak']['claimed_value'] is not None:
            errors.append('DECLARATION_IS_NOT_MEASUREMENT')
    if value['requested_phase'] != 'P1':
        errors.append('P1_IS_NOT_P2')
    permissions = value['permissions']
    if set(permissions) != set(requirements['permissions']):
        errors.append('PERMISSION_ROSTER')
    for name, domain in requirements['permissions'].items():
        permission = permissions.get(name)
        if permission is None:
            errors.append('PERMISSION_MISSING')
            continue
        if domain == 'P1':
            if not (permission['granted'] is True and permission['phase'] == domain and
                    permission['authorizer'] in trust['approvers'] and permission['environment_id'] == claims['candidate_id'] and
                    permission['purpose'] and permission['evidence_reference'] in trust['approval_references']):
                errors.append('PERMISSION_MISSING')
        elif permission['granted'] is not False:
            errors.append('P1_IS_NOT_P2')
    trust_fields = value['trust']
    if set(trust_fields) != set(['executor_id', 'root_id', 'root_approver', 'approval_identity', 'signing_policy', 'verification_policy', 'freshness_policy', 'replay_policy']) or not all(supplied(r) for r in trust_fields.values()):
        errors.append('TRUST_BINDING_INCOMPLETE')
    root = trust_fields['root_id']['claimed_value']
    actor = trust_fields['executor_id']['claimed_value']
    approver = trust_fields['root_approver']['claimed_value']
    if root not in trust['roots']:
        errors.append('TRUST_ROOT_NOT_APPROVED')
    if approver not in trust['approvers']:
        errors.append('APPROVER_NOT_APPROVED')
    if actor is not None and actor == approver:
        errors.append('EXECUTOR_SELF_APPROVAL')
    if actor not in trust['executors']:
        errors.append('EXECUTOR_NOT_IDENTIFIED')
    identity = claims['trusted_executor_identity']
    roots = claims['approved_trust_roots']
    if (not isinstance(identity, dict) or identity.get('identity') != actor or not isinstance(roots, list) or
        not any(r['key_id'] == root and r['approval_reference'] in trust['approval_references'] for r in roots) or
        trust_fields['approval_identity']['claimed_value'] not in trust['approval_references']):
        errors.append('TRUST_BINDING_INCOMPLETE')
    if value['execution_id'] in trust['old_execution_ids'] or value['signature_id'] in trust['old_signature_ids']:
        errors.append('REPLAY')
    if json.dumps(value['lane_b'], sort_keys=True) != json.dumps(dict(paths=2403, runs=82, generators=41, byte_drift=0, expanded_tree_pass=False), sort_keys=True):
        errors.append('LANE_B_SCOPE')
    if value['execution_status'] != 'Not Executed':
        errors.append('UNEXECUTED_GATE_PASS')
    if value['positive_root_export'] or value['environment_qualified']:
        errors.append('QUALIFICATION_IS_NOT_CAPABILITY')
    return dict(package_complete=not errors, errors=sorted(set(errors)),
                actual_environment_connected=False, actual_environment_qualified=False,
                actual_executor_attested=False, historical_builds_executed='0/8', frozen_g3_executed=False,
                P1_authorized=False, P2_authorized=False, positive_root_export=False,
                cryptographic_trust='Not Established', executor_authority='Not Established')


def empty_application(requirements):
    names = requirements['nominal_fields'] + ['source_transfer_permission', 'evidence_return_permission']
    return dict(nomination={n: unknown() for n in names},
        resources={n: dict(requested=unknown(), owner_capacity=unknown(), approved=unknown(), measured_peak=unknown(), scope=None, environment_id=None, purpose=None, approval_reference=None) for n in requirements['baseline']},
        permissions={n: dict(granted=None, authorizer=None, environment_id=None, purpose=None, phase=None, evidence_reference=None) for n in requirements['permissions']},
        trust={n: unknown() for n in ['executor_id', 'root_id', 'root_approver', 'approval_identity', 'signing_policy', 'verification_policy', 'freshness_policy', 'replay_policy']},
        toolchain_evidence=None, gate_claims=None, requested_phase='P1', execution_id=None, signature_id=None,
        lane_b=dict(paths=2403, runs=82, generators=41, byte_drift=0, expanded_tree_pass=False),
        execution_status='Not Executed', environment_qualified=False, positive_root_export=False)


def phase_files():
    files = {p.relative_to(R).as_posix(): V.record(p.relative_to(R)) for p in D.glob(P + '*') if p.is_file()}
    files['tools/review-r5g2r4p1a.py'] = V.record('tools/review-r5g2r4p1a.py')
    return files


def unchanged():
    for line in V.git('diff', '--name-status', START).decode().splitlines():
        status, name = line.split('\t')
        assert status == 'A' and (name.startswith('docs/implementation/' + P) or name == 'tools/review-r5g2r4p1a.py'), line
    assert V.sha(R / 'tools/run-production-input-graph-safe.py') == RUNNER_SHA
    frozen = read('target/' + P + 'old-inputs.json')
    assert all(V.record(name) == metadata for name, metadata in frozen.items())


def lane():
    r2 = 'phase-04c1b4c4r3c2r5g2r2-'
    maps = [read('target/' + r2 + 'B-' + n + '.json') for n in ['baseline', 'round1', 'round2', 'final']]
    assert len(maps[0]) == 2403 and all(v == maps[0] for v in maps)
    assert all(V.record(n) == row for n, row in maps[0].items())
    roster = read('target/' + r2 + 'B-generator-roster.json')
    assert len(roster) == 41 and sorted(v['path'] for v in roster) == sorted(p.relative_to(R).as_posix() for p in (R / 'tools').glob('gen-*.py'))
    assert all(V.record(row['path']) == {k: v for k, v in row.items() if k != 'path'} for row in roster)
    complete = read('target/' + r2 + 'B-complete.json')
    assert complete['commands'] == 82
    for round_ in complete['rounds']:
        for row in round_['commands']:
            assert row['exit'] == 0 and V.sha(Path(row['log'])) == row['log_sha256']
    write('lane-b-credit-preservation', dict(status='Inherited PASS — original protected scope only',
        generators=41, runs=82, paths=2403, raw_maps_equal=4, byte_drift=0, new_generator_executions=0,
        p0_added_files=sorted(read('target/' + P0 + 'final-runtime-results.json')['added_files']),
        p1a_added_files=sorted(phase_files()), affected_old_generator_inputs=[], affected_formal_source=[], affected_old_build_entrypoints=[],
        discovery_basis='Only additive prefixed approval audit documents/non-generator Python tool; old scripts, spec, conformance and frozen build trees unchanged',
        expanded_tree_double_round_credit=False,
        future='Expanded-tree input/non-interference adjudication still required before total acceptance; no generator rerun in P1A'))


def prepare():
    assert V.git('rev-parse', 'HEAD').decode().strip() == START
    assert not V.git('diff', START, '--name-only')
    ledger = read('target/' + P0 + 'final-runtime-results.json')
    assert ledger['final_head'] == START and ledger['static_results']['manifest_vectors'] == 105
    assert ledger['static_results']['r3_vectors']['preserved'] == 110
    frozen = read('target/' + P0 + 'old-evidence-freeze.json')
    frozen.update(ledger['added_files'])
    for review in ledger['reviews'].values():
        for name, row in review['reviewed_inputs'].items():
            assert V.record(name) == row
            frozen[name] = row
    for path in (R / 'target').glob(P0 + '*'):
        if path.is_file():
            frozen[path.relative_to(R).as_posix()] = V.record(path.relative_to(R))
    assert all(V.record(n) == row for n, row in frozen.items())
    # Existing accepted worktree raw bytes need not equal Git text bytes.
    # Preserve the COMPLETE prior raw universe, never normalize old files or
    # introduce a new general EOL equivalence. All2448 paths must be covered.
    baseline = read('target/phase-04c1b4c4r3c2r5g2r2-B-final.json')
    baseline.update(read('target/phase-04c1b4c4r3c2r5g2r3-final-runtime-results.json')['added_files'])
    baseline.update(ledger['added_files'])
    tracked = V.git('ls-tree', '-rz', '--name-only', START).decode().strip('\0').split('\0')
    assert set(tracked) == set(baseline) and len(tracked) == len(baseline) == 2448
    current = {n: V.record(n) for n in tracked}
    assert current == baseline
    write('start-source-map', current, True)
    write('old-inputs', frozen, True)
    failed = R / 'target' / (SAFE + 'r5g2r4p1a-input-review.json')
    if failed.exists():
        raw = json.loads(failed.read_bytes())
        assert raw['exit'] == 1 and raw['cancelled'] is None
        text = Path(raw['log']).read_text(encoding='utf8')
        assert 'phase-04c1b4a-comparator-mutations.md' in text and 'AssertionError' in text
        write('development-failure', dict(status='Failed', raw_receipt=raw,
            receipt_sha256=V.sha(failed), log_sha256=V.sha(Path(raw['log'])),
            reason='Invalid development precondition: compared existing worktree raw CRLF directly with Git LF; no asset drift, normalization or build executed',
            correction='Entire2448 prior raw map exactly matched; Git identity retained separately',
            no_old_file_changed=True), True)
    required = ['environment-approval-request.json', 'environment-candidate.json', 'environment-candidate-schema.json',
                'environment-equivalence-audit.json', 'resource-policy-analysis.json', 'evidence-manifest-schema.json',
                'historical-gate-contracts.json', 'frozen-g3-gate-contract.json', 'p1-qualification-runbook.md', 'p2-mandatory-execution-runbook.md']
    write('input-audit', dict(status='Static PASS', source_head=START, tracked_paths=len(current),
        p0_files={n: V.record('docs/implementation/' + P0 + n) for n in required},
        preserved_p0_105_and_r3_110=True, frozen_prior_evidence_files=len(frozen),
        worktree_raw_baseline='Full2448 path union of original Lane B and R3/P0 added-file maps; zero drift; not a fresh general EOL mapping',
        git_tree_reference=V.git('rev-parse', START + '^{tree}').decode().strip(),
        attachment=dict(path=str(ATTACHMENT), raw_sha256=V.sha(ATTACHMENT), role='Local approval-review instructions only; no concrete nomination/approval supplied'),
        historical_archive_file_rehash='Not Executed in P1A; pinned P0 raw maps/contract hashes retained, not claimed as new archive execution'))
    req = contract()
    value = empty_application(req)
    write('requirements-reference', req)
    write('application-input', value)
    write('environment-nomination', dict(record_type='EnvironmentNominationRecord', fields=value['nomination'],
        source_policy='Only explicit user-supplied candidate facts; no inference from host/network/config/history',
        candidate_nominated=False, status='P1 AUTHORIZATION INCOMPLETE'))
    write('authorization-scope-matrix', dict(scope='LOCAL APPROVAL REVIEW ONLY',
        permissions={name: dict(**row, effective_permission='DENIED', intended_domain=domain) for (name, domain), row in zip(req['permissions'].items(), value['permissions'].values())},
        P1='Not Authorized', P2='Not Authorized'))
    write('resource-approval-matrix', dict(resources={name: dict(prior_baseline=req['baseline'][name], **row) for name, row in value['resources'].items()},
        baseline_is_not_target_approval=True, P_build=None, J_build=None, old_14GiB_threshold_applies_to_new_environment=False,
        four_columns='User requested; owner-declared capacity; formally approved; future measured peak — all independent, currently null'))
    write('trust-and-executor-review', dict(fields=value['trust'], cryptographic_trust='Not Established', executor_authority='Not Established', environment='NOT QUALIFIED',
        independent_approved_roots=[], actual_signature_verification='Not Executed', private_keys_or_credentials_written=False,
        public_rfc_test_key_is_not_production_root=True))
    write('source-transport-review', dict(network_boundary=value['nomination']['network_access_policy'],
        source_transfer=value['permissions']['Source Transfer'], artifact_transfer=value['permissions']['Artifact Transfer'], evidence_return=value['permissions']['Evidence Return'],
        evidence_storage=value['nomination']['evidence_storage_policy'], effective_permission='DENIED', transports=0))
    histories = doc('historical-gate-contracts', P0)['groups']
    g3 = doc('frozen-g3-gate-contract', P0)['gate']
    rows = []
    for gate in [*histories, g3]:
        rows.append(dict(gate_id=gate['gate_id'], frozen_head=gate['frozen_head'], refs=gate['refs'], debug=gate['debug'],
            required_cases=gate['case_count'], source_identity=gate['frozen_source_manifest'], commands=gate['commands'],
            lockfile=gate['test_targets']['lockfile_identity'], static_target_census=gate['test_targets'],
            original_oracle=gate['original_contract']['OracleIdentity'],
            proposed_environment=None, proposed_toolchain=None, dependency_evidence=None,
            candidate_comparison='INCOMPLETE — no user candidate evidence', actual_equivalence='REQUIRES REAL P1 QUALIFICATION',
            environment_qualification=False, eligible_for_launch=False, launch_permission='DENIED', actual_execution='Not Executed'))
    write('toolchain-equivalence-review', dict(source='Exact P0 contracts, no new compatibility rules',
        p0_equivalence_audit=doc('environment-equivalence-audit', P0), gates=rows,
        proposed_evidence=None, status='REQUIRES REAL P1 QUALIFICATION; candidate absent', static_requirements_reviewed=True, candidate_equivalence_proved=False))
    write('historical-gate-eligibility', dict(gates=rows[:8], historical_builds='0/8', eligible=0, required=8, P2='Not Authorized'))
    write('frozen-g3-eligibility', dict(gate=rows[8], complete_executions=0, actual='Not Executed', P2='Not Authorized'))
    lane()
    print(json.dumps(dict(input_audit='Static PASS', tracked_paths=len(current), candidate_nominated=False, outcome='D')), flush=True)


def trust_context(synthetic=False):
    return dict(approvers=['SYNTHETIC-INDEPENDENT-APPROVER'] if synthetic else [],
                roots=['SYNTHETIC-ROOT-ONLY'] if synthetic else [],
                executors=['SYNTHETIC-EXECUTOR'] if synthetic else [],
                approval_references=['urn:synthetic:p1-approval-env'] if synthetic else [],
                old_execution_ids=['SYNTHETIC-OLD-EXECUTION'], old_signature_ids=['SYNTHETIC-OLD-SIGNATURE'],
                scope='PUBLICLY LABELLED SYNTHETIC TEST CONTEXT ONLY' if synthetic else 'No real user-supplied approvals or trust roots')


def synthetic_application(req):
    """Typed synthetic fixture, never used to fill the real nomination record."""
    value = empty_application(req)
    def make(schema):
        if 'enum' in schema:
            return next(x for x in schema['enum'] if x is not None)
        types = schema.get('type')
        kind = next(t for t in types if t != 'null') if isinstance(types, list) else types
        if kind == 'object':
            return {k: make(s) for k, s in schema['properties'].items()}
        if kind == 'array':
            return [make(schema['items'])]
        if kind == 'integer':
            return max(schema.get('minimum', 1), 1)
        if schema.get('pattern') == '^[0-9a-f]{64}$':
            return 'a' * 64
        return 'SYNTHETIC-DATA'
    def declaration(v, actor='SYNTHETIC-USER'):
        return dict(claimed_value=v, supplied_by=actor, evidence_reference='urn:synthetic:nomination-data',
                    verified_status='Declared — Not Verified', approval_status='Documented')
    schema = doc('environment-candidate-schema', P0)
    nominal = {n: make(schema['properties'][n]) for n in req['nominal_fields']}
    nominal.update(req['os_abi_requirements'])
    nominal.update(candidate_id='SYNTHETIC-ENV', environment_owner='SYNTHETIC-OWNER',
                   source_transfer_permission='SYNTHETIC-P1-ONLY', evidence_return_permission='SYNTHETIC-P1-ONLY')
    nominal.update(physical_memory=4 * req['baseline']['SystemReserve'], available_memory=3 * req['baseline']['SystemReserve'],
                   commit_limit=8 * req['baseline']['SystemReserve'], filesystem_capacity=128 * req['baseline']['SystemReserve'])
    nominal['trusted_executor_identity']['identity'] = 'SYNTHETIC-EXECUTOR'
    nominal['approved_trust_roots'] = [dict(key_id='SYNTHETIC-ROOT-ONLY', public_key_base64='NOT-A-REAL-KEY', approval_reference='urn:synthetic:p1-approval-env')]
    for name in value['nomination']:
        value['nomination'][name] = declaration(nominal[name])
    for name, row in value['resources'].items():
        # Fake 1/2-byte compile budgets only exercise binding logic; not a
        # recommended or activated real-host policy. Actual rows remain null.
        budget = req['baseline'][name] or (1 if name == 'P_build' else 2)
        row.update(requested=declaration(budget), owner_capacity=declaration(budget + 1, 'SYNTHETIC-OWNER'),
                   approved=declaration(budget, 'SYNTHETIC-INDEPENDENT-APPROVER'),
                   scope='build-only' if name in ['P_build', 'J_build'] else 'test-only' if name in ['P_test', 'J_test'] else 'system-protection',
                   environment_id='SYNTHETIC-ENV', purpose='SYNTHETIC bounded P1 qualification application', approval_reference='urn:synthetic:p1-approval-env')
    for name, row in value['permissions'].items():
        row.update(granted=req['permissions'][name] == 'P1', authorizer='SYNTHETIC-INDEPENDENT-APPROVER',
                   environment_id='SYNTHETIC-ENV', purpose='SYNTHETIC P1 scope only; no actual execution',
                   phase=req['permissions'][name], evidence_reference='urn:synthetic:p1-approval-env')
    for name in value['trust']:
        value['trust'][name] = declaration({'executor_id':'SYNTHETIC-EXECUTOR', 'root_id':'SYNTHETIC-ROOT-ONLY',
            'root_approver':'SYNTHETIC-INDEPENDENT-APPROVER', 'approval_identity':'urn:synthetic:p1-approval-env'}.get(name, 'SYNTHETIC-POLICY-ONLY'))
    value.update(toolchain_evidence=dict(requirement_digest=digest(req['gates']), changed_components=[], change_approval=unknown()),
                 gate_claims=copy.deepcopy(req['gates']), execution_id='SYNTHETIC-NEW-EXECUTION', signature_id='SYNTHETIC-NEW-SIGNATURE')
    return value


def tests():
    unchanged()
    req = doc('requirements-reference')
    oracle = module(D / (P + 'independent-static-oracle.py'), 'p1a_independent_review_oracle')
    vectors = doc('authorization-static-vectors')['vectors']
    actual, synthetic = doc('application-input'), synthetic_application(req)
    def locate(value, path):
        parts = path.split('.')
        for part in parts[:-1]:
            value = value[int(part)] if isinstance(value, list) else value[part]
        return value, int(parts[-1]) if isinstance(value, list) else parts[-1]
    rows = []
    for vector in vectors:
        is_actual = vector.get('base') == 'actual'
        value = copy.deepcopy(actual if is_actual else synthetic)
        for path in vector.get('delete', []):
            node, key = locate(value, path)
            del node[key]
        for source, destination in vector.get('copy', []):
            node, key = locate(value, source)
            other, target = locate(value, destination)
            other[target] = copy.deepcopy(node[key])
        for path, field in vector['edits']:
            node, key = locate(value, path)
            node[key] = field
        context = trust_context(not is_actual)
        production = evaluate(value, req, context)
        independent = oracle.evaluate(value, req, context, doc('environment-candidate-schema', P0))
        assert production['package_complete'] == independent['package_complete'] == vector['expected_complete'], (vector['id'], production, independent)
        if 'error' in vector:
            assert vector['error'] in production['errors'], (vector['id'], production)
        assert not any(production[k] for k in ['actual_environment_connected', 'actual_environment_qualified', 'actual_executor_attested', 'P1_authorized', 'P2_authorized', 'positive_root_export'])
        assert not any(independent[k] for k in ['actual_environment_connected', 'actual_environment_qualified', 'actual_executor_attested', 'P1_authorized', 'P2_authorized', 'positive_root_export'])
        rows.append(dict(id=vector['id'], expected_complete=vector['expected_complete'], actual_complete=production['package_complete'],
                         independent_complete=independent['package_complete'], errors=production['errors'], static_result='Static PASS', actual_execution_credit=0))
    assert len({r['id'] for r in rows}) == len(rows)
    write('authorization-static-results', dict(status='Static PASS', independent_static_oracle='Static PASS',
        tests=len(rows), passed=len(rows), rows=rows,
        expected_source='Separately authored authorization-static-vectors; frozen P0 schemas/contracts, never reviewer Actual',
        inherited_expected=dict(p0_105=dict(path='docs/implementation/' + P0 + 'static-test-results.json', **V.record('docs/implementation/' + P0 + 'static-test-results.json')),
            r3_110=dict(path='docs/implementation/phase-04c1b4c4r3c2r5g2r3-design-vector-results.json', **V.record('docs/implementation/phase-04c1b4c4r3c2r5g2r3-design-vector-results.json'))),
        inherited_vectors_rerun=False, new_real_environment=False, real_authority_issued=False))
    review_actual = evaluate(actual, req, trust_context())
    assert not review_actual['package_complete']
    first = ['Candidate Nominated', 'Environment Owner Identified', 'Execution Authority Documented', 'Environment Identity Complete',
             'Toolchain Evidence Complete', 'Resource Policy Approved', 'Transfer Scope Approved', 'Trust Roots Approved', 'Executor Identified', 'Environment Equivalence Reviewed']
    last = ['Real Environment Connected', 'Real Environment Qualified', 'Real Executor Attested', 'Historical Builds Executed', 'Frozen G3 Executed', 'P2 Authorized']
    write('p1-authorization-readiness', dict(title='CONCRETE ENVIRONMENT AUTHORIZATION REVIEW',
        outcome='D — Candidate/Approval Incomplete', status='P1 AUTHORIZATION INCOMPLETE', P0='ACCEPTED — NOT REVOKED',
        matrix={n: dict(value=False, reason='No explicit user candidate/approval/evidence' if n in first else 'Not Authorized / Not Executed in this local review') for n in first + last},
        static_requirements_reviewed=True, candidate_comparison_complete=False,
        actual_review=review_actual, real_P1='Not Authorized', P2='Not Authorized', historical_builds='0/8', frozen_g3='Not Executed',
        r5g2_registration='INCOMPLETE', next_input_required='Explicit nomination + owner/authority/provenance + separately scoped resource/transport/trust approvals',
        no_formal_binding_gap=True, no_actual_contract_contradiction_claimed=True))
    print(json.dumps(dict(status='Static PASS', vectors=len(rows), actual_outcome='D', actual_P1='Not Authorized')), flush=True)


def review(kind):
    unchanged()
    results = doc('authorization-static-results')
    readiness = doc('p1-authorization-readiness')
    assert results['status'] == results['independent_static_oracle'] == 'Static PASS'
    assert readiness['outcome'].startswith('D') and not any(row['value'] for row in readiness['matrix'].values())
    if kind == 'spec':
        old = read('target/phase-04c1b4c4r3c2r5g2r2-specification-review.json')
        assert [old['tuple'][n].split('@')[-1] for n in ['analysis', 'model', 'report', 'obligation']] == ['9', '8', '8', '8']
        assert len(old['exact_some_edges']['fields']) == 3
        assert all(d['item']['ref'] == 'NativeRelationSnapshotSource' and d['minimum'] == 0 for n, d in old['exact_some_edges']['fields'])
        req = doc('requirements-reference')
        assert [g['debug'] for g in req['gates']] == [2] * 7 + [0, 2]
        assert [g['case_count'] for g in req['gates'][:8]] == [512, 51, 173, 889, 262, 254, 534, 3662]
        assert req['gates'][8]['frozen_head'] == 'a9e296407c682a69f596be84ac8974eb870ebc18'
        checks = ['Frozen tuple/Some registration and old70 Spec/522 Rule objects retained through exact old-source continuity',
                  'P0 source contracts imported, no new compatibility/Registry/AFIDL/profile rules',
                  'Missing user data is Outcome D, not a binding impossibility or actual contradiction; P0 not revoked']
    elif kind == 'quality':
        source = (D / (P + 'independent-static-oracle.py')).read_text(encoding='utf8')
        assert 'review-r5g2r4p1a.py' not in source
        assert results['tests'] == results['passed'] and all(r['static_result'] == 'Static PASS' for r in results['rows'])
        assert any(r['id'] == 'synthetic-documentation-complete-only' and r['actual_complete'] for r in results['rows'])
        assert any(r['id'] == 'actual-no-candidate' and not r['actual_complete'] for r in results['rows'])
        assert any(r['id'] == 'executor-self-approves' and not r['independent_complete'] for r in results['rows'])
        checks = ['Independent P1A oracle does not call primary; uses separate frozen P0 independent schema interpreter',
                  'Hand-authored expected decisions, control positive and all required negative categories agree with both implementations',
                  'P0 105/R3 110 original Expected and receipts unchanged; no synthetic values enter real nomination/approval matrices']
    elif kind == 'environment':
        value = doc('application-input')
        assert all(row['claimed_value'] is None and row['supplied_by'] is None and row['evidence_reference'] is None for row in value['nomination'].values())
        assert all(row['granted'] is None for row in value['permissions'].values())
        assert all(all(row[column]['claimed_value'] is None for column in ['requested', 'owner_capacity', 'approved', 'measured_peak']) for row in value['resources'].values())
        assert not doc('resource-approval-matrix')['old_14GiB_threshold_applies_to_new_environment']
        assert readiness['actual_review']['cryptographic_trust'] == readiness['actual_review']['executor_authority'] == 'Not Established'
        checks = ['No real nomination/owner/toolchain/permission/root supplied; no local or other machine inference',
                  'Resource request/capacity/approval/measurement separated and all actual values null; old baseline not target approval',
                  'All real P1/P2 actions denied, executor trust unestablished; resource runner unchanged; local audit only']
    else:
        raise ValueError(kind)
    write('review-' + kind, dict(status='Static PASS', kind=kind,
        method='Independent static readback in separate protected process; same agent, not external human/environment certification',
        checks=checks, inputs=phase_files(), actual_decision='D — Candidate/Approval Incomplete'), True)
    print(json.dumps(dict(review=kind, status='Static PASS', actual_outcome='D')), flush=True)


def seal():
    assert not V.git('status', '--porcelain')
    unchanged()
    prior = read('target/' + P + 'start-source-map.json')
    assert all(V.record(n) == {k: v for k, v in row.items() if k != 'git_blob'} for n, row in prior.items())
    lane()
    assert not V.git('status', '--porcelain')
    reviews = {kind: read('target/' + P + 'review-' + kind + '.json') for kind in ['spec', 'quality', 'environment']}
    for item in reviews.values():
        assert item['status'] == 'Static PASS' and all(V.record(n) == row for n, row in item['inputs'].items())
    receipts = []
    for path in sorted((R / 'target').glob(SAFE + 'r5g2r4p1a-*.json')):
        row = json.loads(path.read_bytes())
        assert row['cancelled'] is None and row['limits_verified']
        if row['exit'] != 0:
            assert row['name'] == 'r5g2r4p1a-input-review' and row['exit'] == 1
            failure = read('target/' + P + 'development-failure.json')
            assert V.sha(path) == failure['receipt_sha256'] and V.sha(Path(row['log'])) == failure['log_sha256']
        assert (row['cpu_hard_cap_percent'], row['process_memory_mib'], row['whole_job_memory_mib']) == (5, 768, 1536)
        receipts.append(row)
    result = dict(title='CONCRETE ENVIRONMENT AUTHORIZATION REVIEW', outcome='D — Candidate/Approval Incomplete',
        status='P1 AUTHORIZATION INCOMPLETE', local_review_complete=True, P0='ACCEPTED — NOT REVOKED',
        start_head=START, final_head=V.git('rev-parse', 'HEAD').decode().strip(),
        commit_list=V.git('log', '--format=%H %s', START + '..HEAD').decode().splitlines(),
        readiness=doc('p1-authorization-readiness'), static_results=doc('authorization-static-results'), reviews=reviews,
        historical_builds='0/8', frozen_g3='Not Executed', P1='Not Authorized', P2='Not Authorized',
        environment='NOT QUALIFIED', cryptographic_trust='Not Established', executor_authority='Not Established',
        remote_connections=0, transports=0, builds=0, generator_executions=0, resource_amendments=0,
        old_tracked_paths_exact=len(prior), old_evidence_files_exact=len(read('target/' + P + 'old-inputs.json')),
        prior_P0_execution_states=read('target/' + P0 + 'final-runtime-results.json')['prior_execution_states'],
        resource_receipts=receipts, added_files=phase_files(), runner_sha256=RUNNER_SHA,
        development_failure_preserved=read('target/' + P + 'development-failure.json'),
        clean_worktree=True, pushed=False, final_post_job_check='Pending closeout; current seal Job has not yet exited')
    assert not (R / 'target' / (P + 'pending-final-seal.json')).exists()
    write('pending-final-seal', result, True)
    print(json.dumps(dict(outcome=result['outcome'], final_head=result['final_head'], status=result['status'])), flush=True)


def closeout():
    assert not V.LOCK.exists() and not V.git('status', '--porcelain')
    result = read('target/' + P + 'pending-final-seal.json')
    safety = read('target/' + SAFE + 'r5g2r4p1a-final-seal.json')
    assert safety['exit'] == 0 and safety['cancelled'] is None and safety['limits_verified']
    assert V.git('rev-parse', 'HEAD').decode().strip() == result['final_head']
    query = "@(Get-CimInstance Win32_Process -Filter \"Name = 'python.exe' OR Name = 'cargo.exe' OR Name = 'rustc.exe' OR Name LIKE 'af_portable%'\" | Where-Object { $_.ProcessId -ne " + str(V.os.getpid()) + " -and $_.CommandLine -match 'r5g2r4p1a' } | Select-Object ProcessId,ParentProcessId,Name) | ConvertTo-Json -Compress"
    raw = subprocess.check_output(['powershell', '-NoProfile', '-Command', query], cwd=R)
    assert not raw.strip() or json.loads(raw) == []
    result.update(final_resource_receipt=safety, pending_background_jobs=[], protected_lock_present=False,
                  final_post_job_check='Seal exited0/cancelled-null, lock absent, clean worktree, no other P1A processes. Tiny metadata-only closeout outside ended Job.')
    assert not (R / 'target' / (P + 'final-runtime-results.json')).exists()
    write('final-runtime-results', result, True)
    print(json.dumps(dict(outcome=result['outcome'], final_head=result['final_head'], clean=True, pushed=False, pending=[])), flush=True)


if __name__ == '__main__':
    if sys.argv[1:] == ['prepare']:
        prepare()
    elif sys.argv[1:] == ['test']:
        tests()
    elif sys.argv[1:] == ['lane']:
        lane()
    elif sys.argv[1:2] == ['review']:
        review(sys.argv[2])
    elif sys.argv[1:] == ['seal']:
        seal()
    elif sys.argv[1:] == ['closeout']:
        closeout()
    else:
        raise SystemExit('Only local prepare/test/lane/review/seal/closeout are implemented')
