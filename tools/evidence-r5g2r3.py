"""Tiny DESIGN-ONLY evidence vectors. Never validates real execution authority."""
import copy
import hashlib
import json
from pathlib import Path

FIELDS = tuple('SchemaVersion SyntheticScope ManifestStatus GateId EnvironmentId OSAndArchitecture RustCargoLLVMVersions PythonVersion FrozenSourceHEAD CompleteSourceRawHash RegistryHash CompilerHash ExpectedHash OracleHash ToolchainIdentityHash BuildProfile BuildCommand TestCommand BuildArtifactHashes ExecutionStartAndEnd ActualExitCode ResourcePolicy ResourceSafetyStatus RequiredAndCompletedCaseIds ActualOutputHashes IndependentOracleReceipt PreAndPostSourceIntegrity CapabilityDenialReceipt EnvironmentQualificationReceipt ExecutorReceipt PermittedEquivalenceApproval'.split())
CASE_IDS = ['toy/a', 'toy/b']
OUTPUTS = {'toy/a': '4' * 64, 'toy/b': '5' * 64}
ARTIFACTS = {'bin/toy.exe': '6' * 64, 'symbols/toy.pdb': '7' * 64}
POLICY = dict(cpu_percent=5, process_mib=768, job_mib=1536, reserve_bytes=8589934592,
              test_scheduler_threads=1, cargo_jobs=1, priority='BelowNormal')


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def toy():
    result = {k: 'a' * 64 for k in FIELDS}
    result.update(SchemaVersion='R3-design-fixture-v1', SyntheticScope='DESIGN_VECTOR_ONLY',
                  ManifestStatus='PASS', GateId='R3/toy', EnvironmentId='toy-qualified-env',
                  OSAndArchitecture={'os': 'toy-windows', 'architecture': 'x86_64-pc-windows-msvc'},
                  RustCargoLLVMVersions={'rustc': 'toy-rustc', 'cargo': 'toy-cargo', 'llvm': 'toy-llvm'},
                  PythonVersion='toy-python', FrozenSourceHEAD='1' * 40,
                  BuildProfile={'dev_debug': 2, 'test_debug': 2, 'features': 'frozen-default'},
                  BuildCommand=['cargo', 'test', '--all', '-j', '1', '--no-run'],
                  TestCommand=['cargo', 'test', '--all', '-j', '1'],
                  BuildArtifactHashes=dict(ARTIFACTS), ExecutionStartAndEnd={'start': 10, 'end': 20},
                  ActualExitCode=0, ResourcePolicy=dict(POLICY),
                  ResourceSafetyStatus={'status': 'PASS', 'cancelled': None, 'limits_verified': True},
                  RequiredAndCompletedCaseIds={'required': list(CASE_IDS), 'completed': list(CASE_IDS)},
                  ActualOutputHashes=dict(OUTPUTS),
                  IndependentOracleReceipt={'status': 'PASS', 'case_ids': list(CASE_IDS), 'outputs_hash': digest(OUTPUTS)},
                  PreAndPostSourceIntegrity={'before': 'a' * 64, 'after': 'a' * 64},
                  CapabilityDenialReceipt={'root_export': 'NotAuthorized', 'portable': 'NotAuthorized',
                                            'complete_view': 'NotIssued', 'obligation_identity': 'NotIssued'},
                  EnvironmentQualificationReceipt='SYNTHETIC_PINNED_QUALIFICATION',
                  ExecutorReceipt='SYNTHETIC_OUT_OF_BAND_RECEIPT',
                  PermittedEquivalenceApproval='SYNTHETIC_SPLIT_APPROVAL')
    return result


def validate_design_fixture(value):
    """Fixed out-of-band toy requirements, not a caller-submitted denominator.

    This function intentionally refuses every real gate/environment. Synthetic
    receipt sentinels model an independent verifier; they are NOT signatures.
    """
    expected = toy()
    assert type(value) is dict and set(value) == set(FIELDS)
    assert value['SyntheticScope'] == 'DESIGN_VECTOR_ONLY' and value['GateId'] == 'R3/toy'
    assert value['ManifestStatus'] == 'PASS' and type(value['ActualExitCode']) is int and value['ActualExitCode'] == 0
    assert value['EnvironmentId'] == expected['EnvironmentId']
    assert value['ResourcePolicy'] == POLICY
    assert value['RequiredAndCompletedCaseIds'] == expected['RequiredAndCompletedCaseIds']
    assert value['ActualOutputHashes'] == OUTPUTS and value['BuildArtifactHashes'] == ARTIFACTS
    assert value['ResourceSafetyStatus'] == expected['ResourceSafetyStatus']
    assert value['IndependentOracleReceipt'] == expected['IndependentOracleReceipt']
    assert value['CapabilityDenialReceipt'] == expected['CapabilityDenialReceipt']
    def exact(actual, required):
        assert type(actual) is type(required)
        if type(required) is dict:
            assert set(actual) == set(required)
            for key in required:
                exact(actual[key], required[key])
        elif type(required) is list:
            assert len(actual) == len(required)
            for a, b in zip(actual, required):
                exact(a, b)
        else:
            assert actual == required
    exact(value, expected)
    return dict(design_shape='VALID_SYNTHETIC_FIXTURE', execution_credit=0, authority=False)


