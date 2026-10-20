"""Project-only cgroup v2 runner. No shell and no arbitrary argv interface.

Uses systemd's user manager to enter a NEW owned slice/service, because an SSH
session outside the delegation cannot legally migrate its PIDs across that boundary.
The workload tree is bounded independently of the out-of-cgroup control plane.
This is an engineering protector for trusted commands, NOT a hostile same-UID sandbox.
"""
import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import resource
import select
import signal
import subprocess
import sys
import time
import uuid

W = Path(__file__).resolve().parents[1]
ROOT = W.parent
E = ROOT / 'evidence/phase-linux-production-protector-pilot'
POLICY = W / 'engineering/policies/linux-production-protector-v1.json'
CASES = ('pids','threads','disk-preflight','disk-growth','disk-free','guardian-handoff','guardian-kill')
GATES = {
 'pilot': ['cargo','check','--locked','--offline','-j','1','-p','af-core'],
 'fmt': ['cargo','fmt','--all','--','--check'],
 'clippy': ['cargo','clippy','--locked','--offline','-j','1','--all-targets','--all-features','--','-D','warnings'],
 'test': ['cargo','test','--locked','--offline','--all','-j','1'],
 'spec-check': ['cargo','run','--locked','--offline','-j','1','-p','af-spec','--bin','spec-check','--','check'],
}
CGROOT = Path('/sys/fs/cgroup')


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def save(path, value):
    with Path(path).open('x', encoding='utf8') as stream:
        json.dump(value, stream, indent=2)
        stream.write('\n')


def read(path):
    return json.loads(Path(path).read_bytes())


def online_cpus(text):
    ids = set()
    for part in text.strip().split(','):
        if '-' in part:
            lo, hi = map(int, part.split('-'))
            if lo > hi:
                raise ValueError('CPU range')
            ids.update(range(lo, hi+1))
        else:
            ids.add(int(part))
    if not ids:
        raise ValueError('No online CPUs')
    return len(ids)


def cpu_max(count, percent=5):
    if type(count) is not int or count <= 0 or percent != 5:
        raise ValueError('Unapproved CPU policy')
    return f'{count * 100000 * percent // 100} 100000'


def authorize(mode):
    policy=read(POLICY)
    approval=read(E/'human-approval.json')
    if approval.get('policy_sha256') != sha(POLICY) or approval.get('scope') != 'Checkpoint A new small fault tests only':
        raise PermissionError('POLICY_APPROVAL_BINDING')
    if mode not in CASES or not policy['qualification']['Approved']:
        raise PermissionError('NOT_ADMITTED: no Pilot/full-gate authorization')
    if mode=='guardian-kill':
        proof=read(E/'guardian-handoff-proof.json')
        if not (proof.get('passed') is True and proof.get('executor_sha256')==sha(__file__) and proof.get('policy_sha256')==sha(POLICY)):
            raise PermissionError('GUARDIAN_TAKEOVER_NOT_QUALIFIED')
    return policy['qualification']['Proposed']


def ctl(args, check=True):
    p = subprocess.run(['systemctl','--user',*args], capture_output=True, timeout=10,
        env=dict(os.environ, LC_ALL='C', SYSTEMD_COLORS='0'))
    row = dict(argv=['systemctl','--user',*args], exit_code=p.returncode,
               stdout=p.stdout.decode('utf8','replace'), stderr=p.stderr.decode('utf8','replace'))
    if check and p.returncode:
        raise RuntimeError(row)
    return row


def props(unit):
    text = ctl(['show',unit,'--property=ControlGroup','--property=InvocationID','--property=ActiveState',
                '--property=Result','--property=ExecMainCode','--property=ExecMainStatus'])['stdout']
    return dict(line.split('=',1) for line in text.splitlines() if '=' in line)


def delegated_root():
    return CGROOT / f'user.slice/user-{os.getuid()}.slice/user@{os.getuid()}.service'


def checked_group(meta):
    path = Path(meta['cgroup'])
    expected = delegated_root() / meta['slice']
    if path != expected or path.is_symlink() or path.stat().st_ino != meta['cgroup_inode']:
        raise RuntimeError('OWNERSHIP_CGROUP_IDENTITY')
    return path


def snapshot(path):
    fields = ['cpu.max','cpu.stat','memory.max','memory.current','memory.peak',
              'memory.swap.max','memory.swap.current','memory.oom.group','memory.events','memory.pressure',
              'pids.max','pids.current','pids.events','cgroup.events']
    return {key:(path/key).read_text().strip() for key in fields if (path/key).is_file()}


