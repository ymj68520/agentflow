"""Outcome-C diagnostic regression/freeze audit, not Raw Capture acceptance."""
import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PREFIX = 'phase-04c1b4c4r3c2-'
DOCS = ROOT / 'docs/implementation'


def main():
    freeze = json.loads((DOCS / (PREFIX + 'freeze.json')).read_bytes())
    frozen = freeze['frozen_raw_sha256']
    assert len(frozen) == freeze['raw_original_paths'] == 2047
    assert freeze['reviewed_existing_changes'] == ['crates/af-computation/Cargo.toml']
    for path, digest in frozen.items():
        assert hashlib.sha256((ROOT / path).read_bytes()).hexdigest() == digest, path
    actual_path = ROOT / 'target' / (PREFIX + 'document-actual.json')
    result = subprocess.run(
        [sys.executable, '-X', 'utf8', 'tools/oracle/raw_document_identity_c2.py'],
        cwd=ROOT, input=actual_path.read_bytes(), capture_output=True)
    assert result.returncode == 2, (result.returncode, result.stderr)
    report = json.loads(result.stdout)
    retained = json.loads((DOCS / (PREFIX + 'formal-binding-gap.json')).read_bytes())
    assert report == retained, 'Live diagnostic differs from retained minimal repro'
    assert report['finding_id'] == 'C2-GAP-01'
    assert report['full_raw_seed_conformance'] is False
    assert report['c1_gap_01'] == 'FORMAL BINDING REGISTERED (not reopened)'
    # No existing source/Expected/Registry edits can hide behind added files.
    changed = subprocess.check_output(
        ['git', 'diff', '--name-only', '--diff-filter=M', freeze['source_head']],
        cwd=ROOT, text=True).splitlines()
    assert changed == freeze['reviewed_existing_changes'], changed
    print(json.dumps({
        'diagnostic_regression': 'PASS — expected open binding gap remains observable',
        'finding_id': report['finding_id'], 'formal_gap_exit': result.returncode,
        'frozen_original_raw_paths': len(frozen),
        'only_reviewed_existing_change': changed,
        'new_diagnostic_case_ids': 1, 'new_typed_raw_capture_case_ids': 0,
        'new_raw_comparator_mutations': 0, 'required_field_coverage': 'NOT IMPLEMENTED',
        'outcome': 'C', 'positive_root_export': 'NotAuthorized'}, ensure_ascii=True))


if __name__ == '__main__':
    main()
