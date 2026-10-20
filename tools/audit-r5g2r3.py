"""R3 static evidence collector. No build/test/generator/remote execution paths.

Only new R3 derived documents and target receipts are written. Existing receipts
are read and hashed, never repaired. Git calls are read-only identity queries.
"""
import importlib.util
import json
from pathlib import Path
import sys

R = Path(__file__).resolve().parents[1]
P = 'phase-04c1b4c4r3c2r5g2r3-'
R2 = 'phase-04c1b4c4r3c2r5g2r2-'
START = '633615cc07f9510452c193f81d5bbc2fc96f677b'
G3 = 'a9e296407c682a69f596be84ac8974eb870ebc18'
spec = importlib.util.spec_from_file_location('r3_readonly_helpers', R / 'tools/verify-r5g2r1.py')
V = importlib.util.module_from_spec(spec)
spec.loader.exec_module(V)
spec = importlib.util.spec_from_file_location('r3_design_vectors', R / 'tools/evidence-r5g2r3.py')
E = importlib.util.module_from_spec(spec)
spec.loader.exec_module(E)
D = R / 'docs/implementation'
GATE_FIELDS = set('GateId FrozenSourceHEAD SourceIdentity RegistryIdentity CompilerIdentity ExpectedIdentity OracleIdentity ToolchainIdentity BuildProfile CommandAndFlags RequiredCases RequiredArtifacts ResourcePolicy ExecutionEnvironment PermittedEquivalence ForbiddenSubstitutions CurrentEvidenceStatus'.split())


def read(name):
    return json.loads((R / name).read_bytes())


def write(name, value, document=False):
    path = (D if document else R / 'target') / (P + name + '.json')
    # Derived R3 files may be refreshed during development; never old inputs.
    assert path.name.startswith(P) and path.parent in (D, R / 'target')
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf8')
    return str(path.relative_to(R)).replace('\\', '/')


def identity(path):
    return dict(path=str(path), **V.record(path))


def old(name):
    return read('target/' + R2 + name + '.json')


def unchanged_old_tree():
    for line in V.git('diff', '--name-status', START).decode().splitlines():
        status, name = line.split('\t')
        assert status == 'A' and (name.startswith('docs/implementation/' + P) or
            name in ['tools/audit-r5g2r3.py', 'tools/evidence-r5g2r3.py']), line


def preserve():
    maps = [old('B-' + n) for n in ['baseline', 'round1', 'round2', 'final']]
    assert len(maps[0]) == 2403 and all(m == maps[0] for m in maps)
    assert all(V.record(n) == value for n, value in maps[0].items())
    roster = old('B-generator-roster')
    assert len(roster) == 41
    assert sorted(r['path'] for r in roster) == sorted(p.relative_to(R).as_posix() for p in (R / 'tools').glob('gen-*.py'))
    assert all(V.record(r['path']) == {k: v for k, v in r.items() if k != 'path'} for r in roster)
    complete = old('B-complete')
    assert complete['generator_count'] == 41 and complete['commands'] == 82
    assert complete['raw_maps_equal'] and complete['source_paths'] == 2403
    logs = []
    for run in complete['rounds']:
        assert len(run['commands']) == 41 and not any(run['delta'].values())
        for receipt in run['commands']:
            assert receipt['status'] == 'PASS' and receipt['exit'] == 0
            assert V.sha(Path(receipt['log'])) == receipt['log_sha256']
            logs.append(identity(Path(receipt['log'])))
    envelope = old('lane-B')
    assert envelope['exit'] == 0 and envelope['safety']['cancelled'] is None
    assert V.sha(Path(envelope['log'])) == envelope['log_sha256']
    receipts = read('target/' + R2 + 'inheritance.json')['receipts']
    assert len(receipts) == 15
    for row in receipts:
        assert row['exit'] == row['expected_exit'] and row['safety']['cancelled'] is None
        assert V.sha(Path(row['log'])) == row['log_sha256']
    # Detect changed old tracked files too, including R2 closing documents
    # which were added after the original Lane B 2403-path scope.
    unchanged_old_tree()
    saved = [identity('target/' + R2 + 'B-' + n + '.json') for n in ['baseline', 'round1', 'round2', 'final']]
    result = dict(status='PASS — PRESERVED ORIGINAL INPUT SCOPE', generators=41, rounds=2,
                  script_runs=82, protected_paths=2403, byte_drift=0,
                  original_maps=saved, original_maps_equal=True, generator_roster=roster,
                  original_log_hashes=logs, outer_receipt=identity('target/' + R2 + 'lane-B.json'),
                  read_only_recheck=True, generators_executed_in_R3=0,
                  inherited_receipts_verified=15, inherited_cargo_test_records=448,
                  registration_cases=2880, field_variant_checks=1297,
                  credit_scope='Exactly the original 2403-path input universe; not generator execution over the expanded R3 tree',
                  additive_policy='New phase documents and non-generator audit scripts do not automatically revoke old evidence. Expanded-tree acceptance requires extension/non-interference adjudication.',
                  invalidation='Any old input/script/Registry/Expected/config byte change or unproved continuity prevents future inherited credit',
                  discovery_audit='Direct generator glob sites select spec/idl, conformance JSON or pinned oracle output directories, not new R3 docs/audit scripts. No claim of new-tree runtime equality or exhaustive dynamic discovery proof.',
                  inherited_execution_credit_is_not_new_execution=True)
    write('lane-b-credit-preservation', result, True)
    print(json.dumps(dict(check='input-continuity', paths=2403, logs=82, inherited=15)), flush=True)
    return result


