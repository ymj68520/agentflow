"""Read-only G1 evidence/design metadata audit. NO value DAG/rank generation.

--materialize writes additive design inventories only. It does not execute the
48 design vectors, build Node<T>, encode Map values or implement any Oracle.
"""
import argparse
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import re

R = Path(__file__).resolve().parents[1]
P = 'phase-04c1b4c4r3c2r4g1-'
OLD = 'phase-04c1b4c4r3c2r4-'
D = R / 'docs/implementation'
BASE = '328e637dff876bc1791f0ecd156c856751955556'


def load(p):
    return json.loads((R / p).read_bytes())


def model():
    return json.loads(re.search(r'```json\s*(.*?)\s*```', (R / 'spec/tables/proof-report-model-v6.md').read_text(encoding='utf8'), re.S)[1])


def nodes(v):
    yield v
    if isinstance(v, dict):
        for x in v.values():
            yield from nodes(x)
    elif isinstance(v, list):
        for x in v:
            yield from nodes(x)


def nominal(m, key):
    a, slash, b = key.partition('/')
    if slash and re.fullmatch(r'[a-z][a-z0-9-]*(?:\.[a-z][a-z0-9-]*)*@[1-9][0-9]*', a):
        return dict(owner=a, name=b)
    return dict(owner=m['model_id'], name=key)


