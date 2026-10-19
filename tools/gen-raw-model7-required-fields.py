"""R5-A descriptor-derived census, never a runtime coverage or producer claim.

The full Fact closure includes other producer profiles. Conditional rows are kept,
not deleted to improve Raw coverage. Runtime applicability must be derived from
actual source and the reached predicate, independently of proposed Fact values.
"""
import hashlib,json,sys
from pathlib import Path
R=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(R/'tools'))
from oracle.structural_binding_registration import catalogue,closure,nominal,children
P='phase-04c1b4c4r3c2r5-'
ROOT_SOURCES={
 'key':'source-bound G Subject, actual hook occurrence, AF-CONTRACT-0004',
 'role':'RC-PREDICATE supporting_fact', 'evaluation':'actual reached canonical_ports branch',
 'status':'actual PortCondition: positive_fresh/zero/duplicate',
 'prerequisites':'actual queried prerequisite relations; never namespace/schema inference',
 'references':'actual member; duplicate also actual predecessor member',
 'primary':'typed minimum of complete required references',
 'contradiction':'actual branch zero/duplicate witness; None for positive_fresh',
 'cause_fact_keys':'actual queried causes, not diagnostic reasons',
 'companion_evidence_status':'None: this predicate has no companion authority query',
 'observations':'actual local capture observation and legitimate repeated exposures',
 'substitution_basis':'None: another producer family',
 'native_guard_basis':'None: another producer family',
 'raw_validation_basis':'actual canonical_ports branch plus admitted G/C input',
}
def census():
    m=catalogue();t=m['types'];roots=t['Fact']['fields'];rows=[]
    for n in sorted(closure(t,['Fact'])):
        pending=[(t[n],[],None)]
        while pending:
            d,path,variant=pending.pop()
            if 'ref' in d:continue
            kind=d.get('kind')
            if kind=='record':
                for f,child in d['fields']:
                    rows.append(dict(descriptor_owner=nominal(n)['owner'],type_name=nominal(n)['name'],
                      catalogue_key=n,variant=variant,field_path=path+[['field',f]],descriptor=child,
                      requiredness='present in every instance of this active record; explicit Option tags required',
                      applicability='conditional on reached Raw event and active enclosing variant/container instance; other profile alternatives are not execution credit',
                      producer_source=ROOT_SOURCES[f] if n=='Fact' else 'exact registered declaration; source association must be proved per runtime instance',
                      stage='G input for input closure; otherwise post-G actual C then T3 output_record',
                      independent_expected_rule='source-derived descriptor walk plus registered branch semantics; never Expected/Actual presence',
                      reference_environment='explicit per-use policy for dag_ref; ordinary values are not ranks'))
            if kind=='enum':
                for tag,child in d['variants']:
                    rows.append(dict(descriptor_owner=nominal(n)['owner'],type_name=nominal(n)['name'],
                      catalogue_key=n,variant=tag,field_path=path+[['case',tag]],descriptor=child,
                      requiredness='exactly the active closed case, including unit payload',
                      applicability='source-derived active case only; other cases retained as conditional',
                      producer_source='actual source or typed branch, not a diagnostic string',stage='exact source stage',
                      independent_expected_rule='registered declaration order and source-derived tag',
                      reference_environment='exact per-use policy, not declaring-type membership alone'))
            for p,child in children(d):
                pending.append((child,path+p,p[0][1] if p[0][0]=='case' else variant))
    return dict(phase='R3-C2R5',model=m['model_id'],analysis=m['analysis_owner'],
      report=m['model_edition'],obligation=m['obligation_edition'],
      physical_sources={str(p.relative_to(R)).replace('\\','/'):hashlib.sha256(p.read_bytes()).hexdigest()
        for p in [R/'spec/tables/proof-report-model-v6.md',R/'spec/tables/proof-report-model-v7.md']},
      fact_roots=[dict(name=n,descriptor=d,producer_source=ROOT_SOURCES[n]) for n,d in roots],
      transitive_named_descriptors=sorted(closure(t,['Fact'])),declaration_rows=rows,
      reference_policies=m['structural_binding_registration']['reference_policies'],
      static_rows_are_not_runtime_denominators=True,full_field_capture_credit=0)
if __name__=='__main__':
    result=census();p=R/'docs/implementation'/(P+'raw-model7-required-field-manifest.json')
    p.write_bytes((json.dumps(result,ensure_ascii=False,indent=2)+'\n').encode('utf8'))
    print(json.dumps(dict(roots=len(result['fact_roots']),named=len(result['transitive_named_descriptors']),declaration_rows=len(result['declaration_rows']),runtime_credit=0)))
