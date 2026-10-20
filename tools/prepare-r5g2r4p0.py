"""P0 local preparation and read-only audits. Never invokes build/test/remote tools."""
import copy
import hashlib
import importlib.metadata
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tomllib

R = Path(__file__).resolve().parents[1]
D = R / 'docs/implementation'
P = 'phase-04c1b4c4r3c2r5g2r4p0-'
OLD = 'phase-04c1b4c4r3c2r5g2r3-'
START = '745efb585b9874f97f11473d0275b5e27ca638ee'
SAFE = 'phase-04c1b4c4r3c2r4g3-safe-'
RUNNER_SHA = '93672e9d71aaa7fb9a14acc66eeb10e4ef2ca424515fc9eb12fd48cef4daab07'


def module(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


V = module(R / 'tools/verify-r5g2r1.py', 'p0_readonly_helpers')


def read(path):
    return json.loads((R / path).read_bytes())


def document(name):
    return read('docs/implementation/' + P + name + '.json')


def write(name, value, target=False):
    path = (R / 'target' if target else D) / (P + name + '.json')
    path.write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode('utf8'))
    return path


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def obj(fields, nullable=False):
    return dict(type=['object', 'null'] if nullable else 'object',
                properties=fields, required=list(fields), additionalProperties=False)


TEXT = dict(type=['string', 'null'], minLength=1, maxLength=512)
HASH = dict(type='string', pattern='^[0-9a-f]{64}$')
HEAD = dict(type='string', pattern='^[0-9a-f]{40}$')
INT = dict(type=['integer', 'null'], minimum=0, maximum=2 ** 63 - 1)
IDENTITY = obj(dict(identity=TEXT, version=TEXT, sha256=dict(type=['string', 'null'], pattern='^[0-9a-f]{64}$')), True)
FILE = obj(dict(path=dict(type='string', minLength=1, maxLength=512), byte_length=dict(type='integer', minimum=0), sha256=HASH))
POLICY = dict(P_build=None, J_build=None, P_test=805306368, J_test=1610612736,
              SystemReserve=8589934592, cpu_percent=5, top_level_jobs=1,
              test_threads=1, cargo_jobs=1, priority='BelowNormal', admission_threshold=None)
CANDIDATE_FIELDS = '''candidate_id environment_owner approval_reference execution_location execution_authority host_or_vm_identity operating_system os_build architecture target_abi rust_toolchain cargo_version rustc_version llvm_version msvc_toolset windows_sdk python_runtime dependency_sources dependency_lockfiles offline_cache_identity cpu_topology physical_memory available_memory commit_limit filesystem_capacity build_resource_policy test_resource_policy foreground_reserve_policy network_access_policy data_transfer_policy evidence_storage_policy trusted_executor_identity trusted_manifest_verifier approved_trust_roots qualification_status'''.split()


