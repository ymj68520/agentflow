"""Design artifact audit only. Never implements a schema-binding Producer/Oracle.

--materialize emits input-derived census, hand-specified DESIGN vectors and the
unchanged independent minimal diagnostic Expected. No vector Actual is run.
"""
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys

R = Path(__file__).resolve().parents[1]
P = 'phase-04c1b4c4r3c2r3g1-'
D = R / 'docs/implementation'
OWNER = 'agentflow.computation.proof-report.model@5'


def read_model():
    return json.loads(re.search(r'```json\s*(.*?)\s*```',
        (R/'spec/tables/proof-report-model-v5.md').read_text(encoding='utf8'), re.S)[1])


def refs(value, path=()):
    if isinstance(value, dict):
        if 'ref' in value:
            yield {'path': list(path), 'target': value['ref']}
        for k, v in value.items():
            yield from refs(v, (*path, k))
    elif isinstance(value, list):
        for i, v in enumerate(value):
            yield from refs(v, (*path, i))


def descriptors(m):
    assert len(m['historical_selected_imports']) == 97
    rows = []
    for n, t in m['types'].items():
        rows.append(dict(name=n, owner=OWNER, rank=m['type_ranks'][n],
                         descriptor=t, references=list(refs(t))))
    assert len(rows) == 704 and sum(len(r['references']) for r in rows) == 2064
    selected = ['RelationSnapshotContent', 'Raw(RelationSnapshotContent)',
        'ContractProofContextContents', 'RawValidationSubject', 'RawContextBinding',
        'af-semantic::TypeSchemaRef', 'af-semantic::DigestRefValue', 'SpecificationId']
    incoming = {n: [dict(source=r['name'], **edge) for r in rows
                   for edge in r['references'] if edge['target'] == n] for n in selected}
    return dict(owner=OWNER, analysis=m['analysis_owner'], model=m['model_id'],
        report=m['model_edition'], obligation=m['obligation_edition'],
        descriptors=704, references=2064, imports=len(m['historical_selected_imports']),
        selected_incoming=incoming, all_descriptors=rows)


def slots(m):
    fields = dict(m['types']['ContractProofContextContents']['fields'])
    assert len(fields) == 9
    actual = fields['relation_snapshots']['fields']
    native = (R/'crates/af-contract/src/universe.rs').read_text(encoding='utf8')
    names = re.findall(r'impl RelationCatalogWire for (\w+) \{\s*const SCHEMA_ID:.*?= "([^"]+)";\s*const ROOT_TYPE:.*?= "([^"]+)";', native)
    assert len(names) == 3
    result = []
    for (slot, desc), (catalog, family, root) in zip(actual, names):
        assert desc == {'kind': 'option', 'item': {'ref': 'RelationSnapshotContent'}, 'minimum': 0}
        assert re.search(r'pub '+slot+r': Option<RelationCatalogSnapshot<'+catalog+r'>>', native)
        result.append(dict(slot=slot, registered_descriptor=desc, owner=OWNER,
            native_source='StaticContractProofContext.relations.'+slot,
            native_type='Option<af-contract::RelationCatalogSnapshot<'+catalog+'>>',
            family=family, root_type=root, construction='new(revision, catalog, &CompiledModuleSet); schema not retained',
            current_lookup='actual_context.modules.type_by_identity('+family+'/'+root+')',
            required_meaning='source: family; successful current-validation output: exact TypeSchemaRef; never construction history',
            missing_source='None preserved; Some without current schema blocks complete Subject admission, not local port verdict',
            typed_projection='DRAFT A native-family source + separate DRAFT C validation observation',
            new_design_actual_status='Not Executed',
            inherited_dynamic_diagnostic=(slot == 'effect_resource')))
    return dict(context_members=list(fields), slots=result)


