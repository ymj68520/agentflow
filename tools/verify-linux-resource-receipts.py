"""Separate evidence comparator. Does not import the runner or launch workloads.

Same coding agent, different process; not independent third-party certification.
"""
import hashlib
import json
from pathlib import Path
import subprocess

W=Path(__file__).resolve().parents[1]
E=W.parent/'evidence/phase-linux-resource-qualification'
MODES=['normal','nonzero','membership','cpu','process-limit','oom','timeout','cancel','orphan','repeat','controller-crash']

def read(p):return json.loads(p.read_bytes())
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def kv(t):return {k:int(v) for k,v in (line.split() for line in t.splitlines())}

def main():
    checks=[]
    def check(section,name,value):checks.append(dict(section=section,id=name,passed=bool(value)))
    q=read(E/'qualification-results-v2.json')
    check('Quality & Evidence','fixed-complete-small-roster',[r['mode'] for r in q['cases']]==MODES and [q[k] for k in ['Required','Executed','Passed','Failed','NotExecuted']]==[11,11,11,0,0])
    check('Quality & Evidence','source-version-binding',q['runner_sha256']==sha(W/'tools/linux-controlled-runner.py') and q['fixture_sha256']==sha(W/'tools/linux-runner-fixtures.py'))
    for r in q['cases']:
        job=E/'runs'/r['id'];a=read(job/'guardian-receipt.json');meta=read(job/'ownership.json')
        check('Quality & Evidence',r['mode']+'/raw-receipt-hash',sha(job/'guardian-receipt.json')==r['guardian_receipt_sha256'] and sha(job/'ownership.json')==r['ownership_sha256'])
        want={'oom':'OOM','timeout':'Timeout','cancel':'UserCancelled','controller-crash':'ControllerLost'}.get(r['mode'],'Completed')
        check('Execution Safety',r['mode']+'/actual-classification',a['reason']==want)
        check('Execution Safety',r['mode']+'/cleanup',a['cleanup']['empty'] and a['cleanup']['cgroup_removed'] and not Path(meta['cgroup']).exists())
        expected_quota=str(meta['online_cpus']*5000)+' 100000'
        check('Execution Safety',r['mode']+'/readbacks',a['raw_final']['cpu.max']==expected_quota and a['raw_final']['memory.max']=='100663296' and a['raw_final']['memory.swap.max']=='0' and a['raw_final']['memory.swap.current']=='0' and a['raw_final']['pids.max']=='32')
        ids=[read(p) for p in job.glob('*identity.json')]+[read(p) for p in job.glob('child-*.json')]
        leaf='0::'+meta['cgroup'].removeprefix('/sys/fs/cgroup')+'/'+meta['service']
        check('Execution Safety',r['mode']+'/all-recorded-descendant-memberships',bool(ids) and all(x['cgroup'].strip()==leaf and x['rlimit_as']==[268435456,268435456] for x in ids))
        check('Execution Safety',r['mode']+'/host-reserves-observed',all(s['host_available']>=8589934592 and s['disk_free']>=21474836480 and s['disk_growth']<=67108864 for s in a['samples']))
        if r['mode']=='cpu':
            stats=kv(a['raw_final']['cpu.stat'])
            check('Execution Safety','cpu-real-throttle',stats['nr_throttled']>0 and stats['throttled_usec']>0 and stats['usage_usec']<(a['elapsed_seconds']+0.5)*meta['online_cpus']*0.05*1000000)
        elif r['mode']=='oom':
            stats=kv(a['raw_final']['memory.events'])
            check('Execution Safety','oom-machine-events',stats['oom']>0 and stats['oom_kill']>0 and int(a['raw_final']['memory.peak'])<=100663296)
        elif r['mode']=='process-limit':
            x=read(job/'process-limit.json')
            check('Execution Safety','hard-AS-refusal-and-no-raise',x['large_mapping_refused'] and x['hard_raise_denied'] and x['hard']==268435456)
        elif r['mode']=='nonzero':
            check('Quality & Evidence','real-negative-exit',read(job/'workload-exit.json')['exit_code']==7)
        elif r['mode']=='controller-crash':
            check('Execution Safety','actual-controller-sigkill',r['controller_exit_code']==-9 and not (job/'controller-receipt.json').exists())
    registry=read(W/'spec/registry/specifications.json')
    check('Specification','registered-source-pins-unchanged',all(sha(W/x['normative_source']['logical_path'])==x['normative_source']['content_digest']['hex'] for x in registry['specifications']))
    g=['git','-c','core.hooksPath=/dev/null','-c','core.fsmonitor=false','-c','core.packedGitWindowSize=8m','-c','core.packedGitLimit=32m']
    changed=subprocess.check_output(g+['diff','--name-only','90756b046a03432f6415148fcc9116a274e84830'],cwd=W).decode().splitlines()
    allowed=lambda p:p=='engineering/policies/linux-resource-policy-v1.json' or p.startswith('docs/implementation/phase-linux-resource-qualification-') or p in ['tools/linux-controlled-runner.py','tools/linux-runner-fixtures.py','tools/test-linux-controlled-runner.py','tools/qualify-linux-controlled-runner.py','tools/verify-linux-resource-receipts.py']
    check('Specification','no-existing-product-semantic-change',all(allowed(p) for p in changed))
    policy=read(W/'engineering/policies/linux-resource-policy-v1.json')
    approval=read(E/'human-approval.json')
    check('Quality & Evidence','approval-scope-binding',approval['policy_sha256']==sha(W/'engineering/policies/linux-resource-policy-v1.json') and approval['approved_scope']=='qualification' and not policy['scopes']['pilot']['Approved'] and not policy['scopes']['gates']['Approved'])
    check('Quality & Evidence','prior-failed-runs-retained',read(E/'first-live-normal.json')['controller_exit']==1 and read(E/'normal-after-systemd-adaptation.json')['controller_exit']==0 and read(E/'qualification-results.json')['Failed']==1)
    check('Execution Safety','missing-production-proof-disclosed',len(q['untested'])>=5 and 'not kernel' in q['disk_enforcement'])
    print(json.dumps(dict(status='Passed' if all(c['passed'] for c in checks) else 'Failed',checks=checks,
        review_identity='Same coding agent, separate read-only process; not third-party certification',
        SpecificationReview='Passed within unchanged-source scope',
        ExecutionSafetyReview='Partial: 11 small cases verified, production-size and untested controls NOT qualified',
        QualityEvidenceReview='Passed within recorded small-fixture scope',
        production_executor_qualified=False,compile_authorized=False),indent=2))
    return 0 if all(x['passed'] for x in checks) else 1

if __name__=='__main__':raise SystemExit(main())
