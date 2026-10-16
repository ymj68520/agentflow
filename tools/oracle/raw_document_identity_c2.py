"""Independent descriptor/source identity audit. NOT a full Raw Seed Oracle.

Default exit 2 means an open formal binding gap, never conformance success.
--report exits 0 only to let a diagnostic regression assert the known gap.
No Scope encoder, normalizer, Fact constructor, address or synthetic Origin.
"""
import copy
import importlib.util
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MODEL = ROOT / 'spec/tables/proof-report-model-v4.md'
BASE = '23a7c01ab616a268d600ea8b1be9d721341eee31'


def vector():
    # Reuse only the frozen raw input fixture, not its older model@3 audit.
    spec = importlib.util.spec_from_file_location(
        'c2_raw_input', ROOT / 'tools/oracle/native_raw_binding_c1.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    original = module.vector()
    other = copy.deepcopy(original['rust_input'])
    other['inputs'][0]['port_no'] = 1
    return {'case_id': 'c2-binding/independent-identical-native-documents',
            'native_declarations': [original['rust_input'], other],
            'independent_document_occurrences_required': True,
            'selected_entry': 0, 'input_original_ordinal': 0,
            'physical': 'not_supplied'}


def audit(actual):
    expected = vector()  # Independently derived before consulting actual values.
    model = json.loads(re.search(r'```json\s*(.*?)\s*```',
                                MODEL.read_text(encoding='utf-8'), re.S)[1])
    types = model['types']
    assert len(types) == 683
    names = {
        'RawInputOrigin': ['document', 'path'],
        'RawParsedInput': ['candidate', 'origin', 'physical'],
        'RawValidationSubject': ['owner_compiler', 'input', 'context'],
        'RawContextBinding': ['contents', 'roles'],
        'RawValidationOrigin': ['input_origin', 'direction', 'original_ordinal'],
        'Scope': ['edition', 'analysis_semantics', 'content', 'subject'],
        'Origin': ['predicate', 'declared_path', 'source_ordinal', 'mapping_ordinal',
                   'raw_validation_origin'],
        'FactKey': ['stage', 'assertion_path', 'kind', 'dimension', 'source', 'target',
                    'coordinate', 'rule', 'requirement', 'origin'],
    }
    for name, fields in names.items():
        assert [n for n, _ in types[name]['fields']] == fields, name
    documents = dict(types['RawDocument']['variants'])
    assert set(documents) == {'physical_json_utf8', 'declared_native_document'}
    assert [n for n, _ in documents['physical_json_utf8']['fields']] == ['bytes']
    assert [n for n, _ in documents['declared_native_document']['fields']] == ['declarations']
    assert documents['declared_native_document']['fields'][0][1] == {
        'kind': 'sequence', 'item': {'ref': 'RawCandidateContract'}, 'minimum': 0}
    assert dict(types['RawInputOrigin']['fields'])['document'] == {
        'kind': 'dag_ref', 'target': {'ref': 'RawDocument'}}
    assert dict(types['Source']['variants'])['raw_validation'] == {
        'kind': 'record', 'fields': [['subject', {
            'kind': 'dag_ref', 'target': {'ref': 'RawValidationSubject'}}]]}
    assert dict(types['ScopeSubject']['variants'])['raw_contract_validation'] == {
        'ref': 'RawValidationSubject'}
    minimum_dag = [c for c in model['canonical_constraints']
                   if 'Intern equal typed values bottom-up' in c]
    assert len(minimum_dag) == 1
    assert 'redundant equal nodes' in minimum_dag[0]
    assert 'same minimal DAG' in minimum_dag[0]
    assert actual['case_id'] == expected['case_id']
    for name in ['distinct_document_objects', 'distinct_declaration_storage',
                 'same_actual_context_for_both_roles_and_documents',
                 'same_compiler_object_for_both_documents']:
        assert actual[name] is True, name
    assert actual['actual_context_fields'] == [
        n for n, _ in types['ContractProofContextContents']['fields']]
    assert len(actual['actual_context_fields']) == 9
    rows = actual['observations']
    assert len(rows) == 2
    for row in rows:
        assert row == {
            'retained_complete_native_input': expected['native_declarations'],
            'retained_selected_candidate_input': expected['native_declarations'][0],
            'all_typed_declarations_equal': True, 'typed_candidate_equal': True,
            'selected_entry': 0, 'admission': 'Ok(unit)',
            'missing_document': 'MissingDeclarativeOrigin',
            'wrong_entry': 'InvalidSourceAssociation',
            'equal_content_other_context': 'InvalidContextBinding',
            'physical': 'not_supplied',
            'canonical_error': {'rule_id': 'AF-CONTRACT-0004',
                                'code': 'invalid_or_duplicate_port'},
            'checked': False, 'native_guard_records': 0}, row
    assert rows[0] == rows[1]  # Full contents, not digest/count equality.
    assert actual['typed_raw_facts_produced'] == 0
    assert actual['root_export_capability_issued'] is False
    descriptors = {n: types[n] for n in [
        'RawDocument', 'RawInputOrigin', 'RawParsedInput', 'RawPhysicalAssociation',
        'RawValidationSubject', 'RawContextBinding', 'RawContextPartition',
        'RawValidationOrigin', 'Scope', 'CanonicalValueDag', 'Origin', 'FactKey']}
    return {
        'kind': 'IndependentRawDocumentIdentityBindingAudit',
        'outcome': 'C — ADDITIONAL FORMAL BINDING GAP',
        'finding_id': 'C2-GAP-01', 'severity': 'P1 / OPEN / BLOCKING',
        'source_head': BASE,
        'requirement': 'Independent documents with identical complete contents and '
                       'equal declaration paths must have distinct Source identities.',
        'human_clarification': '是，也必须区分独立的同内容文档；如模型不足则按 Outcome C 交付缺口',
        'minimal_input': expected, 'actual_admission_diagnostic': actual,
        'first_unbindable_requirement': 'Stable independent document occurrence '
                                      'identity at RawInputOrigin.document / RawDocument',
        'descriptor_exhaustion': descriptors,
        'raw_source_descriptor': dict(types['Source']['variants'])['raw_validation'],
        'scope_subject_descriptor': dict(types['ScopeSubject']['variants'])['raw_contract_validation'],
        'canonical_dag_rule': minimum_dag[0],
        'proof': [
            'Both real separately owned two-entry documents pass actual source admission.',
            'All thirteen native declaration fields and both complete document values are equal.',
            'The actual same compiler and same nine-member Context object are used in both calls.',
            'Actual entry path, direction, original ordinal and declared member path coincide.',
            'The closed RawDocument variants have content only; there is no occurrence owner binding.',
            'All registered Source/Origin identity inputs therefore coincide by typed value equality.',
            'Node<T> canonical ranks cannot distinguish equal documents: minimal DAG interns equals.',
            'No existing Context partition distinguishes document occurrences; roles bind Context only.',
            'Changing Observations cannot repair the missing Source identity input.',
            'Pending G02 serialization cannot invent an unregistered semantic input.'],
        'scope_of_claim': 'Structural expressibility proof plus live admission diagnostic '
                          'and retained complete deserialization inputs with Rust typed Eq; '
                          'not an actual full-field native projection, executed full '
                          'Scope/Fact encoding or false-Proven finding.',
        'different_contents_or_entry_paths_already_distinguishable': True,
        'c1_gap_01': 'FORMAL BINDING REGISTERED (not reopened)',
        'full_raw_seed_conformance': False, 'raw_capture_case_ids': 0,
        'raw_comparator_mutations': 0, 'producer_hook': 'NotImplemented',
        'required_event_coverage': 'NOT IMPLEMENTED / NOT ZERO GAPS',
        'required_field_coverage': 'NOT IMPLEMENTED / NOT ZERO GAPS',
        'positive_root_export': 'NotAuthorized', 't4': 'NOT ELIGIBLE',
        'required_next_authority': 'Separate normative adjudication of actual document '
                                   'occurrence/owner binding. No registration changes in C2.'}


def main():
    if sys.argv[1:] == ['--vector']:
        print(json.dumps(vector(), ensure_ascii=True))
        return
    assert sys.argv[1:] in ([], ['--report'])
    print(json.dumps(audit(json.load(sys.stdin)), ensure_ascii=True, indent=2))
    if not sys.argv[1:]:
        sys.exit(2)


if __name__ == '__main__':
    main()