def binding_candidates():
    # Complete != having a nominal record alone. Every candidate is explicitly
    # evaluated for the missing native-family -> full-pin semantic mapping.
    rows = [
        ('RelationCatalogPin', False, True, True, False, False, True,
         'crates/af-contract/src/proof.rs:92; native String/revision/checksum only'),
        ('RelationSnapshotContent', True, True, False, False, False, False,
         'model@5 canonical_constraints + types.RelationSnapshotContent; TypeSchemaRef required, source unstated'),
        ('ContractProofContextContents', True, True, True, False, True, False,
         'model@5 exact nine members; AF-SEMANTIC-0198/0262 supplies actual context, not old schema'),
        ('af-semantic::TypeSchemaRef', True, True, False, False, False, False,
         'model@5 imported three-field record; af-semantic/src/types.rs authored-against meaning'),
        ('SpecificationId', True, True, True, False, False, True,
         'model@5 nominal nfc_text; af-idl/src/specification.rs strict family syntax only'),
        ('Relation Catalog Schema', True, True, True, False, True, False,
         '17 AFIDL inputs and catalog roots exist; current definition cannot recover unretained construction input'),
        ('Source Reference', True, True, True, False, False, False,
         'AF-SEMANTIC-0200/0265/0266; association preserves content, no missing schema rule'),
        ('Compiler Source Identity', True, True, True, False, False, False,
         'AF-SEMANTIC-0198; complete current normative/compiler bytes, not snapshot creation receipt'),
        ('Context Relation Snapshot', True, True, True, False, True, False,
         'AF-SEMANTIC-0262; actual context may omit relation module; no fallback permitted'),
        ('Portable relation header', True, True, True, True, True, True,
         'portable-proof-inputs.afidl + af-portable/reconstruction.rs build_relation: explicitly supplied full pin checked against current modules; not native source conversion'),
    ]
    return [dict(candidate=n, Registered='Yes' if reg else 'No',
        SemanticallyApplicable='Yes' if app else 'No', ExactSource='Present' if src else 'Missing',
        TypeMapping='Complete' if mapping else 'Incomplete', ConstructionHistory='Unrecoverable',
        CurrentContextDependence='Required' if ctx else 'Not Required', Lossless='Yes' if loss else 'No',
        source=source, existing_native_to_formal_binding=False)
        for n,reg,app,src,mapping,ctx,loss,source in rows]


