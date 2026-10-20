"""Nonprivileged sampled proc observer; thread stays inside the approved job cgroup.

VmPeak is the kernel high-water value at the instant read, not proof that the
process's final lifetime maximum was captured. No ptrace, wrappers or installs.
"""
import hashlib
import json
import os
from pathlib import Path
import threading
import time

NAMES = {'cc', 'gcc', 'collect2', 'ld', 'ld.lld', 'rust-lld', 'lld', 'cc1'}


def parse_status(text):
    return {k: v.strip() for k, v in (line.split(':', 1) for line in text.splitlines() if ':' in line)
            if k in {'Name', 'Pid', 'PPid', 'VmSize', 'VmPeak', 'VmRSS', 'Threads'}}


def host_memory():
    return {k: v.strip() for k, v in (line.split(':', 1) for line in Path('/proc/meminfo').read_text().splitlines())
            if k in {'MemAvailable', 'CommitLimit', 'Committed_AS'}}


def capture(pid):
    proc = Path('/proc') / str(pid)
    comm = (proc / 'comm').read_text().strip()
    if comm not in NAMES and not comm.endswith(('-gcc', '-cc', '-ld')):
        return None
    identity_before = (proc / 'stat').read_text().rsplit(')', 1)[1].split()[19]
    value = dict(pid=pid, start_ticks=identity_before, comm=comm,
                 executable=os.readlink(proc / 'exe'),
                 argv=(proc / 'cmdline').read_bytes().split(b'\0')[:-1],
                 status=parse_status((proc / 'status').read_text()),
                 effective_limits=(proc / 'limits').read_text(),
                 cgroup=(proc / 'cgroup').read_text().strip())
    if (proc / 'stat').read_text().rsplit(')', 1)[1].split()[19] != identity_before:
        return None
    value['argv'] = [x.decode('utf8', 'replace') for x in value['argv']]
    return value


class Observer:
    def __init__(self, job, leaf, group, period=0.02):
        self.job, self.leaf, self.group, self.period = job, leaf, group, period
        self.stop = threading.Event()
        self.thread = threading.Thread(target=self.run, name='linker-proc-observer', daemon=True)

    def start(self):
        self.thread.start()

    def finish(self):
        self.stop.set()
        self.thread.join(timeout=2)
        if self.thread.is_alive():
            raise RuntimeError('OBSERVER_JOIN_FAILED')

    def run(self):
        count, missed, scans, last, max_gap, error = 0, 0, 0, None, 0, None
        path = self.job / 'linker-memory-samples.jsonl'
        try:
            with path.open('x') as out:
                while not self.stop.is_set():
                    current = time.monotonic()
                    if last is not None:
                        max_gap = max(max_gap, current - last)
                    last = current
                    scans += 1
                    processes = []
                    for raw_pid in (self.leaf / 'cgroup.procs').read_text().split():
                        try:
                            value = capture(int(raw_pid))
                            if value is not None:
                                processes.append(value)
                        except (FileNotFoundError, ProcessLookupError, PermissionError):
                            missed += 1
                    if processes or scans == 1 or scans % 50 == 0:
                        row = dict(monotonic=current, linker_processes=processes, host=host_memory(),
                                   cgroup={name: (self.group / name).read_text().strip()
                                           for name in ['memory.current', 'memory.peak', 'memory.events']})
                        out.write(json.dumps(row, separators=(',', ':')) + '\n')
                        out.flush()
                        count += 1
                    self.stop.wait(self.period)
        except BaseException as exc:
            error = repr(exc)
        finally:
            result = dict(status='Completed' if error is None else 'Observation Error', error=error,
                          scans=scans, rows=count, racing_process_reads=missed,
                          requested_period_seconds=self.period, max_scan_gap_seconds=max_gap,
                          true_final_linker_lifetime_peak='Not Observed',
                          limitation='Sampled /proc observations; may miss short processes or their final maxima. No cgroup-to-VmPeak inference.')
            if path.exists():
                with path.open('rb') as stream:
                    result['stream_sha256'] = hashlib.file_digest(stream, 'sha256').hexdigest()
            with (self.job / 'linker-observer-receipt.json').open('x') as out:
                json.dump(result, out, indent=2)
