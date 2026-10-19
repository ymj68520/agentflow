"""R5 source freeze and fail-closed handoff audit; NOT a full Fact oracle."""
import hashlib,importlib.util,json,subprocess,sys
from pathlib import Path
R=Path(__file__).resolve().parents[1];P='phase-04c1b4c4r3c2r5-'
sys.path.insert(0,str(R/'tools'))
from oracle import raw_model7_full_field as O
BASE='a9e296407c682a69f596be84ac8974eb870ebc18'
ALLOWED={'crates/af-computation/Cargo.toml'}
def read(p):return json.loads((R/p).read_bytes())
def git(*args):return subprocess.check_output(['git',*args],cwd=R)
def rawmap():
    names=set(git('ls-files','--cached','--others','--exclude-standard','-z').decode().split('\0'))-{''}
    return {n:hashlib.sha256((R/n).read_bytes()).hexdigest() for n in sorted(names)}
def audit():
    baseline=read('target/'+P+'original-bytes.json');now=rawmap()
    assert set(baseline)==set(git('ls-tree','-r','--name-only',BASE).decode().splitlines())
    removed=sorted(set(baseline)-set(now));assert not removed
    changed=sorted(n for n,h in baseline.items() if now[n]!=h);assert set(changed)==ALLOWED,changed
    diff=git('diff',BASE,'--','crates/af-computation/Cargo.toml').decode()
    assert '+name = "raw_model7_capture_boundary"' in diff
    added=sorted(set(now)-set(baseline)|{'docs/implementation/'+P+'source-freeze.json'})
    assert all(n.startswith('docs/implementation/'+P) or n in {
        'crates/af-computation/tests/raw_model7_capture_boundary.rs',
        'tools/gen-raw-model7-required-fields.py','tools/oracle/raw_model7_full_field.py',
        'tools/audit-raw-model7-capture.py'} for n in added),added
    assert O.plan()['registered_g_nominal']['name']=='RelationSnapshotContent'
    spec=importlib.util.spec_from_file_location('r5_source_census',R/'tools/gen-raw-model7-required-fields.py')
    census=importlib.util.module_from_spec(spec);spec.loader.exec_module(census)
    O.exact(census.census(),read('docs/implementation/'+P+'raw-model7-required-field-manifest.json'))
    manifest=read('docs/implementation/'+P+'raw-model7-required-field-manifest.json')
    assert len(manifest['fact_roots'])==14 and len(manifest['transitive_named_descriptors'])==690 and len(manifest['declaration_rows'])==3106
    coverage=read('docs/implementation/'+P+'raw-model7-full-coverage-ledger.json')
    assert coverage['roots']['required']==2520 and coverage['roots']['compared']==0 and coverage['strict_total_exit']==2
    assert all(coverage[k]['required'] is None for k in ['nested','relations','observations'])
    mutations=read('docs/implementation/'+P+'raw-model7-comparator-mutations.json')
    assert len(mutations['boundary_mutations'])==46 and mutations['raw_full_field_mutations']==[] and mutations['full_field_credit']==0
    return dict(start_head=BASE,original_frozen_paths=baseline,original_file_count=len(baseline),
        authorized_existing_changes=changed,added_files=added,unexpected_changes=[],removed_files=removed,
        original_source_hash_equality_except_reviewed_build_entry=True,
        unchanged_original_files=len(baseline)-len(changed),production_implementation_changes=[],
        normative_registry_expected_old_oracles_g3_core_unchanged=True,
        host_limits=dict(cpu_hard_cap_percent=5,process_memory_mib=768,whole_job_memory_mib=1536,top_level_jobs=1,test_threads=1,cargo_build_jobs_environment=1),
        root_descriptor_count=14,named_fact_closure=690,static_field_variant_rows=3106,
        outcome='C',raw_full_field=False,positive_root_export='NotAuthorized',t4='NOT ELIGIBLE')
if __name__=='__main__':
    result=audit()
    if '--write' in sys.argv:(R/'docs/implementation'/(P+'source-freeze.json')).write_bytes((json.dumps(result,indent=2)+'\n').encode())
    print(json.dumps({k:v for k,v in result.items() if k!='original_frozen_paths'},indent=2))
