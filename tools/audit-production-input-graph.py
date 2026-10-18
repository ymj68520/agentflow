"""G3 source-derived domain/coverage/freeze audit. Not a T3 or proof issuer."""
import hashlib,importlib.util,json,subprocess,sys
from pathlib import Path
R=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(R/'tools'))
from oracle import production_input_graph as O
from oracle.structural_binding_registration import field_roster
P=O.P
BASE='5ef4e56a3ad5ef9cc3fe050dcbe9b98f678476ea'
D=R/'docs/implementation'
def read(path):return json.loads((R/path).read_bytes())
def save(name,value):
    (D/(P+name+'.json')).write_bytes((json.dumps(value,indent=2)+'\n').encode('utf8'))
def frozen():
    # Raw map is locked before the first G3 edit, including non-ignored files.
    baseline=read('target/'+P+'start-raw-map.json')
    changed=[n for n,h in baseline.items() if not (R/n).is_file() or hashlib.sha256((R/n).read_bytes()).hexdigest()!=h]
    assert set(changed)=={'crates/af-computation/Cargo.toml','crates/af-computation/src/lib.rs'},changed
    reg=O.audit_registration()
    assert (reg['descriptors'],reg['references'],reg['sites'],reg['reference_policies'],reg['required_field_variants'])==(1131,2559,317,66,1297)
    specs=read('spec/registry/specifications.json')['specifications'];rules=read('spec/registry/rules.json')['rules']
    assert (len(specs),len(rules),sum(bool(r['conformance_refs']) for r in rules))==(70,522,521)
    assert len(read('spec/tables/hash-domains.json')['domains'])==14
    assert sum(x['status']=='active' for x in read('spec/registry/abi-numbers.json')['assignments'])==775
    assert len(list((R/'spec/idl').rglob('*.afidl')))==17
    assert read('docs/implementation/phase-04c1b4c4r3c2r4g1-design-vectors.json')['executed']==0
    return dict(start_head=BASE,start_files=len(baseline),changed_old_files=changed,all_other_old_raw_bytes_equal=True,registration=reg,specifications=70,rules=522,reference_covered=521,intentional_exception=1,missing=0,abi=775,afidl=17,hashv1_domains=14,ace1='v1')
def manifests():
    m=O.catalogue();reg=m['structural_binding_registration'];types,fields=O.required_value_shapes()
    source=O.compiler_source()
    assert set(source['support_manifest']['attribution'])=={'full_content_revalidation'}
    assert {r['declaration']['model_missing'] for r in source['support_manifest']['semantic_roles'] if 'model_missing' in r['declaration']}=={'not_formalized'}
    inactive={'StructuralValue0009':'Profile B direct_rules; actual implementation-owned compiler is Profile A',
              'StructuralValue0010':'Profile B transitive_rules; actual implementation-owned compiler is Profile A',
              'StructuralValue0011':'Profile B dependency_edges; actual implementation-owned compiler is Profile A',
              'StructuralValue0061':'future_dependency; current compiler role source has only not_formalized'}
    sites=[]
    for s in reg['sites']:
        definition=s['definition'];eligibility=definition['eligibility'];key=s['alias']
        status='w_output_not_authorized' if eligibility=='w_output' else 'fixed_compiler_source_inactive' if key in inactive else 'required_g_input'
        sites.append(dict(registered_site=s,applicability=status,basis=inactive.get(key,eligibility),actual_source_type=s['source_catalogue_key'],exact_source_path=s['source_path'],target_node_body=definition['kind'],required_child_references=definition['children'],equality='complete nominal typed body with recursively equal typed children; hash candidate never identity',ordering='AF-SEMANTIC-0217; mathematical scalar / UTF8 bytes / lexicographic typed children; not DagRank for Map/Set semantics',producer=None if eligibility=='w_output' else 'input_source::{compiler,context,from_metadata} -> input_graph::Engine::{preflight,build,finish}',oracle='production_input_graph::Builder / validate_graph / strict_compare'))
    counts={s:sum(r['applicability']==s for r in sites) for s in ['required_g_input','fixed_compiler_source_inactive','w_output_not_authorized']}
    assert counts==dict(required_g_input=253,fixed_compiler_source_inactive=4,w_output_not_authorized=60),counts
    sets=[r for r in sites if r['target_node_body']=='set'];assert len(sets)==15
    policies=[]
    for policy in reg['reference_policies']:
        applicable=policy['environment']=='g_value' and policy['domain']=='g_local' and 'raw_contract_validation' in policy['profiles']
        policies.append(dict(registered_policy=policy,applicability='enforced_input_child' if applicable else 'not_an_input_edge; reject if submitted to G',producer='input_graph::Engine::child_key',oracle='Builder::child and validate_graph::edge'))
    failures=read('spec/conformance/production-input-graph/negative-cases.json')+read('spec/conformance/production-input-graph/boundary-cases.json')
    cases=read('spec/conformance/production-input-graph/cases.json')
    manifest=dict(start_head=BASE,analysis=m['analysis_owner'],model=m['model_id'],report=m['model_edition'],obligation=m['obligation_edition'],profile='raw_contract_validation',actual_input_source='caller complete SourcePackage and exact owner/member/document reference; current implementation compiler sources; same actual nine-member Native Context object',source_package_owner_revision='complete supplied package / actual owner anchor / actual document entry ordinal; no fabricated ancestor',document_slot_member='cases.json selects actual package owner/document and internal member; all complete declared native entries deserialize as ComponentContract',compiler='current_registered_analysis_edition; exact 70 normative sources / 4 tables / 44 selected specifications / 217 rules / 10 roles',context='compiled_modules,type_schemas,effect_catalogs,resource_types,flow_catalogs,usage_meters,recovery_operations,execution_universes,relation_snapshots; exact native association',first_original_dynamic_blocker='RawValidationSubject.context.contents.compiled_modules',earlier_static_field='Compiler.specifications; no evidence it dynamically failed before original blocker',stage_contract=reg['stage_contract'],source_closure_types=reg['source_closure_types'],input_types=reg['input_types'],output_types=reg['output_types'],registered_roots=reg['root_profiles'],required_value_types=sorted(types),required_field_variants=[dict(type=k,kind=t,member=v) for k,t,v in sorted(fields)],site_dispositions=counts,sites=sites,sets=sets,reference_policies=policies,production_case_ids=[c['case_id'] for c in cases],negative_case_ids=[c['case_id'] for c in failures],required_failures=failures,old_registration_case_credit=0,old_r4_mutation_credit=0,g1_vector_credit=0,capability=False,raw_full_field=False)
    manifest['registered_required_field_variants']=field_roster(m)
    assert len(manifest['registered_required_field_variants'])==1297
    manifest['registered_model_type_ranks']=m['type_ranks']
    return manifest
