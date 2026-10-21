"""Initialize this human-authorized demo task once, preserving old receipts."""
import importlib.util
import json
from pathlib import Path
import resource

W = Path(__file__).resolve().parents[1]
resource.setrlimit(resource.RLIMIT_AS, (512 * 1024**2,) * 2)
spec = importlib.util.spec_from_file_location('demo', W / 'tools/linux-demo-dev.py')
demo = importlib.util.module_from_spec(spec)
spec.loader.exec_module(demo)
phase = demo.PHASE
phase.mkdir(exist_ok=False)
session = demo.read(W / 'target/thesis-fast-dev/session.json')
for item in session['toolchain'].values():
    if demo.sha(item['path']) != item['sha256']:
        raise RuntimeError('TOOLCHAIN_DRIFT')
session.update(start_head=demo.git(['rev-parse', 'HEAD']),
               disk_baseline_bytes=demo.core.measured_roots([Path(p) for p in session['write_roots']]),
               baseline_scope='Entire visual demo development task, including every retry',
               selected_limits=demo.LIMITS)
policy = demo.read(W / 'engineering/policies/thesis-fast-dev-v1.json')
policy.update(scope='Graduation project Stage 2 visual workflow demo development', commands=demo.COMMANDS)
demo.save(demo.POLICY, policy)
demo.save(phase / 'session.json', session)
demo.save(phase / 'approval.json', dict(
    authority='Explicit human directive: 毕业设计阶段 2：可视化 Agent 工作流演示; continuous development and carried numeric resource/B2 authorization',
    repeat_commands_approved=True, B2_approved=True, policy_sha256=demo.sha(demo.POLICY),
    scope='Trusted project demo, formatting, lock generation, local/workspace tests, Clippy, spec-check and loopback demo service only; no push or formal asset changes'))
print(json.dumps(dict(session=str(phase), start_head=session['start_head'], disk_baseline=session['disk_baseline_bytes'])))