def schemas():
    fields = {n: copy.deepcopy(TEXT) for n in CANDIDATE_FIELDS}
    for name in ['host_or_vm_identity', 'rust_toolchain', 'msvc_toolset', 'windows_sdk',
                 'python_runtime', 'offline_cache_identity', 'trusted_executor_identity', 'trusted_manifest_verifier']:
        fields[name] = copy.deepcopy(IDENTITY)
    for name in ['dependency_sources', 'dependency_lockfiles']:
        fields[name] = dict(type=['array', 'null'], items=FILE, maxItems=10000, uniqueItems=True)
    fields['cpu_topology'] = obj(dict(logical_cpus=dict(type='integer', minimum=1), physical_cores=dict(type='integer', minimum=1)), True)
    for name in ['physical_memory', 'available_memory', 'commit_limit', 'filesystem_capacity']:
        fields[name] = copy.deepcopy(INT)
    fields['build_resource_policy'] = obj(dict(P_build=INT, J_build=INT, approval_reference=TEXT), True)
    fields['test_resource_policy'] = obj(dict(P_test=INT, J_test=INT, approval_reference=TEXT), True)
    fields['foreground_reserve_policy'] = obj(dict(SystemReserve=INT, admission_threshold=INT, approval_reference=TEXT), True)
    for name in ['network_access_policy', 'data_transfer_policy', 'evidence_storage_policy']:
        fields[name] = obj(dict(policy_id=TEXT, approval_reference=TEXT, sha256=dict(type=['string', 'null'], pattern='^[0-9a-f]{64}$')), True)
    fields['approved_trust_roots'] = dict(type=['array', 'null'], maxItems=32,
        items=obj(dict(key_id=dict(type='string'), public_key_base64=dict(type='string'), approval_reference=TEXT)))
    fields['qualification_status'] = dict(enum=['Not Verified', 'NOT QUALIFIED'])
    candidate = dict(**obj(fields), **{'$schema': 'https://json-schema.org/draft/2020-12/schema',
        '$id': 'urn:agentflow:verification-preparation:candidate:p0:v1',
        'title': 'VerificationEnvironmentCandidate — phase-local schema, not AFIDL/Registry registration'})
    write('environment-candidate-schema', candidate)
    ref = obj(dict(owner_head=HEAD, sha256=HASH))
    argv = dict(type='array', items=dict(type='string'), maxItems=128)
    commands = obj(dict(cargo=argv, cwd=dict(type='string'),
        env=obj({n: dict(type='string') for n in ['CARGO_PROFILE_DEV_DEBUG', 'CARGO_PROFILE_TEST_DEBUG', 'CARGO_BUILD_JOBS', 'RUST_TEST_THREADS']}),
        independent_checks=dict(type='array', items=argv), identity_checks=dict(type='string'), target_relocation=dict(type='string')))
    record = obj(dict(case_id=dict(type='string'), fields=obj(dict(verdict=dict(type='string'), proof_hash=HASH))))
    records = dict(type=['array', 'null'], items=record, maxItems=10000, uniqueItems=True)
    artifact = obj(dict(path=dict(type='string'), owner_head=HEAD, sha256=HASH, byte_length=dict(type='integer', minimum=0)))
    gate = obj(dict(gate_id=dict(type='string'), environment_id=TEXT, frozen_head=HEAD,
        debug=dict(type='integer', enum=[0, 2]), refs=obj({k: ref for k in ['source', 'registry', 'compiler', 'expected', 'oracle']}),
        case_count=dict(anyOf=[dict(type='integer', minimum=1), obj(dict(positive=dict(type='integer'), negative_and_resource_nominal_boundaries=dict(type='integer'), comparator_mutations=dict(type='integer')))]),
        commands=commands, closure=obj({k: dict(type='boolean') for k in ['all_targets', 'build_scripts', 'proc_macros', 'doc_tests']}),
        case_ids=dict(type=['array', 'null'], items=dict(type='string'), uniqueItems=True), expected=records, actual=records,
        artifacts=dict(type=['array', 'null'], items=artifact, uniqueItems=True, maxItems=10000), artifact_owner=HEAD))
    statuses = ['Not Executed', 'Failed', 'Interrupted By Resource Safety', 'PASS']
    manifest = obj(dict(schema_version=dict(const='R4P0/evidence@1'), scope=dict(enum=['Preparation', 'Synthetic']),
        candidate={'$ref': '#/$defs/candidate'}, environment_id=TEXT,
        resource_policy=obj({k: (dict(type=['integer', 'null'], minimum=0) if k != 'priority' else dict(type='string')) for k in POLICY}),
        gates=dict(type='array', items=gate, minItems=1, maxItems=9),
        lane_b=obj(dict(generators=dict(type='integer'), rounds=dict(type='integer'), executions=dict(type='integer'), original_paths=dict(type='integer'), byte_drift=dict(type='integer'), expanded_tree_pass=dict(type='boolean'))),
        execution_id=TEXT, execution_status=dict(enum=statuses), claimed_status=dict(enum=statuses),
        execution_window=obj({k: INT for k in ['start_utc_ns', 'end_utc_ns', 'start_monotonic_ns', 'end_monotonic_ns']}),
        actual_exit_code=dict(type=['integer', 'null']), cancelled=TEXT,
        trusted_status=dict(enum=['Not Established', 'Trusted']), positive_authority=dict(type='boolean'),
        capability_denial=obj({k: dict(type='string') for k in ['root_export', 'portable', 'complete_view', 'obligation_identity']}),
        signature=obj(dict(algorithm=dict(enum=[None, 'Ed25519']), key_id=TEXT,
                           payload_sha256=dict(type=['string', 'null'], pattern='^[0-9a-f]{64}$'), signature_base64=TEXT))))
    manifest.update({'$schema': candidate['$schema'], '$id': 'urn:agentflow:verification-preparation:evidence:p0:v1',
                     '$defs': {'candidate': {k: v for k, v in candidate.items() if not k.startswith('$') and k != 'title'}},
                     'title': 'Offline P0 static evidence; never real execution qualification'})
    write('evidence-manifest-schema', manifest)
    return fields


def frozen_inputs():
    assert V.git('rev-parse', 'HEAD').decode().strip() == START
    assert not V.git('diff', START, '--name-only')
    assert V.sha(R / 'tools/run-production-input-graph-safe.py') == RUNNER_SHA
    ledger = read('target/' + OLD + 'final-runtime-results.json')
    assert ledger['final_head'] == START and ledger['historical_builds'] == '0/8'
    assert ledger['final_safety_receipt']['exit'] == 0
    assert all(V.record(n) == row for n, row in ledger['added_files'].items())
    for report in ledger['reviews'].values():
        assert all(V.record(n) == row for n, row in report['reviewed_input_hashes'].items())
    prior = read('target/' + OLD + 'prior-evidence-integrity.json')
    assert all(V.record(n) == row for n, row in prior.items())
    paths = ['docs/implementation/' + OLD + n for n in [
        'final-report.md', 'verification-contract-inventory.json', 'environment-equivalence-contract.md',
        'cross-environment-evidence-manifest.json', 'historical-execution-plan.json', 'lane-b-credit-preservation.json', 'design-vector-results.json']]
    paths.append('target/' + OLD + 'final-runtime-results.json')
    records = {n: dict(**V.record(n), source_head=START, origin='R3 committed artifact' if n.startswith('docs/') else 'External post-commit R3 ledger; source head, not a Git blob') for n in paths}
    result = dict(status='Static PASS', start_head=START, inputs=records,
                  r3_added_paths_exact=len(ledger['added_files']), prior_sealed_evidence_exact=len(prior),
                  formal_tuple=dict(analysis='agentflow.root-export-checker@9', model='agentflow.computation.proof-report.model@8', report='agentflow.g01.report-shape@8', obligation='agentflow.g01.obligation@8'),
                  inherited=dict(cargo_records=448, registration_cases=2880, field_variant_paths=1297, comparator_mutations=16),
                  historical_builds='0/8', frozen_g3_complete=0, actual_environment='Not Qualified',
                  P1='Not Authorized', P2='Not Authorized', source_bytes_unchanged=True)
    write('qualification-inputs', result)
    write('old-evidence-freeze', {**prior, **{n: V.record(n) for n in paths}}, True)
    print(json.dumps(dict(input_audit='Static PASS', old_evidence=len(prior), r3_files=len(ledger['added_files']))), flush=True)


