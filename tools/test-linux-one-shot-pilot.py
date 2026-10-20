"""Static one-shot admission tests; no cgroup/workload/fault injection."""
import ast
import copy
import importlib.util
import json
from pathlib import Path
W=Path(__file__).resolve().parents[1]
s=importlib.util.spec_from_file_location('pilot',W/'tools/linux-one-shot-pilot.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
a=m.read(m.E/'approval-matrix.json');p=m.read(m.POLICY);rows=[]
def check(n,v):rows.append(dict(id=n,passed=bool(v)))
check('three-independent-approvals',m.approvals(a,p))
for k in ['A','B','C']:
 for bad in [False,None,'Approved',1]:
  x=copy.deepcopy(a);x[k]['Approved']=bad;check('deny/'+k+'/'+repr(bad),not m.approvals(x,p))
for change in ['wrong-policy','wrong-disk-mode','retry','full-gates','expanded-crate']:
 x=copy.deepcopy(a);y=copy.deepcopy(p)
 if change=='wrong-policy':x['policy_sha256']='0'*64
 elif change=='wrong-disk-mode':x['B']['choice']='implicit'
 elif change=='retry':y['automatic_retries']=1
 elif change=='full-gates':y['full_gates_authorized']=True
 else:y['command']=['cargo','check','--workspace']
 check('deny/'+change,not m.approvals(x,y))
for key in m.LIMITS:
 y=copy.deepcopy(p);y['limits'][key]+=1;check('exact-budget/'+key,not m.approvals(a,y))
for bad in [True,1.0,'1']:
 y=copy.deepcopy(p);y['limits']['cargo_jobs']=bad;check('deny-budget-type/'+repr(bad),not m.approvals(a,y))
y=copy.deepcopy(p);y['automatic_retries']=0.0;check('deny-retry-float',not m.approvals(a,y))
source=Path(m.__file__).read_text()
audit_source=(W/'tools/audit-linux-one-shot-pilot.py').read_text()
for item in ['pilot-launch-consumed.json','pilot-launch-reserved.json','ONE_SHOT_ALREADY_RESERVED_OR_CONSUMED','release.json','guardian-ready.json','service-ready.json','KERNEL_RLIMIT_AS_READBACK','PRODUCTION_LIMIT_READBACK','MANDATORY_RESOURCE_INTERFACE_MISSING','PRE_CARGO_LIMIT_DRIFT','FINAL_HOST_ADMISSION','immutable_files','SOURCE_NOT_FROZEN_CLEAN','cpu.max','memory.max','memory.swap.max','pids.max','NoAvailableInodes','scan_seconds','detection_to_cleanup_seconds','process_tree_events','GUARDIAN_FAILED_CONTROLLER_TAKEOVER','pidfd_send_signal','CARGO_TARGET_DIR' ]:
 check('structure/'+item,item in source or item in audit_source)
for name in ['linux-one-shot-pilot.py','audit-linux-one-shot-pilot.py']:
 tree=ast.parse((W/'tools'/name).read_text());check('no-shell/'+name,not any(isinstance(n,ast.keyword) and n.arg=='shell' and isinstance(n.value,ast.Constant) and n.value.value is True for n in ast.walk(tree)))
check('qualified-core-unchanged',m.sha(m.BASE)==m.BASE_SHA)
check('no-global-toolchain-proxy',m.read(m.E/'toolchain-identity.json')['binaries']['cargo']['path'].find('/.rustup/toolchains/')>0)
check('no-cargo-was-launched',not (m.E/'pilot-launch-consumed.json').exists())
print(json.dumps(dict(Required=len(rows),Executed=len(rows),Passed=sum(x['passed'] for x in rows),Failed=sum(not x['passed'] for x in rows),cases=rows,scope='Static only; no claim of production effective limits'),indent=2))
assert all(x['passed'] for x in rows)
