"""Small synthetic observer tests; never starts Cargo or injects faults."""
import json
from pathlib import Path
import sys
import tempfile
import time
import linker_memory_observer as m

rows = []
def check(name, value):
    rows.append(dict(id=name, passed=bool(value)))

check('parse-fields', m.parse_status('Name:\tld\nVmPeak:\t123 kB\nUnrelated:\tx\n') == {'Name': 'ld', 'VmPeak': '123 kB'})
check('host-fields', set(m.host_memory()) == {'MemAvailable', 'CommitLimit', 'Committed_AS'})
check('unrelated-process-filter', m.capture(__import__('os').getpid()) is None)
base = Path(__file__).resolve().parents[1].parent / 'evidence/phase-linux-clippy-recovery-8g'
with tempfile.TemporaryDirectory(prefix='observer-static-', dir=base) as tmp:
    root = Path(tmp)
    leaf, group, job = root/'leaf', root/'group', root/'job'
    for path in [leaf, group, job]:
        path.mkdir()
    (leaf/'cgroup.procs').write_text('12345\n')
    for name in ['memory.current', 'memory.peak', 'memory.events']:
        (group/name).write_text('0')
    original = m.capture
    m.capture = lambda pid: dict(pid=pid, status={'VmPeak':'123 kB'})
    observer = m.Observer(job, leaf, group, 0.01)
    observer.start()
    time.sleep(0.04)
    observer.finish()
    m.capture = original
    receipt = json.loads((job/'linker-observer-receipt.json').read_text())
    stream = [json.loads(x) for x in (job/'linker-memory-samples.jsonl').read_text().splitlines()]
    check('thread-finished', not observer.thread.is_alive())
    check('recorded-linker', any(x['linker_processes'][0]['pid']==12345 for x in stream))
    check('rows-match', len(stream)==receipt['rows'] and receipt['scans']>=1)
    check('peak-honesty', receipt['true_final_linker_lifetime_peak']=='Not Observed')
    check('no-observation-error', receipt['error'] is None)
print(json.dumps(dict(Required=len(rows), Passed=sum(x['passed'] for x in rows), cases=rows), indent=2))
assert all(x['passed'] for x in rows)
