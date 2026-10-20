"""Static-only runner policy tests; no systemd/cgroup writes or workload launch."""
import ast
import copy
import importlib.util
import json
from pathlib import Path

W=Path(__file__).resolve().parents[1]
p=W/'tools/linux-controlled-runner.py'
s=importlib.util.spec_from_file_location('controlled',p)
m=importlib.util.module_from_spec(s)
s.loader.exec_module(m)
rows=[]
def check(name,ok):
    rows.append(dict(id=name,passed=bool(ok)))
for text,want in [('0-31',32),('0-3,8-11',8),('0,2,4',3),('0-1,1-2',3)]:
    check('cpu-parse/'+text,m.online_cpus(text)==want)
for n in [1,2,4,16,32,64]:
    check('quota/'+str(n),m.cpu_max(n)==str(n*5000)+' 100000')
for bad in [0,-1,True,1.5]:
    try:
        m.cpu_max(bad);ok=False
    except ValueError:
        ok=True
    check('bad-count/'+repr(bad),ok)
for mode in m.CASES:
    limits=m.authorize(mode)
    check('approved-fixture/'+mode,limits['task_memory_bytes']==96*1024**2 and limits['wall_seconds']==30 and limits['swap_bytes']==0)
for mode in [*m.GATES,'sh','python -c arbitrary','../escape']:
    try:
        m.authorize(mode);ok=False
    except PermissionError:
        ok=True
    check('deny-unapproved/'+mode,ok)
original=m.read
for mutate in ['wrong-policy-hash','wrong-scope','qualification-not-approved']:
    def fake(path,kind=mutate):
        v=copy.deepcopy(original(path))
        if Path(path).name=='human-approval.json':
            if kind=='wrong-policy-hash':v['policy_sha256']='0'*64
            if kind=='wrong-scope':v['approved_scope']='gates'
        elif kind=='qualification-not-approved':v['scopes']['qualification']['Approved']=False
        return v
    m.read=fake
    try:
        m.authorize('normal');ok=False
    except PermissionError:
        ok=True
    check('approval-binding/'+mutate,ok)
    m.read=original
source=p.read_text()
for fragment in ['fcntl.LOCK_EX|fcntl.LOCK_NB','cgroup_inode','cgroup.kill','memory.peak','memory.events',
                 'memory.swap.max','pids.max','cpu.max','ControllerLost','UserCancelled',
                 'HostMemoryReserve','HostDiskReserve','DiskGrowth','Timeout','LimitAS=',
                 'RemainAfterExit=yes','KillMode=control-group','RuntimeMaxSec=',
                 "'set-property','--runtime',slice_name",'CGROUP_DELEGATION_NOT_QUALIFIED']:
    check('structure/'+fragment,fragment in source)
check('no-shell-true',not any(isinstance(node,ast.keyword) and node.arg=='shell' and isinstance(node.value,ast.Constant) and node.value.value is True for node in ast.walk(ast.parse(source))))
check('no-arbitrary-command-cli',"parser.add_argument('mode',choices=[*CASES,*GATES])" in source)
check('no-sudo','sudo' not in source)
check('exact-four-gates',len(m.GATES)==5 and m.GATES['test']==['cargo','test','--locked','--offline','--all','-j','1'])
print(json.dumps(dict(Required=len(rows),Executed=len(rows),Passed=sum(x['passed'] for x in rows),Failed=sum(not x['passed'] for x in rows),cases=rows,scope='Static structure/model checks only; no live enforcement credit'),indent=2))
assert all(x['passed'] for x in rows)
