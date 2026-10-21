"""Continuous trusted-project developer adapter; fixed approved command whitelist.

Reuses the immutable PIVOT-03 qualified ownership/cleanup/measurement core. Cargo
is gated behind actual service + guardian readbacks and a permanent O_EXCL claim.
"""
import argparse
import datetime
import fcntl
import hashlib
import importlib.util
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

W=Path(__file__).resolve().parents[1]
ROOT=W.parent
PHASE=W/'target/studio-model-dev'
E=PHASE
GATE=None
POLICY=W/'engineering/policies/studio-model-dev-v1.json'
BASE=W/'tools/linux-production-protector.py'
BASE_SHA='28b3ffbf05a1815f80ada6c8ffe98cee621fb7805b84ad24deefbfca2e393349'
COMMANDS={'format':['fmt','--all'],'fmt':['fmt','--all','--','--check'],'clippy':['clippy','--locked','--offline','-j','1','--all-targets','--all-features','--','-D','warnings'],'test':['test','--locked','--offline','--workspace','-j','1'],'spec':['run','--locked','--offline','-j','1','-p','af-spec','--bin','spec-check','--','check']}
COMMANDS.update({'lock':['generate-lockfile','--offline'],'demo-test':['test','--locked','--offline','-j','1','-p','af-demo'],'serve':['run','--locked','--offline','-j','1','-p','af-demo']})
COMMAND=None
def configure(gate,runid):
    global E,GATE,COMMAND
    if gate not in COMMANDS:raise PermissionError('COMMAND_NOT_WHITELISTED')
    GATE=gate;E=PHASE/gate/runid;COMMAND=COMMANDS[gate]
LIMITS=dict(cpu_percent_of_online_host=10,process_address_space_bytes=8589934592,task_memory_bytes=6442450944,swap_bytes=0,pids=128,host_reserve_bytes=8589934592,wall_seconds=5400,disk_growth_bytes=12884901888,disk_free_reserve_bytes=21474836480,cargo_jobs=1,rust_test_threads=2,concurrent_tasks=1,monitor_period_seconds=0.1)

def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
assert sha(BASE)==BASE_SHA,'QUALIFIED_CORE_DRIFT'
spec=importlib.util.spec_from_file_location('qualified_p03',BASE)
core=importlib.util.module_from_spec(spec);spec.loader.exec_module(core)
save,read=core.save,core.read

def now():return datetime.datetime.now(datetime.timezone.utc).isoformat()
def git(args):
    return subprocess.check_output(['git','-c','core.hooksPath=/dev/null','-c','core.fsmonitor=false','-c','core.packedGitWindowSize=8m','-c','core.packedGitLimit=32m',*args],cwd=W,env=dict(os.environ,GIT_CONFIG_NOSYSTEM='1',GIT_CONFIG_GLOBAL='/dev/null'),timeout=15).decode().strip()
def source_inputs():
    # Current Rust build inputs, not a repeated scan of archived worktrees/docs.
    paths=git(['ls-files','--cached','--others','--exclude-standard']).splitlines()
    return {p:sha(W/p) for p in sorted(set(paths)) if p.endswith(('.rs','.toml','.js','.css','.html')) or (p.startswith('crates/af-demo/examples/') and p.endswith('.json')) or p=='Cargo.lock'}

def authorize(runid):
    assert str(uuid.UUID(runid))==runid and os.getuid()==1001
    assert W==Path.home()/'projects/agent-r5g2-linux-validation/workspace'
    policy=read(POLICY);a=read(PHASE/'approval.json');f=read(E/'execution-freeze.json')
    if not (a.get('repeat_commands_approved') is True and a.get('B2_approved') is True and a['policy_sha256']==sha(POLICY) and policy['limits']==LIMITS and policy['commands']==COMMANDS):raise PermissionError('DEVELOPMENT_APPROVAL_BINDING')
    if f['gate']!=GATE or f['run_id']!=runid or f['executor_sha256']!=sha(__file__) or f['policy_sha256']!=sha(POLICY):raise PermissionError('ATTEMPT_BINDING')
    if source_inputs()!=f['source_inputs']:raise PermissionError('SOURCE_CHANGED_BEFORE_RELEASE')
    for p,h in f['immutable_files'].items():
        if sha(Path(p))!=h:raise PermissionError('IMMUTABLE_INPUT_DRIFT: '+p)
    return f