def contracts():
    inv = read('docs/implementation/' + OLD + 'verification-contract-inventory.json')['gates'][:9]
    out = []
    for gate in inv:
        source = gate['SourceIdentity']
        frozen = read(source['full_map'])
        assert V.sha(R / source['full_map']) == source['full_map_file_sha256'] and digest(frozen) == source['complete_raw_manifest_sha256']
        # One streamed pass per frozen source; no build directories are executed.
        assert V.blobmap(gate['FrozenSourceHEAD'], Path(gate['CommandAndFlags']['cwd'])) == frozen
        print(json.dumps(dict(frozen_source=gate['GateId'], paths=len(frozen))), flush=True)
        refs = {}
        for key, title in [('source', 'SourceIdentity'), ('registry', 'RegistryIdentity'), ('compiler', 'CompilerIdentity'), ('expected', 'ExpectedIdentity'), ('oracle', 'OracleIdentity')]:
            refs[key] = dict(owner_head=gate['FrozenSourceHEAD'], sha256=gate[title].get('manifest_sha256', source['complete_raw_manifest_sha256']))
        case_ids = ['synthetic/' + gate['GateId'] + '/' + letter for letter in ['a', 'b']]
        records = [dict(case_id=c, fields=dict(verdict='SYNTHETIC_ONLY', proof_hash='e' * 64)) for c in case_ids]
        artifacts = [dict(path=folder + '/' + gate['GateId'] + suffix, owner_head=gate['FrozenSourceHEAD'], sha256=char * 64, byte_length=1)
                     for folder, suffix, char in [('bin', '.exe', 'a'), ('symbols', '.pdb', 'b')]]
        row = dict(gate_id=gate['GateId'], frozen_head=gate['FrozenSourceHEAD'], refs=refs,
                   debug=gate['BuildProfile']['dev_debug'], case_count=gate['RequiredCases']['declared_case_ids'] or gate['RequiredCases']['frozen_g3_classes'],
                   commands=gate['CommandAndFlags'], synthetic_case_ids=case_ids, synthetic_records=records, synthetic_artifacts=artifacts,
                   frozen_source_manifest=source, original_contract=gate,
                   test_targets=dict(status='Requires Qualification', source='Exact frozen Cargo.toml workspace/package/test declarations; autotests and integration/doc tests, build scripts, proc macros, all features/targets as originally selected'),
                   doctest_coverage=dict(required=True, execution='Not Executed in P0', separation_approved=False),
                   required_artifacts=gate['RequiredArtifacts'], real_case_roster='Requires Qualification — full trusted roster, not these two synthetic cases',
                   execution='Not Executed in P0')
        out.append(row)
    assert [g['debug'] for g in out] == [2] * 7 + [0, 2]
    write('historical-gate-contracts', dict(status='8/8 SPECIFIED', execution='Not Executed in P0', groups=out[:8], historical_builds='0/8'))
    write('frozen-g3-gate-contract', dict(status='COMPLETE', execution='Not Executed in P0', gate=out[8]))
    contract = dict(schema='R4P0/trusted-static-contract@1', source_head=START,
        scope='P0 static requirements and synthetic fixtures only; must be independently approved/pinned before any future real validator use',
        gates=out, resource_policy=POLICY, previous_execution_ids=['synthetic-old-1'],
        environment_bindings={'SYNTHETIC-ID-TAKEN': {'environment_owner': 'independent-fixture-owner'}},
        lane_b=dict(generators=41, rounds=2, executions=82, original_paths=2403, byte_drift=0, expanded_tree_pass=False),
        capability_denial=dict(root_export='NotAuthorized', portable='NotAuthorized', complete_view='NotIssued', obligation_identity='NotIssued'))
    write('validation-contract', contract)
    candidate = {n: None for n in CANDIDATE_FIELDS}
    candidate['qualification_status'] = 'Not Verified'
    write('environment-candidate', candidate)
    def manifest(scope):
        gates = [dict(gate_id=g['gate_id'], environment_id=None, frozen_head=g['frozen_head'], debug=g['debug'], refs=g['refs'], case_count=g['case_count'],
            commands=g['commands'], closure=dict(all_targets=True, build_scripts=True, proc_macros=True, doc_tests=True),
            case_ids=g['synthetic_case_ids'] if scope == 'Synthetic' else None,
            expected=copy.deepcopy(g['synthetic_records']) if scope == 'Synthetic' else None,
            actual=copy.deepcopy(g['synthetic_records']) if scope == 'Synthetic' else None,
            artifacts=g['synthetic_artifacts'] if scope == 'Synthetic' else None, artifact_owner=g['frozen_head']) for g in out]
        return dict(schema_version='R4P0/evidence@1', scope=scope, candidate=candidate, environment_id=None,
            resource_policy=POLICY, gates=gates, lane_b=contract['lane_b'], execution_id=None,
            execution_status='Not Executed', claimed_status='Not Executed', actual_exit_code=None, cancelled=None,
            execution_window={k: None for k in ['start_utc_ns', 'end_utc_ns', 'start_monotonic_ns', 'end_monotonic_ns']},
            trusted_status='Not Established', positive_authority=False, capability_denial=contract['capability_denial'],
            signature=dict(algorithm=None, key_id=None, payload_sha256=None, signature_base64=None))
    write('synthetic-manifest', manifest('Synthetic'))
    write('preparation-manifest', manifest('Preparation'))


