"""Read-only identities and project-local one-shot preparation; never launches Cargo."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import resource
import subprocess
import uuid
W=Path(__file__).resolve().parents[1];R=W.parent;E=R/'evidence/phase-linux-full-engineering-gates'
START='4b08031f4c8d54074b3c2f90e6361e5ccf9b376a'
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def read(p):return json.loads(p.read_bytes())
def save(p,v):
    with p.open('x') as f:json.dump(v,f,indent=2)
def git(args,cwd=W):return subprocess.check_output(['git','-c','core.hooksPath=/dev/null','-c','core.fsmonitor=false','-c','core.packedGitWindowSize=8m','-c','core.packedGitLimit=32m',*args],cwd=cwd,timeout=60).decode().strip()
def integrity(name):
    old=R/'evidence/phase-linux-one-shot-rust-pilot';ledger=read(old/'phase-linux-one-shot-rust-pilot-final-runtime-results.json')
    assert all(sha(old/p)==h for p,h in ledger['evidence_files'].items())
    assert not git(['diff','--name-only'])
    rows=[]
    for f in sorted((R/'frozen').iterdir()):
        if not f.is_dir():continue
        key='current' if f.name=='legacy-current-dc1d-l2pivot01' else f.name
        expected=read(R/'evidence'/('source-map-'+key+'.json'))
        assert all((f/p).is_file() and not (f/p).is_symlink() and sha(f/p)==v['raw_sha256'] for p,v in expected.items())
        assert not git(['status','--porcelain'],f)
        rows.append(dict(snapshot=f.name,head=git(['rev-parse','HEAD'],f),paths=len(expected)))
    tracked=git(['ls-tree','-r','--name-only',START]).splitlines()
    protected={p:sha(W/p) for p in tracked if p.startswith(('spec/','crates/')) or 'expected' in p.lower() or 'oracle' in p.lower()}
    cache=R/'cache/cargo-linux-primary-v1';archives={p.name:sha(p) for p in (cache/'archives').glob('*.crate')};vendor={}
    for folder in (cache/'vendor').iterdir():
        if folder.is_dir():
            data=read(folder/'.cargo-checksum.json');assert all(sha(folder/p)==h for p,h in data['files'].items())
            vendor[folder.name]=sha(folder/'.cargo-checksum.json')
    before=read(E/'source-integrity-before.json') if name!='before' else None
    if before:
        assert protected==before['protected_files'] and archives==before['cargo_archives'] and vendor==before['cargo_vendor_checksums']
    result=dict(status='Passed',source_head=git(['rev-parse','HEAD']),frozen_snapshots=rows,frozen_paths=sum(x['paths'] for x in rows),protected_files=protected,
                cargo_lock_sha256=sha(W/'Cargo.lock'),cargo_archives=archives,cargo_vendor_checksums=vendor,previous_phase_receipts_verified=len(ledger['evidence_files']),old_receipts_unchanged=True)
    save(E/('source-integrity-'+name+'.json'),result)
    print(json.dumps(dict(mode=name,status='Passed',frozen_paths=result['frozen_paths'],protected_files=len(protected),cargo_packages=len(archives))))

def freeze():
    assert not git(['status','--porcelain']) and not (E/'shared-freeze.json').exists()
    import importlib.util,fcntl
    spec=importlib.util.spec_from_file_location('quality',W/'tools/linux-quality-gates.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
    bins=read(E/'toolchain-identity.json')['binaries'];head=git(['rev-parse','HEAD']);tree=git(['rev-parse','HEAD^{tree}'])
    units=m.core.ctl(['list-units','--all','--no-legend','--plain','--no-pager','afl2p02*','afl2p03*','afl2p04*','afl2p05*']);assert not units['stdout'].strip()
    with (R/'evidence/phase-linux-resource-qualification/top-level.lock').open('r+') as lock:fcntl.flock(lock.fileno(),fcntl.LOCK_EX|fcntl.LOCK_NB)
    active=[]
    for p in Path('/proc').iterdir():
        if not p.name.isdecimal() or int(p.name)==os.getpid():continue
        try:
            if p.stat().st_uid!=os.getuid():continue
            cwd=(p/'cwd').resolve();cmd=(p/'cmdline').read_bytes().replace(bytes([0]),b' ').decode('utf8','replace')
            if cwd.is_relative_to(R) or str(R) in cmd:active.append(dict(pid=int(p.name),comm=(p/'comm').read_text().strip()))
        except (FileNotFoundError,PermissionError,ProcessLookupError):pass
    assert not active,active
    gates={}
    for gate in ['fmt','clippy']:
        m.configure(gate);m.E.mkdir(exist_ok=True)
        assert m.approvals(read(m.E/'approval-matrix.json'),read(m.POLICY))
        runid=str(uuid.uuid4());tmp=R/'tmp/production-protector'/('quality-'+gate+'-'+runid)
        target=R/'build/linux-primary-v1';gitdir=Path(git(['rev-parse','--absolute-git-dir']))
        for root in [tmp,target,gitdir]:
            assert root.resolve().is_relative_to(R.resolve())
            for ancestor in [root,*root.parents]:assert not ancestor.is_symlink()
        assert not tmp.exists();tmp.mkdir(parents=True)
        for n in ['tmp','home','cache','config','data']:(tmp/n).mkdir()
        target.mkdir(parents=True,exist_ok=True)
        roots=[W,gitdir,R/'cache/cargo-linux-primary-v1',target,tmp,E]
        audits=[]
        for path in roots:
            assert path.stat().st_dev==W.stat().st_dev
            fs=os.statvfs(path);audits.append(dict(path=str(path),resolved=str(path.resolve()),device=path.stat().st_dev,allocated_regular_bytes=m.core.measured_roots([path]),free_bytes=fs.f_bavail*fs.f_frsize,free_inodes=fs.f_favail,filesystem=subprocess.check_output(['findmnt','-J','-T',str(path),'-o','SOURCE,FSTYPE,TARGET,OPTIONS']).decode()))
        forbidden=[k for k in ['RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER'] if os.environ.get(k)]+[k for k in os.environ if k.startswith('CARGO_PROFILE_')]
        assert not forbidden,forbidden
        env=dict(HOME=str(tmp/'home'),TMPDIR=str(tmp/'tmp'),TMP=str(tmp/'tmp'),TEMP=str(tmp/'tmp'),XDG_CACHE_HOME=str(tmp/'cache'),XDG_CONFIG_HOME=str(tmp/'config'),XDG_DATA_HOME=str(tmp/'data'),
                 PATH=str(Path(bins['cargo']['path']).parent)+':'+str(R/'venvs/linux-primary-v1/bin')+':/usr/bin:/bin',LANG='C.UTF-8',LC_ALL='C.UTF-8',CARGO_HOME=str(R/'cache/cargo-linux-primary-v1'),CARGO_TARGET_DIR=str(target),CARGO_BUILD_JOBS='1',RUST_TEST_THREADS='1',CARGO_NET_OFFLINE='true',CARGO_TERM_COLOR='never',RUSTC=bins['rustc']['path'],RUSTDOC=bins['rustdoc']['path'],RUSTFMT=bins['rustfmt']['path'],RUSTUP_AUTO_INSTALL='0',RUSTUP_NO_UPDATE_CHECK='1')
        files=[W/'tools/linux-quality-gates.py',m.POLICY,W/'tools/linux-production-protector.py',W/'Cargo.lock',W/'Cargo.toml',R/'cache/cargo-linux-primary-v1/config.toml',E/'toolchain-identity.json',m.E/'approval-matrix.json',R/'venvs/linux-primary-v1/bin/python']
        files+=[Path(x['path']) for x in bins.values()]+list((W/'crates').glob('*/Cargo.toml'))+list((R/'cache/cargo-linux-primary-v1/vendor').glob('*/.cargo-checksum.json'))
        files+=[p for p in [W/'rustfmt.toml',W/'.rustfmt.toml',W/'.cargo/config.toml',W/'.cargo/config'] if p.is_file()]
        ancestor=m.core.delegated_root();anc=[]
        for path in [ancestor,*ancestor.parents]:
            if not path.is_relative_to(Path('/sys/fs/cgroup')):break
            anc.append(dict(path=str(path),limits={n:(path/n).read_text().strip() for n in ['cpu.max','memory.max','memory.swap.max','pids.max'] if (path/n).exists()}))
        fs=os.statvfs(R);assert m.core.host_available()>=11811160064 and fs.f_bavail*fs.f_frsize>=34359738368 and fs.f_favail>0
        value=dict(gate=gate,run_id=runid,source_head=head,git_tree=tree,executor_sha256=sha(W/'tools/linux-quality-gates.py'),policy_sha256=sha(m.POLICY),cargo_lock_sha256=sha(W/'Cargo.lock'),toolchain=bins,immutable_files={str(path):sha(path) for path in files},effective_env=env,write_roots=[str(path) for path in roots],job_tmp_root=str(tmp),write_root_audits=audits,ancestor_limits=anc,profile_overrides=forbidden,host_preflight=dict(MemAvailable=m.core.host_available(),free_bytes=fs.f_bavail*fs.f_frsize,free_inodes=fs.f_favail,active_processes=active))
        save(m.E/'execution-freeze.json',value);gates[gate]=dict(run_id=runid,freeze_sha256=sha(m.E/'execution-freeze.json'))
    save(E/'shared-freeze.json',dict(source_head=head,git_tree=tree,gates=gates,round2_frozen=False))
    print(json.dumps(dict(source_head=head,git_tree=tree,gates=gates,round2_approved=False)))

def main():
    os.nice(19);resource.setrlimit(resource.RLIMIT_AS,(512*1024**2,)*2);resource.setrlimit(resource.RLIMIT_CPU,(60,)*2)
    p=argparse.ArgumentParser();p.add_argument('mode',choices=['before','after','freeze']);a=p.parse_args()
    return freeze() if a.mode=='freeze' else integrity(a.mode)
if __name__=='__main__':main()