def field_sources():
    rows = [
      ('source.kind', 'actual named optional slot + concrete T', 'borrowed source', 'draft closed kind, owner unallocated', 'effect_resource | recovery_usage | flow_execution', 'no digest', 'slot/T/family/root all agree', 'required'),
      ('source.pin.family', 'snapshot.schema() and snapshot.pin().schema and T::SCHEMA_ID', 'borrowed source', 'draft family nominal wrapping existing SpecificationId grammar', 'strict exact SpecificationId', 'NOT TypeSchemaDigest', 'all three byte-equal; no alias normalization', 'required'),
      ('source.pin.revision', 'snapshot.revision() and snapshot.pin().revision', 'borrowed source', 'draft source pin', 'nonempty NFC text', 'NOT schema identity', 'both exact; expected revision only if an actual consumer supplies one', 'required'),
      ('source.pin.checksum.algorithm', 'snapshot.pin().canonical_content_checksum.algorithm', 'borrowed source + integrity check', 'af-contract source', 'sha256 literal', 'plain SHA256', 'no HashV1 domain', 'required'),
      ('source.pin.checksum.hex', 'snapshot.pin().canonical_content_checksum.hex', 'borrowed source + integrity check', 'af-contract source', '64 lowercase hex', 'SHA256(full canonical bytes)', 'independently recompute and compare', 'required'),
      ('source.canonical_content', 'snapshot.canonical_bytes()', 'borrowed source', 'draft exact bytes', 'complete bytes', 'content checksum only', 'never count/hex-summary substitute', 'required'),
      ('source.catalog', 'snapshot.catalog() full T', 'borrowed source', 'draft closed per-kind native catalog projection', 'all native catalog fields/variants', 'none invented', 'to_value/canonicalization/bytes must agree in C validation', 'required'),
      ('construction', 'no stored value in legacy native snapshot', 'construction history', 'not selected; future provenance type requires separate authorization', 'Unavailable, not guessed TypeSchemaRef', 'cannot recover from family/revision/content', 'a future authentic receipt would bind actual constructor input and source occurrence', 'Unavailable'),
      ('validation.schema.specification_id', 'uniquely selected actual_context.modules root.schema', 'current validation output only', 'existing TypeSchemaRef shape in successor-owned observation', 'exact String representing SpecificationId', 'included in canonical TypeSchema', 'equals source family AND slot family', 'only full lookup result; never default'),
      ('validation.schema.type_name', 'uniquely selected root.name, checked against closed kind root', 'current validation output only', 'af-semantic::TypeSchemaRef shape', 'exact String', 'included in canonical TypeSchema', 'closed table selects query but never supplies missing evidence', 'only successful resolution'),
      ('validation.schema.schema_digest', 'actual full canonical TypeSchema encoded independently', 'current validation output only', 'af-semantic::DigestRefValue shape', '{algorithm,domain,hex}', 'sha256 / type-schema / SHA256(AF\\0type-schema\\0 || ACE1(schema))', 'complete imported closure also retained in input context', 'resolved pin may occur on Invalid; admission requires Valid'),
      ('validation.association', 'full source snapshot + exact context input reference + closed slot + actual validation algorithm', 'current validation output', 'draft observation, no allocated edition', 'typed refs to preceding immutable input nodes', 'no context digest proves aliases', 'analysis/canonical_validation same actual object; distinct contexts stay distinct classes', 'mandatory'),
      ('raw.Source/Origin', 'unchanged occurrence-bound Raw input/owner/document and complete nine-member context', 'Raw capture', 'successor Source.raw_validation/RawValidationOrigin', 'full typed subject; no added resolver', 'compiler/context full values', 'all occurrences reuse one source binding; no diagnostic labels as identity', 'incomplete today'),
      ('raw.Target/Coordinate', 'actual full compiled ComponentContract schema + AF-CONTRACT-0004; reached side/original/sorted U32 entry', 'actual port branch', 'successor Target/Coordinate exact existing nominal owners', 'not the relation schema target', 'real ComponentContract digest retained', 'relation binding cannot overwrite port constraint or predecessor', 'incomplete today'),
      ('raw.Basis/Witness/EvidenceRef', 'same subject and actual sealed branch evidence; duplicate both real members', 'output graph', 'successor RawPortCheckBasis/typed contradiction/evidence', 'typed references only; primary typed minimum', 'no report reason supplies pin', 'input references strictly prior; no observation as Source prerequisite', 'incomplete today'),
    ]
    keys=['field','ActualSourceEvidence','SourceStage','NominalOwner','ExactSchemaType','CompleteDigest','AssociationRule','ValidationStatus']
    return [dict(zip(keys,row)) for row in rows]