def resources():
    diagnosis = old('resource-diagnosis')
    assert len(diagnosis['runs']) == 2
    for row in diagnosis['runs']:
        actual = row['actual_original_receipt']
        assert actual['exit'] == 79 and actual['cancelled'] == 'foreground_memory_reserve'
        assert Path(actual['log']).is_file()
    raw = old('lane-A')
    cargo = old('A-cargo-command')
    safety = raw['safety']
    assert raw['exit'] == cargo['exit'] == 101 and safety['cancelled'] is None
    assert V.sha(Path(cargo['log'])) == cargo['log_sha256']
    log = Path(cargo['log']).read_text(encoding='utf8', errors='replace')
    assert 'LLVM ERROR: out of memory' in log and 'debuginfo=2' in log
    assert safety['peak_process_bytes'] == 805244928 and safety['process_memory_mib'] == 768
    observation = old('history-admission')
    assert observation['samples'] == 121 and not observation['admitted']
    series = Path(observation['raw_series'])
    assert V.sha(series) == observation['raw_sha256']
    samples = [json.loads(line) for line in series.read_bytes().splitlines()]
    assert len(samples) == 121
    assert min(s['available_bytes'] for s in samples) == observation['minimum_available']
    assert max(a['available_bytes'] - b['available_bytes'] for a, b in zip(samples, samples[1:])) == observation['maximum_sample_drop']
    assert samples[-1]['elapsed_monotonic_seconds'] == observation['duration_seconds']
    assert not any(s['other_protected_job'] for s in samples)
    value = dict(status='STATIC CLASSIFICATION COMPLETE', historical_builds='0/8',
                 reserve_interruptions=diagnosis['runs'], frozen_g3=dict(
                     raw_status=raw['status'], raw_exit=101, supervisor_cancelled=None,
                     original_derived_adjudication=old('A-resource-adjudication'),
                     observed='LLVM allocation failure; Cargo 101; no tests completed',
                     debug=2, peak_process_committed=safety['peak_process_bytes'],
                     process_cap=805306368, distance_to_cap=61440,
                     peak_job_committed=safety['peak_job_bytes'],
                     minimum_system_available=safety['minimum_system_available_bytes'],
                     source_cargo_receipt=identity('target/' + R2 + 'A-cargo-command.json'),
                     raw_envelope_receipt=identity('target/' + R2 + 'lane-A.json'),
                     cargo_log=identity(Path(cargo['log'])),
                     causal_status='STRONGLY CONSISTENT WITH QUOTA; NOT FULLY PROVEN',
                     missing=['Exact failed allocation request and allocator detail',
                              'OS allocation error / NTSTATUS at allocation',
                              'Job memory-limit completion-port notification',
                              'Commit and address-space fragmentation at failure']),
                 lane_c=observation,
                 distinctions=dict(physical='GlobalMemoryStatusEx available physical memory',
                     committed='Job/Process committed virtual memory limits and peak commit; not working set',
                     llvm='Allocation failure is observed; exact Windows denial cause unproved',
                     profile='Debug2 actual rustc argv is frozen; no causal comparison with Debug0 performed',
                     cache='Incomplete cold compilation is not a complete reusable artifact provenance chain',
                     external_load='No contemporaneous independent PID series establishes causal blame',
                     audit_overhead='Protected supervisor/worker are included in job peak. No reliable decomposition into compiler versus audit/external overhead.',
                     arithmetic='Never infer external consumption by subtracting job commit from system physical availability'),
                 raw_failure_never_rewritten=True, new_heavy_executions=0,
                 no_stable_14gib_execution_window_demonstrated=True,
                 references=[
                     'https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_extended_limit_information',
                     'https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/ns-sysinfoapi-memorystatusex'])
    write('resource-failure-analysis', value, True)
    return value


