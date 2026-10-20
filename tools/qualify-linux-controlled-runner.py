"""Serial small qualification suite. Stops on first failed invariant; no Cargo."""
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
E=W.parent/'evidence/phase-linux-resource-qualification'
MODES=['normal','nonzero','membership','cpu','process-limit','oom','timeout','cancel','orphan','repeat','controller-crash']

def read(p):
    return json.loads(p.read_bytes())

def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()

def kv(text):
    return {k:int(v) for k,v in (line.split() for line in text.splitlines())}

def main():
    os.nice(19)
    resource.setrlimit(resource.RLIMIT_AS,(256*1024**2,)*2)
    resource.setrlimit(resource.RLIMIT_CPU,(60,)*2)
    assert not (E/'qualification-results-v2.json').exists()
    sentinel=subprocess.Popen(['/bin/sleep','900'])
    rows=[]
    begin=time.monotonic()
    try:
        for mode in MODES:
            if time.monotonic()-begin>900:
                raise RuntimeError('Approved suite wall boundary')
            rid=str(uuid.uuid4());job=E/'runs'/rid
            argv=[str(W.parent/'venvs/linux-primary-v1/bin/python'),'-B',str(W/'tools/linux-controlled-runner.py'),mode,'--id',rid]
            p=subprocess.run(argv,cwd=W,capture_output=True,timeout=55,env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1',PYTHONNOUSERSITE='1'))
            (job/'harness-controller.stdout').write_bytes(p.stdout)
            (job/'harness-controller.stderr').write_bytes(p.stderr)
            for _ in range(300):
                if (job/'guardian-receipt.json').exists() or (job/'guardian-error.json').exists():break
                time.sleep(0.05)
            conditions={}
            expected={'oom':'OOM','timeout':'Timeout','cancel':'UserCancelled','controller-crash':'ControllerLost'}.get(mode,'Completed')
            conditions['controller_exit']=p.returncode==(-9 if mode=='controller-crash' else 0)
            receipt=read(job/'guardian-receipt.json') if (job/'guardian-receipt.json').exists() else {}
            ownership=read(job/'ownership.json')
            conditions['reason']=receipt.get('reason')==expected
            conditions['empty_and_removed']=receipt.get('cleanup',{}).get('empty') is True and receipt.get('cleanup',{}).get('cgroup_removed') is True
            conditions['unrelated_sentinel_unaffected']=sentinel.poll() is None
            conditions['limits_readback']=all(ownership['configured'].get(k)==v for k,v in ownership['expected'].items())
            conditions['live_limits_stable']=all(receipt.get('raw_final',{}).get(k)==v for k,v in ownership['expected'].items())
            conditions['swap_zero']=receipt.get('raw_final',{}).get('memory.swap.current')=='0'
            identities=[read(path) for path in job.glob('*identity.json')]
            leaf=ownership['cgroup'].removeprefix('/sys/fs/cgroup')+'/'+ownership['service']
            conditions['pid_membership_and_process_limits']=bool(identities) and all(x['cgroup'].strip()=='0::'+leaf and x['rlimit_as']==[256*1024**2]*2 for x in identities)
            conditions['service_oom_group']= (job/'service-cgroup.json').exists() and read(job/'service-cgroup.json')['memory.oom.group']=='1'
            if expected=='Completed':
                conditions['workload_exit_code']=(job/'workload-exit.json').exists() and read(job/'workload-exit.json')['exit_code']==(7 if mode=='nonzero' else 0)
            if mode=='cpu':
                cpu=kv(receipt['raw_final']['cpu.stat'])
                conditions['actual_cpu_throttled']=cpu['nr_throttled']>0 and cpu['throttled_usec']>0
                # Allow short initial quota burst; cpu.stat is aggregate, not parent RSS/time.
                conditions['aggregate_cpu_ceiling']=cpu['usage_usec'] <= (receipt['elapsed_seconds']+0.5)*1.6*1000000
            if mode=='process-limit':
                x=read(job/'process-limit.json');conditions['actual_process_as_refusal']=x['large_mapping_refused'] and x['hard_raise_denied']
            if mode=='oom':
                x=kv(receipt['raw_final']['memory.events']);conditions['actual_oom_events']=x['oom']>0 and x['oom_kill']>0
            if mode=='orphan':
                conditions['orphan_was_created']=(job/'orphan-identity.json').exists()
            lock_ok=False
            for _ in range(40):
                with (E/'top-level.lock').open('r+') as stream:
                    try:
                        fcntl.flock(stream.fileno(),fcntl.LOCK_EX|fcntl.LOCK_NB);lock_ok=True
                    except BlockingIOError:pass
                if lock_ok:break
                time.sleep(0.05)
            conditions['lock_released']=lock_ok
            row=dict(id=rid,mode=mode,argv=argv,controller_exit_code=p.returncode,expected_reason=expected,
                     actual_reason=receipt.get('reason'),checks=conditions,passed=all(conditions.values()),
                     ownership_sha256=sha(job/'ownership.json'),guardian_receipt_sha256=sha(job/'guardian-receipt.json') if receipt else None)
            rows.append(row)
            print(json.dumps(dict(mode=mode,id=rid,passed=row['passed'],failed=[k for k,v in conditions.items() if not v])),flush=True)
            if not row['passed']:break
    finally:
        sentinel.terminate()
        sentinel.wait(timeout=3)
    value=dict(status='Passed' if len(rows)==len(MODES) and all(x['passed'] for x in rows) else 'Failed',
        scope='Small <=96MiB fixtures only; not heavy Rust executor acceptance',Required=len(MODES),Executed=len(rows),Passed=sum(x['passed'] for x in rows),Failed=sum(not x['passed'] for x in rows),NotExecuted=len(MODES)-len(rows),
        cases=rows,runner_sha256=sha(W/'tools/linux-controlled-runner.py'),fixture_sha256=sha(W/'tools/linux-runner-fixtures.py'),policy_sha256=sha(W/'engineering/policies/linux-resource-policy-v1.json'),
        earlier_attempts=['first-live-normal.json: setup rejected unsupported MemoryOOMGroup; payload not started','normal-after-systemd-adaptation.json: LimitDrift detected; own group killed and removed','qualification-results.json: 1 failed normal fixture due to missing optional leaf cpu.max; 10 not executed'],
        earlier_failures_not_overwritten=True,qualification_is_not_pilot_or_gate_authorization=True,
        untested=['PIDs exhaustion (readback and membership only)','actual host memory reserve crossing','actual disk-growth/reserve crossing','guardian itself killed','production-size build/test limits'],
        disk_enforcement='Sampled watchdog + RLIMIT_FSIZE, not kernel aggregate disk quota')
    with (E/'qualification-results-v2.json').open('x') as f:json.dump(value,f,indent=2)
    print(json.dumps({k:v for k,v in value.items() if k not in ['cases']}),flush=True)
    return 0 if value['status']=='Passed' else 1

if __name__=='__main__':raise SystemExit(main())