def vectors():
    baseline = toy()
    validate_design_fixture(baseline)
    rows = [dict(id='toy-complete', static_check='PASS', expected_acceptance=True,
                 operational_execution_status='Not Executed', authority=False)]
    def rejection(name, mutate, both=False):
        want, actual = copy.deepcopy(baseline), copy.deepcopy(baseline)
        mutate(actual)
        if both:
            mutate(want)
        try:
            validate_design_fixture(want)
            validate_design_fixture(actual)
        except (AssertionError, KeyError, TypeError):
            rows.append(dict(id=name, static_check='PASS', expected_acceptance=False,
                             both_sides=both, operational_execution_status='Not Executed', authority=False))
            return
        raise AssertionError('Accepted invalid design vector: ' + name)
    for field in FIELDS:
        rejection('omit/' + field, lambda v, f=field: v.pop(f))
        rejection('both-omit/' + field, lambda v, f=field: v.pop(f), True)
    for parent in ['OSAndArchitecture', 'RustCargoLLVMVersions', 'BuildProfile',
                   'ResourcePolicy', 'ResourceSafetyStatus', 'RequiredAndCompletedCaseIds',
                   'IndependentOracleReceipt', 'PreAndPostSourceIntegrity', 'CapabilityDenialReceipt']:
        for field in baseline[parent]:
            rejection('both-nested-omit/' + parent + '/' + field,
                      lambda v, p=parent, f=field: v[p].pop(f), True)
    for state in ['FAILED', 'NOT EXECUTED', 'INTERRUPTED BY RESOURCE SAFETY']:
        rejection('cannot-promote/' + state, lambda v, s=state: v.update(ManifestStatus=s))
    rejection('both-shrink-case-roster', lambda v: v['RequiredAndCompletedCaseIds'].update(required=['toy/a'], completed=['toy/a']), True)
    rejection('both-delete-output', lambda v: v['ActualOutputHashes'].pop('toy/b'), True)
    rejection('both-delete-artifact', lambda v: v['BuildArtifactHashes'].pop('symbols/toy.pdb'), True)
    rejection('cross-environment-swap', lambda v: v.update(EnvironmentId='other-host'))
    rejection('wrong-profile', lambda v: v['BuildProfile'].update(dev_debug=0))
    rejection('nonzero-exit', lambda v: v.update(ActualExitCode=79))
    rejection('bool-is-not-integer', lambda v: v['ResourcePolicy'].update(cargo_jobs=True))
    rejection('float-is-not-integer', lambda v: v['ExecutionStartAndEnd'].update(start=10.0))
    rejection('cancelled-not-pass', lambda v: v['ResourceSafetyStatus'].update(cancelled='foreground_memory_reserve'))
    rejection('changed-source', lambda v: v['PreAndPostSourceIntegrity'].update(after='b' * 64))
    rejection('forged-root-authority', lambda v: v['CapabilityDenialReceipt'].update(root_export='Authorized'))
    rejection('caller-trust-flag', lambda v: v.update(CallerTrusted=True))
    rejection('reject-real-gate', lambda v: v.update(SyntheticScope='REAL', GateId='history-1'))
    rejection('missing-qualification', lambda v: v.update(EnvironmentQualificationReceipt=None))
    rejection('unapproved-split', lambda v: v.update(PermittedEquivalenceApproval=None))
    assert len({r['id'] for r in rows}) == len(rows)
    return dict(scope='Design-only algorithm vectors; no real environment, build, test, or gate executed',
                operational_execution_status='Not Executed', static_check_executed=True,
                total=len(rows), passed=len(rows), execution_credit=0, authority=False, vectors=rows)