def kv(text):
    return {k:int(v) for k,v in (line.split() for line in text.splitlines())}


def disk_admission(free,reserve,growth=0):
    if any(type(v) is not int or v<0 for v in [free,reserve,growth]):raise ValueError('Invalid disk metric')
    return free>=reserve+growth


def host_available():
    return int(next(line.split()[1] for line in Path('/proc/meminfo').read_text().splitlines() if line.startswith('MemAvailable:'))) * 1024


def allocated(path):
    return measured_roots([Path(path)])


def measured_roots(roots):
    count=0;seen=set()
    for root in roots:
        if root.is_symlink():raise RuntimeError('OUTPUT_ROOT_SYMLINK')
        if not root.exists():continue
        for base,dirs,files in os.walk(root,followlinks=False):
            dirs[:]=[n for n in dirs if n!='.git']
            for name in [*dirs,*files]:
                p=Path(base)/name
                if p.is_symlink():raise RuntimeError('OUTPUT_SYMLINK')
            for name in files:
                p=Path(base)/name
                try:st=p.stat()
                except FileNotFoundError:continue
                key=(st.st_dev,st.st_ino)
                if key not in seen:count+=st.st_blocks*512;seen.add(key)
    return count


def managed_roots(job):
    return [W, ROOT/'cache/cargo-linux-primary-v1', ROOT/'build/linux-primary-v1',
            ROOT/'tmp/production-protector',job]


def process_tree(group):
    rows=[]
    for node in [group,*[p for p in group.rglob('*') if p.is_dir()]]:
        if (node/'cgroup.procs').is_file():
            for pid in (node/'cgroup.procs').read_text().split():
                p=Path('/proc')/pid
                try:
                    rows.append(dict(pid=int(pid),cgroup=(p/'cgroup').read_text(),
                                     threads=sorted(int(t.name) for t in (p/'task').iterdir())))
                except FileNotFoundError:pass
    return rows


def reclaim_test_output(job):
    output=job/'output';before=allocated(output);files=[]
    free_before=os.statvfs(ROOT).f_bavail*os.statvfs(ROOT).f_frsize
    if output.exists():
        assert output.resolve()==job.resolve()/'output' and not output.is_symlink()
        for p in output.iterdir():
            assert p.is_file() and not p.is_symlink() and p.stat().st_uid==os.getuid() and p.stat().st_nlink==1
            files.append(dict(name=p.name,bytes=p.stat().st_size,allocated_bytes=p.stat().st_blocks*512,sha256=sha(p)))
            p.unlink()  # Only the approved, bounded temporary qualification payload.
        output.rmdir()
    return dict(allocated_before=before,allocated_after=allocated(output),files=files,
                filesystem_free_before=free_before,filesystem_free_after=os.statvfs(ROOT).f_bavail*os.statvfs(ROOT).f_frsize,
                note='Global free-space changes can include other projects; recovered owned allocated blocks are the isolated metric')


def takeover(meta,job,wait_status):
    result=dict(reason='GuardianHandoff' if meta['mode']=='guardian-handoff' else 'GuardianDied',
                guardian_wait_status=wait_status,guardian_exit_code=os.waitstatus_to_exitcode(wait_status),
                raw_final=snapshot(checked_group(meta)),all_descendants=process_tree(checked_group(meta)),
                cleanup=cleanup(meta,job))
    save(job/'takeover-receipt.json',result)
    return result


def cleanup(meta, job):
    group = checked_group(meta)
    # Names are unguessable per-run identities; only these two units are operated on.
    # cgroup.kill is used only after exact path + inode validation.
    before = snapshot(group)
    descendants=process_tree(group)
    if kv((group/'cgroup.events').read_text()).get('populated'):
        (group/'cgroup.kill').write_text('1')
    commands = [ctl(['stop',meta['service']],False)]
    for _ in range(50):
        if kv((group/'cgroup.events').read_text()).get('populated') == 0:
            break
        time.sleep(0.02)
    empty = kv((group/'cgroup.events').read_text()).get('populated') == 0
    after = snapshot(group)
    service = props(meta['service'])
    commands += [ctl(['stop',meta['slice']],False), ctl(['reset-failed',meta['service']],False),
                 ctl(['revert',meta['slice']],False)]
    return dict(before=before, after=after, empty=empty, cgroup_removed=not group.exists(),
                service_properties=service, unit_commands=commands,all_descendants=descendants,test_output_cleanup=reclaim_test_output(job))


