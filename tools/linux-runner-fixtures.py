"""Fixed tiny qualification fixtures, not a general execution endpoint."""
import errno
import json
import mmap
import os
from pathlib import Path
import resource
import sys
import time


def save(path,value):
    with path.open('x') as stream:
        json.dump(value,stream,indent=2)
        stream.flush()
        os.fsync(stream.fileno())


def identity():
    return dict(pid=os.getpid(),ppid=os.getppid(),cgroup=Path('/proc/self/cgroup').read_text(),
                rlimit_as=list(resource.getrlimit(resource.RLIMIT_AS)))


def payload(mode,job):
    save(job/'payload-identity.json',identity())
    if mode=='nonzero':
        return 7
    if mode=='process-limit':
        refused=False
        try:
            mmap.mmap(-1,512*1024**2,prot=0)
        except OSError as exc:
            refused=exc.errno==errno.ENOMEM
        hard=resource.getrlimit(resource.RLIMIT_AS)[1]
        denied=False
        try:
            resource.setrlimit(resource.RLIMIT_AS,(hard*2,hard*2))
        except (ValueError,PermissionError):
            denied=True
        save(job/'process-limit.json',dict(large_mapping_refused=refused,hard_raise_denied=denied,hard=hard))
        return 0 if refused and denied else 3
    if mode=='oom':
        data=bytearray(128*1024**2)
        for i in range(0,len(data),4096):
            data[i]=1
        return 8
    if mode in ['timeout','cancel','controller-crash']:
        time.sleep(60)
        return 8
    if mode=='orphan':
        pid=os.fork()
        if pid==0:
            save(job/'orphan-identity.json',identity())
            time.sleep(60)
            os._exit(0)
        deadline=time.monotonic()+3
        while not (job/'orphan-identity.json').exists() and time.monotonic()<deadline:
            time.sleep(0.01)
        return 0
    if mode in ['cpu','membership']:
        children=[]
        for i in range(4 if mode=='cpu' else 2):
            pid=os.fork()
            if pid==0:
                save(job/('child-'+str(i)+'.json'),identity())
                end=time.monotonic()+(4 if mode=='cpu' else 0.2)
                x=1
                while time.monotonic()<end:
                    x=(x*3+17)%1000003
                os._exit(0)
            children.append(pid)
        codes=[os.waitstatus_to_exitcode(os.waitpid(pid,0)[1]) for pid in children]
        return 0 if all(c==0 for c in codes) else 9
    if mode not in ['normal','repeat']:
        raise ValueError('Not allowlisted')
    return 0


def main():
    mode,raw=sys.argv[1:3]
    job=Path(raw).resolve()
    root=Path.home()/'projects/agent-r5g2-linux-validation/evidence/phase-linux-resource-qualification/runs'
    assert job.parent==root and not job.is_symlink()
    meta=json.loads((job/'ownership.json').read_bytes())
    expected=Path(meta['cgroup']).as_posix().removeprefix('/sys/fs/cgroup')
    member=Path('/proc/self/cgroup').read_text().strip().split('::',1)[1]
    assert member==expected+'/'+meta['service']
    assert list(resource.getrlimit(resource.RLIMIT_AS))==[meta['process_address_space_limit']]*2
    if len(sys.argv)==4 and sys.argv[3]=='payload':
        return payload(mode,job)
    os.umask(0o077)
    cg=Path('/sys/fs/cgroup')/member.lstrip('/')
    assert (cg/'memory.oom.group').read_text().strip() == '1', 'OOMPolicy=kill must bind the service group'
    save(job/'service-cgroup.json', {n:(cg/n).read_text().strip() for n in ['memory.oom.group','memory.events','memory.max','cpu.max','memory.swap.max','pids.max'] if (cg/n).is_file()})
    save(job/'workload-start.json',identity())
    child=os.fork()
    if child==0:
        env={k:v for k,v in os.environ.items() if k in ['HOME','USER','LANG','PATH','XDG_RUNTIME_DIR','DBUS_SESSION_BUS_ADDRESS']}
        env.update(PYTHONDONTWRITEBYTECODE='1',PYTHONNOUSERSITE='1')
        os.execve(sys.executable,[sys.executable,'-B',__file__,mode,str(job),'payload'],env)
    pid,status,usage=os.wait4(child,0)
    save(job/'workload-exit.json',dict(pid=pid,wait_status=status,exit_code=os.waitstatus_to_exitcode(status),
         user_seconds=usage.ru_utime,system_seconds=usage.ru_stime,process_maxrss_bytes=usage.ru_maxrss*1024))
    while True:
        time.sleep(1)


if __name__=='__main__':
    raise SystemExit(main())