def policies():
    analysis = dict(status='EXPLICIT / PENDING APPROVAL', current_baseline=POLICY,
        P_build=None, J_build=None, target_admission_threshold=None,
        old_host_only=dict(threshold_bytes=15032385536, observed_duration_seconds=600.0005618, samples=121, admitted=False),
        old_threshold_is_not_new_environment_norm=True,
        frozen_g3_oom=dict(cargo_exit=101, debug=2, process_peak_bytes=805244928, process_cap_bytes=805306368,
            conclusion='Highly quota-consistent, not full causal proof or measurement of required peak', missing='Failed allocation/OS error/Job notifications/commit details'),
        two_budgets=['System physical/commit/reserve/host load and sustained admission', 'Compiler process and Job committed memory budget'],
        compile_policy='P_build/J_build need evidence and separate bounded qualification-pilot approval; no arbitrary higher value proposed or applied',
        runtime_policy='P_test/J_test/reserve retained baseline; compilation amendment never changes D0/D4/Native/Portable/RootExport product constraints',
        change_applied=False, runner_sha256=RUNNER_SHA)
    write('resource-policy-analysis', analysis)
    fields = '''target_environment_identifier owner execution_authority windows architecture toolchain source_transport_permission evidence_return_permission approved_network_boundary cpu_policy P_build J_build P_test J_test SystemReserve allowed_historical_gates allowed_frozen_g3_gate allowed_execution_time_window approved_evidence_trust_roots authorized_executor approval_identity approval_scope'''.split()
    request = {n: None for n in fields}
    request.update(P_test=805306368, J_test=1610612736, SystemReserve=8589934592)
    write('environment-approval-request', dict(status='REQUIRED USER AUTHORIZATION', requested_values=request,
        existing_baseline_not_target_approval=POLICY,
        permissions={k: 'DENIED' for k in ['Environment Connection', 'Source Transfer', 'Remote Execution', 'Build Resource Amendment', 'Historical Build Launch', 'Frozen G3 Build Launch']},
        P0='Authorized preparation only', P1='Not Authorized', P2='Not Authorized'))
    kinds = [
        ('OS/Windows x64 MSVC ABI', 'Requires Qualification'),
        ('Rust/Cargo/LLVM/MSVC/SDK/Python exact identity', 'Requires Qualification'),
        ('Historical debug 2/2/2/2/2/2/2/0 and G3 debug2', 'Proved By Existing Contract'),
        ('Feature/target/build script/proc macro/doc test execution closure', 'Requires Qualification'),
        ('Cargo.lock tracked bytes', 'Proved By Static Evidence'),
        ('Dependency source/cache full closure', 'Requires Qualification'),
        ('Working directory and path relocation', 'Requires User Approval'),
        ('Original per-path LF/CRLF correspondence', 'Proved By Existing Contract'),
        ('Build artifact/source/toolchain binding', 'Requires Qualification'),
        ('Exact actual test/CaseId/Expected complete roster', 'Requires Qualification'),
        ('Resource policies and executor identity', 'Requires User Approval'),
        ('Frozen source GitBlob/raw identity at P0 audit', 'Proved By Static Evidence'),
        ('Post-execution intact source and exits/Actual/independent Oracle', 'Requires Qualification'),
        ('General EOL normalization or cross-Edition artifact sharing', 'Unsupported')]
    write('environment-equivalence-audit', dict(status='STATIC CLASSIFICATION COMPLETE', real_environment='NOT QUALIFIED',
        rows=[dict(requirement=n, status=s, source_head=START,
                   evidence='R3 frozen contract and raw manifests; see qualification-inputs/historical/G3 contracts',
                   limitation='Static source identity is not live environment equivalence; missing approvals are not PASS') for n, s in kinds]))


def lane_b():
    r2 = 'phase-04c1b4c4r3c2r5g2r2-'
    maps = [read('target/' + r2 + 'B-' + n + '.json') for n in ['baseline', 'round1', 'round2', 'final']]
    assert len(maps[0]) == 2403 and all(m == maps[0] for m in maps)
    assert all(V.record(n) == row for n, row in maps[0].items())
    roster = read('target/' + r2 + 'B-generator-roster.json')
    assert len(roster) == 41 and sorted(row['path'] for row in roster) == sorted(p.relative_to(R).as_posix() for p in (R / 'tools').glob('gen-*.py'))
    assert all(V.record(row['path']) == {k: v for k, v in row.items() if k != 'path'} for row in roster)
    receipt = read('target/' + r2 + 'B-complete.json')
    assert receipt['commands'] == 82 and receipt['source_paths'] == 2403
    for round_ in receipt['rounds']:
        for row in round_['commands']:
            assert row['exit'] == 0 and V.sha(Path(row['log'])) == row['log_sha256']
    write('lane-b-credit-audit', dict(status='Inherited PASS — original protected scope only',
        prior_protected_scope=dict(paths=2403, generators=41, rounds=2, actual_executions=82, raw_maps_equal=4, byte_drift=0),
        original_map_hashes=[V.record('target/' + r2 + 'B-' + n + '.json') for n in ['baseline', 'round1', 'round2', 'final']],
        new_p0_files=sorted(phase_files()),
        affected_build_inputs=[], affected_generator_inputs=[], affected_formal_source=[],
        scope_basis='Old protected paths and exact historical/frozen source trees only. Expanded root is a distinct unexecuted input universe.',
        expanded_tree_double_round_credit=False, generators_executed_in_P0=0,
        later_acceptance='Independent extension/non-interference adjudication mandatory before total acceptance; no automatic expanded-tree credit'))


