"""Authorized Checkpoint A serial tests; no Cargo. No automatic retry."""
import fcntl
import hashlib
import json
import os
from pathlib import Path
import resource
import subprocess
import time
import uuid
W=Path(__file__).resolve().parents[1]
E=W.parent/'evidence/phase-linux-production-protector-pilot'
MODES=['disk-preflight','pids','threads','disk-growth','disk-free','guardian-handoff','guardian-kill']

def read(p):return json.loads(p.read_bytes())
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def kv(t):return {k:int(v) for k,v in (line.split() for line in t.splitlines())}
def save(p,v):
    with p.open('x') as f:json.dump(v,f,indent=2)

def main():
    os.nice(19);resource.setrlimit(resource.RLIMIT_AS,(256*1024**2,)*2);resource.setrlimit(resource.RLIMIT_CPU,(60,)*2)
    assert not (E/'qualification-results.json').exists()
    started=time.monotonic();rows=[]
    sentinel=subprocess.Popen(['/bin/sleep','900'])
    try:
        for mode in MODES:
            assert time.monotonic()-started<900
            rid=str(uuid.uuid4());job=E/'runs'/rid
            argv=[str(W.parent/'venvs/linux-primary-v1/bin/python'),'-B',str(W/'tools/linux-production-protector.py'),mode,'--id',rid]
            p=subprocess.run(argv,cwd=W,capture_output=True,timeout=55,env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1',PYTHONNOUSERSITE='1'))
            save(E/('attempt-'+rid+'.json'),dict(mode=mode,id=rid,argv=argv,exit_code=p.returncode,stdout=p.stdout.decode(),stderr=p.stderr.decode()))
            conditions={'controller_exit':p.returncode==0,'unrelated_sentinel_alive':sentinel.poll() is None}
            result=None;meta=None
            if mode=='disk-preflight':
                f=E/('preflight-'+rid+'.json')
                conditions['receipt']=f.exists()
                if f.exists():
                    result=read(f);conditions['denied_without_launch']=result['reason']=='SimulatedDiskAdmissionDenied' and not result['task_launched'] and not job.exists()
            elif job.exists():
                try:
                    meta=read(job/'ownership.json')
                    result=read(job/('takeover-receipt.json' if mode.startswith('guardian-') else 'guardian-receipt.json'))
                    want={'pids':'Completed','threads':'Completed','disk-growth':'ProjectDiskGrowth','disk-free':'SimulatedDiskReserve','guardian-handoff':'GuardianHandoff','guardian-kill':'GuardianDied'}[mode]
                    conditions['reason']=result['reason']==want
                    conditions['owned_tree_removed']=result['cleanup']['empty'] and result['cleanup']['cgroup_removed'] and not Path(meta['cgroup']).exists()
                    conditions['raw_limits']=all(result['raw_final'].get(k)==v for k,v in meta['expected'].items())
                    conditions['no_new_swap']=result['raw_final']['memory.swap.current']=='0'
                    identities=[read(f) for f in job.glob('*identity.json') if f.name!='guardian-identity.json']
                    leaf='0::'+meta['cgroup'].removeprefix('/sys/fs/cgroup')+'/'+meta['service']
                    conditions['all_fixture_pid_ownership']=bool(identities) and all(x['cgroup'].strip()==leaf and x['rlimit_as']==[268435456,268435456] for x in identities)
                    if mode in ['pids','threads']:
                        x=read(job/'pids-exhaustion.json')
                        conditions['actual_creation_failure']=x['failure'] is not None and int(x['at_failure']['pids.max'])==8 and int(x['at_failure']['pids.current'])==8
                        conditions['kernel_pids_event']=kv(x['at_failure']['pids.events'])['max']>kv(x['before']['pids.events'])['max']
                        conditions['no_oom_substitution']=kv(result['raw_final']['memory.events'])['oom_kill']==0
                        conditions['actual_workload_exit']=read(job/'workload-exit.json')['exit_code']==0
                    if mode in ['disk-growth','disk-free']:
                        x=result['cleanup']['test_output_cleanup']
                        conditions['file_growth_and_recovery']=0<x['allocated_before']<=8*1024**2 and x['allocated_after']==0 and bool(x['files'])
                        conditions['threshold_crossed']=max(s['output_growth'] for s in result['samples'])>2*1024**2
                        conditions['real_host_reserve_safe']=all(s['disk_free']>=21474836480 and s['disk_growth']<33554432 for s in result['samples'])
                        if mode=='disk-free':conditions['simulated_free_crossed']=min(s['simulated_free_bytes'] for s in result['samples'])<4*1024**2
                    if mode=='guardian-handoff':
                        conditions['normal_handoff_before_death']=result['guardian_exit_code']==0 and (job/'handoff-request.json').exists()
                    if mode=='guardian-kill':
                        conditions['actual_guardian_sigkill']=result['guardian_exit_code']==-9 and (job/'guardian-injection.json').exists()
                        conditions['prior_verified_takeover']=read(E/'guardian-handoff-proof.json')['passed'] is True
                except BaseException as exc:
                    conditions['complete_machine_receipt']=False
                    save(job/'harness-validation-error.json',dict(error=repr(exc)))
            else:conditions['job_was_created']=False
            with (W.parent/'evidence/phase-linux-resource-qualification/top-level.lock').open('r+') as f:
                try:fcntl.flock(f.fileno(),fcntl.LOCK_EX|fcntl.LOCK_NB);conditions['no_active_lock']=True
                except BlockingIOError:conditions['no_active_lock']=False
            row=dict(mode=mode,id=rid,source_head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=W).decode().strip(),
                     executor_sha256=sha(W/'tools/linux-production-protector.py'),resource_policy_sha256=sha(W/'engineering/policies/linux-production-protector-v1.json'),
                     actual_controller_exit=p.returncode,checks=conditions,passed=all(conditions.values()))
            rows.append(row);print(json.dumps(dict(mode=mode,id=rid,passed=row['passed'],failed=[k for k,v in conditions.items() if not v])),flush=True)
            if mode=='guardian-handoff' and row['passed']:
                save(E/'guardian-handoff-proof.json',dict(passed=True,executor_sha256=row['executor_sha256'],policy_sha256=row['resource_policy_sha256'],id=rid,
                     receipt_sha256=sha(job/'takeover-receipt.json'),scope='Controller cleanup after orderly guardian handoff verified BEFORE death injection'))
            if not row['passed']:break
    finally:
        sentinel.terminate();sentinel.wait(timeout=3)
    value=dict(status='Passed' if len(rows)==7 and all(x['passed'] for x in rows) else 'Failed',Required=7,Executed=len(rows),Passed=sum(r['passed'] for r in rows),Failed=sum(not r['passed'] for r in rows),NotExecuted=7-len(rows),cases=rows,
        source_head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=W).decode().strip(),executor_sha256=sha(W/'tools/linux-production-protector.py'),fixture_sha256=sha(W/'tools/linux-production-fixtures.py'),policy_sha256=sha(W/'engineering/policies/linux-production-protector-v1.json'),
        scope='New authorized small fault tests ONLY; no kernel hard-disk quota or Pilot approval',rust_commands_launched=0)
    save(E/'qualification-results.json',value)
    print(json.dumps({k:v for k,v in value.items() if k!='cases'}),flush=True)
    return 0 if value['status']=='Passed' else 1
if __name__=='__main__':raise SystemExit(main())
