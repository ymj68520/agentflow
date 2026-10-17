"""C2R3 baseline and full-root gate. Never a permissive subset gate."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

R = Path(__file__).resolve().parents[1]
P = 'phase-04c1b4c4r3c2r3-'
BASE = '0d471ed7840d39c24e95e8e6598ed843940ce0bf'


def model():
    return json.loads(re.search(r'```json\s*(.*?)\s*```',
        (R/'spec/tables/proof-report-model-v5.md').read_text(encoding='utf8'), re.S)[1])


def references(value):
    if isinstance(value, dict):
        if 'ref' in value:
            yield value['ref']
        for child in value.values():
            yield from references(child)
    elif isinstance(value, list):
        for child in value:
            yield from references(child)


def closure(types, descriptor):
    pending = list(references(descriptor))
    seen = set()
    while pending:
        name = pending.pop()
        if name in seen:
            continue
        seen.add(name)
        pending.extend(references(types[name]))
    return sorted(seen)


def baseline():
    source = R/'docs/implementation/phase-04c1b4c4r3c2r2-field-and-stage-audit.json'
    previous = json.loads(source.read_bytes())
    m = model()
    ledger = previous['RootFieldClosureLedger']
    assert ledger['required_instances'] == 1750
    assert ledger['compared_registered_subset_instances'] == 1000
    assert ledger['remaining_unmaterialized_instances'] == 625
    assert ledger['remaining_partial_basis_instances'] == 125
    assert ledger['remaining_incomplete_instances'] == 750
    prior = {row['exact_field_name']: row for row in previous['RawFactRootFieldManifest']}
    plan = []
    for name, descriptor in m['types']['Fact']['fields']:
        row = prior[name]
        assert descriptor == row['exact_registered_type']
        compared = row['final_mapping'] == 'REGISTERED_SUBSET_COMPARED'
        partial = not compared and row['current_local_mapping'] == 'PARTIAL_LOCAL_MAPPING'
        plan.append({
            'exact_registered_field': name, 'descriptor_owner': m['model_id'],
            'descriptor_type': descriptor,
            'applicability': row['required_optional_applicability'],
            'producer_source': ('RawLocalCapture::registered_root_subset (eight-field metadata subset only)'
                                if compared else row['actual_rust_source']),
            'stage_owner': row['required_producer_stage'],
            'current_status': 'PREVIOUS_SUBSET_COMPARED' if compared else 'PARTIAL' if partial else 'MISSING',
            'direct_nested_dependencies': sorted(set(references(descriptor))),
            'transitive_nested_dependencies': closure(m['types'], descriptor),
            'expected_projection': 'FULL_FACT_EXPECTED_NOT_IMPLEMENTED',
            'actual_projection': 'FULL_FACT_PROJECTION_NOT_IMPLEMENTED',
            'comparison_status': 'NOT_REACCEPTED_ON_COMPLETE_FACT',
        })
    assert len(plan) == 14
    assert sum(p['current_status'] == 'PREVIOUS_SUBSET_COMPARED' for p in plan) == 8
    assert sum(p['current_status'] == 'MISSING' for p in plan) == 5
    assert sum(p['current_status'] == 'PARTIAL' for p in plan) == 1
    instances = []
    state = {r['exact_registered_field']: r['current_status'] for r in plan}
    for old in previous['RootFieldGapBaseline']['instances']:
        status = state[old['root_field']]
        instances.append({'event_id': old['event_id'], 'field': old['root_field'],
                          'start_status': status, 'final_status': status,
                          'compared_on_complete_fact': False})
    assert len(instances) == 1750
    return {
        'start_head': BASE,
        'baseline_source': str(source),
        'baseline_raw_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
        'RootFieldClosurePlan': plan,
        'RootFieldInstanceLedger': {'required':1750,'previous_subset_compared':1000,
            'missing':625,'partial':125,'otherwise_unresolved':0,
            'full_fact_reaccepted':0,'instances':instances},
        'initial_direct_nested_obligations':7125,
        'full_required_nested_instances':None,
        'full_required_relation_instances':None,
        'main_cases':59,'main_events':125,
        'inherited_development_cases':16,'inherited_development_events':48,
        'new_complete_capture_case_ids':0,
        'strict_event_gate':'SEE_FRESH_LOCAL_RECEIPT',
        'strict_root_gate':2,'strict_nested_gate':2,
        'strict_relation_gate':2,'strict_total_gate':2,
        'positive_root_export':'NotAuthorized',
        'outcome':'D_PENDING_BINDING_INVESTIGATION',
    }


def binding_gap(report):
    actual_path = R/'target'/(P+'binding-actual.json')
    if not actual_path.exists():
        return
    import importlib.util
    spec = importlib.util.spec_from_file_location('c2r3_independent_binding', R/'tools/oracle/raw_relation_schema_binding.py')
    oracle = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(oracle)
    actual = json.loads(actual_path.read_bytes())
    diagnostic = oracle.report(actual)
    expected = oracle.expected()
    m = model()
    field = dict(dict(m['types']['RelationSnapshotContent']['fields'])['pin']['fields'])['schema']
    assert field == {'ref':'af-semantic::TypeSchemaRef'}
    report['FormalBindingGap'] = {
        'finding_id':'R3-CTX-REL-01','severity':'P1','blocking':True,
        'classification':'FIELD_SOURCE_BINDING_UNDERSPECIFIED_NOT_FALSE_PROVEN',
        'descriptor_owner':m['model_id'],'descriptor_name':'RelationSnapshotContent',
        'first_unbound_field':diagnostic['first_unbound_field'],
        'registered_schema_descriptor':field,
        'registered_type_schema_ref':m['types']['af-semantic::TypeSchemaRef'],
        'native_source':{'path':'crates/af-contract/src/proof.rs','type':'RelationCatalogPin','field':'schema','field_type':'String'},
        'snapshot_source':{'path':'crates/af-contract/src/universe.rs','type':'RelationCatalogSnapshot<T>',
            'stored_fields':['schema','revision','canonical_bytes','catalog','pin'],
            'construction_schema_definition_retained':False},
        'affected_parent_type':'ContractProofContextContents.relation_snapshots',
        'relevant_rules':['AF-SEMANTIC-0198','AF-SEMANTIC-0261','AF-SEMANTIC-0262','AF-SEMANTIC-0265','AF-SEMANTIC-0266'],
        'not_claimed':['construction history is already normative identity',
            'native snapshot values ought to differ under current semantics',
            'the port Checker gave a wrong verdict',
            'every None relation snapshot has this problem',
            'a new edition or a particular amendment is already authorized'],
        'unregistered_choices_not_taken':['bind to construction schema pin not retained by the actual snapshot',
            'substitute current-context or globally registered schema as if it were the native stored pin',
            'change the registered field to schema-family text',
            'drop the Some snapshot or invent a placeholder TypeSchemaRef'],
        'required_adjudication':'Define the precise source and meaning of RelationSnapshotContent.pin.schema for native snapshots, including absent/mismatched current schema and unrecoverable construction schema.',
    }
    report['BindingDiagnostic'] = {'expected_from_inputs':expected,'rust_actual':actual,
        'actual_path':str(actual_path),'actual_raw_sha256':hashlib.sha256(actual_path.read_bytes()).hexdigest(),
        'comparison':diagnostic,
        'new_raw_executions':3,'new_complete_conformance_events':0,
        'unaccepted_incremental_root_obligations':42,
        'increment_is_not_folded_into_previous_main_or_development_denominators':True}
    report['outcome'] = 'C_FORMAL_BINDING_GAP'
    report['related_implementation_stopped'] = True
    report['new_complete_capture_case_ids'] = 0


def freeze():
    path = R/'target'/(P+'initial-byte-map.json')
    if not path.exists():
        assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=R,text=True).strip() == BASE
        assert not subprocess.check_output(['git','diff','--name-only'],cwd=R)
        names = subprocess.check_output(['git','ls-files','-z'],cwd=R).decode().split('\0')[:-1]
        records = {n:hashlib.sha256((R/n).read_bytes()).hexdigest() for n in names}
        path.write_text(json.dumps({'head':BASE,'files':records},indent=2)+'\n',encoding='utf8')
    else:
        records = json.loads(path.read_bytes())['files']
    allowed = {'crates/af-computation/tests/raw_port_capture_conformance.rs'}
    changed = [n for n,digest in records.items() if not (R/n).is_file() or hashlib.sha256((R/n).read_bytes()).hexdigest()!=digest]
    assert not set(changed)-allowed, changed
    return {'initial_paths':len(records),'changed_paths':changed,
            'allowed_existing_paths':sorted(allowed),'unauthorized_changes':[],
            'raw_map':str(path)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--materialize', action='store_true')
    args = parser.parse_args()
    preservation = freeze()
    report = baseline()
    binding_gap(report)
    report['FrozenInputPreservation'] = preservation
    if args.materialize:
        (R/'docs/implementation'/(P+'closure-and-binding-audit.json')).write_text(
            json.dumps(report,ensure_ascii=True,separators=(',',':'))+'\n',encoding='utf8')
    print(json.dumps({'outcome':report['outcome'],'required_roots':1750,
        'previous_subset_compared':1000,'missing':625,'partial':125,
        'strict_total_exit':2,'preservation':preservation},indent=2))
    return 2


if __name__ == '__main__':
    sys.exit(main())