def vectors():
    groups = [
      ('anonymous', [
       ('same-typed-value','Two equal values at the same registered site','Intern one typed value; preserve two parent edges','SB-02'),
       ('different-nominal','Same record shape under two declared owners','Distinct types and nodes; no structural cast','SB-02'),
       ('nested-record','Record contains an inline record','Both sites need closed registered alias bindings','SB-01'),
       ('inline-struct','FullSupportManifest.closed_tables complete four byte fields','Bind declared four-field record, not count/digest','SB-01'),
       ('tuple','Actual anonymous tuple-shaped declared payload','Use exact registered positional fields/order; no invented Rust layout','SB-03'),
       ('nested-option','None versus Some(None) versus Some(Some(value))','Three typed values; preserve explicit tags','SB-03'),
       ('sequence-order','Equal members in opposite sequence order','Different sequence values; preserve occurrences','SB-03'),
       ('closed-variant','Known case with complete inline payload','Resolve case site and exact payload type','SB-03'),
       ('unknown-variant','Unknown case supplied','UnsupportedModel; no node or port verdict rewrite','SB-12'),
       ('missing-record-field','Declared inline record field absent','IncompleteFacts or closed-shape refusal, no defaults','SB-12'),
       ('same-site-alias','CompleteCompiler aliases Compiler declaration body','Preserve expected nominal ref; share only explicitly identical child sites','SB-02'),
       ('cross-site-shape','Two different fields have equal structural descriptors','Different registered site identities unless expressly aliased','SB-01')]),
      ('map-set', [
       ('insertion-order','Same typed map inserted in opposite orders','One canonical typed map; key comparison independent of insertion','SB-04'),
       ('conflicting-key','Same complete canonical key with different values','Reject duplicate key before interning; no last writer','SB-04'),
       ('equal-duplicate-key','Repeated canonical key with equal value','Reject raw duplicate entries; no repair by deduplication','SB-04'),
       ('nominal-key','Same spelling/width under different nominal key types','No coercion; validate exact registered map key type','SB-02'),
       ('nested-map','Map values contain another anonymous map','Resolve both closed map-site aliases and key/value types','SB-04'),
       ('optional-record-map','Map values are None or Some(complete record)','Preserve option tags and record field roster','SB-04'),
       ('duplicate-set-value','Repeated equal typed Set members','Reject duplicate input membership, not sequence conversion','SB-05'),
       ('set-order','Same valid typed set in different host orders','Canonical full typed member order; no HashMap/Debug order','SB-05'),
       ('numeric-key-order','U64 key two versus key ten','Mathematical numeric order, not decimal text or rank order','SB-06'),
       ('nfc-key-collision','NFC-equivalent Map<string,T> keys','Apply existing string-map domain normalization then reject collision','SB-04'),
       ('raw-byte-key','Distinct Raw byte keys with canonically equivalent text display','Keep distinct bytes where Raw descriptor requires bytes; no NFC repair','SB-06'),
       ('empty-map','Actual empty typed Map versus empty typed Set','Distinct nominal types/body cases; no unspecified empty node','SB-04')]),
      ('node-rank', [
       ('shared-child','Two parents reference one equal child','Minimal one child; retain both valid typed edges','SB-07'),
       ('missing-rank','Required input reference has no binding','IncompleteFacts; no allocated placeholder rank','SB-12'),
       ('wrong-rank','Reference resolves to a different value/type','InvalidFacts; compare actual target and expected descriptor','SB-08'),
       ('dangling-reference','Reference target not in declared carrier','InvalidFacts; no numeric lookup in another arena','SB-08'),
       ('unreachable-node','Extra valid node unrelated to authorized roots','Reject unreachable node; no synthetic root to save it','SB-07'),
       ('duplicate-equal-node','Equal subtree stored twice','Reject nonminimal graph; canonical interning first','SB-07'),
       ('alternate-numbering','Same graph with arbitrary node permutation','Reject alternative numbering under frozen canonical rules','SB-07'),
       ('rank-dependency','Local child not before its parent','Reject local topological/rank violation','SB-07'),
       ('root-order','Authorized subject roots reordered','Reject wrong profile root order; no caller selection','SB-07'),
       ('both-omit-node','Expected and Actual both delete a required node','Independent input-derived roster still requires it; reject','SB-15'),
       ('ordinary-u64','U64 data happens to equal a reference spelling','Scalar value is not a Node reference; retain descriptor distinction','SB-08'),
       ('output-backedge','Input value refers to generated Fact or C receipt','Reject cycle/backedge before construction; no rank backfill','SB-09')]),
      ('stage-history-authority', [
       ('document-occurrence','Equal complete documents at distinct valid owner/slots','May share content; distinct complete occurrence/source inputs','SB-10'),
       ('same-occurrence','Repeated observation of same full scoped fact','Same origin; union legitimate observations only','SB-10'),
       ('a-versus-c','Equal Native snapshots under different current modules','A equality preserved; C binds actual distinct Context/current schema','SB-11'),
       ('t3-without-binder','Sealed local event available but no canonical input carrier','Keep PendingScope/local evidence; not complete registered Fact','SB-13'),
       ('old-content-only','Old edition content-only/local Fact submitted as new carrier','No relabeling; preserve source and reject migration','SB-14'),
       ('cross-edition-ref','Input reference uses another exact model/graph environment','Reject even if rank spelling and shape agree','SB-08'),
       ('root-capability','Structural binding returns successfully','No CompleteView/CheckedRootExport/Portable positive proof','SB-16'),
       ('c-missing','Some relation snapshot, required current module missing','Preserve A and C Missing and port event; no full Fact','SB-11'),
       ('split-context','Equal content but analysis/canonical objects are distinct','Reject actual association; bytes/partition claim is insufficient','SB-10'),
       ('t4-rerank','T4 attempts to append input nodes and reuse old references','Fail incomplete roster; require new immutable carrier and fresh admission/capture','SB-13'),
       ('bootstrap-cycle','Binder requests Fact/Obligation/T4 result as input','Reject dependency cycle; prebinding consumes declared inputs only','SB-09'),
       ('resource-budget','Binding exceeds intersected work/node/depth/output bounds','ResourceLimit; no successful partial graph or budget widening','SB-12')])]
    out = []
    for group, rows in groups:
        assert len(rows) == 12
        for slug, inp, expected, clause in rows:
            out.append(dict(vector_id='R4G1-' + group + '-' + slug, category=group,
                            input_design=inp, expected_design=expected, draft_clause=clause,
                            status='Not Executed', assigned_ranks=None,
                            rust_actual=None, source_of_expected='Frozen registered semantics + explicitly proposed G1 design; not Rust Actual'))
    assert len(out) == 48 and len({x['vector_id'] for x in out}) == 48
    return dict(status='DESIGN ONLY — NOT REGISTERED', vectors=out,
                categories=dict(Counter(x['category'] for x in out)), executed=0,
                previous_r4_boundary_mutations_credit_to_these_vectors=0)