def guardian(meta, job, pipe, lockfd, limits):
    # Independent guardian survives the controller's SIGKILL. Inherited flock
    # stays held until owned task cleanup, so a dead controller cannot admit a peer.
    resource.setrlimit(resource.RLIMIT_AS,(128*1024**2,)*2)
    started = time.monotonic()
    peak = 0
    samples = []
    reason = 'Unknown'
    group = checked_group(meta)
    roots=managed_roots(job)
    baseline=measured_roots(roots)
    output_baseline=allocated(job/'output')
    try:
        while True:
            snap = snapshot(group)
            peak = max(peak,int(snap['memory.current']))
            available = host_available()
            disk_free = os.statvfs(ROOT).f_bavail * os.statvfs(ROOT).f_frsize
            growth=max(0,measured_roots(roots)-baseline)
            output_growth=max(0,allocated(job/'output')-output_baseline)
            simulated_free=6*1024**2-output_growth if meta['mode']=='disk-free' else None
            samples.append(dict(seconds=time.monotonic()-started,memory_current=int(snap['memory.current']),
                                host_available=available,disk_free=disk_free,disk_growth=growth,output_growth=output_growth,simulated_free_bytes=simulated_free))
            if len(samples)>10000:
                raise RuntimeError('Sample bound')
            ready,_,_ = select.select([pipe],[],[],0)
            if ready and os.read(pipe,1024) == b'':
                reason='ControllerLost';break
            if online_cpus(Path('/sys/devices/system/cpu/online').read_text()) != meta['online_cpus']:
                reason='OnlineCPUChanged';break
            if any(snap.get(k) != v for k,v in meta['expected'].items()):
                reason='LimitDrift';break
            if kv(snap['memory.events']).get('oom_kill',0):
                reason='OOM';break
            if (job/'cancel.request').exists():
                if read(job/'cancel.request')['token'] != meta['token']:
                    raise RuntimeError('Cancel ownership')
                reason='UserCancelled';break
            if available < limits['host_reserve_bytes']:
                reason='HostMemoryReserve';break
            if not disk_admission(disk_free,limits['disk_free_reserve_bytes']):
                reason='HostDiskReserve';break
            if meta['mode']=='disk-free' and not disk_admission(simulated_free,4*1024**2):
                reason='SimulatedDiskReserve';break
            if meta['mode']=='disk-growth' and output_growth > 2*1024**2:
                reason='ProjectDiskGrowth';break
            if meta['mode']=='guardian-handoff' and (job/'workload-ready.json').exists():
                save(job/'handoff-request.json',dict(reason='Orderly monitor handoff rehearsal',raw=snap,descendants=process_tree(group)))
                os._exit(0)
            if growth > limits['disk_growth_bytes']:
                reason='DiskGrowth';break
            if (job/'workload-exit.json').exists():
                reason='Completed';break
            if (job/'launch.json').exists() and len(samples)%5 == 0:
                state=props(meta['service'])
                if state.get('ActiveState') not in ['active','activating','reloading']:
                    reason='UnexpectedServiceExit';break
            if time.monotonic()-started >= limits['wall_seconds']:
                reason='Timeout';break
            time.sleep(limits['monitor_period_seconds'])
        final = snapshot(group)
        result = dict(reason=reason,observed_peak_current_bytes=peak,raw_final=final,samples=samples,
                      elapsed_seconds=time.monotonic()-started,cleanup=cleanup(meta,job))
        save(job/'guardian-receipt.json',result)
        os._exit(0)
    except BaseException as exc:
        error=dict(error=repr(exc),reason='GuardianError')
        try:
            error['cleanup']=cleanup(meta,job)
        except BaseException as cleanup_error:
            error['cleanup_error']=repr(cleanup_error)
        save(job/'guardian-error.json',error)
        os._exit(2)


