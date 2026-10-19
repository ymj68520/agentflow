"""Windows-only, fail-closed resource envelope for ONE phase-local command.

The worker blocks on stdin until its Job Object limits have been installed.
No heavy child is launched before assignment. Closing the parent kills its job.
"""
import ctypes as C,json,os,subprocess,sys,time
from ctypes import wintypes as W
from pathlib import Path
R=Path(__file__).resolve().parent.parent
if sys.argv[1:2]==['--worker']:
    if sys.stdin.buffer.read(1)!=b'G':sys.exit(78)
    sys.exit(subprocess.call(sys.argv[2:],cwd=R))
assert os.name=='nt'
class Basic(C.Structure):
    _fields_=[('ProcessTime',C.c_longlong),('JobTime',C.c_longlong),('Flags',W.DWORD),('MinWS',C.c_size_t),('MaxWS',C.c_size_t),('Active',W.DWORD),('Affinity',C.c_size_t),('Priority',W.DWORD),('Scheduling',W.DWORD)]
class IO(C.Structure):
    _fields_=[(n,C.c_ulonglong) for n in ['ReadOps','WriteOps','OtherOps','ReadBytes','WriteBytes','OtherBytes']]
class Extended(C.Structure):
    _fields_=[('Basic',Basic),('IO',IO),('ProcessMemory',C.c_size_t),('JobMemory',C.c_size_t),('PeakProcess',C.c_size_t),('PeakJob',C.c_size_t)]
class Rate(C.Structure):_fields_=[('Flags',W.DWORD),('Rate',W.DWORD)]
class Memory(C.Structure):
    _fields_=[('Length',W.DWORD),('Load',W.DWORD)]+[(n,C.c_ulonglong) for n in ['TotalPhys','AvailPhys','TotalPage','AvailPage','TotalVirtual','AvailVirtual','AvailExtended']]
k=C.WinDLL('kernel32',use_last_error=True)
k.CreateJobObjectW.argtypes=[C.c_void_p,W.LPCWSTR];k.CreateJobObjectW.restype=W.HANDLE
k.SetInformationJobObject.argtypes=[W.HANDLE,C.c_int,C.c_void_p,W.DWORD];k.SetInformationJobObject.restype=W.BOOL
k.QueryInformationJobObject.argtypes=[W.HANDLE,C.c_int,C.c_void_p,W.DWORD,C.c_void_p];k.QueryInformationJobObject.restype=W.BOOL
k.AssignProcessToJobObject.argtypes=[W.HANDLE,W.HANDLE];k.AssignProcessToJobObject.restype=W.BOOL
k.TerminateJobObject.argtypes=[W.HANDLE,W.UINT];k.TerminateJobObject.restype=W.BOOL
k.CloseHandle.argtypes=[W.HANDLE];k.CloseHandle.restype=W.BOOL
k.GlobalMemoryStatusEx.argtypes=[C.POINTER(Memory)];k.GlobalMemoryStatusEx.restype=W.BOOL
def check(ok):
    if not ok:raise C.WinError(C.get_last_error())
def memory():
    m=Memory();m.Length=C.sizeof(m);check(k.GlobalMemoryStatusEx(C.byref(m)));return m
assert C.sizeof(Basic)==64 and C.sizeof(Extended)==144 and C.sizeof(Rate)==8
name=sys.argv[1];args=sys.argv[2:];assert args
assert name.replace('-','').replace('_','').isalnum()
smoke=name.startswith('safety-smoke')
process_mib=64 if smoke else 768;job_mib=128 if smoke else 1536
min_free=8192*1024*1024;before=memory()
assert before.AvailPhys>=min_free and before.Load<80,'Insufficient foreground memory reserve; no command launched'
prefix=R/'target'/('phase-04c1b4c4r3c2r4g3-safe-'+name)
lock=R/'target/phase-04c1b4c4r3c2r4g3-safe-active.json'
if lock.exists():
    prior=json.loads(lock.read_bytes())
    # A stale lock after reboot is never automatically removed or bypassed.
    raise RuntimeError(('Safety envelope already active or interrupted; inspect its PID before removing the lock',prior))
