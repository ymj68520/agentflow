"""Independent structural binding audit for a direct nine-member native D6 call.

This does not claim a complete Scope, invent an empty resolver, or issue facts.
It enumerates the frozen Subject alternatives for the concrete actual source.
"""
import importlib.util,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('c1_raw_native',ROOT/'tools/oracle/native_guard_capture.py')
NG=importlib.util.module_from_spec(spec);spec.loader.exec_module(NG)

def vector():
    raw=NG.CC.empty_contract()
    raw['inputs']=[{'port_no':0,'type':NG.reference(NG.schema('int64'))}]
    return {'case_id':'c1-binding/direct-native-raw-port-zero','raw':raw,'rust_input':NG.rust_input(raw)}

def audit(actual):
    m=json.loads(re.search(r'```json\s*(.*?)\s*```',(ROOT/'spec/tables/proof-report-model-v3.md').read_text(encoding='utf-8'),re.S)[1])
    types=m['types'];variants=dict(types['ScopeSubject']['variants'])
    native_fields=[n for n,_ in types['ContractProofContextContents']['fields']]
    old_fields=dict(types['Context']['fields']);validation=dict(variants['contract_validation']['fields'])
    assert validation['context']=={'kind':'dag_ref','target':{'ref':'Context'}}
    assert len(native_fields)==9 and len(old_fields)==10
    assert actual['actual_context_fields']==native_fields
    assert actual['canonical_validation']['passed'] is False
    assert actual['canonical_validation']['code']=='invalid_or_duplicate_port'
    assert actual['checked'] is False and actual['native_observation_admitted'] is False
    assert actual['schema_fact']['verdict']=='violation' and actual['schema_fact']['rule_id']=='AF-CONTRACT-0004'
    missing=sorted(set(old_fields)-set(native_fields));assert missing==['resolver_dependencies']
    assert old_fields['resolver_dependencies']=={'ref':'ResolverDependencyManifest'}
    assert actual['native_records']==0
    alternatives=[]
    for name in variants:
        if name=='contract_validation':
            reason={'missing_actual_input':missing,'registered_context':'Context','actual_context':'ContractProofContextContents',
                    'additional_shape_difference':'relation_snapshots sequence versus three named option slots',
                    'why_pending_scope_is_insufficient':'No actual resolver datum exists at this direct native call; delaying serialization cannot create it.'}
        elif name=='native_recovery_guard':
            reason={'required':'CanonicalNativeContractDeclaration','actual':'canonical validation rejected raw candidate',
                    'case_roster':'seven native guard cases only; not a generic raw contract validator profile'}
        elif name=='standalone_substitution':
            reason={'required':'two genuine Checked operands plus declarative ApplicationDocument','actual':'one rejected Raw candidate; neither operands nor application supplied'}
        else:
            reason={'required':'actual caller/root IR, computation and resolver provenance','actual':'direct native API has none of these inputs'}
        alternatives.append({'scope_subject_variant':name,'usable_without_new_data_or_new_binding':False,'reason':reason})
    return {'kind':'NativeRawValidationSubjectBindingAudit','outcome':'C — FORMAL BINDING GAP',
            'finding_id':'C1-GAP-01','severity':'P1 / BLOCKING','source_head':'98c7f0a8df1da880da104d0f322aa0727d0bcfb4',
            'minimal_input':vector(),'actual':actual,'actual_context_fields':native_fields,
            'registered_old_context_fields':list(old_fields),'subject_alternatives':alternatives,
            'registered_rules':['AF-SEMANTIC-0214','AF-SEMANTIC-0219','AF-SEMANTIC-0230','AF-SEMANTIC-0245','AF-SEMANTIC-0246'],
            'first_unbindable_field':'ScopeSubject.contract_validation.context.resolver_dependencies',
            'producer_site':{'path':'crates/af-contract/src/validate.rs','function':'invalid_contract_report','producer':'DimensionProof::violation(schema-validity)',
                             'earliest_lossless_hook':'analyze_contract before calling invalid_contract_report; raw candidate is still available there'},
            'not_a_native_guard_gap':True,'not_a_scope_serialization_only_gap':True,
            'native_full_seed_complete':False,'r3_complete':False,'positive_root_export':'NotAuthorized',
            'required_next_authority':'Separate normative adjudication of direct native raw-validation subject/context binding; do not create it in C1.'}

def main():
    import sys
    if sys.argv[1:]==['--vector']:print(json.dumps(vector(),ensure_ascii=True));return
    report=audit(json.load(sys.stdin));print(json.dumps(report,ensure_ascii=True))
    if sys.argv[1:]!=['--report']:sys.exit(2)
if __name__=='__main__':main()
