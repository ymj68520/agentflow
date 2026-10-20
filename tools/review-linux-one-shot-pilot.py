"""Post-Pilot readback only. No commands, cleanup, retries or third-party claims."""
import hashlib
import json
from pathlib import Path
W=Path(__file__).resolve().parents[1];E=W.parent/'evidence/phase-linux-one-shot-rust-pilot'
def read(p):return json.loads(p.read_bytes())
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def main():
    f=read(E/'execution-freeze.json');job=E/'runs'/f['run_id'];rows=[]
    def check(s,n,v):rows.append(dict(section=s,id=n,passed=bool(v)))
    a=read(E/'approval-matrix.json');check('Quality & Evidence','three-real-approvals',all(a[k]['Approved'] is True for k in ['A','B','C']) and a['B']['choice']=='B2')
    consumed=read(E/'pilot-launch-consumed.json') if (E/'pilot-launch-consumed.json').exists() else None
    workload=read(job/'workload-exit.json') if (job/'workload-exit.json').exists() else None
    g=read(job/'guardian-receipt.json') if (job/'guardian-receipt.json').exists() else None
    check('Quality & Evidence','one-claim',consumed is not None and consumed['id']==f['run_id'])
    check('Quality & Evidence','exact-argv',consumed is not None and consumed['argv']==[f['toolchain']['cargo']['path'],'check','--locked','--offline','-j','1','-p','af-core'])
    check('Quality & Evidence','execution-source-hashes',consumed is not None and consumed['source_head']==f['source_head'] and consumed['executor_sha256']==sha(W/'tools/linux-one-shot-pilot.py') and consumed['policy_sha256']==sha(W/'engineering/policies/linux-one-shot-rust-pilot-v1.json'))
    ready=read(job/'resource-policy-readbacks.json');raw=ready['root'];count=read(job/'ownership.json')['online_cpus']
    check('Execution Safety','production-not-small',raw['memory.max']=='3221225472' and raw['pids.max']=='128' and raw['memory.swap.max']=='0' and raw['cpu.max']==str(count*5000)+' 100000')
    check('Execution Safety','real-process-AS',ready['service']['rlimit_as']==[2147483648,2147483648] and '2147483648' in ready['kernel_service_limits'])
    check('Execution Safety','guardian-ready-before-release',(job/'guardian-ready.json').stat().st_mtime_ns<(job/'release.json').stat().st_mtime_ns)
    check('Execution Safety','host-admitted',read(job/'host-admission.json')['status']=='ADMITTED')
    check('Execution Safety','raw-resource-receipt',g is not None)
    if g:
        check('Execution Safety','empty-removed-owned-group',g['cleanup']['empty'] and g['cleanup']['cgroup_removed'])
        check('Execution Safety','all-inode-samples-present',all('free_inodes' in x['disk'] and 'scan_seconds' in x['disk'] for x in g['samples']))
        check('Execution Safety','disk-not-hard-quota',read(W/'engineering/policies/linux-one-shot-rust-pilot-v1.json')['disk_mode'].startswith('B2'))
        check('Quality & Evidence','real-termination-not-promoted',g['reason']=='Completed' if workload and workload.get('actual_exit_code')==0 else g['reason']!='Unknown')
    if workload:
        check('Quality & Evidence','stdout-sha',sha(job/'cargo.stdout.log')==workload.get('stdout_sha256'))
        check('Quality & Evidence','stderr-sha',sha(job/'cargo.stderr.log')==workload.get('stderr_sha256'))
    before=read(E/'source-integrity-before.json');after=read(E/'source-integrity-after.json')
    check('Specification','unchanged-protected-inputs',before['protected_files']==after['protected_files'])
    check('Specification','same-frozen-snapshots',before['frozen_snapshots']==after['frozen_snapshots'])
    check('Specification','same-lock-dependencies',before['cargo_lock_sha256']==after['cargo_lock_sha256'] and before['cargo_archives']==after['cargo_archives'] and before['cargo_vendor_checksums']==after['cargo_vendor_checksums'])
    check('Specification','previous-receipts-unchanged',after['old_receipts_unchanged'])
    success=bool(g and g['reason']=='Completed' and workload and workload.get('actual_exit_code')==0)
    print(json.dumps(dict(status='Passed' if all(x['passed'] for x in rows) else 'Failed',checks=rows,pilot_success=success,actual_exit_code=workload.get('actual_exit_code') if workload else None,termination=g['reason'] if g else 'Missing receipt',reviewer='Same coding agent, separate read-only process, not third-party certification',full_linux_acceptance=False),indent=2))
    return 0 if all(x['passed'] for x in rows) else 1
if __name__=='__main__':raise SystemExit(main())
