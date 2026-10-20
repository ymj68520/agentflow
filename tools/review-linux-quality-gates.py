"""Round1 receipt readback. Same agent, separate process, no launches/retries."""
import hashlib
import json
from pathlib import Path
W=Path(__file__).resolve().parents[1];E=W.parent/'evidence/phase-linux-full-engineering-gates'
def read(p):return json.loads(p.read_bytes())
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def main():
    shared=read(E/'shared-freeze.json');rows=[];results={}
    def check(s,n,v):rows.append(dict(section=s,id=n,passed=bool(v)))
    for gate in ['fmt','clippy']:
        base=E/gate;f=read(base/'execution-freeze.json');job=base/'runs'/f['run_id'];a=read(base/'approval-matrix.json')
        check('Quality & Evidence',gate+'/per-gate-approvals',all(a[k]['Approved'] is True for k in ['A','B','C']) and a['B']['choice']=='B2')
        check('Specification',gate+'/same-shared-head',f['source_head']==shared['source_head'] and f['git_tree']==shared['git_tree'])
        consumed=read(base/'gate-launch-consumed.json') if (base/'gate-launch-consumed.json').exists() else None
        workload=read(job/'workload-exit.json') if (job/'workload-exit.json').exists() else None
        g=read(job/'guardian-receipt.json') if (job/'guardian-receipt.json').exists() else None
        check('Quality & Evidence',gate+'/one-consumed-launch',consumed is not None and consumed['id']==f['run_id'])
        if consumed:check('Quality & Evidence',gate+'/exact-argv',consumed['argv']==[f['toolchain']['cargo']['path'],*a['C']['argv'][1:]])
        ready=read(job/'resource-policy-readbacks.json') if (job/'resource-policy-readbacks.json').exists() else None
        check('Execution Safety',gate+'/effective-limits',ready is not None and ready['root']['memory.max']=='3221225472' and ready['root']['pids.max']=='128' and ready['root']['memory.swap.max']=='0' and ready['service']['rlimit_as']==[2147483648]*2)
        check('Execution Safety',gate+'/guardian-receipt',g is not None)
        if g:
            check('Execution Safety',gate+'/cleaned',g['cleanup']['empty'] and g['cleanup']['cgroup_removed'])
            check('Quality & Evidence',gate+'/stream-hashes',sha(job/'resource-samples.jsonl')==g['samples_sha256'] and sha(job/'process-trees.jsonl')==g['process_trees_sha256'])
            with (job/'resource-samples.jsonl').open() as stream:count=sum(1 for line in stream if isinstance(json.loads(line),dict))
            check('Quality & Evidence',gate+'/stream-count',count==g['sample_count'])
        if workload and 'stdout_sha256' in workload:
            check('Quality & Evidence',gate+'/cargo-log-hashes',sha(job/'cargo.stdout.log')==workload['stdout_sha256'] and sha(job/'cargo.stderr.log')==workload['stderr_sha256'])
        results[gate]=dict(reason=g['reason'] if g else 'Missing guardian receipt',actual_exit_code=workload.get('actual_exit_code') if workload else None,executed_source_head=f['source_head'])
    before=read(E/'source-integrity-before.json');after=read(E/'source-integrity-after.json')
    check('Specification','protected-inputs-unchanged',before['protected_files']==after['protected_files'])
    check('Specification','locked-inputs-unchanged',before['cargo_lock_sha256']==after['cargo_lock_sha256'] and before['cargo_archives']==after['cargo_archives'] and before['cargo_vendor_checksums']==after['cargo_vendor_checksums'])
    check('Specification','frozen-snapshots-unchanged',before['frozen_snapshots']==after['frozen_snapshots'])
    check('Quality & Evidence','Round2-unapproved',all(not read(E/gate/'approval-matrix.json')['C']['Approved'] and not (E/gate/'gate-launch-consumed.json').exists() for gate in ['test','spec-check']))
    passed=all(v['reason']=='Completed' and v['actual_exit_code']==0 for v in results.values())
    print(json.dumps(dict(status='Passed' if all(x['passed'] for x in rows) else 'Failed',checks=rows,round1=results,round1_passed=passed,round1_credit='FORMAT_AND_CLIPPY_GATES_PASSED' if passed else 'ROUND1_FAILED',round2_authorized=False,reviewer='Same coding agent, separate readback process, not third-party certification'),indent=2))
    return 0 if all(x['passed'] for x in rows) else 1
if __name__=='__main__':raise SystemExit(main())
