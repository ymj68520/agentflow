"""Read-only C1 freeze/source census. Does not certify T3 conformance."""
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / 'tools'))
from native_guard_source_binding import check


def audit():
    manifest = json.loads((ROOT / 'docs/implementation/phase-04c1b4c4r3c1-frozen-inputs.json').read_bytes())
    frozen = manifest['frozen_paths']
    assert len(frozen) == 1997
    for p, h in frozen.items():
        assert hashlib.sha256((ROOT / p).read_bytes()).hexdigest() == h, p
    original = set(subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', manifest['baseline_head']], cwd=ROOT, text=True).splitlines())
    assert original == set(frozen) | set(manifest['authorized_existing_changes'])
    changed = set(subprocess.check_output(['git', 'diff', manifest['baseline_head'], '--name-only'], cwd=ROOT, text=True).splitlines()) & original
    assert changed <= set(manifest['authorized_existing_changes'])
    model = json.loads(re.search(r'```json\s*(.*?)\s*```', (ROOT / 'spec/tables/proof-report-model-v3.md').read_text(encoding='utf-8'), re.S)[1])
    binding = check(model, json.loads((ROOT / 'spec/conformance/g01-native-guard-staging/historical-imports.json').read_bytes()))
    census = json.loads(subprocess.check_output([sys.executable, 'tools/audit-g01-t3-resumption.py'], cwd=ROOT))
    census['baseline'] = manifest['baseline_head']
    census['kind'] = 'C1ReadOnlyLexicalInventory;NOT semantic producer closure'
    census['outcome'] = 'C — FORMAL BINDING GAP'
    census['native_collector'] = 'PARTIAL; no complete independently compared seeds'
    census['blocking_finding'] = 'C1-GAP-01'
    for site in census['sites']:
        site['actual_capture_status'] = 'UNADAPTED; stopped before C1-D/E'
        if site['applicable_source_category'] == 'native_guard_record':
            site['actual_capture_status'] = 'PARTIAL_NATIVE_TRACE; full seed conformance PENDING'
        if site['function'] == 'invalid_contract_report':
            site['actual_capture_status'] = 'BLOCKED_C1-GAP-01; direct raw/context binding'
    assert len(list((ROOT / 'spec/idl').rglob('*.afidl'))) == 17
    return {'frozen_raw_paths': len(frozen), 'changed_original_paths': sorted(changed),
            'native_declarations_compared': binding['live_declarations_compared'],
            'registered_counts': census['registered_counts'], 'abi': 775, 'afidl': 17,
            'hash_domains': 14, 'abi_hash_counts_basis': 'unchanged frozen sources plus current Cargo/spec gates',
            'historical_source_bundles': 19, 'old_expected_and_f1_sources_unchanged': True,
            'census': census, 'certified_complete_t3_case_ids': 0,
            'required_field_gaps': 'NOT CLOSED; not counted as zero',
            'positive_root_export': 'NotAuthorized'}


if __name__ == '__main__':
    print(json.dumps(audit(), ensure_ascii=True, indent=2))
