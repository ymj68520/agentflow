"""Read-only F1 frozen inputs and narrow production exception audit.

The old R3 byte-lock on af-effect/canonical.rs describes pre-repair evidence,
not F1's expressly authorized exception. No historical manifest is rewritten.
"""
import hashlib,json,re,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/'tools'))
from native_guard_source_binding import check
BASE='be04c05e07d3a72a7c795d289e689281bceaa65f'
def sha(b):return hashlib.sha256(b).hexdigest()
def read(p):return json.loads((ROOT/p).read_bytes())
def mask_functions(text):
    names=['to_value','row_value','entry_value','guard_value','constant_value']
    for name in names:
        match=re.search(r'(?m)^\s*(?:pub )?fn '+name+r'\(',text);assert match,name
        start=match.start();brace=text.index('{',match.end());depth=1;end=brace+1
        # These reviewed bodies have balanced format placeholders; no macro
        # or raw-literal grammar is used as a semantic identity parser here.
        while depth:
            depth+=(text[end]=='{')-(text[end]=='}');end+=1
        text=text[:start]+'\n<reviewed F1 function '+name+'>\n'+text[end:]
    return text
def main():
    manifest=read('docs/implementation/phase-04c1b4c4r3f1-frozen-inputs.json')
    assert manifest['baseline_head']==BASE and manifest['initial_nonignored_paths']==1987
    assert len(manifest['frozen_paths'])==1985
    for p,h in manifest['frozen_paths'].items():assert sha((ROOT/p).read_bytes())==h,('frozen raw bytes',p)
    original=set(subprocess.check_output(['git','ls-tree','-r','--name-only',BASE],cwd=ROOT,text=True).splitlines())
    assert original-set(manifest['authorized_existing_changes'])==set(manifest['frozen_paths'])
    changed=set(subprocess.check_output(['git','-c','core.safecrlf=false','diff',BASE,'--name-only'],cwd=ROOT,text=True).splitlines())&original
    assert changed<=set(manifest['authorized_existing_changes']),changed
    p='crates/af-effect/src/canonical.rs';old=subprocess.check_output(['git','show',BASE+':'+p],cwd=ROOT).decode()
    live=(ROOT/p).read_text(encoding='utf-8');assert mask_functions(old)==mask_functions(live)
    assert 'Value::Bytes(af_core::base64url::decode(value).map_err' in ''.join(live.splitlines())
    specs=read('spec/registry/specifications.json')['specifications'];rules=read('spec/registry/rules.json')['rules']
    assert (len(specs),len(rules))==(54,468)
    for s in specs:assert sha((ROOT/s['normative_source']['logical_path']).read_bytes())==s['normative_source']['content_digest']['hex']
    model=json.loads(re.search(r'```json\s*(.*?)\s*```',(ROOT/'spec/tables/proof-report-model-v3.md').read_text(encoding='utf-8'),re.S)[1])
    binding=check(model,read('spec/conformance/g01-native-guard-staging/historical-imports.json'))
    assert len(model['types'])==652 and len(model['historical_selected_imports'])==97
    def refs(x):
        if isinstance(x,dict):return int('ref' in x)+sum(refs(v) for v in x.values())
        if isinstance(x,list):return sum(refs(v) for v in x)
        return 0
    assert refs(model['types'])==1941
    assert len(list((ROOT/'spec/idl').rglob('*.afidl')))==17
    print(json.dumps({'frozen_raw_paths':1985,'authorized_existing_changes':sorted(changed),'production_files_changed':1,
        'reviewed_private_functions':['EffectSignature::to_value','row_value','entry_value','guard_value','constant_value'],
        'other_production_bytes_equal':True,'native_declarations_compared':binding['live_declarations_compared'],
        'selected_imports':97,'historical_source_bundles':19,'specifications':54,'rules':468,'types':652,'refs':1941,
        'old_expected_and_original_oracle_unchanged':True,'positive_capability':'NotAuthorized','full_t3_seed_conformance':False},indent=2))
if __name__=='__main__':main()