def manifest_design():
    return dict(design_status='DRAFT — NOT A REGISTERED SPECIFICATION',
                operational_execution_status='Not Executed', required_fields=list(FIELDS),
                additional_fields_allowed=False, fixture_only_example=toy(),
                real_execution_template={k: ('Not Executed' if k == 'ManifestStatus' else None) for k in FIELDS},
                real_template_qualifies_for_pass=False,
                field_constraints=dict(
                    SchemaVersion='Exact independently approved schema identity; draft not valid for real PASS',
                    SyntheticScope='Real execution must never use DESIGN_VECTOR_ONLY; toy validator refuses real scope',
                    ManifestStatus='Real PASS only after all independent validation obligations; Not Executed never PASS',
                    GateId='Exact approved gate identity and execution nonce/sequence in signed executor receipt',
                    EnvironmentId='Out-of-band qualified executor/environment identity, not self-assertion',
                    OSAndArchitecture='Closed approved OS/version/build/architecture/ABI record',
                    RustCargoLLVMVersions='Exact versions plus toolchain binary/SDK/linker hashes in ToolchainIdentityHash',
                    PythonVersion='Exact interpreter/dependency identity, not version string alone',
                    FrozenSourceHEAD='40 lowercase hexadecimal characters; independently frozen expected HEAD',
                    hash_fields='CompleteSourceRawHash/RegistryHash/CompilerHash/ExpectedHash/OracleHash/ToolchainIdentityHash: 64 lowercase hex, bind independently complete path/length/hash maps',
                    BuildProfile='Exact dev/test debug, opt-level, features, target, rustflags and effective configuration closure',
                    commands='BuildCommand/TestCommand: exact argv arrays plus signed cwd and environment in ExecutorReceipt; no shell interpolation',
                    BuildArtifactHashes='Complete independently required logical path to raw length/hash map, including symbols/dependencies/build outputs/fixtures',
                    ExecutionStartAndEnd='Exact UTC integer nanoseconds and monotonic sequence, ordered and executor-authenticated; toy values are synthetic',
                    ActualExitCode='Integer, never bool/float; zero only for positive PASS gates; frozen expected negative exits require separate explicitly typed denial receipt',
                    ResourcePolicy='Exact approved policy identifier plus hard budgets and per-step class; no caller-defined P_build/J_build',
                    ResourceSafetyStatus='Exact authenticated limits, peaks, samples, cancellations and observation coverage; no cancellation or unknown safety for PASS',
                    RequiredAndCompletedCaseIds='Two complete duplicate-free namespaced lists matching independent approved roster; both-side agreement alone insufficient',
                    ActualOutputHashes='Every required CaseId/output logical path plus raw hash/length and full structured value package for field comparison',
                    IndependentOracleReceipt='Independent frozen Oracle identity, exact case/field roster and compared/missing/unexpected/uncompared counts, original actual/expected/diffs',
                    PreAndPostSourceIntegrity='Both complete raw/GitBlob maps match independent frozen input; no general EOL normalization',
                    CapabilityDenialReceipt='Exact required negative boundaries; no RootExport/Portable positive, CompleteView or ObligationIdentity issued',
                    EnvironmentQualificationReceipt='Authenticated independently approved qualification/policy/OS/ABI/toolchain/package receipt',
                    ExecutorReceipt='Detached signature binds all fields, package hash, requirements hash, key identity, nonce, sequence and effective cwd/env; trusted key out of band',
                    PermittedEquivalenceApproval='Out-of-band exact applicable approvals; no implicit host/path/split/profile/resource substitution'),
                validation_protocol=[
                    'Bound bytes/path count/depth and reject duplicate keys, unknown keys, invalid UTF-8, links/traversal, nonfinite/float numeric values',
                    'Verify independent schema/requirements/complete rosters and authenticated qualification/executor/approval trust roots',
                    'Check every side independently for exact types/fields/CaseIds/artifacts/outputs and denied capabilities',
                    'Verify canonical signed payload, nonce/replay rules and transport package hashes',
                    'Validate exact frozen inputs/toolchain/effective profile/commands/environment/resource/time/exit/oracle closure',
                    'Compare complete structured outputs field-by-field; preserve raw hashes and precise differences',
                    'Only approved real validator can issue scoped execution evidence; transport never grants production authority'],
                trust_roots='Approved requirements roster and environment/executor keys must be supplied out of band; never trusted from the manifest itself',
                signature_design='Detached versioned canonical payload signature binds schema, gate requirements, source, toolchain, policy, environment qualification, artifacts and outputs. Actual cryptographic verifier is not implemented or qualified here.',
                both_sided_omission_rule='Validate each side against fixed approved schema and complete trusted case/artifact rosters before equality comparison',
                implementation_boundaries='No actual signature, environment qualification, Native receipt, RootExport, Portable capability or PASS certificate is issued')