def prepare():
    frozen_inputs()
    schemas()
    contracts()
    policies()
    lane_b()
    # Audit dependency identities refer to this LOCAL static tool, not to any
    # proposed target environment or approved production trust root.
    rows = {}
    for name in ['jsonschema', 'cryptography', 'referencing']:
        spec = importlib.util.find_spec(name)
        rows[name] = dict(version=importlib.metadata.version(name), entry=spec.origin,
                          raw_sha256=V.sha(Path(spec.origin)))
    write('local-audit-dependencies', dict(scope='Local P0 audit runtime only; target remains unknown', python_version=sys.version,
          python_executable=dict(path=sys.executable, raw_sha256=V.sha(Path(sys.executable))), libraries=rows))
    print(json.dumps(dict(preparation='Static PASS', historical_contracts=8, frozen_g3=1, actual_builds=0, P1='Not Authorized', P2='Not Authorized')), flush=True)


def tooling():
    return (module(D / (P + 'offline-manifest-validator.py'), 'p0_validator'),
            module(D / (P + 'independent-static-oracle.py'), 'p0_independent_oracle'))


def refresh_schemas():
    schemas()
    for name in ['synthetic-manifest', 'preparation-manifest']:
        value = document(name)
        value.setdefault('execution_window', {k: None for k in ['start_utc_ns', 'end_utc_ns', 'start_monotonic_ns', 'end_monotonic_ns']})
        write(name, value)


def targets():
    """Read frozen TOML/source metadata only; cargo metadata/test --list forbidden."""
    contract = document('validation-contract')
    for gate in contract['gates']:
        root = Path(gate['commands']['cwd'])
        source_map = read(gate['frozen_source_manifest']['full_map'])
        workspace = tomllib.loads((root / 'Cargo.toml').read_text(encoding='utf8'))['workspace']
        packages = []
        for member in workspace['members']:
            assert not any(c in member for c in '*?[')
            relative = member + '/Cargo.toml'
            data = tomllib.loads((root / relative).read_text(encoding='utf8'))
            package = data['package']
            files = [n for n in source_map if n.startswith(member + '/')]
            auto_tests = [n for n in files if n.startswith(member + '/tests/') and n.endswith('.rs')]
            declarations = {k: data.get(k, []) for k in ['bin', 'test', 'example', 'bench']}
            lib = data.get('lib', {})
            lib_path = member + '/' + lib.get('path', 'src/lib.rs')
            packages.append(dict(package=package['name'], manifest=relative, manifest_sha256=source_map[relative]['raw_sha256'],
                features=data.get('features', {}), explicit_declarations=declarations,
                automatic_discovery={n: package.get(n, True) for n in ['autobins', 'autotests', 'autoexamples', 'autobenches']},
                tracked_rust_test_sources=auto_tests,
                auto_discovered_tests_selected=package.get('autotests', True),
                lib_source=lib_path if lib_path in source_map else None,
                doctests_required=(lib_path in source_map and lib.get('doctest', True)),
                build_script=dict(declaration=package.get('build', 'build.rs default if present'),
                    tracked_sources=[n for n in files if n == member + '/build.rs']),
                proc_macro=lib.get('proc-macro', False),
                dependencies=data.get('dependencies', {}), build_dependencies=data.get('build-dependencies', {}),
                target_specific_dependencies=data.get('target', {})))
        gate['test_targets'] = dict(status='Proved By Static Evidence — declaration/source census only',
            workspace_members=workspace['members'], packages=packages,
            lockfile_identity=source_map.get('Cargo.lock'), root_manifest_identity=source_map['Cargo.toml'],
            runtime_roster='Requires Qualification: cfg/feature-expanded Rust test IDs and doc snippets; no --list or build run in P0',
            required_selection='Unchanged original complete Cargo command; explicit tests/autotests flags preserved. Census does not assert every bench/example is executed.')
    write('validation-contract', contract)
    histories = document('historical-gate-contracts')
    histories['groups'] = contract['gates'][:8]
    write('historical-gate-contracts', histories)
    g3 = document('frozen-g3-gate-contract')
    g3['gate'] = contract['gates'][8]
    write('frozen-g3-gate-contract', g3)
    print(json.dumps(dict(static_frozen_target_censuses=9, cargo_invocations=0)), flush=True)


def unchanged():
    for line in V.git('diff', '--name-status', START).decode().splitlines():
        status, name = line.split('\t')
        assert status == 'A' and (name.startswith('docs/implementation/' + P) or name == 'tools/prepare-r5g2r4p0.py'), line
    frozen = read('target/' + P + 'old-evidence-freeze.json')
    assert all(V.record(n) == row for n, row in frozen.items())
    assert V.sha(R / 'tools/run-production-input-graph-safe.py') == RUNNER_SHA