def vectors():
    # These are independently hand-specified expected design judgments, NOT
    # outputs of a newly implemented production validation algorithm.
    groups = [
      ('family-construction', [
       ('A/B empty catalog', 'same family/revision, A=list<Pair>, B=list<string>, both empty', 'native values equal; construction provenance unavailable; current-A observations equal'),
       ('Digest versus bytes', 'A/B distinct full TypeSchema; equal catalog ACE1', 'schema digests differ; checksums equal; no hash collision'),
       ('Revision is opaque', 'same family and binding-probe revision under A/B', 'revision cannot select one definition'),
       ('Missing construction receipt', 'legacy snapshot stores five native fields only', 'construction Unavailable; never infer A'),
       ('Explicit authentic receipt', 'hypothetical future constructor retains exact A schema and source association', 'B design could preserve A; not accepted as legacy A+C proof or implemented here'),
       ('False receipt', 'caller labels old snapshot constructed-under-A without evidence', 'reject construction claim; no authority from tag'),
       ('Wrong schema name', 'source kind effect_resource; proposed construction root RecoveryUsageCoverageCatalog', 'reject kind/root association'),
       ('Family grammar', 'family agentflow.effect.resource.compatibility@01', 'reject source: noncanonical SpecificationId'),
       ('Revision invalid', 'empty or non-NFC revision', 'reject source integrity, not port Violation'),
       ('Checksum corruption', 'full native bytes unchanged; checksum hex changed', 'reject source integrity'),
       ('Changed full catalog', 'nonempty supported_pairs added but retained empty catalog bytes', 'current re-encoding invalid; no valid observation'),
       ('Native equality preserved', 'native A and native B compared in old edition', 'true remains true; no construction-sensitive rewrite'),
      ]),
      ('current-validation', [
       ('Current A valid', 'source empty effect catalog; actual current modules A', 'Found then Valid; exact current pin A, no construction claim'),
       ('Current missing', 'genuine source Some; actual current modules omit relation module', 'Missing; no pin; full Subject incomplete; raw zero remains Violation'),
       ('Ambiguous inputs', 'portable/imported input contains duplicate family roots', 'reject context association as Ambiguous before validation; native CompiledModuleSet cannot represent it'),
       ('Family mismatch', 'source pin family differs from selected slot family', 'FamilyMismatch; no current lookup fallback'),
       ('Revision mismatch', 'actual consuming comparison expects revision r1; source revision r2', 'RevisionMismatch for that comparison; schema has no catalog revision to guess'),
       ('Current incompatible', 'source Pair-valued nonempty supported_pairs; actual current schema B list<string>', 'Found then Invalid during real shape validation; no Valid pin'),
       ('Current B empty', 'equal empty native source; actual modules B', 'Valid pin B if full round-trip/catalog checks succeed; distinct current observation, not distinct construction history'),
       ('Across contexts', 'one equal source; current A versus current B', 'source equality unchanged; current schema observations distinct'),
       ('Missing import', 'actual current root references absent external type', 'reject/incomplete module closure before catalog validation; no global import'),
       ('Not reached', 'context/source association fails before lookup', 'later schema/catalog validation NotReached; not automatic Invalid'),
       ('Invalid catalog semantics', 'shape-valid rows contain duplicate exact catalog keys', 'actual catalog validation Invalid with structured offending rows'),
       ('Found not valid', 'lookup found root but round-trip never executed', 'Found alone cannot discharge full-schema consumer or Subject admission'),
      ]),
      ('slots-and-pins', [
       ('Effect Some', 'effect_resource Some correct kind/family/root/current modules', 'project full EffectResourceCompatibilityCatalog; require C Valid'),
       ('Recovery Some', 'recovery_usage Some correct kind/family/root/current modules', 'project full RecoveryUsageCoverageCatalog; require C Valid; no dynamic credit here'),
       ('Flow Some', 'flow_execution Some correct kind/family/root/current modules', 'project full InformationFlowExecutionCatalog; require C Valid; no dynamic credit here'),
       ('All None', 'all three real source options None', 'preserve three None; no invented validation success or relation coverage'),
       ('Wrong nominal owner', 'same structural fields supplied as unrelated model-owned schema ref', 'reject nominal association, no structural duck typing'),
       ('Wrong root name', 'effect source pin accompanied by TypeSchemaRef for another current type', 'reject exact root association'),
       ('Wrong digest', 'correct family/name; schema digest changed one hex nibble', 'reject complete schema pin mismatch'),
       ('Checksum as schema digest', 'catalog SHA256 substituted for TypeSchemaDigest', 'reject: distinct preimage and domain'),
       ('Same contents not alias', 'analysis/canonical_validation refer to equal but distinct objects', 'reject role association; equality cannot establish pointer alias'),
       ('Swapped slot', 'recovery source placed in effect_resource slot', 'reject closed slot/kind/family association'),
       ('Two Some one missing', 'effect valid and recovery missing current schema', 'effect observation may be Valid; complete Subject admission still incomplete'),
       ('Other eight members', 'relation binding valid but usage_meters content omitted', 'reject incomplete nine-member Subject; relation design cannot repair rest'),
      ]),
      ('migration-dag-capability', [
       ('Old source lacks history', 'old five-field snapshot asks construction-sensitive migration', 'not migratable without genuine retained receipt; current validation is not history'),
       ('Cross-edition', 'family source draft labeled as frozen model@5 RelationSnapshotContent', 'reject UnsupportedModel/type mismatch; no relabel'),
       ('Unknown variant', 'unregistered source or validation tag in current edition', 'reject closed descriptor; draft tag is not supported'),
       ('Source-to-observation cycle', 'input snapshot points to its derived C validation output', 'reject cycle; output may point to input only'),
       ('Witness-to-input cycle', 'source identity includes Raw contradiction referencing same Fact', 'reject; canonical input DAG excludes report output'),
       ('Raw zero with missing schema', 'actual reached input port_no=0, current schema Missing', 'local Violation remains; no full typed Fact with fabricated relation pin'),
       ('Raw positive', 'actual fresh port>0; relation validation Invalid', 'local Proven remains; full Subject admission blocked; no whole-contract Proven'),
       ('Duplicate witness', 'two actual duplicate sorted ports and valid relation observation', 'both original member references retained; no reason parsing or schema substitution'),
       ('Current-only routing', 'draft observation supplied to existing D4/current route', 'deny unsupported successor; no automatic activation'),
       ('Historical Expected', 'migration wants recomputed old Expected with family-only schema', 'deny rewrite; frozen old exact Expected retained'),
       ('Positive RootExport', 'all G1 documents and 48 design vectors exist', 'NotAuthorized; T4 NOT ELIGIBLE; strict Raw Total 2'),
       ('No registration authority', 'design Outcome A complete', 'registration pending; no edition/RuleId allocation; C2R3 remains incomplete'),
      ]),
    ]
    rows = []
    for category, cases in groups:
        assert len(cases) == 12
        for title, inputs, expected in cases:
            rows.append(dict(id='G1-D%02d' % (len(rows)+1), category=category,
                title=title, independent_input=inputs, hand_derived_expected=expected,
                ActualStatus='Not Executed', actual=None, production_conformance_credit=0))
    assert len(rows) == 48
    return dict(kind='DESIGN ONLY; artifact audit is not vector execution', vectors=rows)