def source_ids(head, folder, name):
    rows = V.blobmap(head, Path(folder))
    file = write('source-map-' + name, rows)
    def subset(predicate):
        chosen = {n: r for n, r in rows.items() if predicate(n)}
        assert chosen
        return dict(paths=len(chosen), manifest_sha256=E.digest(chosen),
                    full_map=file, selection='See audit-r5g2r3.py source_ids predicates')
    result = dict(SourceIdentity=dict(frozen_head=head, git_tree=V.git('rev-parse', head + '^{tree}').decode().strip(),
                                     paths=len(rows), complete_raw_manifest_sha256=E.digest(rows),
                                     full_map=file, full_map_file_sha256=V.sha(R / file),
                                     git_blob_and_raw_verified=True),
                  RegistryIdentity=subset(lambda n: n.startswith('spec/registry/')),
                  CompilerIdentity=subset(lambda n: n.startswith('crates/') or n in ['Cargo.toml', 'Cargo.lock']),
                  ExpectedIdentity=subset(lambda n: n.startswith('spec/conformance/')),
                  OracleIdentity=subset(lambda n: n.startswith('tools/')))
    print(json.dumps(dict(check='frozen-source', gate=name, paths=len(rows))), flush=True)
    return result


def inventory():
    cfg = read('target/phase-04c1b4c4r3c2r5g2r1-historical-configuration.json')['groups']
    prior = read('target/phase-04c1b4c4r3c2r5-histories.json')
    audited = []
    for path in [R / 'target/.rustc_info.json', R / ('target/' + R2 + 'frozen-g3/build/.rustc_info.json')]:
        if path.is_file():
            audited.append(identity(path))
    toolchain = dict(status='NOT QUALIFIED — CACHED METADATA ONLY', audited_metadata=audited,
                     cached_rustc='1.92.0 (ded5c06cf 2025-12-08)', cached_llvm='21.1.3',
                     host='x86_64-pc-windows-msvc', executed_version_queries=0,
                     needed='Exact rustc, cargo, rustdoc, LLVM/linker, MSVC/SDK, Python and dependency bytes plus effective config/environment and authenticated executor',
                     reusable_artifact='No fully authenticated Source+Toolchain+Profile+binary chain found in the audited receipts/cache metadata; not an exhaustive disk search')
    policy = dict(current=E.POLICY, identity=V.runner_identity(),
                  authority='Mandatory phase operational constraints; no unilateral host-protection reinterpretation',
                  change_allowed_now=False, proposed_compile_budget=dict(process='P_build — UNAPPROVED', job='J_build — UNAPPROVED'),
                  policy_amendment='Separate explicit approval required, even on a larger host. Semantic runtime resource contracts remain frozen.')
    forbidden = ['Debug2 replaced by Debug0', 'Current HEAD artifacts substituted for historical artifacts',
                 'Skipped targets/doc-tests/Oracle/denials', 'Changed Expected/Registry/Compiler',
                 'Reduced case denominator or double-counted cases', 'Unapproved resource-policy change',
                 'Precompiled execution called a fresh Cargo compile pass', 'Transport manifest granting RootExport/Portable authority']
    artifacts = ['Full source raw and GitBlob manifest before/after', 'Per-gate Registry/Compiler/Expected/Oracle manifest',
                 'Effective toolchain/config/env/dependency/feature/target identities',
                 'Build graph/target roster including build scripts/proc macros/doc-tests',
                 'All executables, PDBs, dynamic deps, build outputs and required runtime fixtures with hashes',
                 'Exact argv/cwd/env, authentic start/end, exit, stdout/stderr and resource/cancellation series',
                 'Trusted required CaseId roster and every actual output, without duplicates/omissions',
                 'Independent frozen Oracle receipts and capability-denial receipt',
                 'Approved policy, environment/executor qualification and integrity-sealed evidence package']
    rows = []
    for item in [*cfg, dict(group='G3', frozen_head=G3, case_ids=None,
                approved_debug=2, source=str(R / ('target/' + R2 + 'frozen-g3/source')),
                build=str(R / ('target/' + R2 + 'frozen-g3/build')))]:
        number = item['group']
        gate = 'frozen-g3' if number == 'G3' else 'history-' + str(number)
        ids = source_ids(item['frozen_head'], item['source'], gate)
        command = ['cargo', 'test', '--all', '-j', '1', '--manifest-path',
                   str(Path(item['source']) / 'Cargo.toml'), '--target-dir', item['build']]
        auditors = []
        if number in [3, 7, 8]:
            script, flags = {3: ('audit-substitution-staging.py', []),
                             7: ('audit-relation-schema-staging.py', ['--receipts']),
                             8: ('audit-structural-binding-staging.py', ['--receipts'])}[number]
            auditors.append(['<qualified-python>', '-X', 'utf8', 'tools/' + script, *flags])
        if number == 'G3':
            auditors = [['<qualified-python>', '-X', 'utf8', 'tools/oracle/production_input_graph.py', flag]
                        for flag in ['--all', '--mutations']]
            auditors.append(['<approved-frozen-evidence-checker>', 'A-evidence'])
        row = dict(GateId=gate, FrozenSourceHEAD=item['frozen_head'], **ids,
                   ToolchainIdentity=toolchain,
                   BuildProfile=dict(dev_debug=item['approved_debug'], test_debug=item['approved_debug'],
                                     features='Frozen defaults and manifest-selected targets; no narrowing',
                                     config_precedence='Snapshot root profiles, Cargo config, environment, rustflags and build scripts; no hidden override'),
                   CommandAndFlags=dict(cargo=command, cwd=item['source'],
                       env=dict(CARGO_PROFILE_DEV_DEBUG=str(item['approved_debug']),
                                CARGO_PROFILE_TEST_DEBUG=str(item['approved_debug']),
                                CARGO_BUILD_JOBS='1', RUST_TEST_THREADS='1'),
                       independent_checks=auditors,
                       identity_checks='Full source pre/post, historical per-path correspondence and independent frozen Registry/Expected/Oracle; no fresh reconstruction',
                       target_relocation='Design mapping only: source/build paths may be rebound only by approved path/fixture equivalence; no execution here'),
                   RequiredCases=dict(declared_case_ids=item['case_ids'],
                       frozen_g3_classes=dict(positive=154, negative_and_resource_nominal_boundaries=26,
                                              comparator_mutations=59) if number == 'G3' else None,
                       exact_roster_status='Not Executed — enumerate/certify from frozen inputs before execution admission',
                       source_of_requirement='R1 frozen historical configuration and original frozen manifests/oracles; G3 complete --all/--mutations and original A-evidence requirements',
                       rust_test_count='Distinct from declared CaseIds; complete frozen Cargo target/test/doc-test roster required',
                       denominator='Independently approved complete roster; never infer from submitted actual/expected intersections'),
                   RequiredArtifacts=artifacts, ResourcePolicy=policy,
                   ExecutionEnvironment=dict(recommended='Option B isolated Windows x64 MSVC-compatible environment',
                       environment_id=None, qualification='Not Executed', admission='NOT ADMITTED',
                       conditions='OS/ABI/toolchain/Python/fixtures compatible; authenticated executor; system AND compile-process budgets separately approved and measured'),
                   PermittedEquivalence=dict(already_allowed='Exact frozen byte identity and previously approved per-path historical correspondence only',
                       proposed='Host/path mapping, authenticated evidence transport, split build/execute, any changed compile protection budget require separate explicit approval',
                       currently_granted=False), ForbiddenSubstitutions=forbidden,
                   CurrentEvidenceStatus=dict(new_execution='Not Executed', fresh_pass=False,
                       prior_receipt='Frozen R5 old pass retained as provenance only' if number != 'G3' else 'R2 FAILED 101 / LLVM OOM retained',
                       historical_completed=0))
        assert GATE_FIELDS <= set(row)
        if number != 'G3':
            assert item['frozen_head'] == prior[number - 1]['frozen_head']
        rows.append(row)
    assert len(rows) == 9 and sum(r['RequiredCases']['declared_case_ids'] or 0 for r in rows) == 6337
    inherited = read('target/' + R2 + 'inheritance.json')['receipts']
    current_map = old('B-final')
    categories = dict(SourceIdentity=dict(raw_map='target/' + R2 + 'B-final.json',
                       scope_paths=2403, manifest_sha256=E.digest(current_map)),
                     RegistryIdentity=dict(scope='spec/registry and exact nominal tuple 9/8/8/8'),
                     CompilerIdentity=dict(scope='All old production Rust/config unchanged'),
                     ExpectedIdentity=dict(scope='All old conformance Expected unchanged'),
                     OracleIdentity=dict(scope='All old tools/Oracle unchanged'))
    for index, receipt in enumerate(inherited, 1):
        row = dict(GateId='inherited-' + str(index), FrozenSourceHEAD=receipt['head'], **categories,
                   ToolchainIdentity=dict(status='Historical receipt only; not a new qualified environment'),
                   BuildProfile=dict(status='Original receipt frozen; no new invocation', receipt=receipt),
                   CommandAndFlags=receipt['command'],
                   RequiredCases=dict(status='Exact original command/case/field/denial scope',
                       totals_not_additive='2880 registration cases, 1297 field/variant checks, 448 Cargo records across original receipts; no overlapping sum'),
                   RequiredArtifacts=['Original log, receipt and unchanged original source input map'],
                   ResourcePolicy=policy, ExecutionEnvironment=dict(status='Original historical local execution only'),
                   PermittedEquivalence='Same-source inherited credit only; new docs not a rerun',
                   ForbiddenSubstitutions=forbidden,
                   CurrentEvidenceStatus=dict(status='INHERITED SAME-SOURCE PASS', new_execution='Not Executed',
                       original_log_hash=receipt['log_sha256'], new_credit=0))
        assert GATE_FIELDS <= set(row)
        rows.append(row)
    lane = dict(rows[-1], GateId='lane-B', FrozenSourceHEAD=old('lane-B')['head'],
                CommandAndFlags=dict(source='target/' + R2 + 'B-complete.json', commands=82),
                RequiredCases=dict(generators=41, rounds=2, runs=82, input_paths=2403),
                CurrentEvidenceStatus=dict(status='PRESERVED ORIGINAL-SCOPE PASS', new_execution='Not Executed', new_credit=0))
    rows.append(lane)
    value = dict(status='STATIC CONTRACT INVENTORY', gates=rows,
                 contract_dimensions=['Source/spec identity', 'Build configuration', 'Execution environment', 'Resource protection policy'],
                 historical_builds='0/8', original_gate_not_replaced=True,
                 case_roster_qualification_is_mandatory_before_future_admission=True,
                 no_currently_qualified_reusable_artifact=toolchain['reusable_artifact'])
    write('verification-contract-inventory', value, True)
    plan = dict(status='Not Executed', historical_builds='0/8', recommended_option='B',
                groups=[dict(gate=r['GateId'], frozen_head=r['FrozenSourceHEAD'],
                    source_identity=r['SourceIdentity'], debug=r['BuildProfile']['dev_debug'],
                    required_case_ids=r['RequiredCases']['declared_case_ids'],
                    commands=r['CommandAndFlags'], target_environment='Not selected / Not Executed',
                    minimum_trust_conditions=r['ExecutionEnvironment']['conditions'],
                    risk='System reserve instability independent of process commit cap; no measured full compile peak',
                    split_legality='NOT APPROVED: original full Cargo gate remains mandatory',
                    missing_evidence_state='NOT ADMITTED / Not Executed; partial logs never PASS',
                    old_evidence='Preserve all old pass/failure/interruption logs and approved per-path correspondence; no new credit',
                    rerun='Full frozen Cargo targets and required independent Oracle, identity, denial, resource and final receipt gates')
                    for r in rows[:8]])
    write('historical-execution-plan', plan, True)
    return value


