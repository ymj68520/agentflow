"""Static per-gate authorization tests; no Cargo/systemd launch."""
import ast
import copy
import importlib.util
import io
import json
from pathlib import Path
W=Path(__file__).resolve().parents[1]
s=importlib.util.spec_from_file_location('quality',W/'tools/linux-quality-gates.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
policy=m.read(m.POLICY);rows=[]
def check(n,v):rows.append(dict(id=n,passed=bool(v)))
for gate in ['fmt','clippy']:
 m.configure(gate);a=m.read(m.E/'approval-matrix.json')
 check(gate+'/approved',m.approvals(a,policy))
 for decision in ['A','B','C']:
  for bad in [False,None,'Approved',1]:
   b=copy.deepcopy(a);b[decision]['Approved']=bad;check(gate+'/deny-'+decision+'/'+repr(bad),not m.approvals(b,policy))
 for k in m.LIMITS:
  p=copy.deepcopy(policy);p['per_gate_limits'][gate][k]+=1;check(gate+'/budget-'+k,not m.approvals(a,p))
 for change in ['other-gate','missing-B2','expanded-command','retry','float-claim']:
  b=copy.deepcopy(a)
  if change=='other-gate':b['gate']='test'
  elif change=='missing-B2':b['B']['choice']=None
  elif change=='expanded-command':b['C']['argv']=['cargo','check','--workspace']
  elif change=='retry':b['C']['retries']=1
  else:b['C']['maximum_launches']=1.0
  check(gate+'/deny-'+change,not m.approvals(b,policy))
for gate in ['test','spec-check']:
 m.configure(gate);check(gate+'/not-approved',not m.approvals(m.read(m.E/'approval-matrix.json'),policy))
source=(W/'tools/linux-quality-gates.py').read_text();tree=ast.parse(source)
for text in ['gate-launch-consumed.json','gate-launch-reserved.json','ROUND1_SHARED_SOURCE_BINDING','ROUND1_HEAD_MISMATCH','FMT_MUST_COMPLETE_FIRST','ROUND2_NOT_ADMITTED','uniform_build_runtime_approved','coverage_contract_approved','resource-samples.jsonl','process-trees.jsonl','samples_sha256','kernel_service_limits','MANDATORY_RESOURCE_INTERFACE_MISSING','Guardian','GUARDIAN_FAILED_CONTROLLER_TAKEOVER','pidfd_send_signal','RuntimeMaxSec=5400s']:
 check('structure/'+text,text in source)
check('long-gates-no-10000-sample-abort','SAMPLE_BOUND' not in source)
check('only-one-cargo-spawn-site',sum(isinstance(n,ast.Call) and isinstance(n.func,ast.Attribute) and isinstance(n.func.value,ast.Name) and n.func.value.id=='subprocess' and n.func.attr=='Popen' for n in ast.walk(tree))==1)
check('no-shell',not any(isinstance(n,ast.keyword) and n.arg=='shell' and isinstance(n.value,ast.Constant) and n.value.value is True for n in ast.walk(tree)))
node=next(n for n in ast.walk(tree) if isinstance(n,ast.FunctionDef) and n.name=='write_line')
namespace={'json':json};exec(compile(ast.fix_missing_locations(ast.Module(body=[node],type_ignores=[])),'static-stream-test','exec'),namespace)
stream=io.StringIO();namespace['write_line'](stream,{'a':1});namespace['write_line'](stream,{'b':2})
check('stream-is-real-jsonl',[json.loads(x) for x in stream.getvalue().splitlines()]==[{'a':1},{'b':2}])
check('5400-seconds',m.LIMITS['wall_seconds']==5400)
check('no-lib-or-filters','--lib' not in m.COMMANDS['clippy'] and '--all-targets' in m.COMMANDS['clippy'] and '--all-features' in m.COMMANDS['clippy'] and m.COMMANDS['clippy'][-2:]==['-D','warnings'])
check('qualified-core-exact',m.sha(m.BASE)==m.BASE_SHA)
check('no-round1-consumed',all(not (m.PHASE/k/'gate-launch-consumed.json').exists() for k in ['fmt','clippy']))
print(json.dumps(dict(Required=len(rows),Executed=len(rows),Passed=sum(x['passed'] for x in rows),Failed=sum(not x['passed'] for x in rows),cases=rows,scope='Static admission/streaming only'),indent=2))
assert all(x['passed'] for x in rows)