def source_inventory():
    pattern = r'RelationCatalogPin|RelationSnapshotContent|relation_snapshots|RelationCatalogSnapshot|RelationSnapshotSlots'
    cmd = ['rg', '-n', pattern, 'crates', 'spec/component', 'spec/computation', 'spec/idl', '-g', '*.rs', '-g', '*.md', '-g', '*.afidl']
    result = subprocess.run(cmd, cwd=R, capture_output=True, text=True, encoding='utf8', check=True)
    hits = sorted(result.stdout.splitlines())
    files = sorted({line.split(':', 1)[0].replace('\\', '/') for line in hits})
    return dict(search_command=cmd, hits=hits,
        sources=[dict(path=p, raw_sha256=hashlib.sha256((R/p).read_bytes()).hexdigest()) for p in files])


def native_and_rule_inventory():
    path = 'crates/af-contract/src/universe.rs'
    text = (R/path).read_text(encoding='utf8')
    names = ['ScopeValueRule', 'SelectorKeyRule', 'EffectQualifierProfile',
        'SupportedEffectResourcePair', 'EffectResourceCompatibilityCatalog',
        'RecoveryUsageCoverageRule', 'RecoveryUsageCoverageCatalog', 'RegionPair',
        'RegionPairPolicy', 'TrustDomainPolicy', 'SandboxFloor',
        'ConfidentialityExecutionRule', 'TaintExecutionRule', 'AuthorityExecutionRule',
        'FlowExecutionSink', 'FlowExecutionRule', 'InformationFlowExecutionCatalog']
    declarations = []
    for name in names:
        match = re.search(r'pub (?:struct|enum) '+name+r' \{', text)
        assert match, name
        start = match.start(); end = text.index('\n}', match.end()) + 2
        declarations.append(dict(name='af-contract::'+name, source=path,
            line=text.count('\n', 0, start)+1, exact_declaration=text[start:end]))
    rules = json.loads((R/'spec/registry/rules.json').read_bytes())['rules']
    assert len(rules) == 493
    covered = [r for r in rules if r['conformance_refs']]
    assert len(covered) == 492
    specs = json.loads((R/'spec/registry/specifications.json').read_bytes())
    assert len(specs['specifications']) == 62
    abi = json.loads((R/'spec/registry/abi-numbers.json').read_bytes())
    assert sum(a['status']=='active' for a in abi['assignments']) == 775
    domains = json.loads((R/'spec/tables/hash-domains.json').read_bytes())
    assert len(domains['domains']) == 14
    wanted = {'AF-SEMANTIC-%04d' % n for n in [198,200,206,208,209,210,211,212,261,262,263,264,265,266,267,268]}
    wanted |= {'AF-CONTRACT-%04d' % n for n in [4,13,16,18,19]}
    selected = [r for r in rules if r['rule_id'] in wanted]
    assert len(selected) == len(wanted)
    return dict(native_catalog_owner='af-contract', declarations=declarations,
        source_sha256=hashlib.sha256((R/path).read_bytes()).hexdigest(),
        registered_rules=selected, specifications=62, rules=493, covered=492,
        intentional_exceptions=[r['rule_id'] for r in rules if not r['conformance_refs']],
        coverage_summary=json.loads((R/'spec/registry/coverage-summary.json').read_bytes()),
        abi_active=775, afidl=len(list((R/'spec/idl').rglob('*.afidl'))), hash_domains=14,
        exact_hash_domain_registry=domains)