def collect():
    assert V.git('rev-parse', 'HEAD').decode().strip() == START
    preserve()
    resources()
    inventory()
    write('cross-environment-evidence-manifest', E.manifest_design(), True)
    v = E.vectors()
    write('design-vector-results', v, True)
    snapshot = {str(p.relative_to(R)).replace('\\', '/'): V.record(p.relative_to(R))
                for p in (R / 'target').glob(R2 + '*') if p.is_file()}
    # Also seal the two original history interruption envelopes and logs.
    for item in old('resource-diagnosis')['runs']:
        log = Path(item['actual_original_receipt']['log'])
        for path in [log, log.with_suffix('.json')]:
            snapshot[path.relative_to(R).as_posix()] = V.record(path.relative_to(R))
    write('prior-evidence-integrity', snapshot)
    print(json.dumps(dict(static_collection='PASS', design_vectors=v['total'], historical_builds='0/8', heavy_executions=0)), flush=True)


def refresh_design():
    """Refresh only newly added R3 design data, without rehashing frozen trees."""
    write('cross-environment-evidence-manifest', E.manifest_design(), True)
    v = E.vectors()
    write('design-vector-results', v, True)
    path = 'docs/implementation/' + P + 'verification-contract-inventory.json'
    inv = read(path)
    for gate in inv['gates'][:9]:
        gate['RequiredCases']['frozen_g3_classes'] = (dict(positive=154,
            negative_and_resource_nominal_boundaries=26, comparator_mutations=59)
            if gate['GateId'] == 'frozen-g3' else None)
    write('verification-contract-inventory', inv, True)
    resources()
    print(json.dumps(dict(design_vectors=v['total'], actual_gate_execution='Not Executed')), flush=True)


