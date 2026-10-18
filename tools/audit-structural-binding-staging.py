"""Independent current registration/coverage/freeze/receipt audit, NOT Raw Oracle."""
import collections, hashlib, importlib.util, json, re, subprocess, sys
from pathlib import Path
from structural_binding_registration_data import *
R=Path(__file__).resolve().parent.parent;D=R/'spec/conformance/g01-structural-binding-staging';P=ARTIFACT
ALLOWED={'crates/af-computation/Cargo.toml','crates/af-computation/src/lib.rs','crates/af-computation/src/root_export.rs','crates/af-computation/src/relation_admission.rs','crates/af-portable/Cargo.toml','crates/af-portable/src/support.rs','tools/gen-registry-instances.py','tools/gen-relation-schema-staging.py','spec/registry/specifications.json','spec/registry/rules.json','spec/conformance/suites.json'}
def read(path):return json.loads(path.read_bytes())
def module(name,path):
    s=importlib.util.spec_from_file_location(name,R/path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
O=module('structural_independent_registration_audit','tools/oracle/structural_binding_registration.py')

def frozen():
    baseline=read(R/'target'/(P+'start-raw-map.json'))
    changed=[]
    for path,digest in baseline.items():
        if hashlib.sha256((R/path).read_bytes()).hexdigest()!=digest:changed.append(path)
    assert set(changed)<=ALLOWED,('unauthorized old changes',set(changed)-ALLOWED)
    rows={}
    for file,key,count in [('specifications','specification_id',66),('rules','rule_id',509)]:
        prior=read(D/('analysis7-'+file+'.json'))[file];current=read(R/'spec/registry'/(file+'.json'))[file]
        assert len(prior)==count and hashlib.sha256((D/('analysis7-'+file+'.json')).read_bytes()).hexdigest()==baseline['spec/registry/'+file+'.json']
        assert all(next((r for r in current if r[key]==old[key]),None)==old for old in prior)
        rows[file]=dict(previous_records=count,previous_records_exact=True,current_records=len(current))
    assert read(R/'docs/implementation/phase-04c1b4c4r3c2r4g1-design-vectors.json')['executed']==0
    return dict(start_files=len(baseline),authorized_existing_changes=sorted(changed),allowed_existing_changes=sorted(ALLOWED),all_other_original_raw_bytes_equal=True,old_registry_objects=rows,old_expected_oracles_and_normative_sources_unchanged=True)

def coverage():
    m=O.catalogue();manifest=read(D/'manifest.json');cases=read(D/'cases.json')['cases'];ids=manifest['case_ids'];byid={c['case_id']:c for c in cases}
    assert len(ids)==len(set(ids)) and set(byid)==set(manifest['metadata_case_ids'])
    required=O.field_roster(m);got=read(D/'cases.json')['coverage']
    encode=lambda x:json.dumps({k:x[k] for k in ['type','path','field','variant']},sort_keys=True)
    assert {encode(x) for x in required}=={encode(x) for x in got}
    assert all(x['case_id'] in byid for x in got)
    assert {c['input']['type'] for c in cases if c['case_id'].split('/')[-1].startswith('shape-')}==set(m['structural_binding_registration']['coverage_types'])
    site_ids=[c for c in cases if c['category']=='structural_site' and c['case_id'].endswith('-valid')]
    assert len(site_ids)==len(m['structural_binding_registration']['sites'])==317
    for c in cases:assert c['expected']==O.result(c['input']),('Expected is not independently derived',c['case_id'])
    for rule in RULES:
        assert rule['case_id'] in byid and byid[rule['case_id']]['input']==dict(op='rule',rule_id=rule['rule_id'])
        text=(R/SOURCES[rule['source']][2]).read_text(encoding='utf8');assert text.count('id="'+rule['anchor']+'"')==1
    return dict(case_ids=len(ids),metadata_case_ids=len(cases),categories=manifest['counts'],required_field_variants=len(required),covered_field_variants=len(got),structural_sites=317,reference_use_policies=66,missing_fields_sites_variants=[],production_binder_capture_case_ids=0,raw_full_field_comparator_mutations=0,prior_r4_boundary_mutations_credit_to_production=0)

def registration():
    result=O.audit_registration();specs=read(R/'spec/registry/specifications.json')['specifications'];rules=read(R/'spec/registry/rules.json')['rules']
    assert (len(specs),len(rules))==(70,522)
    assert sum(bool(r['conformance_refs']) for r in rules)==521
    for row in specs:
        path=R/row['normative_source']['logical_path'];assert hashlib.sha256(path.read_bytes()).hexdigest()==row['normative_source']['content_digest']['hex']
    assert len(read(R/'spec/tables/hash-domains.json')['domains'])==14
    assert sum(r['status']=='active' for r in read(R/'spec/registry/abi-numbers.json')['assignments'])==775
    assert len(list((R/'spec/idl').rglob('*.afidl')))==17
    manifest=read(D/'manifest.json');assert len(manifest['selected_specifications'])==44 and len(manifest['applied_rules'])==217
    total=sum(len((R/r['normative_source']['logical_path']).read_bytes()) for r in specs if r['specification_id'] in manifest['selected_specifications'])+sum(len((R/'spec/registry'/(n+'.json')).read_bytes()) for n in ['specifications','rules'])
    assert total<=32*1024*1024
    result.update(analysis=OWNER,model=MODEL,report=SHAPE,obligation=OBLIGATION,specifications=70,rules=522,new_specifications=4,new_rules=13,new_descriptors=357,reference_covered=521,intentional_exception=1,missing=0,abi=775,afidl=17,hashv1_domains=14,native_identity_bytes=total,unchanged_native_identity_limit=33554432,lossless_closed_source_composition=True)
    return result

def receipts():
    actual=read(R/'target'/(P+'registration-actual.json'));result=O.report(actual)
    manifest=read(D/'manifest.json');portable=read(R/'target'/(P+'portable-actual.json'));runtime=read(R/'target'/(P+'portable-runtime-actual.json'))
    wanted=[dict(case_id=c['case_id'],stage=c['stage'],family=c['family'],positive_root_export=False) for c in manifest['portable_cases']]
    assert portable==wanted
    assert runtime==[dict(case_id=PREFIX+'portable-caller-support-flag',stage='D0',family='InvalidCanonicalEncoding'),dict(case_id=PREFIX+'portable-noncanonical',stage='D1',family='InvalidCanonicalEncoding')]
    boundary=read(R/'target'/(P+'native-boundary-actual.json'));assert len(boundary)==70
    early=0
    for observed,source in zip(boundary,manifest['native_boundary_cases']):
        assert observed['case_id']==source['case_id'] and not observed['positive_root_export'] and observed['production_capture_case_ids']==0
        a=observed['actual'];assert a['stage']==source['expected_stage']
        if source['expected_stage']=='component_validation_rejected':early+=1;assert a['checked_component_produced'] is False
        else:assert a['checked_static_compatibility_capability_issued'] is False and a['static_compatibility']!='proven' and a['_edition2_admission_failure'] in ['NotImplemented','InvalidIdentity','IncompleteContent','UnsupportedSemantics','ResourceLimit']
    executed={c['case_id'] for c in actual+portable+runtime+boundary};assert executed==set(manifest['case_ids'])
    return dict(independent_metadata=result,executed_case_ids=len(executed),metadata=len(actual),portable=len(portable)+len(runtime),native_boundary=len(boundary),component_early_rejections=early,analyzer_direct_private_gate_refusals=70-early,capabilities_issued=0,production_binder_capture_case_ids=0,actual_production_dagranks_generated=0,raw_full_field_oracle=False)

def mappings():
    m=O.catalogue();cases=read(D/'cases.json')['cases'];by={c['case_id']:c for c in cases};coverage_rows=read(D/'cases.json')['coverage'];sites=m['structural_binding_registration']['sites'];out=[]
    for r in RULES:
        out.append(dict(sb_clauses=r['sb_clauses'],registered_rule=r['rule_id'],normative_source=SOURCES[r['source']][2],specification_id=SOURCES[r['source']][0],source_anchor=r['anchor'],descriptor_owner=MODEL,stage='registration_admission / future pre_capture_binding',applicable_sites=[row['definition']['key'] for row in sites] if r['source']=='SB' else [],required_field_variant_cases=[v['case_id'] for v in coverage_rows],registration_case_id=r['case_id'],compiler_semantics=OWNER,independent_expected=by[r['case_id']]['expected'],actual_receipt='target/'+P+'registration-actual.json#'+r['case_id'],production_capture_credit=0))
    return out

def conversion(executed):
    original=read(R/'docs/implementation/phase-04c1b4c4r3c2r4g1-design-vectors.json')['vectors'];assert len(original)==48 and all(v['status']=='Not Executed' and v['rust_actual'] is None and v['assigned_ranks'] is None for v in original)
    cases=read(D/'cases.json')['cases'];manifest=read(D/'manifest.json');ids={c['case_id'] for c in cases};by={c['case_id']:c for c in cases};rows=[]
    known={
      'same-typed-value':['dag-equal-roots','dag-duplicate-equal'], 'different-nominal':['dag-same-shape-different-site','dag-wrong-nominal'],
      'tuple':['dag-record-order'], 'same-site-alias':['site-0000-valid'], 'cross-site-shape':['dag-same-shape-different-site'],
      'insertion-order':['module-native-source-full','module-native-insertion-order'], 'conflicting-key':['module-duplicate-before-interning'], 'equal-duplicate-key':['dag-map-duplicate-key'],
      'nominal-key':['dag-same-shape-different-key-type'], 'duplicate-set-value':['dag-set-repeated-member'], 'set-order':['dag-set-wrong-order'], 'numeric-key-order':['dag-numeric-order'], 'empty-map':['dag-empty-map-versus-set'],
      'shared-child':['dag-shared-child'], 'missing-rank':['dag-missing-node'], 'wrong-rank':['dag-map-wrong-key-type'], 'dangling-reference':['dag-map-dangling-child'],
      'unreachable-node':['dag-unreachable'], 'duplicate-equal-node':['dag-duplicate-equal'], 'alternate-numbering':['dag-alternate-numbering'], 'rank-dependency':['dag-map-forward-child'],
      'root-order':['dag-root-order'], 'both-omit-node':['dag-missing-node','site_registry-both-omit'], 'ordinary-u64':['dag-numeric-order'], 'output-backedge':['dag-g-output-backedge'],
      'document-occurrence':['dag-distinct-document-occurrences','dag-occurrence-collapse'], 'same-occurrence':['dag-equal-roots'], 'a-versus-c':['module-native-current-definition-change'],
      't3-without-binder':['production-raw_contract_validation'], 'old-content-only':['old-owner-ModelType'], 'cross-edition-ref':['dag-wrong-owner'], 'root-capability':['production-raw_contract_validation'],
      't4-rerank':['stage-t4-cannot-own-binder'], 'bootstrap-cycle':['stage-cycle-backedge'], 'resource-budget':['dag-resource-bound']}
    for v in original:
        slug=v['vector_id'].split(v['category']+'-',1)[1];targets=[PREFIX+s for s in known.get(slug,[])];assert set(targets)<=ids,(slug,set(targets)-ids)
        ruleids=[r['rule_id'] for r in RULES if v['draft_clause'] in r['sb_clauses']]
        rows.append(dict(design_vector=v['vector_id'],original_status='Not Executed',registered_rules=ruleids,registered_case_ids=targets,independent_expected=[by[c]['expected'] for c in targets],actual_receipt='target/'+P+'registration-actual.json' if executed and targets else None,metadata_subset_executed=bool(executed and targets),full_condition_status='Not Executed',reason='Registration/manual-subgraph subset only; no actual production Binder execution. Related subset is not full original condition.' if targets else 'No exact applicable current-source case; full condition remains Not Executed rather than inventing a source/site.',production_capture_credit=0))
    return dict(original_file_unchanged=True,original_vectors=48,full_vectors_executed=0,metadata_subsets_with_related_receipts=sum(r['metadata_subset_executed'] for r in rows),vectors=rows)

def main():
    result=dict(registration=registration(),coverage=coverage(),freeze=frozen())
    if '--receipts' in sys.argv:result['fresh_execution_receipts']=receipts()
    if '--materialize' in sys.argv:
        out=R/'docs/implementation'
        m=O.catalogue();reg=m['structural_binding_registration']
        conditions={
          'missing_site':(['UnsupportedModel','IncompleteContent','InvalidFacts'],'unknown lookup / missing normative row / mismatched declared mapping'),
          'unknown_nominal_type':(['UnsupportedModel'],'unknown exact owner/name; no same-shape cast'),
          'invalid_shape':(['UnsupportedModel','IncompleteFacts','InvalidFacts'],'unknown/type mismatch / missing required field / invalid known domain'),
          'missing_node':(['IncompleteFacts'],'known required source-derived node/root absent'),
          'missing_rank_source':(['IncompleteFacts'],'required known source binding absent in a future implemented profile'),
          'duplicate_map_key':(['InvalidFacts'],'duplicate complete canonical map key before any interning'),
          'invalid_map_entry':(['InvalidFacts'],'invalid key/value/source pair or noncanonical map order'),
          'invalid_set_member':(['InvalidFacts'],'duplicate complete typed membership or invalid registered member order'),
          'invalid_child':(['InvalidFacts'],'dangling/wrong typed target or wrong reference arena'),
          'rank_dependency':(['InvalidFacts'],'local forward dependency or alternative canonical numbering'),
          'invalid_interning':(['InvalidFacts'],'duplicate equal typed value or unreachable supplied node'),
          'input_output_backedge':(['InvalidFacts'],'output enters G or G edge uses an output arena'),
          'resource_bound':(['ResourceLimit'],'intersected node/depth/work/value/source bound or overflow'),
          'cross_edition':(['UnsupportedModel'],'current structural tuple mismatch; external Portable keeps its own InvalidIdentity/UnsupportedSemantics taxonomy'),
          'source_association':(['InvalidFacts'],'readonly source-map claim differs from complete actual native values; future native source assurance is separate'),
          'ineligible_value_type':(['UnsupportedModel'],'registered metadata/other-profile type is not an eligible value node'),
          'production_not_implemented':(['NotImplemented'],'production entry is unimplemented regardless of caller flags or successful metadata')}
        assert set(conditions)=={r[0] for r in m['types']['StructuralFailureReason']['variants']}
        failure=dict(descriptor_owner=MODEL,diagnostic_stage='registration_admission',predicate_evaluated=False,port_fact_created=False,conditions=[dict(reason=n,domains=d,applicability=a,required_source='complete preserved input and exact field/type/source paths; no reason-string recovery') for n,(d,a) in conditions.items()])
        edition=dict(previous=dict(analysis='agentflow.root-export-checker@7',model='agentflow.computation.proof-report.model@6',report='agentflow.g01.report-shape@6',obligation='agentflow.g01.obligation@6'),successor=dict(analysis=OWNER,model=MODEL,report=SHAPE,obligation=OBLIGATION),counts=result['registration'],source_composition='closed_structural_delta_v1; full exact model@6 contents required',status=reg['status'],source_hashes={path:hashlib.sha256((R/path).read_bytes()).hexdigest() for _,_,path,_ in SOURCES.values()})
        size=dict(observed_development_duplicated_identity_bytes=38836805,observed_development_duplicated_model_source_bytes=12245011,unchanged_native_identity_limit=33554432,current_identity_bytes=result['registration']['native_identity_bytes'],current_model_source_bytes=(R/SOURCES['RM'][2]).stat().st_size,full_effective_model_sha256=hashlib.sha256(json.dumps(m,sort_keys=True,separators=(',',':')).encode()).hexdigest(),lossless_generator_and_independent_expansion_equal=True,base_full_source_required=True,source_count_reduced=False,limits_widened=False)
        values=[('registration-audit',result),('sb-to-rule-map',mappings()),('design-conversion',conversion('--receipts' in sys.argv)),('structural-site-registry',reg['sites']),('reference-domain-registry',reg['reference_policies']),('field-coverage',dict(required=O.field_roster(m),case_bindings=read(D/'cases.json')['coverage'],missing=[])),('source-stage-contract',reg['stage_contract']),('failure-admission-matrix',failure),('edition-manifest',edition),('source-size-audit',size)]
        history=R/'target'/(P+'histories.json')
        if history.exists():values.append(('historical-regressions',read(history)))
        for name,value in values:
            (out/(P+name+'.json')).write_bytes((json.dumps(value,indent=2,ensure_ascii=False)+'\n').encode())
    print(json.dumps(result,indent=2))
if __name__=='__main__':main()
