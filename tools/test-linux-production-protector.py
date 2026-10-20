"""Checkpoint A static checks. No service, kill or Cargo invocation."""
import ast
import copy
import importlib.util
import json
from pathlib import Path
W=Path(__file__).resolve().parents[1]
s=importlib.util.spec_from_file_location('protector',W/'tools/linux-production-protector.py')
m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
rows=[]
def check(n,v):rows.append(dict(id=n,passed=bool(v)))
for text,want in [('0-31',32),('0-1,4-5',4),('1,3,5',3)]:check('online/'+text,m.online_cpus(text)==want)
for n in [1,4,16,32,64]:check('cpu/'+str(n),m.cpu_max(n)==str(n*5000)+' 100000')
for free,reserve,growth,want in [(3,4,0,False),(4,4,0,True),(6,4,3,False),(7,4,3,True)]:check('disk-admission/'+str((free,reserve,growth)),m.disk_admission(free,reserve,growth)==want)
for mode in ['pids','threads','disk-preflight','disk-growth','disk-free','guardian-handoff']:
    limits=m.authorize(mode);check('new-only-approval/'+mode,limits['pids']==8 and limits['disk_growth_bytes']==33554432 and limits['task_memory_bytes']==100663296)
for key in [*m.GATES,'normal','oom','shell','../escape']:
    try:m.authorize(key);ok=False
    except PermissionError:ok=True
    check('deny-unapproved/'+key,ok)
original=m.read
for case in ['absent-proof','bad-implementation','bad-policy','failed-proof']:
    def fake(p,case=case):
        if Path(p).name=='guardian-handoff-proof.json':
            if case=='absent-proof':raise FileNotFoundError('not qualified')
            return dict(passed=case!='failed-proof',executor_sha256='0'*64 if case=='bad-implementation' else m.sha(m.__file__),policy_sha256='0'*64 if case=='bad-policy' else m.sha(m.POLICY))
        return original(p)
    m.read=fake
    try:m.authorize('guardian-kill');ok=False
    except (PermissionError,FileNotFoundError):ok=True
    check('death-needs-prior-proof/'+case,ok)
    m.read=original
text=Path(m.__file__).read_text()
for item in ['pidfd_send_signal','pidfd_open','GuardianDied','GuardianHandoff','GUARDIAN_TAKEOVER_NOT_QUALIFIED',
             'guardian-injection.json','guardian-handoff-proof.json','test_output_cleanup','output_growth','simulated_free_bytes',
             'HostDiskReserve','ProjectDiskGrowth','SimulatedDiskReserve','memory.max','memory.swap.max','pids.events',
             'memory.peak','all_descendants','cgroup_inode','cgroup.kill','LOCK_EX|fcntl.LOCK_NB',
             'phase-linux-resource-qualification/top-level.lock','ChildProcessError','GUARDIAN_DEADLINE']:
    check('structure/'+item,item in text)
for name in ['linux-production-protector.py','linux-production-fixtures.py']:
    tree=ast.parse((W/'tools'/name).read_text())
    check('no-shell/'+name,not any(isinstance(x,ast.keyword) and x.arg=='shell' and isinstance(x.value,ast.Constant) and x.value.value is True for x in ast.walk(tree)))
check('full-gates-never-admitted',m.read(m.POLICY)['full_gates_authorized'] is False)
check('disk-risk-not-accepted',m.read(m.POLICY)['disk_monitor_risk']['Approved'] is False)
check('pilot-not-approved',m.read(m.POLICY)['pilot']['Approved'] is False)
check('fixed-pilot-roster',m.GATES['pilot']==['cargo','check','--locked','--offline','-j','1','-p','af-core'])
check('no-shared-unit-budget-setting',"'set-property','--runtime',slice_name" in text)
check('growth-sampled-not-quota','hard kernel disk quota' in text)
print(json.dumps(dict(Required=len(rows),Executed=len(rows),Passed=sum(x['passed'] for x in rows),Failed=sum(not x['passed'] for x in rows),cases=rows,scope='Static/admission/structure only; not live qualification'),indent=2))
assert all(x['passed'] for x in rows)