def review(kind):
    # Independent readback assertions, not self-reported PASS from a generator.
    inv = read('docs/implementation/' + P + 'verification-contract-inventory.json')
    b = read('docs/implementation/' + P + 'lane-b-credit-preservation.json')
    resources_doc = read('docs/implementation/' + P + 'resource-failure-analysis.json')
    manifest = read('docs/implementation/' + P + 'cross-environment-evidence-manifest.json')
    prior = read('target/' + P + 'prior-evidence-integrity.json')
    assert all(V.record(n) == r for n, r in prior.items())
    unchanged_old_tree()
    gates = inv['gates']
    assert len(gates) == 25 and len({g['GateId'] for g in gates}) == 25
    assert all(GATE_FIELDS <= set(g) for g in gates)
    assert inv['historical_builds'] == '0/8' and all(g['CurrentEvidenceStatus']['new_execution'] == 'Not Executed' for g in gates)
    checks = []
    if kind == 'specification':
        report = old('specification-review')
        assert report['status'] == 'PASS'
        assert [report['tuple'][k].split('@')[-1] for k in ['analysis', 'model', 'report', 'obligation']] == ['9', '8', '8', '8']
        edges = report['exact_some_edges']['fields']
        assert len(edges) == 3 and all(d['item'] == {'ref': 'NativeRelationSnapshotSource'} and d['kind'] == 'option' and d['minimum'] == 0 for _, d in edges)
        assert all(V.record(n) == r for n, r in old('B-final').items())
        assert [g['BuildProfile']['dev_debug'] for g in gates[:9]] == [2] * 7 + [0, 2]
        assert [g['RequiredCases']['declared_case_ids'] for g in gates[:8]] == [512, 51, 173, 889, 262, 254, 534, 3662]
        assert all(g['PermittedEquivalence']['currently_granted'] is False for g in gates[:9])
        checks = ['Frozen 9/8/8/8 and exact three nominal Some edges preserved by full source continuity',
                  'Every gate has all mandatory contract fields; 8 historical profiles and denominators exact',
                  'No Spec/Registry/production/Expected/Oracle/config authority changes; no historical test credit']
    elif kind == 'quality':
        assert (b['generators'], b['rounds'], b['script_runs'], b['protected_paths'], b['byte_drift']) == (41, 2, 82, 2403, 0)
        v = E.vectors()
        saved = read('docs/implementation/' + P + 'design-vector-results.json')
        assert saved == v and v['total'] == v['passed'] and v['execution_credit'] == 0
        assert manifest['required_fields'] == list(E.FIELDS) and not manifest['real_template_qualifies_for_pass']
        assert not manifest['additional_fields_allowed']
        checks = ['82 original generator log hashes and all four 2403-path raw maps retained; additive R3 scope not claimed executed',
                  'Fixed trusted toy schema/rosters reject one- and both-sided omissions and forged PASS/authority',
                  'Synthetic sentinels cannot validate real gates; no cryptographic executor verifier claimed implemented']
    elif kind == 'environment':
        assert resources_doc['frozen_g3']['raw_exit'] == 101
        assert resources_doc['frozen_g3']['supervisor_cancelled'] is None
        assert resources_doc['frozen_g3']['causal_status'].endswith('NOT FULLY PROVEN')
        assert resources_doc['lane_c']['samples'] == 121 and not resources_doc['lane_c']['admitted']
        runner = V.runner_identity()
        assert (runner['cpu_percent'], runner['process_mib'], runner['job_mib'], runner['foreground_reserve_bytes']) == (5, 768, 1536, 8589934592)
        assert all(not g['ResourcePolicy']['change_allowed_now'] and g['ExecutionEnvironment']['admission'] == 'NOT ADMITTED' for g in gates[:9])
        for name in ['recommended-execution-strategy', 'security-and-resource-policy', 'next-authorization-request']:
            text = (D / (P + name + '.md')).read_text(encoding='utf8')
            assert 'Not Executed' in text and 'P_build' in text and 'J_build' in text
        checks = ['Raw FAILED101/cancelled-null separate from derived resource interruption; missing allocation/OS/Job causal evidence explicit',
                  'System physical headroom and compile-process committed budget independent; 14GiB window not established',
                  'Runner caps unchanged; resource amendment, concrete environment and split/transport equivalence need separate approval']
    else:
        raise ValueError(kind)
    inputs = {}
    for entry in D.glob(P + '*'):
        for p in ([entry] if entry.is_file() else entry.rglob('*')):
            if p.is_file():
                inputs[p.relative_to(R).as_posix()] = V.record(p.relative_to(R))
    for name in ['tools/audit-r5g2r3.py', 'tools/evidence-r5g2r3.py']:
        inputs[name] = V.record(name)
    result = dict(status='PASS', kind=kind, method='Independent readback checks in a separate protected Python invocation; same agent, not external human review',
                  scope='Design/static review only; no environment qualification or real gate execution',
                  checks=checks, prior_evidence_files_exact=len(prior), historical_builds='0/8',
                  new_builds=0, new_generator_runs=0, positive_authority_issued=False,
                  reviewed_input_hashes=inputs)
    write('review-' + kind, result)
    print(json.dumps(result), flush=True)