def receipts(manifest):
    result=read('target/'+P+'input-g-oracle-results.json');runtime=read('target/'+P+'runtime-coverage-ledger.json')
    assert result['case_ids']==len(manifest['production_case_ids'])==154
    assert result['required']==result['compared'] and result['missing']==result['unexpected']==result['uncompared']==0
    assert (result['required_types'],result['required_field_variants'])==(455,538)
    assert set(result['executed_sites'])=={r['registered_site']['alias'] for r in manifest['sites'] if r['applicability']=='required_g_input'}
    for row in manifest['sites']:
        row['production_cases']=runtime['sites'].get(row['registered_site']['alias'],[])
        assert bool(row['production_cases'])==(row['applicability']=='required_g_input')
    failures=[]
    for name,file in [('negative-cases','source-negative-actual'),('boundary-cases','boundary-actual')]:
        wanted=read('spec/conformance/production-input-graph/'+name+'.json');actual=read('target/'+P+file+'.json')
        assert len(wanted)==len(actual)
        for w,a in zip(wanted,actual):
            assert a['case_id']==w['case_id'] and a['graph_returned'] is a['capability'] is a['raw_fact'] is False
            for k in ['domain','reason','stage']:assert a['failure'][k]==w[k],(w,a)
            failures.append(dict(case_id=w['case_id'],expected=w,actual=a))
    compatibility=[]
    snapshot_sources=[]
    for shard in range(4):
        for row in read('target/'+P+f'shard-{shard}.json'):
            assert row['positive_root_export'] is row['complete_raw_fact'] is False
            c=row['c_compatibility']
            if c['executed_outside_binder']:
                assert not c['model7_subject_admission']
                assert c['complete_partition']==(row['input_case']['current_modules']=='complete')
                compatibility.append(dict(case_id=row['case_id'],**c))
                snapshot_sources.append(O.case_context(row['input_case'])['contents']['relation_snapshots'])
    assert {r['complete_partition'] for r in compatibility}=={True,False}
    assert snapshot_sources and all(s==snapshot_sources[0] for s in snapshot_sources)
    mutations=read('target/'+P+'mutations.json');assert mutations['mutations']>=40 and mutations['unexpected_accepted']==0
    return dict(outcome='INPUT-G CONFORMANCE PASS; ENGINEERING/HISTORICAL/GENERATOR GATES SEPARATE',production_case_ids=154,source_negative_case_ids=23,resource_nominal_boundaries=3,comparator_mutations=mutations['mutations'],runtime=result,failures=failures,c_layer_outside_binder=compatibility,missing=0,unexpected=0,uncompared=0,positive_root_export=False,raw_full_field=False)
if __name__=='__main__':
    manifest=manifests();result=dict(freeze=frozen(),required_types=len(manifest['required_value_types']),required_field_variants=len(manifest['required_field_variants']),sites=manifest['site_dispositions'],registration_cases=3662,production_cases=len(manifest['production_case_ids']))
    if '--receipts' in sys.argv:result['receipts']=receipts(manifest)
    if '--write' in sys.argv:
        save('production-binder-input-manifest',manifest);save('production-audit',result)
        vectors=read('docs/implementation/phase-04c1b4c4r3c2r4g1-design-vectors.json')['vectors']
        assert len(vectors)==48
        save('g1-vector-conversion',dict(original_vector_count=48,converted_executed=0,not_executed=48,production_credit=0,vectors=[dict(vector_id=v['vector_id'],status='Not Executed',credit=0,basis='Exact original vector execution contract not separately run; related production inputs and comparator attacks do not automatically convert it') for v in vectors]))
        if '--receipts' in sys.argv:save('runtime-coverage-ledger',read('target/'+P+'runtime-coverage-ledger.json'))
    print(json.dumps({k:v for k,v in result.items() if k!='receipts'},indent=2))