job=k.CreateJobObjectW(None,None);check(job)
limits=Extended();limits.Basic.Flags=0x2000|0x100|0x200|0x20
limits.Basic.Priority=0x4000;limits.ProcessMemory=process_mib*1024*1024;limits.JobMemory=job_mib*1024*1024
check(k.SetInformationJobObject(job,9,C.byref(limits),C.sizeof(limits)))
rate=Rate(0x1|0x4,500);check(k.SetInformationJobObject(job,15,C.byref(rate),C.sizeof(rate)))
queried=Extended();actual_rate=Rate()
check(k.QueryInformationJobObject(job,9,C.byref(queried),C.sizeof(queried),None))
check(k.QueryInformationJobObject(job,15,C.byref(actual_rate),C.sizeof(actual_rate),None))
assert queried.ProcessMemory==limits.ProcessMemory and queried.JobMemory==limits.JobMemory and queried.Basic.Flags&limits.Basic.Flags==limits.Basic.Flags and actual_rate.Rate==500 and actual_rate.Flags&5==5
environment=dict(os.environ,PYTHONUTF8='1',RUST_TEST_THREADS='1',CARGO_BUILD_JOBS='1')
started=time.time();proc=None;cancel=None;peak_job=peak_process=0;minimum_available=before.AvailPhys
lock.write_bytes(json.dumps(dict(parent_pid=os.getpid(),name=name,command=args,started_unix=started)).encode())
try:
    with Path(str(prefix)+'.log').open('wb') as output:
        proc=subprocess.Popen([sys.executable,str(Path(__file__).resolve()),'--worker',*args],cwd=R,stdin=subprocess.PIPE,stdout=output,stderr=subprocess.STDOUT,env=environment,creationflags=0x08000000|0x4000)
        check(k.AssignProcessToJobObject(job,W.HANDLE(int(proc._handle))))
        print(json.dumps(dict(name=name,limits_verified=True,worker_pid=proc.pid,cpu_hard_cap_percent=5,process_memory_mib=process_mib,whole_job_memory_mib=job_mib,test_threads=1,active_top_level_jobs=1)),flush=True)
        proc.stdin.write(b'G');proc.stdin.close()
        while proc.poll() is None:
            m=memory();minimum_available=min(minimum_available,m.AvailPhys)
            q=Extended();check(k.QueryInformationJobObject(job,9,C.byref(q),C.sizeof(q),None));peak_job=max(peak_job,q.PeakJob);peak_process=max(peak_process,q.PeakProcess)
            if m.AvailPhys<min_free or m.Load>=80:
                cancel='foreground_memory_reserve';check(k.TerminateJobObject(job,79));break
            time.sleep(1)
        code=proc.wait()
        q=Extended();check(k.QueryInformationJobObject(job,9,C.byref(q),C.sizeof(q),None));peak_job=max(peak_job,q.PeakJob);peak_process=max(peak_process,q.PeakProcess)
except BaseException:
    if proc is not None and proc.poll() is None:
        proc.terminate();proc.wait()
    raise
finally:
    # This handle owns only this task's job; no unrelated process is targeted.
    check(k.CloseHandle(job))
    lock.unlink()
row=dict(name=name,command=args,started_unix=started,seconds=round(time.time()-started,2),exit=code,cancelled=cancel,limits_verified=True,cpu_hard_cap_percent=5,process_memory_mib=process_mib,whole_job_memory_mib=job_mib,peak_process_bytes=peak_process,peak_job_bytes=peak_job,minimum_system_available_bytes=minimum_available,test_threads=1,active_top_level_jobs=1,cargo_build_jobs_env=1,log=str(prefix)+'.log')
Path(str(prefix)+'.json').write_bytes((json.dumps(row,indent=2)+'\n').encode())
print(json.dumps(row),flush=True);sys.exit(code)