def seal():
    assert not V.git('status', '--porcelain'), 'Final commit must precede external seal'
    changes = V.git('diff', '--name-status', START, 'HEAD').decode().splitlines()
    assert changes and all(line.startswith('A\t') and
        (line.split('\t')[1].startswith('docs/implementation/' + P) or
         line.split('\t')[1] in ['tools/audit-r5g2r3.py', 'tools/evidence-r5g2r3.py']) for line in changes)
    preserve()
    # preserve() rewrites a derived JSON with identical bytes; status must stay clean.
    assert not V.git('status', '--porcelain')
    prior = read('target/' + P + 'prior-evidence-integrity.json')
    assert all(V.record(n) == r for n, r in prior.items())
    reviews = {k: read('target/' + P + 'review-' + k + '.json') for k in ['specification', 'quality', 'environment']}
    assert all(v['status'] == 'PASS' for v in reviews.values())
    for report in reviews.values():
        assert all(V.record(n) == r for n, r in report['reviewed_input_hashes'].items())
    all_new = {line.split('\t')[1]: V.record(line.split('\t')[1]) for line in changes}
    frozen = []
    inv = read('docs/implementation/' + P + 'verification-contract-inventory.json')
    for gate in inv['gates'][:9]:
        source = Path(gate['CommandAndFlags']['cwd'])
        expected = read(gate['SourceIdentity']['full_map'])
        assert E.digest(expected) == gate['SourceIdentity']['complete_raw_manifest_sha256']
        actual = V.blobmap(gate['FrozenSourceHEAD'], source)
        assert actual == expected
        frozen.append(dict(gate=gate['GateId'], paths=len(actual), changed=0))
        print(json.dumps(dict(final_frozen_check=gate['GateId'], paths=len(actual))), flush=True)
    receipts = []
    for path in sorted((R / 'target').glob('phase-04c1b4c4r3c2r4g3-safe-r5g2r3-*.json')):
        row = json.loads(path.read_bytes())
        assert row['exit'] == 0 and row['cancelled'] is None and row['limits_verified']
        assert (row['cpu_hard_cap_percent'], row['process_memory_mib'], row['whole_job_memory_mib']) == (5, 768, 1536)
        receipts.append(identity(path))
    value = dict(outcome='A — Verification Strategy Design Accepted',
                 status='VERIFICATION ENVIRONMENT / STRATEGY DESIGN COMPLETE',
                 r5g2_status='Registration Incomplete', implementation_admission='NOT ADMITTED',
                 start_head=START, final_head=V.git('rev-parse', 'HEAD').decode().strip(),
                 historical_builds='0/8', new_cargo_builds=0, new_rustc_invocations=0,
                 new_generator_runs=0, new_remote_executions=0, artifact_transfers=0,
                 proposed_environments='Not Executed', real_evidence_manifest='Not Executed',
                 preserved_lane_b=dict(generators=41, rounds=2, runs=82, paths=2403, drift=0),
                 inherited_cargo_records=448, registration_cases=2880, field_variant_checks=1297,
                 reviews=reviews, design_vectors=read('docs/implementation/' + P + 'design-vector-results.json')['total'],
                 runner=V.runner_identity(), protected_static_receipts=receipts,
                 prior_evidence_integrity=dict(files=len(prior), changed=0),
                 final_frozen_source_checks=frozen,
                 added_files=all_new, clean_worktree=True, pushed=False,
                 next_stage='R5G2R4 only after separate concrete environment/resource/equivalence authorization',
                 no_positive_native_portable_root_authority=True)
    path = R / 'target' / (P + 'pending-final-seal.json')
    assert not path.exists(), 'Preserve final seal; do not overwrite'
    write('pending-final-seal', value)
    print(json.dumps({k: value[k] for k in ['outcome', 'status', 'final_head', 'historical_builds']}), flush=True)