def prepare(runid):
    E.mkdir(parents=True,exist_ok=False)
    session=read(PHASE/'session.json');bins=session['toolchain'];tmp=PHASE/'tmp'/runid
    tmp.mkdir(parents=True,exist_ok=False)
    for name in ['tmp','home','cache','config','data']:(tmp/name).mkdir()
    forbidden=[k for k in os.environ if k.startswith('CARGO_PROFILE_') or k in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER'] if os.environ.get(k)]
    if forbidden:raise PermissionError('AMBIENT_BUILD_OVERRIDE: '+repr(forbidden))
    env=dict(HOME=str(tmp/'home'),TMPDIR=str(tmp/'tmp'),TMP=str(tmp/'tmp'),TEMP=str(tmp/'tmp'),XDG_CACHE_HOME=str(tmp/'cache'),XDG_CONFIG_HOME=str(tmp/'config'),XDG_DATA_HOME=str(tmp/'data'),PATH=str(Path(bins['cargo']['path']).parent)+':'+str(ROOT/'venvs/linux-primary-v1/bin')+':/usr/bin:/bin',LANG='C.UTF-8',LC_ALL='C.UTF-8',CARGO_HOME=str(ROOT/'cache/cargo-linux-primary-v1'),CARGO_TARGET_DIR=str(ROOT/'build/linux-primary-v1'),CARGO_BUILD_JOBS='1',RUST_TEST_THREADS='2',CARGO_NET_OFFLINE='true',CARGO_TERM_COLOR='never',RUSTC=bins['rustc']['path'],RUSTDOC=bins['rustdoc']['path'],RUSTFMT=bins['rustfmt']['path'],RUSTUP_AUTO_INSTALL='0',RUSTUP_NO_UPDATE_CHECK='1')
    if GATE=='serve' and (W/'target/private/model.json').exists():env['AF_DEMO_MODEL_CONFIG']=str(W/'target/private/model.json')
    files=[Path(__file__),BASE,POLICY,PHASE/'approval.json',W/'Cargo.lock',ROOT/'cache/cargo-linux-primary-v1/config.toml',Path('/usr/bin/curl')]+[Path(x['path']) for x in bins.values()]
    f=dict(gate=GATE,run_id=runid,source_head=git(['rev-parse','HEAD']),git_tree=git(['rev-parse','HEAD^{tree}']),worktree_status=git(['status','--porcelain']),source_inputs=source_inputs(),executor_sha256=sha(__file__),policy_sha256=sha(POLICY),toolchain=bins,immutable_files={str(p):sha(p) for p in files},write_roots=session['write_roots'],job_tmp_root=str(tmp),effective_env=env)
    save(E/'execution-freeze.json',f)
    save(E/'approval-matrix.json',read(PHASE/'approval.json'))
    print(json.dumps(dict(status='Prepared',gate=GATE,id=runid,source_head=f['source_head'],dirty=bool(f['worktree_status']))))
    return 0

def disk_sample(roots,baseline):
    scan=time.monotonic();usage=core.measured_roots(roots);fs=os.statvfs(ROOT)
    return dict(monotonic=time.monotonic(),scan_seconds=time.monotonic()-scan,allocated_bytes=usage,growth_bytes=max(0,usage-baseline),free_bytes=fs.f_bavail*fs.f_frsize,free_inodes=fs.f_favail)

def roots_for(job,freeze):return [Path(p) for p in freeze['write_roots']]+[job]

def guardian_release_check(pidfd, identity, ready, freeze):
    if select.select([pidfd],[],[],0)[0]:raise RuntimeError('GUARDIAN_DEAD_AT_RELEASE')
    if ready['pid']!=identity['pid'] or ready['executor_sha256']!=sha(__file__) or ready['source_head']!=freeze['source_head']:
        raise RuntimeError('GUARDIAN_RELEASE_IDENTITY')
    return dict(pid=ready['pid'],pidfd_alive=True,checked_monotonic=time.monotonic(),executor_sha256=ready['executor_sha256'])

def cleanup_succeeded(receipt):
    return receipt.get('empty') is True and receipt.get('cgroup_removed') is True

def guardian(meta,job,pipe,lockfd,freeze):
    resource.setrlimit(resource.RLIMIT_AS,(128*1024**2,)*2)
    group=core.checked_group(meta);roots=roots_for(job,freeze);baseline=read(PHASE/'session.json')['disk_baseline_bytes']
    started=time.monotonic();reason='Unknown';detected=None;last_tree=None;sample_count=0;tree_count=0;last_sample=None
    extrema=dict(memory_current_peak=0,pids_sampled_peak=0,disk_growth_sampled_peak=0,min_free_bytes=None,min_free_inodes=None,min_mem_available=None,max_sample_gap_seconds=0,max_scan_seconds=0)
    def write_line(stream,value):stream.write(json.dumps(value,separators=(',',':'))+'\n');stream.flush()
    try:
        with (job/'resource-samples.jsonl').open('x') as out,(job/'process-trees.jsonl').open('x') as trees:
            while True:
                raw=core.snapshot(group);disk=disk_sample(roots,baseline);available=core.host_available();tree=core.process_tree(group)
                if tree!=last_tree:write_line(trees,dict(monotonic=time.monotonic(),processes=tree));tree_count+=1;last_tree=tree
                sample=dict(seconds=time.monotonic()-started,host_mem_available=available,pids_current=int(raw['pids.current']),memory_current=int(raw['memory.current']),disk=disk)
                write_line(out,sample);sample_count+=1
                extrema['memory_current_peak']=max(extrema['memory_current_peak'],sample['memory_current']);extrema['pids_sampled_peak']=max(extrema['pids_sampled_peak'],sample['pids_current']);extrema['disk_growth_sampled_peak']=max(extrema['disk_growth_sampled_peak'],disk['growth_bytes'])
                for key,value in [('min_free_bytes',disk['free_bytes']),('min_free_inodes',disk['free_inodes']),('min_mem_available',available)]:extrema[key]=value if extrema[key] is None else min(extrema[key],value)
                if last_sample is not None:extrema['max_sample_gap_seconds']=max(extrema['max_sample_gap_seconds'],disk['monotonic']-last_sample)
                last_sample=disk['monotonic'];extrema['max_scan_seconds']=max(extrema['max_scan_seconds'],disk['scan_seconds'])
                if any(raw.get(k)!=v for k,v in meta['expected'].items()):reason='LimitDrift'
                elif core.online_cpus(Path('/sys/devices/system/cpu/online').read_text())!=meta['online_cpus']:reason='OnlineCPUChanged'
                elif core.kv(raw['memory.events']).get('oom_kill',0):reason='OOM'
                elif raw['memory.swap.current']!='0':reason='SwapUsage'
                elif available<LIMITS['host_reserve_bytes']:reason='HostMemoryReserve'
                elif disk['free_bytes']<LIMITS['disk_free_reserve_bytes']:reason='HostDiskReserve'
                elif disk['free_inodes']<=0:reason='NoAvailableInodes'
                elif disk['growth_bytes']>LIMITS['disk_growth_bytes']:reason='DiskGrowth'
                elif (job/'cancel.request').exists():
                    if read(job/'cancel.request')['token']!=meta['token']:raise RuntimeError('CANCEL_IDENTITY')
                    reason='UserCancelled'
                elif (job/'workload-exit.json').exists():reason='Completed'
                elif time.monotonic()-started>=LIMITS['wall_seconds']:reason='Timeout'
                ready,_,_=select.select([pipe],[],[],0)
                if reason=='Unknown' and ready and os.read(pipe,1024)==b'':reason='ControllerLost'
                if reason!='Unknown':detected=time.monotonic();break
                if not (job/'guardian-ready.json').exists():save(job/'guardian-ready.json',dict(pid=os.getpid(),raw=raw,first_disk_sample=disk,source_head=freeze['source_head'],executor_sha256=sha(__file__)))
                time.sleep(LIMITS['monitor_period_seconds'])
        final=core.snapshot(group);before_cleanup=time.monotonic();cleanup=core.cleanup(meta,job)
        save(job/'guardian-receipt.json',dict(reason=reason,start_utc=meta['start_utc'],end_utc=now(),elapsed_seconds=time.monotonic()-started,raw_final=final,sample_count=sample_count,process_tree_event_count=tree_count,extrema=extrema,
             samples_sha256=sha(job/'resource-samples.jsonl'),process_trees_sha256=sha(job/'process-trees.jsonl'),
             detection_monotonic=detected,cleanup_started_monotonic=before_cleanup,detection_to_cleanup_seconds=before_cleanup-detected,cleanup_finished_monotonic=time.monotonic(),cleanup=cleanup,disk_after_task=disk_sample(roots,baseline)))
        os._exit(0)
    except BaseException as exc:
        error=dict(reason='GuardianError',error=repr(exc))
        try:error['cleanup']=core.cleanup(meta,job)
        except BaseException as clean:error['cleanup_error']=repr(clean)
        save(job/'guardian-error.json',error);os._exit(2)

def service(runid):
    freeze=authorize(runid);job=E/'runs'/runid;meta=read(job/'ownership.json');group=core.checked_group(meta)
    expected_member='0::'+meta['cgroup'].removeprefix('/sys/fs/cgroup')+'/'+meta['service']
    if Path('/proc/self/cgroup').read_text().strip()!=expected_member:raise PermissionError('SERVICE_MEMBERSHIP')
    limit=list(resource.getrlimit(resource.RLIMIT_AS))
    if limit!=[8589934592]*2:raise RuntimeError('RLIMIT_AS_READBACK')
    leaf=Path('/sys/fs/cgroup')/expected_member.split('::',1)[1].lstrip('/')
    if (leaf/'memory.oom.group').read_text().strip()!='1':raise RuntimeError('SERVICE_OOM_GROUP')
    env=freeze['effective_env'];argv=[freeze['toolchain']['cargo']['path'],*COMMAND]
    save(job/'service-ready.json',dict(pid=os.getpid(),cgroup=expected_member,rlimit_as=limit,actual_argv=argv,cwd=str(W),environment=env,cgroup_readbacks=core.snapshot(group),starttime=Path('/proc/self/stat').read_text().split()[21]))
    deadline=time.monotonic()+30
    while not (job/'release.json').exists():
        if time.monotonic()>deadline:raise RuntimeError('NO_ADMISSION_RELEASE')
        time.sleep(0.02)
    release=read(job/'release.json')
    if release['token']!=meta['token'] or release['freeze_sha256']!=sha(E/'execution-freeze.json'):raise PermissionError('RELEASE_BINDING')
    authorize(runid)
    # Replay guard for this attempt only. New attempts use new UUIDs under the
    # same human-approved continuous task; failed attempt logs stay intact.
    save(E/'gate-launch-consumed.json',dict(gate=GATE,id=runid,utc=now(),argv=argv,source_head=freeze['source_head'],policy_sha256=sha(POLICY),executor_sha256=sha(__file__)))
    start=now();begin=time.monotonic()
    save(job/'cargo-started.json',dict(start_utc=start,argv=argv,cwd=str(W),environment=env,rlimit_as=limit,pid=os.getpid()))
    actual=None
    try:
        with (job/'cargo.stdout.log').open('xb') as out,(job/'cargo.stderr.log').open('xb') as err:
            p=subprocess.Popen(argv,cwd=W,env=env,stdout=out,stderr=err)
            save(job/'cargo-pid.json',dict(pid=p.pid))
            actual=p.wait()
        result=dict(actual_exit_code=actual,start_utc=start,end_utc=now(),seconds=time.monotonic()-begin,argv=argv,cwd=str(W),stdout_sha256=sha(job/'cargo.stdout.log'),stderr_sha256=sha(job/'cargo.stderr.log'))
    except BaseException as exc:result=dict(actual_exit_code=actual,start_utc=start,end_utc=now(),startup_error=repr(exc))
    save(job/'workload-exit.json',result)
    while True:time.sleep(1)

def run(runid):
    resource.setrlimit(resource.RLIMIT_AS,(128*1024**2,)*2);os.umask(0o077);os.nice(19)
    freeze=authorize(runid)
    if (E/'gate-launch-reserved.json').exists() or (E/'gate-launch-consumed.json').exists():raise PermissionError('ONE_SHOT_ALREADY_RESERVED_OR_CONSUMED')
    if core.host_available()<LIMITS['host_reserve_bytes']+LIMITS['task_memory_bytes']:raise RuntimeError('HOST_MEMORY_ADMISSION')
    fs=os.statvfs(ROOT)
    if not core.disk_admission(fs.f_bavail*fs.f_frsize,LIMITS['disk_free_reserve_bytes'],LIMITS['disk_growth_bytes']) or fs.f_favail<=0:raise RuntimeError('HOST_DISK_INODE_ADMISSION')
    lock=ROOT/'evidence/phase-linux-resource-qualification/top-level.lock';fd=os.open(lock,os.O_RDWR|os.O_NOFOLLOW)
    if os.fstat(fd).st_uid!=os.getuid() or os.fstat(fd).st_nlink!=1:raise RuntimeError('LOCK_IDENTITY')
    fcntl.flock(fd,fcntl.LOCK_EX|fcntl.LOCK_NB)
    units=core.ctl(['list-units','--all','--no-legend','--plain','--no-pager','afl2p02*','afl2p03*','afl2p04*','afl2p05*','afl2p06*','afl2p07*','afdev*'])
    if units['stdout'].strip():raise RuntimeError('OWNED_UNIT_RESIDUE')
    job=E/'runs'/runid;job.mkdir(parents=True,exist_ok=False)
    stem='afdev'+runid.replace('-','');slice_name,svc=stem+'.slice',stem+'.service'
    online=core.online_cpus(Path('/sys/devices/system/cpu/online').read_text());meta=None;guard=None;guardfd=None;writer=None
    core.ctl(['start',slice_name])
    try:
        core.ctl(['set-property','--runtime',slice_name,f'CPUQuota={online*10}%','CPUQuotaPeriodSec=100ms','MemoryMax=6442450944','MemorySwapMax=0','TasksMax=128'])
        group=Path('/sys/fs/cgroup')/core.props(slice_name)['ControlGroup'].lstrip('/')
        if group!=core.delegated_root()/slice_name or group.is_symlink() or group.stat().st_uid!=os.getuid():raise RuntimeError('OWNED_SCOPE_IDENTITY')
        raw=core.snapshot(group)
        required={'cpu.max','cpu.stat','memory.max','memory.current','memory.peak','memory.events','memory.swap.max','memory.swap.current','pids.max','pids.current','pids.events'}
        if not required.issubset(raw):raise RuntimeError('MANDATORY_RESOURCE_INTERFACE_MISSING')
        expected={'cpu.max':f'{online*10000} 100000','memory.max':'6442450944','memory.swap.max':'0','pids.max':'128','memory.oom.group':'0'}
        if any(raw.get(k)!=v for k,v in expected.items()) or raw['memory.swap.current']!='0':raise RuntimeError('PRODUCTION_LIMIT_READBACK')
        meta=dict(id=runid,token=uuid.uuid4().hex,uid=os.getuid(),mode=GATE,slice=slice_name,service=svc,cgroup=str(group),cgroup_inode=group.stat().st_ino,expected=expected,configured=raw,online_cpus=online,source_head=freeze['source_head'],git_tree=freeze['git_tree'],executor_sha256=sha(__file__),qualified_core_sha256=BASE_SHA,policy_sha256=sha(POLICY),start_utc=now())
        save(job/'ownership.json',meta);reader,writer=os.pipe();guard=os.fork()
        if guard==0:os.close(writer);guardian(meta,job,reader,fd,freeze)
        os.close(reader);guardfd=os.pidfd_open(guard)
        save(job/'guardian-identity.json',dict(pid=guard,pidfd_bound=True))
        cmd=['systemd-run','--user','--no-ask-password','--quiet','--service-type=exec','--unit='+svc,'--slice='+slice_name,
             '--property=RemainAfterExit=yes','--property=KillMode=control-group','--property=TimeoutStopSec=2s','--property=RuntimeMaxSec=5400s','--property=OOMPolicy=kill','--property=LimitCORE=0','--property=LimitAS=8589934592','--property=LimitFSIZE=12884901888','--property=Nice=19','--property=WorkingDirectory='+str(W),
             '--property=StandardOutput=append:'+str(job/'service.stdout.log'),'--property=StandardError=append:'+str(job/'service.stderr.log'),
             str(ROOT/'venvs/linux-primary-v1/bin/python'),'-B',str(Path(__file__)),'service','--gate',GATE,'--id',runid]
        p=subprocess.run(cmd,capture_output=True,timeout=10);save(job/'launch.json',dict(argv=cmd,actual_exit_code=p.returncode,stdout=p.stdout.decode(),stderr=p.stderr.decode()))
        if p.returncode:raise RuntimeError('SERVICE_LAUNCH_FAILED')
        wait=time.monotonic()
        while not ((job/'service-ready.json').exists() and (job/'guardian-ready.json').exists()):
            pid,status=os.waitpid(guard,os.WNOHANG)
            if pid:raise RuntimeError('GUARDIAN_DIED_BEFORE_ADMISSION')
            if time.monotonic()-wait>15:raise RuntimeError('SERVICE_GUARDIAN_NOT_READY')
            time.sleep(0.05)
        service_ready=read(job/'service-ready.json');guard_ready=read(job/'guardian-ready.json')
        kernel_limits=Path('/proc/'+str(service_ready['pid'])+'/limits').read_text()
        as_line=next(line for line in kernel_limits.splitlines() if line.startswith('Max address space'))
        if as_line.split()[-3:-1]!=['8589934592','8589934592']:raise RuntimeError('KERNEL_RLIMIT_AS_READBACK')
        live_ancestors=[]
        for ancestor in group.parents:
            if not ancestor.is_relative_to(Path('/sys/fs/cgroup')):break
            live_ancestors.append(dict(path=str(ancestor),limits={n:(ancestor/n).read_text().strip() for n in ['cpu.max','memory.max','memory.swap.max','pids.max'] if (ancestor/n).exists()}))
        if service_ready['rlimit_as']!=[8589934592]*2 or guard_ready['executor_sha256']!=sha(__file__):raise RuntimeError('READY_IDENTITY')
        if any(core.snapshot(group).get(k)!=v for k,v in expected.items()):raise RuntimeError('PRE_CARGO_LIMIT_DRIFT')
        authorize(runid)
        host=core.host_available();fs=os.statvfs(ROOT)
        if host<LIMITS['host_reserve_bytes']+LIMITS['task_memory_bytes'] or not core.disk_admission(fs.f_bavail*fs.f_frsize,LIMITS['disk_free_reserve_bytes'],LIMITS['disk_growth_bytes']) or fs.f_favail<=0:raise RuntimeError('FINAL_HOST_ADMISSION')
        save(E/'gate-launch-reserved.json',dict(gate=GATE,id=runid,utc=now(),scope='One release only; never retry after a consumed startup'))
        save(job/'resource-policy-readbacks.json',dict(root=core.snapshot(group),service=service_ready,guardian=guard_ready,ancestors=live_ancestors,kernel_service_limits=kernel_limits,controller_rlimit_as=list(resource.getrlimit(resource.RLIMIT_AS))))
        save(job/'host-admission.json',dict(status='ADMITTED',source_head=freeze['source_head'],git_tree=freeze['git_tree'],host_mem_available=host,free_bytes=fs.f_bavail*fs.f_frsize,free_inodes=fs.f_favail,approval_matrix_sha256=sha(E/'approval-matrix.json'),freeze_sha256=sha(E/'execution-freeze.json'),resource_readback_sha256=sha(job/'resource-policy-readbacks.json')))
        release_check=guardian_release_check(guardfd,read(job/'guardian-identity.json'),read(job/'guardian-ready.json'),freeze)
        save(job/'guardian-release-readback.json',release_check)
        guardian_release_check(guardfd,read(job/'guardian-identity.json'),read(job/'guardian-ready.json'),freeze)
        save(job/'release.json',dict(token=meta['token'],freeze_sha256=sha(E/'execution-freeze.json'),utc=now()))
        print(json.dumps(dict(status='ADMITTED',gate=GATE,id=runid,source_head=freeze['source_head'],command=['cargo',*COMMAND])),flush=True)
        started=time.monotonic();last=started
        while True:
            pid,status=os.waitpid(guard,os.WNOHANG)
            if pid:
                if status!=0 or not (job/'guardian-receipt.json').exists():raise RuntimeError('GUARDIAN_FAILED_CONTROLLER_TAKEOVER')
                receipt=read(job/'guardian-receipt.json');actual=read(job/'workload-exit.json') if (job/'workload-exit.json').exists() else None
                if not cleanup_succeeded(receipt['cleanup']):raise RuntimeError('GUARDIAN_CLEANUP_INCOMPLETE_CONTROLLER_TAKEOVER')
                save(job/'controller-receipt.json',dict(reason=receipt['reason'],guardian_exit=0,workload=actual,cleanup=receipt['cleanup'],end_utc=now()))
                print(json.dumps(dict(reason=receipt['reason'],workload=actual,job=str(job))),flush=True)
                return 0 if receipt['reason']=='Completed' and actual and actual.get('actual_exit_code')==0 else 1
            if time.monotonic()-last>25:print(json.dumps(dict(status='Running',gate=GATE,seconds=time.monotonic()-started)),flush=True);last=time.monotonic()
            if time.monotonic()-started>LIMITS['wall_seconds']+20:raise RuntimeError('GUARDIAN_DEADLINE')
            time.sleep(0.05)
    except BaseException as exc:
        save(job/'controller-error.json',dict(error=repr(exc),utc=now()))
        if writer is not None:os.close(writer);writer=None
        if guard:
            for _ in range(60):
                try:pid,status=os.waitpid(guard,os.WNOHANG)
                except ChildProcessError:pid=guard
                if pid:break
                time.sleep(0.05)
            else:signal.pidfd_send_signal(guardfd,signal.SIGKILL);os.waitpid(guard,0)
        if meta and Path(meta['cgroup']).exists():save(job/'fallback-cleanup.json',core.cleanup(meta,job))
        elif meta is None:core.ctl(['stop',svc],False);core.ctl(['stop',slice_name],False);core.ctl(['revert',slice_name],False)
        raise
    finally:
        if writer is not None:os.close(writer)
        if guardfd is not None:os.close(guardfd)
        os.close(fd)

def cancel(runid):
    assert str(uuid.UUID(runid))==runid
    job=E/'runs'/runid;meta=read(job/'ownership.json')
    core.checked_group(meta)
    save(job/'cancel.request',dict(token=meta['token'],source='Explicit operator cancel request',utc=now()))

def main():
    p=argparse.ArgumentParser();p.add_argument('action',choices=['prepare','run','service','cancel']);p.add_argument('--gate',choices=list(COMMANDS),required=True);p.add_argument('--id',required=True);a=p.parse_args();configure(a.gate,a.id)
    return prepare(a.id) if a.action=='prepare' else run(a.id) if a.action=='run' else service(a.id) if a.action=='service' else cancel(a.id)
if __name__=='__main__':raise SystemExit(main())