def run(mode, runid):
    resource.setrlimit(resource.RLIMIT_AS,(128*1024**2,)*2)
    limits = dict(authorize(mode))
    if mode == 'timeout':
        limits['wall_seconds'] = 1  # Stricter than the approved <=30s ceiling.
    assert str(uuid.UUID(runid)) == runid
    assert W == Path.home()/'projects/agent-r5g2-linux-validation/workspace'
    assert os.getuid() == 1001 and not E.is_symlink()
    os.umask(0o077)
    os.nice(19)
    if host_available() < limits['host_reserve_bytes'] + limits['task_memory_bytes']:
        raise RuntimeError('HOST_MEMORY_ADMISSION')
    disk = os.statvfs(ROOT)
    if not disk_admission(disk.f_bavail*disk.f_frsize,limits['disk_free_reserve_bytes'],limits['disk_growth_bytes']):
        raise RuntimeError('HOST_DISK_ADMISSION')
    if mode=='disk-preflight':
        assert not disk_admission(3*1024**2,4*1024**2)
        save(E/('preflight-'+runid+'.json'),dict(reason='SimulatedDiskAdmissionDenied',actual_exit_code=None,simulated_free_bytes=3*1024**2,simulated_reserve_bytes=4*1024**2,real_free_bytes=disk.f_bavail*disk.f_frsize,source_head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=W).decode().strip(),executor_sha256=sha(__file__),policy_sha256=sha(POLICY),approval=read(E/'human-approval.json'),task_launched=False))
        return 0
    lockpath = ROOT/'evidence/phase-linux-resource-qualification/top-level.lock'
    fd = os.open(lockpath,os.O_CREAT|os.O_RDWR|os.O_NOFOLLOW,0o600)
    if os.fstat(fd).st_uid != os.getuid() or os.fstat(fd).st_nlink != 1:
        raise RuntimeError('LOCK_OWNERSHIP')
    fcntl.flock(fd,fcntl.LOCK_EX|fcntl.LOCK_NB)
    job=E/'runs'/runid
    job.mkdir(parents=True,exist_ok=False)
    stem='afl2p03'+runid.replace('-','')
    slice_name,service=stem+'.slice',stem+'.service'
    # Never modify resource settings on a shared slice or user service.
    delegated=delegated_root()
    if not os.access(delegated,os.W_OK) or not {'cpu','memory','pids'} <= set((delegated/'cgroup.controllers').read_text().split()):
        raise RuntimeError('CGROUP_DELEGATION_NOT_QUALIFIED')
    assert not (delegated/slice_name).exists()
    online=online_cpus(Path('/sys/devices/system/cpu/online').read_text())
    quota=cpu_max(online)
    commands=[ctl(['start',slice_name])]
    meta=None;guard=None;writer=None;guard_fd=None
    try:
        commands.append(ctl(['set-property','--runtime',slice_name,
            f'CPUQuota={online*5}%', 'CPUQuotaPeriodSec=100ms',
            f'MemoryMax={limits["task_memory_bytes"]}', 'MemorySwapMax=0',
            f'TasksMax={limits["pids"]}']))
        cg=CGROOT/props(slice_name)['ControlGroup'].lstrip('/')
        if cg != delegated/slice_name or not cg.is_dir() or cg.is_symlink():
            raise RuntimeError('DEDICATED_SCOPE_BINDING')
        if cg.stat().st_uid != os.getuid():
            raise RuntimeError('OWNED_SLICE_UID')
        configured=snapshot(cg)
        expected={'cpu.max':quota,'memory.max':str(limits['task_memory_bytes']),'memory.swap.max':'0','pids.max':str(limits['pids']),'memory.oom.group':'0'}
        if any(configured.get(k)!=v for k,v in expected.items()):
            raise RuntimeError(('LIMIT_READBACK',configured,expected))
        meta=dict(id=runid,token=uuid.uuid4().hex,uid=os.getuid(),mode=mode,slice=slice_name,service=service,
                  cgroup=str(cg),cgroup_inode=cg.stat().st_ino,configured=configured,expected=expected,
                  online_cpus=online,limits=limits,policy_sha256=sha(POLICY),runner_sha256=sha(__file__),
                  fixture_sha256=sha(W/'tools/linux-production-fixtures.py'),source_head=subprocess.check_output(['git','-c','core.packedGitWindowSize=8m','rev-parse','HEAD'],cwd=W).decode().strip(),approval=read(E/'human-approval.json'),disk_roots=[str(p) for p in managed_roots(job)],commands=commands,
                  process_address_space_limit=limits['process_address_space_bytes'],
                  control_plane='Controller hard128MiB from entry; guardian hard128MiB; both outside the workload slice',
                  disk_guard='Sampled guard, NOT a hard kernel disk quota')
        save(job/'ownership.json',meta)
        reader,writer=os.pipe()
        guard=os.fork()
        if guard==0:
            os.close(writer)
            guardian(meta,job,reader,fd,limits)
        os.close(reader)
        guard_fd=os.pidfd_open(guard)
        save(job/'guardian-identity.json',dict(pid=guard,starttime=Path('/proc/'+str(guard)+'/stat').read_text().split()[21],pidfd_bound=True))
        # External systemd manager starts the fixed entrypoint *inside* its slice,
        # after resource properties are installed and read back. No migration race.
        cmd=['systemd-run','--user','--no-ask-password','--quiet', '--service-type=exec',
             '--unit='+service,'--slice='+slice_name,
             '--property=RemainAfterExit=yes','--property=KillMode=control-group',
             '--property=TimeoutStopSec=2s','--property=RuntimeMaxSec='+str(limits['wall_seconds'])+'s',
             '--property=OOMPolicy=kill','--property=LimitCORE=0',
             '--property=LimitAS='+str(limits['process_address_space_bytes']),
             '--property=LimitFSIZE='+str(limits['disk_growth_bytes']),
             '--property=Nice=19','--property=WorkingDirectory='+str(W),
             '--property=StandardOutput=append:'+str(job/'stdout.log'),
             '--property=StandardError=append:'+str(job/'stderr.log'),
             str(ROOT/'venvs/linux-primary-v1/bin/python'),'-B',str(W/'tools/linux-production-fixtures.py'),mode,str(job)]
        p=subprocess.run(cmd,capture_output=True,timeout=10)
        save(job/'launch.json',dict(argv=cmd,exit_code=p.returncode,stdout=p.stdout.decode(),stderr=p.stderr.decode()))
        if p.returncode:
            raise RuntimeError('SYSTEMD_LAUNCH_FAILED')
        resource.setrlimit(resource.RLIMIT_AS,(128*1024**2,)*2)
        start=time.monotonic()
        while True:
            if mode=='guardian-kill' and (job/'workload-ready.json').exists() and not (job/'guardian-injection.json').exists():
                save(job/'guardian-injection.json',dict(signal='SIGKILL',target='Own unreaped guardian child via pidfd',pid=guard,proof_sha256=sha(E/'guardian-handoff-proof.json')))
                signal.pidfd_send_signal(guard_fd,signal.SIGKILL)
            if mode=='controller-crash' and (job/'workload-start.json').exists():
                os.kill(os.getpid(),signal.SIGKILL)
            if mode=='cancel' and (job/'workload-start.json').exists() and not (job/'cancel.request').exists():
                save(job/'cancel.request',dict(token=meta['token'],source='Fixed explicit-cancel qualification action'))
            pid,status=os.waitpid(guard,os.WNOHANG)
            if pid:
                if mode in ['guardian-handoff','guardian-kill'] and not (job/'guardian-receipt.json').exists():
                    receipt=takeover(meta,job,status)
                    save(job/'controller-receipt.json',dict(guardian_exit_code=os.waitstatus_to_exitcode(status),reason=receipt['reason'],cleanup=receipt['cleanup'],exit_code=None))
                    print(json.dumps(dict(id=runid,reason=receipt['reason'],job=str(job))),flush=True)
                    return 0
                if status!=0 or not (job/'guardian-receipt.json').exists():
                    raise RuntimeError('GUARDIAN_FAILURE')
                receipt=read(job/'guardian-receipt.json')
                save(job/'controller-receipt.json',dict(guardian_exit_code=0,reason=receipt['reason'],
                     cleanup=receipt['cleanup'],exit_code=read(job/'workload-exit.json')['exit_code'] if (job/'workload-exit.json').exists() else None))
                print(json.dumps(dict(id=runid,reason=receipt['reason'],job=str(job))),flush=True)
                return 0
            if time.monotonic()-start > limits['wall_seconds']+20:
                raise RuntimeError('GUARDIAN_DEADLINE')
            time.sleep(0.05)
    except BaseException as exc:
        save(job/'controller-error.json',dict(error=repr(exc)))
        if writer is not None:
            os.close(writer);writer=None
        if guard:
            for _ in range(60):
                try:
                    pid,status=os.waitpid(guard,os.WNOHANG)
                except ChildProcessError:
                    pid=guard
                if pid:break
                time.sleep(0.05)
            else:
                signal.pidfd_send_signal(guard_fd,signal.SIGKILL)
                os.waitpid(guard,0)
        if meta and Path(meta['cgroup']).exists():
            save(job/'fallback-cleanup.json',cleanup(meta,job))
        elif meta is None:
            ctl(['stop',service],False);ctl(['stop',slice_name],False);ctl(['revert',slice_name],False)
        raise
    finally:
        if writer is not None:
            os.close(writer)
        os.close(fd)
        if guard_fd is not None:os.close(guard_fd)


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('mode',choices=[*CASES,*GATES])
    parser.add_argument('--id',required=True)
    args=parser.parse_args()
    return run(args.mode,args.id)


if __name__=='__main__':
    raise SystemExit(main())
