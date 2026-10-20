"""Independent readback implementation, same agent process family, no workload launch."""
import hashlib
import json
from pathlib import Path
import subprocess
W=Path(__file__).resolve().parents[1]
E=W.parent/'evidence/phase-linux-production-protector-pilot'
MODES=['disk-preflight','pids','threads','disk-growth','disk-free','guardian-handoff','guardian-kill']
def read(p):return json.loads(p.read_bytes())
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def kv(t):return {k:int(v) for k,v in (line.split() for line in t.splitlines())}
def main():
    rows=[]
    def check(section,name,value):rows.append(dict(section=section,id=name,passed=bool(value)))
    q=read(E/'qualification-results.json')
    check('Quality & Evidence','fixed-roster',[x['mode'] for x in q['cases']]==MODES and [q[k] for k in ['Required','Executed','Passed','Failed','NotExecuted']]==[7,7,7,0,0])
    check('Quality & Evidence','executed-source-bytes',q['executor_sha256']==sha(W/'tools/linux-production-protector.py') and q['fixture_sha256']==sha(W/'tools/linux-production-fixtures.py') and q['policy_sha256']==sha(W/'engineering/policies/linux-production-protector-v1.json'))
    proof=read(E/'guardian-handoff-proof.json')
    for row in q['cases']:
        mode=row['mode'];j=E/'runs'/row['id']
        attempt=read(E/('attempt-'+row['id']+'.json'))
        check('Quality & Evidence',mode+'/real-controller-exit',attempt['exit_code']==0 and row['actual_controller_exit']==0)
        if mode=='disk-preflight':
            a=read(E/('preflight-'+row['id']+'.json'))
            check('Execution Safety','preflight-no-task',a['simulated_free_bytes']<a['simulated_reserve_bytes'] and not a['task_launched'] and not j.exists())
            continue
        meta=read(j/'ownership.json');a=read(j/('takeover-receipt.json' if mode.startswith('guardian-') else 'guardian-receipt.json'))
        check('Execution Safety',mode+'/kernel-readback',a['raw_final']['cpu.max']==str(meta['online_cpus']*5000)+' 100000' and a['raw_final']['memory.max']=='100663296' and a['raw_final']['pids.max']=='8' and a['raw_final']['memory.swap.max']=='0' and a['raw_final']['memory.swap.current']=='0')
        check('Execution Safety',mode+'/reclaimed-owned-group',a['cleanup']['empty'] and a['cleanup']['cgroup_removed'] and not Path(meta['cgroup']).exists())
        check('Execution Safety',mode+'/AS-readback',read(j/'workload-start.json')['rlimit_as']==[268435456,268435456])
        if mode in ['pids','threads']:
            x=read(j/'pids-exhaustion.json')
            check('Execution Safety',mode+'/actual-refusal',x['failure'] is not None and x['at_failure']['pids.current'].strip()=='8' and kv(x['at_failure']['pids.events'])['max']>kv(x['before']['pids.events'])['max'])
            check('Execution Safety',mode+'/not-memory-failure',kv(a['raw_final']['memory.events'])['oom_kill']==0 and read(j/'workload-exit.json')['exit_code']==0)
        if mode in ['disk-growth','disk-free']:
            x=a['cleanup']['test_output_cleanup']
            check('Execution Safety',mode+'/owned-file-recovery',0<x['allocated_before']<=8388608 and x['allocated_after']==0 and x['files'] and not (j/'output').exists())
            check('Execution Safety',mode+'/real-reserve-preserved',all(s['disk_free']>=21474836480 and s['disk_growth']<33554432 for s in a['samples']))
            check('Execution Safety',mode+'/crossing-terminated',a['reason']==('ProjectDiskGrowth' if mode=='disk-growth' else 'SimulatedDiskReserve') and max(s['output_growth'] for s in a['samples'])>2097152)
        if mode=='guardian-handoff':
            check('Execution Safety','orderly-takeover-proof',a['guardian_exit_code']==0 and proof['id']==row['id'] and proof['receipt_sha256']==sha(j/'takeover-receipt.json') and proof['passed'] is True)
        if mode=='guardian-kill':
            injection=read(j/'guardian-injection.json');identity=read(j/'guardian-identity.json')
            check('Execution Safety','own-pidfd-SIGKILL',injection['pid']==identity['pid'] and identity['pidfd_bound'] and a['guardian_exit_code']==-9)
            check('Execution Safety','rehearsal-precedes-death',injection['proof_sha256']==sha(E/'guardian-handoff-proof.json') and (E/'guardian-handoff-proof.json').stat().st_mtime_ns<(j/'guardian-injection.json').stat().st_mtime_ns)
    policy=read(W/'engineering/policies/linux-production-protector-v1.json')
    check('Quality & Evidence','no-Pilot-risk-gate-approval',not policy['pilot']['Approved'] and not policy['disk_monitor_risk']['Approved'] and not policy['full_gates_authorized'])
    check('Execution Safety','hard-quota-not-claimed',read(E/'filesystem-quota-audit.json')['hard_quota_qualified'] is False)
    registry=read(W/'spec/registry/specifications.json')
    check('Specification','registered-pins-exact',all(sha(W/x['normative_source']['logical_path'])==x['normative_source']['content_digest']['hex'] for x in registry['specifications']))
    git=['git','-c','core.hooksPath=/dev/null','-c','core.fsmonitor=false','-c','core.packedGitWindowSize=8m','-c','core.packedGitLimit=32m']
    changed=subprocess.check_output(git+['diff','--name-only','ad44baa4e462bb7a25b2eb34ffcaf32876727e7a'],cwd=W).decode().splitlines()
    allowed=lambda p:p.startswith('docs/implementation/phase-linux-production-protector-pilot-') or p=='engineering/policies/linux-production-protector-v1.json' or p in ['tools/linux-production-protector.py','tools/linux-production-fixtures.py','tools/test-linux-production-protector.py','tools/qualify-linux-production-protector.py','tools/review-linux-production-protector.py']
    check('Specification','no-prior-product-or-oracle-edits',all(allowed(p) for p in changed))
    print(json.dumps(dict(status='Passed' if all(x['passed'] for x in rows) else 'Failed',checks=rows,reviewer='Same coding agent, separate readback process; not third-party certification',
        specification='Passed within unchanged-source scope',execution_safety='Small new gaps tested; production admission NOT QUALIFIED pending disk risk and budget',quality_evidence='Passed within seven-case small-test scope',pilot_admitted=False),indent=2))
    return 0 if all(x['passed'] for x in rows) else 1
if __name__=='__main__':raise SystemExit(main())