def protect_maps():
    frozen = read('target/' + P + 'old-evidence-freeze.json')
    for gate in document('validation-contract')['gates']:
        source = gate['frozen_source_manifest']
        assert V.sha(R / source['full_map']) == source['full_map_file_sha256']
        frozen[source['full_map']] = V.record(source['full_map'])
    for name in ['prior-evidence-integrity', 'pending-final-seal']:
        path = 'target/' + OLD + name + '.json'
        frozen[path] = V.record(path)
    write('old-evidence-freeze', frozen, True)


def tests():
    unchanged()
    validator, oracle = tooling()
    contract, schema = document('validation-contract'), document('evidence-manifest-schema')
    vector_file = document('static-test-vectors')
    bases = {scope: document(name) for scope, name in [('Preparation', 'preparation-manifest'), ('Synthetic', 'synthetic-manifest')]}
    def parent(value, path):
        fields = path.split('.')
        for key in fields[:-1]:
            value = value[int(key)] if isinstance(value, list) else value[key]
        return value, int(fields[-1]) if isinstance(value, list) else fields[-1]
    cases = copy.deepcopy(vector_file['vectors'])
    for field in schema['required']:
        cases.append(dict(id='omit-root/' + field, edits=[], delete=[field], valid=False, error='SCHEMA'))
    for field in document('environment-candidate-schema')['required']:
        cases.append(dict(id='omit-candidate/' + field, edits=[], delete=['candidate.' + field], valid=False, error='SCHEMA'))
    rows = []
    for case in cases:
        value = copy.deepcopy(bases[case.get('base', 'Synthetic')])
        for path in case.get('delete', []):
            node, field = parent(value, path)
            del node[field]
        for source, destination in case.get('copy', []):
            a, ka = parent(value, source)
            b, kb = parent(value, destination)
            b[kb] = copy.deepcopy(a[ka])
        for path, new_value in case['edits']:
            node, field = parent(value, path)
            node[field] = new_value
        actual = validator.validate(value, contract, schema)
        independent = oracle.evaluate(value, contract, schema)
        accepted = actual['static_status'] == 'Static PASS'
        assert accepted == independent['valid'] == case['valid'], (case['id'], actual, independent, case)
        if 'error' in case:
            assert case['error'] in actual['errors'], (case['id'], actual)
        for blocker in case.get('blockers', []):
            assert blocker in actual['qualification_blockers'] and blocker in independent['qualification_blockers'], (case['id'], blocker)
        assert actual['environment_status'] == 'NOT QUALIFIED' and actual['execution_authorization'] == 'Not Authorized'
        assert actual['cryptographic_trust'] == 'Not Established' and not actual['positive_authority'] and not independent['authority']
        rows.append(dict(id=case['id'], expected_static_acceptance=case['valid'], validator_static_acceptance=accepted,
                         independent_oracle_acceptance=independent['valid'], expected_error=case.get('error'),
                         actual_errors=actual['errors'], qualification_blockers=actual['qualification_blockers'],
                         static_result='Static PASS', real_execution='Not Executed', authority=False))
    assert len({r['id'] for r in rows}) == len(rows)
    primitive = vector_file['crypto_public_vectors']
    pub, sig, message = [bytes.fromhex(primitive[k]) for k in ['public_key_hex', 'signature_hex', 'message_hex']]
    public_tests = [dict(id='RFC8032-public-valid', expected=True, actual=validator.verify_public_signature(message, sig, pub)),
                    dict(id='RFC8032-corrupted', expected=False, actual=validator.verify_public_signature(message, bytes([sig[0] ^ 1]) + sig[1:], pub)),
                    dict(id='RFC8032-changed-message', expected=False, actual=validator.verify_public_signature(b'\0', sig, pub)),
                    dict(id='RFC8032-short-key', expected=False, actual=validator.verify_public_signature(message, sig, pub[:-1])),
                    dict(id='RFC8032-short-signature', expected=False, actual=validator.verify_public_signature(message, sig[:-1], pub))]
    assert all(r['expected'] == r['actual'] for r in public_tests)
    raw_cases = [b'{"a":1,"a":2}', b'{"a":NaN}', b'{"a":1.0}', b'[' * 33 + b']' * 33,
                 b'{"a":9223372036854775808}', b'{"a":"\\u0000"}']
    for raw in raw_cases:
        try:
            validator.loads(raw)
        except (ValueError, RecursionError):
            pass
        else:
            raise AssertionError(('Parser accepted unsafe input', raw))
    source = 'docs/implementation/' + OLD + 'final-report.md'
    metadata = V.record(source)
    ref = dict(path=source, byte_length=metadata['byte_length'], sha256=metadata['raw_sha256'])
    assert validator.verify_file_reference(R, ref)
    assert not validator.verify_file_reference(R, dict(ref, sha256='0' * 64))
    for unsafe in ['../outside', '/absolute', 'C:/outside', 'docs\\outside', 'docs//outside']:
        try:
            validator.verify_file_reference(R, dict(ref, path=unsafe))
        except ValueError:
            pass
        else:
            raise AssertionError('Unsafe identity path accepted: ' + unsafe)
    original = read('docs/implementation/' + OLD + 'design-vector-results.json')
    r3 = module(R / 'tools/evidence-r5g2r3.py', 'p0_frozen_r3_vectors').vectors()
    assert original == r3 and r3['total'] == 110
    result = dict(status='Static PASS', independent_static_oracle='Static PASS',
        manifest_vectors=len(rows), passed=len(rows), cases=rows,
        public_signature_vectors=public_tests, public_key_scope='Published public RFC test vector only; never a trusted executor/root; no private keys or key generation',
        strict_parser_negative_vectors=len(raw_cases), local_file_identity_vectors=7,
        r3_vectors=dict(preserved=110, replayed_static_pass=110,
                       original_expected_file='docs/implementation/' + OLD + 'design-vector-results.json', original_expected_sha256=V.sha(D / (OLD + 'design-vector-results.json'))),
        independent_expected_source='Separately authored static-test-vectors.json and fixed frozen contracts, not validator Actual',
        concrete_environment='NOT QUALIFIED', executor_authority='Not Established', cryptographic_trust='Not Established',
        P1='Not Authorized', P2='Not Authorized', real_environment='Not Executed', historical_builds='0/8', frozen_g3='Not Executed',
        production_execution_credit=0, positive_authority=False)
    write('static-test-results', result)
    print(json.dumps({k: result[k] for k in ['status', 'manifest_vectors', 'passed', 'historical_builds']}), flush=True)