def audit():
    m = model();t = m['types']
    assert len(t) == 774
    specs = load('spec/registry/specifications.json')['specifications']
    rules = load('spec/registry/rules.json')['rules']
    refs = [v['ref'] for v in nodes(t) if isinstance(v, dict) and 'ref' in v]
    assert len(refs) == 2204 and all(n in t for n in refs)
    prior = {}
    for rel in ['docs/implementation/' + OLD + x for x in ['final-report.md','binding-gap.md','coverage-receipt.json','model-diff.json']] + ['target/' + OLD + 'final-runtime-results.json']:
        raw = (R / rel).read_bytes()
        val = json.loads(raw) if rel.endswith('.json') else raw.decode('utf8')
        prior[rel] = dict(raw_sha256=hashlib.sha256(raw).hexdigest(), bytes=len(raw),
                          complete_structured_walk_nodes=sum(1 for _ in nodes(val)), review='ENTIRE FILE PARSED/TRAVERSED; claims below use explicit field paths')
    diff = load('docs/implementation/' + OLD + 'model-diff.json')
    cov = load('docs/implementation/' + OLD + 'coverage-receipt.json')
    act = load('target/' + OLD + 'binding-actual.json')
    assert cov['events']['actual_events'] == 177 and cov['applicable_events'] == 176
    assert cov['root_coverage']['required_instances'] == 2464 and cov['root_coverage']['full_fact_comparisons'] == 0
    assert len(cov['boundary_comparator_mutations']) == 74
    authority_ids = ['AF-ACE1-%04d' % i for i in range(1,15)] + ['AF-SEMANTIC-%04d' % i for i in [197,198,200,208,210,211,212,213,214,215,216,217,218,219,220,221,223,224,225,226,227,228,247,255,257,261,262,263,264,265,266,267,268,269,272,273,275,277,278,280,281,282,283,286,289,290,291,292,293,294,295,296,297,298,299,300,301]]
    sources = {r['specification_id']: r for r in specs}
    authority = []
    for rid in authority_ids:
        rule = next(r for r in rules if r['rule_id'] == rid)
        src = rule['normative_source'];raw = (R / src).read_bytes();text = raw.decode('utf8')
        assert 'id="' + rule['source_anchor'] + '"' in text
        spec = next(s for s in specs if s['normative_source']['logical_path'] == src)
        assert hashlib.sha256(raw).hexdigest() == spec['normative_source']['content_digest']['hex']
        authority.append(dict(specification_id=spec['specification_id'], descriptor_application_owner=m['model_id'], **rule))
    mapping = {'compiled_modules':'modules','type_schemas':'schema_universe','effect_catalogs':'effects','resource_types':'resources','flow_catalogs':'flow','usage_meters':'usage_meters','recovery_operations':'recovery','execution_universes':'execution','relation_snapshots':'relations'}
    fields = []
    for n,d in t['ContractProofContextContents']['fields']:
        fields.append(dict(owner=m['model_id'], type='ContractProofContextContents', field_path=['RawValidationSubject','context','contents',n], descriptor=d,
                           actual_rust_source='StaticContractProofContext.' + mapping[n], actual_source_file='crates/af-contract/src/validate.rs',
                           r4_actual_value_pointer='/context/' + n, status='ACTUAL REPRODUCED LOGICAL METADATA; COMPLETE NODE CARRIER NOT VERIFIED',
                           producing_stage='Native input / T3 source preservation', current_rank_source=None,
                           proposed_rank_owner='PROPOSED pre-capture structural binding; not registered or implemented'))
    static = diff['anonymous_structural_descriptor_sites']
    first_static = next(s for s in static if s['descriptor_path'] == ['Compiler','fields',3,1])
    first = dict(recorded_reproduction_path=diff['representation_gap']['path'],
                 first_meaning='FIRST R4 MACHINE-RECORDED REPRODUCED SITE; NOT GLOBAL RANK/EXECUTION ORDER',
                 expected_enclosing_node='Node<RawValidationSubject>', expected_child_type='anonymous Map<SpecificationId,CompiledModuleContents>',
                 dagrank_domain=t['$U64'], descriptor_owner=m['model_id'], descriptor=diff['representation_gap']['field_descriptor'],
                 actual_rust_type='StaticContractProofContext.modules: &af_idl::CompiledModuleSet; private BTreeMap<String, CompiledSchema>',
                 actual_source=['crates/af-contract/src/validate.rs','crates/af-idl/src/resolve.rs','crates/af-idl/src/canonical.rs'],
                 actual_value_source='target/' + OLD + 'binding-actual.json#/context/compiled_modules',
                 actual_value=act['context']['compiled_modules'], native_rule='AF-CONTRACT-0004', actual_port_verdict=act['native_verdict'],
                 producing_stage='Native declarations / T3 source preservation', rank_source=None,
                 required_binding_rules=['AF-SEMANTIC-0214','AF-SEMANTIC-0215','AF-SEMANTIC-0217','AF-SEMANTIC-0262'],
                 missing_source=['Registered structural-site-to-ModelType identity binding','Registered Map NodeBody binding','Registered pre-capture authority to provide canonical carrier'],
                 earliest_static_declared_field_dfs=dict(path=['RawValidationSubject','owner_compiler','CompleteCompiler','Compiler','specifications'], descriptor=first_static['descriptor'],
                                                        status='STATIC REACHABLE; FULL COMPILER RUNTIME PROJECTION NOT MATERIALIZED IN R4',
                                                        order_is_not_registered_runtime_order=True))
    roster=[]
    for s in static:
        key=s['descriptor_path'][0]
        roster.append(dict(**nominal(m,key), catalogue_key=key, descriptor_path=s['descriptor_path'], kind=s['descriptor']['kind'],
                           descriptor=s['descriptor'], matching_named_descriptors=s['matching_registered_names'],
                           status='STATIC REACHABLE', semantic_applicability='VALUE/VARIANT DEPENDENT; NOT A RUNTIME FIELD COUNT',
                           complete_carrier_status='NOT VERIFIED', rank_source=None))
    contract_names=['ModelType','DagNode','NodeBody','CanonicalValueDag','ScalarValue','Scope','RawValidationSubject','RawContextBinding','Compiler','FullSupportManifest','ContractProofContextContents','CurrentContextRelationSchemaValidation','RelationSubjectAdmission']
    compiler_sources={
        'normative_sources':('RegisteredAnalysisEdition.registered_sources','actual implementation-owned full66 source entries'),
        'closed_tables':('RegisteredAnalysisEdition.closed_tables','actual four complete byte arrays; complete Raw typed projection absent in R4'),
        'selected_specifications':('RegisteredAnalysisEdition.compiler.specifications','actual40 complete selected identities'),
        'applied_rules':('RegisteredAnalysisEdition.compiler.rules','actual204 attributed rule identities, NOT transitive closure'),
        'semantic_roles':('full existing Portable manifest / frozen compiler role and schema-pin definitions','NO complete native Raw projection captured in R4'),
        'attribution':('full existing Portable manifest / full_content_revalidation profile','NO complete native Raw projection captured in R4'),
        'report_shape':('RegisteredAnalysisEdition.report_shape','exact current label AND full matching source contents'),
        'obligation_edition':('RegisteredAnalysisEdition.obligation_edition','exact current label AND full matching source contents')}
    compiler_matrix=[dict(owner=m['model_id'],type='FullSupportManifest',field_path=['RawValidationSubject','owner_compiler','support_manifest',n],descriptor=d,actual_source=compiler_sources[n][0],actual_source_status=compiler_sources[n][1],runtime_full_typed_value='NOT MATERIALIZED IN R4',rank_source=None,source_preservation_owner='T3 capture',node_binding_owner='PROPOSED separate structural input binding',normalizer_owner='T4 consumes immutable G, no input rerank',scope_owner='G02 complete lossless integration remains separately pending') for n,d in t['FullSupportManifest']['fields']]
    category_review=[
      dict(category='anonymous_record',example=['ContractProofContextContents','resource_types'],status='ACTUAL REPRODUCED metadata/name-refusal probe',semantic_applicability='actual empty schemas/descriptors record',carrier='NOT VERIFIED'),
      dict(category='inline_struct',example=['FullSupportManifest','closed_tables'],status='STATIC REACHABLE',semantic_applicability='required full Compiler projection not materialized',carrier='NOT VERIFIED'),
      dict(category='map',example=diff['representation_gap']['path'],status='ACTUAL REPRODUCED',semantic_applicability='actual16-module nonempty map',carrier='UNBOUND at recorded current nominal/body boundary'),
      dict(category='set',example=['ContractProofContextContents','flow_catalogs','confidentiality'],status='ACTUAL LOGICAL EMPTY VALUE; STATIC SITE BINDING REVIEW',semantic_applicability='Set body EXISTS; no runtime member instances in this minimal probe',carrier='NO claim of same missing-body gap as Map'),
      dict(category='nested_option',example=['RawValidationSubject','input','candidate'],status='STATIC REACHABLE / VARIANT DEPENDENT',semantic_applicability='new nesting vectors Not Executed',carrier='NOT VERIFIED'),
      dict(category='nested_sequence',example=['CompiledModuleContents','types'],status='ACTUAL LOGICAL VALUE; STATIC NODE SITE',semantic_applicability='full schema list in reproduced Context',carrier='not independently materialized as typed node'),
      dict(category='nested_variant',example=['af-idl::CanonicalTypeSchema','body'],status='ACTUAL LOGICAL SCHEMA METADATA; STATIC NODE SITE',semantic_applicability='only selected actual cases, not every reachable variant',carrier='NOT VERIFIED')]
    return dict(status='DESIGN AUDIT ONLY', start_head=BASE, previous_evidence_review=prior,
                authority=authority, descriptors={n:t[n] for n in contract_names}, canonical_constraints=m['canonical_constraints'], bounds=m['bounds'],
                registered_counts=dict(specifications=len(specs),rules=len(rules),reference_covered=sum(bool(x['conformance_refs']) for x in rules),intentional_exception=1,missing=0,descriptors=len(t),references=len(refs),historical_imports=len(m['historical_selected_imports']),abi=775,afidl=17,hashv1_domains=14),
                first_unbound=first, context_field_matrix=fields, full_support_manifest=dict(owner=m['model_id'],descriptor=t['FullSupportManifest'], closed_tables=dict(t['FullSupportManifest']['fields'])['closed_tables'], actual_complete_runtime_projection='NOT MATERIALIZED IN R4'),
                structural_sites=roster, structural_category_counts=dict(Counter(s['kind'] for s in roster)), category_review=category_review, compiler_field_stage_matrix=compiler_matrix,
                set_body_registered='set' in [n for n,_ in t['NodeBody']['variants']], map_body_registered='map' in [n for n,_ in t['NodeBody']['variants']],
                local_typed_fact_seed_registered_descriptor=t.get('LocalTypedFactSeed'),
                local_entry_contract='AF-SEMANTIC-0268 Raw hook + AF-SEMANTIC-0255 Native NG-LINKS; name LocalTypedFactSeed is not a closed ModelType',
                ownership_existing='No registered implemented pre-capture canonical rank allocator; AF-SEMANTIC-0217 supplies mathematical constraints, not an executable T3 binding source',
                retained_coverage=dict(port_events=177,otherwise_full_fact_applicable=176,required_roots=2464,full_field_compared=0,nested=None,relations=None,verdict_only_coordinate_gaps=3),
                new_design_vector_status='48 Not Executed', new_design_vector_executions=0, formal_ranks_generated=0,
                normative_registration=False, edition_allocation=False, raw_verified=False, t4='NOT ELIGIBLE', positive_root_export='NotAuthorized')


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--materialize',action='store_true');args=parser.parse_args()
    a=audit();v=vectors()
    if args.materialize:
        for filename,value in [('authority-and-field-audit.json',a),('first-unbound-field.json',a['first_unbound']),('design-vectors.json',v)]:
            (D/(P+filename)).write_text(json.dumps(value,indent=2,ensure_ascii=False)+'\n',encoding='utf8')
    print(json.dumps(dict(counts=a['registered_counts'],structural_sites=len(a['structural_sites']),categories=a['structural_category_counts'],first_reproduced=a['first_unbound']['recorded_reproduction_path'],first_static=a['first_unbound']['earliest_static_declared_field_dfs']['path'],new_design_vectors=len(v['vectors']),executed=0,ranks_generated=0),indent=2))


if __name__=='__main__':main()
