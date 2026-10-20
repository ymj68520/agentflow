"""Fixed tiny qualification fixtures, not a general execution endpoint."""
import errno
import json
import mmap
import os
from pathlib import Path
import resource
import sys
import time
import threading


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
    meta=json.loads((job/'ownership.json').read_bytes())
    group=Path(meta['cgroup'])
    if mode in ['pids','threads']:
        before={n:(group/n).read_text() for n in ['pids.max','pids.current','pids.events']}
        children=[];threads=[];failure=None
        rd,wr=os.pipe();event=threading.Event()
        threading.stack_size(1024*1024)
        try:
            for i in range(16):
                try:
                    if mode=='pids':
                        pid=os.fork()
                        if pid==0:
                            os.close(wr)
                            save(job/('child-'+str(i)+'-identity.json'),identity())
                            os.read(rd,1)
                            os._exit(0)
                        children.append(pid)
                    else:
                        t=threading.Thread(target=event.wait)
                        t.start();threads.append(t)
                except (OSError,RuntimeError) as exc:
                    failure=dict(type=type(exc).__name__,errno=getattr(exc,'errno',None),message=str(exc));break
            after={n:(group/n).read_text() for n in ['pids.max','pids.current','pids.events']}
            save(job/'pids-exhaustion.json',dict(mode=mode,before=before,at_failure=after,
                  failure=failure,created_children=children,created_threads=len(threads),identity=identity()))
        finally:
            os.close(wr);os.close(rd);event.set()
            for pid in children:os.waitpid(pid,0)
            for t in threads:t.join(timeout=2)
        return 0 if failure is not None else 5
    if mode in ['disk-growth','disk-free']:
        output=job/'output';output.mkdir()
        with (output/'growth.bin').open('xb',buffering=0) as f:
            for i in range(16):
                f.write(bytes([i])*262144);os.fsync(f.fileno());time.sleep(0.1)
        time.sleep(60)
        return 5
    if mode in ['guardian-handoff','guardian-kill']:
        pid=os.fork()
        if pid==0:
            save(job/'descendant-identity.json',identity())
            time.sleep(60);os._exit(0)
        while not (job/'descendant-identity.json').exists():time.sleep(0.01)
        save(job/'workload-ready.json',dict(child_pid=pid,identity=identity()))
        time.sleep(60)
        return 5
    raise ValueError('Not allowlisted')


def main():
    mode,raw=sys.argv[1:3]
    job=Path(raw).resolve()
    root=Path.home()/'projects/agent-r5g2-linux-validation/evidence/phase-linux-production-protector-pilot/runs'
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
        env.update(PYTHONDONTWRITEBYTECODE='1',PYTHONNOUSERSITE='1',MALLOC_ARENA_MAX='1')
        os.execve(sys.executable,[sys.executable,'-B',__file__,mode,str(job),'payload'],env)
    pid,status,usage=os.wait4(child,0)
    save(job/'workload-exit.json',dict(pid=pid,wait_status=status,exit_code=os.waitstatus_to_exitcode(status),
         user_seconds=usage.ru_utime,system_seconds=usage.ru_stime,process_maxrss_bytes=usage.ru_maxrss*1024))
    while True:
        time.sleep(1)


if __name__=='__main__':
    raise SystemExit(main())