def phase_files():
    files = {}
    for item in D.glob(P + '*'):
        for path in ([item] if item.is_file() else item.rglob('*')):
            if path.is_file():
                files[path.relative_to(R).as_posix()] = V.record(path.relative_to(R))
    files['tools/prepare-r5g2r4p0.py'] = V.record('tools/prepare-r5g2r4p0.py')
    return files


def review(kind):
    unchanged()
    contract, results = document('validation-contract'), document('static-test-results')
    assert results['status'] == results['independent_static_oracle'] == 'Static PASS'
    assert len(contract['gates']) == 9 and [g['debug'] for g in contract['gates']] == [2] * 7 + [0, 2]
    assert [g['case_count'] for g in contract['gates'][:8]] == [512, 51, 173, 889, 262, 254, 534, 3662]
    assert contract['gates'][8]['case_count'] == dict(positive=154, negative_and_resource_nominal_boundaries=26, comparator_mutations=59)
    if kind == 'specification':
        prior = read('target/phase-04c1b4c4r3c2r5g2r2-specification-review.json')
        assert prior['status'] == 'PASS' and prior['tuple'] == document('qualification-inputs')['formal_tuple']
        assert all(d['item']['ref'] == 'NativeRelationSnapshotSource' and d['minimum'] == 0 for n, d in prior['exact_some_edges']['fields'])
        assert document('environment-candidate') == {**{n: None for n in CANDIDATE_FIELDS}, 'qualification_status': 'Not Verified'}
        checks = ['Frozen tuple9/8/8/8 and nominal Some edges retained through complete old-source continuity',
                  '8 historical and frozen G3 heads/profiles/denominators/commands inherited exactly, no fresh execution credit',
                  'Closed phase-local schema only; no official AFIDL/Registry/Expected/Oracle edits or authority changes']
    elif kind == 'quality':
        import ast
        tree = ast.parse((D / (P + 'independent-static-oracle.py')).read_text(encoding='utf8'))
        imported = [n.names[0].name for n in ast.walk(tree) if isinstance(n, ast.Import)]
        assert set(imported) <= {'json', 're', 'hashlib', 'base64'}
        assert 'offline-manifest-validator' not in (D / (P + 'independent-static-oracle.py')).read_text(encoding='utf8')
        assert results['r3_vectors']['preserved'] == 110 and all(row['static_result'] == 'Static PASS' for row in results['cases'])
        assert any(r['id'] == 'both-omit-field' and not r['validator_static_acceptance'] and not r['independent_oracle_acceptance'] for r in results['cases'])
        assert results['public_signature_vectors'][0]['actual'] is True
        assert all(not r['actual'] for r in results['public_signature_vectors'][1:])
        checks = ['Independent oracle does not import/call validator; independently authored expected decisions compared with both implementations',
                  'Schema/CaseId/artifact/source/gate roster, both-sided omissions, replay, status preservation, authority and signature boundaries covered',
                  'Original110 Expected bytes intact; public RFC verification tests are not production trust roots; no private keys generated']
    elif kind == 'environment':
        policy, request = document('resource-policy-analysis'), document('environment-approval-request')
        assert policy['P_build'] is policy['J_build'] is policy['target_admission_threshold'] is None
        assert policy['old_threshold_is_not_new_environment_norm'] and not policy['change_applied']
        assert all(v == 'DENIED' for v in request['permissions'].values())
        assert request['requested_values']['target_environment_identifier'] is None
        assert request['requested_values']['P_build'] is request['requested_values']['J_build'] is None
        assert results['concrete_environment'] == 'NOT QUALIFIED' and results['P1'] == results['P2'] == 'Not Authorized'
        checks = ['Candidate not guessed; actual environment/approval/toolchain/trust roots absent, qualification and execution denied',
                  'P_build/J_build remain null; test/reserve baseline retained; old-host14GiB is not universal policy',
                  'Full Cargo required; split/transport/profile changes not authorized; all P1/P2 access/transfer/build permissions denied']
    else:
        raise ValueError(kind)
    write('review-' + kind, dict(status='Static PASS', kind=kind, method='Separate protected static readback; same agent, not external certification',
          checks=checks, reviewed_inputs=phase_files(), historical_builds='0/8', P1='Not Authorized', P2='Not Authorized'), True)
    print(json.dumps(dict(review=kind, status='Static PASS', historical_builds='0/8')), flush=True)