def artifacts():
    m = read_model()
    sp = importlib.util.spec_from_file_location('independent_g1_gap', R/'tools/oracle/raw_relation_schema_binding.py')
    oracle = importlib.util.module_from_spec(sp); sp.loader.exec_module(oracle)
    return {
        'descriptor-census.json': descriptors(m),
        'three-slot-matrix.json': slots(m),
        'candidate-matrix.json': dict(binding_candidates=binding_candidates(),
            options=[dict(option='A', source='exact legacy family', current_schema_precision='insufficient alone', selected='source layer only'),
                     dict(option='B', source='new authentic construction receipt', current_schema_precision='historical, not current', selected=False),
                     dict(option='C', source='actual current context and executed revalidation', current_schema_precision='exact current TypeSchemaRef', selected='validation layer only')],
            unique_recommendation='A+C; separate immutable source and derived current validation; B not selected'),
        'field-source-matrix.json': dict(status='unregistered amendment DESIGN', fields=field_sources()),
        'design-vectors.json': vectors(),
        'minimal-expected.json': oracle.expected(),
        'source-inventory.json': source_inventory(),
        'native-and-rule-inventory.json': native_and_rule_inventory(),
    }


def main():
    out = artifacts()
    if sys.argv[1:] == ['--materialize']:
        for name, value in out.items():
            (D/(P+name)).write_text(json.dumps(value, indent=2, ensure_ascii=True)+'\n', encoding='utf8')
    else:
        assert not sys.argv[1:]
        for name, value in out.items():
            assert json.loads((D/(P+name)).read_bytes()) == value, name
    # No production judgment is executed here. Count/shape checks cannot promote vectors.
    assert all(v['ActualStatus']=='Not Executed' and v['actual'] is None
               for v in out['design-vectors.json']['vectors'])
    print(json.dumps(dict(descriptors=704, references=2064, selected_imports=97,
        slots=[s['slot'] for s in out['three-slot-matrix.json']['slots']],
        design_vectors=48, ActualStatus='Not Executed', production_vectors_executed=0,
        existing_exact_native_binding=False, raw_total_exit=2, t4='NOT ELIGIBLE')))


if __name__ == '__main__':
    main()