def closeout():
    """Tiny post-Job metadata readback; no protected workload can be active.

    This does not build, sample load, hash source trees or qualify an environment.
    It records the already-ended seal Job's real exit and a read-only PID check.
    """
    assert not V.LOCK.exists() and not V.git('status', '--porcelain')
    value = read('target/' + P + 'pending-final-seal.json')
    file = 'target/phase-04c1b4c4r3c2r4g3-safe-r5g2r3-final-seal.json'
    safety = read(file)
    assert safety['exit'] == 0 and safety['cancelled'] is None and safety['limits_verified']
    assert (safety['cpu_hard_cap_percent'], safety['process_memory_mib'], safety['whole_job_memory_mib']) == (5, 768, 1536)
    assert V.git('rev-parse', 'HEAD').decode().strip() == value['final_head']
    command = "@(Get-CimInstance Win32_Process -Filter \"Name = 'python.exe' OR Name = 'cargo.exe' OR Name = 'rustc.exe' OR Name LIKE 'af_portable%'\" | Where-Object { $_.ProcessId -ne " + str(V.os.getpid()) + " -and $_.CommandLine -match 'r5g2r3' } | Select-Object ProcessId,ParentProcessId,Name,CommandLine) | ConvertTo-Json -Compress"
    raw = V.subprocess.check_output(['powershell', '-NoProfile', '-Command', command], cwd=R)
    tasks = json.loads(raw) if raw.strip() else []
    assert tasks == [], ('R3 process still present', tasks)
    value.update(final_safety_receipt=dict(file=file, **safety),
                 pending_background_jobs=[], protected_lock_present=False,
                 post_job_check='Actual protected seal exited0/cancelled-null; lock absent; no other R3 Python/Cargo/rustc/test processes. Only this tiny metadata closeout runs outside the ended Job.',
                 clean_worktree=True, pushed=False)
    path = R / 'target' / (P + 'final-runtime-results.json')
    assert not path.exists()
    write('final-runtime-results', value)
    print(json.dumps(dict(outcome=value['outcome'], final_head=value['final_head'],
                         clean_worktree=True, pending_background_jobs=tasks, pushed=False)), flush=True)


if __name__ == '__main__':
    if sys.argv[1:] == ['collect']:
        collect()
    elif sys.argv[1:2] == ['review']:
        review(sys.argv[2])
    elif sys.argv[1:] == ['seal']:
        seal()
    elif sys.argv[1:] == ['refresh-design']:
        refresh_design()
    elif sys.argv[1:] == ['closeout']:
        closeout()
    else:
        raise SystemExit('Only collect, review specification|quality|environment, seal are allowed')