def seal():
    assert not V.git('status', '--porcelain')
    unchanged()
    reviews = {kind: read('target/' + P + 'review-' + kind + '.json') for kind in ['specification', 'quality', 'environment']}
    for review_ in reviews.values():
        assert review_['status'] == 'Static PASS'
        assert all(V.record(n) == row for n, row in review_['reviewed_inputs'].items())
    # Reread actual historical trees without touching any build directory.
    for gate in document('validation-contract')['gates']:
        assert V.blobmap(gate['frozen_head'], Path(gate['commands']['cwd'])) == read(gate['frozen_source_manifest']['full_map'])
    lane_b()
    assert not V.git('status', '--porcelain')
    receipts = []
    for path in (R / 'target').glob(SAFE + 'r5g2r4p0-*.json'):
        row = json.loads(path.read_bytes())
        assert row['exit'] == 0 and row['cancelled'] is None and row['limits_verified']
        assert (row['cpu_hard_cap_percent'], row['process_memory_mib'], row['whole_job_memory_mib']) == (5, 768, 1536)
        receipts.append(row)
    value = dict(outcome='A — Qualification Package Ready', status='ENVIRONMENT QUALIFICATION PACKAGE READY — EXECUTION NOT AUTHORIZED',
        start_head=START, final_head=V.git('rev-parse', 'HEAD').decode().strip(),
        P0='Static PASS', inherited='Inherited PASS — original scope only', P1='Not Authorized', P2='Not Authorized',
        concrete_environment='Not Qualified', real_environment_qualification='Not Executed', real_executor_attestation='Not Executed',
        cryptographic_trust='Not Established', executor_authority='Not Established',
        historical_builds='0/8', frozen_g3_build='Not Executed', r5g2_registration='INCOMPLETE',
        successor_production_binder='NotImplemented', raw_total_t3='INCOMPLETE', T4='NOT ELIGIBLE', positive_root_export='NotAuthorized',
        new_cargo_builds=0, new_rustc_invocations=0, new_generator_runs=0, remote_executions=0, artifact_transfers=0,
        static_results=document('static-test-results'), reviews=reviews, resource_receipts=receipts,
        prior_execution_states=dict(scope='Original evidence only, never P0 new execution',
            frozen_g3=read('docs/implementation/' + OLD + 'resource-failure-analysis.json')['frozen_g3'],
            reserve_interruptions=read('docs/implementation/' + OLD + 'resource-failure-analysis.json')['reserve_interruptions']),
        added_files=phase_files(), runner_sha256=RUNNER_SHA, clean_worktree=True, pushed=False)
    assert not (R / 'target' / (P + 'pending-final-seal.json')).exists()
    write('pending-final-seal', value, True)
    print(json.dumps(dict(outcome=value['outcome'], final_head=value['final_head'], historical_builds='0/8')), flush=True)


def closeout():
    assert not V.LOCK.exists() and not V.git('status', '--porcelain')
    result = read('target/' + P + 'pending-final-seal.json')
    safety = read('target/' + SAFE + 'r5g2r4p0-final-seal.json')
    assert safety['exit'] == 0 and safety['cancelled'] is None and safety['limits_verified']
    assert V.git('rev-parse', 'HEAD').decode().strip() == result['final_head']
    command = "@(Get-CimInstance Win32_Process -Filter \"Name = 'python.exe' OR Name = 'cargo.exe' OR Name = 'rustc.exe' OR Name LIKE 'af_portable%'\" | Where-Object { $_.ProcessId -ne " + str(V.os.getpid()) + " -and $_.CommandLine -match 'r5g2r4p0' } | Select-Object ProcessId,ParentProcessId,Name) | ConvertTo-Json -Compress"
    raw = subprocess.check_output(['powershell', '-NoProfile', '-Command', command], cwd=R)
    assert not raw.strip() or json.loads(raw) == []
    result.update(final_resource_receipt=safety, protected_lock_present=False, pending_background_jobs=[], clean_worktree=True,
                  closeout_scope='Tiny post-Job metadata readback outside ended Job; no build, source traversal or environment qualification')
    assert not (R / 'target' / (P + 'final-runtime-results.json')).exists()
    write('final-runtime-results', result, True)
    print(json.dumps(dict(status=result['status'], final_head=result['final_head'], clean=True, pushed=False, pending=[])), flush=True)


if __name__ == '__main__':
    if sys.argv[1:] == ['prepare']:
        prepare()
    elif sys.argv[1:] == ['test']:
        tests()
    elif sys.argv[1:] == ['refresh-schemas']:
        refresh_schemas()
    elif sys.argv[1:] == ['targets']:
        targets()
    elif sys.argv[1:] == ['lane']:
        lane_b()
    elif sys.argv[1:] == ['protect-maps']:
        protect_maps()
    elif sys.argv[1:2] == ['review']:
        review(sys.argv[2])
    elif sys.argv[1:] == ['seal']:
        seal()
    elif sys.argv[1:] == ['closeout']:
        closeout()
    else:
        raise SystemExit('Only prepare is available; no execution mode exists')
