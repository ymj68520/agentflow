"""Run only a whitelisted gate via the project-owned protected adapter."""
import argparse
import json
import os
from pathlib import Path
import resource
import subprocess
import uuid

W=Path(__file__).resolve().parents[1]
E=W/'target/thesis-fast-dev'

def main():
    resource.setrlimit(resource.RLIMIT_AS,(512*1024**2,)*2)
    os.nice(19)
    parser=argparse.ArgumentParser()
    parser.add_argument('gate',choices=['format','fmt','clippy','test','spec'])
    gate=parser.parse_args().gate
    runid=str(uuid.uuid4())
    base=[str(W.parent/'venvs/linux-primary-v1/bin/python'),'-B',str(W/'tools/linux-thesis-dev.py')]
    prepared=subprocess.run([*base,'prepare','--gate',gate,'--id',runid],cwd=W,capture_output=True,timeout=30)
    if prepared.returncode:
        print(prepared.stderr.decode(),flush=True)
        return prepared.returncode
    attempt=E/gate/runid
    print(prepared.stdout.decode(),flush=True)
    with (attempt/'controller.stdout.log').open('xb') as out,(attempt/'controller.stderr.log').open('xb') as err:
        proc=subprocess.run([*base,'run','--gate',gate,'--id',runid],cwd=W,stdout=out,stderr=err,timeout=5450)
    job=attempt/'runs'/runid
    result=dict(gate=gate,id=runid,controller_exit_code=proc.returncode,job=str(job))
    if (job/'workload-exit.json').exists():result['cargo']=json.loads((job/'workload-exit.json').read_bytes())
    if (job/'guardian-receipt.json').exists():
        guard=json.loads((job/'guardian-receipt.json').read_bytes())
        result['guardian']=dict(reason=guard['reason'],peak_memory=guard['raw_final']['memory.peak'],cleanup_empty=guard['cleanup']['empty'],cgroup_removed=guard['cleanup']['cgroup_removed'],extrema=guard['extrema'])
    with (attempt/'result.json').open('x') as out:json.dump(result,out,indent=2)
    print(json.dumps(result),flush=True)
    if proc.returncode:
        for name in ['cargo.stdout.log','cargo.stderr.log']:
            if (job/name).exists():print(name+':\n'+(job/name).read_text()[-25000:],flush=True)
        print((attempt/'controller.stderr.log').read_text(),flush=True)
    return proc.returncode

if __name__=='__main__':raise SystemExit(main())
